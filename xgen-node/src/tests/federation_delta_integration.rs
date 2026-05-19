// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// F-1a federation handshake tip-exchange delta-delivery integration tests
// (Phase 3 — runbook `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.3 +
// §3.3.1 Locks 1-7).
//
// Three scenarios per the runbook §3.3 Definition of Done:
//
// 1. `brand_new_relationship_full_history_delivered_session_stays_open` —
//    initiator's `tips` is empty, receiver has events for the shared Space.
//    Verifies full-history delivery, state.federation_add appended per the
//    a-i symmetry rule (§3.3.1 Lock 2), SyncComplete terminator, and
//    session-stays-open invariant.
//
// 2. `reconnect_with_existing_tip_small_delta_delivered` — initiator's
//    `tips[S]` is populated (mid-history); receiver streams only events
//    after the cursor, with NO state.federation_add (peer's tip is present,
//    so a-i symmetry rule does not fire).
//
// 3. `pre_f1a_peer_compatibility_explicit_empty_tips_full_history` —
//    integration-level back-compat coverage. The wire-layer half (a Hello
//    JSON missing the `tips` field deserialises with empty BTreeMap per
//    `#[serde(default)]`) is locked by
//    `xgen_core::federation::handshake::tests::hello_without_tips_field_deserialises_with_empty_map`.
//    This integration test asserts that the post-deserialisation behaviour
//    (initiator carrying an explicitly-empty BTreeMap) produces full-history
//    delivery — i.e., a pre-F-1a peer that omits `tips` entirely receives
//    the same outcome as a brand-new F-1a-aware joiner.

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use chrono::{SecondsFormat, Utc};
    use serde_json::json;
    use tempfile::tempdir;
    use tokio::sync::Mutex;

    use crate::{
        crypto::encoding,
        federation::handshake::{run_initiating, run_receiving},
        federation_session::stream_federation_delta,
        identity::{keypair, registry::IdentityRecord},
        node::runtime::NodeRuntime,
        space::state::{
            build_room_create_event, build_space_create_event, sign_event,
        },
        transport::{client, connection::Inbound, server::Server},
        wire::types::{Event, EventType, FederationCapabilities, TransportMessage},
    };

    fn now() -> String {
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
            identity_id: pubkey_uri(key),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-19T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: home_node.to_string(),
            update_version: 0,
        }
    }

    /// Build a NodeRuntime hosting a Space with `n_messages` text events on
    /// a single linear chain (space_create → room_create → msg0 → msg1 → ...).
    /// Returns the runtime, the Node's signing key, the space_id, and the
    /// ordered event_ids in topological order — useful for asserting per-event
    /// delivery in delta tests.
    fn build_node_with_space_history(
        n_messages: usize,
    ) -> (
        NodeRuntime,
        ed25519_dalek::SigningKey,
        String,
        Vec<String>,
    ) {
        let node_key = keypair::generate();
        let mut node = NodeRuntime::new(node_key.clone());
        let owner_key = keypair::generate();
        let owner_id = pubkey_uri(&owner_key);
        node.register_identity(make_record(&owner_key, &node.node_id))
            .unwrap();

        let space_ev = sign_event(
            build_space_create_event(&owner_key, "test-space", None, 1, &node.node_id),
            &owner_key,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        let mut ordered = vec![space_id.clone()];
        node.ingest_event(space_ev);

        let room_ev = sign_event(
            build_room_create_event(&owner_key, &space_id, "general", None),
            &owner_key,
        );
        let room_id = room_ev.event_id.clone().unwrap();
        ordered.push(room_id.clone());
        node.ingest_event(room_ev);

        let mut prev = vec![room_id.clone()];
        for i in 0..n_messages {
            let msg_ev = sign_event(
                Event::new(
                    EventType::MessageText,
                    owner_id.clone(),
                    room_id.clone(),
                    space_id.clone(),
                    prev.clone(),
                    now(),
                    json!({ "text": format!("msg{}", i) }),
                ),
                &owner_key,
            );
            let msg_id = msg_ev.event_id.clone().unwrap();
            ordered.push(msg_id.clone());
            prev = vec![msg_id];
            node.ingest_event(msg_ev);
        }

        (node, node_key, space_id, ordered)
    }

    /// F-1a Phase 3 — brand-new federation relationship: peer's `tips` map
    /// is empty for the shared Space, receiver has events → full history is
    /// delivered, state.federation_add appended per the a-i symmetry rule
    /// (runbook §3.3.1 Lock 2), SyncComplete terminates, session stays open.
    ///
    /// Session-stays-open verification uses a deterministic exit-reason
    /// pattern: the server task's drain loop reports which match arm it
    /// exited through. A timeout-based "did recv() block?" assertion was
    /// initially used but proved flaky under workspace-concurrent test
    /// scheduling — the deterministic pattern locks the invariant without
    /// timing races.
    ///
    /// `#[serial_test::serial]` (Phase 9 Commit 2 — task file §3 Lock Q3
    /// option (i)) serialises the three federation_delta_integration tests
    /// among themselves so concurrent `127.0.0.1:0` binds + WS framing inside
    /// the same test binary don't race under `cargo test --workspace`. The
    /// race was observed at ~10 % rate in full workspace runs (CLAUDE.md
    /// known-test-flake state); locking the three tests is the minimal fix.
    #[tokio::test]
    #[serial_test::serial]
    async fn brand_new_relationship_full_history_delivered_session_stays_open() {
        let (node_a, node_a_key, space_id, ordered) = build_node_with_space_history(3);
        let expected_pre_federation_count = ordered.len(); // 5 = space + room + 3 msgs

        let runtime_a = Arc::new(Mutex::new(node_a));
        let spaces_dir = tempdir().unwrap();

        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = server.local_addr();

        let space_id_task = space_id.clone();
        let spaces_dir_path = spaces_dir.path().to_path_buf();
        let runtime_a_task = runtime_a.clone();

        let server_task: tokio::task::JoinHandle<&'static str> = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            conn.server_authenticate().await.unwrap();

            // F-1a: receiver-side tips — Node A's tip for the Space.
            let our_tips = {
                let rt = runtime_a_task.lock().await;
                let mut m = BTreeMap::new();
                if let Some(t) = rt.dag_tips(&space_id_task).into_iter().min() {
                    m.insert(space_id_task.clone(), t);
                }
                m
            };

            let session = run_receiving(
                &mut conn,
                &node_a_key,
                FederationCapabilities::default(),
                our_tips,
            )
            .await
            .unwrap();

            // Drive the production delta-stream helper.
            stream_federation_delta(
                &mut conn,
                &runtime_a_task,
                &session.shared_spaces,
                &session.peer_tips,
                &session.peer_node_id,
                &session.session_id,
                &session.negotiated_version,
                &session.negotiated_serialisation,
                &node_a_key,
                &spaces_dir_path,
            )
            .await
            .unwrap();

            // Steady-state: drain inbound until the client side closes.
            // Returns which match arm we exited via — the test asserts this
            // is "goodbye" (= client sent an explicit goodbye while the
            // server's drain loop was still running, i.e. session was alive
            // long enough to observe it).
            loop {
                match conn.recv().await {
                    Ok(Inbound::Transport(TransportMessage::Goodbye { .. })) => return "goodbye",
                    Ok(Inbound::Closed) => return "closed",
                    Err(_) => return "err",
                    _ => {}
                }
            }
        });

        let node_b_key = keypair::generate();
        let mut client_conn = client::connect(addr).await.unwrap();
        client_conn.client_authenticate(&node_b_key).await.unwrap();

        let _client_session = run_initiating(
            &mut client_conn,
            &node_b_key,
            FederationCapabilities::default(),
            vec![space_id.clone()],
            BTreeMap::new(),
            None,
        )
        .await
        .unwrap();

        let mut received: Vec<Event> = vec![];
        loop {
            match client_conn.recv().await.unwrap() {
                Inbound::Event(ev) => received.push(ev),
                Inbound::Transport(TransportMessage::SyncComplete { .. }) => break,
                other => panic!("unexpected during delta drain: {:?}", other),
            }
        }

        // Full history + state.federation_add (a-i symmetry rule fired).
        assert_eq!(
            received.len(),
            expected_pre_federation_count + 1,
            "expected {} history events + 1 state.federation_add = {}; received {}",
            expected_pre_federation_count,
            expected_pre_federation_count + 1,
            received.len()
        );

        // Last event is state.federation_add per a-i symmetry rule.
        let last = received.last().unwrap();
        assert_eq!(
            last.event_type,
            EventType::StateFederationAdd,
            "a-i symmetry rule: last event must be state.federation_add"
        );

        // First N events are the original history in SOME valid topological
        // order. The DAG has two roots (state.space_create has empty
        // prev_events; state.room_create also has empty prev_events per
        // build_room_create_event), so the topological order is not unique
        // across the two roots. Assert (a) the set of received history
        // event_ids equals the set of expected event_ids, and (b) each
        // received event's prev_events all appear at a strictly earlier
        // index in received — the actual topological invariant.
        use std::collections::HashSet;
        let received_history: &[Event] = &received[..ordered.len()];
        let received_ids: HashSet<&str> = received_history
            .iter()
            .map(|e| e.event_id.as_deref().unwrap_or(""))
            .collect();
        let expected_ids: HashSet<&str> = ordered.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            received_ids, expected_ids,
            "history event set mismatch (received != expected)"
        );

        // Topological invariant: every prev_event of received[i] appears in
        // received[..i]. Holds for any valid topo order regardless of root
        // tie-breaking.
        let mut seen: HashSet<&str> = HashSet::new();
        for (i, ev) in received_history.iter().enumerate() {
            for prev in &ev.prev_events {
                assert!(
                    seen.contains(prev.as_str()),
                    "topological invariant violated: received[{}] (event_id {:?}) references prev {:?} which was not yet emitted",
                    i,
                    ev.event_id,
                    prev
                );
            }
            if let Some(id) = &ev.event_id {
                seen.insert(id.as_str());
            }
        }

        // Session-stays-open invariant — send a goodbye and verify the
        // server's drain loop observes it (rather than the server having
        // exited via Closed/Err before the goodbye arrived).
        client_conn
            .goodbye("test_session_stays_open")
            .await
            .unwrap();
        let exit_reason = server_task.await.unwrap();
        assert_eq!(
            exit_reason, "goodbye",
            "session must stay open after SyncComplete; server exited via {:?} (expected \"goodbye\")",
            exit_reason
        );
    }

    /// F-1a Phase 3 — reconnect with existing tip: peer's `tips[S]` points
    /// at a mid-history event; receiver streams only the events after that
    /// cursor and does NOT build state.federation_add (a-i symmetry rule
    /// does not fire — peer's tip is present, so the relationship is not
    /// brand-new for this Space).
    ///
    /// `#[serial_test::serial]` per Phase 9 Commit 2 — see the sibling
    /// `brand_new_relationship_*` test's doc comment.
    #[tokio::test]
    #[serial_test::serial]
    async fn reconnect_with_existing_tip_small_delta_delivered() {
        // 5 message events in the ordered chain. Topological order:
        // ordered = [space_create, room_create, msg0, msg1, msg2, msg3, msg4].
        let (node_a, node_a_key, space_id, ordered) = build_node_with_space_history(5);
        assert_eq!(ordered.len(), 7);

        // Pretend Node B already has events up through msg1 (index 3) →
        // delta should be [msg2, msg3, msg4] = ordered[4..7] = 3 events.
        let peer_tip_event_id = ordered[3].clone();
        let expected_delta_count = ordered.len() - 4;

        let runtime_a = Arc::new(Mutex::new(node_a));
        let spaces_dir = tempdir().unwrap();

        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = server.local_addr();

        let space_id_task = space_id.clone();
        let spaces_dir_path = spaces_dir.path().to_path_buf();
        let runtime_a_task = runtime_a.clone();

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            conn.server_authenticate().await.unwrap();

            let our_tips = {
                let rt = runtime_a_task.lock().await;
                let mut m = BTreeMap::new();
                if let Some(t) = rt.dag_tips(&space_id_task).into_iter().min() {
                    m.insert(space_id_task.clone(), t);
                }
                m
            };

            let session = run_receiving(
                &mut conn,
                &node_a_key,
                FederationCapabilities::default(),
                our_tips,
            )
            .await
            .unwrap();

            stream_federation_delta(
                &mut conn,
                &runtime_a_task,
                &session.shared_spaces,
                &session.peer_tips,
                &session.peer_node_id,
                &session.session_id,
                &session.negotiated_version,
                &session.negotiated_serialisation,
                &node_a_key,
                &spaces_dir_path,
            )
            .await
            .unwrap();

            loop {
                match conn.recv().await {
                    Ok(Inbound::Transport(TransportMessage::Goodbye { .. }))
                    | Ok(Inbound::Closed)
                    | Err(_) => break,
                    _ => {}
                }
            }
        });

        let node_b_key = keypair::generate();
        let mut client_conn = client::connect(addr).await.unwrap();
        client_conn.client_authenticate(&node_b_key).await.unwrap();

        // Client carries its known tip for this Space.
        let mut peer_tips = BTreeMap::new();
        peer_tips.insert(space_id.clone(), peer_tip_event_id.clone());

        let _client_session = run_initiating(
            &mut client_conn,
            &node_b_key,
            FederationCapabilities::default(),
            vec![space_id.clone()],
            peer_tips,
            None,
        )
        .await
        .unwrap();

        let mut received: Vec<Event> = vec![];
        loop {
            match client_conn.recv().await.unwrap() {
                Inbound::Event(ev) => received.push(ev),
                Inbound::Transport(TransportMessage::SyncComplete { .. }) => break,
                other => panic!("unexpected during delta drain: {:?}", other),
            }
        }

        // Delta is just the events after the peer's tip; NO federation_add
        // because the peer's tip is present (a-i symmetry rule does not fire).
        assert_eq!(
            received.len(),
            expected_delta_count,
            "expected {} delta events (after peer's tip, no federation_add); received {}",
            expected_delta_count,
            received.len()
        );

        for (i, expected_id) in ordered[4..].iter().enumerate() {
            assert_eq!(
                received[i].event_id.as_deref(),
                Some(expected_id.as_str()),
                "delta event {} mismatch (expected ordered[{}])",
                i,
                4 + i
            );
        }

        // No state.federation_add in the delta.
        for ev in &received {
            assert_ne!(
                ev.event_type,
                EventType::StateFederationAdd,
                "reconnect path must NOT produce state.federation_add (a-i symmetry rule)"
            );
        }

        let _ = client_conn.goodbye("test_done").await;
        let _ = server_task.await;
    }

    /// F-1a Phase 3 — pre-F-1a peer compatibility: a peer omitting the
    /// `tips` field entirely on the wire deserialises into an empty BTreeMap
    /// (locked by the wire-layer unit test in
    /// `xgen_core::federation::handshake::tests::hello_without_tips_field_deserialises_with_empty_map`).
    /// This integration test asserts the integration-level half: an
    /// initiator carrying an explicitly-empty BTreeMap (the post-
    /// deserialisation shape of a pre-F-1a peer's Hello) receives full
    /// history per Space — i.e., the receiver does not require the tips
    /// field downstream of deserialisation.
    ///
    /// `#[serial_test::serial]` per Phase 9 Commit 2 — see the sibling
    /// `brand_new_relationship_*` test's doc comment.
    #[tokio::test]
    #[serial_test::serial]
    async fn pre_f1a_peer_compatibility_explicit_empty_tips_full_history() {
        let (node_a, node_a_key, space_id, ordered) = build_node_with_space_history(2);
        let expected_pre_federation_count = ordered.len(); // 4 = space + room + 2 msgs

        let runtime_a = Arc::new(Mutex::new(node_a));
        let spaces_dir = tempdir().unwrap();

        let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = server.local_addr();

        let space_id_task = space_id.clone();
        let spaces_dir_path = spaces_dir.path().to_path_buf();
        let runtime_a_task = runtime_a.clone();

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();
            conn.server_authenticate().await.unwrap();

            let our_tips = {
                let rt = runtime_a_task.lock().await;
                let mut m = BTreeMap::new();
                if let Some(t) = rt.dag_tips(&space_id_task).into_iter().min() {
                    m.insert(space_id_task.clone(), t);
                }
                m
            };

            let session = run_receiving(
                &mut conn,
                &node_a_key,
                FederationCapabilities::default(),
                our_tips,
            )
            .await
            .unwrap();

            stream_federation_delta(
                &mut conn,
                &runtime_a_task,
                &session.shared_spaces,
                &session.peer_tips,
                &session.peer_node_id,
                &session.session_id,
                &session.negotiated_version,
                &session.negotiated_serialisation,
                &node_a_key,
                &spaces_dir_path,
            )
            .await
            .unwrap();

            loop {
                match conn.recv().await {
                    Ok(Inbound::Transport(TransportMessage::Goodbye { .. }))
                    | Ok(Inbound::Closed)
                    | Err(_) => break,
                    _ => {}
                }
            }
        });

        let node_b_key = keypair::generate();
        let mut client_conn = client::connect(addr).await.unwrap();
        client_conn.client_authenticate(&node_b_key).await.unwrap();

        // Explicit empty BTreeMap — same post-deserialisation shape as a
        // pre-F-1a peer's Hello with no `tips` field on the wire.
        let _client_session = run_initiating(
            &mut client_conn,
            &node_b_key,
            FederationCapabilities::default(),
            vec![space_id.clone()],
            BTreeMap::new(),
            None,
        )
        .await
        .unwrap();

        let mut received: Vec<Event> = vec![];
        loop {
            match client_conn.recv().await.unwrap() {
                Inbound::Event(ev) => received.push(ev),
                Inbound::Transport(TransportMessage::SyncComplete { .. }) => break,
                other => panic!("unexpected during delta drain: {:?}", other),
            }
        }

        // Full history + state.federation_add — same outcome as a brand-new
        // F-1a-aware joiner. Receiver tolerates the empty-tips case
        // identically whether produced by post-deserialisation back-compat
        // or by explicit F-1a-aware semantics.
        assert_eq!(
            received.len(),
            expected_pre_federation_count + 1,
            "pre-F-1a back-compat path must deliver full history + state.federation_add"
        );
        assert_eq!(
            received.last().unwrap().event_type,
            EventType::StateFederationAdd
        );

        let _ = client_conn.goodbye("test_done").await;
        let _ = server_task.await;
    }
}
