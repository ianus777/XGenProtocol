<script lang="ts">
  // entity-panel — the LAST data-dependent composite (M-RP5.2), and the top of the dd-composite
  // tier. It materializes a LIST of address-book entries as a keyboard-navigable, single-select
  // group: a `section` (group chrome) wrapping a `<ul role="listbox">` of `entity-item` rows, and
  // it OWNS the roving focus + selection + empty state — which is why it is its own composite and
  // not a bare `section` with rows dropped in. `spaces-panel` is a consumer PRESET (a title + a
  // spaces `items` array), not a separate component (entity-generic: it lists any entities).
  //
  // COMPOSES (all real children; the matrix multiplies): `section` (di, self-registers `__section`)
  // + `entity-item ×N` (each `entity-item#<panelid>-<itemkey>`, itself composing `entity-avatar`
  // `__avatar` + optional `status` `__status`). Every row inherits the avatar + status corner-slot
  // for free.
  //
  // ROOT (the "wrap, not beside" lock): the outermost DOM is the `section` (from the Section
  // child) — the honest group container. The panel's OWN identity + getter (G) ride the
  // `<ul class="entity-panel" use:envelope>` INSIDE the section body: the panel IS the listbox,
  // Section frames it. (Unlike status-indicator/entity-item, whose root is their own `<div>`.)
  // The `<ul>` always renders so the envelope anchor is stable; it holds the option rows or, when
  // empty, a single presentational message row.
  //
  // FOCUS (C, D): roving tabindex — exactly one row is `tabindex=0` (the active row), the rest
  // `-1`; ArrowUp/Down move, Home/End jump, Enter/Space select+activate; click selects+activates.
  // `selected` ($bindable, an entity id) is the persistent choice (→ each row's `aria-selected`
  // and the row's own `selected` highlight); `activeIndex` is the transient focus position. The
  // list/panel owns this — `entity-item` deliberately deferred roving focus to here (M-RP5.1 D).
  //
  // SOURCE-AGNOSTIC: `items` is an `EntityItemInput[]` view-model (an `EntityDescriptor` + the
  // caller-supplied secondary/status/meta slots); `core` imports NO protocol type.
  import { envelope } from '$common/components/base/envelope';
  import Section from '../data-independent/section.svelte';
  import Paragraph from '../data-independent/paragraph.svelte';
  import EntityItem from './entity-item.svelte';
  import type { EntityDescriptor } from './types';

  type EntityItemInput = {
    descriptor: EntityDescriptor;
    secondary?: string; // topic / handle / last-message — caller string
    status?: { emoji?: string; text?: string; updatedAt?: string; expiresAt?: string };
    meta?: string; // unread / timestamp — caller string
  };

  let {
    items = [],
    title,
    badge,
    collapsible = false,
    collapsed = $bindable(false),
    selected = $bindable(),
    onActivate,
    interactive = true,
    emptyText = 'No entries',
    id,
  }: {
    items?: EntityItemInput[];
    title?: string;
    badge?: string;
    collapsible?: boolean;
    collapsed?: boolean;
    /** The selected entity id ($bindable). Undefined = no selection. */
    selected?: string;
    onActivate?: (id: string) => void;
    /**
     * M-RP-PANEL-INERT — whether the list is a single-select listbox (default) or an inert
     * render-only list. Default `true` = today's behaviour verbatim (R1/R2 + the five sampler sites
     * unchanged). When `false`: `<ul>`/`<li>` drop the `listbox`/`option` ARIA contract for
     * `list`/`listitem`, and the click/keydown/roving/`aria-selected` paths are NOT wired — so the
     * highlight can never be WRITTEN from inside the component. `selected` STILL flows to the row
     * (the row renders state, it never produces it). Used by R7 (members), whose `selected` is a
     * data-driven DM highlight with no feedback loop, so entity-panel's own selectAt would drift it.
     */
    interactive?: boolean;
    emptyText?: string;
    id?: string;
  } = $props();

  const count = $derived(items.length);

  // Composite-derived stable child ids: Section → `<id>__section`, empty message → `<id>__empty`,
  // each row → `<id>-<itemkey>` (itemkey = the descriptor id's last path segment, readable).
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);
  const itemKey = (xgid: string) => xgid.split('/').pop() || xgid;
  const rowId = (xgid: string) => (id ? `${id}-${itemKey(xgid)}` : undefined);

  // Roving-focus position (transient). Init at the selected row, else the first — a deliberate
  // ONE-TIME capture (focus is its own concern once the user navigates), so it lives in mutable
  // `$state`, seeded via a function call (not a direct prop reference in the initializer).
  function initialActive(): number {
    const i = items.findIndex((it) => it.descriptor.id === selected);
    return i >= 0 ? i : 0;
  }
  let activeIndex = $state(initialActive());
  let rowEls: HTMLLIElement[] = [];

  function focusRow(i: number) {
    rowEls[i]?.focus();
  }

  function selectAt(i: number) {
    const it = items[i];
    if (!it) return;
    activeIndex = i;
    selected = it.descriptor.id;
    onActivate?.(it.descriptor.id);
  }

  function onKey(e: KeyboardEvent) {
    const n = items.length;
    if (n === 0) return;
    let next: number | null = null;
    switch (e.key) {
      case 'ArrowDown':
        next = Math.min(n - 1, activeIndex + 1);
        break;
      case 'ArrowUp':
        next = Math.max(0, activeIndex - 1);
        break;
      case 'Home':
        next = 0;
        break;
      case 'End':
        next = n - 1;
        break;
      case 'Enter':
      case ' ':
        e.preventDefault();
        selectAt(activeIndex);
        return;
      default:
        return;
    }
    e.preventDefault();
    activeIndex = next;
    focusRow(next);
  }

  // G — aggregate getter. `collapsed` is the shared bindable (Section toggles it); the panel
  // reports what it owns (count / selection / collapse / empty), children report the rest.
  const debug = () => ({
    count,
    selected: selected ?? null,
    collapsed,
    hasEmpty: items.length === 0,
    interactive,
  });
</script>

<!-- The row body is identical in both modes; only its <li> wrapper differs (interactive listbox
     option vs inert listitem). A snippet keeps the EntityItem markup single-sourced. The wrapper is
     split into two STATIC-role branches rather than one dynamic-role <li> so the a11y analyser can
     prove each: role="option" carries the roving tabindex; role="listitem" carries none (a dynamic
     role would read as "tabindex on a possibly-noninteractive element", M-RP-PANEL-INERT). -->
{#snippet rowBody(item: EntityItemInput)}
  <EntityItem
    descriptor={item.descriptor}
    variant="row"
    secondary={item.secondary}
    status={item.status}
    meta={item.meta}
    selected={item.descriptor.id === selected}
    id={rowId(item.descriptor.id)}
  />
{/snippet}

<Section {title} {badge} {collapsible} bind:collapsed id={cid('section')}>
  <ul
    use:envelope={{ name: 'entity-panel', id, debug }}
    role={interactive ? 'listbox' : 'list'}
    aria-label={title ?? 'entities'}
    aria-multiselectable={interactive ? 'false' : undefined}
  >
    {#if items.length > 0}
      {#each items as item, i (item.descriptor.id)}
        {#if interactive}
          <li
            bind:this={rowEls[i]}
            role="option"
            class="entity-panel-option"
            aria-selected={item.descriptor.id === selected}
            tabindex={i === activeIndex ? 0 : -1}
            onclick={() => selectAt(i)}
            onkeydown={onKey}
          >
            {@render rowBody(item)}
          </li>
        {:else}
          <!-- Inert: a plain listitem. No tabindex, no click/keydown, no aria-selected — the row
               renders `selected` (via the EntityItem) but can never WRITE it. -->
          <li role="listitem" class="entity-panel-listitem">
            {@render rowBody(item)}
          </li>
        {/if}
      {/each}
    {:else}
      <li class="entity-panel-empty" role="presentation">
        <Paragraph text={emptyText} id={cid('empty')} />
      </li>
    {/if}
  </ul>
</Section>
