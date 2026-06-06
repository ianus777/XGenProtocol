// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M8.6 C1 — F-10 unknown-signer HeldPending, drain consistency across an F-1a
//! re-stream (design §5 C1, runbook §4).
//!
//! **Single-Node realization (below-the-lock amendment of the runbook's "two
//! in-process Nodes" for C1 — Joe-confirmed 2026-06-06).** The locked assertion
//! is buffer↔drain consistency across the F-1a re-stream — no orphan, Bob a
//! member exactly once, the join in the store exactly once. That surface is
//! entirely B-side and process-resident, so it is fully captured single-Node by
//! modelling the F-1a re-stream as a RE-DELIVERY of the buffered join. A
//! two-Node version would mainly reintroduce the join-vs-identity-replication
//! ordering race the harness deliberately avoids, for no added coverage (the
//! buffer is process-resident, so "survives the drop" is trivially true either
//! way).
//!
//! **The spine is the re-delivery idempotency.** Plain
//! dispatch→HeldPending→identity→drain is already
//! `phase9_unknown_signer_first_contact`. What makes this C1 is re-dispatching
//! the SAME buffered join (modelling the F-1a re-stream after a reconnect) and
//! asserting it produced NO second buffer entry, then that identity arrival
//! drains it EXACTLY once and the join lands in the store exactly once — the
//! M3-under-reconnect surface (duplicate-ingest hazard).
//!
//! **Honest scope (D-065).** Single-Node proves the buffer/drain half
//! deterministically; it does NOT exercise transport drop/reconnect realism
//! (that F-1a actually re-streams over a dropped + recovered WS) — that half
//! lives in `phase9_drop_and_recover`, not here.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;

    use crate::tests::phase9_harness::{
        edx, event_id_str, idx, make_identity_record, ndx, now_rfc, pubkey_uri, rdx, sdx,
        spawn_in_process_node, InProcessNode,
    };
    use crate::{
        identity::keypair,
        node::runtime::{DispatchOutcome, EventOrigin},
        space::state::{
            build_federation_add_event, build_room_create_event, build_space_create_event,
            sign_event,
        },
        wire::types::{Event, EventType},
    };

    /// B's pending-identity count for `space_id` (0 if no buffer exists yet).
    async fn pending_identity_count(node: &InProcessNode, space_id: &str) -> usize {
        let rt = node.runtime.lock().await;
        rt.pending
            .get(&sdx(space_id))
            .map(|b| b.pending_identity_count())
            .unwrap_or(0)
    }

    /// Count events in B's store for `space_id` whose `event_id` equals
    /// `event_id`. The duplicate-ingest detector: must be exactly 1 after a
    /// re-stream + drain.
    async fn count_store_events_with_id(
        node: &InProcessNode,
        space_id: &str,
        event_id: &str,
    ) -> usize {
        let rt = node.runtime.lock().await;
        match rt.stores.get(&sdx(space_id)) {
            Some(store) => store
                .range(0)
                .unwrap_or_default()
                .iter()
                .filter(|e| e.event_id.as_ref().map(|i| i.as_str()) == Some(event_id))
                .count(),
            None => 0,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn c1_held_pending_drains_cleanly_across_f1a_restream() {
        let node_b = spawn_in_process_node().await;

        // ── Alice setup on B (Space owner, registered) ────────────────────────
        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_b.register_identity(&alice_key).await;

        let space_ev = sign_event(
            build_space_create_event(&alice_key, "m8_6-c1-space", None, 1, &node_b.node_id, None, false),
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

        let alice_invite = sign_event(
            Event::new(
                EventType::MembershipInvite,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&space_id), edx(&room_id)],
                now_rfc(),
                // M8.5-B INV-D6: a non-DM invite must carry an absolute
                // `valid_until` or the invited join is fail-closed-rejected 3044.
                json!({ "target_identity": alice_id, "role": "member", "valid_until": "2099-01-01T00:00:00.000Z" }),
            ),
            &alice_key,
        );
        let alice_invite_id: String = event_id_str(&alice_invite);
        node_b.ingest(alice_invite).await;

        let alice_join = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&alice_invite_id)],
                now_rfc(),
                json!({}),
            ),
            &alice_key,
        );
        node_b.ingest(alice_join).await;

        // Alice invites Bob (so Bob's join can grant membership once applied).
        let bob_key = keypair::generate();
        let bob_id = pubkey_uri(&bob_key);
        let bob_invite = sign_event(
            Event::new(
                EventType::MembershipInvite,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                node_b.dag_tips(&space_id).await.iter().map(|t| edx(t)).collect(),
                now_rfc(),
                json!({ "target_identity": bob_id, "role": "member", "valid_until": "2099-01-01T00:00:00.000Z" }),
            ),
            &alice_key,
        );
        let bob_invite_id: String = event_id_str(&bob_invite);
        node_b.ingest(bob_invite).await;

        // ── Federation peer X added so F-3 passes for X's pushes ──────────────
        let peer_x_key = keypair::generate();
        let peer_x_id = pubkey_uri(&peer_x_key);
        let peer_x_id_typed = ndx(&peer_x_id);
        let fed_add = {
            let kp = (*node_b.keypair).clone();
            let tips = node_b.dag_tips(&space_id).await;
            sign_event(
                build_federation_add_event(
                    &kp,
                    &space_id,
                    tips,
                    &peer_x_id,
                    "xgen://hash/sha256:m8_6_c1_session",
                    "0.1",
                    "json",
                ),
                &kp,
            )
        };
        node_b.ingest(fed_add).await;

        // ── Bob's join arrives via X; Bob's Identity is NOT on B → HeldPending ─
        let bob_join = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(&bob_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&bob_invite_id)],
                now_rfc(),
                json!({}),
            ),
            &bob_key,
        );
        let bob_join_id: String = event_id_str(&bob_join);

        let outcome1 = {
            let mut rt = node_b.runtime.lock().await;
            rt.dispatch_event(
                bob_join.clone(),
                EventOrigin::ReceivedViaFederation,
                Some(&peer_x_id_typed),
            )
        };
        assert!(
            matches!(outcome1, DispatchOutcome::HeldPending),
            "first delivery: Bob's join must HeldPending on the F-10 unknown-signer trigger; got {outcome1:?}"
        );
        assert_eq!(
            pending_identity_count(&node_b, &space_id).await,
            1,
            "exactly one buffered entry after the first delivery"
        );

        // ── SPINE — F-1a re-stream re-delivers the SAME buffered join ─────────
        // A reconnect re-streams the delta; the same join (same event_id)
        // arrives again while still HeldPending. The buffer must stay at ONE
        // entry (idempotent under re-delivery) — no orphan / second entry. This
        // is the M3-under-reconnect surface the test exists to defend.
        let outcome2 = {
            let mut rt = node_b.runtime.lock().await;
            rt.dispatch_event(
                bob_join.clone(),
                EventOrigin::ReceivedViaFederation,
                Some(&peer_x_id_typed),
            )
        };
        assert!(
            matches!(outcome2, DispatchOutcome::HeldPending),
            "re-stream: the re-delivered join must HeldPending again; got {outcome2:?}"
        );
        assert_eq!(
            pending_identity_count(&node_b, &space_id).await,
            1,
            "re-delivery must NOT create a second buffer entry (idempotency under the F-1a re-stream)"
        );
        assert!(
            !node_b.has_event(&space_id, &bob_join_id).await,
            "Bob's join must NOT be in the DAG while still HeldPending"
        );

        // ── Identity arrives → drain fires (the production hook order) ────────
        {
            let mut rt = node_b.runtime.lock().await;
            rt.register_identity(make_identity_record(&bob_key, &node_b.node_id))
                .expect("Bob identity registers");
            let _ = rt.identity_registry.save(&node_b.identities_path);
            let _drained =
                rt.drain_pending_by_identity(&idx(&bob_id));
        }

        // ── Assertions — no orphan / member once / store once ────────────────
        assert_eq!(
            pending_identity_count(&node_b, &space_id).await,
            0,
            "buffer must be empty after the drain (no orphaned HeldPending entry)"
        );
        assert!(
            node_b
                .wait_for_event(&space_id, &bob_join_id, Duration::from_millis(200))
                .await,
            "Bob's join must land in the DAG after the drain re-validates + ingests it"
        );
        assert_eq!(
            count_store_events_with_id(&node_b, &space_id, &bob_join_id).await,
            1,
            "Bob's join must be in the store EXACTLY once — the re-stream must not have produced a duplicate"
        );
        let state = node_b
            .space_state(&space_id)
            .await
            .expect("Space state must be present");
        assert!(
            state.members.contains_key(&idx(&bob_id)),
            "Bob must be a member after the drain applies his (invited) join"
        );

        node_b.shutdown().await;
    }
}
