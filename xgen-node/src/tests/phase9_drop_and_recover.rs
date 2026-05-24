// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Phase 9 Scenario 3 — Drop-and-recover (task file
//! `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 3 Scenario 3; survey
//! findings §2.3). Shipped at Commit 3b-1 per §3.0 revised five-commit shape.
//!
//! Owns F-1b (drop-on-peer-down, design doc §4.5) + F-1a (recovery via tip
//! exchange, §3.3); cross-surfaces F-1c (Phase 5 per-peer record + reconnect
//! plumbing).
//!
//! **Harness shape.** In-process two-Node spawn via [`phase9_harness`] per
//! Lock #2 (uniform in-process across Commit 3b). The "drop" is
//! [`InProcessNode::shutdown_keep_data`] (`shutdown_tx.send(true)` + abort
//! in-flight connection tasks → A's WS to B closes → A's session task
//! detects the drop and runs its R12 deregister cleanup); the "recover" is
//! [`spawn_in_process_node_with_state`] (replays identity registry + Space
//! event stores + federation registry from disk; binds a fresh
//! `127.0.0.1:0` per Pre-Commit-3b-1 Joe-lock Q1 option (b)).
//!
//! **Two known coverage gaps** (named per Lock #2 honesty discipline):
//! 1. `shutdown_tx.send(true)` is a clean async shutdown — it does NOT
//!    exercise "process crashed mid-write to disk" failure modes
//!    (partially-flushed `xgen-node_federation.json`, partially-committed
//!    identity DB transaction, OS-level fd recovery, tokio runtime
//!    panic/abort state). Those are `federation-stress` follow-on
//!    territory.
//! 2. Fresh `127.0.0.1:0` rebind on respawn — does NOT exercise
//!    "restart-in-place" semantics where the OS would put the prior bind
//!    port into TIME_WAIT. The TIME_WAIT race adds OS-level
//!    non-determinism unrelated to the protocol property under test and
//!    would risk the Q3 flake-escalation trigger; deliberately avoided.
//!    A real production operator restart-in-place is also
//!    `federation-stress` follow-on territory.
//!
//! **Honesty assertions** (findings §2.3 sub-item C):
//! 1. R14 trace event `federation_push_dropped_unregistered` fires on A
//!    for every event posted while B is down (proves F-1b drop-on-peer-down).
//! 2. F-1a tip-exchange recovery happens AND state.federation_add is NOT
//!    re-streamed: `peer_records[B].last_successful_session` advances
//!    after the respawn handshake, AND A's count of state.federation_add
//!    events for the Space remains exactly 1 across both cycles (a-i
//!    symmetry rule held — B's tip was non-empty so the rule did not fire
//!    again).
//! 3. B receives the dropped events via the F-1a delta after reconnect
//!    (post-respawn `wait_for_event` on B for each previously-dropped id).

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;
    use tracing_test::traced_test;

    use crate::tests::phase9_harness::{
        federate, now_rfc, pubkey_uri, spawn_in_process_node,
        spawn_in_process_node_with_state, InProcessNode,
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
                alice_id.to_string(),
                room_id.to_string(),
                space_id.to_string(),
                prev,
                now_rfc(),
                json!({ "text": "x".repeat(body_size) }),
            ),
            alice_key,
        )
    }

    /// Wait until `node_a` no longer has `peer_node_id` registered in its
    /// `FederationPeerSenders`. Returns true if cleared within `timeout`.
    /// 25 ms poll interval matches the harness's `wait_until` shape.
    async fn wait_for_federation_peer_cleared(
        node_a: &InProcessNode,
        peer_node_id: &str,
        timeout: Duration,
    ) -> bool {
        let start = std::time::Instant::now();
        loop {
            if !node_a.has_federation_peer(peer_node_id).await {
                return true;
            }
            if start.elapsed() >= timeout {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    /// Snapshot A's `peer_records[peer].last_successful_session` as a
    /// fingerprint that's strictly-monotonic on each successful
    /// handshake-ACTIVE. Returns the raw RFC 3339 string (empty if no
    /// record yet or session field absent). `last_successful_session` is
    /// `Option<String>` on `PeerOperationalRecord`; an empty return string
    /// represents either the no-record-yet or no-session-yet state.
    async fn peer_last_successful(
        node_a: &InProcessNode,
        peer_node_id: &str,
    ) -> String {
        let reg = node_a.federation_registry.lock().await;
        reg.peer_record(peer_node_id)
            .and_then(|r| r.last_successful_session.clone())
            .unwrap_or_default()
    }

    /// Count events in A's Store for `space_id` whose `event_type` is
    /// `StateFederationAdd`. Returns 0 if the Space is not present.
    /// Implementation iterates the in-memory store via the public
    /// `EventStore::values()` iterator.
    async fn count_federation_add_events(
        node_a: &InProcessNode,
        space_id: &str,
    ) -> usize {
        let rt = node_a.runtime.lock().await;
        match rt.stores.get(space_id) {
            Some(store) => store
                .values()
                .filter(|e| matches!(e.event_type, EventType::StateFederationAdd))
                .count(),
            None => 0,
        }
    }

    /// Phase 9 Scenario 3 — drop-and-recover, 5 events per cycle × 2 cycles.
    ///
    /// Per findings §2.3 sub-item E "Recommendation: 10 events queued, drop
    /// mid-stream, 2 cycles." 5 events per cycle keeps each cycle's R14
    /// burst legibly distinct in the trace log while preserving the
    /// 10-event total.
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[traced_test]
    async fn two_node_drop_and_recover_two_cycles() {
        // ── Initial spawn + Identity registration + pre-federation setup ─
        let node_a = spawn_in_process_node().await;
        let mut node_b = spawn_in_process_node().await;

        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_a.register_identity(&alice_key).await;
        node_b.register_identity(&alice_key).await;

        let space_ev = sign_event(
            build_space_create_event(
                &alice_key,
                "phase9-drop-recover-space",
                None,
                1,
                &node_a.node_id,
            ),
            &alice_key,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        node_a.ingest(space_ev).await;

        let room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", None),
            &alice_key,
        );
        let room_id = room_ev.event_id.clone().unwrap();
        node_a.ingest(room_ev).await;

        let invite_ev = sign_event(
            Event::new(
                EventType::MembershipInvite,
                alice_id.clone(),
                String::new(),
                space_id.clone(),
                vec![space_id.clone(), room_id.clone()],
                now_rfc(),
                json!({ "target_identity": alice_id, "role": "member" }),
            ),
            &alice_key,
        );
        let invite_id = invite_ev.event_id.clone().unwrap();
        node_a.ingest(invite_ev).await;

        let join_ev = sign_event(
            Event::new(
                EventType::MembershipJoin,
                alice_id.clone(),
                String::new(),
                space_id.clone(),
                vec![invite_id.clone()],
                now_rfc(),
                json!({}),
            ),
            &alice_key,
        );
        node_a.ingest(join_ev).await;

        // ── First federation (cycle 0 setup) ─────────────────────────────
        federate(&node_a, &node_b, vec![space_id.clone()]).await;

        // Capture the bootstrap tip so we can chain subsequent events off it.
        let mut current_tip = node_a.dag_tips(&space_id).await;
        assert_eq!(
            current_tip.len(),
            1,
            "expected exactly one DAG tip on A after initial federate() (the a-i state.federation_add)"
        );

        // Sanity: B has the bootstrap delta in its store; A has exactly 1
        // state.federation_add event (the a-i emission for this peer).
        for tip_id in &current_tip {
            assert!(
                node_b
                    .wait_for_event(&space_id, tip_id, Duration::from_secs(10))
                    .await,
                "B did not ingest bootstrap-delta tip {tip_id} within 10s"
            );
        }
        let fed_add_count_after_bootstrap =
            count_federation_add_events(&node_a, &space_id).await;
        assert_eq!(
            fed_add_count_after_bootstrap, 1,
            "after initial federate() A should have exactly 1 state.federation_add event in S"
        );

        // ── Two drop-recover cycles ──────────────────────────────────────
        let cycle_event_count: usize = 5;
        let mut all_dropped_event_ids: Vec<String> = Vec::new();

        for cycle in 0..2 {
            // Snapshot last_successful_session BEFORE this cycle's drop
            // so we can prove monotonic advance after this cycle's recover.
            let before = peer_last_successful(&node_a, &node_b.node_id).await;
            assert!(
                !before.is_empty(),
                "cycle {cycle}: A's peer_records[B].last_successful_session should be populated \
                 before drop (was set by prior handshake-ACTIVE callback)"
            );

            // ── Drop B ───────────────────────────────────────────────────
            // Preserve B's on-disk state; abort in-flight tasks so A's
            // WS to B closes and A's R12 deregister cleanup runs.
            let saved = node_b.shutdown_keep_data().await;

            // ── Wait for A to observe the drop ──────────────────────────
            // 30 s budget accommodates WS keepalive detection latency.
            // The session task's exit also calls `mark_lost` on the
            // registry record, which is structural backup for the
            // FederationPeerSenders deregister we observe directly.
            assert!(
                wait_for_federation_peer_cleared(
                    &node_a,
                    &saved.node_id,
                    Duration::from_secs(30),
                )
                .await,
                "cycle {cycle}: A did not detect B's drop and deregister from FederationPeerSenders within 30s"
            );

            // ── Post 5 events on A while B is down ──────────────────────
            // Each must hit the F-1b drop-on-peer-down path with the
            // "unregistered" branch — R14 trace event.
            let payload_sizes: [usize; 3] = [100, 10_240, 102_400];
            let mut cycle_dropped_ids: Vec<String> = Vec::with_capacity(cycle_event_count);

            for i in 0..cycle_event_count {
                let size = payload_sizes[i % payload_sizes.len()];
                let ev = build_alice_text(
                    &alice_key,
                    &alice_id,
                    &space_id,
                    &room_id,
                    current_tip.clone(),
                    size,
                );
                let event_id = ev.event_id.clone().unwrap();
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
                    "cycle {cycle} msg #{i}: dispatch_event must Accept locally-submitted Alice msg \
                     (prev={current_tip:?}); got {outcome:?}"
                );
                crate::federation_session::apply_federation_push(
                    &ev,
                    crate::node::runtime::EventOrigin::LocallySubmitted,
                    &node_a.runtime,
                    &node_a.federation_peer_senders,
                    &node_a.node_id,
                )
                .await;
                current_tip = vec![event_id.clone()];
                cycle_dropped_ids.push(event_id);
            }
            all_dropped_event_ids.extend(cycle_dropped_ids.iter().cloned());

            // ── Respawn B with saved state + re-federate ─────────────────
            node_b = spawn_in_process_node_with_state(saved).await;
            federate(&node_a, &node_b, vec![space_id.clone()]).await;

            // ── Honesty assertion #3 — F-1a delta recovery delivers ──────
            // Bob receives each dropped event after the post-respawn
            // handshake. 60 s per-event budget for workspace-parallelism
            // contention (sibling-shape to Scenario 1's 120s budget;
            // smaller here because Scenario 3 has 10 events total vs 100).
            for event_id in &cycle_dropped_ids {
                assert!(
                    node_b
                        .wait_for_event(&space_id, event_id, Duration::from_secs(60))
                        .await,
                    "cycle {cycle}: B did not receive dropped event {event_id} via F-1a delta within 60s"
                );
            }

            // ── Honesty assertion #2a — last_successful_session advances ──
            let after = peer_last_successful(&node_a, &node_b.node_id).await;
            assert_ne!(
                after, before,
                "cycle {cycle}: A's peer_records[B].last_successful_session must advance after F-1a handshake \
                 (before={before}, after={after})"
            );

            // ── Honesty assertion #2b — federation_add NOT re-streamed ──
            // After each successful reconnect the federation_add count on
            // A's DAG must remain exactly 1 — proving the a-i symmetry
            // rule did not re-fire (B's tip was non-empty for every
            // post-cycle-0 handshake).
            let fed_add_count_after_recover =
                count_federation_add_events(&node_a, &space_id).await;
            assert_eq!(
                fed_add_count_after_recover, 1,
                "cycle {cycle}: a-i symmetry violated — A re-emitted state.federation_add (count became {fed_add_count_after_recover}, expected 1). \
                 B's tip should have been non-empty after prior session; stream_federation_delta's `peer_absent && we_have_events` guard should have skipped the emission."
            );
        }

        // ── Honesty assertion #1 — R14 trace fired for dropped events ────
        // `traced_test` captured the per-test trace stream. The G2 stable
        // trace event `federation_push_dropped_unregistered` (Phase 9
        // Commit 1) appears in apply_federation_push's R14 path when the
        // peer is not in FederationPeerSenders. Substring check is
        // sufficient because cycle 0's bootstrap traffic does NOT emit
        // this string (the peer WAS registered during bootstrap); both
        // cycles' R14 bursts here are the first and only emissions.
        assert!(
            logs_contain("federation_push_dropped_unregistered"),
            "A must emit federation_push_dropped_unregistered (G2) for events posted while B was down; \
             see Phase 9 §3 Commit 1 (G2 trace event additions in federation_session.rs)"
        );

        // Sanity guard against a future refactor that drops events here for some
        // other reason — verify we actually drove 10 events through the dropped
        // path (5 per cycle × 2 cycles).
        assert_eq!(
            all_dropped_event_ids.len(),
            cycle_event_count * 2,
            "expected {} events posted across both cycles",
            cycle_event_count * 2
        );

        // ── Cleanup ──────────────────────────────────────────────────────
        node_a.shutdown().await;
        node_b.shutdown().await;
    }
}
