# Clair — adversarial read of RUNBOOK_MEMBER_ACT_LEG_E3.md v1.0 (M-RP-MEMBER-ACT Leg E-3)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS IS

A read, not an implementation. **No code written, no source edited, no app launched, no message sent, no DM minted, no file re-annotated.** Every `file:line` below came from a tool that printed it **this session at HEAD `6d9ed5d`**, not from the runbook and not from memory.

🛑 **State at open, re-measured, not inherited:** `git status` **clean**; `git rev-parse HEAD` = **`6d9ed5d`**; `git ls-remote origin refs/heads/main` = **`6d9ed5d`** — identical, not the tracking ref.

🔑 **The read was pointed at §5/§6/§7 FIRST, cold, before §3** — the E-2 lesson (both real defects were in the VERIFY half). It paid off: **the two plan-movers are both gate defects, not code defects,** exactly as at E-2.

---

## §1 — VERDICT

**LOCKABLE WITH TWO NAMED VERIFY CHANGES, plus wording.**

**The BUILD (E-3a) is lockable as written. I could not break it.** The design is sound inside the two-file scope, and — unlike Leg E (seven wrong pointers) and E-2 (four) — **every `file:line` in §1 is exact.** The two-name `visible`/`spaces` split is justified, not over-shaped. `F1`, `F2` and `F3` are all correct against source.

**The VERIFY (E-3b) carries two holes, both on gates, both the same species this arc keeps shipping — a probe that passes whether or not the mechanism works:**

- **`PM-1` — V7 cannot fail as written.** It drives the new `revert()` line, but the thing it checks (`dm-spaces` present after revert) is true whether `revert()` ran or not, because the live grid *already* holds `dm-spaces`. As written, **V7 does not discharge E-2's V3** — which is the entire reason §3.3 widens the leg to a second file. Until V7 stages a transition, the second file buys nothing.
- **`PM-2` — the runbook says F1 is ungated; it is not.** §7.1 and the V6 row claim "if F1 is wrong, no gate here would catch it." **False.** V6's own instrument — `debug().selectedId` with a DM latched — returns `null` iff F1 is honored and the DM's own id iff F1 is violated. V6 IS the F1 discriminator; the runbook talks itself out of its own best gate.

Neither is "not lockable." Both are one-paragraph verify fixes. **The design does not move; the proofs of the design do.** That is the E-2 pattern to the letter, and it is why the read was worth its cost.

---

## §2 — PLAN-MOVING (gates first, per the brief)

### 🛑 PM-1 — V7 IS PM-1 REINCARNATED: IT CANNOT FAIL, AND IT DOES NOT DISCHARGE E-2's V3.

**What V7 is for.** §3.3 adds `revert()` to `__XGEN_LAYOUT__` (`app_client.svelte:392-404`) specifically to give an eval a route to `handleRevertUi` (`:585-587`) so that E-2's undriven **V3** — "does the re-inject fire on the File▸Revert path?" — can finally be driven. V7 (§5, runbook line 138) is the gate that is supposed to discharge it: `revert()` → grid re-reads disk → **`dm-spaces` still present** → `session.layout` unchanged.

**Why it cannot fail.** Measured, not argued:

- `handleRevertUi` (`app_client.svelte:585-587`) is `layout = await loadLayout(mountedPlugins)`.
- `loadLayout` (`layout-default.ts:177`) ends **unconditionally** with `return reinjectSystemRegions(loaded, plugins)` (`:193`), and the re-inject is "idempotent + TOTAL" (`:192`).
- E-2's **P-1** means the re-inject **never persists** (a read path stays a reader). So after boot, the live grid **already holds `dm-spaces`** (injected on the boot `loadLayout`), while disk's `session.layout` does not.

⇒ Calling `revert()` re-runs `loadLayout` → re-injects → a grid that **still holds `dm-spaces`** — **identical to the grid that was already on screen.** V7's assertion "`dm-spaces` still present" is therefore **true whether `revert()` ran, no-op'd, or was wired wrong** (`revert() {}`, `revert() { handleRevertUi; }` missing the call, etc). **This is `N-194`'s exact shape and E-2's own `PM-1` one leg later** — a probe that reports SUCCESS by the absence of a change that was never going to happen.

**What if `revert()` were wired wrong?** V7 would read the *same* "`dm-spaces` present" and **PASS**. The only falsifiable half of V7 as written is "disk unchanged" — but that proves `revert()` did not *persist*, not that it *re-read disk*. Nothing in V7 as written proves `loadLayout` ran, so nothing discharges V3.

**The fix — stage a transition (the V0 discipline, applied to V7).** `set` does **not** persist (`app_client.svelte:394`, confirmed: `set(l) { layout = l; }`), so it is disk-safe:

1. read `__XGEN_LAYOUT__.current` → capture a distinctive structural feature (a split's `sizes`, or the leaf set);
2. `__XGEN_LAYOUT__.set(mutatedClone)` — a clone with that feature changed (e.g. swapped split weights) — **not `move`/`fold`, which DO persist (§4)**;
3. confirm the live grid reflects the mutation (getter/DOM);
4. `revert()`;
5. confirm the grid **returns to disk's arrangement** (mutation gone) AND `dm-spaces` present AND `session.layout` byte-unchanged.

The **absent→present / mutated→restored** transition (2→3, 4→5) is what makes V7 falsifiable and what actually proves `loadLayout` re-read disk — the only thing that discharges V3. If `revert()` no-op'd, the mutation would survive step 5 and V7 would fail.

🔑 **Strongest form, if the eval can build it:** remove the `dm-spaces` leaf from a clone of `current`, `set` it, confirm `dm-spaces` **gone**, `revert()`, confirm it **comes back** — a direct observation of the re-inject. `removeRegion` is unreachable (§4), so this needs hand-tree-surgery in the eval; the sizes-swap proxy above is the safe, sufficient fallback and proves `loadLayout` ran (hence, by `:193`, the re-inject ran). Either way, **`revert()` must be driven across a state it can undo, or it proves nothing.**

⚠️ **Consequence for §7.4 / the two-file scope:** the runbook's stated justification for opening `app_client.svelte` at all is "discharge V3, second reachable load path." **That justification only holds once V7 is falsifiable.** As written, the second file adds a `revert()` that no gate exercises — the rider costs a file and buys an undriven line. Not a reason to reject `R-a`; a reason to fix V7 before the lock.

### 🛑 PM-2 — §7.1 IS WRONG: F1 IS GATED, AND V6 IS THE GATE.

**The claim under attack.** §7.1 item 1 (runbook line 166): *"V6 cannot distinguish the two guards once both hold, so if F1 is wrong, no gate here would catch it."* The V6 row (line 137): *"say plainly that it is now doubly guarded, so this gate cannot distinguish the two guards."* The whole two-name design is declared **unverifiable by its own gates**, to be defended by reasoning alone.

**It is verifiable, and the runbook's own V6 instrument does it.** Traced against `spaces-panel.svelte:58-63` and `:71-75`:

`selected` (`:58-63`):
```
const id = spaceLatch.latchedSpaceId;      // a DM's space_id, latched from the home (dm-spaces:114)
if (id == null) return undefined;
const s = spaces.find((x) => x.space_id === id);
return s?.counterpart != null ? undefined : id;
```
`debug().selectedId` (`:73`) = `selected ?? null`.

- **F1 honored (correct build: `:47` unfiltered, `visible` feeds only `items`):** `spaces` still holds the DM → `s` found → `counterpart != null` → returns `undefined` → **`selectedId === null`**.
- **F1 violated (naive `:47` filter: `const spaces = $derived(spacesState.spaces.filter(s => !isDmSpace(s)))`):** `spaces` no longer holds the DM → `s` is `undefined` → `undefined?.counterpart != null` is **false** → returns **`id`** (the DM's own space_id) → **`selectedId === <the DM's id>`, non-null.**

⇒ **V6 reading `debug().selectedId` with a DM latched returns `null` under the correct build and a specific non-null id under the naive F1 violation. It discriminates.** The V6 row *literally instructs reading `selectedId`* — so the runbook's own gate is the discriminator, and the "cannot distinguish" clause in the same row **contradicts the instrument the row specifies.**

**Where the author went wrong — a real distinction, misapplied.** The *painted* highlight (a DOM `[aria-selected]` read) IS doubly caused: even with the suppression removed, no DM row is rendered, so nothing paints. But **`selected` is computed only from `spaceLatch` + `spaces` + `counterpart` (`:58-63`) — it has no dependency on `items`/rendering.** So the *getter* `selectedId` is **singly** caused by the suppression (Guard B); Guard A (no row rendered) does not touch it. The author conflated the doubly-guarded *paint* with the singly-guarded *getter value.* V6 reads the getter → tests Guard B alone → catches a wrong F1.

**The fix.** Two words of substance:

1. **V6 reads `debug().selectedId` (the getter), asserts `=== null` with a DM latched** — and the runbook stops saying F1 is ungated. That alone upgrades F1 from "reasoned, not driven" to "driven, and it is the F2-of-F1" (V2 gates F2 the same way — getter vs render; V6 gates F1 — getter vs the raw id).
2. **Optional positive control** (the E-2 discipline): momentarily filter `:47`, observe `selectedId` becomes the DM id, revert. This proves the probe can fail. It is a source edit, so it belongs in E-3b only if Joe wants the control; the minimum (item 1) already makes F1 a live guard.

🔑 This is **good news for the runbook** — the milestone's central lock is testable after all — which is exactly the kind of finding that survives scrutiny. The design does not change; the claim that it cannot be checked does.

---

## §3 — WORDING / SHARPENING (not plan-movers)

### W-1 — V0/V1: anchor on the RENDER, and warn off the store, because the store is the trap.
`F3` is correct and load-bearing: **the store is never filtered, so `__XGEN_SPACES__.spaces` returns all 7 (3 DMs) before AND after E-3.** A prober who reaches for `__XGEN_SPACES__` in V1 out of habit would find the 3 DMs "still there" and file a false FAILURE (`N-194`), or mismatch a store-read V0 against a render-read V1. The runbook's intent is render-anchored ("rendered rows", "R1 getter") — but it never *names the trap*. Add to V0: the "id" recorded is the **row's registry id** `region-spaces__panel-<itemKey>` (`entity-panel.svelte:96`, `itemKey = xgid.split('/').pop()`, `:95`), derived from the DM's `space_id` and confirmed **present in the pre-build DOM/registry**; V1 asserts those exact registry ids **absent** after. State plainly: **the DM-absence check reads the RENDER, never `__XGEN_SPACES__` (F3 keeps DMs in the store by design).**

### W-2 — V8: "enumerated" must mean the six removed ids, not a −6 size delta.
`F4`'s −6 warning is right (`N-194`: a predicted and observed number once agreed with no mechanism in common). Make it bite: V8 must **list the actual removed registry entries** and confirm they are the **`entity-item` + `entity-avatar` pair for each of the 3 DM rows** (`N-184`), not merely observe `184 → 178`. A −6 count delta with the wrong six entities removed would pass a size check and be wrong.

### W-3 — V4: confirm the latched Space is genuinely a DM.
V4 proves "store not filtered" by latching **a DM** from the home and reading `canSend` true (`__XGEN_ECHO_BRIDGE__.room`, `app_client.svelte:461-463`). A **non-DM** latch would also give `canSend` true and prove nothing about DMs. Add: confirm the latched space has `counterpart != null` (it is the DM you meant), and note V4's power **depends on V3 passing first** — an empty home (store filtered) has no DM to latch, so V4 and V3 are one argument, not two.

### W-4 — §3.1's code block shows only `visible`; the three edits it drives are in the LOCK tables, not the block.
The snippet (runbook lines 71-75) declares `visible` but does not show the three edits it forces: `items` (`:50`) → `visible.map(...)`, and `count`/`hasEmpty` (`:72`/`:74`) → `visible.length`. LOCK 1, LOCK 2 and the DoD carry them, so it is complete — but a reader taking the code block literally would add `visible` and change nothing else. One line naming all four edits (import `isDmSpace` at `:29`; add `visible`; repoint `items`; repoint `count`+`hasEmpty`) would close it.

### W-5 — "spaceLatch resolves" (V4) has no direct bridge.
`__XGEN_ECHO_BRIDGE__.room` exposes `roomLatch`'s `effectiveSpaceId` (`:462`), not `spaceLatch.latchedSpaceId` directly. The eval reads `spaceLatch` only **indirectly** via the panel getters (`spaces-panel`/`dm-spaces` `selectedId`, both computed from `spaceLatch`). Reachable, but V4's "spaceLatch resolves" should name the surface it is read through, so it is not mistaken for a bridge that does not exist.

---

## §4 — SURVIVED — VERIFIED FROM SOURCE, DO NOT RELITIGATE

- **Every §1 pointer is exact** (checked at `6d9ed5d`): `:47` `spaces`, `:50` `items`, `:58-63` `selected`, `:65-68` `onActivate`, `:71-75` `debug` (`count:` `:72`, `hasEmpty:` `:74`), `:29` import, `spaces-state.svelte.ts:43` `isDmSpace` and `:40` the `D-067` comment, bridge `:392-404` (`set` `:394`, `move` `:398`, `fold` `:399`, `setBackground` `:403`, close `:404`), `handleRevertUi` `:585-587`, `commandTable['layout.revert']` `:612`.
- **F3 is exactly right — 9 read sites across 5 files, 8 need DMs present.** Grepped `spacesState`: `room-latch:51` · `dm-spaces:69/92/103` · `space-latch:59` · `members-panel:156/227/285` · `spaces-panel:47`. `app_client:256` is `setSpaces` (a write, correctly excluded). `spaces-panel:47` is the one exception (it removes DMs at render); the other eight need DMs in the store. **Touching the store breaks the home, both latches, `canSend` and member activation in one edit — the lock is real.**
- **F1's mechanism is correct** (traced above): filtering `:47` inverts the `:62` suppression (returns the raw id). The two-name split is the right answer, not over-shaping — `visible` DRYs one filter across `items`+`count`+`hasEmpty`; the alternative is three duplicated inline filters. **§7.3 is a non-issue.**
- **F2 is gated by V2 and V2 is well-built** — getter (`visible.length`) vs painted rows; an unfiltered getter (`spaces.length=7` vs 4 rendered) fails it. This is the model PM-2 asks V6 to follow.
- **`revert()`'s "one path, never two" claim holds:** the bridge `revert()` returns `handleRevertUi()`, and `commandTable['layout.revert']` (`:612`) resolves to the **same** `handleRevertUi`. Bridge and command are genuinely one function.
- **Hoisting is fine:** `async function handleRevertUi` (`:585`) is a hoisted function declaration; referencing it in the bridge closure at `:392` (executed only when `revert()` is called) is sound.
- **No unreachable-symbol gate (brief ⑥ clean):** every gate read resolves — `__XGEN_SPACES__` (spaces-state:63), `__XGEN_ECHO_BRIDGE__.room` (app_client:451), `__XGEN_DEBUG__`, `__XGEN_LAYOUT__.current`/`set`/`revert` (after §3.3). V5 drives clicks via the harness (`-Mode click`, the E-1 V5 precedent), not a symbol. §4 correctly names `insertLeaf`/`removeRegion`/`DEFAULT_LAYOUT` as unreachable and **no gate needs them.** No fourth unreachable-symbol command ships.
- **No svelte-check movement expected:** `visible` is `KnownSpace[]`, `isDmSpace` is a typed exported const, `.map`/`.length` sources change only; the JS bridge line is untyped. `0/34/15` should hold.
- **Catalogue 435 by scope is honest** — zero `ui/core` opened; it is a scope argument and the runbook says so.

---

## §5 — WHERE THIS READ IS MOST LIKELY WRONG

1. **PM-2 hinges on V6 reading the getter, not the paint.** I read the V6 row's word "`selectedId`" as the getter (`debug().selectedId`), which discriminates. If the author *intends* V6 to read the DOM `[aria-selected]` highlight, then "cannot distinguish" is defensible (both builds paint nothing), and my finding softens to "V6 is ambiguous about its surface; specify the getter and it becomes the F1 discriminator." Either way there is a change to make — but the strength of "the runbook contradicts itself" depends on the getter reading, which the row's own wording supports.
2. **PM-1's fix assumes the eval can safely stage a mutation via `set`.** I confirmed `set` does not persist (`:394`) and `revert()` re-reads disk, so the round-trip is disk-safe. If some reactive effect persists on a bare `set` reassignment (I did not exhaustively trace every `$effect` watching `layout`), the staging could touch disk — in which case the mutation must be smaller or the cleanup (N-123) made explicit. I believe `set` is inert, but I did not prove no effect fires on it.
3. **The registry-id matching in W-1 assumes `itemKey` uniqueness across spaces.** `itemKey = xgid.split('/').pop()` collides if two space_ids share a last path segment. Space_ids are hashes, so this is near-impossible, but the before/after id match relies on it.
4. **I did not launch the app.** Every finding is from source and from the arc's recorded bridge behavior. The runbook's own §7.1 warning stands: F1 has not been *observed* — and PM-2 is precisely the claim that it *can* be, which itself wants driving to confirm the selectedId values I traced.
