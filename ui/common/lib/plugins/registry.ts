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
import ComposerPanel from '$common/components/widgets/composer-panel.svelte';
import GridPlate from '$common/components/widgets/grid-plate.svelte';
import GridPlateSettings from '$common/components/widgets/grid-plate-settings.svelte';
import ConnectionStats from '$common/components/widgets/connection-stats.svelte';
import SubstitutionsEditor from '$common/components/widgets/substitutions-editor.svelte';
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

/** The props `region-node` passes a REGION widget — grounded against region-node.svelte: it passes
 *  `regionId` and nothing else (an earlier runbook draft assumed `{regionId, id}`; it is not that).
 *  A widget may declare further OPTIONAL props (`self-panel`'s self-derived `id`) — optional props
 *  stay assignable, which is exactly why this contract can be stated without touching a widget. */
export type RegionWidgetProps = { regionId: string };

/** The props `region-shell` passes a BACKGROUND mount (M-RP-PLATE) — the `backgroundLive` settings
 *  switch, alongside the mount's own `WidgetMount.props`. Distinct from a region contract: a
 *  backdrop has no region, no tile and no `regionId`. */
export type BackgroundWidgetProps = { backgroundLive?: boolean };

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
  /** iff it has a surface the shell mounts via the descriptor (i.e. a region), OR it is a
   *  `surface: 'none'` row the shell mounts into a named host socket (the grid-plate shape).
   *
   *  ⚠️ TYPED AS THE UNION OF THE TWO REAL MOUNT CONTRACTS (M-RP-TYPECHECK, root cause A).
   *  A bare `Component` types this as *"a component taking NO props"*, which is false for every
   *  region row: `region-node` mounts them WITH a required `regionId`, so six of the eight
   *  assignments below were type errors nothing could see. `grid-plate` did not error only
   *  because all of ITS props are optional — the field was never checking the contract, it was
   *  checking whether the props happened to be omissible. */
  component?: Component<RegionWidgetProps> | Component<BackgroundWidgetProps>;
  /** The plugin's own settings component (M-RP-SETTINGS Leg B, D-B). Its PRESENCE is `hasSettings`:
   *  the row's [settings] button enables iff this is set — so a row without one is greyed for the real
   *  per-plugin reason "this plugin has no settings", never a missing verb (J-500 / 6.1j: an unbuilt
   *  verb ships absent, a plugin-true fact ships greyed-legible). The modal hosts it in the content
   *  pane (D-B — component-per-plugin; NOT a declarative schema). TWO rows carry one today:
   *  `grid-plate` (M-RP-SETTINGS Leg C, the first) and `text-processing` (M-RP-PROCESSOR-WIRE Leg A).
   *  (This sentence read "UNDEFINED on every row this leg" until Leg C fed the first one.) */
  settingsComponent?: Component;
}

// THREE honest rows (D5). All `host: client, delivery: compiled, kind: system` — they are our own
// binary's built-in widgets. NOT listed: `entity-context-menu` — the client never instantiates it
// (sampler-only), and registering an unmounted plugin is the unfed-branch shape (N-091). It enters the
// registry at the milestone that mounts it.
// (`substitutions-editor` WAS on that not-listed line until M-RP-PROCESSOR-WIRE Leg A: the client now
// instantiates it as the `Text Processing` row's settings pane, so it is no longer unfed.)
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
    id: 'composer-panel',
    name: 'Composer',
    description: 'Write and send a message to the selected Room.',
    version: '1.0.0',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    surface: 'region',
    regionId: 'composer',
    // icon UNSET (M-RP6.3 Leg D2, the M-RP6.2 D8 discipline): no verified composer glyph in-repo, and a
    // Material `d` path is not fabricated from memory (Rule 5 / D-108). plugin-list falls back to its
    // documented placeholder; the real glyph is deferred to M-RP-ICON-ADOPT / M-RP-SKIN.
    component: ComposerPanel,
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
    // The FIRST `settingsComponent` (M-RP-SETTINGS Leg C, D-B). This assignment lights the [settings] button
    // on the Grid Backdrop row (`hasSettings = !!settingsComponent`, plugin-list); the Settings modal hosts
    // it in its content pane (component-per-plugin — NOT a declarative schema). (It read "and NOWHERE else"
    // while it was the only one; `text-processing` is the second, M-RP-PROCESSOR-WIRE Leg A.)
    settingsComponent: GridPlateSettings,
  },
  {
    id: 'text-processing',
    name: 'Text Processing',
    description: 'Substitution rules applied to text as you type it.',
    version: '1.0.0',
    kind: 'system',
    host: 'client',
    delivery: 'compiled',
    // THE FIRST ROW THAT IS PURELY A SETTINGS SURFACE (M-RP-PROCESSOR-WIRE §3.4) — `surface: 'none'` with
    // NO `component`: the grid-plate shape minus the thing it mounts. It spends no surface (W-12), and
    // `layout-default` cannot pick it up (its derived `bgWidgets` requires `surface: 'none' && component`),
    // so it contributes a plugin-list row and NOTHING to the grid. The processor engine it configures is
    // an ATTACHMENT spread onto host inputs, not a mounted widget — there is nothing for it to render.
    //
    // `kind: 'system'`, deliberately, and it is a data-safety call rather than a taxonomy one: a `custom`
    // row carries a Disable and an Uninstall (plugin-list, W-13), and this is the only plugin in the
    // project that owns user-authored content — a rule list the user curated by hand. Every existing
    // custom owns none. `system` ⇒ neither button is live ⇒ there is no path that discards it.
    surface: 'none',
    // icon UNSET (M-RP6.2 D8): no verified text/substitution glyph exists in-repo, and a Material `d`
    // path is not fabricated from memory (Rule 5 / D-108). plugin-list falls back to its documented
    // `'square'` placeholder; the real glyph is deferred to M-RP-ICON-ADOPT / M-RP-SKIN.
    //
    // The SECOND `settingsComponent` (after grid-plate). It is the already-shipped M-RP4.3 widget,
    // mounted unchanged through the generic prop-less `<C />`; its durable write-back rides the store's
    // persist seam, which the shell fills at boot (§3.3 — the widget has no prop to receive it through,
    // and cannot import `invoke` itself under W-3).
    settingsComponent: SubstitutionsEditor,
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
