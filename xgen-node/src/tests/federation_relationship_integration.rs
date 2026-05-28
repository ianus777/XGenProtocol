// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// F-3 Federation-relationship verification gate integration tests —
// Phase 7 (runbook `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.7 +
// §3.7.1).
//
// Three scenarios:
//
// 1. Negative — `peer_without_relationship_rejects_with_federation_relationship_missing`:
//    peer X with NO entry in `SpaceState.federation_nodes` for Space S
//    pushes an event for S → `DispatchOutcome::Rejected` with reason
//    `federation_relationship_missing` naming (peer, space).
//
// 2. Positive — `peer_with_relationship_accepts`: peer A with an entry in
//    `federation_nodes` for Space S pushes an event for S → accepted
//    (positive-case regression: F-3 doesn't break the established-
//    relationship path).
//
// 3. B1 skip — `state_federation_add_skips_f3_check`: a state.federation_add
//    event from a peer X NOT yet in `federation_nodes` is accepted (B1's
//    chicken-and-egg skip rule). Verifies the lock B1 behaviour:
//    federation_add is the relationship-establishing event itself and must
//    pass F-3 even when the peer isn't yet federated.
//
// Tests run at the `NodeRuntime` level — no transport scaffolding needed
// because the F-3 surface is pure dispatcher-pipeline logic.

#[cfg(test)]
mod tests {
    use chrono::{SecondsFormat, Utc};
    use serde_json::json;

    use crate::{
        crypto::encoding,
        identity::{keypair, registry::IdentityRecord},
        node::runtime::{DispatchOutcome, EventOrigin, NodeRuntime},
        space::state::{
            build_federation_add_event, build_room_create_event, build_space_create_event,
            sign_event,
        },
        wire::types::{Event, EventType},
    };
    use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};

    fn idx(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn ndx(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn sdx(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn edx(s: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn rdx(s: &str) -> RoomXgid {
        RoomXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn event_id_str(ev: &Event) -> String {
        ev.event_id
            .as_ref()
            .expect("event must have event_id")
            .as_str()
            .to_string()
    }

    fn now_str() -> String {
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
    }

    fn pubkey_uri(key: &ed25519_dalek::SigningKey) -> String {
        format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        )
    }

    fn make_record(key: &ed25519_dalek::SigningKey, home_node: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: idx(&pubkey_uri(key)),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-19T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: ndx(home_node),
            update_version: 0,
        }
    }

    /// Build a Node with Alice as Space + Room member. Returns
    /// `(node, alice_key, alice_id, space_id, room_id, current_tip)`.
    /// No federation_peer is added; tests add one explicitly when needed
    /// (or omit it for the negative test).
    fn build_node_with_alice() -> (
        NodeRuntime,
        ed25519_dalek::SigningKey,
        String,
        String,
        String,
        String,
    ) {
        let node_key = keypair::generate();
        let mut node = NodeRuntime::new(node_key);
        let node_id_str = node.node_id.as_str().to_string();

        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node.register_identity(make_record(&alice_key, &node_id_str))
            .unwrap();

        let space_ev = sign_event(
            build_space_create_event(&alice_key, "test-space", None, 1, &node_id_str),
            &alice_key,
        );
        let space_id: String = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        let room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", None),
            &alice_key,
        );
        let room_id: String = event_id_str(&room_ev);
        node.ingest_event(room_ev);

        let invite_ev = sign_event(
            Event::new(
                EventType::MembershipInvite,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&space_id), edx(&room_id)],
                now_str(),
                json!({ "target_identity": alice_id, "role": "member" }),
            ),
            &alice_key,
        );
        let invite_id: String = event_id_str(&invite_ev);
        node.ingest_event(invite_ev);

        let join_ev = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&invite_id)],
                now_str(),
                json!({}),
            ),
            &alice_key,
        );
        let join_id: String = event_id_str(&join_ev);
        node.ingest_event(join_ev);

        (node, alice_key, alice_id, space_id, room_id, join_id)
    }

    /// Build a MessageText event from Alice referencing `prev`.
    fn build_alice_message(
        alice_key: &ed25519_dalek::SigningKey,
        alice_id: &str,
        space_id: &str,
        room_id: &str,
        prev: Vec<String>,
    ) -> Event {
        sign_event(
            Event::new(
                EventType::MessageText,
                idx(alice_id),
                rdx(room_id),
                sdx(space_id),
                prev.iter().map(|p| edx(p)).collect(),
                now_str(),
                json!({ "text": "hello from alice" }),
            ),
            alice_key,
        )
    }

    /// Test 1 (behaviour) — peer X with NO entry in federation_nodes pushes
    /// an event for Space S → HeldPending on the federation-relationship
    /// trigger (Phase 7.5 §6 — held-not-bypassed posture replaces Phase 7's
    /// permanent reject). The event waits for `state.federation_add` for
    /// (X, S) to land, or for the 180 s timeout to fire (4007).
    ///
    /// Phase 7's authoring vintage of this test asserted DispatchOutcome::Rejected.
    /// Phase 7.5 updates the assertion to HeldPending — F-3 is deferred,
    /// not weakened (the buffer is a holding cell, not a back-channel; the
    /// event is not accepted into storage until F-3 passes on re-validation).
    #[test]
    fn peer_without_relationship_held_pending_on_federation_relationship() {
        let (mut node, alice_key, alice_id, space_id, room_id, tip) = build_node_with_alice();
        let space_id_typed = sdx(&space_id);

        // No federation_peer has been added — federation_nodes is empty for
        // this Space.
        let unfederated_peer = keypair::generate();
        let unfederated_peer_id = pubkey_uri(&unfederated_peer);
        let unfederated_peer_id_typed = ndx(&unfederated_peer_id);

        let msg = build_alice_message(&alice_key, &alice_id, &space_id, &room_id, vec![tip]);
        let msg_event_id: String = event_id_str(&msg);

        let outcome = node.dispatch_event(
            msg,
            EventOrigin::ReceivedViaFederation,
            Some(&unfederated_peer_id_typed),
        );

        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "expected HeldPending, got {:?}",
            outcome
        );
        let buf = node
            .pending
            .get(&space_id_typed)
            .expect("pending buffer must exist for the Space");
        assert!(
            buf.contains(&msg_event_id),
            "event must be buffered on the federation-relationship trigger"
        );
        assert_eq!(
            buf.pending_federation_relationship_count(),
            1,
            "exactly one event should be on the federation-relationship counter"
        );
    }

    /// Test 2 (positive) — peer A with an entry in federation_nodes for
    /// Space S pushes an event for S → accepted.
    #[test]
    fn peer_with_relationship_accepts() {
        let (mut node, alice_key, alice_id, space_id, room_id, tip) = build_node_with_alice();
        let space_id_typed = sdx(&space_id);
        let node_key_clone = node.node_keypair.clone();

        // Add a federation_peer to the Space's federation_nodes (Phase 4 test
        // pattern). The state.federation_add is signed by THIS Node's
        // keypair, adding the federation_peer to federation_nodes[space_id].
        let federation_peer = keypair::generate();
        let federation_peer_id = pubkey_uri(&federation_peer);
        let federation_peer_id_typed = ndx(&federation_peer_id);
        let fed_add = sign_event(
            build_federation_add_event(
                &node_key_clone,
                &space_id,
                node.dag_tips(&space_id_typed),
                &federation_peer_id,
                "xgen://hash/sha256:test_session",
                "0.1",
                "json",
            ),
            &node_key_clone,
        );
        let fed_add_id: String = event_id_str(&fed_add);
        node.ingest_event(fed_add);
        assert!(
            node.spaces[&space_id_typed]
                .federation_nodes
                .iter()
                .any(|n| n == &federation_peer_id_typed),
            "setup: federation_nodes must include federation_peer"
        );

        // Dispatch an event from Alice as if relayed in by federation_peer.
        // The event references the post-federation_add tip.
        let _ = tip;
        let msg = build_alice_message(
            &alice_key,
            &alice_id,
            &space_id,
            &room_id,
            vec![fed_add_id],
        );

        let outcome = node.dispatch_event(
            msg,
            EventOrigin::ReceivedViaFederation,
            Some(&federation_peer_id_typed),
        );

        match outcome {
            DispatchOutcome::Accepted { new_joiner, .. } => {
                assert!(
                    new_joiner.is_none(),
                    "MessageText should not produce new_joiner"
                );
            }
            other => panic!("expected DispatchOutcome::Accepted, got {:?}", other),
        }
    }

    /// Test 3 (B1 skip) — state.federation_add events bypass the F-3 check.
    /// A federation_add arrives via federation when its peer is NOT yet in
    /// federation_nodes for Space S (that's what the event WILL ADD).
    /// Without B1's skip, this would reject as
    /// `federation_relationship_missing` and federation could never
    /// bootstrap.
    ///
    /// **Scope note.** This test verifies *only* that F-3 is skipped for
    /// state.federation_add — that the outcome is NOT
    /// `federation_relationship_missing`. The downstream validation core
    /// (step 11 sender registered + step 12 signature) treats the event's
    /// Node-pubkey sender per the Phase 6 unknown-signer rules; under those
    /// rules, a Node-pubkey not in `identity_registry` produces HeldPending
    /// (not Accept). That HeldPending behavior is a Phase 6 / pre-Phase-7
    /// consequence orthogonal to Lock B1. Asserting "not the F-3 rejection"
    /// is the precise B1 verification.
    #[test]
    fn state_federation_add_skips_f3_check() {
        let (mut node, _alice_key, _alice_id, space_id, _room_id, _tip) = build_node_with_alice();
        let space_id_typed = sdx(&space_id);
        let node_key_clone = node.node_keypair.clone();

        // peer X is NOT in federation_nodes. The federation_add event
        // signed by node_key adds X.
        let peer_x_key = keypair::generate();
        let peer_x_id = pubkey_uri(&peer_x_key);
        let peer_x_id_typed = ndx(&peer_x_id);

        let fed_add = sign_event(
            build_federation_add_event(
                &node_key_clone,
                &space_id,
                node.dag_tips(&space_id_typed),
                &peer_x_id,
                "xgen://hash/sha256:bootstrap_session",
                "0.1",
                "json",
            ),
            &node_key_clone,
        );

        // X delivers its own federation_add via federation. Without B1,
        // F-3 would reject (X is not in federation_nodes). With B1 (Phase
        // 7), F-3 is skipped and the event flows on to the validation
        // core.
        let outcome = node.dispatch_event(
            fed_add,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_x_id_typed),
        );

        // B1 verification: the outcome MUST NOT be the F-3 rejection.
        if let DispatchOutcome::Rejected(reason) = &outcome {
            assert!(
                !reason.contains("federation_relationship_missing"),
                "B1 skip failed — F-3 should have skipped state.federation_add but rejected: {reason}"
            );
        }
        // Other outcomes (Accepted, HeldPending, other rejections from
        // downstream validation steps) are all consistent with B1 being
        // applied. The test does not constrain the downstream outcome.
    }
}
