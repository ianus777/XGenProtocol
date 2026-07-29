# M-DOC-ROADTREE — LEG D RUNBOOK — the `CLAUDE.md` B2 archive move
> **Status**: ACTIVE  
> Version: 1.1  
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

1. **Freeze the input.** `git status --porcelain` empty. A clean tree is the undo and stops being one at commit.
2. **Extract by range**, never by headline.
3. **Annotate before moving** (§B). Annotations travel with the block.
4. **Append to `CLAUDE_HISTORY.md`** in live-head order under D-094; boundary preserved (history J-81–J-405, head J-519–J-604, zero duplication).
5. **Delete from `CLAUDE.md`** highest line first, so earlier ranges keep their indices.
6. **Run V1–V8 before writing and again after.**
7. **Never a rewrite.** A moved block is byte-identical to its annotated pre-move text.

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

## ⚠️ 6. §E — THE HOLD SET

**L75 — the M-RP-SKIN countdown.** Re-points the M-RP-7.1 SE-corner-triangle obligation at **M-RP-SKIN**, which has **no task doc and no ROADMAP node**. Archiving it files an undischarged obligation against nothing. **Home the countdown first.**

**L235 — M-RP-SELF-SURFACE.** Claims a design lock for a milestone with no task doc and no node; §B cannot check it. **Held pending a reading pass.**

📌 **ONE SOFT ITEM, JOE'S.** `--region-border: 1px` and `--region-edge: var(--s4)` are live in `ui/assets/skin.css`, but the constraint — *the edge must read against both the backdrop `--s5` and the tile body `--s`* — exists only in a block in the move set. `skin.css` is Joe's.

---

## ⚠️ 7. DEFECTS FOUND IN THIS PASS

- **🔑 THE PARTITION WAS INHERITED, AND THAT IS THE WHOLE DEFECT.** v1.0 classified 22 blocks because the census said *22 non-work · 17 live · 42 ✅*. **The 17 were "live" by lead symbol only** — the exact reading ruling E was made to forbid — and **eleven of them are closed work.** ⇒ **A CLASSIFICATION THAT ACCEPTS SOMEONE ELSE'S PARTITION HAS NOT CLASSIFIED; IT HAS RE-SORTED ONE BUCKET.**
- **THE MOVE SET WAS CARRIED AS A COUNT ACROSS FOUR DOCUMENTS** — kickoff 7, brief 6, runbook v1.0 50, actual 62. Same recurring class: *a claim narrower than the thing it describes, reused as if complete.*
- **A SUSPECT LIST IS NOT A CHECK.** v1.0 verified the `M-RP-LOCK-RECHECK REMAINS ACTIVE` clause on the three blocks the documents named. Grepping the file finds it on **five** — L241, **L243**, L245, **L247**, L249 — and L247 is a ✅ block, outside every earlier list. ⇒ **RUN THE PREDICATE OVER THE WHOLE CORPUS, NOT OVER THE NAMES YOU WERE HANDED.**
- **`tasks/M_DOC_ROADTREE_LEGD_PHASE0_BRIEF.md` HAD A NON-CONFORMING HEADER** — corrected at v1.5 with the correction recorded in the file. ⚠️ `JOURNAL.md`'s header carries the same defect and is **left as found**; rewriting a canonical header is not this leg's business.
- **🔑 `Filesystem:edit_file` NORMALISES THE ENTIRE FILE TO LF — MEASURED.** `JOURNAL.md` was CRLF on disk (1,534,320 disk vs 1,526,965 blob; the 7,355 difference is exactly its line count). After an edit that **added** ~5 KB, disk read **1,531,984 with 0 CRLF**. ⇒ **A DISK BYTE COUNT THAT FALLS WHILE CONTENT IS ADDED IS REPORTING ON THE TOOL.** Harmless to git — `git diff --numstat` showed 27 insertions, 1 deletion, no EOL churn.
- **THE CRLF CONVENTION NOTE WAS WRONG.** `CLAUDE.md` is LF, not CRLF; `docs/ROADMAP.md` is the CRLF file. Corrected in §1 by measurement.
- **STATUS-HEADER SCAN NOISE.** **Five** task docs return multiple `Status` matches in their first four lines (`M_RP2_22`, `M_RP6_1E_A`, `M_RP6_1E_B`, `RUNBOOK_PROTO_STATUS_2`, `RUNBOOK_STATUS`). Not blocking; filed so it is not rediscovered as a finding.

---

## 🔓 8. OPEN FOR JOE — NONE OF IT GATES THE MOVE

1. **§3a — W1 / W2 / W3** for L195, L199, L201. Recommend **W1**.
2. **Leg E's descriptive title** (B1 + the 124,299-char L29).
3. **§10 U1** — add the `M-DOC-ROADTREE` node, and **name the ROADMAP back-fill milestone** for the 66 absent IDs.
4. **The M-RP-SKIN countdown's home** (§E), M-RP-SKIN being an appearance milestone.
