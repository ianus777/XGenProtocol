// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// DM Space promotion sequence handlers (spec 3.16.2–3.16.3).
//
// The Node drives the two-step approval sequence:
//   1. handle_propose  — validate propose, store DmProposal, identify delivery target.
//   2. handle_confirm  — validate confirm, produce the unsigned state.dm_promote Event.
//   3. handle_reject   — discard proposal, return proposer ID for notification.
//
// Actual WebSocket delivery and DAG commitment are done by the caller (xgen-node).

use ed25519_dalek::SigningKey;
use thiserror::Error;

use crate::wire::types::Event;

use super::state::{build_dm_promote_event, sign_event, SpaceState};

// ── Types ─────────────────────────────────────────────────────────────────────

/// In-memory record of a pending DM Space promotion proposal.
/// Not stored in the DAG — discarded when accepted, rejected, or the Node restarts.
#[derive(Debug, Clone)]
pub struct DmProposal {
    pub space_id: String,
    pub proposed_by: String,
    pub proposed_name: String,
    pub proposed_at: String,
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum PromoteError {
    #[error("sender is not a member of the DM Space")]
    SenderNotMember,
    #[error("confirm must be sent by the other member, not the proposer")]
    SenderIsProposer,
    #[error("no active proposal for this Space")]
    NoActiveProposal,
}

/// Returned by handle_propose.
pub struct ProposeResult {
    pub proposal: DmProposal,
    /// The other member's identity_id — caller delivers dm.promote_propose to this identity.
    pub deliver_to: String,
}

/// Returned by handle_confirm.
pub struct ConfirmResult {
    /// Unsigned state.dm_promote Event — caller must sign with the Node keypair.
    pub dm_promote_event: Event,
    /// Both member identity_ids — caller delivers state.dm_promote to each.
    pub deliver_to: Vec<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// Validate a dm.promote_propose message.
///
/// Returns a `ProposeResult` with the stored proposal and the other member's
/// identity_id (the caller delivers the propose message to them).
pub fn handle_propose(
    space_id: &str,
    proposer: &str,
    proposed_name: &str,
    proposed_at: &str,
    space_state: &SpaceState,
) -> Result<ProposeResult, PromoteError> {
    if !space_state.is_member(proposer) {
        return Err(PromoteError::SenderNotMember);
    }

    // Find the other member (DM Space always has exactly 2 members).
    let deliver_to = space_state
        .members
        .keys()
        .find(|id| id.as_str() != proposer)
        .ok_or(PromoteError::SenderNotMember)?
        .clone();

    let proposal = DmProposal {
        space_id: space_id.to_string(),
        proposed_by: proposer.to_string(),
        proposed_name: proposed_name.to_string(),
        proposed_at: proposed_at.to_string(),
    };

    Ok(ProposeResult { proposal, deliver_to })
}

/// Validate a dm.promote_confirm message, produce and sign state.dm_promote.
///
/// `node_key` is the Node's signing keypair — the Event is signed by the Node,
/// not by either member (spec 3.16.3 Step 4).
pub fn handle_confirm(
    confirmer: &str,
    space_state: &SpaceState,
    proposal: &DmProposal,
    node_key: &SigningKey,
    prev_events: Vec<String>,
    timestamp: &str,
) -> Result<ConfirmResult, PromoteError> {
    if !space_state.is_member(confirmer) {
        return Err(PromoteError::SenderNotMember);
    }
    if confirmer == proposal.proposed_by {
        return Err(PromoteError::SenderIsProposer);
    }

    let unsigned = build_dm_promote_event(
        node_key,
        &proposal.space_id,
        prev_events,
        &proposal.proposed_by,
        confirmer,
        &proposal.proposed_name,
        timestamp,
    );
    let dm_promote_event = sign_event(unsigned, node_key);

    let deliver_to = space_state.members.keys().cloned().collect();

    Ok(ConfirmResult { dm_promote_event, deliver_to })
}

/// Validate a dm.promote_reject message.
///
/// Returns the proposer's identity_id so the caller can notify them.
/// Discarding the stored proposal is the caller's responsibility
/// (remove from NodeRuntime::dm_proposals).
pub fn handle_reject(
    rejecter: &str,
    space_state: &SpaceState,
    proposal: &DmProposal,
) -> Result<String, PromoteError> {
    if !space_state.is_member(rejecter) {
        return Err(PromoteError::SenderNotMember);
    }
    if rejecter == proposal.proposed_by {
        return Err(PromoteError::SenderIsProposer);
    }
    Ok(proposal.proposed_by.clone())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        crypto::encoding,
        identity::keypair,
        space::state::{
            build_dm_space_create_event, build_membership_event, sign_event, SpaceState,
        },
        wire::types::EventType,
    };
    use serde_json::json;

    const HOME: &str = "xgen://pubkey/ed25519:NODE";
    const TS: &str = "2026-05-14T10:00:00.000Z";

    fn id_of(key: &ed25519_dalek::SigningKey) -> String {
        format!("xgen://pubkey/ed25519:{}", encoding::encode(key.verifying_key().as_bytes()))
    }

    fn make_dm_space() -> (SpaceState, String, ed25519_dalek::SigningKey, ed25519_dalek::SigningKey) {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_id = id_of(&bob);
        let create_ev = sign_event(build_dm_space_create_event(&alice, &bob_id, HOME), &alice);
        let space_id = create_ev.event_id.clone().unwrap();
        let (mut state, _, _) = SpaceState::from_dm_space_create(&create_ev, &alice).unwrap();
        // Bob joins.
        state.pending_invites.insert(bob_id.clone(), crate::space::membership::Role::Member);
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev).unwrap();
        (state, space_id, alice, bob)
    }

    #[test]
    fn promote_propose_stored_and_delivered() {
        let (state, space_id, alice, bob) = make_dm_space();
        let alice_id = id_of(&alice);
        let bob_id = id_of(&bob);

        let result = handle_propose(&space_id, &alice_id, "Our Project", TS, &state).unwrap();
        // Proposal is stored with correct fields.
        assert_eq!(result.proposal.proposed_by, alice_id);
        assert_eq!(result.proposal.proposed_name, "Our Project");
        // Delivered to the other member.
        assert_eq!(result.deliver_to, bob_id);
    }

    #[test]
    fn promote_confirm_produces_dm_promote_event() {
        let (state, space_id, alice, bob) = make_dm_space();
        let alice_id = id_of(&alice);
        let bob_id = id_of(&bob);
        let node_key = keypair::generate();

        let ProposeResult { proposal, .. } =
            handle_propose(&space_id, &alice_id, "Our Project", TS, &state).unwrap();
        let result = handle_confirm(&bob_id, &state, &proposal, &node_key, vec![], TS).unwrap();

        let ev = &result.dm_promote_event;
        assert_eq!(ev.event_type, EventType::StateDmPromote);
        assert_eq!(ev.content["proposed_by"].as_str(), Some(alice_id.as_str()));
        assert_eq!(ev.content["confirmed_by"].as_str(), Some(bob_id.as_str()));
        assert_eq!(ev.content["new_name"].as_str(), Some("Our Project"));
        assert!(ev.event_id.is_some(), "event must be signed and have an event_id");
        assert!(ev.signature.is_some());
    }

    #[test]
    fn promote_signed_by_node_not_member() {
        let (state, space_id, alice, bob) = make_dm_space();
        let alice_id = id_of(&alice);
        let bob_id = id_of(&bob);
        let node_key = keypair::generate();
        let node_id = id_of(&node_key);

        let ProposeResult { proposal, .. } =
            handle_propose(&space_id, &alice_id, "Our Project", TS, &state).unwrap();
        let result = handle_confirm(&bob_id, &state, &proposal, &node_key, vec![], TS).unwrap();

        // Sender of state.dm_promote must be the Node, not Alice or Bob.
        let ev = &result.dm_promote_event;
        assert_eq!(ev.sender, node_id);
        assert_ne!(ev.sender, alice_id);
        assert_ne!(ev.sender, bob_id);

        // Signature must verify against node's public key (embedded in sender field).
        assert!(crate::space::state::verify_event_signature(ev));
    }

    #[test]
    fn promote_reject_cancels_proposal() {
        let (state, space_id, alice, bob) = make_dm_space();
        let alice_id = id_of(&alice);
        let bob_id = id_of(&bob);

        let ProposeResult { proposal, .. } =
            handle_propose(&space_id, &alice_id, "Our Project", TS, &state).unwrap();

        // Bob rejects — returns proposer ID for notification.
        let notify = handle_reject(&bob_id, &state, &proposal).unwrap();
        assert_eq!(notify, alice_id);

        // Proposer cannot reject their own proposal.
        let err = handle_reject(&alice_id, &state, &proposal).unwrap_err();
        assert_eq!(err, PromoteError::SenderIsProposer);
    }
}
