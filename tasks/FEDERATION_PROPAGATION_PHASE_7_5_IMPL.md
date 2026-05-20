# Federation Event Propagation — Phase 7.5 Implementation Runbook (Cold-Start Bootstrap)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-20  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This is the **implementation runbook** for Phase 7.5 of the Federation Event Propagation milestone. The design phase closed 2026-05-19 with four framework decisions (P7.5-A through P7.5-D) locked at `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` (Status: COMPLETED v1.0 — flipped in Commit 1 of this runbook). This document is Clair-facing — it sequences code-level work to ship Phase 7.5 across five atomic commits.

The design task file is authoritative on **what** to build and **why**. This runbook is authoritative on **how** to ship it, **in what order**, with **what verification at each step**.

**Reading order on session start:**
1. This document, §2 (sequence overview) — get the shape of the five commits.
2. Design task file §5–§8 — re-read the four locks before touching code.
3. Canonical design doc `docs/xgen_federation_propagation_design.md` §6 (F-3) — re-read Phase 7's existing B1 skip so the new Phase 7.5 skip is a clean sibling.
4. Then back to this document, §3 onward, for per-commit work.

**Latitude reminder.** Implementation-internal decisions (data-structure field names, internal function signatures, test helper shapes, module organisation, SQLite-table naming, internal refactors that preserve wire shape) are Clair's latitude per design doc §9.2. Wire-format-visible or trust-model-visible decisions require Joe-lock — pause and ask. Concrete starting suggestions below are exactly that: starting points. Clair may revise if a cleaner option surfaces during implementation.

---

## 2. Sequence overview

Five atomic commits, in this order. Each commit is shippable in isolation (tests pass at each step). Hard ordering: Commit 1 first (so the canonical design doc reflects the locked design before any code references it); Commit 5 last (so the milestone-close housekeeping happens after all code has shipped and verified).

| # | Commit | Scope | Test count change |
|---|---|---|---|
| 1 | Doc-pass | Canonical design doc §6.4.1 + §15 row; design task file flipped COMPLETED | 519 → 519 (no code) |
| 2 | F-4 step 1 + F-3 skip | Skip rules for `state.space_create` + `state.dm_space_create` in unified validation core | 519 → 519+N (new unit tests) |
| 3 | HeldPending third trigger | Data structure, arrival hook, drain helper, config field, error code 4007, observability counter, `SpaceLocalMetadata` structure | 519+N → 519+N+M (new unit tests) |
| 4 | Integration tests | NodeRuntime-level cold-start, mid-bootstrap drop/resume, F-10 + Phase-7.5 combination | 519+N+M → 519+N+M+K (new integration tests) |
| 5 | Phase 7.5 close | CLAUDE.md PLAY block, ROADMAP.md Past entry, milestone-internal status flip | unchanged from Commit 4 |

Phase 9 resumes from Commit 3 boundary **after** Commit 5 closes. XGID work sits between Phase 7.5 close and Phase 9 resume per ROADMAP.md Near future — **do NOT implement XGID during Phase 7.5 implementation.**

**Test-count discipline.** N, M, K are not pre-locked. Each commit's DoD requires actual `cargo test --workspace` output quoting the new count. Do not invent numbers (CLAUDE.md Rule 5).

**Two pre-existing flakes carried forward** (from CLAUDE.md): precedence env-var race (D-068, commit 3e2f311); `reconnect_with_existing_tip_small_delta_delivered` (Phase 3 test under workspace parallelism). If either fires during Phase 7.5 verification, retry once to confirm flake signature; do not treat as regression.

---

## 3. Commit 1 — Doc-pass commit

### 3.1 Scope

This commit is documentation only. No code changes, no test changes. The purpose: make the canonical design doc and the design task file reflect the Joe-locked state of Phase 7.5 before any implementation work begins. This is the canonical-document discipline from D-069.

### 3.2 Files touched

- `docs/xgen_federation_propagation_design.md` — add §6.4.1 sibling subsection; add §15 Implementation Complete table row for Phase 7.5.
- `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` — header Status flipped ACTIVE → COMPLETED, Last updated bumped to commit date.

### 3.3 §6.4.1 content sketch

The canonical design doc's §6.4 currently covers Phase 7's F-3 framework with Lock A1 (data source: `SpaceState.federation_nodes`), Lock B1 (skip for `state.federation_add`), and Lock B2 (self-establishing tightening deferred). §6.4.1 is a sibling subsection — same depth, same prose style — covering Phase 7.5's four locks. Read §6.4 first and match its tone.

Required content in §6.4.1:

1. **One-paragraph framing.** Phase 7.5 closes the cold-start bootstrap chicken-and-egg that surfaced during Phase 9 Scenario 1 setup (failure-mode catalogue M5). Reference the design task file for full provenance.
2. **P7.5-A summary.** Narrow skip rule for `state.space_create` and `state.dm_space_create` EventTypes at F-4 step 1 AND F-3. Sibling to B1's skip for `state.federation_add`. Authority preserved via signature verification through F-10 HeldPending for unknown-signer case. Mention the new `SpaceLocalMetadata` sibling structure with `introducer_node_id` field as the in-scope structural addition.
3. **P7.5-B summary.** Third HeldPending trigger condition: "missing federation relationship for (peer, space)". Resolved by idempotent `state.federation_add` arrival hook. New error code `4007 federation_relationship_timeout`. Combination semantics with F-4a (predecessor) and F-10 (Identity) via existing struct-variant Option fields. Precedence ranking: predecessor (4002) > federation-relationship (4007) > Identity (4006).
4. **P7.5-C summary.** Per-trigger timeout. Predecessor + Identity stay at 30s. Federation-relationship defaults to 180s with new `[sync].federation_relationship_timeout_seconds` config field. Brings F-10a's v2 evolution path forward to v1.
5. **P7.5-D summary.** New `pending_federation_relationship: usize` counter in `xgen-node_state.json`. Existing `f3_reject` trace event extended with disposition field (`rejected` vs `held_pending`) — not renamed. Introducer field NOT exposed in state file (queryable via SQL only until M6 operator CLI).
6. **Forward-reference to the design task file** for the full reasoning trail.

Do not duplicate the design task file's reasoning prose. §6.4.1 is a load-bearing summary that names the locks and points at the design task file for the why. Five-to-eight paragraphs total is the right length.

### 3.4 §15 Implementation Complete table row

§15 currently records Phases 1–8 (J-082 through J-089). Add a Phase 7.5 row in chronological position (between Phase 8 and where Phase 9 will eventually land). Format matching existing rows: Phase 7.5 | Cold-Start Bootstrap | implementation commits | tests | journal entry reference.

Journal entry reference will not exist yet at Commit 1 time (J-094 or later writes the journal entry for the Phase 7.5 implementation session). Use `J-094+` or similar placeholder; update in Commit 5 when the actual journal number is known. Add an explicit `[PENDING]` marker in the row so the placeholder is visible.

### 3.5 Design task file flip

`tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` header:
- Status: ACTIVE → COMPLETED
- Last updated: bump to commit date, with one-sentence narrative noting design phase closure and the runbook handoff.

No body changes to the design task file — its content is frozen as the locked authoritative record.

### 3.6 Per-commit DoD checklist

- [ ] §6.4.1 added to canonical design doc, five-to-eight paragraphs, covering all four locks at summary depth.
- [ ] §15 row added with `[PENDING]` journal-number placeholder.
- [ ] Design task file Status flipped COMPLETED, Last updated bumped.
- [ ] Canonical design doc header `Last updated` bumped.
- [ ] `cargo test --workspace` run, actual output quoted in journal entry, count = 519 (no change expected).
- [ ] Both files' headers verified to have two trailing spaces on every `>` line.

---

## 4. Commit 2 — F-4 step 1 + F-3 skip rules for Space-create EventTypes

### 4.1 Scope

Implement P7.5-A's skip rules at two sites in the unified validation core. After this commit, `state.space_create` and `state.dm_space_create` events arriving over a federation session land cleanly on a receiver that has no local record of the target Space.

### 4.2 Files touched (expected)

- `xgen-core/src/node/runtime.rs` — `dispatch_event` and/or its F-3 step (currently around the runtime.rs:349 area per Phase 7's notes in CLAUDE.md).
- `xgen-core/src/message/exchange.rs` — `validate_event` step 1 / step 2, wherever F-4 step 1's "Space exists" check lives. Confirm exact site by reading current code; CLAUDE.md notes the legacy `validate_steps_8_13` path is test-only post-Phase-6.
- `xgen-common/src/state.rs` — new `SpaceLocalMetadata` structure (suggested module/location; Clair latitude).
- Storage layer — new SQLite table for `SpaceLocalMetadata` persistence. Suggested table name `space_local_metadata`; suggested module location alongside existing Space-related storage in xgen-core or xgen-node (depending on where Space-state persistence already lives — confirm by reading current code).
- Unit tests in the same crates.

### 4.3 What to do

**F-3 skip site** (in `dispatch_event`):

The existing F-3 check, per Phase 7's Lock B1, already skips for `EventType::StateFederationAdd`. Extend the same skip to cover `EventType::StateSpaceCreate` and `EventType::StateDmSpaceCreate`. The skip is structural: these EventTypes by definition create the Space they reference, so `SpaceState.federation_nodes[space]` cannot exist yet.

Suggested code shape (pseudocode — Clair will write the real Rust):

```
// Verbatim code-comment block:
// Phase 7.5 §5 — F-3 skip extension for Space-create EventTypes.
// state.space_create and state.dm_space_create by structural necessity
// bring the Space into existence; the federation_nodes check cannot apply
// to them. Sibling to Phase 7 Lock B1 (federation_add skip). Signature
// verification is NOT skipped — only the structural federation-relationship
// check. Unknown-signer case is covered by F-10 HeldPending.
let skip_f3 = matches!(
    event.event_type,
    EventType::StateFederationAdd
        | EventType::StateSpaceCreate
        | EventType::StateDmSpaceCreate
);
```

**F-4 step 1 skip site** (in the validation core's "Space exists locally" check):

Same shape. The two new EventTypes are skipped at step 1's Space-existence lookup; the validation core proceeds. `EventType::StateFederationAdd` is NOT skipped at F-4 step 1 (it still requires the target Space to exist locally — the case where federation_add arrives before space_create is what HeldPending in Commit 3 handles).

Suggested code-comment block:

```
// Phase 7.5 §5 — F-4 step 1 skip for Space-create EventTypes.
// state.space_create and state.dm_space_create create the Space they
// reference; the Space-exists check cannot apply to them. This skip is
// narrower than the F-3 skip above — it does NOT extend to
// state.federation_add, which still requires the target Space to exist
// locally (the federation_add-arrives-before-space_create case is
// handled by HeldPending in Phase 7.5 §6).
```

**`SpaceLocalMetadata` structure** (per design doc §5.3 + §5.6):

```rust
/// Local-only metadata about a Space, sibling to SpaceState.
/// Populated at Space-create ingestion via federation; never modified afterward.
/// Persisted to dedicated local SQLite table; not federated, not in event log.
pub struct SpaceLocalMetadata {
    pub space_id: String,
    pub introducer_node_id: Option<String>,  // Name locked through any future XGID-typing pass
    pub introduced_at: chrono::DateTime<chrono::Utc>,  // RFC 3339 UTC per project convention
}
```

The structure is a sibling to `SpaceState`, not a field on it — this preserves SpaceState's invariant that all its content is derived from federated events. Population happens at `state.space_create` / `state.dm_space_create` ingestion, in the F-3-skip code path: when the skip fires AND origin is `ReceivedViaFederation`, populate `introducer_node_id` from the peer ID; when origin is `LocallySubmitted`, leave it `None` (local creation has no introducer). Once populated, never modified. SQLite UNIQUE constraint on `space_id` ensures append-only-on-create semantics.

### 4.4 Tests

Suggested unit tests (in the same modules):

- `f3_skips_state_space_create_from_federation` — ReceivedViaFederation origin, F-3 does not reject, event flows through.
- `f3_skips_state_dm_space_create_from_federation` — same for the DM variant.
- `f3_does_not_skip_state_room_create` — negative test confirming the skip is narrow (state.room_create still rejected without federation relationship).
- `f4_step1_skips_state_space_create` — Space does not exist locally; F-4 step 1 does not reject `state.space_create`.
- `f4_step1_skips_state_dm_space_create` — same for DM.
- `f4_step1_does_not_skip_state_federation_add` — negative test confirming `state.federation_add` still requires Space to exist (HeldPending in Commit 3 covers the deferred case, not a step-1 skip).
- `space_local_metadata_populated_on_federation_space_create` — `introducer_node_id` set from peer ID.
- `space_local_metadata_introducer_none_on_local_space_create` — local creation leaves field None.
- `space_local_metadata_immutable_after_create` — second `state.space_create` for same space_id does NOT update introducer (idempotency at the ingestion layer, plus SQLite UNIQUE constraint as belt-and-braces).

### 4.5 Per-commit DoD checklist

- [ ] F-3 skip code shipped with verbatim code-comment block citing Phase 7.5 §5.
- [ ] F-4 step 1 skip code shipped with verbatim code-comment block citing Phase 7.5 §5.
- [ ] `SpaceLocalMetadata` structure shipped in `xgen-common` (or wherever Clair lands it; field name `introducer_node_id` locked).
- [ ] SQLite table for `SpaceLocalMetadata` created with UNIQUE constraint on `space_id`.
- [ ] Population logic wired at Space-create ingestion path, federation-only.
- [ ] All suggested unit tests above written; passing.
- [ ] `cargo test --workspace` clean; actual output quoted in journal entry with new total.
- [ ] Headers bumped on any file edited.

---

## 5. Commit 3 — HeldPending third trigger + config + error code + observability

### 5.1 Scope

Implement P7.5-B (new HeldPending trigger), P7.5-C (180s timeout + config field), and P7.5-D (observability counter + trace-event disposition extension). After this commit, events arriving for a Space that exists locally but for which the (peer, space) federation relationship has not yet been established are held pending `state.federation_add` arrival rather than rejected.

This is the largest commit in the sequence. Clair may consider splitting it into sub-commits if testing each piece in isolation is cleaner — that's implementation latitude. Recommended split if any: 3a (data structure + drain helper, no integration yet) + 3b (arrival hook + observability + config). The DoD below assumes a single commit; adapt the checklist if split.

### 5.2 Files touched (expected)

- `xgen-core/src/dag/pending.rs` — `PendingBuffer` data structure extended; new `missing_federation_relationship: Option<(String, String)>` field on held entries; new `waiting_for_federation_relationship: HashMap<(peer, space), HashSet<event_id>>` secondary index (analogous to F-10's `waiting_for_identity`); new `resolve_federation_relationship` method; `try_release` / `pending_federation_relationship_count` extensions.
- `xgen-core/src/node/runtime.rs` — F-3 reject path on bootstrap case routes to HeldPending instead of permanent reject; new `drain_pending_by_federation_relationship` method (analogous to `drain_pending_by_identity`); timeout sweep extended with predecessor-code-wins precedence (4002 > 4007 > 4006).
- `xgen-node/src/app.rs` — `state.federation_add` ingestion path gains arrival-hook call to `drain_pending_by_federation_relationship` (idempotent — fires on every successful ingestion, not only first).
- `xgen-core/src/message/exchange.rs` — `ValidationOutcome::HeldPending` struct variant extended with the new `missing_federation_relationship: Option<(String, String)>` field; `TimedOut` variant extended with `missing_federation_relationship: Option<(String, String)>`.
- `xgen-core/src/resolution/mod.rs` — new error code `4007 federation_relationship_timeout` (next-free after 4006 per CLAUDE.md namespace verification rule). Verify next-free at implementation time.
- `xgen-common/src/state.rs` — new `pending_federation_relationship: usize` field on `NodeState` with `#[serde(default)]`; `build_node_state` populates it from `PendingBuffer::pending_federation_relationship_count()`.
- `xgen-common/src/config.rs` or equivalent — new `[sync].federation_relationship_timeout_seconds` field, default 180. Confirm exact file by reading current config-struct location.
- Trace-event emission site for `f3_reject` (per CLAUDE.md, lives in `xgen-core/src/node/runtime.rs`'s F-3 reject path) — extend with new `disposition` field, value `rejected` or `held_pending`. Phase 9 Commit 1's existing field set stays unchanged; this is purely additive.
- Unit tests in the same modules.

### 5.3 Key implementation details

**HeldPending struct extension.** Per design §6.3, the struct already supports `Option<...>` per trigger. Phase 6's `missing_identity: Option<String>` is the precedent. Phase 7.5 adds `missing_federation_relationship: Option<(String, String)>` (peer_node_id, space_id). An event missing both Identity and federation relationship has BOTH fields populated; resolution requires BOTH arrivals.

**Idempotent arrival-hook semantics.** Per design §6.3: the federation_add arrival hook fires on **every** successful ingestion of `state.federation_add` for (peer, space), not only the first. Each fire scans HeldPending for matching entries and attempts re-validation; no-op if nothing matches. Mirrors F-10's Identity-arrival hook semantics. No "already-drained pairs" tracking required.

**Precedence at timeout.** Per design §6.3:
- If `missing_predecessors` non-empty at timeout → emit `4002 predecessor_timeout` (F-4a unchanged).
- Else if `missing_federation_relationship` populated → emit `4007 federation_relationship_timeout` (Phase 7.5 new).
- Else if `missing_identity` populated → emit `4006 identity_record_timeout` (F-10 unchanged).
- Final precedence: 4002 > 4007 > 4006.

Verbatim code-comment block at the timeout-emit site (sibling to Phase 6's block at the same site):

```
// Phase 7.5 §6.3 — predecessor-code-wins precedence extended.
// Final precedence: 4002 (predecessor) > 4007 (federation-relationship) > 4006 (Identity).
// Rationale: federation-relationship is the most upstream blocker in the
// dependency chain because Identity replication is conditionally downstream
// of federation establishment (Identity events themselves flow over
// federation transport). Reporting the most upstream blocker directs the
// operator to the right diagnostic question.
```

**Two-stage cascade.** Per design §6.3: if `state.federation_add` itself enters HeldPending on F-10's Identity trigger (federation_add's signer Identity isn't yet replicated), the federation-relationship hook cannot fire for events waiting on the same (peer, space) until federation_add itself drains. This resolves naturally without special handling — each hook responds to its own trigger; no cross-hook coordination is needed. Integration tests in Commit 4 cover this case explicitly.

**Trace event disposition field** (P7.5-D). Existing `f3_reject` trace event in `runtime.rs` gains a new field `disposition` with value `"rejected"` (the dominant non-bootstrap case, F-3 permanent reject) or `"held_pending"` (Phase 7.5's new path, F-3 deferred via HeldPending). Suggested field shape: `disposition: &'static str`. Phase 9 Commit 1's other stable `event = "..."` fields stay unchanged.

**Config field.** `[sync].federation_relationship_timeout_seconds: u64`, default `180`. Loaded into `SyncConfig` (or equivalent), passed through to PendingBuffer or wherever timeout sweep reads its timeouts. Confirm timeout-sweep architecture by reading Phase 6's pattern for `identity_record_timeout` — Phase 7.5's federation-relationship timeout reads from the same config-struct shape with a sibling field.

**Observability counter.** `pending_federation_relationship: usize` on `NodeState`. Computation: `sum(buf.pending_federation_relationship_count() for buf in self.pending.values())` — same pattern as F-10's `pending_identity_replication`. `#[serde(default)]` on the field for forward-compat with pre-Phase-7.5 state files.

### 5.4 Tests

Suggested unit tests:

- `held_pending_buffers_event_with_missing_federation_relationship` — F-3 fails for federation-channel event with peer not in federation_nodes; event enters HeldPending with `missing_federation_relationship: Some((peer, space))`.
- `federation_add_arrival_hook_drains_pending_events` — held event drains after federation_add ingestion for matching (peer, space).
- `federation_add_arrival_hook_idempotent` — second federation_add for same (peer, space) is a no-op; no panic, no double-drain.
- `federation_add_arrival_hook_for_different_pair_does_not_drain` — federation_add for (peer, otherspace) does not drain entries waiting on (peer, space).
- `held_pending_with_both_identity_and_federation_relationship_missing` — both fields populated; resolution requires both arrivals.
- `held_pending_timeout_fires_with_4007_when_federation_relationship_missing` — timeout fires after 180s (test injects shorter duration); error code is 4007.
- `held_pending_timeout_predecessor_wins_over_federation_relationship` — both missing at timeout → 4002 emitted.
- `held_pending_timeout_federation_relationship_wins_over_identity` — both missing at timeout → 4007 emitted.
- `pending_federation_relationship_counter_populated` — counter equals sum across Spaces.
- `config_federation_relationship_timeout_seconds_default_180` — config default verified.
- `trace_event_f3_reject_disposition_held_pending` — trace event includes disposition field with value `held_pending` when the path is the new one; `rejected` when it's the existing permanent-reject path.

### 5.5 Per-commit DoD checklist

- [ ] `PendingBuffer` data structure extended with new field + secondary index + methods.
- [ ] `ValidationOutcome::HeldPending` + `TimedOut` struct variants extended.
- [ ] `drain_pending_by_federation_relationship` shipped (analogous to F-10's `drain_pending_by_identity`).
- [ ] Arrival hook wired in `state.federation_add` ingestion path; idempotency verified by test.
- [ ] Error code `4007 federation_relationship_timeout` added with namespace next-free verified.
- [ ] Timeout sweep precedence (4002 > 4007 > 4006) shipped with verbatim code-comment block.
- [ ] Two-stage cascade verified by test (federation_add itself held on F-10, drains naturally on Identity arrival, then dependent events drain on federation_add arrival).
- [ ] `pending_federation_relationship: usize` counter on `NodeState` shipped + populated + `#[serde(default)]` for forward-compat.
- [ ] `[sync].federation_relationship_timeout_seconds` config field shipped with default 180.
- [ ] `f3_reject` trace event extended with disposition field (NOT renamed).
- [ ] All suggested unit tests above written; passing.
- [ ] `cargo test --workspace` clean; output quoted in journal entry with new total.
- [ ] Headers bumped on any file edited.

---

## 6. Commit 4 — Integration tests at NodeRuntime level

### 6.1 Scope

NodeRuntime-level integration tests that exercise the full cold-start bootstrap path end-to-end. These are the tests Phase 9 Scenario 1 setup could not get past; they prove Phase 7.5 closes failure-mode catalogue M5.

Tests at NodeRuntime level (not deployment level — deployment tests are Phase 9's job). NodeRuntime-level means: in-process Node A and Node B, real federation handshake, real `process_inbound` pipeline, real PendingBuffer, but no TCP / WebSocket / binary spawning. Same pattern as Phase 6's `heldpending_identity_integration.rs` and Phase 7's `federation_relationship_integration.rs`.

### 6.2 Files touched (expected)

- New module `xgen-node/src/tests/cold_start_bootstrap_integration.rs` (suggested name; Clair latitude). Or new submodule under existing federation-related test module.
- Possibly extensions to existing test helpers if cold-start setup needs new helpers (e.g., a "brand-new Node B with no Space S" builder).

### 6.3 Required scenarios

**Scenario A — Cold-start bootstrap end-to-end.** Brand-new Node B receives Space S from Node A via federation handshake. Stream order: `state.space_create`, `state.room_create`, `membership.invite`, `state.federation_add`, `membership.join`, then a message. After stream completes: Space S exists on B; `SpaceLocalMetadata` for S has `introducer_node_id = Some(A)`; all events ingested; no HeldPending entries remain; F-3 passes on subsequent federation traffic for (A, S).

**Scenario B — Mid-bootstrap session drop and resume.** Federation handshake delivers `state.space_create` + several intermediate events; session drops before `state.federation_add` arrives. Held events remain in PendingBuffer. New session establishes; F-1a tip exchange re-delivers the remaining stream including `state.federation_add`. Held events drain on federation_add arrival in the new session. Idempotency of arrival hook verified (no double-processing).

**Scenario C — Combination with F-10 (both missing).** Cold-start where Identity records for some signers have not yet replicated. Events enter HeldPending with BOTH `missing_identity` and `missing_federation_relationship` populated. Identity arrives first via separate Identity-replication path; event re-validates but still held on federation-relationship trigger. federation_add arrives second; event drains and ingests.

**Scenario D — Two-stage cascade.** `state.federation_add` itself is held in PendingBuffer on F-10's Identity trigger (federation_add's signer Identity isn't yet replicated). Events depending on the federation relationship are also held. Identity arrives → F-10 hook drains federation_add → federation_add ingests → P7.5-B hook drains dependent events. Verifies no cross-hook coordination needed (each hook fires on its own trigger).

**Scenario E — Timeout precedence at NodeRuntime level.** Inject a short timeout (e.g., 1s) via test-only config override. Bootstrap stream that never delivers federation_add → held events time out with `4007 federation_relationship_timeout`. Bootstrap stream where predecessors are also missing → held events time out with `4002 predecessor_timeout` (predecessor wins precedence).

**Scenario F — Negative regression.** Non-cold-start federation traffic (Space S already exists, peer already federated) flows through F-3 / F-4 unchanged. No HeldPending entries created for federation-relationship trigger. Trace event `f3_reject` has disposition `"rejected"` (not `"held_pending"`) for any actual non-bootstrap rejections — confirms the Phase-7.5 path is narrow and doesn't catch normal traffic.

### 6.4 Test infrastructure

Existing NodeRuntime-level test patterns from Phase 6 and Phase 7 are the templates. Read `xgen-node/src/tests/heldpending_identity_integration.rs` and `xgen-node/src/tests/federation_relationship_integration.rs` before authoring; copy structural patterns rather than reinventing.

If a test needs to inject a short timeout for Scenario E, prefer a config override over a test-only constant: keeps test infrastructure honest about production config shape.

### 6.5 Per-commit DoD checklist

- [ ] Scenario A passing.
- [ ] Scenario B passing.
- [ ] Scenario C passing.
- [ ] Scenario D passing.
- [ ] Scenario E passing (both 4007 and 4002 precedence paths).
- [ ] Scenario F passing (negative regression — no false positives on normal traffic).
- [ ] `cargo test --workspace` clean; output quoted in journal entry with new total.
- [ ] Two pre-existing flakes (precedence env-var race, reconnect_with_existing_tip_small_delta_delivered): if either fires, retry once to confirm flake signature; do not treat as regression.
- [ ] Headers bumped on any file edited.

---

## 7. Commit 5 — Phase 7.5 close commit

### 7.1 Scope

Milestone-internal close. After this commit, Phase 7.5 is shipped; XGID work is next-active per ROADMAP.md Near future; Phase 9 resumes from Commit 3 boundary after XGID work closes.

This commit is documentation only — same shape as Commit 1, no code changes.

### 7.2 Files touched

- `CLAUDE.md` — PLAY block flipped from "Phase 7.5 Federation Cold-Start Bootstrap implementation runbook authoring" to whatever the next state is. If XGID is the next-active item, PLAY reflects XGID concept work. If XGID is still in design-phase pending, PLAY reflects the gap between Phase 7.5 close and Phase 9 resume with XGID as the named blocker. Header `Last updated` bumped.
- `docs/ROADMAP.md` — Past section gains Phase 7.5 implementation closure entry; Present section reflects the new next-active item (XGID or its design phase); Phase 9 status reflects "ready to resume after XGID closes".
- `docs/xgen_federation_propagation_design.md` — §15 row's `[PENDING]` journal-number placeholder updated with the actual journal number for the Phase 7.5 implementation session (J-094 or whatever it lands as).
- `JOURNAL.md` — new journal entry written for the Phase 7.5 implementation session, covering all five commits with actual test counts and any in-session findings.

### 7.3 Special note on PLAY block content

Clair authors a draft of the new PLAY block; Joe reviews and may revise. Discussion in chat is expected here — the PLAY block sets the next session's orientation, so the framing matters.

If Phase 7.5 implementation surfaced any findings worth recording (anything that should land in DECISIONS.md, JOURNAL.md commentary, or future-milestone carry-over notes), those go in the journal entry first; CLAUDE.md PLAY block only references them at summary depth.

### 7.4 Per-commit DoD checklist

- [ ] CLAUDE.md PLAY block reflects post-Phase-7.5 state (XGID next-active or design-phase pending, as appropriate).
- [ ] CLAUDE.md header `Last updated` bumped with one-paragraph narrative of Phase 7.5 closure.
- [ ] ROADMAP.md Past + Present sections updated.
- [ ] Design doc §15 row journal-number placeholder updated.
- [ ] JOURNAL.md entry written covering all five commits, with actual test-count quotes from each commit's `cargo test --workspace` output.
- [ ] All headers verified to have two trailing spaces on every `>` line.
- [ ] No code changes — confirm with `git diff --stat` showing only `.md` files.

---

## 8. Cross-references

- `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` — Design task file (COMPLETED v1.0 after Commit 1 of this runbook). Authoritative on the four locks.
- `docs/xgen_federation_propagation_design.md` — Canonical design doc (ACTIVE v1.0). Gains §6.4.1 in Commit 1 of this runbook.
- `tasks/FEDERATION_PROPAGATION_PHASE_9.md` — Phase 9 implementation task file (ACTIVE v1.0). Paused at Commit 3 boundary; resumes after Phase 7.5 closes AND XGID work closes.
- `docs/ROADMAP.md` — Canonical project navigation. XGID sits between Phase 7.5 closure and Phase 9 Commit 3 resume.
- `xgen-node/src/tests/heldpending_identity_integration.rs` — Phase 6 template for NodeRuntime-level HeldPending integration tests.
- `xgen-node/src/tests/federation_relationship_integration.rs` — Phase 7 template for federation-relationship integration tests.
- D-065 (honest behaviour over polite behaviour) — informs Lock P7.5-B held-not-bypassed posture.
- D-069 (Joe-locked design phase + canonical-document rule) — discipline that produced the design task file.
- D-071 (subsystem audits precede dependent milestones) — extends to "design gaps surface during dependent work and close before the dependent work proceeds."

---

## 9. Definition of Done

Phase 7.5 implementation is complete when:

- [ ] Commit 1 (doc-pass) shipped: §6.4.1 in canonical design doc, §15 row added, design task file flipped COMPLETED.
- [ ] Commit 2 (F-3 + F-4 step 1 skip + SpaceLocalMetadata) shipped: unit tests passing, headers bumped.
- [ ] Commit 3 (HeldPending third trigger + config + error code + observability) shipped: unit tests passing, headers bumped.
- [ ] Commit 4 (integration tests) shipped: all six scenarios (A through F) passing.
- [ ] Commit 5 (milestone close) shipped: CLAUDE.md + ROADMAP.md + design doc §15 journal-number all updated; JOURNAL.md entry written.
- [ ] All four `[JOE-LOCK: locked 2026-05-19]` decisions from design phase implemented as specified.
- [ ] Test count incremented by sum of N + M + K (new unit + integration tests across Commits 2, 3, 4). Actual count quoted in JOURNAL.md per CLAUDE.md Rule 5.
- [ ] No carry-overs to a follow-on milestone unless explicitly flagged in JOURNAL.md and added to a tracked PENDING list (no silent deferrals).

After Phase 7.5 implementation closes: **do NOT proceed directly to Phase 9 Commit 3.** Per ROADMAP.md, XGID concept work is Near future first-in-queue and sequences between Phase 7.5 closure and Phase 9 resume. XGID is a separate milestone with its own design + implementation cadence.

---

*End of document. Phase 7.5 implementation is Clair's pickup. Design authority remains with `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` §5–§8.*  
