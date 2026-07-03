<script lang="ts">
  // combobox — data-independent COMPOSITE (M-RP2.26): 5th di composite, 21st `core`.
  // OWNED-POPUP: renders its own <ul role="listbox"> instead of a native <datalist>, so
  // rows are fully styleable (compact, left-aligned, no balloon, rich content). Native
  // datalist is text-only + unstyleable — the reason this owns its list.
  //
  // PASSIVE di, not a `widget` (N-059): it owns exactly one UI flag, `open` — the same
  // order of state as password-field's `revealed` or file-field's `data-dragging`, none of
  // which are widgets. No behaviour contract, no keyboard subsystem — just a styled dropdown.
  //
  // Composite-registration (N-054): root registers one aggregate getter; the `textfield`
  // child self-registers as `<id>__input` (suffix chosen to NOT collide with password-field's
  // `<id>__field` when both share an instance id). The <ul> is a raw element (no atomic).
  //
  // `options` = normalized rich rows {value,label,status?,disabled?,icon?} (back-compat
  // string[]). `icon?` is declared but UNWIRED until the icon primitive lands (#1 lock).
  //
  // Icon swap: collapsed=chevron (--tri), expanded=closed-triangle (--tri-open), swapped on
  // `.combobox[data-open]`; `open` makes the swap honest (real open/close, unlike native).
  import { envelope } from '$common/components/base/envelope';
  import Textfield from './textfield.svelte';

  type Row = { value: string; label: string; status?: string; disabled?: boolean; icon?: string };

  let {
    value = $bindable(''),
    options = [],
    placeholder = '',
    disabled = false,
    id,
    name,
  }: {
    value?: string;
    options?: (string | Row)[];
    placeholder?: string;
    disabled?: boolean;
    id?: string;
    name?: string;
  } = $props();

  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // Normalize both shapes to one internal row (N-034 / N-019).
  const rows = $derived(
    options.map((o) =>
      typeof o === 'string'
        ? { value: o, label: o, disabled: false }
        : { value: o.value, label: o.label ?? o.value, status: o.status, disabled: o.disabled ?? false, icon: o.icon }
    )
  );

  let open = $state(false);

  // Case-insensitive substring on label; empty query = all (Lock 3).
  const shown = $derived(
    value.trim() === '' ? rows : rows.filter((r) => r.label.toLowerCase().includes(value.toLowerCase()))
  );

  function pick(r: Row) {
    if (r.disabled) return;
    value = r.label;   // v1: field text = label; value/label code-split deferred (like icon)
    open = false;
  }

  // Blur closes next-tick so a row click lands first; pointerdown on the row keeps focus.
  let blurT: ReturnType<typeof setTimeout>;
  function onFocusIn() { if (!disabled) open = true; }
  function onFocusOut() { clearTimeout(blurT); blurT = setTimeout(() => (open = false), 0); }
  function onKey(e: KeyboardEvent) { if (e.key === 'Escape') open = false; }

  const debug = () => $state.snapshot({ value, open, count: rows.length });

  let fieldEl: HTMLDivElement;
  function onChevDown(e: PointerEvent) {
    e.preventDefault();
    if (disabled) return;
    fieldEl?.querySelector('input')?.focus();
  }
</script>

<div
  class="combobox"
  bind:this={fieldEl}
  use:envelope={{ name: 'combobox', id, debug }}
  data-open={open || undefined}
  aria-disabled={disabled || undefined}
  onfocusin={onFocusIn}
  onfocusout={onFocusOut}
  onkeydown={onKey}
>
  <Textfield bind:value {placeholder} {disabled} {name} id={cid('input')} />
  <span class="chev" aria-hidden="true" onpointerdown={onChevDown}></span>

  {#if open && shown.length}
    <ul class="combobox-list" role="listbox">
      {#each shown as r (r.value)}
        <li
          role="option"
          aria-selected={value === r.label}
          aria-disabled={r.disabled || undefined}
          onpointerdown={(e) => e.preventDefault()}
          onclick={() => pick(r)}
        >
          <span class="lbl">{r.label}</span>
          {#if r.status}<span class="status">{r.status}</span>{/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>
