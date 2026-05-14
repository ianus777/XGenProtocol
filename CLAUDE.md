# XGen Protocol — Claude Code Briefing
> For: Claude Code (claude.ai/code)  
> Date: April 2026  
> **Status:** ACTIVE  
> **Last updated:** 2026-05-14 (J-055)  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

---

## 🔴 CURRENT TASK — Phase 2 protocol implementation

**Status: COMPLETED — all layers 11–19 done; xgen-core cleanup done (J-055); next: integration testing + data structure appendix**

The xgen-core crate split (prerequisite) is **COMPLETE** (D-044, J-045). Layer 11 (Wire Format Phase 2 Extensions) is **COMPLETE** (D-045, J-046, 202 tests). Layer 12 (State Resolution Algorithm) is **COMPLETE** (D-046, J-047, 226 tests). Layer 13 (Pending Event Timeout) is **COMPLETE** (D-047, J-048, 229 tests). Layer 14 (DM Space Promotion) is **COMPLETE** (D-048, J-049, 237 tests). Layer 15 (Identity Replication) is **COMPLETE** (D-049, J-050, 246 tests). Layer 16 (Space Migration Protocol) is **COMPLETE** (D-050, J-051, 263 tests). Layer 17 (Bootstrap Node and Node Reputation) is **COMPLETE** (D-051, J-052, 275 tests). Layer 18 (End-to-End Encryption) is **COMPLETE** (D-052, J-053, 290 tests). Layer 19 (Auth Module Tier 2–4 Interfaces) is **COMPLETE** (D-053, J-054, 300 tests). **All Phase 2 protocol layers (11–19) are COMPLETE.** Next: Phase 2 integration testing + data structure appendix.

**Priority order:**
1. ~~xgen-core crate split~~ — **DONE** (D-044, J-045)
2. ~~Phase 2 protocol implementation — layers 11–19 per `IMPLEMENTATION_GUIDE_ph2.md`~~ — **DONE** (D-045–D-053, J-046–J-054)
3. New appendix: all object/data structures — after Phase 2 implementation stable
4. UI work — fully postponed until after Phase 2 implementation complete

---

## ✅ DONE — Phase 2 Track 1 infrastructure (Sessions 14–18)

All of the following are COMPLETED. Do not re-implement.

| Instruction file | What it covers | Journal |
|---|---|---|
| `CLIENT_CORE_UI_ph2.md` | Tauri scaffold, 11 lifecycle states, state indicator wired, `--instance` flag | J-034, J-038–J-040 |
| `NODE_CORE_UI_ph2.md` | Node Tauri scaffold, systray, 7 lifecycle states + degraded stacking, `--service` flag | J-040 |
| `FIXES_core_ui_ph2.md` | Four UI bugs fixed (startup sequence, state event, systray tooltip, window show/hide) | J-041 |
| `FIXES_sec_01_ph2.md` | `--instance` label path traversal fix — `validate_instance_label` in both binaries | J-042 |
| `BATCH_FLAG_ph2.md` | `--batch` flag, named pipe IPC (D-043), `.xgb` format, 8 batch commands, clap dispatch | J-043–J-044 |
| `XGEN_CORE_SPLIT_ph2.md` | xgen-core crate split — extract protocol logic into GPL library (D-022, D-044) | J-045 |

**Batch command set** (available in `.xgb` files and pipe dispatch):
`whoami`, `status`, `register`, `create-space`, `create-room`, `invite`, `join`, `send`
See `docs/xgen_appendix_f_en.md` §F.8 for full reference.

**Key decisions added this phase:** D-042 (Tauri event emission), D-043 (named pipe naming convention `\\.\pipe\xgen-{binary}-{label}`)

---

## ✅ PHASE 1 IS COMPLETE — DO NOT RE-IMPLEMENT

All Phase 1 deliverables are done:

1. **Binary wiring** — both `xgen-node` and `xgen-client` are real runnable processes.
2. **Smoke test** — `xgen-client smoke-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen` runs all 17 steps against real Node processes over real TCP. Verified 2026-04-29. Tag `v0.10.3`.
3. **Documentation gates** — complete. All lifecycle, Console, deployment model, and UI architecture decisions are documented. Phase 2 implementation may begin.
4. **Stress test** — complete. `docs/tests/STRESSTEST_ph1_findings.md` status: COMPLETED. All findings resolved, verification run passed (commit `8c9402b`).

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

## ⏸ POSTPONED — UI Phase 2 prep (run 1.5)

UI design work for Phase 2 Track 1 is paused at the element-modelling step (J-033, 2026-05-08).

**Deliverables in `ui/run_1.5/`:**

- `skeleton_audit.md` — initial audit of chat mockups (`ui/backup/fixed_samples/`). Detailed div/span vs semantic-tag conversion conventions. Top-of-file note: framed against the wrong reference; see `comparative_analysis.md` for the corrected take.
- `comparative_analysis.md` — corrected analysis. Miss Design's skeleton (`ui/backup/skeleton/`) already implements ~95% of recommended semantic structure. The gap between her skeleton and the chat mockups lives in CSS reset rigour, visual coding density, and Run 2 evolutions (D-038, D-039, Run 2 Change 1) — not in HTML structure. The current `ui/xgen-mockup-*.html` files are a partial merge attempt that did not fully capture the chat mockups' visual quality.

**Visual merge plan postponed.** A 10-milestone roadmap in `comparative_analysis.md` covers merging the chat mockups' visual treatment onto Miss Design's semantic structure under Ch2 fixed conditions (lifecycle state coverage for all 7 Node + 11 Client states, open-enum fallback rules, slot system intact, Layer 4 boundary, accessibility `:focus-visible`, replaceable skins). Architecture: `tokens.css` always-loaded variables only; `skin-{name}.css` self-contained with own reset; reset coupled to skin so a missing skin degrades to raw HTML. Theme loader behaviour locked in `D-041` (default `skin-dark.css`; fallback chain on skin failure: requested → default → raw HTML).

**Gating step before resume:** confirm and expand the absent-element list in `ui/docs/xgen-ui-design-brainstorm.md` Point 3 (event types in the message stream — member-originated, self mirrored, system/protocol, module-injected) and Point 2 (avatar as first-class object — DOM element with hover context menu). The list is currently marked "to be confirmed" and must be reconciled with Ch3's authoritative event taxonomy. A Run 3 design briefing is drafted from the consolidated list before any visual merge work begins.

Do not start the visual merge or write any skin CSS until the element list is confirmed and the Run 3 briefing exists.

Recorded in `JOURNAL.md` J-033 and `DECISIONS.md` D-041.

**This entire track is postponed until Phase 2 protocol implementation is complete.**

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

## Error Code Convention

Error codes are plain integers on the wire and in exit codes (e.g. `4002`). For human-readable display in logs, UI, and documentation, codes are shown with an `E` prefix and zero-padded to 6 digits (e.g. `E004002`). The `E` prefix is display-only — never transmitted, never used programmatically. `E004002` and `4002` are the same error.

Domain ranges: 1000–1999 transport, 2000–2999 federation, 3000–3999 identity, 4000–4999 state resolution, 5000–5999 E2E encryption, 6000–6999 migration, 7000–7999 bootstrap, 8000–8999 reputation, 9000–9999 DM promotion. Future domains extend naturally: domain 10 = 10000–10999, etc.

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

Phase 2 has two parallel tracks: **UI** and **protocol**. The UI skeleton must be visually validated before any protocol wiring begins.

### Track 1 — UI (prerequisite for two-sided testing)

**Read these documents before writing any Tauri or Svelte code:**
- `docs/xgen_ch6_client_design.md` — full UI architecture, component system, screen inventories, Console spec (§6.9–6.11)
- `docs/xgen_appendix_e_en.md` — **APPLICATION LIFECYCLE STATES** — the authoritative state machine spec for both binaries
- `DECISIONS.md` D-037 — Node deployment model (systray singleton, two personalities, one binary)
- `ui/docs/xgen-ui-chat-briefing.md` — all design decisions: color, chrome, Console, first-run flow, tier glyphs

**Library-first rule still applies.** All lifecycle state machine logic lives in `lib.rs`. Tauri `main.rs` is a thin shell. Svelte frontend calls Tauri commands only — no protocol logic in Svelte.

**UI implementation order:**
1. Tauri scaffold — both binaries open a window, custom chrome (Option 2, no native titlebar), app icon + name only
2. Console overlay — `Backquote` scancode (`KeyboardEvent.code = "Backquote"`) toggle, slides from top, semi-transparent, green-on-dark VT220 scheme locked
3. Lifecycle state machine in Rust — states from Appendix E wired to real application behaviour
4. Console status bar — left/right division: `App name · ● STATE` | `DisplayName / @Nick [Tn] · Space › #Room · ~ close`
5. First-run SETUP flow — display name, passphrase, keypair generation (local only, **zero network traffic**)
6. `auto_connect_local` — silent scan of `ws://127.0.0.1:8080/xgen` after `INITIALISING`, non-blocking, 2s timeout, no error if nothing found
7. Skeleton screens — Space list, Room view, Node dashboard (from design Claude's HTML files in `ui/`)
8. `--batch` flag — both binaries accept `--batch <file.xgb>`, one command per line, sequential execution, no UI required

**Node deployment model (D-037):**
- Normal launch → systray icon + on-demand admin window (detachable — closing window does NOT stop Node)
- `--service` flag → headless, no systray, no window
- Systray icon: grey animated (INITIALISING), green (READY), amber (any DEGRADED_*), blue (MAINTENANCE), grey (CLOSING)

### Track 2 — Protocol (ACTIVE — current priority)

Ch3 spec sections 3.9–3.16 are complete. `IMPLEMENTATION_GUIDE_ph2.md` is written. Implementation begins with the xgen-core crate split. Key items:

| Item | Decision | Reference |
|---|---|---|
| xgen-core crate split | Extract all protocol logic from `xgen-node/src/` into new GPL `xgen-core` crate | D-022, D-044 ✅ DONE |
| `self` account | Local-only synthetic Identity, accessible from any client | D-021 |
| DPI resistance | Phase 3 investigation only | D-023 |
| Phase 2 spec sections | 3.9–3.16 (state resolution, E2E encryption, higher Auth Tiers, etc.) | Ch3 Phase 2 |
| Slovak translation pass | Single pass after full document completion | Deferred |
| Registry file encryption | Identity and federation registries encrypted at rest | Phase 2 |
| Console IPC protocol | Named pipe / local socket for AI agent and batch operation | Ch6 §6.9 — Phase 2 design question |

---

## Repository Layout

```
docs/
  xgen_ch0_content.md             # table of contents
  xgen_ch1_philosophy.md          # philosophy, motivation, Human and Agent Operation
  xgen_ch2_architecture.md        # architecture, primitives, deployment model (D-037)
  xgen_ch3_specification.md       # AUTHORITATIVE SPEC (§3.1–3.8 Phase 1 + §3.9–3.16 Phase 2 — complete)
  xgen_ch4_implementation.md      # Phase 1 complete; Phase 2 scope defined
  xgen_ch5_protocol.md            # stub
  xgen_ch6_client_design.md       # UI architecture, Console §6.9–6.11, IPC protocol §6.9
  xgen_appendix_a_en.md           # Why XGen must be its own protocol
  xgen_appendix_b_en.md           # Kyberia lineage and acknowledgment
  xgen_appendix_c_en.md           # Mermaid class diagrams
  xgen_appendix_d_en.md           # Node data, privacy, and storage (GDPR reference)
  xgen_appendix_e_en.md           # APPLICATION LIFECYCLE STATES — state machine spec for BOTH binaries
  xgen_appendix_f_en.md           # CLI reference and usage examples
  xgen_appendix_g_en.md           # Log line convention
ui/
  client.html                     # Client skeleton (design Claude)
  node.html                       # Node admin skeleton (design Claude)
  console.html                    # Console overlay skeleton (design Claude)
  docs/
    xgan-ui-overview.md           # Design Claude's overview and open questions
    xgan-ui-debug-console-questions.md  # Console Q&A
    xgen-ui-chat-briefing.md      # ALL design decisions answered — read before UI work
IMPLEMENTATION_GUIDE_ph1.md       # Phase 1 layer-by-layer guide — COMPLETED
IMPLEMENTATION_GUIDE_ph2.md       # Phase 2 layer-by-layer guide (layers 11–19) — ACTIVE
DECISIONS.md                      # Implementation decision log (D-000 through D-044)
JOURNAL.md                        # Contemporaneous development journal (IP record)
CLAUDE.md                         # This file
LICENSE                           # BSL 1.1
```

Source crates:
```
xgen-common/    # shared types (no runtime, no I/O) — BSL 1.1
xgen-core/      # all protocol logic — GPL-2.0-or-later (created in Phase 2 crate split)
xgen-node/      # thin Node shell — main.rs + lifecycle, depends on xgen-core — BSL 1.1
xgen-client/    # thin client shell — main.rs + commands, depends on xgen-core — BSL 1.1
```

Build target directory is kept outside the project folder to avoid file locking:
```
C:/cargo-targets/XGenProtocol
```

---

## Document Header Convention

### Core pattern

```
# Title
> **Status**: {}  
> Version: {}  
> Date: {MMM YYYY}  
> **Last updated**: YYYY-MM-DD  
> Language: {}  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  
```

### Specification

- Every `> ...` line requires **two trailing spaces before EOL** (mandatory for correct line rendering)
- `{MMM YYYY}` = month-name + year, e.g. `May 2026`
- **This header MUST be updated on every file edit**

Status values:
- `ACTIVE` — current, act on it
- `PENDING` — written, not yet the current task
- `COMPLETED` — done, do not re-execute
- `DEPRECATED` — no longer valid / replaced — replacement named if applicable
- `ARCHIVED` — frozen historical record, do not modify

**When looking for the next task**, scan `docs/tests/` file headers first. The next instruction file to run is the first one with `PENDING` status that is not explicitly deferred.

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
