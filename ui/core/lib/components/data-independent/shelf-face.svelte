<script lang="ts">
  // shelf-face — data-independent chrome (M-RP6.1i): one face of a `shelf`. A face is a widget's
  // compact HANDLE on a shelf (N-100): an icon-only command button, never a shrunken widget (S-4).
  // Its own native <button> root composing the `icon` core child, self-registering with its own
  // getter — the `menu-item` leaf pattern, adapted for a toolbar.
  //
  // WHY ITS OWN REGISTERED COMPONENT (not an internal non-registering part like `sb-cell`, N-064):
  // a face CARRIES STATE (its command, its disabled flag, its roving `active` flag) and each face
  // must be independently CDP-readable. sb-cell registers nothing because it is a value-less flex
  // group; a face is not.
  //
  // WHY IT DOES NOT REUSE THE `button` core (grounded, button.svelte): `button` renders `{label}`
  // TEXT only — no children snippet, no icon child, no tabindex, and native `disabled`. A face is
  // icon-only, its tabindex is owned by the STRIP (roving), and it must stay focusable while
  // disabled (below). The shipped precedent for exactly this is `menu-trigger` (inside menu.svelte)
  // and `menu-item`: a chrome component that owns its own <button>/<li> because roving + ARIA role
  // belong to the parent.
  //
  // DISABLED = aria-disabled, NOT native `disabled` (grounded: menu-item.svelte does exactly this).
  // A natively-disabled <button> is NOT focusable; at M-RP6.1j EVERY face on the bottom shelf is
  // disabled, so native `disabled` would leave the strip with no focusable element — invisible to
  // the keyboard. A disabled face must stay focusable so its aria-label can be read: the disabled
  // state is a COUNTDOWN (its command does not exist yet, W-8), and a countdown nobody can reach
  // communicates nothing. Therefore aria-disabled + a GUARDED onclick (native Enter/Space route
  // through the click, so the one guard covers both) + skin hooks on [aria-disabled="true"].
  //
  // Faces dispatch COMMANDS, never a bare onclick (S-7): the face calls onActivate; the SHELF owns
  // the face→command mapping and turns it into onCommand(command). `command` is passed down purely
  // so the face can PUBLISH it in its getter — the thing CDP asserts a click dispatched. One
  // dispatch path.
  //
  // The type-class is supplied by `envelope` (N-023); no `class` hardcoded. No component <style>:
  // all appearance is skin, keyed by `.shelf-face` in the one skin file (N-025 / N-090).
  import { envelope } from '$common/components/base/envelope';
  import Icon from './icon.svelte';

  let {
    icon,
    label,
    command,
    active = false,
    disabled = false,
    pressed = false,
    onActivate,
    ref = $bindable(),
    id,
  }: {
    /** Glyph name (icons.ts key) → <Icon name={icon} />. A face IS an icon. */
    icon: string;
    /** Accessible name → aria-label (the button has no visible text). */
    label: string;
    /** The opaque command id this face publishes (dispatched by the shelf on activate). */
    command: string;
    /** Roving-tabindex active flag, OWNED BY shelf: active → tabindex 0, else -1. */
    active?: boolean;
    disabled?: boolean;
    /** Toggle latch (M-RP7.6, the FIRST stateful face) → aria-pressed. A DIFFERENT axis from `active`
     *  (roving) — a face can be pressed and unfocused, or focused and unpressed. Skin hooks on
     *  [aria-pressed="true"]; no glyph swap here (that is a skin decision, deferred to M-RP-SKIN). */
    pressed?: boolean;
    /** Activation callback (suppressed while disabled); the shelf turns it into onCommand(command). */
    onActivate?: () => void;
    /** The face's <button> element, exposed to the shelf for programmatic roving focus. */
    ref?: HTMLButtonElement;
    id?: string;
  } = $props();

  // N-024 opt-in. Config/task-state only (icon/led plain-return form): `command` is what CDP asserts
  // a click dispatched; `active`/`disabled` are the roving + phase-limit flags; `pressed` is the latch.
  const debug = () => ({ command, hasIcon: icon != null, disabled, active, pressed });
</script>

<button
  bind:this={ref}
  use:envelope={{ name: 'shelf-face', id, debug }}
  type="button"
  aria-label={label}
  tabindex={active ? 0 : -1}
  aria-disabled={disabled || undefined}
  aria-pressed={pressed || undefined}
  onclick={() => !disabled && onActivate?.()}
>
  <Icon name={icon} id={id ? `${id}__icon` : undefined} />
</button>
