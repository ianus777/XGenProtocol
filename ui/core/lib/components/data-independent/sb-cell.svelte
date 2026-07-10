<script lang="ts">
  // sb-cell — an INTERNAL layout part of `status-bar` (M-RP6.1e-A). NOT a separately
  // catalogued component: it carries NO `use:envelope`, so it does NOT register into
  // window.__XGEN_DEBUG__ (the N-064 used-internally-without-registration pattern —
  // chip's `register` opt-out precedent). Reasoning: it is a value-LESS flex group whose
  // entire contribution is CSS positioning (`.sb-cell[data-side]`); it owns no state and
  // interprets no data, so a registry getter would be pure ordinal noise. It exists as its
  // own file only to keep `status-bar` readable and to make the "a cell hosts any display
  // component" contract concrete (via a `children` snippet), NOT to add a matrix row.
  //
  // A cell stacks its children horizontally on one SIDE of the bar. `side` reflects to
  // `data-side` (the skin hook: left group flows from the start, right group is pushed to
  // the end via margin-left:auto). The inner display components (status-indicator / label /
  // separator / the resize-grip) each own their own look; the cell owns only positioning.
  import type { Snippet } from 'svelte';

  let {
    side = 'left',
    children,
  }: {
    side?: 'left' | 'right';
    children?: Snippet;
  } = $props();
</script>

<span class="sb-cell" data-side={side}>
  {@render children?.()}
</span>

<style></style>
