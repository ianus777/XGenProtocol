<script lang="ts">
  // region-node — the INTERNAL recursion part of renderer A (M-RP6.1f). NOT a catalogued component:
  // it carries NO `use:envelope`, so it registers NOTHING into window.__XGEN_DEBUG__ (the `sb-cell` /
  // N-064 opt-out). The meaningful registry entries are `region-shell` (one aggregate getter, §4) + each
  // leaf's own `region-tile` (M-RP7.1) + the widget the tile hosts.
  //
  // It renders ONE resolved node:
  //   - `leaf`  → a `region-tile` (M-RP7.1) framing the widgetId's component from the injected registry.
  //               The tile owns the chrome (stripe · fold · dead grips) and the scroll box (D5, ex-leaf).
  //   - `split` → a `.region-split` flex box; `dir` rides `data-dir` (the skin reads it — flex-direction
  //               is layout = skin, N-090), and each child gets its descriptor weight as an INLINE
  //               `flex: {n} 1 0` (the ONE skin exception: `sizes[]` is DATA — now applied by the tile
  //               itself for a leaf, still here for a nested split; gaps/mins/overflow stay in skin.css).
  //
  // AXIS (M-RP7.1, D8): a split passes its OWN `dir` down to each child as `axis` — the parent split's
  // direction, which is the axis a leaf collapses along. A leaf reflects it as the tile's `data-axis`.
  // The root node (called from region-shell with no `axis`) defaults to `col` (a top-level degenerate
  // leaf collapses to a horizontal stripe — the common case).
  import type { Component } from 'svelte';
  import type { ResolvedNode } from './resolve';
  import type { FoldAxis } from './types';
  import RegionNode from './region-node.svelte'; // self-import recursion (Svelte 5; no <svelte:self>)
  import RegionTile from './region-tile.svelte';

  let {
    node,
    widgets,
    titles = {},
    onFold,
    axis,
    flex,
  }: {
    node: ResolvedNode;
    widgets: Record<string, Component>;
    /** widgetId → display title (D2); resolved by the shell from CLIENT_PLUGINS + REGION_NAMES. */
    titles?: Record<string, string>;
    /** Fold toggle seam threaded to every tile (D6). `undefined` collapsed ⇒ unfold. */
    onFold?: (regionId: string, collapsed: FoldAxis | undefined) => void;
    /** The PARENT split's `dir` — the axis a leaf collapses along/across (§4.1). Undefined at the root. */
    axis?: 'row' | 'col';
    /** Descriptor weight from the parent split; applied inline (skin exception, D4). */
    flex?: number;
  } = $props();

  // Shrink-wrap (§4.4): a split whose children are ALL folded ACROSS its own axis takes `flex: 0 0 auto`
  // (the inline weight is omitted so the skin's `[data-shrinkwrap]` rule wins), so it collapses to its
  // children's strip size and the parent's SIBLINGS absorb the freed weight — the same mechanic a folded
  // leaf uses, one rule for two node types. DERIVED, never stored: a stored flag would go stale the instant
  // one child unfolds (D-067 in miniature). A `col` split's across-axis is width; a `row` split's is height.
  // ⚠️ SCOPE: `every` requires the children to be LEAVES — a split of nested splits does NOT shrink-wrap
  // this leg (recursive shrink-wrap is §4.4's filed open mechanic). Mixed folds do not fire either → a hole
  // opens and the raster explains it (both intentional).
  const shrinkWrap = $derived.by(() => {
    if (node.type !== 'split' || node.children.length === 0) return false;
    const across: FoldAxis = node.dir === 'col' ? 'width' : 'height';
    return node.children.every((c) => c.type === 'leaf' && c.collapsed === across);
  });

  // Omit the inline weight when shrink-wrapping so the skin's `flex: 0 0 auto` takes over (else an inline
  // `flex: {n} 1 0` would out-specify it and the split would keep its old width — the leaf-fold precedent).
  const flexStyle = $derived(
    shrinkWrap ? undefined : flex !== undefined ? `flex: ${flex} 1 0` : undefined,
  );
</script>

{#if node.type === 'leaf'}
  <RegionTile
    regionId={node.widgetId}
    title={titles[node.widgetId] ?? node.widgetId}
    collapsed={node.collapsed}
    axis={axis ?? 'col'}
    {flex}
    {onFold}
  >
    {#if widgets[node.widgetId]}
      {@const W = widgets[node.widgetId]}
      <W regionId={node.widgetId} />
    {/if}
  </RegionTile>
{:else}
  <div class="region-split" data-dir={node.dir} data-shrinkwrap={shrinkWrap || undefined} style={flexStyle}>
    {#each node.children as child, i (i)}
      <RegionNode node={child} {widgets} {titles} {onFold} axis={node.dir} flex={node.sizes[i]} />
    {/each}
  </div>
{/if}
