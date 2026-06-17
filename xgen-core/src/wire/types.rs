// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Wire types for Phase 1: Event envelope, EventType registry, transport messages.
// spec 3.2.1, 3.2.2, 3.3.4
//
// Event and EventType are canonical protocol types — they live in xgen-common so
// both xgen-node and xgen-client can reference them without a circular dependency.
// Re-exported here so all internal code using `crate::wire::types::{Event, EventType}`
// continues to compile without change (Fix 17).

pub use xgen_common::wire::{
    clamp_temperature, AiCapabilities, Event, EventType, MembershipMuteContent,
    StateAiOperatorDelegateContent, StateAiOperatorRevokeContent,
    StateSpacePacingContent, StateSpaceTemperatureVisibilityContent, ThreadStatus,
    TemperatureThresholds, DEFAULT_AI_PACING_MS, DEFAULT_HUMAN_PACING_MS,
    DEFAULT_MEMBER_TEMPERATURE_VISIBILITY, INFRA_EVENT_KINDS, META_ATT_MEMBER_TEMPERATURE,
    META_ATT_ROOM_TEMPERATURE, REASON_AUTO_TEMPERATURE, VISIBILITY_EVERYONE,
    VISIBILITY_MODERATOR, VISIBILITY_SELF_ONLY,
};

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[inline]
fn is_false(b: &bool) -> bool {
    !*b
}

/// Content for message.text events (spec 3.2.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTextContent {
    pub text: String,
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
        /// The Node's own pubkey `node_id` (`xgen://pubkey/ed25519:…`), echoed so
        /// the client can write it as a Space's `home_node` (M10.4 / MP-F13,
        /// Shape B). Additive optional (Ch3 §3.0.3): old nodes omit it, old
        /// clients ignore it — no `protocol_version` bump.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        node_id: Option<String>,
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
        /// M6 §3.3 — the hash URI of the protocol event this error pertains to,
        /// when the rejection is about a specific event submission (so the
        /// originator can correlate it to an in-flight `Event`). `None` for
        /// transport-level errors not tied to an event (malformed framing, etc.).
        /// Read via `TransportMessage::event_id()`. Per-variant realisation of the
        /// envelope-level correlation field (D-070); byte-identical on the wire to
        /// an envelope wrapper because the enum is internally tagged.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },
    /// Graceful connection close (spec 3.3.9).
    #[serde(rename = "transport.goodbye")]
    Goodbye {
        protocol_version: String,
        reason: String,
        timestamp: String,
    },
    /// Request missed Events since a given event_id.
    ///
    /// `limit` (F-7) is the maximum number of events the responder should send
    /// in a single response page. Absent means "responder's implementation
    /// default" (`[sync].batch_size`, default 1000). Backwards-compatible:
    /// pre-F-7 senders omit the field; pre-F-7 receivers ignore it.
    #[serde(rename = "transport.sync_request")]
    SyncRequest {
        protocol_version: String,
        since: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
    },
    /// Authoritative end-of-batch signal for a `transport.sync_request` response
    /// (F-6 / spec 3.3.6). Sent by the responder after the last Event of a batch.
    ///
    /// Replaces the 500ms quiet-time heuristic — see DECISIONS.md D-065 ("honest
    /// behaviour over polite behaviour"). The requester waits for this message
    /// instead of guessing via inter-event silence.
    ///
    /// Fields:
    /// - `since` — echo of the request's `since` cursor, for correlation when
    ///   multiple sync_requests are in flight.
    /// - `new_tip` — the requester's position after the current batch. Whole-
    ///   batch model (Clair-locked at Phase 1, see `collect_sync_history`
    ///   emission site): `new_tip` is the event_id of the last event delivered
    ///   in this batch, or echoes `since` when the batch is empty.
    /// - `continue_from` — non-null when more events are available; the value
    ///   is the cursor to pass as `since` on the follow-up `SyncRequest`.
    ///   Null when catch-up is complete.
    #[serde(rename = "transport.sync_complete")]
    SyncComplete {
        protocol_version: String,
        since: String,
        new_tip: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        continue_from: Option<String>,
    },
    /// Node signalling the client to back off.
    #[serde(rename = "transport.rate_limit")]
    RateLimit {
        protocol_version: String,
        retry_after_ms: u64,
    },
    /// M8.5-B (INV-D1/INV-D2, CP-2) — a pending invitee's request to bootstrap
    /// its membership. The Node serves the *structural* events of `space_id`
    /// (Space/Room creates + the membership chain incl. the invite naming the
    /// requester) — **no message content** — so the invitee can read the
    /// invite's `event_id` and chain its `membership.join` causally after it
    /// (dissolving the M85-A3 concurrency). Authorization is the requester
    /// holding an **unexpired** `pending_invite` in `space_id`; an absent or
    /// expired invite is refused with transport code `1011`
    /// (`invite_bootstrap_refused`). On success the Node replies with the same
    /// `HistoryBatch` + `SyncComplete` shape as a `sync_request`. The requester
    /// is the authenticated connection Identity (no separate principal field);
    /// `space_id` + the home-node endpoint are the inherent out-of-band data of
    /// any invitation. `collect_sync_history` (member-only) is untouched.
    #[serde(rename = "transport.invite_bootstrap_request")]
    InviteBootstrapRequest {
        protocol_version: String,
        space_id: String,
    },
    /// Positive event-acceptance signal (M6 §3.1/§3.2, D-070). Sent by the Node
    /// to the originator after the submitted event has been validated AND durably
    /// persisted to the event store, and BEFORE local fan-out begins (the G2
    /// boundary). Sibling to `Error`: acceptance and rejection are two signals of
    /// equal importance, opposite direction. Originator-only; does not propagate
    /// (the accepted event itself propagates via the normal fan-out + federation
    /// machinery). `event_id` always pertains to an event, hence required (not
    /// `Option`); read uniformly with `Error` via `TransportMessage::event_id()`.
    #[serde(rename = "transport.event_accepted")]
    EventAccepted {
        protocol_version: String,
        /// Hash URI of the accepted event — the same `event_id` the originator
        /// sees on the `Event` they submitted.
        event_id: String,
        /// RFC 3339 UTC timestamp of acceptance. Trace/audit only; not used for
        /// any timing-sensitive logic.
        accepted_at: String,
    },

    // ── M12.1 blob transfer (chunked base64 over WS; M12-D1, R-2) ──────────────
    // The byte transfer rides WS (the client↔node channel, R-2), not the named
    // pipe. Encoding is chunked base64 (M12-D1, unchanged) — WS frames are JSON
    // too, so raw bytes can't ride a field without base64; the framing is these
    // begin/chunk/end + fetch variants instead of pipe lines. The bytes are
    // already ciphertext (M12-D5).
    /// Begin a chunked-base64 blob upload (client→node). `blob_ref` =
    /// `hash_uri(ciphertext)` the client claims; `size` = ciphertext bytes.
    /// Followed by `BlobChunk`* then `BlobUploadEnd`.
    #[serde(rename = "transport.blob_upload_begin")]
    BlobUploadBegin {
        protocol_version: String,
        blob_ref: String,
        size: u64,
    },
    /// One base64 chunk of a blob's ciphertext (≤ `BLOB_CHUNK_BYTES` raw bytes).
    /// Used both ways: upload (client→node) and fetch (node→client). `seq` is the
    /// 0-based order index (informational — a single WS stream preserves order).
    #[serde(rename = "transport.blob_chunk")]
    BlobChunk {
        protocol_version: String,
        seq: u32,
        data: String,
    },
    /// End of a chunked blob upload; the node reassembles, verifies
    /// `hash_uri(bytes) == blob_ref`, stores content-blind, and replies
    /// `BlobUploadOk` (or `Error` 10001 on hash mismatch).
    #[serde(rename = "transport.blob_upload_end")]
    BlobUploadEnd {
        protocol_version: String,
        blob_ref: String,
    },
    /// Node→client acknowledgement that a blob upload was stored.
    #[serde(rename = "transport.blob_upload_ok")]
    BlobUploadOk {
        protocol_version: String,
        blob_ref: String,
    },
    /// Client→node request to fetch a blob by content-address. The node replies
    /// `BlobChunk`* then `BlobFetchEnd` (or `Error` on miss / mismatch).
    ///
    /// `space_id` (M12.3-D1/P1, additive) scopes a federated lazy fetch: on a
    /// local store miss the node resolves the Space's federated holders
    /// (`home_node` ∪ `federation_nodes`) and fetches across homes (M12.3-D4).
    /// Absent (M12.1 self-thread / any non-Space-scoped caller) → local-only, no
    /// federation fetch (→ W5). Old peers omit / ignore it (Ch3 §3.0.3).
    #[serde(rename = "transport.blob_fetch_request")]
    BlobFetchRequest {
        protocol_version: String,
        blob_ref: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        space_id: Option<String>,
    },
    /// Node→client end-of-fetch marker (after the last `BlobChunk`).
    #[serde(rename = "transport.blob_fetch_end")]
    BlobFetchEnd {
        protocol_version: String,
        blob_ref: String,
    },
}

/// M12.1 (M12-D1, V1) — raw ciphertext bytes per `BlobChunk` before base64.
/// 128 KiB → base64 ≈ 174 KB, comfortably under the `MAX_PAYLOAD_BYTES` (256 KB)
/// frame ceiling (`wire::framing`) even with the JSON envelope.
pub const BLOB_CHUNK_BYTES: usize = 128 * 1024;

/// M12.2a (D4/VB) — default F6 ceiling on a single blob's **ciphertext** bytes.
/// 16 MiB: clearly multi-MB, but modest — the node reassembles the whole
/// ciphertext into one in-memory buffer, so this bounds the per-upload memory a
/// single client can force (the DoS surface F6 exists to close). Operator-
/// overridable via `[node].max_blob_bytes`; the client pre-check uses this
/// default (it cannot know the operator's live ceiling — the node-at-Begin gate
/// is authoritative).
pub const DEFAULT_MAX_BLOB_BYTES: u64 = 16 * 1024 * 1024;

impl TransportMessage {
    /// The hash URI of the protocol event this transport message pertains to, if
    /// any. The correlation primitive (M6 §3.1 / D-070): an originator with
    /// multiple in-flight event submissions matches each incoming `EventAccepted`
    /// or `Error` to its originating event by this value. Centralising the read in
    /// one accessor keeps the per-variant `event_id` fields drift-free — the
    /// correlation consumer calls this; the match lives in exactly one place.
    /// Returns `None` for messages that do not pertain to a specific event
    /// (`Goodbye`, transport-level `Error`s, the handshake messages, etc.).
    pub fn event_id(&self) -> Option<&str> {
        match self {
            TransportMessage::EventAccepted { event_id, .. } => Some(event_id),
            TransportMessage::Error { event_id, .. } => event_id.as_deref(),
            _ => None,
        }
    }
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
        /// F-1a bilateral tip exchange — per-Space tip event_id known to this Node
        /// (runbook §3.3 Locked wire shape). BTreeMap for deterministic JSON ordering
        /// (precondition for signed canonical form). #[serde(default)] for pre-F-1a
        /// peer back-compat — absent field deserialises as empty map, which under the
        /// locked semantics means "no shared Spaces" or, when a space_id is present
        /// in shared_spaces but absent from tips, "send full history for that Space."
        #[serde(default)]
        tips: BTreeMap<String, String>,
        timestamp: String,
        /// WebSocket endpoint URL of the initiating Node (advisory; excluded from signature).
        #[serde(skip_serializing_if = "Option::is_none")]
        node_endpoint: Option<String>,
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
        /// F-1a bilateral tip exchange — receiver-side counterpart of Hello.tips.
        /// See Hello.tips above for full semantics (runbook §3.3 Locked wire shape).
        #[serde(default)]
        tips: BTreeMap<String, String>,
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
        /// AI declaration (spec 3.6.10). Omitted from canonical form when `false`
        /// so the signature of pre-3.6.10 human registrations is unchanged.
        #[serde(default, skip_serializing_if = "is_false")]
        is_ai: bool,
        /// AI capability flag set (spec 3.6.10.3). Required when `is_ai = true`;
        /// MUST be omitted/null when `is_ai = false`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ai_capabilities: Option<AiCapabilities>,
        #[serde(skip_serializing_if = "Option::is_none")]
        trust_assertion: Option<Value>,
        /// Re-registration flag for orphan recovery (spec 3.6.3 / 3.13.8). When
        /// `true`, the Node permits registration of an already-known `identity_id`
        /// (re-home) instead of rejecting it as a duplicate. Omitted from the
        /// canonical form when `false` so signatures of normal registrations are
        /// byte-identical to pre-3.13.8 ones (mirrors `is_ai`).
        #[serde(default, skip_serializing_if = "is_false")]
        re_registration: bool,
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

// ── Phase 2 Event content structs ───────────────────────────────────────────────
//
// These structs represent the `content` field of DAG events. They are separate
// from the control message enums below, which are sent as standalone protocol
// messages outside the Event envelope.

/// Content for state.node_priority events (spec 3.9.3 Layer 5a).
/// Space owner declares manual ordering of federated Nodes for conflict resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateNodePriorityContent {
    /// Ordered from highest priority (index 0) to lowest.
    pub ordered_nodes: Vec<String>,
}

/// Content for state.dm_promote events (spec 3.16.3).
/// Signed by the Node (not either member) — protocol state change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDmPromoteContent {
    pub space_id: String,
    pub proposed_by: String,
    pub confirmed_by: String,
    pub new_name: String,
    pub promoted_at: String,
}

/// Content for state.space_migrate events (spec 3.12.7).
/// Permanent DAG record of a completed Space migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSpaceMigrateContent {
    pub space_id: String,
    pub source_node_id: String,
    pub destination_node_id: String,
    pub destination_node_url: String,
    pub migrated_at: String,
}

// ── DM Space promotion control messages (spec 3.16.3) ──────────────────────────

/// DM Space promotion control messages — sent between client and Node.
/// These are NOT Events and NOT stored in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DmControlMessage {
    /// Initiating member proposes promotion of a DM Space (3.16.3 step 1).
    #[serde(rename = "dm.promote_propose")]
    PromotePropose {
        protocol_version: String,
        space_id: String,
        proposed_name: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Other member confirms the promotion proposal (3.16.3 step 3).
    #[serde(rename = "dm.promote_confirm")]
    PromoteConfirm {
        protocol_version: String,
        space_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Other member rejects the promotion proposal (3.16.3 step 3).
    #[serde(rename = "dm.promote_reject")]
    PromoteReject {
        protocol_version: String,
        space_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

// ── Space migration control messages (spec 3.12.3–3.12.8) ──────────────────────

/// Space migration control messages — sent between owner client, source Node,
/// and destination Node. These are NOT Events and NOT stored in the DAG
/// (except state.space_migrate which IS an Event in the DAG).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MigrationMessage {
    /// Owner sends to source Node to initiate migration (3.12.3).
    #[serde(rename = "migration.request")]
    Request {
        protocol_version: String,
        space_id: String,
        destination_node_id: String,
        destination_node_url: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Source Node proposes migration to destination Node (3.12.3).
    #[serde(rename = "migration.propose")]
    Propose {
        protocol_version: String,
        space_id: String,
        source_node_id: String,
        space_auth_tier: u32,
        event_count: u64,
        estimated_size_bytes: u64,
        owner_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Destination Node accepts the migration proposal (3.12.3).
    #[serde(rename = "migration.accept")]
    Accept {
        protocol_version: String,
        space_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Destination Node rejects the migration proposal (3.12.3).
    #[serde(rename = "migration.reject")]
    Reject {
        protocol_version: String,
        space_id: String,
        /// One of: insufficient_storage, version_incompatible, policy_rejected, already_hosting
        reason: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Source Node notifies owner that migration failed (3.12.3).
    #[serde(rename = "migration.failed")]
    Failed {
        protocol_version: String,
        space_id: String,
        reason: String,
        timestamp: String,
    },
    /// Batch of Events transferred from source to destination (3.12.4).
    /// Also used for tail batches (Events produced during transfer, 3.12.5).
    #[serde(rename = "migration.event_batch")]
    EventBatch {
        protocol_version: String,
        space_id: String,
        batch_index: u64,
        events: Vec<Event>,
        batch_hash: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Destination acknowledges a received batch (3.12.4).
    #[serde(rename = "migration.batch_ack")]
    BatchAck {
        protocol_version: String,
        space_id: String,
        batch_index: u64,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Source signals end of Event transfer (3.12.5).
    #[serde(rename = "migration.transfer_complete")]
    TransferComplete {
        protocol_version: String,
        space_id: String,
        total_events: u64,
        dag_tips: Vec<String>,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Destination confirms successful verification (3.12.6).
    #[serde(rename = "migration.verified")]
    Verified {
        protocol_version: String,
        space_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Destination reports verification failure (3.12.6).
    #[serde(rename = "migration.verification_failed")]
    VerificationFailed {
        protocol_version: String,
        space_id: String,
        reason: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Source notifies federated peers of new home Node (3.12.8).
    #[serde(rename = "migration.federation_notify")]
    FederationNotify {
        protocol_version: String,
        space_id: String,
        new_node_id: String,
        new_node_url: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

// ── Identity replication messages (spec 3.13.4) ─────────────────────────────────

/// Identity replication messages — sent between home Node and replica Nodes.
/// These are NOT Events and NOT stored in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IdentityReplicateMessage {
    /// Home Node pushes Identity record to a replica Node (3.13.4).
    /// Signed by the Identity keypair.
    #[serde(rename = "identity.replicate")]
    Replicate {
        protocol_version: String,
        identity_id: String,
        identity_record: Value,
        update_version: u64,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Replica Node acknowledges successful replication (3.13.4).
    #[serde(rename = "identity.replicate_ack")]
    ReplicateAck {
        protocol_version: String,
        identity_id: String,
        update_version: u64,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Identity notifies the network of a new home Node after orphan recovery
    /// (spec 3.13.8 / 3.13.9). Signed by the Identity keypair. Delta-shaped —
    /// peers re-point the stored record's `home_node`; a peer holding no prior
    /// record treats it as a no-op.
    #[serde(rename = "identity.home_changed")]
    HomeChanged {
        protocol_version: String,
        identity_id: String,
        old_home_node_id: String,
        new_home_node_id: String,
        new_home_node_url: String,
        update_version: u64,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

// ── Bootstrap Node protocol messages (spec 3.14.3, 3.14.7) ─────────────────────

/// Bootstrap Node protocol messages — sent between a Node and a Bootstrap Node.
/// These are NOT Events and NOT stored in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BootstrapMessage {
    /// New Node registers with a Bootstrap Node (3.14.3).
    /// Signed by the registrant Node's keypair.
    #[serde(rename = "bootstrap.register")]
    Register {
        protocol_version: String,
        node_id: String,
        endpoint: String,
        region: String,
        capabilities: Vec<String>,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Bootstrap Node acknowledges registration (3.14.3).
    #[serde(rename = "bootstrap.register_ack")]
    RegisterAck {
        protocol_version: String,
        node_id: String,
        directory_url: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Node pings Bootstrap Node to refresh its directory entry before TTL expiry (3.14.7).
    #[serde(rename = "bootstrap.keepalive")]
    Keepalive {
        protocol_version: String,
        node_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Bootstrap Node acknowledges keepalive and resets TTL (3.14.7).
    #[serde(rename = "bootstrap.keepalive_ack")]
    KeepaliveAck {
        protocol_version: String,
        node_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Node explicitly removes itself from a Bootstrap Node's directory (3.14.7).
    #[serde(rename = "bootstrap.deregister")]
    Deregister {
        protocol_version: String,
        node_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

// ── Reputation messages (spec 3.15.3) ──────────────────────────────────────────

/// Reputation protocol messages — sent from a Node to a Bootstrap Node.
/// These are NOT Events and NOT stored in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ReputationMessage {
    /// Node reports a defederation event to a Bootstrap Node (3.15.3).
    #[serde(rename = "reputation.defederation_signal")]
    DefederationSignal {
        protocol_version: String,
        reporting_node_id: String,
        defederated_node_id: String,
        space_id: String,
        reason: String,
        evidence_event_ids: Vec<String>,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

// ── MLS (E2E encryption) protocol messages (spec 3.10.3, 3.10.5) ───────────────

/// MLS protocol messages — sent between clients and their home Node.
/// These wrap TLS-serialised MLS structures as base64url strings.
/// These are NOT Events and NOT stored in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MlsMessage {
    /// Client uploads a KeyPackage to its home Node (3.10.3).
    #[serde(rename = "mls.key_package")]
    KeyPackage {
        protocol_version: String,
        identity_id: String,
        device_id: String,
        /// Base64url-encoded TLS-serialised MLS KeyPackage per RFC 9420.
        mls_key_package: String,
        uploaded_at: String,
        valid_until: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Node acknowledges KeyPackage upload.
    #[serde(rename = "mls.key_package_ack")]
    KeyPackageAck {
        protocol_version: String,
        identity_id: String,
        device_id: String,
        timestamp: String,
    },
    /// Node requests a KeyPackage for a given Identity from a peer Node.
    #[serde(rename = "mls.key_package_request")]
    KeyPackageRequest {
        protocol_version: String,
        identity_id: String,
        device_id: String,
        room_id: String,
        space_id: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Node responds with a requested KeyPackage.
    #[serde(rename = "mls.key_package_response")]
    KeyPackageResponse {
        protocol_version: String,
        identity_id: String,
        device_id: String,
        /// Base64url-encoded TLS-serialised MLS KeyPackage per RFC 9420.
        mls_key_package: String,
        valid_until: String,
        timestamp: String,
    },
    /// MLS Commit advances the group to a new epoch (3.10.5).
    #[serde(rename = "mls.commit")]
    Commit {
        protocol_version: String,
        room_id: String,
        space_id: String,
        epoch: u64,
        /// Base64url-encoded TLS-serialised MLS MLSMessage of type commit.
        mls_commit: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// MLS Welcome delivered to a newly added group member (3.10.5).
    #[serde(rename = "mls.welcome")]
    Welcome {
        protocol_version: String,
        room_id: String,
        space_id: String,
        recipient_identity_id: String,
        recipient_device_id: String,
        /// Base64url-encoded TLS-serialised MLS Welcome message.
        mls_welcome: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// MLS Proposal routed to group members (3.10.5).
    #[serde(rename = "mls.proposal")]
    Proposal {
        protocol_version: String,
        room_id: String,
        space_id: String,
        epoch: u64,
        /// Base64url-encoded TLS-serialised MLS MLSMessage of type proposal.
        mls_proposal: String,
        timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};

    // ── Pass 1 Commit 4a typed XGID test helpers ─────────────────────────────
    fn idx(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn rmx(s: &str) -> RoomXgid {
        RoomXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn spx(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }
    #[allow(dead_code)]
    fn evx(s: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(s.to_string()))
    }

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
            idx("xgen://pubkey/ed25519:abc"),
            rmx("xgen://hash/sha256:room"),
            spx("xgen://hash/sha256:space"),
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
        assert_eq!(
            ev.event_id.as_ref().map(|e| e.as_str()),
            Some("xgen://hash/sha256:abc")
        );
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
            node_id: Some("xgen://pubkey/ed25519:NODE".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"transport.auth_ok\""));
        assert!(json.contains("\"node_id\":\"xgen://pubkey/ed25519:NODE\""));
        match serde_json::from_str::<TransportMessage>(&json).unwrap() {
            TransportMessage::AuthOk { node_id, .. } => {
                assert_eq!(node_id.as_deref(), Some("xgen://pubkey/ed25519:NODE"));
            }
            other => panic!("expected AuthOk, got {other:?}"),
        }
    }

    #[test]
    fn transport_auth_ok_node_id_backcompat() {
        // M10.4 — `node_id` is additive optional: an old node omits it (skip_
        // serializing_if) and an old-shape JSON without it deserializes to None.
        let none = TransportMessage::AuthOk {
            protocol_version: "0.1".to_string(),
            identity_id: "xgen://pubkey/ed25519:abc".to_string(),
            timestamp: "2026-04-27T12:00:00.000Z".to_string(),
            node_id: None,
        };
        let json = serde_json::to_string(&none).unwrap();
        assert!(!json.contains("node_id"), "None node_id must be omitted: {json}");

        let legacy = r#"{"type":"transport.auth_ok","protocol_version":"0.1","identity_id":"xgen://pubkey/ed25519:abc","timestamp":"2026-04-27T12:00:00.000Z"}"#;
        match serde_json::from_str::<TransportMessage>(legacy).unwrap() {
            TransportMessage::AuthOk { node_id, .. } => assert_eq!(node_id, None),
            other => panic!("expected AuthOk, got {other:?}"),
        }
    }

    #[test]
    fn transport_event_accepted_round_trip() {
        // M6 §3.1 — positive accept signal carries event_id + accepted_at.
        let msg = TransportMessage::EventAccepted {
            protocol_version: "0.1".to_string(),
            event_id: "xgen://hash/sha256:abc123".to_string(),
            accepted_at: "2026-05-29T12:00:00.000Z".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"transport.event_accepted\""));
        assert!(json.contains("\"event_id\":\"xgen://hash/sha256:abc123\""));
        let parsed: TransportMessage = serde_json::from_str(&json).unwrap();
        // Accessor returns the id uniformly.
        assert_eq!(parsed.event_id(), Some("xgen://hash/sha256:abc123"));
        match parsed {
            TransportMessage::EventAccepted { event_id, accepted_at, .. } => {
                assert_eq!(event_id, "xgen://hash/sha256:abc123");
                assert_eq!(accepted_at, "2026-05-29T12:00:00.000Z");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn transport_error_with_event_id_round_trip() {
        // M6 §3.3 — Error pertaining to a rejected event carries its event_id.
        let msg = TransportMessage::Error {
            protocol_version: "0.1".to_string(),
            error_code: 4002,
            error_string: "predecessor_timeout".to_string(),
            timestamp: "2026-05-29T12:00:00.000Z".to_string(),
            event_id: Some("xgen://hash/sha256:def456".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"event_id\":\"xgen://hash/sha256:def456\""));
        let parsed: TransportMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_id(), Some("xgen://hash/sha256:def456"));
    }

    #[test]
    fn transport_error_without_event_id_is_omitted_and_backward_compatible() {
        // skip_serializing_if: an Error with no event_id must not emit the field
        // (byte-identical to a pre-M6 Error on the wire).
        let msg = TransportMessage::Error {
            protocol_version: "0.1".to_string(),
            error_code: 1001,
            error_string: "bad frame".to_string(),
            timestamp: "2026-05-29T12:00:00.000Z".to_string(),
            event_id: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.contains("event_id"), "event_id must be omitted when None: {json}");
        assert_eq!(msg.event_id(), None);

        // A pre-M6 Error JSON (no event_id field at all) must still deserialise,
        // defaulting event_id to None.
        let legacy = r#"{"type":"transport.error","protocol_version":"0.1","error_code":1001,"error_string":"bad frame","timestamp":"2026-05-29T12:00:00.000Z"}"#;
        let parsed: TransportMessage = serde_json::from_str(legacy).unwrap();
        assert_eq!(parsed.event_id(), None);
        assert!(matches!(parsed, TransportMessage::Error { event_id: None, .. }));
    }

    #[test]
    fn transport_event_id_accessor_none_for_non_event_messages() {
        let goodbye = TransportMessage::Goodbye {
            protocol_version: "0.1".to_string(),
            reason: "bye".to_string(),
            timestamp: "2026-05-29T12:00:00.000Z".to_string(),
        };
        assert_eq!(goodbye.event_id(), None);
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

    // ── Phase 2 round-trip tests ────────────────────────────────────────────

    #[test]
    fn state_node_priority_content_round_trip() {
        let c = StateNodePriorityContent {
            ordered_nodes: vec![
                "xgen://node/ed25519:AAA".to_string(),
                "xgen://node/ed25519:BBB".to_string(),
            ],
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: StateNodePriorityContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.ordered_nodes.len(), 2);
        assert_eq!(parsed.ordered_nodes[0], "xgen://node/ed25519:AAA");
    }

    #[test]
    fn state_dm_promote_content_round_trip() {
        let c = StateDmPromoteContent {
            space_id: "xgen://hash/sha256:space".to_string(),
            proposed_by: "xgen://pubkey/ed25519:AAAA".to_string(),
            confirmed_by: "xgen://pubkey/ed25519:BBBB".to_string(),
            new_name: "Our Project".to_string(),
            promoted_at: "2026-04-30T10:00:31.000Z".to_string(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: StateDmPromoteContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.new_name, "Our Project");
        assert_eq!(parsed.proposed_by, "xgen://pubkey/ed25519:AAAA");
    }

    #[test]
    fn state_space_migrate_content_round_trip() {
        let c = StateSpaceMigrateContent {
            space_id: "xgen://hash/sha256:space".to_string(),
            source_node_id: "xgen://pubkey/ed25519:CCCC".to_string(),
            destination_node_id: "xgen://pubkey/ed25519:DDDD".to_string(),
            destination_node_url: "wss://destination.example.com/xgen".to_string(),
            migrated_at: "2026-04-30T10:01:36.000Z".to_string(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: StateSpaceMigrateContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.destination_node_url, "wss://destination.example.com/xgen");
    }

    #[test]
    fn dm_promote_propose_round_trip() {
        let msg = DmControlMessage::PromotePropose {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            proposed_name: "Our Project".to_string(),
            timestamp: "2026-04-30T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"dm.promote_propose\""));
        let parsed: DmControlMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DmControlMessage::PromotePropose { .. }));
    }

    #[test]
    fn dm_promote_confirm_round_trip() {
        let msg = DmControlMessage::PromoteConfirm {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            timestamp: "2026-04-30T10:00:30.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"dm.promote_confirm\""));
        assert!(matches!(
            serde_json::from_str::<DmControlMessage>(&json).unwrap(),
            DmControlMessage::PromoteConfirm { .. }
        ));
    }

    #[test]
    fn dm_promote_reject_round_trip() {
        let msg = DmControlMessage::PromoteReject {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            timestamp: "2026-04-30T10:00:30.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"dm.promote_reject\""));
    }

    #[test]
    fn migration_request_round_trip() {
        let msg = MigrationMessage::Request {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            destination_node_id: "xgen://pubkey/ed25519:DDDD".to_string(),
            destination_node_url: "wss://destination.example.com/xgen".to_string(),
            timestamp: "2026-04-30T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"migration.request\""));
        let parsed: MigrationMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, MigrationMessage::Request { .. }));
    }

    #[test]
    fn migration_propose_round_trip() {
        let msg = MigrationMessage::Propose {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            source_node_id: "xgen://pubkey/ed25519:CCCC".to_string(),
            space_auth_tier: 1,
            event_count: 4821,
            estimated_size_bytes: 2_048_576,
            owner_id: "xgen://pubkey/ed25519:AAAA".to_string(),
            timestamp: "2026-04-30T10:00:01.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"migration.propose\""));
        match serde_json::from_str::<MigrationMessage>(&json).unwrap() {
            MigrationMessage::Propose { event_count, .. } => assert_eq!(event_count, 4821),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn migration_accept_round_trip() {
        let msg = MigrationMessage::Accept {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            timestamp: "2026-04-30T10:00:02.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"migration.accept\""));
    }

    #[test]
    fn migration_reject_round_trip() {
        let msg = MigrationMessage::Reject {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            reason: "insufficient_storage".to_string(),
            timestamp: "2026-04-30T10:00:02.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"migration.reject\""));
        assert!(json.contains("insufficient_storage"));
    }

    #[test]
    fn migration_event_batch_round_trip() {
        let msg = MigrationMessage::EventBatch {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            batch_index: 0,
            events: vec![],
            batch_hash: "xgen://hash/sha256:batchhash".to_string(),
            timestamp: "2026-04-30T10:00:03.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"migration.event_batch\""));
        match serde_json::from_str::<MigrationMessage>(&json).unwrap() {
            MigrationMessage::EventBatch { batch_index, .. } => assert_eq!(batch_index, 0),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn migration_transfer_complete_round_trip() {
        let msg = MigrationMessage::TransferComplete {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            total_events: 4847,
            dag_tips: vec!["xgen://hash/sha256:tip1".to_string()],
            timestamp: "2026-04-30T10:01:30.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"migration.transfer_complete\""));
        match serde_json::from_str::<MigrationMessage>(&json).unwrap() {
            MigrationMessage::TransferComplete { total_events, dag_tips, .. } => {
                assert_eq!(total_events, 4847);
                assert_eq!(dag_tips.len(), 1);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn migration_verified_round_trip() {
        let msg = MigrationMessage::Verified {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            timestamp: "2026-04-30T10:01:35.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"migration.verified\""));
    }

    #[test]
    fn migration_verification_failed_round_trip() {
        let msg = MigrationMessage::VerificationFailed {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            reason: "event_count_mismatch".to_string(),
            timestamp: "2026-04-30T10:01:36.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"migration.verification_failed\""));
    }

    #[test]
    fn migration_federation_notify_round_trip() {
        let msg = MigrationMessage::FederationNotify {
            protocol_version: "0.1".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            new_node_id: "xgen://pubkey/ed25519:DDDD".to_string(),
            new_node_url: "wss://destination.example.com/xgen".to_string(),
            timestamp: "2026-04-30T10:01:38.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"migration.federation_notify\""));
    }

    #[test]
    fn identity_replicate_round_trip() {
        let msg = IdentityReplicateMessage::Replicate {
            protocol_version: "0.1".to_string(),
            identity_id: "xgen://pubkey/ed25519:AAAA".to_string(),
            identity_record: serde_json::json!({"display_name": "Alice"}),
            update_version: 1,
            timestamp: "2026-04-30T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"identity.replicate\""));
        match serde_json::from_str::<IdentityReplicateMessage>(&json).unwrap() {
            IdentityReplicateMessage::Replicate { update_version, .. } => {
                assert_eq!(update_version, 1);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn identity_replicate_ack_round_trip() {
        let msg = IdentityReplicateMessage::ReplicateAck {
            protocol_version: "0.1".to_string(),
            identity_id: "xgen://pubkey/ed25519:AAAA".to_string(),
            update_version: 1,
            timestamp: "2026-04-30T10:00:01.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"identity.replicate_ack\""));
    }

    #[test]
    fn bootstrap_register_round_trip() {
        let msg = BootstrapMessage::Register {
            protocol_version: "0.1".to_string(),
            node_id: "xgen://pubkey/ed25519:NNNN".to_string(),
            endpoint: "wss://newnode.example.com/xgen".to_string(),
            region: "EU".to_string(),
            capabilities: vec!["xgen.federation".to_string()],
            timestamp: "2026-04-30T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"bootstrap.register\""));
        match serde_json::from_str::<BootstrapMessage>(&json).unwrap() {
            BootstrapMessage::Register { region, capabilities, .. } => {
                assert_eq!(region, "EU");
                assert_eq!(capabilities, vec!["xgen.federation"]);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn bootstrap_register_ack_round_trip() {
        let msg = BootstrapMessage::RegisterAck {
            protocol_version: "0.1".to_string(),
            node_id: "xgen://pubkey/ed25519:NNNN".to_string(),
            directory_url: "https://bootstrap.example.com/xgen-directory".to_string(),
            timestamp: "2026-04-30T10:00:01.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"bootstrap.register_ack\""));
    }

    #[test]
    fn bootstrap_keepalive_round_trip() {
        let msg = BootstrapMessage::Keepalive {
            protocol_version: "0.1".to_string(),
            node_id: "xgen://pubkey/ed25519:NNNN".to_string(),
            timestamp: "2026-04-30T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"bootstrap.keepalive\""));
    }

    #[test]
    fn bootstrap_deregister_round_trip() {
        let msg = BootstrapMessage::Deregister {
            protocol_version: "0.1".to_string(),
            node_id: "xgen://pubkey/ed25519:NNNN".to_string(),
            timestamp: "2026-04-30T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"bootstrap.deregister\""));
    }

    #[test]
    fn reputation_defederation_signal_round_trip() {
        let msg = ReputationMessage::DefederationSignal {
            protocol_version: "0.1".to_string(),
            reporting_node_id: "xgen://pubkey/ed25519:AAAA".to_string(),
            defederated_node_id: "xgen://pubkey/ed25519:CCCC".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            reason: "repeated_protocol_violations".to_string(),
            evidence_event_ids: vec![
                "xgen://hash/sha256:ev1".to_string(),
                "xgen://hash/sha256:ev2".to_string(),
            ],
            timestamp: "2026-04-30T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"reputation.defederation_signal\""));
        match serde_json::from_str::<ReputationMessage>(&json).unwrap() {
            ReputationMessage::DefederationSignal { evidence_event_ids, reason, .. } => {
                assert_eq!(evidence_event_ids.len(), 2);
                assert_eq!(reason, "repeated_protocol_violations");
            }
        }
    }

    #[test]
    fn mls_key_package_round_trip() {
        let msg = MlsMessage::KeyPackage {
            protocol_version: "0.1".to_string(),
            identity_id: "xgen://pubkey/ed25519:AAAA".to_string(),
            device_id: "xgen://pubkey/ed25519:BBBB".to_string(),
            mls_key_package: "base64url-encoded-kp".to_string(),
            uploaded_at: "2026-04-26T10:00:00.000Z".to_string(),
            valid_until: "2026-07-26T00:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"mls.key_package\""));
        match serde_json::from_str::<MlsMessage>(&json).unwrap() {
            MlsMessage::KeyPackage { mls_key_package, .. } => {
                assert_eq!(mls_key_package, "base64url-encoded-kp");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn mls_key_package_ack_round_trip() {
        let msg = MlsMessage::KeyPackageAck {
            protocol_version: "0.1".to_string(),
            identity_id: "xgen://pubkey/ed25519:AAAA".to_string(),
            device_id: "xgen://pubkey/ed25519:BBBB".to_string(),
            timestamp: "2026-04-26T10:00:01.000Z".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"mls.key_package_ack\""));
    }

    #[test]
    fn mls_commit_round_trip() {
        let msg = MlsMessage::Commit {
            protocol_version: "0.1".to_string(),
            room_id: "xgen://hash/sha256:room".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            epoch: 3,
            mls_commit: "base64url-encoded-commit".to_string(),
            timestamp: "2026-04-26T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"mls.commit\""));
        match serde_json::from_str::<MlsMessage>(&json).unwrap() {
            MlsMessage::Commit { epoch, .. } => assert_eq!(epoch, 3),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn mls_welcome_round_trip() {
        let msg = MlsMessage::Welcome {
            protocol_version: "0.1".to_string(),
            room_id: "xgen://hash/sha256:room".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            recipient_identity_id: "xgen://pubkey/ed25519:CCCC".to_string(),
            recipient_device_id: "xgen://pubkey/ed25519:DDDD".to_string(),
            mls_welcome: "base64url-encoded-welcome".to_string(),
            timestamp: "2026-04-26T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"mls.welcome\""));
        match serde_json::from_str::<MlsMessage>(&json).unwrap() {
            MlsMessage::Welcome { recipient_identity_id, .. } => {
                assert_eq!(recipient_identity_id, "xgen://pubkey/ed25519:CCCC");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn mls_proposal_round_trip() {
        let msg = MlsMessage::Proposal {
            protocol_version: "0.1".to_string(),
            room_id: "xgen://hash/sha256:room".to_string(),
            space_id: "xgen://hash/sha256:space".to_string(),
            epoch: 3,
            mls_proposal: "base64url-encoded-proposal".to_string(),
            timestamp: "2026-04-26T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"mls.proposal\""));
    }

    #[test]
    fn event_type_from_str_all_phase2_variants() {
        let cases = [
            "state.node_priority", "state.dm_promote", "state.space_migrate",
            "state.ai_operator_delegate", "state.ai_operator_revoke",
            "dm.promote_propose", "dm.promote_confirm", "dm.promote_reject",
            "migration.request", "migration.propose", "migration.accept",
            "migration.reject", "migration.failed", "migration.event_batch",
            "migration.batch_ack", "migration.transfer_complete",
            "migration.verified", "migration.verification_failed",
            "migration.federation_notify",
            "identity.replicate", "identity.replicate_ack",
            "bootstrap.register", "bootstrap.register_ack",
            "bootstrap.keepalive", "bootstrap.keepalive_ack", "bootstrap.deregister",
            "reputation.defederation_signal",
            "mls.key_package", "mls.key_package_ack",
            "mls.key_package_request", "mls.key_package_response",
            "mls.commit", "mls.welcome", "mls.proposal",
        ];
        for s in cases {
            assert!(EventType::from_str(s).is_some(), "failed for {s}");
        }
    }

    // ── AI Identity extension (spec 3.6.10) ─────────────────────────────────

    #[test]
    fn ai_capabilities_round_trip() {
        let caps = AiCapabilities {
            dm_initiate: true,
            spontaneous_post: false,
            extra: Default::default(),
        };
        let json = serde_json::to_string(&caps).unwrap();
        let parsed: AiCapabilities = serde_json::from_str(&json).unwrap();
        assert!(parsed.dm_initiate);
        assert!(!parsed.spontaneous_post);
        assert!(parsed.extra.is_empty());
    }

    #[test]
    fn ai_capabilities_open_enum_preserves_extra_keys() {
        let json = r#"{"dm_initiate":false,"spontaneous_post":true,"com.example.extra":42}"#;
        let parsed: AiCapabilities = serde_json::from_str(json).unwrap();
        assert!(!parsed.dm_initiate);
        assert!(parsed.spontaneous_post);
        assert_eq!(parsed.extra.get("com.example.extra"), Some(&json!(42)));
        // Round-trip must keep the extra key.
        let re = serde_json::to_string(&parsed).unwrap();
        assert!(re.contains("com.example.extra"));
    }

    #[test]
    fn ai_capabilities_missing_required_field_fails() {
        // dm_initiate omitted — serde fails to deserialise.
        let json = r#"{"spontaneous_post":false}"#;
        let parsed: Result<AiCapabilities, _> = serde_json::from_str(json);
        assert!(parsed.is_err(), "missing dm_initiate must reject");
    }

    #[test]
    fn state_ai_operator_delegate_content_round_trip() {
        let c = StateAiOperatorDelegateContent {
            space_id: spx("xgen://hash/sha256:space"),
            ai_identity_id: idx("xgen://pubkey/ed25519:BOT"),
            new_operator_identity_id: idx("xgen://pubkey/ed25519:NEW_OP"),
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: StateAiOperatorDelegateContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.ai_identity_id.as_str(), "xgen://pubkey/ed25519:BOT");
        assert_eq!(parsed.new_operator_identity_id.as_str(), "xgen://pubkey/ed25519:NEW_OP");
    }

    #[test]
    fn state_ai_operator_revoke_content_round_trip() {
        let c = StateAiOperatorRevokeContent {
            space_id: spx("xgen://hash/sha256:space"),
            ai_identity_id: idx("xgen://pubkey/ed25519:BOT"),
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: StateAiOperatorRevokeContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.ai_identity_id.as_str(), "xgen://pubkey/ed25519:BOT");
    }

    #[test]
    fn event_type_ai_operator_variants_round_trip() {
        for v in [
            EventType::StateAiOperatorDelegate,
            EventType::StateAiOperatorRevoke,
        ] {
            let s = v.as_str();
            assert_eq!(EventType::from_str(s), Some(v));
        }
    }

    // ── Pacing rules (spec 3.7.12) ──────────────────────────────────────────

    #[test]
    fn event_type_state_space_pacing_round_trip() {
        let v = EventType::StateSpacePacing;
        assert_eq!(v.as_str(), "state.space_pacing");
        assert_eq!(EventType::from_str("state.space_pacing"), Some(v));
    }

    #[test]
    fn state_space_pacing_content_round_trip() {
        let c = StateSpacePacingContent {
            human_pacing_ms: 1500,
            ai_pacing_ms: 10_000,
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: StateSpacePacingContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.human_pacing_ms, 1500);
        assert_eq!(parsed.ai_pacing_ms, 10_000);
    }

    #[test]
    fn state_space_pacing_content_zero_valid() {
        // Spec 3.7.12.1: zero is valid and disables pacing.
        let c = StateSpacePacingContent {
            human_pacing_ms: 0,
            ai_pacing_ms: 0,
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: StateSpacePacingContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.human_pacing_ms, 0);
        assert_eq!(parsed.ai_pacing_ms, 0);
    }

    #[test]
    fn pacing_defaults_match_spec_3_7_12_2() {
        assert_eq!(DEFAULT_HUMAN_PACING_MS, 500);
        assert_eq!(DEFAULT_AI_PACING_MS, 2000);
    }

    // ── Temperature property (spec 3.7.13) ──────────────────────────────────

    #[test]
    fn event_type_temperature_variants_round_trip() {
        for v in [
            EventType::StateSpaceTemperatureVisibility,
            EventType::MembershipMute,
        ] {
            let s = v.as_str();
            assert_eq!(EventType::from_str(s), Some(v));
        }
    }

    #[test]
    fn temperature_visibility_default_is_moderator() {
        assert_eq!(DEFAULT_MEMBER_TEMPERATURE_VISIBILITY, "moderator");
        assert_eq!(VISIBILITY_MODERATOR, "moderator");
        assert_eq!(VISIBILITY_EVERYONE, "everyone");
        assert_eq!(VISIBILITY_SELF_ONLY, "self_only");
    }

    #[test]
    fn meta_atts_keys_are_reserved_xgen_namespace() {
        assert_eq!(META_ATT_ROOM_TEMPERATURE, "xgen.room_temperature");
        assert_eq!(META_ATT_MEMBER_TEMPERATURE, "xgen.member_temperature");
    }

    #[test]
    fn auto_temperature_reason_constant() {
        assert_eq!(REASON_AUTO_TEMPERATURE, "auto_temperature");
    }

    #[test]
    fn clamp_temperature_clamps_below_zero() {
        assert_eq!(clamp_temperature(-0.5), 0.0);
        assert_eq!(clamp_temperature(-1e6), 0.0);
    }

    #[test]
    fn clamp_temperature_clamps_above_one() {
        assert_eq!(clamp_temperature(1.5), 1.0);
        assert_eq!(clamp_temperature(1e6), 1.0);
    }

    #[test]
    fn clamp_temperature_preserves_in_range() {
        assert_eq!(clamp_temperature(0.0), 0.0);
        assert_eq!(clamp_temperature(0.5), 0.5);
        assert_eq!(clamp_temperature(1.0), 1.0);
    }

    #[test]
    fn clamp_temperature_handles_nan() {
        // NaN cannot be ordered — clamp to 0.0 (the conservative cool baseline).
        assert_eq!(clamp_temperature(f64::NAN), 0.0);
    }

    #[test]
    fn temperature_thresholds_round_trip() {
        let t = TemperatureThresholds {
            warm: 0.30,
            hot: 0.55,
            fiery: 0.80,
        };
        let json = serde_json::to_string(&t).unwrap();
        let parsed: TemperatureThresholds = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }

    #[test]
    fn temperature_thresholds_valid_orderings() {
        // Spec 3.7.13.2: 0.0 < warm < hot < fiery <= 1.0.
        let good = TemperatureThresholds { warm: 0.3, hot: 0.55, fiery: 0.8 };
        assert!(good.is_valid());
        let edge = TemperatureThresholds { warm: 0.1, hot: 0.5, fiery: 1.0 };
        assert!(edge.is_valid());
    }

    #[test]
    fn temperature_thresholds_invalid_orderings_rejected() {
        // warm not < hot
        let t = TemperatureThresholds { warm: 0.5, hot: 0.5, fiery: 0.8 };
        assert!(!t.is_valid());
        // hot not < fiery
        let t = TemperatureThresholds { warm: 0.3, hot: 0.8, fiery: 0.8 };
        assert!(!t.is_valid());
        // warm not > 0.0
        let t = TemperatureThresholds { warm: 0.0, hot: 0.5, fiery: 0.8 };
        assert!(!t.is_valid());
        // fiery > 1.0
        let t = TemperatureThresholds { warm: 0.3, hot: 0.5, fiery: 1.2 };
        assert!(!t.is_valid());
        // NaN
        let t = TemperatureThresholds { warm: f64::NAN, hot: 0.5, fiery: 0.8 };
        assert!(!t.is_valid());
    }

    #[test]
    fn state_space_temperature_visibility_content_round_trip() {
        let c = StateSpaceTemperatureVisibilityContent {
            member_temperature_visibility: VISIBILITY_EVERYONE.to_string(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: StateSpaceTemperatureVisibilityContent =
            serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.member_temperature_visibility, "everyone");
    }

    #[test]
    fn membership_mute_content_round_trip() {
        let c = MembershipMuteContent {
            target_identity: idx("xgen://pubkey/ed25519:MEMBER"),
            reason: REASON_AUTO_TEMPERATURE.to_string(),
            cooldown_until: "2026-05-15T14:00:00.000Z".to_string(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let parsed: MembershipMuteContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, c);
    }

    #[test]
    fn identity_register_with_ai_round_trip() {
        let msg = IdentityMessage::Register {
            protocol_version: "0.1".to_string(),
            identity_id: "xgen://pubkey/ed25519:BOT".to_string(),
            display_name: Some("Bot".to_string()),
            is_ai: true,
            ai_capabilities: Some(AiCapabilities {
                dm_initiate: false,
                spontaneous_post: false,
                extra: Default::default(),
            }),
            trust_assertion: None,
            re_registration: false,
            timestamp: "2026-05-15T10:00:00.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"is_ai\":true"));
        assert!(json.contains("\"dm_initiate\":false"));
        let parsed: IdentityMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            IdentityMessage::Register { is_ai, ai_capabilities, .. } => {
                assert!(is_ai);
                let caps = ai_capabilities.expect("ai_capabilities present");
                assert!(!caps.dm_initiate);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn identity_register_human_omits_is_ai_in_canonical_form() {
        // is_ai = false and ai_capabilities = None → both omitted from JSON.
        let msg = IdentityMessage::Register {
            protocol_version: "0.1".to_string(),
            identity_id: "xgen://pubkey/ed25519:ALICE".to_string(),
            display_name: Some("Alice".to_string()),
            is_ai: false,
            ai_capabilities: None,
            trust_assertion: None,
            re_registration: false,
            timestamp: "2026-05-15T10:00:00.000Z".to_string(),
            signature: None,
        };
        let v: serde_json::Value = serde_json::to_value(&msg).unwrap();
        let obj = v.as_object().unwrap();
        assert!(!obj.contains_key("is_ai"));
        assert!(!obj.contains_key("re_registration"));
        assert!(!obj.contains_key("ai_capabilities"));
    }

    #[test]
    fn identity_home_changed_round_trip() {
        let msg = IdentityReplicateMessage::HomeChanged {
            protocol_version: "0.1".to_string(),
            identity_id: "xgen://pubkey/ed25519:ALICE".to_string(),
            old_home_node_id: "xgen://pubkey/ed25519:OLD".to_string(),
            new_home_node_id: "xgen://pubkey/ed25519:NEW".to_string(),
            new_home_node_url: "wss://new.example.com/xgen".to_string(),
            update_version: 5,
            timestamp: "2026-04-30T10:00:00.000Z".to_string(),
            signature: Some("ed25519:AAAA:sig".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"identity.home_changed\""));
        let parsed: IdentityReplicateMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            IdentityReplicateMessage::HomeChanged {
                identity_id,
                old_home_node_id,
                new_home_node_id,
                new_home_node_url,
                update_version,
                ..
            } => {
                assert_eq!(identity_id, "xgen://pubkey/ed25519:ALICE");
                assert_eq!(old_home_node_id, "xgen://pubkey/ed25519:OLD");
                assert_eq!(new_home_node_id, "xgen://pubkey/ed25519:NEW");
                assert_eq!(new_home_node_url, "wss://new.example.com/xgen");
                assert_eq!(update_version, 5);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn identity_record_round_trip_preserves_ai_fields() {
        // Replication round-trip: an AI Identity record's is_ai and ai_capabilities
        // must survive serialisation via `identity_record` (3.6.10.9).
        use crate::identity::registry::{DeviceRecord, IdentityRecord};

        let record = IdentityRecord {
            identity_id: idx("xgen://pubkey/ed25519:BOT"),
            display_name: Some("Bot".to_string()),
            is_ai: true,
            ai_capabilities: Some(AiCapabilities {
                dm_initiate: true,
                spontaneous_post: false,
                extra: Default::default(),
            }),
            registered_at: "2026-05-15T10:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![DeviceRecord {
                device_id: "xgen://pubkey/ed25519:BOT".to_string(),
                device_name: None,
                authorised_at: "2026-05-15T10:00:00.000Z".to_string(),
            }],
            home_node: xgen_common::xgid::NodeXgid::from_xgid(Xgid::new(
                "xgen://pubkey/ed25519:NODE".to_string(),
            )),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        };
        let v = serde_json::to_value(&record).unwrap();

        // Wrap in IdentityReplicateMessage to mirror the on-the-wire shape.
        let msg = IdentityReplicateMessage::Replicate {
            protocol_version: "0.1".to_string(),
            identity_id: "xgen://pubkey/ed25519:BOT".to_string(),
            identity_record: v,
            update_version: 0,
            timestamp: "2026-05-15T10:00:01.000Z".to_string(),
            signature: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        // Round-trip through JSON.
        let parsed: IdentityReplicateMessage = serde_json::from_str(&json).unwrap();
        let replayed_record: IdentityRecord = match parsed {
            IdentityReplicateMessage::Replicate { identity_record, .. } => {
                serde_json::from_value(identity_record).unwrap()
            }
            _ => panic!("wrong variant"),
        };
        assert!(replayed_record.is_ai);
        let caps = replayed_record.ai_capabilities.expect("must carry caps");
        assert!(caps.dm_initiate);
        assert!(!caps.spontaneous_post);
    }

    #[test]
    fn identity_record_human_omits_ai_fields_on_replication() {
        use crate::identity::registry::IdentityRecord;
        let record = IdentityRecord {
            identity_id: idx("xgen://pubkey/ed25519:ALICE"),
            display_name: Some("Alice".to_string()),
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-15T10:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: xgen_common::xgid::NodeXgid::from_xgid(Xgid::new(
                "xgen://pubkey/ed25519:NODE".to_string(),
            )),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        };
        let v = serde_json::to_value(&record).unwrap();
        let obj = v.as_object().unwrap();
        assert!(!obj.contains_key("is_ai"));
        assert!(!obj.contains_key("ai_capabilities"));
    }

    #[test]
    fn event_type_as_str_round_trips_for_phase2() {
        let variants = [
            EventType::StateNodePriority,
            EventType::StateDmPromote,
            EventType::StateSpaceMigrate,
            EventType::DmPromotePropose,
            EventType::MigrationTransferComplete,
            EventType::MigrationVerified,
            EventType::MigrationVerificationFailed,
            EventType::BootstrapRegister,
            EventType::ReputationDefederationSignal,
            EventType::MlsKeyPackage,
            EventType::MlsCommit,
            EventType::MlsWelcome,
            EventType::MlsProposal,
        ];
        for v in variants {
            let s = v.as_str();
            let parsed = EventType::from_str(s).unwrap_or_else(|| panic!("from_str failed for {s}"));
            assert_eq!(parsed, v);
        }
    }
}
