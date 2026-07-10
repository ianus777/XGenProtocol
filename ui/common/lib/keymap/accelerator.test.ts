// accelerator.test.ts — pure unit suite for the Accelerator value-object (M-RP6.1c verify leg,
// D4-V1). Runs under vitest from the sampler package (`npm test`). No DOM: `matches` is fed plain
// KeyEventLike literals.

import { describe, it, expect } from 'vitest';
import { Accelerator, accelerator, type KeyEventLike } from './accelerator';

/** Build a KeyEventLike, modifiers default to false. */
function ev(key: string, mods: Partial<Omit<KeyEventLike, 'key'>> = {}): KeyEventLike {
  return { key, ctrlKey: false, metaKey: false, altKey: false, shiftKey: false, ...mods };
}

describe('Accelerator.parse — modifiers', () => {
  it('parses a plain accelerator', () => {
    const a = accelerator('Ctrl+Q');
    expect(a.accel).toBe(true);
    expect(a.meta).toBe(false);
    expect(a.alt).toBe(false);
    expect(a.shift).toBe(false);
    expect(a.key).toBe('Q');
  });

  it('is case-insensitive on tokens', () => {
    const a = accelerator('cTRl+shIFT+s');
    expect(a.accel).toBe(true);
    expect(a.shift).toBe(true);
    expect(a.key).toBe('S');
  });

  it('treats Ctrl and Control as the same platform-accel token', () => {
    expect(accelerator('Control+Q').toString()).toBe(accelerator('Ctrl+Q').toString());
  });

  it('maps Cmd/Command/Meta/Super to literal meta (not accel)', () => {
    for (const tok of ['Cmd', 'Command', 'Meta', 'Super']) {
      const a = accelerator(`${tok}+K`);
      expect(a.meta).toBe(true);
      expect(a.accel).toBe(false);
    }
  });

  it('maps Alt and Option to alt', () => {
    expect(accelerator('Alt+F4').alt).toBe(true);
    expect(accelerator('Option+F4').alt).toBe(true);
  });

  it('carries every modifier together', () => {
    const a = accelerator('Ctrl+Alt+Shift+Meta+P');
    expect([a.accel, a.alt, a.shift, a.meta]).toEqual([true, true, true, true]);
    expect(a.key).toBe('P');
  });
});

describe('Accelerator.parse — keys', () => {
  it('upper-cases single letters', () => {
    expect(accelerator('Ctrl+q').key).toBe('Q');
  });

  it('keeps function keys canonical', () => {
    expect(accelerator('F5').key).toBe('F5');
    expect(accelerator('f12').key).toBe('F12');
  });

  it('canonicalizes named-key aliases to event.key form', () => {
    expect(accelerator('Esc').key).toBe('Escape');
    expect(accelerator('Ctrl+Del').key).toBe('Delete');
    expect(accelerator('Up').key).toBe('ArrowUp');
    expect(accelerator('Space').key).toBe(' ');
  });

  it('accepts the struct constructor form', () => {
    const a = new Accelerator({ accel: true, shift: true, key: 's' });
    expect(a.key).toBe('S');
    expect(a.toString()).toBe(accelerator('Ctrl+Shift+S').toString());
  });
});

describe('Accelerator.parse — throws (Tier-1 fail-fast)', () => {
  it('throws on empty spec', () => {
    expect(() => accelerator('')).toThrow();
    expect(() => accelerator('   ')).toThrow();
  });

  it('throws on an empty token', () => {
    expect(() => accelerator('Ctrl++')).toThrow();
  });

  it('throws on modifiers only', () => {
    expect(() => accelerator('Ctrl+Shift')).toThrow();
  });

  it('throws on two keys', () => {
    expect(() => accelerator('Ctrl+Q+R')).toThrow();
  });

  it('throws on an empty struct key', () => {
    expect(() => new Accelerator({ accel: true, key: '' })).toThrow();
  });
});

describe('Accelerator.matches — exact modifier match (win)', () => {
  it('matches the exact chord', () => {
    expect(accelerator('Ctrl+Q').matches(ev('q', { ctrlKey: true }))).toBe(true);
  });

  it('is case-insensitive for letters', () => {
    expect(accelerator('Ctrl+Q').matches(ev('Q', { ctrlKey: true }))).toBe(true);
  });

  it('does NOT match when an extra modifier is held', () => {
    expect(accelerator('Ctrl+Q').matches(ev('q', { ctrlKey: true, shiftKey: true }))).toBe(false);
  });

  it('does NOT match when a required modifier is missing', () => {
    expect(accelerator('Ctrl+Q').matches(ev('q'))).toBe(false);
  });

  it('does NOT match a different key', () => {
    expect(accelerator('Ctrl+Q').matches(ev('w', { ctrlKey: true }))).toBe(false);
  });

  it('matches named keys verbatim', () => {
    expect(accelerator('F5').matches(ev('F5'))).toBe(true);
    expect(accelerator('F5').matches(ev('f5'))).toBe(false);
  });

  it('literal meta requires metaKey on win, not ctrl', () => {
    const a = accelerator('Meta+K');
    expect(a.matches(ev('k', { metaKey: true }))).toBe(true);
    expect(a.matches(ev('k', { ctrlKey: true }))).toBe(false);
  });
});

describe('Accelerator.matches — platform mac (SHORTCUT abstraction)', () => {
  it('accel maps to metaKey on mac, ctrl is rejected', () => {
    const a = accelerator('Ctrl+Q');
    expect(a.matches(ev('q', { metaKey: true }), 'mac')).toBe(true);
    expect(a.matches(ev('q', { ctrlKey: true }), 'mac')).toBe(false);
  });
});

describe('Accelerator.toDisplay', () => {
  it('renders the Windows hint', () => {
    expect(accelerator('Ctrl+Q').toDisplay()).toBe('Ctrl+Q');
    expect(accelerator('Ctrl+Shift+S').toDisplay('win')).toBe('Ctrl+Shift+S');
  });

  it('renders the mac hint with symbols', () => {
    expect(accelerator('Ctrl+Q').toDisplay('mac')).toBe('⌘Q');
    expect(accelerator('Ctrl+Shift+S').toDisplay('mac')).toBe('⇧⌘S');
  });

  it('renders Space as a word', () => {
    expect(accelerator('Ctrl+Space').toDisplay()).toBe('Ctrl+Space');
  });
});

describe('Accelerator.toString — canonical / dedup', () => {
  it('is stable and platform-neutral', () => {
    expect(accelerator('Shift+Ctrl+A').toString()).toBe('accel+shift+A');
  });
});
