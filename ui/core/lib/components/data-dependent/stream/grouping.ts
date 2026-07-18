// message-stream pure helpers (M-RP5.6 A, `docs/xgen-dd-message-family-phase0.md` v1.1 §9).
// Colocated + unit-testable (the `processor/transform.ts` / `clamp.ts` precedent): the stream
// component owns the DOM + envelope; THIS module owns the sibling-relationship maths — grouping
// runs + day-divider placement — as pure functions over a `MessageDescriptor[]`. `core` stays
// protocol-free: `MessageDescriptor` is a `core` view-model (data-dependent/types.ts), not a
// protocol type.

import type { MessageDescriptor } from '../types';

/**
 * Grouping window (§9.1). A `text` message renders `grouped` (continuation mode) only if the
 * previous rendered message is a `text` from the same author within this window and no day-divider
 * sits between them. Build-time const, Joe-tunable.
 */
export const GROUP_WINDOW_MS = 5 * 60 * 1000; // 5 min

/**
 * Divider re-label cadence (M-RP6.3 Leg C1, F-3). The captured-once `now` froze divider labels for
 * the life of the process — invisible on a 30s sampler fixture, wrong on a resident that runs for
 * days ("Today (Jul 18)" is still "Today" on the 19th). The stream advances its reactive `now` on an
 * interval of this length; 60s is far finer than the one-DAY granularity the labels actually have,
 * and cheap. Build-time const, colocated with `GROUP_WINDOW_MS` (D5 — one place, Joe-tunable).
 */
export const DIVIDER_REFRESH_MS = 60_000; // 1 min

/**
 * A connection-gap STATUS episode (M-RP6.3 Leg C1, F-4) — the live mutable item R5 does not have
 * today. This carries FACTS, never pre-formatted copy (formatting + appearance are the component's
 * and Ms Design's, §3.3/§3.4). An ARRAY of these is supplied because a session accumulates history:
 * "the live widget IS the historical marker, matured" (§2) — three outages leave three markers, two
 * resolved and one live.
 *
 * `id` is a STABLE episode identity that SURVIVES every phase transition — it is what
 * `StreamRow.key` derives from, so the row matures/collapses in place instead of being destroyed and
 * recreated by Svelte on each phase change (§3.3, the V6 trap). Consumer supplies these ordered by
 * `timestamp`, the same contract `messages` already carries (the stream does NOT re-sort).
 */
export interface StreamStatus {
  id: string;
  phase: 'counting-down' | 'dialling' | 'resolved' | 'exhausted';
  timestamp: number; // ms epoch — placement in the single walk (when the gap began)
  attempt?: number;
  maxAttempts?: number;
  remainingMs?: number | null; // prospective — time to the next dial
  resolvedAfterMs?: number | null; // retrospective — how long the outage lasted (on resolve)
}

/**
 * A stream row is a materialized message (carrying its stream-computed `grouped` flag), a day-divider
 * separator, OR a connection-gap status row (carrying DATA, never copy). The component renders this
 * ordered list 1:1 — grouping, divider placement and status placement are decided here, once, per
 * render.
 */
export type StreamRow =
  | { kind: 'message'; key: string; descriptor: MessageDescriptor; grouped: boolean }
  | { kind: 'divider'; key: string; label: string }
  | {
      kind: 'status';
      key: string; // `status-${StreamStatus.id}` — STABLE across phase transitions (§3.3)
      phase: StreamStatus['phase'];
      attempt?: number;
      maxAttempts?: number;
      remainingMs?: number | null;
      resolvedAfterMs?: number | null;
    };

// Fixed en-US formatters (DOM-free `Intl`, the `converter-field` precedent) so the divider label is
// deterministic regardless of the runtime locale: "Jul 8, 2026" + "Saturday".
const DATE_FMT = new Intl.DateTimeFormat('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
const WEEKDAY_FMT = new Intl.DateTimeFormat('en-US', { weekday: 'long' });

/** Whole LOCAL calendar days from `ts` back to `now` (today → 0, yesterday → 1). */
function calendarDayDiff(now: Date, ts: Date): number {
  const a = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const b = new Date(ts.getFullYear(), ts.getMonth(), ts.getDate()).getTime();
  return Math.round((a - b) / 86_400_000);
}

/**
 * Day-divider label (§9.2). Always carries the date; the relative prefix drops once old:
 *   - today            → `Today (Jul 8, 2026)`
 *   - yesterday        → `Yesterday (Jul 7, 2026)`
 *   - 2–6 days ago     → `Saturday (Jul 6, 2026)` (weekday + date)
 *   - ≥7 days ago      → `Jul 1, 2026` (date only)
 * Build-time formatter, Joe-tunable.
 */
export function formatDayDivider(ts: Date, now: Date): string {
  const dateStr = DATE_FMT.format(ts);
  const diff = calendarDayDiff(now, ts);
  if (diff === 0) return `Today (${dateStr})`;
  if (diff === 1) return `Yesterday (${dateStr})`;
  if (diff >= 2 && diff <= 6) return `${WEEKDAY_FMT.format(ts)} (${dateStr})`;
  return dateStr; // ≥7 days (and any future/edge diff) → date only
}

/**
 * Walk the ordered messages once, producing the interleaved row list (§9.1 + §9.2 + §3.3).
 *
 * DIVIDERS — a separator is inserted BETWEEN two consecutive messages when the local calendar day
 * changes (compare `toDateString()`). There is NO leading divider before the first message (the spec
 * compares "the two timestamps" — two are required); the oldest day therefore heads the stream
 * un-labelled, and the four label bands are exhibited by day-CHANGES down the stream.
 *
 * GROUPING — a `text` message is `grouped` iff the previous rendered message is a `text` with the
 * SAME `author.id`, within `GROUP_WINDOW_MS`, and NO divider AND NO status row was inserted between
 * them. Breaks: different author · any `system` message (authorless) · a day boundary (divider) · a
 * connection-gap status row · the first row. A `deleted` tombstone is still `kind:'text'` with its
 * author, so it does NOT break a run.
 *
 * STATUS — connection-gap rows (§3.3, F-4) are placed by `timestamp` in this SAME single walk (a
 * pointer advances through the `statuses` array — no second pass). A status row does NOT participate
 * in day-divider computation (it has no calendar day of its own), and it BREAKS a grouping run (a
 * continuation rendered across a visible disconnect reads wrong, §9.7). Absent/empty `statuses` ⇒
 * the pointer never advances ⇒ output is byte-identical to the message-only walk.
 */
export function computeRows(
  messages: MessageDescriptor[],
  now: Date,
  statuses: StreamStatus[] = [],
): StreamRow[] {
  const rows: StreamRow[] = [];
  let prev: MessageDescriptor | null = null;
  let prevDay: string | null = null;
  let si = 0; // pointer into `statuses` (assumed timestamp-ordered, like `messages`)

  // Emit every not-yet-emitted status whose timestamp is at/before `boundaryMs`; returns whether any
  // were emitted (a placed status breaks the current grouping run).
  const drainStatusesUpTo = (boundaryMs: number): boolean => {
    let emitted = false;
    while (si < statuses.length && statuses[si].timestamp <= boundaryMs) {
      const s = statuses[si];
      rows.push({
        kind: 'status',
        key: `status-${s.id}`, // STABLE across phase transitions — never keyed on phase/data (V6)
        phase: s.phase,
        attempt: s.attempt,
        maxAttempts: s.maxAttempts,
        remainingMs: s.remainingMs,
        resolvedAfterMs: s.resolvedAfterMs,
      });
      si++;
      emitted = true;
    }
    return emitted;
  };

  messages.forEach((m, i) => {
    const ts = new Date(m.timestamp);
    const day = ts.toDateString();

    const dividerBetween = prevDay !== null && day !== prevDay;
    if (dividerBetween) {
      rows.push({ kind: 'divider', key: `div-${i}`, label: formatDayDivider(ts, now) });
    }

    // Status rows sit before this message when their timestamp precedes it; they break the run.
    const statusBetween = drainStatusesUpTo(ts.getTime());

    let grouped = false;
    if (
      m.kind === 'text' &&
      prev !== null &&
      prev.kind === 'text' &&
      !dividerBetween &&
      !statusBetween &&
      prev.author?.id != null &&
      m.author?.id === prev.author.id
    ) {
      const gap = ts.getTime() - new Date(prev.timestamp).getTime();
      grouped = gap >= 0 && gap <= GROUP_WINDOW_MS;
    }

    rows.push({ kind: 'message', key: m.id, descriptor: m, grouped });
    prev = m;
    prevDay = day;
  });

  // Trailing statuses — timestamps after the last message, or the whole set when there are no
  // messages. They render at the tail (the live gap is the most recent thing in the stream).
  drainStatusesUpTo(Infinity);

  return rows;
}
