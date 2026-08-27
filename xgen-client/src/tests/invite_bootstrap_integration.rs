// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M8.5-B (INV-D1/INV-D2/INV-D3) — client-seam e2e for the scoped
//! invite-bootstrap fetch (`crate::batch::get_invite_bootstrap`).
//!
//! Approach mirrors `events_pipe_integration.rs`: a **real** ephemeral WS stub
//! server (built from `xgen-core::transport` primitives) plays the home Node,
//! while a real client `Connection` drives the production wire helper over the
//! concrete `connect_url` path. The node-side bootstrap (`collect_invite_bootstrap`
//! plus the join-acceptance dissolution of M85-A3) is proven node-side in the
//! `xgen-node::fanout` tests; this closes the client half by checking that the
//! client sends `transport.invite_bootstrap_request`, drains the structural
//! batch, and extracts the `event_id` of the invite naming itself (so
//! `ops::join` can chain `prev_events=[invite_id]`).

use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpListener;

use xgen_core::crypto::encoding;
use xgen_core::identity::keypair;
use xgen_core::space::state::{
    build_membership_event, build_room_create_event, build_space_create_event, sign_event,
};
use xgen_core::transport::client::connect_url;
use xgen_core::transport::connection::{Connection, Inbound};
use xgen_core::resolution::{state_key_for_event, StateKey};
use xgen_core::wire::types::{Event, EventType, TransportMessage};

use crate::batch::get_invite_bootstrap;

/// M-SPACE-ADMISSION Leg G-4 — the state key of the join the caller is about to
/// sign, built the way `ops::join` builds it (Space join ⇒ empty `room_id`).
/// Production ALWAYS passes `Some(..)` here, so these tests do too: passing
/// `None` would exercise a configuration the product never reaches.
fn space_join_key(who: &ed25519_dalek::SigningKey, space_id: &str) -> StateKey {
    let probe = build_membership_event(who, space_id, "", EventType::MembershipJoin, serde_json::json!({}));
    state_key_for_event(&probe).expect("a membership.join always yields a state key")
}

const T: Duration = Duration::from_secs(5);
const NODE: &str = "xgen://pubkey/ed25519:NODE";

fn id_uri(key: &ed25519_dalek::SigningKey) -> String {
    format!("xgen://pubkey/ed25519:{}", encoding::encode(key.verifying_key().as_bytes()))
}

fn event_id(ev: &Event) -> String {
    ev.event_id.as_ref().unwrap().as_str().to_string()
}

/// A minimal real WS server playing the home Node for one bootstrap request:
/// accept, auth, read the `InviteBootstrapRequest`, then either serve `events`
/// (HistoryBatch shape: each event then `SyncComplete`) or refuse with wire
/// `1011`. Mirrors `events_pipe_integration::spawn_event_server`.
async fn spawn_bootstrap_server(
    events: Vec<Event>,
    refuse: bool,
) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let (tx, rx) = tokio::sync::oneshot::channel::<SocketAddr>();
    let handle = tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _ = tx.send(addr);
        let (stream, _) = listener.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut conn = Connection::new(ws);
        if conn.server_authenticate("xgen://pubkey/ed25519:TESTNODE").await.is_err() {
            return;
        }
        // The client's first post-auth message is the bootstrap request.
        match conn.recv().await {
            Ok(Inbound::Transport(TransportMessage::InviteBootstrapRequest { .. })) => {}
            _ => return,
        }
        if refuse {
            let _ = conn
                .send_transport(&TransportMessage::Error {
                    protocol_version: "0.1".into(),
                    error_code: 1011,
                    error_string: "invite_bootstrap_refused".into(),
                    timestamp: "2026-06-06T00:00:00.000Z".into(),
                    event_id: None,
                })
                .await;
        } else {
            for ev in &events {
                if conn.send_event(ev).await.is_err() {
                    return;
                }
            }
            let _ = conn
                .send_transport(&TransportMessage::SyncComplete {
                    protocol_version: "0.1".into(),
                    since: String::new(),
                    new_tip: String::new(),
                    continue_from: None,
                })
                .await;
        }
        // Keep open until the client closes.
        loop {
            if conn.recv().await.is_err() {
                break;
            }
        }
    });
    let addr = rx.await.expect("server reported its address");
    (addr, handle)
}

/// Build the structural bootstrap set (Space create, Room create, invite naming
/// `invitee_uri` with a future `valid_until`) and return (events, space_id,
/// invite_id).
fn structural_set(
    inviter: &ed25519_dalek::SigningKey,
    invitee_uri: &str,
) -> (Vec<Event>, String, String) {
    use chrono::{SecondsFormat, Utc};
    use serde_json::json;
    let space_ev =
        sign_event(build_space_create_event(inviter, "Boot", None, 1, NODE, None, false), inviter);
    let space_id = event_id(&space_ev);
    let room_ev = sign_event(build_room_create_event(inviter, &space_id, "general", None), inviter);
    let room_id = event_id(&room_ev);
    let future = (Utc::now() + chrono::Duration::days(14)).to_rfc3339_opts(SecondsFormat::Millis, true);
    let mut invite = build_membership_event(
        inviter,
        &space_id,
        "",
        EventType::MembershipInvite,
        json!({ "target_identity": invitee_uri, "role": "member", "valid_until": future }),
    );
    invite.prev_events =
        vec![xgen_common::xgid::EventXgid::from_xgid(xgen_common::xgid::Xgid::new(room_id))];
    let invite = sign_event(invite, inviter);
    let invite_id = event_id(&invite);
    (vec![space_ev, room_ev, invite], space_id, invite_id)
}

#[tokio::test]
async fn get_invite_bootstrap_returns_invite_id_naming_self() {
    let alice = keypair::generate();
    let bob = keypair::generate();
    let bob_id = id_uri(&bob);
    let (events, space_id, invite_id) = structural_set(&alice, &bob_id);

    let (addr, _server) = spawn_bootstrap_server(events, false).await;
    let mut conn = connect_url(&format!("ws://{addr}/xgen")).await.expect("connect");
    conn.client_authenticate(&bob).await.expect("auth");

    // Leg G-4: the rejoin selector is ARMED (as in production) and the invite
    // still wins — V-1, an invitee is byte-identical to before this leg.
    let key = space_join_key(&bob, &space_id);
    let found = get_invite_bootstrap(&mut conn, &space_id, &bob_id, Some(&key), T)
        .await
        .expect("bootstrap fetch");
    assert_eq!(
        found,
        vec![invite_id.clone()],
        "client must discover the event_id of the invite naming it (to chain its join)"
    );
}

#[tokio::test]
async fn get_invite_bootstrap_refusal_yields_none_for_fallback() {
    let alice = keypair::generate();
    let bob = keypair::generate();
    let bob_id = id_uri(&bob);
    let (_events, space_id, _invite_id) = structural_set(&alice, &bob_id);

    // Node refuses with 1011 → the helper returns EMPTY so ops::join falls back
    // to get_dag_tips (a normal outcome, not an error). Leg G-4 did not change
    // this: a refused requester was served no events, so there is nothing to
    // select from either.
    let (addr, _server) = spawn_bootstrap_server(vec![], true).await;
    let mut conn = connect_url(&format!("ws://{addr}/xgen")).await.expect("connect");
    conn.client_authenticate(&bob).await.expect("auth");

    let key = space_join_key(&bob, &space_id);
    let found = get_invite_bootstrap(&mut conn, &space_id, &bob_id, Some(&key), T)
        .await
        .expect("bootstrap fetch must not error on a 1011 refusal");
    assert!(found.is_empty(), "a 1011 refusal must yield empty (fall back), not an error");
}
