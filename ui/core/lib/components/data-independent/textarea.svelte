<script lang="ts">
  // textarea — data-independent, interaction semantic: free-text MULTI-line (N-022).
  // Atomic (N-020): the root IS the native <textarea>. A DISTINCT component from
  // `textfield`, not a fold of it: the root tag differs (<textarea> vs <input>), and
  // the root tag is the atomic discriminator. It is the EDIT-side multi-line
  // counterpart to `paragraph`'s render-side single prose string (N-032 EDIT-vs-RENDER
  // axis): `paragraph` wraps one READ-ONLY string visually (a text node); `textarea`
  // holds literal `\n`-bearing EDITABLE free text.
  //
  // Eighth `core` component (M-RP2.13): the string bind-IN path (bind:value) again,
  // after toggle (boolean-in), button (event-out), textfield (string-in), select
  // (content-carrying string-in) — the substrate generalizes across the textfield→
  // textarea tag change unchanged.
  //
  // Prop surface = the string-input vocabulary shared with `textfield`, MINUS what
  // <textarea> can't carry, PLUS `rows`:
  //   keep  — value (bind:value), placeholder, disabled, readonly, id, name
  //   drop  — `type` (no such attribute on <textarea>); `pattern` (<input>-only, so no
  //           :invalid-via-pattern path here)
  //   add   — `rows` (initial visible height; the one textarea-specific prop)
  // `maxlength` is deliberately NOT added (orthogonal native-state addition, out of
  // scope — mirrors textfield).
  //
  // PROCESSOR-READY, NOT processor-bearing. The future EDIT-side text processor (text
  // morphs / emoji-combo / pattern formatting, re-run as the user types) lives ONCE in
  // `common` as a `use:processor={config}` action shared with `textfield`/`number` —
  // the edit-side counterpart to paragraph's render-side `use:render`. A consumer simply
  // layers it on (`use:processor={config}`); this component neither contains nor blocks
  // it. NOT built here: the atomic is function-complete without it, and the N-038 track
  // order builds the engine in its own arc AFTER all atomic di, with every consumer in
  // hand (D-065 — no empty machinery). This is the reserved insertion point.
  //
  // AUTO-GROW is a future SKIN shape, not built. The single-engine WebView2/Chromium
  // target affords a pure-CSS path (`field-sizing: content`) — reserved as a skin shape
  // (like select's `appearance:base-select`), not authored now (D-065). The atomic ships
  // native fixed-`rows` height + vertical resize.
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // (N-023). No local CSS: a bare <textarea> is function-complete; all appearance — size,
  // greyed-disabled, focus ring, the resize affordance — is skin, keyed by `.textarea`
  // in the one skin file (N-025 / N-021 layer 2). Pre-skin it renders as the bare
  // normalize.css/native control.
  import { envelope } from '$common/components/base/envelope';

  let {
    value = $bindable(''),
    placeholder = '',
    disabled = false,
    readonly = false,
    rows = 3,
    id,
    name,
  }: {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    rows?: number;
    id?: string;
    name?: string;
  } = $props();

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies the value so CDP's
  // returnByValue receives plain JSON rather than a reactive proxy. `rows` is static
  // config, not user-mutable state — value-only, like select/textfield.
  const debug = () => $state.snapshot({ value });
</script>

<textarea
  {placeholder}
  {disabled}
  {readonly}
  {rows}
  {name}
  bind:value
  use:envelope={{ name: 'textarea', id, debug }}
></textarea>
