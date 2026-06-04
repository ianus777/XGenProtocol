// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// State key concept (spec 3.9.1).
//
// A state key is a logical tuple (category, key_field) that uniquely identifies
// a piece of mutable state in a Space. Two Events conflict when they share the
// same state key and have no causal ordering in the DAG.

use crate::wire::types::{Event, EventType};

/// Logical identifier for a single piece of mutable Space state.
///
/// Two Events with the same StateKey that have no causal ordering are in conflict.
/// Message Events (`message.text`, etc.) never produce a StateKey — concurrent
/// messages are both displayed, there is no conflict.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateKey {
    /// Logical category: "membership", "state.room_update", etc.
    pub category: String,
    /// The specific entity within that category, formatted as "scope:id".
    pub key_field: String,
}

impl StateKey {
    pub fn new(category: impl Into<String>, key_field: impl Into<String>) -> Self {
        Self { category: category.into(), key_field: key_field.into() }
    }
}

impl std::fmt::Display for StateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.category, self.key_field)
    }
}

/// Derive the StateKey for an Event. Returns None for Events that do not define
/// mutable state (message events, transport events, etc.).
///
/// Pass 1 Commit 4: Event field types are now typed XGIDs; project to &str for
/// the String-typed StateKey fields. Pass 2 widens StateKey to carry typed XGIDs.
pub fn state_key_for_event(event: &Event) -> Option<StateKey> {
    match &event.event_type {
        // Membership events targeting an identity's own membership.
        // Sender IS the affected identity for join/leave.
        EventType::MembershipJoin | EventType::MembershipLeave => Some(StateKey::new(
            "membership",
            format!("{}:{}", event.space_id.as_str(), event.sender.as_str()),
        )),

        // Membership events where an actor targets another identity.
        // M6 A4-D1: node_eject / node_unban also target another identity and
        // compete for the same membership state key (so an eject and a member
        // kick/ban/join on the same target resolve against each other).
        EventType::MembershipInvite
        | EventType::MembershipKick
        | EventType::MembershipBan
        | EventType::MembershipNodeEject
        | EventType::MembershipNodeUnban => {
            let target = event.content["target_identity"].as_str()?;
            Some(StateKey::new(
                "membership",
                format!("{}:{}", event.space_id.as_str(), target),
            ))
        }

        // Room state — keyed by the room being updated.
        EventType::StateRoomUpdate => Some(StateKey::new(
            "state.room_update",
            event.room_id.as_str().to_string(),
        )),

        // Space state — keyed by the space being updated.
        EventType::StateSpaceUpdate => Some(StateKey::new(
            "state.space_update",
            event.space_id.as_str().to_string(),
        )),

        // Thread status (Arc E PG-08) — keyed by the thread being transitioned.
        // `thread.resolved` and `thread.archived` share the **same** category so
        // a concurrent resolved-vs-archived pair on one thread is a genuine
        // state-key conflict that `resolve()` settles (no Layer-1 pair for these;
        // falls to Layer-5c lexicographic — both are terminal read-only states,
        // the distinction is advisory; M9-flag only). `thread.create` is a
        // creation event (root-ish, like `state.room_create`) — not a conflict
        // key, so it falls through to `None`.
        EventType::ThreadResolved | EventType::ThreadArchived => {
            let thread = event.content["thread"].as_str()?;
            Some(StateKey::new("thread.status", thread))
        }

        // Node priority declaration — only one active ordering per Space.
        EventType::StateNodePriority => Some(StateKey::new(
            "state.node_priority",
            event.space_id.as_str().to_string(),
        )),

        // Key rotation — keyed by the identity rotating their key.
        EventType::SystemKeyRotation => Some(StateKey::new(
            "system.key_rotation",
            event.sender.as_str().to_string(),
        )),

        // All other EventTypes (message.*, federation.*, migration.*, mls.*, etc.)
        // do not define mutable state that requires conflict resolution.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::types::EventType;
    use serde_json::json;

    use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};

    fn make_event(event_type: EventType, sender: &str, space_id: &str, room_id: &str, content: serde_json::Value) -> Event {
        Event::new(
            event_type,
            IdentityXgid::from_xgid(Xgid::new(sender.to_string())),
            RoomXgid::from_xgid(Xgid::new(room_id.to_string())),
            SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
            vec![EventXgid::from_xgid(Xgid::new("prev".to_string()))],
            "2026-01-01T00:00:00.000Z".to_string(),
            content,
        )
    }

    #[test]
    fn membership_join_state_key_uses_sender() {
        let ev = make_event(EventType::MembershipJoin, "xgen://pubkey/ed25519:ALICE", "space1", "", json!({}));
        let key = state_key_for_event(&ev).unwrap();
        assert_eq!(key.category, "membership");
        assert!(key.key_field.contains("ALICE"));
    }

    #[test]
    fn membership_ban_state_key_uses_target() {
        let ev = make_event(
            EventType::MembershipBan,
            "xgen://pubkey/ed25519:ADMIN",
            "space1",
            "",
            json!({"target_identity": "xgen://pubkey/ed25519:BOB"}),
        );
        let key = state_key_for_event(&ev).unwrap();
        assert_eq!(key.category, "membership");
        assert!(key.key_field.contains("BOB"));
        assert!(!key.key_field.contains("ADMIN"));
    }

    #[test]
    fn join_and_ban_on_same_target_share_state_key() {
        let identity = "xgen://pubkey/ed25519:TARGET";
        let join_ev = make_event(EventType::MembershipJoin, identity, "space1", "", json!({}));
        let ban_ev = make_event(
            EventType::MembershipBan,
            "xgen://pubkey/ed25519:ADMIN",
            "space1",
            "",
            json!({"target_identity": identity}),
        );
        let join_key = state_key_for_event(&join_ev).unwrap();
        let ban_key = state_key_for_event(&ban_ev).unwrap();
        assert_eq!(join_key, ban_key);
    }

    #[test]
    fn message_text_has_no_state_key() {
        let ev = make_event(EventType::MessageText, "id", "space", "room", json!({"text":"hi"}));
        assert!(state_key_for_event(&ev).is_none());
    }

    #[test]
    fn state_room_update_keyed_by_room() {
        let ev = make_event(EventType::StateRoomUpdate, "id", "space", "room1", json!({}));
        let key = state_key_for_event(&ev).unwrap();
        assert_eq!(key.category, "state.room_update");
        assert_eq!(key.key_field, "room1");
    }

    #[test]
    fn state_node_priority_keyed_by_space() {
        let ev = make_event(EventType::StateNodePriority, "id", "space1", "", json!({}));
        let key = state_key_for_event(&ev).unwrap();
        assert_eq!(key.category, "state.node_priority");
        assert_eq!(key.key_field, "space1");
    }

    // ── Thread model (Arc E PG-08) ────────────────────────────────────────────

    #[test]
    fn thread_resolved_and_archived_on_same_thread_share_state_key() {
        // The load-bearing convergence property: resolved + archived on one
        // thread must produce the SAME state key (shared "thread.status"
        // category) so a concurrent pair is a genuine conflict resolve() settles.
        let thread = "xgen://thread/sha256:T";
        let resolved = make_event(EventType::ThreadResolved, "id", "space1", "room1", json!({"thread": thread}));
        let archived = make_event(EventType::ThreadArchived, "id2", "space1", "room1", json!({"thread": thread}));
        let rk = state_key_for_event(&resolved).unwrap();
        let ak = state_key_for_event(&archived).unwrap();
        assert_eq!(rk, ak);
        assert_eq!(rk.category, "thread.status");
        assert_eq!(rk.key_field, thread);
    }

    #[test]
    fn thread_status_on_different_threads_do_not_share_key() {
        let a = make_event(EventType::ThreadResolved, "id", "space1", "room1", json!({"thread": "xgen://thread/sha256:A"}));
        let b = make_event(EventType::ThreadResolved, "id", "space1", "room1", json!({"thread": "xgen://thread/sha256:B"}));
        assert_ne!(state_key_for_event(&a).unwrap(), state_key_for_event(&b).unwrap());
    }

    #[test]
    fn thread_create_has_no_state_key() {
        // thread.create is a creation event (root-ish like room.create) — not a
        // conflict key.
        let ev = make_event(EventType::ThreadCreate, "id", "space1", "room1", json!({"auth_tier_min": 1}));
        assert!(state_key_for_event(&ev).is_none());
    }
}
