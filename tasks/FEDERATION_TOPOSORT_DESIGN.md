# Federation Topological-Sort Wire-Order Determinism Design
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-22 (Design task file shipped at design-phase close. Three Joe-locks recorded: Q3 at Q3.ii (canonical wire ordering required); Q2 at Q2 middle + Q2.γ (fix primitive's contract once; forward-bind to Node-to-Client siblings); Q1 at Shape A v1 + sibling Site 1 fix (event_id lex sort at topo primitive + HashMap-feed sort at compute_federation_delta_for_space). D-076 promoted to DECISIONS.md as the protocol-design principle the locks instantiate, sibling-distinct from D-067 + D-075, pairs with D-070 as the no-drift-surface discipline family. Rejected alternatives (Shape A v2, Shape C, Shape D.1, Shape D.1 + Shape A) preserved as authoritative record. Status flips ACTIVE → COMPLETED in implementation runbook Commit 1 per the bidirectional precedent. Sibling-in-shape to `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` (COMPLETED v1.0). Per D-069 + D-071 + D-074 + D-076 discipline.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This document is the design task file for the **topological-sort wire-order determinism phase** — a small, focused milestone phase that closes a protocol-level gap surfaced during the bidirectional `federation_nodes` Commit 4 verification (JOURNAL J-096 Finding 2). It sits between the audit doc (`tasks/FEDERATION_TOPOSORT_AUDIT.md`, Status ACTIVE v1.0 — flips COMPLETED in implementation runbook Commit 1) and the implementation runbook (`tasks/FEDERATION_TOPOSORT_IMPL.md`, to be authored next session).

This is a design task file, not an implementation runbook. The runbook (Clair-facing, with code-level commit sequence) is the follow-on artefact produced in a fresh session per D-069 canonical-document rule.

### 1.1 Position in the milestone

The phase sits as a sibling to the bidirectional `federation_nodes` phase inside the Federation Event Propagation milestone — same pattern, same shape, third recurrence of the audit→design→impl→close arc within this milestone. Dependent work (Phase 9 Commit 4 verification of the bidirectional fix) surfaced a separate pre-existing protocol gap that closes in its own design + implementation phase before the dependent work resumes. Phase 9 Scenario 1 stays `#[ignore]`-annotated as the regression witness; when the fix lands, `#[ignore]` lifts a second time and the scenario becomes the activating regression lock for D-076.

Pass 1 implementation (`tasks/XGID_RETROFIT_PASS_1_IMPL.md`, Status ACTIVE v2.0) remains downstream of Federation Event Propagation milestone closure. **The Shape A v1 lock is Pass-1-neutral**: `&str` sort at v1 + code-comment block flagging Pass 3 retype to `EventXgid`. Pass 1's coverage table and sub-question locks are unaffected.

### 1.2 Reading order on session start

For Chat Claude + Joe re-entering this conversation:

1. This document, §2 (audit summary in one paragraph) — refresh the gap shape.
2. This document, §3 (Q3.ii lock + reasoning) — the wire-format-normative decision (load-bearing).
3. This document, §4 (Q2 middle + Q2.γ lock + reasoning) — the audit-scope decision.
4. This document, §5 (Q1 Shape A v1 lock + reasoning) — the fix-shape decision.
5. This document, §6 (rejected alternatives) — for principle clarity.
6. This document, §7 (D-076 framing) — the protocol-design principle.

For the implementation runbook author (Chat Claude in a future session, or Clair on direct read):

1. This document end-to-end — the design is what to build.
2. The audit doc (`tasks/FEDERATION_TOPOSORT_AUDIT.md`) — code-grounded mechanism evidence at §3 (Site 1 + Site 2 compounding), §6.1's verbatim code-comment block, §5.1's in-scope DoD list.
3. DECISIONS.md D-076 — the principle the locks instantiate.

---

## 2. Audit summary (one paragraph)

Phase 9 Scenario 1's post-D-075 verification surfaced a separate pre-existing bug at the wire-order layer: `topological_sort_events` in `xgen-node/src/fanout.rs:193` preserves input-vector order when tie-breaking ready siblings (events with all predecessors already emitted, including DAG roots with empty `prev_events`). Its caller `compute_federation_delta_for_space` at `xgen-node/src/fanout.rs:321` feeds it via `store.values().cloned().collect()` — `EventStore` is `HashMap<String, Event>` with randomized iteration per instance. Two `xgen-node` processes with identical Space state produce different federation-delta wire orderings ~50% of runs. When `state.room_create` (DAG root, empty `prev_events`) wins the race against `state.space_create` (also DAG root), B's `dispatch_event` Step 1 rejects with "space not found"; cascading rejections through the bootstrap chain produce 2 Accepted / 2 Rejected / 101 HeldPending against the passing-run baseline of 102 Accepted / 3 HeldPending. Sibling function `topological_sort` in `xgen-core/src/node/runtime.rs:859-912` (used for in-process ordering, separate code path) uses Kahn's algorithm with explicit `queue_vec.sort()` for stable tie-breaking; the xgen-node-side delta function does not. Full code-grounded evidence at `tasks/FEDERATION_TOPOSORT_AUDIT.md` §3 with file:line references.

The audit framed three structural questions (Q1, Q2, Q3) and presented four candidate fix shapes (A/B/C/D) with code-grounded cost/benefit. This design task file walks the questions and locks the choice.

---

## 3. Q3 lock — Q3.ii: canonical wire ordering required

### 3.1 The Joe-lock

**Q3 lock: Q3.ii.** Wire-order determinism is a sender-side normative property for Node-to-Node federation. Two senders with identical Space history MUST produce byte-identical federation deltas (modulo signature-bearing fields that vary by author and time). Wire ordering is part of the protocol's contract, not implementation latitude.

### 3.2 Reasoning

Four reasons, in order of weight:

**Reason 1 — D-067 wire-format analogue.** The project has consistently locked no-drift-surface properties explicitly rather than trusting them to emerge from local primitives. D-068's five-site CLI Audit closure, M5's 13-verb consolidation, D-070's two-events-with-correlation, D-075's vantage-aware applier all instantiate the same posture. A wire-format-determinism property fits the same family; locking it explicitly is in keeping with the rest of the project's discipline.

**Reason 2 — MLS coupling.** Ch3 §3.10 + D3 parallel-workstream milestone require canonical wire ordering at the application layer. Locking Q3.i would surface this as a late-stage discovery, exactly the shape D-071 audit-precedes-dependent-design was created to prevent.

**Reason 3 — Cross-Node debugging benefit is immediate, not forward-only.** "Do these two senders' deltas match byte-for-byte?" becomes a yes/no question available from today, not from MLS landing. Operators investigating federation incidents can compare byte streams across Nodes; deltas that differ are evidence of state divergence, not implementation noise.

**Reason 4 — Catalogue alignment.** The audit's §4.3 catalogue row name ("Sender-side wire-order non-determinism between Nodes") already implicitly assumes Q3.ii framing; locking Q3.ii aligns the lock with the catalogue row.

### 3.3 What Q3.i would have been (and was rejected)

Under Q3.i, each receiver would independently run a deterministic topological sort on the events it accumulates. Two receivers consuming the same events from two different senders would end up with the same local DAG (causality preserved) but might have observed the events in different orders at the wire layer. Wire ordering would be implementation latitude; the receiver's local DAG would be the protocol's contract.

This was rejected for the four reasons above. The rejection is the project's commitment to wire-format determinism as a first-class normative property. D-076 promotes that commitment to a project-wide principle so future contributors do not silently drift toward Q3.i framing when designing new federation event paths.

### 3.4 Why Q3 was walked first

Q3 was load-bearing for shape admissibility (see §6). Under Q3.i, Shape A alone would have sufficed; Shape B (timestamp sort) would have been admissible. Under Q3.ii, Shape A requires the sibling Site 1 fix; Shapes B and D.2 are disqualified. Walking Q3 first narrowed the shape-space before Q1 was evaluated, which is the disciplined order. Same posture the bidirectional design phase used (Q1 walked first because it determined which shapes were on the table at all).

---

## 4. Q2 lock — Q2 middle + Q2.γ

### 4.1 The Joe-lock

**Q2 lock: Q2 middle + Q2.γ.** Fix the primitive's contract once (`topological_sort_events` becomes canonical regardless of input). Q3.ii is scoped to Node-to-Node federation today, with explicit forward-binding to Node-to-Client sender output where analogous (collectively the `collect_sync_history` + `apply_fanout` history-push sites in `xgen-node/src/fanout.rs`). The Node-to-Client siblings are flagged for future scheduling against their own consumer pressure; not fixed in this milestone.

**Fix scope (what's IN this milestone):**

- Fix `topological_sort_events` at `xgen-node/src/fanout.rs:193` so it produces canonical output regardless of input ordering. The primitive's contract changes from "respects causality" to "canonical, given a fixed event set."
- Fix the sibling Site 1 at `compute_federation_delta_for_space:321` (`HashMap.values()` feed) so federation delta is Q3.ii-compliant end-to-end.

**Q3.ii forward-binding (what's flagged but OUT of this milestone):**

- `collect_sync_history` — Node-to-Client wire output; same bug class; flagged for future scheduling.
- `apply_fanout` history-push — Node-to-Client wire output; same bug class; flagged for future scheduling.

### 4.2 Reasoning

Four reasons, in order of weight:

**Reason 1 — Matches D-067 + D-070 + D-075 locking pattern.** The project has consistently locked principles where they're load-bearing today and forward-bound siblings explicitly. D-067 was locked at single-source-of-truth for derived state reads (not at a codebase-wide HashMap audit). D-070 was locked at transport-layer correlation pair (not at every wire envelope). D-075 was locked at the vantage-aware applier for `state.federation_add` (not at every relationship-shaped event). Q2 middle + Q2.γ follows the same discipline shape.

**Reason 2 — The primitive fix architecturally satisfies D-067 at the topo-sort surface.** Every future consumer of `topological_sort_events` gets canonical output by default. No per-caller drift surface is possible at this layer. The drift surface between `topological_sort_events` (non-canonical) and the canonical sibling `xgen-core/src/node/runtime.rs::topological_sort` (Kahn + `queue_vec.sort()`) is closed.

**Reason 3 — Q2 wide would couple unrelated scope into a federation milestone.** Client-facing surfaces (`collect_sync_history`, `apply_fanout`) belong in their own design discussion with their own consumer-pressure framing. The project has consistently scoped milestones tightly; expanding this milestone's scope to close every wire-visible-ordering site at every boundary would mean Phase 9 stays paused longer than needed, M6 + Pass 1 stay blocked longer than needed, and the work itself becomes a sweep rather than a targeted fix.

**Reason 4 — Q2 narrow under Q2.α would be honest scope discipline but leaves a discipline-pattern inconsistency.** D-067/D-070/D-075 all forward-bound siblings explicitly. Q2 narrow would lock Q3.ii at federation only with no forward-binding language, which the project's discipline pattern argues against.

### 4.3 The Q2.γ forward-binding framing

Q3.ii applies to Node-to-Node federation today, with explicit forward-binding that the principle applies to Node-to-Client sender output **where analogous** and should be reviewed when scheduling allows. D-076 includes the forward-binding language so future event-design or wire-format discussions inherit the principle. Future Chat Claude + Joe revisiting either site picks up the Q3.ii framing already locked.

---

## 5. Q1 lock — Shape A v1 + sibling Site 1 fix

### 5.1 The Joe-lock

**Q1 lock: Shape A v1 + sibling Site 1 fix.**

- **Tie-break source:** event_id lexicographic sort at `topological_sort_events`, applied to ready siblings at each iteration of the outer loop.
- **Sibling Site 1 fix:** sort the `Vec<Event>` at `compute_federation_delta_for_space:321` before passing to `topological_sort_events`. Belt-and-braces: explicit canonical-ordering chain end-to-end matching Q2 middle's letter (primitive fixed + feed canonical).
- **Pass-1 posture:** v1 — `&str` sort with code-comment block at the sort site flagging Pass 3 retype to `EventXgid`. Pass-1-neutral; preserves Pass 1's Status ACTIVE v2.0 unchanged.

### 5.2 Reasoning

Four reasons, in order of weight:

**Reason 1 — Q2 middle's letter wants the primitive fixed at the primitive layer.** Shape A v1 does exactly that with minimum footprint. One sort line in `topological_sort_events`, one sort line in `compute_federation_delta_for_space`, one code-comment block citing D-076 + Appendix J's content-hash framing.

**Reason 2 — Shape C's duplicate-event_id argument is hypothetical.** Content-hash-derived event_ids cannot collide except through SHA-256 break, which is its own different bug. Shape C's contract-layer explicit-canonicality benefit is achieved by Shape A's code-comment block + D-076 citation without the cross-crate dep or performance cost. Canonical bytes include event_id by construction (Appendix C primitive schema), so Shape C's ordering ≈ Shape A's for distinct-event_id events; the additional benefit is concentrated in an edge case the protocol does not currently emit.

**Reason 3 — Shape D.1's structural-depth argument is real but overscopes the milestone.** Touches every `EventStore` consumer for a problem the milestone's surfaced bug doesn't require. D.1's right home would be a separate milestone on `EventStore` canonical-iteration discipline if ever scheduled.

**Reason 4 — Shape A v2's type-level guarantee is real but couples this milestone to Pass 1's `EventXgid` flavour.** Pass 1 is currently blocked behind Phase 9 + Federation milestone close + this topo-sort milestone; coupling here adds dependency surface. v1 with the wrap-or-comment precedent established by XGID Adoption v1 Commit 2 (`Xgid::new(...)` wrap in `SpaceLocalMetadata.introducer_node_id`) + bidirectional Commit 2 (`Xgid::new(peer.to_string())` wrap at the type-boundary entry) is the consistent posture.

### 5.3 The code-comment block at the sort site

Verbatim shape (exact phrasing is implementation latitude):

```rust
// D-076 wire-order determinism (locked at topological-sort
// design-phase close 2026-05-22; sibling-distinct from D-067
// at code-organisation layer and D-075 at event-model layer;
// all three lock no-drift-surface properties explicitly).
//
// Sort ready siblings by event_id lexicographically. event_id is
// content-hash-derived per Appendix J (xgen_appendix_j_en.md), so
// the sort key is byte-stable across senders with identical Space
// history, which is exactly what D-076's "two senders with
// identical state produce byte-identical federation deltas"
// contract obligates.
//
// v1 ships with &str sort; Pass 3 retypes to EventXgid when
// xgen-node-side dispatch widens to XGID flavours. The retype is
// purely type-level; sort semantics unchanged.
```

The code-comment block is mandatory at the sort site per Q2 middle's "fix the primitive's contract once" framing — the comment block is what makes the primitive's new contract explicit to future contributors who would otherwise see only a one-line sort.

### 5.4 Scope of the implementation work

The runbook (next session's authoring) walks the four-commit sequence per bidirectional precedent. Scope summary from the audit's §5.1 in-scope list:

- `topological_sort_events` tie-break (the primitive fix).
- `compute_federation_delta_for_space:321` sibling Site 1 fix.
- Code-comment block at the sort site citing D-076 + Appendix J.
- Unit tests at the primitive level: deterministic output across input permutations; stable tie-break for ready siblings with empty `prev_events`; no-op-equivalence for already-canonically-ordered input.
- Phase 9 Scenario 1 re-resurrection (lift `#[ignore]` again; activating integration-level regression lock for D-076).

---

## 6. Rejected alternatives

Recorded for principle clarity and future-revisit context. The audit doc's §7 carries the full code-grounded cost/benefit; this section captures the rejection reasoning at design-phase close.

### 6.1 Shape A v2 + sibling Site 1 fix (NOT LOCKED)

Same code change as Shape A v1, but typed `EventXgid` from outset (either Pass 1 ships first, or the wrap pattern `Xgid::new(event.event_id.clone())` is used at the call boundary).

**Why rejected:** couples this milestone's design to Pass 1's `EventXgid` flavour. Pass 1 is currently blocked behind Phase 9 + Federation milestone close + this topo-sort milestone; coupling here adds dependency surface in the wrong direction. The wrap-or-comment precedent (`SpaceLocalMetadata.introducer_node_id` v1; `dispatch_event` peer_node_id v1) establishes that the consistent posture is "ship &str at v1, retype under Pass 3 with code-comment marker." Shape A v2 would have been a deliberate departure from that pattern for no concrete v1 benefit (the type-level guarantee is real but not load-bearing for D-076's contract — the content-hash framing in the code-comment block carries the canonicality argument).

### 6.2 Shape C + sibling Site 1 fix (NOT LOCKED)

Sort key is the canonical serialisation of the full event (via `xgen-core::wire::canonical`) instead of event_id alone.

**Why rejected:** the additional benefit over Shape A is concentrated in the duplicate-event_id edge case, which the protocol does not currently emit (event_ids are SHA-256 content hashes; collisions require an SHA-256 break, which is its own different bug). The contract-layer explicit-canonicality benefit is achieved by Shape A's code-comment block + D-076 citation without the cross-crate dep (`xgen-node` → `xgen-core::wire::canonical`) or per-comparison serialisation cost. Shape C would have paid real cost for hypothetical benefit.

### 6.3 Shape D.1 alone (NOT LOCKED)

`EventStore` becomes `BTreeMap<String, Event>` keyed by event_id; canonically sorted iteration at the data layer; `topological_sort_events` needs no change because its input is already canonical.

**Why rejected:** overscopes the milestone. Touches every `EventStore.{get,values,contains,insert}` call site (audit-scope verification required across xgen-core + xgen-node + xgen-client + tests). The Q2 middle "in spirit vs letter" question is real — the primitive itself remains unfixed under Shape D.1; future callers that construct event vectors directly bypass the data-layer canonicality. D.1's right home would be a separate milestone on `EventStore` canonical-iteration discipline if ever scheduled; bundling it into a topo-sort fix loses the focus and forces unrelated scope review.

### 6.4 Shape D.1 + Shape A (NOT LOCKED)

Belt-and-braces version of D.1: data layer canonical (BTreeMap) AND primitive canonical (Shape A sort). The primitive's contract is fixed AND the data layer's contract is fixed.

**Why rejected:** maximum scope of any option. Closes the bug class along two redundant dimensions for one surfaced bug. The bidirectional precedent's discipline ("ship the minimum that closes the surfaced bug + matches the locked principle; don't pre-emptively close related concerns") argues against bundling.

### 6.5 Shape B + Shape D.2 (DISQUALIFIED AT Q3 LOCK)

Shape B (timestamp sort) and Shape D.2 (`IndexMap` insertion order) were disqualified by Q3.ii at lock-time per §3 above. Timestamps are wall-clock and non-canonical across senders (different Nodes have different clocks); insertion order is process-local and non-canonical across Nodes. Both fail Q3.ii's "two senders produce byte-identical deltas" contract structurally.

---

## 7. The principle promoted to D-076

The three Joe-locks above instantiate a more general protocol-design principle, promoted to `DECISIONS.md` D-076 in the same commit as this design task file:

> **Wire-order determinism is a sender-side normative property for Node-to-Node federation.** Two senders with identical Space history MUST produce byte-identical federation deltas (modulo signature-bearing fields that vary by author and time). Wire ordering is part of the protocol's contract, not implementation latitude. Forward-bound by Q2.γ to Node-to-Client sender output where analogous and should be reviewed when scheduling allows.

### 7.1 The no-drift-surface discipline family

D-076 joins D-067 + D-070 + D-075 as the four-decision no-drift-surface discipline family. Each member locks a no-drift-surface property at a different layer:

| Decision | Layer | Property locked |
|---|---|---|
| **D-067** | Code-organisation | Single source of truth for derived state reads (no two readers consulting different sources for the same logical question) |
| **D-070** | Transport-layer | Two events of equal importance, opposite direction (acceptance + rejection signals both exist + both carry envelope-level correlation) |
| **D-075** | Event-model | Relationship-shaped events record one party's act + derived projection with vantage-aware applier logic |
| **D-076** | Wire-format | Two senders with identical state produce byte-identical federation deltas |

The four decisions operate at different layers and address different questions, but share a common posture: **lock the no-drift property explicitly at the layer where it's load-bearing today; forward-bind to sibling surfaces; reject the alternative of leaving the property implicit and trusting it to emerge from local primitives.**

### 7.2 The binding D-076 creates

Future event-design Joe-locks must include "does this event's serialisation produce canonical wire ordering across senders" as a design-phase question. That cost is deliberate, not incidental — it ensures that the next time a protocol-event family is added, the wire-order question is surfaced at design time rather than discovered at integration-test time (which is how D-076 itself surfaced).

D-076 is the first D-NNN to lock a wire-format-normative property explicitly. Future wire-format properties (e.g., canonical JSON serialisation order; canonical UTF-8 normalisation; canonical timestamp precision) layer on D-076 cleanly if they are ever needed.

### 7.3 Forward-binding to Node-to-Client siblings

The Q2.γ framing in §4 is the load-bearing forward-binding for D-076. The two known Node-to-Client analogue sites (`collect_sync_history`, `apply_fanout` history-push) are flagged in the audit's §5.2 + this document's §4.1 as Q3.ii-analogues that the principle applies to but that don't get fixed in this milestone. Future Chat Claude + Joe revisiting either site picks up D-076 directly; the principle does not need re-litigation at the analogue's design phase.

---

## 8. Scope + ordering + downstream coordination

### 8.1 Scope summary

**IN scope** for the implementation runbook:

- `topological_sort_events` primitive fix (one sort line + code-comment block).
- `compute_federation_delta_for_space:321` sibling Site 1 fix (one sort line).
- Three unit tests at the primitive level (deterministic output, stable tie-break, no-op-equivalence).
- Phase 9 Scenario 1 `#[ignore]` lift; integration-level regression lock for D-076.
- D-076 promotion to DECISIONS.md (this design-phase close commit; runbook references the already-promoted D-076).
- Catalogue row addition in `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` per audit's §4.3.

**OUT of scope** (per audit's §5.2):

- `collect_sync_history` + `apply_fanout` history-push fixes (Q2.γ forward-binding flags them; out of this milestone).
- `EventStore` container changes (Shape D.1 territory; separate milestone if ever scheduled).
- Codebase-wide `HashMap` iteration audit (Q2 wide considered and rejected).
- Promoting Q3.ii to spec-normative Ch3 entry (D-076 in DECISIONS.md is sufficient at this phase).
- Test-surface restructuring beyond Phase 9 Scenario 1's regression-witness role.

### 8.2 Commit ordering for the runbook

Four atomic commits expected per bidirectional precedent:

1. **Commit 1 — doc-pass.** Design task file (this document) Status flip ACTIVE → COMPLETED; audit doc Status flip ACTIVE → COMPLETED; canonical design doc `docs/xgen_federation_propagation_design.md` gains §6.4.3 sibling subsection (sibling to §6.4.1 Phase 7.5 + §6.4.2 bidirectional) + §15 Implementation Complete row.
2. **Commit 2 — primitive + sibling fix + unit tests.** Code change in `xgen-node/src/fanout.rs:193` (sort line + code-comment block) + `xgen-node/src/fanout.rs:321` (sort line). Three unit tests added covering deterministic output across input permutations + stable tie-break for ready siblings with empty `prev_events` + no-op-equivalence for already-canonically-ordered input.
3. **Commit 3 — Phase 9 Scenario 1 resurrection.** Lift `#[ignore]` annotation on `two_node_federation_push_smoke_100_messages` in `xgen-node/src/tests/phase9_two_node_smoke.rs`. Verify Scenario 1 passes in isolation and as part of full workspace.
4. **Commit 4 — milestone close.** JOURNAL entry, CLAUDE.md PLAY block flip, ROADMAP.md updates, runbook Status flip ACTIVE → COMPLETED. Catalogue row addition.

### 8.3 Downstream coordination

**Federation Event Propagation milestone closure.** This milestone is now part of the dependency chain. Order: (1) topological-sort milestone closes → (2) Phase 9 Commit 3b resumes → (3) Phase 9 closes → (4) Federation Event Propagation milestone flips PLAY → DONE.

**M6 (new) + Pass 1.** Both unblock simultaneously when Federation Event Propagation milestone closes. Dependency chain extends by one node (this milestone), unchanged in shape.

**MLS coupling (D3 parallel workstream).** D-076 is the precondition for MLS-over-federation. Locking it now means MLS integration does not surface the wire-order question late.

---

## 9. Discipline notes

This design task file is a worked instance of three project-management principles already in DECISIONS.md:

- **D-069 (canonical-document rule).** This document is the canonical record of the design phase's locked decisions. Future references to the locks cite this document.
- **D-071 (subsystem audits precede dependent milestones).** Third recurrence in this milestone — J-081 → Federation design phase (first); bidirectional `federation_nodes` audit → design → impl (second, just closed); topological-sort audit → design (this) → impl + close (third, in flight). The pattern is now durable across three closures.
- **D-074 (milestone-close commits include JOURNAL).** The design-phase close commit includes JOURNAL.md per the same-commit discipline pattern J-095 established and J-096 followed.

### 9.1 The split-session discipline (bidirectional precedent)

Design task file + implementation runbook are kept as separate artefacts in separate sessions, matching the bidirectional precedent. The design task file's audience is future Chat Claude + Joe reading "why did we lock this"; the runbook's audience is Clair reading "what do I ship in what order." Both audiences benefit from different headspace.

Reasoning recorded at the design-phase close decision:

- Three Joe-locks (Q3.ii, Q2, Q1) already landed in the arc; pacing discipline matters at this point in the session-arc.
- The bidirectional precedent is "same-arc, not same-day" — the design task file there was exposition of already-locked decisions, which is the equivalent state here.
- Clair has nothing blocking on her side; bringing her runbook online a session earlier doesn't change downstream timing.

### 9.2 Lock-inline-rather-than-defer pattern (third recurrence)

The bidirectional audit recorded Q2 as code-verified-yes inline at audit lifecycle rather than carrying it into design-phase deliberation. This topological-sort audit followed the same pattern at design-phase opening: all three audit-phase questions (Q1, Q2, Q3) were locked inline in the audit doc rather than carried as open questions for the design phase to deliberate. The design task file then expounded the already-locked decisions rather than walking the questions afresh.

The pattern is now visible across three audit closures (bidirectional, topological-sort, plus the inline-lock at design-phase opening for both). When Joe is ready to lock at audit-doc lifecycle, the audit doc records the lock; the design task file becomes exposition rather than deliberation.

### 9.3 Honest longer work over fast shortcuts — third instance in this milestone

Third recurrence of the principle within the Federation Event Propagation milestone alone (Phase 7.5 was the first; bidirectional was the second; this is the third). Each instance: dependent work surfaced a load-bearing gap; the gap closed properly (audit → design → impl → close) rather than via a workaround.

The pattern's cost is real (each recurrence delays Federation Event Propagation milestone closure by approximately one session-arc). The pattern's benefit is also real (each recurrence is a bug that gets fixed before it ships to production, before downstream code depends on the broken behaviour, before a future contributor has to discover and fix it from a production deployment).

### 9.4 D-076 + the four-decision no-drift-surface family

D-076 brings the no-drift-surface discipline family to four members (D-067, D-070, D-075, D-076). Each at a different layer; each locking a no-drift property explicitly rather than letting it emerge from local primitives. Future contributors reading any of the four find pointers to the other three; the family operates as a single principle expressed across four protocol layers.

---

## 10. Cross-references

### 10.1 Design documents

- **`docs/xgen_federation_propagation_design.md`** (Status: ACTIVE, v1.0) — the canonical Federation Event Propagation design. Runbook Commit 1 adds §6.4.3 sibling subsection (sibling to §6.4.1 Phase 7.5 + §6.4.2 bidirectional) + §15 Implementation Complete row.
- **`tasks/FEDERATION_TOPOSORT_AUDIT.md`** (Status: ACTIVE v1.0 at this commit; flips to COMPLETED in runbook Commit 1) — the audit doc this design phase consumes as input. Three locks recorded inline at audit lifecycle per the lock-inline pattern.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md`** (Status: COMPLETED v1.0) — sibling-shape precedent for the audit doc.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md`** (Status: COMPLETED v1.0) — structural template for this design task file.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`** (Status: COMPLETED v1.1) — sibling-shape precedent for the implementation runbook (to be authored next session).
- **`tasks/FEDERATION_TOPOSORT_IMPL.md`** (to be authored next session) — the implementation runbook. Sibling-shape to bidirectional runbook.

### 10.2 Code surfaces

- `xgen-node/src/fanout.rs::topological_sort_events` (lines 193-220) — Site 2 of the audit's §3, the non-canonical sort primitive. Primitive fix lands here.
- `xgen-node/src/fanout.rs::compute_federation_delta_for_space` (lines 311-333; HashMap feed at ~321) — Site 1 of the audit's §3. Sibling fix lands here.
- `xgen-core/src/node/runtime.rs::topological_sort` (lines 859-912) — **canonical sibling sort precedent.** Kahn's algorithm with explicit `queue_vec.sort()` tie-breaking. The reference site for the primitive fix.
- `xgen-node/src/tests/phase9_two_node_smoke.rs::two_node_federation_push_smoke_100_messages` — Phase 9 Scenario 1 regression witness (`#[ignore]`-annotated; lifts at runbook Commit 3).
- `xgen-node/src/fanout.rs::collect_sync_history` + `apply_fanout` history-push — Q2.γ forward-binding sites; out-of-scope this milestone.

### 10.3 DECISIONS

- **D-067** — Single source of truth. Code-organisation-layer sibling to D-076.
- **D-068** — Original drift-surface decision; precursor framing.
- **D-069** — Canonical-document rule. This document instantiates.
- **D-070** — Transport-layer correlation pair. No-drift-surface family member.
- **D-071** — Subsystem audits precede dependent milestones. Third recurrence in this milestone.
- **D-074** — Milestone-close commits include JOURNAL.
- **D-075** — Event-model vantage-aware applier rule. Event-model-layer sibling to D-076.
- **D-076** (this commit) — Wire-format determinism. Wire-format-layer member of the no-drift-surface discipline family.

### 10.4 JOURNAL

- **J-097** (this commit) — Topological-sort design-phase close. Records the three Joe-locks, D-076 promotion, rejected alternatives, discipline-pattern observations.
- **J-096** — Originating record of the finding. Finding 2 (topological-sort discovery section) is the canonical evidence source.

---

*End of design task file. Implementation runbook authoring is the next-active step for Chat Claude + Joe in a fresh session. The runbook walks the four-commit sequence per §8.2 and produces the Clair-facing artefact at `tasks/FEDERATION_TOPOSORT_IMPL.md`. After the runbook lands, Clair picks up Commit 1; after the runbook's four commits close, this milestone flips PLAY → DONE, Phase 9 Commit 3b resumes, and the Federation Event Propagation milestone closure dependency chain advances by one node.*  
