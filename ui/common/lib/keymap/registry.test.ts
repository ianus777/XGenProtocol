// registry.test.ts — pure unit suite for KeymapRegistry (M-RP6.1c verify leg, D4-V1).

import { describe, it, expect } from 'vitest';
import { accelerator, type KeyEventLike } from './accelerator';
import { KeymapRegistry } from './registry';

function ev(key: string, mods: Partial<Omit<KeyEventLike, 'key'>> = {}): KeyEventLike {
  return { key, ctrlKey: false, metaKey: false, altKey: false, shiftKey: false, ...mods };
}

describe('KeymapRegistry.resolve', () => {
  it('resolves a bound accelerator to its command', () => {
    const km = new KeymapRegistry();
    km.register(accelerator('Ctrl+Q'), 'app.exit');
    expect(km.resolve(ev('q', { ctrlKey: true }))).toBe('app.exit');
  });

  it('returns null when nothing matches', () => {
    const km = new KeymapRegistry();
    km.register(accelerator('Ctrl+Q'), 'app.exit');
    expect(km.resolve(ev('w', { ctrlKey: true }))).toBeNull();
    expect(km.resolve(ev('q'))).toBeNull();
  });

  it('honours exact-modifier match through the registry', () => {
    const km = new KeymapRegistry();
    km.register(accelerator('Ctrl+Q'), 'app.exit');
    expect(km.resolve(ev('q', { ctrlKey: true, shiftKey: true }))).toBeNull();
  });

  it('routes distinct accelerators to distinct commands', () => {
    const km = new KeymapRegistry();
    km.register(accelerator('Ctrl+Q'), 'app.exit');
    km.register(accelerator('Ctrl+Shift+Q'), 'app.force-quit');
    expect(km.resolve(ev('q', { ctrlKey: true }))).toBe('app.exit');
    expect(km.resolve(ev('q', { ctrlKey: true, shiftKey: true }))).toBe('app.force-quit');
  });

  it('passes the platform through to matching', () => {
    const km = new KeymapRegistry();
    km.register(accelerator('Ctrl+Q'), 'app.exit');
    expect(km.resolve(ev('q', { metaKey: true }), 'mac')).toBe('app.exit');
    expect(km.resolve(ev('q', { metaKey: true }), 'win')).toBeNull();
  });
});

describe('KeymapRegistry — bookkeeping', () => {
  it('throws when the same accelerator is bound twice', () => {
    const km = new KeymapRegistry();
    km.register(accelerator('Ctrl+Q'), 'app.exit');
    expect(() => km.register(accelerator('Ctrl+Q'), 'app.other')).toThrow();
    // A case/order variant is the same canonical accelerator → also a duplicate.
    expect(() => km.register(accelerator('ctrl+q'), 'app.other')).toThrow();
  });

  it('reports has() and size', () => {
    const km = new KeymapRegistry();
    expect(km.size).toBe(0);
    expect(km.has(accelerator('Ctrl+Q'))).toBe(false);
    km.register(accelerator('Ctrl+Q'), 'app.exit');
    expect(km.size).toBe(1);
    expect(km.has(accelerator('Ctrl+Q'))).toBe(true);
  });

  it('lists bindings in registration order', () => {
    const km = new KeymapRegistry();
    km.register(accelerator('Ctrl+Q'), 'app.exit');
    km.register(accelerator('F1'), 'app.help');
    expect(km.list().map((b) => b.command)).toEqual(['app.exit', 'app.help']);
  });
});
