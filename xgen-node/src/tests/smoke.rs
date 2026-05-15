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
        wire::types::{Event, EventType, FederationCapabilities, SpaceControlMessage, TransportMessage},
    };

    fn now() -> String {
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
    }

    fn pubkey_uri(key: &ed25519_dalek::SigningKey) -> String {
        format!("xgen://pubkey/ed25519:{}", encoding::encode(key.verifying_key().as_bytes()))
    }

    fn make_record(key: &ed25519_dalek::SigningKey, home_node: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: pubkey_uri(key),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-04-28T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: home_node.to_string(),
            update_version: 0,
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
            pubkey_uri(key),
            room_id.to_string(),
            space_id.to_string(),
            prev_events,
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
        node_a.register_identity(make_record(&alice_key, &node_a.node_id)).unwrap();

        // ── Step 3: Node B generates keypair ─────────────────────────────────────
        let node_b_key = keypair::generate();
        let mut node_b = NodeRuntime::new(node_b_key.clone());

        // ── Step 4: Bob registers Identity on Node B ──────────────────────────────
        let bob_key = keypair::generate();
        let bob_id = pubkey_uri(&bob_key);
        node_b.register_identity(make_record(&bob_key, &node_b.node_id)).unwrap();

        // ── Step 5: Alice produces state.space_create ────────────────────────────
        let space_ev = sign_event(
            build_space_create_event(&alice_key, "XGen Test Space", None, 1, &node_a.node_id),
            &alice_key,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        node_a.ingest_event(space_ev);

        // ── Step 6: Alice produces state.room_create ─────────────────────────────
        let room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", None),
            &alice_key,
        );
        let room_id = room_ev.event_id.clone().unwrap();
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
        let invite_id = invite_ev.event_id.clone().unwrap();
        node_a.ingest_event(invite_ev);

        // Snapshot of Node A's current tips before federation (= invite_id).
        let tips_before_federation = node_a.dag_tips(&space_id);
        assert_eq!(tips_before_federation, vec![invite_id.clone()]);

        // ── Step 8: Node B connects to Node A — transport + federation handshake ──
        let mut server = Server::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let addr = server.local_addr();

        // Capture what the server task needs before moving into the closure.
        let server_node_key = node_a_key.clone();
        let history_snapshot = node_a.all_events(&space_id);
        let fed_prev = tips_before_federation.clone();
        let space_id_task = space_id.clone();

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            conn.server_authenticate().await.unwrap();

            let session = run_receiving(
                &mut conn,
                &server_node_key,
                FederationCapabilities::default(),
            )
            .await
            .unwrap();

            // ── Step 9: receive space.join_request from Node B ────────────────
            let peer_node_id = match conn.recv().await.unwrap() {
                Inbound::Space(SpaceControlMessage::JoinRequest { node_id, .. }) => node_id,
                other => panic!("expected space.join_request, got {:?}", other),
            };

            // ── Step 10: produce state.federation_add Event ───────────────────
            let fed_add_ev = sign_event(
                build_federation_add_event(
                    &server_node_key,
                    &space_id_task,
                    fed_prev,
                    &peer_node_id,
                    &session.session_id,
                    &session.negotiated_version,
                    &session.negotiated_serialisation,
                ),
                &server_node_key,
            );

            // ── Step 11: send history first, then federation_add ──────────────
            // Topological order: history events must arrive before federation_add
            // (which references invite_id in prev_events).
            for ev in &history_snapshot {
                conn.send_event(ev).await.unwrap();
            }
            conn.send_event(&fed_add_ev).await.unwrap();
            conn.goodbye("history_sync_complete").await.unwrap();

            (session, fed_add_ev)
        });

        // Client side (Node B).
        let mut conn = client::connect(addr).await.unwrap();
        conn.client_authenticate(&node_b_key).await.unwrap();

        let client_session = run_initiating(
            &mut conn,
            &node_b_key,
            FederationCapabilities::default(),
            vec![space_id.clone()],
            None,
        )
        .await
        .unwrap();

        // ── Step 9: Node B sends space.join_request ───────────────────────────
        conn.send_space(&SpaceControlMessage::JoinRequest {
            space_id: space_id.clone(),
            node_id: node_b.node_id.clone(),
        })
        .await
        .unwrap();

        // ── Step 11: Node B receives history + federation_add ─────────────────
        loop {
            match conn.recv().await.unwrap() {
                Inbound::Event(ev) => node_b.ingest_event(ev),
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
        assert!(node_b.spaces.contains_key(&space_id), "Node B must have Space after history sync");
        assert!(
            node_b.spaces[&space_id].rooms.contains_key(&room_id),
            "Node B must have Room after history sync"
        );
        // Bob's invite must be reflected in Node B's SpaceState.
        assert!(
            node_b.spaces[&space_id].pending_invites.contains_key(&bob_id),
            "Node B must see Bob's pending invite"
        );
        // Node A must know Node B is now federated.
        assert!(
            node_a.spaces[&space_id].federation_nodes.contains(&node_b.node_id),
            "Node A must list Node B as federated"
        );

        // Node B's current tip after history sync is the federation_add event.
        let tip_after_sync = node_b.dag_tips(&space_id);
        assert_eq!(tip_after_sync.len(), 1);
        let fed_add_id = tip_after_sync[0].clone();

        // Alice also needs to be known on Node B for message validation.
        node_b.register_identity(make_record(&alice_key, &node_a.node_id)).unwrap();

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
        let bob_join_space_id = bob_join_space_ev.event_id.clone().unwrap();
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
        let bob_join_room_id = bob_join_room_ev.event_id.clone().unwrap();
        node_b.ingest_event(bob_join_room_ev.clone());
        node_a.ingest_event(bob_join_room_ev); // propagate to Node A

        // Bob must now be in the Space + Room on both nodes.
        assert!(node_a.spaces[&space_id].is_member(&bob_id), "Node A: Bob must be Space member");
        assert!(node_b.spaces[&space_id].is_member(&bob_id), "Node B: Bob must be Space member");
        assert!(
            node_a.spaces[&space_id].is_room_member(&bob_id, &room_id),
            "Node A: Bob must be Room member"
        );
        assert!(
            node_b.spaces[&space_id].is_room_member(&bob_id, &room_id),
            "Node B: Bob must be Room member"
        );

        // Bob must be known to Node A's identity registry for message validation.
        node_a.register_identity(make_record(&bob_key, &node_b.node_id)).unwrap();

        // Both nodes' tips are now bob_join_room_id.
        assert!(node_a.graphs[&space_id].is_tip(&bob_join_room_id));
        assert!(node_b.graphs[&space_id].is_tip(&bob_join_room_id));

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
        let hello_bob_id = hello_bob_ev.event_id.clone().unwrap();

        node_a.accept_message(&space_id, hello_bob_ev.clone()).unwrap();
        // Propagate to Node B.
        node_b.accept_message(&space_id, hello_bob_ev).unwrap();

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
        let hello_alice_id = hello_alice_ev.event_id.clone().unwrap();

        node_b.accept_message(&space_id, hello_alice_ev.clone()).unwrap();
        // Propagate to Node A.
        node_a.accept_message(&space_id, hello_alice_ev).unwrap();

        // ── Step 16: Both Nodes have both Events in their Room DAG ────────────
        assert!(
            node_a.stores[&space_id].contains(&hello_bob_id),
            "Node A must have Alice's message"
        );
        assert!(
            node_a.stores[&space_id].contains(&hello_alice_id),
            "Node A must have Bob's message"
        );
        assert!(
            node_b.stores[&space_id].contains(&hello_bob_id),
            "Node B must have Alice's message"
        );
        assert!(
            node_b.stores[&space_id].contains(&hello_alice_id),
            "Node B must have Bob's message"
        );

        // Signature integrity on both nodes for both messages.
        for (label, store) in [("Node A", &node_a.stores[&space_id]), ("Node B", &node_b.stores[&space_id])] {
            let msg_a = store.get(&hello_bob_id).unwrap();
            let msg_b = store.get(&hello_alice_id).unwrap();
            assert!(verify_event_signature(msg_a), "{label}: Alice's message signature must be valid");
            assert!(verify_event_signature(msg_b), "{label}: Bob's message signature must be valid");
        }

        // ── Step 17: Both clients can display the conversation ────────────────
        let alice_msg_on_b = node_b.stores[&space_id].get(&hello_bob_id).unwrap();
        assert_eq!(alice_msg_on_b.content["text"].as_str().unwrap(), "Hello Bob");

        let bob_msg_on_a = node_a.stores[&space_id].get(&hello_alice_id).unwrap();
        assert_eq!(bob_msg_on_a.content["text"].as_str().unwrap(), "Hello Alice");

        // Final DAG tip on both nodes is hello_alice_id (linear chain).
        assert!(node_a.graphs[&space_id].is_tip(&hello_alice_id), "Node A tip must be hello_alice");
        assert!(node_b.graphs[&space_id].is_tip(&hello_alice_id), "Node B tip must be hello_alice");
    }
}
