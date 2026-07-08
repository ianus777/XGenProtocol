<script lang="ts">
  // message-stream — a data-dependent COMPOSITE (M-RP5.6, the `entity-panel` analogue for the
  // message dd sub-family; `docs/xgen-dd-message-family-phase0.md` v1.1 §9). It owns the
  // RELATIONSHIPS between messages — chronological ordering (consumer-supplied), grouping
  // computation, day-dividers, empty state, and a persistent background layer — while each child
  // `message` owns HOW ONE MESSAGE LOOKS. This is STEP A (the shell): the root IS the scroll
  // viewport (`overflow-y:auto`) so B (the scroll machine: stick-to-bottom / jump-pill /
  // preserve-on-prepend) has a home, but A builds NO scroll behaviour.
  //
  // ROOT — `<div class="message-stream" role="log" use:envelope>` (§9): a scrolling live region, NOT
  // a `listbox` (a chat is a log, not a select-one-of list) → click-select, no roving tabindex.
  // Children = `message`s + interleaved `<div class="day-divider" role="separator">` rows, in the
  // GIVEN order (the stream does not re-sort; the consumer supplies chronological order).
  //
  // GROUPING + DIVIDERS are pure (`stream/grouping.ts`, colocated + unit-testable): `computeRows`
  // walks the ordered messages once, producing the interleaved row list and each message's
  // stream-computed `grouped` flag (passed DOWN as a PROP — never a descriptor field, Phase-0 §4/§5).
  //
  // WIDGET SOCKETS (W-12/W-13). `background?: WidgetMount[]` is a persistent fixed layer behind the
  // log (chat-wallpaper); `backgroundLive?` is the settings switch passed into each mount (a reactive
  // widget renders frozen when false; a static object ignores it) — the store BINDING is M-RP6.x, A
  // just exposes the prop. Both background mounts and each child `message`'s `details` socket resolve
  // `widgetId` → component via the consumer `widgets` registry (the `message` precedent), DROPPING an
  // unknown id on render (W-13). `core` imports no concrete widget.
  import { envelope } from '$common/components/base/envelope';
  import type { Component } from 'svelte';
  import Message from './message.svelte';
  import Paragraph from '../data-independent/paragraph.svelte';
  import type { MessageDescriptor, WidgetMount } from './types';
  import { computeRows } from './stream/grouping';

  let {
    messages = [],
    background,
    backgroundLive = true,
    widgets = {},
    selected = $bindable(),
    onSelect,
    id,
  }: {
    messages?: MessageDescriptor[]; // ordered (chronological); the stream does NOT re-sort
    background?: WidgetMount[]; // persistent fixed layer (chat-wallpaper); undefined = none
    backgroundLive?: boolean; // settings switch, default true; binding deferred to M-RP6.x
    widgets?: Record<string, Component>; // widgetId → component (background + message details); W-13
    selected?: string; // $bindable clicked-message id
    onSelect?: (id: string) => void; // reserved selection-bus hook (wiring M-RP6.x)
    id?: string;
  } = $props();

  // Single `now` captured at mount → deterministic divider labels for this render (A is static
  // fixtures; live re-labelling as wall-clock advances is not a Step-A concern).
  const now = new Date();

  const rows = $derived(computeRows(messages, now));
  const count = $derived(messages.length);
  const groupedCount = $derived(rows.filter((r) => r.kind === 'message' && r.grouped).length);
  const dividerCount = $derived(rows.filter((r) => r.kind === 'divider').length);

  // Background mounts — resolve each declared widgetId against the consumer registry; DROP unknown
  // ids (W-13, the `message.details` mechanism). The resolved list is the render truth, so
  // `backgroundMountCount` reports what is actually shown (a dropped unknown lowers it).
  const resolvedBg = $derived(
    (background ?? [])
      .map((m, i) => ({ key: `${m.widgetId}-${i}`, component: widgets[m.widgetId], props: m.props ?? {} }))
      .filter((b) => !!b.component),
  );

  // Empty fallback (§9.4): a default paragraph shows ONLY when there are no messages AND no
  // background was DECLARED. If `background` is set (even if all mounts drop as unknown) it "shows
  // through" — no separate empty paragraph.
  const backgroundDeclared = $derived((background?.length ?? 0) > 0);
  const showEmpty = $derived(count === 0 && !backgroundDeclared);

  // Composite-derived stable child ids so the self-registering children read cleanly (the
  // entity-panel / message precedent). Each message → `<id>__m-<msgid>`, its own atomics then nest
  // `<id>__m-<msgid>__avatar` etc. — a clean parent-prefix chain (0 orphans both directions).
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);
  const msgId = (mid: string) => (id ? `${id}__m-${mid}` : undefined);

  function selectRow(mid: string) {
    selected = mid;
    onSelect?.(mid);
  }

  // G — aggregate getter (§9.6). `atBottom` initialises to `true` in A (B drives it live from the
  // scroll machine); the rest are the shell observables (count / selection / empty / grouping /
  // dividers / background pair) so grouping + background are CDP-readable now.
  const debug = () => ({
    count,
    selected: selected ?? null,
    hasEmpty: count === 0,
    groupedCount,
    dividerCount,
    atBottom: true,
    backgroundMountCount: resolvedBg.length,
    backgroundLive,
  });
</script>

<!-- dd-composite root = the scroll viewport (role="log"). The background layer and the row list are
  SIBLINGS: the background is `position:absolute; inset:0` behind (z-index 0, does not scroll — chat
  wallpaper), the rows sit above (z-index 1). Structure (scroll / stacking) lives in the scoped
  <style> below; all appearance is in ui/assets/skin.css. -->
<div class="message-stream" role="log" use:envelope={{ name: 'message-stream', id, debug }}>
  {#if resolvedBg.length}
    <div class="message-stream-bg" aria-hidden="true">
      {#each resolvedBg as b (b.key)}
        {@const W = b.component}
        <W {...b.props} {backgroundLive} />
      {/each}
    </div>
  {/if}

  <div class="message-stream-rows">
    {#if showEmpty}
      <!-- never bare: a centered default paragraph when empty with no wallpaper (§9.4). -->
      <div class="message-stream-empty">
        <Paragraph text="No messages yet" id={cid('empty')} />
      </div>
    {:else}
      {#each rows as row (row.key)}
        {#if row.kind === 'divider'}
          <div class="day-divider" role="separator">{row.label}</div>
        {:else}
          <div
            class="message-stream-row"
            data-selected={row.descriptor.id === selected || undefined}
            onclick={() => selectRow(row.descriptor.id)}
            role="presentation"
          >
            <Message descriptor={row.descriptor} grouped={row.grouped} {widgets} id={msgId(row.descriptor.id)} />
          </div>
        {/if}
      {/each}
    {/if}
  </div>
</div>

<style>
  /* Structure only (scroll viewport + layer stacking); appearance is in skin.css. The literal
     `message-stream` class keeps this scoped rule from being pruned as unused — envelope also
     supplies it (N-023, supplies-never-erases). */
  .message-stream {
    position: relative;
    overflow-y: auto;
    min-height: 64px;
    max-height: 340px;
  }
  .message-stream-bg {
    position: absolute;
    inset: 0;
    z-index: 0;
    pointer-events: none;
  }
  .message-stream-rows {
    position: relative;
    z-index: 1;
  }
</style>
