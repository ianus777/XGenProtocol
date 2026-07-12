# M-RP7.1 — the tile frame: stripe, grip, fold
> **Status**: PENDING  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Leg 1 of 5 in the dock-engine arc (`docs/xgen-dock-engine-phase0.md`). **Read that Phase-0 first — this runbook implements it and does not restate it.**

**What this milestone is:** the chrome moves **from the widget into the renderer**. Every tile grows a title stripe (grip · name · fold), a corner resize grip **that is inert this leg**, and a fold state that lives in the descriptor.

**What this milestone is NOT:** nothing drags. Nothing resizes. **No pointer gesture ships except the fold button's click.** The corner triangle is **painted and dead** — see D7, it is deliberate and it is bounded.

---

## 0. Session-open grounding (Rule 0, and it already paid here)

**GROUNDED, not remembered — re-verify before touching anything:**

- **All EIGHT regions root in a titled `Section`.** `region-placeholder.svelte` → `<Section {title} id={`region-${regionId}`}>`; `self-panel.svelte:70` → `<Section title="Self" …>`; `inspector-panel.svelte:68` → `<Section title="Selection" …>`. **Add a stripe without unwrapping these and every region has two titles.**
- **`section` ships `collapsible` + `collapsed` ($bindable, 27th `core`)** — a collapsible section inside a foldable tile stacks **two fold affordances**.
- **`.region-split` already paints a `gap: 1px` over `--s5`** — the seam a splitter will use at 7.2. **Do not add geometry for it here.**
- **`sizes[]` already ride an inline `flex: {n} 1 0`** (the one N-090 carve-out — descriptor data, not skin).
- **`CLIENT_PLUGINS` already carries `name`** (M-RP6.1l). **The stripe title needs NO new data and NO new verb.**
- **`__XGEN_LAYOUT__` is `{ current, set }`** — the live tree, readable. (Chat once *guessed* this handle's shape and pushed `null` into it → **N-095**. Read it; do not guess it.)

**Baseline — reproduce, never quote (N-108):** client registry **67, quiescent, EMPTY STORE** (9222) · sampler catalogue **328** · `cargo test` **1517/0/62** · `vite build` **168** · `npm test` **41**.

---

## 1. 🔒 Design decisions (Chat, under the §0 autonomy grant)

### D1 — `region-tile` is a NEW `core` component; `region-node` mounts it
`region-node.svelte` currently renders a bare `<div class="region-leaf">` around the widget. It now renders **`region-tile`**, which owns: the stripe (grip · title · fold button), the body slot, and the (inert) corner grip.

- **`core` tier, not shell-local** — the node app inherits the grid at M-RP7.6, exactly the `region-shell` / `menu-bar` / `status-bar` precedent. **No Tauri, no protocol import.**
- **Ordinal assigned at build. DO NOT pre-book a number** (S-2's standing rule; `region-shell` took the 32nd at J-499).

### D2 — the title comes from the registry, not from the widget
`region-tile` takes a `title: string` prop. The **shell** resolves it from `CLIENT_PLUGINS` (`name`), falling back to the `regionId` for the six placeholder regions that are **not plugins**.

> **⚠️ A placeholder is scaffolding, not a plugin, and it is NOT in `CLIENT_PLUGINS`** (M-RP6.1l/D5 — deliberate). So the title map is **`plugin.name ?? REGION_NAMES[regionId] ?? regionId`**, and `REGION_NAMES` is the display map **already living in `region-placeholder.svelte`** — **move it to `layout-default.ts`, do not duplicate it.** *(A second copy of a name map is a D-067 drift surface the size of a postage stamp, and it is still a drift surface.)*

### D3 — 🔑 the eight `Section` roots are UNWRAPPED
- `region-placeholder.svelte` → renders its body **without** a `Section` wrapper.
- `self-panel.svelte` → drops `<Section title="Self">`, renders its rows directly.
- `inspector-panel.svelte` → drops `<Section title="Selection">`, renders its rows directly.

**`section` stays a fully legal `core` component and stays legal INSIDE a region body** — *contained, not the main form* (Joe). **Nothing about `section` changes. Do not touch `section.svelte`.**

### D4 — ⚠️ the leaf-id convention MIGRATES OWNER (N-096)
`id = region-${regionId}` is the leaf's durable registry handle. It currently sits on the `Section`. **It moves to `region-tile`.**

**→ `section#region-spaces` … `section#region-inspector` (8 entries) LEAVE the registry; `region-tile#region-spaces` … (8) ARRIVE.** Plus `self-panel#region-self` / `inspector-panel#region-inspector` keep their own ids **unchanged** (they register separately — grounded at J-500/J-501).

> **🔒 THE REGISTRY BASELINE WILL MOVE, AND THE NEW NUMBER IS MEASURED, NEVER DERIVED (N-105/N-108).** Do not compute it by arithmetic. Read it, **quiescent**, on a **stated store state**, and enumerate the delta.

### D5 — `collapsed?: boolean` on the `leaf` node; `version` → 2
`types.ts` gains **one optional field**. Optional → **absent means expanded** → every layout on disk stays valid → **the migrate is a no-op today**.

**Take the `version` bump anyway.** §9's migrate path has **never been exercised**, and a no-op first customer is the cheapest possible way to find out whether it works. `resolve.ts` carries `collapsed` through to the `ResolvedNode` **verbatim** — the walk does not interpret it; the renderer does.

### D6 — fold state is written to the DESCRIPTOR, held in memory this leg
The fold button mutates the live `layout` object (`__XGEN_LAYOUT__.current` reflects it immediately). **It is NOT persisted this leg** — the session feeder is M-RP7.5.

> **⚠️ AND THAT MEANS A W-8 DISCLOSURE IS FORBIDDEN HERE — N-109, pre-empted.** Do **not** add a "not saved yet" note anywhere in the UI. **A fold that survives a relaunch is not promised, so nothing is being misrepresented** — and a note added now is a note that must be swept at 7.5, which is exactly the defect N-109 records. *If any leg does ship a disclosure, its REMOVAL enters the DoD of the leg that lifts the limit, in the same edit that adds it.*

### D7 — 🔒 the corner triangle is PAINTED AND DEAD — and it is bounded, not smuggled
The triangle is **appearance**, and Joe must see it in position to judge it. But **`startResizeDragging` is not wired and no pointer handler is attached.**

**This is a deliberate, single exception to *"no dead controls" (D-065)*, and it is bounded three ways:**
1. It is **not a control** — no `role`, no `tabindex`, **`aria-hidden`**, not keyboard-reachable. It is a **painted affordance**, the `status-bar` grip's own precedent before 6.1e-B wired it.
2. Its **owner and its discharger are named**: **M-RP7.2 — splitter resize on the seam** wires it. *(A countdown names WHO, never WHEN — J-513's correction. 7.2 is the next leg because Chat chose the split, not because a dead triangle is nagging.)*
3. **It carries NO claim.** It says nothing to the user that later becomes false.

**If Joe would rather not see a dead triangle at all, it is dropped from this leg — appearance is his.** *Bring both.*

### D8 — ⚠️ fold in a `row` split is OPEN (Phase-0 §4.1) — Joe's call, and it BLOCKS the skin, not the code
`spaces` is the only row-parented tile. Two readings: **the fold button is ABSENT on a row-parented tile** (element-absent, J-500's precedent) · or **it folds sideways** (a second, rotated chrome).

**Build the mechanism direction-agnostically** (`collapsed` is just a flag on a leaf). **Render the fold button behind a `foldable` prop the shell computes from the parent's `dir`.** Then **either** answer is a one-line shell change and **no core rewrite**. **Do not decide it. Draw both for Joe.**

### D9 — zero component-local `<style>`. All of it in `skin.css` (N-090).
Stripe height, grip size, triangle size, seam, folded appearance, hover states — **every pixel is skin**, keyed off `.region-tile*`. **N-090 is Joe's rule and this is the milestone that tests it hardest: he must be able to retune the entire look without a component edit.**

### D10 — `region-tile` registers ONE getter G
```
{ regionId, title, collapsed, foldable }
```
**`collapsed` is RENDER TRUTH** — what actually painted, not what was intended (the `message.detailsCount` precedent). That is what makes a fold CDP-provable.

---

## 2. Scope — files

| file | change |
|---|---|
| `ui/core/lib/components/layout/types.ts` | `+ collapsed?: boolean` on `leaf`; `version` doc note |
| `ui/core/lib/components/layout/resolve.ts` | carry `collapsed` through to `ResolvedNode`, verbatim |
| `ui/core/lib/components/layout/resolve.test.ts` | + cases: `collapsed` survives the walk · absent ⇒ undefined · a collapsed leaf still resolves (it is **not** a drop) |
| `ui/core/lib/components/layout/region-tile.svelte` | **NEW** — the tile frame |
| `ui/core/lib/components/layout/region-node.svelte` | mounts `region-tile` instead of a bare `.region-leaf` div; passes `title` / `foldable` / `onFold` |
| `ui/core/lib/components/layout/region-shell.svelte` | threads `titles` + `onFold` down; G unchanged |
| `ui/client/src/layout-default.ts` | `REGION_NAMES` moves here; title resolver (`plugin.name ?? REGION_NAMES[id] ?? id`) |
| `ui/client/src/region-placeholder.svelte` | **unwrap `Section`** |
| `ui/client/src/app_client.svelte` | fold handler mutates `layout` |
| `ui/common/lib/components/widgets/self-panel.svelte` | **unwrap `Section`** |
| `ui/common/lib/components/widgets/inspector-panel.svelte` | **unwrap `Section`** |
| `ui/assets/skin.css` | all `.region-tile*` rules |
| `ui/docs/xgen-ui-components.md` | the new `core` component |

**NO RUST. NO sampler cells.** *(`region-tile` is grid chrome, like `region-shell` — the sampler catalogue stays **328**, and that is a verify leg, proven **by scope**.)*

---

## 3. Verify — real client 9222 ONLY (D-097)

**Chat re-drives every leg itself (Rule 5). A number Chat has not measured does not enter a canonical record.**

- **V1 — registry.** Quiescent, **empty store, stated**. `count === unique`. **Enumerate the delta**: 8 × `section#region-*` out, 8 × `region-tile#region-*` in; `self-panel#region-self` + `inspector-panel#region-inspector` **still present**. **Measure the new baseline. Do not derive it.**
- **V2 — G, all eight tiles.** `{regionId, title, collapsed, foldable}` exact. Titles: `self` → `Self Panel` (from `CLIENT_PLUGINS`), `inspector` → `Inspector Panel`, the six placeholders → `REGION_NAMES`.
- **V3 — 🔑 FOLD ON THE PAINTED PIXEL.** Click a `col`-parented tile's fold button → the tile's **measured height collapses to stripe height** and its **siblings grow** (`getBoundingClientRect`, not the attribute — **N-097**). Unfold → **exact** return. ⚠️ **Split the state-change and the DOM read across TWO evals (N-099)** and **assert the subject is READABLE first (N-110 — the DOM keys on `data-debug-id`, NOT `id`; a `#tile` selector returns `[]`, an empty array that looks exactly like a measurement).**
- **V4 — ⚠️ THE BUS MUST SURVIVE THE FRAME.** Click R3's `entity-item` → `__XGEN_SEL__` carries it → **R8 renders it**. Then **fold R3** → **the bus payload is unchanged and R8 still renders it**. *A widget must not learn it has been folded, and it must not lose its state to a frame change.* **This is the leg that proves the chrome move was structural and not a rewrite.**
- **V5 — geometry.** `docNoScroll` true · the grid still **fills** · split ratios **`[1,2,7,2]` exact at a FIFTH distinct width** · the leaf still self-scrolls, the document does not (D5/J-499) · the collapsed strip paints **no stray hairline** (computed style, not the attribute).
- **V6 — descriptor.** `__XGEN_LAYOUT__.current` shows `collapsed: true` on the folded leaf and the key **absent** elsewhere. `region-shell` G: `leafCount` **unchanged at 8** — *a folded tile is still a tile; folding is not dropping.*
- **V7 — skin.** All `.region-tile*` rules in cascade · **zero component `<style>`** · **accent-neutral** (a dock grid is chrome — assert on `readable:true` first, **never `null === null`**).
- **V8 — static gates.** `cargo test` **1517/0/62 IDENTICAL** *(the inverse leg: identical PROVES no Rust landed)* · sampler catalogue **328 unchanged, by scope** (`git show --stat`) · `vite build` · `npm test` (**+ the new resolve cases**).

> **⚠️ Static gates need the apps DOWN.** `cargo test` with the client up dies on `failed to remove file …xgen-client.exe` — the running app holds the binary.

---

## 4. DoD

1. Every tile draws grip · title · fold. **No region draws its own title.**
2. A `col`-parented tile folds to its stripe and unfolds to **exactly** its previous size.
3. `collapsed` round-trips through `resolve.ts`; `version: 2`; **absent ⇒ expanded**.
4. **The selection bus survives a fold** (V4).
5. Registry re-baselined, **measured, quiescent, store state stated**.
6. **`cargo test` identical.** Sampler **328**. **Zero component `<style>`.**
7. **Joe has seen it and the appearance is signed off** — stripe height · grip · triangle (or its absence, D7) · the folded form · the row-fold question (D8).
8. **No W-8 phase-limit note anywhere in the UI** (D6 / N-109).

**NOT in the DoD:** *"commit pushed"* (chicken-and-egg). `Status: COMPLETED` is the signal.

---

## 5. Lanes (D-074)

Design walk + this runbook = **Chat's**. Implementation + design closes = **Clair's**. Clair's commit = **code only**; Chat's doc-bridge = **commit 2**. **Joe pushes both. Chat NEVER pushes.**

---

*End of runbook.*
