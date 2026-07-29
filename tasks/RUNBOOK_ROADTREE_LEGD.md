# M-DOC-ROADTREE — LEG D RUNBOOK — the `CLAUDE.md` B2 archive move
> **Status**: COMPLETED  
> Version: 1.3  
> Date: Jul 2026  
> **Last updated**: 2026-07-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 🛑 0. WHAT THIS IS

The execution document for **Leg D**, scoped by §6b to **B2 only** (`CLAUDE.md` L71–261). It carries the block classification, the move procedure and the checks. It does **not** re-open §6a, §6b or §6c — those are ruled.

🔑 **IT EXISTS BECAUSE THE MOVE SET WAS A NUMBER, NOT A CLASSIFICATION.** The session kickoff carried *7 self-declared-closed blocks*; the Phase-0 brief §6a carried *6*; **v1.0 of this runbook carried a move set of 50 and was also wrong.** §A below is the first classification that covers **all 81 blocks** with an evidence string on every row.

---

## 🔒 1. GROUNDING RE-MEASURED AT OPEN — 2026-07-29

| item | value | lens |
|---|---|---|
| HEAD at §A's read | `b0298bc` = `origin/main`, tree clean | `git rev-parse` / `git status --porcelain` |
| `CLAUDE.md` | 640,645 / 640,645 · 821 lines | disk / `git cat-file -s` |
| `CLAUDE_HISTORY.md` | 869,178 / 869,178 · 2,218 lines | disk / blob |
| `DECISIONS.md` | 564,366 / 564,366 · 4,917 lines | disk / blob |
| `docs/ROADMAP.md` | 44,758 / 44,313 · 445 lines | disk / blob |
| `CLAUDE.md` longest line | 124,299 chars (L29) | `.Length` max |
| B2 block count | **81**, L71 → L261 | `> ###` headline scan over L71–271 |
| symbol census, by code point, total asserted | ✅ 42 · 🟢 15 · 🔒 9 · 🔑 7 · 🛑 3 · ⚠️ 3 · 🟡 2 = **81** ✔ | `ConvertToUtf32` |

⚠️ **A BLOCK IS A RANGE.** Headline `> ###` → line before the next headline, trailing blanks trimmed. Nineteen of the 81 carry continuation lines.

⚠️ **THE EOL CONVENTION NOTE WAS WRONG AND DISK SAYS SO.** `CLAUDE.md` is **LF** on disk — disk equals blob at 640,645, and a CRLF file's disk size exceeds its blob by exactly its line count. Same for `CLAUDE_HISTORY.md` and `DECISIONS.md`. The CRLF files are **`docs/ROADMAP.md`** (44,758 − 44,313 = 445 = its line count) and, until edited this session, `JOURNAL.md`. ⇒ **Measured, not inherited.**

---

## 🛑 2. §A — ALL 81 BLOCKS, CLASSIFIED BY WHAT THEY SAY AND CHECKED AGAINST WHAT IS TRUE

🔑 **THE DECISIVE EVIDENCE IS INSIDE `CLAUDE.md`, NOT IN THE TASK DOCS.** Every 🟢 block that declares a milestone playing is followed, **in the same head**, by a ✅ block declaring that same milestone **CLOSED**. The head contradicts itself in eleven places. A task-doc header is an external check; a closure block twenty lines below is an internal one, and it is stronger.

### 2a — the 42 ✅ blocks

**42 of 42 self-declare closure in their own first sentence** (`— CLOSED (J-nnn)` or `— DONE (J-nnn)`), machine-verified by code point and by first-sentence read. The symbol is reliable on this set. **All 42 archive.**

⚠️ **TWO CARRY DEFECTS THAT TRAVEL WITH THEM:**
- **L247** (`M-RP-FONTS — DONE (J-570)`) also carries *"⚠️ M-RP-LOCK-RECHECK REMAINS ACTIVE"* — stale, annotate before moving. It was outside every earlier suspect list because it is a ✅ block.
- **L143** (`M-RP6.6 — CLOSED (J-543)`) names a milestone whose `tasks/M_RP6_6_RESIDENT.md` still reads **ACTIVE** — the inverse mismatch. Annotate, do not resolve; resolving it is M-RP6.6's business.

### 2b — the 22 non-work blocks (🔑 🔒 🛑 ⚠️)

| L | subject | evidence | verdict |
|---|---|---|---|
| 75 | N-120 resize/index defect | *"→ M-RP7.3 OPENS WITH IT"* · re-points the SE-corner countdown at **M-RP-SKIN** | **HOLD** (§E) |
| 87 | backdrop `.region-split` → `.region-shell` | reports a change; declares no state | stateless — stays |
| 90 | the three-for-three defect pattern | defect class; fourth-instance version in `ui/docs/xgen-ui-notes.md` | stateless — stays |
| 103 | N-124 — *"tile↔hole is 0"* not observable | finding; N-124 headed in the UI notes | stateless — stays |
| 108 | N-124a — two baselines over-precise | finding; N-124a headed in the UI notes | stateless — stays |
| 175 | *"THE TEXT PROCESSOR IS WIRED TO NOTHING"* | **L189 in this head reads `M-RP-PROCESSOR-WIRE — CLOSED (J-563)`**; rule homed as D-099 third amendment | **STALE-CLOSED** — annotate + archive |
| 177 | retraction and answer (J-561) | *"M-RP-SETTINGS IS CLOSED"* | **self-declared closed** — archive |
| 179 | the J-564 five-item sequencing lock | ① `M-RP-MSG-NEWLINE` marked 🟢; **L183 reads it CLOSED (J-565)** | **LIVE, partially stale** — stays, annotate ① |
| 193 | Leg C closure | *"LEG C ✅ CLOSED"*; its *"LEG D … 🔓 UNRULED"* is spent | **self-declared closed** — archive + annotate |
| 195 | M-DOC-ROADTREE — *"🟢 PLAY · LEG C IS OPEN AND RUNNABLE"* | Leg C closed at J-604 | **SUPERSEDED** — 🔓 §3a |
| 197 | M-RP-LIVEFEED-REFRESH (J-601) | *"🟢 PLAY · v1.10"*; doc **ACTIVE v1.10** ✔ | **LIVE** — stays |
| 199 | M-DOC-ROADTREE §6/§7 ruled, *"🟢 PLAY"* | those legs closed at J-604 | **SUPERSEDED** — 🔓 §3a |
| 201 | M-DOC-ROADTREE Leg B (J-599) | reports Leg B's result; Leg C closed after it | **SUPERSEDED** — 🔓 §3a |
| 203 | M-RP-MEMBERS Leg C | *"BLOCKED AND ⏸️ PAUSED"*; doc **ACTIVE v1.15** ✔ | **LIVE** — stays |
| 209 | H1 / H2 — the address book at rest | open question, still 🔓 for Joe | **LIVE** — stays |
| 235 | M-RP-SELF-SURFACE design walk | **no task doc, no ROADMAP node** for M-RP-SELF-SURFACE | **HOLD** (§E) |
| 241 | D-123 — the seat division | *"⚠️ M-RP-LOCK-RECHECK REMAINS ACTIVE"*; doc **COMPLETED v1.2** · D-123 headed in `DECISIONS.md` | **STALE-CLOSED** — annotate + archive |
| 243 | D-122 FINAL — window vocabulary | *"M-RP-LOCK-RECHECK REMAINS ACTIVE — #11 has its verdict"* — **missed in v1.0**, which read only its opening | **STALE-CLOSED** — annotate + archive |
| 245 | D-122 + M-RP-VIEW-BINDING | *"⚠️ M-RP-LOCK-RECHECK REMAINS ACTIVE"* | **STALE-CLOSED** — annotate + archive |
| 249 | M-RP-LOCK-RECHECK Leg A | *"LEG A DONE, MILESTONE STILL ACTIVE"*; doc **COMPLETED v1.2** · **L239 in this head closes it** | **STALE-CLOSED** — annotate + archive |
| 251 | M-RP-SWEEP | *"DONE (J-568)"* | **self-declared closed** — archive |
| 255 | M-RP-PROCESSOR-SEED | *"⚠️ SUPERSEDED AT J-567 — CLOSED"* | **self-declared closed** — archive |

### 2c — the 17 blocks the census called "live" (🟢 🟡) — ELEVEN ARE NOT

🛑 **THESE WERE NEVER CLASSIFIED. THE CENSUS CALLED THEM LIVE BY THEIR LEAD SYMBOL — WHICH IS PRECISELY WHAT RULING E FORBIDS.** Read, each one is a **phase-0 or design-lock block for a milestone this same head later closes**.

| L | declares | the closure block that disproves it | verdict |
|---|---|---|---|
| 123 | `NEXT-ACTIVE — M-RP7.5` | **L125** — M-RP7.5 CLOSED (J-528) | **STALE-CLOSED** |
| 145 | M-RP6.3 arc phase-0 locked | **L173** — Leg D2+D3 CLOSED (J-560) | **STALE-CLOSED** |
| 151 | M-RP6.3 Leg C phase-0 locked | **L153 · L157** — Leg C1, Leg C2 CLOSED | **STALE-CLOSED** |
| 155 | M-RP6.3 Leg C2 phase-0 locked | **L157** — Leg C2 CLOSED (J-550) | **STALE-CLOSED** |
| 161 | M-RP6.3 Leg D phase-0 locked | **L163 · L173** — D1, D2+D3 CLOSED | **STALE-CLOSED** |
| 165 | M-RP6.9 phase-0 locked | **L169** — M-RP6.9 CLOSED (J-556) | **STALE-CLOSED** |
| 167 | M-RP6.9 appearance returns to Chat | **L169** — M-RP6.9 CLOSED (J-556) | **STALE-CLOSED** |
| 191 | `M-RP-PROCESSOR-SEED — 🟢 NEXT` | **L253** — CLOSED · **L255** — *SUPERSEDED, CLOSED* | **STALE-CLOSED** |
| 219 | M-RP-MEMBERS **Leg B** designed and locked | **L213** — Leg B CLOSED (J-597) | **STALE-CLOSED** |
| 259 | `M-RP-PROCESSOR-WIRE … 🟢 PLAY` | **L189** — CLOSED (J-563) | **STALE-CLOSED** |
| 261 | `M-RP6.3 LEG D2 … 🟢 PLAY` | **L173** — D2+D3 CLOSED (J-560) | **STALE-CLOSED** |
| 185 | 🟡 M-RP-PROCESSOR-RENDER, fifth in the J-564 sequence | no closure anywhere | **PENDING** — stays |
| 205 | M-RP-LIVEFEED-REFRESH phase-0 | doc **ACTIVE v1.10** | **LIVE** — stays |
| 207 | 🟡 the design walk behind LIVEFEED | its arc is live | **LIVE** — stays |
| 211 | M-DOC-ROADTREE phase-0 | doc **ACTIVE v1.12** | **LIVE** — stays |
| 229 | M-RP-MEMBERS arc phase-0 | arc **ACTIVE**, Leg C paused (L203) | **LIVE** — stays |
| 233 | M-RP-OWN-ROW-NAME phase-0 | doc **ACTIVE v1.0** | **LIVE** — stays |

📌 **WHY L229 STAYS AND L219 DOES NOT.** L219 locks **Leg B**, and Leg B is closed at L213. L229 is the **arc's** phase-0 and the arc is still open at Leg C. **A phase-0 block dies with its phase, not with its milestone.**

### 2d — the tally

| verdict | count | blocks |
|---|---|---|
| ✅ self-declared closed | 42 | the DONE set |
| self-declared closed (non-work) | 4 | L177 L193 L251 L255 |
| stale-closed (non-work) | 5 | L175 L241 L243 L245 L249 |
| stale-closed (called "live" by symbol) | 11 | L123 L145 L151 L155 L161 L165 L167 L191 L219 L259 L261 |
| **→ MOVE SET** | **62** | |
| superseded — 🔓 §3a | 3 | L195 L199 L201 |
| live / pending | 10 | L179 L185 L197 L203 L205 L207 L209 L211 L229 L233 |
| stateless and homed | 4 | L87 L90 L103 L108 |
| held | 2 | L75 L235 |
| **→ STAYS** | **19** | |
| **TOTAL** | **81** ✔ | |

⚠️ **THE TALLY IS ASSERTED, NOT EYEBALLED.** 42 + 4 + 5 + 11 = **62** move · 3 + 10 + 4 + 2 = **19** stay · 62 + 19 = **81**, which equals the measured block count. 📌 The live/pending row was first written as 9 against a list of 10 and corrected here — **an arithmetic slip in a tally is exactly the class this document exists to catch**, so the correction is recorded rather than typed over.

**Under §3a W1 the move set becomes 65 and STAYS becomes 16.**

---

## 🔒 3. §B — THE THIRD INPUT: WHAT THE BLOCK SAYS vs WHAT IS TRUE NOW

🔒 **THE RULE.** Every block whose claim names a milestone is checked against **(1) the closure blocks in `CLAUDE.md` itself**, then **(2) `docs/ROADMAP.md` and the `tasks/` doc header**. Source (1) is checked first because it is the same document and cannot be out of sync with itself. Where neither source resolves the claim, the block is **held**, not guessed at.

🔒 **THE ANNOTATION FORM** — D-131 applied to state instead of citations, inline at the site, travelling with the block into history:

`[⚠️ CLAIM STALE 2026-07-29 — closed at L189 of this head, M-RP-PROCESSOR-WIRE CLOSED (J-563). Claim was true when written.]`

### 🔓 3a — WHERE DO THE SUPERSEDED THREE GO? (L195, L199, L201)

They declare `🟢 PLAY` on M-DOC-ROADTREE legs that Leg C closed. §6a's **D** sends the residue to the live head.

| option | ① user-visible | ② tier | ③ resource cost |
|---|---|---|---|
| **W1 — archive all three, annotated** | none | none | 3 annotations; head loses 3 false 🟢 PLAY declarations |
| **W2 — leave in head, annotated** | none | none | 3 annotations; head keeps 3 blocks saying a closed leg is playing |
| **W3 — leave untouched** | none | none | zero; head keeps asserting Leg C is runnable |

**Recommend W1.** §2c archives **eleven** blocks for declaring PLAY on closed milestones. These three do the same thing about *this arc's own legs*. ⚠️ **W1 widens §6a's D and is therefore Joe's.** It gates nothing: the other 62 move without it.

---

## 🔒 4. §C — THE MOVE PROCEDURE

🛑 **§C DOES NOT RUN UNTIL §F IS LOCKED.** Opening `CLAUDE_HISTORY.md` disproved three of the steps below as they were first written. They are corrected here; the two that need a ruling are in §F.

1. **Freeze the input.** `git status --porcelain` empty. A clean tree is the undo and stops being one at commit.
2. **Extract by range**, never by headline. ⚠️ **AND THE LAST BLOCK'S RANGE IS NOT OPEN-ENDED.** B2 ends at **L262**, not L271: L263–L269 are four standing `> **` items — the trusted-mouse harness, M-RP7.2's eight design locks, **M-RP-FOCUS**, and Track A — which are **not blocks and must not move**. An end-of-region default of 271 would have carried all four into the archive without touching a single headline.
3. **Annotate before moving** (§B). Annotations travel with the block.
4. **Insert at the TOP of `CLAUDE_HISTORY.md`**, immediately after the `---` on L13. ⚠️ **NOT APPENDED.** The file's own preamble states *order is newest-archived-first*; appending at the tail would file this batch behind blocks from project start. Relative head order is preserved **within** the batch.
5. **Delete from `CLAUDE.md`** highest line first, so earlier ranges keep their indices.
6. **Run V1–V9 before writing and again after.**
7. **Never a rewrite.** A moved block is byte-identical to its annotated pre-move text — see §F1, where that rule and the archive's heading convention collide.

📌 **MOVE SET: 62 BLOCKS** (65 under W1) — **not 49, not 50.** Third revision, and each revision came from re-deriving the set rather than re-reading the count.

---

## 🔒 5. §D — THE CHECKS

- **V1** — every moved block appears in `CLAUDE_HISTORY.md` byte-identical to its annotated pre-move text.
- **V2** — every designation cited by a moved block resolves in its canonical file, suffix convention encoded in the pattern (`D-030a`, `N-124b`).
- **V3** — blocks before = blocks remaining + blocks archived. **81 in, 81 out.**
- **V4** — no J-number is a lead citation in both the live head and the history file.
- **V5** — symbol census by code point with a total assertion, before and after.
- **V6** — every size reported with its lens named; disk bytes and `git cat-file -s` never quoted as each other. ⚠️ **THE EDITING TOOL IS PART OF THE LENS** — `Filesystem:edit_file` rewrites the whole file to LF, so a post-edit disk count is not comparable to a pre-edit one. **Compare blobs across an edit.**
- **V7** — the resulting live head measured against D-094's *small live working head*; number stated, not asserted.
- **V8 — WHOLE-DOCUMENT INVARIANTS ASSERTED BEFORE EVERY WRITE**, not the size of the change: line count, block count, symbol census, first line, **last line intact**, L29 still 124,299.
- **V9 — NO CLOSURE BLOCK IS SEPARATED FROM ITS PHASE-0.** Eleven of the move set are phase-0 blocks whose ✅ closure block also moves. Assert that for each such pair **both** ends land in history, so the arc is not split across two files.

---

## 🛑 6. §F — THE ARCHIVE FILE DOES NOT ACCEPT THIS BATCH AS §C DESCRIBED

Measured in `CLAUDE_HISTORY.md`: **185 `## ` headings · zero `> ###` blocks · 2,218 lines · Status ARCHIVED, Version 1.1, Last updated 2026-06-22.** Its preamble states, in its own words, that blocks were *lifted verbatim* per **D-094** — *a move, never a rewrite* — that **order is newest-archived-first**, and that the file is **ARCHIVED — do not edit**.

🔑 **THE HEAD'S BLOCK FORMAT CHANGED AFTER THE LAST ARCHIVING RUN AND NOBODY NOTICED, BECAUSE NOTHING HAS BEEN ARCHIVED SINCE.** The archive holds `## ` blocks; the live head writes `> ### `. D-094 lapsed on 2026-06-22 and the format drifted underneath it. ⇒ **A LAPSED RULE DOES NOT SIT STILL; THE THING IT GOVERNS MOVES AWAY FROM IT.**

### 🔓 F1 — verbatim lift vs. the archive's heading convention

| option | ① user-visible | ② tier | ③ resource cost |
|---|---|---|---|
| **F1a** — lift the 62 verbatim as `> ###` | none | none | zero transform; archive carries two block formats; V1 byte-identity holds exactly |
| **F1b** — rewrite each `> ### **X**` to `## X` | none | none | 62 transforms; archive stays uniform; **breaks D-094's own words — *never a rewrite*** |
| **F1c** — lift verbatim **under one `## ` batch heading** | none | none | 1 heading written, 62 blocks untouched; archive gains a conforming `## ` entry **and** every block stays byte-identical |

**Recommend F1c.** It is the only option that satisfies both rules at once: the archive's structure sees a conforming heading, and D-094's verbatim requirement is met block for block. Proposed heading: `## 🟢 Leg D archive batch — 62 PLAY blocks lifted from the live head (J-614, D-094 re-applied after a 5-week lapse)`.

### 🔓 F2 — the file says ARCHIVED, and ARCHIVED means do not modify

Project status vocabulary: **ARCHIVED — frozen historical record, do not modify.** D-094 nevertheless requires blocks to be moved *into* it. **The convention contradicts itself and has done so since the file was created**; it went unnoticed because the rule lapsed before anyone hit it twice.

| option | ① user-visible | ② tier | ③ resource cost |
|---|---|---|---|
| **F2a** — Status ARCHIVED → ACTIVE | none | none | 1 header line; but the file is not *active work*, and the word then means nothing here |
| **F2b** — keep ARCHIVED, write the D-094 append into the preamble as the single sanctioned exception | none | none | 1 header bump + 1 preamble sentence; **names the exception that already exists in practice** |
| **F2c** — new file `CLAUDE_HISTORY_2.md` for this batch | none | none | new file, new pointer, a second place to look; the 185 stay frozen |

**Recommend F2b.** The exception is real whether or not it is written down — D-094 has always required exactly this. F2c splits the archive to protect a word.

📌 **BOTH ARE CHEAP AND NEITHER IS CHAT'S.** They set records convention, not technical execution. ⚠️ **THE MOVE IS BLOCKED ON F1 AND F2** — unlike §3a, which blocks nothing.

---

## ⚠️ 7. §E — THE HOLD SET

**L75 — the M-RP-SKIN countdown.** Re-points the M-RP-7.1 SE-corner-triangle obligation at **M-RP-SKIN**, which has **no task doc and no ROADMAP node**. Archiving it files an undischarged obligation against nothing. **Home the countdown first.**

**L235 — M-RP-SELF-SURFACE.** Claims a design lock for a milestone with no task doc and no node; §B cannot check it. **Held pending a reading pass.**

📌 **ONE SOFT ITEM, JOE'S.** `--region-border: 1px` and `--region-edge: var(--s4)` are live in `ui/assets/skin.css`, but the constraint — *the edge must read against both the backdrop `--s5` and the tile body `--s`* — exists only in a block in the move set. `skin.css` is Joe's.

---

## ⚠️ 8. DEFECTS FOUND IN THIS PASS

- **🔑 THE PARTITION WAS INHERITED, AND THAT IS THE WHOLE DEFECT.** v1.0 classified 22 blocks because the census said *22 non-work · 17 live · 42 ✅*. **The 17 were "live" by lead symbol only** — the exact reading ruling E was made to forbid — and **eleven of them are closed work.** ⇒ **A CLASSIFICATION THAT ACCEPTS SOMEONE ELSE'S PARTITION HAS NOT CLASSIFIED; IT HAS RE-SORTED ONE BUCKET.**
- **THE MOVE SET WAS CARRIED AS A COUNT ACROSS FOUR DOCUMENTS** — kickoff 7, brief 6, runbook v1.0 50, actual 62. Same recurring class: *a claim narrower than the thing it describes, reused as if complete.*
- **A SUSPECT LIST IS NOT A CHECK.** v1.0 verified the `M-RP-LOCK-RECHECK REMAINS ACTIVE` clause on the three blocks the documents named. Grepping the file finds it on **five** — L241, **L243**, L245, **L247**, L249 — and L247 is a ✅ block, outside every earlier list. ⇒ **RUN THE PREDICATE OVER THE WHOLE CORPUS, NOT OVER THE NAMES YOU WERE HANDED.**
- **🔑 THE LAST BLOCK'S RANGE WAS OPEN-ENDED AND WOULD HAVE SWALLOWED FOUR STANDING ITEMS.** The range builder defaulted the final block's end to L271, the region terminator. **B2 actually ends at L262**; L263–L269 hold the trusted-mouse harness, M-RP7.2's eight design locks, **M-RP-FOCUS** and Track A — `> **` lines, not `> ###` blocks. ⇒ **A REGION TERMINATOR IS NOT A BLOCK TERMINATOR**, and the error is invisible to every headline-based check because no headline is involved.
- **THE ARCHIVE FILE WAS NEVER OPENED BEFORE §C WAS WRITTEN.** Three of seven steps were wrong: append vs. insert-at-top, the heading form, and the permission to write at all. ⇒ **A PROCEDURE WRITTEN AGAINST ONE END OF A MOVE IS HALF A PROCEDURE.**
- **`tasks/M_DOC_ROADTREE_LEGD_PHASE0_BRIEF.md` HAD A NON-CONFORMING HEADER** — corrected at v1.5 with the correction recorded in the file. ⚠️ `JOURNAL.md`'s header carries the same defect and is **left as found**; rewriting a canonical header is not this leg's business.
- **🔑 `Filesystem:edit_file` NORMALISES THE ENTIRE FILE TO LF — MEASURED.** `JOURNAL.md` was CRLF on disk (1,534,320 disk vs 1,526,965 blob; the 7,355 difference is exactly its line count). After an edit that **added** ~5 KB, disk read **1,531,984 with 0 CRLF**. ⇒ **A DISK BYTE COUNT THAT FALLS WHILE CONTENT IS ADDED IS REPORTING ON THE TOOL.** Harmless to git — `git diff --numstat` showed 27 insertions, 1 deletion, no EOL churn.
- **THE CRLF CONVENTION NOTE WAS WRONG.** `CLAUDE.md` is LF, not CRLF; `docs/ROADMAP.md` is the CRLF file. Corrected in §1 by measurement.
- **STATUS-HEADER SCAN NOISE.** **Five** task docs return multiple `Status` matches in their first four lines (`M_RP2_22`, `M_RP6_1E_A`, `M_RP6_1E_B`, `RUNBOOK_PROTO_STATUS_2`, `RUNBOOK_STATUS`). Not blocking; filed so it is not rediscovered as a finding.

---

## ✅ 10. EXECUTION RECORD — THE MOVE RAN 2026-07-29 (J-615)

**Rulings applied:** §F1c · §F2b · §3a W1 — all delegated by Joe (*go as you recommend*), options authored by Chat.

| measure | before | after |
|---|---|---|
| `CLAUDE.md` | 640,645 B · 821 lines · 81 blocks | **316,680 B · 687 lines · 16 blocks** |
| `CLAUDE_HISTORY.md` | 869,178 B · 2,218 lines · 185 `## ` | **1,197,988 B · 2,366 lines · 186 `## `** |
| reduction | | **50.6%** — the §6b forecast was ~410,000 B, beaten by 93,000 |
| L29 | 124,299 chars | **124,299 chars, untouched** — Leg E's target |

**Executed by `legd-move.ps1` + `legd-annotations.json`, both committed.** The script is ASCII-only by construction: a BOM-less UTF-8 `.ps1` is read as ANSI by PS 5.1, so every non-ASCII string it writes is `\uXXXX`-escaped in the JSON and decoded at run time. It refuses to write unless all assertions pass, and it was run dry first.

**Assertions that passed, in order:** L29 = 124,299 · history headings = 185 · block count = 81 · **L263 is a `> **` standing item, so B2 ends at L262** · symbol census totals to 81 · 42 blocks carry ✅ · move set = 65 with no duplicate · stay set = 16 · 65 + 16 = 81 · 23 annotations, each on a verified headline, each on the correct side of the move · **V9** all 11 phase-0/closure pairs travel together · line arithmetic closes · first and last non-empty line intact · L29 unchanged · **V1** every moved block byte-identical in history and absent from the head · exactly one new `## ` heading · the four standing items past L262 still in the head.

⚠️ **TWO OF MY OWN CHECKS FAILED FIRST, AND BOTH WERE THE CHECK'S FAULT.** `.Contains('trusted-mouse')` is case-sensitive and the head says `TRUSTED-MOUSE`; and the raw last element of a split on a file ending in a newline is the empty string, so *last line intact* was comparing `''` to `''` and proving nothing. Replaced with a last-**non-empty**-line assertion. ⇒ **A CHECK THAT PASSES ON EMPTINESS IS NOT A CHECK** — the same shape as the checker that once reported 22,664 of 22,664 unresolved.

**Still in the head, by design (16 blocks):** the 3 live milestones + 2 pending + 4 stateless findings + 1 partially-stale sequencing lock (annotated, ① only) + 4 standing items + **the 2 held under §E**.

🔓 **LEG D OWES:** `§E L75 — the M-RP-SKIN corner-triangle countdown has no home` · `§E L235 — M-RP-SELF-SURFACE has no task doc and no ROADMAP node`. Both blocks stay in the live head until their targets exist. Under D-133 this milestone is **COMPLETED with a non-empty `Owes:`**, not ARCHIVED.

---

## 🔓 11. OPEN FOR JOE

✅ **§F1 (F1c), §F2 (F2b) AND §3a (W1) WERE DELEGATED AND ARE SPENT.** What remains gates nothing in this leg:

1. **Leg E's descriptive title** (B1 + the 124,299-char L29). Its trigger has **fired**.
2. **§10 U1** — ⚠️ **partly disproved by measurement:** `M-DOC-ROADTREE` **does** have a node in `docs/ROADMAP.md` with a full six-child subtree, so the claim that both playing milestones are absent from the tree was wrong. `M-RP-LIVEFEED-REFRESH` really is absent (zero hits). The 66-absent-ID figure needs re-deriving before anything is built on it. **Naming the back-fill milestone is still Joe's.**
3. **The M-RP-SKIN countdown's home** and **M-RP-SELF-SURFACE's missing records** — the two `Owes:` items above.
4. **H1 · H2's scope rule · the visit-card verb · M_RP_MEMBERS §6/§8 · the GitHub board** — untouched by this leg.
