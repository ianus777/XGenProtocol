// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M-SPACE-ADMISSION Leg C test 7 — the composition assertion.
//! Runbook: `tasks/RUNBOOK_SPACE_ADMISSION_LEG_C.md` v1.2 (LOCKED), §4.8 item 7.
//!
//! **Why this test exists, and why it is not a seventh unit test.** The defect
//! Leg C is built to refuse (`M-1`) is a COMPOSITION failure, not a logic
//! failure. Every individual piece can be correct — the applier refuses, the
//! permission predicate is right, the wire code is mapped — while the sender
//! still receives `Accepted`, because the applier's error is discarded at every
//! production call site (`let _ = ...apply_event(...)` at `runtime.rs:867`,
//! `derive.rs:231`, `ai_service.rs:553`). The codebase already has a name for
//! the result: `runtime.rs:1505-1522` calls it *the reply lied*.
//!
//! The other six tests would go GREEN over exactly that. This one asserts on the
//! `DispatchOutcome` the SENDER receives, through `dispatch_event`, end to end.
//!
//! **The control is load-bearing.** A `Rejected` outcome on its own is equally
//! consistent with the event being malformed, mis-chained, or refused by any of
//! the many gates between dispatch entry and the applier. The owner's identical
//! event must be ACCEPTED and must actually move the value — otherwise this test
//! cannot tell an owner-only gate from a closed door.

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
            state::{build_space_create_event, sign_event},
        },
        wire::types::{Event, EventType, ADMISSION_INVITE, ADMISSION_OPEN},
    };

    /// Build an unsigned space-level Event chained on the given tips.
    ///
    /// Deliberately NOT `state::build_membership_event`: that helper emits
    /// `prev_events: vec![]`, which is fine for the `state.rs` unit tests (they
    /// call `apply_event` directly and never touch the DAG) and is a structural
    /// violation on this path — `ingest_event` will not apply an unchained
    /// non-root event, so the invite and join would silently fail to land and the
    /// preconditions below would report bob as a non-member.
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

    /// Build an unsigned `state.space_admission` Event chained on the given tips.
    fn admission_event(
        key: &ed25519_dalek::SigningKey,
        space_id: &str,
        tips: &[String],
        value: &str,
    ) -> Event {
        space_level_ev(
            key,
            space_id,
            tips,
            EventType::StateSpaceAdmission,
            json!({ "admission": value }),
        )
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn non_owner_admission_change_is_rejected_to_the_sender_end_to_end() {
        let node = spawn_in_process_node().await;

        let alice_key = keypair::generate(); // Space owner
        let bob_key = keypair::generate(); // ADMIN — a privileged non-owner
        let alice_id = pubkey_uri(&alice_key);
        let bob_id = pubkey_uri(&bob_key);

        // Both registered: Step 11 HeldPends an unregistered sender universally,
        // and a `HeldPending` outcome would prove nothing about admission.
        node.register_identity(&alice_key).await;
        node.register_identity(&bob_key).await;

        // Setup via `ingest` only. The subject events are the ONLY ones that go
        // through `submit_locally`, which is the path under test.
        let space_ev = sign_event(
            build_space_create_event(
                &alice_key,
                "Leg C Space",
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

        // Each setup event is chained on the Space's CURRENT tips, re-read after
        // the previous ingest. An unchained non-root event is not applied.
        let tips = node.dag_tips(&space_id).await;
        let invite = sign_event(
            space_level_ev(
                &alice_key,
                &space_id,
                &tips,
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "admin" }),
            ),
            &alice_key,
        );
        node.ingest(invite).await;

        let tips = node.dag_tips(&space_id).await;
        let join = sign_event(
            space_level_ev(
                &bob_key,
                &space_id,
                &tips,
                EventType::MembershipJoin,
                json!({}),
            ),
            &bob_key,
        );
        node.ingest(join).await;

        // Preconditions, asserted rather than assumed.
        let before = node.space_state(&space_id).await.expect(
            "the Space resolves; otherwise `space not found` would reject the subject \
             and the failure would read as if admission refused it",
        );
        assert!(
            !before.dm_constraints_active,
            "this is an ORDINARY Space — the DM bar must not be what refuses bob, or \
             the test would be measuring the wrong branch"
        );
        assert_eq!(
            before.admission, ADMISSION_OPEN,
            "it starts open, so a later `invite` proves the owner's change landed"
        );
        assert_eq!(
            before.member_role(&alice_id),
            Some(&Role::Owner),
            "alice holds the owner role"
        );
        assert_eq!(
            before.member_role(&bob_id),
            Some(&Role::Admin),
            "bob is an ADMIN — a refusal cannot be explained by him being a non-member, \
             and step 11 would have rejected a non-member before the permission check \
             ever ran"
        );

        let tips = node.dag_tips(&space_id).await;
        assert!(
            !tips.is_empty(),
            "the DAG has tips; step 10 rejects malformed prev_events BEFORE the \
             permission check, and that failure would look like a refusal"
        );

        // SUBJECT — bob, an admin, tries to change admission.
        let bob_try = sign_event(
            admission_event(&bob_key, &space_id, &tips, ADMISSION_INVITE),
            &bob_key,
        );
        let outcome = node.submit_locally(bob_try).await;

        // THE ASSERTION THIS WHOLE TEST EXISTS FOR: what the SENDER receives.
        let reject = match outcome {
            DispatchOutcome::Rejected(info) => info,
            other => panic!(
                "a non-owner's admission change must be REJECTED to its sender. Got \
                 {other:?}. If this is `Accepted`, the permission check is applier-only \
                 and its error is being discarded — the reply lied."
            ),
        };
        assert_eq!(
            reject.code, 4000,
            "a plain non-owner refusal is an ordinary permission failure: 4000 generic, \
             not a bespoke code"
        );
        assert_eq!(reject.name, "generic");

        // And it did not merely fail to reply — the value did not move.
        let after_reject = node
            .space_state(&space_id)
            .await
            .expect("Space still resolves");
        assert_eq!(
            after_reject.admission, ADMISSION_OPEN,
            "the refused change did NOT take effect"
        );

        // CONTROL — the OWNER's identical event must be accepted and must move the
        // value. Without this the rejection above is consistent with the event
        // being malformed rather than with the gate working.
        let tips = node.dag_tips(&space_id).await;
        let alice_try = sign_event(
            admission_event(&alice_key, &space_id, &tips, ADMISSION_INVITE),
            &alice_key,
        );
        let outcome = node.submit_locally(alice_try).await;
        assert!(
            matches!(outcome, DispatchOutcome::Accepted { .. }),
            "the OWNER's identical event must be accepted, or this test cannot \
             distinguish an owner-only gate from a closed door; got {outcome:?}"
        );

        let after = node
            .space_state(&space_id)
            .await
            .expect("Space still resolves");
        assert_eq!(
            after.admission, ADMISSION_INVITE,
            "and the owner's change actually landed — an `Accepted` whose value never \
             moved would be the same defect wearing the other face"
        );
    }
}
