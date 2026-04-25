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

*Status: pending*

The complete wire specification for all Event types. Covers:

- Full Event envelope schema (all mandatory and optional fields)
- Event ID derivation — hash algorithm, input canonicalisation, output format
- Content schemas per EventType — one schema per event type in the taxonomy
- `prev_events` DAG construction rules
- Signature input canonicalisation — what exactly is signed, in what order
- Signature field format
- Event validation rules — what a receiving Node must check before accepting
- Event ordering rules — DAG traversal and causal ordering
- Handling unknown EventTypes — store, forward, do not reject

---

### 3.3 Transport Protocol

*Status: pending*

The network transport layer between clients and Nodes, and between Nodes. Covers:

- WebSocket as the primary transport
- TLS requirements (optional for v0.1 local testing, mandatory for production)
- Message framing — how protocol messages are wrapped for transport
- Connection lifecycle — connect, authenticate, heartbeat, disconnect
- Reconnection behaviour — backoff strategy, state recovery on reconnect
- Connection multiplexing — one connection carries all Spaces and Rooms
- Error codes and error message format
- Rate limiting signals from Node to client

---

### 3.4 Federation Handshake

*Status: pending*

The protocol for establishing a federation relationship between two Nodes. Covers:

- `federation.hello` message schema
- `federation.capabilities` message schema
- `federation.accept` message schema
- `federation.goodbye` message schema
- Handshake sequence and state machine
- Capability negotiation rules — what happens when capabilities don't match
- Version negotiation — how Nodes agree on a common protocol version
- Handshake failure codes and retry behaviour
- Federation relationship persistence — how established relationships are stored

---

### 3.5 Node Identity Protocol

*Status: pending*

How a Node establishes, announces, and proves its identity on the network. Covers:

- Node keypair generation on first run
- Node announcement message schema (`node_announcement`)
- Announcement signing — what fields are signed, in what order
- Announcement verification by receiving Nodes and clients
- Node ID derivation from public key
- Node announcement refresh — how often, what triggers a re-announcement
- Node announcement propagation — how announcements spread through the network
- Bootstrap Node registration — how a new Node announces itself to the network

---

### 3.6 Identity Registration Protocol

*Status: pending*

How a user creates an Identity and registers it with a Node. Covers:

- Client-side keypair generation
- Identity ID derivation from public key
- Initial device authorisation — the first device registration sequence
- Identity registration request message schema
- Node acceptance criteria — what a Node checks before accepting a new Identity
- Identity record storage format on the Node
- Identity record retrieval — how other Nodes and clients resolve an Identity
- Identity update protocol — how updates (key rotation, new device) are propagated
- Simplified Tier 0 registration for testing — no Auth Module required

---

### 3.7 Space & Room Protocol

*Status: pending*

How Spaces and Rooms are created, maintained, and federated. Covers:

- Space creation message schema
- Room creation message schema
- Space and Room ID derivation
- Space and Room current state storage format
- State Event processing — how State Events update current state
- Space membership — join, leave, invite message schemas
- Room membership — join, leave message schemas
- Space federation initiation — how a new Node joins a Space's federation
- Room Event log format — how Events are stored and retrieved
- Minimal Space for testing — one Space, one Room, two Nodes

---

### 3.8 Auth Module — Tier 1 Specification

*Status: pending*

The complete specification for the Tier 1 Community Auth Module. This is the only Auth Module that ships with XGen as a reference implementation. Covers:

- Tier 1 verification method: email address + phone number confirmation
- Verification request and response message schemas
- Trust Assertion schema for Tier 1
- Trust Assertion signing by the Auth Module
- Trust Assertion validation by Nodes and clients
- Trust Assertion expiry and renewal
- Tier 0 bypass for internal testing — raw keypair, no assertion required
- Auth Module interface contract — the slot specification that all Auth Modules must implement
- Auth Module registration with a Node

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

**Next session to begin with:**
> **3.2 Event Specification.** Full Event envelope schema, Event ID derivation, content schemas per EventType, prev_events DAG construction, signature canonicalisation, Event validation rules, causal ordering, unknown EventType handling.
