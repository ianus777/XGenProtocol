<script lang="ts">
  // label — data-independent, DISPLAY-kind (N-032): a short caption naming another
  // control. First of the display-di trio (label/paragraph/image) — value-carrying
  // but READ-ONLY (no bind-in of user action, no event-out), the display half of the
  // di model vs the four interactive di (toggle/button/textfield/select). Atomic
  // (N-020): the root IS the native <label> — chosen over <span> (inline, semantically
  // empty) and over <p> (that is `paragraph`, a different component).
  //
  // Value prop is `text`, NOT `value`: `value` is the editable/$bindable marker across
  // the codebase; the display-di take a semantic value-name (label & paragraph = text,
  // image = src). Read-only -> a plain prop, no $bindable.
  //
  // ASSOCIATION (for=/nesting) is a COMPOSITE concern (N-032), not the atom: there is
  // no `for` here — the pairing is wired by the group (textfield-group, leaning implicit
  // nesting). A standalone <label> with no control is valid-but-inert, a tolerated edge.
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // (N-023). No local CSS: a bare <label> is function-complete; all appearance — color,
  // size, and block-vs-inline — is skin, keyed by `.label` in the one skin file
  // (N-025 / N-021 layer 2). Pre-skin it renders as the bare normalize/native control.
  //
  // Founds the read-only display-di pattern that paragraph/image inherit: the SAME
  // `use:envelope` substrate, a plain (non-$bindable) value prop, the getter exposes the
  // value, and verification is render + computed-style (no event to dispatch).
  import { envelope } from '$common/components/base/envelope';

  let { text = '', id }: { text?: string; id?: string } = $props();

  // N-024 opt-in is one greppable line. Read-only: there is no user-driven delta, but the
  // registry stays uniform (N-030 §4 — the registry is one projection of the value).
  // $state.snapshot de-proxies so CDP's returnByValue receives plain JSON.
  const debug = () => $state.snapshot({ text });
</script>

<label use:envelope={{ name: 'label', id, debug }}>{text}</label>
