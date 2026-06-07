// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Cross-actor `{{key}}` resolution (M9-D8 / Checkpoint #1).
//!
//! A [`Registry`] is shared across all actor drivers in a scenario run. When an
//! actor's command names an export ([`crate::manifest::Export`]), the driver
//! [`publish`](Registry::publish)es the reply field under the `{{key}}` name.
//! Before sending a command whose args contain `{{key}}` placeholders, the
//! driver [`wait_for`](Registry::wait_for)s each key (blocking until the
//! producing actor publishes it), then [`substitute`]s the resolved values in.
//! This data-dependency edge is what orders the run.
//!
//! The placeholder grammar is deliberately tiny: `{{` + a key of
//! `[A-Za-z0-9_]+` + `}}`. Keys appear inside JSON string values (the only place
//! the worked-example scenarios use them); substitution is a plain string
//! replace, so a placeholder that *is* the whole value yields the bare id, and
//! an embedded placeholder is spliced in place.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, Notify};
use tokio::time::Instant;

use crate::Result;
use anyhow::anyhow;

/// A shared publish/await registry for cross-actor `{{key}}` values.
#[derive(Clone, Default)]
pub struct Registry {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    map: Mutex<HashMap<String, String>>,
    notify: Notify,
}

impl Registry {
    pub fn new() -> Registry {
        Registry::default()
    }

    /// Publish a value under `key`, waking any waiters. Re-publishing overwrites
    /// (the last writer wins; scenarios export a key from exactly one command).
    pub async fn publish(&self, key: impl Into<String>, value: impl Into<String>) {
        {
            let mut map = self.inner.map.lock().await;
            map.insert(key.into(), value.into());
        }
        self.inner.notify.notify_waiters();
    }

    /// Non-blocking lookup.
    pub async fn get(&self, key: &str) -> Option<String> {
        self.inner.map.lock().await.get(key).cloned()
    }

    /// Block until `key` is published or `timeout` elapses.
    pub async fn wait_for(&self, key: &str, timeout: Duration) -> Result<String> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(v) = self.get(key).await {
                return Ok(v);
            }
            // Arm the notification *before* re-checking would race; `Notified`
            // created here catches any publish that lands while we await.
            let notified = self.inner.notify.notified();
            if let Some(v) = self.get(key).await {
                return Ok(v);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(anyhow!(
                    "timed out after {timeout:?} waiting for cross-actor key `{{{{{key}}}}}` \
                     (no actor exported it)"
                ));
            }
            // Wait for either a publish wake-up or the remaining time.
            let _ = tokio::time::timeout(remaining, notified).await;
        }
    }
}

/// Extract the distinct `{{key}}` placeholder names appearing in `s`, in first-
/// seen order. A malformed `{{` without a closing `}}` is ignored.
pub fn placeholders_in(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(rel_end) = s[i + 2..].find("}}") {
                let key = &s[i + 2..i + 2 + rel_end];
                if !key.is_empty()
                    && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && !out.iter().any(|k| k == key)
                {
                    out.push(key.to_string());
                }
                i = i + 2 + rel_end + 2;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Replace every `{{key}}` in `s` with its value from `resolved`. Keys absent
/// from `resolved` are left untouched (the caller resolves them first via the
/// [`Registry`]).
pub fn substitute(s: &str, resolved: &HashMap<String, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in resolved {
        out = out.replace(&format!("{{{{{k}}}}}"), v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_extracts_keys_in_order_and_distinct() {
        let s = r#"{"space":"{{space_id}}","invite_event":"{{invite_event_id}}","again":"{{space_id}}"}"#;
        assert_eq!(placeholders_in(s), vec!["space_id", "invite_event_id"]);
    }

    #[test]
    fn placeholders_ignores_unclosed_and_non_key_chars() {
        assert!(placeholders_in("{{ not a key }}").is_empty());
        assert!(placeholders_in("{{unterminated").is_empty());
        assert_eq!(placeholders_in("a {{good_key1}} b"), vec!["good_key1"]);
    }

    #[test]
    fn substitute_replaces_whole_value_and_embedded() {
        let mut m = HashMap::new();
        m.insert("space_id".to_string(), "xgen://hash/sha256:S".to_string());
        m.insert("bob".to_string(), "BOB".to_string());
        let s = r#"{"space":"{{space_id}}","note":"hi {{bob}}!"}"#;
        assert_eq!(
            substitute(s, &m),
            r#"{"space":"xgen://hash/sha256:S","note":"hi BOB!"}"#
        );
    }

    #[tokio::test]
    async fn registry_publish_then_wait_resolves() {
        let reg = Registry::new();
        reg.publish("space_id", "S").await;
        let v = reg.wait_for("space_id", Duration::from_millis(100)).await.unwrap();
        assert_eq!(v, "S");
    }

    #[tokio::test]
    async fn registry_wait_blocks_until_published() {
        let reg = Registry::new();
        let r2 = reg.clone();
        let waiter = tokio::spawn(async move {
            r2.wait_for("late", Duration::from_secs(2)).await
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        reg.publish("late", "value").await;
        let got = waiter.await.unwrap().unwrap();
        assert_eq!(got, "value");
    }

    #[tokio::test]
    async fn registry_wait_times_out_when_never_published() {
        let reg = Registry::new();
        let r = reg.wait_for("never", Duration::from_millis(50)).await;
        assert!(r.is_err());
        assert!(format!("{:#}", r.unwrap_err()).contains("never"));
    }
}
