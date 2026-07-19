// Widget-mount resolution — the one rule, extracted (M-RP6.9, D-3). Pure + unit-testable (the
// `stream/grouping.ts` / `layout/resolve.ts` precedent): no Svelte, no `$derived`, no DOM.
//
// Until M-RP6.9 this logic was written THREE times, character-identical bar the registry variable —
// `message.details`, `message-stream.background`, `region-shell.background` (the grid backdrop,
// M-RP-PLATE) — and had zero tests. A fourth call site (`message.bodyExtras`) lands in the same
// milestone, which is what made a shared function worth more than a third copy.
//
// NOT a match, recorded so a grep returning four does not have to be re-derived: `region-node`
// resolves a SINGLE mount by `node.widgetId` behind an inline `{#if}` — no list, no key, no props
// spread, no filter. Different rule; it stays where it is.
//
// THE FENCE (§0): this module renders nothing and knows nothing about what a mount MEANS. It maps
// declared ids to components and drops what it cannot resolve. A reaction, a send-status led and a
// wallpaper are all the same thing here, deliberately.

import type { Component } from 'svelte';
import type { WidgetMount } from './types';

/** One mount that RESOLVED against the registry. Unresolvable mounts never reach this shape. */
export interface ResolvedMount {
  /** Svelte `{#each}` key. Stable across runtime add/remove iff the consumer supplied `mountKey`. */
  key: string;
  /** Envelope id handed to the mount (D-1). `undefined` when the caller passed no `idPrefix`. */
  id: string | undefined;
  component: Component;
  props: Record<string, unknown>;
}

/** Pre-filter shape — `component` is still possibly-undefined here (an unresolved id). */
type CandidateMount = Omit<ResolvedMount, 'component'> & { component: Component | undefined };

/**
 * Resolve declared mounts against a consumer-supplied registry.
 *
 * - **Unknown `widgetId` ⇒ DROPPED** (W-13 reconcile). A stale or uninstalled widget never crashes
 *   the host; the resolved list is therefore the RENDER truth, which is what makes every
 *   `…MountCount` getter a drop-unknown proof.
 * - **`key`** = `mountKey ?? `${widgetId}-${i}`` — the fallback is byte-identical to the legacy
 *   index-composed key, so every existing mount keeps its current key and current behaviour.
 *   ⚠️ `i` is the DECLARED index, taken before the filter: dropping an unknown mount does NOT
 *   renumber its survivors. That is today's behaviour and it is preserved deliberately.
 * - **`id`** = `idPrefix + key`, or `undefined` when no prefix is passed. The caller owns the
 *   namespace (`cid('d-')` / `cid('x-')` / `cid('bg-')`); this module never invents one.
 *
 * ⚠️ Reactivity (§1.3, and D-119 paid for it once already): a caller wrapping this in `$derived`
 * re-derives on the `widgets` REFERENCE. A consumer mutating the registry object in place gets
 * nothing — reassign a fresh object.
 */
export function resolveMounts(
  mounts: WidgetMount[] | undefined,
  widgets: Record<string, Component>,
  idPrefix?: string,
): ResolvedMount[] {
  return (mounts ?? [])
    .map((m, i): CandidateMount => {
      const key = m.mountKey ?? `${m.widgetId}-${i}`;
      return {
        key,
        id: idPrefix ? `${idPrefix}${key}` : undefined,
        component: widgets[m.widgetId],
        props: m.props ?? {},
      };
    })
    .filter((r): r is ResolvedMount => !!r.component);
}
