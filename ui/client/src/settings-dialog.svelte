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
  import AboutContent from './about-content.svelte';
  import { envelope } from '$common/components/base/envelope';

  let { open = $bindable(false), section = null, info = null, onOpenLink } = $props();

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
    if (open) active = section ?? DEFAULT_SECTION;
  });

  // Roving-tabindex nav refs, for Arrow/Home/End focus moves (a plain array — the shelf `faces` idiom;
  // used only for .focus(), never read reactively, so no $state proxy over DOM nodes).
  let navButtons = [];

  function selectAt(i) {
    active = SECTIONS[i].key;
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
  const debug = () => ({ section: active, sectionCount: SECTIONS.length, open });
</script>

<Dialog bind:open title="Settings" id="settings">
  <div use:envelope={{ name: 'settings', id: 'settings-body', debug }}>
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
        <PluginList id="plugin-list" />
      </div>
    </div>
  </div>
</Dialog>
