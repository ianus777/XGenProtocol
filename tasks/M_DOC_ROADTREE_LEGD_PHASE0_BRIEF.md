# M-DOC-ROADTREE — LEG D PHASE-0 BRIEF
> **Status**: ACTIVE  
> Version: 1.6  
> Date: Jul 2026  
> **Last updated**: 2026-07-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  
> Seat: Chat (grounding + measurement). Provenance of every recommendation here: **DELEGATED**.  

⚠️ **HEADER BROUGHT TO CONVENTION AT v1.5 (2026-07-29).** v1.1–v1.4 carried `> **Status:**` for `> **Status**:`, `Version` above `Status`, and no Date, Language, Credits or License lines. Recorded rather than corrected silently.

🛑 **§6a's MOVE-SET COUNT IS SUPERSEDED — SEE `tasks/RUNBOOK_ROADTREE_LEGD.md` §A (v1.1, J-613).** This document says **six** blocks self-declare closed; the session kickoff said **seven**; the runbook at v1.0 said **fifty** in the move set. **All three are wrong, and for one reason — each classified a bucket handed to it instead of all 81 blocks.** The measured move set is **62** (65 under §3a W1): 42 ✅ + 4 self-declared closed + 5 stale-closed among the 22 + **11 blocks the census called 🟢 live that are phase-0 records of milestones this same head later closes**. ⚠️ **THE 17 🟢🟡 BLOCKS WERE NEVER CLASSIFIED IN THIS DOCUMENT** — they were taken as live by lead symbol, which is what ruling E forbids. The runbook's §A, one row per block with an evidence string, is the authority from here.

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

📌 **AND THE SIX ARE A LOCAL VIEW OF A REPO-WIDE SURFACE (J-607).** Across 245 live-surface files: **22,664 citation sites, 474 unresolved — 2.09% — over only 20 distinct designations.** The work is **per-designation, not per-site**, and it collapses to four knots: the **D-030…D-056 bare-retirement cluster** (7 designations, 110 sites, one historical renumbering) · **records never written** (`J-098`, `J-109`, `J-113` — 268 sites, **already investigated at J-603**) · the **J-044/J-045 collision** (12 sites, open as §8b) · **genuinely uninvestigated** (`N-092a`, `N-092b`, `N-095b`, `J-067`, `J-171`, `J-81` — 84 sites). Ruled by **D-131**: annotate, do not repair.

---

## ⚠️ 5. METHOD DEFECTS EARNED IN THIS PASS — ALL MINE

1. **A designation regex that did not know the naming rules.** `\bJ-\d{2,3}\b` cannot match `N-124a`. Three lines read as uncited that were not. ⇒ **A CENSUS THAT DOES NOT ENCODE THE SUFFIX CONVENTION REPORTS A CLEAN MISS AS A CLEAN RESULT.**
2. **A symbol census by `.Contains()` returned 75 for five different symbols.** Impossible, and it looked like data. Redone by code point with a total assertion. ⇒ **THE TRAP WAS WRITTEN DOWN IN THE BRIEF AND THE WARNING WAS NOT ENOUGH; ONLY THE SELF-CHECKING TOTAL CAUGHT IT.**
3. **A token-uniqueness triage that measured phrasing, not substance.** 924 of 7,369 tokens in the PLAY head appear nowhere else across a 495-file corpus — but the top scorers were the blocks' own headlines, unique by construction even when the cited entry holds the substance. ⇒ **UNIQUE WORDING IS NOT UNIQUE CONTENT.** The triage is discarded, not reported as a result.
---

## 🔓 6. OPEN FOR JOE — NONE OF IT GATES THE NEXT MEASUREMENT

### 🟢 6a — E IS RULED; THE RULING IS NOT THE WORK — RE-READ FOR STATE, THEN LEAVE THE RESIDUE (Joe locked 2026-07-28; option authored by Chat)

**C** — every block is classified by **what its own text says**, not by its lead symbol. Blocks that self-declare closed join the 42 and archive under D-094. **D** — the residue stays in the live head and is dealt with when work next reaches it, which is **D-131's logic applied to blocks instead of citations**.

⚠️ **R2 IS WITHDRAWN, AND READING THE 22 IS WHAT KILLED IT.** R2 was recommended off the symbol census **without reading the blocks**. Reading them shows two things.

🔑 **THE LEAD SYMBOL MARKS THE FINDING INSIDE THE BLOCK, NOT THE BLOCK'S STATE.** L193 leads 🔒 and opens *Leg C ✅ CLOSED (J-604)*; L249 leads 🔒 and says *LEG A DONE*; L251 leads 🔒 and says *DONE (J-568)*; L195, L199 and L201 lead 🛑 🛑 🔑 and are all M-DOC-ROADTREE legs that Leg C closed. **Six of the 22 self-declare closed in their own first sentence** — L75, L177, L193, L249, L251, L255. ⚠️ **CORRECTED from an earlier claim of eight**: L195, L199 and L201 do **not** self-declare closed, they were **superseded** when Leg C closed. Superseded needs a judgement about what superseded it; self-declared-closed needs only reading, and the two must not be counted together. ⇒ **SORTING BY SYMBOL WOULD FILE FINISHED WORK AS STANDING DECISIONS.**

🔑 **AND THEIR DESIGNATIONS ALREADY HAVE CANONICAL HOMES.** `N-118`, `N-120`, `N-124`, `N-124a` and `N-124b` all have headings in `ui/docs/xgen-ui-notes.md`; `D-122` and `D-123` both have headings in `DECISIONS.md`. These blocks are **session narrative about a record that is already canonical**, not orphans awaiting promotion. ⇒ **PROMOTE WAS THE WRONG VERB.**

📌 **THE PROMOTION SUCCESSOR MILESTONE MAY NOT EXIST AS SEPARATE WORK.** R2 needed 22 judgements and its own milestone. On this reading C's measured output is **6 self-declared closed · 1 blocked · 1 live · 14 declaring no state at all**, so **14 blocks go to the check** (⚠️ corrected from an eyeball estimate of 4–6, which was a guess at the *final residue*, not a count of the work) — M-RP-MEMBERS Leg C blocked/paused, M-RP-LIVEFEED-REFRESH, the H1/H2 address-book question, the J-564 sequencing lock — and D-131 already covers them. **Worth knowing before that milestone is named.**

⚠️ **WHAT C IS NOT.** C is **reading, not measurement**. It classifies; it does not license the archive move. That still waits on §7 item 2 as corrected below.

**Status derives from the children, per R-2.**

| child | state | |
|---|---|---|
| the ruling — **E** | ✅ | Joe locked 2026-07-28 |
| **C** — classify all 81 by their own text | ✅ | **42 of 42** ✅ blocks self-declare closed ⇒ the symbol is reliable on the DONE set and **lies only on the non-work set** |
| **operative-instruction check** — the 22 non-work blocks | ✅ | **17 checked** (7 self-declared closed + 10 stateless); 5 stay live and need none |
| **operative-instruction check** — the 42 ✅ blocks | 🟡 | ↳ trigger: the move opens |
| the **residue disposition** | ✅ | the residue stays in the live head under D-131 |

⚠️ **THIS SECTION CARRIED ✅ AND SHOULD NOT HAVE.** A ✅ needing the qualifier *the ruling is closed but its execution is not* is **R-4's own example** — a claim its own symbol contradicts. Corrected to 🟢, deriving from the children above. 📌 **§6c's ✅ is correct by contrast:** D-131 is minted and forbids a sweep, so it has **no unfinished children**. Same symbol, different truth, and only the child list tells them apart.

🔒 **WHAT THE CHECK FOUND — 17 BLOCKS, FOUR CANDIDATE GAPS, ALL FOUR NOW CLOSED.**

| gap | verdict |
|---|---|
| L90 — the three-instance defect pattern | **never a gap.** `ui/docs/xgen-ui-notes.md` carries a **fourth-instance** version and is on the session-open list. My measurement had searched two phrasings across four files and concluded repo-wide |
| L175 — the processor wiring policy | **D-099 third amendment** (J-610). Both existing copies carried a wrong framing, so it was written rather than copied |
| L251 — `INTERACTIVE — HANDS OFF` | **D-132** (J-609) |
| L251 — the `Owes:` line | **D-133** (J-609) |

📌 **EVERY CANDIDATE WAS A CONVENTION, RULE OR DEFECT-CLASS THAT WAS NEVER GIVEN A NUMBER.** Not one was milestone narrative. ⇒ **THE CHECK IS NOT LOOKING FOR LOST CONTENT; IT IS LOOKING FOR UNNUMBERED RULES.**

⚠️ **FIVE STALE SELF-DECLARATIONS — AND THEY EXPOSE A LIMIT IN E ITSELF.** **L195** and **L199** declare `🟢 PLAY`; Leg C closed them. **L241**, **L245** and **L249** all say M-RP-LOCK-RECHECK is ACTIVE; `tasks/M_RP_LOCK_RECHECK.md` reads **COMPLETED v1.2**. ⇒ **C CLASSIFIES BY WHAT THE BLOCK SAYS, AND WHAT IT SAYS WAS TRUE WHEN WRITTEN.** E therefore needs a third input at move time: **the block's claim checked against current state** in `docs/ROADMAP.md` and the task docs. Cheap wherever a task doc exists, and it is the difference between archiving a closed block and archiving one that still says it is live.
### ✅ 6b — RULED: S1 — LEG D COVERS B2 ONLY (delegated 2026-07-28)

**B1 leaves Leg D's scope and becomes its own leg.** Leg D archives the B2 blocks and stops.

🔑 **WHY THEY CANNOT SHARE A LEG.** B2 is **81 discrete `> ###` blocks** — each one has a findable start and end, so the whole leg is *move block, verify byte-identical, count in equals count out*. **B1 has no boundaries to move.** Its bulk is **L29, a single line of 124,299 characters**, a `Next-active (UI/RP track):` line appended to across dozens of sessions. Nothing can happen to it until someone **reads it and decides where one statement ends and the next begins**. ⇒ **THAT IS PARSING, NOT MOVING, AND IT IS A DIFFERENT KIND OF RISK.**

📌 **AND THE PRECEDENT IS ONE LEG OLD.** Leg C was locked as *keep the tree, link its nodes, delete the prose* and came in at roughly **three times** that, because each expansion was individually correct and the aggregate was never re-locked. S2 would weld a scriptable job to an unestimated interpretive one.

⚠️ **THE HONEST COST OF S1, STATED SO IT IS NOT DISCOVERED LATER.** Leg D takes `CLAUDE.md` from **640,645 → ~410,000 bytes** (36% off) and **L29 survives untouched**. The file stays large and its worst line is unchanged. **S1 is bounded and finishes; it does not fix the thing that made the file unreadable.** That is the trade, taken deliberately.

🔓 **LEG E IS CREATED AND ITS TITLE IS JOE'S.** Chat proposes **Leg E — the B1 prose and the 124,299-char accretion line**. The letter is mechanical (next after D); **the descriptive title is naming and therefore Joe's** (D-123). It opens with **its own grounding pass** — what L29 contains has never been measured, only sized.
### ✅ 6c — RULED: BROKEN CITATIONS ARE ANNOTATED, NOT REPAIRED (D-131, delegated 2026-07-28)

**T1 is withdrawn, and the argument that supported it was wrong.** T1 rested on *a block archived with a dead pointer keeps it forever*. That holds for a **repair**. It does not hold for an **annotation**, which survives freezing and tells the future reader precisely what not to trust. ⇒ **Archiving under D-094 does not gate on link repair.**

🔒 **D-131** — a citation proven broken is **annotated in place with what is known**, never silently repointed, never deleted, and investigated only when work reaches that site. Form is inline: `· J-098 — never written, see J-603` · `D-030 — bare retired, see D-030a/D-030b`.

📌 **THE SIX ARE PART OF A REPO-WIDE SURFACE THAT IS SMALLER THAN IT LOOKS.** 22,664 citation sites across 245 live-surface files; **474 unresolved (2.09%) across only 20 distinct designations** — four knots, and one of them (`J-098`/`J-109`/`J-113`, 268 sites) was **already investigated at J-603**. Register lives in D-131.

⚠️ **NO RETROACTIVE SWEEP.** Leg D annotates only the sites it touches. `D-130` and `N-092b` stay cited-and-annotated until Joe rules whether they are records still owed or citations to withdraw.

---

## 🛑 7. WHAT IS **NOT** ESTABLISHED

1. **That any block is redundant.** A resolving citation proves a record exists. **It does not prove the block's substance is in it.** That is exactly the step that turned *git holds every byte* into *the prose is redundant* in Leg C. **NOT MEASURED.**
2. **That any block is safe to move — and this question was stated wrongly here at first.** ⚠️ **CORRECTED 2026-07-28.** The original wording asked whether anything would be *lost*. Under **D-094 archiving is a move**: the bytes stay in the repo and nothing is lost. The real risk is narrower — **a rule still in force stops being read**, because `CLAUDE.md` is read at session open and `CLAUDE_HISTORY.md` is not. ⇒ **THE TEST IS: does this block carry an operative instruction to a future session that is written nowhere in the session-open set** (`CLAUDE.md`, recent `JOURNAL`, `docs/ROADMAP.md`, `DECISIONS.md`, `ui/docs/xgen-ui-notes.md`, active `tasks/` docs)? 📌 Seven designations are **already cleared** by it — `N-118`/`N-120`/`N-124`/`N-124a`/`N-124b` sit in the UI notes and `D-122`/`D-123` in `DECISIONS.md`, both on that list. **STILL NOT MEASURED for the rest.**
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

The **operative-instruction check** over whatever set C produces: for each block, does it carry an instruction to a future session that the session-open set does not already hold? That is Leg D's **first execution step** and the only thing that converts §7 item 2 into a result.

⚠️ **A MARKER TRIAGE IS NOT THAT CHECK.** Counting `OPEN`/`OWED`/`trigger`/`BLOCKED` against `CLOSED`/`DONE` scored **L193 — the Leg C closure entry — at 7 open markers**, because a closure names what it filed and what remains. ⇒ **VOCABULARY IS NOT STATE.** Run on the block's claim, not on its word list.

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