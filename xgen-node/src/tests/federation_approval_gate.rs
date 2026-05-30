// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! federation-admin-control 2a — Commit 3 pause-point integration test
//! (FAC-D1a). Drives a real inbound federation handshake against a receiver
//! Node with `require_approval = true` and asserts the gate's durable effect:
//! the request lands in the receiver's pending-approval queue and NO
//! relationship is established (the peer is answered `Reject 2003` and gives
//! up the attempt).
//!
//! **Coverage split.** The load-bearing gate *decision* (the truth table
//! including the prime default-off case and the option-1 Rejected re-enqueue)
//! is unit-tested in `xgen_core::federation::pending_queue`
//! (`should_queue_for_approval` + `gate_off_never_queues`). The default-off
//! *full-flow* regression — a receiving handshake with `require_approval =
//! false` still auto-establishes byte-for-byte — is covered by the entire
//! existing federation integration suite (every `federate()`-based test runs
//! with the gate off and would break if the default path regressed). This
//! module covers the `require_approval = true` full-flow effect that nothing
//! else exercises: the gate, wired into `handle_federation_incoming`, actually
//! enqueues + refuses.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::tests::phase9_harness::{
        attempt_federation_no_wait, ndx, spawn_in_process_node,
        spawn_in_process_node_with_approval,
    };

    /// require_approval = true → an inbound handshake from a not-yet-known
    /// peer is queued and refused, not auto-established (FAC-D1a).
    #[tokio::test]
    async fn require_approval_gates_inbound_handshake_into_queue() {
        let receiver = spawn_in_process_node_with_approval().await;
        let initiator = spawn_in_process_node().await;

        let space = "xgen://hash/sha256:approval-gate-space".to_string();
        attempt_federation_no_wait(&initiator, &receiver, vec![space.clone()]).await;

        // Poll the receiver's pending queue for the initiator's request.
        let initiator_key = ndx(&initiator.node_id);
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut enqueued = false;
        while Instant::now() < deadline {
            {
                let q = receiver.federation_queue.lock().await;
                if q.get(&initiator_key).is_some() {
                    enqueued = true;
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(
            enqueued,
            "receiver with require_approval=true did not enqueue the inbound request within 10s"
        );

        // The queued request carries the handshake-derived facts.
        {
            let q = receiver.federation_queue.lock().await;
            let req = q.get(&initiator_key).expect("request present");
            assert_eq!(req.peer_node_id, initiator_key);
            assert_eq!(req.negotiated_serialisation, "json");
            assert!(
                req.shared_spaces.iter().any(|s| s.as_str() == space),
                "queued request should carry the peer's shared space"
            );
        }

        // The gate must NOT have established a relationship — the queue is the
        // only durable record until the operator accepts/rejects (FAC-D1a).
        {
            let reg = receiver.federation_registry.lock().await;
            assert!(
                reg.get(&initiator_key).is_none(),
                "gate must not create a federation relationship before approval"
            );
        }

        receiver.shutdown().await;
        initiator.shutdown().await;
    }

    /// Checkpoint #3: a peer carrying a `Rejected` tombstone is refused
    /// (`Reject 2003`) WITHOUT being re-enqueued — the tombstone suppresses
    /// re-queuing so a rejected peer can't re-fill the operator's queue.
    #[tokio::test]
    async fn rejected_tombstone_suppresses_re_enqueue() {
        use crate::federation::registry::{FederationRelationship, FederationState};

        let receiver = spawn_in_process_node_with_approval().await;
        let initiator = spawn_in_process_node().await;
        let initiator_key = ndx(&initiator.node_id);

        // Pre-seed the receiver's registry with a Rejected tombstone for the
        // initiator (as `federation reject` would have written).
        {
            let mut reg = receiver.federation_registry.lock().await;
            reg.upsert(FederationRelationship {
                peer_node_id: initiator_key.clone(),
                shared_spaces: vec![],
                negotiated_version: "0.1".to_string(),
                negotiated_serialisation: "json".to_string(),
                session_id: "xgen://rejected/tombstone".to_string(),
                last_connected: "2026-05-30T00:00:00.000Z".to_string(),
                peer_url: None,
                state: FederationState::Rejected,
            });
        }

        let space = "xgen://hash/sha256:rejected-gate-space".to_string();
        attempt_federation_no_wait(&initiator, &receiver, vec![space]).await;

        // Give the handshake time to reach the gate, then assert the queue was
        // NOT touched and the tombstone is intact.
        tokio::time::sleep(Duration::from_secs(2)).await;
        {
            let q = receiver.federation_queue.lock().await;
            assert!(
                q.get(&initiator_key).is_none(),
                "a Rejected peer must not be re-enqueued"
            );
        }
        {
            let reg = receiver.federation_registry.lock().await;
            assert_eq!(
                reg.get(&initiator_key).map(|r| r.state),
                Some(FederationState::Rejected),
                "tombstone must remain Rejected"
            );
        }

        receiver.shutdown().await;
        initiator.shutdown().await;
    }
}
