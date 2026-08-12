# Clair — adversarial read of the Leg E Phase-0 (M-RP-MEMBER-ACT)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — YOUR SEAT, STATED FIRST

**You are CLAIR — Code Claude.** Not Chat Claude. If you find yourself writing records, updating `CLAUDE.md`, `JOURNAL.md`, `docs/ROADMAP.md` or any Phase-0, **you have taken the wrong seat and should stop.**

🛑 **THIS TASK CARRIES NO AUTHORITY TO CODE.** No `.rs`, no `ui/**`, no runbook, no commit. **It is a READ.** The deliverable is a findings list in your reply — nothing on disk.

📌 **Precedent: Leg 0 of this same milestone** (`tasks/M_RP_MEMBER_ACT_PHASE0.md` §6, closed 2026-08-06) was exactly this task and returned **five plan-moving findings plus three wording**, including *the milestone's central behaviour having no leg*. That is the bar.

---

## §2 — WHAT TO READ

**Baseline:** `HEAD` should be `origin/main`, verified with `git ls-remote origin refs/heads/main` — **not** the tracking ref. Re-measure it; do not inherit it from this document.

1. 🎯 **`tasks/M_RP_MEMBER_ACT_LEG_E_PHASE0.md` v1.2** — the subject of the read.
2. `tasks/M_RP_MEMBER_ACT_PHASE0.md` §6 (the leg table), `A3`, `F-D`, `K2` — Leg E's parent.
3. `JOURNAL.md` J-718 (this Phase-0's own entry), J-717, J-711, J-710, J-709.
4. `DECISIONS.md` → `D-065` · `D-071` · `D-114` · `D-121` (**three** lenses) · `D-123` · `D-131` · `D-141` · `D-143` · `D-146`.
5. The code the document makes claims about — **read it rather than trusting the quotes**: `spaces-panel.svelte` · `rooms-panel.svelte` · `members-panel.svelte` · `space-latch.svelte.ts` · `spaces-state.svelte.ts` · `ui/core/lib/components/layout/{resolve,mutate,types}.ts` · `layout-default.ts` · `ui/common/lib/plugins/registry.ts`.

---

## §3 — ALREADY KNOWN, SO YOU DO NOT SPEND THE READ RE-FINDING IT

🛑 **`E-2` was re-framed AFTER the ruling and the document is annotated in place.** The original *"`v3 → v4` migrate"* was recommended **before `resolve.ts` was read**. Superseded: `E-2` builds **`D-114` §9's re-inject rule** (a system `regionId` absent from a loaded layout is re-injected at a default dock). **Attack the NEW framing if it deserves it — do not re-report the old one.**

📌 The document's own **§8** is the author's list of where she thinks she is wrong. **It is therefore the lowest-value place to look.** A second reader's value is what §8 does not suspect.

---

## §4 — THE TARGETS, IN PRIORITY ORDER

### 🎯 T1 — **Walk every 🔒 in §4① and §5 and ask: WHICH LEG BUILDS THIS?**
This is the rule minted at `M_RP_MEMBERS.md` §8b after Leg C was **blocked** by exactly this defect: *scope written in FILES, requirements in BEHAVIOURS, never reconciled.* §4① carries **five numbered constraints**. §5 carries six sub-legs. **Any constraint with no leg behind it is a finding.** The same document that quotes this rule committed the error the rule names — twice in this milestone already.

### 🎯 T2 — **Are `E-1` and `E-3` genuinely separable?**
The document locks *the home ships before the filter, in the same leg*. **Try to break it.** Does `E-1` silently need something only `E-3` builds, or the reverse? Does an intermediate state exist that is worse than either endpoint? *(The `M-RP7.1b` precedent: §4.1 and §4.4 had to ship together, and shipping them apart would have been strictly worse than shipping neither.)*

### 🎯 T3 — **Is `F5` sound?** — that the R1 filter strands the self thread
It rests on three call sites: `members-panel.svelte:246` (self-click no-op), `OQ6-E2` having deleted the `self_open` command, and `counterpart` holding the session identity for the self thread. **It has never been driven live.** If any leg of that argument is wrong, `①`'s constraint 1 loses its ground.

### 🎯 T4 — **`F7`'s pricing of H1**
§8 item 2 flags this as *the kind of pricing that has been wrong three times in this arc* (OQ1, OQ6, OQ8 — *each time a cost was priced against one leg and never re-checked against the others*). **Price it yourself against the code.** The re-inject work is the piece most likely to be larger than stated.

### 🎯 T5 — **`F6`'s registry claim**
The document says A3 changes `items`, removing rows, which moves the registry — and that C-bis-6's miniature proves nothing about it. **Check the row-count arithmetic**: `N-184` says one DM Space row registers **two** entities (`entity-item` + `entity-avatar`). Does the document's verification plan actually account for that, and for `N-190`'s draft axis?

### 🎯 T6 — **Anything the document asserts about code that the code does not say.**
Every file:line quote in §2 and §3 is fair game. **`N-180`'s rule applies to this document as much as to any other: an architecture claim needs the source, not memory.**

---

## §5 — HOW TO REPORT (Rule 6 shape)

For each finding: **what the document says · what the code or record says · file:line · whether it moves the plan or is wording.** Separate **plan-moving** from **wording** explicitly, as Leg 0 did.

🛑 **Do not silently absorb a defect and work around it** — Rule 6 exists because *an implementer who silently absorbs a bad instruction ships the architect's mistake*. That rule earned its keep at M-RP7.1b, where your `migrateLayout(raw, fallback)` deviation stopped `core` importing a shell constant.

✅ **A finding that the document is CORRECT on a suspect point is also worth reporting** — Leg 0 confirmed one of §8's own doubts and dissolved another, and both were useful.

⚠️ **If a target above rests on a false premise, say so and stop working it.** A brief handed to you is an assertion like any other and gets no exemption from the read it is handed to.

---

## §6 — WHAT HAPPENS NEXT

Your findings go to Joe. He rules. **Then** Chat writes the runbook, and you get a **second, shorter read** of that — because the TAIL8 evidence is that your findings came from trying to **run** a runbook, not from reading a design. **This read is the design pass; the runbook pass follows.**
