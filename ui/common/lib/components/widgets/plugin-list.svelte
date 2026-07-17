<script lang="ts">
  // plugin-list — the FIFTH widget (M-RP6.1l), and the gear's pane. `kind: system` (W-13,
  // non-removable), `surface: none` (W-12): it is CONTENT rendered inside a host (a shell dialog now,
  // Settings later — surfaces §3.2, S-2 "one component, two mounts"), so it spends no surface and is
  // NOT a region leaf. Level-2 marked via `data-tier="widget"` on the root (the self-panel /
  // inspector-panel shape).
  //
  // IT RENDERS THE ACTIVE REGISTRY, AND NOTHING MORE (D1/D5). The active list (M-RP-CONNSTATS) is the
  // client's own compiled system plugins PLUS the customs the user has installed — one reactive source
  // (`installed.active`), several readers (this list + the shell's derived widgetRegistry), N-096. It lists
  // ONLY what is loaded (no node-plugin read verb; M-RP-PLUGINS-NODE is filed, not smuggled in). System rows:
  // self-panel · inspector-panel · plugin-list (it lists itself) · grid-plate; a `custom` row appears iff it
  // is installed (e.g. connection-stats), disappears on uninstall. A list that faked a universal registry
  // would be worse than an honest one.
  //
  // THE ACTION ROW (M-RP-SETTINGS Leg B). Read-only is SUPERSEDED: each row now carries an
  // [info][settings][disable][uninstall] action bar + a leading host kind-glyph (this IS M-RP6.1m,
  // landing per-line inside Settings' Plugins section). W-3 HELD: this widget stays `$common` and imports
  // no shell dep — the buttons emit through ONE callback seam `onAction(id, verb)`; the SHELL
  // (settings-dialog → app_client) wires it (info → an in-modal detail view, disable/uninstall → the
  // D-119 leaf primitives). Every button's state is DESCRIPTOR-DERIVED and greyed only for a reason true
  // of that plugin (W-13 rendered), never dead-grey for a missing verb (J-500 / 6.1j). Appearance — the
  // glyph shapes, colours, row density — is PROVISIONAL → M-RP-SKIN; the mechanic (which control, what
  // feeds it, when it is honest) is what this leg fixes. The `[system]`/`[user]` badge is KEPT: it conveys
  // KIND (built-in vs installed), a DIFFERENT axis from the host glyph (module vs widget) — dropping it
  // would lose info; whether it survives is Joe's M-RP-SKIN call.
  import { envelope } from '$common/components/base/envelope';
  import { type PluginDescriptor } from '$common/plugins/registry';
  import { installed } from '$common/plugins/installed.svelte';
  import Label from '$core/components/data-independent/label.svelte';
  import Icon from '$core/components/data-independent/icon.svelte';

  // The one action verb set (order Joe-set). `onAction` is the W-3 seam the shell wires; absent when the
  // host doesn't provide it (the list still renders — the About-section reuse is read-only, no seam).
  type PluginVerb = 'info' | 'settings' | 'disable' | 'uninstall';

  // Mounted directly by the host (settings-dialog's Plugins section), not by `region-node` — so it takes a
  // plain `id` (default `plugin-list`), from which the composed row-value Labels + action-glyph icons derive
  // their ids. `onAction` is the single seam every row button emits through.
  let {
    id = 'plugin-list',
    onAction,
  }: { id?: string; onAction?: (pluginId: string, verb: PluginVerb) => void } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // Alphabetical (D8) — the list is not a priority indicator (Ch6 §6.8.5). Reads the reactive active list,
  // so an installed custom appears/disappears live; `.active` already returns a fresh array (never mutated).
  const plugins = $derived(
    [...installed.active].sort((a, b) => a.name.localeCompare(b.name)),
  );

  // Aggregate getter G (W-4). rowCount is RENDER-TRUTH: counted from `plugins` (what the {#each}
  // renders), the message.detailsCount precedent. system/custom split so the badge is observable off G
  // as well as off the painted DOM.
  const systemCount = $derived(plugins.filter((p) => p.kind === 'system').length);
  const customCount = $derived(plugins.filter((p) => p.kind === 'custom').length);
  const debug = () => ({ count: plugins.length, systemCount, customCount });

  const rid = (p: PluginDescriptor, s: string) => cid(`${p.id}__${s}`);

  // The leading PLUGIN icon (Joe: "there has to be a plugin icon"): the plugin's own identity glyph from its
  // descriptor (`icon`), NOT a host-kind glyph. The which-glyph is per-plugin; its COLOUR is host-derived in
  // the skin (`.pl-icon[data-host]` — module red / widget blue, Joe's cue). Falls back to a neutral
  // placeholder when a plugin declares no icon.
  const pluginIcon = (p: PluginDescriptor): string => p.icon ?? 'square';

  // Per-row action-button model (§4 feeder table). Every state is DESCRIPTOR-DERIVED here — a button is
  // greyed (aria-disabled, kept focusable & legible — the shelf-face / menu-item pattern) only for a reason
  // TRUE of the plugin, never because a verb is missing. Glyphs (Joe-set): info = the little-i info glyph ·
  // settings = gear · disable = a SWITCH (toggle-on when mounted / toggle-off when disabled — the switch
  // shows the live state, clicking flips it) · uninstall = the recycle-bin delete glyph. The tooltip carries
  // the meaning. A built-in cannot be disabled or removed (W-13); uninstall-on-system ships GREYED-LEGIBLE
  // (the W-13-rendered reading, J-513) — absent-vs-greyed is Joe's M-RP-SKIN call. `settings` greys for ALL
  // this leg (no plugin ships a settingsComponent yet) — the honest "no settings", not a missing verb.
  type PluginAction = { verb: PluginVerb; icon: string; disabled: boolean; title: string };
  function actionsFor(p: PluginDescriptor): PluginAction[] {
    const isCustom = p.kind === 'custom';
    const dis = installed.isDisabled(p.id);
    const hasSettings = !!p.settingsComponent;
    return [
      { verb: 'info', icon: 'info', disabled: false, title: 'Details' },
      {
        verb: 'settings',
        icon: 'gear',
        disabled: !hasSettings,
        title: hasSettings ? 'Settings' : 'No settings',
      },
      {
        verb: 'disable',
        icon: dis ? 'toggle-off' : 'toggle-on',
        disabled: !isCustom,
        title: !isCustom ? 'Built-in — cannot be disabled' : dis ? 'Enable' : 'Disable',
      },
      {
        verb: 'uninstall',
        icon: 'delete',
        disabled: !isCustom,
        title: !isCustom ? 'Built-in — cannot be removed' : 'Uninstall',
      },
    ];
  }
</script>

<div class="plugin-list" data-tier="widget" use:envelope={{ name: 'plugin-list', id, debug }}>
  <ul class="plugin-list-items">
    {#each plugins as p (p.id)}
      <!-- ONE line per plugin (Joe): [plugin-icon] name [badge] … [actions]. Description + the
        host·delivery·surface axes live in the [info] detail view, NOT in the list. -->
      <li class="plugin-list-row">
        <!-- Leading PLUGIN icon: the plugin's own identity glyph, coloured per host in skin (PROVISIONAL). -->
        <span class="pl-icon" data-host={p.host}>
          <Icon name={pluginIcon(p)} id={rid(p, 'icon')} />
        </span>
        <span class="pl-name"><Label text={p.name} id={rid(p, 'name')} /></span>
        <!-- Version (Joe): replaces the [system]/[user] badge — kind is already the icon colour + the info
          view, so the badge was redundant. The version is the plugin's own (declared today; D-118 manifest
          later). Plain text, not a registry Label — it's a small metadata glyph, read off the painted DOM. -->
        {#if p.version}
          <span class="pl-version">v{p.version}</span>
        {/if}
        <!-- The action bar (§3.3): icon-buttons + hover tooltips, one `onAction` seam (W-3). aria-disabled
          (not native) keeps a greyed button focusable & legible (the shelf-face / menu-item pattern); the
          guarded onclick suppresses a disabled fire. data-verb/data-plugin are the CDP driving/verify hooks. -->
        <div class="pl-actions" data-plugin-action>
          {#each actionsFor(p) as a (a.verb)}
            <button
              type="button"
              class="pl-action"
              data-verb={a.verb}
              data-plugin={p.id}
              aria-disabled={a.disabled || undefined}
              aria-label={a.title}
              title={a.title}
              onclick={() => !a.disabled && onAction?.(p.id, a.verb)}
            ><Icon name={a.icon} id={rid(p, a.verb)} /></button>
          {/each}
        </div>
      </li>
    {/each}
  </ul>
</div>
