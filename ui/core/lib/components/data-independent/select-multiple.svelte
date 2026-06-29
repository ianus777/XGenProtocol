<script lang="ts">
  // select-multiple — data-independent, interaction semantic: multi-select / list-box (N-022).
  // Atomic (N-020): the root IS the native <select multiple>; pick-only, multi. Fourteenth real
  // `core` component (M-RP2.19) and the LAST input-family atomic di (N-038).
  //
  // OWN ATOMIC under the sharpened D-096 fold criterion (root + value-type + shared skin/surface):
  // it shares the <select> tag with `select` but fails BOTH the value-type clause (string[] vs the
  // scalar string) AND the skin-surface clause (a static scrolling list-box, not a dropdown that
  // opens). Two of three criteria fail -> own atomic, the same logic that split `range` from
  // `number`/`date`. Applies D-096, no amendment.
  //
  // THE HEADLINE is the binding shape: this is the FIRST plain-ARRAY value-type in the library —
  // `bind:value` -> string[] — the 5th binding shape after boolean-in (`checked`, toggle) /
  // event-out (`onclick`, button) / string-in (`value`, the input family) / number / FileList
  // (`bind:files`, file). The EMPTY MODEL is `[]`, NOT `null` (set-absent is an empty set, not a
  // scalar-null — an array prop is always an array, so consumers `.length`/`.map` with no guard).
  // This deliberately diverges from `select`'s single-select `null` empty; the divergence is the
  // point. Unlike FileList, a plain array IS $state.snapshot-serialisable, so the getter is trivial.
  //
  // `options` carries over UNCHANGED from `select` (N-034): the same dual input shape (`string[]`
  // or `{value,label?,disabled?}[]`) normalized here to one internal shape — the two siblings stay
  // API-symmetric on options. `size` (visible rows of the list-box) is the one genuinely multi-
  // specific knob, default 4. No `placeholder` (a leading empty option is meaningless for a multi
  // list-box). `multiple` is hardcoded — it is the component's identity, not a prop.
  //
  // The type-class is supplied by `envelope` (N-023). No local CSS: a bare <select multiple> is
  // function-complete; all appearance (the list-box surface, selected-row accent tint, focus ring,
  // greyed-disabled) is skin, keyed by `.select-multiple` in the one skin file (N-031 L2).
  import { envelope } from '$common/components/base/envelope';

  type Option = { value: string; label?: string; disabled?: boolean };

  let {
    value = $bindable([]),
    options = [],
    size = 4,
    disabled = false,
    id,
    name,
  }: {
    value?: string[];
    options?: (string | Option)[];
    size?: number;
    disabled?: boolean;
    id?: string;
    name?: string;
  } = $props();

  // Normalize the two accepted shapes to one internal shape (N-034 carryover from `select`).
  const items = $derived(
    options.map((o) =>
      typeof o === 'string'
        ? { value: o, label: o, disabled: false }
        : { value: o.value, label: o.label ?? o.value, disabled: o.disabled ?? false }
    )
  );

  // N-024 opt-in. $state.snapshot de-proxies the array so CDP returnByValue receives plain JSON.
  // Getter shape mirrors `file`'s {count, ...} for sampler-row consistency (D-d).
  const debug = () => ({ values: $state.snapshot(value), count: value.length });
</script>

<select
  multiple
  {size}
  {disabled}
  {name}
  bind:value
  use:envelope={{ name: 'select-multiple', id, debug }}
>
  {#each items as opt (opt.value)}
    <option value={opt.value} disabled={opt.disabled}>{opt.label}</option>
  {/each}
</select>
