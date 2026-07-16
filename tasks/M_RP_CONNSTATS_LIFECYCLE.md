# M-RP-CONNSTATS — connection-stats: the custom-plugin lifecycle exemplar
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Runbook for **Clair** (Code Claude). Design is LOCKED — no design authority is taken during implementation. Chat authored this; Chat re-drives every verification leg on the live client 9222 (Rule 5) before the doc-bridge. Joe pushes.

> **⚠️ Milestone ID `M-RP-CONNSTATS` is PROVISIONAL — Joe's to confirm/rename** (Rule 8: an ID never travels bare, so it carries the title here). Do not treat the ID as final until Joe locks it.

---

## 0. One sentence

Build the FIRST `kind:'custom'` widget (`connection-stats`) and, with it, the runtime **install → dock → uninstall** lifecycle that does not exist today — register an in-tree compiled custom widget into a now-reactive registry, inject its region into the layout, remove it **without blanking**, and **persist which customs are installed** so a reload does not W-13-drop them.

## 1. Why this milestone exists (the exemplar's job)

The plate (M-RP-PLATE) closed the *containment/mount* half of the plugin lifecycle. This closes the *install/uninstall* half — the harder, more general case (M-RP6.1l/J-513: the registry is a **static TS array with no runtime install path**). The widget's data is deliberately thin; **the lifecycle IS the milestone**.

**Grounded facts this runbook rests on** (do not re-derive — N-116):
- **Connection data already exists.** `ui/common/lib/stores/self-state.svelte.ts` exposes `selfState.connection : {state, label}` across 11 lifecycle states + `STATE_COLOURS` + `PULSING_STATES`, and `selfState.identity : {registered, identity_id, display_name, home_node, spaces_joined}`. The shell already writes it (the `xgen-client-state-changed` listen + `get_self_state`) — **W-3/D1: connection-stats READS this store, adds NO new channel, no new Rust.** Live traffic (bytes/latency/uptime/reconnect) does NOT exist yet — that is **M-RP6.6** (the resident), out of scope.
- **The leaf machinery already exists, private.** `mutate.ts` `move` is built on `removeLeaf(node,id)` (+ collapse-degenerate) and `insertBeside(node,targetId,source,axis,before)`. **Remove-without-blanking IS the collapse-degenerate** `move` already relies on. This milestone EXPOSES these, it does not invent algebra.
- **The registry is static.** `CLIENT_PLUGINS` (`registry.ts`) is a `const`; `widgetRegistry`/`bgWidgets` (`layout-default.ts`) derive once. There is **no runtime mutation path** — that is the gap.
- **A session bag persists per-device keys with zero Rust.** `uistate.svelte.ts` `persist()` does a per-key merge INSIDE `session` (N-107), overriding one key while preserving `geometry` (Rust's) + `layout` + unknown keys. `locked` (M-RP7.6) rode this. The installed-set rides it the same way — the opaque-blob path, **Rust never learns the shape** (the `get_substitutions`/`layout` precedent).

## 2. Locked decisions

- **D1 — one reactive registry.** A new `$common` store holds the runtime **installed-set**; `widgetRegistry`/`bgWidgets` derive reactively from `[...system rows, ...installed custom rows]`. System rows stay non-removable (W-13). NOT a second registry — one source, the N-096 shape.
- **D2 — install/uninstall reuse `move`'s primitives.** Expose `insertLeaf` (dock a new leaf beside a target) + `removeLeaf` (already collapse-degenerate) as public verbs. Install injects the leaf at a defined target; the user then drags it (M-RP7.4). No new algebra.
- **D3 — the installed-set persists in the UI-state SESSION bag** (per-device; the J-503 test — a compiled plugin's install is per-device arrangement, not synced config). Read on boot **BEFORE `loadLayout`**, so custom leaves resolve instead of W-13-dropping.
- **D4 — minimal DEV trigger now; real install/uninstall UI at M-RP-SETTINGS** (Ch6 §6.8.5's `[Remove]` action row = M-RP6.1m, POSTPONED). This milestone ships the MECHANISM + a DEV bridge (`__XGEN_PLUGINS__`, DCE'd in release) to drive+verify it. **No install/uninstall buttons are added to the plugins dialog** (that would start M-RP6.1m early).
- **Compact + extensible (Joe).** The widget is a **compact `{label,value}` metric-row list** (the inspector-panel `Label`-row shape), seeded ONLY with rows that have real data today; **adding a future metric = appending a row, never a rewrite**. **Do NOT fabricate empty/placeholder metric rows** for future tools (an unfed row is an unverified branch, N-091) — build the container to grow, seed it with what's real.

## 3. Scope — files (NO Rust · NO schema · NO sampler · registry mutates at runtime)

1. `ui/common/lib/components/widgets/connection-stats.svelte` — **NEW** — the custom widget (compact metric-row list reading `selfState`).
2. `ui/common/lib/plugins/installed.svelte.ts` — **NEW** — the reactive installed-set store + the AVAILABLE-custom catalogue + the derived active-plugin list (D1).
3. `ui/common/lib/plugins/registry.ts` — add the `connection-stats` descriptor to a new `AVAILABLE_CUSTOM` list (NOT `CLIENT_PLUGINS`); export it for the store.
4. `ui/core/lib/components/layout/mutate.ts` — expose `insertLeaf` + make `removeLeaf` public (or a thin `installRegion`/`uninstallRegion` wrapper) (D2).
5. `ui/client/src/layout-default.ts` — `widgetRegistry`/`bgWidgets` derive from the reactive active-plugin list; boot-order note.
6. `ui/client/src/uistate.svelte.ts` — `UiStateBag.installed?: string[]` + `setSessionInstalled` (mirrors `setSessionLocked`) + the per-key merge (N-107).
7. `ui/client/src/app_client.svelte` — boot: hydrate installed-set → register BEFORE `loadLayout`; wire install/uninstall (store + layout mutation + persist); the reactive `widgetRegistry`; the `__XGEN_PLUGINS__` DEV bridge.
8. `ui/assets/skin.css` — `.connection-stats` compact metric-row skin.

**Out of scope:** `ui/sampler/**` (a client widget, not a catalogue cell — D-097). No `.rs`. No install/uninstall UI in `plugins-dialog` (M-RP6.1m). No live-traffic metrics (M-RP6.6). No `Layout` schema change (`version` stays 3 — the installed-set is a `session` key, not a `Layout` field).

## 4. Design detail (implement exactly)

### 4.1 `connection-stats.svelte` — the compact widget

- Home `$common/components/widgets/`. W-3: imports only `$common` (`envelope`, `selfState`). Reads `selfState.connection` + `selfState.identity` reactively.
- Root `<div class="connection-stats" use:envelope={{ name:'connection-stats', id, debug }}>`. `id = region-${regionId}` (the self-panel convention → stable enumerable registration).
- **A metric-row LIST, not a fixed template.** Render an array of `{label, value}` rows so a future metric is one array entry. **Seed rows (real data only):** State (`connection.label` + a `led` coloured by `STATE_COLOURS[connection.state]`, pulsing iff `PULSING_STATES.includes(state)`) · Endpoint (`identity.home_node` or the ws address the self-panel already shows — reuse its source, do NOT introduce a second) · Registered (`identity.registered`) · Spaces joined (`identity.spaces_joined`). Rows whose source is null render absent (the `hasValue` precedent, N-060), not blank.
- **Compact skin** — tight rows, small type; appearance PROVISIONAL → M-RP-SKIN (Joe). Zero component-local CSS (N-090); `.connection-stats` lives in `skin.css`.
- Getter `debug = () => ({ state: connection.state, rowCount })` where `rowCount` = **render-truth** (the count of rows actually rendered) — this is what makes the compact/extensible growth CDP-observable and keeps future rows honest.

### 4.2 `installed.svelte.ts` — the reactive registry (D1)

- Module-level `$state` `installedIds = new Set<string>()` (the `selection.svelte.ts` reactivity precedent).
- `AVAILABLE_CUSTOM: PluginDescriptor[]` lives in `registry.ts` (§4.3); this store imports it.
- Exports: `installed` (getter → readonly view) · `isInstalled(id)` · `install(id)` / `uninstall(id)` (mutate the set; the LAYOUT injection/removal is wired in `app_client`, §4.7 — the store owns the SET, the shell owns the leaf) · `hydrate(ids: string[])` (boot seed, BEFORE loadLayout) · a derived `activePlugins` = `[...CLIENT_PLUGINS, ...AVAILABLE_CUSTOM.filter(p => installedIds.has(p.id))]`.
- DEV handle `__XGEN_PLUGINS__` mirroring `__XGEN_SEL__` (DCE'd) exposing `install`/`uninstall`/`installed` for the verify pass — but see §4.7: install/uninstall must ALSO move the layout, so the shell wraps them.

### 4.3 `registry.ts` — the available-custom catalogue

Add (NOT into `CLIENT_PLUGINS`):
```
export const AVAILABLE_CUSTOM: PluginDescriptor[] = [
  {
    id: 'connection-stats',
    name: 'Connection Stats',
    description: 'Live connection state and endpoint.',
    kind: 'custom',
    host: 'client',
    delivery: 'compiled',   // D-085: in-tree, NOT a loader. "Install" = register + inject, not dlopen.
    surface: 'region',
    regionId: 'connection-stats',
    component: ConnectionStats,
  },
];
```
Update the file comment: `CLIENT_PLUGINS` = the always-present system rows; `AVAILABLE_CUSTOM` = compiled customs the user MAY install (the first runtime-installable rows).

### 4.4 `mutate.ts` — expose the leaf verbs (D2)

- Make `removeLeaf` public (it already collapse-degenerates — the remove-without-blanking property). Signature unchanged.
- Add `export function insertLeaf(layout: Layout, newWidgetId: string, targetId: string, edge: Edge): Layout` — wraps `insertBeside` (reuse `edgeAxis`/`edgeBefore`), building a fresh `{type:'leaf', widgetId:newWidgetId}`. Pure + total, the `move` shape. Returns input by reference on a no-op (target missing).
- Do NOT change `move`/`resizeSplit`/`foldLeaf`.

### 4.5 `layout-default.ts` — reactive registry + boot order

- `widgetRegistry`/`bgWidgets` derive from `activePlugins` (the reactive list), not the static `CLIENT_PLUGINS`. Since the derive must be reactive, it moves to where Svelte reactivity works — either a `$derived` in `installed.svelte.ts` (exported) or computed in `app_client` (§4.7). Pick the `$common` store home so the node inherits it at 7.7.
- **Boot order (load-bearing):** `hydrate(installedIds)` runs BEFORE `loadLayout()` resolves, so a persisted custom leaf finds its registered widget instead of W-13-dropping. Document this in the boot comment.

### 4.6 `uistate.svelte.ts` — persist the installed-set (D3)

- `UiStateBag.installed?: string[]` (first-class, the `locked` precedent).
- `setSessionInstalled(ids: string[])` mirroring `setSessionLocked`; `persist()` per-key merge adds `installed` to the session override list (`layout` | `locked` | `installed`) — preserve `geometry` (Rust's) + unknown keys (N-107). `installed` undefined ⇒ no key written.

### 4.7 `app_client.svelte` — wire it

- **Boot:** after hydrate of the uistate store, read `session.installed ?? []` → `installedStore.hydrate(ids)` → THEN `loadLayout()`.
- **`widgetRegistry`** becomes a `$derived` off `activePlugins`, passed to `<RegionShell widgets={...}>`.
- **Install(id):** `installedStore.install(id)` + `layout = insertLeaf(layout, id, <defaultTarget>, <defaultEdge>)` + `uiStateStore.setSessionInstalled([...installed])` + persist. Default target/edge: a defined spot (e.g. beside `inspector`, edge `bottom`) — PROVISIONAL, Joe may retune; the user drags it after.
- **Uninstall(id):** `layout = removeLeaf(layout.root,...)`-wrapped (remove the leaf) + `installedStore.uninstall(id)` + `setSessionInstalled` + persist. Order: remove the leaf BEFORE deregistering so no frame renders an unresolved leaf; the collapse-degenerate absorbs the space (no blank).
- **`__XGEN_PLUGINS__` DEV bridge** exposing `install(id)`/`uninstall(id)` (the shell wrappers, so they move BOTH set and layout) + `installed` (getter). DCE'd in release.

## 5. Verify (Rule 5 — Chat re-drives every leg on live 9222 AFTER A FULL RELOAD, N-132)

- **V0 baseline (reloaded).** connection-stats NOT installed → registry = the current quiescent (measure fresh after reload — 73 today, but re-measure); 8 regions; `activePlugins` excludes it; `plugins-dialog` does NOT list it (only registered rows show — M-RP6.1m not started).
- **V1 install.** `__XGEN_PLUGINS__.install('connection-stats')` → registry gains `connection-stats#region-connection-stats` (+ its rendered rows) + a `connection-stats` plugin-list row (+3 labels, the N-096 ripple — enumerate, it is honest); a 9th region leaf appears; `count===unique`.
- **V2 render.** The widget shows the REAL connection state — today `DISCONNECTED`/`--err` (matches the status-bar led); `rowCount` = the seeded rows actually rendered; compact.
- **V3 uninstall.** `__XGEN_PLUGINS__.uninstall('connection-stats')` → leaf removed, widget + plugin-list row deregistered, registry back to baseline; **remove-without-blanking**: the freed space is absorbed (collapse-degenerate), `docNoScroll` holds, no blank centre.
- **V4 persist across reload (the load-bearing leg).** install → confirm `session.installed` on disk carries `["connection-stats"]` → **full reload** → the widget is STILL there (re-registered on boot BEFORE loadLayout, leaf NOT W-13-dropped). Then uninstall → reload → gone, `session.installed` empty.
- **V5 W-13 honesty.** With connection-stats installed and in the layout, temporarily NOT hydrating it (or feeding an unknown custom id) → its leaf drops (W-13), no crash — proving the boot-order dependency is real, not decorative.
- **V6 extensibility (assert the shape, don't fake data).** Confirm the widget renders from a `{label,value}[]` array (a metric is an entry), `rowCount` reports render-truth — NOT that N future rows exist. Do not add placeholder rows.
- **V7 gates.** `git diff --stat` no `.rs` → `cargo test` IDENTICAL by construction (state the baseline). `npm test` green. `vite build` clean (record modules). No `ui/sampler/**`.

## 6. Definition of Done

- The custom lifecycle works end-to-end: install (register + inject) → render real connection state → uninstall (deregister + remove, no blank) → **persists across reload** (V4).
- All V-legs green, Chat-re-driven on 9222 after a reload (N-132).
- No Rust, no schema (`version` 3), no sampler; registry mutates at runtime, `count===unique` at every step.
- `Status: COMPLETED` set in this file's header on close (the shipped signal — "commit pushed" is NOT a DoD item).

## 7. Records on close (Chat's doc-bridge, not Clair's)

JOURNAL J-533 · CLAUDE.md PLAY (NEXT-ACTIVE → M-RP-SETTINGS) · ROADMAP (M-RP-CONNSTATS ✅ once Joe blesses the ID; M-RP-SETTINGS + M-RP6.6 on the horizon) · ui-notes N-133 (the runtime install path: reactive registry + the available-vs-installed split + boot-order-before-loadLayout + persist-per-device) · components registry bump (the first `kind:'custom'` widget + the AVAILABLE_CUSTOM catalogue) · region-dock model if the install/uninstall verbs want recording there. **A D-number for the runtime-install path may be warranted** (the registry going reactive + the available/installed split is a real architectural decision) — **that is Joe's call at close, not taken unilaterally.**

## 8. Notes for Clair

- Ground before naming a risk (N-116). If any §4 shape does not compose against the real `mutate.ts`/`app_client`/`uistate` (e.g. the reactive-registry derive location, or the boot-order against the async `loadLayout`), that is a **finding** — flag it (Rule 6), do not absorb it.
- The endpoint row: reuse the self-panel's existing source; do NOT introduce a second connection-detail projection (D-067).
- Any disk write BOM-free (N-124). `Filesystem:*` for all `E:\` writes.
- Do not push (Joe pushes). Do not tune the widget's look (PROVISIONAL — Joe/M-RP-SKIN). Do not add install UI to the plugins dialog (M-RP6.1m).
