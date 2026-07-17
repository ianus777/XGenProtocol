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

// The DISABLED set (M-RP-SETTINGS Leg B, D-B). A SECOND runtime axis, orthogonal to installed: a disabled
// custom stays INSTALLED (still LISTED in the plugin list, still persisted) but its widget UNMOUNTS. Same
// fresh-Set-reassignment reactivity as `_installed`. Only ever holds ids that are also installed; uninstall
// clears the flag (a forgotten plugin is not "disabled"). Persisted per-device as `session.disabled` — the
// `session.installed` precedent (the shell owns the persist; §4).
let _disabled = $state<Set<string>>(new Set());

export const installed = {
  /** The installed custom ids (a copy — the source set is never handed out mutable). Reactive. */
  get ids(): string[] {
    return [..._installed];
  },
  /** Is this custom plugin installed? Reactive. */
  isInstalled(id: string): boolean {
    return _installed.has(id);
  },
  /** The disabled custom ids (a copy). Reactive. What the shell persists as `session.disabled`. */
  get disabledIds(): string[] {
    return [..._disabled];
  },
  /** Is this custom plugin installed-but-disabled? Reactive. Drives the row's Disable/Enable label and
   *  keeps a disabled custom out of `mounted` while leaving it in `active`. */
  isDisabled(id: string): boolean {
    return _disabled.has(id);
  },
  /** The reactive LISTED plugin list: the always-present system rows + the installed customs (INCLUDING
   *  disabled ones — a disabled plugin stays listed, shown disabled). `plugin-list` renders THIS. Reactive. */
  get active(): PluginDescriptor[] {
    return [...CLIENT_PLUGINS, ...AVAILABLE_CUSTOM.filter((p) => _installed.has(p.id))];
  },
  /** The reactive MOUNTED plugin list: system rows + installed customs NOT in the disabled set. The shell
   *  derives `widgetRegistry`/`bgWidgets`/titles from THIS, so a disabled custom's widget unmounts (and can
   *  never resolve, even from a stale leaf — the belt to the shell's leaf-removal braces, §3.2). Reactive. */
  get mounted(): PluginDescriptor[] {
    return [
      ...CLIENT_PLUGINS,
      ...AVAILABLE_CUSTOM.filter((p) => _installed.has(p.id) && !_disabled.has(p.id)),
    ];
  },
  /** Register a custom plugin as installed (no-op if unknown or already installed). The SHELL wraps this to
   *  also inject the layout leaf + persist (§4.7); the store only owns the set. */
  install(id: string): void {
    if (!isAvailable(id) || _installed.has(id)) return;
    const next = new Set(_installed);
    next.add(id);
    _installed = next;
  },
  /** Deregister a custom plugin (no-op if not installed). The SHELL wraps this to also remove the leaf.
   *  Also clears the disabled flag: a forgotten plugin is not "disabled", and a later re-install must not
   *  come back disabled from a stale flag. */
  uninstall(id: string): void {
    if (!_installed.has(id)) return;
    const next = new Set(_installed);
    next.delete(id);
    _installed = next;
    if (_disabled.has(id)) {
      const nd = new Set(_disabled);
      nd.delete(id);
      _disabled = nd;
    }
  },
  /** Mark an installed custom as DISABLED (no-op if not installed or already disabled). The SHELL wraps this
   *  to also REMOVE the layout leaf (the D-119 removeRegion path) + persist. The store only owns the flag. */
  disable(id: string): void {
    if (!_installed.has(id) || _disabled.has(id)) return;
    const next = new Set(_disabled);
    next.add(id);
    _disabled = next;
  },
  /** Clear the DISABLED flag (no-op if not disabled). The SHELL wraps this to also RE-INJECT the leaf
   *  (insertLeaf) + persist. */
  enable(id: string): void {
    if (!_disabled.has(id)) return;
    const next = new Set(_disabled);
    next.delete(id);
    _disabled = next;
  },
  /** Boot seed (§4.7): install the persisted ids BEFORE `loadLayout` resolves, so a persisted custom leaf
   *  finds its registered widget instead of W-13-dropping. Unknown ids are filtered (a retired custom must
   *  not resurrect a leaf — the W-13 tolerance one level up). */
  hydrate(ids: string[]): void {
    _installed = new Set((ids ?? []).filter(isAvailable));
  },
  /** Boot seed for the disabled set (§4.7). Runs AFTER `hydrate` and BEFORE `loadLayout`: filtered to ids
   *  that are BOTH available AND installed, so a disabled-but-uninstalled id (or an unknown id) cannot mark a
   *  phantom. A persisted disabled custom is then excluded from `mounted` before the layout resolves. */
  hydrateDisabled(ids: string[]): void {
    _disabled = new Set((ids ?? []).filter((id) => isAvailable(id) && _installed.has(id)));
  },
};
