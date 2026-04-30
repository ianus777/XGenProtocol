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
