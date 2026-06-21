<script>
  import { onMount, onDestroy } from 'svelte';
  import Button from './lib/Button.svelte';
  import Toggle from '$core/components/data-independent/toggle.svelte';
  import AppLogo from './assets/logo_client_64.png';

  // Initial state before the first Tauri event arrives.
  let currentState = $state({ state: 'INITIALISING', label: 'Initialising' });
  let unlisten;

  // M-RP2.3 substrate proof: a throwaway demo instance of the first `core` component
  // (toggle). Flip it in the running app, re-dump via cdp-debug.ps1, observe the
  // {checked:true} delta. Not a real client affordance — removed once the proof lands.
  let demoChecked = $state(false);

  onMount(async () => {
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

  <Toggle bind:checked={demoChecked} id="demo" />

  <Button text="Quit" app="client" onAction={handleQuit} />
</main>
