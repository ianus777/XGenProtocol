// layout-default.ts — the shell-side default layout, the widget registry map, and the layout SOURCE (D2).
// Shell-local: renderer A + the descriptor type are `core`, but the concrete default tree, the id→component
// map, and the (future Tauri) load seam are the client's.
//
// `loadLayout()` is the D2 seam: async from day one so the M-RP7.3 swap to `invoke('get_layout')` is a
// BODY change, not a call-shape change (and Rust never learns the node shape — it persists an opaque blob).
// Today it just returns DEFAULT_LAYOUT.

import type { Component } from 'svelte';
import type { Layout } from '$core/components/layout/types';
import type { WidgetMount } from '$core/components/data-dependent/types';
import { migrateLayout } from '$core/components/layout/resolve';
import RegionPlaceholder from './region-placeholder.svelte';
import type { PluginDescriptor } from '$common/plugins/registry';

// All 8 D-103 region ids (region-dock §2), in the default row order.
export const REGION_IDS = [
  'spaces', 'rooms', 'self', 'room-header', 'stream', 'composer', 'members', 'inspector',
] as const;

// Region display names (region-dock §2) — MOVED here from region-placeholder.svelte (M-RP7.1, D2). A
// second copy of a name map is a D-067 drift surface, so it lives in ONE place and both the (unwrapped)
// placeholder body and the tile title read it. These are the fallback titles for the SIX regions that are
// NOT plugins; `self`/`inspector` ARE plugins and take their `CLIENT_PLUGINS` name instead (below).
export const REGION_NAMES: Record<string, string> = {
  spaces: 'R1 · Spaces',
  rooms: 'R2 · Rooms',
  self: 'R3 · Self / connection',
  'room-header': 'R4 · Room header',
  stream: 'R5 · Message stream',
  composer: 'R6 · Composer',
  members: 'R7 · Members',
  inspector: 'R8 · Selection info',
};

// The registry maps are DERIVED from the ACTIVE plugin list (M-RP6.1l, D2 → M-RP-CONNSTATS, D1): a widget is
// in the grid — and in the plugin list — BECAUSE it is an active plugin (one source, several readers, N-096).
// Until M-RP-CONNSTATS the source was the static `CLIENT_PLUGINS` and these were module-level consts. Now the
// active list is RUNTIME-reactive (`installed.active` = system rows + installed customs), and a plain `.ts`
// module cannot hold a `$derived` — so these become PURE BUILDERS the shell (`app_client`) calls inside a
// `$derived` off `installed.active`. They stay HERE (not in the `$common` installed store) because they close
// over `RegionPlaceholder` / `REGION_IDS` / `REGION_NAMES`, and `RegionPlaceholder` is a SHELL component a
// `$common` store may not import (W-3) — the node builds its own equivalents at M-RP7.7, mirroring this shape.

/** widgetId → tile title. Fixed regions fall back to `REGION_NAMES`; a region plugin (system OR custom) uses
 *  its `name` (self → "Self Panel", inspector → "Inspector Panel", connection-stats → "Connection Stats"). */
export function buildTitles(plugins: PluginDescriptor[]): Record<string, string> {
  const pluginNames: Record<string, string> = Object.fromEntries(
    plugins.filter((p) => p.surface === 'region' && p.regionId).map((p) => [p.regionId as string, p.name]),
  );
  const out: Record<string, string> = Object.fromEntries(
    REGION_IDS.map((id) => [id, pluginNames[id] ?? REGION_NAMES[id] ?? id]),
  );
  // A custom region plugin's id is NOT in REGION_IDS; add its title so its tile shows the name, not the raw id.
  for (const p of plugins) {
    if (p.surface === 'region' && p.regionId && !(p.regionId in out)) out[p.regionId] = p.name;
  }
  return out;
}

/** region id → component: every fixed region id → the placeholder, then each `surface: 'region'` plugin
 *  (with a component) replaces/adds ONE entry. Today: self → SelfPanel, inspector → InspectorPanel, and —
 *  when installed — connection-stats → ConnectionStats. The other six regions stay placeholders. */
export function buildWidgetRegistry(plugins: PluginDescriptor[]): Record<string, Component> {
  return {
    ...Object.fromEntries(REGION_IDS.map((id) => [id, RegionPlaceholder])),
    ...Object.fromEntries(
      plugins
        .filter((p) => p.surface === 'region' && p.regionId && p.component)
        .map((p) => [p.regionId as string, p.component as Component]),
    ),
  };
}

/** The BACKGROUND socket registry (M-RP-PLATE, D1) — the `widgetRegistry` shape, one socket over (N-096).
 *  From the `surface: 'none' && component` plugins (today exactly `grid-plate`). Kept SEPARATE: a plate id is
 *  not a region id, so region-shell resolves the two sockets against two maps. */
export function buildBgWidgets(plugins: PluginDescriptor[]): Record<string, Component> {
  return Object.fromEntries(
    plugins.filter((p) => p.surface === 'none' && p.component).map((p) => [p.id, p.component as Component]),
  );
}

// The default grid backdrop (D3): ONE inert system plate. A fixed default from here, NOT a user setting —
// the live-switchable backdrop + persistence land with M-RP-SETTINGS (reserved-nothing holds: no store key,
// no descriptor key, no schema change). Props-less → the plate self-ids to `grid-plate` (stable register id).
export const DEFAULT_BACKGROUND: WidgetMount[] = [{ widgetId: 'grid-plate' }];

// DEFAULT_LAYOUT (D8) — exercises row + col + nesting, all 8 regions, NO unknown id, NO tabs (a broken
// default is not a test fixture; the drop/tabs/mismatch paths are driven at verify via __XGEN_LAYOUT__).
// `version: 3` (M-RP7.1b) — the first REAL schema bump since D-103 (leaf `collapsed`: boolean → FoldAxis).
// A persisted v1/v2 layout is upgraded by `migrateLayout` (below): its boolean `collapsed` flags become
// explicit fold directions using each leaf's parent-split dir — the first exercised migrate this project
// has ever run (§4.4-migration; N-091 — an unfed branch is an unverified branch).
export const DEFAULT_LAYOUT: Layout = {
  version: 3,
  root: {
    type: 'split', dir: 'row', sizes: [1, 2, 7, 2], children: [
      { type: 'leaf', widgetId: 'spaces' },
      { type: 'split', dir: 'col', sizes: [3, 1], children: [
        { type: 'leaf', widgetId: 'rooms' },
        { type: 'leaf', widgetId: 'self' },
      ] },
      { type: 'split', dir: 'col', sizes: [1, 8, 2], children: [
        { type: 'leaf', widgetId: 'room-header' },
        { type: 'leaf', widgetId: 'stream' },
        { type: 'leaf', widgetId: 'composer' },
      ] },
      { type: 'split', dir: 'col', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'members' },
        { type: 'leaf', widgetId: 'inspector' },
      ] },
    ],
  },
};

/**
 * Load the active layout (M-RP6.1k, Leg B — the D2 seam BODY swap, J-499). Reads the persisted
 * UI-state store via `get_ui_state`, pulls the SESSION arrangement's `layout` key, and falls back to
 * `DEFAULT_LAYOUT` on ANY of: no-Tauri (browser dev), an absent/empty store, a corrupt (unparseable)
 * store, or a present-but-malformed layout. It NEVER returns null — a null layout unmounts
 * `region-shell` → a blank centre (measured at J-499, registry 30→21). That is exactly N-095's
 * fallback, whose DoD moved to this milestone (D-115): recover to DEFAULT, EXERCISED not asserted.
 *
 * (The session layout is written on every mutation now (M-RP7.5 feeder, Leg B) and read back here — but the SEAM now
 * parses a real file, which is what makes N-095's guard reachable here rather than an unreachable
 * branch at M-RP7.3.)
 */
export async function loadLayout(): Promise<Layout> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const raw = await invoke<string>('get_ui_state');
    if (raw && raw.trim()) {
      const store = JSON.parse(raw);
      const layout = store?.session?.layout;
      // `migrateLayout` subsumes the old `isValidLayout` guard AND upgrades a v1/v2 boolean-`collapsed` tree
      // to v3 (M-RP7.1b). It NEVER returns null (N-095 — a malformed/older layout falls back to DEFAULT, so
      // the centre never blanks; D-115). DEFAULT_LAYOUT is injected because `core` must not own a default.
      if (layout) return migrateLayout(layout, DEFAULT_LAYOUT);
    }
  } catch (_) {
    // no-Tauri OR corrupt store → DEFAULT (N-095). A read/parse error must never blank the centre.
  }
  return DEFAULT_LAYOUT;
}
