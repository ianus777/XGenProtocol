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
// Serialize/Deserialize are hand-written below (NOT derived) so that an Event
// whose wire `type` is unrecognised deserializes into `EventType::Unknown(s)`
// instead of failing outright — the forward-compatibility guarantee (PG-09 /
// FC-D1, FC-D3). Because the derive is removed, the per-variant
// `#[serde(rename = "...")]` helper attributes are removed too; the wire strings
// now live solely in `as_str`/`from_str`, which the custom impls delegate to.
// Do NOT re-add `#[derive(Serialize, Deserialize)]` or the rename attributes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    // ── Phase 1 event types ───────────────────────────────────────────────
    MessageText,
    MessageFile,
    MessageReaction,
    MessageRedact,
    StateSpaceCreate,
    StateDmSpaceCreate,
    StateRoomCreate,
    StateRoomUpdate,
    StateSpaceUpdate,
    StateFederationAdd,
    MembershipInvite,
    MembershipJoin,
    MembershipLeave,
    MembershipKick,
    MembershipBan,
    /// M6 A4-D1: Node-administrator force-eject. Node-signed (sender = home Node
    /// keypair), authority = signature + `sender == space.home_node` (NOT member
    /// role). Removes the target from the Space + bans them. Distinct first-class
    /// Node authority, not a masqueraded member kick (D-070).
    MembershipNodeEject,
    /// M6 A4-D1: the reversible counterpart to `membership.node_eject` — Node-
    /// signed, same authority gate; lifts the ban (allows rejoin).
    MembershipNodeUnban,
    /// Phase 2 (3.7.8): silence a member for a bounded period without removing them.
    /// Supports the AI-specific `auto_temperature` consequence (3.7.13.6).
    MembershipMute,
    SystemKeyRotation,

    // ── Phase 2 state events (stored in DAG) ─────────────────────────────
    /// Manual Node ordering declaration by Space owner (3.9.3 Layer 5a).
    StateNodePriority,
    /// DM Space promotion completion recorded in DAG by Node (3.16.3).
    StateDmPromote,
    /// Permanent DAG record of a completed Space migration (3.12.7).
    StateSpaceMigrate,

    // ── DM Space promotion control messages (3.16.3) ─────────────────────
    DmPromotePropose,
    DmPromoteConfirm,
    DmPromoteReject,

    // ── Space migration control messages (3.12.3–3.12.8) ─────────────────
    MigrationRequest,
    MigrationPropose,
    MigrationAccept,
    MigrationReject,
    /// Source Node notifies owner that migration failed (rejection or timeout).
    MigrationFailed,
    MigrationEventBatch,
    MigrationBatchAck,
    MigrationTransferComplete,
    MigrationVerified,
    MigrationVerificationFailed,
    /// Courtesy notification to federated peers after migration (3.12.8).
    MigrationFederationNotify,

    // ── Identity replication (3.13.4) ────────────────────────────────────
    IdentityReplicate,
    IdentityReplicateAck,

    // ── Bootstrap Node protocol (3.14.3, 3.14.7) ─────────────────────────
    BootstrapRegister,
    BootstrapRegisterAck,
    BootstrapKeepalive,
    BootstrapKeepaliveAck,
    BootstrapDeregister,

    // ── Reputation (3.15.3) ───────────────────────────────────────────────
    ReputationDefederationSignal,

    // ── AI Identity operator delegation (3.6.10.6) ───────────────────────
    /// Records transfer of operator role for an AI Identity within a Space.
    StateAiOperatorDelegate,
    /// Removes the operator role for an AI Identity within a Space.
    StateAiOperatorRevoke,

    // ── Pacing rules (3.7.12) ────────────────────────────────────────────
    /// Owner-issued update of per-Space pacing rules.
    StateSpacePacing,

    // ── Temperature visibility (3.7.13.3) ────────────────────────────────
    /// Owner-issued update of the per-Space `member_temperature_visibility` setting.
    StateSpaceTemperatureVisibility,

    // ── MLS (E2E encryption) protocol messages (3.10.3, 3.10.5) ─────────
    MlsKeyPackage,
    /// Node acknowledges KeyPackage upload.
    MlsKeyPackageAck,
    /// Node requests a KeyPackage for a given Identity from a peer Node.
    MlsKeyPackageRequest,
    /// Node responds with a requested KeyPackage.
    MlsKeyPackageResponse,
    MlsCommit,
    MlsWelcome,
    MlsProposal,

    // ── Forward-compatibility catch-all (PG-09 / FC-D1) ──────────────────
    /// An event type this build does not recognise. Holds the exact wire `type`
    /// string verbatim. Produced only by the custom `Deserialize` (tolerant:
    /// unknown wire type → `Unknown(s)`); `from_str` stays strict and never
    /// yields this variant. Such events are stored + relayed but never applied
    /// (FC-D6) — see the apply-dispatch no-op arm in `exchange.rs`.
    Unknown(String),
}

impl EventType {
    /// Returns the wire string for this event type.
    ///
    /// Returns `&str` (not `&'static str`): known variants return their
    /// compiled-in literal, but `Unknown(s)` borrows the stored wire string,
    /// which is not `'static` (FC-D1). The raw string is the canonical-bytes
    /// truth for unknown events (FC-D3), so it round-trips byte-identically.
    pub fn as_str(&self) -> &str {
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
            // FC-D1: emit the stored raw wire string verbatim → byte-identical
            // round-trip for relayed unknown events.
            Self::Unknown(s) => s,
        }
    }

    /// Parse from wire string; returns `None` if unrecognised.
    ///
    /// **Stays strict (FC-D5): unknown → `None`, never `Unknown(s)`.** This is
    /// the load-bearing half of Shape A's two entry points — `Deserialize` is
    /// *tolerant* (unknown → `Unknown(s)`), `from_str` is *strict*. They do
    /// different jobs and must not be merged: the M7-events subscription filter
    /// resolves a named-type filter via `from_str`, so keeping it strict means a
    /// filter can only name *known* types (an unknown exact name → `BAD_ARGUMENT`,
    /// EV-D4) — the fail-closed property. Do NOT add an `Unknown` arm here.
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

// Custom serde impls (replace the derive) — Shape A, FC-D1/FC-D3.
//
// Serialize emits the wire string via `as_str()` (a plain JSON string), exactly
// as the derived unit-variant impl did, so known-type wire bytes — and therefore
// `event_id` canonical bytes — are byte-identical (FC-D2/§4 #4). Unknown emits
// its stored string verbatim.
impl Serialize for EventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

// Deserialize is *tolerant* (the whole point of Shape A): read the field as a
// plain string, resolve known types via the strict `from_str`, and fall back to
// `Unknown(s)` for anything unrecognised — never errors on an unknown `type`.
// `from_str` itself stays strict; the tolerance lives here, not there.
impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match EventType::from_str(&s) {
            Some(known) => known,
            None => EventType::Unknown(s),
        })
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

    /// The Space this event addresses. `state.space_create` and
    /// `state.dm_space_create` carry an empty `space_id` and use their own
    /// `event_id` as the Space id; every other event carries `space_id`
    /// explicitly. Returns `None` only for an unsigned outbound create event
    /// (empty `space_id` and no `event_id` yet) — such events never reach a
    /// filter predicate, which only ever sees accepted, signed events.
    ///
    /// Canonical home for this resolution (M7-events EV-D4 v1.1) — the
    /// subscription-filter `spaces` arm uses it so a `spaces:[S]` filter also
    /// sees S's own `state.space_create`. `xgen-node::fanout::event_space_id`
    /// converges onto this helper in C3 (no lasting two-copy drift).
    pub fn effective_space_id(&self) -> Option<SpaceXgid> {
        if self.space_id.as_str().is_empty() {
            self.event_id
                .as_ref()
                .map(|e| SpaceXgid::from_xgid(Xgid::new(e.as_str().to_string())))
        } else {
            Some(self.space_id.clone())
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

/// Unknown-event forward-compat (PG-09 / FC-D1) — C1 type-layer tests.
///
/// Guards the load-bearing invariant: `Deserialize` is tolerant (unknown wire
/// type → `Unknown(s)`), `from_str` stays strict (unknown → `None`); and known
/// types serialize byte-identically to their prior `#[serde(rename)]` wire form.
#[cfg(test)]
mod event_type_forward_compat_tests {
    use super::EventType;

    fn to_json(t: &EventType) -> String {
        serde_json::to_string(t).unwrap()
    }
    fn from_json(s: &str) -> EventType {
        serde_json::from_str(s).unwrap()
    }

    /// The known variants. Used for the round-trip + inverse-drift sweeps;
    /// `Unknown` is deliberately excluded (covered by the dedicated tests).
    fn known_variants() -> Vec<EventType> {
        use EventType::*;
        vec![
            MessageText, MessageFile, MessageReaction, MessageRedact,
            StateSpaceCreate, StateDmSpaceCreate, StateRoomCreate, StateRoomUpdate,
            StateSpaceUpdate, StateFederationAdd, MembershipInvite, MembershipJoin,
            MembershipLeave, MembershipKick, MembershipBan, MembershipNodeEject,
            MembershipNodeUnban, MembershipMute, SystemKeyRotation, StateNodePriority,
            StateDmPromote, StateSpaceMigrate, DmPromotePropose, DmPromoteConfirm,
            DmPromoteReject, MigrationRequest, MigrationPropose, MigrationAccept,
            MigrationReject, MigrationFailed, MigrationEventBatch, MigrationBatchAck,
            MigrationTransferComplete, MigrationVerified, MigrationVerificationFailed,
            MigrationFederationNotify, IdentityReplicate, IdentityReplicateAck,
            BootstrapRegister, BootstrapRegisterAck, BootstrapKeepalive,
            BootstrapKeepaliveAck, BootstrapDeregister, ReputationDefederationSignal,
            StateAiOperatorDelegate, StateAiOperatorRevoke, StateSpacePacing,
            StateSpaceTemperatureVisibility, MlsKeyPackage, MlsKeyPackageAck,
            MlsKeyPackageRequest, MlsKeyPackageResponse, MlsCommit, MlsWelcome,
            MlsProposal,
        ]
    }

    #[test]
    fn known_type_serializes_to_exact_wire_string() {
        // FC-D2 / §4 #4: byte-identity for known types must not regress vs the
        // prior `#[serde(rename = "...")]` derive.
        assert_eq!(to_json(&EventType::MessageText), "\"message.text\"");
        assert_eq!(to_json(&EventType::StateSpaceCreate), "\"state.space_create\"");
        assert_eq!(to_json(&EventType::MembershipNodeEject), "\"membership.node_eject\"");
        assert_eq!(to_json(&EventType::MlsProposal), "\"mls.proposal\"");
    }

    #[test]
    fn known_type_round_trips_byte_identically() {
        // serialize → deserialize → serialize, for every known variant.
        for t in known_variants() {
            let json = to_json(&t);
            let back = from_json(&json);
            assert_eq!(back, t, "deserialize mismatch for {json}");
            assert_eq!(to_json(&back), json, "re-serialize drift for {json}");
        }
    }

    #[test]
    fn as_str_and_from_str_are_inverse_for_known() {
        // Now that the rename attrs are gone, as_str (Serialize) and from_str
        // are independent tables — this guards them against drift (FC-D2 / §4 #4).
        for t in known_variants() {
            assert_eq!(
                EventType::from_str(t.as_str()),
                Some(t.clone()),
                "as_str/from_str drift for {t:?}"
            );
        }
    }

    #[test]
    fn unknown_type_deserializes_to_unknown_variant() {
        // Tolerant Deserialize: an unrecognised `type` becomes Unknown(s).
        assert_eq!(
            from_json("\"bogus.type\""),
            EventType::Unknown("bogus.type".to_string())
        );
    }

    #[test]
    fn unknown_type_re_serializes_byte_identically() {
        // FC-D3: the raw type string round-trips verbatim.
        let t = from_json("\"bogus.type\"");
        assert_eq!(to_json(&t), "\"bogus.type\"");
        assert_eq!(from_json(&to_json(&t)), t, "unknown full round-trip drift");
    }

    #[test]
    fn from_str_stays_strict_on_unknown() {
        // FC-D5: from_str is strict — unknown → None, never Unknown.
        assert_eq!(EventType::from_str("bogus.type"), None);
        assert_eq!(
            EventType::from_str("message.text"),
            Some(EventType::MessageText)
        );
    }

    #[test]
    fn as_str_and_display_expose_raw_string_for_unknown() {
        let t = EventType::Unknown("x.y.z".to_string());
        assert_eq!(t.as_str(), "x.y.z");
        assert_eq!(t.to_string(), "x.y.z"); // Display delegates to as_str
    }
}
