// transform.test.ts — unit suite for the text-processor pure cores (M-RP-PROCESSOR-SEED, Leg D).
//
// SCOPE: `findUnreachableRules` only. The rest of transform.ts (applyRules / assertSafeRules /
// parseRules / applyClamp / intlNumber) predates this file and is exercised through its consumers;
// this suite is not a retro-fit of coverage it never had, it is the diagnostic's own harness.
//
// Runs under the standing sampler vitest harness (M-RP6.1c, D4-V1) via its
// `../common/lib/**/*.test.ts` include — no config change was needed to pick it up.

import { describe, it, expect } from 'vitest';
import { findUnreachableRules, parseRules, assertSafeRules } from './transform';

describe('findUnreachableRules', () => {
  it('reports nothing for a rule set with no shared prefixes', () => {
    const rules = parseRules('-> → | <- ← | :) 🙂');
    expect(rules).toHaveLength(3); // the subject is real, not an empty list
    expect(findUnreachableRules(rules)).toEqual([]);
  });

  it('flags the shipped defect: `--` shadows `-->`', () => {
    const rules = parseRules('--> → | -- ‒');
    expect(findUnreachableRules(rules)).toEqual([{ find: '-->', shadowedBy: '--' }]);
  });

  it('is independent of list order — typing order decides, not position', () => {
    const before = findUnreachableRules(parseRules('--> → | -- ‒'));
    const after = findUnreachableRules(parseRules('-- ‒ | --> →'));
    expect(after).toEqual(before);
  });

  it('does not flag a rule that merely CONTAINS another find — only a proper PREFIX shadows', () => {
    // `b` occurs inside `abc`, but typing `abc` never passes through a buffer of `b`
    // at position 0, so `abc` is reachable. A substring test would wrongly flag it.
    expect(findUnreachableRules(parseRules('abc x | b y'))).toEqual([]);
  });

  it('does not flag a suffix match either', () => {
    expect(findUnreachableRules(parseRules('abc x | c y'))).toEqual([]);
  });

  it('never flags a rule as shadowing itself', () => {
    expect(findUnreachableRules(parseRules('-- ‒'))).toEqual([]);
  });

  it('treats two rules with an identical find as duplicates, not shadows', () => {
    // Equal length ⇒ neither is a PROPER prefix. Duplicates are a different problem
    // (last-write-wins in applyRules) and are deliberately not this check's business.
    expect(findUnreachableRules(parseRules('-- a | -- b'))).toEqual([]);
  });

  it('reports each unreachable rule once, naming one shadower', () => {
    // `--` shadows both `-->` and `--x`; `->` is shadowed by nothing.
    const found = findUnreachableRules(parseRules('--> → | --x y | -- ‒ | -> a'));
    expect(found).toEqual([
      { find: '-->', shadowedBy: '--' },
      { find: '--x', shadowedBy: '--' },
    ]);
  });

  it('is total on an empty list', () => {
    expect(findUnreachableRules([])).toEqual([]);
  });

  it('ignores an empty find rather than flagging every other rule as shadowed', () => {
    // parseRules drops these, so they can only arrive from a trusted caller building
    // a config by hand — but '' is a prefix of everything, so admitting it would make
    // the diagnostic claim the whole list is unreachable.
    const rules = [
      { find: '', replace: 'x' },
      { find: '-->', replace: '→' },
    ];
    expect(findUnreachableRules(rules)).toEqual([]);
  });

  it('does not throw on input that assertSafeRules rejects — it is a diagnostic, not a gate', () => {
    // `a`→`aa` is non-convergent: assertSafeRules throws on it. findUnreachableRules must
    // still answer, because it is computed separately and must never gate Apply (D-100 ④).
    const rules = parseRules('a aa | ab b');
    expect(() => assertSafeRules(rules, { trusted: false })).toThrow();
    expect(findUnreachableRules(rules)).toEqual([{ find: 'ab', shadowedBy: 'a' }]);
  });

  it('passes the shipped seed and fails its predecessor — the regression this milestone closes', () => {
    const seed = '-> → | <- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒';
    const historicalS2 = '--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒';
    expect(findUnreachableRules(parseRules(seed))).toEqual([]);
    // Positive control: the clean result above means something only because the value
    // it replaced genuinely fails.
    //
    // EXACTLY ONE entry, and that is the interesting part: `<--` was NOT broken. Its
    // only candidate shadower would be `<-`, which S2 does not contain — so `<--`
    // always worked, and it changed to `<-` for symmetry with `->`, not as a fix.
    // Asserting the whole array rather than `.toContain` is what pins that down.
    expect(findUnreachableRules(parseRules(historicalS2))).toEqual([
      { find: '-->', shadowedBy: '--' },
    ]);
  });
});
