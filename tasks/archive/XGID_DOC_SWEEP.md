# XGID Adoption v1 — Phase 2 Doc-Tree Sweep
> **Status**: COMPLETED  
> Version: 1.2  
> Date: May 2026  
> **Last updated**: 2026-05-20 (Phase 2 walk closed; classification table reviewed and Joe-confirmed across all 23 in-scope docs; Status flipped ACTIVE → COMPLETED in same commit as CLAUDE.md PLAY refresh + ROADMAP.md Past/Present/Near future updates per same-commit discipline; classification table becomes input for future Retrofit Pass 1–5 runbook authoring)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Scope statement

Phase 2 of XGID Adoption v1 walks every main document in `docs/` and produces a classification table answering, for each document:

- **Update at v1?** (gets edited in the same scope as the Phase 1 canonical sources commit, or as an immediate follow-on.)
- **Update with a specific Retrofit Pass?** (which Pass 1–5 owns its retype.)
- **No update needed?** (document carries no XGID-bearing content, or is a non-normative surface that doesn't need pointer treatment.)

The deliverable is the **classification table itself**, not the per-document edits. Edits land afterwards under their assigned Retrofit Pass (or, for v1-tagged items, as a small follow-on commit if any surface). Phase 2 is the design-of-the-edits; each Retrofit Pass's runbook is downstream.

This task file is paired upstream with `tasks/XGID_ADOPTION_DESIGN.md` (the design walkthrough record that flagged the Phase 2 sweep as a deferral) and parallel to `tasks/XGID_ADOPTION_IMPL.md` (Clair's two-commit implementation runbook, which does not depend on Phase 2 completing).

---

## Pre-walk lock: Scope-A vs Scope-B

**Locked: Scope B** (2026-05-20, Joe-lock during Sub-phase 1 framing).

### The question that locked

Phase 1 shipped one-line normative pointers in `docs/xgen_aicontrol_implementation.md` and `docs/xgen_appendix_f_en.md`. Phase 2 had to resolve whether the same pointer treatment extends to every other XGID-bearing document in the tree (Scope A) or only to a subset (Scope B).

### Audience analysis that resolved it

The classification table itself is read by future Chat Claude sessions and by Joe; Clair never reads it directly. The pointer text inside spec/implementation docs is read by **humans browsing documentation** — Joe, future contributors, future Chat Claude doing orientation reads. No code is shaped by pointer presence or absence; no implementation work depends on it. The Scope question is fundamentally a human-facing documentation discoverability question.

Humans browsing the spec tree arrive in a "reading the spec" frame and are willing to consult Ch3 §3.0 for normative authority. Humans consulting implementation references arrive tactically — looking up a CLI flag, checking a reply schema — and may not have read Ch3 first. Pointers earn their place in the latter case, not the former.

### Translation rule (applied uniformly across the walk)

- **Implementation-flavoured docs** — consulted tactically, reader may not have read Ch3. Examples: `xgen_aicontrol_implementation.md`, Appendix F (CLI reference), Ch4 (implementation chapter). **Pointer at v1 or Pass N as appropriate.**
- **Spec-flavoured docs** — reader is in spec-reading frame, Ch3 §3.0 binds them by structure. Examples: Ch0, Ch1, Ch2, Ch5, Ch6, Appendices A–E + G–I. **No pointer; XGID retrofit when content is touched in the appropriate Retrofit Pass, but no normative pointer added.**
- **Design records, audit records, ROADMAP, JOURNAL** — not normative surfaces. **No pointer.**
- **Task files** — operational, not normative. **No pointer.**

### Rationale recorded for the lock

Scope B aligns with how every other Ch3 normative section already takes its authority. Error code domains live in Ch3 §3.9 without pointer-backs from every error-mentioning section; meta_atts namespace lives in Ch3 §3.1.3 without pointer-backs from every event-type description; wire format lives in Ch3 §3.1 without pointer-backs from every wire-touching doc. XGID Adoption v1 deliberately created Ch3 §3.0 to be that same kind of authority. Adding pointers throughout the spec tree would treat XGID as a special case in a way that contradicts the canonical-document rule (D-069).

Counter-pull noted: drift defence (the J-081 finding that four of five sections had spec-vs-code drift) is recent enough that "Ch3 is sufficient" isn't a free assumption. The lock accepts that pointers in implementation-flavoured docs cover the tactical-reader case, and that the spec-reader case is sufficiently covered by spec readers being in spec-reading frame. If drift surfaces post-Pass-5 that wouldn't have surfaced with Scope A pointers everywhere, that is a future-walkthrough conversation, not a Phase 2 conversation.

---

## Walk approach

1. **Enumerate the doc tree.** `list_directory` on `docs/`. Group the result by type:
   - Spec chapters (Ch0–Ch6).
   - Appendices A–J (J shipped at Phase 1; remaining nine are classification candidates).
   - Design docs (`xgen_federation_propagation_design.md`, `xgen_node_admin_ops_design.md`, etc.).
   - Audit/historical docs (`xgen_propagation_reliability.md`, etc.).
   - Navigation/project-management (`ROADMAP.md`).
   - Implementation references (`xgen_aicontrol_implementation.md`).
   - Anything else surfacing during enumeration.
2. **Triage pass per group.** Quick read of each doc (headers + early content + grep for identifier-shaped terms: `event_id`, `space_id`, `room_id`, `node_id`, `identity`, `signature`, `trust_assertion`, `xgid`). Classify obvious cases immediately. Flag ambiguous cases for full read.
3. **Full read on ambiguous cases.** Surface any sub-questions as inline Joe-locks; do not pre-decide.
4. **Populate classification table.** One row per doc. Columns: doc path, doc type, XGID-bearing content summary, classification verdict (v1 / Pass 1–5 / no update), brief rationale.
5. **Joe review of populated table.** Confirm or adjust per-row classifications. Resolve any remaining sub-locks.
6. **Status flip to COMPLETED.** Classification table becomes input for Retrofit Pass 1–5 runbook authoring (future Chat Claude work).

---

## Classification table

Populated 2026-05-20 across six walk groups (A: spec chapters; B: English appendices; C: design docs; D: audit/historical; E: implementation reference; F: project navigation). Pre-walk housekeeping flipped the two Slovak appendix translations (`xgen_appendix_a_sk.md`, `xgen_appendix_b_sk.md`) to DEPRECATED with the future-translation-pass rationale; both excluded from this table per Joe's call.

| # | Doc path | Doc type | XGID surface | Classification | Rationale |
|---|---|---|---|---|---|
| A1 | `docs/xgen_ch0_content.md` | Spec — TOC | None (48 lines, 0 hits) | **No update** | Table of contents; no identifier content. |
| A2 | `docs/xgen_ch1_philosophy.md` | Spec — philosophy | Negligible (2 illustrative URIs) | **No update** | Philosophy doc; URIs are illustrative, not normative. |
| A3 | `docs/xgen_ch2_architecture.md` | Spec — architecture | Moderate (36 hits, `xgen://` example URIs in architecture diagrams) | **No update** | Spec-flavoured; consumes XGID concepts via example URIs but doesn't define them; reader is in spec-reading frame (Scope B). |
| A4 | `docs/xgen_ch3_specification.md` | Spec — authoritative | **§3.0 is the v1 canonical normative source** | **v1 — already shipped** | Phase 1 commit landed §3.0 here as the terse normative section. |
| A5 | `docs/xgen_ch4_implementation.md` | **Implementation** | High (97 hits) | **v1 — follow-on pointer** | Implementation-flavoured per Scope-B translation rule; pointer earns its place for tactical readers (Joe-locked option (a) during walk); per-section retypes during Passes 1–4 as content is touched. |
| A6 | `docs/xgen_ch5_protocol.md` | Spec — stub | None (9 lines, 0 hits) | **No update** | Empty stub; will be classified when populated. |
| A7 | `docs/xgen_ch6_client_design.md` | Spec — client design | Moderate (14 hits, mostly `identity_id` in §6.15 + UI Identity Registry refs) | **No update** | Spec-flavoured; this was the Q1 deferred question from `XGID_ADOPTION_DESIGN.md` — Scope B resolves as no pointer; content retypes during Pass 4 when adjacent client code is touched. |
| B1 | `docs/xgen_appendix_a_en.md` | Spec — positioning | None (0 hits) | **No update** | Philosophy/positioning doc; no identifier content. |
| B2 | `docs/xgen_appendix_b_en.md` | Spec — funding | None (0 hits) | **No update** | Sustainability doc; no identifier content. |
| B3 | `docs/xgen_appendix_c_en.md` | Spec — primitive schemas | Moderate (15 hits in schema field definitions) | **Pass 1 (`xgen-common`)** | Primitive schemas define XGID-bearing field shapes; retype with xgen-common core types; no pointer (spec-flavoured). |
| B4 | `docs/xgen_appendix_d_en.md` | Spec — storage/privacy | Low (4 hits in field-description tables) | **Pass 3 (`xgen-node`)** | Node-side storage/privacy surface; field tables retype with xgen-node; no pointer. |
| B5 | `docs/xgen_appendix_e_en.md` | Spec — UI lifecycle | None (0 hits) | **No update** | Application lifecycle states; no identifier content. |
| B6 | `docs/xgen_appendix_f_en.md` | **Implementation — CLI** | High (93 hits) | **v1 — already shipped** | Phase 1 pointer landed here; per-section retypes during Pass 3 + Pass 4 as content is touched. |
| B7 | `docs/xgen_appendix_g_en.md` | Spec — log convention | Moderate (23 hits in log-line format examples) | **Pass 5** | Log line formatters and trace event field types; Pass 5 explicitly covers this surface; no pointer. |
| B8 | `docs/xgen_appendix_h_en.md` | Historical — test records | Moderate (12 hits in frozen test output) | **No update** | Historical test transcripts; D-065 forbids retroactive rewrites; same shape as JOURNAL exclusion. |
| B9 | `docs/xgen_appendix_i_en.md` | Spec — data structures | Very high (122 hits) | **Pass 1 (`xgen-common`)** | Already named in ROADMAP Near future as primary Pass 1 doc; field tables retype column-by-column; no pointer (spec-flavoured). **Coordination flag for the Pass 1 runbook author:** Pass 1's code retype and Appx I doc retype must land in the same commit set to prevent spec drift surfacing immediately at Pass 1 close. |
| B10 | `docs/xgen_appendix_j_en.md` | Spec — canonical XGID doc | Very high (122 hits) | **v1 — already shipped** | Phase 1 canonical expository document. |
| C1 | `docs/xgen_federation_propagation_design.md` | Design — canonical | Low (16 hits, field-name refs in locked F-decisions) | **No update** | Design doc, non-normative per Scope-B translation rule. |
| C2 | `docs/xgen_node_admin_ops_design.md` | Design — canonical | Low (17 hits, field-name refs in M6 verb sketches) | **No update** | Design doc, non-normative; M6 implementation uses XGID types from start per Q3 forward-discipline. |
| C3 | `docs/xgen_lifecycle_states.md` | Spec — state machine | None (0 hits) | **No update** | Lifecycle state machine; no identifier content. |
| D1 | `docs/xgen_propagation_reliability.md` | Audit — canonical (COMPLETED) | Moderate (34 hits in code-trace references frozen at audit-time) | **No update** | Frozen audit record; D-065 forbids retroactive rewrites. Doubly-defensible: non-normative surface AND historical artefact. |
| E1 | `docs/xgen_aicontrol_implementation.md` | **Implementation reference** | High (29 hits) | **v1 — already shipped** | Phase 1 pointer landed here; per-section retypes during Pass 4 when AI control surface lands. |
| F1 | `docs/ROADMAP.md` | Project navigation | Low (17 hits in navigation prose) | **No update** | Project-navigation surface, not normative; milestone-close housekeeping happens naturally on each Pass close. |

### Excluded from classification (recorded for completeness)

| Doc | Reason for exclusion |
|---|---|
| `docs/xgen_appendix_a_sk.md` | DEPRECATED (flipped 2026-05-20 pre-walk); future Slovak translation pass will retype from completed English docs |
| `docs/xgen_appendix_b_sk.md` | DEPRECATED (flipped 2026-05-20 pre-walk); future Slovak translation pass will retype from completed English docs |
| `docs/backup/` | Archival subdirectory; out of scope |
| `docs/tests/` | Legacy test-instruction files; out of scope per CLAUDE.md folder convention note |
| `JOURNAL.md` (project root) | Per-entry rewrites violate D-065 honest-provenance; future entries naturally use XGID types once Pass 1 ships |
| `CLAUDE.md` (project root) | Operational/session-state surface, not normative |
| `DECISIONS.md` (project root) | Recorded decisions are historical; new entries (e.g. D-072, D-073 from XGID Adoption v1) use XGID types as appropriate |
| `tasks/*.md` | Operational task files, not normative |

---

## Verdict distribution

23 docs classified across the six groups.

| Verdict | Count | Docs |
|---|---|---|
| **v1 — already shipped** | 4 | Ch3, Appx F, Appx J, `xgen_aicontrol_implementation.md` |
| **v1 — follow-on pointer** | 1 | Ch4 |
| **Pass 1 (`xgen-common`)** | 2 | Appx C, Appx I |
| **Pass 2 (`xgen-core`)** | 0 | (none — see observation below) |
| **Pass 3 (`xgen-node`)** | 1 | Appx D |
| **Pass 4 (`xgen-client` + AI control)** | 0 new | (Appx F and `xgen_aicontrol_implementation.md` are already pointer-tagged at v1; their per-section annotation lands in Pass 4) |
| **Pass 5 (tests, helpers, remaining)** | 1 | Appx G |
| **No update** | 14 | Ch0, Ch1, Ch2, Ch5, Ch6, Appx A, Appx B, Appx E, Appx H, `xgen_federation_propagation_design.md`, `xgen_node_admin_ops_design.md`, `xgen_lifecycle_states.md`, `xgen_propagation_reliability.md`, ROADMAP.md |

Total: 4 + 1 + 2 + 0 + 1 + 0 + 1 + 14 = 23. ✓

---

## Observations to carry forward into Retrofit Pass runbook authoring

These are flags for future Chat Claude sessions authoring the per-Pass runbooks (`tasks/XGID_RETROFIT_PASS_1_IMPL.md` etc.). Not Phase 2 action items; recorded here so they aren't rediscovered.

**Pass 1 coordination requirement.** Pass 1's code retype (in `xgen-common`) and its two doc retypes (Appx C primitive schemas, Appx I data structures) must land in a coordinated commit set. Letting code and docs diverge during Pass 1 would surface spec drift immediately at Pass 1 close. The Pass 1 runbook should make this explicit, the same shape Phase 7.5's implementation runbook coordinated code + canonical-design-doc §6.4.1 + §15 entries in the same commit sequence.

**Pass 2 has zero doc work.** `xgen-core` consumes types defined in `xgen-common`; the surface is documented across Appx C and Appx I, both of which retype with Pass 1. Pass 2's runbook should explicitly state "doc work is empty, focus is code-only" so the runbook author doesn't search for missing doc rows.

**Pass 4 has zero new doc rows but substantial per-section work in two already-pointer-tagged docs.** Appx F (CLI reference, 890 lines, 93 hits) and `xgen_aicontrol_implementation.md` (372 lines, 29 hits) both received v1 pointers but their full per-section annotation is Pass 4's deliverable. The Pass 4 runbook should anticipate this is the heaviest doc-work pass of the five despite having no new rows in the classification table.

**Ch4 v1 follow-on commit.** Ch4 needs a one-line pointer landing as a small follow-on commit (option (a) from the A5 Joe-lock). Same shape as the two pointers shipped at Phase 1. Can ride along with the XGID Adoption v1 milestone close commit (when Clair's two-commit implementation lands and the milestone flips to DONE), or as a separate small commit if timing doesn't align. Either way is fine; it's a 1-line edit.

**No design docs receive pointers under the Scope-B rule.** The Scope-B lock cleanly excludes design records, audit records, ROADMAP, JOURNAL, and task files from pointer treatment. The pointer-bearing surface in the project is exactly three docs: Ch4, Appx F, `xgen_aicontrol_implementation.md`. Phase 1 caught two of three; Ch4 is the third.

---

## Definition of Done

- [x] Doc tree enumerated; inventory recorded in this file under the Classification table section.
- [x] Each doc in the inventory has a row in the classification table.
- [x] Every row carries a classification verdict and brief rationale.
- [x] Any inline sub-locks that surfaced during the walk are Joe-resolved and the resolution is recorded inline (A5 Ch4 pointer-timing locked option (a) during walk).
- [x] Joe has reviewed the populated table and confirmed it.
- [x] Status header flipped from ACTIVE to COMPLETED.
- [x] CLAUDE.md PLAY block refreshed to reflect Phase 2 closure; ROADMAP.md Present/Near future entries updated per the same-commit discipline.

---

## Cross-references

- `tasks/XGID_ADOPTION_DESIGN.md` — design walkthrough record; "Findings deferred to Phase 2 sweep" section flagged this work.
- `tasks/XGID_ADOPTION_IMPL.md` — Clair's two-commit implementation runbook; parallel workstream, does not block this sweep.
- `DECISIONS.md` D-072 — XGID Adoption v1 architectural commitment.
- `DECISIONS.md` D-073 — field-name-vs-type discipline.
- `docs/xgen_appendix_j_en.md` — canonical expository document.
- `docs/xgen_ch3_specification.md` §3.0 — normative section.
- `docs/ROADMAP.md` — Near future entries for XGID Retrofit Passes 1–5 inherit per-doc scope from this table.
- D-069 — canonical-document rule (one authority per surface; pointers only where they earn their place).
- D-065 — honest behaviour over polite behaviour (Scope B accepts pointer scarcity over noise).

---

*End of Phase 2 doc-tree sweep task file.*  
