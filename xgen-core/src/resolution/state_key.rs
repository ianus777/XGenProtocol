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
        // Membership events targeting an identity's own membership. Sender IS the
        // affected identity for join/leave. Scope-aware (MP-F4 / A1): a space-level
        // join/leave (`room_id` empty) keys room-agnostically (unchanged from
        // pre-F4); a room-level join/leave (`room_id` populated) keys with a room
        // dimension, so space-membership and room-membership of one identity occupy
        // different key-groups and never collide in resolution (the DM invitee's
        // space-join + room-join were dropped onto one key before this).
        EventType::MembershipJoin | EventType::MembershipLeave => Some(membership_scope_key(
            event.space_id.as_str(),
            event.room_id.as_str(),
            event.sender.as_str(),
        )),

        // Kick targets another identity AND has a room-level applier branch
        // (`apply_kick`, `room_id` populated → room-level removal), so it is
        // scope-aware on the SAME basis as join/leave (MP-F4 / A1): a room-kick and
        // a room-join of one identity in one room must share a key (conflict
        // preserved), while a room-kick and a space-join must not (different facts).
        // Keyed on the target, not the sender.
        EventType::MembershipKick => {
            let target = event.content["target_identity"].as_str()?;
            Some(membership_scope_key(event.space_id.as_str(), event.room_id.as_str(), target))
        }

        // Always-space-level membership events: invite / ban / node_eject /
        // node_unban have NO room-level applier branch — they change space
        // membership (members / pending_invites / banned) and cascade to all rooms —
        // so they stay room-agnostic, keyed on the target identity.
        // M6 A4-D1: node_eject / node_unban also target another identity and compete
        // for the same membership state key (so an eject and a member kick/ban/join
        // on the same target resolve against each other).
        EventType::MembershipInvite
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

        // Space admission (spec 3.7.14.5) — keyed by the Space, so two concurrent
        // admission changes are a genuine conflict that resolution settles to a
        // single value on every Node. Required rather than optional: admission
        // decides an irreversible act, and a value that differed between Nodes
        // would mean the same join was admitted on one and refused on another,
        // with no later correction possible.
        //
        // NOTE the asymmetry, because it is deliberate and looks like an
        // oversight: the nearest sibling `state.space_temperature_visibility` has
        // NO arm here at all. This arm is written from `StateSpaceUpdate`'s shape,
        // not copied from that twin.
        EventType::StateSpaceAdmission => Some(StateKey::new(
            "state.space_admission",
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

        // MLS group genesis anchor (Arc H PG-05, AH-D3) — keyed per-Room so a
        // concurrent re-init of the same Room's group is a genuine state-key
        // conflict that resolve() settles to one (a Room has exactly one MLS
        // group genesis). Epoch *advances* are `mls.commit` events, keyed by the
        // `MlsCommit` arm below (M8.7 CC-D2) — they no longer fall through to
        // `None`.
        EventType::MlsGroupInit => Some(StateKey::new(
            "state.mls_group_init",
            event.room_id.as_str().to_string(),
        )),

        // MLS epoch advance (Arc H PG-05 / M8.7 CC-D2) — keyed by
        // `(room, target_epoch)`. Two members committing the same `N → N+1`
        // advance at one frontier are a genuine state-key conflict that
        // `resolve()` settles to one (Layer-5c lexicographic, CC-D3 — concurrent
        // advances carry no semantic priority). **Keyed by target epoch, not
        // per-Room:** per-Room would group *all* of a Room's commits, so a later
        // `2 → 3` and a losing `1 → 2` (mutually frontier-concurrent) would
        // compete and the tiebreak could pick the epoch-2 loser — an epoch
        // regression. Keying by target epoch means only same-transition commits
        // group; sequential advances are different keys. Absent/malformed `epoch`
        // ⇒ `None` (matches `apply_mls_commit`'s silent no-op).
        EventType::MlsCommit => {
            let target_epoch = event.content["epoch"].as_u64()?;
            Some(StateKey::new(
                "state.mls_commit",
                format!("{}:{}", event.room_id.as_str(), target_epoch),
            ))
        }

        // Key rotation — keyed by the identity rotating their key.
        EventType::SystemKeyRotation => Some(StateKey::new(
            "system.key_rotation",
            event.sender.as_str().to_string(),
        )),

        // All other EventTypes (message.*, federation.*, migration.*, etc.) do
        // not define mutable state that requires conflict resolution. Among
        // `mls.*`, `mls.commit` now keys (the `MlsCommit` arm above, M8.7 CC-D2);
        // `mls.welcome` / `mls.proposal` / `mls.key_package` remain keyless.
        _ => None,
    }
}

/// Scope-aware membership key (MP-F4 / A1). A membership event whose applier has a
/// room-level branch (join / leave / kick) keys by the **scope** of the fact it
/// asserts: an empty `room_id` is a space-level fact (room-agnostic key, unchanged
/// from pre-F4); a populated `room_id` is a room-level fact (a room dimension in the
/// key). The `room:` infix guarantees a room-scoped key can never alias a
/// space-level key whose identity literal happens to match a room literal.
/// `affected` is the identity whose membership the event changes (sender for
/// join/leave, target for kick).
fn membership_scope_key(space: &str, room: &str, affected: &str) -> StateKey {
    if room.is_empty() {
        StateKey::new("membership", format!("{space}:{affected}"))
    } else {
        StateKey::new("membership", format!("{space}:room:{room}:{affected}"))
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

    /// Leg C test 5 — `state.space_admission` is keyed by the SPACE, and two
    /// changes to one Space therefore share one key (spec 3.7.14.5).
    #[test]
    fn space_admission_state_key_is_space_scoped_and_shared() {
        let a = make_event(
            EventType::StateSpaceAdmission,
            "xgen://pubkey/ed25519:OWNER",
            "space1",
            "",
            json!({"admission": "invite"}),
        );
        let key_a = state_key_for_event(&a).expect("admission events define mutable state");
        assert_eq!(key_a.category, "state.space_admission");
        assert_eq!(key_a.key_field, "space1");
        assert_eq!(key_a.to_string(), "state.space_admission:space1");

        // A SECOND change to the same Space — different sender, different value —
        // must produce the SAME key, or two concurrent changes would not be seen as
        // a conflict and could resolve differently per Node.
        let b = make_event(
            EventType::StateSpaceAdmission,
            "xgen://pubkey/ed25519:SOMEONE_ELSE",
            "space1",
            "",
            json!({"admission": "open"}),
        );
        let key_b = state_key_for_event(&b).expect("admission events define mutable state");
        assert_eq!(key_a, key_b, "two admission changes to one Space share one state key");

        // And a change to a DIFFERENT Space must NOT collide with it.
        let c = make_event(
            EventType::StateSpaceAdmission,
            "xgen://pubkey/ed25519:OWNER",
            "space2",
            "",
            json!({"admission": "invite"}),
        );
        let key_c = state_key_for_event(&c).expect("admission events define mutable state");
        assert_ne!(key_a, key_c, "admission is scoped per Space, not global");
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

    // ── MP-F4 (A1) — scope-aware membership keys ──────────────────────────────

    #[test]
    fn mp_f4_space_join_and_room_join_by_one_id_have_distinct_keys() {
        // The finding: a space-join (room_id empty) and a room-join (room_id
        // populated) by the SAME sender in the SAME space must NOT share a key, so
        // resolution never drops one as a concurrent sibling of the other.
        let bob = "xgen://pubkey/ed25519:BOB";
        let space_join = make_event(EventType::MembershipJoin, bob, "space1", "", json!({}));
        let room_join = make_event(EventType::MembershipJoin, bob, "space1", "room1", json!({}));
        assert_ne!(
            state_key_for_event(&space_join).unwrap(),
            state_key_for_event(&room_join).unwrap(),
        );
        // Space-level key is unchanged (room-agnostic); room-level carries the room.
        assert_eq!(
            state_key_for_event(&space_join).unwrap().key_field,
            "space1:xgen://pubkey/ed25519:BOB"
        );
        assert_eq!(
            state_key_for_event(&room_join).unwrap().key_field,
            "space1:room:room1:xgen://pubkey/ed25519:BOB"
        );
    }

    #[test]
    fn mp_f4_room_joins_differ_by_room_match_within_room() {
        let bob = "xgen://pubkey/ed25519:BOB";
        let r1 = make_event(EventType::MembershipJoin, bob, "space1", "room1", json!({}));
        let r1b = make_event(EventType::MembershipJoin, bob, "space1", "room1", json!({}));
        let r2 = make_event(EventType::MembershipJoin, bob, "space1", "room2", json!({}));
        assert_eq!(state_key_for_event(&r1).unwrap(), state_key_for_event(&r1b).unwrap());
        assert_ne!(state_key_for_event(&r1).unwrap(), state_key_for_event(&r2).unwrap());
    }

    #[test]
    fn mp_f4_room_kick_shares_key_with_room_join_not_space_join() {
        // room-kick(bob in R) and room-join(bob in R) MUST share a key (conflict
        // preserved — the kick-widening's reason); room-kick(bob in R) and
        // space-join(bob) MUST NOT (different facts).
        let bob = "xgen://pubkey/ed25519:BOB";
        let room_kick = make_event(
            EventType::MembershipKick,
            "xgen://pubkey/ed25519:ADMIN",
            "space1",
            "room1",
            json!({ "target_identity": bob }),
        );
        let room_join = make_event(EventType::MembershipJoin, bob, "space1", "room1", json!({}));
        let space_join = make_event(EventType::MembershipJoin, bob, "space1", "", json!({}));
        assert_eq!(
            state_key_for_event(&room_kick).unwrap(),
            state_key_for_event(&room_join).unwrap(),
            "room-kick and room-join of one id in one room must share a key"
        );
        assert_ne!(
            state_key_for_event(&room_kick).unwrap(),
            state_key_for_event(&space_join).unwrap(),
            "room-kick must not collide with a space-join (different facts)"
        );
    }

    #[test]
    fn mp_f4_room_leave_shares_key_with_room_join_same_room() {
        // room-leave(bob, R) and room-join(bob, R) must share the room-scoped key
        // (conflict preserved — the leave-widening's reason).
        let bob = "xgen://pubkey/ed25519:BOB";
        let room_leave = make_event(EventType::MembershipLeave, bob, "space1", "room1", json!({}));
        let room_join = make_event(EventType::MembershipJoin, bob, "space1", "room1", json!({}));
        assert_eq!(
            state_key_for_event(&room_leave).unwrap(),
            state_key_for_event(&room_join).unwrap(),
        );
    }

    #[test]
    fn mp_f4_invite_ban_stay_room_agnostic_even_with_room_id() {
        // invite/ban have no room-level applier branch → always space-level, keyed
        // on target, regardless of any room_id the event carries (a DM invite
        // carries the DM room_id).
        let bob = "xgen://pubkey/ed25519:BOB";
        let owner = "xgen://pubkey/ed25519:OWNER";
        let invite_no_room =
            make_event(EventType::MembershipInvite, owner, "space1", "", json!({ "target_identity": bob }));
        let invite_with_room = make_event(
            EventType::MembershipInvite,
            owner,
            "space1",
            "room1",
            json!({ "target_identity": bob }),
        );
        assert_eq!(
            state_key_for_event(&invite_no_room).unwrap(),
            state_key_for_event(&invite_with_room).unwrap(),
            "invite is always space-level — room_id does not change its key"
        );
        let ban_with_room = make_event(
            EventType::MembershipBan,
            owner,
            "space1",
            "room1",
            json!({ "target_identity": bob }),
        );
        assert_eq!(state_key_for_event(&ban_with_room).unwrap().key_field, "space1:xgen://pubkey/ed25519:BOB");
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

    // ── Arc H (PG-05) — MLS group init ────────────────────────────────────────

    #[test]
    fn mls_group_init_keyed_per_room() {
        // Two group-inits for the SAME Room share a state key (a Room has exactly
        // one MLS genesis — a concurrent re-init resolves to one). Different Rooms
        // do not collide.
        let a = make_event(EventType::MlsGroupInit, "id", "space1", "room1", json!({"epoch": 0}));
        let b = make_event(EventType::MlsGroupInit, "id2", "space1", "room1", json!({"epoch": 0}));
        let c = make_event(EventType::MlsGroupInit, "id", "space1", "room2", json!({"epoch": 0}));
        let ka = state_key_for_event(&a).unwrap();
        let kb = state_key_for_event(&b).unwrap();
        assert_eq!(ka, kb);
        assert_eq!(ka.category, "state.mls_group_init");
        assert_eq!(ka.key_field, "room1");
        assert_ne!(ka, state_key_for_event(&c).unwrap());
    }

    // ── M8.7 (CC-D2) — MLS commit (epoch advance) conflict domain ─────────────

    #[test]
    fn mls_commit_keyed_by_room_and_target_epoch() {
        // Two commits advancing the SAME target epoch in one Room share a key
        // (genuine conflict resolve() settles to one). Different target epoch, or
        // different Room, do not collide.
        let a = make_event(EventType::MlsCommit, "id", "space1", "room1", json!({"epoch": 1}));
        let b = make_event(EventType::MlsCommit, "id2", "space1", "room1", json!({"epoch": 1}));
        let c = make_event(EventType::MlsCommit, "id", "space1", "room1", json!({"epoch": 2}));
        let d = make_event(EventType::MlsCommit, "id", "space1", "room2", json!({"epoch": 1}));
        let ka = state_key_for_event(&a).unwrap();
        let kb = state_key_for_event(&b).unwrap();
        assert_eq!(ka, kb);
        assert_eq!(ka.category, "state.mls_commit");
        assert_eq!(ka.key_field, "room1:1");
        assert_ne!(ka, state_key_for_event(&c).unwrap(), "different target epoch ⇒ different key");
        assert_ne!(ka, state_key_for_event(&d).unwrap(), "different room ⇒ different key");
    }

    #[test]
    fn mls_commit_sequential_advances_do_not_share_key_epoch_regression_guard() {
        // CC-D2 regression guard: a losing `1 → 2` commit (target epoch 2) and a
        // later `2 → 3` commit (target epoch 3) on the SAME Room must NOT share a
        // key — per-Room keying would let them compete and the tiebreak could pick
        // the epoch-2 loser (an epoch regression). Per-target-epoch keeps them
        // separate: sequential advances are different keys.
        let c_1to2 = make_event(EventType::MlsCommit, "id", "space1", "room1", json!({"epoch": 2}));
        let c_2to3 = make_event(EventType::MlsCommit, "id", "space1", "room1", json!({"epoch": 3}));
        assert_ne!(
            state_key_for_event(&c_1to2).unwrap(),
            state_key_for_event(&c_2to3).unwrap()
        );
    }

    #[test]
    fn mls_commit_absent_epoch_has_no_state_key() {
        // Absent/malformed epoch ⇒ None (matches apply_mls_commit's silent no-op).
        let ev = make_event(EventType::MlsCommit, "id", "space1", "room1", json!({}));
        assert!(state_key_for_event(&ev).is_none());
    }
}
