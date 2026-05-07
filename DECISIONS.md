# XGen Protocol — Implementation Decisions
> **Status:** ACTIVE  
> **Last updated:** 2026-05-06  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

Every decision that goes beyond spec prescription is recorded here before advancing to the next layer.
Format: title, date, layer, spec reference, decision narrative.

---

## D-039 — Pending buffer wiring: NodeRuntime holds PendingBuffer directly

**Date:** 2026-05-06
**Layer:** Message exchange / Federation (Phase 1 bug fix — F-001)
**Spec reference:** Spec 3.2.5 (pending buffer for unknown prev_events)

### Context

The Phase 1 stress test (STRESSTEST_ph1_findings.md) identified finding F-001: during the concurrent message flood, federated events arriving at Node B with unknown `prev_events` were being silently dropped rather than buffered. The stress test report showed PASS at the client level but Node B was applying only ~53% of expected federated messages.

`PendingBuffer` (`dag/pending.rs`) was already fully implemented and tested. `RoomDag` (`dag/mod.rs`) correctly wraps `EventStore + DagGraph + PendingBuffer` and handles out-of-order delivery with cascading drain. However, `NodeRuntime::accept_message` bypassed both: it called `accept_event` directly using the raw `EventStore` and `DagGraph` fields. On `HeldPending`, the error bubbled up to `main.rs`, which logged it as `ERROR` and traced it as `RejectEvent` — dropping the event permanently.

### Decision

Add `pending: HashMap<String, PendingBuffer>` directly to `NodeRuntime` rather than replacing the existing `stores + graphs` fields with `RoomDag` instances.

**Reason for not switching to `RoomDag`:** `RoomDag::insert` only performs DAG-level checks (missing prev_events, structural validation). `accept_message` must run the full 13-step pipeline (steps 8–13: event_id hash, DAG structure, sender identity, space membership, signature, permissions). These steps require `SpaceState` and `IdentityRegistry` which `RoomDag` does not hold. Switching to `RoomDag` would have required either passing those dependencies into `RoomDag` (changing its interface) or duplicating the validation logic. Adding `PendingBuffer` alongside the existing fields is the minimal change that fixes the gap without altering the `RoomDag` interface or adding responsibilities it was not designed for.

### Implementation

- `NodeRuntime` gains `pub pending: HashMap<String, PendingBuffer>`.
- `accept_message`: on `HeldPending(missing)` → calls `pending.add(event, &missing)` and returns `Err(HeldPending)`.
- `accept_message`: on `Ok(())` → calls `drain_pending_messages(space_id, event_id)`.
- `drain_pending_messages`: resolves the buffer using `pending.resolve(resolved_id, store)`, re-runs `accept_event` on each unblocked event, recurses for every newly accepted event.
- `main.rs`: `Err(ExchangeError::HeldPending(_))` arm logs at `DEBUG` ("event buffered — waiting for unknown prev_events") and does not emit a `RejectEvent` trace, since the event is buffered not rejected.

### Verification

Stress test re-run post-fix: 0 ERROR lines on Node B, 0 reject_event traces, 284 apply_event entries (up from 134, now symmetrical with Node A's 280). With resting point after Phase 3, 0 buffered entries (all membership events settled before flood, no out-of-order arrivals at all).

---

## D-038 — Client session header omits `identity_id` and `connected_node`

**Date:** 2026-05-06
**Layer:** Logging — xgen-client
**Spec reference:** docs/xgen_appendix_g_en.md (session header); LOGGING_implementation.md Step 2

### Decision

Appendix G specifies that the `xgen-client` session header includes `identity_id` and `connected_node`. These fields cannot be placed in the header block because log body lines appear before those values are available:

- `"Log file opened"` fires immediately after subscriber init, before any keypair is loaded or connection is made.
- `"Connecting to Node"` fires inside each network command handler, before authentication completes.

The header must precede all body lines (Appendix G, session structure). Deferring the header until auth completes would violate that constraint. Buffering log output until auth completes is not idiomatic with the `tracing` subscriber model.

**Resolution:** the `xgen-client` session header is written immediately after subscriber init with the fields that are available at that moment (`app_type`, `protocol_version`, `build`, `session_id`, `started_at`). The fields `identity_id` and `connected_node` are omitted from the header and are instead emitted as operational body lines at the point where they become known:

- `identity_id` is logged as a body line after keypair load and `client_authenticate()` completes.
- `connected_node` is logged as a body line after the WebSocket connection is established.

This applies to the CLI client only. The future Tauri UI client (Ch6) has a persistent session with a natural startup sequence and will be able to supply both fields in the header at open time.

---

## D-037 — Tier 1 identity: precise definition of persistent accountable identity

**Date:** 2026-05-05
**Layer:** Philosophy / Specification
**Spec reference:** Ch1 Pillar 2 (no anonymity); Ch3 authentication tiers

### Decision

The original "no anonymity" pillar was correct in intent but imprecise in language, creating a risk of misreading Tier 1 as requiring verified real-world identity. This entry locks the precise definition.

**Tier 1 establishes persistent accountable identity, not civil identity.**

The identity anchor at Tier 1 is the keypair. It is permanent and non-respawnable. This is what "no anonymity" means in XGen: not "we know who you really are," but "you cannot disappear and reappear as someone else."

**Tier 1 requirements:**
- A keypair (the identity anchor — permanent, cryptographically bound to the user)
- At least one contact field: email, phone number, or both — self-declared, not verified by the protocol

**Contact data purpose:** operator reach-back channel (ban notices, account recovery). Not an identity proof.

**Optional node behaviour:** a node may implement an email confirmation code flow as a local policy. This is recommended practice but is not a protocol mandate. Phone number SMS verification requires external provider agreements and is outside the protocol's scope entirely.

**What Tier 1 proves:** this is the same cryptographic actor as before. Nothing more, nothing less.

**What Tier 1 does not prove:** that the email address is the user's real address, that the phone number belongs to them, or that they are a specific real-world person.

Tiers 2–4 progressively verify contact data and eventually tie identity to real-world institutional or legal proof.

**Philosophical note:** the anti-abuse guarantee at Tier 1 rests on keypair permanence, not on contact data truthfulness. You cannot ban a keypair's biography — you can ban the keypair. The contact data makes respawning costly enough to matter; it does not make identity transparent.

---

## D-034 — Client log lifecycle deferred to UI application era

**Date:** 2026-04-30  
**Layer:** Phase 2 — client application  
**Spec reference:** docs/tests/LOGGING_debug_ph2.md (future update)

### Decision

The CLI client has no natural session lifecycle — each command invocation connects, acts, and exits. Creating a new log file per command invocation is wasteful and produces meaningless fragmented logs.

The correct log session boundary is the UI application lifecycle: from when the client UI opens to when it closes. This cannot be implemented until a persistent UI client exists.

This item is deferred until the Tauri + Svelte client application (Ch6) is implemented. At that point, `LOGGING_debug_ph2.md` will be updated to specify that the client log file spans the full application session (open to close), not individual command invocations.

**Current behaviour (acceptable for Phase 1 CLI):** one log file per command invocation. Wasteful but functional. Not a bug — a known limitation of CLI architecture.

---

## D-036 — XGen Module Architecture (resolves OQ-01)

**Date:** April 2026  
**Layer:** Architecture — both Node and Client  
**Spec reference:** Ch6 section 6.8 Module Architecture; Ch3 OQ-01 (resolved)

### Decision

XGen modules use **Event subscription + `meta_atts`** as their communication model (Approach C). A module connects to the Node or Client via WebSocket, subscribes to the Event stream, and communicates module-specific payload via the `meta_atts` field on Events. No separate IPC protocol is invented. Modules speak native XGen.

### Module package

A module is distributed as a **package** — one folder containing a manifest file plus any number of handlers, assets, and UI components. Inside one package there may be a single micro-handler or a complex multi-handler system. The packaging, registration, and discovery mechanism is identical regardless of internal complexity. There is no separate concept of "micro-module" vs "full module" at the system level — only packages of varying complexity.

### Module identity mode

Declared in the module manifest as an enum:

- **`system`** — the module has its own keypair and its own identity_id. It signs Events as itself. It is a distinct actor on the network. Used for bots, bridges, aggregators, compliance reporters.
- **`user`** — the module acts on behalf of the authenticated user. It produces Events signed by the user's keypair. Requires explicit user consent at install time. Used for productivity extensions, UI enhancements, workflow automation.

The Node/Client enforces the declared mode at install time and at Event signing time. A `user`-mode module that attempts to sign as a different Identity is rejected.

### Module UI forms

Three UI forms, declared in the manifest. A module may declare one or more:

- **Headless** — no UI representation beyond the module list entry. Runs silently. Used for background services, bridges, reporters.
- **Widget** — a UI component injected into a defined slot in the XGen application shell. Used for inline tools, sidebar panels, message decorators.
- **Window** — a full separate window launched from the module list. Used for substantial self-contained UIs like the Auth Module verification flow.

### Module list — universal registry

Every installed module appears in the module list regardless of its UI form. The module list entry is always the same structure: title, description, version, author, mode badge (`system`/`user`), status indicator (running/stopped/error), and a settings access point. The module list is the single place a user discovers, enables, disables, configures, and removes modules.

### Capability advertisement

When a Node loads a module that adds a new capability, it adds the capability string to its `capabilities` array in its node announcement (3.5.2). Other Nodes and clients that receive the announcement learn about the capability automatically via the open enum mechanism (3.4.3). Unknown capability values are silently ignored by Nodes that do not support them.

### meta_atts as module communication channel

The `meta_atts` field on every Event (defined in 3.2.1) is the designated channel for module-specific payload. A module that needs to attach additional data to an Event uses `meta_atts` rather than extending the core schema. Conventions:

- Keys in `meta_atts` are namespaced by module: `"xgen.module.<module_id>.<key>"`
- Values are strings or JSON-serialisable objects
- Core protocol Nodes that do not recognise a `meta_atts` key silently ignore it (open enum principle)
- `meta_atts` is never used for core protocol data — it is strictly an extension channel

### Injection slots (widget modules)

The XGen application shell defines a set of named injection slots where widget modules may render components. The slot inventory is specified in Ch6 section 6.8.3. A widget module declares which slot(s) it targets in its manifest.

### Manifest format

Specified in Ch6 section 6.8.2.

---

## D-035 — Node data paths derived from working directory — not config-editable

**Date:** 2026-04-30  
**Layer:** Implementation — Node configuration  
**Spec reference:** Ch4 section 4.3 (runtime folder layout)

### Decision

`log_path` and `spaces_dir` MUST NOT be user-editable fields in `xgen-node_config.toml`. Hardcoded absolute paths in an operator-editable config file are a security problem: they reveal data locations, can be tampered with, and create no separation between config (operators read) and data (nobody touches).

The Node derives ALL data paths from its working directory by convention:

```
<working_dir>/
  xgen-node_config.toml     ← config (operators may read)
  xgen-node_keypair.enc     ← keypair (nobody touches)
  xgen-node_state.json      ← runtime state
  xgen-node_identities.db   ← identity registry
  spaces/                   ← Event stores (nobody touches)
  logs/                     ← debug logs
  audit/                    ← audit logs (Phase 2)
```

No path overrides in config. No way to accidentally or maliciously redirect data storage elsewhere. The keypair path remains configurable via `keypair_path` in `[paths]` as a single narrow exception — operators may legitimately store the keypair on a different device or partition for security.

### Implementation requirement for Mr. Code

Remove `log_path` and `spaces_dir` from `[paths]` in `NodeConfig` struct and both test config files. Replace with hardcoded relative path constants in the Rust source:

```rust
const SPACES_DIR: &str = "spaces";
const LOGS_DIR: &str = "logs";
const AUDIT_DIR: &str = "audit";
```

All path construction uses `working_dir.join(SPACES_DIR)` etc. The working directory is wherever the Node binary is run from — documented as a convention, not a config option.

---

## D-033 — Global Event tracing interface — architectural requirement

**Date:** 2026-04-30  
**Layer:** Phase 2 implementation — core architecture  
**Spec reference:** docs/tests/LOGGING_debug_ph2.md  

### Decision

Debug logging must be implemented as a **global Event tracing interface** — a single chokepoint that every inbound and outbound Event passes through automatically. Enumerated manual `tracing::` calls scattered across individual command handlers are rejected as the primary logging mechanism.

### Rationale — why this should have been first

Logging should have been the very first capability implemented, before any protocol logic, so every Event was observable from the first commit. The Phase 1 implementation reversed this order — 173 tests and a full smoke test were written before any logging existed. As a result:
- Some Events ran without any observability
- Log points were added by enumeration — one per command, one per handler — which is fragile and incomplete
- New commands or handlers added in Phase 2 will silently produce no log output unless someone remembers to add a call
- There is no guarantee that a client log entry and a Node log entry can be paired, because pairing depends on both sides having logged the same event_id

This decision corrects the architecture for Phase 2.

### Required architecture

Every Event that enters or leaves the Node or client MUST pass through a single global tracing interface. This interface is not optional and not bypassed by any code path.

**Interface contract:**

```rust
// Every inbound and outbound Event passes through this — no exceptions
pub fn trace_event(
    event: &XgenEvent,
    direction: EventDirection,   // Inbound | Outbound
    session: &SessionContext,    // who is authenticated, their role
)
```

Inside this function:
1. Check session role — if no owner or admin is authenticated, suppress output (see role gate below)
2. Log the Event at `debug` level with structured fields: `event_id`, `event_type`, `direction`, `sender`, `space_id`, `room_id`, `timestamp`
3. Never log `content` field — message content is never written to the debug log at any level

**Role gate:**
- Debug log output is suppressed unless an owner or admin Identity is authenticated in the current session
- Regular members produce no debug log output even if `level = "debug"` is set in config
- The config `level` field still controls the global ceiling — but the role gate is an additional AND condition
- Rationale: prevents sensitive conversations from leaking into log files when regular members are active

**Pairing guarantee:**
- Every Event has an `event_id` (content hash, globally unique)
- Client log: `direction=Outbound event_id=X`
- Node log: `direction=Inbound event_id=X`
- Pairing is trivially possible by matching `event_id` across log files — no coordination needed

### What this means for the current Phase 1 implementation

The Phase 1 debug log infrastructure (datetime-stamped files, `logs/` subfolder, config level switch, subscriber init) is correct and stays. What changes is the log point generation mechanism — from enumerated manual calls to the global interface above. The manual `tracing::info!` calls in individual command handlers become secondary annotations only; the global interface is the primary and mandatory logging path.

### Implementation priority

Implement the global Event tracing interface as the **first task** of Phase 2 implementation, before any Phase 2 protocol features. See `LOGGING_debug_ph2.md` for full instructions.

---

## D-032 — Two distinct log types: debug log and audit log

**Date:** 2026-04-29  
**Layer:** Phase 2 specification — Node implementation and Auth Module interface  
**Spec reference:** 3.11.8 Audit Log Requirements; docs/tests/LOGGING_debug_ph1.md; docs/tests/LOGGING_audit_ph2.md

### Decision

XGen defines two independent and non-interchangeable log types. They are never merged, never share a file, and serve different audiences.

**Debug log** — technical diagnostic output. Operator-controlled verbosity via `[logging].level` in config. Files accumulate in `logs/` subfolder, one per session with datetime suffix. Operator may delete at any time. Serves developer and operator.

**Audit log** — permanent accountability record. Cannot be disabled by config. Append-only JSON Lines, monthly rotation to `audit/protocol_audit_YYYY-MM.jsonl`. MUST NOT be auto-deleted. Serves auditor, compliance officer, regulator.

### Two audit log layers

**Node-level protocol audit log:** records protocol Events with membership and state-change significance. Always present on every Node regardless of Tier. 11 EventTypes covered. Retention is operator/regulatory decision — no protocol minimum at Tier 1/2.

**Auth Module audit log:** records identity verification decisions made by the Auth Module. Lives inside the Auth Module, not the Node. Required at Tier 3 (7-year retention, SOX §802) and Tier 4 (10-year minimum healthcare, mandatory tamper-evident storage, data localisation constraint).

### Rationale

A system where a Tier 4 government or healthcare operator cannot prove who accessed what data and when is not viable for institutional adoption. The audit log is what makes XGen credible to compliance teams, not just to developers. Specifying it at the protocol level — not as an implementation afterthought — ensures third-party implementations are also compliant.

---

## D-031 — End-to-End Encryption: MLS (RFC 9420) selected over Megolm

**Date:** 2026-04-29  
**Layer:** Phase 2 specification  
**Spec reference:** 3.10 End-to-End Encryption (to be written)

### Decision

XGen will use MLS (Messaging Layer Security, RFC 9420) as its end-to-end encryption protocol. Megolm (the Signal-derived group ratchet used by Matrix/Element) was considered and rejected.

### Rationale

MLS is an IETF standard (RFC 9420, published 2023) designed specifically for asynchronous group messaging with dynamic membership. It provides full forward secrecy and post-compromise security for groups of any size, with mathematically clean key tree updates on every join and leave event. Megolm is a proven production protocol but carries well-documented weaknesses in group membership transitions that have caused real security issues in Matrix deployments.

XGen is designed as future infrastructure, not a fast-ship product. The implementation complexity of MLS is the correct tradeoff for a protocol intended to be adopted as open infrastructure by institutions that require cryptographic correctness. Megolm's weaknesses are knowingly inherited — MLS eliminates them by design.

### Implications for 3.10

- Key package format follows RFC 9420
- Group state is represented as an MLS ratchet tree
- Join/leave Events trigger tree updates (Welcome messages for joins, Commit messages for updates)
- The Node is an MLS Delivery Service — it routes MLS handshake messages but cannot decrypt content
- Key material never touches the Node — the Node is structurally excluded from E2E decryption
- Phase 1 Nodes are forward-compatible: they store and route encrypted Event payloads as opaque blobs

---

## D-030 — xgen-node will be packaged as a system service post-stabilisation

**Date:** 2026-04-29  
**Layer:** operational (post-Phase 2)  
**Spec reference:** Ch4 — production deployment section (to be written)

### Decision

Once `xgen-node` is debugged and tuned after Phase 2, it will be packaged as a system service on all supported platforms. This is a production deployment requirement — a Node that requires manual restart after reboot or dies when a terminal session closes is not production-grade infrastructure.

### Platform approach

| Platform | Mechanism | Notes |
|---|---|---|
| Linux | `systemd` unit file | Primary reference deployment. ~15-line unit file, handles restart-on-failure, journald logging, dedicated user account. |
| Windows | NSSM (Non-Sucking Service Manager) | Wraps the binary as a Windows Service without Rust source changes. Pragmatic choice for early production. |
| macOS | `launchd` plist | Standard macOS daemon mechanism. |

### Timing

Not before Phase 2 implementation is complete and the Node has been tested through multiple restart cycles with full state recovery (Fix 16 regression confirmed stable). Service packaging on an unstable process makes bugs harder to diagnose.

### Documentation impact

A new "Production Deployment" section in Ch4 will document the systemd unit file as the primary reference, with NSSM noted for Windows. No changes to Ch3 protocol spec — this is purely operational.

---

## D-000 — Historic First Compile

**Date:** 2026-04-27
**Layer:** 0 (pre-implementation baseline)
**Spec reference:** —

The first successful compile of the XGen Protocol codebase. No protocol logic implemented — both `xgen-node` and `xgen-client` were pure stubs printing a placeholder line. Marked retroactively as version `0.0.0` in semantic terms: state=0 (building), section=0 (no section started), session=0.

The compile itself took seconds. However, the first two attempts froze overnight and for several hours respectively due to Google Drive file locking on build artifacts. Resolved by moving `CARGO_TARGET_DIR` to a local path (`C:/cargo-targets/XGenProtocol`) outside the synced folder.

Tagged on GitHub as `v0.1.0` (build infrastructure baseline). Real versioning — `[state].[section].[session].[build]` — begins with D-001 and the first line of Wire Format code.

---

## D-001 — Versioning Scheme

**Date:** 2026-04-27 (revised 2026-04-28)
**Layer:** 0 (pre-implementation baseline)
**Spec reference:** —

Adopted a three-component version format: `[state].[layer].[session]`

- **state** — 0 while building Phase 1; 1 when Phase 1 complete and stable
- **layer** — implementation layer number (1–10, per IMPLEMENTATION_GUIDE_ph1.md)
- **session** — work session in which that layer was completed

`Cargo.toml` stores this three-part version. Layer numbering follows the implementation order, not the spec section order (spec sections are non-sequential by necessity — e.g., Layer 6 implements spec 3.4). Using layer numbers makes tags monotonically increasing: v0.1.1 → v0.2.2 → … → v0.9.3.

Originally the second component was intended to be the spec section number, which produced non-monotonic tags (e.g., v0.4.2 for Layer 6 before v0.5.2 for Layer 5). Corrected to layer numbers in session 3.

---

## D-002 — Layer 1: Keypair Encryption Scheme

**Date:** 2026-04-27
**Layer:** 1 — Cryptographic Foundation
**Spec reference:** 3.5.1

The spec requires keypairs to be "encrypted at rest" but does not prescribe the encryption algorithm. Chose **ChaCha20-Poly1305** (AEAD) with **Argon2id** key derivation.

- **ChaCha20-Poly1305** — modern, well-audited AEAD cipher. No timing side-channels from table lookups (unlike AES without hardware acceleration). Available in the `chacha20poly1305` crate.
- **Argon2id** — current recommended KDF for password-based key derivation (RFC 9106). Resistant to GPU and side-channel attacks. Parameters for Phase 1: m=64MB, t=3, p=1 — tuned for interactive use.
- **Phase 1 passphrase** — Local Node mode uses an empty string passphrase. The file is still encrypted (the AEAD tag still provides integrity), but without meaningful key stretching. A non-empty passphrase is supported and works correctly. Production deployments must use a strong passphrase.

File format: JSON with `version`, `algorithm`, `kdf`, `salt` (base64url, 32 bytes), `nonce` (base64url, 12 bytes), `ciphertext` (base64url, 48 bytes = 32-byte key + 16-byte AEAD tag).

---

## D-003 — Layer 1: SigningKey Generation Without rand_core Feature

**Date:** 2026-04-27
**Layer:** 1 — Cryptographic Foundation
**Spec reference:** 3.5.1

`ed25519-dalek v2` exposes `SigningKey::generate(&mut rng)` only when the `rand_core` feature flag is enabled. To avoid adding a feature flag, keypair generation uses `OsRng.fill_bytes()` to produce 32 random bytes and constructs the key with `SigningKey::from_bytes()`. This is equivalent — `SigningKey::generate` does the same internally.

---

## D-004 — Layer 2: Event Fields `event_id` and `signature` as `Option<String>`

**Date:** 2026-04-27
**Layer:** 2 — Wire Format
**Spec reference:** 3.2.1, 3.2.3, 3.2.4

The spec defines `event_id` and `signature` as required fields on received Events, but they cannot exist during construction — `event_id` is derived by hashing the canonical form, and `signature` is produced by signing those same bytes. Both fields are therefore `Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]`.

This means an unsigned, unsigned Event serialises without those fields (correct for computing the canonical form), and a signed Event includes them (correct for the wire). The validation pipeline (step 3) enforces presence on received Events; the type system prevents accidental use of an unsigned Event where a signed one is required.

---

## D-005 — Layer 3: Root Event Types Require Empty `prev_events`

**Date:** 2026-04-27
**Layer:** 3 — DAG Event Store
**Spec reference:** 3.2.5

The spec defines `prev_events` DAG rules but does not explicitly enumerate which event types are DAG roots. Decided that `state.space_create`, `state.dm_space_create`, and `state.room_create` are root types (empty `prev_events` required). All other event types must reference at least one predecessor.

Rationale: Space and Room creation events are the structural origins of their respective DAGs — they have no meaningful predecessors within the same namespace. Enforcing empty `prev_events` on these types makes the DAG structure explicit and prevents accidental chaining that would complicate state derivation.

---

## D-006 — Layer 3: Cycle Detection Reduces to Self-Reference Check

**Date:** 2026-04-27
**Layer:** 3 — DAG Event Store
**Spec reference:** 3.2.5

Full cycle detection (verifying no `prev_event` is a descendant of the new Event) is expensive — it requires a graph traversal. For a newly inserted Event this reduces to a single check: does the Event reference itself? A new Event has no descendants yet, so no other cycle is possible at insertion time. Only self-reference (`event_id ∈ prev_events`) needs an explicit check.

This is correct as an invariant because the store is append-only: once an event_id is in the store, no future Event can retroactively become its ancestor.

---

## D-007 — Layer 3: Phase 1 `prev_events` Fanin Limit = 10

**Date:** 2026-04-27
**Layer:** 3 — DAG Event Store
**Spec reference:** 3.2.5

The spec does not specify a hard limit on `prev_events` entries for Phase 1. Chose 10 as a practical ceiling that accommodates realistic concurrent edit scenarios while preventing degenerate inputs. Phase 2 may revisit based on observed network behaviour.

---

## D-008 — Layer 5: Node Announcement TTL = 90 Days

**Date:** 2026-04-27
**Layer:** 5 — Node Identity and Announcement
**Spec reference:** 3.5.6

The spec requires announcements to carry a `valid_until` field but does not prescribe the TTL duration. Chose 90 days for Phase 1. This is long enough that operators on routine schedules (e.g., weekly restarts) never need to worry about expiry, but short enough that a decommissioned node's announcement falls off peer tables within a quarter. Expiry is checked before signature verification to avoid wasting crypto work on stale announcements.

---

## D-009 — Layer 6: Federation `session_id` Derivation

**Date:** 2026-04-27
**Layer:** 6 — Federation Handshake
**Spec reference:** 3.4.4

The spec requires a `session_id` to be agreed during the handshake but does not specify its derivation. Chose: `hash_uri(sort([node_a_id, node_b_id]) + timestamp)` where node IDs are sorted alphabetically before concatenation.

Sorting ensures the same `session_id` is independently computed by both sides regardless of which is initiating and which is receiving. The timestamp is taken from the `federation.hello` message so both sides use the same value.

---

## D-010 — Layer 6: `FederationMessage` Signing Excludes `signature` via Field Order Constants

**Date:** 2026-04-27
**Layer:** 6 — Federation Handshake
**Spec reference:** 3.4.3

Each `FederationMessage` variant carries `signature: Option<String>` with `skip_serializing_if = "Option::is_none"`. The canonical form for signing uses per-variant field order constants that do not include `"signature"`, so the signature field is always absent from the bytes that get signed — whether it is `None` (unsigned) or `Some` (already signed). This avoids the need to temporarily clear the field before computing the canonical form.

---

## D-011 — Layer 7: `MAX_DISPLAY_NAME_LEN` = 128

**Date:** 2026-04-27
**Layer:** 7 — Identity Registration
**Spec reference:** 3.6.5

The spec requires display name validation but does not prescribe a maximum length. Chose 128 characters (Unicode code points). This comfortably accommodates real names, handles emoji and CJK characters, and is simple to communicate. Empty strings and strings containing control characters (codepoints < 0x20) are also rejected.

---

## D-012 — Layer 7: Phase 1 Uses `identity_id` as `device_id`

**Date:** 2026-04-27
**Layer:** 7 — Identity Registration
**Spec reference:** 3.6.6

The spec defines a `devices` array for multi-device support. Phase 1 supports one device per Identity. Rather than omitting the `devices` array entirely, the registration pipeline populates it with a single entry using `identity_id` as the `device_id`. This keeps the wire schema stable for Phase 2 multi-device support without breaking changes.

---

## D-013 — Layer 8: Empty `room_id` Distinguishes Space-Level Events from Room-Level Events

**Date:** 2026-04-27
**Layer:** 8 — Space and Room Protocol
**Spec reference:** 3.7.1, 3.7.3

The spec defines both Space-level and Room-level events sharing the same `Event` envelope. Rather than introducing a separate envelope field, the existing `room_id` field doubles as a discriminator: an empty string means the event targets the Space; a non-empty string means it targets a specific Room. This is consistent with the spec's use of `room_id = ""` on `state.space_create`.

The `apply_event` state machine and the Layer 9 pipeline both branch on `room_id.is_empty()` before dispatching.

---

## D-014 — Layer 8: `apply_join` Branches on `room_id` Before Membership Check

**Date:** 2026-04-27
**Layer:** 8 — Space and Room Protocol
**Spec reference:** 3.7.5

The initial implementation of `apply_join` checked `self.members.contains_key(joiner)` before branching on whether the event was a Space join or a Room join. This caused existing Space members to receive `AlreadyMember` when trying to join a Room (because they were already in `self.members`). Fixed by checking `room_id.is_empty()` first — if non-empty, route to the Room join path; if empty, route to the Space join path with its own duplicate check.

---

## D-015 — Layer 8: `state.space_create` and `state.room_create` Have Empty ID Fields During Construction

**Date:** 2026-04-27
**Layer:** 8 — Space and Room Protocol
**Spec reference:** 3.2.3, 3.7.2

Both `space_id` and `room_id` are derived as `event_id`, which is computed by hashing the canonical event bytes. This creates a circular dependency: the ID fields cannot be known before serialisation, but they must be part of the canonical form. Resolution: event builders set both fields to empty strings during construction. `sign_event` then computes `event_id = hash_uri(canonical_bytes)` — the empty strings are part of the canonical form and the resulting hash becomes the ID. Callers set `space_id` / `room_id` in subsequent events using the derived value.

---

## D-016 — Layer 9: `validate_steps_8_13` Is Read-Only; Callers Control Insertion

**Date:** 2026-04-28
**Layer:** 9 — Message Exchange
**Spec reference:** 3.2.6

Steps 8–13 of the validation pipeline are implemented as a pure read-only function (`validate_steps_8_13`). It does not mutate the `EventStore` or `DagGraph`. Mutation happens only in `accept_event`, which calls the validator and then inserts on success.

This design lets callers inspect the specific failure reason before deciding whether to buffer (step 9 `HeldPending`) or reject (all other errors). It also makes the validator easily testable in isolation without needing mutable state.

Step 10 (DAG structural check) intentionally duplicates the logic from `DagGraph::add_event` rather than extracting a shared helper, because the DAG check requires a read-only view — there is no `DagGraph::validate_only` method and adding one would be scope creep.

---

## D-017 — Layer 9: Test Setup Merges Two DAG Roots via Invite `prev_events`

**Date:** 2026-04-28
**Layer:** 9 — Message Exchange
**Spec reference:** 3.2.5

In test setup, `state.space_create` and `state.room_create` are both DAG root events (empty `prev_events`). Without intervention, they remain as two independent tips indefinitely. The first membership event (`membership.invite`) references both roots as `prev=[space_id, room_id]`, merging the two roots into a single linear chain and leaving exactly one tip. This ensures message events have a single, unambiguous predecessor for `prev_events` in tests.

This is a test-only convention. In production, the protocol does not require roots to be merged — two persistent tips are valid DAG state.

---

## D-018 — meta_atts Key Namespace: Dot Separator, Reverse-Domain Ownership

**Date:** 2026-04-28
**Layer:** 0 (protocol specification)
**Spec reference:** 3.1.3

`meta_atts` keys follow a dot-separated namespace scheme: `<namespace>.<key>`. The `xgen.` prefix is reserved for specification use. Third-party keys MUST use reverse-domain prefixes (e.g. `com.example.my_key`). Key segments use `snake_case`. Max key length 128 characters. Values are strings; structured values must be JSON-encoded as strings rather than embedded as nested objects.

Spec 3.1.3 updated accordingly.

---

## D-019 — Transport Pluggability: WebSocket as Default, Alternative Streams Permitted

**Date:** 2026-04-28
**Layer:** 0 (protocol specification)
**Spec reference:** 3.3.1

WebSocket over TLS is the mandatory production transport. However, the spec explicitly permits operators to substitute any reliable bidirectional stream transport (Tor hidden services, I2P, pluggable transport proxies) without protocol-layer changes. This is noted in spec 3.3.1. DPI-resistance via custom transports is flagged as a Phase 3 investigation area — no Phase 1 or Phase 2 work required.

---

## D-020 — File Placement: Two-Tier Model (System Files vs User-Configurable Files)

**Date:** 2026-04-28
**Layer:** 0 (deployment model)
**Spec reference:** IMPLEMENTATION_GUIDE_ph1.md — Deployment Model

Refined the Pattern A deployment model into an explicit two-tier system. Tier 1 (system files: config, registries, announcements) is mandatory co-location with the binary — not configurable. Tier 2 (keypair, TLS cert, logs, UI settings) defaults to binary folder but can be redirected via explicit config fields. This accommodates HSM-backed keys, OS keystore integration (Phase 2), and system log aggregation without scattering files by default. No file moves silently — every Tier 2 redirect requires an explicit config entry.

---

## D-021 — Self Account (`self`): Local-Only Synthetic Identity, Post-Phase-1

**Date:** 2026-04-28
**Layer:** 0 (deferred post-Phase-1 feature)
**Spec reference:** —

A `self` account (analogous to Skype's own-account or Telegram's Saved Messages) is planned for implementation after the Phase 1 smoke test, during local testing. Design decision: `self` is a local-only synthetic Identity with its own keypair, never registered on any Node and never appearing in federation. It signs local Events but those Events are never broadcast. The `self` account must be accessible from any user client connecting to the Node — it is not device-local. In Phase 2, a "Saved Messages" Space may be implemented as a proper DM Space where both sides of the DM are the user's own keypair.

---

## D-022 — xgen-core Library Split: Deferred to Post-Phase-1

**Date:** 2026-04-28
**Layer:** 0 (architecture — deferred)
**Spec reference:** —

All protocol logic currently lives in `xgen-node/src/`. A planned post-Phase-1 restructure will extract this into a new `xgen-core` crate: GPL-licensed from day one, the primary library for third-party developers. `xgen-node` and `xgen-client` become thin runtime shells wrapping `xgen-core`, retaining their BSL 1.1 wrapper. `xgen-common` remains as shared serde types.

Rationale for deferring: restructuring crates mid-implementation introduces risk right before the Phase 1 finish line. Do the smoke test first, tag Phase 1 complete, then restructure as the first Phase 2 prep task.

---

## D-023 — Traffic Masking / DPI Resistance: Phase 3 Investigation

**Date:** 2026-04-28
**Layer:** 0 (deferred — Phase 3)
**Spec reference:** 3.3.1

Deep-packet-inspection resistance (obfuscating XGen traffic to evade state-level network surveillance) is acknowledged as a legitimate concern. Phase 1 and Phase 2 impact: none — transport pluggability (D-019) already ensures Tor/I2P are usable without protocol changes, which is sufficient for most adversarial environments. Active DPI resistance (disguising XGen traffic as generic HTTPS, pluggable transport integration) is flagged as a Phase 3 area of investigation. Steganographic transport is explicitly out of scope for the core protocol.

---

## D-024 — History Sync: Individual Events, Not Batch Snapshot

**Date:** 2026-04-28
**Layer:** 10 — Smoke Test
**Spec reference:** 3.7.10 (step 8), 3.7.11

The spec requires Node A to "send full Space state and Room Event history to Node B" (step 11 of the smoke test) but does not prescribe a wire format. Two options were considered: (a) individual Events sent one by one, (b) a new batch snapshot message type.

Chose **individual Events**. Rationale: Events are already the atomic protocol unit; every federated Node must be able to validate each Event independently; no new message type is needed; and the individual approach scales correctly to Phase 2 where `transport.sync_request` handles catching up on missed Events after reconnection — it is additive, not a replacement. Batch delivery would require defining a new message type that Phase 2 would likely supersede anyway.

In the smoke test, Node A sends history Events in insertion order over the active connection, followed by the `state.federation_add` Event (which references the pre-history tip as its `prev_events`, and therefore must be received after the history to be correctly linked in Node B's DAG). Connection is closed with `transport.goodbye` to signal end of sync.

---

## D-025 — File Naming Convention: `xgen-node_*` and `xgen-client_*` Prefixes

**Date:** 2026-04-29
**Layer:** 0 (deployment model)
**Spec reference:** IMPLEMENTATION_GUIDE_ph1.md — Deployment Model, Ch4 section 4.3

All runtime files produced or consumed by a binary are prefixed with the binary name: `xgen-node_*` for Node files, `xgen-client_*` for client files.

Rationale: when two Node instances run side by side for testing (NodeA and NodeB folders), every file in the folder is immediately identifiable by name alone — no ambiguity about which binary owns it. Also makes glob patterns unambiguous in scripts (`xgen-node_*.db`, `xgen-client_*.toml`).

Applied to: config (`xgen-node_config.toml`), keypair (`xgen-node_keypair.enc`), state file (`xgen-node_state.json`), databases (`xgen-node_identities.db`, `xgen-node_federation.db`), logs (`xgen-node.log`). Space databases are in a `spaces/` subfolder and are named by space ID hex — the subfolder itself provides the ownership context.

---

## D-026 — Status File (`*_state.json`): Plain JSON, File Permissions as Security Boundary

**Date:** 2026-04-29
**Layer:** 0 (deployment model / CLI design)
**Spec reference:** Ch4 section 4.14

**What the state file contains**

The running Node writes `xgen-node_state.json` to its application folder every 5 seconds. It contains operational metadata: node ID (a public key — already public by protocol design), uptime, connected client identity IDs and display names, federated peer endpoints, hosted space names, and event counts. The client writes `xgen-client_state.json` with: identity ID, display name, known nodes, joined spaces, and last activity timestamps.

**Why it is safe for Phase 1**

No secret material ever enters the state file. The private key lives only in `*_keypair.enc` (encrypted at rest). Signatures are computed in memory and never written to disk in plaintext. The state file contains only information that is already visible to any authenticated participant in the protocol — a connected client can already see who else is in a Space.

**What it leaks and to whom**

The state file leaks topology: who is connected to this Node, which peers it federates with, which Spaces it hosts. This is only a concern if a third party has filesystem read access to the Node's application folder. On a personal development machine: not a concern. On a shared server: the file MUST be protected by OS-level file permissions (Unix: `chmod 600`; Windows: restrict ACL to the operator account). The Node SHOULD set these permissions itself on first write.

**Planned improvements for Phase 2**

Three improvements are planned but explicitly deferred beyond Phase 1:

1. **Redact identity IDs from state file** — replace full `pubkey_uri` values with display names only, or truncated IDs. The full public key of a connected user is already public, but there is no reason to persist it in a file that may be read by monitoring tools.

2. **Separate admin socket** — replace the file-based status mechanism with a Unix domain socket (or named pipe on Windows) that only the operator's process can connect to. Status commands connect to the socket rather than reading a file. This eliminates the file entirely and makes the data available only to processes with the right OS credentials.

3. **Encrypted state file** — encrypt the state file with a key derived from the node keypair passphrase. Only the operator who can unlock the keypair can read the state file. Adds meaningful protection on shared infrastructure without requiring the admin socket approach.

For Phase 1, file permissions are the sufficient and correct mitigation. The planned improvements are recorded here so they are not forgotten when Phase 2 deployment hardening is scoped.

---

## D-027 — CLI Observability Commands: Phase 1 Scope Extension

**Date:** 2026-04-29
**Layer:** 0 (CLI design — Phase 1 scope extension)
**Spec reference:** Ch4 section 4.16

The original Phase 1 definition of done (spec 3.7.11, IMPLEMENTATION_GUIDE_ph1.md Layer 10) specifies the smoke test as the completion criterion. It does not specify a CLI interface beyond what is needed to drive the smoke test.

The following commands are added to Phase 1 scope as a deliberate extension:

**xgen-node:** `status`, `connections`, `spaces`, `peers`, `identity list`
**xgen-client:** `status`, `spaces`, `whoami`

**Rationale:** the smoke test proves the library works in-process. Runnable binaries need to be observable — an operator running two Nodes on localhost needs to see that they are alive, that clients are connected, and that federation is active. Without these commands, the only evidence the system works is log output. These commands transform log output into structured, queryable state.

All observability commands read `xgen-node_state.json` or `xgen-client_state.json` (D-026) — they do not open a new network connection to the running process. This keeps them instant and dependency-free.

**These commands are NOT Phase 2 work.** They are Phase 1 CLI completeness. Phase 2 will replace or supplement them with a GUI dashboard. The state file mechanism (D-026) persists into Phase 2 as the data source for that dashboard.

**What is explicitly NOT in Phase 1 CLI scope:**
- Admin operations that modify Node state (ban identity, force-disconnect peer, etc.) — Phase 2
- Real-time streaming output (live event feed, live connection monitor) — Phase 2
- Auth Module management commands — Phase 2
- Multi-node management (controlling a remote Node) — Phase 2

---

## D-028 — `--help` Built-in: clap Derive Macros, Section 4.16 as Authoritative Source

**Date:** 2026-04-29
**Layer:** 0 (CLI design)
**Spec reference:** Ch4 section 4.16

`clap` with derive macros generates `--help` output automatically from doc comments (`///`) on struct fields and command variants. The help text in the source code is therefore documentation — it must match section 4.16 of Ch4 exactly.

The authoring rule: write section 4.16 first. Copy the argument descriptions and examples from 4.16 into the Rust doc comments. Never write help text in the code first and retrofit it into 4.16 — the spec is the source of truth, the code is the implementation.

Both `xgen-node --help` and `xgen-client --help` (and all subcommand `--help` variants) are generated by clap at compile time from these doc comments. No hand-written help strings.

---

## D-030 — Runtime file placement: GetModuleFileNameW on Windows; data_dir from config path

**Date:** 2026-04-29
**Layer:** 0 (deployment / binary wiring)
**Spec reference:** D-025 (file naming and placement)

### Problem

`xgen-node init` must create its runtime files (keypair, config, identities DB, state file) in a deterministic, predictable location. The natural choice is the directory that contains the running executable. Rust's `std::env::current_exe()` is sufficient on Linux/macOS but has documented edge cases on Windows: Windows Defender, UAC elevation, App Compatibility shims, and some third-party security products can run a process from a shadow copy at a temp path, causing `current_exe()` to return the temp location rather than the original binary location.

Additionally, Phase 1 requires running two Node instances simultaneously for testing (Node A on port 8080, Node B on 8081). When both nodes share the same binary, a single `exe_dir()` would cause Tier-1 file collisions between instances.

### Decision

**1 — `exe_dir()` on Windows uses `GetModuleFileNameW` directly.**

`GetModuleFileNameW(NULL, ...)` (Win32 API, `windows-sys` crate, Windows-only dependency) returns the full path of the module loaded into the calling process. This is the definitive answer to "where does this executable live" — it is immune to CWD, PATH lookup order, symlinks, shell wrappers, and any launcher that might shadow-copy the binary. The function is called with a growing buffer starting at `MAX_PATH` (260) and doubling until the path fits, ensuring correctness for paths beyond `MAX_PATH` (e.g., with `\\?\` extended-length prefix). On non-Windows the standard library's `current_exe()` is used unchanged.

`exe_dir()` panics rather than falling back to `"."` (the CWD). Silent fallback to CWD was the original failure mode — files appeared in a "random" working directory instead of next to the executable. A panic with a clear message is strictly better: it tells the operator exactly what is wrong rather than silently polluting the working directory.

**2 — `data_dir` is derived from the config file path.**

All Tier-1 runtime files are placed in the parent directory of the config file in use:

```
data_dir = config_path.parent()
```

- **Without `--config`:** `config_path` defaults to `exe_dir()/xgen-node_config.toml`, so `data_dir = exe_dir()`. Tier-1 files are co-located with the binary — matches spec D-025.
- **With `--config /path/to/config.toml`:** `data_dir = /path/to/`. This allows multiple Node instances to run from the same binary with fully isolated data directories, by giving each instance its own config file in its own directory.

This rule is simple, explicit, and composable: operators who need multi-instance deployments create one directory per instance and specify `--config`. Operators who run a single instance (the common case) run `xgen-node init` with no flags and get everything in the binary's directory, as expected.

**3 — `xgen-node init` accepts `--passphrase` flag.**

`init` calls `rpassword::prompt_password()` to read the passphrase interactively. This blocks automated setup (CI, scripted deployments, smoke-test harnesses). The `--passphrase` flag provides the passphrase directly without prompting. It is intentionally undocumented in `--help` (hidden flag) — it is not intended for interactive human use, only for scripting. Passing an empty string produces a keypair encrypted with empty passphrase (Phase 1 Local Node mode).

### Files affected

- `xgen-node/src/main.rs` — `exe_dir()`, `main()`, `cmd_init()`, `run_node()`, all observability commands
- `xgen-node/Cargo.toml` — `windows-sys = { version = "0.59", features = ["Win32_System_LibraryLoader"] }` as `[target.'cfg(windows)'.dependencies]`

---

## D-031 — Phase 1 Node configuration reference (xgen-node_config.toml)

**Date:** 2026-04-29
**Layer:** 0 (deployment / reference)
**Spec reference:** Ch4 section 4.8.1

`xgen-node init` generates a default `xgen-node_config.toml` in the data directory. Below is the canonical Phase 1 reference config with all fields documented.

```toml
# XGen Protocol Node — Phase 1 configuration
# Generated by: xgen-node init
# All paths are absolute. Relative paths resolve from the working directory
# at startup, which may differ from the binary location — use absolute paths.

[node]
# WebSocket endpoint this Node listens on.
# Phase 1: ws:// (plain TCP, localhost only).
# Phase 2: wss:// (TLS, public endpoint).
listen = "ws://127.0.0.1:8080/xgen"

# Local Node mode: skip signature verification on incoming events.
# TRUE for Phase 1 development. FALSE for any production or multi-operator setup.
local_mode = true

[paths]
# Ed25519 signing keypair, encrypted at rest (ChaCha20-Poly1305 + Argon2id).
# Phase 1: encrypted with empty passphrase. Phase 2: OS keystore or HSM redirect.
# This is the ONLY mandatory path. The Node will not start without it.
keypair_path = "C:\\XGen\\NodeA\\xgen-node_keypair.enc"

# Optional: redirect log output. Omit to suppress file logging (stderr only).
# log_path = "C:\\XGen\\NodeA\\xgen-node.log"

# Optional: directory for per-space DAG stores. Omit to use in-memory only.
# spaces_dir = "C:\\XGen\\NodeA\\spaces"
```

### Field reference

| Field | Required | Default if omitted | Phase 2 change |
|---|---|---|---|
| `node.listen` | yes | `ws://127.0.0.1:8080/xgen` | Change to `wss://` with real hostname |
| `node.local_mode` | yes | `true` | Set to `false` for production |
| `paths.keypair_path` | yes | — (Node refuses to start) | May redirect to HSM path |
| `paths.log_path` | no | no file logging | Route to syslog aggregator |
| `paths.spaces_dir` | no | in-memory only | Persistent DAG store directory |

### Multi-instance setup (Phase 1 testing)

To run two Nodes on the same machine:

```
E:\XGen\NodeA\xgen-node.exe --config E:\XGen\NodeA\xgen-node_config.toml init
E:\XGen\NodeB\xgen-node.exe --config E:\XGen\NodeB\xgen-node_config.toml init
```

Edit Node B's config to use port 8081. Each instance has its own keypair, identity registry, and state file — no collisions.

---

## D-029 — xgen-client depends on xgen-node lib for Phase 1 binary wiring

**Date:** 2026-04-29
**Layer:** 0 (binary wiring)
**Spec reference:** D-022 (xgen-core crate split, Phase 2)

`xgen-client` depends directly on the `xgen-node` library crate for Phase 1 binary wiring. This gives the client access to the transport layer (`Connection`, `connect_url`), wire types (`Event`, `IdentityMessage`, etc.), federation handshake, identity registration protocol, event building, and crypto — without duplicating ~2 000 lines of code.

The "circular" concern mentioned earlier was conceptual (two binaries sharing a library), not a Cargo constraint. `xgen-client → xgen-node-lib` is a valid, acyclic dependency.

In Phase 2, D-022 (xgen-core crate) extracts the shared protocol logic from `xgen-node` into a new `xgen-core` library. Both `xgen-node` and `xgen-client` will depend on `xgen-core` instead. The direct `xgen-client → xgen-node` dependency is replaced at that point.

---

## D-037 — Node deployment model: systray singleton with detachable admin window

**Date:** 2026-05-07  
**Layer:** 6 (UI / deployment)  
**Spec reference:** Ch6 §6.1, §6.4  

`xgennode.exe` is a singleton process — it starts once and runs permanently. The UI is not the lifecycle host; the process is.

**Desktop deployment (normal launch):**
- Node starts → sits in system tray as a minimal persistent icon
- Systray icon reflects Node health at a glance (green = healthy, amber = warning, red = error)
- Double-click or right-click → Open Dashboard opens the full Tauri admin window
- Closing the admin window does not stop the Node — Node continues running in the tray
- Right-click context menu: Open Dashboard, View Logs, Stop Node

**Server/headless deployment:**
- `--service` flag or OS service wrapper (Windows Service, systemd, launchd)
- No systray, no window — process runs fully headless
- Managed via OS service tooling; logs routed to system aggregator

**One binary, two personalities.** No separate service executable. Launch mode determines behaviour.

**Architectural horizon (not scheduled):** long-term, Node administration via privileged client identity — the operator manages their Node through the XGen client itself as a protocol-native admin surface. This is philosophically aligned with XGen's identity-first model but requires a stable client first and has a bootstrapping challenge. Noted for post-Phase 2 consideration.

---
