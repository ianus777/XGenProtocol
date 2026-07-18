// grouping.test.ts — pure unit suite for the message-stream sibling-relationship maths (M-RP5.6 A
// grouping/dividers + M-RP6.3 Leg C1 status placement). No DOM, no Svelte — runs in the sampler
// vitest harness (`ui/sampler/vitest.config.js` scans `../core/lib/**/*.test.ts`).
//
// The C1 additions this suite locks: status rows placed by timestamp in the single walk · a status
// row breaks a grouping run · absent/empty status ⇒ output byte-identical to the message-only walk ·
// `formatDayDivider` unchanged · the status row key derives ONLY from the episode id (the V6 trap,
// asserted at the pure level too).

import { describe, it, expect } from 'vitest';
import { computeRows, formatDayDivider, GROUP_WINDOW_MS, type StreamStatus } from './grouping';
import type { MessageDescriptor } from '../types';

const alice = { kind: 'identity' as const, id: 'xgid-alice' };
const bob = { kind: 'identity' as const, id: 'xgid-bob' };

const T0 = Date.parse('2026-07-18T12:00:00Z');
const min = 60_000;

const text = (id: string, author: typeof alice, atMs: number): MessageDescriptor => ({
  kind: 'text',
  id,
  author,
  body: id,
  timestamp: new Date(atMs).toISOString(),
});

const gap = (id: string, atMs: number, phase: StreamStatus['phase'] = 'counting-down'): StreamStatus => ({
  id,
  phase,
  timestamp: atMs,
  attempt: 2,
  maxAttempts: 10,
  remainingMs: 6000,
});

// `now` fixed a hair after the newest message so all same-day messages are "Today".
const NOW = new Date(T0 + 10 * min);

describe('computeRows — grouping (M-RP5.6 A, regression-locked)', () => {
  it('collapses a same-author run within the window', () => {
    const rows = computeRows([text('a1', alice, T0), text('a2', alice, T0 + 1 * min)], NOW);
    expect(rows.map((r) => r.kind)).toEqual(['message', 'message']);
    expect(rows[0].kind === 'message' && rows[0].grouped).toBe(false); // first row never grouped
    expect(rows[1].kind === 'message' && rows[1].grouped).toBe(true);
  });

  it('breaks a run on a different author', () => {
    const rows = computeRows([text('a1', alice, T0), text('b1', bob, T0 + 1 * min)], NOW);
    expect(rows[1].kind === 'message' && rows[1].grouped).toBe(false);
  });

  it('breaks a run once the window is exceeded', () => {
    const rows = computeRows([text('a1', alice, T0), text('a2', alice, T0 + GROUP_WINDOW_MS + 1)], NOW);
    expect(rows[1].kind === 'message' && rows[1].grouped).toBe(false);
  });
});

describe('computeRows — status placement (M-RP6.3 Leg C1, F-4)', () => {
  it('places a status row between two messages by timestamp', () => {
    const rows = computeRows(
      [text('a1', alice, T0), text('a2', alice, T0 + 2 * min)],
      NOW,
      [gap('g1', T0 + 1 * min)],
    );
    expect(rows.map((r) => r.kind)).toEqual(['message', 'status', 'message']);
  });

  it('places a status row BEFORE the first message when its timestamp precedes it', () => {
    const rows = computeRows([text('a1', alice, T0)], NOW, [gap('g1', T0 - 1 * min)]);
    expect(rows.map((r) => r.kind)).toEqual(['status', 'message']);
  });

  it('places a trailing status row after the last message', () => {
    const rows = computeRows([text('a1', alice, T0)], NOW, [gap('g1', T0 + 5 * min)]);
    expect(rows.map((r) => r.kind)).toEqual(['message', 'status']);
  });

  it('renders the whole status set when there are no messages (history accumulates)', () => {
    const rows = computeRows([], NOW, [gap('g1', T0), gap('g2', T0 + min, 'resolved')]);
    expect(rows.map((r) => r.kind)).toEqual(['status', 'status']);
  });

  it('carries DATA, never copy, and keys ONLY on the episode id (V6 — stable across phase)', () => {
    const countingDown = computeRows([], NOW, [gap('g1', T0, 'counting-down')]);
    const resolved = computeRows([], NOW, [gap('g1', T0, 'resolved')]);
    const a = countingDown[0];
    const b = resolved[0];
    expect(a.kind).toBe('status');
    expect(b.kind).toBe('status');
    // key is identical across a phase transition — the row matures in place, never re-keys.
    expect(a.key).toBe('status-g1');
    expect(a.key).toBe(b.key);
    // phase + numeric facts ride the row; no pre-formatted string.
    if (a.kind === 'status') {
      expect(a.phase).toBe('counting-down');
      expect(a.attempt).toBe(2);
      expect(a.maxAttempts).toBe(10);
      expect(a.remainingMs).toBe(6000);
    }
    if (b.kind === 'status') expect(b.phase).toBe('resolved');
  });

  it('a status row does NOT insert a day-divider (it has no calendar day of its own)', () => {
    // two same-day messages with a status between → no divider anywhere.
    const rows = computeRows(
      [text('a1', alice, T0), text('a2', alice, T0 + 2 * min)],
      NOW,
      [gap('g1', T0 + 1 * min)],
    );
    expect(rows.some((r) => r.kind === 'divider')).toBe(false);
  });
});

describe('computeRows — status BREAKS a grouping run (§9.7)', () => {
  it('a same-author pair split by a status row → the second message is not grouped', () => {
    // without the status the pair would group (same author, within window).
    const grouped = computeRows([text('a1', alice, T0), text('a2', alice, T0 + 2 * min)], NOW);
    expect(grouped[1].kind === 'message' && grouped[1].grouped).toBe(true);

    const broken = computeRows(
      [text('a1', alice, T0), text('a2', alice, T0 + 2 * min)],
      NOW,
      [gap('g1', T0 + 1 * min)],
    );
    const second = broken.find((r) => r.kind === 'message' && r.descriptor.id === 'a2');
    expect(second?.kind === 'message' && second.grouped).toBe(false);
  });
});

describe('computeRows — absent/empty status ⇒ byte-identical to the message-only walk', () => {
  const msgs = [
    text('a1', alice, T0),
    text('a2', alice, T0 + 1 * min),
    text('b1', bob, T0 + 2 * min),
  ];
  it('two-arg call === three-arg call with []', () => {
    expect(computeRows(msgs, NOW)).toEqual(computeRows(msgs, NOW, []));
  });
  it('and the pointer never advancing leaves grouping untouched', () => {
    const rows = computeRows(msgs, NOW, []);
    expect(rows.map((r) => r.kind)).toEqual(['message', 'message', 'message']);
    expect(rows[1].kind === 'message' && rows[1].grouped).toBe(true); // a1→a2 still grouped
    expect(rows[2].kind === 'message' && rows[2].grouped).toBe(false); // b1 different author
  });
});

describe('formatDayDivider — unchanged (M-RP5.6 A)', () => {
  // Local-constructed dates at noon: `calendarDayDiff` compares LOCAL calendar days, so a UTC-Z
  // string would be timezone-fragile. Weekday assertion is name-agnostic (every en-US weekday ends
  // in "day") so it does not depend on which weekday 4-days-back happens to be.
  const now = new Date(2026, 6, 18, 12, 0, 0); // 2026-07-18 noon local
  it('today → "Today (…)"', () => {
    expect(formatDayDivider(new Date(2026, 6, 18, 9, 0, 0), now)).toMatch(/^Today \(/);
  });
  it('yesterday → "Yesterday (…)"', () => {
    expect(formatDayDivider(new Date(2026, 6, 17, 9, 0, 0), now)).toMatch(/^Yesterday \(/);
  });
  it('2–6 days → weekday + date', () => {
    expect(formatDayDivider(new Date(2026, 6, 14, 9, 0, 0), now)).toMatch(/^[A-Za-z]+day \(/);
  });
  it('≥7 days → date only (no relative prefix)', () => {
    expect(formatDayDivider(new Date(2026, 6, 1, 9, 0, 0), now)).not.toMatch(/[()]/);
  });
});
