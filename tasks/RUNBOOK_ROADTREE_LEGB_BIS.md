# RUNBOOK — M-DOC-ROADTREE Leg B-bis: the journal repair
> **Status**: PENDING  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and its state

**Leg B-bis of `M-DOC-ROADTREE` — the journal repair.** Parent: `tasks/M_DOC_ROADTREE.md` v1.18. Governed by **`D-134`** (designations are issued unique; a duplicate is repaired by lettered split, bare number retired).

⚠️ **THIS RUNBOOK IS `PENDING`. §4 carries ONE open question and it is Joe's** — it decides exactly **one line** of the work. Everything else is measured and settled. Status flips to `ACTIVE` when §4 carries a lock.

🔒 **UNBLOCKED BY §8b's RULING (Joe, 2026-07-29, J-618):** *`ARCHIVED` means no new records; correction of an existing record is permitted whenever a defect is found.* Under the previous reading this leg could not exist.

🔑 **THIS LEG EXISTS BECAUSE OF A RULING, NOT A DISCOVERY.** The defects were measured at J-599 and J-604. What changed is that repairing them became legal.

---

## §1 — Grounding (re-verified on disk 2026-07-29 at `5a0086b`; nothing here is quoted from an earlier pass)

**`JOURNAL_ARCHIVE.md`** — 2,333,803 B · 17,416 lines · **358 entry headings, 351 unique.** Seven designations appear twice, and they are **two different defects**.

### §1a — Defect A: five byte-identical duplicate pairs

| designation | first copy | second copy | body length each | identical |
|---|---|---|---|---|
| `J-317` | L1372 | L1400 | 5,100 | ✅ |
| `J-318` | L1332 | L1352 | 4,046 | ✅ |
| `J-319` | L1292 | L1312 | 4,482 | ✅ |
| `J-320` | L1248 | L1270 | 5,801 | ✅ |
| `J-321` | L1208 | L1228 | 5,698 | ✅ |

**25,127 bytes.** 🛑 **THESE ARE NOT COLLISIONS AND MUST NOT BE SUFFIXED** — `D-134` §6: the test is whether the bodies differ, and these are byte-identical ⇒ **delete one copy of each.** Suffixing would enshrine an accident as two distinct events.

### §1b — Defect B: two genuine collisions

| designation | line | title | takes |
|---|---|---|---|
| `J-044` | **17057** | *BATCH_FLAG_ph2.md: M1–M3 implemented (code complete, M4 walkthrough pending)* | **`J-044a`** |
| `J-044` | **16957** | *BATCH_FLAG_ph2.md: implementation review; error message fix; documentation updates* | **`J-044b`** |
| `J-045` | **17161** | *XGEN_CORE_SPLIT_ph2.md: xgen-core crate split complete* | **`J-045a`** |
| `J-045` | **17128** | *Design note: `--batch` as a primary AI tool for tuning and debugging* | **`J-045b`** |

🔑 **THE ASSIGNMENT IS `D-134` §2 APPLIED, AND THE HIGHER LINE NUMBER TAKES `a`.** `a` means **first in the record, not first in the file**; `JOURNAL_ARCHIVE.md` runs **newest-first**, so the higher line number is the **earlier** record. 📌 **`J-044`'s content corroborates it** — *implemented* precedes *implementation review*. ⚠️ **`J-045`'s content is neutral between its two entries, so that assignment rests on sort direction alone. Stated, not hidden** (parent §4c).

### §1c — Defect C: the header contradicts itself

`JOURNAL_ARCHIVE.md`'s own header reads *"Live window (J-395 … J-376) continues in `JOURNAL.md`"*. **`JOURNAL.md` holds 242 entries, J-619 … J-376** (measured 2026-07-29). ⚠️ **Under the pre-ruling reading this sentence was unfixable by its own rule.** It is now simply a repair, and it is the smallest piece of evidence that the ruling was needed.

---

## §2 — The citation set: **12**, not 28

🛑 **THE `28` FIGURE IN EVERY PRIOR RECORD IS A REFERENCE COUNT, NOT A CITATION COUNT, AND IT IS SUPERSEDED HERE.** §8b measured 26 + 22 = 48 bare references on 2026-07-26 and reported 28 as "surviving re-points" **without partitioning citations from discussion**. Re-run 2026-07-29 over all 507 `.md` files (excluding `.claude/`, `target/`, `node_modules/`) with a bounded predicate, then **read individually**: **87 bare hits → 4 definition sites · 71 discussion · 12 citations.**

| # | file | line | resolves to | file status |
|---|---|---|---|---|
| 1 | `DECISIONS.md` | 2827 | `J-044a` | ACTIVE |
| 2 | `DECISIONS.md` | 2874 | `J-044a` | ACTIVE |
| 3 | `CLAUDE.md` | 561 | `J-045a` | ACTIVE |
| 4 | `docs/tests/BATCH_FLAG_ph2.md` | 5 | `J-044b` | COMPLETED |
| 5 | `docs/tests/BATCH_FLAG_ph2.md` | 21 | `J-045b` | COMPLETED |
| 6 | `docs/tests/BATCH_FLAG_ph2.md` | 338 | `J-044a` | COMPLETED |
| 7 | `docs/tests/BATCH_FLAG_ph2.md` | 383 | `J-044b` | COMPLETED |
| 8 | `docs/tests/XGEN_CORE_SPLIT_ph2.md` | 5 | `J-045a` | COMPLETED |
| 9 | `tasks/archive/M2_NODE_PIPE_SERVER.md` | 23 | `J-044a` | COMPLETED |
| 10 | `tasks/archive/M2_NODE_PIPE_SERVER.md` | 37 | `J-044a` | COMPLETED |
| 11 | `JOURNAL_ARCHIVE.md` | 17052 | `J-044a` | 🛑 **ARCHIVED** |
| 12 | `docs/ROADMAP_ARCHIVE_2026-07-26.md` | 348 | `J-045a` | 🛑 **ARCHIVED** |

🔑 **`BATCH_FLAG_ph2.md` SPLITS WITHIN ITSELF — `:338` → `a`, `:383` → `b` — WHICH IS THE PROOF THAT NO MECHANICAL REWRITE IS POSSIBLE.** `:338` heads *Implementation Notes, Session 19*; `:383` heads *Verification Results, Session 19 (continued), all 14 M4 checks passed*. **Same file, same session, different entries.** ⚠️ **A find-and-replace over this file would corrupt one of the two.**

⚠️ **AND THE FIRST PASS OF THIS MEASUREMENT WAS WRONG, WHICH IS WHY THE TABLE ABOVE IS PER-HIT AND NOT A COUNT.** A keyword classifier (*collision · split · suffix · bare · designation*) was used to separate discussion from citation. It **dropped `CLAUDE.md:561` and `ROADMAP_ARCHIVE:348`** — both real citations — while **keeping seven of this session's own sentences** as citations. 🔑 **A keyword classifier applied to a corpus that discusses itself fails in both directions at once.** It was caught only because §8b's older per-file table named files the filter had emptied. ⇒ **Whoever re-derives this must read hits, not count them.**

---

## §3 — Order of work, and why it is not free

1. **§1a deletions first.** Removing 25,127 B shifts every line number below the deletion points. All five pairs sit at L1208–L1400, which is **above** everything in §1b and §2 ⇒ **doing them first invalidates the line numbers in §1b and §2.**
2. 🛑 **THEREFORE: §1b and §2 FIRST, §1a LAST.** ⚠️ **The tables in §1b and §2 are positional and this runbook creates the churn that breaks them** — the same defect this milestone corrected in `L29 → L25` at J-619, and it is avoidable here only by ordering.
3. **Re-verify before each write.** Locate each target by its **content** (the heading text, the citing sentence), never by the stored line number alone. 📌 **The line numbers in this document are an index for finding things, not an address to write to.**
4. **§1c header fix** — independent of the others, any time.

---

## §4 — 🔓 OPEN, JOE'S: MAY A CITATION INSIDE AN ARCHIVED FILE BE RE-POINTED?

**Two of the twelve sit in ARCHIVED files: `JOURNAL_ARCHIVE.md:17052` and `docs/ROADMAP_ARCHIVE_2026-07-26.md:348`.** 📌 **And the second is nearly moot — Leg G deletes that file if Leg B clears** ⇒ in practice this question governs **one line.**

`JOURNAL_ARCHIVE.md:17052` reads: *"2. ~~Mr. Code implements the batch flag~~ ✅ Done — see J-044"*. After the split, `J-044` names nothing.

⚠️ **§8b's ruling permits three kinds of repair — removing a duplicate, splitting a designation, correcting header metadata. This is a fourth: editing the BODY of a frozen entry.** The boundary Joe set was *no new records*, substance untouched. **Re-pointing a pointer arguably does not touch substance — but it does edit the body, and that was not among the three.**

### (A) RE-POINT IT — `J-044` → `J-044a`
1. **Consequence:** the citation resolves. A reader following it lands on the right entry.
2. **Cost:** one token changed in one archived line. Covered by the `Repaired:` line either way.
3. **Tier:** treats a dead pointer as exactly the class of defect the ruling was made for.

### (B) LEAVE IT BARE — the split is recorded forward only
1. **Consequence:** the archived body is untouched. ⚠️ **But the archive then contains a citation to a designation that no longer exists** — and `D-134` §2 retires the bare number precisely so that nothing resolves to it.
2. **Cost:** zero edits; the forward `JOURNAL.md` entry carries the resolution.
3. **Tier:** maximal fidelity to *verbatim and unaltered*, at the price of one knowingly-dead pointer.

🔑 **WHY THIS IS JOE'S AND §5 OF THE LEG A RUNBOOK WAS NOT.** Both options here are **honest** — neither asserts anything false. It is a trade between immutability and navigability, two goods. **A choice between honest options is Joe's; whether to assert something unknown is not a choice at all.**

📌 **Chat's lean is (A)**, on the ruling's own logic: a pointer that resolves to nothing is a defect, and the ruling exists so defects can be fixed. **But the lean is weak and the leg is fine either way** — under (B) the leg loses one row.

---

## §5 — Verification

- **Heading census re-run:** `JOURNAL_ARCHIVE.md` must go **358 headings / 351 unique → 353 / 353.** 🔑 **Both numbers, and they must be EQUAL** — equality is the assertion that no duplicate designation survives anywhere in the file, which a spot-check of the seven cannot prove.
- **Byte delta:** −25,127 from the deletions, plus the suffix and header deltas. **Measured, not predicted.**
- **Bare-designation sweep:** `J-044(?![0-9a-z])` and `J-045(?![0-9a-z])` re-run repo-wide. ⚠️ **The count will NOT be zero** — 71 discussion hits legitimately remain, and this document adds more. ⇒ **assert that the twelve rows of §2 are gone, individually, not that the pattern is absent.** 🔑 **A sweep that expects zero here is a check that must fail.**
- **`J-317`–`J-321`:** each must appear **exactly once** as a heading, and the surviving body must be byte-identical to the pre-deletion copy. **Compare against git, not against memory.**
- **Read the archive header back** and confirm it names the real live window.

---

## §6 — Definition of Done

- [ ] `J-317`–`J-321`: one copy each deleted; surviving bodies byte-identical to `5a0086b`
- [ ] `J-044` → `J-044a` (L17057) / `J-044b` (L16957); `J-045` → `J-045a` (L17161) / `J-045b` (L17128)
- [ ] **all twelve §2 citations re-pointed individually**, or ten plus §4's ruling recorded
- [ ] `JOURNAL_ARCHIVE.md` heading census **353 / 353, equal**
- [ ] archive header names the live window as measured, not as remembered
- [ ] `Repaired:` line added per §8b's clause, naming what changed
- [ ] **Leg B P2 re-measured and cleared** — this leg exists to clear it, and P2 is the actual test
- [ ] `docs/ROADMAP_ARCHIVE_2026-07-26.md` retention or deletion is **Leg G's**, not this leg's
- [ ] **the `28` figure corrected wherever it appears** — `ROADMAP.md`, `M_DOC_ROADTREE.md` §11 and §8b's table all still carry it

📌 **Owes on close:** nothing. This leg was spawned by §8a and discharges it.

---

## §7 — Out of scope

- **Establishing WHY the duplicates exist.** `D-134` §6 records that no cause is established for either defect and that the mechanism exists because collisions occur. **Not this leg's to find.**
- **The other 71 discussion hits.** They are prose about the collision, not citations, and they stay.
- **`CLAUDE_HISTORY.md`'s seven bare hits** — all discussion (§4c/§8b summaries carried into archived PLAY blocks). ⚠️ **Measured, not assumed: each was read.**
- **Leg E, F, G.**
