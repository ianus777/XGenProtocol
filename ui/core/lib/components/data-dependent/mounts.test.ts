// mounts.test.ts — pure unit suite for the extracted widget-mount resolver (M-RP6.9, D-3). No DOM,
// no Svelte — runs in the sampler vitest harness (`ui/sampler/vitest.config.js` scans
// `../core/lib/**/*.test.ts`).
//
// The rule had THREE identical copies and ZERO tests before this milestone (§1.4). What this suite
// locks: drop-unknown (W-13) · the legacy key reproduced EXACTLY by the fallback · the declared
// index surviving a drop · `mountKey` giving stable keys across a runtime removal — asserted
// against its own negative control, because "the keys are stable" means nothing unless the same
// probe is shown moving when the stabiliser is absent (§1.3, the defect this field exists to fix).

import { describe, it, expect } from 'vitest';
import type { Component } from 'svelte';
import { resolveMounts } from './mounts';
import type { WidgetMount } from './types';

// The resolver never renders, so a marker object is a sufficient stand-in for a Svelte component.
const A = { $$: 'A' } as unknown as Component;
const B = { $$: 'B' } as unknown as Component;
const C = { $$: 'C' } as unknown as Component;
const REG: Record<string, Component> = { 'w.a': A, 'w.b': B, 'w.c': C };

const m = (widgetId: string, extra: Partial<WidgetMount> = {}): WidgetMount => ({ widgetId, ...extra });

describe('resolveMounts — absent / empty', () => {
  it('returns [] for an absent list', () => {
    expect(resolveMounts(undefined, REG)).toEqual([]);
  });

  it('returns [] for an empty list', () => {
    expect(resolveMounts([], REG)).toEqual([]);
  });

  it('returns [] when nothing resolves', () => {
    expect(resolveMounts([m('nope.1'), m('nope.2')], REG)).toEqual([]);
  });
});

describe('resolveMounts — drop-unknown (W-13)', () => {
  it('drops an unresolvable widgetId and keeps the rest', () => {
    const out = resolveMounts([m('w.a'), m('does.not.exist'), m('w.b')], REG);
    expect(out.map((r) => r.component)).toEqual([A, B]);
  });

  it('drops against an EMPTY registry without throwing', () => {
    expect(resolveMounts([m('w.a')], {})).toEqual([]);
  });

  // The index is taken BEFORE the filter, so a dropped mount does not renumber its survivors.
  // This is today's behaviour and the migration must preserve it byte-for-byte.
  it('does NOT renumber survivors when a middle mount drops', () => {
    const out = resolveMounts([m('w.a'), m('does.not.exist'), m('w.b')], REG);
    expect(out.map((r) => r.key)).toEqual(['w.a-0', 'w.b-2']);
  });
});

describe('resolveMounts — key', () => {
  // The load-bearing compatibility assertion: the fallback IS the legacy expression.
  it('falls back to the legacy `${widgetId}-${i}` key exactly', () => {
    const mounts = [m('w.a'), m('w.b'), m('w.c')];
    const legacy = mounts.map((x, i) => `${x.widgetId}-${i}`);
    expect(resolveMounts(mounts, REG).map((r) => r.key)).toEqual(legacy);
  });

  it('honours an explicit mountKey', () => {
    const out = resolveMounts([m('w.a', { mountKey: 'react-thumbsup' })], REG);
    expect(out[0].key).toBe('react-thumbsup');
  });

  it('mixes: a mount with mountKey and one without, in the same list', () => {
    const out = resolveMounts([m('w.a', { mountKey: 'pinned' }), m('w.b')], REG);
    expect(out.map((r) => r.key)).toEqual(['pinned', 'w.b-1']);
  });

  it('gives duplicate widgetIds distinct keys', () => {
    const out = resolveMounts([m('w.a'), m('w.a'), m('w.a')], REG);
    expect(out.map((r) => r.key)).toEqual(['w.a-0', 'w.a-1', 'w.a-2']);
    expect(new Set(out.map((r) => r.key)).size).toBe(3);
  });
});

describe('resolveMounts — runtime removal (§1.3, the defect mountKey exists to fix)', () => {
  const keyed = [
    m('w.a', { mountKey: 'k-a' }),
    m('w.b', { mountKey: 'k-b' }),
    m('w.c', { mountKey: 'k-c' }),
  ];

  it('keeps the survivors keys unchanged when the FIRST mount is removed', () => {
    const before = resolveMounts(keyed, REG);
    const after = resolveMounts(keyed.slice(1), REG);
    expect(before.slice(1).map((r) => r.key)).toEqual(after.map((r) => r.key));
    expect(after.map((r) => r.key)).toEqual(['k-b', 'k-c']);
  });

  // NEGATIVE CONTROL. Without `mountKey` the same removal DOES churn every survivor — which is
  // what makes the assertion above a measurement rather than a tautology. If this test ever goes
  // green in the stable direction, the one above has stopped proving anything.
  it('WITHOUT mountKey the same removal churns every survivor key', () => {
    const unkeyed = [m('w.a'), m('w.b'), m('w.c')];
    const before = resolveMounts(unkeyed, REG);
    const after = resolveMounts(unkeyed.slice(1), REG);
    expect(before.slice(1).map((r) => r.key)).toEqual(['w.b-1', 'w.c-2']);
    expect(after.map((r) => r.key)).toEqual(['w.b-0', 'w.c-1']);
    expect(before.slice(1).map((r) => r.key)).not.toEqual(after.map((r) => r.key));
  });

  it('keeps the ids stable too, since the id derives from the key', () => {
    const before = resolveMounts(keyed, REG, 'msg__x-');
    const after = resolveMounts(keyed.slice(1), REG, 'msg__x-');
    expect(after.map((r) => r.id)).toEqual(['msg__x-k-b', 'msg__x-k-c']);
    expect(before.slice(1).map((r) => r.id)).toEqual(after.map((r) => r.id));
  });
});

describe('resolveMounts — id', () => {
  it('leaves id undefined when no idPrefix is passed', () => {
    const out = resolveMounts([m('w.a')], REG);
    expect(out[0].id).toBeUndefined();
  });

  it('composes idPrefix + key when a prefix is passed', () => {
    const out = resolveMounts([m('w.a'), m('w.b', { mountKey: 'kb' })], REG, 'm-1__x-');
    expect(out.map((r) => r.id)).toEqual(['m-1__x-w.a-0', 'm-1__x-kb']);
  });

  it('never invents a namespace of its own', () => {
    const out = resolveMounts([m('w.a')], REG, '');
    expect(out[0].id).toBeUndefined(); // empty prefix is falsy → no id, not a bare key
  });
});

describe('resolveMounts — props', () => {
  it('defaults absent props to an empty object', () => {
    expect(resolveMounts([m('w.a')], REG)[0].props).toEqual({});
  });

  it('passes declared props through untouched', () => {
    const props = { label: '12:05', tone: 'muted' };
    expect(resolveMounts([m('w.a', { props })], REG)[0].props).toEqual(props);
  });
});
