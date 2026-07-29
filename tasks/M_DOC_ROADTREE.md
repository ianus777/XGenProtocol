# M-DOC-ROADTREE — the roadmap becomes a state board
> **Status**: ACTIVE  
> Version: 1.14  
> Date: Jul 2026  
> **Last updated**: 2026-07-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

**This is the Phase-0 for converting `docs/ROADMAP.md` from a chronicle into a state board.** Joe, 2026-07-26: *"we have the journal, do we really need the second part of the file with quasi journal records? … now i propose that we keep just roadtree. other chronological records are in journal."*

🔑 **THE TRIGGER WAS A MEASURED FAILURE, NOT A TIDINESS URGE.** This session found that `M-RP6.2`'s deferral of live spaces/rooms push lived **only in a code comment** (`app_client.svelte:564-565`), named `M-RP6.6` as its gate, and was never picked up when that gate closed at J-543. **The roadmap did not carry it.** A document that misses a deferral has earned an audit.

**IT IS NOT** a rewrite of project history. Nothing is deleted that the journal does not already hold — §4 is the precondition that makes that claim true rather than assumed.

**IT IS NOT** an edit of `JOURNAL.md`. The journal is the chronicle and stays untouched, except where §4 finds it genuinely missing something.

⚠️ **DECLARED CONFLICT OF INTEREST.** The authoring defect behind the roadmap's drift is substantially Chat's (`M_RP_MEMBERS.md` §8b — *scope written in files, requirements written in behaviours, never reconciled*). **This is partly an audit of Chat's own record-keeping**, which is why §3 and §9 are written as **checkable criteria** rather than as Chat's judgement entry by entry.

---

## §1 — Grounding (measured 2026-07-26 at `0466cb2`, HEAD = origin)

| Document | Size | Contents |
|---|---|---|
| `docs/ROADMAP.md` | **749,717 bytes / 1,017 lines**, longest line **38,559 chars** | 775 status markers across 75 headings |
| `JOURNAL.md` | 1,376,695 bytes | **222 entries**, J-376 … J-597 |
| `JOURNAL_ARCHIVE.md` | 2,333,803 bytes | **358 entries** |
| `CLAUDE.md` | 608,702 bytes | one line is **124,299 chars** |
| `CLAUDE_HISTORY.md` | 869,178 bytes | prior PLAY blocks (D-094) |

⚠️ **§1a — RE-MEASURED 2026-07-26 23:07 AT `ec0d305`. THE DENOMINATOR HAS MOVED, AND THE ARCHIVE CLAIM WAS NEVER TRUE AS WRITTEN.**

| Quantity | §1 as written (at `0466cb2`) | Measured at `ec0d305` |
|---|---|---|
| `docs/ROADMAP.md` | 749,717 B / 1,017 lines | **755,033 B / 1,025 lines** — +5,316 B / +8 lines across `78d44f9` + `ec0d305`, **both doc-only commits** |
| `docs/ROADMAP_ARCHIVE_2026-07-26.md` | *"byte-identical at 749,717"* (§8 Leg A) | **749,428 B** — ⚠️ **NOT byte-identical, and never was.** The archive's header was replaced with `Status: ARCHIVED`, a 289-byte difference. It is **content-identical, header-differing** |
| `CLAUDE.md` longest line | 124,299 chars | **124,299 — confirmed unchanged** |

📌 **Neither delta changes any ruling.** The archive's purpose — a recoverable pre-migration copy — is unaffected; only the word *byte-identical* was wrong, and §8 Leg A is corrected below. 🔑 **But the drift itself is the argument for the milestone:** two doc-only commits carrying no new work grew the state board by 5 KB in one evening. **§3's ~70 KB projection now runs against 755 KB.**

**Status-marker census of `ROADMAP.md`:**

| Symbol | Meaning | Count |
|---|---|---|
| ✅ | DONE | **586** |
| 🟢 | PLAY | **69** |
| 🟡 | PENDING | **69** |
| ⏸️ | POSTPONED | **22** |
| ⬛ | DEPRECATED | **20** |
| ❌ | CANCELLED | **9** |
| | **total** | **775** |

🔑 **THE FINDING THAT REFRAMES THE WHOLE TASK: THERE ARE 69 🟢 PLAY MARKERS.** Nothing can have sixty-nine things simultaneously in play. ⇒ **PLAY is being used as *"this was in play when this was written"* — historical narration, not current state.** And **586 of 775 markers (76%) are ✅ DONE.**

⇒ **`ROADMAP.md` has become a chronicle. That is the defect, and it is upstream of any individual wrong entry.** Auditing 775 entries inside a 750 KB document would be days of work, most of it re-checking history that cannot change.

---

## 🛑 §1b — THE DOCUMENT ALREADY CONTAINS A MAINTAINED STATE TREE, AND §1 NEVER LOOKED. MEASURED 2026-07-26 AT `a1d3630`

⚠️ **INSTANCE SIX OF THIS ARC'S RECURRING DEFECT, AND IT IS THE LARGEST: IT IS ABOUT THE MILESTONE'S OWN PREMISE, AND IT IS CHAT'S.** §1 measured the document's **size** and its **marker census** and never asked what its **sections** were. 🔑 **Found by running the P1 enumeration properly instead of re-reading the summary of it.**

🔑 **`docs/ROADMAP.md` LINES 66–282 ARE A FENCED ASCII TREE OF THE MILESTONE HIERARCHY — 215 NODE LINES, 78,841 BYTES, 10.4% OF THE FILE — UNDER THE HEADING *"Visual structure — nested view"*.** One node per line, structure from nesting, a status symbol on each node. **That is substantially the artefact §3 specifies and Leg C was to build.**

🔑 **AND IT IS NOT ABANDONED. THE DOCUMENT STATES ITS MAINTENANCE RULE IN BOLD, TWO LINES ABOVE IT:** *“Same-commit discipline applies to the tree, no exceptions. When updating ROADMAP.md for any state change, the tree also updates.”* ⚠️ **§4c called it *“an old ASCII tree diagram”*. It is not old. That single word carried an assumption that was never checked**, and it is why §4's decomposition counted **4** tree nodes where there are **75**.

### The corrected P1 decomposition — re-run at `a1d3630`, not inherited

📌 **The headline numbers REPRODUCE EXACTLY: 94 ✅ markers on 90 unlinked lines.** (Whole-file census is now **592 ✅**, up from 586 — six added by this session's three commits, **all carrying J-refs**, which is why the unlinked set is unchanged.) **What was wrong was the classification, not the count.**

| Class | §4 said | **Measured** | Lines |
|---|---|---|---|
| ASCII-tree nodes | *"4 nodes of an old ASCII tree"* | ⚠️ **75** | 70 … 277, all inside the fence |
| legend / table rows | 4 | **2** | 30, 35 |
| bullets | *(inside the 4)* | **2** | 42, 292 |
| everything else | 5 risk + 1 prose | **11** | 372, 376, 396, 412, 414, **428, 444, 450, 454, 997, 1023** |

⚠️ **§4's *"8 — NOT AT RISK"* row lists 4 + 4 + 1 = NINE items and totals them as EIGHT.** Arithmetic slip, recorded rather than quietly fixed.

⚠️ **AND THE RISK SET IS NOT 5.** §4 resolved `372 · 376 · 396 · 412 · 414` and closed P1. **Six further unlinked non-tree lines were never in that list: `428 · 444 · 450 · 454 · 997 · 1023`.** 📌 **Not yet classified — stated as open, not guessed at.** ⚠️ **§4's 🔒 *"P1 IS CLEARED"* is therefore RETRACTED.**

### 🔑 What the tree's own census proves — and it sharpens §1's diagnosis rather than weakening it

| Symbol | Whole document | **Inside the tree** |
|---|---|---|
| ✅ DONE | 592 | 241 |
| 🟢 PLAY | **69** | ✅ **8** |
| 🟡 PENDING | 69 | 3 |
| ⏸️ POSTPONED | 22 | 5 |
| ⬛ DEPRECATED | 20 | 5 |
| ❌ CANCELLED | 9 | 0 |

🔑 **§1's headline finding was *"nothing can have SIXTY-NINE things in play"*. Inside the tree, PLAY is EIGHT.** ⇒ **the tree already behaves as a state board; the 69 comes entirely from the prose sections.** **§1's diagnosis is right about the document and wrong about which part of it is sick.**

📌 **102 tree nodes carry a `J-` reference; 81 do not.** That is Leg C's real link-audit surface — **81 nodes, not 775.**

### 🔓 CONSEQUENCE FOR LEG C, AND IT IS JOE'S BECAUSE IT IS STRUCTURE

**Leg C was scoped as *convert 775 prose entries into a tree that does not yet exist*. Measured, the work is different and smaller:**
1. **The tree EXISTS and is maintained.** It is kept, not invented.
2. **§3's FIELD RULES are what it lacks** — a canonical-record link on every ✅ (81 owed), a trigger on every 🟡/⏸️, an arrow on closed nodes. **That is the actual deliverable.**
3. **The PROSE sections are the chronicle** — ~680 KB — **and they are what leaves.**

🔑 **AND THE ARITHMETIC NOW CLOSES, WHICH IT DID NOT BEFORE.** §3 projected the end state at **~70 KB**. **The existing tree is 78,841 bytes.** ⇒ **the projection and the artefact already agree to within 12%** — §3 was, without knowing it, predicting the size of a thing already in the file. **That is corroboration from an independent direction, and it is the strongest evidence yet that the target shape is right.**

⚠️ **§7's S1-versus-S2 ruling is UNAFFECTED and was decided on the correct grounds** — the tree it chose is the tree that exists. 📌 **§6's *"both"* is likewise unaffected**, though Leg D should now ask the same question first: **does `CLAUDE.md` also already contain a structure nobody measured?**

---

## §2 — 🔒 DECISION 1: THE CHRONICLE IS DELETED, NOT RELOCATED (Joe, 2026-07-26)

🔒 **LOCKED.** ⚠️ **Chat proposed moving the closed entries to a `ROADMAP_HISTORY.md`, on the `CLAUDE_HISTORY.md` precedent (D-094). Joe rejected it and was right:** that would create **a second source of truth for the same chronicle**, alongside `JOURNAL.md` — **D-067 violated in the documentation layer.** The journal already is the chronicle. 📌 **Chat's proposal is recorded as withdrawn, not as an alternative.**

🔑 **AND THE JOURNAL REALLY DOES HOLD IT — MEASURED, NOT ASSUMED:**
- **580 journal entries exist** (222 live + 358 archived).
- `ROADMAP.md` cites **389 unique J-numbers** (1,573 references in total).
- **378 of the 389 resolve.**

---

## §3 — 🔒 DECISION 2: THE ROADTREE NODE FORMAT (Joe, 2026-07-26)

🔒 **Joe's shape, locked verbatim from the walk:**

```
✅ **M-RP6.2** — the known-Space tree · J-xxx → M-RP-LIVEFEED-REFRESH Leg B
```

**One node = one line.** Structure comes from nesting alone. **The prose leaves; the tree stays.**

### The field rules — written so the format enforces itself

| Status | Must carry |
|---|---|
| ✅ **DONE** | **a link to its canonical record** — non-negotiable. The link is what licenses the prose to leave. ⚠️ **Usually `· J-nnn`, but NOT always — see §4's first-pass finding: some closes are canonically recorded in a DESIGN DOCUMENT, not the journal, and the node points there instead** |
| 🟢 **PLAY** | nothing extra — it is active now |
| 🟡 **PENDING** · ⏸️ **POSTPONED** | **a trigger**, or `↳ trigger: none — filed, not scheduled` stated **explicitly** |
| ❌ **CANCELLED** · ⬛ **DEPRECATED** | a reason, and a successor where one exists |

🔒 **`trigger: none` IS A LEGAL ANSWER AND STAYS LEGAL.** The failure mode is not unscheduled work — it is work whose unscheduled-ness was **never stated**. Forcing a fabricated trigger would be worse than none.

🔒 **THE MILESTONE-NAMING RULE APPLIES AT EVERY NODE:** an identifier **never** appears bare; always identifier + short descriptive title.

🔒 **OWNER is omitted where it is the default** (Joe locks · Clair implements · Chat verifies) and named only where it is not.

### 🔑 The arrow, and the second thing it does

**`→ successor`** on a **closed** node is a forwarding address for a loose end.

🔑 **PLACEMENT IS THE POINT, AND IT SOLVES THE M-RP6.6 FAILURE DIRECTLY.** That failure was not only that the deferral was hidden — it is that **when the gate finally opened, nothing on the gate's side said *"things are waiting on me."*** Putting the arrow on the **closed** node means whoever closes a milestone **sees what it unblocks, on the same line.** Discoverability solved by placement, not by process.

⚠️ **THE ARROW'S PRECONDITION — AND IT FAILED ON THE FIRST SAMPLE OF ONE.** An arrow points at a journal entry, so **that entry must actually record the thing being forwarded.**
- Chat first wrote this node against **M-RP6.6**. **J-543 records a different deferral** (*live ingest stayed out; R5 real-time ingest remains its own milestone, gated on R5 + M-RP6.3*) — **nothing about spaces or rooms.**
- The shell comment reads `// M-RP6.2 — the known-Space tree (D1). Static per session (no live push until the resident, M-RP6.6)`. ⇒ **M-RP6.2 AUTHORED the deferral; M-RP6.6 was only the GATE it named.** Chat collapsed gate and author and stated it confidently.
- 🔑 **The format caught a mis-attribution that prose had let pass unchecked.** That is a point in its favour, and it is why §4 is a **precondition, not a later tidy.**
- ⇒ **Where the cited entry does not record the loose end, either the arrow moves to the right node, or the journal is corrected first.** A forward pointer to an entry that says nothing makes the loss *quieter*, not smaller.

📌 **The status symbol disambiguates the arrow's meaning without extra syntax:** ✅ + → = *a loose end continues here*; ⬛ + → = *replaced by*.

**Projected size:** 775 nodes at roughly 90 characters ≈ **70 KB, against today's 750 KB.** 🔑 **Small enough to read in full at every session open — which is the only thing that lets it function as a state board rather than a chronicle.**

---

## §4 — ⚠️ THE MIGRATION PRECONDITION — TWO SETS, BOTH BOUNDED

### 🔑 LEG B FIRST PASS — RUN 2026-07-26. **P1 COLLAPSES FROM 94 TO 5, AND THE LAST ONE IS STILL OPEN.**

**The decomposition, measured:**

| Step | Count |
|---|---|
| ✅ markers on lines carrying no `J-` reference | **94** |
| … distinct **lines** they sit on | **90** |
| … of which **INHERIT** a `J-` reference from an ancestor line in the same block (sub-items under a linked entry) | **77 — NOT AT RISK** |
| … **true orphans** | **13** |
| … of which are **not entries at all** — 4 legend rows defining the symbols (lines 30/35/42/1017) + 4 nodes of an old ASCII tree diagram (82/199/200/263) + 1 prose line | **8 — NOT AT RISK** |
| ⇒ **GENUINE RISK SET** | **5 entries** |

**The five, and their verdicts:**

| Line | Entry | Verdict |
|---|---|---|
| 412 | XGID Adoption v1 design walkthrough + Phase 1 canonical sources commit (2026-05-20) | ✅ **RESOLVED — `J-102`**, itself a *"JOURNAL Gap 2 closure: retrospective entry"* |
| 414 | Topological-sort wire-order non-determinism design phase closed (2026-05-22) | ✅ **RESOLVED — `J-097`** |
| 376 | Federation Event Propagation **Pass 2** closed (2026-05-18) | ⚠️ **NOT IN THE JOURNAL — but canonically recorded in `docs/xgen_federation_propagation_design.md` (40 hits).** Redundant against **that** document |
| 396 | Federation Event Propagation **Pass 3** closed (2026-05-18) | ⚠️ **Same — 57 hits in the design document.** Redundant against **that** document |
| 372 | **M6 Phase 0** design phase closed (2026-05-18) | ✅ **RESOLVED — `docs/xgen_node_admin_ops_design.md`** (6 hits), the canonical M6 design document. **Same pattern as 376/396** |

🔒 **P1 IS CLEARED. All five resolved: two to journal entries, three to design documents.**

🔑 **AND THE PATTERN IS NOT A DEFECT — IT IS THE CORRECT HOME.** All three document-resolved entries are **design-phase closes with no code**. Such a close is canonically recorded in **the design document it closed**, not in the journal. ⇒ **§3's link rule was wrong as first written**, and it was wrong in the direction that would have destroyed information: *invent a J-number, or declare loss.* **Neither would have been true.**

### 🔑 LEG B SECOND PASS — P2, AND IT IS BIGGER THAN 11

**Enumerating every `## Entry J-nnn` heading across both journal files: 574 entries exist between J-001 and J-598. ⇒ 24 NUMBERS ARE MISSING.**

| Set | Numbers | Reading |
|---|---|---|
| **A — one contiguous block of 13** | `J-021 … J-033` | ⚠️ **Smells like a numbering artifact, not thirteen lost entries** — a reset, a renumber, or a batch never allocated. **Not claimed either way.** |
| **B — 10 scattered singles** | `J-067 · J-098 · J-109 · J-113 · J-123 · J-124 · J-125 · J-131 · J-132 · J-135` | 🔑 **These are the ones ROADMAP CITES.** A citation means someone, at the time of writing, **believed the entry existed.** That is a far stronger signal of real loss than set A |
| **C — one straggler** | `J-171` | Missing, **never referenced by ROADMAP** |

📌 **The 11 originally reported are set B plus `J-029`, which falls inside set A.**

🔑 **AND THE PROJECT HAS ALREADY SOLVED THIS EXACT PROBLEM, TWICE — THE PRECEDENT IS ITS OWN.** `J-102` is *"JOURNAL Gap 2 closure: retrospective entry for XGID Adoption v1…"* and `J-103` is *"JOURNAL Gap 1 closure: retrospective entry for … Phase 7.5"*. ⇒ **retrospective gap-closure entries are an established, named practice here.** Set B is closed the same way, or each number is recorded as **never allocated**.

⚠️ **CONSEQUENCE FOR THE MIGRATION, STATED PLAINLY: for set B's ten entries the ROADMAP LINE IS CURRENTLY THE ONLY TRACE.** ⇒ **their prose must NOT be deleted until the journal carries it.** Everything else in the document is free to go once §3's format is applied.

🔒 **PRECONDITION STATUS: P1 CLEARED · P2 MEASURED, NOT CLEARED.** ⚠️ **The archive's delete condition is therefore NOT met**, and `docs/ROADMAP_ARCHIVE_2026-07-26.md` stays until set B is closed.

🔓 **JOE'S DECISION — SUPERSEDED BY §4a BELOW; the framing here was wrong.** *Does set B get retrospective entries (the J-102/J-103 precedent), or is the ROADMAP prose kept in place?*

### ⚠️ §4a — THE ABOVE READING WAS WRONG. RETRACTED AND CORRECTED (2026-07-26, after Joe asked for the context)

⚠️ **CHAT'S RETRACTED CLAIM, quoted so it is not re-derived:** *"A citation means someone, at the time of writing, believed the entry existed. That is a far stronger signal of real loss than set A."* **FALSE.** It reasoned from a pattern without checking what produced the pattern — **the same defect shape as the M-RP6.6 gate-versus-author collapse, twice in one session.**

🔑 **WHAT ACTUALLY PRODUCED SET B — A DOCUMENTED CONVENTION CHANGE, RECORDED IN J-129 SUB-SECTION 8.** The `> **Last updated:**` line in `CLAUDE.md`, `JOURNAL.md` and `ROADMAP.md` had grown into a **chain** — a running narrative appended to the header. Measured there: **CLAUDE.md ~125 KB of chain across lines 5-16, one line alone 71.8 KB; ROADMAP 80 KB in line 9 alone.** It caused two concrete failures — reads returned the chain instead of the document body (*"false state diagnosis followed"*), and edits grew fragile because `oldText` had to match an ever-larger string (*"the root of the prose-then-batch atomicity-slip family"*).

⇒ **DOC-ONLY MILESTONE EVENTS WERE DELIBERATELY RECORDED AS HEADER-CHAIN ENTRIES WITH NO JOURNAL BODY** — ROADMAP names it verbatim as the *"chain-only doc-only milestone-event precedent"* for J-123 / J-124 / J-125. **At J-129 the chain was stripped.** The pointers went; the bodies had never existed. **Nothing was lost by accident. It was never written, by a rule, and then its index was deleted, by another rule.**

🔑 **AND THE RECORD FOR SOME OF THOSE EVENTS IS IN `ROADMAP.md`'s PAST SECTION**, in full body shape — `✅ **J-124 … runbook SHIPPED 2026-05-27**`, and likewise J-125, J-131, J-132, J-135. **ROADMAP became the de facto journal for that class of event.**

### ⚠️ §4a-i — CORRECTION: *"SOME"* IS DOING WORK THE ORIGINAL SENTENCE DID NOT. **SIX OF THE TEN HAVE A BODY; FOUR HAVE NONE ANYWHERE.** MEASURED 2026-07-26 AT `a1d3630`

⚠️ **THE SENTENCE ABOVE ORIGINALLY READ *"THE RECORD FOR THOSE EVENTS IS IN ROADMAP's PAST SECTION"* and named five examples.** Read as covering set B, it is **false for four of the ten** — and it was written without enumerating the other five. 🔑 **This is the FIFTH instance this session of the arc's recurring defect: a claim narrower than the thing it described, reused as if complete. It is Chat's.** 📌 The word *"some"* is inserted above so the sentence is true where it stands; the enumeration is here.

**Every occurrence of each set-B number in `ROADMAP.md`, classified by whether the line is the entry's OWN record or a reference to it from elsewhere:**

| Set-B number | ROADMAP refs | Own body in ROADMAP | Verdict |
|---|---|---|---|
| `J-135` | 7 | ✅ line 506 | body survives |
| `J-132` | 5 | ✅ line 518 | body survives |
| `J-131` | 10 | ✅ line 522 | body survives |
| `J-125` | 27 | ✅ line 542 | body survives |
| `J-124` | 24 | ✅ line 546 | body survives |
| `J-123` | 15 | ✅ line 550 | body survives — 📌 **not named in the original sentence** |
| `J-113` | 6 | ❌ none | ⚠️ **cited only** |
| `J-109` | 8 | ❌ none | ⚠️ **cited only** |
| `J-098` | **45** | ❌ none | ⚠️ **cited only — forty-five references and no record of its own** |
| `J-067` | 1 | ❌ none | ⚠️ **cited only** |

**Sampled to confirm the classification rather than trusting the pattern:** `J-067`'s single occurrence is *"the drift surface that produced F-003/F-004 in J-067"* (line 360) — another entry referring back. `J-098`'s are a Joe-lock citation (line 110), a cross-reference list (416) and *"Shape 2 per Joe-lock at J-098 session close"* (418). `J-109` and `J-113` appear almost entirely inside one recurring phrase, *"three-instance threshold met (J-099 + J-109 + J-113)"*. ⇒ **all four are referenced BY other records and hold none themselves.**

🔑 **TWO CONSEQUENCES, AND THEY PULL IN OPPOSITE DIRECTIONS.**
1. ✅ **THE MIGRATION IS SMALLER THAN §4b's RULING ASSUMED — SIX BODIES, NOT TEN.** §4b says *migrate the ROADMAP prose*; for four of the ten **there is no prose to migrate.**
2. ⚠️ **AND THOSE FOUR ARE A DIFFERENT KIND OF THING, NOT A SMALLER ONE.** §4a's diagnosis — *never written by rule, then de-indexed by another rule* — **now applies with nothing to fall back on.** For the six, ROADMAP held the body all along. **For these four, no document has ever held one**, and the only honest record is a line stating that the number was cited by others and never itself written. 📌 **That is not information loss** (nothing was destroyed) **and it is not a gap to be filled with reconstruction** — inventing a body from the citations would be exactly the invent-a-link failure §4c already rejected once.

🔓 **CONSEQUENT AMENDMENT PROPOSED TO §4b, JOE'S — THREE WORDS.** §4b reads *"migrate the ROADMAP prose into `JOURNAL` **at those numbers**"*. ⚠️ **That phrase is the only thing that forces `JOURNAL_ARCHIVE.md` open**, because those numbers fall inside its span — which is what §8b then ran into. **CHAT PROPOSES: *"labelled with those numbers, in a forward entry"*.** One new entry in `JOURNAL.md` carries the six surviving bodies, each labelled with the designation it was originally allocated, plus one line each for the four never written.
- **① User-visible:** none, either way.
- **② Tier:** none.
- **③ Resource: one journal entry, against opening a frozen archive plus 28 citation re-points.** 🔑 **And the structural gain is the point, not the saving: §8b comes OFF the critical path.** It remains a real and unruled question — the 24 KB duplication and the `J-044`/`J-045` collisions still sit in the archive — but **nothing waits on it.**

⚠️ **Searchability is the one thing this must not cost, and it does not:** a reader looking for `J-124` finds the forward entry by the same text search that finds anything else. **What it does cost is chronological position** — the six bodies sit at the end of the journal rather than at their 2026-05 slot. 📌 **Stated as the trade it is; the provenance line carries the original date.**

🔑 **WHICH MAKES THIS MILESTONE THE SECOND HALF OF A TREATMENT THAT STARTED AT J-129.** J-129's own words: *"The chain was emergent prose that bled JOURNAL's job into the header line. Substantive narrative belongs in JOURNAL body entries."* **The Past section is the same prose bleeding into the same document's body.** Same defect, same three files, same cure — diagnosed by this project two months ago and half-treated. 📌 **Joe reached the diagnosis independently tonight without the J-129 text in view.**

### 🔒 §4b — RULING: MIGRATE, PLUS A PROJECT-WIDE DESIGNATION RULE (Joe, 2026-07-26)

🔒 **OPTION (a) — MIGRATE THE ROADMAP PROSE INTO `JOURNAL` AT THOSE NUMBERS**, with honest provenance (*migrated from ROADMAP; originally a stripped header-chain entry*). The journal becomes genuinely complete — the premise the whole roadtree rests on — and the roadtree comes out pure.

🔒 **AND THE DESIGNATION-COLLISION RULE (Joe, general, not migration-local): WHERE A DESIGNATION IS DUPLICATED, THE ORIGINAL TAKES AN `a` SUFFIX AND THE NEW ONE TAKES `b`** — e.g. `J-044a` / `J-044b`.

📌 **PROPOSAL: PROMOTE IT.** The rule is project-wide — it governs J-, D-, N- and M- designations alike, not just this migration. It belongs in `CLAUDE.md`'s conventions or as a D-entry, **not buried in a milestone task file.** 🔓 Joe's.

### 🔑 §4c — THE RULE HAS IMMEDIATE RETROACTIVE WORK, AND IT SPLITS IN TWO. MEASURED 2026-07-26

**581 entry headings across both journal files, 574 unique ⇒ SEVEN duplicated designations. No `a`/`b` suffix exists anywhere yet.** Comparing bodies byte-for-byte splits them cleanly:

| Set | Designations | Bodies | Verdict |
|---|---|---|---|
| **Literal duplication** | `J-317 · J-318 · J-319 · J-320 · J-321` — five consecutive | **BYTE-IDENTICAL** (4853 / 3834 / 4275 / 5510 / 5443 chars, each pair exact) | ⚠️ **NOT a designation collision — a copy-paste or merge accident, ~24 KB of exact duplication.** 🔒 **The fix is DELETION of one copy. Suffixing would enshrine an accident as two events** |
| **True collision** | `J-044 · J-045` | **DIFFER** (3017 vs 3653 · 2502 vs 2706) | ✅ **Exactly what Joe's rule is for — two distinct events sharing one number** |

**The two true collisions, and which copy is which.** All four carry the same date (`2026-05-13`), so **date does not disambiguate — file order does**: the archive runs newest-first, so the **higher line number is the original**.

| Designation | Original → `a` | New → `b` |
|---|---|---|
| **J-044** | line 17057 — *BATCH_FLAG_ph2: M1–M3 implemented (code complete, M4 walkthrough pending)* | line 16957 — *BATCH_FLAG_ph2: implementation review; error message fix; documentation updates* |
| **J-045** | line 17161 — *XGEN_CORE_SPLIT_ph2: xgen-core crate split complete* | line 17128 — *Design note: `--batch` as a primary AI tool for tuning and debugging* |

📌 **J-044's content corroborates the file order** — *implemented* precedes *implementation review*. ⚠️ **J-045's content is neutral between the two**, so that assignment rests on file order alone. **Stated, not hidden.**

🔑 **AND THE PASS PRODUCED A FORMAT AMENDMENT, WHICH IS WHY IT RAN BEFORE THE REWRITE AND NOT AFTER.** §3 originally required `· J-nnn` on every ✅ node, *non-negotiable*. **Applied literally to lines 376 and 396 it would have forced one of two wrong outcomes: invent a link, or declare information loss** — when the record exists, correctly, in the design document that owns that work. ⇒ **§3's rule now reads *a link to its canonical record*, usually but not always a J-number.** 📌 **The 11 P2 refs are still J-numbers and are unaffected.**

📌 **Cost revised: the P1 walk is not an afternoon — it was five entries and all five are resolved.** ⚠️ **The archive's delete condition is NOT yet met** — P2 set B is open.

---

🔒 **NOTHING IS DELETED UNTIL BOTH SETS ARE CLEARED.** This is what makes §2's claim true rather than asserted.

**(P1) 94 ✅ DONE MARKERS SIT ON LINES CARRYING NO `J-` REFERENCE AT ALL.** ⚠️ **This is the real risk.** Unlike P2 there is **no pointer to check against** — if the prose goes, whatever it says goes with it. **Each must gain a journal link or be confirmed redundant.** 94 items: an afternoon, not an arc.

**(P2) 11 J-REFERENCES RESOLVE NOWHERE** — not in `JOURNAL.md`, not in `JOURNAL_ARCHIVE.md`:
`J-029 · J-067 · J-098 · J-109 · J-113 · J-123 · J-124 · J-125 · J-131 · J-132 · J-135`
📌 Either genuinely absent, or a heading-format variant the entry-regex missed. **Spot-check, do not guess.**

📌 **Expect more arrow mis-attributions of the §3 kind among P1.** The first sample of one produced one.

---

## §5 — 🔒 DECISION 3: M-RP-MEMBERS LEG C IS PAUSED (Joe, 2026-07-26)

🔒 **Joe: *"we need to pause M-RP-MEMBERS — LEG C."*** ⇒ ⏸️ **POSTPONED.**

⚠️ **AND IT CARRIES ITS RESUME TRIGGER, BECAUSE A PAUSE WITHOUT ONE IS EXACTLY THE DEFECT THIS MILESTONE EXISTS TO FIX.** Pausing Leg C the way M-RP6.2's deferral was written would manufacture instance two on the same day.

```
⏸️ **Leg C** — live CDP verify + the real-join membership test
   ↳ trigger: M-RP-LIVEFEED-REFRESH Leg A lands (§5 was never built)
```

📌 **Legs A / A-bis / A-ter / A-quater / PANEL-INERT / B remain ✅ DONE.** Only Leg C pauses; the milestone reads ⏸️ POSTPONED with the named condition.

📌 **GitHub project board must agree** — the POSTPONED field option is `d0103551`. A board and a roadmap that disagree are two sources of truth.

🔑 **AND THE ORDER MATTERS: THE PAUSE LANDS BEFORE THE AUDIT WALKS THE TREE.** Auditing a document while one of its known-false entries is still false means the audit either rediscovers it or carries it in its head.

---

## §6 — 🔒 DECISION 4: SCOPE — BOTH. ROADMAP **AND** THE CLAUDE.md PLAY HEAD (Joe, 2026-07-26)

`ROADMAP.md` and the `CLAUDE.md` PLAY block are **paired under D-074** — they travel in the same commit on every state change. `CLAUDE.md` is **608,702 bytes with a single 124,299-character line** and has the identical disease: append-only narration in a document meant to show current state.

- **① User-visible:** none. Both are internal records. **"No user-facing impact" is a legal answer (D-121).**
- **② Tier:** none.
- **③ Resource:** the split mechanism is the same for both, so doing them together is **cheaper than doing them twice**. Doing only ROADMAP leaves the two **free to disagree**, which is the state D-074 exists to prevent.

**CHAT PROPOSED: BOTH.**

🔒 **RULED — BOTH. ⚠️ PROVENANCE: DELEGATED, NOT A CONSIDERED LOCK.** Joe: *"both points by your recommendations."* **Recorded as delegation** per the standing convention that *"as you recommend"* is not the same act as a lock. 📌 **The practical consequence of that distinction: if Leg D's first pass shows the PLAY head does not fit §3's node format, that is not a Joe reversal — it is the delegation reaching its limit, and it comes back to him.**

🔑 **AND THE SCOPE RULING CARRIES ONE THING CHAT'S PROPOSAL EXPLICITLY DID NOT:** Leg D is in scope, but **§3's format is NOT assumed to be Leg D's format.** The two documents share a *disease* (append-only narration in a document meant to show current state); they do **not** demonstrably share a *cure*. ROADMAP is 586 closed nodes in a nested structure; the PLAY head is a **block sequence**, its worst offender a single 124,299-character line. ⚠️ **Applying a node format to a block document without measuring it first is exactly the "claim narrower than the thing it described, reused as if complete" defect.** ⇒ **Leg D opens with its own grounding pass, and its format is 🔓 OPEN until that pass runs.**

---

## §7 — 🔒 DECISION 5: BY STRUCTURE. IT IS A TREE, NOT A BOARD (Joe, 2026-07-26)

§3's format takes its shape from **nesting alone** — no track/status/date columns. That is a bet that **structure is what Joe navigates by**.

- **(S1) BY STRUCTURE — the tree as specified.** ① You find a thing by knowing where it lives. ③ One document, one ordering.
- **(S2) BY STATUS — "what is in play right now" as its own section.** ① You find a thing by knowing its state. ⚠️ **That is a board, not a tree**, and it is a different document from the one §3 describes.

⚠️ **THIS CHANGES EVERYTHING DOWNSTREAM AND MUST BE ANSWERED BEFORE LEG B.** **CHAT PROPOSED S1** — the status symbols already make state greppable within a tree, whereas structure cannot be recovered from a status grouping.

🔒 **RULED — S1, BY STRUCTURE. ⚠️ PROVENANCE: DELEGATED, NOT A CONSIDERED LOCK** (same act as §6; Joe: *"both points by your recommendations"*).

🔑 **THE TWO REASONS, RESTATED SO LEG C CAN BE CHECKED AGAINST THEM RATHER THAN AGAINST A PREFERENCE:**
1. **RECOVERABILITY IS ASYMMETRIC.** State is recoverable from a tree by one grep on a symbol. **Structure is NOT recoverable from a status grouping** — once a node sits under *DONE* rather than under its arc, the arc it belonged to is gone from the document and can only be reconstructed from the journal.
2. **A BOARD RE-CREATES THE ORIGINATING DEFECT.** M-RP6.2's deferral was lost because nothing *adjacent to it* said so. A status grouping puts a closed node and its `→ successor` **in different sections** — which is precisely the separation §3's arrow-on-the-closed-node exists to abolish. ⇒ **S2 would have been format and mechanism working against each other.**

⚠️ **THE COST S1 DOES NOT PAY, STATED AND NOT HIDDEN: the tree does not answer *"what is in play right now"* at a glance.** The 69 🟢 markers prove that question is currently unanswerable anyway, and §10 already files that no convention forces 🟢 to be exclusive. 📌 **If that glance turns out to be what the file is wanted for, the answer is a derived view, never a second grouping of the same nodes** — a second grouping is a second source of truth (D-067), the same objection that killed `ROADMAP_HISTORY.md` at §2.

---

## §8 — Legs

**Leg 0 — Phase-0.** This document. No code. ✅ **UNGATED — §6 and §7 both ruled 2026-07-26 (delegated).**

**Leg A — the pause + the archive.** ⏸️ M-RP-MEMBERS Leg C with its trigger (§5), in `ROADMAP.md` + `CLAUDE.md` PLAY + the GitHub board. ✅ **The archive is already taken** — `docs/ROADMAP_ARCHIVE_2026-07-26.md`, **content-identical, header replaced with `Status: ARCHIVED` (749,428 B vs the source's then-749,717 — ⚠️ corrected from "byte-identical", §1a)**, **carrying its own delete condition**. **Surface: `docs/ROADMAP.md`, `CLAUDE.md`, board field `d0103551`.**

**Leg B — clear the precondition.** Walk P1's **94** unlinked DONE markers and P2's **11** unresolved refs. **Surface: `docs/ROADMAP.md`, `JOURNAL.md`, `JOURNAL_ARCHIVE.md`.** ⚠️ **No deletion happens in this leg.** Output is a link-or-redundant verdict per item, and any journal corrections P2 turns up. 📌 **P1 ✅ CLEARED at J-599. P2 measured, NOT cleared — its remedy is §8a below.**

**Leg C — the migration.** 🔒 **SCOPE LOCKED (Joe, 2026-07-26): *keep the tree that exists · apply §3's field rules to its nodes · delete the prose.*** 📌 **A LOCK, not a delegation** — Joe answered a bounded direct question, unlike §6/§7. **Runbook: `tasks/RUNBOOK_ROADTREE_LEGC.md` v1.0 ACTIVE.** **Surface: `docs/ROADMAP.md`.** ✅ **UNBLOCKED.**

🔑 **AND THE UNBLOCK IS §0 OF THAT RUNBOOK, NOT A WAIVER OF §4.** §4 demanded 775 entries be proven redundant before any deletion. **`ROADMAP_ARCHIVE_2026-07-26.md` is committed and git holds every byte of every version** ⇒ **nothing here is recoverable-in-principle; it is recoverable in two commands.** ⚠️ **The bar was mis-set, not met early:** the real question is *do the surviving tree's links work* — **81 lookups** — and that is Pass 1. 📌 **§4's precondition is therefore SATISFIED BY V7 (`git show a1d3630:docs/ROADMAP.md`), which runs BEFORE the deletion, not by the walk it originally specified.**

⚠️ **ONE DoD NUMBER MOVES AND IT IS NOT FUDGED.** §9 requires ROADMAP *under 100 KB*; the surviving tree alone is **79,802 B** and the kept scaffolding brings the end state to **~113 KB**. 🔓 **Chat proposes §9's bar move to `≤ 120 KB`** — the alternative is trimming *"How to use this view"* (12.9 KB) and *"Cross-cutting"* (12.4 KB), which are **instructions and standing context, not chronicle**, and cutting them to hit a round number would be **optimising the metric rather than the document.**

**Leg D — CLAUDE.md.** Same treatment on the PLAY head. **Surface: `CLAUDE.md`, `CLAUDE_HISTORY.md`.** ✅ **In scope — §6 ruled BOTH.** 🔓 **Its FORMAT is open** and opens with its own grounding pass (§6's rider): §3's node format is not assumed to transfer to a block document.

---

**Leg C — CLOSED (J-604).** ✅ `docs/ROADMAP.md` **761,422 → 43,741 B, 94.3% smaller**. The tree was kept and repaired, ten missing milestones grafted, `Past` and `Present` deleted, and five format rules R-1…R-5 written into the decode key with R-6 for link chains. ⚠️ **THE 120 KB AMENDMENT ABOVE IS WITHDRAWN** — the end state beat the original 100 KB bar by more than half. Runbook `tasks/RUNBOOK_ROADTREE_LEGC.md` v1.4 COMPLETED.

**Leg D — `CLAUDE.md` B2. ✅ CLOSED (J-615).** **640,645 → 316,680 B — 50.6%**, beating the §6b forecast of ~410,000 by 93,000. **65 of 81 blocks archived, 16 remain, 65 + 16 = 81 asserted.** `CLAUDE_HISTORY.md` 869,178 → 1,197,988 B, 185 → 186 `## ` headings — **one** heading written, every block byte-identical (F1c). **23 annotations** applied at the sites of stale claims, travelling with their blocks (D-131 form). ⚠️ **L29 SURVIVES AT 124,299 CHARS** — 39% of the remaining file, and Leg E's whole subject. Runbook `tasks/RUNBOOK_ROADTREE_LEGD.md` v1.3 COMPLETED · Phase-0 brief v1.6. Executed by `legd-move.ps1` + `legd-annotations.json`, dry-run first, refusing to write unless every assertion passed.

**Owes:** §E L75 — the M-RP-SKIN corner-triangle countdown has no home · §E L235 — M-RP-SELF-SURFACE has no task doc and no ROADMAP node.

🔒 **THREE RULINGS DELEGATED 2026-07-29 (Joe: *go as you recommend*), options authored by Chat.** **§F1c** — the archive uses `## ` headings and the head writes `> ###`; the batch lands verbatim **under one `## ` batch heading**, which is the only form that keeps D-094's *never a rewrite* while leaving the archive's structure conforming. **§F2b** — `CLAUDE_HISTORY.md` stays **ARCHIVED**, and the D-094 insert is written into its preamble as its **single sanctioned exception**; the vocabulary had contradicted itself since the file was created and only the lapse hid it. **§3a W1** — L195, L199, L201 archive annotated, because they declare 🟢 PLAY on closed legs exactly as the eleven mis-symbolled blocks do.

🔑 **IT IS NOT A FORMAT-INVENTION PROBLEM.** **D-094 already rules PLAY-block archiving**, `CLAUDE_HISTORY.md` already holds **185 archived blocks**, and the rule **lapsed 2026-06-22**. 81 blocks have accreted in the live head since, with a clean boundary — history J-81…J-405, head J-519…J-604 — and **zero duplication in either direction**. Census of the 81: **42 ✅ · 17 live (🟢🟡) · 22 non-work (🔒🔑🛑⚠️)**. ⚠️ **THIS IS A SYMBOL CENSUS AND NOTHING MORE — IT WAS READ AS A PARTITION AND THAT COST TWO REVISIONS (J-613).** *"17 live"* names the blocks that **lead** with 🟢🟡; **eleven of them are closed work.**

🔒 **§6a, §6b AND §6c ARE ALL RULED (§6b — S1, B2 only, J-611).** **§6a — E (Joe locked 2026-07-28, option authored by Chat):** classify each block by **what its own text says, not by its lead symbol**; blocks that self-declare closed archive with the 42, and the residue stays in the live head to be handled when work next reaches it — D-131 applied to blocks instead of citations. ⚠️ **R2 IS WITHDRAWN.** Reading the 22 showed the lead symbol marks the **finding inside** the block, not the block's state (L193 leads 🔒 and opens *Leg C ✅ CLOSED*), and that their designations — `N-118`, `N-120`, `N-124`, `N-124a`, `N-124b`, `D-122`, `D-123` — **already have canonical headings**. 📌 **The promotion-successor milestone may not exist as separate work; the residue looks like 4–6 blocks.** *[⚠️ SUPERSEDED — the measured residue is 14 blocks (J-611), and §A's read re-cuts it again (J-612).]* **§6c — D-131:** a citation proven broken is annotated at the site, never silently repaired. 🔒 **§6b RULED — S1 (delegated 2026-07-28): LEG D COVERS B2 ONLY.** B2 is 81 discrete `> ###` blocks with findable starts and ends; **B1 has no boundaries to move** — its bulk is L29, one line of 124,299 chars. ⚠️ **Stated cost:** Leg D takes `CLAUDE.md` **640,645 → ~410,000 B** and **L29 survives untouched**. Bounded and finishing, but it does not fix what made the file unreadable.
🛑 **NOTHING IS AUTHORISED FOR DELETION.** Phase-0 §7 lists four things **NOT MEASURED**, the first being that a resolving citation proves a record exists and **not** that the block's substance is in it.

🛑 **THE MOVE SET IS 62 BLOCKS — THIRD REVISION, ONE CAUSE (J-613).** Kickoff said 7 self-declared closed, Phase-0 §6a said 6, runbook v1.0 said a move set of 50. **Each classified a bucket it was handed rather than all 81 blocks.** Measured: **42 ✅ + 4 self-declared closed** (L177 L193 L251 L255) **+ 5 stale-closed** (L175 L241 L243 L245 L249) **+ 11 that the census called 🟢 live** (L123 L145 L151 L155 L161 L165 L167 L191 L219 L259 L261) **= 62**; **19 stay**; 62 + 19 = 81 ✔. 🔑 **THE DECISIVE EVIDENCE IS INSIDE `CLAUDE.md`:** every one of those eleven is a phase-0 or design-lock block whose milestone **this same head later closes** — L123 against L125, L259 against L189, L219 against L213. ⚠️ **THE 17 🟢🟡 BLOCKS HAD NEVER BEEN CLASSIFIED**, only counted as live by lead symbol, which is exactly what ruling E forbids. 🔓 **L195 L199 L201 remain open under §3a (W1 recommended)**; under W1 the set is 65. **2 held** for pointing at milestones with no record anywhere (L75 → M-RP-SKIN, L235 → M-RP-SELF-SURFACE). `tasks/RUNBOOK_ROADTREE_LEGD.md` §A is the authority.

**Leg E — the B1 prose and the 124,299-char accretion line.** 🟡 **PENDING.** ↳ trigger: Leg D closes — **FIRED 2026-07-29 (J-615)**. **Surface: `CLAUDE.md` L21–70.** 🔑 **It opens with its own grounding pass** — what L29 contains has never been measured, only sized, and nothing can move until someone reads it and decides where one statement ends and the next begins. ⇒ **PARSING, NOT MOVING.** 🔓 **The descriptive title is Joe's** (D-123 — naming); the letter is mechanical.

📌 **LEG D SHIPS NO PLAY BLOCK (P2, delegated 2026-07-28).** D-074's `CLAUDE.md` limb is deliberately empty for this milestone: adding block 82 to the head Leg D exists to drain would grow the defect it is closing. Recorded so the omission is never read as drift.
### ⚠️ §8a — THREE PIECES OF RULED WORK HAVE NO LEG, AND ONE OF THEM CONTRADICTS THE LEG IT WOULD FALL INTO. 🔓 SEQUENCING IS JOE'S

🔑 **FOUND BY APPLYING `M_RP_MEMBERS.md` §8b's OWN RULE TO THIS DOCUMENT** — *walk every 🔒 and ask WHICH LEG BUILDS THIS.* **§4b and §4c lock three concrete actions. §8 as written builds none of them:**

| Ruled work | Where it is locked | Which leg builds it |
|---|---|---|
| **MIGRATE** the ten set-B entries' prose from ROADMAP's Past section into `JOURNAL.md` at those numbers, with honest provenance | §4b, 🔒 Joe | — **none** |
| **DELETE** one copy of the byte-identical `J-317 … J-321` blocks (~24 KB) | §4c, 🔒 | — **none**, ⚠️ **and Leg B says *"No deletion happens in this leg"*** |
| **SPLIT** `J-044` / `J-045` into `a` / `b` and **retire the bare numbers** | §4c + the CLAUDE.md convention, 🔒 | — **none** |

⚠️ **THIS IS INSTANCE FOUR OF THE SAME DEFECT, AND IT IS CHAT'S AGAIN** — *scope written in FILES, requirements written in BEHAVIOURS, never reconciled.* §8's legs are named by **file surface** (ROADMAP · CLAUDE.md · JOURNAL); §4's rulings are named by **behaviour** (migrate · delete · split). **Nobody reconciled them.** 📌 **Filed as a live instance rather than absorbed silently, per the standing rule.**

🔑 **AND THE ORDERING IS NOT COSMETIC — IT IS LOAD-BEARING.** §4's rule is that **nothing is deleted until both precondition sets clear**. For set B, **the ROADMAP line is currently the only trace.** ⇒ **Leg C cannot rewrite ROADMAP until that prose lives in JOURNAL**, or the rewrite destroys the only copy. **The journal work is a HARD PREDECESSOR of the migration, not a tidy-up after it.**

**CHAT PROPOSES — a new leg, inserted between B and C:**

> **Leg B-bis — the journal repair.** Three actions, one surface. **Surface: `JOURNAL.md`, `JOURNAL_ARCHIVE.md`** (ROADMAP is **read-only** in this leg — its prose is copied out, not yet removed).
> 1. **Migrate** the **six surviving set-B bodies** into `JOURNAL` (§4a-i — `J-123 · J-124 · J-125 · J-131 · J-132 · J-135`), each carrying the provenance line *“migrated from ROADMAP; originally a stripped header-chain entry (J-129 §8)”*, **plus one line each recording `J-067 · J-098 · J-109 · J-113` as cited-but-never-written.** ⚠️ **Do NOT reconstruct a body for those four from their citations.** 🔓 **Whether this lands in the archive at those numbers or as a forward entry labelled with them is §4a-i's open amendment — and it decides whether this leg touches `JOURNAL_ARCHIVE.md` at all.**
> 2. **Delete** one copy of each byte-identical `J-317–J-321` block. ⚠️ **Verify byte-identity immediately before deleting, not from this document's record of it** — a duplicate re-verified is cheap; a wrong delete is unrecoverable outside git.
> 3. **Split** `J-044` → `J-044a`/`J-044b` and `J-045` → `J-045a`/`J-045b`, **retiring both bare numbers** (normative, per the discriminator rule). ⚠️ **`J-045`'s assignment rests on FILE ORDER ALONE** — §4c states this and it must survive into the entry text, not just this task file.
> ⚠️ **Every existing citation of `J-044` / `J-045` must be re-pointed in the same commit** — ROADMAP, CLAUDE.md, DECISIONS.md, other task files. **A retired bare number left cited is the exact silent-mis-point failure the convention forbids.** ➕ **Unmeasured: how many such citations exist.** Chat owes that count before the leg is runbooked.

- **① User-visible:** none, either way.
- **② Tier:** none.
- **③ Resource:** the three actions share one surface, one verification method and one commit ⇒ **cheaper as one leg than distributed across B and C.** Splitting them would also put a **deletion** inside a leg that forbids deletion.

🔓 **OPEN — order of work is Joe's (D-123).** Chat can write it; Chat does not get to insert a leg into the sequence.

---

### 🛑 §8b — AND MEASURING THAT LEG'S SURFACE FOUND SOMETHING THAT OUTRANKS IT: **EVERY PIECE OF §4's RULED RETROACTIVE WORK TARGETS A FILE THAT DECLARES ITSELF IMMUTABLE.** MEASURED 2026-07-26 AT `ec0d305`

🔑 **`JOURNAL_ARCHIVE.md`'s OWN HEADER, QUOTED:**

> `> **Status:** ARCHIVED`
> *“This document is the frozen archive of older XGen Protocol development-journal entries… **Entries are verbatim and unaltered; do not modify.**”*

**And Joe's own status taxonomy: `ARCHIVED — Frozen historical record, do not modify.`**

**WHERE THE WORK ACTUALLY LANDS — enumerated, not inferred:**

| Ruled action | Target numbers | File that holds them |
|---|---|---|
| §4b **migrate** set B | `J-067 · J-098 · J-109 · J-113 · J-123 · J-124 · J-125 · J-131 · J-132 · J-135` | ⚠️ **`JOURNAL_ARCHIVE.md`** — all ten fall inside its declared span `J-375 … J-046`. 📌 **Only if *"at those numbers"* stands** — §4a-i's amendment removes this row from the table entirely |
| §4c **delete** the duplicates | `J-317 · J-318 · J-319 · J-320 · J-321` | ⚠️ **`JOURNAL_ARCHIVE.md`, both copies** — L1208/L1228 · L1248/L1270 · L1292/L1312 · L1332/L1352 · L1372/L1400 |
| §4c **split** the collisions | `J-044 · J-045` | ⚠️ **`JOURNAL_ARCHIVE.md`** — L16957/L17057 · L17128/L17161 |

📌 **`JOURNAL.md` is clean: zero duplicate headings, zero set-B numbers.** The entire retroactive workload sits in the one file nobody is allowed to touch. **⇒ Leg B-bis as §8a proposes it cannot be executed as written.**

🔑 **AND THE DUPLICATION MECHANISM IS NOT WHAT §4c ASSUMED.** §4c reads *“a copy-paste or merge accident”*, which implies a contiguous five-entry block pasted twice. **The measured line order is `321,321,320,320,319,319,318,318,317,317` — each entry doubled IN PLACE, not the block repeated.** ⚠️ **A block paste would read `321,320,319,318,317,321,320,319,318,317`. It does not.** ⇒ **The cause was per-entry, most likely a write that emitted each entry twice** — stated as the better-supported reading, **not** as established. 📌 **The remedy is unchanged (delete one copy of each); the record of WHY is corrected.**

➕ **CITATION LOAD FOR THE `a`/`b` SPLIT — MEASURED, DISCHARGING §8a's OWED COUNT.** Repo-wide `.md`, `.claude` excluded:

| Designation | Total refs | Excluding this task file and `ROADMAP_ARCHIVE` | Files needing a re-point |
|---|---|---|---|
| `J-044` | 26 | **16** | `BATCH_FLAG_ph2.md` 3 · `M2_NODE_PIPE_SERVER.md` 2 · `ROADMAP.md` 1 · `CLAUDE.md` 1 · `DECISIONS.md` 2 · `JOURNAL.md` 4 · `JOURNAL_ARCHIVE.md` 3 |
| `J-045` | 22 | **12** | `BATCH_FLAG_ph2.md` 1 · `XGEN_CORE_SPLIT_ph2.md` 1 · `ROADMAP.md` 2 · `CLAUDE.md` 2 · `JOURNAL.md` 4 · `JOURNAL_ARCHIVE.md` 2 |

⚠️ **Each of the 28 must be resolved to `a` or `b` INDIVIDUALLY — a bare `J-044` in `DECISIONS.md` means one of the two events and there is no mechanical way to tell which.** 📌 **This is the real cost of the split, and it is larger than the split itself.**

---

#### 🔓 THE DECISION §8b FORCES — WHAT DOES `ARCHIVED` MEAN WHEN THE ARCHIVE IS WRONG? **JOE'S. NOT CHAT'S TO DELEGATE-AWAY.**

⚠️ **This is not a process question — it sets what an archival guarantee is worth project-wide, and the project's whole IP-provenance claim rests on that guarantee.** Chat proposes; Chat must not rule.

**(R1) THE ARCHIVE IS TRULY IMMUTABLE.** §4b/§4c are **withdrawn as stated**; corrections land as **new forward entries in `JOURNAL.md`** that say *“J-044 designates two events; they are hereafter J-044a and J-044b”*, and the archive keeps its defects with a pointer.
① None. ② None. ③ **Cheapest** — ~2 new entries, zero re-points, no deletion. ⚠️ **But the 24 KB of duplication STAYS, the ten entries STAY missing, and every reader must know to check a corrections list.** 🔑 **It preserves the guarantee by preserving the errors.**

**(R2) ARCHIVED MEANS “NO NEW CONTENT”, NOT “NO REPAIR”.** A bounded exception: **deduplication, designation-splitting and gap-filling are REPAIRS; adding new narrative is not.** §4b/§4c execute in place; the archive's header gains an explicit amendment clause and a `Last updated`.
① None. ② None. ③ Highest — the full 28-citation re-point plus the migration. ⚠️ **Its real cost is precedent: once “repair” is a legal reason to edit an archive, the class of things that count as repair is decided case by case, by whoever is editing.**

**(R3) UNFREEZE, REPAIR, RE-FREEZE — AS ONE NAMED, DATED EVENT.** The archive goes `ARCHIVED → ACTIVE` for exactly one commit, takes §4b + §4c, and returns to `ARCHIVED` with a **`Repaired: 2026-07-xx (J-nnn)`** line naming precisely what changed and why.
① None. ② None. ③ Same as R2, **plus one status round-trip.** 🔑 **The difference from R2 is that the exception is an EVENT with a date and a journal number, not a STANDING CLAUSE** — so it cannot be invoked again without repeating the whole visible act. ⚠️ **And git already holds the pre-repair bytes**, so “verbatim and unaltered” survives as a recoverable claim rather than a file-level one.

**CHAT PROPOSES R3.** The defects are real and R1 leaves them in place forever; R2 buys the same repair by permanently weakening the word `ARCHIVED`; **R3 pays once, in public, and the guarantee's exception is itself part of the record.** 🔓 **OPEN — and it gates Leg B-bis, which gates Leg C.**

📌 **A smaller one riding along:** the archive's own header says *“Live window (J-395 … J-376) continues in `JOURNAL.md`”* — **`JOURNAL.md` now runs J-376 … J-599.** The header is stale **and unfixable under its own rule**, which is the same collision in miniature.

---

**Leg E — the bidirectional sweep.** ⚠️ **Not only *is every roadmap entry true*, but *is every known work item ON the roadmap*.** **Surface: this session's open threads** — M-RP-LIVEFEED-REFRESH · the resync sibling · the outbox · H1 · H2 · D-130 · the address-book eviction question · `NegotiatedCapabilities` · the Ch0–Ch2 thesis read. **They exist in chat and in one task document and nowhere else.** They are the natural test case for whether the sweep works.

**Leg F — records + close.** JOURNAL + CLAUDE.md PLAY + ROADMAP + this document, **one commit** (D-074). Delete `ROADMAP_ARCHIVE_2026-07-26.md` **iff** Leg B cleared.

---

## §9 — DoD

Applying `M_RP_MEMBERS.md` §8b's rule — **every item naming an action names its surface in §8, and every 🔒 in this document has a leg that builds it:**

- [ ] M-RP-MEMBERS Leg C reads ⏸️ **with its trigger** in ROADMAP, CLAUDE.md and the board — **Leg A**
- [ ] All **94** P1 items **measured**: journal link found, or redundancy confirmed in writing — **Leg B**
- [ ] All **11** P2 refs **measured**: entry located, or absence recorded — **Leg B**
- [ ] The **`ARCHIVED`-versus-repair question (§8b) ruled** by Joe, and the ruling recorded where a future reader of `JOURNAL_ARCHIVE.md` will find it — **precedes Leg B-bis**
- [ ] The **six surviving set-B bodies present in a journal file**, each carrying its provenance line, **and the four never-written numbers recorded as such** (§4a-i) — **Leg B-bis**
- [ ] `J-317`–`J-321` **re-verified byte-identical, then reduced to one copy each**, count re-measured after — **Leg B-bis**
- [ ] `J-044`/`J-045` split to `a`/`b`, **bare numbers retired**, and **all 28 surviving citations individually re-pointed** — **Leg B-bis**
- [ ] Every ✅ node in the new tree carries `· J-nnn` — **Leg C**
- [ ] Every 🟡 / ⏸️ node carries a trigger **or an explicit `trigger: none`** — **Leg C**
- [ ] Every existing ⏸️ **POSTPONED** entry (**22** of them) **audited for a resume trigger** — **Leg C**
- [ ] No `→` arrow points at a journal entry that does not record the loose end — **Legs B + C**
- [ ] ROADMAP re-**measured** under 100 KB — **Leg C** 📌 *baseline is now 755,033 B (§1a), not 749,717*
- [ ] `CLAUDE.md`'s PLAY head **grounded and its format ruled** before any rewrite — **Leg D** (§6's rider)
- [ ] All nine of this session's threads present or explicitly declined — **Leg E**
- [ ] Archive deleted, or its retention **stated with a reason** — **Leg F**

---

## §10 — Filed, NOT fixed

- ⚠️ **`.claude/worktrees/` IS A STALE-CODE DECOY**, same class as the repo-local `target/`: **eight** copies of the tree, carrying an **old layout** (`TransportMessage` at `xgen-node/src/wire/types.rs` versus `xgen-core/` live). **Exclude `.claude` from every repo-wide search.**
- ⚠️ **A DEFERRAL WRITTEN AS A CODE COMMENT HAS NO OWNER AND NO TRIGGER.** The originating defect. **Proposal, Joe's to rule:** a deferral that outlives its milestone belongs in ROADMAP or DECISIONS, never in a comment.
- `DECISIONS.md` is **552,015 bytes / 136 entries**. Not obviously diseased — a decision record is *supposed* to accumulate — but unexamined here.
- **69 🟢 PLAY markers** is itself evidence that no convention forced PLAY to be exclusive. The new format does not enforce exclusivity either. **Filed, not proposed.**

---

## §11 — Handoff

✅ **RULED 2026-07-26 (delegated):** §6 — **BOTH** · §7 — **S1, the tree.**
✅ **LOCKED 2026-07-26 (a considered answer to a bounded question, NOT delegated):** **Leg C's scope** — keep the tree, link its nodes, delete the prose. **Runbook `tasks/RUNBOOK_ROADTREE_LEGC.md` v1.0.**

🟢 **LEG C IS OPEN AND IS CHAT'S TO RUN. Nothing below blocks it.**

🔓 **STILL OPEN FOR JOE — NONE OF IT GATING:**
1. **§4a-i** — the three-word amendment to §4b (*"at those numbers"* → *"labelled with those numbers, in a forward entry"*). Cheapest item on the board; **takes §8b off the critical path permanently.**
2. **§8b** — what `ARCHIVED` means when the archive is wrong. Chat proposes **R3**. ⚠️ **Still a real question** — 24 KB of duplication and the `J-044`/`J-045` collisions do not go away.
3. **§8a** — does `Leg B-bis` exist. Sequencing.
4. **§9's size bar** — 100 KB → 120 KB (Leg C §1).
5. **§1b's question for Leg D** — does `CLAUDE.md` also already contain a structure nobody measured?

⚠️ **THE AUDIT SURFACED SIX DEFECT INSTANCES IN ONE SESSION AND DID NOT CONVERGE.** The runbook's §0 bound and §7 reporting rule exist to stop that, **and they only work if honoured during the pass rather than merely written down.**

**Chat owes, carried:** the registry composition model (needs a live client ⇒ INTERACTIVE) · the Ch0–Ch2 thesis read (272,516 B across three chapters) · the six unclassified P1 lines `428 · 444 · 450 · 454 · 997 · 1023` (§1b) — **absorbed into Leg C Pass 1, not carried separately.**
