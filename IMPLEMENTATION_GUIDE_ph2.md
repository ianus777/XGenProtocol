# XGen Protocol — Phase 2 Implementation Guide

> **Status:** ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated:** 2026-05-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Overview

This guide directs Phase 2 implementation of the XGen Protocol. Phase 2 produces the full protocol: end-to-end encryption, state resolution, higher-tier auth interfaces, space migration, identity replication, bootstrap node discovery, node reputation, and DM space promotion.

**Prerequisite:** the xgen-core crate split (`docs/tests/XGEN_CORE_SPLIT_ph2.md`) must be complete and all 173 tests passing before any layer in this guide is started.

All implementation decisions must be consistent with `docs/xgen_ch3_specification.md` sections 3.9–3.16. When this guide and the spec conflict, the spec is authoritative. When the spec is ambiguous, flag the ambiguity before implementing — do not resolve it silently.

---

## Architecture Principle — xgen-core is the Destination

After the crate split, all Phase 2 protocol code goes into `xgen-core/src/`. Neither `xgen-node` nor `xgen-client` receive new protocol logic directly. The library-first rule from Phase 1 still holds — and now has a proper home.

```
New Phase 2 protocol code → xgen-core/src/
Node-specific runtime wiring → xgen-node/src/ (thin)
Client-specific command wiring → xgen-client/src/ (thin)
```

Every new module described in this guide is created inside `xgen-core/src/` unless explicitly stated otherwise.

---

## New Crates for Phase 2

The following crates are added to `xgen-core/Cargo.toml` as Phase 2 layers require them. Do not add all at once — add each crate only when the layer that needs it begins.

| Crate | Version | Added at | Purpose |
|---|---|---|---|
| `openmls` | `0.5` | Layer 18 | MLS (RFC 9420) group operations |
| `openmls_rust_crypto` | `0.2` | Layer 18 | MLS cryptographic backend |
| `openmls_basic_credential` | `0.2` | Layer 18 | MLS credential type |
| `axum` | `0.7` | Layer 17 | HTTP server for Bootstrap Node directory endpoint |
| `reqwest` | `0.12` | Layer 17 | HTTP client for fetching Bootstrap Node directory |
| `tower` | `0.4` | Layer 17 | Middleware support for axum |

Note on MLS crate versions: the `openmls` ecosystem is actively developed. Verify the latest stable versions at implementation time. The versions above are reference points — use the latest stable release that compiles cleanly with the existing dependency tree.

---

## Implementation Order

Implement in this exact order. Each layer depends on the previous. Do not skip ahead.

---

### Layer 11 — Wire Format Phase 2 Extensions
**Spec refs:** 3.9–3.16 (all new EventTypes and message types)

Phase 2 introduces new EventTypes and message structs across all sections. All of them are added to `xgen-core/src/wire/types.rs` in a single pass before any protocol logic is implemented. This prevents import churn across subsequent layers.

**New EventTypes to add to the `EventType` enum:**

State events:
- `state.node_priority` — manual Node ordering declaration (3.9.3 Layer 5a)
- `state.dm_promote` — DM Space promotion completion (3.16.3)

DM promotion control messages:
- `dm.promote_propose` — initiating member proposes promotion (3.16.3)
- `dm.promote_confirm` — other member confirms (3.16.3)
- `dm.promote_reject` — other member rejects (3.16.3)

Migration messages:
- `migration.request` — owner sends to source Node (3.12.3)
- `migration.propose` — source Node sends to destination Node (3.12.3)
- `migration.accept` — destination Node accepts (3.12.3)
- `migration.reject` — destination Node rejects (3.12.3)
- `migration.event_batch` — Events transferred in bulk (3.12.5)
- `migration.tail_batch` — Events produced during transfer (3.12.5)
- `migration.complete` — destination signals transfer complete (3.12.6)
- `migration.verify_ok` — verification passed (3.12.6)
- `migration.verify_fail` — verification failed (3.12.6)
- `migration.abort` — migration cancelled (3.12.7)

Identity replication:
- `identity.replicate` — home Node pushes Identity record to replica (3.13.4)
- `identity.replicate_ack` — replica acknowledges receipt (3.13.4)

Bootstrap Node:
- `bootstrap.node_register` — Node registers with Bootstrap Node (3.14.3)
- `bootstrap.node_register_ack` — Bootstrap Node confirms registration (3.14.3)
- `bootstrap.node_lookup` — Node queries Bootstrap Node for peers (3.14.4)
- `bootstrap.node_lookup_response` — Bootstrap Node returns peer list (3.14.4)

Reputation:
- `reputation.defederation_signal` — Node reports defederation to Bootstrap Node (3.15.3)

MLS (E2E encryption) — these are protocol-level wrappers around MLS messages:
- `mls.key_package` — client uploads KeyPackage to home Node (3.10.3)
- `mls.key_package_ack` — Node acknowledges KeyPackage receipt (3.10.3)
- `mls.welcome` — Node delivers MLS Welcome to new member (3.10.5)
- `mls.commit` — Node routes MLS Commit to group members (3.10.5)
- `mls.proposal` — Node routes MLS Proposal to group members (3.10.5)

**New message structs** to add (one per EventType above). Follow the existing pattern in `wire/types.rs` — serde derives, snake_case fields, no null values, all required fields explicit. Refer to the spec section listed next to each EventType for the exact field list.

**Test:** serialise each new struct to JSON and deserialise back. Verify round-trip equality. No logic test needed at this layer — types only.

---

### Layer 12 — State Resolution Algorithm
**Spec refs:** 3.9.1–3.9.5, 3.9.7

Create `xgen-core/src/resolution/` module. The state resolution algorithm is a pure function — it takes a set of conflicting Events and returns the single winning Event. No I/O, no database access, no async.

**Files to create:**

| File | Description |
|---|---|
| `resolution/mod.rs` | Module declaration and public API |
| `resolution/algorithm.rs` | The seven-layer resolution function |
| `resolution/state_key.rs` | `StateKey` type — tuple of `(EventType, key_field)` |
| `resolution/conflict.rs` | Conflict detection — identify competing Events for the same state key |

**Core function signature:**

```rust
pub fn resolve(conflicts: &[Event], space_state: &SpaceState) -> Result<&Event, ResolutionError>
```

**Implement the seven layers in order (3.9.3):**

- Layer 1: EventType hardcoded priority table — `membership.ban` beats `membership.join` / `membership.invite` / `membership.kick`; `membership.kick` beats `membership.join` / `membership.invite`; `membership.leave` beats `membership.join`
- Layer 2: Auth Tier of the producing Node — higher Tier wins (Tier 4 > 3 > 2 > 1). Note: always tied in Phase 2 Tier 1 deployments — this layer is future-proofing
- Layer 3: Home Node assertion — for conflicts involving an Identity's own state, the Event originating from the Identity's home Node wins
- Layer 4: Role within Space — higher role wins (owner > admin > moderator > member)
- Layer 5a: Manual Node ordering — consult most recent `state.node_priority` Event if present
- Layer 5b: Federation recency — most recently joined Node wins; home Node treated as joined at Space creation
- Layer 5c: Lexicographic event_id backstop — lower event_id string wins; always produces a unique winner

**Integrate into existing pipeline:**

Update `message/exchange.rs` (`accept_event`) and the DAG snapshot update logic (3.9.7) to call `resolve()` when a new state or membership Event conflicts with an existing state value for the same state key. The loser Event is stored in the DAG permanently — it is never deleted. Only the snapshot is updated to reflect the winner.

**Error codes (3.9.8):** implement the 4xxx error range. Add to the existing error code module.

| Code | String |
|---|---|
| 4001 | `state_conflict_unresolvable` |
| 4002 | `predecessor_timeout` |
| 4003 | `dag_cycle_detected` |
| 4004 | `state_key_invalid` |
| 4005 | `resolution_stack_exhausted` |

**Tests:**

| Test | What it verifies |
|---|---|
| `ban_beats_concurrent_join` | Layer 1: membership.ban wins over concurrent membership.join |
| `kick_beats_invite` | Layer 1: membership.kick wins over membership.invite |
| `same_type_falls_through_to_layer4` | Layer 1 produces no winner for same EventType conflict |
| `higher_role_wins_same_type_conflict` | Layer 4: admin beats member on concurrent state.room_name |
| `owner_beats_admin` | Layer 4: owner beats admin |
| `node_priority_respected` | Layer 5a: manual ordering produces correct winner |
| `lexicographic_backstop_always_resolves` | Layer 5c: always produces unique winner |
| `loser_event_stays_in_dag` | Loser is stored, not deleted |
| `resolution_is_deterministic` | Same input always produces same winner regardless of input order |

---

### Layer 13 — Pending Event Timeout
**Spec refs:** 3.9.6

This is a small addition to the existing `dag/pending.rs` module — not a new module.

**What to add:**

Each entry in the pending buffer must carry a `received_at` timestamp. A background task checks the pending buffer periodically (every 5 seconds is sufficient) and discards any entry whose `received_at` is more than **30 seconds** ago. On discard: log the pending Event ID and the missing predecessor IDs at `WARN` level. Emit error code 4002 (`predecessor_timeout`).

**Note:** 30 seconds is a work definition (WD-08) from the spec — do not hardcode the literal. Define it as a named constant `PENDING_TIMEOUT_SECS: u64 = 30` so it can be adjusted from one place.

**Tests:**

| Test | What it verifies |
|---|---|
| `pending_event_discarded_after_timeout` | Event held for >30s is removed from buffer |
| `pending_event_retained_within_timeout` | Event held for <30s is not discarded |
| `timeout_logs_missing_predecessor_ids` | Discard log contains the correct missing IDs |

---

### Layer 14 — DM Space Promotion
**Spec refs:** 3.16.1–3.16.4

Extends the existing `space/state.rs` module. No new module needed — add promotion logic alongside existing Space state handling.

**What to implement:**

1. **DM constraint enforcement** — on receiving any Event in a DM Space, the Node checks the DM constraints (3.16.1): max 2 members, max 1 Room, no federation, no invitations. Reject violating Events with appropriate error codes before they reach the DAG.

2. **Promotion sequence handler** — when the Node receives `dm.promote_propose`:
   - Validate: sender is one of the two DM Space members
   - Store the proposal (in-memory — not a DAG Event)
   - Deliver `dm.promote_propose` to the other member's connected client

3. When the Node receives `dm.promote_confirm`:
   - Validate: sender is the other member (not the proposer)
   - Match against stored proposal by `space_id`
   - Produce `state.dm_promote` Event signed by the Node keypair (not by either member)
   - Commit `state.dm_promote` to the Space DAG
   - Lift DM constraints immediately
   - Deliver `state.dm_promote` to both connected clients

4. When the Node receives `dm.promote_reject`:
   - Discard stored proposal
   - Notify proposing member

**Add to `SpaceState`:** a `dm_constraints_active: bool` field, set to `true` for `dm_space_create` Spaces and set to `false` when `state.dm_promote` is applied.

**Tests:**

| Test | What it verifies |
|---|---|
| `dm_space_rejects_third_member_invite` | DM constraint: invitation rejected |
| `dm_space_rejects_second_room` | DM constraint: second room creation rejected |
| `promote_propose_stored_and_delivered` | Proposal stored, delivered to other member |
| `promote_confirm_produces_dm_promote_event` | Confirm → state.dm_promote in DAG |
| `promote_signed_by_node_not_member` | state.dm_promote signature is Node's, not a member's |
| `dm_constraints_lifted_after_promotion` | Post-promotion: invitation accepted |
| `promote_reject_cancels_proposal` | Reject discards stored proposal |
| `history_preserved_after_promotion` | All pre-promotion Events remain in DAG |

---

### Layer 15 — Identity Replication
**Spec refs:** 3.13.1–3.13.6

Extends the existing `identity/` module. Add a new file `identity/replication.rs`.

**What to implement:**

1. **Outbound replication** — when a new Identity is registered on its home Node (after `accept_registration` succeeds), the Node selects up to N=3 replica Nodes from its federation registry (3.13.3 selection criteria: geographic diversity preferred, high announcement freshness, no existing replica, random from remaining) and pushes `identity.replicate` to each.

2. **Inbound replication handler** — when a Node receives `identity.replicate`:
   - Verify the signature (signed by the Identity's own keypair)
   - Check `update_version` — reject if lower than the stored version for this Identity
   - Store or update the Identity record
   - Respond with `identity.replicate_ack`

3. **Re-replication on update** — when an Identity record is updated (`identity.update`), the home Node re-pushes to all known replicas.

4. **Replica fallback** — when a client requests an Identity record (`identity.get`) and the Identity is not registered on this Node, the Node queries its known replicas before returning `identity.not_found`.

**Replication factor constant:** `REPLICATION_FACTOR: usize = 3` — define as a named constant (WD-19).

**Tests:**

| Test | What it verifies |
|---|---|
| `replicate_pushed_after_registration` | New Identity triggers replication to N nodes |
| `replication_respects_factor` | Never replicates to more than N replicas |
| `higher_update_version_accepted` | Incoming replicate with higher version updates record |
| `lower_update_version_rejected` | Stale replicate rejected |
| `replicate_ack_returned_on_success` | Successful replication returns ack |
| `replica_fallback_on_identity_get` | Unknown Identity triggers replica query |
| `re_replication_on_identity_update` | Identity update triggers re-push to replicas |

---

### Layer 16 — Space Migration Protocol
**Spec refs:** 3.12.1–3.12.8

Create `xgen-core/src/migration/` module. This is the most structurally complex Phase 2 layer — a full state machine with two sides (source Node and destination Node) running in parallel.

**Files to create:**

| File | Description |
|---|---|
| `migration/mod.rs` | Module declaration and public API |
| `migration/state_machine.rs` | Migration state machine — both source and destination sides |
| `migration/transfer.rs` | Event batch transfer logic |
| `migration/verification.rs` | Post-transfer integrity verification |

**Migration states (3.12.2):**

```
IDLE → NEGOTIATING → TRANSFERRING → VERIFYING → COMPLETE
                                              ↓
                                          FAILED
```

Both the source Node and destination Node track migration state independently. The state machine must be implemented for both sides.

**Implement in order:**

1. **Initiation** (3.12.3) — handle `migration.request` from the Space owner. Validate: sender is the Space owner. Open a federation connection to the destination Node. Send `migration.propose` with event count and estimated size.

2. **Acceptance handshake** — destination Node receives `migration.propose`. Validate: compatible protocol version, sufficient capacity (implementation-defined check). Respond with `migration.accept` or `migration.reject`. Source Node transitions to `TRANSFERRING` on accept.

3. **Event transfer** (3.12.5) — source Node sends Events in topological order as `migration.event_batch` messages. Batch size: 100 Events per message (implementation-defined — record in DECISIONS.md). Track any new Events produced during transfer as a tail batch. After all historical Events are sent, send tail Events, then `migration.complete`.

4. **Verification** (3.12.6) — destination Node replays all received Events through the full 13-step validation pipeline. Computes a Merkle root over all event_ids in topological order and compares with the source Node's declared root. On match: send `migration.verify_ok`. On mismatch: send `migration.verify_fail` — source Node retransfers failed batches.

5. **Completion** — on `migration.verify_ok`: source Node produces `state.space_migrate` Event in the Space DAG, notifies all Space members of the new Node endpoint, then enters a 48-hour read-only grace period before removing the Space.

6. **Abort handling** (3.12.7) — either party may send `migration.abort` at any point before `COMPLETE`. Source Node stays live. Destination Node discards all received Events for this migration. Both sides return to `IDLE`.

**Error codes:** 6xxx range for migration errors. Define as needed — record each new code in DECISIONS.md.

**Tests:**

| Test | What it verifies |
|---|---|
| `migration_requires_owner_auth` | Non-owner migration.request rejected |
| `migration_propose_sent_to_destination` | Source sends propose after request |
| `destination_reject_closes_migration` | Reject returns both sides to IDLE |
| `events_transferred_in_topological_order` | Transfer order is causally correct |
| `tail_batch_includes_events_during_transfer` | New Events during transfer captured |
| `verification_passes_on_valid_transfer` | Correct transfer → verify_ok |
| `verification_fails_on_tampered_event` | Tampered Event → verify_fail |
| `abort_discards_destination_state` | Abort → destination has no partial state |
| `full_migration_end_to_end` | Integration: Space fully migrated between two Nodes |

---

### Layer 17 — Bootstrap Node and Node Reputation
**Spec refs:** 3.14.1–3.14.5, 3.15.1–3.15.4

Create `xgen-core/src/bootstrap/` module. Bootstrap Nodes are ordinary Nodes with an additional declared capability — no special binary, no privileged protocol position.

**Files to create:**

| File | Description |
|---|---|
| `bootstrap/mod.rs` | Module declaration and public API |
| `bootstrap/capability.rs` | Bootstrap capability declaration and announcement extension |
| `bootstrap/directory.rs` | Bootstrap Node directory — in-memory store of known Nodes |
| `bootstrap/http.rs` | HTTP server for directory endpoint (uses `axum`) |
| `bootstrap/client.rs` | HTTP client for fetching remote Bootstrap Node directory (uses `reqwest`) |
| `bootstrap/reputation.rs` | Reputation record, score computation, propagation |

**What to implement:**

1. **Bootstrap capability** (3.14.1) — add `xgen.bootstrap` to the `capabilities` array in node announcement. Add `bootstrap_info` field to `NodeAnnouncement` struct: `directory_url`, `accepts_registrations`, `region`, `operator`.

2. **Node registration** (3.14.3) — when a Node with `xgen.bootstrap` capability receives `bootstrap.node_register`, add the registering Node to its directory. Respond with `bootstrap.node_register_ack`.

3. **Directory HTTP endpoint** (3.14.2) — Bootstrap Node serves its directory as a signed JSON document over HTTPS at the declared `directory_url`. Use `axum` for the HTTP server. The directory lists known Nodes ordered by `reputation_score` descending. Sign the directory document with the Bootstrap Node's keypair.

4. **Directory lookup** (3.14.4) — handle `bootstrap.node_lookup` from Nodes seeking peers. Return a filtered list from the directory. Also implement the HTTP client (`bootstrap/client.rs`) for fetching another Bootstrap Node's directory.

5. **Reputation record** (3.15.1) — implement the reputation record struct with all components: `uptime_ratio`, `announcement_freshness`, `defederation_count`, `successful_federations`, `failed_federations`, `protocol_violations`. Implement the score computation function: `score = sum(component × weight)` clamped to `[0.0, 1.0]`.

6. **Defederation signal** (3.15.3) — handle `reputation.defederation_signal` from other Nodes. Validate: the reporting Node is known. Increment `defederation_count` for the reported Node. Recompute score.

7. **Reputation propagation** (3.15.2) — Bootstrap Nodes periodically (default 6 hours, `REPUTATION_PROPAGATION_INTERVAL_HOURS: u64 = 6`) broadcast their reputation records to known Bootstrap Nodes. Merge incoming records using the weighted average rule: `merged = (local × 0.6) + (remote × 0.4)` per component.

**Note on HTTP vs WebSocket:** the Bootstrap Node directory endpoint is HTTP(S), not WebSocket. This is the only place in XGen where HTTP is used. The `axum` HTTP server runs alongside the existing WebSocket server — both bind on the same Node process but different ports. Record the port separation decision in DECISIONS.md.

**Tests:**

| Test | What it verifies |
|---|---|
| `bootstrap_capability_declared_in_announcement` | Capability present in node announcement |
| `node_register_adds_to_directory` | Registered Node appears in directory |
| `directory_signed_by_bootstrap_node` | Directory document carries valid signature |
| `lookup_returns_nodes_ordered_by_reputation` | Highest reputation first |
| `reputation_score_computed_correctly` | Known inputs → known score |
| `defederation_signal_increments_count` | Count increases, score decreases |
| `reputation_merge_applies_weights` | 60/40 merge rule applied correctly |
| `stale_announcement_reduces_freshness` | Old announcement_timestamp → lower freshness component |

---

### Layer 18 — End-to-End Encryption (MLS)
**Spec refs:** 3.10.1–3.10.9

This is the largest Phase 2 layer. Create `xgen-core/src/encryption/` module. The Node acts purely as an MLS Delivery Service — it routes MLS messages and stores encrypted blobs, but cannot decrypt any content.

Add `openmls`, `openmls_rust_crypto`, and `openmls_basic_credential` to `xgen-core/Cargo.toml`.

**Files to create:**

| File | Description |
|---|---|
| `encryption/mod.rs` | Module declaration and public API |
| `encryption/delivery_service.rs` | Node-side MLS Delivery Service — routes MLS messages |
| `encryption/key_package.rs` | KeyPackage storage and retrieval |
| `encryption/group.rs` | MLS group state tracking per Room (Node perspective only) |
| `encryption/client_mls.rs` | Client-side MLS group operations (join, send, receive) |

**What to implement — Node side (Delivery Service):**

1. **KeyPackage store** (3.10.3) — when a client sends `mls.key_package`, the Node stores the KeyPackage indexed by `(identity_id, room_id)`. When a new member is being added to a Room, the Node retrieves and distributes the appropriate KeyPackage. KeyPackages are one-time-use — delete after distribution.

2. **MLS message routing** (3.10.5) — the Node receives `mls.welcome`, `mls.commit`, `mls.proposal` from clients and routes them to the appropriate Room members. The Node does not inspect or modify the MLS message content — it routes the opaque bytes as-is.

3. **Encrypted content handling** — for `message.text` and other message Events, the `content` field carries an encrypted blob when E2E encryption is active. The Node stores and propagates this blob without decryption. The content field in the event_trace log is always empty for E2E-encrypted Events (this rule already exists — enforce it here).

4. **Epoch tracking** — the Node tracks the current MLS epoch number per Room (opaque counter, not the key material). This is used to detect stale Welcome messages.

**What to implement — Client side:**

1. **Group creation** (3.10.4) — when the Space owner creates a Room, the client initialises an MLS group using `openmls`. The client generates its own KeyPackage and uploads it to the Node.

2. **Member add flow** (3.10.5) — when a new member joins a Room, the client fetches the new member's KeyPackage from the Node, creates an MLS Proposal + Commit, and sends them via the Node. The Node delivers the MLS Welcome to the new member.

3. **Member remove flow** (3.10.6) — when a member leaves or is kicked, the remaining members advance the epoch by producing a Remove Proposal + Commit. This generates fresh encryption keys, ensuring the removed member cannot decrypt future messages.

4. **Message send** — client encrypts `content` using the current MLS epoch keys, wraps it in the standard `message.text` Event envelope, signs the envelope (signature covers the encrypted blob, not the plaintext), and sends to the Node.

5. **Message receive** — client receives an Event with encrypted `content`, decrypts using MLS group state, presents plaintext to the UI layer.

**Important constraint:** the MLS `openmls` crate manages all key material on the client side. The Node never sees MLS private keys. The `encryption/client_mls.rs` module is client-side only — it is used by `xgen-client` and by the Tauri client frontend, never by the Node runtime.

**Tests:**

| Test | What it verifies |
|---|---|
| `key_package_stored_and_retrieved` | Node stores and serves KeyPackage |
| `key_package_deleted_after_use` | One-time-use: KeyPackage removed after distribution |
| `mls_welcome_routed_to_new_member` | Welcome delivered to joining member |
| `node_cannot_decrypt_content` | Node has no access to plaintext — content is opaque bytes |
| `epoch_advances_on_member_join` | Group epoch increments after join commit |
| `epoch_advances_on_member_remove` | Group epoch increments after remove commit |
| `removed_member_cannot_decrypt_future_messages` | Post-removal messages use new epoch keys |
| `mls_round_trip` | Integration: Alice sends encrypted message → Bob decrypts correctly |
| `encrypted_content_not_logged` | event_trace content field empty for E2E messages |

---

### Layer 19 — Auth Module Tier 2–4 Interfaces
**Spec refs:** 3.11.1–3.11.5

**Scope:** interface definitions and slot contract extensions only. XGen ships Tier 1 as the only reference implementation. Tiers 2–4 are built in institutional collaboration with qualified organisations. This layer defines the contracts those organisations build against — not the verification logic itself.

Create `xgen-core/src/auth/tiers.rs` alongside the existing Tier 1 auth implementation.

**What to implement:**

1. **Tier model extension** — extend the existing `AuthTier` enum to include `Tier2`, `Tier3`, `Tier4`.

2. **Trust Assertion claims extension** (3.11.2–3.11.4) — add Tier 2, 3, and 4 claim structs that extend the base `claims` object from Tier 1. Each struct carries the additional fields defined in the spec. These structs are used for parsing and validating incoming Trust Assertions — the Node validates the signature and checks that the claimed fields are present, but does not perform the underlying verification (that is the Auth Module's responsibility).

Tier 2 additional claims: `tier_verified`, `legal_name_verified`, `organisation_verified`, `organisation_domain`, `iso27001_operator`.

Tier 3 additional claims: all Tier 2 claims plus `aml_kyc_cleared`, `corporate_role_verified`, `audit_trail_maintained`, `regulatory_compliance`.

Tier 4 additional claims: all Tier 3 claims plus `security_clearance_level`, `jurisdiction`, `hardware_token_bound`, `biometric_verified`.

3. **Slot contract enforcement** — extend the existing `verify_trust_assertion()` function to check that a Trust Assertion's `tier_verified` claim is equal to or higher than the Space's declared `auth_tier`. Reject with appropriate error code if not.

4. **TTL rules** — Tier 2: 1 year. Tier 3: 6 months. Tier 4: 3 months. Add these as named constants. (All work definitions — WD-09 through WD-11.)

**No verification logic is implemented here.** The Trust Assertion is signed by an external Auth Module. The Node verifies the signature. The content of the claims is trusted if the signature is valid — the Node does not independently re-verify legal names, ISO certifications, or security clearances. That is the Auth Module's domain.

**Tests:**

| Test | What it verifies |
|---|---|
| `tier2_claims_parsed_correctly` | Tier 2 Trust Assertion fields deserialise |
| `tier3_claims_parsed_correctly` | Tier 3 Trust Assertion fields deserialise |
| `tier4_claims_parsed_correctly` | Tier 4 Trust Assertion fields deserialise |
| `tier_mismatch_rejected` | Tier 1 assertion rejected in Tier 2 Space |
| `higher_tier_accepted_in_lower_space` | Tier 3 assertion accepted in Tier 2 Space |
| `tier2_ttl_enforced` | Expired Tier 2 assertion rejected |

---

## Testing Strategy

Each layer has its own unit tests run immediately after implementation. Do not advance to the next layer until all tests for the current layer pass.

**Per-component smoke tests** — after each layer, use the `--batch` flag with `.xgb` command files to exercise the new functionality against running binaries. This validates that the in-process unit tests reflect real binary behaviour. See `docs/xgen_appendix_f_en.md` §F.8 for batch command format.

**Stress tests** — after all layers (11–19) are implemented and unit tests pass, run stress tests equivalent to the Phase 1 stress test suite, extended to cover Phase 2 scenarios: concurrent state conflicts under load, MLS group operations at scale, migration under message traffic.

**Full integration test** — the Phase 2 definition of done. Run both `xgen-node` and `xgen-client` as complete compiled binaries against each other. Exercise the full Phase 2 feature set end-to-end: E2E encrypted message exchange, DM promotion, Space migration between two live Nodes, Identity replication across three Nodes, Bootstrap Node discovery. Both Phase 1 and Phase 2 features must work together correctly. The Phase 1 17-step smoke test must still pass unchanged.

---

## Error Code Ranges

Error codes are plain integers on the wire. The domain occupies the leading digits; the specific code occupies the last three digits.

| Range | Domain |
|---|---|
| 1000–1999 | Transport |
| 2000–2999 | Federation |
| 3000–3999 | Identity |
| 4000–4999 | State resolution (new in Phase 2) |
| 5000–5999 | E2E encryption (new in Phase 2) |
| 6000–6999 | Space migration (new in Phase 2) |
| 7000–7999 | Bootstrap (new in Phase 2) |
| 8000–8999 | Reputation (new in Phase 2) |
| 9000–9999 | DM promotion (new in Phase 2) |

Future domains beyond 9 follow the same pattern naturally: domain 10 uses 10000–10999, domain 534 uses 534000–534999. No wire format change is needed — integers handle any size.

**Display convention:** error codes MAY be displayed with a zero-padded `E` prefix for readability in logs, UI, and documentation. The padding is always 6 digits. Examples: `E001001`, `E004002`, `E534001`. The `E` prefix and zero-padding are display-only — they are never transmitted on the wire, never used as exit codes, and never used in programmatic comparisons. An implementation displaying `E002453` and one transmitting `2453` as a wire integer or exit code are referring to the same error.

**Display rule** (same pattern as Phase 1, extended):

```
Error E004002 (predecessor_timeout): Pending Event discarded — missing predecessors
not received within the 30-second window.
```

Define specific codes within each range as needed during implementation. Record every new error code in DECISIONS.md.

---

## Implementation Decision Log — Mandatory Rule

Same rule as Phase 1: every implementation decision beyond what the spec prescribes must be recorded in `DECISIONS.md` before moving to the next layer. Phase 2 decisions begin at D-044 (D-044 is reserved for the xgen-core crate split — see `docs/tests/XGEN_CORE_SPLIT_ph2.md`). Phase 2 specific decisions start at D-045.

---

## Spec Cross-Reference Quick Index

| Topic | Spec section |
|---|---|
| State resolution algorithm | 3.9.3 |
| Convergence guarantee | 3.9.2 |
| Pending Event timeout | 3.9.6 |
| State snapshot update rule | 3.9.7 |
| State resolution error codes | 3.9.8 |
| MLS encryption model | 3.10.1 |
| MLS group → XGen Room mapping | 3.10.2 |
| KeyPackage management | 3.10.3 |
| MLS group creation | 3.10.4 |
| MLS handshake routing | 3.10.5 |
| Auth Tier model | 3.11.1 |
| Tier 2 claims | 3.11.2 |
| Tier 3 claims | 3.11.3 |
| Tier 4 claims | 3.11.4 |
| Space migration who can trigger | 3.12.1 |
| Migration state machine | 3.12.2 |
| Migration initiation sequence | 3.12.3 |
| Migration event transfer | 3.12.5 |
| Migration verification | 3.12.6 |
| Migration abort | 3.12.7 |
| Identity replication model | 3.13.1 |
| Replication factor N=3 | 3.13.2 |
| Replica Node selection | 3.13.3 |
| Replication wire protocol | 3.13.4 |
| Bootstrap Node capability | 3.14.1 |
| Bootstrap directory format | 3.14.2 |
| Node registration with Bootstrap | 3.14.3 |
| Bootstrap directory lookup | 3.14.4 |
| Reputation signal structure | 3.15.1 |
| Reputation propagation | 3.15.2 |
| Defederation signal | 3.15.3 |
| DM Space constraints | 3.16.1 |
| DM promotion sequence | 3.16.3 |
| DM history preservation | 3.16.4 |

---

## Phase 2 Definition of Done

- [ ] All Phase 1 tests still passing (173/173)
- [ ] Layer 11–19 unit tests all passing
- [ ] Per-component batch smoke tests passing for each layer
- [ ] Stress tests passing
- [ ] Full integration test passing — both binaries, Phase 1 + Phase 2 features together
- [ ] All new error codes defined and recorded in DECISIONS.md
- [ ] All implementation decisions recorded in DECISIONS.md (D-045+)
- [ ] CLAUDE.md updated to reflect Phase 2 complete
- [ ] JOURNAL.md entries written for each session
- [ ] Version tag applied
