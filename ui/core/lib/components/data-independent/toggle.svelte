<script lang="ts">
  // toggle — data-independent, interaction semantic: boolean-toggle (N-022).
  // Atomic (N-020): the root IS the native <input type="checkbox">; the on/off-switch
  // shape is pure skin over this same element, never a different component.
  // First real `core` component and the M-RP2.3 substrate proof — its reactive
  // `checked` is exposed to the dev registry through the N-024 envelope debug-branch,
  // so the CDP harness reads a live, flippable value (flip -> re-dump -> delta).
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // here (N-023). No local CSS: a bare checkbox is function-complete; all appearance is
  // skin, keyed by `.toggle` in the one skin file (N-025).
  import { envelope } from '$common/components/base/envelope';

  let {
    checked = $bindable(false),
    id,
  }: { checked?: boolean; id?: string } = $props();

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies the value so CDP's
  // returnByValue receives plain JSON rather than a reactive proxy.
  const debug = () => $state.snapshot({ checked });
</script>

<input
  type="checkbox"
  bind:checked
  use:envelope={{ name: 'toggle', id, debug }}
/>
