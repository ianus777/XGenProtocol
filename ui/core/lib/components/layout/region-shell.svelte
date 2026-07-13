<script lang="ts">
  // region-shell — renderer A (M-RP6.1f): the recursive config-grid renderer for the client centre.
  // Reads the D-103 `Layout` descriptor + a `widgets` registry (widgetId → component), resolves the tree
  // (pure walk, `./resolve`), and tiles the resolved leaves into the centre. The FIRST step of the widget
  // grid; renderer B (the owned dock engine, M-RP7) is a renderer UPGRADE reading the SAME descriptor.
  //
  // `core` tier (D4): the NODE app inherits this at M-RP7.x, so — like `menu-bar`/`status-bar` — it lives
  // in the reference lib and imports NO Tauri, NO protocol. The descriptor and the leaf components are
  // injected by the shell.
  //
  // Registers exactly ONE getter G (§4). `leafCount`/`widgetIds` are the RESOLVED, RENDERED truth (the
  // `message.detailsCount` precedent — the getter reports what mounted, not what was intended), which is
  // what makes a drop CDP-provable. `region-node` (the recursion) registers nothing; each leaf's own
  // component self-registers under its own id (`section#region-spaces`, …).
  //
  // Zero component-local CSS (N-090): flex direction, gaps, tracks, overflow, mins — ALL in skin.css keyed
  // by `.region-*`. The one non-appearance inline is a split child's `flex` weight (DATA, region-node).
  import { envelope } from '$common/components/base/envelope';
  import type { Component } from 'svelte';
  import type { FoldAxis, Layout } from './types';
  import { resolveLayout, treeDepth } from './resolve';
  import RegionNode from './region-node.svelte';

  let {
    layout,
    widgets = {},
    titles = {},
    onFold,
    id,
  }: {
    layout: Layout;
    widgets?: Record<string, Component>; // widgetId → component; an unknown id is dropped by resolveLayout (W-13)
    titles?: Record<string, string>; // widgetId → tile title (M-RP7.1, D2); threaded to each tile
    onFold?: (regionId: string, collapsed: FoldAxis | undefined) => void; // fold seam (M-RP7.1b, D6)
    id?: string;
  } = $props();

  const knownIds = $derived(new Set(Object.keys(widgets)));
  const resolved = $derived(resolveLayout(layout, knownIds));

  // Aggregate getter G — resolved (rendered) truth, so a drop lowers leafCount and a tabs node raises
  // unsupportedCount, both CDP-observable.
  const debug = () => ({
    version: layout?.version ?? null,
    leafCount: resolved.leafIds.length,
    widgetIds: resolved.leafIds,
    droppedCount: resolved.dropped.length,
    unsupportedCount: resolved.unsupported,
    depth: treeDepth(resolved.root),
  });
</script>

<div class="region-shell" use:envelope={{ name: 'region-shell', id, debug }}>
  {#if resolved.root}
    <RegionNode node={resolved.root} {widgets} {titles} {onFold} />
  {/if}
</div>
