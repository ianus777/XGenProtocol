# M-RP6.1f — Centre region-shell scaffold + selection bus
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Runbook for **M-RP6.1f** — the first real step of the widget grid. Renderer **A** reads the D-103 `Layout` descriptor, mounts placeholder leaves into the client centre, and the **selection bus** primitive lands.

Phase-0 is **already locked** (J-488 / D-107 · `docs/xgen-client-frame-phase0.md` §6 · `ui/docs/xgen-region-dock-model.md` §3/§5). **This is a BUILD walk, not a re-design.** Design walked + Joe-locked 2026-07-11 (autonomy granted to Chat's recommendations).

**Independent of `docs/xgen-widget-surfaces-phase0.md`** (shelves / surfaces / UI-state store). That doc is written but **NOT locked**; nothing in it changes 6.1f. Do not design against it.

---

## 0. Grounding — read this before the design (the J-497 §2.0 discipline)

Four findings from reading the **shipped code**, not memory. Each one changes what gets built.

### 0.1 The leaf-resolution machine ALREADY SHIPS — reuse it, do not invent it

`ui/core/lib/components/data-dependent/message.svelte` already implements the exact seam renderer A needs:

- it takes a **prop-injected registry** `widgets?: Record<string, Component>` (widgetId → component),
- it resolves each `WidgetMount.widgetId` against it,
- and it **DROPS a mount whose id it cannot resolve** (W-13 reconcile) — the getter's `detailsCount` reports the **resolved** count, so the drop is CDP-provable.

`WidgetMount { widgetId, props? }` already lives in `$core/components/data-dependent/types.ts`.

**Renderer A uses the same registry shape and the same drop rule.** No second mechanism.

### 0.2 `$common` widgets already import `$core` types

`entity-context-menu.svelte` (in `ui/common/lib/components/widgets/`) imports `EntityDescriptor` from `$core/components/data-dependent/types`. So a `$common` store may type its payload against `EntityDescriptor` on **shipped precedent** — this is a **type-only** import, erased at build. No layering argument needs to be re-litigated.

### 0.3 The client has SIX app-defined Tauri commands, all one shape

`get_state` · `get_pacing_state` · `get_substitutions` · `set_substitutions` · `get_about_info` · `quit` (`xgen-client/src/desktop.rs`). All app-defined → **no capability grant** (J-497). **6.1f adds NONE of them** — see D2.

### 0.4 ⚠️ `.app-center` is `overflow-y: auto` — and that is WRONG for a grid

`ui/client/src/app.css` today:

```
.app-center { flex: 1; min-height: 0; overflow-y: auto; padding: 12px 16px; }
```

J-495 locked **"the centre is the ONLY scroller"** (D5) — correct for a single placeholder paragraph, **wrong for a dock layout**. A grid must **fill** its box and **not** scroll as a whole; each **leaf** owns its own scroll (that is what a docked panel is). The `padding` would also inset the grid from the frame.

**6.1f therefore supersedes the J-495 D5 lock, inside the centre only** (see D5 below). Flagged here rather than absorbed silently (Rule 6). `docs/xgen-client-frame-phase0.md` §10.3 gets amended at the **doc-bridge**, after it is measured — not before.

---

## 1. Scope

| in | out |
|---|---|
| `Layout` descriptor TS types + a **pure** resolve/walk module | any Rust (no `get_layout` command — D2) |
| `region-shell` — the recursive renderer A, a **`core`** component | tabs rendering, splitters, drag/drop (renderer B, M-RP7) |
| the **selection bus** `$common` store | any bus **writer** (R1/R2 don't exist yet) |
| a shell-local placeholder leaf + the default layout + the registry map | the 8 real region widgets (6.1g+) |
| the `.app-center` scroll flip | layout **persistence** (M-RP7.3) · shelves (unlocked doc) |

---

## 2. Locked decisions

### D1 — Descriptor: ship the FULL type, render the SUBSET

The TS type is the **complete D-103 §3 contract** — `leaf | split | tabs`, `Layout { version, root }`. It is the serializable contract **both** renderers read; typing it costs nothing and a later schema change is what `version` exists for.

Renderer A implements **`leaf` + `split` only**. A **`tabs` node is DROPPED with a DEV warn** — the §0.1 unknown-id drop precedent. **No tab-strip code ships.** (D-065: build when consumed. N-091: unexercised code is unverified code — a `tabs` branch nobody feeds is a branch nobody proved.)

### D2 — `get_layout`: a FRONTEND constant. No Rust this milestone.

A Rust `get_layout` returning a hardcoded default would either **duplicate the descriptor type in Rust** — precisely the D-067 drift surface the C2 grounding killed — or return an opaque blob Rust does not own (theatre for one call site).

The seam that actually matters is expressed **in the frontend**:

```
// ui/client/src/layout-source.ts  (or inline in app_client.svelte — Clair's call)
export async function loadLayout(): Promise<Layout> { return DEFAULT_LAYOUT; }
```

`app_client.svelte` **awaits** it on mount. At **M-RP7.3** its body becomes `invoke('get_layout')` — **one function, one swap**, and Rust persists the tree as an **opaque blob** (the `get_substitutions` shape, region-dock §9), so Rust never learns the node shape. **Async from day one so the swap is a body change, not a call-shape change.**

### D3 — The selection bus is a `$common` store. This is GROUNDED, not preferred.

Both eventual consumers — **R8 inspector** and **`entity-context-menu`** — are widgets living in `ui/common/lib/components/widgets/`, and **W-3 forbids a `common` widget importing a shell dep**. A shell-local bus is therefore **structurally impossible to consume**. It is a `$common` store or it is nothing.

- **Home:** `ui/common/lib/stores/selection.svelte.ts` — a **new `stores/` folder**. (The existing `components/processor/store.svelte.ts` lives inside `processor/` because it is the *processor's* store; the selection bus is the first **shell-wide** primitive and earns its own home. The future UI-state store joins it.)
- **Shape — EXACTLY as locked, do not widen:** `{ regionId: string, entity: EntityDescriptor } | null`. **ONE meaning.** (Joe killed the shelf minus-button precisely so a second widget/leaf selection bus is never needed — `xgen-widget-surfaces-phase0.md` S-6. **Do not introduce one.**)
- **Surface:** `selection.current` (getter, reactive) · `selection.set(regionId, entity)` · `selection.clear()`.
- **DEV handle:** `__XGEN_SEL__` on `window` under `import.meta.env.DEV` — the `__XGEN_SUBS__` / `__XGEN_PROC__` idiom (N-024). This is the **only** way the bus is CDP-drivable at 6.1f (there is no writer yet — an honest phase-limit, W-8; state it in the code comment, do not fake a writer).
- The `EntityDescriptor` import is **`import type`** from `$core` (§0.2).

### D4 — Where it lives: SPLIT (`core` renderer + shell mount)

The frame precedent exactly (frame-phase0 §2: *"frame containers are `core`; window-effects are shell-wired"*):

| file | tier | why |
|---|---|---|
| `ui/core/lib/components/layout/types.ts` | `core` | the `LayoutNode` / `Layout` / `RegionId` contract |
| `ui/core/lib/components/layout/resolve.ts` | `core` | **pure**, DOM-free walk (vitest — the `stream/grouping.ts` precedent) |
| `ui/core/lib/components/layout/region-shell.svelte` | `core` | renderer A. **Registers ONE getter.** No Tauri, no protocol. |
| `ui/core/lib/components/layout/region-node.svelte` | `core` | the **internal, non-registering** recursion part (the `sb-cell` / N-064 opt-out — a per-node getter would be pure ordinal noise) |
| `ui/client/src/region-placeholder.svelte` | shell | the throwaway leaf |
| `ui/client/src/layout-default.ts` | shell | `DEFAULT_LAYOUT` + the widget registry map |
| `ui/client/src/app_client.svelte` | shell | mounts `region-shell` into `.app-center` |
| `ui/client/src/app.css` | shell | the `.app-center` flip (D5) — **frame skeleton only** |
| `ui/core/skin.css` *(wherever `skin.css` lives)* | skin L2 | **ALL** `.region-*` appearance |

**The renderer is `core` because the NODE app inherits it at M-RP7.x.** That is the whole reason it is not shell-local.

**⚠️ N-090 — every skinnable setting is in `skin.css`.** *Skinnable includes gaps, sizing, grid tracks and layout.* So `.region-shell`, `.region-split`, `.region-leaf`, `.region-placeholder` — **flex direction, gaps, borders, min-sizes, overflow, colour, type — ALL of it in `skin.css`.** `app.css` gains **nothing** but the `.app-center` edit. **No component-local `<style>` block anywhere** (N-023/N-025). A scoped `<style>` in `region-shell.svelte` would be the first in the codebase — it is not starting here.

**The one exception, and it is not appearance:** a split's `sizes[]` are **DATA from the descriptor**, so they ride an **inline** `style="flex: {n} 1 0"` on each child — the `led` `--led-colour` / `meter` `--meter-fill` precedent (data-driven value inline, skin owns shape). Gaps/borders/mins stay in skin.

### D5 — `.app-center` flips: the grid FILLS, the leaves scroll

Supersedes the J-495 D5 "centre is the only scroller" lock **within the centre**. The frame chrome (menu-bar / status-bar) still never scrolls.

```
.app-center { flex: 1; min-height: 0; overflow: hidden; padding: 0; display: flex; }
```

- `overflow-y: auto` → **`overflow: hidden`** — the grid must not scroll as a unit.
- `padding: 12px 16px` → **`0`** — the grid meets the frame; inner gaps are the skin's job.
- `display: flex` so `region-shell` fills the box.
- **`min-height: 0` must ride EVERY nested flex level** or the classic flexbox blowout puts a scrollbar back on the document. This is a required verify leg (§5, geometry).
- **The leaf is the scroller** — `.region-leaf { overflow: auto; min-height: 0; min-width: 0; }` in **skin.css**.

### D6 — Placeholder leaves: ONE component, EIGHT registry entries

The registry map ships **all 8 D-103 region ids** — `spaces` · `rooms` · `self` · `room-header` · `stream` · `composer` · `members` · `inspector` — **all pointing at the same** `region-placeholder.svelte`. Each real widget later replaces **one map entry**; no rewrite.

`region-placeholder` renders a **`section`** (`title` = the region's display name, `id = "region-" + regionId`) with a **plain text node** body.

- **Deliberately NO `paragraph` inside** — it would double the registry delta for a throwaway. One `section` per leaf, clean delta.
- Placeholder body text says what it is: e.g. *"R5 · Message stream — placeholder (M-RP6.1g+)"*.

### D7 — Layout state stays SHELL-LOCAL this milestone

A `$state` in `app_client.svelte` (seeded by `await loadLayout()`), plus a DEV `__XGEN_LAYOUT__` handle (`{ current, set(layout) }`) so CDP can drive the drop tests (§5).

*Recorded so it is not re-derived:* the **widget manager** and the **shelf** will eventually need to mutate the layout **from `common` widgets** → the same W-3 argument as D3 will force the layout into a `$common` store. **That is not today** (D-065 — the shell is the only consumer, and the surfaces doc that introduces those widgets is not locked). Promotion is reserved, not pre-built.

### D8 — `DEFAULT_LAYOUT` (exercises row + col + nesting, all 8 regions, no unknown ids)

```
{ version: 1, root:
  { type: "split", dir: "row", sizes: [1, 2, 7, 2], children: [
    { type: "leaf", widgetId: "spaces" },
    { type: "split", dir: "col", sizes: [3, 1], children: [
      { type: "leaf", widgetId: "rooms" },
      { type: "leaf", widgetId: "self" } ]},
    { type: "split", dir: "col", sizes: [1, 8, 2], children: [
      { type: "leaf", widgetId: "room-header" },
      { type: "leaf", widgetId: "stream" },
      { type: "leaf", widgetId: "composer" } ]},
    { type: "split", dir: "col", sizes: [1, 1], children: [
      { type: "leaf", widgetId: "members" },
      { type: "leaf", widgetId: "inspector" } ]} ]}}
```

**The shipped default contains NO unknown id and NO `tabs` node** — a broken default is not a test fixture. The drop paths are driven at verify time through `__XGEN_LAYOUT__` (§5).

---

## 3. The pure module (`resolve.ts`) — vitest owns this

DOM-free, so the whole walk is unit-testable without an app (the `grouping.ts` / `Accelerator` precedent; the vitest harness already stands from M-RP6.1c, 35/35).

```
resolveLayout(layout: Layout, knownIds: Set<string>): {
  root: ResolvedNode | null;   // the walked tree with unresolvable nodes removed
  leafIds: string[];           // resolved leaves, in document order
  dropped: string[];           // widgetIds dropped (unknown to the registry)
  unsupported: number;         // `tabs` nodes dropped by renderer A
}
```

Rules (all testable, all pure):
1. `leaf` with an id **in** `knownIds` → kept.
2. `leaf` with an unknown id → **dropped**, recorded in `dropped`. **Never throws.**
3. `tabs` → **dropped**, `unsupported++`, **one DEV warn**. Never throws.
4. A `split` whose children all drop → the split itself drops (an empty box is noise).
5. `sizes.length !== children.length` → fall back to equal sizes + DEV warn. **Do not throw** (a stale descriptor must degrade, never crash — region-dock §9: *"never crash on a stale tree"*).
6. Depth is unbounded; the walk is recursive and pure.

**Test file:** `ui/core/lib/components/layout/resolve.test.ts`. Minimum cases: default layout resolves 8 leaves / 0 dropped / 0 unsupported · unknown id drops · all-unknown split collapses · `tabs` drops + counts · sizes-mismatch degrades · empty/`null` root survives.

---

## 4. Getter G (`region-shell`, ONE registered entry)

```
{ version, leafCount, widgetIds: string[], droppedCount, unsupportedCount, depth }
```

`leafCount` / `widgetIds` are the **RESOLVED, RENDERED** truth — the `message.detailsCount` precedent (the getter reports what rendered, not what was intended). That is what makes the drop CDP-provable.

Each **leaf's** component self-registers under its own id (`section#region-spaces`, …). `region-node` registers **nothing**.

---

## 5. Verify — REAL CLIENT 9222 (D-097). The sampler is not the home for this.

The sampler is a component-cell grid with no frame; the region shell is the client's centre. (Individual `core` components still get sampler cells — `region-shell` is a frame-assembly component like `menu-bar`/`status-bar` were, and those were verified in the client.)

**Chat re-drives every non-destructive leg itself before the doc-bridge (Rule 5).**

### 5.1 Registry
- `count === unique`, **enumerated ids** quoted.
- Client registry **grows from 22 — MEASURE IT. Do not predict it, do not fabricate it** (Rule 5).
- Expected **shape** (not a number to assert): `+1 region-shell` and `+1 section per resolved leaf`; `region-node` adds none.
- **⚠️ N-092a — DO NOT RUN THE ORPHAN LEG.** The client's debug bridge (`ui/common/lib/components/base/debug.ts`) is **state-only** (`id → {type, get}`, no DOM handle, no marker attribute). `domCount` / "0 orphans both directions" is a **sampler-only** capability. It is not expressible here. **Anyone copying it into this runbook is repeating Chat's J-498 miss.**

### 5.2 Geometry — a REQUIRED leg (N-091)
N-091 exists because `dialog` shipped CLOSED and CDP-VERIFIED and still rendered top-left: **"verified" is only as wide as the legs you ran.** The region shell **IS layout** — geometry is not optional here.

- `region-shell` root rect **fills** the `.app-center` content box (both axes).
- **NO document scroll**: `documentElement.scrollHeight === clientHeight` and `.app-center.scrollHeight === clientHeight` (this is what proves the D5 flip + the `min-height: 0` chain).
- Leaf rects **tile** — no overlap, children of a split sum (± gaps) to the parent.
- Split `sizes` are honoured: measure the row's four children against `[1,2,7,2]` (ratios, not absolutes).
- A leaf with overflowing content **scrolls itself** (inject tall content into one leaf → that leaf's `scrollHeight > clientHeight`, document still 0). Restore after.

### 5.3 Drop paths (driven via `__XGEN_LAYOUT__`)
- Push a layout with an **unknown `widgetId`** → getter `droppedCount` rises, `leafCount` falls, that `section#region-*` **unregisters**, **no crash**.
- Push a layout containing a **`tabs`** node → `unsupportedCount` rises, node not rendered, **no crash**, DEV warn present.
- Push a **sizes-mismatched** split → renders with equal sizes, no crash.
- **Restore `DEFAULT_LAYOUT`** and re-measure the registry back to baseline.

### 5.4 Selection bus (driven via `__XGEN_SEL__`)
- `set('spaces', {kind:'space', id:'x', name:'X'})` → `current` exactly `{regionId:'spaces', entity:{…}}`.
- `set(...)` again → **replaces** (one selection, not a list).
- `clear()` → `null`.
- **Honest phase-limit (W-8):** there is **no UI writer** at 6.1f. Say so; do not fake one.

### 5.5 Skin / accents
- `.region-*` rules present **in the cascade** (stylesheet-rule inspection, N-042 method — `getComputedStyle` alone is not the proof for pseudo/inherited rules).
- **Accent-neutral**: inject an `--accent2` swap → region chrome unchanged (the grid is chrome, not an accent carrier).
- **Zero component-local `<style>`**: prove by `git diff` / grep — no `<style>` in any new `.svelte`.

### 5.6 Build + tests
- `vite build` clean — **quote the module count**.
- `npm test` (vitest) — **quote the real pass count**, including the new `resolve.test.ts` cases. Baseline is 35/35 (M-RP6.1c).
- `git diff --stat` — **scope-clean**: no Rust, no `xgen-client/**`, no `ui/sampler/**`, no `ui/node/**`.

### 5.7 Harness gotchas (do not rediscover)
- **PS 5.1:** single-expression `JSON.stringify({…})` evals only. Multi-statement evals with local `var` + callbacks intermittently throw. A read **after** a thrown eval is **inconclusive, not a failure** (Rule 1).
- **Svelte 5** flips state synchronously but tears DOM down on the **effect flush** — **read after settle**, not same-tick.
- **CDP race:** the port opens before Svelte mounts `window.__XGEN_DEBUG__` — retry `snapshot()` until non-null; port-up ≠ ready.
- **Long-running processes** (`cargo tauri dev`) **hang the MCP server** — **Joe launches the dev session**; Claude runs only short-lived commands.
- Harness: `.\cdp-debug.ps1 -App client -Mode eval -Expression '...'` (port **9222**).

---

## 6. Definition of Done

- [ ] `layout/types.ts` — full `leaf|split|tabs` + `Layout {version, root}` (D1)
- [ ] `layout/resolve.ts` — pure walk, all six rules (§3), **never throws**
- [ ] `layout/resolve.test.ts` — the six minimum cases green; `npm test` count quoted
- [ ] `region-shell.svelte` (`core`) — registers ONE getter G (§4); **no `<style>` block**
- [ ] `region-node.svelte` (`core`) — internal, **non-registering** (N-064)
- [ ] `selection.svelte.ts` (`$common/stores/`) — `{regionId, entity} | null`, `set`/`clear`/`current`, `__XGEN_SEL__` DEV handle
- [ ] `region-placeholder.svelte` + `layout-default.ts` + registry map (all 8 ids → placeholder)
- [ ] `app_client.svelte` — `await loadLayout()` on mount, `region-shell` mounted in `.app-center`, `__XGEN_LAYOUT__` DEV handle
- [ ] `app.css` — `.app-center` flip (D5); **nothing else added**
- [ ] `skin.css` — **ALL** `.region-*` appearance incl. gaps/tracks/overflow (N-090)
- [ ] CDP 9222: registry `count===unique` + enumerated + **measured** delta (§5.1) — **no orphan leg**
- [ ] CDP 9222: geometry — fills, tiles, ratios honoured, **no document scroll**, leaf self-scrolls (§5.2)
- [ ] CDP 9222: unknown-id drop · `tabs` drop · sizes-mismatch degrade · restore to baseline (§5.3)
- [ ] CDP 9222: selection bus set/replace/clear (§5.4)
- [ ] Skin in cascade + accent-neutral + zero component-local CSS (§5.5)
- [ ] `vite build` clean (module count quoted) · `git diff --stat` scope-clean (§5.6)
- [ ] Rule-6 deviations flagged in the handback, **not absorbed**

*(D-074 — Clair's feat commit is **code only**. The JOURNAL / CLAUDE.md / ROADMAP / frame-phase0 §10.3 amendment / `ui/docs/` records are Chat's **doc-bridge**, the second commit. Joe pushes both. "Commit pushed" is deliberately **not** a DoD item.)*

---

## 7. Deferred out of this milestone (D-065)

- `tabs` rendering, splitters, drag-drop, hover-to-plug-in → **renderer B, M-RP7**.
- Layout **persistence** (`get_layout` / `save_layout`, the opaque-blob Rust) → **M-RP7.3**.
- **Re-inject-missing-system-widgets** reconcile (region-dock §9) — untestable with no persistence; the **drop** half is all that ships.
- Layout state → `$common` store (D7) — reserved, triggered by the widget manager / shelf.
- The 8 **real** region widgets → 6.1g (R3 Self/connection) · 6.1h (R8 inspector) · 6.2+.
- Any bus **writer** → 6.1g+ (R1/R2 row activation).
- Shelves / surfaces / UI-state store → their own doc, **not yet locked**.

---

*End of runbook.*
