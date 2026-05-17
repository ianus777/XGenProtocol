// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Shared command implementations for all xgen-client dispatchers (M5, D-067).
//!
//! Each function takes `&mut OpContext<'_>` plus command-specific args and
//! returns `Result<<Verb>Result>` where the result struct carries pure data.
//! The CLI dispatcher (`app::cmd_*` shims) formats results for stdout; the
//! pipe dispatcher (`batch::dispatch_line` arms) drives the D-066-frozen
//! `OK\n` / `ERROR: …\n` pipe shape and discards data; a future
//! `--aicontrol` surface (M7) will format the same results as JSONL.
//!
//! Per-verb migrations land as atomic commits per the M5 contract (see
//! `tasks/M5_OPS_REFACTOR.md` §3 and `tasks/BATCH_FLAG_review.md` Chat
//! Claude addendum §7). This file grows by one function per commit until
//! all 13 verbs are routed through it.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::session::SessionState;

/// Per-command execution context. Constructed by each dispatcher per
/// invocation in M5; the same instance will be reused across commands
/// in a persistent `--aicontrol` connection in M7.
pub struct OpContext<'a> {
    pub session: &'a mut SessionState,
    pub data_dir: &'a Path,
    pub node_override: Option<&'a str>,
}

// ── whoami ────────────────────────────────────────────────────────────────────

/// Result of `ops::whoami`. Flat field-by-field so any dispatcher can format
/// it for its own output channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoamiResult {
    pub identity_id: String,
    pub display_name: String,
    pub home_node: String,
    pub spaces_joined: usize,
}

/// Read the on-disk client state and project the whoami subset.
///
/// Pure data extraction — no println, no pipe writes. Replaces the data
/// half of the historical `cmd_whoami` (the println half lives in the
/// CLI shim in `app.rs`).
pub fn whoami(ctx: &mut OpContext<'_>) -> Result<WhoamiResult> {
    let state = crate::app::load_client_state(ctx.data_dir)?;
    Ok(WhoamiResult {
        identity_id: state.identity_id,
        display_name: state.display_name,
        home_node: state.home_node,
        spaces_joined: state.spaces.len(),
    })
}

// ── status ────────────────────────────────────────────────────────────────────

/// Result of `ops::status`. Wider field set than `WhoamiResult` — includes
/// `version` and the age (in seconds) of the on-disk state file so the
/// CLI shim can format the staleness warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResult {
    pub identity_id: String,
    pub display_name: String,
    pub version: String,
    pub home_node: String,
    pub spaces_joined: usize,
    pub state_file_age_seconds: i64,
}

/// Read the on-disk client state and compute the staleness age.
pub fn status(ctx: &mut OpContext<'_>) -> Result<StatusResult> {
    let state = crate::app::load_client_state(ctx.data_dir)?;
    let age = crate::app::age_seconds(&state.updated_at);
    Ok(StatusResult {
        identity_id: state.identity_id,
        display_name: state.display_name,
        version: state.version,
        home_node: state.home_node,
        spaces_joined: state.spaces.len(),
        state_file_age_seconds: age,
    })
}

// ── spaces ────────────────────────────────────────────────────────────────────

/// Result of `ops::spaces`. Carries the known-Space list straight from
/// disk; the CLI shim formats the indented per-Space / per-Room printout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacesResult {
    pub spaces: Vec<xgen_common::state::KnownSpace>,
}

/// Read the on-disk client state and return the known-Space list verbatim.
pub fn spaces(ctx: &mut OpContext<'_>) -> Result<SpacesResult> {
    let state = crate::app::load_client_state(ctx.data_dir)?;
    Ok(SpacesResult {
        spaces: state.spaces,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use xgen_common::state::ClientState;

    fn write_state(dir: &Path, state: &ClientState) {
        let path = dir.join("xgen-client_state.json");
        fs::write(path, serde_json::to_string_pretty(state).unwrap()).unwrap();
    }

    #[test]
    fn whoami_projects_state_subset() {
        let dir = tempdir().unwrap();
        let state = ClientState {
            identity_id: "xgen://pubkey/ed25519:abc".into(),
            display_name: "alice".into(),
            version: "0.10.3".into(),
            build: "deadbeef".into(),
            home_node: "ws://127.0.0.1:8080/xgen".into(),
            updated_at: "2026-05-17T00:00:00.000Z".into(),
            spaces: vec![],
        };
        write_state(dir.path(), &state);

        let mut session = SessionState::new(String::new(), dir.path().to_path_buf());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: dir.path(),
            node_override: None,
        };
        let r = whoami(&mut ctx).unwrap();
        assert_eq!(r.identity_id, "xgen://pubkey/ed25519:abc");
        assert_eq!(r.display_name, "alice");
        assert_eq!(r.home_node, "ws://127.0.0.1:8080/xgen");
        assert_eq!(r.spaces_joined, 0);
    }

    #[test]
    fn whoami_missing_state_file_errors() {
        let dir = tempdir().unwrap();
        let mut session = SessionState::new(String::new(), dir.path().to_path_buf());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: dir.path(),
            node_override: None,
        };
        let err = whoami(&mut ctx).unwrap_err();
        assert!(err.to_string().contains("state file not found"));
    }

    #[test]
    fn spaces_returns_known_spaces() {
        use xgen_common::state::{KnownRoom, KnownSpace};
        let dir = tempdir().unwrap();
        let state = ClientState {
            identity_id: "xgen://pubkey/ed25519:x".into(),
            display_name: "carol".into(),
            version: "0.10.3".into(),
            build: "feed".into(),
            home_node: "ws://127.0.0.1:8082/xgen".into(),
            updated_at: "2026-05-17T00:00:00.000Z".into(),
            spaces: vec![KnownSpace {
                space_id: "xgen://hash/sha256:abc".into(),
                name: "Test Space".into(),
                node_endpoint: "ws://127.0.0.1:8082/xgen".into(),
                role: "owner".into(),
                rooms: vec![KnownRoom {
                    room_id: "xgen://hash/sha256:def".into(),
                    name: "general".into(),
                    joined: true,
                }],
            }],
        };
        write_state(dir.path(), &state);

        let mut session = SessionState::new(String::new(), dir.path().to_path_buf());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: dir.path(),
            node_override: None,
        };
        let r = spaces(&mut ctx).unwrap();
        assert_eq!(r.spaces.len(), 1);
        assert_eq!(r.spaces[0].name, "Test Space");
        assert_eq!(r.spaces[0].rooms.len(), 1);
        assert_eq!(r.spaces[0].rooms[0].name, "general");
    }

    #[test]
    fn status_projects_state_with_age() {
        let dir = tempdir().unwrap();
        let state = ClientState {
            identity_id: "xgen://pubkey/ed25519:def".into(),
            display_name: "bob".into(),
            version: "0.10.3".into(),
            build: "cafe".into(),
            home_node: "ws://127.0.0.1:8081/xgen".into(),
            // Far-past timestamp so age is comfortably > 30s and stable.
            updated_at: "2020-01-01T00:00:00.000Z".into(),
            spaces: vec![],
        };
        write_state(dir.path(), &state);

        let mut session = SessionState::new(String::new(), dir.path().to_path_buf());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: dir.path(),
            node_override: None,
        };
        let r = status(&mut ctx).unwrap();
        assert_eq!(r.identity_id, "xgen://pubkey/ed25519:def");
        assert_eq!(r.display_name, "bob");
        assert_eq!(r.version, "0.10.3");
        assert_eq!(r.home_node, "ws://127.0.0.1:8081/xgen");
        assert_eq!(r.spaces_joined, 0);
        assert!(r.state_file_age_seconds > 30);
    }
}
