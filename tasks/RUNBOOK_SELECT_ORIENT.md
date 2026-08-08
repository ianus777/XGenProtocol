# M-RP-SELECT-ORIENT — the panels keep saying where you are — RUNBOOK
> **Status**: COMPLETED  
> Version: 1.4  
> Date: Aug 2026  
> **Last updated**: 2026-08-08  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## ✅ CLOSED — J-697, 2026-08-08

**All four commits shipped and every gate re-driven by Chat (Rule 5):** C-1 `517cf94` · C-2 `d8edd85` · C-3 `cd53c6d` · C-4 `62c72f6`. **`L-1` through `L-13` and `L-15` all satisfied and driven on the real client.** Floors at close: svelte-check **0/34/15** · catalogue **435 = unique = domCount, zero orphans** · sampler `npm test` **154/9** · cargo **1597/0/62 × 56** untouched by scope · client registry **164 quiescent → 174 selected**.

📌 **`L-14` IS ANNOTATED, NOT SATISFIED AS WRITTEN.** It said *"the importer count stays six."* Its prohibition — do not **decrement** when R1/R2 leave the bus — **held**: neither panel dropped its import, because both still WRITE. But C-1 **incremented** the count to **seven** (`space-latch.svelte.ts:37`, a dormant default-arg) for a reason the lock never contemplated. 🔑 ***A claim narrower than the thing it describes — this milestone's own named error class, landing on this milestone's own lock.*** Corrected at the site in `selection.svelte.ts` per `D-145`, corpus stated before the claim.

🛑 **AND THIS DOCUMENT'S OWN LINE NUMBERS WENT STALE INSIDE ITS OWN EXECUTION — Clair's finding, reported not fixed.** §5 and §2/G3 cite the `rooms-panel` derive at **`:42-45` / `:42`**; measured at `d8edd85` it was **`:35-38`**. **C-1 shrank the file, so this runbook staled its own citations two commits before they were used.** Clair trusted the code and flagged the divergence. 🔑 ***A runbook that cites line numbers is a runbook that expires against its own commits — cite the SITE, re-measure the LINE.*** Annotated here rather than repaired: the document was **locked** when the drift occurred (`D-145`).

⚠️ **ONE SHIPPED CLAIM IS REASONED, NOT DRIVEN.** Clair routed the **arrow keys** through `activeClamped` as well as the render and Enter sites, on the argument that a raw `ArrowUp` after a multi-position shrink re-writes a stale index. Sound and cost-free — but **unreachable on Joe's data**: the largest Space has 2 rooms, so every shrink here is 2→1, where `Math.max(0, n-1)` and `Math.min(n-1, n+1)` both self-heal even from the raw state. **The routing stays; no gate exercised it.**

⚠️ **`L-12`'s ACTIVATION ASSERTION SHIPPED WITH A STATED LIMIT.** `cdp-debug.ps1` has `click` and `drag` and **cannot press a key**, so the Enter assertion was driven with a **dispatched** `keydown`. That genuinely runs `onKey → selectAt(activeClamped) → onActivate` — the listener is Svelte's, and `isTrusted` gates only native defaults — but it **does not prove the browser routes a physical Enter to that `<li>`**; the roving `tabindex=0` plus an asserted `document.activeElement` is the separate evidence for that. **A harness `key` mode is filed as Chat's own tooling commit, before `M-RP-MEMBER-ACT` Leg C.**

## §0 — FOR CLAIR: WHAT THIS BUILDS AND WHAT IT MUST NOT

**Four commits. Two behaviour changes, one lift, one `ui/core` clamp.**

🔒 **NOTHING HERE IS OPEN. `OQ-1` closed as option A** (`M_RP_SELECT_ORIENT_PHASE0.md` §5). **If you find something that reopens it, STOP and report — do not choose.**

🔒 **LOCKED 2026-08-08 (Joe: *"locked"*), ALL FIFTEEN POINTS `L-1`–`L-15` AS PUT TO HIM. ✅ CLAIR MAY NOW WRITE CODE.** The fifteen, in one line each: **L-1** select a room ⇒ the Space stays lit · **L-2** select an identity ⇒ the room stays lit · **L-3** a shorter list no longer strands the panel · **L-4** a NEW `$common` store, sibling to `roomLatch`, not folded in · **L-5** write rule SPACE-SELECTION ONLY · **L-6** the write is SHELL-driven, one effect two latches · **L-7** C-1 REPAIRS the fold defect, Gate C-1 has two parts · **L-8** R1 reads the RAW `latchedSpaceId` · **L-9** BOTH stale `D4` comments rewritten · **L-10** R2 reads `effectiveRoomId` · **L-11** `activeIndex` CLAMPED at every consumer, not re-seeded · **L-12** Gate C-4 asserts ACTIVATION · **L-13** four commits in order · **L-14** the close annotation corrects the CONSEQUENCE paragraph, the count stays six · **L-15** option D NOT built.

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

## §2 — GROUND TRUTH (re-verified at `e0fc072`; re-measure before you edit — the tree has writers who are not you)

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

## §3 — C-1: LIFT THE SPACE LATCH. **Unchanged on the non-fold path; REPAIRS the fold path.**

**NEW FILE** `ui/common/lib/stores/space-latch.svelte.ts`.

🔒 **THE WRITE RULE IS SPACE-SELECTION ONLY** — byte-for-byte the predicate at **G1**. 🛑 **DO NOT add a room-resolves-to-Space arm.** *That is option B's behaviour and Joe's requirement rejects it (Phase-0 §5).*

**Shape — mirror `room-latch.svelte.ts`, do not import it (G6):**

- `note(sel)` — the **single writer**; assigns only when `sel?.entity.kind === 'space'`.
- `clear()`.
- `latchedSpaceId` — raw getter.
- `scopedSpace` — the resolve, carrying **G2's stale-latch guard**: gone-between-hydrations ⇒ `null`, never throw.

🔒 **DRIVEN BY THE SHELL, NOT BY A WIDGET.** `ui/client/src/app_client.svelte:195-198` already runs the bus→latch `$effect` for `roomLatch`; **add the `spaceLatch.note(sel)` call inside that SAME effect**, under the same `untrack`. *One effect, two latches — do not add a second `$effect` reading the bus.*

**Then edit `rooms-panel.svelte`:** delete `:23-27` (the private latch) and `:31-33`'s local resolve; read `scopedSpace` from the store. **Both honest empty states must survive verbatim** — `"Select a space"` when nothing is latched, `"No rooms"` when a Space is latched and empty. *They are different truths (N-091).*

🛑 **AND REPOINT `:58` — THE THIRD `latchedSpaceId` SITE, WHICH v1.0 DID NOT NAME (Clair's F5).** `latchedSpaceId` appears at `:23`, `:26`, `:32` **and `:58`, inside the debug getter**. **Repoint it to `spaceLatch.latchedSpaceId`; do not delete the field** — the getter is the verify surface every gate reads. *svelte-check would catch a dangling reference, but a runbook that enumerates deletions by line owes the complete set.*

⚠️ **THE COMMENT AT `rooms-panel:19-22` EXPLAINS A MECHANISM THAT IS MOVING.** Rewrite it to point at the store; **keep the N-136 note** (the effect writes but never reads the latch, so there is no self-invalidating read-modify-write).

📌 **`app_client` needs the `spaceLatch` import** — stated for completeness; svelte-check catches its absence.

**Gate C-1:** svelte-check **0/34/15** · catalogue **435** · **behaviour unchanged ON THE NON-FOLD PATH — the clean baseline C-2 is measured against.**

🛑 **C-1 IS NOT A NEUTRAL REFACTOR. IT REPAIRS A LIVE DEFECT — MEASURED 2026-08-08 ON THE REAL CLIENT (Clair's F2, driven by Chat).** v1.0 said *"no rendered behaviour change"*. **That is FALSE on the fold path, and the truth is better than the claim.**

**Folding a region tile UNMOUNTS the widget** — `region-node.svelte:225-228` mounts it as `RegionTile`'s children; `region-tile.svelte:173` gates `{@render children?.()}` behind `{#if collapsed === undefined}`. ⇒ **the component-local `latchedSpaceId` `$state` is DESTROYED.**

🔒 **DRIVEN: select Engineering → select a room → fold R2 → unfold.**

| | measured |
|---|---|
| on fold | `rooms-panel#region-rooms` and its panel **deregister** — catalogue 174 → **167** |
| after unfold | R2 reads **`count: 0`, `hasEmpty: true`** — *"Select a space"* |
| meanwhile | `roomLatch.effectiveRoomId` **UNCHANGED** — R5 and the composer still work in that room |

⇒ ***Two clicks empty the rooms panel while the stream and composer keep going. Shipped today.*** **After C-1 the scope survives, because the store is app-lifetime** — which is `room-latch.svelte.ts:24`'s own *"APP-LIFETIME, NOT WIDGET-LIFETIME"* argument, applied to the Space latch.

🔒 **GATE C-1 THEREFORE HAS TWO PARTS:** ① **non-fold path unchanged** (the C-2 baseline, intact) · ② **fold→unfold now PRESERVES the scope** — R2 lists the same rooms, `count` unchanged, **not `hasEmpty`.**

---

## §4 — C-2: R1 HIGHLIGHTS FROM THE LIFTED LATCH. **The `D4` opt-2 behaviour.**

`spaces-panel.svelte:44-46` — `selected` derives from **`spaceLatch.latchedSpaceId`**, not the bus.

🛑 **AND G5's COMMENT BLOCK NOW ASSERTS A SUPERSEDED LOCK.** Rewrite `:16-18`: R1 highlights the **latched** Space, cite **`D-146`** and the `D4` opt-1 → opt-2 supersede. *A comment that describes the opposite of the code is the defect this project annotates hardest.*

🛑 **THERE IS A SECOND ONE, AND v1.0 MISSED IT (Clair's F1). `spaces-panel.svelte:42-43` — SITTING DIRECTLY ABOVE THE LINES C-2 CHANGES:**

> *"Read the bus BACK (D5). R1 owns the 'space' facet of the one selection; a room selection (R2) leaves this undefined -> R1 un-highlights (D4 opt-1)."*

⇒ **BOTH comments become false the moment `:44-46` repoints. Rewrite `:42-43` too, or fold it into the `:16-18` rewrite.** 🔑 ***v1.0's own stated principle condemned the exact line it left standing — leaving it would ship this milestone's named defect in the file the commit just edited.***

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

🛑 **DO NOT re-seed it on every `items` change.** G8's one-time capture is deliberate — focus is its own concern once the user navigates. **Clamp only.** 📌 **A form EXISTS — §9-2's fear that none might was unfounded (Clair's F3).** **Not a new `$effect` writing `activeIndex`** (that reintroduces the N-136 read-modify-write G1's comment warns about).

🛑 **BUT THE FORM v1.0 SUGGESTED IS INCOMPLETE, AND ITS OWN GATE GREENLIGHTS THE HOLE (Clair's F3 — the sharpest finding of the read).** v1.0 offered *"a `$derived` read used at the tabindex site"*. **`activeIndex` has FOUR consumers, not one:**

| site | reads | clamped by the tabindex-only form? |
|---|---|---|
| `:183` | the `tabindex` render | ✅ yes |
| `:115` ArrowDown | `Math.min(n - 1, activeIndex + 1)` | ✅ self-heals internally |
| `:118` ArrowUp | `Math.max(0, activeIndex - 1)` | ✅ self-heals internally |
| **`:129` Enter / Space** | **`selectAt(activeIndex)` — the RAW state** | 🛑 **NO** |

🔑 **TRACE, after a 2 → 1 shrink with `activeIndex = 1`:** the clamp lights row 0 ⇒ **Gate C-4 passes verbatim** ⇒ the user tabs in and lands on row 0 ⇒ presses **Enter** ⇒ `selectAt(1)` ⇒ `items[1]` is `undefined` ⇒ `if (!it) return` ⇒ ***silent no-op.*** ⚠️ **And nothing re-syncs on focus** — verified: `entity-panel` has no `onfocus`/`onfocusin` handler touching `activeIndex`. It self-heals only on an arrow press.

⇒ ***The panel becomes reachable and its first keyboard activation does nothing — "the panel lies about where you are", this milestone's own theme.***

🔒 **THE FIX MUST ROUTE EVERY CONSUMER THROUGH THE CLAMP** (or clamp the state at the point `items` is consumed) — **not the render site alone.**

⚠️ **Empty list:** `items.length === 0` renders `emptyText`, no `<li>` — make sure the clamp does not produce `-1`. *(`onKey` already guards with `if (n === 0) return` at `:111`.)*

**Gate C-4:** cargo **untouched** · svelte-check · **catalogue 435 — a clamp registers no id; if it moves, STOP and report** · **live, TWO assertions:** ① 2-room Space → click index 1 → 1-room Space ⇒ exactly ONE `li[tabindex="0"]` · 🔒 ② **ACTIVATION: focus that row, press Enter ⇒ `onActivate` FIRES.** *① alone is satisfied by the incomplete form — that is why ② exists.*

📌 **Blast radius: `spaces-panel`, `rooms-panel`, `members-panel` and seven sampler cells.** *Which is why it is alone.*

---

## §7 — C-5: RECORDS (`D-074`)

JOURNAL + `CLAUDE.md` PLAY + ROADMAP + this runbook `COMPLETED` + the Phase-0, **one commit**.

**Owed, named in the Phase-0:** the **milestone-close annotation** on `selection.svelte.ts`.

🛑 **AND v1.0 MIS-DESCRIBED IT (Clair's F4). THE IMPORTER COUNT DOES NOT CHANGE — IT STAYS SIX.** R1 and R2 keep `selection.set` in their `onActivate` writers (`spaces-panel:50`, `rooms-panel:49`), both listed in §8 as NOT TOUCHED. **What goes stale is the CONSEQUENCE paragraph at `selection.svelte.ts:22-30`** — the one asserting the shipped defect — because R1/R2 stop reading `.current` **for the highlight**. ⚠️ ***Decrementing the "SIX IMPORTERS" line would be factually wrong. That comment block has already been wrong three times by its own count; a fourth would be this milestone's.***

---

## §8 — NOT TOUCHED

`selection`'s **shape** (S-6: one bus, do not widen — this milestone adds a **latch**, not a bus) · `roomLatch`'s room semantics and its `note()` single-writer rule · `onActivate` in either panel · R5, R7, R8, the composer · `canSend` · the wire, `xgen-core`, `xgen-node`, `xgen-common` · **`skin.css` — Joe's** · **everything in `M-RP-MEMBER-ACT` Leg C** · **option D, the two-state highlight — FILED, not built.**

---

## §9 — WHERE THIS RUNBOOK IS MOST LIKELY WRONG

🔒 **CLAIR'S ADVERSARIAL READ LANDED 2026-08-08 AND RETURNED FIVE FINDINGS. CHAT RE-DROVE ALL FIVE; ALL FIVE CONFIRMED.** *Both of v1.0's self-suspected items were driven to ground: one held, one was a real hole this document only half-saw.*

1. ✅ **DISCHARGED — §3's SHELL-DRIVEN WRITE IS SOUND. v1.0's #1 NAMED RISK HOLDS.** Clair traced it: `app_client.svelte:195-198` tracks **only** `selection.current`; adding `spaceLatch.note(sel)` beside `roomLatch.note(sel)` under the same `untrack` composes, because **both `note()` are pure writers gated on MUTUALLY EXCLUSIVE `entity.kind`** (`room` vs `space`), neither reads its own state, neither writes `selection.current` ⇒ **no re-entrancy, no ordering dependency.** ✅ **And no cross-talk with the sibling members effect at `:207-210`**, which tracks `roomLatch.effectiveSpaceId` — untouched by the Space-latch write. *Superseded text follows.* — 🛑 **§3's SHELL-DRIVEN WRITE IS THE RISKIEST INSTRUCTION HERE.** *Asserted from reading `:195-198`, not driven. Report rather than improvise.*
2. ✅ **DISCHARGED, AND IT WENT THE OTHER WAY — A FORM EXISTS, BUT THE ONE v1.0 SUGGESTED IS INCOMPLETE (Clair's F3).** §6 now carries the four-consumer table and the Enter-key trace. 🔑 ***The fear was "no form satisfies the constraints"; the truth was "a form does, and this document named a partial one whose own gate passes it."*** ⇒ **Gate C-4 gained an ACTIVATION assertion.** *Superseded text follows.* — 🛑 **§6's CLAMP MECHANISM IS NAMED BY CONSTRAINT, NOT BY CODE.** *If no form satisfies all three constraints, that is a finding.*
3. ⚠️ **`rooms-panel:25`'s `$effect` is traced as SCOPE, not highlight** — the basis for C-1 replacing it wholesale while C-3 touches only `:43-45`. **READ, NOT DRIVEN** (Phase-0 §8-3). ✅ *Clair confirmed the trace on the same evidence; still not driven.*
4. ⚠️ **NO CATALOGUE PREDICTION.** A store registers no id, a clamp registers no id ⇒ **435 throughout, expected not proven.** 📌 **Note: the catalogue BREATHES with folding — measured 174 → 167 on a single fold.** *Any catalogue gate must run with every tile unfolded.*
5. ✅ **DISCHARGED — THE LEG E COLLISION IS GENUINELY UNREACHABLE TODAY.** Clair enumerated every `selection.set` writer: **`rooms-panel:49` (room) · `spaces-panel:50` (space) · `self-panel:77` (identity)** — and `members-panel`'s appears **only in comments** (`:14`, `:17`); `L-7` is not wired. **A room cannot be selected without a latched Space, because rooms only render when `scopedSpace !== null`.** ⇒ **confirmed unreachable; owed to `M-RP-MEMBER-ACT` Leg E.**
6. 🔑 **v1.0's ITEM 6 SAID ITS AUTHOR GOT THIS MILESTONE'S CENTRAL IDENTIFIER WRONG. IT DID NOT PREDICT THE THREE THINGS CLAIR FOUND** — a second stale comment in the file C-2 edits, a false *"behaviour UNCHANGED"*, and a clamp form its own gate would greenlight. ⇒ ***v1.0's self-suspicion list was RIGHT about what it doubted and BLIND to all three plan-movers. Tenth consecutive arc in which Chat's own re-reads caught nothing that mattered.***
7. ⚠️ **THE GATE C-2 / C-3 "MEASURED THIS SESSION" BASELINES WERE NOT RE-VERIFIED BY CLAIR** — no client was up on her seat and Joe's state is read-only. **She marked them plausible-not-reproduced, and said so rather than assuming.** 📌 *Chat drove them originally and drove F2 live afterwards; the C-2/C-3 baselines remain single-sourced to Chat.*
