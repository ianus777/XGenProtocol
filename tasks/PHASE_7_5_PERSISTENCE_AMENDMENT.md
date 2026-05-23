# Phase 7.5 Persistence Amendment — Drain-Without-Persist Audit
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-23 (Status flipped ACTIVE → COMPLETED v1.1 at design-phase close per audit §1's named milestone shape. Design phase produced `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` (ACTIVE v1.0) with four Joe-locks recorded: Q1 at (a).ii + (a).iii.β + candidate D-NNN flag (sort-on-replay + `ingest_event` returns `Result<(), GraphError>` + "ingest path invariant encoding" flagged for future walk); Q2 at (a) return-vector (`DispatchOutcome::Accepted { new_joiner, additional_persisted: Vec<Event> }`); Q3 at all-three drain helpers; Q4 at (a) in-scope (sentinel-tree ships atomic at milestone close as activating regression lock; Commit 3b-1 collapses into milestone close). No re-walk fired during Q1→Q4 walk per Lock #2 discipline. Audit doc body §1–§11 stays authoritative as historical record of audit-at-lock-time per topo-sort + bidirectional precedents — NOT ARCHIVED, because milestone-internal sub-amendment audits have a different artifact lifecycle than the project-wide subsystem audit at `docs/xgen_propagation_reliability.md` (J-081). Sibling-shape COMPLETED-at-design-close precedents: `tasks/FEDERATION_TOPOSORT_AUDIT.md` (COMPLETED v1.1) + `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` (COMPLETED v1.0). Per D-065 + D-069 + D-071 honest-behaviour-over-polite-behaviour discipline. Previous 2026-05-23 audit-doc-open content stands authoritative as historical record — see audit-doc-open paragraph below.) Previous 2026-05-23 update: Audit doc authored at session-arc open per J-104's Path 1 Joe-lock: open the sub-amendment milestone at this file, retain sentinel working tree uncommitted, single-file atomic commit per J-098 atomicity lesson. Pattern name "Drain-without-persist gap at the `xgen-core` ↔ `xgen-node` layer boundary" Joe-locked at J-104 session close. Eleven sections sibling-in-shape to `tasks/FEDERATION_TOPOSORT_AUDIT.md` (COMPLETED v1.1) and `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` (COMPLETED v1.0). Audit doc surfaces the gap surface, the secondary layered silent-discard surface, the cascade trace, the originating architectural move, and a four-question set carried open to design phase per D-069 audit-vs-design boundary discipline. Status flips ACTIVE → COMPLETED at design close, sibling to bidirectional + topo-sort precedents.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Purpose, pattern name, and milestone shape

**Pattern name (Joe-locked at J-104 session close).** "Drain-without-persist gap at the `xgen-core` ↔ `xgen-node` layer boundary."

**Purpose of this audit doc.** Characterise the full surface of a structural gap in the Federation Event Propagation milestone's persistence layer, surfaced during Phase 9 Commit 3b-1 implementation when Clair's Scenario 3 (`two_node_drop_and_recover_two_cycles`) failed at the cycle 0 → cycle 1 boundary with a predecessor-missing cascade. The gap is between (a) drain hooks in `xgen-core/src/node/runtime.rs` that re-dispatch released events through full validation + ingest, and (b) the persistence call site in `xgen-node/src/app.rs::process_inbound` that only persists explicitly-passed events. Drained events become visible in B's in-memory `EventStore` + `SpaceState` but never hit disk; on Node restart, `replay_spaces_from_dir` only sees the persisted subset, producing a destructive live-store-vs-disk divergence.

**Milestone shape (D-071 four-phase, sibling to bidirectional + topo-sort).**

1. **Audit phase** (this session-arc) — this document at v1.0 ACTIVE. Names the surface, evidence, and open question set; does not lock answers per D-069 audit-vs-design boundary.
2. **Design phase** (next session-arc) — Joe-lock the four open questions in §7; design task file authored per topo-sort precedent (separate file at design open, not folded into this audit doc). New D-NNN candidate at design close if Q2(c) is chosen (callback shape would be the fifth no-drift-surface family member alongside D-067/D-070/D-075/D-076).
3. **Implementation phase** (one ROADMAP-state phase across two session-arcs — runbook authoring then Clair impl, per topo-sort precedent) — runbook in `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md`; multi-commit Clair-facing sequence per chosen design.
4. **Milestone close** (one session-arc) — atomic per D-074. Sentinel working tree finally lands; Commit 3b-1 of Phase 9 ships at the same or immediate-next commit with Scenarios 2 + 3 both green.

**Sibling-in-shape precedents.** `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` (COMPLETED v1.0) and `tasks/FEDERATION_TOPOSORT_AUDIT.md` (COMPLETED v1.1) — both milestone-internal audits inside Federation Event Propagation milestone scope. Both flipped ACTIVE → COMPLETED at design close. Both retained authoritative as historical record of audit-at-lock-time after design landed.

**Distinct artifact lifecycle from J-081.** `docs/xgen_propagation_reliability.md` (J-081 audit, ARCHIVED) is the project-wide subsystem audit at top of the D-071 stack — the motivating audit for the Federation Event Propagation milestone itself. This audit is a milestone-internal sub-amendment whose lifecycle ends at design close, not at milestone close.

---

## §2 — The primary surface: drain hooks in `xgen-core/src/node/runtime.rs`

Three drain hooks exist in `xgen-core/src/node/runtime.rs`, added across three implementation phases of the Federation Event Propagation milestone. Each re-dispatches released events through full validation + ingest via `let _ = self.dispatch_event(ev, origin, None);`. The `let _ =` pattern discards the `DispatchOutcome` — including the `Accepted` variant that would be the signal to persist.

**Call site 1 — `drain_pending_uniform`** at `xgen-core/src/node/runtime.rs:~670-682` (Phase 4 / F-4a — predecessor resolution). Released when a newly-accepted event satisfies the predecessor dependency of a previously-buffered event. Re-dispatches the released event through `dispatch_event` with the original `EventOrigin` preserved. Iterates inside `dispatch_event` Step 6.

**Call site 2 — `drain_pending_by_identity`** at `xgen-core/src/node/runtime.rs:~745-760` (Phase 6 / F-10 — unknown-signer resolution). Released when an Identity record arrives that was previously the missing dependency for one or more buffered events across Spaces. Cross-Space fan-out per Phase 6 Lock A2; iterates `self.pending` keys.

**Call site 3 — `drain_pending_by_federation_relationship`** at `xgen-core/src/node/runtime.rs:~795-810` (Phase 7.5 / F-3 resolution). Released when a `state.federation_add` event newly establishes a federation relationship that satisfies the F-3 federation-relationship check for previously-buffered events. The hook surfaced during J-104's Scenario 3 implementation as the entry point for the cascade — buffered `state.room_create` + `membership.invite` + `membership.join` events get released through this hook after `state.federation_add(A→B)` lands.

**The persist call site (xgen-node layer).** `xgen-node/src/app.rs::process_inbound` at lines 1503-1505 inside the `DispatchOutcome::Accepted` match arm:

```
DispatchOutcome::Accepted { space_id: space_id_for_persist, .. } => {
    if let Err(e) = persist_event(spaces_dir, &space_id_for_persist, &event).await {
        // log + continue
    }
}
```

`process_inbound` calls `dispatch_event` once with the explicitly-received event and persists exactly that event on `Accepted`. The drain helpers internally call `dispatch_event` recursively for the released events but their `Accepted` outcomes are silently dropped via `let _ =`. The persist call only fires for the event that arrived from the wire, never for the events the drain helpers release.

**Why the drain helpers can't reach `persist_event`.** `persist_event` lives in `xgen-node` (the I/O crate). The drain helpers live in `xgen-core` (the pure protocol-logic crate). Calling `persist_event` from inside `xgen-core` would require either (a) a reverse dependency (xgen-core depends on xgen-node — not architecturally legitimate), (b) the persist call moved to xgen-core (violates xgen-core's no-I/O design constraint from D-022 / D-044), or (c) some form of indirection through callback / return-vector / re-lifted architecture. The gap is structural at the crate boundary; the choice of indirection is a design-phase decision (Q2 in §7 below).

---

## §3 — The secondary layered surface: silent-discard at `runtime.rs:181`

A second silent-error pattern exists at `xgen-core/src/node/runtime.rs:181` inside `ingest_event`:

```
let _ = graph.add_event(&event, store);
```

`graph.add_event` is defined at `xgen-core/src/dag/graph.rs:105-110` and returns `GraphError::UnknownPrevEvent` when a non-root event's predecessor isn't in the store. The `let _ =` pattern in `ingest_event` silently discards this error: the event lands in `EventStore` and `SpaceState` (via the applier path) but does NOT register in the DAG `Graph` for tip computation purposes.

**Why this layered surface matters.** Without this silent-discard, the primary drain-without-persist gap would surface as a panic on respawn rather than a silent live-store-vs-disk divergence. Replay would attempt `graph.add_event` for an event whose predecessor (held in memory only at cycle 0, never persisted) is missing from the on-disk store; the silent-discard turns "missing predecessor on replay" into "event accepted but invisible to tip computation", which means cycle 1's federation tip exchange sends the wrong tip set, which means A re-streams events that B already has in its in-memory state from cycle 0 but doesn't have on disk, which means the cascade repeats.

The silent-discard is what makes the primary gap **destructive-on-restart** rather than **merely lossy**. A lossy gap would surface immediately at cycle 0 close as inconsistent state and an honest panic; a destructive gap surfaces only on restart, looks like the system is working until it isn't, and produces production failure modes whose root cause is two layers below the symptom.

**Sibling-shape to topo-sort Commit 2a's two layered B3 surfaces (J-101).** Second project-wide instance of the layered-B3 pattern at the audit level. Topo-sort Commit 2a's pattern: primary fix at `xgen-core/src/space/state.rs:797` (`build_room_create_event` constructor) surfaced two sibling-layer encodings of the DAG-root invariant (`is_dag_root_type` at `graph.rs:29` and `validate_dag_structure` at `exchange.rs:550`); Option E unification per D-067 closed both atomically. This audit's pattern: primary fix at the drain-hook layer surfaces a secondary silent-error encoding (`graph.add_event` UnknownPrevEvent silent-discard) at a sibling layer (the DAG-graph ingest layer beneath the applier layer).

Two instances is not yet a durable pattern — three would be. The shape is "primary fix surfaces a secondary silent-error encoding of the same invariant at a sibling layer." Future audits should look for layered surfaces but not pre-assume their presence.

---

## §4 — Grep evidence + architectural fact

**Grep at audit-authoring time.**

```
grep -n "persist_event" xgen-core/src/node/runtime.rs
(no results)

grep -rn "persist_event" xgen-core/
(no results across the entire xgen-core crate)
```

The entire `xgen-core` crate has no `persist_event` references and no I/O whatsoever. This is a deliberate design property recorded in D-022 (crate-split decision) and D-044 (xgen-core library-crate creation). `xgen-core` holds pure protocol logic; `xgen-node` holds the I/O shell that wraps it.

**Why the gap is structural, not accidental.** The architecture chose to put a wall at the `xgen-core` ↔ `xgen-node` boundary, exactly where persist-vs-no-persist separates. The drain-hook lift at Phase 7.5 Commit 3.5 (J-103) moved code that needed to reach across the wall to the wrong side of the wall. The gap is exactly where the architecture chose to put the boundary, but the drain hooks now live on the side without the I/O primitive they need.

Naming this honestly: the architecture is correct; the lift was not unsound at the layer it was solving for (B3-shape gap closure at Commit 4 integration time per J-103); the second-order consequence at the persistence layer was unobserved. This is not an indictment of D-022 / D-044 or of the Commit 3.5 lift. It is a record that the lift's structural surface area extended further than the B3-shape gap it was directly addressing.

---

## §5 — Cascade trace from cycle 0 → cycle 1

The 30-line log capture from J-104, captured during the `cargo test --workspace` run that exercised `xgen-node/src/tests/phase9_drop_and_recover.rs::two_node_drop_and_recover_two_cycles`. Anchored here as ground-truth evidence; named as the failure-mode signature.

```
2026-05-23T11:32:03.711867Z  INFO  Federation handshake reached ACTIVE  shared_spaces_count=1
2026-05-23T11:32:03.711983Z  INFO  Federation delta delivery complete  role=Receiver
2026-05-23T11:32:03.713439Z  INFO  Federation delta delivery complete  role=Initiator
2026-05-23T11:32:03.720605Z  DEBUG F-5 anti-transitivity: skipping ... event_id=...81d501fb...
2026-05-23T11:32:03.720673Z  DEBUG event buffered — waiting for unknown prev_events  event_id=...11bfd97b... event_type=state.room_create
2026-05-23T11:32:03.720705Z  DEBUG event buffered — waiting for unknown prev_events  event_id=...fa74fd14... event_type=membership.invite
2026-05-23T11:32:03.720729Z  DEBUG event buffered — waiting for unknown prev_events  event_id=...9c1ccdfd... event_type=membership.join
2026-05-23T11:32:03.747595Z  DEBUG F-5 anti-transitivity: skipping ... event_id=...ea783490...   (← fed_add accepted)
2026-05-23T11:32:03.748659Z  INFO  Federation session ended  role=Initiator                   (← shutdown_keep_data triggered)
2026-05-23T11:32:03.754214Z  WARN  F-1b drop-on-peer-down: federation push dropped (peer unreachable) ... event="federation_push_dropped_unregistered" event_id=...69cd02e2...
...4 more R14 drops...
2026-05-23T11:32:03.845460Z  INFO  Incoming federation connection                            (← cycle 0's respawn)
2026-05-23T11:32:03.859187Z  INFO  Reconnect: handshake reached ACTIVE
2026-05-23T11:32:03.865753Z  INFO  Federation handshake reached ACTIVE  shared_spaces_count=1
2026-05-23T11:32:03.865859Z  INFO  Federation delta delivery complete  role=Receiver
2026-05-23T11:32:03.869130Z  INFO  Federation delta delivery complete  role=Initiator
2026-05-23T11:32:03.869661Z  DEBUG event buffered — waiting for unknown prev_events  event_type=state.room_create   (← A re-sent it; B doesn't have prev)
2026-05-23T11:32:03.869748Z  DEBUG event buffered — waiting for unknown prev_events  event_type=membership.invite
2026-05-23T11:32:03.869813Z  DEBUG event buffered — waiting for unknown prev_events  event_type=membership.join
2026-05-23T11:32:03.876740Z  DEBUG F-5 anti-transitivity: skipping ... event_id=...ea783490...   (← fed_add re-accepted)
2026-05-23T11:32:03.876853Z  DEBUG event buffered — waiting for unknown prev_events  event_type=message.text   (cycle-0 dropped events arrive but predecessor still missing)
2026-05-23T11:32:03.877330Z  DEBUG event buffered — waiting for unknown prev_events  event_type=message.text
...3 more message.text buffered...

thread 'tests::phase9_drop_and_recover::tests::two_node_drop_and_recover_two_cycles' (42284) panicked at xgen-node\src\tests\phase9_drop_and_recover.rs:331:17:
cycle 0: B did not receive dropped event xgen://hash/sha256:69cd02e2... via F-1a delta within 60s
```

**Failure-mode signature.** Cycle 0 accepts events via drain hooks → `shutdown_keep_data` triggered → replay misses drained events because they were never persisted → cycle 1's bootstrap chain re-fails because the predecessor chain is incomplete on disk → message.text events HeldPending on missing `join` membership state → test times out at 60s per-event.

**The graph-add-event silent-discard's role in the cascade.** During cycle 1's replay, `fed_add(A→B)` ingests into `EventStore` and `SpaceState` (federation_nodes populated correctly via the vantage-aware applier per D-075) but does NOT register in the DAG `Graph` because its predecessor (`join.event_id`) isn't in store. B's `dag_tips` returns `[space_create.event_id]` — `fed_add` is invisible to tip computation. On cycle 1's `federate(A, B_new)`, B sends Hello with `tips = {S: space_create.event_id}` rather than `fed_add.event_id`; A computes a stale delta; the cascade repeats.

This is §3's layered silent-discard surface firing during cycle 1 replay — not at cycle 0 close. The cycle 0 close looks like a successful drain (in-memory state is correct); the failure surfaces only when the replay path traverses the disk-only subset of state.

---

## §6 — Originating architectural move: Phase 7.5 Commit 3.5 drain-hook lift

J-103 retrospective records the Phase 7.5 Commit 3.5 architectural move: the drain hook for F-3 resolution was lifted from `xgen-node/src/app.rs::process_inbound` into `xgen-core/src/node/runtime.rs::dispatch_event` Step 7. Pre-lift, drain happened at the `xgen-node` layer where `persist_event` was natively reachable. Post-lift, drain runs at the `xgen-core` layer where it isn't.

**Why the lift happened.** B3-shape gap surfaced at Phase 7.5 Commit 4 integration time. Two production gaps in close-proximity layers: predecessor-chain deadlock + step-11 sender-membership rejection. The lift solved these by moving the drain hook to a structurally sounder layer (`dispatch_event` Step 7 instead of post-validation in `process_inbound`). At the layer the lift was solving for, it was correct and remains correct.

**The second-order consequence.** The pre-lift architecture's `process_inbound` drain hook ran inside the `Accepted` match arm's scope, where `persist_event` was the natural next call. Post-lift, the drain hook runs inside `dispatch_event` itself — at a layer where the persist primitive doesn't exist. The lift's structural surface area extended further than the B3-shape gap it was directly addressing; the persistence consequence was unobserved at Phase 7.5 Commit 3.5 review and at every subsequent verification round until J-104's Scenario 3 implementation surfaced it.

**Not a criticism of J-103 / Commit 3.5.** The lift was structurally correct for its problem. The audit's job is to record honestly that solutions can have unobserved consequences at adjacent layers; the project's response is to surface those consequences when they appear, not to retroactively reject the original move. This is the same pattern J-101's two layered B3 surfaces followed: Commit 2a's primary fix at `build_room_create_event` was correct; the validator-layer encodings it surfaced at `is_dag_root_type` and `validate_dag_structure` needed Option E unification per D-067; neither the primary fix nor the validator layer was unsound, but the unification was load-bearing for codebase coherence.

---

## §7 — Scope question set carried to design phase

Per D-069 audit-vs-design boundary discipline, this audit doc names questions and does not lock answers. Design phase decides.

**Q1 — Silent-error pattern at `runtime.rs:181` (`graph.add_event` UnknownPrevEvent silent-discard).** Three options carried open:

- **(a) Fix in this milestone.** Address as a sibling-layer fix inside the persistence-amendment milestone scope. Sibling-shape to topo-sort Commit 2a's two-layered-surface atomic close per D-067 Option E.
- **(b) Fix in a follow-on D-071 arc.** Treat as adjacent surface that needs its own audit → design → impl pass. Sibling-shape to how bidirectional `federation_nodes` opened mid-Phase-9 as a separate D-071 arc rather than being folded into Phase 9's scope.
- **(c) Determine load-bearing-by-design and document the invariant.** The silent-discard may be deliberate defensive behaviour with valid invariants we don't want to break — e.g. tolerating partial-state during legitimate replay-mid-stream scenarios. Until design enumerates call sites and invariants, we don't know. If (c) is chosen, the audit's call-site enumeration becomes a feed for a doc-comment block at `graph.add_event` or `ingest_event` explaining what the silent-discard is load-bearing for.

Design phase decides. Three-option framing preserves the load-bearing-by-design path explicitly; two-option framing ("fix vs. defer") prematurely excludes it.

**Q2 — Three candidate fix shapes for the primary gap.** All named with structural pros/cons; design picks one.

- **(a) Return-vector.** `dispatch_event` returns `Vec<Event>` of additionally-ingested events for `process_inbound` to persist. The persist responsibility stays at `xgen-node`; xgen-core stays I/O-free.
  - **Pros.** Preserves layer separation cleanly. No new abstraction. Signature change is mechanical at call sites.
  - **Cons.** Changes `dispatch_event` signature — touches every existing call site (16+ per J-088 count). Shifts coupling: callers now have to know that `dispatch_event` may produce N additional events for persistence; if a caller forgets to persist the vector, the gap re-opens silently. Drain-hook ordering becomes caller-visible.

- **(b) Drain-hook re-lift.** Revert Phase 7.5 Commit 3.5 — lift drain back to `xgen-node`. The persist call site becomes natively reachable from drain hooks again.
  - **Pros.** Restores the pre-Phase-7.5-Commit-3.5 architecture's natural persist reachability. Minimal new code; well-understood shape.
  - **Cons.** Reverses the B3 fix that motivated the lift per J-103. Design phase needs to confirm whether the original B3-shape gap remains closed under a re-lifted architecture — possibly closed by other Phase 7.5+ changes, possibly not. May reopen latent surfaces the lift's three-month operational history has implicitly verified are closed.

- **(c) Persistence callback.** Persistence callback parameter through `dispatch_event` (e.g. `dispatch_event(... , persist_callback: Option<&dyn PersistCallback>)`). The callback is invoked from inside drain hooks for each released event's `Accepted` outcome.
  - **Pros.** Preserves both layer separation and signature stability (callback is optional). Abstracts the I/O concern correctly: xgen-core knows there's a persist hook; doesn't know what it does.
  - **Cons.** Injects an I/O concern (via abstraction) into the pure-protocol crate. Would be the fifth no-drift-surface family member alongside D-067 (code-organisation) / D-070 (transport) / D-075 (event-model) / D-076 (wire-format). Adds a layer of indirection at every call site.

Design phase weighs trade-offs and locks. A D-NNN candidate may emerge at design close if (c) is chosen (new no-drift-surface family member); (a) and (b) likely don't motivate new D-NNN promotions.

**Q3 — Drain-hook scope.** Three drain helpers identified in §2:

- (i) `drain_pending_uniform` — Phase 4 / F-4a
- (ii) `drain_pending_by_identity` — Phase 6 / F-10
- (iii) `drain_pending_by_federation_relationship` — Phase 7.5 / F-3 (surfaced the cascade)

In-scope: just (iii), or all three? All three share the same architectural shape (re-dispatch via `let _ = self.dispatch_event(...)` with the persist outcome discarded), so any fix shape applied to (iii) should naturally extend to (i) and (ii). But the cascade trace exercises only (iii); applying the fix to all three at once might widen the surface beyond what Scenario 3 verifies. Design phase decides.

**Q4 — Sentinel working tree's `phase9_harness.rs` extension.** The `shutdown_keep_data` machinery + `SavedNodeState` carrier + `spawn_in_process_node_with_state` + `abort_connection_tasks` helper authored during J-104's Scenario 3 implementation: in-scope for this milestone (it's load-bearing for verifying the fix and may need refinement as the design phase surfaces edge cases), or already-shipping-via-Commit-3b-1's-eventual-commit (treat as fixed at J-104's snapshot, milestone close inherits as-is)?

Design phase decides. The choice affects the milestone-close commit's file count and the runbook's verification rigour requirements.

---

## §8 — Sentinel working tree as verification contract

Four files retained uncommitted at J-104 session close, listed here explicitly with as-of-J-104 anchor.

**File 1 — `xgen-node/src/tests/phase9_harness.rs` (modified).**

Adds harness machinery for restart-with-data-preservation testing:

- `SavedNodeState` carrier struct holding paths to data_dir, identity registry, federation registry, and Space event stores
- `InProcessNode::shutdown_keep_data(&mut self) -> SavedNodeState` method — clean async shutdown via `shutdown_tx.send(true)` without deleting `data_dir`; returns the carrier
- `spawn_in_process_node_with_state(saved: SavedNodeState) -> InProcessNode` constructor — replays identity registry + Space event stores + federation registry from disk; binds fresh `127.0.0.1:0` per Pre-Commit-3b-1 Joe-lock Q1 option (b) to avoid TIME_WAIT non-determinism
- `connection_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>` field on `InProcessNode`; `abort_connection_tasks` helper invoked by both `shutdown` and `shutdown_keep_data` (required so the WS socket actually closes on shutdown — prior implementation only stopped the accept loop, leaving in-flight `handle_connection` tasks alive and the remote peer unable to detect drop)

Doc-comment names the two known coverage gaps per J-104 Lock #2 honesty discipline:

- (a) `shutdown_tx.send(true)` is clean async shutdown, not process-crashed-mid-write — production crash modes are out of test scope
- (b) Fresh `127.0.0.1:0` rebind, not restart-in-place TIME_WAIT — production restarts may surface different timing

**File 2 — `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs` (new).**

Scenario 2 implementation. **PASSES on current code.** Uses `current_thread` runtime (NOT `multi_thread` as in Scenario 1) so `tracing-test` 0.2.6's thread-local subscriber captures the F-5 `federation_push_skipped_origin` G2 trace fired inside B's and C's spawned `handle_connection` tasks — tracing-test-internal limitation discovered during implementation. Uses the post-`federate(A,B)` tip (state.federation_add(A→B)) as the common ancestor for the message chain because state.federation_add(A→C), emitted by the second federation, is inherently invisible to B (per-Space federation visibility property).

**File 3 — `xgen-node/src/tests/phase9_drop_and_recover.rs` (new).**

Scenario 3 implementation. **FAILS on current code at cycle 0 boundary with the cascade documented in §5.** Uses `multi_thread` runtime because its `logs_contain("federation_push_dropped_unregistered")` assertion targets A's `apply_federation_push` trace fired from the test's main task. Test code is correct as written; the failure is the production gap, not a test bug — honest framing per Rule 1.

**Verification contract.** **Scenario 3 must PASS at milestone close as the activating integration-level regression lock for the persistence fix, sibling-shape to Scenario 1's role for D-075 + D-076 v1.1 at J-101.** Just as Scenario 1's `#[ignore]` lift at J-101 marked the activating regression lock for the bidirectional + topological-sort milestones' substantive fixes, Scenario 3's transition from FAIL → PASS at this milestone close marks the activating regression lock for whatever fix shape design phase locks at Q2.

**File 4 — `xgen-node/src/tests/mod.rs` (modified).**

Two new `pub mod` declarations:

```
pub mod phase9_three_node_anti_transitivity;
pub mod phase9_drop_and_recover;
```

**Verification target at milestone close.** Full workspace test produces **580 PASS + 0 FAIL** (= 578 baseline + Scenario 2 + Scenario 3) at sentinel-tree close. Honest framing per Rule 1: test code as written is correct; the current 578 PASS + 1 FAIL = 579 attempted reflects Scenario 3's failure as production gap evidence, not as suppressed signal.

**Verification rigour requirements** to be locked at runbook authoring per topo-sort precedent: candidate is 5 isolated runs (cargo clean between each) + 3 workspace runs = 8 green runs minimum before Scenario 3's transition from FAIL → PASS is considered verified. Pre-existing flakes (precedence env-var race; reconnect_with_existing_tip_small_delta_delivered) carried forward as known signatures; their firing during verification does not invalidate the green run if Scenario 3's pass is independent of their state.

---

## §9 — Failure-mode catalogue entry candidate

Pre-shape for the entry to be added to `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` at milestone close. Sibling-shape to M15 added at J-101 for topo-sort. Captures context at audit-phase before it dissipates per topo-sort precedent.

**Candidate row (to be locked at milestone close, not now).**

| Field | Content |
|---|---|
| **ID** | M16 (next available after M15) |
| **Detection scenario** | `xgen-node/src/tests/phase9_drop_and_recover.rs::two_node_drop_and_recover_two_cycles` — cycle 0 → cycle 1 boundary; test PASSES at milestone close per §8 verification contract |
| **Failure-mode classification** | Silent live-store-vs-disk divergence at the `xgen-core` ↔ `xgen-node` layer boundary, with destructive replay consequence via layered B3 silent-discard at `graph.add_event` |
| **Primary surface** | Drain hooks in `xgen-core/src/node/runtime.rs` re-dispatch released events via `let _ = self.dispatch_event(...)`; persist call site at `xgen-node/src/app.rs::process_inbound` only sees explicitly-passed events |
| **Secondary layered surface** | `ingest_event` silently discards `GraphError::UnknownPrevEvent` from `graph.add_event` at `runtime.rs:181` — turns "missing predecessor on replay" into "event invisible to tip computation" |
| **Severity at integration level** | HIGH — structural; would not surface in production until first Node restart-after-buffered-drain; cascading consequence at federation tip exchange |
| **Severity at unit level** | LOW — affected paths have unit-test coverage of drain mechanics in isolation; the gap is in the cross-layer composition, not in any single component |
| **Originating architectural move** | Phase 7.5 Commit 3.5 drain-hook lift from `process_inbound` into `dispatch_event` Step 7 (J-103 retrospective) |
| **Closure mechanism** | Fix shape locked at design phase per §7 Q2; activating regression lock at integration level: Scenario 3 |
| **Sibling-shape pattern** | Layered-B3 surface (second project-wide instance after topo-sort Commit 2a's two layered surfaces at J-101) |

Pointing-only-at-survey-findings is the failure mode (context dissipates between audit-phase and milestone-close); pre-shaping at audit-phase with explicit "to be locked at milestone close, not now" framing is the discipline.

---

## §10 — Cross-references

**JOURNAL entries.**

- **J-103** — Phase 7.5 (Federation Cold-Start Bootstrap) implementation retrospective; records the Commit 3.5 drain-hook lift that originated the architectural surface this audit characterises.
- **J-104** — This gap surfacing; Path 1 Joe-lock (open sub-amendment milestone); sentinel-tree decision; eighth recurrence of "honest longer work over fast shortcuts" within Federation Event Propagation milestone scope.

**DECISIONS entries.**

- **D-022** — Crate-split decision (xgen-common, xgen-core conceptual split — predecessor to D-044).
- **D-044** — xgen-core library-crate creation; the architectural fact that puts the wall at the boundary this gap straddles.
- **D-065** — Honest behaviour over polite behaviour; the discipline that surfaces the gap rather than papering it over.
- **D-067** — No-drift-surface code-organisation principle; relevant to §3's layered-surface framing and to Q2(c) as fifth family member candidate.
- **D-069** — Joe-locked design phase + canonical-document discipline; the audit-vs-design boundary §7 honours.
- **D-070** — Transport-layer no-drift-surface (two-events-per-relationship-of-equal-importance principle); sibling family member relevant to Q2(c) family-completion.
- **D-071** — Subsystem audits precede dependent milestones; the principle this milestone instantiates.
- **D-074** — Milestone-close commit's changed-files list MUST include JOURNAL.md (atomicity); informs §11 paragraph 3.
- **D-075** — Event-model no-drift-surface (vantage-aware applier principle); sibling family member relevant to Q2(c) family-completion.
- **D-076 v1.1** — Wire-format no-drift-surface (two-property: byte-identical-determinism + causal-DAG-respecting order); sibling family member relevant to Q2(c) family-completion.

**Sibling audit precedents.**

- `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` (COMPLETED v1.0) — milestone-internal audit shape precedent.
- `tasks/FEDERATION_TOPOSORT_AUDIT.md` (COMPLETED v1.1) — milestone-internal audit shape precedent + §7 discipline-notes precedent.
- `docs/xgen_propagation_reliability.md` (J-081, ARCHIVED) — project-wide subsystem audit at top of D-071 stack; distinct artifact lifecycle, recorded for completeness.

**Code surface line anchors (as-of-2026-05-23).**

- `xgen-core/src/node/runtime.rs:181` — `ingest_event` silent-discard at `graph.add_event` (§3).
- `xgen-core/src/node/runtime.rs:~670-682` — `drain_pending_uniform` re-dispatch loop (§2 site 1).
- `xgen-core/src/node/runtime.rs:~745-760` — `drain_pending_by_identity` cross-Space fan-out (§2 site 2).
- `xgen-core/src/node/runtime.rs:~795-810` — `drain_pending_by_federation_relationship` cascade entry point (§2 site 3).
- `xgen-core/src/dag/graph.rs:105-110` — `graph.add_event` UnknownPrevEvent return path.
- `xgen-node/src/app.rs:1503-1505` — `persist_event` call site inside `DispatchOutcome::Accepted` match arm.

---

## §11 — Discipline notes

**1. Layered-B3 recurrence count.** Second project-wide instance, sibling to topo-sort Commit 2a (J-101). The pattern is "primary fix surfaces a secondary silent-error encoding of the same invariant at a sibling layer." Two instances is not yet a durable pattern; three would be. Future audits should look for the shape but not pre-assume its presence. Recording the count here so a future audit author finds the running tally and either extends it or reframes the pattern if the third instance has a different shape.

**2. "Honest longer work over fast shortcuts" recurrence.** Eighth within Federation Event Propagation milestone scope (J-104 already counted the count at journal close). Phase 7.5 (originating); bidirectional `federation_nodes` (second); topo-sort design close (third); runbook landing (fourth); design-phase re-walk Step 2 (fifth); Step 3 (sixth); topo-sort implementation close (seventh); drain-without-persist gap (eighth — this milestone). Pattern continues to hold: each delay closes a real gap before it ships. Federation Event Propagation milestone closure dependency chain extended by one more node (this sub-amendment); Phase 9 stays PLAY; M6 (new) + XGID Retrofit Pass 1 stay PENDING.

**3. D-074 atomicity discipline.** The audit doc itself ships as single-file at its own commit per J-104's Path 1 single-file precedent. Sibling-shape to J-098/J-099/J-100 single-file fix-up atoms and to J-104's own single-file JOURNAL-only commit; the audit-open commit is not a milestone-close commit and does not trigger D-074's milestone-close-includes-JOURNAL requirement. D-074 application count stays at 7 (last instance J-101); the sub-amendment milestone's own milestone-close commit will be the next D-074 application.

**4. Audit-vs-design boundary preserved.** This doc names questions, does not lock answers — D-069 discipline. Topo-sort precedent is the canonical example of why pre-locking at audit forecloses options that should be design-phase choices: that audit named "determinism + causality" as two surfaces, and design chose the layered framing via D-076 v1.1. The audit didn't pre-lock the layered approach; design did. Pre-locking at audit would have foreclosed Path A (determinism-only) which was genuinely on the table at design open. The same discipline applies here: §7's four questions are explicitly open, and the three-option framing on Q1 plus the three-shape framing on Q2 are designed to preserve design-phase optionality.

**5. Three-option framing on Q1.** Two-option framing ("fix vs. defer") prematurely excludes "load-bearing-by-design and document the invariant" as a third legitimate possibility. `graph.add_event`'s silent-discard may turn out to be defensive behaviour with valid invariants we don't want to break — e.g. tolerating partial-state during legitimate replay-mid-stream scenarios. Until design enumerates call sites and invariants, we don't know. The audit doc surfaces all three options; design decides. This is the same discipline that produced topo-sort's amended D-076 v1.1: surface all candidate shapes, let design weigh, lock at design close.

**6. File-location decision recorded.** Audit doc lives at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` per actual on-disk precedent of bidirectional + topo-sort audits, both of which are milestone-internal sub-amendment audits inside Federation Event Propagation milestone scope. The `docs/`-resident audit (J-081 `xgen_propagation_reliability.md`) belongs to a distinct artifact lifecycle: it was the motivating audit for the Federation Event Propagation milestone itself, archived to `docs/` at milestone-close to preserve the project-wide subsystem-audit framing. This audit doc is a milestone-internal sub-amendment whose lifecycle ends at design close, not at milestone close. Status discipline downstream: ACTIVE → COMPLETED at design close, sibling to bidirectional + topo-sort.
