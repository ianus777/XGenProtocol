// installed.svelte.ts — the RUNTIME installed-set for custom (`kind: 'custom'`) plugins (M-RP-CONNSTATS,
// D1). The registry (`registry.ts`) is STATIC: `CLIENT_PLUGINS` is a const, always present. This module is
// the runtime half — which of the AVAILABLE custom plugins the user has installed THIS session, and a
// reactive `active` list that is `[...CLIENT_PLUGINS, ...installed customs]`.
//
// ONE reactive source, several readers (N-096): the shell DERIVES the region `widgetRegistry` / `bgWidgets`
// / titles from `active`, and the `plugin-list` widget lists `active`. A custom widget is in the grid, and
// in the plugin list, BECAUSE it is installed — not because a literal map said so. This is the exact shape
// `layout-default` already used against the static `CLIENT_PLUGINS`, now made runtime-reactive.
//
// A `$common` store because both readers that need it are `$common` (`plugin-list`) or shell-derived, and
// W-3 forbids a `common` reader importing a shell dep; the store home is `$common` so the node inherits it
// at M-RP7.7. A `.svelte.ts` module so its module-level `$state` participates in Svelte 5 reactivity (the
// `selection.svelte` / `self-state.svelte` getter-over-`$state` precedent).
//
// The store owns the SET only. Installing/uninstalling ALSO injects/removes the layout leaf and persists
// the set — that is the shell's job (app_client, §4.7), because the leaf mutation needs the shell-local
// `layout` state and the persistence needs the shell-local uistate store. So `__XGEN_PLUGINS__` lives in
// the shell (wrapping these), NOT here.

import { CLIENT_PLUGINS, AVAILABLE_CUSTOM, type PluginDescriptor } from './registry';

const isAvailable = (id: string): boolean => AVAILABLE_CUSTOM.some((p) => p.id === id);

// Module-level `$state`. Reassigned (never `.add`/`.delete` in place): Svelte 5 does not make a native Set
// reactive on its methods, so a fresh Set on every mutation is what a `$derived`/template read tracks. The
// dependency is on the `_installed` binding itself (read to call `.has`), so reassignment invalidates.
let _installed = $state<Set<string>>(new Set());

export const installed = {
  /** The installed custom ids (a copy — the source set is never handed out mutable). Reactive. */
  get ids(): string[] {
    return [..._installed];
  },
  /** Is this custom plugin installed? Reactive. */
  isInstalled(id: string): boolean {
    return _installed.has(id);
  },
  /** The reactive active plugin list: the always-present system rows + the installed customs. The shell
   *  derives `widgetRegistry`/`bgWidgets`/titles from this; `plugin-list` renders it. Reactive. */
  get active(): PluginDescriptor[] {
    return [...CLIENT_PLUGINS, ...AVAILABLE_CUSTOM.filter((p) => _installed.has(p.id))];
  },
  /** Register a custom plugin as installed (no-op if unknown or already installed). The SHELL wraps this to
   *  also inject the layout leaf + persist (§4.7); the store only owns the set. */
  install(id: string): void {
    if (!isAvailable(id) || _installed.has(id)) return;
    const next = new Set(_installed);
    next.add(id);
    _installed = next;
  },
  /** Deregister a custom plugin (no-op if not installed). The SHELL wraps this to also remove the leaf. */
  uninstall(id: string): void {
    if (!_installed.has(id)) return;
    const next = new Set(_installed);
    next.delete(id);
    _installed = next;
  },
  /** Boot seed (§4.7): install the persisted ids BEFORE `loadLayout` resolves, so a persisted custom leaf
   *  finds its registered widget instead of W-13-dropping. Unknown ids are filtered (a retired custom must
   *  not resurrect a leaf — the W-13 tolerance one level up). */
  hydrate(ids: string[]): void {
    _installed = new Set((ids ?? []).filter(isAvailable));
  },
};
