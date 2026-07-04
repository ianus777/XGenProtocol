<script lang="ts">
  // tag-select — data-independent COMPOSITE (M-RP2.28): 6th di composite, 23rd `core`.
  // THE CHIP CONSUMER: renders selected values as `chip` instances via {#each} WITHOUT
  // per-instance registration (N-064 — chips are dynamic/data-driven, not fixed structural
  // children), so the matrix stays predictable (+2/cell: composite + textfield child; chips
  // don't multiply). Candidate rows come from an OWNED popup (the combobox owned-popup pattern).
  //
  // PASSIVE di, not a `widget` (N-059): owns `open` + a transient `query` buffer, no host I/O.
  // Multi-select model = `value: string[]` (option *values*), $bindable, empty []. The query
  // buffer is LOCAL state (bind:value on the child textfield), NOT the model — cleared on pick.
  //
  // Composite-registration (N-054): root registers one aggregate getter; the `textfield` child
  // self-registers as `<id>__filter` (suffix chosen to NOT collide with combobox `__input` or
  // password-field `__field` on a shared instance id). The <ul> + chips are raw (no atomics).
  //
  // `options` = combobox schema {value,label,status?,disabled?} (back-compat string[]). Chip
  // label resolved from options; freeform (allowCreate) value===label. Source-agnostic (N-057):
  // the real client TOML `[tags]` feeds options via a consumer/M-RP4.3; sampler passes a literal.
  //
  // Popup = TWO sections: top "Selected (N)" (all picked rows, reachable even when the control
  // row collapses to +N) + main list (notSelected && matchesQuery — hide-selected). Pick STAYS
  // OPEN, clears query, refocuses. Backspace on empty query pops the last tag. `max?` cap →
  // picks no-op + field dims (data-full). Dedup: case-insensitive, silent.
  import { envelope } from '$common/components/base/envelope';
  import Textfield from './textfield.svelte';
  import Chip from './chip.svelte';

  type Row = { value: string; label: string; status?: string; disabled?: boolean };

  let {
    value = $bindable<string[]>([]),
    options = [],
    placeholder = '',
    disabled = false,
    allowCreate = false,
    max,
    width,
    onManage,
    id,
    name,
  }: {
    value?: string[];
    options?: (string | Row)[];
    placeholder?: string;
    disabled?: boolean;
    allowCreate?: boolean;
    max?: number;
    width?: string;
    onManage?: () => void;
    id?: string;
    name?: string;
  } = $props();

  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // Normalize both option shapes to one internal row (N-034 / N-019).
  const rows = $derived(
    options.map((o) =>
      typeof o === 'string'
        ? { value: o, label: o, disabled: false }
        : { value: o.value, label: o.label ?? o.value, status: o.status, disabled: o.disabled ?? false }
    )
  );

  let query = $state('');
  let open = $state(false);

  const atMax = $derived(max != null && value.length >= max);

  // label lookup: options first, else the raw value (freeform value===label).
  const labelOf = (v: string) => rows.find((r) => r.value === v)?.label ?? v;

  // Selected rows (top section), in selection order.
  const selectedRows = $derived(value.map((v) => ({ value: v, label: labelOf(v) })));

  // Visible-chip fit (Joe's width system). No `width` → field auto-sizes to content, cap at
  // DEFAULT_CAP. `width` set → measure how many chips fit the fixed field (a hidden mirror row
  // gives natural chip widths; ResizeObserver tracks the field); the rest collapse to `+N`.
  const DEFAULT_CAP = 3;
  const RESERVE = 92; // filter-input min + `+N` pill + gaps, reserved from the fixed field width
  let measureEl: HTMLDivElement | undefined = $state();
  let chipW = $state<number[]>([]);
  let availW = $state(0);

  $effect(() => {
    if (!measureEl) return;
    selectedRows.length; // dep: re-measure when the set changes
    chipW = Array.from(measureEl.querySelectorAll('.chip')).map((c) => (c as HTMLElement).offsetWidth);
  });
  $effect(() => {
    if (!width || !fieldEl) return;
    const ro = new ResizeObserver(() => { availW = fieldEl!.clientWidth - RESERVE; });
    ro.observe(fieldEl);
    return () => ro.disconnect();
  });

  // No width → DEFAULT_CAP; width set → count chips that fit availW (pure derived, no oscillation).
  const cap = $derived.by(() => {
    if (!width) return Math.min(selectedRows.length, DEFAULT_CAP);
    let used = 0, fit = 0;
    for (const w of chipW) { used += w + 4; if (used <= availW) fit++; else break; }
    return Math.min(fit, selectedRows.length);
  });
  const visibleChips = $derived(selectedRows.slice(0, cap));
  const overflow = $derived(Math.max(0, selectedRows.length - cap));

  // Main list: not-yet-selected AND matches query (case-insensitive substring; empty = all).
  const shown = $derived(
    rows.filter(
      (r) =>
        !value.includes(r.value) &&
        (query.trim() === '' || r.label.toLowerCase().includes(query.toLowerCase()))
    )
  );

  const has = (v: string) => value.some((x) => x.toLowerCase() === v.toLowerCase());

  function add(v: string) {
    if (disabled || atMax) return;
    if (v.trim() === '' || has(v)) { query = ''; return; }  // silent dedup
    value = [...value, v];
    query = '';
    fieldEl?.querySelector('input')?.focus();
  }
  function pick(r: Row) { if (!r.disabled) add(r.value); }
  function remove(v: string) {
    if (disabled) return;
    value = value.filter((x) => x !== v);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') { open = false; return; }
    if (e.key === 'Backspace' && query === '' && value.length) { e.preventDefault(); value = value.slice(0, -1); return; }
    if (e.key === 'Enter' && allowCreate && query.trim() !== '') {
      e.preventDefault();
      const exact = rows.find((r) => r.label.toLowerCase() === query.toLowerCase());
      add(exact ? exact.value : query.trim());
    }
  }

  let blurT: ReturnType<typeof setTimeout>;
  function onFocusIn() { if (!disabled) open = true; }
  function onFocusOut() { clearTimeout(blurT); blurT = setTimeout(() => (open = false), 0); }

  const debug = () => $state.snapshot({ values: value, count: value.length });

  let fieldEl: HTMLDivElement | undefined = $state();
</script>

<div
  class="tag-select"
  use:envelope={{ name: 'tag-select', id, debug }}
  aria-disabled={disabled || undefined}
>
  <div
    class="tag-field"
    bind:this={fieldEl}
    role="combobox"
    aria-expanded={open}
    data-open={open || undefined}
    data-full={atMax || undefined}
    data-fixed={width || undefined}
    style={width ? `width: ${width}` : undefined}
    onfocusin={onFocusIn}
    onfocusout={onFocusOut}
    onkeydown={onKey}
  >
    <div class="tag-row">
      {#each visibleChips as r (r.value)}
        <Chip label={r.label} removable={!disabled} onRemove={() => remove(r.value)} register={false} />
      {/each}
      {#if overflow > 0}
        <button type="button" class="tag-more" aria-label="{overflow} more selected" onclick={() => (open = true)}>+{overflow}</button>
      {/if}
      <Textfield
        bind:value={query}
        placeholder={value.length === 0 ? placeholder : ''}
        disabled={disabled || atMax}
        {name}
        id={cid('filter')}
      />
    </div>

    <!-- hidden mirror: all chips at natural width, measured to compute the fitted count (width mode) -->
    <div class="tag-measure" aria-hidden="true" bind:this={measureEl}>
      {#each selectedRows as r (r.value)}
        <Chip label={r.label} removable={!disabled} register={false} />
      {/each}
    </div>

    {#if open && !atMax && (selectedRows.length || shown.length)}
      <ul class="tag-select-list" role="listbox" aria-multiselectable="true">
        {#if selectedRows.length}
          <li class="section" aria-hidden="true">Selected ({selectedRows.length})</li>
          {#each selectedRows as r (r.value)}
            <li role="option" aria-selected="true" onpointerdown={(e) => e.preventDefault()} onclick={() => remove(r.value)}>
              <span class="lbl">{r.label}</span>
              <span class="mark">✓</span>
            </li>
          {/each}
        {/if}
        {#if shown.length}
          <li class="section" aria-hidden="true">Options</li>
          {#each shown as r (r.value)}
            <li
              role="option"
              aria-selected="false"
              aria-disabled={r.disabled || undefined}
              onpointerdown={(e) => e.preventDefault()}
              onclick={() => pick(r)}
            >
              <span class="lbl">{r.label}</span>
              {#if r.status}<span class="status">{r.status}</span>{/if}
            </li>
          {/each}
        {/if}
      </ul>
    {/if}
  </div>

  {#if onManage}
    <button
      type="button"
      class="tag-manage"
      aria-label="Manage tags"
      {disabled}
      onclick={() => onManage?.()}
    ></button>
  {/if}
</div>
