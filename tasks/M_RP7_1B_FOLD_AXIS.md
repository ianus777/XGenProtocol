# M-RP7.1b — the fold axis becomes the user's choice; splits shrink-wrap; the hole gets a floor
> **Status**: PENDING  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Read first

**Canonical stack for this milestone, in order:** `CLAUDE.md` PLAY block → `JOURNAL.md` head (**J-515**) → **`docs/xgen-dock-engine-phase0.md` v1.3 §4.1 · §4.2 · §4.3 · §4.4 · §4.5 · §7.1 · §11 IN FULL** → this runbook. **§4.1-H is HISTORY — the derived-axis rule it describes is what you are replacing. Do not implement it.**

**Lanes.** You hold **no design authority** (dock Phase-0 §0 — Joe granted Chat the mechanics, Joe keeps the graphical appearance). Deviations are **flagged, not absorbed** (Rule 6). **Chat re-drives every verification leg independently** (Rule 5) — your numbers do not enter the record; the conclusions you reach may.

**Appearance is JOE'S and it is WIDE (N-090):** stripe height, chevron shape, strip width, the raster's pattern/weight/colour, spacing. **Ship the raster PLAIN and OBVIOUSLY PROVISIONAL.** Do not tune it. Do not defend it.

---

## 1. What this milestone is

M-RP7.1 shipped `collapsed?: boolean` with the fold axis **derived from the parent split**. **Joe looked at it on screen and replaced the design** (J-515). Two things came out of that, and **they ship together or not at all**:

1. **The user picks the fold axis** — two buttons (`[<]` fold-to-left, `[v]` fold-to-top).
2. **A split shrink-wraps when all its children fold across it** — which is what actually gives the freed space back.

> ### ⚠️ **(1) WITHOUT (2) IS STRICTLY WORSE THAN WHAT SHIPS TODAY.**
> Fold a tile `[<]` inside a `col` split with no shrink-wrap and you get **a thin strip beside a huge hole**, in a column that did not get any narrower. **Do not hand back a leg that has (1) and not (2).** *Shipping the disease and the cure a week apart is how a bad appearance gets defended.*

And **holes** — which **already exist in the shipped build** (fold every child of a split and it under-fills; N-111) — **get a painted floor** so the user knows the empty band is system area, not a broken widget.

---

## 2. 🔑 Grounded findings you must not re-derive

**These were measured on 2026-07-13. Trust them; verify them if you like; do not re-invent them.**

1. **⚠️ `migrate` DOES NOT EXIST.** Grepped `ui/**`: the word appears **only in comments** (`types.ts:16-17`, `layout-default.ts:63`). **The `version` field has been bumped twice and there has never been a migrate function at all.** → **you are CREATING it, not extending it.**
2. **⚠️ A LAYOUT CAN ALREADY BE ON DISK.** `app_client.svelte:227` — `uiStateStore.save(name, { layout: $state.snapshot(layout), … })`. **Named UI states have carried layouts since M-RP6.1k.** Today Joe's `xgen-client_uistate.json` has `named: {}` (measured), so there is **nothing to migrate right now — by LUCK.** **One click on the diskette changes that.** → **the migrate must be REAL, and it must be EXERCISED.**
3. **The tile registers; its buttons (as shipped) do not.** **Do not assume this survives your edit — MEASURE the registry, never derive it** (N-105 / N-108 / **N-112**).
4. **The client CDP bridge WRAPS getters:** `get(id)` returns `{type, state}`. Read **`get(id).state.foo`**. Reading `get(id).foo` returns **all nulls, which looks exactly like a broken build** (N-114).
5. **`cargo test` must stay `1517/0/62` IDENTICAL** — *the inverse leg: identical PROVES no Rust landed.* Run it **DETACHED** and **assert the final `test result:` line exists before reading the numbers** (N-114 — a killed run leaves a measurement-shaped artifact).

---

## 3. The descriptor change — `collapsed` becomes a DIRECTION

**`ui/core/lib/components/layout/types.ts`**

```ts
export type FoldAxis = 'width' | 'height';

export type LayoutNode =
  | { type: 'leaf'; widgetId: RegionId; collapsed?: FoldAxis }   // ← was `collapsed?: boolean`
  | { type: 'split'; dir: 'row' | 'col'; sizes: number[]; children: LayoutNode[] }
  | { type: 'tabs'; active: number; children: LayoutNode[] };
```

- **`collapsed` names WHAT IS COLLAPSED, not where the strip goes** — because the strip's position is not a free choice: it **parks at the tile's leading edge** (§4.1). `'width'` ⇒ vertical strip at the left. `'height'` ⇒ horizontal stripe at the top.
- **Absent = expanded.** Unchanged.
- **`'width' | 'height'` and NOT `'left' | 'top'`** — deliberately. **A direction vocabulary invites a fourth value later** (`'right'`, `'bottom'`), and §4.1 locks **two axes, not four directions.** *Name the thing you mean and the wrong extension stops being sayable.*
- **`version: 3`** in `layout-default.ts`.

---

## 4. The migrate — CREATE it, and FEED it

**New export in `layout/resolve.ts`** (beside the pure walk — it is DOM-free and belongs with it):

```ts
/** v2 → v3: `collapsed: true` carried no axis; it was DERIVED from the parent split's `dir`.
 *  Make it explicit. col divides height → 'height'. row divides width → 'width'.
 *  The root has no parent; M-RP7.1's region-node defaulted it to 'col' → 'height'. */
export function migrateLayout(raw: unknown): Layout
```

**Rules:**
- `version >= 3` → return as-is.
- `version <= 2` → walk the tree; for every `leaf` with `collapsed === true`, set `collapsed = parentDir === 'row' ? 'width' : 'height'` (root ⇒ `'height'`). `collapsed === false` / absent ⇒ **delete the key** (absent means expanded; **do not write `false`**).
- **Never throws.** A shape it cannot read ⇒ `DEFAULT_LAYOUT` (the N-095 rule — *fall back to the default, never to a blank centre*).
- Call it from **`loadLayout()`** in `layout-default.ts`, and from the named-state load path in `app_client.svelte` (`if (s?.layout) layout = migrateLayout(s.layout)`).

> ### ⚠️ **DoD — IT IS EXERCISED, NOT ASSERTED.** Vitest, in `resolve.test.ts`, against **hand-built `v2` trees**:
> - a `collapsed: true` leaf under a **`col`** parent → becomes `'height'`
> - a `collapsed: true` leaf under a **`row`** parent → becomes `'width'`
> - a `collapsed: true` leaf **at the root** → becomes `'height'`
> - a `collapsed: false` leaf → **key deleted**, not `false`
> - a `version: 3` tree → **returned untouched** (idempotent)
> - garbage → `DEFAULT_LAYOUT`, no throw
>
> **This is the first migrate this project has ever run. An unfed branch is an unverified branch (N-091), and this branch has been unfed since D-103.**

---

## 5. The tile — two buttons

**`region-tile.svelte`**

**Props:** `collapsed?: FoldAxis` (was `boolean`) · `axis: 'row' | 'col'` (the **parent split's dir**, unchanged) · `flex?: number` · `onFold?: (regionId: string, collapsed: FoldAxis | undefined) => void`.

**Derived — computed, never stored:**

```
foldMode = collapsed === undefined
  ? 'none'
  : ((collapsed === 'height' && axis === 'col') || (collapsed === 'width' && axis === 'row'))
    ? 'along'    // the fold runs ALONG the parent's dividing axis → siblings absorb → NO hole
    : 'across'   // the fold runs ACROSS it → nobody can absorb → HOLE (§4.5)
```

*(A `col` split divides **height**, so folding **height** is ALONG it. A `row` split divides **width**, so folding **width** is ALONG it.)*

**Reflected on the tile root:**
- `data-collapsed="width|height"` — **absent when expanded**
- `data-axis="row|col"` — the parent's dir (**unchanged from M-RP7.1**)
- `data-fold-mode="along|across"` — **new**

**Stripe DOM order — UNCHANGED and this now matters (§4.3):** `[move-grip · title · fold-width · fold-height]`.

- **Two `<button>`s.** Same class family (`region-tile-fold`), distinguished by **`data-fold="width"` / `data-fold="height"`**.
- **Both are ALWAYS PRESENT** — folded or not. That is what keeps the rotated strip's content identical to the unfolded stripe's.
- **When folded, the OTHER button is `aria-disabled="true"`** (the `shelf-face`/`menu-item` pattern — **`aria-disabled`, NOT native `disabled`**, so it stays keyboard-reachable). The **matching** button unfolds (`onFold(regionId, undefined)`).
- **⚠️ Do NOT render `aria-disabled="false"`.** `shelf-face` renders `aria-disabled={disabled || undefined}` — **absent when enabled**. Follow it.
- The **resize triangle stays ELEMENT-ABSENT when collapsed** (§4.3.2, unchanged).
- Grip + triangle stay **painted-dead** (`aria-hidden`, no handler, no role, no tabindex, no cursor). **Unchanged. Do not wire them.**

**Inline `flex` (the one skin exception):**
- `foldMode === 'none'` → `flex: {n} 1 0` (as today)
- `foldMode === 'along'` → **omit `flex` entirely** (as M-RP7.1 does for `collapsed`) so the skin can pin the tile to stripe size
- `foldMode === 'across'` → **KEEP `flex: {n} 1 0`.** The tile still owns its share of the parent's main axis; only its **cross** axis collapses. **The skin does the rest** (`align-self: flex-start` + pin the collapsed dimension).

**Getter G:** `() => ({ regionId, title, collapsed, axis, foldMode })` — `collapsed`/`foldMode` are **render truth**.

---

## 6. The split — shrink-wrap (§4.4)

**`region-node.svelte`**, on the `split` branch:

```
shrinkWrap = node.children.length > 0 && node.children.every(
  c => c.type === 'leaf' && c.collapsed === (node.dir === 'col' ? 'width' : 'height')
)
```

*(A `col` split's **across** axis is width; a `row` split's is height.)*

- `shrinkWrap` → **omit the split's inline `flex`** (it takes `flex: 0 0 auto` from the skin) → **it shrink-wraps to its children's strip size, and its siblings absorb the freed weight by the `flex` they already have.**
- Reflect **`data-shrinkwrap`** on `.region-split` so the skin can pin it.
- **Unfold ONE child → the condition fails → the split re-inflates. Weights are never mutated.**

> **⚠️ SCOPE, AND IT IS DELIBERATE: `every` requires the children to be LEAVES.** A split containing a nested split does **not** shrink-wrap in this leg, even if everything inside it is folded. **Recursive shrink-wrap is FILED, not built** (§4.4's open mechanic). **Do not extend it. Flag it if it bites; do not fix it.**

> **⚠️ MIXED FOLDS DO NOT SHRINK-WRAP, AND THAT IS CORRECT.** Fold one child `[<]` and one `[v]` → the column stays wide and a hole opens. **The raster explains it. No magic, no guessing what the user meant.**

---

## 7. The raster (§4.5) — skin only

The hole is **flex leftover space inside a split — not an element.** So:

- **A background on `.region-split`**, showing through wherever no tile covers it.
- **⚠️ Verify the tiles are OPAQUE.** If `.region-tile` has a transparent background, the raster bleeds through it and the whole thing is wrong. **Check the computed style; do not eyeball it.**
- **Zero new DOM. One skin rule.**
- **Ship it PLAIN AND OBVIOUSLY PROVISIONAL.** It is Joe's to tune, and he cannot tune it until he has seen holes under it.

---

## 8. Seam updates

- **`region-node.svelte`** — pass `collapsed` through unchanged; compute + apply `shrinkWrap`; thread `axis={node.dir}` (unchanged).
- **`app_client.svelte`** — `handleFold(regionId, collapsed: FoldAxis | undefined)`; `setLeafCollapsed` writes the direction (or **deletes the key**). Still a **pure rebuild**, never an in-place proxy mutation. **Still MEMORY-ONLY this leg** — the session feeder is **M-RP7.5**.
- **`layout-default.ts`** — `version: 3`; `loadLayout()` runs `migrateLayout`.

---

## 9. Definition of Done

- [ ] `types.ts` — `collapsed?: FoldAxis`; `version: 3`.
- [ ] `migrateLayout` **created** in `resolve.ts`, wired into `loadLayout()` **and** the named-state load path.
- [ ] **Vitest: all six migrate cases above, fed with hand-built trees.** Plus the existing `resolve.test.ts` suite still green.
- [ ] `region-tile` — two buttons; `data-collapsed` / `data-axis` / `data-fold-mode`; `aria-disabled` on the unused one (**absent, never `"false"`, when enabled**); triangle still absent when folded; grip + triangle still painted-dead; `flex` omitted only for `along`.
- [ ] `region-node` — `shrinkWrap` computed + `data-shrinkwrap` reflected + `flex` omitted when it fires.
- [ ] Skin — the two chevron orientations (**convention B**, and **⚠️ NO double-rotation inside the rotated strip**); `align-self` + cross-pin for `across`; `flex: 0 0 auto` for `along` and for a shrink-wrapped split; the **provisional raster** on `.region-split`.
- [ ] `cargo test` **1517/0/62 IDENTICAL** (detached; **terminator asserted**).
- [ ] `npm test` (in **`ui/sampler`**) — green, count reported.
- [ ] `vite build` (in **`ui/client`**) — green, module count reported.
- [ ] **Sampler catalogue 328 UNCHANGED** (by scope — no sampler cell for a frame component).
- [ ] **Client registry MEASURED, not derived** — and the report **states the store state, the selection state AND the fold state** (N-112).

## 10. Hand-back — what Chat re-drives (Rule 5)

**Report your conclusions and your evidence. Chat re-drives every leg on the real client (9222) and only Chat's numbers enter the record.**

The legs Chat will run — build so they pass:

1. **`[v]` in a `col`** (self-panel) → `foldMode: 'along'` → **siblings absorb, NO hole**, column width unchanged.
2. **`[<]` in a `col`** (self-panel) → `foldMode: 'across'` → **hole appears, raster visible** on the computed background.
3. **`[<]` on BOTH children of the left column** → **the column SHRINK-WRAPS**; the `2/12` returns to the stream; **ratios re-measured**; **NO hole**.
4. **Mixed (`[<]` + `[v]`)** → **no shrink-wrap, hole present.** *(The honest negative. It must be reachable.)*
5. **The disabled button** — folded ⇒ the other button is `aria-disabled="true"` **and keyboard-reachable**; the matching button **unfolds**; exact return to baseline.
6. **Migrate** — a hand-built `v2` tree with `collapsed: true` under both parent kinds, pushed through `__XGEN_LAYOUT__`, renders the **correct** directions.
7. **The bus survives** (Phase-0 §9): **fold R3, drive R8** — the selection loop still works. *An unfed branch is an unverified branch, and this is the arc's only proof the mechanics do not lean on their content.*
8. **Accent-neutral**, `readable:true` asserted **before** any comparison (N-099 / N-110).

---

## 11. Filed, NOT in this leg

- **Recursive shrink-wrap** (a split of splits) — §4.4's open mechanic.
- **The rotation-direction user setting** — §4.3.1: no settings mechanism exists; it lands with the milestone that creates the `theme` key. **Reserve NO key, NO prop, NO control.**
- **The `mergeClasses` dedupe sweep** (N-113) — **its own milestone, NEVER a rider.**
- **Persistence** — M-RP7.5. **This leg is memory-only and says so by doing nothing.**
- **The grid lock** — M-RP7.6. **Do not add a lock check anywhere.**
