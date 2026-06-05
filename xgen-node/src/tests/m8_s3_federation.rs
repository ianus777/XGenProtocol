// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M8 — Wave 2 / C3 — S3 federation topology, jurisdiction, migration. Runbook
//! `tasks/M8_MULTIPARTY_IMPL.md` §4 C3; design `tasks/M8_MULTIPARTY_DESIGN.md`
//! §3 (S3 row).
//!
//! **What is referenced vs new.** Three of the four S3 aspects already have
//! proven workspace coverage — this file adds only the genuinely-new piece
//! (multiparty **migration** convergence) and `MULTIPARTY_S3_findings.md` records
//! the references + the headline finding. Specifically:
//!   - **3-Node multi-node delivery** (A's event reaches both B and C directly):
//!     `phase9_three_node_anti_transitivity.rs` assertion #2.
//!   - **Anti-transitivity / F-5** (a chain B↔C-unfederated does NOT leak A's
//!     event B→C): same test, assertions #1 + #3.
//!   - **Jurisdiction reject** (Arc G PG-04 — a cross-jurisdiction Space's events
//!     are dropped at a peer per `allowed_jurisdictions`):
//!     `federation_policy_enforcement.rs::inbound_jurisdiction_drops_excluded_space_event`.
//!
//! **Headline S3 finding (M8-D4 → M9 input).** The S3/S0 premise of *transitive*
//! propagation across **non-adjacent** Nodes (a chain A↔B↔C delivering A→C via B)
//! is **not** the built model: federation is **anti-transitive** (the F-5 guard at
//! `federation_session.rs` returns immediately for `EventOrigin::ReceivedViaFederation`,
//! so a Node never re-forwards a federation-received event to its other peers). A
//! multi-Node Space therefore relies on **full-mesh** federation + anti-transitive
//! delivery (each hosting Node pushes directly to every peer; no Node duplicates by
//! re-forwarding), NOT chain-with-transitive-forward. This is a clarification the
//! spec §3.2 "forward on accept" gap (S0 §"S3 spec gap") should settle in M9 — a
//! surfaced-weakness *success* per M8-D4, recorded, not redesigned in-arc.
//!
//! **This file — migration multiparty convergence (Arc F PG-11, new).** The
//! `state.space_migrate` cutover applier flips `SpaceState.home_node` source→dest
//! under the AF-D2 authority gate (`sender == home_node`) and is reachable via the
//! plain `ingest` → `derive_resolved` → `apply_event` path (no migration-driver
//! protocol needed for the applier). The test proves the flip + full-state
//! convergence across three independent Nodes: after all ingest the cutover event,
//! every Node's resolved `SpaceState` is byte-identical, `home_node` is the
//! destination on all three, and the migrated Space's members/rooms are intact.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        identity::keypair,
        space::state::{
            build_membership_event, build_room_create_event, build_space_create_event,
            sign_event,
        },
        wire::types::{Event, EventType},
    };
    use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, Xgid};

    use crate::tests::phase9_harness::{now_rfc, pubkey_uri, spawn_in_process_node};

    fn eid(ev: &Event) -> String {
        ev.event_id.as_ref().unwrap().as_str().to_string()
    }
    fn edx(s: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn idx(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }

    /// Arc F (PG-11) multiparty migration convergence. Three Nodes host the same
    /// Space (home = Node A); a Node-A-authored `state.space_migrate` flips
    /// `home_node` to Node B. After every Node ingests the cutover, all three
    /// resolved `SpaceState`s are byte-identical, `home_node == B` on all three,
    /// and the migrated members/rooms survive. The applier is reached via plain
    /// `ingest` (no driver protocol); the AF-D2 authority gate (`sender ==
    /// home_node`) is satisfied because A's Node keypair signs while A is still
    /// the home Node.
    #[tokio::test]
    async fn s3_migration_flips_home_node_and_converges_across_three_nodes() {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;
        let node_c = spawn_in_process_node().await;

        let alice = keypair::generate();
        let alice_id = pubkey_uri(&alice);

        // Space created on A (home_node = A). Room + alice membership so the
        // migration carries real state, not an empty shell.
        let create = sign_event(
            build_space_create_event(&alice, "S3-migrate", None, 1, &node_a.node_id, None, false),
            &alice,
        );
        let sid = eid(&create);

        let room = sign_event(build_room_create_event(&alice, &sid, "general", None), &alice);
        let rid = eid(&room);

        let mut invite = build_membership_event(
            &alice,
            &sid,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": alice_id, "role": "member" }),
        );
        invite.prev_events = vec![edx(&sid), edx(&rid)];
        let invite = sign_event(invite, &alice);
        let iid = eid(&invite);

        let mut join = build_membership_event(&alice, &sid, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![edx(&iid)];
        let join = sign_event(join, &alice);

        // All three Nodes ingest the pre-migration state, in order.
        let prefix = [create.clone(), room.clone(), invite, join];
        for ev in &prefix {
            node_a.ingest(ev.clone()).await;
            node_b.ingest(ev.clone()).await;
            node_c.ingest(ev.clone()).await;
        }

        // Pre-migration: home_node == A on all three.
        let a_home = node_a.space_state(&sid).await.unwrap().home_node;
        assert_eq!(a_home.as_str(), node_a.node_id, "pre-migration home is Node A");
        assert_eq!(
            node_b.space_state(&sid).await.unwrap().home_node.as_str(),
            node_a.node_id
        );
        assert_eq!(
            node_c.space_state(&sid).await.unwrap().home_node.as_str(),
            node_a.node_id
        );

        // Node A (the current home) authors the cutover to Node B. `prev_events`
        // are A's current tips; B and C share the identical DAG so the same event
        // applies on all three.
        let migrate = xgen_core::migration::state_machine::build_space_migrate_event(
            node_a.keypair.as_ref(),
            &sid,
            &node_b.node_id,
            &node_b.endpoint,
            node_a.dag_tips(&sid).await,
            &now_rfc(),
        );
        let migrate = sign_event(migrate, node_a.keypair.as_ref());

        for n in [&node_a, &node_b, &node_c] {
            n.ingest(migrate.clone()).await;
        }

        let state_a = node_a.space_state(&sid).await.unwrap();
        let state_b = node_b.space_state(&sid).await.unwrap();
        let state_c = node_c.space_state(&sid).await.unwrap();

        // home_node flipped to B on all three (AF-D2 cutover).
        let b_node = NodeXgid::from_xgid(Xgid::new(node_b.node_id.clone()));
        assert_eq!(state_a.home_node, b_node, "Node A view: home flipped to B");
        assert_eq!(state_b.home_node, b_node, "Node B view: home flipped to B");
        assert_eq!(state_c.home_node, b_node, "Node C view: home flipped to B");

        // Full-state convergence: every Node's resolved snapshot is byte-identical.
        assert_eq!(state_a, state_b, "A and B converge post-migration");
        assert_eq!(state_b, state_c, "B and C converge post-migration");

        // Migrated state intact: alice still a member, the Room survived.
        assert!(
            state_b.members.contains_key(&idx(&alice_id)),
            "alice's membership survives the migration"
        );
        assert_eq!(state_b.rooms.len(), 1, "the Room survives the migration");

        node_a.shutdown().await;
        node_b.shutdown().await;
        node_c.shutdown().await;
    }

    /// AF-D2 authority gate, multiparty: once `home_node` has flipped to B, a
    /// stale Node-A-signed re-migrate (A is no longer the home) is **rejected** by
    /// the applier on every Node — so the resolved state cannot be hijacked back.
    /// This is the convergence-preserving half of the authority gate
    /// (`apply_space_migrate` returns `PermissionDenied` when `sender !=
    /// home_node`; `derive_resolved` drops the rejected event's effect uniformly).
    #[tokio::test]
    async fn s3_stale_source_remigrate_rejected_on_all_nodes() {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        let alice = keypair::generate();
        let create = sign_event(
            build_space_create_event(&alice, "S3-migrate-auth", None, 1, &node_a.node_id, None, false),
            &alice,
        );
        let sid = eid(&create);
        node_a.ingest(create.clone()).await;
        node_b.ingest(create.clone()).await;

        // Cutover A → B (valid; A is home).
        let migrate1 = sign_event(
            xgen_core::migration::state_machine::build_space_migrate_event(
                node_a.keypair.as_ref(),
                &sid,
                &node_b.node_id,
                &node_b.endpoint,
                node_a.dag_tips(&sid).await,
                &now_rfc(),
            ),
            node_a.keypair.as_ref(),
        );
        for n in [&node_a, &node_b] {
            n.ingest(migrate1.clone()).await;
        }
        let b_node = NodeXgid::from_xgid(Xgid::new(node_b.node_id.clone()));
        assert_eq!(node_a.space_state(&sid).await.unwrap().home_node, b_node);

        // Stale re-migrate authored by A (no longer the home) → applier rejects;
        // home_node stays B on both Nodes (no hijack, convergence preserved).
        let migrate2 = sign_event(
            xgen_core::migration::state_machine::build_space_migrate_event(
                node_a.keypair.as_ref(),
                &sid,
                &node_a.node_id, // A tries to grab the Space back
                &node_a.endpoint,
                node_a.dag_tips(&sid).await,
                &now_rfc(),
            ),
            node_a.keypair.as_ref(),
        );
        for n in [&node_a, &node_b] {
            n.ingest(migrate2.clone()).await;
        }

        let state_a = node_a.space_state(&sid).await.unwrap();
        let state_b = node_b.space_state(&sid).await.unwrap();
        assert_eq!(state_a.home_node, b_node, "stale A re-migrate rejected on A — home stays B");
        assert_eq!(state_b.home_node, b_node, "stale A re-migrate rejected on B — home stays B");
        assert_eq!(state_a, state_b, "both Nodes still converge after the rejected re-migrate");

        node_a.shutdown().await;
        node_b.shutdown().await;
    }
}
