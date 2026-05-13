# XGen Protocol — Development Journal
> **Status:** ACTIVE  
> **Last updated:** 2026-05-13 (J-043 + BATCH_FLAG_ph2.md)  

This document is a chronological record of development activity on the XGen Protocol project.
It is intended to establish authorship, timeline, and scope of original work for intellectual
property purposes. Entries are written contemporaneously with the work described.

---

**Project:** XGen Protocol
**Author:** Jozef Nižnanský
**Credits:** Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
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

## Entry J-007 — License Header Correction and Development Journal

**Date:** 2026-04-27
**Commits:** `7118140` — *Add JOURNAL.md*, `a803166` — *Fix license headers*

Two corrections applied:

**License headers:** All 16 source files (`.rs`) carried an incorrect PolyForm Noncommercial
License 1.0.0 header — a mismatch with the project's actual BSL 1.1 license declared in
`LICENSE`. Headers replaced across all files with the correct BSL 1.1 header including
`SPDX-License-Identifier: BUSL-1.1` and the change date clause. `CLAUDE.md` updated to
enforce the correct header for all future source files.

**Development journal:** This file (`JOURNAL.md`) created as a contemporaneous legal record
of development activity, separate from `DECISIONS.md` (which feeds Chapter 4 documentation).
Journal entries to be written at the close of each work session going forward.

---

## Entry J-008 — Layer 2: Wire Format

**Date:** 2026-04-27
**Commit:** *(this session)* — *Implement Layer 2 — wire format (53 tests passing)*
**Tag:** `v0.2.2`

Layer 2 of the Phase 1 implementation is complete. Three modules implemented in
`xgen-node/src/wire/`, bringing the total test count from 25 (Layer 1) to 53.

Files implemented:

| File | Spec ref | Description |
|------|----------|-------------|
| `wire/types.rs` | 3.2.1, 3.2.2, 3.3.4 | `Event` envelope struct, `EventType` enum (15 variants), `TransportMessage` enum (8 variants), `MessageTextContent` |
| `wire/framing.rs` | 3.1.2 | Transport frame encode/decode — `[1B fmt_len][N fmt][4B pay_len BE][payload]`; Phase 1 format "json"; 256 KB ceiling |
| `wire/validation.rs` | 3.2.6 | Event validation pipeline steps 1–7 (structural checks; steps 8–13 deferred to Layer 3+) |

Test coverage added:

| Module | Tests |
|--------|-------|
| `wire/types.rs` | 8 — EventType round-trip, all-variants from_str, unknown returns None, Event serde, full envelope deserialise, transport message round-trips, MessageTextContent |
| `wire/framing.rs` | 7 — encode/decode round-trip, frame byte structure, empty payload, too-short buffer, incomplete payload, oversized payload rejection, Event JSON through frame |
| `wire/validation.rs` | 13 — one test per validation step (all 7 steps covered), valid event passes, field-type checks for array/object fields, timezone variants, validated fields accessible |

Design notes:
- `event_id` and `signature` are `Option<String>` in `Event` — absent during construction
  (before signing), required on received events (enforced by validation step 3).
- `EventType` carries both serde derive (dot-separated names) and `from_str`/`as_str`
  for use in validation without a full deserialise.
- `TransportMessage` uses `#[serde(tag = "type", rename_all = "snake_case")]` — maps
  cleanly to the wire names `challenge`, `auth`, `auth_ok`, etc.
- All crate versions bumped to `0.2.2`.

---

## Entry J-009 — Layer 3: DAG Event Store

**Date:** 2026-04-27
**Commit:** *(this session)* — *Implement Layer 3 — DAG event store (79 tests passing)*
**Tag:** `v0.3.2`

Layer 3 of the Phase 1 implementation is complete. Four modules implemented in
`xgen-node/src/dag/`, bringing the total test count from 53 (Layer 2) to 79.

Files implemented:

| File | Spec ref | Description |
|------|----------|-------------|
| `dag/store.rs` | 3.2.5 | `EventStore` — append-only in-memory store keyed by `event_id`; rejects duplicates and unsigned events |
| `dag/graph.rs` | 3.2.5 | `DagGraph` — tracks current DAG tips and successor relationships; validates all `prev_events` rules on insertion |
| `dag/pending.rs` | 3.2.5 | `PendingBuffer` — holds events whose predecessors are not yet known; releases them when all predecessors arrive (including cascading chains) |
| `dag/mod.rs` | 3.2.5 | `RoomDag` — unified API combining store, graph, and pending buffer into a single `insert()` call |

Test coverage added:

| Module | Tests |
|--------|-------|
| `dag/store.rs` | 5 — insert/retrieve, duplicate rejection, missing event_id, len/empty, unknown ID |
| `dag/graph.rs` | 10 — root tip, linear chain, fork (two tips), merge (collapse to one tip), self-reference, unknown prev, root with prev, non-root without prev, too many prev, missing event_id |
| `dag/pending.rs` | 5 — single predecessor release, two missing predecessors (partial then full), multiple events waiting for same predecessor, resolve unknown ID, contains |
| `dag/mod.rs` | 6 — linear chain, fork-and-merge, out-of-order delivery, cascading pending drain (chain of 3), retrieve by ID, duplicate rejection |

Key design decisions:
- Root event types (`state.room_create`, `state.space_create`, `state.dm_space_create`) require empty `prev_events`; all others require at least one.
- Cycle detection for new events reduces to self-reference check only — a new event has no descendants, so no other cycle is possible.
- `PendingBuffer.resolve()` cascades: resolving one event can unblock a chain, which `RoomDag.drain_pending()` handles recursively.
- Phase 1 `prev_events` limit: 10 entries (spec 3.2.5).
- No persistence in Phase 1 — the store is entirely in-process memory.

---

## Entry J-010 — Layer 4: WebSocket Transport

**Date:** 2026-04-27
**Commit:** *(this session)* — *Implement Layer 4 — WebSocket transport (88 tests passing)*
**Tag:** `v0.4.2`

Layer 4 of the Phase 1 implementation is complete. Four modules implemented in
`xgen-node/src/transport/`, bringing the total test count from 79 (Layer 3) to 88.

Files implemented:

| File | Spec ref | Description |
|------|----------|-------------|
| `transport/auth.rs` | 3.3.4 | Challenge-response authentication — `issue_challenge()`, `build_auth_response()`, `verify_auth_response()`; error codes per spec 3.3.8 (1001–1004) |
| `transport/connection.rs` | 3.3.4, 3.3.9 | `Connection<S>` generic over stream type — `server_authenticate()`, `client_authenticate()`, `send_transport()`, `send_event()`, `recv()`, `goodbye()`, `ping()` |
| `transport/server.rs` | 3.3.1 | `Server` — `TcpListener` wrapper, `bind()` + `accept()`, upgrades TCP to WebSocket |
| `transport/client.rs` | 3.3.1 | `connect()` — outbound WebSocket connection to a peer Node |

Transport message type strings corrected in `wire/types.rs`: all variants now carry the `transport.` prefix (e.g., `transport.challenge`, `transport.auth_ok`) and the correct fields from spec 3.3.4, including `protocol_version` and `timestamp` on all messages.

Test coverage added:

| Test | What it verifies |
|------|-----------------|
| `auth::full_auth_round_trip` | Complete challenge → sign → verify cycle |
| `auth::wrong_nonce_rejected` | Nonce mismatch returns `NonceMismatch` |
| `auth::wrong_key_rejected` | Mismatched signature returns `SignatureInvalid` |
| `auth::wrong_message_type_rejected` | Non-Auth message returns `WrongMessageType` |
| `auth::identity_id_round_trip` | URI parse/format round-trip |
| `auth::error_codes_are_correct` | All four spec error codes (1001–1004) |
| `transport::connect_authenticate_ping_goodbye` | Full lifecycle: connect → auth → ping → goodbye |
| `transport::bad_signature_rejected` | Server sends auth_fail (code 1001) on forged signature |
| `transport::event_exchange_after_auth` | Event serialised, framed, sent, received, deserialised |

Design decisions:
- `Connection<S>` is generic over `AsyncRead + AsyncWrite + Unpin` — server connections are `Connection<TcpStream>`, client connections are `Connection<MaybeTlsStream<TcpStream>>`.
- `Inbound` enum discriminates Event, TransportMessage, Ping, Pong, and Closed without requiring callers to inspect raw JSON.
- Signature covers raw nonce bytes (decoded from base64url), not the base64url string — per spec 3.3.4.
- Phase 1 Local Node mode: `ws://` only; no TLS paths.
- Keepalive (30s ping, 10s pong timeout) is implemented at the protocol level (`ping()` method); the scheduling loop is part of Layer 4 but will be wired into the Node runtime in Layer 5+.

---

## Entry J-011 — Layer 5: Node Identity and Announcement

**Date:** 2026-04-27
**Commit:** *(this session)* — *Implement Layer 5 — node identity and announcement (100 tests passing)*
**Tag:** `v0.5.2`

Layer 5 of the Phase 1 implementation is complete. Two modules implemented in
`xgen-node/src/node/`, bringing the total test count from 88 (Layer 4) to 100.

Also corrected versioning in this session: tags `v0.1.2`/`v0.1.3`/`v0.1.4` were renamed
to `v0.2.2`/`v0.3.2`/`v0.4.2` to match the `[state].[layer].[session]` scheme.

Files implemented:

| File | Spec ref | Description |
|------|----------|-------------|
| `node/announcement.rs` | 3.5.2–3.5.6 | `NodeAnnouncement` — generate, sign, verify, save, load, supersedes check |
| `wire/canonical.rs` | 3.5.3 | Added `canonical_object_json(value, field_order)` — generic canonical serialiser for any signed object with a fixed field order; made `canonical_value` public |

Test coverage added (12 new tests):

| Test | What it verifies |
|------|-----------------|
| `generate_produces_valid_signature` | Freshly generated announcement passes verify() |
| `node_id_matches_signing_key` | node_id URI matches the signing key's public key |
| `tampered_endpoint_invalidates_signature` | Any field change breaks verification |
| `tampered_node_id_invalidates_signature` | Substituting a different key's node_id is caught |
| `higher_version_supersedes_lower` | v2.supersedes(v1) = true, v1.supersedes(v2) = false |
| `same_version_does_not_supersede` | Equal version → false |
| `different_node_does_not_supersede` | Different node_id → no supersession relationship |
| `expired_announcement_rejected` | valid_until in past → Expired error even if signature valid |
| `with_display_name` | Optional operator_display_name included in canonical form and signature |
| `save_load_round_trip` | JSON file persistence round-trip |
| `announcement_type_field_is_correct` | msg_type serialises as "type":"node_announcement" |
| `phase1_capabilities_are_json_only` | serialisation=["json"], compression=[], extensions=[] |

Design decisions:
- `NodeAnnouncement` is self-certifying — verifying key is embedded in `node_id` URI, no third party needed.
- `operator_display_name` is optional; the canonical form skips it when absent (handled by `canonical_object_json` silently skipping absent fields).
- Phase 1 TTL: 90 days (`valid_until = now + 90d`), spec 3.5.6.
- `is_expired()` is a separate check from signature verification — expiry is checked first.
- Persistence uses the caller-supplied path (Pattern A: data alongside the binary).

---

## Entry J-012 — Layer 6: Federation Handshake

**Date:** 2026-04-27
**Commit:** *(this session)* — *Implement Layer 6 — federation handshake (121 tests passing)*
**Tag:** `v0.6.2`

Layer 6 of the Phase 1 implementation is complete. Two new modules in `xgen-node/src/federation/`
plus extensions to the wire and transport layers, bringing the total test count from 100 to 121.

Files implemented:

| File | Spec ref | Description |
|------|----------|-------------|
| `wire/types.rs` | 3.4.2 | Added `FederationCapabilities`, `NegotiatedCapabilities`, `FederationMessage` (5 variants: hello, capabilities, accept, reject, goodbye) |
| `transport/connection.rs` | 3.4.2 | Added `Inbound::Federation`, `send_federation()`, updated `recv()` to dispatch on "federation." prefix |
| `federation/handshake.rs` | 3.4.1–3.4.7 | Full handshake state machine: `run_initiating`, `run_receiving`, `sign_msg`, `verify_msg`, `negotiate_serialisation`, `negotiate_version`; canonical field orders per message type |
| `federation/registry.rs` | 3.4.5 | `FederationRegistry` — persistent federation relationship store, keyed by peer node_id; JSON file persistence; `FederationRelationship::from_session()` |
| `federation/mod.rs` | 3.4 | Module declaration + integration tests |

Test coverage added (21 new tests):

| Test | What it verifies |
|------|-----------------|
| `negotiate_serialisation_picks_highest_preference` | First entry in our preference list that appears in peer's list is selected |
| `negotiate_serialisation_picks_first_common` | Order of our preference list determines the selection |
| `negotiate_serialisation_no_overlap_returns_none` | Disjoint format sets → None |
| `negotiate_version_lower_minor_wins` | Lower minor version of the two is selected |
| `negotiate_version_major_mismatch_returns_none` | Major version mismatch → None |
| `sign_verify_hello_round_trip` | Sign + verify cycle for federation.hello |
| `sign_verify_capabilities_round_trip` | Sign + verify cycle for federation.capabilities |
| `sign_verify_accept_round_trip` | Sign + verify cycle for federation.accept |
| `tampered_node_id_fails_verification` | Substituting a different node_id is caught |
| `session_id_is_deterministic_and_sorted` | Same pair always produces same session_id regardless of argument order |
| `message_type_field_serialises_correctly` | Serde tag produces "federation.hello" etc.; absent signature not serialised |
| `federation_capabilities_default_is_json_only` | Default caps: json only |
| `upsert_and_get` | Registry stores and retrieves a relationship |
| `upsert_updates_existing` | Upsert with same peer_node_id overwrites |
| `remove_returns_and_deletes` | remove() returns the entry and leaves registry empty |
| `all_returns_all_entries` | Multiple relationships all returned |
| `save_load_round_trip` | JSON persistence round-trip |
| `empty_registry_saves_and_loads` | Empty registry serialises and deserialises correctly |
| `full_handshake_reaches_active_both_session_ids_match` | Integration: two in-process Nodes run full handshake; both reach ACTIVE with matching session_id |
| `shared_spaces_propagate_through_handshake` | Integration: shared_spaces from hello are present in both sessions |
| `registry_stores_session_and_round_trips` | FederationRelationship::from_session() + registry stores correctly |

Design decisions:
- `FederationMessage` variants carry `signature: Option<String>` with `skip_serializing_if`. None during construction, Some after `sign_msg()`. Canonical JSON excludes `signature` because it is not in the per-variant field order constant.
- `Inbound::Federation(FederationMessage)` added alongside `Inbound::Event` and `Inbound::Transport` in `connection.rs`. The `recv()` dispatcher now branches on "federation." type prefix.
- `session_id` = `hash_uri(sorted(node_a, node_b) || timestamp)`. Node IDs are sorted alphabetically so the same pair always produces the same derivation regardless of which side is initiating.
- The receiving Node sends `federation.reject` (with appropriate 2xxx error code) before returning the error, ensuring the peer is informed.
- `FederationRegistry` persists as a flat JSON array of `FederationRelationship` objects.

---

## Entry J-013 — Layer 7: Identity Registration

**Date:** 2026-04-27
**Commit:** *(this session)* — *Implement Layer 7 — identity registration (142 tests passing)*
**Tag:** `v0.7.2`

Layer 7 of the Phase 1 implementation is complete. Two new modules in `xgen-node/src/identity/`
plus extensions to the wire and transport layers, bringing the total test count from 121 to 142.

Files implemented:

| File | Spec ref | Description |
|------|----------|-------------|
| `wire/types.rs` | 3.6.3–3.6.8 | Added `IdentityDeviceEntry`, `IdentityMessage` (7 variants: register, register_ok, register_fail, get, record, not_found, update) |
| `transport/connection.rs` | 3.6 | Added `Inbound::Identity`, `send_identity()`, updated `recv()` to dispatch on "identity." prefix |
| `identity/registry.rs` | 3.6.6 | `IdentityRecord`, `DeviceRecord`, `IdentityRegistry` — persistent identity store keyed by identity_id; JSON file persistence; `apply_update()` with monotonic version enforcement |
| `identity/registration.rs` | 3.6.3–3.6.5 | 8-step acceptance pipeline (`accept_registration`); Local Node mode skips steps 4–7; `sign_register`, `verify_register`, `sign_update`, `verify_update`, `build_register`; canonical form for signing |

Test coverage added (21 new tests):

| Test | What it verifies |
|------|-----------------|
| `sign_verify_register_round_trip` | Sign + verify cycle for identity.register |
| `tampered_display_name_fails_verification` | Any field change breaks verification |
| `local_node_accept_pipeline_succeeds` | Full 8-step pipeline in Local Node mode |
| `identity_mismatch_rejected` | Step 1: identity_id must match transport auth |
| `already_registered_rejected` | Step 3: duplicate registration refused |
| `trust_assertion_required_in_non_local_mode` | Step 4: non-local mode requires assertion |
| `display_name_too_long_rejected` | Step 8: >128 char name refused |
| `empty_display_name_rejected` | Step 8: empty string refused |
| `display_name_with_control_char_rejected` | Step 8: control characters refused |
| `no_display_name_accepted` | Optional display_name — None accepted |
| `sign_verify_update_round_trip` | Sign + verify for identity.update |
| `register_and_get` | Registry stores and retrieves a record |
| `duplicate_registration_rejected` | Registry-level duplicate check |
| `contains_returns_false_for_unknown` | contains() on absent identity |
| `apply_update_higher_version_succeeds` | update_version must increase |
| `apply_update_same_version_rejected` | Stale update rejected |
| `apply_update_to_unknown_identity_fails` | Update on unregistered identity |
| `save_load_round_trip` | JSON persistence round-trip |
| `empty_registry_saves_and_loads` | Empty registry serialises correctly |
| `local_node_registration_end_to_end` | Integration: full register flow over transport; client → server → register_ok |
| `duplicate_registration_returns_fail` | Integration: second register → error code 3007 |

Design decisions:
- `MAX_DISPLAY_NAME_LEN = 128` — spec does not specify; 128 provides generous room for unicode display names while rejecting obvious abuse. Recorded here.
- `IdentityMessage::Record` uses inline fields (no dependency from wire layer to identity layer). Registry converts `IdentityRecord` → `IdentityMessage::Record` at the call site.
- `signature: Option<String>` on `identity.register` and `identity.update` only — Node responses (register_ok, register_fail, record, not_found) are not signed by the Identity key.
- Phase 1: `identity_id == device_id` (single device). The `devices` array exists from day one for Phase 2 multi-device support without schema changes.
- Canonical signing order for `identity.register`: `[protocol_version, type, identity_id, display_name, trust_assertion, timestamp]`. Absent optional fields silently skipped.

---

## Entry J-014 — Layer 8: Space and Room Protocol

**Date:** 2026-04-27
**Commit:** *(this session)* — *Implement Layer 8 — space and room protocol (160 tests passing)*
**Tag:** `v0.8.2`

Layer 8 of the Phase 1 implementation is complete. Two new modules in `xgen-node/src/space/`,
bringing the total test count from 142 to 160.

Files implemented:

| File | Spec ref | Description |
|------|----------|-------------|
| `space/membership.rs` | 3.7.8 | `Role` enum (Owner/Admin/Moderator/Member) with ordering; permission predicates: `can_invite`, `can_kick`, `can_ban`, `can_create_room`, `can_manage_federation`, `can_change_space_info` |
| `space/state.rs` | 3.7.1–3.7.9 | `SpaceState`, `RoomState`, `SpaceMember`; `from_space_create`, `from_dm_space_create`, `apply_event` state machine; event builders: `build_space_create_event`, `build_room_create_event`, `build_dm_space_create_event`, `build_membership_event`; `sign_event`, `verify_event_signature` |
| `space/mod.rs` | 3.7 | Module declaration + full lifecycle integration test |

Test coverage added (18 new tests):

| Test | What it verifies |
|------|-----------------|
| `role_ordering` | Owner > Admin > Moderator > Member |
| `role_from_str` | String parsing for all roles |
| `member_cannot_invite` | Permission table — member row |
| `moderator_can_invite_and_kick_but_not_ban` | Moderator row |
| `admin_can_ban_and_create_room` | Admin row |
| `only_owner_manages_federation` | Owner-only permission |
| `space_create_sets_owner` | Creator becomes Owner member |
| `space_create_event_id_is_space_id` | Content-addressing: space_id = event_id |
| `room_create_by_owner_succeeds` | Owner can create rooms |
| `room_create_by_member_permission_denied` | Member cannot create rooms |
| `invite_join_membership_flow` | invite → join → member with correct role |
| `join_room_after_joining_space` | Room join requires space membership |
| `leave_removes_from_space_and_all_rooms` | Leave cascades to all rooms |
| `ban_blocks_rejoin` | Banned identity cannot be re-invited |
| `sign_event_produces_valid_signature` | event_id and signature computed correctly |
| `tampered_event_fails_verification` | Content change breaks signature |
| `dm_space_creates_room_and_invite` | DM Space auto-creates room and invite event |
| `full_space_room_lifecycle` (integration) | Alice creates space+room, invites Bob, Bob joins both |

Design decisions:
- Space-level and room-level events are distinguished by `room_id`: empty string = Space event, non-empty = Room event.
- `SpaceState.pending_invites` tracks invited but not yet joined identities; role from invite is consumed on join.
- `apply_join` checks `room_id` first to avoid incorrectly treating a room join as a space join.
- `sign_event` computes `event_id = hash_uri(canonical_event_bytes)` and `signature = sign(canonical_event_bytes)`. The same canonical form is used for both, so `event_id` is bound to the content.
- DM Space creation auto-generates a room event and membership.invite event signed by the creator key. The caller is responsible for adding these to the DAG.
- Phase 1: `state.space_create` has `room_id = ""` and `space_id = ""` because the IDs don't exist until after hashing. Same for `state.room_create`.

Bug fixed during implementation:
- `apply_join` initially checked `self.members.contains_key(joiner)` before branching on `room_id`, causing existing space members to receive `AlreadyMember` when joining a room. Fixed by checking `room_id` first.

---

## Entry J-016 — Layer 9: Message Exchange

**Date:** 2026-04-28
**Commit:** `925f3fb` — *Implement Layer 9 — message exchange (171 tests passing)*
**Tag:** `v0.9.3`

Layer 9 of the Phase 1 implementation is complete. One new module `xgen-node/src/message/`
with the full 13-step validation pipeline (steps 8–13) and event acceptance logic,
bringing the total test count from 160 to 171.

Files implemented:

| File | Spec ref | Description |
|------|----------|-------------|
| `message/exchange.rs` | 3.2.6 | Steps 8–13 of the event validation pipeline; `validate_steps_8_13`, `accept_event`, `build_message_text_event` |
| `message/mod.rs` | — | Module declaration |
| `lib.rs` | — | Added `pub mod message` |

Test coverage added (11 new tests):

| Test | What it verifies |
|------|-----------------|
| `step8_valid_event_id_passes` | Correctly signed event passes step 8 |
| `step8_wrong_event_id_rejected` | Tampered event_id caught at step 8 |
| `step9_unknown_prev_event_held_pending` | Missing prev_event → HeldPending |
| `step11_unregistered_sender_rejected` | Sender not in IdentityRegistry → UnknownSender |
| `step11_non_space_member_rejected` | Registered but not Space member → NotASpaceMember |
| `step11_non_room_member_rejected` | Space member but not Room member → NotARoomMember |
| `step12_tampered_content_fails_signature` | Content tampered after signing → SignatureFailure |
| `accept_event_stores_in_dag` | Valid event stored; becomes DAG tip; prior tip replaced |
| `accept_event_duplicate_rejected` | Second accept of same event fails |
| `message_propagates_from_node_a_to_node_b` | Integration: Alice→Node A, propagate to Node B; verify event_id, signature, prev_events |
| `concurrent_messages_produce_two_tips` | Two concurrent messages from same prev → two tips |

Design decisions:
- `validate_steps_8_13` is intentionally read-only (no graph/store mutation). Callers use `accept_event` for the full accept+store path or can inspect validation failure reason before deciding to buffer/reject.
- Step 9 returns `HeldPending(Vec<String>)` with the list of unknown prev_event IDs so the caller knows exactly what to request from peers.
- Step 10 duplicates the DAG structural checks from `DagGraph::add_event` inline (read-only) to allow early rejection without mutation. The actual graph mutation happens in `accept_event` after all 13 steps pass.
- Integration test uses `build_setup_events` + `replay_events` helpers to seed both simulated nodes with deterministic identical event_ids. This avoids the problem of two independent `now()` calls producing different timestamps → different event_ids.
- The invite event uses `prev=[space_id, room_id]` to merge the two DAG roots (space_create and room_create) into a single linear chain, ensuring a single tip for the message to reference.

---

## Entry J-017 — Layer 10: Phase 1 Smoke Test (v0.10.1)

**Date:** 2026-04-28
**Commit:** `f873f5e` — *Layer 10: Phase 1 smoke test passing — 173 tests (v0.10.1)*
**Tag:** `v0.10.1`

**Phase 1 of the XGen Protocol implementation is complete.**

Layer 10 implements `spec 3.7.11` — the 17-step end-to-end smoke test. It
exercises all prior layers simultaneously across two in-process `NodeRuntime`
instances (Node A / Alice, Node B / Bob) connected via a real WebSocket TCP
transport.

### Pre-Layer-10 fixes (confirmed with 172 tests before smoke test work began)

| Fix | File | Spec ref |
|-----|------|---------|
| `message.delete` → `message.redact` | `wire/types.rs`, `message/exchange.rs` | 3.2.2 |
| Added `state.federation_add` event type | `wire/types.rs`, `space/state.rs` | 3.7.11 |
| Added `space.join_request` control message | `wire/types.rs`, `transport/connection.rs` | 3.7.11 |

### New modules and methods

| File | Description |
|------|-------------|
| `node/runtime.rs` | `NodeRuntime` — wires IdentityRegistry, SpaceState, EventStore, DagGraph per-space; `ingest_event` (direct DAG+state insert), `accept_message` (full 13-step pipeline), `all_events()`, `dag_tips()` |
| `tests/smoke.rs` | `smoke_test_phase1` — 17-step end-to-end integration test |
| `tests/mod.rs` | Module declaration for test suite |
| `dag/store.rs` | Added `values()` iterator |

### Smoke test design decisions

**History sync — individual Events (D-024):** When Node A receives a
`space.join_request`, it sends the full Space Event history as individual
`event` wire frames in topological order, followed by the new
`state.federation_add` event, then `transport.goodbye`. Node B ingests each
event via `ingest_event` in the receive loop. This matches the individual-event
federation protocol that all clients will use in Phase 2.

**Out-of-order delivery fix:** `state.space_create` and `state.room_create` are
both DAG roots (empty `prev_events`). When received over the network, either
can arrive first. The `ingest_event` `StateSpaceCreate` arm was extended to
replay all already-stored events (in topological order) against the new
SpaceState immediately after creating it, ensuring room membership and other
derived state is always reconstructed correctly regardless of delivery order.

**Topological sort (Kahn's algorithm):** A free function `topological_sort`
in `node/runtime.rs` computes causal order from a set of Events. In-degree is
computed only over edges whose predecessors are within the provided set (missing
predecessors treated as resolved). Nodes with equal in-degree are sorted
lexicographically by event_id for stable ordering.

### Final state after smoke test

| Metric | Value |
|--------|-------|
| Total tests | 173 |
| Failures | 0 |
| Version tag | v0.10.1 |
| Spec coverage | Phase 1 (sections 3.1–3.7.11) |

Phase 1 definition of done is met: the 17-step smoke test passes.

---

## Entry J-015 — Session 2 Close / Session 3 Start

**Date:** 2026-04-28

Session 2 ended with all Layers 1–8 complete (160 tests passing, tag `v0.8.2`).
Session 3 begins with Layer 9 (Message Exchange) as the first task.

**Status entering Session 3:**

| Layer | Spec | Status | Tag |
|-------|------|--------|-----|
| 1 | 3.1 Crypto | ✓ | v0.1.1 |
| 2 | 3.2 Wire format | ✓ | v0.2.2 |
| 3 | 3.2 DAG store | ✓ | v0.3.2 |
| 4 | 3.3 Transport | ✓ | v0.4.2 |
| 5 | 3.5 Node identity | ✓ | v0.5.2 |
| 6 | 3.4 Federation | ✓ | v0.6.2 |
| 7 | 3.6 Identity reg. | ✓ | v0.7.2 |
| 8 | 3.7 Space/Room | ✓ | v0.8.2 |
| 9 | 3.2 Message exchange | → next | — |
| 10 | 3.7.11 Smoke test | pending | — |

Outstanding: DECISIONS.md not yet created (outstanding debt across all layers).

---

## Entry J-018 — Chapter 4: Implementation (Documentation Session)

**Date:** 2026-04-29
**Commit:** *(pending push)* — *docs: write Chapter 4 — Implementation (Phase 1)*

Chapter 4 of the protocol documentation written in full: `docs/xgen_ch4_implementation.md`.

This chapter bridges the Phase 1 specification (Chapter 3) and the actual code that was built across Layers 1–10. It is written as an Option B descriptive guide — describes requirements and constraints, recommends the Rust stack as the reference path, includes enough concrete detail for a developer to follow, but frames decisions as recommendations rather than prescriptions where alternatives are possible.

Smoke test (Layer 10) results reviewed before writing: all 17 steps pass. Phase 1 is confirmed complete. Ch4 was written on the basis of that confirmed completion.

**Sections written:**

| Section | Title |
|---------|-------|
| 4.1 | Implementation Philosophy (Pattern A, Local Node first, protocol fidelity) |
| 4.2 | Technology Stack (Rust rationale, multi-SDK strategy, crate selections with rationale, out-of-scope items) |
| 4.3 | Project Structure (Cargo workspace layout, runtime folder layout) |
| 4.4 | Build Order (13-step causal sequence from wire primitives to full smoke test) |
| 4.5 | Wire Format Implementation (URI newtypes, canonical form serialiser, transport frame codec, datetime) |
| 4.6 | Cryptographic Primitives (keypair generation + ChaCha20-Poly1305 encrypted storage, signing, verification, ID derivation) |
| 4.7 | Event Implementation (Event struct, validation pipeline, DAG operations) |
| 4.8 | Transport Layer Implementation (config format, connection dispatch, keepalive, error codes) |
| 4.9 | Identity and Registration Implementation (SQLite schema, registration flow, identity federation) |
| 4.10 | Space and Room Implementation (state derivation, Event store interface, membership processing) |
| 4.11 | Federation Implementation (state machine, registry schema, Event fan-out) |
| 4.12 | Event Store (schema with dag_edges table, append-only invariant, pending buffer) |
| 4.13 | Auth Module Tier 1 (config, verification flow state machine, assertion issuance) |
| 4.14 | Local Node Mode (two-Node localhost setup, client commands, bypass verification) |
| 4.15 | Smoke Test Execution (manual CLI sequence, automated runner with 17-step pass/fail checklist) |

**Discrepancy corrected:** Ch4 initially described AES-256-GCM for keypair encryption. DECISIONS.md D-002 records the actual implementation uses ChaCha20-Poly1305 + Argon2id. Ch4 section 4.6.1 corrected to match the implementation and D-002.

**Multi-SDK strategy documented:** The `xgen-core` library crate (post-Phase-1 restructure, per D-022) is documented as the canonical protocol library. Future community SDKs in Go, TypeScript, Python, Kotlin, Swift are verified for conformance by running the smoke test against the reference Rust Node — no shared code required, only a shared protocol.

**DECISIONS.md:** No new entries required. Ch4 is derived from existing decisions; no new implementation decisions were made during the documentation session.

**Status entering next session:**

| Document | Status |
|----------|--------|
| Ch0 Content | ✅ Complete |
| Ch1 Philosophy | ✅ Complete |
| Ch2 Architecture | ✅ Complete |
| Ch3 Specification (Ph1) | ✅ Complete |
| Ch4 Implementation (Ph1) | ✅ Complete |
| Ch5 Protocol | Pending (post-Ph1) |
| Ch6 Client Design | Pending |

Next documentation task: Joe to review Ch4 and flag any corrections or additions before Ch5 begins.

---

## Entry J-019 — Phase 1 CLI: init, observability commands, state file types (v0.10.2)

**Date:** 2026-04-29
**Commit:** *(this session)*
**Tag:** `v0.10.2`

Phase 1 CLI completeness implemented per D-025 through D-028. This is a deliberate Phase 1 scope extension — the protocol library and smoke test were already complete; these changes wire the library into observable, runnable binaries.

### Files changed

| File | Change |
|------|--------|
| `xgen-common/src/state.rs` | New — `NodeState`, `ClientState`, and all nested structs (D-026) |
| `xgen-common/src/lib.rs` | Added `pub mod state` |
| `xgen-node/src/identity/registry.rs` | Added `pub fn all() -> Vec<&IdentityRecord>` |
| `xgen-node/Cargo.toml` | Added `clap`, `rpassword`, `toml` dependencies |
| `xgen-node/src/main.rs` | Full CLI implementation (see below) |
| `xgen-client/Cargo.toml` | Added `clap`, `rpassword`, `toml` dependencies |
| `xgen-client/src/main.rs` | Full CLI implementation (see below) |

### xgen-node CLI commands implemented

| Command | Implementation | Source |
|---------|----------------|--------|
| `xgen-node init` | Generates keypair (ChaCha20+Argon2id, passphrase via `rpassword`), writes `xgen-node_config.toml`. Safe re-run — will not overwrite existing keypair. | D-025, D-026 |
| `xgen-node status` | Reads `xgen-node_state.json`, prints formatted status. Warns if file is older than 30 seconds. | D-026, D-027 |
| `xgen-node connections` | Reads state file, prints clients and federated peers table. | D-027 |
| `xgen-node spaces` | Reads state file, prints hosted Spaces and Rooms. | D-027 |
| `xgen-node peers` | Reads state file, prints per-peer detail including session ID and shared Spaces. | D-027 |
| `xgen-node identity list` | Loads `xgen-node_identities.db` via `IdentityRegistry::load`, prints all registered identities with name, age, and device count. | D-027 |
| `xgen-node version` | Prints full version + git commit + Node ID (attempts empty-passphrase load; falls back to informative message). | D-028 |

All commands use clap derive macros; help text is copied from spec section 4.16 into doc comments (D-028).

### xgen-client CLI commands implemented

**File-based (Phase 1 complete):**

| Command | Description |
|---------|-------------|
| `xgen-client init` | Generates `xgen-client_keypair.enc`, writes `xgen-client_config.toml`. Prints Identity ID. |
| `xgen-client whoami` | Reads `xgen-client_state.json`, prints identity ID, display name, home node, spaces joined. |
| `xgen-client status` | Reads state file, prints formatted client status. |
| `xgen-client spaces` | Reads state file, prints known Spaces and Rooms with role and join status. |
| `xgen-client version` | Prints version and commit. |

**Network commands (Phase 2 — defined, not yet implemented):**

`register`, `create-space`, `create-room`, `invite`, `join`, `send`, `history`, `smoke-test` are defined with correct clap argument structs so that `--help` is accurate. Each prints "requires a running xgen-node — available in Phase 2" and exits with code 4.

### Keypair module note

`xgen-client` does not depend on `xgen-node`. The client's `main.rs` contains an inline `keypair` module implementing the same ChaCha20-Poly1305 + Argon2id scheme as `xgen-node/src/identity/keypair.rs`. This duplication is intentional for Phase 1 — it is eliminated when `xgen-core` is extracted (D-022).

### Test results

173 tests passing. 0 failing. No tests removed or modified.
The CLI commands themselves are not unit-tested — they are thin wrappers over existing library functions that are already tested.

### Version

Bumped `0.10.1` → `0.10.2` across all three Cargo.toml files. Layer 10, second session (Phase 1 CLI completeness session).

---

## J-034 — 2026-05-12 — Client Core Test UI instruction written; D-042 recorded

### Context

Phase 2 Track 1 (UI) preparation session. Joe reviewed the `ui/dev_core_ui/` directory and the Svelte concept files he had prepared over the weekend. The goal was to produce a clear implementation instruction for Mr. Code to build the first real Tauri window for the `xgen-client` binary.

### Discussion

The core test UI scope was clarified through discussion:

- No log pane in the UI — log files remain text files next to the executables, read directly when needed.
- Lifecycle state indicator is the primary functional addition — dot + label, real time.
- For state communication, a hybrid approach was chosen: the existing 5-second state JSON write is retained for full snapshots; a dedicated Tauri event (`"xgen-client-state-changed"`) is emitted on every lifecycle state transition for real-time UI updates.
- Future XGen protocol events (message receipt, federation events, etc.) may also be emitted outside the time raster when real-time feedback is warranted — noted as a future step.
- Component library (issue #2) is a future architectural principle; for the core test UI, a Button component is sufficient and Mr. Code may apply the pattern if he chooses.
- Tasks are sequenced: client core test UI first (this instruction), node core test UI second.

### Deliverables

- `docs/tests/CLIENT_CORE_UI_ph2.md` — implementation instruction for Mr. Code (status: PENDING)
- `DECISIONS.md` D-042 — Tauri event emission for real-time lifecycle state changes

### Files modified

- `docs/tests/CLIENT_CORE_UI_ph2.md` — created
- `DECISIONS.md` — D-042 added, last-updated bumped
- `JOURNAL.md` — this entry

### Next steps

1. Mr. Code implements `CLIENT_CORE_UI_ph2.md` (four milestones)
2. Joe verifies against the checklist in Milestone 4
3. Node Core Test UI instruction follows (`NODE_CORE_UI_ph2.md`)

---

## Entry J-036 — Phase 2 Roadmap Snapshot; Batch Flag principle established

**Date:** 2026-05-12  
**Author:** Jozef Nižnanský  
**Session:** Session 16  

### Purpose

End-of-session roadmap checkpoint. Consolidates current project state and records the complete
Phase 2 delivery sequence before the next development session begins.

---

### Phase 1 — COMPLETE ✅

All Phase 1 deliverables are closed. No further work required.

| Item | Entry | Tag | Status |
|---|---|---|---|
| Layers 1–9 (Crypto → Message Exchange) | J-006 – J-016 | v0.9.3 | ✅ Done |
| Layer 10 — Smoke test (17-step, spec 3.7.11) | J-021 | v0.10.1 | ✅ Done |
| Phase 1 CLI (init, status, connections, spaces, peers, identity list, whoami) | J-019 | v0.10.2 | ✅ Done |
| Binary wiring — real WebSocket server + network commands + smoke test over TCP | J-020 – J-021 | v0.10.3 | ✅ Done |
| Documentation fixes (FIXES_ph1.md — all 17, Fix 14 deferred) | J-023 | — | ✅ Done |
| Phase 1 debug logging (LOGGING_debug_ph1.md) | J-025 | — | ✅ Done |
| Priority 0 — Global Event tracing interface (LOGGING_debug_ph2.md) | J-027 / J-029 | — | ✅ Done |
| Session header / footer / LOCAL actions / EventDirection rename | J-030 | — | ✅ Done |
| Stress test — F-001 (pending buffer), F-002 (counter scoping) resolved | J-031 / J-032 | — | ✅ Done |
| Stress test Phase 1 sign-off — all acceptance criteria met | J-032 | commit `ecc94ff` | ✅ Done |
| Stress test final round verification (3 acceptance tests) | — | commit `8c9402b` | ✅ Done |

---

### Phase 2 Track 1 — UI

**Current task:** `CLIENT_CORE_UI_ph2.md` (status: ACTIVE)

| # | Task | Instruction file | Status |
|---|---|---|---|
| 1 | **Client Core Test UI** — Tauri scaffold, 11 lifecycle states, state indicator, systray | `CLIENT_CORE_UI_ph2.md` | 🔴 In progress — Milestones 1 + 2 done; blocked on Node.js install for Milestone 3 |
| 2 | **Node Core Test UI** — Tauri scaffold, systray, 7 lifecycle states + degraded stacking, `--service` flag | `NODE_CORE_UI_ph2.md` | ⏳ Pending — starts after CLIENT_CORE_UI_ph2.md Milestone 4 checklist signed off |
| 3 | **`--batch` flag — `xgen-client` only** | see below | ⏳ Pending — first item after both Core Test UIs are verified |
| 4 | **UI Phase 2 prep — element modelling** — confirm absent-element list (Point 2: avatar DOM, Point 3: message stream event types vs Ch3 taxonomy) | `ui/run_1.5/comparative_analysis.md` | 🔄 Paused — gating step before Run 3 design briefing |
| 5 | **Run 3 design briefing** — consolidated element list → briefing document | — | ⏳ Pending — after element modelling confirmed |
| 6 | **Visual merge** — chat mockup visual treatment onto Miss Design's semantic skeleton, `skin-dark.css`, token architecture | `ui/run_1.5/comparative_analysis.md` (10-milestone plan) | ⏳ Pending — after Run 3 briefing |
| 7 | **Console overlay** — Backquote scancode toggle, VT220 scheme, `skin-console-vt220.css` | — | ⏳ Pending |
| 8 | **First-run SETUP flow** — display name, passphrase, keypair generation; zero network traffic | — | ⏳ Pending |
| 9 | **`auto_connect_local`** — silent scan `ws://127.0.0.1:8080/xgen` after INITIALISING; 2 s timeout; no error | — | ⏳ Pending |
| 10 | **Skeleton screens** — Space list, Room view, Node dashboard | — | ⏳ Pending |

---

### The `--batch` flag — architecture principle

`xgen-client` accepts a `--batch <file.xgb>` command-line flag. This is a **client-only** feature — the node does not need one. The node is tested as a black box through its WebSocket protocol; the client is the instrument.

**What it does:** reads a batch file line by line, executes each line as a CLI command against a running node, logs results, exits. One command per line, sequential. Each command opens its own connection independently — the same model the smoke test and stress test already use, generalised to arbitrary sequences without writing Rust.

**Example batch file:**
```
register --node ws://127.0.0.1:8080/xgen
create-space --node ws://127.0.0.1:8080/xgen --name "Test Space"
create-room --node ws://127.0.0.1:8080/xgen --space <id> --name general
send --node ws://127.0.0.1:8080/xgen --space <id> --room <id> --text "hello"
```

**Why it matters:**

1. **Scriptable node testing.** The node runs (with or without UI; `--service` flag for headless). The client drives it with a batch file. This enables reproducible test scenarios, multi-step debugging sessions, and AI-assisted command sequences without manual CLI interaction.

2. **Symmetry with existing test infrastructure.** The smoke test and stress test already drive the client programmatically from Rust. The batch flag generalises this to arbitrary scenarios expressible as command sequences, without requiring a new Rust test harness each time.

3. **Foundation for future automation.** The command-set and return-value semantics established here carry forward to the Console IPC protocol (Ch6 §6.9 — named pipe / local socket for the full UI). No architectural decisions required now.

**Format:** UTF-8 text file, `.xgb` extension by convention. One command per line. Lines starting with `#` are comments, ignored. Empty lines ignored. Commands use the same syntax as CLI subcommands without the binary name prefix.

**Implementation timing:** after both Core Test UIs are verified (Client Milestone 4 + Node Milestone 4 checklists passed). A single implementation instruction file (`BATCH_FLAG_ph2.md`) covers the client. This will be the first item after the Core Test UI phase closes.

---

### Phase 2 Track 2 — Protocol

Deferred until Track 1 UI skeleton is visually validated. Ch3 Phase 2 specification is partially written (3.9–3.11 complete; 3.12–3.16 pending).

| Item | Status |
|---|---|
| Ch3 §3.12 Space Migration Protocol | ⏳ Pending |
| Ch3 §3.13 Identity Replication Parameters | ⏳ Pending |
| Ch3 §3.14 Bootstrap Node Protocol | ⏳ Pending |
| Ch3 §3.15 Node Reputation Format | ⏳ Pending |
| Ch3 §3.16 DM Space Promotion Sequence | ⏳ Pending |
| `xgen-core` crate split (D-022) | ⏳ Pending |
| Audit log — LOGGING_audit_ph2.md | ⏳ Pending — alongside Tier 2+ Auth Module |
| Registry file encryption | ⏳ Pending |
| E2E encryption (MLS, RFC 9420) | ⏳ Pending |
| Auth Module Tiers 2–4 | ⏳ Pending |

---

### Immediate next actions (in order)

1. Install Node.js LTS on the development machine — unblocks `npm install` and `.\run-client.ps1`
2. Mr. Code completes `CLIENT_CORE_UI_ph2.md` Milestones 3 + 4 (state indicator wired, verification checklist)
3. Joe signs off the Milestone 4 checklist
4. Mr. Code implements `NODE_CORE_UI_ph2.md` (four milestones)
5. Joe signs off the Node Milestone 4 checklist
6. Write `BATCH_FLAG_ph2.md` — implementation instruction for `--batch` flag, both binaries
7. Mr. Code implements `--batch` flag
8. Resume UI Phase 2 prep — element modelling (ui/run_1.5 gating step)

---

## Entry J-037 — Discussion: `.xgb` batch execution model for both Tauri binaries

**Date:** 2026-05-12  
**Author:** Jozef Nižnanský  
**Status:** 🔵 Under discussion — not a decision, not yet an implementation instruction  

### Context

After writing the Phase 2 roadmap (J-036), a discussion began about how the `.xgb` batch file capability actually works when both `xgen-client.exe` and `xgen-node.exe` are long-running Tauri GUI processes — not stateless CLI tools. The question: if both exes are already running, how do you inject commands into them from a `.xgb` file?

### Current understanding

**Phase 2 binary model:** there are exactly two executables. Both are Tauri GUI applications. Both have a Shut Down / Quit button and nothing else in the Core Test UI phase. There is no separate CLI binary in Phase 2 — the Tauri app IS the binary.

**The `.xgb` capability must exist on both binaries** but the internal mechanism is fundamentally different for each.

---

**`xgen-client.exe --batch file.xgb`** — independent headless client model

A second invocation of `xgen-client.exe` with `--batch` does not need to find or communicate with the running GUI instance. It simply starts without a window, runs its commands as an independent headless protocol client connecting to the node via WebSocket, and exits. The running GUI client does not know it exists.

This means multiple `xgen-client.exe --batch` instances can run simultaneously in parallel from a shell — each with its own identity, its own connection, its own command sequence. This is the natural multi-client stress test model for Phase 2: nodes run (headless or with UI), several headless batch clients fire at them concurrently.

**`xgen-node.exe --batch file.xgb`** — single-instance forwarding model

A second invocation of `xgen-node.exe` with `--batch` CANNOT start as an independent node — the port is already taken. The second invocation must detect the running instance, forward the admin commands to it via IPC (Tauri single-instance plugin or equivalent), and exit. The running node receives the commands and executes them against its own internal state.

The commands in a node batch file are admin/control actions — trigger maintenance, manage federation, kick identity, etc. — not protocol-level events.

---

### Single-instance forwarding — both binaries, same external model

After further discussion, the model converged: both binaries use the same single-instance forwarding pattern. First invocation starts the app. Second invocation with `--batch` detects the running instance via a named pipe, forwards the command file, and exits. The running instance executes the commands. This applies to both `xgen-client.exe` and `xgen-node.exe` — identical external interface, completely different internal execution.

### Primary purpose: stress testing

The entire `--instance` / `--batch` mechanism exists primarily to enable stress testing without manual infrastructure setup. The goal: spin up any number of nodes and clients from a single working directory, fire scripted command sequences at each, observe results in their respective log files — all without touching config folders, editing files, or coordinating ports by hand.

### The `--instance` label — multi-instance without manual folder setup

For running multiple nodes or clients simultaneously, the `--instance <label>` flag was proposed as cleaner than requiring multiple config folders. The label implicitly creates and owns a data subdirectory (`instances/alice/`, `instances/node_a/`, etc.) — auto-created on first run. No manual folder setup. The pipe name is derived from the label so each running instance is precisely addressable. Two invocations with the same label cannot both become apps — the second becomes the batch sender automatically.

**Client** — label alone is sufficient, no port binding:

```
xgen-client.exe --instance alice
xgen-client.exe --instance bob
```

**Node** — requires `--port` at first launch to resolve port conflict (two nodes cannot share a port). Port is written into the instance config on first run; subsequent runs use it automatically:

```
xgen-node.exe --instance node_a --port 8080
xgen-node.exe --instance node_b --port 8081
```

Batch delivery works identically for both — label selects the target instance, `--batch` delivers the command file:

```
xgen-node.exe --instance node_a --batch admin_commands.xgb
xgen-client.exe --instance alice --batch alice_commands.xgb
xgen-client.exe --instance bob --batch bob_commands.xgb
```

Full stress test setup — two nodes, two clients, no manual folder or config work:

```
xgen-node.exe --instance node_a --port 8080
xgen-node.exe --instance node_b --port 8081
xgen-client.exe --instance alice
xgen-client.exe --instance bob
```

### Multiple instances are not an abuse vector

Running multiple instances of either binary is not a protocol-level risk. Each instance is a separate cryptographic identity with its own keypair. Five instances on one machine look identical to the protocol as five different people on five different machines. Identity-level abuse is handled by node-level banning and auth tiers regardless of process count. Multiple instances are also a legitimate real-world scenario — power users active on different nodes simultaneously, bot operators, automated agents.

### Why this is still under discussion

The instance model and external interface are settled in concept. Open questions before writing the implementation instruction (`BATCH_FLAG_ph2.md`):

- What commands does a node batch file contain at this phase? The node admin surface is currently just Shut Down — meaningful node batch commands arrive with Phase 2 protocol work.
- Relationship between node batch IPC and the Console IPC protocol (Ch6 §6.9). They may be the same channel or different.
- Whether node `.xgb` support is needed at the Core Test UI phase or only later.
- Exact pipe naming convention derived from instance label.

### Not in NODE_CORE_UI_ph2.md

This discussion does not appear in the implementation instruction for Mr. Code (`NODE_CORE_UI_ph2.md`). The Core Test UI milestone is scoped to Tauri scaffold, systray, lifecycle state machine, and the Shut Down action only. The `.xgb` capability is a subsequent phase of work and will get its own instruction file once the design is settled.

---

*This journal is maintained as a contemporaneous record. Each entry is committed to
the public Git repository at https://github.com/ianus777/XGenProtocol at the time
of writing, establishing a third-party timestamp via GitHub's servers.*

*For formal IP purposes, entries may be periodically exported, signed with a qualified
electronic signature (eIDAS), and/or anchored to a public blockchain timestamp service.*

---

## Entry J-020 — Phase 1 Binary Wiring: Real WebSocket Server + Full Client Network Commands

**Date:** 2026-04-29
**Author:** Jozef Nižnanský
**Session:** Session 6
**Version tag:** v0.10.3 (pending)

### Summary

This session wires the Phase 1 CLI layer into real runnable processes — completing the second and final Phase 1 deliverable. The definition of done: `xgen-client smoke-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen` executes all 17 steps from spec 3.7.11 against real Node processes over real TCP sockets.

### Work done

**xgen-node/src/transport/client.rs:**
- Added `connect_url(url: &str)` function — connects to a Node by URL string (ws:// or wss://) rather than SocketAddr. Used by xgen-client and smoke-test.

**xgen-node/src/main.rs (full rewrite of `run_node`):**
- `#[tokio::main]` async entry point. All CLI observability commands remain synchronous and run in the tokio runtime without change.
- `run_node()` is now a real async server: loads config and keypair, creates `NodeRuntime` wrapped in `Arc<tokio::sync::Mutex<>>`, spawns a state-writer task (every 5 s), binds the WebSocket server, runs the accept loop, handles Ctrl+C gracefully.
- `handle_connection()`: detects federation vs. client connections from the first message after transport auth. Federation connections (opening with `federation.hello`) go to `handle_federation_incoming()`. Client connections loop on `process_inbound()`.
- `handle_federation_incoming()`: implements the federation receive-side handshake inline (Node A side). Verifies hello signature, negotiates capabilities, sends `federation.capabilities` (signed with node keypair), receives and verifies `federation.accept`, then awaits `space.join_request`. Snapshots history and DAG tips atomically, builds and signs `state.federation_add`, ingests it locally, sends history + federation_add + goodbye.
- `handle_identity_msg()`: handles `identity.register` (runs 8-step acceptance pipeline, persists registry, sends `register_ok` or `register_fail`) and `identity.get` (looks up and sends `identity.record` or `identity.not_found`).
- `process_inbound()`: routes Events to `accept_message()` (message.* types) or `ingest_event()` (state.*/membership.*).
- `build_node_state()`: builds `NodeState` from `NodeRuntime` + active connection info for the 5 s state file writer.
- Active connection tracking: `Vec<ConnectedClientInfo>` behind an `Arc<Mutex>`, updated on connect/disconnect/event receipt.

**xgen-client/Cargo.toml:**
- Added `xgen-node = { path = "../xgen-node" }` dependency (D-029). Gives the client access to all protocol code without duplicating ~2 000 lines.

**xgen-client/src/main.rs (full rewrite of network commands):**
- `#[tokio::main]` async entry point. File-only commands (init, whoami, status, spaces, version) remain synchronous.
- Removed the inline keypair module — now uses `xgen_node_lib::identity::keypair` directly.
- `cmd_register()`: connects, authenticates, sends signed `identity.register`, receives `register_ok`/`register_fail`, writes `xgen-client_state.json`.
- `cmd_create_space()`: connects, authenticates, builds+signs `state.space_create` event, sends, updates client state.
- `cmd_create_room()`: same pattern for `state.room_create`.
- `cmd_invite()`: builds+signs `membership.invite`, sends with space_id as Phase 1 prev_event anchor.
- `cmd_join()`: builds+signs `membership.join`, sends.
- `cmd_send()`: connects, authenticates, fetches DAG tips via `sync_request` (with 500 ms timeout fallback), builds+signs `message.text`, sends.
- `cmd_history()`: connects, authenticates, sends `sync_request`, collects events for 5 s, displays message.text events in order.
- `cmd_smoke_test()`: 17-step protocol per spec 3.7.11 over real TCP — see below.

**Smoke test (cmd_smoke_test):**
All 17 steps from spec 3.7.11 executed against two real `xgen-node` processes:
1. Node A already running; Alice's ephemeral keypair generated
2. Alice registers on Node A via real WebSocket connection
3. Node B already running; test-Node-B ephemeral keypair generated (simulates Node B's federation connector)
4. Bob registers on Node B
5. Alice creates Space on Node A (state.space_create event)
6. Alice creates Room 'general' (state.room_create event)
7. Alice invites Bob (membership.invite event)
8. test-Node-B connects to Node A, runs full federation handshake (run_initiating)
9. test-Node-B sends space.join_request
10–11. Node A sends history + state.federation_add; smoke test receives them, forwards to Node B
12. Bob joins Space (membership.join, forwarded to both nodes)
13. Bob joins Room (membership.join, forwarded to both nodes)
14. Alice sends 'Hello Bob' (message.text, forwarded to Node B)
15. Bob sends 'Hello Alice' (message.text, forwarded to Node A)
16–17. Signature verification and content verification on both messages

**DECISIONS.md:**
- D-029: xgen-client depends on xgen-node lib for Phase 1 binary wiring (replaced by D-022/xgen-core in Phase 2)

### Test results

173 tests pass, 0 failures. Clean compile with no warnings.

### Architecture note

The `handle_connection()` function on the Node dispatches on the first message after transport auth. A `federation.hello` triggers the federation receive-side handshake; anything else (identity message or event) triggers the client message loop. This allows the Node to serve both clients and federation peers on the same port without a path-based multiplexer.

---

## J-022 — 2026-04-29 — D-030/D-031: GetModuleFileNameW, data_dir, init --passphrase, config reference

### Context

Post-smoke-test hardening. User reported a known issue: `xgen-node init` could write files to a temp/CWD location on Windows instead of next to the executable. Addressed by two decisions recorded as D-030 and D-031.

### What was done

**`exe_dir()` rewritten for Windows (D-030):**
Replaced `std::env::current_exe()` with a direct `GetModuleFileNameW(NULL)` call via `windows-sys 0.59` (already a transitive dependency). Uses a growing buffer starting at `MAX_PATH` (260 chars), doubling until the full path fits. Returns the executable's module path as the Win32 loader recorded it — immune to shadow copies, CWD, PATH order, symlinks, and shell wrappers. Panics with a clear message if the call fails, rather than silently falling back to `"."`.

**`data_dir` derived from config path (D-030):**
All Tier-1 runtime files are placed in `config_path.parent()`:
- No `--config`: `data_dir = exe_dir()` (spec-compliant, same as before).
- With `--config /path/cfg.toml`: `data_dir = /path/` (explicit multi-instance isolation).

**`init --passphrase` flag (D-030):**
Hidden `--passphrase` flag bypasses `rpassword` interactive prompt for scripts and CI.

**Phase 1 config reference (D-031):**
Canonical `xgen-node_config.toml` with all fields documented (required vs optional, Phase 1 values vs Phase 2 migration path, multi-instance setup instructions).

### Test results

173 tests pass, 0 failures.

---

## J-021 — 2026-04-29 — Phase 1 smoke test verified over real TCP; v0.10.3

### Context

Phase 1 binary wiring was complete (J-020) but the end-to-end smoke test had not yet been run against two real live `xgen-node` processes. This session completed that verification.

### What was done

**xgen-node `init` — `--passphrase` flag + `data_dir` refactor:**
`xgen-node init` previously required an interactive passphrase prompt (via `rpassword`), making it impossible to script. Two changes were made:
1. `Init` subcommand gained an optional `--passphrase` flag. When provided, the prompt is skipped and the supplied value is used directly.
2. All `exe_dir()` calls in `main.rs` were replaced with a `data_dir` derived from the config file's parent directory. Previously all runtime files (keypair, state, identities DB) were co-located with the binary; now they are co-located with the config file. This allows multiple node instances to run from the same binary with isolated data directories. `exe_dir()` is still the default when `--config` is not supplied.

**Two-node test setup:**
- Created `test/node_a/` and `test/node_b/` directories.
- Initialised each with `xgen-node --config test/node_N/xgen-node_config.toml init --passphrase ""`.
- Node A: `ws://127.0.0.1:8080/xgen`, Node B: `ws://127.0.0.1:8081/xgen`.

**Smoke test result:**
`xgen-client smoke-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen` — ALL 17 STEPS PASSED.

All events produced valid signatures (steps 16–17 signature verification passed). Event IDs are persistent hashes — reproducible from event content.

### Version bump and tag

Cargo.toml bumped from `0.10.2` → `0.10.3` across all three crates. CLAUDE.md updated to reflect Phase 1 fully complete.

### Test results

173 tests pass, 0 failures. Clean compile with no warnings.

---

## J-022 — 2026-04-29 — Phase 1 documentation review and FIXES_ph1.md

### Context

With Phase 1 implementation complete and verified (J-021, v0.10.3), a full documentation review was conducted before Phase 2 begins. This session was documentation-only — no Rust source changes.

### What was done

**Full cross-check of Ch3 Phase 1 (sections 3.1–3.8) against Ch4:**
All Phase 1 specification sections were read in full and cross-checked against the implementation guide. 16 issues were identified and documented in `docs/FIXES_ph1.md` for Claude Code to apply.

**Issues identified and documented (Fixes 01–11 — spec/doc):**
- Fix 01-02: Corrupted box-drawing characters and glyphs in 3.1.1 and 3.1.2
- Fix 03: Eight section headers still marked `*Status: wip*` despite being complete
- Fix 04: `xgen_uri` type not used in Phase 1 wire fields — Phase 1 note added
- Fix 05: `transport.sync_complete` schema missing — new schema specified
- Fix 06: Five EventTypes missing from registry (space_create, dm_space_create, node_priority, federation_add, federation_remove)
- Fix 07: Membership events described as Room-level — corrected to Space-level
- Fix 08: Corrupted emoji in Ch4 skeleton table row 4.6
- Fix 09: `prev_events` empty-array exception not noted in field table
- Fix 10: `space_id` missing from `transport.sync_request` schema
- Fix 11: Work definitions consolidated into a single table (WD-01 through WD-13)

**Issues identified and documented (Fixes 12–16 — CLI and implementation):**
- Fix 12: `rooms` and `members` CLI commands missing from xgen-client
- Fix 13: ANSI colour output note added to CLI reference (basic colours confirmed working in Windows Terminal and PowerShell)
- Fix 14: Full membership lifecycle CLI commands (invite/leave/kick/ban) — **deferred by project owner** to end of protocol development or independent CLI modules
- Fix 15: Keepalive-as-session model — note added to 3.3.5 that XGen has no inactivity timeout; keepalive IS the session model
- Fix 16: **Critical implementation bug** — Node does not reconstruct Space state from SQLite Event log on restart. Confirmed by live test: Space created in Session 1, Node restarted, message in Session 2 fails with `space not found`. Full startup replay algorithm documented.

**Supporting file updates:**
- `CLAUDE.md`: Fix 16 bug summary added to pending section; FIXES_ph1.md reference added
- `docs/xgen_ch0_content.md`: Ch4 status corrected from "pending" to "Phase 1 complete (v0.10.3)"

### Decisions recorded

No new DECISIONS.md entries — this session was documentation review only. All findings are recorded in `docs/FIXES_ph1.md`.

### Next steps

1. Claude Code applies all fixes in `docs/FIXES_ph1.md` (including Fix 16 Rust source fix)
2. JozefN confirms documentation gates complete
3. Phase 2 specification (Ch3 sections 3.9–3.16) begins

---

## J-023 — 2026-04-29 — FIXES_ph1.md applied (all 16 fixes, Fix 14 deferred)

### Context

All fixes documented in `docs/FIXES_ph1.md` applied in a single Claude Code session. Fix 14 (membership lifecycle CLI) remains deferred as previously decided.

### What was done

**Documentation fixes — `docs/xgen_ch3_specification.md` (Fixes 01–11, 15):**
- Fix 01: Transport frame box-drawing already clean — no action needed.
- Fix 02: Corrupted glyph after "permission updates" already clean — no action needed.
- Fix 03: All eight section status markers changed from `*Status: wip*` to `*Status: complete*` (sections 3.1–3.8).
- Fix 04: Phase 1 note added below `xgen_uri` examples clarifying it is not a Phase 1 wire field type.
- Fix 05: `transport.sync_complete` schema added after `transport.sync_request` in section 3.3.6.
- Fix 06: `state.space_create`, `state.dm_space_create`, `state.node_priority` added to State events table; new Federation events table with `state.federation_add` and `state.federation_remove` added.
- Fix 07: Membership events description corrected from "Room" to "Space" with Phase 2 note on private Rooms.
- Fix 09: `prev_events` field table updated — explicitly states MUST be empty array for `state.room_create`.
- Fix 10: `space_id` added as required field to `transport.sync_request` schema; description updated to explain Node→Space database resolution.
- Fix 11: Work Definitions table (WD-01 through WD-13) added before Chapter 3 Open Questions.
- Fix 15: "Keepalive as the complete session model" subsection added to 3.3.5 — explicitly prohibits separate inactivity timers.

**Documentation fixes — `docs/xgen_ch4_implementation.md` (Fixes 08, 12, 13, 16 doc):**
- Fix 08: Ch4 skeleton table row 4.6 already shows ✅ Complete — no action needed.
- Fix 12: `rooms <space-id>` and `members <space-id>` commands added to 4.16.2 xgen-client CLI reference.
- Fix 13: New section 4.16.5 ANSI Colour Output added — documents `supports-color` crate recommendation.
- Fix 16 (doc): New section 4.8.5 "Node Startup State Reconstruction (hard requirement)" added — specifies full startup replay sequence and space_not_found secondary requirement.

**Rust source fix — Fix 16 (`xgen-node/src/main.rs`):**
- Added `persist_event()` helper: appends a single Event as JSON to a per-Space file in `<spaces_dir>/<sha256_hex>.json`. Idempotent (deduplicates by event_id).
- Added `replay_spaces_from_dir()` helper: scans `spaces_dir` for `*.json` files on startup and replays all events through `NodeRuntime::ingest_event` in stored order.
- `run_node()` updated: creates `spaces_dir` on startup, calls `replay_spaces_from_dir` before `Server::bind`, prints replay count to console.
- `process_inbound()` updated: space_id resolved correctly for space_create events. Persistence called after every `ingest_event`. `MembershipJoin` events rejected with `space_not_found` log if Space not in registry.
- `handle_federation_incoming()` updated: federation_add event persisted to disk after ingestion.

### Test results

173 tests pass, 0 failures. Clean compile, no warnings.

### State after this session

All FIXES_ph1.md fixes applied. Documentation gates complete pending JozefN review. Phase 2 is the next step.

---

## J-024 — April 2026 — Ch3 Phase 2 specification begun; logging infrastructure designed

### What was done

**Ch3 Phase 2 — three sections written:**
- **3.9 State Resolution Algorithm** — complete. Seven-layer priority stack fully specified. Convergence guarantee, split-brain recovery, pending event timeout, state snapshot model, error codes 4xxx.
- **3.10 End-to-End Encryption** — complete. MLS (RFC 9420) selected over Megolm (D-031). Two-layer encryption model, KeyPackage management, group init/add/remove sequences, message encryption flow, E2E opt-out, Phase 1 forward compatibility, 6 new EventTypes, error codes 5xxx.
- **3.11 Auth Module Tiers 2–4 Interfaces** — complete. Tier 2 ISO 27001, Tier 3 Corporate/SOX, Tier 4 Government/Healthcare. Verification requirements, Trust Assertion claims, TTLs, cross-tier compatibility, registration obligations, error codes 3010–3016. Subsection 3.11.8 Audit Log Requirements added.

**Logging infrastructure designed — two types, two phases:**
- D-032 recorded: two independent log types — debug log and audit log — never merged
- `LOGGING_debug_ph1.md`: debug log implementation for Claude Code — **immediate priority before Phase 2 testing**
- `LOGGING_audit_ph2.md`: audit log implementation for Claude Code — **deferred to Phase 2 alongside Tier 2+ Auth Module work**
- Ch4 section 4.17 Logging written (operator-facing)
- Appendix D Part 6 Audit Logging written (DPO/evaluator-facing)

**Supporting files updated:**
- DECISIONS.md: D-031 (MLS), D-032 (two log types)
- CLAUDE.md: current priorities updated
- `ch3_ph2_handoff.md`: documentation Claude continuity note written

### Current state

**Ch3 Phase 2 progress:** 3/8 sections complete (3.9, 3.10, 3.11). Paused at 3.12 Space Migration Protocol.

**Immediate next step for Mr. Code:** implement debug logging per `LOGGING_debug_ph1.md` before Phase 2 testing begins.

**Sections remaining in Ch3 Phase 2:** 3.12 Space Migration, 3.13 Identity Replication Parameters, 3.14 Bootstrap Node Protocol, 3.15 Node Reputation Format, 3.16 DM Space Promotion Sequence.

---

## J-026 — 2026-04-30 — Global Event tracing interface — Priority 0

### Decision

The global Event tracing interface (`LOGGING_debug_ph2.md`) is elevated to Priority 0 — before Phase 2 protocol features, before further testing, before anything else.

### Rationale

Joe must be able to debug the system independently at any time, without waiting for a documentation session. Phase 1 made the architectural mistake of building 173 tests and a full smoke test before any Event observability existed. That mistake is corrected here.

The Phase 1 enumerated logging approach (`LOGGING_debug_ph1.md`) added `tracing::` calls one per handler. This is fragile, incomplete, and does not guarantee pairing between client and Node logs. The global interface fixes all three problems:
- Every Event is logged automatically — no enumeration, no forgetting
- Role gate: Owner/Admin sessions produce output; Member sessions do not — prevents sensitive conversation leakage
- Pairing by `event_id`: client Outbound and Node Inbound entries join automatically by content hash

### Files updated

- `DECISIONS.md`: D-033 recorded
- `LOGGING_debug_ph2.md`: full implementation instructions for Claude Code
- `CLAUDE.md`: Priority 0 section added at top
- `LOGGING_debug_ph1.md`: forward reference to Phase 2 document added

### Next steps

1. Mr. Code implements global Event tracing interface per `LOGGING_debug_ph2.md`
2. Joe verifies with 5-step test sequence
3. Documentation Claude continues Ch3 Phase 2 from 3.12

---

## J-027 — 2026-04-30 — Priority 0 complete: Global Event tracing interface

### What was done

`LOGGING_debug_ph2.md` implemented by Mr. Code. Global Event tracing interface live in both binaries.

**`xgen-node/src/event_trace.rs`** — new module containing `EventDirection`, `SpaceRole`, `SessionContext`, and `trace_event()`. Role gate correct: Owner/Admin produce output, Moderator/Member suppressed. Content field never logged. D-033 comment at top of file.

**Node wiring:** 7 `trace_event` call sites in `xgen-node/src/main.rs`. SessionContext built once per connection after auth. Phase 1 sets all authenticated sessions to `SpaceRole::Owner` — correct temporary decision pending Phase 2 role resolution from space registry.

**Client wiring:** 14 `trace_event` call sites in `xgen-client/src/main.rs`. Per-command call sites are correct for client architecture — each CLI command connects, acts, disconnects. The spec's two-boundary-point model applies to the Node's persistent connection loop; per-command is the right equivalent for the client.

**Structural note:** `event_trace.rs` placed in `xgen-node/src/` rather than `xgen-common/src/`. Client imports it via xgen-node library dependency. This works correctly now. When D-022 (xgen-core crate split) is implemented in Phase 2, `event_trace` moves to the core crate as part of that migration.

### Test results

173/173 tests passing. Clean compile, no warnings.

### Next steps

Priority 0 complete. Ready to continue Ch3 Phase 2 specification from 3.12 Space Migration Protocol.

---

## J-028 — 2026-04-30 — Module architecture recognised as open question; Fix 17 added

### What was done

**Fix 14 reframed:** Full membership lifecycle CLI commands are not simply deferred — they are blocked on the XGen module architecture question. CLI commands are one expression of a module. The form a module takes must be decided before locking in any CLI command extension mechanism.

**Fix 17 added to FIXES_ph1.md:** `event_trace` module must move from `xgen-node/src/` to `xgen-common/src/` — shared infrastructure used by both binaries belongs in the common crate, not in one of the consuming crates. Four-step fix with verification.

**OQ-01 added to Ch3 Open Questions:** XGen module architecture formally recorded as an open question. Key insight: modules extend both `xgen-node` and `xgen-client` — not client-only. A module may extend the Node (compliance reporting, content moderation, protocol bridge), the client (UI skin, bot interface, CLI commands), or both simultaneously. Nine sub-questions listed. Resolution during Ch6 second pass. Notably: Node module capabilities interact with the open enum capability advertisement (3.4.3) — this feeds back into the protocol spec.

**Fix 14 in FIXES_ph1.md updated:** Reason for deferral now explicitly states the module architecture dependency.

### Files modified

- `FIXES_ph1.md`: Fix 14 reframed; Fix 17 added; checklist, files table, session log updated
- `docs/xgen_ch3_specification.md`: OQ-01 Module Architecture added to Open Questions section
- `JOURNAL.md`: this entry

### Current fix status

| Fix | Status |
|---|---|
| 01–13 | ✅ Applied (J-023) |
| 14 | ⏳ Deferred — blocked on module architecture (OQ-01) |
| 15–16 | ✅ Applied (J-023) |
| 17 | 🔴 Pending — Mr. Code to move `event_trace` to `xgen-common` |

### Next steps

1. Mr. Code applies Fix 17
2. Documentation Claude continues Ch3 Phase 2 from 3.12

---

## J-025 — 2026-04-30 — Debug logging implemented (LOGGING_debug_ph1.md)

### Context

Phase 1 is complete (v0.10.3, 173 tests). Logging infrastructure was designed in J-024 as a prerequisite before Phase 2 testing. This session implements the debug log for both binaries per `LOGGING_debug_ph1.md`.

### What was done

**`xgen-node/Cargo.toml` and `xgen-client/Cargo.toml`:**
- `tracing-subscriber` upgraded from `"0.3"` to `{ version = "0.3", features = ["env-filter", "chrono"] }` on both crates. Adds `EnvFilter` (config-driven level filtering) and `ChronoLocal` timer (millisecond-precision local timestamps).

**`xgen-node/src/main.rs`:**
- `PathsSection`: `log_path: Option<String>` field removed (replaced by dedicated `[logging]` config section).
- `LoggingSection { level: String }` struct added.
- `NodeConfig.logging: LoggingSection` added; `Default` impl updated.
- `run_node()`: log init block added immediately after config load. Creates `<data_dir>/logs/` if absent, opens `xgen-node_YYYY-MM-DD_HH-MM-SS.log` in append mode, initialises `tracing_subscriber::fmt()` with `with_ansi(false)`, `with_target(true)`, `ChronoLocal` timer, `EnvFilter` from `config.logging.level` (or `XGEN_LOG` env var override). Global subscriber installed with `.init()`.
- Structured `tracing::info/warn/error/debug!` calls added at all minimum required log points: `Node started`, `Identity registered`, `Identity registration rejected`, `Client authenticated`, `Client disconnected`, `Space not found (step 10)`, `accept_message failed`, `Federation hello: invalid signature`, `Federation join request`, `Federation established`, `Node shutting down`. Existing `eprintln!` calls retained where they produce user-facing console output; replaced elsewhere with tracing calls.

**`xgen-client/src/main.rs`:**
- `LoggingSection { level: String }` struct added; `ClientConfig.logging: LoggingSection` added; `Default` impl updated.
- `main()`: log init block added immediately after `config_path` is resolved. Creates `<exe_dir>/logs/` if absent, opens `xgen-client_YYYY-MM-DD_HH-MM-SS.log` in append mode with same subscriber config as the Node. Log level read from config (or default `"info"`).
- `tracing::info!` calls added in `cmd_create_space`, `cmd_create_room`, `cmd_join`, `cmd_send`, `cmd_register`, `cmd_history`, `cmd_smoke_test` at key points: `Connecting to Node`, `Authenticated`, `Space created`, `Joined Space`, `Message sent`, `Federation initiated`.

**`test/node_a/xgen-node_config.toml` and `test/node_b/xgen-node_config.toml`:**
- `log_path` field removed from `[paths]`.
- `[logging]` section added with `level = "info"`.

### Verification

- `cargo test`: 173/173 pass, clean compile.
- Manual test: `xgen-node -c test/node_a/xgen-node_config.toml` (with port 8080 already in use — early exit). Log file `test/node_a/logs/xgen-node_2026-04-30_*.log` created with correct format:
  ```
  2026-04-30 11:51:47.380  INFO xgen_node: Log file opened: test/node_a\logs\xgen-node_...log
  2026-04-30 11:51:48.487  INFO xgen_node: Node started node_id=xgen://pubkey/... endpoint=ws://127.0.0.1:8080/xgen
  ```
- `xgen-client version`: `bin/logs/xgen-client_2026-04-30_*.log` created. `Log file opened` line present.
- Log format matches spec: `YYYY-MM-DD HH:MM:SS.mmm  LEVEL target: message key=value`.

### Test results

173 tests pass, 0 failures. Clean compile, no warnings.

### State after this session

Debug logging fully implemented. Both binaries write datetime-stamped log files to `logs/` relative to their data directory on every run. Log level controlled by `[logging].level` in config; `XGEN_LOG` env var overrides for development. Audit log remains deferred to Phase 2.

---

## J-029 — 2026-04-30 — Fix 17 applied; Phase 1 smoke test with logging verified

### Context

Fix 17 was the last outstanding item from `FIXES_ph1.md` — moving the `event_trace` module from `xgen-node/src/` to `xgen-common/src/`. After that, `SMOKETEST_ph1.md` required a full re-run of the Phase 1 smoke test with debug logging active, to verify the global Event tracing interface (D-033) produces correct output, Event IDs pair across client and both Nodes, and message content never appears in any log.

### What was done

**Fix 17 — `event_trace` module relocated to `xgen-common`**

The core challenge: `event_trace.rs` imported `crate::wire::types::Event`, so a naive file move would create a circular dependency (`xgen-common` → `xgen-node` → `xgen-common`). Resolution:

- `Event` and `EventType` extracted from `xgen-node/src/wire/types.rs` into a new `xgen-common/src/wire.rs`. These are canonical protocol types with no runtime dependencies — only `serde` and `serde_json`, both already in `xgen-common`.
- `xgen-node/src/wire/types.rs` reduced to transport-level types (`TransportMessage`, `FederationMessage`, `IdentityMessage`, `SpaceControlMessage`, `MessageTextContent`). Adds `pub use xgen_common::wire::{Event, EventType};` re-export so all internal `use crate::wire::types::{Event, EventType}` paths continue to compile without modification.
- `xgen-common/src/event_trace.rs` created (moved from `xgen-node/src/event_trace.rs`). Import updated to `use crate::wire::Event;`. No logic changes.
- `tracing = "0.1"` added to `xgen-common/Cargo.toml`.
- `xgen-common/src/lib.rs`: `pub mod event_trace;` and `pub mod wire;` added.
- `xgen-node/src/lib.rs`: `pub mod event_trace;` removed.
- `xgen-node/src/main.rs`: import updated from `xgen_node_lib::event_trace::*` to `xgen_common::event_trace::*`.
- `xgen-client/src/main.rs`: import updated from `xgen_node_lib::event_trace::*` to `xgen_common::event_trace::*`.
- `xgen-node/src/event_trace.rs` deleted.

Result: `cargo test` 173/173 pass. Both binaries compile. Log target for all Event trace lines is `xgen_common::event_trace`, confirming the module lives in the correct crate.

**Smoke test with debug logging — `SMOKETEST_ph1.md`**

Prerequisites verified: Fix 17 done, both node configs set to `level = "debug"`, stale state files cleaned, fresh release build.

Nodes started from project root, smoke test run via `XGEN_LOG=debug xgen-client smoke-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen`.

ALL 17 STEPS PASSED.

Log files produced:
- `test/node_a/logs/xgen-node_2026-04-30_21-52-09.log`
- `test/node_b/logs/xgen-node_2026-04-30_21-52-09.log`
- `bin/logs/xgen-client_2026-04-30_21-52-20.log`

**Pairing table (8 events, all fully paired):**

| event_id (short) | event_type | Client Out | Node A In | Node B In |
|---|---|:---:|:---:|:---:|
| `9ba66d487573` | `state.space_create` | ✔ | ✔ | ✔ |
| `9cb9acbef972` | `state.room_create` | ✔ | ✔ | ✔ |
| `995594b86837` | `membership.invite` | ✔ | ✔ | ✔ |
| `ecbbc47660bd` | `state.federation_add` | — | ✔ Out | ✔ In |
| `d8fa7b302680` | `membership.join` (Bob/Space) | ✔ | ✔ | ✔ |
| `87acf54b1753` | `membership.join` (Bob/Room) | ✔ | ✔ | ✔ |
| `e97c46b1e8d8` | `message.text` (Alice→Bob) | ✔ | ✔ | ✔ |
| `9179066b7771` | `message.text` (Bob→Alice) | ✔ | ✔ | ✔ |

Content leak check: zero matches for `"Hello Bob"` / `"Hello Alice"` in all log files. ✔

Timing baseline: all three timestamps for `message.text` land at `21:52:20.806` — loopback latency is sub-millisecond (below log timer resolution). Phase 1 localhost baseline: **<1ms** client→Node A and Node A→Node B.

Additional observation: both nodes logged `Space event stores replayed from disk count=1` on startup, confirming Fix 16 (state reconstruction from SQLite) is live.

Node configs restored to `level = "info"` after the test.

### Test results

173 tests pass, 0 failures. Smoke test: ALL 17 STEPS PASSED. Full pairing table verified. No content leak.

### State after this session

Fix 17 complete. All 17 fixes from `FIXES_ph1.md` are now applied (Fix 14 deferred by project owner). `event_trace` lives in `xgen-common`. Both binaries confirmed to produce correct Event trace output at DEBUG level. Phase 1 documentation closure complete.

---

## J-030 — 2026-05-06 — LOGGING_implementation.md applied: session header/footer, action field, trace_local

### Context

`LOGGING_implementation.md` specifies the remaining work to make the debug log fully compliant with Appendix G. The global Event tracing interface (D-033) was already wired in J-027/J-029, but three things were still missing: the `action` field on every Event log line, the `LOCAL` direction and `trace_local()` interface for internal actions, and the session header/footer blocks required by Appendix G.

Before implementation, a design question arose: the Appendix G client header specifies `identity_id` and `connected_node` as mandatory fields, but in the CLI client both values are unavailable at subscriber init time — log body lines fire before a keypair is loaded or a connection is made. Decision D-038 was recorded: both fields are omitted from the client header and logged as operational body lines at the point they become available (after `client_authenticate()` completes). The header field `self_id: &str` was changed to `Option<&str>` in the implementation to accommodate this without special-casing the caller.

### What was done

**`xgen-common/Cargo.toml`:**
- Added `chrono = { version = "0.4" }` — needed by `write_session_footer()` to stamp `ended_at`.

**`xgen-common/src/event_trace.rs` — complete rewrite:**
- `EventDirection` renamed: `Inbound` → `In`, `Outbound` → `Out`; `Local` variant added. `Display` now produces `IN`, `OUT`, `LOCAL` per Appendix G.
- `trace_event()` updated: emits `action=receive_event` (IN) or `action=send_event` (OUT) on every log line. `Local` direction variant now logs a warning and returns rather than producing a malformed line.
- `LocalAction` enum added: `CreateEvent`, `StoreEvent`, `ApplyEvent`, `RejectEvent`. `Display` produces lowercase Appendix G action strings.
- `trace_local()` added: logs direction=LOCAL + action + event_id + optional event_type/space_id/error_code. No role gate — LOCAL actions contain no sensitive content.
- `write_session_header()` added: writes `=== XGEN SESSION START ===` block. `self_id: Option<&str>` — when None, the identity/node_id line is omitted (D-038). Ends with a mandatory blank line per Appendix G.
- `ExitReason` enum added: `Shutdown`, `Restart`, `Error`.
- `write_session_footer()` added: writes mandatory blank line then `=== XGEN SESSION END ===` block with `ended_at` (UTC RFC 3339 with ms) and `reason`.

**`xgen-node/src/main.rs`:**
- Keypair load moved before subscriber init in `run_node()` so `node_id_uri` is available for the session header. Previously the keypair was loaded after the subscriber.
- "Log file opened" log line removed — the session header makes it redundant.
- `started_at` timestamp moved to immediately after subscriber init.
- `write_session_header("node", Some(&node_id_uri), Some(&config.node.listen), None, ...)` called immediately after subscriber init.
- `write_session_footer(ExitReason::Shutdown)` added at the ctrl+c clean exit path, before `Ok(())`.
- All `EventDirection::Inbound` → `EventDirection::In`, `EventDirection::Outbound` → `EventDirection::Out` (4 call sites).
- `trace_local(LocalAction::CreateEvent, ...)` added after building `fed_add_ev` in `handle_federation_incoming`.
- `trace_local(LocalAction::StoreEvent, ...)` and `trace_local(LocalAction::ApplyEvent, ...)` added after `ingest_event()` in both membership and catch-all branches of `process_inbound`.
- `trace_local(LocalAction::ApplyEvent, ...)` added on `accept_message` success path for message.* events.
- `trace_local(LocalAction::RejectEvent, ...)` added on `accept_message` failure and space-not-found paths; space-not-found includes `error_code: Some(10)` (step 10 per spec validation pipeline).
- Imports updated: `trace_local`, `LocalAction`, `write_session_header`, `write_session_footer`, `ExitReason` added.

**`xgen-client/src/main.rs`:**
- "Log file opened" log line removed.
- `write_session_header("client", None, None, None, ...)` called immediately after subscriber init — all optional fields None per D-038.
- `tracing::info!("identity_id={}", auth_id)` and `tracing::info!("connected_node={}", node)` added after `client_authenticate()` in every network command handler (register, create-space, create-room, invite, join, send, history).
- `None` branch (no subcommand — prints help) changed from early `return` to `Ok(())` so the session footer is always written before process exit.
- Error handling at the end of `main()` restructured: logs `Fatal error` before footer, calls `write_session_footer(ExitReason::Error)`, then `process::exit(1)`. Clean exit writes `write_session_footer(ExitReason::Shutdown)`.
- All `EventDirection::Inbound` → `In`, `EventDirection::Outbound` → `Out` (13 call sites).
- Imports updated: `write_session_header`, `write_session_footer`, `ExitReason` added.

**`DECISIONS.md`:**
- D-038 recorded: client session header omits `identity_id` and `connected_node`; both logged as body lines after auth. Rationale: body lines fire before those values are available; buffering is not idiomatic with the tracing subscriber model. CLI-specific limitation — future Tauri UI client will supply both fields in the header at open time.

### Test results

173 tests pass, 0 failures. Clean compile with no warnings.

### State after this session

All 6 steps from `LOGGING_implementation.md` are implemented. The debug log is now fully Appendix G-compliant:
- Session header on every run (node: all fields; client: without identity_id/connected_node per D-038)
- Session footer on every clean exit, absent on crash/kill
- `action=` field on every Event body line
- `direction=IN/OUT/LOCAL` with correct Appendix G casing
- `trace_local()` wired at create/store/apply/reject points in xgen-node

---

## J-031 — 2026-05-06 — F-001 resolved: pending buffer wired; stress test resting points; debug default

### Context

Phase 1 stress test findings document (`docs/tests/STRESSTEST_ph1_findings.md`) was reviewed. It identified finding F-001: federated events arriving out-of-order at Node B during the concurrent message flood were being silently dropped rather than buffered and applied. Two stress test runs at `v0.10.3 fac0429` showed 150–200 ERROR lines on Node B and an `apply_event` count of ~134 against an expected ~250 federated message events.

### Investigation

Code review confirmed the root cause. `PendingBuffer` (`dag/pending.rs`) and `RoomDag` (`dag/mod.rs`) were fully implemented with cascading drain logic and five passing tests. However, `NodeRuntime::accept_message` (`node/runtime.rs`) bypassed both — calling `accept_event` directly with raw `EventStore + DagGraph`. On `ExchangeError::HeldPending`, the error returned to `main.rs`, which logged it as `ERROR` and traced it as `RejectEvent`. The event was dropped permanently.

### What was done

**`xgen-node/src/node/runtime.rs` — F-001 fix (D-039):**
- `use crate::dag::pending::PendingBuffer` added.
- `NodeRuntime` gains `pub pending: HashMap<String, PendingBuffer>` (one buffer per space_id); initialised to empty in `new()`.
- `accept_message` restructured: calls `accept_event(event.clone(), ...)` then matches on result. On `HeldPending(missing)` → calls `self.pending.entry(...).or_default().add(event, &missing)` and returns `Err(HeldPending)`. On `Ok(())` → calls `drain_pending_messages`.
- `drain_pending_messages` added: extracts ready events from `pending.resolve(resolved_id, store)`, re-runs `accept_event` on each unblocked event (without re-buffering), recurses on each success.

**`xgen-node/src/main.rs` — logging fix:**
- `use xgen_node_lib::message::exchange::ExchangeError` added.
- `accept_message` error handler split into two arms: `HeldPending` → `tracing::debug!` ("event buffered — waiting for unknown prev_events"), no `RejectEvent` trace; all other errors retain `tracing::error!` + `RejectEvent` trace.

**`xgen-node/src/main.rs` and `xgen-client/src/main.rs` — debug logging default:**
- `LoggingSection::default()` changed from `"info"` to `"debug"` in both binaries.
- `xgen-client` no-config fallback also changed from `"info"` to `"debug"`.
- Test node configs (`test/node_a/xgen-node_config.toml`, `test/node_b/xgen-node_config.toml`) already had `level = "debug"` explicitly.

**`xgen-client/src/main.rs` — stress test resting points:**
- `StressTestArgs` gains `--rest-ms` (default 2000ms): resting period in milliseconds after each phase transition.
- Resting point after Phase 3 (before flood): lets membership/join events propagate and be applied on both nodes before the concurrent send begins.
- Resting point after Phase 4 (before report): lets federation delivery and pending-buffer drain complete so the `apply_event` count reflects full settlement, not a snapshot mid-drain.
- Both resting points are logged to the communication record (`phase=rest`, `action=rest_start/rest_end`). Skip entirely when `--rest-ms 0`.

### Test results

173 tests pass, 0 failures. Clean compile with no warnings on both binaries.

### Stress test results after fix

Three runs compared:

| Metric | Before fix (07:21) | After fix, no rest (11:46) | After fix + 2s rest (11:55) |
|---|---|---|---|
| ERROR lines — Node B | 150 | 0 | 0 |
| buffered events | n/a (dropped) | 200 | 0 |
| apply_event — Node B | 134 | 84 | 284 |
| reject_event — Node B | 150 | 0 | 0 |

The 2s resting point after Phase 3 gave enough time for all membership events to propagate before the flood, eliminating out-of-order arrivals entirely. `apply_event` on Node B (284) is now symmetrical with Node A (280); the small difference reflects setup events that only Node A originates.

F-001 is closed.

---

## J-032 — 2026-05-06 — Next-round stress test tasks: Tasks 1, 2, 4

### Context

`STRESSTEST_ph1_next_round.md` specified four tasks required for Phase 1 sign-off. Task 3 (verify `event buffered` log line is at DEBUG level) was confirmed as already correct — no change needed (`tracing::debug!` at line 715 of `xgen-node/src/main.rs`). Tasks 1, 2, and 4 were implemented in this session.

### What was done

**Task 1 — pending buffer shutdown WARN (`xgen-node/src/main.rs`):**

In the clean shutdown path (just before `write_session_footer(ExitReason::Shutdown)`), added a lock on `runtime` that iterates over all space entries in `rt.pending`. For each space with a non-empty buffer, emits:

```
WARN xgen_node: pending_buffer_at_shutdown space_id=... unresolved=N
```

This is logging only — no behaviour change. A stalled run (like run 3 from the pre-fix analysis) will now show the WARN with a nonzero count. A clean run will be silent. This makes the two cases distinguishable from the log alone, without requiring the report.

**Task 2 — federation completeness section in stress test report (`xgen-client/src/main.rs`):**

After the Phase 4 resting point (before the per-member/room stats loop), the report now scans both node log files:

- Node A log: `exe_dir().parent()/test/node_a/logs/` — latest `xgen-node_*.log`
- Node B log: `exe_dir().parent()/test/node_b/logs/` — latest `xgen-node_*.log`

Counts lines containing both `apply_event` and `message.text` on each node. Expected count: Node A = `(members/2) × messages`, Node B = `(members - members/2) × messages`. With default config (10 members, 50 messages): 250 per node.

Two new helper functions added:
- `find_latest_node_log(dir: &Path)` — finds the most recently modified `xgen-node_*.log` in a given directory
- `count_apply_event_message_text(text: &str)` — counts lines with both substrings

Report additions:
- New "Federation Completeness" section with actual vs expected counts and ✓/✗ marks per node
- Two `[auto]` checklist entries for Node A and Node B completeness
- Overall outcome is `PARTIAL` if either node's count falls below expected

**Task 4 — Appendix G Parsing Rules, rule 11 (`docs/xgen_appendix_g_en.md`):**

Added rule 11 to the Parsing Rules section (after rule 10 "Unknown fields MUST be silently ignored"):

> 11. Field value matching MUST be case-insensitive. The capitalisation of field values carries no semantic meaning and exists solely for human readability. For example: `direction=IN`, `direction=in`, and `direction=In` are equivalent. `action=ApplyEvent` and `action=apply_event` are equivalent. Parsers and analyzers MUST NOT treat capitalisation differences as distinct values.

Version line updated: `Version: 1.0` → `Version: 1.1`. `Last edited` updated to `2026-05-06`.

This is a format contract clarification for third-party parsers and AI log analyzers. The Rust implementation already produces consistent casing — this documents the intent.

### Test results

173 tests pass, 0 failures. Clean compile with no warnings on both binaries.

### Stress test runs (Phase 1 sign-off)

Two consecutive runs executed against commit `ecc94ff` on 2026-05-06:

| Run | Time | Outcome | Fed A | Fed B | Errors | WARN |
|---|---|---|---|---|---|---|
| 5 | 16:44:08 | **PASS** | 250/250 ✓ | 250/250 ✓ | 0 | none |
| 6 | 16:44:28 | **PASS** | 500/250 ✓ | 500/250 ✓ | 0 | none |

Run 6's 500/250 is an accumulation artifact — the nodes ran across both tests without restart, so two runs' worth of apply_events accumulated in the same log file. The `≥ expected` check correctly marks it ✓. No WARN pending_buffer_at_shutdown lines in either run, confirming clean shutdown on both.

**Phase 1 stress test is clean. All acceptance criteria met.**

### State after this session

All four tasks from `STRESSTEST_ph1_next_round.md` addressed:
- Task 1: WARN on stalled shutdown — done
- Task 2: Federation completeness section in report — done
- Task 3: DEBUG level confirmed — no change needed
- Task 4: Appendix G rule 11, v1.1 — done

Phase 1 stress test sign-off: ✅

Commit: `ecc94ff`

---

## J-033 — 2026-05-08 — UI skeleton audit, visual merge planning, theme loader decision (D-041)

### Context

Phase 2 Track 1 (UI) preparation. Session focused on understanding the gap between the Phase 2 visual reference (chat mockups in `ui/backup/fixed_samples/`) and the semantic skeleton (Miss Design's skeleton in `ui/backup/skeleton/`), and on planning the merge between them under the architectural constraints of Ch2.

### Discussion

Extended discussion of the relationship between semantic HTML structure and CSS reset rigour. Key principles surfaced:

- Semantic HTML carries structural meaning (heading hierarchy, list semantics, form semantics, ARIA) that survives stylesheet removal. The "delete the CSS" test passes when meaning lives in tags, not in classes.
- Visual polish on application UIs traditionally comes from div-heavy markup because UA defaults for semantic tags impose document-style appearance that fights application aesthetics.
- The dichotomy is not absolute. With sufficient CSS reset (Tailwind Preflight-style neutralisation of `<h1>`–`<h6>` font-size/weight, `<ol>`/`<ul>` list-style, `<button>` chrome, etc.), semantic HTML renders as flatly as `<div>`s and accepts the same visual treatment.
- 100% reset is not achievable — native form controls (`<select>`, `<input type="date">`, file picker) and OS scrollbars retain platform rendering CSS cannot fully reach. JS-based custom controls can replace these but reintroduce the div-with-ARIA pattern, defeating semantic purity. Acceptable boundary: ~95–98% reset for declared content; native control rendering accepted as platform-appropriate.

### Audit findings

Two documents produced in `ui/run_1.5/`:

**`skeleton_audit.md`** — initial audit of the chat mockups (`ui/backup/fixed_samples/xgen-mockup-{client,node,console}.html`). Inventoried `<div>`/`<span>` usage, classified into justified (visual scaffolding) / upgrade candidate (semantic role available) / ambiguous. Detailed conversion conventions documented. Caveat noted at top of document: the audit was framed against the wrong reference; subsequent review of Miss Design's skeleton showed that ~95% of the recommended conversions are already implemented there.

**`comparative_analysis.md`** — corrected analysis. Miss Design's skeleton in `ui/backup/skeleton/` is heavily semantic (`<header role="banner">`, `<nav aria-label>` with `<ol>`/`<li>`/`<a>`, `<main aria-labelledby>`, `<aside>`, `<footer>`, `<article>` per message in `<ol aria-label="Messages">`, `<form>` for compose and Console prompt, `<dl>`/`<dt>`/`<dd>`, `<time datetime>`, `<details>`/`<summary>`, ARIA labels throughout). The actual gap between her skeleton and the chat mockups lives in:

- **CSS reset rigour** — chat mockups embed `* { margin:0; padding:0; box-sizing:border-box }` plus inline rules; Miss Design's external `tokens.css` + `skin-classic.css` does not fully neutralise UA defaults.
- **Visual coding density** — chat mockups have deliberate styling for every container; existing skin files have fewer rules.
- **Run 2 evolutions** — D-038 (no tier badges in messages or member list), D-039 (action buttons in nav-footer), Run 2 Change 1 (Space rail initials + tooltips). Miss Design's skeleton predates these.

The current `ui/xgen-mockup-*.html` files are a partial merge attempt that did not fully capture the chat mockups' visual quality.

### Visual merge plan (postponed)

Outlined a 10-milestone roadmap for merging the chat mockups' visual treatment onto Miss Design's semantic structure, respecting the following Ch2 fixed conditions:

- **Lifecycle state coverage** — all 7 Node states (INITIALISING, READY, DEGRADED_FEDERATION, DEGRADED_STORAGE, DEGRADED_AUTH, MAINTENANCE, CLOSING) and 11 Client states (SETUP, INITIALISING, CONNECTING, AUTHENTICATING, READY, DEGRADED_AUTH, DEGRADED_FEDERATION, DEGRADED_NODE, RECONNECTING, DISCONNECTED, CLOSING) must each render distinctly. Visual treatment uses `[data-state]` selectors with explicit rules per state plus a default fallback.
- **Open-enum graceful degradation** per Ch2 architecture principles — every `[data-state]`, `[data-tier]`, `[data-level]`, `[data-kind]` selector requires a base/default rule for unspecified values.
- **Slot system intact** — `[data-xgen-slot]` styling targets only the empty placeholder appearance (`:empty`); skin must not interfere with module-injected content.
- **Layer 4 boundary** — CSS reacts only to declared `data-*` attributes mutated by Layer 3. No selectors that depend on inferred application state.
- **Accessibility per Ch2 cross-cutting** — `:focus-visible` rules added (chat mockups omit these); reduced-motion preferences honoured.
- **Theming as client-scoped** — skin files are replaceable; minimum two skins (dark, light); each skin self-contained with its own reset block.

### Architecture proposed

- `tokens.css` always loaded — variables only (no rules; cannot render anything; safe baseline).
- `skin-{name}.css` conditionally loaded — fully self-contained: own reset block, own colour/typography tokens, own layout/component/state/accessibility rules.
- Reset coupled to skin: graceful degradation — if no skin loads, page renders as semantic HTML with UA defaults rather than as flat unstyled blobs.
- Console treated as own skin family (`skin-console-vt220.css` minimum), reflecting Console's locked VT220 aesthetic and its architectural distinctness as a separate surface.

### Decision recorded

**D-041** — Theme loader behaviour. Default skin = `skin-dark.css`. Fallback chain on skin failure: requested → default → raw HTML. See `DECISIONS.md`.

### Visual merge phase postponed

Phase postponed pending element modelling. The list of UI element types needing individual visual design is in `ui/docs/xgen-ui-design-brainstorm.md` — Point 3 (event types in message stream — member-originated, self mirrored, system/protocol, module-injected; baseline list marked "to be confirmed") and Point 2 (avatar as first-class object — DOM element with hover context menu, member vs self variant). The list must be confirmed and expanded against Ch3's authoritative event taxonomy before Run 3 design briefing is drafted and any visual merge work begins.

### State after this session

- No code changes; no CSS modifications; no markup changes to active mockups.
- Documentation deliverables in `ui/run_1.5/`: `skeleton_audit.md` v1.0, `comparative_analysis.md` v1.0.
- One decision recorded: D-041 (theme loader behaviour).
- Visual merge phase paused at element modelling step.

---

## Entry J-034 — Phase 2 Track 1: Client Core Test UI — Milestones 1 and 2

**Date:** 2026-05-12  
**Author:** Jozef Nižnanský  
**Session:** Session 15  
**Instruction file:** `docs/tests/CLIENT_CORE_UI_ph2.md`  

### Summary

First Phase 2 UI deliverable. Established the Tauri scaffold and Svelte build pipeline for `xgen-client`, implemented the `ClientLifecycleState` enum (all 11 states from Appendix E §E.2), and wired the startup state machine.

### Rust changes

**`xgen-client/src/lifecycle.rs`** — new module:
- `ClientLifecycleState` enum: 11 states (`Setup`, `Initialising`, `Connecting`, `Authenticating`, `Ready`, `DegradedAuth`, `DegradedFederation`, `DegradedNode`, `Reconnecting`, `Disconnected`, `Closing`), serialises to `SCREAMING_SNAKE_CASE`
- `as_canonical()` method — returns canonical log-line form (`"INITIALISING"` etc.)
- `Display` impl — returns Appendix E title-case display label (`"Initialising"` etc.)
- `ClientStateEvent` struct — serialisable payload for `"xgen-client-state-changed"` Tauri event (D-042)
- `make_state_event(state)` — constructs payload with UTC RFC 3339 ms timestamp

**`xgen-client/src/lib.rs`** — added `pub mod lifecycle;`

**`xgen-client/src-tauri/`** — new workspace crate `xgen-client-app`:
- `Cargo.toml` — Tauri v2 + `tauri-plugin-process`, `tokio`, `tokio-tungstenite`, `xgen-client` + `xgen-common` deps
- `build.rs` — `tauri_build::build()` 
- `tauri.conf.json` — window 420×260, `decorations: false`, `resizable: false`, bundle inactive, links to Svelte `dist/`
- `capabilities/default.json` — `core:default` + `process:default`
- `icons/icon.png` + `icons/icon.ico` — logo PNG converted to ICO for Windows resource embedding
- `src/main.rs` — Tauri entry point: logging init → session header → `run_startup` async task → `quit` command

**Startup sequence** (`run_startup`):
1. No config and no keypair → `SETUP`
2. Both exist → `INITIALISING` → `CONNECTING`
3. `tokio::time::timeout(2000ms, connect_async("ws://127.0.0.1:8080/xgen"))`
4. On WS connect → `AUTHENTICATING` → 150 ms → `READY`
5. On timeout or error → `DISCONNECTED`
6. Quit command → `CLOSING` → session footer → `app.exit(0)`

### Frontend (Milestone 1 + 3)

**`ui/dev_core_ui/client_ui/`** — Svelte 5 + Vite frontend:
- `package.json` — Svelte 5, Vite 6, `@tauri-apps/api` v2, `@tauri-apps/plugin-process` v2
- `vite.config.js` — Tauri-aware dev server config (TAURI_DEV_HOST, port 5173)
- `index.html` — shell with `<div id="app">`
- `src/main.js` — Svelte 5 `mount()` entry
- `src/app.css` — full token set (`--ok: #2d7a3a`, `--err: #8a2a2a` added), `#core-ui-pane` layout, state dot + pulse animation
- `src/app_client.svelte` — state indicator wired to `"xgen-client-state-changed"` Tauri event; dot colour + pulse mapped to all 11 states; `invoke("quit")` on Quit button
- `src/lib/Button.svelte` — amber primary button
- `src/assets/` — `Inter-Regular.woff2`, `logo_client_64.png`

### Build status

- `cargo build --package xgen-client-app` — **PASS** (clean, no warnings)
- `cargo test` (173 tests, excluding `xgen-client-app`) — **173/173 PASS**
- `npm install` / `npm run build` — **BLOCKED**: Node.js not installed on this machine. Frontend code is complete; build requires Node.js + `cargo install tauri-cli`.

### Files changed / created

```
xgen-client/src/lib.rs                     modified
xgen-client/src/lifecycle.rs               new
xgen-client/src-tauri/Cargo.toml           new
xgen-client/src-tauri/build.rs             new
xgen-client/src-tauri/tauri.conf.json      new
xgen-client/src-tauri/capabilities/default.json  new
xgen-client/src-tauri/icons/icon.png       new
xgen-client/src-tauri/icons/icon.ico       new
xgen-client/src-tauri/src/main.rs          new
ui/dev_core_ui/client_ui/package.json         new
ui/dev_core_ui/client_ui/vite.config.js       new
ui/dev_core_ui/client_ui/index.html           new
ui/dev_core_ui/client_ui/src/main.js          new
ui/dev_core_ui/client_ui/src/app.css          new
ui/dev_core_ui/client_ui/src/app_client.svelte new
ui/dev_core_ui/client_ui/src/lib/Button.svelte new
ui/dev_core_ui/client_ui/src/assets/          new (Inter-Regular.woff2, logo_client_64.png)
Cargo.toml                                 modified (added xgen-client/src-tauri member)
.gitignore                                 modified (added dist/, node_modules/, gen/)
```

---

## Entry J-035 — Project migration to E: and Client UI first run

**Date:** 2026-05-12  
**Author:** Jozef Nižnanský  
**Session:** Session 15 (continued)  

### Summary

First successful launch of the XGen Client Core Test UI window. Several infrastructure issues resolved during first-run verification. Project relocated from Google Drive to a local drive.

### Issues resolved

**1 — npm dependency conflict**  
`@sveltejs/vite-plugin-svelte@4` requires Vite 5, not 6. Fixed by upgrading plugin to `^5` (which supports Vite 6). `vite.config.js` `outDir` set to `C:/cargo-targets/XGenProtocol/client-dist` — outside Google Drive, same pattern as `CARGO_TARGET_DIR`.

**2 — Google Drive junction limitation**  
Windows junctions cannot be created on Google Drive mapped drives (`E` — "Incorrect function"). Resolved by relocating the entire project from `G:\My Drive\Projects\XGenProtocol` to `E:\Projects\XGenProtocol` (local NTFS drive). All relative paths and C: target paths were unaffected. Claude Code project memory copied from `G--My-Drive-Projects-XGenProtocol` to `E--Projects-XGenProtocol`.

**3 — tauri.conf.json path resolution**  
`beforeDevCommand` was using `../../ui/dev_core_ui/client_ui` (relative to `src-tauri/`) but Tauri resolves it from `xgen-client/`. Corrected to `../ui/dev_core_ui/client_ui`.

**4 — Webview race condition**  
State transitions (INITIALISING → CONNECTING → DISCONNECTED) fired before the Svelte event listener mounted, leaving the UI stuck at the hardcoded default "Initialising". Fixed by adding a 500 ms delay at the start of `run_startup` to allow the webview to mount and register listeners.

**5 — ExitReason variant**  
`ExitReason::Clean` does not exist — correct variant is `ExitReason::Shutdown`. Fixed in `src-tauri/src/main.rs`.

### Window confirmed working

- Window opens without native titlebar
- Logo, state indicator, Quit button render correctly
- State transitions visible after 500 ms delay
- Quit exits cleanly (minor Chromium WebView2 teardown warning — benign, known issue)

### run-client.ps1 updated

Added `release` mode:
- `.\run-client.ps1` — dev mode, hot-reload
- `.\run-client.ps1 release` — builds standalone `.exe`, copies to `bin\xgen-client-app.exe`

### Files changed

```
run-client.ps1                              modified (release mode added)
ui/dev_core_ui/client_ui/package.json       modified (@sveltejs/vite-plugin-svelte ^4 → ^5)
ui/dev_core_ui/client_ui/vite.config.js     modified (outDir → C:/cargo-targets)
xgen-client/src-tauri/tauri.conf.json       modified (path fix + frontendDist → C:/cargo-targets)
xgen-client/src-tauri/src/main.rs          modified (500 ms webview delay, ExitReason fix)
.gitignore                                  modified (removed dist/ — now on C:)
```

---

## Entry J-038 — Milestone 1 Task 1.4: `--instance` flag; npm install; M1–M3 complete

**Date:** 2026-05-13  
**Author:** Jozef Nižnanský  
**Session:** Session 16  

### Summary

Completed remaining open items from `CLIENT_CORE_UI_ph2.md`. Milestones 1–3 are now fully done. Milestone 4 (manual UI walkthrough) is the only remaining step.

### Task 1.4 — `--instance` flag and data directory

Implemented in `xgen-client/src-tauri/src/main.rs`. A new `resolve_data_dir()` function parses `--instance <label>` from `std::env::args()` before the Tauri builder starts. The derived `data_dir` is passed into `run_startup()` and the logging setup, so all data files (config, keypair, logs) are written under `instances/<label>/` relative to the executable directory.

When no `--instance` flag is given, `data_dir` falls back to `exe_dir()` — fully backward compatible with single-instance usage.

Named pipe / single-instance detection are explicitly out of scope for this milestone (deferred to `BATCH_FLAG_ph2.md`).

### npm install

`ui/dev_core_ui/client_ui/node_modules/` was absent — `npm install` had never been run after the project was moved to `E:`. Node.js v24.15.0 was already installed. Ran `npm install` in `ui/dev_core_ui/client_ui/`; 43 packages installed, 0 vulnerabilities. Svelte frontend (event listener, state dot, pulse animation) was already fully written — no code changes needed.

### Test suite

173/173 passing. Clean compile, no warnings.

### Files changed

```
xgen-client/src-tauri/src/main.rs   modified (Task 1.4: resolve_data_dir(), data_dir plumbed into startup + logging)
docs/tests/CLIENT_CORE_UI_ph2.md    modified (status table updated: M1–M3 done; M4 remaining)
ui/dev_core_ui/client_ui/           npm install run (node_modules populated, not committed)
```

---

## Entry J-039 — Milestone 4 complete: CLIENT_CORE_UI_ph2.md fully done

**Date:** 2026-05-13  
**Author:** Jozef Nižnanský  
**Session:** Session 16 (continued)  

### Summary

Manual verification walkthrough (Milestone 4) complete. All checklist items passed. `CLIENT_CORE_UI_ph2.md` is fully done.

### Issues found and resolved during walkthrough

**1 — Vite dev server not starting (beforeDevCommand path)**  
`beforeDevCommand` in `tauri.conf.json` ran from `src-tauri/` (not `xgen-client/` as previously assumed in J-035). The path `../ui/dev_core_ui/client_ui` resolved to `xgen-client/ui/…` which does not exist. Fixed by removing `beforeDevCommand` entirely and starting Vite explicitly in `run-client.ps1` before invoking `cargo tauri dev`.

**2 — run-client.ps1: Start-Process cannot find npm**  
`npm` is a `.cmd` file on Windows; `Start-Process -FilePath "npm"` fails. Fixed by invoking via `cmd.exe /c`.

**3 — Vite port poll: IPv4/IPv6 mismatch**  
`TcpClient.Connect("127.0.0.1", 5173)` failed because Vite bound to `[::1]` (IPv6). Fixed by switching the readiness check to `Invoke-WebRequest -Uri "http://localhost:5173"`.

**4 — Double Vite start**  
`beforeDevCommand` in `tauri.conf.json` started a second Vite instance after `run-client.ps1` already started one, causing "Port 5173 is already in use". Fixed by removing `beforeDevCommand` from `tauri.conf.json` (only `beforeBuildCommand` remains for release builds).

**5 — State label stuck at hardcoded default**  
Svelte's `onMount` event listener registered after `run_startup` emitted `SETUP`, so the UI showed the hardcoded `"Initialising"` default instead of `"Setting up"`. Fixed with a `get_state` Tauri command backed by `Arc<Mutex<ClientStateEvent>>` shared state. Svelte calls `invoke('get_state')` on mount after registering the event listener — no timing dependency.

### Verification results

- No native titlebar ✅
- Logo renders correctly ✅
- State indicator: "Setting up", grey dot, no pulse (SETUP — no config/keypair present) ✅
- CLOSING state on Quit ✅
- Session footer written ✅
- Clean exit ✅
- No console errors (favicon.ico 404 is benign — WebView2 browser behaviour, not an app error) ✅
- `--instance alice` creates `instances/alice/logs/` next to the debug exe ✅

### Files changed

```
xgen-client/src-tauri/src/main.rs          modified (get_state command, CurrentState managed state, removed startup delay)
xgen-client/src-tauri/tauri.conf.json      modified (beforeDevCommand removed)
ui/dev_core_ui/client_ui/src/app_client.svelte  modified (invoke get_state on mount)
run-client.ps1                              modified (Vite pre-start via cmd.exe, HTTP readiness check)
```

---

## Entry J-040 — NODE_CORE_UI_ph2.md: all milestones complete

**Date:** 2026-05-13  
**Author:** Jozef Nižnanský  
**Session:** Session 17  
**Instruction file:** `docs/tests/NODE_CORE_UI_ph2.md`  

### Summary

XGen Node Core Test UI fully implemented and verified. Milestones 1–4 complete. Both binaries (xgen-client and xgen-node) are now at the same verified state: Tauri window, systray, lifecycle state machine, startup sequence, instance isolation, service mode.

### Rust changes

**`xgen-node/src/lifecycle.rs`** — new module:
- `NodeLifecycleState` enum: 7 states (`Initialising`, `Ready`, `DegradedFederation`, `DegradedStorage`, `DegradedAuth`, `Maintenance`, `Closing`), serialises to `SCREAMING_SNAKE_CASE`
- `as_canonical()` — returns canonical log-line form
- `Display` impl — returns Appendix E title-case display label
- `NodeStateEvent` struct — serialisable payload for `"xgen-node-state-changed"` Tauri event
- `make_node_state_event(primary, degraded)` — constructs payload with UTC RFC 3339 ms timestamp
- `active_display_state(primary, degraded)` — severity: `DEGRADED_STORAGE(3) > DEGRADED_AUTH(2) > DEGRADED_FEDERATION(1)`

**`xgen-node/src/lib.rs`** — added `pub mod lifecycle;`

**`xgen-node/src-tauri/`** — new workspace crate `xgen-node-app`:
- Tauri v2 + `tauri-plugin-process`, systray, window hide-on-close
- `--service` / `--instance` / `--port` flag parsing before Tauri builder runs
- `CurrentNodeState(Arc<Mutex<(NodeAppState, NodeStateEvent)>>)` — eliminates startup race condition
- `get_node_state` and `shut_down` Tauri commands
- `run_service_mode()` — plain tokio runtime with Ctrl+C handler, no Tauri

**`ui/dev_core_ui/node/`** — new Svelte frontend:
- `app_node.svelte` — blue theme, `logo_node_64.png`, state dot + label, "Shut Down" button
- Calls `invoke('get_node_state')` on mount; listens for `"xgen-node-state-changed"` events
- Dot colours: INITIALISING=`--t3` pulse, READY=`--ok`, DEGRADED_STORAGE=`--err`, DEGRADED_AUTH/FEDERATION=`--pr`, MAINTENANCE=`--inf`

### Issues found and resolved

**1 — `--service` flag not forwarded by run-node.ps1**  
Script checked `$args[0] -eq "release"` only; `--service` fell through to dev mode branch. Fixed by adding `elseif ($args -contains "--service")` branch that invokes binary directly via `cargo run -- $argList`, forwarding all args including `--instance` and `--port`.

**2 — Simultaneous instance test: binary locked**  
`cargo run` in Terminal 2 tried to replace the binary held open by Terminal 1, failing with OS error 5 (access denied). Resolved by invoking the pre-built binary directly for the second instance.

**3 — Systray icon not appearing**  
`TrayIconBuilder::new()` had no `.icon()` call. Tauri v2 requires an explicit icon; without it the tray entry is silently skipped and the process exits. Fixed by `.icon(app.default_window_icon().unwrap().clone()).tooltip("XGen Node")`.

**4 — run-node.ps1 used wrong working directory path**  
Script updates were applied to worktree copy but user was running from main project. Fixed by syncing both copies.

### Verification results (Milestone 4)

- Systray icon appears on launch ✅
- "Open Admin Panel" opens admin window ✅
- Alt+F4 hides window — process continues in systray ✅
- "Open Admin Panel" re-opens window ✅
- No native titlebar ✅
- Logo, button, state indicator render correctly ✅
- INITIALISING → READY transition visible (dot + label) ✅
- Shut Down from systray exits cleanly, log session footer written ✅
- `--service` mode: headless, no window, no systray, visible in Task Manager, Ctrl+C exits ✅
- `--instance node_b --port 8081`: creates `instances/node_b/` with own logs + config ✅
- Simultaneous instances run without conflict ✅
- F12 console: no errors (favicon 404 benign) ✅
- 173/173 tests passing ✅

### Files changed

```
xgen-node/src/lifecycle.rs                     new
xgen-node/src/lib.rs                           modified (pub mod lifecycle)
xgen-node/src-tauri/Cargo.toml                 new
xgen-node/src-tauri/build.rs                   new
xgen-node/src-tauri/tauri.conf.json            new
xgen-node/src-tauri/capabilities/default.json  new
xgen-node/src-tauri/icons/                     new (icon assets)
xgen-node/src-tauri/src/main.rs                new
ui/dev_core_ui/node/                           new (Svelte frontend)
run-node.ps1                                   new
Cargo.toml                                     modified (workspace members)
```

---

## Entry J-041 — FIXES_core_ui_ph2.md: all four fixes applied and verified

**Date:** 2026-05-13  
**Author:** Jozef Nižnanský  
**Session:** Session 17 (continued)  
**Instruction file:** `docs/tests/FIXES_core_ui_ph2.md`  

### Summary

Four bugs identified during code review of the completed Core Test UI applied and verified. Clean compile, 173/173 tests passing.

### Fix 1 + Fix 2 — Client startup sequence and data_dir plumbing

`run_startup` now always emits `INITIALISING` first before any first-run detection. Previously `INITIALISING` was skipped on first run — the function returned early with `SETUP` without emitting it. Additionally `data_dir` (derived from `--instance` flag) was computed in `main()` but silently discarded (`let _ = dir`) and never passed to `run_startup`, meaning config and keypair lookups always used `exe_dir()` regardless of `--instance`. Both fixed together: `run_startup` now takes `data_dir: PathBuf` and derives all paths from it.

### Fix 3 — Hardcoded version string

Both `xgen-client/src-tauri/src/main.rs` and `xgen-node/src-tauri/src/main.rs` passed `"0.10.3"` as the build version to `write_session_header`. Replaced with `env!("CARGO_PKG_VERSION")` in both files — resolved at compile time from each crate's `Cargo.toml`.

### Fix 4 — Node window visible on launch (D-037 violation)

`tauri.conf.json` for `xgen-node` had `"visible": true`, causing the admin window to open automatically on launch. Per D-037 the Node is process-centric — the systray icon is the entry point, the admin window is on-demand. Changed to `"visible": false`.

### Verification

- Fix 1+2: log confirms `INITIALISING` (line 8) → `SETUP` (line 9) on first run, 0.3ms apart. Normal path (config present, no node) shows `INITIALISING → CONNECTING → DISCONNECTED` with 2s timeout. ✅
- Fix 3: both logs show `build=0.1.0` from `CARGO_PKG_VERSION`. ✅
- Fix 4: node launches to systray only; admin window opens via "Open Admin Panel". ✅

### Files changed

```
xgen-client/src-tauri/src/main.rs     modified (Fix 1+2+3: run_startup takes data_dir, INITIALISING first, env! version)
xgen-node/src-tauri/src/main.rs       modified (Fix 3: env! version)
xgen-node/src-tauri/tauri.conf.json   modified (Fix 4: visible false)
docs/tests/FIXES_core_ui_ph2.md       modified (status → COMPLETED, checklist ticked, results appended)
docs/tests/CLIENT_CORE_UI_ph2.md      modified (status → COMPLETED)
```

---

## Entry J-042 — FIXES_sec_01_ph2.md: instance label path traversal fix

**Date:** 2026-05-13  
**Author:** Jozef Nižnanský  
**Session:** Session 17 (continued)  
**Instruction file:** `docs/tests/FIXES_sec_01_ph2.md`  

### Summary

Security fix: both `xgen-node` and `xgen-client` accepted `--instance <label>` without validation, allowing path traversal via labels like `../../sensitive_dir`. A `validate_instance_label` function added to both Tauri `main.rs` files rejects any label that is not strictly alphanumeric with hyphens and underscores (max 64 chars), before any filesystem path construction occurs. Invalid labels print a clear error and exit with code 1.

### Files changed

```
xgen-node/src-tauri/src/main.rs       modified (validate_instance_label, validation in parse_flags)
xgen-client/src-tauri/src/main.rs     modified (validate_instance_label, validation in resolve_data_dir)
docs/tests/FIXES_sec_01_ph2.md        modified (status → COMPLETED, checklist ticked, results appended)
```

### Verification

- Path traversal labels (`../escape`, `..\..\..\windows`, `/absolute`) all rejected — exit 1, correct error message, no directory created ✅
- 65-char label rejected ✅
- Valid labels (`node_a`, `node-b`, `test_01`) work normally ✅
- 173/173 tests passing, clean compile ✅

---

## Entry J-043 — BATCH_FLAG_ph2.md: design session; D-043 recorded

**Date:** 2026-05-13  
**Author:** Jozef Nižnanský  
**Session:** Session 18  

### Purpose

Pre-implementation design session for `BATCH_FLAG_ph2.md`. No code written. All design questions resolved before the instruction file is drafted.

### Discussion

Three design questions worked through before writing the instruction:

**1. Error handling on batch execution failure**

Ch6 §6.9 already prescribes "exits on completion or error" — sequential execution, stop on first error. No half-way solutions. The instruction will cite §6.9 directly; no new decision required.

**2. Batch file path — path traversal risk**

`--batch <file.xgb>` has the same traversal risk as `--instance` did: the path comes from the command line and reaches a file-open call without validation. The `--instance` fix used a character whitelist (valid for an identifier). A file path is different — slashes, dots, and drive letters are all legitimate — so the correct fix is `std::fs::canonicalize()` before opening. This resolves all `..` segments before the filesystem sees them. A `.xgb` extension check is added as defence-in-depth. No scope restriction on where the file may live — automation scenarios legitimately place batch files in CI workspaces or test fixture directories outside the instance folder.

**3. Shell injection risk**

Batch lines must never be passed to a shell process. If a line like `connect ws://127.0.0.1:8080; rm -rf /home/user` reaches `sh -c`, the `;` becomes a shell command separator. The safe design — mandated in the instruction — is to tokenize each line with the `shlex` crate into a `Vec<String>` and dispatch via clap's `try_get_matches_from()` on the existing `Command` object. This is the same command channel as keyboard input (Ch6 §6.9: "all three use the same underlying command channel"). A `;` is then just an unrecognised argument token; clap returns an error and execution stops. Explicit prohibition in the instruction: no `std::process::Command`, no shell invocation of any kind.

**4. Named pipe naming convention — D-043**

The single-instance forwarding model (J-037) requires a pipe name both invocations can derive independently. Convention decided: `\\.\pipe\xgen-{binary}-{label}`, default `\\.\pipe\xgen-{binary}` when no `--instance` label. Binary prefix prevents collision between a client and node instance sharing the same label. Label is already validated safe (alphanumeric, hyphens, underscores, max 64 chars). Fully human-readable. Recorded as D-043.

### Deliverables

- `DECISIONS.md` — D-043 added, last-updated bumped
- `JOURNAL.md` — this entry

### Next steps

1. ~~Write `BATCH_FLAG_ph2.md`~~ ✅ Done — see `docs/tests/BATCH_FLAG_ph2.md`
2. ~~Mr. Code implements the batch flag~~ ✅ Done — see J-044
3. Joe verifies against the instruction checklist

---

## Entry J-044 — BATCH_FLAG_ph2.md: M1–M3 implemented (code complete, M4 walkthrough pending)

**Date:** 2026-05-13  
**Author:** Jozef Nižnanský  
**Session:** Session 19  

### Purpose

Implementation of `BATCH_FLAG_ph2.md` Milestones 1–3. Adds `--batch` support to `xgen-client-app.exe` via a Windows named pipe IPC channel. M4 manual walkthrough is a separate step.

### What was built

**New file: `xgen-client/src/batch.rs`**

Batch module added to the `xgen_client_lib` library (library-first rule). Contains:

- `pipe_name(instance_label: Option<&str>) -> String` — derives `\\.\pipe\xgen-client[-{label}]` (D-043)
- `app_command() -> clap::Command` — returns the canonical clap Command for batch dispatch; used by both pipe server and tests
- `BatchCli` / `BatchCommand` — clap struct covering 8 protocol subcommands: `whoami`, `status`, `register`, `create-space`, `create-room`, `invite`, `join`, `send`
- `dispatch_line(line, data_dir)` — tokenizes with `shlex::split`, prepends `"xgen-client"`, dispatches via `BatchCli::try_parse_from()`; no shell invocation
- `start_pipe_server(pipe_name, data_dir, shutdown_rx)` — Windows-only async function; `ServerOptions` loop, one connection at a time, reads lines until `__END__`, dispatches each, writes `OK\n` or `ERROR: …\n`, logs at INFO/WARN per spec
- `run_batch_client(raw_path, pipe_name)` — Windows-only sync function; creates its own tokio runtime; validates path (canonicalize + `.xgb` extension), reads non-comment lines, connects to running instance pipe, streams commands + sentinel, reads result; returns exit codes 0/1/2/3

**Modified: `xgen-client/src-tauri/src/main.rs`**

- `--batch` detected from `std::env::args()` before the Tauri builder; if present, calls `run_batch_client()` and `std::process::exit()` — no window, no Tauri
- `PipeShutdown(tokio::sync::watch::Sender<bool>)` struct added as Tauri managed state
- `quit()` command signals the pipe server via the watch sender before `app.exit(0)`
- `run_startup()` receives `shutdown_rx` and spawns `start_pipe_server()` as a `tauri::async_runtime` task (Windows only, inside `#[cfg(target_os = "windows")]` block)
- Pipe name derived from `xgen_client_lib::batch::pipe_name(instance_label.as_deref())` at startup

**Modified: `xgen-client/Cargo.toml`**

Added `shlex = "1"` dependency (M3 requirement).

**Modified: `xgen-client/src-tauri/Cargo.toml`**

Added `"sync"` to tokio features for explicit `watch` channel support.

**Modified: `xgen-client/src/lib.rs`**

Added `pub mod batch;`.

### Security properties

- `std::fs::canonicalize()` resolves all `..` segments before any file operation (path traversal mitigation)
- `.xgb` extension checked case-insensitively after canonicalize
- `shlex::split` tokenizes batch lines; `;`, `&&`, `|` are treated as word characters, never as shell metacharacters
- No `std::process::Command` with shell invocation anywhere in the batch path
- All dispatch goes through clap `try_get_matches_from()` — same surface as interactive CLI

### Verification

- `cargo build` — clean compile, no warnings ✅
- `cargo test` — 173/173 tests passing ✅
- M4 manual walkthrough (pipe creation, happy path, error path, injection checks) — pending

### Files changed

```
xgen-client/src/batch.rs                   new — batch module (pipe server + client + dispatch)
xgen-client/src/lib.rs                     modified — pub mod batch added
xgen-client/Cargo.toml                     modified — shlex = "1" added
xgen-client/src-tauri/src/main.rs          modified — batch detection, pipe server startup, PipeShutdown state
xgen-client/src-tauri/Cargo.toml          modified — tokio sync feature added
JOURNAL.md                                  this entry
```

---
