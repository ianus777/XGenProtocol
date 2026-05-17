// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Per-invocation session state threaded through every `ops::*` call via
//! `OpContext` (M5, D-067).
//!
//! M5 sessions are one-shot: each dispatcher invocation (CLI arm in
//! `main.rs`, pipe arm in `batch::dispatch_line`) builds a fresh
//! `SessionState`, calls one `ops::*` function, and drops it. The `conn`
//! field opens lazily inside `ops::*` on first network operation; offline
//! commands (whoami / status / spaces) never touch it. The `bindings` and
//! `spaces` maps are M7 extension points — present so the type signature
//! is M7-stable (the future `--aicontrol` persistent-session surface
//! populates them across commands), empty in M5.

use std::collections::HashMap;
use std::path::PathBuf;

/// Concrete WebSocket connection alias used throughout the Client. Matches
/// the return type of `xgen_core::transport::client::connect_url`.
pub type ClientConnection = xgen_core::transport::connection::Connection<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

/// Per-Space cache. Empty unit struct in M5; M7 populates with
/// last-event-observed tracking that eliminates the per-command
/// `get_dag_tips` sync_request round trip.
#[derive(Debug, Default)]
pub struct SpaceCache;

pub struct SessionState {
    pub conn: Option<ClientConnection>,
    pub home_node: String,
    pub data_dir: PathBuf,

    pub bindings: HashMap<String, String>,
    pub spaces: HashMap<String, SpaceCache>,
}

impl SessionState {
    /// Construct a fresh one-shot session.
    pub fn new(home_node: String, data_dir: PathBuf) -> Self {
        Self {
            conn: None,
            home_node,
            data_dir,
            bindings: HashMap::new(),
            spaces: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_fields_default_empty() {
        let s = SessionState::new("ws://127.0.0.1:8080/xgen".into(), PathBuf::from("."));
        assert!(s.bindings.is_empty());
        assert!(s.spaces.is_empty());
        assert!(s.conn.is_none());
    }
}
