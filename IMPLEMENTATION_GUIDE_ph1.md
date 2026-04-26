# XGen Protocol — Phase 1 Implementation Guide

> Target: Claude Code  
> Language: Rust  
> Spec: `docs/xgen_ch3_specification.md`  
> Phase: 1 — Minimal Viable Protocol  
> Date: April 2026  

---

## Overview

This guide directs Phase 1 implementation of the XGen Protocol. Phase 1 produces a working two-Node, two-user, one-Room federated messaging system. The definition of done is the smoke test in spec section 3.7.11.

All implementation decisions must be consistent with `docs/xgen_ch3_specification.md`. When the guide and the spec conflict, the spec is authoritative. When the spec is ambiguous, flag the ambiguity before implementing — do not resolve it silently.

---

## Language and Toolchain

**Language:** Rust (stable, current version)  
**Async runtime:** Tokio  
**Build system:** Cargo  

### Required crates

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"          # WebSocket (spec 3.3)
serde = { version = "1", features = ["derive"] }
serde_json = "1"                    # JSON serialisation (spec 3.1.2)
ed25519-dalek = "2"                 # Ed25519 keypair + signing (spec 3.1.9, 3.2.4)
sha2 = "0.10"                       # SHA-256 for Event IDs (spec 3.2.3)
rand = "0.8"                        # Nonce generation
base64 = "0.21"                     # base64url encoding (spec 3.1.9)
chrono = { version = "0.4", features = ["serde"] }  # RFC 3339 datetime (spec 3.1.7)
uuid = { version = "1", features = ["v4"] }         # Internal IDs
tracing = "0.1"                     # Structured logging
tracing-subscriber = "0.3"
anyhow = "1"                        # Error handling
thiserror = "1"                     # Error types
```

---

## Project Structure

```
XGenProtocol/
├── IMPLEMENTATION_GUIDE_ph1.md     ← this file
├── docs/
│   └── xgen_ch3_specification.md  ← authoritative spec
├── xgen-node/                      ← Node implementation (binary)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── config.rs               ← Node configuration, keypair loading
│       ├── identity/
│       │   ├── mod.rs
│       │   ├── keypair.rs          ← Ed25519 keypair generation and storage
│       │   └── registry.rs         ← Identity record storage
│       ├── crypto/
│       │   ├── mod.rs
│       │   ├── signing.rs          ← Sign and verify Event canonical forms
│       │   ├── hashing.rs          ← SHA-256 Event ID derivation
│       │   └── encoding.rs         ← base64url encode/decode
│       ├── wire/
│       │   ├── mod.rs
│       │   ├── types.rs            ← All message type structs (serde)
│       │   ├── canonical.rs        ← Canonical form serialisation (spec 3.2.4)
│       │   ├── framing.rs          ← Transport frame encode/decode (spec 3.1.2)
│       │   └── validation.rs       ← Message validation pipeline (spec 3.2.6)
│       ├── transport/
│       │   ├── mod.rs
│       │   ├── server.rs           ← WebSocket server (accept connections)
│       │   ├── client.rs           ← WebSocket client (connect to peers)
│       │   ├── connection.rs       ← Connection lifecycle (spec 3.3.4)
│       │   └── auth.rs             ← Challenge-response authentication (spec 3.3.4)
│       ├── federation/
│       │   ├── mod.rs
│       │   ├── handshake.rs        ← Federation handshake (spec 3.4)
│       │   └── registry.rs         ← Federation relationship registry
│       ├── dag/
│       │   ├── mod.rs
│       │   ├── store.rs            ← Event log storage
│       │   ├── graph.rs            ← DAG structure, tips, prev_events (spec 3.2.5)
│       │   └── pending.rs          ← Pending buffer for missing predecessors
│       ├── space/
│       │   ├── mod.rs
│       │   ├── state.rs            ← Space and Room current state (spec 3.7.6, 3.7.7)
│       │   └── membership.rs       ← Membership event processing (spec 3.7.8, 3.7.9)
│       └── node/
│           ├── mod.rs
│           └── announcement.rs     ← Node announcement (spec 3.5.3)
├── xgen-client/                    ← Minimal CLI test client (binary)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── identity.rs             ← Client keypair generation and storage
│       └── commands.rs             ← register, join, send, receive
└── xgen-common/                    ← Shared types library
    ├── Cargo.toml
    └── src/
        └── lib.rs                  ← Types shared between node and client
```

---

## Deployment Model

Each XGen binary is a single self-contained executable. No runtime dependencies, no external libraries, no registry entries. The executable discovers its own location on first run and creates all required files and folders alongside itself. The folder is the application — deleting it removes everything cleanly.

**This is a permanent architectural principle, not a Phase 1 convenience.** It applies equally to Phase 2 and all future versions. No future feature, capability, or platform requirement justifies deviating from it. If a future requirement appears to demand external files or system-level integration, the correct solution is to bundle or embed — never to scatter.

### Node deployment structure

```
<any folder the operator chooses>\
  xgennode.exe                    ← the Node binary
  node_config.json                ← created on first run if absent
  node_keypair.enc                ← encrypted Ed25519 private key, created on first run
  auth_modules.json               ← trusted Auth Module registry, created on first run
  federation_registry.json        ← federation relationships, created on first run
  identity_registry.json          ← registered Identity records, created on first run
  spaces.json                     ← Space and Room state, created on first run
  event_logs\                     ← Room Event DAGs, created on first run
    <space_id>\                   ← one subfolder per Space
      <room_id>.jsonl             ← one append-only log per Room
  logs\                           ← operational logs, created on first run
    xgennode.log
```

### Client deployment structure

```
<any folder the user chooses>\
  xgenclient.exe                  ← the client binary
  client_config.json              ← created on first run if absent
  client_keypair.enc              ← encrypted Ed25519 private key, created on first run
  known_nodes.json                ← Node endpoint registry
  logs\
    xgenclient.log
```

### First-run initialisation sequence

On first run, both binaries:

1. Detect executable location — all data paths are relative to this folder
2. Check for existing keypair file — if absent, generate Ed25519 keypair and save encrypted
3. Check for existing config file — if absent, write default config and prompt operator to review
4. Create all required subfolders if absent (`event_logs\`, `logs\`)
5. Start normal operation

The operator never needs to configure paths. The binary manages everything inside its own folder.

### Multiple instances for testing

To run two Nodes on the same machine for Phase 1 testing:

```
C:\XGenTest\
  NodeA\
    xgennode.exe    ← copy of the binary
    ...             ← Node A's data files
  NodeB\
    xgennode.exe    ← copy of the binary
    ...             ← Node B's data files
```

Each Node has its own keypair and its own data. Both run simultaneously on different ports. This is the exact setup for the Phase 1 smoke test.

### Implementation note

In Rust, use `std::env::current_exe()` to get the executable path, then `parent()` to get the containing folder. All file paths are constructed relative to this base path. Never use hardcoded paths or platform-specific config directories (no AppData, no ~/.config) — the folder-is-the-application principle must hold.

```rust
fn base_dir() -> PathBuf {
    std::env::current_exe()
        .expect("Cannot determine executable location")
        .parent()
        .expect("Executable has no parent directory")
        .to_path_buf()
}
```

---

## Implementation Order

Implement in this exact order. Each layer depends on the previous. Do not skip ahead.

### Layer 1 — Cryptographic Foundation
**Spec refs:** 3.1.6, 3.1.9, 3.2.3, 3.2.4

1. `crypto/encoding.rs` — base64url encode/decode (no padding, URL-safe alphabet)
2. `crypto/hashing.rs` — SHA-256 hash of bytes → lowercase hex → hash URI
3. `identity/keypair.rs` — Ed25519 keypair generation, encrypted storage, loading
4. `crypto/signing.rs` — sign bytes with private key, verify signature with public key
5. `wire/canonical.rs` — produce canonical JSON form of a message (sorted keys, no whitespace)

**Test:** Generate a keypair, sign a known byte sequence, verify the signature. Derive a hash URI from a known input and check the output format.

---

### Layer 2 — Wire Format
**Spec refs:** 3.1.1–3.1.10, 3.2.1, 3.2.2

1. `wire/types.rs` — define Rust structs for all Phase 1 message types with serde derives:
   - `Event` (the base envelope, spec 3.2.1)
   - All transport messages: `TransportChallenge`, `TransportAuth`, `TransportAuthOk`, `TransportAuthFail`, `TransportError`, `TransportGoodbye`, `TransportSyncRequest`, `TransportRateLimit`
   - All federation messages: `FederationHello`, `FederationCapabilities`, `FederationAccept`, `FederationReject`, `FederationGoodbye`
   - All identity messages: `IdentityRegister`, `IdentityRegisterOk`, `IdentityRegisterFail`, `IdentityUpdate`, `IdentityGet`
   - All space/room messages: `StateSpaceCreate`, `StateDmSpaceCreate`, `StateRoomCreate`, `MembershipInvite`, `MembershipJoin`, `MembershipLeave`, `MembershipKick`, `MembershipBan`
   - All message events: `MessageText`, `MessageImage`, `MessageFile`, `MessageReaction`, `MessageRedact`
2. `wire/framing.rs` — encode/decode transport frame: `[1-byte length][format string][4-byte payload length][payload]` (spec 3.1.2)
3. `wire/validation.rs` — implement the 13-step Event validation pipeline (spec 3.2.6). Steps 1–7 are pure structural checks; steps 8–13 require crypto and state context

**Test:** Serialise an `Event` struct to JSON, wrap in a transport frame, unwrap the frame, deserialise back, verify round-trip equality.

---

### Layer 3 — DAG and Event Storage
**Spec refs:** 3.2.3, 3.2.5

1. `dag/store.rs` — append-only in-memory Event store (Phase 1: no persistence required). Store Events by `event_id`. Retrieve by ID.
2. `dag/graph.rs` — track DAG tips (Events with no successors). On new Event: validate `prev_events` all exist, check for cycles, update tips.
3. `dag/pending.rs` — buffer for Events whose `prev_events` are not yet known. On receiving a missing predecessor, process any unblocked pending Events.

**Test:** Insert a sequence of Events forming a fork (two Events with the same prev), then a merge. Verify tips are correct after each insertion.

---

### Layer 4 — Transport
**Spec refs:** 3.3.1–3.3.9

1. `transport/server.rs` — start a WebSocket server on configured endpoint. Accept connections. Hand each connection to `connection.rs`.
2. `transport/connection.rs` — implement the 4-phase connection lifecycle (CONNECT → AUTHENTICATE → ACTIVE → CLOSE). Enforce timeout on authentication.
3. `transport/auth.rs` — implement challenge-response: generate nonce, send `TransportChallenge`, receive `TransportAuth`, verify signature, send `TransportAuthOk` or `TransportAuthFail`.
4. `transport/client.rs` — connect outbound to a peer Node endpoint. Run the same authentication as a client (respond to challenge with signed nonce).
5. Keepalive: WebSocket ping every 30s, disconnect if pong not received within 10s.
6. Reconnection: exponential backoff with jitter on connection loss.

**Test (Local Node):** Start a Node in Local Node mode. Connect a second Node client. Complete authentication. Exchange a ping/pong. Verify connection drops cleanly on `TransportGoodbye`.

---

### Layer 5 — Node Identity and Announcement
**Spec refs:** 3.5.1–3.5.8

1. `node/announcement.rs` — produce a signed `node_announcement`. Load from disk if exists; generate if first run.
2. On transport connection established: send current announcement to peer.
3. Receive and store peer announcements. Verify signature. Check `valid_until`. Apply `announcement_version` gating.

**Test:** Generate two Node announcements. Verify one supersedes the other based on `announcement_version`.

---

### Layer 6 — Federation Handshake
**Spec refs:** 3.4.1–3.4.7

1. `federation/handshake.rs` — implement the full state machine: IDLE → HELLO_RECEIVED → CAPS_SENT → ACTIVE → CLOSED.
2. Capability negotiation: compute intersection of `serialisation` arrays, select highest-preference common format, negotiate protocol minor version (lower wins).
3. On `federation.accept`: record the `session_id`. Produce `state.federation_add` Event in each shared Space's DAG.
4. `federation/registry.rs` — persistent federation relationship registry (JSON file on disk). Consult on startup to re-establish connections.

**Test:** Two Node instances on localhost. Run the full handshake. Verify both reach ACTIVE state and the `session_id` matches.

---

### Layer 7 — Identity Registration
**Spec refs:** 3.6.1–3.6.9

1. `identity/registry.rs` — persistent Identity record store (JSON file on disk for Phase 1).
2. Receive `IdentityRegister`. Run the 8-step acceptance pipeline (spec 3.6.4). In Local Node mode: skip steps 4–7.
3. On success: create Identity record, store to disk, send `IdentityRegisterOk`.
4. Handle `IdentityGet` queries.

**Test (Local Node):** Client generates keypair. Authenticates transport. Sends `IdentityRegister` (no trust assertion — Local Node mode). Verify Node creates record and responds with `IdentityRegisterOk`.

---

### Layer 8 — Space and Room Protocol
**Spec refs:** 3.7.1–3.7.11

1. `space/state.rs` — Space and Room state derived from Event DAG. Track current state by processing State Events in causal order.
2. Handle `StateSpaceCreate`: derive Space ID from Event hash, create Space state, creator becomes owner.
3. Handle `StateRoomCreate`: derive Room ID from Event hash, create Room state in Space.
4. Handle `StateDmSpaceCreate`: create DM Space with single auto-Room, send `MembershipInvite` to invitee.
5. `space/membership.rs` — handle all `membership.*` Events: join, leave, invite, kick, ban. Enforce role permission table (spec 3.7.8).
6. Handle `space.join_request` from federated Node. In Phase 1: auto-approve. Produce `state.federation_add`.
7. On federation established: send full Space state and Room Event history to new Node.

**Test:** Alice creates a Space and Room on Node A. Bob's Node B federates. Bob joins Space and Room. Verify both Nodes have identical Space/Room state.

---

### Layer 9 — Message Exchange
**Spec refs:** 3.2.2 (`message.*` EventTypes), 3.2.6

1. Accept `MessageText` Events from authenticated, authorised clients.
2. Run full 13-step validation pipeline.
3. Store in Room DAG. Update tips.
4. Propagate to all federated Nodes that participate in the Space.

**Test:** Alice sends "Hello Bob". Verify Event appears in Node B's Room DAG with correct `event_id`, valid signature, correct `prev_events`.

---

### Layer 10 — Smoke Test
**Spec refs:** 3.7.11

Run the full 17-step Phase 1 smoke test exactly as written in the spec. Both regular Space and DM Space variants must pass.

This is the Phase 1 definition of done. ✅

---

## Local Node Mode

All development and testing uses Local Node mode throughout Layers 1–10. Local Node mode means:

- `local_node: true` in Node config
- No TLS — use `ws://` not `wss://`
- No Trust Assertion required for Identity registration
- No external network interfaces — localhost only
- 256KB message size ceiling

Do not implement production TLS or Trust Assertion validation until the smoke test passes in Local Node mode.

---

## Error Handling Conventions

- Use `thiserror` for library error types, `anyhow` for application-level errors
- Every validation failure maps to a defined error code from the spec (1xxx transport, 2xxx federation, 3xxx identity)
- Error display follows the spec format: `Error <code> (<string>): <description>` (spec 3.3.8)
- Never panic in network-handling code — use `Result` and propagate errors
- Log every rejected Event with the step number that failed and the error code

---

## Testing Strategy

- Unit tests for every crypto primitive (sign/verify round-trip, hash derivation, base64url)
- Unit tests for canonical form serialisation — known input → known output
- Unit tests for transport frame encode/decode round-trip
- Unit tests for Event validation pipeline — one test per validation step, including failure cases
- Integration tests for each Layer using two in-process Node instances on localhost
- Smoke test as the final integration test (Layer 10)

Run `cargo test` after completing each Layer before moving to the next.

---

## Spec Cross-Reference Quick Index

| Topic | Spec section |
|---|---|
| Transport frame format | 3.1.2 |
| URI formats | 3.1.6 |
| Datetime format | 3.1.7 |
| base64url encoding | 3.1.9 |
| Event envelope schema | 3.2.1 |
| EventType registry | 3.2.2 |
| Event ID derivation | 3.2.3 |
| Signature canonicalisation | 3.2.4 |
| prev_events DAG rules | 3.2.5 |
| Event validation pipeline | 3.2.6 |
| WebSocket transport | 3.3.1 |
| Connection lifecycle | 3.3.4 |
| Keepalive | 3.3.5 |
| Reconnection backoff | 3.3.6 |
| Transport error codes | 3.3.8 |
| Federation handshake | 3.4.2–3.4.4 |
| Node keypair and announcement | 3.5.1–3.5.6 |
| Identity registration | 3.6.3–3.6.4 |
| Space creation | 3.7.3 |
| DM Space creation | 3.7.4 |
| Room creation | 3.7.5 |
| Space membership | 3.7.8 |
| Federation initiation | 3.7.10 |
| Phase 1 smoke test | 3.7.11 |

---

## Implementation Decision Log — Mandatory Rule

During implementation, decisions will be made that are not covered by the spec — how a particular module is structured, why a specific storage approach was chosen, what tradeoffs were encountered, what the spec turned out to be ambiguous about in practice. These decisions MUST be recorded.

**Rule:** Every implementation decision that goes beyond what the spec explicitly prescribes must be noted in `DECISIONS.md` in the project root before moving to the next layer.

`DECISIONS.md` is not a formal document — it is a running log. Each entry needs only:

```
## <short title>
Date: YYYY-MM-DD
Layer: <which implementation layer this relates to>
Spec ref: <section number if applicable>

<What the decision was, why it was made, what alternatives were considered, what tradeoffs were accepted.>
```

Example entry:

```
## Event log storage format — append-only JSONL per Room
Date: 2026-05-01
Layer: 3 — DAG and Event Storage
Spec ref: 3.2.5

Chose one .jsonl file per Room (one JSON Event per line) rather than a single
database. Reason: simplest possible format for Phase 1, human-readable,
easy to inspect during debugging, no external database dependency. Tradeoff:
no indexing — full scan required to find an Event by ID. Acceptable for Phase 1
with small Event counts. Phase 2 will likely need an index.
```

**Why this matters:** These entries are the raw material for Chapter 4. They capture the honest history of what was built and why — decisions that will otherwise be forgotten by the time Chapter 4 is written. They also feed back into spec revisions: if an implementation decision reveals a spec ambiguity, that ambiguity is fixed in the spec before Phase 2 begins.

---

## What Comes Next

After the Phase 1 smoke test passes:

1. Return to the spec project to write Chapter 3 Phase 2 sections — informed by implementation experience
2. Write Chapter 4 (Implementation) guided by what was built
3. Begin Phase 2 implementation

Do not start Phase 2 implementation before Phase 2 is specified.
