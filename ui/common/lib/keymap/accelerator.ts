// accelerator.ts — the `Accelerator` value-object (M-RP6.1c). Pure, DOM-free, framework-free
// (the transform.ts / seed-colour.ts posture: no Svelte, no DOM runtime calls — only DOM *types*,
// which erase). ONE canonical definition projected two ways so display and dispatch can never
// drift (the Converter<T> "one object, two reps" shape):
//   - display  → toDisplay(platform)  : the menu-item hint ("Ctrl+Q" / "⌘Q")
//   - dispatch → matches(event, platform) : the keydown predicate
//
// Modifier model (D2, SHORTCUT abstraction, Joe-locked M-RP6.1c):
//   - `Ctrl` ≡ `Control` → the platform-ACCELERATOR token (ctrlKey on win, metaKey/⌘ on mac).
//     There is deliberately NO "literal ctrl" token — the accelerator key is always the primary.
//   - `Cmd` / `Command` / `Meta` / `Super` → literal `meta` (metaKey on every platform).
//   - `Option` → `alt`; `Alt` → `alt`; `Shift` → `shift`.
// Exact-modifier match: an accelerator fires ONLY when the required modifier set matches the event
// EXACTLY — extra modifiers held (e.g. Shift on a plain "Ctrl+Q") suppress the match. `event.key`
// is compared case-insensitively for single letters, verbatim for named keys ("F5", "ArrowLeft").
//
// Provenance: bindings are Tier-1 TRUSTED code (never user-authored strings), so parse THROWS on a
// malformed spec — fail-fast at author time (contrast the total/lenient Tier-2 processor grammar).

/** Target platform for display + dispatch. Windows is the only shipped target today; `mac` keeps
 *  the SHORTCUT abstraction correct-by-construction for a future build. Default everywhere: `win`. */
export type Platform = 'win' | 'mac';

/**
 * The structural subset of `KeyboardEvent` that `matches` reads. Typing the parameter this way
 * keeps the module DOM-free (a real `KeyboardEvent` satisfies it, and a unit test can pass a plain
 * object literal — no DOM construction needed).
 */
export type KeyEventLike = Pick<
  KeyboardEvent,
  'key' | 'ctrlKey' | 'metaKey' | 'altKey' | 'shiftKey'
>;

/** The normalized, platform-neutral shape of an accelerator. Also the programmatic constructor form. */
export interface AcceleratorSpec {
  /** The platform-accelerator modifier (Ctrl on win, ⌘ on mac). From the `Ctrl`/`Control` token. */
  accel?: boolean;
  /** Literal meta (metaKey) — from `Cmd`/`Command`/`Meta`/`Super`. Distinct from `accel`. */
  meta?: boolean;
  /** Literal alt — from `Alt`/`Option`. */
  alt?: boolean;
  /** Literal shift. */
  shift?: boolean;
  /** The main key. Single letters are normalized upper-case; named keys to their `event.key` form. */
  key: string;
}

// Friendly aliases → canonical `event.key` names, applied case-insensitively at parse time so an
// author can write "Esc" / "Del" / "Up" and still match the exact DOM key ("Escape" / "Delete" / …).
const NAMED_KEYS = new Map<string, string>([
  ['space', ' '],
  ['esc', 'Escape'],
  ['escape', 'Escape'],
  ['enter', 'Enter'],
  ['return', 'Enter'],
  ['tab', 'Tab'],
  ['del', 'Delete'],
  ['delete', 'Delete'],
  ['backspace', 'Backspace'],
  ['ins', 'Insert'],
  ['insert', 'Insert'],
  ['home', 'Home'],
  ['end', 'End'],
  ['pageup', 'PageUp'],
  ['pagedown', 'PageDown'],
  ['up', 'ArrowUp'],
  ['down', 'ArrowDown'],
  ['left', 'ArrowLeft'],
  ['right', 'ArrowRight'],
  ['arrowup', 'ArrowUp'],
  ['arrowdown', 'ArrowDown'],
  ['arrowleft', 'ArrowLeft'],
  ['arrowright', 'ArrowRight'],
]);

/** Normalize a parsed key token to its stored form (upper-case letters; canonical named keys). */
function normalizeKey(k: string): string {
  if (k.length === 1) {
    // Single char: letters upper-case (case-insensitive match later); digits/symbols verbatim.
    return /[a-z]/i.test(k) ? k.toUpperCase() : k;
  }
  const lower = k.toLowerCase();
  const named = NAMED_KEYS.get(lower);
  if (named !== undefined) return named;
  if (/^f([1-9]|1\d|2[0-4])$/.test(lower)) return 'F' + lower.slice(1); // F1..F24
  return k; // unknown multi-char key: stored verbatim, matched verbatim
}

/** True when a stored accelerator key matches an event's `key`: case-insensitive for single
 *  letters, exact for everything else (named keys, digits, symbols, space). */
function keyMatches(accKey: string, eventKey: string): boolean {
  if (accKey.length === 1 && /[a-z]/i.test(accKey)) {
    return eventKey.length === 1 && eventKey.toLowerCase() === accKey.toLowerCase();
  }
  return eventKey === accKey;
}

/**
 * A single keyboard accelerator. Construct from a normalized `AcceleratorSpec` (programmatic) or,
 * more usually, via `Accelerator.parse("Ctrl+Q")` / the `accelerator()` helper (authored form).
 */
export class Accelerator {
  readonly accel: boolean;
  readonly meta: boolean;
  readonly alt: boolean;
  readonly shift: boolean;
  readonly key: string;

  constructor(spec: AcceleratorSpec) {
    if (typeof spec.key !== 'string' || spec.key.length === 0) {
      throw new Error('Accelerator: `key` must be a non-empty string');
    }
    this.accel = spec.accel ?? false;
    this.meta = spec.meta ?? false;
    this.alt = spec.alt ?? false;
    this.shift = spec.shift ?? false;
    this.key = normalizeKey(spec.key);
  }

  /**
   * Parse an authored accelerator string ("Ctrl+Shift+S", "F5", "Cmd+K") into an `Accelerator`.
   * Tokens are `+`-separated, trimmed, case-insensitive; every token but the last must be a known
   * modifier alias, the last is the key. THROWS on: empty spec, an empty token (e.g. "Ctrl++"),
   * an unknown modifier before the key, or more than one key token — all author errors (Tier-1).
   */
  static parse(spec: string): Accelerator {
    if (typeof spec !== 'string') {
      throw new Error('Accelerator.parse: spec must be a string');
    }
    const trimmed = spec.trim();
    if (trimmed.length === 0) {
      throw new Error('Accelerator.parse: empty accelerator');
    }
    const tokens = trimmed.split('+').map((t) => t.trim());
    const out: AcceleratorSpec = { key: '' };
    let keySet = false;
    for (const tok of tokens) {
      if (tok.length === 0) {
        throw new Error(`Accelerator.parse: empty token in "${spec}"`);
      }
      const lower = tok.toLowerCase();
      switch (lower) {
        case 'ctrl':
        case 'control':
          out.accel = true;
          break;
        case 'cmd':
        case 'command':
        case 'meta':
        case 'super':
          out.meta = true;
          break;
        case 'alt':
        case 'option':
          out.alt = true;
          break;
        case 'shift':
          out.shift = true;
          break;
        default:
          if (keySet) {
            throw new Error(`Accelerator.parse: multiple keys in "${spec}"`);
          }
          out.key = tok;
          keySet = true;
      }
    }
    if (!keySet) {
      throw new Error(`Accelerator.parse: no key in "${spec}" (modifiers only)`);
    }
    return new Accelerator(out);
  }

  /** Does this accelerator fire for `event` on `platform`? Exact-modifier match (see file header). */
  matches(event: KeyEventLike, platform: Platform = 'win'): boolean {
    // On mac the accelerator token IS meta; on win it is ctrl. A literal `meta` is metaKey on both.
    const wantCtrl = platform === 'mac' ? false : this.accel;
    const wantMeta = platform === 'mac' ? this.accel || this.meta : this.meta;
    return (
      event.ctrlKey === wantCtrl &&
      event.metaKey === wantMeta &&
      event.altKey === this.alt &&
      event.shiftKey === this.shift &&
      keyMatches(this.key, event.key)
    );
  }

  /** The display hint for a menu-item: "Ctrl+Shift+S" (win) / "⇧⌘S" (mac). */
  toDisplay(platform: Platform = 'win'): string {
    if (platform === 'mac') {
      // Apple HIG order: Control, Option, Shift, Command, then key — symbols, no separator.
      let s = '';
      if (this.alt) s += '⌥';
      if (this.shift) s += '⇧';
      if (this.accel || this.meta) s += '⌘';
      return s + this.displayKey();
    }
    const parts: string[] = [];
    if (this.accel) parts.push('Ctrl');
    if (this.meta) parts.push('Win');
    if (this.alt) parts.push('Alt');
    if (this.shift) parts.push('Shift');
    parts.push(this.displayKey());
    return parts.join('+');
  }

  /** The canonical, platform-neutral string form — used by the registry to dedup bindings. */
  toString(): string {
    const m: string[] = [];
    if (this.accel) m.push('accel');
    if (this.meta) m.push('meta');
    if (this.alt) m.push('alt');
    if (this.shift) m.push('shift');
    m.push(this.key);
    return m.join('+');
  }

  private displayKey(): string {
    return this.key === ' ' ? 'Space' : this.key;
  }
}

/** Convenience factory — `accelerator("Ctrl+Q")` is `Accelerator.parse("Ctrl+Q")`. */
export function accelerator(spec: string): Accelerator {
  return Accelerator.parse(spec);
}
