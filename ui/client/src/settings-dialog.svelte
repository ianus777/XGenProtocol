<script>
  // settings-dialog — SHELL-LOCAL (M-RP-SETTINGS Leg A). The ONE Settings surface (D-C): a single in-DOM
  // modal shaped like Discord's — a left category menu (~¼ width) + a content pane that swaps per
  // selection. Wraps the core `dialog` (C1), exactly as about-dialog / uistate-*-dialog do; lives in
  // ui/client/ (shell chrome, not `core`).
  //
  // D-A: `surface:'window'` = a standalone in-DOM MODAL, not a second OS window. So there is no frame arc
  // here — just another <dialog> in the one window.
  //
  // TWO ENTRY POINTS, ONE MODAL (D-C): the gear (widget.manager) opens Settings landed on `plugins`;
  // File ▸ Settings (settings.open) opens it on the default (first) section. The deep-link is the
  // `section` prop; app_client sets it per entry point.
  //
  // THE SIDEBAR ROVER — a shell-local linear-rove COPY (selection-follows-focus), NOT a reuse of a core
  // rover and NOT the M-RP-ROVING extraction (runbook §3.2). Neither core rover fits cleanly: entity-panel
  // is the right interaction (roving single-select) but data-dependent (it renders entities/avatars, wrong
  // domain); shelf is the right chrome family but the wrong interaction (a horizontal transient
  // command-dispatch toolbar, not a persistent vertical single-select nav). So the sidebar is a small
  // shell-local nav — chrome, not a new `core` component (the sampler catalogue is untouched), and
  // M-RP-ROVING stays filed. The buttons carry data-section for CDP driving (a synthetic .click() fires a
  // plain onclick fine — only synthetic KEY events are untrusted, J-496).
  //
  // THE CONTENT PANE — all sections MOUNTED, inactive ones display:none (the M-RP3.2 tabbed-sampler
  // precedent: "never {#if}, preserves the CDP registry", and the always-mounted dialog posture). The swap
  // is CSS visibility keyed by [data-active]; it is DRIVEN (select a category) and READ off this root's
  // getter G `{section}` + each panel's computed `display` — not asserted (N-091).
  //
  // STOCK dialog Close footer KEPT — deliberately NOT reaching for the `:has()` footer suppression the
  // uistate dialogs use. That suppression is one recurrence already; a second would trigger the
  // footer-snippet-slot extraction (its own milestone, never a rider — J-512 D9). A bottom Close is honest
  // for this read-only leg; the X-in-corner look is Joe's via M-RP-SKIN.
  //
  // ≥2 REAL sections (N-091): About (the real, already-fetched get_about_info content, reused via
  // about-content — S-2; Help ▸ About is left untouched) and Plugins (the existing read-only plugin-list,
  // its mount MOVED here from the absorbed plugins-dialog). The action row is Leg B; the settings
  // mechanism is Leg C.
  import Dialog from '$core/components/data-independent/dialog.svelte';
  import PluginList from '$common/components/widgets/plugin-list.svelte';
  import PluginDetail from './plugin-detail.svelte';
  import AboutContent from './about-content.svelte';
  import Icon from '$core/components/data-independent/icon.svelte';
  import { installed } from '$common/plugins/installed.svelte';
  import { envelope } from '$common/components/base/envelope';

  // M-RP-SETTINGS Leg B — `onPluginAction` is the SHELL-level seam for the row verbs that need shell-local
  // state (disable/uninstall touch the layout + the uistate store; `settings` is greyed this leg). `info` is
  // handled HERE (it swaps THIS modal's content pane, the pane's own concern), never forwarded.
  let { open = $bindable(false), section = null, info = null, onOpenLink, onPluginAction } = $props();

  // The Plugins-section drill-in (§3.5): when a row's [info] fires, the pane swaps to a plugin-detail view
  // for that id; Back returns to the list. `null` = the list is shown. Looked up in `installed.active` (the
  // LISTED set — a disabled custom is still inspectable). If the plugin vanishes (uninstalled while open),
  // the derived is null and the template falls back to the list.
  let detailId = $state(null);
  const detailPlugin = $derived(detailId ? (installed.active.find((p) => p.id === detailId) ?? null) : null);

  // The one seam every plugin-list row button emits through. `info` → open the detail view (local); every
  // other verb → the shell (app_client), which owns the layout + store. A greyed button never fires (guarded
  // onclick in plugin-list), so `settings` reaches here only once a plugin ships one (Leg C).
  function handlePluginAction(pluginId, verb) {
    if (verb === 'info') detailId = pluginId;
    else onPluginAction?.(pluginId, verb);
  }

  // The category list. First = the default (File ▸ Settings lands here). About is real content today;
  // Plugins hosts the read-only manager. More app-level categories arrive as their content earns a home.
  const SECTIONS = [
    { key: 'about', label: 'About' },
    { key: 'plugins', label: 'Plugins' },
  ];
  const DEFAULT_SECTION = SECTIONS[0].key;

  // The active (selected AND roving-focused) section — selection follows focus, the ARIA
  // automatic-activation nav pattern. One state for both concepts (a nav, not a listbox with a separate
  // cursor).
  let active = $state(DEFAULT_SECTION);

  // Deep-link: when the modal opens, land on the requested section (or the default). Keyed on `open` +
  // `section` (NOT `active`, which is only written here) so a user selecting a category while open is not
  // overwritten; re-opening via the gear re-lands on `plugins`.
  $effect(() => {
    if (open) {
      active = section ?? DEFAULT_SECTION;
      detailId = null; // a fresh open always lands on the list, never a stale drill-in
    }
  });

  // Roving-tabindex nav refs, for Arrow/Home/End focus moves (a plain array — the shelf `faces` idiom;
  // used only for .focus(), never read reactively, so no $state proxy over DOM nodes).
  let navButtons = [];

  function selectAt(i) {
    active = SECTIONS[i].key;
    if (SECTIONS[i].key !== 'plugins') detailId = null; // leaving Plugins closes any open drill-in
    navButtons[i]?.focus?.();
  }
  function indexOfActive() {
    const i = SECTIONS.findIndex((s) => s.key === active);
    return i < 0 ? 0 : i;
  }
  function onNavKey(e) {
    const n = SECTIONS.length;
    switch (e.key) {
      case 'ArrowDown':
      case 'ArrowRight':
        e.preventDefault();
        selectAt((indexOfActive() + 1) % n);
        break;
      case 'ArrowUp':
      case 'ArrowLeft':
        e.preventDefault();
        selectAt((indexOfActive() - 1 + n) % n);
        break;
      case 'Home':
        e.preventDefault();
        selectAt(0);
        break;
      case 'End':
        e.preventDefault();
        selectAt(n - 1);
        break;
      default:
        break;
    }
  }

  // Getter G — the section swap made CDP-observable: which section is active + how many there are (so the
  // ≥2-real-sections invariant is readable), plus `open` (the DOM truth rides the core dialog's own G).
  const debug = () => ({ section: active, sectionCount: SECTIONS.length, open, detail: detailId });
</script>

<Dialog bind:open id="settings">
  <div use:envelope={{ name: 'settings', id: 'settings-body', debug }}>
    <!-- Modal header (Joe): Back (round, left — only in the plugin detail drill-in) + the context title +
      the round × close (right, same line). The stock dialog title + footer are suppressed for this modal in
      skin (`:has(.settings)`) so this header owns the chrome. NOTE: this is the 2nd `:has()` footer
      suppression — per J-512 D9 the dialog header/footer-snippet extraction is now owed as its own milestone
      (flagged, not built here). The header buttons compose the core `icon` (shelf-face pattern). -->
    <div class="settings-header">
      {#if detailId && detailPlugin}
        <button
          type="button"
          class="settings-icon-btn"
          aria-label="Back"
          title="Back"
          onclick={() => (detailId = null)}
        ><Icon name="chevron-left" id="settings__back" /></button>
      {/if}
      <span class="settings-header-title"
        >{detailId && detailPlugin ? detailPlugin.name : 'Settings'}</span
      >
      <button
        type="button"
        class="settings-icon-btn settings-close"
        aria-label="Close"
        title="Close"
        onclick={() => (open = false)}
      ><Icon name="close" id="settings__close" /></button>
    </div>

    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <nav class="settings-nav" aria-label="Settings categories" onkeydown={onNavKey}>
      {#each SECTIONS as s, i (s.key)}
        <button
          type="button"
          class="settings-nav-item"
          data-section={s.key}
          aria-current={active === s.key}
          tabindex={active === s.key ? 0 : -1}
          bind:this={navButtons[i]}
          onclick={() => selectAt(i)}
        >{s.label}</button>
      {/each}
    </nav>

    <div class="settings-content">
      <!-- All sections mounted; inactive ones collapse via [data-active] (never {#if} — CDP registry
        stability, M-RP3.2). -->
      <div class="settings-panel" data-active={active === 'about' || undefined}>
        <AboutContent {info} {onOpenLink} idPrefix="settings-about" />
      </div>
      <div class="settings-panel" data-active={active === 'plugins' || undefined}>
        {#if detailId && detailPlugin}
          <PluginDetail plugin={detailPlugin} />
        {:else}
          <PluginList id="plugin-list" onAction={handlePluginAction} />
        {/if}
      </div>
    </div>
  </div>
</Dialog>
