// mutate.test.ts — the pure WRITE algebra (M-RP7.2, L1). DOM-free vitest, the `resolve.test.ts` precedent.
// resizeSplit is the only export; the cases cover the integer scale-up (L2), the pair-only invariant, the
// no-rescale-when-already-scaled property, the clamp, the total temperament (bad input → unchanged, never
// throws — N-095), row/col, and passthrough of collapsed/version. Input immutability is proven with a
// deep-freeze (a mutation would throw in strict mode).

import { describe, it, expect } from 'vitest';
import { resizeSplit } from './mutate';
import type { Layout, LayoutNode } from './types';

// The shipped DEFAULT_LAYOUT shape (row of four columns). version 3 (M-RP7.1b).
function defaultLayout(): Layout {
  return {
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
}

function deepFreeze<T>(o: T): T {
  if (o && typeof o === 'object') {
    Object.values(o as Record<string, unknown>).forEach(deepFreeze);
    Object.freeze(o);
  }
  return o;
}

const rootSizes = (l: Layout): number[] => (l.root as Extract<LayoutNode, { type: 'split' }>).sizes;

describe('resizeSplit', () => {
  it('scales the root split to total 1200 on the first resize; untouched siblings keep their proportions to the byte', () => {
    const l = resizeSplit(defaultLayout(), [], 0, 0.5); // seam between spaces(1) and the rooms/self column(2)
    const s = rootSizes(l);
    expect(s.reduce((a, b) => a + b, 0)).toBe(1200); // total 12 → k=100 → 1200
    // pair was [100,200] (300); fraction 0.5 → a=150, b=150.
    expect(s[0]).toBe(150);
    expect(s[1]).toBe(150);
    // the two untouched siblings are exactly their scaled selves (700, 200).
    expect(s[2]).toBe(700);
    expect(s[3]).toBe(200);
  });

  it('changes ONLY the dragged pair; every other entry equals its scaled self', () => {
    const l = resizeSplit(defaultLayout(), [], 2, 0.25); // seam between the 7-col and the 2-col
    const s = rootSizes(l);
    // scaled = [100,200,700,200]; pair (700+200=900); fraction .25 → a=225, b=675.
    expect(s[0]).toBe(100);
    expect(s[1]).toBe(200);
    expect(s[2]).toBe(225);
    expect(s[3]).toBe(675);
  });

  it('keeps the pair total invariant and the split total invariant', () => {
    const before = defaultLayout();
    const l = resizeSplit(before, [], 1, 0.7);
    const s = rootSizes(l);
    // scaled pair (200+700=900) → still sums 900 after; whole split still 1200.
    expect(s[1] + s[2]).toBe(900);
    expect(s.reduce((a, b) => a + b, 0)).toBe(1200);
  });

  it('does NOT rescale a split that is already at/above 1000 (k=1)', () => {
    const once = resizeSplit(defaultLayout(), [], 0, 0.5); // → total 1200
    const twice = resizeSplit(once, [], 2, 0.5); // seam between 700 and 200
    const s = rootSizes(twice);
    expect(s.reduce((a, b) => a + b, 0)).toBe(1200); // NOT 120000 — no second scale-up
    // pair (700+200=900) split 50/50 → 450/450; the [150,150] pair from the first drag is untouched.
    expect(s[0]).toBe(150);
    expect(s[1]).toBe(150);
    expect(s[2]).toBe(450);
    expect(s[3]).toBe(450);
  });

  it('fraction 0 and fraction 1 each leave both sides ≥ 1 unit', () => {
    const lo = rootSizes(resizeSplit(defaultLayout(), [], 0, 0));
    expect(lo[0]).toBe(1);
    expect(lo[1]).toBe(299); // pair 300 → 1 / 299
    const hi = rootSizes(resizeSplit(defaultLayout(), [], 0, 1));
    expect(hi[0]).toBe(299);
    expect(hi[1]).toBe(1);
  });

  it('resizes a NESTED split addressed by a real path', () => {
    const l = resizeSplit(defaultLayout(), [1], 0, 0.5); // the rooms/self col, sizes [3,1]
    const col = (l.root as Extract<LayoutNode, { type: 'split' }>).children[1] as Extract<LayoutNode, { type: 'split' }>;
    // total 4 → k=1000 (smallest power of ten with 4·k ≥ 1000) → [3000,1000]; pair 4000; .5 → 2000/2000.
    expect(col.sizes).toEqual([2000, 2000]);
    // the root split is untouched (identity preserved) — same object as the input's root child.
    expect(rootSizes(l)).toEqual([1, 2, 7, 2]);
  });

  it('returns the input UNCHANGED and throws nothing on: bad path, non-split target, out-of-range seam', () => {
    const l = defaultLayout();
    expect(resizeSplit(l, [9], 0, 0.5)).toBe(l); // bad index mid-walk
    expect(resizeSplit(l, [0], 0, 0.5)).toBe(l); // path targets a LEAF (spaces), not a split
    expect(resizeSplit(l, [], 9, 0.5)).toBe(l); // seamIndex out of range
    expect(resizeSplit(l, [], -1, 0.5)).toBe(l);
    expect(resizeSplit(l, [], 3, 0.5)).toBe(l); // seam 3 needs child 4 in a 4-child split → OOR
    expect(resizeSplit(l, [], 0, NaN)).toBe(l); // non-finite fraction
  });

  it('does NOT mutate a deep-frozen input', () => {
    const frozen = deepFreeze(defaultLayout());
    expect(() => resizeSplit(frozen, [], 0, 0.5)).not.toThrow();
    const out = resizeSplit(frozen, [], 0, 0.5);
    expect(out).not.toBe(frozen);
    expect(rootSizes(frozen)).toEqual([1, 2, 7, 2]); // original untouched
  });

  it('carries collapsed and version through a resize', () => {
    const base = defaultLayout();
    // fold a leaf inside the members/inspector col (path [3]) so a collapsed sibling rides the resize.
    (base.root as Extract<LayoutNode, { type: 'split' }>).children[3] = {
      type: 'split', dir: 'col', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'members', collapsed: 'height' },
        { type: 'leaf', widgetId: 'inspector' },
      ],
    };
    const l = resizeSplit(base, [3], 0, 0.5);
    const col = (l.root as Extract<LayoutNode, { type: 'split' }>).children[3] as Extract<LayoutNode, { type: 'split' }>;
    expect((col.children[0] as Extract<LayoutNode, { type: 'leaf' }>).collapsed).toBe('height');
    expect(l.version).toBe(3);
  });

  it('resizes a col split as readily as a row split (dir passes through)', () => {
    const l = resizeSplit(defaultLayout(), [2], 1, 0.5); // the [1,8,2] col
    const col = (l.root as Extract<LayoutNode, { type: 'split' }>).children[2] as Extract<LayoutNode, { type: 'split' }>;
    expect(col.dir).toBe('col');
    // total 11 → k=100 → [100,800,200]; pair (800+200=1000) .5 → 500/500.
    expect(col.sizes).toEqual([100, 500, 500]);
  });
});
