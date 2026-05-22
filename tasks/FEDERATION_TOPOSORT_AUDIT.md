# Federation Topological-Sort Wire-Order Non-Determinism Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-22  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This document is the canonical record of a protocol-level audit finding surfaced during Phase 9 Scenario 1's post-D-075 verification (JOURNAL.md J-096 Finding 2): **`topological_sort_events` in `xgen-node/src/fanout.rs` produces non-deterministic wire-order for DAG-root events because it has no tie-break rule for ready siblings AND its input feed comes from `HashMap.values()` iteration. Two `xgen-node` processes with identical Space state produce different federation-delta wire orderings ~50% of runs. When `state.room_create` wins the race against `state.space_create`, B rejects it ("space not found"), cascading rejections through the bootstrap chain and timing Scenario 1 out at the per-event budget.**

It is a subsystem audit per D-071 — produced before any design-phase work begins, so the design walkthrough that follows has a code-grounded shared baseline to walk against. Sibling-in-shape to `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` (Status: COMPLETED v1.0), which preceded the bidirectional `federation_nodes` design phase. The pattern is: dependent work surfaces a load-bearing protocol gap → audit documents the gap with code-grounded evidence and candidate fix shapes → design phase walks the candidates and locks a fix → implementation runbook ships it → dependent work resumes.

This document does not lock a fix. Locking is the design phase's job. This document's job is to surface the gap precisely, ground it in actual code, and frame the option space cleanly enough that the design walkthrough runs efficiently.

### 1.1 Position in the milestone

This audit sits between the bidirectional `federation_nodes` milestone closure (2026-05-21, HEAD `827303d`) and the topological-sort design phase (next-active). Phase 9 Scenario 1 was lifted at Commit 3 of the bidirectional milestone and re-stood-down at Commit 4 on this finding (`#[ignore]`-annotated with inline doc comment naming this audit's forward-reference). Phase 9 Commit 3b (Scenarios 2 + 3, plus the compound scenarios) is paused inside the Federation Event Propagation milestone scope until the topological-sort fix lands.

Pass 1 implementation (`tasks/XGID_RETROFIT_PASS_1_IMPL.md`, Status: ACTIVE v2.0) remains downstream of Federation Event Propagation milestone closure. Whether the fix affects Pass 1's scope depends on which fix shape is locked — see §7.5.

### 1.2 Reading order

1. JOURNAL.md J-096 Finding 2 — Clair's diagnostic write-up with verbatim evidence (timestamp comparison, outcome tally, 105-`dispatch_event`-count parity).
2. This document, §3 → §4 → §6 → §7 → §7.5 (the framing-and-options chain).
3. `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` — structural template; matches this audit's section depth and discipline-note framing.
4. `xgen-node/src/fanout.rs` (lines 193-220 + 311-333) — the two sites named in §3.
5. `xgen-core/src/node/runtime.rs` (lines 859-912) — the canonical sibling sort that the federation path's sort should mirror.

---

## 2. Provenance — how the gap surfaced

The bidirectional `federation_nodes` milestone closed 2026-05-21 in a four-commit sequence (J-096). Commit 3 (`f051039`) lifted Phase 9 Scenario 1's `#[ignore]` annotation and added `#[serial_test::serial]`. The first task at Commit 4's session open was to verify the Scenario 1 pass holds across multiple isolated runs before shipping the milestone-close commit.

### 2.1 The four-check sanity sequence

The pre-Commit-4 verification ruled out four upstream causes before settling on the genuine finding:

1. **4/4 pre-reboot runs failed** at exactly ~122s (the 120s per-event budget plus overhead). Hypothesis 1 (single-run transient): ruled out by repeat.
2. **Machine reboot** to clear OS-level state drift (file handles, port reservations, async runtime residue). Post-reboot result was mixed across multiple isolated runs at the original 120s budget — about 50% pass at 5.34s / 50% fail at ~122s, with no middle ground. Hypothesis 2 (OS-level drift) partially helped but did not eliminate the flake.
3. **Clean cargo rebuild** (`cargo clean && cargo test -p xgen-node --lib tests::phase9_two_node_smoke::tests::two_node_federation_push_smoke_100_messages -- --include-ignored`). Hypothesis 3 (stale build artefacts): ruled out — the flake persisted at the same ~50% rate.
4. **Time-pressure hypothesis test.** Three per-event/per-condition budgets in `phase9_two_node_smoke.rs:261` (10s → 60s), `:343` (120s → 300s) and `phase9_harness.rs:380, :386` (10s → 60s each) were temporarily relaxed. 5 isolated runs sampled. Outcome: 4 fail / 1 pass with the failures hitting **exactly** the new 300s ceiling (301.96 / 301.97 / 301.94 / 301.98 — all within ~0.05s of the budget). The relaxation did not reveal slow-but-eventual arrival; it simply delayed the panic by the extra time. Hypothesis 4 (time pressure): ruled out. **Events truly never drain in failing runs.**

### 2.2 The instrumented run that pinpointed `topological_sort_events`

Diagnostic instrumentation was then added on the xgen-node side (visible to `traced_test`, which filters out `xgen_core::*` targets and would have hidden a `tracing::debug!` placed inside `dispatch_event` itself). Two `tracing::warn!` lines bracketed the `dispatch_event` call in `process_inbound` — one before naming `event_type`, `event_id`, `space_id`, `sender`, `peer_node_id_for_f3`, `prev_events`, and the current `federation_nodes` snapshot for the target Space; one after naming the returned `DispatchOutcome` variant.

Three runs (1 pass + 2 fail) gave the conclusive data (verbatim from J-096):

| Run | Result | dispatch_event calls | Accepted | HeldPending | Rejected |
|---|---|---|---|---|---|
| 1 | PASS (5.34s) | 105 | 102 | 3 | 0 |
| 2 | FAIL (121.94s) | 105 | 2 | 101 | 2 |
| 3 | FAIL (121.98s) | 105 | 2 | 101 | 2 |

**Decisive evidence: 105 dispatch calls in all three runs.** Every event reaches `dispatch_event` on B with identical event-type distribution (1 `state.space_create` + 1 `state.room_create` + 1 `state.federation_add` + 1 `membership.invite` + 1 `membership.join` + 100 `message.text`). The bidirectional fix's drain hook is exercised correctly. The divergence is in the **wire order** of events arriving at B.

### 2.3 The wire-order divergence

In the PASS run, the bootstrap arrival sequence on B was `state.space_create` (29.484313Z, accepted) → `state.room_create` (29.491389Z, accepted) → `membership.invite` → `membership.join` → `state.federation_add` → 100 messages. Standard happy path.

In the FAIL run, the sequence was `state.room_create` (35.246931Z, `federation_nodes=(no-space-state)`, **Rejected with "space not found"**) → `state.space_create` (35.247021Z, accepted) → `state.federation_add` (accepted) → `membership.invite` (HeldPending — its `prev_events` reference the now-missing `room_create_id`) → `membership.join` (HeldPending — refs `invite_id` which is buffered) → `message_1` (its `prev_events=[federation_add_id]` are present, but Step 11 fails: "sender is not a member of room 'xgen://hash/sha256:488a901d...'" because Alice's join never applied — **Rejected**) → `message_2..100` (each refs the previous message, the chain is buffered behind the rejected `message_1` — **HeldPending**). Tally checks out exactly: 2 Accepted (`space_create` + `federation_add`) + 2 Rejected (`room_create` + `message_1`) + 101 HeldPending (`invite` + `join` + 99 chained messages) = 105.

### 2.4 Root-cause trace to source

`build_room_create_event` in `xgen-core/src/space/state.rs:797-820` constructs the event with `vec![]` for `prev_events` (DAG-root semantic). `state.space_create` also has empty `prev_events`. Both are roots.

In `compute_federation_delta_for_space` at `xgen-node/src/fanout.rs:311-333`, the local store contents are collected via `store.values().cloned().collect()` (non-deterministic HashMap iteration per Rust's `RandomState`) and then passed to `topological_sort_events` at `xgen-node/src/fanout.rs:193-220`. That function uses a single-pass scan over the input vector, removing events whose predecessors are already emitted; for tied root events (both ready from iteration 1), the algorithm preserves **input order**, which is therefore non-deterministic.

Sibling function `topological_sort` in `xgen-core/src/node/runtime.rs:859-912` (used for in-process ordering, separate code path) uses Kahn's algorithm with explicit `queue_vec.sort()` for stable tie-breaking. The xgen-node-side delta function does not.

### 2.5 Bidirectional fix verified not implicated

All evidence agrees: identical `dispatch_event` call count across pass/fail; the unit-level mirror test `apply_federation_add_two_vantages_mirror` from bidirectional Commit 2 remains green; the divergence is upstream of the dispatcher in delta serialisation. The bidirectional milestone closure (Commit 4, `827303d`) is correct and the fix stands. This audit's finding is a separate pre-existing bug surfaced during verification, NOT implicated by D-075.

---

## 3. The mechanism, code-verified

Two sites in `xgen-node/src/fanout.rs` compound into one symptom. Neither is wrong in isolation.

### 3.1 Site 1 — `compute_federation_delta_for_space:321`

```rust
let store = match rt.stores.get(space_id) {
    Some(s) => s,
    None => return Vec::new(),
};
let all: Vec<Event> = store.values().cloned().collect();   // line ~321
drop(rt);
let sorted = topological_sort_events(all);
```

`EventStore` is `HashMap<String, Event>`. Rust's default hasher is randomized per HashMap instance. Two Nodes with identical Space state produce different input orderings to the sort. **Non-determinism enters here.**

### 3.2 Site 2 — `topological_sort_events:193`

```rust
pub fn topological_sort_events(mut events: Vec<Event>) -> Vec<Event> {
    let mut emitted: HashSet<String> = HashSet::new();
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut changed = true;
    while !events.is_empty() && changed {
        changed = false;
        let mut i = 0;
        while i < events.len() {
            let ready = events[i].prev_events.iter().all(|p| {
                emitted.contains(p) || !events.iter().any(|e| e.event_id.as_deref() == Some(p))
            });
            if ready {
                let ev = events.remove(i);
                if let Some(id) = &ev.event_id { emitted.insert(id.clone()); }
                out.push(ev);
                changed = true;
            } else { i += 1; }
        }
    }
    out.extend(events);
    out
}
```

Single-pass scan: emit events whose predecessors are already emitted. **No tie-break.** Ready siblings (including DAG roots with empty `prev_events`) emit in input-vector order. The function's silent contract is "input is already meaningfully ordered" — but its caller feeds it `HashMap.values()`. **Non-determinism laundered through to wire order.**

### 3.3 Combined outcome

`state.space_create` and `state.room_create` both have empty `prev_events`. Both ready at iteration 1. Relative order = HashMap iteration order = random ~50% per run. See §2.2 evidence table and §2.3 wire-order divergence for the resulting cascade.

### 3.4 What B does (not implicated)

105 `dispatch_event` calls in pass AND fail runs. Bidirectional fix's six unit tests (incl. `apply_federation_add_two_vantages_mirror`) green. **Divergence is entirely upstream of the dispatcher.**

### 3.5 The sibling that does it right

`xgen-core/src/node/runtime.rs::topological_sort` (lines 859-912) — Kahn's algorithm with explicit `queue_vec.sort()` and `next.sort()` at every level. Two topo-sort implementations in the codebase: one canonical, one not. Federation delta path uses the non-canonical one. **D-067 drift surface in flight** — §6 Q2 walks scope.

---

## 4. The relationship to the design as locked

`tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.3.1 R4: "sorted-by-`space_id` cross-Space ordering for determinism." R4 is correct at the cross-Space layer.

**R4 was silent on within-Space ordering.** The dimension was not on Phase 3's design surface — assumed handled by the topo-sort primitive. The primitive doesn't honour that assumption for tied events.

### 4.1 Why the locked design missed it

1. **R4's framing was cross-Space-only.** Within-Space was assumed-handled, not deliberated.
2. **The two topo-sort implementations were not cross-checked.** `xgen-core/src/node/runtime.rs:859-912` already had stable tie-break when Phase 3 began. `xgen-node/src/fanout.rs:193-220` didn't. Asymmetry not surfaced as a design question.
3. **Test surface gap.** No test before Phase 9 Scenario 1 Commit 3 (`f051039`, 2026-05-21) exercised full bootstrap delivery with two `NodeRuntime` instances + non-trivial state. Bug was there the whole time.

### 4.2 The principle revealed

> **Wire-order determinism is a normative protocol property and must be locked explicitly, not assumed to emerge from local primitives.**

Anticipated at the cross-Space level by R4. Not anticipated at the within-Space level. Same "considered one dimension, missed another" shape as the bidirectional audit's §4.

Audit analogue: **All sender-side code paths that produce wire-visible ordering must be canonical, not merely correct.** "Correct" topo-sort respects causality. "Canonical" topo-sort additionally produces byte-identical output for byte-identical input sets.

### 4.3 Catalogue extension

Phase 9 catalogue (J-091, 14 entries) has no row for delta-serialisation-order non-determinism between Nodes. Design phase adds: **"Sender-side wire-order non-determinism between Nodes."**

---

## 5. Scope

### 5.1 In scope for the design phase that follows

- **Resolve Q1, Q2, Q3** (§6.1–§6.3). Q3 is load-bearing and must be answered before any shape is evaluated.
- **Pick one of Shapes A / B / C / D** with Joe-lock (§7).
- **Confirm or revise the Pass 1 runbook scope** based on the locked shape's structural footprint (§7.5).
- **Coordinate the fix with all existing Federation Event Propagation milestone locks** — Phase 3 a-i symmetry rule + R4, Phase 4 origin gating, Phase 7 A1 + B1 + B3, Phase 7.5 P7.5-A through P7.5-D, D-075 bidirectional vantage rule. The fix must not regress any of these.
- **Confirm the catalogue row name** per §4.3 and place it in `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`.
- **Decide whether Q2-known-leaky sites ride alongside the primitive fix or get scheduled separately** (Q2 middle reading clarification — §6.2).

### 5.2 Out of scope for this audit and the design phase

- **Full audit of every `HashMap` iteration in the codebase.** Q2-wide is a design-phase option, not an audit pre-decision. The audit lists known-leaky sites for the Q2 framing; an exhaustive codebase sweep is its own work item if Q2-wide lands.
- **Changing `compute_federation_delta_for_space`'s API contract.** The function's signature and call-site protocol are outside the fix's blast radius — only its internal behaviour at the HashMap-feed boundary is in scope (Shape A optional sibling, or Shape D's container change at the EventStore layer).
- **Promoting canonical wire ordering to spec-normative** (i.e., adding a Ch3 / Ch4 / Appendix entry that locks "two senders with identical state MUST produce byte-identical deltas") without design-phase deliberation. Q3's resolution may produce such a spec change — that's downstream of the lock, not in this audit's scope.
- **Test-surface restructuring beyond the fix's regression coverage requirements.** Phase 9 Scenario 1 is the activating regression witness; expansion beyond it is Phase 9 Commit 3b's scope.
- **Stress-test follow-on coverage.** The four deferred compound scenarios in `tasks/FEDERATION_STRESS_FOLLOWON.md` remain blocked on the clock-injection seam and are unaffected by this audit.

### 5.3 Non-scope decisions explicitly recorded

| Item | Why deferred | Where it lands |
|---|---|---|
| Codebase-wide `HashMap.values()` audit | Q2 framing offers narrow/middle/wide as design options; exhaustive sweep is conditional on Q2-wide selection | Separate work item if Q2-wide locks |
| Canonical-wire-ordering spec entry | Q3 lock may justify; needs deliberation | Design phase decides; if locked, follow-on doc-pass to Ch3 / Ch4 / Appendix |
| Per-call-site fix for `collect_sync_history` + `apply_fanout` history-push | Q2 middle reading defers per-site question | Q2 narrow: deferred indefinitely; Q2 middle: scheduled as own design question; Q2 wide: rides with primitive fix |
| Migration of any persisted federation deltas | None exist (no production deployments) | Out-of-scope |

---

## 6. Audit-phase questions for the design walkthrough

Three questions. Design phase must answer all three. Q3 is load-bearing.

### 6.1 Q1 — Tie-break choice

When two events are ready in the same topo-sort iteration, what canonical rule decides their order?

- **Q1.a — `event_id` sort.** Lexicographic, arbitrary, cheap. Pass-1 coupling (event_id → EventXgid).
- **Q1.b — Timestamp sort.** Semantic but millisecond collisions need secondary tie-break; wall-clock non-canonical across senders.
- **Q1.c — Canonical-event-bytes sort.** Strongest guarantee, most expensive, cross-crate dependency.
- **Q1.d — Container change (BTreeMap / IndexMap at EventStore).** Resolves at the data layer.

Downstream of Q3.

### 6.2 Q2 — D-067 audit scope

Other code paths that consult `HashMap.values()` and leak to wire-visible output:

- `xgen-node/src/fanout.rs::collect_sync_history` — non-deterministic; own comment block flags it. Client-facing pagination; **not** acceptable if any federation path consumes the same data structure.
- `xgen-node/src/fanout.rs::apply_fanout` history-push — same bug shape. Local-client only today.
- `xgen-core/src/node/runtime.rs::all_events` — uses canonical sibling sort. Reference site.
- `xgen-node/src/federation_session.rs::stream_federation_delta` — R4 covers cross-Space; within-Space comes from §3.1.

Three Q2 readings:

- **Q2 narrow** — fix `topological_sort_events` + (optionally) §3.1 feed. Other sites deferred indefinitely; known-leaky bug class recorded but not closed.
- **Q2 middle** — fix the primitive so it's canonical regardless of input. Per-caller surfacing becomes its own design question: known-leaky sites (`collect_sync_history`, `apply_fanout`) are NOT ignored — the design phase decides whether they ride alongside the primitive fix or get scheduled separately. The distinction: "fix the primitive's contract once" vs. "fix each site individually."
- **Q2 wide** — fix every site at every wire-visible boundary. Largest blast radius; closes the D-067 drift surface wholesale.

Design phase picks.

### 6.3 Q3 — Wire-format normative question (LOAD-BEARING)

Canonical wire ordering required, or per-receiver-deterministic tie-break sufficient?

- **Q3.i — Per-receiver determinism sufficient.** Receivers run deterministic sort locally; wire ordering is transient. Any shape closes Scenario 1.
- **Q3.ii — Canonical wire ordering required.** Two senders with identical state MUST produce byte-identical deltas. Justifications: cross-Node verifiability; audit-trail consistency; future MLS group-state ordering.

Q3.i admits Shape A as sufficient. Q3.ii requires Shape A + sibling §3.1 fix, OR Shape D, OR hybrid.

**Joe-lock-threshold question. Both readings legitimate.**

### 6.4 Subordinate questions

- **Q4** — Replay determinism on Node restart (Shape D only).
- **Q5** — Sort cost profile (Shape C primarily).
- **Q6** — Forward compat with future EventTypes that emit empty `prev_events` (all shapes).
- **Q7** — Confirm catalogue row name per §4.3.

---

## 7. Candidate fix shapes

Four shapes, increasing structural depth. Design phase picks one with Joe-lock.

### 7.1 Shape A — `event_id` sort at topo primitive

**Mechanism.** Add `events.sort_by(|a, b| a.event_id.cmp(&b.event_id));` at the top of each outer-loop iteration in `topological_sort_events`. Optional sibling: sort input at §3.1 before passing (required under Q3.ii).

**Cost.** One line. Sort cost O(n log n) per iteration — negligible at typical delta sizes.

**Benefit.** Smallest footprint. Pure-function property restored at the primitive.

**Pass-1 coupling.** event_id → EventXgid under Pass 1. Two v-options:

- **v1** — `&str` sort at v1, retype to `EventXgid` under Pass 3 with code-comment block flagging the future retype. Pass-1-neutral.
- **v2** — typed `EventXgid` sort from outset. Pass-1-coupled: requires Pass 1's `EventXgid` to land before this shape ships, OR ships with `Xgid::new(event.event_id.clone())` wrap pattern matching bidirectional Commit 2's `SpaceLocalMetadata.introducer_node_id` precedent.

Design phase picks v-option against Pass 1 coordination posture.

**Q3.** Admissible under both readings (Q3.ii requires sibling §3.1 fix).

### 7.2 Shape B — Timestamp sort

**Mechanism.** Same as A but sort key is `event.timestamp`.

**Cost.** Millisecond collisions realistic → secondary tie-break needed (collapses to A or C). Wall-clock non-canonical across senders.

**Benefit.** Semantically meaningful in logs.

**Q3.** Admissible under Q3.i only. **Disqualified under Q3.ii.**

### 7.3 Shape C — Canonical-event-bytes sort

**Mechanism.** Same as A but sort key is canonical wire bytes via `xgen-core::wire::canonical`.

**Cost.** Serialises every event per comparison (cacheable). Cross-crate dependency `xgen-node` → `xgen-core::wire::canonical`.

**Structural note.** Canonical bytes include `event_id` by construction (per Appendix C primitive schema). For distinct-`event_id` events, canonical-bytes lexicographic order ≈ `event_id` lexicographic order. Shape C's additional benefit over Shape A is concentrated in the case of duplicate-`event_id` events (which the protocol does not currently emit but could in principle under future EventType extensions), and in the explicit-canonicality property at the primitive contract layer (a documentation/auditability benefit beyond the wire-output equivalence for current EventTypes).

**Benefit.** Strongest canonical guarantee. Maximal D-067 closure at the primitive.

**Q3.** Admissible under both readings.

### 7.4 Shape D — Container change at EventStore

**D.1 — `BTreeMap<String, Event>` keyed by event_id.** Canonically sorted iteration. HashMap feed at §3.1 becomes deterministic at source. `topological_sort_events` needs no change.

**D.2 — `IndexMap` insertion-ordered.** Deterministic per process; insertion order non-canonical across Nodes. **Disqualified under Q3.ii.**

**Cost (D.1).** Every `EventStore.{get,values,contains,insert}` call site verified for BTreeMap semantics. API-compatible; perf shifts O(1) → O(log n). Replay-on-disk: BTreeMap reconstructs canonical order regardless of disk order.

**Benefit (D.1).** Closes bug class at the data layer. Q2-wide naturally satisfied. Future consumers automatically canonical.

**Pass-1 coupling.** Indirect — EventStore keys are event_id strings.

**Q3.** D.1 admissible under both readings. D.2 disqualified under Q3.ii.

### 7.5 Summary table

| Shape | Tie-break source | Code change | Q3.i | Q3.ii | D-067 reach | Pass-1 coupling |
|---|---|---|---|---|---|---|
| A | event_id (lex) | 1 line + optional sibling | ✅ alone | ✅ with sibling | Partial — primitive only | v1 Pass-1-neutral; v2 coupled |
| B | timestamp | 1 line + secondary | ✅ with secondary | ❌ | Partial | None |
| C | canonical bytes | 1 line + cross-crate dep | ✅ | ✅ strongest | Partial maximal | Implicit |
| D.1 | event_id at data layer | EventStore container swap | ✅ | ✅ | Wide | Indirect |
| D.2 | insertion order | EventStore container swap | ✅ within process | ❌ | Wide within process | Indirect |

Cost-benefit genuinely distributed. No shape dominates. Design phase locks against the Q3 reading.

---

## 8. Phase-9 + milestone implications

### 8.1 Phase 9 Scenario 1 regression witness

Scenario 1 at `xgen-node/src/tests/phase9_two_node_smoke.rs` is `#[ignore]`-annotated with an inline doc comment naming this audit's forward-reference. When the fix lands, the `#[ignore]` lifts. The scenario becomes the activating regression lock for the topological-sort bug — any future change that re-introduces the non-determinism will fail Scenario 1.

The scenario stays on disk as authored; no modifications needed when the fix ships. The fix's implementation runbook should include "remove #[ignore] from `phase9_two_node_smoke.rs::two_node_federation_push_smoke_100_messages` and verify the test passes" as a DoD item — same shape as the bidirectional fix's Commit 3.

### 8.2 Phase 9 Commit 3b (Scenarios 2 + 3)

Both remain paused inside the Federation Event Propagation milestone scope. Scenario 2 (anti-transitivity) and Scenario 3 (drop-and-recover) both presuppose working bidirectional federation bootstrap — which presupposes deterministic delta ordering. Both unblock when the fix ships, regardless of which shape is locked.

### 8.3 Compound scenarios C2 / C3 / C5 / C7 / C9 / C10

All six compound scenarios from the Phase 9 survey presuppose working bidirectional federation. All six are paused behind the same gate. The four deferred compounds (C1 / C4 / C6 / C8) in `tasks/FEDERATION_STRESS_FOLLOWON.md` remain blocked on the clock-injection seam, independently of this fix.

### 8.4 M6 (new) Node admin write path

Remains 🟡 PENDING behind Federation Event Propagation milestone closure. The topological-sort fix is now part of the milestone closure dependency chain. M6 (new) is unaffected in scope by this audit; the dependency chain just gained one more node.

### 8.5 Pass 1 implementation

Status: ACTIVE v2.0 with all four sub-questions Joe-locked. The Pass 1 runbook is unmodified by Shapes B, C, D.1, or Shape A v1. Shape A v2 introduces a Pass-1 coordination posture (either Pass 1 ships first, or Shape A ships with the `Xgid::new(...)` wrap pattern). The decision is downstream of the design-phase Joe-lock; until then, Pass 1 stays at v2.0.

---

## 9. Discipline notes

This audit is a worked instance of three project-management principles already in DECISIONS.md:

- **D-069 (canonical-document rule).** This document is the canonical record of the topological-sort audit finding. Future references to the finding cite this document.
- **D-071 (subsystem audits precede dependent milestones).** The audit runs before the design phase that fixes the gap. The design phase has this document as its Pass 1 input, mirroring how the bidirectional design phase had `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` as its input.
- **D-074 (milestone-close commits include JOURNAL).** When the design phase closes, its closure commit will include a JOURNAL entry. When the implementation runbook closes, that closure commit will also include a JOURNAL entry.

### 9.1 Fourth recurrence of the audit→design→impl→close pattern

This is the project's fourth instance of the audit-precedes-dependent-work pattern: J-081 Propagation Reliability Audit → Federation Event Propagation design phase (first); Phase 7.5 design task file → Phase 7.5 implementation runbook (second); Bidirectional `federation_nodes` audit → design → implementation arc just closed (third); this audit → topological-sort design → implementation → Phase 9 Scenario 1 re-resurrection (fourth, in flight).

The pattern is now repeating-pattern rather than one-off. Each substantial bug class that survives the unit-test surface and surfaces at the deployment-level integration test becomes its own phase. D-071's natural cadence is visible: integration-level testing exercises code paths that surface protocol gaps invisible to unit testing.

### 9.2 "Honest longer work over fast shortcut" — fourth instance

The fastest path at J-096 Finding 2's surface would have been to ship Commit 4 of the bidirectional milestone with a `#[serial_test::serial]` annotation as a workaround and a hand-wave acknowledgement of the flake ("known intermittent issue, follow up later"). The chosen path — full diagnostic walk, root-cause traced to source, verbatim evidence captured in J-096, separate phase opened per D-071, Scenario 1 re-stood-down as the regression witness for the new fix — is longer than the workaround AND shorter than rediscovering the bug from a production deployment.

The diagnostic specifically found the bug **upstream** of where the bidirectional fix's symptom appeared to land. The honest framing — "the bidirectional fix is verified correct; this is a separate pre-existing bug that the new test surface revealed" — required acknowledging that the first instinct ("the bidirectional fix is incomplete") was wrong. Same shape as the Phase 9 Scenario 1 → bidirectional finding arc that the just-closed milestone resolved.

### 9.3 D-067 finer-grain instance

The bidirectional audit recorded D-067 as the "single source of truth, no drift surface" principle. This audit's Q2 framing is a worked instance of D-067 at a finer grain: two implementations of the same primitive (topological sort) exist in the codebase, one canonical, one not. Two sites are now confirmed leaky on the wire-output-determinism dimension: `topological_sort_events` (the primary find) and `collect_sync_history` (named in §6.2's Q2 audit list, flagged by its own comment block). More may exist; Q2-wide reading would discover them. The design phase decides whether to close the drift surface at the primitive (Q2 middle), at every consumer (Q2 wide), or only at the activating site (Q2 narrow).

### 9.4 Catalogue extension

The Phase 9 failure-mode catalogue (J-091, 14 entries) named protocol-level deadlocks and validation asymmetries. The topological-sort wire-order non-determinism does NOT map cleanly to any single catalogue entry — it is a category of bug that the catalogue did not anticipate. Phase 9's value as a deployment-stress surface is the ability to find this kind of bug too. The catalogue should be extended in the design phase to add the row per §4.3 so the next contributor reading the catalogue finds the category.

---

## 10. Cross-references

### 10.1 Design documents

- **`docs/xgen_federation_propagation_design.md`** (Status: ACTIVE, v1.0) — the canonical Federation Event Propagation design. §6.4 Phase 7 Lock A1 + B1, §6.4.1 Phase 7.5 P7.5-A through P7.5-D, §6.4.2 Phase 8 bidirectional vantage-aware applier (D-075), §15 Implementation Complete table.
- **`tasks/FEDERATION_PROPAGATION_COMPLETION.md`** (Status: ACTIVE, v1.0) — the implementation runbook. §3.3 Phase 3 wire shape, §3.3.1 R4 (sorted-by-`space_id` cross-Space ordering — anticipated cross-Space dimension, did not surface within-Space dimension; see §4 above).
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md`** (Status: COMPLETED v1.0) — structural template for this audit. Same ten-section shape; same discipline-note framing.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md`** (Status: COMPLETED v1.0) — sibling-shape precedent for the next-active design phase. D-075 locked here.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`** (Status: COMPLETED v1.1) — sibling-shape precedent for the implementation runbook that follows the design phase.
- **`tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md`** (Status: COMPLETED v1.0) — earlier sibling-shape audit→design precedent.
- **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (Status: ACTIVE v1.0) — Phase 9 implementation task file. Scope intact; Scenario 1 re-stood-down on this finding; Commit 3b paused.

### 10.2 Code surfaces

- `xgen-node/src/fanout.rs::topological_sort_events` (lines 193-220) — Site 2 of §3, the non-canonical sort primitive.
- `xgen-node/src/fanout.rs::compute_federation_delta_for_space` (lines 311-333; HashMap feed at ~321) — Site 1 of §3.
- `xgen-node/src/fanout.rs::collect_sync_history` (lines ~225-275) — Q2 audit list, known-leaky on cross-Space iteration.
- `xgen-node/src/fanout.rs::apply_fanout` (lines ~115-200) — Q2 audit list, history-push path.
- `xgen-core/src/node/runtime.rs::topological_sort` (lines 859-912) — **canonical sibling sort precedent.** Kahn's algorithm with explicit `queue_vec.sort()` and `next.sort()` tie-breaking. The reference site for any of Shapes A/B/C.
- `xgen-core/src/node/runtime.rs::all_events` (lines ~920-930) — uses canonical sort; reference site.
- `xgen-core/src/space/state.rs::build_room_create_event` (lines 797-820) — `prev_events: vec![]` (DAG-root semantic).
- `xgen-core/src/space/state.rs::build_space_create_event` — `prev_events: vec![]` (DAG-root semantic).
- `xgen-node/src/tests/phase9_two_node_smoke.rs::two_node_federation_push_smoke_100_messages` — Phase 9 Scenario 1 regression witness (`#[ignore]`-annotated, doc-comment forward-references this audit).
- `xgen-core/src/dag/store.rs` — `EventStore` definition (`HashMap<String, Event>`); relevant for Shape D.

### 10.3 JOURNAL

- **J-096** — the originating record of the finding. Finding 2 (topological-sort discovery section) is the canonical evidence source. Quote verbatim per Rule 2.
- J-088 (Phase 7 closure) — F-3 implementation context.
- J-093 (Phase 7.5 design closure) — sibling precedent for the design-phase pattern.
- J-094 (Phase 7.5 implementation closure) — sibling precedent for the implementation-phase pattern.
- J-095 (XGID Adoption v1 implementation milestone closure) — D-074 first instance.

### 10.4 DECISIONS

- **D-067** — Single source of truth / no drift surface. The principle whose finer-grain instance Q2 walks (§6.2, §9.3).
- **D-068** — Original "drift surface" decision; precursor to D-067's framing.
- **D-069** — Canonical-document rule. This document is the canonical home for the topological-sort audit finding.
- **D-071** — Subsystem audits precede dependent milestones. The discipline this document follows (§9.1).
- **D-074** — Milestone-close commits include JOURNAL. Forward-binding for the design + implementation phases that follow.
- **D-075** — Bidirectional vantage-aware applier rule. Sibling-distinct from this audit's eventual decision; the principle the just-closed bidirectional milestone locked.

---

*End of audit document. Design phase walkthrough is next-active for Chat Claude + Joe. The walkthrough resolves Q1 + Q2 + Q3 (§6.1, §6.2, §6.3) and picks one of the four candidate fix shapes (§7) with Joe-lock. The locked design phase produces its own task file `tasks/FEDERATION_TOPOSORT_DESIGN.md` (sibling to `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md`) that captures the framework decisions and may promote a new D-NNN to DECISIONS.md. After the design phase closes, the implementation runbook `tasks/FEDERATION_TOPOSORT_IMPL.md` is authored (sibling to `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`) and handed off to Clair. After the runbook ships, Phase 9 Scenario 1's `#[ignore]` lifts again and the topological-sort phase closes.*  
