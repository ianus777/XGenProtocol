// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Arc C / M8 — Commit C3 two-node convergence smoke (runbook
//! `tasks/STATE_RESOLUTION_IMPL.md` §3 C3; SR-D5 secondary proof).
//!
//! **What this guards.** The algorithm-level convergence proof is the
//! permutation property tests in `xgen-core::resolution::derive::tests`
//! (C1); the runtime-seam proof is `m8_c2_wiring_tests` against a bare
//! `NodeRuntime` (C2). This is the *integration*-level lock: two real
//! in-process Nodes, each with its own keypair / node_id / on-disk Space
//! store, receive the **same** concurrent same-key events in **different
//! arrival orders** and must derive an identical resolved snapshot. It
//! exercises the live `InProcessNode::ingest` → `NodeRuntime::ingest_event`
//! → SR-D1 conflict gate → `derive_resolved` seam end-to-end, including the
//! per-rebuild `identity_home_nodes` build and the disk-persist side path,
//! which the C2 bare-runtime test does not.
//!
//! **Membership-core scenario (one solid over many shallow).** A Layer-1
//! ban-vs-join conflict: alice creates Space `s`; bob's `membership.join`
//! and alice's `membership.ban(bob)` both reference the create root and so
//! are causally concurrent at the same membership key (target bob). The §3.9
//! resolution gives Layer 1 (membership-removal) precedence: ban wins, bob
//! is banned and not a member — on both Nodes, regardless of the order each
//! ingested join vs ban.
//!
//! **No federation transport.** Convergence is an *arrival-order
//! independence* property at each Node independently; feeding two
//! independent Nodes deliberately different orders is the faithful
//! realisation. Federation push would impose dispatch-order delivery +
//! `prev_events` validation, defeating the "different orders" premise. The
//! `ingest` path is the one the SR-D1 gate sits on, so it is the correct
//! target.
//!
//! **D-075 note.** `derive_resolved` threads each Node's own `node_id` into
//! `apply_event` for the federation-vantage projection. This scenario emits
//! no `state.federation_add`, so `federation_nodes` stays empty on both
//! Nodes and the differing node_ids do not perturb the snapshot — the
//! convergence is exact, not modulo-vantage.

#![cfg(test)]

use serde_json::json;

use crate::{
    crypto::encoding,
    identity::keypair,
    space::state::{build_membership_event, build_space_create_event, sign_event},
    wire::types::{Event, EventType},
};
use xgen_common::xgid::{EventXgid, IdentityXgid, Xgid};

use super::phase9_harness::spawn_in_process_node;

fn pubkey_uri(k: &ed25519_dalek::SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(k.verifying_key().as_bytes())
    )
}
fn edx(s: &str) -> EventXgid {
    EventXgid::from_xgid(Xgid::new(s.to_string()))
}
fn eid(ev: &Event) -> String {
    ev.event_id.as_ref().unwrap().as_str().to_string()
}

/// `(space_create, join_bob, ban_bob, space_id, bob_id)`. `join` and `ban`
/// both reference the create root → causally concurrent, same membership
/// key (target bob). Built **once** so both Nodes ingest byte-identical
/// events and only the arrival order differs.
fn ban_join_scenario() -> (Event, Event, Event, String, String) {
    let alice = keypair::generate();
    let bob = keypair::generate();
    let bob_id = pubkey_uri(&bob);
    let create = sign_event(
        build_space_create_event(&alice, "s", None, 1, "xgen://pubkey/ed25519:HOME", None),
        &alice,
    );
    let sid = eid(&create);

    let mut join = build_membership_event(&bob, &sid, "", EventType::MembershipJoin, json!({}));
    join.prev_events = vec![edx(&sid)];
    let join = sign_event(join, &bob);

    let mut ban = build_membership_event(
        &alice,
        &sid,
        "",
        EventType::MembershipBan,
        json!({ "target_identity": bob_id }),
    );
    ban.prev_events = vec![edx(&sid)];
    let ban = sign_event(ban, &alice);

    (create, join, ban, sid, bob_id)
}

#[tokio::test]
async fn two_node_membership_conflict_converges_across_arrival_orders() {
    let (create, join, ban, sid, bob_id) = ban_join_scenario();

    // Two real Nodes — distinct keypairs, node_ids, on-disk Space stores.
    let node_a = spawn_in_process_node().await;
    let node_b = spawn_in_process_node().await;

    // A ingests create → join → ban; B ingests create → ban → join. Same
    // events, opposite conflict-arrival order.
    node_a.ingest(create.clone()).await;
    node_a.ingest(join.clone()).await;
    node_a.ingest(ban.clone()).await;

    node_b.ingest(create).await;
    node_b.ingest(ban).await;
    node_b.ingest(join).await;

    let state_a = node_a
        .space_state(&sid)
        .await
        .expect("Node A derived Space s");
    let state_b = node_b
        .space_state(&sid)
        .await
        .expect("Node B derived Space s");

    // The convergence property: identical resolved snapshot across both
    // Nodes despite the opposite arrival order at the SR-D1 gate.
    assert_eq!(
        state_a, state_b,
        "two Nodes converge to one resolved SpaceState regardless of the \
         order each received the concurrent join vs ban"
    );

    // The membership-core outcome: Layer 1 ban beats the concurrent join on
    // both Nodes — bob is banned, not a member.
    let bob = IdentityXgid::from_xgid(Xgid::new(bob_id));
    assert!(
        state_a.banned.contains(&bob),
        "ban wins at the live two-node seam (Node A)"
    );
    assert!(
        !state_a.members.contains_key(&bob),
        "the dropped join leaves bob a non-member (Node A)"
    );
    assert!(
        state_b.banned.contains(&bob),
        "ban wins at the live two-node seam (Node B)"
    );
    assert!(
        !state_b.members.contains_key(&bob),
        "the dropped join leaves bob a non-member (Node B)"
    );

    node_a.shutdown().await;
    node_b.shutdown().await;
}
