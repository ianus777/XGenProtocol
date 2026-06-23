<script lang="ts">
  // button — data-independent, interaction semantic: action-trigger (N-022).
  // Atomic (N-020): the root IS the native <button>; the text/icon/link shape
  // variants are pure skin over this same element, never different components.
  // Second real `core` component and the M-RP2.4 pipeline-tuning pass — unlike the
  // toggle (event-in via bind:checked) the button is event-OUT (onclick), so it
  // exercises the other `use:envelope` path. It keeps a small honest `clicks`
  // counter purely so the N-024 dev registry has a LIVE, flippable value to report
  // (click -> re-dump -> delta), the event-out mirror of the toggle's bind proof.
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is
  // hardcoded here (N-023). No local CSS: a bare <button> is function-complete; all
  // appearance — default x/y size, text size, colour, border — is skin, keyed by
  // `.button` in the one skin file, and overridable per-instance via a pass-through
  // class on the same root (N-025 / N-021 layer 2). Pre-skin it renders as the bare
  // normalize.css/native control.
  //
  // M-RP2.6 retrofit (additive, N-030): optional `ariaLabel` (→ aria-label, makes the
  // icon skin-shape accessible) + a `mode` of 'momentary' (default) / 'toggle'. One
  // inner `pressed` bool is the single source of truth: momentary leaves it false and
  // delegates its down-state to CSS :active (M-RP2.7 skin); toggle latches it as a
  // bind-out ($bindable). `aria-pressed` is reflected from `pressed` in toggle-mode only.
  import { envelope } from '$common/components/base/envelope';

  let {
    label = '',
    onclick,
    disabled = false,
    id,
    ariaLabel,
    mode = 'momentary',
    pressed = $bindable(false),
  }: {
    label?: string;
    onclick?: () => void;
    disabled?: boolean;
    id?: string;
    ariaLabel?: string;
    mode?: 'momentary' | 'toggle';
    pressed?: boolean;
  } = $props();

  // Honest internal state: how many times this button has fired. Event-out has no
  // bound value, so this counter is the live observable the harness reads.
  let clicks = $state(0);

  function handleClick() {
    clicks += 1;
    if (mode === 'toggle') pressed = !pressed;
    onclick?.();
  }

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies so CDP's
  // returnByValue receives plain JSON rather than a reactive proxy.
  const debug = () => $state.snapshot({ clicks, disabled, pressed });
</script>

<button
  {disabled}
  aria-label={ariaLabel || undefined}
  aria-pressed={mode === 'toggle' ? pressed : undefined}
  onclick={handleClick}
  use:envelope={{ name: 'button', id, debug }}
>{label}</button>
