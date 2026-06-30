// processor.ts — text-processor KIND 1 edit-side ENGINE: a forwarded Svelte 5 ATTACHMENT
// (P-1a). The ONLY framework touch in the processor folder (transform.ts/configs.ts stay
// framework-free). A `use:` action can't be forwarded from a consumer onto an atomic's
// internal element, so the engine ships as an attachment keyed by createAttachmentKey():
// the consumer spreads `{...processor(rules)}` and the atomic spreads `{...rest}` onto its
// root, landing the attachment on the inner <input>/<textarea>. The atomic carries NO
// processing logic — only the generic spread (ready, not containing — D-065).
//
// Reactivity = the attachment lifecycle: a new `rules` array -> a new processor() object ->
// Svelte cleans up (removeEventListener) and re-attaches.

import { createAttachmentKey } from 'svelte/attachments';
import { applyRules, assertSafeRules, parseRules, type TransformConfig } from './transform';

// DEV-only pure-core hook for CDP verification (mirrors envelope's import.meta.env.DEV idiom,
// N-024): exposes the framework-free functions so a test can call them directly. Dead-code-
// eliminated + tree-shaken in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_PROC__?: unknown }).__XGEN_PROC__ = {
    applyRules,
    assertSafeRules,
    parseRules,
  };
}

type EditEl = HTMLInputElement | HTMLTextAreaElement;

/**
 * Build a forwardable edit-side transform attachment for `rules`.
 *
 * @param rules ordered kind-1 transform rules (order-significant; sequential).
 * @param opts.trusted Tier-1 code rules bypass the provenance gate; Tier-2 (settings/user)
 *   rules default to untrusted and are validated (caps + convergence lint). Default false.
 *
 * Validation runs HERE (at processor() call = render time): an invalid untrusted rule set
 * throws ProcessorRuleError before the attachment is ever applied.
 */
export function processor(rules: TransformConfig, opts: { trusted?: boolean } = {}) {
  const trusted = opts.trusted ?? false;
  assertSafeRules(rules, { trusted });

  const attach = (node: EditEl) => {
    let reentrant = false;
    const onInput = () => {
      if (reentrant) return; // ignore our own synthetic re-dispatch (P-4)
      const before = node.value;
      const next = applyRules(before, rules);
      if (next === before) return; // no morph -> no caret churn, no synthetic input

      // Caret restore (P-4): the new caret is where the text BEFORE the old caret lands after
      // transformation — i.e. the transformed-prefix length. This is exactly "old caret + net
      // length-delta of replacements before the caret". Holds for the dominant case (the user
      // just completed a token AT the caret, so the token is wholly inside the prefix). A token
      // straddling the caret is the documented limitation (N-056); CDP can't drive :focus+caret,
      // so the caret behaviour is screenshot-verified, not CDP-asserted.
      const caretBefore = node.selectionStart ?? before.length;
      const caret = applyRules(before.slice(0, caretBefore), rules).length;

      reentrant = true;
      node.value = next;
      node.setSelectionRange(caret, caret);
      node.dispatchEvent(new Event('input', { bubbles: true })); // sync Svelte bind:value
      reentrant = false;
    };
    node.addEventListener('input', onInput);
    return () => node.removeEventListener('input', onInput);
  };

  return { [createAttachmentKey()]: attach };
}
