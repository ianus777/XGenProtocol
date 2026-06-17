# Handoff — Topological-sort Design Re-Walk (Step 2 of Shape 2 Procedure)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-22 (Status flipped ACTIVE → COMPLETED v1.1 at Step 2 atomic commit. All eight files per §8 DoD shipped in one atomic commit per D-074: audit doc §11 amendment + header v1.0 → v1.1; design doc §11 amendment + header v1.0 → v1.1; DECISIONS.md D-076 in-place amendment + header bump; JOURNAL J-099 entry; CLAUDE.md PLAY block flipped + Rule 0 added (mandatory session-open reading sequence per the §5-promoted framing) + header bumped; ROADMAP.md v1.14 → v1.15; this HANDOFF Status flipped ACTIVE → COMPLETED; new Step-3 HANDOFF authored ACTIVE at `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md`. §8's checkbox list was the load-bearing count (§3 "four-file" and §7 "five files" earlier in this note were drafting residue from earlier plan-shape iterations; the eight-file count is what shipped). Clair stays stood down until Step 3 closes; her Commit 3 working tree (`xgen-node/src/tests/phase9_two_node_smoke.rs`, uncommitted) remains as sentinel per Joe-lock. Previous v1.0 content stands authoritative as the canonical record of the Step-2 work-plan; this Status flip closes the lifecycle. Per D-065 + D-069 + D-071 + D-074 + D-076 discipline.) Previous 2026-05-22 update:  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Purpose of this note

Re-entry note for the next Chat Claude + Joe session. The topological-sort milestone surfaced a design-phase framing gap at Clair's Commit 3 verification: Commit 2's sort fix is correct under D-076's stated contract (byte-identical wire output across senders) but does not close the surfaced bug because the contract did not name the second property the wire order must also satisfy — **causal-DAG-respecting ordering**.

This note exists because the topological-sort milestone is now mid-flight at an unexpected boundary. Clair has stood down. The runbook v1.0 still describes the original Q1 Shape A v1 fix shape and is partially outdated. The audit doc + design doc + DECISIONS.md D-076 all need amendments before the runbook can be revised. Step 2 (this re-entry's session-arc) authors those amendments. Step 3 (a later session-arc) revises the runbook. Clair resumes after Step 3 ships.

The procedural shape is **Shape 2** per Joe's lock at the end of the J-098 session — targeted patch, not full re-walk. Specifically: amend existing canonical-record documents in place rather than re-walking three phases.

## §2 — Status snapshot at handoff time

**Commits already pushed in this milestone's arc (Clair-side):**
- Commit 1 (doc-pass, audit + design Status flips, canonical design doc §6.4.3 + §15 row).
- Commit 2 (primitive fix at `xgen-node/src/fanout.rs:193` + sibling Site 1 fix at `:321` + unit tests including the wire-order-determinism witness `compute_federation_delta_byte_identical_across_two_senders`).

**Commit 3 working tree state (uncommitted, left as sentinel per Joe-lock):**
- `xgen-node/src/tests/phase9_two_node_smoke.rs` — doc-comment rewritten per runbook §5.5 + `#[ignore]` removed. Working tree has these changes uncommitted. The doc-comment text is currently forward-looking (describes the fix as landed) which is false; do NOT `git restore` — leave as sentinel signalling "in-flight, not closed."

**Clair's stand-down:** complete. Clair was sent the stand-down message at the close of the J-098 session. She is awaiting Step 3's runbook v1.1 before resuming.

**JOURNAL state:** J-098 is the latest entry (companion-updates housekeeping atom with discipline retrospective). Next-free is **J-099** (this re-walk's milestone-event entry).

**ROADMAP state:** v1.14. Tree shows topological-sort cluster with Implementation row 🟡 PENDING (Clair pickup); after Step 2 ships, the tree needs a new row or annotation reflecting the in-flight re-walk. Suggestion deferred to Step 2 authoring time.

**CLAUDE.md PLAY block:** currently describes Clair's four-commit sequence as ready-to-pick-up. After Step 2 ships, PLAY block needs a flip to "Step 3 runbook revision authoring ←── HERE."

## §3 — What this session must produce

**Four-file atomic commit per D-074, sibling-shape to the J-097 design-phase-close commit (six files) but smaller (four files):**

1. **`tasks/FEDERATION_TOPOSORT_AUDIT.md`** — Status stays COMPLETED with v1.0 → v1.1 amendment header per D-069 amendment discipline. Append a new section (suggested §3.6 or §11, depending on best fit) titled something like "Framing gap surfaced at implementation: determinism vs. causal correctness." Records the gap honestly. The amendment does not invalidate v1.0 content; it extends it.

   **Must include the narrow-scope honesty note** locked at the J-098-session-close Step 1: "Path B scope is narrow by Joe-lock — `build_room_create_event` only; sibling event constructors (`state.federation_add`, `membership.*`, `message.*`) not audited in this milestone, may surface later as their own audit-precedes-dependent-design arc per D-071."

2. **`tasks/FEDERATION_TOPOSORT_DESIGN.md`** — Status v1.0 → v1.1. Add a new section (suggested §11) titled something like "Q4 (post-lock): causal ordering as a load-bearing property; Q1 supplement with Path B at event-construction layer." Records:
   - The new Q4 framing: "What semantic ordering property must the canonical wire order satisfy?" Answer locked: causal-DAG-respecting order, not merely byte-determinism.
   - The Q1 supplement: Path B is the fix shape at the event-construction layer (fix `build_room_create_event` at `xgen-core/src/space/state.rs:797` to set `prev_events: vec![space_id.to_string()]` so the event-DAG honestly reflects the protocol-level parent-child relationship the function's own doc-comment already claims).
   - The relationship between Commit 2's already-shipped sort fix (still useful — deterministic-across-senders is still a real property) and the new Path B fix (layers above it — causality fix at construction layer).
   - Rejected alternatives at this re-walk: Path A (EventType-priority sort) and Path C (broader re-walk including audit of sibling event constructors). Both considered, both rejected on locked-scope grounds.

3. **`DECISIONS.md`** — D-076 amended in place per Joe-lock (Q2(a) at the J-098-session-close Step 1). Add a paragraph extending the principle to include causal-DAG-respecting wire order alongside byte-identical determinism. The two properties are complementary aspects of the same thing — what makes wire output "correct for the receiver." Splitting them into D-076 + D-077 was considered and rejected on grounds that they cannot vary independently (a causal-but-non-deterministic wire format is broken; a deterministic-but-non-causal wire format is broken). One principle, refined.

4. **`JOURNAL.md` J-099 entry** — honest retrospective. Sibling-shape to J-094's JOURNAL-not-written retrospective and J-098's prose-then-batch discipline-failure retrospective, but framing a different kind of slip: a design-phase Q3 framing gap that surfaced at implementation. Must include:
   - The mechanism evidence (room_create with empty `prev_events` despite doc-comment claiming a parent; lex-by-event_id deterministic-but-not-causal; ~50% failure rate per nonce roll).
   - The Q3 framing gap (audit asked "is determinism normative?" but didn't ask "what semantic property must the canonical order satisfy?").
   - The "honest longer work over fast shortcuts" fifth recurrence within Federation Event Propagation milestone scope — the gap could have been papered over by amending Commit 2 at the sort layer; the honest path is the design-phase amendment + event-construction-layer fix.
   - Clair's stand-down + the Shape 2 procedure decision (re-walk via in-place amendments, not full audit→design→impl re-author).
   - The five-step session arc: Step 1 (Joe-lock conversation at J-098 session close) ✅, Step 2 (this commit) ✅, Step 3 (runbook revision in a future session) 🟡, Clair's Commit-2-amendment + Commit 3 + Commit 4 🟡, milestone close 🟡.

## §4 — What this session must NOT produce

**The runbook is NOT touched in Step 2.** `tasks/FEDERATION_TOPOSORT_IMPL.md` stays at v1.0 until Step 3. Canonical record (audit + design + DECISIONS) gets amended first; runbook follows in its own session-arc. This is the structural protection against drift — bounded outputs per step, hard boundaries between steps.

**CLAUDE.md PLAY block flip is small.** Just update to reflect "Step 2 done; Step 3 runbook revision authoring ←── HERE next." Don't pre-author Step 3's PLAY block details.

**ROADMAP.md update is small.** Version bump v1.14 → v1.15. Past gets a Step 2 paragraph (the design-phase re-walk amendments + D-076 amendment). Present updated to reflect the in-flight re-walk. The tree's topological-sort cluster may want an annotation but no new rows added — the re-walk is a sub-arc within the existing cluster, not a new phase.

**No commit to Clair's working tree.** `xgen-node/src/tests/phase9_two_node_smoke.rs` stays uncommitted as sentinel. Step 3's runbook will tell Clair what to ship there.

## §5 — Reading order for re-entry

1. **This note** (you're reading it).
2. **`JOURNAL.md` J-098 entry** — context on the session that produced the slip surface + the Path B Joe-lock at session close.
3. **The bug evidence document** Clair surfaced (the message starting "Commit 2 shipped. Moving to Commit 3.") — re-read for the mechanism evidence in Clair's own words. If unavailable in chat history, reconstruct from `xgen-node/src/tests/phase9_two_node_smoke.rs` doc-comment current text + `xgen-core/src/space/state.rs:797` `build_room_create_event` definition.
4. **`tasks/FEDERATION_TOPOSORT_AUDIT.md`** — re-read Q3 framing to see where the gap lives in the original audit text.
5. **`tasks/FEDERATION_TOPOSORT_DESIGN.md`** — re-read Q3.ii lock + Q1 Shape A v1 lock to see where the supplement attaches.
6. **`DECISIONS.md` D-076** — re-read current text to see where the amendment attaches.
7. **`xgen-core/src/space/state.rs:797`** `build_room_create_event` — read the function body + its own doc-comment to confirm the lie (function claims `space_id` is "the event_id of the parent state.space_create" but constructs `prev_events: vec![]`).

## §6 — Locks already in place (do not re-litigate)

These were settled at the J-098 session close. Step 2 implements them; it does not re-walk them.

- **Path B is the fix shape.** Event-construction layer fix at `build_room_create_event`, NOT additional sort-layer refinement (Path A rejected).
- **Path B scope is narrow.** `build_room_create_event` only. Sibling event constructors not in scope (Q1=(a)).
- **D-076 amends in place.** One principle, two complementary properties. No D-077 sibling (Q2=(a)).
- **Commit 2's sort fix stays useful.** Determinism layer beneath the causality layer; not reverted, not modified.

If any of these surface as unsettled during Step 2 authoring, that's a Rule 3 stop-and-surface moment, not a free-form re-walk.

## §7 — Discipline reminders for the Step 2 session

- **Write each file edit to disk via `Filesystem:edit_file` before moving to the next file's draft, not as prose-then-batch.** This is the discipline lesson from J-098. Prose-then-batch defers tool calls past the point where confirmation requests trigger, breaking the implicit assumption that drafted content has landed. Safe pattern: one tool call per file edit, in sequence, with the diff visible to Joe after each, before moving to the next.
- **Same-commit discipline per D-074.** All four file modifications + JOURNAL J-099 entry ride in one atomic commit. Do not split into multiple commits.
- **Same-commit discipline includes ROADMAP.md + CLAUDE.md.** Five files actually — the four named above plus ROADMAP.md v1.14 → v1.15 + CLAUDE.md PLAY block flip. (Adjust commit framing accordingly.)
- **PowerShell push per project convention.** Explicit `git add <file>` per modified file (never `git add .`), `git status` sanity-check before commit, multi-paragraph commit message via multiple `-m` flags, push only after sanity-check.
- **D-065 honest-behaviour discipline applies.** Step 2 is itself the response to a design-phase miss; the authoring should mirror that honesty (the audit + design amendments name the gap explicitly; D-076 amendment notes what the v1 text missed).

## §8 — Exit criteria for Step 2

Step 2 closes when:

- [ ] `tasks/FEDERATION_TOPOSORT_AUDIT.md` v1.0 → v1.1 amendment landed on disk.
- [ ] `tasks/FEDERATION_TOPOSORT_DESIGN.md` v1.0 → v1.1 with Q4 + Q1 supplement landed on disk.
- [ ] `DECISIONS.md` D-076 amended in place landed on disk.
- [ ] `JOURNAL.md` J-099 entry landed on disk.
- [ ] `CLAUDE.md` PLAY block flipped to "Step 3 runbook revision authoring HERE" + header `Last updated` bumped.
- [ ] `docs/ROADMAP.md` v1.14 → v1.15 with Past entry + Present updated.
- [ ] Six-file atomic commit pushed per D-074.
- [ ] This handoff note (`tasks/HANDOFF_TOPOSORT_DESIGN_REWALK.md`) Status flipped ACTIVE → COMPLETED at the same commit.
- [ ] A new handoff note for Step 3 authored at `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md` Status: ACTIVE (sibling-shape to this note).

After Step 2 closes, the next session-arc is Step 3 (runbook revision). Clair stays stood down until Step 3 closes.

## §9 — Out-of-scope reminders (do not slide into these during Step 2)

The "lose ourselves" concern Joe named explicitly at the start of the J-098 session close. The bounded outputs above are the structural protection. These specifically are out of scope:

- Auditing sibling event constructors (`state.federation_add`, `membership.*`, `message.*`) for similar `prev_events` lies — narrow-scope honesty note records this as a deferred-not-forgotten concern; do not audit now.
- Revising the runbook (Step 3's job).
- Authoring Clair's Commit 2-amendment instructions (Step 3's job).
- Walking new questions beyond Q4 (Q4 closes the gap; do not open Q5/Q6/etc. unless one genuinely surfaces and that surface is a Rule 3 stop-and-surface moment, not a drift moment).
- Touching code in `xgen-core/` or `xgen-node/` (no code changes in Step 2; canonical record only).

---

## §10 — One-line summary

**Step 2 produces four canonical-record amendments + JOURNAL J-099 in a six-file atomic commit; the runbook stays at v1.0; Clair stays stood down; Step 3 follows in its own session.**
