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

/// All known event type strings (spec 3.2.2 + Phase 2 sections 3.9–3.16).
///
/// Phase 2 additions use spec-authoritative wire names. Where the
/// implementation guide diverges from the spec, the spec wins (D-045).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    // ── Phase 1 event types ───────────────────────────────────────────────
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

    // ── Phase 2 state events (stored in DAG) ─────────────────────────────
    /// Manual Node ordering declaration by Space owner (3.9.3 Layer 5a).
    #[serde(rename = "state.node_priority")]
    StateNodePriority,
    /// DM Space promotion completion recorded in DAG by Node (3.16.3).
    #[serde(rename = "state.dm_promote")]
    StateDmPromote,
    /// Permanent DAG record of a completed Space migration (3.12.7).
    #[serde(rename = "state.space_migrate")]
    StateSpaceMigrate,

    // ── DM Space promotion control messages (3.16.3) ─────────────────────
    #[serde(rename = "dm.promote_propose")]
    DmPromotePropose,
    #[serde(rename = "dm.promote_confirm")]
    DmPromoteConfirm,
    #[serde(rename = "dm.promote_reject")]
    DmPromoteReject,

    // ── Space migration control messages (3.12.3–3.12.8) ─────────────────
    #[serde(rename = "migration.request")]
    MigrationRequest,
    #[serde(rename = "migration.propose")]
    MigrationPropose,
    #[serde(rename = "migration.accept")]
    MigrationAccept,
    #[serde(rename = "migration.reject")]
    MigrationReject,
    /// Source Node notifies owner that migration failed (rejection or timeout).
    #[serde(rename = "migration.failed")]
    MigrationFailed,
    #[serde(rename = "migration.event_batch")]
    MigrationEventBatch,
    #[serde(rename = "migration.batch_ack")]
    MigrationBatchAck,
    #[serde(rename = "migration.transfer_complete")]
    MigrationTransferComplete,
    #[serde(rename = "migration.verified")]
    MigrationVerified,
    #[serde(rename = "migration.verification_failed")]
    MigrationVerificationFailed,
    /// Courtesy notification to federated peers after migration (3.12.8).
    #[serde(rename = "migration.federation_notify")]
    MigrationFederationNotify,

    // ── Identity replication (3.13.4) ────────────────────────────────────
    #[serde(rename = "identity.replicate")]
    IdentityReplicate,
    #[serde(rename = "identity.replicate_ack")]
    IdentityReplicateAck,

    // ── Bootstrap Node protocol (3.14.3, 3.14.7) ─────────────────────────
    #[serde(rename = "bootstrap.register")]
    BootstrapRegister,
    #[serde(rename = "bootstrap.register_ack")]
    BootstrapRegisterAck,
    #[serde(rename = "bootstrap.keepalive")]
    BootstrapKeepalive,
    #[serde(rename = "bootstrap.keepalive_ack")]
    BootstrapKeepaliveAck,
    #[serde(rename = "bootstrap.deregister")]
    BootstrapDeregister,

    // ── Reputation (3.15.3) ───────────────────────────────────────────────
    #[serde(rename = "reputation.defederation_signal")]
    ReputationDefederationSignal,

    // ── MLS (E2E encryption) protocol messages (3.10.3, 3.10.5) ─────────
    #[serde(rename = "mls.key_package")]
    MlsKeyPackage,
    /// Node acknowledges KeyPackage upload.
    #[serde(rename = "mls.key_package_ack")]
    MlsKeyPackageAck,
    /// Node requests a KeyPackage for a given Identity from a peer Node.
    #[serde(rename = "mls.key_package_request")]
    MlsKeyPackageRequest,
    /// Node responds with a requested KeyPackage.
    #[serde(rename = "mls.key_package_response")]
    MlsKeyPackageResponse,
    #[serde(rename = "mls.commit")]
    MlsCommit,
    #[serde(rename = "mls.welcome")]
    MlsWelcome,
    #[serde(rename = "mls.proposal")]
    MlsProposal,
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
            // Phase 2 state events
            Self::StateNodePriority => "state.node_priority",
            Self::StateDmPromote => "state.dm_promote",
            Self::StateSpaceMigrate => "state.space_migrate",
            // DM promotion
            Self::DmPromotePropose => "dm.promote_propose",
            Self::DmPromoteConfirm => "dm.promote_confirm",
            Self::DmPromoteReject => "dm.promote_reject",
            // Migration
            Self::MigrationRequest => "migration.request",
            Self::MigrationPropose => "migration.propose",
            Self::MigrationAccept => "migration.accept",
            Self::MigrationReject => "migration.reject",
            Self::MigrationFailed => "migration.failed",
            Self::MigrationEventBatch => "migration.event_batch",
            Self::MigrationBatchAck => "migration.batch_ack",
            Self::MigrationTransferComplete => "migration.transfer_complete",
            Self::MigrationVerified => "migration.verified",
            Self::MigrationVerificationFailed => "migration.verification_failed",
            Self::MigrationFederationNotify => "migration.federation_notify",
            // Identity replication
            Self::IdentityReplicate => "identity.replicate",
            Self::IdentityReplicateAck => "identity.replicate_ack",
            // Bootstrap
            Self::BootstrapRegister => "bootstrap.register",
            Self::BootstrapRegisterAck => "bootstrap.register_ack",
            Self::BootstrapKeepalive => "bootstrap.keepalive",
            Self::BootstrapKeepaliveAck => "bootstrap.keepalive_ack",
            Self::BootstrapDeregister => "bootstrap.deregister",
            // Reputation
            Self::ReputationDefederationSignal => "reputation.defederation_signal",
            // MLS
            Self::MlsKeyPackage => "mls.key_package",
            Self::MlsKeyPackageAck => "mls.key_package_ack",
            Self::MlsKeyPackageRequest => "mls.key_package_request",
            Self::MlsKeyPackageResponse => "mls.key_package_response",
            Self::MlsCommit => "mls.commit",
            Self::MlsWelcome => "mls.welcome",
            Self::MlsProposal => "mls.proposal",
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
            // Phase 2 state events
            "state.node_priority" => Some(Self::StateNodePriority),
            "state.dm_promote" => Some(Self::StateDmPromote),
            "state.space_migrate" => Some(Self::StateSpaceMigrate),
            // DM promotion
            "dm.promote_propose" => Some(Self::DmPromotePropose),
            "dm.promote_confirm" => Some(Self::DmPromoteConfirm),
            "dm.promote_reject" => Some(Self::DmPromoteReject),
            // Migration
            "migration.request" => Some(Self::MigrationRequest),
            "migration.propose" => Some(Self::MigrationPropose),
            "migration.accept" => Some(Self::MigrationAccept),
            "migration.reject" => Some(Self::MigrationReject),
            "migration.failed" => Some(Self::MigrationFailed),
            "migration.event_batch" => Some(Self::MigrationEventBatch),
            "migration.batch_ack" => Some(Self::MigrationBatchAck),
            "migration.transfer_complete" => Some(Self::MigrationTransferComplete),
            "migration.verified" => Some(Self::MigrationVerified),
            "migration.verification_failed" => Some(Self::MigrationVerificationFailed),
            "migration.federation_notify" => Some(Self::MigrationFederationNotify),
            // Identity replication
            "identity.replicate" => Some(Self::IdentityReplicate),
            "identity.replicate_ack" => Some(Self::IdentityReplicateAck),
            // Bootstrap
            "bootstrap.register" => Some(Self::BootstrapRegister),
            "bootstrap.register_ack" => Some(Self::BootstrapRegisterAck),
            "bootstrap.keepalive" => Some(Self::BootstrapKeepalive),
            "bootstrap.keepalive_ack" => Some(Self::BootstrapKeepaliveAck),
            "bootstrap.deregister" => Some(Self::BootstrapDeregister),
            // Reputation
            "reputation.defederation_signal" => Some(Self::ReputationDefederationSignal),
            // MLS
            "mls.key_package" => Some(Self::MlsKeyPackage),
            "mls.key_package_ack" => Some(Self::MlsKeyPackageAck),
            "mls.key_package_request" => Some(Self::MlsKeyPackageRequest),
            "mls.key_package_response" => Some(Self::MlsKeyPackageResponse),
            "mls.commit" => Some(Self::MlsCommit),
            "mls.welcome" => Some(Self::MlsWelcome),
            "mls.proposal" => Some(Self::MlsProposal),
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
