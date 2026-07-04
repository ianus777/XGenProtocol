// clamp.ts — text-processor KIND 3 (filter/guard) edit-side ENGINE: a forwarded Svelte 5
// ATTACHMENT, the change-triggered sibling of processor.ts (kind 1). Kind 1 fires on `input`
// with caret restore; kind 3 fires on `change` (commit) and coerces the numeric value into
// [min,max] — no caret concern (the whole value is replaced, not spliced). The pure core is
// applyClamp in transform.ts; this file is the only framework touch (the createAttachmentKey
// spread the consumer forwards onto number.svelte's inner <input>). D-099 kind 3 / N-056.
//
// Reactivity = the attachment lifecycle: a new {min,max} -> a new clamp() object -> Svelte
// removes the old listener and re-attaches.

import { createAttachmentKey } from 'svelte/attachments';
import { applyClamp, type ClampRule } from './transform';

// DEV-only pure-core hook for CDP verification (mirrors __XGEN_PROC__, N-024). Dead-code-
// eliminated + tree-shaken in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_CLAMP__?: unknown }).__XGEN_CLAMP__ = { applyClamp };
}

/**
 * Build a forwardable clamp attachment for `rule` ({min?, max?}). Fires on `change` (commit,
 * not per-keystroke — clamping mid-type is hostile), coerces the field's numeric value into
 * range via applyClamp, writes it back, and dispatches a synthetic `input` so Svelte's
 * `bind:value` syncs. Re-entrancy-guarded against its own dispatch. An empty or unparseable
 * field is left alone (empty = null; applyClamp(null) = null).
 */
export function clamp(rule: ClampRule) {
  const attach = (node: HTMLInputElement) => {
    let reentrant = false;
    const onChange = () => {
      if (reentrant) return;
      if (node.value === '') return; // empty field = null; nothing to clamp
      const n = node.valueAsNumber; // NaN if unparseable
      if (Number.isNaN(n)) return;
      const next = applyClamp(n, rule);
      if (next === n) return; // already in range -> no write, no churn
      reentrant = true;
      node.value = String(next);
      node.dispatchEvent(new Event('input', { bubbles: true })); // sync Svelte bind:value
      reentrant = false;
    };
    node.addEventListener('change', onChange);
    return () => node.removeEventListener('change', onChange);
  };
  return { [createAttachmentKey()]: attach };
}
