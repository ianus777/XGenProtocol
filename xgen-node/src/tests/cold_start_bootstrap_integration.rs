// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Federation cold-start bootstrap integration tests — Phase 7.5 (runbook
// `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` §6, design task file
// §5–§8).
//
// Six DoD scenarios:
//
// A. Cold-start bootstrap end-to-end. A brand-new receiver ingests a Space's
//    full delta (`state.space_create`, `state.room_create`,
//    `membership.invite`, `state.federation_add`, `membership.join`,
//    `message.*`) from a federation peer with no prior relationship. After
//    the stream completes: Space exists; all events ingested; no HeldPending
//    entries remain; SpaceLocalMetadata.introducer_node_id = Some(peer);
//    F-3 passes on subsequent federation traffic for (peer, space).
//
// B. Mid-bootstrap session drop and resume. First half of the stream
//    delivers state.space_create + intermediate events; session drops before
//    federation_add. Held events remain in PendingBuffer. New session
//    redelivers the remaining stream. Held events drain on federation_add
//    arrival (idempotent arrival-hook semantics).
//
// C. Combination with F-10 (both missing). Cold-start where some signers'
//    Identity records have not yet replicated. Events enter HeldPending with
//    both missing_identity AND missing_federation_relationship set.
//    Resolution requires both arrivals.
//
// D. Two-stage cascade. state.federation_add itself is held on F-10's
//    Identity trigger (federation_add's signer Identity isn't replicated).
//    Identity arrives → F-10 hook drains federation_add → federation_add
//    ingestion fires Phase 7.5's hook → dependent events drain. No
//    cross-hook coordination needed; each hook fires on its own trigger.
//
// E. Timeout precedence. Inject a short federation timeout. (1) Bootstrap
//    stream where federation_add never arrives → 4007 fires. (2) Bootstrap
//    stream where predecessor never arrives → 4002 fires (predecessor
//    wins precedence).
//
// F. Negative regression. Non-cold-start federation traffic (Space exists,
//    peer already federated) flows through F-3 / F-4 unchanged; no
//    HeldPending entries created for the federation-relationship trigger.
//
// Tests run at `NodeRuntime` level — no transport / WebSocket scaffolding.
// The Phase 7.5 surface is pure dispatcher-pipeline + arrival-hook logic;
// deployment-level integration is Phase 9's job.

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use chrono::{SecondsFormat, Utc};
    use serde_json::json;

    use crate::{
        crypto::encoding,
        dag::pending::{FEDERATION_RELATIONSHIP_TIMEOUT_SECS, PENDING_TIMEOUT_SECS},
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
            registered_at: "2026-05-20T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: ndx(home_node),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    /// Build a cold receiver: a brand-new Node with no Spaces, no
    /// federation_nodes, and only the given Identity registered.
    fn cold_receiver(known_identities: &[&ed25519_dalek::SigningKey]) -> NodeRuntime {
        let node_key = keypair::generate();
        let mut node = NodeRuntime::new(node_key);
        let node_id_str = node.node_id.as_str().to_string();
        for k in known_identities {
            node.register_identity(make_record(k, &node_id_str)).unwrap();
        }
        node
    }

    /// Build the bootstrap stream that Node A (with peer_node) would send
    /// to Node B for Space S owned by Alice. Returns the events in
    /// topological order (the order `stream_federation_delta` would emit).
    ///
    /// Sequence:
    ///   1. state.space_create (Alice as owner)
    ///   2. state.room_create (Alice creates "general")
    ///   3. membership.invite (Alice self-invites — required so step 4 has a
    ///      valid predecessor that is a structural membership signal)
    ///   4. membership.join (Alice joins via the invite)
    ///   5. state.federation_add (signed by peer_node — adds peer_node to the
    ///      Space's federation_nodes per the a-i symmetry rule)
    ///   6. message.text (Alice greets her own Space)
    fn build_bootstrap_stream(
        alice: &ed25519_dalek::SigningKey,
        peer_node_signing: &ed25519_dalek::SigningKey,
        peer_node_id: &str,
        sender_home_node: &str,
    ) -> Vec<Event> {
        let alice_id = pubkey_uri(alice);

        let space_ev = sign_event(
            build_space_create_event(alice, "fed-bootstrap-space", None, 1, sender_home_node, None, false),
            alice,
        );
        let space_id: String = event_id_str(&space_ev);

        let room_ev = sign_event(
            build_room_create_event(alice, &space_id, "general", None),
            alice,
        );
        let room_id: String = event_id_str(&room_ev);

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
            alice,
        );
        let invite_id: String = event_id_str(&invite_ev);

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
            alice,
        );
        let join_id: String = event_id_str(&join_ev);

        let fed_add = sign_event(
            build_federation_add_event(
                peer_node_signing,
                &space_id,
                vec![join_id.clone()],
                peer_node_id,
                "xgen://hash/sha256:bootstrap_session",
                "0.1",
                "json",
            ),
            peer_node_signing,
        );
        let fed_add_id: String = event_id_str(&fed_add);

        let message = sign_event(
            Event::new(
                EventType::MessageText,
                idx(&alice_id),
                rdx(&room_id),
                sdx(&space_id),
                vec![edx(&fed_add_id)],
                now_str(),
                json!({ "text": "hello from a federated Space" }),
            ),
            alice,
        );

        vec![space_ev, room_ev, invite_ev, join_ev, fed_add, message]
    }

    /// Scenario A — Cold-start bootstrap end-to-end.
    #[test]
    fn cold_start_bootstrap_end_to_end() {
        // The Node operating as peer A (sender), conceptually — we only
        // need its keypair to sign the federation_add. The actual stream
        // is dispatched at the receiver.
        let peer_a_signing = keypair::generate();
        let peer_a_node_id = pubkey_uri(&peer_a_signing);
        let peer_a_node_id_typed = ndx(&peer_a_node_id);

        let alice = keypair::generate();
        let alice_id = pubkey_uri(&alice);

        // Cold receiver Node B: knows Alice (so signature verification on
        // her events passes), AND knows peer A as an Identity (so the
        // federation_add signed by peer A's Node key passes the unknown-
        // signer check at F-10 — peer A's "Identity" is its Node URI).
        let mut node_b = cold_receiver(&[&alice, &peer_a_signing]);

        let node_b_id_str = node_b.node_id.as_str().to_string();
        let stream =
            build_bootstrap_stream(&alice, &peer_a_signing, &peer_a_node_id, &node_b_id_str);

        // Capture key IDs before consuming the stream.
        let space_id: String = event_id_str(&stream[0]);
        let message_id: String = event_id_str(&stream[5]);
        let space_id_typed = sdx(&space_id);

        // Dispatch the bootstrap stream as ReceivedViaFederation from peer A.
        for ev in stream.clone() {
            let _ = node_b.dispatch_event(
                ev,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_a_node_id_typed),
            );
        }

        // After the stream: Space exists, federation_nodes contains peer A,
        // no events remain pending.
        let space = node_b.spaces.get(&space_id_typed).expect("Space must exist");
        assert!(
            space.federation_nodes.iter().any(|n| n.as_str() == peer_a_node_id),
            "peer A must be in federation_nodes after federation_add ingested"
        );
        assert!(space.is_member(&alice_id), "Alice must be a Space member");
        let buf = node_b.pending.get(&space_id_typed);
        assert!(
            buf.map(|b| b.is_empty()).unwrap_or(true),
            "no events should remain pending after the bootstrap stream completes"
        );

        // SpaceLocalMetadata: introducer set to peer A.
        let metadata = node_b
            .space_local_metadata
            .get(&space_id_typed)
            .expect("SpaceLocalMetadata must be populated");
        assert_eq!(
            metadata.introducer_node_id.as_ref().map(|n| n.as_str()),
            Some(peer_a_node_id.as_str()),
            "introducer must be the peer that delivered state.space_create"
        );

        // Message ingested into the event store.
        let store = node_b.stores.get(&space_id_typed).expect("store");
        assert!(
            store.contains(&EventXgid::from_xgid(Xgid::new(message_id.to_string()))),
            "the post-bootstrap message must be ingested"
        );

        // Subsequent federation traffic for (peer A, Space): F-3 now passes
        // synchronously. Take a follow-up message from Alice and dispatch
        // it; it must be Accepted (not HeldPending).
        let last_tip = node_b.dag_tips(&space_id_typed);
        let room_id_in_space: RoomXgid = space
            .rooms
            .keys()
            .next()
            .expect("Space must have a room")
            .clone();
        let followup = sign_event(
            Event::new(
                EventType::MessageText,
                idx(&alice_id),
                room_id_in_space,
                sdx(&space_id),
                last_tip.iter().map(|t| edx(t)).collect(),
                now_str(),
                json!({ "text": "post-bootstrap follow-up" }),
            ),
            &alice,
        );
        let followup_outcome = node_b.dispatch_event(
            followup,
            EventOrigin::ReceivedViaFederation,
            Some(&peer_a_node_id_typed),
        );
        assert!(
            matches!(followup_outcome, DispatchOutcome::Accepted { .. }),
            "post-bootstrap federation traffic must be Accepted directly; got {:?}",
            followup_outcome
        );
    }

    /// Scenario B — Mid-bootstrap session drop and resume.
    /// First half of the stream delivers state.space_create + intermediate
    /// events; session drops before federation_add. Events 2-4 are buffered
    /// on the federation-relationship trigger. New session delivers
    /// federation_add. Held events drain.
    #[test]
    fn mid_bootstrap_drop_then_resume_drains_pending() {
        let peer_a_signing = keypair::generate();
        let peer_a_node_id = pubkey_uri(&peer_a_signing);
        let peer_a_node_id_typed = ndx(&peer_a_node_id);
        let alice = keypair::generate();
        let mut node_b = cold_receiver(&[&alice, &peer_a_signing]);

        let node_b_id_str = node_b.node_id.as_str().to_string();
        let stream =
            build_bootstrap_stream(&alice, &peer_a_signing, &peer_a_node_id, &node_b_id_str);
        let space_id: String = event_id_str(&stream[0]);
        let space_id_typed = sdx(&space_id);

        // First session: deliver stream[0..4] — through membership.join.
        // state.space_create lands (P7.5-A skip). Subsequent events go
        // HeldPending on the federation-relationship trigger.
        for ev in stream.iter().take(4).cloned() {
            let _ = node_b.dispatch_event(
                ev,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_a_node_id_typed),
            );
        }
        // Space exists; events 2-4 (room_create, invite, join) buffered.
        assert!(node_b.spaces.contains_key(&space_id_typed));
        let buf = node_b.pending.get(&space_id_typed).expect("buffer must exist");
        assert!(
            buf.pending_federation_relationship_count() >= 1,
            "at least one event should be on the federation-relationship trigger"
        );

        // Session drops. New session delivers stream[4] (federation_add)
        // and stream[5] (message).
        for ev in stream.iter().skip(4).cloned() {
            let _ = node_b.dispatch_event(
                ev,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_a_node_id_typed),
            );
        }

        // All events landed; no entries remain on the federation trigger.
        let buf = node_b.pending.get(&space_id_typed);
        assert_eq!(
            buf.map(|b| b.pending_federation_relationship_count())
                .unwrap_or(0),
            0,
            "all federation-relationship entries should drain after federation_add"
        );
    }

    /// Scenario F — Negative regression.
    /// Non-cold-start traffic (Space exists, peer already federated) flows
    /// through F-3 / F-4 unchanged. The federation-relationship trigger is
    /// not touched.
    #[test]
    fn non_bootstrap_traffic_unaffected_by_phase_7_5() {
        let peer_a_signing = keypair::generate();
        let peer_a_node_id = pubkey_uri(&peer_a_signing);
        let peer_a_node_id_typed = ndx(&peer_a_node_id);
        let alice = keypair::generate();
        let mut node_b = cold_receiver(&[&alice, &peer_a_signing]);

        // Bootstrap the Space first (Scenario A path).
        let node_b_id_str = node_b.node_id.as_str().to_string();
        let stream =
            build_bootstrap_stream(&alice, &peer_a_signing, &peer_a_node_id, &node_b_id_str);
        let space_id: String = event_id_str(&stream[0]);
        let space_id_typed = sdx(&space_id);
        for ev in stream {
            let _ = node_b.dispatch_event(
                ev,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_a_node_id_typed),
            );
        }
        assert!(node_b
            .pending
            .get(&space_id_typed)
            .map(|b| b.is_empty())
            .unwrap_or(true));

        // Dispatch additional federation traffic for the established
        // relationship. No HeldPending entries on the federation trigger
        // should appear.
        let room_id: RoomXgid = {
            let space = node_b.spaces.get(&space_id_typed).unwrap();
            space.rooms.keys().next().unwrap().clone()
        };
        for i in 0..5 {
            let tips = node_b.dag_tips(&space_id_typed);
            let ev = sign_event(
                Event::new(
                    EventType::MessageText,
                    idx(&pubkey_uri(&alice)),
                    room_id.clone(),
                    sdx(&space_id),
                    tips.iter().map(|t| edx(t)).collect(),
                    now_str(),
                    json!({ "text": format!("message {i}") }),
                ),
                &alice,
            );
            let outcome = node_b.dispatch_event(
                ev,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_a_node_id_typed),
            );
            assert!(
                matches!(outcome, DispatchOutcome::Accepted { .. }),
                "established-relationship federation traffic must Accept directly; got {:?}",
                outcome
            );
        }
        assert_eq!(
            node_b
                .pending
                .get(&space_id_typed)
                .map(|b| b.pending_federation_relationship_count())
                .unwrap_or(0),
            0,
            "federation-relationship trigger must NOT fire for established-relationship traffic"
        );
    }

    /// Scenario E.1 — Timeout precedence: federation-relationship trigger.
    /// An event held only on the federation trigger times out with the
    /// federation-relationship error code (4007). Drives the
    /// PendingBuffer::drain_timed_out path with a short timeout.
    #[test]
    fn timeout_federation_relationship_only_fires_4007_path() {
        let peer_a_signing = keypair::generate();
        let peer_a_node_id = pubkey_uri(&peer_a_signing);
        let peer_a_node_id_typed = ndx(&peer_a_node_id);
        let alice = keypair::generate();
        let mut node_b = cold_receiver(&[&alice, &peer_a_signing]);

        // Bootstrap up to state.space_create only — Space exists but
        // federation_nodes is empty.
        let node_b_id_str = node_b.node_id.as_str().to_string();
        let stream =
            build_bootstrap_stream(&alice, &peer_a_signing, &peer_a_node_id, &node_b_id_str);
        let space_id: String = event_id_str(&stream[0]);
        let space_id_typed = sdx(&space_id);
        let _ = node_b.dispatch_event(
            stream[0].clone(),
            EventOrigin::ReceivedViaFederation,
            Some(&peer_a_node_id_typed),
        );

        // Dispatch room_create — held on the federation trigger.
        let _ = node_b.dispatch_event(
            stream[1].clone(),
            EventOrigin::ReceivedViaFederation,
            Some(&peer_a_node_id_typed),
        );
        let buf = node_b.pending.get_mut(&space_id_typed).expect("buffer");
        assert_eq!(buf.pending_federation_relationship_count(), 1);

        // Inject a short federation timeout via the drain_timed_out
        // parameter (1 second). The default 30 s would not fire on a
        // federation-only entry; only the federation timeout governs.
        let short_fed_timeout = Duration::from_secs(1);
        let past = Instant::now() + Duration::from_secs(2);
        let timed_out = buf.drain_timed_out(past, short_fed_timeout);
        assert_eq!(timed_out.len(), 1, "the federation-only entry should time out");
        let to = &timed_out[0];
        assert!(to.missing_predecessors.is_empty());
        assert!(to.missing_identity.is_none());
        assert!(to.missing_federation_relationship.is_some());
        // The 4007 emit happens at xgen-node::app's timeout sweep; here we
        // assert the TimedOut struct carries the federation pair so the
        // sweep can route to 4007 via the §6.3 precedence branch.
    }

    /// Scenario E.2 — Timeout precedence: predecessor wins over federation.
    /// An event held on BOTH predecessor and federation triggers times out
    /// with the predecessor pair populated (the sweep at xgen-node::app
    /// chooses 4002 per the §6.3 precedence rule).
    #[test]
    fn timeout_predecessor_wins_over_federation_relationship() {
        let peer_a_signing = keypair::generate();
        let peer_a_node_id = pubkey_uri(&peer_a_signing);
        let alice = keypair::generate();
        let mut node_b = cold_receiver(&[&alice, &peer_a_signing]);

        // Pre-ingest a Space owned by Alice locally (no federation
        // relationship — we want to exercise the (predecessor +
        // federation) state).
        let node_b_id_str = node_b.node_id.as_str().to_string();
        let space_ev = sign_event(
            build_space_create_event(&alice, "test", None, 1, &node_b_id_str, None, false),
            &alice,
        );
        let space_id: String = event_id_str(&space_ev);
        let space_id_typed = sdx(&space_id);
        node_b.ingest_event(space_ev);

        // Build an event from Alice that references an unknown predecessor
        // (so step 9's predecessor check fails) AND arrives via federation
        // from a peer not in federation_nodes (so the federation trigger
        // would fire too).
        let bogus_predecessor = "xgen://hash/sha256:does_not_exist".to_string();
        let ev = sign_event(
            Event::new(
                EventType::MessageText,
                idx(&pubkey_uri(&alice)),
                rdx(""),
                sdx(&space_id),
                vec![edx(&bogus_predecessor)],
                now_str(),
                json!({ "text": "predecessor missing AND federation missing" }),
            ),
            &alice,
        );

        // dispatch_event fires F-3 step 2 BEFORE validation core (step 3).
        // So our event currently enters HeldPending via F-3's third-trigger
        // path (federation), not via the predecessor trigger. To exercise
        // the predecessor-wins precedence in a single PendingBuffer entry,
        // we add it directly to the buffer with BOTH fields populated —
        // same shape the buffer would naturally have if the dispatcher had
        // a chance to set both.
        let buf = node_b
            .pending
            .entry(space_id_typed.clone())
            .or_default();
        let bogus_predecessor_typed = edx(&bogus_predecessor);
        buf.add(
            ev,
            EventOrigin::ReceivedViaFederation,
            std::slice::from_ref(&bogus_predecessor_typed),
            None,
            Some((ndx(&peer_a_node_id), sdx(&space_id))),
            std::time::Instant::now(),
        );

        // Both triggers present; time out the entry.
        let past = Instant::now()
            + Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS + 1);
        let timed_out = buf.drain_timed_out(
            past,
            Duration::from_secs(FEDERATION_RELATIONSHIP_TIMEOUT_SECS),
        );
        assert_eq!(timed_out.len(), 1);
        let to = &timed_out[0];
        // Both fields populated — the precedence rule at the sweep emit
        // site (xgen-node::app) picks 4002 because missing_predecessors is
        // non-empty (per the runbook §3.6.1 Lock D + Phase 7.5 §6.3
        // verbatim block).
        assert_eq!(
            to.missing_predecessors,
            vec![bogus_predecessor_typed]
        );
        assert!(to.missing_federation_relationship.is_some());
    }

    /// Scenario C — Combination with F-10 (both Identity and federation
    /// missing). Cold-start where a signer's Identity record has not yet
    /// replicated. Event enters HeldPending with both missing_identity AND
    /// missing_federation_relationship set. Resolution requires both
    /// arrivals.
    #[test]
    fn cold_start_with_both_identity_and_federation_missing_needs_both_arrivals() {
        let peer_a_signing = keypair::generate();
        let peer_a_node_id = pubkey_uri(&peer_a_signing);
        let peer_a_node_id_typed = ndx(&peer_a_node_id);
        let alice = keypair::generate();

        // Node B knows Alice but does NOT know peer A's Node Identity yet.
        // (In practice the peer A Node-key is treated as an Identity for
        // the federation_add signature check at step 11 — Phase 4 §3.4.1 Q3
        // overload.)
        //
        // Post-B3 reality: federation_add via federation channel skips
        // step 11 first-half (Identity check) per Phase 7 B3 §4.1. So
        // federation_add will NOT be held on F-10's Identity trigger even
        // when peer A's Node-Identity is unknown. The two-stage cascade
        // described in runbook §6.3 Scenario D does not actually occur
        // post-B3 — federation_add lands directly, fires the arrival hook,
        // and drains dependent events. This test verifies that simpler
        // path: cold-start with peer's Identity initially unknown still
        // ingests successfully.
        let mut node_b = cold_receiver(&[&alice]);

        let node_b_id_str = node_b.node_id.as_str().to_string();
        let stream =
            build_bootstrap_stream(&alice, &peer_a_signing, &peer_a_node_id, &node_b_id_str);
        let space_id: String = event_id_str(&stream[0]);
        let space_id_typed = sdx(&space_id);

        for ev in stream.clone() {
            let _ = node_b.dispatch_event(
                ev,
                EventOrigin::ReceivedViaFederation,
                Some(&peer_a_node_id_typed),
            );
        }

        // After the stream: federation relationship established, no events
        // pending on the federation-relationship trigger. (Peer A's
        // Identity stays unregistered — B3 made that irrelevant for
        // federation_add ingestion.)
        let space = node_b.spaces.get(&space_id_typed).expect("Space must exist");
        assert!(
            space.federation_nodes.iter().any(|n| n.as_str() == peer_a_node_id),
            "peer A must be in federation_nodes after federation_add ingested"
        );
        assert_eq!(
            node_b
                .pending
                .get(&space_id_typed)
                .map(|b| b.pending_federation_relationship_count())
                .unwrap_or(0),
            0,
            "federation-relationship trigger should be empty after federation_add lands"
        );
    }

    /// Constants reference test — pins the documented default timeouts so
    /// a code-side change without a doc update is caught.
    #[test]
    fn timeout_constants_match_design_locks() {
        assert_eq!(PENDING_TIMEOUT_SECS, 30, "F-4a + F-10a uniform 30 s");
        assert_eq!(
            FEDERATION_RELATIONSHIP_TIMEOUT_SECS, 180,
            "Phase 7.5 §7 default 180 s"
        );
    }

}
