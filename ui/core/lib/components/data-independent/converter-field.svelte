<script lang="ts" generics="T">
  // converter-field — data-independent, interaction semantic: a TWO-REPRESENTATION text field
  // (N-022). Kind 2 of the processor taxonomy (D-099 / N-056) — the BRIDGE. Atomic (N-020): the
  // root IS the native <input type="text">, the SAME root as `textfield`/`number`, but a DISTINCT
  // component on the discriminator that makes kind 2 special: it holds TWO reps of DIFFERENT type
  // at once — a DISPLAY string (what the <input> shows) and a bound TYPED value `T` (what the
  // consumer reads). Kinds 1/3 forward an attachment and sync through a single same-type
  // bind:value; kind 2 can't (two types), so it is a real component, not an attachment.
  //
  // The bridge is a caller-supplied `Converter<T>` (Tier-1 code — LOGIC, never a user string, so
  // no provenance caps/lint; contrast kind 1). First concrete: `intlNumber()` from
  // $common/.../transform (Intl.NumberFormat display + a formatToParts-derived parser).
  //
  // Timing (Joe-lock): parse on `change`/`blur`; success reformats the display via toString;
  // `focus` shows the RAW toEditable form (so "1234.56" edits without fighting "1,234.56");
  // NOTHING on `input` — the field is decoupled, so there is NO caret-restore machinery (the
  // kind-1 concern doesn't arise). Parse failure = REJECT-AND-MARK: keep the user's text, set
  // [data-invalid], leave `value` untouched. Empty text on commit = no-op revert (never an
  // "invalid empty").
  //
  // Two-rep state: `value` is $bindable (the typed OUT); `text` is internal $state (the display/
  // edit string). `text` is NOT a $derived of value — deriving it would clobber live typing. An
  // external value change reformats the display, but ONLY while unfocused (never overwrites an
  // active edit).
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded (N-023).
  // No local CSS: all appearance is skin, keyed by `.converter-field` (+ [data-invalid]) in the
  // one skin file (N-025 / N-021 layer 2), assembled from the `.number` vocabulary.
  import { envelope } from '$common/components/base/envelope';
  import { PARSE_FAILED, intlNumber, type Converter } from '$common/components/processor/transform';

  // DEV-only pure-core hook for CDP verification (mirrors __XGEN_PROC__/__XGEN_CLAMP__, N-024).
  // The component is kind 2's framework touch (there is no attachment file), so the hook lives
  // here. transform.ts stays DOM/window-free. Dead-code-eliminated in a production build.
  if (import.meta.env.DEV && typeof window !== 'undefined') {
    (window as unknown as { __XGEN_CONVERT__?: unknown }).__XGEN_CONVERT__ = { intlNumber, PARSE_FAILED };
  }

  let {
    value = $bindable(),
    converter,
    placeholder = '',
    disabled = false,
    readonly = false,
    id,
    name,
    ...rest
  }: {
    /** The typed bound value (the OUT). Undefined = no value yet -> empty display. */
    value?: T;
    /** The bidirectional bridge (Tier-1 code). Required. */
    converter: Converter<T>;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    id?: string;
    name?: string;
    /** Extra native attrs land on <input> (the index signature covers them). */
    [key: string]: unknown;
  } = $props();

  const display = (v: T | undefined): string => (v === undefined ? '' : converter.toString(v));
  const editable = (v: T | undefined): string =>
    v === undefined ? '' : (converter.toEditable ?? converter.toString)(v);

  let focused = $state(false);
  let invalid = $state(false);
  let text = $state(display(value)); // the display/edit string the <input> shows

  // External value change (from the consumer, not our own commit) reformats the display — but
  // only while UNFOCUSED, so a live typing session is never overwritten.
  $effect(() => {
    const v = value;
    if (!focused) {
      text = display(v);
      invalid = false;
    }
  });

  function onFocus() {
    focused = true;
    text = editable(value); // raw form for editing (strip grouping)
    invalid = false;
  }

  function commit() {
    if (text.trim() === '') {
      // empty = no-op: revert display to the current value, clear invalid (never "invalid empty")
      text = display(value);
      invalid = false;
      return;
    }
    const parsed = converter.fromString(text);
    if (parsed === PARSE_FAILED) {
      invalid = true; // reject-and-mark: keep text, value UNCHANGED
      return;
    }
    value = parsed as T;
    invalid = false;
    text = display(value); // reformat (grouping etc.)
  }

  function onBlur() {
    focused = false;
    commit();
  }

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies for CDP returnByValue.
  const debug = () => $state.snapshot({ value, text, valid: !invalid });
</script>

<input
  {...rest}
  type="text"
  bind:value={text}
  {placeholder}
  {disabled}
  {readonly}
  {name}
  data-invalid={invalid ? 'true' : undefined}
  onfocus={onFocus}
  onblur={onBlur}
  onchange={commit}
  use:envelope={{ name: 'converter-field', id, debug }}
/>
