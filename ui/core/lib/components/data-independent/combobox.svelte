<script lang="ts">
  // combobox — data-independent, COMPOSITE (M-RP2.26): the 5th di composite and the
  // 21st `core` component. Root IS `<div class="combobox">` (N-020/N-022 composite
  // marker via `envelope`). PASSIVE (Path A): a free-text `textfield` bound to a native
  // `<datalist>` via `<input list>` — the ENGINE owns the suggestion popup + the
  // type-to-filter, exactly as `select` leaves its option popup native (N-034). The
  // composite owns NO open/filter/highlight state, so it stays passive di, not `widget`
  // (N-059). This is the specimen N-061 flagged to pressure-test the passive line; it
  // clears it by delegating all popup behaviour to the UA.
  //
  // COMPOSITE-REGISTRATION (N-054): the root registers ONE aggregate getter; the child
  // `textfield` self-registers with a composite-derived id `<id>__field`. A cell yields
  // multiple registry entries. The `<datalist>` is a raw native element (no atomic), the
  // suggestion sink for `list` — mirrors file-field's raw hidden input pattern.
  //
  // `options` reuses the `select` normalized shapes (N-034): `string[]` or
  // `{value,label?,disabled?}[]` -> one internal shape. Rendered as `<option>` inside the
  // datalist (value drives the suggestion; label is the shown hint where the UA honours it).
  //
  // The ▼ affordance is DECORATIVE skin (`.combobox::after`), NOT a button: a native
  // datalist popup is not reliably click-openable and there is no toggle action, so — unlike
  // password-field's reveal button — the glyph carries no control. Single static stroke-only
  // masked ▼ (N-052 mask lineage; the eye/drop precedent renders outline from a fill=none SVG).
  import { envelope } from '$common/components/base/envelope';
  import Textfield from './textfield.svelte';

  type Option = { value: string; label?: string; disabled?: boolean };

  let {
    value = $bindable(''),
    options = [],
    placeholder = '',
    disabled = false,
    id,
    name,
    autocomplete,
  }: {
    value?: string;
    options?: (string | Option)[];
    placeholder?: string;
    disabled?: boolean;
    id?: string;
    name?: string;
    autocomplete?: string;
  } = $props();

  // Composite-derived stable child/datalist ids (so the self-registering child reads
  // cleanly, not ordinal; `list` wires field -> datalist by id).
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);
  const listId = $derived(id ? `${id}__list` : 'combobox-list');

  // Normalize the two accepted shapes to one internal shape (N-034 / N-019).
  const items = $derived(
    options.map((o) =>
      typeof o === 'string'
        ? { value: o, label: o, disabled: false }
        : { value: o.value, label: o.label ?? o.value, disabled: o.disabled ?? false }
    )
  );

  // N-024 opt-in. Aggregate of what the COMPOSITE owns; `count` = suggestion pool size.
  const debug = () => $state.snapshot({ value, count: items.length });
</script>

<div use:envelope={{ name: 'combobox', id, debug }} aria-disabled={disabled || undefined}>
  <Textfield
    bind:value
    {placeholder}
    {disabled}
    {name}
    {autocomplete}
    list={listId}
    id={cid('field')}
  />
  <datalist id={listId}>
    {#each items as opt (opt.value)}
      <option value={opt.value} label={opt.label}></option>
    {/each}
  </datalist>
</div>
