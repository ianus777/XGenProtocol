// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M8 — Wave 1 / C2 — S2 concurrent **state-event** convergence (the M2 headline:
//! byte-identical resolved `SpaceState` across all Nodes AND every client
//! projection, under every arrival permutation). Runbook `tasks/M8_MULTIPARTY_IMPL.md`
//! §3 C2; design `tasks/M8_MULTIPARTY_DESIGN.md` §3 (S2 row) + §4 (M2/M8-D2).
//!
//! **What this extends.** The shipped S2 instruction (`MULTIPARTY_S2_concurrent_send.md`)
//! and `phase9_m8_convergence_smoke.rs` cover concurrent *message* sends + one
//! Layer-1 ban/join conflict on two Nodes. C2 widens to concurrent **state**
//! events that genuinely conflict, across the three resolution layers a
//! client-reachable conflict can land on under the R2-F01 A-pure empty
//! `identity_home_nodes` map — Layer 1 (membership removal), Layer 4 (role
//! precedence), Layer 5c (lexicographic backstop). These are exactly the
//! G-ALIGN-safe layers: Layers 3/5a/5b consult the home-node map and would
//! diverge between a node (real map) and a client (empty map), so the design's
//! three cases deliberately avoid them (R2-F01 finding).
//!
//! **CP-4 placement (M8-D6).** Convergence correctness is a deterministic pure
//! property — real OS processes add no signal to a permutation proof — so the M2
//! headline lives here as a workspace integration test. The operator-realistic
//! concurrent-federation-send + DAG-coherence run stays binary-level
//! (`MULTIPARTY_S2_findings.md`).
//!
//! **Three assertions per case (M2 + G-ALIGN):**
//!   1. **Exhaustive permutation convergence.** `derive_resolved` over *every*
//!      permutation of the event log yields one byte-identical `SpaceState`
//!      (`SpaceState: PartialEq`). `derive_resolved` topo-sorts internally, so
//!      this is the faithful "every arrival permutation" statement.
//!   2. **Cross-node live seam.** Two real in-process Nodes ingest the same
//!      events with the concurrent pair in opposite order through the live
//!      `InProcessNode::ingest` → `ingest_event` → SR-D1 gate → `derive_resolved`
//!      path; their resolved `space_state()` are byte-identical to each other and
//!      to (1).
//!   3. **G-ALIGN.** The client A-pure projection — `derive_resolved(log, "",
//!      &empty)`, exactly what `xgen-client`'s `members_projection` calls after
//!      R2-F01 C1 — equals each Node's resolved view. Holds for Layers 1/4/5c
//!      because none consult the home-node map.
//!
//! **M8-D4 key-rotation substitution (recorded finding, not a fix).** The design
//! lists "key-rotation" as the third conflict case. `EventType::SystemKeyRotation`
//! has a `state_key_for_event` arm (`resolution/state_key.rs`) but **no builder
//! and no `apply_event` arm** — a dormant forward-ready EventType. A concurrent
//! key-rotation conflict is therefore not buildable on B without new wire surface
//! (which M8 must not add). It is replaced here by `thread.status`
//! resolved-vs-archived, which exercises the **same resolution layer** (5c). The
//! unbuilt key-rotation path is an M9-scoping input (feeds the multiparty
//! redesign / Arc-H real crypto), per M8-D4.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ed25519_dalek::SigningKey;
    use serde_json::json;

    use xgen_core::resolution::derive_resolved;

    use crate::{
        crypto::encoding,
        identity::keypair,
        space::{
            membership::{Effect, Role, RoomPermission},
            state::{
                build_membership_event, build_room_create_event,
                build_room_update_event, build_space_create_event,
                build_thread_archived_event, build_thread_create_event,
                build_thread_resolved_event, sign_event, thread_id_from_event_id,
                SpaceState,
            },
        },
        wire::types::{Event, EventType},
    };
    use xgen_common::xgid::{EventXgid, IdentityXgid, Xgid};

    use crate::tests::phase9_harness::spawn_in_process_node;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn pubkey_uri(k: &SigningKey) -> String {
        format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(k.verifying_key().as_bytes())
        )
    }
    fn eid(ev: &Event) -> String {
        ev.event_id.as_ref().unwrap().as_str().to_string()
    }
    fn edx(s: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn idx(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }

    /// All permutations of `items` (Heap's algorithm). N ≤ 6 here, so ≤ 720 —
    /// cheap, and the strongest possible "every arrival permutation" statement.
    fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
        let mut out = Vec::new();
        let mut a: Vec<T> = items.to_vec();
        let n = a.len();
        let mut c = vec![0usize; n];
        out.push(a.clone());
        let mut i = 0;
        while i < n {
            if c[i] < i {
                if i % 2 == 0 {
                    a.swap(0, i);
                } else {
                    a.swap(c[i], i);
                }
                out.push(a.clone());
                c[i] += 1;
                i = 0;
            } else {
                c[i] = 0;
                i += 1;
            }
        }
        out
    }

    fn empty_map() -> HashMap<String, String> {
        HashMap::new()
    }

    /// The client A-pure projection (R2-F01: `derive_resolved(log, "", &empty)`),
    /// which is exactly what `xgen-client::ops::members_projection` calls.
    fn client_projection(log: &[Event]) -> SpaceState {
        derive_resolved(log.to_vec(), "", &empty_map())
            .expect("client projection derives a SpaceState")
    }

    /// Assert tiers 1 + 3 of the convergence contract on a built event log:
    ///   (1) every permutation derives one identical SpaceState;
    ///   (3) that state == the client A-pure projection.
    /// Returns the canonical resolved state for the caller's semantic + cross-node
    /// assertions.
    fn assert_permutation_convergence_and_alignment(log: &[Event]) -> SpaceState {
        let canonical = client_projection(log);
        let perms = permutations(log);
        for (n, perm) in perms.iter().enumerate() {
            let s = derive_resolved(perm.clone(), "", &empty_map())
                .expect("each permutation derives a SpaceState");
            assert_eq!(
                s, canonical,
                "permutation #{n} diverged — derive_resolved is not order-independent \
                 for this conflict (M2 byte-identical FAIL)"
            );
        }
        // G-ALIGN tier (3): the canonical IS the client projection by construction;
        // re-derive under a non-empty vantage node_id to prove my_node_id does not
        // perturb these layers (no federation_add → federation_nodes empty).
        let as_node_vantage = derive_resolved(log.to_vec(), "xgen://pubkey/ed25519:NODE_X", &empty_map())
            .expect("node-vantage derive");
        assert_eq!(
            as_node_vantage, canonical,
            "node vantage perturbed the resolved state — G-ALIGN would not hold"
        );
        canonical
    }

    /// Tier 2: two real Nodes ingest `causal_prefix` in order, then the concurrent
    /// pair in opposite orders; both resolved snapshots must equal `canonical`
    /// (cross-node M2) and each must equal the client projection (G-ALIGN at the
    /// live seam).
    async fn assert_cross_node_seam(
        causal_prefix: &[Event],
        concurrent_a: &Event,
        concurrent_b: &Event,
        space_id: &str,
        canonical: &SpaceState,
    ) {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        for ev in causal_prefix {
            node_a.ingest(ev.clone()).await;
            node_b.ingest(ev.clone()).await;
        }
        // A: a then b; B: b then a — opposite conflict arrival order.
        node_a.ingest(concurrent_a.clone()).await;
        node_a.ingest(concurrent_b.clone()).await;
        node_b.ingest(concurrent_b.clone()).await;
        node_b.ingest(concurrent_a.clone()).await;

        let state_a = node_a.space_state(space_id).await.expect("Node A state");
        let state_b = node_b.space_state(space_id).await.expect("Node B state");

        assert_eq!(state_a, state_b, "two Nodes diverged at the live SR-D1 seam");
        assert_eq!(&state_a, canonical, "Node A diverged from the pure-derive canonical");
        // G-ALIGN at the live seam: client A-pure projection == node resolved view.
        let full_log: Vec<Event> = causal_prefix
            .iter()
            .cloned()
            .chain([concurrent_a.clone(), concurrent_b.clone()])
            .collect();
        assert_eq!(
            client_projection(&full_log),
            state_a,
            "client projection (A-pure) diverged from the Node's resolved view (G-ALIGN FAIL)"
        );

        node_a.shutdown().await;
        node_b.shutdown().await;
    }

    // ── Case 1 — Layer 1: concurrent ban vs join (same target) ─────────────────

    /// alice (owner) bans bob; bob joins; both reference the create root → genuine
    /// concurrent conflict on `membership:space:bob`. Layer 1 (removal precedence):
    /// ban wins → bob banned, not a member — on every Node, every arrival order.
    #[tokio::test]
    async fn s2_layer1_ban_vs_join_converges() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_id = pubkey_uri(&bob);

        let create = sign_event(
            build_space_create_event(&alice, "S2-L1", None, 1, "xgen://pubkey/ed25519:HOME", None, false),
            &alice,
        );
        let sid = eid(&create);

        let mut join = build_membership_event(&bob, &sid, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![edx(&sid)];
        let join = sign_event(join, &bob);

        let mut ban = build_membership_event(
            &alice,
            &sid,
            "",
            EventType::MembershipBan,
            json!({ "target_identity": bob_id }),
        );
        ban.prev_events = vec![edx(&sid)];
        let ban = sign_event(ban, &alice);

        let log = vec![create.clone(), join.clone(), ban.clone()];
        let canonical = assert_permutation_convergence_and_alignment(&log);

        // Semantic winner: ban beats join.
        let bob_x = idx(&bob_id);
        assert!(canonical.banned.contains(&bob_x), "Layer 1: ban must win — bob banned");
        assert!(!canonical.members.contains_key(&bob_x), "the dropped join leaves bob a non-member");

        assert_cross_node_seam(&[create], &join, &ban, &sid, &canonical).await;
    }

    // ── Case 2 — Layer 4: concurrent role-precedence (owner vs admin) ───────────

    /// alice (owner) and bob (admin) concurrently issue `state.room_update` with
    /// **different** permission overrides on the same Room. Same EventType → Layer 1
    /// abstains; Layer 4 (role) → Owner > Admin → alice's overrides win, on every
    /// Node, every arrival order. Exercises the Arc-D per-Room override path under
    /// multiparty conflict (a capability with zero prior multiparty coverage).
    #[tokio::test]
    async fn s2_layer4_role_precedence_converges() {
        let alice = keypair::generate(); // owner
        let bob = keypair::generate(); // admin
        let bob_id = pubkey_uri(&bob);

        let create = sign_event(
            build_space_create_event(&alice, "S2-L4", None, 1, "xgen://pubkey/ed25519:HOME", None, false),
            &alice,
        );
        let sid = eid(&create);

        let room = sign_event(build_room_create_event(&alice, &sid, "general", None), &alice);
        let rid = eid(&room);

        // alice (owner) invites bob as admin.
        let mut invite = build_membership_event(
            &alice,
            &sid,
            "",
            EventType::MembershipInvite,
            json!({ "target_identity": bob_id, "role": "admin" }),
        );
        invite.prev_events = vec![edx(&rid)];
        let invite = sign_event(invite, &alice);
        let iid = eid(&invite);

        // bob joins (space-level) → becomes Admin member.
        let mut join = build_membership_event(&bob, &sid, "", EventType::MembershipJoin, json!({}));
        join.prev_events = vec![edx(&iid)];
        let join = sign_event(join, &bob);
        let jid = eid(&join);

        // Concurrent room_updates with DIFFERENT overrides (both prev=[join]).
        let ru_owner = sign_event(
            build_room_update_event(
                &alice,
                &sid,
                &rid,
                vec![jid.clone()],
                &[(Role::Member, RoomPermission::SendMessages, Effect::Deny)],
            ),
            &alice,
        );
        let ru_admin = sign_event(
            build_room_update_event(
                &bob,
                &sid,
                &rid,
                vec![jid.clone()],
                &[(Role::Member, RoomPermission::SendMessages, Effect::Allow)],
            ),
            &bob,
        );

        let log = vec![
            create.clone(),
            room.clone(),
            invite.clone(),
            join.clone(),
            ru_owner.clone(),
            ru_admin.clone(),
        ];
        let canonical = assert_permutation_convergence_and_alignment(&log);

        // Semantic winner: owner's override (Deny) applied, not admin's (Allow).
        let room_state = canonical
            .rooms
            .get(&crate::tests::phase9_harness::rdx(&rid))
            .expect("room present in resolved state");
        let effect = room_state
            .permission_overrides
            .get(&(Role::Member, RoomPermission::SendMessages));
        assert_eq!(
            effect,
            Some(&Effect::Deny),
            "Layer 4: owner's override must win over the concurrent admin override"
        );

        assert_cross_node_seam(
            &[create, room, invite, join],
            &ru_owner,
            &ru_admin,
            &sid,
            &canonical,
        )
        .await;
    }

    // ── Case 3 — Layer 5c: concurrent thread resolved vs archived ──────────────

    /// alice (owner) concurrently resolves AND archives the same Thread. Both share
    /// `thread.status:thread_id`; not membership (Layer 1 abstains); same sender →
    /// same role → Layer 4 abstains; empty map → 5a/5b abstain → **Layer 5c
    /// lexicographic event_id backstop** elects one. Deterministic terminal status
    /// on every Node, every arrival order. (Substitutes the dormant key-rotation
    /// case — same resolution layer; see module docs / M8-D4.)
    #[tokio::test]
    async fn s2_layer5c_thread_status_converges() {
        let alice = keypair::generate();

        let create = sign_event(
            build_space_create_event(&alice, "S2-L5c", None, 1, "xgen://pubkey/ed25519:HOME", None, false),
            &alice,
        );
        let sid = eid(&create);

        let room = sign_event(build_room_create_event(&alice, &sid, "general", None), &alice);
        let rid = eid(&room);

        let tc = sign_event(
            build_thread_create_event(&alice, &sid, &rid, vec![rid.clone()], Some("topic"), 1),
            &alice,
        );
        let tc_id = eid(&tc);
        let thread_id = thread_id_from_event_id(&tc_id);

        let resolved = sign_event(
            build_thread_resolved_event(&alice, &sid, &rid, &thread_id, vec![tc_id.clone()]),
            &alice,
        );
        let archived = sign_event(
            build_thread_archived_event(&alice, &sid, &rid, &thread_id, vec![tc_id.clone()]),
            &alice,
        );

        let log = vec![
            create.clone(),
            room.clone(),
            tc.clone(),
            resolved.clone(),
            archived.clone(),
        ];
        let canonical = assert_permutation_convergence_and_alignment(&log);

        // Semantic winner: Layer 5c picks the lexicographically-lower event_id; the
        // Thread's terminal status is that winner's status. Assert the resolved
        // status matches the lower-id winner deterministically.
        use xgen_common::wire::ThreadStatus;
        let winner_is_resolved = eid(&resolved) < eid(&archived);
        let expected = if winner_is_resolved {
            ThreadStatus::Resolved
        } else {
            ThreadStatus::Archived
        };
        let thread = canonical.threads.get(&thread_id).expect("thread present");
        assert_eq!(
            thread.status, expected,
            "Layer 5c: terminal status must be the lexicographically-lower event_id's status"
        );

        assert_cross_node_seam(&[create, room, tc], &resolved, &archived, &sid, &canonical).await;
    }
}
