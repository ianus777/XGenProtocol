<script>
  import { onMount, onDestroy } from 'svelte';
  import Button from './lib/Button.svelte';
  import AppLogo from './assets/logo_node_64.png';

  // Initial state before the first Tauri event arrives.
  let currentState = $state({ state: 'INITIALISING', label: 'Initialising', active_degraded: [] });
  let unlisten;

  onMount(async () => {
    try {
      const { listen } = await import('@tauri-apps/api/event');
      const { invoke } = await import('@tauri-apps/api/core');

      // Subscribe to live state changes.
      unlisten = await listen('xgen-node-state-changed', (event) => {
        currentState = event.payload;
      });

      // Fetch the current state immediately — handles the case where the startup
      // sequence ran before this listener was registered.
      currentState = await invoke('get_node_state');
    } catch (_) {
      // Running outside Tauri (browser dev preview) — state stays at placeholder.
    }
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  async function handleShutDown() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('shut_down');
    } catch (e) {
      console.error('Shut down failed:', e);
    }
  }

  function dotColor(state) {
    switch (state) {
      case 'INITIALISING':         return 'var(--t3)';
      case 'CLOSING':              return 'var(--t3)';
      case 'READY':                return 'var(--ok)';
      case 'DEGRADED_STORAGE':     return 'var(--err)';
      case 'DEGRADED_AUTH':
      case 'DEGRADED_FEDERATION':  return 'var(--pr)';
      case 'MAINTENANCE':          return 'var(--inf)';
      default:                     return 'var(--t4)';
    }
  }

  function isPulsing(state) {
    return state === 'INITIALISING';
  }
</script>

<main id="core-ui-pane">
  <img id="app-logo" src={AppLogo} alt="XGen Node" />

  <div class="state-indicator">
    <span
      class="state-dot"
      class:pulse={isPulsing(currentState.state)}
      style="background-color: {dotColor(currentState.state)}"
    ></span>
    <span class="state-label">{currentState.label}</span>
  </div>

  <Button text="Shut Down" app="node" onAction={handleShutDown} />
</main>
