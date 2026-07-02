<script lang="ts">
  // password-field — data-independent, COMPOSITE (M-RP2.23): the SECOND di composite and the
  // EIGHTEENTH `core` component. Root IS `<div class="password-field">` (the N-020/N-022 composite
  // marker via `envelope`). It is the home for the reveal toggle deferred at the `textfield` `type`
  // fold (M-RP2.12/D-096). Composes built atomics: `textfield` (child, `type` driven password<->text)
  // + `button` in toggle-mode (the reveal control) + an OPTIONAL `label` caps-lock warning. Binding =
  // the string bind:value forwarded from the child; **di** — no domain interpretation.
  //
  // COMPOSITE-REGISTRATION MODEL (N-054): the composite root registers ONE aggregate getter; the
  // children self-register (each passes its own `debug`), handed composite-derived stable ids
  // `<id>__field` / `<id>__reveal` / `<id>__capswarn`. A cell yields multiple registry entries.
  //
  // Three composite-specific mechanics:
  //  1. REVEAL = `button mode="toggle"`, `bind:pressed={revealed}`; the child textfield's `type`
  //     is `revealed ? 'text' : 'password'`. The reveal button carries no label (glyph is skin
  //     `::before`, keyed off the reflected `aria-pressed`) + an `ariaLabel` that flips Show/Hide.
  //  2. SECRET SAFETY = the child textfield is passed `redactValue` (M-RP2.23 Step A), so its own
  //     getter reports `value:null` — the live secret never reaches the dev registry. The composite
  //     getter reports only a boolean `hasValue`, never the value.
  //  3. CAPS-LOCK = keyboard events bubble from the inner `<input>` to this wrapper `<div>`, so a
  //     composite-level `onkeyup`/`onkeydown` reads `getModifierState('CapsLock')` — NO textfield
  //     touch needed. The warning is an OPTIONAL `label` child rendered `{#if capsLock}` (the
  //     status-indicator optional-link precedent, N-054 — NOT the N-053 mount rule).
  //
  // Confirm-password match is a DIFFERENT unit (a future `password-confirm` composite wrapping two
  // fields; equality-checking interprets values -> leans dd). Not built here.
  import { envelope } from '$common/components/base/envelope';
  import Textfield from './textfield.svelte';
  import Button from './button.svelte';
  import Label from './label.svelte';

  let {
    value = $bindable(''),
    placeholder = '',
    disabled = false,
    readonly = false,
    id,
    name,
    autocomplete,
    revealedByDefault = false,
  }: {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    id?: string;
    name?: string;
    autocomplete?: string;
    revealedByDefault?: boolean;
  } = $props();

  let revealed = $state(revealedByDefault);
  let capsLock = $state(false);

  // Composite-derived stable child ids (so the self-registering children read cleanly, not ordinal).
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // Caps-lock is read off the bubbled keyboard event at the wrapper (no textfield touch).
  function onKey(e: KeyboardEvent) {
    capsLock = e.getModifierState?.('CapsLock') ?? false;
  }

  // N-024 opt-in. Aggregate of what the COMPOSITE owns; `hasValue` is a boolean — never the value.
  const debug = () => ({ revealed, hasValue: value.length > 0, capsLock });
</script>

<div use:envelope={{ name: 'password-field', id, debug }} onkeyup={onKey} onkeydown={onKey}>
  <Textfield
    type={revealed ? 'text' : 'password'}
    bind:value
    {placeholder}
    {disabled}
    {readonly}
    {name}
    {autocomplete}
    redactValue
    id={cid('field')}
  />
  <Button
    mode="toggle"
    bind:pressed={revealed}
    {disabled}
    ariaLabel={revealed ? 'Hide password' : 'Show password'}
    id={cid('reveal')}
  />
  {#if capsLock}
    <Label text="Caps Lock is on" id={cid('capswarn')} />
  {/if}
</div>
