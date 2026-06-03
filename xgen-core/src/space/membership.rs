// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Space membership roles and permission enforcement (spec 3.7.8).

use serde::{Deserialize, Serialize};

/// Space member roles in ascending privilege order.
// `Hash` (added at Arc D / PG-12-min) lets `Role` key the per-Room
// `permission_overrides` map `HashMap<(Role, RoomPermission), Effect>`; it is
// consistent with the derived `Eq` and otherwise inert.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Member,
    Moderator,
    Admin,
    Owner,
}

impl Role {
    /// Parse from wire string; returns `None` if unrecognised.
    ///
    /// Same shape as `xgen_common::wire::EventType::from_str` — name shadows
    /// `std::str::FromStr::from_str` but signature differs (`Option<Self>` vs
    /// `Result<Self, Err>`). Implementing `FromStr` would force every call
    /// site to pick an error type; not warranted for a parser that returns
    /// `None` on unknown wire strings.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "member" => Some(Self::Member),
            "moderator" => Some(Self::Moderator),
            "admin" => Some(Self::Admin),
            "owner" => Some(Self::Owner),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Member => "member",
            Self::Moderator => "moderator",
            Self::Admin => "admin",
            Self::Owner => "owner",
        }
    }
}

// ── Per-Room permission overrides (PG-12-min, Arc D) ──────────────────────────

/// A governance axis that a per-Room override may gate (PG-12-min, Arc D PM-D4).
///
/// A small fixed enum on the existing governance axes **plus `send_messages`**.
/// Bounded by the existing `can_X` table — "cannot grant what the Space hasn't
/// defined" holds for free because the override axes are a subset of the
/// Space-defined permission set under the fixed-role model. The first-class Role
/// object (custom roles, `permissions[]`, `position`, `Guest`) is **Arc E**, not
/// here. Serde shape is snake_case so the `state.room_update` content array
/// (`{role, permission, effect}`) round-trips through `from_str`/`as_str`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomPermission {
    SendMessages,
    Invite,
    Kick,
    Ban,
    ChangeInfo,
}

impl RoomPermission {
    /// Parse from wire string; `None` on unrecognised (forward-compat — the
    /// applier skips unknown entries rather than erroring). Same shape as
    /// `Role::from_str`.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "send_messages" => Some(Self::SendMessages),
            "invite" => Some(Self::Invite),
            "kick" => Some(Self::Kick),
            "ban" => Some(Self::Ban),
            "change_info" => Some(Self::ChangeInfo),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SendMessages => "send_messages",
            Self::Invite => "invite",
            Self::Kick => "kick",
            Self::Ban => "ban",
            Self::ChangeInfo => "change_info",
        }
    }
}

/// The effect of a per-Room permission override (PG-12-min, Arc D PM-D4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Allow,
    Deny,
}

impl Effect {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "allow" => Some(Self::Allow),
            "deny" => Some(Self::Deny),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

// ── Permission table (spec 3.7.8) ─────────────────────────────────────────────

pub fn can_invite(role: &Role) -> bool {
    *role >= Role::Moderator
}

pub fn can_kick(role: &Role) -> bool {
    *role >= Role::Moderator
}

/// Whether `role` may issue `membership.mute` events (spec 3.7.8 role table).
pub fn can_mute(role: &Role) -> bool {
    *role >= Role::Moderator
}

pub fn can_ban(role: &Role) -> bool {
    *role >= Role::Admin
}

pub fn can_create_room(role: &Role) -> bool {
    *role >= Role::Admin
}

pub fn can_manage_federation(role: &Role) -> bool {
    *role == Role::Owner
}

pub fn can_change_space_info(role: &Role) -> bool {
    *role >= Role::Admin
}

/// Whether `role` may sign `state.ai_operator_delegate` / `state.ai_operator_revoke`
/// events (spec 3.6.10.6, M3 architecture lock). Owner or admin only — moderator
/// is below the threshold because operator assignment is a Space-wide responsibility
/// decision, not a per-Room moderation action.
pub fn can_delegate_ai_operator(role: &Role) -> bool {
    *role >= Role::Admin
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_ordering() {
        assert!(Role::Owner > Role::Admin);
        assert!(Role::Admin > Role::Moderator);
        assert!(Role::Moderator > Role::Member);
    }

    #[test]
    fn role_from_str() {
        assert_eq!(Role::from_str("member"), Some(Role::Member));
        assert_eq!(Role::from_str("moderator"), Some(Role::Moderator));
        assert_eq!(Role::from_str("admin"), Some(Role::Admin));
        assert_eq!(Role::from_str("owner"), Some(Role::Owner));
        assert_eq!(Role::from_str("superuser"), None);
    }

    #[test]
    fn member_cannot_invite() {
        assert!(!can_invite(&Role::Member));
    }

    #[test]
    fn moderator_can_invite_and_kick_but_not_ban() {
        assert!(can_invite(&Role::Moderator));
        assert!(can_kick(&Role::Moderator));
        assert!(!can_ban(&Role::Moderator));
    }

    #[test]
    fn admin_can_ban_and_create_room() {
        assert!(can_ban(&Role::Admin));
        assert!(can_create_room(&Role::Admin));
        assert!(!can_manage_federation(&Role::Admin));
    }

    #[test]
    fn only_owner_manages_federation() {
        assert!(can_manage_federation(&Role::Owner));
        assert!(!can_manage_federation(&Role::Admin));
        assert!(!can_manage_federation(&Role::Moderator));
        assert!(!can_manage_federation(&Role::Member));
    }

    #[test]
    fn room_permission_str_roundtrip() {
        for p in [
            RoomPermission::SendMessages,
            RoomPermission::Invite,
            RoomPermission::Kick,
            RoomPermission::Ban,
            RoomPermission::ChangeInfo,
        ] {
            assert_eq!(RoomPermission::from_str(p.as_str()), Some(p));
        }
        assert_eq!(RoomPermission::from_str("create_room"), None);
    }

    #[test]
    fn effect_str_roundtrip() {
        assert_eq!(Effect::from_str("allow"), Some(Effect::Allow));
        assert_eq!(Effect::from_str("deny"), Some(Effect::Deny));
        assert_eq!(Effect::from_str("maybe"), None);
    }
}
