# M-DOC-ROADTREE — LEG D PHASE-0 BRIEF

> Version: 1.0  
> **Status:** ACTIVE  
> **Last updated**: 2026-07-28  
> Author: JozefN  
> Seat: Chat (grounding + measurement). Provenance of every recommendation here: **DELEGATED**.  

---

## 🛑 0. WHAT THIS IS, AND WHAT IT IS NOT

The grounding pass for **Leg D**, the `CLAUDE.md` half of M-DOC-ROADTREE. It records what was **measured**, what those measurements rule out, and what is **open for Joe**. It rules nothing about appearance or structure.

⚠️ **NOTHING HERE AUTHORISES A DELETION.** Section 7 states what has *not* been established. Leg C nearly erased a Joe-lock because an argument that a deletion was safe got reused as a measurement that it was.

---

## 🔒 1. GROUNDING — WHAT `CLAUDE.md` ACTUALLY IS

640,645 bytes, 821 lines, longest line **124,299 chars** (L29). Five documents in one file:

| region | lines | chars | share |
|---|---|---|---|
| A preamble + standing conventions | 1-19 | ~4,500 | 0.7% |
| **B1 PLAY head, prose + accretion** | **21-70** | **197,164** | **30.8%** |
| **B2 PLAY head, 81 blockquoted blocks** | **71-271** | **370,342** | **57.8%** |
| MANDATORY behaviour rules | 273-306 | 4,945 | 0.8% |
| C DONE archive (M1-M6, Phase 1/2) | 307-542 | 31,715 | 5.0% |
| reference (architecture, layout, build) | 543-821 | 17,032 | 2.7% |

🔑 **THE FILE IS THE PLAY HEAD.** B1 + B2 = **88.6%**. The DONE archive that looks like the obvious target is worth **5%**.

📌 **THE RULES SIT IN TWO PLACES, 252 LINES APART.** Standing conventions at L11-19; MANDATORY behaviour rules at L273. The entry instruction *conventions at the top, then the PLAY head* is true of A and false of L273.

📌 **B1 AND B2 ARE DIFFERENT SHAPES.** B1 is an H2 heading of 4,706 chars plus prose, including **L29, a single Next-active line of 124,299 chars** appended to across many sessions — a log written into one line, not one statement. B2 is a clean sequence of 81 blockquoted blocks. ⇒ **A FORMAT RULED FOR B2 DOES NOT TRANSFER TO B1.**
---

## 🔑 2. THE BLOCK CENSUS — 81 HEADLINES BY LEAD SYMBOL

| symbol | meaning | count |
|---|---|---|
| ✅ | DONE | **42** |
| 🟢 | PLAY | 15 |
| 🔒 | LOCK | 9 |
| 🔑 | KEY / finding | 7 |
| 🛑 | STOP / blocked | 3 |
| ⚠️ | WARN / defect | 3 |
| 🟡 | PENDING | 2 |
| | **total** | **81** |

Counted by reading the first code point above U+2000 on each headline and tallying by code point, **with the total asserted equal to the headline count** — so the census cannot be partially complete.

⇒ **THREE BUCKETS, NOT ONE.** **42 finished** · **17 live** (🟢 + 🟡) · **22 non-work** (🔒 🔑 🛑 ⚠️) — locks, findings, defects and blockers that have no *done* state to reach.

---

## 🔑 3. THE FINDING THAT REFRAMES LEG D — THE ARCHIVE EXISTS AND THE RULE LAPSED

📌 **THIS IS NOT A NEW DISCOVERY AND MUST NOT BE RECORDED AS ONE.** `M_DOC_ROADTREE.md` §1 already lists `CLAUDE_HISTORY.md` at 869,178 bytes as *prior PLAY blocks (D-094)*. What is new is everything after it: that the rule **lapsed**, when, and by how much.

🔒 **`CLAUDE_HISTORY.md`** — 869,178 bytes, 2,218 lines, **185 archived blocks**, `Status: ARCHIVED`, `Last updated: 2026-06-22`. Its own preamble calls it the frozen archive of superseded PLAY blocks lifted verbatim from `CLAUDE.md`.

🔒 **D-094 ALREADY RULES THIS.** *Canonical-record archiving: relocate superseded content to a frozen ARCHIVED sibling with a forward pointer; never rewrite history.* Dated 2026-06-17. It names `CLAUDE.md` explicitly and requires a **small live working head**. Archiving is a **move, never a rewrite**.

🔒 **THE BOUNDARY IS CLEAN AND THERE IS NO DUPLICATION.** `CLAUDE_HISTORY.md` spans J-81 to J-405. The PLAY head spans J-519 to J-604. **Not one of the 81 block headlines has its lead J-number present in the history file.** Nothing has been archived twice; nothing in the head is already filed there.

⇒ **LEG D IS NOT A FORMAT-INVENTION PROBLEM. IT IS A LAPSED CONVENTION.** The mechanism was designed, ruled, used 185 times, and last applied **2026-06-22**. 81 blocks have accreted in the live head since.

⚠️ **THE EARLIER FRAMING — rule a new node grammar for the PLAY head — WAS MINE AND IT WAS WRONG.** Recorded here so it is not re-derived.
---

## ⚠️ 4. CITATION INTEGRITY — 372 DESIGNATIONS IN THE PLAY HEAD, 6 BROKEN

| cited | reality | class |
|---|---|---|
| `D-030` | `D-030a` **and** `D-030b` both exist | bare number RETIRED by collision split |
| `D-056` | `D-056a` and `D-056b` exist; `D-056b` used 21 times | bare number RETIRED by collision split |
| `D-117` | reserved for the fold axis inside another decision body | never issued |
| `D-130` | **0 mentions in `DECISIONS.md`** | cited but never written |
| `N-092a` | mentioned once; `N-092` is the one with a heading | never issued as a note |
| `N-092b` | **0 mentions in `xgen-ui-notes.md`** | cited but never written |

366 of 372 resolve. 📌 **THIS CUTS AGAINST A DELETION ARGUMENT, NOT FOR ONE.** The PLAY head is far better linked than the ROADMAP prose Leg C removed.

📌 `J-123` was a false positive of mine: it exists as `### J-123 — recovered body`. **The journal now carries two heading grammars** — native `## Entry J-NNN` and J-603 recovered bodies `### J-NNN`. Any census keyed on one silently misses the other.

---

## ⚠️ 5. METHOD DEFECTS EARNED IN THIS PASS — ALL MINE

1. **A designation regex that did not know the naming rules.** `\bJ-\d{2,3}\b` cannot match `N-124a`. Three lines read as uncited that were not. ⇒ **A CENSUS THAT DOES NOT ENCODE THE SUFFIX CONVENTION REPORTS A CLEAN MISS AS A CLEAN RESULT.**
2. **A symbol census by `.Contains()` returned 75 for five different symbols.** Impossible, and it looked like data. Redone by code point with a total assertion. ⇒ **THE TRAP WAS WRITTEN DOWN IN THE BRIEF AND THE WARNING WAS NOT ENOUGH; ONLY THE SELF-CHECKING TOTAL CAUGHT IT.**
3. **A token-uniqueness triage that measured phrasing, not substance.** 924 of 7,369 tokens in the PLAY head appear nowhere else across a 495-file corpus — but the top scorers were the blocks' own headlines, unique by construction even when the cited entry holds the substance. ⇒ **UNIQUE WORDING IS NOT UNIQUE CONTENT.** The triage is discarded, not reported as a result.
---

## 🔓 6. OPEN FOR JOE — NONE OF IT GATES THE NEXT MEASUREMENT

### 🔓 6a — WHERE DO THE 22 NON-WORK BLOCKS GO?

The 42 ✅ blocks have a destination D-094 already names. The 17 live blocks stay. **The 22 🔒 🔑 🛑 ⚠️ blocks are the whole question**: a lock is not superseded when the work around it closes, and R-3 says a container of non-work carries a FORCE, not a STATE. This is the exact class Leg C nearly erased.

| option | ① user-visible | ② tier | ③ resource cost |
|---|---|---|---|
| **R1 — archive all 22 with the 42** | none | none | cheapest; **buries live locks in a file marked ARCHIVED** |
| **R2 — promote by kind: 🔒 to `DECISIONS.md` or the standing conventions, 🔑 ⚠️ to `xgen-ui-notes.md` or the roadtree, 🛑 to the roadtree as blocked nodes; archive the remainder** | none | none | 22 separate judgements; the only route that leaves the live head genuinely small |
| **R3 — keep all 22 in the live head, archive only the 42** | none | none | least risk of loss; head stays large and the problem returns |

**Recommend R2.** It is the only option honouring both D-094 (a *small* live head) and R-3 (non-work is not work-state).

⚠️ **R2 IS THE PROMOTION SUCCESSOR THE BRIEF ALREADY FLAGS AS ITS OWN MILESTONE AND WARNS MUST NOT BE INFERRED AS DONE.** If R2 is chosen it is a named milestone with its own runbook, not a Leg D sub-step.

### 🔓 6b — DOES B1 GET THE SAME TREATMENT AS B2?

| option | ① user-visible | ② tier | ③ resource cost |
|---|---|---|---|
| **S1 — Leg D covers B2 only; B1 becomes its own leg** | none | none | keeps Leg D mechanical and bounded |
| **S2 — Leg D covers B1 + B2 together** | none | none | one pass, but couples a mechanical move to a 124,299-char line that must first be **parsed** |

**Recommend S1.** 📌 B2 can be archived block by block with a mechanical rule; **B1 has no block boundaries to move**. Leg C grew to roughly three times its lock because each expansion was individually correct — B1 needs its own grounding pass before anyone rules a format for it.

### 🔓 6c — THE SIX BROKEN CITATIONS: REPAIR NOW OR INSIDE LEG D?

| option | ① user-visible | ② tier | ③ resource cost |
|---|---|---|---|
| **T1 — own commit now, before any block moves** | none | none | 6 edits; no block is ever archived carrying a dead pointer |
| **T2 — repair as each block is touched** | none | none | zero extra commits; a block archived before its turn keeps the dead pointer **permanently**, because `CLAUDE_HISTORY.md` is frozen |

**Recommend T1.** ⚠️ `D-130` and `N-092b` are cited but were never written — that is not a link repair. It is either a record Joe still owes or a citation that must be withdrawn, and **only Joe can say which**.
---

## 🛑 7. WHAT IS **NOT** ESTABLISHED

1. **That any block is redundant.** A resolving citation proves a record exists. **It does not prove the block's substance is in it.** That is exactly the step that turned *git holds every byte* into *the prose is redundant* in Leg C. **NOT MEASURED.**
2. **That the 42 ✅ blocks are safe to move.** D-094 makes archiving a **move, not a delete**, which lowers the stake — but a move into a file marked ARCHIVED is still a claim that nothing live depends on the content. **NOT MEASURED.**
3. **What L29 contains.** 124,299 chars, never parsed. **NOT MEASURED.**
4. **What region C duplicates.** 139 lines of M1-M6 and Phase 1/2 blocks the roadtree now owns. **NOT MEASURED.**

---

## 🔒 8. THE CHECKS, WRITTEN BEFORE THE ARGUMENT

Leg C's runbook V1-V7 paid for itself four times in one session. Leg D writes its own first.

- **V1** — every block moved appears in `CLAUDE_HISTORY.md` **byte-identical** to its pre-move text. Archiving is a move, never a rewrite (D-094).
- **V2** — every designation cited by a moved block resolves in its canonical file, **with the suffix convention encoded in the pattern**.
- **V3** — blocks before = blocks remaining + blocks archived. No block lost, none duplicated.
- **V4** — no J-number appears as a lead citation in **both** the live head and the history file.
- **V5** — symbol census by code point with a total assertion, before and after.
- **V6** — every size reported with its lens named: `git cat-file -s` for blobs, disk bytes for disk. Neither quoted as the other.
- **V7** — the live head after Leg D is measured against D-094's *small live working head*, and the number is stated, not asserted.

---

## 📌 9. NEXT MEASUREMENT — NOT GATED ON SECTION 6

The substance pass on the 42 ✅ blocks: open each block's cited entry primary body and check that the block's locks, numbers and defects actually appear there. That is the only thing that converts section 7 item 2 from **NOT MEASURED** into a result.
---

## 🛑 10. A FINDING FROM OUTSIDE LEG D'S SCOPE — THE STATE BOARD OMITS THE WORK

Measured while placing Leg D's roadmap node: **66 of the 75 milestone IDs named in the live PLAY head have no presence in `docs/ROADMAP.md`.** Nine appear.

Absent include **both currently-playing milestones** — `M-DOC-ROADTREE` and `M-RP-LIVEFEED-REFRESH` — plus `M-RP-MEMBERS`, the entire `M-RP6.x` and `M-RP7.x` series, and `M-SEC-TLS` / `M-SEC-AUTHSESS`.

📌 **THIS IS NOT A LEG C DEFECT, AND THE CHECK SAYS SO.** Leg C's **V4** was defined as *names in deleted prose ∩ tree node names*, and its result — **0 milestones lost** — is correct as scoped. Leg C did not lose them; they were never in `ROADMAP.md`. The gap predates the leg. *(Checked before writing, because the opposite reading was the tempting one.)*

⚠️ **IT IS VISIBLE ON THE FACE OF THE TREE.** The container `🟢 UI component-library / substrate` has five children and **all five are ✅**. Under **R-2** a container whose children are all ✅ derives ✅. It reads 🟢 only because of children that are not there. ⇒ **A DERIVED STATUS COMPUTED OVER AN INCOMPLETE CHILD SET IS NOT DERIVED, IT IS ASSERTED.**

🔓 **WHAT LEG D DOES ABOUT IT IS JOE'S. NONE OF IT GATES LEG D.**

| option | ① user-visible | ② tier | ③ resource cost |
|---|---|---|---|
| **U1 — add only the `M-DOC-ROADTREE` node; file the RP gap as its own milestone** | none | none | one node; the RP branch stays wrong, but no more wrong than today |
| **U2 — back-fill all 66 inside Leg D** | none | none | 66 nodes, each needing a status, a link chain and R-4 compliance; Leg C grew threefold from less |
| **U3 — add nothing until the gap is closed** | none | none | zero; leaves the *active* milestone off its own board |

**Recommend U1.** Back-filling 66 nodes is a milestone, not a step, and it needs R-1 through R-6 applied per node exactly as Leg C did.

⚠️ **ADDING ONE NODE TO A BRANCH THAT IS 66 BEHIND MAKES THE TREE LOOK MAINTAINED.** That is why U1 must ship with the gap **filed as a named milestone**, not merely noted in this document.