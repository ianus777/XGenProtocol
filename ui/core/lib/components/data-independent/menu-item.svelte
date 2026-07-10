<script lang="ts">
  // menu-item — data-independent chrome (M-RP6.1d): one row of an owned menu popup. A
  // `<li role="menuitem">` = optional leading `icon` (6.1a, composed child) + a label +
  // an optional trailing ACCELERATOR HINT rendered from an `Accelerator` (6.1c) via
  // `toDisplay(platform)`. Part of the minimal `core` menu trio (menu-bar / menu / menu-item)
  // that assembles the client's fixed top-pane menu-bar (frame Phase-0 §4.1, D-107).
  //
  // NOT the closed `entity-context-menu` widget: that is a dd-widget (binds an EntityDescriptor,
  // portal, async dispatch). This trio is pure di chrome with a fresh-minimal machine (owned by
  // `menu`); menu-item is the leaf. Frame chrome, NOT a sampler cell — verified in the real
  // client (§4/§5 of the runbook), the sampler catalogue registry is untouched.
  //
  // The `<li role="menuitem">` root is exactly why `separator` was rooted `<div role="separator">`
  // and this is `<li>` — they slot into the parent `<ul role="menu">` with no branch.
  //
  // Focus/roving is OWNED BY `menu` (the roving-tabindex model, entity-panel precedent): `menu`
  // sets `active` (→ tabindex 0/-1) and reads back the row element via `bind:ref`. menu-item
  // reports selection via `onSelect` (click) and stays inert when `disabled`. Enter/Space are
  // handled centrally by `menu`'s popup keydown, so the row needs no key handler of its own.
  //
  // The type-class is supplied by `envelope` (N-023); no `class` hardcoded. All appearance is
  // skin, keyed by `.menu-item` / `.mi-label` / `.mi-accel` in the one skin file (N-025).
  import { envelope } from '$common/components/base/envelope';
  import Icon from './icon.svelte';
  import type { Accelerator, Platform } from '$common/keymap/accelerator';

  let {
    label,
    icon,
    accelerator,
    platform = 'win',
    active = false,
    disabled = false,
    onSelect,
    ref = $bindable(),
    id,
  }: {
    /** The item caption. */
    label: string;
    /** Optional leading glyph name (icons.ts key). Absent → no icon slot. */
    icon?: string;
    /** Optional accelerator whose `toDisplay(platform)` becomes the trailing hint. */
    accelerator?: Accelerator;
    /** Display platform for the hint (win → "Ctrl+Q", mac → "⌘Q"). */
    platform?: Platform;
    /** Roving-tabindex active flag (owned by `menu`): active → tabindex 0, else -1. */
    active?: boolean;
    disabled?: boolean;
    /** Click/selection callback (suppressed while disabled). */
    onSelect?: () => void;
    /** The row's `<li>` element, exposed to `menu` for programmatic roving focus. */
    ref?: HTMLLIElement;
    id?: string;
  } = $props();

  const accelText = $derived(accelerator ? accelerator.toDisplay(platform) : null);

  // N-024 opt-in. Config/task-state only (no value carried): the hint text is reported as `accel`
  // so the real-client CDP can confirm the accelerator surfaced (§5). Plain-return (icon/led form).
  const debug = () => ({
    label,
    hasIcon: icon != null,
    accel: accelText,
    disabled,
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<li
  bind:this={ref}
  use:envelope={{ name: 'menu-item', id, debug }}
  role="menuitem"
  tabindex={active ? 0 : -1}
  aria-disabled={disabled || undefined}
  onclick={() => !disabled && onSelect?.()}
>
  {#if icon}<Icon name={icon} id={id ? `${id}__icon` : undefined} />{/if}
  <span class="mi-label">{label}</span>
  {#if accelText}<kbd class="mi-accel">{accelText}</kbd>{/if}
</li>
