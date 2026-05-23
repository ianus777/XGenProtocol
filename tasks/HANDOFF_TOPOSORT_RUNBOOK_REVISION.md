# Handoff — Topological-sort Runbook Revision (Step 3 of Shape 2 Procedure)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-22 (Step 3 of the post-J-098 design-phase re-walk SHIPPED. Five-file atomic commit per D-074 lands the runbook revision: `tasks/FEDERATION_TOPOSORT_IMPL.md` v1.0 → v1.1 with new §4a Commit 2a section + eleven local amendments end-to-end; this HANDOFF Status flipped ACTIVE → COMPLETED v1.1; JOURNAL J-100 entry recording Step 3's close + the Step-2-bis fix-up atom retrospective; CLAUDE.md PLAY block flipped from "Step 3 runbook revision authoring ←── HERE" to "Clair pickup at Commit 2a → Commit 3 → Commit 4 ←── HERE" + header bump; docs/ROADMAP.md v1.15 → v1.16. **Naming decision at Step 3 open:** the HANDOFF body below uses "Commit-2-amendment" throughout; the actual shipped naming in the revised runbook is **"Commit 2a"** (Joe-locked at the start of the Step 3 session on three grounds: (a) honest about sequence — lands between Commit 2 and Commit 3, letter suffix says "after Commit 2, before Commit 3" structurally; (b) table-friendly — fits cleanly in §2.1's table column without wrapping; (c) precedent fit — "Commit 2a" follows letter-suffix-insertion shape rather than "Commit 2.5" decimal-sibling shape, which was the bidirectional precedent's posture for a true in-flight sibling fix at the same layer; this case is sequential-insertion-at-different-layer, not sibling-half-step). The HANDOFF body's "Commit-2-amendment" references below are preserved as authoritative historical record of the Step-2-bis-time naming; runbook revision shipped with "Commit 2a" naming throughout. The naming choice is the only structural-decision call made at Step 3 open beyond the locks already settled at J-098 session close + J-099 Step 2. Per D-065 + D-074 + D-076 v1.1 discipline.) Previous content (Step-2-bis fix-up atom authoring 2026-05-22) preserved authoritative below.) Previous 2026-05-22 update: Authored as Step-2-bis fix-up atom. This file was named in J-099's "Files changed at this commit (eight, atomic per D-074)" enumeration as file #4 (NEW, ACTIVE v1.0) and was named in `tasks/HANDOFF_TOPOSORT_DESIGN_REWALK.md` §8 exit-criteria as a load-bearing Step-2 deliverable, but did not land on disk inside the e0c5d36 atomic commit. `git show --stat e0c5d36` confirms seven files in that commit, not eight; the eighth-file slip surfaced at post-J-099 session-open verification when the entry-point file named in CLAUDE.md PLAY block could not be read from disk. Sibling-shape to J-098's companion-files-deferred-as-chat-prose slip — same prose-then-batch root cause that updated the memory rule "write each file edit to disk via Filesystem:edit_file/write_file BEFORE next — no prose-then-batch (J-098 lesson)." Authored as a single-file fix-up atom rather than rolled into Step 3's substantive commit because the Step-2 HANDOFF v1.1 Status update already claims this file shipped at Step 2; honest framing per D-065 requires surfacing the gap explicitly rather than papering it over inside a downstream commit. No JOURNAL entry for this fix-up — sibling-shape to J-098's housekeeping atom (single sentence inside J-098 framing the slip), this fix-up gets a single sentence inside J-100 when Step 3 closes. Per D-065 + D-074 honest-provenance discipline.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Purpose of this note

Re-entry note for the next Chat Claude + Joe session. Step 2 of the topological-sort design-phase re-walk shipped the canonical-record amendments (audit doc §11 + design doc §11 + DECISIONS.md D-076 in-place amendment + Rule 0 in CLAUDE.md) in an atomic commit per D-074. Step 3 (this note's session-arc) revises the implementation runbook from v1.0 to v1.1 to reflect the amended principle and the Path B fix shape; after Step 3 ships, Clair resumes with the revised runbook in hand.

The procedural shape stays **Shape 2** per Joe-lock at J-098 session close — targeted patch via runbook revision, not full re-author. The runbook's locked four-commit shape (Commit 1 doc-pass; Commit 2 primitive + sibling + unit tests; Commit 3 Phase 9 Scenario 1 lift; Commit 4 milestone close) is preserved; Step 3 inserts a new **Commit 2-amendment** (Path B fix at event-construction layer) between the original Commit 2 and Commit 3, and updates the existing Commit 2 + Commit 3 + Commit 4 sections to reflect amended D-076 + new Commit-2-amendment placement.

## §2 — Status snapshot at handoff time

**Step 2 commit (`e0c5d36`) shipped:**
- Audit doc §11 amendment + header v1.0 → v1.1.
- Design doc §11 amendment + header v1.0 → v1.1 (Q4 + Q1 supplement).
- DECISIONS.md D-076 in-place amendment (new "Amendment (2026-05-22)" subsection between "Decision" and "Originating incident") + header bump.
- JOURNAL J-099 entry (honest framing-gap retrospective + Rule 0 origin story).
- CLAUDE.md Rule 0 added as fourth member of MANDATORY Behaviour rules + PLAY block flipped to Step 3 + header bump.
- ROADMAP.md v1.14 → v1.15.
- `tasks/HANDOFF_TOPOSORT_DESIGN_REWALK.md` Status flipped ACTIVE → COMPLETED v1.1.

**Step 2 commit gap:** This file (`tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md`) was named in J-099's eight-file enumeration but did not land. Surfaced at post-J-099 session-open verification. Authored as Step-2-bis fix-up atom (this commit, single-file). See header `Last updated` paragraph for honest framing.

**Clair's stand-down:** complete. Clair was sent the stand-down message at J-098 session close. She is awaiting Step 3's runbook v1.1 before resuming.

**Clair's Commit 3 working tree state (uncommitted, left as sentinel per Joe-lock):**
- `xgen-node/src/tests/phase9_two_node_smoke.rs` — doc-comment rewritten per runbook §5.5 + `#[ignore]` removed. Working tree has these changes uncommitted. The doc-comment text is currently forward-looking (describes the fix as landed) which is false but expected; do NOT `git restore` — leave as sentinel signalling "in-flight, not closed."

**JOURNAL state:** J-099 is the latest entry (this Step-2 fix-up atom adds no JOURNAL entry per honest-provenance precedent; sibling-shape to J-098's housekeeping atom which got a single sentence inside J-098). Next-free is **J-100** (Step 3's milestone-event entry).

**ROADMAP state:** v1.15. Tree shows topological-sort cluster with Step 1 ✅ + Step 2 ✅ + Step 3 🟢 frontier. After Step 3 ships, the tree needs Step 3 → ✅ + frontier moves to Clair's Commit-2-amendment.

**CLAUDE.md PLAY block:** currently describes Step 3 as the named-active work with a five-item numbered list. After Step 3 ships, PLAY block needs a flip to "Clair pickup at Commit-2-amendment → Commit 3 → Commit 4 ←── HERE."

## §3 — What the Step 3 session must produce

**Five-file atomic commit per D-074, sibling-shape to the J-099 Step-2 commit (seven files actual, eight named) but smaller (five files):**

1. **`tasks/FEDERATION_TOPOSORT_IMPL.md`** — v1.0 → v1.1. The substantive revision. Add a new **Commit 2-amendment** section between the existing Commit 2 and Commit 3 sections (suggested placement in §2.1 table as a new row "2.5" or "2a"; full per-commit section between §4 and §5). The Commit 2-amendment section covers:

   - **What changes:** `xgen-core/src/space/state.rs:797` — modify `build_room_create_event` to set `prev_events: vec![space_id.to_string()]`. The function's own doc-comment already claims `space_id` is *"the event_id of the parent state.space_create"* — Path B makes the event-DAG honestly reflect what the doc-comment claims.
   - **Why this Commit specifically:** The original Commit 2's Shape A v1 sort fix shipped at e0c5d36's parent commits (`0543a86` per CLAUDE.md DONE-IN-FLIGHT block) is correct under D-076 v1's stated contract (byte-identical wire output across senders) but does not close Phase 9 Scenario 1, because D-076 v1 did not name the second load-bearing property — causal-DAG-respecting order. D-076 v1.1 (amended at Step 2) makes both properties explicit; Path B is the construction-layer fix that satisfies the second property.
   - **Why this scope:** narrow by Joe-lock — `build_room_create_event` only. Sibling event constructors (`state.federation_add`, `membership.*`, `message.*`) are NOT in scope for this amendment. May surface later as their own audit-precedes-dependent-design arc per D-071 if dependent work surfaces need. The amendment carries the same scope-honesty paragraph the audit + design amendments carry.
   - **New unit test:** `room_create_event_records_space_create_as_predecessor` in `xgen-core/src/space/state.rs::mod tests` (or wherever `build_room_create_event`'s existing tests live — Clair verifies at write time). The test asserts that `build_room_create_event(space_id, ...)` produces an Event whose `prev_events` is exactly `vec![space_id.to_string()]`, not empty. Structural sibling to bidirectional's `apply_federation_add_two_vantages_mirror` — a unit-level regression lock that catches Path B regression before it reaches integration tests.
   - **Verbatim code-comment block:** placed at the `prev_events: vec![space_id.to_string()]` line. Locked content includes: (a) D-076 v1.1 reference; (b) Path B citation by tasks/FEDERATION_TOPOSORT_DESIGN.md §11; (c) the doc-comment-was-already-correct framing ("the function's doc-comment already claims this; the construction is now honest about it"); (d) narrow-scope note ("sibling event constructors not audited at this milestone — D-071 if dependent work surfaces need"). Exact wording is Clair's draft at write time; structural elements above are locked.

   **The existing Commit 2 section stays.** It is not reverted. Commit 2's Shape A v1 sort fix at `xgen-node/src/fanout.rs:193` + `:321` lands as already-shipped (the section's status moves to "shipped at `0543a86`"); the section becomes historical-record-of-what-shipped rather than future-action. The two layers (causality at Commit-2-amendment, determinism at Commit-2-already-shipped) layer cleanly per amended D-076.

2. **`tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md`** (this file) — Status flipped ACTIVE → COMPLETED v1.1. Same shape as `HANDOFF_TOPOSORT_DESIGN_REWALK.md` Status flip at Step-2 close.

3. **`JOURNAL.md` J-100 entry** — honest retrospective. Sibling-shape to J-097 (design-phase close) and J-098 (runbook-landing) — milestone-procedural entry recording Step 3's runbook revision close. Must include:
   - The Step-2-bis fix-up atom mention (single sentence framing the eighth-file slip + this fix-up atom as the honest correction; sibling-shape to J-098's first-commit slip + housekeeping atom).
   - The Commit 2-amendment placement (between original Commit 2 and original Commit 3 in the runbook's locked four-commit shape, becoming a five-commit shape post-revision).
   - Confirmation that Clair's stand-down ends with this commit; she picks up at Commit-2-amendment per the revised runbook.
   - D-074 application count: J-100 is the sixth instance.

4. **`CLAUDE.md` PLAY block** — flip from "Step 3 runbook revision authoring ←── HERE (Chat Claude + Joe next session)" to "Clair pickup at Commit-2-amendment → Commit 3 → Commit 4 ←── HERE" + header `Last updated` bumped. PLAY block content describes the revised five-commit Clair sequence with Commit-2-amendment between original Commit 2 (now historical) and Commit 3 (Phase 9 Scenario 1 lift, post-Path-B).

5. **`docs/ROADMAP.md`** — v1.15 → v1.16. Past gains a Step 3 paragraph (runbook revision shipped). Present rewritten to reflect Clair-active state (Track 2 active; Track 1 standby). Tree's topological-sort cluster: Step 3 row 🟢 → ✅; frontier annotation moves to Clair's Commit-2-amendment.

## §4 — What this session must NOT produce

**The runbook revision is the substantive deliverable.** Other documents (audit, design, DECISIONS.md) were amended at Step 2 and are not touched at Step 3. If a drift surface appears during Step 3 authoring (e.g., the canonical design doc §6.4.3 sibling subsection now drifts from the amended design doc §11), that's a Rule 3 stop-and-surface moment for Joe — not silent re-amendment.

**No code touches in Step 3.** The runbook tells Clair *what* to code at Commit-2-amendment; the code itself is Clair's deliverable in a later session-arc.

**No Clair-side commits at Step 3.** Clair's working tree (`xgen-node/src/tests/phase9_two_node_smoke.rs`) stays uncommitted as sentinel through Step 3. Step 3 produces the revised runbook; Clair acts on it after.

**No new D-NNN promotions at Step 3.** D-076 v1.1 (the amendment) was promoted at Step 2. The no-drift-surface discipline family stays at four members (D-067 + D-070 + D-075 + D-076).

**No JOURNAL entry beyond J-100.** J-100 covers the runbook revision close. The Step-2-bis fix-up atom (this file's authoring) gets a single sentence inside J-100, not its own J-NNN entry — sibling-shape to J-098's housekeeping atom precedent.

## §5 — Reading order for re-entry

1. **This note** (you're reading it).
2. **CLAUDE.md PLAY block** — read the §1-§5 numbered Step-3 work plan; it's the load-bearing spec for what this session ships.
3. **`JOURNAL.md` J-099 entry** — context on Step 2's canonical-record amendments + Rule 0 origin story + the framing gap Q3 missed.
4. **`tasks/HANDOFF_TOPOSORT_DESIGN_REWALK.md`** — the closed Step-2 HANDOFF; sibling-shape precedent for this HANDOFF's §3/§4/§8 structure.
5. **`tasks/FEDERATION_TOPOSORT_IMPL.md`** v1.0 — the runbook to be revised. Read §1 (purpose) + §2 (sequence overview) + §3 (Commit 1) + §4 (Commit 2) + §5 (Commit 3) + §6 (Commit 4) end-to-end to understand the existing four-commit shape before revising.
6. **`tasks/FEDERATION_TOPOSORT_DESIGN.md`** §11 — the amended design doc; Q4 (causal-DAG-respecting order) + Q1 supplement (Path B at event-construction layer).
7. **`DECISIONS.md` D-076** — the amended decision; v1 prose stays authoritative; "Amendment (2026-05-22)" subsection names the second load-bearing property.
8. **`xgen-core/src/space/state.rs:797`** — the function the Commit-2-amendment modifies. Read `build_room_create_event` definition + its doc-comment to confirm the lie (doc-comment claims `space_id` is "the event_id of the parent state.space_create" but constructs `prev_events: vec![]`).

## §6 — Locks already in place (do not re-litigate)

These were settled at the J-098 session close + Step 2 commit. Step 3 implements them via runbook revision; it does not re-walk them.

- **Path B is the fix shape.** Event-construction layer fix at `build_room_create_event`, NOT additional sort-layer refinement (Path A rejected at J-098 + recorded canonically at design doc §11).
- **Path B scope is narrow.** `build_room_create_event` only. Sibling event constructors not in scope (Q1=(a)). Surface in Commit-2-amendment scope-honesty paragraph; sibling event constructors deferred to D-071 audit arc if dependent work surfaces need.
- **D-076 amended in place (not D-076 + D-077).** One principle, two complementary properties. The amendment lives in DECISIONS.md between "Decision" (v1) and "Originating incident" subsections.
- **Commit 2's Shape A v1 sort fix stays useful.** Determinism layer beneath the causality layer; not reverted, not modified; the runbook's existing Commit 2 section becomes historical-record-of-what-shipped.
- **Four-commit shape becomes five-commit shape.** Commit-2-amendment inserts between original Commit 2 and original Commit 3; the four original commits are not renumbered (Commit 1, Commit 2, Commit-2-amendment, Commit 3, Commit 4).

If any of these surface as unsettled during Step 3 authoring, that's a Rule 3 stop-and-surface moment, not a free-form re-walk.

## §7 — Discipline reminders for the Step 3 session

- **Write each file edit to disk via `Filesystem:write_file` or `Filesystem:edit_file` BEFORE moving to the next file's draft.** Locked memory rule from J-098 + J-099 Step-2-bis. Prose-then-batch defers tool calls past the confirmation-trigger point; the user assumes drafted content has landed when it has not. Safe pattern: one tool call per file edit, in sequence, with diff visible after each, before moving to the next.
- **Verify new files via `Filesystem:get_file_info` after write.** Sandbox-vs-user-disk confusion is a known failure mode (J-099 Step-2-bis lesson). `Filesystem:write_file` writes to user's disk; verify by reading file metadata.
- **Same-commit discipline per D-074.** All five file modifications (revised runbook + this HANDOFF closed + J-100 + CLAUDE.md PLAY flip + ROADMAP bump) ride in one atomic commit. Do not split into multiple commits.
- **PowerShell push per project convention.** Explicit `git add <file>` per modified file (never `git add .`), `git status` sanity-check before commit, multi-paragraph commit message via multiple `-m` flags, push only after sanity-check.
- **D-065 honest-behaviour discipline applies.** Step 3 ships the runbook revision that closes the framing gap Step 2 amended in the canonical record. The authoring should mirror that honesty (the new Commit 2-amendment section names what was missing in the v1 runbook explicitly; J-100 names the eighth-file slip explicitly).
- **Rule 0 applies.** Step 3 session-open reads CLAUDE.md PLAY block + J-099 + this HANDOFF + then the runbook v1.0 in that order — not the runbook alone.

## §8 — Exit criteria for Step 3

Step 3 closes when:

- [ ] `tasks/FEDERATION_TOPOSORT_IMPL.md` v1.0 → v1.1 with new Commit-2-amendment section landed on disk.
- [ ] `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md` (this file) Status flipped ACTIVE → COMPLETED v1.1.
- [ ] `JOURNAL.md` J-100 entry landed on disk (includes Step-2-bis fix-up atom mention).
- [ ] `CLAUDE.md` PLAY block flipped to "Clair pickup at Commit-2-amendment → Commit 3 → Commit 4 ←── HERE" + header `Last updated` bumped.
- [ ] `docs/ROADMAP.md` v1.15 → v1.16 with Past entry + Present updated for Clair-active state.
- [ ] Five-file atomic commit pushed per D-074.

After Step 3 closes, Clair resumes at Commit-2-amendment. The four-commit shape becomes five-commit shape; the milestone closure dependency chain stays unchanged otherwise.

## §9 — Out-of-scope reminders (do not slide into these during Step 3)

The "lose ourselves" concern stays in force. The bounded outputs above are the structural protection. These specifically are out of scope:

- Auditing sibling event constructors (`state.federation_add`, `membership.*`, `message.*`) for similar `prev_events` lies — deferred to D-071 arc if dependent work surfaces need; do not audit now.
- Writing Clair's actual Commit-2-amendment code (Clair's job after Step 3 ships).
- Re-walking the audit + design + DECISIONS.md amendments (Step 2's job, already shipped at `e0c5d36`).
- Touching code in `xgen-core/` or `xgen-node/` (no code changes in Step 3; runbook revision only).
- Promoting new D-NNN decisions (none needed; D-076 v1.1 + Rule 0 cover the surface).
- Walking new questions beyond the locked Q1–Q4 (Q4 closed the framing gap at Step 2; do not open Q5/Q6 unless one genuinely surfaces and that surface is a Rule 3 stop-and-surface moment, not a drift moment).

---

## §10 — One-line summary

**Step 3 produces a runbook revision (v1.0 → v1.1) inserting a Commit-2-amendment between original Commit 2 and original Commit 3, in a five-file atomic commit; the existing Commit 2 stays as historical-record-of-what-shipped; Clair's stand-down ends with this commit; she picks up at Commit-2-amendment per the revised runbook.**
