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

// ── M-SPACE-ADMISSION Leg G-4 v1.2 — the resolution order ─────────────────────

fn xgid_of(id: &str) -> xgen_common::xgid::EventXgid {
    xgen_common::xgid::EventXgid::from_xgid(xgen_common::xgid::Xgid::new(id.to_string()))
}

/// The batch Leg G-3 route 2 serves a **retained former member**: the creates,
/// plus only the membership events naming her — and that includes the invite
/// she already used.
///
/// 🔑 **Two objects share one word, and this fixture is where they separate.**
/// The *entitlement* (`PendingInvite` in `space.pending_invites`) was CONSUMED
/// by `apply_join` at her first join (`xgen-core/src/space/state.rs:1251` at
/// `951b758`). The *record* (the `membership.invite` EVENT) is PERMANENT,
/// because her own first join names it by hash and `D-154`② rules that
/// membership history is remembered, never erased. A client scanning the served
/// batch for an invite naming her therefore gets `yes, forever` — it is asking a
/// HISTORY question and cannot answer the STATE question it means to ask.
///
/// Returns `(events, space_id, room_id, spent_invite_id, join_id, leave_id)`.
fn departed_set(
    inviter: &ed25519_dalek::SigningKey,
    leaver: &ed25519_dalek::SigningKey,
) -> (Vec<Event>, String, String, String, String, String) {
    let leaver_uri = id_uri(leaver);
    let (mut events, space_id, spent_invite_id) = structural_set(inviter, &leaver_uri);
    let room_id = event_id(&events[1]);

    // Her first join, chained on the invite it consumed (INV-D3).
    let mut join = build_membership_event(
        leaver,
        &space_id,
        "",
        EventType::MembershipJoin,
        serde_json::json!({}),
    );
    join.prev_events = vec![xgid_of(&spent_invite_id)];
    let join = sign_event(join, leaver);
    let join_id = event_id(&join);

    // Her departure, chained on that join.
    let mut leave = build_membership_event(
        leaver,
        &space_id,
        "",
        EventType::MembershipLeave,
        serde_json::json!({}),
    );
    leave.prev_events = vec![xgid_of(&join_id)];
    let leave = sign_event(leave, leaver);
    let leave_id = event_id(&leave);

    events.push(join);
    events.push(leave);
    (events, space_id, room_id, spent_invite_id, join_id, leave_id)
}

/// **V-2b (v1.2) — THE CASE THAT WAS RED ON A LIVE WIRE.**
///
/// A departed member whose **spent** invite is still in the served batch. The
/// spent invite must be SUBTRACTED at §3 step 3 — her own first join references
/// it — and the anchor must be her DEPARTURE.
///
/// 🛑 **RED against v1.1's resolution order, GREEN against v1.2's.** Under v1.1
/// the invite scan won unconditionally, so this returned `vec![spent_invite_id]`
/// and the rejoin was left concurrent with her own leave — the Node answered
/// `3048 rejoin_not_anchored`. This is the unit-level twin of that live failure.
#[tokio::test]
async fn v2b_a_spent_invite_is_subtracted_and_the_anchor_is_her_departure() {
    let alice = keypair::generate();
    let bob = keypair::generate();
    let bob_id = id_uri(&bob);
    let (events, space_id, _room_id, spent_invite_id, join_id, leave_id) =
        departed_set(&alice, &bob);

    let (addr, _server) = spawn_bootstrap_server(events, false).await;
    let mut conn = connect_url(&format!("ws://{addr}/xgen")).await.expect("connect");
    conn.client_authenticate(&bob).await.expect("auth");

    let key = space_join_key(&bob, &space_id);
    let found = get_invite_bootstrap(&mut conn, &space_id, &bob_id, Some(&key), T)
        .await
        .expect("bootstrap fetch");

    assert_eq!(
        found,
        vec![leave_id.clone()],
        "a rejoin must chain on her own DEPARTURE — the leaf of her membership record — \
         not on the invite she spent to get in the first time"
    );
    assert!(
        !found.contains(&spent_invite_id),
        "the spent invite is referenced by her own join, so step 3 must subtract it; \
         anchoring here leaves the rejoin concurrent with her leave and the Node answers 3048"
    );
    assert!(
        !found.contains(&join_id),
        "her first join is referenced by her leave, so step 3 must subtract it too"
    );
}

/// **V-2c (v1.2) — DUAL ENTITLEMENT.**
///
/// Departed **and** re-invited with a LIVE invite. Both her departure and the
/// live invite must be selected: the live invite survives step 3 because nothing
/// in the kept subset references it, so the join is chained past her leave **and**
/// `pending_invites.get(sender)` still finds the entitlement and grants the role.
///
/// 🔑 `D-154`① — the invite is the CARRIER OF THE ROLE, and this is the only
/// case where that still matters after the precedence clause was deleted. Under
/// v1.1 this returned the invite ALONE and silently dropped the departure.
///
/// 📌 The re-invite is anchored on the room create, not on her leave — alice
/// signs it against the tips of her own moment, and `fanout.rs`'s own words are
/// that the served batch is a DISCOVERY payload, not an authoritative DAG.
#[tokio::test]
async fn v2c_dual_entitlement_selects_both_her_departure_and_the_live_invite() {
    use chrono::{SecondsFormat, Utc};

    let alice = keypair::generate();
    let bob = keypair::generate();
    let bob_id = id_uri(&bob);
    let (mut events, space_id, room_id, spent_invite_id, _join_id, leave_id) =
        departed_set(&alice, &bob);

    let future =
        (Utc::now() + chrono::Duration::days(14)).to_rfc3339_opts(SecondsFormat::Millis, true);
    let mut reinvite = build_membership_event(
        &alice,
        &space_id,
        "",
        EventType::MembershipInvite,
        serde_json::json!({
            "target_identity": bob_id,
            "role": "moderator",
            "valid_until": future,
        }),
    );
    reinvite.prev_events = vec![xgid_of(&room_id)];
    let reinvite = sign_event(reinvite, &alice);
    let reinvite_id = event_id(&reinvite);
    events.push(reinvite);

    let (addr, _server) = spawn_bootstrap_server(events, false).await;
    let mut conn = connect_url(&format!("ws://{addr}/xgen")).await.expect("connect");
    conn.client_authenticate(&bob).await.expect("auth");

    let key = space_join_key(&bob, &space_id);
    let found = get_invite_bootstrap(&mut conn, &space_id, &bob_id, Some(&key), T)
        .await
        .expect("bootstrap fetch");

    let mut expected = vec![leave_id.clone(), reinvite_id.clone()];
    expected.sort();
    assert_eq!(
        found, expected,
        "a dual-entitled requester chains on BOTH her departure and the live invite: \
         dropping the departure leaves the rejoin concurrent with her own leave (3048), \
         and dropping the invite would be harmless here only by luck"
    );
    assert!(
        !found.contains(&spent_invite_id),
        "the SPENT invite is still subtracted — two invites name her and only the live one survives"
    );
}

/// **V-1 (v1.2) — CONVERGENCE, ASSERTED RATHER THAN ASSUMED.**
///
/// For a first-time invitee the key-based selection and the invite scan return
/// **the same single id**. The deleted precedence clause used to make that true
/// by rule; it is now true only because her key carries exactly ONE event — the
/// invitation.
///
/// 🛑 **That is why this is asserted explicitly.** It is a property of her
/// history, not a guarantee of the mechanism, so the day it stops holding this
/// goes RED instead of quiet. Under v1.2 the key path is the one production
/// takes; the `None` arm is the no-key fallback (`§3` step 1, `§4` G4-1), and it
/// is the only handle that isolates the invite scan.
#[tokio::test]
async fn v1_invitee_key_selection_and_invite_scan_converge_on_one_id() {
    let alice = keypair::generate();
    let bob = keypair::generate();
    let bob_id = id_uri(&bob);
    let (events, space_id, invite_id) = structural_set(&alice, &bob_id);

    // Path 1 — the key-based selection (what production takes at v1.2).
    let (addr1, _s1) = spawn_bootstrap_server(events.clone(), false).await;
    let mut c1 = connect_url(&format!("ws://{addr1}/xgen")).await.expect("connect");
    c1.client_authenticate(&bob).await.expect("auth");
    let key = space_join_key(&bob, &space_id);
    let by_key = get_invite_bootstrap(&mut c1, &space_id, &bob_id, Some(&key), T)
        .await
        .expect("bootstrap fetch");

    // Path 2 — the invite scan, reached only with no key.
    let (addr2, _s2) = spawn_bootstrap_server(events, false).await;
    let mut c2 = connect_url(&format!("ws://{addr2}/xgen")).await.expect("connect");
    c2.client_authenticate(&bob).await.expect("auth");
    let by_scan = get_invite_bootstrap(&mut c2, &space_id, &bob_id, None, T)
        .await
        .expect("bootstrap fetch");

    assert_eq!(
        by_key, by_scan,
        "the two anchor sources must agree for a first-time invitee — if they ever diverge, \
         deleting the precedence clause changed an invitee's behaviour, which it must not"
    );
    assert_eq!(
        by_key,
        vec![invite_id],
        "and the id they agree on is the invitation: her key carries exactly one event"
    );
}
