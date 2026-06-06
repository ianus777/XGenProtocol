// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Shared test support for the F-1c reconnect scheduler tests (the M8.6 C4
//! scheduler-churn tests).
//!
//! The responsive-receiver + lost-peer-registry fixtures mirror those in
//! `reconnect_integration.rs` (the Phase-5 originals). **Honest residue
//! (D-065):** `run_mock_receiver` + `blank_runtime` + `registry_with_lost_peer`
//! are duplicated here rather than extracted-and-rewired, to avoid disturbing
//! the passing Phase-5 integration test mid-milestone; consolidating the two
//! into this one module is a future test-only DRY cleanup. M8.6 adds two
//! helpers the Phase-5 tests don't have: a silent TCP black-hole (for the
//! connect-timeout / attempt-task-gauge test) and `advance_all` (the
//! MockClock + tokio-clock lockstep step, design §3.2).

#![cfg(test)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use tokio::sync::Mutex;

use xgen_common::clock::MockClock;
use xgen_common::xgid::{NodeXgid, SpaceXgid, Xgid};

use crate::{
    crypto::encoding,
    federation::{
        handshake::{negotiate_serialisation, negotiate_version, sign_msg, verify_msg},
        registry::{FederationRegistry, FederationRelationship, FederationState},
    },
    federation_session::stream_federation_delta,
    identity::keypair,
    node::runtime::NodeRuntime,
    transport::{connection::Inbound, server::Server},
    wire::types::{
        FederationCapabilities, FederationMessage, NegotiatedCapabilities, TransportMessage,
    },
};

pub(crate) fn ndx(s: &str) -> NodeXgid {
    NodeXgid::from_xgid(Xgid::new(s.to_string()))
}
pub(crate) fn sdx(s: &str) -> SpaceXgid {
    SpaceXgid::from_xgid(Xgid::new(s.to_string()))
}

pub(crate) fn pubkey_uri(key: &ed25519_dalek::SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(key.verifying_key().as_bytes())
    )
}

pub(crate) fn now_rfc() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Build a fresh NodeRuntime with no Spaces / identities. Sufficient for
/// handshake-only Phase 5 / M8.6 scheduler tests.
pub(crate) fn blank_runtime() -> (NodeRuntime, ed25519_dalek::SigningKey) {
    let k = keypair::generate();
    let rt = NodeRuntime::new(k.clone());
    (rt, k)
}

/// Pre-populate a registry with `peer` marked lost long enough ago that the
/// scheduler tick will find it due (next_reconnect_attempt ~5 min in the past).
pub(crate) fn registry_with_lost_peer(peer_node_id: &str, peer_url: &str) -> FederationRegistry {
    let mut reg = FederationRegistry::new();
    let peer_typed = ndx(peer_node_id);
    reg.upsert(FederationRelationship {
        peer_node_id: peer_typed.clone(),
        shared_spaces: vec![],
        negotiated_version: "0.1".to_string(),
        negotiated_serialisation: "json".to_string(),
        session_id: "xgen://hash/sha256:stale-session".to_string(),
        last_connected: now_rfc(),
        peer_url: Some(peer_url.to_string()),
        state: FederationState::Active,
    });
    // Mark lost 20 min ago → next_reconnect_attempt = 5 min ago → due.
    reg.mark_lost(&peer_typed, Utc::now() - chrono::Duration::minutes(20));
    reg
}

/// Run a mock receiver-side federation session against a single inbound TCP
/// connection. Inlines the Hello → Caps → Accept state machine (mirroring
/// `handle_federation_incoming`'s first 100 lines), then `stream_federation_delta`
/// and a drain loop. Returns when the connection closes or sends Goodbye; the
/// initiator (the Node under test) reaches handshake-ACTIVE against this peer.
pub(crate) async fn run_mock_receiver(
    mut server: Server,
    runtime: Arc<Mutex<NodeRuntime>>,
    node_key: ed25519_dalek::SigningKey,
    spaces_dir: PathBuf,
) {
    let mut conn = match server.accept().await {
        Ok(c) => c,
        Err(_) => return,
    };
    if conn.server_authenticate().await.is_err() {
        return;
    }
    let hello = match conn.recv().await {
        Ok(Inbound::Federation(fm)) if matches!(&fm, FederationMessage::Hello { .. }) => fm,
        _ => return,
    };
    if verify_msg(&hello).is_err() {
        return;
    }
    let (peer_node_id, peer_caps, peer_version, peer_shared_spaces, peer_tips) = match hello {
        FederationMessage::Hello {
            node_id,
            capabilities,
            protocol_version,
            shared_spaces,
            tips,
            ..
        } => (node_id, capabilities, protocol_version, shared_spaces, tips),
        _ => unreachable!(),
    };
    let our_caps = FederationCapabilities::default();
    let serial = negotiate_serialisation(&our_caps.serialisation, &peer_caps.serialisation)
        .unwrap_or_else(|| "json".to_string());
    let neg_version = negotiate_version("0.1", &peer_version).unwrap_or_else(|| "0.1".to_string());

    let peer_shared_spaces_typed: Vec<SpaceXgid> =
        peer_shared_spaces.iter().map(|s| sdx(s)).collect();
    let our_tips: BTreeMap<String, String> = {
        let rt = runtime.lock().await;
        peer_shared_spaces_typed
            .iter()
            .filter_map(|space_id| {
                rt.dag_tips(space_id)
                    .into_iter()
                    .min()
                    .map(|t| (space_id.as_str().to_string(), t))
            })
            .collect()
    };

    let caps_msg = sign_msg(
        FederationMessage::Capabilities {
            protocol_version: "0.1".to_string(),
            node_id: { runtime.lock().await.node_id.as_str().to_string() },
            capabilities: our_caps,
            negotiated: NegotiatedCapabilities {
                serialisation: serial.clone(),
                protocol_version: neg_version.clone(),
            },
            tips: our_tips,
            timestamp: now_rfc(),
            signature: None,
        },
        &node_key,
    );
    if conn.send_federation(&caps_msg).await.is_err() {
        return;
    }
    let accept_msg = match conn.recv().await {
        Ok(Inbound::Federation(fm @ FederationMessage::Accept { .. })) => fm,
        _ => return,
    };
    if verify_msg(&accept_msg).is_err() {
        return;
    }
    let session_id = match accept_msg {
        FederationMessage::Accept { session_id, .. } => session_id,
        _ => return,
    };

    let peer_node_id_typed = ndx(&peer_node_id);
    let _ = stream_federation_delta(
        &mut conn,
        &runtime,
        &peer_shared_spaces_typed,
        &peer_tips,
        &peer_node_id_typed,
        &session_id,
        &neg_version,
        &serial,
        &node_key,
        &spaces_dir,
    )
    .await;

    loop {
        match conn.recv().await {
            Ok(Inbound::Transport(TransportMessage::Goodbye { .. }))
            | Ok(Inbound::Closed)
            | Err(_) => break,
            _ => {}
        }
    }
}

/// M8.6 (C4) — a silent TCP black-hole. Binds `127.0.0.1:0`, accepts inbound
/// TCP connections, then holds each socket open forever WITHOUT ever sending
/// the WebSocket upgrade response. A client's `connect_async` completes its TCP
/// connect but then hangs awaiting the HTTP 101 — which is exactly the
/// non-responsive-peer condition the reconnect connect-timeout (CONNECT_TIMEOUT_SECS)
/// must bound. Returns the accept-loop task handle (drop/abort to stop) and the
/// `ws://…` URL to dial. Real I/O works under `tokio::time` pause, so only the
/// timeout (driven by `advance`) resolves a hung attempt.
pub(crate) async fn silent_blackhole_listener() -> (tokio::task::JoinHandle<()>, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{}/", addr);
    let handle = tokio::spawn(async move {
        while let Ok((stream, _peer)) = listener.accept().await {
            // Hold the accepted stream open in a never-resolving task so the TCP
            // connection stays ESTABLISHED (no RST) but silent — the client
            // hangs on the WS upgrade.
            tokio::spawn(async move {
                let _held = stream;
                std::future::pending::<()>().await;
            });
        }
    });
    (handle, url)
}

/// M8.6 (design §3.2) — advance the mock clock's single cursor AND the tokio
/// timer clock by the same delta, so the W / M / T domains move in lockstep
/// from one call. Lives in the xgen-node harness (not xgen-common) because it
/// touches `tokio::time`, which xgen-common does not depend on.
pub(crate) async fn advance_all(clock: &MockClock, d: Duration) {
    clock.advance(d);
    tokio::time::advance(d).await;
}
