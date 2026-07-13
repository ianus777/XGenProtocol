<script lang="ts">
  // region-placeholder — the throwaway leaf for M-RP6.1f. ONE component, mapped to the six not-yet-built
  // region ids in the registry (D6); each real region widget later replaces ONE registry entry, no rewrite.
  // Shell-local (it is disposable client chrome, not a reusable `core` component).
  //
  // M-RP7.1 (D3) — the `Section` wrapper is UNWRAPPED. The tile frame (`region-tile`) now draws the title
  // stripe, so a `Section title=…` here would be a SECOND title (Joe's "group container is the wrong main
  // form"). The body is rendered directly; it registers nothing (the leaf's registry entry is now the tile,
  // `region-tile#region-<id>`, N-096). The display names moved to `layout-default.ts` (D2 — one map, no
  // duplicate); this reads that map for its own body text, and the SAME map feeds the tile title.
  import { REGION_NAMES } from './layout-default';

  let { regionId }: { regionId: string } = $props();

  const title = $derived(REGION_NAMES[regionId] ?? regionId);
  const body = $derived(`${title} — placeholder (M-RP6.1g+)`);
</script>

<div class="region-placeholder">{body}</div>
