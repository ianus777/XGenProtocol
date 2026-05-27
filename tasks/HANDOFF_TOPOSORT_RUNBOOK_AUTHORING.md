# Session Handoff — Topological-Sort Runbook Authoring (in-flight)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-27 (J-129 — Stale-Status flag fix. Topological-sort milestone closed at J-101 (2026-05-23); this handoff note's Status flag was never flipped from ACTIVE → COMPLETED at milestone close — four-day-elapsed-time stale flag. Surfaced by Clair's pre-Clair six-dimension audit at session-open for Pass 3 implementation kickoff (Rule 0 ACTIVE-HANDOFF sweep). Folded into the J-129 Track 1 canonical-record amendment atomic per anti-tempfile-deletion + J-107 + J-100 retention precedent: handoff content stays as historical record of topo-sort runbook-authoring session-boundary state; only the Status flag flips. Sibling-shape to J-100's retrospective fix-up framing but distinct in time-scale (this is four-day-elapsed-time discovery, J-100 was same-day). Per Rule 0 + D-065 + D-074.) Previous 2026-05-22 update content stands authoritative as historical record — see below.  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

Compact re-entry note for the next Chat-Claude session. The topological-sort implementation-runbook authoring is in-flight at session boundary; this note captures **where we are, what's drafted, what's locked, and what remains** so the next session can resume at Step 6 without re-reading the full arc.

The runbook itself is on disk at `tasks/FEDERATION_TOPOSORT_IMPL.md` (~93 KB, Status: ACTIVE v1.0, eight sections §1-§8). It is the authoritative artefact; this note is purely re-entry context.

## 2. Where we are in the session-arc

Original arc (recorded in earlier sessions):

```
Step 1 ✅ Align on runbook shape (this session, message 1)
Step 2 ✅ Re-read audit + design + bidirectional runbook; resolve Q1-Q4
Step 3 ✅ Draft runbook skeleton (header + §1-§2)
Step 4 ✅ Per-commit detail
  ├─ Step 4a ✅ §3 (Commit 1 — doc-pass)
  ├─ Step 4b ✅ §4 (Commit 2 — primitive + sibling fix + unit tests)
  └─ Step 4c ✅ §5 + §6 (Commit 3 + Commit 4)
Step 5 + 5.5 ✅ §7 discipline notes + §8 cross-references
Step 6 ⬜ Final read-through (next session)
Step 7 ⬜ PowerShell push instructions for the four-file runbook-landing commit (next session)
```

Eight sections drafted on disk. Two steps remain.

## 3. What's locked at session boundary

### 3.1 Path B vs Path A — Path B locked

Runbook ships with **8 sections** (§1 What this document is, §2 Sequence overview, §3 Commit 1, §4 Commit 2, §5 Commit 3, §6 Commit 4, §7 Discipline notes, §8 Cross-references) — not 9 like the bidirectional precedent. The precedent departure was Joe-locked with three-point defence recorded inline at §7.1 (trilogy consistency + absence-was-absence-of-need + D-076 family-completion needs runbook-visible pointer). Test-count discipline + risk-surface material absorbed inline at §2.1 + §5.6 + §6.7 rather than as standalone sections.

### 3.2 Joe-locks within the runbook

- **Q3.ii canonical wire ordering required** (design §3) — wire-order determinism is sender-side normative.
- **Q2 middle + Q2.γ** (design §4) — fix primitive's contract once; forward-bind to Node-to-Client siblings.
- **Q1 Shape A v1 + sibling Site 1 fix** (design §5) — event_id lex sort at `topological_sort_events:193` + sort `Vec<Event>` at `compute_federation_delta_for_space:321`; v1 `&str` sort, Pass 3 retype to `EventXgid`.

All three locked in `tasks/FEDERATION_TOPOSORT_DESIGN.md` (Status ACTIVE v1.0; flips COMPLETED at Commit 1 of Clair's impl arc). D-076 promoted to DECISIONS.md at J-097 design-phase close.

### 3.3 Runbook structural choices Joe-confirmed

- **Four-commit Clair sequence**: doc-pass → primitive + sibling + unit tests → Phase 9 Scenario 1 lift → milestone close.
- **Three Joe-lock checkpoints** (§2.3): post-Commit-1 if doc-pass surfaces drift; pre-Commit-2 unit-test list proposal; post-Commit-2 / pre-Commit-3 primitive shape locked.
- **What this milestone CANNOT close** (§2.4): Commit 4 closes topological-sort milestone only; Phase 9 Commit 3b unblocks → resumes; Federation Event Propagation milestone STAYS PLAY; M6 (new) + Pass 1 stay PENDING. Three-state-change framing applied at four sites (§2.4 + §6.5 + §6.6 + §6.7 final DoD item).
- **`#[serial_test::serial]` asymmetry framing** (§5.4): default-keep is silent; remove requires commit-message justification + 5 isolated parallel-workspace runs with `cargo clean` between + full output paste (not summary).
- **Verification rigour** (§5.3): 5 isolated runs + 3 workspace runs = 8 green runs minimum before lifting `#[ignore]`. Optional post-reboot bonus check if flake suspicion arises.
- **Doc-comment rewrite** (§5.5): exact target text locked verbatim with `J-NNN` placeholder pattern; "third and final rewrite" framing.
- **Catalogue row M15** (§6.3): exact phrasing locked; verified 14 existing entries (M1-M14) at runbook authoring; M15 is next-free.
- **J-NNN placeholders at three sites** that freeze together at Commit 4: canonical-design-doc §15 row + doc-comment in `phase9_two_node_smoke.rs` + catalogue M15 row.

### 3.4 J-098 expected at runbook-landing

J-097 is the latest JOURNAL entry (verified via `read_text_file` head:10 mid-session). Runbook-landing commit's JOURNAL entry is **J-098** unless something lands between now and the next session. **Next session must re-verify** with `read_text_file` head:5 on `JOURNAL.md` before drafting Step 7's PowerShell — same discipline that caught the J-098 question this session.

## 4. What's drafted on disk

**File**: `tasks/FEDERATION_TOPOSORT_IMPL.md` — Status: ACTIVE v1.0, ~93 KB, eight sections.

**§1 What this document is** — framing + reading order (§1.1) + latitude (§1.2) + pre-existing flakes carried forward (§1.3).

**§2 Sequence overview** — four-commit table (§2.1) + files-touched across the four commits (§2.2, including sibling-shape verification nudge for test-file placement) + three Joe-lock checkpoints (§2.3) + what this milestone CANNOT close (§2.4 with dependency-chain diagram).

**§3 Commit 1 — Doc-pass commit** — scope (§3.1) + files touched (§3.2) + §6.4.3 content sketch (§3.3, verified §6.4.3 is next-free slot via grep) + §15 Implementation Complete row (§3.4, with ~470-word headline blockquote draft) + audit doc Status flip rationale (§3.5) + design task file Status flip rationale (§3.6) + DoD (§3.7).

**§4 Commit 2 — Primitive + sibling fix + unit tests** — scope (§4.1) + files touched (§4.2) + primitive fix at :193 with pre/post code + verbatim code-comment block (§4.3) + sibling Site 1 fix at :321 with pre/post code (§4.4) + four seed tests + two optional tests (§4.5) + DoD (§4.6) + seven anti-drift guardrails (§4.7).

**§5 Commit 3 — Phase 9 Scenario 1 `#[ignore]` lift** — scope (§5.1) + files touched (§5.2) + verification rigour with 8-green-runs minimum (§5.3) + `#[serial_test::serial]` asymmetry framing (§5.4) + exact doc-comment rewrite text (§5.5) + diagnostic playbook if scenario fails (§5.6) + DoD (§5.7).

**§6 Commit 4 — Milestone close** — scope (§6.1) + six files touched (§6.2) + catalogue M15 row exact phrasing (§6.3) + JOURNAL J-NNN entry shape with seven sub-sections (§6.4, including the D-074-count-defer-to-J-096-convention adjustment and the bullet-list-not-prose Files-changed format) + CLAUDE.md PLAY block flip detail (§6.5) + ROADMAP.md flips detail (§6.6) + DoD (§6.7).

**§7 Discipline notes** (6 paragraphs) — precedent departure framing (§7.1) + sibling-in-shape third recurrence (§7.2) + inline-lock pattern third recurrence (§7.3) + split-session discipline (§7.4) + four-decision no-drift-surface family with full table (§7.5) + "honest longer work" fourth recurrence (§7.6).

**§8 Cross-references** (6 subsections) — audit + design trilogy (§8.1) + DECISIONS.md eight entries (§8.2) + JOURNAL nine entries (§8.3) + code surfaces five entries (§8.4) + Appendix J reference (§8.5) + operational state references (§8.6).

## 5. What remains — Step 6 + Step 7

### 5.1 Step 6 — Final read-through

Joe's last sanity-check before the runbook lands. Open `tasks/FEDERATION_TOPOSORT_IMPL.md` end-to-end and read for:

- **Section-numbering coherence** — verify §2.1 → §2.2 → §2.3 → §2.4 reads in order (this was a fix-in-flight during Step 3); verify no duplicate headings.
- **Cross-reference accuracy** — §-anchors mentioned in one section actually exist in the section referenced (e.g., §5.4 refers to §5.7 DoD item; verify §5.7 exists and has the relevant item).
- **Verbatim code-comment block matches design §5.3** with the two adjustments noted (four-member family naming + 2026-05-22 date).
- **Catalogue M15 row text** matches the exact phrasing Joe locked in the prior session (the row in §6.3).
- **Doc-comment exact target text** matches the exact phrasing Joe locked in the prior session (the block in §5.5).
- **J-NNN placeholder convention** is consistent across the three sites (§3.4 §15 row + §5.5 doc-comment + §6.3 catalogue row).
- **Three-state-change framing** at §2.4 + §6.5 + §6.6 + §6.7 final DoD item is consistent.
- **Discipline notes precedent-departure defence** at §7.1 is self-defending (a future audit asking "why does this runbook have §7 when its sibling doesn't?" finds the answer in §7.1 itself).

If anything reads off, fix in-place via `Filesystem:edit_file` before Step 7.

### 5.2 Step 7 — PowerShell push instructions for the runbook-landing commit

**Four files in one atomic state-change commit per D-074**:

1. `tasks/FEDERATION_TOPOSORT_IMPL.md` — the runbook itself (new file; Status: ACTIVE v1.0).
2. `CLAUDE.md` — PLAY block flip from "runbook authoring HERE (Chat Claude + Joe)" to "runbook authored, Clair pickup HERE." The bidirectional precedent's runbook-landing commit pattern is the reference.
3. `docs/ROADMAP.md` — Visual structure tree: runbook-authoring row 🟢 → ✅; implementation row 🟡 → 🟢 (handoff ready). Past section gains the runbook-shipped paragraph. Present section updates. Header version bump 1.13 → 1.14.
4. `JOURNAL.md` — small entry **J-098** (re-verify the J-number per §3.4 before drafting): "Topological-sort implementation runbook SHIPPED at `tasks/FEDERATION_TOPOSORT_IMPL.md` v1.0. Eight sections, Clair-facing, four-commit sequence: doc-pass → primitive + sibling fix + unit tests → Phase 9 Scenario 1 lift → milestone close. Three Joe-lock checkpoints. D-076 implementation arc unblocks Clair pickup." Reference the sibling bidirectional runbook-landing precedent if it had one; if not, this entry establishes the pattern.

**PowerShell command sequence per project convention** (per memory: explicit `git add <file>` per modified file, never `git add .`; `git status` as sanity-check before commit; multi-paragraph commit message via multiple `-m` flags; Claude never pushes directly — Joe runs the commands):

```powershell
cd E:\Projects\XGenProtocol

git add tasks/FEDERATION_TOPOSORT_IMPL.md
git add CLAUDE.md
git add docs/ROADMAP.md
git add JOURNAL.md

git status  # sanity-check: four files modified, no others

git commit `
  -m "Topological-sort implementation runbook shipped" `
  -m "Eight-section Clair-facing runbook for the four-commit topological-sort wire-order determinism milestone. Sibling-shape to tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md with one structural addition (§7 discipline notes; precedent departure self-defended at §7.1)." `
  -m "Three Joe-locks from design phase carried forward as already-decided: Q3.ii canonical wire ordering required; Q2 middle + Q2.γ primitive-fix + forward-binding; Q1 Shape A v1 + sibling Site 1 fix. D-076 implementation arc unblocked for Clair pickup." `
  -m "JOURNAL J-098 + CLAUDE.md PLAY block flip + ROADMAP.md v1.13 → v1.14 in same commit per D-074 same-commit discipline."

git push
```

The exact `-m` paragraph wording is Joe's call at push time. The structural pieces (four-file `git add`, `git status` sanity-check, multi-`-m` commit message, `git push` only after the four-file add is verified) are non-negotiable.

## 6. Re-entry checklist for the next session

Things the next session should do in order:

1. **Verify J-098 still expected** — `read_text_file` head:5 on `E:\Projects\XGenProtocol\JOURNAL.md`. If J-098 is now taken (something landed between sessions), use the next-free J-number across all references in the runbook-landing commit's JOURNAL entry.
2. **Read this handoff note end-to-end** — captures session-boundary state, locked decisions, what's drafted, what remains.
3. **Open `tasks/FEDERATION_TOPOSORT_IMPL.md`** — read end-to-end for Step 6 final read-through per §5.1 above.
4. **Surface any drift** — flag inconsistencies to Joe before proceeding. If §6 final read-through finds nothing off, proceed to Step 7.
5. **Draft Step 7 PowerShell commands** per §5.2 above + the three companion-file updates (CLAUDE.md PLAY block flip; ROADMAP.md tree + Past + Present + version bump; JOURNAL.md J-098 entry).
6. **Joe runs PowerShell push** — Claude never pushes directly. Confirm clean push.
7. **Session closes** — Clair pickup is the next session's job, not this one.

## 7. Critical context — five most-important framings

If the next session reads nothing else from this note, read these five framings:

1. **Three Joe-locks are settled.** Q3.ii + Q2 middle + Q2.γ + Q1 Shape A v1 + sibling Site 1 fix. The runbook is exposition + sequencing, NOT re-deliberation. Inline-lock pattern (§7.3) governs.
2. **D-076 is the fourth member of the no-drift-surface discipline family.** D-067 + D-070 + D-075 + D-076 across four protocol layers (code-org + transport + event-model + wire-format). Family-completion is structural and the runbook makes the connection visible (§7.5).
3. **This milestone CANNOT close Federation Event Propagation.** Commit 4 closes topological-sort milestone only. Phase 9 Commit 3b unblocks → resumes (Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10 + Phase 9's own milestone-close commit). Federation Event Propagation milestone stays PLAY. M6 + Pass 1 stay PENDING. Three-state-change framing applied at four sites (§2.4 + §6.5 + §6.6 + §6.7 final DoD item).
4. **Four-file runbook-landing commit** (this session's Step 7) is a state-change commit per D-074: runbook + CLAUDE.md PLAY block flip + ROADMAP.md (1.13 → 1.14) + JOURNAL.md J-098.
5. **J-NNN placeholders at three sites** in the runbook freeze together at Clair's Commit 4: canonical-design-doc §15 row + doc-comment in `phase9_two_node_smoke.rs` + catalogue M15 row. All three reference the same J-number (the milestone-close J-NNN, NOT J-098 which is the runbook-landing J-number).

## 8. Files referenced

- **`tasks/FEDERATION_TOPOSORT_IMPL.md`** (this session's drafted artefact; ~93 KB; Status: ACTIVE v1.0) — the in-flight runbook. Authoritative.
- **`tasks/FEDERATION_TOPOSORT_DESIGN.md`** (Status: ACTIVE v1.0) — design task file; three Joe-locks; flips COMPLETED at Clair's Commit 1.
- **`tasks/FEDERATION_TOPOSORT_AUDIT.md`** (Status: ACTIVE v1.0) — audit doc; mechanism evidence at file:line; flips COMPLETED at Clair's Commit 1.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`** (Status: COMPLETED v1.1) — structural sibling-shape precedent.
- **`JOURNAL.md`** — J-097 is the latest entry as of session boundary; verify before drafting Step 7.
- **`CLAUDE.md`** — operational state; PLAY block flips at runbook-landing.
- **`docs/ROADMAP.md`** — version 1.13 at session boundary; bumps to 1.14 at runbook-landing.
- **`docs/xgen_federation_propagation_design.md`** — canonical Federation Event Propagation design; §6.4.3 is the next-free sibling slot verified at runbook authoring (Step 4a).
- **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** — catalogue currently has 14 entries (M1-M14); M15 is next-free for Clair's Commit 4.
- **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** — Phase 9 task file; Status: ACTIVE v1.0; Commit 3b unblocks at Clair's Commit 4.

---

*End of session-handoff note. Next session re-enters at Step 6 (final read-through) per §5.1, then Step 7 (PowerShell push instructions) per §5.2. Status flips ACTIVE → COMPLETED when the next session's Step 7 ships and this note becomes historical record of the in-flight session boundary.*  
