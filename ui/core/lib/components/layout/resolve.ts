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

import type { FoldAxis, Layout, LayoutNode } from './types';

/** The walked tree with unresolvable nodes removed. `tabs` never appears — renderer A drops it. */
export type ResolvedNode =
  // `collapsed` is carried through VERBATIM (M-RP7.1, D5): the walk does not interpret it, the renderer
  // does. A collapsed leaf still resolves (it is a folded tile, NOT a drop) — so `leafCount` is unchanged.
  | { type: 'leaf'; widgetId: string; collapsed?: FoldAxis } // boolean → FoldAxis (M-RP7.1b)
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

// ── migrateLayout (M-RP7.1b) ────────────────────────────────────────────────────────────────────────
// The FIRST real migrate this project has ever run: `version` was bumped twice (D-103 → 2 → this) and
// migrated NOTHING both times (§9's path was never exercised). v1/v2 stored a leaf's `collapsed` as a
// BOOLEAN, with the fold AXIS derived at render from the parent split's `dir` (M-RP7.1). v3 makes the axis
// EXPLICIT and user-owned (M-RP7.1b): a `col` split divides height → a folded child collapsed 'height'; a
// `row` split divides width → 'width'. The root has no parent split → 'height' (M-RP7.1's region-node
// defaulted the root axis to 'col'). It is the old DERIVED rule, made honest — and it must be EXERCISED,
// not asserted (N-091, applied to the one branch in this codebase that has never once run).
//
// ⚠️ DEVIATION FROM RUNBOOK §4 (flagged, not absorbed — Rule 6): the signature takes a `fallback: Layout`
// rather than the runbook's bare `migrateLayout(raw): Layout`. `resolve.ts` is `core`; the concrete
// DEFAULT_LAYOUT is SHELL-LOCAL (D2/D4 — the default tree is the client's, and core owning one would be a
// second source of truth, the exact D-067 drift the J-499 grounding killed). Core cannot know the default,
// so the shell injects it. The load-bearing property is preserved: it NEVER returns null (N-095 — recover
// to the default, never a blank centre; D-115). A shape it cannot read returns `fallback`, and it never
// throws. A v3+ layout is returned untouched (idempotent).

/** Migrate a persisted layout of any prior version to the current v3 shape. Pure, DOM-free, never throws. */
export function migrateLayout(raw: unknown, fallback: Layout): Layout {
  try {
    if (!raw || typeof raw !== 'object') return fallback;
    const l = raw as { version?: unknown; root?: unknown };
    if (typeof l.version !== 'number' || !l.root || typeof l.root !== 'object') return fallback;
    if (l.version >= 3) return raw as Layout; // already current — idempotent, untouched.
    return { version: 3, root: migrateNode(l.root as LayoutNode, 'col') }; // root: no parent → 'col' → 'height'
  } catch {
    return fallback; // unreadable shape → default, never throw (N-095).
  }
}

// `parentDir` is the enclosing split's `dir` — 'col' divides height, 'row' divides width. A root leaf uses
// 'col' → 'height', matching M-RP7.1's region-node root-axis default. Pure: builds fresh nodes, never mutates.
function migrateNode(node: LayoutNode, parentDir: 'row' | 'col'): LayoutNode {
  if (node.type === 'leaf') {
    // v1/v2 `collapsed` was a boolean. `true` ⇒ folded along the parent's dividing axis → explicit FoldAxis.
    // Anything else (false / absent) ⇒ expanded → DROP the key entirely (never write `false`, never a v2 bool).
    if ((node as { collapsed?: unknown }).collapsed === true) {
      const axis: FoldAxis = parentDir === 'row' ? 'width' : 'height';
      return { type: 'leaf', widgetId: node.widgetId, collapsed: axis };
    }
    return { type: 'leaf', widgetId: node.widgetId };
  }
  if (node.type === 'split') {
    return {
      type: 'split',
      dir: node.dir,
      sizes: node.sizes,
      children: node.children.map((c) => migrateNode(c, node.dir)),
    };
  }
  // tabs — still typed, dropped by renderer A. Recurse for completeness with the inherited parentDir.
  return { type: 'tabs', active: node.active, children: node.children.map((c) => migrateNode(c, parentDir)) };
}
