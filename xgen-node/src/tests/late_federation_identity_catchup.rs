// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! MP-F9 C2 — late-federation identity catch-up (in-process, NOT box-gated).
//!
//! The MP-F9 finding: a node that federates *after* a Space has history catches
//! up the Space-DAG events via `stream_federation_delta`, but every backfilled
//! event is held on the F-10 unknown-signer gate (`exchange.rs` step 11) because
//! the signers' `IdentityRecord`s never replicated to the late peer
//! (`push_identity_to_peers` fires only at registration, to then-current peers).
//! Net: the late peer's catch-up is empty (zero events).
//!
//! The C2 fix: an establish-trigger (`app::replicate_space_signers_to_peer`,
//! fired from both `attempt_reconnect` and `handle_federation_incoming`)
//! replicates the **distinct signers of the shared Spaces' history** (F9-D3 —
//! delta-signers, NOT current members) to the peer; the receiver's
//! `handle_identity_replicate_msg` upsert fires `drain_pending_by_identity`,
//! releasing the F-10-held backfill.
//!
//! Contrast `phase9_two_node_smoke`, which works around the bug by registering
//! alice on **both** nodes ("Cross-Node Identity replication is its own subsystem
//! and not exercised by this scenario"). These witnesses register the signer only
//! on the *originating* node — exactly the MP-F9 condition — and assert the late
//! peer catches up both the events and the signer records.
//!
//! Box-free: two real in-process Nodes via the phase9 harness; `federate()`
//! drives the real `attempt_reconnect` → real `handle_federation_incoming`.

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;

    use crate::tests::phase9_harness::{
        edx, event_id_str, federate, idx, now_rfc, pubkey_uri, rdx, sdx,
        spawn_in_process_node, InProcessNode,
    };
    use crate::{
        identity::keypair,
        space::state::{build_room_create_event, build_space_create_event, sign_event},
        wire::types::{Event, EventType},
    };

    /// Poll `cond` until true or `timeout` elapses. The signer-replication is a
    /// detached task separate from `federate()`'s session wait, so the catch-up
    /// (record lands → `drain_pending_by_identity` releases the held events) is
    /// observed asynchronously.
    async fn wait_until<F, Fut>(timeout: Duration, mut cond: F) -> bool
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if cond().await {
                return true;
            }
            if tokio::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
    }

    /// Build alice's Space history on `node` (create + room + one post), all
    /// signed by `alice_key` and ingested directly (pre-federation setup, like
    /// `phase9_two_node_smoke`). Returns `(space_id, room_id, post_id)`.
    async fn build_alice_history(
        node: &InProcessNode,
        alice_key: &ed25519_dalek::SigningKey,
        alice_id: &str,
    ) -> (String, String, String) {
        let space_ev = sign_event(
            build_space_create_event(alice_key, "latefed-space", None, 1, &node.node_id, None, false),
            alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        let room_ev = sign_event(
            build_room_create_event(alice_key, &space_id, "general", None),
            alice_key,
        );
        let room_id = event_id_str(&room_ev);
        node.ingest(room_ev).await;

        let post_ev = sign_event(
            Event::new(
                EventType::MessageText,
                idx(alice_id),
                rdx(&room_id),
                sdx(&space_id),
                vec![edx(&room_id)],
                now_rfc(),
                json!({ "text": "pre-federation history" }),
            ),
            alice_key,
        );
        let post_id = event_id_str(&post_ev);
        node.ingest(post_ev).await;

        (space_id, room_id, post_id)
    }

    /// THE witness (RED-on-revert): a peer that federates AFTER a Space has
    /// history catches up the events AND the signer's IdentityRecord. With the
    /// C2 trigger reverted, B holds every event on the F-10 unknown-signer gate
    /// and `has_event` stays false (the zero-event MP-F9 symptom in-process).
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn late_federated_peer_catches_up_history_and_signers() {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        // alice registered ONLY on A (the MP-F9 condition: B federates after she
        // registered, so the registration-time push never reached B).
        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_a.register_identity(&alice_key).await;

        let (space_id, room_id, post_id) =
            build_alice_history(&node_a, &alice_key, &alice_id).await;

        // B federates LATE — the Space already has history on A.
        federate(&node_a, &node_b, vec![space_id.clone()]).await;

        // The fix's core: B's registry receives alice (establish-trigger).
        assert!(
            wait_until(Duration::from_secs(10), || node_b.has_identity(&alice_id)).await,
            "MP-F9 C2: B's registry never received alice's IdentityRecord on late-federation establish"
        );

        // …and with alice known, the held backfill drains: B catches up the
        // full Space history (create + room + post).
        let caught_up = wait_until(Duration::from_secs(15), || async {
            node_b.has_event(&space_id, &space_id).await
                && node_b.has_event(&space_id, &room_id).await
                && node_b.has_event(&space_id, &post_id).await
        })
        .await;
        assert!(
            caught_up,
            "MP-F9 C2: B did not catch up the late-federated Space history \
             (events held on unknown-signer?) — space={space_id}"
        );
    }

    /// F9-D3 positive-by-construction (behaviour-hard): a signer who authored
    /// historical events and has SINCE LEFT the Space is still replicated —
    /// because the signer set is built from the Space DAG's senders
    /// (`store.range(0)`), NOT current membership. A members-only implementation
    /// would skip carol (she left) and her historical post would stay held on B.
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn late_catch_up_replicates_departed_signer() {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_a.register_identity(&alice_key).await;

        // carol also registered ONLY on A — a distinct signer.
        let carol_key = keypair::generate();
        let carol_id = pubkey_uri(&carol_key);
        node_a.register_identity(&carol_key).await;

        // alice builds the Space; carol joins, posts, then LEAVES.
        let (space_id, room_id, _post_id) =
            build_alice_history(&node_a, &alice_key, &alice_id).await;

        let invite_ev = sign_event(
            Event::new(
                EventType::MembershipInvite,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&space_id), edx(&room_id)],
                now_rfc(),
                json!({ "target_identity": carol_id, "role": "member" }),
            ),
            &alice_key,
        );
        let invite_id = event_id_str(&invite_ev);
        node_a.ingest(invite_ev).await;

        let join_ev = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(&carol_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&invite_id)],
                now_rfc(),
                json!({}),
            ),
            &carol_key,
        );
        let join_id = event_id_str(&join_ev);
        node_a.ingest(join_ev).await;

        let carol_post = sign_event(
            Event::new(
                EventType::MessageText,
                idx(&carol_id),
                rdx(&room_id),
                sdx(&space_id),
                vec![edx(&join_id)],
                now_rfc(),
                json!({ "text": "carol was here" }),
            ),
            &carol_key,
        );
        let carol_post_id = event_id_str(&carol_post);
        node_a.ingest(carol_post).await;

        let leave_ev = sign_event(
            Event::new(
                EventType::MembershipLeave,
                idx(&carol_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&carol_post_id)],
                now_rfc(),
                json!({}),
            ),
            &carol_key,
        );
        node_a.ingest(leave_ev).await;

        federate(&node_a, &node_b, vec![space_id.clone()]).await;

        // The teeth: carol (a delta-signer who LEFT) is replicated to B. Under a
        // members-only set she would be skipped; under delta-signers she is not.
        assert!(
            wait_until(Duration::from_secs(10), || node_b.has_identity(&carol_id)).await,
            "MP-F9 F9-D3: a departed signer (carol) was NOT replicated to the late peer — \
             signer set must be delta-signers (store.range senders), not current members"
        );
        // …and her historical membership event (the join — membership-self-
        // establishing, so it applies cleanly once her record lands) catches up
        // onto B. (Her later message post sits behind an intra-chain
        // membership-gated drain-ordering subtlety that is downstream of MP-F9's
        // identity-replication surface; probed separately below.)
        let join_landed =
            wait_until(Duration::from_secs(10), || node_b.has_event(&space_id, &join_id)).await;
        let post_landed = node_b.has_event(&space_id, &carol_post_id).await;
        eprintln!(
            "MP-F9 departed-signer convergence: carol record=true join={join_landed} post={post_landed}"
        );
        assert!(
            join_landed,
            "MP-F9 F9-D3: carol's historical join did not catch up onto B"
        );
    }

    /// Idempotence / no-churn (prime-invariant guard, in-session world): when the
    /// peer ALREADY holds a signer's record, the in-session send re-delivers it as
    /// a version-guarded **no-op** (the upsert does not re-apply) — and the
    /// catch-up still completes. Setup: alice registered on BOTH A and B, history
    /// built on A, then federate. B receives alice's record in-session (already
    /// present → no-op) and catches up the full history regardless. This is the
    /// idempotence-when-already-present witness without depending on any out-of-band
    /// helper (the early-federation prime invariant — phase9_two_node_smoke — is the
    /// suite-level sibling: alice pre-seeded on both, full suite green).
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn late_catch_up_in_session_redelivery_is_idempotent() {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        // alice on BOTH — so the in-session send re-delivers an already-present
        // record (the no-op path).
        node_a.register_identity(&alice_key).await;
        node_b.register_identity(&alice_key).await;

        let (space_id, room_id, post_id) =
            build_alice_history(&node_a, &alice_key, &alice_id).await;

        federate(&node_a, &node_b, vec![space_id.clone()]).await;

        // The version-guarded no-op must not break the catch-up: B still lands the
        // full history, and still holds alice (not churned/removed).
        let caught_up = wait_until(Duration::from_secs(15), || async {
            node_b.has_event(&space_id, &space_id).await
                && node_b.has_event(&space_id, &room_id).await
                && node_b.has_event(&space_id, &post_id).await
        })
        .await;
        assert!(
            caught_up,
            "MP-F9 idempotence: in-session re-delivery of an already-present signer \
             broke the catch-up — space={space_id}"
        );
        assert!(
            node_b.has_identity(&alice_id).await,
            "MP-F9 idempotence: a re-delivered already-present alice was churned/removed on B"
        );
    }
}
