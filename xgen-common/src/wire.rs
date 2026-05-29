// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Core XGen Event envelope and EventType registry (spec 3.2.1, 3.2.2).
// Lives in xgen-common so both xgen-node and xgen-client can reference the
// concrete types without a circular dependency.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};

/// All known event type strings (spec 3.2.2 + Phase 2 sections 3.9–3.16).
///
/// Phase 2 additions use spec-authoritative wire names. Where the
/// implementation guide diverges from the spec, the spec wins (D-045).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// M6 A4-D1: Node-administrator force-eject. Node-signed (sender = home Node
    /// keypair), authority = signature + `sender == space.home_node` (NOT member
    /// role). Removes the target from the Space + bans them. Distinct first-class
    /// Node authority, not a masqueraded member kick (D-070).
    #[serde(rename = "membership.node_eject")]
    MembershipNodeEject,
    /// M6 A4-D1: the reversible counterpart to `membership.node_eject` — Node-
    /// signed, same authority gate; lifts the ban (allows rejoin).
    #[serde(rename = "membership.node_unban")]
    MembershipNodeUnban,
    /// Phase 2 (3.7.8): silence a member for a bounded period without removing them.
    /// Supports the AI-specific `auto_temperature` consequence (3.7.13.6).
    #[serde(rename = "membership.mute")]
    MembershipMute,
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

    // ── AI Identity operator delegation (3.6.10.6) ───────────────────────
    /// Records transfer of operator role for an AI Identity within a Space.
    #[serde(rename = "state.ai_operator_delegate")]
    StateAiOperatorDelegate,
    /// Removes the operator role for an AI Identity within a Space.
    #[serde(rename = "state.ai_operator_revoke")]
    StateAiOperatorRevoke,

    // ── Pacing rules (3.7.12) ────────────────────────────────────────────
    /// Owner-issued update of per-Space pacing rules.
    #[serde(rename = "state.space_pacing")]
    StateSpacePacing,

    // ── Temperature visibility (3.7.13.3) ────────────────────────────────
    /// Owner-issued update of the per-Space `member_temperature_visibility` setting.
    #[serde(rename = "state.space_temperature_visibility")]
    StateSpaceTemperatureVisibility,

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
            Self::MembershipNodeEject => "membership.node_eject",
            Self::MembershipNodeUnban => "membership.node_unban",
            Self::MembershipMute => "membership.mute",
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
            // AI Identity operator delegation
            Self::StateAiOperatorDelegate => "state.ai_operator_delegate",
            Self::StateAiOperatorRevoke => "state.ai_operator_revoke",
            // Pacing rules
            Self::StateSpacePacing => "state.space_pacing",
            // Temperature visibility
            Self::StateSpaceTemperatureVisibility => "state.space_temperature_visibility",
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
    ///
    /// Note: the name shadows `std::str::FromStr::from_str` but the signature
    /// differs (`Option<Self>` vs `Result<Self, Self::Err>`). Implementing
    /// `FromStr` would force every call site to pick an error type or call
    /// `.parse::<EventType>()` instead — broader change than warranted for
    /// a parser that simply returns `None` on unknown wire strings.
    #[allow(clippy::should_implement_trait)]
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
            "membership.node_eject" => Some(Self::MembershipNodeEject),
            "membership.node_unban" => Some(Self::MembershipNodeUnban),
            "membership.kick" => Some(Self::MembershipKick),
            "membership.ban" => Some(Self::MembershipBan),
            "membership.mute" => Some(Self::MembershipMute),
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
            // AI Identity operator delegation
            "state.ai_operator_delegate" => Some(Self::StateAiOperatorDelegate),
            "state.ai_operator_revoke" => Some(Self::StateAiOperatorRevoke),
            // Pacing rules
            "state.space_pacing" => Some(Self::StateSpacePacing),
            // Temperature visibility
            "state.space_temperature_visibility" => Some(Self::StateSpaceTemperatureVisibility),
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
///
/// XGID Retrofit Pass 1 Commit 3 retyped the identifier-bearing fields from
/// `String` / `Option<String>` / `Vec<String>` to flavour-typed XGIDs per D-072
/// and D-073. Wire format is unchanged thanks to each flavour wrapper's
/// `#[serde(transparent)]` impl (Appendix J §J.5 invariance 2). `signature`
/// stays `Option<String>` per D-072 "what XGID is not" — signature strings
/// are not XGIDs.
///
/// `room_id` and `space_id` carry an honest empty-string-wrapped XGID on
/// events where the protocol envelope has no parent of that kind:
///   * `state.space_create` / `state.dm_space_create` — `room_id` and
///     `space_id` are both empty strings (the event IS the Space).
///   * `state.room_create` — `room_id` is empty (the event IS the Room);
///     `space_id` carries the parent SpaceXgid.
///
/// A future Pass may refactor these to `Option<RoomXgid>` / `Option<SpaceXgid>`
/// at the wire layer; Pass 1 deliberately preserves the empty-string-wrapped
/// form so the wire shape stays byte-equal to the pre-XGID layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub protocol_version: String,

    #[serde(rename = "type")]
    pub event_type: EventType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<EventXgid>,

    pub sender: IdentityXgid,
    pub room_id: RoomXgid,
    pub space_id: SpaceXgid,
    pub prev_events: Vec<EventXgid>,
    pub timestamp: String,
    pub content: Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_atts: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Event {
    /// Construct a new unsigned outgoing event. event_id and signature are set later.
    ///
    /// Pass 1 Commit 3 retyped this signature to take flavour-typed XGIDs
    /// directly. Callers that hold raw `String` URIs (Pass 2 / 3 / 4 / 5 work
    /// territory) wrap at the call site via `IdentityXgid::from_xgid(Xgid::new(s))`,
    /// etc. — the explicit wrap surfaces type-boundary crossings per sub-question 3.
    pub fn new(
        event_type: EventType,
        sender: IdentityXgid,
        room_id: RoomXgid,
        space_id: SpaceXgid,
        prev_events: Vec<EventXgid>,
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

/// Convenience: empty-string-wrapped `RoomXgid` for events whose envelope has
/// no parent Room (e.g. `state.space_create`, `state.room_create`). Pass 1
/// wire-shape preservation; a future Pass may refactor the envelope.
pub fn empty_room_xgid() -> RoomXgid {
    RoomXgid::from_xgid(Xgid::new(String::new()))
}

/// Convenience: empty-string-wrapped `SpaceXgid` for events whose envelope has
/// no parent Space (`state.space_create` / `state.dm_space_create` — the event
/// IS the Space). Pass 1 wire-shape preservation.
pub fn empty_space_xgid() -> SpaceXgid {
    SpaceXgid::from_xgid(Xgid::new(String::new()))
}

/// AI capability flag set carried by an AI Identity (spec 3.6.10.3).
///
/// Phase 2 defines two required keys (`dm_initiate`, `spontaneous_post`). The
/// `extra` map carries any additional capability keys for forward compatibility —
/// older Nodes ignore unknown keys and newer Nodes enforce them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiCapabilities {
    pub dm_initiate: bool,
    pub spontaneous_post: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Content for state.ai_operator_delegate events (spec 3.6.10.6).
///
/// Records a transfer of the operator role for an AI Identity within a Space.
/// Signed by the current operator; accountability-only (no privilege grant).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateAiOperatorDelegateContent {
    pub space_id: SpaceXgid,
    pub ai_identity_id: IdentityXgid,
    pub new_operator_identity_id: IdentityXgid,
}

/// Content for state.ai_operator_revoke events (spec 3.6.10.6).
///
/// Removes the operator role for an AI Identity within a Space without naming
/// a replacement; the inviter remains the responsible Identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateAiOperatorRevokeContent {
    pub space_id: SpaceXgid,
    pub ai_identity_id: IdentityXgid,
}

// ── Pacing rules (spec 3.7.12) ───────────────────────────────────────────────

/// Protocol-recommended Phase 2 default for `human_pacing_ms` (spec 3.7.12.2).
pub const DEFAULT_HUMAN_PACING_MS: u64 = 500;

/// Protocol-recommended Phase 2 default for `ai_pacing_ms` (spec 3.7.12.2).
pub const DEFAULT_AI_PACING_MS: u64 = 2000;

/// Content for state.space_pacing events (spec 3.7.12.3).
///
/// Both fields are required — partial updates are not supported (the owner
/// sets both values explicitly on each update).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateSpacePacingContent {
    pub human_pacing_ms: u64,
    pub ai_pacing_ms: u64,
}

// ── Temperature property (spec 3.7.13) ───────────────────────────────────────

/// Reserved `meta_atts` key for the Room-level temperature signal.
/// Float in `[0.0, 1.0]`; always visible to every Room member (spec 3.7.13.3).
pub const META_ATT_ROOM_TEMPERATURE: &str = "xgen.room_temperature";

/// Reserved `meta_atts` key for the per-member temperature signal.
/// Float in `[0.0, 1.0]`; subject to `member_temperature_visibility` filtering
/// (spec 3.7.13.4).
pub const META_ATT_MEMBER_TEMPERATURE: &str = "xgen.member_temperature";

/// Reserved `reason` value used on `membership.kick` / `membership.mute` Events
/// when the action was issued automatically by a temperature plugin
/// (spec 3.7.8 "Standard reason values", 3.7.13.6).
pub const REASON_AUTO_TEMPERATURE: &str = "auto_temperature";

/// Permitted values for `SpaceState.member_temperature_visibility` (spec 3.7.13.3).
/// Default. Visibility limited to moderators-or-higher plus the subject themselves.
pub const VISIBILITY_MODERATOR: &str = "moderator";
/// Every member sees every other member's temperature.
pub const VISIBILITY_EVERYONE: &str = "everyone";
/// Only the subject sees their own temperature.
pub const VISIBILITY_SELF_ONLY: &str = "self_only";

/// Default value at Space creation when `member_temperature_visibility` is absent
/// (spec 3.7.13.3).
pub const DEFAULT_MEMBER_TEMPERATURE_VISIBILITY: &str = VISIBILITY_MODERATOR;

/// Clamp a temperature value to the protocol's permitted range `[0.0, 1.0]`
/// (spec 3.7.13.1). Out-of-range values are clamped silently before transmission.
///
/// The explicit `is_nan()` branch is load-bearing: `f64::clamp` propagates
/// NaN through its input (and panics if its bounds contain NaN), so a
/// straight `v.clamp(0.0, 1.0)` would let a NaN temperature leak past this
/// function. The protocol spec requires that NaN be normalised to `0.0`
/// before transmission, so the manual form is the correct one.
///
/// `clippy::if_same_then_else` flags the NaN-branch and `v < 0.0` branch as
/// duplicates because both return `0.0`; that is correct by construction (a
/// NaN is "out of range below" semantically) but the explicit branch must
/// fire BEFORE the `<` comparison since `NaN < 0.0` is `false`. Removing the
/// branch would silently change observable behaviour for NaN input.
#[allow(clippy::manual_clamp, clippy::if_same_then_else)]
pub fn clamp_temperature(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}

/// Room metadata response: threshold table (spec 3.7.13.2).
///
/// Published by the home Node at session open as part of the Room metadata
/// response (3.7.7). All three fields are required when the table is present;
/// values MUST satisfy `0.0 < warm < hot < fiery <= 1.0`. A malformed table is
/// omitted by the Node — clients fall back to Ch6 default thresholds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemperatureThresholds {
    pub warm: f64,
    pub hot: f64,
    pub fiery: f64,
}

impl TemperatureThresholds {
    /// Validate the threshold table per spec 3.7.13.2.
    /// Returns `true` iff `0.0 < warm < hot < fiery <= 1.0`. NaNs are rejected.
    pub fn is_valid(&self) -> bool {
        let nan_free = !self.warm.is_nan() && !self.hot.is_nan() && !self.fiery.is_nan();
        nan_free
            && self.warm > 0.0
            && self.warm < self.hot
            && self.hot < self.fiery
            && self.fiery <= 1.0
    }
}

/// Content for `state.space_temperature_visibility` events (spec 3.7.13.3).
///
/// The `member_temperature_visibility` field is an open-enum string. Permitted
/// values are `moderator`, `everyone`, `self_only`; unknown values are accepted
/// by the Node but treated as `moderator` at enforcement time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateSpaceTemperatureVisibilityContent {
    pub member_temperature_visibility: String,
}

/// Content for `membership.mute` events (spec 3.7.8).
///
/// `cooldown_until` is an RFC 3339 timestamp; the mute auto-lifts at that time.
/// `reason` is free-text; the reserved value `auto_temperature` marks the mute
/// as issued by a temperature plugin (3.7.13.6).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MembershipMuteContent {
    pub target_identity: IdentityXgid,
    pub reason: String,
    pub cooldown_until: String,
}
