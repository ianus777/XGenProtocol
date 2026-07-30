# RUNBOOK — M-DOC-ROADTREE Leg E, the two-way closure log
> **Status**: PENDING  
> Version: 1.0  
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
| **B** | 18,426 – 110,488 | 92,062 | `J-nnn (` **at a `) / ` delimiter** | 51 |
| **C** | 110,488 – 124,299 | 13,811 | `**M-<id>` bold, milestone-first | 24 |

**18,426 + 92,062 + 13,811 = 124,299 ✔**

📌 **The boundaries above are derived FROM head positions.** The sum is a *consequence* of independently located heads, not an input. Part two's boundaries put the B/C seam at 110,516; the true seam is **110,488** — C's first head `**M-RP2.23` opens 28 chars earlier.

📌 **A's arithmetic is cross-checked two independent ways:** 20 `CLOSED (J-nnn)` marks, and 19 bold-M heads + 1 `Next-active` head.

📌 **The A/B seam carries a one-off marker** — `**Entry (Rule 0): this PLAY → JOURNAL ` immediately precedes B's first head (`J-503 (` at 18,426). It is unique in the line. Treat it as A's tail, not as a head.

### §1b Counts

- **95 heads** total.
- **B holds 4 stub segments that are NOT records** — under 100 chars: `J-447 (star-rating` **18** · `J-446 (password-field` **21** · `J-461 (PROTO-STATUS.2 — Track A, see head note` **46** · `J-448` **93**. Two are pointers; two are stubs whose real record is in C.
- ⇒ **91 records.**
- **C's 24 heads split 11 closure-bearing / 13 not.** The 13 are forward-looking entries (M-RP5.4, M-RP6.1, M-RP5.5, M-RP5.6 …).
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
3. Assert head counts **1 + 19 / 51 / 24**. Any deviation **STOPS** the pass.
4. Cut records at head positions. Assert the concatenated record spans sum to **124,299** with no gap and no overlap.
5. Classify B's 51 segments into **47 records + 4 stubs** by the under-100-char test, then **read all four** to confirm each is a pointer or a stub. ⚠️ The threshold is a finding aid, not the verdict.
6. 🛑 **Assert nothing about record content from a marker.** A resolving citation proves a record exists, **not** that its substance is in it (Phase-0 §7).

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
