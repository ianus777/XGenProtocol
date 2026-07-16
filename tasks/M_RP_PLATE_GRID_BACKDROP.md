# M-RP-PLATE — grid backdrop plate (the grid-wide mount/containment exemplar)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Runbook for **Clair** (Code Claude). Design is LOCKED here — no design authority is taken during implementation. Chat authored this; Chat re-drives every verification leg on the live client 9222 (Rule 5) before the doc-bridge. Joe pushes.

---

## 0. One sentence

Promote the dev raster from a `.region-shell` CSS `background-image` (paint) to a real **grid-wide background-widget socket + one inert system plate widget** (element) — the exact `message-stream` `background?: WidgetMount[]` socket, one level up — so the grid backdrop mounts through the plugin registry, shows through every hole/seam/perimeter, and **never captures the pointer**.

## 1. Why this milestone exists (the exemplar's job)

A custom widget is the harder, more general case than any system widget; before scaling to install→dock→uninstall (the next exemplar, connection-stats), prove the two hard halves on self-contained instances. **This milestone closes the CONTAINMENT/MOUNT half**: *where does a `surface:'none'` grid-wide widget live, and how does the shell mount it there.* It does **not** touch runtime install (the registry stays a static array; that is connection-stats' job).

**Grounded facts this runbook rests on** (do not re-derive — N-116):
- `region-shell.svelte` has **no** background mount today. The raster is a pure CSS `background-image` on `.region-shell` (skin.css ~2785, moved there J-521 so the shell paints while splits are transparent and tiles opaque → seams, holes, and perimeter are ONE surface).
- `message-stream.svelte` already ships the socket one level down: `background?: WidgetMount[]` + `backgroundLive?` (default true, `<W {...b.props} {backgroundLive} />`), resolved against a `widgets` registry, unknown-id dropped (W-13), `backgroundMountCount` in G, rendered `position:absolute; inset:0; pointer-events:none`. **Mirror it — do not invent a second model.**
- `WidgetMount` = `{ widgetId: string; props?: Record<string, unknown> }` (`ui/core/lib/components/data-dependent/types.ts:53`).
- `CLIENT_PLUGINS` (`ui/common/lib/plugins/registry.ts`) is the static registry; `layout-default.ts` DERIVES the region `widgetRegistry` from `surface==='region'` rows (N-096, one source two readers). `plugin-list` is the existing `surface:'none'` row — but it carries **no** `component` (the shell mounts it into a dialog directly). The plate is the **first `surface:'none'` row WITH a `component`** (content the shell mounts into the grid background socket).

## 2. Locked decisions (D1–D4)

- **D1 — the socket lives ON `region-shell`** (core), mirroring `message-stream` one level up. A shell-side wrapper in `app_client` was rejected: it loses the precedent and the free node inheritance at M-RP7.7.
- **D2 — the plate is `kind:'system' · delivery:'compiled' · surface:'none'`** — the PROMOTED dev raster (§4.5.1: "the dev raster is promoted, not deleted… the first system plate widget, so the socket ships FED", D-065/N-091). It is the **mount** exemplar; connection-stats is the custom install exemplar. *(Corrects the loose "first custom-widget exemplar" wording in the session-open note — the record and §4.5.1 both say system.)*
- **D3 — inert now, switchable later.** Ship the socket + ONE inert plate with a single fixed appearance. `backgroundLive` is **exposed but unbound** (the `message-stream` "binding deferred to M-RP6.x" precedent). The live-switchable backdrop (solid-black → reactive fractal clouds) + its user setting ride **M-RP-SETTINGS** — gated on the J-513 settings-mechanism collision, unchanged by this milestone.
- **D4 — appearance is Joe's (§0), and it is the CURRENT raster.** The plate renders the same soft "system area — no functionality here" raster it renders today, now as an element. It ships **PROVISIONAL** → M-RP-SKIN. Do not tune the look; move the mechanism.

**Reserved-nothing holds:** no descriptor key, no store key, no `Layout` schema change (`version` stays 3), no persistence. The inert plate is a fixed default from `layout-default.ts`, not a user setting. Persistence of a chosen backdrop lands with M-RP-SETTINGS. (§4.5.1: "THIS ARC RESERVES NOTHING FOR IT" was about the grid arc; building the plate now is the milestone that adds the socket.)

## 3. Scope — files (NO Rust · NO schema · registry +1)

1. `ui/core/lib/components/layout/region-shell.svelte` — add the background socket + G fields.
2. `ui/common/lib/components/widgets/grid-plate.svelte` — **NEW** — the inert system plate widget.
3. `ui/common/lib/plugins/registry.ts` — add the `grid-plate` row (first `surface:'none'` WITH a `component`); update the file/plugin-list comments.
4. `ui/client/src/layout-default.ts` — derive the background registry + export `DEFAULT_BACKGROUND`.
5. `ui/client/src/app_client.svelte` — pass `background` / `bgWidgets` / `backgroundLive` to `<RegionShell>`.
6. `ui/assets/skin.css` — move the raster from `.region-shell` to `.grid-plate`; remove the `.region-shell` `background-image` (element replaces paint, does not double it).

**Out of scope:** `ui/sampler/**` (the plate is a client-shell backdrop, not a catalogue cell — like `region-shell` itself, D-097). No `ui/node/**` (the node inherits the socket free at M-RP7.7). No `.rs`. No `mutate.ts` / `resolve.ts` / `types.ts` change beyond importing `WidgetMount`.

## 4. Design detail (implement exactly)

### 4.1 `region-shell.svelte` — the background socket (mirror message-stream)

New props (additive; every existing prop unchanged):
```
background?: WidgetMount[];              // grid-wide backdrop mounts; undefined = none
backgroundLive?: boolean;               // default true — the settings switch, passed into each mount (unbound this arc)
bgWidgets?: Record<string, Component>;  // widgetId → component for the background socket ONLY (NOT the tile `widgets` map)
```
- Import `WidgetMount` from `$core/components/data-dependent/types`.
- **Use a SEPARATE `bgWidgets` registry, not the tile `widgets` prop.** The tile `widgets` map is region-id-keyed and feeds `resolveLayout`/W-13 for tiles; the plate id (`grid-plate`) is not a region id and must never be mistaken for a leaf. Two sockets, two registries — independently W-13-testable.
- `resolvedBg = $derived((background ?? []).map((m,i) => ({ key: `${m.widgetId}-${i}`, component: bgWidgets[m.widgetId], props: m.props ?? {} })).filter(x => x.component))` — the `message-stream` shape; a dropped unknown lowers the count (W-13).
- **Render placement:** the backdrop is the FIRST child inside `.region-shell`, BEFORE `<RegionNode>`, so paint order puts it under the opaque tiles; the drag overlay stays mounted LAST (above everything). Wrap the mounts in one `<div class="region-backdrop">` with `position:absolute; inset:0; pointer-events:none` **in skin.css** (N-090 — no component-local CSS). Each mount: `<W {...x.props} {backgroundLive} />` (backgroundLive threaded even though the inert plate ignores it).
- Getter G gains exactly two fields: `backgroundMountCount: resolvedBg.length`, `backgroundLive`. Nothing else in G changes.

### 4.2 `grid-plate.svelte` — the inert system plate (NEW)

- Home `$common/components/widgets/` (system widgets live here: self-panel, inspector-panel, plugin-list). W-3 holds — imports only `$common` (`envelope`), no shell/Tauri/protocol dep.
- Props: `backgroundLive?: boolean` (accepted, **ignored** — a static object ignores the switch, the `message-stream` contract). Optional `id?`.
- Root: one `<div class="grid-plate" use:envelope={{ name:'grid-plate', id, debug }}>`. No content (the look is a skin background). `role` none / decorative.
- Getter `debug = () => ({ backgroundLive })` — proves the switch is threaded even while inert.
- **Zero component-local CSS** — the raster look is `.grid-plate` in skin.css.

### 4.3 `registry.ts` — the row

Add after `plugin-list`, import `GridPlate`:
```
{
  id: 'grid-plate',
  name: 'Grid Backdrop',
  description: 'The backdrop shown behind the grid, in the gaps between panels.',
  kind: 'system',
  host: 'client',
  delivery: 'compiled',
  surface: 'none',       // content the shell mounts into the grid background socket (§3.2 / W-12) — NOT a tile
  component: GridPlate,   // FIRST surface:'none' row WITH a component (plugin-list has none)
}
```
Update the file header comment + the `plugin-list` inline comment to record: `surface:'none'` rows come in two shapes — no-component (shell mounts directly, e.g. plugin-list into a dialog) and with-component (shell mounts into a named host socket, e.g. grid-plate into the grid backdrop).

### 4.4 `layout-default.ts` — the background registry + default

- Derive (N-096, mirrors `widgetRegistry`): 
```
export const bgWidgets: Record<string, Component> = Object.fromEntries(
  CLIENT_PLUGINS.filter(p => p.surface === 'none' && p.component).map(p => [p.id, p.component as Component]),
);
export const DEFAULT_BACKGROUND: WidgetMount[] = [{ widgetId: 'grid-plate' }];
```
- Import `WidgetMount` from `$core/components/data-dependent/types`.

### 4.5 `app_client.svelte` — wire it

At the `<RegionShell>` mount (line ~362): add `background={DEFAULT_BACKGROUND}` `bgWidgets={bgWidgets}` `backgroundLive={true}`. `backgroundLive` is a hardcoded literal `true` this arc (unbound — the M-RP-SETTINGS binding replaces the literal). Import `bgWidgets` + `DEFAULT_BACKGROUND` from `./layout-default`.

### 4.6 `skin.css` — raster becomes the plate

- Add a `.region-backdrop { position:absolute; inset:0; pointer-events:none; }` rule (the socket wrapper; z-index below the tiles' stacking — the tiles are opaque and flow above, so no explicit negative z-index is needed, but if a stacking context forces it, use `z-index:0` on the backdrop and leave the drag overlay's existing z above).
- Move the raster paint (the `background-image` + its sizing) from `.region-shell` to `.grid-plate` (add `position:absolute; inset:0` so it fills the wrapper). Keep the exact same raster (PROVISIONAL — D4).
- **Remove** the `background-image` from `.region-shell` (leave any non-backdrop `.region-shell` rules intact). The element now paints it; the shell must not double it.

## 5. Verify (Rule 5 — Chat re-drives every leg on live client 9222; Clair reports her own numbers, Chat re-measures)

- **V1 registry delta.** Quiescent baseline (measure fresh) → **+1** (`grid-plate#…`, no children). `count===unique===domCount`. Enumerate the added id, do not infer the total.
- **V2 shows-through.** The `.region-backdrop` / `.grid-plate` fills the shell box (`inset:0`, rect == shell rect). Fold a region to open a HOLE → the raster is visible in the hole; the perimeter shows it; a seam shows it. Measure: `.grid-plate` `getBoundingClientRect()` covers the shell; a hole-point's paint is the plate, not a tile.
- **V3 pointer — the load-bearing leg (D-116).** `getComputedStyle('.region-backdrop').pointerEvents === 'none'`. `document.elementFromPoint(<a hole point>)` does NOT return the plate/backdrop (it is transparent to hit-testing). A grip drag still docks (hitTest still finds tiles; a hole still offers no band — unchanged from M-RP7.4). Prove a drag STARTED over a hole/plate cannot be captured by the plate. *A reactive backdrop is fine; a clickable one retires the tree.*
- **V4 backgroundLive threaded.** Shell G `backgroundLive:true`, `backgroundMountCount:1`. `grid-plate`'s own getter reads `backgroundLive:true` (the switch reaches the mount even while inert).
- **V5 W-13 drop.** Via `__XGEN_LAYOUT__` or a temporary prop path, feed `background:[{widgetId:'nope'}]` → `backgroundMountCount:0`, no crash, no blank/breakage; restore → 1. (If no DEV bridge for background exists, add a minimal DEV-only handle mirroring `__XGEN_LAYOUT__`; DEV-gated, dead-code-eliminated in release.)
- **V6 raster relocation is pure.** `getComputedStyle('.region-shell').backgroundImage === 'none'` (paint removed); `.grid-plate` carries the raster. Eye-check: the grid backdrop looks identical to before (Joe-confirmable).
- **V7 gates.** `git diff --stat` shows **no `.rs`** → `cargo test` IDENTICAL by construction (state the baseline count). `npm test` green. `vite build` clean (record the module count). No `ui/sampler/**`, no `ui/node/**` in the diff.

## 6. Definition of Done

- All six files land per §3; the six V-legs green, Chat-re-driven on 9222.
- Registry +1, `grid-plate` enumerated; `count===unique===domCount`.
- `.region-shell` no longer paints the raster; `.grid-plate` does; `pointer-events:none` proven.
- No Rust, no schema (`version` 3), no sampler/node touch.
- `Status: COMPLETED` set in this file's header on close (this header is the shipped signal — "commit pushed" is NOT a DoD item).

## 7. Records on close (Chat's doc-bridge, not Clair's)

JOURNAL J-532 · CLAUDE.md PLAY (NEXT-ACTIVE → connection-stats exemplar) · ROADMAP (M-RP-PLATE ✅; connection-stats + M-RP-SETTINGS on the horizon) · ui-notes N-131 (the grid-wide backdrop socket = message-stream one level up; `surface:'none'`-with-component; the pointer-events lock) · components registry bump · `docs/xgen-dock-engine-phase0.md` §4.5.1 (⏸️ FILED → ✅ BUILT; the inert/switchable split recorded). **No new D expected** (§4.5.1 + D-103/D-112/W-12 extension); if the `surface:'none'`-with-component case wants a D-number, that is Joe's call at close, not taken unilaterally.

## 8. Notes for Clair

- Ground before you name a risk (N-116). If any §4 shape does not compose cleanly against the real `region-shell`/`app_client` (e.g. a stacking-context surprise forcing an explicit z-index, or the DEV background bridge for V5), that is a **finding** — flag it (Rule 6), do not absorb it.
- Any disk write BOM-free (N-124). `Filesystem:*` for all `E:\` writes.
- Do not push (Joe pushes). Do not tune the raster look (D4 — Joe/M-RP-SKIN).
