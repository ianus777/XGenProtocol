// stream/derive.ts — the PURE R5 logic (M-RP6.3 Leg C2): the event → `MessageDescriptor` projection
// allowlist (C-2) and the resident → gap-phase mapping (§3.1). Colocated + unit-testable — the
// `stream/grouping.ts` / `processor/transform.ts` / `clamp.ts` precedent: the widget (`stream-panel`) owns
// the reactive store reads + the DOM; THIS module owns the pure maps as functions over plain values. No
// runes, no store imports (all imports are TYPE-ONLY and erased at build), so it runs in the node vitest
// harness that cannot load a `.svelte.ts`.
//
// `core` stays protocol-free (W-3): `MessageDescriptor` / `StreamStatus` are `core` view-models, imported as
// TYPES only; `IngestEvent` / `ResidentStatus` are `$common` store shapes, also type-only.

import type { IngestEvent } from '$common/stores/ingest.svelte';
import type { ResidentStatus } from '$common/stores/self-state.svelte';
import type { MessageDescriptor } from '$core/components/data-dependent/types';
import type { StreamStatus } from '$core/components/data-dependent/stream/grouping';

/**
 * ⚠️ GROUNDED FINDING (surfaced): the wire `Event` serialises its kind as `type` (serde rename,
 * `xgen-common/src/wire.rs:476`), NOT `event_type` as the Leg-B `IngestEvent` interface declares. C2 is the
 * FIRST reader of this field (Leg B only ever read counts), so this drift was latent. Read the grounded wire
 * name, falling back to `event_type` so a later interface correction cannot silently break this. Every other
 * `IngestEvent` field matches the wire.
 */
export function wireType(e: IngestEvent): string | undefined {
  return (e as IngestEvent & { type?: string }).type ?? e.event_type;
}

/** A short readable handle from an XGID (`xgen://hash/sha256:<hex>`) — the honest identifier while no
 *  XGID → display-name resolution exists (C-8: do NOT fabricate a name map). */
export function shortId(id: string): string {
  return id ? id.slice(-6) : '';
}

/**
 * membership.* → a system centred notice (C-2). join/leave subject = the SENDER (grounded); kick/ban/
 * node_eject carry an actor-not-subject shape this leg does not ground (untested — only membership.join is
 * driven at verify, C-3) → a generic notice, no misattribution. WORDING is PROVISIONAL (Ms Design).
 */
export function membershipCopy(t: string, e: IngestEvent): string {
  const who = shortId(e.sender ?? '');
  switch (t) {
    case 'membership.join':
      return `${who} joined`;
    case 'membership.leave':
      return `${who} left`;
    case 'membership.kick':
      return 'A member was removed';
    case 'membership.ban':
      return 'A member was banned';
    case 'membership.node_eject':
      return 'A member was ejected';
    default:
      return '';
  }
}

/**
 * Project ONE ingest Event to a `MessageDescriptor`, or `null` to DROP it (C-2). An EXPLICIT allowlist with a
 * `default: ignore` arm — never a denylist (a denylist admits every future protocol type by default, and the
 * wire already ships `Unknown(String)` precisely because new types are expected).
 *
 *   message.text                → `text` (body = content.text; isOwn = sender === self; deleted iff redacted)
 *   membership.join/leave/kick/ban/node_eject → `system` centred notice
 *   message.redact              → NOT a row (handled by the caller via `redactedIds` → the target's `deleted`)
 *   everything else (state.*, mls.*, migration.*, Unknown, …) → ignore, silently and by design
 */
export function projectEvent(
  e: IngestEvent,
  selfId: string | null,
  redactedIds: ReadonlySet<string>,
): MessageDescriptor | null {
  const t = wireType(e);
  if (t === 'message.text') {
    const eid = e.event_id ?? '';
    return {
      kind: 'text',
      id: eid,
      author: { kind: 'identity', id: e.sender ?? '' }, // NO name (C-8) — avatar falls back to xgid-tail initials
      body: typeof e.content?.text === 'string' ? e.content.text : '',
      timestamp: e.timestamp ?? '',
      isOwn: selfId != null && e.sender === selfId,
      deleted: eid !== '' && redactedIds.has(eid),
    };
  }
  if (
    t === 'membership.join' ||
    t === 'membership.leave' ||
    t === 'membership.kick' ||
    t === 'membership.ban' ||
    t === 'membership.node_eject'
  ) {
    return { kind: 'system', id: e.event_id ?? '', body: membershipCopy(t, e), timestamp: e.timestamp ?? '' };
  }
  return null;
}

/**
 * The gap PHASE from a resident snapshot (§3.1) — DERIVED, never stored. `exhausted` wins (parked at the
 * cap); then a live countdown is `counting-down`; otherwise `dialling` (not terminal, no countdown = a dial
 * in flight, or a null resident before the first poll).
 */
export function phaseFor(res: ResidentStatus | null): StreamStatus['phase'] {
  if (res?.terminal) return 'exhausted';
  if (res && res.next_attempt_in_ms != null) return 'counting-down';
  return 'dialling';
}
