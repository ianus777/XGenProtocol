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

    /// Submit a space-level `membership.join` for `key` through the production
    /// local-client path and return what the SENDER receives.
    async fn submit_join(
        node: &InProcessNode,
        key: &ed25519_dalek::SigningKey,
        space_id: &str,
    ) -> DispatchOutcome {
        let tips = node.dag_tips(space_id).await;
        assert!(
            !tips.is_empty(),
            "the DAG has tips; step 10 rejects malformed prev_events BEFORE the \
             admission gate, and that failure would look like an admission refusal"
        );
        let ev = sign_event(
            space_level_ev(key, space_id, &tips, EventType::MembershipJoin, json!({})),
            key,
        );
        node.submit_locally(ev).await
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
}
