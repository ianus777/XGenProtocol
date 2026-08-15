// derive.test.ts — pure unit suite for the R5 projection map + gap-phase mapping (M-RP6.3 Leg C2). No DOM,
// no Svelte, no store — runs in the sampler vitest harness (`ui/sampler/vitest.config.js` scans
// `../common/lib/**/*.test.ts`). Locks the two things §5 named as "should grow npm test": the projection
// allowlist and the phase mapping — AND the grounded wire-field finding, FIXED AT SOURCE at J-551
// (`IngestEvent` now declares `type`; the `event_type` fallback is kept deliberately, and still covered).

import { describe, it, expect } from 'vitest';
import {
  wireType,
  shortId,
  membershipCopy,
  projectEvent,
  phaseFor,
  normaliseIntro,
  readIntro,
  INTRO_CONTENT_KEY,
} from './derive';
import type { IngestEvent } from '$common/stores/ingest.svelte';
import type { ResidentStatus } from '$common/stores/self-state.svelte';

const SELF = 'xgen://hash/sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const BOB = 'xgen://hash/sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb12345';
const noRedact = new Set<string>();

// A wire Event as it reaches the frontend — the kind rides `type` (serde rename), so tests build with `type`.
const ev = (over: Partial<IngestEvent> & { type?: string }): IngestEvent =>
  ({ event_id: 'e1', sender: BOB, room_id: 'r1', timestamp: '2026-07-19T10:00:00Z', ...over }) as IngestEvent;

describe('wireType — the grounded finding (wire.rs:476)', () => {
  it('reads the wire `type` field', () => {
    expect(wireType(ev({ type: 'message.text' }))).toBe('message.text');
  });
  it('still falls back to the legacy `event_type` name (kept deliberately, J-551)', () => {
    expect(wireType({ event_type: 'message.text' } as unknown as IngestEvent)).toBe('message.text');
  });
  it('is undefined when neither is present', () => {
    expect(wireType({} as IngestEvent)).toBeUndefined();
  });
});

describe('shortId', () => {
  it('is the last 6 chars of the xgid tail', () => {
    expect(shortId(BOB)).toBe('b12345');
  });
  it('empty in, empty out', () => {
    expect(shortId('')).toBe('');
  });
});

describe('projectEvent — the allowlist (C-2), default: ignore', () => {
  it('message.text → a text descriptor, body from content.text', () => {
    const m = projectEvent(ev({ type: 'message.text', content: { text: 'hello' } }), SELF, noRedact);
    expect(m).toMatchObject({ kind: 'text', id: 'e1', body: 'hello', author: { kind: 'identity', id: BOB } });
    expect(m?.author?.name).toBeUndefined(); // C-8: no name fabricated
  });
  it('isOwn iff sender === self', () => {
    expect(projectEvent(ev({ type: 'message.text', sender: SELF }), SELF, noRedact)?.isOwn).toBe(true);
    expect(projectEvent(ev({ type: 'message.text', sender: BOB }), SELF, noRedact)?.isOwn).toBe(false);
  });
  it('isOwn false when self is unknown', () => {
    expect(projectEvent(ev({ type: 'message.text', sender: BOB }), null, noRedact)?.isOwn).toBe(false);
  });
  it('non-string / absent content.text → empty body', () => {
    expect(projectEvent(ev({ type: 'message.text', content: {} }), SELF, noRedact)?.body).toBe('');
    expect(projectEvent(ev({ type: 'message.text', content: { text: 42 } }), SELF, noRedact)?.body).toBe('');
  });
  it('a redacted message.text → deleted (redact mutates the target, is not its own row)', () => {
    const m = projectEvent(ev({ type: 'message.text', event_id: 'e9' }), SELF, new Set(['e9']));
    expect(m?.deleted).toBe(true);
  });
  it('an un-redacted message.text → not deleted', () => {
    expect(projectEvent(ev({ type: 'message.text', event_id: 'e9' }), SELF, new Set(['other']))?.deleted).toBe(false);
  });
  it('membership.join/leave → a system notice with the sender tail', () => {
    expect(projectEvent(ev({ type: 'membership.join' }), SELF, noRedact)).toMatchObject({
      kind: 'system',
      body: 'b12345 joined',
    });
    expect(projectEvent(ev({ type: 'membership.leave' }), SELF, noRedact)?.body).toBe('b12345 left');
  });
  it('membership.kick/ban/node_eject → a generic system notice (no misattribution)', () => {
    expect(projectEvent(ev({ type: 'membership.kick' }), SELF, noRedact)?.kind).toBe('system');
    expect(projectEvent(ev({ type: 'membership.ban' }), SELF, noRedact)?.body).toBe('A member was banned');
  });
  it('message.redact → dropped (null): handled via redactedIds, never its own row', () => {
    expect(projectEvent(ev({ type: 'message.redact' }), SELF, noRedact)).toBeNull();
  });
  it('everything else is ignored by design', () => {
    for (const t of ['state.space_create', 'mls.commit', 'membership.invite', 'membership.mute', 'reaction', 'x.unknown']) {
      expect(projectEvent(ev({ type: t }), SELF, noRedact)).toBeNull();
    }
  });
  it('an event with no `type` is dropped, not crashed', () => {
    expect(projectEvent({} as IngestEvent, SELF, noRedact)).toBeNull();
  });
});

// ── M-RP-INTRO Leg 3: the intro content key ────────────────────────────────────────────────────────────
//
// 🛑 THIS IS A TRUST BOUNDARY, so the suite is written as a hostile-input table rather than a happy path.
// The payload is authored by a person the reader has never met, and NOTHING type-checks a `WidgetMount`'s
// prop bag (`B-8`) — so the property under test is "no mount rather than a broken one", and every case
// additionally asserts that `body` survives (1-bis: the row still renders the sender's sentence).

describe('normaliseIntro — the ONE validation rule, shared by both directions', () => {
  it('keeps two non-blank strings', () => {
    expect(normaliseIntro({ headline: 'hi', blurb: 'about me' })).toEqual({ headline: 'hi', blurb: 'about me' });
  });
  it('either field alone is a usable intro', () => {
    expect(normaliseIntro({ headline: 'hi' })?.headline).toBe('hi');
    expect(normaliseIntro({ blurb: 'about me' })?.blurb).toBe('about me');
  });
  it('drops a blank or whitespace-only field rather than rendering an empty box', () => {
    expect(normaliseIntro({ headline: '', blurb: 'b' })?.headline).toBeUndefined();
    expect(normaliseIntro({ headline: '   ', blurb: 'b' })?.headline).toBeUndefined();
  });
  it('drops a non-string field — a number is not text', () => {
    expect(normaliseIntro({ headline: 42, blurb: 'b' })?.headline).toBeUndefined();
    expect(normaliseIntro({ headline: { nested: 'x' } })).toBeNull();
  });
  it('NOTHING renderable ⇒ null, never an empty object (N-182 one layer down)', () => {
    expect(normaliseIntro({})).toBeNull();
    expect(normaliseIntro({ headline: '', blurb: '  ' })).toBeNull();
    expect(normaliseIntro({ unexpected: 'member' })).toBeNull();
  });
  it('a non-object is null, not a crash — string, array, number, null, undefined', () => {
    for (const bad of ['a string', ['a', 'list'], 7, null, undefined, true]) {
      expect(normaliseIntro(bad)).toBeNull();
    }
  });
  it('preserves the raw string verbatim — no trimming, no escaping, no rewriting', () => {
    expect(normaliseIntro({ headline: '  padded  ' })?.headline).toBe('  padded  ');
  });
});

describe('readIntro — the key is read from content and nowhere else', () => {
  it('reads the versioned key', () => {
    expect(readIntro({ [INTRO_CONTENT_KEY]: { headline: 'hi' } })?.headline).toBe('hi');
  });
  it('the key string is exactly `xgen.intro.v1` (its Rust mirror is resident.rs)', () => {
    expect(INTRO_CONTENT_KEY).toBe('xgen.intro.v1');
  });
  it('a FUTURE version is ignored by a v1 reader — the whole point of versioning the key', () => {
    expect(readIntro({ 'xgen.intro.v2': { headline: 'hi' } })).toBeNull();
  });
  it('absent key, or absent content, ⇒ null', () => {
    expect(readIntro({ text: 'hello' })).toBeNull();
    expect(readIntro(undefined)).toBeNull();
  });
});

describe('projectEvent — the intro is ADDITIVE and never displaces the body (1-bis)', () => {
  const withIntro = (intro: unknown) =>
    projectEvent(
      ev({ type: 'message.text', content: { text: 'hello', [INTRO_CONTENT_KEY]: intro } }),
      SELF,
      noRedact,
    );

  it('a valid intro mounts `message-intro` in bodyExtras, and the body is unchanged', () => {
    const m = withIntro({ headline: 'hi', blurb: 'about me' });
    expect(m?.body).toBe('hello');
    expect(m?.bodyExtras).toEqual([
      { widgetId: 'message-intro', mountKey: 'message-intro', props: { intro: { headline: 'hi', blurb: 'about me' } } },
    ]);
  });
  it('NO intro ⇒ NO bodyExtras key at all — an ordinary row is byte-identical to before (N-182)', () => {
    const m = projectEvent(ev({ type: 'message.text', content: { text: 'hello' } }), SELF, noRedact);
    expect(m?.bodyExtras).toBeUndefined();
    expect('bodyExtras' in (m as object)).toBe(false);
  });
  it('a MALFORMED intro produces no mount and no crash — and the sentence still renders', () => {
    for (const bad of ['a string', ['a'], 7, null, {}, { headline: '' }, { headline: 42 }]) {
      const m = withIntro(bad);
      expect(m?.bodyExtras).toBeUndefined();
      expect(m?.body).toBe('hello');
    }
  });
  it('an OVERSIZED blurb still mounts — bounding is the widget’s job, not the projection’s', () => {
    const m = withIntro({ blurb: 'x'.repeat(10_000) });
    expect(m?.bodyExtras).toHaveLength(1);
  });
  it('an intro on an event whose text is MISSING ⇒ empty body, and the intro must not supply one', () => {
    // ⚠️ The `text` key is genuinely ABSENT here, not passed as `undefined` through the helper — a default
    // parameter would have silently reinstated `'hello'` and the assertion would have been testing nothing.
    const m = projectEvent(
      ev({ type: 'message.text', content: { [INTRO_CONTENT_KEY]: { headline: 'hi' } } }),
      SELF,
      noRedact,
    );
    expect(m?.body).toBe('');
    expect(m?.bodyExtras).toHaveLength(1);
  });
  it('the key on a NON-text event is ignored — the allowlist decides the row kind, not the payload', () => {
    const m = projectEvent(
      ev({ type: 'membership.join', content: { [INTRO_CONTENT_KEY]: { headline: 'hi' } } }),
      SELF,
      noRedact,
    );
    expect(m?.kind).toBe('system');
    expect(m?.bodyExtras).toBeUndefined();
  });
  it('a REDACTED row carrying an intro is still marked deleted', () => {
    const m = projectEvent(
      ev({ type: 'message.text', event_id: 'e9', content: { text: 'hello', [INTRO_CONTENT_KEY]: { headline: 'hi' } } }),
      SELF,
      new Set(['e9']),
    );
    expect(m?.deleted).toBe(true);
  });
});

describe('phaseFor — the gap phase mapping (§3.1)', () => {
  const res = (over: Partial<ResidentStatus>): ResidentStatus =>
    ({ attempt: 1, max_attempts: 10, next_attempt_in_ms: null, terminal: false, connect_timeout_ms: 10000, ping_interval_ms: 10000, ...over });
  it('terminal wins → exhausted', () => {
    expect(phaseFor(res({ terminal: true, next_attempt_in_ms: 5000 }))).toBe('exhausted');
  });
  it('a live countdown → counting-down', () => {
    expect(phaseFor(res({ next_attempt_in_ms: 5000 }))).toBe('counting-down');
  });
  it('not terminal, no countdown → dialling', () => {
    expect(phaseFor(res({ next_attempt_in_ms: null }))).toBe('dialling');
  });
  it('a null resident (pre-first-poll) → dialling', () => {
    expect(phaseFor(null)).toBe('dialling');
  });
});

describe('membershipCopy directly', () => {
  it('unknown membership type → empty (defensive)', () => {
    expect(membershipCopy('membership.whatever', ev({}))).toBe('');
  });
});
