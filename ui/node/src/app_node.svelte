<script>
  import { onMount, onDestroy } from 'svelte';
  import Button from '$core/components/data-independent/button.svelte';
  import Toggle from '$core/components/data-independent/toggle.svelte';
  import Textfield from '$core/components/data-independent/textfield.svelte';
  import Select from '$core/components/data-independent/select.svelte';
  import AppLogo from './assets/logo_node_64.png';

  // Initial state before the first Tauri event arrives.
  let currentState = $state({ state: 'INITIALISING', label: 'Initialising', active_degraded: [] });
  let unlisten;

  // M-RP2.3 substrate proof (node side): same throwaway demo instance of the first
  // `core` component as the client. Flip it, re-dump via `cdp-debug.ps1 -App node`
  // (port 9322), observe the {checked:true} delta. Not a real node affordance.
  let demoChecked = $state(false);

  // M-RP2.5 (node side): throwaway demo of the third `core` component (textfield).
  // Type into it, re-dump via `cdp-debug.ps1 -App node` (port 9322), observe the
  // {value} delta — the bind-in live-read proof. Not a real node affordance.
  let demoText = $state('');

  // M-RP2.6 (node side): throwaway demo of the button toggle-mode retrofit. Click it,
  // re-dump via `cdp-debug.ps1 -App node` (port 9322), observe the {pressed} latch
  // delta — the event-driven self-redump the terminal Shut Down button could not give.
  let demoPressed = $state(false);

  // M-RP2.8 (node side): throwaway demo of the fourth `core` component (select). Pick an
  // option, re-dump via `cdp-debug.ps1 -App node` (port 9322), observe the {value} delta.
  // Not a real node affordance.
  let demoSelect = $state('');

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

  <Toggle bind:checked={demoChecked} id="demo" shape="switch" />

  <Textfield bind:value={demoText} id="demo" />

  <Select bind:value={demoSelect} id="demo" placeholder="Pick one" options={['alpha', 'beta', 'gamma']} />

  <Button mode="toggle" bind:pressed={demoPressed} label="toggle" id="demo-toggle" />

  <Button label="Shut Down" onclick={handleShutDown} id="shutdown" />
</main>
