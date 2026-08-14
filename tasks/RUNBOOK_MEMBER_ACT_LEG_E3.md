# M-RP-MEMBER-ACT — Leg E-3 Runbook: the R1 filter (Clair)
> **Status**: PENDING  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — SEAT, AND WHAT THIS LEG IS

🛑 **NOT LOCKED. DO NOT IMPLEMENT YET.** ✅ **CLAIR'S ADVERSARIAL READ RAN 2026-08-13 — brief `tasks/CLAIR_LEG_E3_RUNBOOK_READ.md`, verdict LOCKABLE WITH TWO NAMED VERIFY CHANGES + wording; swept into this v1.1, every claim re-driven by Chat (Rule 5).** 🔑 **THE BUILD SURVIVED AND BOTH PLAN-MOVERS WERE GATE DEFECTS — AGAIN, AND THEY WERE FOUND BECAUSE THE READ HIT `§5` FIRST.** *`PM-1`: `V7` could not fail. `PM-2`: `§7.1` claimed `F1` was ungated and it is not — `V6` IS the discriminator.* **Nothing further is owed before Joe locks.**

**You are CLAIR — Code Claude.** You implement **once Joe locks it**. **You never push.** Deviations are **reported under Rule 6, never absorbed** — *an implementer who silently absorbs a bad instruction ships the architect's mistake.*

**E-3 removes DM Spaces from R1's RENDER. It does NOT touch the store** — and `F3` is why that is a lock, not a preference. It does not build anything new; `E-1` gave DMs a home and `E-2` guarantees that home is placed.

🔒 **THIS IS A TWO-FILE LEG, STATED UP FRONT (Joe ruled `R-a`)** — the filter, plus **one line** discharging `E-2`'s undriven `V3`.

**Read first:** `tasks/M_RP_MEMBER_ACT_LEG_E3_PHASE0.md` **v1.1** (§2 audit · §3 `F1`–`F5` · §4 both rulings · §5.1) · `tasks/M_RP_MEMBER_ACT_LEG_E2_PHASE0.md` v1.6 · `CLAUDE.md` PLAY head · `JOURNAL.md` **J-726**.

---

## §1 — GROUNDING (measured 2026-08-13 at `3a15112`; re-verify, do not inherit)

📌 **Every `file:line` below came from a tool that printed it.** At Leg E, seven estimated pointers were all wrong (`W1`); at E-2, four more were. **A mismatch is a Rule 6 report, not a silent adjustment.**

| fact | where |
|---|---|
| the local handle | `spaces-panel.svelte:47` — `const spaces = $derived(spacesState.spaces)` |
| 🔒 **the render list — THE FILTER SITE** | `spaces-panel.svelte:50` — `items = spaces.map(...)` |
| 🛑 the DM-highlight suppression (C-bis-6) | `spaces-panel.svelte:58-63` — `s?.counterpart != null ? undefined : id` |
| `onActivate` | `spaces-panel.svelte:65-68` |
| 🛑 the aggregate getter | `spaces-panel.svelte:71-75` — `count: spaces.length`, `hasEmpty: spaces.length === 0` |
| the predicate, already exported | `spaces-state.svelte.ts:43` — `isDmSpace = (s) => s.counterpart != null` |
| the store import line | `spaces-panel.svelte:29` |
| the DEV bridge object | `app_client.svelte:392-404` (`set` `:394` · `move` `:398` · `fold` `:399` · `setBackground` `:403` · closes `:404`) |
| `handleRevertUi` | `app_client.svelte:585-587` |

**LIVE SCREEN, recorded AS A SCREEN (`N-184`/`N-190`/`N-194`):** 7 Spaces of which **3 are DMs** · R1 getter `{count: 7, selectedId: null, hasEmpty: false}` · **14** R1 row entities · DM home `{count: 3}` · registry **184**, `count === unique` · zero saved UI states.
🛑 **NOT A FLOOR. DO NOT CARRY IT.** E-3 measures its own before/after in ONE sitting (§5).

---

## §2 — SCOPE: THE FILES YOU MAY OPEN

1. `ui/common/lib/components/widgets/spaces-panel.svelte` — the filter
2. `ui/client/src/app_client.svelte` — **ONE line** in the `__XGEN_LAYOUT__` object (§3.3)

🛑 **NOT in this leg:** the `spacesState` store (🔒 **`F3` — nine read sites across five files and EIGHT need DMs PRESENT**; touching it breaks the DM home, both latches, `canSend` and member activation in one edit) · `dm-spaces.svelte` (`E-1`) · `layout-default.ts` (`E-2`) · `ui/core/**` (would move the catalogue) · `skin.css` (**Joe's file, never in a Clair commit**) · any `.rs` (🔒 **`cargo` is NOT a floor — §6**).

⚠️ **If E-3 cannot be built inside these two files, STOP AND REPORT.** A scope that has to grow is a finding; one that grows silently is a defect.

---

## §3 — WHAT TO BUILD

### 3.1 The filter (`spaces-panel.svelte`)

Import the shipped predicate — **never an inline `counterpart != null`** (`spaces-state.svelte.ts:40` names that copy as the `D-067` drift surface this export exists to prevent):

```ts
import { spacesState, isDmSpace, type KnownSpace } from '$common/stores/spaces-state.svelte';
```

Add ONE derived beside `:47`, and feed it to the render list and the getter ONLY:

```ts
// E-3 (OQ3 = A3, Joe J-709): DM Spaces leave R1's RENDER. `visible` is the rendered set; `spaces` stays the
// FULL set and both are used deliberately — see the two locks below. The store is never filtered (F3).
const visible = $derived(spaces.filter((s) => !isDmSpace(s)));

// :50 — the render list is the ONLY consumer that switches
const items = $derived(visible.map((s) => ({ descriptor: toDescriptor(s) })));

// :71-75 — the aggregate getter follows the RENDER, not the store (LOCK 2)
const debug = () => ({
  count: visible.length,
  selectedId: selected ?? null,
  hasEmpty: visible.length === 0,
});
```

📌 **All three edits are shown together deliberately** — v1.0 showed only the `visible` line, and an implementer could have added it while leaving `items` and the getter on `spaces`, which is exactly `F2`'s failure. **`selected` and `onActivate` are ABSENT from this block because they must NOT change.**

🔒 **LOCK 1 — `items` FILTERS; `selected` AND `onActivate` DO NOT (`F1`).**

| site | reads | why |
|---|---|---|
| `:50` `items` | **`visible`** | the render list — this is the whole leg |
| `:58-63` `selected` | 🛑 **`spaces` (UNFILTERED)** | it resolves the latched id **in order to recognise a DM and suppress the highlight**. Against a filtered list a latched DM **stops resolving** ⇒ `s` is `undefined` ⇒ `s?.counterpart != null` is **false** ⇒ **the suppression stops suppressing.** *The guard would invert while looking untouched.* |
| `:65-68` `onActivate` | `spaces` (unfiltered) | only rendered rows can fire it; leaving it unfiltered keeps the id→descriptor lookup total |

📌 After E-3 the suppression is **doubly guarded** (no row to light, *and* the latch resolves to `undefined`). **KEEP IT** — removing it is scope creep and would leave R1 handing `entity-panel` an id it does not render.

🔒 **LOCK 2 — THE GETTER FILTERS, OR IT LIES (`F2`).** `debug()` is the primary verify surface. Leave `count: spaces.length` and the panel renders **4 rows while reporting 7** — a probe would then file **FAILURE against correct code** (`N-194`). Both lines move:

```ts
count: visible.length,
selectedId: selected ?? null,
hasEmpty: visible.length === 0,
```

*`W-4`: the aggregate getter reports what the panel OWNS, and after E-3 the panel owns the visible rows.*

### 3.2 What must NOT change in this file

`toDescriptor` (`:43-45`) · the `EntityPanel` mount (`:79`) · `emptyText` · the `selection.set` write · every comment explaining C-bis-6. **No component `<style>`, no `skin.css`** (`N-090`/`N-025`).

### 3.3 The `revert()` bridge line (`app_client.svelte`)

🔒 **ONE LINE, inside the existing `__XGEN_LAYOUT__` object (`:392-404`), discharging `E-2`'s undriven `V3`.** `layout.revert` is a live command that is deliberately **element-absent** (J-500), so no user route exists and **no eval could reach it** — which is why `V3` could not be driven.

```js
// E-3 (J-726 discharger): `layout.revert` is a LIVE command with no interactive element (J-500), so the
// File▸Revert path had no reachable route and E-2's V3 went undriven. This delegates to the SHIPPED handler
// — the same function the command resolves to — so the bridge and the command are ONE path, never two.
revert() { return handleRevertUi(); },
```

📌 Place it beside `fold` (`:399`) — the `move`/`fold` precedent: **both delegate to shell handlers rather than reimplementing them.** ⚠️ `handleRevertUi` is `async`; **return** its promise so a driver can await it.
🛑 **NOTHING ELSE IN `app_client.svelte` MOVES.** Not the commandTable, not `handleRevertUi` itself, not a menu item.

---

## §4 — TYPE + BUILD NOTES

- `spaces-panel.svelte` is **TS**; `app_client.svelte` is **plain JS**. The floor that must not move is `svelte-check` **0/34/15**.
- 🛑 **`__XGEN_LAYOUT__` currently exposes ONLY `current`·`set`·`move`·`fold`·`background`·`setBackground`.** After §3.3 it also exposes `revert`. **`insertLeaf`, `removeRegion` and `DEFAULT_LAYOUT` remain UNREACHABLE from a CDP eval** — they are module imports. ***Three verify commands in this arc named symbols the eval could not reach; do not write a fourth.***
- 📌 `set(l) { layout = l; }` does **NOT** persist and does **NOT** re-inject. `move`/`fold` **DO** persist. `revert()` re-reads disk and **does not persist** (`P-1`).

---

## §5 — VERIFY (drive it; do not predict it)

🛑 **THE PASS CONDITION IS A TRANSITION, NOT A STATE.** *"The DM rows are gone"* is an **EMPTY RESULT** (`N-099`): it reads identically if the filter works, **or** if the probe is looking at the wrong panel, the wrong attribute, or a client that failed to load. **Every gate below states its screen, and the screen is one client, one Space tree, one draft state, one sitting.**

| # | check | how |
|---|---|---|
| **V0** | 🔒 **THE POSITIVE CONTROL, RUN FIRST, ON THE PRE-E-3a BUILD** | **PRINT the ids of the DM rows RENDERED IN R1** — read them from R1's **rendered rows**, never from `spacesState`. 🛑 **`F3` KEEPS DMs IN THE STORE, so a store-anchored probe reads them present AFTER the filter too and looks like failure against correct code.** ⚠️ **Without V0, every later "absent" reading is worthless.** Record all three ids. |
| **V1** | R1 drops the DM rows | R1 getter `count` **7 → 4**; the three V0 ids are **absent from R1's RENDERED ROWS** — **matched against the ids V0 printed**, not a fresh guess. 🛑 **ANCHOR ON THE RENDER, NOT THE STORE** — the DMs are still in `spacesState` and must be (`F3`) |
| **V2** | the getter does not lie (`F2`) | `count` **equals the painted row count**; read BOTH the getter and the DOM |
| **V3** | the DM home is unaffected | DM home getter still `{count: 3}`; the three DMs still listed and still openable |
| **V4** | 🔒 **the store was NOT filtered (`F3`)** | latch a **REAL DM — confirm `counterpart != null` on the latched Space before asserting anything** → `roomLatch.effectiveRoomId` resolves · **`canSend` true**. 📌 **`spaceLatch` has NO direct bridge** — read it through a consumer (R2's rooms populate) or drop the clause; **do not name a reading you cannot take.** *This is what proves the filter is a RENDER filter.* |
| **V5** | the complement, BOTH directions (the E-1 `V5` precedent) | click a non-DM Space → **R1 lights exactly one row**; open a DM from the home → **R1 unlit**, DM home lit |
| **V6** | 🔒 **THE `F1` DISCRIMINATOR — THIS IS THE GATE THAT PROVES LOCK 1 (`PM-2`, Clair)** | Latch a **REAL DM**, then read `spaces-panel#region-spaces`'s **`selectedId`**. 🔑 **`selected` (`:58-63`) has NO dependency on rendering — only on `spaceLatch` and `spaces` — so the GETTER VALUE is SINGLY caused even though the PAINT is doubly caused.** ⇒ **`selectedId: null` iff `F1` is honoured** (the DM resolves, `counterpart != null`, suppressed) · **`selectedId: <the DM's id>` iff `:47` was naively filtered** (the DM does not resolve, `s?.counterpart != null` is **false**, the raw id is returned). **Both outcomes are named in advance; a third is a Rule 6 report.** |
| **V7** | 🔒 **`revert()` — A TRANSITION, NOT A STATE. DISCHARGES E-2's `V3` (`PM-1`, Clair)** | 🛑 **DO NOT ASSERT "`dm-spaces` present after revert" — THAT CANNOT FAIL.** `loadLayout` re-injects unconditionally at its single exit (`layout-default.ts:193`) and `P-1` never persists, so the home is present **whether `revert()` ran or not**. 🔒 **Drive a transition instead:** ① read `session.layout` off disk and record its SHA · ② `__XGEN_LAYOUT__.set(<a visibly different tree — e.g. one leaf folded or an edge weight moved>)` — `set` is a bare reassignment (`:394`), **it does NOT persist** · ③ **CONFIRM the live grid actually changed** (this is the control — without it ④ proves nothing) · ④ `await __XGEN_LAYOUT__.revert()` · ⑤ the live tree is **restored to the disk tree**, and the on-disk SHA is **unchanged**. 📌 **No `$effect` persists `layout`** — measured: all five effects (`:128`/`:145`/`:172`/`:205`/`:221`) and all seven `setSessionLayout` calls are **gesture handlers** ⇒ `set` is safe as the staging vehicle. **NO grid gesture during ②–⑤.** |
| **V8** | registry transition | before → after, **ENUMERATED, not derived** — **"enumerated" means NAMING THE SIX REMOVED IDS**, not observing that a total fell by six. `F4` predicts **−6** (3 rows × `N-184`'s two entities) — 🛑 **a hypothesis to TEST. A coincidental match must not pass: if the six ids you name are not the six that left, the gate FAILS even at −6** (`N-194` — a prediction and an observation once agreed with no mechanism in common) |
| **V9** | floors | `svelte-check` **0/34/15** · catalogue **435 BY SCOPE** (zero `ui/core`) · **no `cargo` claim** |

🔑 **`N-194`, BINDING ON EVERY GATE: A PROBE MUST BE ABLE TO DEMONSTRATE SUCCESS, NOT MERELY THE ABSENCE OF FAILURE.** Before reporting any gate FAILED, ask **what this read would return if the code were RIGHT.** Same answer ⇒ **the probe is wrong.** ⚠️ *At E-2, a control read the wrong KEY and returned nothing — and only survived as a finding because the step printed its values instead of asserting absence.* **Print what you find; do not assert what you don't.**

🛑 **DO NOT SEND A MESSAGE.** A send mints a **permanent DM** in Joe's live client.
📌 **Joe's client state is READ-ONLY.** No `setSessionLayout`, no install, no saved UI state — **E-3 needs none.** Any probe that persists a mutation OWES its cleanup, and the cleanup is part of the probe (`N-123`).

---

## §6 — DEFINITION OF DONE

- [ ] `visible` derived from `isDmSpace` (**imported, never inlined**); `items` reads it
- [ ] 🔒 `selected` and `onActivate` still read the **UNFILTERED** `spaces` (`F1`)
- [ ] 🔒 `debug().count` **and** `hasEmpty` read `visible` (`F2`)
- [ ] `revert()` on `__XGEN_LAYOUT__`, **delegating to `handleRevertUi`**, returning its promise
- [ ] **zero store change · zero `ui/core` · zero `.rs` · zero `skin.css` · no component `<style>`**
- [ ] **V0 run FIRST, ANCHORED ON R1's RENDERED ROWS (never the store), and its three ids recorded** — an empty result with no control is not a pass
- [ ] **V1–V9 driven, each with its screen stated**; **`V6` reports which of its two named `selectedId` outcomes it saw**; **`V7` shows the transition, not the state**; **V8 NAMES THE SIX REMOVED IDS**
- [ ] `svelte-check` 0/34/15 · catalogue 435 by scope · **no `cargo` claim made**
- [ ] deviations reported (Rule 6)
- [ ] 🔓 hand-back names anything the filter makes visually odd — **Joe's, `S-a` ships silent, no hint text**

---

## §7 — WHERE THIS RUNBOOK IS MOST LIKELY WRONG

1. ✅ ~~**`F1` IS REASONED FROM SOURCE, NOT DRIVEN — and `V6` cannot distinguish the two guards, so if `F1` is wrong no gate here would catch it.**~~ 🛑 **RETRACTED, NOT HEDGED (`D-111`), BY CLAIR'S `PM-2` — THE CLAIM WAS FALSE AND IT WAS THE MOST LOAD-BEARING SENTENCE IN THE DOCUMENT.** `V6` **IS** the `F1` discriminator: `selected` (`:58-63`) depends only on `spaceLatch` and `spaces`, **never on rendering**, so `debug().selectedId` (`:73`) is **singly caused** — `null` iff `F1` is honoured, **the DM's raw id** iff `:47` was naively filtered. 🔑 ***The author conflated the doubly-caused PAINT with the singly-caused GETTER VALUE*** — and the mechanism was written in a comment **two lines above** the code he reasoned about (`:56-57`: *"A stale id that no longer resolves in `spaces` keeps highlighting the raw id… (undefined `s` → not suppressed)"*). **`N-180` at arm's length: the source was read, the comment ON it was not.** ✅ **`F1` is now GATED. `V6` is the gate.**
2. **`F4`'s −6 is arithmetic.** `N-194`: a predicted number and an observed number once agreed with **no mechanism in common**.
3. **`visible` vs `spaces` may read as over-shaped in a 40-line component.** The honest counter is `F1`: they serve **opposite purposes**. If it feels wrong while implementing, that is a Rule 6 report.
4. **§3.3 widens the leg to two files for one line.** Ruled `R-a` deliberately — but a rider is a rider, and if it grows past one line **STOP AND REPORT** rather than absorbing it.
5. ✅ ~~**This runbook has not been read by anyone but its author.**~~ **READ 2026-08-13 (Clair).** 🔑 **THE BUILD SURVIVED AND BOTH PLAN-MOVERS WERE GATE DEFECTS — THE THIRD CONSECUTIVE LEG WHERE THAT IS TRUE, AND ALL OF THEM WERE THE AUTHOR'S.** *`E-2`: `PM-1` a probe that could not fail, `Q2` a control asserting absence from a value it never printed. `E-3`: `V7` a probe that could not fail, `§7.1` a false claim that the central lock was ungated.* ✅ **Reading `§5` FIRST, cold, is what surfaced both** — the inversion was deliberate and it worked.
6. 🛑 **AND THE REMAINING SOFT SPOT IS `V7`'s NEW ROUTE.** `§5`'s `V7` is now the newest text in the document and was written **after** the read that fixed it, so **nobody has attacked it.** Its `set`-safety rests on a measured absence (no `$effect` persists `layout`) — **an absence is evidence, not proof.** If step ③ shows no visible change, **STOP AND REPORT**; do not proceed to ④ and call the restoration a pass.
