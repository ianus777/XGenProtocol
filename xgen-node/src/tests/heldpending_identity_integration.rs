// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// F-10 HeldPending generalisation integration tests — Phase 6 (runbook
// `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.6 + §3.6.1).
//
// Four scenarios per the runbook §3.6 Definition of Done:
//
// (a) Identity arrives within timeout → event validates on Identity-arrival
//     re-dispatch.
// (b) Predecessor arrives first within timeout, Identity arrives later but
//     still within timeout → event validates on second retry.
// (c) Identity never arrives, timeout fires, event discarded with 4006.
// (d) Both predecessors AND Identity missing — the reverse-order case from
//     (b): Identity arrives first while predecessor is still missing → event
//     stays buffered until predecessor also arrives.
//
// Tests are written at the `NodeRuntime` level (no transport / WebSocket
// scaffolding) because the F-10 surface is entirely in-process: the
// HeldPending buffer, the validation core's HeldPending shape, the
// `drain_pending_by_identity` hook. The Phase 6 survey confirmed
// `handle_identity_replicate_msg` in `xgen-node::app` is the single
// production arrival path; for these tests we simulate that path by calling
// `IdentityRegistry::upsert` + `NodeRuntime::drain_pending_by_identity`
// directly — same shape as the production hook.

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

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

    // Pass 3 Commit 2a test-fixture helpers.
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

    /// Build a Node with Alice known + Space + Room + Alice joined as a
    /// member + a `federation_peer` Node added to the Space's
    /// `federation_nodes`. Returns `(node, alice_key, alice_id, space_id,
    /// room_id, federation_peer_id, current_tips)`. Bob is NOT in the
    /// Identity registry — that's the scenario each F-10 test sets up
    /// against. The federation_peer setup lets ReceivedViaFederation
    /// dispatches pass the Phase 7 F-3 check (runbook §3.7.1 Lock A1):
    /// the test acts as if Bob's event was relayed in by `federation_peer`,
    /// which IS federated with us for this Space.
    fn build_node_with_alice_member() -> (
        NodeRuntime,
        ed25519_dalek::SigningKey,
        String,
        String,
        String,
        String,
        Vec<String>,
    ) {
        let node_key = keypair::generate();
        let mut node = NodeRuntime::new(node_key.clone());
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

        // Self-invite + join for Alice (mirrors Phase 4 test pattern).
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
        let _join_id: String = event_id_str(&join_ev);
        node.ingest_event(join_ev);

        // Phase 7 (F-3) setup: add a federation_peer Node to the Space's
        // federation_nodes so ReceivedViaFederation dispatches in the tests
        // below pass the F-3 check. Mirrors Phase 4
        // `federation_push_integration.rs` setup pattern.
        let federation_peer_key = keypair::generate();
        let federation_peer_id = pubkey_uri(&federation_peer_key);
        let fed_add = sign_event(
            build_federation_add_event(
                &node_key,
                &space_id,
                node.dag_tips(&sdx(&space_id)),
                &federation_peer_id,
                "xgen://hash/sha256:session_dummy_for_phase7_tests",
                "0.1",
                "json",
            ),
            &node_key,
        );
        let fed_add_id: String = event_id_str(&fed_add);
        node.ingest_event(fed_add);

        let tips = vec![fed_add_id];
        (node, alice_key, alice_id, space_id, room_id, federation_peer_id, tips)
    }

    /// Build Bob's MembershipJoin (self-invite + self-join is folded into
    /// one MembershipJoin event since validate_event's step 11 skips
    /// membership for MembershipJoin — the event itself makes the sender
    /// a member). prev_events references `current_tip` so the predecessor
    /// check passes once the tip is in store.
    fn build_bob_join(
        bob_key: &ed25519_dalek::SigningKey,
        bob_id: &str,
        space_id: &str,
        prev: Vec<String>,
    ) -> Event {
        sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(bob_id),
                rdx(""),
                sdx(space_id),
                prev.iter().map(|p| edx(p)).collect(),
                now_str(),
                json!({}),
            ),
            bob_key,
        )
    }

    /// Test (a) — Identity arrives within timeout → event validates on
    /// Identity-arrival re-dispatch. The clean F-10 happy path.
    #[test]
    fn identity_arrives_within_timeout_releases_event() {
        let (mut node, _alice_key, _alice_id, space_id, _room_id, federation_peer_id, tips) =
            build_node_with_alice_member();
        let federation_peer_id_typed = ndx(&federation_peer_id);
        let space_id_typed = sdx(&space_id);

        let bob_key = keypair::generate();
        let bob_id = pubkey_uri(&bob_key);

        // Bob's MembershipJoin signed by Bob's key, prev_events = current
        // tip (Alice's join). Bob's Identity is NOT in the registry yet.
        let bob_join = build_bob_join(&bob_key, &bob_id, &space_id, tips);
        let outcome = node.dispatch_event(
            bob_join,
            EventOrigin::ReceivedViaFederation,
            Some(&federation_peer_id_typed),
        );
        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "expected HeldPending due to unknown signer Identity; got {:?}",
            outcome
        );
        assert_eq!(
            node.pending.get(&space_id_typed).map(|b| b.len()).unwrap_or(0),
            1,
            "event should be buffered in this Space's PendingBuffer"
        );
        assert_eq!(
            node.pending.get(&space_id_typed).map(|b| b.pending_identity_count()).unwrap_or(0),
            1,
            "the buffered event should be counted toward pending_identity_count"
        );

        // Bob's Identity arrives via replication (simulating
        // handle_identity_replicate_msg's hook site).
        let node_id_str = node.node_id.as_str().to_string();
        node.register_identity(make_record(&bob_key, &node_id_str))
            .unwrap();
        let bob_id_typed = idx(&bob_id);
        node.drain_pending_by_identity(&bob_id_typed, EventOrigin::ReceivedViaFederation);

        assert!(
            node.pending.get(&space_id_typed).map(|b| b.is_empty()).unwrap_or(true),
            "PendingBuffer should be empty after Identity arrival + drain"
        );
        let space = node.spaces.get(&space_id_typed).expect("Space must exist");
        assert!(
            space.is_member(&bob_id),
            "Bob should be a Space member after his MembershipJoin re-validates and ingests"
        );
    }

    /// Test (b) — Predecessor arrives first within timeout, Identity arrives
    /// later but still within timeout → event validates on second retry.
    #[test]
    fn predecessor_first_then_identity_arrives_within_timeout_releases() {
        let (mut node, alice_key, _alice_id, space_id, room_id, federation_peer_id, tips) =
            build_node_with_alice_member();
        let federation_peer_id_typed = ndx(&federation_peer_id);
        let space_id_typed = sdx(&space_id);

        // Alice has a follow-up MessageText that will become Bob's
        // predecessor. Build it but DON'T dispatch yet so its event_id
        // is computed and reserved but it's absent from the store.
        let alice_followup = sign_event(
            Event::new(
                EventType::MessageText,
                idx(&pubkey_uri(&alice_key)),
                rdx(&room_id),
                sdx(&space_id),
                tips.iter().map(|t| edx(t)).collect(),
                now_str(),
                json!({ "text": "alice followup message" }),
            ),
            &alice_key,
        );
        let alice_followup_id: String = event_id_str(&alice_followup);

        let bob_key = keypair::generate();
        let bob_id = pubkey_uri(&bob_key);

        // Bob references the (not-yet-dispatched) alice_followup. At
        // dispatch time: predecessor missing + identity missing.
        let bob_join = build_bob_join(&bob_key, &bob_id, &space_id, vec![alice_followup_id.clone()]);
        let outcome = node.dispatch_event(
            bob_join,
            EventOrigin::ReceivedViaFederation,
            Some(&federation_peer_id_typed),
        );
        assert!(
            matches!(outcome, DispatchOutcome::HeldPending),
            "expected HeldPending (both predecessor and Identity missing)"
        );

        // Predecessor arrives — alice_followup is dispatched. Alice's
        // identity is known so it accepts and lands in store. The
        // predecessor-arrival drain fires (drain_pending_uniform), but
        // Bob's event stays buffered because his Identity is still
        // missing.
        let outcome = node.dispatch_event(alice_followup, EventOrigin::LocallySubmitted, None);
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "alice_followup should accept normally"
        );
        assert_eq!(
            node.pending.get(&space_id_typed).map(|b| b.len()).unwrap_or(0),
            1,
            "Bob's event should STILL be buffered — predecessor satisfied, identity not"
        );

        // Identity arrives second.
        let node_id_str = node.node_id.as_str().to_string();
        node.register_identity(make_record(&bob_key, &node_id_str))
            .unwrap();
        let bob_id_typed = idx(&bob_id);
        node.drain_pending_by_identity(&bob_id_typed, EventOrigin::ReceivedViaFederation);

        assert!(
            node.pending.get(&space_id_typed).map(|b| b.is_empty()).unwrap_or(true),
            "PendingBuffer empty after both dependencies satisfied"
        );
        let space = node.spaces.get(&space_id_typed).expect("Space must exist");
        assert!(space.is_member(&bob_id), "Bob should be a Space member after re-validation");
    }

    /// Test (c) — Identity never arrives, timeout fires, event discarded.
    /// Asserts the TimedOut payload carries the missing_identity and that
    /// the caller (in production, the timeout sweep task in
    /// `xgen-node::app::run_node`) would emit `4006 identity_record_timeout`
    /// per the predecessor-code-wins sub-rule (here, predecessors are empty
    /// so 4006 fires).
    #[test]
    fn identity_never_arrives_timeout_fires_4006_path() {
        let (mut node, _alice_key, _alice_id, space_id, _room_id, federation_peer_id, tips) =
            build_node_with_alice_member();
        let federation_peer_id_typed = ndx(&federation_peer_id);
        let space_id_typed = sdx(&space_id);

        let bob_key = keypair::generate();
        let bob_id = pubkey_uri(&bob_key);

        let bob_join = build_bob_join(&bob_key, &bob_id, &space_id, tips);
        let outcome = node.dispatch_event(
            bob_join,
            EventOrigin::ReceivedViaFederation,
            Some(&federation_peer_id_typed),
        );
        assert!(matches!(outcome, DispatchOutcome::HeldPending));

        // Simulate the timeout-sweep task firing after PENDING_TIMEOUT_SECS
        // have elapsed.
        let future =
            Instant::now() + Duration::from_secs(crate::dag::pending::PENDING_TIMEOUT_SECS + 1);
        let buf = node.pending.get_mut(&space_id_typed).expect("buffer must exist");
        let discarded = buf.drain_timed_out(
            future,
            Duration::from_secs(crate::dag::pending::FEDERATION_RELATIONSHIP_TIMEOUT_SECS),
        );
        assert_eq!(discarded.len(), 1, "exactly one event should time out");
        let timed_out = &discarded[0];
        assert!(
            timed_out.missing_predecessors.is_empty(),
            "predecessors were in store; only identity was missing"
        );
        assert_eq!(
            timed_out.missing_identity.as_ref().map(|i| i.as_str()),
            Some(bob_id.as_str()),
            "missing_identity should match Bob's id"
        );
        // Predecessor-code-wins sub-rule (verified at the emit site in
        // app.rs, not in the buffer): with missing_predecessors empty and
        // missing_identity set, the log emits error_code 4006. That code
        // path's branch condition is `!entry.missing_predecessors.is_empty()`
        // → false here → 4006 branch.
    }

    /// Test (d) — Both predecessors and Identity missing, Identity arrives
    /// first while predecessor is still missing → event stays buffered
    /// until predecessor also arrives. Reverse-order case from (b).
    #[test]
    fn both_missing_identity_first_then_predecessor_releases() {
        let (mut node, alice_key, _alice_id, space_id, room_id, federation_peer_id, tips) =
            build_node_with_alice_member();
        let federation_peer_id_typed = ndx(&federation_peer_id);
        let space_id_typed = sdx(&space_id);

        let alice_followup = sign_event(
            Event::new(
                EventType::MessageText,
                idx(&pubkey_uri(&alice_key)),
                rdx(&room_id),
                sdx(&space_id),
                tips.iter().map(|t| edx(t)).collect(),
                now_str(),
                json!({ "text": "alice followup message" }),
            ),
            &alice_key,
        );
        let alice_followup_id: String = event_id_str(&alice_followup);

        let bob_key = keypair::generate();
        let bob_id = pubkey_uri(&bob_key);

        let bob_join = build_bob_join(&bob_key, &bob_id, &space_id, vec![alice_followup_id.clone()]);
        let outcome = node.dispatch_event(
            bob_join,
            EventOrigin::ReceivedViaFederation,
            Some(&federation_peer_id_typed),
        );
        assert!(matches!(outcome, DispatchOutcome::HeldPending));

        // Identity arrives first — predecessor still missing.
        let node_id_str = node.node_id.as_str().to_string();
        node.register_identity(make_record(&bob_key, &node_id_str))
            .unwrap();
        let bob_id_typed = idx(&bob_id);
        node.drain_pending_by_identity(&bob_id_typed, EventOrigin::ReceivedViaFederation);

        // Bob's event must STILL be buffered (predecessor not yet
        // satisfied). The drain_pending_by_identity call cleared the
        // missing_identity flag on Bob's entry but try_release found
        // predecessors not all in store → no release.
        assert_eq!(
            node.pending.get(&space_id_typed).map(|b| b.len()).unwrap_or(0),
            1,
            "Bob's event should still be buffered — predecessor still missing"
        );
        assert_eq!(
            node.pending.get(&space_id_typed).map(|b| b.pending_identity_count()).unwrap_or(0),
            0,
            "missing_identity should be cleared after Identity arrival"
        );

        // Predecessor arrives second.
        let outcome = node.dispatch_event(alice_followup, EventOrigin::LocallySubmitted, None);
        assert!(matches!(outcome, DispatchOutcome::Accepted { .. }));

        // alice_followup's accept triggers drain_pending_uniform for
        // alice_followup_id — Bob's entry is now release-ready (preds
        // satisfied AND identity satisfied via the prior arrival).
        assert!(
            node.pending.get(&space_id_typed).map(|b| b.is_empty()).unwrap_or(true),
            "PendingBuffer should be empty after both dependencies satisfied"
        );
        let space = node.spaces.get(&space_id_typed).expect("Space must exist");
        assert!(space.is_member(&bob_id), "Bob should be a Space member after re-validation");
    }
}
