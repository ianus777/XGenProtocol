// registry.ts — the `KeymapRegistry` (M-RP6.1c). The PURE half of the keymap machine (D3, split
// confirmed): a binding table + `resolve(event) → commandId | null`. No DOM. The shell (M-RP6.1d)
// owns the singleton instance, the binding population, and the single `window.addEventListener
// ('keydown', …)` that calls `resolve` — that DOM touch is the only shell-side keymap code.
//
// This lives in `$common` (not the shell) because it is generic + unit-testable, and BOTH shells
// (client + node) will keymap. Only the app-specific bindings and the listener are shell-local.

import { Accelerator, type KeyEventLike, type Platform } from './accelerator';

/** An opaque command identifier the shell maps to an action (e.g. "app.exit"). */
export type CommandId = string;

/** One accelerator → command entry. */
export interface Binding {
  accelerator: Accelerator;
  command: CommandId;
}

/**
 * A table of accelerator → command bindings with an exact-match resolver. Bindings are deduped on
 * the accelerator's canonical string (`Accelerator.toString()`); registering a second binding for
 * an already-bound accelerator THROWS (author error, Tier-1 fail-fast — the same posture as
 * `Accelerator.parse`). `resolve` walks bindings in registration order, first match wins (order is
 * only theoretically observable: distinct accelerators can't both exact-match one event).
 */
export class KeymapRegistry {
  private readonly bindings: Binding[] = [];
  private readonly canonical = new Set<string>();

  /** Bind `accelerator` to `command`. THROWS if that accelerator is already bound. */
  register(accelerator: Accelerator, command: CommandId): void {
    const key = accelerator.toString();
    if (this.canonical.has(key)) {
      throw new Error(`KeymapRegistry: accelerator "${accelerator.toDisplay()}" is already bound`);
    }
    this.canonical.add(key);
    this.bindings.push({ accelerator, command });
  }

  /** The command bound to the first accelerator matching `event` on `platform`, or `null`. */
  resolve(event: KeyEventLike, platform: Platform = 'win'): CommandId | null {
    for (const b of this.bindings) {
      if (b.accelerator.matches(event, platform)) return b.command;
    }
    return null;
  }

  /** Whether an accelerator (by canonical form) is already bound. */
  has(accelerator: Accelerator): boolean {
    return this.canonical.has(accelerator.toString());
  }

  /** The number of registered bindings. */
  get size(): number {
    return this.bindings.length;
  }

  /** A snapshot of the current bindings (registration order). */
  list(): readonly Binding[] {
    return this.bindings.slice();
  }
}
