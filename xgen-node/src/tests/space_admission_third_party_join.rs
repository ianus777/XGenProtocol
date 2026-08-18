// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M-SPACE-ADMISSION Leg A-bis leg ① — the before-assertion.
//! Runbook: `tasks/RUNBOOK_SPACE_ADMISSION_LEG_A_BIS.md` v1.4 (LOCKED).
//!
//! **What this records.** Today a *registered* third-party Identity may submit a
//! space-level `membership.join` against a Space it is not party to — including a
//! **DM** — and the local dispatch path admits it as `Role::Member` with no invite.
//! Nothing between `dispatch_event` entry and `apply_join`'s `members` insert
//! refuses it.
//!
//! **Why it is a before-assertion, and why it lands before the gate.** Leg D ships
//! the admission gate, and under `D-148` clause 4 a DM pins `invite` — so once the
//! gate exists there is no configuration under which a DM can be observed admitting
//! an uninvited third party, and no later session can produce this measurement.
//!
//! ## The three traps this fixture is built against
//!
//! 1. **The actor must be a REGISTERED Identity.** `exchange.rs:601-634` Step 11
//!    HeldPends an *unregistered* sender universally; `MembershipJoin` is **not**
//!    exempt (`skip_membership` covers the *membership* check in the block below
//!    it). A fresh-keypair fixture would assert `HeldPending`, go green, and prove
//!    nothing about admission.
//! 2. **The actor must be a THIRD identity — neither creator nor invitee.** The DM
//!    counterpart holds a `PendingInvite` (`state.rs:496-540`), and the
//!    invite-expiry gate (`runtime.rs:1580-1610`) branches on exactly that. With
//!    the counterpart as actor the test would be measuring the invite-expiry gate
//!    and reporting it as admission.
//! 3. **`room_id` must be EMPTY.** A non-empty `room_id` takes `apply_join`'s
//!    room-level arm (`state.rs:1002-1005`), which requires existing Space
//!    membership — the test would assert `NotASpaceMember` and read as if a gate
//!    existed.
//!
//! **And the tier gate passes on a coincidence.** `verify_tier_assertion(joiner_tier,
//! space.auth_tier)` (`runtime.rs:1532-1544`) is a no-op only because
//! `build_dm_space_create_event` writes `"auth_tier": 1` and `make_identity_record`
//! sets `trust_assertion: None`. Neither is pinned by anything else, so both tests
//! assert `auth_tier == 1` *before* submitting: without that line a `3030` rejection
//! would read as an admission result — a wrong diagnosis, delivered silently.
//!
//! ## What these tests do NOT prove (runbook §6, stated before they are run)
//!
//! 1. They do **not** exercise `accept_registration`, and therefore none of the
//!    three deployment states: the harness registers by calling
//!    `NodeRuntime::register_identity` directly, so no `AssertionPolicy` is
//!    consulted. These tests prove what a *registered* Identity can do; they say
//!    nothing about how hard it is to become one — and that is the entire content
//!    of the hole's size.
//! 2. They do **not** exercise the wire. `server_authenticate` / the WebSocket
//!    listener / `is_revoked` are not on the harness path.
//! 3. They are **not** an exploit against a running Node. In-process, one node, no
//!    network.
//! 4. They prove **nothing about federation**. `peer_node_id` is `None` throughout,
//!    so the F-3 relationship gate is never evaluated.
//!
//! **The companion test is the permanent half.** The DM test is a one-way witness.
//! `third_party_registered_identity_joins_an_open_space` runs the identical join
//! against an ordinary Space, which under `D-148` clause 3 defaults to `open`
//! forever — it is the standing assertion that open join still works.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::tests::phase9_harness::{
        edx, event_id_str, idx, now_rfc, pubkey_uri, rdx, sdx, spawn_in_process_node,
    };
    use crate::{
        identity::keypair,
        node::runtime::DispatchOutcome,
        space::{
            membership::Role,
            state::{build_dm_space_create_event, build_space_create_event, sign_event},
        },
        wire::types::{Event, EventType},
    };

    /// Leg ① — a registered third party joins a **DM** it is not party to.
    ///
    /// Records today's behaviour: `Accepted`, `Role::Member`, `invited_by: None`,
    /// and — the assertion that names the shape of the hole — the DM's *actual*
    /// counterpart is still not a member while the stranger is.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn third_party_registered_identity_joins_a_dm_it_is_not_party_to() {
        let node = spawn_in_process_node().await;

        // ── Setup (runbook §4.2) — `ingest` only; only the subject event goes
        // through `submit_locally`, the path under test. ─────────────────────
        let alice_key = keypair::generate(); // DM creator
        let bob_key = keypair::generate(); // DM counterpart — emits no event
        let carol_key = keypair::generate(); // the third party
        let alice_id = pubkey_uri(&alice_key);
        let bob_id = pubkey_uri(&bob_key);
        let carol_id = pubkey_uri(&carol_key);

        // Carol's registration is the X-1 requirement and the single most
        // load-bearing line in this fixture: unregistered, Step 11 HeldPends and
        // the test would prove nothing. Bob is registered too — he emits no
        // event, but an unregistered counterpart would make the DM's own state a
        // second variable.
        node.register_identity(&alice_key).await;
        node.register_identity(&bob_key).await;
        node.register_identity(&carol_key).await;

        let dm_ev = sign_event(
            build_dm_space_create_event(&alice_key, &bob_id, &node.node_id),
            &alice_key,
        );
        let space_id = event_id_str(&dm_ev);
        node.ingest(dm_ev).await;

        // ── Preconditions (runbook §4.3) — asserted, not assumed ─────────────
        let before = node.space_state(&space_id).await.expect(
            "the DM Space resolves; otherwise `space not found` would reject the join \
             and the failure would read as if admission refused it",
        );
        assert!(before.is_dm, "the subject is a DM");
        assert!(
            before.dm_constraints_active,
            "the DM's two-party constraints are active"
        );
        assert_eq!(
            before.auth_tier, 1,
            "H-6: the tier gate is a no-op only at auth_tier 1; if this ever changes, \
             the join rejects with 3030 and the failure message would be about \
             admission rather than about the tier"
        );
        assert!(
            before.is_member(&alice_id),
            "the DM has its creator, so the Space is real and not an empty shell"
        );
        assert!(
            !before.is_member(&carol_id),
            "carol is not a member before the subject event"
        );
        assert!(
            !before.pending_invites.contains_key(&idx(&carol_id)),
            "H-5: carol holds no pending invite — this is what proves the subject is on \
             the admission path and not on the invite-expiry path"
        );
        assert!(
            before.pending_invites.contains_key(&idx(&bob_id)),
            "and the counterpart DOES hold one, which is why he cannot be the actor"
        );

        let tips = node.dag_tips(&space_id).await;
        assert!(
            !tips.is_empty(),
            "the DAG has tips; `validate_dag_structure` (step 10, which runs BEFORE the \
             predecessor lookup) rejects empty or malformed prev_events with a DagError"
        );

        // ── The subject event (runbook §4.4) ─────────────────────────────────
        // `rdx("")` is load-bearing: a non-empty room_id takes apply_join's
        // room-level arm, which requires existing Space membership.
        let carol_join = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(&carol_id),
                rdx(""),
                sdx(&space_id),
                tips.iter().map(|t| edx(t)).collect(),
                now_rfc(),
                json!({}),
            ),
            &carol_key,
        );
        let outcome = node.submit_locally(carol_join).await;

        // ── Assertions (runbook §4.5) ────────────────────────────────────────
        // 1 — the dispatch path admitted it.
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "an uninvited registered third party's join of a DM is ACCEPTED today; got {outcome:?}"
        );

        let after = node
            .space_state(&space_id)
            .await
            .expect("the DM Space still resolves after the join");

        // 2 — and the state records her as a full member. Assertion 1 alone is an
        // outcome of the dispatch; `Role::Member` is the fact the milestone is about.
        assert_eq!(
            after.member_role(&carol_id),
            Some(&Role::Member),
            "carol is recorded as a full Space member"
        );

        // 3 — and nobody invited her. This is the assertion that names the hole:
        // `apply_join`'s `None => (Role::Member, None)` arm made visible.
        let carol_member = after
            .members
            .get(&idx(&carol_id))
            .expect("carol is in `members`");
        assert!(
            carol_member.invited_by.is_none(),
            "carol was admitted with no invite at all — `invited_by` is None"
        );

        // 4 — the DM's ACTUAL counterpart is not a member while the STRANGER is.
        // Membership went {alice} -> {alice, carol} and `is_dm` is still true: the
        // Space is a three-name DM whose named second party never joined, and
        // nothing enforces the two-party invariant.
        assert!(
            !after.is_member(&bob_id),
            "the DM's named counterpart is STILL not a member, while the third party is"
        );
        assert!(after.is_dm, "and the Space is still flagged as a DM");
        assert!(after.is_member(&alice_id), "the creator is unaffected");

        node.shutdown().await;
    }

    /// The companion (runbook §4.6) — the same third-party join against an
    /// **ordinary** Space, which under `D-148` clause 3 defaults to `open` forever.
    ///
    /// This one is not touched by Leg D. If a later leg finds it must weaken or
    /// edit this test, that is a finding about the gate's scope — the gate would be
    /// refusing an open join.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn third_party_registered_identity_joins_an_open_space() {
        let node = spawn_in_process_node().await;

        // No bob: an ordinary Space has no counterpart, and adding one would import
        // the DM's shape into a test whose whole point is that it is not a DM.
        let alice_key = keypair::generate();
        let carol_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        let carol_id = pubkey_uri(&carol_key);

        node.register_identity(&alice_key).await;
        node.register_identity(&carol_key).await;

        // The trailing `false` is `e2e_encryption`, NOT `is_dm` — `is_dm` is not a
        // parameter at all; `from_space_create` hardcodes it false.
        let space_ev = sign_event(
            build_space_create_event(&alice_key, "Open Space", None, 1, &node.node_id, None, false),
            &alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        // ── Preconditions ────────────────────────────────────────────────────
        let before = node
            .space_state(&space_id)
            .await
            .expect("the Space resolves");
        assert_eq!(
            before.auth_tier, 1,
            "H-6: the tier gate is a no-op only at auth_tier 1"
        );
        // Structurally guaranteed rather than a risk being guarded — asserted
        // because it documents the test's subject.
        assert!(!before.is_dm, "the subject is an ordinary Space, not a DM");
        assert!(!before.dm_constraints_active, "no DM constraints");
        // The create seeded her — alice needs no `membership.join`.
        assert_eq!(
            before.member_role(&alice_id),
            Some(&Role::Owner),
            "the creator is a member with Role::Owner the instant the create Event is ingested"
        );
        assert!(
            !before.is_member(&carol_id),
            "carol is not a member before the subject event"
        );
        assert!(
            !before.pending_invites.contains_key(&idx(&carol_id)),
            "carol holds no pending invite"
        );

        let tips = node.dag_tips(&space_id).await;
        assert!(!tips.is_empty(), "the DAG has tips");

        // ── The subject event ────────────────────────────────────────────────
        let carol_join = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(&carol_id),
                rdx(""),
                sdx(&space_id),
                tips.iter().map(|t| edx(t)).collect(),
                now_rfc(),
                json!({}),
            ),
            &carol_key,
        );
        let outcome = node.submit_locally(carol_join).await;

        // ── Assertions (§4.5 items 1-3; §4.5's fourth is DM-specific and has no
        // subject here — there is no counterpart in an ordinary Space) ────────
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "an open Space admits an uninvited registered identity; got {outcome:?}"
        );

        let after = node
            .space_state(&space_id)
            .await
            .expect("the Space still resolves after the join");
        assert_eq!(
            after.member_role(&carol_id),
            Some(&Role::Member),
            "carol is recorded as a full Space member"
        );
        let carol_member = after
            .members
            .get(&idx(&carol_id))
            .expect("carol is in `members`");
        assert!(
            carol_member.invited_by.is_none(),
            "carol was admitted with no invite at all — `invited_by` is None"
        );

        node.shutdown().await;
    }
}
