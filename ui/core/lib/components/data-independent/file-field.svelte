<script lang="ts">
  // file-field — di COMPOSITE (M-RP2.25): the 4th di composite (after status-indicator N-054,
  // password-field N-060, star-rating N-061) and the 20th `core` component. **Shape A
  // (child-composite):** composes the real `file` atomic as a HIDDEN child input (`__input`,
  // self-registers under the N-054 model) driven by a styled drop-zone + a file-list display.
  // Contrast to star-rating's Shape B (self-contained). di, binding forwarded (`bind:files`).
  //
  // PASSIVE SLICE ONLY (Rule 6 scope, Joe-lock): drop-zone + file-list. **No remove** (a FileList is
  // immutable; remove needs a `File[]` model + `DataTransfer` write-back — tag-select territory,
  // logged follow-up). **No progress/upload** (host I/O = widget-tier, N-059, deferred). So the
  // composite stays FileList-native + passive: drop/pick REPLACES the selection.
  //
  // Mechanics: the drop-zone `<div>` (role=button, tabindex, Enter/Space → picker) drives the hidden
  // child input via a queried ref (`root.querySelector('input[type=file]')` — the child self-owns its
  // `<input>`, so no atomic change). A drop builds a `DataTransfer` (respecting `multiple` — keeps
  // the first file when single), sets `input.files`, and dispatches `change` so the child's
  // `bind:files` syncs up to this composite's `bind:files`. `data-dragging` (reflected) drives the
  // skin highlight; `disabled` drops all interaction.
  import { envelope } from '$common/components/base/envelope';
  import File from './file.svelte';

  let {
    files = $bindable(null),
    accept,
    multiple = false,
    disabled = false,
    id,
    name,
    label = 'Drop files here or click to browse',
  }: {
    files?: FileList | null;
    accept?: string;
    multiple?: boolean;
    disabled?: boolean;
    id?: string;
    name?: string;
    label?: string;
  } = $props();

  let root = $state<HTMLElement>();
  let dragging = $state(false);
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  const input = () => root?.querySelector('input[type="file"]') as HTMLInputElement | null;

  function openPicker() {
    if (!disabled) input()?.click();
  }
  function onKey(e: KeyboardEvent) {
    if (disabled) return;
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      openPicker();
    }
  }
  function onDragOver(e: DragEvent) {
    if (disabled) return;
    e.preventDefault();
    dragging = true;
  }
  function onDragLeave() {
    dragging = false;
  }
  function onDrop(e: DragEvent) {
    if (disabled) return;
    e.preventDefault();
    dragging = false;
    const dropped = e.dataTransfer?.files;
    const el = input();
    if (!dropped || dropped.length === 0 || !el) return;
    const dt = new DataTransfer();
    const list = multiple ? Array.from(dropped) : [dropped[0]];
    for (const f of list) dt.items.add(f);
    el.files = dt.files;
    el.dispatchEvent(new Event('change', { bubbles: true })); // sync child bind:files → composite
  }

  // N-024 opt-in. Aggregate de-FileList view (mirrors the `file` atomic's serialisable shape).
  const debug = () => ({
    count: files ? files.length : 0,
    files: files ? Array.from(files).map((f) => ({ name: f.name, size: f.size, type: f.type })) : [],
  });
</script>

<div
  bind:this={root}
  use:envelope={{ name: 'file-field', id, debug }}
  data-dragging={dragging || undefined}
  aria-disabled={disabled || undefined}
>
  <div
    class="drop-zone"
    role="button"
    tabindex={disabled ? -1 : 0}
    aria-label={label}
    onclick={openPicker}
    onkeydown={onKey}
    ondragover={onDragOver}
    ondragenter={onDragOver}
    ondragleave={onDragLeave}
    ondrop={onDrop}
  >{label}</div>

  <File bind:files {accept} {multiple} {disabled} {name} id={cid('input')} />

  {#if files && files.length}
    <ul class="file-list">
      {#each Array.from(files) as f}
        <li>{f.name} <span class="size">({f.size} B)</span></li>
      {/each}
    </ul>
  {/if}
</div>
