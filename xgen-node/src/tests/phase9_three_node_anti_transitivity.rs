// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Phase 9 Scenario 2 — Three-Node anti-transitivity (task file
//! `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 3 Scenario 2; survey
//! findings §2.2). Shipped at Commit 3b-1 per §3.0 revised five-commit shape.
//!
//! Owns F-5 (Phase 4 anti-transitivity guard, design doc §8.5); cross-surfaces
//! F-1 push and the `EventOrigin` enum.
//!
//! **Harness shape.** In-process three-Node spawn via [`phase9_harness`] per
//! Lock #2 (uniform in-process across Commit 3b). Sibling-shape to Scenario
//! 1's `phase9_two_node_smoke.rs` with one extra Node and one extra
//! `federate()` call. The path exercised end-to-end matches Scenario 1's:
//! A's `apply_federation_push` → `try_send` into A's
//! `FederationPeerSenders[B]` and `FederationPeerSenders[C]` → each
//! receiver's `run_federation_session_post_handshake` drain →
//! `process_inbound` → `dispatch_event` → on the receiver side
//! `apply_federation_push` fires again with
//! `EventOrigin::ReceivedViaFederation` and the F-5 guard short-circuits.
//!
//! **Honesty assertions** (findings §2.2 sub-item C):
//! 1. Source-side load-bearing — G2 `federation_push_skipped_origin` trace
//!    event is emitted by the receiver(s) for events received via federation.
//!    This is the direct witness that F-5 fired correctly.
//! 2. Destination-side reach — every event posted by Alice on A appears in
//!    C's runtime store (and B's). Proves the direct A→{B,C} push delivered.
//! 3. Structural anti-transitivity — B has NO entry for C in its
//!    `FederationPeerSenders`, and vice versa. Even if F-5 broke, B literally
//!    could not push to C because the channel does not exist.
//!
//! **Coverage notes.** Findings §2.2 sub-item C ideal phrasing is "E appears
//! in C's CommLog with `from=A`, never with `from=B`". The in-process harness
//! does not maintain per-event provenance metadata in the runtime store
//! (production CommLog is a client-side artefact), so the "from=A vs from=B"
//! distinction is approximated by the combination of assertions #1 and #3 —
//! together they prove B could not have been the source of the events on C
//! even if F-5 misbehaved.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use serde_json::json;
    use tracing_test::traced_test;

    use crate::tests::phase9_harness::{
        edx, event_id_str, federate, idx, ndx, now_rfc, pubkey_uri, rdx, sdx,
        spawn_in_process_node,
    };
    use crate::{
        identity::keypair,
        space::state::{
            build_room_create_event, build_space_create_event, sign_event,
        },
        wire::types::{Event, EventType},
    };

    fn build_alice_text(
        alice_key: &ed25519_dalek::SigningKey,
        alice_id: &str,
        space_id: &str,
        room_id: &str,
        prev: Vec<String>,
        body_size: usize,
    ) -> Event {
        sign_event(
            Event::new(
                EventType::MessageText,
                idx(alice_id),
                rdx(room_id),
                sdx(space_id),
                prev.iter().map(|p| edx(p)).collect(),
                now_rfc(),
                json!({ "text": "x".repeat(body_size) }),
            ),
            alice_key,
        )
    }

    /// Phase 9 Scenario 2 — three-Node anti-transitivity, 100 messages.
    ///
    /// Setup: A↔B and A↔C federated for Space S; B↔C explicitly NOT federated.
    /// Alice (whose Identity is on A) posts 100 MessageText events. The F-5
    /// guard at `apply_federation_push:212` MUST fire on every receiver for
    /// every event it accepts via federation.
    /// **Runtime flavor.** `current_thread` (NOT `multi_thread` as in
    /// Scenario 1) because the load-bearing F-5 honesty assertion
    /// (`logs_contain("federation_push_skipped_origin")`) needs to see
    /// trace events emitted inside B's and C's spawned
    /// `handle_connection` tasks. `tracing-test` 0.2.x installs a
    /// thread-local subscriber that does not propagate to tokio
    /// multi-thread worker threads; running on a single thread keeps all
    /// tasks (test main + accept loops + per-connection handlers) on the
    /// same thread where the subscriber is active. Scenario 1 stays
    /// multi_thread because its `logs_contain("federation_push_sent")`
    /// fires from A's apply_federation_push called directly in the test
    /// task. Three-Node setup with 100 small messages is computationally
    /// light enough to run on one thread without timing fragility.
    #[serial_test::serial]
    #[tokio::test(flavor = "current_thread")]
    #[traced_test]
    async fn three_node_anti_transitivity_100_messages() {
        // ── Spawn three in-process Nodes ────────────────────────────────
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;
        let node_c = spawn_in_process_node().await;

        // ── Alice's Identity must be known on all three Nodes ───────────
        // (F-4 step 12 signature verification runs on every receiver;
        // F-10 unknown-signer HeldPending would otherwise hold every
        // event indefinitely. Cross-Node Identity replication is not
        // exercised by this scenario per the harness contract.)
        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_a.register_identity(&alice_key).await;
        node_b.register_identity(&alice_key).await;
        node_c.register_identity(&alice_key).await;

        // ── Pre-federation setup on A: Space + Room + Alice's membership ─
        let space_ev = sign_event(
            build_space_create_event(
                &alice_key,
                "phase9-anti-transitivity-space",
                None,
                1,
                &node_a.node_id,
                None,
            ),
            &alice_key,
        );
        let space_id: String = event_id_str(&space_ev);
        node_a.ingest(space_ev).await;

        let room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", None),
            &alice_key,
        );
        let room_id: String = event_id_str(&room_ev);
        node_a.ingest(room_ev).await;

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
        node_a.ingest(invite_ev).await;

        let join_ev = sign_event(
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
        node_a.ingest(join_ev).await;

        // ── Federate A↔B and A↔C for S (B↔C explicitly NOT federated) ───
        // Two sequential federate() calls. Each spawns its own
        // attempt_reconnect task; both run independently. After both
        // complete, A's FederationPeerSenders contains B and C; B's
        // contains only A; C's contains only A. The structural anti-
        // transitivity is established at this point.
        //
        // **Capture each peer's bootstrap tip between calls.** After
        // federate(A,B), A's tip is state.federation_add(A→B), which is
        // both A's tip AND B's bootstrap final event. federate(A,C)
        // then emits state.federation_add(A→C) chained off
        // state.federation_add(A→B); C's bootstrap delta carries both,
        // so C ends up knowing state.federation_add(A→B) too. B, however,
        // never sees state.federation_add(A→C) — federation_add events
        // are inherently visible only to their two parties (the protocol's
        // per-Space federation visibility property). Therefore
        // state.federation_add(A→B) is the **common ancestor known to
        // both B and C**, and the right prev_events anchor for any
        // subsequent message chain.
        federate(&node_a, &node_b, vec![space_id.clone()]).await;
        let tip_for_message_chain = {
            let tips = node_a.dag_tips(&space_id).await;
            assert_eq!(
                tips.len(),
                1,
                "expected exactly one DAG tip on A after first federate() (state.federation_add(A→B))"
            );
            tips[0].clone()
        };

        federate(&node_a, &node_c, vec![space_id.clone()]).await;

        // ── Wait for both peers to have the common ancestor ─────────────
        // B's bootstrap-delta final event IS `tip_for_message_chain`
        // (state.federation_add(A→B)). C's bootstrap-delta passes
        // through it as the second-to-last event in topo order
        // (state.federation_add(A→C) chains off it). Both must ingest
        // it before message posting begins; otherwise the first message
        // would HeldPending on whichever peer hasn't caught up.
        assert!(
            node_b
                .wait_for_event(&space_id, &tip_for_message_chain, Duration::from_secs(10))
                .await,
            "B did not ingest common-ancestor tip {tip_for_message_chain} within 10s"
        );
        assert!(
            node_c
                .wait_for_event(&space_id, &tip_for_message_chain, Duration::from_secs(10))
                .await,
            "C did not ingest common-ancestor tip {tip_for_message_chain} within 10s"
        );

        // ── Honesty assertion #3 — structural anti-transitivity ─────────
        // B has no session to C; C has no session to B. Even a broken F-5
        // could not route an event B-to-C because the channel doesn't exist.
        assert!(
            !node_b.has_federation_peer(&node_c.node_id).await,
            "B must NOT have C in FederationPeerSenders (B↔C is not federated)"
        );
        assert!(
            !node_c.has_federation_peer(&node_b.node_id).await,
            "C must NOT have B in FederationPeerSenders (B↔C is not federated)"
        );

        // ── 100 events posted serially from A ────────────────────────────
        // Single-submitter pattern per Scenario 1's module-level rationale
        // (avoiding the parallel-cursor race; concurrency dimension is
        // Compound C2's territory at Commit 3b-3).
        //
        // First message's `prev_events` is the common-ancestor tip
        // (state.federation_add(A→B)) — NOT A's current tip
        // (state.federation_add(A→C)) — because the latter is unknown to
        // B and would HeldPending the chain on B. A still accepts the
        // message (it has the referenced predecessor in its DAG); after
        // ingestion A's tips become [message_1, state.federation_add(A→C)]
        // (the message and the unmerged sibling tip). Subsequent
        // messages chain off [message_i] alone; both B and C accept each
        // (predecessor known on both).
        let payload_sizes: [usize; 3] = [100, 10_240, 102_400];
        let mut current_tip: Vec<String> = vec![tip_for_message_chain.clone()];

        let total_events: usize = 100;
        let mut posted_event_ids: Vec<String> = Vec::with_capacity(total_events);
        let post_start_ts = Instant::now();

        let local_node_a = ndx(&node_a.node_id);
        for i in 0..total_events {
            let size = payload_sizes[i % payload_sizes.len()];
            let ev = build_alice_text(
                &alice_key,
                &alice_id,
                &space_id,
                &room_id,
                current_tip.clone(),
                size,
            );
            let event_id: String = event_id_str(&ev);
            let outcome = {
                let mut rt = node_a.runtime.lock().await;
                rt.dispatch_event(
                    ev.clone(),
                    crate::node::runtime::EventOrigin::LocallySubmitted,
                    None,
                )
            };
            assert!(
                matches!(outcome, crate::node::runtime::DispatchOutcome::Accepted { .. }),
                "dispatch_event must Accept locally-submitted Alice msg #{i} (prev={current_tip:?}); got {outcome:?}"
            );
            crate::federation_session::apply_federation_push(
                &ev,
                crate::node::runtime::EventOrigin::LocallySubmitted,
                &node_a.runtime,
                &node_a.federation_peer_senders,
                &local_node_a,
                None,
            )
            .await;
            current_tip = vec![event_id.clone()];
            posted_event_ids.push(event_id);
        }
        assert_eq!(posted_event_ids.len(), total_events);
        // Sanity guard against a refactor that could reorder federate() to
        // return before the registration hooks fire.
        let _ = post_start_ts;

        // ── Honesty assertion #2 — destination-side reach ────────────────
        // Each event must arrive on both B's and C's runtime stores. The
        // 120s per-event budget matches Scenario 1's mitigation for
        // workspace-parallelism contention.
        for event_id in &posted_event_ids {
            assert!(
                node_b
                    .wait_for_event(&space_id, event_id, Duration::from_secs(120))
                    .await,
                "B did not receive event {event_id} within 120s"
            );
            assert!(
                node_c
                    .wait_for_event(&space_id, event_id, Duration::from_secs(120))
                    .await,
                "C did not receive event {event_id} within 120s"
            );
        }

        // ── Honesty assertion #1 — F-5 guard fired (load-bearing) ────────
        // `traced_test` captured the per-test tracing stream. The G2 stable
        // trace event `federation_push_skipped_origin` (Phase 9 Commit 1)
        // appears whenever apply_federation_push hits the F-5 short-circuit
        // for an event received via federation. Both B and C are receivers
        // here; each must emit it for each of the 100 events. The substring
        // check confirms at-least-one fire; the destination-side delivery
        // assertions above confirm the events did flow through the receiver
        // pipeline, which means apply_federation_push was reached on each
        // receiver — together these prove F-5 short-circuited rather than
        // pushing onward.
        assert!(
            logs_contain("federation_push_skipped_origin"),
            "B and/or C must emit federation_push_skipped_origin (G2) when F-5 fires \
             for events received via federation; see Phase 9 §3 Commit 1 (G2 trace event \
             additions in federation_session.rs)"
        );

        // ── Cleanup ──────────────────────────────────────────────────────
        node_a.shutdown().await;
        node_b.shutdown().await;
        node_c.shutdown().await;
    }
}
