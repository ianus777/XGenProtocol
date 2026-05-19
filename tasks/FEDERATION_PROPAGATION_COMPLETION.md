# Federation Event Propagation — Implementation Runbook

> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-19 (§3.6.1 "Phase 6 implementation locks" added — Lock A (A2: per-PendingBuffer secondary index with cross-Space fan-out), Lock B (B1: struct variant for HeldPending with optional missing_identity), Lock C (C2: counter in xgen-node_state.json), Lock D (D1+D3: TimedOut extension + new error code 4004 identity_record_timeout with predecessor-code-wins sub-rule for both-missing case). §3.5.1 Phase 5 implementation locks shipped earlier 2026-05-19; §3.4.1 Phase 4 implementation locks shipped earlier same day; §3.3.1 Phase 3 implementation locks shipped earlier same day; §3.3 Joe-locked to Option 3 wire shape; §9 / §10 walked from flagged to shipped reflecting D-070 and D-071 promotions on 2026-05-18.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Overview

This runbook implements the Federation Event Propagation completion milestone. It is the implementation-phase task file that follows the milestone's Joe-locked design phase. The design is locked; this runbook is how the locked design gets shipped.

**The canonical design document is the source of truth.** Read `docs/xgen_federation_propagation_design.md` (Status: ACTIVE, v1.0) before starting any phase of this runbook. This runbook does not re-specify the design; it sequences the implementation work, identifies the call sites, and locks the ordering of phases. When this runbook and the design doc differ, the design doc wins and this runbook gets corrected.

**Why this milestone exists.** The Propagation Reliability Audit (J-081, `docs/xgen_propagation_reliability.md`) found that Node-to-Node federation event propagation does not exist as a production mechanism. Three independent traces converged: zero production callers of `run_initiating`, no Node-to-Node pull mechanism, and stress-test "Federation Completeness" measuring only local-clients delivery. The federation surface today is a one-time history dump on peer-initiated handshake, then the connection closes. This milestone closes that gap.

**Two pieces of work, one milestone.** Federation event push (the new mechanism) and `process_inbound` validation pipeline unification (the precondition) belong together. The audit's §3 finding established that pushing federation events through a `process_inbound` with three asymmetric validation paths would land a vulnerability — peers could push membership and state events that bypass signature verification. F-4 closes the asymmetry; F-1 lands the push channel. **Phase 2 (validation unification) MUST land before Phase 4 (federation push).** This runbook makes that ordering hard.

**Coordination point with M6 (new) Phase 2.** The M6 design doc (`docs/xgen_node_admin_ops_design.md`) Joe-locked at audit close that `TransportMessage::Error` gains an envelope-level `event_id: Option<String>` field and the five event-rejection sites in `process_inbound` populate it. This work is M6 Phase 2's responsibility, but the rejection paths are touched by this milestone's F-4 work. The two milestones coordinate at the rejection-signal interface: F-4 ensures the rejection paths exist consistently across all event families; M6 Phase 2 wires them to the `Error` variant with envelope `event_id`. Sequencing is documented in §3 below.

**No code changes in this milestone's design phase.** The design phase (Passes 1–3) shipped 0 code changes. This runbook is where code changes start. Test count baseline at runbook handoff is 468.

---

## 2. Test environment and current state

### 2.1 State at runbook handoff (2026-05-18)

- **Workspace test count:** 468 tests passing. Established at Propagation Reliability Audit close (J-081).
- **Last shipped milestone:** M5 `ops::*` refactor (J-078, 435 tests at that point; grew to 463 in J-079 CLI Audit; to 468 in J-080 carry-over pass; held at 468 through M6 Phase 0 and the Audit).
- **Active design phases closed:** M6 (new) Phase 0 (canonical doc `docs/xgen_node_admin_ops_design.md`, Status ACTIVE), Federation Event Propagation Phase 0 (canonical doc `docs/xgen_federation_propagation_design.md`, Status ACTIVE, v1.0).
- **Blocking dependency on M6 (new):** None. M6 (new) is blocked behind this milestone, not the other way around.
- **Coordination dependency on M6 (new) Phase 2:** The envelope `event_id` on `TransportMessage::Error`. See §3 sequencing.

### 2.2 Test count growth expectations

Phase-by-phase expected count growth is sketched per phase in §3 below. The runbook does not pre-lock exact numbers; what matters is that:

- Every phase ships with `cargo test` passing.
- Every phase's commit lists the actual test count from real output (per CLAUDE.md Rule 5).
- A phase that drops the test count is a regression that must be explained before commit.

Cumulative growth across all nine phases is expected to be in the range of +30 to +60 tests (new unit tests for the validation core, federation push tests, integration tests for the push path, sync_complete migration tests, pagination tests). The exact number is determined by Clair's implementation; this is a rough sizing, not a target.

### 2.3 Build environment

- Workspace: `E:\Projects\XGenProtocol` (Cargo workspace).
- Target dir: `C:/cargo-targets/XGenProtocol` (set via `CARGO_TARGET_DIR` to avoid Google Drive file locking).
- Build script: `build.sh` (copies binaries to `bin/` in project folder).
- Tauri compiled into both binaries per D-062.

No build-environment changes are expected during this milestone.

---

## 3. Phase plan

Nine phases. Each is independently committable. The ordering between phases is significant; the ordering within a phase is Clair's call.

**Hard precondition: Phase 2 MUST land before Phase 4.** Federation push without validation asymmetry closure is the audit-identified vulnerability vector. The runbook makes this ordering hard.

**Soft preconditions:**
- Phase 1 (sync_complete + pagination) lands first because Phases 4 and 6 depend on it.
- Phase 6 (HeldPending generalisation for unknown signer) needs the unified validation core from Phase 2.
- Phase 7 (F-3 federation-relationship check) needs Phase 2's dispatcher structure.
- Phase 9 (integration tests) needs all preceding phases to be in place.

### 3.1 Phase 1 — `sync_complete` wire shape + pagination (F-6 + F-7)

**Scope.** Implement `TransportMessage::SyncComplete` and the pagination fields on `TransportMessage::SyncRequest`. Migrate all four production `SyncRequest` callers from the 500ms quiet-time heuristic to the explicit signal with pagination loops.

**Design reference.** `docs/xgen_federation_propagation_design.md` §9 (F-6), §10 (F-7).

**Call sites (audit §4.5).** All four:
- `xgen-client/src/batch.rs:83`
- `xgen-client/src/ai_service.rs:224`
- `xgen-client-lib/src/ops.rs:721`
- `xgen-client-lib/src/ops.rs:939`

**Node-side emission point.** End of `collect_sync_history` delivery, around `xgen-node/src/app.rs:613-619`. After the last event of a batch is sent, emit `SyncComplete { since, new_tip, continue_from }`.

**Cross-Space behaviour decision.** F-6 design doc §9.7 flagged this as Clair's latitude: emit one `SyncComplete` per Space (with that Space's tip), or one `SyncComplete` per batch (with a `space_id → tip` map). Pick one, document the choice in the commit message and in a code comment at the emission site, ensure the four call sites all handle the chosen shape consistently. Per-Space is the more natural pagination granularity; whole-batch is fewer messages. Trade-off is Clair's call.

**Config fields.** Add to both `xgen-node_config.toml` and `xgen-client_config.toml`:
- `[sync].completion_timeout_seconds` — default 5 (F-6b)
- `[sync].batch_size` — default 1000 (F-7a)

Both fields are optional with defaults. Both should be wired through the `xgen-common::precedence::resolve_setting<T>` helper per D-068 (CLI flag > env > config > default).

**Definition of Done — Phase 1.**

- [ ] `TransportMessage::SyncComplete` variant added to `xgen-common::wire::types`, with serde derives that handle `continue_from: Option<String>` as omittable for backward-compat.
- [ ] `TransportMessage::SyncRequest::limit: Option<u32>` field added.
- [ ] `collect_sync_history` honours `limit` and emits `continue_from` correctly.
- [ ] All four production callers migrated from quiet-time to explicit-signal with pagination loops.
- [ ] Cross-Space behaviour choice (per-Space or per-batch SyncComplete) made, documented at emission site and in commit message.
- [ ] Config fields surfaced in both Node and Client configs with documented defaults.
- [ ] Unit tests for the wire-shape additions (serialise/deserialise round-trip with and without `continue_from`).
- [ ] Integration tests for the pagination loop (small page size forces pagination, requester collects all events, terminates on null `continue_from`).
- [ ] Integration test for the safety-net timeout (peer never sends `SyncComplete` → timeout fires, requester logs "peer never said done" and surfaces error; does NOT silently proceed).
- [ ] `cargo test` passes with actual test count quoted in commit message.
- [ ] JOURNAL entry written after the work is verified, quoting actual command output per CLAUDE.md Rule 2.
- [ ] Phase-1 commit pushed by Joe (Clair never pushes directly).

**Why this lands first.** Phase 4 (federation push) issues sync_requests via the pull-on-gap recovery code path. That code path needs the reliable completion signal from day one. Migrating the existing four callers in a separate commit ahead of federation push keeps the wire-shape change reviewable in isolation.

---

### 3.2 Phase 2 — `process_inbound` validation pipeline unification (F-4)

**Scope.** Refactor `process_inbound` into the dispatcher shape locked in F-4 §7.7: structural pre-checks → federation-relationship check → unified validation core → semantic pre-checks → event-type-specific post-validation handlers → fan-out. All three event-type families (Path A messages, Path B membership.join, Path C other state events) flow through one validation core. HeldPending becomes a property of the validation core, not of `accept_message` specifically.

**Design reference.** `docs/xgen_federation_propagation_design.md` §7 (F-4), §7.5 (F-4a 30s uniform timeout), §7.6 (F-4b pre-check placement), §7.7 (pipeline shape).

**Files touched.** `xgen-node/src/app.rs` (`process_inbound`), `xgen-node-lib/src/runtime` (validation core; existing `accept_event` and `accept_message`), HeldPending buffer module.

**The validation core's contract.**

```rust
enum ValidationOutcome {
    Validated(Event),
    HeldPending,
    Rejected { reason: RejectionReason, stage: ValidationStage },
}

fn validate_event(event: &Event, ctx: &mut ValidationContext) -> ValidationOutcome;
```

The exact type names and signature are Clair's latitude — what matters is one function reachable from every code path, returning one of three outcomes. The post-validation handlers branch on event type only after `Validated` returns.

**Refactor latitude.** F-4 §7.4 reasoning recorded that this is the M5 / D-067 precedent for validation. The `accept_message` boundary may evaporate as a separate function (becoming just the message-handler arm of the unified dispatcher) OR may remain as a thin wrapper for backward-compat with existing callers. Clair's call. Criterion is *cleaner is better*; a refactor that produces two ways to validate an event is wrong regardless of how it got there.

**HeldPending move.** Today HeldPending lives inside `accept_message`. After Phase 2 it must be reachable from all three event families' code paths. The buffer's identity (one per Node, one per Space, or one per (Space, EventFamily)) is a runbook detail; the design only requires that buffer behaviour applies uniformly. **Phase 6 generalises the trigger condition; Phase 2 moves the buffer.** Keep them as separate commits so each is reviewable in isolation.

**Pre-check placement (F-4b).**

| Check | Placement | Why |
|---|---|---|
| Space exists locally | Before validation | Cheap HashMap lookup; fail-fast avoids wasting Ed25519 crypto |
| Federation-relationship lookup (only for federation-channel events) | Before validation | Cheap registry lookup; fail-fast same reason |
| Signature verification | Inside validation core | Crypto work; uniform across families |
| Timestamp check | Inside validation core | Uniform across families |
| Predecessor presence (HeldPending decision) | Inside validation core | Uniform across families |
| AI role violation | After validation | Semantic check on validated event |
| AI operator target/permission | After validation | Semantic check on validated event |

**Coordination with M6 (new) Phase 2.** Both this phase and M6 (new) Phase 2 touch the rejection sites in `process_inbound`. F-4's contribution is ensuring those rejection paths exist consistently across all three event families (today Paths B and C reject inline via `tracing::error!` + `trace_local(RejectEvent)`; after F-4 they reject through the dispatcher's `Rejected` return). M6 (new) Phase 2's contribution is wiring those rejections to emit `TransportMessage::Error` with envelope `event_id: Some(...)`. **Coordination point:** F-4 produces the rejection sites; M6 Phase 2 wires them to the wire-layer signal. Sequencing: Phase 2 of this milestone lands the rejection-site consistency; M6 Phase 2 then wires the wire-layer signal in its own milestone. If M6 Phase 2 lands first by accident (it shouldn't, since M6 is blocked behind this milestone), the wiring works against the pre-refactor rejection sites in B and C and would need adjustment.

**Definition of Done — Phase 2.**

- [ ] `process_inbound` refactored to the dispatcher shape from F-4 §7.7.
- [ ] Validation core function exists; every event family reaches event-handling code only via the core.
- [ ] HeldPending buffer moved out of `accept_message` to a shared module reachable from all paths.
- [ ] HeldPending 30-second timeout uniform across all event families (F-4a).
- [ ] Pre-check placement matches F-4b (structural before, semantic after).
- [ ] Existing HeldPending tests for messages still pass.
- [ ] New tests for the three Scenario-A cases the audit identified: Path B unknown predecessor → HeldPending; Path C unknown predecessor → HeldPending; Path A regression.
- [ ] `cargo test` passes with actual test count quoted in commit message.
- [ ] JOURNAL entry written after verification.
- [ ] Phase-2 commit pushed by Joe.

**Why this lands second.** Phase 4 (federation push) is the audit's exact vulnerability vector. Pushing federation events into a `process_inbound` with three asymmetric validation paths means peers can push membership and state events that bypass signature verification. F-4 closes that asymmetry. Without Phase 2, Phase 4 lands a known vulnerability. **The runbook makes this ordering hard: Phase 4 MUST NOT start before Phase 2 ships and verifies.**

---

### 3.3 Phase 3 — Federation handshake reshape to tip exchange (F-1a)

**Scope.** Replace today's `handle_federation_incoming` history-dump logic with the F-1a tip exchange: peer sends current tip per shared Space, home Node responds with delta from peer's tip up to home's current tip. After delta delivery, the session **stays open** as the persistent push channel — no connection close.

**Design reference.** `docs/xgen_federation_propagation_design.md` §4.4 (F-1a tip exchange).

**Files touched.** `xgen-node/src/app.rs` (`handle_federation_incoming` and surrounding handshake logic), federation message types in `xgen-common::wire`.

**What changes on the wire.** The federation handshake message family gains tip-exchange semantics. Wire shape Joe-locked 2026-05-19 to Option 3 (bilateral, fold into existing handshake messages) after Clair surfaced the three sub-options for Joe-lock per D-069's "would a future contributor ask why is this what it is" threshold.

**Locked wire shape.** Both `federation.hello` and `federation.capabilities` gain a `tips` field:

```rust
// xgen-core/src/wire/types.rs (extensions)

#[serde(rename = "federation.hello")]
Hello {
    protocol_version: String,
    node_id: String,
    capabilities: ...,
    shared_spaces: Vec<String>,
    #[serde(default)]
    tips: BTreeMap<String, String>,    // NEW: space_id → tip event_id
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
}

#[serde(rename = "federation.capabilities")]
Capabilities {
    protocol_version: String,
    node_id: String,
    capabilities: ...,
    negotiated: ...,
    #[serde(default)]
    tips: BTreeMap<String, String>,    // NEW: space_id → tip event_id
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
}
```

`federation.accept`, `federation.reject`, `federation.goodbye` unchanged.

**Locked semantics.**

- **Empty `tips` map** = "I participate in zero shared Spaces" — distinct from "I have shared Spaces but no events yet."
- **Absent entry for a `space_id` that appears in `shared_spaces`** = "I have this Space but no events yet — send full history."
- **Present entry** = "I have events up through this `event_id` — send delta from here."
- **`#[serde(default)]` on `tips`** — back-compat with pre-F-1a peers. Their absent field deserialises as empty map. Behaviour against a pre-F-1a peer that omits `tips` entirely is "send full history for every shared Space," which matches the pre-F-1a dump-then-close behaviour at the Space-list level. Clean semantic degradation.
- **`BTreeMap` not `HashMap`** — deterministic JSON serialisation. The `signature: Option<String>` field already exists on both Hello and Capabilities; if F-3 ever extends to signing handshake messages, deterministic map ordering is the precondition. The performance argument doesn't apply on the once-per-handshake path.

**Sequence with the locked shape.**

```
B → A: Hello { ..., tips_B }
A → B: Capabilities { ..., tips_A }
B → A: Accept
  (both sides now know each other's tips)
A → B: delta events for each Space (events after tips_B per Space) via collect_sync_history + SyncComplete from Phase 1
B → A: delta events for each Space (events after tips_A per Space) via the same mechanism
  (session stays open as the persistent F-2 push channel — no goodbye)
```

The symmetric exchange means a single handshake fully reconciles both directions. F-1c's reconnect scheduling does not depend on bidirectional ordering for correctness — whichever side comes back first and initiates achieves full reconciliation.

**Forward-looking note.** Capability-gated tip behaviour (e.g. a future capability "peer without X can't request deltas larger than N events") would tie tip data to a pre-capability-negotiation moment under Option 3. No such capability is specced or planned today. If one ever surfaces, that constraint could retroactively push toward Option 2 (a dedicated `federation.tip_exchange` message that runs after capability negotiation). Not a blocker for Option 3 today; recorded here so the rationale is visible if the question ever comes up.

**[JOE-LOCK: locked 2026-05-19 (Phase 3 pre-implementation Joe-lock session)]**

This lock supersedes the Clair-latitude framing the runbook originally carried. The shape, semantics, and `#[serde(default)]` back-compat behaviour are all canonical. Phase 3 implementation references this section as the authoritative wire shape; any deviation requires a fresh Joe-lock conversation, not engineering judgement.

---

### 3.3.1 Phase 3 implementation locks

This subsection captures Joe-locks made during the Phase 3 pre-implementation conversation that surfaced *after* the wire-shape lock in §3.3. They are implementation-strategy decisions, not wire-format decisions — the runbook keeps them separate from §3.3 to preserve concern separation. All of these decisions are canonical for Phase 3 implementation; deviation requires fresh Joe-lock.

**Lock 1 — Migration scope: Option A (full migration).** Today's federation join model wraps the handshake with `space.join_request` and finishes with dump-then-close. Option A migrates all seven `xgen-client/src/app.rs` call sites and the smoke.rs caller to the new shape (populate `tips: BTreeMap<String, String>`, drop `JoinRequest` send, consume delta via `SyncComplete` pattern). The alternative (Option B — coexistence of legacy `JoinRequest`/dump-then-close with new tip-exchange) was rejected because it re-introduces the kind of asymmetry that J-081 §3 finding and Phase 2's F-4 just closed. Two federation models in production = drift surface = exactly what D-067 (single source of truth) was named to prevent. The seven-call-site migration cost is bounded; the Phase 1 four-call-site migration is the precedent that landed cleanly.

**Lock 2 — `state.federation_add` trigger: a-i with symmetry rule.** Under Option 3's bilateral exchange, both sides see each other's `tips` maps. The trigger rule for who builds `state.federation_add`:

> The side that **has events** for a Space builds `state.federation_add` when the **other side's `tips` map shows that Space absent** (i.e., the other side is brand-new for that Space).

Three cases:

1. **A has Space S with events, B's `tips[S]` is absent** → A builds `state.federation_add` for Space S. (B is joining S from A's perspective. Matches today's receiver-builds behaviour.)
2. **B has Space S with events, A's `tips[S]` is absent** → B builds `state.federation_add` for Space S. (Symmetric to case 1.)
3. **Both sides' `tips[S]` are absent for some Space S in shared `shared_spaces`** → no one builds. The Space is genuinely empty for both sides; relationship-establishment fires naturally when the first event arrives in a future handshake.

Case 3 is theoretically reachable but practically impossible under current Space semantics (a Space exists because some Node created it, producing at least `state.space_create` as the root event), so the side that created the Space always has at least one event. The rule needs to be correct for the impossible case in case Space semantics ever change.

This makes the "who builds it" decision deterministic from the wire-visible `tips` maps. No race conditions, no duplicate DAG events. Both sides compute the same answer from the same data.

The rejected alternatives were a-ii (build `state.federation_add` always on handshake completion, idempotent if already federated — produces redundant DAG events on every reconnect; "idempotent" check would be semantic rather than structural, fragile) and a-iii (drop `state.federation_add` from this milestone, move federation relationship state to F-1c per-peer record — loses the DAG audit trail of relationship establishment without giving the data anywhere else to live; F-1c is operational state, not protocol-visible relationship history).

**Lock 3 — R1: Federation session-stays-open loop shape.** After delta delivery, the federation connection task enters a steady-state loop that drains inbound `recv()` and routes `Inbound::Event` through `process_inbound` (Phase 2's unified dispatcher). Handles `Ping`/`Pong` keepalive and `Goodbye`/`Closed` exit. The loop wires an outbound `tokio::sync::mpsc::channel(1024)` as Phase-4 prep even though Phase 3 doesn't use the outbound arm — this avoids restructuring the loop twice in adjacent commits. The channel size 1024 matches the client-connection precedent at `xgen-node/src/app.rs:622-734`.

Code-comment requirement at the channel wire site (verbatim per Clair's lock acknowledgement):

```rust
// Phase 4 plugs in the federation-push sender here; intentionally unused in
// Phase 3 — channel exists to avoid restructuring the loop at Phase 4 ship.
```

**Lock 4 — R2: New sibling helper for federation delta, not generalise `collect_sync_history`.** Phase 3 introduces `compute_federation_delta_for_space(runtime, space_id, peer_tip_opt: Option<&str>) -> Vec<Event>` (likely in `xgen-node/src/fanout.rs`) as a sibling of the existing `collect_sync_history`. The two helpers serve genuinely different callers: `collect_sync_history` is Identity-membership-shaped (requester is a Client tied to an Identity, scope derives from Identity membership across all Spaces); the new helper is per-peer-per-Space-tip-shaped (requester is a peer Node, scope is per-Space-cursor). Forcing both shapes through one function with optional parameters that mean different things in different contexts is exactly the asymmetry pattern D-067 was named to prevent. Two helpers, two callers, two responsibilities — not a drift surface because they serve genuinely different callers (unlike M5's case of two implementations of the same verb).

**Lock 5 — R3: `SyncComplete.new_tip` semantic for federation = best-effort `last_event_id_sent` across all Spaces, empty if delta was empty.** `new_tip` is informational, not load-bearing — receivers track per-Space tips through event ingestion (same as Phase 1 clients). Don't change the `SyncComplete` wire shape. Stays compatible with Phase 1's cross-Space whole-batch lock.

Code-comment requirement near the federation `SyncComplete` consumption site (per Clair's lock acknowledgement, exact wording at her discretion but conveying):

```rust
// new_tip is informational for federation deltas. Receivers MUST NOT compare
// it to a single-Space tip — under cross-Space whole-batch delivery (Phase 1
// lock) it carries last_event_id_sent across all Spaces. Trust the
// SyncComplete frame as the done signal; read per-Space tips from ingested
// events post-stream.
```

**Lock 6 — R4: Cross-Space ordering in delta delivery = sorted by `space_id`.** Per-Space topological order is mandatory (events within a Space must arrive in DAG-valid order — predecessors before successors). Cross-Space order is free at the protocol level but locked to sorted-by-`space_id` for two consequences worth naming: (1) test determinism — both sides under bilateral exchange produce identical event-stream orderings for a given (history, tips) pair, so integration tests can assert exact event sequences rather than set-membership; (2) future audit-log correlation — deterministic ordering means two replays from the same starting state produce the same log entries.

The `BTreeMap` iteration of `tips` gives sorted order for free for Spaces present in the peer's `tips` map; iterate the full `shared_spaces` list in sorted order for the absent-tip cases.

The rationale is recorded in the JOURNAL entry, not in source comments — per CLAUDE.md's "default to no comments" policy, why-we-chose-sorted belongs in the JOURNAL (rationale-as-record), not in source (which says what the code does, not why).

**Lock 7 — R5: Bilateral delta initiator-side usage in Phase 3.** Under Option 3, both sides have each other's tips post-handshake and both must stream delta in their direction. The seven `xgen-client/src/app.rs` call sites are client-driven establishments — those clients don't have a local `NodeRuntime` with a DAG to stream FROM. Initiator-side delta delivery is skipped in those flows; the receiver streams its delta, the initiator consumes it.

The new integration tests (brand-new, reconnect-with-existing-tip) run two in-process `NodeRuntime` instances bilaterally — both `stream_federation_delta` calls fire, one per direction. These integration tests are the *only* Phase-3 callers of the initiator-side path. They double as the regression-locking surface for Phase 5's reconnect scheduler, which is the first production caller of the initiator-side path. Phase 3 ships the mechanism; Phase 5 wires the production caller.

Not a drift surface because Phase 3 + Phase 5 sequence is locked in the runbook §3.10 phase ordering. The integration tests prove the mechanism works bilaterally; Phase 5 plugs in the deployment-level caller.

---

**Step 6 of Clair's implementation sequence** (refactor `handle_federation_incoming`) requires three code comments per the locks above, all pointing at this section of the runbook:

```rust
// §3.3 Locked wire shape (Option 3 bilateral tips)
```

near the tip-exchange parsing,

```rust
// §3.3 a-i symmetry rule
```

at the `state.federation_add` build call inside `stream_federation_delta`, and the R1 channel-wire-site comment quoted in Lock 3 above.

**[JOE-LOCK: locked 2026-05-19 (Phase 3 pre-implementation Joe-lock session, second pass)]**

All seven locks above (Lock 1 through Lock 7) are canonical for Phase 3 implementation. Deviation from any of them requires a fresh Joe-lock conversation, not engineering judgement.

**Delta delivery.** Uses the `collect_sync_history` mechanism from Phase 1 (with pagination and explicit `sync_complete`). For a brand-new relationship the delta is the full history; for recovery after downtime the delta is small. The tip-exchange model is symmetric for both cases.

**Session stays open.** Today's handshake closes the connection after the dump. F-1a keeps it open. This means the federation connection task transitions from "handshake → dump → close" to "handshake → tip exchange → persistent push/pull session." The connection lifecycle now matches the F-2 long-lived continuous session model (see Phase 4 for the push side).

**Definition of Done — Phase 3.**

- [ ] `handle_federation_incoming` refactored from history-dump to tip-exchange.
- [ ] Tip-exchange wire shape per locked Option 3 (see §3.3 "Locked wire shape" above): bilateral `tips: BTreeMap<String, String>` on `Hello` + `Capabilities`, `#[serde(default)]` for back-compat. Code comment at the handshake site cites "§3.3 Locked wire shape" of this runbook for the rationale.
- [ ] Delta delivery uses Phase 1's `collect_sync_history` + `SyncComplete` + pagination.
- [ ] After delta delivery, the federation session stays open (no close).
- [ ] Existing federation tests updated to reflect the new handshake shape (some will need rewrites; the dump-then-close shape no longer holds).
- [ ] Integration test for "brand-new relationship → full history delivery (both sides empty `tips` map for the shared Space) → session stays open" passes.
- [ ] Integration test for "reconnect with existing tip → small delta delivery (both sides populated `tips` map) → session stays open" passes.
- [ ] Integration test for "pre-F-1a peer compatibility" passes (peer omitting the `tips` field deserialises as empty map; home Node sends full history per Space).
- [ ] `cargo test` passes with actual test count quoted in commit message.
- [ ] JOURNAL entry written.
- [ ] Phase-3 commit pushed by Joe.

---

### 3.4 Phase 4 — Federation event push (F-1 + F-1b + F-5)

**Scope.** Implement the push mechanism. On Stage-4 accept of an event in `process_inbound`'s post-validation handler (Phase 2's dispatcher step 5), the post-fan-out step pushes the event over the persistent F-2 session to each federated peer that participates in the relevant Space. Implements F-1 main (the push), F-1b (drop on peer-down — no outbound queue), and F-5 (origin gating — only locally-submitted events get pushed).

**Design reference.** `docs/xgen_federation_propagation_design.md` §4 (F-1), §4.5 (F-1b), §8 (F-5 transitive locked-out), §8.5 (origin gating).

**Files touched.** `xgen-node/src/app.rs` (the `apply_fanout` call site at approximately `app.rs:637` — federation push lands as a sibling, not a wrapper); the federation peer session management module.

**The push site.** F-1 §4.7 noted: "The push-on-Stage-4 hook integrates at or near `app.rs:637` (the existing `apply_fanout` call site). Federation push is a sibling of local fan-out, not a wrapper around it." The dispatcher pipeline from Phase 2 §7.7 step 6 already shows the shape:

```
apply_fanout(...)        # Stage 5 local fan-out (unchanged)
apply_federation_push(...)  # Stage 6 federation push (NEW in Phase 4)
```

The federation-push function takes the validated event and pushes it over the active session to each federated peer. Drop on peer-down: if there's no active session to a peer (the F-1c lost-connection flag is set or the WS write fails), the push is dropped and recovery falls to the peer's tip-exchange on next handshake. **No outbound queue.** This is F-1b.

**Origin gating — load-bearing.** F-5 §8.5 is explicit:

> The implementation MUST explicitly check, before calling `apply_federation_push`, that the event being pushed was **locally submitted** to this Node. Events that arrived via federation (over a peer session) MUST NOT enter the federation-push code path.

Implementation marker: events carry an in-memory `EventOrigin` enum (`LocallySubmitted` or `ReceivedViaFederation`) that the federation-push function inspects before forwarding. Wire-invisible. The runbook's regression test specifically asserts that a federation-received event does NOT trigger another federation push.

**Definition of Done — Phase 4.**

- [ ] `apply_federation_push` function exists at sibling position to `apply_fanout`.
- [ ] Push site uses the persistent F-2 session established by Phase 3's handshake reshape.
- [ ] Drop-on-peer-down semantics: peer unreachable → push dropped → no outbound queue → log line emitted for observability.
- [ ] Origin gating: `EventOrigin::ReceivedViaFederation` events are NOT pushed (hard guard at the top of `apply_federation_push` with comment citing F-5 §8.5).
- [ ] Integration test for "Alice on Node A posts E → Node B receives E via federation push → Node B's fan-out delivers to Bob on B" passes.
- [ ] Integration test for "Bob on Node B receives E via federation push → Node B does NOT push E to any other peer" passes (the F-5 anti-transitivity regression test).
- [ ] Integration test for "Node A pushes E while Node B is down → E is dropped → Node B comes back → tip-exchange delivers E" passes.
- [ ] `cargo test` passes with actual test count quoted in commit message.
- [ ] JOURNAL entry written.
- [ ] Phase-4 commit pushed by Joe.

**Why Phase 4 is the milestone's load-bearing phase.** This is the work the audit said was architecturally absent. After Phase 4 ships, Stage 6 of the propagation lifecycle exists as a production mechanism. The "missing mechanism" verdict from J-081 §2 is closed.

---

### 3.4.1 Phase 4 implementation locks

This subsection captures Joe-locks made during the Phase 4 pre-implementation conversation that surfaced after the §3.4 design lock. They are implementation-strategy decisions, not design decisions — the runbook keeps them separate from §3.4 to preserve concern separation. All of these decisions are canonical for Phase 4 implementation; deviation requires fresh Joe-lock.

**Lock 1 — Q1: `EventOrigin` as a runtime parameter through `dispatch_event` / `process_inbound` / `apply_federation_push`.** The `EventOrigin` enum lives in `xgen-core::node::runtime` (with `DispatchOutcome`), not in `xgen-common::wire`. Wire data and runtime observation are separate concerns: the `Event` struct in `xgen-common::wire` is the wire form; origin is a runtime observation about the event's path through this Node and properly belongs as a parameter to the in-process dispatcher.

```rust
// xgen-core::node::runtime
pub enum EventOrigin {
    LocallySubmitted,        // arrived via a client connection
    ReceivedViaFederation,   // arrived via a federation peer session
}
```

**Forward-compatibility note.** Today only two variants are needed. The enum shape leaves room for future origin types (`ReceivedViaAdminInjection` for M6 admin write-path work, `ReceivedViaBackfill` for hypothetical future replay tooling, etc.). Doc comment on the enum should note this.

The rejected alternatives were Option B (`EventWithOrigin` wrapper struct flowing through `dispatch_event` — same end state with more API churn at `DispatchOutcome`, every test that constructs an event for dispatch, etc.; encapsulation gain is mild because callers construct the pair atomically at the two entry points anyway) and Option C (`#[serde(skip)]` field on `Event` itself — puts an in-memory-only field on the wire-shape struct in `xgen-common::wire`, hides the data-flow seam from a future contributor reading the wire definition; `#[serde(skip)]` is silent in review). Option C is exactly the D-069 "would a future contributor ask why is this here" failure mode applied to a wire-shape struct.

**Lock 2 — Q2: `FederationPeerSenders` mirroring `ClientSenders`.** `Arc<Mutex<HashMap<peer_node_id, mpsc::Sender<OutboundMsg>>>>`. Single source of truth for federation membership stays in `SpaceState.federation_nodes` (already authoritative). Registry is purely "active session presence." Two responsibilities, two structures, no drift surface.

```rust
pub type FederationPeerSenders = Arc<Mutex<HashMap<String, mpsc::Sender<OutboundMsg>>>>;
```

The lookup pattern: `apply_federation_push` reads `SpaceState.federation_nodes` for the event's Space; for each `peer_id`, look up sender in registry; if present `try_send`; if absent or `try_send` fails, log F-1b drop. F-2a (one WS per pair bidirectional) justifies single-sender-per-peer at the registry layer.

The rejected alternative was Option B (`Arc<Mutex<HashMap<peer_id, FederationPeerInfo>>>` co-locating sender and `shared_spaces` set in the registry) — duplicates `SpaceState.federation_nodes` data, produces two sources of truth that go silently out of sync. The failure mode is silent message loss (stale registry doesn't push to peers that should receive), not loud error. Performance argument for B is small at Phase 1/2 scale; F-3 §6.8 already established the rule that lookups optimise only if profiling shows they're expensive.

**Lock 3 — Q3: Inbound federation events reuse `process_inbound` with two-comment overload documentation.** F-4's whole point was to unify dispatch paths so validation can't be bypassed on any code path. Adding a parallel `process_federation_inbound` (Option C) would re-introduce the asymmetry pattern J-081 §3 found and Phase 2 closed. The federation session task calls `process_inbound(..., msg, &peer_node_id, ..., EventOrigin::ReceivedViaFederation, ...)`. The `identity_id` parameter accepts any pubkey URI — Identity URIs (client connections) and Node URIs (federation sessions) share wire shape.

The latent semantic overload of `identity_id` is documented at two sites, not one:

**At the federation session's `process_inbound` call site (the new caller):**

```rust
// peer_node_id is passed as the wire-authenticated sender; process_inbound's
// identity_id parameter accepts any pubkey URI — see runbook §3.4 Q3 lock.
```

**At the `process_inbound` function definition itself (one line above the function):**

```rust
// identity_id: the wire-authenticated sender — Identity URI (client
// connection) or Node URI (federation session). Used for trace context;
// downstream validation does not depend on which kind of principal this is.
```

The function-definition comment puts the semantic at the dispatcher where future contributors will read it. The call-site comment puts the rationale at the federation call site where the dual usage is visible. Two comments, no signature change, no rename — resolves the latent overload through documentation rather than M5-shape churn.

The rejected alternative was Option B (rename `identity_id` to `wire_sender_id` across the codebase) — appealing in isolation but touches xgen-client, xgen-node, and a bunch of tests. The rename was judged M5-shape scope creep for Phase 4. The latent overload has been latent for months without producing bugs because the dispatcher logic that uses `identity_id` doesn't actually care which kind of principal it is — it's purely for trace context. Two comments close the documentation gap without forcing the rename.

**Clair-latitude items (R12–R15).** These follow established patterns and lock via code-comment runbook citations, not Joe-lock. Surfaced here so they aren't late discoveries:

- **R12 — Registry lifecycle.** Register `out_tx.clone()` into `FederationPeerSenders` after handshake reaches ACTIVE and before entering the steady-state loop. Deregister on loop exit. Exact mirror of `xgen-node/src/app.rs:751` for `client_senders`. Code comment cites runbook §3.4.
- **R13 — Push delivery semantics.** `try_send` not `send` — non-blocking, drop on channel-full per F-1b "no outbound queue." Exact mirror of `apply_fanout`'s pattern at `fanout.rs:135`.
- **R14 — Drop-on-peer-down log line.** `tracing::warn!(peer_node_id, space_id, event_id, "F-1b drop-on-peer-down: federation push dropped (peer unreachable; recovery via tip-exchange on next handshake)")`. Fires when (a) peer absent from registry or (b) `try_send` fails (channel full = peer slow / disconnecting).
- **R15 — Origin attach at entry points.** Client connection's `process_inbound` call at approximately `app.rs:724` passes `EventOrigin::LocallySubmitted`. Federation session's (new) `process_inbound` call passes `EventOrigin::ReceivedViaFederation`. The boundary is where the wire-source is known; downstream of that, origin flows through the dispatcher.

---

**Step coverage in Clair's 14-step implementation sequence.** The locks above govern these specific steps:

- Step 1 (`EventOrigin` enum declaration) — Lock 1.
- Step 2 (`dispatch_event` signature with `origin` parameter) — Lock 1. Threaded through for signature consistency with Option A; unused inside validation (validation is origin-uniform).
- Step 3 (`FederationPeerSenders` type + shared state) — Lock 2.
- Step 4 (`process_inbound` signature with `origin` parameter + two-comment Q3 documentation) — Lock 3 (two-comment requirement applies here).
- Step 5 (`apply_federation_push` function) — Origin check from Lock 1 at the top; F-5 §8.5 citation in code comment.
- Step 6 (wire `apply_federation_push` at client-connection's post-`apply_fanout` position) — mechanical, follows the F-1 §4.7 sibling-not-wrapper rule from §3.4.
- Step 7 (`handle_federation_incoming` Phase-3-plug-in points wired) — R12 registry lifecycle, R14 drop log line. R1's unused outbound mpsc receiver from Phase 3 (§3.3.1 Lock 3) is now plugged in.
- Step 8 (three new integration tests in `federation_push_integration.rs`) — maps to §3.4 DoD scenarios 1, 2, 3 verbatim.
- Steps 9–14 (existing test updates, runbook citations, `cargo test`, JOURNAL, CLAUDE.md + ROADMAP.md updates, commit) — standard Phase-3-pattern shape.

**[JOE-LOCK: locked 2026-05-19 (Phase 4 pre-implementation Joe-lock session)]**

All three locks above (Lock 1 through Lock 3) plus the four Clair-latitude items (R12 through R15) are canonical for Phase 4 implementation. Deviation from any of them requires a fresh Joe-lock conversation, not engineering judgement.

---

### 3.5 Phase 5 — F-1c per-peer record + reconnect scheduling

**Scope.** Implement the F-1c persistent per-peer record at the Node-implementation layer. Persisted in the federation registry. Read by reconnect scheduling. `run_initiating` gains its first production caller in `xgen-node/src/`.

**Design reference.** `docs/xgen_federation_propagation_design.md` §4.6 (F-1c).

**Files touched.** Federation registry storage layer (`xgen-node-lib::federation` or wherever the existing `peer_announcements` table lives); a new reconnect scheduler module; `run_initiating` (gains production callers).

**Schema decision.** F-1c §4.6 left this to Clair: extend `peer_announcements` with new columns, or add a sibling table. Criterion: cleaner is better. Recommendation: a sibling table is probably cleaner because the F-1c record has a different lifetime than `peer_announcements` (records survive past announcement expiry; lost-connection flag is operational state, not protocol state) — but Clair's call after looking at the existing schema.

**Required fields (F-1c §4.6 "Operational state").**

- `peer_node_id` (PK or part of composite key)
- `lost_connection: bool`
- `last_seen: RFC 3339 UTC`
- `last_successful_session: RFC 3339 UTC | None`
- `next_reconnect_attempt: RFC 3339 UTC | None`
- `operator_notes: TEXT | None` (operator-set freeform)
- `priority: INTEGER | None` (operator-set, future per-peer override)

**Reconnect scheduler.** A long-running task in the Node that periodically reads F-1c records flagged `lost_connection: true`, computes the next attempt time, and if the time has elapsed, calls `run_initiating` with the peer's endpoint. Global backoff schedule in v1 (e.g. 15 / 30 / 60 / 120 min capped). Per-peer override is a future enhancement; not required in v1.

**`run_initiating` gains production callers.** F-1 §4.7 noted this is a meaningful test-coverage delta. Today `run_initiating` is called only by tests and the stress relay (audit §2.2 found zero production callers in `xgen-node/src/`). After Phase 5, the reconnect scheduler is the first production caller.

**Definition of Done — Phase 5.**

- [ ] F-1c record schema landed (extension or sibling table — choice documented in code and commit).
- [ ] Record persisted in federation registry; survives Node restart.
- [ ] Lost-connection flag set on goodbye, keepalive failure, WebSocket error (per F-2 §5.4 session-close events).
- [ ] Last-seen and last-successful-session timestamps updated correctly.
- [ ] Reconnect scheduler task spawns at Node startup, reads F-1c records, fires `run_initiating` per global backoff schedule.
- [ ] `run_initiating` test coverage extended for the production-caller path.
- [ ] Integration test for "Node A loses connection to Node B → A's F-1c flags lost-connection → backoff schedule fires → A initiates outbound to B → handshake completes → session re-established" passes.
- [ ] Bilateral re-establishment test: same scenario but the *other* direction (Node B comes back first and initiates) also works.
- [ ] `cargo test` passes with actual test count quoted in commit message.
- [ ] JOURNAL entry written.
- [ ] Phase-5 commit pushed by Joe.

---

### 3.5.1 Phase 5 implementation locks

This subsection captures Joe-locks made during the Phase 5 pre-implementation conversation that surfaced after the §3.5 design framing. They are implementation-strategy decisions, not design decisions — the runbook keeps them separate from §3.5 to preserve concern separation. All of these decisions are canonical for Phase 5 implementation; deviation requires fresh Joe-lock.

**Note on §3.5 framing.** The §3.5 "Schema decision" paragraph framed the choice as "extend `peer_announcements` with new columns vs sibling table" — a framing that assumed a SQLite-backed federation registry. Clair's Phase 5 survey found that the actual storage is JSON-backed `FederationRegistry`, not SQLite. The actual choice is therefore between (A1) extending `FederationRelationship` directly, (A2) a new sibling type in a separate JSON file, or (A3) a new sibling type as a field inside `FederationRegistry`. Phase 8's documentation pass corrects §3.5's framing along with the related drift surfaces in Ch4 §4.11.2 and CLAUDE.md's Tier-1 file table. Phase 5 implementation proceeds against Lock A's substance, not §3.5's stale framing.

**Lock A — F-1c storage shape: A3 (`peer_records: HashMap<String, PeerOperationalRecord>` field inside `FederationRegistry`, persisted in the same JSON file).** Single file, single save site, type-clean separation through field shape rather than file separation. The operational record outlives the protocol relationship within the same registry. Forward-compat via `#[serde(default)]` matches the standard project pattern.

```rust
// xgen-core::federation::FederationRegistry (extension)
pub struct FederationRegistry {
    // ... existing fields ...
    #[serde(default)]
    pub peer_records: HashMap<String, PeerOperationalRecord>,
}

pub struct PeerOperationalRecord {
    pub peer_node_id: String,
    pub lost_connection: bool,
    pub last_seen: String,                                 // RFC 3339 UTC
    pub last_successful_session: Option<String>,           // RFC 3339 UTC
    pub next_reconnect_attempt: Option<String>,            // RFC 3339 UTC
    pub operator_notes: Option<String>,                    // operator-set freeform
    pub priority: Option<i32>,                             // operator-set, future per-peer override
}
```

The rejected alternatives were A1 (extend `FederationRelationship` directly with operational fields — mixes protocol state and operational state on one type; the existing `last_connected` field on the relationship is already operational-shaped, so extending further would make the mixing worse without introducing it for the first time; a future contributor reading `FederationRelationship` couldn't tell which fields are wire-protocol-meaningful vs runtime-only) and A2 (sibling type in a separate JSON file `xgen-node_peer_state.json` — cleanest type separation but two-file sync burden is real; the cross-failure mode where one file is corrupted while the other is intact produces silent operational degradation rather than loud error, which is the wrong failure mode for the project to accept). A3 captures F-1c §4.6's spec intent (operational record outlives relationship) without forcing the two-file shape the SQLite-imagined spec language implied.

**Lock B — Scheduler tick + backoff schedule parameters.** Four sub-decisions:

**B1 — Scheduler tick interval = 60 seconds.** Natural cadence for a backoff schedule whose first attempt is at 15 minutes; matches the project's separation from the state-writer's 5s tick (which is too aggressive for reconnect work). Each scheduler tick scans the F-1c records, fires `run_initiating` for any peer whose `next_reconnect_attempt <= now` and `lost_connection == true`.

**B2 — Backoff progression = 15 min → 30 min → 60 min → 120 min → 120 min (capped).** On each failed attempt, advance through the ladder. On successful reconnect, reset to the bottom of the ladder. Critically, "successful reconnect" means **handshake completes to ACTIVE state**, not just TCP-connect succeeding — a peer that accepts TCP but never completes the handshake should not reset the backoff. Worth being explicit in the code comment that distinguishes the two notions of "success."

**B3 — Initial delay after first observed loss = 15 minutes.** First attempt fires at `last_seen + 15min`, not immediately. The reasoning is bias toward "let the network settle before retrying" — avoids retry-storming on transient blips that may resolve via the peer's incoming handshake within seconds. Matches the runbook §3.5 wording ("15/30/60/120 min capped" reads as ladder-from-15-min, not immediate-then-15-min).

**B4 — Concurrent attempts: parallel via `tokio::spawn` per due peer per tick, detached.** Each spawned task is **detached** — the scheduler does NOT await its completion. If a `run_initiating` call hangs (peer accepts TCP but never completes handshake), the scheduler's next tick is not blocked. The hung task lives in its own future and dies when the WebSocket times out at the transport layer. Serial dispatch was rejected because head-of-line blocking on one hung peer would delay all other reconnect attempts on this Node until the WS timeout fires.

Code-comment requirement at the scheduler spawn site (verbatim):

```rust
// Reconnect attempts are spawned detached (tokio::spawn, no .await on the
// handle). The scheduler's tick MUST NOT block on any peer's run_initiating
// completion. A hung handshake dies via the WebSocket transport timeout in
// its own task; subsequent ticks proceed unaffected.
```

**Forward-compat note on `priority: Option<i32>`.** The F-1c spec includes a `priority` field for operator-set per-peer override of the global backoff schedule. v1 (Phase 5) does NOT implement priority-based override — the global schedule applies to all peers. The field exists in `PeerOperationalRecord` so future versions can plug in per-peer override semantics (e.g., `priority=high` could map to `5/10/20/40` capped, `priority=low` to `30/60/120/240` capped) without protocol or schema change. Until then, the field is read but not consulted by the scheduler. Mention this in the field's doc comment.

---

**Drift surfaces flagged for Phase 8 doc-pass (recorded for visibility; NOT Phase 5's burden).**

Clair's Phase 5 survey surfaced three documentation drift surfaces describing SQLite-backed federation storage that doesn't exist (the registry is JSON-backed). All three are corrected in Phase 8's documentation pass, not Phase 5:

1. `docs/xgen_ch4_implementation.md` §4.11.2 — describes SQLite federation storage that doesn't match the JSON-backed `FederationRegistry`.
2. `CLAUDE.md` Tier-1 file table — lists `xgen-node_federation.db` (SQLite) as a Tier-1 system file; the actual file is JSON-backed.
3. This runbook's §3.5 "Schema decision" paragraph — frames the choice as "extend `peer_announcements` with new columns vs sibling table" assuming SQLite columns; the actual choice (Lock A above) is between Rust struct extension strategies for a JSON-backed registry.

Phase 8's documentation pass updates all three to reflect the JSON-backed reality. Phase 5 proceeds against Lock A's substance directly.

---

**Step coverage in Clair's 5-step implementation sequence.** The locks above govern these specific steps:

- Step 1 (`FederationRegistry::peer_records` field + `PeerOperationalRecord` type + operational-state methods + xgen-core unit tests) — Lock A.
- Step 2 (wire `FederationRegistry` into Node startup, lifecycle hooks at `app.rs:962` register / `app.rs:1071` deregister, save-on-mutation) — Lock A storage shape feeds the lifecycle hooks; save coalescing is Clair-latitude per small-N reasoning.
- Step 3 (reconnect scheduler task spawned at Node startup, B1–B4 parameters, verbatim B4 code-comment block) — Lock B in its entirety.
- Step 4 (two bilateral integration tests: A-initiates-recovery, B-initiates-recovery) — Lock B's parallel-spawn-detached semantic is exercised by both tests; bilateral coverage closes the Phase 3 §3.3.1 Lock 7 R5 carry-over of "integration tests are the regression-locking surface for Phase 5's production caller."
- Step 5 (`cargo test --workspace` + commit + JOURNAL + push by Joe) — standard Phase-3/Phase-4 pattern shape; known-flake retry protocol applies per CLAUDE.md if `reconnect_with_existing_tip_small_delta_delivered` or the precedence env-var race fires.

**[JOE-LOCK: locked 2026-05-19 (Phase 5 pre-implementation Joe-lock session)]**

Lock A and Lock B (with B1–B4 sub-locks) are canonical for Phase 5 implementation. Deviation from any of them requires a fresh Joe-lock conversation, not engineering judgement.

---

### 3.6 Phase 6 — HeldPending generalisation for unknown signer Identity (F-10)

**Scope.** Extend HeldPending's trigger condition from "unknown predecessor" to "unknown predecessor OR unknown signer Identity OR both." The buffer waits for both dependencies. When all arrive, the event is re-routed through the validation core. Timeout policy unchanged from F-4a (30 seconds uniform).

**Design reference.** `docs/xgen_federation_propagation_design.md` §13 (F-10), §13.5 (F-10a timeout).

**Files touched.** The HeldPending buffer module (after Phase 2 moved it out of `accept_message`); the Identity replication hook (where new Identity records arrive); the validation core's "predecessor missing" check (to also check Identity-record presence).

**The two arrival hooks.** HeldPending today watches for one arrival event: predecessor arrival. After Phase 6, it watches for two:

1. **Predecessor arrival** — existing, from F-4 / Phase 2.
2. **Identity record arrival** — new in F-10. When a new Identity record lands via replication, fire a hook that re-checks HeldPending events whose missing dependency was that Identity.

The Identity-arrival hook needs to find buffered events efficiently. Suggested indexing: keep a secondary map `pending_identity_id → set<event_id_buffered>` so the arrival hook can look up the affected events in O(1). Implementation detail; Clair's call.

**Discard-on-timeout unchanged.** If the 30-second timeout fires before all dependencies arrive, the event is discarded. Recovery is via F-1a tip exchange on the next session re-establishment. F-10 §13.6 records the "Identity record never arrives" loop case as correct (loud and recoverable) behaviour.

**Observability surface.** F-10 §13.7 noted: surface "events currently in HeldPending pending Identity record" in Node-side observability so operators can see when Identity replication is the bottleneck. Exact metric design is Clair's call; this is a debugging surface, not a load-bearing feature.

**Definition of Done — Phase 6.**

- [ ] HeldPending trigger condition generalised to handle unknown-signer-Identity.
- [ ] Identity-arrival hook wired from the Identity replication subsystem.
- [ ] Buffered events with missing Identity record re-check on Identity arrival and validate-and-ingest on success.
- [ ] 30-second discard timeout still applies uniformly (F-10a same as F-4a).
- [ ] Four integration tests per F-10 §13.7: (a) Identity arrives within timeout → validates; (b) predecessors arrive within timeout, Identity arrives later but still within timeout → validates on second retry; (c) Identity never arrives, timeout fires, event discarded, next sync re-delivers; (d) both predecessors and Identity missing → event waits for both, validates when both arrive.
- [ ] Observability metric for "events in HeldPending pending Identity record" exposed (mechanism is Clair's call: log line, counter, admin-UI surface — pick one).
- [ ] `cargo test` passes with actual test count quoted in commit message.
- [ ] JOURNAL entry written.
- [ ] Phase-6 commit pushed by Joe.

---

### 3.6.1 Phase 6 implementation locks

This subsection captures Joe-locks made during the Phase 6 pre-implementation conversation that surfaced after the §3.6 design framing. They are implementation-strategy decisions, not design decisions — the runbook keeps them separate from §3.6 to preserve concern separation. All of these decisions are canonical for Phase 6 implementation; deviation requires fresh Joe-lock.

**Lock A — Where the cross-Space identity index lives: A2 (per-`PendingBuffer` secondary index, arrival hook fans out across Spaces).** Each `PendingBuffer` gains a `waiting_for_identity: HashMap<String, HashSet<String>>` parallel to its existing `waiting_for` predecessor index, keyed by `identity_id`. The Identity-arrival hook at `xgen-node/src/app.rs:1555` (`handle_identity_replicate_msg` on `Ok(())`) calls a new `NodeRuntime::drain_pending_by_identity(identity_id, origin)` method that iterates all Spaces' `PendingBuffer`s and asks each to resolve the arrived identity. Cross-Space iteration is small (~1-10 Spaces per Node at deployment scale); `O(n_spaces) ≅ O(1)`.

The rejected alternatives were A1 (single cross-Space index on `NodeRuntime: pending_by_identity: HashMap<identity_id, HashSet<(space_id, event_id)>>` — true O(1) lookup but every `PendingBuffer::add`/`resolve` must also update the `NodeRuntime`-level index, creating a sync surface across two state locations; failure mode if they desync is silent-but-self-healing message stuck-until-timeout, which is exactly the kind of silent-recovery surface D-067 was named to prevent) and A3 (per-buffer index plus `NodeRuntime`-level coarse `identity → space-set` index — middle ground that doesn't pay for the sync burden at Phase 1/2 scale; overengineered).

A2 keeps `PendingBuffer` self-contained: each buffer owns its own indices, the arrival hook is a thin loop over `NodeRuntime.pending`, not a stateful coordinator. The "true O(1)" of A1 buys nothing measurable; the no-drift-surface property of A2 is the load-bearing benefit.

**Lock B — `ValidationOutcome::HeldPending` shape: B1 (struct variant with `missing_predecessors: Vec<String>` + `missing_identity: Option<String>`).** Matches F-10's "predecessor OR identity OR both" semantic naturally: each axis has its own field, "both" is populated when both fields are set, "neither" is unrepresentable as a HeldPending case (we wouldn't be in HeldPending if nothing was missing).

```rust
// xgen-core/src/message/exchange.rs (extension)
pub enum ValidationOutcome {
    Validated(Event),
    HeldPending {
        missing_predecessors: Vec<String>,
        missing_identity: Option<String>,
    },
    Rejected(ExchangeError),
}
```

Constructor sites pass `missing_identity: None` at predecessor-only sites (the common case from Phase 2) and `missing_predecessors: vec![]` at identity-only sites. The "both" case populates both fields naturally without a third variant.

**Step 1 verification requirement: legacy `validate_steps_8_13` path.** If the legacy `validate_steps_8_13` function at `xgen-core/src/message/exchange.rs:109` is still production-reachable after Phase 2's F-4 unification, the parallel `ExchangeError::HeldPending(Vec<String>)` variant at `exchange.rs:50` needs the same struct-variant extension as `ValidationOutcome::HeldPending`. Clair verifies legacy reachability in Step 1 before deciding whether the mirror change rides Step 1 or is dead-code (no change needed). Result flagged in the commit message either way.

The rejected alternatives were B2 (enum-of-enums `HeldPending(HeldPendingReason)` with `Predecessors / Identity / Both` variants — type-safe in making "missing nothing" unrepresentable, but forces three match arms at every consumer site and the `Both` case duplicates the field shape of `Predecessors` + `Identity` combined; verbosity doesn't pay for safety improvement because B1's `Option<String>` doesn't really let "missing nothing" through either) and B3 (sigil-prefix stringly-typed encoding like `"id:xgen://..."` vs raw event_id in a single `Vec<String>` — NACK-pattern, stringly-typed, ugly).

**Lock C — Observability mechanism: C2 (counter in `xgen-node_state.json` via `build_node_state`).** The state.json file is the existing Node-observability home, written every 5 seconds by the state writer task at approximately `xgen-node/src/app.rs:413-426`. Phase 5 added the federation registry alongside; this is a sibling field. Adding `pending_identity_replication: usize` (sum across all Space `PendingBuffer`s of events with `missing_identity.is_some()`) is one line in `build_node_state` and tracks the existing convention. External monitoring tools that already poll state.json get this surface for free.

Per-Node total is the right grain for v1. Per-Space breakdown is a future enhancement if operators ever ask for it; the per-Node total is sufficient for the F-10 §13.7 use case ("is Identity replication the bottleneck right now").

The rejected alternatives were C1 (periodic log line via `tracing::info` gated on count > 0 — fine but ephemeral; operator must read logs; doesn't compose with monitoring) and C3 (both log line + counter — doubles the implementation surface without doubling the operational benefit for v1).

**Lock D — Timeout-firing behaviour with missing_identity still pending: D1 (extend `TimedOut`) + D3 (new error code) with predecessor-code-wins sub-rule for both-missing case.**

**D1 — `TimedOut` extended.** The `TimedOut` struct at `xgen-core/src/dag/pending.rs:32` gains `missing_identity: Option<String>` alongside the existing `missing_predecessors: Vec<String>`. The timeout sweep log line at `xgen-node/src/app.rs:460` extends to surface `missing_identity = <pubkey>` when set.

**D3 — New error code 4004 `identity_record_timeout`.** Allocated within domain 4000-4999 (state resolution). The operational benefit is the explicit F-10 §13.7 use case: operators filter "predecessor timeouts" from "identity timeouts" in log aggregation without parsing log payloads. Two distinct failure modes get two distinct error codes.

**Step 6 verification requirement: error code 4004 namespace.** Before locking the number, Clair greps the codebase for all currently-allocated codes in domain 4000-4999 to confirm 4003 is free and 4004 is the natural next allocation. If 4003 is already used somewhere, bump to the next free number and note the actual allocated number in the JOURNAL. Pure verification step, not a lock-worthy decision — the lock is "new error code for identity_record_timeout," not the specific integer.

**Sub-rule for the both-missing case** (event was waiting on both predecessor AND identity, both never arrived, timeout fires): the predecessor error code wins. If the predecessor was missing at time-of-timeout, the log line carries `4002 predecessor_timeout` (whether or not identity was also missing). If only the identity was missing, the log line carries `4004 identity_record_timeout`. The predecessor-first preference reflects the historically-prior code and the more common failure mode at scale. A third "both-missing" error code would be overengineering.

Code-comment requirement at the timeout-emit site (verbatim per Clair's lock acknowledgement):

```rust
// On timeout with both predecessor AND identity missing, the predecessor
// error code (4002) wins by convention — predecessor is the historically
// prior failure mode and the more common case. Identity-only timeouts get
// 4004 (identity_record_timeout). See runbook §3.6.1 Lock D sub-rule.
```

The rejected alternatives were D2 (reuse error code 4002, extend log line only — treats both flavors as "the same kind of timeout," semantically muddier; `4002`'s name `predecessor_timeout` suggests predecessor-specific behaviour) and a hypothetical "new both-missing error code" (overengineering — three codes for what's at most an "and" of two states).

---

**Drift surfaces flagged for Phase 8 doc-pass (recorded for visibility; NOT Phase 6's burden).**

Phase 6 surfaces two additional documentation drift surfaces, bringing the running total flagged for Phase 8 to **five**:

1. `docs/xgen_ch4_implementation.md` §4.11.2 — describes SQLite federation storage that doesn't match the JSON-backed `FederationRegistry` (Phase 5).
2. `CLAUDE.md` Tier-1 file table — lists `xgen-node_federation.db` (SQLite) as a Tier-1 system file; the actual file is JSON-backed (Phase 5).
3. This runbook's §3.5 "Schema decision" paragraph — frames the choice as "extend `peer_announcements` with new columns vs sibling table" assuming SQLite columns; the actual choice was Rust struct extension strategies for a JSON-backed registry (Phase 5).
4. `docs/xgen_ch4_implementation.md` §4.12.3 — Pending Event Buffer paragraph still describes pre-F-1/F-10 behaviour (predecessor-only buffering, no Identity-arrival hook, no F-1a tip-exchange recovery); needs updating to reflect the post-F-10 dual-dependency buffer and post-F-1a recovery path (Phase 6).
5. `docs/xgen_ch3_specification.md` §3.9.6 — needs a new error-code entry for `4004 identity_record_timeout` (Phase 6, assuming 4004 lands as the natural next allocation; if Clair's namespace verification surfaces a different number, the entry uses that number).

Phase 8's documentation pass updates all five to reflect post-milestone reality. Phase 6 proceeds against the locks above directly; doc drift is captured for Phase 8 cleanup.

---

**Step coverage in Clair's 8-step implementation sequence.** The locks above govern these specific steps:

- Step 1 (`ValidationOutcome::HeldPending` struct variant in `exchange.rs`, update one HeldPending-emitting site + add the new identity-missing emitting site, legacy `validate_steps_8_13` reachability verification) — Lock B with the Step 1 verification requirement.
- Step 2 (extend `PendingBuffer` with `waiting_for_identity` secondary index, new `add` signature accepting optional `missing_identity`, new `resolve_identity` method gating on both predecessors-present AND identity-present, `TimedOut.missing_identity: Option<String>`) — Lock A storage shape feeds the buffer extension; Lock D1 feeds the `TimedOut` field.
- Step 3 (`runtime.rs::dispatch_event` HeldPending arm threads new struct fields into `PendingBuffer.add`; new `drain_pending_by_identity` method mirroring `drain_pending_uniform`, iterating all Spaces per Lock A's cross-Space fan-out) — Lock A's arrival hook semantics.
- Step 4 (wire Identity-arrival hook in `xgen-node/src/app.rs::handle_identity_replicate_msg` adjacent to line 1555 — on `Ok(())`, call `rt.drain_pending_by_identity(&record.identity_id, EventOrigin::ReceivedViaFederation)` before sending the ack) — Lock A's arrival site.
- Step 5 (extend `NodeState` with `pending_identity_replication: usize`; update `build_node_state` and state.json schema) — Lock C verbatim.
- Step 6 (new error code `4004 identity_record_timeout` with namespace-verification step; update timeout sweep log line at `app.rs:460` to emit 4002 or 4004 based on which dependency is missing per the predecessor-code-wins sub-rule) — Lock D in its entirety, including the verbatim code-comment block.
- Step 7 (four integration tests per F-10 §13.7: identity arrives within timeout; predecessors first then identity; identity never arrives; both dependencies missing) — exercises the full Lock A + Lock B + Lock D surface.
- Step 8 (`cargo test --workspace` + commit + JOURNAL + push by Joe) — standard Phase-3/4/5 pattern shape; known-flake retry protocol applies per CLAUDE.md if `reconnect_with_existing_tip_small_delta_delivered` or the precedence env-var race fires.

**[JOE-LOCK: locked 2026-05-19 (Phase 6 pre-implementation Joe-lock session)]**

Lock A (A2 cross-Space fan-out), Lock B (B1 struct variant with Step 1 legacy-path verification), Lock C (C2 state.json counter), and Lock D (D1+D3 with predecessor-code-wins sub-rule and Step 6 namespace verification) are canonical for Phase 6 implementation. Deviation from any of them requires a fresh Joe-lock conversation, not engineering judgement.

---

### 3.7 Phase 7 — Federation-relationship verification gate (F-3 second check)

**Scope.** Implement the federation-relationship check that Phase 2's dispatcher pipeline shape (§7.7 step 2) reserves. For events that arrived via federation, look up the federation registry to confirm the delivering peer has an established federation relationship with this Node for the Space the event belongs to. If no relationship, reject.

**Design reference.** `docs/xgen_federation_propagation_design.md` §6 (F-3), §6.4 (the two-check ingestion gate), §6.5 (why this is not redundant with session auth).

**Why this is its own phase, separate from Phase 2.** Phase 2 establishes the dispatcher's pipeline structure with a placeholder for the federation-relationship check. The check itself depends on the federation-channel concept being meaningful — which requires Phase 4's push channel to exist. Splitting them keeps each commit reviewable; Phase 7's net effect is small (one lookup, one rejection path), but the lookup is on the hot path for every federation-received event.

**Files touched.** The dispatcher arm of `process_inbound` (Phase 2's pipeline step 2 gets its real implementation); the federation registry's lookup API.

**Hot-path concern.** F-3 §6.8 noted: "The federation registry lookup is in the hot path for every federation-received event. The runbook should consider caching or in-memory indexing if profiling shows the lookup is expensive at scale. Phase 1 / Phase 2 scale will not stress this." For v1, do the lookup straight; if profiling later shows it's slow, add caching.

**Rejection path.** When the check fails, emit a log line and (in coordination with M6 Phase 2's envelope `event_id` work) the wire-layer rejection signal. Same coordination point as F-4's rejection paths — M6 Phase 2 wires the wire-layer signal; this milestone produces the rejection site.

**Definition of Done — Phase 7.**

- [ ] Federation-relationship check implemented in the dispatcher (Phase 2's step 2 gains real logic).
- [ ] Check runs only for federation-channel events (locally-submitted events skip it).
- [ ] Failure rejects the event with a clear log line and structured error (the actual wire-layer signal arrives via M6 Phase 2).
- [ ] Integration test for "Node X with no federation relationship to Node B pushes event for Space S to B → B rejects" passes.
- [ ] Integration test for "Node A with federation relationship to B for Space S pushes event for Space S → B accepts" passes (positive case regression).
- [ ] `cargo test` passes with actual test count quoted in commit message.
- [ ] JOURNAL entry written.
- [ ] Phase-7 commit pushed by Joe.

---

### 3.8 Phase 8 — Documentation pass

**Scope.** Update Ch3 §3.3.6 to reflect that `sync_complete` is no longer "deferred" but "shipped." Update cross-references. Confirm Ch4 §4.11.3 and §4.12.3 corrections from Pass 3 of the design phase are still accurate (the federation propagation completion milestone has now closed, so the forward-references can be updated to "implemented in this milestone, see release notes / changelog").

**Design reference.** `docs/xgen_federation_propagation_design.md` §9.7 (Spec update note).

**Files touched.**

- `docs/xgen_ch3_specification.md` §3.3.6 — describe shipped `sync_complete` and pagination wire shape, remove "deferred" language.
- `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 — update from "implementation lands in the federation propagation completion milestone" to "implemented in this milestone, see commit / journal entry XXX for verification." The forward-reference to the design doc stays.
- `docs/xgen_node_admin_ops_design.md` §4.2 — same update as Ch4: "implementation lands in milestone" → "implemented in milestone, see XXX."
- `docs/xgen_federation_propagation_design.md` — bump `Last updated`, add a "Implementation Complete" note to the closure section (§14) recording the commits / phases that shipped it.

**Definition of Done — Phase 8.**

- [ ] Ch3 §3.3.6 updated for shipped `sync_complete` + pagination.
- [ ] Ch4 §4.11.3 + §4.12.3 updated from "deferred" to "shipped in milestone."
- [ ] Admin-ops design §4.2 same.
- [ ] Federation propagation design doc bumped to record implementation completion.
- [ ] All four file headers' `Last updated` lines bumped.
- [ ] No code changes in this phase (documentation only).
- [ ] `cargo test` still passes (sanity check that doc-only commit didn't accidentally break anything).
- [ ] JOURNAL entry written.
- [ ] Phase-8 commit pushed by Joe.

---

### 3.9 Phase 9 — Integration tests for full federation push path

**Scope.** End-to-end integration tests for the full federation push path against live Nodes. Two-Node smoke test (Phase-4 verification at the deployment level). Three-Node smoke test if affordable. Validation-asymmetry regression tests (post-Phase-2 + post-Phase-7 verification that federation events cannot bypass signature verification on any path).

**Files touched.** Integration test files in `xgen-client/src/` or wherever the existing `smoke-ph2` / `stress-complete` tests live.

**Test scenarios.**

1. **Two-Node federation push smoke.** Node A and Node B federate for Space S. Alice on A posts E. Verify B receives E via federation push (not via handshake dump), B's local fan-out delivers to Bob on B, the Event IDs match across both Nodes' DAGs.
2. **Three-Node federation push smoke (if affordable).** Nodes A, B, C; A federates with B and C; B and C have no direct relationship. Alice on A posts E. Verify B and C both receive E via direct A-push. Verify B does NOT push E to C (F-5 anti-transitivity check at the three-Node level).
3. **Drop-and-recover.** Node A and Node B federated. B goes down. Alice on A posts E1, E2, E3 (all dropped per F-1b). B comes back. Verify B's tip-exchange handshake delivers E1, E2, E3.
4. **Validation asymmetry regression.** A malformed-but-syntactically-valid membership.join event with a bad signature arrives at Node B via federation. Verify B rejects it (post-Phase-2 + Phase-7 the rejection is consistent across event families).
5. **Unknown-signer first-contact.** Node B accepts a fresh federation relationship with Node A. A pushes events for Space S that B's Identity registry has no records for. Verify B HeldPends them; once Identity replication catches up, the events validate and ingest (F-10 verification).
6. **Federation-relationship rejection.** Node X with no federation relationship to B pushes an event for Space S that B happens to be a member of. Verify B rejects (F-3 second check).

**Definition of Done — Phase 9.**

- [ ] All six test scenarios implemented and passing.
- [ ] Test runtime documented in commit message (the existing `smoke-ph2` / `stress-complete` tests are reference points for what "affordable" means).
- [ ] `cargo test` passes with actual test count quoted in commit message.
- [ ] JOURNAL entry written, including the integration test output (per CLAUDE.md Rule 2 — quote actual output).
- [ ] Phase-9 commit pushed by Joe.

**Why this is its own phase.** Each preceding phase had its own unit + small-scale integration tests. Phase 9 is the deployment-level proof — multiple Nodes, real wire-protocol traffic, end-to-end verification that the milestone closed the audit's HIGH-severity findings. The milestone is only complete when Phase 9 passes.

---

### 3.10 Phase ordering summary

| Phase | Blocks | Why |
|---|---|---|
| 1 | 4, 6 | Phases 4 and 6 depend on `sync_complete` + pagination being available |
| 2 | 4, 6, 7 | Validation core is the precondition for federation push; HeldPending move precedes generalisation; dispatcher structure precedes F-3 check |
| 3 | 4 | Persistent session must exist before push can use it |
| 4 | 5, 9 | Per-peer record needs the push channel to monitor; integration tests need the channel |
| 5 | 9 | Reconnect scheduling exercises end-to-end during integration tests |
| 6 | 9 | F-10 verification is one of the Phase 9 scenarios |
| 7 | 9 | F-3 verification is one of the Phase 9 scenarios |
| 8 | 9 | Documentation precedes the "milestone complete" closing |
| 9 | — | Final |

Sequential execution: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9. Phases 5–7 are *somewhat* parallelisable but the runbook recommends sequential to keep each commit reviewable. Phase 8 is doc-only and could in principle land anywhere after Phases 1–7 ship, but landing it last keeps the doc state consistent.

---

## 4. Per-phase Definition of Done — meta

Each phase's DoD checklist above lists the phase-specific verification. The following items apply to **every** phase:

- [ ] Tests passing (`cargo test`) with actual count quoted from real terminal output per CLAUDE.md Rule 5.
- [ ] JOURNAL.md entry written *after* the work is verified, quoting actual command output per Rule 2.
- [ ] CLAUDE.md updated if the phase changes project state (test count, milestone block status).
- [ ] ROADMAP.md updated in the same commit as CLAUDE.md per project discipline.
- [ ] Commit pushed by Joe — Clair never pushes directly per project rule.
- [ ] No `commit pushed` checkbox in any task file's DoD per project rule — the `Status: COMPLETED` header is the real ship signal.

When Phase 9 closes, the Federation Event Propagation completion milestone is shipped. CLAUDE.md and ROADMAP.md flip the milestone block from 🟢 PLAY to ✅ DONE in the same commit as Phase 9 ships.

---

## 5. Cross-references

### 5.1 Design documents (source of truth)

- **`docs/xgen_federation_propagation_design.md`** (Status: ACTIVE, v1.0) — the canonical design. Every phase of this runbook implements something from this document. Read it first.
- **`docs/xgen_propagation_reliability.md`** (J-081 audit, ARCHIVED) — the audit doc that motivated the milestone. Read this to understand why the work exists.

### 5.2 Coordinating milestones

- **M6 (new) `docs/xgen_node_admin_ops_design.md`** (Status: ACTIVE, v1.0) — the admin write path milestone. M6 (new) Phase 2 lands the envelope-level `event_id` on `TransportMessage::Error` and wires it into the rejection paths that this milestone's Phases 2 and 7 produce. Coordination is at the rejection-signal interface. M6 (new) is blocked behind this milestone going DONE.

### 5.3 Spec chapters

- **`docs/xgen_ch3_specification.md`** §3.3.6 — `sync_request` / `sync_response` / `sync_complete`. Phase 8 updates this to reflect shipped state.
- **`docs/xgen_ch4_implementation.md`** §4.11.3 + §4.12.3 — federation fan-out and pending-buffer paragraphs. Pass 3 of the design phase corrected these to forward-references; Phase 8 updates them once the milestone ships.

### 5.4 DECISIONS

- **D-065** — Honest behaviour over polite behaviour. Cited multiple times throughout the design (F-6, F-7, F-10). Implementation work must respect it: every phase has rejection paths that surface honest errors, not silent fallbacks.
- **D-067** — Single source of truth for command implementations. The M5 `ops::*` refactor precedent that F-4's Option 1 (unified validation core) follows. Phase 2 is the M5-shaped fix for the validation-pipeline layer.
- **D-068** — CLI flag > config precedence. Phase 1's new config fields (`[sync].completion_timeout_seconds`, `[sync].batch_size`) wire through `xgen-common::precedence::resolve_setting<T>`.
- **D-069** — Joe-locked design phase + canonical-document rule. This runbook is the canonical-document-rule realisation: one implementation runbook against the one canonical design doc.

### 5.5 Pass 3 task file

- **`tasks/FEDERATION_PROPAGATION_PASS_3.md`** (COMPLETED at Pass 3 close) — the task file that produced this runbook plus the design doc consolidation and F-8/F-9 corrections. Historical record.

---

## 6. Operating discipline (restated from CLAUDE.md)

These rules apply to every phase of this runbook. They are restated here so they are at hand without context-switching to CLAUDE.md.

**Rule 1 — Never fabricate results.** If a command fails, report the failure. Do not describe what the output *should* have been. Do not write a journal entry claiming success until success is actually confirmed.

**Rule 2 — Show actual output, not a description of output.** Every verification step requires quoting real terminal output in the journal entry. Do not paraphrase. Paste the actual lines.

**Rule 3 — Stop and report when a tool fails.** If a command, file operation, or any tool call fails or returns an unexpected result: stop, report exactly what failed and the error, do not work around silently, do not write a success summary. Joe decides how to proceed.

**Rule 4 — Write the journal entry last.** JOURNAL.md entry is written *after* all work is complete and all verification steps are confirmed with real output. Order: work → verification → confirm outputs → journal entry quoting actual output → CLAUDE.md update → commit + push (Joe pushes).

**Rule 5 — Never invent numbers.** Test counts, file counts, line counts — these come from actual command output. If you did not run `cargo test`, you do not know the current test count — say so.

**Rule 6 — When in doubt, do less and ask.** If a task instruction is ambiguous or completing it would require a decision not covered, stop and flag the ambiguity. Do not decide silently.

**Rule 7 — Definition of Done is a checklist, not a formality.** Every phase has its own DoD checklist. Each item must be independently verified before being marked complete.

**No `commit pushed` checkbox in any task file.** The `Status: COMPLETED` header is the real ship signal.

**Joe pushes; Clair does not.** When PowerShell pushes are needed, Chat Claude generates the sequence; Joe executes via GitHub Desktop or PowerShell directly. Clair never runs `git push`.

---

## 7. Test count tracking

| Milestone marker | Test count | Source |
|---|---|---|
| Runbook handoff (2026-05-18) | 468 | J-081 close |
| Phase 1 close | TBD (Phase 1 commit message) | actual `cargo test` output |
| Phase 2 close | TBD | actual `cargo test` output |
| Phase 3 close | TBD | actual `cargo test` output |
| Phase 4 close | TBD | actual `cargo test` output |
| Phase 5 close | TBD | actual `cargo test` output |
| Phase 6 close | TBD | actual `cargo test` output |
| Phase 7 close | TBD | actual `cargo test` output |
| Phase 8 close | TBD (no code change; same as Phase 7 close) | actual `cargo test` output |
| Phase 9 close — milestone shipped | TBD | actual `cargo test` output |

Numbers are filled in as each phase ships. The table is the running record of the milestone's growth.

---

## 8. Validation asymmetry as precondition — explicit

This section exists to make the Phase 2 → Phase 4 ordering hard.

**The asymmetry today (J-081 audit §3 finding).** `process_inbound` validates messages (Path A) through the full 13-step pipeline including signature verification, timestamp checks, and HeldPending. It does NOT do this for membership.join (Path B) or other state events (Path C). Paths B and C reach `ingest_event` directly after two pre-checks, bypassing signature verification entirely.

**Why this is LOW today.** Locally-authenticated submission is the only entry point. A client connection's events are already vetted by transport-layer authentication; the missing signature check on Paths B and C is real but unreachable by external attack.

**Why this becomes HIGH the moment Phase 4 ships.** Federation push is the exact vector that makes Paths B and C reachable with externally-sourced events. A federated peer (or compromised peer) pushes a membership.join event with a forged signature; Path B accepts without verification; the receiver's DAG now contains an unauthenticated state change.

**Phase 2 closes the asymmetry. Phase 4 lands the vector.** Order matters absolutely. Shipping Phase 4 before Phase 2 closes is shipping a known vulnerability.

**The runbook's hard rule:**

> Phase 4 MUST NOT start before Phase 2 ships, verifies, and Joe pushes the Phase-2 commit.

If during implementation any temptation surfaces to ship Phase 4 first (because "it's just a refactor, we can do it later") — stop and re-read this section. The audit's foundational reason for existing was that Stage 6 reliability claims need Stage 6 to be safe, and Phase 4 lands Stage 6. Phase 4 without Phase 2 is unsafe.

**Coordination with M6 (new) Phase 2.** M6 (new) Phase 2 is blocked behind this milestone going DONE. So M6 (new) Phase 2 cannot ship its wire-layer rejection signal until Phase 4 of this milestone is also live. The dependency chain is:

```
Phase 2 (this milestone)  →  Phase 4 (this milestone)  →  M6 (new) Phase 2 (next milestone)
```

The runbook does not attempt to ship M6 Phase 2 inline. M6 is its own milestone.

---

## 9. D-070 — SHIPPED to DECISIONS.md (2026-05-18)

D-070 ("Two events of equal importance, opposite direction") was originally drafted in `docs/xgen_node_admin_ops_design.md` §9 as a Pass-3 proposal. It was promoted to a numbered DECISIONS.md entry on 2026-05-18 in a same-day post-audit recording session with the corrected post-audit framing (both halves load-bearing: existence AND envelope-level `event_id` correlation).

**Canonical reference:** `DECISIONS.md` D-070. The M6 design doc §9 is SUPERSEDED with the original Pass-3 framing preserved as historical record.

**Why this runbook still flags it.** Phase 2's rejection paths and Phase 4's federation-push paths are the implementations of the symmetry D-070 names. When a future contributor reads Clair's commits and asks "why does the protocol have both `Error` and `EventAccepted` and a federation-relationship rejection path?", D-070 is the answer. The DECISIONS.md entry makes the citation durable.

**Coordination at this milestone:** F-4 (Phase 2) produces the rejection sites consistently across all event families. M6 (new) Phase 2 wires those rejection sites to the wire-layer signal with envelope-level `event_id`. Both halves of D-070 land in coordinated milestones — the symmetry is realised at the moment both ship.

---

## 10. D-071 — SHIPPED to DECISIONS.md (2026-05-18)

D-071 ("Subsystem audits precede dependent milestones") was promoted to a numbered DECISIONS.md entry on 2026-05-18 in a same-day post-D-070 recording session. The pattern emerged organically during the Propagation Reliability Audit (J-081) when findings consistently exceeded the audit's nominal scope and the audit became Pass 1 input for two downstream design phases (M6 Phase 0, Federation Event Propagation Phase 0).

**Canonical reference:** `DECISIONS.md` D-071. Sibling to D-065 (honest behaviour over polite behaviour) and D-070 (two events of equal importance). D-065 and D-070 are protocol-design principles; D-071 is the project-management analogue.

**Why this runbook still flags it.** This milestone is one of D-071's two worked instances at promotion (the other is M6 Phase 0). The audit ran, found HIGH-severity gaps, the milestone exists to close those gaps, the runbook implements the closure. When future readers ask "what's an example of an audit-driven milestone," this runbook is the answer.

**No action required in this runbook.** D-071's promotion is shipped; this section exists for cross-reference visibility only.

---

*End of runbook. Implementation Phase 1 starts when Clair picks this up. Until then: design phase closed, runbook handoff effective at commit.*  
