# M-DOC-ROADTREE — LEG D RUNBOOK — the `CLAUDE.md` B2 archive move
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 🛑 0. WHAT THIS IS

The execution document for **Leg D**, scoped by §6b to **B2 only** (`CLAUDE.md` L71–261). It carries the block classification, the move procedure and the checks. It does **not** re-open §6a, §6b or §6c — those are ruled.

🔑 **IT EXISTS BECAUSE THE MOVE SET WAS A NUMBER, NOT A CLASSIFICATION.** The session kickoff carried *7 self-declared-closed blocks*; `M_DOC_ROADTREE_LEGD_PHASE0_BRIEF.md` §6a carries *6*. Neither list survived contact with the blocks. ⇒ **§A below is the first per-block record with an evidence string on every row.**

---

## 🔒 1. GROUNDING RE-MEASURED AT OPEN — 2026-07-29 06:41 +02:00

Every number below was measured this session, not inherited.

| item | value | lens |
|---|---|---|
| HEAD | `c857ad0` = `origin/main`, tree clean | `git rev-parse` / `git status --porcelain` |
| `CLAUDE.md` | 640,645 / 640,645 · 821 lines | disk / `git cat-file -s` |
| `CLAUDE_HISTORY.md` | 869,178 / 869,178 · 2,218 lines | disk / blob |
| `JOURNAL.md` | 1,534,320 / 1,526,965 · 7,355 lines | disk / blob (mixed EOL) |
| `DECISIONS.md` | 564,366 / 564,366 · 4,917 lines | disk / blob |
| `docs/ROADMAP.md` | 44,758 / 44,313 · 445 lines | disk / blob |
| `CLAUDE.md` longest line | 124,299 chars (L29) | `.Length` max |
| B2 block count | **81**, L71 → L261 | `> ###` headline scan over L71–271 |
| symbol census, by code point, total asserted | ✅ 42 · 🟢 15 · 🔒 9 · 🔑 7 · 🛑 3 · ⚠️ 3 · 🟡 2 = **81** ✔ | `ConvertToUtf32`, V5 form |
| ports 5173 · 5174 · 9222 · 9322 · 9422 | all free | `Test-NetConnection` |

⚠️ **A BLOCK IS A RANGE.** Every measurement above and below operates on `> ###` headline → line before the next headline, trailing blanks trimmed. Nineteen of the 81 carry continuation lines; the earlier headline-only pass had to be redone.

---

## 🛑 2. §A — THE 22 NON-WORK BLOCKS, CLASSIFIED BY READING

The 42 ✅ blocks are not re-litigated: **42 of 42 self-declare closed**, so the symbol is reliable on the DONE set. The lie is confined to the 22 blocks that carry no state symbol (🔑 🔒 🛑 ⚠️). Each row below was read on disk; the evidence column quotes the block's own words.

| L | range | subject | evidence in the block's own text | verdict |
|---|---|---|---|---|
| 75 | 75–83 | N-120 resize/index defect | *"→ M-RP7.3 OPENS WITH IT"* · re-points the SE-corner countdown at **M-RP-SKIN** | **HOLD** — see §E |
| 87 | 87–89 | backdrop `.region-split` → `.region-shell` (J-521) | reports a change made; declares no state | stateless — eligible |
| 90 | 90–99 | the three-for-three defect pattern | defect class, no milestone claim; fourth-instance version lives in `ui/docs/xgen-ui-notes.md` | stateless — eligible |
| 103 | 103–107 | N-124 — *"tile↔hole is 0"* is not observable | finding; N-124 headed in `ui/docs/xgen-ui-notes.md` | stateless — eligible |
| 108 | 108–114 | N-124a — two baselines more precise than their measurement | finding; N-124a headed in the UI notes | stateless — eligible |
| 175 | 175–176 | *"THE TEXT PROCESSOR IS WIRED TO NOTHING"* | claim is **false today** — `tasks/M_RP_PROCESSOR_WIRE.md` reads **COMPLETED v1.4**; the rule is homed as the **D-099 third amendment** (J-610) | **STALE-CLOSED** — annotate + archive |
| 177 | 177–178 | retraction and answer (J-561) | *"M-RP-SETTINGS IS CLOSED"* | **self-declared closed** — archive |
| 179 | 179–180 | the J-564 five-item sequencing lock (Joe-locked) | still names ②–⑤, but ① is marked 🟢 while `M_RP_MSG_NEWLINE.md` reads **COMPLETED v1.0** | **LIVE, partially stale** — stays, annotate ① |
| 193 | 193–194 | Leg C closure | *"LEG C ✅ CLOSED"* · also *"LEG D … ITS FORMAT IS 🔓 UNRULED"*, ruled since | **self-declared closed** — archive + annotate the spent 🔓 |
| 195 | 195–196 | M-DOC-ROADTREE — Leg C open and runnable | *"🟢 PLAY · LEG C IS OPEN AND RUNNABLE"* — Leg C closed at J-604 | **SUPERSEDED** — disposition 🔓 §3 |
| 197 | 197–198 | M-RP-LIVEFEED-REFRESH (J-601) | *"🟢 PLAY · v1.10"* — `M_RP_LIVEFEED_REFRESH.md` reads **ACTIVE v1.10** ✔ | **LIVE, correct** — stays |
| 199 | 199–200 | M-DOC-ROADTREE §6/§7 ruled | *"🟢 PLAY"* — those legs closed at J-604 | **SUPERSEDED** — disposition 🔓 §3 |
| 201 | 201–202 | M-DOC-ROADTREE Leg B (J-599) | reports Leg B's result; declares no close of its own | **SUPERSEDED** — disposition 🔓 §3 |
| 203 | 203–204 | M-RP-MEMBERS Leg C | *"BLOCKED AND ⏸️ PAUSED"* — `M_RP_MEMBERS.md` **ACTIVE v1.15** ✔ | **LIVE, correct** — stays |
| 209 | 209–210 | H1 / H2 — the address book at rest | open question, no state claim; still 🔓 for Joe | **LIVE, correct** — stays |
| 235 | 235–236 | M-RP-SELF-SURFACE design walk (J-576) | claims a design lock; **no `tasks/` doc and no ROADMAP node exists for M-RP-SELF-SURFACE** | **STATE UNKNOWN** — see §E |
| 241 | 241–242 | D-123 — the seat division | *"⚠️ M-RP-LOCK-RECHECK REMAINS ACTIVE"* — task doc reads **COMPLETED v1.2**; D-123 itself is headed in `DECISIONS.md` | **STALE-CLOSED** — annotate + archive |
| 243 | 243–244 | D-122 FINAL — window vocabulary | vocabulary lock, headed in `DECISIONS.md`; no state claim | stateless — eligible |
| 245 | 245–246 | D-122 + M-RP-VIEW-BINDING (J-571) | *"⚠️ M-RP-LOCK-RECHECK REMAINS ACTIVE"* — same contradiction | **STALE-CLOSED** — annotate + archive |
| 249 | 249–250 | M-RP-LOCK-RECHECK Leg A (J-569) | *"LEG A DONE, MILESTONE STILL ACTIVE"* — task doc **COMPLETED v1.2** | **STALE-CLOSED** — annotate + archive |
| 251 | 251–252 | M-RP-SWEEP (J-568) | *"DONE (J-568)"* | **self-declared closed** — archive |
| 255 | 255–256 | M-RP-PROCESSOR-SEED (J-566) | *"⚠️ SUPERSEDED AT J-567 — CLOSED"* | **self-declared closed** — archive |

**Tally — 22 = 4 self-declared closed · 4 stale-closed · 3 superseded · 3 live-correct · 1 live-partially-stale · 5 stateless-and-homed · 2 held (L75, L235).**

🔑 **NEITHER SOURCE COUNT WAS RIGHT, AND THE REASON IS THE SAME BOTH TIMES.** L75 appears on **both** lists as self-declared closed and declares no closure at all — it opens the next milestone. L241 appears on one list and declares another milestone *active*. ⇒ **A BLOCK THAT NAMES A CLOSURE IS NOT A BLOCK THAT DECLARES ITS OWN.** The token `CLOSED`/`DONE` inside a block says nothing about the block; §9 of the brief predicted this and the marker triage was run anyway one layer up, as a count.

---

## 🔒 3. §B — THE THIRD INPUT: WHAT THE BLOCK SAYS vs WHAT IS TRUE NOW

§6a's **E** classifies by what a block's own text says. **Four blocks say a milestone is live that is closed on disk** (L175, L241, L245, L249) and **three say a milestone of this very arc is playing that Leg C closed** (L195, L199, L201). E alone cannot dispose of them: what they say was true when written.

🔒 **THE RULE.** Every block whose claim names a milestone is checked against `docs/ROADMAP.md` **and** its `tasks/` doc header before it moves. The check is cheap wherever a task doc exists; where none exists the block is **held**, not guessed at (L235).

🔒 **THE ANNOTATION FORM** — D-131 applied to state instead of citations, inline at the site, never a silent repair:

`[⚠️ CLAIM STALE 2026-07-29 — M-RP-LOCK-RECHECK reads COMPLETED v1.2 in tasks/M_RP_LOCK_RECHECK.md. Claim was true when written.]`

### 🔓 3a — THE ONE OPEN QUESTION: WHERE DO THE SUPERSEDED THREE GO? (L195, L199, L201)

§6a's **D** sends the residue to the live head. But these three declare `🟢 PLAY` on **M-DOC-ROADTREE legs that are closed** — a false state assertion in the head that Leg D exists to drain.

| option | ① user-visible impact | ② tier | ③ resource cost |
|---|---|---|---|
| **W1 — archive all three, annotated** | none | none | 3 annotations; the head loses 3 stale 🟢 PLAY declarations; consistent with the 4 stale-closed rows above |
| **W2 — leave in head under D-131, annotated** | none | none | 3 annotations; head keeps 3 blocks that say a closed leg is playing |
| **W3 — leave untouched** | none | none | zero; the head keeps asserting Leg C is runnable |

**Recommend W1.** The four stale-closed rows and these three differ only in *who* closed them — a task doc or this arc. Treating them differently would put the same defect on two shelves. ⚠️ **W1 widens §6a's D by three blocks and is therefore Joe's, not Chat's.** It is not a gate: the other 19 rows move without it.

---

## 🔒 4. §C — THE MOVE PROCEDURE

1. **Freeze the input.** Confirm `git status --porcelain` empty. A clean tree is the undo and stops being one at commit (Leg C, three times).
2. **Extract by range.** For each block in the move set: headline line → line before next headline, trailing blanks trimmed. Never by headline alone.
3. **Annotate before moving**, in place, per §3 — annotations travel with the block into history.
4. **Append to `CLAUDE_HISTORY.md`** in live-head order, under the existing D-094 archive convention, boundary preserved (history J-81–J-405, head J-519–J-604, zero duplication measured).
5. **Delete from `CLAUDE.md`** in one pass, highest line first, so earlier ranges keep their indices.
6. **Run V1–V8 (§D) before writing anything to disk and again after.**
7. **Never a rewrite.** D-094 makes archiving a move; a moved block is byte-identical to its annotated pre-move text.

📌 **THE MOVE SET IS THE 42 ✅ BLOCKS + the 4 self-declared closed + the 4 stale-closed = 50 blocks**, ± the 3 superseded pending §3a. ⚠️ **This is not the 49 the kickoff carried.** The composition changed, not just the count.

---

## 🔒 5. §D — THE CHECKS

- **V1** — every moved block appears in `CLAUDE_HISTORY.md` byte-identical to its annotated pre-move text.
- **V2** — every designation cited by a moved block resolves in its canonical file, suffix convention encoded in the pattern (`D-030a`, `N-124b`).
- **V3** — blocks before = blocks remaining + blocks archived. 81 in, 81 out.
- **V4** — no J-number is a lead citation in both the live head and the history file.
- **V5** — symbol census by code point with a total assertion, before and after.
- **V6** — every size reported with its lens named; disk bytes and `git cat-file -s` never quoted as each other. ⚠️ **AND THE EDITING TOOL IS PART OF THE LENS** — see §7: `Filesystem:edit_file` rewrites the **whole file** to LF, so a disk byte count taken after an edit is not comparable to one taken before it. **Compare blobs, not disk, across an edit.**
- **V7** — the resulting live head measured against D-094's *small live working head*, number stated not asserted.
- **V8 — WHOLE-DOCUMENT INVARIANTS ASSERTED BEFORE EVERY WRITE**, not the size of the change: total line count, block count, symbol census, first line, **last line intact**, L29 length unchanged at 124,299. ⚠️ The length assertion is the only thing that has ever caught these — a positional rewrite silently deleted a 2,063-char §10 with no failed anchor.

---

## ⚠️ 6. §E — THE HOLD SET: BLOCKS THAT MUST NOT MOVE YET

**L75 — the M-RP-SKIN countdown.** The block re-points the M-RP-7.1 SE-corner-triangle obligation at **M-RP-SKIN**. `M_RP7_3_MUTATION_ALGEBRA.md` is COMPLETED, so N-120's own successor is discharged — but **M-RP-SKIN has no `tasks/` doc and no node in `docs/ROADMAP.md`**. ⇒ Archiving this block files an undischarged obligation against a milestone that does not exist anywhere else. **Home the countdown first, then move the block.**

**L235 — M-RP-SELF-SURFACE.** Claims a design lock for a milestone with no task doc and no roadmap node. Its state cannot be checked, so §B cannot clear it. **Held pending a reading pass.**

📌 **ONE SOFT ITEM, JOE'S — NOT HELD BY CHAT.** `--region-border: 1px` and `--region-edge: var(--s4)` are live in `ui/assets/skin.css`, but the constraint (*the edge must read against both the backdrop `--s5` and the tile body `--s`*) exists only in a block in the move set. `skin.css` is Joe's; Chat will not move that constraint into it.

---

## ⚠️ 7. DEFECTS FOUND IN THIS PASS

- **THE MOVE SET WAS CARRIED AS A COUNT ACROSS THREE DOCUMENTS AND NEVER RE-READ.** Same recurring class: *a claim narrower than the thing it describes, reused as if complete*. Caught by reading the blocks, not by re-reading the documents. ⇒ **A SET IS NOT A NUMBER; IT IS A LIST WITH EVIDENCE ON EVERY ROW.**
- **`tasks/M_DOC_ROADTREE_LEGD_PHASE0_BRIEF.md` HAD A NON-CONFORMING HEADER** — `> **Status:**` instead of `> **Status**:`, `Version` before `Status`, and **Date, Language, Credits and License all absent**. **Corrected at v1.5 with the correction recorded in the file**, not silently. ⚠️ `JOURNAL.md`'s own header carries the same defect and is **left as found** — it is a canonical document and rewriting its header is not this leg's business.
- **🔑 `Filesystem:edit_file` NORMALISES THE ENTIRE FILE TO LF — MEASURED, NOT ASSUMED.** `JOURNAL.md` was all-CRLF on disk (1,534,320 disk vs 1,526,965 blob — the 7,355 difference is exactly its line count). After one edit that **added** ~7 KB, disk read **1,531,984 and 0 CRLF**: the file had been converted wholesale. ⇒ **A DISK BYTE COUNT THAT FALLS WHILE CONTENT IS ADDED IS REPORTING ON THE TOOL, NOT THE EDIT.** 📌 **Harmless to git here** — `core.autocrlf=true` gives the same blob, and `git diff --numstat` showed **27 insertions, 1 deletion**, no EOL churn. ⚠️ **BUT `CLAUDE.md` AND `docs/ROADMAP.md` ARE DOCUMENTED CRLF FILES**, and the move edits `CLAUDE.md`. Expect the conversion, assert the **blob**, and never quote a post-edit disk number against a pre-edit one.
- **STATUS-HEADER SCAN NOISE.** **Five** task docs return multiple `Status` matches in their first four lines (`M_RP2_22`, `M_RP6_1E_A`, `M_RP6_1E_B`, `RUNBOOK_PROTO_STATUS_2`, `RUNBOOK_STATUS`). Not blocking Leg D; filed so it is not rediscovered as a finding.

---

## 🔓 8. OPEN FOR JOE — NONE OF IT GATES §A OR §B

1. **§3a — W1 / W2 / W3** for the three superseded blocks. Recommend **W1**.
2. **Leg E's descriptive title** (B1 + the 124,299-char L29). Letter is mechanical; the title is naming (D-123).
3. **§10 U1** — add the `M-DOC-ROADTREE` node, and **name the ROADMAP back-fill milestone** for the 66 absent IDs. ⚠️ U1 without the named milestone makes a 66-behind branch look maintained.
4. **The M-RP-SKIN countdown's home** (§E), since M-RP-SKIN is an appearance milestone.
