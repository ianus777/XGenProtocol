// mutate.test.ts — the pure WRITE algebra (M-RP7.2/7.3). DOM-free vitest, the `resolve.test.ts` precedent.
// M-RP7.3 makes it the COMPLETE algebra: `resizeSplit` (N-120 two-index pair), `foldLeaf` (migrated verbatim
// out of the shell), and `move` (remove → collapse-degenerate → insert). Every verb is pure and TOTAL
// (N-095: bad input → unchanged, never throws) and immutable (proven with a deep-freeze — a mutation throws
// in strict mode).

import { describe, it, expect } from 'vitest';
import { resizeSplit, foldLeaf, move, isMoveNoop } from './mutate';
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

type Split = Extract<LayoutNode, { type: 'split' }>;
type Leaf = Extract<LayoutNode, { type: 'leaf' }>;
const rootSizes = (l: Layout): number[] => (l.root as Split).sizes;
const asSplit = (n: LayoutNode): Split => n as Split;

/** Depth-first find of the leaf carrying `id` in a descriptor tree (test helper). */
function findLeaf(node: LayoutNode, id: string): Leaf | null {
  if (node.type === 'leaf') return node.widgetId === id ? node : null;
  if (node.type === 'split') {
    for (const c of node.children) { const r = findLeaf(c, id); if (r) return r; }
  }
  return null;
}
/** Count leaves in a descriptor tree (the `move` invariant: leafCount never changes). */
function leafCount(node: LayoutNode): number {
  if (node.type === 'leaf') return 1;
  if (node.type === 'split') return node.children.reduce((s, c) => s + leafCount(c), 0);
  return 0;
}

describe('resizeSplit', () => {
  // N-120: the pair is now TWO descriptor indices (aIdx, bIdx) instead of a seamIndex. The 12 cases below
  // are the M-RP7.2 cases MIGRATED (seamIndex i → the adjacent pair (i, i+1)); the non-adjacent case is new.
  it('scales the root split to total 1200 on the first resize; untouched siblings keep their proportions to the byte', () => {
    const l = resizeSplit(defaultLayout(), [], 0, 1, 0.5); // pair spaces(0)/rooms-col(1)
    const s = rootSizes(l);
    expect(s.reduce((a, b) => a + b, 0)).toBe(1200); // total 12 → k=100 → 1200
    expect(s[0]).toBe(150);
    expect(s[1]).toBe(150);
    expect(s[2]).toBe(700); // untouched siblings = their scaled selves
    expect(s[3]).toBe(200);
  });

  it('changes ONLY the dragged pair; every other entry equals its scaled self', () => {
    const l = resizeSplit(defaultLayout(), [], 2, 3, 0.25); // pair the 7-col(2)/2-col(3)
    const s = rootSizes(l);
    // scaled = [100,200,700,200]; pair (700+200=900); fraction .25 → a=225, b=675.
    expect(s[0]).toBe(100);
    expect(s[1]).toBe(200);
    expect(s[2]).toBe(225);
    expect(s[3]).toBe(675);
  });

  it('keeps the pair total invariant and the split total invariant', () => {
    const l = resizeSplit(defaultLayout(), [], 1, 2, 0.7);
    const s = rootSizes(l);
    expect(s[1] + s[2]).toBe(900); // scaled pair 200+700 → still 900
    expect(s.reduce((a, b) => a + b, 0)).toBe(1200);
  });

  it('does NOT rescale a split that is already at/above 1000 (k=1)', () => {
    const once = resizeSplit(defaultLayout(), [], 0, 1, 0.5); // → total 1200
    const twice = resizeSplit(once, [], 2, 3, 0.5); // pair 700/200
    const s = rootSizes(twice);
    expect(s.reduce((a, b) => a + b, 0)).toBe(1200); // NOT 120000 — no second scale-up
    expect(s[0]).toBe(150);
    expect(s[1]).toBe(150);
    expect(s[2]).toBe(450);
    expect(s[3]).toBe(450);
  });

  it('fraction 0 and fraction 1 each leave both sides ≥ 1 unit', () => {
    const lo = rootSizes(resizeSplit(defaultLayout(), [], 0, 1, 0));
    expect(lo[0]).toBe(1);
    expect(lo[1]).toBe(299); // pair 300 → 1 / 299
    const hi = rootSizes(resizeSplit(defaultLayout(), [], 0, 1, 1));
    expect(hi[0]).toBe(299);
    expect(hi[1]).toBe(1);
  });

  it('resizes a NESTED split addressed by a real path', () => {
    const l = resizeSplit(defaultLayout(), [1], 0, 1, 0.5); // the rooms/self col, sizes [3,1]
    const col = asSplit(asSplit(l.root).children[1]);
    // total 4 → k=1000 → [3000,1000]; pair 4000; .5 → 2000/2000.
    expect(col.sizes).toEqual([2000, 2000]);
    expect(rootSizes(l)).toEqual([1, 2, 7, 2]); // root split untouched (identity preserved)
  });

  it('resizes a NON-ADJACENT pair, leaving the entry between them byte-identical (the N-120 case)', () => {
    // The 3-child col [1,8,2]; resize the pair (0,2) — room-header and composer — over stream (index 1).
    const l = resizeSplit(defaultLayout(), [2], 0, 2, 0.5);
    const col = asSplit(asSplit(l.root).children[2]);
    // total 11 → k=100 → [100,800,200]; pair (0,2)=300; .5 → 150/150. Index 1 is UNTOUCHED.
    expect(col.sizes).toEqual([150, 800, 150]);
    expect(col.sizes[1]).toBe(800); // the between-entry: byte-identical to its scaled self
  });

  it('returns the input UNCHANGED and throws nothing on: bad path, non-split target, out-of-range/equal indices, NaN', () => {
    const l = defaultLayout();
    expect(resizeSplit(l, [9], 0, 1, 0.5)).toBe(l); // bad index mid-walk
    expect(resizeSplit(l, [0], 0, 1, 0.5)).toBe(l); // path targets a LEAF (spaces), not a split
    expect(resizeSplit(l, [], 4, 5, 0.5)).toBe(l); // both indices out of range (n=4)
    expect(resizeSplit(l, [], -1, 0, 0.5)).toBe(l); // aIdx negative
    expect(resizeSplit(l, [], 0, 4, 0.5)).toBe(l); // bIdx out of range
    expect(resizeSplit(l, [], 1, 1, 0.5)).toBe(l); // aIdx === bIdx — a pair of one
    expect(resizeSplit(l, [], 0, 1, NaN)).toBe(l); // non-finite fraction
  });

  it('does NOT mutate a deep-frozen input', () => {
    const frozen = deepFreeze(defaultLayout());
    expect(() => resizeSplit(frozen, [], 0, 1, 0.5)).not.toThrow();
    const out = resizeSplit(frozen, [], 0, 1, 0.5);
    expect(out).not.toBe(frozen);
    expect(rootSizes(frozen)).toEqual([1, 2, 7, 2]); // original untouched
  });

  it('carries collapsed and version through a resize', () => {
    const base = defaultLayout();
    asSplit(base.root).children[3] = {
      type: 'split', dir: 'col', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'members', collapsed: 'height' },
        { type: 'leaf', widgetId: 'inspector' },
      ],
    };
    const l = resizeSplit(base, [3], 0, 1, 0.5);
    const col = asSplit(asSplit(l.root).children[3]);
    expect((col.children[0] as Leaf).collapsed).toBe('height');
    expect(l.version).toBe(3);
  });

  it('resizes a col split as readily as a row split (dir passes through)', () => {
    const l = resizeSplit(defaultLayout(), [2], 1, 2, 0.5); // the [1,8,2] col
    const col = asSplit(asSplit(l.root).children[2]);
    expect(col.dir).toBe('col');
    // total 11 → k=100 → [100,800,200]; pair (800+200=1000) .5 → 500/500.
    expect(col.sizes).toEqual([100, 500, 500]);
  });
});

describe('foldLeaf', () => {
  it('sets a leaf fold axis by region id (identity-addressed)', () => {
    const l = foldLeaf(defaultLayout(), 'spaces', 'width');
    expect((asSplit(l.root).children[0] as Leaf).collapsed).toBe('width');
    expect(l.version).toBe(3);
  });

  it('sets a NESTED leaf fold axis', () => {
    const l = foldLeaf(defaultLayout(), 'stream', 'height');
    expect((asSplit(asSplit(l.root).children[2]).children[1] as Leaf).collapsed).toBe('height');
  });

  it('unfold DELETES the key (absent ⇒ expanded; never writes collapsed: undefined)', () => {
    const folded = foldLeaf(defaultLayout(), 'spaces', 'width');
    const back = foldLeaf(folded, 'spaces', undefined);
    const leaf = asSplit(back.root).children[0] as Leaf;
    expect(leaf.collapsed).toBeUndefined();
    expect('collapsed' in leaf).toBe(false); // the key is GONE, not present-and-undefined
  });

  it('returns a structurally-unchanged tree for an unknown id (behaviour-identical to the old shell code)', () => {
    expect(foldLeaf(defaultLayout(), 'ghost', 'width')).toEqual(defaultLayout());
  });

  it('does NOT mutate a deep-frozen input', () => {
    const frozen = deepFreeze(defaultLayout());
    expect(() => foldLeaf(frozen, 'spaces', 'width')).not.toThrow();
    expect((asSplit(frozen.root).children[0] as Leaf).collapsed).toBeUndefined(); // original untouched
  });
});

describe('move', () => {
  it('SIBLING insert: target parent already runs on the drop axis → bisect the target, half to the source', () => {
    // Move spaces to the BOTTOM of rooms. rooms is in the col [3,1]; edge bottom → axis col (== parent dir).
    const l = move(defaultLayout(), 'spaces', 'rooms', 'bottom');
    expect(leafCount(l.root)).toBe(8); // the move invariant
    // spaces removed from the root (weight 1 dropped); root row now [2,7,2] over three columns.
    const root = asSplit(l.root);
    expect(root.sizes).toEqual([2, 7, 2]);
    const col = asSplit(root.children[0]); // the rooms/self col, now with spaces inserted after rooms
    expect(col.children.map((c) => (c as Leaf).widgetId)).toEqual(['rooms', 'spaces', 'self']);
    // doubled [6,2]; target rooms slot 6 bisected → rooms 3 / spaces 3; self keeps its doubled 2 (§3.5).
    expect(col.sizes).toEqual([3, 3, 2]);
  });

  it('SIBLING insert on the BEFORE side puts the source ahead of the target', () => {
    const l = move(defaultLayout(), 'spaces', 'self', 'top'); // top → col, before
    const col = asSplit(asSplit(l.root).children[0]);
    expect(col.children.map((c) => (c as Leaf).widgetId)).toEqual(['rooms', 'spaces', 'self']);
    // self slot doubled (2) bisected → self 1 / spaces 1; rooms keeps doubled 6.
    expect(col.sizes).toEqual([6, 1, 1]);
  });

  it('WRAP insert: target parent runs on the OTHER axis → new split of the drop axis in the target slot', () => {
    // Move spaces to the RIGHT of rooms. rooms parent is a COL split; edge right → axis ROW (≠ col) → wrap.
    const l = move(defaultLayout(), 'spaces', 'rooms', 'right');
    expect(leafCount(l.root)).toBe(8);
    const col = asSplit(asSplit(l.root).children[0]); // the col [3,1] — its grandparent sizes untouched
    expect(col.sizes).toEqual([3, 1]); // WRAP does not touch the grandparent's weights
    const wrapped = asSplit(col.children[0]);
    expect(wrapped.dir).toBe('row');
    expect(wrapped.sizes).toEqual([1, 1]);
    expect(wrapped.children.map((c) => (c as Leaf).widgetId)).toEqual(['rooms', 'spaces']); // right → after
  });

  it('COLLAPSE-DEGENERATE: removing a leaf from a 2-child split makes the survivor inherit the split slot', () => {
    // Move self out of the rooms/self col (2 children) → the col vanishes, rooms takes its weight slot (2).
    const l = move(defaultLayout(), 'self', 'members', 'bottom');
    expect(leafCount(l.root)).toBe(8);
    const root = asSplit(l.root);
    expect(root.sizes).toEqual([1, 2, 7, 2]); // the col's slot (2) survives, now held by the bare rooms leaf
    expect(root.children[1]).toEqual({ type: 'leaf', widgetId: 'rooms' }); // the split collapsed to its child
    const membersCol = asSplit(root.children[3]);
    expect(membersCol.children.map((c) => (c as Leaf).widgetId)).toEqual(['members', 'self', 'inspector']);
  });

  it('CASCADE: the collapse recurses — a chain of degenerate splits all vanish', () => {
    // A deliberately degenerate tree (a 1-child split — a shape the algebra never PRODUCES, but must not
    // choke on: migration/hand-edit can). Removing `self` empties the inner split → its parent drops it →
    // the root is left with one child → the root itself collapses. TWO levels vanish in one move.
    const degenerate: Layout = {
      version: 3,
      root: { type: 'split', dir: 'row', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'members' },
        { type: 'split', dir: 'col', sizes: [1], children: [ { type: 'leaf', widgetId: 'self' } ] },
      ] },
    };
    const l = move(degenerate, 'self', 'members', 'bottom');
    expect(leafCount(l.root)).toBe(2);
    // remove self → inner split empty → root [members] → root collapses to members leaf; then self re-inserts
    // below members → root becomes a fresh col split of members/self.
    const root = asSplit(l.root);
    expect(root.dir).toBe('col');
    expect(root.children.map((c) => (c as Leaf).widgetId)).toEqual(['members', 'self']);
    expect(root.sizes).toEqual([1, 1]);
  });

  it('ROOT re-wrap: moving one of two root leaves collapses the root then wraps it fresh', () => {
    const twoLeaf: Layout = {
      version: 3,
      root: { type: 'split', dir: 'row', sizes: [3, 1], children: [
        { type: 'leaf', widgetId: 'spaces' },
        { type: 'leaf', widgetId: 'rooms' },
      ] },
    };
    const l = move(twoLeaf, 'spaces', 'rooms', 'bottom'); // remove spaces → root collapses to rooms leaf → wrap
    const root = asSplit(l.root);
    expect(root.dir).toBe('col');
    expect(root.sizes).toEqual([1, 1]);
    expect(root.children.map((c) => (c as Leaf).widgetId)).toEqual(['rooms', 'spaces']);
    expect(leafCount(l.root)).toBe(2);
  });

  it('a folded region KEEPS its fold axis across the move (§3.6 lock 2)', () => {
    const base = defaultLayout();
    (findLeaf(base.root, 'spaces') as Leaf).collapsed = 'width';
    const l = move(base, 'spaces', 'rooms', 'bottom');
    expect((findLeaf(l.root, 'spaces') as Leaf).collapsed).toBe('width'); // rides along even into a col split
  });

  it('NO-OP: source already sits on the target side in a split of that axis → input unchanged (identity)', () => {
    const l = defaultLayout();
    // rooms is immediately BEFORE self in a col split; move rooms to the TOP of self reproduces that.
    expect(move(l, 'rooms', 'self', 'top')).toBe(l);
    // and self immediately AFTER rooms; move self to the BOTTOM of rooms reproduces that.
    expect(move(l, 'self', 'rooms', 'bottom')).toBe(l);
  });

  it('is TOTAL: same leaf / unknown source / unknown target / a tabs node anywhere → input unchanged (identity)', () => {
    const l = defaultLayout();
    expect(move(l, 'spaces', 'spaces', 'top')).toBe(l); // source === target
    expect(move(l, 'ghost', 'rooms', 'top')).toBe(l); // unknown source
    expect(move(l, 'spaces', 'ghost', 'top')).toBe(l); // unknown target
    const withTabs: Layout = {
      version: 3,
      root: { type: 'split', dir: 'row', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'spaces' },
        { type: 'tabs', active: 0, children: [ { type: 'leaf', widgetId: 'rooms' } ] },
      ] },
    };
    expect(move(withTabs, 'spaces', 'rooms', 'top')).toBe(withTabs); // tabs present → bail
  });

  it('does NOT mutate a deep-frozen input', () => {
    const frozen = deepFreeze(defaultLayout());
    expect(() => move(frozen, 'spaces', 'rooms', 'bottom')).not.toThrow();
    const out = move(frozen, 'spaces', 'rooms', 'bottom');
    expect(out).not.toBe(frozen);
    expect(rootSizes(frozen)).toEqual([1, 2, 7, 2]); // original untouched
    expect(leafCount(frozen.root)).toBe(8);
  });
});

describe('isMoveNoop — the single predicate D4 and move() both read', () => {
  it('is TRUE exactly when move() returns the input unchanged', () => {
    const l = defaultLayout();
    // no-op cases
    expect(isMoveNoop(l, 'spaces', 'spaces', 'top')).toBe(true);
    expect(isMoveNoop(l, 'ghost', 'rooms', 'top')).toBe(true);
    expect(isMoveNoop(l, 'spaces', 'ghost', 'top')).toBe(true);
    expect(isMoveNoop(l, 'rooms', 'self', 'top')).toBe(true); // rooms already before self in a col split
    expect(isMoveNoop(l, 'self', 'rooms', 'bottom')).toBe(true); // self already after rooms
    // real moves
    expect(isMoveNoop(l, 'spaces', 'rooms', 'bottom')).toBe(false);
    expect(isMoveNoop(l, 'spaces', 'rooms', 'right')).toBe(false);
    // and move() agrees: no-op ⟺ same reference
    for (const c of [['spaces', 'spaces', 'top'], ['rooms', 'self', 'top'], ['self', 'rooms', 'bottom']] as const) {
      expect(move(l, c[0], c[1], c[2])).toBe(l);
    }
    expect(move(l, 'spaces', 'rooms', 'bottom')).not.toBe(l);
  });

  it('flags a tabs tree as a no-op (move bails on tabs)', () => {
    const withTabs: Layout = {
      version: 3,
      root: { type: 'split', dir: 'row', sizes: [1, 1], children: [
        { type: 'leaf', widgetId: 'spaces' },
        { type: 'tabs', active: 0, children: [ { type: 'leaf', widgetId: 'rooms' } ] },
      ] },
    };
    expect(isMoveNoop(withTabs, 'spaces', 'rooms', 'top')).toBe(true);
  });
});
