// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M10.4 C2 (MP-F13, Shape B) — client-seam e2e for the home_node namespace fix.
//!
//! Mirrors `reregistration_integration.rs`: a **real** ephemeral WS stub server
//! (from `xgen-core::transport` primitives) plays the home Node while the
//! production `ops::create_space` drives the concrete `ensure_connected` +
//! `connect_url` path.
//!
//! Witness 1 — the client writes the Node's pubkey `node_id` (captured from the
//!   `AuthOk` echo), NOT the `ws://` transport URL, into the Space's signed
//!   `content["home_node"]`. This is the fast witness that fixes the
//!   in-process-test blind spot (the unit fixtures wrote a pubkey, hiding the
//!   real-binary URL bug). RED on revert of the ops.rs write.
//! Witness 4 — against a pre-echo Node (`AuthOk` with no `node_id`), the client
//!   REFUSES to create the Space (explicit error) rather than silently writing a
//!   transport URL. RED on revert of the absent-echo guard.

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::sync::oneshot;

use xgen_common::xgid::{IdentityXgid, Xgid};
use xgen_core::crypto::encoding;
use xgen_core::identity::keypair;
use xgen_core::transport::auth;
use xgen_core::transport::connection::{Connection, Inbound};
use xgen_core::wire::types::TransportMessage;

use crate::app::CreateSpaceArgs;
use crate::ops::{create_space, OpContext};
use crate::session::{ClientIdentity, SessionState};

/// The Node pubkey id the stub advertises — deliberately pubkey-shaped and
/// distinct from the `ws://` URL the client dials.
const NODE_ID: &str = "xgen://pubkey/ed25519:WITNESSNODE";

fn id_uri(key: &ed25519_dalek::SigningKey) -> String {
    format!("xgen://pubkey/ed25519:{}", encoding::encode(key.verifying_key().as_bytes()))
}

/// Stub home Node that echoes `NODE_ID` on `AuthOk`, captures the one submitted
/// `state.space_create` event, reports its `content["home_node"]`, and accepts it.
async fn spawn_home_node_echo_server() -> (SocketAddr, oneshot::Receiver<String>) {
    let (addr_tx, addr_rx) = oneshot::channel::<SocketAddr>();
    let (home_tx, home_rx) = oneshot::channel::<String>();
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _ = addr_tx.send(addr);
        let (stream, _) = listener.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut conn = Connection::new(ws);
        // The full server-side handshake — populates AuthOk.node_id with NODE_ID.
        if conn.server_authenticate(NODE_ID).await.is_err() {
            return;
        }
        if let Ok(Inbound::Event(ev)) = conn.recv().await {
            let home = ev
                .content
                .get("home_node")
                .and_then(|v| v.as_str())
                .unwrap_or("<absent>")
                .to_string();
            let _ = home_tx.send(home);
            // Confirm so create_space's send_event_confirmed returns Accepted.
            if let Some(eid) = ev.event_id.as_ref() {
                let ok = TransportMessage::EventAccepted {
                    protocol_version: "0.1".into(),
                    event_id: eid.as_str().to_string(),
                    accepted_at: "2026-06-14T00:00:00.000Z".into(),
                };
                let _ = conn.send_transport(&ok).await;
            }
        }
        // Keep open until the client closes (courtesy goodbye).
        loop {
            if conn.recv().await.is_err() {
                break;
            }
        }
    });
    let addr = addr_rx.await.expect("server reported its address");
    (addr, home_rx)
}

/// Stub PRE-ECHO Node: does the handshake by hand and sends `AuthOk` WITHOUT a
/// `node_id` (an older Node). Used to prove the client refuses rather than
/// writing a URL.
async fn spawn_pre_echo_node_server() -> SocketAddr {
    let (addr_tx, addr_rx) = oneshot::channel::<SocketAddr>();
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _ = addr_tx.send(addr);
        let (stream, _) = listener.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut conn = Connection::new(ws);

        let (issued, challenge) = auth::issue_challenge();
        if conn.send_transport(&challenge).await.is_err() {
            return;
        }
        let auth_msg = match conn.recv().await {
            Ok(Inbound::Transport(tm @ TransportMessage::Auth { .. })) => tm,
            _ => return,
        };
        let identity_id = match auth::verify_auth_response(&issued, &auth_msg) {
            Ok(id) => id,
            Err(_) => return,
        };
        // Legacy-shaped AuthOk: node_id omitted (None).
        let ok = TransportMessage::AuthOk {
            protocol_version: "0.1".into(),
            identity_id,
            timestamp: "2026-06-14T00:00:00.000Z".into(),
            node_id: None,
        };
        let _ = conn.send_transport(&ok).await;
        // The client should error out before sending any Event; drain to EOF.
        loop {
            if conn.recv().await.is_err() {
                break;
            }
        }
    });
    addr_rx.await.expect("server reported its address")
}

fn session_for(url: String, key: &ed25519_dalek::SigningKey) -> SessionState {
    let tmp = tempfile::tempdir().unwrap();
    let mut session = SessionState::new(url, tmp.path().to_path_buf());
    // Leak the tempdir guard so data_dir stays valid for the test body.
    std::mem::forget(tmp);
    session.identity = Some(ClientIdentity {
        signing_key: key.clone(),
        identity_id: IdentityXgid::from_xgid(Xgid::new(id_uri(key))),
    });
    session
}

#[tokio::test]
async fn create_space_writes_node_id_not_url_into_home_node() {
    let key = keypair::generate();
    let (addr, home_rx) = spawn_home_node_echo_server().await;
    let url = format!("ws://{addr}/xgen");
    let mut session = session_for(url.clone(), &key);
    let dd = session.data_dir.clone();
    let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
    let args = CreateSpaceArgs { name: "M10.4-witness".into(), auth_tier: 1 };

    let res = create_space(&mut ctx, &args).await;
    assert!(res.is_ok(), "create_space should succeed against the echo Node: {res:?}");

    let observed = home_rx.await.expect("server reported the Space's home_node");
    assert_eq!(
        observed, NODE_ID,
        "Space content[home_node] must carry the Node's pubkey node_id, not the URL"
    );
    assert_ne!(observed, url, "home_node must NOT be the ws:// transport URL");
}

#[tokio::test]
async fn create_space_refuses_when_node_id_absent() {
    let key = keypair::generate();
    let addr = spawn_pre_echo_node_server().await;
    let url = format!("ws://{addr}/xgen");
    let mut session = session_for(url, &key);
    let dd = session.data_dir.clone();
    let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
    let args = CreateSpaceArgs { name: "M10.4-fallback".into(), auth_tier: 1 };

    let res = create_space(&mut ctx, &args).await;
    let err = res.expect_err("create_space must refuse when the Node advertises no node_id");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("did not advertise its node_id"),
        "expected the explicit absent-echo refusal, got: {msg}"
    );
}
