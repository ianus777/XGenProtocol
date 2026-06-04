// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Phase 1 smoke test — spec 3.7.11 (17-step end-to-end).
//
// Two in-process NodeRuntime instances. Alice on Node A, Bob on Node B.
// Drives all 17 steps and asserts final state on both nodes.

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::{SecondsFormat, Utc};
    use serde_json::json;

    use crate::{
        crypto::encoding,
        federation::handshake::{run_initiating, run_receiving},
        identity::{
            keypair,
            registry::IdentityRecord,
        },
        message::exchange::build_message_text_event,
        node::runtime::NodeRuntime,
        space::state::{
            build_federation_add_event, build_room_create_event, build_space_create_event,
            sign_event, verify_event_signature,
        },
        transport::{client, connection::Inbound, server::Server},
        wire::types::{Event, EventType, FederationCapabilities, TransportMessage},
    };
    use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};

    // Pass 3 Commit 2a test-fixture helpers: cheap typed-XGID constructors.
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

    fn now() -> String {
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
    }

    fn pubkey_uri(key: &ed25519_dalek::SigningKey) -> String {
        format!("xgen://pubkey/ed25519:{}", encoding::encode(key.verifying_key().as_bytes()))
    }

    fn make_record(key: &ed25519_dalek::SigningKey, home_node: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: idx(&pubkey_uri(key)),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-04-28T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: ndx(home_node),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    /// Build a membership event with explicit prev_events.
    fn membership_ev(
        key: &ed25519_dalek::SigningKey,
        space_id: &str,
        room_id: &str,
        event_type: EventType,
        prev_events: Vec<String>,
        content: serde_json::Value,
    ) -> Event {
        Event::new(
            event_type,
            idx(&pubkey_uri(key)),
            rdx(room_id),
            sdx(space_id),
            prev_events.iter().map(|p| edx(p)).collect(),
            now(),
            content,
        )
    }

    #[tokio::test]
    async fn smoke_test_phase1() {
        // ── Step 1: Node A generates keypair ─────────────────────────────────────
        let node_a_key = keypair::generate();
        let mut node_a = NodeRuntime::new(node_a_key.clone());

        // ── Step 2: Alice registers Identity on Node A ────────────────────────────
        let alice_key = keypair::generate();
        let node_a_id_str = node_a.node_id.as_str().to_string();
        node_a.register_identity(make_record(&alice_key, &node_a_id_str)).unwrap();

        // ── Step 3: Node B generates keypair ─────────────────────────────────────
        let node_b_key = keypair::generate();
        let mut node_b = NodeRuntime::new(node_b_key.clone());

        // ── Step 4: Bob registers Identity on Node B ──────────────────────────────
        let bob_key = keypair::generate();
        let bob_id = pubkey_uri(&bob_key);
        let node_b_id_str = node_b.node_id.as_str().to_string();
        node_b.register_identity(make_record(&bob_key, &node_b_id_str)).unwrap();

        // ── Step 5: Alice produces state.space_create ────────────────────────────
        let space_ev = sign_event(
            build_space_create_event(&alice_key, "XGen Test Space", None, 1, &node_a_id_str, None, false),
            &alice_key,
        );
        let space_id: String = event_id_str(&space_ev);
        node_a.ingest_event(space_ev);

        // ── Step 6: Alice produces state.room_create ─────────────────────────────
        let room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", None),
            &alice_key,
        );
        let room_id: String = event_id_str(&room_ev);
        node_a.ingest_event(room_ev);

        // ── Step 7: Alice produces membership.invite for Bob ─────────────────────
        // prev=[space_id, room_id] merges the two DAG roots into one chain.
        let invite_ev = sign_event(
            membership_ev(
                &alice_key,
                &space_id,
                "",
                EventType::MembershipInvite,
                vec![space_id.clone(), room_id.clone()],
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice_key,
        );
        let invite_id: String = event_id_str(&invite_ev);
        node_a.ingest_event(invite_ev);

        // Snapshot of Node A's current tips before federation (= invite_id).
        let space_id_typed = sdx(&space_id);
        let tips_before_federation = node_a.dag_tips(&space_id_typed);
        assert_eq!(tips_before_federation, vec![invite_id.clone()]);

        // ── Step 8: Node B connects to Node A — transport + federation handshake ──
        let mut server = Server::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let addr = server.local_addr();

        // Capture what the server task needs before moving into the closure.
        let server_node_key = node_a_key.clone();
        let history_snapshot = node_a.all_events(&space_id_typed);
        let fed_prev = tips_before_federation.clone();
        let space_id_task = space_id.clone();

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            conn.server_authenticate().await.unwrap();

            // F-1a tip-exchange (runbook §3.3 Locked wire shape): Node A's
            // tip for the Space is the current DAG tip (invite_id). The
            // server side sends this on Capabilities for bilateral exchange.
            let mut our_tips_a = BTreeMap::new();
            if let Some(tip) = fed_prev.iter().min().cloned() {
                our_tips_a.insert(space_id_task.clone(), tip);
            }

            let session = run_receiving(
                &mut conn,
                &server_node_key,
                FederationCapabilities::default(),
                our_tips_a,
            )
            .await
            .unwrap();

            // F-1a a-i symmetry rule (runbook §3.3.1 Lock 2): Node B's tips
            // are absent for this Space (brand-new join) and Node A has
            // events → Node A builds state.federation_add. Verify the peer's
            // tips map matches this expectation.
            assert!(
                session.peer_tips.get(&space_id_task).map(|s| s.is_empty()).unwrap_or(true),
                "expected Node B's tips[space_id] absent (brand-new join under F-1a)"
            );

            // Steps 9-10 fold into the F-1a tip-exchange handshake. Step 11
            // becomes the delta-stream + SyncComplete terminator (replaces the
            // pre-F-1a dump-then-`goodbye` shape).
            let fed_add_ev = sign_event(
                build_federation_add_event(
                    &server_node_key,
                    &space_id_task,
                    fed_prev,
                    session.peer_node_id.as_str(),
                    &session.session_id,
                    &session.negotiated_version,
                    &session.negotiated_serialisation,
                ),
                &server_node_key,
            );

            // Topological order: history events must arrive before
            // federation_add (which references invite_id in prev_events).
            for ev in &history_snapshot {
                conn.send_event(ev).await.unwrap();
            }
            conn.send_event(&fed_add_ev).await.unwrap();
            let complete = TransportMessage::SyncComplete {
                protocol_version: "0.1".to_string(),
                since: String::new(),
                new_tip: fed_add_ev
                    .event_id
                    .as_ref()
                    .map(|e| e.as_str().to_string())
                    .unwrap_or_default(),
                continue_from: None,
            };
            conn.send_transport(&complete).await.unwrap();

            (session, fed_add_ev)
        });

        // Client side (Node B).
        let mut conn = client::connect(addr).await.unwrap();
        conn.client_authenticate(&node_b_key).await.unwrap();

        // Brand-new join — empty tips for the Space; peer (Node A) computes
        // and streams the full Space history under the a-i symmetry rule.
        let client_session = run_initiating(
            &mut conn,
            &node_b_key,
            FederationCapabilities::default(),
            vec![space_id.clone()],
            BTreeMap::new(),
            None,
        )
        .await
        .unwrap();

        // Step 11 — drain delta; SyncComplete terminates. Goodbye + Closed
        // remain as fallbacks for pre-F-1a peers (Locked semantics: empty
        // tips field decay = full-history dump-then-close).
        loop {
            match conn.recv().await.unwrap() {
                Inbound::Event(ev) => node_b.ingest_event(ev),
                Inbound::Transport(TransportMessage::SyncComplete { .. }) => break,
                Inbound::Transport(TransportMessage::Goodbye { .. }) | Inbound::Closed => break,
                _ => {}
            }
        }

        let (server_session, fed_add_ev) = server_task.await.unwrap();

        // Apply the federation_add on Node A (it produced it but must also ingest it).
        node_a.ingest_event(fed_add_ev);

        // Handshake agreement.
        assert_eq!(client_session.session_id, server_session.session_id);

        // Node B now has the full Space state from history sync.
        assert!(node_b.spaces.contains_key(&space_id_typed), "Node B must have Space after history sync");
        assert!(
            node_b.spaces[&space_id_typed].rooms.contains_key(room_id.as_str()),
            "Node B must have Room after history sync"
        );
        // Bob's invite must be reflected in Node B's SpaceState.
        assert!(
            node_b.spaces[&space_id_typed].pending_invites.contains_key(bob_id.as_str()),
            "Node B must see Bob's pending invite"
        );
        // Node A must know Node B is now federated.
        assert!(
            node_a.spaces[&space_id_typed]
                .federation_nodes
                .iter()
                .any(|n| n == &node_b.node_id),
            "Node A must list Node B as federated"
        );

        // Node B's current tip after history sync is the federation_add event.
        let tip_after_sync = node_b.dag_tips(&space_id_typed);
        assert_eq!(tip_after_sync.len(), 1);
        let fed_add_id = tip_after_sync[0].clone();

        // Alice also needs to be known on Node B for message validation.
        node_b.register_identity(make_record(&alice_key, &node_a_id_str)).unwrap();

        // ── Step 12: Bob produces membership.join for the Space ───────────────
        let bob_join_space_ev = sign_event(
            membership_ev(
                &bob_key,
                &space_id,
                "",
                EventType::MembershipJoin,
                vec![fed_add_id],
                json!({}),
            ),
            &bob_key,
        );
        let bob_join_space_id: String = event_id_str(&bob_join_space_ev);
        node_b.ingest_event(bob_join_space_ev.clone());
        node_a.ingest_event(bob_join_space_ev); // propagate to Node A

        // ── Step 13: Bob produces membership.join for the Room ────────────────
        let bob_join_room_ev = sign_event(
            membership_ev(
                &bob_key,
                &space_id,
                &room_id,
                EventType::MembershipJoin,
                vec![bob_join_space_id.clone()],
                json!({}),
            ),
            &bob_key,
        );
        let bob_join_room_id: String = event_id_str(&bob_join_room_ev);
        node_b.ingest_event(bob_join_room_ev.clone());
        node_a.ingest_event(bob_join_room_ev); // propagate to Node A

        // Bob must now be in the Space + Room on both nodes.
        assert!(node_a.spaces[&space_id_typed].is_member(&bob_id), "Node A: Bob must be Space member");
        assert!(node_b.spaces[&space_id_typed].is_member(&bob_id), "Node B: Bob must be Space member");
        assert!(
            node_a.spaces[&space_id_typed].is_room_member(&bob_id, &room_id),
            "Node A: Bob must be Room member"
        );
        assert!(
            node_b.spaces[&space_id_typed].is_room_member(&bob_id, &room_id),
            "Node B: Bob must be Room member"
        );

        // Bob must be known to Node A's identity registry for message validation.
        node_a.register_identity(make_record(&bob_key, &node_b_id_str)).unwrap();

        // Both nodes' tips are now bob_join_room_id.
        assert!(node_a.graphs[&space_id_typed].is_tip(&bob_join_room_id));
        assert!(node_b.graphs[&space_id_typed].is_tip(&bob_join_room_id));

        // ── Step 14: Alice produces message.text ("Hello Bob") ───────────────
        let hello_bob_ev = sign_event(
            build_message_text_event(
                &alice_key,
                &space_id,
                &room_id,
                vec![bob_join_room_id.clone()],
                "Hello Bob",
            ),
            &alice_key,
        );
        let hello_bob_id: String = event_id_str(&hello_bob_ev);

        node_a.accept_message(&space_id_typed, hello_bob_ev.clone()).unwrap();
        // Propagate to Node B.
        node_b.accept_message(&space_id_typed, hello_bob_ev).unwrap();

        // ── Step 15: Bob produces message.text ("Hello Alice") ───────────────
        // Bob's message references Alice's message as predecessor (linear chain).
        let hello_alice_ev = sign_event(
            build_message_text_event(
                &bob_key,
                &space_id,
                &room_id,
                vec![hello_bob_id.clone()],
                "Hello Alice",
            ),
            &bob_key,
        );
        let hello_alice_id: String = event_id_str(&hello_alice_ev);

        node_b.accept_message(&space_id_typed, hello_alice_ev.clone()).unwrap();
        // Propagate to Node A.
        node_a.accept_message(&space_id_typed, hello_alice_ev).unwrap();

        // ── Step 16: Both Nodes have both Events in their Room DAG ────────────
        assert!(
            node_a.stores[&space_id_typed].contains(&EventXgid::from_xgid(Xgid::new(hello_bob_id.to_string()))),
            "Node A must have Alice's message"
        );
        assert!(
            node_a.stores[&space_id_typed].contains(&EventXgid::from_xgid(Xgid::new(hello_alice_id.to_string()))),
            "Node A must have Bob's message"
        );
        assert!(
            node_b.stores[&space_id_typed].contains(&EventXgid::from_xgid(Xgid::new(hello_bob_id.to_string()))),
            "Node B must have Alice's message"
        );
        assert!(
            node_b.stores[&space_id_typed].contains(&EventXgid::from_xgid(Xgid::new(hello_alice_id.to_string()))),
            "Node B must have Bob's message"
        );

        // Signature integrity on both nodes for both messages.
        for (label, store) in [("Node A", &node_a.stores[&space_id_typed]), ("Node B", &node_b.stores[&space_id_typed])] {
            let msg_a = store.get(&EventXgid::from_xgid(Xgid::new(hello_bob_id.to_string()))).unwrap().unwrap();
            let msg_b = store.get(&EventXgid::from_xgid(Xgid::new(hello_alice_id.to_string()))).unwrap().unwrap();
            assert!(verify_event_signature(&msg_a), "{label}: Alice's message signature must be valid");
            assert!(verify_event_signature(&msg_b), "{label}: Bob's message signature must be valid");
        }

        // ── Step 17: Both clients can display the conversation ────────────────
        let alice_msg_on_b = node_b.stores[&space_id_typed].get(&EventXgid::from_xgid(Xgid::new(hello_bob_id.to_string()))).unwrap().unwrap();
        assert_eq!(alice_msg_on_b.content["text"].as_str().unwrap(), "Hello Bob");

        let bob_msg_on_a = node_a.stores[&space_id_typed].get(&EventXgid::from_xgid(Xgid::new(hello_alice_id.to_string()))).unwrap().unwrap();
        assert_eq!(bob_msg_on_a.content["text"].as_str().unwrap(), "Hello Alice");

        // Final DAG tip on both nodes is hello_alice_id (linear chain).
        assert!(node_a.graphs[&space_id_typed].is_tip(&hello_alice_id), "Node A tip must be hello_alice");
        assert!(node_b.graphs[&space_id_typed].is_tip(&hello_alice_id), "Node B tip must be hello_alice");
    }
}
