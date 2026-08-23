// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M-SPACE-ADMISSION Leg A-bis leg ① — **INVERTED BY LEG D**.
//! Runbook: `tasks/RUNBOOK_SPACE_ADMISSION_LEG_A_BIS.md` v1.4 (LOCKED);
//! inverted per that runbook's §4.6 and `J-755`'s Leg D DoD item.
//!
//! **What this records NOW.** A registered third-party Identity submitting a
//! space-level `membership.join` against a **DM** it is not party to is refused
//! `3047 admission_required`: a DM pins `admission = invite` at creation
//! (`D-148` clause 4), and the Leg D gate at `runtime.rs:1580`'s block refuses a
//! joiner holding no pending invite before the applier is ever reached.
//!
//! **WHAT IT RECORDED BEFORE, KEPT BECAUSE THE HOLE IS THE POINT.** Until Leg D
//! this same fixture asserted the opposite, and it passed: the local dispatch
//! path admitted carol as `Role::Member` with `invited_by: None`, and **nothing
//! between `dispatch_event` entry and `apply_join`'s `members` insert refused
//! her**. That was not a gap in enforcement — there was no enforcement to have a
//! gap in. Each of the four assertions below carries the line it replaced.
//!
//! **Why it was written as a one-way witness.** Once the gate exists there is no
//! configuration under which a DM can be observed admitting an uninvited third
//! party, so the measurement could not be produced by any later session — it had
//! to be taken before the gate, and then edited into its opposite by the leg that
//! shipped the gate. **A GREEN run of the UN-EDITED test after Leg D would have
//! been a failure of the gate, not a pass** (`J-755`, `N-109`). It went RED, on
//! assertion 1, which is the gate reporting for duty.
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
//! **The companion test is the permanent half, and Leg D did NOT touch it.**
//! `third_party_registered_identity_joins_an_open_space` runs the identical join
//! against an ordinary Space, which under `D-148` clause 3 defaults to `open`
//! forever — it is the standing assertion that open join still works, and it was
//! green through Leg D's gate and through Leg D's own negative controls. That is
//! what makes the DM inversion above readable as *the gate closed a DM* rather
//! than as *the gate closed everything*. If a later leg finds this companion must
//! be weakened or edited, that is a finding about the gate's SCOPE.

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

    /// Leg ① — a registered third party ATTEMPTS to join a **DM** it is not
    /// party to, and is refused. **INVERTED BY LEG D.**
    ///
    /// Asserts, since the gate: `Rejected(3047 admission_required)`, no role, no
    /// member record — and, unchanged from the before-assertion and meaning the
    /// opposite of what it used to, the DM's *actual* counterpart is still not a
    /// member either.
    ///
    /// ⚠️ **The function NAME still describes the attempt rather than the
    /// outcome**, which is at odds with this codebase's convention (`..._is_
    /// rejected_to_the_sender_end_to_end`). It is kept because renaming is a
    /// naming decision and the current name is cited by `docs/ROADMAP.md`, the
    /// `JOURNAL` and the A-bis runbook. Routed at Leg D's hand-back §2, not
    /// absorbed.
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

        // ── Assertions — INVERTED BY LEG D (M-SPACE-ADMISSION, J-755's DoD item) ──
        //
        // Each of the four assertions below was written as its opposite when this
        // test was the BEFORE-assertion, and each is reversed here rather than
        // deleted, so a reader can see the exact behaviour the gate closed.
        //
        // Assertion 4 is the exception and is UNCHANGED — it was true before the
        // gate and is true after it, for opposite reasons. That is not an oversight;
        // it is the sharpest line in the file, and the comment on it says why.
        //
        // 1 — the dispatch path REFUSES it, and names the reason.
        //     WAS: `matches!(outcome, DispatchOutcome::Accepted { .. })`.
        let reject = match outcome {
            DispatchOutcome::Rejected(info) => info,
            other => panic!(
                "a DM pins `invite` (`D-148` clause 4), so an uninvited third party's \
                 join MUST now be refused. Got {other:?}. A GREEN `Accepted` here is a \
                 FAILURE OF THE GATE, not a pass — it is this file recording the hole \
                 again, after the leg that was supposed to close it."
            ),
        };
        assert_eq!(
            reject.code, 3047,
            "wire 3047 admission_required — the refusal NAMES its reason. A 4000 \
             generic here would mean carol is being refused by something else \
             (banned, tier, malformed prev_events) and the gate is still absent"
        );
        assert_eq!(reject.name, "admission_required");

        let after = node
            .space_state(&space_id)
            .await
            .expect("the DM Space still resolves after the refused join");

        // 2 — and the state records NO role for her. Assertion 1 alone is an outcome
        //     of the dispatch; this is the fact the milestone is about, and a
        //     `Rejected` reply over a state that admitted her anyway would be the
        //     reply lying in the other direction.
        //     WAS: `Some(&Role::Member)`.
        assert_eq!(
            after.member_role(&carol_id),
            None,
            "carol holds no role in the Space at all"
        );

        // 3 — and she is not in `members`. This is the assertion that named the
        //     hole: `apply_join`'s `None => (Role::Member, None)` arm, which is now
        //     unreachable for her because the gate returns before the applier runs.
        //     WAS: an `expect("carol is in `members`")` followed by
        //     `carol_member.invited_by.is_none()` — the uninvited admission made
        //     visible.
        assert!(
            !after.members.contains_key(&idx(&carol_id)),
            "carol never reached the applier — no member record was created"
        );

        // 4 — UNCHANGED, AND IT MEANS THE OPPOSITE NOW. Before the gate this line
        //     read as an indictment: the DM's named counterpart was still absent
        //     while a STRANGER had been admitted, a three-name DM whose second party
        //     never joined. After the gate it reads as the ordinary state of a DM
        //     nobody has accepted yet — bob has not joined, and neither has anyone
        //     else. The assertion did not have to change for its meaning to invert,
        //     which is exactly why it is worth keeping.
        assert!(
            !after.is_member(&bob_id),
            "the DM's named counterpart has still not joined — and now nor has the \
             third party"
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
