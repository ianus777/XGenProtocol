# Handoff — Topological-sort Runbook Revision (Step 3 of Shape 2 Procedure)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-22 (Authored as Step-2-bis recovery atom — surfaced post-J-099 by FC (future Chat Claude session) on `git show --stat` review: the Step-2 atomic commit `e0c5d36` enumerated eight files in J-099's "Files changed at this commit" list, but the file did not land on disk; git showed seven files in the commit, not eight. The failure mode was NOT prose-then-batch (memory rule #16 from J-098's slip — write each file to disk before next, one tool call per file). The failure mode at this commit was tool-routing confusion: Chat Claude issued `create_file` (Claude's container-filesystem sandbox tool, paths under `/mnt/user-data/...`) rather than `Filesystem:write_file` (user's disk, paths under `E:\Projects\XGenProtocol\...`). The `create_file` call returned success — to a sandbox the user cannot see. Memory rule #16 did not catch the slip because the rule's wording covered prose-then-batch (defer-to-batch failure), not tool-routing-confusion (wrong-filesystem failure). Discipline lesson extended at this recovery: verify any new file actually landed on the user's filesystem via `Filesystem:get_file_info` (or equivalent) immediately after the write call — success returned by `create_file` to Claude's sandbox is not success on the user's disk. Sibling-shape to J-098's companion-file slip in the discipline-failure-surfaces-and-becomes-explicit pattern, but at a different layer (J-098 = prose-then-batch deferral; J-099 = tool-routing confusion). Per D-065 honest-behaviour-over-polite-behaviour, this Last-updated paragraph records the slip explicitly rather than papering it over by silently appearing in Step 3's atomic commit. The Step-2 atomic-commit count is honestly seven-files-of-eight-named-in-J-099-plus-this-recovery-atom, not eight-as-enumerated; J-099 stays as written and authoritative for the canonical record (the eight-file enumeration was the intended commit; the recovery atom restores honest count). No JOURNAL entry written for this recovery — J-100 stays reserved for Step 3 close per the existing plan; this slip is recorded inline in the HANDOFF header per D-065 framing-the-slip discipline. Memory rule #16 should be extended to cover tool-routing-verification in a follow-up edit after this atom ships.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Purpose of this note

Re-entry note for the next Chat Claude + Joe session. Step 2 of the post-J-098 design-phase re-walk closed at commit `e0c5d36`: the four canonical-record amendments (audit doc §11 + design doc §11 + DECISIONS.md D-076 in-place amendment + JOURNAL J-099) shipped together with the cross-doc state changes (CLAUDE.md PLAY block flip + Rule 0 addition; ROADMAP.md v1.14 → v1.15; Step-2 HANDOFF Status flip; this Step-3 HANDOFF intended-but-deferred-to-Step-2-bis-recovery). The principle now has its second load-bearing property named explicitly; Path B at `build_room_create_event` is the locked fix shape.

Step 3 revises the implementation runbook to reflect the amended D-076 principle and the new Commit-2-amendment that ships the Path B fix. Clair stays stood down until Step 3 closes.

The procedural shape is **Shape 2** per Joe-lock at J-098 session close — targeted patch, not full re-walk. Step 3 amends `tasks/FEDERATION_TOPOSORT_IMPL.md` in place rather than re-authoring from scratch.

## §2 — Status snapshot at handoff time

**Commits already pushed in the topological-sort milestone's arc (Clair-side):**
- Commit 1 (doc-pass, audit + design Status flips, canonical design doc §6.4.3 + §15 row).
- Commit 2 (primitive fix at `xgen-node/src/fanout.rs:193` + sibling Site 1 fix at `:321` + unit tests including the wire-order-determinism witness `compute_federation_delta_byte_identical_across_two_senders`).

**Commit 3 working tree state (uncommitted, left as sentinel per Joe-lock at J-098 session close):**
- `xgen-node/src/tests/phase9_two_node_smoke.rs` — doc-comment rewritten per runbook v1.0 §5.5 + `#[ignore]` removed. Working tree has these changes uncommitted. The doc-comment text is currently forward-looking (describes the fix as landed) which is false; do NOT `git restore` — leave as sentinel signalling "in-flight, not closed."

**Step-2 commit shipped (this re-entry's predecessor session-arc) — honest provenance:**
- Eight-file atomic commit intended per D-074: audit doc §11 amendment + header v1.0 → v1.1; design doc §11 amendment + header v1.0 → v1.1; DECISIONS.md D-076 in-place amendment + header bump; JOURNAL.md J-099 entry; CLAUDE.md Rule 0 addition + PLAY block flip + header bump; ROADMAP.md v1.14 → v1.15; Step-2 HANDOFF Status flip ACTIVE → COMPLETED; this Step-3 HANDOFF authored ACTIVE.
- Actual shipped count: seven files. This HANDOFF was authored via `create_file` (Claude's sandbox-side tool) rather than `Filesystem:write_file` (user's disk), so it never landed on the user's filesystem; the commit closed without it. Surfaced post-commit by FC.
- This file (`tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md`) lands at Step-2-bis recovery atom as its own honest single-file commit, framed in the Last-updated paragraph above. Atomic-commit count is honestly seven-of-eight-named-in-J-099 + one-file-recovery, not silently-eight.

**Clair's stand-down:** continues. Clair is awaiting Step 3's runbook v1.1 before resuming.

**JOURNAL state:** J-099 is the latest entry (Step-2 atomic commit + Rule 0 origin-story retrospective). Next-free is **J-100** (Step 3's milestone-event entry). J-100 stays reserved for Step 3 — no JOURNAL entry for this recovery atom per the existing plan and D-065 framing-the-slip-inline-rather-than-inflating-the-numbered-record convention.

**ROADMAP state:** v1.15. Tree topological-sort cluster reflects the in-flight design-phase re-walk; Step 3 may want a small tree annotation when it ships.

**CLAUDE.md PLAY block:** currently reads "Step 3 runbook revision authoring ←── HERE next." After Step 3 ships, PLAY block needs a flip to "Clair pickup at Commit 2-amendment ←── HERE" (or whatever shape the revised runbook's Commit sequence makes appropriate).

**CLAUDE.md Rule 0 landed at Step 2.** Mandatory session-open reading sequence is now permanent project discipline. Step 3's session opens under that rule.

## §3 — What this session must produce

**Atomic commit per D-074, sibling-shape to the Step-2 commit (eight files) but expected smaller (~five files):**

1. **`tasks/FEDERATION_TOPOSORT_IMPL.md`** — Status stays ACTIVE; Version 1.0 → 1.1 amendment. The runbook is revised in place to reflect the amended D-076 principle and the Commit-2-amendment shape Clair will ship. Required content updates:
   - **§1 + §2 overview** — name that the runbook is now v1.1 reflecting the post-J-098 amendment; cross-reference audit doc §11 + design doc §11 + DECISIONS.md D-076 amendment + JOURNAL J-099 as the authoritative canonical record.
   - **New Commit 2-amendment section** — inserted between existing §4 (Commit 2 detail) and §5 (Commit 3 detail). Authors the Path B fix at `xgen-core/src/space/state.rs:797`: change `prev_events: vec![]` to `prev_events: vec![space_id.to_string()]` in `build_room_create_event`. Verbatim code-comment block citing D-076 v1.1 amendment + the function's existing doc-comment. New unit test pinning the causality contract (suggested name: `room_create_event_records_space_create_as_predecessor`; sibling-shape to bidirectional's `apply_federation_add_two_vantages_mirror`).
   - **§5 (Commit 3) revision** — doc-comment text on `two_node_federation_push_smoke_100_messages` rewritten to acknowledge both fixes (Commit 2's sort + Commit-2-amendment's Path B). Verification rigour stays at 5 isolated + 3 workspace = 8 green runs; updated to reflect Path B as the substantive fix that closes Scenario 1.
   - **§6 (Commit 4) revision** — milestone-close commit's file list extended to include any Step-3-introduced surfaces (e.g., catalogue row M15 already-locked at v1.0 stays; new catalogue framing for the Path B fix may want a sibling row M16 or expansion of M15's Detection column).
   - **§7 + §8 + §9 + §10** — discipline notes / DoD / cross-references updated to reflect amended D-076 + Step-2 amendments in the canonical record.

2. **`tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md`** (this file) — Status flipped ACTIVE → COMPLETED at Step 3 close.

3. **`JOURNAL.md`** — new J-100 entry recording Step 3's runbook revision close. Sibling-shape to J-099 but shorter (Step 3 is operational, not discipline-failure-retrospective). Quote the revised runbook's table of contents / Commit sequence; cite the four amended canonical-record sources from Step 2.

4. **`CLAUDE.md`** — PLAY block flipped from "Step 3 runbook revision authoring ←── HERE" to "Clair pickup at Commit 2-amendment ←── HERE." Header `Last updated` bumped.

5. **`docs/ROADMAP.md`** — small update. Version bump v1.15 → v1.16. Past gets a Step-3 paragraph. Present updated to reflect Clair-active state.

Five files. The Step 3 commit is honestly smaller than Step 2 because the amended-runbook content is in scope; the cross-doc surface changes are minimal.

## §4 — What this session must NOT produce

**Audit doc + design doc + DECISIONS.md D-076 stay untouched in Step 3.** Those are the canonical-record amendments from Step 2; Step 3 reads them as authoritative input and references them, does not re-amend them.

**No code touched.** `xgen-core/src/space/state.rs:797` stays unchanged in Step 3; Clair ships the Path B fix at Commit-2-amendment after Step 3's revised runbook lands.

**Clair's Commit 3 working tree stays as sentinel.** `xgen-node/src/tests/phase9_two_node_smoke.rs` stays uncommitted. Clair handles it when she resumes.

**CLAUDE.md Rule 0 stays as Step 2 authored it.** No re-edit; the rule lands once.

## §5 — MANDATORY session-open reading sequence (Rule 0)

**This section is the operational restatement of CLAUDE.md Rule 0.** On any session open, the FIRST reads are always:

1. **CLAUDE.md PLAY block** — the load-bearing operational-state anchor for the session.
2. **Latest JOURNAL entry** — context on the session that produced the current PLAY-block state (often surfaces operational details the PLAY block summarises).
3. **Any ACTIVE HANDOFF notes in `tasks/`** — including this file, if the session opens against the topological-sort milestone.
4. **Then whatever document Joe pointed the session at** — the runbook, an audit, a design task file, code surfaces, etc.

This holds regardless of what filename or topic the session opens with. A narrow pointer ("read X") is treated as "expand to context per Rule 0, THEN read X."

**Why this is the load-bearing protection against the failure mode this milestone surfaced.** The post-J-098 session opened with a runbook pointer (the user pasted only `HANDOFF_TOPOSORT_DESIGN_REWALK.md`'s filename, no surrounding context — or, prior to that, only the runbook filename). A narrow-reading interpretation of "read this in isolation" bypassed the bridges (PLAY block + JOURNAL + HANDOFF) that the project's structural defences exist to provide. The runbook v1.0 was partially superseded at session-open time; reading it as ground truth produced an offer to do work that was two commits stale and missed the Path B Joe-lock entirely.

Rule 0 makes the session-open reading sequence permanent project discipline rather than tacit expectation. Sibling-shape to how D-076 v1.1's amendment in Step 2 made the second load-bearing property explicit: v1 contract written → gap surfaced at implementation → amendment makes the missing property explicit. Same pattern at the meta-level for session-open discipline.

**For this specific Step-3 session opening:** read CLAUDE.md PLAY block first (it'll point at this Step-3 HANDOFF as ACTIVE); read J-099 second (Step-2's retrospective covering Rule 0's origin story); read this HANDOFF third; then proceed to revise the runbook per §3.

## §6 — Reading order for re-entry to Step 3

After Rule 0's mandatory sequence (CLAUDE.md PLAY → J-099 → this note), the substantive references for Step 3 work:

1. **`tasks/FEDERATION_TOPOSORT_AUDIT.md` §11** — the v1.1 amendment naming the framing gap. Read for the design-phase Q3 framing miss + Path B locked fix shape.
2. **`tasks/FEDERATION_TOPOSORT_DESIGN.md` §11** — the v1.1 amendment naming Q4 (causal-DAG-respecting order as load-bearing property) + Q1 supplement (Path B at event-construction layer). Read for the locked-decision exposition + rejected alternatives at re-walk.
3. **DECISIONS.md D-076 Amendment subsection** — the in-place amendment of D-076 inserted between "Decision" and "Originating incident." Read for the canonical principle as it stands post-amendment.
4. **`tasks/FEDERATION_TOPOSORT_IMPL.md` v1.0** — the runbook to be revised. Read fresh; the v1.0 content's Commit-2 sort fix sequencing is preserved, Commit-2-amendment is the new insertion.
5. **`xgen-core/src/space/state.rs:797`** `build_room_create_event` — read the function body + its own doc-comment to confirm the construction site is as described. Then read sibling `build_space_create_event` for shape contrast.
6. **`xgen-node/src/tests/phase9_two_node_smoke.rs`** — the Commit 3 sentinel-state working tree. Read the current uncommitted doc-comment to plan the Step-3 rewrite that will land at Clair's Commit 3.

## §7 — Locks already in place (do not re-litigate)

These were settled at J-098 session close and Step 2. Step 3 implements them in the runbook; it does not re-walk them.

- **Path B is the fix shape.** Event-construction layer fix at `build_room_create_event` only; Path A (EventType-priority sort) and Path C (broader sibling-constructor audit) rejected.
- **Path B scope is narrow.** `build_room_create_event` only. Sibling event constructors not in scope.
- **D-076 amends in place.** One principle, two complementary properties. No D-077 sibling.
- **Commit 2's sort fix stays useful.** Determinism layer beneath the causality layer; not reverted, not modified.
- **Rule 0 stays as Step 2 authored it.** Mandatory session-open reading sequence is permanent project discipline.

If any of these surface as unsettled during Step 3 authoring, that's a Rule 3 stop-and-surface moment, not a free-form re-walk.

## §8 — Commit-2-amendment shape (locked content for the runbook revision)

Step 3's substantive content addition to the runbook is the new Commit-2-amendment section. Locked content for that section, recorded here so Step 3's authoring has clear input:

**Commit-2-amendment scope:** Path B fix at `xgen-core/src/space/state.rs:797` + new unit test pinning the causality contract.

**Code change (single file, single edit):**
- `build_room_create_event` at `xgen-core/src/space/state.rs:797` — change `prev_events: vec![]` to `prev_events: vec![space_id.to_string()]`.
- Verbatim code-comment block at the change site: citing D-076 v1.1 amendment ("causal-DAG-respecting order as load-bearing property"); naming the function's own doc-comment's "`space_id` is the event_id of the parent state.space_create" claim that the v1 implementation contradicted; explicit Pass 3 retype marker (when xgen-core widens dispatch to XGID flavours, the `space_id.to_string()` collapses to the typed form).

**New unit test (suggested name `room_create_event_records_space_create_as_predecessor`):**
- Constructs a `state.space_create` event with known event_id.
- Calls `build_room_create_event` with that event_id as `space_id`.
- Asserts the returned event's `prev_events` is `vec![space_id_string]`, not `vec![]`.
- Sibling-shape to bidirectional's `apply_federation_add_two_vantages_mirror` (the unit-level regression lock for D-075). This test is the unit-level regression lock for D-076 v1.1's causality property.

**Test placement:** in-module `#[cfg(test)] mod tests` block in `xgen-core/src/space/state.rs`; sibling to existing `build_*_event` tests in the same module.

**Test count delta:** +1 unit test against Clair's post-Commit-2 baseline.

**Phase 9 Scenario 1 lift posture:** stays at Clair's Commit 3 (now Commit-3-after-Commit-2-amendment). Lift still requires the 5 isolated + 3 workspace = 8 green runs verification rigour from runbook v1.0 §5.3.

## §9 — Discipline reminders for the Step 3 session

- **Write each file edit to disk via `Filesystem:edit_file` (or `Filesystem:write_file` for new files) before moving to the next file's draft, not as prose-then-batch.** This is the J-098 discipline lesson plus the J-099 tool-routing-verification extension this HANDOFF's authoring just surfaced. Step 2 followed the prose-then-batch rule cleanly for the seven files that did land; the eighth file (this one) was lost to tool-routing confusion. Step 3 must verify each file via `Filesystem:get_file_info` immediately after each write so the same failure mode does not recur. Pattern: one tool call per file edit → verify via `get_file_info` → only then move to next file.
- **Same-commit discipline per D-074.** All five file modifications (runbook + this HANDOFF flip + JOURNAL J-100 + CLAUDE.md + ROADMAP.md) ride in one atomic commit. Do not split.
- **PowerShell push per project convention.** Explicit `git add <file>` per modified file (never `git add .`), `git status` sanity-check before commit, multi-paragraph commit message via multiple `-m` flags, push only after sanity-check.
- **D-065 honest-behaviour discipline applies.** Step 3 records the runbook revision as honest amendment, not as if v1.0 was always correct.

## §10 — Exit criteria for Step 3

Step 3 closes when:

- [ ] `tasks/FEDERATION_TOPOSORT_IMPL.md` v1.0 → v1.1 revision landed on disk with Commit-2-amendment section + cross-references to Step-2 canonical-record amendments + §5/§6/§7 updates reflecting amended D-076.
- [ ] `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md` (this file) Status flipped ACTIVE → COMPLETED.
- [ ] `JOURNAL.md` J-100 entry written.
- [ ] `CLAUDE.md` PLAY block flipped to "Clair pickup at Commit 2-amendment ←── HERE" + header `Last updated` bumped.
- [ ] `docs/ROADMAP.md` v1.15 → v1.16 with Past entry + Present updated.
- [ ] Five-file atomic commit pushed per D-074.
- [ ] Clair receives the resume signal (a small ping noting v1.1 runbook is ready; she picks up at Commit-2-amendment).

After Step 3 closes, Clair resumes the implementation arc: ships Commit-2-amendment (Path B fix + new unit test) → ships Commit 3 (Phase 9 Scenario 1 `#[ignore]` lift with 5+3 verification rigour against the amended fix) → ships Commit 4 (milestone close, six files per runbook v1.1).

## §11 — Out-of-scope reminders (do not slide into these during Step 3)

The "lose ourselves" concern still applies. Step 3's bounded outputs are the structural protection. Specifically out of scope:

- Re-amending audit doc, design doc, or DECISIONS.md D-076 (Step 2's work is canonical).
- Auditing sibling event constructors (`state.federation_add`, `membership.*`, `message.*`) for similar `prev_events` lies — narrow-scope honesty note from Step 2's amendments stands.
- Touching code in `xgen-core/` or `xgen-node/` (no code changes in Step 3; canonical record only).
- Walking new questions beyond Q4 (Step 2 closed the framing gap; do not open new questions unless one genuinely surfaces and that surface is a Rule 3 stop-and-surface moment, not a drift moment).
- Re-walking Rule 0's framing (Step 2 locked it; Step 3 reads it as established discipline).

---

## §12 — One-line summary

**Step 3 produces a runbook v1.1 revision + JOURNAL J-100 in a five-file atomic commit; Clair resumes at Commit-2-amendment after Step 3 closes.**  
