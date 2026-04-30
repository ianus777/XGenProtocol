// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Wire types for Phase 1: Event envelope, EventType registry, transport messages.
// spec 3.2.1, 3.2.2, 3.3.4

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

/// Transport-layer control messages (spec 3.3.4).
/// These are NOT Events — they carry no event_id, sender, room_id, etc.
/// All include protocol_version. Type values use the "transport." prefix.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransportMessage {
    /// Sent by Node immediately after WebSocket connection is established.
    #[serde(rename = "transport.challenge")]
    Challenge {
        protocol_version: String,
        nonce: String,
        timestamp: String,
    },
    /// Sent by client in response to Challenge. Signature covers nonce bytes only.
    #[serde(rename = "transport.auth")]
    Auth {
        protocol_version: String,
        /// xgen://pubkey/ed25519:<base64url-pubkey>
        identity_id: String,
        nonce: String,
        signature: String,
    },
    /// Sent by Node on successful authentication.
    #[serde(rename = "transport.auth_ok")]
    AuthOk {
        protocol_version: String,
        identity_id: String,
        timestamp: String,
    },
    /// Sent by Node on failed authentication, followed immediately by connection close.
    #[serde(rename = "transport.auth_fail")]
    AuthFail {
        protocol_version: String,
        error_code: u32,
        error_string: String,
        timestamp: String,
    },
    /// General transport error.
    #[serde(rename = "transport.error")]
    Error {
        protocol_version: String,
        error_code: u32,
        error_string: String,
        timestamp: String,
    },
    /// Graceful connection close (spec 3.3.9).
    #[serde(rename = "transport.goodbye")]
    Goodbye {
        protocol_version: String,
        reason: String,
        timestamp: String,
    },
    /// Request missed Events since a given event_id.
    #[serde(rename = "transport.sync_request")]
    SyncRequest {
        protocol_version: String,
        since: String,
    },
    /// Node signalling the client to back off.
    #[serde(rename = "transport.rate_limit")]
    RateLimit {
        protocol_version: String,
        retry_after_ms: u64,
    },
}

/// Content for message.text events (spec 3.2.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTextContent {
    pub text: String,
}

// ── Federation message types ────────────────────────────────────────────────────

/// Capabilities declared during federation handshake (spec 3.4.2, 3.4.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationCapabilities {
    pub serialisation: Vec<String>,
    #[serde(default)]
    pub compression: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
}

impl Default for FederationCapabilities {
    fn default() -> Self {
        Self {
            serialisation: vec!["json".to_string()],
            compression: vec![],
            extensions: vec![],
        }
    }
}

/// Resolved capabilities selected by the receiving Node (spec 3.4.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiatedCapabilities {
    pub serialisation: String,
    pub protocol_version: String,
}

/// Federation handshake messages (spec 3.4.2).
/// Each is signed by the sender's node keypair. `signature` is None only while
/// constructing an outgoing message — it is always present on received messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FederationMessage {
    /// Initiating Node opens the handshake.
    #[serde(rename = "federation.hello")]
    Hello {
        protocol_version: String,
        node_id: String,
        capabilities: FederationCapabilities,
        shared_spaces: Vec<String>,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Receiving Node replies with its own capabilities and the negotiated values.
    #[serde(rename = "federation.capabilities")]
    Capabilities {
        protocol_version: String,
        node_id: String,
        capabilities: FederationCapabilities,
        negotiated: NegotiatedCapabilities,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Initiating Node confirms negotiated capabilities and opens the active session.
    #[serde(rename = "federation.accept")]
    Accept {
        protocol_version: String,
        node_id: String,
        session_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Either Node refuses the handshake with a 2xxx error code.
    #[serde(rename = "federation.reject")]
    Reject {
        protocol_version: String,
        node_id: String,
        error_code: u32,
        error_string: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Either Node ends an active federation session.
    #[serde(rename = "federation.goodbye")]
    Goodbye {
        protocol_version: String,
        node_id: String,
        reason: String,
        session_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

// ── Identity registration message types ────────────────────────────────────────

/// A device entry embedded in `identity.record` (spec 3.6.6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityDeviceEntry {
    pub device_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    pub authorised_at: String,
}

/// Identity protocol messages (spec 3.6.3–3.6.8).
/// `identity.register` and `identity.update` are signed by the Identity keypair.
/// Response messages (register_ok, register_fail, record, not_found) are unsigned.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IdentityMessage {
    /// Client requests Identity registration (spec 3.6.3).
    #[serde(rename = "identity.register")]
    Register {
        protocol_version: String,
        identity_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        display_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        trust_assertion: Option<Value>,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Node confirms successful registration (spec 3.6.4).
    #[serde(rename = "identity.register_ok")]
    RegisterOk {
        protocol_version: String,
        identity_id: String,
        registered_at: String,
    },
    /// Node rejects registration with a 3xxx error code (spec 3.6.4–3.6.5).
    #[serde(rename = "identity.register_fail")]
    RegisterFail {
        protocol_version: String,
        error_code: u32,
        error_string: String,
        timestamp: String,
    },
    /// Client or Node requests an Identity record (spec 3.6.7).
    #[serde(rename = "identity.get")]
    Get {
        protocol_version: String,
        identity_id: String,
    },
    /// Node responds with the full Identity record (spec 3.6.7).
    #[serde(rename = "identity.record")]
    Record {
        protocol_version: String,
        identity_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        display_name: Option<String>,
        registered_at: String,
        devices: Vec<IdentityDeviceEntry>,
        home_node: String,
    },
    /// Node responds when the requested Identity is not found (spec 3.6.7).
    #[serde(rename = "identity.not_found")]
    NotFound {
        protocol_version: String,
        identity_id: String,
    },
    /// Client updates its Identity record; signed (spec 3.6.8).
    #[serde(rename = "identity.update")]
    Update {
        protocol_version: String,
        identity_id: String,
        update_version: u64,
        changes: Value,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

// ── Space control messages ──────────────────────────────────────────────────────

/// Space-level control messages sent over an active federation connection (spec 3.7.10).
/// These are NOT Events — they carry no event_id, sender, or prev_events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SpaceControlMessage {
    /// Sent by a new Node to request participation in a Space (spec 3.7.10 step 5).
    #[serde(rename = "space.join_request")]
    JoinRequest {
        space_id: String,
        node_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn event_type_round_trip() {
        let et = EventType::MessageText;
        let serialised = serde_json::to_string(&et).unwrap();
        assert_eq!(serialised, "\"message.text\"");
        let parsed: EventType = serde_json::from_str(&serialised).unwrap();
        assert_eq!(parsed, EventType::MessageText);
    }

    #[test]
    fn event_type_from_str_all_variants() {
        let cases = [
            "message.text", "message.file", "message.reaction", "message.redact",
            "state.space_create", "state.dm_space_create", "state.room_create",
            "state.room_update", "state.space_update", "state.federation_add",
            "membership.invite", "membership.join", "membership.leave",
            "membership.kick", "membership.ban", "system.key_rotation",
        ];
        for s in cases {
            assert!(EventType::from_str(s).is_some(), "failed for {s}");
        }
    }

    #[test]
    fn event_type_unknown_returns_none() {
        assert!(EventType::from_str("bogus.type").is_none());
    }

    #[test]
    fn event_serialises_with_type_field() {
        let ev = Event::new(
            EventType::MessageText,
            "xgen://pubkey/ed25519:abc".to_string(),
            "xgen://hash/sha256:room".to_string(),
            "xgen://hash/sha256:space".to_string(),
            vec![],
            "2026-04-27T12:00:00Z".to_string(),
            json!({"text": "hello"}),
        );
        let v: serde_json::Value = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["type"], "message.text");
        assert_eq!(v["protocol_version"], "0.1");
        // event_id and signature omitted when None
        assert!(v.get("event_id").is_none());
        assert!(v.get("signature").is_none());
    }

    #[test]
    fn event_deserialises_full_envelope() {
        let json = json!({
            "protocol_version": "0.1",
            "type": "message.text",
            "event_id": "xgen://hash/sha256:abc",
            "sender": "xgen://pubkey/ed25519:xyz",
            "room_id": "xgen://hash/sha256:room",
            "space_id": "xgen://hash/sha256:space",
            "prev_events": [],
            "timestamp": "2026-04-27T12:00:00Z",
            "content": {"text": "hello"},
            "signature": "ed25519:key:sig",
        });
        let ev: Event = serde_json::from_value(json).unwrap();
        assert_eq!(ev.event_type, EventType::MessageText);
        assert_eq!(ev.event_id.as_deref(), Some("xgen://hash/sha256:abc"));
    }

    #[test]
    fn transport_challenge_round_trip() {
        let msg = TransportMessage::Challenge {
            protocol_version: "0.1".to_string(),
            nonce: "abc123".to_string(),
            timestamp: "2026-04-27T12:00:00.000Z".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"transport.challenge\""));
        let parsed: TransportMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            TransportMessage::Challenge { nonce, .. } => assert_eq!(nonce, "abc123"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn transport_auth_ok_round_trip() {
        let msg = TransportMessage::AuthOk {
            protocol_version: "0.1".to_string(),
            identity_id: "xgen://pubkey/ed25519:abc".to_string(),
            timestamp: "2026-04-27T12:00:00.000Z".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"transport.auth_ok\""));
        let parsed: TransportMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TransportMessage::AuthOk { .. }));
    }

    #[test]
    fn message_text_content_round_trip() {
        let c = MessageTextContent { text: "hello world".to_string() };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: MessageTextContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.text, "hello world");
    }

    #[test]
    fn space_join_request_round_trip() {
        let msg = SpaceControlMessage::JoinRequest {
            space_id: "xgen://hash/sha256:space".to_string(),
            node_id: "xgen://pubkey/ed25519:node".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"space.join_request\""));
        let parsed: SpaceControlMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            SpaceControlMessage::JoinRequest { space_id, node_id } => {
                assert_eq!(space_id, "xgen://hash/sha256:space");
                assert_eq!(node_id, "xgen://pubkey/ed25519:node");
            }
        }
    }
}
