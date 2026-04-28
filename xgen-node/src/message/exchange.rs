// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Message exchange — pipeline steps 8–13 and event acceptance (spec 3.2.6).
//
// Steps 1–7 (structural) are in wire/validation.rs.
// This module adds the crypto and state-dependent steps:
//
//   8.  event_id matches hash of canonical content
//   9.  all prev_events are known to this Node (→ hold pending if not)
//   10. no DAG structural violation (self-reference, root-type rules, fanin limit)
//   11. sender is a registered Identity and a Space/Room member
//   12. signature verifies against sender's public key
//   13. sender has permission to produce this EventType in this Room

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use serde_json::json;
use thiserror::Error;

use crate::{
    crypto::{encoding, hashing},
    dag::{
        graph::{DagGraph, MAX_PREV_EVENTS},
        store::EventStore,
    },
    identity::registry::IdentityRegistry,
    space::{
        membership::{can_ban, can_change_space_info, can_invite, can_kick},
        state::{verify_event_signature, SpaceState},
    },
    wire::{
        canonical::canonical_event_bytes,
        types::{Event, EventType},
    },
};

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExchangeError {
    #[error("step 8: event_id does not match canonical content hash")]
    EventIdMismatch,

    #[error("step 9: unknown prev_events — event held pending")]
    HeldPending(Vec<String>),

    #[error("step 10: DAG structural violation — {0}")]
    DagError(String),

    #[error("step 11: sender is not a registered Identity")]
    UnknownSender,

    #[error("step 11: sender is not a Space member")]
    NotASpaceMember,

    #[error("step 11: sender is not a member of room '{0}'")]
    NotARoomMember(String),

    #[error("step 12: signature verification failed")]
    SignatureFailure,

    #[error("step 13: permission denied for {0}")]
    PermissionDenied(String),

    #[error("event is missing event_id")]
    MissingEventId,
}

// ── Validation pipeline steps 8–13 ───────────────────────────────────────────

/// Run pipeline steps 8–13 on an Event that has already passed steps 1–7.
///
/// Does NOT mutate `store` or `graph` — the caller must call `accept_event`
/// (or insert manually) after this returns `Ok`.
///
/// Returns `HeldPending(missing_ids)` when prev_events reference unknown Events;
/// the caller should buffer the event and request the missing predecessors.
pub fn validate_steps_8_13(
    event: &Event,
    space: &SpaceState,
    id_registry: &IdentityRegistry,
    store: &EventStore,
) -> Result<(), ExchangeError> {
    // Step 8 — event_id matches canonical content hash.
    let event_id = event.event_id.as_deref().ok_or(ExchangeError::MissingEventId)?;
    let v = serde_json::to_value(event).expect("Event is always serialisable");
    let canonical = canonical_event_bytes(&v);
    let expected_id = hashing::hash_uri(&canonical);
    if event_id != expected_id {
        return Err(ExchangeError::EventIdMismatch);
    }

    // Step 9 — all prev_events known to this Node.
    let unknown: Vec<String> = event
        .prev_events
        .iter()
        .filter(|id| !store.contains(id.as_str()))
        .cloned()
        .collect();
    if !unknown.is_empty() {
        return Err(ExchangeError::HeldPending(unknown));
    }

    // Step 10 — DAG structural rules (no mutation).
    validate_dag_structure(event)?;

    // Step 11 — sender is a registered Identity and a Space/Room member.
    let sender = &event.sender;
    if !id_registry.contains(sender) {
        return Err(ExchangeError::UnknownSender);
    }
    if !space.is_member(sender) {
        return Err(ExchangeError::NotASpaceMember);
    }
    if !event.room_id.is_empty() && !space.is_room_member(sender, &event.room_id) {
        return Err(ExchangeError::NotARoomMember(event.room_id.clone()));
    }

    // Step 12 — signature verifies against the sender's embedded public key.
    if !verify_event_signature(event) {
        return Err(ExchangeError::SignatureFailure);
    }

    // Step 13 — sender has permission to produce this EventType in this Room.
    check_permission(event, space)?;

    Ok(())
}

/// Accept an Event: run all 13 pipeline steps, then insert into the DAG and store.
///
/// Callers are responsible for propagating the event to federated Nodes after
/// this returns `Ok`.
pub fn accept_event(
    event: Event,
    space: &SpaceState,
    id_registry: &IdentityRegistry,
    store: &mut EventStore,
    graph: &mut DagGraph,
) -> Result<(), ExchangeError> {
    validate_steps_8_13(&event, space, id_registry, store)?;
    graph
        .add_event(&event, store)
        .map_err(|e| ExchangeError::DagError(e.to_string()))?;
    store
        .insert(event)
        .map_err(|e| ExchangeError::DagError(e.to_string()))?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn validate_dag_structure(event: &Event) -> Result<(), ExchangeError> {
    let id = event.event_id.as_deref().ok_or(ExchangeError::MissingEventId)?;

    let is_root = matches!(
        event.event_type,
        EventType::StateSpaceCreate | EventType::StateDmSpaceCreate | EventType::StateRoomCreate
    );

    if is_root && !event.prev_events.is_empty() {
        return Err(ExchangeError::DagError(
            "root event type must have empty prev_events".to_string(),
        ));
    }
    if !is_root && event.prev_events.is_empty() {
        return Err(ExchangeError::DagError(
            "non-root event must reference at least one predecessor".to_string(),
        ));
    }
    if event.prev_events.len() > MAX_PREV_EVENTS {
        return Err(ExchangeError::DagError(format!(
            "prev_events has {} entries; maximum is {}",
            event.prev_events.len(),
            MAX_PREV_EVENTS
        )));
    }
    if event.prev_events.iter().any(|p| p == id) {
        return Err(ExchangeError::DagError(
            "self-reference in prev_events".to_string(),
        ));
    }

    Ok(())
}

fn check_permission(event: &Event, space: &SpaceState) -> Result<(), ExchangeError> {
    let sender = &event.sender;
    match &event.event_type {
        // Message events require room membership only — verified in step 11.
        EventType::MessageText
        | EventType::MessageFile
        | EventType::MessageReaction
        | EventType::MessageDelete => Ok(()),

        // State updates require Admin or above.
        EventType::StateRoomUpdate | EventType::StateSpaceUpdate => {
            let role = space.member_role(sender);
            if role.map(can_change_space_info).unwrap_or(false) {
                Ok(())
            } else {
                Err(ExchangeError::PermissionDenied(
                    event.event_type.as_str().to_string(),
                ))
            }
        }

        // Membership operations require the appropriate role.
        EventType::MembershipInvite => {
            let role = space.member_role(sender);
            if role.map(can_invite).unwrap_or(false) {
                Ok(())
            } else {
                Err(ExchangeError::PermissionDenied(
                    event.event_type.as_str().to_string(),
                ))
            }
        }
        EventType::MembershipKick => {
            let role = space.member_role(sender);
            if role.map(can_kick).unwrap_or(false) {
                Ok(())
            } else {
                Err(ExchangeError::PermissionDenied(
                    event.event_type.as_str().to_string(),
                ))
            }
        }
        EventType::MembershipBan => {
            let role = space.member_role(sender);
            if role.map(can_ban).unwrap_or(false) {
                Ok(())
            } else {
                Err(ExchangeError::PermissionDenied(
                    event.event_type.as_str().to_string(),
                ))
            }
        }

        // All other event types: permitted for any Space member.
        _ => Ok(()),
    }
}

// ── Event builder ─────────────────────────────────────────────────────────────

/// Build an unsigned `message.text` Event.
/// Call `space::state::sign_event` to compute event_id and signature.
pub fn build_message_text_event(
    key: &SigningKey,
    space_id: &str,
    room_id: &str,
    prev_events: Vec<String>,
    text: &str,
) -> Event {
    Event::new(
        EventType::MessageText,
        format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        ),
        room_id.to_string(),
        space_id.to_string(),
        prev_events,
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        json!({ "text": text }),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{SecondsFormat, Utc};
    use crate::{
        dag::{graph::DagGraph, store::EventStore},
        identity::{
            keypair,
            registry::{IdentityRecord, IdentityRegistry},
        },
        space::state::{
            build_room_create_event, build_space_create_event, sign_event,
            SpaceState,
        },
        wire::types::{Event, EventType},
    };
    use serde_json::json;

    const HOME: &str = "xgen://pubkey/ed25519:NODE";

    // ── Test fixtures ─────────────────────────────────────────────────────────

    fn make_identity_record(key: &SigningKey, home: &str) -> IdentityRecord {
        let id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        );
        IdentityRecord {
            identity_id: id,
            display_name: None,
            registered_at: "2026-04-28T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: home.to_string(),
            update_version: 0,
        }
    }

    /// Build a membership event with explicit prev_events (DAG-aware variant for tests).
    fn membership_ev_with_prev(
        key: &SigningKey,
        space_id: &str,
        room_id: &str,
        event_type: EventType,
        prev_events: Vec<String>,
        content: serde_json::Value,
    ) -> Event {
        Event::new(
            event_type,
            format!(
                "xgen://pubkey/ed25519:{}",
                encoding::encode(key.verifying_key().as_bytes())
            ),
            room_id.to_string(),
            space_id.to_string(),
            prev_events,
            Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            content,
        )
    }

    /// Build the canonical set of setup events for a two-user Space, returned in
    /// insertion order so any node can replay them deterministically.
    ///
    /// DAG chain (linear, single tip at the end):
    ///   state.space_create (root)
    ///   state.room_create (root, but linked into chain via invite's prev)
    ///   membership.invite  prev=[space_id, room_id]
    ///   membership.join (space)  prev=[invite_id]
    ///   membership.join (room)   prev=[join_space_id]  ← tip
    fn build_setup_events(
        alice_key: &SigningKey,
        bob_key: &SigningKey,
    ) -> (Vec<Event>, String, String) {
        let mut events = Vec::new();

        let space_ev = sign_event(
            build_space_create_event(alice_key, "Test Space", None, 1, HOME),
            alice_key,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        events.push(space_ev);

        let room_ev = sign_event(
            build_room_create_event(alice_key, &space_id, "general", None),
            alice_key,
        );
        let room_id = room_ev.event_id.clone().unwrap();
        events.push(room_ev);

        let bob_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(bob_key.verifying_key().as_bytes())
        );
        // invite prev = [space_id, room_id] — merges the two roots into one chain.
        let invite_ev = sign_event(
            membership_ev_with_prev(
                alice_key,
                &space_id,
                "",
                EventType::MembershipInvite,
                vec![space_id.clone(), room_id.clone()],
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            alice_key,
        );
        let invite_id = invite_ev.event_id.clone().unwrap();
        events.push(invite_ev);

        let join_space_ev = sign_event(
            membership_ev_with_prev(
                bob_key,
                &space_id,
                "",
                EventType::MembershipJoin,
                vec![invite_id],
                json!({}),
            ),
            bob_key,
        );
        let join_space_id = join_space_ev.event_id.clone().unwrap();
        events.push(join_space_ev);

        let join_room_ev = sign_event(
            membership_ev_with_prev(
                bob_key,
                &space_id,
                &room_id,
                EventType::MembershipJoin,
                vec![join_space_id],
                json!({}),
            ),
            bob_key,
        );
        events.push(join_room_ev);

        (events, space_id, room_id)
    }

    /// Replay a set of ordered events into a store + graph, building SpaceState as we go.
    fn replay_events(
        events: &[Event],
        store: &mut EventStore,
        graph: &mut DagGraph,
    ) -> SpaceState {
        let mut space: Option<SpaceState> = None;
        for ev in events {
            graph.add_event(ev, store).unwrap();
            store.insert(ev.clone()).unwrap();
            match &ev.event_type {
                EventType::StateSpaceCreate => {
                    space = Some(SpaceState::from_space_create(ev).unwrap());
                }
                _ => {
                    if let Some(ref mut s) = space {
                        let _ = s.apply_event(ev);
                    }
                }
            }
        }
        space.expect("space_create must be in events")
    }

    /// Seed an EventStore and DagGraph. Returns (SpaceState, IdentityRegistry, space_id, room_id, tip_id).
    fn setup_node(
        alice_key: &SigningKey,
        bob_key: &SigningKey,
        store: &mut EventStore,
        graph: &mut DagGraph,
    ) -> (SpaceState, IdentityRegistry, String, String, String) {
        let (events, space_id, room_id) = build_setup_events(alice_key, bob_key);
        let tip_id = events.last().unwrap().event_id.clone().unwrap();
        let space = replay_events(&events, store, graph);

        let mut registry = IdentityRegistry::new();
        registry.register(make_identity_record(alice_key, HOME)).unwrap();
        registry.register(make_identity_record(bob_key, HOME)).unwrap();

        (space, registry, space_id, room_id, tip_id)
    }

    // ── Step 8 tests ──────────────────────────────────────────────────────────

    #[test]
    fn step8_valid_event_id_passes() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id], "hello"),
            &alice,
        );
        assert!(validate_steps_8_13(&ev, &space, &registry, &store).is_ok());
    }

    #[test]
    fn step8_wrong_event_id_rejected() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let mut ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id], "hello"),
            &alice,
        );
        ev.event_id = Some("xgen://hash/sha256:TAMPERED".to_string());

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::EventIdMismatch)
        ));
    }

    // ── Step 9 tests ──────────────────────────────────────────────────────────

    #[test]
    fn step9_unknown_prev_event_held_pending() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, _) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        // Reference an event that is not in the store.
        let unknown_id = "xgen://hash/sha256:UNKNOWN_PREV";
        let ev = sign_event(
            build_message_text_event(
                &alice,
                &space_id,
                &room_id,
                vec![unknown_id.to_string()],
                "hello",
            ),
            &alice,
        );

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::HeldPending(_))
        ));
    }

    // ── Step 11 tests ─────────────────────────────────────────────────────────

    #[test]
    fn step11_unregistered_sender_rejected() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let charlie = keypair::generate(); // not registered
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let ev = sign_event(
            build_message_text_event(&charlie, &space_id, &room_id, vec![tip_id], "hello"),
            &charlie,
        );

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::UnknownSender)
        ));
    }

    #[test]
    fn step11_non_space_member_rejected() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let charlie = keypair::generate(); // registered but not a member
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let (space, mut registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        // Register Charlie but don't add to the Space.
        registry.register(make_identity_record(&charlie, HOME)).unwrap();

        let ev = sign_event(
            build_message_text_event(&charlie, &space_id, &room_id, vec![tip_id], "hello"),
            &charlie,
        );

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::NotASpaceMember)
        ));
    }

    #[test]
    fn step11_non_room_member_rejected() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let charlie = keypair::generate();
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let (mut space, mut registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        // Register Charlie and invite/join the Space but NOT the room.
        registry.register(make_identity_record(&charlie, HOME)).unwrap();
        let charlie_id = format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(charlie.verifying_key().as_bytes())
        );
        let invite_ev = sign_event(
            membership_ev_with_prev(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                vec![tip_id.clone()],
                json!({ "target_identity": charlie_id, "role": "member" }),
            ),
            &alice,
        );
        space.apply_event(&invite_ev).unwrap();
        graph.add_event(&invite_ev, &store).unwrap();
        let charlie_invite_id = invite_ev.event_id.clone().unwrap();
        store.insert(invite_ev).unwrap();

        let join_ev = sign_event(
            membership_ev_with_prev(
                &charlie,
                &space_id,
                "",
                EventType::MembershipJoin,
                vec![charlie_invite_id],
                json!({}),
            ),
            &charlie,
        );
        space.apply_event(&join_ev).unwrap();
        graph.add_event(&join_ev, &store).unwrap();
        let new_tip = join_ev.event_id.clone().unwrap();
        store.insert(join_ev).unwrap();

        // Charlie tries to message in a room they haven't joined.
        let ev = sign_event(
            build_message_text_event(&charlie, &space_id, &room_id, vec![new_tip], "hello"),
            &charlie,
        );

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::NotARoomMember(_))
        ));
    }

    // ── Step 12 tests ─────────────────────────────────────────────────────────

    #[test]
    fn step12_tampered_content_fails_signature() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let mut ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id], "hello"),
            &alice,
        );
        // Tamper with content AFTER signing — event_id stays the same so step 8 passes,
        // but the signature no longer matches.
        ev.content = json!({ "text": "TAMPERED" });
        // Recompute event_id to match the tampered content so step 8 passes.
        let v = serde_json::to_value(&ev).unwrap();
        let bytes = crate::wire::canonical::canonical_event_bytes(&v);
        ev.event_id = Some(hashing::hash_uri(&bytes));

        assert!(matches!(
            validate_steps_8_13(&ev, &space, &registry, &store),
            Err(ExchangeError::SignatureFailure)
        ));
    }

    // ── accept_event tests ────────────────────────────────────────────────────

    #[test]
    fn accept_event_stores_in_dag() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id.clone()], "hello"),
            &alice,
        );
        let ev_id = ev.event_id.clone().unwrap();

        accept_event(ev, &space, &registry, &mut store, &mut graph).unwrap();

        assert!(store.contains(&ev_id));
        assert!(graph.is_tip(&ev_id));
        assert!(!graph.is_tip(&tip_id)); // previous tip replaced
    }

    #[test]
    fn accept_event_duplicate_rejected() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id], "hello"),
            &alice,
        );

        accept_event(ev.clone(), &space, &registry, &mut store, &mut graph).unwrap();
        // Second accept of the same event must fail.
        assert!(accept_event(ev, &space, &registry, &mut store, &mut graph).is_err());
    }

    // ── Integration: two-node propagation ────────────────────────────────────

    /// Alice sends "Hello Bob" on Node A. We simulate propagation by calling
    /// accept_event on Node B with the same event. Verify the event appears in
    /// Node B's store with correct event_id, valid signature, and correct prev_events.
    #[test]
    fn message_propagates_from_node_a_to_node_b() {
        let alice = keypair::generate();
        let bob = keypair::generate();

        // Build the shared setup events once — both nodes replay the identical events
        // so they arrive at the same event_ids and tips.
        let (setup_events, space_id, room_id) = build_setup_events(&alice, &bob);
        let tip_id = setup_events.last().unwrap().event_id.clone().unwrap();

        let mut registry = IdentityRegistry::new();
        registry.register(make_identity_record(&alice, HOME)).unwrap();
        registry.register(make_identity_record(&bob, HOME)).unwrap();

        // Node A
        let mut store_a = EventStore::new();
        let mut graph_a = DagGraph::new();
        let space_a = replay_events(&setup_events, &mut store_a, &mut graph_a);

        // Node B — seeded with the exact same events (deterministic replay).
        let mut store_b = EventStore::new();
        let mut graph_b = DagGraph::new();
        let space_b = replay_events(&setup_events, &mut store_b, &mut graph_b);

        // Alice sends a message on Node A.
        let msg_ev = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip_id.clone()], "Hello Bob"),
            &alice,
        );
        let msg_id = msg_ev.event_id.clone().unwrap();
        let msg_prev = msg_ev.prev_events.clone();

        accept_event(msg_ev.clone(), &space_a, &registry, &mut store_a, &mut graph_a).unwrap();

        // Propagate to Node B (simulate forwarding the event over the wire).
        accept_event(msg_ev, &space_b, &registry, &mut store_b, &mut graph_b).unwrap();

        // Verify event is in Node B's store.
        let stored = store_b.get(&msg_id).expect("event must be in Node B's store");
        assert_eq!(stored.event_id.as_deref(), Some(msg_id.as_str()));
        assert_eq!(stored.event_type, EventType::MessageText);
        assert_eq!(stored.prev_events, msg_prev);
        assert!(stored.signature.is_some());
        // Signature is still valid on the stored copy.
        assert!(verify_event_signature(stored));
        // Event is the current DAG tip on Node B.
        assert!(graph_b.is_tip(&msg_id));
    }

    /// Two concurrent messages from different senders produce two DAG tips.
    #[test]
    fn concurrent_messages_produce_two_tips() {
        let alice = keypair::generate();
        let bob = keypair::generate();
        let mut store = EventStore::new();
        let mut graph = DagGraph::new();
        let (space, registry, space_id, room_id, tip_id) =
            setup_node(&alice, &bob, &mut store, &mut graph);

        let ev_alice = sign_event(
            build_message_text_event(
                &alice,
                &space_id,
                &room_id,
                vec![tip_id.clone()],
                "Hi from Alice",
            ),
            &alice,
        );
        let ev_bob = sign_event(
            build_message_text_event(
                &bob,
                &space_id,
                &room_id,
                vec![tip_id.clone()],
                "Hi from Bob",
            ),
            &bob,
        );

        accept_event(ev_alice, &space, &registry, &mut store, &mut graph).unwrap();
        accept_event(ev_bob, &space, &registry, &mut store, &mut graph).unwrap();

        // Both messages reference the same predecessor → fork → two tips.
        assert_eq!(graph.tip_count(), 2);
    }
}
