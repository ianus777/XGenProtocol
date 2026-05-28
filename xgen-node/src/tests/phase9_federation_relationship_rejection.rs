// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Phase 9 Scenario 6 + Lock B1 + Phase 7.5 §5 extended skip set —
//! Federation-relationship deferral via HeldPending (task file
//! `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 4; survey findings
//! v1.2 §2.6 + §2.6.1). Shipped at Commit 3b-2 per §3.0a revised
//! five-commit shape.
//!
//! Owns F-3 (Phase 7 federation-relationship verification gate as amended
//! by Phase 7.5 §6 held-not-bypassed posture); cross-surfaces
//! `SpaceState.federation_nodes`, Phase 7 Lock B1 + Phase 7.5 §5 extended
//! skip set, the third HeldPending trigger condition
//! (federation-relationship), the `drain_pending_by_federation_relationship`
//! arrival hook, and the `pending_federation_relationship: usize` counter
//! on `NodeState`.
//!
//! **Harness shape.** In-process single-Node spawn via [`phase9_harness`]
//! per Lock #2 (uniform in-process across Commit 3b). X (the unfederated
//! peer attempting to push) is modelled as a fresh keypair and a fake
//! wire-authenticated peer ID — no second `InProcessNode` is spawned
//! because F-3's surface is entirely B-side (the buffer-add at runtime.rs
//! `dispatch_event` Step 2, the drain hook fired at Step 7 of every
//! successful federation_add ingestion, and the
//! `pending_federation_relationship` counter on the `PendingBuffer`'s
//! third secondary index).
//!
//! **Coverage gaps** (named per Lock #2 honesty discipline):
//! 1. In-process Node X session establishment via session-level federation
//!    doesn't exercise WS handshake under adversarial peer-side behaviour
//!    (malformed frames, slow-loris, etc.); F-3 reject path is fully
//!    exercised but the WS framing is in-process trusted.
//! 2. The 4007 timeout-path assertion (findings v1.2 §2.6 sub-item C step
//!    7 "Optional, harness-cost-permitting") is NOT covered here. The
//!    harness's `spawn_in_process_node` deliberately skips the pending-
//!    buffer timeout sweep task (production-only concern named in the
//!    harness docstring). Exercising 4007 would require either firing
//!    `drain_timed_out` directly with a forward-clock `Instant` (NodeRuntime
//!    sibling to `heldpending_identity_integration::identity_never_arrives_timeout_fires_4006_path`)
//!    or driving the actual sweep task — neither fits deployment-level
//!    framing without harness modification. Timeout-path coverage stays
//!    at the NodeRuntime level (Phase 7.5 unit tests in `runtime.rs`)
//!    plus the production sweep wiring in `xgen-node/src/app.rs`.
//! 3. The federation-add arrival hook is exercised by dispatching the
//!    federation_add via `submit_locally`'s production dispatch_event path
//!    (Step 7 fires `drain_pending_by_federation_relationship`). The
//!    in-process shape does NOT exercise the cross-Node case where the
//!    federation_add arrives via `process_inbound` from a peer; the
//!    Phase 7.5 NodeRuntime unit test
//!    `drain_pending_by_federation_relationship_drains_buffered_events`
//!    at `xgen-core/src/node/runtime.rs:1517+` covers the helper directly.
//!    The deployment shape adds the production-path exercise that
//!    dispatch_event Step 7 fires the hook in the first place.
//! 4. **G2 trace-event assertions are NOT carried at this layer.** Findings
//!    v1.2 §2.6 sub-item C step 3 names the `event = "f3_reject"` G2
//!    trace event with `disposition = "held_pending"` etc. as a load-bearing
//!    observability surface. The trace fires from `xgen-core/src/node/
//!    runtime.rs:537` (warn-level). `tracing-test 0.2`'s per-crate scope
//!    (default `xgen_node=trace`) does NOT capture cross-crate events from
//!    `xgen-core`, so `logs_contain("f3_reject")` would always return false
//!    from an xgen-node test even when the trace fires. Two consequences:
//!    (a) the protocol-level F-3 third-trigger path is verified via the
//!    stronger structural assertions below (`DispatchOutcome::HeldPending`
//!    outcome + `pending_federation_relationship_count == 1` — a regression
//!    that broke F-3 buffering would fail both); (b) trace-event regression
//!    coverage (silent removal of the f3_reject event or its disposition
//!    field) falls to the Phase 7.5 NodeRuntime unit tests at
//!    `xgen-core/src/node/runtime.rs:1486+` (which live in the same crate
//!    as the trace emit site). Promoting `f3_reject` capture to the
//!    deployment level would require either (1) a cross-crate test-only
//!    subscriber broadening or (2) an xgen-node wrapper trace mirror —
//!    both are structural surfaces, both require Joe-lock per
//!    `tasks/HANDOFF_PHASE_9_COMMIT_3B_2.md` §7 Trigger 1 + Trigger 3.
//!
//! **Three test functions** in this file:
//! - [`unfederated_peer_event_defers_via_held_pending_and_recovers_on_federation_add`]
//!   — main defer + recovery test (findings v1.2 §2.6 sub-item C
//!   steps 1-6, modulo step 7 timeout-path per coverage gap #2).
//! - [`extended_skip_set_does_not_trigger_f3_deferral`] — Phase 7.5 §5
//!   negative-assertion test for the three skip-set members
//!   (`StateFederationAdd` + `StateSpaceCreate` + `StateDmSpaceCreate`)
//!   per task file §3 Commit 4 "Lock B1 + Phase 7.5 §5 honesty test".
//! - [`room_create_still_triggers_f3_deferral_narrowness_regression`] —
//!   the narrowness regression check for the Phase 7.5 §5 skip set
//!   (`StateRoomCreate` is NOT in the skip set; F-3 still fires for it).

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;
    use tracing_test::traced_test;

    use crate::tests::phase9_harness::{
        edx, event_id_str, idx, ndx, now_rfc, pubkey_uri, rdx, sdx,
        spawn_in_process_node, InProcessNode,
    };
    use crate::{
        identity::keypair,
        node::runtime::{DispatchOutcome, EventOrigin},
        space::state::{
            build_dm_space_create_event, build_federation_add_event,
            build_room_create_event, build_space_create_event, sign_event,
        },
        wire::types::{Event, EventType},
    };

    /// Read B's `pending_federation_relationship` counter for `space_id`.
    /// Returns 0 when the buffer doesn't exist yet.
    async fn pending_federation_relationship_count(
        node: &InProcessNode,
        space_id: &str,
    ) -> usize {
        let rt = node.runtime.lock().await;
        rt.pending
            .get(space_id)
            .map(|b| b.pending_federation_relationship_count())
            .unwrap_or(0)
    }

    /// Build Alice + Space S setup on `node_b`. Returns
    /// `(alice_key, alice_id, space_id, room_id, current_tip_after_alice_join)`.
    /// Sibling-shape to `federation_relationship_integration::
    /// build_node_with_alice` but operating on the in-process harness's
    /// runtime via `ingest`.
    async fn setup_alice_and_space(
        node_b: &InProcessNode,
        space_name: &str,
    ) -> (ed25519_dalek::SigningKey, String, String, String, String) {
        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_b.register_identity(&alice_key).await;

        let space_ev = sign_event(
            build_space_create_event(&alice_key, space_name, None, 1, &node_b.node_id),
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
        let alice_join_id: String = event_id_str(&alice_join_ev);
        node_b.ingest(alice_join_ev).await;

        (alice_key, alice_id, space_id, room_id, alice_join_id)
    }

    /// Phase 9 Scenario 6 — main defer + recovery test.
    ///
    /// Per findings v1.2 §2.6 sub-item C the assertion shape layers four
    /// structural properties (the trace-event property is documented as a
    /// known gap in the module doc-comment §4):
    /// 1. Outcome is `DispatchOutcome::HeldPending` (not Rejected — the
    ///    Phase 7.5 §6 held-not-bypassed posture replaces Phase 7's
    ///    permanent reject).
    /// 2. `pending_federation_relationship_count()` increments by 1 for
    ///    the target Space (proves the buffer-add lands on the third
    ///    secondary index, not predecessor or Identity triggers — the
    ///    counter is keyed per-trigger so a regression that landed on the
    ///    wrong index would fail this assertion).
    /// 3. The event is NOT in B's DAG (`has_event` returns false — the
    ///    buffer is a holding cell, not a back-channel).
    /// 4. **Recovery** (load-bearing for held-not-bypassed posture): after
    ///    a valid state.federation_add for (X, S) is dispatched through
    ///    the production path, Step 7's `drain_pending_by_federation_relationship`
    ///    hook fires; the deferred event re-dispatches and ingests
    ///    cleanly; counter decrements to 0; event lands in B's DAG. If
    ///    properties 1-3 pass but the recovery property fails, the buffer
    ///    is a black hole, not a holding cell.
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[traced_test]
    async fn unfederated_peer_event_defers_via_held_pending_and_recovers_on_federation_add() {
        let node_b = spawn_in_process_node().await;
        let (alice_key, _alice_id, space_id, _room_id, _alice_join_id) =
            setup_alice_and_space(&node_b, "phase9-fed-rel-recovery-space").await;

        // ── X arrives unfederated (no entry in S's federation_nodes) ──────
        let peer_x_key = keypair::generate();
        let peer_x_id = pubkey_uri(&peer_x_key);
        let peer_x_id_typed = ndx(&peer_x_id);

        // ── X pushes Alice's room_create — F-3 defers via HeldPending ─────
        // room_create is the canonical "structurally valid, federation-
        // gated" event: signed by registered Alice, predecessor is the
        // Space root (which exists on B), missing only the federation
        // relationship for X.
        let tips_pre = node_b.dag_tips(&space_id).await;
        let pushed_room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "from-x", None),
            &alice_key,
        );
        // build_room_create_event hard-codes prev_events = [space_id]
        // (D-076 v1.1 causal-DAG-respecting order). For Phase 9 deferral
        // semantics that's correct: the event would validate cleanly in
        // F-4 if F-3 weren't an obstacle. tips_pre is captured for the
        // recovery-side federation_add (whose prev_events must reflect
        // current DAG tips on B, post-Alice-join).
        let pushed_room_id: String = event_id_str(&pushed_room_ev);

        let outcome = {
            let mut rt = node_b.runtime.lock().await;
            rt.dispatch_event(
                pushed_room_ev.clone(),
                EventOrigin::ReceivedViaFederation,
                Some(&peer_x_id_typed),
            )
        };

        // ── Honesty assertion #1 — HeldPending outcome ────────────────────
        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "expected HeldPending on F-3 third-trigger (held-not-bypassed posture per Phase 7.5 §6); got {:?}",
            outcome
        );

        // ── Honesty assertion #2 — counter incremented on third trigger ──
        assert_eq!(
            pending_federation_relationship_count(&node_b, &space_id).await,
            1,
            "pending_federation_relationship counter must increment by 1 \
             (not pending_identity / not pending_predecessor — must land on \
             the third secondary index per Phase 7.5 §6)"
        );

        // (G2 trace-event assertions intentionally omitted — see module
        // doc-comment §4 for the rationale. Structural assertions above and
        // below carry the load-bearing protocol-level F-3 third-trigger
        // verification; trace-event regression is covered upstream by Phase
        // 7.5 NodeRuntime tests at xgen-core/src/node/runtime.rs:1486+.)

        // ── Honesty assertion #3 — event NOT in B's DAG ───────────────────
        assert!(
            !node_b.has_event(&space_id, &pushed_room_id).await,
            "deferred event MUST NOT be in B's DAG (buffer is a holding cell, not a back-channel)"
        );

        // ── Recovery — X's bootstrap federation_add arrives via federation ─
        // The production drain-hook firing path is `dispatch_event`'s
        // Step 7, which runs on any successful federation_add ingestion
        // regardless of origin. The federation-channel form of bootstrap
        // is the cleanest deployment shape: X delivers its own
        // federation_add as part of the F-1a tip-exchange handshake, B
        // ingests it under Phase 7 Lock B1 (F-3 skipped for the
        // bootstrap-establishing event) + Phase 7 B3 (F-4 step 11
        // skipped because the sender is a federation peer not yet known
        // as a Space member), Step 7 fires the drain hook for the
        // (peer_x_id, space_id) pair, and the previously-buffered
        // room_create unblocks.
        //
        // For B3's signature verification (step 12) to pass we still
        // need X's Node-pubkey present in B's identity_registry — this
        // mirrors the production path where the F-10 identity-arrival
        // hook would have replicated X's Node-pubkey-as-Identity before
        // or concurrent with the federation_add ingestion. Doing the
        // registration ahead of time in the test isolates the recovery
        // assertion from F-10's separate timing surface (which Scenario
        // 5 covers in its own file).
        //
        // Vantage derivation (D-075 at runtime.rs:740-756):
        //   - event.sender = pubkey_uri(peer_x_key) = peer_x_id
        //   - event.content.node_id = node_b.node_id (X claims federation
        //     with B from X's vantage)
        //   - At B's vantage: content_node_id == self.node_id → drain
        //     pair peer = event.sender = peer_x_id, space = space_id
        //   - Matches the F-3 buffer-add key (peer_x_id, space_id) from
        //     the deferral above
        node_b.register_identity(&peer_x_key).await;

        let fed_add = sign_event(
            build_federation_add_event(
                &peer_x_key,
                &space_id,
                tips_pre,
                &node_b.node_id,
                "xgen://hash/sha256:phase9_scenario6_recovery_session",
                "0.1",
                "json",
            ),
            &peer_x_key,
        );
        let fed_add_outcome = {
            let mut rt = node_b.runtime.lock().await;
            rt.dispatch_event(
                fed_add,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_x_id_typed),
            )
        };
        // The federation_add MUST be Accepted — Step 7 fires only on
        // the Accepted branch (runtime.rs:794+). Any other outcome
        // means the recovery setup itself broke (signature, predecessor,
        // permission, etc.) and the drain hook never had a chance to
        // fire; assert explicitly so a future setup regression surfaces
        // here rather than masquerading as a drain-hook failure.
        assert!(
            matches!(fed_add_outcome, DispatchOutcome::Accepted { .. }),
            "recovery setup — bootstrap state.federation_add from X must be Accepted \
             so Step 7 drain hook can fire; got {:?}",
            fed_add_outcome
        );

        // ── Honesty assertion #4 — counter drained, event reaches DAG ────
        // Brief poll budget for the dispatch_event re-entry path the
        // drain hook invokes; the drain helper itself is synchronous on
        // the runtime lock, but Tokio scheduling between the
        // submit_locally call and our observation may add a few ms.
        assert!(
            node_b
                .wait_for_event(&space_id, &pushed_room_id, Duration::from_millis(200))
                .await,
            "deferred room_create event MUST land in B's DAG after federation_add \
             fires the drain hook (Step 7 of dispatch_event for StateFederationAdd) — \
             if this fails but assertions 1-4 pass, the buffer is a black hole, not \
             a holding cell (Phase 7.5 §6 held-not-bypassed posture is broken)"
        );
        assert_eq!(
            pending_federation_relationship_count(&node_b, &space_id).await,
            0,
            "pending_federation_relationship counter must decrement to 0 after drain"
        );

        node_b.shutdown().await;
    }

    /// Phase 7.5 §5 — extended skip set negative-assertion test for the
    /// three EventTypes whose F-3 check is skipped:
    /// `StateFederationAdd` (Phase 7 Lock B1 original),
    /// `StateSpaceCreate` (Phase 7.5 §5 new), and
    /// `StateDmSpaceCreate` (Phase 7.5 §5 new). The discriminator is
    /// "creates the Space it references"; `state.room_create` is NOT in
    /// the skip set (covered by the narrowness regression test below).
    ///
    /// Per task file §3 Commit 4 + findings v1.2 §2.6 sub-item C "Lock B1
    /// + Phase 7.5 §5 honesty check": outcome MUST NOT contain the
    /// `held_pending` + `federation_relationship_missing` conjunction
    /// (the conjunction distinguishes "skipped F-3" from "deferred via
    /// F-3"). Other outcomes (Accepted, HeldPending on F-10 Identity
    /// trigger, Rejected for downstream validation reasons) are all
    /// consistent with the F-3 skip being applied.
    ///
    /// Per the module doc-comment §4, `xgen-core`-emitted trace events
    /// are not captured by `tracing-test 0.2` from an xgen-node test.
    /// The negative assertion here is structural: each of the three
    /// dispatched events MUST NOT return a `Rejected` outcome whose
    /// reason string contains `federation_relationship_missing` (the
    /// pre-Phase-7.5 F-3 permanent-reject signal that no longer exists
    /// post-Phase-7.5) AND MUST NOT increment
    /// `pending_federation_relationship_count` on the target Space (the
    /// third-trigger HeldPending signal). Other outcomes (Accepted,
    /// HeldPending on F-10 Identity trigger, Rejected for other downstream
    /// validation reasons) are all consistent with F-3 being skipped.
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[traced_test]
    async fn extended_skip_set_does_not_trigger_f3_deferral() {
        let node_b = spawn_in_process_node().await;
        // Pre-existing Space so state.federation_add against it can be
        // dispatched (state.federation_add against an unknown Space hits
        // F-4 step 1 'space not found' — that's a different code path,
        // also a valid "not F-3 deferral" outcome, but the test reads
        // more cleanly when the Space exists).
        let (_alice_key, _alice_id, space_id, _room_id, _alice_join_id) =
            setup_alice_and_space(&node_b, "phase9-skip-set-space").await;

        let peer_x_key = keypair::generate();
        let peer_x_id = pubkey_uri(&peer_x_key);
        let peer_x_id_typed = ndx(&peer_x_id);

        // Counter snapshot before any dispatch — must stay at 0 across
        // all three skip-set sub-paths if Phase 7.5 §5 holds.
        let counter_before = pending_federation_relationship_count(&node_b, &space_id).await;

        // ── Sub-path 1 — state.federation_add (Lock B1 original) ──────────
        // X delivers its own federation_add via federation. Without B1,
        // F-3 would defer (X is not in federation_nodes). With B1, F-3 is
        // skipped; the event flows on to the validation core (where it
        // may HeldPending on F-10 Identity for X's Node-pubkey-as-Identity).
        let fed_add_from_x = sign_event(
            build_federation_add_event(
                &peer_x_key,
                &space_id,
                node_b.dag_tips(&space_id).await,
                &peer_x_id,
                "xgen://hash/sha256:phase9_skip_set_b1_session",
                "0.1",
                "json",
            ),
            &peer_x_key,
        );
        let outcome_fed_add = {
            let mut rt = node_b.runtime.lock().await;
            rt.dispatch_event(
                fed_add_from_x,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_x_id_typed),
            )
        };
        // Allowed outcomes: anything EXCEPT a Rejected whose reason
        // contains the pre-Phase-7.5 F-3 permanent-reject signal.
        if let DispatchOutcome::Rejected(reason) = &outcome_fed_add {
            assert!(
                !reason.contains("federation_relationship_missing"),
                "B1 skip failed for state.federation_add — F-3 should be skipped but got reject: {reason}"
            );
        }

        // ── Sub-path 2 — state.space_create (Phase 7.5 §5 new) ────────────
        // X creates a brand-new Space via federation. F-3 must be skipped
        // (chicken-and-egg: this event IS what creates the Space; there
        // is no federation_nodes for it yet).
        let space_create_from_x = sign_event(
            build_space_create_event(&peer_x_key, "x-new-space", None, 1, &peer_x_id),
            &peer_x_key,
        );
        let space_create_id: String = event_id_str(&space_create_from_x);
        let outcome_space_create = {
            let mut rt = node_b.runtime.lock().await;
            rt.dispatch_event(
                space_create_from_x,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_x_id_typed),
            )
        };
        if let DispatchOutcome::Rejected(reason) = &outcome_space_create {
            assert!(
                !reason.contains("federation_relationship_missing"),
                "Phase 7.5 §5 skip failed for state.space_create — got reject: {reason}"
            );
        }

        // ── Sub-path 3 — state.dm_space_create (Phase 7.5 §5 new) ─────────
        let dm_invitee_key = keypair::generate();
        let dm_invitee_id = pubkey_uri(&dm_invitee_key);
        let dm_create_from_x = sign_event(
            build_dm_space_create_event(&peer_x_key, &dm_invitee_id, &peer_x_id),
            &peer_x_key,
        );
        let dm_create_id: String = event_id_str(&dm_create_from_x);
        let outcome_dm = {
            let mut rt = node_b.runtime.lock().await;
            rt.dispatch_event(
                dm_create_from_x,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_x_id_typed),
            )
        };
        if let DispatchOutcome::Rejected(reason) = &outcome_dm {
            assert!(
                !reason.contains("federation_relationship_missing"),
                "Phase 7.5 §5 skip failed for state.dm_space_create — got reject: {reason}"
            );
        }

        // ── Negative structural assertion ─────────────────────────────────
        // Counter on the pre-existing Space S must not have changed
        // (none of the three skip-set members targets S; if F-3 were
        // erroneously firing for any of them it would still buffer on a
        // different Space's third-trigger index — checked next).
        let counter_after_space = pending_federation_relationship_count(&node_b, &space_id).await;
        assert_eq!(
            counter_after_space, counter_before,
            "Phase 7.5 §5 regression — Space S's third-trigger counter changed across \
             three skip-set dispatches (before={counter_before}, after={counter_after_space}); \
             at least one dispatch reached the F-3 third-trigger buffer-add site"
        );

        // The two new Spaces (space_create_id, dm_create_id) target their
        // own brand-new effective space_id (Phase 7.5 §5 derives space_id
        // from event_id for state.space_create / state.dm_space_create
        // when wire space_id is empty). For each, the F-3 path would
        // produce HeldPending and buffer on that Space's third-trigger
        // index. The skip set says it MUST NOT — so the counter for each
        // new Space must remain 0.
        assert_eq!(
            pending_federation_relationship_count(&node_b, &space_create_id).await,
            0,
            "Phase 7.5 §5 regression — state.space_create from federation peer was deferred \
             via F-3 third-trigger (the skip set must include StateSpaceCreate)"
        );
        assert_eq!(
            pending_federation_relationship_count(&node_b, &dm_create_id).await,
            0,
            "Phase 7.5 §5 regression — state.dm_space_create from federation peer was \
             deferred via F-3 third-trigger (the skip set must include StateDmSpaceCreate)"
        );

        node_b.shutdown().await;
    }

    /// Phase 7.5 §5 narrowness regression — `state.room_create` is NOT
    /// in the skip set. It is a DAG root but references an existing
    /// parent Space (the skip discriminator is "creates the Space it
    /// references", not "DAG root"). F-3 must still fire for room_create
    /// when the wire-pushing peer isn't in S's federation_nodes.
    ///
    /// The Phase 7.5 unit test
    /// `f3_does_not_skip_state_room_create` at
    /// `xgen-core/src/node/runtime.rs:1244+` is the upstream regression
    /// lock; this Phase 9 test elevates the same narrowness check to the
    /// deployment level over the in-process harness (per Lock #2).
    ///
    /// A regression that widened the skip set to include room_create
    /// (e.g. mistakenly using "DAG root" as the discriminator) would
    /// silently bypass F-3 for room_create. The attack surface would
    /// then be: a federated peer creates rooms in Spaces it shouldn't
    /// have access to. This test pins the narrowness.
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[traced_test]
    async fn room_create_still_triggers_f3_deferral_narrowness_regression() {
        let node_b = spawn_in_process_node().await;
        let (alice_key, _alice_id, space_id, _room_id, _alice_join_id) =
            setup_alice_and_space(&node_b, "phase9-narrowness-space").await;

        let peer_x_key = keypair::generate();
        let peer_x_id = pubkey_uri(&peer_x_key);
        let peer_x_id_typed = ndx(&peer_x_id);

        // room_create against the EXISTING Space S, signed by Alice, pushed
        // by unfederated X. The Space exists locally (skip-set discriminator
        // "creates the Space it references" does NOT apply here — room_create
        // references S, doesn't create it). F-3 must fire and defer.
        let room_from_x = sign_event(
            build_room_create_event(&alice_key, &space_id, "x-narrowness-room", None),
            &alice_key,
        );
        let room_id_from_x: String = event_id_str(&room_from_x);

        let outcome = {
            let mut rt = node_b.runtime.lock().await;
            rt.dispatch_event(
                room_from_x,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_x_id_typed),
            )
        };

        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "narrowness regression — room_create from unfederated peer MUST defer via HeldPending \
             on F-3 third trigger (Phase 7.5 §5 skip set must NOT include state.room_create); got {:?}",
            outcome
        );
        assert_eq!(
            pending_federation_relationship_count(&node_b, &space_id).await,
            1,
            "narrowness regression — room_create deferral must land on the \
             federation-relationship secondary index, not predecessor or Identity"
        );
        assert!(
            !node_b.has_event(&space_id, &room_id_from_x).await,
            "narrowness regression — deferred room_create MUST NOT be in B's DAG"
        );

        // (G2 trace-event positive assertions intentionally omitted —
        // see module doc-comment §4. The HeldPending outcome + the
        // third-trigger counter increment together carry the load-bearing
        // narrowness regression lock: a regression that erroneously
        // widened the skip set to include room_create would land Accepted
        // or HeldPending on a different trigger, failing the assertions
        // above.)

        node_b.shutdown().await;
    }
}
