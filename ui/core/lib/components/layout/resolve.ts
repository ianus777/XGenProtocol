// resolve.ts — the pure, DOM-free descriptor→layout walk. Renderer A (region-shell) reads the resolved
// tree; the same walk is unit-tested without an app (the `stream/grouping.ts` / `Accelerator` precedent,
// vitest). No Svelte, no DOM, no I/O — a plain function of (descriptor, known-widget-ids).
//
// It reuses the shipped leaf-resolution shape (§0.1): a widgetId is resolved against a known-id set, and a
// leaf whose id is unknown is DROPPED (the `message.svelte` W-13 reconcile). The resolved tree is the
// RENDER truth — `leafIds` is what actually mounts, so a drop is CDP-provable via the getter (§4).
//
// Rules (all pure, NONE throw — a stale descriptor must DEGRADE, never crash: region-dock §9):
//   1. leaf with an id in `knownIds`            → kept (recorded in `leafIds`, document order).
//   2. leaf with an unknown id                  → dropped (recorded in `dropped`).
//   3. tabs                                     → dropped, `unsupported++`, one DEV warn (renderer A gap).
//   4. split whose children ALL drop            → the split itself drops (an empty box is noise).
//   5. sizes.length !== children.length         → equal-weight fallback + DEV warn (never throw).
//   6. depth is unbounded; the walk is recursive.

import type { Layout, LayoutNode } from './types';

/** The walked tree with unresolvable nodes removed. `tabs` never appears — renderer A drops it. */
export type ResolvedNode =
  // `collapsed` is carried through VERBATIM (M-RP7.1, D5): the walk does not interpret it, the renderer
  // does. A collapsed leaf still resolves (it is a folded tile, NOT a drop) — so `leafCount` is unchanged.
  | { type: 'leaf'; widgetId: string; collapsed?: boolean }
  | { type: 'split'; dir: 'row' | 'col'; sizes: number[]; children: ResolvedNode[] };

export interface ResolveResult {
  /** The resolved tree, or `null` when everything dropped / the root was absent. */
  root: ResolvedNode | null;
  /** Resolved (rendered) leaf widgetIds, in document order. */
  leafIds: string[];
  /** widgetIds dropped because the registry did not know them. */
  dropped: string[];
  /** `tabs` nodes dropped by renderer A (unsupported this milestone). */
  unsupported: number;
}

// DEV-warn helper — guarded so a production build (and a warn-free assertion) stays quiet. `import.meta`
// carries `env` under Vite AND Vitest; optional-chained so a bare-node import can never throw here.
function devWarn(msg: string): void {
  if (import.meta.env?.DEV) console.warn(`[layout] ${msg}`);
}

export function resolveLayout(layout: Layout | null | undefined, knownIds: Set<string>): ResolveResult {
  const dropped: string[] = [];
  const leafIds: string[] = [];
  let unsupported = 0;

  function walk(node: LayoutNode): ResolvedNode | null {
    if (node.type === 'leaf') {
      if (knownIds.has(node.widgetId)) {
        leafIds.push(node.widgetId);
        // Carry `collapsed` through verbatim (D5) — a folded leaf is still a resolved, counted leaf.
        return { type: 'leaf', widgetId: node.widgetId, collapsed: node.collapsed };
      }
      dropped.push(node.widgetId); // rule 2
      return null;
    }

    if (node.type === 'tabs') {
      unsupported++; // rule 3
      devWarn(`tabs node unsupported by renderer A — dropped (renders at M-RP7)`);
      return null;
    }

    // split (rule 4 + rule 5)
    const children = node.children ?? [];
    const sizesValid = Array.isArray(node.sizes) && node.sizes.length === children.length;
    if (!sizesValid) devWarn(`split sizes (${node.sizes?.length ?? 0}) != children (${children.length}) — equal-weight fallback`);

    const resolvedChildren: ResolvedNode[] = [];
    const keptSizes: number[] = [];
    children.forEach((child, i) => {
      const r = walk(child);
      if (r) {
        resolvedChildren.push(r);
        // Kept children retain their original weight; a dropped sibling's weight is simply skipped, so the
        // survivors keep their relative proportions. Invalid sizes → equal weight (1 each).
        keptSizes.push(sizesValid ? node.sizes[i] : 1);
      }
    });

    if (resolvedChildren.length === 0) return null; // rule 4 — every child dropped, so the box drops too
    return { type: 'split', dir: node.dir, sizes: keptSizes, children: resolvedChildren };
  }

  const root = layout?.root ? walk(layout.root) : null; // empty / null descriptor survives (never throws)
  return { root, leafIds, dropped, unsupported };
}

/** Max depth of a resolved tree (null=0, leaf=1, split=1+max child). Feeds the getter's `depth` (§4). */
export function treeDepth(node: ResolvedNode | null): number {
  if (!node) return 0;
  if (node.type === 'leaf') return 1;
  return 1 + node.children.reduce((m, c) => Math.max(m, treeDepth(c)), 0);
}
