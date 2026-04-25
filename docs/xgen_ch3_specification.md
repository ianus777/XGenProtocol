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
**Covered:** Section 3.1.1 Message Size Limits written. Two-layer size model established: Tier ceiling (hard protocol limit by Auth Tier) and Space override (tighter limit set at creation, immutable). Binary content excluded from protocol messages by design — content by reference only, base64url reserved for cryptographic material. Size reference table added covering 2KB–256KB range with byte counts, ASCII character counts, and usable JSON content estimates. Tier ceiling table: Local Node = 256KB (localhost only, not a wire-level Tier), Tier 1 = 64KB, Tier 2 = 32KB, Tier 3 = 16KB, Tier 4 = 8KB. All values marked as work definitions pending Phase 1 testing validation. Local Node mode defined as a Node configuration flag, not a protocol-level concept — no external federation permitted, localhost only, structurally prevents network exploitation. Enforcement rule: reject before signature verification.

**Next session to begin with:**
> **3.1 Wire Format continued.** Field naming conventions, required vs optional fields, null/absent field handling, URI formats (xgen_uri, hash_uri, pubkey_uri), datetime format, integer precision, base64url encoding, versioning in messages.
