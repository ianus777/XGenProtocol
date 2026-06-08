// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! Phase 9 Compound C5 — Validation asymmetry under load.
//!
//! Test that the F-4 validation pipeline preserves per-event outcome
//! independence under load. Sends 100 mixed valid+forged events to a
//! `NodeRuntime` via direct `dispatch_event` calls in a shuffled order
//! and asserts each event's outcome matches the per-event expectation.
//!
//! Composition (Joe-locked at checkpoint #4):
//! - 50 valid: 10 per family across the 5 families (uniform distribution).
//! - 50 forged: variant-stratified across the 4 forgery variants
//!   (~12-13 per variant), each randomly assigned to one of the 5 families.
//! - **At most 1** `forged_sender_with_resign_state_federation_add` per the
//!   B3 asymmetric-cell caveat — that forgery ingests rather than rejects,
//!   so allowing many would polute the isolation property the test pins.
//!
//! Shuffle seed: `0xC5_5EED_0000_0001` (Joe-locked at checkpoint #4 —
//! mnemonic, deterministic).
//!
//! Load-bearing property: per-event outcome independence. If a forged
//! event's rejection left state that affected a subsequent valid event's
//! acceptance, this test would fail. The 100-event sequence proves
//! isolation by construction.

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
use rand::{seq::SliceRandom, SeedableRng};
use rand::rngs::StdRng;
use serde_json::json;
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};

const SHUFFLE_SEED: u64 = 0xC5_5EED_0000_0001;

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
fn sdx(s: &str) -> SpaceXgid {
    SpaceXgid::from_xgid(Xgid::new(s.to_string()))
}
fn event_id_str(ev: &Event) -> String {
    ev.event_id
        .as_ref()
        .expect("signed event has event_id")
        .as_str()
        .to_string()
}

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
        revoked: false,
        revoked_at: None,
        revocation_reason: None,
    }
}

/// Per-event expected outcome class.
#[derive(Debug, Clone)]
enum Expected {
    Accepted,
    /// Rejected with a specific Display-substring on the reason.
    Rejected(&'static str),
    /// HeldPending (F-10 unknown-signer or F-4a missing predecessor).
    HeldPending,
}

/// Setup: NodeRuntime with alice registered + 10 extra registered Identities
/// for membership.join, Space + room + federation_peer X.
///
/// Returns (rt, alice_key, joiners[10], space_id, room_id, peer_id).
fn setup_c5() -> (NodeRuntime, SigningKey, Vec<SigningKey>, String, String, String) {
    let alice = keypair::generate();
    let node_key = keypair::generate();
    let mut rt = NodeRuntime::new(node_key);
    rt.register_identity(make_record(&alice, rt.node_id.as_str())).expect("alice");

    // Register 10 extra joiners for the membership.join family.
    let mut joiners: Vec<SigningKey> = Vec::with_capacity(10);
    for _ in 0..10 {
        let k = keypair::generate();
        rt.register_identity(make_record(&k, rt.node_id.as_str())).expect("joiner");
        joiners.push(k);
    }

    let space_ev = sign_event(
        build_space_create_event(&alice, "c5-space", None, 1, rt.node_id.as_str(), None, false),
        &alice,
    );
    let space_id = event_id_str(&space_ev);
    rt.ingest_event(space_ev);

    let room_ev = sign_event(
        build_room_create_event(&alice, &space_id, "general", None),
        &alice,
    );
    let room_id = event_id_str(&room_ev);
    rt.ingest_event(room_ev);

    let peer_key = keypair::generate();
    let peer_id = pubkey_uri(&peer_key);
    let node_key_clone = rt.node_keypair.clone();
    let fed_add = sign_event(
        build_federation_add_event(
            &node_key_clone,
            &space_id,
            rt.dag_tips(&sdx(&space_id)),
            &peer_id,
            "xgen://hash/sha256:c5-setup",
            "0.1",
            "json",
        ),
        &node_key_clone,
    );
    rt.ingest_event(fed_add);

    (rt, alice, joiners, space_id, room_id, peer_id)
}

// ── Family-specific UNSIGNED event factories ─────────────────────────────────

fn unsigned_message_text(sender: &SigningKey, space_id: &str, room_id: &str, prev: Vec<String>, i: usize) -> Event {
    Event::new(
        EventType::MessageText,
        idx(&pubkey_uri(sender)),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev.into_iter()
            .map(|s| EventXgid::from_xgid(Xgid::new(s)))
            .collect(),
        format!("2026-05-25T00:00:{:02}.000Z", i % 60),
        json!({ "body": format!("c5-msg-{i}") }),
    )
}

fn unsigned_membership_join(sender: &SigningKey, space_id: &str, room_id: &str, prev: Vec<String>) -> Event {
    Event::new(
        EventType::MembershipJoin,
        idx(&pubkey_uri(sender)),
        RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
        SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
        prev.into_iter()
            .map(|s| EventXgid::from_xgid(Xgid::new(s)))
            .collect(),
        "2026-05-25T00:00:01.000Z".to_string(),
        json!({}),
    )
}

fn unsigned_membership_kick(sender: &SigningKey, space_id: &str, room_id: &str, target: &str, prev: Vec<String>) -> Event {
    let mut ev = build_membership_event(
        sender,
        space_id,
        room_id,
        EventType::MembershipKick,
        json!({ "target_identity": target }),
    );
    ev.prev_events = prev
        .into_iter()
        .map(|s| EventXgid::from_xgid(Xgid::new(s)))
        .collect();
    ev
}

fn unsigned_state_federation_add(sender: &SigningKey, space_id: &str, prev: Vec<String>, new_peer_uri: &str, sid: &str) -> Event {
    build_federation_add_event(
        sender,
        space_id,
        prev,
        new_peer_uri,
        sid,
        "0.1",
        "json",
    )
}

fn unsigned_state_room_create(sender: &SigningKey, space_id: &str, prev: Vec<String>, i: usize) -> Event {
    let mut ev = build_room_create_event(sender, space_id, &format!("c5-room-{i}"), None);
    ev.prev_events = prev
        .into_iter()
        .map(|s| EventXgid::from_xgid(Xgid::new(s)))
        .collect();
    ev
}

// ── Forgery primitives (same shape as Sc4) ───────────────────────────────────

fn forge_bad_signature(unsigned: Event, signer: &SigningKey) -> Event {
    let mut ev = sign_event(unsigned, signer);
    if let Some(sig) = ev.signature.as_mut() {
        let mut bytes = sig.clone().into_bytes();
        if let Some(last) = bytes.last_mut() {
            *last = if *last == b'A' { b'B' } else { b'A' };
        }
        *sig = String::from_utf8(bytes).expect("ascii sig");
    }
    ev
}

fn forge_mutated_event_id(unsigned: Event, signer: &SigningKey) -> Event {
    let mut ev = sign_event(unsigned, signer);
    ev.event_id = Some(EventXgid::from_xgid(Xgid::new(
        "xgen://hash/sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
    )));
    ev
}

fn forge_malformed_prev_events(mut unsigned: Event, signer: &SigningKey) -> Event {
    unsigned.prev_events = vec![];
    sign_event(unsigned, signer)
}

// ── Test ─────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn c5_validation_asymmetry_under_load_100_events_mixed_valid_and_forged() {
    let (mut rt, alice, joiners, space_id, room_id, peer_id) = setup_c5();
    let tip = rt.dag_tips(&sdx(&space_id));

    // Build 100 events: 50 valid (10 per family) + 50 forged (variant-stratified).
    let mut events: Vec<(Event, Expected)> = Vec::with_capacity(100);

    // ── 10 valid message.text ──
    for i in 0..10 {
        let ev = sign_event(unsigned_message_text(&alice, &space_id, &room_id, tip.clone(), i), &alice);
        events.push((ev, Expected::Accepted));
    }

    // ── 10 valid membership.join (each from a fresh registered joiner) ──
    for j in &joiners {
        let ev = sign_event(unsigned_membership_join(j, &space_id, &room_id, tip.clone()), j);
        events.push((ev, Expected::Accepted));
    }

    // ── 10 valid membership.kick (alice kicks fake targets; apply_kick is
    // a no-op if target isn't a member, so each kick ingests as a no-op
    // mutation — proves the validation pipeline accepts the event shape) ──
    for i in 0..10 {
        let target = format!("xgen://pubkey/ed25519:c5-kick-target-{i:056}");
        let ev = sign_event(unsigned_membership_kick(&alice, &space_id, &room_id, &target, tip.clone()), &alice);
        events.push((ev, Expected::Accepted));
    }

    // ── 10 valid state.federation_add (each adds a unique fresh peer URI) ──
    for i in 0..10 {
        let new_peer = pubkey_uri(&keypair::generate());
        let sid = format!("xgen://hash/sha256:c5-valid-fedadd-{i}");
        let ev = sign_event(
            unsigned_state_federation_add(&alice, &space_id, tip.clone(), &new_peer, &sid),
            &alice,
        );
        events.push((ev, Expected::Accepted));
    }

    // ── 10 valid state.room_create (alice creates 10 distinct rooms) ──
    for i in 0..10 {
        let ev = sign_event(unsigned_state_room_create(&alice, &space_id, vec![space_id.clone()], i), &alice);
        events.push((ev, Expected::Accepted));
    }

    // ── 50 forged: 4 variants stratified (12+13+12+13) × 5 families,
    // with at-most-1 forged_sender_with_resign_state_federation_add. ──
    //
    // Strategy: enumerate all 20 (variant, family) cells, repeat each to
    // reach ~12-13 per variant, then trim to exactly 50 and ensure the
    // forged_sender_with_resign_state_federation_add count is ≤ 1.
    //
    // Per-variant quotas chosen to sum to 50:
    //   bad_signature             — 12
    //   forged_sender_with_resign — 13 (5 families × 2 + 3, but federation_add capped at 1)
    //   mutated_event_id          — 12
    //   malformed_prev_events     — 13
    //
    // Within each variant, distribute round-robin across families.

    let families = [
        FamilyId::MessageText,
        FamilyId::MembershipJoin,
        FamilyId::MembershipKick,
        FamilyId::StateFederationAdd,
        FamilyId::StateRoomCreate,
    ];

    // 12 bad_signature, round-robin family.
    for i in 0..12 {
        let fam = families[i % 5];
        let (forged, expected) = forge_for(VariantId::BadSignature, fam, &rt, &alice, &joiners, &space_id, &room_id, tip.clone(), i);
        events.push((forged, expected));
    }

    // 13 forged_sender_with_resign — but cap state.federation_add at 1.
    // Distribute the other 12 across the remaining 4 families.
    let mut fed_add_count = 0;
    for i in 0..13 {
        let fam_idx = i % 5;
        let fam = families[fam_idx];
        // Cap federation_add at 1; shift to next family if over quota.
        let fam = if matches!(fam, FamilyId::StateFederationAdd) && fed_add_count >= 1 {
            FamilyId::MessageText
        } else {
            if matches!(fam, FamilyId::StateFederationAdd) { fed_add_count += 1; }
            fam
        };
        let (forged, expected) = forge_for(VariantId::ForgedSenderWithResign, fam, &rt, &alice, &joiners, &space_id, &room_id, tip.clone(), i);
        events.push((forged, expected));
    }

    // 12 mutated_event_id, round-robin family.
    for i in 0..12 {
        let fam = families[i % 5];
        let (forged, expected) = forge_for(VariantId::MutatedEventId, fam, &rt, &alice, &joiners, &space_id, &room_id, tip.clone(), i);
        events.push((forged, expected));
    }

    // 13 malformed_prev_events, round-robin family.
    for i in 0..13 {
        let fam = families[i % 5];
        let (forged, expected) = forge_for(VariantId::MalformedPrevEvents, fam, &rt, &alice, &joiners, &space_id, &room_id, tip.clone(), i);
        events.push((forged, expected));
    }

    assert_eq!(events.len(), 100, "composition error — must be exactly 100 events");

    // Shuffle with deterministic seed.
    let mut rng = StdRng::seed_from_u64(SHUFFLE_SEED);
    events.shuffle(&mut rng);

    // Dispatch each event; assert outcome matches expected.
    for (idx, (ev, expected)) in events.into_iter().enumerate() {
        let outcome = rt.dispatch_event(ev, EventOrigin::ReceivedViaFederation, Some(&ndx(&peer_id)));
        match expected {
            Expected::Accepted => assert!(
                matches!(outcome, DispatchOutcome::Accepted { .. }),
                "event #{idx}: expected Accepted, got {outcome:?}"
            ),
            Expected::Rejected(substring) => {
                assert!(
                    matches!(outcome, DispatchOutcome::Rejected(_)),
                    "event #{idx}: expected Rejected({substring}), got {outcome:?}"
                );
                let DispatchOutcome::Rejected(reason) = outcome else { unreachable!() };
                let reason = reason.reason;
                assert!(
                    reason.contains(substring),
                    "event #{idx}: expected reason to contain '{substring}', got '{reason}'"
                );
            }
            Expected::HeldPending => assert!(
                matches!(outcome, DispatchOutcome::HeldPending),
                "event #{idx}: expected HeldPending, got {outcome:?}"
            ),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum FamilyId {
    MessageText,
    MembershipJoin,
    MembershipKick,
    StateFederationAdd,
    StateRoomCreate,
}

#[derive(Clone, Copy, Debug)]
enum VariantId {
    BadSignature,
    ForgedSenderWithResign,
    MutatedEventId,
    MalformedPrevEvents,
}

#[allow(clippy::too_many_arguments)]
fn forge_for(
    variant: VariantId,
    family: FamilyId,
    _rt: &NodeRuntime,
    alice: &SigningKey,
    joiners: &[SigningKey],
    space_id: &str,
    room_id: &str,
    tip: Vec<String>,
    idx: usize,
) -> (Event, Expected) {
    // For variants 1/3/4 the signer matches the family's sender; for variant 2
    // (forged_sender_with_resign) we use an attacker key NOT in registry.
    match variant {
        VariantId::BadSignature => {
            let unsigned = build_unsigned_for(family, alice, joiners, space_id, room_id, tip, idx, alice);
            let signer_for_family = signer_for(family, alice, joiners, idx);
            (forge_bad_signature(unsigned, signer_for_family), Expected::Rejected("step 12: signature verification failed"))
        }
        VariantId::MutatedEventId => {
            let unsigned = build_unsigned_for(family, alice, joiners, space_id, room_id, tip, idx, alice);
            let signer_for_family = signer_for(family, alice, joiners, idx);
            (forge_mutated_event_id(unsigned, signer_for_family), Expected::Rejected("step 8: event_id does not match canonical content hash"))
        }
        VariantId::MalformedPrevEvents => {
            let unsigned = build_unsigned_for(family, alice, joiners, space_id, room_id, tip, idx, alice);
            let signer_for_family = signer_for(family, alice, joiners, idx);
            (forge_malformed_prev_events(unsigned, signer_for_family), Expected::Rejected("step 10: DAG structural violation"))
        }
        VariantId::ForgedSenderWithResign => {
            // Attacker key NOT in IdentityRegistry. Used as both sender and signer.
            let attacker = keypair::generate();
            let unsigned = build_unsigned_for(family, alice, joiners, space_id, room_id, tip, idx, &attacker);
            let forged = sign_event(unsigned, &attacker);
            // Variant 2 expectation: HeldPending for 4 families; Accepted for state.federation_add (B3 asymmetric).
            let expected = match family {
                FamilyId::StateFederationAdd => Expected::Accepted,
                _ => Expected::HeldPending,
            };
            (forged, expected)
        }
    }
    // rt parameter is unused but kept in signature for future symmetric expansion.
    // Mark via let-binding to satisfy the linter's unused-parameter check.
    // (rt is &NodeRuntime so it cannot be modified; the &-bind is cheap.)
}

fn signer_for<'a>(family: FamilyId, alice: &'a SigningKey, joiners: &'a [SigningKey], idx: usize) -> &'a SigningKey {
    match family {
        FamilyId::MembershipJoin => &joiners[idx % joiners.len()],
        _ => alice,
    }
}

/// Build an unsigned event of the given family with `sender_for_event` as sender.
/// `attacker_or_alice` controls the sender URI (alice for variants 1/3/4; attacker
/// for variant 2).
#[allow(clippy::too_many_arguments)]
fn build_unsigned_for(
    family: FamilyId,
    alice: &SigningKey,
    joiners: &[SigningKey],
    space_id: &str,
    room_id: &str,
    tip: Vec<String>,
    idx: usize,
    sender_for_event: &SigningKey,
) -> Event {
    match family {
        FamilyId::MessageText => unsigned_message_text(sender_for_event, space_id, room_id, tip, idx),
        FamilyId::MembershipJoin => {
            // For variant 2 (forged_sender), sender = attacker. For others,
            // sender = fresh joiner (registered). signer_for() returns the
            // matching key.
            let _ = alice;
            let _ = joiners;
            unsigned_membership_join(sender_for_event, space_id, room_id, tip)
        }
        FamilyId::MembershipKick => {
            let target = format!("xgen://pubkey/ed25519:c5-fkick-{idx:055}");
            unsigned_membership_kick(sender_for_event, space_id, room_id, &target, tip)
        }
        FamilyId::StateFederationAdd => {
            let new_peer = pubkey_uri(&keypair::generate());
            let sid = format!("xgen://hash/sha256:c5-fadd-{idx}");
            unsigned_state_federation_add(sender_for_event, space_id, tip, &new_peer, &sid)
        }
        FamilyId::StateRoomCreate => unsigned_state_room_create(sender_for_event, space_id, vec![space_id.to_string()], idx),
    }
}
