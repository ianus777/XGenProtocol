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
  //
  // M-RP2.6 (additive, N-030): a semantic `shape` of 'checkbox' (default) / 'switch'.
  // switch-shape reflects role="switch" + aria-checked from the same `checked` bool now;
  // its visual switch look is M-RP2.7 skin keyed on [role="switch"]. shape is a static
  // prop (not state), so the debug getter is unchanged.
  import { envelope } from '$common/components/base/envelope';

  let {
    checked = $bindable(false),
    id,
    shape = 'checkbox',
  }: { checked?: boolean; id?: string; shape?: 'checkbox' | 'switch' } = $props();

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies the value so CDP's
  // returnByValue receives plain JSON rather than a reactive proxy.
  const debug = () => $state.snapshot({ checked });
</script>

<input
  type="checkbox"
  bind:checked
  role={shape === 'switch' ? 'switch' : undefined}
  aria-checked={shape === 'switch' ? checked : undefined}
  use:envelope={{ name: 'toggle', id, debug }}
/>
