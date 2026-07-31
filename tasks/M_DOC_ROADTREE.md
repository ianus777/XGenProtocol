# M-DOC-ROADTREE — the roadmap becomes a state board
> **Status**: ACTIVE  
> Version: 1.32  
> Date: Jul 2026  
> **Last updated**: 2026-07-31  
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

### ✅ CONSEQUENCE FOR LEG C — SPENT (Leg C closed J-604; marker struck 2026-07-29, J-618)

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

✅ **SPENT — SUPERSEDED BY §4a BELOW; the framing here was wrong** (marker struck 2026-07-29, J-618). *Does set B get retrospective entries (the J-102/J-103 precedent), or is the ROADMAP prose kept in place?*

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

🔒 **CONSEQUENT AMENDMENT TO §4b — RULED 2026-07-29, DELEGATED. THREE WORDS.** §4b read *"migrate the ROADMAP prose into `JOURNAL` **at those numbers**"*. ⚠️ **That phrase is the only thing that forces `JOURNAL_ARCHIVE.md` open**, because those numbers fall inside its span — which is what §8b then ran into. **CHAT PROPOSED, AND JOE LOCKED BY RECOMMENDATION: *"labelled with those numbers, in a forward entry"*.** One new entry in `JOURNAL.md` carries the six surviving bodies, each labelled with the designation it was originally allocated, plus one line each for the four never written. **Applied in §4b above.**
- **① User-visible:** none, either way.
- **② Tier:** none.
- **③ Resource: one journal entry, against opening a frozen archive plus 28 citation re-points.** 🔑 **And the structural gain is the point, not the saving: §8b comes OFF the critical path.** It remains a real and unruled question — the 24 KB duplication and the `J-044`/`J-045` collisions still sit in the archive — but **nothing waits on it.**

⚠️ **Searchability is the one thing this must not cost, and it does not:** a reader looking for `J-124` finds the forward entry by the same text search that finds anything else. **What it does cost is chronological position** — the six bodies sit at the end of the journal rather than at their 2026-05 slot. 📌 **Stated as the trade it is; the provenance line carries the original date.**

🔑 **WHICH MAKES THIS MILESTONE THE SECOND HALF OF A TREATMENT THAT STARTED AT J-129.** J-129's own words: *"The chain was emergent prose that bled JOURNAL's job into the header line. Substantive narrative belongs in JOURNAL body entries."* **The Past section is the same prose bleeding into the same document's body.** Same defect, same three files, same cure — diagnosed by this project two months ago and half-treated. 📌 **Joe reached the diagnosis independently tonight without the J-129 text in view.**

### 🔒 §4b — RULING: MIGRATE, PLUS A PROJECT-WIDE DESIGNATION RULE (Joe, 2026-07-26)

🔒 **OPTION (a) — MIGRATE THE ROADMAP PROSE INTO `JOURNAL`, LABELLED WITH THOSE NUMBERS, IN A FORWARD ENTRY**, with honest provenance (*migrated from ROADMAP; originally a stripped header-chain entry*). The journal becomes genuinely complete — the premise the whole roadtree rests on — and the roadtree comes out pure.

🔒 **§4a-i's AMENDMENT IS APPLIED ABOVE — LOCKED 2026-07-29, PROVENANCE DELEGATED (Joe: *go by your last paragraph*, locking by Chat's recommendation without independently examining it; recorded as delegated, NOT as a walked lock).** The phrase *"at those numbers"* is struck and replaced. 🔑 **CONSEQUENCE, AND IT IS THE WHOLE REASON THE AMENDMENT EXISTS: §8b COMES OFF THE CRITICAL PATH.** *"At those numbers"* was the only thing forcing `JOURNAL_ARCHIVE.md` open for the migration ⇒ the migration row leaves §8b's table, and **nothing now waits on the `ARCHIVED`-versus-repair ruling.** ⚠️ **§8b IS NOT ANSWERED AND IS NOT WITHDRAWN** — the 24 KB of duplication and the `J-044`/`J-045` collisions still sit in the archive, both still need it ruled, and it is still 🔓 **JOE'S**. It simply no longer gates anything.

🔒 **AND THE DESIGNATION-COLLISION RULE (Joe, general, not migration-local): WHERE A DESIGNATION IS DUPLICATED, THE ORIGINAL TAKES AN `a` SUFFIX AND THE NEW ONE TAKES `b`** — e.g. `J-044a` / `J-044b`.

✅ **PROMOTED — `D-134` MINTED 2026-07-29 (J-619). MARKER STRUCK.** The rule is project-wide (`J-`/`D-`/`N-`/`M-`) and now lives in `DECISIONS.md`, with the `CLAUDE.md` block reduced to a signpost. 🔒 **Joe CONFIRMED it and AMENDED it in the same act:** the split extends past `a`/`b` to `c` and beyond, and **designations are ISSUED UNIQUE — the split is a repair applied at revision, never an issuing path.** 📌 **Rule ② (addendum, bare number survives) was verified against the record and carried into D-134 alongside ①**; both were measured 2026-07-29, ① at 7 of 7. ~~PROPOSAL: PROMOTE IT. The rule is project-wide — it governs J-, D-, N- and M- designations alike, not just this migration. It belongs in `CLAUDE.md`'s conventions or as a D-entry, not buried in a milestone task file. 🔓 Joe's.~~

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

🔑 **AND THE SCOPE RULING CARRIES ONE THING CHAT'S PROPOSAL EXPLICITLY DID NOT:** Leg D is in scope, but **§3's format is NOT assumed to be Leg D's format.** The two documents share a *disease* (append-only narration in a document meant to show current state); they do **not** demonstrably share a *cure*. ROADMAP is 586 closed nodes in a nested structure; the PLAY head is a **block sequence**, its worst offender a single 124,299-character line. ⚠️ **Applying a node format to a block document without measuring it first is exactly the "claim narrower than the thing it described, reused as if complete" defect.** ⇒ **Leg D opens with its own grounding pass, and its format is ✅ SPENT — the pass ran and Leg D closed at J-615 (marker struck 2026-07-29, J-618).**

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

⚠️ **ONE DoD NUMBER MOVES AND IT IS NOT FUDGED.** §9 requires ROADMAP *under 100 KB*; the surviving tree alone is **79,802 B** and the kept scaffolding brings the end state to **~113 KB**. ✅ **SPENT — Leg C landed at 43,741 B and beat the original 100 KB bar by more than half; the 120 KB amendment was WITHDRAWN at J-604** (marker struck 2026-07-29, J-618). ~~Chat proposes §9's bar move to `≤ 120 KB`~~ — the alternative is trimming *"How to use this view"* (12.9 KB) and *"Cross-cutting"* (12.4 KB), which are **instructions and standing context, not chronicle**, and cutting them to hit a round number would be **optimising the metric rather than the document.**

**Leg D — CLAUDE.md.** Same treatment on the PLAY head. **Surface: `CLAUDE.md`, `CLAUDE_HISTORY.md`.** ✅ **In scope — §6 ruled BOTH.** ✅ **SPENT — the grounding pass ran and Leg D closed at J-615** (marker struck 2026-07-29, J-618). ~~Its FORMAT is open~~ and opens with its own grounding pass (§6's rider): §3's node format is not assumed to transfer to a block document.

---

**Leg C — CLOSED (J-604).** ✅ `docs/ROADMAP.md` **761,422 → 43,741 B, 94.3% smaller**. The tree was kept and repaired, ten missing milestones grafted, `Past` and `Present` deleted, and five format rules R-1…R-5 written into the decode key with R-6 for link chains. ⚠️ **THE 120 KB AMENDMENT ABOVE IS WITHDRAWN** — the end state beat the original 100 KB bar by more than half. Runbook `tasks/RUNBOOK_ROADTREE_LEGC.md` v1.4 COMPLETED.

**Leg D — `CLAUDE.md` B2. ✅ CLOSED (J-615).** **640,645 → 316,680 B — 50.6%**, beating the §6b forecast of ~410,000 by 93,000. **65 of 81 blocks archived, 16 remain, 65 + 16 = 81 asserted.** `CLAUDE_HISTORY.md` 869,178 → 1,197,988 B, 185 → 186 `## ` headings — **one** heading written, every block byte-identical (F1c). **23 annotations** applied at the sites of stale claims, travelling with their blocks (D-131 form). ⚠️ **L29 SURVIVES AT 124,299 CHARS** — 39% of the remaining file, and Leg E's whole subject. Runbook `tasks/RUNBOOK_ROADTREE_LEGD.md` v1.3 COMPLETED · Phase-0 brief v1.6. Executed by `legd-move.ps1` + `legd-annotations.json`, dry-run first, refusing to write unless every assertion passed.

**Owes:** §E L75 — the M-RP-SKIN corner-triangle countdown has no home · §E L235 — M-RP-SELF-SURFACE has no task doc and no ROADMAP node.

🔒 **THREE RULINGS DELEGATED 2026-07-29 (Joe: *go as you recommend*), options authored by Chat.** **§F1c** — the archive uses `## ` headings and the head writes `> ###`; the batch lands verbatim **under one `## ` batch heading**, which is the only form that keeps D-094's *never a rewrite* while leaving the archive's structure conforming. **§F2b** — `CLAUDE_HISTORY.md` stays **ARCHIVED**, and the D-094 insert is written into its preamble as its **single sanctioned exception**; the vocabulary had contradicted itself since the file was created and only the lapse hid it. **§3a W1** — L195, L199, L201 archive annotated, because they declare 🟢 PLAY on closed legs exactly as the eleven mis-symbolled blocks do.

🔑 **IT IS NOT A FORMAT-INVENTION PROBLEM.** **D-094 already rules PLAY-block archiving**, `CLAUDE_HISTORY.md` already holds **185 archived blocks**, and the rule **lapsed 2026-06-22**. 81 blocks have accreted in the live head since, with a clean boundary — history J-81…J-405, head J-519…J-604 — and **zero duplication in either direction**. Census of the 81: **42 ✅ · 17 live (🟢🟡) · 22 non-work (🔒🔑🛑⚠️)**. ⚠️ **THIS IS A SYMBOL CENSUS AND NOTHING MORE — IT WAS READ AS A PARTITION AND THAT COST TWO REVISIONS (J-613).** *"17 live"* names the blocks that **lead** with 🟢🟡; **eleven of them are closed work.**

🔒 **§6a, §6b AND §6c ARE ALL RULED (§6b — S1, B2 only, J-611).** **§6a — E (Joe locked 2026-07-28, option authored by Chat):** classify each block by **what its own text says, not by its lead symbol**; blocks that self-declare closed archive with the 42, and the residue stays in the live head to be handled when work next reaches it — D-131 applied to blocks instead of citations. ⚠️ **R2 IS WITHDRAWN.** Reading the 22 showed the lead symbol marks the **finding inside** the block, not the block's state (L193 leads 🔒 and opens *Leg C ✅ CLOSED*), and that their designations — `N-118`, `N-120`, `N-124`, `N-124a`, `N-124b`, `D-122`, `D-123` — **already have canonical headings**. 📌 **The promotion-successor milestone may not exist as separate work; the residue looks like 4–6 blocks.** *[⚠️ SUPERSEDED — the measured residue is 14 blocks (J-611), and §A's read re-cuts it again (J-612).]* **§6c — D-131:** a citation proven broken is annotated at the site, never silently repaired. 🔒 **§6b RULED — S1 (delegated 2026-07-28): LEG D COVERS B2 ONLY.** B2 is 81 discrete `> ###` blocks with findable starts and ends; **B1 has no boundaries to move** — its bulk is L29, one line of 124,299 chars. ⚠️ **Stated cost:** Leg D takes `CLAUDE.md` **640,645 → ~410,000 B** and **L29 survives untouched**. Bounded and finishing, but it does not fix what made the file unreadable.
🛑 **NOTHING IS AUTHORISED FOR DELETION.** Phase-0 §7 lists four things **NOT MEASURED**, the first being that a resolving citation proves a record exists and **not** that the block's substance is in it.

🛑 **THE MOVE SET IS 62 BLOCKS — THIRD REVISION, ONE CAUSE (J-613).** Kickoff said 7 self-declared closed, Phase-0 §6a said 6, runbook v1.0 said a move set of 50. **Each classified a bucket it was handed rather than all 81 blocks.** Measured: **42 ✅ + 4 self-declared closed** (L177 L193 L251 L255) **+ 5 stale-closed** (L175 L241 L243 L245 L249) **+ 11 that the census called 🟢 live** (L123 L145 L151 L155 L161 L165 L167 L191 L219 L259 L261) **= 62**; **19 stay**; 62 + 19 = 81 ✔. 🔑 **THE DECISIVE EVIDENCE IS INSIDE `CLAUDE.md`:** every one of those eleven is a phase-0 or design-lock block whose milestone **this same head later closes** — L123 against L125, L259 against L189, L219 against L213. ⚠️ **THE 17 🟢🟡 BLOCKS HAD NEVER BEEN CLASSIFIED**, only counted as live by lead symbol, which is exactly what ruling E forbids. ✅ **SPENT — §3a W1 was RULED (delegated) 2026-07-29 and the move set closed at 65** (marker struck 2026-07-29, J-618). ~~L195 L199 L201 remain open under §3a (W1 recommended)~~; under W1 the set is 65. **2 held** for pointing at milestones with no record anywhere (L75 → M-RP-SKIN, L235 → M-RP-SELF-SURFACE). `tasks/RUNBOOK_ROADTREE_LEGD.md` §A is the authority.

**Leg E — the two-way closure log.** 🟢 **ACTIVE — P1 CLEARED (J-628); P2 OPEN.** 📌 *(this read `🟡 PENDING` until 2026-07-30; state refreshed at J-629 by the same sweep, **not** a re-decision.)* 🔒 **TITLE LOCKED (Joe, 2026-07-29, J-618** — a direct answer to a bounded question, **not** a delegation; chosen from three candidates that were themselves derived from the grounding measurement below, after Chat's earlier proposal *"the next-active chain"* **failed the test Joe set for it**). ↳ trigger: Leg D closes — **FIRED 2026-07-29 (J-615)**. **Surface: `CLAUDE.md`'s prose head — from the `## 🟢 UI component-library / substrate` heading down to the first `> ### ` PLAY block.** ⇒ **PARSING, NOT MOVING.**

✅ **§8-E P0 NOTATION CENSUS COMPLETE — RUN 2026-07-30 (J-623). `tasks/RUNBOOK_ROADTREE_LEGE.md` v1.0 🟡 PENDING WRITTEN (9,750 B, LF, no BOM). THE RUNBOOK IS THE AUTHORITY FROM HERE; THIS SECTION IS ITS PROVENANCE.**

🛑 **THIS CENSUS IS SUPERSEDED BY J-627 AND CLEARED BY J-628 — KEPT, NOT REPAIRED (annotation added 2026-07-30, J-629).** Its figures below read **`C 24`**, **`95 heads`**, **`91 records`**. ⚠️ **All three are superseded:** a **FOURTH head form** (`**<emoji> M-<id>` at 113,632) was invisible to `**M-`, and C's `24` had been measured from the **old** boundary 110,516 while the table's own span row already carried the re-derived **110,488**. ✅ **CORRECTED: A 1+19 · B 51 (4 stubs) · C 25 + 1 = 97 heads · 93 records**, and **C splits 14 closure-bearing / 12 not** (`CLOSED` × 11 + `DONE` × 3). ✅ **P1 re-ran against the corrected assertions and CLEARED (J-628).** 🛑 **THIS CORRECTION IS ITSELF SUPERSEDED (J-630), KEPT NOT REPAIRED: `97 / 93` → `94 / 90`, C `25 + 1` → `22 + 1 = 23`, split `14 / 12` → `14 / 9`, and P1's clearance was RETRACTED before being re-earned.** 🔑 **A CORRECTION DECAYS EXACTLY LIKE THE CLAIM IT CORRECTS.**

🔑 **WHY THIS ANNOTATION EXISTS AT ALL, AND IT IS THE SIXTH INSTANCE OF THIS ARC'S DEFECT CLASS.** J-627 corrected the runbook and §11 **and did not touch this section**; J-628 then corrected the runbook's §1a/§1b/§3.4 **and did not touch its §5/§6**. 📌 **Before this line was written, `J-627` and `J-628` appeared EXACTLY ONCE in this entire document — at §11.** ⚠️ **Every earlier instance was a claim narrower than the thing it described; these two are CORRECTIONS narrower than the claims they corrected — the same defect, applied to the repair rather than to the measurement.** ⇒ **A CORRECTION IS NOT APPLIED UNTIL THE WHOLE RECORD SET HAS BEEN SEARCHED FOR THE CLAIM BEING CORRECTED — NOT THE SECTION, NOT THE FILE.** 📌 **Dated records are exempt and must NOT be back-edited:** prior `CLAUDE.md` PLAY blocks and prior `JOURNAL.md` entries are contemporaneous accounts of what was known then. **The rule binds LIVE claims — checklists, grounding tables, task-doc state.**

🔑 **A THIRD HEAD FORM, AND IT CLOSED AN OFF-BY-ONE HONESTLY.** A showed **19** bold-milestone heads against **20** closure marks. The missing head is the line's **first record**, in neither known form: `**Next-active (UI/RP track):** M-RP2.6 ✅ CLOSED (J-410)` ⇒ **19 + 1 = 20 ✔**. 📌 **Recorded for HOW it closed:** by **finding the missing head**, not by filling a hole with arithmetic — the precise failure part three had just logged.

| region | span | size | head form | heads |
|---|---|---|---|---|
| **A** | 0 – 18,426 | 18,426 | `**Next-active (UI/RP track):**` *(once)* + `**M-<id>` | 1 + 19 |
| **B** | 18,426 – 110,488 | **92,062** | `J-nnn (` **at a `) / ` delimiter** | 51 *(4 stubs)* |
| **C** | 110,488 – 124,299 | **13,811** | `**M-<id>` | 24 *(11 closure-bearing)* |

🛑 **ROW `C` SUPERSEDED (J-627), KEPT NOT REPAIRED:** the head form column is incomplete — a **fourth form** `**<emoji> M-<id>` sits at 113,632 — and the count is **25 + 1 = 26**, splitting **14 closure-bearing / 12 not**. 🛑 **AND THAT REPLACEMENT IS ITSELF SUPERSEDED (J-630): three of the 25 are bold EMPHASIS, not heads ⇒ `22 + 1 = 23`, splitting `14 / 9`.**

**Sum 124,299 ✔ · 95 heads · 91 records · 11 known collisions.** 🛑 **`95 / 91` SUPERSEDED BY `97 / 93` (J-627) — SEE THE ANNOTATION AT THE HEAD OF THIS SECTION; KEPT, NOT REPAIRED.** 📌 **The B/C seam is 28 chars earlier than part two's 110,516** — C's first head opens at **110,488**, predicted by part three from one observation, confirmed here independently. 🔑 **AND THIS SUM IS A CONSEQUENCE, NOT AN INPUT** — boundaries came from independently located heads, so it *could* have failed. Part three's could not.

🛑 **THE LOAD-BEARING RESULT IS NEGATIVE: THERE IS NO UNIFORM RECORD HEAD IN THIS LINE.** `**M-` matches **48× inside B** as prose milestone-list mentions (five in one stretch, 41,259–41,394) · `J-nnn (` matches **52×** in B but only **50** are delimiters · `**Next-active` matches **8×** and exactly **1** is a head · `CLOSED (J-nnn)` cannot match B at all. ⇒ **EACH REGION NEEDS ITS OWN PREDICATE, AND EACH PREDICATE NEEDS A SECOND INDEPENDENT ONE TO CHECK IT.** ⚠️ **Four successive shapes for this line were each produced by trusting one predicate.**

📌 **A ONE-OFF SEAM MARKER** — `**Entry (Rule 0): this PLAY → JOURNAL ` immediately precedes B's first head (`J-503 (` at 18,426); unique in the line, filed as A's tail.

🔓→✅ **C's 13 NON-CLOSURE HEADS travel into the archive annotated as forward-looking entries** — **Chat's lean, ADOPTED BY DELEGATION at J-623, NOT a Joe lock, reversible.** Rationale: for several, this stretch of `CLAUDE.md` is the only place the postponement rationale exists.

🔑 **§8-E GROUNDING PASS — PART THREE RUN 2026-07-30 (J-622). THE ELEVEN COLLISIONS READ PAIR BY PAIR, AND PART TWO'S RECORD COUNT RETIRED.**

🛑 **THE ELEVEN SHARED KEYS ARE NOT ELEVEN DUPLICATES — CONTAINMENT RUNS IN BOTH DIRECTIONS.** Each of the eleven J-numbers that appears in both B and C was read on both sides in full:

| J-ref | milestone | B chars | C chars | verdict |
|---|---|---|---|---|
| J-446 | M-RP2.23 | **21** | 256 | **C only** — B is `J-446 (password-field`, a bare mention |
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

🔑 **NO MECHANICAL MERGE RULE SURVIVES THIS TABLE.** *Keep the longer copy* deletes J-446's runbook path and J-448's Shape-A detail. *Keep B* deletes J-454's `temperature-indicator` dd-block and the `ui/docs/`-at-open session rule, **which exist nowhere else in the repo.** *Keep C* deletes J-457's amber hex, J-459's deferred filter/search rationale, J-460's `box-sizing` evidence. ⇒ 🛑 **LEG E DELETES NOTHING WITHOUT A PER-PAIR HAND VERDICT.** 📌 Phase-0 §7's first caveat landing where it was written: **a resolving citation proves a record exists, not that its substance is in it.**

🛑 **AND PART TWO'S REGION-C COUNT IS RETRACTED. THIS IS THE FOURTH SHAPE FOR THE LINE AND IT BROKE J-621's OWN RULE INSIDE ONE TURN.**

🛑 **PART THREE'S OWN REGION-C COUNT IS IN TURN RETRACTED BY J-627 — KEPT, NOT REPAIRED (annotation added J-629).** The **`C 24 heads`** below is **superseded by `25 + 1 = 26`**: `**M-` could not see the fourth head form `**<emoji> M-<id>` at 113,632, and the 24 was measured from the **old** boundary 110,516. ⇒ **honest state is now `A 20 · B 47 records + 4 stubs · C 26 heads` = 97 heads, 93 records**, cleared by P1 at J-628. 📌 **THIS IS THE FIFTH SHAPE FOR THE LINE, AND THE FIRST ONE TO SURVIVE A RE-RUN.**

🛑 **THE FIFTH SHAPE DID NOT SURVIVE EITHER — SUPERSEDED BY THE SIXTH (J-630), KEPT NOT REPAIRED.** `C 26 heads` → **23**, `97 / 93` → **94 / 90**: three `**M-` matches in C are **bold emphasis mid-sentence, not record heads.** ⚠️ **The claim *the first one to survive a re-run* was written BEFORE the re-run that broke it** — 🔑 **a prediction recorded in the past tense.** ✅ **The SIXTH shape has since cleared P0-bis, P1 and P2 (J-630) with three independent cross-checks; it is stated as current, not as final.**
- **C holds 24 bold milestone heads, not 11.** The 11 was a count of `CLOSED (J-nnn)` matches — **one notation** — reported as a record count. J-474, J-478–J-487 and `M-RP-CDP1` sit in C and are **not** in the collision set; four heads (M-RP5.4, M-RP6.1, M-RP5.5, M-RP5.6) carry no closure at all and are **forward-looking entries, not records.**
- **B's 51 segments are not 51 records.** Four fall under 100 chars — `J-447 (star-rating` **18** · `J-446 (password-field` **21** · `J-461 (PROTO-STATUS.2 — Track A, see head note` **46** · J-448 **93**. Two are pointers; two are stubs whose real record is in C.
- ⇒ ⚠️ **`82` IS RETIRED AS A PLANNING NUMBER** — same class as the `28` planned against for three days (J-620). Honest state: **A 20** *(never re-tested under a second notation)* · **B 47 records + 4 stubs** · **C 24 heads.**
- ⚠️ **THE `18,426 + 92,090 + 13,783 = 124,299` PARTITION WAS TRUE BY CONSTRUCTION.** Part two's 10,968-char hole was closed with arithmetic, so the summation **could not fail**. The boundaries are still part two's, and C's first record disproves them: `**M-RP2.23` opens at **110,488**, twenty-eight chars **before** the boundary that supposedly starts C.

🔑 **FORWARD RULE, SHARPENED — J-621's WAS NECESSARY AND NOT SUFFICIENT.** *Enumerate the notations before counting* was written down and broken **within one turn**, because enumerating notations **inside a region a different predicate delimited** still inherits that predicate's boundaries. ⇒ **ENUMERATE THE NOTATIONS, THEN RE-DERIVE THE BOUNDARIES FROM THE ENUMERATION — NEVER COUNT INSIDE BOUNDARIES A DIFFERENT PREDICATE DREW.** 📌 **A summation that closes by construction is not verification; it is the shape of verification.**

🟡 **CONSEQUENCE FOR THE RUNBOOK — A P0 PRECEDES EVERYTHING.** **P0 notation census:** enumerate every record-head *form* in the line (bold-milestone-first · J-ref-first · stub · pointer), classify **all** heads, assert the count by summation over **char offsets derived from that census**. **P1** extraction. **P2** collision resolution — **eleven known; the remaining ~60 records have never been tested for cross-region twins.** **P3** the D-094 batch move. **D-121:** ① user-visible impact **none** on any branch — no runtime reads `CLAUDE.md` · ② resource cost — P0 one probe session, **P2 is the real cost**, eleven hand-diffs consumed a session and scale linearly · ③ elegance tertiary.

🔑 **§8-E GROUNDING PASS — PART TWO RUN 2026-07-30 (J-621). 🛑 ITS RECORD COUNT IS SUPERSEDED BY PART THREE ABOVE (J-622) — KEPT, NOT REPAIRED.** Its **notations** finding (two inverted forms; B delimited on `) / J-nnn`) stands, and part three is built on it; its **counts** — ≈ 82 records, C1 4 + C2 7, *every region delimited* — do not. **IT CORRECTED PART ONE, AND PART ONE HAD ALREADY REACHED THE ROADMAP AND A LOCKED NAME.**

🛑 **THE LINE IS ONE CLOSURE LOG END TO END, IN TWO INVERTED NOTATIONS. PART ONE'S "TWO LOGS WITH A NARRATIVE HOLE" WAS AN ARTEFACT OF THE PREDICATE.**

| region | chars | share | records | notation | J-order |
|---|---|---|---|---|---|
| **A** | 0 – 18,426 | 14.8% | **20** | `**M-RPx ✅ CLOSED (J-nnn):**` — milestone first | **strictly ascending** J-410 → J-435 (19/19 monotonic) |
| **B** | 18,426 – 110,516 | **74.1%** | **51** | `J-nnn (M-RPx … ✅ CLOSED/DONE …)` — **J-ref FIRST** | mostly descending |
| **C1** | 110,516 – 111,445 | — | **4** | milestone first | **ascending** J-446 → J-469 |
| **C2** | 122,026 – 123,912 | — | **7** | milestone first | **strictly descending** J-460 → J-454 |

✅ **≈ 82 RECORDS TOTAL. 61 closure marks in the line.** ✅ **EVERY REGION HAS A DELIMITER:** A and C on the `✅ CLOSED (J-nnn)` marker; **B on `) / J-nnn`, which splits it into 51 segments covering 91,890 chars + exactly 200 delimiter chars = 92,090 — a COMPLETE partition, verified by summation.** Segment lengths 18 → 7,795; 41 of 51 under 2,000.

🛑 **AND THIS IS THE THIRD SHAPE CHAT REPORTED FOR THIS ONE LINE. THE FAILURE IS IDENTICAL EACH TIME: PICK ONE NOTATION, COUNT IT, REPORT THE SHAPE IT IMPLIES.**
1. **"A next-active chain, appended 11 times"** — from counting `Next-active` strings. **Killed by Joe's own test:** only 2 of the 11 are structural.
2. **"Two closure logs running in opposite directions"** — from counting J-reference *ordering*. **Reached `docs/ROADMAP.md`, this document, a PLAY block AND the name Joe locked.**
3. **"Two logs with a 93,205-char narrative hole"** — from counting `✅ CLOSED\s*\(J-\d+\)`, a predicate **B cannot match because B puts the J-ref FIRST.** B holds 5 `✅ CLOSED`, 5 `✅ DONE` and 21 checkmarks; the regex saw zero and 74% of the line was declared *not a log*.

🔑 **FORWARD RULE, AND IT IS NOT THE USUAL ONE.** *"Open the thing"* was followed all three times — the line WAS opened and sampled. ⚠️ **What was skipped is prior: ENUMERATE THE NOTATIONS BEFORE COUNTING ANY OF THEM.** A predicate built from one notation reports the absence of the others as the absence of the thing. 📌 **Sampling did not catch it either** — the seven samples of region B were read as *design narrative* because each was mid-record, and a mid-record sample of a `J-nnn (…)` log looks exactly like prose.

✅ **JOE'S LOCKED TITLE SURVIVES AND IS NOW BETTER SUPPORTED THAN WHEN HE CHOSE IT.** *The two-way closure log* — it **is** a closure log end to end, and it genuinely runs **two ways** (ascending in A and C1, descending in B and C2). 📌 **The title was locked on part one's reading; part two disproved that reading and the title still holds. Recorded because it would be easy to present this as vindication of the measurement rather than luck.**

🛑 **`Leg E-bis` WAS RECOMMENDED AND IS WITHDRAWN BEFORE IT EXISTED.** Chat proposed narrowing Leg E to A + C and spawning `Leg E-bis` for B, on the ground that **B had no delimiter.** ⚠️ **It has one, found by the next probe.** ⇒ **Leg E covers ALL THREE regions, ≈ 82 records. No new leg. `Leg E-bis` was never minted and takes no designation.**

⚠️ **STILL UNMEASURED, AND NAMED SO IT IS NOT ASSUMED CLOSED:** whether the ≈ 82 records are **distinct** (no designation appears twice — the `J-317` lesson applies to this line too) · what the **10 B-segments over 3,000 chars** contain, since a 7,795-char record may itself be several · whether A's and C's marker counts are complete under a notation-agnostic predicate rather than the milestone-first one.

🛑 **⚠️ THE LINE MOVED ON THE DAY THIS WAS WRITTEN, AND THE POSITION MUST NOT BE TRUSTED. USE THE CONTENT HANDLE.** At J-619 the five-line designation-convention block above it was collapsed to one line, so **`L29` → `L25` and the surface `L21–70` → `L17–66`** — the line itself byte-identical and untouched by git. 🔑 **A LINE NUMBER INTO `CLAUDE.md` IS A FRAGILE CITATION PRECISELY BECAUSE THIS PROJECT DRAINS THAT FILE** — J-615 archived 65 blocks out of it, and any edit above the line moves it again. ⇒ **IDENTIFY IT BY CONTENT, NOT POSITION:** it is the **only line in `CLAUDE.md` over 20,000 chars**, it is **124,299 chars**, and it opens `**Next-active (UI/RP track):** M-RP2.6 ✅ CLOSED (J-410)`. 📌 **Every `L29` in this document, in `RUNBOOK_ROADTREE_LEGD.md` and in the LEG D brief was written when the number was true; they are NOT re-pointed** — the handle above supersedes them. ⚠️ **`M_DOC_ROADTREE_LEGD_PHASE0_BRIEF.md` still reads `Status: ACTIVE` although Leg D closed at J-615; flagged, not flipped.**

🔑 **§8-E GROUNDING PASS — PART ONE RUN 2026-07-29 (J-618). 🛑 SUPERSEDED BY PART TWO ABOVE (J-621) — KEPT, NOT REPAIRED.** Its measurements of *position* (the seam at char 18,426, the region shares, the J-order runs) are correct and still used. ⚠️ **Its INTERPRETATION — that the line is two logs with a narrative hole — is disproved:** the predicate `✅ CLOSED\s*\(J-\d+\)` cannot match region B, which puts the J-ref first. 📌 **Kept because it is the record of how the wrong shape was reached, and because part one's own numbers are what part two built on.**

| region of L29 | share | what it is |
|---|---|---|
| chars **0 – 18,426** | 14.8% | a closed-milestone log running **OLDEST-FIRST** — J-410 climbing to J-435 |
| char **18,426** | — | a single `Entry (Rule 0): this PLAY → JOURNAL J-503` pointer. **Occurs EXACTLY ONCE in the whole line.** The seam |
| chars **18,426 – 124,299** | 85.2% | a second closed-milestone log running **NEWEST-FIRST** — J-503 falling to J-454 |

**Whole-line content:** 141 `J-` references (**79 distinct**) · 197 `M-RP` milestone mentions · 61 ✅ · 31 `CLOSED (J-…)`.

🛑 **AND THE MEASUREMENT KILLED CHAT'S PROPOSED NAME, WHICH IS WHY IT WAS RUN FIRST.** Chat proposed *"the next-active chain"* on a count of **11** `Next-active` strings. Opened: **only 2 of the 11 are structural heads; the other 9 sit mid-sentence.** The line is not a pointer appended to eleven times — it is **a log of closed milestones**, and the name described a feature incidental to it. 🔑 **Fifth instance this session of a claim narrower than the thing it describes, and it fell to opening rather than counting.**

✅ **GOOD NEWS FOR THE LEG: the seam is a SINGLE FINDABLE POINT, not a gradient** — char 18,426, marked by a string that occurs once.
⚠️ **BAD NEWS: THE PARSE MUST HANDLE A DIRECTION REVERSAL.** Anything written assuming one ordering **silently mis-orders 85% or 15% of the output** depending on which half it was tested against. 📌 **A parser tested on one half is a probe that cannot fail on the other.**
📌 **STILL UNMEASURED (part two of the grounding pass):** where one *statement* ends and the next begins **within** each half. The seam is found; the internal boundaries are not.

🛑 **AND THE LETTER `E` WAS ISSUED TWICE — FOUND AND RESOLVED 2026-07-29 (J-617), DELEGATED.** When §6b created *Leg E — the B1 prose* at J-611, §8b's tail already held **`Leg E` — the bidirectional sweep** and **`Leg F` — records + close**, ninety lines below in this same file. **One document, one leg list, two unrelated `Leg E`s** — and the second pair was never carried into `docs/ROADMAP.md` at all, so the milestone's own **closing leg had no node on the board.** 🔑 **THE MILESTONE COULD NOT HAVE CLOSED WITHOUT SOMEONE NOTICING, WHICH IS THE ONLY REASON IT WAS SURVIVABLE.** 🔒 **RESOLVED BY RENUMBER, NOT BY SUFFIX:** the B1-prose leg **keeps `E`** — it is the one `ROADMAP.md`, `RUNBOOK_ROADTREE_LEGD.md`, the Phase-0 brief and five JOURNAL entries already cite — the sweep becomes **`F`**, the close becomes **`G`**. 📌 **The `a`/`b` collision rule (§4b) deliberately does NOT apply here:** it governs **designations** (`J-`, `D-`, `N-`), which are permanent identifiers whose citations must never silently re-point. **Leg letters are positional within one document**, `Leg G` was free repo-wide, and the whole re-point cost was **four lines in this file** — measured by classifying all sixteen repo-wide `Leg E` hits by referent before touching one. ⚠️ **JOURNAL entries citing `Leg E` are contemporaneous records and were NOT rewritten** — they describe the state at the time they were written and remain true.

📌 **LEG D SHIPS NO PLAY BLOCK (P2, delegated 2026-07-28).** D-074's `CLAUDE.md` limb is deliberately empty for this milestone: adding block 82 to the head Leg D exists to drain would grow the defect it is closing. Recorded so the omission is never read as drift.
### ✅ §8a — THREE PIECES OF RULED WORK HAVE NO LEG, AND ONE OF THEM CONTRADICTS THE LEG IT WOULD FALL INTO. ✅ ANSWERED AS AN OUTPUT OF §8b (2026-07-29, J-618) — NOT A SEPARATE RULING

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
> 1. **Migrate** the **six surviving set-B bodies** into `JOURNAL` (§4a-i — `J-123 · J-124 · J-125 · J-131 · J-132 · J-135`), each carrying the provenance line *“migrated from ROADMAP; originally a stripped header-chain entry (J-129 §8)”*, **plus one line each recording `J-067 · J-098 · J-109 · J-113` as cited-but-never-written.** ⚠️ **Do NOT reconstruct a body for those four from their citations.** ✅ **SPENT — §4a-i's amendment was LOCKED (delegated) 2026-07-29: the migration is a FORWARD ENTRY in `JOURNAL.md` and this leg does NOT touch `JOURNAL_ARCHIVE.md` for it** (marker struck 2026-07-29, J-618). ~~Whether this lands in the archive at those numbers or as a forward entry labelled with them is §4a-i's open amendment — and it decides whether this leg touches `JOURNAL_ARCHIVE.md` at all.~~
> 2. **Delete** one copy of each byte-identical `J-317–J-321` block. ⚠️ **Verify byte-identity immediately before deleting, not from this document's record of it** — a duplicate re-verified is cheap; a wrong delete is unrecoverable outside git.
> 3. **Split** `J-044` → `J-044a`/`J-044b` and `J-045` → `J-045a`/`J-045b`, **retiring both bare numbers** (normative, per the discriminator rule). ⚠️ **`J-045`'s assignment rests on FILE ORDER ALONE** — §4c states this and it must survive into the entry text, not just this task file.
> ⚠️ **Every existing citation of `J-044` / `J-045` must be re-pointed in the same commit** — ROADMAP, CLAUDE.md, DECISIONS.md, other task files. **A retired bare number left cited is the exact silent-mis-point failure the convention forbids.** ➕ **Unmeasured: how many such citations exist.** Chat owes that count before the leg is runbooked.

- **① User-visible:** none, either way.
- **② Tier:** none.
- **③ Resource:** the three actions share one surface, one verification method and one commit ⇒ **cheaper as one leg than distributed across B and C.** Splitting them would also put a **deletion** inside a leg that forbids deletion.

✅ **OUTPUT OF §8b, NOT AN INPUT (2026-07-29, J-618).** §8b ruled the archive **may** be edited ⇒ **`Leg B-bis` EXISTS**; §4a-i moved the migration to a forward entry ⇒ **two actions, not three**; and there is only one place left in the sequence ⇒ **the ordering was never free.** 📌 Chat's proposal below stands as written except that **row 1 is spent**. ~~OPEN — order of work is Joe's (D-123). Chat can write it; Chat does not get to insert a leg into the sequence.~~

---

### 🛑 §8b — AND MEASURING THAT LEG'S SURFACE FOUND SOMETHING THAT OUTRANKS IT: **EVERY PIECE OF §4's RULED RETROACTIVE WORK TARGETS A FILE THAT DECLARES ITSELF IMMUTABLE.** MEASURED 2026-07-26 AT `ec0d305`

🔑 **`JOURNAL_ARCHIVE.md`'s OWN HEADER, QUOTED:**

> `> **Status:** ARCHIVED`
> *“This document is the frozen archive of older XGen Protocol development-journal entries… **Entries are verbatim and unaltered; do not modify.**”*

**And Joe's own status taxonomy: `ARCHIVED — Frozen historical record, do not modify.`**

**WHERE THE WORK ACTUALLY LANDS — enumerated, not inferred:**

| Ruled action | Target numbers | File that holds them |
|---|---|---|
| §4b **migrate** set B | `J-067 · J-098 · J-109 · J-113 · J-123 · J-124 · J-125 · J-131 · J-132 · J-135` | ✅ **ROW VOID 2026-07-29** — §4a-i's amendment is LOCKED and applied in §4b; the migration is a forward entry in `JOURNAL.md` and **does not touch `JOURNAL_ARCHIVE.md` at all** |
| §4c **delete** the duplicates | `J-317 · J-318 · J-319 · J-320 · J-321` | ⚠️ **`JOURNAL_ARCHIVE.md`, both copies** — L1208/L1228 · L1248/L1270 · L1292/L1312 · L1332/L1352 · L1372/L1400 |
| §4c **split** the collisions | `J-044 · J-045` | ⚠️ **`JOURNAL_ARCHIVE.md`** — L16957/L17057 · L17128/L17161 |

📌 **`JOURNAL.md` is clean: zero duplicate headings, zero set-B numbers.** The entire retroactive workload sits in the one file nobody is allowed to touch. **⇒ Leg B-bis as §8a proposes it cannot be executed as written.**

🔑 **AND THE DUPLICATION MECHANISM IS NOT WHAT §4c ASSUMED.** §4c reads *“a copy-paste or merge accident”*, which implies a contiguous five-entry block pasted twice. **The measured line order is `321,321,320,320,319,319,318,318,317,317` — each entry doubled IN PLACE, not the block repeated.** ⚠️ **A block paste would read `321,320,319,318,317,321,320,319,318,317`. It does not.** ⇒ **The cause was per-entry, most likely a write that emitted each entry twice** — stated as the better-supported reading, **not** as established. 📌 **The remedy is unchanged (delete one copy of each); the record of WHY is corrected.**

➕ **CITATION LOAD FOR THE `a`/`b` SPLIT — MEASURED, DISCHARGING §8a's OWED COUNT.** Repo-wide `.md`, `.claude` excluded:

🛑 **SUPERSEDED 2026-07-29 (J-620) — THE TABLE BELOW COUNTS *REFERENCES*, NOT *CITATIONS*, AND IT WAS READ AS A CITATION COUNT FOR THREE DAYS.** It never separated a **citation** (a pointer at the entry, which must be re-pointed) from **discussion of the collision itself** (prose about the defect, which must not be touched). Re-run over 507 `.md` files with a bounded predicate and **read per hit: 87 bare hits = 4 definition sites · 71 discussion · 12 CITATIONS.** ⇒ **the operative figure is 12**, tabulated per hit with its resolved target in `RUNBOOK_ROADTREE_LEGB_BIS.md` §2. 📌 **The table is kept, not repaired — it was an honest reference count and it is what caught the second measurement's own error** (its per-file breakdown named `CLAUDE.md` and `ROADMAP_ARCHIVE`, which a keyword classifier had wrongly emptied).

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

**CHAT PROPOSED R3.** ⚠️ **NOT TAKEN. See the ruling below.**

🔒 **§8b RULED — R2, WITH JOE'S OWN BOUNDARY (Joe, 2026-07-29, J-618). A CONSIDERED ANSWER TO A BOUNDED QUESTION, NOT A DELEGATION.**

> **`ARCHIVED` means NO NEW RECORDS. Correction of an existing record is permitted whenever a defect is found.**
> — Joe: *"correctly 'no new records', but we cannot know if there are another, now not detected errors, which we discover in the future. so i would say 'no new records, but correctable each time it needs it'."*

🔑 **JOE'S REASON IS STRONGER THAN THE ONE CHAT WROTE FOR R2, AND IT DEFEATS R3 ON ITS OWN TERMS.** Chat's R2 entry offered only *same cost as R3, plus precedent risk*. Joe's argument is the real case: **R3 assumes repair is a ONE-TIME EVENT, and nothing supports that assumption.** The seven collisions surfaced only because somebody counted headings; **nothing else in a 2,333,803-byte file has been counted.** ⚠️ **Under R3 every future discovery becomes a fresh ceremony, and that friction pushes a finder toward NOT REPORTING what they find** — the opposite of what an archive is for. 📌 **Chat's R3 recommendation is WITHDRAWN, not overruled on taste.**

⚠️ **THE CLAUSE BELOW IS CHAT'S, NOT JOE'S — PROPOSED IN THE SAME TURN AND NOT SEPARATELY CONFIRMED. IT IS REVERSIBLE AND SHOULD BE READ AS A DRAFT OF JOE'S RULING, NOT AS PART OF IT.** It exists because Chat named *"whoever is editing decides what counts as a repair"* as R2's cost; **Joe's *no new records* already closes most of that, and the clause only makes the boundary explicit.**

- **PERMITTED:** removing an exact duplicate · splitting a designation that names two events · correcting the file's own header metadata
- **FORBIDDEN:** adding, removing or rewording the substance of any entry
- **REQUIRED:** every repair leaves a dated `Repaired:` line naming what changed — 🔑 **this keeps the one genuinely good property R3 had (each repair is visible and dated) while granting the standing permission Joe actually ruled for**

✅ **CONSEQUENCE — §8a IS ANSWERED AS AN OUTPUT, NOT AS A SEPARATE QUESTION.** The archive may be edited ⇒ **Leg B-bis EXISTS**, with **two** actions (§4a-i having moved the migration to a forward entry), **one surface**, and only one place left in the sequence. 📌 **§8a needs no separate ruling.**

📌 **B-bis ALSO INHERITS THE HEADER FIX**, now legal under the ruling: the archive's own header still says *"Live window (J-395 … J-376) continues in `JOURNAL.md`"*, and `JOURNAL.md` holds **242 entries, J-617 … J-376** (measured 2026-07-29). Under R1 that sentence was **unfixable by its own rule**; it is now simply a repair.

📌 **A smaller one riding along:** the archive's own header says *“Live window (J-395 … J-376) continues in `JOURNAL.md`”* — **`JOURNAL.md` now runs J-376 … J-599.** The header is stale **and unfixable under its own rule**, which is the same collision in miniature.

---

**Leg F — the bidirectional sweep.** ⚠️ **Not only *is every roadmap entry true*, but *is every known work item ON the roadmap*.** **Surface: this session's open threads** — M-RP-LIVEFEED-REFRESH · the resync sibling · the outbox · H1 · H2 · D-130 · the address-book eviction question · `NegotiatedCapabilities` · the Ch0–Ch2 thesis read. **They exist in chat and in one task document and nowhere else.** They are the natural test case for whether the sweep works.

**Leg G — records + close.** JOURNAL + CLAUDE.md PLAY + ROADMAP + this document, **one commit** (D-074). Delete `ROADMAP_ARCHIVE_2026-07-26.md` **iff** Leg B cleared.

---

## §9 — DoD

Applying `M_RP_MEMBERS.md` §8b's rule — **every item naming an action names its surface in §8, and every 🔒 in this document has a leg that builds it:**

- [ ] M-RP-MEMBERS Leg C reads ⏸️ **with its trigger** in ROADMAP, CLAUDE.md and the board — **Leg A**
- [ ] All **94** P1 items **measured**: journal link found, or redundancy confirmed in writing — **Leg B**
- [ ] All **11** P2 refs **measured**: entry located, or absence recorded — **Leg B**
- [ ] The **`ARCHIVED`-versus-repair question (§8b) ruled** by Joe, and the ruling recorded where a future reader of `JOURNAL_ARCHIVE.md` will find it — **precedes Leg B-bis**
- [ ] The **six surviving set-B bodies present in a journal file**, each carrying its provenance line, **and the four never-written numbers recorded as such** (§4a-i) — **Leg B-bis**
- [x] `J-317`–`J-321` **re-verified byte-identical, then reduced to one copy each**, count re-measured after — **Leg B-bis ✅ J-625** (census **358/351 → 353/353, EQUAL**; −**25,614 B** measured — 📌 **the runbook's `25,127 B` was a CHARACTER count**; survivors byte-identical to `6863702`)
- [x] `J-044`/`J-045` split to `a`/`b`, **bare numbers retired**, and **all 12 surviving citations individually re-pointed** (📌 **not 28** — a reference count, superseded J-620; 11 actioned, `ROADMAP_ARCHIVE:348` held under §4's lock) — **Leg B-bis ✅ J-625**
- [ ] Every ✅ node in the new tree carries `· J-nnn` — **Leg C**
- [ ] Every 🟡 / ⏸️ node carries a trigger **or an explicit `trigger: none`** — **Leg C**
- [ ] Every existing ⏸️ **POSTPONED** entry (**22** of them) **audited for a resume trigger** — **Leg C**
- [ ] No `→` arrow points at a journal entry that does not record the loose end — **Legs B + C**
- [ ] ROADMAP re-**measured** under 100 KB — **Leg C** 📌 *baseline is now 755,033 B (§1a), not 749,717*
- [ ] `CLAUDE.md`'s PLAY head **grounded and its format ruled** before any rewrite — **Leg D** (§6's rider)
- [ ] All nine of this session's threads present or explicitly declined — **Leg F**
- [ ] Archive deleted, or its retention **stated with a reason** — **Leg G**

---

## §10 — Filed, NOT fixed

- ⚠️ **`.claude/worktrees/` IS A STALE-CODE DECOY**, same class as the repo-local `target/`: **eight** copies of the tree, carrying an **old layout** (`TransportMessage` at `xgen-node/src/wire/types.rs` versus `xgen-core/` live). **Exclude `.claude` from every repo-wide search.**
- ⚠️ **A DEFERRAL WRITTEN AS A CODE COMMENT HAS NO OWNER AND NO TRIGGER.** The originating defect. **Proposal, Joe's to rule:** a deferral that outlives its milestone belongs in ROADMAP or DECISIONS, never in a comment.
- `DECISIONS.md` is **552,015 bytes / 136 entries**. Not obviously diseased — a decision record is *supposed* to accumulate — but unexamined here.
- **69 🟢 PLAY markers** is itself evidence that no convention forced PLAY to be exclusive. The new format does not enforce exclusivity either. **Filed, not proposed.**

---

## §11 — Handoff

⚠️ **REFRESHED 2026-07-29 (J-618). SEVEN DEAD OPEN-LOCK MARKERS WERE STRUCK IN THIS PASS** — they sat on questions closed by Legs C and D, by §4a-i and by §3a W1, and **none had ever been unmarked.** 🔑 **That was the whole of Joe's stated orientation problem: the document read as if a dozen things were open when three were.**

✅ **CLOSED:** Leg 0 · Leg A · **Leg C (J-604** — 761,422 → 43,741 B**)** · **Leg D (J-615** — 640,645 → 316,680 B, 65 of 81 blocks archived**)**.
🟡 **Leg B — P1 cleared (J-599), P2 measured and NOT cleared.** Cleared by B-bis.
🟢 **Leg B-bis — the journal repair. ✅ EXECUTED, VERIFIED AND CLOSED 2026-07-30 (J-625/J-626).** Runbook v1.3 COMPLETED. ✅ **Census 358/351 → 353/353 EQUAL** · `J-317`–`J-321` one copy each deleted (**−25,614 B measured** — 📌 the runbook's `25,127 B` was a CHARACTER count) · survivors byte-identical to `6863702` · `J-044`/`J-045` split · **11 citations re-pointed individually**, `:348` held · `Repaired:` line written · header corrected (span low end read `J-046`; the archive reaches **`J-001`**). 🛑 **The `a`/`b` assignment was settled by ARTEFACT EVIDENCE after a mid-§1b HALT** — `D-134` §2's *higher line = earlier in a newest-first file* is **FALSE for this file**: newest-first only to ~L14898, then an original oldest-first block `J-001`→`J-047`. **Conclusion unchanged, justification replaced.** 📌 **Its two unmet DoD items were NOT dropped — they are re-homed to Leg B-ter** (J-626, Joe's lock).

🟡 **Leg B-ter — the eleven that resolve nowhere.** 🔒 **SPAWNED AND TITLED 2026-07-30 (Joe, J-626)** from B-bis's DoD gap: §6 demanded P2 cleared and the six set-B bodies migrated, **and no procedure section described either.** **Scope — the eleven P2 numbers, in three classes that are NOT the same kind of thing:** ① **six bodies to migrate** — `J-135 J-132 J-131 J-125 J-124 J-123` ② **four cited-only, no record anywhere** — `J-113 J-109 J-098` (**45 references and no record of its own**) `J-067` ③ **`J-029`, disposition genuinely open** — P2 counts it, §4a-i's table does not cover it, it falls inside set A. 🛑 **SOLE SOURCE: `docs/ROADMAP_ARCHIVE_2026-07-26.md`.** ⚠️ **Leg C DELETED all six bodies from `ROADMAP.md`** (refs fell 7→2 · 5→2 · 10→1 · 27→1 · 24→1 · 15→1); own bodies survive **only** at archive L512 · L524 · L528 · L548 · L552 · L556. 🔑 **§4a required that this prose NOT be deleted until the journal carried it. It was deleted first.** ⇒ 🛑 **HARD ORDERING CONSTRAINT: LEG G MUST NOT RUN BEFORE LEG B-TER.** Leg G deletes that archive iff Leg B clears — **had it run, six bodies would be gone permanently.** 📌 **No `Owes:` line is issued: `D-133` says leg runbooks never get one, and an ACTIVE parent whose own milestone is still in flight does not need one.** The debt is carried by this leg list and by Leg G's constraint. **Not runbooked yet; it is AUTHORING, not repair.**
🟡 **Leg E — the two-way closure log. 📘 RUNBOOKED (J-623): `tasks/RUNBOOK_ROADTREE_LEGE.md` v1.5 🟢 ACTIVE — THE AUTHORITY FROM HERE.** 🔒 Title locked (Joe, J-618). Trigger **FIRED (J-615)**. 🛑 **P1's CLEARANCE IS RETRACTED (J-629). THE HEAD SET IS `A 20 · B 51 · C 23 = 94 HEADS · 90 RECORDS`** — superseding *97 / 93* (J-628) and *95 / 91* (J-627), **annotated never erased.** 🛑 **THREE `**M-` MATCHES WERE BOLD EMPHASIS, NOT HEADS:** @115,900 `**M-RP5.5**` and @115,973 `**M-RP5.6**` inside **one clause** of the record at 115,464, and @119,871 `**M-RP5.6 CLOSED**` after an **em-dash inside J-485's own sentence**. 🔑 **EVERY §3 GATE PASSED WHILE THIS WAS TRUE** — §3.3 compares the predicate's count to a census figure **produced by that same predicate**, so it detects drift and never error; the same disease as §3.4's span-sum, **one line above it**, struck at J-628 while this survived. ⇒ 🔒 **`D-135` MINTED (delegated, reversible): a predicate is tested in BOTH directions, and an assertion built from the predicate it checks cannot fail.** ✅ **LOCKED BY JOE 2026-07-31 (J-631) — no longer reversible; the *delegated* wording above describes its state on 2026-07-30 and is kept, not erased.** ✅ **THE SECOND PREDICATE IS THE WHOLE DIFFERENCE:** A had one (20 `CLOSED` marks vs 1+19) → **0 false heads**; B had one (J-keys **strictly monotonic J-503→J-445**, which also forbids within-B twins) → **0 false heads**; **C had none → all three.** 🛑 **§4a's ELEVEN COLLISIONS ARE SIXTEEN** — **J-478 · J-479 · J-480 · J-483 · J-485** added. **The eleven were measured inside part two's two C sub-windows (110,516–111,445 and 122,026–123,912 = exactly 11); the 10,581 chars between them were never examined and all five sit in that gap.** ⚠️ **Part three positively asserted those J-numbers were NOT in the collision set — false.** 🛑 **AND THE PARSE ERROR WAS HIDING A REVERSED VERDICT:** C's J-485 read **93 chars** (a stub ⇒ *keep B*) while the false head stood; whole it is **1,813** and **richer than B's 906** — **the verdict inverts.** ✅ **§4c COMPLETE: zero collisions within-A, within-B, within-C, A↔B, A↔C**, structurally explained (A spans J-410–J-435, B spans J-445–J-503). ⚠️ **Key extraction is UNSOUND for C's 4 forward-looking heads** (first `(J-nnn)` there is a citation, not an identity) — no false collision resulted, all 16 were read on both sides, **but it must not be reused blind.** ⚠️ **One hand verdict flagged as a judgement:** @116,587 opens after a **semicolon**, admitted on head form. 🛑 **P2 CANNOT RUN.** 📌 *(True as written at J-629; **LIFTED by P0-bis + P1 at J-630**. Kept, not erased — the block was real and the reason it lifted is the record.)* ✅ **P0-BIS, P1 AND P2 ALL CLEARED 2026-07-31 (J-630).** **P0-bis** closed the one direction `D-135` still owed — a **closure-mark bijection** independent of every head predicate: **49 marks line-wide, 49 spans holding exactly one each, ZERO spans with two** ⇒ no missed head brings a closure with it; and the longest zero-mark spans read clean (their only interior `**M-` hits are bold emphasis **in B**, where the `) / ` anchor is structural — **direct evidence for `D-135`'s structural-vs-typographic claim**). **P1** re-ran under corrected §3: **1 line · 124,299 · A 1+19 · B 51 · C 22+1 = 94 · 4 stubs · 90 records**, open **and** close assertions both green, and **4(d) fired exactly once — on @116,587, the ALREADY-RECORDED hand verdict and nothing else.** 📌 **First time in this arc a gate fired on something already understood rather than on something missed.** 🛑 **P2 COMPLETE — ALL 16 PAIRS RE-READ ON BOTH SIDES, AND TWO INHERITED VERDICTS WERE WRONG.** ⇒ **11 B ⊃ C · 3 C-side · 2 DIVERGENT**, superseding *8 · 2 · 1 over eleven* (J-622). 🛑 **J-456 `M-RP4.5` WAS RECORDED `B ⊃ C` AND IS DIVERGENT:** C alone names **`transform.ts`**, **`toEditable`**, *“nothing on `input`”*, `string↔T` and *“3 of 4 processor kinds”*; B alone carries the *two-reps-cannot-ride-one-`bind:value`* rationale, `formatToParts` and `empty=revert`. **A *keep B* resolution would have deleted the only place `transform.ts` and `toEditable` are named.** ⚠️ **J-448 refined: C richer, but B alone carries `CDP-verified`.** ✅ **All FOUR of §4b's named risks confirmed by re-reading:** J-457's hex **`#ba7517`** · J-459's deferred filter/search rationale · J-460's `box-sizing` evidence — all **B-only**; J-454's `temperature-indicator` dd-block and the `ui/docs/`-at-open rule — **C-only**. ✅ **The eleven B figures reconcile to §4a by exactly +4 each** — the head-to-head vs payload convention (J-628); C figures identical. 🔑 **CONSEQUENCE: NOTHING IS A STRAIGHT DELETE, AND NO PAIR MAY BE RESOLVED BY LENGTH.** **Next: P3 — the move into `CLAUDE_HISTORY.md`.**
🟡 **Leg F — the bidirectional sweep** · 🟡 **Leg G — records + close.** 🛑 **LEG G IS NOW ORDER-CONSTRAINED: IT MUST NOT RUN BEFORE LEG B-TER** — it deletes `docs/ROADMAP_ARCHIVE_2026-07-26.md`, which is the **sole surviving source** for the six set-B bodies Leg C removed from `ROADMAP.md`.

🔒 **RULED:** §6 **BOTH** · §7 **S1, the tree** · **Leg C's scope** · §6a **E** · §6b **S1** · §6c **D-131** · §3a **W1** · **§4a-i's amendment** · **the `Leg E` collision** · **§8b — R2 with Joe's boundary** · **Leg E's title** · **§4b's rule promoted → `D-134`** · **B-bis §4 — (A) re-point, ONE LINE, not a class (J-624)** · **`D-135` confirmed — both-directions predicate testing (J-631)**.

🔓 **OPEN, JOE'S — NONE IS A GATE:** ① **sequencing: what runs next** (**Leg E P3 — the move into `CLAUDE_HISTORY.md`, under §5 and the §4r verdicts; NOT YET GO'D** · **Leg B-ter still needs a runbook first**) ② **the back-fill milestone's name** — belongs to Leg F's sweep ③ 📌 **`D-134` §2's justification clause** — recommended as an **annotation**, not an amendment. ✅ **④ `D-135` CLOSED — LOCKED BY JOE 2026-07-31 (J-631, *“confirmed”*); no longer reversible.**

✅ **SPENT, recorded so they are not re-opened:** §9's 100 KB size bar · §1b's Leg D structure question · **§8a's sequencing** (an output of §8b) · §4a's superseded framing · Leg C's structure consequence · Leg D's format question · **§4b's rule promoted → `D-134`**.

🛑 **NOT SPENT AFTER ALL — §8a's OWED CITATION COUNT WAS WRONG, AND §8b's `28` IS SUPERSEDED (J-620).** §8b measured **26 + 22 = 48 bare references** on 2026-07-26 and reported **28** as *surviving re-points* — but it never partitioned **citations** from **discussion of the collision itself.** Re-run 2026-07-29 over 507 `.md` files and **read per hit**: **87 bare hits = 4 definition sites · 71 discussion · 12 citations.** 🔑 **The true figure is 12**, tabulated per hit with its resolved target in `RUNBOOK_ROADTREE_LEGB_BIS.md` §2. ⚠️ **And the first pass of THAT measurement was also wrong** — a keyword classifier dropped two real citations while keeping seven of this session's own sentences; it was caught only because §8b's older per-file table named files the filter had emptied. 📌 **A keyword classifier on a corpus that discusses itself fails in both directions at once.**

**Chat owes, carried:** the registry composition model (needs a live client ⇒ INTERACTIVE) · the Ch0–Ch2 thesis read (272,516 B across three chapters). ✅ **Leg E grounding part two — DISCHARGED J-621.**
