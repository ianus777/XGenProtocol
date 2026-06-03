# XGen Protocol — Appendix I: Data Structures
> **Status:** ACTIVE  
> Version: 1.6  
> Date: May 2026  
> **Last updated:** 2026-06-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Overview

This appendix is the canonical reference for every named data structure in the XGen Protocol. It covers wire-format message types, runtime state objects, and bootstrap/reputation objects drawn from the Phase 1 and Phase 2 specification (Ch3 §3.1–§3.16) and confirmed against the reference implementation in `xgen-common` and `xgen-core`.

Structures are grouped by functional domain. For each structure, the field table lists:

- **Field** — the Rust field name in the reference implementation
- **Wire key** — the JSON key as it appears on the wire (identical to field name unless noted)
- **Type** — Rust type and corresponding JSON type
- **Req/Opt** — whether the field is required on the wire (Req) or may be omitted (Opt)
- **Description** — meaning and constraints

**Convention notes:**
- All field names use `snake_case` (§3.1.3).
- `null` is forbidden in all protocol messages (§3.1.5). Absent optional fields are omitted entirely.
- All datetime values use RFC 3339 UTC format: `"2026-05-15T10:00:00.000Z"`.
- All binary content (signatures, public keys, key material) is base64url-encoded.
- URI formats follow §3.1.6: `xgen://pubkey/ed25519:<base64url>` for identity/node keys, `xgen://hash/sha256:<hex>` for content-addressed identifiers.
- **Typed XGID flavours (D-072 + D-073, Appendix J):** in Rust-typed structures (Parts V/VI/VIII), XGID-bearing fields use the flavour wrappers (`EventXgid`, `SpaceXgid`, `RoomXgid`, `TrustAssertionXgid`, `NodeXgid`, `IdentityXgid`) rather than plain `String`. The flavours are `#[serde(transparent)]`: on the wire each serialises byte-for-byte identically to the same URI as a plain `String` would have. Wire-format tables (Parts II/III/IV/IX/X) continue to show the JSON wire type (`string`, `array of string`, etc.) — the wire shape is unchanged by the typed-flavour adoption. Identifier slots that do not have a flavoured XGID today (e.g. `DeviceRecord.device_id` and session-binding nonces such as `FederationRelationship.session_id`) stay `String`; the latter are documented "NOT a flavoured XGID per D-072" in their Description columns.

---

## Part I — Event Envelope

### I.1 `Event`

**Source:** `xgen-common/src/wire.rs`  
**Spec:** §3.2.1  
**Description:** The universal signed protocol message. Every piece of protocol state — messages, membership changes, space configuration, encryption metadata — is an `Event`. Events are stored in the Space DAG, signed by their sender, and identified by a content-addressed URI.

| Field | Wire key | Type | Req/Opt | Description |
|---|---|---|---|---|
| `protocol_version` | `protocol_version` | `String` / string | Req | Protocol version string. Current value: `"0.1"`. |
| `event_type` | `type` | `EventType` / string | Req | Event type string from the registry (§I.2). Wire key is `"type"`. |
| `event_id` | `event_id` | `Option<EventXgid>` / string | Opt† | `xgen://hash/sha256:<hex>` — SHA-256 of the canonical form. Absent on outgoing (unsigned) events; required on all received events. |
| `sender` | `sender` | `IdentityXgid` / string | Req | `xgen://pubkey/ed25519:<base64url>` — Identity public key of the sender. |
| `room_id` | `room_id` | `RoomXgid` / string | Req | `xgen://hash/sha256:<hex>` of the `state.room_create` event. Empty string for space-level events (e.g., `state.space_create`, `membership.invite`). |
| `space_id` | `space_id` | `SpaceXgid` / string | Req | `xgen://hash/sha256:<hex>` of the `state.space_create` event. Empty for the `state.space_create` event itself. |
| `prev_events` | `prev_events` | `Vec<EventXgid>` / array | Req | Causal parents — list of `event_id` URIs this event depends on. Empty only for DAG root events (`state.space_create`, `state.room_create`, `state.dm_space_create`). Maximum fanin: 10. |
| `timestamp` | `timestamp` | `String` / string | Req | RFC 3339 UTC creation timestamp. Advisory — not used for ordering. |
| `content` | `content` | `Value` / object | Req | Event-type-specific payload. See Part IX for content schemas by event type. |
| `meta_atts` | `meta_atts` | `Option<Value>` / object | Opt | Extensible key-value metadata. Keys follow dot-namespaced convention (§3.1.3). `xgen.*` namespace reserved for protocol use. Third-party keys must use reverse-domain prefix. |
| `signature` | `signature` | `Option<String>` / string | Opt† | `ed25519:<base64url-pubkey>:<base64url-sig>` — Ed25519 signature over the canonical form. Absent on outgoing events; required on all received events. |

† `event_id` and `signature` are absent while constructing an outgoing event. Both are required on all received events. A Node MUST reject any received Event missing either field.

**Canonical form rules (§3.2.3):**  
Fields are sorted lexicographically by key. `event_id` and `signature` are excluded from the signed form. The canonical form is compact JSON (no whitespace).

**Event ID derivation:**  
`event_id = "xgen://hash/sha256:" + hex(SHA-256(canonical_bytes))`

**Signature format:**  
`signature = "ed25519:" + base64url(public_key_bytes) + ":" + base64url(sig_bytes)`  
The signature covers the canonical bytes. The public key in the signature must match the `sender` field.

---

### I.2 `EventType` Registry

**Source:** `xgen-common/src/wire.rs`  
**Spec:** §3.2.2, §3.9–§3.16  
**Description:** The registry of *known* event type strings. The `type` field is an **open namespace** (Arc B / PG-09, §3.2): a Node MUST accept, store in the DAG, and propagate an Event whose `type` is not in this registry (the forward-compatibility rule) but MUST NOT apply it to SpaceState. As-built, the reference implementation represents an unrecognised type as `EventType::Unknown(String)` (`xgen-common/src/wire.rs`) — deserialisation is tolerant (unknown → `Unknown`), while `EventType::from_str` stays strict (unknown → `None`) so subscription filters cannot name an unknown type.

**Phase 1 — Message events**

| Wire string | Description |
|---|---|
| `message.text` | Plain text message in a Room. |
| `message.file` | File reference (URI pointer; content is stored externally). |
| `message.reaction` | Emoji reaction to another event. |
| `message.redact` | Request to suppress display of a prior event. |

**Phase 1 — State events (stored in DAG, applied to SpaceState)**

| Wire string | Description |
|---|---|
| `state.space_create` | DAG root — creates a new Space. `room_id` and `space_id` are empty. `prev_events` must be empty. |
| `state.dm_space_create` | DAG root — creates a DM Space. Constraints active until `state.dm_promote`. |
| `state.room_create` | DAG root for a Room — creates a new Room within a Space. `room_id` is empty; `space_id` is set. `prev_events` must be empty. |
| `state.room_update` | Updates Room metadata (name, topic). |
| `state.space_update` | Updates Space metadata. |
| `state.federation_add` | Records approval of a new federated Node for this Space. |
| `membership.invite` | Owner/admin/moderator invites an Identity to the Space. |
| `membership.join` | Identity joins the Space or a Room. |
| `membership.leave` | Identity leaves the Space or a Room. |
| `membership.kick` | Moderator or above removes an Identity. |
| `membership.ban` | Admin or above permanently bans an Identity. |
| `membership.node_eject` | Node administrator force-ejects (removes + bans) an Identity (M6 A4-D1). Node-signed; `sender == home_node` authority. Content `{ target_identity, reason? }`. |
| `membership.node_unban` | Node administrator lifts a node-eject ban (M6 A4-D1). Node-signed; same authority. Content `{ target_identity, reason? }`. |
| `system.key_rotation` | Identity key rotation notification. |

**Phase 2 — State events**

| Wire string | Description |
|---|---|
| `state.node_priority` | Space owner declares manual Node ordering for conflict resolution (§3.9.3 Layer 5a). |
| `state.dm_promote` | Records completed DM Space promotion. Signed by the Node. Lifts DM constraints. |
| `state.space_migrate` | Permanent record of a completed Space migration (§3.12.7). |
| `membership.mute` | Moderator-or-higher silences a member for a bounded period without removing them. Supports the automated `auto_temperature` consequence (§3.7.13.6). |
| `state.space_pacing` | Owner-issued update of per-Space pacing rules `human_pacing_ms` and `ai_pacing_ms` (§3.7.12). |
| `state.space_temperature_visibility` | Owner-issued update of the per-Space `member_temperature_visibility` setting (§3.7.13.3). |
| `state.ai_operator_delegate` | Transfers the operator role for an AI Identity within a Space (§3.6.10.6). Accountability-only — no privilege grant. |
| `state.ai_operator_revoke` | Removes the operator role for an AI Identity within a Space without naming a replacement (§3.6.10.6). |

**Phase 2 — DM promotion control (not stored in DAG)**

| Wire string | Description |
|---|---|
| `dm.promote_propose` | Initiating member proposes DM Space promotion. |
| `dm.promote_confirm` | Other member confirms the promotion. |
| `dm.promote_reject` | Other member rejects the promotion. |

**Phase 2 — Space migration control (not stored in DAG, except `state.space_migrate`)**

| Wire string | Description |
|---|---|
| `migration.request` | Space owner initiates migration to a new Node. |
| `migration.propose` | Source Node proposes migration to destination Node. |
| `migration.accept` | Destination Node accepts the proposal. |
| `migration.reject` | Destination Node rejects the proposal. |
| `migration.failed` | Source Node notifies owner of failure. |
| `migration.event_batch` | Batch of Events transferred from source to destination. |
| `migration.batch_ack` | Destination acknowledges a received batch. |
| `migration.transfer_complete` | Source signals end of Event transfer. |
| `migration.verified` | Destination confirms successful verification. |
| `migration.verification_failed` | Destination reports verification failure. |
| `migration.federation_notify` | Source notifies federated peers of the new home Node. |

**Phase 2 — Identity replication (not stored in DAG)**

| Wire string | Description |
|---|---|
| `identity.replicate` | Home Node pushes Identity record to replica Node. |
| `identity.replicate_ack` | Replica Node acknowledges replication. |

**Phase 2 — Bootstrap (not stored in DAG)**

| Wire string | Description |
|---|---|
| `bootstrap.register` | Node registers with a Bootstrap Node. |
| `bootstrap.register_ack` | Bootstrap Node confirms registration. |
| `bootstrap.keepalive` | Node pings Bootstrap Node before TTL expiry. |
| `bootstrap.keepalive_ack` | Bootstrap Node resets TTL and acknowledges. |
| `bootstrap.deregister` | Node removes itself from the Bootstrap directory. |

**Phase 2 — Reputation (not stored in DAG)**

| Wire string | Description |
|---|---|
| `reputation.defederation_signal` | Node reports a defederation event to a Bootstrap Node. |

**Phase 2 — MLS / End-to-End Encryption (not stored in DAG)**

| Wire string | Description |
|---|---|
| `mls.key_package` | Client uploads an MLS KeyPackage to its home Node. |
| `mls.key_package_ack` | Node acknowledges KeyPackage upload. |
| `mls.key_package_request` | Node requests a KeyPackage for an Identity from a peer Node. |
| `mls.key_package_response` | Node responds with a requested KeyPackage. |
| `mls.commit` | MLS Commit — advances the group to a new epoch. |
| `mls.welcome` | MLS Welcome — delivered to a newly added group member. |
| `mls.proposal` | MLS Proposal — routed to group members. |

---

## Part II — Transport Layer Messages

### II.1 `TransportMessage`

**Source:** `xgen-core/src/wire/types.rs`  
**Spec:** §3.3.4, §3.3.9  
**Description:** Control messages exchanged on the WebSocket connection between a client and a Node. These are NOT Events — they carry no `event_id`, `sender`, `room_id`, or `prev_events`. All variants include `protocol_version`. The wire discriminant is the `type` field.

**`transport.challenge`** — sent by Node immediately after WebSocket connection is established.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `nonce` | string | Req | Random base64url challenge nonce. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |

**`transport.auth`** — sent by client in response to challenge. Signature covers nonce bytes only.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` — client's public key. |
| `nonce` | string | Req | The nonce from the challenge, echoed back. |
| `signature` | string | Req | Ed25519 signature over the raw nonce bytes. |

**`transport.auth_ok`** — sent by Node on successful authentication.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | The authenticated identity's pubkey URI. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |

**`transport.auth_fail`** — sent by Node on failed authentication, followed immediately by connection close.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `error_code` | u32 / number | Req | Error code in the 1xxx range. |
| `error_string` | string | Req | Human-readable description. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |

**`transport.error`** — general transport error.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `error_code` | u32 / number | Req | Domain-appropriate error code. |
| `error_string` | string | Req | Human-readable description. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `event_id` | string | Opt | Hash URI of the Event this error pertains to, when the rejection is about a specific Event submission (§3.3.10). Omitted for transport-level errors not tied to an Event. Lets the originator correlate a rejection to its in-flight submission. (M6) |

**`transport.event_accepted`** — positive Event-acceptance signal (§3.3.10); the wire-level sibling of `transport.error`. Sent to the originator after the submitted Event is validated and durably persisted, before fan-out. Originator-only; does not propagate. (M6)

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `event_id` | string | Req | Hash URI of the accepted Event — the same `event_id` the originator sees on the Event they submitted. |
| `accepted_at` | string | Req | RFC 3339 UTC timestamp of acceptance (trace/audit only). |

**`transport.goodbye`** — graceful connection close (§3.3.9).

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `reason` | string | Req | Short machine-readable reason string (e.g., `"session_expired"`). |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |

**`transport.sync_request`** — client requests missed Events since a given event_id.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `since` | string | Req | `event_id` URI — client requests all events after this point. |

**`transport.rate_limit`** — Node signals the client to back off.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `retry_after_ms` | u64 / number | Req | Milliseconds the client should wait before retrying. |

---

## Part III — Federation Messages

### III.1 `FederationMessage`

**Source:** `xgen-core/src/wire/types.rs`  
**Spec:** §3.4.2, §3.4.4  
**Description:** Messages exchanged during the federation handshake between two Nodes. Each message is signed by the sender's node keypair. `signature` is absent only while constructing an outgoing message — it is always required on received messages.

**`federation.hello`** — initiating Node opens the handshake.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the initiating Node. |
| `capabilities` | `FederationCapabilities` / object | Req | Serialisation formats and extensions supported. See §III.2. |
| `shared_spaces` | array of string | Req | `space_id` URIs of Spaces proposed for federation. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `node_endpoint` | string | Opt | WebSocket endpoint URL of the initiating Node. Advisory — excluded from signature. |
| `signature` | string | Opt† | Node keypair signature over canonical form. |

**`federation.capabilities`** — receiving Node replies with its own capabilities and the negotiated values.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the responding Node. |
| `capabilities` | `FederationCapabilities` / object | Req | Responding Node's supported capabilities. |
| `negotiated` | `NegotiatedCapabilities` / object | Req | Agreed serialisation format and protocol version. See §III.3. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Node keypair signature. |

**`federation.accept`** — initiating Node confirms negotiated capabilities and opens the active session.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the initiating Node. |
| `session_id` | string | Req | `xgen://hash/sha256:<hex>` — derived as `hash(sort([node_a_id, node_b_id]) + timestamp)`. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Node keypair signature. |

**`federation.reject`** — either Node refuses the handshake.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `node_id` | string | Req | Rejecting Node's pubkey URI. |
| `error_code` | u32 / number | Req | Error code in the 2xxx range. |
| `error_string` | string | Req | Human-readable reason. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Node keypair signature. |

**`federation.goodbye`** — either Node ends an active federation session.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `node_id` | string | Req | Departing Node's pubkey URI. |
| `reason` | string | Req | Short machine-readable reason string. |
| `session_id` | string | Req | The session being terminated. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Node keypair signature. |

### III.2 `FederationCapabilities`

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `serialisation` | array of string | Req | Supported serialisation formats (e.g., `["json", "msgpack"]`). Must include `"json"`. |
| `compression` | array of string | Req | Supported compression algorithms. Empty array if none. |
| `extensions` | array of string | Req | Optional capability tokens. `"xgen.bootstrap"` marks a Bootstrap Node. |

### III.3 `NegotiatedCapabilities`

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `serialisation` | string | Req | The agreed serialisation format for this session. |
| `protocol_version` | string | Req | The agreed protocol version for this session. |

### III.4 `SpaceControlMessage`

**Spec:** §3.7.10  
**Description:** Space-level control messages sent over an active federation connection. Not Events — carry no `event_id`, `sender`, or `prev_events`.

**`space.join_request`** — sent by a new federated Node to request participation in a Space.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `space_id` | string | Req | `xgen://hash/sha256:<hex>` of the Space. |
| `node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the requesting Node. |

---

## Part IV — Identity Protocol Messages

### IV.1 `IdentityMessage`

**Source:** `xgen-core/src/wire/types.rs`  
**Spec:** §3.6.3–§3.6.8  
**Description:** Messages exchanged during Identity registration, lookup, and update. `identity.register` and `identity.update` are signed by the Identity keypair. Response messages are unsigned.

**`identity.register`** — client requests registration.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` — the new Identity's public key. |
| `display_name` | string | Opt | Human-readable name. UTF-8, max 64 characters. |
| `is_ai` | bool | Opt | AI declaration (§3.6.10). Default `false`. Omitted from the canonical form when `false` so signatures of pre-3.6.10 human registrations are unchanged. Immutable after registration. |
| `ai_capabilities` | object | Opt | `AiCapabilities` payload (§V.3). Required when `is_ai = true`; MUST be omitted when `is_ai = false`. Validated at step 8 of registration (§3.6.10.4). |
| `trust_assertion` | object | Opt | Auth-Tier-specific trust evidence (§3.8). `null` forbidden — omit if not applicable. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Identity keypair signature over canonical form. Required on wire. |

**`identity.register_ok`** — Node confirms successful registration.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | The registered Identity's pubkey URI. |
| `registered_at` | string | Req | RFC 3339 UTC timestamp of registration. |

**`identity.register_fail`** — Node rejects registration.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `error_code` | u32 / number | Req | Error code in the 3xxx range. |
| `error_string` | string | Req | Human-readable reason. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |

**`identity.get`** — client or Node requests an Identity record.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | The Identity to look up. |

**`identity.record`** — Node responds with the full Identity record.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | The Identity's pubkey URI. |
| `display_name` | string | Opt | Human-readable name. Absent if not set. |
| `is_ai` | bool | Opt | AI declaration mirrored from the stored record (§3.6.10). Default `false`; omitted from serialised output when `false`. |
| `ai_capabilities` | object | Opt | `AiCapabilities` payload (§V.3). Present iff `is_ai = true`. |
| `registered_at` | string | Req | RFC 3339 UTC timestamp of registration. |
| `devices` | array of `IdentityDeviceEntry` | Req | Authorised devices. See §IV.2. |
| `home_node` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the home Node. |

**`identity.not_found`** — Node cannot find the requested Identity.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | The Identity that was not found. |

**`identity.update`** — client updates its Identity record; signed.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | Identity being updated. |
| `update_version` | u64 / number | Req | Monotonic counter. Must be strictly greater than the stored version. |
| `changes` | object | Req | Key-value map of fields to update (e.g., `{"display_name": "Alice"}`). Updates to `is_ai` are rejected by the Node with error code 3041 `ai_role_violation` (§3.6.10.5, §3.6.10.10) — the AI declaration is fixed at registration. The 3041 wire name was widened from `ai_flag_immutable` in M3 (D-064) to cover both `is_ai` immutability and AI role structural violations under one umbrella code. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Identity keypair signature. Required on wire. |

### IV.2 `IdentityDeviceEntry`

**Spec:** §3.6.6  
**Description:** A single device associated with an Identity, embedded in `identity.record` messages.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `device_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` — the device keypair's public key. |
| `device_name` | string | Opt | Human-readable label for the device. |
| `authorised_at` | string | Req | RFC 3339 UTC timestamp when the device was added. |

---

## Part V — Identity Runtime Objects

### V.1 `IdentityRecord`

**Source:** `xgen-core/src/identity/registry.rs`  
**Spec:** §3.6.6  
**Description:** Full Identity record stored persistently on the home Node and replicated to federated Nodes. The in-memory and on-disk representation; not transmitted directly on the wire (the `identity.record` message carries the public subset).

| Field | Type | Description |
|---|---|---|
| `identity_id` | `IdentityXgid` | `xgen://pubkey/ed25519:<base64url>` — primary key. |
| `display_name` | `Option<String>` | Human-readable name. Absent if not set. |
| `is_ai` | `bool` | AI declaration (§3.6.10). Default `false`. Immutable after registration — enforced at apply time on `identity.update`. Skipped from the serialised JSON output when `false` so canonical forms of pre-3.6.10 human records are unchanged. |
| `ai_capabilities` | `Option<AiCapabilities>` | Capability flag set (§V.3). Required (`Some`) when `is_ai = true`; MUST be `None` when `is_ai = false`. Skipped from the serialised JSON output when `None`. |
| `registered_at` | `String` | RFC 3339 UTC timestamp of registration. |
| `trust_assertion` | `Option<Value>` | Auth-Tier-specific trust evidence. Present only for Tier 2+. |
| `devices` | `Vec<DeviceRecord>` | Authorised devices. See §V.2. |
| `home_node` | `NodeXgid` | `xgen://pubkey/ed25519:<base64url>` of the home Node. |
| `update_version` | `u64` | Monotonic counter for update propagation (§3.6.8). Starts at 0. |

### V.2 `DeviceRecord`

**Source:** `xgen-core/src/identity/registry.rs`  
**Spec:** §3.6.6  
**Description:** A single device entry in an `IdentityRecord`. Stored on the Node; mirrors `IdentityDeviceEntry` from the wire format.

| Field | Type | Description |
|---|---|---|
| `device_id` | `String` | `xgen://pubkey/ed25519:<base64url>` — device keypair's public key. |
| `device_name` | `Option<String>` | Human-readable device label. Absent if not set. |
| `authorised_at` | `String` | RFC 3339 UTC timestamp when the device was authorised. |

### V.3 `AiCapabilities`

**Source:** `xgen-common/src/wire.rs`  
**Spec:** §3.6.10.3  
**Description:** AI capability flag set carried by an AI Identity. Phase 2 defines two required boolean keys; an open `extra` map carries any additional capability keys for forward compatibility — older Nodes ignore unknown keys, newer Nodes may enforce them. Serialised as a flat JSON object: `{"dm_initiate": bool, "spontaneous_post": bool, ...extra_keys}`.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `dm_initiate` | bool | Req | Whether this AI Identity may initiate `state.dm_space_create` against another Identity. Protocol-enforced at DM creation (§3.6.10.4). |
| `spontaneous_post` | bool | Req | Whether this AI Identity may post in a Room without first being addressed. Phase 2 is informational — Node-side enforcement is deferred to Phase 3. |
| `extra` | `BTreeMap<String, Value>` | Opt | Forward-compatibility map. Additional capability keys from future spec revisions; flattened into the serialised object via `#[serde(flatten)]`. Unknown keys MUST be preserved on round-trip. |

**Validation:**
- Both required fields MUST be present in the serialised form; deserialisation rejects messages missing either.
- The `extra` map MUST be preserved across registration, replication, and round-trip serialisation so that a Node forwarding a record between protocol versions does not silently drop unknown capability flags.

---

## Part VI — Space & Room Runtime State

### VI.1 `SpaceState`

**Source:** `xgen-core/src/space/state.rs`  
**Spec:** §3.7.1–§3.7.9, §3.9, §3.16  
**Description:** In-memory runtime state of a Space, derived by replaying State Events from the DAG in causal order. Rebuilt on Node restart by replaying the full DAG. Not persisted directly — the DAG is the source of truth.

| Field | Type | Description |
|---|---|---|
| `space_id` | `SpaceXgid` | `xgen://hash/sha256:<hex>` — the `event_id` of the `state.space_create` event. |
| `name` | `Option<String>` | Space display name. Absent if not set. Set by `state.space_create` or `state.dm_promote`. |
| `topic` | `Option<String>` | Space topic string. Absent if not set. |
| `auth_tier` | `u32` | Auth Tier (1–4). Determines size limits and cryptographic requirements. |
| `max_event_size` | `Option<u64>` | Space-level size override in bytes. If absent, the Tier ceiling applies. Immutable after creation. |
| `home_node` | `NodeXgid` | `xgen://pubkey/ed25519:<base64url>` of the authoritative Node. Updated after successful migration. |
| `owner_id` | `IdentityXgid` | `xgen://pubkey/ed25519:<base64url>` of the Space creator. |
| `is_dm` | `bool` | True for DM Spaces created via `state.dm_space_create`. |
| `members` | `HashMap<IdentityXgid, SpaceMember>` | Active members, keyed by `identity_id`. |
| `pending_invites` | `HashMap<IdentityXgid, PendingInvite>` | Invited but not yet joined, keyed by `identity_id`. Carries `role` plus `invited_by` (M3 spec 3.6.10.6) for `resolve_operator` step 2. |
| `ai_operator_delegations` | `HashMap<IdentityXgid, IdentityXgid>` | Operator delegations for AI members (spec 3.6.10.6). Key: `ai_identity_id`; value: currently-delegated operator's `identity_id`. Updated by `state.ai_operator_delegate` / `state.ai_operator_revoke`. |
| `banned` | `HashSet<IdentityXgid>` | Identity IDs that are permanently banned from the Space. |
| `rooms` | `HashMap<RoomXgid, RoomState>` | Rooms within the Space, keyed by `room_id`. |
| `federation_nodes` | `Vec<NodeXgid>` | Node IDs of federated peers with `state.federation_add` events recorded. |
| `node_priority_order` | `Vec<NodeXgid>` | Manual Node ordering from the most recent `state.node_priority` event. Empty when no such event exists. Index 0 is highest priority. |
| `dm_constraints_active` | `bool` | True for DM Spaces until `state.dm_promote` is applied. Blocks: additional invites, second Room creation, federation. |
| `human_pacing_ms` | `u64` | Minimum send interval (ms) for members with `is_ai = false` (§3.7.12.1). Default `500` (`DEFAULT_HUMAN_PACING_MS`) when absent from `state.space_create`. Zero is valid and disables pacing for the human class. |
| `ai_pacing_ms` | `u64` | Minimum send interval (ms) for members with `is_ai = true` (§3.7.12.1). Default `2000` (`DEFAULT_AI_PACING_MS`) when absent from `state.space_create`. Zero is valid and disables pacing for the AI class. |
| `member_temperature_visibility` | `String` | Visibility setting for `xgen.member_temperature` (§3.7.13.3). Open enum — standard values are `moderator` (default), `everyone`, `self_only`. Unknown values are stored verbatim but treated as `moderator` at enforcement time. |
| `active_mutes` | `HashMap<IdentityXgid, String>` | Currently active mutes (§3.7.8). Key: target `identity_id`. Value: RFC 3339 `cooldown_until` timestamp. Members with an entry MUST NOT be permitted to post `message.*` Events until the timestamp passes. |

### VI.2 `RoomState`

**Source:** `xgen-core/src/space/state.rs`  
**Spec:** §3.7.3  
**Description:** In-memory state of a single Room within a Space. Derived from the DAG.

| Field | Type | Description |
|---|---|---|
| `room_id` | `RoomXgid` | `xgen://hash/sha256:<hex>` — the `event_id` of the `state.room_create` event. |
| `space_id` | `SpaceXgid` | `xgen://hash/sha256:<hex>` — parent Space ID. |
| `name` | `String` | Room display name. Set at creation; updated by `state.room_update`. |
| `topic` | `Option<String>` | Room topic. Absent if not set. |
| `members` | `HashSet<IdentityXgid>` | Identity IDs of members currently in this Room. |

### VI.3 `SpaceMember`

**Source:** `xgen-core/src/space/state.rs`  
**Spec:** §3.7.8  
**Description:** A single active member entry within `SpaceState.members`.

| Field | Type | Description |
|---|---|---|
| `identity_id` | `IdentityXgid` | `xgen://pubkey/ed25519:<base64url>` — the member's Identity. |
| `role` | `Role` | The member's role in this Space. See §VI.4. |
| `joined_at` | `String` | RFC 3339 UTC timestamp of the `membership.join` event. |
| `invited_by` | `Option<IdentityXgid>` | Identity that signed the `membership.invite` event admitting this member. `None` for the Space owner and for members admitted without an explicit invite (e.g. pre-M3 replay). Used by `resolve_operator` step 2 (spec 3.6.10.6). |

### VI.4 `Role`

**Source:** `xgen-core/src/space/membership.rs`  
**Spec:** §3.7.8  
**Description:** Space member role. Roles are ordered by privilege: `Member < Moderator < Admin < Owner`. The wire representation is the lowercase string.

| Variant | Wire string | Capabilities |
|---|---|---|
| `Member` | `"member"` | Send messages; join/leave Rooms. |
| `Moderator` | `"moderator"` | Member + can invite, kick. |
| `Admin` | `"admin"` | Moderator + can ban, create Rooms. |
| `Owner` | `"owner"` | Admin + can configure Space, migrate, set federation. Only one Owner per Space. |

**Permission table (§3.7.8):**

| Operation | Minimum role |
|---|---|
| Send message | Member |
| Join Room | Member |
| Invite | Moderator |
| Kick | Moderator |
| Create Room | Admin |
| Ban | Admin |
| Configure Space | Owner |
| Initiate migration | Owner |
| Set node priority | Owner |

### VI.5 `TemperatureThresholds`

**Source:** `xgen-common/src/wire.rs`  
**Spec:** §3.7.13.2  
**Description:** Threshold table published by the home Node as part of the Room metadata response (§3.7.7). Defines the boundaries between the four temperature buckets (cool, warm, hot, fiery) for client-side bucket derivation. All three fields are required when the table is present. A malformed table is omitted by the Node and clients fall back to Ch6 default thresholds.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `warm` | f64 | Req | Lower boundary of the warm bucket. Values below `warm` are cool. |
| `hot` | f64 | Req | Lower boundary of the hot bucket. |
| `fiery` | f64 | Req | Lower boundary of the fiery bucket. |

**Validity constraint:** `0.0 < warm < hot < fiery <= 1.0`. NaN values are rejected. The `is_valid()` helper enforces this; Nodes MUST omit malformed tables from the metadata response.

### VI.6 Reserved Constants (Temperature Property)

**Source:** `xgen-common/src/wire.rs`  
**Spec:** §3.7.8, §3.7.13  
**Description:** Constants reserved by the protocol for the temperature property (§3.7.13). Listed here as the canonical reference. Implementations MUST use these literal values; client code that emits or consumes these keys MUST match exactly.

**Reserved `meta_atts` keys (`xgen.*` namespace):**

| Constant | Wire value | Type | Description |
|---|---|---|---|
| `META_ATT_ROOM_TEMPERATURE` | `xgen.room_temperature` | float | Room-level temperature signal. Range `[0.0, 1.0]`. Always visible to every Room member (§3.7.13.3). Clamped to range before transmission via `clamp_temperature`. |
| `META_ATT_MEMBER_TEMPERATURE` | `xgen.member_temperature` | float | Per-member temperature signal. Range `[0.0, 1.0]`. Subject to `member_temperature_visibility` filtering per recipient (§3.7.13.4). |

**Reserved `reason` values:**

| Constant | Wire value | Used on | Description |
|---|---|---|---|
| `REASON_AUTO_TEMPERATURE` | `auto_temperature` | `membership.kick`, `membership.mute` | Marks the action as issued automatically by a temperature plugin (§3.7.13.6). Protocol behaviour is identical to a manually-issued action; the distinction is preserved on the DAG for audit. |

**Visibility values (open enum for `SpaceState.member_temperature_visibility`):**

| Constant | Wire value | Description |
|---|---|---|
| `VISIBILITY_MODERATOR` | `moderator` | Default. Visibility limited to moderators-or-higher plus the subject themselves. |
| `VISIBILITY_EVERYONE` | `everyone` | Every member sees every other member's temperature. |
| `VISIBILITY_SELF_ONLY` | `self_only` | Only the subject sees their own temperature. |
| `DEFAULT_MEMBER_TEMPERATURE_VISIBILITY` | `moderator` | Convenience alias for the default. |

**Pacing constants:**

| Constant | Value | Description |
|---|---|---|
| `DEFAULT_HUMAN_PACING_MS` | `500` | Protocol-recommended default for `human_pacing_ms` when absent from `state.space_create` (§3.7.12.2). |
| `DEFAULT_AI_PACING_MS` | `2000` | Protocol-recommended default for `ai_pacing_ms` when absent from `state.space_create` (§3.7.12.2). |

---

## Part VII — Node Objects

### VII.1 `NodeAnnouncement`

**Source:** `xgen-core/src/node/announcement.rs`  
**Spec:** §3.5.3–§3.5.6  
**Description:** A Node's self-signed public declaration. Signed by the node keypair so any peer can verify it without a third party (self-certifying). TTL is 90 days; announcements must be refreshed before expiry.

| Field | Wire key | Type | Req/Opt | Description |
|---|---|---|---|---|
| `protocol_version` | `protocol_version` | string | Req | `"0.1"` |
| `msg_type` | `type` | string | Req | Always `"node_announcement"`. Wire key is `"type"`. |
| `node_id` | `node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` — node identity; self-certifying (key is in the URI). |
| `endpoint` | `endpoint` | string | Req | Full WebSocket endpoint URI (e.g., `wss://node.example.com/xgen`). |
| `capabilities` | `capabilities` | `NodeCapabilities` / object | Req | Supported serialisation formats and extensions. See §VII.2. |
| `auth_tiers_served` | `auth_tiers_served` | array of u32 | Req | Auth Tiers this Node supports (e.g., `[1]` for Phase 1). |
| `operator_display_name` | `operator_display_name` | string | Opt | Human-readable operator name. |
| `bootstrap_info` | `bootstrap_info` | `BootstrapInfo` / object | Opt | Present only on Bootstrap Nodes. See §VII.3. |
| `announcement_version` | `announcement_version` | u64 / number | Req | Monotonically increasing. Higher version supersedes lower for the same `node_id`. |
| `valid_until` | `valid_until` | string | Req | RFC 3339 UTC expiry datetime. TTL is 90 days from `timestamp`. |
| `timestamp` | `timestamp` | string | Req | RFC 3339 UTC creation timestamp. |
| `signature` | `signature` | string | Req | Ed25519 signature over the canonical form (fixed field order; `signature` excluded). |

**Canonical field order for signing:** `protocol_version`, `type`, `node_id`, `endpoint`, `capabilities`, `auth_tiers_served`, `operator_display_name` (if present), `bootstrap_info` (if present), `announcement_version`, `valid_until`, `timestamp`.

### VII.2 `NodeCapabilities`

**Source:** `xgen-core/src/node/announcement.rs`  
**Spec:** §3.5.3  
**Description:** Capability declaration embedded in `NodeAnnouncement`. Phase 1 default: JSON-only, no compression, no extensions.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `serialisation` | array of string | Req | Supported serialisation formats. Must include `"json"`. Phase 1 default: `["json"]`. |
| `compression` | array of string | Req | Supported compression algorithms. Phase 1 default: `[]`. |
| `extensions` | array of string | Req | Optional capability tokens. `"xgen.bootstrap"` declares Bootstrap capability. Phase 1 default: `[]`. |

### VII.3 `BootstrapInfo`

**Source:** `xgen-core/src/bootstrap/capability.rs`  
**Spec:** §3.14.1  
**Description:** Bootstrap-specific extension on `NodeAnnouncement`. Present only on Bootstrap Nodes; paired with the `"xgen.bootstrap"` capability token in `capabilities.extensions`.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `directory_url` | string | Req | HTTPS URL of the Bootstrap Node's public directory endpoint. |
| `accepts_registrations` | bool | Req | Whether this Bootstrap Node accepts new Node registrations. |
| `region` | string | Req | Geographic region — operator-declared, for diversity routing. |
| `operator` | string | Req | Human-readable operator name. Informational only. |

---

## Part VIII — Federation Runtime Objects

### VIII.1 `FederationRelationship`

**Source:** `xgen-core/src/federation/registry.rs`  
**Spec:** §3.4.5  
**Description:** Persisted record of an active federation relationship. Stored in `xgen-node_federation.db`. Consulted on startup to re-establish connections without a new handshake. One record per peer Node, keyed by `peer_node_id`.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `peer_node_id` | `NodeXgid` | Req | `xgen://pubkey/ed25519:<base64url>` of the peer Node. Primary key. |
| `shared_spaces` | `Vec<SpaceXgid>` | Req | `space_id` URIs of Spaces federated with this peer. |
| `negotiated_version` | `String` | Req | The protocol version negotiated during the handshake. |
| `negotiated_serialisation` | `String` | Req | The serialisation format in use for this federation session. |
| `session_id` | `String` | Req | Session-binding nonce from `federation.accept`. NOT a flavoured XGID per D-072 "what XGID is not" — session IDs are ephemeral per-connection identifiers, not protocol-object handles. |
| `last_connected` | `String` | Req | RFC 3339 UTC timestamp of the last successful connection. |
| `peer_url` | `Option<String>` | Opt | WebSocket endpoint URL of the peer Node, if provided in `federation.hello`. Advisory. |

---

## Part IX — Event Content Schemas

Each `Event` carries a `content` object whose schema depends on `type`. This section defines the required and optional fields for each event type's content.

### IX.1 `state.space_create` content

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `name` | string | Req | Space display name. |
| `auth_tier` | u32 | Req | Auth Tier (1–4). |
| `home_node` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the hosting Node. |
| `nonce` | string | Req | Random base64url value ensuring event_id uniqueness. |
| `topic` | string | Opt | Space topic. |
| `max_event_size` | u64 | Opt | Space-level size override in bytes. Must be ≤ Tier ceiling. Immutable. |
| `human_pacing_ms` | u64 | Opt | Initial value for the per-Space `human_pacing_ms` rule (§3.7.12.1). Default `500` when absent. Zero disables human pacing. |
| `ai_pacing_ms` | u64 | Opt | Initial value for the per-Space `ai_pacing_ms` rule (§3.7.12.1). Default `2000` when absent. Zero disables AI pacing. |
| `member_temperature_visibility` | string | Opt | Initial value for the per-Space `member_temperature_visibility` setting (§3.7.13.3). Open enum; default `"moderator"` when absent. |

### IX.2 `state.dm_space_create` content

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `auth_tier` | u32 | Req | Auth Tier. Default: 1. |
| `invitee` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the other DM participant. |
| `home_node` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the hosting Node. |
| `nonce` | string | Req | Random base64url nonce. |

### IX.3 `state.room_create` content

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `name` | string | Req | Room display name. |
| `nonce` | string | Req | Random base64url nonce. |
| `topic` | string | Opt | Room topic. |

### IX.4 `state.federation_add` content

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the newly federated Node. |
| `session_id` | string | Req | The federation session ID. |
| `negotiated_version` | string | Req | Agreed protocol version. |
| `negotiated_serialisation` | string | Req | Agreed serialisation format. |

### IX.5 `membership.invite` content

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `target_identity` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the invited Identity. |
| `role` | string | Req | Assigned role string (`"member"`, `"moderator"`, `"admin"`). |

### IX.6 `membership.join` content

Empty object `{}` for space-level joins. No required fields.

### IX.7 `membership.kick` / `membership.ban` content

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `target_identity` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the kicked/banned Identity. |

### IX.8 `message.text` content

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `text` | string | Req | The message text. UTF-8. Maximum length subject to Space `max_event_size`. For E2E-encrypted rooms, this field carries the `enc:` prefix followed by a base64url ciphertext. |

### IX.9 `state.node_priority` content

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `ordered_nodes` | array of string | Req | Node ID URIs ordered from highest priority (index 0) to lowest. Used in state resolution Layer 5a (§3.9.3). |

### IX.10 `state.dm_promote` content

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `proposed_by` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the member who proposed promotion. |
| `confirmed_by` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the member who confirmed. |
| `new_name` | string | Req | The new Space name post-promotion. |
| `promoted_at` | string | Req | RFC 3339 UTC timestamp of promotion. |

### IX.11 `state.space_migrate` content

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `space_id` | string | Req | `xgen://hash/sha256:<hex>` of the migrated Space. |
| `source_node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the old host Node. |
| `destination_node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the new host Node. |
| `destination_node_url` | string | Req | WebSocket endpoint URL of the new host Node. |
| `migrated_at` | string | Req | RFC 3339 UTC timestamp of migration completion. |

### IX.12 `state.space_pacing` content

**Source:** `StateSpacePacingContent` (§3.7.12.3).  
Owner-issued update of per-Space pacing rules. Both fields are required — partial updates are not supported; the owner sets both values explicitly on each update. Rejected unless `sender == owner_id`.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `human_pacing_ms` | u64 | Req | New value for the `human_pacing_ms` rule. Zero is valid and disables human pacing. |
| `ai_pacing_ms` | u64 | Req | New value for the `ai_pacing_ms` rule. Zero is valid and disables AI pacing. |

### IX.13 `state.space_temperature_visibility` content

**Source:** `StateSpaceTemperatureVisibilityContent` (§3.7.13.3).  
Owner-issued update of the `member_temperature_visibility` setting. Rejected unless `sender == owner_id`. The value is stored verbatim (open enum); unknown values are treated as `moderator` at enforcement time.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `member_temperature_visibility` | string | Req | Open enum. Standard values: `moderator`, `everyone`, `self_only`. |

### IX.14 `membership.mute` content

**Source:** `MembershipMuteContent` (§3.7.8).  
Silences a member for a bounded period without removing them from the Space or Room. Permitted from moderator-or-higher. The mute auto-lifts at `cooldown_until`; until then the target MUST NOT be permitted to post `message.*` Events. The reserved `reason` value `auto_temperature` (§3.7.13.6) marks the mute as issued by an automated temperature plugin — protocol behaviour is identical, the distinction exists for audit.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `target_identity` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the muted Identity. |
| `reason` | string | Req | Free-text reason. Reserved value `auto_temperature` indicates an automated temperature-driven mute. |
| `cooldown_until` | string | Req | RFC 3339 UTC timestamp at which the mute auto-lifts. |

### IX.15 `state.ai_operator_delegate` content

**Source:** `StateAiOperatorDelegateContent` (§3.6.10.6).  
Records transfer of the operator role for an AI Identity within a Space. Signed by the current operator. Accountability-only — the event carries no privilege grant; the named new operator becomes the responsible party for subsequent AI behaviour in this Space.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `space_id` | string | Req | `xgen://hash/sha256:<hex>` of the Space scope. |
| `ai_identity_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the AI Identity whose operator changes. |
| `new_operator_identity_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the new operator. |

### IX.16 `state.ai_operator_revoke` content

**Source:** `StateAiOperatorRevokeContent` (§3.6.10.6).  
Removes the operator role for an AI Identity within a Space without naming a replacement. The inviter remains the responsible Identity by fallback.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `space_id` | string | Req | `xgen://hash/sha256:<hex>` of the Space scope. |
| `ai_identity_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the AI Identity whose operator is revoked. |

---

## Part X — Phase 2 Control Messages

### X.1 `DmControlMessage`

**Source:** `xgen-core/src/wire/types.rs`  
**Spec:** §3.16.3  
**Description:** DM Space promotion control messages sent between client and Node. Not Events — not stored in the DAG.

**`dm.promote_propose`** — initiating member proposes promotion.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The DM Space being promoted. |
| `proposed_name` | string | Req | The proposed name for the promoted Space. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Sender's Identity keypair signature. |

**`dm.promote_confirm`** — other member confirms.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The DM Space being promoted. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Sender's Identity keypair signature. |

**`dm.promote_reject`** — other member rejects.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The DM Space being promoted. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Sender's Identity keypair signature. |

### X.2 `MigrationMessage`

**Source:** `xgen-core/src/wire/types.rs`  
**Spec:** §3.12.3–§3.12.8  
**Description:** Space migration control messages exchanged between Space owner, source Node, and destination Node. Not Events (except `state.space_migrate` which is a DAG event — see §IX.11).

**`migration.request`** — owner sends to source Node to initiate.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The Space to migrate. |
| `destination_node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the target Node. |
| `destination_node_url` | string | Req | WebSocket endpoint URL of the target Node. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Owner's Identity keypair signature. |

**`migration.propose`** — source Node proposes to destination Node.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The Space to migrate. |
| `source_node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the source Node. |
| `space_auth_tier` | u32 | Req | Auth Tier of the Space — destination uses this for capacity planning. |
| `event_count` | u64 | Req | Number of Events in the Space DAG. |
| `estimated_size_bytes` | u64 | Req | Estimated total size of all Events. |
| `owner_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the Space owner. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Source Node keypair signature. |

**`migration.accept`** — destination accepts.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The Space being accepted. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Destination Node keypair signature. |

**`migration.reject`** — destination rejects.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The Space being rejected. |
| `reason` | string | Req | One of: `insufficient_storage`, `version_incompatible`, `policy_rejected`, `already_hosting`. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Destination Node keypair signature. |

**`migration.event_batch`** — batch of Events from source to destination.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The Space being transferred. |
| `batch_index` | u64 | Req | Zero-based batch sequence number. |
| `events` | array of `Event` | Req | The Event objects in this batch. |
| `batch_hash` | string | Req | `xgen://hash/sha256:<hex>` of the serialised batch content. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Source Node keypair signature. |

**`migration.batch_ack`** — destination acknowledges a batch.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The Space being transferred. |
| `batch_index` | u64 | Req | Index of the acknowledged batch. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Destination Node keypair signature. |

**`migration.transfer_complete`** — source signals end of Event transfer.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The Space being transferred. |
| `total_events` | u64 | Req | Total number of Events transferred across all batches. |
| `dag_tips` | array of string | Req | `event_id` URIs of the current DAG tips at transfer completion. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Source Node keypair signature. |

**`migration.verified`** — destination confirms successful verification.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The Space that was verified. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Destination Node keypair signature. |

**`migration.verification_failed`** — destination reports verification failure.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The Space that failed verification. |
| `reason` | string | Req | Machine-readable failure reason (e.g., `"event_count_mismatch"`, `"dag_tip_mismatch"`). |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Destination Node keypair signature. |

**`migration.federation_notify`** — source notifies federated peers.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The migrated Space. |
| `new_node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the new home Node. |
| `new_node_url` | string | Req | WebSocket endpoint URL of the new home Node. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Source Node keypair signature. |

**`migration.failed`** — source notifies owner of failure.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `space_id` | string | Req | The Space whose migration failed. |
| `reason` | string | Req | Human-readable failure reason. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |

### X.3 `IdentityReplicateMessage`

**Source:** `xgen-core/src/wire/types.rs`  
**Spec:** §3.13.4  
**Description:** Identity replication messages exchanged between home Node and replica Nodes. Not Events — not stored in the DAG.

**`identity.replicate`** — home Node pushes Identity record to replica.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the Identity being replicated. |
| `identity_record` | object | Req | Full Identity record payload (matches `IdentityRecord` schema). |
| `update_version` | u64 | Req | Monotonic version counter of this record state. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Home Node keypair signature. |

**`identity.replicate_ack`** — replica Node acknowledges.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | The Identity that was replicated. |
| `update_version` | u64 | Req | The version that was stored. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Replica Node keypair signature. |

### X.4 `BootstrapMessage`

**Source:** `xgen-core/src/wire/types.rs`  
**Spec:** §3.14.3, §3.14.7  
**Description:** Protocol messages between a Node and a Bootstrap Node. Not Events — not stored in the DAG.

**`bootstrap.register`** — Node registers with a Bootstrap Node.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the registering Node. |
| `endpoint` | string | Req | WebSocket endpoint URL of the registering Node. |
| `region` | string | Req | Geographic region — operator-declared. |
| `capabilities` | array of string | Req | Capability tokens declared by this Node. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Registrant Node keypair signature. |

**`bootstrap.register_ack`** — Bootstrap Node acknowledges registration.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `node_id` | string | Req | The registered Node's ID. |
| `directory_url` | string | Req | HTTPS URL of the Bootstrap Node's directory. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Bootstrap Node keypair signature. |

**`bootstrap.keepalive`** — Node refreshes its directory entry before TTL expiry.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `node_id` | string | Req | The Node's ID. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Node keypair signature. |

**`bootstrap.keepalive_ack`** — Bootstrap Node resets TTL and acknowledges.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `node_id` | string | Req | The Node whose TTL was reset. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Bootstrap Node keypair signature. |

**`bootstrap.deregister`** — Node explicitly removes itself.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `node_id` | string | Req | The Node deregistering. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Node keypair signature. |

### X.5 `ReputationMessage`

**Source:** `xgen-core/src/wire/types.rs`  
**Spec:** §3.15.3  
**Description:** Reputation protocol messages sent from a Node to a Bootstrap Node. Not Events — not stored in the DAG.

**`reputation.defederation_signal`** — Node reports a defederation event.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `reporting_node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the reporting Node. |
| `defederated_node_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the Node being reported. |
| `space_id` | string | Req | `xgen://hash/sha256:<hex>` of the affected Space. |
| `reason` | string | Req | Machine-readable reason (e.g., `"repeated_protocol_violations"`). |
| `evidence_event_ids` | array of string | Req | `event_id` URIs of supporting evidence events. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Reporting Node keypair signature. |

### X.6 `MlsMessage`

**Source:** `xgen-core/src/wire/types.rs`  
**Spec:** §3.10.3, §3.10.5  
**Description:** MLS (RFC 9420) protocol messages. MLS structures are TLS-serialised and wrapped as base64url strings. These messages are NOT Events and NOT stored in the DAG — they serve as the delivery layer for the cryptographic group operations that produce encrypted `message.text` content in DAG Events.

**`mls.key_package`** — client uploads KeyPackage to home Node.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the Identity. |
| `device_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the device keypair. |
| `mls_key_package` | string | Req | Base64url-encoded TLS-serialised MLS KeyPackage (RFC 9420). |
| `uploaded_at` | string | Req | RFC 3339 UTC timestamp of upload. |
| `valid_until` | string | Req | RFC 3339 UTC expiry datetime of the KeyPackage. |
| `signature` | string | Opt† | Identity keypair signature. |

**`mls.key_package_ack`** — Node acknowledges KeyPackage upload.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | The Identity whose KeyPackage was stored. |
| `device_id` | string | Req | The device keypair ID. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |

**`mls.key_package_request`** — Node requests KeyPackage for an Identity from a peer Node.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | The Identity whose KeyPackage is requested. |
| `device_id` | string | Req | The specific device ID. |
| `room_id` | string | Req | Room for which the KeyPackage is needed. |
| `space_id` | string | Req | Parent Space. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Requesting Node keypair signature. |

**`mls.key_package_response`** — Node responds with a requested KeyPackage.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `identity_id` | string | Req | The Identity whose KeyPackage is returned. |
| `device_id` | string | Req | The device keypair ID. |
| `mls_key_package` | string | Req | Base64url-encoded TLS-serialised MLS KeyPackage. |
| `valid_until` | string | Req | Expiry datetime of this KeyPackage. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |

**`mls.commit`** — MLS Commit advances the group to a new epoch.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `room_id` | string | Req | The Room whose MLS group is being updated. |
| `space_id` | string | Req | Parent Space. |
| `epoch` | u64 | Req | The new epoch number after this commit. |
| `mls_commit` | string | Req | Base64url-encoded TLS-serialised MLS `MLSMessage` of type Commit. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Committing member's Identity keypair signature. |

**`mls.welcome`** — MLS Welcome delivered to a newly added group member.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `room_id` | string | Req | The Room. |
| `space_id` | string | Req | Parent Space. |
| `recipient_identity_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the recipient Identity. |
| `recipient_device_id` | string | Req | `xgen://pubkey/ed25519:<base64url>` of the recipient device. |
| `mls_welcome` | string | Req | Base64url-encoded TLS-serialised MLS Welcome message. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Sender's Identity keypair signature. |

**`mls.proposal`** — MLS Proposal routed to group members.

| Field | Type | Req/Opt | Description |
|---|---|---|---|
| `protocol_version` | string | Req | `"0.1"` |
| `room_id` | string | Req | The Room. |
| `space_id` | string | Req | Parent Space. |
| `epoch` | u64 | Req | The current epoch when the proposal was generated. |
| `mls_proposal` | string | Req | Base64url-encoded TLS-serialised MLS `MLSMessage` of type Proposal. |
| `timestamp` | string | Req | RFC 3339 UTC timestamp. |
| `signature` | string | Opt† | Proposing member's Identity keypair signature. |

---

## Part XI — Bootstrap & Reputation Runtime Objects

### XI.1 `DirectoryEntry`

**Source:** `xgen-core/src/bootstrap/directory.rs`  
**Spec:** §3.14.2  
**Description:** A single Node entry in a Bootstrap Node's directory. Stored in `BootstrapDirectory` in memory. Exported in signed directory documents served over HTTPS.

| Field | Type | Description |
|---|---|---|
| `node_id` | `String` | `xgen://pubkey/ed25519:<base64url>` of the Node. Primary key. |
| `endpoint` | `String` | WebSocket endpoint URL of the Node. |
| `region` | `String` | Geographic region as declared by the Node during registration. |
| `last_seen` | `String` | RFC 3339 UTC timestamp of the last successful keepalive or registration. |
| `reputation_score` | `f64` | Computed reputation score in [0.0, 1.0]. Higher is better. Used for directory ordering. |

### XI.2 Bootstrap Directory Document

**Spec:** §3.14.2  
**Description:** The signed JSON document served at the Bootstrap Node's `directory_url`. Signed with the Bootstrap Node's keypair. Clients fetch this to discover Nodes. The document structure is not a named Rust struct — it is built dynamically by `sign_directory()`.

| Field | Type | Description |
|---|---|---|
| `bootstrap_node_id` | string | `xgen://pubkey/ed25519:<base64url>` of the signing Bootstrap Node. |
| `generated_at` | string | RFC 3339 UTC timestamp of document generation. |
| `nodes` | array of `DirectoryEntry` | Known Nodes, sorted by `reputation_score` descending. |
| `protocol_version` | string | `"0.1"` |
| `signature` | string | Ed25519 signature over the canonical form of the above fields (excluding `signature`). |

### XI.3 `ReputationComponents`

**Source:** `xgen-core/src/bootstrap/reputation.rs`  
**Spec:** §3.15.1  
**Description:** Per-component reputation signals maintained by a Bootstrap Node for each known Node. Internal to the Bootstrap Node — not exposed on the wire. The aggregate `reputation_score` in `DirectoryEntry` is derived from these components.

**Component weights (§3.15.1):**

| Component | Weight |
|---|---|
| Uptime ratio | 35% |
| Announcement freshness | 25% |
| Defederation count | 20% |
| Successful federations | 10% |
| Failed federations | 10% |

| Field | Type | Description |
|---|---|---|
| `uptime_ratio` | `f64` | Fraction of keepalive pings responded to. Range: [0.0, 1.0]. |
| `announcement_freshness` | `f64` | Freshness score: 1.0 within 24h, decays linearly to 0.0 at 90 days. |
| `defederation_count` | `u64` | Count of defederation signals received against this Node. Penalises score via `1.0 / (1.0 + count)`. |
| `successful_federations` | `u64` | Count of successful federation handshakes observed. |
| `failed_federations` | `u64` | Count of failed federation handshakes. |
| `protocol_violations` | `u64` | Count of confirmed protocol violations. Currently unused in score formula; reserved. |

**Score formula:**  
`score = uptime × 0.35 + freshness × 0.25 + (1/(1+defed_count)) × 0.20 + (success/(success+1)) × 0.10 + (1 - fails/(fails+success+1)) × 0.10`

**Merge rule (§3.15.2):**  
When Bootstrap Nodes share reputation data, each float component uses a 60/40 weighted average: `merged = local × 0.6 + remote × 0.4`. Integer counts use rounded weighted average.

---

## Part XII — Migration State Machine

### XII.1 `MigrationState`

**Source:** `xgen-core/src/migration/state_machine.rs`  
**Spec:** §3.12.2  
**Description:** Both source and destination Nodes maintain an independent migration state machine. The machine advances through states as protocol messages are exchanged.

| Variant | Description |
|---|---|
| `Idle` | No active migration. Initial state. Returned to after `Complete` or `Failed`. |
| `Negotiating` | Handshake in progress — source has received `migration.request`, destination has received `migration.propose`. Waiting for `migration.accept` or `migration.reject`. |
| `Transferring` | Transfer phase — source is sending `migration.event_batch` messages; destination is receiving and acknowledging. |
| `Verifying` | Transfer complete — destination is verifying Event count, DAG tips, and hash integrity. |
| `Complete` | Migration succeeded. `state.space_migrate` event committed to DAG. Source becomes non-authoritative. |
| `Failed { reason }` | Migration failed — reason is a human-readable string. Not transmitted on the wire; used for internal error reporting. |

**Migration error codes (§3.12):**

| Code | Constant | Meaning |
|---|---|---|
| 6001 | `migration_not_owner` | Requester is not the Space owner. |
| 6002 | `migration_already_hosting` | Destination already hosts this Space. |
| 6003 | `migration_insufficient_storage` | Destination lacks capacity. |
| 6004 | `migration_version_incompatible` | Incompatible protocol version. |
| 6005 | `migration_policy_rejected` | Destination rejected by local policy. |
| 6006 | `migration_wrong_state` | Unexpected message for current migration state. |

---

## Part XIII — Error Code Domains

All error codes are plain integers on the wire. For human-readable display, codes use an `E` prefix zero-padded to six digits (e.g., `E004002`). The `E` prefix is display-only.

| Range | Domain |
|---|---|
| 1000–1999 | Transport |
| 2000–2999 | Federation |
| 3000–3999 | Identity |
| 4000–4999 | State resolution |
| 5000–5999 | E2E encryption (MLS) |
| 6000–6999 | Space migration |
| 7000–7999 | Bootstrap |
| 8000–8999 | Reputation |
| 9000–9999 | DM Space promotion |

---

*End of Appendix I*
