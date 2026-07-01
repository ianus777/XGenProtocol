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
  // PROCESSOR-HOST (M-RP4.0 / N-056). The EDIT-side text processor (kind 1 transformer:
  // text morphs / emoji-combo, re-run as the user types) ships in `common` as a forwarded
  // Svelte 5 ATTACHMENT, NOT a `use:` action — a consumer can't forward a `use:` onto an
  // atomic's internal element, but it CAN spread an attachment. So this atomic forwards
  // `{...rest}` onto <textarea>; a consumer lands it via `<TextArea {...processor(rules)} … />`.
  // The atomic carries NO processing logic — only the generic spread (ready, not containing —
  // D-065). number is the second host (M-RP4.1); see M-RP4.0 / N-056.
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
    ...rest
  }: {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    rows?: number;
    id?: string;
    name?: string;
    // Processor-host (M-RP4.0): a forwarded attachment (+ any extra native attrs) lands here
    // and is spread onto <textarea>. The index signature covers the symbol-keyed attachment.
    [key: string]: unknown;
  } = $props();

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies the value so CDP's
  // returnByValue receives plain JSON rather than a reactive proxy. `rows` is static
  // config, not user-mutable state — value-only, like select/textfield.
  const debug = () => $state.snapshot({ value });
</script>

<textarea
  {...rest}
  {placeholder}
  {disabled}
  {readonly}
  {rows}
  {name}
  bind:value
  use:envelope={{ name: 'textarea', id, debug }}
></textarea>
