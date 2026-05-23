# Phase 7.5 Persistence Amendment — Design Phase
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-NN (Commit 1 doc-pass of implementation milestone — Status flipped ACTIVE → COMPLETED v1.1. Canonical design doc `docs/xgen_federation_propagation_design.md` gained §6.4.4 sibling subsection (sibling-in-shape to §6.4.3 topological-sort wire-order determinism — same one-Joe-lock-per-question lock pattern; same code-comment-block-at-load-bearing-site discipline; same verification-rigour 5+3=8-green-runs minimum at integration close; intro paragraph names the gap closed with file:line evidence at `xgen-core/src/node/runtime.rs:181` and `xgen-node/src/app.rs:2628`) + §15 row appended after the topological-sort row with `[J-NNN] (2026-05-NN)` placeholder syntax for freeze at Commit 4. Design phase content stays authoritative as historical record at design-at-lock-time; runbook `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` ACTIVE v1.0 is the implementation contract Clair ships against. Per D-069 canonical-document discipline + topo-sort + bidirectional design-task-file lifecycle precedent (v1.0 ACTIVE at design close; v1.1 COMPLETED at Commit 1 of implementation milestone). Previous J-105 design-close content stands authoritative — see body §3–§6 for the four Joe-locks Q1→Q4.) Previous 2026-05-23 update: 2026-05-23 (Design doc authored at design-phase session-arc close per audit §1's named milestone shape (D-071 four-phase, sibling to bidirectional + topo-sort). Eleven sections sibling-in-shape to `tasks/FEDERATION_TOPOSORT_DESIGN.md` (COMPLETED v1.0). Four Joe-locks recorded across the walkthrough: Q1 at (a).ii + (a).iii.β + candidate D-NNN flag (sort-on-replay + `ingest_event` returns `Result<(), GraphError>` + "ingest path invariant encoding" flagged for future walk); Q2 at (a) return-vector (`DispatchOutcome::Accepted { new_joiner, additional_persisted: Vec<Event> }`); Q3 at all-three drain helpers (`drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship`); Q4 at (a) in-scope (sentinel-tree ships atomic at milestone close as activating regression lock; Commit 3b-1 collapses into milestone close). No re-walk fired during Q1→Q4 walk per Lock #2 discipline. Status flips ACTIVE → COMPLETED at implementation runbook landing (sibling to bidirectional + topo-sort design doc lifecycle). Per D-065 + D-067 + D-069 + D-071 honest-behaviour-over-polite-behaviour discipline.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Purpose, milestone-shape position, and walk order

**Purpose of this design doc.** Lock the four open questions Q1–Q4 raised at audit-phase close (`tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` §7) into actionable design decisions that the implementation runbook (next session-arc) translates into Clair-facing commits. Per D-069 audit-vs-design boundary discipline, audit names questions and does not lock answers; design locks answers and does not author runbooks. The implementation runbook (own session-arc at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md`) carries the locks into Clair-executable form.

**Milestone-shape position.** Second phase of the four-phase D-071 milestone shape audit §1 named:

1. ✅ **Audit phase** — `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` (COMPLETED v1.1 at design close).
2. 🟢 **Design phase** — this document (ACTIVE v1.0; flips COMPLETED at implementation runbook landing per topo-sort + bidirectional precedent).
3. 🟡 **Implementation phase** — runbook authoring (Chat Claude + Joe, own session-arc) + Clair impl (own session-arc) per topo-sort precedent.
4. 🟡 **Milestone close** — atomic per D-074. Sentinel tree (audit §8) ships at milestone close per Q4(a) lock; Scenario 3 transitions FAIL → PASS.

**Lock #2 walk order.** Q1 → Q2 → Q3 → Q4, with re-walk discipline: if Q2 walkthrough surfaced evidence pulling Q1 toward a different lock, walk would have paused and re-walked Q1 before resuming Q2 — sibling-shape to topo-sort design-phase re-walk at J-099/J-100. **No re-walk fired during this walk.** Q1's Result-returning `ingest_event` became a clean constraint on Q2 rather than a destabilising factor (Q2(a) return-vector composes cleanly with Q1's type chain; Q2(c) would have composed similarly; Q2(b) re-lift would have decoupled from Q1 without contradicting it). Recording explicitly per D-065 honest-framing: the re-walk *did not* fire is itself worth recording so future contributors don't assume it must.

**Deliverable-shape decision at session open.** Option A (separate design doc at this filename) chosen over Option B (§-extension on audit doc). Audit §1 already named the topo-sort precedent (separate file); choosing Option B would have reversed the shipped structural decision. Confirming the decision-shape lock here for canonical-record visibility.

**Sibling-in-shape precedents.** `tasks/FEDERATION_TOPOSORT_DESIGN.md` (COMPLETED v1.0) — primary precedent, eleven-section shape inherited. `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` (COMPLETED v1.0) — secondary precedent, audit-vs-design boundary discipline.

**Distinct artifact lifecycle from J-081.** `docs/xgen_propagation_reliability.md` (J-081, ARCHIVED) is the project-wide subsystem audit. This design doc and its audit sibling are milestone-internal sub-amendment artefacts whose lifecycle ends at implementation runbook landing (design doc) and milestone close (audit doc per audit §1 paragraph 5).

---

## §2 — Audit cross-reference + sentinel-tree contract

**Audit summary.** Drain hooks in `xgen-core/src/node/runtime.rs` re-dispatch released events through `let _ = self.dispatch_event(ev, origin, None);`, silently discarding the `Accepted` outcome. The persist site at `xgen-node/src/app.rs::process_inbound` only sees the explicitly-passed event. Drained events become visible in B's in-memory `EventStore` + `SpaceState` but never hit disk. On Node restart, `replay_spaces_from_dir` only sees the persisted subset; `graph.add_event` at `runtime.rs:181` silently discards `UnknownPrevEvent` for events whose predecessors weren't persisted, turning the gap from "merely lossy" into "destructive on restart." Cascade trace at audit §5. See audit doc for full surface characterisation.

**Secondary layered surface** (audit §3). `let _ = graph.add_event(&event, store);` at `runtime.rs:181` is the second silent-error encoding. The primary fix at the drain-hook layer surfaces this secondary surface at the DAG-graph ingest layer beneath the applier layer. Sibling-shape to topo-sort Commit 2a's two layered B3 surfaces (J-101), though the layered-B3 pattern has only two project-wide instances and is not yet a durable pattern (audit §11.1).

**Sentinel-tree contract** (audit §8). Four files retained uncommitted at J-104 session close:

1. `xgen-node/src/tests/phase9_harness.rs` (modified) — `SavedNodeState` carrier, `shutdown_keep_data` method, `spawn_in_process_node_with_state` constructor, `connection_tasks` field + `abort_connection_tasks` helper.
2. `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs` (new) — Scenario 2, **PASSES on current code**.
3. `xgen-node/src/tests/phase9_drop_and_recover.rs` (new) — Scenario 3, **FAILS on current code at cycle 0 boundary** with the cascade documented in audit §5. Production gap evidence per Rule 1.
4. `xgen-node/src/tests/mod.rs` (modified) — two new `pub mod` declarations.

**Activating regression-lock contract.** Scenario 3's transition FAIL → PASS at milestone close is the activating integration-level regression lock for the persistence fix, sibling-shape to Scenario 1's role for D-075 + D-076 v1.1 at J-101.

**Verification target at milestone close.** Full workspace test produces 580 PASS + 0 FAIL (= 578 baseline + Scenario 2 + Scenario 3). Verification rigour candidate (to be locked at runbook authoring): 5 isolated runs (cargo clean between each) + 3 workspace runs = 8 green runs minimum, sibling to topo-sort runbook §5.3 precedent. Pre-existing flakes carried forward as known signatures.

---

## §3 — Q1 walkthrough → lock

**Q1 (audit §7).** Silent-discard pattern at `runtime.rs:181` (`graph.add_event` UnknownPrevEvent silent-discard). Three options:

- **(a) Fix in this milestone.** Sibling-layer fix inside scope.
- **(b) Fix in follow-on D-071 arc.** Adjacent surface gets its own audit → design → impl pass.
- **(c) Load-bearing-by-design.** Document the invariant; don't change behaviour.

### Q1 walkthrough findings

**Finding 1 — `ingest_event` has exactly two production callers in xgen-node:**

- `dispatch_event` line 668 (runtime's internal post-validation handler).
- `replay_spaces_from_dir` at `app.rs:2628` (startup replay loop).

**Finding 2 — `replay_spaces_from_dir` runs through `ingest_event`, which calls `graph.add_event` with the silent-discard.** This is exactly the path that produces the cascade in audit §5. The silent-discard at `runtime.rs:181` is what makes `fed_add(A→B)` invisible to tip computation on cycle 1's replay — its predecessor (`join.event_id`) isn't on disk, so `graph.add_event` returns `UnknownPrevEvent`, the error is discarded, fed_add isn't registered as a tip, B sends the wrong Hello tip.

**Finding 3 — `dispatch_event` itself also calls `ingest_event` (line 668), but events reaching that call site have already passed validation.** The only path through `dispatch_event` that reaches `ingest_event` is the `ValidationOutcome::Validated` arm; events with missing predecessors take the `ValidationOutcome::HeldPending` arm before they ever reach `ingest_event`.

**Consequence.** The silent-discard at `runtime.rs:181` is doing two different things at two different call sites:

| Call site | What `UnknownPrevEvent` means | Is silent-discard correct? |
|---|---|---|
| `dispatch_event` → `ingest_event` | Should not happen — validate_event already checked predecessors are in store. If it fires, it's a state-machine bug. | Silent-discard hides bugs but is functionally inert (case shouldn't fire). |
| `replay_spaces_from_dir` → `ingest_event` | Happens whenever the on-disk store has predecessor-after-child ordering OR (the bug we're fixing) in-memory state had drained-but-not-persisted events that the child depends on. | Silent-discard turns a destructive divergence into a silent one. |

This is **not** a single load-bearing-by-design invariant. The silent-discard serves two different masters with two different correctness profiles. **Option (c) falls.**

**Option (b) falls.** The silent-discard is structurally inseparable from the primary fix at this milestone. Q2's lock either ensures the silent-discard never fires in practice (by guaranteeing predecessor-before-child persistence ordering) or replaces it with something safer. Either way, Q1 lands inside this milestone.

**Option (a) stands.** Sub-shape question: three candidates (a).i / (a).ii / (a).iii enumerated below.

### Q1 sub-shape walkthrough

- **(a).i** — Propagate `GraphError` from `ingest_event`, caller decides.
- **(a).ii** — Topologically-sort the on-disk store before the ingest loop in `replay_spaces_from_dir`.
- **(a).iii** — Replace silent-discard with explicit error handling at `runtime.rs:181`. Two sub-variants: (a).iii.α `tracing::error!` (log-level); (a).iii.β `ingest_event` returns `Result<(), GraphError>` (type-level).

**Future-sustainability question.** "Is the project's whole philosophy 'structurally incapable of enshittification' — that's a future-proofing thesis at the protocol layer. Implementing it at the code-organisation layer means choosing the rung where the most likely future drift surfaces get caught by the type system." Three drift surfaces (a).iii.α does NOT catch:

1. Future caller bypasses validate_event (M6 admin write path, M8 federation-depth migration tool) — silent-discard fires, log line unwatched. Exactly J-104's gap shape.
2. Disk format change (Ch4 §4.12.3 SQLite per spec, currently JSON drift surface flagged at J-089 close) — sort-on-replay may not apply.
3. Future protocol revision introduces event family with async predecessor validation — destructive case re-opens.

Log-level vigilance is **recurring**; type-level enforcement is **one-time**. (a).iii.β is more sustainable than (a).iii.α but is not absolutely future-proof — there is at least one more rung above it (ValidatedEvent wrapper at the type system level; sealed traits + visitor pattern; formal verification). Naming the rungs explicitly per D-065 honest framing so the project knows where it landed:

| Rung | Protects against | Doesn't protect against |
|---|---|---|
| (a).iii.α log-level | Audit trail | Anything if no one reads logs |
| **(a).iii.β Result type (LOCKED)** | Compiler-forced handling | Wrong handling, new bypass functions, semantic drift |
| ValidatedEvent wrapper | Compiler-forced *correct* path | Sealed-trait shape, future invariant changes |
| Sealed traits + visitor | New-caller shape constraint | Formal-spec-level drift |
| Formal verification | Machine-checked invariants | Whatever's outside the model |

**Project-philosophy alignment.** The project's pattern is to discover discipline failures and convert them to durable rules (D-067/D-070/D-075/D-076 no-drift-surface family; Rule 0 session-open; D-071 audit-precedes-design). The discipline question raised by "is this future-proof?" is itself a candidate for that pattern — name it, walk it in its own session-arc, lock the rung the project wants to live at. (a).iii.β is the right *immediate* answer because it's the smallest correct-today fix that meaningfully raises the floor. The "right rung" question is bigger than Q1 and deserves its own walk — recorded as candidate D-NNN flag per §8 below.

### Q1 lock

**[JOE-LOCK: locked 2026-05-23]**

**Q1 → (a).ii + (a).iii.β + candidate D-NNN flag.**

1. **(a).ii** — `replay_spaces_from_dir` calls `topological_sort(events)` before the ingest loop. Defensive layer at the replay path.
2. **(a).iii.β** — `ingest_event` signature changes from `pub fn ingest_event(&mut self, event: Event)` to `pub fn ingest_event(&mut self, event: Event) -> Result<(), GraphError>`. Compiler enforces every caller handles `UnknownPrevEvent`. Production sites: `dispatch_event` uses `.expect("validate_event guarantees predecessors")`; `replay_spaces_from_dir` uses sort-first so the error is unreachable by construction. Test sites: mechanical `.expect()` additions (~10 sites in runtime.rs's own test module per audit §10 line anchors).
3. **Candidate D-NNN flag** — "ingest path invariant encoding" as project-level open question. Pointer only at this design close; goes through its own audit → design → impl arc in a future session-arc if Joe locks it as worth pursuing. The candidate names the rung-above-(a).iii.β question (ValidatedEvent wrapper, sealed traits, etc.) without pre-committing the project to a specific shape. Sibling to no-drift-surface family but at code-organisation layer. Recorded in JOURNAL J-105 + ROADMAP cross-cutting-principles section per D-069 audit-vs-design boundary (DECISIONS entries are for *locked* principles, not flagged ones).

---

## §4 — Q2 walkthrough → lock

**Q2 (audit §7).** Three candidate fix shapes for the primary drain-without-persist gap. Drain hooks at `xgen-core/src/node/runtime.rs` lines ~670 / ~745 / ~795 all call `let _ = self.dispatch_event(ev, origin, None);` and discard the Accepted outcome. The persist site at `xgen-node/src/app.rs::process_inbound:1503` only sees the explicitly-passed event.

### Q2 walkthrough

**Q1 as constraint.** Q1 locked (a).iii.β — `ingest_event` returns `Result`. That means `dispatch_event` now has a place where ingest can fail loudly. Two Q2 options interact with this:

- **Q2(a) return-vector** — pairs cleanly with Q1. `dispatch_event`'s drain loop collects Accepted events into a vector AND surfaces ingest failures via the Result chain. Type-level enforcement at both layers.
- **Q2(c) callback** — pairs cleanly with Q1. Callback signature can return `Result<(), PersistError>`.
- **Q2(b) re-lift** — pairs less cleanly. Re-lifting drain hooks back to xgen-node means `dispatch_event` no longer needs to know about persistence at all; Q1's Result-returning ingest is fine independently, but the layered architecture loses the integration point.

**Q2(a) return-vector — pros:**

- Layer separation preserved cleanly. xgen-core stays I/O-free.
- Caller (process_inbound) has explicit access to all persisted events. No callback-injection cognitive load.
- Pairs naturally with Q1's Result chain.
- Signature change is mechanical: `Accepted { new_joiner, additional_persisted: Vec<Event> }`.

**Q2(a) return-vector — cons:**

- Every existing `dispatch_event` caller has to destructure the new field, even when they don't care about drained events.
- Drain order becomes caller-visible — if a caller forgets to iterate the vector in order before persisting, ordering bugs surface at the persist layer.
- The vector can grow large under cascade-drain scenarios (Phase 7.5 + Phase 6 chained). Memory bounded by pending-buffer size (~10–100 events realistic), not unbounded.

**Q2(b) re-lift — pros:**

- Restores the pre-Phase-7.5-Commit-3.5 architecture's natural persist reachability.
- No new abstractions, no new types.
- Smallest signature surface change in the dispatcher itself.

**Q2(b) re-lift — cons:**

- Re-opens the B3-shape gap J-103 closed unless other Phase 7.5+ changes have implicitly fixed it. Verification surface is the predecessor-chain deadlock + step-11 sender-membership rejection that motivated the lift. Non-trivial.
- Three drain hooks live in xgen-core today. Re-lifting means moving ~150 lines from `xgen-core/src/node/runtime.rs` into `xgen-node/src/app.rs`. The hooks have access to xgen-core internals (`self.pending`, `self.stores`, `self.identity_registry`) that aren't exposed across the crate boundary today — re-lift requires either widening xgen-core's public surface OR restructuring the helpers to take their dependencies as parameters.
- Reverses a design decision the project explicitly committed to (Phase 7.5 Commit 3.5 J-103) and would need an honest retrospective on *why* the lift was wrong (it wasn't — it solved a real B3 gap; the persistence surface was unobserved, not unsound).

**Q2(c) callback — pros:**

- Preserves both layer separation AND signature stability (callback is optional).
- xgen-core knows "there's a persist hook" without knowing what it does. Clean abstraction.
- Fifth no-drift-surface family member candidate alongside D-067/D-070/D-075/D-076. Sibling-shape: code-organisation layer's principled solution to the cross-crate-boundary I/O concern.
- Future M6 admin write path inherits the callback structure cleanly.

**Q2(c) callback — cons:**

- Adds a layer of indirection at every `dispatch_event` call site. Test fixtures gain `None` parameters; production gains real callbacks.
- Injects an I/O concern (via abstraction) into the pure-protocol crate. The wall at the crate boundary gets a window cut into it.
- New trait definition + new error type, more vocabulary to maintain.
- If the callback signature evolves later (async, batch, transactional), it touches every caller.

### Q2 ranking

**Q2(a) return-vector is the right answer for this milestone.** Five reasons:

1. **Q1 + Q2(a) compose cleanly.** Q1's Result-returning ingest gives type-level enforcement of "ingest didn't lose anything"; Q2(a)'s additional_persisted vector gives type-level visibility of "what was persisted." Both layers visible in the type signature.
2. **No new abstractions.** Just data flowing through return types. Easier to reason about than a callback's indirection.
3. **Caller-explicit persist responsibility.** `process_inbound` literally iterates the vector and calls `persist_event` — the discipline is visible at the call site, not hidden behind a callback signature.
4. **Doesn't reverse J-103.** Phase 7.5 Commit 3.5's lift stays in place; the persistence surface gets closed through return-types instead of architecture-rollback.
5. **Doesn't promote to D-NNN.** Q2(c)'s callback shape would be a fifth no-drift-surface family member — durable discipline. But the persistence concern is *narrow* to the drain hooks; promoting it to a family-level principle treats one milestone's surface as if it were a project-wide invariant. (a) keeps the fix proportional to the gap.

**Q2(b) re-lift** is the second-rank option, gated on design-phase verification that Phase 7.5+ has implicitly closed the B3 gap that motivated the lift. That verification is its own audit-phase surface; not something to commit to without doing the work first. Not chosen.

**Q2(c) callback** is third-rank. Architecturally elegant but over-built for this gap. If a future surface needs the callback shape (M6 admin path is the most likely candidate), the project promotes Q2(a)'s pattern to Q2(c) at that point — same shape as how D-076 v1.1 emerged from D-076 v1 when the second invariant surfaced. Don't pre-build the abstraction.

### Q2 lock

**[JOE-LOCK: locked 2026-05-23]**

**Q2 → (a) return-vector.**

`dispatch_event` returns `DispatchOutcome::Accepted { new_joiner, additional_persisted: Vec<Event> }`. Drain helpers return their drained Accepted events to `dispatch_event`, which aggregates them into the vector. `process_inbound` iterates and persists each.

Sibling-in-shape to Q1's Result-returning ingest: both fixes raise visibility at the type level rather than relying on log-level vigilance or architecture rollback.

---

## §5 — Q3 walkthrough → lock

**Q3 (audit §7).** Drain-hook scope. In-scope: just (iii), or all three? Three drain helpers:

- **(i)** `drain_pending_uniform` — Phase 4 / F-4a, predecessor resolution
- **(ii)** `drain_pending_by_identity` — Phase 6 / F-10, unknown-signer resolution
- **(iii)** `drain_pending_by_federation_relationship` — Phase 7.5 / F-3, surfaced the cascade

### Q3 walkthrough

**Q2 as constraint.** Q2(a) return-vector is the answer. All three drain helpers share the same architectural shape — `let _ = self.dispatch_event(ev, origin, None);` discards the Accepted outcome. The Q2(a) fix applies identically to all three: each helper changes from returning `()` (or implicit unit) to returning `Vec<Event>` of the drained Accepted events, and `dispatch_event` aggregates them into `additional_persisted`.

**Scope-(iii)-only argument.** Only (iii) surfaced the cascade. Scenario 3 exercises only (iii). Applying the fix to (i) and (ii) widens the diff beyond what the verification contract covers.

**Scope-all-three argument:**

1. Same code shape across all three helpers. Q2(a)'s change is mechanical and identical at each site. Risk of touching all three is no greater than touching one.
2. Latent gap at (i) and (ii). Right now (i) and (ii) silently produce the same drain-without-persist consequence — they just haven't been exercised by a test scenario that catches it yet. (i) fires whenever a predecessor arrives out-of-order; (ii) fires whenever an Identity record arrives for events buffered awaiting it. Both are real production paths; both have the same destructive-on-restart shape Scenario 3 surfaced for (iii).
3. **Same family of gap, same atomic close.** Sibling-shape to topo-sort Commit 2a's two layered B3 surfaces (J-101): primary fix at one site surfaced two encodings of the same invariant; both closed atomically per D-067. Same pattern here: three encodings of the same drain-without-persist pattern; closing only the one that happened to surface in testing would be the same "tests-finally-lock-what-they-claimed" anti-pattern J-101 documented.
4. **Q1's type-level enforcement compounds.** Q1's Result-returning `ingest_event` forces every drain-helper site to handle the error explicitly. Doing this at (iii) only leaves (i) and (ii) as call sites that *should* never hit `UnknownPrevEvent` (their drained events have predecessor-known by construction at drain time) — but the type system doesn't know that. Either we `.expect()` at (i) and (ii) (which is fine but means we *touched* them anyway), OR we leave them with `let _ =` ignoring the Result (which silently reintroduces a Q1-shape gap one milestone later). Touching all three is structurally cleaner.

**Verification-coverage answer.** Scenario 3 exercises (iii) at integration level. (i) and (ii) get unit-level coverage via existing test sites in runtime.rs's own test module (the `drain_pending_by_federation_relationship_drains_buffered_events` test at runtime.rs:~1310 is a sibling-shape precedent; (i) and (ii) get analogous tests asserting the returned vector's contents).

### Q3 ranking

**Scope-all-three is correct.** Three reasons:

1. **The gap pattern is the same at all three sites.** Closing one and leaving two open is shipping a known-incomplete fix. Sibling-shape to topo-sort Commit 2a where the primary fix surfaced two more encodings of the invariant; all three closed atomically.
2. **Q1's type system forces interaction with (i) and (ii) anyway.** Either we handle their Result honestly (which means we touched them) or we silently discard (which means Q1's enforcement is half-applied).
3. **Diff cost is small.** Same mechanical change at three sites instead of one. ~3x the diff but ~1x the conceptual surface.

### Q3 lock

**[JOE-LOCK: locked 2026-05-23]**

**Q3 → all three drain helpers** (`drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship`) get the Q2(a) return-vector treatment. Same mechanical change at each site. Unit tests at sites (i) and (ii) extended to cover the returned vector; integration coverage at Scenario 3 covers (iii) at the deployment level.

---

## §6 — Q4 walkthrough → lock

**Q4 (audit §7).** Sentinel-tree scope: in-scope for this milestone, or already-shipping-via-Commit-3b-1's-eventual-commit?

### Q4 walkthrough

**Q4(a) in-scope argument:**

1. **Verification contract requires it.** Audit §8 names Scenario 3 as the activating integration-level regression lock for the persistence fix, sibling-shape to Scenario 1's role for D-075 + D-076 v1.1 at J-101. The fix doesn't get verified-correct without Scenario 3 transitioning FAIL → PASS. That transition happens at this milestone's close, not at Commit 3b-1's close.
2. **Refinement risk is real.** Design phase locked Q1 + Q2 + Q3 today; implementation runbook will surface edge cases that may require harness changes. Treating the sentinel tree as frozen at J-104 forecloses these adjustments.
3. **Atomicity per D-074.** Milestone-close commit's changed-files list captures everything that closes the milestone. Sentinel-tree files are part of the close — they're the verification artefact.
4. **Sibling precedent.** J-101's topo-sort milestone close was eight files atomic, including the test file (`phase9_two_node_smoke.rs`) whose `#[ignore]` lift was the activating regression lock. Same shape here.

**Q4(b) already-shipping argument:**

1. **Phase 9 Commit 3b-1 was the original commit shape.** J-104 paused Commit 3b-1 mid-flight; the sentinel tree is Commit 3b-1's work-in-progress. Letting Commit 3b-1 land it preserves Commit 3b-1's identity as a Phase 9 commit rather than absorbing it into the sub-amendment milestone.
2. **Milestone separation.** Persistence-amendment milestone is sub-amendment scope; Phase 9 Commit 3b-1 is Phase 9 scope. Mixing them muddles milestone boundaries.
3. **File-count shape.** Q4(a) makes the milestone-close commit larger; Q4(b) keeps it focused on substantive fix files.

### Q4 ranking

**Q4(a) is correct.** Three reasons:

1. **The audit's own verification contract names Scenario 3 as the regression lock.** Audit §8: "Scenario 3 must PASS at milestone close as the activating integration-level regression lock for the persistence fix." That's a Joe-locked verification contract at audit time. Q4(b) would defer the regression-lock-passing to Commit 3b-1, which means the persistence-amendment milestone closes *without* the integration-level verification that its fix actually works at the deployment level.
2. **Refinement-risk argument is decisive.** Design phase didn't fully verify that the J-104 harness shape is correct for the locked fix. Implementation-runbook authoring (next session-arc) will walk the harness in detail; refinements are likely. Q4(b) would either force those refinements into Commit 3b-1 (Phase 9 scope, not sub-amendment scope) OR leave them as silent drift between J-104 snapshot and what ships. Q4(a) keeps refinements inside the milestone they belong to.
3. **Milestone-separation argument doesn't hold.** The sub-amendment milestone exists *because* Commit 3b-1 surfaced the persistence gap. Commit 3b-1's verification artefact (Scenario 3) IS the milestone's activating regression lock. Two scopes entangled by causation; closing the milestone without verifying Scenario 3 passes is closing it without honest evidence per Rule 1.

### Phase 9 Commit 3b-1 framing after milestone close

**Commit 3b-1 collapses into the sub-amendment milestone close.** The persistence-amendment milestone's milestone-close commit is large (substantive fix + sentinel tree + canonical-record updates) and Scenario 3's transition FAIL → PASS *is* what Commit 3b-1 was trying to ship. After milestone close, next-active Phase 9 work is whatever was Phase 9 Commit 3b-2+ — compounds C2/C3/C5/C7/C9/C10 (Scenarios 2 + 3 already done as part of milestone close). Phase 9's Commit 3b numbering effectively skips "3b-1"; or, more honestly, Commit 3b-1 *is* the persistence-amendment milestone close under a different milestone name. CLAUDE.md PLAY block flip at milestone close must acknowledge this explicitly per D-065 honest framing.

### Q4 lock

**[JOE-LOCK: locked 2026-05-23]**

**Q4 → (a) in-scope.** Four sentinel-tree files ship atomic at milestone close as part of the activating regression lock. Honest framing in JOURNAL J-NNN milestone-close entry that Commit 3b-1's intended work collapses into this milestone close; Phase 9 resumes at Commit 3b-2-equivalent (compounds remaining).

Sub-question recorded as §9 handoff requirement: whether sentinel-tree refinement (likely during implementation walk) counts as one commit or splits across the four-file atomic per D-074 same-commit discipline. Runbook-authoring decides the exact shape.

---

## §7 — Fix shape summary

Consolidated table of all four locks, sibling-shape to topo-sort design doc §7.5 summary table.

| Q | Lock | Reasoning ref | Code surface |
|---|---|---|---|
| **Q1** | (a).ii + (a).iii.β + candidate D-NNN flag | §3 — sustainability question forced re-rank from (a).iii.α to (a).iii.β; D-NNN flag preserves "right rung" question for future walk per D-069 audit-vs-design boundary | `xgen-core/src/node/runtime.rs:181` (ingest_event signature + body); `xgen-node/src/app.rs:2606-2630` (replay_spaces_from_dir sort-first); ~10 test sites in runtime.rs::tests module |
| **Q2** | (a) return-vector | §4 — composes cleanly with Q1; preserves layer separation; doesn't reverse J-103; doesn't promote to D-NNN family member prematurely | `xgen-core/src/node/runtime.rs:200-205` (DispatchOutcome::Accepted variant); drain-helper return types; `xgen-node/src/app.rs:1503-1520` (process_inbound iterate-and-persist loop) |
| **Q3** | All three drain helpers | §5 — same gap pattern at all three sites; Q1's type system forces interaction anyway; sibling-shape to topo-sort Commit 2a layered-B3 atomic close | `xgen-core/src/node/runtime.rs:~670-682` (drain_pending_uniform); `:~745-760` (drain_pending_by_identity); `:~795-810` (drain_pending_by_federation_relationship) |
| **Q4** | (a) in-scope | §6 — audit-locked verification contract requires Scenario 3 transition FAIL → PASS at milestone close; refinement-risk argument decisive; Commit 3b-1 collapses into milestone close | `xgen-node/src/tests/phase9_harness.rs` (modified); `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs` (new); `xgen-node/src/tests/phase9_drop_and_recover.rs` (new); `xgen-node/src/tests/mod.rs` (modified) |

**Combined surface.** ~5 files in xgen-core + ~3 files in xgen-node production + ~4 files in xgen-node tests + sentinel-tree integration = milestone-close commit at ~12 files atomic per D-074. Exact count locks at runbook authoring.

**Layered defense summary.** Q1 + Q2 + Q3 together produce three layers of protection against the drain-without-persist gap recurring:

1. **Q2 + Q3 layer (primary fix):** drained events flow through return-types from drain helpers → dispatch_event → process_inbound → persist_event. Type-level visibility at every layer.
2. **Q1 (a).ii layer (replay defense):** sort-on-replay protects against any future persistence-order drift.
3. **Q1 (a).iii.β layer (silent-discard elimination):** `ingest_event` returns Result; future callers compiler-forced to handle UnknownPrevEvent rather than silently dropping.

Each layer protects against a different class of future drift; together they raise the floor structurally per "structurally incapable of enshittification" philosophy at the code-organisation layer.

---

## §8 — Candidate D-NNN flag — "ingest path invariant encoding"

**Status at this design close.** FLAG ONLY. Not promoted to DECISIONS.md per D-069 audit-vs-design boundary discipline. Recorded for visibility in JOURNAL J-105 + ROADMAP cross-cutting-principles section.

**The candidate names this question:** what is the right rung on the type-safety ladder for the ingest path? Q1's (a).iii.β raises the floor to Result-returning ingest. The next-rung-up question — should ingest_event take a `ValidatedEvent` wrapper that can only be constructed via `validate_event` or an explicit audit-trail-bearing escape hatch? — was raised during Q1 walkthrough but is bigger than Q1's milestone scope.

**Sibling-shape to no-drift-surface family** (D-067 code-organisation + D-070 transport + D-075 event-model + D-076 wire-format), but at the code-organisation layer rather than at the protocol-shape layers. The family's pattern is "principled solution to a drift surface that would otherwise require ongoing vigilance." The candidate names a sibling at the "ingest path type-safety" surface.

**Future walk shape (when Joe locks the candidate as worth pursuing).** Sibling-shape to D-076's v1 → v1.1 progression: surface a discipline failure or a structural concern; walk it through audit → design → impl arc; lock the principle. The candidate would surface a candidate audit-doc at `tasks/INGEST_INVARIANT_ENCODING_AUDIT.md` or similar; design phase enumerates the rungs above (a).iii.β explicitly; implementation lands the chosen rung.

**Why flagged, not locked.** Three reasons:

1. **D-069 audit-vs-design boundary.** DECISIONS entries are for *locked* principles. Promoting at design close without a proper audit-design-impl walk would be the same mistake Q1(c) "load-bearing-by-design" would have made — committing to a shape before walking the alternatives properly.
2. **Scope proportionality.** The persistence-amendment milestone is fixing a specific drain-without-persist gap. Locking a project-wide ingest-path invariant principle as a side-effect of Q1 would be over-scoping the milestone.
3. **Future-walk preserves optionality.** Rungs above (a).iii.β include ValidatedEvent wrappers, sealed traits, formal verification. The right rung depends on project-philosophy weighting that a proper walk would surface. Pre-locking forecloses that.

**Recorded for the future walker.** §3 Q1 walkthrough's rung-by-rung table is the starting point. The walker should expect the question "which rung does the project want to live at?" to surface project-philosophy concerns about future drift surfaces (M6 admin write path, M8 federation-depth migration, MLS operationalisation, AI-assisted contributors). Sibling-shape to how Rule 0's origin story (J-099) traced a discipline failure back to project-philosophy concerns about session-open structural defences.

---

## §9 — Implementation runbook handoff requirements

Authored at design close per topo-sort precedent (J-097 design close → J-098 runbook landing). The runbook (own session-arc at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md`) must contain:

### §9.1 — Commit sequence

Candidate sequence (runbook-authoring locks exact shape):

1. **Commit 1 — doc-pass.** Audit doc Status ACTIVE → COMPLETED v1.1; this design doc Status ACTIVE → COMPLETED v1.0; canonical design doc (`docs/xgen_federation_propagation_design.md`) §15 row added (Implementation Complete for persistence amendment); ROADMAP.md updated. No code changes; test count unchanged.
2. **Commit 2 — Q1 + Q2 type changes.** `ingest_event` signature change; `DispatchOutcome::Accepted` variant change; three drain-helper return type changes; `topological_sort` call in `replay_spaces_from_dir`; mechanical `.expect()` additions at ~10 test sites; new unit tests for drain helpers (a) and (b) covering returned-vector contents (sibling to existing `drain_pending_by_federation_relationship_drains_buffered_events` test).
3. **Commit 3 — sentinel-tree refinement + Scenario 3 PASS verification.** Sentinel-tree four files refined as implementation surfaces edge cases; Scenario 3 transitions FAIL → PASS; verification rigour 5 isolated + 3 workspace = 8 green runs minimum.
4. **Commit 4 — milestone close per D-074.** Atomic commit: JOURNAL J-NNN milestone-close entry + CLAUDE.md PLAY block flip + ROADMAP.md version bump + audit doc Status → COMPLETED (if not already in Commit 1) + this design doc Status → COMPLETED (if not already in Commit 1) + failure-mode catalogue M16 row added to `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` per audit §9 candidate.

**Open at runbook authoring:** whether Commit 1 doc-pass collapses into Commit 4 milestone-close (file count concern) OR ships separate (atomicity-per-D-074 concern). Sibling-shape to topo-sort Commit 1 doc-pass precedent.

### §9.2 — Joe-lock checkpoints for Clair

Sibling-shape to topo-sort runbook §2.3:

1. **Post-Commit-1**, if doc-pass surfaces drift in canonical design doc.
2. **Pre-Commit-2 unit-test list proposal** — Clair proposes the new tests for drain helpers (a) and (b) before writing them; Joe locks.
3. **Post-Commit-2 / pre-Commit-3 primitive shape locked** — workspace tests green, type changes landed, no compile-driven surface ambiguities outstanding before lifting Scenario 3's FAIL state through the fix's verification.
4. **Pre-Commit-3 sentinel-tree refinement scope** — if implementation surfaces required harness changes, Clair surfaces them before refinement lands; Joe locks the refinement set.

### §9.3 — Verification rigour requirements

Candidate (runbook authoring confirms): 5 isolated runs (cargo clean between each) + 3 workspace runs = 8 green runs minimum before Scenario 3's transition FAIL → PASS is considered verified. Pre-existing flakes (precedence env-var race; reconnect_with_existing_tip_small_delta_delivered) carried forward as known signatures; their firing during verification does not invalidate the green run if Scenario 3's pass is independent of their state.

Sibling-shape to topo-sort runbook §5.3 verification rigour.

### §9.4 — Sentinel-tree integration shape

Q4(a) lock: sentinel-tree four files ship atomic at milestone close per D-074. Open at runbook authoring: whether refinement (likely during implementation walk) counts as one commit (folded into Commit 3) or splits across the four-file atomic. My read: one commit, runbook-authoring confirms exact shape.

### §9.5 — Milestone-close file count per D-074

Candidate: ~12 files atomic at Commit 4 (Q1 + Q2 + Q3 code surfaces + Q4 sentinel-tree + canonical-record updates). Exact count locks at runbook authoring. Sibling-shape to topo-sort J-101 eight-file atomic precedent.

### §9.6 — J-NNN placeholder freeze sites

Identified at design close (runbook authoring confirms; freeze happens at Commit 4 commit-authoring time):

- This design doc Status: COMPLETED line (when flipped at runbook landing, ships with J-NNN ref to runbook-landing entry).
- JOURNAL J-NNN milestone-close entry itself.
- CLAUDE.md PLAY block flip (current "Phase 9 Commit 3b ←── HERE" → "persistence-amendment milestone CLOSED at J-NNN; Phase 9 resumes at Commit 3b-2-equivalent").
- ROADMAP.md Past entry (persistence-amendment milestone closure paragraph).
- Failure-mode catalogue M16 row in `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` (per audit §9 candidate).

### §9.7 — Candidate D-NNN handling at milestone close

Recorded in JOURNAL J-NNN milestone-close entry per §8 framing. NOT promoted to DECISIONS.md at milestone close. ROADMAP.md cross-cutting-principles section gains entry pointing at JOURNAL J-105 (this design close's entry) + future-walk requirement. Per D-069 audit-vs-design boundary.

### §9.8 — Verification of B3-shape audit answer

At milestone close, audit the milestone for B3-shape gap recurrence (sibling-shape to topo-sort J-101 milestone-close audit answer + Phase 7.5 J-103 originating B3-shape pattern). Candidate answer: no gap expected (the layered surface at runtime.rs:181 is already addressed by Q1 (a).iii.β; the primary surface at drain hooks is closed by Q2 + Q3; no third sibling-layer encoding identified at design close). Honest verification at runbook close per D-065.

### §9.9 — "Honest longer work over fast shortcuts" recurrence count

This milestone is the eighth recurrence within Federation Event Propagation milestone scope (counted at J-104 close). Milestone-close JOURNAL entry continues the count; future recurrences within the milestone scope continue the count. Sibling family member to D-074 atomicity discipline + D-071 audit-precedes-design + D-069 audit-vs-design boundary.

---

## §10 — Cross-references

### JOURNAL entries

- **J-103** — Phase 7.5 (Federation Cold-Start Bootstrap) implementation retrospective; records the Commit 3.5 drain-hook lift that originated the architectural surface this milestone characterises.
- **J-104** — Surfacing entry; Path 1 Joe-lock (open sub-amendment milestone); sentinel-tree decision; eighth recurrence of "honest longer work over fast shortcuts" within Federation Event Propagation milestone scope.
- **J-105 (placeholder)** — This design-close entry. Records Q1–Q4 walkthrough + four-lock outcomes + candidate D-NNN flag + no-re-walk-fired honest framing.

### DECISIONS entries

- **D-022** — Crate-split decision (xgen-common, xgen-core conceptual split).
- **D-044** — xgen-core library-crate creation; the architectural fact that puts the wall at the boundary this milestone's primary gap straddles.
- **D-065** — Honest behaviour over polite behaviour; surfaces gaps rather than papering over them; informs §3's no-re-walk-fired note, §6's Commit-3b-1-collapse honest framing, §8's candidate-flag-not-promotion framing.
- **D-067** — No-drift-surface code-organisation principle; relevant to §5 layered-B3 atomic-close framing and to §8 candidate D-NNN family-member sibling-shape.
- **D-069** — Joe-locked design phase + canonical-document discipline; the boundary §8 honours by flagging not promoting.
- **D-070** — Transport-layer no-drift-surface; sibling family member relevant to §8 family-completion framing.
- **D-071** — Subsystem audits precede dependent milestones; the principle this milestone instantiates.
- **D-074** — Milestone-close commit's changed-files list MUST include JOURNAL.md (atomicity); informs §9.5 milestone-close file count.
- **D-075** — Event-model no-drift-surface; sibling family member relevant to §8.
- **D-076 v1.1** — Wire-format no-drift-surface (two-property: byte-identical-determinism + causal-DAG-respecting order); sibling family member; sibling-shape precedent for v1→v1.1 progression mentioned in §8.

### Sibling-shape design-doc precedents

- **`tasks/FEDERATION_TOPOSORT_DESIGN.md`** (COMPLETED v1.0) — primary precedent; eleven-section shape inherited; design-phase-walks-questions-and-locks-them precedent; J-097 design-close commit shape.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md`** (COMPLETED v1.0) — secondary precedent; audit-vs-design boundary discipline.

### Code-surface line anchors (as-of-2026-05-23)

- `xgen-core/src/node/runtime.rs:156` — `ingest_event` function declaration (Q1 surface).
- `xgen-core/src/node/runtime.rs:181` — `let _ = graph.add_event(&event, store);` silent-discard (Q1 secondary surface).
- `xgen-core/src/node/runtime.rs:~668` — `dispatch_event` → `ingest_event` call site (Q1 dispatch-path caller; Q2 Accepted-variant surface).
- `xgen-core/src/node/runtime.rs:~670-682` — `drain_pending_uniform` (Q2 + Q3 site i).
- `xgen-core/src/node/runtime.rs:~745-760` — `drain_pending_by_identity` (Q2 + Q3 site ii).
- `xgen-core/src/node/runtime.rs:~795-810` — `drain_pending_by_federation_relationship` (Q2 + Q3 site iii).
- `xgen-core/src/dag/graph.rs:105-110` — `graph.add_event` UnknownPrevEvent return path (Q1 underlying).
- `xgen-node/src/app.rs:1503-1505` — `persist_event` call site inside `DispatchOutcome::Accepted` match arm (Q2 caller-side persist surface).
- `xgen-node/src/app.rs:2606-2630` — `replay_spaces_from_dir` (Q1 (a).ii surface; second `ingest_event` production caller).

### Audit cross-references (sibling document)

- Audit §1 — Pattern name + milestone shape; informs §1 here.
- Audit §2 — Primary drain-hook surface; informs §4 + §5.
- Audit §3 — Layered silent-discard at runtime.rs:181; informs §3.
- Audit §5 — Cascade trace from cycle 0 → cycle 1; informs §2 sentinel-tree contract.
- Audit §7 — Open question set carried forward to this doc.
- Audit §8 — Sentinel-tree verification contract; informs §6 + §9.4.
- Audit §9 — Failure-mode catalogue M16 candidate row; informs §9.6.
- Audit §10 — Cross-reference list; this design doc's §10 inherits and extends.

---

## §11 — Discipline notes

### §11.1 — No-re-walk-fired honest framing

Lock #2 walk order Q1 → Q2 → Q3 → Q4 included re-walk discipline: if Q2 surfaced evidence pulling Q1 toward a different lock, walk would have paused and re-walked Q1 before resuming Q2 (sibling-shape to topo-sort design-phase re-walk J-099/J-100). **No re-walk fired during this walk.** Q1's Result-returning `ingest_event` became a clean constraint on Q2 rather than a destabilising factor.

Recording explicitly per D-065 honest-framing so future contributors understand that the walk's clean shape is itself worth recording — not because the walk was easy (Q1's sustainability question required revising the recommendation from (a).iii.α to (a).iii.β mid-walk), but because once Q1 locked, downstream questions composed cleanly. Sibling-shape lesson: re-walk discipline is the safety net for downstream contradictions, not for upstream refinement; (a).iii.α → (a).iii.β was upstream-refinement within Q1, not downstream-contradiction across Q1→Q2.

### §11.2 — D-074 atomicity at design close

This design doc ships as one of five files at design-close commit per topo-sort precedent (J-097 six-file atomic; this milestone five-file because no DECISIONS update at design close — candidate D-NNN flagged not promoted). Five files:

1. This design doc (NEW, ACTIVE v1.0).
2. Audit doc Status flipped ACTIVE → COMPLETED v1.1.
3. JOURNAL J-105 entry.
4. CLAUDE.md PLAY block updated + header bump.
5. ROADMAP.md version bump + tree row updates + Past entry + Present updated + cross-cutting principles candidate D-NNN flag entry.

D-074 application count at this commit: eighth instance (J-095 first; J-096 second; J-097 third; J-098 across two commits fourth; J-099 fifth; J-100 sixth; J-101 seventh; this commit eighth). Sub-amendment milestone-internal — design close is not a milestone close, but the same atomicity discipline applies to coordinated-deliverable shape per D-069 + D-074 family.

### §11.3 — Audit-vs-design boundary preserved

Audit named questions (audit §7); design locks answers (this doc §3 + §4 + §5 + §6). The candidate D-NNN flag at §8 honours the boundary explicitly: the candidate is *named*, not *locked*. Promotion to DECISIONS.md requires its own audit → design → impl walk in a future session-arc. Sibling-shape to D-076's v1 → v1.1 progression precedent.

The discipline is load-bearing: pre-locking at design close would foreclose options that should be design-phase choices in the future-walk. Topo-sort precedent at J-097 design close locked three Joe-locks (Q1 Shape A v1, Q2 middle + Q2.γ, Q3.ii canonical wire ordering required) but did NOT lock the second invariant that later surfaced as D-076 v1.1's causality property; that invariant was surfaced at Clair's Commit 3 verification (J-099) and walked properly through Step 2 + Step 3 of the re-walk. Same pattern: today's walk locks what it can lock honestly; the future-walk question gets recorded for a proper walk, not pre-decided.

### §11.4 — Layered-B3 recurrence count

Sibling-shape pattern from topo-sort Commit 2a (J-101): primary fix surfaces a secondary silent-error encoding of the same invariant at a sibling layer. This milestone is the second project-wide instance:

- **First instance** — Topo-sort Commit 2a (J-101): primary fix at `xgen-core/src/space/state.rs:797` (`build_room_create_event` constructor) surfaced two sibling-layer encodings of the DAG-root invariant (`is_dag_root_type` at `graph.rs:29` and `validate_dag_structure` at `exchange.rs:550`); Option E unification per D-067 closed both atomically.
- **Second instance** — This milestone: primary fix at the drain-hook layer (Q2 + Q3) surfaces a secondary silent-error encoding of the silent-discard invariant at a sibling layer (`graph.add_event` UnknownPrevEvent at `runtime.rs:181`); Q1 closes the secondary surface inside the same milestone.

Two instances is not yet a durable pattern; three would be. Future audits should look for the shape but not pre-assume its presence. Recording the count here so a future audit author finds the running tally and either extends it or reframes the pattern if the third instance has a different shape.

### §11.5 — "Honest longer work over fast shortcuts" recurrence count

Eighth within Federation Event Propagation milestone scope (locked at J-104 close; design close inherits the count without incrementing — recurrences are counted at milestone-events, not design-events). Phase 7.5 (originating, first); bidirectional `federation_nodes` (second); topo-sort design close J-097 (third); runbook landing J-098 (fourth); design-phase re-walk Step 2 J-099 (fifth); Step 3 J-100 (sixth); topo-sort implementation close J-101 (seventh); drain-without-persist gap surfacing J-104 (eighth — this milestone). Pattern continues to hold: each delay closes a real gap before it ships.

Federation Event Propagation milestone closure dependency chain extended by one more node (this sub-amendment); Phase 9 stays PLAY; M6 (new) + XGID Retrofit Pass 1 stay PENDING.

### §11.6 — Trilogy-internal consistency precedent

The persistence-amendment milestone's trilogy is audit + design + impl-runbook, sibling-shape to topo-sort trilogy. Trilogy-internal consistency outranks one-step-earlier-precedent consistency when they conflict (per J-098 §7.1 grounds). This design doc inherits topo-sort design doc's eleven-section shape rather than bidirectional design doc's nine-section shape, because the trilogy this doc belongs to has topo-sort as its closer precedent. Bidirectional's nine-section shape was absence-of-need at the second sibling-recurrence; topo-sort's eleven-section shape became durable at the third.

### §11.7 — Precedent-departure self-defense — none required

This design doc does not depart from topo-sort design doc shape in any substantive way. Eleven-section structure preserved; section-by-section topical mapping preserved; Joe-lock marker convention preserved; cross-reference structure preserved. Sibling-shape precedent fully applied.

The one shape-variant: candidate D-NNN flag at §8 (no equivalent in topo-sort design doc, which locked D-076 directly at design close because the second invariant surfaced cleanly at design time). Sibling-shape rationale: this milestone's parallel surface (the ingest-path invariant encoding question) is at a different scope from D-076's wire-format invariant — broader, longer-term, project-philosophy-touching — and the audit-vs-design discipline says don't lock at design close what hasn't been walked through its own audit-design-impl arc. Recording the shape-variant explicitly per D-065 honest framing so future contributors see it was a deliberate divergence within sibling-shape rather than oversight.

### §11.8 — Sustainability honesty about the locked rung

Q1's locked rung (a).iii.β is more sustainable than (a).iii.α but is not absolutely future-proof. §3's rung table names the rungs above (ValidatedEvent wrapper, sealed traits + visitor pattern, formal verification). The candidate D-NNN flag at §8 preserves the question for future walk. Recording sustainability honesty here so a future contributor reading this doc understands:

1. The project chose (a).iii.β as the *immediate* answer because it's the smallest correct-today fix that meaningfully raises the floor.
2. The "right rung" question is *bigger than Q1* and deserves its own walk.
3. The candidate D-NNN flag is the mechanism by which the question gets preserved without pre-committing.

Sibling-shape to Rule 0's origin story (J-099) where a discipline failure produced both an immediate fix (Step 2 + Step 3 re-walk closed the topo-sort framing gap) and a permanent rule (Rule 0 added to CLAUDE.md MANDATORY Behaviour rules). Same shape applied at different scope.
