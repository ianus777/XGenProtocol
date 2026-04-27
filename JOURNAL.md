# XGen Protocol — Development Journal

This document is a chronological record of development activity on the XGen Protocol project.
It is intended to establish authorship, timeline, and scope of original work for intellectual
property purposes. Entries are written contemporaneously with the work described.

---

**Project:** XGen Protocol
**Author:** Jozef Nižnanský
**Organization:** Alchemy Dump
**Location:** Bratislava, Slovakia
**Repository:** https://github.com/ianus777/XGenProtocol
**License:** Business Source License 1.1 (converts to GPL on community handover)
**Journal started:** 2026-04-27

---

## Entry J-001 — Project Inception

**Date:** 2026-04-22
**Commit:** `43c6e28e` / `3b9a5660` — *Initial commit: XGen Protocol philosophy v0.3*

The XGen Protocol project was initiated. The initial commit establishes the philosophical
foundation of the protocol: a federated, open-source communication infrastructure layer
designed to sit beneath chat, community, and voice applications. The core thesis — that
no single entity should own the communication layer — is documented in `docs/xgen_ch1_philosophy.md`.

The protocol is conceived as a public infrastructure primitive, not a product.

---

## Entry J-002 — Repository Organisation

**Date:** 2026-04-23
**Commits:** `31b898d7` through `a75579d1`

Repository structure established. Legacy brainstorm documents removed. `.gitignore` created.
Document hierarchy reorganised into `docs/` directory. Project identity consolidated under
the XGen Protocol name.

---

## Entry J-003 — Philosophy and Architecture Documentation

**Date:** 2026-04-24 to 2026-04-25
**Commits:** `69231d0a` through `20968fe7`

Chapters 1 and 2 of the protocol documentation written:

- `docs/xgen_ch1_philosophy.md` — project philosophy and motivation
- `docs/xgen_ch2_architecture.md` — architecture design and primitives

The primitive hierarchy (Space → Room → Thread → Event) is defined. The cross-cutting
primitives — Identity (server-independent Ed25519 keypair) and Auth Module (pluggable
trust assertion) — are established as foundational design decisions.

---

## Entry J-004 — Technical Specification Complete (Phase 1 Scope)

**Date:** 2026-04-25 to 2026-04-26
**Commits:** `49fd0707` through `dc635409`

The authoritative technical specification is written and completed for Phase 1 scope:
`docs/xgen_ch3_specification.md`, sections 3.1 through 3.8.

Sections completed:

| Section | Title |
|---------|-------|
| 3.1 | Wire Format |
| 3.2 | Event Specification |
| 3.3 | Transport Protocol |
| 3.4 | Federation Handshake |
| 3.5 | Node Identity Protocol |
| 3.6 | Identity Registration Protocol |
| 3.7 | Space & Room Protocol |
| 3.8 | Auth Module — Tier 1 |

Sections 3.9–3.16 (Phase 2) are specified as deferred.

`IMPLEMENTATION_GUIDE_ph1.md` written — a 10-layer implementation roadmap for Phase 1,
specifying exact file structure, crate dependencies, testing strategy, and the Phase 1
definition of done (17-step smoke test, spec 3.7.11).

Rust crate skeleton committed: `xgen-common`, `xgen-node`, `xgen-client` with stub
`main.rs` and `lib.rs` files. All source files carry the BSL 1.1 copyright header.

License file added: BSL 1.1.

---

## Entry J-005 — Build Infrastructure and Versioning System

**Date:** 2026-04-27
**Commit:** `14b0c6ab` — *Add build infrastructure and versioning system*
**Tag:** `v0.1.0`

First successful compilation of the XGen Protocol codebase. The build infrastructure
is established:

- **Build target directory** moved to `C:/cargo-targets/XGenProtocol` (outside Google
  Drive) to prevent file locking by the Google Drive sync process, which caused the
  first two build attempts to freeze indefinitely.
- **`build.sh`** wrapper script: runs `cargo build` and copies output binaries to
  `bin/` in the project folder on Google Drive.
- **Versioning system** adopted — four-component format `[state].[section].[session].[build]`:
  - `state` — 0 while building; 1 when Phase 1 + Phase 2 complete and stable
  - `section` — spec section being implemented (1–16, mapping to spec 3.1–3.16)
  - `session` — increments each work session
  - `build` — auto-captured at compile time as `yymmdd-hhmm`
- **Build banner** — both binaries print version, git hash, and UTC build timestamp
  on startup, implemented in `xgen-common::build_info`.
- **`DECISIONS.md`** created — running log of implementation decisions beyond spec
  prescription, to be used as source material for Chapter 4 documentation.

Binaries at this point are stubs only. Retroactively designated version `0.0.0` in
semantic terms (no protocol logic implemented).

---

## Entry J-006 — Layer 1: Cryptographic Foundation

**Date:** 2026-04-27
**Commit:** `1a2143b3` — *Implement Layer 1 — cryptographic foundation (25 tests passing)*
**Tag:** `v0.1.1`

Layer 1 of the Phase 1 implementation is complete. All five cryptographic primitive
modules are implemented in `xgen-node/src/`, with 25 unit tests — all passing.

Files implemented:

| File | Spec ref | Description |
|------|----------|-------------|
| `crypto/encoding.rs` | 3.1.9 | base64url encode/decode, RFC 4648 §5, no padding, rejects standard base64 characters |
| `crypto/hashing.rs` | 3.2.3 | SHA-256 hash, lowercase hex output, hash URI format `xgen://hash/sha256:<hex>` |
| `crypto/signing.rs` | 3.2.4 | Ed25519 sign and verify, signature string format `ed25519:<base64url-pubkey>:<base64url-sig>` |
| `identity/keypair.rs` | 3.5.1 | Ed25519 keypair generation, encrypted file storage (ChaCha20-Poly1305 + Argon2id KDF), loading |
| `wire/canonical.rs` | 3.2.4 | Canonical Event JSON: fixed field order, sorted nested object keys, excludes `event_id` and `signature` |

Test coverage: 6 encoding tests, 4 hashing tests, 6 signing tests, 3 keypair tests,
6 canonical form tests.

New dependencies added: `chacha20poly1305 = 0.10`, `argon2 = 0.5`.

---

*This journal is maintained as a contemporaneous record. Each entry is committed to
the public Git repository at https://github.com/ianus777/XGenProtocol at the time
of writing, establishing a third-party timestamp via GitHub's servers.*

*For formal IP purposes, entries may be periodically exported, signed with a qualified
electronic signature (eIDAS), and/or anchored to a public blockchain timestamp service.*

---
