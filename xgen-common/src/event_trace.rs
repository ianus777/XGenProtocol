// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Global Event tracing interface — D-033.
//
// Every inbound and outbound Event passes through trace_event(). No individual
// handler adds its own Event field log calls. The role gate ensures that message
// content is never logged and that debug output is restricted to owner/admin sessions.

use std::fmt;

use crate::wire::Event;

/// Flow direction relative to this binary.
pub enum EventDirection {
    Inbound,
    Outbound,
}

impl fmt::Display for EventDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inbound => write!(f, "Inbound"),
            Self::Outbound => write!(f, "Outbound"),
        }
    }
}

/// Role of the authenticated identity in the relevant Space.
pub enum SpaceRole {
    Owner,
    Admin,
    Moderator,
    Member,
}

/// Session state available at the Event boundary.
pub struct SessionContext {
    /// Authenticated Identity URI, if any.
    pub identity_id: Option<String>,
    /// Role of that Identity in the relevant Space. None = unauthenticated or unknown.
    /// Phase 1: set to Some(Owner) for all local-mode authenticated connections.
    pub role: Option<SpaceRole>,
    /// Space context if known at session level.
    pub space_id: Option<String>,
}

/// Log a single Event at the transport boundary.
///
/// Called at exactly two points per binary: the inbound deserialization boundary
/// and the outbound serialization boundary. Never called from individual handlers.
///
/// The `content` field is never logged. Output is suppressed unless the session
/// holds Owner or Admin role.
pub fn trace_event(event: &Event, direction: EventDirection, session: &SessionContext) {
    let role_permits = matches!(
        session.role,
        Some(SpaceRole::Owner) | Some(SpaceRole::Admin)
    );
    if !role_permits {
        return;
    }

    let event_id = event.event_id.as_deref().unwrap_or("(none)");
    tracing::debug!(
        direction = %direction,
        event_id = %event_id,
        event_type = %event.event_type,
        sender = %event.sender,
        space_id = %event.space_id,
        room_id = %event.room_id,
        timestamp = %event.timestamp,
        "Event"
    );
}
