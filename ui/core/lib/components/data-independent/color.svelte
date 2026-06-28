<script lang="ts">
  // color — data-independent, interaction semantic: colour pick, native swatch + picker (N-022).
  // Atomic (N-020): root IS <input type="color">. A SINGLETON (no type-family) — it stands
  // ALONE, NOT folded. Though it shares the <input> root AND value-type (string) with `date`,
  // it diverges on skin/surface (a SWATCH — `::-webkit-color-swatch*` pseudos, disjoint from
  // date's text-box + calendar indicator) and on prop surface — so by the SHARPENED D-096
  // criterion (root + value-type + shared skin/surface, N-042) it is an own atomic: the `range`
  // case, NOT the `textfield` case. Applies D-096, no amendment. (root + value-type alone would
  // pull toward a date fold — exactly the trap the sharpened criterion exists for.)
  //
  // Twelfth `core` component (M-RP2.17). The value is ALWAYS a 7-char lowercase hex string
  // `#rrggbb`; the native control has no empty state — default `#000000`, never '' (the date
  // divergence) or null (the number divergence). The always-valued shape, like `range`. Getter
  // `{value}` — NO `type` (singleton, type fixed = "color"), unlike date's `{type,value}`.
  //
  // Prop surface = the LEANEST atomic yet:
  //   keep — value (string bind:value, default '#000000'), disabled, id, name
  //   drop — placeholder / pattern (n/a), readonly (native no-op on color — the range
  //          precedent), min / max / step (n/a), :invalid (always a valid hex, never invalid),
  //          type (fixed)
  // No processor seam (a swatch pick, not typed entry — the range reasoning; the typed-entry
  // consumer is `number`/`textfield`). `alpha`/`colorspace` (`#rrggbbaa`) is a reserved future
  // shape, NOT built.
  //
  // The OPEN picker dialog (saturation square / hue slider / eyedropper / hex field / preset
  // swatches) is OS/Chromium-painted, NOT skinnable — a themed custom palette is the deferred
  // `color-picker` COMPOSITE (#2, the password-field-off-textfield shape), not this atomic.
  //
  // The type-class is supplied by `envelope` (N-023), so no `class` is hardcoded. No local CSS:
  // a bare colour <input> is function-complete; the `.color` skin styles ONLY the CLOSED-STATE
  // swatch (the box + the `::-webkit-color-swatch*` pseudos), keyed in the one skin file
  // (N-025 / N-021 layer 2). color-scheme:dark (N-043) is largely moot here — the picker dialog
  // is OS-painted, not webview-painted. Pre-skin it renders as the bare native control.
  import { envelope } from '$common/components/base/envelope';

  let {
    value = $bindable('#000000'),
    disabled = false,
    id,
    name,
  }: {
    value?: string;
    disabled?: boolean;
    id?: string;
    name?: string;
  } = $props();

  // N-024 opt-in. $state.snapshot de-proxies for CDP's returnByValue. No `type` — singleton.
  const debug = () => $state.snapshot({ value });
</script>

<input
  type="color"
  {disabled}
  {name}
  bind:value
  use:envelope={{ name: 'color', id, debug }}
/>
