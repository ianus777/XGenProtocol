// resolve.test.ts — the pure descriptor→layout walk (M-RP6.1f, §3). DOM-free, so the whole walk is
// unit-testable without an app (the `grouping.ts` / `Accelerator` vitest precedent). Six minimum cases
// per the runbook §3: default resolves 8 leaves / 0 dropped / 0 unsupported · unknown id drops ·
// all-unknown split collapses · tabs drops + counts · sizes-mismatch degrades · empty/null root survives.

import { describe, it, expect } from 'vitest';
import { resolveLayout, treeDepth } from './resolve';
import type { Layout } from './types';

const ALL_IDS = new Set(['spaces', 'rooms', 'self', 'room-header', 'stream', 'composer', 'members', 'inspector']);

// Mirrors the shipped DEFAULT_LAYOUT (D8) — row of four columns, all 8 regions, no unknown id, no tabs.
const DEFAULT_LAYOUT: Layout = {
  version: 1,
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
});
