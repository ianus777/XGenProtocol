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
import SelfPanel from '$common/components/widgets/self-panel.svelte';

// All 8 D-103 region ids (region-dock §2), in the default row order.
export const REGION_IDS = [
  'spaces', 'rooms', 'self', 'room-header', 'stream', 'composer', 'members', 'inspector',
] as const;

// The registry map: every id → the placeholder, then each real widget replaces ONE entry as it lands
// (M-RP6.1g swaps `self` → SelfPanel — the first real leaf; this is exactly what renderer A's
// prop-injected registry was built for, N-093). The remaining 7 stay placeholders until their milestones.
export const widgetRegistry: Record<string, Component> = {
  ...Object.fromEntries(REGION_IDS.map((id) => [id, RegionPlaceholder])),
  self: SelfPanel,
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
 * Load the active layout. D2 seam: async so M-RP7.3 becomes a one-line body swap to `invoke('get_layout')`
 * (Rust persists the tree as an opaque blob — it never learns the node shape). Returns the default today.
 */
export async function loadLayout(): Promise<Layout> {
  return DEFAULT_LAYOUT;
}
