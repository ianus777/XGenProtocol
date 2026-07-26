# M-DOC-ROADTREE — the roadmap becomes a state board
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-26  
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
| ✅ **DONE** | `· J-nnn` — **non-negotiable.** The link is what licenses the prose to leave |
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

## §6 — 🔓 DECISION 4: SCOPE — ROADMAP ALONE, OR ROADMAP + THE CLAUDE.md PLAY HEAD? JOE'S

`ROADMAP.md` and the `CLAUDE.md` PLAY block are **paired under D-074** — they travel in the same commit on every state change. `CLAUDE.md` is **608,702 bytes with a single 124,299-character line** and has the identical disease: append-only narration in a document meant to show current state.

- **① User-visible:** none. Both are internal records. **"No user-facing impact" is a legal answer (D-121).**
- **② Tier:** none.
- **③ Resource:** the split mechanism is the same for both, so doing them together is **cheaper than doing them twice**. Doing only ROADMAP leaves the two **free to disagree**, which is the state D-074 exists to prevent.

**CHAT PROPOSES: BOTH.** 🔓 **OPEN.**

---

## §7 — 🔓 DECISION 5: DOES THE TREE GROUP BY STRUCTURE, OR BY STATUS? JOE'S

§3's format takes its shape from **nesting alone** — no track/status/date columns. That is a bet that **structure is what Joe navigates by**.

- **(S1) BY STRUCTURE — the tree as specified.** ① You find a thing by knowing where it lives. ③ One document, one ordering.
- **(S2) BY STATUS — "what is in play right now" as its own section.** ① You find a thing by knowing its state. ⚠️ **That is a board, not a tree**, and it is a different document from the one §3 describes.

⚠️ **THIS CHANGES EVERYTHING DOWNSTREAM AND MUST BE ANSWERED BEFORE LEG B.** **CHAT PROPOSES S1** — the status symbols already make state greppable within a tree, whereas structure cannot be recovered from a status grouping. 🔓 **OPEN.**

---

## §8 — Legs

**Leg 0 — Phase-0.** This document. No code. 🔓 Gated on §6 and §7.

**Leg A — the pause + the archive.** ⏸️ M-RP-MEMBERS Leg C with its trigger (§5), in `ROADMAP.md` + `CLAUDE.md` PLAY + the GitHub board. ✅ **The archive is already taken** — `docs/ROADMAP_ARCHIVE_2026-07-26.md`, byte-identical at 749,717, header ARCHIVED, **carrying its own delete condition**. **Surface: `docs/ROADMAP.md`, `CLAUDE.md`, board field `d0103551`.**

**Leg B — clear the precondition.** Walk P1's **94** unlinked DONE markers and P2's **11** unresolved refs. **Surface: `docs/ROADMAP.md`, `JOURNAL.md`, `JOURNAL_ARCHIVE.md`.** ⚠️ **No deletion happens in this leg.** Output is a link-or-redundant verdict per item, and any journal corrections P2 turns up.

**Leg C — the migration.** Rewrite `ROADMAP.md` to §3's format. **Surface: `docs/ROADMAP.md`.** 🔓 Shape gated on §7.

**Leg D — CLAUDE.md.** Same treatment on the PLAY head. **Surface: `CLAUDE.md`, `CLAUDE_HISTORY.md`.** 🔓 Gated on §6.

**Leg E — the bidirectional sweep.** ⚠️ **Not only *is every roadmap entry true*, but *is every known work item ON the roadmap*.** **Surface: this session's open threads** — M-RP-LIVEFEED-REFRESH · the resync sibling · the outbox · H1 · H2 · D-130 · the address-book eviction question · `NegotiatedCapabilities` · the Ch0–Ch2 thesis read. **They exist in chat and in one task document and nowhere else.** They are the natural test case for whether the sweep works.

**Leg F — records + close.** JOURNAL + CLAUDE.md PLAY + ROADMAP + this document, **one commit** (D-074). Delete `ROADMAP_ARCHIVE_2026-07-26.md` **iff** Leg B cleared.

---

## §9 — DoD

Applying `M_RP_MEMBERS.md` §8b's rule — **every item naming an action names its surface in §8, and every 🔒 in this document has a leg that builds it:**

- [ ] M-RP-MEMBERS Leg C reads ⏸️ **with its trigger** in ROADMAP, CLAUDE.md and the board — **Leg A**
- [ ] All **94** P1 items **measured**: journal link found, or redundancy confirmed in writing — **Leg B**
- [ ] All **11** P2 refs **measured**: entry located, or absence recorded — **Leg B**
- [ ] Every ✅ node in the new tree carries `· J-nnn` — **Leg C**
- [ ] Every 🟡 / ⏸️ node carries a trigger **or an explicit `trigger: none`** — **Leg C**
- [ ] Every existing ⏸️ **POSTPONED** entry (**22** of them) **audited for a resume trigger** — **Leg C**
- [ ] No `→` arrow points at a journal entry that does not record the loose end — **Legs B + C**
- [ ] ROADMAP re-**measured** under 100 KB — **Leg C**
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

**Blocked on Joe:** §6 (scope) · §7 (tree versus board).
**Not blocked:** Leg A (the pause is ruled) · Leg B (the precondition walk needs no decision).
**Chat owes, carried:** the registry composition model · address-book eviction, declared versus wired · `NegotiatedCapabilities`' shape · the Ch0–Ch2 thesis read.
