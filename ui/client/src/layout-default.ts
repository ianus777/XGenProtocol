// layout-default.ts — the shell-side default layout, the widget registry map, and the layout SOURCE (D2).
// Shell-local: renderer A + the descriptor type are `core`, but the concrete default tree, the id→component
// map, and the (future Tauri) load seam are the client's.
//
// `loadLayout()` is the D2 seam: async from day one so the M-RP7.3 swap to `invoke('get_layout')` is a
// BODY change, not a call-shape change (and Rust never learns the node shape — it persists an opaque blob).
// Today it just returns DEFAULT_LAYOUT.

import type { Component } from 'svelte';
import type { Layout, RegionId } from '$core/components/layout/types';
import type { WidgetMount } from '$core/components/data-dependent/types';
import { migrateLayout } from '$core/components/layout/resolve';
import { insertLeaf, type Edge } from '$core/components/layout/mutate';
import RegionPlaceholder from './region-placeholder.svelte';
import type { PluginDescriptor } from '$common/plugins/registry';

// The D-103 region ids (region-dock §2), in the default row order. The first 8 are the canonical R1–R8
// regions; `dm-spaces` is the NINTH region (M-RP-MEMBER-ACT Leg E-1, the DM home). It is NOT in
// DEFAULT_LAYOUT — the home is placed by the E-2 re-inject only (Joe's ①-B), so it never drifts between a
// fresh and a re-injected tree. Its presence here seeds a placeholder + lets `buildWidgetRegistry` /
// `buildTitles` resolve the leaf when the re-inject (or the DEV bridge at verify) mounts it.
export const REGION_IDS = [
  'spaces', 'rooms', 'self', 'room-header', 'stream', 'composer', 'members', 'inspector', 'dm-spaces',
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
  // Ninth region (M-RP-MEMBER-ACT Leg E-1). No R-number — it is not one of the canonical R1–R8. This
  // fallback is only ever shown if the `dm-spaces` plugin does not claim the leaf; the plugin `name`
  // ('DM Spaces') wins in `buildTitles`, so it is a safety net, never the rendered title.
  'dm-spaces': 'DM Spaces',
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

/** A system region's default dock: where it lands when a loaded layout does not contain it. */
export interface RegionPlacement { target: RegionId; edge: Edge }

/** D-114 §9's re-inject rule — where a system region docks when a loaded layout does not contain it.
 *  THE TABLE IS THE DOMAIN (D-c): a system region is re-injectable iff it has a row here — because neither
 *  `REGION_IDS` nor the plugin list carries a target or an edge (N-091, an unfed branch). Every future system
 *  region gets the rule free by adding one row. `target: 'spaces'`, `edge: 'bottom'` is Joe's ①-B placement:
 *  the DM home docks under R1 Spaces. `insertBeside` sibling-bisects the target's slot (mutate.ts sibling
 *  branch) so R1 halves on first boot — EXPECTED, Joe rules B-a (SHIP THE BISECT); once he drags the seam
 *  M-RP7.5's feeder persists the 9-leaf tree and the re-inject no-ops by reference forever after (F8-a). */
export const SYSTEM_REGION_PLACEMENT: Record<RegionId, RegionPlacement> = {
  'dm-spaces': { target: 'spaces', edge: 'bottom' },
};

/** Re-inject every placement-declaring system region the layout is missing (D-114 §9). Pure, TOTAL,
 *  IDEMPOTENT: `insertLeaf` no-ops by reference on an already-docked id AND on a missing target (mutate.ts:266),
 *  so this is free to run on EVERY load and can never double-place, blank the centre (N-095), or fight a user
 *  who moved the region elsewhere. The guard requires a MOUNTED `surface: 'region'` plugin for the id, NOT
 *  mere registry membership — `buildWidgetRegistry` maps every `REGION_IDS` entry to `RegionPlaceholder`, so a
 *  registry test would inject a leaf that paints a placeholder. The stricter test guarantees a real widget. */
export function reinjectSystemRegions(layout: Layout, plugins: PluginDescriptor[]): Layout {
  const mountedRegionIds = new Set(
    plugins.filter((p) => p.surface === 'region' && p.regionId).map((p) => p.regionId as string),
  );
  let out = layout;
  for (const [regionId, place] of Object.entries(SYSTEM_REGION_PLACEMENT)) {
    if (!mountedRegionIds.has(regionId)) continue; // no mounted widget → no leaf (never inject a W-13 drop)
    out = insertLeaf(out, regionId, place.target, place.edge);
  }
  return out;
}

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
 *
 * M-RP-MEMBER-ACT Leg E-2 (D-114 §9): takes the mounted `plugins` and routes BOTH former returns through
 * `reinjectSystemRegions` (a SINGLE EXIT, F2). A system region (today: the DM home) absent from a loaded OR
 * default tree is re-injected. `DEFAULT_LAYOUT` stays at eight leaves (Joe's ①-B), so re-injection is the
 * ONLY path that ever places `dm-spaces` — which is exactly why it must wrap the DEFAULT return too, not just
 * the persisted one (a re-inject on one of two returns is a home that appears only if a file exists). The
 * re-inject is idempotent + TOTAL, so `loadLayout` still cannot return null and the centre still cannot blank.
 */
export async function loadLayout(plugins: PluginDescriptor[]): Promise<Layout> {
  let loaded: Layout = DEFAULT_LAYOUT;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const raw = await invoke<string>('get_ui_state');
    if (raw && raw.trim()) {
      const persisted = JSON.parse(raw)?.session?.layout;
      // `migrateLayout` subsumes the old `isValidLayout` guard AND upgrades a v1/v2 boolean-`collapsed` tree
      // to v3 (M-RP7.1b). It NEVER returns null (N-095 — a malformed/older layout falls back to DEFAULT, so
      // the centre never blanks; D-115). DEFAULT_LAYOUT is injected because `core` must not own a default.
      if (persisted) loaded = migrateLayout(persisted, DEFAULT_LAYOUT);
    }
  } catch (_) {
    // no-Tauri OR corrupt store → DEFAULT (N-095). A read/parse error must never blank the centre.
  }
  // D-114 §9 — the re-inject wraps BOTH former returns (F2). Idempotent + TOTAL, so it cannot blank or double-place.
  return reinjectSystemRegions(loaded, plugins);
}
