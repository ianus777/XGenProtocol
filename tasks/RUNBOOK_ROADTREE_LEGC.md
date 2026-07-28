# RUNBOOK — M-DOC-ROADTREE Leg C: keep the tree, link its nodes, delete the chronicle
> **Status**: COMPLETED  
> Version: 1.4  
> Date: Jul 2026  
> **Last updated**: 2026-07-28  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — The lock this runbook implements

🔒 **Joe, 2026-07-26, answering a direct question:** *Leg C = keep the tree that exists · apply §3's field rules to its nodes · delete the prose.*

📌 **Provenance: a LOCK, not a delegation.** Chat framed a bounded proposal and Joe answered it directly. Recorded as such because `M_DOC_ROADTREE.md` §6/§7 were **delegated**, and the two must not be read as the same act.

⚠️ **AND THE TAG WAS RE-CUT ONCE (2026-07-26) BECAUSE ITS ANNOTATION CARRIED FIGURES FROM `a1d3630`.** `git tag -f` + `git push --force origin roadtree-pre-legc`. 📌 **Provenance: DELEGATED** (*"we go by your recomm"*), not a considered lock.

🛑 **THIS IS NOT A PRECEDENT FOR §8b, AND MUST NOT BE CITED AS ONE.** §8b asks *may we edit a published immutable record because it is wrong* — and the answer proposed there is **R3: repair as a visible, dated event**, precisely because silent in-place fixes erode a guarantee. **Force-rewriting a pushed tag has the same SHAPE and a different SUBSTANCE:**
- `JOURNAL_ARCHIVE.md` is immutable **because the project promises it** — an IP-provenance claim with a reader who relies on it. **Editing it costs something real.**
- A git tag is immutable **by distribution convention only**, and the sole reason is other people's clones. **There are none — Joe is the only pusher.** **Nothing is promised about a tag's annotation text.**

⇒ **The tag re-cut is a typo fix on a label. §8b remains open and unruled.**

🔑 **AND THE PRECONDITION THAT BLOCKED THIS FOR A SESSION WAS GUARDING A LOSS THAT CANNOT HAPPEN.** §4 required proving 775 entries redundant before deleting any. But `docs/ROADMAP_ARCHIVE_2026-07-26.md` is committed, **git holds every byte of every version of this file**, and 🔒 **the tag `roadtree-pre-legc` marks the exact pre-edit state (Joe, 2026-07-26 — *"for any occasion, especially if we would have to make fallback"*).** Nothing removed here is recoverable-in-principle — it is recoverable in **one command**: `git reset --hard roadtree-pre-legc`. ⇒ **the real bar is not *prove the prose redundant*, it is *the surviving tree's links work*.** That is **86 lookups**, and it is what this runbook does.

---

## §1 — Grounding (re-measured 2026-07-26 **at the tag `roadtree-pre-legc` = `1c3f3d1`**; working copy delta **0**)

⚠️ **v1.0/v1.1 GROUNDED AT `a1d3630` WHILE V7 ANCHORED AT THE TAG — TWO ANCHORS IN ONE DOCUMENT, WHICH IS THE DEFECT V7's OWN FIX WAS ABOUT.** `1c3f3d1` added the J-602 ROADMAP entry (**+2,509 B**), so every v1.1 figure was 2,509 B stale against the state Leg C actually starts from. **All numbers below are now on ONE anchor**, verified `git cat-file -s` blob = working copy = **761,422**, delta **0**.

**`docs/ROADMAP.md` = 761,422 bytes / 1,031 lines.** Section map:

| Section | Lines | Bytes | Leg C verb |
|---|---|---|---|
| header · legend · update discipline | 1–59 | 4,100 | **KEEP** |
| **Visual structure — nested view (THE TREE + how-to-use)** | 60–331 | **92,673** | 🔑 **KEEP + REPAIR** — ⚠️ **the how-to-use half is REWRITE, see §8.2** |
| **Past — settled** (62 `###` subsections) | 332–663 | **283,465** | ⚠️ **DELETE** |
| **Present — playing now** | 664–946 | **364,630** | ⚠️ **DELETE** |
| Near future | 947–958 | 1,455 | **FOLD INTO TREE** |
| Far future | 959–988 | 1,489 | **FOLD INTO TREE** |
| Cross-cutting | 989–1016 | 12,380 | **KEEP** |
| How to read this document | 1017–1031 | 827 | **KEEP** |

🔑 **THE SINGLE MOST TELLING NUMBER IN THE ARC: *"Present — playing now"* IS 364,630 B AND LARGER THAN *"Past — settled"*.** 283 lines averaging **1,289 bytes each**. **A section named *playing now* that is half the document is the diagnosis stating itself** — and it is the same shape as `CLAUDE.md`'s PLAY head, which is why §6 ruled both.

**Delete target: 648,095 B (85.1%). Survivor: 113,327 B ≈ 113 KB.**

⚠️ **THAT MISSES `M_DOC_ROADTREE.md` §9's *"under 100 KB"* DoD BY ~13 KB, AND THE NUMBER IS NOT FUDGED HERE.** The tree fence alone is **79,802 B**. Either the DoD moves to **under 120 KB**, or *"How to use this view"* (12.9 KB) and *"Cross-cutting"* (12.4 KB) are trimmed as their own decision. 🔓 **Chat proposes moving the DoD number:** those two sections are instructions and standing context, not chronicle, and **trimming them to hit a round number would be optimising the metric rather than the document.**

**Tree contents, measured:** 215 node lines inside the fence at L66–282 · **241 ✅ · 8 🟢 · 3 🟡 · 5 ⏸️ · 5 ⬛ · 0 ❌ · 1 ⚫** · **191 status nodes: 105 carry a `J-` reference, 86 do not.**

⚠️ **THE HEADLINE NUMBER WAS 81 IN v1.0 AND IT WAS WRONG — IT IS 86.** The first census counted only ✅ 🟢 🟡 and **omitted ⏸️ ⬛ ⚫**. 🔑 **§5's emoji trap firing on this runbook's own bounding number**, caught at the start of Pass 1 by running the complete pattern rather than the one already written down.

---

## §2 — What this leg does NOT do

- ❌ **It does not touch `JOURNAL.md` or `JOURNAL_ARCHIVE.md`.** The §4b migration, the `J-317`–`J-321` dedup and the `J-044`/`J-045` split are **Leg B-bis**, still 🔓 unruled, and **nothing here waits on them.**
- ❌ **It does not touch `CLAUDE.md`'s PLAY head.** That is Leg D, and §6's rider stands: its format is decided by its own grounding pass, not inherited from §3.
- ❌ **It does not invent a format.** §3's node format is applied to nodes that already exist.
- ❌ **It reconstructs nothing.** A node whose canonical record cannot be found is **marked, not fabricated.**
- ❌ **No code. Zero `.rs`, zero `ui/**`, zero `skin.css`.** The floors hold by scope, not by measurement.

---

## §3 — The three passes, in order. Each is countable.

### PASS 1 — THE LINK AUDIT. **86 nodes. This is the whole substantive job.**

For each of the 86 status nodes inside the fence carrying no `J-` reference, resolve **a link to its canonical record** and append it in §3's shape: `· J-nnn`, `· DECISIONS.md D-nnn`, or `· <design-document>`.

**Resolution order, cheapest first — stop at the first hit:**
1. **Inherit from the node's DIRECT parent only.** 🔒 **Never climb past an unlinked parent** — §5 records why: the climb produces confident wrong links. **56 of 86 resolve here.**
2. **The Past/Present prose entry for the same milestone**, before it is deleted. 🔑 **This is why Pass 1 runs BEFORE Pass 3, and the ordering is load-bearing.**
3. **`JOURNAL.md` / `JOURNAL_ARCHIVE.md`** by title match.
4. **A design document** — `docs/xgen_*_design.md`, `docs/xgen_*_phase0.md`, `tasks/*`. §4c established this is a legitimate canonical record, **not a fallback**.
5. **None found** ⇒ mark `· record: not located` and **list it in the close.** ⚠️ **Never invent a J-number.** §4c rejected exactly that once already.

**Output: 86 rows, one verdict each.** Reported as `n/86`.

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
| V7 | recoverability | `git cat-file -s $(git rev-parse roadtree-pre-legc:docs/ROADMAP.md)` | **761,422 bytes retrievable** |

⚠️ **V7 IS NOT CEREMONY.** It is the check that makes §0's argument **true rather than asserted**, and it runs **before Pass 3, not after.**

🔑 **V7 WAS WRONG IN v1.0 AND THE FIX IS A CLASS FIX, NOT AN INSTANCE FIX.** It read `git show a1d3630:docs/ROADMAP.md` — **the MEASUREMENT commit, not the PRE-DELETION commit.** `1c3f3d1` landed the J-602 entry afterwards, so ROADMAP was **already not** what that command retrieves: **V7 would have passed while proving the wrong thing.** ⇒ **anchored to the TAG `roadtree-pre-legc`, deliberately placed at the last state before Leg C edits any document.** 📌 **A hash pasted into a document goes stale the moment anything else commits; a tag is placed at the state you mean and stays there.**

⚠️ **AND THE TAG IS A RESTORE POINT, NOT ONLY A VERIFICATION ANCHOR (Joe, 2026-07-26: *"for any occasion, especially if we would have to make fallback"*).** `git reset --hard roadtree-pre-legc` returns the whole tree. 📌 **Recorded in `CLAUDE.md` as a ♻️ Recovery anchor beside `m-rp6.0-gate-go`** — **a tag nobody can find is not a fallback.**

---

## §5 — Traps specific to this leg

- 🛑 **V7 MUST NOT READ THROUGH A POWERSHELL PIPELINE. MEASURED:** `git show roadtree-pre-legc:docs/ROADMAP.md | Out-String` returns **762,453 bytes across 655 lines**; the file is **761,422 bytes across 1,031 lines**. PS 5.1 re-encodes and re-wraps git's stdout, so **both numbers are wrong and neither looks obviously wrong.** 🔒 **Use `git cat-file -s` on the blob** — it reports the stored object size and never touches a pipeline. ⚠️ **A verification that goes through a lossy transport is not a verification.**
- ⚠️ **PS 5.1 emoji matching:** `Select-String -SimpleMatch` on a surrogate-pair emoji returns **zero**. 🟢 = `\uD83D\uDFE2` · 🟡 = `\uD83D\uDFE1` · ⏸️ = `\u23F8` · ⬛ = `\u2B1B` · ❌ = `\u274C`. **✅ is `\u2705` and is BMP** — it matches either way, **which is exactly how a partial census can look complete.** Use `[regex]::Matches` for all six.
- ⚠️ **Every line number in this runbook is valid AT THE TAG `roadtree-pre-legc` ONLY.** Each commit shifts them. **Re-derive section boundaries by heading text, never by stored line number.** 🔑 **Same disease as V7's original hash** — a stored coordinate goes stale silently while still looking authoritative.
- 🛑 **AN ANCESTOR WALK THAT KEEPS CLIMBING PAST AN UNLINKED PARENT PRODUCES CONFIDENT WRONG LINKS (Pass 1, measured).** **30 of the 86 nodes have a direct parent carrying no `J-` ref.** A walk that climbs further lands on an unrelated ancestor — `L197 M2 Node Pipe Server` inherited from *XGID Retrofit Pass series*. 🔒 **RULE: inherit from the DIRECT parent only. Everything else is resolved by hand.** 📌 **Caught by spot-checking six rows, not by trusting eighty-four.**
- 🛑 **TITLE-REGEX LOOKUP AGAINST JOURNAL HEADINGS PRODUCES FALSE POSITIVES.** `Phase 7 B3` matched `J-107 — Persistence-amendment…`; `Phase 7.5 design` matched `J-093 — Phase 9 Commit 3…`. **Both rejected.** 🔒 **A wrong link is worse than a missing one** — `record: not located` is a legal outcome and an invented link is not.
- ⚠️ **A PARTIAL SYMBOL CENSUS LOOKS EXACTLY LIKE A COMPLETE ONE.** §1's *"81 nodes"* omitted ⏸️ ⬛ ⚫; **the real count is 86.** Corrected throughout. **This is the §5 emoji trap firing on the runbook's own headline number.**
- ⚠️ **`.claude/worktrees/` holds eight copies of the tree.** Exclude `.claude` from every repo-wide search.
- ⚠️ **`Get-Content` without `-Encoding UTF8` mojibakes this file** and reads exactly like corruption. Absolute paths, always `-Encoding UTF8`.
- ⚠️ **`core.autocrlf=true` ⇒ CRLF and LF commit to the same blob (N-167).** Never convert line endings to satisfy a convention; `git diff --stat` decides.

---

## §6 — DoD

- [ ] **Pass 1: 86/86** nodes resolved — link found, or `record: not located` recorded **and listed in the close**
- [ ] **Pass 2: 8/8** triggers · **5/5** deprecation reasons · arrows added **only** where the cited record documents the loose end
- [ ] **Pass 2: 8/8** 🟢 PLAY nodes confirmed actually in play
- [ ] **Pass 3:** `Past — settled` and `Present — playing now` deleted whole; `Near future` / `Far future` folded then deleted
- [ ] **V1–V7 all pass**, each with its number reported
- [ ] **D-074 set in one commit:** `docs/ROADMAP.md` · `JOURNAL.md` · `CLAUDE.md` PLAY · `tasks/M_DOC_ROADTREE.md`
- [ ] `docs/ROADMAP_ARCHIVE_2026-07-26.md` **deleted** — its stated condition was pre-migration recoverability, and **V7 discharges it**

📌 **This checklist does NOT include "commit pushed".** `Status: COMPLETED` in the header is the shipped signal.

---

## §7 — Reporting discipline

🔒 **Chat reports a fraction, not a narrative: `n/86`, then `n/8`, then the byte delta.** Findings that are not blockers are collected and reported **at the close**, not mid-pass.

⚠️ **THE REASON THIS SECTION EXISTS:** the audit surfaced **six defect instances in one session and did not converge.** §0's bound is what stops that — **and it only works if it is honoured during the pass, not merely written here.**

⚠️ **A pass that cannot complete stops and says so with its count. It does not widen.**

---

# §8 — PASS RESULTS, AND THE SCOPE THE PASSES FOUND (2026-07-26, at the tag)

📌 **§§8.1–8.7 were locked in conversation across one sitting and are written here because a decision in chat is not a record.**

## §8.1 — PASS 1 ✅ COMPLETE: 83/83 · 0 not located · 3 reclassified

| Resolution route | n |
|---|---|
| direct parent carries the link | 56 |
| D-entry nodes → `DECISIONS.md` | 9 |
| XGID Adoption v1 subtree, once its container resolved | 6 |
| M-series → `J-074` · `J-075` · `J-077` · `J-078` | 4 |
| container nodes → `J-095` and `J-102 + J-103` | 2 |
| 🟢 needs no link per §3 | 1 |
| **the hard five** — M1 → `J-073` · ⬛ M6-original → `DECISIONS.md D-069` · Phase 7 B3 → `tasks/archive/FEDERATION_PROPAGATION_PHASE_7_B3_AMENDMENT.md` · Phase 7.5 design → `J-093` · Phase 7.5 impl → `J-103` | 5 |
| **reclassified to Pass 2** (⏸️ needs a trigger, not a link) | 3 |

⚠️ **`J-093` was a REJECTED false positive that turned out correct.** The title regex matched on incidental words; it is right only because `J-103`'s **body** states *“Design phase closed at J-093”*. **A false positive that happens to be true is still a broken method.**

## §8.2 — PASS 2 ✅ COMPLETE, and §1's census counted the wrong thing

⚠️ **§1 counted symbol OCCURRENCES; a state board is made of NODES.** Corrected: **✅ 172** (not 241) · **🟢 4** (not 8) · 🟡 3 · **⏸️ 3** (not 5) · **⬛ 4** (not 5) · ⚫ 1. **The gap is prose: a ✅ node's text names other ✅ items inline.**

🛑 **THE TRIGGER AUDIT FOUND A STALE NODE — THE DEFECT THIS MILESTONE EXISTS FOR.** `🟡 Component sampler — dev exhibit app (deferred, post-textfield)`: **`ui/sampler/` holds 1,896 files, crate `xgen-sampler` is on disk, `J-422` closed it (M-RP3.0)**, and its trigger fired earlier still at M-RP2.12/J-417. ⇒ **🟡 → ✅ · J-422.** 🔑 **A trigger fired, the work shipped, the roadmap never noticed — the M-RP6.2/M-RP6.6 shape verbatim.**

**Triggers:** ✅ already compliant — `Multi-device arc` · `Slovak translation` · `DPI resistance`. **`Clean-table UI milestone`** gains `↳ trigger: Round-2 audit GO + M10 closed`, 📌 **transcribed from parent L247 and marked as transcribed**. **`Registry file encryption`** → `↳ trigger: none — filed, not scheduled`: *“rides the D-080/085 framework”* **names a home, not a condition**, and D-085 shipped at J-232. ⚠️ **`DPI resistance`'s *“resume when Phase 3 opens”* may point at a DEAD phase** — `⬛ Phase 3 (J-153) — COLLAPSED`; two different Phase 3s. **Flagged, not fabricated.** **⬛ 4/4 already carry reason + successor.**

## §8.3 — 🛑 V4 FAILED BEFORE THE DELETION: TEN MILESTONES EXIST ONLY IN A RIVAL TREE

🔑 **`How to use this view` (L284–331) contains a SECOND TREE in a THIRD notation** — ASCII pipes, status in **square brackets** (`[CLOSED J-269]`). **§1 marked that section KEEP. It holds content the main tree never received:**

| Absent from the main tree | Record |  | Absent | Record |
|---|---|---|---|---|
| `M8.5` finalization | **J-279** | | `Arc F` space migration | **J-252** |
| `Arc A` doc-drift | **J-233** | | `Arc G` jurisdictional ns | **J-250** |
| `Arc B` forward-compat | **J-235** | | `Arc H` E2E encryption | **J-257** |
| `Arc D` privilege model | **J-244** | | `Arc I` GDPR erasure | **J-253** |
| `Arc E` primitive completion | **J-248** | | `Round 2` pre-UI gate | **J-390** |

⚠️ **AND THE RIVAL TREE IS WRONG ABOUT M8.7.** It reads *“D3 MLS operationalisation (openmls) [CLOSED J-302]”*; the main tree and J-302 both say **“concurrent-commit resolution (R only)”**, *“no key material”*. 🔒 **`openmls` is ABSENT from `Cargo.lock` — measured. D3 is NOT discharged.** 🔑 **A reader of the decode section would conclude MLS shipped in June.**

⇒ **`How to use this view` is REWRITTEN as the decode key** — node grammar, field rules, exemptions. **No second tree, no frozen snapshot** (it answers *“what's playing right now?”* with **M-RP2.3 / J-403** and admits its own bullets froze at ~J-256).

## §8.4 — FIVE FORMAT RULES LOCKED DURING THE PASS

**R-1 — LEADING SYMBOL ON EVERY NODE.** 🔒 Joe. 4 parenthetical containers normalise; 15 unstatused containers gain one.

**R-2 — CONTAINER STATUS IS DERIVED AND PROPAGATES.** 🔒 Joe. *all children ✅ ⇒ ✅ · any child 🟢 ⇒ 🟢 · otherwise the weakest live state*, root exempt. 🔑 **A milestone with unfinished children is not done.**

**R-3 — A CONTAINER OF NON-WORK CARRIES NO STATUS AT ALL.** 🔒 Joe. It carries a link instead. 🔑 **A standing rule has a FORCE, not a STATE** — `🟢 D-065`, `🟢 D-069`, `🟢 Honest longer work` were all false, **and R-2 would have manufactured a 🟢 container out of them.** ⇒ all 12 `Cross-cutting principles` children collapse to one `· DECISIONS.md` link *(delegated)*. 📌 **The branch cited 11 of 129 decisions and stopped at D-078 — an ABANDONED mirror, not a mirror.**

**R-4 — IF A NODE NEEDS A QUALIFIER TO BE TRUE, IT NEEDS A CHILD INSTEAD.** 🔒 Joe (*“those DONE on H and I looked like some conditional DONE”*). ⚠️ **A ✅ with a subtraction clause in its prose is a claim the symbol contradicts.**

**R-5 — §3 GAINS A NODE LENGTH BOUND** (see §8.5a). Without one the chronicle simply relocates into the tree.

## §8.5 — THE SIX CONDITIONAL-DONES (17 candidates read individually → 6 verdicts)

| Line | The subtraction |
|---|---|
| **L210** | `Phase 7 — A1 Federation mgmt HONEST-SUBSET: list + defederate (2 of 7); 5 verbs → D-071 arc` |
| **L212** | `Phase 9 read subset — audit-events DEFERRED (log unbuilt); force-eject A4-D1 gated` |
| **L204** | `Phase 1 (R1 rooms) — members deferred (no local data source)` |
| **L228** | `M7-completion cluster — FIVE explicitly OUT, each with a named home` |
| **L235** | `M8.7 (R only) — S + home-DS serialization + loser-rebuild folded into the production openmls-client arc` ⚠️ **openmls absent** |
| **L239** | `Multiparty tests — all-green-except-{MP-C-06, MP-C-16}, both ⏸️ → M10` |

📌 **Resolved by a later link in their own chain:** L226 → L227 → L228; L136 → Pass 2. **Descriptive, not subtractive:** L144 · L147 · L148 · L149 · L152 · L165 · L234. 🔑 **The regex gave 17 CANDIDATES; reading gave 6 VERDICTS.**

### 🛑 §8.5a — L239 IS NOT A NODE, IT IS A CHRONICLE IN ONE LINE

One tree row carries the **entire multiparty arc** — MP-R1→MP-R3, MP-F1…MP-F14, J-324 through J-357, commit hashes, falsified hypotheses. ⚠️ **This is the 124,299-char-line disease INSIDE the tree we agreed to keep.** §1's premise was *tree good, prose bad*; **the prose migrated into a tree node, because a tree node has no length limit.**

🔒 **RESOLUTION (delegated): COLLAPSE, do not split** → `✅ **Multiparty tests** — R1+R2+R3 · J-356 → M10 (MP-C-06, MP-C-16)`. **`tasks/HANDOFF_MP_R3.md` §3 already holds the 37-scenario ledger** — reproducing it in the tree is D-067.

## §8.6 — ARC H AND ARC I, GRAFTED UNDER R-4

```
🟡 Arc H — E2E encryption                 🟡 Arc I — GDPR erasure
├── ✅ design + interface lock · J-257    ├── ✅ design + D-088 · J-253
└── 🟡 PG-05 implementation              ├── 🟡 content erasure
    ↳ trigger: D3 — RFC 9420/openmls    │   ↳ trigger: PG-05 ships
      (measured ABSENT from Cargo.lock)  └── 🟡 identity orphaning
                                             ↳ trigger: none — PG-05-independent;
                                               rides the Tier-1 auth-module rebuild
```

🔑 **J-253's sequencing finding, verbatim:** *“PG-05 (Arc H) precedes the content-erasure implementation; the identity-orphan half is PG-05-independent and could ride the Tier-1 auth-module rebuild.”*

🔑 **FIVE NODES CARRY ONE UNFINISHED THING:** `Arc H` · `Arc I` · `M8.5` residue · `Round 2` (GO with *gap register Open 1/13*) · the existing `🟡 Multi-device (R2-F09)`. **All of it is D3 / PG-05.** ⇒ **under R-2 the container reads 🟡 once and says why; today the tree says ✅ six times and the gap is invisible.**

⚠️ **`ARC_H_E2E_IMPL.md:94` is this project's own precedent:** *“PG-05 → interface-locked / impl-deferred — **not** ✅ DONE (D-065; PG-02 shape)”*. **Arc H already refused the false ✅. A bare ✅ here would break a rule its own sibling honoured.**

## §8.7 — REVISED EXECUTION ORDER

1. Graft the ten (§8.3); Arc H/I per §8.6
2. Apply Pass 1's 83 links + Pass 2's status corrections
3. Split the six conditional-DONEs (§8.5); collapse L239 (§8.5a)
4. Apply R-1, R-2, R-3
5. ⬛→✅ for `Storage-Engine` (L232); delete the ⚫ legend row. ⚠️ **`CLAUDE_HISTORY.md`'s 65 ⚫ are a DIFFERENT notation — Leg D, untouched**
6. Rewrite `How to use this view` as the decode key
7. **V1–V7**
8. **Then** Pass 3, the deletion

⚠️ **STEPS 1–6 ARE ALL SCOPE FOUND DURING THE PASS, NOT IN v1.0's PLAN.** Recorded as growth rather than absorbed silently — that habit is what produced `M_DOC_ROADTREE.md` §8a.
