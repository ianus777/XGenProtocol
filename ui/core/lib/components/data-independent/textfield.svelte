<script lang="ts">
  // textfield — data-independent, interaction semantic: free-text single-line (N-022).
  // Atomic (N-020): the root IS the native <input type="text">; `type` is fixed, NOT a
  // prop — neighbouring semantics (email/url/tel = constrained-text, password = secret,
  // number = numeric) are their own components, and the search field is a shape variant,
  // not a new component. Third real `core` component (M-RP2.5): where the toggle is
  // event-IN (bind:checked) and the button event-OUT (onclick), the textfield is the
  // string bind-IN path (bind:value) — the third `use:envelope` binding shape. It is
  // also the component that CAN self-redump a live delta: type -> re-dump -> {value}
  // changes (re-proving the N-024 live-reactive read on the bind-in path, which the
  // terminal-action button could not, N-028).
  //
  // Native state is the whole surface: `disabled` (inert + skin-greyed), `readonly`
  // (shown/selectable, not editable — distinct from disabled, NOT greyed), and template
  // matching via native `pattern` driving `:invalid` (the consumer owns the rule, the
  // skin owns the red look — no bespoke validation engine here).
  //
  // Processor-READY, not processor-bearing: a text processor (emoji-combo, pattern
  // formatting) lives once in `common` as a `use:` action shared with <textarea> and is
  // simply layered on by a consumer (`use:processor={pairs}`) — the field neither
  // contains nor blocks it. Built separately, later.
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // here (N-023). No local CSS: a bare <input> is function-complete; all appearance —
  // size, text alignment, greyed-disabled, invalid border, focus ring — is skin, keyed
  // by `.textfield` in the one skin file (N-025 / N-021 layer 2). Pre-skin it renders as
  // the bare normalize.css/native control.
  import { envelope } from '$common/components/base/envelope';

  let {
    value = $bindable(''),
    placeholder = '',
    disabled = false,
    readonly = false,
    id,
    pattern,
    name,
  }: {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    id?: string;
    pattern?: string;
    name?: string;
  } = $props();

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies the value so CDP's
  // returnByValue receives plain JSON rather than a reactive proxy.
  const debug = () => $state.snapshot({ value });
</script>

<input
  type="text"
  {placeholder}
  {disabled}
  {readonly}
  {pattern}
  {name}
  bind:value
  use:envelope={{ name: 'textfield', id, debug }}
/>
