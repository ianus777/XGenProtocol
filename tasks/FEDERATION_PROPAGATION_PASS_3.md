# Federation Event Propagation — Pass 3 Task

> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-18 (Pass 2 closed; Pass 3 opened as next-session work)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this task is

Pass 3 of the Federation Event Propagation milestone's Joe-locked design phase. The phase started 2026-05-18 with `tasks/FEDERATION_PROPAGATION_DESIGN.md` (Pass 2 task file) and produced one main design doc plus three addenda. Pass 3 closes the design phase.

This task file replaces `tasks/FEDERATION_PROPAGATION_DESIGN.md` as the active design-phase task file. The Pass 2 task file is marked COMPLETED in Pass 3's same-commit work.

**This is NOT implementation work.** No code changes. The deliverables are a canonical document, two documentation corrections in adjacent files, an implementation runbook for Clair, and status flips in CLAUDE.md and ROADMAP.md.

---

## 2. Where Pass 2 left things

Pass 2 ran in conversation over 2026-05-18 and surfaced ten framework decisions, each Joe-locked with an explicit `[JOE-LOCK]` marker. The work split across four files because the main design doc grew too large for full-rewrite-per-edit to remain sustainable:

| File | Contents |
|---|---|
| `docs/xgen_federation_propagation_design.md` | Main design doc (v0.6). Header, §1-3 scaffold, §4 F-1, §5 F-2, §6 F-3, §7 F-4, §8 F-5, §9 F-6. |
| `docs/xgen_federation_propagation_design_F7_addendum.md` | F-7 pagination. |
| `docs/xgen_federation_propagation_design_F8_F9_addendum.md` | F-8 and F-9 documentation correction timing. |
| `docs/xgen_federation_propagation_design_F10_addendum.md` | F-10 DAG hole semantics. |

All four files carry `[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]` markers throughout. Pass 3 promotes them to their final locked state.

### 2.1 Summary of all ten framework decisions

| F-item | Topic | Decision |
|---|---|---|
| F-1 | Push direction | Hybrid (push for steady state, pull for gap recovery) |
| F-1a | Initial handshake | Tip exchange replaces full history dump |
| F-1b | Buffering on peer-down | Drop, recover via pull (Option α) |
| F-1c | Per-peer record | Node-implementation persistent state, global backoff reconnect |
| F-2 | Session model | Long-lived continuous |
| F-2 lifecycle | Session boundaries | Opens on handshake, closes on goodbye/keepalive/error, fresh session on re-establishment |
| F-2a | Session topology per pair | One WebSocket per pair, bidirectional event flow |
| F-3 | Identity authority | Event signature + federation relationship verification (Option 2) |
| F-4 | Validation asymmetry closure | Unified validation core + per-event-type post-validation handlers (Option 1) |
| F-4a | HeldPending timeout for state events | 30s uniform v1, configurable v2 |
| F-4b | Pre-validation check placement | Structural before, semantic after |
| F-5 | Transitive federation | Locked-out v1, Option 3 v2 evolution path documented |
| F-6 | `sync_complete` wire shape | Fold in (`SyncComplete { since, new_tip }`) |
| F-6a | Wire-shape details | `since` echoed, `new_tip` returned |
| F-6b | Safety-net timeout | 5s default, configurable, NOT protocol-fixed |
| F-7 | Pagination on `collect_sync_history` | Fold in (response-size pagination with `continue_from` cursor) |
| F-7a | Page size | 1000 default, configurable via `[sync].batch_size`, NOT protocol-fixed |
| F-8 | Ch4 lines 779/825-827 correction timing | Correct at Pass 3 (same commit as design doc ACTIVE flip) |
| F-9 | `xgen_node_admin_ops_design.md` §4.2 correction timing | Same as F-8 |
| F-10 | DAG hole semantics on validation failure with unknown signer Identity | Extend HeldPending to include unknown signer Identity (Option 2) |
| F-10a | Identity-missing timeout | Same as F-4a (30s uniform v1, configurable v2) |

---

## 3. Pass 3 deliverables

Five deliverables, in execution order. Each is independently verifiable.

### 3.1 Consolidate the design doc

Fold F-7, F-8/F-9, F-10 addendum sections into the main `docs/xgen_federation_propagation_design.md` as §10, §11, §12, §13 respectively.

**Why this happens at Pass 3, not earlier.** The addendum pattern was a Pass 2 efficiency move (full-file rewrite per F-item became disproportionately expensive). Pass 3 has the canonical-document responsibility — when the doc flips to ACTIVE, it has to be one document, not five.

**Specific structural changes to the consolidated doc:**

- Header: bump Version to 1.0 (first canonical version), Status from PENDING to ACTIVE, update Last updated.
- §3.1 (Scope): add the F-7 pagination scope item ("Pagination on `collect_sync_history`" alongside the F-6 sync_complete line). Already mentions F-6.
- §3.3 (Non-scope decisions table): remove the "Pagination — possibly in scope (see F-7)" row (it's now in scope).
- Append F-7 as §10, F-8 as §11, F-9 as §12, F-10 as §13. Renumber the cross-references inside each section if they reference each other.
- Remove the "Pass 2 — remaining framework decisions to surface" section at the bottom (it's now empty / obsolete).
- Add a brief "§14 Pass 3 closure notes" section recording: Pass 2 ran 2026-05-18, all ten F-items locked in conversation, addenda consolidated at Pass 3, runbook follows.

**After consolidation, delete the three addendum files.** They served their purpose; the canonical document is whole.

### 3.2 Walk every `[JOE-LOCK]` marker for final confirmation

Pass 2 marked every framework decision with `[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`. Pass 3 promotes these to their final form. Suggested final marker text:

```
`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`
```

The walk is mechanical — Chat Claude can do it as part of the consolidation in §3.1. Joe does not need to re-approve each marker individually; Pass 2's conversation already secured the approvals.

The walk is also a final sanity check: if any marker still has language that suggests "not yet locked" or "to be decided at Pass 3," that's a discrepancy that needs catching before the Status flip.

### 3.3 Documentation corrections per F-8 and F-9

In the SAME commit that consolidates the design doc and flips its Status to ACTIVE:

**Ch4 corrections (F-8):**
- **Line 779** of `docs/xgen_ch4_implementation.md`: replace the implementation-style description of `transport.sync_request` (which describes the unimplemented `sync_response` and `sync_complete` reply shapes as if implemented) with a forward-reference to the design doc and Ch3 §3.3.6.
- **Lines 825-827** of `docs/xgen_ch4_implementation.md`: replace the description of Node-to-Node sync behaviour (which describes mechanisms that don't exist in production) with a forward-reference to the design doc and acknowledgement that the mechanism is deferred to the federation propagation completion milestone.

Exact phrasing is Chat Claude's call, guided by:
- Forward-reference the canonical design doc (`docs/xgen_federation_propagation_design.md`).
- Acknowledge the deferred implementation state honestly (D-065 alignment).
- Don't describe behaviour as implemented when it isn't.

**Admin-ops doc correction (F-9):**
- **`docs/xgen_node_admin_ops_design.md` §4.2**: replace the description of Node-to-Node federation push (which describes a mechanism that doesn't exist) with a forward-reference to the canonical design doc and acknowledgement that implementation lands in the federation propagation completion milestone.

Same phrasing principles as F-8.

### 3.4 Write the implementation runbook for Clair

Create `tasks/FEDERATION_PROPAGATION_COMPLETION.md` as the Clair-facing runbook for the implementation work. This is the bridge from "design locked" to "code shipped."

**Required sections in the runbook:**

1. **Overview** — what this task implements, with explicit pointer to the canonical design doc as the source of truth.
2. **Test environment** — current state (468 tests, M5/M6 Phase 0/J-080/J-081 all closed), target state (passing tests after each phase).
3. **Phase plan** — the runbook decomposes implementation into phases. Suggested decomposition (Chat Claude refines during runbook authoring):
   - Phase 1: `sync_complete` + pagination wire shape + four-call-site migration (F-6 + F-7). Smaller, self-contained, lands first because subsequent phases depend on it.
   - Phase 2: `process_inbound` validation pipeline unification (F-4). The precondition for federation push. Touches multiple files; significant refactor.
   - Phase 3: Federation handshake reshape to tip exchange (F-1a). Replaces today's history-dump logic in `handle_federation_incoming`.
   - Phase 4: Federation event push (F-1 main + F-1b drop-on-peer-down + F-5 origin gating). The new push mechanism wired at `apply_fanout` sibling site.
   - Phase 5: F-1c per-peer record + reconnect scheduling (Node-implementation, not protocol). Persisted state, global backoff schedule, `run_initiating` gains first production caller.
   - Phase 6: HeldPending generalisation for unknown signer Identity (F-10). Small, builds on Phase 2.
   - Phase 7: Federation-relationship verification gate in `process_inbound` (F-3 second check). Small, depends on Phase 2's pipeline.
   - Phase 8: Documentation pass — Ch3 §3.3.6 reflects shipped `sync_complete`, design-doc cross-references all up to date, audit cross-references corrected.
   - Phase 9: Integration tests for the full federation push path — two-Node smoke, three-Node smoke if affordable, validation-asymmetry regression tests.
4. **Per-phase Definition of Done** — each phase has its own checklist. Phase-level DoD includes: tests passing, JOURNAL entry written, commit pushed by Joe.
5. **Cross-references** — pointers to the design doc, the audit doc, M6 design doc (for envelope-`event_id` coordination), DECISIONS.md.
6. **Operating discipline** — restate the project's mandatory behaviour rules from CLAUDE.md so Clair has them inline.
7. **Test count tracking** — current count is 468; runbook tracks expected count growth per phase.
8. **Validation asymmetry as precondition** — explicitly call out that Phase 2 (validation pipeline) MUST land before Phase 4 (federation push), because otherwise federation push lands a vulnerability. The runbook makes the ordering hard.
9. **D-070 promotion** — if not already done, the runbook flags it as a parallel small task to be done at the same time as design doc ACTIVE flip.
10. **D-071 candidate** — sibling project-management principle ("Subsystem audits precede dependent milestones"). Flag for Joe to promote when convenient.

Don't write the runbook with implementation specifics that overconstrain Clair. The framework is locked (the design doc has it); the implementation has latitude per the M6/M5 precedent ("cleaner is better"). Wire shapes are locked, Rust types are not.

### 3.5 Status flips in CLAUDE.md and ROADMAP.md

In the SAME commit as the consolidation + corrections + runbook:

**CLAUDE.md updates:**
- Header `Last updated` line: add "Federation Event Propagation Pass 3 closed; runbook written; ACTIVE flip pending Clair's start."
- Federation Event Propagation milestone block: flip from 🟡 PENDING to 🟢 ACTIVE.
- The block needs new content: pointer to the runbook (`tasks/FEDERATION_PROPAGATION_COMPLETION.md`) and the canonical design doc.
- Pass 2 task file (`tasks/FEDERATION_PROPAGATION_DESIGN.md`): flip Status to COMPLETED.
- Pass 3 task file (this file): flip Status to COMPLETED at session close.
- Current State section: update to reflect Pass 3 closure + runbook handoff. The roadmap line should now read: "M5 ✅ → CLI Audit ✅ → J-080 ✅ → M6 Phase 0 Pass 3 ✅ → Propagation Reliability Audit ✅ → Federation Event Propagation design ✅ → **Federation Event Propagation implementation** (🟢 ACTIVE next) → M6 (new) → M7 → M8 → M9."

**ROADMAP.md updates:**
- Past section: add the design phase as ✅ DONE under the Federation Event Propagation track.
- Present section: replace "(none currently active)" with the implementation track as 🟢 PLAY (or keep "none" if the runbook handoff to Clair isn't the same as starting implementation — depends on Joe's sequencing call).
- Near future section: F-7-style "Documentation correction pass" and "D-070 promotion" entries — promote them out of pending if they were folded into Pass 3 (F-8/F-9 corrections happen in this same commit; D-070 promotion may or may not).

The two-document-coordination rule (CLAUDE.md and ROADMAP.md updated in the same commit) applies. Both files reflect Pass 3 closure together.

---

## 4. Sequencing within Pass 3

Pass 3 is one logical unit but operationally several distinct steps. Suggested order:

1. **Read all four design-doc files** end to end. The session starts here. Fresh eyes catch inconsistencies Pass 2 missed.
2. **Consolidation pass.** Build the single canonical doc with §10-§13 folded in, header bumped, addenda deleted. This is the largest single write of the session.
3. **`[JOE-LOCK]` marker walk.** Mechanical update of marker text to final form. Done during consolidation; revisit at end of consolidation to confirm all markers are uniform.
4. **Documentation corrections per F-8 and F-9.** Edit `docs/xgen_ch4_implementation.md` and `docs/xgen_node_admin_ops_design.md` per §3.3 above.
5. **Write the runbook.** `tasks/FEDERATION_PROPAGATION_COMPLETION.md` per §3.4 above.
6. **Status flips.** CLAUDE.md and ROADMAP.md per §3.5 above. Both in the same commit as the rest.
7. **One coordinated commit** with all changes. The PowerShell command sequence at the end of the session.

**Estimated session shape.** Pass 3 is meaningful work but not as deep as Pass 2 was — the decisions are all made; this is consolidation + documentation + handoff. A single focused session should close it.

---

## 5. Operating constraints

### 5.1 No code changes

This is design-phase closure work. The runbook for Clair is a task file, not implementation. Test count stays at 468.

### 5.2 Header discipline

Every edited file gets its header `Last updated` bumped. Two trailing spaces before EOL on every `> ...` line. The federation propagation design doc bumps from v0.6 to v1.0 (first canonical, not v0.7 — canonicalisation deserves the major bump).

### 5.3 Header status transitions

- `tasks/FEDERATION_PROPAGATION_DESIGN.md` (Pass 2 task): ACTIVE → COMPLETED.
- `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (newly created runbook): Status ACTIVE on creation (it's the next active task once the milestone ACTIVE-flips).
- `tasks/FEDERATION_PROPAGATION_PASS_3.md` (this file): ACTIVE → COMPLETED at session close.
- `docs/xgen_federation_propagation_design.md` (canonical): PENDING → ACTIVE.

### 5.4 D-070 / D-071 coordination

D-070 promotion to DECISIONS.md and D-071 candidate ("Subsystem audits precede dependent milestones") are separate small tasks listed in `docs/ROADMAP.md` near-future. Pass 3 may fold them in if Joe wants the same-commit efficiency, or may defer them. The default is defer (keeps Pass 3's scope contained); the decision is Joe's call at session start.

### 5.5 Validation asymmetry as precondition — repeat in runbook

The runbook must make the "Phase 2 validation pipeline before Phase 4 federation push" ordering hard. Federation push without validation asymmetry closure lands a vulnerability per audit §3.6 and the milestone's foundational reason for existing. The runbook should not let Clair slip on this ordering.

---

## 6. Definition of Done

This task completes when ALL of the following are true:

- [ ] Canonical design doc `docs/xgen_federation_propagation_design.md` consolidated to v1.0, Status ACTIVE, all ten F-items inline, addenda deleted.
- [ ] All `[JOE-LOCK]` markers walked to final form.
- [ ] `docs/xgen_ch4_implementation.md` lines 779 and 825-827 corrected per F-8.
- [ ] `docs/xgen_node_admin_ops_design.md` §4.2 corrected per F-9.
- [ ] `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (runbook for Clair) created with all sections per §3.4 above.
- [ ] `tasks/FEDERATION_PROPAGATION_DESIGN.md` (Pass 2 task) Status COMPLETED.
- [ ] CLAUDE.md updated: header `Last updated`, Federation milestone block ACTIVE, roadmap line updated, current state section reflects Pass 3 closure.
- [ ] ROADMAP.md updated: design phase logged as ✅ DONE in past, implementation surfaced as 🟢 PLAY or 🟡 PENDING (per Joe's call).
- [ ] Pass 3 task file Status COMPLETED.
- [ ] All changes committed and pushed in one coordinated commit (per CLAUDE.md Rule 4 — journal entry / record-update comes last, in the same commit as the work).

`Status: COMPLETED` header is the real ship signal per project discipline. Not the commit message, not the conversation. The status header.

---

## 7. Cross-references

- **Pass 2 task file (closes here):** `tasks/FEDERATION_PROPAGATION_DESIGN.md`
- **Main design doc:** `docs/xgen_federation_propagation_design.md`
- **Addenda (folded in, then deleted):** `docs/xgen_federation_propagation_design_F7_addendum.md`, `_F8_F9_addendum.md`, `_F10_addendum.md`
- **Audit (precondition input):** `docs/xgen_propagation_reliability.md` (J-081)
- **Files needing corrections:** `docs/xgen_ch4_implementation.md` (lines 779, 825-827), `docs/xgen_node_admin_ops_design.md` (§4.2)
- **Discipline rules:** D-069 (Joe-locked design phase + canonical-document rule), D-065 (honest behaviour over polite behaviour)
- **Downstream:** `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (newly created in Pass 3), then M6 (new) implementation, then M7.

---

*End of Pass 3 task file.*  
