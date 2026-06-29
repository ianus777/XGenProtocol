<script lang="ts">
  // led — data-independent, DISPLAY-kind (N-032): an atomic inline status light. The FOURTH
  // simple display-di (after label/paragraph/image) — value-carrying but READ-ONLY, the display
  // half of the di model. Atomic (N-020): the root IS a <span> (the type-class `.led` via
  // `envelope`, N-023) — chosen as the neutral inline root for composite use beside a label;
  // <output> is avoided (reserved for the deferred dd progress/meter/output primitives).
  //
  // THE HEADLINE is the caller-supplied colour map. Like `select`'s `options` (N-034), the atomic
  // carries a `states: Record<string,string>` map it does NOT interpret — fully data-independent.
  // `led` is also the FIRST component whose colour is caller-supplied, not accent-derived: every
  // prior component's colour came from the skin (`--accent*`/`--t*`/`--err`); `led`'s colour comes
  // from the PROP, injected as an inline CSS custom property (`--led-colour`) the `.led` skin reads.
  // The skin owns SHAPE only. The shells' bespoke `.state-dot` + `dotColor(state)` becomes this,
  // generalised; the `status-indicator` composite builds on it.
  //
  // CONTRACT (lands on the caller): `colour = states[state] ?? "#000000"` — full black `#000000` is
  // the RESERVED unknown/undefined sentinel (an always-visible solid; a transparent dot would
  // disappear). CONSUMERS MUST NEVER MAP A REAL STATE TO `#000000`. `title = state ?? "?"` shows the
  // live key as a native tooltip (a set-but-unmapped key still shows — diagnostic; only truly-
  // undefined shows "?"). `role="img"` + `aria-label={title}` keeps a standalone led accessible
  // (colour is not the only signal). Values accept hex OR `var(--token)` (consumer hardcodes or rides
  // the skin tokens). `pulse?` is an optional animation, orthogonal to colour, reflected as a
  // `data-pulse` attribute the skin keys on (the toggle `role="switch"` attribute-hook precedent,
  // since `envelope` owns `class`). Read-only display-di: plain props, no `$bindable`, no processor.
  import { envelope } from '$common/components/base/envelope';

  let {
    states = {},
    state,
    pulse = false,
    id,
  }: {
    states?: Record<string, string>;
    state?: string;
    pulse?: boolean;
    id?: string;
  } = $props();

  // Resolve from the caller map; `#000000` is the reserved unknown/undefined sentinel. A missing
  // `state` (undefined key) and a set-but-unmapped key both fall through to the sentinel.
  const colour = $derived(states[state as string] ?? '#000000');
  // Native tooltip: the live key, or "?" when truly undefined.
  const title = $derived(state ?? '?');

  // N-024 opt-in. Read-only (no user delta); registry stays uniform (N-030 §4). The resolved
  // `colour` may be a `var(--token)` string (not a computed rgb) — that is correct; the computed
  // colour is a skin/computed-style concern verified separately.
  const debug = () => ({ state: state ?? null, colour });
</script>

<span
  use:envelope={{ name: 'led', id, debug }}
  role="img"
  aria-label={title}
  {title}
  data-pulse={pulse || undefined}
  style="--led-colour: {colour}"
></span>
