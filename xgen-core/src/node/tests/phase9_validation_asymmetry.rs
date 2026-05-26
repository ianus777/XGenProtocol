// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! Phase 9 Scenario 4 — F-4 validation asymmetry across the 5 event families.
//!
//! 20 tests: 4 forgery variants × 5 event families. Each test forges one
//! property of an Event for one family and asserts the variant's expected
//! outcome from `NodeRuntime::dispatch_event`.
//!
//! Contract anchors:
//! - 4 variants per runbook §4.2 (J-113 enumeration; variant 2 refined J-115).
//! - 5 families per runbook §4.3.
//! - Variants 1/3/4: `DispatchOutcome::Rejected(String)` tuple with
//!   ExchangeError Display substring. Production at
//!   `xgen-core/src/node/runtime.rs:88-95` (DispatchOutcome enum) +
//!   `xgen-core/src/message/exchange.rs:46-87` (ExchangeError + Display).
//! - Variant 2 for 4 families: `DispatchOutcome::HeldPending` unit + PendingBuffer
//!   inspection (F-10 unknown-signer per Phase 6, J-087).
//! - Variant 2 for state.federation_add: `DispatchOutcome::Accepted` + ingest
//!   (Phase 7 B3 asymmetric cell, J-088).
//!
//! Helpers (within-file per runbook §1.1 Finding A — no shared library API):
//! - `setup_runtime_with_alice_in_space` — common fixture.
//! - `forge_bad_signature`, `forge_mutated_event_id`, `forge_malformed_prev_events`,
//!   `forge_resigned_with_attacker` — per-variant primitives.
//! - `build_*_for_family` — per-family unsigned-event factories.

use crate::{
    crypto::encoding,
    identity::{keypair, registry::IdentityRecord},
    node::runtime::{DispatchOutcome, EventOrigin, NodeRuntime},
    space::state::{
        build_federation_add_event, build_membership_event, build_room_create_event,
        build_space_create_event, sign_event,
    },
    wire::types::{Event, EventType},
};
use ed25519_dalek::SigningKey;
use serde_json::json;
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};

// ── Common helpers ───────────────────────────────────────────────────────────

/// Derive an Identity URI from an Ed25519 signing key.
fn pubkey_uri(key: &SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(key.verifying_key().as_bytes())
    )
}

// ── Pass 1 Commit 4a typed XGID test helpers ─────────────────────────────────
fn idx(s: &str) -> IdentityXgid {
    IdentityXgid::from_xgid(Xgid::new(s.to_string()))
}
fn ndx(s: &str) -> NodeXgid {
    NodeXgid::from_xgid(Xgid::new(s.to_string()))
}
fn event_id_str(ev: &Event) -> String {
    ev.event_id
        .as_ref()
        .expect("signed event has event_id")
        .as_str()
        .to_string()
}

/// Build a minimal IdentityRecord for a registered (human) Identity.
fn make_record(key: &SigningKey, home_node: &str) -> IdentityRecord {
    IdentityRecord {
        identity_id: idx(&pubkey_uri(key)),
        display_name: None,
        is_ai: false,
        ai_capabilities: None,
        registered_at: "2026-05-25T00:00:00.000Z".to_string(),
        trust_assertion: None,
        devices: vec![],
        home_node: ndx(home_node),
        update_version: 0,
    }
}

/// Setup: NodeRuntime B with Alice registered + Alice-owned Space + Alice-created
/// general Room + federation_peer X in S's federation_nodes (so F-3 passes
/// for federation-channel dispatches of non-create event families).
///
/// Returns (rt, alice_key, space_id, room_id, peer_node_id).
fn setup_runtime_with_alice_in_space() -> (NodeRuntime, SigningKey, String, String, String) {
    let alice = keypair::generate();
    let node_key = keypair::generate();
    let mut rt = NodeRuntime::new(node_key);
    rt.register_identity(make_record(&alice, &rt.node_id))
        .expect("alice registration");

    // Space (DAG root for S).
    let space_ev = sign_event(
        build_space_create_event(&alice, "sc4-space", None, 1, &rt.node_id),
        &alice,
    );
    let space_id = event_id_str(&space_ev);
    rt.ingest_event(space_ev);

    // Room (non-root under D-076 v1.1; predecessor is the space_create).
    let room_ev = sign_event(
        build_room_create_event(&alice, &space_id, "general", None),
        &alice,
    );
    let room_id = event_id_str(&room_ev);
    rt.ingest_event(room_ev);

    // Federation peer X in S's federation_nodes (so F-3 passes for the 4
    // non-create families dispatched via federation channel).
    let peer_key = keypair::generate();
    let peer_node_id = pubkey_uri(&peer_key);
    let node_key_clone = rt.node_keypair.clone();
    let fed_add = sign_event(
        build_federation_add_event(
            &node_key_clone,
            &space_id,
            rt.dag_tips(&space_id),
            &peer_node_id,
            "xgen://hash/sha256:setup-session",
            "0.1",
            "json",
        ),
        &node_key_clone,
    );
    rt.ingest_event(fed_add);
    assert!(
        rt.spaces[&space_id]
            .federation_nodes
            .iter()
            .any(|n| n.as_str() == peer_node_id.as_str()),
        "setup: peer must be in federation_nodes after federation_add ingest"
    );

    (rt, alice, space_id, room_id, peer_node_id)
}

// ── Per-family unsigned-event factories ──────────────────────────────────────
//
// Each factory builds an UNSIGNED event with realistic content + correct
// (non-empty for non-roots) prev_events. Signing/forgery is applied by the
// variant helpers below.
//
// `sender_key` controls the sender URI on the event. For variants 1/3/4 the
// signer is the same key (alice or peer_node_key as appropriate). For variant
// 2 (forged_sender_with_resign) the signer is an attacker key NOT in registry.

fn build_message_text_for_family(sender_key: &SigningKey, space_id: &str, room_id: &str, prev: Vec<String>) -> Event {
    Event::new(
        EventType::MessageText,
        idx(&pubkey_uri(sender_key)),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev.into_iter()
            .map(|s| EventXgid::from_xgid(Xgid::new(s)))
            .collect(),
        "2026-05-25T00:00:01.000Z".to_string(),
        json!({ "body": "hello" }),
    )
}

fn build_membership_join_for_family(sender_key: &SigningKey, space_id: &str, room_id: &str, prev: Vec<String>) -> Event {
    Event::new(
        EventType::MembershipJoin,
        idx(&pubkey_uri(sender_key)),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev.into_iter()
            .map(|s| EventXgid::from_xgid(Xgid::new(s)))
            .collect(),
        "2026-05-25T00:00:01.000Z".to_string(),
        json!({}),
    )
}

fn build_membership_kick_for_family(sender_key: &SigningKey, space_id: &str, room_id: &str, target: &str, prev: Vec<String>) -> Event {
    let mut ev = build_membership_event(
        sender_key,
        space_id,
        room_id,
        EventType::MembershipKick,
        json!({ "target": target }),
    );
    ev.prev_events = prev
        .into_iter()
        .map(|s| EventXgid::from_xgid(Xgid::new(s)))
        .collect();
    ev
}

fn build_state_federation_add_for_family(sender_key: &SigningKey, space_id: &str, prev: Vec<String>, new_peer_uri: &str) -> Event {
    build_federation_add_event(
        sender_key,
        space_id,
        prev,
        new_peer_uri,
        "xgen://hash/sha256:forged-session",
        "0.1",
        "json",
    )
}

fn build_state_room_create_for_family(sender_key: &SigningKey, space_id: &str, prev: Vec<String>) -> Event {
    let mut ev = build_room_create_event(sender_key, space_id, "forged-room", None);
    // build_room_create_event hard-codes prev_events = vec![space_id] under
    // D-076 v1.1. Override per the variant's needs (malformed_prev_events
    // sets vec![] to trigger step 10; others retain the canonical predecessor).
    ev.prev_events = prev
        .into_iter()
        .map(|s| EventXgid::from_xgid(Xgid::new(s)))
        .collect();
    ev
}

// ── Per-variant forgery primitives ───────────────────────────────────────────

/// Variant 1 — sign with the real key, then corrupt the signature string so
/// step 12 verification fails.
fn forge_bad_signature(unsigned: Event, signer: &SigningKey) -> Event {
    let mut ev = sign_event(unsigned, signer);
    // Mutate the signature payload while keeping the prefix shape valid so
    // signature parsing reaches the cryptographic check. The corruption is a
    // single byte flip in the base64 payload portion.
    if let Some(sig) = ev.signature.as_mut() {
        let mut bytes = sig.clone().into_bytes();
        if let Some(last) = bytes.last_mut() {
            *last = if *last == b'A' { b'B' } else { b'A' };
        }
        *sig = String::from_utf8(bytes).expect("ascii signature payload");
    }
    ev
}

/// Variant 3 — sign with the real key, then mutate the event_id so step 8
/// canonical-hash comparison fails.
fn forge_mutated_event_id(unsigned: Event, signer: &SigningKey) -> Event {
    let mut ev = sign_event(unsigned, signer);
    ev.event_id = Some(EventXgid::from_xgid(Xgid::new(
        "xgen://hash/sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
    )));
    ev
}

/// Variant 4 — set prev_events to vec![] before signing. For all 5 families
/// (none of which are DAG roots under D-076 v1.1), step 10 rejects with
/// DagError "non-root event must reference at least one predecessor".
///
/// Construction-time helper: caller-supplied factory builds the event with
/// some prev_events, which this helper overwrites to vec![] before signing.
fn forge_malformed_prev_events(mut unsigned: Event, signer: &SigningKey) -> Event {
    unsigned.prev_events = vec![];
    sign_event(unsigned, signer)
}

// ── Scenario 4 — Variant 1: bad_signature ────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn bad_signature_message_text() {
    let (mut rt, alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    let unsigned = build_message_text_for_family(&alice, &space_id, &room_id, rt.dag_tips(&space_id));
    let forged = forge_bad_signature(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id post-sign").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 12: signature verification failed"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn bad_signature_membership_join() {
    let (mut rt, alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    let unsigned = build_membership_join_for_family(&alice, &space_id, &room_id, rt.dag_tips(&space_id));
    let forged = forge_bad_signature(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id post-sign").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 12: signature verification failed"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn bad_signature_membership_kick() {
    let (mut rt, alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    let target = pubkey_uri(&keypair::generate());
    let unsigned = build_membership_kick_for_family(&alice, &space_id, &room_id, &target, rt.dag_tips(&space_id));
    let forged = forge_bad_signature(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id post-sign").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 12: signature verification failed"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn bad_signature_state_federation_add() {
    // For federation_add via federation channel, B3 skips steps 9 + 11 + 13.
    // Steps 8 + 10 + 12 fire. The sender must be a Node URI signing with its
    // own key (signature self-verification per B3 authority chain). We use the
    // setup's federation peer as the sender.
    let (mut rt, _alice, space_id, _room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    // Reconstruct the peer's key from a fresh pair tied to peer_id... but the
    // setup's peer_key isn't returned. Generate a NEW peer Node key, register
    // it as a federation_node so F-3 (which still applies for our forgery
    // purposes? — no, F-3 skips federation_add anyway), and use it as sender.
    //
    // Simpler approach: generate a fresh peer Node key here, use its URI as
    // both the sender of the federation_add AND as the peer_node_id passed to
    // dispatch_event. The federation peer in `setup_runtime_with_alice_in_space`
    // need not match — F-3 skips federation_add, so peer-in-federation_nodes
    // check is bypassed for this event type.
    let _ = peer_id;
    let signing_peer = keypair::generate();
    let signing_peer_uri = pubkey_uri(&signing_peer);
    let new_peer_to_add = pubkey_uri(&keypair::generate());

    let unsigned = build_state_federation_add_for_family(
        &signing_peer,
        &space_id,
        rt.dag_tips(&space_id),
        &new_peer_to_add,
    );
    let forged = forge_bad_signature(unsigned, &signing_peer);
    let forged_id = forged.event_id.as_deref().expect("event_id post-sign").to_string();

    let outcome = rt.dispatch_event(
        forged,
        EventOrigin::ReceivedViaFederation,
        Some(signing_peer_uri.as_str()),
    );

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 12: signature verification failed"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn bad_signature_state_room_create() {
    let (mut rt, alice, space_id, _room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    // Alice creates a new room (she's a Space member with create permission).
    // Predecessor: the space_create event_id (D-076 v1.1 — state.room_create
    // is non-root with sole predecessor being state.space_create).
    let unsigned = build_state_room_create_for_family(&alice, &space_id, vec![space_id.clone()]);
    let forged = forge_bad_signature(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id post-sign").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 12: signature verification failed"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

// ── Scenario 4 — Variant 2: forged_sender_with_resign ────────────────────────
//
// Attacker key NOT in IdentityRegistry. Sender = attacker_uri. Sign with
// attacker's own key. Step 12 passes (sig matches embedded pubkey) but step
// 11 unknown-signer route fires → HeldPending with missing_identity per F-10
// (Phase 6, J-087). The 4 non-fed-add families hit this path. state.federation_add
// via federation channel hits B3 skip → Accepted.

#[tokio::test(flavor = "current_thread")]
async fn forged_sender_with_resign_message_text() {
    let (mut rt, _alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let attacker = keypair::generate();
    let attacker_uri = pubkey_uri(&attacker);
    let baseline_store_len = rt.stores[&space_id].len();
    let baseline_pending_identity = rt
        .pending
        .get(&space_id)
        .map(|b| b.pending_identity_count())
        .unwrap_or(0);

    let unsigned = build_message_text_for_family(&attacker, &space_id, &room_id, rt.dag_tips(&space_id));
    let forged = sign_event(unsigned, &attacker);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::HeldPending), "expected HeldPending (F-10), got {:?}", outcome);
    let buf = rt.pending.get(&space_id).expect("PendingBuffer must exist after F-10 buffering");
    assert!(buf.contains(&forged_id), "forged event must be in PendingBuffer");
    assert!(
        buf.pending_identity_count() > baseline_pending_identity,
        "pending_identity_count must increment (was {baseline_pending_identity}, now {})",
        buf.pending_identity_count()
    );
    assert!(!rt.stores[&space_id].contains(&forged_id), "HeldPending != Accepted");
    assert_eq!(rt.stores[&space_id].len(), baseline_store_len);
    let _ = attacker_uri; // attacker_uri unused at this assertion level per Option ε.iii lock at J-116
}

#[tokio::test(flavor = "current_thread")]
async fn forged_sender_with_resign_membership_join() {
    let (mut rt, _alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let attacker = keypair::generate();
    let attacker_uri = pubkey_uri(&attacker);
    let baseline_store_len = rt.stores[&space_id].len();
    let baseline_pending_identity = rt
        .pending
        .get(&space_id)
        .map(|b| b.pending_identity_count())
        .unwrap_or(0);

    let unsigned = build_membership_join_for_family(&attacker, &space_id, &room_id, rt.dag_tips(&space_id));
    let forged = sign_event(unsigned, &attacker);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::HeldPending), "expected HeldPending (F-10), got {:?}", outcome);
    let buf = rt.pending.get(&space_id).expect("PendingBuffer must exist");
    assert!(buf.contains(&forged_id));
    assert!(buf.pending_identity_count() > baseline_pending_identity);
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_store_len);
    let _ = attacker_uri;
}

#[tokio::test(flavor = "current_thread")]
async fn forged_sender_with_resign_membership_kick() {
    let (mut rt, _alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let attacker = keypair::generate();
    let attacker_uri = pubkey_uri(&attacker);
    let target = pubkey_uri(&keypair::generate());
    let baseline_store_len = rt.stores[&space_id].len();
    let baseline_pending_identity = rt
        .pending
        .get(&space_id)
        .map(|b| b.pending_identity_count())
        .unwrap_or(0);

    let unsigned = build_membership_kick_for_family(&attacker, &space_id, &room_id, &target, rt.dag_tips(&space_id));
    let forged = sign_event(unsigned, &attacker);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::HeldPending), "expected HeldPending (F-10), got {:?}", outcome);
    let buf = rt.pending.get(&space_id).expect("PendingBuffer must exist");
    assert!(buf.contains(&forged_id));
    assert!(buf.pending_identity_count() > baseline_pending_identity);
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_store_len);
    let _ = attacker_uri;
}

#[tokio::test(flavor = "current_thread")]
async fn forged_sender_with_resign_state_federation_add() {
    // Phase 7 B3 asymmetric cell: federation_add via federation channel SKIPS
    // step 9 (predecessor presence) + step 11 (sender registration + sender
    // membership) + step 13 (sender permission). Steps 8 + 10 + 12 fire. The
    // attacker's signature passes step 12 (sender = attacker_uri, signed with
    // attacker_key → embedded pubkey matches). Event INGESTS.
    //
    // This is the security property the test pins per findings v1.5 §2.4.2 +
    // design doc §6.4: the protocol does NOT prevent an attacker who already
    // has a federation channel from ingesting forged federation_add events via
    // that channel. Authority chain at the federation layer is session-level
    // handshake + signature self-verification.
    let (mut rt, _alice, space_id, _room_id, _peer_id) = setup_runtime_with_alice_in_space();
    let attacker = keypair::generate();
    let attacker_uri = pubkey_uri(&attacker);
    let new_peer_to_add = pubkey_uri(&keypair::generate());
    let baseline_store_len = rt.stores[&space_id].len();

    let unsigned = build_state_federation_add_for_family(
        &attacker,
        &space_id,
        rt.dag_tips(&space_id),
        &new_peer_to_add,
    );
    let forged = sign_event(unsigned, &attacker);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(
        forged,
        EventOrigin::ReceivedViaFederation,
        Some(attacker_uri.as_str()),
    );

    assert!(
        matches!(outcome, DispatchOutcome::Accepted { .. }),
        "expected Accepted (Phase 7 B3 asymmetric cell), got {:?}",
        outcome
    );
    assert!(
        rt.stores[&space_id].contains(&forged_id),
        "forged federation_add must be in EventStore per Phase 7 B3 design property"
    );
    assert_eq!(rt.stores[&space_id].len(), baseline_store_len + 1);
}

#[tokio::test(flavor = "current_thread")]
async fn forged_sender_with_resign_state_room_create() {
    let (mut rt, _alice, space_id, _room_id, peer_id) = setup_runtime_with_alice_in_space();
    let attacker = keypair::generate();
    let attacker_uri = pubkey_uri(&attacker);
    let baseline_store_len = rt.stores[&space_id].len();
    let baseline_pending_identity = rt
        .pending
        .get(&space_id)
        .map(|b| b.pending_identity_count())
        .unwrap_or(0);

    let unsigned = build_state_room_create_for_family(&attacker, &space_id, vec![space_id.clone()]);
    let forged = sign_event(unsigned, &attacker);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::HeldPending), "expected HeldPending (F-10), got {:?}", outcome);
    let buf = rt.pending.get(&space_id).expect("PendingBuffer must exist");
    assert!(buf.contains(&forged_id));
    assert!(buf.pending_identity_count() > baseline_pending_identity);
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_store_len);
    let _ = attacker_uri;
}

// ── Scenario 4 — Variant 3: mutated_event_id ─────────────────────────────────
//
// Step 8 (canonical-hash check) fires first in validate_event. Mutating the
// event_id after signing makes the hash comparison fail before any later step.
// Sender details don't matter for the rejection outcome; alice is used for
// consistency. For state.federation_add, B3 doesn't skip step 8.

#[tokio::test(flavor = "current_thread")]
async fn mutated_event_id_message_text() {
    let (mut rt, alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    let unsigned = build_message_text_for_family(&alice, &space_id, &room_id, rt.dag_tips(&space_id));
    let forged = forge_mutated_event_id(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 8: event_id does not match canonical content hash"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn mutated_event_id_membership_join() {
    let (mut rt, alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    let unsigned = build_membership_join_for_family(&alice, &space_id, &room_id, rt.dag_tips(&space_id));
    let forged = forge_mutated_event_id(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 8: event_id does not match canonical content hash"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn mutated_event_id_membership_kick() {
    let (mut rt, alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();
    let target = pubkey_uri(&keypair::generate());

    let unsigned = build_membership_kick_for_family(&alice, &space_id, &room_id, &target, rt.dag_tips(&space_id));
    let forged = forge_mutated_event_id(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 8: event_id does not match canonical content hash"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn mutated_event_id_state_federation_add() {
    let (mut rt, _alice, space_id, _room_id, _peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    let signing_peer = keypair::generate();
    let signing_peer_uri = pubkey_uri(&signing_peer);
    let new_peer_to_add = pubkey_uri(&keypair::generate());

    let unsigned = build_state_federation_add_for_family(
        &signing_peer,
        &space_id,
        rt.dag_tips(&space_id),
        &new_peer_to_add,
    );
    let forged = forge_mutated_event_id(unsigned, &signing_peer);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(
        forged,
        EventOrigin::ReceivedViaFederation,
        Some(signing_peer_uri.as_str()),
    );

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 8: event_id does not match canonical content hash"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn mutated_event_id_state_room_create() {
    let (mut rt, alice, space_id, _room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    let unsigned = build_state_room_create_for_family(&alice, &space_id, vec![space_id.clone()]);
    let forged = forge_mutated_event_id(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 8: event_id does not match canonical content hash"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

// ── Scenario 4 — Variant 4: malformed_prev_events ────────────────────────────
//
// Step 10 (DAG structural rules) fires before step 9/11/12. For all 5
// families (none are DAG roots under D-076 v1.1), empty prev_events triggers
// "non-root event must reference at least one predecessor". Step 10 catches
// before step 11, so sender membership concerns don't affect outcome. For
// state.federation_add, B3 skips 9/11/13 but NOT 10.

#[tokio::test(flavor = "current_thread")]
async fn malformed_prev_events_message_text() {
    let (mut rt, alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    let unsigned = build_message_text_for_family(&alice, &space_id, &room_id, rt.dag_tips(&space_id));
    let forged = forge_malformed_prev_events(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 10: DAG structural violation"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_prev_events_membership_join() {
    let (mut rt, alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    let unsigned = build_membership_join_for_family(&alice, &space_id, &room_id, rt.dag_tips(&space_id));
    let forged = forge_malformed_prev_events(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 10: DAG structural violation"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_prev_events_membership_kick() {
    let (mut rt, alice, space_id, room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();
    let target = pubkey_uri(&keypair::generate());

    let unsigned = build_membership_kick_for_family(&alice, &space_id, &room_id, &target, rt.dag_tips(&space_id));
    let forged = forge_malformed_prev_events(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 10: DAG structural violation"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_prev_events_state_federation_add() {
    let (mut rt, _alice, space_id, _room_id, _peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    let signing_peer = keypair::generate();
    let signing_peer_uri = pubkey_uri(&signing_peer);
    let new_peer_to_add = pubkey_uri(&keypair::generate());

    let unsigned = build_state_federation_add_for_family(
        &signing_peer,
        &space_id,
        rt.dag_tips(&space_id),
        &new_peer_to_add,
    );
    let forged = forge_malformed_prev_events(unsigned, &signing_peer);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(
        forged,
        EventOrigin::ReceivedViaFederation,
        Some(signing_peer_uri.as_str()),
    );

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 10: DAG structural violation"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_prev_events_state_room_create() {
    let (mut rt, alice, space_id, _room_id, peer_id) = setup_runtime_with_alice_in_space();
    let baseline_count = rt.stores[&space_id].len();

    // Build with the canonical predecessor first, then forge_malformed_prev_events
    // overwrites prev_events to vec![] before signing.
    let unsigned = build_state_room_create_for_family(&alice, &space_id, vec![space_id.clone()]);
    let forged = forge_malformed_prev_events(unsigned, &alice);
    let forged_id = forged.event_id.as_deref().expect("event_id").to_string();

    let outcome = rt.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id.as_str()));

    assert!(matches!(outcome, DispatchOutcome::Rejected(_)), "expected Rejected, got {:?}", outcome);
    let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
    assert!(reason.contains("step 10: DAG structural violation"), "got reason: {reason}");
    assert!(!rt.stores[&space_id].contains(&forged_id));
    assert_eq!(rt.stores[&space_id].len(), baseline_count);
}
