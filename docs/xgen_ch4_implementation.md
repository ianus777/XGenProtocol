# XGen Protocol — Chapter 4: Implementation
> Status: wip  
> Version: 0.1  
> Date: April 2026  
> Last edited: April 2026  
> Language: English  
> Author: JozefN  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Overview

Chapter 4 bridges the protocol specification of Chapter 3 and the practical work of building XGen. Where Chapter 3 says *how XGen works*, Chapter 4 says *how to build it* — the technology choices, project structure, recommended build order, and implementation guidance for each major protocol area.

This chapter is written for the first implementer: someone who has read Chapters 2 and 3 and is ready to write code. It does not repeat the protocol specification. It supplements it with the decisions that the spec deliberately leaves to the implementer — language, libraries, module boundaries, persistence strategy, and testing approach.

**Reference implementation language: Rust.** The XGen reference implementation is written in Rust. The rationale is stated in 4.2. The protocol is language-agnostic by design — the wire format, message schemas, and state machines specified in Chapter 3 can be implemented in any language. Future community SDKs in Go, TypeScript, Python, Kotlin, Swift, and others are not just possible, they are expected. The reference Rust implementation is the canonical artifact against which those SDKs are verified. Section 4.2.1 describes how the project is structured to support this.

**Scope: Phase 1 only.** This chapter covers the Minimal Viable Protocol defined in Chapter 3 Phase 1. Phase 2 implementation guidance will be written after Phase 1 is stable and tested.

---

## Chapter 4 — Section Skeleton

**Phase 1 — Reference Implementation**

| Section | Title | Status |
|---|---|---|
| 4.1 | Implementation Philosophy | ✅ Complete |
| 4.2 | Technology Stack | ✅ Complete |
| 4.3 | Project Structure | ✅ Complete |
| 4.4 | Build Order | ✅ Complete |
| 4.5 | Wire Format Implementation | ✅ Complete |
| 4.6 | Cryptographic Primitives | ✅ Complete |
| 4.7 | Event Implementation | ✅ Complete |
| 4.8 | Transport Layer Implementation | ✅ Complete |
| 4.9 | Identity and Registration Implementation | ✅ Complete |
| 4.10 | Space and Room Implementation | ✅ Complete |
| 4.11 | Federation Implementation | ✅ Complete |
| 4.12 | Event Store | ✅ Complete |
| 4.13 | Auth Module — Tier 1 Implementation | ✅ Complete |
| 4.14 | Local Node Mode | ✅ Complete |
| 4.15 | Smoke Test Execution | ✅ Complete |
| 4.16 | CLI Reference | ✅ Complete |

---

## Phase 1 — Reference Implementation

### 4.1 Implementation Philosophy

Three principles shape every implementation decision in this chapter.

**Pattern A — folder is the application**

The XGen Node and reference client ship as self-contained directories. Everything the program needs — the binary, the configuration file, the database, the log output — lives in one folder. The operator copies the folder to deploy, deletes the folder to uninstall. There is no system-wide installation, no registry entries, no scattered configuration files.

This principle is already established for key file handling in 3.5.1, where the exception taxonomy is documented. Chapter 4 operationalises it: the project structure in 4.3 and the configuration format in 4.8 both follow Pattern A. The rule is: if it belongs to the program, it lives in the program's folder unless there is a structural reason it cannot.

**Local Node first**

All Phase 1 development and testing uses Local Node mode. Local Node mode (defined in 3.1.1 and 3.8.8) disables Auth Module requirements and restricts connections to localhost. A developer can run the full smoke test sequence — two Nodes, two Identities, one Space, message exchange — on a single machine with no external dependencies and no Auth Module infrastructure.

Local Node mode is not a simplified version of the protocol. It is the full protocol running in a constrained environment. The same Event validation pipeline, the same signature verification, the same federation handshake all execute in Local Node mode. The only differences are the bypass of Trust Assertion requirements and the localhost-only network restriction.

Production federation with external Nodes and a real Auth Module is the second step, after the smoke test passes cleanly on localhost.

**Protocol fidelity over convenience**

The reference implementation is the spec made executable. It is not a product and it is not optimised for end-user experience at this stage. When the spec requires a specific field order for canonical form computation, the implementation encodes that field order explicitly — it does not rely on HashMap iteration order or any other non-deterministic behaviour. When the spec requires a specific error code, the implementation returns that exact code — it does not approximate.

This discipline matters because the reference implementation will be used to verify future SDKs in other languages. An implementation that takes shortcuts in signature canonicalisation or Event ID derivation will produce results that do not interoperate with compliant implementations.

Convenience, ergonomics, and performance optimisations are appropriate at a later stage, after protocol correctness is established.

---

### 4.2 Technology Stack

#### 4.2.1 Language and Multi-SDK Strategy

The reference implementation is written in **Rust**. Rust was chosen for four reasons that align directly with XGen's design values.

First, memory safety without a garbage collector. XGen Nodes are expected to run continuously for months. A Node that crashes due to a memory error in a session handler is a reliability failure. Rust eliminates whole categories of runtime errors at compile time.

Second, explicit error handling. Rust's `Result<T, E>` type forces every error path to be acknowledged at the call site. In a protocol implementation where incorrect error handling can lead to silent data loss or incorrect state, this explicitness is a feature, not a burden.

Third, strong type system. The XGen wire format has several distinct identifier types — `pubkey_uri`, `hash_uri`, `xgen_uri` — that carry different semantics and validation rules. Rust's newtype pattern allows these to be distinct types at compile time, preventing accidental interchange.

Fourth, excellent async support via `tokio`. XGen Nodes handle many concurrent connections — clients and federated peers. Async I/O is the correct model, and `tokio` is the mature, well-documented async runtime for Rust.

**Multi-SDK strategy**

The Rust project is structured as a Cargo workspace with `xgen-core` as a library crate (4.3). The library crate contains all protocol logic: wire format types, Event construction and validation, cryptographic operations, transport state machines, and federation state machines. The Node binary and the reference client are thin wrappers over this library.

This structure serves two purposes. First, it enforces a clean separation between protocol logic and application logic during Phase 1 development. Second, it produces a publishable artifact: `xgen-core` on crates.io as the canonical Rust SDK.

Future community SDKs in other languages implement the same wire format and state machines independently. They are verified for correctness by running the smoke test (4.15) against the reference Rust Node — a Go client that can successfully complete the 17-step sequence with a Rust Node is a conformant Go client. No shared code is required; only a shared protocol.

#### 4.2.2 Crate Selections

The following crates are recommended for the reference implementation. These are recommendations, not hard requirements — an alternative crate that satisfies the same interface contract is acceptable. The rationale for each choice is given so that a future implementer choosing a different library understands what properties matter.

**Async runtime**

`tokio` (latest stable) is the async runtime. It provides the task scheduler, the async I/O primitives, and the timer utilities that the transport layer requires. No alternative is recommended — `tokio` is the de facto standard and the ecosystem of compatible crates assumes it.

**WebSocket**

`tokio-tungstenite` provides async WebSocket support built on `tokio`. It handles the WebSocket handshake, framing, and ping/pong at the library level, leaving the application responsible only for the XGen transport framing defined in 3.1.2. An alternative implementer might use `axum` with its built-in WebSocket support — either is acceptable provided the transport framing behaviour matches the spec.

**Ed25519 signatures**

`ed25519-dalek` from the RustCrypto project is the recommended Ed25519 implementation. It has undergone external security audit and is widely used in production cryptographic software. The key generation, signing, and verification APIs are straightforward. The crate works correctly alongside `rand_core` for key generation entropy.

The property that matters: the signature produced by `ed25519-dalek` for a given private key and message must be identical to the signature produced by any other compliant Ed25519 implementation for the same inputs. Ed25519 is a deterministic signature scheme — there is no randomness in signing. Any conformant implementation produces the same output.

**SHA-256**

`sha2` from the RustCrypto project provides SHA-256. It is the same ecosystem as `ed25519-dalek` and integrates cleanly with `digest` traits. Used for Event ID derivation (3.2.3) and Space/Room ID derivation (3.7.2).

**JSON serialisation**

`serde` with `serde_json` is the standard JSON serialisation layer for Rust. All protocol message types implement `serde::Serialize` and `serde::Deserialize`. The canonical form for signature computation (3.2.4) requires custom serialisation — it cannot rely on `serde_json`'s default output because JSON key order is not guaranteed by the standard. The canonical form serialiser must be written explicitly (see 4.5.2).

**Base64url encoding**

`base64` crate with the `URL_SAFE_NO_PAD` alphabet. This is the encoding used for public keys, signatures, and content hashes in URI fields. The `NO_PAD` variant matches the XGen spec requirement (3.1.9) — no trailing `=` characters.

**Event store (SQLite)**

`sqlx` with the SQLite backend provides async SQLite access. The Event store (4.12) is an append-only SQLite database — one database file per Space, living in the Node's application folder. `sqlx` was chosen over `rusqlite` because it integrates with `tokio` natively and supports compile-time query checking against the schema.

The schema is simple: Events are stored as JSON blobs indexed by `event_id`. The Event store does not need to be a relational engine — it needs to support append, lookup by ID, and ordered scan by DAG position. SQLite is a straightforward choice that satisfies Pattern A (the database file lives in the application folder).

**Configuration**

`toml` crate for parsing `node_config.toml` and `client_config.toml`. TOML was chosen over JSON for configuration files because it is more human-writable (comments are supported, quoting is less strict) and because it is idiomatic in the Rust ecosystem. The config schema is defined in 4.8.

**CLI interface**

`clap` with the derive macro for both the Node binary and the reference client CLI. The reference client is a command-line tool used for smoke testing — it does not have a GUI. `clap` provides argument parsing, help text generation, and subcommand support.

**Logging**

`tracing` with `tracing-subscriber` for structured logging. Log output follows the Pattern A rule — it goes to stdout by default and can be redirected by the operator. The `tracing` ecosystem supports structured fields (event IDs, node IDs, connection identifiers) which are essential for debugging protocol sequences. Log level is configured via the standard `RUST_LOG` environment variable.

**Random number generation**

`rand` crate with `OsRng` for all cryptographic randomness — nonce generation, key generation, Space/Room ID nonces. `OsRng` reads from the OS-provided entropy source and is the correct choice for cryptographic use. Non-cryptographic randomness (e.g. reconnection backoff jitter) may use `rand`'s thread-local RNG.

#### 4.2.3 What Is Not Included

The following are explicitly out of scope for Phase 1:

- **TLS termination**: Local Node mode uses unencrypted WebSocket (`ws://`). Production TLS is expected to be handled by a reverse proxy (nginx, Caddy) in front of the Node, not by the Node itself. This keeps the Node binary simple and follows common deployment practice. The Node's endpoint URI is configured to match whatever the proxy exposes.
- **HTTP REST API**: XGen is a WebSocket protocol. There is no HTTP REST API in Phase 1. Future management endpoints (admin dashboard, health check) are Phase 2.
- **GUI client**: The reference client is a CLI tool. A GUI client is future work and is out of scope for this chapter.
- **Database migrations**: Phase 1 uses a fixed schema. Migration tooling is Phase 2.

---

### 4.3 Project Structure

The XGen reference implementation is a Cargo workspace. The workspace root contains a `Cargo.toml` that declares the member crates. Each crate has a focused responsibility.

```
xgen/
  Cargo.toml                  ← workspace root
  Cargo.lock                  ← committed to version control
  LICENSE
  README.md
  docs/                       ← protocol documentation (this document)
  xgen-core/                  ← protocol library crate
    Cargo.toml
    src/
      lib.rs
      wire/                   ← 3.1 Wire Format types and primitives
        mod.rs
        types.rs              ← XgenUri, HashUri, PubkeyUri, Datetime, etc.
        canonical.rs          ← canonical form serialiser
        framing.rs            ← transport frame encode/decode
      crypto/                 ← 4.6 Ed25519 keypair, signing, verification
        mod.rs
        keypair.rs
        signature.rs
        hashing.rs
      event/                  ← 3.2 Event types, validation pipeline
        mod.rs
        envelope.rs           ← Event struct and field definitions
        types.rs              ← EventType registry
        validation.rs         ← 13-step validation pipeline
        dag.rs                ← prev_events DAG logic
      transport/              ← 3.3 Transport protocol state machine
        mod.rs
        messages.rs           ← transport.* message types
        lifecycle.rs          ← CONNECT/AUTHENTICATE/ACTIVE/CLOSE state machine
        errors.rs             ← 1xxx error codes
      federation/             ← 3.4 Federation handshake state machine
        mod.rs
        messages.rs           ← federation.* message types
        handshake.rs          ← IDLE/HELLO/CAPS/ACTIVE/CLOSED state machine
        errors.rs             ← 2xxx error codes
      node_identity/          ← 3.5 Node keypair, announcement
        mod.rs
        keypair.rs
        announcement.rs
      identity/               ← 3.6 Identity registration and updates
        mod.rs
        registration.rs
        record.rs
        errors.rs             ← 3xxx error codes
      space/                  ← 3.7 Space, Room, membership
        mod.rs
        space.rs
        room.rs
        membership.rs
        state.rs              ← Space state and Room state derivation
      auth/                   ← 3.8 Auth Module interface and Tier 1
        mod.rs
        interface.rs          ← Auth Module interface contract
        assertion.rs          ← Trust Assertion schema and validation
        tier1.rs              ← Tier 1 verification states
  xgen-node/                  ← Node binary
    Cargo.toml
    src/
      main.rs
      config.rs               ← node_config.toml loading and validation
      server.rs               ← WebSocket server, connection dispatch
      store/
        event_store.rs        ← append-only SQLite Event store
        identity_store.rs     ← Identity registry
        federation_store.rs   ← Federation registry
  xgen-client/                ← Reference CLI client
    Cargo.toml
    src/
      main.rs
      config.rs               ← client_config.toml loading
      commands/
        register.rs           ← identity registration flow
        space.rs              ← space and room creation
        message.rs            ← send and receive messages
        smoke_test.rs         ← automated smoke test runner (4.15)
  xgen-auth-module/           ← Tier 1 Auth Module reference implementation
    Cargo.toml
    src/
      main.rs
      config.rs
      verification.rs         ← email and phone verification flows
      assertion.rs            ← Trust Assertion issuance
      store.rs                ← verification state persistence
```

**Why this structure**

The `xgen-core` library crate contains everything that a future SDK in another language would need to reimplement. It has no binary dependencies, no network I/O, no file I/O. It is pure protocol logic: types, state machines, and cryptographic operations. A developer building a Go XGen client does not link against `xgen-core` — they reimplement the same logic in Go. But the structure of `xgen-core` serves as a specification of what that reimplementation must cover.

The `xgen-node` and `xgen-client` binaries are thin wrappers. They handle I/O (WebSocket connections, file reads, database access) and delegate all protocol logic to `xgen-core`. This boundary is enforced structurally: `xgen-core` has no `tokio` dependency and no file system access. It receives data and returns data. The binaries decide when and how to perform I/O.

**Application folder layout at runtime**

A deployed Node has the following folder structure, following Pattern A (D-025 — all files prefixed `xgen-node_*`):

```
xgen-node/                      ← the application folder
  xgen-node.exe                 ← the binary
  xgen-node_config.toml         ← configuration
  xgen-node_keypair.enc         ← encrypted Ed25519 keypair (default location)
  xgen-node_state.json          ← live status snapshot, updated every 5s (D-026)
  xgen-node_identities.db       ← identity registry (SQLite)
  xgen-node_federation.db       ← federation registry (SQLite)
  spaces/                       ← one SQLite database per Space
    <space_id_hex>.db
  logs/
    xgen-node.log               ← optional, if log_path configured
```

The reference client folder follows the same pattern:

```
xgen-client/                    ← the application folder
  xgen-client.exe               ← the binary
  xgen-client_config.toml       ← configuration
  xgen-client_keypair.enc       ← encrypted Ed25519 keypair (default location)
  xgen-client_state.json        ← identity, known nodes, joined spaces (D-026)
  logs/
    xgen-client.log             ← optional, if log_path configured
```

---

### 4.4 Build Order

The recommended implementation sequence for Phase 1. This order is not alphabetical by spec section — it is causal. Each step produces a testable artifact that the next step builds on. A developer who follows this order will have working, testable code at each stage rather than a large partially-complete system.

```
Step 1 — Wire format primitives
  Implement XgenUri, HashUri, PubkeyUri as newtypes.
  Implement Datetime as an RFC 3339 UTC wrapper.
  Implement base64url encode/decode.
  Implement the transport frame encode/decode.
  Unit test: round-trip each type through serialisation and deserialisation.
  Reference: 3.1

Step 2 — Cryptographic primitives
  Implement Ed25519 keypair generation.
  Implement signing and verification.
  Implement SHA-256 hashing.
  Implement canonical form serialiser.
  Unit test: sign a known message, verify signature, confirm determinism.
  Unit test: canonical form produces the same bytes regardless of struct field order.
  Reference: 3.2.4, 4.6

Step 3 — Event types and ID derivation
  Implement the Event envelope struct.
  Implement EventType enum.
  Implement Event ID derivation (SHA-256 of canonical form without event_id and signature).
  Unit test: derive Event ID from a known Event, confirm it matches expected hash.
  Reference: 3.2.1, 3.2.2, 3.2.3

Step 4 — Transport WebSocket skeleton
  Stand up a tokio-tungstenite WebSocket server that accepts connections.
  Implement transport frame parsing on incoming messages.
  Implement the CONNECT phase — accept a connection without authentication.
  Smoke test: client connects to Node, Node accepts.
  Reference: 3.3.1, 3.3.2, 3.3.3

Step 5 — Challenge-response authentication
  Implement transport.challenge issuance on connection.
  Implement transport.auth parsing and nonce verification.
  Implement transport.auth_ok and transport.auth_fail.
  Implement the AUTHENTICATE phase state machine.
  Smoke test: client authenticates, Node confirms, connection enters ACTIVE.
  Reference: 3.3.4

Step 6 — Local Node Identity registration
  Implement identity.register parsing.
  Implement the 8-step acceptance pipeline (Local Node mode — steps 4–7 skipped).
  Implement identity.register_ok and identity.register_fail.
  Implement Identity record storage (SQLite).
  Smoke test: client registers Identity on Local Node, receives register_ok.
  Reference: 3.6.3, 3.6.4, 3.6.9

Step 7 — Space and Room creation
  Implement state.space_create Event construction and validation.
  Implement state.room_create Event construction and validation.
  Implement Space ID and Room ID derivation.
  Implement Space state and Room state derivation from Events.
  Implement the Event store (append-only SQLite) for storing Events.
  Smoke test: Alice creates a Space and a Room, both IDs derived correctly.
  Reference: 3.7.2, 3.7.3, 3.7.5, 4.12

Step 8 — Event validation pipeline
  Implement all 13 steps of the Event validation pipeline.
  Unit test: one test per validation step — each test exercises the pass and fail
             path for that step in isolation.
  Integration test: send a valid message.text Event through the full pipeline.
  Reference: 3.2.6

Step 9 — Federation handshake
  Implement Node keypair generation and Node announcement construction.
  Implement federation.hello / federation.capabilities / federation.accept
  message construction and parsing.
  Implement the federation handshake state machine
  (IDLE → HELLO_RECEIVED → CAPS_SENT → ACTIVE → CLOSED).
  Implement state.federation_add Event production on handshake completion.
  Smoke test: Node B connects to Node A, handshake completes,
              state.federation_add recorded in Space DAG.
  Reference: 3.4, 3.5

Step 10 — Federation Event sync
  Implement transport.sync_request / sync_response.
  Implement Event delivery from home Node to federated Node on new Event.
  Implement the pending Event buffer for unknown prev_events references.
  Smoke test: Alice sends a message.text, Node B receives and stores it.
  Reference: 3.3.6, 3.4.6

Step 11 — Membership Events
  Implement membership.invite / membership.join / membership.leave
  Event construction and validation.
  Implement role permission checks for membership Events.
  Smoke test: Alice invites Bob, Bob joins Space and Room.
  Reference: 3.7.8, 3.7.9

Step 12 — Full smoke test
  Execute the complete 17-step sequence from 3.7.11.
  All steps pass. Both Nodes have both messages in their Room DAG.
  Phase 1 implementation complete.
  Reference: 3.7.11, 4.15

Step 13 — Auth Module Tier 1 (optional for smoke test, required for production)
  Implement email and phone verification flows.
  Implement Trust Assertion issuance.
  Implement Node-side Trust Assertion validation (steps 4–7 of 3.6.4).
  Smoke test: run the full smoke test with Auth Module enabled (Tier 1 production mode).
  Reference: 3.8, 4.13
```

Steps 1–12 can be completed without any external services. Step 13 requires an email or SMS sending integration for the verification codes. A developer running Phase 1 for the first time should complete steps 1–12 first and treat step 13 as a separate milestone.

---

### 4.5 Wire Format Implementation

#### 4.5.1 URI Types

The three URI types defined in 3.1.6 — `XgenUri`, `HashUri`, and `PubkeyUri` — SHOULD be implemented as Rust newtypes wrapping `String`, not as plain strings. This prevents accidental interchange between URI types at compile time.

Each newtype implements:
- `TryFrom<String>` with validation of the URI grammar
- `Display` for formatting into JSON fields
- `serde::Serialize` and `serde::Deserialize` with validation on deserialisation
- `Eq`, `Hash` for use as map keys

Validation at deserialisation time ensures that any `HashUri` value received in a protocol message has been verified to match the `xgen://hash/<algorithm>:<hexbytes>` grammar before it is used anywhere in the application.

For Phase 1, the algorithm segment of `HashUri` MUST be `sha256` and the algorithm segment of `PubkeyUri` MUST be `ed25519`. Deserialisation MUST reject values with any other algorithm. This constraint is relaxed in Phase 2 when the Algorithm Registry is defined.

#### 4.5.2 Canonical Form Serialiser

The canonical form serialiser is the most important piece of wire format infrastructure. All signature computation and Event ID derivation depends on it producing the same byte sequence for the same logical Event regardless of how the Event was constructed.

The canonical form is NOT the output of `serde_json::to_string()` applied to the Event struct. `serde_json` does not guarantee field ordering — the struct's derive macros determine field order based on struct declaration order, which may not match the canonical field order required by the spec (3.2.4).

The canonical form serialiser MUST:
- Include exactly the fields specified in 3.2.4 (excluding `event_id` and `signature`)
- Emit fields in the exact canonical order: `protocol_version`, `type`, `sender`, `room_id`, `space_id`, `prev_events`, `timestamp`, `content`, `meta_atts`
- Sort all object keys within `content` and `meta_atts` lexicographically (Unicode code point order, which is byte order for valid UTF-8 strings)
- Emit no whitespace outside string values
- Normalise Unicode escape sequences to literal UTF-8

The recommended implementation is a custom serialiser function that builds the JSON string manually for the fixed outer fields and delegates only the `content` and `meta_atts` values to a recursive key-sorting serialiser.

A correct implementation passes the following invariant: for any two Event structs with identical field values, the canonical form bytes are identical — including when the structs were constructed from different JSON input with different field ordering.

#### 4.5.3 Transport Frame Codec

The transport frame codec (3.1.2) reads and writes the length-prefixed format identifier + payload structure. It MUST be implemented as a stateful codec that can operate incrementally — a WebSocket message is delivered as a complete unit by `tokio-tungstenite`, so for XGen's one-frame-per-message model the codec is straightforward: read the frame from the complete WebSocket message bytes, deserialise the payload, and return.

The codec MUST validate:
- The format identifier length byte is within bounds
- The format identifier string is a valid UTF-8 sequence
- The payload length field matches the actual remaining bytes
- The format identifier is a known registered format (for Phase 1: `json` only; `msgpack` and `cbor` if capability negotiation selected them)

A malformed frame results in immediate connection termination (3.3.3). The codec MUST NOT attempt to recover from a malformed frame.

#### 4.5.4 Datetime Handling

Datetime values in XGen protocol messages use RFC 3339 UTC with millisecond precision (3.1.7). The implementation MUST:
- Store timestamps as a dedicated `XgenDatetime` newtype, not as a raw string or a generic timestamp type
- Validate on deserialisation that the timestamp conforms exactly to the format `YYYY-MM-DDTHH:MM:SS.mmmZ`
- Reject timestamps with timezone offsets, date-only values, or missing milliseconds
- Produce correct UTC timestamps when creating Events, regardless of the Node's local timezone

The `chrono` crate with UTC timezone support is a suitable implementation choice. Alternatively, `time` crate provides similar functionality. The key requirement is that timestamp generation always produces UTC regardless of the host system's timezone configuration.

---

### 4.6 Cryptographic Primitives

#### 4.6.1 Keypair Generation

Node keypair generation (3.5.1) and client Identity keypair generation (3.6.1) both use the same Ed25519 keypair generation function. The difference is where the resulting keypair is stored and how it is used.

The generation function:
1. Sources entropy from `OsRng` — the OS-provided cryptographically secure random number generator
2. Generates an Ed25519 keypair using `ed25519-dalek`
3. Returns the keypair as a struct containing the public key bytes and the secret key bytes

The function MUST check for key file existence before generating. If a key file already exists at the configured path, the function MUST load the existing key and MUST NOT generate a new one. Accidental key regeneration breaks the Node ID, all existing federation relationships, and all Trust Assertions issued for that Identity.

The private key is stored as an encrypted file. For Phase 1, the encryption uses a passphrase-derived key via Argon2id (from the `argon2` crate, m=64MB, t=3, p=1) to derive a 32-byte key, which then encrypts the 32-byte Ed25519 secret key using **ChaCha20-Poly1305** AEAD (from the `chacha20poly1305` crate). ChaCha20-Poly1305 was chosen over AES-256-GCM because it has no timing side-channels from table lookups and does not require AES hardware acceleration — correct on all target hardware. The file is stored as JSON (D-002):

```json
{
  "version": 1,
  "algorithm": "chacha20poly1305",
  "kdf": "argon2id",
  "salt": "<base64url, 32 bytes>",
  "nonce": "<base64url, 12 bytes>",
  "ciphertext": "<base64url, 48 bytes = 32-byte key + 16-byte AEAD tag>"
}
```

The passphrase is entered by the operator at startup (for a Node) or by the user (for a client). It is never stored. A Node startup script MAY read the passphrase from an environment variable for automated deployments — this is an operational decision, not a protocol requirement.

#### 4.6.2 Signing

Event signing follows the canonical form process:
1. Compute the canonical form bytes (4.5.2)
2. Sign the canonical form bytes with `ed25519-dalek`'s `Signer` trait
3. Base64url-encode the signature bytes (no padding)
4. Format the signature field: `"ed25519:<base64url-pubkey>:<base64url-signature>"`

Signing is deterministic — the same canonical form and private key always produce the same signature bytes. This property is guaranteed by Ed25519.

#### 4.6.3 Verification

Event signature verification:
1. Parse the `signature` field to extract algorithm, public key, and signature bytes
2. Verify that the extracted public key matches the `sender` field's public key component
3. Compute the canonical form of the received Event (4.5.2)
4. Verify the signature using `ed25519-dalek`'s `Verifier` trait

Verification MUST fail explicitly — the function returns `Ok(())` on success and `Err(SignatureError)` on failure. A failed verification MUST be treated as a hard rejection (step 12 of the Event validation pipeline, 3.2.6).

#### 4.6.4 Event ID and Space/Room ID Derivation

Event ID derivation (3.2.3):
1. Compute the canonical form (excluding `event_id` and `signature`)
2. SHA-256 hash the canonical form bytes
3. Hex-encode the hash (lowercase)
4. Construct `HashUri`: `xgen://hash/sha256:<hexstring>`

Space ID and Room ID derivation (3.7.2) follow the same process applied to the canonical form of their creation Events. The nonce field in the creation Event content guarantees uniqueness.

---

### 4.7 Event Implementation

#### 4.7.1 Event Struct

The Event envelope struct maps directly to the schema in 3.2.1. In Rust, the struct is defined in `xgen-core/src/event/envelope.rs` and derives `serde::Serialize` and `serde::Deserialize`. All field types are the wire format newtypes from 4.5 — there are no raw `String` fields for typed values.

The `content` field is typed as `EventContent`, which is an enum over all known EventType payloads plus an `Unknown(serde_json::Value)` variant for forward compatibility. When an unknown EventType is received, the content is stored as an opaque JSON value and the Event is accepted and propagated — the unknown EventType handling rule from 3.2.2.

The `prev_events` field is a `Vec<HashUri>` with a maximum length of 10 enforced at deserialisation.

#### 4.7.2 Event Validation Pipeline

The 13-step validation pipeline (3.2.6) is implemented as a single function that takes an `Event` and the current `NodeContext` (which carries the Space state, the Identity registry, and the federation registry) and returns `Result<ValidatedEvent, ValidationError>`.

The function applies checks in strict order. Each check returns early on failure with a typed `ValidationError` variant corresponding to the error condition. The typed error is then mapped to the appropriate rejection response for the connection.

Step 9 (predecessor check) has a different return path — it returns a `PendingEvent` result rather than a `ValidationError`. The caller holds the Event in a pending buffer and requests the missing predecessors from peers via `transport.sync_request`.

**Critical implementation note**: Steps 1–7 are pure structural validation requiring no I/O or cryptographic operations. Steps 8 and 12 require CPU-bound cryptographic work. Step 11 and 13 require I/O (database lookups). The implementation MUST apply steps in the specified order — do not reorder for perceived efficiency. The ordering is the spec.

#### 4.7.3 DAG Operations

The DAG management code in `xgen-core/src/event/dag.rs` provides two operations that the Node uses during Event processing:

**Get current tips**: returns the set of Event IDs that have no successors in the current Room DAG. Used by a client or Node when constructing a new Event — `prev_events` MUST reference all current tips.

**Check for cycles**: given a new Event's `prev_events`, verifies that none of the referenced Events are descendants of Events already in the pending batch. In Phase 1 with sequential message flow, this check will rarely trigger. It must nonetheless be implemented correctly.

---

### 4.8 Transport Layer Implementation

#### 4.8.1 Configuration

The Node's configuration file is `xgen-node_config.toml`, located in the application folder. Minimum required fields for Phase 1:

```toml
# xgen-node_config.toml

[node]
operator_display_name = "My XGen Node"
keypair_path = "./xgen-node_keypair.enc"   # default location
local_node = false

[network]
endpoint = "ws://127.0.0.1:8080/xgen" # for Local Node mode
# endpoint = "wss://node.example.org:8443/xgen"  # for production
bind_address = "127.0.0.1"            # for Local Node mode; "0.0.0.0" for production
bind_port = 8080

[storage]
spaces_dir = "./spaces"
identities_db = "./identities/identities.db"
federation_db = "./federation/federation.db"

[auth]
local_node_bypass = true               # matches local_node above
# trusted_auth_modules = ["./auth_modules/module1.json"]  # for production
```

When `local_node = true`, the Node MUST verify at startup that `bind_address` is a loopback address.

#### 4.8.2 Connection Dispatch

The Node's WebSocket server accepts incoming connections and spawns a `tokio` task per connection. Each task runs the connection lifecycle state machine (3.3.4):

```
task spawned ──► CONNECT phase
                 ↓
               AUTHENTICATE phase (challenge → auth → auth_ok)
                 ↓
               ACTIVE phase (Event exchange loop)
                 ↓
               CLOSE phase (goodbye or drop)
```

The connection type — client or Node-to-Node — is determined during the AUTHENTICATE phase. A client authenticates with an `identity_id` that matches a registered Identity. A peer Node authenticates with a `node_id` that matches a known Node announcement. The subsequent message processing differs by connection type.

All active connections are tracked in a shared `ConnectionRegistry` (a `tokio::sync::RwLock<HashMap>`) so that the Node can fan out incoming Events to all connected clients subscribed to the relevant Space and Room.

#### 4.8.3 Keepalive Implementation

The keepalive timer is a `tokio::time::interval` running every 30 seconds in each connection task. On each tick, the task sends a WebSocket ping frame via `tokio-tungstenite`. A separate `tokio::time::timeout` wraps the pong wait — if no pong is received within 10 seconds, the task closes the connection.

`tokio-tungstenite` surfaces WebSocket ping/pong as `Message::Ping` and `Message::Pong` variants in its message stream. The connection task handles these transparently alongside protocol messages.

#### 4.8.4 Error Code Implementation

Transport error codes (3.3.8) are implemented as a Rust enum `TransportError` with variants corresponding to the 1xxx codes. The enum derives `serde::Serialize` so that `transport.auth_fail` and `transport.error` messages can be constructed directly from the enum value. Each variant carries the numeric code and string name as associated constants.

Federation error codes (3.4.7) follow the same pattern as `FederationError`. Identity registration error codes (3.6.5) as `RegistrationError`. All three enums implement a common `ProtocolError` trait that provides `code()` and `error_string()` methods.

#### 4.8.5 Node Startup State Reconstruction (hard requirement)

On startup, a Node MUST reconstruct all in-memory state from its persisted Event stores before opening the network listener. This is a hard requirement, not an optimisation — a Node that accepts connections before state reconstruction is complete will reject legitimate Events from clients referencing Spaces from previous sessions.

**Startup sequence:**

```
1. Load and decrypt keypair
2. Read node_config.toml
3. Load identity registry from xgen-node_identities.db
4. Scan the spaces directory for all Space Event stores (*.db files)
5. For each Space database found, in any order:
   a. Open the database
   b. Read all Events in causal order (topological sort: parents before children)
   c. Apply each Event to reconstruct current in-memory state via ingest_event:
      - state.space_create      → register Space in memory
      - state.room_create       → register Room under its Space
      - state.dm_space_create   → register DM Space in memory
      - membership.join         → add Identity to Space membership
      - membership.leave / kick / ban → remove Identity from Space membership
      - state.federation_add / remove → reconstruct federation registry
      - state.node_priority     → reconstruct Node priority ordering
      - state.room_name / topic / avatar → update Room state
6. Only after all databases are replayed: open network listener and accept connections
```

**The principle:** the Event store is the source of truth. In-memory state is always derived from it, never the other way around. A Node that has replayed its Event log is in exactly the same state as a Node that has been running continuously since genesis.

**Secondary requirement:** A Node receiving a `membership.join` Event for a Space it does not recognise MUST reject it with a clear `space_not_found` error (rather than silently accepting the join and failing later when the client sends a message). This makes the failure visible at the correct point in the protocol flow.

---

### 4.9 Identity and Registration Implementation

#### 4.9.1 Identity Record Storage

The Identity registry is a SQLite database (`identities.db`) with a single table:

```sql
CREATE TABLE identities (
    identity_id     TEXT PRIMARY KEY,   -- pubkey_uri string
    display_name    TEXT,
    registered_at   TEXT NOT NULL,      -- RFC 3339 UTC
    trust_assertion TEXT,               -- JSON blob, nullable
    devices         TEXT NOT NULL,      -- JSON array blob
    home_node       TEXT NOT NULL,      -- pubkey_uri string
    update_version  INTEGER NOT NULL DEFAULT 0
);
```

The `trust_assertion` column is nullable — it is NULL for Local Node registrations. The `devices` column stores the devices array as a JSON blob. The schema is intentionally flat — there are no separate devices table or Trust Assertion table in Phase 1. Normalisation is Phase 2.

#### 4.9.2 Registration Flow

The registration handler receives an `identity.register` message on an ACTIVE connection and runs the 8-step acceptance pipeline (3.6.4). The pipeline is implemented as sequential database and cryptographic checks, each returning early on failure.

For Local Node mode, steps 4–7 are behind a feature flag checked at the start of the pipeline:

```rust
if !config.auth.local_node_bypass {
    // steps 4–7: Trust Assertion validation
}
```

On success, the Identity record is inserted into `identities.db` and `identity.register_ok` is sent. The registration does not close the connection — the client remains connected and can immediately proceed to Space operations.

#### 4.9.3 Identity Retrieval and Federation

For Phase 1 with two Nodes, Identity record sharing is handled directly over the active federation connection. When a new Identity registers on Node A, Node A sends the full Identity record to Node B via a `identity.record` message on the Node-to-Node connection. Node B stores it in its local `identities.db`.

This direct push model is sufficient for Phase 1. The N-replica propagation model described in 3.6.7 and 3.13 is Phase 2.

---

### 4.10 Space and Room Implementation

#### 4.10.1 Space State Derivation

Space state (3.7.6) is derived by replaying all State Events in the Space's Event log in causal order and applying each one to a mutable `SpaceState` struct. For Phase 1, State Events arrive sequentially — there are no concurrent state changes that require conflict resolution. The most recent State Event of each type is applied last and is therefore authoritative.

The `SpaceState` struct is held in memory by the Node for each active Space, updated on each new State Event, and recomputed from the Event log on Node restart. It is not stored separately in the database — the Event log is the source of truth, and the in-memory state is a derived cache.

#### 4.10.2 Event Store Schema

The Event store schema is described in 4.12. From the Space and Room protocol's perspective, the key operations on the Event store are:

- **Append**: store a validated Event by `event_id`
- **Get by ID**: retrieve an Event by its `event_id` — used for predecessor lookups during validation
- **Get Room history**: retrieve all Events for a `room_id` in DAG order — used for history sync (3.3.6) and state derivation
- **Get DAG tips**: retrieve the current tip Event IDs for a Room — used when constructing new Events

#### 4.10.3 Membership Event Processing

Membership Events (3.7.8, 3.7.9) update the member list in the in-memory `SpaceState` or `RoomState`. The processing is straightforward for Phase 1:

- `membership.invite`: add the target Identity to the Space's pending invite list
- `membership.join`: move the Identity from the pending invite list to the active member list, assign the offered role
- `membership.leave` / `membership.kick` / `membership.ban`: remove the Identity from the active member list (ban adds to a ban list)

Role permission checking for authorisation (step 13 of the Event validation pipeline) consults the `SpaceState.member_list` to determine the sender's current role before accepting a membership action.

---

### 4.11 Federation Implementation

#### 4.11.1 Federation Handshake State Machine

The federation handshake (3.4) is implemented as an explicit state machine with states matching 3.4.3:

```rust
enum FederationState {
    Idle,
    HelloReceived { peer_node_id: PubkeyUri, capabilities: Capabilities },
    CapsSent { negotiated: NegotiatedCapabilities },
    Active { session_id: HashUri, peer_node_id: PubkeyUri },
    Closed,
}
```

The state machine is driven by incoming messages on the Node-to-Node connection. An unexpected message type for the current state triggers `federation.reject` with error code `2005` and connection close.

Timeout handling: a `tokio::time::timeout` wraps the wait for responses in `HelloReceived` and `CapsSent` states. A 15-second timeout triggers connection close with logging. The initiating Node may retry after the reconnection backoff (3.3.6).

#### 4.11.2 Federation Registry

The federation registry is a SQLite database (`federation.db`) that persists active federation relationships across Node restarts:

```sql
CREATE TABLE federation_relationships (
    peer_node_id            TEXT NOT NULL,
    space_id                TEXT NOT NULL,
    session_id              TEXT NOT NULL,
    negotiated_version      TEXT NOT NULL,
    negotiated_serial       TEXT NOT NULL,
    last_connected_at       TEXT NOT NULL,
    PRIMARY KEY (peer_node_id, space_id)
);

CREATE TABLE peer_announcements (
    node_id                 TEXT PRIMARY KEY,
    announcement_json       TEXT NOT NULL,
    announcement_version    INTEGER NOT NULL,
    valid_until             TEXT NOT NULL
);
```

On Node startup, the federation registry is consulted to re-establish connections with known peers. The Node attempts to reconnect to each registered peer using the stored endpoint from the peer's announcement, applying the reconnection backoff from 3.3.6.

#### 4.11.3 Event Fan-out

When the Node accepts a new Event (passes all 13 validation steps), it performs three actions in sequence:

1. **Store**: append the Event to the Space's Event store
2. **Fan out to local clients**: deliver the Event to all connected clients subscribed to the relevant Room
3. **Fan out to federated peers**: deliver the Event to all federated peer Nodes that participate in the relevant Space

Fan-out to federated peers wraps the Event in a transport frame and sends it over the active Node-to-Node WebSocket connection. If the connection to a peer is temporarily down, the Event is held in a per-peer outbound queue. When the peer reconnects, the queue is flushed and the peer sends a `transport.sync_request` to catch up on any Events it missed.

---

### 4.12 Event Store

The Event store is the append-only persistence layer for the XGen Event log. It is the most critical component of the Node — the Event log is the ground truth for all Space and Room state.

#### 4.12.1 Schema

One SQLite database per Space, stored at `spaces/<space_id_hex>.db`:

```sql
CREATE TABLE events (
    event_id        TEXT PRIMARY KEY,   -- hash_uri string
    room_id         TEXT NOT NULL,
    space_id        TEXT NOT NULL,
    event_type      TEXT NOT NULL,      -- EventType string for fast filtering
    sender          TEXT NOT NULL,
    timestamp       TEXT NOT NULL,      -- RFC 3339 UTC
    event_json      TEXT NOT NULL,      -- complete Event as JSON
    received_at     TEXT NOT NULL       -- when this Node received/stored the Event
);

CREATE INDEX idx_events_room_id ON events(room_id);
CREATE INDEX idx_events_event_type ON events(event_type);

CREATE TABLE dag_edges (
    event_id        TEXT NOT NULL,
    prev_event_id   TEXT NOT NULL,
    PRIMARY KEY (event_id, prev_event_id),
    FOREIGN KEY (event_id) REFERENCES events(event_id),
    FOREIGN KEY (prev_event_id) REFERENCES events(event_id)
);
```

The `dag_edges` table stores the `prev_events` relationships explicitly. This allows efficient tip computation (find `event_id` values that do not appear as `prev_event_id` for any other Event in the same Room) and predecessor lookup without parsing the `event_json` blob.

#### 4.12.2 Append-Only Invariant

The Event store enforces the append-only invariant structurally: there is no UPDATE or DELETE operation on the `events` table. Once an Event is written, it is permanent.

The database file itself should be protected at the OS level — the application folder should not be world-writable. The Node does not implement its own access control layer on top of SQLite for Phase 1.

#### 4.12.3 Pending Event Buffer

Events that fail validation step 9 (unknown predecessor) are held in a per-Room in-memory pending buffer. The Node sends `transport.sync_request` to its peers for the missing predecessor IDs. If the predecessors arrive within a timeout window (30 seconds, work definition), the pending Event is re-submitted to the validation pipeline. If the timeout expires, the Event is discarded and logged.

The pending buffer is not persisted to disk — if the Node restarts, pending Events are lost. This is acceptable for Phase 1. A reconnecting peer will re-send Events via `transport.sync_request`.

---

### 4.13 Auth Module — Tier 1 Implementation

The Tier 1 Auth Module reference implementation is a separate binary (`xgen-auth-module`) that runs alongside a Node. It is not part of the Node itself — it is an independent service that the Node operator trusts by registering its public key.

#### 4.13.1 Configuration

```toml
# auth_module_config.toml

[module]
name = "XGen Community Verifier"
keypair_path = "./auth_module_keypair.enc"
verification_state = "D"   # A, B, C, or D — see 3.8.3

[network]
bind_address = "127.0.0.1"
bind_port = 9090

[verification]
email_provider = "smtp"    # or "sendgrid", "mailgun"
phone_provider = "twilio"  # or "vonage"
code_expiry_seconds = 600  # 10 minutes

[storage]
db_path = "./auth_module.db"
```

#### 4.13.2 Verification Flow

The Tier 1 verification flow (3.8.3) operates as an HTTP service (the `endpoint` declared in the Auth Module public record). For Phase 1, a simple HTTP POST/GET interface is sufficient — the wire format for `auth.verify_request` and `auth.verify_confirm` is JSON over HTTP, not WebSocket.

The verification state machine per Identity:

```
  IDLE
   │  auth.verify_request received
   ↓
  PENDING_VERIFICATION
   │  send email/phone code(s)
   │  auth.verify_confirm received with correct code(s)
   ↓
  VERIFIED
   │  issue Trust Assertion
   ↓
  ASSERTION_ISSUED
```

The Auth Module stores verification state in a SQLite database. Each Identity has one row tracking its current verification state, the issued codes, the code expiry timestamp, and the issued Trust Assertion (if any).

#### 4.13.3 Trust Assertion Issuance

On successful verification:
1. Construct the Trust Assertion struct (3.8.4) with the verified claims
2. Compute the canonical form and sign with the Auth Module's Ed25519 private key
3. Return the signed Trust Assertion to the client in the `auth.verify_request` response
4. Store a record of the issuance for later validity query responses

The Trust Assertion is a JSON object returned directly in the HTTP response body. The client stores it locally and presents it to the Node at registration.

---

### 4.14 Local Node Mode

Local Node mode is the primary development and testing environment for Phase 1. A complete two-Node, two-Identity smoke test can be run entirely on localhost with no external dependencies.

#### 4.14.1 Running Two Nodes Locally

Two Node instances run on different ports on the same machine:

```
Node A: bind_address = "127.0.0.1", bind_port = 8080
Node B: bind_address = "127.0.0.1", bind_port = 8081
```

Each Node has its own application folder with its own keypair, its own `node_config.toml` (with `local_node = true`), and its own database files. They are independent Node instances that happen to run on the same physical machine.

#### 4.14.2 Running the Reference Client

The reference client (`xgen-client`) connects to a Node by endpoint URI:

```
xgen-client --node ws://127.0.0.1:8080/xgen register --display-name "Alice"
xgen-client --node ws://127.0.0.1:8080/xgen create-space --name "Test Space"
xgen-client --node ws://127.0.0.1:8080/xgen send --space <space_id> --room <room_id> --text "Hello"
```

The client stores its keypair at `client_config.toml`'s `keypair_path` and its Identity state (registered Node, Space memberships) in a local `client_state.json`.

#### 4.14.3 Auth Module Bypass Verification

In Local Node mode, the Node MUST log a clear startup message indicating that Auth Module verification is bypassed. This prevents accidental operation in bypass mode in a production environment. The message should include the Node ID and endpoint so the operator can confirm they are running the intended instance.

---

### 4.15 Smoke Test Execution

The smoke test is the Phase 1 definition of done. It executes the complete 17-step sequence from 3.7.11. It can be run manually using the reference client CLI or automatically using the `smoke_test` subcommand.

#### 4.15.1 Manual Execution

The manual smoke test is a sequence of CLI commands against two running Node instances. Each command should produce the expected output before proceeding to the next.

```bash
# Terminal 1 — start Node A
cd xgen-node-a && ./xgen-node
# Expected: "Node A started. ID: xgen://pubkey/ed25519:AAA..."
# Expected: "Local Node mode active. Auth Module bypass enabled."

# Terminal 2 — start Node B  
cd xgen-node-b && ./xgen-node
# Expected: "Node B started. ID: xgen://pubkey/ed25519:BBB..."
# Expected: "Local Node mode active. Auth Module bypass enabled."

# Terminal 3 — Alice (connected to Node A)
xgen-client --node ws://127.0.0.1:8080/xgen register --display-name "Alice"
# Expected: "Registered. Identity ID: xgen://pubkey/ed25519:CCC..."

xgen-client --node ws://127.0.0.1:8080/xgen create-space --name "Smoke Test"
# Expected: "Space created. ID: xgen://hash/sha256:..."

xgen-client --node ws://127.0.0.1:8080/xgen create-room --space <space_id> --name "general"
# Expected: "Room created. ID: xgen://hash/sha256:..."

xgen-client --node ws://127.0.0.1:8080/xgen invite --space <space_id> --identity <bob_id> --role member
# Expected: "Invite sent."

# Terminal 4 — Bob (connected to Node B)
xgen-client --node ws://127.0.0.1:8081/xgen register --display-name "Bob"
# Expected: "Registered. Identity ID: xgen://pubkey/ed25519:DDD..."

xgen-client --node ws://127.0.0.1:8081/xgen join --space <space_id>
# Expected: "Joined space."

xgen-client --node ws://127.0.0.1:8081/xgen join --space <space_id> --room <room_id>
# Expected: "Joined room."

# Conversation
xgen-client --node ws://127.0.0.1:8080/xgen send --space <space_id> --room <room_id> --text "Hello Bob"
# Expected: "Message sent. Event ID: xgen://hash/sha256:..."

xgen-client --node ws://127.0.0.1:8081/xgen send --space <space_id> --room <room_id> --text "Hello Alice"
# Expected: "Message sent. Event ID: xgen://hash/sha256:..."

# Verification
xgen-client --node ws://127.0.0.1:8080/xgen history --space <space_id> --room <room_id>
# Expected: both messages in order

xgen-client --node ws://127.0.0.1:8081/xgen history --space <space_id> --room <room_id>
# Expected: both messages in order — same Event IDs on both Nodes
```

#### 4.15.2 Automated Smoke Test

The `xgen-client smoke-test` subcommand automates the full sequence. It takes the endpoints of Node A and Node B as arguments, runs all 17 steps programmatically, and reports pass/fail for each step.

```bash
xgen-client smoke-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen
```

Expected output:
```
XGen Phase 1 Smoke Test
=======================
[ ✅ ] Step  1 — Node A keypair generated
[ ✅ ] Step  2 — Node B keypair generated
[ ✅ ] Step  3 — Alice registered on Node A
[ ✅ ] Step  4 — Bob registered on Node B
[ ✅ ] Step  5 — Space created
[ ✅ ] Step  6 — Room created
[ ✅ ] Step  7 — Alice invited Bob
[ ✅ ] Step  8 — Federation handshake complete
[ ✅ ] Step  9 — Node B sent space.join_request
[ ✅ ] Step 10 — state.federation_add recorded
[ ✅ ] Step 11 — Space history synced to Node B
[ ✅ ] Step 12 — Bob joined Space
[ ✅ ] Step 13 — Bob joined Room
[ ✅ ] Step 14 — Alice's message delivered to Node B
[ ✅ ] Step 15 — Bob's message delivered to Node A
[ ✅ ] Step 16 — Both Nodes have both Events
[ ✅ ] Step 17 — Event IDs match across Nodes

Phase 1 complete. ✅
```

Step 17 is the critical verification step: it confirms that the same `event_id` hash appears in both Nodes' Event stores for each message. If the canonical form implementation is correct and signatures verify on both sides, Step 17 passes. If there is any discrepancy in canonical form serialisation between the two Node instances, Step 17 will fail with mismatched Event IDs — this is the diagnostic signal that the canonical form implementation has a bug.

---

### 4.16 CLI Reference

Both reference binaries expose a command-line interface built with `clap` (derive API). Help text is generated automatically from doc comments on command structs and field definitions. The canonical source of all argument descriptions and examples is this section — the Rust doc comments in source code MUST match exactly (D-028).

All commands support `--help` / `-h` at the top level and on every subcommand.

---

#### 4.16.1 `xgen-node`

**Top-level usage**

```
xgen-node [OPTIONS] [COMMAND]
```

When invoked with no subcommand, `xgen-node` starts the Node in foreground mode. It runs until interrupted (CTRL+C) or until it receives a shutdown signal. All protocol activity is logged to stdout unless redirected via `log_path` in config.

**Top-level options**

| Option | Short | Description |
|---|---|---|
| `--config <path>` | `-c` | Path to config file. Default: `./xgen-node_config.toml` |
| `--local` | | Override: start in Local Node mode regardless of config setting |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version and build info |

**Subcommands**

```
xgen-node init
```
Generate a default `xgen-node_config.toml` and a new encrypted keypair (`xgen-node_keypair.enc`) in the current directory, then exit. Does not start the Node. Safe to run multiple times — will not overwrite an existing keypair. Prompts for a passphrase to encrypt the keypair.

Example:
```
> xgen-node init
Generating keypair...
Passphrase: ********
Confirm:    ********
Keypair saved:  ./xgen-node_keypair.enc
Config saved:   ./xgen-node_config.toml
Node ID: xgen://pubkey/ed25519:AAAB...
Run 'xgen-node' to start.
```

---

```
xgen-node status
```
Print the current Node status from `xgen-node_state.json`. The Node must be running for this file to exist and be current. If the file does not exist or is older than 30 seconds, a warning is shown.

Example output:
```
xgen-node status
================
Node ID:      xgen://pubkey/ed25519:AAAB...
Version:      0.10.1 (build 260429-1423)
Uptime:       2h 14m 38s
Mode:         Local Node
Endpoint:     ws://127.0.0.1:8080/xgen
Connections:  2 clients, 1 federated peer
Spaces:       1 hosted
Events:       47 total across all spaces
State file:   updated 3s ago
```

---

```
xgen-node connections
```
List all currently connected clients and federated peers in a table. Reads from `xgen-node_state.json`.

Example output:
```
Connections (2 clients, 1 peer)

CLIENTS
  Identity                                           Display name   Connected     Events sent  Received
  xgen://pubkey/ed25519:CCCC...                      Alice          14m 22s ago   12           8
  xgen://pubkey/ed25519:DDDD...                      Bob            9m 05s ago    6            12

FEDERATED PEERS
  Node ID                                            Endpoint                      State    Since
  xgen://pubkey/ed25519:BBBB...                      ws://127.0.0.1:8081/xgen      ACTIVE   14m 20s ago
```

---

```
xgen-node spaces
```
List all Spaces hosted on this Node with their Rooms and event counts. Reads from `xgen-node_state.json`.

Example output:
```
Spaces (1)

  Space: Smoke Test
  ID:    xgen://hash/sha256:a3f9...
  Rooms: 1   Members: 2   Events: 47

    Room: general
    ID:   xgen://hash/sha256:b2c3...
    Events: 45   Last activity: 2m 11s ago
```

---

```
xgen-node peers
```
List all known federated peer Nodes (active and previously connected). Reads from `xgen-node_state.json` and `xgen-node_federation.db`.

Example output:
```
Federated Peers (1)

  Node ID:     xgen://pubkey/ed25519:BBBB...
  Endpoint:    ws://127.0.0.1:8081/xgen
  State:       ACTIVE
  Session ID:  xgen://hash/sha256:e5f6...
  Version:     0.1 / json
  Spaces:      Smoke Test
  Connected:   14m 20s ago
  Last seen:   3s ago
```

---

```
xgen-node identity list
```
List all Identities registered on this Node. Reads from `xgen-node_identities.db`.

Example output:
```
Registered Identities (2)

  xgen://pubkey/ed25519:CCCC...   Alice    registered 14m ago   1 device
  xgen://pubkey/ed25519:DDDD...   Bob      registered  9m ago   1 device
```

---

```
xgen-node version
```
Print version, build metadata, and Node ID if a keypair exists.

Example output:
```
xgen-node 0.10.1
Build:    260429-1423
Commit:   f873f5e
Node ID:  xgen://pubkey/ed25519:AAAB...
```

---

#### 4.16.2 `xgen-client`

**Top-level usage**

```
xgen-client [OPTIONS] <COMMAND>
```

`xgen-client` does not run persistently. Every invocation executes one command, connects to the Node if required, completes its work, and exits. The Node endpoint is provided via `--node` or read from `xgen-client_config.toml`.

**Top-level options**

| Option | Short | Description |
|---|---|---|
| `--node <endpoint>` | `-n` | Node WebSocket endpoint. Example: `ws://127.0.0.1:8080/xgen`. Overrides config. |
| `--config <path>` | `-c` | Path to config file. Default: `./xgen-client_config.toml` |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version and build info |

**Subcommands**

```
xgen-client init
```
Generate a default `xgen-client_config.toml` and a new encrypted keypair (`xgen-client_keypair.enc`) in the current directory, then exit. Does not connect to any Node. Prompts for a passphrase.

Example:
```
> xgen-client init
Generating keypair...
Passphrase: ********
Confirm:    ********
Keypair saved:    ./xgen-client_keypair.enc
Config saved:     ./xgen-client_config.toml
Identity ID: xgen://pubkey/ed25519:CCCC...
Run 'xgen-client register --name "Your Name"' to register on a Node.
```

---

```
xgen-client whoami
```
Print the local Identity ID and display name from `xgen-client_state.json`. No Node connection required.

Example output:
```
Identity ID:    xgen://pubkey/ed25519:CCCC...
Display name:   Alice
Registered on:  ws://127.0.0.1:8080/xgen
Spaces joined:  1
```

---

```
xgen-client register --name <display-name>
```
Register this Identity on the Node. Requires `--node`. In Local Node mode, no Trust Assertion is required. On success, saves registration state to `xgen-client_state.json`.

| Argument | Required | Description |
|---|---|---|
| `--name <display-name>` | yes | Display name to register. Max 128 characters. |

Example:
```
> xgen-client --node ws://127.0.0.1:8080/xgen register --name "Alice"
Registered.
Identity ID: xgen://pubkey/ed25519:CCCC...
```

---

```
xgen-client create-space --name <name>
```
Create a new Space on the Node. The caller becomes the Space Owner. Returns the Space ID.

| Argument | Required | Description |
|---|---|---|
| `--name <name>` | yes | Display name for the Space. Max 128 characters. |

Example:
```
> xgen-client --node ws://127.0.0.1:8080/xgen create-space --name "Smoke Test"
Space created.
Space ID: xgen://hash/sha256:a3f9...
```

---

```
xgen-client create-room --space <space-id> --name <name>
```
Create a new Room within a Space. The caller must be the Space Owner or Admin.

| Argument | Required | Description |
|---|---|---|
| `--space <space-id>` | yes | Space ID (`xgen://hash/sha256:...`) |
| `--name <name>` | yes | Display name for the Room. Max 128 characters. |

Example:
```
> xgen-client --node ws://127.0.0.1:8080/xgen create-room \
    --space xgen://hash/sha256:a3f9... \
    --name "general"
Room created.
Room ID: xgen://hash/sha256:b2c3...
```

---

```
xgen-client invite --space <space-id> --identity <identity-id> --role <role>
```
Invite an Identity to a Space. The caller must have invite permission for the target role (see spec 3.7.8).

| Argument | Required | Description |
|---|---|---|
| `--space <space-id>` | yes | Space ID |
| `--identity <identity-id>` | yes | Identity ID to invite (`xgen://pubkey/ed25519:...`) |
| `--role <role>` | yes | Role to assign on join: `owner`, `admin`, `moderator`, `member` |

Example:
```
> xgen-client --node ws://127.0.0.1:8080/xgen invite \
    --space xgen://hash/sha256:a3f9... \
    --identity xgen://pubkey/ed25519:DDDD... \
    --role member
Invite sent.
```

---

```
xgen-client join --space <space-id> [--room <room-id>]
```
Join a Space (if `--room` is omitted) or a specific Room within a Space. Requires a prior invite for Space joins. Room joins require Space membership.

| Argument | Required | Description |
|---|---|---|
| `--space <space-id>` | yes | Space ID |
| `--room <room-id>` | no | Room ID. If omitted, joins the Space itself. |

Examples:
```
> xgen-client --node ws://127.0.0.1:8081/xgen join \
    --space xgen://hash/sha256:a3f9...
Joined space.

> xgen-client --node ws://127.0.0.1:8081/xgen join \
    --space xgen://hash/sha256:a3f9... \
    --room xgen://hash/sha256:b2c3...
Joined room.
```

---

```
xgen-client send --space <space-id> --room <room-id> --text <text>
```
Send a `message.text` Event to a Room. The caller must be a member of both the Space and the Room.

| Argument | Required | Description |
|---|---|---|
| `--space <space-id>` | yes | Space ID |
| `--room <room-id>` | yes | Room ID |
| `--text <text>` | yes | Message text. Quoted string. Max length subject to Space size limit. |

Example:
```
> xgen-client --node ws://127.0.0.1:8080/xgen send \
    --space xgen://hash/sha256:a3f9... \
    --room xgen://hash/sha256:b2c3... \
    --text "Hello Bob"
Message sent.
Event ID: xgen://hash/sha256:c3d4...
```

---

```
xgen-client history --space <space-id> --room <room-id> [--limit <n>]
```
Fetch and display the message history for a Room in causal (DAG) order. The caller must be a member of the Space and Room.

| Argument | Required | Description |
|---|---|---|
| `--space <space-id>` | yes | Space ID |
| `--room <room-id>` | yes | Room ID |
| `--limit <n>` | no | Maximum number of messages to display. Default: 50. |

Example output:
```
Room: general  (2 messages)

  2026-04-29T14:10:22.000Z  Alice
  Hello Bob
  Event: xgen://hash/sha256:c3d4...

  2026-04-29T14:10:45.000Z  Bob
  Hello Alice
  Event: xgen://hash/sha256:d4e5...
```

---

```
xgen-client spaces
```
List Spaces and Rooms known to this client from `xgen-client_state.json`. No Node connection required.

Example output:
```
Known Spaces (1)

  Space: Smoke Test
  ID:    xgen://hash/sha256:a3f9...
  Node:  ws://127.0.0.1:8080/xgen
  Role:  owner

    Room: general
    ID:   xgen://hash/sha256:b2c3...
    Joined: yes
```

---

```
xgen-client rooms <space-id>
```
List all Rooms in the specified Space that this Identity is a member of.

| Argument | Required | Description |
|---|---|---|
| `<space-id>` | yes | Space ID (`xgen://hash/sha256:...`) |

Example:
```
xgen-client rooms xgen://hash/sha256:a3f9b2c1...
```

---

```
xgen-client members <space-id>
```
List all Identity IDs and display names currently in the specified Space.

In XGen, the protocol-level term for a user is **Identity**. The human-facing term at Space level is **member**. The `members` command lists all Identities that have a current `membership.join` state in the Space and have not subsequently `membership.leave`d, been `membership.kick`ed, or been `membership.ban`ned.

| Argument | Required | Description |
|---|---|---|
| `<space-id>` | yes | Space ID (`xgen://hash/sha256:...`) |

Example:
```
xgen-client members xgen://hash/sha256:a3f9b2c1...
```

---

```
xgen-client status
```
Print the local client status from `xgen-client_state.json`. No Node connection required.

Example output:
```
xgen-client status
==================
Identity ID:   xgen://pubkey/ed25519:CCCC...
Display name:  Alice
Version:       0.10.1 (build 260429-1423)
Home node:     ws://127.0.0.1:8080/xgen
Spaces joined: 1
State file:    updated 8s ago
```

---

```
xgen-client version
```
Print version and build metadata.

Example output:
```
xgen-client 0.10.1
Build:   260429-1423
Commit:  f873f5e
```

---

```
xgen-client smoke-test --node-a <endpoint> --node-b <endpoint>
```
Run the complete Phase 1 smoke test (spec 3.7.11) automatically against two running Node instances. Creates two temporary Identities, a Space, a Room, federates the Nodes, exchanges messages in both directions, and verifies Event ID consistency across both Nodes.

| Argument | Required | Description |
|---|---|---|
| `--node-a <endpoint>` | yes | Endpoint of Node A. Example: `ws://127.0.0.1:8080/xgen` |
| `--node-b <endpoint>` | yes | Endpoint of Node B. Example: `ws://127.0.0.1:8081/xgen` |
| `--keep` | no | Do not clean up test Identities and Spaces after the run. Default: clean up. |

Example output:
```
XGen Phase 1 Smoke Test
=======================
Node A: ws://127.0.0.1:8080/xgen
Node B: ws://127.0.0.1:8081/xgen

[ ✅ ] Step  1 — Node A keypair verified
[ ✅ ] Step  2 — Node B keypair verified
[ ✅ ] Step  3 — Alice registered on Node A
[ ✅ ] Step  4 — Bob registered on Node B
[ ✅ ] Step  5 — Space created
[ ✅ ] Step  6 — Room created
[ ✅ ] Step  7 — Alice invited Bob
[ ✅ ] Step  8 — Federation handshake complete
[ ✅ ] Step  9 — Node B sent space.join_request
[ ✅ ] Step 10 — state.federation_add recorded
[ ✅ ] Step 11 — Space history synced to Node B
[ ✅ ] Step 12 — Bob joined Space
[ ✅ ] Step 13 — Bob joined Room
[ ✅ ] Step 14 — Alice's message delivered to Node B
[ ✅ ] Step 15 — Bob's message delivered to Node A
[ ✅ ] Step 16 — Both Nodes have both Events
[ ✅ ] Step 17 — Event IDs match across Nodes

Phase 1 complete. ✅  (elapsed: 1.4s)
```

If any step fails, the output shows the specific failure with the error code and description, and all subsequent steps are skipped:

```
[ ✅ ] Step  1 — Node A keypair verified
[ ✅ ] Step  2 — Node B keypair verified
[ ❌ ] Step  3 — Alice registered on Node A
          Error 3007: identity already registered
          Hint: use --keep on a previous run? Delete xgen-node_identities.db and retry.
[ ⏭ ] Steps 4–17 skipped.
```

---

#### 4.16.3 Short ID Notation

All commands accept full `xgen://hash/sha256:<hexstring>` and `xgen://pubkey/ed25519:<base64url>` URI values. For readability in terminal output, the CLI truncates long URIs to their first 8 hex/base64 characters followed by `...` when displaying them in tables and status output. Full URIs are always used in machine-readable output and in the `history` command Event ID lines.

Example: `xgen://hash/sha256:a3f9b2c1...` displayed as `sha256:a3f9b2c1...` in tables.

#### 4.16.4 Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | General error (see stderr for details) |
| 2 | Configuration error (missing config file, invalid field) |
| 3 | Connection error (Node unreachable, authentication failed) |
| 4 | Protocol error (Node returned an error code) |
| 5 | Smoke test failure (one or more steps failed) |

#### 4.16.5 ANSI Colour Output

The CLI uses ANSI escape codes for coloured output (error messages in red, success
in green, warnings in yellow). Supported terminal environments include Windows
Terminal, PowerShell, and all standard Linux/macOS terminals.

Implementation note (Rust): use the `supports-color` crate for runtime detection.
This crate checks the `TERM` and `COLORTERM` environment variables and calls the
Windows `GetConsoleMode` API with `ENABLE_VIRTUAL_TERMINAL_PROCESSING` where
applicable. If detection returns false, strip escape sequences from output —
never suppress the message text itself, only the colour codes.

---



*To be populated as implementation begins.*

---

### 4.17 Logging

XGen nodes produce two independent and non-interchangeable log types. They are never merged and never share a file.

#### 4.17.1 Debug Log

The debug log is a technical diagnostic output for developers and operators. It records transport events, validation steps, connection lifecycle, and error details.

**Location:** `logs/` subfolder in the Node's working directory (the folder where `xgen-node_config.toml` lives). Created automatically on first run.

**Filename pattern:** one new file per Node startup session:
```
logs/xgen-node_2026-04-29_14-35-22.log
```
Pattern: `logs/xgen-node_YYYY-MM-DD_HH-MM-SS.log` (local time at startup). Files accumulate and are never auto-deleted. The operator may delete old files at any time.

**Log line format:**
```
2026-04-29 14:35:22.401 [INFO ] xgen_node_lib::node::runtime: Node started node_id=xgen://pubkey/ed25519:... endpoint=ws://127.0.0.1:8080/xgen
```
Fields: timestamp (local, millisecond precision), fixed-width level, Rust module path, message, structured key=value pairs.

**Verbosity control:** set `level` in the `[logging]` section of `xgen-node_config.toml`:

```toml
[logging]
level = "info"   # off | error | warn | info | debug | trace
```

| Level | What appears |
|---|---|
| `off` | Nothing |
| `error` | Errors only |
| `warn` | Errors and warnings |
| `info` | Normal operational milestones — recommended default |
| `debug` | Full internal detail — use when diagnosing problems |
| `trace` | Step-by-step internals — very verbose |

The `xgen-client` binary produces an equivalent debug log in its own working directory under `logs/xgen-client_YYYY-MM-DD_HH-MM-SS.log`, controlled by the same `[logging].level` field in `xgen-client_config.toml`.

---

#### 4.17.2 Audit Log

The audit log is a permanent, append-only, machine-readable record of all membership and state-change Events. It exists for compliance and accountability purposes — not for debugging. It cannot be disabled by config and must never be auto-deleted.

**Spec reference:** 3.11.8 Audit Log Requirements.

**Location:** `audit/` subfolder in the Node's working directory. Created automatically on first run.

**Filename pattern:** one file per calendar month:
```
audit/protocol_audit_2026-04.jsonl
audit/protocol_audit_2026-05.jsonl
```

**Format:** JSON Lines — one JSON object per line, UTF-8. Example:
```json
{"ts":"2026-04-29T14:35:31.014Z","event_type":"membership.join","event_id":"xgen://hash/sha256:a3f9...","node_id":"xgen://pubkey/ed25519:CCCC...","identity_id":"xgen://pubkey/ed25519:AAAA...","space_id":"xgen://hash/sha256:b2c3..."}
```

**Events recorded:** all membership and state-change Events — `membership.join/leave/invite/kick/ban`, `state.space_create`, `state.room_create`, `state.federation_add/remove`, `identity.register`, `system.key_rotation`. Full list in 3.11.8.

**Cannot be disabled.** Setting `[logging].level = "off"` suppresses the debug log only. The audit log always runs regardless of config.

**Retention:** audit files MUST NOT be auto-deleted by the Node. Deletion is an operator decision. At Tier 3 and Tier 4, regulatory minimum retention periods apply (see 3.11.8).

**Client:** the client does not produce an audit log. The audit log is a Node responsibility only.

---

#### 4.17.3 The Two Logs Are Independent

The debug log and audit log serve different audiences, have different retention rules, and must never be merged:

| | Debug log | Audit log |
|---|---|---|
| Audience | Developer, operator | Auditor, regulator |
| Controlled by | `[logging].level` in config | Always on |
| Location | `logs/xgen-node_*.log` | `audit/protocol_audit_*.jsonl` |
| Retention | Operator's choice | Never auto-deleted; regulatory minimum at Tier 3/4 |
| Client produces | Yes | No |

---

## Chapter 4 — Known Tradeoffs

**SQLite per Space vs. single database**

One SQLite file per Space is simple and follows Pattern A. It becomes unwieldy if a Node hosts hundreds of Spaces. A single database with a `space_id` partition is a valid Phase 2 option. For Phase 1, one file per Space is correct.

**HTTP for Auth Module interface**

The Auth Module uses HTTP POST/GET for its verification interface rather than WebSocket. This is pragmatic — verification is a short-lived request/response interaction, not a persistent session. A future improvement could unify the transport, but it adds no value for Phase 1.

**No migration tooling**

Phase 1 uses a fixed SQLite schema with no migration tooling. Any schema change requires recreating the database. This is acceptable for Phase 1 development environments where the Event store is not persistent across protocol changes.

---

## Chapter 4 — Handoff to Chapter 5

*To be written when Phase 1 implementation is complete and the smoke test passes cleanly.*

---

## Session Log

### Session 1 — April 2026 (JozefN)
**Covered:** Chapter 4 Phase 1 written in full. Fifteen sections: 4.1–4.15. Technology stack confirmed: Rust, tokio, tokio-tungstenite, ed25519-dalek, sha2, serde_json, sqlx+SQLite, toml, clap, tracing. Multi-SDK strategy documented.

### Session 2 — April 2026 (JozefN)
**Covered:** Section 4.6.1 corrected: AES-256-GCM → ChaCha20-Poly1305 + Argon2id (matching D-002 and actual implementation). Runtime folder layouts updated in 4.3: `xgen-node_*` / `xgen-client_*` file naming convention (D-025), state file added (D-026), client folder layout added. Section 4.8.1 config filename updated to `xgen-node_config.toml`. Section 4.16 CLI Reference added: complete command surface for both binaries including all subcommands, argument tables, expected output examples, short ID notation, and exit codes (D-027, D-028). Section skeleton table updated.