# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

XGen Protocol is a foundational, open-source, federated communication protocol — the infrastructure layer beneath chat/community/voice applications, not a product itself. Current state: **Phase 1 implementation in progress** (specification complete, Rust source code not yet started).

License: BSL 1.1 (converts to GPL on community handover). See `LICENSE`.

## Build & Test Commands

Phase 1 implementation uses Rust (stable) with Tokio. Once `Cargo.toml` exists:

```sh
cargo build                   # debug build
cargo build --release         # release build
cargo test                    # run all tests
cargo test <test_name>        # run a single test
cargo test --package xgen-common   # run tests for one crate
```

Target binaries: `xgen-node` and `xgen-client`.

**There is no build configuration yet.** The repo is currently specification and documentation only.

## Repository Layout

```
docs/
  xgen_ch1_philosophy.md          # project philosophy and motivation
  xgen_ch2_architecture.md        # architecture design and primitives
  xgen_ch3_specification.md       # authoritative technical spec (Sections 3.1–3.8 complete)
  xgen_ch6_client_design.md       # Phase 2 Tauri+Svelte client design decisions
IMPLEMENTATION_GUIDE_ph1.md       # Phase 1 roadmap and layer-by-layer guide
DECISIONS.md                      # (to be created) implementation decision log
```

Source crates (`xgen-common/`, `xgen-node/`, `xgen-client/`) do not exist yet.

## Architecture

### Crate Structure (Phase 1)

- **xgen-common** — shared types and serialization (no runtime, no I/O)
- **xgen-node** — protocol node; `lib.rs` contains all logic, `main.rs` is a thin CLI shell
- **xgen-client** — CLI test client; same library-first structure

### Primitive Hierarchy

```
Space → Room → Thread → Event
```

Cross-cutting primitives: **Identity** (server-independent Ed25519 keypair), **Auth Module** (pluggable trust assertion).

### Key Design Rules

- **Specification is authoritative.** When `IMPLEMENTATION_GUIDE_ph1.md` and `xgen_ch3_specification.md` conflict, the spec wins.
- **Library-first.** All protocol logic lives in `lib.rs`. `main.rs` only parses args and calls into the library. This is mandatory from Layer 1.
- **Pattern A deployment.** Each binary is self-contained; all data and config live in the binary's own folder. No system-wide install paths.
- **meta-atts everywhere.** Every protocol object exposes a `meta-atts` field — a namespaced key-value map for extension without breaking the core schema.

## Implementation Order

Phase 1 is implemented in strict layer sequence. Do not skip layers.

| Layer | Focus |
|-------|-------|
| 1 | Cryptographic primitives (base64url, SHA-256, Ed25519, canonical JSON) |
| 2 | Wire format (message types, serde, validation pipeline) |
| 3 | DAG event store (in-memory, graph structure, pending buffer) |
| 4 | WebSocket transport (server/client, challenge-response auth, keepalive) |
| 5 | Node identity & announcement (signed announcements, peer storage) |
| 6 | Federation handshake (state machine, capability negotiation) |
| 7 | Identity registration (persistent registry, acceptance pipeline) |
| 8 | Space and room protocol (state derivation, membership, permissions) |
| 9 | Message exchange (MessageText events, validation, propagation) |
| 10 | Smoke test — 17-step end-to-end test (definition of done for Phase 1) |

Run `cargo test` and confirm all tests pass before advancing to the next layer.

## Local Node Mode

Phase 1 development runs exclusively in **Local Node Mode**:

- Config flag: `local_node: true`
- Transport: `ws://` (no TLS)
- No Trust Assertion required for identity registration
- Localhost only — no external network reachability
- Message size ceiling: 256 KB

This mode is used for all Layers 1–10 and all integration tests.

## Testing Strategy

- Unit tests for every crypto primitive (Layer 1)
- Unit tests for canonical serialization round-trips
- Event validation pipeline tests — one test per validation step
- Integration tests use two in-process Node instances (no real network)
- Layer 10 smoke test (spec 3.7.11) is the Phase 1 definition of done

## DECISIONS.md

Every implementation decision that goes beyond spec prescription must be recorded in `DECISIONS.md` before moving to the next layer. Format: title, date, layer, spec reference, decision narrative. This feeds into Chapter 4 (post-Phase 1 writeup).

## Phase 2 Preview

After Phase 1: Tauri (desktop shell) + Svelte (frontend) wraps the Phase 1 Rust backend. Design decisions are locked in `docs/xgen_ch6_client_design.md`. Spec sections 3.9–3.16 (state resolution, E2E encryption, higher Auth Tiers, space migration) are deferred to Phase 2.
