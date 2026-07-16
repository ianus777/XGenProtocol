// registry.ts — the client plugin registry (M-RP6.1l, D1). D-112's THREE AXES in code for the
// first time: `host` (system | ui area) · `delivery` (where trust lives) · `surface` (at most one,
// W-12). There is NO registry to enumerate today — `xgen-common/src/module.rs`'s `Descriptor` has
// none of these fields, the node-side plugins have no client verb, and no manifest exists (grounded
// 2026-07-12, J-512). So this file CREATES the first registry, in TS, in the frontend, and lists
// exactly what is real: the client's own `host: client` compiled plugins.
//
// W-3 holds: this file lives in `$common` and imports only `$common` widgets — never a shell dep.
// The client cannot see a single `host: node` plugin, and this milestone does not invent a way to
// (that needs a Rust/protocol read verb, filed as M-RP-PLUGINS-NODE). So there are NO `host: 'node'`
// rows here — a placeholder row for one would be the unfed-branch shape (N-091). They enter when the
// verb lands.
//
// TWO READERS, ONE SOURCE (N-096): the plugin-list widget lists these rows; `layout-default.ts`
// DERIVES the region `widgetRegistry` from the `surface === 'region'` rows (D2). A widget is in the
// grid *because* it is a registered plugin with a region surface — not because a literal map said so.

import type { Component } from 'svelte';
import SelfPanel from '$common/components/widgets/self-panel.svelte';
import InspectorPanel from '$common/components/widgets/inspector-panel.svelte';
import GridPlate from '$common/components/widgets/grid-plate.svelte';
// plugin-list.svelte is NOT imported here: its descriptor carries no `component` (surface: 'none' →
// it is content the shell mounts inside a host, never resolved through this registry), so importing
// it would be both unused and a needless circular import (it reads CLIENT_PLUGINS at runtime).
//
// TWO SHAPES of `surface: 'none'` (M-RP-PLATE): a `none` row spends no surface (W-12), but it may still be
// CONTENT the shell mounts inside a host (§3.2). NO-component (`plugin-list`) → the shell mounts it directly
// (into a dialog now, Settings later). WITH-component (`grid-plate`) → the shell mounts it into a NAMED host
// socket (the grid-wide background socket on `region-shell`) via a DERIVED registry (layout-default), the
// `widgetRegistry` shape one socket over.

/** system = the node/system area · client = the ui area (Joe's frame; already the words in module.rs). */
export type PluginHost = 'node' | 'client';
/** where trust lives (D-112 §10): compiled = our binary · service = an XGID at an endpoint · packaged = third-party (S-7: none can load until the sandbox floor ships). */
export type PluginDelivery = 'compiled' | 'service' | 'packaged';
/** at most one (W-12): none = headless · region = a D-103 leaf · shelf = a face · window = its own OS window. */
export type PluginSurface = 'none' | 'region' | 'shelf' | 'window';
/** W-13: system => non-removable (built-in); custom => install/remove. */
export type PluginKind = 'system' | 'custom';

export interface PluginDescriptor {
  /** Stable local id. */
  id: string;
  /** Display name — also the alphabetical sort key (D8, Ch6 §6.8.5: the list is not a priority indicator). */
  name: string;
  description?: string;
  kind: PluginKind;
  host: PluginHost;
  delivery: PluginDelivery;
  surface: PluginSurface;
  /** iff surface === 'region' — the D-103 leaf it occupies (regionId === widgetId, N-100). */
  regionId?: string;
  /** iff it has a surface the shell mounts via the descriptor (i.e. a region). */
  component?: Component;
}

// THREE honest rows (D5). All `host: client, delivery: compiled, kind: system` — they are our own
// binary's built-in widgets. NOT listed: `substitutions-editor` / `entity-context-menu` — the client
// never instantiates them (sampler-only), and registering an unmounted plugin is the unfed-branch
// shape (N-091). They enter the registry at the milestone that mounts them (M-RP-SETTINGS).
export const CLIENT_PLUGINS: PluginDescriptor[] = [
  {
    id: 'self-panel',
    name: 'Self Panel',
    description: 'Your identity and connection status.',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    surface: 'region',
    regionId: 'self',
    component: SelfPanel,
  },
  {
    id: 'inspector-panel',
    name: 'Inspector Panel',
    description: 'Details of the current selection.',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    surface: 'region',
    regionId: 'inspector',
    component: InspectorPanel,
  },
  {
    id: 'plugin-list',
    name: 'Plugin List',
    description: 'The plugins loaded in this client.',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    // surface: 'none', NO component — the list is CONTENT inside a host (a dialog now, Settings later,
    // S-2/§3.2), so it spends no surface (W-12) and the shell mounts it DIRECTLY (not via a registry).
    surface: 'none',
  },
  {
    id: 'grid-plate',
    name: 'Grid Backdrop',
    description: 'The backdrop shown behind the grid, in the gaps between panels.',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    // surface: 'none', WITH component — the FIRST such row (M-RP-PLATE). It spends no surface (W-12) but
    // IS content the shell mounts into a NAMED host socket: the grid-wide background socket on region-shell
    // (the `message-stream` `background` socket, one level up). layout-default DERIVES a `bgWidgets` map
    // from the `surface: 'none' && component` rows — the `widgetRegistry` shape, one socket over (N-096).
    surface: 'none',
    component: GridPlate,
  },
];
