<script>
  import { onMount, onDestroy } from 'svelte';
  import MenuBar from '$core/components/data-independent/menu-bar.svelte';
  import StatusBar from '$core/components/data-independent/status-bar.svelte';
  import RegionShell from '$core/components/layout/region-shell.svelte';
  import AboutDialog from './about-dialog.svelte';
  import { loadLayout, widgetRegistry } from './layout-default';
  import { substitutions } from '$common/components/processor/store.svelte';
  // The self-state store (M-RP6.1g, D3) — ONE channel, TWO views: the shell WRITES it below (the existing
  // state listen + get_state + a once get_self_state invoke), and BOTH the status-bar (here) AND the
  // self-panel widget READ it. STATE_COLOURS/PULSING_STATES relocated here (they were shell-local; the
  // widget needs them and cannot receive shell props — W-3).
  import { selfState, STATE_COLOURS, PULSING_STATES } from '$common/stores/self-state.svelte';
  // The selection bus (M-RP6.1f, D3): its first real writer is now the self-panel widget (D5). This
  // side-effect import keeps the DEV __XGEN_SEL__ handle installed for CDP; the shell is the bus's host,
  // its readers (R8 inspector / entity-context-menu) wire in at 6.1h+.
  import '$common/stores/selection.svelte';
  import { KeymapRegistry } from '$common/keymap/registry';
  import { accelerator } from '$common/keymap/accelerator';

  // Connection state now lives in the self-state store (D3 — one source, two views); no shell-local
  // mirror. The store seeds itself to INITIALISING until the first Tauri event arrives.
  let unlisten;

  // About dialog (M-RP6.1e-C3): the get_about_info payload (build metadata + paths), fetched once
  // on mount (static per session), and the open state flipped by Help→About.
  let aboutInfo = $state(null);
  let aboutOpen = $state(false);

  // Centre region layout (M-RP6.1f). Seeded by `await loadLayout()` on mount (D2 — async so M-RP7.3 is a
  // one-line swap to invoke('get_layout')). Shell-local this milestone (D7 — the shell is the only
  // consumer; the widget-manager/shelf promotion to a $common store is reserved, not built).
  let layout = $state(null);

  // DEV-only CDP handle (N-024 idiom) so the verify pass can drive the drop / tabs / mismatch paths
  // (§5.3): push a test layout, read region-shell's getter G. Dead-code-eliminated in a release build.
  if (import.meta.env.DEV && typeof window !== 'undefined') {
    window.__XGEN_LAYOUT__ = {
      get current() { return layout; },
      set(l) { layout = l; },
    };
  }

  // ── Keymap wiring (M-RP6.1d — the 6.1c-deferred shell half) ──────────────────────────────
  // Windows is the only shipped target; the keymap objects take `platform` as a parameter (no
  // `navigator` read), so the shell names it here. mac stays correct-by-construction for a future build.
  const PLATFORM = 'win';

  // The command TABLE — the single source of truth an accelerator OR a File→Exit menu-item both
  // resolve to. `exitCommand` REUSES the exact Tauri close the existing Quit/Shut-Down button wires
  // (invoke('quit')) — no new close call invented (runbook §2.4, Rule 5).
  const commandTable = {
    'app.exit': handleQuit,
    'help.about': () => (aboutOpen = true),
  };
  function runCommand(commandId) {
    commandTable[commandId]?.();
  }

  // The keymap registry singleton: Ctrl+Q → app.exit. One global keydown → resolve → run.
  const registry = new KeymapRegistry();
  registry.register(accelerator('Ctrl+Q'), 'app.exit');

  function onKeydown(e) {
    const commandId = registry.resolve(e, PLATFORM);
    if (commandId) {
      e.preventDefault();
      runCommand(commandId);
    }
  }

  // The top-pane menu-bar: File → Exit carries the SAME "app.exit" command AND the Ctrl+Q
  // accelerator (which renders the trailing hint). One Accelerator, one command — Ctrl+Q and
  // File→Exit are one truth.
  const menus = [
    {
      label: 'File',
      items: [{ label: 'Exit', accelerator: accelerator('Ctrl+Q'), command: 'app.exit' }],
    },
    // Help → About (M-RP6.1e-C3). NO accelerator — F1 conventionally means Help *contents*.
    {
      label: 'Help',
      items: [{ label: 'About', command: 'help.about' }],
    },
  ];

  onMount(async () => {
    window.addEventListener('keydown', onKeydown);

    // Seed the centre layout (D2). Not Tauri, never throws, so it runs OUTSIDE the try that swallows the
    // no-Tauri (browser dev preview) case — the grid must render even without a backend.
    layout = await loadLayout();

    try {
      const { listen } = await import('@tauri-apps/api/event');
      const { invoke } = await import('@tauri-apps/api/core');

      // Subscribe to live state changes — write the SAME store the status-bar and the self-panel read
      // (D3). No new emit channel: this is the existing `xgen-client-state-changed` (D1).
      unlisten = await listen('xgen-client-state-changed', (event) => {
        selfState.setConnection(event.payload);
      });

      // Fetch the current state immediately — handles the case where the startup
      // sequence ran before this listener was registered (pre-listener race, already solved).
      selfState.setConnection(await invoke('get_state'));

      // M-RP6.1g — self-identity (D1/D2). Static per session (no in-app registration exists), so fetched
      // ONCE here (the get_about_info shape). Inside the same try so the browser-dev/no-Tauri path keeps
      // working. Unregistered → registered:false + a real keypair-derived XGID (rendered honestly, D6).
      selfState.setIdentity(await invoke('get_self_state'));

      // M-RP4.2 — hydrate the user-owned substitution pairs from the client
      // TOML ([substitutions] rules). The store parses + validates (Tier-2);
      // every processor-host then sources from it.
      substitutions.setRules(await invoke('get_substitutions'));

      // M-RP6.1e-C3 — About data (build metadata + Rust/Tauri/Svelte versions + paths). Static
      // per session, so fetched once here; the dialog reads it synchronously.
      aboutInfo = await invoke('get_about_info');
    } catch (_) {
      // Running outside Tauri (browser dev preview) — state stays at placeholder.
    }
  });

  onDestroy(() => {
    window.removeEventListener('keydown', onKeydown);
    if (unlisten) unlisten();
  });

  async function handleQuit() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('quit');
    } catch (e) {
      console.error('Quit failed:', e);
    }
  }

  // M-RP6.1e-C3 (F1) — open the About website in the OS browser. The `link` atomic stays dumb;
  // the consumer wires this. preventDefault stops the <a target=_blank> in-app-webview path;
  // the lazy import keeps the browser-dev preview (no Tauri) working — the handleQuit pattern.
  async function handleAboutLink(e) {
    e?.preventDefault?.();
    try {
      const { openUrl } = await import('@tauri-apps/plugin-opener');
      await openUrl(aboutInfo?.common?.link ?? 'https://www.alchemydump.com');
    } catch (err) {
      console.error('Open link failed:', err);
    }
  }

  // STATE_COLOURS / PULSING_STATES relocated to `$common/stores/self-state.svelte` (D3) — the widget
  // needs them and cannot receive shell props, so the shell now IMPORTS the same map it reads for the
  // status-bar (one map, two views). The relocation is a pure move (V7 — colour byte-identical).

  // ── Resize-grip wiring ─────────────────────────────────────────────────────────────────────
  // The status-bar's SE grip exposes `onResizeGrip?` and drives Tauri's startResizeDragging
  // (SE = width+height). The window is now OS-decorated (native title bar + native edge resize),
  // so the grip is a supplementary corner affordance — a conventional explicit resize handle at
  // the bottom-right — not the sole resize path it was under the original frameless design.
  // Lazy-imported inside the handler — the exact handleQuit pattern, so the browser-dev preview
  // (no Tauri) keeps working. 'SouthEast' is a valid ResizeDirection string literal (confirmed
  // against @tauri-apps/api window.d.ts — no enum import needed).
  async function handleResizeGrip() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().startResizeDragging('SouthEast');
    } catch (e) {
      console.error('Resize drag failed:', e);
    }
  }
</script>

<!-- Fixed BorderPane frame (M-RP6.1e-B): the full-width menu-bar and the bottom status-bar are
  frame chrome, OUTSIDE the future Layout descriptor (region-dock model §2 — File→Exit can never
  be docked away), below the OS-native title bar. The center is the ONLY scroller (D5) and holds
  a placeholder until 6.1f. Window move + min/max/close come from the native title bar (Joe's
  pivot away from the original frameless design), so there is no in-app drag region. -->
<div class="app-frame">
  <MenuBar {menus} platform={PLATFORM} onCommand={runCommand} id="app-menubar" />

  <!-- The centre region shell (M-RP6.1f): renderer A reads the Layout descriptor and tiles placeholder
    leaves. It FILLS .app-center (no whole-grid scroll, D5) — each leaf owns its own scroll. -->
  <main class="app-center">
    {#if layout}
      <RegionShell {layout} widgets={widgetRegistry} id="region-root" />
    {/if}
  </main>

  <!-- The connection light + caption migrate here from the retired hand-rolled .state-indicator
    (D1); the SE grip is a supplementary corner resize affordance (D3). -->
  <StatusBar
    states={STATE_COLOURS}
    state={selfState.connection.state}
    pulse={PULSING_STATES.includes(selfState.connection.state)}
    caption={selfState.connection.label}
    onResizeGrip={handleResizeGrip}
    id="app-statusbar"
  />

  <!-- The About modal (M-RP6.1e-C3). A top-layer <dialog> — DOM position doesn't affect stacking;
    opened by Help→About (help.about → aboutOpen). Always mounted (closed = display:none). -->
  <AboutDialog bind:open={aboutOpen} info={aboutInfo} onOpenLink={handleAboutLink} />
</div>
