// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Arc F (PG-11) — two-node Space Migration end-to-end (C2 DoD).
//!
//! Exercises the full driver over the real `phase9_harness` two-Node WS
//! transport: `run_source_migration` (source side, driven inline) ↔
//! `handle_migration_incoming` (destination side, reached from the harness's
//! production `handle_connection` first-message branch).
//!
//! Covered:
//!   * happy path — propose → accept → batch(+tail) → transfer_complete →
//!     verify → cutover → `home_node` flipped on BOTH nodes;
//!   * retention (AF-D5) — the source keeps its store after cutover;
//!   * AF-D2 self-protection — a post-cutover stale-source migrate is rejected;
//!   * dormant admission (AF-D6 / 6002) — the destination rejects a Space it
//!     already hosts.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;

    use crate::tests::phase9_harness::{
        edx, event_id_str, idx, ndx, now_rfc, pubkey_uri, rdx, sdx, spawn_in_process_node,
        InProcessNode,
    };
    use crate::{
        identity::keypair,
        migration_driver::run_source_migration,
        node::runtime::{DispatchOutcome, EventOrigin},
        space::state::{build_room_create_event, build_space_create_event, sign_event},
        wire::types::{Event, EventType},
    };

    /// Seed a Space (homed on `node`) with a Room, Alice's invite + join.
    /// Returns `(space_id, room_id)`. Alice must already be registered on `node`.
    async fn seed_space(
        node: &InProcessNode,
        alice_key: &ed25519_dalek::SigningKey,
        alice_id: &str,
    ) -> (String, String) {
        let space_ev = sign_event(
            build_space_create_event(alice_key, "migrate-space", None, 1, &node.node_id, None),
            alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        let room_ev =
            sign_event(build_room_create_event(alice_key, &space_id, "general", None), alice_key);
        let room_id = event_id_str(&room_ev);
        node.ingest(room_ev).await;

        let invite_ev = sign_event(
            Event::new(
                EventType::MembershipInvite,
                idx(alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&space_id), edx(&room_id)],
                now_rfc(),
                json!({ "target_identity": alice_id, "role": "member" }),
            ),
            alice_key,
        );
        let invite_id = event_id_str(&invite_ev);
        node.ingest(invite_ev).await;

        let join_ev = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&invite_id)],
                now_rfc(),
                json!({}),
            ),
            alice_key,
        );
        node.ingest(join_ev).await;

        (space_id, room_id)
    }

    /// Poll `node`'s `home_node` for `space_id` until it equals `want` or times out.
    async fn wait_for_home(node: &InProcessNode, space_id: &str, want: &str, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        loop {
            if let Some(st) = node.space_state(space_id).await {
                if st.home_node.as_str() == want {
                    return true;
                }
            }
            if start.elapsed() >= timeout {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn two_node_migration_flips_home_node_on_both() {
        let source = spawn_in_process_node().await;
        let dest = spawn_in_process_node().await;

        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        source.register_identity(&alice_key).await;

        let (space_id, _room_id) = seed_space(&source, &alice_key, &alice_id).await;
        // Pre-condition: source is the home Node.
        assert_eq!(
            source.space_state(&space_id).await.unwrap().home_node.as_str(),
            source.node_id
        );

        // ── Drive the source-side migration to `dest` ──
        run_source_migration(
            source.runtime.clone(),
            source.keypair.clone(),
            ndx(&source.node_id),
            source.spaces_dir.clone(),
            source.federation_peer_senders.clone(),
            source.federation_policy.clone(),
            space_id.clone(),
            dest.node_id.clone(),
            dest.endpoint.clone(),
        )
        .await;

        // ── home_node flipped to the destination on BOTH nodes (AF-D1) ──
        assert!(
            wait_for_home(&dest, &space_id, &dest.node_id, Duration::from_secs(10)).await,
            "destination must host the Space as home after cutover"
        );
        assert_eq!(
            source.space_state(&space_id).await.unwrap().home_node.as_str(),
            dest.node_id,
            "source home_node must flip to the destination after cutover"
        );

        // ── Destination has the full migrated state ──
        let dest_state = dest.space_state(&space_id).await.unwrap();
        assert_eq!(dest_state.members.len(), 1, "Alice's membership migrated");
        assert!(dest_state.rooms.len() == 1, "the Room migrated");

        // ── Retention (AF-D5): the SOURCE still holds the Space + its events ──
        assert!(
            source.has_space(&space_id).await,
            "source retains the Space store after migration (no auto-teardown)"
        );
        assert!(
            source.has_event(&space_id, &space_id).await,
            "source retains the create event (retention, AF-D5)"
        );

        // ── AF-D2: a post-cutover stale-source migrate is rejected ──
        // The source is no longer home_node, so a fresh source-signed migrate
        // fails the authority gate (wire 6007).
        let stale = xgen_core::migration::state_machine::build_space_migrate_event(
            &source.keypair,
            &space_id,
            "xgen://pubkey/ed25519:THIRDNODE",
            "wss://third/x",
            source.dag_tips(&space_id).await,
            &now_rfc(),
        );
        let outcome = {
            let mut rt = source.runtime.lock().await;
            rt.dispatch_event(stale, EventOrigin::LocallySubmitted, None)
        };
        assert!(
            matches!(outcome, DispatchOutcome::Rejected(_)),
            "post-cutover stale-source migrate must be rejected (AF-D2); got {outcome:?}"
        );

        source.shutdown().await;
        dest.shutdown().await;
    }

    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn destination_rejects_already_hosted_space() {
        let source = spawn_in_process_node().await;
        let dest = spawn_in_process_node().await;

        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        source.register_identity(&alice_key).await;
        dest.register_identity(&alice_key).await;

        let (space_id, _room_id) = seed_space(&source, &alice_key, &alice_id).await;

        // The destination ALREADY hosts the same Space — ingest the source's own
        // (content-id-derived) create event so the ids collide exactly.
        dest.ingest(source_create_event(&source, &space_id).await).await;
        assert!(dest.has_space(&space_id).await, "dest now hosts the Space");

        // Migrate source → dest: dest must reject (already hosting), so the
        // source's home_node stays the source (no cutover).
        run_source_migration(
            source.runtime.clone(),
            source.keypair.clone(),
            ndx(&source.node_id),
            source.spaces_dir.clone(),
            source.federation_peer_senders.clone(),
            source.federation_policy.clone(),
            space_id.clone(),
            dest.node_id.clone(),
            dest.endpoint.clone(),
        )
        .await;

        assert_eq!(
            source.space_state(&space_id).await.unwrap().home_node.as_str(),
            source.node_id,
            "rejected migration leaves the source as home Node (no cutover)"
        );

        source.shutdown().await;
        dest.shutdown().await;
    }

    /// Read the source's `state.space_create` event back out of its store so the
    /// destination can host the identical Space (same content-derived id).
    async fn source_create_event(source: &InProcessNode, space_id: &str) -> Event {
        let rt = source.runtime.lock().await;
        let sx = xgen_common::xgid::SpaceXgid::from_xgid(xgen_common::xgid::Xgid::new(
            space_id.to_string(),
        ));
        rt.stores
            .get(&sx)
            .and_then(|s| s.range(0).ok())
            .unwrap_or_default()
            .into_iter()
            .find(|e| e.event_type == EventType::StateSpaceCreate)
            .expect("source store holds the create event")
    }
}
