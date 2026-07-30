# RUNBOOK — M-DOC-ROADTREE Leg E, the two-way closure log
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this leg is, and the one sentence that governs it

**Surface:** the single longest line in `CLAUDE.md` — **124,299 chars**, the only line over 20,000. It is **one closure log end to end**, written in **three head notations**.

🔑 **IDENTIFY THE LINE BY CONTENT, NEVER BY LINE NUMBER.** It opens `**Next-active (UI/RP track):** M-RP2.6 ✅ CLOSED (J-410)`. It has moved twice (L29 → L25 at J-619) **because this milestone drains the file above it**. Locate it as *the only line whose length exceeds 20,000*, and assert that exactly one such line exists before doing anything else.

🛑 **THE GOVERNING SENTENCE: THIS LEG DELETES NOTHING WITHOUT A PER-PAIR HAND VERDICT.** Measured at J-622 — see §4.

---

## §1 — Grounding (P0 census, run 2026-07-30, J-622)

### §1a The head vocabulary — enumerated, not assumed

| region | span (chars) | size | head form | heads |
|---|---|---|---|---|
| **A** | 0 – 18,426 | 18,426 | `**Next-active (UI/RP track):**` *(once — the line's first record)* | 1 |
| **A** | " | " | `**M-<id>` bold, milestone-first | 19 |
| **B** | 18,426 – 110,488 | **92,062** | `J-nnn (` **at a `) / ` delimiter** | 51 *(4 stubs)* |
| **C** | 110,488 – 124,299 | **13,811** | `**M-<id>` bold, milestone-first | **25** |
| **C** | " | " | 🔑 **`**<emoji> M-<id>` — THE FOURTH HEAD FORM** | **1** |

**18,426 + 92,062 + 13,811 = 124,299 ✔**

🛑 **CORRECTED 2026-07-30 (J-627) BY P1's OWN ASSERT-AND-STOP. SUPERSEDED FIGURES ANNOTATED, NOT ERASED:**
- ⚠️ **C read `24` and TOTAL read `95`.** The `24` was measured from the **old** boundary **110,516**; the span in the same row is the **re-derived** 110,488 — **two figures from different passes, combined as if consistent.** From 110,488 the strict predicate finds **25**; the extra head is `**M-RP2.23 ✅ CLOSED (J-446)` at exactly **110,488**, **the very record whose position proved the old boundary wrong at J-622.**
- ⚠️ **THE HEAD VOCABULARY WAS INCOMPLETE.** `**M-` cannot match `**🟢 M-RP6.0 Pre-UI Node↔Client Functional Gate — ✅ DONE (J-473): GO.` at **113,632** — a **status emoji** sits between the bold marker and the milestone ID. Widened predicate `\*\*[^A-Za-z\*]{0,6}M-` yields 8 extra line-wide hits; **7 are mid-record** (the `Next-active` head's own tail at 28 · prose milestone lists at 34,215 / 34,314 · parentheticals at 76,913 / 86,035 / 87,614 / 95,653) and **exactly 1 is a genuine head.**
- ⚠️ **C's closure split used one predicate too.** `CLOSED (J-nnn)` alone gives 11; C also carries `DONE (J-nnn)` × 3 — J-473 · J-483 (`M-RP-CDP1`) · J-485 (`M-RP5.6`).

🔑 **FIFTH NOTATION VARIANT MISSED BY A PREDICATE IN THIS ARC, AND §1c PREDICTED IT.** The P0 census enumerated three head forms and asserted a total; **it never tested its own vocabulary for completeness.** ⇒ **ENUMERATING NOTATIONS IS NOT PROVING THE ENUMERATION COMPLETE. RUN A WIDENED PREDICATE AGAINST EVERY NARROW ONE AND READ ITS EXTRA HITS INDIVIDUALLY.**

📌 **The boundaries above are derived FROM head positions.** The sum is a *consequence* of independently located heads, not an input. Part two's boundaries put the B/C seam at 110,516; the true seam is **110,488** — C's first head `**M-RP2.23` opens 28 chars earlier.

📌 **A's arithmetic is cross-checked two independent ways:** 20 `CLOSED (J-nnn)` marks, and 19 bold-M heads + 1 `Next-active` head.

📌 **The A/B seam carries a one-off marker** — `**Entry (Rule 0): this PLAY → JOURNAL ` immediately precedes B's first head (`J-503 (` at 18,426). It is unique in the line. Treat it as A's tail, not as a head.

### §1b Counts

- **97 heads** total — 📌 **superseding `95` (J-627).**
- **B holds 4 stub segments that are NOT records** — under 100 chars: `J-447 (star-rating` **18** · `J-446 (password-field` **21** · `J-461 (PROTO-STATUS.2 — Track A, see head note` **46** · `J-448` **93**. Two are pointers; two are stubs whose real record is in C.
  - 📌 **CONVENTION NAMED 2026-07-30 (J-628) — NOT A DISCREPANCY, AND NOT REPAIRED.** These four figures measure the **payload only**. P1 measures **head-to-head**, which includes the trailing `) / ` delimiter (**4 chars**), and therefore reads **22 · 25 · 50 · 97**. Both readings are correct; **neither stated its convention.** ⇒ **STATE THE CONVENTION ALONGSIDE ANY SPAN FIGURE IN THIS LEG.**
- ⇒ **93 records** — 📌 **superseding `91` (J-627).**
- **C's 26 heads split 14 closure-bearing / 12 not** — 📌 **superseding *24 heads, 11 / 13* (J-627).** The 12 are forward-looking entries (M-RP5.4, M-RP6.1, M-RP5.5 …). ✅ **The delegated adoption at J-623 — *they travel into the archive annotated* — STANDS; only the count moved.**
- **11 known B↔C collisions** (§4).

### §1c 🛑 The negative result, which is the load-bearing one

**THERE IS NO UNIFORM RECORD HEAD IN THIS LINE.** Every single-predicate reading has failed:

- `**M-` matches **48 times inside B** as prose milestone-list mentions (five in one stretch at 41,259 · 41,320 · 41,350 · 41,368 · 41,382). It is **not** a head marker in B.
- `J-nnn (` matches **52 times in B** but only **50** are delimiters. It is **not** a head marker on its own.
- `**Next-active` matches **8 times**; exactly **1** is a head. The other 7 are mid-record forward-pointers.
- `CLOSED (J-nnn)` **cannot match B at all** — B puts the J-ref first. This is what produced the "93,205-char narrative hole" at J-621.

⇒ 🔑 **EACH REGION NEEDS ITS OWN PREDICATE, AND EACH PREDICATE NEEDS A SECOND INDEPENDENT ONE TO CHECK IT.** Four successive shapes for this line were each produced by trusting one predicate.

---

## §2 — Scope

**IN.** Parse the line into records; resolve the B↔C collisions; archive the result into `CLAUDE_HISTORY.md` under D-094.

**OUT.** ❌ Any change to `.rs` or `ui/**`. ❌ Any renumbering of designations (D-134). ❌ Repair of superseded measurements — grounding parts one and two stay in `M_DOC_ROADTREE.md` **annotated, not repaired** (D-131 family).

⇒ **PARSING, NOT MOVING** — until P3.

---

## §3 — P1, extraction

1. Assert exactly one line over 20,000 chars; assert its length is **124,299**. If either fails, **STOP** — the file has changed under the runbook and §1's offsets are void.
2. Extract heads per region using §1a's per-region predicate.
3. Assert head counts **1 + 19 / 51 / 25 + 1**. Any deviation **STOPS** the pass.
4. Cut records at head positions. Assert **(a)** the first head sits at **0** (no leading gap), **(b)** **zero duplicate head positions** (no overlap), **(c)** the head list is **strictly ascending**.
   - 🛑 **SHARPENED 2026-07-30 (J-628). SUPERSEDED TEXT ANNOTATED, NOT ERASED.** This step read: *“assert the concatenated record spans sum to **124,299** with no gap and no overlap.”* ⚠️ **THAT ASSERTION CANNOT FAIL.** Consecutive differences over a sorted list always sum to the span they cover — **the sum closes by construction.** It is the *shape* of verification, not verification. **(a), (b) and (c) are the three checks that can actually fail.** 📌 **Found by running the very runbook whose assert-and-STOP had just caught the fifth notation defect — the arc's own rule, turned on itself.**
5. Classify B's 51 segments into **47 records + 4 stubs** by the under-100-char test, then **read all four** to confirm each is a pointer or a stub. ⚠️ The threshold is a finding aid, not the verdict.
6. 🛑 **Assert nothing about record content from a marker.** A resolving citation proves a record exists, **not** that its substance is in it (Phase-0 §7).

### §3r — P1 RESULT: ✅ CLEARED 2026-07-30 (J-628)

**Re-run from §3.1 against the J-627-corrected assertions. Nothing stopped.**

| step | assert | measured |
|---|---|---|
| §3.1 | one line > 20,000 · length **124,299** | **1 · 124,299** ✔ |
| §3.3 | **1 + 19 / 51 / 25 + 1 = 97** | **97** ✔ |
| §3.4 | first **0** · duplicates **0** · strictly ascending | **0 · 0 · yes** ✔ |
| §3.5 | **47 records + 4 stubs** | **47 + 4** ✔ |

✅ **93 RECORDS.** ✅ **A's independent cross-check held** — 20 `CLOSED (J-nnn)` marks against 1 + 19 heads. ✅ **C splits 14 closure-bearing / 12 not** — 11 `CLOSED (J-nnn)` **+ 3 `DONE (J-nnn)`**: J-473 (the emoji-form head, @113,632) · J-483 `M-RP-CDP1` · J-485 `M-RP5.6 B`. Last head **@123,902**; smallest span **22**.

🔑 **THE LAST UNWIDENED NARROW PREDICATE WAS WIDENED AND ITS HITS READ INDIVIDUALLY.** `**Next-active` occurs **8** times line-wide: **1 in A (@0 — the head)**, **6 in B** (@41,090 · 48,588 · 49,396 · 62,308 · 68,885 · 74,262), **1 in C** (@121,699). All seven non-heads sit mid-sentence after `].` or `too).`. ⇒ **§1c's claim is now MEASURED rather than INHERITED.**

🔑 **§1c's `52 / 50` RECONCILES TO 51 BY MEASUREMENT.** Widening B's delimiter predicate to bare `J-nnn (` returns **2** extras: **@18,426 is the seam head, already counted**, and **@85,891 is `Prior: J-487 (` — a mid-record BACKWARD pointer.** ⇒ **50 delimiters + 1 seam head = 51**, with exactly one genuine non-head. 📌 **The apparent 51-vs-50 discrepancy was a missing convention, not a bad count.**

⚠️ **STILL NEVER TESTED, UNCHANGED BY THIS PASS:** within-A · within-B · A↔B · A↔C twins. **P1 parsed; it did not de-duplicate.**

🟡 **NEXT: §4, P2 — the eleven collisions, per-pair hand verdicts. NOT STARTED.**

---

## §4 — P2, collision resolution

### §4a The eleven measured at J-622 — read on both sides in full

| J-ref | milestone | B | C | verdict |
|---|---|---|---|---|
| J-446 | M-RP2.23 | **21** | 256 | **C only** — B is a bare mention |
| J-448 | M-RP2.25 | **93** | 297 | **C richer** — B is a stub |
| J-454 | M-RP4.3 | 391 | 397 | **DIVERGENT BOTH WAYS** |
| J-455 | M-RP4.1 | 356 | 216 | B ⊃ C |
| J-456 | M-RP4.5 | 533 | 458 | B ⊃ C |
| J-457 | M-RP2.30 | 745 | 389 | B ⊃ C |
| J-458 | M-RP2.30a | 431 | 240 | B ⊃ C |
| J-459 | M-RP2.31 | 767 | 383 | B ⊃ C |
| J-460 | M-RP2.31a | 303 | 202 | B ⊃ C |
| J-462 | M-RP5.0 | 1,037 | 372 | B ⊃ C |
| J-469 | M-RP5.3 | 1,460 | 697 | B ⊃ C |

**8 B ⊃ C · 2 C ⊃ B · 1 divergent.**

### §4b 🛑 Why no mechanical rule is permitted

- *Keep the longer copy* → deletes **J-446's runbook path** and **J-448's Shape-A detail**.
- *Keep B* → deletes **J-454's `temperature-indicator` dd-block** and the **`ui/docs/`-at-open session rule**, which exist **nowhere else in the repo**.
- *Keep C* → deletes **J-457's amber hex**, **J-459's deferred filter/search rationale**, **J-460's `box-sizing` evidence**.

⇒ 🛑 **PER-PAIR HAND VERDICT, NO EXCEPTIONS. WHEN A PAIR IS DIVERGENT, BOTH SURVIVE AND BOTH ARE ANNOTATED. NOTHING IS DELETED TO MAKE A COUNT COME OUT.**

### §4c The untested remainder

⚠️ **Only key collisions between B and C have been tested.** The remaining ~60 records have **never** been checked for twins — not within A, not within B, not A↔B, not A↔C. **P2 runs the same key test across all region pairs before any verdict is written.** The J-317 lesson (25,127 bytes of byte-identical duplication, undetected for weeks) is the reason this is not optional.

---

## §5 — P3, the move

Batch into `CLAUDE_HISTORY.md` under **one** `## ` heading, D-094 form (§F1c), matching Leg D's convention. Broken citations are **annotated at the site, never silently repaired** (D-131).

**C's 13 non-closure heads travel with the rest, annotated as forward-looking entries.** 📌 *Delegated adoption of Chat's lean at J-623, not a Joe lock — reversible.* Rationale: for several of them this stretch of `CLAUDE.md` is the only place the postponement rationale exists.

---

## §6 — Definition of Done

- [ ] Exactly one line over 20,000 chars asserted at pass open and at pass close.
- [ ] Head counts asserted **1 + 19 / 51 / 24**; spans sum to **124,299** with no gap or overlap.
- [ ] All four B stubs read individually and classified.
- [ ] Key-collision test run across **all** region pairs, not just B↔C.
- [ ] Every collision carries a written per-pair verdict; no deletion without one.
- [ ] `CLAUDE_HISTORY.md` gains one `## ` block; byte counts before/after recorded on disk.
- [ ] D-074: `JOURNAL.md` + `CLAUDE.md` PLAY + `docs/ROADMAP.md` + `tasks/M_DOC_ROADTREE.md` + this runbook in **one** commit, **and the ROADMAP child nodes too**.

📌 **"Commit pushed" is deliberately NOT a checklist item.** `Status: COMPLETED` in this header is the real signal.

---

## §7 — 🔓 Open, and who owns it

**§7a — none currently gating.** Leg E can run on §1–§6 as written.

📌 Joe's two live items belong to **other** legs and do not gate this one: §4 of `RUNBOOK_ROADTREE_LEGB_BIS.md` (governs one line, `JOURNAL_ARCHIVE.md:17052`) · the back-fill milestone's name (Leg F's sweep).

---

## §8 — Traps, all earned

- ⚠️ **EOL:** `Filesystem:edit_file` rewrites the whole file to LF. Harmless under `autocrlf=true` (same blob) but **a disk byte count that falls while content is added is reporting on the tool, not the content.**
- ⚠️ **`Windows-MCP:FileSystem` mode=write emits CRLF.** Normalise and re-measure after every new file.
- ⚠️ **Bound the regexes.** `L29` matches inside `L2951`.
- ⚠️ **`.Contains()` is case-sensitive and lies on surrogate-pair emoji.** Compare by code point.
- ⚠️ **`[IO.File]::ReadAllBytes` with a relative path ignores `cd`.** Absolute paths only.
- ⚠️ **A failed `ReadAllLines` can leave the previous buffer in `$ls`.** Assert the path exists.
- ⚠️ **Never `create_file` for the user's disk** — it writes to the Claude sandbox.
- 🛑 **A summation that closes by construction is not verification; it is the shape of verification.** If a gap is filled with arithmetic, the sum cannot fail and proves nothing.
- 🛑 **Sampling cannot catch a notation error.** Seven mid-record samples of a `J-nnn (…)` log all read as design prose — real evidence for a false conclusion. **Sample AT boundaries.**
- 🛑 **Annotate wrong measurements, never delete them.** At J-620 a superseded table was the only thing that disproved its own replacement.
