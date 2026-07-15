<script lang="ts">
  // region-node — the INTERNAL recursion part of renderer A (M-RP6.1f). NOT a catalogued component:
  // it carries NO `use:envelope`, so it registers NOTHING into window.__XGEN_DEBUG__ (the `sb-cell` /
  // N-064 opt-out). The meaningful registry entries are `region-shell` (one aggregate getter, §4) + each
  // leaf's own `region-tile` (M-RP7.1) + the widget the tile hosts. The SEAM (M-RP7.2) registers nothing
  // either (L8) — it is the split's chrome, like the hairline it replaces.
  //
  // It renders ONE resolved node:
  //   - `leaf`  → a `region-tile` (M-RP7.1) framing the widgetId's component from the injected registry.
  //               The tile owns the chrome (stripe · fold · dead grips) and the scroll box (D5, ex-leaf).
  //   - `split` → a `.region-split` flex box; `dir` rides `data-dir` (the skin reads it — flex-direction
  //               is layout = skin, N-090), and each child gets its descriptor weight as an INLINE
  //               `flex: {n} 1 0` (the ONE skin exception: `sizes[]` is DATA). Between children sits a
  //               `.region-seam` — the drag handle (M-RP7.2), one fewer than the children count.
  //
  // AXIS (M-RP7.1, D8): a split passes its OWN `dir` down to each child as `axis` — the parent split's
  // direction, which is the axis a leaf collapses along. A leaf reflects it as the tile's `data-axis`.
  // The root node (called from region-shell with no `axis`) defaults to `col`.
  //
  // PATH (M-RP7.2, L5): a split threads its own `path` (child indices from the root) down as `[...path, i]`.
  // The seam gesture reports `(path, seamIndex, fraction)` to the shell, which calls `resizeSplit`. DERIVED,
  // no schema change; M-RP7.4's `move` needs the identical addressing, so it is paid once here.
  import type { Component } from 'svelte';
  import type { ResolvedNode } from './resolve';
  import { carriesMainAxisWeight, splitShrinkWraps } from './resolve';
  import type { FoldAxis } from './types';
  import RegionNode from './region-node.svelte'; // self-import recursion (Svelte 5; no <svelte:self>)
  import RegionTile from './region-tile.svelte';

  let {
    node,
    widgets,
    titles = {},
    onFold,
    onResize,
    onMoveStart,
    draggingId = null,
    locked = false,
    axis,
    flex,
    path = [],
  }: {
    node: ResolvedNode;
    widgets: Record<string, Component>;
    /** widgetId → display title (D2); resolved by the shell from CLIENT_PLUGINS + REGION_NAMES. */
    titles?: Record<string, string>;
    /** Fold toggle seam threaded to every tile (D6). `undefined` collapsed ⇒ unfold. */
    onFold?: (regionId: string, collapsed: FoldAxis | undefined) => void;
    /** Splitter-resize seam (M-RP7.2/7.3). `path` addresses THIS split (DESCRIPTOR indices); the pair is the
     *  two neighbours' DESCRIPTOR indices (`srcIndex`, N-120) — never resolved positions. */
    onResize?: (path: number[], aIdx: number, bIdx: number, fraction: number) => void;
    /** Move-grip pointerdown seam threaded to every tile (M-RP7.4, D1) — the shell owns the gesture. */
    onMoveStart?: (regionId: string, e: PointerEvent) => void;
    /** The regionId currently being dragged (M-RP7.4) — a tile marks itself `dragging` when it matches. */
    draggingId?: string | null;
    /** Grid lock (M-RP7.6): a locked seam becomes a DEAD seam (no listeners, no cursor, no hit-expansion —
     *  `data-live` folds `!locked` into `live`, so the honesty layer matches the algebra's refusal). Threaded
     *  recursively; the tile suppresses its grip + fold buttons on the same flag. */
    locked?: boolean;
    /** The PARENT split's `dir` — the axis a leaf collapses along/across (§4.1). Undefined at the root. */
    axis?: 'row' | 'col';
    /** Descriptor weight from the parent split; applied inline (skin exception, D4). */
    flex?: number;
    /** Child indices from the root TO THIS node (L5). Root = `[]`. */
    path?: number[];
  } = $props();

  // A split's children are keyed by STABLE node identity, not by position (M-RP7.3, N-120's sibling). Index
  // keying reused a tile instance across DIFFERENT regionIds the moment a `move` changed a split's child
  // count/order — and a tile's `use:envelope` stamps `data-debug-id` on MOUNT without re-keying, so a reused
  // instance kept a STALE id and the registry desynced (measured: a move clobbered 2 entries, region-tiles
  // stamped `#region-rooms` while titled "Room header"). fold/resize never changed a split's child SET, so
  // this stayed latent until `move`. A leaf keys by its (globally unique) widgetId; a split by its subtree's
  // leaf ids — so a split whose leaf-set is unchanged reconciles IN PLACE (a resize), and one that gained or
  // lost a leaf is torn down and rebuilt (correct — its children re-register under the right ids).
  function nodeKey(n: ResolvedNode): string {
    return n.type === 'leaf' ? `L:${n.widgetId}` : `S:${n.children.map(nodeKey).join(',')}`;
  }

  // Shrink-wrap (§4.4): a split whose children are ALL leaves folded ACROSS its own axis takes `flex: 0 0
  // auto` (the inline weight is omitted so the skin's `[data-shrinkwrap]` rule wins), so it collapses to its
  // children's strip size and the parent's SIBLINGS absorb the freed weight. DERIVED (a stored flag would go
  // stale the instant one child unfolds). Shared with seam liveness via `splitShrinkWraps` (L4 — one source).
  const shrinkWrap = $derived(splitShrinkWraps(node));

  // Omit the inline weight when shrink-wrapping so the skin's `flex: 0 0 auto` takes over (else an inline
  // `flex: {n} 1 0` would out-specify it and the split would keep its old width — the leaf-fold precedent).
  const flexStyle = $derived(
    shrinkWrap ? undefined : flex !== undefined ? `flex: ${flex} 1 0` : undefined,
  );

  // ── Splitter resize (M-RP7.2, L3/L4) ──────────────────────────────────────────────────────────────
  // Live preview writes a TRANSIENT `dragSizes` (floats allowed — L3, never reaches the descriptor) used
  // for the pair's inline flex ONLY. `node.sizes` is not touched; the descriptor is written ONCE on
  // `pointerup` via `onResize` (integers, L2). On cancel/Escape, `dragSizes` clears and NOTHING is called.
  let dragSizes = $state<number[] | null>(null);
  const effectiveSizes = $derived(node.type === 'split' ? (dragSizes ?? node.sizes) : []);

  // A seam is LIVE iff BOTH its neighbours carry a main-axis weight (L4) — folded-along and shrink-wrapped
  // neighbours fall out with no special case. A dead seam stays as the visual divider but gets NO cursor,
  // NO hit expansion, and NO listeners.
  function seamLive(seamIndex: number): boolean {
    if (node.type !== 'split') return false;
    return (
      carriesMainAxisWeight(node.children[seamIndex], node.dir) &&
      carriesMainAxisWeight(node.children[seamIndex + 1], node.dir)
    );
  }

  // Per-drag captured state. `origin`/`combined` are the pair's fixed px extent along the split axis, taken
  // ONCE at pointerdown (the pair's total space does not change during the drag — only the boundary moves).
  let drag: {
    seamIndex: number;
    axis: 'x' | 'y';
    origin: number; // leading edge of the pair (px)
    combined: number; // the two neighbours' summed extent along the axis (px)
    pairWeight: number; // node.sizes[i] + node.sizes[i+1] — the weight the pair splits (unchanged during drag)
    minPx: number; // --region-min, read from the skin (N-090); the clamp STOPS here, never auto-folds (L6)
    snap: number; // --region-snap (0 if unreadable → no snap)
    pointerId: number;
    seam: HTMLElement;
  } | null = null;

  function computeFraction(e: PointerEvent): number {
    const d = drag!;
    let aPx = (d.axis === 'x' ? e.clientX : e.clientY) - d.origin;
    if (d.snap > 0) aPx = Math.round(aPx / d.snap) * d.snap;
    const lo = d.minPx;
    const hi = d.combined - d.minPx;
    aPx = hi >= lo ? Math.max(lo, Math.min(hi, aPx)) : d.combined / 2; // clamp: both neighbours ≥ min. It stops.
    return aPx / d.combined;
  }

  function startResize(e: PointerEvent, seamIndex: number): void {
    if (node.type !== 'split') return;
    const seam = e.currentTarget as HTMLElement;
    const prev = seam.previousElementSibling as HTMLElement | null;
    const next = seam.nextElementSibling as HTMLElement | null;
    if (!prev || !next) return;

    e.preventDefault(); // belt to N-118's braces: no compat mousedown → no text selection starts
    seam.setPointerCapture(e.pointerId);

    // The minimum is a skin token (N-090). --region-min = var(--region-stripe), both hoisted to
    // .region-shell so they inherit down to the seam. An EMPTY read is NaN and a clamp of NaN is no clamp —
    // so ASSERT the read and abort loudly rather than fall back to a literal (a second source of truth).
    const cs = getComputedStyle(seam);
    const minPx = parseFloat(cs.getPropertyValue('--region-min'));
    if (!Number.isFinite(minPx)) {
      console.error('[layout] --region-min unreadable on seam — aborting resize (no literal fallback)');
      try { seam.releasePointerCapture(e.pointerId); } catch { /* capture may not be held */ }
      return;
    }
    const snap = parseFloat(cs.getPropertyValue('--region-snap'));

    const pr = prev.getBoundingClientRect();
    const nr = next.getBoundingClientRect();
    const row = node.dir === 'row';
    drag = {
      seamIndex,
      axis: row ? 'x' : 'y',
      origin: row ? pr.left : pr.top,
      combined: row ? pr.width + nr.width : pr.height + nr.height,
      pairWeight: node.sizes[seamIndex] + node.sizes[seamIndex + 1],
      minPx,
      snap: Number.isFinite(snap) ? snap : 0,
      pointerId: e.pointerId,
      seam,
    };
    window.addEventListener('keydown', onDragKey);
  }

  function moveResize(e: PointerEvent): void {
    if (!drag || node.type !== 'split') return;
    const fraction = computeFraction(e);
    const a = fraction * drag.pairWeight;
    const next = node.sizes.slice();
    next[drag.seamIndex] = a; // floats — live preview only (L3)
    next[drag.seamIndex + 1] = drag.pairWeight - a;
    dragSizes = next;
  }

  function endResize(e: PointerEvent): void {
    if (!drag || node.type !== 'split') { cleanupDrag(); return; }
    const fraction = computeFraction(e);
    // The seam sits between resolved neighbours `seamIndex` and `seamIndex+1`; the descriptor write needs
    // their DESCRIPTOR indices (`srcIndex`, N-120) — a dropped sibling makes the resolved position lie, and
    // they need not be adjacent in the descriptor. The LIVE PREVIEW stayed in resolved space (dragSizes /
    // effectiveSizes / flex); ONLY this write crosses over.
    const aIdx = node.children[drag.seamIndex].srcIndex;
    const bIdx = node.children[drag.seamIndex + 1].srcIndex;
    cleanupDrag(); // clears the transient preview + releases capture BEFORE the descriptor write
    onResize?.(path, aIdx, bIdx, fraction); // written ONCE, integers (L2/L3)
  }

  function cancelResize(): void {
    cleanupDrag(); // Escape / pointercancel / lostpointercapture → drop the preview, call NOTHING (L3)
  }

  function cleanupDrag(): void {
    const d = drag;
    drag = null;
    dragSizes = null;
    window.removeEventListener('keydown', onDragKey);
    if (d) { try { d.seam.releasePointerCapture(d.pointerId); } catch { /* already released */ } }
  }

  function onDragKey(e: KeyboardEvent): void {
    if (e.key === 'Escape') cancelResize();
  }
</script>

{#if node.type === 'leaf'}
  <RegionTile
    regionId={node.widgetId}
    title={titles[node.widgetId] ?? node.widgetId}
    collapsed={node.collapsed}
    axis={axis ?? 'col'}
    {flex}
    {onFold}
    {onMoveStart}
    {locked}
    dragging={node.widgetId === draggingId}
  >
    {#if widgets[node.widgetId]}
      {@const W = widgets[node.widgetId]}
      <W regionId={node.widgetId} />
    {/if}
  </RegionTile>
{:else}
  <div class="region-split" data-dir={node.dir} data-shrinkwrap={shrinkWrap || undefined} style={flexStyle}>
    {#each node.children as child, i (nodeKey(child))}
      {#if i > 0}
        {@const live = seamLive(i - 1)}
        <!-- The seam between children[i-1] and children[i]; seamIndex = i-1. `user-select`/`touch-action`
          none live on the skin (N-118). A DEAD seam gets NO listeners (L4) — it is only the divider.
          M-RP7.6: `locked` folds into liveness (`draggable`) so a locked seam is dead in EVERY sense the
          skin reads (`data-live` drives the cursor + hit-expansion + z-index) — no live-looking dead
          control (D1). The load-bearing refusal is still the shell handler; this is the honesty layer. -->
        {@const draggable = live && !locked}
        <div
          class="region-seam"
          data-dir={node.dir}
          data-live={draggable}
          onpointerdown={draggable ? (e) => startResize(e, i - 1) : undefined}
          onpointermove={draggable ? moveResize : undefined}
          onpointerup={draggable ? endResize : undefined}
          onpointercancel={draggable ? cancelResize : undefined}
          onlostpointercapture={draggable ? cancelResize : undefined}
        ></div>
      {/if}
      <RegionNode
        node={child}
        {widgets}
        {titles}
        {onFold}
        {onResize}
        {onMoveStart}
        {draggingId}
        {locked}
        axis={node.dir}
        flex={effectiveSizes[i]}
        path={[...path, child.srcIndex]}
      />
    {/each}
  </div>
{/if}
