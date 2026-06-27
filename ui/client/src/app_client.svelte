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
  import Range from '$core/components/data-independent/range.svelte';
  import AppLogo from './assets/logo_client_64.png';
  import Placeholder from '$assets/img-placeholder.svg';

  // Initial state before the first Tauri event arrives.
  let currentState = $state({ state: 'INITIALISING', label: 'Initialising' });
  let unlisten;

  // M-RP2.3 substrate proof: a throwaway demo instance of the first `core` component
  // (toggle). Flip it in the running app, re-dump via cdp-debug.ps1, observe the
  // {checked:true} delta. Not a real client affordance — removed once the proof lands.
  let demoChecked = $state(false);

  // M-RP2.5: throwaway demo of the third `core` component (textfield). Type into it,
  // re-dump via cdp-debug.ps1, observe the {value} delta — the bind-in live-read proof
  // the terminal button could not give. Not a real client affordance.
  let demoText = $state('');

  // M-RP2.6: throwaway demo of the button toggle-mode retrofit. Click it, re-dump via
  // cdp-debug.ps1, observe the {pressed} latch delta — the event-driven self-redump the
  // terminal Quit button could not give. Not a real client affordance.
  let demoPressed = $state(false);

  // M-RP2.8: throwaway demo of the fourth `core` component (select). Pick an option,
  // re-dump via cdp-debug.ps1, observe the {value} delta. Not a real client affordance.
  let demoSelect = $state('');

  // M-RP2.12: throwaway demo of the textfield `type` fold (search variant). Proves the
  // `type` prop reaches the <input> + the string bind:value still works on a non-text
  // type; the inset magnifier icon renders. Not a real client affordance.
  let demoSearch = $state('');

  // M-RP2.13: throwaway demo of the eighth `core` component (textarea). Type multi-line
  // text, re-dump via cdp-debug.ps1, observe the {value} delta carry a literal newline.
  // Not a real client affordance.
  let demoTextarea = $state('');

  // M-RP2.14: throwaway demo of the ninth `core` component (number). Spin/type a value,
  // re-dump via cdp-debug.ps1, observe the {value} delta carry a JSON number (not a
  // string). Imported as NumberField to avoid shadowing the global `Number`.
  let demoNumber = $state(null);

  // M-RP2.15: throwaway demo of the tenth `core` component (range). Drag the slider,
  // re-dump via cdp-debug.ps1, observe the {value} delta carry a JSON number. A range is
  // always valued (default 0; demo seeds 50), no null state. Not a real client affordance.
  let demoRange = $state(50);

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

  <Toggle bind:checked={demoChecked} id="demo" shape="switch" />

  <Textfield bind:value={demoText} id="demo" />

  <Textfield type="search" bind:value={demoSearch} id="demo-search" placeholder="Search" />

  <Select bind:value={demoSelect} id="demo" placeholder="Pick one" options={['alpha', 'beta', 'gamma']} />

  <Textarea bind:value={demoTextarea} id="demo" placeholder="Multi-line text" />

  <NumberField bind:value={demoNumber} id="demo" placeholder="0" min={0} max={100} step={1} />

  <Range bind:value={demoRange} id="demo" min={0} max={100} step={1} />

  <!-- M-RP2.9: throwaway demo of the first display-di (label, read-only). Snapshot via
       cdp-debug.ps1 -> {text}; no bind (read-only, no user delta). Not a real affordance. -->
  <Label text="Demo label" id="demo" />

  <!-- M-RP2.10: throwaway demo of the second display-di (paragraph, read-only prose). -->
  <Paragraph text="Demo paragraph of prose." id="demo" />

  <!-- M-RP2.11: throwaway demo of the third display-di (image, read-only). Bundled
       neutral placeholder asset; src lands as a resolved URL, alt is required. -->
  <Image src={Placeholder} alt="Image placeholder" id="demo" />

  <Button mode="toggle" bind:pressed={demoPressed} label="toggle" id="demo-toggle" />

  <Button label="Quit" onclick={handleQuit} id="quit" />
</main>
