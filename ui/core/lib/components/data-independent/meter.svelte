<script lang="ts">
  // meter — data-independent, DISPLAY semantic: a bounded, read-only value bar (N-032). The
  // 5th simple display-di (after label/paragraph/image/led) and the 25th `core`. Atomic (N-020):
  // the root IS the native <meter>. The read-only SIBLING of `range` (range = editable numeric-in
  // via bind:value; meter = read-only value-against-range-out) — same value shape {value,min,max},
  // opposite direction; distinct atomic on the display-vs-edit axis (the paragraph/textarea
  // precedent). di, NOT dd: it binds a plain number against a range, interprets no domain
  // structure (a dd would materialize IdentityRecord/SpaceState/etc.).
  //
  // Value is a PLAIN prop (read-only display-di rule — label/paragraph/led take a semantic value,
  // not $bindable). `low`/`high`/`optimum` drive the NATIVE semantic fill: the UA picks
  // ::-webkit-meter-optimum-value / -suboptimum-value / -even-less-good-value by where `value`
  // sits relative to optimum + the low/high thresholds. With no `optimum`, the bar is a single
  // neutral fill (skin default). The atomic supplies no caption/readout — a bare <meter> is just
  // the bar; the consuming composite/widget adds label + value text (the range/number rule).
  //
  // Width (Joe-lock): FULL-WIDTH by default (`.meter` sets display:block + width:100%, fills the
  // container — unlike range's pinned 160px, since a status bar reads better stretched). Optional
  // `width?` pins a fixed width (inline override, e.g. "200px"/"12rem"); a skin min-width floor
  // stops a narrow parent crushing it. No collapse/measure logic (nothing to overflow).
  //
  // The type-class is supplied by `envelope` (N-023), so no `class` is hardcoded. No local CSS:
  // all appearance is skin, keyed by `.meter` (pseudo-heavy, PROVISIONAL) in the one skin file
  // (N-025 / N-021 layer 2) — the track + the three semantic value-pseudos + disabled.
  import { envelope } from '$common/components/base/envelope';

  let {
    value,
    min = 0,
    max = 1,
    optimum,
    low,
    high,
    width,
    disabled = false,
    id,
    name,
  }: {
    /** The current value (read-only; plain, not $bindable). */
    value: number;
    /** Lower bound (native default 0). */
    min?: number;
    /** Upper bound (native default 1). */
    max?: number;
    /** The ideal value — drives which semantic fill the UA paints. Omitted = single neutral fill. */
    optimum?: number;
    /** Lower "sub-optimum begins" threshold (native `low`). */
    low?: number;
    /** Upper "sub-optimum begins" threshold (native `high`). */
    high?: number;
    /** Fixed width (e.g. "200px" / "12rem"). Omitted = full-width (skin 100%). */
    width?: string;
    disabled?: boolean;
    id?: string;
    name?: string;
  } = $props();

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies for CDP returnByValue. Config
  // (min/max/optimum) travels with the value since it defines what the value MEANS on the bar.
  const debug = () => $state.snapshot({ value, min, max, optimum });
</script>

<meter
  {value}
  {min}
  {max}
  {optimum}
  {low}
  {high}
  {name}
  aria-disabled={disabled ? 'true' : undefined}
  style={width ? `width: ${width}` : undefined}
  use:envelope={{ name: 'meter', id, debug }}
></meter>
