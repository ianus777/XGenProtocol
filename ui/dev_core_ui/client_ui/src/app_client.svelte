<script>
  import { onMount, onDestroy } from 'svelte';
  import Button from './lib/Button.svelte';
  import AppLogo from './assets/logo_client_64.png';

  // Initial state before the first Tauri event arrives.
  let currentState = $state({ state: 'INITIALISING', label: 'Initialising' });
  let unlisten;

  onMount(async () => {
    try {
      const { listen } = await import('@tauri-apps/api/event');
      unlisten = await listen('xgen-client-state-changed', (event) => {
        currentState = event.payload;
      });
    } catch (_) {
      // Running outside Tauri (browser dev preview) — state stays at placeholder.
    }
  });

  onDestroy(() => {
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

  <Button text="Quit" app="client" onAction={handleQuit} />
</main>
