// layout-default.ts — the shell-side default layout, the widget registry map, and the layout SOURCE (D2).
// Shell-local: renderer A + the descriptor type are `core`, but the concrete default tree, the id→component
// map, and the (future Tauri) load seam are the client's.
//
// `loadLayout()` is the D2 seam: async from day one so the M-RP7.3 swap to `invoke('get_layout')` is a
// BODY change, not a call-shape change (and Rust never learns the node shape — it persists an opaque blob).
// Today it just returns DEFAULT_LAYOUT.

import type { Component } from 'svelte';
import type { Layout } from '$core/components/layout/types';
import RegionPlaceholder from './region-placeholder.svelte';
import { CLIENT_PLUGINS } from '$common/plugins/registry';

// All 8 D-103 region ids (region-dock §2), in the default row order.
export const REGION_IDS = [
  'spaces', 'rooms', 'self', 'room-header', 'stream', 'composer', 'members', 'inspector',
] as const;

// The registry map is DERIVED from CLIENT_PLUGINS (M-RP6.1l, D2): every region id → the placeholder,
// then each plugin with `surface: 'region'` replaces ONE entry with its component. A widget is in the
// grid BECAUSE it is a registered region plugin — one source (the registry), two readers (this map +
// the plugin-list widget), the N-096 shape. The literal `self: SelfPanel, inspector: InspectorPanel`
// lines are gone; today the derive yields exactly those two (self-panel M-RP6.1g, inspector-panel
// M-RP6.1h). The remaining 6 regions stay placeholders — a placeholder is scaffolding, not a plugin,
// and it is not listed.
export const widgetRegistry: Record<string, Component> = {
  ...Object.fromEntries(REGION_IDS.map((id) => [id, RegionPlaceholder])),
  ...Object.fromEntries(
    CLIENT_PLUGINS.filter((p) => p.surface === 'region' && p.regionId && p.component).map(
      (p) => [p.regionId as string, p.component as Component],
    ),
  ),
};

// DEFAULT_LAYOUT (D8) — exercises row + col + nesting, all 8 regions, NO unknown id, NO tabs (a broken
// default is not a test fixture; the drop/tabs/mismatch paths are driven at verify via __XGEN_LAYOUT__).
export const DEFAULT_LAYOUT: Layout = {
  version: 1,
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
 * (The session layout is written at Leg D; until then this always falls to DEFAULT — but the SEAM now
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
      if (isValidLayout(layout)) return layout;
    }
  } catch (_) {
    // no-Tauri OR corrupt store → DEFAULT (N-095). A read/parse error must never blank the centre.
  }
  return DEFAULT_LAYOUT;
}

/** Minimal shape guard so a malformed persisted layout falls back instead of unmounting the shell. */
function isValidLayout(l: unknown): l is Layout {
  if (!l || typeof l !== 'object') return false;
  const o = l as { version?: unknown; root?: unknown };
  return typeof o.version === 'number' && !!o.root && typeof o.root === 'object';
}
