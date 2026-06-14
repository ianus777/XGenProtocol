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
//! `SessionState`, calls `ensure_identity` eagerly so I/O failures surface
//! at the dispatcher boundary, then hands `&mut self` into `ops::*` for
//! the protocol work. The `conn` field opens lazily inside `ops::*` on
//! first network operation via `ensure_connected`; offline commands
//! (whoami / status / spaces) never touch it. The `bindings` and `spaces`
//! maps are M7 extension points — present so the type signature is
//! M7-stable (the future `--aicontrol` persistent-session surface
//! populates them across commands), empty in M5.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use ed25519_dalek::SigningKey;

use xgen_common::xgid::{IdentityXgid, Xgid};
use xgen_core::{
    identity::registration::identity_id_from_key,
    transport::client::connect_url,
};

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

/// Loaded keypair + cached `identity_id`. Constructed by
/// `SessionState::ensure_identity` from the on-disk encrypted keypair file.
/// Caching `identity_id` here avoids re-hashing the public key on every
/// access; M7 persistent sessions get O(1) identity lookup for free.
#[derive(Clone)]
pub struct ClientIdentity {
    pub signing_key: SigningKey,
    pub identity_id: IdentityXgid,
}

impl ClientIdentity {
    /// Load + decrypt the keypair file and derive `identity_id` in one call.
    /// File-not-found / decrypt failures surface here as `Err` so dispatchers
    /// can fail at the I/O boundary rather than inside `ops::*`.
    pub fn load(keypair_path: &Path) -> Result<Self> {
        let signing_key = crate::app::load_keypair(keypair_path)?;
        let identity_id = IdentityXgid::from_xgid(Xgid::new(identity_id_from_key(&signing_key)));
        Ok(Self {
            signing_key,
            identity_id,
        })
    }
}

pub struct SessionState {
    pub conn: Option<ClientConnection>,
    pub identity: Option<ClientIdentity>,
    pub home_node: String,
    /// The connected Node's pubkey `node_id` (`xgen://pubkey/ed25519:…`),
    /// captured from the `AuthOk` echo during `ensure_connected` (M10.4 /
    /// MP-F13, Shape B). Distinct from `home_node` (a `ws://` transport URL):
    /// this is what `create_space` / `create_dm_space` write into a Space's
    /// signed `content["home_node"]`. `None` until connected, or against a
    /// pre-echo Node.
    pub node_id: Option<String>,
    pub data_dir: PathBuf,

    pub bindings: HashMap<String, String>,
    pub spaces: HashMap<String, SpaceCache>,
}

impl SessionState {
    /// Construct a fresh one-shot session.
    pub fn new(home_node: String, data_dir: PathBuf) -> Self {
        Self {
            conn: None,
            identity: None,
            home_node,
            node_id: None,
            data_dir,
            bindings: HashMap::new(),
            spaces: HashMap::new(),
        }
    }

    /// Idempotent identity load. First call decrypts the keypair file and
    /// derives `identity_id`; subsequent calls return the cached reference.
    /// M5 dispatchers call this eagerly *before* building the `OpContext`
    /// so file-not-found / decrypt failures surface at the dispatcher
    /// boundary (Q-D2 confirmation, 2026-05-17).
    pub fn ensure_identity(&mut self, keypair_path: &Path) -> Result<&ClientIdentity> {
        if self.identity.is_none() {
            self.identity = Some(ClientIdentity::load(keypair_path)?);
        }
        Ok(self.identity.as_ref().unwrap())
    }

    /// Idempotent WebSocket connect + authenticate. First call opens the
    /// socket and runs the client-side auth handshake; subsequent calls
    /// return the cached connection. `node_override` takes precedence over
    /// `self.home_node` so CLI/pipe dispatchers can route per-call without
    /// rebuilding the session.
    ///
    /// Precondition: `ensure_identity` must have been called first
    /// (the dispatcher always does so in M5). Calling without identity
    /// loaded returns an `Err` rather than panicking.
    pub async fn ensure_connected(
        &mut self,
        node_override: Option<&str>,
    ) -> Result<&mut ClientConnection> {
        if self.conn.is_none() {
            let signing_key = self
                .identity
                .as_ref()
                .ok_or_else(|| anyhow!("identity not loaded; call ensure_identity first"))?
                .signing_key
                .clone();
            let node = node_override.unwrap_or(self.home_node.as_str());
            if node.is_empty() {
                bail!("no node endpoint resolved (home_node empty and no override given)");
            }
            tracing::info!(node_url = %node, "Connecting to Node");
            let mut conn = connect_url(node)
                .await
                .context("failed to connect to Node")?;
            let auth = conn
                .client_authenticate(&signing_key)
                .await
                .context("authentication failed")?;
            let auth_id = auth.identity_id;
            // M10.4 (MP-F13, Shape B): capture the Node's pubkey node_id from
            // the AuthOk echo so create_space/create_dm_space can write it as a
            // Space's home_node. Tied to the connection actually used, so a
            // --node-override is honoured automatically.
            self.node_id = auth.node_id;
            // Greppable diagnostic lines preserved verbatim from the
            // pre-M5 cmd_* implementations so log-parsing tooling
            // (smoke/stress tests, multiparty findings parsers) keeps
            // working without churn.
            tracing::info!("identity_id={}", auth_id);
            tracing::info!("connected_node={}", node);
            tracing::info!(identity_id = %auth_id, "Authenticated");
            self.conn = Some(conn);
        }
        Ok(self.conn.as_mut().unwrap())
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
        assert!(s.identity.is_none());
    }

    #[tokio::test]
    async fn ensure_connected_without_identity_errors() {
        let mut s = SessionState::new("ws://127.0.0.1:8080/xgen".into(), PathBuf::from("."));
        // ClientConnection doesn't impl Debug, so `unwrap_err()` won't compile —
        // pattern-match the Result explicitly.
        match s.ensure_connected(None).await {
            Ok(_) => panic!("expected error when identity is not loaded"),
            Err(e) => assert!(e.to_string().contains("identity not loaded")),
        }
    }

    /// T10 — `ClientIdentity.identity_id` is `IdentityXgid`; `SessionState.home_node`
    /// stays `String` (a `ws://` transport URL, not a Node XGID) per design
    /// doc §4.6.b.
    #[test]
    fn client_identity_identity_id_typed_home_node_stays_string() {
        let id = ClientIdentity {
            signing_key: SigningKey::from_bytes(&[7u8; 32]),
            identity_id: IdentityXgid::from_xgid(Xgid::new(
                "xgen://pubkey/ed25519:X".to_string(),
            )),
        };
        assert_eq!(id.identity_id.as_str(), "xgen://pubkey/ed25519:X");

        let s = SessionState::new("ws://127.0.0.1:8080/xgen".to_string(), PathBuf::from("."));
        let _home_node: &String = &s.home_node; // stays String — transport URL
        assert_eq!(s.home_node, "ws://127.0.0.1:8080/xgen");
    }

    /// T11 — the on-disk `xgen-client_state.json` format (`ClientState`)
    /// round-trips with String identifier slots at the serde boundary per
    /// design doc §4.6.b (Q5.4): the persisted shape is unchanged by the
    /// in-memory `ClientIdentity` retype.
    #[test]
    fn session_state_on_disk_persistence_format_round_trip_string_at_boundary() {
        use xgen_common::state::ClientState;
        let json = r#"{"identity_id":"xgen://pubkey/ed25519:Z","display_name":"z","version":"0.1","build":"b","home_node":"ws://127.0.0.1:8080/xgen","updated_at":"t","spaces":[]}"#;
        let st: ClientState = serde_json::from_str(json).unwrap();
        assert_eq!(st.identity_id, "xgen://pubkey/ed25519:Z"); // String at boundary
        assert_eq!(st.home_node, "ws://127.0.0.1:8080/xgen");
        let back = serde_json::to_string(&st).unwrap();
        assert!(back.contains(r#""identity_id":"xgen://pubkey/ed25519:Z""#));
    }
}
