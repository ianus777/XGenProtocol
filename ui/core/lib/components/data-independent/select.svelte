<script lang="ts">
  // select — data-independent, interaction semantic: single-select / any-size (N-022).
  // Atomic (N-020): the root IS the native <select>; pick-only. Fourth real `core`
  // component (M-RP2.8) and the FIRST content-carrying di component — where toggle is
  // event-IN (bind:checked), button event-OUT (onclick) and textfield string bind-IN
  // (bind:value), select is also bind-IN (bind:value, string) but additionally carries
  // *list content* via the `options` prop. It is the first component authored AFTER the
  // skin stack exists (N-033), so it is authored AND skinned in the same pass.
  //
  // `options` accepts the lightweight shapes a consumer naturally has — a `string[]` or
  // a `{value,label?,disabled?}[]` — normalized here to a single internal shape. This
  // keeps the root atomic (no wrapper, N-020) and the component data-INDEPENDENT (the
  // consumer passes a small static set, like a radio group's items); the data-derived
  // layer will later feed the same prop. Optional `placeholder` renders a leading
  // disabled `<option value="">` (shown while value is empty). No `multiple` — that is a
  // separate interaction semantic / shape, not this component.
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // (N-023). No local CSS: a bare <select> is function-complete; all appearance — box,
  // border, radius, focus ring, greyed-disabled, invalid border, and the dropdown arrow
  // (appearance:none + inline-SVG background-image) — is skin, keyed by `.select` in the
  // one skin file (N-025 / N-021 layer 2). The open option-list popup stays native
  // (engine-rendered). Pre-skin it renders as the bare normalize.css/native control.
  import { envelope } from '$common/components/base/envelope';

  type Option = { value: string; label?: string; disabled?: boolean };

  let {
    value = $bindable(''),
    options = [],
    placeholder,
    disabled = false,
    id,
    name,
    required = false,
  }: {
    value?: string;
    options?: (string | Option)[];
    placeholder?: string;
    disabled?: boolean;
    id?: string;
    name?: string;
    required?: boolean;
  } = $props();

  // Normalize the two accepted shapes to one internal shape (N-019: the consumer's
  // convenience input collapses to a single render path).
  const items = $derived(
    options.map((o) =>
      typeof o === 'string'
        ? { value: o, label: o, disabled: false }
        : { value: o.value, label: o.label ?? o.value, disabled: o.disabled ?? false }
    )
  );

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies so CDP's
  // returnByValue receives plain JSON rather than a reactive proxy.
  const debug = () => $state.snapshot({ value });
</script>

<select
  {disabled}
  {name}
  {required}
  bind:value
  use:envelope={{ name: 'select', id, debug }}
>
  {#if placeholder !== undefined}
    <option value="" disabled>{placeholder}</option>
  {/if}
  {#each items as opt (opt.value)}
    <option value={opt.value} disabled={opt.disabled}>{opt.label}</option>
  {/each}
</select>
