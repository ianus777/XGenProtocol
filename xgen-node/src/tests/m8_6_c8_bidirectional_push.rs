// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M8.6 C8 — bidirectional simultaneous federation push under provoked
//! back-pressure (design §5 C8, runbook §4).
//!
//! **This is a regression lock against a future blocking-`send` change, NOT a
//! deadlock-freedom proof (D-065).** The M8 deadlock the catalogue named
//! (bidirectional simultaneous push deadlocks the F-2a session) is
//! *structurally absent today* because the per-peer outbound federation push
//! uses non-blocking `try_send` (on a full channel it DROPS; the peer backfills
//! via sync). The only route back to the deadlock is a regression to a blocking
//! `send().await`, which blocks only when the channel is full.
//!
//! So the sensitivity requirement is a SMALL channel: this test lowers the
//! per-peer federation channel capacity to **2** (the C8 seam — prod default
//! 1024) before federating, then drives **8 events per direction across 4
//! interleavings** (A-first · B-first · A-mid-B · B-mid-A) — enough that a
//! hypothetical blocking `send` would block under the mutual full-channel burst
//! and the bidirectional push would deadlock. With today's `try_send` it
//! completes.
//!
//! **Assertions (Joe-locked at checkpoint #2, tightened 2026-06-06):**
//! - **no-hang / bounded completion** — the whole bidirectional burst finishes
//!   inside a bounded `tokio::time::timeout`. A blocking-`send` regression would
//!   hang here (the deadlock detector — C8's reason to exist).
//! - **local liveness** — every event a Node posts is applied LOCALLY even
//!   though both peer channels are being driven full: `try_send` dropping on a
//!   full channel must not stall or lose the local apply path. Distinct from
//!   "kept its own events" — it asserts the federation push-channel back-pressure
//!   does not back-propagate into the local apply (a blocking send would).
//!
//! **Cross-node convergence is deliberately OUT of scope (below-the-lock
//! narrowing of the checkpoint-#2 "eventual convergence" wording, Joe-confirmed
//! 2026-06-06; D-065).** Under lossy concurrent bidirectional posting,
//! cross-convergence depends on vantage-aware `state.federation_add` / F-3
//! reconciliation — a federation-propagation property the phase9 suite (+
//! M8.5-A) owns, not the back-pressure property C8 locks. (Sibling-shape to the
//! C1 single-Node scope-note; both narrowings recorded in the close JOURNAL.)

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;

    use crate::tests::phase9_harness::{
        edx, event_id_str, federate, idx, ndx, now_rfc, pubkey_uri, rdx, sdx,
        spawn_in_process_node, InProcessNode,
    };
    use crate::{
        identity::keypair,
        node::runtime::EventOrigin,
        space::state::{build_room_create_event, build_space_create_event, sign_event},
        wire::types::{Event, EventType},
    };

    fn alice_msg(
        alice_key: &ed25519_dalek::SigningKey,
        alice_id: &str,
        space_id: &str,
        room_id: &str,
        prev: Vec<String>,
        tag: &str,
    ) -> Event {
        sign_event(
            Event::new(
                EventType::MessageText,
                idx(alice_id),
                rdx(room_id),
                sdx(space_id),
                prev.iter().map(|p| edx(p)).collect(),
                now_rfc(),
                json!({ "text": tag }),
            ),
            alice_key,
        )
    }

    /// Post one Alice message on `node` (off its current tip) and push it to the
    /// federated peer via the production `apply_federation_push` (non-blocking
    /// `try_send` — drops if the cap-2 channel is full). Returns the event_id.
    async fn post_and_push(
        node: &InProcessNode,
        alice_key: &ed25519_dalek::SigningKey,
        alice_id: &str,
        space_id: &str,
        room_id: &str,
        tag: &str,
    ) -> String {
        let prev = node.dag_tips(space_id).await;
        let ev = alice_msg(alice_key, alice_id, space_id, room_id, prev, tag);
        let event_id = event_id_str(&ev);
        {
            let mut rt = node.runtime.lock().await;
            rt.dispatch_event(ev.clone(), EventOrigin::LocallySubmitted, None);
        }
        let local = ndx(&node.node_id);
        crate::federation_session::apply_federation_push(
            &ev,
            EventOrigin::LocallySubmitted,
            &node.runtime,
            &node.federation_peer_senders,
            &local,
            None,
        )
        .await;
        event_id
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn c8_bidirectional_push_completes_under_provoked_backpressure() {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        // ── C8 seam — small channel BEFORE federating, so a blocking-send
        // regression would block (deadlock) under the mutual burst. ──────────
        node_a.set_federation_channel_capacity(2).await;
        node_b.set_federation_channel_capacity(2).await;

        // ── Alice on both Nodes; the Space genesis (incl. her membership) is
        // seeded IDENTICALLY on both by ingesting the same signed events. This
        // makes both Nodes equal members of the Space before the bidirectional
        // burst, sidestepping federation membership-catch-up (orthogonal to the
        // back-pressure property under test). The bidirectional traffic that
        // actually exercises the cap-2 channel is the NEW message burst below. ─
        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_a.register_identity(&alice_key).await;
        node_b.register_identity(&alice_key).await;

        let space_ev = sign_event(
            build_space_create_event(&alice_key, "m8_6-c8-space", None, 1, &node_a.node_id, None, false),
            &alice_key,
        );
        let space_id: String = event_id_str(&space_ev);
        let room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", None),
            &alice_key,
        );
        let room_id: String = event_id_str(&room_ev);
        let invite = sign_event(
            Event::new(
                EventType::MembershipInvite,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&space_id), edx(&room_id)],
                now_rfc(),
                json!({ "target_identity": alice_id, "role": "member", "valid_until": "2099-01-01T00:00:00.000Z" }),
            ),
            &alice_key,
        );
        let invite_id: String = event_id_str(&invite);
        let join = sign_event(
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
        let join_id: String = event_id_str(&join);

        for node in [&node_a, &node_b] {
            node.ingest(space_ev.clone()).await;
            node.ingest(room_ev.clone()).await;
            node.ingest(invite.clone()).await;
            node.ingest(join.clone()).await;
            assert!(
                node.has_event(&space_id, &join_id).await,
                "both Nodes must seed Alice's membership genesis"
            );
        }

        // ── Federate A↔B (bilateral) — establishes the live session + the
        // per-peer cap-2 channels the bidirectional push drives full. ─────────
        federate(&node_a, &node_b, vec![space_id.clone()]).await;

        // ── Bidirectional burst: 8 events/direction × 4 interleavings under a
        // no-hang timeout. Each Node posts its own events (applied LOCALLY) AND
        // pushes them to the peer's cap-2 channel via try_send (drops on full).
        // With try_send the burst completes; a blocking-send regression would
        // deadlock under cap-2 mutual back-pressure. ──────────────────────────
        let burst = async {
            let mut a_ids = Vec::new();
            let mut b_ids = Vec::new();
            for interleaving in 0..4u8 {
                for i in 0..8u8 {
                    let a_first = match interleaving {
                        0 => true,        // A-first
                        1 => false,       // B-first
                        2 => i % 2 == 0,  // A-mid-B
                        _ => i % 2 == 1,  // B-mid-A
                    };
                    let tag_a = format!("A-{interleaving}-{i}");
                    let tag_b = format!("B-{interleaving}-{i}");
                    if a_first {
                        a_ids.push(post_and_push(&node_a, &alice_key, &alice_id, &space_id, &room_id, &tag_a).await);
                        b_ids.push(post_and_push(&node_b, &alice_key, &alice_id, &space_id, &room_id, &tag_b).await);
                    } else {
                        b_ids.push(post_and_push(&node_b, &alice_key, &alice_id, &space_id, &room_id, &tag_b).await);
                        a_ids.push(post_and_push(&node_a, &alice_key, &alice_id, &space_id, &room_id, &tag_a).await);
                    }
                }
            }
            (a_ids, b_ids)
        };

        // ── Assertion 1 — no-hang / bounded completion (the deadlock detector) ─
        let (a_ids, b_ids) = match tokio::time::timeout(Duration::from_secs(45), burst).await {
            Ok(ids) => ids,
            Err(_) => panic!(
                "bidirectional push HUNG — under cap-2 mutual back-pressure the burst did not \
                 complete within 45s. With non-blocking try_send it must; a blocking-send \
                 regression would deadlock here (the M8 F-2a deadlock vector)."
            ),
        };
        assert_eq!(a_ids.len(), 4 * 8, "8 events/direction × 4 interleavings (A)");
        assert_eq!(b_ids.len(), 4 * 8, "8 events/direction × 4 interleavings (B)");

        // ── Assertion 2 — local liveness ──────────────────────────────────────
        // Every event a Node posted is applied LOCALLY even though both peer
        // channels are being driven full: try_send dropping on a full channel
        // must not stall or lose the local apply path (dispatch_event). This is
        // distinct from "kept its own events" — it asserts the federation
        // push-channel back-pressure does not back-propagate into the local
        // apply (which a blocking send under a full channel WOULD do).
        for id in &a_ids {
            assert!(
                node_a.has_event(&space_id, id).await,
                "A's own posted event {id} must be applied locally despite the full push channel \
                 (try_send drop must not stall the local apply path)"
            );
        }
        for id in &b_ids {
            assert!(
                node_b.has_event(&space_id, id).await,
                "B's own posted event {id} must be applied locally despite the full push channel"
            );
        }

        // Cross-node convergence is deliberately NOT asserted here — a
        // below-the-lock narrowing of the checkpoint-#2 "eventual convergence"
        // wording (Joe-confirmed 2026-06-06). Under lossy concurrent
        // bidirectional posting, cross-convergence depends on vantage-aware
        // `state.federation_add` / F-3 reconciliation — a federation-propagation
        // property the phase9 suite (+ M8.5-A) owns, NOT the back-pressure
        // property C8 exists to lock. C8 = the no-hang deadlock-regression lock
        // + local liveness.

        node_a.shutdown().await;
        node_b.shutdown().await;
    }
}
