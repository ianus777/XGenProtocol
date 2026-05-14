// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Space Migration state machine — both source and destination sides (spec 3.12.2–3.12.8).
//
// The migration state machine has two independent sides:
//   Source Node:       IDLE → NEGOTIATING → TRANSFERRING → VERIFYING → COMPLETE / FAILED
//   Destination Node:  IDLE → NEGOTIATING → TRANSFERRING → VERIFYING → COMPLETE / FAILED
//
// All handlers are pure functions. No I/O or async — the caller (xgen-node) is
// responsible for sending wire messages and committing events to the DAG.

use ed25519_dalek::SigningKey;
use serde_json::json;
use thiserror::Error;

use crate::{
    crypto::encoding,
    space::state::{sign_event, SpaceState},
    wire::types::{Event, EventType},
};

// ── Migration state ───────────────────────────────────────────────────────────

/// Shared migration state — both source and destination use this enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationState {
    Idle,
    Negotiating,
    Transferring,
    Verifying,
    Complete,
    /// Migration failed — reason is a human-readable string (not sent on the wire).
    Failed { reason: String },
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Error)]
pub enum MigrationError {
    /// 6001 — requester is not the Space owner.
    #[error("6001 migration_not_owner: only the Space owner may initiate migration")]
    SenderNotOwner,
    /// 6002 — destination already hosts this Space.
    #[error("6002 migration_already_hosting: destination already hosts this Space")]
    AlreadyHosting,
    /// 6003 — destination lacks storage for this Space.
    #[error("6003 migration_insufficient_storage: destination has insufficient storage")]
    InsufficientStorage,
    /// 6004 — destination runs an incompatible protocol version.
    #[error("6004 migration_version_incompatible: incompatible protocol version")]
    VersionIncompatible,
    /// 6005 — destination rejected by local policy.
    #[error("6005 migration_policy_rejected: destination rejected by policy")]
    PolicyRejected,
    /// 6006 — message received in unexpected state.
    #[error("6006 migration_wrong_state: unexpected message for current migration state")]
    WrongState,
}

impl MigrationError {
    pub fn error_code(&self) -> u32 {
        match self {
            Self::SenderNotOwner => 6001,
            Self::AlreadyHosting => 6002,
            Self::InsufficientStorage => 6003,
            Self::VersionIncompatible => 6004,
            Self::PolicyRejected => 6005,
            Self::WrongState => 6006,
        }
    }
}

// ── Result types ──────────────────────────────────────────────────────────────

/// Parameters for the `migration.propose` message sent from source to destination.
#[derive(Debug)]
pub struct ProposeParams {
    pub space_id: String,
    pub owner_id: String,
    pub event_count: usize,
    pub estimated_size_bytes: usize,
}

/// Returned by `handle_verified` — what the source Node must do for cutover.
pub struct CutoverResult {
    /// state.space_migrate Event — caller must commit to the DAG and transfer to destination.
    pub space_migrate_event: Event,
    /// Identity IDs of all Space members — caller sends transport.redirect to each.
    pub member_ids: Vec<String>,
}

/// Destination's response to a migration.propose.
#[derive(Debug, PartialEq, Eq)]
pub enum DestinationResponse {
    Accept,
    Reject { reason: &'static str },
}

// ── Source side handlers ──────────────────────────────────────────────────────

/// Handle a `migration.request` from a Space member.
///
/// Returns `ProposeParams` on success — the caller uses these to construct the
/// `migration.propose` message sent to the destination Node.
///
/// Returns `Err(SenderNotOwner)` if `requester_id` is not the Space owner.
pub fn handle_migration_request(
    requester_id: &str,
    space_state: &SpaceState,
    event_count: usize,
    estimated_size_bytes: usize,
) -> Result<ProposeParams, MigrationError> {
    if requester_id != space_state.owner_id {
        return Err(MigrationError::SenderNotOwner);
    }
    Ok(ProposeParams {
        space_id: space_state.space_id.clone(),
        owner_id: requester_id.to_string(),
        event_count,
        estimated_size_bytes,
    })
}

/// Handle a `migration.reject` received by the source Node.
///
/// Maps the wire `reason` string to the corresponding `MigrationError`.
/// The source Node returns to `IDLE`; the Space continues unchanged.
pub fn handle_migration_reject(reason: &str) -> MigrationError {
    match reason {
        "already_hosting" => MigrationError::AlreadyHosting,
        "insufficient_storage" => MigrationError::InsufficientStorage,
        "version_incompatible" => MigrationError::VersionIncompatible,
        _ => MigrationError::PolicyRejected,
    }
}

/// Build the `state.space_migrate` Event and collect member IDs for `transport.redirect`.
///
/// Called by the source Node after receiving `migration.verified` from the destination.
/// The returned `CutoverResult.space_migrate_event` must be committed to the Space DAG
/// and transferred to the destination Node before the redirect notifications are sent.
pub fn handle_verified(
    space_state: &SpaceState,
    node_key: &SigningKey,
    destination_node_id: &str,
    destination_node_url: &str,
    prev_events: Vec<String>,
    timestamp: &str,
) -> CutoverResult {
    let event = build_space_migrate_event(
        node_key,
        &space_state.space_id,
        destination_node_id,
        destination_node_url,
        prev_events,
        timestamp,
    );
    let member_ids = space_state.members.keys().cloned().collect();
    CutoverResult { space_migrate_event: event, member_ids }
}

/// Build and sign a `state.space_migrate` Event.
/// Signed by the source Node's keypair (not by any member).
pub fn build_space_migrate_event(
    node_key: &SigningKey,
    space_id: &str,
    destination_node_id: &str,
    destination_node_url: &str,
    prev_events: Vec<String>,
    timestamp: &str,
) -> Event {
    let source_node_id = format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(node_key.verifying_key().as_bytes())
    );
    let unsigned = Event {
        protocol_version: "0.1".to_string(),
        event_type: EventType::StateSpaceMigrate,
        sender: source_node_id.clone(),
        space_id: space_id.to_string(),
        room_id: String::new(),
        prev_events,
        content: json!({
            "source_node_id": source_node_id,
            "destination_node_id": destination_node_id,
            "destination_node_url": destination_node_url,
            "migrated_at": timestamp,
        }),
        meta_atts: None,
        timestamp: timestamp.to_string(),
        event_id: None,
        signature: None,
    };
    sign_event(unsigned, node_key)
}

// ── Destination side handlers ─────────────────────────────────────────────────

/// Evaluate a `migration.propose` received by the destination Node.
///
/// Phase 2 implementation: always accepts unless the Space is already hosted here.
/// `existing_space_ids` is the set of space_ids currently hosted on this Node.
pub fn handle_migration_propose(
    space_id: &str,
    existing_space_ids: &[String],
) -> DestinationResponse {
    if existing_space_ids.iter().any(|id| id == space_id) {
        return DestinationResponse::Reject { reason: "already_hosting" };
    }
    DestinationResponse::Accept
}

/// Append a received event batch to the destination's in-progress transfer buffer.
///
/// Returns the total number of Events received so far for this migration.
pub fn accept_event_batch(received: &mut Vec<Event>, batch: Vec<Event>) -> usize {
    received.extend(batch);
    received.len()
}

/// Abort a migration in progress — destination discards all received Events.
///
/// Caller must also transition the destination's `MigrationState` back to `Idle`.
pub fn abort_destination(received: &mut Vec<Event>) {
    received.clear();
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        crypto::encoding,
        identity::keypair,
        space::state::{
            build_membership_event, build_space_create_event, sign_event, SpaceState,
        },
        wire::types::EventType,
    };
    use serde_json::json;

    const HOME: &str = "xgen://pubkey/ed25519:HOME";
    const TS: &str = "2026-05-14T10:00:00.000Z";

    fn id_of(key: &SigningKey) -> String {
        format!("xgen://pubkey/ed25519:{}", encoding::encode(key.verifying_key().as_bytes()))
    }

    /// Create a Space with owner (alice) and one member (bob) already joined.
    fn make_space() -> (SpaceState, String, SigningKey, SigningKey) {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let bob_id = id_of(&bob);

        let create_ev = sign_event(build_space_create_event(&alice, "Test Space", None, 1, HOME), &alice);
        let space_id = create_ev.event_id.clone().unwrap();
        let mut state = SpaceState::from_space_create(&create_ev).unwrap();

        // Invite and join bob.
        state.pending_invites.insert(bob_id.clone(), crate::space::membership::Role::Member);
        let join_ev = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        state.apply_event(&join_ev).unwrap();
        (state, space_id, alice, bob)
    }

    // ── Source side ───────────────────────────────────────────────────────────

    #[test]
    fn migration_requires_owner_auth() {
        let (state, _, _, bob) = make_space();
        let bob_id = id_of(&bob);

        let err = handle_migration_request(&bob_id, &state, 100, 1024).unwrap_err();
        assert_eq!(err, MigrationError::SenderNotOwner);
        assert_eq!(err.error_code(), 6001);
    }

    #[test]
    fn migration_propose_sent_to_destination() {
        let (state, _, alice, _) = make_space();
        let alice_id = id_of(&alice);

        let params = handle_migration_request(&alice_id, &state, 250, 102400).unwrap();
        assert_eq!(params.space_id, state.space_id);
        assert_eq!(params.owner_id, alice_id);
        assert_eq!(params.event_count, 250);
        assert_eq!(params.estimated_size_bytes, 102400);
    }

    #[test]
    fn destination_reject_closes_migration() {
        // All rejection reasons map to MigrationError.
        let err = handle_migration_reject("insufficient_storage");
        assert_eq!(err, MigrationError::InsufficientStorage);
        assert_eq!(err.error_code(), 6003);

        let err = handle_migration_reject("already_hosting");
        assert_eq!(err, MigrationError::AlreadyHosting);

        let err = handle_migration_reject("version_incompatible");
        assert_eq!(err, MigrationError::VersionIncompatible);

        // Unknown reason → policy rejected.
        let err = handle_migration_reject("other_reason");
        assert_eq!(err, MigrationError::PolicyRejected);
    }

    #[test]
    fn cutover_produces_space_migrate_event() {
        let (state, space_id, _, _) = make_space();
        let node_key = keypair::generate();
        let node_id = id_of(&node_key);
        let dst_id = "xgen://pubkey/ed25519:DST";
        let dst_url = "wss://dst.example.com/xgen";

        let result = handle_verified(&state, &node_key, dst_id, dst_url, vec![], TS);

        let ev = &result.space_migrate_event;
        assert_eq!(ev.event_type, EventType::StateSpaceMigrate);
        assert_eq!(ev.space_id, space_id);
        assert_eq!(ev.sender, node_id);
        assert_eq!(ev.content["destination_node_id"].as_str(), Some(dst_id));
        assert_eq!(ev.content["destination_node_url"].as_str(), Some(dst_url));
        assert!(ev.event_id.is_some(), "event must be signed");
        assert!(ev.signature.is_some());

        // member_ids must contain both alice and bob.
        assert_eq!(result.member_ids.len(), 2);
    }

    // ── Destination side ──────────────────────────────────────────────────────

    #[test]
    fn destination_accepts_new_space() {
        let space_id = "xgen://hash/sha256:abc123";
        let existing: Vec<String> = vec![];
        let resp = handle_migration_propose(space_id, &existing);
        assert_eq!(resp, DestinationResponse::Accept);
    }

    #[test]
    fn destination_rejects_already_hosted_space() {
        let space_id = "xgen://hash/sha256:abc123";
        let existing = vec![space_id.to_string()];
        let resp = handle_migration_propose(space_id, &existing);
        assert_eq!(resp, DestinationResponse::Reject { reason: "already_hosting" });
    }

    #[test]
    fn abort_discards_destination_state() {
        let node_key = keypair::generate();
        let mut received: Vec<Event> = Vec::new();

        // Simulate receiving a batch.
        let ev = sign_event(build_space_create_event(&node_key, "Space", None, 1, HOME), &node_key);
        let count = accept_event_batch(&mut received, vec![ev]);
        assert_eq!(count, 1);

        // Abort clears all received events.
        abort_destination(&mut received);
        assert!(received.is_empty());
    }

    // ── End-to-end ────────────────────────────────────────────────────────────

    #[test]
    fn full_migration_end_to_end() {
        use crate::migration::{
            transfer::{batch_events, compute_batch_hash, identify_tail},
            verification::verify_transfer,
        };
        use std::collections::HashSet;

        let (state, space_id, alice, _) = make_space();
        let alice_id = id_of(&alice);
        let node_key = keypair::generate();
        let dst_key = keypair::generate();
        let dst_id = id_of(&dst_key);
        let dst_url = "wss://dst.example.com/xgen";

        // ── Source: request ──
        let params = handle_migration_request(&alice_id, &state, 3, 1024).unwrap();
        assert_eq!(params.space_id, space_id);

        // ── Destination: propose → accept ──
        let resp = handle_migration_propose(&space_id, &[]);
        assert_eq!(resp, DestinationResponse::Accept);

        // ── Source: build main transfer events + snapshot ──
        let ev1 = sign_event(build_space_create_event(&alice, "S1", None, 1, HOME), &alice);
        let ev2 = sign_event(build_space_create_event(&alice, "S2", None, 1, HOME), &alice);
        let main_events = vec![ev1.clone(), ev2.clone()];

        let snapshot_ids: HashSet<String> = main_events
            .iter()
            .filter_map(|e| e.event_id.clone())
            .collect();

        // ── Source: batch and compute hash ──
        let batches = batch_events(main_events.clone());
        assert_eq!(batches.len(), 1);
        let _batch_hash = compute_batch_hash(&batches[0]);

        // ── Destination: receive batch ──
        let mut dst_received: Vec<Event> = Vec::new();
        accept_event_batch(&mut dst_received, batches[0].clone());

        // ── Tail: new event produced during transfer ──
        let ev3 = sign_event(build_space_create_event(&alice, "S3", None, 1, HOME), &alice);
        let all_events = vec![ev1.clone(), ev2.clone(), ev3.clone()];
        let tail = identify_tail(&all_events, &snapshot_ids);
        assert_eq!(tail.len(), 1);

        // ── Destination: receive tail ──
        let tail_batch: Vec<Event> = tail.into_iter().cloned().collect();
        let total = accept_event_batch(&mut dst_received, tail_batch);
        assert_eq!(total, 3);

        // ── Source declares transfer complete with tips ──
        let total_events = 3usize;
        let tip_ids: Vec<String> = all_events
            .iter()
            .filter_map(|e| e.event_id.clone())
            .collect();

        // ── Destination: verify ──
        verify_transfer(dst_received.len(), &tip_ids, total_events, &tip_ids).unwrap();

        // ── Source: cutover ──
        let cutover = handle_verified(&state, &node_key, &dst_id, dst_url, vec![], TS);
        let ev = &cutover.space_migrate_event;
        assert_eq!(ev.event_type, EventType::StateSpaceMigrate);
        assert_eq!(ev.content["destination_node_id"].as_str(), Some(dst_id.as_str()));
        assert!(ev.event_id.is_some());
    }
}
