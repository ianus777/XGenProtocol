<script>
  import { onMount, onDestroy } from 'svelte';
  import Button from '$core/components/data-independent/button.svelte';
  import Toggle from '$core/components/data-independent/toggle.svelte';
  import Textfield from '$core/components/data-independent/textfield.svelte';
  import Select from '$core/components/data-independent/select.svelte';
  import Label from '$core/components/data-independent/label.svelte';
  import Paragraph from '$core/components/data-independent/paragraph.svelte';
  import Image from '$core/components/data-independent/image.svelte';
  import Textarea from '$core/components/data-independent/textarea.svelte';
  import NumberField from '$core/components/data-independent/number.svelte';
  import AppLogo from './assets/logo_node_64.png';
  import Placeholder from '$assets/img-placeholder.svg';

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

  // M-RP2.12 (node side): throwaway demo of the textfield `type` fold (search variant).
  // Proves the `type` prop reaches the <input> + the string bind:value still works on a
  // non-text type; the inset magnifier icon renders. Not a real node affordance.
  let demoSearch = $state('');

  // M-RP2.13 (node side): throwaway demo of the eighth `core` component (textarea). Type
  // multi-line text, re-dump via `cdp-debug.ps1 -App node` (port 9322), observe the
  // {value} delta carry a literal newline. Not a real node affordance.
  let demoTextarea = $state('');

  // M-RP2.14 (node side): throwaway demo of the ninth `core` component (number). Spin/type
  // a value, re-dump via `cdp-debug.ps1 -App node` (port 9322), observe the {value} delta
  // carry a JSON number (not a string). Imported as NumberField to avoid shadowing the
  // global `Number`. Not a real node affordance.
  let demoNumber = $state(null);

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

  <Textfield type="search" bind:value={demoSearch} id="demo-search" placeholder="Search" />

  <Select bind:value={demoSelect} id="demo" placeholder="Pick one" options={['alpha', 'beta', 'gamma']} />

  <Textarea bind:value={demoTextarea} id="demo" placeholder="Multi-line text" />

  <NumberField bind:value={demoNumber} id="demo" placeholder="0" min={0} max={100} step={1} />

  <!-- M-RP2.9 (node side): throwaway demo of the first display-di (label, read-only).
       Snapshot via cdp-debug.ps1 -App node -> {text}; no bind. Not a real affordance. -->
  <Label text="Demo label" id="demo" />

  <!-- M-RP2.10 (node side): throwaway demo of the second display-di (paragraph, read-only prose). -->
  <Paragraph text="Demo paragraph of prose." id="demo" />

  <!-- M-RP2.11 (node side): throwaway demo of the third display-di (image, read-only).
       Bundled neutral placeholder asset; src lands as a resolved URL, alt is required. -->
  <Image src={Placeholder} alt="Image placeholder" id="demo" />

  <Button mode="toggle" bind:pressed={demoPressed} label="toggle" id="demo-toggle" />

  <Button label="Shut Down" onclick={handleShutDown} id="shutdown" />
</main>
