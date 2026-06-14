// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M8.5-C C2 (S5-D1/D2) — client-seam e2e for the `--re-registration` flag.
//!
//! Mirrors `invite_bootstrap_integration.rs`: a **real** ephemeral WS stub
//! server (built from `xgen-core::transport` primitives) plays the home Node,
//! while the production `ops::register` drives the concrete
//! `ensure_connected` + `connect_url` path. The check is that the
//! `re_registration` flag set on `RegisterArgs` reaches the wire as
//! `identity.register { re_registration: true }`.
//!
//! The node-side re-home itself (Step-3 bypass + `upsert` + version bump) is
//! proven in the xgen-core / xgen-node C1 tests. The `identity.home_changed`
//! emit is deferred (CP-5: the client holds no Node pubkey ids — `new_home_node_id`
//! needs a RegisterOk-echo surface, a follow-on arc).

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::sync::oneshot;

use xgen_common::xgid::{IdentityXgid, Xgid};
use xgen_core::crypto::encoding;
use xgen_core::identity::keypair;
use xgen_core::transport::connection::{Connection, Inbound};
use xgen_core::wire::types::IdentityMessage;

use crate::app::RegisterArgs;
use crate::ops::{register, OpContext};
use crate::session::{ClientIdentity, SessionState};

fn id_uri(key: &ed25519_dalek::SigningKey) -> String {
    format!("xgen://pubkey/ed25519:{}", encoding::encode(key.verifying_key().as_bytes()))
}

/// Stub home Node for one registration: accept, auth, read the register, report
/// its `re_registration` flag on a channel, reply `RegisterOk`, keep open.
async fn spawn_register_capture_server() -> (SocketAddr, oneshot::Receiver<bool>) {
    let (addr_tx, addr_rx) = oneshot::channel::<SocketAddr>();
    let (flag_tx, flag_rx) = oneshot::channel::<bool>();
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _ = addr_tx.send(addr);
        let (stream, _) = listener.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut conn = Connection::new(ws);
        let auth_id = match conn.server_authenticate("xgen://pubkey/ed25519:TESTNODE").await {
            Ok(id) => id,
            Err(_) => return,
        };
        match conn.recv().await {
            Ok(Inbound::Identity(IdentityMessage::Register { re_registration, .. })) => {
                let _ = flag_tx.send(re_registration);
                let ok = IdentityMessage::RegisterOk {
                    protocol_version: "0.1".into(),
                    identity_id: auth_id,
                    registered_at: "2026-06-06T00:00:00.000Z".into(),
                };
                let _ = conn.send_identity(&ok).await;
            }
            _ => return,
        }
        // Keep open until the client closes (it sends a courtesy goodbye).
        loop {
            if conn.recv().await.is_err() {
                break;
            }
        }
    });
    let addr = addr_rx.await.expect("server reported its address");
    (addr, flag_rx)
}

/// Drive `ops::register` with the given flag against the stub Node and return
/// the `re_registration` value the Node observed on the wire.
async fn register_and_capture_flag(re_registration: bool) -> bool {
    let key = keypair::generate();
    let id = id_uri(&key);
    let (addr, flag_rx) = spawn_register_capture_server().await;
    let url = format!("ws://{addr}/xgen");

    let tmp = tempfile::tempdir().unwrap();
    let mut session = SessionState::new(url, tmp.path().to_path_buf());
    // Inject the loaded identity (the dispatcher does this via ensure_identity
    // in production); ensure_connected clones this key for the auth handshake.
    session.identity = Some(ClientIdentity {
        signing_key: key.clone(),
        identity_id: IdentityXgid::from_xgid(Xgid::new(id)),
    });
    let dd = tmp.path().to_path_buf();
    let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
    let args = RegisterArgs { name: "Alice".to_string(), re_registration };

    let res = register(&mut ctx, &args, None).await;
    assert!(res.is_ok(), "register should succeed against the stub Node: {res:?}");
    flag_rx.await.expect("server reported the observed flag")
}

#[tokio::test]
async fn re_registration_flag_reaches_wire_when_set() {
    assert!(
        register_and_capture_flag(true).await,
        "--re-registration must set re_registration:true on the identity.register wire message"
    );
}

#[tokio::test]
async fn re_registration_absent_sends_false() {
    assert!(
        !register_and_capture_flag(false).await,
        "without the flag the registration is normal (re_registration:false)"
    );
}
