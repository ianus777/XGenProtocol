# RUNBOOK — M-DOC-ROADTREE Leg C: keep the tree, link its nodes, delete the chronicle
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-26  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — The lock this runbook implements

🔒 **Joe, 2026-07-26, answering a direct question:** *Leg C = keep the tree that exists · apply §3's field rules to its nodes · delete the prose.*

📌 **Provenance: a LOCK, not a delegation.** Chat framed a bounded proposal and Joe answered it directly. Recorded as such because `M_DOC_ROADTREE.md` §6/§7 were **delegated**, and the two must not be read as the same act.

🔑 **AND THE PRECONDITION THAT BLOCKED THIS FOR A SESSION WAS GUARDING A LOSS THAT CANNOT HAPPEN.** §4 required proving 775 entries redundant before deleting any. But `docs/ROADMAP_ARCHIVE_2026-07-26.md` is committed, and **git holds every byte of every version of this file**. Nothing removed here is recoverable-in-principle — it is recoverable in two commands. ⇒ **the real bar is not *prove the prose redundant*, it is *the surviving tree's links work*.** That is 81 lookups, and it is what this runbook does.

---

## §1 — Grounding (measured 2026-07-26 at `a1d3630`; HEAD = origin, tree clean)

**`docs/ROADMAP.md` = 758,913 bytes / 1,029 lines.** Section map, measured, not described:

| Section | Lines | Bytes | Share | Leg C verb |
|---|---|---|---|---|
| header · legend · update discipline | 1–59 | 4,503 | 0.6% | **KEEP** |
| **Visual structure — nested view (THE TREE)** | 60–283 | **79,802** | **10.5%** | 🔑 **KEEP + REPAIR** |
| How to use this view | 284–331 | 12,871 | 1.7% | **KEEP**, re-point at the repaired tree |
| **Past — settled** (62 `###` subsections) | 332–663 | **283,465** | 37.4% | ⚠️ **DELETE** |
| **Present — playing now** | 664–944 | **362,121** | **47.7%** | ⚠️ **DELETE** |
| Near future | 945–956 | 1,455 | 0.2% | **FOLD INTO TREE** |
| Far future | 957–986 | 1,489 | 0.2% | **FOLD INTO TREE** |
| Cross-cutting | 987–1014 | 12,380 | 1.6% | **KEEP** |
| How to read this document | 1015–1029 | 827 | 0.1% | **KEEP** |

🔑 **THE SINGLE MOST TELLING NUMBER IN THE ARC: *"Present — playing now"* IS 362 KB AND LARGER THAN *"Past — settled"*.** 281 lines averaging **1,289 bytes each**. **A section named *playing now* that is half the document is the diagnosis stating itself** — and it is the same shape as `CLAUDE.md`'s PLAY head, which is why §6 ruled both.

**Delete target: 645,586 B (85.1%). Survivor: ~113 KB.**

⚠️ **THAT MISSES `M_DOC_ROADTREE.md` §9's *"under 100 KB"* DoD BY ~13 KB, AND THE NUMBER IS NOT FUDGED HERE.** The tree alone is 79.8 KB. Either the DoD moves to **under 120 KB**, or *"How to use this view"* (12.9 KB) and *"Cross-cutting"* (12.4 KB) are trimmed as their own decision. 🔓 **Chat proposes moving the DoD number:** those two sections are instructions and standing context, not chronicle, and **trimming them to hit a round number would be optimising the metric rather than the document.**

**Tree contents, measured:** 215 node lines inside the fence at L66–282 · **241 ✅ · 8 🟢 · 3 🟡 · 5 ⏸️ · 5 ⬛ · 0 ❌ · 1 ⚫** · **102 status nodes carry a `J-` reference, 81 do not.**

---

## §2 — What this leg does NOT do

- ❌ **It does not touch `JOURNAL.md` or `JOURNAL_ARCHIVE.md`.** The §4b migration, the `J-317`–`J-321` dedup and the `J-044`/`J-045` split are **Leg B-bis**, still 🔓 unruled, and **nothing here waits on them.**
- ❌ **It does not touch `CLAUDE.md`'s PLAY head.** That is Leg D, and §6's rider stands: its format is decided by its own grounding pass, not inherited from §3.
- ❌ **It does not invent a format.** §3's node format is applied to nodes that already exist.
- ❌ **It reconstructs nothing.** A node whose canonical record cannot be found is **marked, not fabricated.**
- ❌ **No code. Zero `.rs`, zero `ui/**`, zero `skin.css`.** The floors hold by scope, not by measurement.

---

## §3 — The three passes, in order. Each is countable.

### PASS 1 — THE LINK AUDIT. **81 nodes. This is the whole substantive job.**

For each of the 81 status nodes inside the fence carrying no `J-` reference, resolve **a link to its canonical record** and append it in §3's shape: `· J-nnn` or `· <design-document>`.

**Resolution order, cheapest first — stop at the first hit:**
1. **Inherit from the node's tree ancestor.** A child of a linked parent inherits it; **append the parent's link explicitly** rather than relying on visual nesting. ⚠️ **Reason it is written out: the prose that made the nesting legible is being deleted in Pass 3.**
2. **The Past/Present prose entry for the same milestone**, before it is deleted. 🔑 **This is why Pass 1 runs BEFORE Pass 3, and the ordering is load-bearing.**
3. **`JOURNAL.md` / `JOURNAL_ARCHIVE.md`** by title match.
4. **A design document** — `docs/xgen_*_design.md`, `docs/xgen_*_phase0.md`, `tasks/*`. §4c established this is a legitimate canonical record, **not a fallback**.
5. **None found** ⇒ mark `· record: not located` and **list it in the close.** ⚠️ **Never invent a J-number.** §4c rejected exactly that once already.

**Output: 81 rows, one verdict each.** Reported as `n/81`.

### PASS 2 — TRIGGERS AND ARROWS. **8 + 5 nodes.**

- **8 nodes** (3 🟡 + 5 ⏸️) gain `↳ trigger: <condition>` or the explicit `↳ trigger: none — filed, not scheduled`. 🔒 *`trigger: none` is a legal answer and stays legal.*
- **5 ⬛ DEPRECATED nodes** gain a reason and a successor where one exists.
- **Arrows on closed nodes** (`→ successor`) are added **only where a loose end is known AND the cited record actually documents it.** ⚠️ **§3's arrow precondition failed on its first sample of one** (M-RP6.2 authored the deferral; M-RP6.6 was only the gate). **An arrow whose cited entry says nothing makes the loss quieter, not smaller** ⇒ an unverifiable arrow is **omitted, not guessed.**
- ⚠️ **The 8 🟢 PLAY nodes are checked against reality.** Whole-file PLAY count is 69; inside the tree it is 8. **If any of the 8 is not actually in play, it is corrected here** — that is this milestone's founding defect, and this is the pass that catches it.

### PASS 3 — THE DELETION. **Two sections, 645,586 bytes.**

Delete `## Past — settled` (L332–663) and `## Present — playing now` (L664–944) **whole**. Fold `Near future` and `Far future` into the tree as 🟡 nodes with triggers, then delete their sections too.

⚠️ **Any milestone appearing in the deleted prose but NOT as a tree node must become one first.** That check runs **before** the delete and its count is reported.

---

## §4 — Verification

| # | Check | Method | Pass condition |
|---|---|---|---|
| V1 | every status node carries a link or an explicit `not located` | regex over the fence | **0 unaccounted** |
| V2 | every `J-nnn` cited in the new tree resolves | grep each against both journal files | **0 dangling** |
| V3 | every 🟡 / ⏸️ carries a trigger line | regex | **8/8** |
| V4 | no milestone lost | names in deleted prose ∩ tree node names | **0 orphans** |
| V5 | file size | `Get-Item .Length` | **≤ 120 KB** (§1's open DoD question) |
| V6 | scope | `git diff --stat` | **`docs/ROADMAP.md` only**, plus the D-074 records |
| V7 | recoverability | `git show a1d3630:docs/ROADMAP.md` | **758,913 bytes retrievable** |

⚠️ **V7 IS NOT CEREMONY.** It is the check that makes §0's argument **true rather than asserted**, and it runs **before Pass 3, not after.**

---

## §5 — Traps specific to this leg

- ⚠️ **PS 5.1 emoji matching:** `Select-String -SimpleMatch` on a surrogate-pair emoji returns **zero**. 🟢 = `\uD83D\uDFE2` · 🟡 = `\uD83D\uDFE1` · ⏸️ = `\u23F8` · ⬛ = `\u2B1B` · ❌ = `\u274C`. **✅ is `\u2705` and is BMP** — it matches either way, **which is exactly how a partial census can look complete.** Use `[regex]::Matches` for all six.
- ⚠️ **Every line number in this runbook is valid at `a1d3630` only.** Each commit shifts them. **Re-derive section boundaries by heading text, never by stored line number.**
- ⚠️ **`.claude/worktrees/` holds eight copies of the tree.** Exclude `.claude` from every repo-wide search.
- ⚠️ **`Get-Content` without `-Encoding UTF8` mojibakes this file** and reads exactly like corruption. Absolute paths, always `-Encoding UTF8`.
- ⚠️ **`core.autocrlf=true` ⇒ CRLF and LF commit to the same blob (N-167).** Never convert line endings to satisfy a convention; `git diff --stat` decides.

---

## §6 — DoD

- [ ] **Pass 1: 81/81** nodes resolved — link found, or `record: not located` recorded **and listed in the close**
- [ ] **Pass 2: 8/8** triggers · **5/5** deprecation reasons · arrows added **only** where the cited record documents the loose end
- [ ] **Pass 2: 8/8** 🟢 PLAY nodes confirmed actually in play
- [ ] **Pass 3:** `Past — settled` and `Present — playing now` deleted whole; `Near future` / `Far future` folded then deleted
- [ ] **V1–V7 all pass**, each with its number reported
- [ ] **D-074 set in one commit:** `docs/ROADMAP.md` · `JOURNAL.md` · `CLAUDE.md` PLAY · `tasks/M_DOC_ROADTREE.md`
- [ ] `docs/ROADMAP_ARCHIVE_2026-07-26.md` **deleted** — its stated condition was pre-migration recoverability, and **V7 discharges it**

📌 **This checklist does NOT include "commit pushed".** `Status: COMPLETED` in the header is the shipped signal.

---

## §7 — Reporting discipline

🔒 **Chat reports a fraction, not a narrative: `n/81`, then `n/8`, then the byte delta.** Findings that are not blockers are collected and reported **at the close**, not mid-pass.

⚠️ **THE REASON THIS SECTION EXISTS:** the audit surfaced **six defect instances in one session and did not converge.** §0's bound is what stops that — **and it only works if it is honoured during the pass, not merely written here.**

⚠️ **A pass that cannot complete stops and says so with its count. It does not widen.**
