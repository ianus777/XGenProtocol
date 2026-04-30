// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Core XGen Event envelope and EventType registry (spec 3.2.1, 3.2.2).
// Lives in xgen-common so both xgen-node and xgen-client can reference the
// concrete types without a circular dependency.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// All known event type strings (spec 3.2.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    #[serde(rename = "message.text")]
    MessageText,
    #[serde(rename = "message.file")]
    MessageFile,
    #[serde(rename = "message.reaction")]
    MessageReaction,
    #[serde(rename = "message.redact")]
    MessageRedact,
    #[serde(rename = "state.space_create")]
    StateSpaceCreate,
    #[serde(rename = "state.dm_space_create")]
    StateDmSpaceCreate,
    #[serde(rename = "state.room_create")]
    StateRoomCreate,
    #[serde(rename = "state.room_update")]
    StateRoomUpdate,
    #[serde(rename = "state.space_update")]
    StateSpaceUpdate,
    #[serde(rename = "state.federation_add")]
    StateFederationAdd,
    #[serde(rename = "membership.invite")]
    MembershipInvite,
    #[serde(rename = "membership.join")]
    MembershipJoin,
    #[serde(rename = "membership.leave")]
    MembershipLeave,
    #[serde(rename = "membership.kick")]
    MembershipKick,
    #[serde(rename = "membership.ban")]
    MembershipBan,
    #[serde(rename = "system.key_rotation")]
    SystemKeyRotation,
}

impl EventType {
    /// Returns the wire string for this event type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MessageText => "message.text",
            Self::MessageFile => "message.file",
            Self::MessageReaction => "message.reaction",
            Self::MessageRedact => "message.redact",
            Self::StateSpaceCreate => "state.space_create",
            Self::StateDmSpaceCreate => "state.dm_space_create",
            Self::StateRoomCreate => "state.room_create",
            Self::StateRoomUpdate => "state.room_update",
            Self::StateSpaceUpdate => "state.space_update",
            Self::StateFederationAdd => "state.federation_add",
            Self::MembershipInvite => "membership.invite",
            Self::MembershipJoin => "membership.join",
            Self::MembershipLeave => "membership.leave",
            Self::MembershipKick => "membership.kick",
            Self::MembershipBan => "membership.ban",
            Self::SystemKeyRotation => "system.key_rotation",
        }
    }

    /// Parse from wire string; returns None if unrecognised.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "message.text" => Some(Self::MessageText),
            "message.file" => Some(Self::MessageFile),
            "message.reaction" => Some(Self::MessageReaction),
            "message.redact" => Some(Self::MessageRedact),
            "state.space_create" => Some(Self::StateSpaceCreate),
            "state.dm_space_create" => Some(Self::StateDmSpaceCreate),
            "state.room_create" => Some(Self::StateRoomCreate),
            "state.room_update" => Some(Self::StateRoomUpdate),
            "state.space_update" => Some(Self::StateSpaceUpdate),
            "state.federation_add" => Some(Self::StateFederationAdd),
            "membership.invite" => Some(Self::MembershipInvite),
            "membership.join" => Some(Self::MembershipJoin),
            "membership.leave" => Some(Self::MembershipLeave),
            "membership.kick" => Some(Self::MembershipKick),
            "membership.ban" => Some(Self::MembershipBan),
            "system.key_rotation" => Some(Self::SystemKeyRotation),
            _ => None,
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// XGen Event envelope (spec 3.2.1).
///
/// `event_id` and `signature` are Option because they are absent while constructing
/// an outgoing event (computed/added during signing). Both are required on received events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub protocol_version: String,

    #[serde(rename = "type")]
    pub event_type: EventType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,

    pub sender: String,
    pub room_id: String,
    pub space_id: String,
    pub prev_events: Vec<String>,
    pub timestamp: String,
    pub content: Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_atts: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Event {
    /// Construct a new unsigned outgoing event. event_id and signature are set later.
    pub fn new(
        event_type: EventType,
        sender: String,
        room_id: String,
        space_id: String,
        prev_events: Vec<String>,
        timestamp: String,
        content: Value,
    ) -> Self {
        Self {
            protocol_version: "0.1".to_string(),
            event_type,
            event_id: None,
            sender,
            room_id,
            space_id,
            prev_events,
            timestamp,
            content,
            meta_atts: None,
            signature: None,
        }
    }
}
