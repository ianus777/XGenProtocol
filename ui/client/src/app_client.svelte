<script>
  import { onMount, onDestroy } from 'svelte';
  import MenuBar from '$core/components/data-independent/menu-bar.svelte';
  import StatusBar from '$core/components/data-independent/status-bar.svelte';
  import Paragraph from '$core/components/data-independent/paragraph.svelte';
  import { substitutions } from '$common/components/processor/store.svelte';
  import { KeymapRegistry } from '$common/keymap/registry';
  import { accelerator } from '$common/keymap/accelerator';

  // Initial state before the first Tauri event arrives.
  let currentState = $state({ state: 'INITIALISING', label: 'Initialising' });
  let unlisten;

  // ── Keymap wiring (M-RP6.1d — the 6.1c-deferred shell half) ──────────────────────────────
  // Windows is the only shipped target; the keymap objects take `platform` as a parameter (no
  // `navigator` read), so the shell names it here. mac stays correct-by-construction for a future build.
  const PLATFORM = 'win';

  // The command TABLE — the single source of truth an accelerator OR a File→Exit menu-item both
  // resolve to. `exitCommand` REUSES the exact Tauri close the existing Quit/Shut-Down button wires
  // (invoke('quit')) — no new close call invented (runbook §2.4, Rule 5).
  const commandTable = {
    'app.exit': handleQuit,
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
  ];

  onMount(async () => {
    window.addEventListener('keydown', onKeydown);
    try {
      const { listen } = await import('@tauri-apps/api/event');
      const { invoke } = await import('@tauri-apps/api/core');

      // Subscribe to live state changes.
      unlisten = await listen('xgen-client-state-changed', (event) => {
        currentState = event.payload;
      });

      // Fetch the current state immediately — handles the case where the startup
      // sequence ran before this listener was registered.
      currentState = await invoke('get_state');

      // M-RP4.2 — hydrate the user-owned substitution pairs from the client
      // TOML ([substitutions] rules). The store parses + validates (Tier-2);
      // every processor-host then sources from it.
      substitutions.setRules(await invoke('get_substitutions'));
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

  // ── Connection state → colour (D1) ────────────────────────────────────────────────────────
  // The legacy dotColor() switch, now a literal map handed to status-bar's `states` prop; the
  // legacy isPulsing() as an explicit array. ALL 11 client lifecycle states are enumerated —
  // `led`'s unknown sentinel is BLACK (#000000), so an unenumerated state changes colour
  // visibly rather than falling back silently to the legacy `default: var(--t4)`. The sentinel
  // is the honest signal (Rule 5 / led contract); no fallback branch.
  const STATE_COLOURS = {
    SETUP: 'var(--t4)',            CLOSING: 'var(--t4)',
    INITIALISING: 'var(--t3)',
    CONNECTING: 'var(--inf)',      AUTHENTICATING: 'var(--inf)',     RECONNECTING: 'var(--inf)',
    READY: 'var(--ok)',
    DEGRADED_AUTH: 'var(--pr)',    DEGRADED_FEDERATION: 'var(--pr)', DEGRADED_NODE: 'var(--pr)',
    DISCONNECTED: 'var(--err)',
  };
  const PULSING_STATES = ['INITIALISING', 'CONNECTING', 'AUTHENTICATING', 'RECONNECTING'];

  // ── Resize-grip wiring (D3) ────────────────────────────────────────────────────────────────
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

  <main class="app-center">
    <Paragraph
      text="No layout mounted — the center region shell lands at M-RP6.1f."
      id="center-placeholder"
    />
  </main>

  <!-- The connection light + caption migrate here from the retired hand-rolled .state-indicator
    (D1); the SE grip is a supplementary corner resize affordance (D3). -->
  <StatusBar
    states={STATE_COLOURS}
    state={currentState.state}
    pulse={PULSING_STATES.includes(currentState.state)}
    caption={currentState.label}
    onResizeGrip={handleResizeGrip}
    id="app-statusbar"
  />
</div>
