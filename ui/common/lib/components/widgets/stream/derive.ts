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
 * `xgen-common/src/wire.rs:476`), because `type` is a Rust keyword and the RUST field is therefore
 * `event_type`. The Leg-B `IngestEvent` interface declared the RUST name; C2 was the FIRST reader of this
 * field (Leg B only ever read counts), so the drift was latent. FIXED AT SOURCE at J-551: `IngestEvent` now
 * declares `type`. The `event_type` fallback below is KEPT DELIBERATELY as a legacy read — it costs one `??`,
 * its unit test is part of the `npm test` floor, and removing it is a separate change. Every other
 * `IngestEvent` field matches the wire.
 */
export function wireType(e: IngestEvent): string | undefined {
  const w = e as { type?: string; event_type?: string };
  return w.type ?? w.event_type;
}

/** A short readable handle from an XGID (`xgen://hash/sha256:<hex>`) — the honest identifier while no
 *  XGID → display-name resolution exists (C-8: do NOT fabricate a name map). */
export function shortId(id: string): string {
  return id ? id.slice(-6) : '';
}

// ── The DM welcome intro (M-RP-INTRO) ────────────────────────────────────────────────────────────────────

/**
 * 🔒 THE INTRO CONTENT KEY — NAMED HERE AND NOWHERE ELSE IN TS (runbook §4.1). Its only mirror is the single
 * `pub const` in `xgen-client/src/resident.rs`, which is where the WRITE side names it: the frontend hands
 * the bare payload to `send_message` and Rust wraps it under this key, so TS names it exactly once, on read.
 * A second spelling on either side is how drift starts (D-122).
 *
 * 🔑 VERSIONED DELIBERATELY (Joe, Phase-0 §3.1) — a successor is `xgen.intro.v2`, and a reader that knows
 * only v1 ignores v2 WHILE STILL RENDERING `content.text`. That degradation is rich → plain rather than
 * rich → nothing, and it is the entire reason 1-bis requires `text` to stay load-bearing forever.
 */
export const INTRO_CONTENT_KEY = 'xgen.intro.v1';

/**
 * The intro payload as it rides `content['xgen.intro.v1']` (Phase-0 §3.1 / runbook §2.1).
 *
 * TWO OPTIONAL STRING FIELDS AND NOTHING ELSE. Every field added here must be rendered, escaped,
 * length-bounded and versioned forever; `xgen.intro.v2` exists precisely so fields do not have to be guessed
 * now. 🛑 NO url, no image ref, no avatar in v1 — a fetch the recipient did not ask for on first contact is
 * `M-INTRO-POLICY`'s problem, not this milestone's.
 *
 * ⚠️ ONE TYPE FOR BOTH DIRECTIONS, DELIBERATELY. The composer AUTHORS this shape and `readIntro` RETURNS it.
 * They are the same payload, and two declarations of one payload is the D-067 drift surface this codebase
 * keeps paying for — which is also why `normaliseIntro` below is the single validation rule both sides call.
 */
export interface MessageIntro {
  headline?: string;
  blurb?: string;
}

/** A field survives only as a non-blank string. `''` would render an empty box that reads as a layout
 *  defect, and `42` is not text. */
function keepText(v: unknown): string | undefined {
  return typeof v === 'string' && v.trim() !== '' ? v : undefined;
}

/**
 * THE ONE VALIDATION RULE, called by BOTH sides (the composer before it sends, `readIntro` after it arrives).
 * Returns `null` for anything that is not a usable intro.
 *
 * 🛑 IT IS A TRUST BOUNDARY ON THE READ SIDE. The value is wire data authored by a person the reader has
 * never met, and NOTHING type-checks a `WidgetMount`'s prop bag (`B-8`, `types.ts:53-71`) — so a non-object,
 * an array, `null`, a string, or unexpected members must produce NO MOUNT rather than a broken one.
 *
 * 🛑 AND IF NOTHING SURVIVES THERE IS NO INTRO. An intro object with no renderable field is the
 * reserved-unfed shape `N-182` forbids, one layer down: on the write side it is what keeps a send with an
 * untouched intro form byte-identical to today's, and on the read side it is what stops an empty mount.
 */
export function normaliseIntro(raw: unknown): MessageIntro | null {
  if (raw == null || typeof raw !== 'object' || Array.isArray(raw)) return null;
  const o = raw as Record<string, unknown>;
  const headline = keepText(o.headline);
  const blurb = keepText(o.blurb);
  if (headline === undefined && blurb === undefined) return null;
  return { headline, blurb };
}

/** Read + validate the intro key off an event's content. Delegates to `normaliseIntro` so the read side and
 *  the write side can never disagree about what counts as an intro. */
export function readIntro(content: Record<string, unknown> | undefined): MessageIntro | null {
  return normaliseIntro(content?.[INTRO_CONTENT_KEY]);
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
    // M-RP-INTRO Leg 3 — the intro is PURELY ADDITIVE here. `body` is untouched: it still reads
    // `content.text` defensively, so a row whose intro is malformed, unknown or absent renders exactly what
    // it rendered before this milestone (1-bis). `readIntro` returns `null` for anything unusable, and a
    // `null` produces NO `bodyExtras` key at all rather than an empty array (`N-182`).
    const intro = readIntro(e.content);
    return {
      kind: 'text',
      id: eid,
      author: { kind: 'identity', id: e.sender ?? '' }, // NO name (C-8) — avatar falls back to xgid-tail initials
      body: typeof e.content?.text === 'string' ? e.content.text : '',
      timestamp: e.timestamp ?? '',
      isOwn: selfId != null && e.sender === selfId,
      deleted: eid !== '' && redactedIds.has(eid),
      // `mountKey` is a CONSTANT, not id-composed: `resolveMounts` scopes keys PER ROW, so one mount per row
      // is already unique (the `send-status` reasoning at `stream-panel.svelte:126-128`).
      ...(intro
        ? { bodyExtras: [{ widgetId: 'message-intro', mountKey: 'message-intro', props: { intro } }] }
        : {}),
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
