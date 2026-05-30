// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Phase 9 Compound C2 — F-5 anti-transitivity under push queue depth
//! (task file `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 5; survey
//! findings §3.2). Shipped at Commit 3b-3 per §3.0a revised five-commit
//! shape.
//!
//! Owns F-5 (Phase 4 anti-transitivity guard, design doc §8.5) at deployment
//! load; cross-surfaces F-1 push and the `EventOrigin` enum. **Stronger sibling
//! of Scenario 2** at `phase9_three_node_anti_transitivity.rs` — same 3-Node
//! setup, same 100-event chain, same destination-side reach assertion, but
//! with two additional load-bearing assertions enabled by the Phase 9 Commit
//! 3b-3-pre harness extension:
//!
//! 1. **Positive G2 trace** via `harness_logs_contain("federation_push_skipped_origin")`
//!    (sibling to Scenario 2's `logs_contain` but using the harness facility
//!    instead of `#[traced_test]` — required because the push-attempt counter
//!    lives behind `install_harness_subscriber` which is mutually exclusive
//!    with `#[traced_test]` on the same thread).
//! 2. **Negative source-side assertion**: for every Alice event_id, B's
//!    `push_attempt_count(event_id, &node_a.node_id) == 0` AND C's
//!    `push_attempt_count(event_id, &node_a.node_id) == 0` — proving B
//!    never re-pushed any A-origin event back to A (the load-bearing F-5
//!    regression lock; Scenario 2's positive trace assertion proves
//!    `federation_push_skipped_origin` fires AT LEAST ONCE, while this
//!    counter assertion proves the absence of `federation_push_sent` on
//!    every single one of the 100 events).
//!
//! **Why this distinction matters.** Scenario 2's `logs_contain` substring
//! match is at-least-one positive existence; a regression that bypassed F-5
//! on, say, every other event would still fire `federation_push_skipped_origin`
//! for the events that DID short-circuit and so the positive trace
//! assertion would pass. Compound C2's counter assertion is per-event
//! negative-existence: it catches partial bypasses. The F-5 contract is
//! "every event with origin = ReceivedViaFederation short-circuits"; the
//! contrapositive at the counter is "no event with origin =
//! ReceivedViaFederation reaches the per-peer push iteration site that
//! emits federation_push_sent".
//!
//! **Why this test does NOT use `#[traced_test]`.** The push-attempt counter
//! is fed by the harness's `CounterLayer` which is installed via
//! `install_harness_subscriber()` returning a `DefaultGuard` scoped per-test
//! on the test thread. `#[traced_test]` installs its own per-thread default
//! via `set_default` which would override the harness subscriber on the
//! test thread. The two are mutually exclusive; the counter assertion
//! requires the harness subscriber to be the active default, so this test
//! uses `install_harness_subscriber` instead. The positive G2 trace
//! assertion uses `harness_logs_contain` (which queries the harness's
//! LogBufferLayer) rather than `tracing_test::logs_contain`. See
//! `phase9_harness.rs` module-level "Composition with `#[traced_test]`"
//! comment for the full contract.
//!
//! **Coverage note** (sibling-shape to Scenario 2's coverage notes). The
//! in-process three-Node harness does not maintain per-event provenance
//! metadata in the runtime store (production CommLog is a client-side
//! artefact). The negative source-side assertion above replaces Scenario
//! 2's coverage-gap framing — instead of approximating "from=A vs from=B"
//! via destination-side structural absence, Compound C2 reads source-side
//! push-attempt evidence directly from the per-Node counter.
//!
//! **Honesty assertions** (findings §3.2 + Pre-Commit-3b-3 lock):
//! 1. Source-side load-bearing — for every Alice event_id E, B's
//!    `push_attempt_count(E, &node_a.node_id) == 0` AND C's
//!    `push_attempt_count(E, &node_a.node_id) == 0`. Per-event
//!    negative-existence; catches partial-bypass regressions Sc2 cannot.
//! 2. Source-side positive — `harness_logs_contain("federation_push_skipped_origin")`
//!    fires (sibling to Scenario 2's positive assertion).
//! 3. Destination-side reach — every Alice event arrives in B's and C's
//!    runtime stores (sibling to Scenario 2; tighter 60s per-event budget
//!    signaling rapid-succession push queue did not block delivery).
//! 4. Structural anti-transitivity — B has NO entry for C in its
//!    `FederationPeerSenders`, and vice versa (sibling to Scenario 2).
//! 5. Source-side positive of A's pushes — A's
//!    `push_attempt_count(E, &node_b.node_id) >= 1` AND
//!    `push_attempt_count(E, &node_c.node_id) >= 1` for each Alice
//!    event_id E. Confirms the counter mechanism itself is working
//!    end-to-end (if A's counter were stuck at 0, the negative assertion
//!    on B and C would pass trivially for the wrong reason — this assertion
//!    distinguishes "counter wired correctly" from "counter trivially zero").

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;

    use crate::tests::phase9_harness::{
        edx, event_id_str, federate, harness_clear_logs, harness_logs_contain, idx,
        install_harness_subscriber, ndx, now_rfc, pubkey_uri, rdx, sdx, spawn_in_process_node,
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

    /// Phase 9 Compound C2 — three-Node anti-transitivity under push queue
    /// depth, 100 messages. Setup: A↔B and A↔C federated for Space S;
    /// B↔C explicitly NOT federated. Alice (Identity on A) posts 100
    /// MessageText events in rapid succession. The F-5 guard at
    /// `apply_federation_push:212` MUST fire on every receiver for every
    /// event accepted via federation; the per-event push-attempt counter
    /// on B and C MUST stay at 0 for every Alice event_id targeting A.
    ///
    /// **Runtime flavor** `current_thread` (sibling-shape to Scenario 2's
    /// rationale). The harness CounterLayer + LogBufferLayer are installed
    /// via `install_harness_subscriber` returning a per-thread DefaultGuard;
    /// in `current_thread` runtime all tokio tasks (test main + accept loops
    /// + per-connection handlers) run on the same thread where the
    /// subscriber is the active default. Multi-thread runtime would route
    /// some events to worker threads where the per-thread default is absent
    /// and they would fall back to the global default (tracing-test's, if
    /// any other test installed it) — the counter would silently miss those
    /// events. Single-thread keeps the trace flow whole.
    ///
    /// **Test scope.** This test does NOT use `#[traced_test]` because the
    /// push-attempt counter requires the harness subscriber to be the
    /// active per-thread default. See module doc-comment and
    /// `phase9_harness.rs` "Composition with `#[traced_test]`" comment for
    /// the contract.
    #[serial_test::serial]
    #[tokio::test(flavor = "current_thread")]
    async fn three_node_anti_transitivity_at_load_100_messages() {
        // ── Install harness subscriber on the current thread for the test's
        //    lifetime; clear log buffer so prior test pollution doesn't leak
        //    into harness_logs_contain. ────────────────────────────────────
        let _sub = install_harness_subscriber();
        harness_clear_logs();

        // ── Spawn three in-process Nodes ────────────────────────────────
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;
        let node_c = spawn_in_process_node().await;

        // ── Alice's Identity must be known on all three Nodes ───────────
        // (F-4 step 12 signature verification runs on every receiver;
        // F-10 unknown-signer HeldPending would otherwise hold every
        // event indefinitely. Same setup as Scenario 2.)
        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_a.register_identity(&alice_key).await;
        node_b.register_identity(&alice_key).await;
        node_c.register_identity(&alice_key).await;

        // ── Pre-federation setup on A: Space + Room + Alice's membership ─
        let space_ev = sign_event(
            build_space_create_event(
                &alice_key,
                "phase9-compound-c2-space",
                None,
                1,
                &node_a.node_id,
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

        // ── Federate A↔B and A↔C (B↔C explicitly NOT federated) ─────────
        // Same pattern as Scenario 2: two sequential federate() calls.
        // After both complete, A's FederationPeerSenders contains B and C;
        // B's contains only A; C's contains only A.
        //
        // Common-ancestor tip rationale (verbatim from Scenario 2):
        // After federate(A,B), A's tip is state.federation_add(A→B), which
        // is both A's tip AND B's bootstrap final event. federate(A,C)
        // then emits state.federation_add(A→C) chained off
        // state.federation_add(A→B); C's bootstrap delta carries both,
        // so C ends up knowing state.federation_add(A→B) too. B, however,
        // never sees state.federation_add(A→C) — federation_add events
        // are inherently visible only to their two parties. Therefore
        // state.federation_add(A→B) is the common ancestor known to both
        // B and C, and the right prev_events anchor for the message chain.
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

        // ── Honesty assertion #4 — structural anti-transitivity ─────────
        // (Same as Scenario 2; cheap structural baseline before the
        // load-bearing assertions below.)
        assert!(
            !node_b.has_federation_peer(&node_c.node_id).await,
            "B must NOT have C in FederationPeerSenders (B↔C is not federated)"
        );
        assert!(
            !node_c.has_federation_peer(&node_b.node_id).await,
            "C must NOT have B in FederationPeerSenders (B↔C is not federated)"
        );

        // ── 100 events posted in rapid succession from A ─────────────────
        // Sibling-shape to Scenario 2's loop; no artificial throttling
        // between posts. The "queue depth" framing per findings §3.2 is
        // satisfied by the back-to-back submission pattern — receiver-side
        // processing of 100 events arriving without inter-event gaps
        // exercises the FederationPeerSenders mpsc queue under depth
        // pressure on both B and C.
        //
        // First message's prev_events is the common-ancestor tip
        // (state.federation_add(A→B)) per Scenario 2's reasoning above.
        let payload_sizes: [usize; 3] = [100, 10_240, 102_400];
        let mut current_tip: Vec<String> = vec![tip_for_message_chain.clone()];

        let total_events: usize = 100;
        let mut posted_event_ids: Vec<String> = Vec::with_capacity(total_events);

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

        // ── Honesty assertion #3 — destination-side reach ────────────────
        // Tighter per-event budget (60s) than Scenario 2's 120s. The
        // shorter budget signals that the rapid-succession push queue did
        // not block delivery; if 60s isn't enough under workspace-parallelism
        // contention, the regression signal is "queue depth is a real
        // observable problem at this load."
        for event_id in &posted_event_ids {
            assert!(
                node_b
                    .wait_for_event(&space_id, event_id, Duration::from_secs(60))
                    .await,
                "B did not receive event {event_id} within 60s (queue-depth stress: \
                 rapid-succession push of 100 events appears to have blocked delivery — \
                 investigate FederationPeerSenders mpsc backpressure or downstream contention)"
            );
            assert!(
                node_c
                    .wait_for_event(&space_id, event_id, Duration::from_secs(60))
                    .await,
                "C did not receive event {event_id} within 60s (queue-depth stress)"
            );
        }

        // ── Honesty assertion #2 — F-5 guard fired (positive trace) ──────
        // Sibling to Scenario 2's positive assertion but via the harness
        // facility (harness_logs_contain instead of tracing_test::logs_contain).
        // The G2 stable trace event federation_push_skipped_origin fires
        // from apply_federation_push when origin = ReceivedViaFederation
        // (xgen-node/src/federation_session.rs:216, post-3b-3-pre with
        // local_node_id field). Both B and C are receivers; each emits this
        // for every event they accept via federation. Substring match
        // confirms at-least-one fire.
        assert!(
            harness_logs_contain("federation_push_skipped_origin"),
            "B and/or C must emit federation_push_skipped_origin (G2) when F-5 fires \
             for events received via federation; if this fails but assertion #1 below \
             passes, the trace event regressed but F-5 is still working — operator \
             observability degraded"
        );

        // ── Honesty assertion #5 — A's push counter wired correctly ──────
        // Source-side POSITIVE existence check. For every Alice event_id, A
        // pushed it to B (peer_node_id = B.node_id) at least once AND to C
        // at least once. If A's counter stuck at 0 for all events, the
        // negative assertions on B and C (#1) below would pass trivially
        // for the wrong reason — this assertion distinguishes "counter
        // wired correctly" from "counter trivially zero."
        for event_id in &posted_event_ids {
            assert!(
                node_a.push_attempt_count(event_id, &node_b.node_id) >= 1,
                "A's push_attempt_count((event_id={}, B.node_id={})) must be >= 1 — \
                 if 0, either the trace event was not emitted by A's apply_federation_push \
                 OR the CounterLayer wiring is broken; assertion #1 below would be \
                 trivially true for the wrong reason",
                event_id, node_b.node_id
            );
            assert!(
                node_a.push_attempt_count(event_id, &node_c.node_id) >= 1,
                "A's push_attempt_count((event_id={}, C.node_id={})) must be >= 1",
                event_id, node_c.node_id
            );
        }

        // ── Honesty assertion #1 — F-5 negative source-side (load-bearing) ─
        // For every Alice event_id, B's push_attempt_count for (event_id,
        // A.node_id) MUST be exactly 0. Same for C. This is the per-event
        // negative-existence assertion that Scenario 2 cannot make:
        // Scenario 2's positive substring match would pass even if only
        // SOME events were correctly short-circuited; this counter
        // assertion catches partial-bypass regressions where F-5 leaks
        // for, say, every other event.
        //
        // The F-5 contract is "every event with origin = ReceivedViaFederation
        // short-circuits"; the contrapositive at the per-Node counter
        // observation site is "no event with origin = ReceivedViaFederation
        // reaches the per-peer push iteration loop that emits
        // federation_push_sent." Since B and C have no peer except A
        // (B↔C is not federated), any leak from F-5 would emit a
        // federation_push_sent trace with peer_node_id = A — which the
        // CounterLayer routes to (event_id, A.node_id) on the leaking
        // Node's counter.
        for event_id in &posted_event_ids {
            let b_leak = node_b.push_attempt_count(event_id, &node_a.node_id);
            assert_eq!(
                b_leak, 0,
                "F-5 anti-transitivity regression on B — B pushed Alice's event {event_id} \
                 back to A {b_leak} time(s). Every event with origin = ReceivedViaFederation \
                 must short-circuit at apply_federation_push:212; a non-zero counter here \
                 means F-5 leaked for this event. See `xgen-node/src/federation_session.rs` \
                 around line 216 for the F-5 guard."
            );
            let c_leak = node_c.push_attempt_count(event_id, &node_a.node_id);
            assert_eq!(
                c_leak, 0,
                "F-5 anti-transitivity regression on C — C pushed Alice's event {event_id} \
                 back to A {c_leak} time(s)"
            );
        }

        // ── Cleanup ──────────────────────────────────────────────────────
        node_a.shutdown().await;
        node_b.shutdown().await;
        node_c.shutdown().await;
    }
}
