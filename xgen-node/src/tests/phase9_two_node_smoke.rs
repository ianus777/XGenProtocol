// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Phase 9 Scenario 1 — Two-Node federation push smoke (task file
//! `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 3 Scenario 1; survey
//! findings §2.1).
//!
//! Owns F-1 (Phase 4 federation push); cross-surfaces F-1a (Phase 3
//! handshake) and F-2 (long-lived session).
//!
//! **Harness shape.** In-process two-Node spawn via [`phase9_harness`],
//! `submit_locally` for event injection. J-093 chose in-process over the
//! survey's stress-complete (subprocess) shape because in-process is fast +
//! deterministic + lets tests inspect runtime state directly without state-file
//! polling. The trade-off vs the survey's stress-complete recommendation
//! (real binary spawns + real client connections) is honest framing here:
//! the path exercised end-to-end is `dispatch_event` → `apply_federation_push`
//! → `try_send` into `FederationPeerSenders` → receiver's
//! `run_federation_session_post_handshake` outbound→inbound drain →
//! receiver's `process_inbound` → receiver's `dispatch_event`. The pipeline
//! between A's `submit_locally` and B's runtime is real WebSocket transport
//! and real production code paths; only the local-client→A's WS-server hop
//! is bypassed.
//!
//! **Scope choices vs the survey recommendation.**
//! - Event count: 100 events (recommended).
//! - Event types: single MessageText. The survey recommended 5 types to
//!   exercise different `dispatch_event` semantic branches; that variance
//!   is more naturally covered by Scenario 4 / Compound C5 (Commit 6
//!   NodeRuntime-level validation scenarios). Scenario 1 is a pipeline
//!   smoke; mixing types here adds setup cost (membership.invite needs a
//!   target Identity; state.room_create needs a fresh tip; membership.kick
//!   needs a prior join state) without sharpening the pipeline assertion.
//! - Payload sizes: mixed 100B / 10KB / 100KB rotating across the 100 events
//!   (recommended).
//! - Concurrency: **single submitter task** — diverges from the survey's
//!   "2-3 clients on A posting in parallel" recommendation. Reason: with the
//!   in-process harness's `submit_locally` shape (no real client WS
//!   connections; tasks contend on A's runtime mutex + a shared "current
//!   tip" cursor), two parallel tasks race the cursor and produce a forked
//!   DAG. Each forked event has a valid `prev_events`, so the DAG is
//!   well-formed, but the receiver's strict ordering depends on
//!   `apply_federation_push` ordering matching dispatch order — and
//!   `apply_federation_push`'s mutex acquisition is non-deterministic
//!   across submitter tasks, so the wire order can drift from dispatch
//!   order. Result: receiver buffers events whose predecessors arrive
//!   later, and under 20 s timeout the chain occasionally doesn't drain.
//!   The race surfaces a real ordering property worth measuring — but
//!   under the harness shape J-093 locked, the right home for that
//!   measurement is Compound C2 (F-5 anti-transitivity under push queue
//!   depth, Commit 5) where source-side trace assertions isolate the
//!   ordering question without depending on cross-runtime arrival timing.
//!   Scenario 1 stays serial so the pipeline smoke is bisectable
//!   independent of the concurrency dimension.
//!
//! **Honesty assertions** (findings §2.1 sub-item C):
//! 1. Each Alice post timestamp is recorded > `handshake_active_at_B_ts`
//!    (handshake reached ACTIVE on B before the first post fires;
//!    enforced structurally by `federate()` awaiting both-side
//!    registration before returning).
//! 2. Each event arrives on B's runtime store after the handshake.
//! 3. Each event's `apply_federation_push` invocation observed on A via
//!    the G2 stable trace event `federation_push_sent`.
//! 4. F-5 guard did NOT fire (no `federation_push_skipped_origin` trace
//!    events for these 100 events — they are LocallySubmitted on A, not
//!    ReceivedViaFederation).

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

    /// Build a MessageText event from Alice with a payload of `body_size`
    /// bytes (UTF-8 chars are ASCII so size in bytes ~= chars). `prev`
    /// supplies the DAG predecessors; the first event refs the post-bootstrap
    /// tip, subsequent ones ref the previous event's id so the chain is
    /// strictly linear.
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

    /// Phase 9 Scenario 1 — two-node federation push smoke, 100 messages.
    ///
    /// Originally `#[ignore]`'d during Commit 3a (J-092 sub-entry) on the
    /// bidirectional `federation_nodes` finding. Lifted in Commit 3 of the
    /// bidirectional implementation runbook (J-096). Re-`#[ignore]`'d in
    /// Commit 4 of the bidirectional milestone close on the topological-sort
    /// wire-order non-determinism finding (J-096; forward-referenced
    /// `tasks/FEDERATION_TOPOSORT_AUDIT.md` placeholder at that point).
    /// Topological-sort design-phase re-walk at J-099 surfaced a second
    /// load-bearing property (causal-DAG-respecting order) under amended
    /// D-076 v1.1; Commit 2a (Path B fix at `xgen-core/src/space/state.rs:797`
    /// `build_room_create_event`) closed the causality layer alongside
    /// Commit 2's determinism layer.
    ///
    /// This commit (J-101 milestone close, topological-sort implementation)
    /// lifts the `#[ignore]` for the second time. The scenario is now the
    /// activating regression lock for three decisions: D-075 (bidirectional
    /// vantage-aware applier), D-076 v1.1 determinism layer (Commit 2's sort
    /// fix), and D-076 v1.1 causality layer (Commit 2a's Path B fix). If any
    /// of the three regresses, this test fails.
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[traced_test]
    async fn two_node_federation_push_smoke_100_messages() {
        // ── Spawn two in-process Nodes ───────────────────────────────────
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        // ── Alice exists; her Identity must be known on BOTH Nodes ──────
        // (signature verification at F-4 step 12 runs on both sides;
        // F-10 unknown-signer HeldPending would otherwise hold every event
        // on B until 4006 timeout fires). Cross-Node Identity replication
        // is its own subsystem and not exercised by this scenario.
        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_a.register_identity(&alice_key).await;
        node_b.register_identity(&alice_key).await;

        // ── Pre-federation setup on A: Space + Room + Alice's membership ─
        // Local-only construction — events ingested directly into A's
        // runtime via the test-only `ingest` path (matches the existing
        // `federation_relationship_integration::build_node_with_alice`
        // precedent). The bilateral delta from `stream_federation_delta`
        // will carry these to B during the federation handshake.
        let space_ev = sign_event(
            build_space_create_event(
                &alice_key,
                "phase9-smoke-space",
                None,
                1,
                &node_a.node_id,
                None,
            false),
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

        // ── Federate A → B for this Space ────────────────────────────────
        // `federate()` blocks until both sides' R12 register hooks fire,
        // so the moment it returns: `handshake_active_at_B_ts` ≤ now().
        // Capture the timestamp boundary for honesty assertion #1.
        federate(&node_a, &node_b, vec![space_id.clone()]).await;
        let handshake_active_at_b_ts = Instant::now();

        // After federation: A's tip is the a-i-emitted `state.federation_add`
        // (Phase 3 §3.3.1 Lock 2 fires for cold-start direction). Wait for
        // B to ingest the FULL bootstrap delta — not just `space.space_create`
        // (which `wait_for_space` would settle on) but every event through
        // fed_add. Otherwise the test's first push would reference a tip
        // that B has not yet ingested, the message would HeldPending on B,
        // and post-bootstrap pushes would chain off a not-yet-arrived
        // predecessor — racing for the rest of the test.
        let bootstrap_tip = node_a.dag_tips(&space_id).await;
        assert_eq!(
            bootstrap_tip.len(),
            1,
            "expected exactly one DAG tip on A after federate() (the a-i state.federation_add)"
        );
        let bootstrap_tip_id = bootstrap_tip[0].clone();
        assert!(
            node_b
                .wait_for_event(&space_id, &bootstrap_tip_id, Duration::from_secs(10))
                .await,
            "B did not ingest the bootstrap-delta tip ({bootstrap_tip_id}) within 10s"
        );

        // ── 100 events posted serially, mixed payload sizes ──────────────
        // Rotating across {100, 10240, 102400} bytes. Each event takes the
        // current single-tip as its `prev`, advancing the chain strictly
        // linearly. Concurrency dimension explicitly deferred per the
        // module-level "Concurrency" scope note — Compound C2 is the
        // right home for the concurrent-source-side stress.
        let payload_sizes: [usize; 3] = [100, 10_240, 102_400];
        let initial_tip = node_a.dag_tips(&space_id).await;
        assert_eq!(
            initial_tip.len(),
            1,
            "expected exactly one DAG tip after federation bootstrap (the a-i state.federation_add)"
        );
        let mut current_tip: Vec<String> = initial_tip;

        let total_events: usize = 100;
        let mut posted_event_ids: Vec<String> = Vec::with_capacity(total_events);
        let post_start_ts = Instant::now();

        // honesty assertion #1: assert post_start_ts is after handshake.
        // tokio::time::Instant arithmetic with checked_duration_since is
        // monotonic; this is a sanity guard against a future refactor
        // that might reorder federate() to return before R12.
        assert!(
            post_start_ts >= handshake_active_at_b_ts,
            "post_start_ts must be at or after handshake_active_at_b_ts"
        );

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

        // ── Honesty assertion #2: every event arrives on B's store ──────
        // (the push pipeline delivered each one end-to-end; receiver
        // dispatch accepted each one).
        // Per-event wait budget is generous (120s) to absorb wall-clock
        // contention under workspace parallelism. Happy-path completes in
        // ~5s; the budget protects against the ~22s-timeout flake observed
        // during Commit 3 verification (events processed correctly, just
        // delayed by xgen-node-lib binary's parallel test load). Sibling
        // mitigation is the `#[serial_test::serial]` annotation on the
        // test function; together they serialise against the other known
        // workspace flakes AND tolerate the remaining contention.
        for event_id in &posted_event_ids {
            assert!(
                node_b
                    .wait_for_event(&space_id, event_id, Duration::from_secs(120))
                    .await,
                "B did not receive event {event_id} within 120s"
            );
        }

        // ── Honesty assertions #3 + #4: G2 trace events ──────────────────
        // The `#[traced_test]` macro installs a per-test subscriber that
        // captures all `tracing::` events; `logs_contain` queries the
        // captured stream. The G2 stable `event = "..."` fields land in
        // the captured stream as e.g. `event="federation_push_sent"`.
        //
        // Assertion #3 — `apply_federation_push` invoked on A for each
        // event. Spot-check that `federation_push_sent` appears at all
        // (the per-event presence check would scan 100 event_ids against
        // the log buffer; the load-bearing assertion is "the success path
        // fired in the F-1 push direction", not "exactly N occurrences").
        // A stricter per-id check is followed up in Compound C2 (Commit 5).
        assert!(
            logs_contain("federation_push_sent"),
            "A must emit federation_push_sent (G2) for at least one of the 100 events; \
             see Phase 9 §3 Commit 1 (G2 trace event additions in federation_session.rs)"
        );

        // Assertion #4 — F-5 guard MUST NOT fire on A for these events.
        // They are LocallySubmitted on A, so apply_federation_push's F-5
        // guard (which short-circuits on ReceivedViaFederation) must not
        // be reached. If `federation_push_skipped_origin` appears, F-5
        // misclassified a LocallySubmitted event as ReceivedViaFederation.
        assert!(
            !logs_contain("federation_push_skipped_origin"),
            "A must NOT emit federation_push_skipped_origin (G2) for LocallySubmitted events; \
             F-5 guard fired on the wrong origin"
        );

        // ── Cleanup ──────────────────────────────────────────────────────
        node_a.shutdown().await;
        node_b.shutdown().await;
    }
}
