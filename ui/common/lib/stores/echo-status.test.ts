// echo-status.test.ts — M-RP6.3 Leg D2/D3. These cover the three rules that decide what a user is TOLD
// about their own message and whether they are handed a button that can double-post it.
//
// ⚠️ THE LOAD-BEARING CASE IS `narrowStatus('<anything else>') === 'timed_out'`. Mutate that to 'failed'
// and every test below still passes except that one — which is exactly why it is written down: `failed`
// retries freely, so the wrong default silently hands the user a way to put a permanent,
// identity-attributed duplicate on the federated network.

import { describe, it, expect } from 'vitest';
import { narrowStatus, toneOf, isRetryable, type SendStatus } from './echo-status';

describe('narrowStatus', () => {
  it('passes the four honest Rust outcomes through unchanged', () => {
    expect(narrowStatus('accepted')).toBe('accepted');
    expect(narrowStatus('rejected')).toBe('rejected');
    expect(narrowStatus('timed_out')).toBe('timed_out');
    expect(narrowStatus('failed')).toBe('failed');
  });

  it('maps an UNRECOGNISED status to timed_out, never to failed (D6 / §3.1)', () => {
    // A version skew is the only way here. `timed_out` says "we do not know", which is true; `failed`
    // would claim "never reached the wire" AND unlock a retry that can duplicate the message.
    expect(narrowStatus('quantum')).toBe('timed_out');
    expect(narrowStatus('')).toBe('timed_out');
    expect(narrowStatus('ACCEPTED')).toBe('timed_out'); // case-sensitive on purpose: the wire is exact
  });

  it('never returns a status that is retryable when it did not recognise the input', () => {
    // The property behind the case above, stated as the property rather than the value — so a future
    // change of the fallback has to confront the actual invariant.
    expect(isRetryable(narrowStatus('something-new'))).toBe(false);
  });
});

describe('toneOf — lock #6, THREE states not two', () => {
  it('gives timed_out its OWN tone, distinct from both neighbours', () => {
    expect(toneOf('timed_out')).toBe('unresolved');
    expect(toneOf('timed_out')).not.toBe(toneOf('accepted'));
    expect(toneOf('timed_out')).not.toBe(toneOf('failed'));
  });

  it('groups rejected and failed as one tone (different copy, same state)', () => {
    expect(toneOf('rejected')).toBe('not-sent');
    expect(toneOf('failed')).toBe('not-sent');
  });

  it('renders accepted as sent and pending as its own quiet fourth', () => {
    expect(toneOf('accepted')).toBe('sent');
    expect(toneOf('pending')).toBe('pending');
  });
});

describe('isRetryable — lock #7 as narrowed at §3.1', () => {
  it('allows retry ONLY for failed', () => {
    expect(isRetryable('failed')).toBe(true);
  });

  it('refuses retry for timed_out — the only status where a click can duplicate on the wire', () => {
    expect(isRetryable('timed_out')).toBe(false);
  });

  it('refuses retry for rejected (it will be refused again) and pending (nothing to retry yet)', () => {
    expect(isRetryable('rejected')).toBe(false);
    expect(isRetryable('pending')).toBe(false);
  });

  it('is exhaustive over SendStatus — exactly one status is retryable', () => {
    const all: SendStatus[] = ['pending', 'accepted', 'rejected', 'timed_out', 'failed'];
    expect(all.filter(isRetryable)).toEqual(['failed']);
  });
});
