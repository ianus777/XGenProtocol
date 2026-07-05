<script lang="ts">
  // entity-avatar — the FIRST data-dependent component (M-RP5.0, dd-ATOMIC). It materializes
  // ONE address-book entry (an identity or a space, per the D-071 Phase-0 audit of
  // IdentityRecord/SpaceState) into a visual. What makes it dd (not di): the rendered SHAPE
  // itself BRANCHES on the data — identity → circle + AI-badge; non-DM space → rounded-square;
  // revoked → greyed + slashed. That domain → presentation mapping IS the reason it exists.
  //
  // It consumes a source-agnostic `EntityDescriptor` view-model (the W-11 dd-socket payload),
  // NOT the raw protocol type — `core` imports NO `IdentityRecord`/`SpaceState` (GPL ref lib
  // stays protocol-free; the SHELL owns the protocol → descriptor map).
  //
  // The avatar is a PARTIAL read of the record (the context menu, M-RP5.3, is the 100% read).
  // The primary axis is `variant` (PURPOSE) — size + content are DERIVED presets, not free
  // props (the led.state / textfield.type discipline): `presence` = shape + colour seed only;
  // `list` = + initials. `labeled`/`card` land with `container-list-item` (M-RP5.1, dd-composite).
  import { envelope } from '$common/components/base/envelope';
  import { seedColour } from '$common/components/base/seed-colour';
  import type { EntityDescriptor } from './types';

  let {
    descriptor,
    variant = 'presence',
    onActivate,
    id,
  }: {
    descriptor: EntityDescriptor;
    variant?: 'presence' | 'list';
    onActivate?: () => void;
    id?: string;
  } = $props();

  const kind = $derived(descriptor.kind);
  const name = $derived(descriptor.name);
  const flags = $derived(descriptor.flags ?? {});

  // C — kind → shape. identity = circle; DM space = circle (people-shaped); non-DM space =
  // rounded-square. The dd's shape branches on the data — this is the dd ≠ di line.
  const shape = $derived(kind === 'space' && !flags.isDm ? 'square' : 'circle');

  // E — content-derived colour, seeded on `name ?? id` (an absent-name avatar still gets a
  // stable colour). The shared `seedColour` helper (chip's muted band); NO `--accent`
  // dependency, so a given entity reads the same colour under gold or blue.
  const seed = $derived(seedColour(name ?? descriptor.id));

  // initials — 1–2 graphemes from `name` (grapheme-safe: a ZWJ family / skin-tone / combining
  // mark stays one unit — `chars`/`slice` would split it); absent name → xgid-tail fallback
  // (last 2 alphanumerics of the id). Uppercased for display.
  const initials = $derived(deriveInitials(name, descriptor.id));

  function graphemes(s: string): string[] {
    if (typeof Intl !== 'undefined' && 'Segmenter' in Intl) {
      return [...new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(s)].map(
        (x) => x.segment,
      );
    }
    return [...s]; // codepoint fallback (still better than UTF-16 units)
  }

  function deriveInitials(nm: string | undefined, xgid: string): string {
    const n = (nm ?? '').trim();
    if (n) {
      const words = n.split(/\s+/);
      const pick =
        words.length >= 2
          ? (graphemes(words[0])[0] ?? '') + (graphemes(words[1])[0] ?? '')
          : graphemes(n).slice(0, 2).join('');
      return pick.toUpperCase();
    }
    // xgid-tail fallback — deterministic, stable per id.
    return xgid.replace(/[^a-z0-9]/gi, '').slice(-2).toUpperCase();
  }

  // G — the dd-atomic self-registers ONE aggregate getter. `seed` = the fill hsl string
  // (directly comparable across shells → the seed shell-independence proof).
  const debug = () => ({ kind, variant, name: name ?? null, initials, seed: seed.bg, flags });

  // H — `onActivate` is the RESERVED menu-trigger seam (the `entity-context-menu` widget,
  // M-RP5.3, consumes it). Wired to onclick; NO menu is built here (reserve, don't build).
</script>

<!-- B — dd root = HONEST HTML for the materialized thing: <figure role="img"> (a self-
  contained figure; dd does NOT inherit the di <div>=composite litmus — N-075). aria-label =
  name ?? kind. <figcaption> is RESERVED — the seam for the `labeled`/`card` variants
  (M-RP5.1) — deliberately unused in v1 (an avatar is a glyph, not a captioned figure yet).
  `presence` renders shape + colour only; `list` adds the initials text. -->
<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
<figure
  use:envelope={{ name: 'entity-avatar', id, debug }}
  role="img"
  aria-label={name ?? kind}
  data-variant={variant}
  data-kind={kind}
  data-shape={shape}
  data-ai={flags.isAi || undefined}
  data-revoked={flags.revoked || undefined}
  style="--seed-bg: {seed.bg}; --seed-fg: {seed.fg}; --seed-bd: {seed.bd}"
  onclick={() => onActivate?.()}
>
  {#if variant === 'list'}
    <span class="ea-initials">{initials}</span>
  {/if}
</figure>
