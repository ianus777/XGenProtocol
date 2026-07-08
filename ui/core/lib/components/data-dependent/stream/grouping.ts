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
 * A stream row is EITHER a materialized message (carrying its stream-computed `grouped` flag) OR a
 * day-divider separator. The component renders this ordered list 1:1 — grouping + divider placement
 * are decided here, once, per render.
 */
export type StreamRow =
  | { kind: 'message'; key: string; descriptor: MessageDescriptor; grouped: boolean }
  | { kind: 'divider'; key: string; label: string };

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
 * Walk the ordered messages once, producing the interleaved row list (§9.1 + §9.2).
 *
 * DIVIDERS — a separator is inserted BETWEEN two consecutive messages when the local calendar day
 * changes (compare `toDateString()`). There is NO leading divider before the first message (the spec
 * compares "the two timestamps" — two are required); the oldest day therefore heads the stream
 * un-labelled, and the four label bands are exhibited by day-CHANGES down the stream.
 *
 * GROUPING — a `text` message is `grouped` iff the previous rendered message is a `text` with the
 * SAME `author.id`, within `GROUP_WINDOW_MS`, and NO divider was inserted between them. Breaks:
 * different author · any `system` message (authorless) · a day boundary (divider) · the first row.
 * A `deleted` tombstone is still `kind:'text'` with its author, so it does NOT break a run.
 */
export function computeRows(messages: MessageDescriptor[], now: Date): StreamRow[] {
  const rows: StreamRow[] = [];
  let prev: MessageDescriptor | null = null;
  let prevDay: string | null = null;

  messages.forEach((m, i) => {
    const ts = new Date(m.timestamp);
    const day = ts.toDateString();

    const dividerBetween = prevDay !== null && day !== prevDay;
    if (dividerBetween) {
      rows.push({ kind: 'divider', key: `div-${i}`, label: formatDayDivider(ts, now) });
    }

    let grouped = false;
    if (
      m.kind === 'text' &&
      prev !== null &&
      prev.kind === 'text' &&
      !dividerBetween &&
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

  return rows;
}
