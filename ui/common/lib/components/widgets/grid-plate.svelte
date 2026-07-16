<script lang="ts">
  // grid-plate — the grid backdrop, promoted from a CSS paint to an ELEMENT (M-RP-PLATE, the
  // containment/mount exemplar). It is the FIRST `surface: 'none'` widget WITH a component: content the
  // shell mounts into a named host socket — here the grid-wide background socket on `region-shell` — the
  // exact `message-stream` `background?: WidgetMount[]` model, one level up (N-131).
  //
  // `kind: system` (W-13, non-removable) · `delivery: compiled` — the PROMOTED dev raster (dock-engine
  // Phase-0 §4.5.1: "the dev raster is promoted, not deleted… the first system plate widget, so the socket
  // ships FED", D-065/N-091). It shows through every hole/seam/perimeter (the tiles are opaque and paint
  // above; the shell's gap surface shows wherever no tile covers it) and NEVER captures the pointer — the
  // `.region-backdrop` wrapper is `pointer-events: none` (D-116: a clickable hole would have an ADDRESS and
  // retire the tree; a reactive backdrop is fine, a clickable one is not).
  //
  // INERT this arc (D3): one fixed appearance, no data source. `backgroundLive` is ACCEPTED and IGNORED —
  // a static object ignores the settings switch (the `message-stream` contract) — but the getter reads it
  // back so the switch is provably threaded even while inert. The live-switchable backdrop (solid-black →
  // reactive fractal clouds) + its user setting ride M-RP-SETTINGS, unchanged by this milestone.
  //
  // W-3 holds: imports only `$common` (`envelope`) — no shell / Tauri / protocol dep. Zero component-local
  // CSS (N-090): the raster look is `.grid-plate` in skin.css (PROVISIONAL → M-RP-SKIN, D4).
  import { envelope } from '$common/components/base/envelope';

  // `id` self-defaults to 'grid-plate' (the self-panel `id = region-${regionId}` self-derivation shape),
  // so the plate registers as a STABLE, enumerable `grid-plate#grid-plate` even when mounted from a
  // props-less `WidgetMount` — not a mount-order ordinal (envelope's `id ?? ++ordinal` fallback).
  let { backgroundLive, id = 'grid-plate' }: { backgroundLive?: boolean; id?: string } = $props();

  // G — proves the settings switch reaches the mount even while the plate ignores it (V4).
  const debug = () => ({ backgroundLive });
</script>

<div class="grid-plate" use:envelope={{ name: 'grid-plate', id, debug }}></div>
