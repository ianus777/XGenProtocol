// transform.ts — text-processor KIND 1 (transformer): the pure, DOM-free, framework-free
// core. Mirrors logic.ts in posture (no Svelte, no DOM) — the edit-side engine's wrapper
// (processor.ts) is the only framework touch. Part of the four-kind taxonomy (D-099 / N-056):
// this file holds the DOM-free pure cores: kind 1 (`string -> string`, live replace-all) and
// now kind 3 (`number|null -> number|null`, the idempotent clamp guard, M-RP4.1).
//
// SCOPE (D-065, codify-four / build-progressively): `TransformRule` (kind 1) + `ClampRule`
// (kind 3) exist in code. The union `ProcessorRule = TransformRule | ConvertRule | ClampRule |
// RenderRule` is documented in D-099, NOT declared here, so the namespace stays clean.

/** A single literal find/replace pair (kind 1). */
export type TransformRule = {
  /** Literal substring to match (NOT a regex). Empty string is invalid (see assertSafeRules). */
  find: string;
  /** Literal replacement; every occurrence of `find` is replaced. */
  replace: string;
  /**
   * DECLARED, NOT IMPLEMENTED (reserved). A pair the author certifies invertible (curated,
   * collision-free) so a future un-morph path can reverse it. Default false. No reverse path
   * is built this arc — the flag only reserves the type surface.
   */
  reversible?: boolean;
};

export type TransformConfig = TransformRule[];

/** Tier-2 (untrusted) provenance caps. Tier-1 trusted code bypasses these entirely. */
export const CAP_RULES = 100;
export const CAP_LEN = 200;

/** Thrown by assertSafeRules when an untrusted rule set violates a provenance constraint. */
export class ProcessorRuleError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ProcessorRuleError';
  }
}

/**
 * Apply `rules` to `input` left-to-right. Each rule sees the prior rule's output; each rule
 * replaces ALL literal occurrences of its `find`. Pure and total: no throw, no DOM, no I/O.
 * Literal replace-all is done split/join (regex-free) so no `find` ever needs escaping. An
 * empty `find` is a no-op (defensive — assertSafeRules rejects it for untrusted input, but
 * applyRules must stay total for trusted callers too).
 */
export function applyRules(input: string, rules: TransformConfig): string {
  let out = input;
  for (const rule of rules) {
    if (rule.find === '') continue; // never expand on an empty match
    out = out.split(rule.find).join(rule.replace); // literal replace-all, regex-free
  }
  return out;
}

/**
 * Provenance gate. Trusted (Tier-1 `common` code): always passes — full power. Untrusted
 * (Tier-2 user/settings literal pairs): enforces
 *   - count <= CAP_RULES, each find/replace length <= CAP_LEN
 *   - non-empty `find`
 *   - convergence lint: a rule whose `replace` still CONTAINS its `find` is rejected, because
 *     the engine re-runs the whole value on every keystroke, so such a pair would re-match its
 *     own output and loop (`a`->`aa`). Convergent literals (`:)`->`🙂`, `-->`->`→`) pass.
 * Untrusted regex is not representable (rules are literal strings); a regex rule-kind + its
 * ReDoS guard are reserved for an explicit advanced opt-in (D-099).
 */
export function assertSafeRules(rules: TransformConfig, opts: { trusted: boolean }): void {
  if (opts.trusted) return;
  if (rules.length > CAP_RULES) {
    throw new ProcessorRuleError(`too many rules: ${rules.length} > ${CAP_RULES}`);
  }
  for (const rule of rules) {
    if (typeof rule.find !== 'string' || typeof rule.replace !== 'string') {
      throw new ProcessorRuleError('rule find/replace must be strings');
    }
    if (rule.find === '') {
      throw new ProcessorRuleError('rule `find` must be non-empty');
    }
    if (rule.find.length > CAP_LEN || rule.replace.length > CAP_LEN) {
      throw new ProcessorRuleError(`rule string exceeds ${CAP_LEN} chars`);
    }
    if (rule.replace.includes(rule.find)) {
      throw new ProcessorRuleError(
        `non-convergent rule: replace ${JSON.stringify(rule.replace)} contains find ${JSON.stringify(rule.find)}`,
      );
    }
  }
}

/**
 * Parse one user-owned rules string into a TransformConfig (M-RP4.2 grammar, literal — no regex):
 *   - pairs are separated by the literal " | " (space-pipe-space)
 *   - within a pair, split on the FIRST space -> `find` (before) | `replace` (everything after)
 *   - `find` = any string with no whitespace; `replace` = any string at all (multi-char, emoji,
 *     internal spaces, a lone `|` e.g. ":| 😐") — the only forbidden token substring is " | " itself
 *   - a pair with no space (no separator) or an empty `find` is skipped
 * Pure and total (no throw): malformed pairs are dropped, not raised. Validation is a separate
 * concern — feed the result to assertSafeRules({trusted:false}) for the Tier-2 caps + convergence
 * lint. The inverse (config -> string) is stringifyRules, deferred to the M-RP4.3 editor.
 */
export function parseRules(text: string): TransformConfig {
  const out: TransformConfig = [];
  for (const pair of text.split(' | ')) {
    const i = pair.indexOf(' ');
    if (i < 0) continue; // no separator space -> not a pair
    const find = pair.slice(0, i);
    const replace = pair.slice(i + 1);
    if (find.length === 0) continue; // empty find -> skip (assertSafeRules would reject anyway)
    out.push({ find, replace });
  }
  return out;
}

// ── KIND 3 — filter/guard (M-RP4.1) ─────────────────────────────────────────────────────────
// Pure numeric guard: `number|null -> number|null`, idempotent. Separate from kind 1 (which is
// string->string); co-located here because both are DOM-free pure cores (the logic.ts posture).
// The change-triggered edit-side engine is the sibling attachment processor/clamp.ts.

/** A numeric clamp bound (kind 3). Both bounds optional; absent = that side is unclamped. */
export type ClampRule = {
  /** Inclusive lower bound. Omitted = no lower clamp. */
  min?: number;
  /** Inclusive upper bound. Omitted = no upper clamp. */
  max?: number;
};

/**
 * Clamp `n` into the ClampRule's [min, max]. Pure, total, IDEMPOTENT
 * (applyClamp(applyClamp(n, r), r) === applyClamp(n, r)). `null` (the empty numeric field)
 * passes through unchanged — clamping an absent value is meaningless. Each bound applies only if
 * present, so {min} / {max} / {min,max} / {} all behave. If min > max (author error) the upper
 * clamp runs last and wins; kept total rather than throwing.
 */
export function applyClamp(n: number | null, rule: ClampRule): number | null {
  if (n === null) return null;
  let out = n;
  if (rule.min !== undefined) out = Math.max(rule.min, out);
  if (rule.max !== undefined) out = Math.min(rule.max, out);
  return out;
}

// ── KIND 2 — converter/bridge (M-RP4.5) ────────────────────────────────────────
// The taxonomy's BRIDGE: `string <-> T`. Unlike kinds 1/3 (same-type in/out, shipped as a
// forwarded attachment), kind 2 has TWO representations of DIFFERENT type coexisting — a display
// string (toString, may use Intl) and a bound typed value (fromString). One `bind:value` can't
// carry both, so kind 2 is a real COMPONENT (converter-field.svelte), not an attachment. This
// file holds only the DOM-free contract + first concrete (still the logic.ts posture: no Svelte,
// no DOM — Intl is ECMA-402, not DOM; the component owns the one framework touch + the DEV hook).

/**
 * Sentinel returned by `Converter.fromString` when the text cannot be parsed into `T`. A unique
 * symbol (not `null`/`undefined`/`NaN`) so those stay usable as legitimate `T` values. The host
 * treats it as "reject-and-mark": keep the user's text, flag invalid, leave the bound value alone.
 */
export const PARSE_FAILED = Symbol('PARSE_FAILED');

/**
 * A bidirectional bridge between a typed value `T` and its editable text form (kind 2). Trusted
 * (Tier-1) code supplies this — it is LOGIC, never a user-authored string, so no provenance caps
 * or convergence lint apply (contrast kind 1's assertSafeRules).
 *   - toString    : the DISPLAY form (formatted; grouping, currency, Intl, …)
 *   - fromString   : parse text -> `T`, or PARSE_FAILED when it doesn't parse
 *   - toEditable?  : the RAW form shown while the field is focused (default = toString). Lets the
 *                    user edit "1234.56" without fighting the "1,234.56" group separators.
 */
export type Converter<T> = {
  toString(v: T): string;
  fromString(s: string): T | typeof PARSE_FAILED;
  toEditable?(v: T): string;
};

/**
 * First concrete converter: a locale-aware number bridge over `Intl.NumberFormat`. `toString`
 * formats (grouping/decimals/currency per `opts`); `toEditable` is the raw ungrouped number so
 * typing is unimpeded; `fromString` reverses the format by discovering THIS locale's group +
 * decimal glyphs via `formatToParts` (Intl has no parser), stripping the group glyph, normalising
 * the decimal glyph to '.', then `Number()` + a finite check. Empty/blank or non-finite -> PARSE_FAILED.
 * Pure + total (no throw). `T = number`.
 */
export function intlNumber(
  opts: Intl.NumberFormatOptions = {},
  locale?: string,
): Converter<number> {
  const fmt = new Intl.NumberFormat(locale, opts);
  // Discover the locale's separators from a sample that exercises both (11111.1).
  const parts = fmt.formatToParts(11111.1);
  const group = parts.find((p) => p.type === 'group')?.value ?? '';
  const decimal = parts.find((p) => p.type === 'decimal')?.value ?? '.';
  return {
    toString: (v) => fmt.format(v),
    toEditable: (v) => String(v),
    fromString: (s) => {
      const trimmed = s.trim();
      if (trimmed === '') return PARSE_FAILED;
      let normalised = trimmed;
      if (group) normalised = normalised.split(group).join(''); // strip group separators
      if (decimal !== '.') normalised = normalised.split(decimal).join('.'); // decimal -> '.'
      const n = Number(normalised);
      return Number.isFinite(n) ? n : PARSE_FAILED;
    },
  };
}
