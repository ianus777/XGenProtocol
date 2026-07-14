// mutate.ts — the pure, DOM-free descriptor WRITE algebra (M-RP7.2, L1). Born beside `resolve.ts`, which
// is a READ walk (descriptor → render tree, lossy — it drops). This is a WRITE (Layout × op → Layout): it
// takes a descriptor and returns a NEW descriptor the read walk can still resolve. No Svelte, no DOM, no
// I/O — the `resolve.ts` / `grouping.ts` / `Accelerator` vitest precedent.
//
// This milestone ships ONE verb: `resizeSplit` (the splitter drag). M-RP7.3 adds `move` (drag-to-dock) and
// pulls `fold` out of the shell into here (L1 — the first mutation IS algebra, so the module is born now).
//
// 🔒 L2 — WEIGHTS STAY INTEGERS. A float never enters the descriptor (§7: a splitter emits 0.3333…; persist
// → reload → re-normalise → drag → the layout rots by rounding). Resolution comes from an EXACT integer
// scale-up, not from rounding: a split's `sizes` are multiplied by a power of ten large enough that one unit
// is a usable drag resolution, then ONLY the dragged pair moves. Every untouched sibling keeps its
// proportion to the byte (all multiplied by the same k), and the pair's total is invariant, so nothing else
// moves. The stated cost (L2): a saved workspace reads the scaled numbers after a drag — the price of §7.

import type { Layout, LayoutNode } from './types';

const clamp = (v: number, lo: number, hi: number): number => Math.max(lo, Math.min(hi, v));

/**
 * Resize the seam between `children[seamIndex]` and `children[seamIndex+1]` of the split addressed by
 * `path`, so their shared boundary now sits at `fraction` of the pair's combined weight.
 *
 * - `path`     child indices from the root TO THE SPLIT node (`[]` = root). Derived, no schema change (L5).
 * - `seamIndex` the seam between `children[i]` and `children[i+1]`.
 * - `fraction`  where the pair's boundary sits, as a fraction of the pair's combined weight. The caller
 *               clamps in PIXELS (the min-tile clamp, N-090); this clamps defensively to ≥ 1 unit a side.
 *
 * Pure and TOTAL (N-095's temperament): a bad path / non-split target / out-of-range seam / non-finite
 * fraction returns the input UNCHANGED and throws NOTHING. Never mutates the input tree.
 */
export function resizeSplit(layout: Layout, path: number[], seamIndex: number, fraction: number): Layout {
  if (!layout || !layout.root) return layout;
  if (!Number.isFinite(fraction)) return layout;

  const target = nodeAtPath(layout.root, path);
  if (!target || target.type !== 'split') return layout; // rule 1 — bad address / non-split → unchanged
  const sizes = target.sizes;
  const n = target.children.length;
  if (!Array.isArray(sizes) || sizes.length !== n) return layout;
  if (seamIndex < 0 || seamIndex >= n - 1) return layout; // out-of-range seam → unchanged

  const total = sizes.reduce((s, x) => s + (Number.isFinite(x) ? x : 0), 0);
  if (!(total > 0)) return layout; // degenerate weights → unchanged

  // k = the SMALLEST power of ten such that total * k >= 1000. An already-scaled split (total >= 1000) is
  // NEVER rescaled again (k = 1). A power of ten (not ceil(1000/total)) keeps the descriptor legible — a
  // human opens the file. Integer scaling multiplies EVERY sibling by the same k, so untouched siblings keep
  // their proportion to the byte: no rounding, no drift, no float (the reason §7's no-floats lock survives).
  let k = 1;
  while (total * k < 1000) k *= 10;
  const scaled = sizes.map((x) => x * k);

  const i = seamIndex;
  const pair = scaled[i] + scaled[i + 1];
  if (pair < 2) return layout; // no room to give each side ≥ 1 unit → unchanged (defensive)

  let a = Math.round(pair * clamp(fraction, 0, 1));
  a = clamp(a, 1, pair - 1); // each side keeps ≥ 1 unit (covers fraction 0 and 1)
  const b = pair - a;

  const newSizes = scaled.slice();
  newSizes[i] = a;
  newSizes[i + 1] = b; // pair total invariant → the split's own total is invariant → no other tile moves

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
