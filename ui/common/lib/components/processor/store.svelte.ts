// store.svelte.ts — the source-agnostic substitution-rules store (M-RP4.2). Holds the user's
// ONE list of pairs as a reactive TransformConfig. Source-agnostic by design (D-099 decision 9):
// the real client feeds the raw string from xgen-client_config.toml [substitutions] via the
// Tauri command get_substitutions; the sampler seeds a literal. Either way the entry point is
// the same setRules(string). Processor-hosts read `substitutions.rules` and pass it to
// processor(), so a new string -> a new TransformConfig -> processor() re-evaluates -> the
// attachment re-attaches (the kind-1 lifecycle, D-099). A `.svelte.ts` module so module-level
// $state participates in Svelte 5 reactivity.

import { parseRules, assertSafeRules, type TransformConfig } from './transform';

let _rules = $state<TransformConfig>([]);

export const substitutions = {
  /** The live, validated rule list. Read by processor-hosts; reactive. */
  get rules(): TransformConfig {
    return _rules;
  },
  /**
   * Replace the whole list from one raw string (the TOML / one-textarea shape, M-RP4.2 grammar).
   * Parses (parseRules: pairs on " | ", first-space per pair) then validates as Tier-2 user data
   * (assertSafeRules({trusted:false}) — caps + convergence lint, so a self-authored looping pair
   * like `a aa` can't hang the field). On a rejected set the store fails safe to empty + a DEV
   * warning. (Per-pair partition + inline warnings are the M-RP4.3 editor's job.)
   */
  setRules(text: string): void {
    const parsed = parseRules(text);
    try {
      assertSafeRules(parsed, { trusted: false });
      _rules = parsed;
    } catch (e) {
      if (import.meta.env.DEV) {
        console.warn('[substitutions] rejected rule set, keeping empty:', e);
      }
      _rules = [];
    }
  },
};

// DEV-only CDP handle (mirrors __XGEN_PROC__ / envelope's DEV idiom, N-024). Lets a test drive
// setRules and read the live rules out-of-band. Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_SUBS__?: unknown }).__XGEN_SUBS__ = substitutions;
}
