<script lang="ts">
  // chip — data-independent, a standalone removable token (M-RP2.27): 22nd `core`, atomic-ish
  // (root IS a <span class="chip"> via `envelope`, no self-registering child components — the
  // `×` is a raw <button>, not an atomic). Built standalone (its own registry row + sampler
  // cells) because it recurs downstream: dd facets, tier/`is_ai` badges, entity tokens.
  // `tag-select` (M-RP2.28) renders chip instances via {#each} WITHOUT per-instance
  // registration (dynamic instances → the N-064 pattern).
  //
  // THE HEADLINE is self-computed colour. `led` (N-034) was caller-supplied; every other di is
  // accent-derived. `chip` is the FIRST di whose colour is COMPUTED from its own content:
  // hash(label) → hue, at a fixed muted S/L band (never bright/white), injected as inline CSS
  // vars (`--chip-bg`/`--chip-fg`/`--chip-bd`) the `.chip` skin reads (the `--led-colour`
  // mechanism). Deterministic + shell-independent: a tag reads the same under gold or blue.
  //
  // `label` is the RAW stored value; uppercase is display-only (skin `text-transform`). Remove:
  // `×` on the RIGHT, `×`-only (the chip body stays inert-selectable for later); `onRemove?`
  // fires on click; the glyph is a masked stroke (`--chip-x`, N-052 lineage). `removable?`
  // default true — false drops the `×`, same fill. Plain props, no `$bindable` (display token).
  import { envelope } from '$common/components/base/envelope';

  let {
    label = '',
    removable = true,
    onRemove,
    id,
    register = true,
  }: {
    label?: string;
    removable?: boolean;
    onRemove?: () => void;
    id?: string;
    register?: boolean;
  } = $props();

  // Deterministic string hash → hue 0..359 (djb2-ish). Content-derived, stable per label.
  const hue = $derived.by(() => {
    let h = 5381;
    for (let i = 0; i < label.length; i++) h = ((h << 5) + h + label.charCodeAt(i)) | 0;
    return ((h % 360) + 360) % 360;
  });

  // Fixed muted S/L band — fill light, text dark, border mid — all same hue. Never white.
  const bg = $derived(`hsl(${hue} 45% 82%)`);
  const fg = $derived(`hsl(${hue} 55% 30%)`);
  const bd = $derived(`hsl(${hue} 40% 80%)`);

  // Registration opt-out (N-064): a consumer rendering chips dynamically (tag-select) passes
  // register={false} so the chip renders + stamps `.chip` but does NOT self-register (envelope
  // registers only when a debug getter is present). Standalone chips keep register=true.
  const debug = register ? () => ({ label, removable }) : undefined;
</script>

<span
  class="chip"
  use:envelope={{ name: 'chip', id, debug }}
  style="--chip-bg: {bg}; --chip-fg: {fg}; --chip-bd: {bd}"
>
  <span class="chip-label">{label}</span>
  {#if removable}
    <button
      class="chip-x"
      type="button"
      aria-label="Remove {label}"
      onclick={() => onRemove?.()}
    ></button>
  {/if}
</span>
