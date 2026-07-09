<script lang="ts">
  // message-stream — a data-dependent COMPOSITE (M-RP5.6, the `entity-panel` analogue for the
  // message dd sub-family; `docs/xgen-dd-message-family-phase0.md` v1.2 §9). It owns the
  // RELATIONSHIPS between messages — chronological ordering (consumer-supplied), grouping
  // computation, day-dividers, empty state, a persistent background layer, and the SCROLL MACHINE
  // (step B) — while each child `message` owns HOW ONE MESSAGE LOOKS.
  //
  // STEP A (shipped) built the shell: the scroll viewport is `overflow-y:auto` / `max-height:340px`.
  // STEP B (this pass, §9.10) adds the scroll behaviour on that viewport:
  //   1. `atBottom` LIVE — a `scroll` listener, rAF-throttled, one const `BOTTOM_THRESHOLD_PX = 80`;
  //      getter G reads the live `$state` (the A `true` stub is gone).
  //   2. Stick-to-bottom on append when at/near bottom; no yank + jump-to-latest pill when scrolled up.
  //   3. Preserve-position-on-prepend (older-load anchor): capture scrollHeight in `$effect.pre`, after
  //      `tick()` re-apply the delta; prepend detected by count-grew AND prev-first-id no-longer-at-0.
  //   4. Initial scroll to bottom on mount (a chat opens at the latest).
  //
  // ROOT — `<div class="message-stream" role="log" use:envelope>` (§9): a scrolling live region, NOT
  // a `listbox` (a chat is a log, not a select-one-of list) → click-select, no roving tabindex. The
  // root IS the scroll viewport; it is wrapped in a NON-scrolling `.message-stream-shell` (position:
  // relative) so the jump-pill can sit absolute over the VISIBLE viewport (an absolute child INSIDE
  // the scroll container would scroll away with the content). This mirrors the `entity-panel` rooting
  // precedent (outermost DOM = chrome wrapper; the envelope rides the inner scroll element). Children
  // = `message`s + interleaved `<div class="day-divider" role="separator">` rows, in the GIVEN order.
  //
  // GROUPING + DIVIDERS are pure (`stream/grouping.ts`, colocated + unit-testable): `computeRows`
  // walks the ordered messages once, producing the interleaved row list and each message's
  // stream-computed `grouped` flag (passed DOWN as a PROP — never a descriptor field, Phase-0 §4/§5).
  // B does NOT touch grouping — it recomputes for free (`rows` is `$derived`) as the array mutates.
  //
  // WIDGET SOCKETS (W-12/W-13). `background?: WidgetMount[]` is a persistent fixed layer behind the
  // log (chat-wallpaper); `backgroundLive?` is the settings switch passed into each mount (a reactive
  // widget renders frozen when false; a static object ignores it) — the store BINDING is M-RP6.x, A
  // just exposes the prop. Both background mounts and each child `message`'s `details` socket resolve
  // `widgetId` → component via the consumer `widgets` registry (the `message` precedent), DROPPING an
  // unknown id on render (W-13). `core` imports no concrete widget.
  import { envelope } from '$common/components/base/envelope';
  import { tick, untrack } from 'svelte';
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

  // Single `now` captured at mount → deterministic divider labels for this render (live re-labelling
  // as wall-clock advances is not a Step-A/B concern; the sampler dates append/prepend relative to it).
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

  // ── SCROLL MACHINE (step B, §9.10) ──────────────────────────────────────────────────────────
  // One build-time const governs both `atBottom` and pill visibility — no hysteresis / second band.
  const BOTTOM_THRESHOLD_PX = 80; // Joe-tunable (§9.10 Q1)

  let viewportEl: HTMLDivElement | undefined = $state();
  let atBottom = $state(true); // live once mounted; initial `true` matches the mount-to-bottom scroll

  // `atBottom` = the viewport is within the threshold of its scroll bottom. Read imperatively from
  // geometry so a `scroll` event, a mount scroll, or a stick/jump all converge on the same truth.
  function recomputeAtBottom() {
    const el = viewportEl;
    if (!el) return;
    atBottom = el.scrollHeight - el.scrollTop - el.clientHeight <= BOTTOM_THRESHOLD_PX;
  }

  // rAF-throttle the scroll listener — coalesce a burst of scroll events to ONE recompute per frame
  // (a clean, non-flickering CDP read; §9.10 Q1).
  let rafPending = false;
  function onScroll() {
    if (rafPending) return;
    rafPending = true;
    requestAnimationFrame(() => {
      rafPending = false;
      recomputeAtBottom();
    });
  }

  function jumpToBottom() {
    const el = viewportEl;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
    recomputeAtBottom(); // → atBottom true → pill hides
  }

  // Prepend / append classification refs (plain closure vars — display truth, not reactive UI).
  let mounted = false;
  let prevFirstId: string | undefined;
  let prevLen = 0;

  // `$effect.pre` runs BEFORE the DOM flushes for a `messages` change, so `el.scrollHeight/scrollTop`
  // here are the PRE-update geometry. We classify the update (mount / append / prepend), then `tick()`
  // to apply after the flush. `atBottom` is read UNTRACKED — otherwise scrolling (which updates it)
  // would re-trigger this effect and re-anchor spuriously.
  $effect.pre(() => {
    const msgs = messages; // track
    const el = viewportEl; // track (assigned once at mount)
    if (!el) return;

    const beforeScrollHeight = el.scrollHeight;
    const beforeScrollTop = el.scrollTop;
    const wasAtBottom = untrack(() => atBottom);
    const firstBefore = prevFirstId;
    const lenBefore = prevLen;

    const newLen = msgs.length;
    const grew = newLen > lenBefore;
    // prepend = count grew AND the previous first id is no longer at index 0 (older inserted on top);
    // append = count grew, first id still leads. (§9.10 Q3 heuristic — self-contained, no new prop.)
    const isPrepend =
      grew && firstBefore !== undefined && msgs.findIndex((m) => m.id === firstBefore) > 0;

    // advance the refs for the next comparison
    prevFirstId = msgs[0]?.id;
    prevLen = newLen;

    tick().then(() => {
      const e = viewportEl;
      if (!e) return;
      if (!mounted) {
        // §9.10 sub-point 1 — chat opens at the newest; neither append nor prepend.
        e.scrollTop = e.scrollHeight;
        mounted = true;
        recomputeAtBottom();
        return;
      }
      if (isPrepend) {
        // §9.10 Q3 — hold the previously-top message at the same visual offset.
        e.scrollTop = beforeScrollTop + (e.scrollHeight - beforeScrollHeight);
      } else if (grew && wasAtBottom) {
        // §9.10 Q2 — stick: append while at bottom keeps the newest in view.
        e.scrollTop = e.scrollHeight;
      }
      // else: append while scrolled up → NO yank (leave scrollTop); the pill shows (!atBottom).
      recomputeAtBottom();
    });
  });

  // G — aggregate getter (§9.6). `atBottom` is now the LIVE `$state` (B), the rest are the shell
  // observables (count / selection / empty / grouping / dividers / background pair) so grouping +
  // scroll + background are CDP-readable.
  const debug = () => ({
    count,
    selected: selected ?? null,
    hasEmpty: count === 0,
    groupedCount,
    dividerCount,
    atBottom,
    backgroundMountCount: resolvedBg.length,
    backgroundLive,
  });
</script>

<!-- Outermost = a NON-scrolling positioning shell (entity-panel rooting precedent). The scroll
  viewport (`.message-stream`, role="log", envelope root) is the first child; the jump-pill is its
  SIBLING so `position:absolute` anchors it to the visible viewport, not the scrollable content.
  Inside the viewport, the background layer and the row list are SIBLINGS: the background is
  `position:absolute; inset:0` behind (z-index 0, does not scroll — chat wallpaper), the rows sit
  above (z-index 1). Structure (scroll / stacking / pill placement) lives in the scoped <style>
  below; all appearance is in ui/assets/skin.css. -->
<div class="message-stream-shell">
  <div
    class="message-stream"
    role="log"
    bind:this={viewportEl}
    onscroll={onScroll}
    use:envelope={{ name: 'message-stream', id, debug }}
  >
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

  {#if !atBottom}
    <!-- jump-to-latest pill — INLINE CHROME (§9.10 Q2, the `day-divider` precedent): NOT a component,
      NOT registered in __XGEN_DEBUG__; its state is already CDP-observable via getter `atBottom`. -->
    <button type="button" class="jump-to-latest" aria-label="Jump to latest" onclick={jumpToBottom}>
      ↓ Latest
    </button>
  {/if}
</div>

<style>
  /* Structure only (positioning shell + scroll viewport + layer stacking + pill placement);
     appearance is in skin.css. The literal `message-stream` class keeps this scoped rule from being
     pruned as unused — envelope also supplies it (N-023, supplies-never-erases). */
  .message-stream-shell {
    position: relative;
  }
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
  /* pill placement — bottom-right over the VISIBLE viewport (anchored to the non-scrolling shell),
     above the rows. Pill look (shape / bg / border) is in skin.css. */
  .jump-to-latest {
    position: absolute;
    bottom: 12px;
    right: 12px;
    z-index: 2;
  }
</style>
