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
  // B2 (M-RP-SETTINGS Leg C): the plate STOPS BEING FULLY INERT. It reads ONE value from the $common backdrop
  // store and branches its render (`data-pattern`), so the setting's writer is proven on the PAINTED DOM
  // (N-097 — a setting that moves nothing on screen is an untested writer), not by a getter alone. The two
  // states' LOOK is Joe's (PROVISIONAL → M-RP-SKIN, D4); this reads one value and reflects it so the skin can
  // paint two states. `backgroundLive` STAYS accepted (its live/frozen contract is the FUTURE reactive plate,
  // solid-black → fractal clouds — M-RP-BACKDROP) and reported so the switch stays provably threaded; the
  // shell mirrors the backdrop value into it this leg, but the plate paints from the store, not from it.
  //
  // W-3 holds: imports only `$common` (`envelope`, the backdrop store) — no shell / Tauri / protocol dep.
  // Zero component-local CSS (N-090): the raster look — and the two-state flip — is `.grid-plate` in
  // skin.css (PROVISIONAL → M-RP-SKIN, D4).
  import { envelope } from '$common/components/base/envelope';
  import { backdrop } from '$common/stores/backdrop.svelte';

  // `id` self-defaults to 'grid-plate' (the self-panel `id = region-${regionId}` self-derivation shape),
  // so the plate registers as a STABLE, enumerable `grid-plate#grid-plate` even when mounted from a
  // props-less `WidgetMount` — not a mount-order ordinal (envelope's `id ?? ++ordinal` fallback).
  let { backgroundLive, id = 'grid-plate' }: { backgroundLive?: boolean; id?: string } = $props();

  // The one painted value (B2). Reactive — the settings component writes the store, this repaints.
  const pattern = $derived(backdrop.pattern);

  // G — reports BOTH: the mirrored `backgroundLive` (threaded proof) AND the rendered `pattern` (the flip),
  // so the setting is CDP-observable on the getter as well as the painted DOM.
  const debug = () => ({ backgroundLive, pattern });
</script>

<div class="grid-plate" data-pattern={pattern || undefined} use:envelope={{ name: 'grid-plate', id, debug }}></div>
