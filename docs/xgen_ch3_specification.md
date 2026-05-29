# XGen Protocol — Chapter 3: Specification
> **Status:** ACTIVE  
> Version: 0.3  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Overview

> **Traceability is a core invariant of XGen. All protocol Events are observable across their lifecycle via `event_id`, without exposing content.**

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
| 3.0 | Identifiers (XGID) | ✅ Complete |
| 3.1 | Wire Format | ✅ Complete |
| 3.1.11 | Reference Implementation Binary Names | ✅ Complete |
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
| 3.9 | State Resolution Algorithm | ✅ Complete |
| 3.10 | End-to-End Encryption | ✅ Complete |
| 3.11 | Auth Module — Tiers 2–4 Interfaces | ✅ Complete |
| 3.12 | Space Migration Protocol | ✅ Complete |
| 3.13 | Identity Replication Parameters | ✅ Complete |
| 3.14 | Bootstrap Node Protocol | ✅ Complete |
| 3.15 | Node Reputation Format | ✅ Complete |
| 3.16 | DM Space Promotion Sequence | ✅ Complete |
| 3.6.10 | AI Identity Extension | ✅ Complete |
| 3.7.12 | Pacing Rules on Spaces | ✅ Complete |
| 3.7.13 | Temperature Property | ✅ Complete |

---

## Phase 1 — Minimal Viable Protocol

### 3.0 Identifiers (XGID)

*Status: complete*

This section is the normative home for **XGID**, the XGen Protocol's named type discipline for first-class identifiers. The expository long-form (taxonomy, construction details, worked examples, type-representation strategy) lives in `docs/xgen_appendix_j_en.md`. The architectural commitment is recorded in `DECISIONS.md` D-072; the field-name-vs-type composition rule that governs use sites is recorded in D-073. This section states the rules; the appendix explains them.

---

#### 3.0.1 Definition

An **XGID** is the canonical name and type discipline for a first-class identifier of a protocol object in XGen. The protocol recognises **six flavours** of XGID at v1, organised into two families by construction:

**Hash-anchored family**

- `EventXgid` — a single Event in the DAG
- `SpaceXgid` — a Space (top-level container)
- `RoomXgid` — a Room (nested under a Space)
- `TrustAssertionXgid` — a Trust Assertion record

**Principal family**

- `NodeXgid` — a Node (server-side participant)
- `IdentityXgid` — an Identity (user-side principal)

Hash-anchored XGIDs are content-derived (the XGID is computed from the canonical-form bytes of the object via cryptographic hash). Principal XGIDs are key-derived (the XGID is computed from the Ed25519 public key of the principal). Both families share the same wire-format invariances (§3.0.3), the same immutability property (§3.0.2), and the same role-bearing field-name discipline (D-073).

The six flavours are exhaustive at v1. Sub-axes within a flavour (for example, an ephemeral session_id is a sub-axis of the Event flavour) are taxonomic refinements documented in Appendix J §J.7, not new top-level flavours. Adding a seventh flavour requires explicit promotion through a new DECISIONS.md entry.

---

#### 3.0.2 Immutability

**An XGID is immutable. Once issued, the binding from XGID to object is permanent. Properties of the object MAY change via subsequent events; the XGID does not.**

This property is structural, not policy. Hash-anchored XGIDs are immutable because the hash is content-derived from the founding object's canonical form, and that founding object is never modified — subsequent state changes are *new* events with *new* XGIDs. Principal XGIDs are immutable because the public key is the protocol-level identity, and the XGID is a bijective encoding of the public key. Both cases admit no operation that could re-bind an XGID to a different object.

Key rotation, when it becomes a feature of the protocol, will be expressed as the retirement of one principal XGID and the introduction of a new one, with cryptographic linkage event-recorded between them. No XGID is mutated.

See Appendix J §J.4 for the construction-derived explanation and §J.10 for worked examples of proposals this property rejects.

---

#### 3.0.3 Wire-format invariance

XGen Protocol guarantees five **wire-format invariances** for XGIDs across every boundary where they cross between processes. The invariances apply equally to the **federation wire** (Node-to-Node WebSocket messages) and the **AI control / batch JSONL wire** (the protocol-shaped surface between AI drivers or batch scripts and reference-implementation instances, documented in `docs/xgen_aicontrol_implementation.md` and Appendix F's batch reply schemas).

1. **Field names.** The JSON field name carrying an XGID does not change between v1 and any future retrofit pass. Renames require explicit protocol-version negotiation, not silent retrofit.
2. **Field types.** The on-wire JSON type for any XGID is `string`, regardless of which Rust newtype wraps it on the reference implementation side.
3. **Canonical form.** The string contents of any XGID are byte-identical when produced from the same inputs anywhere in the federation. No normalisation, no case-folding, no whitespace tolerance.
4. **URI grammar.** The structural shape of XGID strings (prefix, separator characters, length characteristics, character class) is fixed at v1. Retrofit work at the Rust-type-system level does not alter URI grammar.
5. **String-equality semantics.** Two XGIDs are equal iff their string contents are equal. Bytes equal bytes; no flavour-aware comparison, no normalisation hooks.

These invariances apply at v1 and through every Retrofit Pass that lands under D-072's adoption discipline. They do not foreclose future protocol versions making different choices — but those would be explicit version bumps with explicit migration paths, not silent changes.

See Appendix J §J.5 for the full reasoning and §J.9 for worked examples of proposals these invariances reject.

---

#### 3.0.4 Field-name-vs-type discipline

Every field that carries an XGID obeys the composition rule recorded in `DECISIONS.md` D-073:

> **The field name carries the role; the type carries the contract.**

The field name identifies *what role this particular XGID plays at this use site* (`introducer_node_id`, `peer_node_id`, `sender`, `room_id`, `delegated_to_identity`). The type identifies *what kind of XGID this field can ever hold* (`NodeXgid`, `IdentityXgid`, `RoomXgid`). Both pieces of information are load-bearing: a field name without type discipline loses contract enforcement; a type without role-bearing field name loses self-documentation at the use site.

The principle applies to Rust struct fields, function parameters, trace event fields, and JSON wire fields alike. JSON wire readers see strings (by invariance 2), but the surrounding field name still names the role.

See Appendix J §J.1 for the rule's relationship to the XGID type vocabulary, and the originating precedent at `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` §5.6 (`introducer_node_id: NodeXgid`).

---

#### 3.0.5 Scope boundaries

The following are explicitly **not XGIDs** at v1, recorded here to prevent miscategorisation:

- **Wire-envelope correlation handles.** `TransportMessage.event_id: Option<String>` is a transport-layer correlation field. By construction its string value is byte-equal to the corresponding Event XGID when populated, but it is type-level distinct from `EventXgid` and serves a different purpose (signal correlation, not protocol-object identification).
- **Error codes.** Numeric or string-tagged error codes (`4002`, `4006`, `4007`, etc.) are a separate identifier space.
- **Config field names.** Configuration keys like `[sync].batch_size` are not XGIDs.
- **File paths, log line tokens, debug formatters.** XGID types may appear in these surfaces via `Display` or `Debug`, but the surfaces themselves are not XGIDs.
- **Bootstrap discovery URIs.** Operational network addresses (e.g. `wss://bootstrap.example.org/`) route to Nodes; they are not protocol-object identifiers. The Nodes themselves have `NodeXgid` identifiers.

See Appendix J §J.8 for the full enumeration of boundary cases and the reasoning behind each.

---

#### 3.0.6 Adoption discipline

XGID Adoption v1 (D-072) ships the type vocabulary, the Rust reference implementation in `xgen-common`, and the wire-format invariance promise. Existing String-typed XGID fields across the codebase are retyped via five subsystem-scoped **Retrofit Passes** that land in ROADMAP.md Near future immediately after v1 ships:

- **Pass 1** — `xgen-common` core types
- **Pass 2** — `xgen-core` validation and dispatch surfaces
- **Pass 3** — `xgen-node` federation and fan-out surfaces
- **Pass 4** — `xgen-client` operational and AI-control surfaces, closing the AI control / batch JSONL documentation surface
- **Pass 5** — test fixtures, helpers, trace events, remaining surfaces

During the transition, the codebase MAY carry mixed discipline; every **new** field, function signature, and trace event field MUST use XGID types from v1 onward. After Pass 5 closes, mixed discipline ends.

This discipline does not affect the wire-format invariances of §3.0.3 — wire format is fixed at v1 regardless of how the Rust types evolve underneath it.

See Appendix J §J.11 for the full reasoning and D-072 for the architectural commitment.

---

### 3.1 Wire Format

*Status: complete*

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

All field names in XGen protocol messages use `snake_case` — lowercase letters, digits, and underscores only. No camelCase, no PascalCase, no hyphens. This convention applies uniformly to all protocol fields, `meta_atts` keys, and all field names in Auth Module message schemas.

Field names MUST be stable across protocol versions. A field name, once published in a released version of the spec, is permanent. Renaming a field is a breaking change and requires a new field name alongside the old one under a deprecation policy, not a silent replacement.

Implementations that encounter unknown field names MUST ignore them silently and MUST NOT reject the message on that basis alone. This is the forward-compatibility rule: new fields added in later protocol versions do not break older implementations.

**meta_atts key namespace rules**

The `meta_atts` field is an extensible key-value map present on all Events and certain other protocol objects. Keys in `meta_atts` follow a dot-separated namespace scheme:

```
<namespace>.<key>
```

Namespace ownership rules:

- The `xgen.` namespace is **reserved** for XGen Protocol specification use. No third-party key may begin with `xgen.`. Examples: `xgen.client`, `xgen.thread_id`, `xgen.tags`.
- Third-party and application-defined keys MUST use a **reverse-domain prefix** to avoid collisions. Examples: `com.example.priority`, `org.myapp.color`, `io.company.workflow_id`.
- Keys MUST use only lowercase letters, digits, underscores, and dots. No uppercase, no hyphens.
- Key segments (the parts between dots) follow `snake_case`. Example: `com.example.my_custom_field`, not `com.example.myCustomField`.
- The maximum key length is 128 characters.
- Values are strings. Structured values MUST be JSON-encoded as a string, not embedded as nested objects.

A receiving Node MUST ignore unknown `meta_atts` keys silently. Keys in the `xgen.` namespace that the Node does not recognise are treated as forward-compatible extensions and stored as opaque data.

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

> **Phase 1 note:** In Phase 1 protocol messages, `xgen_uri` does not appear as a
> standalone field type. Identity IDs and Node IDs use `pubkey_uri` directly.
> Space IDs, Room IDs, and Event IDs use `hash_uri` directly. The `xgen_uri` wrapper
> form (`xgen://identity/...`, `xgen://space/...`, etc.) is reserved for Phase 2
> contexts such as resource addressing in REST-style management APIs and Bootstrap
> Node directories. Phase 1 implementers do not need to parse or produce `xgen_uri`
> values — only `hash_uri` and `pubkey_uri` appear in Phase 1 wire fields.

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

#### 3.1.11 Reference Implementation Binary Names

The XGen reference implementation produces two executable binaries. Their canonical names are defined here so that documentation, scripts, deployment guides, and community tooling all use a consistent identifier.

| Binary | Produced by | Role |
|---|---|---|
| `xgen-node` | `xgen-node` crate | XGen Node — accepts client and peer connections, stores the Event log, handles federation |
| `xgen-client` | `xgen-client` crate | XGen reference client — registers Identities, creates Spaces and Rooms, sends and receives messages |

On Windows, the OS appends `.exe` — `xgen-node.exe` and `xgen-client.exe`. The canonical names are the names without the platform extension.

Third-party implementations are not required to use these names. The binary names are a reference implementation convention, not a protocol requirement. A compliant Node implemented in Go may be distributed as any executable name its authors choose — it is compliant because it implements the protocol correctly, not because of what the file is called.

---

### 3.2 Event Specification

*Status: complete*

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
| `prev_events` | array of hash_uri | yes | IDs of the Events this Event causally follows. MUST contain at least one entry except for `state.room_create`, where this MUST be an empty array `[]` — it is the DAG root (3.2.5) |
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
| `message.edit` | Edit of a prior message — references original via `original_event_id`; UI renders latest version in place with "edited" marker and history accessible on click (Ch6 6.7) |
| `message.delete` | Deletion/redaction of a prior message — references original via `original_event_id`; UI renders placeholder preserving timeline position; original content remains in DAG (Ch6 6.7) |

*State events* — define current Room or Space state. Multiple state events of the same type resolve to the most recent valid one. State Resolution algorithm is Phase 2:

| EventType | Description |
|---|---|
| `state.room_create` | Room creation — first Event in a Room's DAG, no `prev_events` |
| `state.room_name` | Sets or updates the Room display name |
| `state.room_topic` | Sets or updates the Room topic |
| `state.room_avatar` | Sets the Room avatar (URI reference) |
| `state.space_create` | Space creation — root Event for a Space, establishes auth_tier and home_node |
| `state.dm_space_create` | DM Space creation — two-member variant of Space, auto-creates one Room |
| `state.node_priority` | Space owner declares manual ordering of federated Nodes for conflict resolution |

*Federation events* — record federation relationship changes in a Space's DAG:

| EventType | Description |
|---|---|
| `state.federation_add` | Records that a new Node has joined the federation for this Space |
| `state.federation_remove` | Records that a Node has left or been removed from federation for this Space |

*Membership events* — record Identity membership transitions in a Space.
Room membership in Phase 1 is derived from Space membership — a Space member
has access to all Rooms in that Space (see 3.7.9). Private Rooms with
independent membership are Phase 2:

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

*Status: complete*

The network transport layer between clients and Nodes, and between Nodes. Two distinct connection types exist — client→Node and Node→Node — each with different trust assumptions. Both use the same underlying transport and framing mechanism.

---

#### 3.3.1 Transport Layer

XGen uses WebSocket (RFC 6455) as the mandatory transport for all connections. WebSocket was chosen for three reasons: it is bidirectional and long-lived, eliminating the overhead of repeated connection establishment; it operates over standard HTTP/HTTPS infrastructure, making it compatible with firewalls, proxies, and load balancers; and it is universally supported across all target implementation languages and environments.

**Transport pluggability**

WebSocket over TLS is the standard and mandatory transport for production deployments. However, the XGen protocol does not prevent operators from substituting any reliable bidirectional stream transport that can carry the same framed messages. A Node MAY run over alternative stream transports — including Tor hidden services, I2P tunnels, or pluggable transport proxies — without any changes to the protocol layer above the transport. Authentication, federation handshake, and Event validation operate identically regardless of the underlying stream. Node Announcements (3.5) may declare non-standard endpoint URIs where applicable.

This pluggability is a deliberate design choice: XGen should remain accessible in network environments where standard WebSocket traffic is restricted or monitored. The protocol makes no assumptions about transport-layer observability. Deep-packet-inspection resistance via custom transports is a Phase 3 area of investigation.

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

**Keepalive as the complete session model**

XGen does not implement a separate inactivity timeout. Authentication in XGen is
stateless on the Node side — the challenge-response at connection time (3.3.4) proves
the client holds the private key, but the Node maintains no session token and no
session table. There is nothing to "expire".

The keepalive mechanism above IS the session health model. If the WebSocket connection
drops — due to network failure, device sleep, or any other cause — the Node detects
this via the missed pong and closes its end. The client detects the dropped connection
and reconnects using the backoff sequence (3.3.6). Re-authentication is instant (a
single challenge-response round trip). From the user's perspective, the client
reconnects transparently in the background — there is no "logged out" state.

Implementers MUST NOT add a separate inactivity timer that closes the connection or
requires the user to re-enter credentials. The correct model is: connection alive =
authenticated. Connection dropped = reconnect and re-authenticate automatically.

The only valid reason to present a credential prompt to the user is if the encrypted
key file cannot be decrypted on startup (wrong passphrase or missing file). That is a
key access failure, not a session expiry.

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
  "space_id": "xgen://hash/sha256:c3d4e5f6...",
  "room_id": "xgen://hash/sha256:b2c3d4e5...",
  "last_event_id": "xgen://hash/sha256:a3f9b2c1..."
}
```

The `space_id` field identifies which Space's Event store to query.
The `room_id` field identifies which Room within that Space to sync.
The `last_event_id` field is the Event ID the client last received —
the Node returns all Events that causally follow it in the Room's DAG.
If the client has no prior Events for a Room (first join or fresh install),
it omits `last_event_id` and the Node sends the full Room history from the
DAG root, subject to any history limits declared by the Space.

**`transport.sync_response`** — the Node sends all Events that follow `last_event_id` in causal order (parents before children) as individual Event messages on the active connection. The response set is bounded by the Node's `[sync].batch_size` limit (default `1000` events per response cycle); when the requester's outstanding range exceeds this, the Node paginates and emits a `continue_from` cursor in the terminator.

**`transport.sync_complete`** — the Node sends this message after the last Event of a sync batch to mark the end of the response. The terminator is cross-Space whole-batch: a single `sync_request` may span multiple Spaces the requester participates in, and one `sync_complete` covers the entire batch (Federation Event Propagation milestone, F-6 + F-7, runbook §3.3.1 Lock 5).

```json
{
  "protocol_version": "0.1",
  "type": "transport.sync_complete",
  "since": "xgen://hash/sha256:a3f9b2c1...",
  "new_tip": "xgen://hash/sha256:f7e8d9c0...",
  "continue_from": "xgen://hash/sha256:c4d5e6f7..."
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `protocol_version` | string | yes | Protocol version negotiated for this session — `"0.1"` for Phase 1 |
| `since` | hash_uri | yes | Echoes the request's `last_event_id` (empty string if the request had none) — disambiguates concurrent in-flight sync_requests |
| `new_tip` | hash_uri | yes | Best-effort: the last `event_id` emitted in this batch, across all Spaces. Empty string when the delta was fully empty. Receivers MUST NOT compare it to a single-Space tip; per-Space tips are tracked through event ingestion |
| `continue_from` | hash_uri \| null | omittable | If present and non-null: the requester should issue a follow-up `sync_request` with `last_event_id` set to this value to retrieve the next page. If null or absent: the response is complete for this batch |

If there are no missed Events (the requester is already up to date), the Node sends `transport.sync_complete` immediately with `new_tip: ""` and no `continue_from`.

If `last_event_id` is unknown to the Node (the referenced Event is not in its log), the Node sends the full history from the DAG root for each Space in scope, subject to any Space history limits. This handles the case where a requester's state is too stale to anchor.

The completion-signal semantic replaces the pre-Federation-Event-Propagation quiet-time-elapsed heuristic. Requesters wait for `sync_complete` as the explicit end-of-batch marker, with `[sync].completion_timeout_seconds` (default `5`) as the safety-net upper bound (F-6b). The cross-Space whole-batch design lets multi-Space requesters wait once rather than per-Space.

**Configuration.** Both Node and Client configs surface a `[sync]` section:

```toml
[sync]
completion_timeout_seconds = 5
batch_size = 1000
```

Implementation reference: `xgen-core/src/wire/types.rs` `TransportMessage::SyncComplete`. Federation Event Propagation milestone Phase 1 shipped this wire shape; see `docs/xgen_federation_propagation_design.md` §9 (F-6) and §10 (F-7) for the design rationale.

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

#### 3.3.10 Event Acceptance Signal

After an inbound DAG Event clears the full validation pipeline (§3.7) AND is durably written to the Node's event store — and before local fan-out begins — the Node sends the originator a positive acceptance signal:

```json
{
  "protocol_version": "0.1",
  "type": "transport.event_accepted",
  "event_id": "xgen://hash/sha256:...",
  "accepted_at": "2026-05-29T12:00:00.000Z"
}
```

`transport.event_accepted` is the wire-level sibling of `transport.error`: acceptance and rejection are two signals of equal importance, opposite direction. It is sent only to the originator's connection and does not propagate (the accepted Event itself propagates via fan-out and federation). On receipt with `event_id` equal to a submitted Event's `event_id`, the originator MAY claim the Event is in the home Node's authoritative DAG store — validated and persisted — but MUST NOT claim other members or federation peers have it yet (those are asynchronous downstream concerns). A Node MUST NOT send `transport.event_accepted` before the durable write completes.

When an Event submission is rejected instead, the Node sends `transport.error` carrying the rejected Event's `event_id` so the originator can correlate the rejection to its in-flight submission. The `event_id` field is OPTIONAL on `transport.error`: present when the error pertains to a specific Event, absent for transport-level errors not tied to an Event (e.g. malformed framing). This shared correlation field lets a client with multiple in-flight submissions match each `transport.event_accepted` or `transport.error` to its originating Event.

*(Reference implementation: the Node admin write path milestone, M6 — `docs/xgen_node_admin_ops_design.md` §3.)*

---

### 3.4 Federation Handshake

*Status: complete*

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

*Status: complete*

How a Node establishes, announces, and proves its identity on the network. A Node's identity is derived directly from its keypair — no registration authority, no certificate chain, no external validation. The keypair IS the identity, consistent with XGen's identity-first model throughout.

---

#### 3.5.1 Node Keypair Generation

On first run, a Node generates an Ed25519 keypair. This keypair is the Node's permanent identity for Phase 1. The public key becomes the Node ID. The private key never leaves the Node — it is used only to sign Node announcements and to authenticate transport connections (3.3.4).

The private key MUST be stored encrypted at rest using a strong symmetric cipher. The encryption key MUST be derived from a secret known only to the Node operator — not hardcoded, not stored in the same location as the encrypted key. The specific encryption mechanism is implementation-defined; the spec requires only that the private key is not stored in plaintext.

**Key file location is configurable.** The encrypted private key file does not need to reside in the Node's application folder. The operator declares the key file path in `node_config.json` via the `keypair_path` field. Valid locations include a dedicated secure local folder, a cloud-synced location (Google Drive, OneDrive), a network share, or a hardware security module. If `keypair_path` is absent from config, the Node defaults to looking for the key file alongside the executable. The key file is always encrypted at rest — storing it on cloud storage is safe because without the decryption passphrase it is useless to any party that obtains it.

**Pattern A exception taxonomy**

XGen follows a folder-is-the-application deployment model (see `IMPLEMENTATION_GUIDE_ph1.md`). Key files are the primary exception. Two categories of exception exist and are defined here permanently:

*Structural exceptions — physically cannot live in the application folder:*

| Exception | Notes |
|---|---|
| Cryptographic key files | `keypair_path` config field — cloud storage, network share, or HSM |
| Hardware Security Module | Physical device — key never touches the filesystem |
| OS keystore | Windows Credential Manager, macOS Keychain — Phase 2, platform-specific |
| Tauri webview cache | Phase 2 — WebView2/WebKit manages its own storage location |

*Operational exceptions — default to application folder, may be routed elsewhere by operator:*

| Exception | Notes |
|---|---|
| TLS certificates | May use system-managed certs (certbot, nginx, OS store) |
| Log output | May route to syslog, Windows Event Log, or centralised aggregator |
| Shared Identity registry | HA deployments with primary/standby Nodes — network share or database |

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
  "capabilities": ["json", "msgpack", "xgen.federation"],
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
| `bootstrap_info` | object | no | Present only on Bootstrap Nodes (capability `xgen.bootstrap`). Contains `directory_url` (string), `accepts_registrations` (boolean), `region` (string), `operator` (string). See 3.14.1 for full schema. |

**Canonical form for signing**

The canonical form excludes `signature` and follows the same rules as Event canonicalisation (3.2.4): no whitespace, keys sorted within objects, UTF-8 encoding. Field order: `protocol_version`, `type`, `node_id`, `endpoint`, `capabilities`, `auth_tiers_served`, `operator_display_name` (if present), `announcement_version`, `valid_until`, `timestamp`, `bootstrap_info` (if present).

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

*Status: complete*

How a user creates an Identity and registers it with a Node. An Identity is the user's permanent presence on the XGen network — derived from a keypair, self-certifying, and independent of any specific Node. Registration is the process of making that Identity known to a Node so it can send and receive Events.

---

#### 3.6.1 Client-Side Keypair Generation

The client generates an Ed25519 keypair on the user's device before registration begins. The keypair is generated locally — it never leaves the device in plaintext. The public key becomes the Identity ID. The private key is used to sign Events and to authenticate transport connections.

The private key MUST be stored encrypted at rest on the client device. The encryption mechanism is implementation-defined and platform-appropriate — a mobile client may use the device's secure enclave; a desktop client may use an OS keychain or encrypted file. The spec requires only that the private key is not stored in plaintext.

**Key file location is configurable.** The encrypted private key file does not need to reside in the client's application folder. The user declares the key file path in `client_config.json` via the `keypair_path` field. Cloud storage (Google Drive, OneDrive) is explicitly supported and encouraged — it allows the user to access their Identity key from multiple machines before Phase 2 multi-device support is built. The key file is always encrypted at rest — storing it on cloud storage is safe because without the decryption passphrase it is useless to any party that obtains it.

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
  "is_ai": false,
  "ai_capabilities": null,
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
| `is_ai` | boolean | no | Declares whether this Identity represents an AI agent. Defaults to `false` (human). **Immutable after registration** — see 3.6.10. |
| `ai_capabilities` | object | conditional | Required when `is_ai = true`; MUST be `null` or omitted when `is_ai = false`. Open-enum map of capability flags governing AI behaviour — see 3.6.10 for the Phase 2 set. |
| `trust_assertion` | object | conditional | Required for Tier 1+ registration. Omitted for Local Node mode only |
| `re_registration` | boolean | no | Set to `true` when re-registering an orphaned Identity on a new home Node (3.13.8). Omit or set to `false` for initial registration. When `true`, the Node permits registration of an `identity_id` that is already known (from a prior replica record) without treating it as a duplicate. |
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
| 8 | `is_ai` / `ai_capabilities` shape is consistent (3.6.10): if `is_ai = true`, `ai_capabilities` MUST be a non-null object containing all Phase 2 required capability keys; if `is_ai = false`, `ai_capabilities` MUST be `null` or absent | Reject — ai_declaration_invalid |
| 9 | Node has capacity to accept new Identities | Reject — node_capacity_exceeded |

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
  "is_ai": false,
  "ai_capabilities": null,
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

The `is_ai` and `ai_capabilities` fields are recorded as supplied in the registration request. Both are part of the Identity record and are subject to the same replication and update propagation as the rest of the record. The `is_ai` field is **immutable after registration** — see 3.6.10. The `ai_capabilities` map MAY be updated within the constraints defined in 3.6.10.

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

#### 3.6.10 AI Identity Extension

*Status: complete (Phase 2)*

An AI agent participates in XGen as a first-class Identity with the same structural shape as a human Identity — one keypair, one `identity_id`, one persistent record, one set of memberships, one DM relationship model. AI Identities differ from human Identities in two ways: they declare `is_ai = true` at registration, and they carry a set of capability flags that constrain their behaviour. Everything else is identical.

The design principle is that AI is not a separate kind of actor in the protocol — it is a kind of Identity with declared asymmetric rules. The protocol concept of an Identity (signing keypair, persistent accountable presence, member of Spaces) does not change; only the rule set that applies to a given Identity changes based on `is_ai`.

**Cross-references:**
- **D-059** (AI users as first-class XGen Identities with declared capabilities) — the decision narrative for this section.
- **D-064** (M3 AI operator role: distinct role, fall-upward resolution, AI-owned-Space prohibition) — extends 3.6.10.6 below.
- **D-065** (M4 AI Client reference implementation) — names the recurring "honest behaviour over polite behaviour" principle this section's enforcement model implicitly relies on (capability checks reject violations; they don't queue them for later).
- **Ch6 §6.15** (AI Client resident mode) — the client-side implementation that consumes this section's protocol surface. Read 3.6.10 for the protocol semantics; read Ch6 §6.15 for how a reference AI Client is built on top.

##### 3.6.10.1 Registration

An Identity declared as `is_ai = true` is created via the same `identity.register` flow as a human Identity (3.6.3). The two added fields:

- `is_ai: boolean` — declared at registration; `true` marks the Identity as an AI agent, `false` (default) marks it as human.
- `ai_capabilities: object` — required when `is_ai = true`; an open-enum map of capability flags. Required when `is_ai = false` to be `null` or absent.

The Node enforces shape consistency in the §3.6.4 acceptance pipeline (step 8). A registration with `is_ai = true` and a missing, null, or invalid `ai_capabilities` map MUST be rejected with error `3040 ai_declaration_invalid`. A registration with `is_ai = false` and a non-null `ai_capabilities` map MUST be rejected with the same error.

The Trust Assertion requirement is unchanged. An AI Identity is verified to a Tier by the same Auth Module mechanism as a human Identity (3.8 for Tier 1; 3.11 for Tiers 2–4). What counts as "verification" for an AI is the operator's institutional credentials; the protocol does not interpret the Trust Assertion contents differently for AI vs human.

##### 3.6.10.2 Immutability of the AI declaration

The `is_ai` field is **immutable after registration**. The Node MUST reject any `identity.update` (3.6.8) message whose `changes` object includes the `is_ai` key, with error `3041 ai_flag_immutable`.

This immutability is structural, not policy. An Identity cannot be re-classified between AI and human after registration because the asymmetric rules in 3.6.10.4 are bound to the declaration at creation. Allowing the flag to change would mean a human Identity could accumulate trust and then re-classify as AI to escape human-only restrictions, or vice versa. The cryptographic anchor of the keypair (D-037, persistent accountable identity) is matched by the structural anchor of the AI declaration.

An operator who needs to change the classification creates a new Identity. The new Identity is a different accountable actor with no shared history with the old one.

##### 3.6.10.3 Capability flag set (Phase 2)

The `ai_capabilities` map is an open-enum structure. Phase 2 defines a minimum required set; the structure is open so future phases may add capability keys without breaking older Nodes (an older Node that does not recognise a capability key MUST ignore that key and not reject the Identity).

**Phase 2 required capability keys:**

| Key | Type | Default | Description |
|---|---|---|---|
| `dm_initiate` | boolean | `false` | When `false`, the AI Identity MUST NOT create a new DM Space (3.7.4). When `true`, no restriction beyond standard human-equivalent rules. |
| `spontaneous_post` | boolean | `false` | When `false`, the AI Identity SHOULD NOT post in a Room without being addressed by a human member; enforcement of this is governed by the per-Room permission set defined in Ch6 (it is a soft client-side and admin-policy mechanism, not a Node validation rule in Phase 2). |

All Phase 2 required capability keys MUST be present in the `ai_capabilities` map at registration. A registration that omits any required key MUST be rejected with error `3040 ai_declaration_invalid`.

**Reserved capability namespace:** capability keys MUST follow the same naming rules as `meta_atts` keys (3.1.3). The `ai.*` prefix is reserved for protocol-defined capabilities. Third-party operators MAY add reverse-domain-prefixed capability keys (e.g. `com.example.ai_module_feature: true`) for their own purposes; Nodes that do not recognise such keys ignore them.

##### 3.6.10.4 Enforcement model

Capability flag enforcement is **protocol-level and hard**. A Node that receives an Event signed by an `is_ai = true` Identity MUST check the Event type against the AI's declared capabilities and reject Events that violate the declared restrictions.

**Phase 2 enforcement points:**

| Event type | Capability required | Violation error |
|---|---|---|
| `state.dm_space_create` (3.7.4) where sender `is_ai = true` | `dm_initiate = true` | `3042 ai_capability_violation` — "dm_initiate disallowed" |

The `spontaneous_post` capability is **not** Node-validated in Phase 2 — it is a client-side and admin-policy concern, surfaced in Ch6. A future phase may promote it to Node validation if enforcement experience justifies the cost.

**Note on the `dm_initiate` rule.** The restriction is on *creating* a DM Space, not on *sending into* one. An AI Identity may freely participate in a DM Space that another Identity has created and invited the AI into. This includes posting reminders, follow-up messages, and scheduled check-ins inside an already-established DM relationship. The protocol-level rule prevents the AI from initiating a new private channel with a human; it does not silence the AI in channels the human has already opened.

##### 3.6.10.5 Capability updates

The `ai_capabilities` map MAY be updated via `identity.update` (3.6.8) by the Identity itself. The update applies to all future Events signed by the Identity; Events already in the DAG are validated against the capabilities in effect at the time of their inclusion (capabilities are not retroactively applied).

A capability update follows the standard `identity.update` semantics with a monotonic `update_version`. Replica Nodes receive the updated capabilities through standard identity replication (3.13.5).

Phase 2 imposes no policy restriction on which capabilities may be flipped or by whom. The Identity holds its own private key and may modify its own record. The accountability rests on the AI's operator and the public visibility of the declared capabilities; a community that does not trust a particular AI's stated capabilities may decline to invite it (3.6.10.6) or remove it (3.6.10.7).

##### 3.6.10.6 AI operator role and accountability

An AI Identity does not appear in a Space by coincidence. It is invited via `membership.invite` (3.7.8) by a Space owner or admin, like any other member. The inviter is recorded permanently in the DAG and on the resulting `SpaceMember`. If the AI subsequently misbehaves, the inviter is on record as the Identity that authorised the AI's presence.

**Operator role.** Within each Space that contains an AI member, exactly one Identity resolves as that AI's **operator** at any moment. Operator is a distinct role — its scope is "responsibility for this specific AI", not Space-wide privileges (which remain admin's and owner's). Operator is per-(AI, Space): the same Identity may be operator for AI-X in Space S, a plain member in Space T, and an admin in Space U.

**AI-owned Space prohibition.** An AI Identity MUST NOT be a Space owner. Nodes MUST reject `state.space_create` and `state.dm_space_create` whose sender is registered with `is_ai = true`, with error code 3041 (`ai_role_violation`). This is a structural rule, not a capability-flag check — it fires before, and supersedes, the `dm_initiate` capability gate (3.6.10.4) for any AI sender of a Space-creation Event.

**Delegation and revocation.** The operator concept is exposed via two protocol Events:

| EventType | Purpose | Valid signer | Body fields |
|---|---|---|---|
| `state.ai_operator_delegate` | Record a delegation of the operator role for `ai_identity_id` to `new_operator_identity_id` within `space_id`. | Space owner OR admin. | `space_id`, `ai_identity_id`, `new_operator_identity_id`. |
| `state.ai_operator_revoke` | Clear the stored delegation for `ai_identity_id` within `space_id`. Resolution falls through to the inviter (then to the owner). | Space owner OR admin. | `space_id`, `ai_identity_id`. |

Both Events are Space-scoped — they apply only within the Space identified by `space_id`. The same AI may have different resolved operators in different Spaces. The previous operator's consent is **not** required for delegation; in this protocol version, operator assignment is entirely under admin/owner authority.

Node-side validation rules for both EventTypes:

1. Signer's role in the Space MUST be owner or admin (otherwise reject with 3041).
2. `ai_identity_id` MUST be a current Space member (otherwise reject with 3041).
3. `ai_identity_id` MUST resolve to an Identity record with `is_ai = true` (otherwise reject with 3041).
4. For `state.ai_operator_delegate`: `new_operator_identity_id` MUST be a current Space member (otherwise reject with 3041).

**Fall-upward resolution algorithm.** The current operator for an AI Identity in a Space is a *resolved value*, not a stored one. On query, the Node walks upward through stored state until it finds a live Identity:

```
resolve_operator(space, ai_id) -> identity_id:
    1. If a stored delegation exists for ai_id AND the named delegate is a
       current Space member: return the delegate.
    2. Else if the AI's recorded inviter (the sender of the original
       membership.invite Event admitting the AI) is a current Space member:
       return the inviter.
    3. Else: return the Space owner.
```

The two-step fallback ensures **no orphan state is reachable**. The owner is always a member of a live Space (Spaces with no owner are abandoned). The stored delegation may name an Identity who has since left or been kicked — the resolution function transparently skips past such records, so an explicit revoke is not required when the delegate disappears. Conversely, `state.ai_operator_revoke` explicitly clears the stored delegation, collapsing resolution to step 2 or 3.

**Inviter-as-operator is an output of resolution, not stored state.** There is no separate "initial operator" record. When an AI joins a Space and no delegation has yet been signed, the resolution function returns the inviter — identical to how the operator is resolved at any other time.

**No protocol-enforced operator privileges.** This protocol version records who the operator is and provides the resolution function. It does **not** confer protocol-level privileges on the operator: the operator does not gain a new event-signing capability, cannot act on behalf of the AI, and cannot block admin/owner actions against the AI. The AI continues to sign its own Events; admin/owner retains the ability to mute, kick, or ban the AI through the standard `membership.*` Events (3.7.8) regardless of the operator's identity. Practical operator privileges (a DM command surface for the operator to instruct the AI, audit access, capability override, etc.) emerge in later protocol versions as features need them, layered on the resolution function from this section.

##### 3.6.10.7 Removal

An AI Identity is removed from a Space by the standard `membership.kick` or `membership.ban` Events (3.7.8). There is no AI-specific removal mechanism. Any Identity holding admin or owner rights in the Space may issue the kick or ban. Moderators may issue `membership.mute` (3.7.8). The AI's operator does not have special protection — a Space admin who is not the operator may remove the AI without operator consent, particularly in cases where the AI's behaviour is causing disturbance and the operator is unreachable.

##### 3.6.10.8 Tier inheritance

An AI Identity holds a Trust Assertion at a specific Tier (3.6.3). To be invited into a Space with a Tier requirement, the AI's Tier MUST meet or exceed the Space's requirement — identical rule to human Identities. There is no separate Tier dimension for AI Identities.

The practical implication: an AI in a Tier 4 healthcare Space is a Tier 4 entity. What that means for an AI — what institutional verification an Auth Module performs to issue a Tier 4 assertion to an AI Identity — is the Auth Module's domain (3.11). The protocol concept of Tier is unchanged; the verification procedure that produces a Tier N assertion for an AI is an Auth Module concern.

##### 3.6.10.9 Replication

The `is_ai` and `ai_capabilities` fields are part of the Identity record (3.6.6) and are replicated to replica Nodes through the standard identity replication mechanism (3.13). No additional wire format is required — the `identity_record` payload in `identity.replicate` (3.13.4) carries these fields as part of the full record. A replica Node that receives an Identity record with `is_ai = true` and a properly shaped `ai_capabilities` map MUST store and enforce them on Events sourced from that Identity, identically to the home Node's enforcement.

##### 3.6.10.10 Error codes

| Code | Name | Condition |
|---|---|---|
| `3040` | `ai_declaration_invalid` | Registration: `is_ai` and `ai_capabilities` shapes inconsistent, or required capability keys missing |
| `3041` | `ai_role_violation` | Umbrella for structural AI role rules: `identity.update` attempted to change `is_ai`; an `is_ai = true` sender attempted `state.space_create` / `state.dm_space_create`; or a `state.ai_operator_delegate` / `state.ai_operator_revoke` failed signer-role / target-membership / `is_ai`-target validation (3.6.10.6) |
| `3042` | `ai_capability_violation` | An Event from an `is_ai = true` Identity violates a declared capability restriction (3.6.10.4) |

All three codes live in the existing identity domain (3000–3999, per CLAUDE.md error code convention).

##### 3.6.10.11 Phase 2 vs future phases

The Phase 2 capability set (`dm_initiate`, `spontaneous_post`) is deliberately minimal. The structural design — open-enum capability map, hard Node enforcement of declared restrictions, ignore-unknown for forward compatibility — is the durable contribution. Future phases may add capability keys (proactive moderation rights, cross-Space coordination, scheduled job execution, autonomous DM initiation under specified conditions) without changing the protocol's wire format or Node validation skeleton. Older Nodes that do not understand a new capability ignore it; newer Nodes enforce it.

The expected evolution: as AI agents in XGen mature and as institutional trust frameworks develop, capabilities that default to `false` in Phase 2 MAY be flipped to `true` for specific AI Identities under specific conditions. The protocol does not prescribe when or how. It provides the structure for the change to happen accountably.

---

### 3.7 Space & Room Protocol

*Status: complete*

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
| `human_pacing_ms` | set at creation; updatable via `state.space_pacing` | Minimum send interval for human members (3.7.12) |
| `ai_pacing_ms` | set at creation; updatable via `state.space_pacing` | Minimum send interval for AI members (3.7.12) |
| `member_temperature_visibility` | set at creation; updatable via `state.space_temperature_visibility` | Who sees per-member temperature values (3.7.13) |

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

**`membership.mute`** — sent by moderator, admin, or owner to silence a member for a bounded period without removing them from the Space. Introduced in Phase 2 to support AI-specific temperature escalation (3.7.12.8, Ch6 §6.12) and human moderation use cases that warrant temporary silence rather than removal:

```json
{
  "type": "membership.mute",
  "content": {
    "target_identity": "xgen://pubkey/ed25519:MEMBER_KEY...",
    "reason": "Disturbing the room rhythm",
    "cooldown_until": "2026-05-15T14:00:00.000Z"
  }
}
```

A muted member retains Space and Room membership, retains visibility into ongoing conversation, retains DM threads, and retains all stored context. The mute prevents the member from posting `message.*` Events into Rooms within the Space until `cooldown_until` is reached. The mute is automatically lifted at `cooldown_until` — no explicit `membership.unmute` Event is required to end a time-bound mute. A separate explicit `membership.unmute` Event MAY be sent by an admin to end the mute early.

**Standard reason values.** The `reason` field on `membership.kick`, `membership.ban`, and `membership.mute` is a free-text string by default. The following reason values are reserved by the protocol and have defined semantics:

| Reason value | Used on | Meaning |
|---|---|---|
| `auto_temperature` | `membership.kick` (human), `membership.mute` (AI) | The action was issued automatically by a client implementing the temperature mechanism (Ch6 §6.12) in response to sustained pacing violations. The `cooldown_until` field SHOULD accompany `auto_temperature` reasons to indicate the temperature mechanism's recommended re-entry time. |

Reserved reason values are an open-enum extension point — future automated moderation mechanisms may add reason values; clients and Nodes that do not recognise a reason value display it as free text without further interpretation.

**Note on Event signing for `auto_temperature` actions.** A `membership.kick` or `membership.mute` Event with `reason = auto_temperature` is signed by the Identity that issued it through standard signing rules. The signing Identity is the client (or Node) that observed the sustained pacing violation — typically a Space admin or moderator's automated client, or the room's home Node operating an automated moderation policy. The protocol does not specify who is permitted to issue `auto_temperature` actions; that is a Space governance choice surfaced in Ch6.

**Role permission table**

| Action | member | moderator | admin | owner |
|---|---|---|---|---|
| Send messages | ✅ | ✅ | ✅ | ✅ |
| Invite members | ❌ | ✅ | ✅ | ✅ |
| Mute members | ❌ | ✅ | ✅ | ✅ |
| Kick members | ❌ | ✅ | ✅ | ✅ |
| Ban members | ❌ | ❌ | ✅ | ✅ |
| Create Rooms | ❌ | ❌ | ✅ | ✅ |
| Change Space name/topic | ❌ | ❌ | ✅ | ✅ |
| Update Space pacing | ❌ | ❌ | ❌ | ✅ |
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

#### 3.7.12 Pacing Rules on Spaces

*Status: complete (Phase 2)*

Every Space carries two pacing rules in its state. These rules define minimum intervals between consecutive messages from a single member, distinguished by whether the member's Identity is human or AI. The pacing rules are a Space-level cultural setting — they shape the rhythm of conversation in the Space — enforced by participating clients as a condition of participation.

**Cross-reference:** D-060 (per-space pacing rules) is the decision narrative for this section. D-061 (room temperature mechanism) extends pacing into a dynamic moderation feedback signal and is specified in Ch6 §6.12.

##### 3.7.12.1 Fields

The Space state (3.7.6) carries two pacing fields:

- `human_pacing_ms: integer` — minimum interval in milliseconds between consecutive Events of type `message.*` or `state.*` from a single member whose Identity has `is_ai = false`.
- `ai_pacing_ms: integer` — minimum interval in milliseconds between consecutive Events of type `message.*` or `state.*` from a single member whose Identity has `is_ai = true`.

Both fields are integers, MUST be non-negative, and MAY be zero (zero disables pacing for that member class in the Space).

##### 3.7.12.2 Defaults

At Space creation (`state.space_create`), the Space owner MAY specify `human_pacing_ms` and `ai_pacing_ms` values. If either is omitted, the Node fills in the protocol-recommended Phase 2 default:

| Field | Default |
|---|---|
| `human_pacing_ms` | `500` |
| `ai_pacing_ms` | `2000` |

The defaults are conservative: 500 ms catches accidental rapid triple-posts without being noticeable for normal typing; 2000 ms gives human members time to read between AI messages and prevents an AI from monopolising attention in active discussion.

Space cultures may diverge widely. A contemplative Space may set `human_pacing_ms: 5000` and `ai_pacing_ms: 30000`. A fast-chat Space may set both to `0` (disabled). Both are legitimate.

##### 3.7.12.3 Updates

Pacing values MAY be updated by a Space owner via a new EventType `state.space_pacing`:

```json
{
  "type": "state.space_pacing",
  "content": {
    "human_pacing_ms": 1000,
    "ai_pacing_ms": 5000
  }
}
```

The Event is signed by the Space owner, applies to the Space identified by the Event's `space_id`, and supersedes prior pacing values from the moment it is included in the DAG. Both fields are required in the `content` object — partial updates are not supported (the operator sets both values explicitly each time).

Members MAY discover the current pacing values from the Space state at any time; clients SHOULD react to a pacing change without requiring reconnection.

##### 3.7.12.4 Authority and enforcement

The pacing rules are **Space rules**, on the same level of authority as the Space's `auth_tier` requirement, role-based permissions, and federation list. A client that wishes to participate in the Space MUST enforce these rules locally for its own outbound Events.

**Phase 2 enforcement model: client-side only.** The Node does not validate that incoming Events respect pacing. Bad-actor clients that attempt to violate pacing produce Events whose timestamps clearly show the violation; admins remove such members through standard `membership.kick` / `membership.ban` mechanisms (3.7.8), and the temperature mechanism (Ch6 §6.12) provides automated dynamic responses.

**Phase 3 deferred:** Node-side pacing validation MAY be added in a future phase if abuse patterns justify the additional Node cost. The decision is recorded as a Phase 3 open question; Phase 2 trusts clients for pacing the same way it trusts them to respect role permissions client-side before Node-side validation.

##### 3.7.12.5 Member classification at enforcement time

Which pacing value applies to a given member is determined by the member's Identity record at the time of the outbound Event:

- If `is_ai = false` (or absent): `human_pacing_ms` applies.
- If `is_ai = true`: `ai_pacing_ms` applies.

The `is_ai` value is immutable post-registration (3.6.10.2), so the pacing class for a member is stable for the lifetime of the Identity.

##### 3.7.12.6 Pacing scope

Pacing applies **per Space, per member**. Two simultaneous Spaces have independent pacing counters for the same member — a member moving quickly in Space A does not affect their pacing status in Space B. Within a Space, pacing applies across all Rooms uniformly — a member cannot circumvent a Space's `human_pacing_ms` by alternating between Rooms.

DM Spaces (3.7.4) MAY carry their own pacing values; the default rule (3.7.12.2) applies to DM Spaces just as to regular Spaces unless the creator specifies otherwise.

##### 3.7.12.7 Rigid enforcement for AI Identities

For an `is_ai = true` Identity, the pacing rule is **rigid**: the AI's client MUST queue outbound Events when sending would violate `ai_pacing_ms` and MUST NOT release them before the interval has elapsed. This is the AI equivalent of a hard tier requirement — a property of being a participant in the Space, not a recommendation.

For human Identities, the pacing rule is also a MUST, but client UI typically applies it as a silent throttle (queue the message briefly without alerting the user) since human typing rates rarely sustain pacing violations and silent throttling preserves a conversational experience. The behaviour distinction is described in Ch6.

##### 3.7.12.8 Interaction with temperature mechanism

The room temperature mechanism (Ch6 §6.12, D-061) consumes pacing overpass events as its primary input. A client that fully respects pacing (queues messages within the cap) produces zero pacing overpasses and contributes zero heat. A client that attempts to violate pacing produces overpass signals which feed the temperature counters, eventually triggering UI warnings and — at sustained violation — throttling or removal per the asymmetric escalation rules (kick for humans, mute for AI).

This means the pacing rule is enforced through two complementary mechanisms: the client's own queue (the immediate, cooperative enforcement) and the temperature mechanism (the community-visible, accountable enforcement for clients that fail or refuse to queue).

##### 3.7.12.9 EventType registry addition

| Type | Purpose | Phase |
|---|---|---|
| `state.space_pacing` | Updates the `human_pacing_ms` and `ai_pacing_ms` values for a Space. Signed by the Space owner. | Phase 2 |

---

#### 3.7.13 Temperature Property

*Status: complete (Phase 2)*

A Room carries a numeric **temperature** signal expressing the rhythm of its recent traffic. Two values are published per Room: a Room-level value reflecting the collective state, and a per-member value reflecting each individual member's accumulated overpass of the Space's pacing rules (3.7.12). The Room's home Node is the authoritative source for both; the protocol carries the values, the home Node's plugin chooses how to compute them.

**Cross-reference:** D-061 (room temperature: protocol carries the signal, plugin owns the math) is the decision narrative for this section. Ch6 §6.12 specifies the client-side rendering of the values defined here. The mathematical model that produces the values is intentionally outside the protocol and lives in a plugin on the Room's home Node — see D-061 for the reasoning.

##### 3.7.13.1 Reserved `meta_atts` keys

The `xgen.*` namespace (3.1.3) reserves two keys for temperature, both carrying float values in the closed range `[0.0, 1.0]`:

| Key | Subject | Description |
|---|---|---|
| `xgen.room_temperature` | Room | Collective temperature of the Room; visible to every member. |
| `xgen.member_temperature` | Member-in-Room | Individual temperature of one member in the Room; visibility governed by `member_temperature_visibility` (3.7.13.4). |

The values are floats serialised as JSON numbers. A value of `0.0` represents the cool baseline; `1.0` represents the maximum classified state (`fiery`, per Ch6 §6.12.2 default thresholds). Intermediate values are interpolated continuously by the home Node's plugin.

Both keys are optional. Absence of a key means the home Node is not publishing temperature for that subject. A Room MAY publish room temperature without member temperature, or vice versa. A Room whose home Node runs no temperature plugin publishes neither.

The Node MUST validate the value type (float) and range (`0.0 ≤ v ≤ 1.0`) on outgoing `meta_atts`. Out-of-range values are clamped to the range at the Node before transmission.

##### 3.7.13.2 Threshold table

The home Node publishes a threshold table once at room-open time as part of the Room metadata response (3.7.7 — the Node-to-client session message carrying Room state). The table declares which float values correspond to which named states:

```json
"temperature_thresholds": {
  "warm":  0.30,
  "hot":   0.55,
  "fiery": 0.80
}
```

All three fields are required when the table is present. Values MUST satisfy `0.0 < warm < hot < fiery ≤ 1.0`. The implicit `cool` state covers `[0.0, warm)`. A client receiving an invalid threshold table treats the table as absent and falls back to Ch6 defaults (§6.12.2).

The table is part of the Room metadata response, not a DAG event. It is transmitted at session open and re-transmitted by the Node when the underlying plugin configuration changes. Clients adopt the new table on receipt and re-derive any displayed bucket states.

The threshold table is **optional**. If the Node does not publish a table, clients apply the Ch6 default thresholds (§6.12.2). The default thresholds are protocol-recommended Ch6 defaults; they are not part of the protocol's required behaviour beyond falling back to them when no table is supplied.

##### 3.7.13.3 Visibility setting on Space state

A new field on Space state (3.7.6) controls who receives `xgen.member_temperature` for members other than themselves:

- `member_temperature_visibility: string` — one of `moderator`, `everyone`, `self_only`.

Permitted values and their effect:

| Value | Effect |
|---|---|
| `moderator` | Default. The Node publishes `xgen.member_temperature` for a member `M` only to clients whose authenticated Identity holds a moderator-or-higher role in the Space, plus `M`'s own client. |
| `everyone` | The Node publishes `xgen.member_temperature` for every member to every other member. |
| `self_only` | The Node publishes `xgen.member_temperature` only to the subject's own client. Moderators and admins see only their own; automated consequences (3.7.13.6) run entirely Node-side. |

The default at Space creation is `moderator`. The field MAY be updated by a Space owner via a new EventType `state.space_temperature_visibility`:

```json
{
  "type": "state.space_temperature_visibility",
  "content": {
    "member_temperature_visibility": "everyone"
  }
}
```

The Event is signed by the Space owner, applies to the Space identified by the Event's `space_id`, and supersedes prior values from the moment it is included in the DAG.

The field is an **open enum** — future phases MAY introduce additional values without breaking older Nodes (an older Node that does not recognise a value defaults to `moderator` behaviour). Third-party operators MUST NOT define their own values; the visibility enum is protocol-reserved.

Visibility for `xgen.room_temperature` is NOT configurable — Room temperature is always visible to every Room member. This is structural: concealing the Room's collective state from the people in the Room would defeat the purpose of self-correcting feedback (D-061).

##### 3.7.13.4 Visibility enforcement

The home Node enforces visibility. Clients receive only what their role permits; the client does not implement filtering. Specifically:

- A client requesting Room metadata or subscribing to Room events SHALL receive `xgen.member_temperature` for member `M` only if (a) the client is authenticated as `M`, or (b) the client's authenticated Identity holds the role required by the current `member_temperature_visibility` setting.
- A client whose role does not permit visibility for `M` SHALL receive events for that member with the `xgen.member_temperature` key omitted entirely — not set to a placeholder value.
- `xgen.room_temperature` is always included for any member of the Room regardless of role.

The Node enforces visibility on outgoing `meta_atts` filtering, not by computing different values. The plugin produces one value per (Room, member) pair; the Node decides whether to include that value in each outgoing client subscription based on the visibility setting and the recipient's role.

##### 3.7.13.5 Computation locality

The Room's home Node is the **authoritative source** for both temperature values. The plugin running on the home Node computes the values; the Node transmits them.

Federated copies of the Room's events MAY carry temperature values via `meta_atts` on relayed events. Receiving Nodes do not recompute — they relay the home Node's values to their own connected clients (subject to the visibility rules above). If the home Node changes (Space migration, 3.12), the new home Node's plugin takes over and may produce different temperature values; this is correct behaviour.

The protocol does not specify the plugin interface. The Node implementation is free to load the plugin in whatever form it supports (native library, WASM module, external process). What the protocol observes is the `meta_atts` keys on outgoing events. As long as the values are well-formed floats in range, the Node has satisfied the protocol contract.

##### 3.7.13.6 Automated consequences

When a member's temperature crosses a threshold the plugin considers actionable, the plugin instructs the home Node to issue a signed `membership.kick` or `membership.mute` Event (3.7.8) with `reason = auto_temperature`:

- `membership.kick` with `reason = "auto_temperature"` is the recommended consequence for human members. The Event carries a `cooldown_until` timestamp; the member is removed from the Space until the cooldown elapses.
- `membership.mute` with `reason = "auto_temperature"` is the recommended consequence for AI members (`is_ai = true`). The Event carries a `cooldown_until` timestamp; the member retains Space and Room membership but cannot post until the cooldown elapses.

The asymmetry (human kick vs AI mute) is a **recommendation for plugin authors**, not a protocol mandate. The protocol distinguishes `membership.kick` from `membership.mute` (3.7.8) and makes `is_ai` observable (3.6.10.1); plugin authors are free to use, ignore, or invert the asymmetry. A plugin that issues no automated consequences at all is valid — the plugin may compute and publish temperature purely as a display signal without ever issuing kicks or mutes.

The `cooldown_until` timestamp on the issued Event is the plugin's choice. Ch6 §6.12.6 documents recommended UI defaults of 2 hours for `auto_temperature` kicks and 15 minutes for `auto_temperature` mutes; the plugin MAY use these defaults, configure them per Space, or compute them dynamically.

The signing rules for `auto_temperature` Events follow 3.7.8 — the Event is signed by the Identity that issues it, which for `auto_temperature` actions is typically the home Node's operator Identity acting as an automated moderation agent.

##### 3.7.13.7 Temperature is not part of state resolution

Temperature values travel as `meta_atts` on existing Events and as a Node-to-client metadata field; they are not state events in the DAG and are not subject to state resolution (3.9). Two clients observing the same Room may briefly see different temperature values during transient network conditions; this is acceptable because temperature is a live signal, not a consensus value. The DAG-resident consequences (`membership.kick`, `membership.mute` with `auto_temperature`) are subject to standard state resolution like any other membership event.

##### 3.7.13.8 EventType registry addition

| Type | Purpose | Phase |
|---|---|---|
| `state.space_temperature_visibility` | Updates the `member_temperature_visibility` value for a Space. Signed by the Space owner. | Phase 2 |

---

### 3.8 Auth Module — Tier 1 Specification

*Status: complete*

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
| `email_hash` | hash string | Salted SHA-256 hash of email — only the hash propagates |
| `phone_hash` | hash string | Salted SHA-256 hash of phone — only the hash propagates |

**Two contact data options for Node operators**

Node operators choose how contact details appear in assertions. Plaintext contact details are not permitted in XGen Trust Assertions — the federation is append-only and plaintext contact data propagated to peer Nodes cannot be reliably recalled. Two privacy-preserving options are available:

**Option A — Hashed**
A salted SHA-256 hash of the contact detail appears. The Auth Module can re-verify the hash against its own records. The Node cannot extract the original contact detail from the hash. Only the hash propagates across the federation — useless to an attacker without the original value.

```json
"claims": { "tier_verified": true, "email_verified": true, "email_hash": "sha256:a3f9b2c1..." }
```

**Option B — Flag only**
Only the verification fact appears. No contact detail — not even a hash — enters the protocol. Any Node needing to verify contact details must query the Auth Module directly using the `identity_id`.

```json
"claims": { "tier_verified": true, "email_verified": true }
```

**Why plaintext contact details are not permitted**

Plaintext email addresses and phone numbers propagated into a federated, append-only Event log cannot be recalled. Once replicated to peer Nodes, they persist indefinitely regardless of right-to-erasure requests. This is incompatible with GDPR Article 17 and equivalent obligations in other jurisdictions. The Auth Module holds the authoritative contact record — the protocol does not need to carry it.

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

*Status: complete*

The complete conflict resolution algorithm for XGen. Implements the seven-layer priority stack declared as a forward reference in 3.2.7. State resolution is a pure function of the Event log — every Node that holds the same Events applies the same algorithm and reaches the same answer, without communication.

---

#### 3.9.1 What State Resolution Solves

The DAG (3.2.5) guarantees that no Event is ever lost when concurrent Events are produced — both are preserved as parallel branches and a later Event merges them. What the DAG does not guarantee is *which value wins* when two concurrent Events make mutually exclusive claims about the same state key. A Room cannot simultaneously have two names. A banned Identity cannot simultaneously be a member. State resolution is the deterministic algorithm that selects one authoritative answer from a set of competing concurrent state Events.

**When state resolution applies:** only to state Events and membership Events — Event types that define the current value of a state key. It does not apply to message Events (`message.text`, `message.image`, etc.) — concurrent messages are both displayed, in whatever order the Node presents them, and there is no conflict.

**State key concept:** a state key is a tuple of `(EventType, state_key_field)` that uniquely identifies a piece of mutable state. For example:
- `(state.room_name, room_id)` — the current name of a specific Room
- `(membership.join, identity_id)` — the current membership status of a specific Identity
- `(state.node_priority, space_id)` — the current Node ordering for a specific Space

Two Events conflict when they carry the same state key and are not causally ordered — that is, neither appears in the other's `prev_events` chain.

---

#### 3.9.2 Convergence Guarantee

State resolution in XGen produces **strong eventual consistency**: every Node that holds the same set of Events will compute the same state, regardless of the order in which it received those Events.

This guarantee holds because:
1. The algorithm is a pure function of the Event content — no random elements, no timestamps used as tiebreakers (clocks are unreliable across Nodes), no node-local state consulted
2. Every layer of the priority stack is deterministic given the Event content alone
3. The absolute backstop (Layer 5c — lexicographic event_id) is a content hash — ungameable and always produces a unique winner
4. The algorithm is commutative and associative — applying it to Events in any arrival order produces the same result

A Node that receives Events in a different order than its peers will temporarily hold a different view of current state. As soon as it receives the missing Events (via sync_request on reconnect or federation propagation), it recomputes state and converges to the same answer as all other Nodes.

---

#### 3.9.3 The Seven-Layer Resolution Algorithm

When two or more Events conflict on the same state key, the Node applies the following layers in order. The first layer that produces a unique winner terminates resolution. Subsequent layers are only reached when all higher layers are tied or inapplicable.

**Input:** a set of two or more conflicting Events `{E1, E2, ..., En}` — all carrying the same state key, none causally ordered relative to the others.

**Output:** exactly one winning Event.

---

**Layer 1 — EventType logic**

Certain EventType pairs have a hardcoded winner regardless of any other factor. These represent logical truths about the protocol — a ban cannot be overridden by a concurrent join because that would make banning meaningless.

Hardcoded EventType priority rules:

| Winner EventType | Beats |
|---|---|
| `membership.ban` | `membership.join`, `membership.invite`, `membership.kick` |
| `membership.kick` | `membership.join`, `membership.invite` |
| `membership.leave` | `membership.join` |

`membership.ban` is never overridden by a concurrent Event at this layer. `membership.kick` loses only to a concurrent `membership.ban`.

If all conflicting Events carry the same EventType, Layer 1 produces no winner — proceed to Layer 2.

If conflicting Events carry different EventTypes and the table above applies, the winner is determined here. Resolution terminates.

---

**Layer 2 — Auth Tier of the producing Node**

Higher Auth Tier wins for same-type conflicts. The Auth Tier is the Tier at which the Space operates — declared in `state.space_create` and immutable thereafter.

Tier ordering (highest to lowest): Tier 4 > Tier 3 > Tier 2 > Tier 1.

Rationale: a higher-Tier Node has stronger identity verification and stricter institutional accountability. Its state assertions carry more institutional weight.

Note: in Phase 1, all Spaces are Tier 1, so Layer 2 never produces a winner in Phase 1 deployments. It becomes active when Tier 2+ Spaces are introduced in production.

If all conflicting Events were produced in Spaces operating at the same Tier, Layer 2 produces no winner. Proceed to Layer 3.

---

**Layer 3 — Home Node assertion**

For conflicts involving an Identity's own state — such as an Identity's membership status or their own `system.key_rotation` Event — the Identity's home Node is the authoritative source of truth. The home Node is declared in the Identity record (`home_node` field, 3.6.6).

If one of the conflicting Events was produced by (or directly authorised by) the Identity's home Node, that Event wins.

Rationale: the home Node holds the Identity's full history and current key material. In authority conflicts — where an Identity's permissions were being changed simultaneously with an action they took — the home Node's version of the Identity's current authorisation state is the ground truth.

If neither or both conflicting Events originate from the relevant Identity's home Node, Layer 3 produces no winner. Proceed to Layer 4.

---

**Layer 4 — Role within Space**

Higher role wins for same-Tier, same-type conflicts. The role of the `sender` Identity at the time the Event was produced is consulted from the Space membership state.

Role priority (highest to lowest): owner > admin > moderator > member.

Edge case — the role change problem: if an Identity's role was itself being changed concurrently with an action they took, the Node MUST apply Layer 3 (home Node assertion) first to determine which role assignment is authoritative, then apply that authoritative role to the conflict.

If conflicting Events are from Identities with the same role, Layer 4 produces no winner. Proceed to Layer 5.

---

**Layer 5 — Node ordering (three sublayers)**

Layer 5 is reached only when Layers 1–4 are all tied or inapplicable. It resolves conflicts by Node identity. Three sublayers apply in order.

**Layer 5a — Manual Node ordering (user-defined)**

The Space owner may declare an explicit priority ordering of federated Nodes via a `state.node_priority` Event (3.2.7). When present in the Space DAG:

1. Find the most recent valid `state.node_priority` Event in the Space DAG
2. Extract its `ordered_nodes` array
3. For each conflicting Event, find the `node_id` of the Node that produced it (home Node of the `sender` Identity)
4. The Event whose producing Node appears earliest (lowest index) in `ordered_nodes` wins
5. Nodes not listed fall through to Layer 5b

If no `state.node_priority` Event exists, or all conflicting Events are from unlisted Nodes, proceed to Layer 5b.

**Layer 5b — Federation recency (automatic default)**

The Node that most recently joined the federation for this Space has higher priority.

1. For each conflicting Event, find the `state.federation_add` Event recording when the producing Node joined this federation
2. The producing Node with the most recent `state.federation_add` timestamp wins
3. The home Node of the Space (which was never "added" via federation) is treated as having joined at Space creation time — lowest recency priority

Rationale: recently joined Nodes are more likely to have been vetted under current Trust Assertion policies and current Space rules.

If all conflicting Events are from Nodes with identical federation join timestamps, proceed to Layer 5c.

**Layer 5c — Lexicographic event_id (absolute backstop)**

The Event whose `event_id` sorts lower in lexicographic (Unicode code point) order wins.

```
xgen://hash/sha256:a3f9b2c1...  ← wins ("a" < "b")
xgen://hash/sha256:b2c3d4e5...
```

This layer is purely mechanical — a deterministic tiebreaker that requires no communication between Nodes, cannot be gamed (the event_id is a content hash), and always produces a unique winner since SHA-256 collisions are computationally infeasible.

---

#### 3.9.4 Resolution by Conflict Category

The four conflict categories from 3.2.7, with their characteristic resolution path:

**State conflict** — two concurrent Events setting the same state key to different values (e.g. two `state.room_name` Events).

Typical path: Layer 1 inapplicable (same EventType) → Layer 2 (same Tier in most deployments, no winner) → Layer 3 inapplicable (not an Identity's own state) → **Layer 4** (role priority, most common winner) → Layer 5 if roles are equal.

**Permission conflict** — two Events with opposing effects on the same Identity's status (e.g. concurrent `membership.ban` and `membership.invite`).

Typical path: **Layer 1** terminates resolution immediately — `membership.ban` always beats `membership.invite`.

**Authority conflict** — an Identity's permission was being changed simultaneously with an action they took (e.g. an admin being demoted at the same moment they kicked a member).

Typical path: Layer 1 inapplicable → Layer 2 (often same Tier, no winner) → **Layer 3** (home Node assertion determines authoritative role) → Layer 4 (apply home-Node-authoritative role to the conflict).

**Ordering conflict** — causal ambiguity where the meaning of subsequent Events depends on which of two concurrent Events is treated as prior.

Typical path: full Layer 5 sequence, as Layers 1–4 are usually inapplicable. The lexicographic backstop produces the canonical ordering.

---

#### 3.9.5 Split-Brain Recovery

A split-brain occurs when a network partition divides federated Nodes into groups that cannot communicate. Each group continues to produce Events independently. When the partition heals, the groups must converge.

**During partition:** each Node continues normally, accepting Events from locally connected clients. DAG branches diverge. No special protocol action is taken — there is no mechanism to distinguish a partition from a peer being temporarily offline.

**On reconnection:**
1. Nodes re-establish the federation transport connection and complete re-federation (3.4.6)
2. Each Node sends `transport.sync_request` for each shared Space and Room
3. Each Node receives Events it missed during the partition
4. Each Node independently replays the merged Event set through state resolution
5. Because state resolution is a pure function of the Event set, both Nodes compute identical current state

**Conflict volume:** bounded by the number of state Events produced concurrently during the partition. Message Events do not conflict — all are preserved and displayed. Only state and membership Events targeting the same state key require resolution.

**No special recovery protocol is needed.** The standard DAG merge plus state resolution algorithm handles split-brain recovery automatically as a consequence of the convergence guarantee (3.9.2).

---

#### 3.9.6 Pending Event Timeout

A Node receiving an Event whose dependencies are not yet satisfied MUST hold it in a pending buffer. Two dependency conditions trigger buffering:

1. **Unknown predecessor.** The Event's `prev_events` references an Event ID the Node does not yet hold (3.2.6 step 9).
2. **Unknown signer Identity.** The Event's `sender` is an Identity URI that is not yet in the Node's local Identity registry — typically the case for federation first-contact events where the author's Identity record has not yet replicated to this Node (Federation Event Propagation milestone, F-10).

If both conditions hold, the Event waits for both arrivals before re-entering the validation pipeline.

**Timeout rule:** if the dependencies are not received within **30 seconds** (WD-08), the pending Event is discarded. The Node logs the discard with the Event ID, missing predecessor IDs, and missing signer Identity (whichever applied). Two error codes carry the discard reason:

- `4002 predecessor_timeout` — at least one predecessor was still missing at the moment of timeout (whether or not the Identity was also missing).
- `4006 identity_record_timeout` — only the signer Identity was missing; all predecessors were present.

The predecessor-code-wins rule reflects the historically prior failure mode (4002 was the only timeout error before F-10 generalised the buffer); 4006 surfaces specifically when Identity replication is the bottleneck, supporting operator diagnostics.

Rationale: indefinite holds can be exploited to consume unbounded memory. The 30-second window is generous for legitimate slow federation paths. A discarded Event will be re-requested via the next `transport.sync_request` or F-1a tip-exchange on next federation handshake.

The timeout value is a work definition (WD-08) pending Phase 1 testing validation.

---

#### 3.9.7 State Snapshot and Incremental Application

For efficiency, a Node SHOULD maintain a current state snapshot in memory rather than replaying the full Event log from genesis on every state query. The snapshot is updated incrementally as new Events are validated and accepted.

**Snapshot update rule:** when a new state or membership Event passes the full 13-step validation pipeline (3.2.6):
1. Check whether the new Event conflicts with any existing state value for the same state key
2. If no conflict: apply the new Event directly to the snapshot
3. If conflict: run the state resolution algorithm over the conflicting set, apply the winner to the snapshot. The loser Event remains in the DAG permanently — it is never deleted.

**Snapshot persistence:** the Node MUST be able to reconstruct its state snapshot from the Event log on startup (Ch4 section 4.8.5 — state reconstruction is a hard startup requirement). The snapshot is a performance optimisation, not the source of truth. The Event log is always authoritative.

**Snapshot format:** implementation-defined. The reference implementation uses SQLite tables derived from Event replay.

---

#### 3.9.8 State Resolution Error Codes

State resolution failures use the 4000 error code range, distinct from transport (1xxx), federation (2xxx), and identity (3xxx) ranges.

| Code | Error string | Meaning |
|---|---|---|
| 4001 | `state_conflict_unresolvable` | Resolution algorithm failed to produce a winner — should never occur; indicates implementation bug |
| 4002 | `predecessor_timeout` | Pending Event discarded — missing predecessors not received within timeout window |
| 4003 | `dag_cycle_detected` | Event rejected — `prev_events` would create a cycle |
| 4004 | `state_key_invalid` | Event carries a state key not valid for its EventType |
| 4005 | `resolution_stack_exhausted` | All five resolution layers applied without winner — should never occur due to Layer 5c backstop; indicates implementation bug |
| 4006 | `identity_record_timeout` | Pending Event discarded — signer Identity record not received within timeout window (Federation Event Propagation milestone, F-10) |

**Display rule** — same pattern as all other error ranges:

```
Error 4002 (predecessor_timeout): Pending Event discarded — missing predecessors
not received within the 30-second window. The sender will re-request on next sync.

Error 4006 (identity_record_timeout): Pending Event discarded — signer Identity
record not received within the 30-second window. Recovery is via the next
sync_request or F-1a tip-exchange on next federation handshake.
```

**Predecessor-code-wins sub-rule for the both-missing case** (3.9.6): if both a predecessor AND the signer Identity were missing at the moment of timeout, the emitted error code is 4002 (not 4006). The reasoning is that 4002 is the historically prior failure mode and the more common case; a third "both-missing" error code would be overengineering. Operators filtering log streams on 4006 specifically diagnose Identity-replication bottlenecks unambiguously.

Both the numeric code and the error string MUST be included in all error messages.

---

### 3.10 End-to-End Encryption

*Status: complete*

The end-to-end encryption layer for XGen. All message content in XGen Spaces is end-to-end encrypted using MLS (Messaging Layer Security, RFC 9420). The Node is an MLS Delivery Service — it routes MLS handshake messages and stores encrypted payloads, but is structurally excluded from decrypting any content. Only Space members holding valid MLS group state can decrypt messages.

**Decision record:** D-031 — MLS selected over Megolm. See DECISIONS.md for full rationale.

---

#### 3.10.1 Encryption Model and Node Role

XGen uses a two-layer model:

**Layer 1 — Transport security (3.3.1):** TLS encrypts the wire between clients and Nodes, and between Nodes. This protects against network-level eavesdropping. The Node can see plaintext of Events at this layer — it must, in order to validate signatures, route by space_id/room_id, and enforce access control.

**Layer 2 — End-to-end encryption (this section):** MLS encrypts the `content` field of message Events. The Node sees the encrypted blob but cannot decrypt it. Only clients holding valid MLS epoch keys for the group can read the content.

**What the Node can see:**
- All Event envelope fields: `protocol_version`, `type`, `event_id`, `sender`, `room_id`, `space_id`, `prev_events`, `timestamp`, `signature`, `meta_atts`
- The encrypted `content` blob (opaque bytes)
- MLS handshake messages (Welcome, Commit, Proposal) — routed as part of the Delivery Service role

**What the Node cannot see:**
- The plaintext of any message content
- MLS private key material of any client
- The ratchet tree leaf secrets of any member

**E2E encryption scope:** applies to all message Events (`message.text`, `message.image`, `message.file`, `message.reaction`, `message.edit`, `message.delete`). State Events and membership Events (`state.*`, `membership.*`, `system.*`, `federation.*`) are **not** E2E encrypted — they must be readable by Nodes for protocol operation. Metadata is visible to the infrastructure layer; content is not.

---

#### 3.10.2 MLS Concepts in XGen Context

MLS defines a group as a set of members sharing a ratchet tree. Each leaf corresponds to one member device. The group state advances through epochs — each epoch corresponds to a snapshot of group membership and a unique set of encryption keys. When membership changes (join, leave, update), the group advances to a new epoch with fresh keys.

**XGen mappings:**

| MLS concept | XGen mapping |
|---|---|
| MLS Group | One Room within a Space |
| MLS Member | One client device of a Space member |
| MLS Epoch | Advances on every membership change Event |
| MLS Delivery Service | The XGen Node |
| MLS Authentication Service | The XGen Auth Module |
| MLS KeyPackage | Uploaded by each client on joining, stored by the Node |
| MLS Welcome message | Sent to new members on join |
| MLS Commit message | Advances the group to a new epoch |
| MLS Proposal | Proposes a change to group state (add/remove/update) |

**One MLS group per Room, not per Space.** A Space with ten Rooms has ten independent MLS groups. Members of the Space are members of all Rooms (Phase 1), but each Room has its own key material. Compromising keys for one Room does not compromise another.

---

#### 3.10.3 KeyPackage Management

A KeyPackage is a signed bundle containing a client's public key material for MLS group operations. Clients upload KeyPackages to their home Node, which stores them and distributes them on request when a new member is being added to a group.

**KeyPackage schema:**

```json
{
  "protocol_version": "0.1",
  "type": "mls.key_package",
  "identity_id": "xgen://pubkey/ed25519:AAAA...",
  "device_id": "xgen://pubkey/ed25519:BBBB...",
  "mls_key_package": "<base64url-encoded MLS KeyPackage TLS-serialised per RFC 9420>",
  "uploaded_at": "2026-04-26T10:00:00.000Z",
  "valid_until": "2026-07-26T00:00:00.000Z",
  "signature": "ed25519:BBBB...:base64url-signature"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `identity_id` | pubkey_uri | yes | The Identity this KeyPackage belongs to |
| `device_id` | pubkey_uri | yes | The specific device keypair — may differ from identity_id in multi-device setups |
| `mls_key_package` | string | yes | base64url-encoded, TLS-serialised MLS KeyPackage per RFC 9420 §4 |
| `uploaded_at` | datetime | yes | When this KeyPackage was uploaded |
| `valid_until` | datetime | yes | Expiry — 90 days recommended (work definition, WD-14) |
| `signature` | string | yes | Signed by the device keypair over the canonical form of this message |

**KeyPackage lifecycle:**
- A client generates a fresh KeyPackage on first run and after each use (a KeyPackage is single-use — consuming it for a Welcome message invalidates it)
- The Node MUST maintain a pool of at least 3 unused KeyPackages per client device
- The client is responsible for replenishing its pool when the Node signals that the count is low
- Expired KeyPackages MUST be discarded by the Node

---

#### 3.10.4 Group Initialisation

When a Room is created, the creating client initialises the MLS group for that Room. The group starts with one member (the creator) in epoch 0.

**Sequence:**
1. Client produces `state.room_create` Event (3.7.5) — the Room DAG root
2. Client initialises a new MLS group with its own KeyPackage as the sole leaf
3. Client stores the initial MLS group state locally
4. Client produces `state.mls_group_init` Event:

```json
{
  "protocol_version": "0.1",
  "type": "state.mls_group_init",
  "room_id": "xgen://hash/sha256:b2c3d4e5...",
  "space_id": "xgen://hash/sha256:c3d4e5f6...",
  "mls_group_id": "<base64url-encoded MLS group_id>",
  "mls_cipher_suite": 2,
  "epoch": 0,
  "timestamp": "2026-04-26T10:00:00.000Z",
  "signature": "ed25519:AAAA...:base64url-signature"
}
```

| Field | Description |
|---|---|
| `mls_group_id` | The MLS group identifier, derived as `SHA-256(room_id_bytes)` encoded as base64url |
| `mls_cipher_suite` | MLS cipher suite identifier per RFC 9420 §17.1. XGen mandates cipher suite 2: `MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519` for Phase 2 |
| `epoch` | Always 0 at group initialisation |

**Cipher suite:** XGen mandates MLS cipher suite 2 (`MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519`). This uses X25519 for key exchange, AES-128-GCM for AEAD encryption, SHA-256 for hashing, and Ed25519 for signatures — consistent with XGen's existing cryptographic choices throughout. Algorithm agility for MLS cipher suites is Phase 3.

---

#### 3.10.5 Member Addition (Welcome)

When a new Identity joins a Room (via `membership.join`), the existing members extend the MLS group to include the new member's device(s).

**Sequence:**
1. The adding client requests the new member's KeyPackage from the Node via `mls.key_package_request`
2. The Node returns the KeyPackage via `mls.key_package_response`
3. The adding client produces an MLS Add Proposal and a Commit advancing the group to a new epoch
4. The adding client produces an MLS Welcome message addressed to the new member's device
5. The adding client sends the Commit and Welcome to the Node:

```json
{
  "protocol_version": "0.1",
  "type": "mls.commit",
  "room_id": "xgen://hash/sha256:b2c3d4e5...",
  "space_id": "xgen://hash/sha256:c3d4e5f6...",
  "epoch": 3,
  "mls_commit": "<base64url-encoded TLS-serialised MLS MLSMessage of type commit>",
  "timestamp": "2026-04-26T10:00:00.000Z",
  "signature": "ed25519:AAAA...:base64url-signature"
}
```

```json
{
  "protocol_version": "0.1",
  "type": "mls.welcome",
  "room_id": "xgen://hash/sha256:b2c3d4e5...",
  "space_id": "xgen://hash/sha256:c3d4e5f6...",
  "recipient_identity_id": "xgen://pubkey/ed25519:CCCC...",
  "recipient_device_id": "xgen://pubkey/ed25519:DDDD...",
  "mls_welcome": "<base64url-encoded TLS-serialised MLS Welcome message>",
  "timestamp": "2026-04-26T10:00:00.000Z",
  "signature": "ed25519:AAAA...:base64url-signature"
}
```

6. The Node delivers `mls.welcome` to the new member's connected client
7. The new member processes the Welcome and derives its initial epoch keys
8. All existing members process the Commit and advance to the new epoch

**Epoch advance:** after a successful Commit, messages encrypted in the new epoch are accessible to the new member. Messages in prior epochs are not — this is forward secrecy.

---

#### 3.10.6 Member Removal (Leave / Kick / Ban)

When a member leaves, is kicked, or is banned, the group MUST advance to a new epoch to exclude them from future message decryption.

**Sequence:**
1. Any remaining member produces an MLS Remove Proposal and Commit
2. The Commit advances the group to a new epoch with the removed member's leaf blanked
3. The committing client sends `mls.commit` to the Node
4. All remaining members process the Commit and advance their local group state
5. No Welcome message is sent — the removed member receives nothing

**Post-removal security:** the removed member holds key material for epochs they participated in — they retain the ability to decrypt historical messages they received during membership. They cannot decrypt messages in any epoch after removal. This is correct behaviour: messages delivered to them while they were a member remain readable; future messages are inaccessible.

---

#### 3.10.7 Message Encryption

With an active MLS group, clients encrypt message content before producing an Event.

**Encryption flow:**
1. Client composes the plaintext message content (e.g. `{"text": "Hello"}` for `message.text`)
2. Client encrypts the plaintext using the current MLS epoch's application secret, producing an MLS PrivateMessage
3. Client base64url-encodes the TLS-serialised PrivateMessage
4. Client places the encrypted blob in the Event's `content` field:

```json
{
  "protocol_version": "0.1",
  "type": "message.text",
  "event_id": "xgen://hash/sha256:a3f9b2c1...",
  "sender": "xgen://pubkey/ed25519:AAAA...",
  "room_id": "xgen://hash/sha256:b2c3d4e5...",
  "space_id": "xgen://hash/sha256:c3d4e5f6...",
  "prev_events": ["xgen://hash/sha256:d4e5f6a7..."],
  "timestamp": "2026-04-26T10:00:00.000Z",
  "content": {
    "mls_ciphertext": "<base64url-encoded MLS PrivateMessage>"
  },
  "signature": "ed25519:AAAA...:base64url-signature"
}
```

When E2E encryption is active, `content` contains only `mls_ciphertext`. The plaintext schema (`text`, `uri`, `filename`, etc.) is inside the MLS PrivateMessage payload, invisible to the Node.

**Decryption flow:**
1. Receiving client verifies the Event signature (proves sender identity)
2. Extracts `content.mls_ciphertext`, base64url-decodes, TLS-deserialises
3. Decrypts the MLS PrivateMessage using the current epoch's application secret
4. Parses the plaintext payload and renders the message

**Signature separation:** the Event signature covers the encrypted `content` field, not the plaintext. This proves the sender produced this encrypted blob, without revealing the plaintext to the Node during verification.

---

#### 3.10.8 Spaces Without E2E Encryption

E2E encryption is the default for all Spaces. A Space MAY be created without E2E encryption for use cases where content inspection by the Node operator is a deliberate requirement (e.g. a public community Space with moderation, or a compliance-monitored corporate Space).

A Space's E2E encryption mode is declared at creation time in `state.space_create` via the `e2e_encryption` field:

```json
"e2e_encryption": true    // default — E2E enabled
"e2e_encryption": false   // explicit opt-out — content plaintext at Node layer
```

This field is **immutable after Space creation.** A Space cannot be retroactively encrypted or decrypted. Changing the mode would make existing messages inaccessible or expose previously encrypted content — both unacceptable outcomes.

Clients MUST display a visible indicator when operating in a Space without E2E encryption.

---

#### 3.10.9 Phase 1 Forward Compatibility

Phase 1 Nodes and clients do not implement MLS — E2E encryption is Phase 2. Phase 1 is forward-compatible without wire format changes.

**Phase 1 behaviour:**
- `content` carries plaintext payloads — `{"text": "Hello"}` for `message.text`, etc.
- No `mls.*` or `state.mls_group_init` Events exist
- `e2e_encryption` defaults to `false` for Phase 1 Spaces

**Phase 2 upgrade path:**
- Phase 1 Spaces cannot be upgraded to E2E encryption (field is immutable)
- Phase 2 Spaces declare `e2e_encryption: true` at creation and operate fully encrypted from genesis
- Phase 1 and Phase 2 Spaces coexist on the same Node — distinguished by the `e2e_encryption` field
- Phase 2 Nodes receiving Events from Phase 1 Nodes treat `mls_ciphertext`-absent `content` as unencrypted Phase 1 content

---

#### 3.10.10 MLS EventType Registry Additions

The following EventTypes extend the Phase 1 registry (3.2.2):

*MLS group management events:*

| EventType | Description |
|---|---|
| `state.mls_group_init` | Initialises the MLS group for a Room at epoch 0 (3.10.4) |
| `mls.key_package` | Client uploads a KeyPackage to the Node (3.10.3) |
| `mls.key_package_request` | Node requests a KeyPackage for a given Identity from a peer Node |
| `mls.key_package_response` | Node responds with a KeyPackage |
| `mls.commit` | Advances the MLS group to a new epoch (3.10.5, 3.10.6) |
| `mls.welcome` | Delivers MLS Welcome message to a newly added member (3.10.5) |

---

#### 3.10.11 E2E Encryption Error Codes

E2E encryption failures use the 5000 error code range.

| Code | Error string | Meaning |
|---|---|---|
| 5001 | `mls_key_package_not_found` | No valid KeyPackage available for the requested Identity/device |
| 5002 | `mls_key_package_expired` | KeyPackage exists but has passed its `valid_until` date |
| 5003 | `mls_commit_invalid` | MLS Commit message failed validation |
| 5004 | `mls_welcome_delivery_failed` | Welcome message could not be delivered to the new member |
| 5005 | `mls_epoch_mismatch` | Client operating on a different epoch than the group — re-sync required |
| 5006 | `mls_cipher_suite_unsupported` | Requested cipher suite is not supported by this implementation |
| 5007 | `e2e_required` | Space requires E2E encryption but client sent an unencrypted `content` field |

**Display rule** — same pattern as all other error ranges:

```
Error 5005 (mls_epoch_mismatch): Client is operating on a stale MLS epoch.
Re-sync your MLS group state before sending further messages.
```

---

### 3.11 Auth Module — Tiers 2–4 Interfaces

*Status: complete*

Interface specifications for Tier 2 (ISO 27001 Professional), Tier 3 (Corporate / Regulated Industry), and Tier 4 (Government / Healthcare) Auth Modules. The slot contract established in 3.8.2 applies to all Tiers. This section specifies what each higher Tier adds to that contract — the additional verification requirements, Trust Assertion fields, and institutional context each Tier operates in.

Implementations of Tier 2, 3, and 4 Auth Modules are developed in institutional collaboration with qualified organisations. XGen ships only Tier 1 as a reference implementation.

*See Chapter 2 — Auth Module & Trust Assertion for the architectural framework and the cumulative Tier model.*

---

#### 3.11.1 Tier Model Recap

Tiers are cumulative: a Tier 3 Auth Module satisfies all Tier 2 requirements plus Tier 3 additions. A Space declaring `auth_tier: 3` accepts Trust Assertions from Tier 3 and Tier 4 Auth Modules, but not from Tier 1 or Tier 2.

The Auth Tier is a property of the Space, declared immutably in `state.space_create`. An Identity's Trust Assertion Tier must be equal to or higher than the Space's `auth_tier` for that Identity to be accepted as a member.

All four Tiers share the same slot contract defined in 3.8.2:
- Auth Module is an external independent service with its own keypair
- Communicates with the Node via the `auth.verify_request` / `auth_assertion_query` interface
- Issues Trust Assertions signed with its own keypair
- Node validates Trust Assertions by verifying the Auth Module's signature against its registered public key

What differs between Tiers is the **verification depth**, the **identity evidence required**, and the **Trust Assertion claims produced**.

---

#### 3.11.2 Tier 2 — ISO 27001 Professional

**Target operators:** professional services firms, SMEs, academic institutions, managed service providers operating under ISO 27001 or equivalent information security management standards.

**Verification requirements beyond Tier 1:**

| Requirement | Description |
|---|---|
| Real name verification | Legal name confirmed against government-issued ID (passport, national ID card, or driver's licence) — document inspection required, not self-declaration |
| Organisational affiliation | Employment or membership verified against official organisational records (e.g. company email domain + HR confirmation, or institution credential) |
| ISO 27001 operator attestation | The Auth Module operator MUST hold or be under contract with an ISO 27001-certified organisation. The certification scope must cover identity verification operations. |

**Verification states for Tier 2:**

Tier 2 replaces Tier 1's phone/email verification states with identity document verification states:

| State | Meaning |
|---|---|
| `A` | Identity document presented and inspected, name verified, affiliation pending |
| `B` | Identity document verified, organisational affiliation verified |
| `C` | Identity document verified, affiliation verified, role within organisation confirmed |

State `B` is the minimum acceptable state for a Tier 2 Trust Assertion to be valid in a Tier 2 Space.

**Trust Assertion additional claims for Tier 2:**

The `claims` object in the Trust Assertion (3.8.4) extends with the following optional fields for Tier 2:

```json
"claims": {
  "tier_verified": 2,
  "legal_name_verified": true,
  "organisation_verified": true,
  "organisation_domain": "example.com",
  "iso27001_operator": true
}
```

| Field | Type | Description |
|---|---|---|
| `tier_verified` | integer | Always `2` for Tier 2 assertions |
| `legal_name_verified` | boolean | True if legal name confirmed against government ID |
| `organisation_verified` | boolean | True if organisational affiliation confirmed |
| `organisation_domain` | string | The verified organisation's primary domain — propagates in the assertion |
| `iso27001_operator` | boolean | True if the Auth Module operator holds ISO 27001 certification covering identity verification |

**Trust Assertion TTL for Tier 2:** 1 year, same as Tier 1 (work definition, WD-09). Renewal requires re-verification of organisational affiliation — employment status can change.

---

#### 3.11.3 Tier 3 — Corporate / Regulated Industry

**Target operators:** publicly listed companies, financial institutions, payment processors, large enterprises subject to SOX, Basel II/III, or PCI DSS compliance obligations.

**Verification requirements beyond Tier 2:**

| Requirement | Description |
|---|---|
| Enhanced due diligence | Identity verification meets AML/KYC standards — includes watchlist screening (PEP, sanctions) and adverse media checks |
| Corporate role verification | The Identity's role and authority level within the organisation is verified and attested (e.g. CFO, compliance officer, trader) — role determines what Space permissions are possible |
| Audit trail | The Auth Module operator MUST maintain a complete audit trail of all verification decisions, retained for a minimum of 7 years (SOX §802 requirement) |
| Regulatory compliance attestation | The Auth Module operator MUST be able to demonstrate compliance with applicable financial regulations in the jurisdictions of the Spaces it serves |

**Trust Assertion additional claims for Tier 3:**

```json
"claims": {
  "tier_verified": 3,
  "legal_name_verified": true,
  "organisation_verified": true,
  "organisation_domain": "bank.example.com",
  "iso27001_operator": true,
  "kyc_verified": true,
  "kyc_level": "enhanced",
  "corporate_role_verified": true,
  "corporate_role": "compliance_officer",
  "watchlist_clear": true,
  "watchlist_checked_at": "2026-04-01T00:00:00.000Z"
}
```

| Field | Type | Description |
|---|---|
| `tier_verified` | integer | Always `3` for Tier 3 assertions |
| `kyc_verified` | boolean | True if KYC/AML verification completed |
| `kyc_level` | string | `standard` or `enhanced` — level of due diligence applied |
| `corporate_role_verified` | boolean | True if role within organisation confirmed |
| `corporate_role` | string | The verified role — Auth Module operator defines the vocabulary |
| `watchlist_clear` | boolean | True if Identity cleared all applicable watchlist checks |
| `watchlist_checked_at` | datetime | When the watchlist check was performed — for staleness assessment |

**Trust Assertion TTL for Tier 3:** 6 months (work definition, WD-15). Watchlist status and corporate role can change more rapidly than at Tier 2 — shorter TTL reflects higher-stakes context.

**Audit trail note:** the Node does not verify the existence of the Auth Module operator's audit trail. This is an institutional obligation enforced by the operator's compliance regime, not by the protocol. The Trust Assertion `iso27001_operator: true` and `kyc_verified: true` fields are attestations from the Auth Module operator — the Node accepts them at face value and trusts the operator's institutional accountability.

---

#### 3.11.4 Tier 4 — Government / Healthcare

**Target operators:** government agencies, defence organisations, national health services, healthcare providers subject to HDS, SGB V, or equivalent national health data regulations. eIDAS-compliant identity providers.

**Verification requirements beyond Tier 3:**

| Requirement | Description |
|---|---|
| eIDAS Level of Assurance High | Identity verification meets eIDAS LoA High: in-person or remote verification using qualified electronic signature or qualified certificate. REF-04. |
| Government credential binding | The Identity is bound to a government-issued credential: national identity card with chip, passport with biometric verification, or equivalent. The credential's digital signature must be verified. |
| Role and clearance verification | For government Spaces, the Identity's security clearance level and organisational role are verified against official government records. Auth Module operator must have government accreditation. |
| Hardware authentication support | For Tier 4 Spaces that require it, the Auth Module MUST support hardware-bound authentication (FIDO2/WebAuthn, HSM-backed keys). REF-16. |
| Data localisation | The Auth Module operator MUST process and store identity verification data within the jurisdiction(s) required by applicable law. |

**Trust Assertion additional claims for Tier 4:**

```json
"claims": {
  "tier_verified": 4,
  "legal_name_verified": true,
  "organisation_verified": true,
  "organisation_domain": "ministry.gov.example",
  "iso27001_operator": true,
  "kyc_verified": true,
  "kyc_level": "enhanced",
  "corporate_role_verified": true,
  "corporate_role": "data_protection_officer",
  "watchlist_clear": true,
  "watchlist_checked_at": "2026-04-01T00:00:00.000Z",
  "eidas_loa": "high",
  "government_credential_bound": true,
  "credential_type": "national_id_chip",
  "clearance_verified": true,
  "clearance_level": "confidential",
  "jurisdiction": "EU",
  "data_localisation": "EU"
}
```

| Field | Type | Description |
|---|---|
| `tier_verified` | integer | Always `4` for Tier 4 assertions |
| `eidas_loa` | string | eIDAS Level of Assurance: `substantial` or `high`. Tier 4 requires `high`. |
| `government_credential_bound` | boolean | True if Identity is bound to a verified government-issued credential |
| `credential_type` | string | Type of government credential: `national_id_chip`, `passport_biometric`, `qualified_certificate` |
| `clearance_verified` | boolean | True if security clearance verified (government Spaces only; omit for healthcare) |
| `clearance_level` | string | Verified clearance level — vocabulary is operator-defined and jurisdiction-specific |
| `jurisdiction` | string | The jurisdiction(s) under whose law this verification was performed — ISO 3166-1 alpha-2 or `EU` |
| `data_localisation` | string | Where identity data is stored and processed — jurisdiction code(s) |

**Trust Assertion TTL for Tier 4:** 3 months (work definition, WD-16). Government clearances and roles can be revoked at short notice. Shorter TTL reduces the window during which a revoked Identity retains a valid Trust Assertion.

**Key rotation note for Tier 4:** key rotation is optional even at Tier 4 (D-001 in DECISIONS.md). HSM-backed permanent keys are a legitimate and compliant operational choice. The shorter TTL mitigates the risk of a compromised key persisting indefinitely.

---

#### 3.11.5 Cross-Tier Trust Assertion Compatibility

A Space declares its minimum required Tier in `state.space_create`. The Node enforces this at membership time:

- A `tier_verified: 4` Trust Assertion is accepted in a Space requiring Tier 1, 2, 3, or 4
- A `tier_verified: 2` Trust Assertion is accepted in a Space requiring Tier 1 or 2 only
- A `tier_verified: 1` Trust Assertion is **not** accepted in a Space requiring Tier 2 or above

The Node validates this by checking `claims.tier_verified >= state.space_create.auth_tier` during the membership acceptance pipeline.

**Federated Spaces and mixed-Tier Auth Modules:** when two federated Nodes serve a shared Space, each Node may trust different Auth Modules. Node A may trust Auth Module X (Tier 2 certified), and Node B may trust Auth Module Y (Tier 3 certified). Both are valid providers for a Tier 2 Space — Node A accepts assertions from X, Node B accepts assertions from Y. The Trust Assertion carries the `tier_verified` claim that the accepting Node verifies. Neither Node needs to trust the other's Auth Module.

---

#### 3.11.6 Auth Module Registration for Higher Tiers

The registration process for Tier 2, 3, and 4 Auth Modules follows the same out-of-band process as Tier 1 (3.8.7): the Auth Module operator provides their public key to the Node operator, who adds it to the Node's trusted Auth Module list.

Additional requirements for higher Tier registration:

| Tier | Additional registration requirement |
|---|---|
| Tier 2 | Node operator SHOULD verify the Auth Module operator's ISO 27001 certificate before registering |
| Tier 3 | Node operator MUST verify regulatory compliance attestations and the KYC/AML capability of the Auth Module operator |
| Tier 4 | Node operator MUST have a formal institutional agreement with the Auth Module operator. Government accreditation documentation required. |

These requirements are institutional obligations — the protocol does not enforce them at the wire level. They are recorded here to define what a conformant Tier 3 or Tier 4 deployment looks like.

---

#### 3.11.7 Auth Module Error Codes for Higher Tiers

Higher-Tier Auth Module failures extend the 3000 error code range established in 3.6.5. The following codes are added for Phase 2:

> **Identity error range reservation:** the full range 3000–3099 is reserved for identity-related errors. Codes 3000–3009 cover registration errors (3.6.5). Codes 3010–3016 cover higher-tier Auth Module errors (this section). Codes 3020–3023 cover identity replication errors (3.13.10). Implementers MUST NOT use any code in the 3000–3099 range for non-identity purposes.

| Code | Error string | Meaning |
|---|---|---|
| 3010 | `auth_tier_insufficient` | Identity's Trust Assertion Tier is below the Space's required `auth_tier` |
| 3011 | `kyc_verification_pending` | Identity's KYC/AML verification has not yet completed — Tier 3/4 |
| 3012 | `watchlist_match` | Identity matched a watchlist entry — Tier 3/4 — requires human review |
| 3013 | `eidas_loa_insufficient` | Trust Assertion eIDAS LoA is below the Space's required level — Tier 4 |
| 3014 | `government_credential_required` | Space requires government credential binding — Tier 4 |
| 3015 | `clearance_level_insufficient` | Identity's clearance level is below the Space's requirement — Tier 4 government Spaces |
| 3016 | `data_localisation_violation` | Auth Module's data localisation does not satisfy the Space's jurisdictional requirements — Tier 4 |

**Display rule** — same pattern as all other error ranges:

```
Error 3010 (auth_tier_insufficient): Your identity verification level (Tier 1) does
not meet the minimum required for this Space (Tier 3). Contact the Space administrator
or upgrade your verification through a qualifying Auth Module.
```

---

#### 3.11.8 Audit Log Requirements

XGen defines two distinct and independent log types. This section specifies the **audit log** — a permanent, structured, accountability record of protocol-level facts. It is not a debug or diagnostic tool.

**The two log types:**

| | Debug log | Audit log |
|---|---|---|
| Purpose | Diagnose technical problems | Prove accountability |
| Audience | Developer, operator | Auditor, compliance officer, regulator |
| Content | Technical events, errors, timings | Who did what, when, to whom |
| Format | Human-readable lines | Structured append-only records |
| Retention | Until operator deletes | Defined by regulation (3–20 years) |
| Integrity | Not required | Append-only, ideally tamper-evident |
| Controlled by | `[logging]` section in config | Separate — always on at Tier 3+ |

---

**Node-level protocol audit log**

The Node MUST maintain an append-only protocol audit log recording all membership and state-change Events. This log is independent of the debug log and cannot be disabled by config.

The protocol audit log records the following Event types whenever they occur in any Space hosted by or federated to this Node:

| EventType | What is recorded |
|---|---|
| `membership.join` | Identity joined Space — identity_id, space_id, timestamp, approving_node_id |
| `membership.leave` | Identity left Space — identity_id, space_id, timestamp |
| `membership.invite` | Identity invited — inviter_id, invitee_id, space_id, timestamp |
| `membership.kick` | Identity kicked — kicker_id, kicked_id, space_id, timestamp, reason if present |
| `membership.ban` | Identity banned — banner_id, banned_id, space_id, timestamp, reason if present |
| `state.space_create` | Space created — creator_id, space_id, auth_tier, timestamp |
| `state.room_create` | Room created — creator_id, room_id, space_id, timestamp |
| `state.federation_add` | Federation established — initiating_node_id, receiving_node_id, space_id, timestamp |
| `state.federation_remove` | Federation ended — node_id, space_id, timestamp, reason |
| `identity.register` | Identity registered — identity_id, home_node_id, timestamp, tier_verified |
| `system.key_rotation` | Key rotation performed — identity_id, old_key_hash, new_key_hash, timestamp |

**Protocol audit log format — one JSON object per line (JSON Lines):**

```json
{"ts":"2026-04-29T14:35:31.014Z","event_type":"membership.join","event_id":"xgen://hash/sha256:a3f9...","identity_id":"xgen://pubkey/ed25519:AAAA...","space_id":"xgen://hash/sha256:b2c3...","node_id":"xgen://pubkey/ed25519:CCCC..."}
```

Mandatory fields in every audit log entry:

| Field | Description |
|---|---|
| `ts` | RFC 3339 UTC timestamp — millisecond precision |
| `event_type` | The XGen EventType string |
| `event_id` | The XGen event_id hash URI — links back to the DAG |
| `node_id` | The Node that produced this audit entry |

Additional fields are EventType-specific as listed in the table above. The full Event is always recoverable from the DAG via `event_id` — the audit log records the summary facts, not the full Event payload.

**Protocol audit log location:** `audit/protocol_audit_YYYY-MM.jsonl` — one file per calendar month, in the Node's working directory. Monthly rotation keeps individual files manageable while maintaining long retention.

**Retention:** audit log files MUST NOT be automatically deleted by the Node. Deletion is an operator decision subject to applicable regulatory requirements. At Tier 1 and Tier 2, no minimum retention period is imposed by the protocol. At Tier 3 and Tier 4, regulatory minimums apply (see below).

---

**Auth Module audit log — Tier 3 requirement**

Tier 3 Auth Module operators MUST maintain an independent audit log of all verification decisions. This log lives inside the Auth Module, not the Node — the Node cannot access or verify it.

Required content per verification decision:

| Field | Description |
|---|---|
| `ts` | Timestamp of verification decision |
| `identity_id` | The XGen Identity ID of the subject |
| `verification_state` | The verification state assigned (A / B / C) |
| `kyc_level` | `standard` or `enhanced` |
| `watchlist_checked_at` | Timestamp of watchlist check |
| `watchlist_result` | `clear` or `match` |
| `operator_id` | The Auth Module operator's identifier |
| `assertion_id` | The issued Trust Assertion ID |
| `assertion_issued_at` | When the Trust Assertion was signed and delivered |
| `assertion_expires_at` | When the Trust Assertion expires |

**Retention:** minimum 7 years from the date of the verification decision (SOX §802, REF-05). Auth Module operators in banking contexts must also satisfy Basel II/III retention requirements (REF-06).

**Tamper evidence:** the Tier 3 Auth Module audit log SHOULD be stored in a write-once or cryptographically append-only system. A simple append-only flat file with periodic hash-chain checkpoints is acceptable for initial deployments. HSM-backed append-only logs are recommended for production.

---

**Auth Module audit log — Tier 4 requirement**

Tier 4 Auth Module operators MUST maintain an audit log that satisfies the following requirements beyond Tier 3:

| Requirement | Description |
|---|---|
| eIDAS LoA evidence | Record the specific evidence type used to establish LoA High (document type, biometric match result) |
| Government credential record | Record the credential type, issuing authority, and verification method |
| Clearance verification record | For government Spaces — record the clearance level verified, the authority that confirmed it, and the date |
| Data access log | For healthcare Spaces — log every access to identity data by any system or operator, per GDPR Art. 30 |
| Jurisdiction record | Record which jurisdiction's law governed the verification |
| Retention | Minimum 10 years for healthcare (REF-09, REF-10). Government retention is jurisdiction-defined. |

**Tamper evidence:** mandatory at Tier 4. The audit log MUST be stored in a cryptographically append-only system. A hash-chain where each entry includes the hash of the previous entry is the minimum acceptable implementation.

**Data localisation:** the Tier 4 audit log MUST be stored within the jurisdiction(s) declared in the Auth Module's `data_localisation` claim. Replication outside those jurisdictions is prohibited unless explicitly permitted by applicable law.

---

**Protocol audit log vs Auth Module audit log — relationship**

These are independent records. The Node's protocol audit log records what happened at the protocol level — Events in the DAG. The Auth Module's audit log records what the Auth Module did to verify an Identity before issuing a Trust Assertion. Both are needed for a complete compliance picture but neither replaces the other.

A compliance auditor examining a Tier 4 Space would consult:
1. The Node's protocol audit log — to establish who was a member, when they joined, when they left
2. The Auth Module's audit log — to establish how each member's identity was verified and under what authority

---

### 3.12 Space Migration Protocol

*Status: complete*

The protocol for migrating a Space from one Node (the **source Node**) to another (the **destination Node**). Migration transfers the full Event history, current state, membership, and federation relationships of the Space. The Space's identity — its `space_id` — is preserved unchanged across migration.

**Design principle:** migration is a protocol-level operation, not a data export/import. The migrated Space on the destination Node is cryptographically identical to the Space on the source Node — the same Event DAG, the same event_ids, the same signatures. No Events are re-signed or re-hashed. The Space's entire history is faithfully reproduced.

---

#### 3.12.1 Who Can Trigger Migration

Only the Space **owner** may initiate a migration. No other role — not admin, not moderator — has migration authority.

The owner must be authenticated on the **source Node** to initiate. The destination Node must be reachable and willing to accept the Space before migration begins — an acceptance handshake precedes all data transfer.

A Space may only be migrated to a Node that:
1. Is running a compatible protocol version
2. Has sufficient storage capacity for the Space's Event history
3. Explicitly accepts the migration (see 3.12.3)

---

#### 3.12.2 Migration State Machine

Migration proceeds through five states. Both the source and destination Node track this state independently.

```
IDLE → NEGOTIATING → TRANSFERRING → VERIFYING → COMPLETE
                                              ↓
                                          FAILED
```

| State | Description |
|---|---|
| `IDLE` | No migration in progress |
| `NEGOTIATING` | Destination Node evaluating acceptance; source awaiting confirmation |
| `TRANSFERRING` | Events being transferred from source to destination |
| `VERIFYING` | Destination verifying completeness and integrity of transferred Events |
| `COMPLETE` | Migration successful — Space live on destination Node |
| `FAILED` | Migration failed — Space remains on source Node, no partial state on destination |

During `TRANSFERRING` and `VERIFYING`, the source Node continues to serve the Space normally to existing members. New Events produced during migration are tracked and transferred as a tail batch (see 3.12.5).

---

#### 3.12.3 Migration Initiation Sequence

The owner initiates migration by sending a `migration.request` Event to the source Node. The source Node opens a federation connection to the destination Node and performs the acceptance handshake.

**`migration.request` — owner sends to source Node:**

```json
{
  "protocol_version": "0.1",
  "type": "migration.request",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "destination_node_id": "xgen://pubkey/ed25519:DDDD...",
  "destination_node_url": "wss://destination.example.com/xgen",
  "timestamp": "2026-04-30T10:00:00.000Z",
  "signature": "ed25519:AAAA...:base64url-signature"
}
```

| Field | Description |
|---|---|
| `space_id` | The Space to migrate |
| `destination_node_id` | The pubkey_uri of the destination Node |
| `destination_node_url` | The WebSocket endpoint of the destination Node |

**Source Node → Destination Node: `migration.propose`:**

```json
{
  "protocol_version": "0.1",
  "type": "migration.propose",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "source_node_id": "xgen://pubkey/ed25519:CCCC...",
  "space_auth_tier": 1,
  "event_count": 4821,
  "estimated_size_bytes": 2048576,
  "owner_id": "xgen://pubkey/ed25519:AAAA...",
  "timestamp": "2026-04-30T10:00:01.000Z",
  "signature": "ed25519:CCCC...:base64url-signature"
}
```

| Field | Description |
|---|---|
| `event_count` | Number of Events in the Space DAG at time of proposal |
| `estimated_size_bytes` | Estimated total byte size of all Events |
| `owner_id` | Identity of the Space owner who authorised migration |

**Destination Node → Source Node: `migration.accept` or `migration.reject`:**

```json
{
  "protocol_version": "0.1",
  "type": "migration.accept",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "timestamp": "2026-04-30T10:00:02.000Z",
  "signature": "ed25519:DDDD...:base64url-signature"
}
```

or:

```json
{
  "protocol_version": "0.1",
  "type": "migration.reject",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "reason": "insufficient_storage",
  "timestamp": "2026-04-30T10:00:02.000Z",
  "signature": "ed25519:DDDD...:base64url-signature"
}
```

Valid rejection reasons: `insufficient_storage`, `version_incompatible`, `policy_rejected`, `already_hosting`.

If the destination rejects, migration state returns to `IDLE` on both Nodes. The source Node notifies the owner with a `migration.failed` response. The Space continues unchanged on the source.

---

#### 3.12.4 Event Transfer

Once accepted, the source Node transfers all Events in the Space DAG to the destination Node in causal order — parents always before children. This is the same ordering as `transport.sync_response` (3.3.6).

Transfer uses a dedicated migration channel, not the standard federation transport, to avoid interfering with normal Space operation during the transfer.

**Transfer batch message:**

```json
{
  "protocol_version": "0.1",
  "type": "migration.event_batch",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "batch_index": 0,
  "events": [ /* array of full Event objects */ ],
  "batch_hash": "xgen://hash/sha256:e5f6a7b8...",
  "timestamp": "2026-04-30T10:00:03.000Z",
  "signature": "ed25519:CCCC...:base64url-signature"
}
```

| Field | Description |
|---|---|
| `batch_index` | Sequential batch number starting from 0 — enables gap detection |
| `events` | Array of full Event objects in causal order |
| `batch_hash` | SHA-256 of the concatenated event_ids in this batch — for integrity verification |

**Batch size:** implementation-defined, subject to the Tier message size ceiling (WD-01 through WD-04). Recommended: 100 Events per batch.

The destination Node validates each received Event using the standard 13-step validation pipeline (3.2.6) before acknowledging the batch. Events that fail validation cause migration to enter `FAILED` state — a migrated Space with invalid Events is not acceptable.

**Batch acknowledgement:**

```json
{
  "protocol_version": "0.1",
  "type": "migration.batch_ack",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "batch_index": 0,
  "timestamp": "2026-04-30T10:00:04.000Z",
  "signature": "ed25519:DDDD...:base64url-signature"
}
```

If a batch is not acknowledged within 30 seconds (WD-17), the source Node retransmits it. Maximum retransmits: 3. After 3 failures, migration enters `FAILED` state.

---

#### 3.12.5 Tail Batch — Events Produced During Transfer

The Space remains live during transfer. New Events produced by members during the transfer period must also be migrated. The source Node tracks all Events produced after the `migration.accept` message and transfers them as a tail batch after the main transfer completes.

There may be multiple tail rounds if the Space is very active. Each tail round follows the same batch protocol. The source Node signals the end of tail transfer with `migration.transfer_complete`:

```json
{
  "protocol_version": "0.1",
  "type": "migration.transfer_complete",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "total_events": 4847,
  "dag_tips": ["xgen://hash/sha256:f6a7b8c9...", "xgen://hash/sha256:a7b8c9d0..."],
  "timestamp": "2026-04-30T10:01:30.000Z",
  "signature": "ed25519:CCCC...:base64url-signature"
}
```

| Field | Description |
|---|---|
| `total_events` | Total number of Events transferred across all batches |
| `dag_tips` | The current DAG tip event_ids on the source Node — the destination must reach the same tips |

---

#### 3.12.6 Verification

On receiving `migration.transfer_complete`, the destination Node enters `VERIFYING` state and performs the following checks:

1. **Event count match** — total Events in destination DAG equals `total_events`
2. **DAG tip match** — destination DAG tips match the `dag_tips` array exactly
3. **State consistency** — replay the full Event log and confirm current state matches a fresh replay (no corruption)
4. **Membership integrity** — all Identities with active `membership.join` on source are present on destination

If all checks pass, the destination sends `migration.verified`:

```json
{
  "protocol_version": "0.1",
  "type": "migration.verified",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "timestamp": "2026-04-30T10:01:35.000Z",
  "signature": "ed25519:DDDD...:base64url-signature"
}
```

If any check fails, the destination sends `migration.verification_failed` with a reason string. Migration enters `FAILED` state. The destination Node discards all transferred Events. The source Node continues to serve the Space unchanged.

---

#### 3.12.7 Cutover and Member Notification

After `migration.verified` is received, the source Node performs the cutover:

1. Source Node produces a `state.space_migrate` Event in the Space DAG recording the migration:

```json
{
  "protocol_version": "0.1",
  "type": "state.space_migrate",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "source_node_id": "xgen://pubkey/ed25519:CCCC...",
  "destination_node_id": "xgen://pubkey/ed25519:DDDD...",
  "destination_node_url": "wss://destination.example.com/xgen",
  "migrated_at": "2026-04-30T10:01:36.000Z",
  "timestamp": "2026-04-30T10:01:36.000Z",
  "signature": "ed25519:CCCC...:base64url-signature"
}
```

2. Source Node transfers this final Event to the destination Node
3. Destination Node activates the Space — it is now live and accepting connections
4. Source Node notifies all currently connected members via `transport.redirect`:

```json
{
  "protocol_version": "0.1",
  "type": "transport.redirect",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "new_node_url": "wss://destination.example.com/xgen",
  "new_node_id": "xgen://pubkey/ed25519:DDDD...",
  "timestamp": "2026-04-30T10:01:37.000Z",
  "signature": "ed25519:CCCC...:base64url-signature"
}
```

5. Clients receiving `transport.redirect` disconnect from the source and reconnect to the destination Node automatically
6. Source Node stops accepting new connections for this Space
7. Migration state on both Nodes transitions to `COMPLETE`

---

#### 3.12.8 Federation Re-establishment

After cutover, the destination Node must re-establish federation with all Nodes that were previously federated with the Space on the source Node. The list of federated Nodes is recoverable from the `state.federation_add` Events in the migrated DAG.

The destination Node initiates a standard federation handshake (3.4) with each previously federated Node. Those Nodes update their federation registry to point to the destination Node for this Space.

The source Node sends `migration.federation_notify` to each previously federated peer as a courtesy, informing them of the new home:

```json
{
  "protocol_version": "0.1",
  "type": "migration.federation_notify",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "new_node_id": "xgen://pubkey/ed25519:DDDD...",
  "new_node_url": "wss://destination.example.com/xgen",
  "timestamp": "2026-04-30T10:01:38.000Z",
  "signature": "ed25519:CCCC...:base64url-signature"
}
```

Federated peers that receive this message update their records and initiate a federation handshake with the destination Node directly.

---

#### 3.12.9 Source Node Decommission

After migration is `COMPLETE`, the source Node:

1. Keeps the Space's SQLite database read-only for a configurable grace period (WD-18, default 30 days) — to serve any lagging clients that did not receive the redirect
2. Responds to any connection attempt for this Space with a `transport.redirect` pointing to the destination Node
3. After the grace period, the operator may safely delete the Space database from the source Node

The source Node MUST NOT delete the Space database immediately on migration completion — lagging members or offline clients may reconnect and need the redirect.

---

#### 3.12.10 Migration EventType Registry Additions

The following EventTypes are added to the Phase 2 registry (extending 3.2.2):

*Migration events:*

| EventType | Description |
|---|---|
| `migration.request` | Owner requests migration of a Space to a new Node |
| `migration.propose` | Source Node proposes migration to destination Node |
| `migration.accept` | Destination Node accepts the migration proposal |
| `migration.reject` | Destination Node rejects the migration proposal |
| `migration.failed` | Source Node notifies owner that migration failed (rejection or timeout) |
| `migration.event_batch` | Batch of Events transferred from source to destination |
| `migration.batch_ack` | Destination acknowledges a received batch |
| `migration.transfer_complete` | Source signals end of Event transfer with DAG tips |
| `migration.verified` | Destination confirms successful verification |
| `migration.verification_failed` | Destination reports verification failure |
| `migration.federation_notify` | Source notifies federated peers of new home Node |
| `state.space_migrate` | Permanent DAG record of a completed migration |
| `transport.redirect` | Node instructs clients to reconnect to a new Node URL |

---

#### 3.12.11 Migration Error Codes

Migration failures use the 6000 error code range.

| Code | Error string | Meaning |
|---|---|---|
| 6001 | `migration_not_authorised` | Requestor is not the Space owner |
| 6002 | `migration_rejected_storage` | Destination rejected — insufficient storage |
| 6003 | `migration_rejected_version` | Destination rejected — incompatible protocol version |
| 6004 | `migration_rejected_policy` | Destination rejected — operator policy |
| 6005 | `migration_rejected_duplicate` | Destination already hosting this Space |
| 6006 | `migration_batch_timeout` | Batch not acknowledged within timeout window |
| 6007 | `migration_verification_failed` | Destination verification failed — event count, tips, or state mismatch |
| 6008 | `migration_in_progress` | Migration already in progress for this Space |

**Display rule** — same pattern as all other error ranges:

```
Error 6007 (migration_verification_failed): The destination Node could not verify
the migrated Space. The Space remains on the source Node. Check destination Node
logs for the specific verification step that failed.
```

**Work definitions added:**

| # | Value | Current setting | Location | Review trigger |
|---|---|---|---|---|
| WD-17 | Migration batch acknowledgement timeout | 30 seconds | 3.12.4 | Observe transfer performance in first migration test |
| WD-18 | Source Node grace period after migration | 30 days | 3.12.9 | Review with first production migration |
| WD-19 | Replication factor N | 3 | 3.13.2 | Review at 100+ Node network scale |
| WD-20 | Replica acknowledgement timeout | 30 seconds | 3.13.5 | Observe in first multi-Node deployment |
| WD-21 | Stale replica retry interval | 24 hours | 3.13.5 | Review with first production deployment |
| WD-22 | Replica refresh interval | 7 days | 3.13.6 | Review with first production deployment |
| WD-23 | Replica record TTL | 90 days | 3.13.7 | Review with first production deployment |
| WD-24 | Bootstrap directory maximum age | 1 hour | 3.14.2 | Review with first production Bootstrap Node deployment |
| WD-25 | Bootstrap directory entry TTL | 7 days | 3.14.3 | Review with first production Bootstrap Node deployment |
| WD-26 | Isolated mode Bootstrap retry interval | 10 minutes | 3.14.6 | Review under real network partition conditions |


---

### 3.13 Identity Replication Parameters

*Status: complete*

Identity records are replicated across multiple Nodes to ensure availability when a home Node is temporarily or permanently offline. Replication is passive and pull-based — replica Nodes hold copies of Identity records, but the home Node is always the authoritative source. Replicas serve read requests only; Identity updates always originate on the home Node.

---

#### 3.13.1 Replication Model

When an Identity registers on its home Node (3.6), the home Node propagates the Identity record to a set of **replica Nodes**. Clients can retrieve an Identity record from any replica when the home Node is unreachable — for example, to verify a signature on a received Event from that Identity.

**What is replicated:** the Identity record as defined in 3.6.6 — the public key, display name, Trust Assertion reference, and metadata. Private key material never leaves the client and is never replicated anywhere.

**Replication is not federation:** replication of Identity records is distinct from Space federation (3.4). A Node may hold a replica of an Identity record without having any federation relationship with that Identity's home Node.

**Authority:** the home Node is always the authoritative source for an Identity record. If a replica and the home Node disagree on an Identity record, the home Node wins. The `update_version` monotonic counter (3.6.7) resolves conflicts deterministically — higher `update_version` wins.

---

#### 3.13.2 Replication Factor (N)

The replication factor **N** is the number of replica Nodes a new Identity record is propagated to on registration.

**N = 3** (work definition, WD-19)

Rationale: N=3 provides availability against single-Node failure while remaining tractable for small networks. In a network with fewer than 3 Nodes beyond the home Node, the home Node propagates to all available Nodes. N is a network-wide parameter, not configurable per Identity or per Space.

The home Node itself is not counted in N — the Identity always exists on the home Node. So the total number of Nodes holding an Identity record is N+1 at full replication.

---

#### 3.13.3 Replica Node Selection

The home Node selects replica Nodes from its known Node registry (populated via node announcements, 3.5). Selection criteria in order of preference:

1. **Geographically diverse** — prefer Nodes in different network regions if region information is available in the node announcement
2. **High availability** — prefer Nodes with a recent announcement timestamp (not stale)
3. **Not already a replica** — do not re-select an existing replica for the same Identity
4. **Random from remaining candidates** — break ties randomly to prevent hotspots

If fewer than N suitable Nodes are known, the home Node propagates to all known Nodes and retries additional replication as new Nodes are discovered via announcements.

---

#### 3.13.4 Replication Wire Protocol

The home Node pushes Identity records to replica Nodes using a `identity.replicate` message over a standard WebSocket connection:

```json
{
  "protocol_version": "0.1",
  "type": "identity.replicate",
  "identity_id": "xgen://pubkey/ed25519:AAAA...",
  "identity_record": { /* full Identity record per 3.6.6 */ },
  "update_version": 1,
  "timestamp": "2026-04-30T10:00:00.000Z",
  "signature": "ed25519:AAAA...:base64url-signature"
}
```

| Field | Description |
|---|---|
| `identity_id` | The Identity being replicated |
| `identity_record` | Full Identity record — same format as stored on home Node |
| `update_version` | Monotonic counter — replica MUST reject if lower than its stored version |
| `signature` | Signed by the Identity's own keypair — proves authenticity |

The receiving (replica) Node responds with `identity.replicate_ack` on success or an error code on failure:

```json
{
  "protocol_version": "0.1",
  "type": "identity.replicate_ack",
  "identity_id": "xgen://pubkey/ed25519:AAAA...",
  "update_version": 1,
  "timestamp": "2026-04-30T10:00:01.000Z",
  "signature": "ed25519:BBBB...:base64url-signature"
}
```

A replica Node that receives an `identity.replicate` with a lower `update_version` than its stored record MUST reject it with error `3020 identity_version_stale` and reply with its stored `update_version` so the home Node can update its replication state.

---

#### 3.13.5 Update Propagation

When an Identity record is updated on the home Node (display name change, Trust Assertion renewal, key rotation), the home Node pushes the updated record to all current replicas using the same `identity.replicate` message with an incremented `update_version`.

Update propagation is **best-effort with retry.** The home Node:
1. Sends `identity.replicate` to all known replicas
2. Waits for `identity.replicate_ack` from each replica
3. For replicas that do not acknowledge within 30 seconds (WD-20), retries up to 3 times
4. After 3 failures, marks the replica as stale in its replication registry
5. Attempts to find a replacement replica from the Node registry and propagates to it

A replica marked as stale is not immediately dropped — it may recover. The home Node retries stale replicas periodically (WD-21, default 24 hours).

---

#### 3.13.6 Replica Refresh (Anti-Entropy)

Replica Nodes periodically verify their stored Identity records are current by querying the home Node. This prevents long-term divergence due to missed updates.

**Refresh interval:** WD-22, default 7 days.

Refresh procedure:
1. The replica Node sends `identity.refresh_query` to the home Node carrying the `identity_id` and its stored `update_version`
2. If the home Node has a newer version, it sends `identity.replicate` with the current record
3. If versions match, the home Node sends `identity.refresh_ack` with no payload — replica is current

```json
{
  "protocol_version": "0.1",
  "type": "identity.refresh_query",
  "identity_id": "xgen://pubkey/ed25519:AAAA...",
  "stored_version": 3,
  "timestamp": "2026-04-30T10:00:00.000Z",
  "signature": "ed25519:BBBB...:base64url-signature"
}
```

If the home Node is unreachable during a refresh attempt, the replica retains its stored record. Replicas do not delete Identity records due to home Node unreachability — the record may still be useful to clients even if stale.

---

#### 3.13.7 Replica Record TTL

Replica records expire after **90 days** without a successful refresh (WD-23). An expired replica record MUST NOT be served to clients. The replica Node should attempt a refresh before expiry and, if the home Node remains unreachable, log the expiry and remove the record.

Rationale: holding replica records indefinitely would cause storage growth from abandoned Identities. The 90-day TTL is longer than the refresh interval (7 days) by a factor sufficient to survive extended home Node outages.

---

#### 3.13.8 Orphaned Identity Recovery

An Identity is **orphaned** when its home Node is permanently lost — hardware failure, operator abandonment, or deliberate shutdown with no migration. The Identity's private key still exists on the client device, but there is no home Node to serve or update the Identity record.

**Recovery procedure:**

1. The client contacts any available replica Node and requests the current Identity record
2. The client selects a new home Node — any Node willing to accept registrations
3. The client registers on the new Node using its existing keypair — same `identity_id`, same public key. The registration is identical to a fresh registration (3.6) except the `re_registration: true` flag is set in the registration request
4. The new Node verifies the keypair ownership via the standard challenge-response, stores the record, and begins propagating it to N new replicas
5. The client sends `identity.home_changed` notifications to all Nodes it knows were previously in contact with the Identity (via federation peers, Space memberships, etc.):

```json
{
  "protocol_version": "0.1",
  "type": "identity.home_changed",
  "identity_id": "xgen://pubkey/ed25519:AAAA...",
  "old_home_node_id": "xgen://pubkey/ed25519:CCCC...",
  "new_home_node_id": "xgen://pubkey/ed25519:DDDD...",
  "new_home_node_url": "wss://newnode.example.com/xgen",
  "update_version": 5,
  "timestamp": "2026-04-30T10:00:00.000Z",
  "signature": "ed25519:AAAA...:base64url-signature"
}
```

**Key continuity:** because the `identity_id` is the pubkey_uri of the Identity's Ed25519 keypair, re-registration on a new Node produces the same `identity_id`. All Events previously signed by this Identity remain valid — their signatures are verifiable against the same public key. The orphaned Identity is not lost; it is re-homed.

**Trust Assertion continuity:** if the Identity held a valid Trust Assertion, it remains valid until its TTL expires. The new home Node stores the existing Trust Assertion. Re-verification is not required unless the TTL expires before re-registration completes.

---

#### 3.13.9 Replication EventType Registry Additions

*Identity replication events:*

| EventType | Description |
|---|---|
| `identity.replicate` | Home Node pushes Identity record to a replica Node |
| `identity.replicate_ack` | Replica Node acknowledges successful replication |
| `identity.refresh_query` | Replica queries home Node for current record version |
| `identity.refresh_ack` | Home Node confirms replica is current |
| `identity.home_changed` | Identity notifies network of new home Node after orphan recovery |

---

#### 3.13.10 Replication Error Codes

Replication failures extend the 3000 identity error code range:

| Code | Error string | Meaning |
|---|---|---|
| 3020 | `identity_version_stale` | Received `identity.replicate` has lower `update_version` than stored — rejected |
| 3021 | `identity_replica_full` | Node cannot accept further Identity replicas — storage limit reached |
| 3022 | `identity_home_node_mismatch` | Re-registration keypair does not match stored Identity record |
| 3023 | `identity_not_found` | Requested Identity record not found on this Node |

**Work definitions added:**

| # | Value | Current setting | Location | Review trigger |
|---|---|---|---|---|
| WD-19 | Replication factor N | 3 | 3.13.2 | Review at 100+ Node network scale |
| WD-20 | Replica acknowledgement timeout | 30 seconds | 3.13.5 | Observe in first multi-Node deployment |
| WD-21 | Stale replica retry interval | 24 hours | 3.13.5 | Review with first production deployment |
| WD-22 | Replica refresh interval | 7 days | 3.13.6 | Review with first production deployment |
| WD-23 | Replica record TTL | 90 days | 3.13.7 | Review with first production deployment |

---

### 3.14 Bootstrap Node Protocol

*Status: complete*

Bootstrap Nodes are well-known, publicly reachable XGen Nodes that help new Nodes discover and join the network. A new Node has no prior knowledge of the network — Bootstrap Nodes are the entry point. They maintain a directory of known Nodes and respond to discovery queries.

**Bootstrap Nodes are ordinary XGen Nodes** with one additional capability declared in their announcement: `xgen.bootstrap`. They run the same software as any other Node. There is no special binary, no privileged protocol position, and no centralised Bootstrap Node authority. Any Node operator may run a Bootstrap Node by declaring the capability.

**The XGen Foundation operates reference Bootstrap Nodes** for the initial network, but these are not the only valid Bootstrap Nodes and should not be the only ones listed in any production deployment.

---

#### 3.14.1 Bootstrap Node Capability

A Node declares Bootstrap capability via the `capabilities` field in its node announcement (3.5.2):

```json
"capabilities": ["xgen.bootstrap"]
```

A Bootstrap Node additionally announces a `bootstrap_info` field in its announcement:

```json
{
  "bootstrap_info": {
    "directory_url": "https://bootstrap.example.com/xgen-directory",
    "accepts_registrations": true,
    "region": "EU",
    "operator": "Example Foundation"
  }
}
```

| Field | Description |
|---|---|
| `directory_url` | HTTPS URL where the Bootstrap Node's directory can be queried |
| `accepts_registrations` | Whether this Bootstrap Node accepts new Node registrations |
| `region` | Geographic region for diversity routing — operator-declared |
| `operator` | Human-readable operator name — informational only |

---

#### 3.14.2 Bootstrap Node Directory Format

The Bootstrap Node maintains a directory of known Nodes. The directory is served over HTTPS as a JSON document at the `directory_url` declared in the announcement.

**Directory document format:**

```json
{
  "protocol_version": "0.1",
  "bootstrap_node_id": "xgen://pubkey/ed25519:BBBB...",
  "generated_at": "2026-04-30T10:00:00.000Z",
  "nodes": [
    {
      "node_id": "xgen://pubkey/ed25519:CCCC...",
      "endpoint": "wss://node1.example.com/xgen",
      "region": "EU",
      "last_seen": "2026-04-30T09:55:00.000Z",
      "reputation_score": 0.92
    },
    {
      "node_id": "xgen://pubkey/ed25519:DDDD...",
      "endpoint": "wss://node2.example.com/xgen",
      "region": "NA",
      "last_seen": "2026-04-30T09:58:00.000Z",
      "reputation_score": 0.87
    }
  ],
  "signature": "ed25519:BBBB...:base64url-signature"
}
```

| Field | Description |
|---|---|
| `bootstrap_node_id` | The Bootstrap Node's own pubkey_uri — for directory authenticity verification |
| `generated_at` | When this directory snapshot was produced |
| `nodes` | Array of known Nodes, ordered by `reputation_score` descending |
| `reputation_score` | Float 0.0–1.0 — see 3.15 for format |
| `signature` | Signed by the Bootstrap Node's keypair over the canonical form of this document |

A Node consuming a directory document MUST verify the `signature` before trusting any entry. The signing key is the Bootstrap Node's `node_id` pubkey, which was obtained from the Bootstrap Node's announcement or from the hardcoded trust list (3.14.5).

**Directory freshness:** directory documents have a maximum age of **1 hour** (WD-24). A consuming Node MUST NOT use a directory document older than this. The Bootstrap Node SHOULD regenerate the directory at least every 30 minutes.

---

#### 3.14.3 New Node Registration with Bootstrap Nodes

When a new Node starts for the first time, it registers itself with one or more Bootstrap Nodes to become discoverable by other Nodes.

**Registration request — sent over WebSocket to the Bootstrap Node:**

```json
{
  "protocol_version": "0.1",
  "type": "bootstrap.register",
  "node_id": "xgen://pubkey/ed25519:NNNN...",
  "endpoint": "wss://newnode.example.com/xgen",
  "region": "EU",
  "capabilities": ["xgen.federation"],
  "timestamp": "2026-04-30T10:00:00.000Z",
  "signature": "ed25519:NNNN...:base64url-signature"
}
```

The Bootstrap Node verifies the signature (proves the registrant holds the private key for `node_id`), adds the Node to its directory, and responds:

```json
{
  "protocol_version": "0.1",
  "type": "bootstrap.register_ack",
  "node_id": "xgen://pubkey/ed25519:NNNN...",
  "directory_url": "https://bootstrap.example.com/xgen-directory",
  "timestamp": "2026-04-30T10:00:01.000Z",
  "signature": "ed25519:BBBB...:base64url-signature"
}
```

**Registration TTL:** a Bootstrap Node directory entry expires after **7 days** (WD-25) without a re-registration or keepalive ping. Nodes MUST re-register before expiry. A Node that fails to re-register is removed from the directory quietly — no notification is sent.

**Registration is not mandatory for operation:** a Node that does not register with any Bootstrap Node can still operate if it has direct peer connections. Bootstrap registration is required only for discoverability by new, unknown Nodes.

---

#### 3.14.4 Directory Query Protocol

A new Node queries a Bootstrap Node's directory to discover peers to connect with.

**Query — HTTP GET request to `directory_url` with optional filters:**

```
GET https://bootstrap.example.com/xgen-directory?region=EU&min_reputation=0.7&limit=10
```

| Query parameter | Description |
|---|---|
| `region` | Filter by geographic region — optional |
| `min_reputation` | Minimum reputation score — optional, float 0.0–1.0 |
| `limit` | Maximum number of Nodes to return — optional, default 20, max 100 |
| `exclude` | Comma-separated list of `node_id` values to exclude — for nodes already known |

The Bootstrap Node returns a filtered directory document in the format described in 3.14.2.

**A new Node's bootstrap sequence:**
1. Fetch directory from one or more known Bootstrap Nodes
2. Verify directory document signatures
3. Connect to 3–5 Nodes from the directory (prefer high reputation, diverse regions)
4. Perform standard federation handshakes (3.4) with each
5. Learn additional Nodes via node announcements received from new peers
6. Register itself with the Bootstrap Node(s) if `accepts_registrations: true`

---

#### 3.14.5 Bootstrap Node Trust at First Run

A new Node has no prior knowledge of the network. It must be configured with at least one trusted Bootstrap Node to begin the discovery sequence. Trust is established via one of three mechanisms, applied in order of preference:

**Mechanism 1 — Hardcoded Foundation Bootstrap Nodes:**
The XGen reference implementation ships with a hardcoded list of Foundation-operated Bootstrap Node IDs and endpoints. These are compiled into the binary and verified by their pubkey_uri. If a hardcoded endpoint responds and its `node_id` matches the compiled value, it is trusted.

The hardcoded list is updated via protocol version upgrades. It is not modifiable at runtime — this is intentional, to prevent operators from accidentally trusting malicious Bootstrap Nodes via misconfiguration.

**Mechanism 2 — Operator-configured Bootstrap Nodes:**
Operators may specify additional trusted Bootstrap Nodes in `xgen-node_config.toml`:

```toml
[bootstrap]
trusted_nodes = [
  { node_id = "xgen://pubkey/ed25519:BBBB...", endpoint = "wss://bootstrap.example.com/xgen" },
  { node_id = "xgen://pubkey/ed25519:CCCC...", endpoint = "wss://bootstrap.other.com/xgen" }
]
```

The `node_id` in the config is the trust anchor — the operator must obtain this out-of-band and include it explicitly. A Bootstrap Node that presents a different `node_id` than configured is rejected.

**Mechanism 3 — Manually provided peer address:**
The operator provides a known peer Node's endpoint directly via CLI at first run. The Node connects to this peer, performs a standard federation handshake, and discovers Bootstrap Nodes via the peer's node announcements.

**All three mechanisms produce the same result:** a verified connection to at least one network peer. Once connected, the standard node announcement and federation mechanisms take over.

---

#### 3.14.6 Bootstrap Node Failure Handling

If all configured Bootstrap Nodes are unreachable at startup:

1. The Node logs a warning and retries with exponential backoff (same parameters as transport reconnection, 3.3.6)
2. After 5 failed attempts across all configured Bootstrap Nodes, the Node enters **isolated mode** — it operates normally for existing connections and local clients but cannot discover new peers
3. In isolated mode, the Node continues to retry Bootstrap Node connections in the background (WD-26, default 10 minutes between retry batches)
4. If a peer connects to this Node directly (e.g. a peer that already knows this Node's address), the Node exits isolated mode and resumes normal federation

**Isolated mode is not a failure state.** A Node in isolated mode is fully functional for its existing members and existing federation relationships. It simply cannot discover new peers until Bootstrap Node connectivity is restored.

---

#### 3.14.7 Bootstrap Node EventType Registry Additions

*Bootstrap protocol messages:*

| EventType | Description |
|---|---|
| `bootstrap.register` | New Node registers with a Bootstrap Node |
| `bootstrap.register_ack` | Bootstrap Node acknowledges registration |
| `bootstrap.keepalive` | Node pings Bootstrap Node to refresh its directory entry before TTL expiry |
| `bootstrap.keepalive_ack` | Bootstrap Node acknowledges keepalive and resets TTL |
| `bootstrap.deregister` | Node explicitly removes itself from a Bootstrap Node's directory |

---

#### 3.14.8 Bootstrap Node Error Codes

Bootstrap protocol failures use the 7000 error code range.

| Code | Error string | Meaning |
|---|---|
| 7001 | `bootstrap_registration_rejected` | Bootstrap Node declined to register this Node — policy or capacity |
| 7002 | `bootstrap_directory_stale` | Directory document age exceeds maximum freshness window (WD-24) |
| 7003 | `bootstrap_signature_invalid` | Directory document signature verification failed |
| 7004 | `bootstrap_node_not_found` | Queried node_id not present in directory |
| 7005 | `bootstrap_isolated_mode` | Node is in isolated mode — no Bootstrap Nodes reachable |

**Work definitions added:**

| # | Value | Current setting | Location | Review trigger |
|---|---|---|---|---|
| WD-24 | Bootstrap directory maximum age | 1 hour | 3.14.2 | Review with first production Bootstrap Node deployment |
| WD-25 | Bootstrap directory entry TTL | 7 days | 3.14.3 | Review with first production Bootstrap Node deployment |
| WD-26 | Isolated mode Bootstrap retry interval | 10 minutes | 3.14.6 | Review under real network partition conditions |

---

### 3.15 Node Reputation Format

*Status: complete*

Node reputation is a soft signal maintained by Bootstrap Nodes that expresses how reliably a Node has behaved on the network over time. It is not a trust or authentication mechanism — it does not replace keypair verification or Trust Assertions. It is a quality-of-service signal that helps new Nodes prioritise which peers to connect to first, and helps Bootstrap Nodes order their directory listings.

**Reputation is non-binding.** No protocol action is gated on reputation score. A Node with a low reputation score can still federate, accept members, and serve Spaces. Reputation only affects directory ordering and peer selection hints.

---

#### 3.15.1 Reputation Signal Structure

Each Bootstrap Node maintains a reputation record per known Node. The reputation record is not a single number but a set of weighted signal components that combine into a final score.

**Reputation record format (internal to Bootstrap Node, not a wire message):**

The `score` field (float 0.0–1.0) is what Bootstrap Nodes expose as `reputation_score` in their directory documents (3.14.2). The `components` breakdown is internal only — it is not transmitted in the directory.

```json
{
  "node_id": "xgen://pubkey/ed25519:CCCC...",
  "score": 0.87,
  "components": {
    "uptime_ratio": 0.95,
    "announcement_freshness": 0.90,
    "defederation_count": 0,
    "successful_federations": 142,
    "failed_federations": 3,
    "protocol_violations": 0,
    "last_updated": "2026-04-30T10:00:00.000Z"
  }
}
```

| Component | Description | Weight |
|---|---|---|
| `uptime_ratio` | Fraction of keepalive pings responded to over the observation window | 0.35 |
| `announcement_freshness` | How recently the Node re-announced itself (1.0 = within 24h, decays to 0.0 at 90 days) | 0.25 |
| `defederation_count` | Number of times other Nodes have sent a defederation signal against this Node (see 3.15.3) — higher count lowers score | 0.20 |
| `successful_federations` | Count of successful federation handshakes observed or reported | 0.10 |
| `failed_federations` | Count of failed federation handshakes — ratio to successful reduces score | 0.10 |
| `protocol_violations` | Count of verified protocol violations reported by other Nodes | bonus penalty |

**Score computation:** `score = sum(component_value × weight)` clamped to `[0.0, 1.0]`. Each component value is normalised to `[0.0, 1.0]` before weighting. `protocol_violations` applies a flat penalty of 0.1 per verified violation, applied after the weighted sum.

**Component weights are work definitions** (WD-27) — the values above are initial estimates pending calibration against real network behaviour.

---

#### 3.15.2 Reputation Propagation

Bootstrap Nodes share reputation signals with each other to build a network-wide view rather than a single-Bootstrap-Node view.

**Propagation mechanism:** each Bootstrap Node periodically broadcasts its reputation records to all other Bootstrap Nodes it knows. The receiving Bootstrap Node merges the incoming records with its own using a weighted average that favours more recent observations.

**Merge rule:** for each component, the merged value is:
```
merged = (local_value × local_weight) + (remote_value × remote_weight)
```
where `local_weight = 0.6` and `remote_weight = 0.4` (WD-28). The local Bootstrap Node's observation is given more weight because it has direct visibility of the Node's behaviour.

**Propagation interval:** WD-29, default 6 hours. Reputation records are not real-time — they are a slow-moving quality signal. Hourly propagation would create unnecessary traffic; daily propagation would make the signal too stale for useful peer selection.

**Propagation scope:** reputation propagation occurs only between Bootstrap Nodes. Regular Nodes do not participate in reputation propagation. The reputation system is Bootstrap-layer infrastructure.

---

#### 3.15.3 Defederation Signal Integration

When a Node defederates from a peer — removes it from its federation registry and ceases federation for a Space — it may optionally submit a **defederation signal** to its registered Bootstrap Nodes. This signal contributes to the `defederation_count` component of the defederated Node's reputation record.

**Defederation signal message — sent to Bootstrap Node over WebSocket:**

```json
{
  "protocol_version": "0.1",
  "type": "reputation.defederation_signal",
  "reporting_node_id": "xgen://pubkey/ed25519:AAAA...",
  "defederated_node_id": "xgen://pubkey/ed25519:CCCC...",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "reason": "repeated_protocol_violations",
  "evidence_event_ids": [
    "xgen://hash/sha256:d4e5f6a7...",
    "xgen://hash/sha256:e5f6a7b8..."
  ],
  "timestamp": "2026-04-30T10:00:00.000Z",
  "signature": "ed25519:AAAA...:base64url-signature"
}
```

| Field | Description |
|---|---|
| `reporting_node_id` | The Node submitting the signal — signed by this Node's keypair |
| `defederated_node_id` | The Node being reported |
| `space_id` | The Space in which the defederation occurred |
| `reason` | Human-readable reason string — not machine-parsed by the Bootstrap Node |
| `evidence_event_ids` | Optional array of event_ids that constitute evidence — verifiable from the DAG |

**Defederation signals are advisory only.** The Bootstrap Node records the signal and increments the `defederation_count` for the reported Node. It does not remove the Node from its directory or take any protocol action. A Node receiving many defederation signals will see its score fall and appear lower in directory listings — but is not excluded.

**Signal verification:** the Bootstrap Node verifies the signature on the signal (proves the reporting Node holds its keypair) but does not verify the evidence_event_ids — that would require fetching Events from the DAG, which is out of scope for Bootstrap Node operation. Evidence is provided for human review by Bootstrap Node operators, not for automated action.

**Abuse prevention:** a Node that submits defederation signals for many other Nodes in a short window is rate-limited by the Bootstrap Node (WD-30, default 10 signals per 24 hours per reporting Node). Excessive signal submission suggests manipulation rather than genuine defederation activity.

---

#### 3.15.4 Privacy Considerations

Reputation signals reveal information about federation history. A Bootstrap Node directory entry shows:
- That a Node exists and is publicly reachable
- Its reputation score (a float)
- When it was last seen

A defederation signal reveals:
- That Node A and Node B had a federation relationship
- That Node A chose to end it
- Which Space it involved

**Operators who value privacy of federation relationships should not submit defederation signals.** Submitting a signal is always voluntary. Defederation itself (removing a peer from the federation registry) is a local operation that produces no network-visible signal unless the operator chooses to report it.

**Reputation scores do not reveal Space contents, member lists, or message history.** They are behavioural metadata about the Node as an infrastructure participant, not about the people using it.

---

#### 3.15.5 Reputation EventType Registry Additions

*Reputation signals:*

| EventType | Description |
|---|---|
| `reputation.defederation_signal` | Node submits a defederation signal to a Bootstrap Node |
| `reputation.violation_report` | Node reports a verified protocol violation by a peer to a Bootstrap Node |

---

#### 3.15.6 Reputation Error Codes

Reputation signal failures use the 8000 error code range.

| Code | Error string | Meaning |
|---|---|
| 8001 | `reputation_signal_rate_limited` | Reporting Node has exceeded the signal submission rate limit |
| 8002 | `reputation_signal_invalid_signature` | Defederation signal signature verification failed |
| 8003 | `reputation_node_unknown` | Reported node_id not known to this Bootstrap Node |

**Work definitions added:**

| # | Value | Current setting | Location | Review trigger |
|---|---|---|---|---|
| WD-27 | Reputation component weights | As table in 3.15.1 | 3.15.1 | Calibrate after 6 months of production network data |
| WD-28 | Local vs remote reputation merge weight | 0.6 / 0.4 | 3.15.2 | Calibrate after Bootstrap Node network reaches 5+ nodes |
| WD-29 | Reputation propagation interval | 6 hours | 3.15.2 | Review with first multi-Bootstrap-Node deployment |
| WD-30 | Defederation signal rate limit | 10 per 24 hours | 3.15.3 | Review with first production deployment |
| WD-31 | DM promotion proposal timeout | 7 days | 3.16.6 | Review with first production deployment |

---

### 3.16 DM Space Promotion Sequence

*Status: complete*

A DM Space (`state.dm_space_create`) is a two-member Space with a single auto-created Room. It operates under constraints that reflect its private, bilateral nature. Promotion lifts those constraints and converts the DM Space into a full Space capable of hosting multiple members, multiple Rooms, and federation.

**Promotion is irreversible.** A promoted Space cannot be demoted back to DM status. This is intentional — once a Space has additional members or Rooms, the DM constraint cannot be meaningfully re-applied.

---

#### 3.16.1 DM Space Constraints

A DM Space has the following constraints that are not present on full Spaces:

| Constraint | Description |
|---|---|
| Maximum members | 2 — the two original participants |
| Maximum Rooms | 1 — auto-created at DM Space creation |
| Federation | Disabled — DM Spaces do not federate |
| Invitations | Disabled — no third party may be invited |
| Space visibility | Private — not discoverable via Bootstrap Node directory |

These constraints are enforced by the Node. Any Event that would violate them (e.g. a `membership.invite` in a DM Space) is rejected with the appropriate error code.

---

#### 3.16.2 Who Can Initiate Promotion

Either member of the DM Space may initiate promotion. Both members must consent — promotion requires a two-step approval sequence. The initiating member proposes; the other member confirms.

---

#### 3.16.3 Promotion Sequence

**Step 1 — Initiating member sends `dm.promote_propose`:**

```json
{
  "protocol_version": "0.1",
  "type": "dm.promote_propose",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "proposed_name": "Our Project",
  "timestamp": "2026-04-30T10:00:00.000Z",
  "signature": "ed25519:AAAA...:base64url-signature"
}
```

| Field | Description |
|---|---|
| `proposed_name` | The display name the Space will carry after promotion — both members can see and agree to this before confirming |

**Step 2 — Node delivers `dm.promote_propose` to the other member's connected client.**

**Step 3 — The other member sends `dm.promote_confirm` or `dm.promote_reject`:**

```json
{
  "protocol_version": "0.1",
  "type": "dm.promote_confirm",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "timestamp": "2026-04-30T10:00:30.000Z",
  "signature": "ed25519:BBBB...:base64url-signature"
}
```

If the other member sends `dm.promote_reject`, the promotion is cancelled. The DM Space continues unchanged. The proposing member is notified.

**Step 4 — Node produces `state.dm_promote` Event in the Space DAG:**

```json
{
  "protocol_version": "0.1",
  "type": "state.dm_promote",
  "space_id": "xgen://hash/sha256:b2c3d4e5...",
  "proposed_by": "xgen://pubkey/ed25519:AAAA...",
  "confirmed_by": "xgen://pubkey/ed25519:BBBB...",
  "new_name": "Our Project",
  "promoted_at": "2026-04-30T10:00:31.000Z",
  "timestamp": "2026-04-30T10:00:31.000Z",
  "signature": "ed25519:node_keypair...:base64url-signature"
}
```

The `state.dm_promote` Event is signed by the **Node**, not by either member — it is a protocol state change, not a member action. Both member signatures are referenced via `proposed_by` and `confirmed_by`.

**Step 5 — Node lifts DM constraints.** Immediately after committing `state.dm_promote` to the DAG:
- Maximum member count removed
- Maximum Room count removed
- Invitations enabled
- Federation enabled
- Space name updated to `new_name`

**Step 6 — Both members notified.** The Node delivers the `state.dm_promote` Event to both connected clients.

---

#### 3.16.4 History Preservation

All Events produced in the DM Space before promotion are preserved unchanged in the DAG. The `state.dm_promote` Event is simply appended as a new tip. No Events are deleted, modified, or re-signed. Members can scroll back through the full conversation history from before promotion.

If E2E encryption was active (Phase 2), messages from before promotion remain encrypted under the pre-promotion MLS group keys. New members added after promotion cannot decrypt pre-promotion messages — this is correct behaviour and is consistent with the forward secrecy guarantee (3.10.6).

---

#### 3.16.5 New Capabilities After Promotion

After promotion, the Space operates identically to a Space created with `state.space_create`. All capabilities that were unavailable in DM mode are now available:

- Additional members may be invited via `membership.invite`
- Additional Rooms may be created via `state.room_create`
- The Space may federate with other Nodes via the standard federation handshake (3.4)
- The Space may appear in Bootstrap Node directories if the owner chooses to make it discoverable
- Roles (admin, moderator) may be assigned to members

The original two members retain their membership. The promoting member becomes the Space **owner**; the confirming member becomes an **admin** by default. Role assignments can be changed after promotion via standard role Events.

---

#### 3.16.6 Promotion Timeout

If the other member does not respond to `dm.promote_propose` within **7 days** (WD-31), the proposal expires. The Node discards the pending proposal and notifies the proposing member. The proposing member may re-initiate promotion at any time.

---

#### 3.16.7 DM Promotion EventType Registry Additions

*DM promotion events:*

| EventType | Description |
|---|---|
| `dm.promote_propose` | Initiating member proposes promotion of a DM Space |
| `dm.promote_confirm` | Other member confirms the promotion proposal |
| `dm.promote_reject` | Other member rejects the promotion proposal |
| `state.dm_promote` | Node records the completed promotion in the DAG |

---

#### 3.16.8 DM Promotion Error Codes

DM promotion failures use the 9000 error code range.

| Code | Error string | Meaning |
|---|---|
| 9001 | `dm_promotion_not_dm_space` | Target Space is not a DM Space — already promoted or was never a DM Space |
| 9002 | `dm_promotion_already_pending` | A promotion proposal is already pending for this Space |
| 9003 | `dm_promotion_rejected` | The other member rejected the promotion proposal |
| 9004 | `dm_promotion_expired` | The promotion proposal expired before the other member responded |
| 9005 | `dm_promotion_not_member` | Requestor is not a member of this DM Space |

**Work definition added:**

| # | Value | Current setting | Location | Review trigger |
|---|---|---|---|---|
| WD-31 | DM promotion proposal timeout | 7 days | 3.16.6 | Review with first production deployment |

---

## Chapter 3 — Work Definitions Pending Phase 1 Validation

The following values were established before implementation testing. Each is
explicitly provisional and MUST be reviewed against real-world measurements
during Phase 1 testing. No value should be treated as final until the Phase 1
smoke test has been run and message sizes and timing behaviour observed.

| # | Value | Current setting | Location | Review trigger |
|---|---|---|---|---|
| WD-01 | Tier 1 message size ceiling | 64 KB | 3.1.1 | Measure actual Event sizes in smoke test |
| WD-02 | Tier 2 message size ceiling | 32 KB | 3.1.1 | Measure actual Event sizes in smoke test |
| WD-03 | Tier 3 message size ceiling | 16 KB | 3.1.1 | Measure actual Event sizes in smoke test |
| WD-04 | Tier 4 message size ceiling | 8 KB | 3.1.1 | Measure actual Event sizes in smoke test |
| WD-05 | Keepalive ping interval | 30 seconds | 3.3.5 | Measure connection stability under load |
| WD-06 | Keepalive pong timeout | 10 seconds | 3.3.5 | Measure connection stability under load |
| WD-07 | Reconnection backoff ceiling | 30 seconds | 3.3.6 | Acceptable during smoke test; review under real network conditions |
| WD-08 | Pending Event buffer timeout | 30 seconds | 4.12.3 | Observe DAG sync latency between Nodes in smoke test |
| WD-09 | Trust Assertion TTL | 1 year | 3.8.6 | Review with Auth Module operator before production deployment |
| WD-10 | Node announcement TTL | 90 days | 3.5.6 | Review with network operator before production deployment |
| WD-11 | `prev_events` maximum array length | 10 entries | 3.2.5 | Observe maximum DAG concurrency in Phase 1 and Phase 2 federation tests |
| WD-12 | Federation handshake response timeout | 10 / 15 seconds | 3.4.3 | Observe handshake latency in smoke test |
| WD-13 | Federation re-initiation cooldown after reject | 60 seconds | 3.4.2 | Acceptable during smoke test; review for production |
| WD-14 | MLS KeyPackage TTL | 90 days | 3.10.3 | Review with first Phase 2 MLS implementation |
| WD-15 | Trust Assertion TTL — Tier 3 | 6 months | 3.11.3 | Review with first Tier 3 Auth Module operator |
| WD-16 | Trust Assertion TTL — Tier 4 | 3 months | 3.11.4 | Review with first Tier 4 institutional partner |
| WD-17 | Migration batch acknowledgement timeout | 30 seconds | 3.12.4 | Observe transfer performance in first migration test |
| WD-18 | Source Node grace period after migration | 30 days | 3.12.9 | Review with first production migration |

After Phase 1 smoke test, update this table: replace "work definition" status
with either "confirmed" (value is appropriate) or "revised to X" (value changed).

---

## Chapter 3 — Open Questions

**OQ-02 — Streaming: needs architectural introduction before specification**

XGen currently handles discrete Events. Streaming is a fundamentally different communication mode — continuous media (audio, video, screen share) — and requires its own dedicated design session before any specification work begins.

**Note for when this topic is opened:** Joe needs an introduction to streaming technology before design begins. The session should cover: WebRTC vs RTMP vs WebTransport, peer-to-peer vs relay (SFU/MCU) topologies, how streaming relates to the existing XGen Node architecture, and what "built-in" means for a federated protocol. Do not jump into specification until the conceptual foundation is established.

**Known related feature — built-in session recording:** Joe wants recording as a first-class protocol feature, not a client-side add-on. Reference: Skype's late-era recording function. This has implications for consent, storage, access control, the audit log (D-032), and E2E encryption (3.10) — if content is encrypted end-to-end, what does a recording capture and who holds the key?

Preliminary questions to address in the introduction session:
- What is a stream in XGen terms — a special EventType, a separate channel, or a separate protocol layer?
- Does streaming use the Node as a relay or is it peer-to-peer between clients?
- How does a stream relate to a Space and a Room?
- Who can initiate, join, and end a stream?
- Recording: where does the recording live — Node storage, client storage, or dedicated media server? Who owns it? Who can access it? How is consent handled?
- How does recording interact with the audit log?
- How does recording interact with E2E encryption?

**Resolve during:** a dedicated streaming introduction and design session. Do not begin specification until Joe has been introduced to the technology landscape.

---

**OQ-01 — XGen Module Architecture — RESOLVED (D-036, April 2026)**

Resolved in Ch6 section 6.8. Summary: Event subscription + meta_atts communication model; one package format regardless of complexity; system/user identity mode enum; three UI forms (headless/widget/window); universal module list with stacked block entries. See D-036 in DECISIONS.md and Ch6 section 6.8 for full specification.

What is the canonical form of an XGen module? This question is unresolved and blocks several downstream decisions including CLI extensibility (Fix 14 in FIXES_ph1.md), the Phase 2 client UI structure, Node extensibility, and the public API surface of the `xgen-core` crate (D-022).

Modules are not a client-only concept. Both `xgen-node` and `xgen-client` will support modules. A module may extend the Node (e.g. a compliance reporting module, a content moderation module, a bridge to another protocol), the client (e.g. a UI skin, a bot interface, a CLI command set), or both simultaneously.

Open sub-questions:
- Does a module have a Node entry point, a client entry point, or both?
- Does a module have a CLI entry point, a UI entry point, or both?
- Is a module one file, one folder, or one process?
- How does a module register itself with the Node and/or client?
- How does it communicate — shared library, subprocess, network API, event subscription?
- Who can author one — Foundation only, or open contribution?
- What is the minimum viable module (the "Hello World" of XGen modules)?
- Does a Node module run as a sidecar process, a loaded plugin, or a federated microservice?
- How does module capability interact with the Node's open enum capability advertisement (3.4.3)?

**Resolved during:** Ch6 second pass — module architecture section. This question must be answered before Fix 14 (CLI lifecycle commands) can be implemented, before Node extensibility is designed, and before any plugin/extension work begins.

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

### Session 10 — April 2026 (JozefN)
**Covered:** Section 3.9 State Resolution Algorithm written in full — first Phase 2 section. Eight subsections: 3.9.1 What State Resolution Solves (state key concept defined as tuple of EventType + state_key_field; conflict defined as same state key with no causal ordering; message Events explicitly excluded from resolution), 3.9.2 Convergence Guarantee (strong eventual consistency; pure function of Event content; no timestamps as tiebreakers; commutative and associative), 3.9.3 Seven-Layer Resolution Algorithm (full algorithm with input/output definition; all seven layers specified: Layer 1 EventType logic hardcoded table — ban beats join/invite/kick, kick beats join/invite, leave beats join; Layer 2 Auth Tier — Tier 4 > 3 > 2 > 1, note Layer 2 inactive in Phase 1 single-Tier deployments; Layer 3 Home Node assertion for Identity's own state conflicts; Layer 4 Role priority — owner > admin > moderator > member, role change edge case handled via Layer 3 first; Layer 5a Manual Node ordering via state.node_priority; Layer 5b Federation recency via state.federation_add timestamp, home Node treated as earliest; Layer 5c Lexicographic event_id absolute backstop), 3.9.4 Resolution by conflict category (four categories each with characteristic layer path — state conflict typically resolves at Layer 4, permission conflict at Layer 1, authority conflict at Layer 3+4, ordering conflict at Layer 5), 3.9.5 Split-Brain Recovery (no special protocol — standard DAG merge plus state resolution handles automatically; convergence guarantee makes this free), 3.9.6 Pending Event Timeout (30s WD-08, discard over indefinite hold, re-requested on next sync), 3.9.7 State Snapshot and Incremental Application (snapshot as performance optimisation, Event log always authoritative, loser Events stay in DAG permanently), 3.9.8 Error Codes (5 codes in 4000 range, new range distinct from 1xxx/2xxx/3xxx).

**Section 3.9 complete.**

**Next:** Section 3.10 End-to-End Encryption — MLS vs Megolm decision needed before writing.

### Session 11 — April 2026 (JozefN)
**Covered:** Section 3.10 End-to-End Encryption written in full. Decision D-031 recorded: MLS (RFC 9420) selected over Megolm — XGen is future infrastructure, correctness over implementation speed. Eleven subsections: 3.10.1 Encryption Model and Node Role (two-layer model: TLS for transport, MLS for content; Node is structurally excluded from decrypting content; E2E scope = message Events only, state/membership Events plaintext by necessity), 3.10.2 MLS Concepts in XGen Context (full XGen↔MLS mapping table; one MLS group per Room not per Space; key isolation between Rooms), 3.10.3 KeyPackage Management (single-use KeyPackage; Node maintains pool of ≥3 per device; 90-day TTL WD-14; full schema), 3.10.4 Group Initialisation (4-step sequence; state.mls_group_init Event schema; MLS cipher suite 2 mandated: MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519; algorithm agility Phase 3), 3.10.5 Member Addition (8-step Welcome sequence; mls.commit and mls.welcome schemas; forward secrecy on epoch advance), 3.10.6 Member Removal (5-step Remove sequence; post-removal security — historical messages readable, future inaccessible), 3.10.7 Message Encryption (encryption/decryption flows; content carries only mls_ciphertext; Event signature covers encrypted blob not plaintext), 3.10.8 Spaces Without E2E Encryption (explicit opt-out via e2e_encryption field in state.space_create; field immutable after creation; client MUST display visible indicator), 3.10.9 Phase 1 Forward Compatibility (Phase 1 defaults e2e_encryption: false; Phase 2 Spaces encrypted from genesis; coexistence on same Node), 3.10.10 MLS EventType Registry Additions (6 new EventTypes: state.mls_group_init, mls.key_package, mls.key_package_request, mls.key_package_response, mls.commit, mls.welcome), 3.10.11 E2E Error Codes (7 codes in 5000 range).

**Section 3.10 complete.**

**Next:** Section 3.11 Auth Module — Tiers 2–4 Interfaces.

### Session 12 — April 2026 (JozefN)
**Covered:** Section 3.11 Auth Module — Tiers 2–4 Interfaces written in full. Seven subsections: 3.11.1 Tier Model Recap (cumulative tiers; Space auth_tier immutable; slot contract identical across tiers; differences are verification depth and claims); 3.11.2 Tier 2 ISO 27001 Professional (real name verified against government ID; organisational affiliation verified; ISO 27001 operator attestation; three verification states A/B/C; additional claims: legal_name_verified, organisation_verified, organisation_domain, iso27001_operator; TTL 1 year WD-09); 3.11.3 Tier 3 Corporate/Regulated (AML/KYC enhanced due diligence; watchlist screening PEP/sanctions; corporate role verification; 7-year audit trail SOX §802; additional claims: kyc_verified, kyc_level, corporate_role_verified, corporate_role, watchlist_clear, watchlist_checked_at; TTL 6 months WD-15; audit trail is institutional obligation not protocol-enforced); 3.11.4 Tier 4 Government/Healthcare (eIDAS LoA High required; government credential binding; clearance verification; hardware auth FIDO2/WebAuthn; data localisation obligation; additional claims: eidas_loa, government_credential_bound, credential_type, clearance_verified, clearance_level, jurisdiction, data_localisation; TTL 3 months WD-16; key rotation still optional at Tier 4 per D-001); 3.11.5 Cross-Tier Compatibility (tier_verified >= auth_tier enforcement; federated Spaces with mixed-Tier Auth Modules — each Node trusts its own Auth Module independently); 3.11.6 Higher Tier Registration (same out-of-band process as Tier 1; institutional obligations per Tier documented); 3.11.7 Error Codes (7 new codes in 3010–3016 range extending existing 3000 range).

**New work definitions:** WD-15 (Tier 3 TTL 6 months), WD-16 (Tier 4 TTL 3 months) — added to Work Definitions table.

**Section 3.11 complete.**

**Next:** Section 3.12 Space Migration Protocol.

### Session 13 — April 2026 (JozefN)
**Covered:** Section 3.11 extended with new subsection 3.11.8 Audit Log Requirements. Two distinct log types formally distinguished and specified: (1) Debug log — technical diagnostic, operator-controlled level, disposable; (2) Audit log — permanent accountability record, cannot be disabled, regulatory retention. Node-level protocol audit log defined: append-only JSON Lines format, 11 EventTypes covered (membership lifecycle, space/room creation, federation, identity registration, key rotation), mandatory fields (ts/event_type/event_id/node_id), monthly rotation to `audit/protocol_audit_YYYY-MM.jsonl`, MUST NOT be auto-deleted. Auth Module audit log specified separately for Tier 3 (required, 7-year retention SOX §802, 10 required fields, tamper evidence SHOULD) and Tier 4 (required, 10-year minimum for healthcare, mandatory hash-chain tamper evidence, data localisation constraint). Relationship clarified: both logs needed for complete compliance picture, neither replaces the other.

**Section 3.11 complete.**

**Next:** Section 3.12 Space Migration Protocol — already recorded above.

### Session 14 — April 2026 (JozefN)
**Covered:** Section 3.12 Space Migration Protocol written in full. Eleven subsections: 3.12.1 Who Can Trigger (owner only, authenticated on source Node, destination must be reachable and willing); 3.12.2 Migration State Machine (6 states: IDLE→NEGOTIATING→TRANSFERRING→VERIFYING→COMPLETE/FAILED, Space remains live during transfer); 3.12.3 Initiation Sequence (migration.request from owner, migration.propose source→destination, migration.accept/reject with 4 rejection reasons); 3.12.4 Event Transfer (causal order, dedicated migration channel not federation transport, 100-Event batches with batch_index and batch_hash, 13-step validation on destination, 30s WD-17 batch ack timeout, max 3 retransmits); 3.12.5 Tail Batch (tracks Events produced during transfer, multiple tail rounds possible, migration.transfer_complete with total_events and dag_tips); 3.12.6 Verification (4 checks: event count, DAG tip match, state consistency replay, membership integrity); 3.12.7 Cutover and Member Notification (state.space_migrate DAG record, destination activation, transport.redirect to all connected members, automatic client reconnect); 3.12.8 Federation Re-establishment (federation_add Events recoverable from DAG, destination initiates standard handshakes, migration.federation_notify to all peers); 3.12.9 Source Node Decommission (30-day WD-18 grace period read-only, transport.redirect for lagging clients, MUST NOT delete immediately); 3.12.10 EventType Registry Additions (12 new EventTypes in migration.* and transport.redirect namespace); 3.12.11 Error Codes (8 codes in 6000 range, new range).

**New work definitions:** WD-17 (batch ack timeout 30s), WD-18 (grace period 30 days).

**Section 3.12 complete.**

**Next:** Section 3.13 Identity Replication Parameters.

### Session 15 — April 2026 (JozefN)
**Covered:** Section 3.13 Identity Replication Parameters written in full. Ten subsections: 3.13.1 Replication Model (passive pull-based, home Node authoritative, update_version resolves conflicts, distinct from Space federation); 3.13.2 Replication Factor N=3 WD-19 (N+1 total, propagates to all if fewer than N known); 3.13.3 Replica Node Selection (4 criteria: geographic diversity, availability, no duplicate, random tiebreak); 3.13.4 Replication Wire Protocol (identity.replicate pushed by home Node, signed by Identity keypair, replicate_ack, version_stale rejection returns stored version); 3.13.5 Update Propagation (best-effort with retry, 30s WD-20 ack timeout, 3 retries, stale marking, replacement replica, 24h WD-21 retry); 3.13.6 Replica Refresh Anti-Entropy (7-day WD-22 interval, identity.refresh_query with stored_version, unreachable home Node retains record); 3.13.7 Replica Record TTL (90 days WD-23, MUST NOT serve expired, factor of 7 days refresh interval for outage survival); 3.13.8 Orphaned Identity Recovery (5-step procedure: fetch from replica, select new home, re-register same keypair with re_registration:true, propagate to N new replicas, send identity.home_changed; key continuity — identity_id unchanged, all prior signatures valid; Trust Assertion continuity); 3.13.9 EventType Registry Additions (5 new EventTypes: replicate, replicate_ack, refresh_query, refresh_ack, home_changed); 3.13.10 Error Codes (4 codes 3020–3023 extending 3000 range).

**New work definitions:** WD-19 through WD-23.

**Section 3.13 complete.**

**Next:** Section 3.14 Bootstrap Node Protocol.

### Session 16 — April 2026 (JozefN)
**Covered:** Section 3.14 Bootstrap Node Protocol written in full. Key design principle: Bootstrap Nodes are ordinary XGen Nodes with `xgen.bootstrap` capability — no special binary, no privileged position, no centralised authority. Foundation operates reference Bootstrap Nodes but any operator may run one. Eight subsections: 3.14.1 Bootstrap Node Capability (xgen.bootstrap in capabilities enum, bootstrap_info field with directory_url, accepts_registrations, region, operator); 3.14.2 Directory Format (HTTPS JSON document, signed by Bootstrap Node keypair, nodes array ordered by reputation_score, 1-hour WD-24 max age, 30-minute regeneration recommendation); 3.14.3 New Node Registration (bootstrap.register over WebSocket, signature proves keypair ownership, register_ack returns directory_url, 7-day WD-25 TTL, registration not mandatory for operation); 3.14.4 Directory Query Protocol (HTTPS GET with optional region/min_reputation/limit/exclude filters, 6-step bootstrap sequence); 3.14.5 Trust at First Run (3 mechanisms in order: hardcoded Foundation list compiled into binary, operator-configured trusted_nodes in config with explicit node_id anchor, manually provided peer CLI endpoint); 3.14.6 Failure Handling (exponential backoff, 5-failure isolated mode, 10-minute WD-26 retry in background, peer-initiated connection exits isolated mode; isolated mode is not a failure state); 3.14.7 EventType Registry (5 EventTypes: register, register_ack, keepalive, keepalive_ack, deregister); 3.14.8 Error Codes (5 codes in 7000 range, new range).

**New work definitions:** WD-24, WD-25, WD-26.

**Section 3.14 complete.**

**Next:** Section 3.15 Node Reputation Format.

### Session 17 — April 2026 (JozefN)
**Covered:** Sections 3.15 Node Reputation Format and 3.16 DM Space Promotion Sequence written in full.

**3.15 Node Reputation Format** — six subsections. Key framing: reputation is a soft, non-binding quality-of-service signal — no protocol action is gated on it. 3.15.1 Signal Structure (6 components: uptime_ratio 0.35, announcement_freshness 0.25, defederation_count 0.20, successful/failed federations 0.10 each, protocol_violations flat penalty; weights are WD-27 pending calibration); 3.15.2 Propagation (Bootstrap-to-Bootstrap only, 6-hour WD-29 interval, 0.6/0.4 WD-28 local/remote merge weights); 3.15.3 Defederation Signal Integration (optional submission, advisory only, evidence_event_ids provided for human review not automated action, 10/24h WD-30 rate limit); 3.15.4 Privacy Considerations (signals are voluntary, reveal federation relationships but not Space content or member lists); 3.15.5 EventType Registry (2 EventTypes); 3.15.6 Error Codes (3 codes in 8000 range).

**3.16 DM Space Promotion Sequence** — eight subsections. Key framing: promotion is irreversible. 3.16.1 DM Constraints (5 constraints: max 2 members, max 1 Room, no federation, no invitations, private); 3.16.2 Who Can Initiate (either member, both must consent); 3.16.3 Promotion Sequence (6-step: propose→deliver→confirm/reject→DAG Event→lift constraints→notify; state.dm_promote signed by Node not member); 3.16.4 History Preservation (all pre-promotion Events preserved, state.dm_promote appended as new tip, MLS forward secrecy applies to pre-promotion messages); 3.16.5 New Capabilities (full Space capabilities, promoting member becomes owner, confirming member becomes admin); 3.16.6 Promotion Timeout (7 days WD-31); 3.16.7 EventType Registry (4 EventTypes: dm.promote_propose/confirm/reject, state.dm_promote); 3.16.8 Error Codes (5 codes in 9000 range, new range).

**New work definitions:** WD-27, WD-28, WD-29, WD-30 (3.15), WD-31 (3.16).

**Chapter 3 Phase 2 specification complete. All 8 sections (3.9–3.16) written.**

**Next:** Chapter 3 Phase 2 review and cross-check before handing to implementation.

### Session 19 — April 2026 (JozefN)
**Covered:** EventType registry updated with two new message EventTypes arising from Ch6 6.7 UI design decisions: `message.edit` (edit of prior message, original_event_id reference, UI renders latest version in place with history on click) and `message.delete` (deletion/redaction, placeholder preserving timeline, original content stays in DAG). `message.redact` renamed to `message.delete` for clarity — was only defined in the registry with one line, no other references. E2E encryption scope in 3.10.1 updated to include both new EventTypes.

### Session 18 — April 2026 (JozefN)
**Covered:** Chapter 3 Phase 2 cross-check review and fixes. Eight issues identified and applied:

1. Fix 1 (3.12.10) — `migration.failed` added to EventType registry — was referenced in prose but missing from table
2. Fix 2 (3.6.3) — `re_registration: true` flag added to `identity.register` field table — used in orphan recovery (3.13.8) but not defined in registration schema
3. Fix 3 (3.5.3) — `bootstrap_info` added as optional field in node announcement schema and canonical signing order
4. Fix 4 (3.5.3) — `capabilities` field corrected from nested object format to flat array to match open enum principle (3.4.3) and 3.14.1 usage
5. Fix 5 (session log) — Session 13 reordered to correct chronological position; orphan duplicate removed from end of file
6. Fix 6 (3.11.7) — Identity error range 3000–3099 reservation note added — clarifies that 3010–3016 and 3020–3023 are sub-ranges of the identity range, not separate ranges
7. Fix 7 — WD-27 through WD-31 confirmed present in Work Definitions table ✔
8. Fix 8 (3.15.1) — Explicit note added: `score` field in reputation record IS the `reputation_score` exposed in Bootstrap directory documents (3.14.2); `components` breakdown is internal only

**Chapter 3 specification — Phase 1 and Phase 2 — fully complete and cross-checked.**

### Session 20 — May 2026 (JozefN)
**Covered:** Cross-check against Appendix E (Application Lifecycle States) and Ch6 §6.11 Console. Confirmed zero impact on Ch3: `SETUP` state is purely local (no network, no protocol messages); `CONNECTING` maps to §3.3.4 Phase 1; `AUTHENTICATING` maps to §3.3.4 Phase 2 (`transport.challenge` / `transport.auth` / `transport.auth_ok`); `auto_connect_local` uses the existing §3.3.4 connection flow unchanged. No new EventTypes, no new wire messages, no Ch3 changes required. Ch3 Phase 1 and Phase 2 remain fully complete and closed.

### Session 21 — 2026-05-15 (JozefN)
**Covered:** AI users and pacing additions to Ch3 (Pass 1 of two-pass spec authoring; Ch1 and Ch6 follow in a separate session). Three D-entries (D-059, D-060, D-061) translated into Ch3 spec surface.

**§3.6 Identity Registration Protocol — additions:**
- §3.6.3: `is_ai` and `ai_capabilities` fields added to the `identity.register` request schema and field definition table.
- §3.6.4: step 8 added to the acceptance pipeline — validates `is_ai` / `ai_capabilities` shape consistency. Pre-existing capacity check renumbered to step 9.
- §3.6.6: Identity record structure extended to include `is_ai` and `ai_capabilities`. Note added explaining replication semantics and immutability of `is_ai`.
- **§3.6.10 AI Identity Extension** (new subsection, 11 sub-sub-sections): registration, immutability of the AI declaration, Phase 2 capability flag set (`dm_initiate`, `spontaneous_post`), enforcement model (hard protocol-level for `dm_initiate`, client-side for `spontaneous_post`), capability updates, invitation and accountability (operator role with `state.ai_operator_delegate` / `state.ai_operator_revoke` Events), removal, Tier inheritance, replication, three new error codes (3040, 3041, 3042), Phase 2 vs future phases framing.

**§3.7 Space & Room Protocol — additions:**
- §3.7.6: `human_pacing_ms` and `ai_pacing_ms` added to the Space state components table.
- §3.7.8: `membership.mute` Event introduced (allows time-bound mute with `cooldown_until` that retains member context). Standard reason values table added; `auto_temperature` reserved as a reason value for `membership.kick` (humans) and `membership.mute` (AI) issued by temperature mechanism. Role permission table extended with `Mute members` (moderator+) and `Update Space pacing` (owner only).
- **§3.7.12 Pacing Rules on Spaces** (new subsection, 9 sub-sub-sections): fields, defaults (500 ms / 2000 ms), updates via new `state.space_pacing` EventType, authority and enforcement (client-side in Phase 2), member classification, scope (per-Space, per-member), rigid AI enforcement, interaction with temperature mechanism (Ch6 §6.12), EventType registry addition.

**Cross-referenced decisions:** D-037 (persistent accountable identity), D-046 (state resolution), D-049 (identity replication), D-059, D-060, D-061.

**Replication impact:** None on the wire format — the `identity_record` payload in `identity.replicate` (§3.13.4) carries `is_ai` and `ai_capabilities` automatically as part of the full Identity record. Replica Nodes enforce capabilities identically to the home Node.

**Section skeleton table:** updated to list 3.6.10 and 3.7.12 as Complete.

**Pending for Ch1 and Ch6 in the next session:** Ch1 philosophical paragraphs on AI participation and on temperature-as-transparency; Ch6 §6.12 full temperature mechanism specification (UI indicators, visibility rules, threshold values, decay model); Ch6 AI badge specification; Ch6 client-side pacing queue implementation guidance. Mr Code disposition file follows after Ch6.

### Session 22 — 2026-05-15 (JozefN)
**Covered:** §3.7.13 Temperature Property written (Pass 2 of the two-pass spec authoring; closes the protocol surface for D-061 after its rewrite). Eight sub-sub-sections: 3.7.13.1 reserved `meta_atts` keys (`xgen.room_temperature`, `xgen.member_temperature`; floats in `[0.0, 1.0]`; both optional); 3.7.13.2 threshold table (`temperature_thresholds` field on Room metadata response; warm/hot/fiery floats; client falls back to Ch6 defaults if absent); 3.7.13.3 visibility setting on Space state (`member_temperature_visibility` open enum: `moderator` / `everyone` / `self_only`; default `moderator`; updatable via new `state.space_temperature_visibility` EventType); 3.7.13.4 visibility enforcement (home Node filters outgoing `xgen.member_temperature` per role; `xgen.room_temperature` always visible); 3.7.13.5 computation locality (home Node authoritative, federated copies relay without recomputation, plugin interface unspecified at protocol level); 3.7.13.6 automated consequences (`membership.kick` for humans / `membership.mute` for AI with `reason = auto_temperature`; asymmetry is a plugin-author recommendation, not a protocol mandate); 3.7.13.7 temperature is not state-resolved (live signal, not consensus value; only the DAG-resident kick/mute consequences are state-resolved); 3.7.13.8 EventType registry addition.

**§3.7.6:** Space state components table extended with `member_temperature_visibility`.

**Section skeleton table:** updated to list 3.7.13 as Complete.

**Design alignment:** This session completes the Ch3 protocol surface for D-061 (post-rewrite). The mathematical model that produces the temperature values is intentionally absent from the protocol — it lives in a plugin running on the Room's home Node. The protocol carries the values, the bucket thresholds, the visibility rules, and the consequence Events; everything else is plugin business. This matches the rest of the protocol's design language (Auth Module Tier slot, `meta_atts` open namespace, vanilla Node `capabilities`, pacing rules) where the protocol provides mechanism and communities supply policy.