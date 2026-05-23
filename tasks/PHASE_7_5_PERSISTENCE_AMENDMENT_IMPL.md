# Phase 7.5 Persistence Amendment — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-23 (Runbook authored at sub-amendment milestone runbook-authoring session following design-phase close at J-105. Sibling-in-shape to `tasks/FEDERATION_TOPOSORT_IMPL.md` (COMPLETED v1.2) and `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (COMPLETED v1.1). Six Joe-locks carried forward from design phase (Q1 (a).ii + (a).iii.β + candidate D-NNN flag; Q2 (a) return-vector; Q3 all-three drain helpers; Q4 (a) sentinel-tree in-scope; runbook structural locks: five-commit shape, five Joe-lock checkpoints, verification rigour 5+3, refinement-folded-into-Commit-3, §15 row in Commit 1, §7 discipline notes section). Two runbook-author code-trace findings carried forward to §4 + §4a as narrow-scope framing: Q1 covers only `graph.add_event` Result-handling, NOT the other four silent-discard sites in `ingest_event` (those are out-of-scope per design doc §8 candidate D-NNN flag); recursive drain pattern is Shape β2 (each helper returns `Vec<Event>`; outer concatenates; initial event stays with `process_inbound`'s existing persist site). Per D-065 + D-067 + D-069 + D-071 + D-074 honest-behaviour-over-polite-behaviour discipline.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Framing, reading order, latitude, pre-existing flakes carry-forward

### §1.1 What this runbook is

This runbook is the implementation contract between Chat Claude + Joe (who authored the design-phase locks at J-105) and Clair (who ships the code). The five Joe-locks from design phase (Q1 + Q2 + Q3 + Q4 + Lock #2 walk-discipline) are treated as already-decided, not as open questions Clair re-litigates. The runbook's job is to make the design-phase locks concrete at the file-and-line level so Clair has minimal ambiguity at commit-authoring time.

Six runbook-structural Joe-locks were added at runbook-authoring session open:

1. **Five-commit shape**: Commit 1 doc-pass / Commit 2 Q1 ingest-path / Commit 2a Q2+Q3 dispatch+persist / Commit 3 sentinel-tree refinement + verify / Commit 4 milestone close.
2. **Five Joe-lock checkpoints**: #1 post-Commit-1 doc-pass drift / #2 pre-Commit-2 unit-test list proposal / #3 post-Commit-2 / pre-Commit-2a primitive shape locked / #4 pre-Commit-2a verbatim code-comment block content (now with rungs-list bullet) / #5 post-Commit-2a / pre-Commit-3 sentinel-tree refinement scope.
3. **Verification rigour**: 5 isolated + 3 workspace = 8 green runs minimum at Commit 3.
4. **Sentinel-tree refinement folded into Commit 3** with refinement-vs-rework distinction made explicit in §5.2.
5. **§15 row of canonical design doc** (`docs/xgen_federation_propagation_design.md`) lands in Commit 1 with J-NNN placeholder; placeholder freezes at Commit 4 to the milestone-close JOURNAL entry number.
6. **§7 discipline notes section** present with six sub-sections (§7.1–§7.7); precedent-departure self-defense at §7.1 per topo-sort precedent J-098.

Two code-trace findings from runbook-authoring session pre-draft work shaped §4 and §4a's narrow scope:

- **Q1 scope narrows to `graph.add_event` Result-handling only.** Code-trace surfaced four other silent-discard sites in `ingest_event` beyond the one named in design doc §3 (event_id-missing-return; store.insert silent; two state.apply_event silents). Those four sites belong to the design doc §8 candidate D-NNN "ingest path invariant encoding" future-walk question, not to this milestone's scope. §4 includes an explicit "narrow scope — what stays silent" sub-section.
- **Recursive drain pattern is Shape β2.** Each drain helper returns `Vec<Event>`; `dispatch_event` aggregates via concatenation; `process_inbound` persists the initial event then iterates `additional_persisted` for the drained events. Sibling to the existing `drain_pending_messages` recursion pattern in runtime.rs. §4a documents the shape explicitly so Clair doesn't reinvent the threading.

### §1.2 Reading order for Clair

Per CLAUDE.md Rule 0 (session-open reading sequence):

1. CLAUDE.md PLAY block (current state — persistence-amendment runbook authoring shipped; Clair pickup at Commit 1)
2. JOURNAL.md latest entry (most recently the runbook-authoring J-NNN; previously J-105 design-close)
3. This runbook §1 → §2 → §3 → §4 → §4a → §5 → §6 → §7 → §8

Then for each commit:

- **Commit 1**: this runbook §3 → `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` §3–§6 (the four Joe-locks)
- **Commit 2**: this runbook §4 → design doc §3 (Q1 walk) + DECISIONS.md D-065 + D-069 (audit-vs-design boundary discipline)
- **Commit 2a**: this runbook §4a → design doc §4 + §5 (Q2 + Q3 walks) + the verbatim code-comment block content at Joe-lock checkpoint #4
- **Commit 3**: this runbook §5 → audit doc §8 (Scenario 3 verification contract) + the four sentinel-tree files at `xgen-node/src/tests/`
- **Commit 4**: this runbook §6 → all files in the milestone-close atomic commit per D-074

### §1.3 Latitude — what Clair decides

The five Joe-lock checkpoints frame where Clair surfaces to Joe vs ships from runbook directly. Outside those checkpoints, Clair has latitude on:

- Exact test-helper function names (provided the locked test-name pins are honoured)
- Per-site code-comment wording (provided the locked verbatim block content at checkpoint #4 is honoured)
- Local variable naming + small refactoring to clean up Result-handling sites
- Choice between match-arm vs `if let` for short Result-handling
- Order of changes within a commit (e.g., signature change first vs caller-site fixes first, as long as the commit is green at close)

The locked items at each checkpoint name the Joe-decision content explicitly. Everything else is Clair's call.

### §1.4 Pre-existing flakes carry-forward

Two known intermittent flakes carry forward into this milestone's verification, sibling to topo-sort milestone J-101's carry-forward framing:

1. **Precedence env-var race** (D-068 originated, ~10-20% workspace runs) — caused by parallel test execution touching `XGEN_LOG_LEVEL` environment variable. Did not fire in any of the 8 verification runs at J-101 milestone close. Stays under workspace parallelism; not blocking.
2. **`reconnect_with_existing_tip_small_delta_delivered`** (Phase 3 era, ~10% workspace runs, 0% isolated) — caused by increased Phase-4 parallelism. Did not fire in any of the 8 verification runs at J-101 milestone close. Stays under workspace parallelism; not blocking.

§5.3 verification rigour assumes both flakes carry forward unchanged. If either fires during Commit 3 verification, Clair surfaces at Joe-lock checkpoint #5 with the failure mode for investigation. **Distinguishing "pre-existing flake fired again under different load" vs "Commit 2/2a regression" is the verification rigour's job**; the 5 isolated + 3 workspace = 8 green runs minimum gates this milestone close.

---

## §2 — Sequence overview

### §2.1 Five-commit table

| # | Commit | Layer | Scope |
|---|--------|-------|-------|
| 1 | Doc-pass | Documentation | Canonical design doc `docs/xgen_federation_propagation_design.md` §6.4.4 new sibling subsection + §15 row gain (with J-NNN placeholder); design task file Status flipped ACTIVE → COMPLETED v1.1; audit task file Status already COMPLETED v1.1 at J-105 (no flip needed) |
| 2 | Q1 ingest-path | `xgen-core` + `xgen-node` + tests | `NodeRuntime::ingest_event` signature changes to `pub fn ingest_event(&mut self, event: Event) -> Result<(), GraphError>`; the single internal production caller inside `dispatch_event` updates to handle the Result; `replay_spaces_from_dir` in `xgen-node/src/app.rs` gains sort-on-replay logic (per Q1(a).ii lock); test fixture callers update to consume the Result; verbatim code-comment block at the signature site (Joe-lock checkpoint #4) |
| 2a | Q2+Q3 dispatch+persist | `xgen-core` + `xgen-node` | `DispatchOutcome::Accepted` variant gains `additional_persisted: Vec<Event>` field; all three drain helpers (`drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship`) return `Vec<Event>`; `dispatch_event` aggregates returned vectors at three drain call sites and emits them in the Accepted outcome; `process_inbound` at `xgen-node/src/app.rs` adds the persist-loop block immediately after the existing initial-event persist call; unit tests for the new return-vector aggregation behaviour |
| 3 | Sentinel + verify | `xgen-node/src/tests/` | Four sentinel-tree files (`phase9_harness.rs`, `phase9_three_node_anti_transitivity.rs`, `phase9_drop_and_recover.rs`, `mod.rs`) refined per Q4(a) in-scope lock + design-phase §6 refinement-risk framing; Scenario 3 lift to active (transition FAIL → PASS verifies the persistence fix at integration level); verification rigour 5 isolated + 3 workspace = 8 green runs minimum |
| 4 | Milestone close | All canonical-record artifacts per D-074 | JOURNAL.md milestone-close entry + CLAUDE.md PLAY block flip + ROADMAP.md version bump + design task file's J-NNN placeholder freeze + audit task file's verification contract reference freeze + canonical design doc §15 row's J-NNN placeholder freeze + catalogue M15-equivalent row gain in `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` + Phase 9 task file header `Last updated` paragraph (Commit 3b-1 collapsed; Commit 3b-2-equivalent the next-active Phase 9 unit) |

### §2.2 Files-touched roll-up (across all five commits)

**xgen-core (libcode)**:
- `xgen-core/src/node/runtime.rs` — `ingest_event` signature change (Commit 2); `DispatchOutcome::Accepted` variant change + three drain helpers' signatures + `dispatch_event` aggregation (Commit 2a); test-fixture updates throughout test mods
- `xgen-core/src/dag/graph.rs` — `GraphError` visibility check (may need `pub`); no functional change expected

**xgen-node (libcode)**:
- `xgen-node/src/app.rs` — `replay_spaces_from_dir` sort-on-replay logic (Commit 2); `process_inbound` adds persist-loop block after initial-event persist (Commit 2a)
- `xgen-node/src/tests/phase9_harness.rs` — sentinel-tree refinement (Commit 3)
- `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs` — sentinel-tree refinement (Commit 3)
- `xgen-node/src/tests/phase9_drop_and_recover.rs` — sentinel-tree refinement (Commit 3)
- `xgen-node/src/tests/mod.rs` — sentinel-tree module declarations (Commit 3)

**xgen-client (test fixtures)**:
- Possibly test fixtures calling `ingest_event` if any exist in `xgen-client`'s test surface — Clair verifies during Commit 2 implementation

**Documentation artifacts**:
- `docs/xgen_federation_propagation_design.md` — §6.4.4 new sibling subsection + §15 row (Commit 1)
- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` — already COMPLETED v1.1 at J-105 (no touch this milestone)
- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` — Status flipped ACTIVE → COMPLETED v1.1 at Commit 4 (per topo-sort precedent's design-task-file lifecycle: stays ACTIVE through implementation; flips to COMPLETED at milestone close)
- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` — this runbook; Status flipped ACTIVE → COMPLETED v1.1 at Commit 4
- `tasks/FEDERATION_PROPAGATION_PHASE_9.md` — header `Last updated` paragraph at Commit 4 (Commit 3b-1 collapsed; Commit 3b-2-equivalent next-active)
- `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` — catalogue M15-equivalent row added at Commit 4 (per audit doc §8 verification contract)
- `JOURNAL.md` — milestone-close entry at Commit 4
- `CLAUDE.md` — PLAY block flip + header bump at Commit 4
- `docs/ROADMAP.md` — version bump + tree updates + Present + Past updates at Commit 4

**Code-comment frozen content** (Joe-lock checkpoint #4 governs):
- `xgen-core/src/node/runtime.rs::ingest_event` verbatim block — the four locked structural elements + rungs-list bullet
- Sentinel-tree doc-comments at Commit 3 — J-NNN placeholder freezes per runbook §5.5

### §2.3 Five Joe-lock checkpoints

**Checkpoint #1 — Post-Commit-1 doc-pass drift surface.** After Commit 1 ships, Clair surfaces to Joe if `docs/xgen_federation_propagation_design.md` §6.4.4's drafted content materially diverges from design doc §3–§6's Q1+Q2+Q3+Q4 reasoning, OR if §15 row's J-NNN placeholder structure differs from topo-sort precedent (line 1141 freeze pattern). Routine doc-pass commits don't surface; only canonical-record drift surfaces. Sibling-shape to topo-sort runbook §2.3 checkpoint #1.

**Checkpoint #2 — Pre-Commit-2 unit-test list proposal.** Before Commit 2 ships, Clair proposes the final unit-test list (3-5 tests covering Q1 Result-handling behaviour at the `graph.add_event` site). Joe locks the list. Test names locked in §4 are the proposed seed list; Clair may rename, add 1-2, or drop 1 with rationale. **Reason for this checkpoint**: per topo-sort precedent #2 — unit-test names are durable regression-lock anchors; Joe-lock at this point prevents test-fixture drift between proposal and ship.

**Checkpoint #3 — Post-Commit-2 / pre-Commit-2a primitive shape locked.** After Commit 2 ships and Clair verifies workspace tests pass (5 isolated + 3 workspace under §1.4 flakes carry-forward), but BEFORE Commit 2a begins, Clair confirms the primitive shape locked: `ingest_event` returns `Result<(), GraphError>`; `replay_spaces_from_dir` sort-on-replay is operational; no compile-driven surface ambiguities outstanding. If Commit 2's ship surfaced any drift from §4's locked code-comment block content, Clair surfaces. Otherwise Commit 2a proceeds directly.

**Checkpoint #4 — Pre-Commit-2a verbatim code-comment block content.** Before Commit 2a ships, Clair confirms the verbatim code-comment block at `xgen-core/src/node/runtime.rs::ingest_event` signature site matches the locked content with four structural elements + rungs-list bullet:

> **Locked structural elements:**
>
> 1. **Reference to candidate D-NNN "ingest path invariant encoding"** flagged at JOURNAL J-105 + `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` §8. Future-walk readers find the rung-above question without re-walking the design phase.
> 2. **The silent-discard-pattern-served-two-masters framing** from design doc §3: `graph.add_event` at this site serves two different masters (dispatch_event where validate_event already guarantees predecessor presence; replay_spaces_from_dir where on-disk ordering can produce miss). Q1(a).ii + (a).iii.β answers both.
> 3. **The rung-above-(a).iii.β list** as a 3-line bullet summary (NOT full reasoning, NOT commented-out exploratory code):
>    - **ValidatedEvent wrapper** — compiler-forced correct path via type-constructor discipline
>    - **Sealed traits + visitor pattern** — new-caller shape constraint
>    - **Formal verification** — machine-checked invariants
>    Note: nothing is future-proof in absolute terms; each rung protects against one more class of drift.
> 4. **Narrow-scope note** — "this is the immediate fix; future work flagged at JOURNAL J-105 + design doc §8 candidate D-NNN 'ingest path invariant encoding'; do not broaden scope without Joe-lock at audit phase."

**Reason for this checkpoint**: per topo-sort precedent #4 — verbatim code-comment block content is the canonical pointer future contributors land on when re-walking; locking the content prevents drift between runbook intent and shipped code.

**Checkpoint #5 — Post-Commit-2a / pre-Commit-3 sentinel-tree refinement scope.** After Commit 2a ships, Clair surfaces to Joe with the sentinel-tree refinement scope: which of the four files need refinement vs structural rework (per §5.2's distinction), and whether the rework — if any — folds into Commit 3 or escalates to a fresh Joe-lock conversation. Routine refinement (text-level, assertion-shape, J-NNN-freeze) folds; structural rework (scenario rename, scenario split/collapse, helper introduction) escalates. **Reason for this checkpoint**: per design doc §6 refinement-risk framing — refinements are likely; the audit verification contract (Scenario 3 transition FAIL → PASS) is the regression lock; ensuring the sentinel-tree shape is correct for the locked fix before Commit 3 ships protects against shipping a regression lock that doesn't actually lock the regression.

### §2.4 What this milestone CANNOT close

After Commit 4 ships, the following stay PAUSED / PENDING:

- **Phase 9 Commit 3b-1 numbering effectively skipped.** Commit 3b-1 IS this persistence-amendment milestone close under a different milestone name per Q4(a) lock at design doc §6. Phase 9 resumes at **Commit 3b-2-equivalent** (Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10 — Scenario 3 already verified at integration level by this milestone close's sentinel-tree ship; remaining Phase 9 scope is Scenarios 2 + compounds).
- **Phase 9 milestone** stays PLAY until Phase 9's own milestone-close commit ships (separate ~5-7 atomic-commit sequence per Q4 Lock from J-091).
- **Federation Event Propagation milestone** stays PLAY until Phase 9 closes.
- **M6 (new) implementation + XGID Retrofit Pass 1** stay PENDING (chain extended by one more node by this sub-amendment milestone; depth unchanged in shape).
- **Candidate D-NNN "ingest path invariant encoding"** stays flagged-not-promoted at JOURNAL J-105 + design doc §8. Future walk triggered when (a) dependent work surfaces a concrete drift instance (a).iii.β didn't catch, OR (b) Joe locks the candidate as worth pursuing on philosophical/strategic grounds independent of a surfacing gap.

Dependency chain after this commit:

```
Persistence-amendment milestone close (THIS commit)
  ↓ unblocks
Phase 9 Commit 3b-2-equivalent (Scenarios 2 + compounds)
  ↓ unblocks
Phase 9 milestone close
  ↓ unblocks
Federation Event Propagation milestone close
  ↓ unblocks
M6 (new) implementation + XGID Retrofit Pass 1 (parallel)
```

---

## §3 — Commit 1 doc-pass

### §3.1 Scope

Commit 1 is the doc-pass that aligns canonical-record artifacts with the implementation milestone's locked design. Per topo-sort precedent (J-098 Commit 1), the doc-pass ships ahead of code so that:

- The canonical design doc records the design at lock-time (sibling-in-shape to topo-sort's §6.4.3 + §15 row pattern)
- Status flips on the design task file (ACTIVE → COMPLETED v1.1) acknowledge that the canonical record has absorbed the design
- The J-NNN placeholder at the §15 row creates the freeze site for Commit 4's milestone close

### §3.2 Files in Commit 1

Two files:

1. `docs/xgen_federation_propagation_design.md` — new §6.4.4 sibling subsection + §15 row gain
2. `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` — Status flipped ACTIVE → COMPLETED v1.1 + header `Last updated` paragraph

Both files in one atomic commit. Per D-074 milestone-close discipline NOT yet applied (D-074 fires at Commit 4 for the milestone close); this Commit 1 is a same-commit-atomicity-for-doc-pass shape.

### §3.3 §6.4.4 new sibling subsection content

The canonical design doc currently has §6.4 with three sibling subsections recording the four sub-amendment milestones:

- §6.4.1 — Phase 7.5 F-3 federation-relationship gate framework
- §6.4.2 — Bidirectional `federation_nodes` projection
- §6.4.3 — Topological-sort wire-order determinism (D-076 v1.1)

§6.4.4 is the new sibling subsection covering the persistence-amendment milestone. Content sketch (Clair drafts the prose; this is the structural skeleton):

**§6.4.4 — Persistence amendment (drain-without-persist gap closure)**

Subsection covers:

1. **The gap closed.** Three drain helpers in `xgen-core::node::runtime` re-dispatched released events INSIDE `dispatch_event` and silently dropped the Accepted outcome via `let _ = self.dispatch_event(...)`; `xgen-node::app::process_inbound` persisted only the explicitly-passed event, never the drained ones; on Node restart `replay_spaces_from_dir` only saw the persisted events; in-flight relationship state was un-replayable. Surfaced by Phase 9 Scenario 3 implementation at J-104.

2. **The fix shape (four Joe-locks).** Q1 ingest-path Result-handling at `graph.add_event` + sort-on-replay at `replay_spaces_from_dir`. Q2 return-vector aggregation through dispatch_event. Q3 applied to all three drain helpers. Q4 sentinel-tree in-scope at milestone close.

3. **The narrow-scope discipline.** Q1 covers only `graph.add_event` Result-handling, not the other four silent-discard sites in `ingest_event`. Those four sites belong to the candidate D-NNN "ingest path invariant encoding" future-walk flagged at JOURNAL J-105 + design doc §8. Sibling-shape to topo-sort Commit 2a's `build_room_create_event`-only narrow scope.

4. **The persistence contract.** `process_inbound` persists the initial event (existing behaviour) plus iterates `additional_persisted` (new field on `DispatchOutcome::Accepted`) for the drained events. Storage layer's persist call is the canonical write site; in-memory dispatch outcomes are not the persist authority.

5. **D-NNN candidate.** Flagged but not promoted at this milestone. Future walk triggered by (a) dependent work surfacing concrete drift, OR (b) Joe locking the candidate as worth pursuing.

Length target: ~500-700 words (sibling-in-shape to §6.4.3's length).

### §3.4 §15 row gain — exact phrasing with J-NNN placeholder

The canonical design doc's §15 "Implementation Complete" log records the milestone-close events. Topo-sort Commit 4 added line 1141 (frozen to J-101). This milestone adds the next row immediately after, with J-NNN placeholder:

```markdown
| 2026-05-NN | Persistence amendment | Drain-without-persist gap closure across `xgen-core::node::runtime`'s three drain helpers + `xgen-node::app::process_inbound`'s persist site. `NodeRuntime::ingest_event` returns `Result<(), GraphError>`. `DispatchOutcome::Accepted` gains `additional_persisted: Vec<Event>`. Phase 9 Scenario 3 transition FAIL → PASS at integration level. Sentinel-tree ships atomic at milestone close per Q4(a). [J-NNN] |
```

**J-NNN placeholder freeze rule**: at Commit 4, the placeholder freezes to the milestone-close JOURNAL entry number. The number is unknown at Commit 1 time (next available J-number after the runbook-authoring entry + any intermediate entries). Frozen exactly once at Commit 4; no other reference to this row's J-NNN exists, so the freeze is local. Sibling-shape to topo-sort's line 1141 freeze at Commit 4 of that milestone.

**Date placeholder**: "2026-05-NN" is the implementation-milestone-close date. Replaced at Commit 4 with the actual close date. Clair may use the Commit 4 commit's date or the date of milestone-close JOURNAL entry; sibling-shape to topo-sort's "2026-05-23" choice (matched JOURNAL entry date).

### §3.5 Design task file Status flip

`tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` flips Status: ACTIVE → COMPLETED at v1.1 in this commit. Sibling-shape to topo-sort `tasks/FEDERATION_TOPOSORT_DESIGN.md` lifecycle:

- v1.0 ACTIVE at design close (J-105)
- v1.1 COMPLETED at Commit 1 of implementation milestone

Header `Last updated` paragraph chains a Commit 1 update entry in front of the J-105 design-close entry:

```
> **Last updated**: 2026-05-NN (Commit 1 doc-pass of implementation milestone — Status flipped ACTIVE → COMPLETED v1.1. Canonical design doc `docs/xgen_federation_propagation_design.md` gained §6.4.4 sibling subsection + §15 row with J-NNN placeholder. Design phase content stays authoritative as historical record at design-at-lock-time; runbook `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` ACTIVE v1.0 is the implementation contract Clair ships against. Per D-069 canonical-document discipline. Previous 2026-05-23 J-105 design-close content stands authoritative — see body §3–§6 for the four Joe-locks.) Previous J-105 update: [...]
```

### §3.6 Commit 1 DoD checklist

- [ ] `docs/xgen_federation_propagation_design.md` §6.4.4 subsection drafted (~500-700 words, sibling-in-shape to §6.4.3)
- [ ] `docs/xgen_federation_propagation_design.md` §15 row appended after line 1141 with J-NNN placeholder
- [ ] `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` Status: ACTIVE → COMPLETED v1.1
- [ ] `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` header `Last updated` paragraph chains Commit 1 entry in front of J-105
- [ ] `cargo check --workspace` passes (no code changes, but verify no doc-comment cross-references broke)
- [ ] Workspace test count unchanged (no test changes in Commit 1)
- [ ] No CLAUDE.md / ROADMAP.md / JOURNAL.md touched yet (those land at Commit 4)
- [ ] `Status: COMPLETED` header line is the unflippable success signal (per D-074 lesson: don't include "commit pushed" as a checklist item — it's a chicken-and-egg flag)
- [ ] **Surface to Joe at Checkpoint #1 IF** §6.4.4 prose surfaces drift from design doc §3–§6 reasoning OR §15 row J-NNN placeholder structure differs from topo-sort precedent

### §3.7 Anti-drift guardrails for Commit 1

1. **§6.4.4 must reference §6.4.3 (topo-sort) precedent explicitly.** A future contributor reading §6.4.4 should see the sibling-shape framing inline. Otherwise §6.4.4 reads as a standalone subsection that happens to follow §6.4.3, missing the discipline pattern.
2. **§6.4.4 must reference design doc §8 candidate D-NNN explicitly.** The narrow-scope discipline only holds if future readers see the broader question's flagged-not-promoted status at the right place in the canonical record.
3. **§15 row J-NNN placeholder must use `[J-NNN]` exact syntax** (not `[TBD]` or `<placeholder>`). Topo-sort runbook §5.5 + §6 freeze pattern depends on the grep-able `[J-NNN]` token.
4. **Design task file Status flip date format** matches `YYYY-MM-DD` (not `YYYY-MM-NN`). The header `Last updated` placeholder convention uses `2026-05-NN` only at runbook-authoring time before the close date is known.

---

## §4 — Commit 2: Q1 ingest-path Result-handling

### §4.1 Scope (narrow)

Commit 2 closes the Q1 lock at design doc §3: (a).ii sort-on-replay in `replay_spaces_from_dir` + (a).iii.β `ingest_event` signature change to `Result<(), GraphError>` for compiler-forced caller handling at the one production call site + test-fixture updates.

**Q1 scope is narrow.** It covers ONLY the `graph.add_event` Result-handling at `xgen-core/src/node/runtime.rs` line ~210. The four other silent-discard sites in `ingest_event` stay UNTOUCHED in this commit:

- Line ~190: `match event.event_id.as_ref() { Some(id) => id.clone(), None => return, // unsigned event — reject silently }` — stays
- Line ~212: `let _ = store.insert(event.clone());` — stays  
- Line ~221: `let _ = state.apply_event(&ev, &my_node_id);` (in StateSpaceCreate replay loop) — stays
- Line ~228: `let _ = state.apply_event(&event, &my_node_id);` (in default branch) — stays

The four out-of-scope sites belong to the candidate D-NNN "ingest path invariant encoding" flagged at JOURNAL J-105 + design doc §8. Future-walk-Joe-lock-needed before touching them. Sibling-shape to topo-sort runbook §4's narrow-scope framing for `build_room_create_event`: Commit 2a was scoped to one file; sibling event constructors (`state.federation_add`, `membership.*`, `message.*`) were NOT audited that milestone for similar `prev_events` lies.

### §4.2 The `ingest_event` signature change

**Pre-Commit-2:**
```rust
pub fn ingest_event(&mut self, event: Event) {
    let space_id = if event.space_id.is_empty() {
        match event.event_id.as_ref() {
            Some(id) => id.clone(),
            None => return,
        }
    } else {
        event.space_id.clone()
    };

    self.stores.entry(space_id.clone()).or_default();
    self.graphs.entry(space_id.clone()).or_default();

    let my_node_id = self.node_id.clone();
    let NodeRuntime { spaces, stores, graphs, .. } = self;
    let store = stores.get_mut(&space_id).unwrap();
    let graph = graphs.get_mut(&space_id).unwrap();

    let _ = graph.add_event(&event, store);  // <-- THE ONE SITE THIS COMMIT CHANGES
    let _ = store.insert(event.clone());

    match &event.event_type {
        EventType::StateSpaceCreate => { /* ... */ }
        _ => { /* ... */ }
    }
}
```

**Post-Commit-2:**
```rust
pub fn ingest_event(&mut self, event: Event) -> Result<(), GraphError> {
    let space_id = if event.space_id.is_empty() {
        match event.event_id.as_ref() {
            Some(id) => id.clone(),
            None => return Ok(()),  // unsigned events stay a silent-return-Ok; out-of-scope per §4.1
        }
    } else {
        event.space_id.clone()
    };

    self.stores.entry(space_id.clone()).or_default();
    self.graphs.entry(space_id.clone()).or_default();

    let my_node_id = self.node_id.clone();
    let NodeRuntime { spaces, stores, graphs, .. } = self;
    let store = stores.get_mut(&space_id).unwrap();
    let graph = graphs.get_mut(&space_id).unwrap();

    graph.add_event(&event, store)?;  // <-- Result-propagation at the one site this commit changes
    let _ = store.insert(event.clone());  // out-of-scope per §4.1; stays silent

    match &event.event_type {
        EventType::StateSpaceCreate => { /* ... apply_event stays silent per §4.1 ... */ }
        _ => { /* ... apply_event stays silent per §4.1 ... */ }
    }

    Ok(())
}
```

**The `?` operator at `graph.add_event(&event, store)?;` is the locked Q1(a).iii.β enforcement at type-level.** Compiler-forced: if the caller doesn't handle `Result`, the compile fails. This is the future-proof-against-three-drift-surfaces fix from design doc §3.

### §4.3 The verbatim code-comment block at the signature site

Per Joe-lock checkpoint #4, the signature site must carry the four-element + rungs-list verbatim block. Exact content to land in code:

```rust
/// Insert an Event directly into the DAG and apply it to SpaceState.
/// No 13-step validation — caller is responsible for event correctness.
///
/// Returns `Result<(), GraphError>` so the caller is compiler-forced to
/// acknowledge DAG-insertion errors (Phase 7.5 persistence-amendment
/// milestone, Q1 (a).iii.β lock at design doc §3). Two production call
/// sites for this fn:
///
///   1. `dispatch_event` (xgen-core::node::runtime). `validate_event`
///      Step 9 already guaranteed predecessor presence; an `UnknownPrevEvent`
///      here is a load-bearing bug, not an expected case.
///   2. `replay_spaces_from_dir` (xgen-node::app). On-disk events arrive
///      in store-iteration order, which is NOT guaranteed to respect the
///      DAG. Q1 (a).ii sort-on-replay covers this case at the caller.
///
/// Q1 covers only `graph.add_event` Result-handling at this site. Four
/// other silent-discard sites in this function remain UNTOUCHED:
///   - `event_id`-missing-return (unsigned event guard)
///   - `store.insert` silent (duplicate-event idempotency guard)
///   - Two `apply_event` silents (SpaceState mutation error swallow)
///
/// Those four are flagged at JOURNAL J-105 + `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` §8
/// as candidate D-NNN "ingest path invariant encoding". Do NOT broaden
/// scope to those sites without Joe-lock at a future audit phase.
///
/// Future-rung options above (a).iii.β (recorded at design doc §8 for
/// reference; not promoted in this milestone):
///   - ValidatedEvent wrapper — compiler-forced correct path via type-constructor discipline
///   - Sealed traits + visitor pattern — new-caller shape constraint
///   - Formal verification — machine-checked invariants
/// Nothing is future-proof in absolute terms; each rung protects against
/// one more class of drift while introducing its own assumptions.
pub fn ingest_event(&mut self, event: Event) -> Result<(), GraphError> {
    // ... implementation per §4.2 above ...
}
```

The doc-comment block above the signature is THE locked content. Clair confirms verbatim at Joe-lock checkpoint #4 before Commit 2 ships.

### §4.4 GraphError visibility check

Q1's `Result<(), GraphError>` signature requires `GraphError` to be reachable from the public `ingest_event` signature. Per the runbook-authoring code-trace, the current visibility of `GraphError` in `xgen-core/src/dag/graph.rs` is the one open question for this commit.

If `GraphError` is currently `pub(crate)` or non-`pub`, Commit 2 adds `pub` visibility. This is additive (widening) and doesn't break callers. Sibling-shape to XGID Adoption v1's `canonical_event_bytes` carry-over: when the type appearing in a public signature needs to be re-exported or visibility-widened, the change is mechanical.

No `GraphError` *movement* expected (it stays in `xgen-core/src/dag/graph.rs`); only visibility. Clair verifies during implementation; surfaces at Checkpoint #3 if `GraphError` needs structural change (e.g., new variants, derive changes for the public surface).

### §4.5 The single internal production caller — `dispatch_event`

Inside `xgen-core::node::runtime::dispatch_event`, the call is currently at line ~559:

```rust
self.ingest_event(event);  // pre-Commit-2: no Result; silent fallthrough
```

Post-Commit-2, the call site handles the Result. Recommended shape:

```rust
// `dispatch_event` arrived here only after `validate_event` Step 9
// confirmed predecessor presence; `UnknownPrevEvent` from `ingest_event`
// would be a load-bearing bug.
if let Err(e) = self.ingest_event(event) {
    let event_id_for_log = event_id.as_deref().unwrap_or("(none)").to_string();
    tracing::error!(
        event = "ingest_event_failed",
        space_id = %space_id,
        event_id = %event_id_for_log,
        error = %e,
        "dispatch_event called ingest_event after validate_event Step 9 cleared predecessor presence — unexpected GraphError"
    );
    return DispatchOutcome::Rejected(format!("ingest_event failed: {}", e));
}
```

**Note on event consumption**: the current `ingest_event(event: Event)` consumes `event` by value. Post-Commit-2, the Result-propagation pattern means `event` is consumed by `ingest_event` regardless of Ok/Err. The `event_id` for logging needs to be captured BEFORE the call (as currently done at line ~543: `let event_id = event.event_id.clone();`).

### §4.6 `replay_spaces_from_dir` sort-on-replay (Q1 (a).ii)

At `xgen-node/src/app.rs:2628` (approximate; Clair confirms exact line):

```rust
pub(crate) fn replay_spaces_from_dir(/* ... */) {
    // ... existing setup ...
    // pre-Commit-2:
    // for event in events_from_disk {
    //     self.ingest_event(event);  // silent on errors
    // }
    //
    // post-Commit-2:
    // events arrive in store-iteration order, NOT DAG-topological order;
    // sort-on-replay here ensures graph.add_event sees predecessors in order
    // and never returns UnknownPrevEvent.
    let sorted = topological_sort(events_from_disk);
    for event in sorted {
        if let Err(e) = self.ingest_event(event) {
            tracing::warn!(
                event = "replay_ingest_failed",
                error = %e,
                "replay_spaces_from_dir hit GraphError during ingest — events on disk may be DAG-corrupted or rotated mid-replay"
            );
        }
    }
}
```

**The sort-on-replay** is the defensive layer that prevents UnknownPrevEvent from firing at this caller. The Q1(a).ii lock says: do the sort here, *not* at the `graph.add_event` site, because the site's caller (`dispatch_event`) already has its predecessor-presence guarantee via `validate_event`. Both callers respect the same invariant via different mechanisms appropriate to each call site's context.

The `topological_sort` function is the same Kahn's-algorithm topological sort already present at `xgen-core/src/node/runtime.rs::topological_sort` (called by `ingest_event`'s StateSpaceCreate branch internally). Clair may either:

- Re-export `topological_sort` from xgen-core's public surface and call it from xgen-node (preferred — single source of truth per D-067 + D-076 no-drift-surface family), OR
- Implement a sibling sort in xgen-node (rejected — introduces D-067 drift surface)

**Locked**: re-export `topological_sort` as `pub` from xgen-core. Clair confirms during Checkpoint #3.

### §4.7 Test-fixture caller updates

The `phase_7_5_tests` mod in `xgen-core/src/node/runtime.rs` contains several `node.ingest_event(...)` calls. Post-Commit-2, each becomes:

```rust
// Test-setup callers use .expect() with explanatory message:
node.ingest_event(space_ev).expect("test setup: ingest_event must not fail");
```

**Pattern**: `.expect()` with explanatory message for test-setup ingestion (these are NOT runtime test assertions but fixture-building calls). Failures here indicate the test fixture itself is broken, not the production code.

Other callers Clair finds during implementation:

- `xgen-node/src/tests/cold_start_bootstrap_integration.rs` (likely calls)
- `xgen-node/src/tests/federation_relationship_integration.rs` (likely calls)
- `xgen-node/src/tests/federation_push_integration.rs` (likely calls)
- Other test files in `xgen-node/src/tests/` and possibly `xgen-client/tests/`

Clair audits the workspace at Commit 2 implementation start (`grep -rn 'ingest_event' xgen-*` from project root) to enumerate caller count for the unit-test-list proposal at Checkpoint #2.

### §4.8 Seed unit tests for Commit 2

Clair proposes the final list at Checkpoint #2. Seed candidates (3-5 tests; Clair refines):

1. **`ingest_event_returns_ok_on_successful_insert`** — happy-path regression lock. Construct a Space, sign a state.space_create, call `node.ingest_event(event)`, assert `Ok(())`.

2. **`ingest_event_returns_unknown_prev_event_error_when_predecessor_missing`** — error-path regression lock. Construct a room.create event whose `prev_events` references an event NOT in the store; call `node.ingest_event(event)`; assert `Err(GraphError::UnknownPrevEvent(_))` (or whatever the GraphError variant is named).

3. **`ingest_event_replay_spaces_from_dir_topologically_sorts_before_calling`** — integration-level regression lock for Q1(a).ii. Write events to a test directory in DAG-violating order; call `replay_spaces_from_dir`; assert all events ingest without error AND the resulting SpaceState matches the expected.

4. **`dispatch_event_path_logs_error_and_rejects_when_ingest_unknown_prev_event_fires`** — the load-bearing-bug regression lock. The case where `dispatch_event` calls `ingest_event` and (somehow, indicating a bug) UnknownPrevEvent fires. Assert the tracing::error logs the event and dispatch returns `DispatchOutcome::Rejected`.

5. **(Optional)** `topological_sort_is_publicly_reachable_from_xgen_node` — if `topological_sort` is re-exported as `pub`, lock the re-export against accidental removal. Sibling-shape to topo-sort runbook's seed-test framing for `is_dag_root_type` becoming `pub(crate)`.

### §4.9 Commit 2 DoD checklist

- [ ] `xgen-core/src/node/runtime.rs::ingest_event` signature: `pub fn ingest_event(&mut self, event: Event) -> Result<(), GraphError>`
- [ ] Verbatim code-comment block at signature site matches §4.3 content (Checkpoint #4 lock confirmed)
- [ ] `graph.add_event(&event, store)?;` uses Result-propagation via `?`
- [ ] Other four silent-discard sites in `ingest_event` UNTOUCHED (§4.1 narrow-scope)
- [ ] `xgen-core/src/dag/graph.rs::GraphError` is `pub` (if it wasn't already)
- [ ] `xgen-core/src/node/runtime.rs::topological_sort` is re-exported as `pub` (per §4.6 lock)
- [ ] `dispatch_event`'s call to `ingest_event` handles Result per §4.5 shape
- [ ] `xgen-node/src/app.rs::replay_spaces_from_dir` sorts events topologically before each `ingest_event` call (§4.6)
- [ ] All test-fixture call sites updated with `.expect()` or `.unwrap()` pattern per §4.7
- [ ] Seed unit tests landed per §4.8 (Checkpoint #2 list confirmed by Joe)
- [ ] `cargo build --workspace` is green
- [ ] `cargo test --workspace` passes (current baseline + new seed tests; pre-existing flakes carry-forward per §1.4)
- [ ] Workspace test count increases by N (where N = final seed test count from Checkpoint #2)
- [ ] No CLAUDE.md / ROADMAP.md / JOURNAL.md touched yet (those land at Commit 4)
- [ ] `Status: COMPLETED` on this runbook stays ACTIVE; flips at Commit 4
- [ ] **Surface to Joe at Checkpoint #3 IF** compile-driven surface ambiguities outstanding, OR `GraphError` needs structural change, OR `topological_sort` re-export surfaces drift

### §4.10 Anti-drift guardrails for Commit 2

1. **Narrow scope must hold.** The four out-of-scope silent-discard sites must NOT be touched even if Clair sees an obvious local improvement. Sibling-shape to topo-sort runbook's narrow-scope discipline at Commit 2a: `build_room_create_event` only; sibling constructors NOT audited.
2. **The `?` operator at `graph.add_event` must be the locked syntax.** Not `match`-based handling, not `if let Err`, but the explicit `?` propagation. Reason: the `?` is the compiler-forced Result-handling pattern the design lock named; match-based handling reintroduces opportunity for silent-drop drift.
3. **No `.unwrap()` on `ingest_event` Result in production code.** Test fixtures use `.expect()` per §4.7. Production callers handle the Result explicitly. `unwrap()` in production is a regression on the design discipline.
4. **`topological_sort` re-export must be `pub`, not `pub(crate)`.** xgen-node needs to call it; `pub(crate)` would scope to xgen-core only, requiring a sibling sort in xgen-node (which violates D-067 + D-076 no-drift-surface family).
5. **Verbatim code-comment block at signature site** — the structural elements + rungs-list bullet must match §4.3 exactly. Clair surfaces drift at Checkpoint #4 before Commit 2 ships.
6. **Test naming pins** — the seed test names at §4.8 are the proposal; Joe locks the final list at Checkpoint #2 BEFORE code is written. Once locked, the names become regression-lock anchors and don't drift.
7. **`replay_spaces_from_dir`'s sort-on-replay** must call the re-exported `topological_sort` from xgen-core, NOT a local sibling impl. Drift-surface elimination per D-067.

---

## §4a — Commit 2a: Q2+Q3 dispatch+persist (return-vector aggregation)

### §4a.1 Scope

Commit 2a closes Q2 and Q3 locks at design doc §4 + §5 in one commit:

- **Q2 (a) return-vector**: `DispatchOutcome::Accepted` variant gains `additional_persisted: Vec<Event>` field. Drained Accepted events flow through return-types from drain helpers → dispatch_event → process_inbound → persist_event.
- **Q3 all-three drain helpers**: `drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship` all get the Q2(a) return-vector treatment. Same mechanical change at each site.

Plus the persistence-call site change at xgen-node:

- **`xgen-node::app::process_inbound`** gains the persist-loop block immediately after the existing initial-event persist call. The block iterates `additional_persisted` from the returned `DispatchOutcome::Accepted` and persists each drained event.

This is the substantive code commit for the drain-without-persist gap closure. Q1's Result chain from Commit 2 composes cleanly with Q2's return-vector chain here.

### §4a.2 The `DispatchOutcome::Accepted` variant change

**Pre-Commit-2a:**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchOutcome {
    Accepted { new_joiner: Option<String> },
    HeldPending,
    Rejected(String),
}
```

**Post-Commit-2a:**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchOutcome {
    Accepted {
        new_joiner: Option<String>,
        /// Events that were drained from PendingBuffer and re-dispatched
        /// inside this dispatch_event call, with their dispatch outcomes
        /// resulting in Accepted. The caller is responsible for persisting
        /// each of these in addition to the explicitly-dispatched event.
        ///
        /// Phase 7.5 persistence-amendment milestone Q2 (a) lock (design
        /// doc §4). Shape β2 recursive drain pattern (runbook §4a.4): each
        /// drain-helper level returns its own Vec; outer levels aggregate
        /// via concatenation. Initial event NOT in this vector — stays
        /// with `process_inbound`'s existing persist site.
        additional_persisted: Vec<Event>,
    },
    HeldPending,
    Rejected(String),
}
```

**Pattern-match impact**: every production and test caller that matches `DispatchOutcome::Accepted { new_joiner }` must update to either `DispatchOutcome::Accepted { new_joiner, additional_persisted }` or `DispatchOutcome::Accepted { new_joiner, .. }`. The `..` shorthand is acceptable for callers that don't need `additional_persisted`.

**Search-and-update pass**: Clair runs `grep -rn 'DispatchOutcome::Accepted' xgen-*` and updates every match. Match arms in `dispatch_event`'s drain helpers and in `process_inbound` need full destructure; test-assertion match arms can use `..`.

### §4a.3 The three drain-helper signatures

All three drain helpers change shape identically. Pattern:

**Pre-Commit-2a:**
```rust
fn drain_pending_uniform(
    &mut self,
    space_id: &str,
    resolved_id: &str,
    origin: EventOrigin,
) {
    let ready = /* ... */;
    for ev in ready {
        let _ = self.dispatch_event(ev, origin, None);  // <-- SILENT DISCARD; THE BUG
    }
}
```

**Post-Commit-2a:**
```rust
fn drain_pending_uniform(
    &mut self,
    space_id: &str,
    resolved_id: &str,
    origin: EventOrigin,
) -> Vec<Event> {
    let ready = /* ... */;
    let mut drained: Vec<Event> = Vec::new();
    for ev in ready {
        let ev_clone = ev.clone();  // capture before consumption
        match self.dispatch_event(ev, origin, None) {
            DispatchOutcome::Accepted { new_joiner: _, additional_persisted } => {
                drained.push(ev_clone);  // this event was Accepted; record for caller persist
                drained.extend(additional_persisted);  // and transitive drains too
            }
            DispatchOutcome::HeldPending | DispatchOutcome::Rejected(_) => {
                // not persisted; not in returned vector
            }
        }
    }
    drained
}
```

**Identical pattern at `drain_pending_by_identity` and `drain_pending_by_federation_relationship`.** Three drain helpers, same mechanical change, sibling-in-shape per Q3 lock.

### §4a.4 Recursive drain pattern — Shape β2

Shape β2 was locked at runbook-authoring session. Each drain level returns its own `Vec<Event>`; outer levels aggregate via concatenation:

```
dispatch_event(initial_event)
  │
  ├─ ingests initial_event
  │
  ├─ drain_pending_uniform(...)         → Vec<Event> {ev_A, ev_B}
  │     for ev_A: dispatch_event recursively
  │       drain_pending_uniform(...)     → Vec<Event> {ev_A1}
  │       └─ returned bubbled up
  │     drained_at_this_level = [ev_A, ev_A1, ev_B]
  │
  ├─ drain_pending_by_identity(...)     → Vec<Event> {}
  │
  ├─ drain_pending_by_federation_relationship(...) → Vec<Event> {ev_C}
  │
  └─ returns DispatchOutcome::Accepted {
        new_joiner: None,
        additional_persisted: [ev_A, ev_A1, ev_B, ev_C]  // concatenation
      }
```

**Initial event is NOT in the returned vector.** `process_inbound` persists the initial event via its existing call site; `additional_persisted` is purely the drained-and-Accepted events.

**Why β2 over β1** (accumulator pattern threaded through recursion):

1. Self-documenting signatures (`-> Vec<Event>` vs `accumulator: &mut Vec<Event>`)
2. Easier code-review and regression-trace
3. Bounded recursion depth (DAG depth ~5-15 events typical) makes Vec allocation cost negligible
4. Sibling-shape to existing `drain_pending_messages` recursion pattern in runtime.rs
5. Avoids "outer caller forgets accumulator" footgun

Full walk at JOURNAL J-NNN (runbook-authoring entry) + design doc cross-reference at §4a.

### §4a.5 `dispatch_event` aggregation

Inside `dispatch_event`, the three drain calls aggregate as follows. Current shape (Step 6 + Step 7):

```rust
// Step 6 — Drain pending events whose missing predecessor just arrived.
if let Some(eid) = event_id.as_deref() {
    self.drain_pending_uniform(&space_id, eid, origin);  // <-- silent
}

// Step 7 — Phase 7.5 §6 federation-relationship arrival hook.
if let Some((peer, sp)) = fed_add_drain_pair {
    self.drain_pending_by_federation_relationship(&peer, &sp, origin);  // <-- silent
}

DispatchOutcome::Accepted { new_joiner }
```

Post-Commit-2a:

```rust
let mut additional_persisted: Vec<Event> = Vec::new();

// Step 6 — Drain pending events whose missing predecessor just arrived.
if let Some(eid) = event_id.as_deref() {
    additional_persisted.extend(self.drain_pending_uniform(&space_id, eid, origin));
}

// Step 7 — Phase 7.5 §6 federation-relationship arrival hook.
if let Some((peer, sp)) = fed_add_drain_pair {
    additional_persisted.extend(self.drain_pending_by_federation_relationship(&peer, &sp, origin));
}

DispatchOutcome::Accepted { new_joiner, additional_persisted }
```

**Note on `drain_pending_by_identity`**: this drain helper is called from `xgen-node::app::handle_identity_replicate_msg`, NOT from inside `dispatch_event`. Its return value flows back to `handle_identity_replicate_msg`, which then persists the drained events using the same pattern as `process_inbound`.

### §4a.6 The `process_inbound` persist-loop block

At `xgen-node/src/app.rs` (process_inbound function, currently around line 1500-1550; Clair confirms exact line):

**Pre-Commit-2a:**
```rust
let outcome = self.runtime.dispatch_event(event.clone(), origin, peer_node_id);
match outcome {
    DispatchOutcome::Accepted { new_joiner } => {
        // persist the initial event
        self.persist_event(&event)?;
        // fan-out logic ...
    }
    DispatchOutcome::HeldPending => { /* ... */ }
    DispatchOutcome::Rejected(reason) => { /* ... */ }
}
```

**Post-Commit-2a:**
```rust
let outcome = self.runtime.dispatch_event(event.clone(), origin, peer_node_id);
match outcome {
    DispatchOutcome::Accepted { new_joiner, additional_persisted } => {
        // persist the initial event (existing behaviour)
        self.persist_event(&event)?;
        // persist any events drained-and-Accepted inside dispatch_event (NEW)
        for drained_event in additional_persisted {
            if let Err(e) = self.persist_event(&drained_event) {
                tracing::warn!(
                    event = "persist_drained_event_failed",
                    error = %e,
                    "failed to persist event drained via dispatch_event cascade; in-flight state may not survive restart"
                );
                // continue persisting other drained events; one failure doesn't abort the cascade
            }
        }
        // fan-out logic ...
    }
    DispatchOutcome::HeldPending => { /* ... */ }
    DispatchOutcome::Rejected(reason) => { /* ... */ }
}
```

**Persist-failure handling note**: the `for drained_event in additional_persisted` loop tolerates per-event persist failures with a tracing::warn rather than aborting the entire cascade. Reason: a single persist failure in the middle of a cascade shouldn't lose subsequent events that might persist successfully. This is honest-broadening framing per D-065: log the failure clearly, continue best-effort.

A future audit may walk this and lock a more aggressive failure policy. Out-of-scope for this milestone.

### §4a.7 Seed unit tests for Commit 2a

Clair proposes the final list; seed candidates:

1. **`dispatch_event_returns_additional_persisted_from_drain_pending_uniform`** — happy-path regression lock for Q2(a) at the predecessor-arrival drain. Construct a scenario where event A arrives, then event B arrives whose predecessor is A; assert `DispatchOutcome::Accepted { additional_persisted: [B], .. }` from A's dispatch (because B was drained).

2. **`dispatch_event_returns_additional_persisted_from_drain_pending_by_federation_relationship`** — same pattern for the Phase 7.5 / F-3 drain helper. Sibling-shape to test 1.

3. **`dispatch_event_aggregates_additional_persisted_across_multiple_drains`** — multi-drain aggregation regression lock. Construct a scenario where the initial dispatch triggers multiple drain helpers (e.g., predecessor-uniform + federation-relationship), assert `additional_persisted` contains events from both.

4. **`drain_pending_uniform_returns_empty_when_no_ready_events`** — the negative case. No buffered events; `Vec::new()` returned.

5. **`recursive_drain_flattens_into_outer_additional_persisted`** — the β2 recursive pattern regression lock. Construct a deeper cascade (A arrives, drains B which drains C); assert all of B and C land in A's `additional_persisted` (not just B).

6. **(Optional)** `process_inbound_persists_initial_event_and_additional_persisted` — integration-level lock at the xgen-node boundary. Less unit-test-shaped; might fold into Phase 9 Scenario 3's coverage rather than ship as a standalone unit test.

### §4a.8 Commit 2a DoD checklist

- [ ] `DispatchOutcome::Accepted` variant gains `additional_persisted: Vec<Event>` field
- [ ] All three drain helpers return `Vec<Event>`
- [ ] `dispatch_event` Step 6 + Step 7 aggregate returned vectors per §4a.5
- [ ] All callers of `DispatchOutcome::Accepted { new_joiner }` updated to handle new field (via `..` or full destructure)
- [ ] `xgen-node/src/app.rs::process_inbound` adds persist-loop block per §4a.6
- [ ] `xgen-node/src/app.rs::handle_identity_replicate_msg` adds persist-loop block (sibling to process_inbound) for drained events from `drain_pending_by_identity`
- [ ] Per-event persist-failure handling per §4a.6 (tracing::warn, continue cascade)
- [ ] Seed unit tests landed per §4a.7
- [ ] `cargo build --workspace` is green
- [ ] `cargo test --workspace` passes (Commit 2 baseline + Commit 2a new tests; pre-existing flakes carry-forward)
- [ ] Workspace test count increases by N more (where N = Commit 2a seed test count)
- [ ] `Status: COMPLETED` on this runbook stays ACTIVE; flips at Commit 4
- [ ] **Surface to Joe at Checkpoint #5 IF** Commit 3 sentinel-tree refinement scope surfaces structural rework (not just refinement) OR pre-existing flakes fire during workspace verification

### §4a.9 Anti-drift guardrails for Commit 2a

1. **All three drain helpers MUST change identically.** Per Q3 lock: same-family-same-atomic-close discipline. Closing one drain helper and leaving two untouched is shipping a known-incomplete fix and reintroduces the drain-without-persist gap at the untouched sites.
2. **`DispatchOutcome::Accepted` variant change is a breaking pattern-match change.** Clair must `grep -rn 'DispatchOutcome::Accepted' xgen-*` and update every match arm. Missing one site fails compile (good) but might fail in production code with `..` shorthand fall-through (bad). Clair audits the workspace exhaustively at Commit 2a start.
3. **β2 recursive pattern must hold at all three drain helpers.** Initial event NOT in returned vector; drained events ARE in returned vector. Drift between drain helpers would produce inconsistent `additional_persisted` semantics, breaking the cascade aggregation invariant.
4. **`process_inbound`'s persist-loop must come AFTER the initial-event persist call**, not before. Order matters: the initial event's persist must succeed before drained events are persisted, because drained events' DAG predecessors include the initial event (in the cascade case where this matters).
5. **Per-event persist failure handling must be tracing::warn + continue**, not panic or early-return. The honest-broadening framing per D-065 (§4a.6); future audits may revisit but this milestone locks tracing::warn + continue.
6. **No new fields on `DispatchOutcome::HeldPending` or `Rejected` variants.** Only `Accepted` gains the new field. Adding `additional_persisted` to non-Accepted variants would be semantically incoherent.
7. **`additional_persisted` doc-comment** must reference design doc §4 + runbook §4a.4 explicitly. Future contributors land here when re-walking the persistence contract.

---

## §5 — Commit 3: Sentinel-tree refinement + verify

### §5.1 Scope

Commit 3 is the integration-level closure for this milestone. Three things happen atomically:

1. **Four sentinel-tree files** (`phase9_harness.rs`, `phase9_three_node_anti_transitivity.rs`, `phase9_drop_and_recover.rs`, `mod.rs`) get refined per §5.2 distinction.
2. **Scenario 3** transitions FAIL → PASS, becoming the activating regression lock at integration level for the persistence-amendment fix (sibling-shape to Scenario 1's role at J-101 for D-075 + D-076 v1.1).
3. **Verification rigour** at §5.3: 5 isolated + 3 workspace = 8 green runs minimum.

The sentinel-tree files are currently uncommitted in Clair's working tree (sentinel preservation per J-104; do NOT `git restore`). Commit 3 includes them in the atomic commit alongside the refinements.

### §5.2 Refinement vs structural rework distinction

Per design doc §6 + Lock 4 framing: refinements are likely; structural rework is what triggers Checkpoint #5.

**Refinement (folds into Commit 3 without escalation)**:
- Doc-comment text changes
- Assertion message rewording (`assert!(x, "better error message")`)
- Test-helper function renames within a single file
- Small-scope `#[serial_test::serial]` posture decisions on individual tests
- Adding or removing a test step inside an existing scenario that doesn't change scenario semantics
- Matching test data structure to actual `DispatchOutcome::Accepted { additional_persisted }` shape
- J-NNN placeholder freezes (runbook §5.5)
- Reordering imports or const declarations
- Local variable renames for clarity

**Structural rework (escalates to Checkpoint #5)**:
- Renaming a scenario file (`phase9_drop_and_recover.rs` → something else)
- Splitting a scenario into two scenarios
- Collapsing two scenarios into one
- Changing the assertion shape from "expect Accepted with N persisted events" to "expect Accepted then restart-then-replay-matches"
- Introducing a new helper function that other tests depend on
- Changing the harness shape (e.g., switching from N-node test pattern to single-node pattern)
- Adding a new sentinel-tree file beyond the four currently uncommitted

Clair surfaces at Checkpoint #5 if any structural rework is needed. Joe locks the rework scope; the rework either folds into Commit 3 (if small enough that the milestone-close timeline accommodates) or escalates to a fresh Joe-lock conversation.

### §5.3 Verification rigour — 5 isolated + 3 workspace = 8 green runs minimum

Per Lock 3, the verification protocol is:

**5 isolated runs**: between each, run `cargo clean`. Each isolated run is `cargo test --workspace --no-fail-fast`. Capture output for each.

**3 workspace runs**: no `cargo clean` between; runs in immediate succession to surface parallelism-induced flakes that isolated runs miss.

**Total: 8 runs minimum. All 8 must be green.** Pre-existing flakes per §1.4 must NOT fire (if they do, Clair surfaces at Checkpoint #5 for investigation).

Sibling-shape to topo-sort J-101 verification at runbook §5.3 of that milestone. Pre-existing flakes did not fire in any of J-101's 8 runs; same expectation here. If they fire, the verification surfaces a new failure mode worth investigating (could be a Commit 2/2a regression that manifests under load, not a flake).

### §5.4 `#[serial_test::serial]` posture

**Default: keep silent.** Sibling-shape to topo-sort runbook §5.4: do not add `#[serial_test::serial]` to sentinel-tree tests unless workspace verification surfaces a parallelism-induced failure.

**Why default-keep-silent**: Adding `#[serial_test::serial]` is a load-bearing claim that the test interacts with shared state in a way that demands serialised execution. The sentinel-tree tests are isolated by design (each constructs its own NodeRuntime, its own temp directory for persistence, etc.). Adding `serial` annotation when not actually needed misleads future readers into thinking shared state exists when it doesn't.

If workspace runs surface a parallelism-induced failure, Clair surfaces at Checkpoint #5 with the specific failure mode and Joe locks the fix.

### §5.5 Sentinel-tree doc-comment freezes

The four sentinel-tree files contain doc-comments referring to J-NNN placeholders for the milestone-close JOURNAL entry. At Commit 3 (or Commit 4; Clair confirms), the placeholders freeze to the milestone-close J-number.

Sibling-shape to topo-sort runbook §5.5: doc-comment text targets the milestone-close JOURNAL entry chronology + decision-regression-lock framing.

**Suggested doc-comment shape** for each sentinel-tree file (Clair refines):

```rust
//! Phase 9 Scenario 3 — drop-and-recover regression lock for persistence-
//! amendment milestone (J-NNN, milestone-close).
//!
//! Verification contract: this scenario fails before the persistence-amendment
//! fix (drain-without-persist gap at the xgen-core ↔ xgen-node layer boundary)
//! and passes after. Three locked decisions converge here:
//!
//!   1. Q1 ingest-path Result-handling (design doc §3, runbook §4).
//!   2. Q2 return-vector aggregation through dispatch_event (design §4, runbook §4a).
//!   3. Q3 applied to all three drain helpers (design §5, runbook §4a.3).
//!
//! Scenario chronology:
//!   - State events arrive at Node A in dispatch order [...]
//!   - Drain helpers re-dispatch HeldPending events via additional_persisted
//!   - process_inbound persists initial + additional_persisted
//!   - Node A restart; replay_spaces_from_dir reads persisted events
//!   - SpaceState matches pre-restart state — verification passes
```

J-NNN freeze rule: the placeholder `J-NNN` becomes the actual J-number of the milestone-close JOURNAL entry. Frozen at Commit 4 alongside the other J-NNN freezes (canonical design doc §15 row, catalogue M15-equivalent row, this runbook's references).

### §5.6 Commit 3 DoD checklist

- [ ] Four sentinel-tree files in `xgen-node/src/tests/` reviewed against §5.2 distinction
- [ ] All refinement-class changes applied directly
- [ ] Any structural-rework-class changes escalated to Checkpoint #5 and resolved
- [ ] Sentinel-tree doc-comments per §5.5 (J-NNN still placeholder, freezes at Commit 4)
- [ ] Scenario 3 transitions FAIL → PASS
- [ ] 5 isolated runs (`cargo clean` between each) all green
- [ ] 3 workspace runs (consecutive) all green
- [ ] Pre-existing flakes per §1.4 did NOT fire in any of the 8 runs
- [ ] Total runs: 8 minimum. If 8 green, ship. If any fail, surface at Checkpoint #5.
- [ ] Workspace test count includes the sentinel-tree tests now passing (previously `#[ignore]` or FAIL)
- [ ] No CLAUDE.md / ROADMAP.md / JOURNAL.md touched yet (Commit 4)
- [ ] `Status: COMPLETED` on this runbook stays ACTIVE; flips at Commit 4

### §5.7 Anti-drift guardrails for Commit 3

1. **Refinement-vs-rework distinction must hold per §5.2.** Refinement does NOT mean "everything I want to change." Structural rework escalates; refinement folds.
2. **8 green runs minimum is the floor, not a stretch goal.** If 7 of 8 green and 1 flaky, surface. Don't ship from 7/8 + assume flake.
3. **Pre-existing flakes that fire MUST be investigated.** They did not fire in J-101's 8 runs. If they fire here under different load, the failure mode is worth a Joe-lock conversation — could be a Commit 2/2a regression that surfaces under workspace parallelism.
4. **J-NNN placeholders stay as `J-NNN` literal token through Commit 3.** Freeze at Commit 4 only. Premature freeze before the milestone-close J-number is determined would require a Commit 4 backout.
5. **`#[serial_test::serial]` posture default is silent.** Adding annotation requires a load-bearing reason surfaced at Checkpoint #5.
6. **Sentinel-tree refinement must NOT change scenario semantics.** Scenario 3 today is "drop-and-recover"; post-refinement it must still be "drop-and-recover" semantically, even if the test code changes substantially.
7. **No new sentinel-tree files beyond the four already uncommitted.** Adding a fifth file is structural-rework-class and escalates to Checkpoint #5.

---

## §6 — Commit 4: Milestone close

### §6.1 Scope

Commit 4 is the atomic milestone-close commit per D-074 eighth-instance discipline. All canonical-record artifacts update in one atomic commit. No code changes in Commit 4 — it's purely the documentation + status-flip commit that crowns the milestone.

### §6.2 Files in Commit 4

Approximately twelve files (Clair confirms exact count during authoring):

1. **`JOURNAL.md`** — new J-NNN milestone-close entry chained ahead of the prior latest entry; header date bumped
2. **`CLAUDE.md`** — PLAY block flipped from "persistence-amendment runbook authoring shipped; Clair pickup at Commit 1" (this milestone's previous PLAY state) to "Phase 9 Commit 3b-2-equivalent" (next-active for Clair); previous PLAY state demoted to DONE-IN-FLIGHT block; header bump
3. **`docs/ROADMAP.md`** — version bump; visual structure tree's persistence-amendment cluster all-✅ with five-commit sub-bullets; Phase 9 row's PAUSED-at-Commit-3b-1 annotation flipped to RESUMES-at-Commit-3b-2-equivalent; Present section's persistence-amendment PLAY entry replaced with Phase 9 Commit 3b-2-equivalent RESUMED PLAY entry; Past section gains persistence-amendment implementation-milestone-CLOSED paragraph under persistence-amendment sub-cluster (sibling-shape to topo-sort milestone-CLOSED paragraph at J-101); cross-cutting candidate D-NNN flag stays 🟡 with reference to JOURNAL J-NNN milestone-close entry
4. **`tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md`** — Status flipped ACTIVE → COMPLETED v1.1 (this runbook); header `Last updated` paragraph chains Commit 4 entry
5. **`tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md`** — already COMPLETED v1.1 at Commit 1; no flip here, but header `Last updated` may chain a Commit 4 reference if Clair locks the J-NNN explicitly
6. **`tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md`** — already COMPLETED v1.1 at J-105; no touch this milestone (sibling-shape to topo-sort audit doc lifecycle at J-101 milestone close)
7. **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** — header `Last updated` paragraph: Commit 3b-1 collapsed into persistence-amendment milestone close; Phase 9 RESUMES at Commit 3b-2-equivalent (Scenarios 2 + compounds C2/C3/C5/C7/C9/C10 remaining; Scenario 3 verified at integration level by sentinel-tree ship at this milestone)
8. **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** — new catalogue M15-equivalent row added per audit doc §8 verification contract (exact phrasing at §6.3 below)
9. **`docs/xgen_federation_propagation_design.md`** — §15 row J-NNN placeholder freezes to milestone-close J-number (the placeholder landed at Commit 1; freezes here)
10. **`xgen-node/src/tests/phase9_drop_and_recover.rs`** — doc-comment J-NNN placeholder freezes (sibling sites at sentinel-tree files; §5.5)
11. **`xgen-node/src/tests/phase9_three_node_anti_transitivity.rs`** — doc-comment J-NNN placeholder freezes (if present)
12. **`xgen-node/src/tests/phase9_harness.rs`** — doc-comment J-NNN placeholder freezes (if present)

Clair audits during Commit 4 authoring: `grep -rn 'J-NNN' .` from project root surfaces all freeze sites; all must freeze in this commit. Sibling-shape to topo-sort J-101's four J-NNN freezes (line 1140 retroactive J-096; line 1141 + M15 + phase9 doc-comment → J-101).

### §6.3 Catalogue M15-equivalent row exact phrasing

`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` gains the new row per audit doc §8 verification contract. Suggested phrasing (Clair refines, Joe locks):

```markdown
| M16 | Drain-without-persist gap | HIGH | Scenario 3 (drop-and-recover) | Three drain helpers in xgen-core::node::runtime re-dispatch released events INSIDE dispatch_event and silently drop the Accepted outcome; xgen-node::app::process_inbound persists only the explicitly-passed event; on Node restart, replay_spaces_from_dir only sees the persisted events and in-flight relationship state is un-replayable. Closed at persistence-amendment sub-amendment milestone close [J-NNN] under Q1+Q2+Q3+Q4 locks (design `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` §3–§6; implementation runbook `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` §4–§5; J-NNN milestone-close entry). Layered-B3 second project-wide instance (sibling-shape to topo-sort Commit 2a layered-B3 at J-101). Activating regression lock: Scenario 3 transition FAIL → PASS at integration level (sibling-shape to Scenario 1's role for D-075 + D-076 v1.1 at J-101). |
```

The M16 numbering is a placeholder; Clair confirms actual next-available catalogue number at Commit 4 authoring time. Sibling-shape to topo-sort milestone-close M15 row.

### §6.4 JOURNAL.md J-NNN milestone-close entry shape

The milestone-close JOURNAL entry follows the seven-sub-section shape per topo-sort J-101 precedent:

1. **Header**: Date + J-NNN + summary line ("Persistence-amendment sub-amendment milestone CLOSED")
2. **Sub-section 1 — Five-commit sequence shipped**: Commit 1 doc-pass + Commit 2 Q1 ingest-path + Commit 2a Q2+Q3 dispatch+persist + Commit 3 sentinel-tree refinement + verify + Commit 4 milestone close (this commit). Each with commit hash + brief content summary.
3. **Sub-section 2 — Layered-B3 second project-wide instance**: name the pattern explicitly; cite topo-sort Commit 2a J-101 as first instance; cite this milestone's drain-hook layer + runtime.rs:181 surface as second instance. Two instances is not yet a durable pattern; three would be.
4. **Sub-section 3 — Pre-existing flake non-firing**: assert the 8 verification runs (5 isolated + 3 workspace) all green; pre-existing flakes did NOT fire. Sibling-shape to J-101 sub-section 3.
5. **Sub-section 4 — D-074 application count**: eighth instance at this milestone-close (J-095 first; J-096 second; J-097 third; J-098-across-two-commits fourth; J-099 fifth; J-100 sixth; J-101 seventh; this commit eighth). Grep-and-quote pattern at authoring time for verification.
6. **Sub-section 5 — "Honest longer work over fast shortcuts" recurrence count**: count inherited at eighth from J-104 (milestone-event), NOT incremented at design close J-105 (design-event), NOT incremented at this milestone close (close-event-not-recurrence-event). Sibling-shape to topo-sort J-101's seventh-recurrence framing.
7. **Sub-section 6 — Candidate D-NNN flag**: "ingest path invariant encoding" stays flagged-not-promoted. Reference design doc §8 + this milestone's narrow-scope discipline. Future walk triggered by (a) dependent work surfacing concrete drift, OR (b) Joe locks the candidate on philosophical/strategic grounds.
8. **Sub-section 7 — What this milestone does NOT close**: Phase 9 Commit 3b-1 numbering effectively skipped (collapsed into this milestone); Phase 9 resumes at Commit 3b-2-equivalent (Scenarios 2 + compounds remaining); Phase 9 milestone stays PLAY; Federation Event Propagation milestone stays PLAY; M6 (new) + XGID Retrofit Pass 1 stay PENDING.

Entry length: ~2000-3000 words sibling-in-shape to topo-sort J-101's length.

### §6.5 CLAUDE.md PLAY block flip

The PLAY block currently reads (at runbook-authoring time):

```
PLAY: « persistence-amendment runbook authoring shipped; Clair pickup at Commit 1 »
```

At Commit 4, the PLAY block flips to:

```
PLAY: « Phase 9 Commit 3b-2-equivalent » (Scenarios 2 + compounds C2/C3/C5/C7/C9/C10 remaining; ~5-7 atomic commits in their own sequence per Q4 Lock from J-091; Scenario 3 verified at integration level by persistence-amendment milestone close per J-NNN)
```

The previous PLAY entry ("persistence-amendment runbook authoring shipped; Clair pickup at Commit 1") demotes to DONE-IN-FLIGHT block per CLAUDE.md's normal PLAY → DONE-IN-FLIGHT → DONE rotation.

### §6.6 ROADMAP.md update detail

Version bumps from `v1.20` (J-105) to `v1.21` (this commit). Five edits:

1. **Visual structure tree**: persistence-amendment sub-cluster Audit ✅ stays; Design ✅ stays; "Implementation runbook authoring" row flipped 🟢 → ✅ with "shipped" annotation; new sub-bullet under 🟡 Implementation row collapses (full implementation now ✅ across five commits); new sub-bullet under 🟡 Milestone close also collapses (✅). Phase 9 row's annotation flipped "PAUSED at Commit 3b-1, COLLAPSES into persistence-amendment milestone close" to "Commit 3b-2-equivalent RESUMES after persistence-amendment milestone close".
2. **Present section**: persistence-amendment milestone PLAY entry replaced with Phase 9 Commit 3b-2-equivalent RESUMED PLAY entry naming Scenarios 2 + compounds. Persistence-amendment milestone collapses to ⬛-CLOSED pointer referencing Past.
3. **Past section**: gains persistence-amendment implementation-milestone-CLOSED paragraph under the persistence-amendment sub-cluster (sibling-shape to topo-sort milestone-CLOSED paragraph at J-101). Length ~1500-2000 words covering all five commits' substantive shape.
4. **"What's playing right now?" frontier line**: rewritten — Phase 9 Commit 3b-2-equivalent next-active for Clair; persistence-amendment milestone moves to ✅ Past.
5. **"What's the live frontier?" frontier line**: parallel-eligible items now read "Phase 9 Commit 3b-2-equivalent (Clair — Scenarios 2 + compounds); M6 (new) Block 4 verb-by-verb walks (Chat Claude + Joe); future-walk of candidate D-NNN 'ingest path invariant encoding' if Joe locks it".

Header `Last updated` paragraph chains a Commit 4 update entry in front of the J-105 entry.

### §6.7 Commit 4 DoD checklist

- [ ] All twelve files (§6.2) in one atomic commit per D-074
- [ ] JOURNAL.md J-NNN milestone-close entry per §6.4 shape
- [ ] CLAUDE.md PLAY block flipped per §6.5; previous PLAY demoted to DONE-IN-FLIGHT
- [ ] CLAUDE.md header bumped
- [ ] docs/ROADMAP.md five-edit update per §6.6
- [ ] docs/ROADMAP.md version: v1.20 → v1.21
- [ ] tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md Status: ACTIVE → COMPLETED v1.1 (this runbook flips at Commit 4)
- [ ] tasks/FEDERATION_PROPAGATION_PHASE_9.md header `Last updated`: Commit 3b-1 collapsed; RESUMES at Commit 3b-2-equivalent
- [ ] tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md new catalogue M16 row added per §6.3
- [ ] All `J-NNN` placeholders frozen to milestone-close J-number across: design doc §15 row, sentinel-tree doc-comments, catalogue M16 row
- [ ] `grep -rn 'J-NNN' .` from project root returns ZERO matches after Commit 4 stages
- [ ] No code changes in Commit 4 (purely documentation + status flips)
- [ ] `cargo build --workspace` is green (no code touch, but verify)
- [ ] `cargo test --workspace` passes at Commit 3 baseline (no test changes in Commit 4)
- [ ] D-074 application count: this is the eighth instance; JOURNAL entry sub-section 4 records the count + grep-quoted application list
- [ ] **Commit message** references J-NNN milestone-close + D-074 eighth-instance + Q1+Q2+Q3+Q4 locks closed

### §6.8 Anti-drift guardrails for Commit 4

1. **Atomic commit per D-074.** All twelve files in ONE commit. NOT split across two commits per J-098 prose-then-batch lesson + J-099 fix-up atom lesson.
2. **J-NNN freeze must be complete.** `grep -rn 'J-NNN' .` returning any matches after Commit 4 stages is a Rule 4 discipline failure. All freeze sites land in one commit.
3. **No code changes in Commit 4.** Code shipped at Commits 2 + 2a + 3. Commit 4 is purely documentation + status flips. Any code change surfaces a milestone-internal scope drift and escalates to a fresh Joe-lock.
4. **Status flip on this runbook MUST happen here, not earlier.** ACTIVE through Commits 1-3; flips to COMPLETED v1.1 at Commit 4. Premature flip leaves the runbook claiming COMPLETED while implementation is still in flight.
5. **`tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` and `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` are NOT touched at Commit 4** (except possibly the design doc's `Last updated` header for completeness, which is optional). Both already COMPLETED v1.1 from J-105.
6. **Commit message must include J-NNN.** The commit message is the canonical record of the milestone close at the git layer; missing J-NNN reference breaks the cross-reference chain JOURNAL ↔ commit history.
7. **D-074 application count line in JOURNAL entry must use grep-quoted format.** Per topo-sort J-101 precedent: `\`grep -c "D-074" JOURNAL.md\`` returns NN mentions; simple-counting milestone-close applications J-095 … J-NNN = eighth instance at this commit".

---

## §7 — Discipline notes

The runbook concludes with discipline notes that record patterns surfacing during this milestone's design + runbook-authoring phases. Sibling-in-shape to topo-sort runbook §7 (six sub-sections). Precedent-departure self-defense at §7.1 below; six recurrence/pattern notes at §7.2-§7.7. Optional §7.8 skipped per runbook-authoring §7 lock.

### §7.1 Precedent-departure self-defense

**Question**: Why does this runbook have §7 discipline notes? The bidirectional runbook didn't.

**Answer (three grounds, sibling-shape to topo-sort J-098 §7.1)**:

1. **Trilogy-internal consistency.** The persistence-amendment trilogy ships its audit doc with inline discipline notes per section and its design doc with explicit §8 candidate D-NNN flag (the discipline-note layer of the design doc). The implementation runbook being the only document in the trilogy without a discipline-notes section would itself be drift; trilogy-internal consistency outranks one-step-earlier-precedent consistency when they conflict.
2. **The bidirectional+topo-sort precedent of including §7 at runbook layer is now durable enough.** Topo-sort runbook J-098 included §7 self-defending its precedent departure from bidirectional. After J-098, the pattern recurred once (this milestone's runbook). Two recurrences across three sibling-recurrence depth doesn't yet make it standard, but the inclusion-by-default posture is now closer to neutral than precedent-departing.
3. **This milestone's new discipline data points need runbook-visible pointers.** Specifically: (a) layered-B3 second project-wide instance — the pattern emerged at J-101 and now recurs; runbook-visible pointer prevents future audits from missing it; (b) candidate D-NNN flag visibility at the code site — the verbatim code-comment block per Joe-lock checkpoint #4 makes the flag survivable at the touch-site, not buried in JOURNAL only.

Future audit asking "why does this runbook have §7 when bidirectional doesn't?" finds this answer at the head of the section.

### §7.2 Inline-lock pattern fourth recurrence

Four Joe-locks from J-105 design phase (Q1 + Q2 + Q3 + Q4) are treated as already-decided rather than re-walked during runbook authoring. Clair reading the runbook sees implementation steps, not litigation steps.

Pattern recurrence count across audit-design closures:

- **Bidirectional federation_nodes** (J-096 milestone-close): Q2 locked inline at audit phase; Q1 walked at design phase (mixed)
- **Topo-sort design phase J-097**: Q3 + Q2 + Q1 all locked inline at audit phase
- **Topo-sort design-phase re-walk J-099/J-100**: Q4 + Q1 supplement locked inline at re-walk phase
- **This milestone J-105**: Q1 + Q2 + Q3 + Q4 all locked inline at design phase

Four recurrences across three sibling-recurrence depths. The pattern is durable: design phase ships in a fresh session with cleaner exposition because the deliberation phase doesn't double as the artefact-authoring phase. Sibling-shape to D-067 + D-070 + D-075 + D-076 family-completion at the no-drift-surface discipline family level: when a pattern recurs four times across distinct phases, it's a durable discipline rather than a coincidence.

Future audit phases should default to inline-lock posture: Joe locks during audit-doc authoring, design phase expounds the already-locked decisions. Surprises during design phase that genuinely require re-deliberation indicate the audit was incomplete, not that the design phase is mis-scoped.

### §7.3 Sibling-in-shape fourth recurrence

Fourth recurrence of the audit → design → runbook → implementation → milestone-close shape within Federation Event Propagation milestone scope:

- **Phase 7.5** (first — J-093/J-104): originating instance, cold-start bootstrap design
- **Bidirectional federation_nodes** (second — J-096): sibling-in-shape closure
- **Topo-sort** (third — J-097/J-098/J-099/J-100/J-101): sibling-in-shape closure with design-phase re-walk (Shape 2 targeted patch via in-place amendments)
- **This milestone** (fourth — J-104/J-105/this commit): sibling-in-shape closure

Four recurrences make the audit → design → runbook → implementation → milestone-close shape durable. The shape is now the project's default pattern when work surfaces a gap that requires its own audit-design-impl arc. Future contributors looking at "how do we close a new milestone-level scope?" find the four-recurrence pattern as the canonical reference.

The shape's components:
- **Audit doc** at `tasks/[NAME]_AUDIT.md` ACTIVE v1.0 → COMPLETED v1.1 at design close
- **Design doc** at `tasks/[NAME]_DESIGN.md` ACTIVE v1.0 → COMPLETED v1.1 at milestone close (NOT at design-phase close, sibling-shape to topo-sort design task file lifecycle)
- **Runbook** at `tasks/[NAME]_IMPL.md` ACTIVE v1.0 → COMPLETED v1.1 at milestone close
- **Implementation** in atomic commits per the runbook's locked sequence
- **Milestone close** as atomic D-074-instance commit

### §7.4 Layered-B3 second project-wide instance

The **layered-B3 recurrence pattern** is named: a primary fix at the locked layer surfaces a secondary encoding of the same invariant at a sibling-layer, both closed atomically within the same commit per D-067 no-drift-surface discipline.

Project-wide instances:

- **First instance — Topo-sort Commit 2a (J-101)**: primary fix at `build_room_create_event` (xgen-core/src/space/state.rs:797) surfaced secondary surface at `is_dag_root_type` (graph.rs) AND `validate_dag_structure` (exchange.rs) inline matches!-block re-encoding the same DAG-root invariant. Two sibling-layer encodings closed atomically per D-067 Option E unification.
- **Second instance — This milestone (Commit 2 + 2a)**: primary fix at `graph.add_event` Result-handling (xgen-core/src/node/runtime.rs:210) surfaced secondary surface at `let _ = graph.add_event(&event, store);` silent-discard pattern (Q1 narrow-scope holds; the secondary surface here is the Q1 scope itself, not a sibling-layer match!-block). One sibling-layer encoding closed atomically; the four out-of-scope silent-discard sites stay open per design doc §8 future-walk discipline.

**Two instances is not yet a durable pattern; three would be.** The recurrence framing matters because:

1. Future audit phases should look for layered-B3 candidates explicitly during code-trace passes ("is this primary fix the only encoding of this invariant, or does a sibling-layer hold a copy that would drift?")
2. The narrow-scope discipline holds even for layered-B3 cases: closing the layered-B3 pair atomically does NOT mean expanding scope to other potentially-related invariants; the layered B3 is a same-invariant pattern, not a related-invariants pattern.

Third instance would trigger Joe to lock a project-wide audit-phase code-trace step: "check for layered-B3 candidates" becomes a default audit-doc section.

### §7.5 Candidate D-NNN flag visibility

The candidate D-NNN "ingest path invariant encoding" stays flagged-not-promoted at this milestone close per design doc §8 audit-vs-design boundary discipline.

Visibility at three layers:

1. **JOURNAL J-NNN milestone-close entry sub-section 6** — names the flag; references this runbook §4.1 + design doc §8
2. **`xgen-core/src/node/runtime.rs::ingest_event` verbatim code-comment block** (Joe-lock checkpoint #4) — names the flag inline at the touch-site; future-walk authors land here when re-walking
3. **`docs/ROADMAP.md` cross-cutting principles section** — stays 🟡 with reference to JOURNAL J-NNN milestone-close entry (already added at J-105 design-close); no flip at this milestone-close

Future-walk trigger conditions:

- **(a) Dependent work surfaces a concrete drift instance.** M6 admin write path; M8 federation-depth migration tool; future protocol revision introducing async predecessor validation. If a future caller bypasses `validate_event` and hits the silent-discard pattern, the candidate D-NNN walk becomes load-bearing.
- **(b) Joe locks the candidate as worth pursuing on philosophical/strategic grounds.** Independent of a surfacing gap. The walk would close at one of three rungs: ValidatedEvent wrapper, sealed traits + visitor pattern, or formal verification. Each rung has its own implementation cost + protection class.

**The narrow-scope discipline at this milestone holds the candidate D-NNN open without pre-committing to a rung.** Sibling-shape to D-076 v1 → v1.1 progression: v1 didn't pre-promote the second invariant; v1.1 emerged after design walked it properly. The candidate D-NNN walk, if triggered, follows the same shape — audit → design → impl → promotion to D-NNN.

### §7.6 "Honest longer work over fast shortcuts" — count inherited at eighth, not incremented

Recurrence count within Federation Event Propagation milestone scope as of this milestone close:

| # | Recurrence | Surface event | Type |
|---|---|---|---|
| 1 | Phase 7.5 cold-start bootstrap | J-093 design close | Originating instance |
| 2 | Bidirectional federation_nodes | J-096 milestone close | Sibling closure |
| 3 | Topo-sort design close | J-097 design close | Sibling closure |
| 4 | Topo-sort runbook landing | J-098 runbook ship | Companion-updates housekeeping |
| 5 | Topo-sort design-phase re-walk Step 2 | J-099 Step 2 ship | Re-walk recurrence |
| 6 | Topo-sort design-phase re-walk Step 3 | J-100 Step 3 ship | Re-walk recurrence |
| 7 | Topo-sort implementation | J-101 milestone close | Sibling closure |
| 8 | Persistence-amendment surfacing | J-104 audit doc + sentinel-tree authored | Originating instance |

**Eighth recurrence count is inherited from J-104, NOT incremented at:**

- **J-105 design close** (design-event, not milestone-event)
- **This milestone runbook-authoring close** (runbook-event, not milestone-event)
- **This milestone-close commit** (close-event-not-recurrence-event; the recurrence opened at J-104 when the milestone surfaced)

Future recurrences increment at milestone-surface events (when work surfaces a new gap that requires its own audit-design-impl arc), NOT at within-milestone events (design walks, runbook ships, milestone closures).

Ninth recurrence would surface if a new gap surfaces during Phase 9 Commit 3b-2-equivalent or beyond. Pattern continues as long as Federation Event Propagation milestone stays PLAY.

### §7.7 Commit-3b-1-collapse honest framing

Per D-065 honest-behaviour discipline, the Phase 9 Commit 3b-1 numbering effectively skips. The honest framing: Commit 3b-1 IS the persistence-amendment milestone close under a different milestone name. Phase 9 resumes at Commit 3b-2-equivalent after this milestone close.

Why this matters for the canonical record:

1. **CLAUDE.md PLAY block flip at milestone close** acknowledges this explicitly: "Phase 9 Commit 3b-2-equivalent" rather than "Phase 9 Commit 3b". The `-equivalent` suffix flags that Phase 9's commit-numbering convention is preserved (Commit 3a shipped at J-092; Commit 3b would have been Scenarios 2 + 3 + compounds; Commit 3b-1 collapsed into this milestone; Commit 3b-2-equivalent is what would be "Commit 3b" if Commit 3b-1 hadn't collapsed).

2. **JOURNAL J-NNN milestone-close entry sub-section 7** names this explicitly: "Phase 9 Commit 3b-1 numbering effectively skipped (collapsed into this milestone); Phase 9 resumes at Commit 3b-2-equivalent".

3. **ROADMAP.md tree update at Commit 4** flips the Phase 9 row's annotation accordingly. No commit-numbering revisionism; the collapse is the honest framing.

Future readers asking "what happened to Commit 3b-1?" find the answer at three points (CLAUDE.md PLAY block + JOURNAL J-NNN sub-section 7 + ROADMAP tree). The skip is intentional and recorded; not a slip.

Alternative framings considered and rejected:

- **"Renumber Commit 3b to start from 3b-1"** — rejected; the J-104 audit doc already used "Commit 3b-1" in its prose, and the persistence-amendment milestone took that scope.
- **"Treat persistence-amendment milestone as a parenthetical aside"** — rejected; the milestone has its own audit-design-impl arc per D-071, which makes it a first-class milestone, not a parenthetical.
- **"Skip the discussion entirely"** — rejected per D-065 honest framing; silent skip would leave a gap in the canonical record.

The honest framing wins on all three: it preserves the audit-design-impl arc's first-class status; it explains the commit-numbering convention without revisionism; it leaves no gap in the canonical record.

---

## §8 — Cross-references

### §8.1 Audit + design + runbook trilogy

- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` — audit doc, COMPLETED v1.1 at J-105 design close. Body §1–§11 stays authoritative as historical record of audit-at-lock-time.
- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` — design doc, ACTIVE v1.0 at J-105. Flips COMPLETED v1.1 at Commit 4 of this milestone (sibling-shape to topo-sort design task file lifecycle).
- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` — this runbook, ACTIVE v1.0. Flips COMPLETED v1.1 at Commit 4.

### §8.2 DECISIONS.md entries

- **D-065** — honest behaviour over polite behaviour. Applied throughout (§4a.6 persist-failure handling; §7.7 Commit-3b-1-collapse framing; sub-amendment milestone surfacing at J-104).
- **D-067** — single source of truth per logical responsibility. Applied at §4.6 (topological_sort re-export from xgen-core to xgen-node).
- **D-069** — canonical-document discipline + audit-vs-design boundary. Applied at §4.1 (narrow scope per design doc §8 candidate D-NNN flag).
- **D-070** — two events of equal importance, opposite direction. Not directly applied at this milestone (sibling-shape, not used).
- **D-071** — audit-precedes-dependency discipline. Applied at J-104 (sub-amendment milestone opened with audit doc; sibling-shape to topo-sort + bidirectional + Phase 7.5 four-recurrence pattern per §7.3).
- **D-074** — milestone-close commits MUST include JOURNAL.md. Eighth-instance application at Commit 4 per §6.4 sub-section 4 framing.
- **D-075** — state.federation_add is one party's act. Not directly applied at this milestone.
- **D-076 v1.1** — wire-order determinism + causal-DAG-respecting order. Not directly applied at this milestone.
- **Candidate D-NNN "Ingest path invariant encoding"** — flagged at J-105, NOT promoted at this milestone close. Future walk per §7.5.

### §8.3 JOURNAL.md entries

- **J-093** — Phase 7.5 design walkthrough (originating instance of the audit-design-impl arc shape per §7.3 first recurrence)
- **J-094** — (NOT WRITTEN at milestone-event time; retrospective J-103 covers Phase 7.5 implementation; prose-then-batch atomicity-slip family originating instance)
- **J-095** — XGID Adoption v1 implementation milestone close; D-074 first-instance application
- **J-096** — bidirectional federation_nodes milestone close; D-074 second-instance application
- **J-097** — topo-sort design close; D-074 third-instance application
- **J-098** — topo-sort runbook landing; D-074 fourth-instance application (across two commits per honest provenance)
- **J-099** — topo-sort design-phase re-walk Step 2; D-074 fifth-instance application
- **J-100** — topo-sort design-phase re-walk Step 3; D-074 sixth-instance application
- **J-101** — topo-sort milestone close; D-074 seventh-instance application
- **J-102** — JOURNAL Gap 2 closure (XGID retrospective)
- **J-103** — JOURNAL Gap 1 closure (Phase 7.5 retrospective)
- **J-104** — persistence-amendment audit doc shipped (originating instance of THIS milestone per §7.3 fourth recurrence)
- **J-105** — persistence-amendment design close; four Joe-locks recorded
- **J-NNN** — persistence-amendment runbook authoring (this runbook ship); single-file housekeeping atom
- **J-NNN+1** (this milestone's close) — persistence-amendment milestone close; D-074 eighth-instance application

### §8.4 Code surfaces

- **`xgen-core/src/node/runtime.rs`**:
  - `ingest_event` signature change site (§4.2)
  - verbatim code-comment block (§4.3)
  - `DispatchOutcome::Accepted` variant change (§4a.2)
  - three drain helpers signature changes (§4a.3)
  - `dispatch_event` aggregation (§4a.5)
  - `topological_sort` re-export (§4.6)
- **`xgen-core/src/dag/graph.rs`**:
  - `GraphError` visibility widening (§4.4)
- **`xgen-node/src/app.rs`**:
  - `replay_spaces_from_dir` sort-on-replay (§4.6)
  - `process_inbound` persist-loop block (§4a.6)
  - `handle_identity_replicate_msg` persist-loop block (§4a.8 DoD checklist)
- **`xgen-node/src/tests/`**:
  - `phase9_harness.rs` (sentinel-tree refinement §5.2)
  - `phase9_three_node_anti_transitivity.rs` (sentinel-tree refinement §5.2)
  - `phase9_drop_and_recover.rs` (sentinel-tree refinement §5.2 + Scenario 3 transition FAIL → PASS §5.1)
  - `mod.rs` (module declarations)

### §8.5 Appendix references

- **Appendix G** (`docs/xgen_appendix_g_en.md`) — logging convention. Applied at §4.5 (tracing::error for ingest_event failure) + §4.6 (tracing::warn for replay_ingest_failed) + §4a.6 (tracing::warn for persist_drained_event_failed).
- **Appendix C** (`docs/xgen_appendix_c_en.md`) — primitive schemas. Not directly modified at this milestone (sibling-shape to topo-sort milestone: Pass-1-neutral discipline holds).
- **Appendix I** (`docs/xgen_appendix_i_en.md`) — data structures. Not directly modified at this milestone.
- **Appendix J** (`docs/xgen_appendix_j_en.md`) — XGID adoption discipline. Not directly modified at this milestone (Pass-1-neutral discipline holds for the runtime-internal Result-shape change).

### §8.6 Operational state references

- **`CLAUDE.md`** — PLAY block at runbook-authoring time; flips at Commit 4 per §6.5
- **`docs/ROADMAP.md`** — v1.20 at J-105 design close; v1.21 at this milestone close per §6.6
- **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** — header `Last updated` flag at Commit 4 (Commit 3b-1 collapsed; Commit 3b-2-equivalent next-active)
- **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** — new catalogue M16 row at Commit 4 per §6.3
- **`docs/xgen_federation_propagation_design.md`** — §6.4.4 new sibling subsection at Commit 1 per §3.3; §15 row at Commit 1 with J-NNN placeholder per §3.4; placeholder freezes at Commit 4 per §6.2

---

*End of runbook.*  
