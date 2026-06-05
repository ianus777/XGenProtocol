// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M8 — Wave 2 / C4 — S4 durability / replay gate (G-DURABILITY). Runbook
//! `tasks/M8_MULTIPARTY_IMPL.md` §4 C4; design `tasks/M8_MULTIPARTY_DESIGN.md`
//! §3 (S4 row) + the G-DURABILITY cross-cutting gate (§3 "folded where a restart
//! is natural").
//!
//! **The gate.** A Node restarts mid-run → replays its on-disk EventStore →
//! resolved state comes back **byte-identical**, with **zero orphans** (every
//! persisted event re-loads). This exercises the production durability path:
//! `ingest`/`submit_locally` persist each event via `persist_event` (per-Space
//! file-backed store, the vanilla engine — sqlite is a production-scale backing,
//! not required for the replay path), `shutdown_keep_data` preserves the on-disk
//! tree, and `spawn_in_process_node_with_state` reloads it via
//! `replay_spaces_from_dir` + registry loads (the same helpers `run_node` uses at
//! startup).
//!
//! **N×N convergence note.** The multi-actor / N-event convergence dimension of
//! S4 is already proven by the C2 S2 convergence suite (`m8_s2_convergence.rs` —
//! multi-actor conflicts across Nodes, every permutation) and the C3 migration
//! convergence (`m8_s3_federation.rs`). C4's genuinely-new contribution is the
//! **restart-replay** dimension proven here; `MULTIPARTY_S4_findings.md` records
//! the composite.
//!
//! **S5 (client rebind) — BLOCKED (M8-D4), findings-only.** `re_registration` is
//! not exposed on `xgen-client register` and `identity.home_changed` has no
//! EventType — so identity-portability rebind is not runnable on B. Recorded in
//! `MULTIPARTY_S5_findings.md` as a BLOCKED scenario + M9 input (the S0 file itself
//! flagged this capability gate). No surface is built here.

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
    use xgen_common::xgid::{EventXgid, Xgid};

    use crate::tests::phase9_harness::{
        pubkey_uri, spawn_in_process_node, spawn_in_process_node_with_state,
    };

    fn eid(ev: &Event) -> String {
        ev.event_id.as_ref().unwrap().as_str().to_string()
    }
    fn edx(s: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(s.to_string()))
    }

    /// G-DURABILITY: build a Space (create + room + member + messages) on a Node,
    /// snapshot its resolved state, `shutdown_keep_data`, respawn from the
    /// preserved on-disk tree, and assert the replayed state is byte-identical and
    /// every persisted event re-loaded (zero orphans). This is the restart-replay
    /// resync the S4 gate requires, on the production `persist_event` /
    /// `replay_spaces_from_dir` path.
    #[tokio::test]
    async fn s4_node_restart_replays_byte_identical_state_no_orphans() {
        let node = spawn_in_process_node().await;

        let alice = keypair::generate();
        let alice_id = pubkey_uri(&alice);

        let create = sign_event(
            build_space_create_event(&alice, "S4-durable", None, 1, &node.node_id, None, false),
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
        let jid = eid(&join);

        // A couple of messages (DAG-persisted but not state-mutating) so the
        // no-orphan replay check covers message events too.
        let mut msg1 = Event::new(
            EventType::MessageText,
            crate::tests::phase9_harness::idx(&alice_id),
            crate::tests::phase9_harness::rdx(&rid),
            crate::tests::phase9_harness::sdx(&sid),
            vec![edx(&jid)],
            crate::tests::phase9_harness::now_rfc(),
            json!({ "text": "durable-msg-1" }),
        );
        msg1 = sign_event(msg1, &alice);
        let m1 = eid(&msg1);
        let mut msg2 = Event::new(
            EventType::MessageText,
            crate::tests::phase9_harness::idx(&alice_id),
            crate::tests::phase9_harness::rdx(&rid),
            crate::tests::phase9_harness::sdx(&sid),
            vec![edx(&m1)],
            crate::tests::phase9_harness::now_rfc(),
            json!({ "text": "durable-msg-2" }),
        );
        msg2 = sign_event(msg2, &alice);
        let m2 = eid(&msg2);

        let all_events = [create, room, invite, join, msg1, msg2];
        for ev in &all_events {
            node.ingest(ev.clone()).await; // persists each via persist_event
        }

        // Pre-restart snapshot.
        let pre = node.space_state(&sid).await.expect("pre-restart state");
        let all_ids: Vec<String> = all_events.iter().map(eid).collect();

        // Restart: preserve on-disk tree, respawn, replay.
        let saved = node.shutdown_keep_data().await;
        let node = spawn_in_process_node_with_state(saved).await;

        // Replayed resolved state is byte-identical.
        let post = node.space_state(&sid).await.expect("post-restart state");
        assert_eq!(pre, post, "restart-replay must reproduce the exact SpaceState");

        // Zero orphans: every persisted event re-loaded into the store.
        for id in &all_ids {
            assert!(
                node.has_event(&sid, id).await,
                "event {id} must survive restart-replay (no orphan / no loss)"
            );
        }
        // Messages specifically survive (the m1/m2 DAG entries).
        assert!(node.has_event(&sid, &m1).await && node.has_event(&sid, &m2).await);

        node.shutdown().await;
    }
}
