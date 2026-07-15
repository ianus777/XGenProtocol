<script lang="ts">
  // region-shell — renderer A (M-RP6.1f) + the move-gesture controller (M-RP7.4). Reads the D-103 `Layout`
  // descriptor + a `widgets` registry, resolves the tree (pure walk, `./resolve`), and tiles the resolved
  // leaves into the centre. Renderer B (the owned dock engine, M-RP7) is a renderer UPGRADE on the SAME
  // descriptor; drag-to-dock is its first user-visible verb.
  //
  // `core` tier (D4): the NODE app inherits this at M-RP7.7 — like `menu-bar`/`status-bar` it imports NO
  // Tauri, NO protocol. The descriptor + leaf components are injected by the shell.
  //
  // Registers exactly ONE getter G (§4). `leafCount`/`widgetIds` are the RESOLVED, RENDERED truth, which is
  // what makes a drop CDP-provable; `dragging` reports the region currently under a move gesture.
  //
  // 🔒 MOVE GESTURE — ONE grid-level overlay (D1). The four drop bands + the drag ghost live in a SINGLE
  // element mounted LAST, above every tile. This designs the whole N-119 paint-order class out in one
  // stroke: one element above everything cannot be painted over by a sibling (per-tile overlays would
  // re-fight the seam's z-index battle on every tile). The band under the pointer is chosen by HIT-TEST
  // (`elementFromPoint` → `data-edge`), NEVER by computing a quadrant from the rect (D2 / N-124 — hit-test
  // the thing that paints; do not run a second model of the truth that can drift). A band exists ONLY over a
  // rendered `.region-tile`, so a hole (no `regionId`) offers no band — unsayable, not guarded (D3, D-116).
  // A band whose drop would be a no-op does not highlight (D4) — the check is `isMoveNoop`, the SAME
  // predicate `move` itself reads, so a highlighted band and a committed move can never disagree.
  //
  // Zero component-local CSS (N-090): flex, gaps, tracks, overflow, mins, band appearance — ALL in skin.css.
  // The one non-appearance inline is a split child's `flex` weight (DATA, region-node) and the bands' rects
  // (DATA — live geometry of the hovered tile, computed here; the skin owns their look, PROVISIONAL).
  import { envelope } from '$common/components/base/envelope';
  import type { Component } from 'svelte';
  import type { FoldAxis, Layout } from './types';
  import { resolveLayout, treeDepth } from './resolve';
  import { isMoveNoop, type Edge } from './mutate';
  import RegionNode from './region-node.svelte';

  let {
    layout,
    widgets = {},
    titles = {},
    onFold,
    onResize,
    onMove,
    id,
  }: {
    layout: Layout;
    widgets?: Record<string, Component>; // widgetId → component; an unknown id is dropped by resolveLayout (W-13)
    titles?: Record<string, string>; // widgetId → tile title (M-RP7.1, D2); threaded to each tile
    onFold?: (regionId: string, collapsed: FoldAxis | undefined) => void; // fold seam (M-RP7.1b, D6)
    onResize?: (path: number[], aIdx: number, bIdx: number, fraction: number) => void; // splitter seam (N-120)
    /** Move commit (M-RP7.4). Called ONCE on a valid drop with the three `move` arguments (→ `handleMove`). */
    onMove?: (source: string, target: string, edge: Edge) => void;
    id?: string;
  } = $props();

  const knownIds = $derived(new Set(Object.keys(widgets)));
  const resolved = $derived(resolveLayout(layout, knownIds));

  // ── Move gesture (M-RP7.4, D1) ──────────────────────────────────────────────────────────────────────
  const EDGES: Edge[] = ['top', 'bottom', 'left', 'right'];
  const BAND_FRAC = 0.3; // PROVISIONAL (M-RP-SKIN): edge-band depth as a fraction of the tile dimension.
  const THRESHOLD = 4; // px of travel before a grip press becomes a drag (V7 — a press under this is a click).

  type Rect = { left: number; top: number; width: number; height: number };
  // The whole gesture's live state. null = idle. `active:false` = pressed but under the threshold (no overlay).
  let drag = $state<null | {
    sourceId: string;
    active: boolean;
    x: number; y: number; // ghost position (viewport px)
    hover: { targetId: string; rect: Rect } | null;
    edge: Edge | null; // the highlighted edge (null = none / centre / suppressed)
    suppressed: Set<Edge>; // edges whose drop would be a no-op (D4) — recomputed per hovered tile
  }>(null);

  // pointerdown record, kept until the threshold decides move-vs-click. `grip` holds the capture.
  let pending: { sourceId: string; sx: number; sy: number; pid: number; grip: HTMLElement } | null = null;

  function onMoveStart(regionId: string, e: PointerEvent): void {
    const grip = e.currentTarget as HTMLElement;
    e.preventDefault(); // N-118: suppress the compat mousedown → no text selection, no native HTML5 drag
    try { grip.setPointerCapture(e.pointerId); } catch { /* not capturable — window listeners still catch */ }
    pending = { sourceId: regionId, sx: e.clientX, sy: e.clientY, pid: e.pointerId, grip };
    window.addEventListener('pointermove', onWinMove);
    window.addEventListener('pointerup', onWinUp);
    window.addEventListener('pointercancel', onWinCancel);
    window.addEventListener('keydown', onWinKey);
  }

  function onWinMove(e: PointerEvent): void {
    if (!pending || e.pointerId !== pending.pid) return;
    if (!drag) {
      if (Math.abs(e.clientX - pending.sx) < THRESHOLD && Math.abs(e.clientY - pending.sy) < THRESHOLD) return;
      drag = { sourceId: pending.sourceId, active: true, x: pending.sx, y: pending.sy, hover: null, edge: null, suppressed: new Set() };
    }
    drag.x = e.clientX;
    drag.y = e.clientY;
    hitTest(e.clientX, e.clientY);
  }

  // D2 — the pointer's edge is READ off the element that actually paints, never computed from the rect.
  function hitTest(x: number, y: number): void {
    if (!drag) return;
    const el = document.elementFromPoint(x, y) as HTMLElement | null;
    const band = el?.closest?.('[data-edge]') as HTMLElement | null;
    if (band && band.dataset.target) {
      const edge = band.dataset.edge as Edge | 'center';
      drag.edge = edge !== 'center' && !drag.suppressed.has(edge as Edge) ? (edge as Edge) : null;
      return;
    }
    // No band under the pointer → we may have entered a new tile whose bands are not drawn yet.
    const tileEl = el?.closest?.('.region-tile[data-region-id]') as HTMLElement | null;
    if (tileEl) {
      const targetId = tileEl.dataset.regionId as string;
      if (!drag.hover || drag.hover.targetId !== targetId) {
        const r = tileEl.getBoundingClientRect();
        drag.hover = { targetId, rect: { left: r.left, top: r.top, width: r.width, height: r.height } };
        // D4 — suppress the no-op edges up front (source over its own tile ⇒ all four). isMoveNoop is the
        // SAME predicate `move` reads, so the highlight can never promise a drop the commit would reject.
        drag.suppressed = new Set<Edge>(EDGES.filter((edge) => isMoveNoop(layout, drag!.sourceId, targetId, edge)));
        drag.edge = null; // resolves next move, when the bands exist to hit-test (D2)
      }
      return;
    }
    // hole / seam / chrome / outside the grid → no target (D3: a band cannot exist off a tile).
    drag.hover = null;
    drag.edge = null;
  }

  function onWinUp(e: PointerEvent): void {
    if (pending && e.pointerId === pending.pid && drag) {
      const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null;
      const band = el?.closest?.('[data-edge]') as HTMLElement | null;
      const edge = band?.dataset.edge as Edge | 'center' | undefined;
      const target = band?.dataset.target;
      // The ONE descriptor write — only on a real, non-suppressed band (D2 hit-test + D4 re-check).
      if (target && edge && edge !== 'center' && !isMoveNoop(layout, drag.sourceId, target, edge)) {
        onMove?.(drag.sourceId, target, edge);
      }
    }
    teardown(); // a drag that changes nothing leaves no trace (§2) — teardown runs on EVERY end
  }

  function onWinCancel(): void { teardown(); }
  function onWinKey(e: KeyboardEvent): void { if (e.key === 'Escape') teardown(); }

  function teardown(): void {
    const p = pending;
    pending = null;
    drag = null;
    window.removeEventListener('pointermove', onWinMove);
    window.removeEventListener('pointerup', onWinUp);
    window.removeEventListener('pointercancel', onWinCancel);
    window.removeEventListener('keydown', onWinKey);
    if (p) { try { p.grip.releasePointerCapture(p.pid); } catch { /* already released */ } }
  }

  // The band RECTS are derived from the hovered tile (D2 places the hit targets; the pointer's edge is still
  // chosen by hit-test, not by these numbers). Four edge bands + an inert centre, tiling the tile so a hover
  // always lands on exactly one — corners belong to top/bottom (full width), the centre is inert.
  const bands = $derived.by(() => {
    if (!drag?.hover) return [] as { edge: Edge | 'center'; left: number; top: number; width: number; height: number }[];
    const { left, top, width, height } = drag.hover.rect;
    const f = BAND_FRAC;
    return [
      { edge: 'top' as const, left, top, width, height: height * f },
      { edge: 'bottom' as const, left, top: top + height * (1 - f), width, height: height * f },
      { edge: 'left' as const, left, top: top + height * f, width: width * f, height: height * (1 - 2 * f) },
      { edge: 'right' as const, left: left + width * (1 - f), top: top + height * f, width: width * f, height: height * (1 - 2 * f) },
      { edge: 'center' as const, left: left + width * f, top: top + height * f, width: width * (1 - 2 * f), height: height * (1 - 2 * f) },
    ];
  });

  const sourceTitle = $derived(drag ? (titles[drag.sourceId] ?? drag.sourceId) : '');

  // Aggregate getter G — resolved (rendered) truth + the live drag source (M-RP7.4, for CDP teardown proofs).
  const debug = () => ({
    version: layout?.version ?? null,
    leafCount: resolved.leafIds.length,
    widgetIds: resolved.leafIds,
    droppedCount: resolved.dropped.length,
    unsupportedCount: resolved.unsupported,
    depth: treeDepth(resolved.root),
    dragging: drag?.sourceId ?? null,
  });
</script>

<div class="region-shell" use:envelope={{ name: 'region-shell', id, debug }}>
  {#if resolved.root}
    <RegionNode
      node={resolved.root}
      {widgets}
      {titles}
      {onFold}
      {onResize}
      {onMoveStart}
      draggingId={drag?.sourceId ?? null}
      path={[]}
    />
  {/if}

  <!-- D1 — the ONE grid-level overlay, mounted LAST, above every tile. Root is pointer-events:none (skin);
    only the bands are hit-testable. Drawn only while a drag is active. -->
  {#if drag?.active}
    <div class="region-drag-overlay">
      {#each bands as b (b.edge)}
        <div
          class="region-drop-band"
          data-edge={b.edge}
          data-target={b.edge !== 'center' && drag.hover ? drag.hover.targetId : undefined}
          data-active={b.edge === drag.edge || undefined}
          data-noop={b.edge !== 'center' && drag.suppressed.has(b.edge) ? true : undefined}
          style="left:{b.left}px;top:{b.top}px;width:{b.width}px;height:{b.height}px"
        ></div>
      {/each}
      <div class="region-drag-ghost" style="left:{drag.x}px;top:{drag.y}px">{sourceTitle}</div>
    </div>
  {/if}
</div>
