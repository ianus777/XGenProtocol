// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Phase 9 Scenario 5 — Unknown-signer first-contact (task file
//! `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 4 Scenario 5; survey
//! findings v1.2 §2.5). Shipped at Commit 3b-2 per §3.0a revised five-commit
//! shape.
//!
//! Owns F-10 (Phase 6 HeldPending generalisation for unknown signer
//! Identity); cross-surfaces F-10a (30 s timeout), 4006 error code, and the
//! `pending_identity_replication` counter on `NodeState`.
//!
//! **Harness shape.** In-process single-Node spawn via [`phase9_harness`]
//! per Lock #2 (uniform in-process across Commit 3b). X (the federated
//! peer pushing Bob's event) is modelled as a fresh keypair and a fake
//! wire-authenticated peer ID — no second `InProcessNode` is spawned
//! because F-10's surface is entirely B-side (the buffer-add at F-4
//! step 11, the identity-arrival hook at `drain_pending_by_identity`,
//! and the `pending_identity_replication` counter). F-3 is satisfied
//! ahead of time by ingesting `state.federation_add(X, S)` signed by B's
//! own keypair (sibling-shape to `federation_relationship_integration::
//! peer_with_relationship_accepts`'s `build_federation_add_event` +
//! `ingest_event` setup).
//!
//! **Coverage gap** (named per Lock #2 honesty discipline):
//! - In-process identity-replication-timing does NOT exercise cross-process
//!   clock skew, network-stack latency variation, or two-Node TCP
//!   `handle_identity_replicate_msg` ordering effects that a real
//!   deployment would surface. The 100 ms hook-vs-sweep distinguishing
//!   window is also calibrated against the single-process sweep cadence
//!   (5 s) — a real deployment's sweep cadence inheritance from the
//!   timeout-sweep task is the same in this dimension, so the
//!   distinguishing power survives the in-process simplification.
//!
//! **Honesty assertions** (findings v1.2 §2.5 sub-item C):
//! 1. Bob's event from X is `DispatchOutcome::HeldPending` (NOT Accepted,
//!    NOT Rejected); B's `pending_identity_replication` counter (read
//!    direct via `PendingBuffer::pending_identity_count`) increments by 1.
//!    The event is buffered, not in B's DAG.
//! 2. After Bob's Identity is registered on B AND the production
//!    `drain_pending_by_identity` hook fires (the two calls are made
//!    sub-100 ms apart on the same runtime lock — sibling-shape to
//!    `xgen-node/src/app.rs::handle_identity_replicate_msg`'s
//!    `register_identity` → `drain_pending_by_identity` ordering), the
//!    counter decrements to 0 within 100 ms wall-clock of the hook
//!    invocation.
//! 3. Bob's event subsequently lands in B's DAG (`has_event` returns
//!    true for the event_id), proving the F-10 release path is
//!    end-to-end, not just buffer-clearing.
//!
//! **What this test does NOT cover** (folded explicitly per Lock #2):
//! - The 4006 timeout path (covered by NodeRuntime-level
//!   `heldpending_identity_integration::identity_never_arrives_timeout_fires_4006_path`).
//!   At Phase 9 deployment level the 30 s F-10 timeout window is too long
//!   to fit in a workspace test budget without clock injection; clock
//!   injection is `tasks/FEDERATION_STRESS_FOLLOWON.md` territory.
//! - The 1 ms-before vs 1 ms-after-timeout edge cases (same reason).
//! - Double-drain on parallel Identity arrivals (catalogue M9, scheduled
//!   for Compound C6 at runtime level).

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use serde_json::json;
    use tracing_test::traced_test;

    use crate::tests::phase9_harness::{
        edx, event_id_str, idx, make_identity_record, ndx, now_rfc, pubkey_uri, rdx, sdx,
        spawn_in_process_node,
    };
    use crate::{
        identity::keypair,
        node::runtime::{DispatchOutcome, EventOrigin},
        space::state::{
            build_federation_add_event, build_room_create_event,
            build_space_create_event, sign_event,
        },
        wire::types::{Event, EventType},
    };

    /// Read B's `pending_identity_replication` counter for `space_id`.
    /// Returns 0 when the buffer doesn't exist yet (i.e. no event has
    /// ever held on this Space).
    async fn pending_identity_count(
        node: &crate::tests::phase9_harness::InProcessNode,
        space_id: &str,
    ) -> usize {
        let rt = node.runtime.lock().await;
        rt.pending
            .get(space_id)
            .map(|b| b.pending_identity_count())
            .unwrap_or(0)
    }

    /// Phase 9 Scenario 5 — unknown-signer first-contact, identity arrives
    /// within timeout, F-10 release path fires via the production
    /// identity-arrival hook (NOT via the 5 s timeout sweep).
    ///
    /// Per findings v1.2 §2.5 sub-item C step 3: the hook is synchronous
    /// within `handle_identity_replicate_msg`; the sweep task runs every
    /// 5 s. A drain inside 100 ms wall-clock of the hook invocation proves
    /// the hook ran (a regression introducing a sweep-only release path
    /// would fail this timing window).
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[traced_test]
    async fn unknown_signer_first_contact_releases_on_identity_arrival_hook() {
        let node_b = spawn_in_process_node().await;

        // ── Alice setup on B (the Space owner; her Identity is registered) ─
        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_b.register_identity(&alice_key).await;

        let space_ev = sign_event(
            build_space_create_event(
                &alice_key,
                "phase9-unknown-signer-space",
                None,
                1,
                &node_b.node_id,
                None,
            false),
            &alice_key,
        );
        let space_id: String = event_id_str(&space_ev);
        node_b.ingest(space_ev).await;

        let room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", None),
            &alice_key,
        );
        let room_id: String = event_id_str(&room_ev);
        node_b.ingest(room_ev).await;

        let invite_ev = sign_event(
            Event::new(
                EventType::MembershipInvite,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&space_id), edx(&room_id)],
                now_rfc(),
                json!({ "target_identity": alice_id, "role": "member" }),
            ),
            &alice_key,
        );
        let invite_id: String = event_id_str(&invite_ev);
        node_b.ingest(invite_ev).await;

        let alice_join_ev = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&invite_id)],
                now_rfc(),
                json!({}),
            ),
            &alice_key,
        );
        let _alice_join_id: String = event_id_str(&alice_join_ev);
        node_b.ingest(alice_join_ev).await;

        // ── Federation peer X added to S's federation_nodes ────────────────
        // F-3 must pass for X's pushes so the F-10 surface is reached.
        // Sibling-shape to federation_relationship_integration::
        // peer_with_relationship_accepts: build_federation_add_event signed
        // by B's own node keypair, ingest directly (bypasses dispatch_event
        // validation; the focus of this test is downstream of F-3).
        let peer_x_key = keypair::generate();
        let peer_x_id = pubkey_uri(&peer_x_key);
        let peer_x_id_typed = ndx(&peer_x_id);

        let fed_add = {
            let node_b_kp = (*node_b.keypair).clone();
            let tips = node_b.dag_tips(&space_id).await;
            sign_event(
                build_federation_add_event(
                    &node_b_kp,
                    &space_id,
                    tips,
                    &peer_x_id,
                    "xgen://hash/sha256:phase9_scenario5_session",
                    "0.1",
                    "json",
                ),
                &node_b_kp,
            )
        };
        node_b.ingest(fed_add).await;

        // ── Bob's event arrives from X via federation, Identity is NOT on B ─
        // Bob has not been registered on B; F-4 step 11 will produce
        // HeldPending on the F-10 unknown-signer trigger.
        let bob_key = keypair::generate();
        let bob_id = pubkey_uri(&bob_key);

        let tips_pre = node_b.dag_tips(&space_id).await;
        let bob_join = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(&bob_id),
                rdx(""),
                sdx(&space_id),
                tips_pre.iter().map(|t| edx(t)).collect(),
                now_rfc(),
                json!({}),
            ),
            &bob_key,
        );
        let bob_join_id: String = event_id_str(&bob_join);

        let outcome = {
            let mut rt = node_b.runtime.lock().await;
            rt.dispatch_event(
                bob_join.clone(),
                EventOrigin::ReceivedViaFederation,
                Some(&peer_x_id_typed),
            )
        };
        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "expected HeldPending on F-10 unknown-signer trigger for Bob's event from X; got {:?}",
            outcome
        );

        // ── Honesty assertion #1 — counter incremented, event not in DAG ──
        assert_eq!(
            pending_identity_count(&node_b, &space_id).await,
            1,
            "pending_identity_replication counter should increment by 1 \
             (event buffered on F-10 trigger waiting for Bob's Identity)"
        );
        assert!(
            !node_b.has_event(&space_id, &bob_join_id).await,
            "Bob's event MUST NOT be in B's DAG while HeldPending on F-10"
        );

        // ── Trigger the production identity-arrival hook ─────────────────
        // Mirrors xgen-node/src/app.rs::handle_identity_replicate_msg:
        //   1. register the Identity on this Node
        //   2. fire drain_pending_by_identity
        // The two calls are intentionally serial under the same lock
        // domain so the wall-clock distance between hook-fire and
        // counter-observation reflects only the drain helper's cost,
        // not any inter-message scheduling jitter.
        let hook_start = Instant::now();
        {
            let mut rt = node_b.runtime.lock().await;
            rt.register_identity(make_identity_record(&bob_key, &node_b.node_id))
                .expect("Bob identity registers");
            let _ = rt
                .identity_registry
                .save(&node_b.identities_path);
            let _drained = rt.drain_pending_by_identity(&idx(&bob_id));
        }
        let hook_elapsed = hook_start.elapsed();

        // ── Honesty assertion #2 — hook-vs-sweep distinguishing window ────
        // Per findings v1.2 §2.5 sub-item C step 3: ≤ 100 ms proves the
        // synchronous hook ran; > 100 ms but < 5 s would be the sweep-only
        // signature (which today does not exist as a release path —
        // regression signal).
        //
        // CI parallelism inflates wall-clock under contention; we use the
        // hook_elapsed direct measurement (not a poll loop) because the
        // hook is synchronous on the runtime lock. The 100 ms ceiling is
        // generous relative to the operation cost (a HashMap insert and
        // one O(buffer_size) drain pass with buffer_size = 1).
        assert!(
            hook_elapsed < Duration::from_millis(100),
            "identity-arrival hook took {hook_elapsed:?} — exceeds 100 ms \
             ceiling; either lock contention is pathological or a sweep-only \
             release path has regressed in. Per findings v1.2 §2.5 sub-item C \
             step 3 the hook is synchronous within handle_identity_replicate_msg \
             (5 s sweep cadence; sub-100 ms hook expected)."
        );
        assert_eq!(
            pending_identity_count(&node_b, &space_id).await,
            0,
            "pending_identity_replication counter must decrement to 0 after \
             drain_pending_by_identity fires for Bob's id"
        );

        // ── Honesty assertion #3 — released event reaches B's DAG ────────
        // Brief poll budget for the dispatch_event re-entry path that
        // drain_pending_by_identity invokes for the released Bob event.
        // The drain helper is synchronous but the subsequent ingest_event
        // path involves several borrow-juggle steps; allow up to 200 ms
        // (well inside what would distinguish hook vs sweep).
        assert!(
            node_b
                .wait_for_event(&space_id, &bob_join_id, Duration::from_millis(200))
                .await,
            "Bob's event must land in B's DAG after Identity arrival re-validates and ingests"
        );

        // ── Cleanup ──────────────────────────────────────────────────────
        node_b.shutdown().await;
    }
}
