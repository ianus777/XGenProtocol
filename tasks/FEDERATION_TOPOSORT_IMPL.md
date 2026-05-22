# Federation Topological-Sort Wire-Order Determinism Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-22 (Commit 1 of 4 — doc-pass — shipped. Three real file edits per pre-Commit-1 Joe-resolved drift: canonical design doc `docs/xgen_federation_propagation_design.md` §6.4.3 sibling subsection added (sibling to §6.4.1 Phase 7.5 + §6.4.2 bidirectional `federation_nodes`; six-paragraph dense prose covering one-paragraph framing, three [JOE-LOCK: locked 2026-05-22]-tagged paragraphs for the three locks Q3.ii + Q2 middle + Q2.γ + Q1 Shape A v1, code-surfaces paragraph, cross-references paragraph, what-this-phase-does-NOT-change paragraph) + §15 Implementation Complete row added between the Bidirectional row and Phase 9 (pending) row with J-NNN+ placeholder pending Commit 4 freeze; `tasks/FEDERATION_TOPOSORT_DESIGN.md` Status flipped ACTIVE → COMPLETED v1.0 + Last updated bumped; this runbook Last updated bumped (Status stays ACTIVE; flips at Commit 4). The audit doc Status flip ACTIVE → COMPLETED v1.0 that runbook §3.2 anticipated was already absorbed into J-097 design-phase close commit (`44fd590`), so Commit 1 ships as three real edits rather than four — drift surfaced and Joe-resolved at pre-Commit-1 ambiguity-check before authoring began. No code touched; test count unchanged from runbook-landing baseline. Pre-Commit-2 Joe-lock checkpoint (§2.3 #2 — unit-test list proposal) is the next surfacing moment. Per D-069 + D-071 + D-074 + D-076 discipline. Previous content (runbook shipped at runbook-authoring milestone close 2026-05-22) stands authoritative as the locked four-commit shape.) Previous 2026-05-22 update: Runbook shipped at runbook-authoring milestone close. Four-commit Clair-facing sequence: Commit 1 doc-pass (audit + design Status flips, canonical design doc §6.4.3 sibling subsection + §15 row); Commit 2 primitive fix (`xgen-node/src/fanout.rs:193` event_id lex sort + verbatim code-comment block) + sibling Site 1 fix (`xgen-node/src/fanout.rs:321` HashMap-feed sort) + three-to-five unit tests including wire-order-determinism witness; Commit 3 Phase 9 Scenario 1 `#[ignore]` lift with 5+ isolated-run verification rigour; Commit 4 milestone close per D-074. Three Joe-lock checkpoints: post-Commit-1 if doc-pass surfaces drift; pre-Commit-2 unit-test list proposal; post-Commit-2 / pre-Commit-3 primitive shape locked. Sibling-in-shape to `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (COMPLETED v1.1). Status flips ACTIVE → COMPLETED in Commit 4 per the bidirectional precedent. Per D-069 + D-071 + D-074 + D-076 discipline.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This is the **implementation runbook** for the topological-sort wire-order determinism phase of the Federation Event Propagation milestone. The design phase closed 2026-05-22 with three Joe-locks captured at `tasks/FEDERATION_TOPOSORT_DESIGN.md` (Status ACTIVE v1.0 — flips COMPLETED in Commit 1 of this runbook): **Q3.ii** (canonical wire ordering required), **Q2 middle + Q2.γ** (fix the primitive's contract once; forward-bind to Node-to-Client siblings), **Q1 Shape A v1 + sibling Site 1 fix** (event_id lex sort at the topo primitive + HashMap-feed sort at `compute_federation_delta_for_space`).

This document is Clair-facing — it sequences code-level work across four atomic commits.

The design task file is authoritative on **what** to build and **why**. This runbook is authoritative on **how** to ship it, **in what order**, with **what verification at each step**. The audit doc (`tasks/FEDERATION_TOPOSORT_AUDIT.md`) is authoritative on the **code-grounded mechanism evidence** — file:line references, the two-site compounding (Site 1 at :321 + Site 2 at :193), the canonical sibling sort at `xgen-core/src/node/runtime.rs:859-912` that is the reference shape.

### 1.1 Reading order on session start

1. This document, §2 (sequence overview) — get the shape of the four commits.
2. Design task file §3 (Q3.ii lock) + §4 (Q2 middle + Q2.γ lock) + §5 (Q1 Shape A v1 lock) — re-read the three locks before touching code.
3. DECISIONS.md D-076 — the protocol-design principle the locks instantiate; fourth member of the no-drift-surface discipline family (D-067 + D-070 + D-075 + D-076).
4. Audit doc `tasks/FEDERATION_TOPOSORT_AUDIT.md` §3 (the mechanism, code-verified) + §3.5 (the canonical sibling sort precedent) — refresh the file:line references for the surfaces touched in Commit 2.
5. Canonical design doc `docs/xgen_federation_propagation_design.md` §6.4 (Phase 7 F-3 framework) — sibling subsections §6.4.1 (Phase 7.5) and §6.4.2 (bidirectional `federation_nodes`) are the precedent shape; §6.4.3 is the new slot that Commit 1 fills for this phase.
6. Then back to this document, §3 onward, for per-commit work.

### 1.2 Latitude

Implementation-internal decisions (test helper shapes, internal function organisation, code-comment phrasing within the verbatim-block guidance below, test fixture builders) are Clair's latitude. Wire-format-visible or correctness-visible decisions require Joe-lock — pause and ask. The three explicit Joe-lock checkpoints in this runbook (§2.3) are non-negotiable; additional pauses for surface ambiguity are encouraged.

Concrete starting suggestions in this runbook (unit test names, code-comment wording within the verbatim-shape block, exact sort-line placement within the outer loop) are exactly that: starting points. Clair may revise if a cleaner option surfaces during implementation, with the constraint that the **vantage of the fix locked at design §5.1 is preserved verbatim**: event_id lexicographic sort applied to ready siblings at each iteration of the outer loop in `topological_sort_events`, plus `Vec<Event>` sort by event_id before the `topological_sort_events` call at `compute_federation_delta_for_space:321`. The semantic of the fix is locked; the surface code shape around it is Clair's call.

### 1.3 Pre-existing flakes carried forward

From CLAUDE.md, two pre-existing workspace-parallelism flakes survive into this milestone: the **precedence env-var race** (D-068, commit `3e2f311`) and **`reconnect_with_existing_tip_small_delta_delivered`** (Phase 3 test under workspace parallelism). If either fires during verification, retry once to confirm the flake signature; do not treat as regression unless it fires consistently (3+ times in 5 runs). This carry-forward is sibling to bidirectional runbook §2's identical disclosure.

---

## 2. Sequence overview

Four atomic commits, in this order. Each commit is shippable in isolation (workspace `cargo test` passes at each step). Hard ordering is documented in design task file §8.2 — Commit 2 must precede Commit 3 (the integration test passes only after the primitive + sibling fix); Commit 1 must precede Commit 2 (the canonical record reflects the locked design before code references it); Commit 4 must be last (milestone-close housekeeping happens after all code has shipped and verified across multiple isolated runs).

### 2.1 The four commits

| # | Commit | Scope | Code? | Test count change |
|---|---|---|---|---|
| 1 | Doc-pass | Canonical design doc §6.4.3 sibling subsection + §15 row; design task file flipped COMPLETED; audit doc flipped COMPLETED | No | unchanged (no code) |
| 2 | Primitive + sibling fix + unit tests | `xgen-node/src/fanout.rs:193` primitive sort + verbatim code-comment block; `xgen-node/src/fanout.rs:321` sibling Site 1 sort; three-to-five unit tests | Yes | +N (new unit tests) |
| 3 | Phase 9 Scenario 1 `#[ignore]` lift | Remove `#[ignore]` from `xgen-node/src/tests/phase9_two_node_smoke.rs::two_node_federation_push_smoke_100_messages`; rewrite doc comment to point at COMPLETED audit + design + this milestone's J-NNN; decide `#[serial_test::serial]` posture; verify with 5+ isolated runs | Yes (annotations + verification) | +1 (one previously-ignored now passing) |
| 4 | Milestone close | CLAUDE.md PLAY block flip; ROADMAP.md state flip (v1.14 → v1.15 expected); JOURNAL.md entry; this runbook Status ACTIVE → COMPLETED; **topological-sort milestone DONE; Phase 9 Commit 3b unblocks → resumes**; catalogue row addition per audit §4.3 | No | unchanged from Commit 3 |

**Test-count discipline.** N is not pre-locked. Design §8.1 names three seed tests (deterministic output across input permutations; stable tie-break for ready siblings with empty `prev_events`; no-op-equivalence for already-canonically-ordered input). The runbook adds the **wire-order-determinism witness** as the load-bearing fourth ("two senders with identical Space history produce byte-identical federation deltas modulo signature-bearing fields") — structural sibling to bidirectional's `apply_federation_add_two_vantages_mirror`. Three-to-five tests total feels right; Clair proposes the final list at the pre-Commit-2 Joe-lock checkpoint (§2.3). Each commit's DoD requires actual `cargo test --workspace` output quoting the new count. Do not invent numbers (CLAUDE.md Rule 5).

### 2.2 Files touched across the four commits

For quick reference. Per-commit detail in §3 through §6.

- **`xgen-node/src/fanout.rs`** — Commit 2. Two named edits at `:193` (primitive) and `:321` (sibling Site 1).
- **`xgen-node/src/tests/phase9_two_node_smoke.rs`** — Commit 3. `#[ignore]` lift on `two_node_federation_push_smoke_100_messages`; doc comment rewrite; `#[serial_test::serial]` posture decision.
- **`docs/xgen_federation_propagation_design.md`** — Commit 1. New §6.4.3 sibling subsection + §15 Implementation Complete row.
- **`tasks/FEDERATION_TOPOSORT_AUDIT.md`** — Commit 1. Status flip ACTIVE → COMPLETED.
- **`tasks/FEDERATION_TOPOSORT_DESIGN.md`** — Commit 1. Status flip ACTIVE → COMPLETED.
- **`tasks/FEDERATION_TOPOSORT_IMPL.md`** (this runbook) — Commit 4. Status flip ACTIVE → COMPLETED.
- **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** — Commit 4. Catalogue row addition per audit §4.3.
- **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** — Commit 4. Header `Last updated` paragraph updated to reflect Commit 3b is now-active (the paused-inside-milestone-scope work resumes).
- **`JOURNAL.md`** — Commit 4. New entry recording the milestone close.
- **`CLAUDE.md`** — Commit 4. PLAY block flip (topological-sort milestone DONE; new PLAY block for Phase 9 Commit 3b as next-active for Clair; Phase 9 and Federation Event Propagation milestone PLAY blocks remain; M6 + Pass 1 PENDING blocks unchanged).
- **`docs/ROADMAP.md`** — Commit 4. Tree (topological-sort cluster Implementation row 🟢 → ✅; Phase 9 resume row 🟡 → 🟢; Federation Event Propagation milestone header stays paused) + Past + Present + version bump (v1.14 → v1.15 expected).

Test files in `xgen-node/src/fanout.rs::mod tests` (if the module has one) or a new `xgen-node/tests/` integration file may host the new unit tests; Clair's call on placement at the pre-Commit-2 checkpoint. Verify the canonical sibling at `xgen-core/src/node/runtime.rs::topological_sort` test placement before deciding — sibling-shape discipline applies here too.

### 2.3 Joe-lock checkpoints

Three explicit checkpoints where Clair pauses and surfaces to Joe before continuing:

1. **Post-Commit-1, if doc-pass surfaces drift.** If Clair's authoring of §6.4.3 in `docs/xgen_federation_propagation_design.md` reveals inconsistency between what the canonical design doc currently states and what the design task file locks (e.g., the canonical doc claims tie-break is implementation-detail when D-076 makes it normative), surface before continuing. Same shape as bidirectional Commit 1's §6.4.2 + §15 row addition. Default: no drift expected, but the canonical design doc is large enough that latent inconsistencies are plausible.
2. **Pre-Commit-2 unit-test list proposal.** Clair proposes the final test list before writing them. Seed three from design §8.1 (deterministic output, stable tie-break, no-op-equivalence). Add the wire-order-determinism witness as the load-bearing fourth. Clair may propose 1-2 more if a coverage gap surfaces during implementation prep. Joe locks the final list (three-to-five) before code is written.
3. **Post-Commit-2 / pre-Commit-3 primitive shape locked.** Before Clair lifts `#[ignore]` from Phase 9 Scenario 1, the primitive's new contract + sibling Site 1 fix are stable: workspace test pass green, unit tests landed, no compile-driven surface ambiguities outstanding. The integration test depends on the primitive shape being final; this checkpoint confirms it is.

Additional pauses for surface ambiguity are encouraged. Per CLAUDE.md Rule 3 (stop and report when tools or expectations diverge from reality), Clair surfacing more often than the three checkpoints above is welcome, not over-careful.

### 2.4 What this milestone CANNOT close

The runbook must be explicit about this because the framing is easy to get wrong: **Commit 4 of this runbook closes the topological-sort milestone only.** It does NOT close Phase 9 (`tasks/FEDERATION_PROPAGATION_PHASE_9.md`, Status ACTIVE v1.0). It does NOT close the Federation Event Propagation milestone. It does NOT unblock M6 (new) or XGID Retrofit Pass 1.

What Commit 4 actually does to the downstream chain:

- **Topological-sort milestone** flips PLAY → DONE. ✅
- **Phase 9 Commit 3b unblocks** (the previously-paused-inside-milestone-scope work resumes). Commit 3b's scope per `tasks/FEDERATION_PROPAGATION_PHASE_9.md` is **Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10** — eight scenarios still to ship in their own multi-commit sequence, plus the Phase 9 milestone-close commit. Phase 9's Q4 Lock locked Phase 9 itself as ~5-7 atomic commits; Commit 3a (Scenario 1 lift) is just one of them, and Commit 3b is substantial work in its own right.
- **Phase 9 milestone** stays PLAY (waiting on its own multi-commit completion). 🟢
- **Federation Event Propagation milestone** stays PLAY (waiting on Phase 9). 🟢
- **M6 (new) + XGID Retrofit Pass 1** stay PENDING (chain extended by one node — this milestone — unchanged in shape from before).

The dependency chain after Commit 4 of this runbook:

```
  topological-sort milestone DONE ✅
         ↓ unblocks
  Phase 9 Commit 3b (Scenarios 2 + 3 + 6 compounds, ~5-7 atomic commits)
         ↓ on completion
  Phase 9 milestone DONE
         ↓ on completion
  Federation Event Propagation milestone DONE
         ↓ on completion
  M6 (new) + XGID Retrofit Pass 1 UNBLOCK simultaneously
```

This mirrors the J-096 "M6 (new) blocking chain extended by one more node, unchanged in shape" discipline that the bidirectional milestone close was careful about. Both ROADMAP.md row flips (at Commit 4 of this runbook) AND CLAUDE.md PLAY block updates must respect this distinction: this milestone flips DONE; Phase 9 and Federation Event Propagation stay PLAY.

Clair will be tempted at Commit 4 housekeeping to over-flip ROADMAP rows or to claim more downstream unblocks than actually happen. Don't. The honest framing is small: one milestone closes, one paused phase resumes, four downstream items remain blocked.

---

## 3. Commit 1 — Doc-pass commit

### 3.1 Scope

Documentation only. No code changes, no test changes. The purpose: make the canonical design doc and the two task-file Status headers reflect the Joe-locked state of the topological-sort phase before any implementation work begins. This is the canonical-document discipline from D-069 + the same-commit discipline from D-074 applied in advance. Sibling-in-shape to bidirectional runbook §3.

### 3.2 Files touched

- `docs/xgen_federation_propagation_design.md` — add §6.4.3 sibling subsection (sibling to §6.4.1 Phase 7.5 + §6.4.2 bidirectional `federation_nodes`); add §15 Implementation Complete table row for the topological-sort phase between the bidirectional row and the Phase 9 placeholder row.
- `tasks/FEDERATION_TOPOSORT_DESIGN.md` — header Status flipped ACTIVE → COMPLETED; Last updated bumped to commit date.
- `tasks/FEDERATION_TOPOSORT_AUDIT.md` — header Status flipped ACTIVE → COMPLETED; Last updated bumped to commit date (audit's role as input to the design phase is over once the design closes; canonical record preserved).
- This runbook (`tasks/FEDERATION_TOPOSORT_IMPL.md`) — header Last updated bumped (Status stays ACTIVE; flips at Commit 4).

### 3.3 §6.4.3 content sketch

The canonical design doc's §6.4 currently covers Phase 7's F-3 framework with Lock A1 (data source: `SpaceState.federation_nodes`), Lock B1 (skip for `state.federation_add`), and Lock B2 (self-establishing tightening deferred). §6.4.1 is Phase 7.5's sibling subsection (cold-start bootstrap closure with P7.5-A through P7.5-D). §6.4.2 is the bidirectional `federation_nodes` sibling (vantage-aware applier; D-075). §6.4.3 is this phase's sibling subsection — same depth, same prose style, same `[JOE-LOCK: locked 2026-05-22]` tag convention on each lock heading.

Verified at runbook authoring: §6.4.3 is the next free sub-slot (grep against `docs/xgen_federation_propagation_design.md` confirms §6.4.1 and §6.4.2 exist; §6.5 follows without an intervening §6.4.3). If by the time Clair authors this commit a different sub-slot has been taken by intervening work, Clair surfaces at the post-Commit-1 Joe-lock checkpoint (§2.3) before writing the subsection.

Required content in §6.4.3:

1. **One-paragraph framing.** This phase closes a separate pre-existing wire-order non-determinism surfaced during Phase 9 Commit 3a Scenario 1's post-bidirectional verification (J-096 Finding 2). The finding: `topological_sort_events` at `xgen-node/src/fanout.rs:193` preserves input-vector order for ready siblings (DAG roots with empty `prev_events`), and its caller `compute_federation_delta_for_space:321` feeds it via `store.values().cloned().collect()` over a `HashMap<String, Event>` with randomized iteration. Two Nodes with identical Space state produce different federation-delta wire orderings ~50% of runs; when `state.room_create` wins the race against `state.space_create`, B's `dispatch_event` Step 1 rejects with "space not found" and the bootstrap chain cascades. Reference the design task file for the full Q1+Q2+Q3 framing and the rejected alternatives B/C/D.
2. **Lock summary.** Three locks: (a) Q3.ii canonical wire ordering required — wire-order determinism is a sender-side normative property; two senders with identical Space history MUST produce byte-identical federation deltas modulo signature-bearing fields; (b) Q2 middle + Q2.γ — fix the primitive's contract once at `topological_sort_events` with explicit forward-binding to Node-to-Client sender output (`collect_sync_history`, `apply_fanout` history-push) flagged for future scheduling; (c) Q1 Shape A v1 + sibling Site 1 fix — event_id lexicographic sort at the primitive applied to ready siblings at each outer-loop iteration, plus sort the `Vec<Event>` at `compute_federation_delta_for_space:321` before passing to the primitive; v1 `&str` sort with code-comment block flagging Pass 3 retype to `EventXgid`; Pass-1-neutral.
3. **Code surfaces.** Name the two file:line edits in Commit 2 (`xgen-node/src/fanout.rs:193` primitive + `xgen-node/src/fanout.rs:321` sibling Site 1). Name the unit-level regression lock (three-to-five unit tests including the wire-order-determinism witness) and the integration-level regression lock (Phase 9 Scenario 1 `#[ignore]` lift at Commit 3 — the same scenario that originally surfaced the finding now becomes its activating regression lock).
4. **Cross-references.** D-076 (the protocol-design principle this phase instantiates; fourth member of the no-drift-surface discipline family alongside D-067 + D-070 + D-075); the design task file (locks + rejected-alternative reasoning); the audit doc (code-grounded mechanism evidence at file:line granularity); this runbook (four-commit Clair sequence).
5. **What this phase does NOT change.** Wire format unchanged. `Event` struct unchanged. `EventStore` container type unchanged. `state.federation_add` content schema unchanged. The fix is purely sender-side serialisation discipline; receivers observe a more canonical wire order but their dispatch logic is untouched. Existing federation deltas on disk (test fixtures, dev-build state files) stay valid — they were never persisted in a wire-canonical form to begin with; the fix produces canonical wire output going forward. Pass-1-neutral. Sibling Q3.ii-analogue sites (`collect_sync_history`, `apply_fanout` history-push) flagged but not touched in this milestone.

Length: six-to-eight paragraphs matching §6.4.2's density. Read §6.4 + §6.4.1 + §6.4.2 first and match their tone (dense prose; `[JOE-LOCK: locked 2026-05-22]` tags after each lock heading; rejected-alternative reasoning preserved in-line).

### 3.4 §15 Implementation Complete table row

§15 currently records Phases 1–8 (J-082 through J-089), Phase 7.5 (J-093 + J-094), Bidirectional `federation_nodes` (J-NNN+ placeholder pattern at bidirectional Commit 1; the J-NNN was J-096 by the time the milestone closed), and a Phase 9 (pending) row. Add a "Topological-sort wire-order determinism" row in chronological position between the Bidirectional row and the Phase 9 (pending) row.

Format matching existing rows: phase identifier | JOURNAL reference | Headline shipped (dense prose, ~400-500 words sibling to the bidirectional row).

The JOURNAL reference at Commit 1 time will be `J-NNN+` placeholder (sibling to the bidirectional Commit 1 pattern). The actual J-number (J-098 expected at runbook-landing for Chat-Claude's small entry; a separate J-NNN for Clair's Commit 4 milestone-close entry) lands when Commit 4 ships and the milestone-close JOURNAL entry is written. Clair's Commit 4 updates the placeholder to the actual J-number.

**Note on placeholder convention.** The `J-NNN+` form is bidirectional-Commit-1 precedent for canonical-design-doc table rows where the J-number is not yet known at authoring time; the `+` reads as "this row's J-number, plus any J-numbers that land between authoring and freeze." This is shape-equivalent to the bare `J-NNN` placeholders used at the doc-comment in `phase9_two_node_smoke.rs` (§5.5) and the catalogue M15 row (§6.3); all three resolve to the same single milestone-close J-number at Commit 4 freeze (§1 critical-context point 5). Clair freezes all three sites together in Commit 4 — the `+` and the bare form collapse to one J-number across all three.

Headline content sketch (Clair writes the prose at Commit 1; full prose lands at Commit 4 placeholder-update once the milestone-close J-number is known):

> Sibling to Phase 7.5 and Bidirectional `federation_nodes`: closes a separate pre-existing wire-order non-determinism surfaced by Phase 9 Commit 3a Scenario 1's post-bidirectional verification (J-096 Finding 2). `topological_sort_events` at `xgen-node/src/fanout.rs:193` preserved input-vector order for ready siblings; its caller `compute_federation_delta_for_space:321` fed it via `HashMap.values().cloned().collect()` with randomized iteration. Two senders with identical Space state produced different federation-delta wire orderings ~50% of runs. Three Joe-locks per `tasks/FEDERATION_TOPOSORT_DESIGN.md` (COMPLETED v1.0): Q3.ii canonical wire ordering required, Q2 middle + Q2.γ (fix the primitive once; forward-bind to Node-to-Client siblings), Q1 Shape A v1 + sibling Site 1 fix (event_id lex sort at the primitive + sort `Vec<Event>` before passing). **D-076 promoted to DECISIONS.md** as the protocol-design principle the locks instantiate; fourth member of the no-drift-surface discipline family (D-067 code-organisation + D-070 transport-layer + D-075 event-model + D-076 wire-format). Four atomic commits per `tasks/FEDERATION_TOPOSORT_IMPL.md`: doc-pass → primitive + sibling fix + unit tests → Phase 9 Scenario 1 resurrection (`#[ignore]` lifted a second time; same scenario that surfaced the finding becomes its activating regression lock) → milestone close. Wire-format-neutral, Pass-1-neutral. Test count baseline 577 (post-J-096 bidirectional milestone close) + N new unit tests + 1 resurrected scenario = 578 + N at milestone close. Phase 9 Commit 3b unblocks at milestone close (Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10); Federation Event Propagation milestone stays PLAY until Phase 9 completes.

Length target: dense prose ~400-500 words matching the bidirectional row. Clair drafts at Commit 1 with placeholder J-NNN; final prose freezes at Commit 4 placeholder-update.

### 3.5 Audit doc Status flip rationale

The audit doc has been ACTIVE since 2026-05-22 because it was the input to the design phase (the design task file's Pass 1 input). With the design phase closed (2026-05-22) and this implementation runbook ACTIVE (also 2026-05-22), the audit doc's role transitions from "active input" to "historical canonical record of the finding." Status flips ACTIVE → COMPLETED in this commit; content unchanged.

This mirrors the sibling pattern: `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` flipped ACTIVE → COMPLETED at bidirectional Commit 1; `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` flipped ACTIVE → COMPLETED at Phase 7.5's implementation runbook Commit 1. The flip belongs in the implementation runbook's first commit because that's when the design's role as "input" ends.

### 3.6 Design task file Status flip rationale

Same pattern as §3.5 applied to the design task file (`tasks/FEDERATION_TOPOSORT_DESIGN.md`, Status ACTIVE v1.0). The design task file's role as "input to the implementation runbook" ends as the runbook becomes ACTIVE. Status flips ACTIVE → COMPLETED in this commit per the bidirectional precedent (`tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` Status flip at bidirectional Commit 1).

Both Status flips (audit + design) land in the same Commit 1 because both documents play the same lifecycle role and reach the same lifecycle transition at the same moment.

### 3.7 DoD for Commit 1

- [ ] §6.4.3 added to `docs/xgen_federation_propagation_design.md`, content per §3.3 above; sibling-shape to §6.4.1 + §6.4.2; `[JOE-LOCK: locked 2026-05-22]` tags on each lock heading.
- [ ] §15 Implementation Complete row added in chronological position between the Bidirectional row and the Phase 9 (pending) row; placeholder J-NNN+ until Commit 4 placeholder-update.
- [ ] `tasks/FEDERATION_TOPOSORT_DESIGN.md` header Status flipped ACTIVE → COMPLETED; Last updated bumped.
- [ ] `tasks/FEDERATION_TOPOSORT_AUDIT.md` header Status flipped ACTIVE → COMPLETED; Last updated bumped.
- [ ] This runbook's header Last updated bumped (Status stays ACTIVE).
- [ ] `cargo test --workspace` passes with unchanged test count from the runbook-landing baseline (no code touched).
- [ ] Commit message names this as "Commit 1 of 4 — doc-pass for topological-sort wire-order determinism."
- [ ] No JOURNAL.md edit in this commit (Commit 4 carries the JOURNAL milestone-close entry per D-074).
- [ ] If §6.4.3 slot conflict or §15 row position drift is observed during authoring (e.g., intervening work changed the canonical design doc structure between runbook-landing and Commit 1 start), Clair surfaces at the post-Commit-1 Joe-lock checkpoint (§2.3) before continuing.

---

## 4. Commit 2 — Primitive + sibling fix + unit tests

### 4.1 Scope

The fix itself. Implements Q3.ii + Q2 middle + Q1 Shape A v1 in code: event_id lexicographic sort applied to ready siblings at each outer-loop iteration of `topological_sort_events` (the primitive fix at `xgen-node/src/fanout.rs:193`), plus sort the `Vec<Event>` before passing to the primitive at `compute_federation_delta_for_space:321` (the sibling Site 1 fix). Verbatim code-comment block at the primitive sort site citing D-076 + Appendix J's content-hash framing. Three-to-five unit tests including the wire-order-determinism witness.

This is the substantive commit. Commit 1 was preparation; Commit 3 is annotation + verification; Commit 4 is housekeeping. The bug closes here.

### 4.2 Files touched

- `xgen-node/src/fanout.rs` — two named edits at `:193` (primitive sort + verbatim code-comment block) and `:321` (sibling Site 1 sort).
- **Test file** — Clair's call at the pre-Commit-2 Joe-lock checkpoint (§2.3 #2). Either a new `#[cfg(test)] mod tests` block at the bottom of `xgen-node/src/fanout.rs`, or a new `xgen-node/tests/topological_sort.rs` integration-file. Verify the canonical sibling at `xgen-core/src/node/runtime.rs::topological_sort` test placement before deciding — sibling-shape discipline applies (§2.2 nudge).

That's it. No other files. The fix is deliberately small (one-line primitive sort + one-line sibling sort + code-comment block + unit tests). Compilation-driven enumeration is not needed because the function signatures don't change — only the function bodies change. This is unlike bidirectional Commit 2, which added a `my_node_id: &str` parameter that cascaded through call sites; this milestone's fix is purely internal-to-function and surfaces zero compile errors at unrelated call sites.

### 4.3 `topological_sort_events` — the primitive fix at `:193`

Pre-Commit-2 code (`xgen-node/src/fanout.rs:193-220`, per audit §3.2):

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

Post-Commit-2 code (target):

```rust
pub fn topological_sort_events(mut events: Vec<Event>) -> Vec<Event> {
    let mut emitted: HashSet<String> = HashSet::new();
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut changed = true;
    while !events.is_empty() && changed {
        changed = false;

        // D-076 wire-order determinism (locked at topological-sort
        // design-phase close 2026-05-22; sibling-distinct from D-067
        // at code-organisation layer and D-075 at event-model layer;
        // all four lock no-drift-surface properties explicitly across
        // four protocol layers — D-067 + D-070 + D-075 + D-076).
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
        events.sort_by(|a, b| a.event_id.cmp(&b.event_id));

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

**The change is minimal:** one sort line added at the top of each outer-loop iteration, immediately after the `changed = false;` reset. The sort sees the current remaining-events vector (which shrinks as events are emitted), so it canonicalises the iteration order of ready siblings without re-canonicalising already-emitted entries.

**Sort key.** `Option<String>` comparison via `cmp` follows Rust's standard ordering: `None < Some(_)`, then lexicographic for `Some(_)`. In current production all `state.*` and `message.text` events carry `Some(event_id)` (content-hash-derived per Appendix J); the `None` case is structurally unreachable for events that have been ingested into the store. If `None` does appear in future EventType extensions, lex-by-`Option` still produces a canonical ordering — just with `None` values floating to the front.

**Placement at top of outer loop, not inner.** The sort canonicalises *ready siblings* at each iteration. Placing the sort inside the inner `while i < events.len()` loop would re-sort on every event removal, which is O(n² log n) worst-case and accomplishes nothing semantically (intra-iteration order is irrelevant; only the order in which ready events are extracted matters). Top of outer loop is the right home.

**Code-comment block is mandatory.** The verbatim block above is locked content per design §5.3 — phrasing may be edited for clarity, but the structural elements must be preserved: (a) D-076 reference, (b) sibling-distinct framing against D-067 + D-070 + D-075, (c) `&str` sort with Pass 3 retype marker, (d) content-hash-derived event_id citation pointing at Appendix J, (e) D-076's contract statement quoted. The block is load-bearing for future readers tracing the sort line back to the principle — a future contributor reading `events.sort_by(|a, b| a.event_id.cmp(&b.event_id))` in isolation would have no idea this is a wire-format-normative invariant rather than implementation latitude.

### 4.4 `compute_federation_delta_for_space` — sibling Site 1 fix at `:321`

Pre-Commit-2 code (`xgen-node/src/fanout.rs:311-333`, per audit §3.1):

```rust
let store = match rt.stores.get(space_id) {
    Some(s) => s,
    None => return Vec::new(),
};
let all: Vec<Event> = store.values().cloned().collect();   // line ~321
drop(rt);
let sorted = topological_sort_events(all);
```

Post-Commit-2 code (target):

```rust
let store = match rt.stores.get(space_id) {
    Some(s) => s,
    None => return Vec::new(),
};
// D-076 belt-and-braces: sort the HashMap-iteration vector by event_id
// before passing to topological_sort_events. The primitive itself
// canonicalises ready-sibling order (line ~193); this sort ensures the
// feed into the primitive is also canonical so the end-to-end
// federation-delta computation is Q3.ii-compliant from store-read to
// wire-emit. Per design task file §4.1 (Q2 middle's letter: "primitive
// fixed + feed canonical").
let mut all: Vec<Event> = store.values().cloned().collect();
all.sort_by(|a, b| a.event_id.cmp(&b.event_id));
drop(rt);
let sorted = topological_sort_events(all);
```

**Why both sorts.** The primitive sort at `:193` is load-bearing for correctness (it's the layer that turns "causality-preserving but order-non-deterministic" into "canonical given a fixed event set"). The sibling sort at `:321` is belt-and-braces — the primitive's contract under Q2 middle is "canonical regardless of input," so feeding it sorted input is structurally redundant. But the sibling sort makes the canonical-end-to-end property explicit at the code-organisation layer; a future reader of `compute_federation_delta_for_space` sees "sort, then sort" and understands the path is canonical at every step. Q2 middle's letter wants "primitive fixed AND feed canonical"; this is what that looks like in code.

**Alternative considered and rejected:** drop the sibling sort and rely on the primitive alone. Rejected because (a) the sibling fix is one line for negligible cost, (b) the explicit-canonicality benefit at the code-organisation layer is real (D-076 + D-067 sibling principle reinforcement), and (c) the audit explicitly named the HashMap-feed as Site 1 of the compounding bug; closing it at the site of origin is honest provenance.

**Mutability change.** `let all: Vec<Event>` becomes `let mut all: Vec<Event>` to permit the in-place sort. Trivial change; no semantic impact.

### 4.5 Unit tests — the regression lock at unit level

Design §8.1 names three seed tests (deterministic output across input permutations; stable tie-break for ready siblings with empty `prev_events`; no-op-equivalence for already-canonically-ordered input). The runbook adds the **wire-order-determinism witness** as the load-bearing fourth. Clair may propose 1-2 more if a coverage gap surfaces during implementation prep. Final list locks at the pre-Commit-2 Joe-lock checkpoint (§2.3 #2) before code is written.

The four seed tests are detailed below. **Names are starting suggestions; Clair's latitude on final naming with the constraint that the wire-order-determinism witness name be transparent about its role as the load-bearing regression lock.**

**Test 1 — `topological_sort_events_deterministic_across_permutations`.** Setup: construct a small DAG of 5-10 events with mixed `prev_events` shapes (some roots with `vec![]`, some with single predecessors, some with multiple predecessors). Generate all permutations of the input vector (or a representative sample if 5! = 120 is too many — Clair's call). Apply `topological_sort_events` to each permutation. Assertion: all outputs are byte-identical (same event_id sequence). **This is the primitive-level statement of Q3.ii.**

**Test 2 — `topological_sort_events_stable_tiebreak_with_empty_prev_events`.** Setup: construct two events with `prev_events: vec![]` (DAG roots) and lexicographically-comparable event_ids (e.g., `event_id = "event_A..."` and `event_id = "event_B..."`). Apply `topological_sort_events` to `[B, A]` (reverse-lex input order). Assertion: output is `[A, B]` (lex order). **This is the regression lock for the specific bug** — pre-fix code with input `[B, A]` would output `[B, A]` (input order preserved); post-fix code outputs `[A, B]` regardless of input order.

**Test 3 — `topological_sort_events_noop_for_canonically_ordered_input`.** Setup: construct a small DAG already in canonical (lex-by-event_id-with-prev_events-respected) order. Apply `topological_sort_events`. Assertion: output is byte-identical to input. **Closes the contract from the other direction** — the sort canonicalises non-canonical input but doesn't perturb canonical input. Equivalent to a fixed-point property.

**Test 4 — `compute_federation_delta_byte_identical_across_two_senders`.** **The wire-order-determinism witness. Load-bearing regression lock for D-076.** Setup: construct two `NodeRuntime` instances (A and B) with **identical Space state** — the same set of events ingested in the same logical order, producing the same `SpaceState.federation_nodes` and the same set of events in each `EventStore`. Call `compute_federation_delta_for_space` on both A and B's runtimes for the same `space_id`. Assertion: the two returned `Vec<Event>` sequences are **byte-identical modulo signature-bearing fields that vary by author and time** — same event_id sequence, same wire-level ordering of every event. **This is the unit-level statement of D-076's full contract:** "two senders with identical Space history produce byte-identical federation deltas." Structural sibling to bidirectional's `apply_federation_add_two_vantages_mirror` (which made D-075's contract unit-testable).

**Test 4 mechanics.** The phrase "identical Space state" needs care because A and B's `NodeRuntime.node_id` differ by construction (different keypairs). The test fixture should construct A and B with the same set of events in their `EventStore` for the target Space (use the same event-construction helpers; ingest the same set in the same logical order). Signature-bearing fields (`signature`, possibly `timestamp` depending on test fixture choices) are excluded from the byte-identical assertion via field-by-field comparison or a `canonical_minus_signature_fields` helper. If `timestamp` is in-scope at test fixture time (events built with `now()`), use a fixed-clock helper to make timestamps deterministic. The HashMap-iteration randomness in `EventStore` is what the test stresses — if HashMap-iteration order leaks into the federation-delta output, the assertion fails. Run the test multiple times within one `cargo test` invocation if needed to amplify the HashMap-iteration variance (per audit §2.1's verification discipline).

**Test 4 placement consideration.** Test 4 may need integration-test placement (`xgen-node/tests/topological_sort.rs` or similar) because constructing two `NodeRuntime` instances is integration-test-shaped rather than unit-test-shaped. The other three tests are unit-test-shaped and live in `xgen-node/src/fanout.rs::mod tests`. Clair's call at the pre-Commit-2 checkpoint; sibling-shape against `xgen-core/src/node/runtime.rs::topological_sort` test placement.

**Optional Test 5 — `topological_sort_events_n_way_tie_break`.** Suggested but not required. Setup: construct N events (N ≥ 3) with empty `prev_events` and lex-comparable event_ids. Apply `topological_sort_events` to a randomised input ordering. Assertion: output is lex-ordered. Generalises Test 2 to N-way ties; useful if a future EventType extension produces 3+ DAG roots in a single delta (which the current protocol does not, but might under future extensions).

**Optional Test 6 — `topological_sort_events_stable_across_hashmap_iteration_variance`.** Suggested but not required. Setup: construct a fresh `HashMap<String, Event>`, insert events in two different orders (e.g., insert by sorted event_id, then insert by reverse-sorted event_id), and apply the full `compute_federation_delta_for_space` path to each. Assertion: both outputs byte-identical. Tests the sibling Site 1 fix in isolation from the primitive fix — if only the primitive fix existed, Test 6 would still pass (because the primitive canonicalises ready-sibling order); if only the sibling Site 1 fix existed, Test 6 would also still pass (because the feed is canonical). The test confirms that BOTH fixes together produce robust canonicalisation against HashMap-iteration variance — the explicit-canonicality posture D-076 takes.

### 4.6 DoD for Commit 2

- [ ] `topological_sort_events` at `xgen-node/src/fanout.rs:193` gains `events.sort_by(|a, b| a.event_id.cmp(&b.event_id));` at the top of each outer-loop iteration (after the `changed = false;` reset).
- [ ] Verbatim code-comment block at the sort site per §4.3 above; structural elements preserved (D-076 reference, sibling-distinct framing, Pass 3 retype marker, Appendix J citation, D-076 contract statement).
- [ ] `compute_federation_delta_for_space` at `xgen-node/src/fanout.rs:321` gains `all.sort_by(|a, b| a.event_id.cmp(&b.event_id));` after the HashMap-values collection, before the `topological_sort_events` call; `let all` becomes `let mut all`.
- [ ] Brief code-comment block at the sibling Site 1 sort site per §4.4 above; structural elements preserved (D-076 belt-and-braces framing, design task file §4.1 cross-reference).
- [ ] Three-to-five unit tests landed per §4.5 above; named per the suggestions or Clair's revised names; **wire-order-determinism witness (Test 4) is non-negotiable** — it is the load-bearing regression lock for D-076.
- [ ] Test list locked at pre-Commit-2 Joe-lock checkpoint (§2.3 #2) before writing tests.
- [ ] `cargo test --workspace` passes; quote actual output with new test count.
- [ ] No regressions in the existing baseline (any test that fails was relying on the bug's wire-order behaviour — investigate and fix in the same commit; do not silently skip).
- [ ] Two pre-existing flakes (§1.3) may fire; retry once to confirm flake signature; do not treat as regression unless consistent.
- [ ] Commit message names this as "Commit 2 of 4 — primitive + sibling fix + unit tests for topological-sort wire-order determinism."
- [ ] No `#[ignore]` lift on Phase 9 Scenario 1 in this commit (Commit 3's job).
- [ ] No JOURNAL.md edit in this commit (Commit 4's job per D-074).

### 4.7 What NOT to do in Commit 2

- **Do not lift `#[ignore]` from Phase 9 Scenario 1.** That's Commit 3's job. Lifting it here mixes scopes and makes the post-Commit-2 / pre-Commit-3 primitive-shape-locked checkpoint (§2.3 #3) less crisp.
- **Do not write the JOURNAL milestone-close entry yet.** Per CLAUDE.md Rule 4, JOURNAL.md is written last (Commit 4). If Clair is tempted to draft the entry during Commit 2 because the work is fresh, save it as a local note for Commit 4 instead.
- **Do not retype `event_id: Option<String>` to `EventXgid`.** Pass 1 work is not in scope. The `&str` sort at v1 is deliberate per design §5.1 (Pass-1-neutral; preserves Pass 1's Status ACTIVE v2.0 unchanged). When Pass 3 widens dispatch to XGID flavours, the sort key widens naturally; v1 keeps it at `&str` for surface neutrality.
- **Do not touch the `EventStore` container type.** Shape D.1 was rejected at design close (§6.3 of the design task file). The store stays `HashMap<String, Event>`. The fix is at the iteration-output layer (canonical sort before use), not at the storage layer (canonical container).
- **Do not refactor `topological_sort_events` to use Kahn's algorithm.** The canonical sibling at `xgen-core/src/node/runtime.rs:859-912` uses Kahn's; the current `xgen-node/src/fanout.rs` primitive uses the single-pass scan. The design phase deliberately did NOT consolidate the two implementations (audit §3.5; design §5.4 scope statement). Consolidation would be a separate D-067-flavoured audit phase if ever scheduled. This milestone's scope is tie-break behaviour, not algorithm replacement.
- **Do not fix `collect_sync_history` or `apply_fanout` history-push.** Both are flagged as Q3.ii-analogues per Q2.γ forward-binding but are explicitly out-of-scope this milestone (design §4.1 + §8.1). They get their own design discussion when scheduling allows.
- **Do not change the function signature of `topological_sort_events` or `compute_federation_delta_for_space`.** Both signatures stay identical to pre-Commit-2. The fix is purely internal-to-function. Signature changes would cascade through call sites and are explicitly avoided by the Shape A v1 lock.

---

## 5. Commit 3 — Phase 9 Scenario 1 `#[ignore]` lift

### 5.1 Scope

`#[ignore]` removed from `xgen-node/src/tests/phase9_two_node_smoke.rs::two_node_federation_push_smoke_100_messages`. Doc-comment rewritten to honest forward-looking shape per §5.5. `#[serial_test::serial]` posture decision per §5.4. Verification rigour per §5.3.

The scenario, originally authored during Phase 9 Commit 3a as the bidirectional-fix regression witness, lifted once during bidirectional Commit 3, re-stood-down during bidirectional Commit 4 on the topological-sort finding, now lifts for the second time at this commit. From this commit forward it is the activating regression lock at integration level for **both** D-075 (bidirectional vantage-aware applier) and D-076 (topological-sort wire-order determinism). The unit-level regression locks are Commit 2's Tests 1-4.

### 5.2 Files touched

- `xgen-node/src/tests/phase9_two_node_smoke.rs` — remove the `#[ignore]` attribute from `two_node_federation_push_smoke_100_messages`; rewrite the doc comment per §5.5; decide `#[serial_test::serial]` posture per §5.4.

That's it. One annotation removed; one doc comment edited; possibly one `#[serial_test::serial]` decision. No other code touched.

### 5.3 Verification rigour

This subsection exists because the bidirectional milestone learned the lesson the hard way: Scenario 1's first verification attempt at bidirectional Commit 3 passed in isolation, then exposed a separate flake structure when re-run under workspace parallelism, which surfaced both the in-flight drain-hook gap (closed in bidirectional Commit 2.5) and ultimately the topological-sort finding (J-096 Finding 2). One green run is not sufficient evidence the scenario holds. Bake the rigour in so Clair doesn't shortcut.

**Minimum verification:**

1. **5 isolated single-test runs**, each preceded by `cargo clean` to neutralise OS-level state drift (file handles, port reservations, async runtime residue per J-096 §2.1's four-check hypothesis sequence). Command shape post-`#[ignore]`-lift:
   ```
   cargo clean
   cargo test -p xgen-node --lib tests::phase9_two_node_smoke::two_node_federation_push_smoke_100_messages -- --nocapture
   ```
   All 5 runs must pass. If any single run fails, **stop and report per §5.7 — do not ship the lift.**

2. **3 workspace-parallel runs**, each via `cargo test --workspace`. Workspace parallelism is the surface that exposed the topological-sort finding in the first place; if the fix doesn't hold under parallelism, the scenario is still broken. All 3 runs must pass. If any single run fails, stop and report per §5.7.

3. **Optional but recommended: post-reboot bonus confidence-check.** If any flake suspicion arises during steps 1 or 2 (one of the 8 runs above hit a transient that's hard to characterise), reboot the machine and re-run steps 1 + 2 to rule out residual OS-level state. This is the J-096 §2.1 hypothesis-2 maneuver applied prophylactically. Cheap insurance; do not skip if any suspicion exists.

4. **Both pre-existing flakes (§1.3) may fire during workspace runs.** Retry once to confirm flake signature; do not treat as regression unless consistent (3+ times in 5 runs). Document any flake activity in the commit message.

**Total minimum: 5 isolated + 3 workspace = 8 green runs before lifting `#[ignore]` is considered safe.** This is meaningfully more rigour than "one green run = done"; it is the rigour J-096's diagnostic walk earned through cost. Sibling-shape to bidirectional milestone's verification protocol but tightened on the J-096-learned fronts.

**Quote actual output in the commit message.** All 8 runs' pass/fail lines. Per CLAUDE.md Rule 5, do not invent or paraphrase. If Clair wants to save the wall-clock cost of pasting 8 runs' output verbatim, the commit message can summarise ("5/5 isolated pass, 3/3 workspace pass") but at least one of each shape must be quoted in full.

### 5.4 `#[serial_test::serial]` posture decision

The annotation was added at bidirectional Commit 3 as workspace-parallelism precaution (per J-096's Commit 3 sub-entry). Whether it stays post-fix is a real decision worth surfacing rather than silently keeping.

**Default: keep `#[serial_test::serial]`.** No defence required, no commit-message note required, no additional verification beyond §5.3. The cost of keeping it is small (the scenario serialises against itself and a few other annotated tests under `cargo test --workspace`); the benefit is cheap insurance against unrelated parallelism races that have nothing to do with the topological-sort fix. Default-keep is the silent-default posture.

**Alternative: remove `#[serial_test::serial]`.** Requires all three of the following, in the same commit:

1. **Commit-message justification.** Explicit reasoning why the annotation is no longer needed ("the topological-sort fix closed the bug that the annotation was prophylactically guarding against; the fix renders the annotation redundant" is the only plausible justification shape, but it must be stated explicitly).
2. **5 isolated parallel-workspace runs with `cargo clean` between**, per §5.3 step 2 verification format but doubled in count. All 5 must pass. Failure of any single run is a Joe-pushback moment, not a flake retry.
3. **Documented evidence in the commit message** that no flake reintroduces under the doubled-run regime. Specifically: paste the full output of all 5 workspace runs, not a summary. The asymmetry is deliberate — keeping is free, removing requires the receipts.

The asymmetry is the point: keeping is the silent-default; removing is a posture decision that surfaces evidence. Clair removing the annotation without the three requirements above is a Joe-pushback moment. Clair keeping the annotation requires no further action.

**Joe-lock note:** the decision lands as part of Commit 3, not as a separate Joe-lock checkpoint. §2.3 #3 (post-Commit-2 / pre-Commit-3 primitive shape locked) covers the broader "is the fix stable" question; §5.4 covers the more specific "is the test annotation still needed" question. Clair surfaces the annotation decision at Commit 3 commit-message time, not at the §2.3 #3 checkpoint.

### 5.5 Doc-comment rewrite — exact target text

The doc-comment on `two_node_federation_push_smoke_100_messages` has been touched twice now (Commit 3a stand-down + Commit 4 of bidirectional milestone re-stand-down). This commit is the third rewrite and should produce clean forward-looking text that survives future surface changes without yet-another rewrite. Exact target:

```rust
/// Phase 9 Scenario 1 — two-node federation push smoke, 100 messages.
///
/// Originally `#[ignore]`'d during Commit 3a (J-092 sub-entry) on the
/// bidirectional `federation_nodes` finding. Lifted in Commit 3 of the
/// bidirectional implementation runbook (J-096). Re-`#[ignore]`'d in
/// Commit 4 of the bidirectional milestone close on the topological-sort
/// wire-order non-determinism finding (J-096; forward-referenced
/// `tasks/FEDERATION_TOPOSORT_AUDIT.md` placeholder at that point).
///
/// This commit (J-NNN milestone close, topological-sort implementation)
/// lifts the `#[ignore]` for the second time. The scenario is now the
/// activating regression lock for both the bidirectional federation_nodes
/// fix (D-075) and the topological-sort wire-order determinism fix (D-076).
/// If either fix regresses, this test fails.
```

**J-NNN substitution.** Replace `J-NNN` with the actual milestone-close J-number when Commit 4 ships. Working estimate is J-098-or-later depending on what lands between runbook-landing (J-098 expected at this session's Step 7) and Commit 4. Clair updates the placeholder at Commit 4 commit-authoring time once the J-number is known. The placeholder pattern matches the canonical-design-doc §15 row J-NNN+ pattern from §3.4.

**Why this is the third and final rewrite.** The doc-comment is the single source of truth for Scenario 1's history. The exact-target text covers the chronology (4 milestone events: original stand-down, bidirectional Commit 3 lift, bidirectional Commit 4 re-stand-down, this commit's lift), names both decisions the scenario locks (D-075 + D-076), and states the future-failure-mode crisply ("if either fix regresses, this test fails"). Future regressions don't require further chronology updates — they require fixing the regression. Future fixes that don't regress these don't require touching the doc-comment.

### 5.6 What to do if the scenario fails post-Commit-2

Per CLAUDE.md Rule 3 (stop and report when a tool fails) and Rule 7 (DoD is a checklist, not a formality):

1. **Stop immediately.** Do not lift `#[ignore]` if any of the 8 verification runs fail — that ships a broken test.
2. **Report the failure to Joe with actual output.** Paste the failing test's output verbatim; do not paraphrase. Quote the run number ("isolated run 3 of 5" or "workspace run 2 of 3") so Joe knows where in the verification sequence the failure occurred.
3. **Diagnose what Commit 2 missed.** Likely shapes: (a) a third site where HashMap-iteration order leaks that the audit didn't surface (Commit 2's sibling Site 1 fix should have caught Site 1; if Site 2 surfaces, it's a new finding); (b) a test fixture in Scenario 1 that depends on a specific event-emit order which the new canonical order changes (the test's assertions may need adjustment, but ONLY after Joe confirms the new canonical order is correct and the old test assumption was wrong); (c) a non-fanout code path that produces events in non-canonical order downstream of the fix (would be a new Q3.ii-analogue site).
4. **Fix Commit 2's gap.** Either amend Commit 2 (if not yet pushed) or land a Commit 2.5 with the gap closure. Do not silently fold the fix into Commit 3 — atomicity matters for `git log` readability. Bidirectional Commit 2.5 is the precedent.

### 5.7 DoD for Commit 3

- [ ] `#[ignore]` attribute removed from `two_node_federation_push_smoke_100_messages` in `xgen-node/src/tests/phase9_two_node_smoke.rs`.
- [ ] Doc comment rewritten per §5.5 exact target text; J-NNN placeholder in place until Commit 4 J-number freeze.
- [ ] `#[serial_test::serial]` posture decision made per §5.4 (default-keep silent; remove requires commit-message justification + 5 workspace runs + documented evidence).
- [ ] **5 isolated runs with `cargo clean` between, all pass**, per §5.3 step 1.
- [ ] **3 workspace-parallel runs, all pass**, per §5.3 step 2.
- [ ] Post-reboot bonus check performed if any flake suspicion arose, per §5.3 step 3.
- [ ] Pre-existing flake (§1.3) activity documented if any fired.
- [ ] Actual `cargo test` output quoted in commit message per Rule 5; at minimum one full isolated run + one full workspace run, plus summary lines for the others.
- [ ] Test count is +1 against Commit 2's count (Scenario 1 now passing). Quote actual workspace test count.
- [ ] Commit message names this as "Commit 3 of 4 — Phase 9 Scenario 1 second `#[ignore]` lift; activating integration-level regression lock for D-075 + D-076."
- [ ] No JOURNAL.md edit in this commit (Commit 4 carries the milestone-close JOURNAL entry per D-074).

---

## 6. Commit 4 — Milestone close

### 6.1 Scope

Cross-doc state-flip housekeeping per D-074. No code, no tests. The work is "make the canonical record reflect that this milestone is done" — and **only** this milestone (per §2.4 framing: Phase 9 Commit 3b unblocks, Federation Event Propagation milestone stays PLAY).

Sibling-shape to bidirectional runbook §6 with one structural addition: this milestone has a catalogue row to add to `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` per audit §4.3 — the M15 entry.

### 6.2 Files touched (six)

Six files in one atomic commit per D-074. JOURNAL.md MUST be in the changed-files list — this is the D-074 discipline check.

1. **`JOURNAL.md`** — new entry (next available J-number; J-098 expected if no intervening entries, J-099+ if any land between this session's runbook-landing and Clair's milestone close). Per D-074, same-commit JOURNAL entry; do not defer to a separate retrospective commit. Entry shape per §6.4 below.
2. **`CLAUDE.md`** — PLAY block flip: topological-sort milestone block flips 🟢 PLAY → ✅ DONE (sibling to existing ✅ DONE-IN-FLIGHT entries); new PLAY block for **Phase 9 Commit 3b** (Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10) as next-active for Clair; Phase 9 PLAY block re-emerges from its paused-inside-milestone-scope state; Federation Event Propagation milestone PLAY block remains (the larger umbrella is still PLAY because Phase 9 hasn't closed). M6 (new) PENDING block and Pass 1 PENDING block both stay PENDING. Header Last updated bumped.
3. **`docs/ROADMAP.md`** — Tree updates: topological-sort cluster Implementation row flipped 🟢 → ✅; Phase 9 resume row flipped 🟡 → 🟢; Federation Event Propagation milestone header stays paused (since Phase 9 is still in-flight). Past section gains the implementation-shipped one-paragraph entry sibling to other ✅ DONE Past entries. Present section replaces the topological-sort-implementation paragraph with a Phase 9 Commit 3b paragraph describing the resume scope. "How to use this view" frontier line updated — parallel-eligible items now read "Phase 9 Commit 3b (Clair); M6 (new) Block 4 verb-by-verb walks (Chat Claude + Joe); the two JOURNAL gap retrospectives (Chat Claude)." Header version bumped 1.14 → 1.15.
4. **This runbook (`tasks/FEDERATION_TOPOSORT_IMPL.md`)** — header Status flipped ACTIVE → COMPLETED; Version 1.0 → 1.1; Last updated bumped to commit date with shipped-content summary per the bidirectional precedent.
5. **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** — catalogue row added per §6.3 below. The M15 entry. Header Last updated bumped.
6. **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** — header Last updated paragraph updated honestly to state Commit 3b is now unblocked and unpaused; the previously-documented "PAUSED at Commit 3a boundary, behind topological-sort wire-order non-determinism fix" framing replaces with "RESUMED at Commit 3b boundary, topological-sort wire-order non-determinism fix LANDED per topological-sort milestone close J-NNN." Status stays ACTIVE per the existing Phase 9 lifecycle (Phase 9 itself is still in-flight; the umbrella stays ACTIVE until Phase 9's own milestone-close commit).

Six files. The Phase 9 task file's Last updated paragraph is the load-bearing state-change the PLAY block flip reflects; without it, the documentation chain doesn't tell the honest story.

### 6.3 Catalogue row — exact phrasing

Verified at runbook authoring: catalogue currently has 14 entries (M1-M14) in `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` §6. Next-free M-number is **M15**. Column shape per existing table: `# | Bug | F-item(s) violated | Detection | Severity`.

Exact row text:

```markdown
| M15 (new) | Wire-order non-determinism: `topological_sort_events` preserves input-vector order for ready siblings (DAG roots with empty `prev_events`); its caller `compute_federation_delta_for_space` feeds it via `HashMap.values()` iteration with `RandomState`-randomised order. Two senders with identical Space state produce different federation-delta wire orderings ~50% of runs; cascading bootstrap rejections when `state.room_create` races `state.space_create` | F-1, F-3 (cascading) | Phase 9 Scenario 1 honesty check — LOCKED by D-076 (wire-order determinism is a sender-side normative property; two senders with identical Space history MUST produce byte-identical federation deltas); closed by topological-sort milestone J-NNN | HIGH |
```

**J-NNN substitution:** same convention as §5.5 — replace with actual milestone-close J-number at Commit 4 commit-authoring time. The catalogue row, the doc-comment in `phase9_two_node_smoke.rs`, and the §15 row in `docs/xgen_federation_propagation_design.md` all share the same J-NNN placeholder pattern and freeze together in Commit 4.

**Sub-bullet on the row's framing.** The row is honest provenance per §2.1 item 1 (catalogue-row-in-Commit-4 reasoning): the row exists because Scenario 1 surfaced the bug class. D-071 working as designed — deployment-level testing finds a bug class the catalogue didn't anticipate; catalogue gets extended. The Detection column explicitly references D-076 as the locking decision ("LOCKED by D-076") and J-NNN as the closing entry, so a future reader walking the catalogue from M15 backward through the references reaches both the principle (D-076) and the milestone closure narrative (J-NNN).

### 6.4 JOURNAL J-NNN entry shape

Mirror J-096's milestone-close shape. Sections in order:

1. **Summary.** One paragraph: what shipped (four atomic commits + topological-sort milestone close), what the test count change was (quote actual workspace numbers from Commit 2 + Commit 3), what unblocks (Phase 9 Commit 3b; the Federation Event Propagation milestone explicitly does NOT close yet). Cite J-097 as the design-phase close that preceded this.

2. **What was done.** Per-commit subsection (4 sub-entries):
   - **Commit 1 (doc-pass).** Cite commit hash. §6.4.3 added to canonical design doc; §15 row added; audit + design Status flips. No code; test count unchanged.
   - **Commit 2 (primitive + sibling fix + unit tests).** Cite commit hash. Two named code edits at `xgen-node/src/fanout.rs:193` (primitive sort + verbatim code-comment block) and `:321` (sibling Site 1 sort). N new unit tests landed (quote actual count and names from Clair's pre-Commit-2 lock). Test count: prior baseline + N. The wire-order-determinism witness (Test 4 — `compute_federation_delta_byte_identical_across_two_senders`) is the unit-level statement of D-076's full contract.
   - **Commit 3 (Scenario 1 second `#[ignore]` lift).** Cite commit hash. `#[ignore]` lifted from `two_node_federation_push_smoke_100_messages`; doc-comment rewritten to forward-looking shape; `#[serial_test::serial]` posture decision (state the decision — keep or remove). Test count +1 against Commit 2. Verification rigour applied: 5 isolated + 3 workspace runs; document any flake activity.
   - **Commit 4 (milestone close, this commit).** Six files per §6.2. Catalogue M15 added.

3. **Verification.** Quote full `cargo test --workspace` output from Commit 3 (the green workspace run with Scenario 1 lifted). Quote any flake activity observed. State explicitly: "two pre-existing flakes (§1.3) status: «fired N times / did not fire» during verification; per CLAUDE.md they remain carried forward." If `#[serial_test::serial]` was removed at Commit 3, include the doubled-run evidence per §5.4.

4. **Discipline notes.** Four sub-points:
   - **D-074 application count.** This is the N-th instance of D-074 same-commit-JOURNAL discipline since the principle was locked. **Use J-096's phrasing of the count as the established convention; do not invent new phrasing.** Grep J-096 at commit-authoring time to copy the phrasing verbatim — it's the cheapest way to avoid drift. Working count: J-095 locked the principle; J-096 (bidirectional milestone close) was the first downstream application; J-097 (topological-sort design close) the second; this entry the third subsequent application = **fourth instance total** by the established-at-J-096 convention. Verify the count at commit-authoring time against J-096's phrasing.
   - **D-071 sibling-shape extension.** This is the **fifth** project-wide instance of the audit-precedes-dependent-work pattern: J-081 Propagation Reliability Audit (first); Phase 7.5 design/impl (second); bidirectional `federation_nodes` audit → design → impl (third); topological-sort audit (fourth); this milestone close (which closes the fifth-instance arc — topological-sort audit → design → impl → milestone close). The pattern is now durable across five closures; recurring rather than one-off.
   - **"Honest longer work over fast shortcuts" fourth recurrence within Federation Event Propagation milestone.** Phase 7.5 was the first; bidirectional was the second; topological-sort design close (J-097) was the third; this milestone close is the fourth. Each recurrence delays Federation Event Propagation milestone closure by approximately one session-arc and produces a bug that gets fixed before downstream code depends on it.
   - **B3-shape gap question.** Bidirectional Commit 2.5 closed a sibling-shape drain-hook gap surfaced by Commit 3's integration test. Equivalent question for this milestone: is there a code path Commit 2 produced that no test exercises? Test 4 (wire-order-determinism witness) covers the contract; Tests 1-3 cover the primitive's mechanics; Scenario 1 covers the integration-level end-to-end. The answer should be "no gap" — the fix is small (two one-line sorts) and the test surface covers both sites and the contract they jointly satisfy. State the answer explicitly in the entry ("audit performed at milestone-close authoring: no Commit 2.5-shape gap surfaces"). If a gap does surface, it's Commit 4.5 territory, not silent in the milestone close.

5. **Carry-overs.** Phase 9 Commit 3b is now-active for Clair; Federation Event Propagation milestone stays PLAY; M6 (new) + XGID Retrofit Pass 1 stay PENDING; sibling Q3.ii-analogue sites (`collect_sync_history`, `apply_fanout` history-push) remain Q2.γ-flagged for future scheduling. No new carry-overs added by this milestone; the chain is unchanged in shape per §2.4.

6. **Files changed in this commit.** Scannable bullet list per J-097's format — each bullet `filename` + one-line description of what changed in that file. No prose paragraphs; Clair scanning at commit-authoring time needs to verify file-by-file against the actual diff and a bullet list makes that scan instant. Six files per §6.2. JOURNAL.md is the first bullet (D-074 discipline check visible from this entry alone).

7. **Next.** Phase 9 Commit 3b (`tasks/FEDERATION_PROPAGATION_PHASE_9.md`, RESUMED). Entry point for Clair: `CLAUDE.md` PLAY block.

**Entry length:** dense paragraphs matching J-096's density. Estimate ~700-900 words. Quote actual numbers (Rule 5); paraphrase actual outputs only where summary is acceptable per the verbatim-or-summary discipline (Rule 5).

### 6.5 CLAUDE.md PLAY block flip detail

The current PLAY blocks (as of J-097 design-phase close) describe the topological-sort milestone implementation-runbook-authoring as 🟢 PLAY next-active for Chat Claude + Joe. Post-Commit-4 state:

- **Topological-sort milestone block** flips ✅ DONE (sibling to existing ✅ DONE-IN-FLIGHT entries). Brief summary: four commits, audit + design + implementation closed, JOURNAL J-NNN, test count delta from baseline.
- **New PLAY block for Phase 9 Commit 3b.** Next-active for Clair. Entry point: `tasks/FEDERATION_PROPAGATION_PHASE_9.md`. Scope: Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10 per the existing Phase 9 task file. Expected commit count ~5-7 atomic commits per the existing Q4 Lock.
- **Federation Event Propagation milestone block** stays 🟢 PLAY (the umbrella is still in-flight because Phase 9 hasn't closed).
- **M6 (new) PENDING block and Pass 1 PENDING block** stay PENDING (blocked behind Federation Event Propagation milestone closure, which is blocked behind Phase 9 closure).
- **Header Last updated bumped** to Commit 4 date.

The load-bearing distinction: the topological-sort milestone block flips DONE, AND a new Phase 9 Commit 3b block emerges, AND the Federation Event Propagation milestone block stays PLAY. Three state changes, not one. Clair will be tempted to over-flip; §2.4's framing guards against that drift but the PLAY-block authoring is where the drift would show.

### 6.6 ROADMAP.md flips detail

The Visual structure tree's topological-sort cluster:

```
└── ✅ Topological-sort wire-order determinism (sibling to Bidirectional federation_nodes)
    ├── ✅ Audit phase (canonical doc shipped 2026-05-22 at tasks/FEDERATION_TOPOSORT_AUDIT.md v1.0 → COMPLETED at impl Commit 1)
    ├── ✅ Design phase (Q3.ii + Q2 middle + Q2.γ + Q1 Shape A v1 locked 2026-05-22; design task file at tasks/FEDERATION_TOPOSORT_DESIGN.md v1.0 → COMPLETED at impl Commit 1; D-076 promoted)
    ├── ✅ Implementation runbook authoring (Chat Claude + Joe shipped at session 2026-05-22; runbook at tasks/FEDERATION_TOPOSORT_IMPL.md v1.0 → 1.1 at Commit 4)
    ├── ✅ Implementation (Clair shipped four-commit sequence)
    └── 🟢 Phase 9 Commit 3b unblocked (Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10; next-active for Clair)
```

The Federation Event Propagation milestone header in the tree stays "🟡 Phase 9 in-flight" because Phase 9 itself hasn't closed yet (Commit 3b is still in-flight after this milestone). The milestone fully flips when Phase 9 closes — separate future commit.

**Past section:** add the implementation-shipped one-paragraph entry. Paragraph length, sibling to other ✅ DONE Past entries. The design-phase Past entry from v1.13 stands authoritative; the implementation-shipped Past entry is its sibling.

**Present section:** replace the "🟢 Topological-sort implementation runbook authoring" paragraph (or whatever the v1.14 Present line reads) with a "🟢 Federation Event Propagation Phase 9 Commit 3b" paragraph describing the resume scope. The other Present-section items (M6 verb-by-verb, JOURNAL gap retrospectives) stay as they were.

**"What's playing right now?" line** updated — topological-sort phase moves from "in-flight" to ✅ Past; Phase 9 Commit 3b becomes next-active for Clair.

**Version bump:** 1.14 → 1.15. Header Last updated includes a shipped-content summary matching the J-097 update's density.

### 6.7 DoD for Commit 4

- [ ] JOURNAL.md entry written per §6.4 with actual `cargo test --workspace` output quoted (not paraphrased — Rule 2).
- [ ] CLAUDE.md PLAY blocks flipped per §6.5: topological-sort ✅; new Phase 9 Commit 3b 🟢; Federation Event Propagation milestone stays 🟢; M6 + Pass 1 PENDING blocks unchanged. Header Last updated bumped.
- [ ] ROADMAP.md tree + Past section + Present section + "What's playing" line + header bumped per §6.6. Version bump 1.14 → 1.15.
- [ ] This runbook's header Status flipped ACTIVE → COMPLETED; Version 1.0 → 1.1; Last updated bumped with shipped-content summary.
- [ ] `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` gains the M15 catalogue row per §6.3 exact phrasing. J-NNN placeholder replaced with actual J-number at this commit. Header Last updated bumped.
- [ ] `tasks/FEDERATION_PROPAGATION_PHASE_9.md` header Last updated paragraph honestly states Commit 3b is RESUMED, topological-sort fix LANDED per J-NNN. Status stays ACTIVE per the existing Phase 9 lifecycle.
- [ ] J-NNN placeholders frozen across all three sites (doc-comment in `phase9_two_node_smoke.rs` from Commit 3; catalogue M15 row; §15 row in `docs/xgen_federation_propagation_design.md` from Commit 1). All three reference the same J-number.
- [ ] B3-shape gap audit performed and result documented in JOURNAL entry per §6.4 sub-point 4. Default expected answer: "no gap."
- [ ] All six files (§6.2) in one atomic commit per D-074. **JOURNAL.md MUST be in the changed-files list** — this is the D-074 discipline check.
- [ ] Commit message names this as "Commit 4 of 4 — topological-sort wire-order determinism milestone close; Phase 9 Commit 3b unblocked."
- [ ] No code touched. No tests touched. If anything in the codebase is unstable at this point, Commit 4 does not ship — back-fill the stability fix in Commit 3 or a Commit 3.5 first.
- [ ] **What this commit does NOT do, restated** (§2.4 anchor): it does NOT close Phase 9; it does NOT close the Federation Event Propagation milestone; it does NOT unblock M6 (new) or XGID Retrofit Pass 1. ROADMAP and CLAUDE.md state changes respect this distinction.

---

## 7. Discipline notes

### 7.1 Precedent departure: why §7 exists in this runbook

This section exists even though the sibling bidirectional impl runbook (`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`, COMPLETED v1.1) ends at §9 cross-references with no discipline-notes section. The departure is grounded in three considerations:

1. **Trilogy consistency outranks one-step-earlier-precedent consistency when they conflict.** The topological-sort phase's audit doc (`tasks/FEDERATION_TOPOSORT_AUDIT.md` §9) and design task file (`tasks/FEDERATION_TOPOSORT_DESIGN.md` §9) both ship with discipline-notes sections. The impl runbook being the only document in the audit→design→impl trilogy without one would itself be drift. Internal-trilogy consistency wins.
2. **The bidirectional precedent's absence was absence-of-need, not deliberate design.** Bidirectional was the second sibling-shape recurrence in Federation Event Propagation milestone scope (Phase 7.5 first, bidirectional second). At second recurrence there is nothing to call out as a pattern. At third recurrence (this milestone) the pattern is durable enough to name. The bidirectional precedent established "impl runbooks don't have discipline-notes when there is no pattern to call out yet" — not "impl runbooks don't have discipline-notes." Applying the precedent contextually rather than literally is the right read.
3. **D-076 promotion as fourth member of the four-decision no-drift-surface discipline family is a structural event that needs a pointer in the runbook.** The runbook is the operational doc for shipping the fourth member; the family-completion framing should be visible here, not only after-the-fact in JOURNAL J-NNN's discipline-notes sub-section. Future contributors reading the runbook standalone (without paired JOURNAL access) need the connection to land within the runbook itself.

These three points are recorded here so the section's existence is self-defending. A future audit asking "why does this runbook have §7 when its sibling doesn't?" finds the answer in the section itself rather than requiring a separate ad-hoc justification.

### 7.2 Sibling-in-shape discipline — third sibling-runbook in Federation Event Propagation milestone

This is the third sibling-in-shape impl runbook within the Federation Event Propagation milestone. The three:

1. `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` (Status COMPLETED v1.0) — five commits closing the cold-start bootstrap milestone.
2. `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (Status COMPLETED v1.1) — four commits closing the bidirectional `federation_nodes` milestone (vantage-aware applier; D-075).
3. This runbook — four commits closing the topological-sort wire-order determinism milestone (primitive + sibling Site 1 fix; D-076).

The pattern is now durable: each phase ships an audit, then a design task file, then an impl runbook, in three separate sessions with two separate audiences (Chat Claude + Joe for audit/design; Clair for impl); the impl runbook's structure follows §1 (What this document is) → §2 (Sequence overview) → §3-§6 (per-commit detail, one section per atomic commit) → §7-§8 (discipline notes + cross-references) where the discipline-notes section is added at third recurrence or later per §7.1. Future Federation-milestone-internal phase implementations should follow sibling shape unless a substantive reason emerges to depart — the cost of departing is documentation drift; the cost of conforming is small.

### 7.3 Inline-lock pattern — third recurrence

The pattern of locking design-phase questions at the audit-document lifecycle rather than carrying them into the design-phase deliberation, established at the bidirectional milestone (Q2 locked inline at the bidirectional audit; Q1 design-phase-walked), recurs in the topological-sort phase at full depth: all three audit-phase questions (Q1, Q2, Q3) locked inline in `tasks/FEDERATION_TOPOSORT_AUDIT.md` at the design-phase opening session (2026-05-22). The design task file then expounded the already-locked decisions rather than re-walking the questions; the design phase was exposition rather than deliberation.

This runbook continues the pattern by treating the three locks (Q3.ii canonical wire ordering required; Q2 middle + Q2.γ primitive-fix + forward-binding; Q1 Shape A v1 + sibling Site 1 fix) as already-decided rather than re-examining them. Clair reading the runbook sees implementation steps, not litigation steps. The split-session discipline (§7.4) reinforces this — by the time the runbook lands, the locks are settled; the runbook is operational, not deliberative.

Future audit closures within this project should expect this pattern as default. The audit doc lifecycle is the right home for locking the structural questions; the design task file lifecycle is the right home for exposition of locks + rejected alternatives; the impl runbook lifecycle is the right home for operational sequencing. Carrying questions across lifecycle boundaries is the failure mode the pattern guards against — it produces design-phase work that re-walks already-resolved questions and impl runbooks that re-examine already-locked decisions, both of which dilute the artefacts' authority.

### 7.4 Split-session discipline

Four sessions across the topological-sort phase: audit doc authoring (2026-05-22, Chat Claude + Joe); design task file authoring (2026-05-22, Chat Claude + Joe in a separate session); this runbook authoring (2026-05-22, Chat Claude + Joe in a third separate session); Clair's implementation arc (multiple sessions across four commits, to be scheduled).

Each session has a different headspace requirement. Audit is forensic — trace the bug code-grounded to source. Design is deliberative — walk the option space and lock against principle. Runbook authoring is operational — sequence the already-locked design as Clair-facing code-level work. Implementation is execution — ship the runbook's instructions against the codebase. Collapsing any two of these into a single session degrades both — audit and design in the same session would produce hasty audits or under-deliberated designs; design and runbook in the same session would produce runbooks that re-deliberate locks; runbook and implementation in the same session would produce implementations that drift from the canonical record.

The split-session discipline is the operational manifestation of the inline-lock pattern (§7.3) — if the design task file is exposition of locks rather than deliberation, the runbook can be operational rather than deliberative, and Clair's implementation can be execution rather than redesign. The discipline cascades: cleanly-separated session arcs produce cleanly-separated artefacts.

### 7.5 Four-decision no-drift-surface discipline family — family-completion

D-076 promotion at the design-phase close commit (2026-05-22, J-097) brings the no-drift-surface discipline family to four members, each locking a no-drift property at a different protocol layer:

| Decision | Layer | Property locked |
|---|---|---|
| **D-067** | Code-organisation | Single source of truth for derived state reads (no two readers consulting different sources for the same logical question) |
| **D-070** | Transport-layer | Two events of equal importance, opposite direction (acceptance + rejection signals both exist + both carry envelope-level correlation) |
| **D-075** | Event-model | Relationship-shaped events record one party's act + derived projection with vantage-aware applier logic |
| **D-076** | Wire-format | Two senders with identical state produce byte-identical federation deltas |

The family is now structurally complete in the sense that each layer of the protocol has a named decision locking its no-drift-surface property. Future protocol additions should explicitly check against the family: does this new wire format have canonical sender output (D-076)? Does this new transport variant have correlation (D-070)? Does this new event-model addition have single-party assertion plus derived projection (D-075)? Does this new code-organisation surface maintain single source of truth (D-067)? Four explicit questions surfaced at design time; no implicit assumptions about no-drift behaviour emerging from local primitives.

The runbook is the operational doc for shipping the fourth member. Pointers from this runbook back to all four members (per §8 cross-references) enable future contributors to navigate the family. The structural completion does not preclude future members — a new layer at the protocol could surface a new no-drift-surface question — but the four named decisions cover the four layers the protocol currently operates across (code-organisation, transport, event-model, wire-format), and the discipline of locking the property explicitly rather than trusting it to emerge from local primitives is durable across the family.

### 7.6 "Honest longer work over fast shortcuts" — fourth recurrence within Federation Event Propagation milestone

Fourth recurrence of the discipline within Federation Event Propagation milestone scope: Phase 7.5 was the first (B3 amendment closed at Commit 3.5 rather than buried in Commit 4); bidirectional `federation_nodes` was the second (full audit→design→impl arc rather than workaround); topological-sort design close (J-097) was the third (load-bearing gap found in bidirectional Commit 4 verification, opened as its own phase per D-071 rather than papered over); this runbook plus Clair's implementation arc is the fourth.

Each recurrence has the same shape: dependent work surfaces a load-bearing protocol gap; the team chooses the longer-but-honest path (audit → design → implementation → close) over the shorter-but-papering-over path (workaround, deferred backlog item, silent `#[ignore]` annotation). The pattern's cost is real — Federation Event Propagation milestone closure delayed by approximately one session-arc per recurrence; four recurrences across the milestone equate to roughly four session-arcs of delay against the initial estimate at milestone opening. The pattern's benefit is also real — four bug classes closed before they ship to production, before downstream code (M6 + Pass 1 implementations) depends on the broken behaviour, before a future contributor has to discover and fix them from a production deployment.

The cost-benefit framing matches the bidirectional milestone close (J-096) and the Phase 7.5 milestone close (J-094) recorded discipline notes. The pattern is the project's commitment to working-functions-not-checkmarks priority recorded in the Phase 9 survey (J-091): a milestone that ships green and turns out to have federation bugs three weeks later is a milestone that failed at its real job. Four-recurrence durability suggests the pattern is now the project's default for in-flight gap discovery, not a Federation-milestone-internal accident.

The B3-shape gap audit at Commit 4 milestone-close authoring (§6.4 sub-point 4) is the in-flight extension of this same discipline — if a fifth gap surfaces in Clair's implementation arc, it gets a named commit (Commit 2.5 or Commit 4.5 territory per the bidirectional precedent), not a silent fold into the milestone close. The default expected answer for this milestone is "no gap" (the fix is small; the test surface is comprehensive); the discipline of asking-and-answering is what makes "no gap" trustworthy.

---

## 8. Cross-references

Documents Clair should have read (or be ready to read) before starting Commit 1, plus principle + JOURNAL + tooling references the runbook depends on.

### 8.1 Audit + design documents (the trilogy)

- **`tasks/FEDERATION_TOPOSORT_AUDIT.md`** (Status ACTIVE v1.0 at runbook-landing; flips COMPLETED v1.0 at Commit 1) — the audit doc this implementation runbook consumes as input. Code-grounded mechanism evidence at §3 (Site 1 + Site 2 compounding); canonical sibling sort precedent at §3.5; scope boundaries at §5.1 + §5.2; the three inline locks recorded at §6.1 + §6.2 + §6.3 are authoritative historical record of the design-at-lock-time.
- **`tasks/FEDERATION_TOPOSORT_DESIGN.md`** (Status ACTIVE v1.0 at runbook-landing; flips COMPLETED v1.0 at Commit 1) — the design task file this implementation runbook ships against. The three Joe-locks at §3 (Q3.ii) + §4 (Q2 middle + Q2.γ) + §5 (Q1 Shape A v1 + sibling Site 1 fix) are load-bearing for Clair's implementation; the verbatim code-comment block at §5.3 is the authoritative source for Commit 2's mandatory code-comment block per §4.3 of this runbook.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md`** (Status COMPLETED v1.0) — structural template for the audit doc; sibling-in-shape precedent.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md`** (Status COMPLETED v1.0) — structural template for the design task file; sibling-in-shape precedent for the inline-lock pattern (§7.3).
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`** (Status COMPLETED v1.1) — structural template for this runbook; sibling-in-shape for the four-commit sequence; reference for what `#[serial_test::serial]` was added to address (§5.4).
- **`tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md`** (Status COMPLETED v1.0) — earlier sibling-shape impl runbook precedent (five commits for Phase 7.5; this milestone has four commits because the scope is smaller).
- **`docs/xgen_federation_propagation_design.md`** (Status ACTIVE v1.0) — the canonical Federation Event Propagation design doc. Commit 1 adds §6.4.3 sibling subsection + §15 row per §3.3 of this runbook. §6.4 (Phase 7 F-3 framework) + §6.4.1 (Phase 7.5) + §6.4.2 (bidirectional `federation_nodes`) are the existing sibling-shape precedent.
- **`tasks/FEDERATION_PROPAGATION_COMPLETION.md`** §3.3.1 R4 (Status ACTIVE v1.0) — the original Phase 3 cross-Space ordering lock that was silent on within-Space ordering, surfacing the gap this milestone closes. Reference for the framing of D-076's complement to R4 (cross-Space + within-Space wire-order both normative).

### 8.2 DECISIONS.md — the four-decision no-drift-surface discipline family

- **D-067** — Single source of truth / no drift surface (code-organisation layer). The principle whose finer-grain wire-format instance this milestone instantiates as D-076.
- **D-068** — Original drift-surface decision (CLI Audit closure precedent); precursor to D-067's framing.
- **D-069** — Canonical-document rule. The audit doc + design task file + this runbook all instantiate.
- **D-070** — Two events of equal importance, opposite direction (transport-layer correlation pair). No-drift-surface family member.
- **D-071** — Subsystem audits precede dependent milestones. Fifth project-wide recurrence at this milestone (per §6.4 discipline notes sub-point 2).
- **D-074** — Milestone-close commits MUST include JOURNAL.md. Applies to Commit 4 of this runbook (§6.7 DoD final discipline check).
- **D-075** — Bidirectional vantage-aware applier rule (event-model layer). No-drift-surface family member; sibling-distinct from D-076.
- **D-076** — Wire-format determinism. Wire-format-layer member of the no-drift-surface discipline family; promoted at design-phase close (J-097, 2026-05-22). The principle this implementation runbook ships.

### 8.3 JOURNAL.md

- **J-081** — Propagation Reliability Audit. First instance of D-071 audit-precedes-dependent-work pattern; structural template for subsequent audits.
- **J-091** — Phase 9 survey. Source for the 14-entry failure-mode catalogue; this milestone adds M15 per audit §4.3.
- **J-093** — Phase 7.5 design closure. Sibling-shape precedent for design-phase pattern.
- **J-094** — Phase 7.5 implementation closure. Sibling-shape precedent for implementation-phase pattern; D-074 first-locking-context reference.
- **J-095** — XGID Adoption v1 implementation milestone closure. D-074 locked here; "Use J-096's phrasing of the D-074 count as the established convention" per §6.4 sub-point 1.
- **J-096** — Bidirectional `federation_nodes` implementation milestone closure + topological-sort finding surfaced. Originating record of J-096 Finding 2 (the wire-order non-determinism this milestone closes); load-bearing for the audit §2.1 four-check hypothesis sequence cited in §5.3 verification rigour. First downstream application of D-074 per the established convention.
- **J-097** — Topological-sort design-phase close. The design-phase commit this runbook directly succeeds; reference for the three Joe-locks landed in DECISIONS.md (D-076 promotion).
- **J-098 (expected at runbook-landing)** — small entry recording this session's runbook-authoring closure. Number subject to verification at commit-authoring time per §1.1 reading order.
- **J-NNN (expected at Commit 4)** — milestone-close JOURNAL entry. Number depends on what lands between runbook-landing and Commit 4; replaces J-NNN placeholders across three sites (§3.4 §15 row + §5.5 doc-comment + §6.3 catalogue row) at Commit 4 freeze.

### 8.4 Code surfaces

- **`xgen-node/src/fanout.rs::topological_sort_events`** (lines 193-220) — the primitive Site 2 of the audit's §3. Primitive sort + verbatim code-comment block lands here at Commit 2 per §4.3.
- **`xgen-node/src/fanout.rs::compute_federation_delta_for_space`** (lines 311-333; HashMap feed at ~321) — Site 1 of the audit's §3. Sibling fix lands here at Commit 2 per §4.4.
- **`xgen-core/src/node/runtime.rs::topological_sort`** (lines 859-912) — **canonical sibling sort precedent.** Kahn's algorithm with explicit `queue_vec.sort()` tie-breaking. Reference site for the primitive fix's tie-break semantic. **Not a consolidation target** — the design phase deliberately did not consolidate the two implementations (§4.7 anti-refactor guardrail).
- **`xgen-node/src/tests/phase9_two_node_smoke.rs::two_node_federation_push_smoke_100_messages`** — Phase 9 Scenario 1 regression witness. `#[ignore]` lifts at Commit 3; doc-comment rewrite per §5.5; `#[serial_test::serial]` posture decision per §5.4.
- **`xgen-node/src/fanout.rs::collect_sync_history`** + **`apply_fanout` history-push** — Q2.γ forward-binding sites; flagged out-of-scope per design §4.1 + §8.1. Sibling Q3.ii-analogue sites that the principle applies to but that do not get fixed in this milestone.

### 8.5 Appendix references

- **`docs/xgen_appendix_j_en.md`** (Appendix J) — content-hash framing for event_id. Cited by the verbatim code-comment block at §4.3 ("event_id is content-hash-derived per Appendix J ... so the sort key is byte-stable across senders with identical Space history"). Verified at runbook authoring: file exists at `E:\Projects\XGenProtocol\docs\xgen_appendix_j_en.md` (~40 KB).

### 8.6 Operational state references

- **`CLAUDE.md`** — operational state; PLAY block flips at Commit 4 per §6.5. MANDATORY behaviour rules referenced throughout (Rule 3 stop-and-report, Rule 4 JOURNAL-written-last, Rule 5 quote-actual-output-never-invent-numbers, Rule 7 DoD-is-a-checklist).
- **`docs/ROADMAP.md`** (v1.13 at runbook-landing baseline; bumps to v1.14 at runbook-landing commit; v1.15 at Commit 4) — navigation map. Tree + Past + Present + version updates at runbook-landing (this session) and Commit 4 (Clair's milestone close).
- **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (Status ACTIVE v1.0) — Phase 9 task file. Commit 3a scope (Scenario 1 lift) was satisfied earlier and re-stood-down on this milestone's finding; Commit 3b scope (Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10) unblocks at Commit 4 milestone close. Header Last updated paragraph updates at Commit 4 per §6.2 item 6.
- **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** (Status COMPLETED v1.1) — 14-entry failure-mode catalogue; M15 row added at Commit 4 per §6.3 exact phrasing.

---

*End of implementation runbook. Status flips ACTIVE → COMPLETED in Commit 4 per the established implementation-runbook lifecycle (sibling to `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` v1.0 → COMPLETED v1.1 at bidirectional milestone close, and `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` v1.0 → COMPLETED at Phase 7.5 milestone close). Locked content above is preserved as authoritative record of the four-commit sequence.*  
