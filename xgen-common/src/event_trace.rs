// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Global Event tracing interface — D-033, D-038.
//
// Every inbound and outbound Event passes through trace_event(). LOCAL actions
// (create, store, apply, reject) pass through trace_local(). No individual handler
// adds its own Event field log calls. The role gate on trace_event() ensures debug
// output is restricted to owner/admin sessions; trace_local() has no role gate
// (LOCAL actions contain no sensitive content).

use std::fmt;

use crate::wire::Event;

// ── Direction ──────────────────────────────────────────────────────────────────

/// Flow direction relative to this binary. Produces Appendix G direction values.
pub enum EventDirection {
    In,    // Event arriving at this binary from the network
    Out,   // Event leaving this binary to the network
    Local, // Action occurring entirely within this binary — no network crossing
}

impl fmt::Display for EventDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::In    => write!(f, "IN"),
            Self::Out   => write!(f, "OUT"),
            Self::Local => write!(f, "LOCAL"),
        }
    }
}

// ── Session context ────────────────────────────────────────────────────────────

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

// ── Network Event tracing ──────────────────────────────────────────────────────

/// Log a single Event at the transport boundary.
///
/// Called at exactly two points per binary: the inbound deserialization boundary
/// and the outbound serialization boundary. Never called from individual handlers.
/// The Local direction variant is not valid here — use trace_local() instead.
/// The content field is never logged. Output is suppressed unless the session
/// holds Owner or Admin role.
pub fn trace_event(event: &Event, direction: EventDirection, session: &SessionContext) {
    let role_permits = matches!(
        session.role,
        Some(SpaceRole::Owner) | Some(SpaceRole::Admin)
    );
    if !role_permits {
        return;
    }

    let action = match direction {
        EventDirection::In  => "receive_event",
        EventDirection::Out => "send_event",
        EventDirection::Local => {
            tracing::warn!("trace_event called with Local direction — use trace_local() instead");
            return;
        }
    };

    let event_id = event.event_id.as_deref().unwrap_or("(none)");
    tracing::debug!(
        direction  = %direction,
        action     = %action,
        event_id   = %event_id,
        event_type = %event.event_type,
        sender     = %event.sender,
        space_id   = %event.space_id,
        room_id    = %event.room_id,
        timestamp  = %event.timestamp,
        "Event"
    );
}

// ── LOCAL action tracing ───────────────────────────────────────────────────────

/// Valid LOCAL action values per Appendix G action registry.
pub enum LocalAction {
    CreateEvent,
    StoreEvent,
    ApplyEvent,
    RejectEvent,
}

impl fmt::Display for LocalAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateEvent => write!(f, "create_event"),
            Self::StoreEvent  => write!(f, "store_event"),
            Self::ApplyEvent  => write!(f, "apply_event"),
            Self::RejectEvent => write!(f, "reject_event"),
        }
    }
}

/// Log a LOCAL action at the Event boundary.
///
/// Called for create_event, store_event, apply_event, reject_event.
/// These actions never cross the network — direction is always LOCAL.
/// The content field is never logged. No role gate — LOCAL actions are always
/// logged when the subscriber level permits.
pub fn trace_local(
    action: LocalAction,
    event_id: &str,
    event_type: Option<&str>,
    space_id: Option<&str>,
    error_code: Option<u32>,
) {
    tracing::debug!(
        direction  = "LOCAL",
        action     = %action,
        event_id   = %event_id,
        event_type = event_type.unwrap_or(""),
        space_id   = space_id.unwrap_or(""),
        error_code = error_code.map(|c| c.to_string()).unwrap_or_default(),
        "Event"
    );
}

// ── Session header ─────────────────────────────────────────────────────────────

/// Write the session header block to the log.
///
/// Must be called once, immediately after subscriber init, before any other logging.
/// Fields that are not yet known at subscriber init time (e.g. identity_id and
/// connected_node for the CLI client — D-038) must be passed as None and logged
/// as body lines when they become available.
pub fn write_session_header(
    app_type: &str,
    self_id: Option<&str>,         // node_id (node) or identity_id (client); None = omit (D-038)
    endpoint: Option<&str>,        // node listen address — None for client
    connected_node: Option<&str>,  // node URL client connected to — None for node and CLI client (D-038)
    protocol_version: &str,
    build: &str,
    session_id: &str,
    started_at: &str,
) {
    tracing::info!("=== XGEN SESSION START ===");
    tracing::info!("app_type={}", app_type);

    if let Some(id) = self_id {
        match app_type {
            "node"   => tracing::info!("node_id={}", id),
            "client" => tracing::info!("identity_id={}", id),
            _        => tracing::info!("id={}", id),
        }
    }

    if let Some(ep) = endpoint {
        tracing::info!("endpoint={}", ep);
    }
    if let Some(cn) = connected_node {
        tracing::info!("connected_node={}", cn);
    }

    tracing::info!("protocol_version={}", protocol_version);
    tracing::info!("build={}", build);
    tracing::info!("session_id={}", session_id);
    tracing::info!("started_at={}", started_at);

    // Mandatory blank line — body start delimiter per Appendix G
    tracing::info!("");
}

// ── Session footer ─────────────────────────────────────────────────────────────

/// Valid exit reason values per Appendix G.
pub enum ExitReason {
    Shutdown,
    Restart,
    Error,
}

impl fmt::Display for ExitReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shutdown => write!(f, "shutdown"),
            Self::Restart  => write!(f, "restart"),
            Self::Error    => write!(f, "error"),
        }
    }
}

/// Write the session footer block to the log.
///
/// Must be called on every clean exit path. Never called on crash or kill —
/// absence of footer is itself the signal of abnormal termination.
pub fn write_session_footer(reason: ExitReason) {
    let ended_at = chrono::Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    // Mandatory blank line — body end delimiter per Appendix G
    tracing::info!("");
    tracing::info!("=== XGEN SESSION END ===");
    tracing::info!("ended_at={}", ended_at);
    tracing::info!("reason={}", reason);
}
