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
    };
    use crate::{
        identity::keypair,
        node::runtime::DispatchOutcome,
        space::state::{
            build_space_create_event, build_space_create_event_with_admission, sign_event,
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
}
