<script lang="ts">
  // textfield — data-independent, interaction semantic: free-text single-line (N-022).
  // Atomic (N-020): the root IS the native <input>; `type` is now a CONSTRAINED PROP
  // (M-RP2.12, → D-096), reversing N-029's "type is fixed". The whitelist folds the
  // structurally-identical string-input family into this one component —
  //   text (default) | search | email | url | tel | password
  // — all share the <input> root, the string bind:value, and the `.textfield` skin;
  // they differ ONLY in UA-supplied validation / soft-keyboard / masking. Value-type-
  // changing or chrome-adding types (number/range/date/color/file) remain their OWN
  // atomics; custom-chromed variants (the password-field reveal toggle, a custom
  // stepper) are COMPOSITES — neither folds in here. The per-type inset icon is a skin
  // look-distinguisher (N-039), not behaviour.
  //
  // Third real `core` component (M-RP2.5): where the toggle is event-IN (bind:checked)
  // and the button event-OUT (onclick), the textfield is the string bind-IN path
  // (bind:value) — the third `use:envelope` binding shape. It can self-redump a live
  // delta: type -> re-dump -> {value} changes (the N-024 live-reactive read on the
  // bind-in path, which the terminal-action button could not, N-028).
  //
  // Native state is the whole surface: `disabled` (inert + skin-greyed), `readonly`
  // (shown/selectable, not editable — distinct from disabled, NOT greyed), and template
  // matching via native `pattern` driving `:invalid` (the consumer owns the rule, the
  // skin owns the red look — no bespoke validation engine here). The `email`/`url` types
  // add native type-validation that drives the SAME `:invalid` look.
  //
  // Processor-READY, not processor-bearing: a text processor (emoji-combo, pattern
  // formatting) lives once in `common` as a `use:` action shared with <textarea> and is
  // simply layered on by a consumer (`use:processor={pairs}`) — the field neither
  // contains nor blocks it. Built separately, later.
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // here (N-023). No local CSS: a bare <input> is function-complete; all appearance —
  // size, text alignment, greyed-disabled, invalid border, focus ring, per-type icon —
  // is skin, keyed by `.textfield` (+ `.textfield[type=…]`) in the one skin file
  // (N-025 / N-021 layer 2). Pre-skin it renders as the bare normalize.css/native control.
  import { envelope } from '$common/components/base/envelope';

  let {
    type = 'text',
    value = $bindable(''),
    placeholder = '',
    disabled = false,
    readonly = false,
    id,
    pattern,
    name,
    autocomplete,
    redactValue = false,
  }: {
    type?: 'text' | 'search' | 'email' | 'url' | 'tel' | 'password';
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    id?: string;
    pattern?: string;
    name?: string;
    autocomplete?: string;
    redactValue?: boolean;
  } = $props();

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies the value so CDP's
  // returnByValue receives plain JSON rather than a reactive proxy. `redactValue`
  // (M-RP2.23, default off) nulls the value in the getter so a `password-field` child
  // never publishes the live secret into the dev registry; existing cells are unchanged.
  const debug = () => $state.snapshot({ type, value: redactValue ? null : value });
</script>

<input
  {type}
  {placeholder}
  {disabled}
  {readonly}
  {pattern}
  {name}
  autocomplete={autocomplete || undefined}
  bind:value
  use:envelope={{ name: 'textfield', id, debug }}
/>
