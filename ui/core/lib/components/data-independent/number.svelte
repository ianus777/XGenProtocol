<script lang="ts">
  // number — data-independent, interaction semantic: numeric free-entry (N-022).
  // Atomic (N-020): the root IS the native <input type="number">. The SAME root tag as
  // `textfield`, but a DISTINCT component, NOT a member of the textfield `type` fold
  // (D-096). The boundary D-096 drew is *same root + same VALUE-TYPE*: the fold works
  // for text|search|email|url|tel|password because one `value: string` is correct for
  // all of them. `number` breaks the second half — Svelte's `bind:value` on a
  // type="number" input coerces to a NUMBER (and to `null` when the field is empty),
  // not a string. Folding it in would force textfield's `value` prop polymorphic
  // (string | number | null) and defeat the single-typed contract the fold exists to
  // provide. So `number` stays its own atomic with one honest numeric value type.
  //
  // Ninth `core` component (M-RP2.14): the first registry value that is neither boolean
  // (toggle) nor string (everything since toggle) — a JSON number | null.
  //
  // Prop surface = the control vocabulary with the numeric bits swapped in:
  //   keep  — value (numeric bind:value), placeholder (shows when empty), disabled,
  //           readonly, id, name
  //   drop  — `type` (fixed "number"); `pattern` (ignored on type="number")
  //   add   — `min` / `max` / `step` (native attributes that shape the control; `step`
  //           drives the native-spinner increment) — config, not state.
  // `maxlength` is deliberately NOT added (orthogonal — mirrors textfield/textarea).
  //
  // The NATIVE SPINNER is kept — the UA up/down arrows ARE the atomic's affordance. The
  // custom-button stepper is a separate composite (later track), not this; so no
  // ::-webkit-*-spin-button suppression in the skin.
  //
  // PROCESSOR-READY, NOT processor-bearing. The future EDIT-side text processor's
  // NUMERIC-FORMATTING consumer (thousands separators, locale, clamping re-run as the
  // user types) layers on as `use:processor={config}` from `common` — shared with
  // textfield/textarea. NOT built here: the atomic is function-complete without it, and
  // the N-038 track order builds the engine in its own arc AFTER all atomic di, with
  // every consumer in hand (D-065). This is the reserved insertion point. (Second
  // consumer to reserve-and-defer, after textarea — D-069 promotion-watch, not yet at
  // the four-recurrence bar.)
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // (N-023). No local CSS: a bare numeric <input> is function-complete; all appearance —
  // size, greyed-disabled, focus ring, the :invalid (native numeric constraint
  // validation) look — is skin, keyed by `.number` in the one skin file (N-025 / N-021
  // layer 2). Pre-skin it renders as the bare normalize.css/native control.
  import { envelope } from '$common/components/base/envelope';

  let {
    value = $bindable(null),
    placeholder = '',
    disabled = false,
    readonly = false,
    min,
    max,
    step,
    id,
    name,
    ...rest
  }: {
    value?: number | null;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    min?: number;
    max?: number;
    step?: number;
    id?: string;
    name?: string;
    // Clamp-host (M-RP4.1): a forwarded attachment (+ any extra native attrs) lands here and is
    // spread onto <input> — the reserved insertion point, now wired. The index signature covers
    // the symbol-keyed attachment. The atomic carries NO clamp logic (ready, not containing, D-065).
    [key: string]: unknown;
  } = $props();

  // N-024 opt-in is one greppable line. $state.snapshot de-proxies for CDP's
  // returnByValue. Value-only — min/max/step are static config, not user-mutable state.
  const debug = () => $state.snapshot({ value });
</script>

<input
  {...rest}
  type="number"
  {placeholder}
  {disabled}
  {readonly}
  {min}
  {max}
  {step}
  {name}
  bind:value
  use:envelope={{ name: 'number', id, debug }}
/>
