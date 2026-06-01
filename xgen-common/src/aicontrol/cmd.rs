// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! AC-D1 `cmd` resolution — reserved control verbs first, then the CLI path.
//!
//! The `cmd` field is the CLI command path minus the binary name. The
//! dispatcher splits on the **first space** (space = structural category/verb
//! separator; hyphen = intra-token, never a split point — this dissolves the
//! `auth-module` collision because no token contains a space). Client verbs
//! are single-token (`category: None`); Node verbs are `category verb`.
//!
//! The category/verb split lives here, in the dispatcher, **not** on the wire
//! (AC-D1 rationale 2): the wire carries an opaque command-path string,
//! decoupled from clap's internal type grouping.

/// A reserved control verb handled by the control surface itself (not a
/// CLI/admin op). v1 command-pipe control-verb set: just `state`.
/// (`subscribe` / `unsubscribe` live on the events pipe, §3 — they are never
/// resolved here.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlVerb {
    /// The in-process `state` verb (§9 / AC-D3c).
    State,
}

/// A CLI-path command after splitting `cmd` on the first space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdPath {
    /// The category token (Node verbs). `None` for single-token Client verbs.
    pub category: Option<String>,
    /// The verb token (a single hyphenated word, e.g. `set-node-policy`).
    pub verb: String,
}

/// The result of resolving a `cmd` string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmdResolution {
    /// A reserved control verb, handled by the control surface itself.
    Control(ControlVerb),
    /// A CLI-path command — the binary's arm maps it onto `ops::*` /
    /// `admin_ops::*` (and answers `UNKNOWN_COMMAND` if it has no such verb).
    Cli(CmdPath),
}

/// Resolve a `cmd` string (AC-D1). Reserved control verbs are matched first;
/// everything else splits on the first space into a [`CmdPath`].
pub fn resolve_cmd(cmd: &str) -> CmdResolution {
    let cmd = cmd.trim();

    // Reserved control verbs first (AC-D1 §3 carve-out).
    if cmd == "state" {
        return CmdResolution::Control(ControlVerb::State);
    }

    match cmd.split_once(' ') {
        Some((category, verb)) => CmdResolution::Cli(CmdPath {
            category: Some(category.to_string()),
            verb: verb.trim().to_string(),
        }),
        None => CmdResolution::Cli(CmdPath {
            category: None,
            verb: cmd.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli(cmd: &str) -> CmdPath {
        match resolve_cmd(cmd) {
            CmdResolution::Cli(p) => p,
            other => panic!("expected Cli, got {other:?}"),
        }
    }

    #[test]
    fn client_single_token_verb() {
        let p = cli("send");
        assert_eq!(p.category, None);
        assert_eq!(p.verb, "send");
    }

    #[test]
    fn node_two_token_verb() {
        let p = cli("federation accept");
        assert_eq!(p.category.as_deref(), Some("federation"));
        assert_eq!(p.verb, "accept");
    }

    #[test]
    fn hyphenated_verb_token_is_not_split() {
        let p = cli("space set-node-policy");
        assert_eq!(p.category.as_deref(), Some("space"));
        assert_eq!(p.verb, "set-node-policy");
    }

    #[test]
    fn auth_module_two_hyphen_case_splits_on_space_only() {
        // The `auth-module` collision: hyphen is intra-token, only the space
        // separates category from verb.
        let p = cli("auth-module register");
        assert_eq!(p.category.as_deref(), Some("auth-module"));
        assert_eq!(p.verb, "register");
    }

    #[test]
    fn state_resolves_as_reserved_control_verb_before_cli() {
        assert_eq!(resolve_cmd("state"), CmdResolution::Control(ControlVerb::State));
    }

    #[test]
    fn state_with_trailing_text_is_not_the_reserved_verb() {
        // Only exact `state` is reserved; anything else is a CLI path the arm
        // will reject as UNKNOWN_COMMAND (there is no `state` category).
        let p = cli("state foo");
        assert_eq!(p.category.as_deref(), Some("state"));
        assert_eq!(p.verb, "foo");
    }

    #[test]
    fn surrounding_whitespace_is_trimmed() {
        let p = cli("  federation   accept  ");
        assert_eq!(p.category.as_deref(), Some("federation"));
        assert_eq!(p.verb, "accept");
    }
}
