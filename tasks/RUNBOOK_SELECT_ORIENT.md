# M-RP-SELECT-ORIENT — the panels keep saying where you are — RUNBOOK
> **Status**: PENDING  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-08  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — FOR CLAIR: WHAT THIS BUILDS AND WHAT IT MUST NOT

**Four commits. Two behaviour changes, one lift, one `ui/core` clamp.**

🔒 **NOTHING HERE IS OPEN. `OQ-1` closed as option A** (`M_RP_SELECT_ORIENT_PHASE0.md` §5). **If you find something that reopens it, STOP and report — do not choose.**

🛑 **THIS DOCUMENT IS `PENDING` UNTIL JOE LOCKS IT. NO CODE BEFORE THE LOCK.**

⚠️ **`R2` IS THE ROOMS PANEL. `R6` IS THE COMPOSER** (`ui/client/src/layout-default.ts:27` and `:31`). *The Phase-0's v1.0 got this wrong in nine places; if you see a bare "R6" anywhere meaning a rooms list, it is a residue — report it.*

---

## §1 — WHAT THE USER SEES WHEN THIS LANDS

| | before | after |
|---|---|---|
| select a room | 🛑 **the Space un-highlights** while you browse its rooms | ✅ **the Space stays lit** |
| select a member / self card (an **identity** on the bus) | 🛑 **the room un-highlights** while the composer still targets it | ✅ **the room stays lit** |
| navigate to a shorter list | 🛑 **the panel leaves the tab order** — no row tabbable | ✅ **a row stays tabbable** |

🔒 **The first row SUPERSEDES `M-RP6.2` `D4` opt-1 → opt-2** (`DECISIONS.md` `D-146`; annotated at `tasks/M_RP6_2_SPACES_ROOMS.md`). **Joe's, uttered.**

---

## §2 — GROUND TRUTH (measured at `dd7d641`; re-measure before you edit — the tree has writers who are not you)

| # | site | what is there now |
|---|---|---|
| **G1** | `ui/common/lib/components/widgets/rooms-panel.svelte:23-27` | `let latchedSpaceId = $state<string \| null>(null)` + an `$effect` writing it **only** when `selection.current?.entity.kind === 'space'` |
| **G2** | `rooms-panel.svelte:31-33` | `scopedSpace = $derived(spacesState.spaces.find(s => s.space_id === latchedSpaceId) ?? null)` — the **stale-latch guard**, N-095 spirit: never throw |
| **G3** | `rooms-panel.svelte:42-45` | `selected` derived from the **bus**, room facet |
| **G4** | `spaces-panel.svelte:44-46` | `selected` derived from the **bus**, space facet |
| **G5** | `spaces-panel.svelte:16-18` | a comment block asserting `D4` opt-1 **by name** — *"R1 un-highlights while you browse its rooms"* |
| **G6** | `ui/common/lib/stores/room-latch.svelte.ts` header | 🛑 *"R2's Space latch is NOT touched by this store and must not be folded into it"* |
| **G7** | `room-latch.svelte.ts` | exposes `latchedRoomId` (raw), `effectiveRoomId` / `effectiveSpaceId` / `canSend` — all from ONE `resolveLatched()` |
| **G8** | `ui/core/lib/components/data-dependent/entity-panel.svelte:90-94` | `activeIndex = $state(initialActive())` — **one-time capture, never clamped** |
| **G9** | `entity-panel.svelte:183` | `tabindex={i === activeIndex ? 0 : -1}`; the `<ul>` carries no tabindex |

🔒 **FLOORS (stated; C-4 and C-5 re-measure):** cargo **1597/0/62 × 56** · svelte-check **0/34/15** · catalogue **435**.

---

## §3 — C-1: LIFT THE SPACE LATCH. **No rendered behaviour change.**

**NEW FILE** `ui/common/lib/stores/space-latch.svelte.ts`.

🔒 **THE WRITE RULE IS SPACE-SELECTION ONLY** — byte-for-byte the predicate at **G1**. 🛑 **DO NOT add a room-resolves-to-Space arm.** *That is option B's behaviour and Joe's requirement rejects it (Phase-0 §5).*

**Shape — mirror `room-latch.svelte.ts`, do not import it (G6):**

- `note(sel)` — the **single writer**; assigns only when `sel?.entity.kind === 'space'`.
- `clear()`.
- `latchedSpaceId` — raw getter.
- `scopedSpace` — the resolve, carrying **G2's stale-latch guard**: gone-between-hydrations ⇒ `null`, never throw.

🔒 **DRIVEN BY THE SHELL, NOT BY A WIDGET.** `ui/client/src/app_client.svelte:195-198` already runs the bus→latch `$effect` for `roomLatch`; **add the `spaceLatch.note(sel)` call inside that SAME effect**, under the same `untrack`. *One effect, two latches — do not add a second `$effect` reading the bus.*

**Then edit `rooms-panel.svelte`:** delete `:23-27` (the private latch) and `:31-33`'s local resolve; read `scopedSpace` from the store. **Both honest empty states must survive verbatim** — `"Select a space"` when nothing is latched, `"No rooms"` when a Space is latched and empty. *They are different truths (N-091).*

⚠️ **THE COMMENT AT `rooms-panel:19-22` EXPLAINS A MECHANISM THAT IS MOVING.** Rewrite it to point at the store; **keep the N-136 note** (the effect writes but never reads the latch, so there is no self-invalidating read-modify-write).

**Gate C-1:** svelte-check **0/34/15** · catalogue **435** · 🔒 **behaviour UNCHANGED — this is the clean baseline C-2 is measured against.**

---

## §4 — C-2: R1 HIGHLIGHTS FROM THE LIFTED LATCH. **The `D4` opt-2 behaviour.**

`spaces-panel.svelte:44-46` — `selected` derives from **`spaceLatch.latchedSpaceId`**, not the bus.

🛑 **AND G5's COMMENT BLOCK NOW ASSERTS A SUPERSEDED LOCK.** Rewrite `:16-18`: R1 highlights the **latched** Space, cite **`D-146`** and the `D4` opt-1 → opt-2 supersede. *A comment that describes the opposite of the code is the defect this project annotates hardest.*

📌 **`onActivate` is UNCHANGED** — R1 still writes the bus. **The bus stays the single selection; R1 stops READING it for the highlight.**

**Gate C-2:** svelte-check · catalogue · **live 9222: select a Space, then a room ⇒ `entity-panel#region-spaces__panel` `state.selected` is STILL the Space id.** 🔒 *Before C-2 that read `null` — measured this session.*

---

## §5 — C-3: R2 HIGHLIGHTS FROM THE ROOM LATCH. **Independent of C-1/C-2.**

`rooms-panel.svelte:42-45` — `selected` derives from **`roomLatch.effectiveRoomId`**, not the bus.

🔒 **`effectiveRoomId`, NOT `latchedRoomId`** (Phase-0 §4). *The raw latch is a verify/debug surface and can name a room that no longer resolves; `effectiveRoomId` is the room R5 renders and the composer targets, so the highlight cannot disagree with them.*

⚠️ **C-1 AND C-3 BOTH EDIT `rooms-panel.svelte`** — different, non-overlapping lines. *The Phase-0 v1.0 said "a different panel"; that was false (Clair's F5). Sequence them; do not merge them.*

**Gate C-3:** svelte-check · catalogue · **live: select a room, then click the R3 self card (writes an identity to the bus) ⇒ `entity-panel#region-rooms__panel` `state.selected` is STILL the room id.** 🔒 *Before C-3 that read `null` while `roomLatch.latchedRoomId` stayed put — measured this session.*

---

## §6 — C-4: CLAMP `activeIndex`. **`ui/core`, measured alone.**

`entity-panel.svelte` — `activeIndex` must not exceed `items.length - 1`.

🛑 **DO NOT re-seed it on every `items` change.** G8's one-time capture is deliberate — focus is its own concern once the user navigates. **Clamp only.** A `$derived` read used at the `tabindex` site, or a clamp where `items` is consumed; **not a new `$effect` writing `activeIndex`** (that reintroduces the N-136 read-modify-write G1's comment warns about).

⚠️ **Empty list:** `items.length === 0` renders `emptyText`, no `<li>` — make sure the clamp does not produce `-1` and light up a row that should not exist.

**Gate C-4:** cargo **untouched** · svelte-check · **catalogue 435 — a clamp registers no id; if it moves, STOP and report** · **live: 2-room Space → click index 1 → 1-room Space ⇒ exactly ONE `li[tabindex="0"]`.** 🔒 *Measured today at **zero**: `focusables` 0, `role="listbox"` intact — a listbox nothing can focus.*

📌 **Blast radius: `spaces-panel`, `rooms-panel`, `members-panel` and seven sampler cells.** *Which is why it is alone.*

---

## §7 — C-5: RECORDS (`D-074`)

JOURNAL + `CLAUDE.md` PLAY + ROADMAP + this runbook `COMPLETED` + the Phase-0, **one commit**.

**Owed, named in the Phase-0:** the **milestone-close annotation** on `selection.svelte.ts` dropping the reader count — R1 and R2 stop reading the bus for their highlight, so *six importers* becomes stale the moment C-3 lands.

---

## §8 — NOT TOUCHED

`selection`'s **shape** (S-6: one bus, do not widen — this milestone adds a **latch**, not a bus) · `roomLatch`'s room semantics and its `note()` single-writer rule · `onActivate` in either panel · R5, R7, R8, the composer · `canSend` · the wire, `xgen-core`, `xgen-node`, `xgen-common` · **`skin.css` — Joe's** · **everything in `M-RP-MEMBER-ACT` Leg C** · **option D, the two-state highlight — FILED, not built.**

---

## §9 — WHERE THIS RUNBOOK IS MOST LIKELY WRONG

1. 🛑 **§3's SHELL-DRIVEN WRITE IS THE RISKIEST INSTRUCTION HERE.** Putting `spaceLatch.note()` in `app_client`'s existing effect is asserted from reading `:195-198`, **not driven**. *If the two latches need different tracking, or `untrack` interacts badly, this is where it shows.* **Report rather than improvise.**
2. 🛑 **§6's CLAMP MECHANISM IS NAMED BY CONSTRAINT, NOT BY CODE.** This document says what it must not be (`$effect`, re-seed) and leaves the form to you. **If no form satisfies all three constraints, that is a finding, not an implementation detail.**
3. ⚠️ **`rooms-panel:25`'s `$effect` is traced as SCOPE, not highlight** — the basis for C-1 replacing it wholesale while C-3 touches only `:43-45`. **READ, NOT DRIVEN** (Phase-0 §8-3).
4. ⚠️ **NO CATALOGUE PREDICTION.** A store registers no id, a clamp registers no id ⇒ **435 throughout, expected not proven.**
5. ⚠️ **THE FUTURE COLLISION IS OUT OF SCOPE AND WILL LOOK LIKE A BUG.** Enter a room whose Space was never clicked — a DM under Leg C — and **R2 lists the previous Space's rooms**. 🔒 **Not reachable today**: the only room-selection writer is `rooms-panel:49`, reachable only after a Space is latched. **Owed to `M-RP-MEMBER-ACT` Leg E.**
6. 📌 **THIS RUNBOOK HAS NOT BEEN READ BY ANYONE OUTSIDE ITS AUTHOR, AND ITS AUTHOR GOT THIS MILESTONE'S CENTRAL IDENTIFIER WRONG.** *The last three runbooks were each sent back once by a Clair read; the Phase-0 was sent back twice.*
