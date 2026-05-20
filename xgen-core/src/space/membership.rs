// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Space membership roles and permission enforcement (spec 3.7.8).

use serde::{Deserialize, Serialize};

/// Space member roles in ascending privilege order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
}
