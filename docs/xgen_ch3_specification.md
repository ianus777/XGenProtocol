# XGen Protocol — Chapter 3: Specification
> Status: wip  
> Version: 0.1  
> Date: April 2026  
> Last edited: April 2026  
> Language: English  
> Author: JozefN  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Overview

Chapter 3 translates the architectural commitments of Chapter 2 into precise, implementable specifications. Where Chapter 2 says *what* XGen is, Chapter 3 says *how* XGen works with enough precision to build it.

Chapter 3 is structured in two phases:

**Phase 1 — Minimal Viable Protocol** covers everything required for a first working test run: two Nodes connecting, a user registering an Identity, joining a Space, and exchanging a verified message. Phase 1 can be fully implemented and tested before Phase 2 begins.

**Phase 2 — Full Protocol** covers the harder algorithmic and institutional problems — state resolution, end-to-end encryption, higher-tier Auth Modules, and the remaining federation details. Phase 2 specifications are informed by implementation experience from Phase 1.

**Interface-first principle:** every section specifies interfaces and contracts completely, even when the internal algorithm is deferred to Phase 2. A developer can always build against an interface. They cannot build against an unspecified algorithm.

---

## Chapter 3 — Section Skeleton

**Phase 1 — Minimal Viable Protocol**

| Section | Title | Status |
|---|---|---|
| 3.1 | Wire Format | ✅ Complete |
| 3.2 | Event Specification | ✅ Complete |
| 3.3 | Transport Protocol | ✅ Complete |
| 3.4 | Federation Handshake | ✅ Complete |
| 3.5 | Node Identity Protocol | ✅ Complete |
| 3.6 | Identity Registration Protocol | ✅ Complete |
| 3.7 | Space & Room Protocol | ✅ Complete |
| 3.8 | Auth Module — Tier 1 Specification | ✅ Complete |

**Phase 2 — Full Protocol**

| Section | Title | Status |
|---|---|---|
| 3.9 | State Resolution Algorithm | deferred |
| 3.10 | End-to-End Encryption | deferred |
| 3.11 | Auth Module — Tiers 2–4 Interfaces | deferred |
| 3.12 | Space Migration Protocol | deferred |
| 3.13 | Identity Replication Parameters | deferred |
| 3.14 | Bootstrap Node Protocol | deferred |
| 3.15 | Node Reputation Format | deferred |
| 3.16 | DM Space Promotion Sequence | deferred |

---

## Phase 1 — Minimal Viable Protocol

### 3.1 Wire Format

*Status: wip*

The serialisation format for all XGen protocol messages. Covers:

- Primary format: JSON (human-readable, universally supported, debuggable)
- Field naming conventions
- Required vs optional fields
- Null and absent field handling
- URI format for all `xgen_uri`, `hash_uri`, `pubkey_uri` fields
- Datetime format: RFC 3339 UTC — `"2026-04-25T12:32:00.000Z"`
- Integer precision and numeric types
- Binary data encoding: base64url
- Maximum message size
- Versioning in messages

---

#### 3.1.1 Message Size Limits

Protocol messages carry structured data only — metadata, identifiers, signatures, and short text payloads. Binary content (images, files, audio, video) MUST be stored externally and referenced by URI. Base64url encoding is reserved for cryptographic material — signatures, public keys, and content hashes — not for file content.

> **Principle:** XGen is a signalling and coordination protocol, not a file transfer protocol. The size limit is the architectural enforcer of that boundary.

**Size reference table**

The table below gives the raw byte capacity and approximate usable JSON content for each power-of-two envelope size. JSON structural overhead (field names, quotes, braces, colons) is estimated at ~400 bytes per envelope. Character counts assume UTF-8 with predominantly ASCII content; non-Latin scripts consume 2–4 bytes per character.

| Size | Bytes | Chars (ASCII) | Usable JSON content | Notes |
|---|---|---|---|---|
| 2KB | 2,048 | ~2,048 | ~1,648 | Short signed state event |
| 4KB | 4,096 | ~4,096 | ~3,696 | Typical protocol message |
| 8KB | 8,192 | ~8,192 | ~7,792 | Long formal document reference |
| 16KB | 16,384 | ~16,384 | ~15,984 | Very large structured payload |
| 32KB | 32,768 | ~32,768 | ~32,368 | Book chapter as plain text |
| 64KB | 65,536 | ~65,536 | ~65,136 | Short novella as plain text |
| 128KB | 131,072 | ~131,072 | ~130,672 | Dev/testing only |
| 256KB | 262,144 | ~262,144 | ~261,744 | Dev/testing only |

*Note: these are work definitions established before implementation testing. Values may be revised downward when real-world Event sizes are measured during Phase 1 testing.*

**Two-layer size model**

Message size enforcement operates in two layers applied in order by the receiving Node:

**Layer 1 — Tier ceiling** (hard protocol limit, defined by spec)  
The Auth Tier of a Space defines the maximum possible envelope size for all Events in that Space. No Space configuration can exceed the Tier ceiling. Higher Tiers enforce smaller ceilings — higher trust contexts carry smaller attack surface.

**Layer 2 — Space override** (soft limit, declared at Space creation)  
The Space owner may declare a `max_event_size` at creation time that is tighter than the Tier ceiling. A Space operating at its Tier ceiling needs no explicit declaration. The Space override is immutable after creation — changing it mid-life creates ambiguity around Events already in the log that were valid under the prior limit. Space migration is the correct path if a different limit is required.

**Tier ceiling table**

| Tier | Context | Ceiling | Rationale |
|---|---|---|---|
| Local Node | Local development only | 256KB | No external federation — localhost only |
| Tier 1 | Community | 64KB | Generous for text; proven in federated protocols |
| Tier 2 | Professional | 32KB | Reduced surface; content goes out-of-band |
| Tier 3 | Corporate | 16KB | Protocol messages only |
| Tier 4 | Government | 8KB | Minimal surface; maximum predictability |

The descending direction is intentional: higher Auth Tier means smaller maximum envelope. Government-tier protocol messages — signed state events, membership changes, permission updates — are rarely larger than 2KB in practice. The 8KB ceiling is generous relative to real usage while enforcing the principle that high-trust Spaces do not embed content in protocol messages.

**Local Node mode**

Local Node is a named operating mode for development and testing, not an Auth Tier. It is structurally distinct from the Tier model in three ways. First, it does not appear in any wire format field — there is no `"tier": "local"` in any protocol message. Second, a Node operating in Local Node mode MUST refuse all external network connections — it accepts connections from localhost only. Third, Local Node mode is activated by a Node configuration flag (`local_node: true`), not by any protocol-level declaration. Because Local Node Spaces never federate externally, the 256KB envelope ceiling cannot be exploited over a network.

Local Node mode exists so implementers can develop and test against a real Node without Auth Module infrastructure. It is not a production deployment option. A Node MUST NOT enter Local Node mode if external network interfaces are active.

**Enforcement rule**

A Node receiving an Event MUST reject it if:
1. The serialized envelope exceeds the Tier ceiling for the Space's declared Auth Tier, OR
2. The serialized envelope exceeds the Space's declared `max_event_size` (if set).

Rejection MUST occur before signature verification and before any content processing.

---

#### 3.1.2 Primary Format and Format Agility

XGen treats serialisation format as a declared, negotiable capability — not a hardcoded protocol property. The same principle governs serialisation format as governs cryptographic algorithms: declare what you support, negotiate what you use, maintain a mandatory baseline that guarantees universal interoperability.

**JSON as mandatory baseline**

JSON (RFC 8259) is the mandatory baseline serialisation format. Every XGen Node MUST support JSON. It was chosen as the baseline for three reasons: it is human-readable and directly inspectable during development, it is universally supported across all target implementation languages without additional dependencies, and it produces unambiguous text output that is straightforward to sign and verify.

A Node that supports only JSON remains fully interoperable with every other Node on the network. JSON support cannot be dropped or negotiated away.

**Format agility**

Additional serialisation formats MAY be supported as optional capabilities declared during the federation handshake (3.4) and during client connection. When both parties declare a common non-JSON format, they MAY negotiate it for the session. The format in use for a session is fixed at connection time and does not change mid-session.

The set of supported formats is an open registry. New formats may be registered and adopted without a protocol version change, provided they can represent the full XGen message schema. Known candidate formats include MessagePack and CBOR, but the registry is not limited to these. A Node that does not recognise a proposed format MUST fall back to JSON rather than rejecting the connection.

The rationale for format agility is forward extensibility: serialisation technology continues to evolve. A format that does not exist today may offer meaningful advantages — in size, parse speed, schema validation, or cryptographic canonicalisation — when it appears. XGen does not close that door.

**Format identifier in transport framing**

Every message transmitted on the wire is prefixed by a format identifier that declares the serialisation format of the payload that follows. This prefix is part of the transport framing layer (3.3), not part of the message payload itself, and is not included in the signed content.

The format identifier is a length-prefixed UTF-8 string: one byte declaring the identifier length in bytes, followed by the identifier bytes. Using a human-readable string rather than a numeric code makes the framing self-describing and forward-extensible — new formats require only a new registered string, not an updated lookup table.

```
┌─────────────────────────────────────────────────────────────┐
│ Transport frame structure                                   │
├──────────┬──────────────────────────────────────────────────┤
│ 1 byte   │ Format identifier length (N)                     │
│ N bytes  │ Format identifier string (UTF-8)                 │
│ 4 bytes  │ Payload length in bytes (unsigned 32-bit int)    │
│ M bytes  │ Serialised message payload                       │
└──────────┴──────────────────────────────────────────────────┘
```

Registered format identifier strings for Phase 1:

| Identifier | Format | Status |
|---|---|---|
| `json` | JSON (RFC 8259) | Mandatory baseline |
| `msgpack` | MessagePack | Optional capability |
| `cbor` | CBOR (RFC 8949) | Optional capability |

**Framing example — JSON message**

A minimal `message.text` event serialised as JSON and wrapped in a transport frame:

```
0x04                     ; format identifier length: 4 bytes
'json'                   ; format identifier string
0x00 0x00 0x00 0xc8      ; payload length: 200 bytes
'{                       ; payload: JSON begins here
  "protocol_version": "0.1",
  "type": "message.text",
  "event_id": "xgen://hash/sha256:a3f9b2c1d4e8f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6",
  "sender": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIHvoNgEMoFYGNhWMTRSXqFGrjWYRBhKVNBnPXVwB",
  "room_id": "xgen://hash/sha256:b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3",
  "content": {
    "text": "Hello"
  },
  "timestamp": "2026-04-26T10:00:00.000Z",
  "signature": "algorithm:keyid:base64signaturebytes"
}'                       ; payload ends
```

The same event serialised as MessagePack, if negotiated:

```
0x07                     ; format identifier length: 7 bytes
'msgpack'                ; format identifier string
0x00 0x00 0x00 0x4e      ; payload length: 78 bytes (smaller than JSON)
0x85 0xa1 ...            ; payload: MessagePack bytes begin here (binary, not human-readable)
```

The parser reads the first byte to get the identifier length, reads that many bytes to get the format string, reads 4 bytes to get the payload length, then hands the payload bytes to the appropriate deserialiser. A parser encountering an unrecognised format identifier MUST close the connection with an error — it cannot safely deserialise an unknown format.

**Signing and format independence**

Signatures in XGen are computed over a canonical representation of the message fields (defined in 3.2), not over the serialised wire bytes. This means the same Event produces the same signature regardless of whether it is transmitted as JSON or MessagePack. Format negotiation does not affect signature verification. A Node receiving a MessagePack-encoded Event verifies its signature by deserialising the payload and computing the canonical form — the same process as for a JSON-encoded Event.

---

#### 3.1.3 Field Naming Conventions

All field names in XGen protocol messages use `snake_case` — lowercase letters, digits, and underscores only. No camelCase, no PascalCase, no hyphens. This convention applies uniformly to all protocol fields, meta-atts keys in the `xgen.*` namespace, and all field names in Auth Module message schemas.

Field names MUST be stable across protocol versions. A field name, once published in a released version of the spec, is permanent. Renaming a field is a breaking change and requires a new field name alongside the old one under a deprecation policy, not a silent replacement.

Implementations that encounter unknown field names MUST ignore them silently and MUST NOT reject the message on that basis alone. This is the forward-compatibility rule: new fields added in later protocol versions do not break older implementations.

---

#### 3.1.4 Required and Optional Fields

Every field in a protocol message is explicitly classified as either **required** or **optional** in its schema definition in Chapter 3.

A **required** field MUST be present in every message of that type. A receiving Node MUST reject a message that is missing any required field. Rejection on missing required fields occurs after size validation (3.1.1) and JSON parse validation (3.1.2), but before signature verification.

An **optional** field MAY be omitted entirely from a message. Omission and absence are the only valid representations of "not applicable" for an optional field. There is no null value in XGen protocol messages.

---

#### 3.1.5 Absent Fields and the Null Prohibition

XGen protocol messages do not use JSON `null`. The value `null` MUST NOT appear anywhere in a protocol message. A receiving Node MUST reject any message containing a `null` value.

The distinction between absent and null is meaningful and intentional. In many systems, `null` is used loosely to mean "not set", "unknown", "not applicable", or "explicitly cleared". These are four different semantic states, and collapsing them into a single `null` value produces ambiguity that is dangerous in a signed, append-only protocol log.

XGen resolves this cleanly: if a field does not apply to a given message, it is absent. An absent optional field and a present optional field carry different meaning. A field that has been explicitly cleared is represented by a dedicated state event, not by setting a field to null. Unknown values do not exist in protocol messages — the message either contains a valid value or the field is absent.

This prohibition also simplifies signature verification: the canonical form of a message never has to account for whether `null` and absent are equivalent.

---

#### 3.1.6 URI Formats

XGen uses three URI types as typed identifiers throughout the protocol. Each has a fixed grammar. All three use the `xgen:` scheme.

**xgen_uri** — the general-purpose XGen resource identifier.

```
xgen://<type>/<identifier>
```

Examples:
```
xgen://identity/ed25519:AAAAC3NzaC1lZDI1NTE5...   ← Identity URI
xgen://space/sha256:a3f9b2c1...                    ← Space URI
xgen://node/ed25519:BBBBD3NzaC1lZDI1NTE5...       ← Node URI
xgen://room/sha256:d4e8f1a2...                     ← Room URI
```

The `<type>` segment is an open enum using dot-namespaced names for extension types (e.g. `xgen.media`, `xgen.thread`). The `<identifier>` segment is the canonical identifier for that resource — typically a public key URI or hash URI as defined below.

**hash_uri** — a content-addressed identifier derived from a cryptographic hash.

```
xgen://hash/<algorithm>:<hexbytes>
```

Examples:
```
xgen://hash/sha256:a3f9b2c1d4e8f1a2b3c4d5e6f7a8b9c0...   ← SHA-256 content hash
xgen://hash/sha3-256:1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d...  ← SHA3-256 (algorithm-agile)
```

Hash URIs are used as Event IDs and as content integrity references for externally stored media. The algorithm prefix makes hash URIs algorithm-agile: upgrading the hash algorithm requires no change to the URI structure, only a new algorithm name.

**pubkey_uri** — a public key identifier.

```
xgen://pubkey/<algorithm>:<base64url-encoded-public-key>
```

Examples:
```
xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5AAAAI...   ← Ed25519 public key (default)
xgen://pubkey/ed448:AAAAC3NzaC1lZDQ0OAAAAIn...       ← Ed448 (algorithm-agile)
```

Public key URIs serve as the root identifier for Identities and Nodes. The Identity ID and Node ID are both derived from the pubkey_uri of the entity's keypair. Algorithm-agility is preserved: a future key algorithm requires only a new prefix.

**URI validation rules**

All three URI types MUST conform to their grammar above. A receiving Node MUST reject any message containing a malformed URI in a field typed as `xgen_uri`, `hash_uri`, or `pubkey_uri`. URIs are case-sensitive. The algorithm segment in `hash_uri` and `pubkey_uri` MUST be a registered algorithm name (see Algorithm Registry, Phase 2). For Phase 1, the only valid algorithm names are `sha256` for hash URIs and `ed25519` for pubkey URIs.

---

#### 3.1.7 Datetime Format

All datetime values in XGen protocol messages use RFC 3339 UTC format with millisecond precision and a mandatory `Z` suffix.

```
"2026-04-25T12:32:00.000Z"
```

The format is fixed: full date, `T` separator, hours, minutes, seconds, three-digit milliseconds, `Z` suffix. No other datetime representation is valid in a protocol message. Timezone offsets (e.g. `+01:00`) are not permitted — all times are UTC. Date-only values are not permitted. Unix timestamps (integer seconds or milliseconds) are not permitted.

A receiving Node MUST reject any message containing a datetime value that does not conform exactly to this format.

Millisecond precision is mandatory even when the millisecond component is zero — `"2026-04-25T12:32:00Z"` is not valid; `"2026-04-25T12:32:00.000Z"` is.

The rationale for this strictness is determinism in the signed Event log. A canonicalisation step that has to normalise datetime formats introduces ambiguity. One format, enforced at the wire level, eliminates the problem entirely.

---

#### 3.1.8 Integer Precision and Numeric Types

XGen protocol messages use integers for all numeric values. Floating-point numbers MUST NOT appear in protocol messages. There are no counters, weights, scores, or ratios in the XGen wire format that require fractional precision — if a future field appears to need a float, the correct solution is to use an integer with an implicit scale factor (e.g. a value in milliunits rather than fractional units).

All integers MUST be within the safe integer range for IEEE 754 double-precision floating point: −9,007,199,254,740,991 to +9,007,199,254,740,991 (2⁵³ − 1). This constraint ensures that integers in JSON protocol messages can be parsed correctly by any compliant JSON implementation, including those in JavaScript environments where all numbers are represented as doubles.

A receiving Node MUST reject any message containing a floating-point number or an integer outside the safe range.

---

#### 3.1.9 Binary Data Encoding

All binary data in XGen protocol messages is encoded as base64url (RFC 4648 §5) without padding characters. Base64url uses a URL-safe alphabet (`A–Z`, `a–z`, `0–9`, `-`, `_`) and omits the trailing `=` padding that standard base64 requires.

Base64url encoding is used exclusively for cryptographic material:

- Ed25519 public keys (~43 characters encoded)
- Ed25519 signatures (86 characters encoded)
- Content hashes embedded in URIs (43 characters for SHA-256)
- Any other fixed-length cryptographic byte sequences

Base64url MUST NOT be used for file content, images, audio, or any variable-length binary payload. Such content belongs on a media server and is referenced by URI in the Event payload.

A receiving Node MUST reject any message containing standard base64 (with `+`, `/`, or `=` characters) in a field typed as base64url.

---

#### 3.1.10 Protocol Versioning

Every XGen protocol message carries a `protocol_version` field at the top level of the envelope. The version is a string in the form `"major.minor"` — for example `"0.1"`.

```json
{
  "protocol_version": "0.1",   ← required in every message envelope
  "type": "message.text",
  ...
}
```

Versioning rules for receiving Nodes:

A Node MUST reject any message whose `major` version it does not recognise. Major version changes indicate breaking wire format changes — messages from an incompatible major version cannot be safely processed.

A Node MUST accept and process any message whose `major` version matches its own, regardless of the `minor` version. Minor version differences indicate additive changes — new optional fields, new event types, new capability declarations. The forward-compatibility rule (3.1.3) ensures that unknown fields are ignored silently.

A Node MAY log a warning when processing a message with a higher `minor` version than its own, but MUST NOT reject the message on that basis.

Version negotiation between Nodes during the federation handshake (3.4) establishes which protocol version the session operates under. The `protocol_version` field in individual messages reflects the version under which that message was constructed, which MUST match the negotiated session version.

---

### 3.2 Event Specification

*Status: wip*

The complete Event model — the atomic unit of the XGen protocol. Every action in XGen, whether a message, a membership change, a permission update, or a state transition, is expressed as a signed, content-addressed Event. Events are immutable once created. They are stored permanently in an append-only log on every Node that participates in the Space where they were produced.

---

#### 3.2.1 Event Envelope Schema

Every XGen Event is a JSON object with the following structure. Fields are listed in canonical order — the order in which they appear in the canonical form used for signature computation (3.2.4).

```json
{
  "protocol_version": "0.1",
  "type": "message.text",
  "event_id": "xgen://hash/sha256:a3f9b2c1d4e8f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6",
  "sender": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIHvoNgEMoFYGNhWMTRSXqFGrjWYRBhKVNBnPXVwB",
  "room_id": "xgen://hash/sha256:b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3",
  "space_id": "xgen://hash/sha256:c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4",
  "prev_events": [
    "xgen://hash/sha256:d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5"
  ],
  "timestamp": "2026-04-26T10:00:00.000Z",
  "content": {
    "text": "Hello"
  },
  "meta_atts": {
    "xgen.client": "xgen-cli/0.1"
  },
  "signature": "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIHvoNgEMoFYGNhWMTRSXqFGrjWYRBhKVNBnPXVwB:base64urlsignaturebytes"
}
```

**Field definitions**

| Field | Type | Required | Description |
|---|---|---|---|
| `protocol_version` | string | yes | Protocol version — `"major.minor"` format (3.1.10) |
| `type` | string | yes | EventType identifier (3.2.2) |
| `event_id` | hash_uri | yes | Content-addressed Event ID, derived from this Event's canonical form (3.2.3) |
| `sender` | pubkey_uri | yes | Public key URI of the Identity that created and signed this Event |
| `room_id` | hash_uri | yes | ID of the Room this Event belongs to |
| `space_id` | hash_uri | yes | ID of the Space containing the Room — redundant with room_id but present for routing without Room lookup |
| `prev_events` | array of hash_uri | yes | IDs of the Events this Event causally follows — at least one required except for Room creation Events (3.2.5) |
| `timestamp` | datetime | yes | RFC 3339 UTC datetime with millisecond precision — when the sender created this Event |
| `content` | object | yes | EventType-specific payload — schema defined per EventType in 3.2.2 |
| `meta_atts` | object | no | Extensible key-value map for application-level metadata — keys in `xgen.*` namespace are reserved |
| `signature` | string | yes | Cryptographic signature over the canonical form of this Event (3.2.4) |

**Field order note**

JSON objects are unordered by specification. The canonical order defined in 3.2.4 is used only for signature computation — it is not enforced on the wire. A receiving Node MUST sort fields into canonical order before computing or verifying a signature, regardless of the order in which fields arrived.

---

#### 3.2.2 EventType Registry

The `type` field identifies the EventType of an Event. EventType determines the schema of the `content` object and the processing rules the receiving Node applies.

**Naming convention**

EventType identifiers use dot-separated namespaced strings in the form `<category>.<action>`. All Phase 1 EventTypes use the bare namespace (no prefix). Third-party and extension EventTypes MUST use a reverse-domain prefix to avoid collisions — for example `com.example.custom_event`.

**Phase 1 EventType registry**

*Message events* — carry user-visible content:

| EventType | Description |
|---|---|
| `message.text` | Plain text message |
| `message.image` | Image reference (URI + metadata — no inline binary) |
| `message.file` | File reference (URI + metadata) |
| `message.reaction` | Reaction to a specific Event (emoji or short string) |
| `message.redact` | Redaction request for a prior Event — content replaced, Event ID preserved |

*State events* — define current Room or Space state. Multiple state events of the same type resolve to the most recent valid one. State Resolution algorithm is Phase 2:

| EventType | Description |
|---|---|
| `state.room_create` | Room creation — first Event in a Room's DAG, no `prev_events` |
| `state.room_name` | Sets or updates the Room display name |
| `state.room_topic` | Sets or updates the Room topic |
| `state.room_avatar` | Sets the Room avatar (URI reference) |

*Membership events* — record Identity membership transitions in a Room:

| EventType | Description |
|---|---|
| `membership.join` | Identity has joined the Room |
| `membership.leave` | Identity has left the Room voluntarily |
| `membership.invite` | Identity has been invited to the Room |
| `membership.kick` | Identity has been removed from the Room by an admin |
| `membership.ban` | Identity has been banned from the Room |

*System events* — protocol-level bookkeeping:

| EventType | Description |
|---|---|
| `system.key_rotation` | Sender is declaring a new signing keypair |

**Unknown EventType handling**

A Node receiving an Event with an unrecognised `type` value MUST store the Event in the log and propagate it to peers — it MUST NOT reject or drop it. The Node treats the `content` object as opaque data it cannot interpret. This is the forward-compatibility rule for EventTypes: new EventTypes added in later protocol versions are preserved by older Nodes even if they cannot process them. A client connected to the Node may be able to interpret the EventType even if the Node cannot.

---

#### 3.2.3 Event ID Derivation

The `event_id` is a content-addressed identifier — it is derived deterministically from the Event's own content. This means the Event ID is a cryptographic commitment to the Event's content: any modification to any field changes the ID, making the Event a different Event. Two Events with identical content always produce the same ID.

**Derivation process**

1. Construct the canonical form of the Event (3.2.4) — the same canonical form used for signature computation, but with the `event_id` and `signature` fields excluded.
2. Encode the canonical form as UTF-8 bytes.
3. Compute the SHA-256 hash of those bytes.
4. Encode the hash as a lowercase hex string.
5. Construct the hash URI: `xgen://hash/sha256:<hexstring>`

This value is the `event_id`. The sender computes it and includes it in the Event before signing. The receiving Node independently recomputes the `event_id` from the received Event content and MUST reject the Event if the computed ID does not match the declared `event_id`.

**Algorithm agility**

The hash algorithm is declared as part of the URI (`sha256` in Phase 1). Future protocol versions may introduce new hash algorithms by registering a new algorithm name. Nodes MUST NOT assume SHA-256 — they MUST read the algorithm from the `event_id` URI and apply the corresponding algorithm. For Phase 1, only `sha256` is a valid algorithm in Event IDs.

---

#### 3.2.4 Signature Canonicalisation

Signatures in XGen are computed over a canonical form of the Event — a deterministic serialisation that produces the same byte sequence regardless of wire format, field order, or whitespace. This is necessary because JSON does not guarantee field ordering, and two valid JSON serialisations of the same object may differ in byte content while being semantically identical.

**Canonical form rules**

1. **Fields included:** all fields in the Event envelope EXCEPT `event_id` and `signature`. The `event_id` is excluded because it is derived from the canonical form. The `signature` is excluded because it is the result of signing the canonical form — including it would be circular.
2. **Field order:** fields appear in the following fixed order: `protocol_version`, `type`, `sender`, `room_id`, `space_id`, `prev_events`, `timestamp`, `content`, `meta_atts` (if present).
3. **No whitespace:** the canonical form contains no spaces, newlines, or indentation outside of string values.
4. **No trailing commas.**
5. **Object keys sorted:** within `content` and `meta_atts`, all keys are sorted lexicographically (Unicode code point order). Nested objects follow the same rule recursively.
6. **String encoding:** all strings are UTF-8. Unicode escape sequences (e.g. `\u0041`) MUST be normalised to their literal UTF-8 representation.
7. **Array order preserved:** `prev_events` array entries appear in the order the sender included them. Receivers MUST NOT reorder `prev_events` before signature verification.

**Example canonical form**

Given the Event envelope from 3.2.1, the canonical form used for signing is:

```json
{"protocol_version":"0.1","type":"message.text","sender":"xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIHvoNgEMoFYGNhWMTRSXqFGrjWYRBhKVNBnPXVwB","room_id":"xgen://hash/sha256:b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3","space_id":"xgen://hash/sha256:c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4","prev_events":["xgen://hash/sha256:d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5"],"timestamp":"2026-04-26T10:00:00.000Z","content":{"text":"Hello"},"meta_atts":{"xgen.client":"xgen-cli/0.1"}}
```

This string is encoded as UTF-8 bytes, then signed with the sender's Ed25519 private key. The resulting signature bytes are base64url-encoded and included in the `signature` field.

**Signature field format**

```
"signature": "<algorithm>:<keyid>:<base64url-signature>"
```

| Component | Content |
|---|---|
| `algorithm` | Signing algorithm — `ed25519` in Phase 1 |
| `keyid` | base64url-encoded public key — matches the `sender` pubkey_uri key component |
| `base64url-signature` | base64url-encoded signature bytes — 86 characters for Ed25519 |

Example:
```
"signature": "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIHvoNgEMoFYGNhWMTRSXqFGrjWYRBhKVNBnPXVwB:U29tZVNpZ25hdHVyZUJ5dGVzSGVyZUluQmFzZTY0dXJsRW5jb2RpbmdXaXRob3V0UGFkZGluZw"
```

---

#### 3.2.5 The prev_events DAG

The `prev_events` field is an array of hash URIs identifying the Events that this Event causally follows. Together, the `prev_events` references of all Events in a Room form a Directed Acyclic Graph (DAG) — the complete causal history of the Room.

**Why a DAG and not a chain**

In a federated system, two Nodes may produce Events simultaneously without knowing about each other. If each Event could reference only one predecessor, simultaneous Events would collide at the same position in the sequence — one would have to be discarded, causing data loss. A DAG accommodates genuine concurrency: both Events reference the same predecessor, creating a fork. A later Event that references both is the merge point. No data is lost.

```
Initial state — single chain:
  Event 1 ← Event 2 ← Event 3

Fork — Node A and Node B both produce an Event after Event 3:
  Event 1 ← Event 2 ← Event 3 ← Event 4a  (from Node A)
                               ↖ Event 4b  (from Node B)

Merge — a later Event references both forks:
  Event 1 ← Event 2 ← Event 3 ← Event 4a ←─┐
                               ↖ Event 4b ←─┤
                                              Event 5  (prev_events: [4a, 4b])
```

Phase 1 implementations will almost always produce a single `prev_events` entry — the most recent Event in the Room — because two-Node testing with low message volume rarely produces genuine concurrency. The array structure is correct from day one so that Phase 2 federation does not require a wire format change.

**Rules for prev_events**

- `prev_events` MUST be an array. It MUST contain at least one entry in all Events except `state.room_create`.
- `state.room_create` is the only EventType where `prev_events` MUST be an empty array `[]` — it is the root of the DAG.
- All entries in `prev_events` MUST be valid hash URIs referencing Events that exist in the Room's Event log.
- A Node MUST NOT accept an Event whose `prev_events` references an Event ID it has not yet seen. It MUST hold the Event in a pending buffer and request the missing Events from its peers before processing.
- An Event MUST NOT reference itself or any of its own descendants in `prev_events` — this would create a cycle, which is invalid in a DAG.
- The maximum number of entries in `prev_events` for Phase 1 is 10. This bounds the merge complexity a Node must handle. Phase 2 may revise this limit.

**DAG tips — the current frontier**

The DAG tips are the Events that have no successors yet — no other Event references them in its `prev_events`. When a Node produces a new Event, it MUST reference all current tips in `prev_events`. This is the merge mechanism: producing a new Event always collapses the current frontier into a single new tip. A Room with healthy federation will have one tip most of the time. Multiple tips indicate concurrent Events that have not yet been merged.

---

#### 3.2.6 Event Validation Pipeline

A Node receiving an Event MUST apply the following validation checks in the order listed. The first failing check causes the Event to be rejected. Rejected Events are not stored and not propagated.

| Step | Check | Action on failure |
|---|---|---|
| 1 | Envelope size ≤ Tier ceiling and Space `max_event_size` | Reject — size violation (3.1.1) |
| 2 | Payload is valid JSON (or negotiated format) | Reject — parse failure (3.1.2) |
| 3 | All required fields present | Reject — missing required field (3.1.4) |
| 4 | No `null` values anywhere in the envelope | Reject — null prohibition (3.1.5) |
| 5 | All URI fields conform to their declared URI type grammar | Reject — malformed URI (3.1.6) |
| 6 | `timestamp` conforms to RFC 3339 UTC millisecond format | Reject — malformed datetime (3.1.7) |
| 7 | `protocol_version` major component matches this Node's major version | Reject — version mismatch (3.1.10) |
| 8 | `event_id` matches the independently computed content hash | Reject — Event ID mismatch (3.2.3) |
| 9 | All `prev_events` entries are known to this Node | Hold pending — request missing Events from peers |
| 10 | `prev_events` contains no cycles | Reject — DAG cycle violation (3.2.5) |
| 11 | `sender` pubkey_uri is a valid registered Identity in this Space | Reject — unknown sender |
| 12 | Signature verifies against the canonical form using the sender's public key | Reject — signature failure (3.2.4) |
| 13 | `sender` has permission to produce this EventType in this Room | Reject — authorisation failure |

**Notes on the pipeline**

Steps 1–7 are pure structural validation — they require no cryptographic operations and no external lookups. They are cheap and MUST be applied first to avoid wasting resources on malformed or oversized Events.

Step 8 (Event ID verification) requires computing a SHA-256 hash — inexpensive but a cryptographic operation.

Step 9 (predecessor check) may result in a hold rather than a rejection. The Node buffers the Event and requests the missing predecessors from its peers. If the predecessors are not received within a timeout window, the Event is discarded.

Step 12 (signature verification) is the most expensive step and is deliberately placed after all structural checks. A Node MUST NOT verify a signature on an Event that has already failed an earlier check.

Step 13 (authorisation) requires consulting the current Room state — which is itself derived from the Event log. This is why authorisation comes last: the Node must have a valid, verified Event before it can consult the state the Event is operating against.

---

#### 3.2.7 Conflict Resolution — Forward Reference to 3.9

*This section declares the conflict resolution framework as an interface. The full algorithm is specified in 3.9 State Resolution (Phase 2). Phase 1 implementations MUST implement the interface declared here so that Phase 2 can be added without wire format changes.*

**What conflict resolution addresses**

The DAG (3.2.5) guarantees that no Event is lost when two Nodes produce Events concurrently — both are preserved, the fork is recorded honestly. What the DAG does not resolve is *which Event wins* when two concurrent Events contradict each other. A room name cannot simultaneously be two different values. A banned Identity cannot simultaneously be a member. Conflict resolution determines the single authoritative answer from a set of concurrent competing Events.

For simple message Events (`message.text`, `message.image`, etc.) there is no meaningful conflict — concurrent messages are both displayed. Conflict resolution applies only to **state Events** and **membership Events** where two values are mutually exclusive.

**The seven-layer priority stack**

XGen resolves conflicts by applying the following priority layers in order. The first layer that produces a clear winner terminates the resolution. Lower layers are only reached when all higher layers are tied or inapplicable.

```
Layer 1 — EventType logic
  Some conflict pairs are resolved by type alone, regardless of
  who sent them or when. Hardcoded in the spec.
  Example: membership.ban always beats membership.join.

Layer 2 — Auth Tier of the producing Node
  Higher Auth Tier wins same-type conflicts.
  Hardcoded in the spec — Tier ordering is fixed.
  Example: Tier 4 state.room_name beats Tier 1 state.room_name.

Layer 3 — Home Node assertion
  The Identity's home Node is the authoritative source of truth
  for that Identity's current state and key material. In authority
  conflicts, the home Node's signed assertion wins.
  Architectural — follows from XGen's identity-first model.

Layer 4 — Role within Space
  Higher role wins same-Tier, same-type conflicts.
  Default role priority defined per Tier in spec.
  Space owner may override role priority at Space creation within
  Tier-permitted bounds.
  Example: Room Admin state.room_name beats Member state.room_name.

Layer 5a — Manual Node ordering  (user-defined, highest sublayer)
  Space owner explicitly orders federated Nodes by priority using
  a drag-and-drop interface. Stored as a signed state.node_priority
  Event in the Space DAG. Beats all automatic sublayers below.
  New Nodes joining after this Event is set are appended at the
  bottom by default until manually repositioned.

Layer 5b — Federation recency  (automatic default)
  When no manual ordering is set, the Node that joined the
  federation most recently has higher priority. Recency is
  determined by the timestamp of the Node's first accepted
  federation Event in the Space DAG.
  Rationale: recently joined Nodes are more likely to have been
  vetted under current policies and Trust Assertions.

Layer 5c — Lexicographic event_id  (absolute backstop)
  When all above layers are tied, the Event whose event_id sorts
  lower in lexicographic (Unicode code point) order wins.
  This is purely mechanical, requires no communication between
  Nodes, and produces the same winner on every Node independently.
  It cannot be gamed — the event_id is a content hash.
```

**Key properties of the stack**

Every layer above Layer 5c involves a human decision or an institutional fact. Layers 1–3 are hardcoded by the spec and reflect logical or architectural truths. Layer 4 reflects verified role assignments. Layer 5a reflects explicit Space owner intent. Layer 5b reflects recency of verified federation relationships. Only Layer 5c is purely mechanical — and it is only reached when every meaningful distinction has been exhausted.

Every Node independently applies the same stack to the same DAG and reaches the same winner without communication. This is the determinism guarantee: conflict resolution is a pure function of the Event log.

**The `state.node_priority` Event (Layer 5a)**

The Space owner declares manual Node ordering by producing a `state.node_priority` Event:

```json
{
  "type": "state.node_priority",
  "content": {
    "ordered_nodes": [
      "xgen://node/ed25519:AAA...",
      "xgen://node/ed25519:BBB...",
      "xgen://node/ed25519:CCC..."
    ]
  }
}
```

The `ordered_nodes` array is ordered from highest priority (index 0) to lowest. Nodes not listed fall back to Layer 5b ordering. Only the Space owner (or a delegated admin with explicit permission) may produce this EventType. A later `state.node_priority` Event supersedes the previous one — the most recent valid Event in the DAG is authoritative.

**Conflict categories**

Four distinct conflict categories exist, each handled slightly differently by the stack:

| Category | Description | Primary resolution layer |
|---|---|---|
| State conflict | Same state key, two concurrent values (e.g. two room names) | Layer 4 — role priority |
| Permission conflict | Two Events with opposing effects on same Identity (e.g. ban vs invite) | Layer 1 — EventType logic |
| Authority conflict | Sender's permission was being modified simultaneously with their action | Layer 3 — home Node assertion |
| Ordering conflict | Causal ambiguity affecting meaning of subsequent Events | Layer 5 — full sublayer sequence |

*Full conflict resolution algorithm including edge cases, timeout handling, and split-brain recovery is specified in 3.9 State Resolution (Phase 2).*

---

### 3.3 Transport Protocol

*Status: wip*

The network transport layer between clients and Nodes, and between Nodes. Two distinct connection types exist — client→Node and Node→Node — each with different trust assumptions. Both use the same underlying transport and framing mechanism.

---

#### 3.3.1 Transport Layer

XGen uses WebSocket (RFC 6455) as the mandatory transport for all connections. WebSocket was chosen for three reasons: it is bidirectional and long-lived, eliminating the overhead of repeated connection establishment; it operates over standard HTTP/HTTPS infrastructure, making it compatible with firewalls, proxies, and load balancers; and it is universally supported across all target implementation languages and environments.

Every Node advertises its WebSocket endpoint URI in its Node Identity record (3.5). There is no hardcoded port. A Node may operate on any port and MUST declare its full endpoint URI including scheme, host, port, and path. Example:

```
"endpoint": "wss://node.example.org:8443/xgen"
```

For Local Node mode, the endpoint is always localhost:

```
"endpoint": "ws://127.0.0.1:8080/xgen"
```

**TLS requirements**

All production connections MUST use TLS — the `wss://` scheme. Unencrypted WebSocket connections (`ws://`) are only permitted in Local Node mode where no external network interfaces are active. A Node operating in production mode MUST reject unencrypted incoming connections. A client or peer Node attempting to connect via `ws://` to a production Node MUST be refused at the transport level before any protocol exchange.

TLS certificate validation follows standard WebSocket/HTTPS rules. Self-signed certificates are only permitted in Local Node mode.

---

#### 3.3.2 Connection Types

XGen has two distinct connection types. They use the same transport and framing but have different authentication requirements and trust levels.

**Client → Node connection**

A user's client application connects to its home Node. This is the primary connection through which the user sends and receives Events. A client maintains one persistent connection to its home Node. All Spaces and Rooms the user participates in are served over this single connection — Events are routed by their `space_id` and `room_id` fields, not by separate connections per Space or Room.

The client is authenticated by its keypair during the connection handshake (3.3.4). The Node verifies the client's identity against the registered Identity record before allowing any Event exchange.

**Node → Node connection**

Two Nodes establish a federation connection to exchange Events for Spaces they share. Node→Node connections are mutually authenticated — both sides prove their identity before any Events are exchanged. The federation relationship itself is established separately via the Federation Handshake protocol (3.4). The transport connection carries the ongoing Event exchange once federation is established.

A Node maintains one persistent connection per federated peer Node. All shared Spaces are multiplexed over that single connection.

---

#### 3.3.3 Message Framing

All messages exchanged over a WebSocket connection use the transport framing defined in 3.1.2. Each WebSocket message carries exactly one transport frame. WebSocket fragmentation MUST NOT be used — a single XGen protocol message fits in a single WebSocket message.

```
0x04                     ; format identifier length: 4 bytes
'json'                   ; format identifier string
0x00 0x00 0x00 0xc8      ; payload length: 200 bytes
'{ ... }'                ; serialised message payload
```

A receiver MUST validate the frame structure before passing the payload to the deserialiser. A malformed frame — one where the declared payload length does not match the actual WebSocket message length — MUST result in immediate connection termination without a graceful close.

---

#### 3.3.4 Connection Lifecycle

Every connection passes through four phases in sequence. A connection that does not advance through all phases in order MUST be terminated.

```
  ┌──────────┐     TCP+TLS      ┌──────────┐
  │  Client  │ ─────────────── │   Node   │
  └──────────┘                 └──────────┘

  Phase 1 — CONNECT
    Client opens WebSocket connection to Node endpoint.
    Node accepts the connection — no Events exchanged yet.

  Phase 2 — AUTHENTICATE
    Node sends: transport.challenge
    Client sends: transport.auth  (signed challenge response)
    Node verifies signature against registered public key.
    Node sends: transport.auth_ok  OR  transport.auth_fail
    On auth_fail: connection closed immediately.

  Phase 3 — ACTIVE
    Full bidirectional Event exchange.
    Keepalive ping/pong running.
    Rate limiting signals may be sent by Node.

  Phase 4 — CLOSE
    Either side sends: transport.goodbye
    Other side acknowledges and closes WebSocket.
    OR: connection drops without goodbye (treated as ungraceful disconnect).
```

**Phase 2 — Authentication messages**

The challenge-response mechanism uses the Identity keypair directly. The Node issues a random nonce. The client signs it with their private key. The Node verifies the signature against the public key registered for that Identity. No session tokens, no server-side session state.

`transport.challenge` — sent by Node immediately after WebSocket connection is established:

```json
{
  "protocol_version": "0.1",
  "type": "transport.challenge",
  "nonce": "base64url-encoded-32-random-bytes",
  "timestamp": "2026-04-26T10:00:00.000Z"
}
```

`transport.auth` — sent by client in response:

```json
{
  "protocol_version": "0.1",
  "type": "transport.auth",
  "identity_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
  "nonce": "base64url-encoded-32-random-bytes",
  "signature": "ed25519:AAAAC3Nz...:base64url-signature-over-nonce"
}
```

The `signature` field covers the nonce bytes only — not the full `transport.auth` envelope. This keeps the signed input minimal and unambiguous.

The `nonce` in `transport.auth` MUST match the nonce from `transport.challenge`. A Node MUST reject any `transport.auth` where the nonce does not match, the timestamp on the challenge is older than 30 seconds, or the signature does not verify against the declared `identity_id` public key.

`transport.auth_ok` — sent by Node on successful authentication:

```json
{
  "protocol_version": "0.1",
  "type": "transport.auth_ok",
  "identity_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
  "timestamp": "2026-04-26T10:00:00.000Z"
}
```

`transport.auth_fail` — sent by Node on failed authentication, followed immediately by connection close:

```json
{
  "protocol_version": "0.1",
  "type": "transport.auth_fail",
  "error_code": 1001,
  "error_string": "auth_signature_invalid",
  "timestamp": "2026-04-26T10:00:00.000Z"
}
```

**Node → Node authentication**

For Node→Node connections, the same challenge-response mechanism applies but is run in both directions. Each Node issues a challenge to the other and verifies the response before the connection enters Phase 3. Both Nodes MUST successfully authenticate before any Events are exchanged.

---

#### 3.3.5 Keepalive

WebSocket provides a built-in ping/pong mechanism (RFC 6455 §5.5.2). XGen uses it for connection keepalive.

A Node MUST send a WebSocket ping frame to each connected client and peer Node every **30 seconds** during Phase 3. A client or peer Node that does not respond with a pong within **10 seconds** of receiving a ping is considered disconnected. The Node MUST close the connection and treat it as an ungraceful disconnect.

Clients and peer Nodes MUST respond to WebSocket ping frames with a pong frame. A client or peer Node MAY also send its own ping frames; the Node MUST respond with pong.

The keepalive interval (30 seconds) and timeout (10 seconds) are Phase 1 work definitions and may be revised based on implementation experience.

---

#### 3.3.6 Reconnection Behaviour

When a connection is lost — either ungracefully or after a `transport.goodbye` — the disconnected party MUST wait before reconnecting. Immediate reconnection attempts create thundering herd problems when a Node restarts or a network partition heals.

**Reconnection algorithm — exponential backoff with jitter**

```
Attempt 1:  wait  1s  ± 0.5s random jitter
Attempt 2:  wait  2s  ± 1s   random jitter
Attempt 3:  wait  4s  ± 2s   random jitter
Attempt 4:  wait  8s  ± 4s   random jitter
Attempt 5:  wait 16s  ± 8s   random jitter
Attempt 6+: wait 30s  ± 15s  random jitter  (ceiling)
```

The ceiling is 30 seconds base wait. A client MUST NOT attempt reconnection more frequently than once per 15 seconds after the ceiling is reached. A Node that receives connection attempts more frequently than once per 15 seconds from the same Identity MAY apply rate limiting (3.3.7).

**State recovery on reconnect**

After reconnecting and re-authenticating, a client MUST request any Events it may have missed during the disconnection. The mechanism for requesting missed Events is defined in 3.3 — a client sends a `transport.sync_request` carrying the `event_id` of the last Event it received. The Node responds with any Events that follow that ID in the DAG.

```json
{
  "protocol_version": "0.1",
  "type": "transport.sync_request",
  "last_event_id": "xgen://hash/sha256:a3f9b2c1...",
  "room_id": "xgen://hash/sha256:b2c3d4e5..."
}
```

If the client has no prior Events for a Room (first join or fresh install), it omits `last_event_id` and the Node sends the full Room history up to the current DAG tip, subject to any history limits declared by the Space.

---

#### 3.3.7 Rate Limiting

A Node MAY rate limit any connection — client or peer Node — that is sending Events or requests at a rate that exceeds the Node's capacity or the Space's declared limits.

When rate limiting is applied, the Node sends a `transport.rate_limit` message before continuing to process or before dropping the connection:

```json
{
  "protocol_version": "0.1",
  "type": "transport.rate_limit",
  "retry_after_ms": 5000,
  "reason": "event_rate_exceeded"
}
```

The `retry_after_ms` field declares how many milliseconds the sender MUST wait before sending further Events or requests. A sender that ignores a `transport.rate_limit` signal and continues sending MUST be disconnected by the Node without further warning. Repeated rate limit violations from the same Identity MAY be reported to the Node's defederation subsystem (Phase 2).

---

#### 3.3.8 Transport Error Codes

Transport-level errors use a defined set of string error codes. These are distinct from application-level errors (Event rejection reasons defined in 3.2.6). Transport errors appear in `transport.auth_fail` and `transport.error` messages.

| Code | Error string | Meaning |
|---|---|---|
| 1001 | `auth_signature_invalid` | Challenge-response signature did not verify |
| 1002 | `auth_identity_unknown` | The `identity_id` is not registered on this Node |
| 1003 | `auth_nonce_expired` | The challenge nonce is older than 30 seconds |
| 1004 | `auth_nonce_mismatch` | The nonce in `transport.auth` does not match the issued challenge |
| 1005 | `version_incompatible` | Major protocol version mismatch |
| 1006 | `format_unknown` | Unrecognised format identifier in transport frame |
| 1007 | `frame_malformed` | Transport frame structure is invalid |
| 1008 | `rate_limit_exceeded` | Sender ignored rate limit signal |
| 1009 | `connection_limit` | Node has reached its maximum connection count |
| 1010 | `tls_required` | Node requires TLS — unencrypted connection refused |

Numeric codes are in the 1000 range, reserving lower ranges for future transport sublayers. Both the numeric code and the string name MUST be included in every `transport.auth_fail` and `transport.error` message.

**Display and internal usage rules**

Internally — in logs, metrics, monitoring systems, and inter-process communication — implementations SHOULD use the numeric code only. Integer comparison is fast, unambiguous, and language-agnostic.

When an error is displayed to a human — in a client UI, an admin dashboard, a log viewer, or any surface a person reads — the implementation MUST render a message in the following form:

```
Error <code> (<error_string>): <short description>. <optional extended explanation>
```

Example:

```
Error 1001 (auth_signature_invalid): Challenge-response signature did not verify.
Your identity key may have changed or the connection timed out. Please reconnect and try again.
```

This format serves three audiences simultaneously. The numeric code gives technical staff an immediate reference for logs and support tickets. The error string gives developers and advanced users the machine-readable name without a lookup. The plain-language description gives anyone — including people unfamiliar with protocol internals — enough information to understand what happened and what to do next.

The short description MUST correspond to the Meaning column in the error code table above. The optional extended explanation is implementation-defined and may be contextual — for instance, including the Node address or the timestamp of the failed attempt. Extended explanations SHOULD be localised.

The wire format carries both fields always. Display rendering is the responsibility of the receiving implementation.

A `transport.error` message carrying one of these codes MAY be sent by the Node before closing the connection, giving the client or peer Node a reason for the closure:

```json
{
  "protocol_version": "0.1",
  "type": "transport.error",
  "error_code": 1007,
  "error_string": "frame_malformed",
  "timestamp": "2026-04-26T10:00:00.000Z"
}
```

---

#### 3.3.9 Graceful Close

Either party MAY initiate a graceful close at any time during Phase 3 by sending a `transport.goodbye` message:

```json
{
  "protocol_version": "0.1",
  "type": "transport.goodbye",
  "reason": "node_shutdown",
  "timestamp": "2026-04-26T10:00:00.000Z"
}
```

Defined `reason` values: `node_shutdown`, `client_disconnect`, `idle_timeout`, `maintenance`. The receiving party MUST acknowledge by closing the WebSocket connection. A Node that receives `transport.goodbye` from a client MUST NOT count the disconnection as an ungraceful failure for reputation or rate limiting purposes.

---

### 3.4 Federation Handshake

*Status: wip*

The protocol for establishing a federation relationship between two Nodes. A federation relationship is distinct from a transport connection (3.3): the transport connection is the wire-level WebSocket session, established cheaply and frequently. The federation relationship is a persistent, recorded trust agreement between two Nodes that enables them to exchange Events for shared Spaces.

One federation handshake covers the entire Node-to-Node relationship. All Spaces shared between two Nodes are multiplexed over a single federation channel — there is no per-Space handshake. Individual Spaces are added to or removed from the federation channel via Space-level Events, not new handshakes.

---

#### 3.4.1 Purpose and Scope

The Federation Handshake serves three purposes. First, it establishes mutual agreement on protocol capabilities — what serialisation formats, compression options, and extension features both Nodes support. Second, it negotiates the protocol version for the session. Third, it records the federation relationship as a signed Event in each participating Space's DAG, creating an auditable and cryptographically verifiable history of federation decisions.

A federation relationship is initiated by either Node. The Node that sends `federation.hello` first is the **initiating Node**. The Node that receives it is the **receiving Node**. Both roles are symmetric after the handshake completes — there is no permanent initiator/receiver distinction once the session is active.

**Relationship to 3.3 Transport**

The federation handshake runs *inside* an already-authenticated transport connection. Before any federation message is exchanged, both Nodes MUST have completed the transport authentication sequence (3.3.4, Phase 2). The federation handshake is the first application-level exchange on a fully authenticated Node→Node connection.

---

#### 3.4.2 Handshake Message Schemas

Five message types are used in the federation handshake. All follow the standard Event envelope (3.2.1) with transport framing (3.1.2).

**`federation.hello`** — sent by the initiating Node to open the handshake:

```json
{
  "protocol_version": "0.1",
  "type": "federation.hello",
  "node_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
  "capabilities": {
    "serialisation": ["json", "msgpack"],
    "compression": [],
    "extensions": []
  },
  "shared_spaces": [
    "xgen://hash/sha256:a3f9b2c1...",
    "xgen://hash/sha256:b2c3d4e5..."
  ],
  "timestamp": "2026-04-26T10:00:00.000Z",
  "signature": "ed25519:AAAAC3Nz...:base64url-signature"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `node_id` | pubkey_uri | yes | The initiating Node's identity |
| `capabilities` | object | yes | What this Node supports — serialisation formats, compression, extensions |
| `shared_spaces` | array of hash_uri | yes | Space IDs this Node wants to federate for — may be empty array if proposing a new relationship with no current shared Spaces |
| `timestamp` | datetime | yes | When this message was created |
| `signature` | string | yes | Signature over the canonical form of this message |

**`federation.capabilities`** — sent by the receiving Node in response, declaring its own capabilities:

```json
{
  "protocol_version": "0.1",
  "type": "federation.capabilities",
  "node_id": "xgen://pubkey/ed25519:BBBBD3NzaC1lZDI1NTE5...",
  "capabilities": {
    "serialisation": ["json"],
    "compression": [],
    "extensions": []
  },
  "negotiated": {
    "serialisation": "json",
    "protocol_version": "0.1"
  },
  "timestamp": "2026-04-26T10:00:01.000Z",
  "signature": "ed25519:BBBBD3Nz...:base64url-signature"
}
```

The `negotiated` object declares the resolved capabilities for the session — the receiving Node picks the best common option from the intersection of both capability sets. The initiating Node MUST accept the negotiated values or reject with `federation.reject`.

**`federation.accept`** — sent by the initiating Node to confirm the negotiated capabilities and open the active federation session:

```json
{
  "protocol_version": "0.1",
  "type": "federation.accept",
  "node_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
  "session_id": "xgen://hash/sha256:c3d4e5f6...",
  "timestamp": "2026-04-26T10:00:02.000Z",
  "signature": "ed25519:AAAAC3Nz...:base64url-signature"
}
```

The `session_id` is a hash URI derived from the concatenation of both Node IDs and the timestamp — a unique identifier for this specific federation session. It is used to correlate federation Events recorded in the Space DAG.

**`federation.reject`** — sent by either Node to refuse the handshake, with a reason:

```json
{
  "protocol_version": "0.1",
  "type": "federation.reject",
  "node_id": "xgen://pubkey/ed25519:BBBBD3NzaC1lZDI1NTE5...",
  "error_code": 2001,
  "error_string": "no_common_capabilities",
  "timestamp": "2026-04-26T10:00:01.000Z",
  "signature": "ed25519:BBBBD3Nz...:base64url-signature"
}
```

After sending `federation.reject`, the Node MUST close the transport connection. The rejecting Node MUST NOT attempt to re-initiate federation with the same peer within 60 seconds.

**`federation.goodbye`** — sent by either Node to gracefully end an active federation relationship:

```json
{
  "protocol_version": "0.1",
  "type": "federation.goodbye",
  "node_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
  "reason": "node_shutdown",
  "session_id": "xgen://hash/sha256:c3d4e5f6...",
  "timestamp": "2026-04-26T10:15:00.000Z",
  "signature": "ed25519:AAAAC3Nz...:base64url-signature"
}
```

Defined `reason` values: `node_shutdown`, `policy_change`, `space_removed`, `maintenance`. After sending `federation.goodbye`, the Node MUST close the transport connection using the graceful close sequence (3.3.9).

---

#### 3.4.3 Handshake State Machine

The federation handshake progresses through the following states. Both Nodes maintain this state independently.

```
  Node A (initiating)                    Node B (receiving)
  ───────────────────                    ─────────────────
  [IDLE]                                 [IDLE]
     │                                      │
     │  ── federation.hello ──────────────► │
     │                                   [HELLO_RECEIVED]
     │                                      │
     │ ◄────────── federation.capabilities ─│
  [CAPS_RECEIVED]                           │
     │                                   [CAPS_SENT]
     │                                      │
     │  ── federation.accept ────────────► │
     │                                   [ACTIVE]
  [ACTIVE]
     │
     │  (bidirectional Event exchange begins)
     │
     │  ── federation.goodbye ───────────► │
     │                                   [CLOSED]
  [CLOSED]
```

**Timeout rules**

A Node in `HELLO_RECEIVED` or `CAPS_SENT` state MUST send its response within **10 seconds**. A Node waiting for a response MUST time out after **15 seconds** and treat the peer as non-responsive. On timeout, the Node MUST close the transport connection and MAY retry after the reconnection backoff defined in 3.3.6.

**Unexpected message handling**

A Node that receives a message of the wrong type for its current state (e.g. `federation.accept` before sending `federation.capabilities`) MUST send `federation.reject` with error code `2005` (`unexpected_message`) and close the connection.

---

#### 3.4.4 Capability Negotiation

Capabilities are declared as arrays of supported option strings in the `capabilities` object of `federation.hello` and `federation.capabilities`. The receiving Node computes the intersection of both capability sets and declares the selected options in the `negotiated` object.

**Serialisation format negotiation**

Both Nodes declare their supported serialisation formats as an ordered array of preference (most preferred first). The receiving Node selects the highest-preference format that both Nodes support. If only JSON is common, JSON is selected. If neither Node supports a common format beyond JSON, JSON MUST be selected — JSON is always available as the mandatory baseline (3.1.2).

```
Node A declares: ["json", "msgpack", "cbor"]
Node B declares: ["json", "cbor"]
Intersection:    ["json", "cbor"]
Selected:        "cbor"  (Node A's highest preference that B supports)
```

**Protocol version negotiation**

Both Nodes operate on the same `major` version — they verified this during transport authentication (3.3.4). The session uses the lower of the two `minor` versions. A Node running `0.3` and a Node running `0.1` negotiate to `0.1`. This ensures neither Node sends Events using features the other doesn't understand.

**Unknown capabilities**

A Node MUST silently ignore capability keys it does not recognise. Unknown capabilities are not grounds for rejection — this is the forward-compatibility rule applied to the capability system. A future capability declared by a newer Node is simply ignored by an older one.

**Mandatory capabilities**

For Phase 1, the only mandatory capability category is `serialisation`, which MUST always be present and MUST always include `json`. All other capability categories (`compression`, `extensions`) are optional and default to empty arrays in Phase 1.

---

#### 3.4.5 Relationship Persistence

Once federation is accepted, the relationship is recorded in the Event log of each shared Space. This creates an auditable, cryptographically verifiable history of when federation was established and which Nodes participated.

**Federation record Event**

Each participating Space receives a `state.federation_add` Event produced by the Space owner (or the Node acting on behalf of the Space):

```json
{
  "type": "state.federation_add",
  "content": {
    "node_id": "xgen://pubkey/ed25519:BBBBD3NzaC1lZDI1NTE5...",
    "session_id": "xgen://hash/sha256:c3d4e5f6...",
    "negotiated_version": "0.1",
    "negotiated_serialisation": "json"
  }
}
```

This Event is produced once per Space, not once per handshake. If two Nodes are already federated and reconnect after a disconnection, no new `state.federation_add` Event is produced — the existing record remains authoritative.

**Relationship storage on the Node**

Each Node maintains a local federation registry — a persistent record of all active federation relationships. Each entry contains the peer Node ID, the shared Space IDs, the negotiated session parameters, and the timestamp of the last successful connection. This registry is consulted on startup to re-establish federation connections without requiring a new handshake sequence.

**Relationship termination**

When a `federation.goodbye` is received, the Node produces a `state.federation_remove` Event in each affected Space's DAG:

```json
{
  "type": "state.federation_remove",
  "content": {
    "node_id": "xgen://pubkey/ed25519:BBBBD3NzaC1lZDI1NTE5...",
    "session_id": "xgen://hash/sha256:c3d4e5f6...",
    "reason": "node_shutdown"
  }
}
```

---

#### 3.4.6 Re-federation

When a previously established federation relationship needs to be re-established — after a long disconnection, a Node restart, or a capability upgrade — the handshake runs again in full. The existing `state.federation_add` record in the Space DAG remains; no new one is produced unless the Space owner explicitly authorises a new federation relationship with different parameters.

A Node that reconnects after an ungraceful disconnection (no `federation.goodbye`) MUST run the full handshake before resuming Event exchange. It MUST NOT assume the previous session parameters are still valid.

After re-federation, the reconnecting Node MUST request any Events it missed during the disconnection using `transport.sync_request` (3.3.6) for each shared Space and Room.

---

#### 3.4.7 Federation Handshake Error Codes

Federation handshake errors follow the same dual numeric+string format as transport errors (3.3.8). Error codes are in the 2000 range, distinct from transport error codes (1000 range).

| Code | Error string | Meaning |
|---|---|---|
| 2001 | `no_common_capabilities` | No common serialisation format or other mandatory capability |
| 2002 | `version_incompatible` | No common protocol minor version — major version mismatch already caught at transport level |
| 2003 | `space_not_found` | A declared `shared_spaces` entry is unknown to this Node |
| 2004 | `federation_policy_rejected` | This Node's federation policy does not permit federation with the requesting Node |
| 2005 | `unexpected_message` | Message received in wrong state |
| 2006 | `signature_invalid` | Handshake message signature did not verify |
| 2007 | `rate_limit` | Too many federation attempts from this Node — retry after cooldown |
| 2008 | `node_unknown` | The `node_id` in `federation.hello` is not registered on this network |

**Display rule** — same pattern as 3.3.8:

```
Error 2004 (federation_policy_rejected): This Node's federation policy does not
permit federation with the requesting Node. Contact the Space administrator.
```

---

### 3.5 Node Identity Protocol

*Status: wip*

How a Node establishes, announces, and proves its identity on the network. A Node's identity is derived directly from its keypair — no registration authority, no certificate chain, no external validation. The keypair IS the identity, consistent with XGen's identity-first model throughout.

---

#### 3.5.1 Node Keypair Generation

On first run, a Node generates an Ed25519 keypair. This keypair is the Node's permanent identity for Phase 1. The public key becomes the Node ID. The private key never leaves the Node — it is used only to sign Node announcements and to authenticate transport connections (3.3.4).

The private key MUST be stored encrypted at rest using a strong symmetric cipher. The encryption key MUST be derived from a secret known only to the Node operator — not hardcoded, not stored in the same location as the encrypted key. The specific encryption mechanism is implementation-defined; the spec requires only that the private key is not stored in plaintext.

On startup, the Node loads and decrypts its private key into memory. If the key cannot be decrypted — wrong passphrase, corrupted file, missing file — the Node MUST refuse to start and MUST produce a clear error message directing the operator to the key management documentation.

A Node MUST NOT generate a new keypair if one already exists. Keypair generation is a one-time operation. Accidental regeneration would change the Node ID, breaking all existing federation relationships and Trust Assertions.

---

#### 3.5.2 Node ID Derivation

The Node ID is the pubkey_uri of the Node's Ed25519 public key:

```
node_id = xgen://pubkey/ed25519:<base64url-encoded-public-key>
```

Example:
```
xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIHvoNgEMoFYGNhWMTRSXqFGrjWYRBhKVNBnPXVwB
```

This is identical in structure to an Identity ID (3.6). The distinction is in the context: a node_id appears in Node announcement fields and Node→Node protocol messages; an identity_id appears in user-facing protocol messages. Both are pubkey_uri values. Both are self-certifying — no external authority needed to validate either.

---

#### 3.5.3 Node Announcement Schema

The `node_announcement` is the Node's public declaration of its existence, endpoint, capabilities, and the Auth Tiers it serves. It is the primary record other Nodes and clients use to discover and verify a Node.

```json
{
  "protocol_version": "0.1",
  "type": "node_announcement",
  "node_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
  "endpoint": "wss://node.example.org:8443/xgen",
  "capabilities": {
    "serialisation": ["json", "msgpack"],
    "compression": [],
    "extensions": []
  },
  "auth_tiers_served": [1],
  "operator_display_name": "Example Community Node",
  "announcement_version": 1,
  "valid_until": "2026-07-26T00:00:00.000Z",
  "timestamp": "2026-04-26T10:00:00.000Z",
  "signature": "ed25519:AAAAC3NzaC1lZDI1NTE5...:base64url-signature"
}
```

**Field definitions**

| Field | Type | Required | Description |
|---|---|---|---|
| `node_id` | pubkey_uri | yes | The Node's permanent identity |
| `endpoint` | string | yes | Full WebSocket endpoint URI — scheme, host, port, path |
| `capabilities` | object | yes | Same structure as federation.hello (3.4.2) |
| `auth_tiers_served` | array of integer | yes | Which Auth Tiers this Node accepts Identities for — e.g. `[1]` for Tier 1 only |
| `operator_display_name` | string | no | Human-readable name for the Node — for display in client UIs |
| `announcement_version` | integer | yes | Monotonically increasing counter — higher version supersedes lower |
| `valid_until` | datetime | yes | When this announcement expires — receiving Nodes MUST discard expired announcements |
| `timestamp` | datetime | yes | When this announcement was created |
| `signature` | string | yes | Signature over the canonical form of this announcement |

**Canonical form for signing**

The canonical form excludes `signature` and follows the same rules as Event canonicalisation (3.2.4): no whitespace, keys sorted within objects, UTF-8 encoding. Field order: `protocol_version`, `type`, `node_id`, `endpoint`, `capabilities`, `auth_tiers_served`, `operator_display_name` (if present), `announcement_version`, `valid_until`, `timestamp`.

---

#### 3.5.4 Announcement Signing and Verification

The Node signs its announcement with its own Ed25519 private key. The signature field follows the same format as Event signatures (3.2.4):

```
"signature": "ed25519:<base64url-public-key>:<base64url-signature>"
```

Any receiver — peer Node, client, or Bootstrap Node — can verify the announcement independently by:

1. Extracting the public key from the `node_id` pubkey_uri.
2. Constructing the canonical form of the announcement (excluding `signature`).
3. Verifying the signature bytes against the canonical form using the extracted public key.

No third party is needed. The announcement is self-certifying. A receiver MUST reject any announcement whose signature does not verify, whose `node_id` does not match the key used in the signature, or whose `valid_until` is in the past.

---

#### 3.5.5 Announcement Propagation

Node announcements spread through the network by two mechanisms.

**Direct exchange on connection**

When a Node establishes a transport connection to a peer — either client→Node or Node→Node — it sends its current `node_announcement` immediately after the transport authentication phase (3.3.4). The peer stores the announcement locally. This ensures every connected peer always has a fresh announcement for the Nodes it talks to directly.

**Peer relay**

A Node MAY relay announcements it has received from peers to its own connected peers. This propagates Node discovery information through the network without requiring every Node to connect directly to every other Node. A Node MUST NOT relay an announcement whose signature it has not verified. A Node MUST NOT relay an announcement with an `announcement_version` lower than the highest version it has seen for that `node_id`.

**Bootstrap Node directory (Phase 2)**

In Phase 2, Bootstrap Nodes (3.14) maintain a queryable directory of current announcements. New Nodes and clients use Bootstrap Nodes to discover the network. For Phase 1 — two Nodes, Local Node mode — Bootstrap discovery is not needed. Nodes connect directly using configured endpoint URIs.

---

#### 3.5.6 Announcement Refresh

A Node MUST re-announce itself before its current announcement expires. The recommended refresh interval is 80% of the TTL — for a 90-day `valid_until`, re-announce after 72 days. This gives peers time to receive the refreshed announcement before the old one expires.

A Node MUST also re-announce immediately when any of the following change:

- Its endpoint URI (e.g. the Node moves to a new host or port)
- Its declared capabilities (e.g. a new serialisation format is enabled)
- Its `auth_tiers_served` list

On re-announcement, the Node increments `announcement_version` by 1. Receiving peers replace their stored announcement for this `node_id` only if the incoming `announcement_version` is strictly higher than the stored one. This prevents replay of old announcements.

**TTL recommendation**

For Phase 1: 90 days `valid_until`. This is a work definition — Phase 2 may standardise TTL values per Auth Tier.

---

#### 3.5.7 Keypair Permanence and Key Rotation Policy

**Phase 1 — permanent keypair**

In Phase 1, Node keypairs are permanent. A Node does not rotate its signing keypair. If a key is compromised, the correct response is to decommission the Node and create a new one with a new ID. Existing federation relationships and Trust Assertions referencing the old Node ID are invalidated — peers must be notified out-of-band and federation re-established with the new Node.

This is sufficient for Phase 1 because Phase 1 deployments are development and testing environments where key compromise is not a realistic threat and federation relationships are short-lived.

**Key rotation — Phase 2**

Key rotation, including the cryptographic continuity proof mechanism (dual-signature transition), is specified in Phase 2. The `system.key_rotation` EventType (3.2.2) is the placeholder for this mechanism.

**Key rotation optionality in high-trust environments**

Key rotation is NOT mandatory even after Phase 2 specifies it. A Node operator MAY choose to operate with a permanent keypair indefinitely, including in Tier 4 deployments. This is a legitimate and defensible operational stance. Some institutional security policies — particularly those built around Hardware Security Modules (HSMs) where the private key is generated in hardware and certified never to leave — actively prefer key permanence over rotation. A key that never rotates has no rotation window during which an attacker could intercept or tamper with the rotation process.

The spec does not impose key rotation as an obligation. It provides the mechanism for operators who require it. Operators who do not require it — including government-tier deployments with HSM-backed keys — may disregard the rotation mechanism entirely without violating protocol compliance.

---

#### 3.5.8 Node Decommission

When a Node is permanently shut down, it SHOULD send a final `node_announcement` with `valid_until` set to the current timestamp. This signals to peers that the Node is no longer available and they should not attempt reconnection. A Node that is decommissioned due to key compromise MUST NOT send a final announcement — doing so would use the compromised key and could mislead peers.

After decommission, the operator SHOULD notify Space administrators of affected Spaces out-of-band so that `state.federation_remove` Events can be produced and federation relationships updated.

---

### 3.6 Identity Registration Protocol

*Status: wip*

How a user creates an Identity and registers it with a Node. An Identity is the user's permanent presence on the XGen network — derived from a keypair, self-certifying, and independent of any specific Node. Registration is the process of making that Identity known to a Node so it can send and receive Events.

---

#### 3.6.1 Client-Side Keypair Generation

The client generates an Ed25519 keypair on the user's device before registration begins. The keypair is generated locally — it never leaves the device in plaintext. The public key becomes the Identity ID. The private key is used to sign Events and to authenticate transport connections.

The private key MUST be stored encrypted at rest on the client device. The encryption mechanism is implementation-defined and platform-appropriate — a mobile client may use the device's secure enclave; a desktop client may use an OS keychain or encrypted file. The spec requires only that the private key is not stored in plaintext.

A user MAY have multiple devices, each with its own keypair. Multi-device Identity management is covered in 3.6.6. For Phase 1, a single device with a single keypair is the baseline.

---

#### 3.6.2 Identity ID Derivation

The Identity ID is the pubkey_uri of the client's Ed25519 public key:

```
identity_id = xgen://pubkey/ed25519:<base64url-encoded-public-key>
```

Example:
```
xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIHvoNgEMoFYGNhWMTRSXqFGrjWYRBhKVNBnPXVwB
```

The Identity ID is self-certifying and globally unique — no central authority assigns it, and no two keypairs can produce the same ID (barring a cryptographic collision, which is computationally infeasible). The Identity ID is permanent for the lifetime of the keypair.

---

#### 3.6.3 Registration Request Schema

To register with a Node, the client sends an `identity.register` message after completing transport authentication (3.3.4). Note that transport authentication proves the client holds the private key — registration is the separate step of creating a persistent Identity record on the Node.

```json
{
  "protocol_version": "0.1",
  "type": "identity.register",
  "identity_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
  "display_name": "Jozef N",
  "trust_assertion": {
    "tier": 1,
    "issuer": "xgen://pubkey/ed25519:AUTH_MODULE_PUBLIC_KEY...",
    "issued_at": "2026-04-26T10:00:00.000Z",
    "valid_until": "2027-04-26T00:00:00.000Z",
    "claims": {
      "email_verified": true,
      "phone_verified": true
    },
    "signature": "ed25519:AUTH_MODULE_KEY...:base64url-signature"
  },
  "timestamp": "2026-04-26T10:00:00.000Z",
  "signature": "ed25519:AAAAC3Nz...:base64url-signature-over-canonical-form"
}
```

**Field definitions**

| Field | Type | Required | Description |
|---|---|---|---|
| `identity_id` | pubkey_uri | yes | The Identity being registered — MUST match the key used in transport authentication |
| `display_name` | string | no | Human-readable name for display in client UIs — not unique, not verified |
| `trust_assertion` | object | conditional | Required for Tier 1+ registration. Omitted for Local Node mode only |
| `timestamp` | datetime | yes | When this request was created |
| `signature` | string | yes | Signature over canonical form of this message, using the Identity private key |

**Trust Assertion**

The `trust_assertion` is a signed statement from an Auth Module certifying that this Identity has been verified to the declared Tier level. For Tier 1, this means email and phone number have been verified. The Trust Assertion is issued by the Auth Module before registration — the client presents it to the Node as proof of verification. The full Trust Assertion schema is specified in 3.8.

For Local Node mode, `trust_assertion` is omitted entirely. The Node accepts registration based on transport authentication alone.

---

#### 3.6.4 Node Acceptance Criteria

On receiving `identity.register`, the Node runs the following checks in order:

| Step | Check | Action on failure |
|---|---|---|
| 1 | `identity_id` matches the identity authenticated in transport Phase 2 | Reject — identity_mismatch |
| 2 | Signature over canonical form verifies against `identity_id` public key | Reject — signature_invalid |
| 3 | `identity_id` is not already registered on this Node | Reject — already_registered |
| 4 | `trust_assertion` present and valid for required Tier (if not Local Node) | Reject — trust_assertion_required |
| 5 | `trust_assertion` signature verifies against declared Auth Module key | Reject — assertion_signature_invalid |
| 6 | `trust_assertion` `valid_until` is in the future | Reject — assertion_expired |
| 7 | Auth Module that issued the assertion is trusted by this Node | Reject — auth_module_untrusted |
| 8 | Node has capacity to accept new Identities | Reject — node_capacity_exceeded |

On success, the Node sends `identity.register_ok`. On any failure, the Node sends `identity.register_fail` with the appropriate error code and closes the registration transaction (but not the transport connection — the client may correct and retry).

**`identity.register_ok`**:

```json
{
  "protocol_version": "0.1",
  "type": "identity.register_ok",
  "identity_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
  "registered_at": "2026-04-26T10:00:01.000Z"
}
```

**`identity.register_fail`**:

```json
{
  "protocol_version": "0.1",
  "type": "identity.register_fail",
  "error_code": 3003,
  "error_string": "trust_assertion_required",
  "timestamp": "2026-04-26T10:00:01.000Z"
}
```

---

#### 3.6.5 Identity Registration Error Codes

Registration errors are in the 3000 range, distinct from transport (1000) and federation (2000) error codes. Same dual numeric+string format and display rule as 3.3.8.

| Code | Error string | Meaning |
|---|---|---|
| 3001 | `identity_mismatch` | `identity_id` does not match the authenticated transport identity |
| 3002 | `signature_invalid` | Registration request signature did not verify |
| 3003 | `trust_assertion_required` | Node requires a Trust Assertion for this Tier — none provided |
| 3004 | `assertion_signature_invalid` | Trust Assertion signature did not verify |
| 3005 | `assertion_expired` | Trust Assertion `valid_until` is in the past |
| 3006 | `auth_module_untrusted` | The Auth Module that issued the assertion is not trusted by this Node |
| 3007 | `already_registered` | This Identity is already registered on this Node |
| 3008 | `node_capacity_exceeded` | Node has reached its maximum registered Identity count |
| 3009 | `display_name_invalid` | Display name contains prohibited characters or exceeds length limit |

**Display rule** — same pattern as 3.3.8:

```
Error 3003 (trust_assertion_required): This Node requires identity verification
before registration. Please complete verification with an Auth Module first.
```

---

#### 3.6.6 Identity Record Storage

On successful registration, the Node creates an Identity record and stores it persistently. The Identity record is the Node's authoritative local copy of what it knows about this Identity.

**Identity record structure**

```json
{
  "identity_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
  "display_name": "Jozef N",
  "registered_at": "2026-04-26T10:00:01.000Z",
  "trust_assertion": { ... },
  "devices": [
    {
      "device_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
      "device_name": "Laptop",
      "authorised_at": "2026-04-26T10:00:01.000Z"
    }
  ],
  "home_node": "xgen://pubkey/ed25519:NODE_PUBLIC_KEY..."
}
```

For Phase 1, the `identity_id` and the `device_id` of the first device are identical — the user has one device and one keypair. The `devices` array exists from day one so Phase 2 multi-device support requires no schema change.

The `home_node` field records which Node the Identity first registered with. The home Node is the authoritative source of truth for this Identity's current record (referenced in the conflict resolution Layer 3, 3.2.7).

---

#### 3.6.7 Identity Record Retrieval

Other Nodes and clients need to resolve an Identity — to fetch its public key, trust assertion, and current record — without connecting directly to the home Node every time.

**Direct retrieval**

A client or Node sends `identity.get` to any Node that has a copy of the record:

```json
{
  "protocol_version": "0.1",
  "type": "identity.get",
  "identity_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5..."
}
```

The Node responds with `identity.record` if it has the record, or `identity.not_found` if it does not. A Node that does not have the record MAY forward the query to the Identity's home Node if it knows it.

**Replication**

When a new Identity registers, the home Node replicates the record to N peer Nodes (the replication parameter N is specified in 3.13, Phase 2). For Phase 1 with two Nodes, the record is shared between both Nodes directly over the federation channel.

---

#### 3.6.8 Identity Update Propagation

An Identity record may change after initial registration. Phase 1 supports one update type: display name change. Phase 2 adds: Trust Assertion renewal, device addition/removal, and key rotation.

Updates are sent as `identity.update` messages signed by the Identity's private key:

```json
{
  "protocol_version": "0.1",
  "type": "identity.update",
  "identity_id": "xgen://pubkey/ed25519:AAAAC3NzaC1lZDI1NTE5...",
  "update_version": 2,
  "changes": {
    "display_name": "Jozef Novak"
  },
  "timestamp": "2026-04-26T12:00:00.000Z",
  "signature": "ed25519:AAAAC3Nz...:base64url-signature"
}
```

The `update_version` is a monotonically increasing integer. Receiving Nodes apply an update only if its `update_version` is strictly higher than the stored version — same pattern as `announcement_version` in 3.5.6. This prevents replay of old updates.

The home Node propagates accepted updates to all replica Nodes. For Phase 1, the update is sent directly over the active federation connection.

---

#### 3.6.9 Local Node Registration

In Local Node mode, Trust Assertions are not required. The Node accepts registration based on transport authentication alone — the client proves it holds the private key, and that is sufficient. The `trust_assertion` field is omitted from `identity.register`. Steps 4–7 in the acceptance pipeline (3.6.4) are skipped.

This mode exists for development and testing only. A Node MUST NOT accept Local Node registrations if it is not in Local Node mode (i.e. if external network interfaces are active).

---

### 3.7 Space & Room Protocol

*Status: wip*

How Spaces and Rooms are created, maintained, and federated. Spaces are the federation and membership containers — they define the Auth Tier, the set of federated Nodes, and the set of member Identities. Rooms are messaging channels within a Space. A Space may contain multiple Rooms; a Room belongs to exactly one Space.

---

#### 3.7.1 Space and Room Model

**Space**

A Space is the top-level organisational unit in XGen. It has:
- A declared Auth Tier that applies to all members and all Rooms within it
- A set of federated Nodes that replicate its Event logs
- A set of member Identities with assigned roles
- A set of Rooms
- A Space owner (the Identity that created it)

A Space is not a communication channel — it is the container that governs who can communicate and under what rules. All communication happens in Rooms.

**Room**

A Room is a messaging channel within a Space. It has:
- Its own Event DAG (the append-only log of all Events in the Room)
- Its own member list (a subset of Space members who have joined the Room)
- Its own state (name, topic, avatar)
- An optional encryption setting (Phase 2)

A Room member must first be a Space member. Joining a Space does not automatically join all Rooms — Room membership is separate.

**DM Space**

A DM Space is a restricted variant of a Space with exactly two members and a single Room. It is created by one Identity inviting another directly — no Space owner role, no federation across multiple Nodes beyond the two participants' home Nodes. A DM Space may be promoted to a full Space later (3.16, Phase 2). For Phase 1, DM Spaces are the simplest test case: two users, two Nodes, one conversation.

---

#### 3.7.2 Space ID and Room ID Derivation

Space IDs and Room IDs are hash URIs derived from the canonical form of their creation Events, following the same content-addressing pattern as Event IDs (3.2.3).

```
space_id = xgen://hash/sha256:<sha256-of-canonical-state.space_create-event>
room_id  = xgen://hash/sha256:<sha256-of-canonical-state.room_create-event>
```

Because creation Events include the creator's `identity_id`, `timestamp`, and a mandatory `nonce` field, Space and Room IDs are unique even if two creators produce identically-named Spaces at the same moment. The nonce MUST be 16 cryptographically random bytes, base64url-encoded.

---

#### 3.7.3 Space Creation

A Space is created by producing a `state.space_create` Event. This Event is the root of the Space's own state DAG. The creator automatically becomes the Space owner.

```json
{
  "protocol_version": "0.1",
  "type": "state.space_create",
  "event_id": "xgen://hash/sha256:...",
  "sender": "xgen://pubkey/ed25519:CREATOR_KEY...",
  "room_id": "",
  "space_id": "",
  "prev_events": [],
  "timestamp": "2026-04-26T10:00:00.000Z",
  "content": {
    "name": "XGen Dev Team",
    "topic": "Protocol development",
    "auth_tier": 1,
    "max_event_size": 65536,
    "nonce": "base64url-16-random-bytes",
    "home_node": "xgen://pubkey/ed25519:NODE_KEY..."
  },
  "signature": "ed25519:...:base64url-signature"
}
```

**Notes on special fields**

`room_id` and `space_id` are empty strings in `state.space_create` — the Space does not yet exist when this Event is created, so there is no ID to reference. The `space_id` is derived from this Event's own hash after the fact.

`prev_events` is an empty array — this is the DAG root, identical to `state.room_create`.

`max_event_size` is optional. If omitted, the Tier ceiling applies (3.1.1). If declared, it MUST be ≤ the Tier ceiling.

`home_node` declares which Node is the authoritative home for this Space. Other Nodes may federate, but the home Node is the source of truth for Space state.

**Space content field definitions**

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | Space display name — max 100 chars |
| `topic` | string | no | Space topic — max 500 chars |
| `auth_tier` | integer | yes | Auth Tier for this Space (1–4) |
| `max_event_size` | integer | no | Space-level envelope size override — MUST be ≤ Tier ceiling |
| `nonce` | string | yes | 16 random bytes base64url — ensures unique Space ID |
| `home_node` | pubkey_uri | yes | Node that hosts this Space |

---

#### 3.7.4 DM Space Creation

A DM Space is created with `state.dm_space_create`. It is structurally identical to `state.space_create` with three constraints enforced by the Node:

1. Maximum member count is 2 — the creator and one invitee
2. Exactly one Room is created automatically at Space creation
3. No additional Rooms may be created in a DM Space

```json
{
  "protocol_version": "0.1",
  "type": "state.dm_space_create",
  "prev_events": [],
  "content": {
    "auth_tier": 1,
    "invitee": "xgen://pubkey/ed25519:OTHER_USER_KEY...",
    "nonce": "base64url-16-random-bytes",
    "home_node": "xgen://pubkey/ed25519:NODE_KEY..."
  },
  "signature": "ed25519:...:base64url-signature"
}
```

The `invitee` field carries the Identity ID of the other participant. No `name` or `topic` — DM Spaces are identified by their participants, not by a name. The Node automatically creates the single Room and sends a `membership.invite` to the invitee's home Node.

---

#### 3.7.5 Room Creation

A Room is created within an existing Space by producing a `state.room_create` Event. Only Space members with role `admin` or higher may create Rooms. The Space owner may always create Rooms.

```json
{
  "protocol_version": "0.1",
  "type": "state.room_create",
  "event_id": "xgen://hash/sha256:...",
  "sender": "xgen://pubkey/ed25519:CREATOR_KEY...",
  "room_id": "",
  "space_id": "xgen://hash/sha256:SPACE_ID...",
  "prev_events": [],
  "timestamp": "2026-04-26T10:00:00.000Z",
  "content": {
    "name": "general",
    "topic": "General discussion",
    "nonce": "base64url-16-random-bytes"
  },
  "signature": "ed25519:...:base64url-signature"
}
```

`room_id` is empty — derived from this Event's hash after creation. `prev_events` is empty — this is the Room DAG root. `space_id` is present and MUST reference a valid existing Space.

**Room content field definitions**

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | Room display name — max 100 chars, unique within Space |
| `topic` | string | no | Room topic — max 500 chars |
| `nonce` | string | yes | 16 random bytes base64url — ensures unique Room ID |

---

#### 3.7.6 Space State

The current state of a Space is derived by processing all State Events in the Space's state DAG in causal order. For Phase 1 (no concurrent state changes expected), the most recent State Event of each type is authoritative.

**Space state components**

| State key | Set by EventType | Description |
|---|---|---|
| `name` | `state.space_name` | Current Space display name |
| `topic` | `state.space_topic` | Current Space topic |
| `avatar` | `state.space_avatar` | URI of Space avatar image |
| `member_list` | `membership.*` Events | Current Space members with roles |
| `federation_list` | `state.federation_add/remove` | Current federated Nodes |
| `node_priority` | `state.node_priority` | Manual Node ordering (3.2.7) |
| `max_event_size` | set at creation, immutable | Envelope size override |
| `auth_tier` | set at creation, immutable | Auth Tier — immutable after creation |

`auth_tier` and `max_event_size` are immutable — they are set at Space creation and cannot be changed. Changing either requires Space migration (3.12, Phase 2).

---

#### 3.7.7 Room State

The current state of a Room is derived from its Event DAG by the same process as Space state.

**Room state components**

| State key | Set by EventType | Description |
|---|---|---|
| `name` | `state.room_name` | Current Room display name |
| `topic` | `state.room_topic` | Current Room topic |
| `avatar` | `state.room_avatar` | URI of Room avatar image |
| `member_list` | `membership.*` Events | Current Room members |

---

#### 3.7.8 Space Membership

Space membership is managed by `membership.*` Events produced in the Space's state DAG. Roles are: `owner`, `admin`, `moderator`, `member`.

**`membership.invite`** — sent by admin or owner to invite an Identity:

```json
{
  "type": "membership.invite",
  "content": {
    "target_identity": "xgen://pubkey/ed25519:INVITEE_KEY...",
    "role": "member"
  }
}
```

**`membership.join`** — sent by the invited Identity to accept:

```json
{
  "type": "membership.join",
  "content": {
    "invited_by": "xgen://pubkey/ed25519:INVITER_KEY..."
  }
}
```

**`membership.leave`** — sent by the member voluntarily:

```json
{
  "type": "membership.leave",
  "content": {}
}
```

**`membership.kick`** — sent by admin or owner to remove a member:

```json
{
  "type": "membership.kick",
  "content": {
    "target_identity": "xgen://pubkey/ed25519:MEMBER_KEY...",
    "reason": "Violated community guidelines"
  }
}
```

**`membership.ban`** — sent by admin or owner to ban a member permanently:

```json
{
  "type": "membership.ban",
  "content": {
    "target_identity": "xgen://pubkey/ed25519:MEMBER_KEY...",
    "reason": "Repeated violations"
  }
}
```

**Role permission table**

| Action | member | moderator | admin | owner |
|---|---|---|---|---|
| Send messages | ✅ | ✅ | ✅ | ✅ |
| Invite members | ❌ | ✅ | ✅ | ✅ |
| Kick members | ❌ | ✅ | ✅ | ✅ |
| Ban members | ❌ | ❌ | ✅ | ✅ |
| Create Rooms | ❌ | ❌ | ✅ | ✅ |
| Change Space name/topic | ❌ | ❌ | ✅ | ✅ |
| Manage federation | ❌ | ❌ | ❌ | ✅ |
| Set node_priority | ❌ | ❌ | ❌ | ✅ |

---

#### 3.7.9 Room Membership

Room membership is a subset of Space membership. A Space member may join any Room they have access to. Room membership is tracked by `membership.*` Events in the Room's Event DAG.

For Phase 1, all Rooms in a Space are open to all Space members — there are no private Rooms. Private Rooms (invitation-only within a Space) are a Phase 2 feature.

**`membership.join` in a Room** — sent by a Space member to join the Room:

```json
{
  "type": "membership.join",
  "space_id": "xgen://hash/sha256:SPACE_ID...",
  "room_id": "xgen://hash/sha256:ROOM_ID...",
  "content": {}
}
```

The same `membership.leave`, `membership.kick`, and `membership.ban` EventTypes apply at Room level with the same schemas. A Space admin may kick/ban from a Room; only the Space owner may kick/ban from the Space itself.

---

#### 3.7.10 Space Federation Initiation

When a new Node wants to host members of an existing Space, it initiates federation with the Space's home Node. The full sequence:

```
1. New Node establishes transport connection to home Node (3.3)
2. New Node completes transport authentication (3.3.4)
3. New Node initiates federation handshake (3.4)
4. Handshake completes — session established
5. New Node sends space.join_request to home Node:

   {
     "type": "space.join_request",
     "space_id": "xgen://hash/sha256:SPACE_ID...",
     "node_id": "xgen://pubkey/ed25519:NEW_NODE_KEY..."
   }

6. Home Node verifies the requesting Node's announcement (3.5.3)
7. Space owner approves (manually or via policy) — home Node produces:

   state.federation_add Event in Space DAG (3.4.5)

8. Home Node sends full Space state and Room Event history to new Node
9. New Node is now a full federation participant
```

For Phase 1 — two Nodes, one Space — step 7 is automatic: the home Node approves all valid federation requests. Manual approval policy is Phase 2.

---

#### 3.7.11 Minimal Test Space — Phase 1 Smoke Test

The exact Event sequence required to reach a working two-Node, two-user, one-Room conversation. This is the Phase 1 definition of done.

```
Node A setup:
  1. Node A generates keypair → Node A ID
  2. User Alice registers Identity on Node A → Alice ID

Node B setup:
  3. Node B generates keypair → Node B ID
  4. User Bob registers Identity on Node B → Bob ID

Space creation:
  5. Alice produces state.space_create → Space ID derived
  6. Alice produces state.room_create → Room ID derived
  7. Alice produces membership.invite (target: Bob, role: member)

Federation:
  8. Node B connects to Node A (transport + federation handshake)
  9. Node B sends space.join_request for Space ID
  10. Node A produces state.federation_add → Bob's Node is federated
  11. Node A sends Space state + Room Event history to Node B

Bob joins:
  12. Bob (via Node B) produces membership.join for the Space
  13. Bob produces membership.join for the Room

Conversation:
  14. Alice produces message.text ("Hello Bob") → Event delivered to Node B
  15. Bob produces message.text ("Hello Alice") → Event delivered to Node A
  16. Both Nodes have both Events in their Room DAG
  17. Both clients display the conversation

Phase 1 complete. ✅
```

**For DM Space smoke test** — steps 5–13 above collapse to:

```
  5. Alice produces state.dm_space_create (invitee: Bob)
  6. Node A sends membership.invite to Node B automatically
  7. Bob produces membership.join
  8. Single Room created automatically — ready for messages
```

---

### 3.8 Auth Module — Tier 1 Specification

*Status: wip*

The complete specification for the Tier 1 Community Auth Module and the Auth Module interface contract that all Tiers share. Section 3.8 has two distinct jobs: it specifies the concrete Tier 1 implementation, and it defines the interface slot that Tier 2–4 Auth Modules implement in Phase 2.

---

#### 3.8.1 Auth Module Role

An Auth Module is an external service that verifies real-world identity claims and issues signed Trust Assertions. It is neither a Node nor a client — it is a trusted third-party service that the Node operator has chosen to rely on for Identity verification.

The relationship chain is:

```
Auth Module  →  issues Trust Assertion  →  carried by client  →  presented to Node at registration
```

The Node trusts specific Auth Modules by their registered public key. Only assertions signed by a registered Auth Module are accepted. The Node operator is responsible for choosing which Auth Modules to trust.

An Auth Module operates independently of the XGen Node infrastructure. It may be run by the Node operator, by a trusted institution, or by a third-party verification service. The spec defines the interface — the implementation is the Auth Module operator's responsibility.

---

#### 3.8.2 The Auth Module Interface Contract

Every Auth Module regardless of Tier MUST implement the following interface. This is the slot specification that Phase 2 Tier 2–4 Auth Modules implement without protocol changes.

**Required capabilities**

- Generate and publish an Ed25519 keypair. The public key is the Auth Module's identity.
- Accept verification requests from clients via a defined message schema.
- Perform verification according to its Tier's requirements.
- Issue signed Trust Assertions upon successful verification.
- Provide a queryable endpoint for Nodes to verify assertion validity.
- Support assertion renewal before expiry.

**Auth Module public record**

Every Auth Module publishes a signed record declaring its identity and capabilities:

```json
{
  "type": "auth_module_record",
  "auth_module_id": "xgen://pubkey/ed25519:AUTH_MODULE_KEY...",
  "tier": 1,
  "name": "XGen Community Verifier",
  "verification_methods": ["email", "phone"],
  "claims_issued": ["tier_verified", "email_verified", "phone_verified", "email_hash", "phone_hash"],
  "endpoint": "https://auth.example.org/xgen",
  "valid_until": "2027-04-26T00:00:00.000Z",
  "signature": "ed25519:AUTH_MODULE_KEY...:base64url-signature"
}
```

This record is registered with the Node operator when the Auth Module is added to the Node's trusted list (3.8.7).

**Verification request** — sent by client to Auth Module:

```json
{
  "type": "auth.verify_request",
  "identity_id": "xgen://pubkey/ed25519:CLIENT_KEY...",
  "tier": 1,
  "timestamp": "2026-04-26T10:00:00.000Z",
  "signature": "ed25519:CLIENT_KEY...:base64url-signature"
}
```

**Verification complete** — Auth Module issues Trust Assertion (see 3.8.4).

**Assertion validity query** — Node queries Auth Module to confirm an assertion is still valid:

```json
{
  "type": "auth.assertion_query",
  "identity_id": "xgen://pubkey/ed25519:CLIENT_KEY...",
  "auth_module_id": "xgen://pubkey/ed25519:AUTH_MODULE_KEY..."
}
```

Auth Module responds with `auth.assertion_valid` or `auth.assertion_revoked`.

---

#### 3.8.3 Tier 1 Verification States

Tier 1 verification is based on email address and/or phone number confirmation. The Node operator chooses which verification state their Auth Module enforces. All four states are valid Tier 1 — they represent operator policy, not trust level.

| State | Phone | Email | Typical use case |
|---|---|---|---|
| A | none | none | Internal/closed community — operator vouches personally |
| B | none | real | Standard community node — email sufficient |
| C | real | none | SMS-verified, email-free deployments |
| D | real | real | Maximum contact verification — default for most production nodes |

**Verification flow**

```
1. Client sends auth.verify_request to Auth Module
2. Auth Module sends verification code(s) to declared contact method(s)
   — email: link or 6-digit code to email address
   — phone: 6-digit SMS code to phone number
3. Client submits code(s) via auth.verify_confirm
4. Auth Module verifies codes, issues Trust Assertion
5. Client presents Trust Assertion to Node at registration (3.6.3)
```

`auth.verify_confirm` — sent by client to submit verification codes:

```json
{
  "type": "auth.verify_confirm",
  "identity_id": "xgen://pubkey/ed25519:CLIENT_KEY...",
  "email_code": "847291",
  "phone_code": "391047",
  "timestamp": "2026-04-26T10:05:00.000Z",
  "signature": "ed25519:CLIENT_KEY...:base64url-signature"
}
```

Fields `email_code` and `phone_code` are present only if the respective verification method is active. Codes expire after 10 minutes.

---

#### 3.8.4 Trust Assertion Schema

The Trust Assertion is the signed statement issued by an Auth Module certifying that an Identity has been verified. It is the central artefact of the XGen trust model.

```json
{
  "type": "trust_assertion",
  "tier": 1,
  "issuer": "xgen://pubkey/ed25519:AUTH_MODULE_KEY...",
  "identity_id": "xgen://pubkey/ed25519:CLIENT_KEY...",
  "issued_at": "2026-04-26T10:06:00.000Z",
  "valid_until": "2027-04-26T00:00:00.000Z",
  "claims": {
    "tier_verified": true,
    "email_verified": true,
    "phone_verified": false,
    "email_hash": "sha256:a3f9b2c1d4e8f1a2b3c4d5e6f7a8b9c0..."
  },
  "signature": "ed25519:AUTH_MODULE_KEY...:base64url-signature"
}
```

**Field definitions**

| Field | Type | Required | Description |
|---|---|---|---|
| `tier` | integer | yes | Auth Tier this assertion certifies |
| `issuer` | pubkey_uri | yes | Auth Module that issued this assertion |
| `identity_id` | pubkey_uri | yes | Identity this assertion is for |
| `issued_at` | datetime | yes | When the assertion was issued |
| `valid_until` | datetime | yes | When the assertion expires |
| `claims` | object | yes | Verification claims — see below |
| `signature` | string | yes | Auth Module signature over canonical form |

**The `claims` object**

The claims object reflects what the Auth Module actually verified. `tier_verified` is the only mandatory claim — all others are optional and reflect the operator's chosen verification state.

| Claim | Type | Meaning |
|---|---|---|
| `tier_verified` | boolean | MANDATORY — Auth Module certifies this Identity meets Tier 1 standard |
| `email_verified` | boolean | An email address was verified |
| `phone_verified` | boolean | A phone number was verified |
| `email` | string | Plaintext email address — propagates to all federated Nodes |
| `phone` | string | Plaintext phone number — propagates to all federated Nodes |
| `email_hash` | hash string | Salted SHA-256 hash of email — only the hash propagates |
| `phone_hash` | hash string | Salted SHA-256 hash of phone — only the hash propagates |

**Three contact data options for Node operators**

Node operators choose how contact details appear in assertions. Each option has different privacy and utility tradeoffs:

**Option 1 — Plaintext**
The actual email or phone number appears in the claim. Maximum utility — the Node can display it, use it for contact, check for duplicates. Data propagates to every federated Node and is stored there indefinitely.

```json
"claims": { "tier_verified": true, "email_verified": true, "email": "user@example.com" }
```

**Option 2 — Hashed**
A salted SHA-256 hash of the contact detail appears. The Auth Module can re-verify the hash against its own records. The Node cannot extract the original contact detail. Only the hash propagates across the federation — useless to an attacker without the original value.

```json
"claims": { "tier_verified": true, "email_verified": true, "email_hash": "sha256:a3f9b2c1..." }
```

**Option 3 — Flag only**
Only the verification fact appears. No contact detail — not even a hash — enters the protocol. Any Node needing contact details must query the Auth Module directly using the `identity_id`.

```json
"claims": { "tier_verified": true, "email_verified": true }
```

**Privacy propagation warning**

Node operators who include plaintext contact details in assertions (Option 1) MUST inform users that this data will be replicated to all federated Nodes and stored there for the duration of the federation relationship. This is a GDPR-relevant consideration. Operators subject to right-to-erasure obligations should use Option 2 or Option 3.

---

#### 3.8.5 Trust Assertion Signing and Validation

**Signing**

The Auth Module signs the canonical form of the Trust Assertion using its Ed25519 private key. The canonical form follows the same rules as Event canonicalisation (3.2.4): no whitespace, keys sorted within objects, UTF-8 encoding. Field order: `type`, `tier`, `issuer`, `identity_id`, `issued_at`, `valid_until`, `claims`.

**Validation by the Node**

On receiving a Trust Assertion (embedded in `identity.register`, 3.6.3), the Node validates:

1. `issuer` is a registered Auth Module on this Node (3.8.7)
2. Signature verifies against the `issuer` public key
3. `identity_id` matches the registering Identity
4. `tier` matches or exceeds the Node's required Tier
5. `valid_until` is in the future
6. `claims` contains `tier_verified: true`
7. `claims` contains the contact verification claims required by this Node's policy

All seven checks MUST pass. Failure at any step results in registration rejection with the appropriate 3xxx error code (3.6.5).

---

#### 3.8.6 Trust Assertion Expiry and Renewal

**TTL**

Tier 1 Trust Assertions have a recommended TTL of 1 year (`valid_until` = 365 days from `issued_at`). This is a work definition — operators may choose shorter TTLs for higher-security deployments.

A Node MUST reject any assertion whose `valid_until` is in the past. An Identity with an expired assertion cannot register on new Nodes. It remains registered on Nodes where it was accepted before expiry, but its registration status on those Nodes becomes `assertion_expired` — the Node MAY restrict the Identity's ability to produce Events until renewal is complete.

**Renewal**

The client initiates renewal by running the full verification flow again with the Auth Module before the current assertion expires. The Auth Module issues a new assertion with a new `valid_until`. The client sends `identity.update` (3.6.8) to its home Node with the new assertion. The home Node propagates the updated record to replica Nodes.

The recommended renewal window is 30 days before expiry — clients SHOULD prompt users to renew when their assertion has less than 30 days remaining.

---

#### 3.8.7 Auth Module Registration with a Node

Before a Node accepts assertions from an Auth Module, the Node operator must explicitly register the Auth Module. This is a deliberate trust decision — a Node does not automatically trust any Auth Module.

**Registration process**

1. The Auth Module operator provides the Node operator with the Auth Module's public record (3.8.2) out-of-band — via secure channel, documented handoff, or in-person.
2. The Node operator verifies the record's signature using the declared `auth_module_id` public key.
3. The Node operator adds the `auth_module_id` to the Node's trusted Auth Module list via Node configuration.
4. The Node stores the full Auth Module public record locally for future assertion validation.

**Trusted Auth Module list**

The Node's trusted Auth Module list is a configuration file, not a protocol-level record. It is not broadcast to peers. Each Node operator independently decides which Auth Modules to trust. Two federated Nodes may trust different Auth Modules — Identities registered under different Auth Modules can coexist in the same Space as long as both Auth Modules are trusted by the Space's home Node.

---

#### 3.8.8 Local Node Bypass

In Local Node mode, the Auth Module is bypassed entirely. No Trust Assertion is required for registration. The Node accepts any Identity that can authenticate at the transport level (3.3.4).

This bypass is governed by the same Local Node constraint that applies throughout the spec: a Node in Local Node mode MUST refuse all external network connections. The bypass cannot be exploited over a network because Local Node mode is structurally localhost-only.

The bypass is stated here as the Auth Module's own rule: *an Auth Module MUST NOT issue assertions to Identities that will register on production Nodes operating in Local Node mode.* This is a logical constraint, not a technical enforcement — Local Node mode is a development tool and production assertions are not meaningful in that context.

---

## Phase 2 — Full Protocol

### 3.9 State Resolution Algorithm

*Status: pending — deferred to Phase 2*

The deterministic algorithm for resolving conflicting State Events in federated Rooms. Architectural commitments from Chapter 2:

- Deterministic — same inputs always produce the same output
- Convergent — all Nodes arrive at the same state given the same Event set
- Scale-aware — tractable as Room membership and federation breadth grow
- Auth-rule-aware — State Events violating auth rules are rejected regardless of ordering

*This section will be specified after Phase 1 implementation provides real conflict scenarios to reason about.*

---

### 3.10 End-to-End Encryption

*Status: pending — deferred to Phase 2*

The encryption protocol for encrypted Rooms. Interface commitments from Chapter 2:

- Encryption is optional per Room — not all Rooms are encrypted
- Federated Nodes receive encrypted Events — they store and propagate but cannot read
- The encryption boundary is a protocol guarantee, not a client feature
- Algorithm agility — the encryption algorithm is declared, not hardcoded

*Algorithm choice (MLS, Megolm, or custom) will be specified after Phase 1 is stable. The interface — how encrypted Events differ from plaintext Events at the wire level — will be specified here regardless.*

---

### 3.11 Auth Module — Tiers 2–4 Interfaces

*Status: pending — deferred to Phase 2*

Interface specifications for Tier 2 (Professional), Tier 3 (Corporate), and Tier 4 (Government) Auth Modules. The slot contract is fully defined here. The implementations are developed in institutional collaboration.

*See Chapter 2 — Auth Module & Trust Assertion for the architectural framework.*

---

### 3.12 Space Migration Protocol

*Status: pending — deferred to Phase 2*

The atomic protocol for migrating a Space from one Node to another. Covers:

- Migration initiation — who can trigger, what permissions are required
- Migration sequence — Event-by-Event atomic transfer
- History preservation — ensuring no Events are lost during migration
- Federation re-establishment at the new Node
- Member notification
- Old Node decommission and redirect

---

### 3.13 Identity Replication Parameters

*Status: pending — deferred to Phase 2*

The precise parameters for Identity record replication across the network. Covers:

- N value — how many replica Nodes a new Identity is propagated to
- Replica Node selection algorithm
- Update propagation — how Identity updates reach all replicas
- Replica refresh — how stale replicas are detected and updated
- Orphaned Identity recovery — how a user re-registers after home Node loss

---

### 3.14 Bootstrap Node Protocol

*Status: pending — deferred to Phase 2*

The protocol for Bootstrap Nodes — the well-known Nodes that help new Nodes discover the network. Covers:

- Bootstrap Node directory format
- New Node registration with Bootstrap Nodes
- Directory query protocol
- Bootstrap Node trust — how a new Node knows which Bootstrap Nodes to trust at first run
- Bootstrap Node failure handling — what happens if all Bootstrap Nodes are unreachable

---

### 3.15 Node Reputation Format

*Status: pending — deferred to Phase 2*

The soft reputation signal format maintained by Bootstrap Nodes. Covers:

- Reputation signal structure
- Propagation mechanism
- Weighting and aggregation
- Defederation signal integration
- Privacy considerations — what reputation signals reveal about federation history

---

### 3.16 DM Space Promotion Sequence

*Status: pending — deferred to Phase 2*

The protocol for promoting a DM Space to a full Space. Covers:

- Promotion initiation — who can trigger, what the trigger Event looks like
- Constraint lifting — removing DM-specific constraints
- History preservation
- Member notification
- New Space capabilities unlocked on promotion

---

## Chapter 3 — Open Questions

*To be populated as specification work progresses.*

---

## Chapter 3 — Known Tradeoffs

*To be populated as specification work progresses.*

---

## Chapter 3 — Handoff to Chapter 4

*To be written when Chapter 3 Phase 1 is complete.*

---

## Session Log

### Session 1 — April 2026 (JozefN)
**Covered:** Chapter 3 skeleton written. Two-phase structure established — Phase 1 (Minimal Viable Protocol, 8 sections) and Phase 2 (Full Protocol, 8 sections). Interface-first principle stated. Each section defined with its scope and deferred/pending status. Phase 1 covers: Wire Format, Event Specification, Transport Protocol, Federation Handshake, Node Identity Protocol, Identity Registration Protocol, Space & Room Protocol, Auth Module Tier 1. Phase 2 covers: State Resolution, E2E Encryption, Auth Modules Tiers 2–4, Space Migration, Identity Replication Parameters, Bootstrap Node Protocol, Node Reputation Format, DM Space Promotion.

**Next session to begin with:**
> **3.1 Wire Format.** The foundation everything else is built on. JSON as primary format, field conventions, URI formats, datetime format, binary encoding, message size limits, versioning.

### Session 2 — April 2026 (JozefN)
**Covered:** Section 3.1.1 Message Size Limits written. Two-layer size model established: Tier ceiling (hard protocol limit by Auth Tier) and Space override (tighter limit set at creation, immutable). Binary content excluded from protocol messages by design — content by reference only, base64url reserved for cryptographic material. Size reference table added covering 2KB–256KB range with byte counts, ASCII character counts, and usable JSON content estimates. Tier ceiling table: Local Node = 256KB (localhost only, not a wire-level Tier), Tier 1 = 64KB, Tier 2 = 32KB, Tier 3 = 16KB, Tier 4 = 8KB. All values marked as work definitions pending Phase 1 testing validation. Local Node mode defined as a Node configuration flag, not a protocol-level concept — no external federation permitted, localhost only, structurally prevents network exploitation. Enforcement rule: reject before signature verification. Section 3.1.2 rewritten as Primary Format and Format Agility: JSON mandatory baseline, serialisation format treated as open registry capability (same principle as crypto algorithm agility), negotiated at session establishment, fixed for session duration. Transport framing defined: length-prefixed UTF-8 format identifier string + 4-byte payload length + payload. Two hex-level framing examples written (JSON and MessagePack). Signing is format-independent — signatures over canonical field representation, not wire bytes. Sections 3.1.3 through 3.1.10 written: Field Naming (snake_case, stable, forward-compatible), Required vs Optional fields, Null Prohibition (null banned — absent means absent), URI Formats (xgen_uri, hash_uri, pubkey_uri grammars with examples), Datetime Format (RFC 3339 UTC, millisecond precision, Z suffix mandatory), Integer Precision (no floats, safe integer range enforced), Binary Data Encoding (base64url without padding, cryptographic material only), Protocol Versioning (major.minor string, major mismatch = reject, minor mismatch = accept with warning). Section 3.1 Wire Format complete.

### Session 3 — April 2026 (JozefN)
**Covered:** Section 3.2 Event Specification written in full. Decision confirmed: full DAG from day one — `prev_events` is always an array, Phase 1 uses it simply (usually one entry), Phase 2 federation stresses it properly without wire format changes. Six subsections written: 3.2.1 Event Envelope Schema, 3.2.2 EventType Registry, 3.2.3 Event ID Derivation, 3.2.4 Signature Canonicalisation, 3.2.5 The prev_events DAG (fork/merge ASCII diagram, rules, DAG tips), 3.2.6 Event Validation Pipeline (13-step ordered pipeline). Section 3.2.7 Conflict Resolution forward reference written: four conflict categories identified (state, permission, authority, ordering); seven-layer priority stack defined: Layer 1 EventType logic (hardcoded), Layer 2 Auth Tier (hardcoded), Layer 3 Home Node assertion (architectural), Layer 4 Role within Space (Tier default, Space-overridable), Layer 5a Manual Node ordering via drag-and-drop stored as state.node_priority Event (user-defined, beats automatic), Layer 5b Federation recency as automatic default (most recently joined Node has higher priority), Layer 5c Lexicographic event_id as absolute backstop (deterministic, ungameable, same result on every Node). state.node_priority Event schema defined. Full algorithm deferred to 3.9 State Resolution Phase 2.

### Session 4 — April 2026 (JozefN)
**Covered:** Section 3.3 Transport Protocol written in full. Nine subsections: 3.3.1 Transport Layer (WebSocket RFC 6455, Node-advertised endpoint URI, TLS mandatory in production, self-signed only in Local Node mode), 3.3.2 Connection Types (client→Node single persistent connection multiplexed by space_id/room_id; Node→Node mutually authenticated, one connection per federated peer), 3.3.3 Message Framing (one frame per WebSocket message, no fragmentation, malformed frame = immediate termination), 3.3.4 Connection Lifecycle (4-phase: CONNECT, AUTHENTICATE, ACTIVE, CLOSE; challenge-response using Identity keypair directly — nonce signed with private key, verified against registered public key, no session tokens, no server-side state; Node→Node mutual authentication; full message schemas for transport.challenge, transport.auth, transport.auth_ok, transport.auth_fail), 3.3.5 Keepalive (WebSocket ping/pong, 30s interval, 10s timeout, work definitions), 3.3.6 Reconnection Behaviour (exponential backoff with jitter, 30s ceiling; transport.sync_request for missed Event recovery after reconnect), 3.3.7 Rate Limiting (transport.rate_limit with retry_after_ms; ignore = disconnect; repeated violations reported to defederation subsystem Phase 2), 3.3.8 Transport Error Codes (10 defined codes with numeric+string dual format; display rule: Error <code> (<string>): <description>. <optional extended explanation> — serves technical staff, developers, and non-technical users simultaneously), 3.3.9 Graceful Close (transport.goodbye with defined reason values). Key decision confirmed: challenge-response with keypair is the natural and only consistent authentication mechanism — the keypair IS the identity.

### Session 5 — April 2026 (JozefN)
**Covered:** Section 3.4 Federation Handshake written in full. Key decision: one handshake per Node pair, not per Space — all shared Spaces multiplexed over a single federation channel. Seven subsections: 3.4.1 Purpose and Scope (federation relationship vs transport connection, initiating/receiving Node roles, handshake runs inside authenticated transport session), 3.4.2 Handshake Message Schemas (five messages: federation.hello with node_id/capabilities/shared_spaces, federation.capabilities with negotiated block, federation.accept with session_id derived from hash of both node IDs + timestamp, federation.reject with 2xxx error codes, federation.goodbye with reason values), 3.4.3 Handshake State Machine (IDLE → HELLO_RECEIVED → CAPS_SENT → ACTIVE → CLOSED; 10s response timeout, 15s wait timeout, unexpected message = reject + close), 3.4.4 Capability Negotiation (intersection algorithm, highest-preference common format selected, lower minor version wins for protocol version, unknown capabilities silently ignored, serialisation mandatory others optional), 3.4.5 Relationship Persistence (state.federation_add Event per Space per relationship — not per reconnection; local federation registry on Node; state.federation_remove on goodbye), 3.4.6 Re-federation (full handshake on reconnect, no session parameter assumptions, sync_request after re-federation), 3.4.7 Federation Handshake Error Codes (8 defined codes in 2000 range, same numeric+string dual format and display rule as 3.3.8).

### Session 6 — April 2026 (JozefN)
**Covered:** Section 3.5 Node Identity Protocol written in full. Eight subsections: 3.5.1 Node Keypair Generation (Ed25519, one-time on first run, private key encrypted at rest, refuse to start if key missing/corrupted, MUST NOT regenerate if key exists), 3.5.2 Node ID Derivation (pubkey_uri identical pattern to Identity ID, self-certifying, no external authority), 3.5.3 Node Announcement Schema (full field table: node_id, endpoint, capabilities, auth_tiers_served, operator_display_name, announcement_version monotonic counter, valid_until TTL, timestamp, signature; canonical form for signing defined), 3.5.4 Announcement Signing and Verification (self-certifying — extract pubkey from node_id, construct canonical form, verify; no third party needed; reject expired, signature-invalid, or node_id-mismatched announcements), 3.5.5 Announcement Propagation (direct exchange on connection after transport auth; peer relay with version gating; Bootstrap Node directory deferred to Phase 2), 3.5.6 Announcement Refresh (refresh at 80% TTL, re-announce on endpoint/capability/tier change, increment announcement_version, peers replace only if version strictly higher; 90-day TTL work definition), 3.5.7 Keypair Permanence and Key Rotation Policy (Phase 1: permanent keypair, decommission on compromise; Phase 2: rotation mechanism via system.key_rotation; key rotation is OPTIONAL not mandatory — including in Tier 4; HSM-backed permanent keys are a legitimate and compliant operational stance; rotation window risk is a valid reason to prefer permanence), 3.5.8 Node Decommission (final announcement with valid_until=now if clean shutdown; MUST NOT send final announcement on compromise; out-of-band notification to Space administrators).

### Session 7 — April 2026 (JozefN)
**Covered:** Section 3.6 Identity Registration Protocol written in full. Nine subsections: 3.6.1 Client-Side Keypair Generation (Ed25519 generated locally on device, private key never leaves in plaintext, encrypted at rest using platform-appropriate mechanism, multi-device array in schema from day one for Phase 2 compatibility), 3.6.2 Identity ID Derivation (pubkey_uri, self-certifying, globally unique, permanent for lifetime of keypair), 3.6.3 Registration Request Schema (identity.register message with identity_id, display_name, trust_assertion, timestamp, signature; transport auth proves key ownership, registration creates persistent record; trust_assertion is conditional — required for Tier 1+, omitted for Local Node), 3.6.4 Node Acceptance Criteria (8-step pipeline: identity_id matches transport auth, signature verifies, not already registered, trust_assertion present/valid, assertion signature verifies, assertion not expired, auth module trusted, node has capacity; register_ok and register_fail message schemas), 3.6.5 Identity Registration Error Codes (9 codes in 3000 range, same dual numeric+string format and display rule as 3.3.8), 3.6.6 Identity Record Storage (full record structure: identity_id, display_name, registered_at, trust_assertion, devices array, home_node; Phase 1 identity_id equals device_id, devices array future-proofed), 3.6.7 Identity Record Retrieval (identity.get → identity.record or identity.not_found; replication to N peers deferred to 3.13 Phase 2; Phase 1 direct federation channel sharing), 3.6.8 Identity Update Propagation (identity.update with update_version monotonic counter, same pattern as announcement_version; Phase 1 supports display name change only; Phase 2 adds Trust Assertion renewal, device management, key rotation), 3.6.9 Local Node Registration (trust_assertion omitted, steps 4–7 skipped, transport auth alone sufficient; MUST NOT accept if external interfaces active).

### Session 8 — April 2026 (JozefN)
**Covered:** Section 3.7 Space & Room Protocol written in full. Decision: DM Space included in Phase 1 (needed for testing). Eleven subsections: 3.7.1 Space and Room Model (Space as federation/membership container, Room as messaging channel, DM Space as two-member single-Room variant promotable to full Space in Phase 2), 3.7.2 Space ID and Room ID Derivation (hash URI of canonical creation Event, nonce field ensures uniqueness), 3.7.3 Space Creation (state.space_create schema, empty room_id/space_id at creation time, space_id derived from own hash, auth_tier and max_event_size immutable, home_node declared), 3.7.4 DM Space Creation (state.dm_space_create, max 2 members, single Room auto-created, invitee field, no name/topic), 3.7.5 Room Creation (state.room_create schema, room_id derived from hash, DAG root with empty prev_events, unique name within Space), 3.7.6 Space State (state components table: name/topic/avatar/member_list/federation_list/node_priority/max_event_size/auth_tier; auth_tier and max_event_size immutable), 3.7.7 Room State (name/topic/avatar/member_list), 3.7.8 Space Membership (membership.invite/join/leave/kick/ban schemas, role permission table: owner/admin/moderator/member), 3.7.9 Room Membership (subset of Space membership, Phase 1 all Rooms open to all Space members, private Rooms Phase 2), 3.7.10 Space Federation Initiation (9-step sequence: transport → auth → federation handshake → space.join_request → Node verification → approval → state.federation_add → history sync; Phase 1 auto-approval), 3.7.11 Minimal Test Space — Phase 1 Smoke Test (17-step full sequence for regular Space + DM Space shortcut, explicit Phase 1 definition of done).

### Session 9 — April 2026 (JozefN)
**Covered:** Section 3.8 Auth Module — Tier 1 Specification written in full. Phase 1 complete — all 8 sections written. Eight subsections: 3.8.1 Auth Module Role (external service, not Node or client, trusted by Node operator via public key, independent of XGen Node infrastructure), 3.8.2 Auth Module Interface Contract (slot spec for all Tiers: keypair, verification request, Trust Assertion issuance, validity query endpoint, renewal support; auth_module_record schema; auth.verify_request and auth.assertion_query schemas), 3.8.3 Tier 1 Verification States (four operator-chosen states: A=no phone+no email, B=no phone+real email, C=real phone+no email, D=real phone+real email; all valid Tier 1; represent policy not trust level; full verification flow with auth.verify_confirm schema, codes expire 10 min), 3.8.4 Trust Assertion Schema (full field table; claims object: tier_verified mandatory, email_verified/phone_verified/email/phone/email_hash/phone_hash optional; three contact data options: plaintext=propagates everywhere, hashed=only hash propagates, flag-only=nothing propagates; GDPR propagation warning for Option 1), 3.8.5 Trust Assertion Signing and Validation (canonical form rules, 7-step Node validation pipeline), 3.8.6 Trust Assertion Expiry and Renewal (1-year TTL work definition, assertion_expired status, renewal flow via identity.update, 30-day renewal window prompt), 3.8.7 Auth Module Registration with a Node (4-step out-of-band registration process, trusted Auth Module list is config file not protocol record, federated Nodes may trust different Auth Modules), 3.8.8 Local Node Bypass (Auth Module bypassed in Local Node mode, structurally unexploitable over network, logical constraint stated).

**Phase 1 complete. All 8 sections written. Ready for Phase 1 review and implementation.**

**Next step:**
> Review Chapter 3 Phase 1 as a whole for consistency, cross-reference accuracy, and completeness before moving to Phase 2 or implementation.