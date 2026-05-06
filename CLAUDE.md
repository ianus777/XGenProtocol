# XGen Protocol — Claude Code Briefing
> For: Claude Code (claude.ai/code)  
> Date: April 2026  
> **Status:** ACTIVE  
> **Last updated:** 2026-05-06  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

---

## ✅ PHASE 1 IS COMPLETE — DO NOT RE-IMPLEMENT

All Phase 1 deliverables are done:

1. **Binary wiring** — both `xgen-node` and `xgen-client` are real runnable processes.
2. **Smoke test** — `xgen-client smoke-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen` runs all 17 steps against real Node processes over real TCP. Verified 2026-04-29. Tag `v0.10.3`.
3. **Documentation gates** — handled by documentation Claude separately. Do not begin Phase 2 implementation until both gates are confirmed complete by JozefN.
4. **Stress test** — `docs/tests/STRESSTEST_ph1.md` is ready for implementation. Add `stress-test` subcommand to `xgen-client` alongside `smoke-test`. Implement after smoke test is confirmed still passing.

**Do not begin Phase 2 implementation until JozefN confirms the documentation gates are complete.**

---

## ✅ DONE — Priority 0: Global Event tracing interface

`docs/tests/LOGGING_debug_ph2.md` implemented (J-027). Definitive implementation instructions (session header/footer, LOCAL actions, EventDirection rename) are in `LOGGING_implementation.md` — implement this before any Phase 2 protocol features. `event_trace` module lives in `xgen-common/src/` (Fix 17 applied, J-029). `Event` and `EventType` also moved to `xgen-common/src/wire.rs` and re-exported from `xgen-node`. Role gate active. Content field never logged. 173/173 tests passing. Smoke test with debug logging confirmed full Event pairing across client and both Nodes (J-029).

---

## 🔧 DONE — Phase 1 debug logging

`docs/tests/LOGGING_debug_ph1.md` is complete and verified (J-025). Datetime-stamped log files, config level switch, subscriber init, operational log calls — all implemented in both binaries. Do not re-implement.

The audit log (`docs/tests/LOGGING_audit_ph2.md`) is **deferred** — implement alongside Tier 2+ Auth Module work only.

---

## ✅ DONE — Documentation fixes (FIXES_ph1.md)

All 17 fixes from `FIXES_ph1.md` have been applied (Fix 14 deferred by project owner). Fix 16 (Node space state replay on restart) and Fix 17 (event_trace relocation) are complete in Rust source. Documentation fixes 1–15 applied to Ch3/Ch4. See `FIXES_ph1.md` for the full record.

---



XGen Protocol is an open, federated, identity-verified communication protocol. Think of what Discord would have been if built as open infrastructure. The core thesis: no single entity should own the communication layer.

This is not a product — it is protocol infrastructure. Phase 1 is a minimal working implementation. Phase 2 is the full protocol. Phase 3+ is everything else.

**The spec is authoritative.** When this file and the spec conflict, the spec wins. When the spec is ambiguous, flag it — do not resolve it silently.

---

## Current State — Where We Are

**Phase 1 is complete. 173 tests passing. CLI complete. Phase 2 is next.**

| Layer | Content | Tests | Tag |
|---|---|---|---|
| 1 | Crypto (Ed25519, SHA-256, base64url, ChaCha20+Argon2id) | 25 | v0.1.1 |
| 2 | Wire format (Event, EventType, framing, validation steps 1–7) | 53 | v0.2.2 |
| 3 | DAG event store (append-only, tips, pending buffer) | 79 | v0.3.2 |
| 4 | WebSocket transport (challenge-response auth, keepalive) | 88 | v0.4.2 |
| 5 | Node identity and announcement | 100 | v0.5.2 |
| 6 | Federation handshake (state machine, registry) | 121 | v0.6.2 |
| 7 | Identity registration (8-step pipeline, registry) | 142 | v0.7.2 |
| 8 | Space and Room protocol (state machine, roles, permissions) | 160 | v0.8.2 |
| 9 | Message exchange (validation steps 8–13, accept_event) | 171 | v0.9.3 |
| 10 | Smoke test — spec 3.7.11, 17-step end-to-end | 173 | v0.10.1 |
| CLI | init, status, connections, spaces, peers, identity list, whoami (D-025–D-028) | 173 | v0.10.2 |
| Binaries | xgen-node WebSocket server + xgen-client network commands + 17-step smoke test over real TCP | 173 | v0.10.3 |

Phase 1 definition of done met: 17-step smoke test passes. Tag `v0.10.1`.
Phase 1 CLI completeness: both binaries have full clap CLI, state file types, and all observability commands. Tag `v0.10.2`.
Phase 1 binary wiring verified: smoke test passes against two real running Node processes over TCP. Tag `v0.10.3`.

---

## Architecture Rules — Non-Negotiable

**1. Library-first.** All protocol logic lives in `lib.rs`. `main.rs` is a thin CLI shell only — argument parsing, startup, shutdown. No business logic in `main.rs`. This is what makes Phase 2 Tauri integration possible without rewriting.

**2. Spec is authoritative.** `docs/xgen_ch3_specification.md` is the source of truth. `IMPLEMENTATION_GUIDE_ph1.md` is the implementation guide. When they conflict, the spec wins.

**3. Verify after every write.** Read back every file after writing it. Silent write failures have caused reconstruction work in past sessions.

**4. DECISIONS.md before advancing.** Every implementation decision beyond spec prescription must be recorded in `DECISIONS.md` before moving to the next layer. Format: title, date, layer, spec reference, decision narrative.

**5. Tests before advancing.** Run `cargo test` and confirm all tests pass before moving to the next layer. Do not skip.

---

## File Placement Rules (D-025 — Updated)

All runtime files are prefixed with the binary name. **`xgen-node_*` for all Node files, `xgen-client_*` for all client files.**

**Tier 1 — System files: mandatory co-location with binary, not configurable**

| File | Binary | Description |
|---|---|---|
| `xgen-node_config.toml` | xgen-node | Node configuration (TOML) |
| `xgen-node_state.json` | xgen-node | Live status snapshot, written every 5s (D-026) |
| `xgen-node_identities.db` | xgen-node | Identity registry (SQLite) |
| `xgen-node_federation.db` | xgen-node | Federation registry (SQLite) |
| `xgen-client_config.toml` | xgen-client | Client configuration (TOML) |
| `xgen-client_state.json` | xgen-client | Identity, known nodes, joined spaces |

**Tier 2 — User-configurable files: default to binary folder, redirectable via config**

| File | Config field | Description |
|---|---|---|
| `xgen-node_keypair.enc` | `keypair_path` | Ed25519 private key — may redirect to HSM or secure share |
| `xgen-client_keypair.enc` | `keypair_path` | Ed25519 private key — may redirect to OS keystore (Phase 2) |
| Log output | `log_path` | May route to system log aggregator |

No file moves silently. Every Tier 2 redirect is explicit in config.

---

## meta_atts Key Namespace Rules (Spec 3.1.3)

`meta_atts` keys use dot-separated namespaces:

- `xgen.*` — **reserved** for protocol use only. Examples: `xgen.client`, `xgen.thread_id`, `xgen.tags`
- Third-party keys MUST use reverse-domain prefix. Examples: `com.example.priority`, `org.myapp.color`
- All lowercase, snake_case segments, dots as separators, no hyphens
- Max key length: 128 characters
- Values are strings. Structured values are JSON-encoded strings, not nested objects.

---

## Transport Pluggability (Spec 3.3.1)

WebSocket over TLS is the mandatory production transport. The protocol also explicitly permits Tor hidden services, I2P, and pluggable transport proxies as alternative stream transports — no protocol changes required. Phase 1 uses `ws://` localhost only. Production uses `wss://`. DPI resistance is a Phase 3 area; no Phase 1 impact.

---

## Key Cryptographic Decisions

- **Keypair encryption at rest:** ChaCha20-Poly1305 + Argon2id KDF. Phase 1 local node uses empty passphrase (file still encrypted for integrity).
- **Event ID derivation:** SHA-256 hash of canonical JSON → `xgen://hash/sha256:<hex>`
- **Signature format:** `ed25519:<base64url-pubkey>:<base64url-sig>` — covers canonical form only, not wire bytes
- **Canonical form:** fixed field order, no whitespace, object keys sorted lexicographically, `event_id` and `signature` excluded
- **DAG root types:** `state.space_create`, `state.room_create`, `state.dm_space_create` require empty `prev_events`. All others require at least one.
- **Cycle detection:** reduces to self-reference check only at insertion time (append-only store invariant)
- **prev_events fanin limit:** 10 (Phase 1)
- **Node announcement TTL:** 90 days
- **Session ID derivation:** `hash_uri(sort([node_a_id, node_b_id]) + timestamp)` — sorted so both sides derive same value

---

## Versioning Scheme

`[state].[layer].[session]` — three components, stored in `Cargo.toml`.

- `state`: 0 while building Phases 1 and 2; 1 when Phase 1 and Phase 2 complete and stable
- `layer`: implementation layer number (1–10)
- `session`: work session in which that layer was completed

Tags are monotonically increasing: `v0.1.1` → `v0.2.2` → … → `v0.10.x`

---

## Phase 2 — What Comes Next

Phase 2 wires the library into a runnable node and client. Read spec sections 3.9–3.16 before starting. Key items from the post-Phase-1 decision log:

| Item | Decision | Reference |
|---|---|---|
| xgen-core crate split | Extract all protocol logic from `xgen-node/src/` into new GPL `xgen-core` crate | D-022 |
| `self` account | Local-only synthetic Identity, accessible from any client | D-021 |
| DPI resistance | Phase 3 investigation only | D-023 |
| Phase 2 spec sections | 3.9–3.16 (state resolution, E2E encryption, higher Auth Tiers, etc.) | Ch3 Phase 2 |
| Slovak translation pass | Single pass after full document completion | Deferred |
| Registry file encryption | Identity and federation registries encrypted at rest | Phase 2 |

---

## Repository Layout

```
docs/
  xgen_ch0_content.md             # table of contents
  xgen_ch1_philosophy.md          # project philosophy and motivation
  xgen_ch2_architecture.md        # architecture design and primitives
  xgen_ch3_specification.md       # AUTHORITATIVE SPEC (Phase 1 sections 3.1–3.8 complete)
  xgen_ch4_implementation.md      # Phase 1 implementation record (complete)
  xgen_ch5_protocol.md            # stub
  xgen_ch6_client_design.md       # Phase 2 Tauri+Svelte client design decisions
  xgen_appendix_a_en.md           # Why XGen must be its own protocol
  xgen_appendix_b_en.md           # Kyberia lineage and acknowledgment
  xgen_appendix_c_en.md           # Mermaid class diagrams
  xgen_appendix_d_en.md           # Node data, privacy, and storage (GDPR reference)
IMPLEMENTATION_GUIDE_ph1.md       # Phase 1 layer-by-layer guide (this file's companion)
DECISIONS.md                      # Implementation decision log (D-000 through D-028)
JOURNAL.md                        # Contemporaneous development journal (IP record)
CLAUDE.md                         # This file
LICENSE                           # BSL 1.1
```

Source crates:
```
xgen-common/    # shared types (no runtime, no I/O)
xgen-node/      # protocol node — lib.rs has all logic, main.rs is thin CLI
xgen-client/    # CLI test client — same library-first structure
```

Build target directory is outside Google Drive to avoid file locking:
```
C:/cargo-targets/XGenProtocol
```

---

## License Header

Every source file MUST carry this exact header:

```rust
// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.
```

Not PolyForm. Not MIT. Not any other license. BSL 1.1 exactly as above.

---

## Build Commands

```sh
cargo build                              # debug build
cargo build --release                    # release build
cargo test                               # run all tests
cargo test smoke                         # run smoke test only
cargo test --package xgen-common         # test one crate
```

Build output goes to `C:/cargo-targets/XGenProtocol` (set via `CARGO_TARGET_DIR` in `build.sh`). Binaries are copied to `bin/` in the project folder by `build.sh`.

---

*Read `DECISIONS.md` (D-000 through D-028) before making any decision that isn't explicitly covered by the spec. If you're unsure whether something needs a DECISIONS.md entry, it does.*
