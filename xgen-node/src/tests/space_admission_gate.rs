// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M-SPACE-ADMISSION Leg D — the admission gate, on the answer path.
//! Runbook: `tasks/RUNBOOK_SPACE_ADMISSION_LEG_D.md` v1.1 (LOCKED), §3 D-1, §4 V-3a.
//!
//! **Why this is a node-path test and not a `dispatch_event` unit test.** The
//! defect this leg refuses (`M-1`) is a composition failure: a check that lives
//! only in the applier is a silent no-op on the answer path, because every
//! production call site discards the applier's error (`let _ = ...apply_event`).
//! The codebase already names the result — `runtime.rs` calls it *the reply
//! lied*. A gate the sender never hears about admits nobody while telling them
//! they got in.
//!
//! So these tests go through `submit_locally`, and every assertion is about the
//! `DispatchOutcome` the SENDER receives, plus the membership that followed.
//!
//! **The controls are load-bearing, and there are two of them.** A `Rejected`
//! outcome on its own is equally consistent with the event being malformed,
//! mis-chained, unregistered, banned, or refused by any of the many gates
//! between dispatch entry and this one:
//!
//!   * the INVITED control proves the refusal is about the INVITE — an invited
//!     joiner's identical event must be Accepted and must actually land;
//!   * the OPEN control proves the refusal is about the SPACE'S `admission`
//!     VALUE — the same uninvited joiner into an `open` Space must be admitted.
//!
//! Without the second, a gate that refused every uninvited join regardless of
//! `admission` would pass — and that gate would close every Space created
//! before the property existed, which is precisely what `L-E` exists to prevent.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::tests::phase9_harness::{
        edx, event_id_str, idx, now_rfc, pubkey_uri, rdx, sdx, spawn_in_process_node,
        InProcessNode,
    };
    use crate::{
        identity::keypair,
        node::runtime::DispatchOutcome,
        space::{
            membership::Role,
            state::{
                build_dm_space_create_event, build_space_create_event,
                build_space_create_event_with_admission, sign_event,
            },
        },
        wire::types::{Event, EventType, ADMISSION_INVITE, ADMISSION_OPEN},
    };

    /// Build an unsigned space-level Event chained on the given tips.
    ///
    /// Deliberately NOT `state::build_membership_event`: that helper emits
    /// `prev_events: vec![]`, which is fine for the `state.rs` unit tests (they
    /// call `apply_event` directly and never touch the DAG) and is a structural
    /// violation on this path — step 10 rejects malformed `prev_events` BEFORE
    /// the admission gate runs, so an unchained join would be refused for a
    /// reason that has nothing to do with admission while looking exactly like
    /// the refusal this test is asserting.
    fn space_level_ev(
        key: &ed25519_dalek::SigningKey,
        space_id: &str,
        tips: &[String],
        event_type: EventType,
        content: serde_json::Value,
    ) -> Event {
        Event::new(
            event_type,
            idx(&pubkey_uri(key)),
            rdx(""),
            sdx(space_id),
            tips.iter().map(|t| edx(t)).collect(),
            now_rfc(),
            content,
        )
    }

    /// SUBJECT — an uninvited join into an invite-only Space is refused `3047`
    /// to its SENDER, while an invited join through the identical path lands.
    ///
    /// RED-on-revert: delete the admission gate in `dispatch_event`'s
    /// `MembershipJoin` block and carol's submission returns `Accepted` with
    /// carol a member — the exact pre-Leg-D behaviour, in which anyone holding a
    /// Space id could join any Space.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn uninvited_join_into_an_invite_only_space_is_rejected_3047_to_the_sender() {
        let node = spawn_in_process_node().await;

        let alice_key = keypair::generate(); // Space owner
        let bob_key = keypair::generate(); // INVITED — the control
        let carol_key = keypair::generate(); // UNINVITED — the subject
        let bob_id = pubkey_uri(&bob_key);
        let carol_id = pubkey_uri(&carol_key);

        // All registered: step 11-pre HeldPends an unregistered sender
        // universally, and a `HeldPending` outcome would prove nothing about
        // admission.
        node.register_identity(&alice_key).await;
        node.register_identity(&bob_key).await;
        node.register_identity(&carol_key).await;

        // The Space is created invite-only through the REAL create path, not by
        // hand-setting the field afterwards. `build_space_create_event_with_admission`
        // exists to close exactly the race a two-step create-then-mutate would
        // open: a Space meant to be invite-only that is `open` in between.
        let space_ev = sign_event(
            build_space_create_event_with_admission(
                &alice_key,
                "Leg D invite-only Space",
                None,
                1,
                &node.node_id,
                None,
                false,
                ADMISSION_INVITE,
            ),
            &alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        // Preconditions, asserted rather than assumed.
        let before = node.space_state(&space_id).await.expect(
            "the Space resolves; otherwise `space not found` would reject the subject \
             at step 1 and the failure would read as if admission refused it",
        );
        assert_eq!(
            before.admission, ADMISSION_INVITE,
            "the CREATE PARSE delivered `invite` to the state the gate reads. If this \
             fails, D-1 and D-2 are each fine in isolation and the thing between them \
             is broken — which is the composition failure this leg exists to refuse"
        );
        assert!(
            !before.dm_constraints_active,
            "this is an ORDINARY Space — the DM bar must not be what refuses carol"
        );
        assert!(
            !before.banned.contains(&idx(&carol_id)),
            "carol is not banned — the banned pre-check runs BEFORE this gate and its \
             refusal would be indistinguishable in shape"
        );

        // bob is invited. carol is not. That difference is the whole subject.
        //
        // `valid_until` is MANDATORY here and its absence is not a detail: the
        // 3044 expiry gate is fail-closed for a non-DM invite that carries none
        // (malformed/legacy), so an invite without it makes bob's control join
        // fail 3044 — a red test whose message is about expiry while the property
        // under test is admission. The first version of this test omitted it and
        // failed exactly that way. One hour is inside the T1 ceiling (14 days),
        // so the 3045 over-ceiling gate on the invite itself is not triggered
        // either.
        let valid_until = (chrono::Utc::now() + chrono::Duration::hours(1))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let tips = node.dag_tips(&space_id).await;
        let invite = sign_event(
            space_level_ev(
                &alice_key,
                &space_id,
                &tips,
                EventType::MembershipInvite,
                json!({
                    "target_identity": bob_id,
                    "role": "member",
                    "valid_until": valid_until,
                }),
            ),
            &alice_key,
        );
        node.ingest(invite).await;

        let mid = node.space_state(&space_id).await.expect("Space resolves");
        assert!(
            mid.pending_invites.contains_key(&idx(&bob_id)),
            "precondition: bob holds a pending invite"
        );
        assert!(
            !mid.pending_invites.contains_key(&idx(&carol_id)),
            "precondition: carol holds NO pending invite — the gate's whole subject"
        );

        // SUBJECT — carol, uninvited, tries to join.
        let tips = node.dag_tips(&space_id).await;
        assert!(
            !tips.is_empty(),
            "the DAG has tips; step 10 rejects malformed prev_events BEFORE the \
             admission gate, and that failure would look like this one"
        );
        let carol_join = sign_event(
            space_level_ev(
                &carol_key,
                &space_id,
                &tips,
                EventType::MembershipJoin,
                json!({}),
            ),
            &carol_key,
        );
        let outcome = node.submit_locally(carol_join).await;

        // THE ASSERTION THIS TEST EXISTS FOR: what the SENDER receives.
        let reject = match outcome {
            DispatchOutcome::Rejected(info) => info,
            other => panic!(
                "an uninvited join into an invite-only Space must be REJECTED to its \
                 sender. Got {other:?}. If this is `Accepted`, there is no admission \
                 gate on the answer path and the join fell past the expiry check — \
                 which lives inside the pending-invite lookup and therefore never \
                 runs for a joiner holding no invite."
            ),
        };
        assert_eq!(
            reject.code, 3047,
            "wire code 3047 admission_required, not the unmapped 4000 fallback: the \
             refusal NAMES its reason so a client can act on it"
        );
        assert_eq!(reject.name, "admission_required");
        assert!(
            reject.reason.contains("3047"),
            "the reason string carries the code too; got {:?}",
            reject.reason
        );

        // And the gate has teeth — it did not merely fail to reply.
        let after_reject = node.space_state(&space_id).await.expect("Space resolves");
        assert!(
            !after_reject.is_member(&carol_id),
            "carol is NOT a member. An `Accepted`-shaped reply with the end state \
             still correct would be the reply lying; a `Rejected` whose end state \
             admitted her would be the same defect wearing the other face"
        );

        // CONTROL 1 — the INVITED joiner's identical event must be accepted and
        // must actually land. Without this the rejection above is consistent with
        // the Space simply being closed to everyone.
        let tips = node.dag_tips(&space_id).await;
        let bob_join = sign_event(
            space_level_ev(
                &bob_key,
                &space_id,
                &tips,
                EventType::MembershipJoin,
                json!({}),
            ),
            &bob_key,
        );
        let outcome = node.submit_locally(bob_join).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "the INVITED joiner's identical event must be accepted, or this test \
             cannot distinguish an invite gate from a closed door; got {outcome:?}"
        );
        let after = node.space_state(&space_id).await.expect("Space resolves");
        assert!(
            after.is_member(&bob_id),
            "and bob actually joined — an `Accepted` that added no member would be \
             an equally dishonest reply in the other direction"
        );
    }

    /// CONTROL 2 — the same uninvited joiner is ADMITTED to an `open` Space.
    ///
    /// This is what proves the gate keys on the Space's `admission` VALUE rather
    /// than on the joiner. It is a separate test so that a failure names which
    /// property broke: if this one goes red, `L-E` is broken and every Space
    /// created before `admission` existed has just been closed.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn the_same_uninvited_join_into_an_open_space_is_admitted() {
        let node = spawn_in_process_node().await;

        let alice_key = keypair::generate();
        let carol_key = keypair::generate();
        let carol_id = pubkey_uri(&carol_key);
        node.register_identity(&alice_key).await;
        node.register_identity(&carol_key).await;

        // The ordinary builder emits NO `admission` key — the absent state, which
        // takes the default. Asserted below rather than assumed.
        let space_ev = sign_event(
            build_space_create_event(
                &alice_key,
                "Leg D open Space",
                None,
                1,
                &node.node_id,
                None,
                false,
            ),
            &alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        let before = node.space_state(&space_id).await.expect("Space resolves");
        assert_eq!(
            before.admission, ADMISSION_OPEN,
            "precondition: an absent `admission` key still yields `open` (`L-E`)"
        );

        let tips = node.dag_tips(&space_id).await;
        let carol_join = sign_event(
            space_level_ev(
                &carol_key,
                &space_id,
                &tips,
                EventType::MembershipJoin,
                json!({}),
            ),
            &carol_key,
        );
        let outcome = node.submit_locally(carol_join).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "an OPEN Space must still admit an uninvited joiner — otherwise the gate \
             is refusing on the joiner rather than on the Space's admission value, \
             and every pre-existing Space has just been closed; got {outcome:?}"
        );
        let after = node.space_state(&space_id).await.expect("Space resolves");
        assert!(after.is_member(&carol_id), "and she actually joined");
    }

    /// Ingest an owner-signed `membership.invite` for `target_id`, chained on
    /// the Space's current tips.
    ///
    /// `valid_until` is MANDATORY here and its absence is not a detail: the 3044
    /// expiry gate is fail-closed for a non-DM invite that carries none
    /// (malformed/legacy), so an invite without it makes the invitee's join fail
    /// `3044` — a red test whose message is about expiry while the property under
    /// test is admission. One hour is inside the T1 ceiling (14 days), so the
    /// 3045 over-ceiling gate on the invite itself is not triggered either.
    async fn ingest_invite(
        node: &InProcessNode,
        owner_key: &ed25519_dalek::SigningKey,
        space_id: &str,
        target_id: &str,
    ) {
        let valid_until = (chrono::Utc::now() + chrono::Duration::hours(1))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let tips = node.dag_tips(space_id).await;
        let ev = sign_event(
            space_level_ev(
                owner_key,
                space_id,
                &tips,
                EventType::MembershipInvite,
                json!({
                    "target_identity": target_id,
                    "role": "member",
                    "valid_until": valid_until,
                }),
            ),
            owner_key,
        );
        node.ingest(ev).await;
    }

    /// Ingest a signed space-level membership event chained on current tips.
    /// The pre-federation SETUP primitive — it bypasses `dispatch_event`, so a
    /// gate on the answer path cannot interfere with fixture construction. Every
    /// call site asserts the resulting state rather than assuming it landed.
    async fn ingest_membership(
        node: &InProcessNode,
        key: &ed25519_dalek::SigningKey,
        space_id: &str,
        event_type: EventType,
        content: serde_json::Value,
    ) {
        let tips = node.dag_tips(space_id).await;
        let ev = sign_event(space_level_ev(key, space_id, &tips, event_type, content), key);
        node.ingest(ev).await;
    }

    /// Submit a space-level `membership.join` for `key` anchored on the GIVEN
    /// tips, through the production local-client path, and return what the
    /// SENDER receives.
    ///
    /// The anchor is a parameter because Leg G-2 is entirely about it: the same
    /// identity submitting the same join into the same Space gets opposite
    /// answers depending on whether the event is chained on her own last
    /// membership event or floats concurrent with it. `submit_join` below is
    /// this function with `tips = current`, so there is ONE construction path
    /// and the anchored and un-anchored cases cannot drift apart in any way
    /// except the one under test.
    async fn submit_join_on(
        node: &InProcessNode,
        key: &ed25519_dalek::SigningKey,
        space_id: &str,
        tips: &[String],
    ) -> DispatchOutcome {
        assert!(
            !tips.is_empty(),
            "the join has an anchor; step 10 rejects malformed prev_events BEFORE the \
             admission gate, and that failure would look like an admission refusal"
        );
        let ev = sign_event(
            space_level_ev(key, space_id, tips, EventType::MembershipJoin, json!({})),
            key,
        );
        node.submit_locally(ev).await
    }

    /// Submit a space-level `membership.join` for `key` through the production
    /// local-client path and return what the SENDER receives. Anchored on the
    /// Space's CURRENT tips — the well-behaved client's shape.
    async fn submit_join(
        node: &InProcessNode,
        key: &ed25519_dalek::SigningKey,
        space_id: &str,
    ) -> DispatchOutcome {
        let tips = node.dag_tips(space_id).await;
        submit_join_on(node, key, space_id, &tips).await
    }

    /// SUBJECT (Leg G-1 `V-1`) — a member who joined, then LEFT, re-joins an
    /// invite-only Space with NO new invite and is ADMITTED.
    ///
    /// This is the gate's third term. Until this leg the dispatch gate carried
    /// two of the three terms `§15.4` specifies, so bob was refused `3047` HERE
    /// — before `apply_join`'s own three-way gate, which has admitted him since
    /// Leg E-1, was ever reached. Two sites disagreed about one question and
    /// this one won, because it answers first.
    ///
    /// CONTROL (`V-2`) — carol, in the SAME Space, has never been a member and
    /// holds no invite; she must still be refused `3047`. Without her the new
    /// term is equally consistent with a blanket *admit everyone*, and bob's
    /// half of this test would pass just the same.
    ///
    /// RED-on-revert: delete the `!...is_some_and(|m| !m.is_present())` conjunct
    /// and bob's re-join is `Rejected` with code `3047` — the exact defect this
    /// leg repairs, and the exact code, which is what makes a revert run evidence
    /// rather than merely a failure.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_departed_member_rejoins_an_invite_only_space_without_a_new_invite() {
        let node = spawn_in_process_node().await;

        let alice_key = keypair::generate(); // Space owner
        let bob_key = keypair::generate(); // joins, leaves, RE-JOINS — the subject
        let carol_key = keypair::generate(); // never a member — the control
        let bob_id = pubkey_uri(&bob_key);
        let carol_id = pubkey_uri(&carol_key);

        node.register_identity(&alice_key).await;
        node.register_identity(&bob_key).await;
        node.register_identity(&carol_key).await;

        let space_ev = sign_event(
            build_space_create_event_with_admission(
                &alice_key,
                "Leg G-1 invite-only Space",
                None,
                1,
                &node.node_id,
                None,
                false,
                ADMISSION_INVITE,
            ),
            &alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        // Preconditions, asserted rather than assumed.
        let before = node.space_state(&space_id).await.expect("the Space resolves");
        assert_eq!(
            before.admission, ADMISSION_INVITE,
            "the create parse delivered `invite` to the state the gate reads"
        );
        assert!(
            !before.dm_constraints_active,
            "this is an ORDINARY Space — the DM bar must not be what admits or refuses"
        );
        assert!(
            before.banned.is_empty(),
            "nobody is banned — the banned pre-check runs BEFORE this gate and its \
             refusal would be indistinguishable in outcome shape"
        );

        // bob is invited and joins. The FIRST join is admitted by the INVITE
        // term, not by the new one — which is what makes the re-join below a
        // measurement of the third term specifically.
        ingest_invite(&node, &alice_key, &space_id, &bob_id).await;
        let mid = node.space_state(&space_id).await.expect("Space resolves");
        assert!(
            mid.pending_invites.contains_key(&idx(&bob_id)),
            "precondition: bob holds a pending invite"
        );

        let outcome = submit_join(&node, &bob_key, &space_id).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "precondition: the INVITED first join lands; got {outcome:?}"
        );
        let joined = node.space_state(&space_id).await.expect("Space resolves");
        assert!(joined.is_member(&bob_id), "precondition: bob is a present member");
        assert!(
            !joined.pending_invites.contains_key(&idx(&bob_id)),
            "precondition, and it is the whole reason this leg exists: `apply_join` \
             CONSUMES the pending invite, so bob's re-join below holds none. Were the \
             invite still standing, the re-join would be admitted by the SECOND term \
             and this test would measure nothing"
        );
        let joined_rec = joined
            .members
            .get(&idx(&bob_id))
            .expect("bob has a membership record");
        assert!(
            joined_rec.invited_by.is_some(),
            "precondition: the first join records alice as the inviter — the contrast \
             the re-join's re-derivation is measured against (`D-154`①)"
        );

        // bob leaves. `D-154`① — the record is RETAINED and MARKED, never removed.
        ingest_membership(&node, &bob_key, &space_id, EventType::MembershipLeave, json!({})).await;
        let left = node.space_state(&space_id).await.expect("Space resolves");
        let left_rec = left.members.get(&idx(&bob_id)).expect(
            "bob's record is RETAINED across the departure (`D-154`①). If this is \
             absent, `apply_leave` removed rather than marked and the third term has \
             nothing to key on — a different defect wearing this one's face",
        );
        assert!(
            !left_rec.is_present(),
            "precondition: bob is DEPARTED — the state the third term admits"
        );
        assert!(
            !left.is_member(&bob_id),
            "and the present-tense accessor agrees: a departed member is not a member"
        );

        // SUBJECT — bob re-joins, holding no invite, into a Space that admits by
        // invite only.
        let outcome = submit_join(&node, &bob_key, &space_id).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "a RETAINED DEPARTED member re-joins WITHOUT a new invite (`D-154`①, \
             `Q-2`(a)). Got {outcome:?}. A `Rejected` with code 3047 here is the \
             pre-Leg-G-1 behaviour: the dispatch gate refusing what `apply_join` \
             would have admitted"
        );

        let after = node.space_state(&space_id).await.expect("Space resolves");
        let bob_rec = after
            .members
            .get(&idx(&bob_id))
            .expect("bob has a membership record after the re-join");
        assert!(
            bob_rec.is_present(),
            "bob is PRESENT again — an `Accepted` that left him departed would be the \
             reply lying in the other direction"
        );
        assert!(
            bob_rec.left_at.is_none(),
            "and the departure boundary is CLEARED, not merely stepped over: `D-154`① \
             says the rejoin is *back as of now*"
        );
        assert_eq!(
            bob_rec.role,
            Role::Member,
            "the role is RE-DERIVED from `pending_invites` (absent means Member), never \
             carried forward — *presence, never position*"
        );
        assert!(
            bob_rec.invited_by.is_none(),
            "and so is `invited_by`: a rejoiner admitted without an invite was admitted \
             by NOBODY. It read `Some(alice)` before the departure, so this assertion \
             discriminates re-derivation from a stale carry-forward"
        );

        // CONTROL — carol has never been a member and holds no invite.
        let outcome = submit_join(&node, &carol_key, &space_id).await;
        let reject = match outcome {
            DispatchOutcome::Rejected(info) => info,
            other => panic!(
                "a STRANGER must still be refused. Got {other:?}. If this is \
                 `Accepted`, the new term is not *admit a departed member* but \
                 *admit everyone*, and bob's half of this test proves nothing"
            ),
        };
        assert_eq!(
            reject.code, 3047,
            "and refused by the ADMISSION gate specifically — 3047 admission_required, \
             not some other gate between dispatch entry and this one; got {reject:?}"
        );
        assert_eq!(reject.name, "admission_required");
        let after = node.space_state(&space_id).await.expect("Space resolves");
        assert!(
            !after.is_member(&carol_id),
            "carol is not a member — the refusal has teeth, it did not merely reply"
        );
    }

    /// CONTROLS B and C (Leg G-1 `V-3` + `V-4`) — a KICKED member re-joins; a
    /// BANNED one is refused, and refused by the PRE-CHECK rather than by the
    /// admission gate.
    ///
    /// They are one test because the CONTRAST is the property: `D-154`②③ says a
    /// kick is REMEMBERED and a ban BARS, and *the difference must be visible on
    /// the answer path*. Split across two tests, a reader has to assemble it.
    ///
    /// This is also what makes the absent ban clause in the new term a MEASURED
    /// decision rather than an omission. `apply_kick` marks departed and does not
    /// touch `banned`; `apply_ban` does both. So the third term admits the kicked
    /// member, and the banned one never reaches it — the dispatch-level banned
    /// pre-check refuses her earlier in the same function. Asserting the SHAPE of
    /// dave's refusal proves that ordering instead of assuming it: a second
    /// `banned.contains` inside the new term would be a second source of truth
    /// for one fact (`D-067`), and this is the assertion that lets it stay absent.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_kicked_member_rejoins_while_a_banned_one_is_refused_by_the_pre_check() {
        let node = spawn_in_process_node().await;

        let alice_key = keypair::generate(); // owner — kicks and bans
        let dave_key = keypair::generate(); // BANNED
        let erin_key = keypair::generate(); // KICKED
        let dave_id = pubkey_uri(&dave_key);
        let erin_id = pubkey_uri(&erin_key);

        node.register_identity(&alice_key).await;
        node.register_identity(&dave_key).await;
        node.register_identity(&erin_key).await;

        let space_ev = sign_event(
            build_space_create_event_with_admission(
                &alice_key,
                "Leg G-1 kick-vs-ban Space",
                None,
                1,
                &node.node_id,
                None,
                false,
                ADMISSION_INVITE,
            ),
            &alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        // Both are invited and both join, so the ONLY difference between them
        // downstream is which removal verb alice used.
        for (key, id) in [(&dave_key, &dave_id), (&erin_key, &erin_id)] {
            ingest_invite(&node, &alice_key, &space_id, id).await;
            let outcome = submit_join(&node, key, &space_id).await;
            assert!(
                matches!(outcome, DispatchOutcome::Accepted { .. }),
                "precondition: {id} joins on their invite; got {outcome:?}"
            );
        }

        ingest_membership(
            &node,
            &alice_key,
            &space_id,
            EventType::MembershipBan,
            json!({ "target_identity": dave_id }),
        )
        .await;
        ingest_membership(
            &node,
            &alice_key,
            &space_id,
            EventType::MembershipKick,
            json!({ "target_identity": erin_id }),
        )
        .await;

        // Preconditions — both DEPARTED, exactly one BANNED. If both were banned,
        // or neither, the two outcomes below would not be attributable to the
        // difference this test is about.
        let mid = node.space_state(&space_id).await.expect("Space resolves");
        for id in [&dave_id, &erin_id] {
            let rec = mid
                .members
                .get(&idx(id))
                .unwrap_or_else(|| panic!("{id}'s record is RETAINED (`D-154`②③)"));
            assert!(!rec.is_present(), "precondition: {id} is departed");
        }
        assert!(
            mid.banned.contains(&idx(&dave_id)),
            "precondition: `apply_ban` banned dave — `self.banned` stays the authority"
        );
        assert!(
            !mid.banned.contains(&idx(&erin_id)),
            "precondition, and it is the load-bearing half: `apply_kick` marks departed \
             and does NOT touch `banned`. If a kick banned, `D-154`②③ would collapse \
             and the third term would need a clause it does not have"
        );

        // CONTROL B (`V-3`) — the banned former member is refused, and NOT by the
        // admission gate.
        let outcome = submit_join(&node, &dave_key, &space_id).await;
        let reject = match outcome {
            DispatchOutcome::Rejected(info) => info,
            other => panic!(
                "a BANNED former member must be refused. Got {other:?}. An `Accepted` \
                 here means the new term admitted her past the ban"
            ),
        };
        assert_ne!(
            reject.code, 3047,
            "THE ORDERING ASSERTION. 3047 would mean the ADMISSION gate refused dave — \
             i.e. the banned pre-check did NOT run first, and the new term's missing \
             ban clause would be a real hole rather than a deliberate absence. Observed \
             refusal: {reject:?}"
        );
        assert_eq!(
            reject.code, 4000,
            "and the observed shape is the pre-check's own: PermissionDenied-class, \
             unmapped to a wire code, so `from_exchange` falls back to generic 4000. \
             Observed: {reject:?}"
        );
        assert_eq!(reject.name, "generic", "observed: {reject:?}");
        assert!(
            reject.reason.contains("banned"),
            "and the reason NAMES the ban, which is what makes 4000 readable here \
             rather than merely unmapped; observed: {:?}",
            reject.reason
        );
        let after_ban = node.space_state(&space_id).await.expect("Space resolves");
        assert!(
            !after_ban.is_member(&dave_id),
            "dave did not re-join — the refusal has teeth"
        );

        // CONTROL C (`V-4`) — the kicked member re-joins.
        let outcome = submit_join(&node, &erin_key, &space_id).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "a KICKED member re-joins: she is departed and not banned, so the third \
             term admits her and nothing stops her upstream (`D-154`②③). Got {outcome:?}"
        );
        let after = node.space_state(&space_id).await.expect("Space resolves");
        assert!(
            after.is_member(&erin_id),
            "and erin is a present member again — an `Accepted` that added nobody \
             would be the same dishonest reply wearing the other face"
        );
        assert!(
            !after.is_member(&dave_id),
            "while dave, in the SAME Space at the SAME moment, is still out. That is \
             the contrast this test exists for"
        );
    }

    /// `V-7` — a DM party who left can re-join her own DM.
    ///
    /// This is why `Q-2`(a) was ruled, and it is the case the missing term hurt
    /// most. Both DM constructors pin `admission = invite` at FOLD time, and the
    /// counterparty's seeded pending invite is CONSUMED by her first join — so
    /// before this leg her departure was irreversible, for BOTH parties, with no
    /// invite path back: `apply_invite` bars every DM invite as its first
    /// statement. A one-way door out of a two-person room.
    ///
    /// It needs no extra setup beyond a DM: it is the same gate, reached through
    /// a Space whose `admission` nobody chose and no owner can open.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_dm_party_who_left_can_rejoin_her_own_dm() {
        let node = spawn_in_process_node().await;

        let alice_key = keypair::generate(); // DM creator
        let bob_key = keypair::generate(); // the counterparty — leaves and returns
        let bob_id = pubkey_uri(&bob_key);

        node.register_identity(&alice_key).await;
        node.register_identity(&bob_key).await;

        let dm_ev = sign_event(
            build_dm_space_create_event(&alice_key, &bob_id, &node.node_id),
            &alice_key,
        );
        let space_id = event_id_str(&dm_ev);
        node.ingest(dm_ev).await;

        let before = node.space_state(&space_id).await.expect("the DM Space resolves");
        assert!(
            before.dm_constraints_active,
            "precondition: this is a DM, not an ordinary Space"
        );
        assert_eq!(
            before.admission, ADMISSION_INVITE,
            "precondition, and the reason every DM was affected: the constructor PINS \
             `invite` at fold time — nobody chose it, and no owner can open it"
        );
        assert!(
            before.pending_invites.contains_key(&idx(&bob_id)),
            "precondition: the create seeds the counterparty's invite"
        );

        let outcome = submit_join(&node, &bob_key, &space_id).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "precondition: bob joins his DM on the seeded invite; got {outcome:?}"
        );
        let joined = node.space_state(&space_id).await.expect("Space resolves");
        assert!(joined.is_member(&bob_id), "precondition: bob is in the DM");
        assert!(
            !joined.pending_invites.contains_key(&idx(&bob_id)),
            "precondition: the seeded invite is CONSUMED — there is no second one, and \
             `apply_invite` refuses to mint one in a DM"
        );

        ingest_membership(&node, &bob_key, &space_id, EventType::MembershipLeave, json!({})).await;
        let left = node.space_state(&space_id).await.expect("Space resolves");
        assert!(!left.is_member(&bob_id), "precondition: bob has left the DM");

        // SUBJECT — bob comes back.
        let outcome = submit_join(&node, &bob_key, &space_id).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "bob re-joins his own DM. Got {outcome:?}. A 3047 here is the one-way \
             door: pinned `invite`, a consumed seed, and no invite path back"
        );
        let after = node.space_state(&space_id).await.expect("Space resolves");
        let rec = after
            .members
            .get(&idx(&bob_id))
            .expect("bob has a membership record");
        assert!(rec.is_present(), "and he is present in the DM again");
        assert!(rec.left_at.is_none(), "with the departure boundary cleared");
        assert!(
            rec.invited_by.is_none(),
            "and `invited_by` re-derived to None — it read `Some(alice)` from the \
             seeded invite before he left (`D-154`①)"
        );
    }

    /// SUBJECT + DISCRIMINATOR (Leg G-2 `V-1` + `V-2`) — a returning member's
    /// UN-ANCHORED re-join is refused `3048` to its SENDER, and the identical
    /// re-join ANCHORED on her own `membership.leave` is admitted.
    ///
    /// They are one test because the contrast is the whole property. `3048`
    /// refusing every rejoin would look exactly like `3048` working, and it
    /// would silently undo Leg G-1 while every other test in this file stayed
    /// green. The second half is what makes the first half mean *un-anchored*
    /// rather than *rejoin*.
    ///
    /// WHAT THE REFUSAL REPLACES. Before this leg the node accepted the
    /// un-anchored re-join, appended it, and then — in `ingest_event`, one step
    /// past the reply — computed `conflicts_in_log` over the very same log,
    /// found the join concurrent with bob's own leave, and rebuilt the Space
    /// from `derive_resolved` WITHOUT him (`algorithm.rs` Layer 1 prefers
    /// `MembershipLeave` over `MembershipJoin`). The sender was told yes and the
    /// fold dropped him. The drop itself is asserted by a shipped green test —
    /// `resolution/derive.rs`'s
    /// `convergence_mp_f7_rejoin_anchored_at_root_is_dropped`; this leg is about
    /// the reply, not the drop.
    ///
    /// THE ORDER IS LOAD-BEARING and gives a property for free: the un-anchored
    /// submission runs FIRST. Had it landed, bob would be a PRESENT member and
    /// the anchored submission below would be refused `3047` (a present member
    /// is deliberately not admitted by the rejoin term). So the anchored half
    /// passing is itself evidence that the un-anchored half did not land.
    ///
    /// RED-on-revert: delete the `if is_rejoin { ... }` block in
    /// `dispatch_event` and the un-anchored submission returns `Accepted` with
    /// bob NOT a member afterwards — the silent failure, reproduced.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn an_unanchored_rejoin_is_refused_3048_while_the_anchored_one_lands() {
        let node = spawn_in_process_node().await;

        let alice_key = keypair::generate(); // Space owner
        let bob_key = keypair::generate(); // joins, leaves, re-joins twice
        let bob_id = pubkey_uri(&bob_key);

        node.register_identity(&alice_key).await;
        node.register_identity(&bob_key).await;

        let space_ev = sign_event(
            build_space_create_event_with_admission(
                &alice_key,
                "Leg G-2 invite-only Space",
                None,
                1,
                &node.node_id,
                None,
                false,
                ADMISSION_INVITE,
            ),
            &alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        // bob is invited, joins, and leaves — the state the rejoin term admits.
        ingest_invite(&node, &alice_key, &space_id, &bob_id).await;
        let outcome = submit_join(&node, &bob_key, &space_id).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "precondition: the invited first join lands; got {outcome:?}"
        );
        assert!(
            node.space_state(&space_id).await.expect("Space resolves").is_member(&bob_id),
            "precondition: bob is a present member"
        );

        ingest_membership(&node, &bob_key, &space_id, EventType::MembershipLeave, json!({})).await;
        let left = node.space_state(&space_id).await.expect("Space resolves");
        let left_rec = left
            .members
            .get(&idx(&bob_id))
            .expect("precondition: bob's record is RETAINED across the departure (`D-154`①)");
        assert!(
            !left_rec.is_present(),
            "precondition: bob is DEPARTED — the state the rejoin term keys on, and \
             therefore the state that makes this gate reachable at all"
        );

        // Captured BEFORE the refused submission so the store-growth check below
        // is a transition and not two reads of one value.
        let tips_before = node.dag_tips(&space_id).await;

        // SUBJECT (`V-1`) — bob re-joins anchored on the CREATE ROOT, which is
        // the fresh-install shape: a client with no local memory of its own
        // membership chain has nothing else to point at. The Space id IS the
        // create event's id, so this anchor is the root.
        let root_anchor = vec![space_id.clone()];
        let outcome = submit_join_on(&node, &bob_key, &space_id, &root_anchor).await;

        // Membership is read BEFORE the outcome is matched, so that a REVERTED
        // run records BOTH halves of the silent failure in one message: the
        // sender was told `Accepted`, and the fold dropped him anyway. Reading it
        // only after the match would let the red run report the wrong reply
        // without showing the drop that makes the reply a lie.
        let member_after = node
            .space_state(&space_id)
            .await
            .expect("Space resolves")
            .is_member(&bob_id);

        let reject = match outcome {
            DispatchOutcome::Rejected(info) => info,
            other => panic!(
                "an un-anchored re-join must be REFUSED, not accepted-then-dropped. \
                 Got {other:?}, and bob.is_member() == {member_after} immediately \
                 afterwards. `Accepted` together with `false` there IS the defect this \
                 leg closes, written out: the sender was told it landed and \
                 `derive_resolved` discarded it"
            ),
        };
        assert_eq!(
            reject.code, 3048,
            "and refused by the ANCHOR gate specifically — not 3047 (bob is a former \
             member, so the admission term admits him), not 3044, not a validation \
             failure; got {reject:?}"
        );
        assert_eq!(reject.name, "rejoin_not_anchored");

        // The refusal precedes `ingest_event`, so nothing was appended: a stored
        // join would have become the Space's sole tip.
        assert_eq!(
            node.dag_tips(&space_id).await,
            tips_before,
            "the store did not grow — the gate returns before `ingest_event`, so the \
             refused join was never appended"
        );
        assert!(
            !member_after,
            "bob is still departed — the refusal has teeth, it did not merely reply"
        );

        // DISCRIMINATOR (`V-2`) — the IDENTICAL re-join, anchored on bob's own
        // `membership.leave` (the Space's current tip). Everything else is the
        // same: same identity, same Space, same event type, same empty content,
        // same helper. Only the anchor moved.
        let outcome = submit_join(&node, &bob_key, &space_id).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "an ANCHORED re-join still lands. Got {outcome:?}. A `3048` here means the \
             gate is refusing every rejoin rather than every UN-ANCHORED one — which \
             would silently undo Leg G-1 while every other test in this file stayed \
             green, and is precisely why this half exists"
        );
        let after = node.space_state(&space_id).await.expect("Space resolves");
        let bob_rec = after
            .members
            .get(&idx(&bob_id))
            .expect("bob has a membership record after the anchored re-join");
        assert!(
            bob_rec.is_present(),
            "and he is PRESENT — an `Accepted` that left him departed would be the \
             reply lying in the other direction, which is the whole species this leg \
             is about"
        );
        assert!(
            bob_rec.left_at.is_none(),
            "with the departure boundary cleared (`D-154`①)"
        );
    }

    /// CONTROL (Leg G-2 `V-3`) — THE GATE IS REJOIN-ONLY, AND THE RESIDUE IS A
    /// TESTED BOUNDARY RATHER THAN AN UNEXAMINED ONE.
    ///
    /// A FIRST-TIME invited joiner submits a join anchored at the create root.
    /// That join is genuinely concurrent with her own `membership.invite` — both
    /// key `membership:{space}:{carol}` and neither is an ancestor of the other
    /// — so `conflicts_in_log` would return true for it. She is nevertheless NOT
    /// refused `3048`, because she has no membership record and the gate is
    /// guarded on `is_rejoin`.
    ///
    /// THAT GUARD IS THE POINT, AND IT COSTS SOMETHING. Refusing her too would
    /// close this residue — and would make the wire name `rejoin_not_anchored`
    /// narrower than the thing it describes, permanently, on a wire string.
    /// §3.1 of the runbook takes the name over the coverage. This test is where
    /// that trade is written down so a later reader finds a measurement instead
    /// of an omission.
    ///
    /// AND THE RESIDUE IS ASSERTED, NOT MERELY NAMED: carol is told `Accepted`
    /// and is NOT a member afterwards. The drop is deterministic rather than a
    /// tiebreak coin-flip — `algorithm.rs` Layer 1 has no invite-vs-join
    /// precedence, so resolution falls to Layer 4 (role within Space), where
    /// alice signs the invite as Owner and carol has no role yet, so the INVITE
    /// wins and the join is the loser the rebuild excludes.
    ///
    /// ⚠️ If this test ever goes red at the membership assertion, that is a
    /// change in RESOLUTION, not in this gate — read `algorithm.rs` before
    /// touching `dispatch_event`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_first_time_joiners_unanchored_join_is_not_refused_3048() {
        let node = spawn_in_process_node().await;

        let alice_key = keypair::generate(); // Space owner
        let carol_key = keypair::generate(); // FIRST-TIME joiner — never a member
        let carol_id = pubkey_uri(&carol_key);

        node.register_identity(&alice_key).await;
        node.register_identity(&carol_key).await;

        let space_ev = sign_event(
            build_space_create_event_with_admission(
                &alice_key,
                "Leg G-2 residue Space",
                None,
                1,
                &node.node_id,
                None,
                false,
                ADMISSION_INVITE,
            ),
            &alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        // The invite chains on the create root, so carol's root-anchored join
        // below is its SIBLING — same state key, neither an ancestor of the
        // other, which is exactly the concurrency `conflicts_in_log` detects.
        ingest_invite(&node, &alice_key, &space_id, &carol_id).await;
        let mid = node.space_state(&space_id).await.expect("Space resolves");
        assert!(
            mid.pending_invites.contains_key(&idx(&carol_id)),
            "precondition: carol holds a pending invite"
        );
        assert!(
            !mid.members.contains_key(&idx(&carol_id)),
            "precondition, and it is what makes this the CONTROL: carol has NO \
             membership record at all, so `is_rejoin` is false for her. A retained \
             departed record here would make this the subject case instead"
        );

        let root_anchor = vec![space_id.clone()];
        let outcome = submit_join_on(&node, &carol_key, &space_id, &root_anchor).await;
        if let DispatchOutcome::Rejected(ref info) = outcome {
            assert_ne!(
                info.code, 3048,
                "the gate must NOT reach a first-time joiner. A `3048` here means it \
                 was widened past rejoins and the wire name is now narrower than what \
                 the code refuses; got {info:?}"
            );
        }
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "today's outcome, unchanged: she is admitted at the reply. Got {outcome:?}"
        );

        // …and then silently dropped by resolution. This is the residue §3.1
        // names, measured rather than described.
        let after = node.space_state(&space_id).await.expect("Space resolves");
        assert!(
            !after.is_member(&carol_id),
            "THE RESIDUE: carol was told `Accepted` and is not a member. This leg does \
             NOT close this case, and the assertion is here so that the boundary is \
             known and tested rather than discovered later as a surprise"
        );
    }

    /// CONTROL (Leg G-2 `V-4`) — THE ORDERING THAT MAKES `conflicts_in_log`'s
    /// FAIL-OPEN UNREACHABLE AT THE GATE.
    ///
    /// `conflicts_in_log` returns `false` for an event carrying no `event_id`:
    /// `event_id_owned` yields `None` and it returns early. That is a check
    /// whose failure mode reads exactly like success — an un-anchored rejoin
    /// with no `event_id` would sail past the gate looking anchored.
    ///
    /// It is unreachable ONLY because `validate_event`'s step 8 already refused
    /// such an event, ~350 lines earlier in the same `dispatch_event`. The gate
    /// therefore contains NO second `event_id` check: re-implementing one would
    /// be a second source of truth for one fact (`D-067`). This test asserts the
    /// ordering instead, which is the same discipline that lets Leg G-1's ban
    /// clause stay absent — the omission is measured, not assumed.
    ///
    /// The subject is a bob who WOULD be refused `3048`: a retained departed
    /// member submitting a root-anchored join. Stripping his `event_id` must
    /// change WHICH gate refuses him, not WHETHER he is refused. An `Accepted`
    /// here would mean the fail-open is live on the answer path.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_join_with_no_event_id_is_refused_by_validation_before_the_anchor_gate() {
        let node = spawn_in_process_node().await;

        let alice_key = keypair::generate();
        let bob_key = keypair::generate();
        let bob_id = pubkey_uri(&bob_key);

        node.register_identity(&alice_key).await;
        node.register_identity(&bob_key).await;

        let space_ev = sign_event(
            build_space_create_event_with_admission(
                &alice_key,
                "Leg G-2 ordering-control Space",
                None,
                1,
                &node.node_id,
                None,
                false,
                ADMISSION_INVITE,
            ),
            &alice_key,
        );
        let space_id = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        // Bring bob to the exact state that produces `3048`.
        ingest_invite(&node, &alice_key, &space_id, &bob_id).await;
        assert!(
            matches!(submit_join(&node, &bob_key, &space_id).await, DispatchOutcome::Accepted { .. }),
            "precondition: the invited first join lands"
        );
        ingest_membership(&node, &bob_key, &space_id, EventType::MembershipLeave, json!({})).await;
        assert!(
            !node.space_state(&space_id).await.expect("Space resolves").is_member(&bob_id),
            "precondition: bob is departed, so the anchor gate is reachable for him"
        );

        // The same root-anchored re-join, signed and then stripped of its
        // `event_id`. Signed FIRST so nothing else about the event differs from
        // the one that earns `3048`.
        let root_anchor = vec![space_id.clone()];
        let mut ev = sign_event(
            space_level_ev(
                &bob_key,
                &space_id,
                &root_anchor,
                EventType::MembershipJoin,
                json!({}),
            ),
            &bob_key,
        );
        assert!(
            ev.event_id.is_some(),
            "precondition: `sign_event` stamps the canonical hash, so removing it \
             below is a real mutation and not a no-op"
        );
        ev.event_id = None;

        let outcome = node.submit_locally(ev).await;
        let reject = match outcome {
            DispatchOutcome::Rejected(info) => info,
            other => panic!(
                "an event with no `event_id` must be refused by VALIDATION. Got \
                 {other:?}. An `Accepted` here means `conflicts_in_log`'s fail-open is \
                 live on the answer path — it returns `false` for an id-less event, so \
                 an un-anchored rejoin would read as anchored"
            ),
        };
        assert_ne!(
            reject.code, 3048,
            "and refused BEFORE the anchor gate, not by it. A `3048` would mean the \
             gate ran on an event validation should already have refused, which is the \
             ordering this control exists to pin; got {reject:?}"
        );
        assert!(
            reject.reason.contains("event_id"),
            "the refusal names the missing `event_id` — step 8's `MissingEventId`, \
             which carries no wire code and lands as the 4000 generic fallback. Got \
             {reject:?}"
        );
    }

    /// THE DM CASE (Leg G-2 `V-7`) — a DM party who left, re-joining
    /// UN-ANCHORED, is refused `3048`.
    ///
    /// This is the case that matters most, and it is not merely another Space
    /// shape. A DM leaver has no other route back: both DM constructors pin
    /// `admission = invite` at fold time, her seeded invite was consumed by her
    /// first join, and `apply_invite` bars every DM invite as its first
    /// statement — so nobody can mint her a second one. Leg G-1 opened that door
    /// by admitting her as a former member; this gate is what stops the door
    /// from opening onto a reply that lies.
    ///
    /// Without it the DM would report her back and then resolve her away, in a
    /// two-person room where she is half the room and the other party has no way
    /// to correct it.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_dm_partys_unanchored_rejoin_is_refused_3048() {
        let node = spawn_in_process_node().await;

        let alice_key = keypair::generate(); // DM creator
        let bob_key = keypair::generate(); // the counterparty — leaves and returns
        let bob_id = pubkey_uri(&bob_key);

        node.register_identity(&alice_key).await;
        node.register_identity(&bob_key).await;

        let dm_ev = sign_event(
            build_dm_space_create_event(&alice_key, &bob_id, &node.node_id),
            &alice_key,
        );
        let space_id = event_id_str(&dm_ev);
        node.ingest(dm_ev).await;

        let before = node.space_state(&space_id).await.expect("the DM Space resolves");
        assert!(
            before.dm_constraints_active,
            "precondition: this is a DM, not an ordinary Space"
        );
        assert_eq!(
            before.admission, ADMISSION_INVITE,
            "precondition: the constructor PINS `invite` — nobody chose it and no owner \
             can open it, which is why the rejoin path is her only way back"
        );

        assert!(
            matches!(submit_join(&node, &bob_key, &space_id).await, DispatchOutcome::Accepted { .. }),
            "precondition: bob joins his DM on the seeded invite"
        );
        ingest_membership(&node, &bob_key, &space_id, EventType::MembershipLeave, json!({})).await;
        let left = node.space_state(&space_id).await.expect("Space resolves");
        assert!(!left.is_member(&bob_id), "precondition: bob has left the DM");
        assert!(
            !left.pending_invites.contains_key(&idx(&bob_id)),
            "precondition, and it is why this case has no alternative route: the seeded \
             invite is CONSUMED and `apply_invite` refuses to mint a DM invite"
        );

        let tips_before = node.dag_tips(&space_id).await;

        // SUBJECT — bob comes back un-anchored, the fresh-install shape.
        let root_anchor = vec![space_id.clone()];
        let outcome = submit_join_on(&node, &bob_key, &space_id, &root_anchor).await;
        let reject = match outcome {
            DispatchOutcome::Rejected(info) => info,
            other => panic!(
                "an un-anchored DM re-join must be REFUSED. Got {other:?}. An \
                 `Accepted` here tells bob he is back in a two-person room the fold is \
                 about to remove him from, and there is no second invite to correct it \
                 with"
            ),
        };
        assert_eq!(reject.code, 3048, "refused by the anchor gate; got {reject:?}");
        assert_eq!(reject.name, "rejoin_not_anchored");
        assert_eq!(
            node.dag_tips(&space_id).await,
            tips_before,
            "and nothing was appended — the gate returns before `ingest_event`"
        );
        assert!(
            !node.space_state(&space_id).await.expect("Space resolves").is_member(&bob_id),
            "bob is still out of the DM"
        );

        // And the anchored re-join still works — the DM door Leg G-1 opened is
        // not closed by this gate, only made honest.
        assert!(
            matches!(submit_join(&node, &bob_key, &space_id).await, DispatchOutcome::Accepted { .. }),
            "an ANCHORED DM re-join lands. A refusal here would mean this gate had \
             re-closed the one-way door Leg G-1 opened"
        );
        let after = node.space_state(&space_id).await.expect("Space resolves");
        assert!(after.is_member(&bob_id), "and he is back in the DM");
    }
}
