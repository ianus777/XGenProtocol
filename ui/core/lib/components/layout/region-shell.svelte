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
  import type { WidgetMount } from '../data-dependent/types';
  import { resolveMounts } from '../data-dependent/mounts';
  import { resolveLayout, treeDepth, carriesMainAxisWeight, type ResolvedNode } from './resolve';
  import { isMoveNoop, move, type Edge } from './mutate';
  import RegionNode from './region-node.svelte';

  let {
    layout,
    widgets = {},
    titles = {},
    background,
    backgroundLive = true,
    bgWidgets = {},
    onFold,
    onResize,
    onMove,
    locked = false,
    id,
  }: {
    layout: Layout;
    widgets?: Record<string, Component>; // widgetId → component; an unknown id is dropped by resolveLayout (W-13)
    titles?: Record<string, string>; // widgetId → tile title (M-RP7.1, D2); threaded to each tile
    /** Grid-wide backdrop mounts (M-RP-PLATE, D1) — the `message-stream` `background` socket, one level up.
     *  undefined = none. Resolved against `bgWidgets` (NOT `widgets`), unknown id dropped (W-13). */
    background?: WidgetMount[];
    /** Settings switch passed into each background mount (unbound this arc — a hardcoded `true` from the
     *  shell; the M-RP-SETTINGS binding replaces the literal). A static plate ignores it (message-stream). */
    backgroundLive?: boolean;
    /** widgetId → component for the BACKGROUND socket ONLY (D1). SEPARATE from the tile `widgets` map: the
     *  tile map is region-id-keyed and feeds resolveLayout/W-13 for LEAVES; a plate id is not a region id
     *  and must never be mistaken for one. Two sockets, two registries, independently W-13-testable. */
    bgWidgets?: Record<string, Component>;
    onFold?: (regionId: string, collapsed: FoldAxis | undefined) => void; // fold seam (M-RP7.1b, D6)
    onResize?: (path: number[], aIdx: number, bIdx: number, fraction: number) => void; // splitter seam (N-120)
    /** Move commit (M-RP7.4). Called ONCE on a valid drop with the three `move` arguments (→ `handleMove`). */
    onMove?: (source: string, target: string, edge: Edge) => void;
    /** Grid lock (M-RP7.6): when true, every rearrange affordance goes element-absent / seam-dead. Threaded
     *  down; the load-bearing guard is the shell's handlers (`if (locked) return`), this only hides the UI.
     *  Bands/preview go inert TRANSITIVELY — a suppressed grip means `onMoveStart` never fires (G4). */
    locked?: boolean;
    id?: string;
  } = $props();

  const knownIds = $derived(new Set(Object.keys(widgets)));
  const resolved = $derived(resolveLayout(layout, knownIds));

  // The shell element — its first child is the rendered root node, whose rect is the grid's container box
  // for the M-RP7.4b proportion walk (the flex proportioning starts from the root split's box).
  let shellEl = $state<HTMLElement>();

  // Background socket (M-RP-PLATE, D1) — the `message-stream` shape exactly: resolve each declared widgetId
  // against `bgWidgets`, DROP an unknown id (W-13), so `backgroundMountCount` reports the RENDERED truth (a
  // dropped unknown lowers it). Resolved into the SEPARATE `bgWidgets` registry — never the tile `widgets`.
  //
  // M-RP6.9 (D-3, S-2b): "the `message-stream` shape exactly" was literally true — this was the THIRD
  // character-identical copy of the resolver, and the comment said so while the runbook still missed it.
  // Now the shared pure `resolveMounts()`; behaviour unchanged.
  // ⚠️ NO `idPrefix`, and that is load-bearing rather than stylistic: this socket mints no mount ids, and
  // `grid-plate` SELF-DEFAULTS `id = 'grid-plate'` so it registers as a stable enumerable
  // `grid-plate#grid-plate`. A prefix here would silently RENAME a registered widget and move every
  // baseline that counts it. Leave it absent unless that rename is the intent.
  const resolvedBg = $derived(resolveMounts(background, bgWidgets));

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

  // The DIVISION PREVIEW — M-RP7.4b: REHEARSE THE DROP, DRAW THE RESULT. 7.4a drew half the target as it is
  // NOW; but `move` does remove → collapse-degenerate → insert, and the *remove* reflows the grid BEFORE the
  // region lands (N-127) — so the region lands elsewhere (measured 60px off on the reflow-heavy case). The
  // fix: compute the preview from a DRY-RUN of `move` on the live descriptor, not from the current DOM.
  //   1. `move` is pure + total (M-RP7.3) → the dry-run has ZERO side-effect (V6); it returns the tree you'd
  //      get, and the SAME reference on a no-op (so `hypo === layout` ⇒ nothing to preview).
  //   2. resolve the hypothetical tree the SAME way the render does (drops/folds handled identically).
  //   3. find the moved leaf's PATH, then PROPORTION its weight down that path.
  // 🔒 The proportion MIRRORS the renderer's own two rules, it does not model flex a second time (D2/N-126):
  // each split child gets `flex: {weight} 1 0` → `weight / Σweights × axis`, EXCEPT a folded-along leaf or a
  // shrink-wrapped split which takes `flex: 0 0 auto` → a fixed `--region-stripe` strip, OUT of the weight
  // pool (the `carriesMainAxisWeight` test, reused from resolve.ts — a concept in the code is not re-derived).
  // Naive weight math lands within ~2px of the real render (§4 floor — gaps are NOT subtracted; 10× better
  // than 7.4a's 60px, but real, so recorded as "~2px", not "pixel-perfect"). Guard is still JUST `drag.edge`
  // (no-op/hole already gave null — D3/D4 inherited). READ-ONLY (D3): PAINT, never read back into detection.
  const previewRect = $derived.by(() => {
    if (!drag?.hover || !drag.edge || !shellEl) return null;
    const containerEl = shellEl.firstElementChild as HTMLElement | null; // the root node's box = the grid area
    if (!containerEl) return null;

    const hypo = move(layout, drag.sourceId, drag.hover.targetId, drag.edge);
    if (hypo === layout) return null; // move returns the input by reference on a no-op → nothing to preview
    const hypoRoot = resolveLayout(hypo, knownIds).root;
    if (!hypoRoot) return null;

    // Path (resolved child indices) from the hypothetical root down to the moved leaf.
    const path: number[] = [];
    const walk = (node: ResolvedNode): boolean => {
      if (node.type === 'leaf') return node.widgetId === drag!.sourceId;
      for (let i = 0; i < node.children.length; i++) {
        path.push(i);
        if (walk(node.children[i])) return true;
        path.pop();
      }
      return false;
    };
    if (!walk(hypoRoot)) return null;

    // A folded-along leaf / shrink-wrapped split takes a fixed strip, not a weight share (mirror the skin).
    const stripPx = parseFloat(getComputedStyle(shellEl).getPropertyValue('--region-stripe')) || 0;
    const cr = containerEl.getBoundingClientRect();
    let rect = { left: cr.left, top: cr.top, width: cr.width, height: cr.height };
    let node: ResolvedNode = hypoRoot;

    for (const idx of path) {
      if (node.type !== 'split') break; // defensive
      const dir = node.dir;
      const weighted = node.children.map((c) => carriesMainAxisWeight(c, dir));
      const totalWeight = node.children.reduce((s, _c, i) => s + (weighted[i] ? node.sizes[i] : 0), 0);
      const strips = weighted.filter((w) => !w).length;
      // Weighted children share the axis MINUS the fixed strips; a strip child takes exactly `stripPx`.
      const share = (dir === 'row' ? rect.width : rect.height) - strips * stripPx;
      let offset = 0;
      for (let i = 0; i < node.children.length; i++) {
        const extent = weighted[i] ? (totalWeight > 0 ? (share * node.sizes[i]) / totalWeight : 0) : stripPx;
        if (i === idx) {
          rect = dir === 'row'
            ? { left: rect.left + offset, top: rect.top, width: extent, height: rect.height }
            : { left: rect.left, top: rect.top + offset, width: rect.width, height: extent };
          break;
        }
        offset += extent;
      }
      node = node.children[idx];
    }
    return rect;
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
    backgroundMountCount: resolvedBg.length, // rendered background mounts (dropped-unknown lowers it, W-13)
    backgroundLive, // the settings switch (unbound this arc), CDP-readable even while the plate is inert
  });
</script>

<div class="region-shell" bind:this={shellEl} use:envelope={{ name: 'region-shell', id, debug }}>
  <!-- M-RP-PLATE (D1) — the grid-wide backdrop socket, FIRST child so it paints UNDER the opaque tiles
    (the message-stream `background` layer, one level up). `pointer-events:none` (skin) is the D-116 lock:
    the plate shows through every gap but is transparent to hit-testing — a clickable hole would have an
    ADDRESS and retire the tree. Wrapper look/stacking is in skin.css (N-090). -->
  {#if resolvedBg.length}
    <div class="region-backdrop" aria-hidden="true">
      {#each resolvedBg as b (b.key)}
        {@const W = b.component}
        <W {...b.props} {backgroundLive} />
      {/each}
    </div>
  {/if}

  {#if resolved.root}
    <RegionNode
      node={resolved.root}
      {widgets}
      {titles}
      {onFold}
      {onResize}
      {onMoveStart}
      {locked}
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
      <!-- M-RP7.4a — the division preview: the real half the moved region will occupy (D1). READ-ONLY
        (pointer-events:none, D3); render guard is just `previewRect` (null on no-op/hole, D4/D3 inherited). -->
      {#if previewRect}
        <div
          class="region-drop-preview"
          style="left:{previewRect.left}px;top:{previewRect.top}px;width:{previewRect.width}px;height:{previewRect.height}px"
        ></div>
      {/if}
      <div class="region-drag-ghost" style="left:{drag.x}px;top:{drag.y}px">{sourceTitle}</div>
    </div>
  {/if}
</div>
