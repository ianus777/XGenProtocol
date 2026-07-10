<script>
  import { onMount, onDestroy } from 'svelte';
  import Button from '$core/components/data-independent/button.svelte';
  import MenuBar from '$core/components/data-independent/menu-bar.svelte';
  import { substitutions } from '$common/components/processor/store.svelte';
  import { KeymapRegistry } from '$common/keymap/registry';
  import { accelerator } from '$common/keymap/accelerator';
  import AppLogo from './assets/logo_client_64.png';

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

  function dotColor(state) {
    switch (state) {
      case 'SETUP':
      case 'CLOSING':       return 'var(--t4)';
      case 'INITIALISING':  return 'var(--t3)';
      case 'CONNECTING':
      case 'AUTHENTICATING':
      case 'RECONNECTING':  return 'var(--inf)';
      case 'READY':         return 'var(--ok)';
      case 'DEGRADED_AUTH':
      case 'DEGRADED_FEDERATION':
      case 'DEGRADED_NODE': return 'var(--pr)';
      case 'DISCONNECTED':  return 'var(--err)';
      default:              return 'var(--t4)';
    }
  }

  function isPulsing(state) {
    return ['INITIALISING', 'CONNECTING', 'AUTHENTICATING', 'RECONNECTING'].includes(state);
  }
</script>

<!-- Fixed frame: the top-pane menu-bar (frame chrome, OUTSIDE the Layout descriptor — region-dock
  model §2) over the centered content body. 6.1e adds the bottom status-bar, 6.1f the center region. -->
<div class="app-frame">
  <MenuBar {menus} platform={PLATFORM} onCommand={runCommand} id="app-menubar" />

  <div class="app-body">
    <main id="core-ui-pane">
      <img id="app-logo" src={AppLogo} alt="XGen Client" />

      <div class="state-indicator">
        <span
          class="state-dot"
          class:pulse={isPulsing(currentState.state)}
          style="background-color: {dotColor(currentState.state)}"
        ></span>
        <span class="state-label">{currentState.label}</span>
      </div>

      <!-- Additive: the existing Quit button stays intact (D-065). File→Exit is the new civilized
        exit; ripping the working button is scope creep — a later cleanup removes the redundant one. -->
      <Button label="Quit" onclick={handleQuit} id="quit" />
    </main>
  </div>
</div>
