# M-RP7.3 — The mutation algebra (pure): fix the address, migrate `fold`, build `move`
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-14  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

`mutate.ts` becomes the **complete** pure write algebra for the dock: `resizeSplit` (exists) · `foldLeaf` (migrated out of the shell) · `move` (new). **Vitest, no DOM, no gestures.** M-RP7.4 gives it a pointer.

**🔒 It opens by fixing N-120 — a REQUIRED leg, not a filed item.** *A misaddressed `resize` nudges two integers. A misaddressed `move` relocates a panel into the wrong branch.*

**Scope.** `ui/core/lib/components/layout/` — `mutate.ts` · `mutate.test.ts` · `resolve.ts` · `region-node.svelte` — plus `ui/client/src/app_client.svelte` (leg 2 removes code from it; leg 4 adds a DEV driver). **No Rust. No sampler. No `skin.css`. No descriptor schema change** (`version` stays **3**).

**⚠️ Phase-0 §6 says "vitest, no DOM".** True of the *module*. It is **not** true of the milestone: the N-120 fix necessarily touches `resolve.ts` and `region-node.svelte`, because that is where the bug lives. **The runbook is the authority here, and this paragraph exists so you do not have to guess which document is stale.**

---

## 1. 🔒 LEG 1 — N-120: an index into the resolved tree is not an address into the descriptor

### 1.1 The bug, restated from the code (not from the record)

`region-node.svelte` iterates a **`ResolvedNode`**'s children and threads `path={[...path, i]}`. `resizeSplit` walks the **descriptor** by that path. **`resolve.ts` DROPS** — unknown leaves (rule 2), `tabs` (rule 3), and splits whose children all drop (rule 4). **The instant anything drops, `i` and the descriptor index diverge.**

Proven live at J-519: one ghost `widgetId` → drag a seam **right to enlarge `spaces`** → **`spaces` HALVED and its neighbour grew**, and 55% of the row's weight went to a widget that does not exist.

### 1.2 ⚠️ AND THERE IS A SECOND BUG INSIDE N-120 THAT THE J-519 WRITE-UP DID NOT NAME

`resizeSplit(layout, path, seamIndex, fraction)` resizes the pair **`(i, i+1)`** — it assumes the two neighbours are **adjacent in the descriptor**. **They need not be.** A dropped ghost can sit *between* the two tiles that actually paint on either side of a seam.

**So fixing the `path` alone is not enough. The signature is wrong.**

### 1.3 The fix — three edits

**(a) `resolve.ts` — every `ResolvedNode` carries its source index.**

```ts
export type ResolvedNode =
  | { type: 'leaf';  srcIndex: number; widgetId: string; collapsed?: FoldAxis }
  | { type: 'split'; srcIndex: number; dir: 'row' | 'col'; sizes: number[]; children: ResolvedNode[] };
```

`srcIndex` = **the node's index in its parent's `children` array in the DESCRIPTOR**. The root's `srcIndex` is `-1` (it has no parent; nothing may read it). The walk already has `i` in hand at the moment it decides to keep or drop — **it currently throws it away.**

**(b) `region-node.svelte` — thread the path from `srcIndex`, and report BOTH descriptor indices.**

- `path={[...path, child.srcIndex]}` (was `i`)
- the seam reports the pair's **descriptor** indices:
  `onResize?.(path, node.children[i-1].srcIndex, node.children[i].srcIndex, fraction)`

**🔒 The separation that makes this coherent, and it is worth naming: the LIVE PREVIEW stays in RESOLVED space; only the WRITE crosses into descriptor space.** `dragSizes`, `seamLive`, `effectiveSizes`, `flex` are all about *what paints* and must keep using the resolved index `i`. **Do not "fix" them.**

**(c) `mutate.ts` — the pair is two explicit indices.**

```ts
resizeSplit(layout, path: number[], aIdx: number, bIdx: number, fraction: number): Layout
```

- `aIdx` / `bIdx` are **descriptor** indices into the addressed split's `children`. They need not be adjacent.
- Weight moves **between exactly those two**. The pair total (`sizes[aIdx] + sizes[bIdx]`) is invariant.
- **⚠️ Anything between them — including a ghost's weight — is LEFT UNTOUCHED.** *The only behaviour that does not silently rewrite a widget that is not there.*
- Total temperament (N-095) unchanged: bad path · non-split target · `aIdx === bIdx` · either index out of range · non-finite `fraction` → **return the input unchanged, throw nothing.**

**⚠️ This breaks the 12 existing `resizeSplit` cases** (they pass `seamIndex`). **Migrate them — do not delete them.** `(l, [], 0, 0.5)` → `(l, [], 0, 1, 0.5)`. **Then ADD the non-adjacent case, which is the one that would have caught N-120:** a 3-child split resized on the pair `(0, 2)` must leave `sizes[1]` **byte-identical**.

---

## 2. LEG 2 — `fold` comes out of the shell

`app_client.svelte:66-84`'s `setLeafCollapsed` / `handleFold` → **`foldLeaf(layout, regionId, collapsed: FoldAxis | undefined): Layout`** in `mutate.ts`.

It is **already** pure, immutable, and **identity-addressed** (it recurses on `widgetId`) — which is exactly *why* fold was drop-safe at J-519 while resize was not. **This is a MOVE, not a rewrite. Do not redesign it.** Keep the unfold semantics **exactly**: `collapsed === undefined` **deletes the key** (never writes `collapsed: undefined`, never writes `false` — the migrate's drop rule depends on it).

`app_client.svelte`'s `handleFold` becomes a one-liner over the import. The shell keeps **no** tree surgery.

---

## 3. 🔒 LEG 3 — `move`

```ts
move(layout, sourceLeafId: RegionId, targetLeafId: RegionId,
     edge: 'top' | 'bottom' | 'left' | 'right'): Layout
```

**Identity-addressed by design** (D-116: a target tile is an **ADDRESS**; and a leaf is the only thing with an id). Four internal steps — except one of them does not exist.

### 3.1 remove

Find the source leaf by `widgetId`; drop it from its parent split's `children` **and** its entry from `sizes` (same index). **The survivors' proportions are already exact** — `[1,2,7,2]` minus one entry is `[2,7,2]`, which is correct as it stands.

### 3.2 collapse-degenerate — *the hard part*

**A split left with ONE child ceases to be a split: it is replaced by that child, which inherits its weight slot in the grandparent.** It **cascades** upward — collapsing one split can leave *its* parent with one child.

- The child's own `sizes` / `dir` / `collapsed` are **carried through untouched**; only its *position* changes.
- If the **root** collapses to a bare leaf, that is **legal and already renders** (`.region-shell > .region-tile { flex: 1 }` — grounded, it ships).
- A split left with **zero** children cannot happen: a `move` always re-inserts, and the source and target are distinct leaves.

### 3.3 insert

The edge names an **axis**: `left`/`right` → `row` · `top`/`bottom` → `col`. It also names a **side**: `left`/`top` → **before** the target · `right`/`bottom` → **after**.

- **Target's parent split already runs on that axis** → **insert as a SIBLING** of the target, on the named side.
- **Otherwise** → **WRAP the target** in a new split of that axis, `children: [target, source]` (or reversed, per side), and the new split **takes the target's old weight slot in the grandparent**.
- **Target is the ROOT leaf** → the root becomes a new split of that axis holding both.

### 3.4 ⚠️ re-normalise — **THIS STEP DOES NOT EXIST**

Phase-0 §6 lists *"re-normalise `sizes[]` so the survivors keep their relative proportions"* as step 4. **Grounded: there is nothing to do.** Deleting an entry leaves the survivors' ratios **already exact**, and `resolve.ts` has always done the same thing when a sibling drops (it skips the weight and keeps the rest). **Phase-0 §6 overstates the work. Do not invent a normalisation pass to satisfy a document.**

### 3.5 🔒 The weight rule — the moved region takes HALF the target's space

**Sibling insert:** double the target split's `sizes`, then bisect the target's doubled weight.

> `[2,7,2]`, dropping onto the middle → `[4,14,4]` → **`[4,7,7,4]`**

**Exact integers. No rounding. No `×10^n` rescale. The split's total is invariant, so nothing outside the pair moves** — the same invariant `resizeSplit` already holds.

**Wrap insert:** the new split gets `sizes: [1, 1]`. The grandparent's `sizes` are **untouched** (the new split occupies the target's slot).

**⚠️ The rejected alternative, and why it is not a preference:** *"the source keeps its original weight"* is a **category error**. A weight is a ratio **inside one split**. Carrying `7` out of one split and into another means **nothing** — it is not a size.

### 3.6 🔒 Three mechanics locks (Chat's, under §0 — argue with them)

1. **No-op guards.** `source === target` → unchanged. A drop that reproduces the region's current position (already the target's sibling on that side, in a split of that axis) → **unchanged**, not a pointless doubling of the descriptor.
2. **A folded region can be moved, and it KEEPS its fold axis.** The axis is the *user's* choice (§4.1), not the tree's. A tile folded `'width'` dropped into a `col` split flips from *absorbing* to *leaving a hole* — **that is correct**, and unfolding on move would discard user intent.
3. **A cycle is unsayable.** You cannot drop a region into its own subtree, because **a leaf has no subtree.** D-116's leaves-only rule pays off a second time — *the entire class of cycle bugs does not need a guard, because it cannot be expressed.* **Do not write one.**

### 3.7 Total temperament (N-095)

Unknown `sourceLeafId` · unknown `targetLeafId` · a `tabs` node anywhere on the path · a malformed tree → **return the input unchanged, throw nothing.** `move` is pure and total, like everything else in this module.

---

## 4. LEG 4 — the live proof, and it is the point of the milestone

`app_client.svelte` gains `handleMove(sourceId, targetId, edge)` (a one-liner over `move`) and **exposes it on the existing DEV handle**:

```js
window.__XGEN_LAYOUT__ = { get current(){…}, set(l){…}, move(s, t, e){…}, fold(id, axis){…} };
```

**Chat then drives `move` through that handle on the real client and WATCHES A REGION RELOCATE — before one line of gesture code exists.**

> ***A branch you cannot reach through the product is a branch you have not tested*** (M-RP7.1b: Clair said the migrate's live leg was not drivable; the way in was three clicks). **The tree healing wrong is a thing you SEE, not a thing you read in a test name.**

The DEV handle is dead-code-eliminated in a release build, exactly as `set` already is.

---

## 5. Verification — every leg re-driven by Chat on the real client (Rule 5)

**Reload before any baseline read** — a client mid-selection reads **71**, not 67 (N-112/N-115).

| # | leg | expected |
|---|---|---|
| **V1** | **N-120, reached not argued** — rebuild J-519's poisoned layout (one unknown `widgetId`; 3 descriptor children, **2 tiles, 1 seam**), drag the seam **right to enlarge `spaces`** | **`spaces` GROWS.** The ghost's weight is **byte-identical** before and after. *The gesture and the result finally agree.* |
| **V2** | **the move invariant** | **`leafCount` NEVER changes.** Registry **67** · `droppedCount` **0** · `leafCount` **8** — through **any** sequence of moves. *A tree that heals wrong will almost always break one of these, and it costs one CDP read.* |
| **V3** | **sibling insert** (drop onto a leaf whose parent already runs on the drop axis) | the target's weight **bisects**; the split total is **invariant**; **untouched siblings byte-identical** |
| **V4** | **wrap insert** (drop onto a leaf whose parent runs on the OTHER axis) | a new split of the drop axis, `sizes: [1,1]`, in the target's old slot; **the grandparent's `sizes` are untouched** |
| **V5** | **collapse-degenerate, and it must CASCADE** | move the second-to-last leaf out of a nested split → the split **vanishes** and its surviving child **inherits its weight slot**; construct a case where that leaves the *grandparent* with one child too, and prove it collapses as well |
| **V6** | **root degeneracy** | move leaves until the root is a **bare leaf** — it must **render**, not blank the centre (N-095's shape; `.region-shell > .region-tile { flex: 1 }` ships) |
| **V7** | **a folded region moves and keeps its axis** | `collapsed` survives the relocation; if the new parent's axis differs, the fold-mode flips along↔across **and that is correct** |
| **V8** | **`foldLeaf` is behaviour-identical after the migration** | the existing fold legs still pass; `app_client.svelte` is **smaller**; unfold still **deletes the key** |
| **V9** | **the live proof** | drive `__XGEN_LAYOUT__.move(...)` on client 9222 and **watch a region relocate on screen** |
| **V10** | suites | `npm test` **59 + the new cases** · `vite build` **169** · `cargo test` **1517 / 0 / 62 IDENTICAL** — *which proves the zero-Rust claim rather than asserting it* (**case-SENSITIVE** grep — N-117) |
| **V11** | cleanup | no inline styles, no probe residue; the session ends with `location.reload()` (N-123) |

**Test enumeration is production-grounded (D-078): grep the exported symbols in `mutate.ts` and enumerate from those — do not infer the suite from this runbook's prose.**

---

## 6. ⚠️ Traps, named up front

1. **`--region-min` / `--region-snap` must stay PLAIN values.** They are read by JS; `getComputedStyle` does **not** resolve `calc()` in a custom property — it returns the raw token stream and `parseFloat` gives **`NaN`**, and **a clamp of NaN is no clamp, and it passes** (N-122).
2. **Do not migrate the preview into descriptor space.** `dragSizes` / `seamLive` / `flex` are about **what paints**. Only `onResize`'s two indices and `path` cross over.
3. **Rule 6 has fired on the runbook three milestones running** (M-RP7.1b's migrate signature · M-RP7.2's hit area · M-RP7.2's §5 premise). **If something here contradicts the code, THE CODE IS RIGHT AND THIS DOCUMENT IS WRONG. Say so — do not silently absorb it.** *An implementer who absorbs a bad instruction ships the architect's mistake.*
4. **A coherent explanation that fits the evidence is not the cause.** Change one variable and look.

---

## 7. Definition of done

- [ ] `resolve.ts` — `srcIndex` on every `ResolvedNode`; the walk stops discarding it
- [ ] `region-node.svelte` — `path` threaded from `srcIndex`; the seam reports **both** descriptor indices; preview untouched
- [ ] `mutate.ts` — `resizeSplit(layout, path, aIdx, bIdx, fraction)` · `foldLeaf` · `move`
- [ ] `mutate.test.ts` — 12 existing cases **migrated** (not deleted) + the **non-adjacent pair** case + `foldLeaf` + `move` (sibling · wrap · cascade · root-degenerate · folded-region · total-temperament · deep-freeze immutability)
- [ ] `app_client.svelte` — `setLeafCollapsed` **gone**; `handleMove` + the DEV `move`/`fold` handle
- [ ] V1–V11 measured on the real client; **numbers Chat did not personally measure do not enter the record**
- [ ] Records: `docs/xgen-dock-engine-phase0.md` §6 (the re-normalise step **deleted**; §6.1 N-120 **discharged**) · `ui/docs/xgen-ui-notes.md` · `JOURNAL.md` · `CLAUDE.md` PLAY · `docs/ROADMAP.md` — one atomic commit (D-074)

*(`Status: COMPLETED` in this header is the shipped signal. "Commit pushed" is not a DoD item — Joe pushes.)*
