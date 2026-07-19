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
import SpacesPanel from '$common/components/widgets/spaces-panel.svelte';
import RoomsPanel from '$common/components/widgets/rooms-panel.svelte';
import StreamPanel from '$common/components/widgets/stream-panel.svelte';
import GridPlate from '$common/components/widgets/grid-plate.svelte';
import GridPlateSettings from '$common/components/widgets/grid-plate-settings.svelte';
import ConnectionStats from '$common/components/widgets/connection-stats.svelte';
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
  /** The plugin's version, shown on its list line (M-RP-SETTINGS Leg B; Joe: version, not a redundant
   *  [system] badge — kind is already the icon colour + the info view). A declared string today (our
   *  compiled built-ins are all at v1.0.0); the real per-plugin version arrives from the D-118 package
   *  manifest / M-RP-PLUGINS-NODE. */
  version?: string;
  kind: PluginKind;
  host: PluginHost;
  delivery: PluginDelivery;
  surface: PluginSurface;
  /** The plugin's own leading glyph name (icons.ts key) — its identity icon in the manager list
   *  (M-RP-SETTINGS Leg B; Joe: "there has to be a plugin icon"). Chosen to match the plugin's purpose;
   *  the list falls back to a neutral placeholder when unset. Appearance (the mapping) is PROVISIONAL →
   *  M-RP-SKIN. Its COLOUR is host-derived in the skin (module = red · widget = blue). */
  icon?: string;
  /** iff surface === 'region' — the D-103 leaf it occupies (regionId === widgetId, N-100). */
  regionId?: string;
  /** iff it has a surface the shell mounts via the descriptor (i.e. a region). */
  component?: Component;
  /** The plugin's own settings component (M-RP-SETTINGS Leg B, D-B). Its PRESENCE is `hasSettings`:
   *  the row's [settings] button enables iff this is set. UNDEFINED on every row this leg — so the
   *  button is greyed for all, for the real per-plugin reason "this plugin has no settings", never a
   *  missing verb (J-500 / 6.1j: an unbuilt verb ships absent, a plugin-true fact ships greyed-legible).
   *  `grid-plate` earns one first at Leg C (its backdrop setting); the modal hosts it in the content
   *  pane (D-B — component-per-plugin; NOT a declarative schema). */
  settingsComponent?: Component;
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
    version: '1.0.0',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    surface: 'region',
    regionId: 'self',
    icon: 'person',
    component: SelfPanel,
  },
  {
    id: 'inspector-panel',
    name: 'Inspector Panel',
    description: 'Details of the current selection.',
    version: '1.0.0',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    surface: 'region',
    regionId: 'inspector',
    icon: 'search',
    component: InspectorPanel,
  },
  {
    id: 'spaces-panel',
    name: 'Spaces',
    description: 'The Spaces you have joined.',
    version: '1.0.0',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    surface: 'region',
    regionId: 'spaces',
    // icon UNSET (M-RP6.2 D8): there is no verified spaces glyph in-repo, and a Material `d` path is not
    // fabricated from memory (Rule 5 / the byte-for-byte icon discipline, D-108). plugin-list falls back to
    // its documented `'square'` placeholder; the real glyph is deferred to M-RP-ICON-ADOPT / M-RP-SKIN.
    component: SpacesPanel,
  },
  {
    id: 'rooms-panel',
    name: 'Rooms',
    description: 'The Rooms in the selected Space.',
    version: '1.0.0',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    surface: 'region',
    regionId: 'rooms',
    // icon UNSET (M-RP6.2 D8) — see spaces-panel.
    component: RoomsPanel,
  },
  {
    id: 'stream-panel',
    name: 'Messages',
    description: 'The live message stream for the selected Room.',
    version: '1.0.0',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    surface: 'region',
    regionId: 'stream',
    // icon UNSET (M-RP6.3 Leg C2, the M-RP6.2 D8 discipline): no verified message glyph in-repo, and a
    // Material `d` path is not fabricated from memory (Rule 5 / D-108). plugin-list falls back to its
    // documented placeholder; the real glyph is deferred to M-RP-ICON-ADOPT / M-RP-SKIN.
    component: StreamPanel,
  },
  {
    id: 'plugin-list',
    name: 'Plugin List',
    description: 'The plugins loaded in this client.',
    version: '1.0.0',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    icon: 'extension',
    // surface: 'none', NO component — the list is CONTENT inside a host (a dialog now, Settings later,
    // S-2/§3.2), so it spends no surface (W-12) and the shell mounts it DIRECTLY (not via a registry).
    surface: 'none',
  },
  {
    id: 'grid-plate',
    name: 'Grid Backdrop',
    description: 'The backdrop shown behind the grid, in the gaps between panels.',
    version: '1.0.0',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    icon: 'wallpaper',
    // surface: 'none', WITH component — the FIRST such row (M-RP-PLATE). It spends no surface (W-12) but
    // IS content the shell mounts into a NAMED host socket: the grid-wide background socket on region-shell
    // (the `message-stream` `background` socket, one level up). layout-default DERIVES a `bgWidgets` map
    // from the `surface: 'none' && component` rows — the `widgetRegistry` shape, one socket over (N-096).
    surface: 'none',
    component: GridPlate,
    // The FIRST `settingsComponent` (M-RP-SETTINGS Leg C, D-B). This single assignment lights the [settings]
    // button on the Grid Backdrop row (`hasSettings = !!settingsComponent`, plugin-list) and NOWHERE else;
    // the Settings modal hosts it in its content pane (component-per-plugin — NOT a declarative schema).
    settingsComponent: GridPlateSettings,
  },
];

// AVAILABLE_CUSTOM — the compiled custom (`kind: 'custom'`) plugins the user MAY install (M-RP-CONNSTATS).
// The FIRST runtime-installable rows: unlike CLIENT_PLUGINS (always present), these enter the active set only
// when installed (installed.svelte.ts). NOT folded into CLIENT_PLUGINS — a listed-but-uninstalled plugin
// would be a widget-in-the-registry the client never mounts (the N-091 unfed-branch shape). `delivery:
// 'compiled'` = in-tree, our own binary (D-085): "install" here means REGISTER + INJECT the layout leaf, not
// dlopen a library — a compiled plugin is not a loader. A packaged/service custom is a later delivery kind
// (S-7: none loads until the sandbox floor ships).
export const AVAILABLE_CUSTOM: PluginDescriptor[] = [
  {
    id: 'connection-stats',
    name: 'Connection Stats',
    description: 'Live connection state and endpoint.',
    version: '1.0.0',
    kind: 'custom',
    host: 'client',
    delivery: 'compiled',
    surface: 'region',
    regionId: 'connection-stats',
    icon: 'signal',
    component: ConnectionStats,
  },
];
