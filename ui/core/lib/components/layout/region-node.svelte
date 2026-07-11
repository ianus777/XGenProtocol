<script lang="ts">
  // region-node — the INTERNAL recursion part of renderer A (M-RP6.1f). NOT a catalogued component:
  // it carries NO `use:envelope`, so it registers NOTHING into window.__XGEN_DEBUG__ (the `sb-cell` /
  // N-064 opt-out). A per-node registry getter would be pure ordinal noise — the meaningful registry
  // entries are `region-shell` (one aggregate getter, §4) + each leaf's own component (`section#region-*`).
  //
  // It renders ONE resolved node:
  //   - `leaf`  → a `.region-leaf` scroll box (D5 — the leaf is the scroller) hosting the widgetId's
  //               component from the injected registry (the `message.svelte` widgets-registry shape, §0.1).
  //   - `split` → a `.region-split` flex box; `dir` rides `data-dir` (the skin reads it — flex-direction
  //               is layout = skin, N-090), and each child gets its descriptor weight as an INLINE
  //               `flex: {n} 1 0` (the ONE skin exception: `sizes[]` is DATA, so it rides inline like
  //               `--led-colour` / `--meter-fill`; gaps/mins/overflow stay in skin.css, D4).
  import type { Component } from 'svelte';
  import type { ResolvedNode } from './resolve';
  import RegionNode from './region-node.svelte'; // self-import recursion (Svelte 5; no <svelte:self>)

  let {
    node,
    widgets,
    flex,
  }: {
    node: ResolvedNode;
    widgets: Record<string, Component>;
    /** Descriptor weight from the parent split; applied inline to THIS node's root (skin exception, D4). */
    flex?: number;
  } = $props();

  const flexStyle = $derived(flex !== undefined ? `flex: ${flex} 1 0` : undefined);
</script>

{#if node.type === 'leaf'}
  {@const W = widgets[node.widgetId]}
  <div class="region-leaf" style={flexStyle}>
    {#if W}
      <W regionId={node.widgetId} />
    {/if}
  </div>
{:else}
  <div class="region-split" data-dir={node.dir} style={flexStyle}>
    {#each node.children as child, i (i)}
      <RegionNode node={child} {widgets} flex={node.sizes[i]} />
    {/each}
  </div>
{/if}
