<script lang="ts">
  // date — data-independent, interaction semantic: date/time entry, native picker (N-022).
  // Atomic (N-020): the root IS the native <input> for the date-input family. FOLDS the five
  // date-input siblings into ONE component via a constrained `type` prop — the `textfield`
  // fold again, NOT the `range` case:
  //   type: 'date' | 'time' | 'datetime-local' | 'month' | 'week'   (default 'date')
  // All share the <input> root, a STRING bind:value, the `.date` skin, and the prop surface;
  // they differ ONLY in UA-supplied picker chrome (calendar / clock / both) — exactly the
  // textfield situation (the string-input family differed only in UA validation/keyboard/
  // masking). Passes the SHARPENED D-096 criterion (root + value-type + shared skin/surface,
  // the N-042 amendment). Contrast: `number` folds-fail on value-type (numeric), `range`
  // folds-fail on disjoint skin (slider pseudos). `date` fails neither.
  //
  // The value is a STRING for every type — Svelte's plain `bind:value` binds the element's
  // `.value`, which is the type's structured string:
  //   date           "2026-06-28"
  //   time           "13:45"
  //   datetime-local "2026-06-28T13:45"
  //   month          "2026-06"
  //   week           "2026-W26"
  // Empty = '' (always-string, never `null` — the clean divergence from `number`'s empty=null).
  // The per-type FORMAT differs, so the getter carries `type` ({ type, value }, the textfield
  // precedent): `type` travels with the value so a consumer can interpret the format through
  // the N-024 registry.
  //
  // Value semantics: plain `bind:value` (string), NOT bind:valueAsDate (`Date | null` is
  // serialization-hostile; the string form is wire-clean and matches the family). valueAsDate
  // is a reserved future shape, not built.
  //
  // Eleventh `core` component (M-RP2.16). Prop surface = the control vocabulary, date-shaped:
  //   keep — value (string bind:value, default ''), disabled, readonly, id, name
  //   add  — min / max (native date/time-string shaping attrs), step (native increment:
  //          days / seconds / months per type) — config, not state; type-appropriate values
  //          are the consumer's job (the `number` min/max/step precedent)
  //   drop — placeholder (native date inputs ignore it — the format hint shows instead),
  //          pattern (no native `pattern` on these types). `type` is the fold prop, not fixed.
  // `maxlength` deliberately NOT added (orthogonal — mirrors the family).
  //
  // Enforcement is the TS union ALONE (no runtime guard, no DEV-warn): an out-of-whitelist
  // value degrades safely (the browser normalizes an unknown `type` to `text`), so a guard
  // would be empty machinery (D-065) — the D-096 / N-039 precedent.
  //
  // The NATIVE PICKER (the calendar-picker indicator + popup) is the atomic's affordance — a
  // custom date-picker dropdown is a later COMPOSITE, not this. NO processor seam: a structured
  // native value, not free-text/free-number entry (the numeric-formatting consumer is `number`).
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // (N-023). No local CSS: a bare date <input> is function-complete; ALL appearance — the box,
  // greyed-disabled, focus ring, :invalid (native min/max range validation), the recoloured
  // calendar-picker indicator — is skin, keyed by `.date` in the one skin file (N-025 / N-021
  // layer 2). Picker chrome renders dark via the global `color-scheme: dark` on :root (N-043,
  // added FOR this family). Pre-skin it renders as the bare normalize.css/native control.
  import { envelope } from '$common/components/base/envelope';

  let {
    type = 'date',
    value = $bindable(''),
    disabled = false,
    readonly = false,
    min,
    max,
    step,
    id,
    name,
  }: {
    type?: 'date' | 'time' | 'datetime-local' | 'month' | 'week';
    value?: string;
    disabled?: boolean;
    readonly?: boolean;
    min?: string;
    max?: string;
    step?: number;
    id?: string;
    name?: string;
  } = $props();

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies for CDP's returnByValue.
  // Carries `type` so the configured type (hence the value's format) is registry-verifiable.
  const debug = () => $state.snapshot({ type, value });
</script>

<input
  {type}
  {disabled}
  {readonly}
  {min}
  {max}
  {step}
  {name}
  bind:value
  use:envelope={{ name: 'date', id, debug }}
/>
