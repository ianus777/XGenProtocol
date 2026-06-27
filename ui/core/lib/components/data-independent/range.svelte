<script lang="ts">
  // range — data-independent, interaction semantic: bounded numeric, slider (N-022).
  // Atomic (N-020): the root IS the native <input type="range">. The SAME root tag AND
  // the SAME value-type (number) as `number` — so by the *literal* D-096 criterion
  // (same root + same VALUE-TYPE) it would fold INTO `number`. It does NOT. D-096's
  // criterion is NECESSARY but NOT SUFFICIENT (range is the case that tests it): the
  // textfield fold was good because the family was genuinely interchangeable (one skin,
  // one prop surface, a thin `type` switch). `range` shares root + value-type with
  // `number` but DIVERGES on:
  //   • skin           — track/thumb pseudo-elements (::-webkit-slider-*), ZERO shared
  //                       appearance with `number`'s text box + spinner;
  //   • prop surface   — no `placeholder` (never empty), no live `:invalid` (the thumb
  //                       is clamped, can't go out of range), no `readonly` (native
  //                       no-op on type=range); bounds are the DEFINING attribute;
  //   • interaction    — clamped drag, ALWAYS valued (never `null`).
  // Folding would put two disjoint skins behind one class and a prop that swaps the
  // whole rendering — the polymorphic-contract problem D-096 exists to prevent, on the
  // APPEARANCE axis. So the fold criterion is SHARPENED: root + value-type + shared
  // skin/surface (genuine interchangeability), not value-type alone (→ D-096 clause,
  // N-042). `range` stays its own atomic.
  //
  // Tenth `core` component (M-RP2.15). Value is ALWAYS present (default 0) — the clean
  // divergence from `number`'s empty=null. The atomic does NOT clamp: if a consumer
  // sets min > 0, they pass an in-range initial (a documented consumer responsibility,
  // exactly as `number` does not clamp).
  //
  // Prop surface = the numeric control, slider-shaped:
  //   keep — value (numeric bind:value, default 0), min / max / step (native shaping
  //          attributes; bounds are the defining attribute, native defaults 0/100/1),
  //          disabled, id, name
  //   drop — `placeholder` (no empty text state), `pattern`, `readonly` (native no-op),
  //          `type` (fixed "range")
  // `maxlength` is deliberately NOT added (orthogonal — mirrors the family).
  //
  // The NATIVE THUMB/TRACK is the atomic's affordance — the custom +/− stepper is the
  // `number` composite track, not this. NO processor seam: `range` is a bounded drag,
  // not free-text/free-number entry, so there are no typed digits to reformat — the
  // numeric-formatting processor consumer is `number`, not `range`.
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // (N-023). No local CSS: a bare range <input> is function-complete; ALL appearance —
  // the track groove, the accent thumb, the focus ring, the disabled grey — is skin,
  // keyed by `.range` in the one skin file (N-025 / N-021 layer 2). Pre-skin it renders
  // as the bare normalize.css/native slider.
  import { envelope } from '$common/components/base/envelope';

  let {
    value = $bindable(0),
    disabled = false,
    min,
    max,
    step,
    id,
    name,
  }: {
    value?: number;
    disabled?: boolean;
    min?: number;
    max?: number;
    step?: number;
    id?: string;
    name?: string;
  } = $props();

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies for CDP's
  // returnByValue. Value-only — min/max/step are static config, not user-mutable state.
  // Always a number (never null).
  const debug = () => $state.snapshot({ value });
</script>

<input
  type="range"
  {disabled}
  {min}
  {max}
  {step}
  {name}
  bind:value
  use:envelope={{ name: 'range', id, debug }}
/>
