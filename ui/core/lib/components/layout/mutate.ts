// mutate.ts — the pure, DOM-free descriptor WRITE algebra (M-RP7.2, L1). Born beside `resolve.ts`, which
// is a READ walk (descriptor → render tree, lossy — it drops). This is a WRITE (Layout × op → Layout): it
// takes a descriptor and returns a NEW descriptor the read walk can still resolve. No Svelte, no DOM, no
// I/O — the `resolve.ts` / `grouping.ts` / `Accelerator` vitest precedent.
//
// This milestone (M-RP7.3) makes it the COMPLETE algebra: `resizeSplit` (the splitter drag, M-RP7.2),
// `foldLeaf` (migrated out of the shell — already pure and identity-addressed, so a MOVE not a rewrite), and
// `move` (drag-to-dock: remove → collapse-degenerate → insert). M-RP7.4 gives them a pointer; the algebra is
// gesture-free. Every verb is pure and TOTAL (N-095): a bad input returns the tree UNCHANGED and throws.
//
// 🔒 L2 — WEIGHTS STAY INTEGERS. A float never enters the descriptor (§7: a splitter emits 0.3333…; persist
// → reload → re-normalise → drag → the layout rots by rounding). Resolution comes from an EXACT integer
// scale-up, not from rounding: a split's `sizes` are multiplied by a power of ten large enough that one unit
// is a usable drag resolution, then ONLY the dragged pair moves. Every untouched sibling keeps its
// proportion to the byte (all multiplied by the same k), and the pair's total is invariant, so nothing else
// moves. The stated cost (L2): a saved workspace reads the scaled numbers after a drag — the price of §7.
//
// 🔒 N-120 — ADDRESSING BY DESCRIPTOR INDEX, NEVER RESOLVED POSITION. `resolve.ts` DROPS, so a resolved
// child's position is not its index in the descriptor. `resizeSplit` therefore takes the pair as TWO
// EXPLICIT descriptor indices (`aIdx`, `bIdx`) that need NOT be adjacent — a dropped ghost can sit between
// the two tiles a seam paints between — and `move` addresses by leaf id (an identity). region-node reads
// each resolved node's `srcIndex` to supply them. A misaddressed resize nudges two integers; a misaddressed
// move relocates a panel into the wrong branch (J-519).

import type { FoldAxis, Layout, LayoutNode, RegionId } from './types';

const clamp = (v: number, lo: number, hi: number): number => Math.max(lo, Math.min(hi, v));

/**
 * Move weight between `children[aIdx]` and `children[bIdx]` of the split addressed by `path`, so their
 * shared boundary sits at `fraction` of THEIR combined weight (`aIdx` takes `fraction`, `bIdx` the rest).
 *
 * - `path`      child indices from the root TO THE SPLIT node (`[]` = root). DESCRIPTOR indices (N-120).
 * - `aIdx/bIdx` the two DESCRIPTOR indices whose boundary moves. They need NOT be adjacent — a dropped
 *               ghost between them is LEFT UNTOUCHED (the only behaviour that never rewrites a widget that
 *               is not there). region-node reports the two resolved neighbours' `srcIndex`.
 * - `fraction`  where the boundary sits, as a fraction of the pair's combined weight. The caller clamps in
 *               PIXELS (the min-tile clamp, N-090); this clamps defensively to ≥ 1 unit a side.
 *
 * Pure and TOTAL (N-095): bad path / non-split target / `aIdx===bIdx` / either index out of range /
 * non-finite fraction → returns the input UNCHANGED, throws NOTHING. Never mutates the input tree.
 */
export function resizeSplit(layout: Layout, path: number[], aIdx: number, bIdx: number, fraction: number): Layout {
  if (!layout || !layout.root) return layout;
  if (!Number.isFinite(fraction)) return layout;

  const target = nodeAtPath(layout.root, path);
  if (!target || target.type !== 'split') return layout; // rule 1 — bad address / non-split → unchanged
  const sizes = target.sizes;
  const n = target.children.length;
  if (!Array.isArray(sizes) || sizes.length !== n) return layout;
  if (aIdx === bIdx) return layout; // a pair of one is no pair
  if (aIdx < 0 || aIdx >= n || bIdx < 0 || bIdx >= n) return layout; // either index out of range → unchanged

  const total = sizes.reduce((s, x) => s + (Number.isFinite(x) ? x : 0), 0);
  if (!(total > 0)) return layout; // degenerate weights → unchanged

  // k = the SMALLEST power of ten such that total * k >= 1000. An already-scaled split (total >= 1000) is
  // NEVER rescaled again (k = 1). A power of ten (not ceil(1000/total)) keeps the descriptor legible — a
  // human opens the file. Integer scaling multiplies EVERY sibling by the same k, so untouched siblings keep
  // their proportion to the byte: no rounding, no drift, no float (the reason §7's no-floats lock survives).
  let k = 1;
  while (total * k < 1000) k *= 10;
  const scaled = sizes.map((x) => x * k);

  const pair = scaled[aIdx] + scaled[bIdx]; // the two need not be adjacent; whatever lies between is untouched
  if (pair < 2) return layout; // no room to give each side ≥ 1 unit → unchanged (defensive)

  let a = Math.round(pair * clamp(fraction, 0, 1));
  a = clamp(a, 1, pair - 1); // each side keeps ≥ 1 unit (covers fraction 0 and 1)
  const b = pair - a;

  const newSizes = scaled.slice();
  newSizes[aIdx] = a;
  newSizes[bIdx] = b; // pair total invariant → the split's own total is invariant → no other tile moves

  // Rebuild only the spine to the target; untouched siblings keep object identity (never mutated).
  return { ...layout, root: rebuildSizes(layout.root, path, 0, newSizes) };
}

/** Walk `path` from `root` to the addressed node; null on any bad step (non-split mid-walk, index OOR). */
function nodeAtPath(root: LayoutNode, path: number[]): LayoutNode | null {
  let node: LayoutNode = root;
  for (const idx of path) {
    if (node.type !== 'split') return null;
    if (idx < 0 || idx >= node.children.length) return null;
    node = node.children[idx];
  }
  return node;
}

/** Immutably replace the target split's `sizes`. Spine nodes are rebuilt; siblings pass through by identity.
 *  `version`, `dir`, `widgetId`, `collapsed` all survive — only the target split's `sizes` changes. */
function rebuildSizes(node: LayoutNode, path: number[], depth: number, newSizes: number[]): LayoutNode {
  if (depth === path.length) return { ...node, sizes: newSizes }; // node is the target split (validated)
  if (node.type !== 'split') return node; // defensive — the walk already validated this path
  const idx = path[depth];
  const children = node.children.map((c, j) => (j === idx ? rebuildSizes(c, path, depth + 1, newSizes) : c));
  return { ...node, children };
}

// ── foldLeaf (M-RP7.3, L2 — migrated verbatim out of the shell) ───────────────────────────────────────
// Was `setLeafCollapsed`/`handleFold` in `app_client.svelte`. Already pure, immutable, and IDENTITY-
// addressed (it recurses on `widgetId`, not on a position) — which is exactly WHY fold was drop-safe at
// J-519 while resize was not (N-120). This is a MOVE, not a redesign: the unfold contract is preserved
// EXACTLY — `collapsed === undefined` DELETES the key (never writes `collapsed: undefined`, never `false`),
// because the migrate's drop rule and the descriptor contract both depend on absent-means-expanded.

/** Set (or clear) a leaf's fold axis by its region id. `collapsed: undefined` unfolds (deletes the key).
 *  Pure, immutable, TOTAL — an unknown id / malformed tree returns the input unchanged, throws nothing. */
export function foldLeaf(layout: Layout, regionId: RegionId, collapsed: FoldAxis | undefined): Layout {
  if (!layout || !layout.root) return layout;
  return { ...layout, root: foldNode(layout.root, regionId, collapsed) };
}

function foldNode(node: LayoutNode, regionId: string, collapsed: FoldAxis | undefined): LayoutNode {
  if (node.type === 'leaf') {
    if (node.widgetId !== regionId) return node;
    if (collapsed === undefined) {
      const { collapsed: _drop, ...rest } = node; // unfold → drop the key entirely (absent ⇒ expanded)
      return rest;
    }
    return { ...node, collapsed };
  }
  if (node.type === 'split') {
    return { ...node, children: node.children.map((c) => foldNode(c, regionId, collapsed)) };
  }
  return node; // tabs — untouched (renderer A drops it anyway)
}

// ── move (M-RP7.3, L3 — drag-to-dock, gesture-free) ───────────────────────────────────────────────────
// Relocate a region so it sits on an EDGE of a target region. IDENTITY-addressed by design (D-116: a target
// tile is an ADDRESS, and a leaf is the only thing with an id) — so, like `foldLeaf`, it is drop-safe by
// construction (N-120). Three internal steps: remove (+ collapse the degenerate split it leaves) → insert
// (sibling if the target's parent already runs on the drop axis, else WRAP the target in a new split). No
// "re-normalise" pass exists (§3.4): deleting a `sizes` entry leaves the survivors' ratios already exact —
// `resolve.ts` has done the same on a sibling drop since M-RP6.1f. Pure, immutable, TOTAL.
//
// 🔒 A CYCLE IS UNSAYABLE (§3.6): you cannot drop a region into its own subtree, because a LEAF has no
// subtree. D-116's leaves-only rule pays off a second time — the whole class of cycle bugs cannot be
// expressed, so there is no guard for it. Do not write one.

/**
 * Move `sourceLeafId` to the `edge` of `targetLeafId`.
 *
 * - `edge` names an AXIS (`left`/`right` → `row`; `top`/`bottom` → `col`) AND a SIDE (`left`/`top` → before
 *   the target; `right`/`bottom` → after). The moved region takes HALF the target's space (§3.5).
 * - The source keeps its `collapsed` fold axis (§3.6 lock 2 — the axis is the user's choice, not the tree's;
 *   the SAME leaf node is re-inserted, so it rides along for free).
 *
 * Pure and TOTAL (N-095): `source === target` / a no-op reposition / unknown source or target / a `tabs`
 * node anywhere in the tree / a malformed tree → returns the input UNCHANGED, throws NOTHING.
 */
export function move(layout: Layout, sourceLeafId: RegionId, targetLeafId: RegionId, edge: 'top' | 'bottom' | 'left' | 'right'): Layout {
  if (!layout || !layout.root) return layout;
  if (sourceLeafId === targetLeafId) return layout; // §3.6 lock 1 — same leaf, nothing to do
  if (hasTabs(layout.root)) return layout; // renderer A never produces tabs; a tree that has one is out of scope

  const source = findLeaf(layout.root, sourceLeafId);
  const target = findLeaf(layout.root, targetLeafId);
  if (!source || !target) return layout; // unknown source / target → unchanged (N-095)

  const axis: 'row' | 'col' = edge === 'left' || edge === 'right' ? 'row' : 'col';
  const before = edge === 'left' || edge === 'top';

  // §3.6 lock 1 — a drop that reproduces the region's current position (already the target's sibling on that
  // side, in a split of that axis) is a no-op; do not churn the descriptor (a pointless remove + doubling).
  if (isAlreadyPositioned(layout.root, sourceLeafId, targetLeafId, axis, before)) return layout;

  const removed = removeLeaf(layout.root, sourceLeafId); // + collapse-degenerate, cascading
  if (!removed) return layout; // defensive: source was the only leaf (cannot happen — target is distinct)

  // Re-insert the SAME leaf node (source), so its `collapsed` axis rides along (§3.6 lock 2).
  return { ...layout, root: insertBeside(removed, targetLeafId, source, axis, before) };
}

/** Depth-first find of the leaf node carrying `id` (returns the node itself — `collapsed` and all). */
function findLeaf(node: LayoutNode, id: string): LayoutNode | null {
  if (node.type === 'leaf') return node.widgetId === id ? node : null;
  if (node.type === 'split') {
    for (const c of node.children) {
      const r = findLeaf(c, id);
      if (r) return r;
    }
  }
  return null;
}

/** True if the tree contains any `tabs` node. `move` bails on it — renderer A never produces one, and its
 *  placement semantics are renderer B's (D-116 keeps the door shut, not locked). */
function hasTabs(node: LayoutNode): boolean {
  if (node.type === 'tabs') return true;
  if (node.type === 'split') return node.children.some(hasTabs);
  return false;
}

/** True when `source` already sits immediately on the `before`/after side of `target` inside a split whose
 *  `dir` is the drop `axis` — i.e. the move would reproduce the current position (§3.6 lock 1). */
function isAlreadyPositioned(node: LayoutNode, sourceId: string, targetId: string, axis: 'row' | 'col', before: boolean): boolean {
  if (node.type === 'split') {
    if (node.dir === axis) {
      const ti = node.children.findIndex((c) => c.type === 'leaf' && c.widgetId === targetId);
      const si = node.children.findIndex((c) => c.type === 'leaf' && c.widgetId === sourceId);
      if (ti !== -1 && si !== -1 && (before ? si === ti - 1 : si === ti + 1)) return true;
    }
    return node.children.some((c) => isAlreadyPositioned(c, sourceId, targetId, axis, before));
  }
  return false;
}

/** Remove the leaf carrying `id`, then collapse the degenerate split it leaves behind — a split with ONE
 *  child BECOMES that child (which inherits its weight slot in the grandparent), and a split emptied
 *  entirely returns null so its own parent drops it. The recursion collapses at every level, so it cascades.
 *  The surviving `sizes` are the kept entries verbatim (already exact — no re-normalise, §3.4). */
function removeLeaf(node: LayoutNode, id: string): LayoutNode | null {
  if (node.type === 'leaf') return node.widgetId === id ? null : node;
  if (node.type === 'split') {
    const children: LayoutNode[] = [];
    const sizes: number[] = [];
    node.children.forEach((c, i) => {
      const r = removeLeaf(c, id);
      if (r !== null) {
        children.push(r);
        sizes.push(node.sizes[i]); // survivors keep their own weight; the removed entry is simply skipped
      }
    });
    if (children.length === 0) return null; // whole split emptied → parent drops it (cascade)
    if (children.length === 1) return children[0]; // one child left → the split BECOMES that child (collapse)
    return { ...node, sizes, children };
  }
  return node; // tabs — untouched (move bails on tabs before reaching here)
}

/** Insert `source` beside the leaf `targetId` on the `before`/after side of `axis`. SIBLING insert when the
 *  target's parent split already runs on `axis` (double the split's `sizes`, bisect the target's slot so the
 *  moved region takes half — §3.5); otherwise WRAP the target in a new split of `axis` (sizes `[1,1]`, taking
 *  the target's old grandparent slot). A ROOT target leaf becomes a new split of `axis` holding both. */
function insertBeside(node: LayoutNode, targetId: string, source: LayoutNode, axis: 'row' | 'col', before: boolean): LayoutNode {
  // Root target leaf: the root becomes a fresh split holding both (the grandparent-less wrap case).
  if (node.type === 'leaf' && node.widgetId === targetId) {
    return { type: 'split', dir: axis, sizes: [1, 1], children: before ? [source, node] : [node, source] };
  }
  if (node.type !== 'split') return node;

  const ti = node.children.findIndex((c) => c.type === 'leaf' && c.widgetId === targetId);
  if (ti !== -1) {
    if (node.dir === axis) {
      // Sibling insert: double, then split the target's doubled weight in half between target and source.
      const doubled = node.sizes.map((s) => s * 2); // even ⇒ exact halves, no rounding (L2)
      const sourceHalf = Math.round(doubled[ti] / 2);
      const targetHalf = doubled[ti] - sourceHalf; // sum preserved even if a stray odd weight ever appears
      const children = node.children.slice();
      const sizes = doubled.slice();
      sizes[ti] = targetHalf;
      const at = before ? ti : ti + 1;
      children.splice(at, 0, source);
      sizes.splice(at, 0, sourceHalf);
      return { ...node, sizes, children };
    }
    // Wrap insert: replace the target with a new split of `axis`; the grandparent's `sizes` are untouched.
    const wrapped: LayoutNode = {
      type: 'split', dir: axis, sizes: [1, 1],
      children: before ? [source, node.children[ti]] : [node.children[ti], source],
    };
    const children = node.children.slice();
    children[ti] = wrapped;
    return { ...node, children };
  }

  // Target is deeper — recurse.
  return { ...node, children: node.children.map((c) => insertBeside(c, targetId, source, axis, before)) };
}
