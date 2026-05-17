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
}
