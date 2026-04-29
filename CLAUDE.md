# XGen Protocol — Claude Code Briefing
> For: Claude Code (claude.ai/code)  
> Date: April 2026  
> Author: JozefN  
> Status: Current — read this before touching any file  

---

## What This Project Is

XGen Protocol is an open, federated, identity-verified communication protocol. Think of what Discord would have been if built as open infrastructure. The core thesis: no single entity should own the communication layer.

This is not a product — it is protocol infrastructure. Phase 1 is a minimal working implementation. Phase 2 is the full protocol. Phase 3+ is everything else.

**The spec is authoritative.** When this file and the spec conflict, the spec wins. When the spec is ambiguous, flag it — do not resolve it silently.

---

## Current State — Where We Are

**Phase 1 is complete. 173 tests passing. Phase 2 is next.**

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

Phase 1 definition of done met: 17-step smoke test passes. Tag `v0.10.1`.

---

## Architecture Rules — Non-Negotiable

**1. Library-first.** All protocol logic lives in `lib.rs`. `main.rs` is a thin CLI shell only — argument parsing, startup, shutdown. No business logic in `main.rs`. This is what makes Phase 2 Tauri integration possible without rewriting.

**2. Spec is authoritative.** `docs/xgen_ch3_specification.md` is the source of truth. `IMPLEMENTATION_GUIDE_ph1.md` is the implementation guide. When they conflict, the spec wins.

**3. Verify after every write.** Read back every file after writing it. Silent write failures have caused reconstruction work in past sessions.

**4. DECISIONS.md before advancing.** Every implementation decision beyond spec prescription must be recorded in `DECISIONS.md` before moving to the next layer. Format: title, date, layer, spec reference, decision narrative.

**5. Tests before advancing.** Run `cargo test` and confirm all tests pass before moving to the next layer. Do not skip.

---

## File Placement Rules (Updated)

XGen uses a two-tier file placement model. **This changed from the original Pattern A spec — read carefully.**

**Tier 1 — System files: mandatory co-location with binary, not configurable**

| File | Binary | Description |
|---|---|---|
| `node_config.json` | xgen-node | Node configuration |
| `auth_modules.json` | xgen-node | Trusted Auth Module registry |
| `federation_registry.json` | xgen-node | Federation relationships |
| `identity_registry.json` | xgen-node | Identity records |
| `node_announcement.json` | xgen-node | Signed node announcement |

**Tier 2 — User-configurable files: default to binary folder, redirectable via config**

| File | Config field | Description |
|---|---|---|
| Node keypair | `keypair_path` | Ed25519 private key — may go to HSM or secure share |
| TLS certificate | `tls_cert_path` | May use system cert manager |
| Log output | `log_path` | May route to system log aggregator |
| Client keypair | `keypair_path` | May redirect to OS keystore (Phase 2) |
| UI settings | `ui_settings_path` | Phase 2 — client preferences |

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
DECISIONS.md                      # Implementation decision log (D-000 through D-023)
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

*Read `DECISIONS.md` (D-000 through D-024) before making any decision that isn't explicitly covered by the spec. If you're unsure whether something needs a DECISIONS.md entry, it does.*
