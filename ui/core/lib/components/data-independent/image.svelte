<script lang="ts">
  // image — data-independent, DISPLAY-kind (N-032): a single bitmap/vector image.
  // Third and final of the display-di trio (label/paragraph/image) — value-carrying
  // but READ-ONLY. Atomic (N-020): the root IS the native <img>.
  //
  // STRUCTURAL NOVELTY: <img> is a VOID element — the first display-di whose value
  // lives in an ATTRIBUTE (`src`), not in a text-node body (label/paragraph put the
  // value in their text content). The read-only display-di pattern (N-035) otherwise
  // carries over unchanged: same `use:envelope`, plain (non-$bindable) props, getter
  // exposes the value, verify = render + computed-style.
  //
  // Value prop is `src` (the display-di semantic value-name — label/paragraph take
  // `text`, image takes `src`). `alt` is REQUIRED (N-032): both `src` and `alt` are
  // typed non-optional with no default, so the consumer must consciously pass them —
  // including `alt=""` for a deliberately decorative image (valid + conscious). The
  // requirement forces the a11y DECISION, it does not forbid empty. A DEV-only guard
  // warns if `alt` was omitted entirely (undefined); no prod throw — an image with a
  // missing alt should still render.
  //
  // Load/error states (broken-src placeholder) and width/height (CLS) are deferred —
  // consumer/composite concerns, kept off the pure atomic (same spirit as `for`/
  // association being off `label`, N-032).
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // (N-023). No local CSS: a bare <img> is function-complete (xgen-normalize floors it
  // block + max-width:100% + transparent ground); all appearance — here just the
  // corner radius — is skin, keyed by `.image` in the one skin file (N-025).
  import { envelope } from '$common/components/base/envelope';

  let { src, alt, id }: { src: string; alt: string; id?: string } = $props();

  if (import.meta.env.DEV && alt === undefined) {
    console.warn('[xgen image] `alt` is required (pass alt="" for a decorative image).');
  }

  // N-024 opt-in. Read-only (no user delta); registry stays uniform (N-030 §4). Unlike
  // label/paragraph's single {text}, image carries {src, alt} — alt being required makes
  // it part of the component's meaningful state, and lets verify confirm it landed.
  const debug = () => $state.snapshot({ src, alt });
</script>

<img use:envelope={{ name: 'image', id, debug }} {src} {alt} />
