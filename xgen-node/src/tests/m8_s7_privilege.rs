// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M8 — Wave 3 / C6 — S7 privilege enforcement (Arc D). Runbook
//! `tasks/M8_MULTIPARTY_IMPL.md` §5 C6; design `tasks/M8_MULTIPARTY_DESIGN.md`
//! §3 (S7 row). **Multiparty behaviour only, NOT the auth-tier matrix (M8-A7):**
//! a synthetic Tier-1/Local-Node setup is sufficient; no Auth Module ref set is
//! built or required.
//!
//! Two enforcement decisions, each shown to be **observed by every member's Node**
//! (the multiparty angle):
//!   1. **Tier-gated join refusal (PG-13).** A Tier-1 joiner is refused entry to a
//!      Tier-2 Space (dispatch step-4 gate → `Rejected` carrying wire 3030
//!      `tier_mismatch`); the rejected join never enters the DAG, so the joiner is
//!      absent from the resolved membership on **every** Node — no event to
//!      disagree on.
//!   2. **Per-Room override (PG-12-min).** A `(Moderator, SendMessages, Deny)`
//!      override on `#announcements` blocks a Moderator's message
//!      (`check_permission` → `PermissionDenied`) on **every** Node — the override
//!      rides `state.room_update` (state-keyed, M8-convergent — the C2 Layer-4
//!      proof), so all Nodes resolve the identical override and reject identically.
//!
//! Path discipline (per the grounding): the **gated** events run through
//! `submit_locally` (→ `dispatch_event`, which carries the step-4 tier gate and
//! the `validate_event`/`check_permission` override layer); the **setup** events
//! (create/room/invite/join/room_update) are `ingest`ed (no gate needed — the test
//! is the authority).

#![cfg(test)]

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::tests::phase9_harness::{edx, event_id_str, idx, pubkey_uri, spawn_in_process_node};
    use crate::{
        identity::keypair,
        message::exchange::build_message_text_event,
        node::runtime::DispatchOutcome,
        space::{
            membership::{Effect, Role, RoomPermission},
            state::{
                build_membership_event, build_room_create_event, build_room_update_event,
                build_space_create_event, sign_event,
            },
        },
        wire::types::EventType,
    };

    /// PG-13 tier-gate, multiparty: a Tier-1 joiner is refused entry to a Tier-2
    /// Space on **every** Node (wire 3030), and is absent from the resolved
    /// membership everywhere (the rejected join never enters the DAG).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn s7_tier_gated_join_refused_on_all_nodes() {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        let alice = keypair::generate(); // owner
        let bob = keypair::generate(); // Tier-1 would-be joiner
        let bob_id = pubkey_uri(&bob);

        // Tier-2 Space (the slot contract a Tier-1 identity cannot satisfy).
        let create = sign_event(
            build_space_create_event(&alice, "S7-tier2", None, 2, &node_a.node_id, None, false),
            &alice,
        );
        let sid = event_id_str(&create);

        // alice (owner) invites bob — so the join is otherwise valid and reaches
        // the step-4 tier gate (not rejected earlier for lacking an invite).
        let mut invite = build_membership_event(
            &alice,
            &sid,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": bob_id, "role": "member" }),
        );
        invite.prev_events = vec![edx(&sid)];
        let invite = sign_event(invite, &alice);
        let iid = event_id_str(&invite);

        // bob's join (Tier-1 identity; trust_assertion None ⇒ tier 1).
        let mut join = build_membership_event(&bob, &sid, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![edx(&iid)];
        let join = sign_event(join, &bob);

        for n in [&node_a, &node_b] {
            n.register_identity(&alice).await;
            n.register_identity(&bob).await; // registered, but Tier 1
            n.ingest(create.clone()).await;
            n.ingest(invite.clone()).await;
            // The join is the gated event — run it through dispatch on each Node.
            let outcome = n.submit_locally(join.clone()).await;
            match outcome {
                DispatchOutcome::Rejected(msg) => {
                    let msg = msg.reason;
                    assert!(
                        msg.contains("3030") && msg.contains("tier_mismatch"),
                        "tier-gate must reject with wire 3030 tier_mismatch, got: {msg}"
                    );
                }
                other => panic!("expected Rejected(tier_mismatch), got {other:?}"),
            }
        }

        // Observed by all: bob is absent from the resolved membership on both Nodes.
        let bob_x = idx(&bob_id);
        assert!(
            !node_a.space_state(&sid).await.unwrap().members.contains_key(&bob_x),
            "Node A: tier-gated joiner is not a member"
        );
        assert!(
            !node_b.space_state(&sid).await.unwrap().members.contains_key(&bob_x),
            "Node B: tier-gated joiner is not a member"
        );

        node_a.shutdown().await;
        node_b.shutdown().await;
    }

    /// PG-12-min per-Room override, multiparty: a `(Moderator, SendMessages, Deny)`
    /// override on a Room blocks a Moderator's message on **every** Node, and the
    /// override itself converges (identical `permission_overrides` on every Node —
    /// it rides state-keyed `state.room_update`).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn s7_per_room_override_blocks_moderator_send_and_converges() {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        let alice = keypair::generate(); // owner
        let bob = keypair::generate(); // moderator
        let bob_id = pubkey_uri(&bob);

        let create = sign_event(
            build_space_create_event(&alice, "S7-override", None, 1, &node_a.node_id, None, false),
            &alice,
        );
        let sid = event_id_str(&create);
        let room = sign_event(build_room_create_event(&alice, &sid, "announcements", None), &alice);
        let rid = event_id_str(&room);

        // invite bob as Moderator, bob joins Space then Room.
        let mut invite = build_membership_event(
            &alice,
            &sid,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": bob_id, "role": "moderator" }),
        );
        invite.prev_events = vec![edx(&sid), edx(&rid)];
        let invite = sign_event(invite, &alice);
        let iid = event_id_str(&invite);

        let mut sjoin = build_membership_event(&bob, &sid, "", EventType::MembershipJoin, json!({}));
        sjoin.prev_events = vec![edx(&iid)];
        let sjoin = sign_event(sjoin, &bob);
        let sjid = event_id_str(&sjoin);

        // Room-level join (room_id set) so bob is a Room member able to send.
        let mut rjoin = build_membership_event(&bob, &sid, &rid, EventType::MembershipJoin, json!({}));
        rjoin.prev_events = vec![edx(&sjid)];
        let rjoin = sign_event(rjoin, &bob);
        let rjid = event_id_str(&rjoin);

        // alice (owner) sets the Deny override: Moderators can't post here.
        let ru = sign_event(
            build_room_update_event(
                &alice,
                &sid,
                &rid,
                vec![rjid.clone()],
                &[(Role::Moderator, RoomPermission::SendMessages, Effect::Deny)],
            ),
            &alice,
        );
        let ruid = event_id_str(&ru);

        // bob's message attempt — the gated event.
        let bob_msg = sign_event(
            build_message_text_event(&bob, &sid, &rid, vec![ruid.clone()], "mods-cant-post-here"),
            &bob,
        );

        for n in [&node_a, &node_b] {
            n.register_identity(&alice).await;
            n.register_identity(&bob).await;
            for ev in [&create, &room, &invite, &sjoin, &rjoin, &ru] {
                n.ingest(ev.clone()).await;
            }
            // The Moderator's send is the gated event — dispatch enforces the override.
            let outcome = n.submit_locally(bob_msg.clone()).await;
            assert!(
                matches!(outcome, DispatchOutcome::Rejected(_)),
                "the per-Room Deny override must block the Moderator's message on each Node, got {outcome:?}"
            );
        }

        // The override converges: identical permission_overrides on both Nodes.
        let ra = node_a.space_state(&sid).await.unwrap();
        let rb = node_b.space_state(&sid).await.unwrap();
        let oa = &ra.rooms.get(&crate::tests::phase9_harness::rdx(&rid)).unwrap().permission_overrides;
        let ob = &rb.rooms.get(&crate::tests::phase9_harness::rdx(&rid)).unwrap().permission_overrides;
        assert_eq!(oa, ob, "the override resolves identically on every Node (state-keyed convergence)");
        assert_eq!(
            oa.get(&(Role::Moderator, RoomPermission::SendMessages)),
            Some(&Effect::Deny),
            "the Deny override is present in the resolved Room state"
        );

        node_a.shutdown().await;
        node_b.shutdown().await;
    }
}
