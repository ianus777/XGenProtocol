// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! §5 named variable bindings — per-connection namespace + `$`-substitution.
//!
//! Named bindings are mandatory; there is no implicit `@last_*` in v1. A
//! `bind:"foo"` names a successful command's result; `$foo` in a later
//! argument resolves to the command's *primary* return, and `$foo.field`
//! reaches the other result fields by dot-notation. Substitution is
//! substring-level inside JSON string values, simple dot-notation only, no
//! expressions. Bindings are scoped to the pipe connection (a fresh
//! connection starts empty). Unknown binding → [`ControlCode::BindingNotFound`].

use std::collections::HashMap;

use serde_json::{Map, Value};

use super::codes::{ControlCode, ControlError};

/// One bound command result. `primary` is what bare `$name` resolves to (the
/// command's primary return, e.g. `space_id`); `fields` is the full result
/// object for `$name.field` dot-notation access.
#[derive(Debug, Clone)]
pub struct BoundValue {
    pub primary: Value,
    pub fields: Map<String, Value>,
}

/// A per-connection binding namespace (§5).
#[derive(Debug, Clone, Default)]
pub struct Bindings {
    map: HashMap<String, BoundValue>,
}

impl Bindings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful command's result under `name` (its `bind` target).
    /// `primary` is the bare-`$name` value; `fields` backs dot-notation.
    /// Overwrites any prior binding of the same name.
    pub fn set(&mut self, name: impl Into<String>, primary: Value, fields: Map<String, Value>) {
        self.map.insert(name.into(), BoundValue { primary, fields });
    }

    pub fn get(&self, name: &str) -> Option<&BoundValue> {
        self.map.get(name)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// The names → primary-value map, for the `state` reply's `bindings`
    /// field (§9).
    pub fn snapshot(&self) -> Map<String, Value> {
        self.map
            .iter()
            .map(|(k, v)| (k.clone(), v.primary.clone()))
            .collect()
    }
}

/// Substitute `$name` / `$name.field` references in every (possibly nested)
/// string value of `args` against `bindings` (§5). An unknown binding or
/// unknown field → [`ControlCode::BindingNotFound`].
pub fn substitute(args: &mut Map<String, Value>, bindings: &Bindings) -> Result<(), ControlError> {
    for v in args.values_mut() {
        substitute_value(v, bindings)?;
    }
    Ok(())
}

fn substitute_value(v: &mut Value, bindings: &Bindings) -> Result<(), ControlError> {
    match v {
        Value::String(s) if s.contains('$') => {
            *v = substitute_in_string(s, bindings)?;
        }
        Value::Array(items) => {
            for item in items.iter_mut() {
                substitute_value(item, bindings)?;
            }
        }
        Value::Object(map) => {
            for item in map.values_mut() {
                substitute_value(item, bindings)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Replace every `$name` / `$name.field` token in `s`. When the entire string
/// is exactly one token, the bound value's type is preserved (a numeric or
/// boolean field substitutes as that JSON type); otherwise the result is a
/// string with each token rendered in its substitution form.
fn substitute_in_string(s: &str, bindings: &Bindings) -> Result<Value, ControlError> {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let mut token_count = 0usize;
    let mut had_literal = false;
    let mut whole_string_token: Option<Value> = None;

    while i < n {
        if chars[i] == '$' {
            let name_start = i + 1;
            let mut j = name_start;
            while j < n && is_ident_char(chars[j]) {
                j += 1;
            }
            if j == name_start {
                // Lone `$` not followed by an identifier — literal text.
                out.push('$');
                had_literal = true;
                i += 1;
                continue;
            }
            let name: String = chars[name_start..j].iter().collect();

            // Optional `.field`. A dot not followed by an identifier is literal.
            let mut field: Option<String> = None;
            let mut end = j;
            if j < n && chars[j] == '.' {
                let fstart = j + 1;
                let mut k = fstart;
                while k < n && is_ident_char(chars[k]) {
                    k += 1;
                }
                if k > fstart {
                    field = Some(chars[fstart..k].iter().collect());
                    end = k;
                }
            }

            let resolved = lookup(&name, field.as_deref(), bindings)?;
            token_count += 1;
            out.push_str(&value_to_subst_string(&resolved));
            if i == 0 && end == n {
                whole_string_token = Some(resolved);
            }
            i = end;
        } else {
            out.push(chars[i]);
            had_literal = true;
            i += 1;
        }
    }

    // Type-preserve only when exactly one token spanned the whole string.
    if token_count == 1 && !had_literal {
        if let Some(v) = whole_string_token {
            return Ok(v);
        }
    }
    Ok(Value::String(out))
}

fn lookup(name: &str, field: Option<&str>, bindings: &Bindings) -> Result<Value, ControlError> {
    let bound = bindings.get(name).ok_or_else(|| {
        ControlError::new(
            ControlCode::BindingNotFound,
            format!("no binding named `{name}` in this session"),
        )
    })?;
    match field {
        None => Ok(bound.primary.clone()),
        Some(f) => bound.fields.get(f).cloned().ok_or_else(|| {
            ControlError::new(
                ControlCode::BindingNotFound,
                format!("binding `{name}` has no field `{f}`"),
            )
        }),
    }
}

/// Render a bound value for substring substitution: a JSON string contributes
/// its inner text (no quotes); other JSON scalars contribute their compact form.
fn value_to_subst_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bindings_with_space_and_send() -> Bindings {
        let mut b = Bindings::new();
        // create-space → primary space_id, plus event_id field.
        b.set(
            "space",
            json!("xgen://hash/sha256:S"),
            json!({"space_id": "xgen://hash/sha256:S", "event_id": "xgen://hash/sha256:E"})
                .as_object()
                .unwrap()
                .clone(),
        );
        b
    }

    #[test]
    fn whole_value_reference_substitutes_primary() {
        let b = bindings_with_space_and_send();
        let mut args = json!({"space": "$space"}).as_object().unwrap().clone();
        substitute(&mut args, &b).unwrap();
        assert_eq!(args["space"], "xgen://hash/sha256:S");
    }

    #[test]
    fn dot_notation_reaches_other_fields() {
        let b = bindings_with_space_and_send();
        let mut args = json!({"parent": "$space.event_id"}).as_object().unwrap().clone();
        substitute(&mut args, &b).unwrap();
        assert_eq!(args["parent"], "xgen://hash/sha256:E");
    }

    #[test]
    fn embedded_token_does_substring_substitution() {
        let b = bindings_with_space_and_send();
        let mut args = json!({"label": "in-$space-room"}).as_object().unwrap().clone();
        substitute(&mut args, &b).unwrap();
        assert_eq!(args["label"], "in-xgen://hash/sha256:S-room");
    }

    #[test]
    fn whole_value_reference_preserves_non_string_type() {
        let mut b = Bindings::new();
        b.set(
            "stats",
            json!("S"),
            json!({"member_count": 3}).as_object().unwrap().clone(),
        );
        let mut args = json!({"limit": "$stats.member_count"}).as_object().unwrap().clone();
        substitute(&mut args, &b).unwrap();
        assert_eq!(args["limit"], json!(3), "numeric field preserves its type");
        assert!(args["limit"].is_number());
    }

    #[test]
    fn unknown_binding_is_binding_not_found() {
        let b = Bindings::new();
        let mut args = json!({"space": "$ghost"}).as_object().unwrap().clone();
        let e = substitute(&mut args, &b).unwrap_err();
        assert_eq!(e.code, ControlCode::BindingNotFound);
    }

    #[test]
    fn unknown_field_is_binding_not_found() {
        let b = bindings_with_space_and_send();
        let mut args = json!({"x": "$space.nope"}).as_object().unwrap().clone();
        let e = substitute(&mut args, &b).unwrap_err();
        assert_eq!(e.code, ControlCode::BindingNotFound);
    }

    #[test]
    fn nested_args_are_substituted() {
        let b = bindings_with_space_and_send();
        let mut args = json!({"wrap": {"inner": "$space"}, "list": ["$space"]})
            .as_object()
            .unwrap()
            .clone();
        substitute(&mut args, &b).unwrap();
        assert_eq!(args["wrap"]["inner"], "xgen://hash/sha256:S");
        assert_eq!(args["list"][0], "xgen://hash/sha256:S");
    }

    #[test]
    fn strings_without_dollar_are_untouched() {
        let b = bindings_with_space_and_send();
        let mut args = json!({"text": "hello world"}).as_object().unwrap().clone();
        substitute(&mut args, &b).unwrap();
        assert_eq!(args["text"], "hello world");
    }

    #[test]
    fn snapshot_maps_names_to_primary_values() {
        let b = bindings_with_space_and_send();
        let snap = b.snapshot();
        assert_eq!(snap["space"], "xgen://hash/sha256:S");
        assert_eq!(b.len(), 1);
        assert!(!b.is_empty());
    }
}
