// resolve.test.ts — the pure descriptor→layout walk (M-RP6.1f, §3). DOM-free, so the whole walk is
// unit-testable without an app (the `grouping.ts` / `Accelerator` vitest precedent). Six minimum cases
// per the runbook §3: default resolves 8 leaves / 0 dropped / 0 unsupported · unknown id drops ·
// all-unknown split collapses · tabs drops + counts · sizes-mismatch degrades · empty/null root survives.

import { describe, it, expect } from 'vitest';
import { resolveLayout, treeDepth, migrateLayout } from './resolve';
import type { Layout } from './types';

const ALL_IDS = new Set(['spaces', 'rooms', 'self', 'room-header', 'stream', 'composer', 'members', 'inspector']);

// Mirrors the shipped DEFAULT_LAYOUT (D8) — row of four columns, all 8 regions, no unknown id, no tabs.
// version: 3 (M-RP7.1b) — the leaf `collapsed` FoldAxis schema bump; the walk ignores version, so this is a
// faithful-mirror cosmetic (the other inline fixtures below stay v1 — arbitrary trees, version-irrelevant).
const DEFAULT_LAYOUT: Layout = {
  version: 3,
  root: {
    type: 'split', dir: 'row', sizes: [1, 2, 7, 2], children: [
      { type: 'leaf', widgetId: 'spaces' },
      { type: 'split', dir: 'col', sizes: [3, 1], children: [
        { type: 'leaf', widgetId: 'rooms' },
        { type: 'leaf', widgetId: 'self' },
      ] },
      { type: 'split', dir: 'col', sizes: [1, 8, 2], children: [
        { type: 'leaf', widgetId: 'room-header' },
        { type: 'leaf', widgetId: 'stream' },
        { type: 'leaf', widgetId: 'composer' },
      ] },
      { type: 'split', dir: 'col', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'members' },
        { type: 'leaf', widgetId: 'inspector' },
      ] },
    ],
  },
};

describe('resolveLayout', () => {
  it('resolves the default layout: 8 leaves, 0 dropped, 0 unsupported, document order', () => {
    const r = resolveLayout(DEFAULT_LAYOUT, ALL_IDS);
    expect(r.leafIds).toEqual(['spaces', 'rooms', 'self', 'room-header', 'stream', 'composer', 'members', 'inspector']);
    expect(r.dropped).toEqual([]);
    expect(r.unsupported).toBe(0);
    expect(r.root).not.toBeNull();
    expect(treeDepth(r.root)).toBe(3); // shell-split → col-split → leaf
  });

  it('drops a leaf with an unknown widgetId, keeping the rest', () => {
    const layout: Layout = {
      version: 1,
      root: { type: 'split', dir: 'row', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'spaces' },
        { type: 'leaf', widgetId: 'ghost' },
      ] },
    };
    const r = resolveLayout(layout, ALL_IDS);
    expect(r.leafIds).toEqual(['spaces']);
    expect(r.dropped).toEqual(['ghost']);
    // Surviving split keeps only the resolved child + its weight.
    expect(r.root).toEqual({ type: 'split', dir: 'row', sizes: [1], children: [{ type: 'leaf', widgetId: 'spaces' }] });
  });

  it('collapses a split whose children all drop (an empty box is noise)', () => {
    const layout: Layout = {
      version: 1,
      root: { type: 'split', dir: 'col', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'ghost-a' },
        { type: 'leaf', widgetId: 'ghost-b' },
      ] },
    };
    const r = resolveLayout(layout, ALL_IDS);
    expect(r.root).toBeNull();
    expect(r.leafIds).toEqual([]);
    expect(r.dropped).toEqual(['ghost-a', 'ghost-b']);
  });

  it('drops a tabs node and counts it as unsupported (never throws)', () => {
    const layout: Layout = {
      version: 1,
      root: { type: 'split', dir: 'row', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'spaces' },
        { type: 'tabs', active: 0, children: [
          { type: 'leaf', widgetId: 'rooms' },
          { type: 'leaf', widgetId: 'self' },
        ] },
      ] },
    };
    const r = resolveLayout(layout, ALL_IDS);
    expect(r.unsupported).toBe(1);
    expect(r.leafIds).toEqual(['spaces']); // the tabs subtree is not walked
    expect(r.root).toEqual({ type: 'split', dir: 'row', sizes: [1], children: [{ type: 'leaf', widgetId: 'spaces' }] });
  });

  it('degrades a sizes/children mismatch to equal weights (never throws)', () => {
    const layout: Layout = {
      version: 1,
      root: { type: 'split', dir: 'row', sizes: [5], children: [ // 1 size, 2 children
        { type: 'leaf', widgetId: 'spaces' },
        { type: 'leaf', widgetId: 'rooms' },
      ] },
    };
    const r = resolveLayout(layout, ALL_IDS);
    expect(r.leafIds).toEqual(['spaces', 'rooms']);
    expect(r.root).toEqual({ type: 'split', dir: 'row', sizes: [1, 1], children: [
      { type: 'leaf', widgetId: 'spaces' },
      { type: 'leaf', widgetId: 'rooms' },
    ] });
  });

  it('survives an empty / null root (never throws)', () => {
    expect(resolveLayout(null, ALL_IDS)).toEqual({ root: null, leafIds: [], dropped: [], unsupported: 0 });
    const unknownRoot: Layout = { version: 1, root: { type: 'leaf', widgetId: 'ghost' } };
    const r = resolveLayout(unknownRoot, ALL_IDS);
    expect(r.root).toBeNull();
    expect(r.dropped).toEqual(['ghost']);
  });

  // M-RP7.1/7.1b — `collapsed` rides through the walk verbatim; the renderer, not the walk, interprets it.
  it('carries a collapsed leaf through the walk verbatim (folding is not dropping)', () => {
    const layout: Layout = {
      version: 3,
      root: { type: 'split', dir: 'col', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'rooms', collapsed: 'height' },
        { type: 'leaf', widgetId: 'self' },
      ] },
    };
    const r = resolveLayout(layout, ALL_IDS);
    // A collapsed leaf is still a resolved, counted leaf — leafCount is unchanged (§4 getter G).
    expect(r.leafIds).toEqual(['rooms', 'self']);
    expect(r.dropped).toEqual([]);
    expect(r.root).toEqual({ type: 'split', dir: 'col', sizes: [1, 1], children: [
      { type: 'leaf', widgetId: 'rooms', collapsed: 'height' },
      { type: 'leaf', widgetId: 'self', collapsed: undefined }, // absent key ⇒ undefined
    ] });
  });

  it('leaves collapsed undefined when the descriptor omits it (absent ⇒ expanded)', () => {
    const r = resolveLayout(DEFAULT_LAYOUT, ALL_IDS);
    function everyLeaf(n: typeof r.root, f: (leaf: { widgetId: string; collapsed?: string }) => void): void {
      if (!n) return;
      if (n.type === 'leaf') { f(n); return; }
      n.children.forEach((c) => everyLeaf(c, f));
    }
    everyLeaf(r.root, (leaf) => expect(leaf.collapsed).toBeUndefined());
  });
});

// M-RP7.1b — migrateLayout is the FIRST exercised migrate this project has run. `collapsed` was a boolean
// (axis derived from the parent split); v3 makes it an explicit FoldAxis. Fed, not asserted (N-091) —
// hand-built v2 trees under BOTH parent kinds, plus the root, the drop-false case, idempotency, and garbage.
describe('migrateLayout (v2 → v3)', () => {
  // A sentinel fallback (core cannot know the shell's DEFAULT_LAYOUT — it is injected, D2/D4).
  const FALLBACK: Layout = { version: 3, root: { type: 'leaf', widgetId: 'spaces' } };

  it("upgrades a collapsed:true leaf under a COL parent to 'height' (col divides height → along)", () => {
    const v2 = { version: 2, root: { type: 'split', dir: 'col', sizes: [1, 1], children: [
      { type: 'leaf', widgetId: 'rooms', collapsed: true },
      { type: 'leaf', widgetId: 'self' },
    ] } };
    const m = migrateLayout(v2, FALLBACK);
    expect(m.version).toBe(3);
    expect(m.root).toEqual({ type: 'split', dir: 'col', sizes: [1, 1], children: [
      { type: 'leaf', widgetId: 'rooms', collapsed: 'height' },
      { type: 'leaf', widgetId: 'self' },
    ] });
  });

  it("upgrades a collapsed:true leaf under a ROW parent to 'width' (row divides width)", () => {
    const v2 = { version: 2, root: { type: 'split', dir: 'row', sizes: [1, 1], children: [
      { type: 'leaf', widgetId: 'spaces', collapsed: true },
      { type: 'leaf', widgetId: 'rooms' },
    ] } };
    const m = migrateLayout(v2, FALLBACK);
    expect(m.root).toEqual({ type: 'split', dir: 'row', sizes: [1, 1], children: [
      { type: 'leaf', widgetId: 'spaces', collapsed: 'width' },
      { type: 'leaf', widgetId: 'rooms' },
    ] });
  });

  it("upgrades a collapsed:true leaf at the ROOT to 'height' (no parent → col default)", () => {
    const v2 = { version: 2, root: { type: 'leaf', widgetId: 'spaces', collapsed: true } };
    const m = migrateLayout(v2, FALLBACK);
    expect(m.root).toEqual({ type: 'leaf', widgetId: 'spaces', collapsed: 'height' });
  });

  it('drops a collapsed:false key (absent ⇒ expanded; never writes false)', () => {
    const v2 = { version: 1, root: { type: 'leaf', widgetId: 'spaces', collapsed: false } };
    const m = migrateLayout(v2, FALLBACK);
    expect(m.root).toEqual({ type: 'leaf', widgetId: 'spaces' });
    expect('collapsed' in (m.root as object)).toBe(false);
  });

  it('returns a v3 layout untouched (idempotent — same reference)', () => {
    const v3: Layout = { version: 3, root: { type: 'split', dir: 'col', sizes: [1, 1], children: [
      { type: 'leaf', widgetId: 'rooms', collapsed: 'width' },
      { type: 'leaf', widgetId: 'self' },
    ] } };
    expect(migrateLayout(v3, FALLBACK)).toBe(v3);
  });

  it('falls back to the injected default on garbage (never throws)', () => {
    expect(migrateLayout(null, FALLBACK)).toBe(FALLBACK);
    expect(migrateLayout(42, FALLBACK)).toBe(FALLBACK);
    expect(migrateLayout({ nope: true }, FALLBACK)).toBe(FALLBACK);
    expect(migrateLayout({ version: 2 }, FALLBACK)).toBe(FALLBACK); // version but no root
  });
});
