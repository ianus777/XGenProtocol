// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-F1a C2 — deterministic stub-WS policy tests for the send-confirm retrofit.
//!
//! Mirrors `reregistration_integration.rs`: a **real** ephemeral WS stub server
//! (built from `xgen-core::transport` primitives) plays the home Node and, per
//! test, acks / rejects / stays silent for the events the production `ops::*`
//! verbs submit through `Connection::send_event_confirmed`. These prove the
//! F1A-D3/D4 policy deterministically and offline (the end-to-end facet-2
//! witness is the single-node-DM mptest scenario `MP-C-07-LOCAL`).
//!
//! `create_*` are used because they build their events locally and issue no
//! `sync_request` — so the stub only has to answer the submitted Events, not a
//! tip-discovery round trip.

use std::net::SocketAddr;
use std::path::Path;

use tokio::net::TcpListener;
use tokio::sync::oneshot;

use xgen_common::xgid::{IdentityXgid, Xgid};
use xgen_core::crypto::encoding;
use xgen_core::identity::keypair;
use xgen_core::transport::connection::{Connection, Inbound};
use xgen_core::wire::types::TransportMessage;

use crate::app::{CreateDmSpaceArgs, CreateSpaceArgs};
use crate::ops::{create_dm_space, create_space, OpContext};
use crate::session::{ClientIdentity, SessionState};

/// Per-event stub action, applied in event arrival order.
#[derive(Clone)]
enum Act {
    /// Reply `EventAccepted` for the event's id.
    Ack,
    /// Reply `Error` for the event's id.
    Reject(u32, String),
    /// Send nothing — forces the client's confirm to time out.
    Silent,
}

fn id_uri(key: &ed25519_dalek::SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(key.verifying_key().as_bytes())
    )
}

/// A stub home Node: authenticate, then apply `actions[i]` to the i-th inbound
/// Event (Silent beyond the list). Stays open until the client closes.
async fn spawn_confirm_stub(actions: Vec<Act>) -> SocketAddr {
    let (addr_tx, addr_rx) = oneshot::channel::<SocketAddr>();
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _ = addr_tx.send(addr);
        let (stream, _) = listener.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut conn = Connection::new(ws);
        if conn.server_authenticate("xgen://pubkey/ed25519:TESTNODE").await.is_err() {
            return;
        }
        let mut i = 0usize;
        loop {
            match conn.recv().await {
                Ok(Inbound::Event(ev)) => {
                    let id = ev
                        .event_id
                        .as_ref()
                        .map(|e| e.as_str().to_string())
                        .unwrap_or_default();
                    let act = actions.get(i).cloned().unwrap_or(Act::Silent);
                    i += 1;
                    match act {
                        Act::Ack => {
                            let ok = TransportMessage::EventAccepted {
                                protocol_version: "0.1".into(),
                                event_id: id,
                                accepted_at: "2026-06-08T00:00:00.000Z".into(),
                            };
                            let _ = conn.send_transport(&ok).await;
                        }
                        Act::Reject(code, reason) => {
                            let err = TransportMessage::Error {
                                protocol_version: "0.1".into(),
                                error_code: code,
                                error_string: reason,
                                timestamp: "2026-06-08T00:00:00.000Z".into(),
                                event_id: Some(id),
                            };
                            let _ = conn.send_transport(&err).await;
                        }
                        Act::Silent => {}
                    }
                }
                Ok(Inbound::Transport(TransportMessage::Goodbye { .. })) | Ok(Inbound::Closed) => {
                    break
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });
    addr_rx.await.expect("stub reported its address")
}

/// A minimal client config with a 1s sync-completion timeout so the TimedOut
/// path tests are fast (the default is 5s). `ClientConfig` requires the
/// `client`/`paths`/`logging` sections, so the doc is full, not `[sync]`-only.
fn write_fast_config(dir: &Path) {
    let toml = "[client]\n\
                node = \"ws://127.0.0.1:0/xgen\"\n\
                \n\
                [paths]\n\
                keypair_path = \"xgen-client_keypair.enc\"\n\
                \n\
                [logging]\n\
                level = \"info\"\n\
                \n\
                [sync]\n\
                completion_timeout_seconds = 1\n";
    std::fs::write(dir.join("xgen-client_config.toml"), toml).unwrap();
}

fn state_path(dir: &Path) -> std::path::PathBuf {
    dir.join("xgen-client_state.json")
}

/// Build a session with an injected identity (the dispatcher does this via
/// `ensure_identity` in production), targeting `url`.
fn session_for(url: String, key: &ed25519_dalek::SigningKey, dir: &Path) -> SessionState {
    let mut session = SessionState::new(url, dir.to_path_buf());
    session.identity = Some(ClientIdentity {
        signing_key: key.clone(),
        identity_id: IdentityXgid::from_xgid(Xgid::new(id_uri(key))),
    });
    session
}

// create_dm_space: the stub acks the root then goes silent on the auto-room →
// the chain aborts (a TimedOut chain event is genuinely-lost, F-5) AND the verb
// writes NO client-state record (the record is written only after the full
// confirmed chain).
#[tokio::test]
async fn create_dm_space_aborts_and_writes_no_record_when_chain_times_out() {
    let key = keypair::generate();
    let tmp = tempfile::tempdir().unwrap();
    write_fast_config(tmp.path());
    let addr = spawn_confirm_stub(vec![Act::Ack]).await; // ack #1, silent thereafter
    let url = format!("ws://{addr}/xgen");
    let mut session = session_for(url, &key, tmp.path());
    let dd = tmp.path().to_path_buf();
    let mut ctx = OpContext {
        session: &mut session,
        data_dir: &dd,
        node_override: None,
    };
    let args = CreateDmSpaceArgs {
        invitee: "xgen://pubkey/ed25519:BOB".into(),
    };

    let res = create_dm_space(&mut ctx, &args).await;
    assert!(res.is_err(), "a timed-out chain must fail the verb: {res:?}");
    assert!(
        !state_path(tmp.path()).exists(),
        "a failed create-dm-space must write no client-state record"
    );
}

// create_dm_space: the stub acks all three events → the verb succeeds and the
// DM Space lands in client state.
#[tokio::test]
async fn create_dm_space_succeeds_and_writes_record_when_all_acked() {
    let key = keypair::generate();
    let tmp = tempfile::tempdir().unwrap();
    write_fast_config(tmp.path());
    let addr = spawn_confirm_stub(vec![Act::Ack, Act::Ack, Act::Ack]).await;
    let url = format!("ws://{addr}/xgen");
    let mut session = session_for(url, &key, tmp.path());
    let dd = tmp.path().to_path_buf();
    let mut ctx = OpContext {
        session: &mut session,
        data_dir: &dd,
        node_override: None,
    };
    let args = CreateDmSpaceArgs {
        invitee: "xgen://pubkey/ed25519:BOB".into(),
    };

    let res = create_dm_space(&mut ctx, &args)
        .await
        .expect("all-acked chain should succeed");
    let raw = std::fs::read_to_string(state_path(tmp.path())).expect("state file written");
    let st: xgen_common::state::ClientState = serde_json::from_str(&raw).unwrap();
    assert_eq!(st.spaces.len(), 1, "exactly one DM Space recorded");
    assert_eq!(st.spaces[0].space_id, res.space_id.as_str());
}

// Single-event op against a silent stub → proceed (Ok) with a warning: a lone
// event's lost-vs-HeldPending is irreducibly ambiguous without a held-signal
// (F1A-D5), so the op returns ok-unconfirmed rather than failing.
#[tokio::test]
async fn single_event_proceeds_on_timeout() {
    let key = keypair::generate();
    let tmp = tempfile::tempdir().unwrap();
    write_fast_config(tmp.path());
    let addr = spawn_confirm_stub(vec![Act::Silent]).await;
    let url = format!("ws://{addr}/xgen");
    let mut session = session_for(url, &key, tmp.path());
    let dd = tmp.path().to_path_buf();
    let mut ctx = OpContext {
        session: &mut session,
        data_dir: &dd,
        node_override: None,
    };
    let args = CreateSpaceArgs { name: "S".into(), auth_tier: 1 };

    let res = create_space(&mut ctx, &args).await;
    assert!(
        res.is_ok(),
        "a single-event timeout must proceed (warn), not fail: {res:?}"
    );
}

// Single-event op whose event the node rejects → Err (CP-1: node rejections
// finally surface to the client).
#[tokio::test]
async fn single_event_errors_on_reject() {
    let key = keypair::generate();
    let tmp = tempfile::tempdir().unwrap();
    write_fast_config(tmp.path());
    let addr = spawn_confirm_stub(vec![Act::Reject(3045, "invite_validity_exceeds_max".into())]).await;
    let url = format!("ws://{addr}/xgen");
    let mut session = session_for(url, &key, tmp.path());
    let dd = tmp.path().to_path_buf();
    let mut ctx = OpContext {
        session: &mut session,
        data_dir: &dd,
        node_override: None,
    };
    let args = CreateSpaceArgs { name: "S".into(), auth_tier: 1 };

    match create_space(&mut ctx, &args).await {
        Ok(_) => panic!("a node reject must surface as Err (CP-1)"),
        Err(e) => assert!(
            e.to_string().contains("rejected by node"),
            "reject should surface in the error message; got: {e}"
        ),
    }
}
