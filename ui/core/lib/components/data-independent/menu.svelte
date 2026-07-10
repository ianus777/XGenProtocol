<script lang="ts">
  // menu — data-independent chrome (M-RP6.1d): ONE menubar entry = a trigger button + an owned
  // popup (`<ul role="menu">`) of `menu-item` rows, plus the fresh-minimal behaviour machine that
  // drives them. Part of the `core` menu trio (menu-bar / menu / menu-item) that assembles the
  // client's fixed top-pane menu-bar (frame Phase-0 §4.1, D-107).
  //
  // FRESH-MINIMAL MACHINE, NOT a refactor of the closed widget (runbook §2.1 / A2). The W-2 machine
  // (open → roving → dispatch → close, dismiss policy, focus-return, wire/teardown) currently lives
  // ENTIRELY inside the CLOSED, verified `entity-context-menu` widget, interwoven with its dd
  // concerns (EntityDescriptor header, status, async onSelect, portal). There is no extracted shared
  // module. This builds a small self-contained sync machine here instead — a second concrete machine.
  // DEFERRED (D-065): extract a shared W-2 / owned-popup module when the 2nd populated menu or a
  // submenu-flyout forces the abstraction; entity-context-menu + menu both adopt it then. No
  // entity-context-menu change this milestone.
  //
  // OPEN IS PARENT-CONTROLLED (menu-bar owns single-open mutual exclusion): `open` is a prop, and
  // the machine requests transitions via `onRequestOpen` / `onRequestClose`. Everything AFTER the
  // request — roving, dispatch, dismiss, focus-return, listener lifecycle — the menu owns.
  //
  // Machine (runbook §2.1):
  //   - open on trigger click / Enter / Space / ArrowDown → focus moves into the first item.
  //   - roving Down/Up (+ Home/End) over items (roving tabindex, entity-panel precedent).
  //   - dispatch on item Enter/Space/click → run the item's command via `onCommand` → close.
  //   - dismiss Esc / outside-click / focus-leaves-the-menu; focus RETURNS to the trigger.
  //   - wire the outside-click listener ON OPEN, tear it down on close/unmount (0 orphans, W-5).
  //
  // Popup positioning is inline `position:absolute` below the trigger (runbook §2.3, C1) — NO portal
  // (the menu-bar is the fixed top pane; a short downward dropdown needs no relocation). If the
  // top-pane / window `overflow` clips it in the real client, portal-to-body is the deferred fix.
  //
  // The type-class is supplied by `envelope` (N-023); no `class` hardcoded. All appearance is skin,
  // keyed by `.menu` / `.menu-trigger` / `.menu-popup` in the one skin file (N-025).
  import { envelope } from '$common/components/base/envelope';
  import MenuItem from './menu-item.svelte';
  import type { Accelerator, Platform } from '$common/keymap/accelerator';

  /** One popup row. `command` is the opaque id dispatched via `onCommand` on select. */
  type MenuItemDef = {
    label: string;
    icon?: string;
    accelerator?: Accelerator;
    command?: string;
    disabled?: boolean;
  };

  let {
    label,
    items = [],
    platform = 'win',
    open = false,
    active = false,
    onRequestOpen,
    onRequestClose,
    onTriggerFocus,
    onCommand,
    triggerRef = $bindable(),
    id,
  }: {
    /** The trigger caption (e.g. "File"). */
    label: string;
    items?: MenuItemDef[];
    platform?: Platform;
    /** Open state — controlled by menu-bar (single-open mutual exclusion). */
    open?: boolean;
    /** Roving-tabindex active flag for the trigger (owned by menu-bar). */
    active?: boolean;
    onRequestOpen?: () => void;
    onRequestClose?: () => void;
    /** Notify menu-bar the trigger took focus (keeps the roving tabindex in sync). */
    onTriggerFocus?: () => void;
    /** Dispatch a chosen item's command id (the shell's command table runs it). */
    onCommand?: (command: string | undefined) => void;
    /** The trigger `<button>`, exposed to menu-bar for roving Left/Right focus. */
    triggerRef?: HTMLButtonElement;
    id?: string;
  } = $props();

  let activeIndex = $state(-1); // roving item position within the popup (transient); -1 = none
  let wrapperEl: HTMLDivElement | undefined; // the `.menu` root — containment checks for dismiss
  let rowEls: HTMLLIElement[] = []; // per-row `<li>` refs for programmatic roving focus

  const cid = (s: string) => (id ? `${id}-${s}` : undefined);
  const itemKey = (s: string) => s.toLowerCase().replace(/\s+/g, '-');

  function focusItem(i: number) {
    rowEls[i]?.focus();
  }

  // Internal transitions set the roving position, then request the parent flip the open flag.
  function doOpen() {
    if (open) return;
    activeIndex = 0; // clamped to a real row by the focus effect once items render
    onRequestOpen?.();
  }
  function doClose(returnFocus: boolean) {
    activeIndex = -1;
    if (returnFocus) triggerRef?.focus();
    onRequestClose?.();
  }

  function selectAt(i: number) {
    const item = items[i];
    if (!item || item.disabled) return;
    onCommand?.(item.command);
    doClose(true);
  }

  function onTriggerKey(e: KeyboardEvent) {
    switch (e.key) {
      case 'Enter':
      case ' ':
      case 'ArrowDown':
        e.preventDefault();
        if (!open) doOpen();
        else focusItem(activeIndex >= 0 ? activeIndex : 0);
        break;
      case 'Escape':
        if (open) {
          e.preventDefault();
          doClose(true);
        }
        break;
      default:
        break;
    }
  }

  function onPopupKey(e: KeyboardEvent) {
    const n = items.length;
    switch (e.key) {
      case 'Escape':
        e.preventDefault();
        doClose(true);
        break;
      case 'ArrowDown':
        e.preventDefault();
        if (n) {
          activeIndex = Math.min(n - 1, activeIndex + 1);
          focusItem(activeIndex);
        }
        break;
      case 'ArrowUp':
        e.preventDefault();
        if (n) {
          activeIndex = Math.max(0, activeIndex - 1);
          focusItem(activeIndex);
        }
        break;
      case 'Home':
        e.preventDefault();
        if (n) {
          activeIndex = 0;
          focusItem(0);
        }
        break;
      case 'End':
        e.preventDefault();
        if (n) {
          activeIndex = n - 1;
          focusItem(n - 1);
        }
        break;
      case 'Enter':
      case ' ':
        e.preventDefault();
        selectAt(activeIndex);
        break;
      default:
        break;
    }
  }

  // Dismiss on focus-leave: moving focus OUT of the `.menu` wrapper closes it; roving between the
  // trigger and its own rows keeps relatedTarget inside → no close. Don't yank focus (it already left).
  function onFocusOut(e: FocusEvent) {
    const next = e.relatedTarget as Node | null;
    if (open && next && wrapperEl && !wrapperEl.contains(next)) doClose(false);
  }

  // W-5 lifecycle: wire the outside-click (document mousedown) listener + move focus into the first
  // item ONLY while open; tear both down the instant the menu closes OR the component unmounts (the
  // effect cleanup covers both) → 0 orphan listeners. The trigger is exempt so its own click doesn't
  // immediately re-dismiss.
  $effect(() => {
    if (!open) return;
    queueMicrotask(() => {
      if (open) focusItem(activeIndex >= 0 ? activeIndex : 0);
    });
    const onDocDown = (e: MouseEvent) => {
      const t = e.target as Node | null;
      if (t && wrapperEl?.contains(t)) return;
      doClose(false);
    };
    document.addEventListener('mousedown', onDocDown, true);
    return () => document.removeEventListener('mousedown', onDocDown, true);
  });

  // G — aggregate getter (task-state only, N-060): the label, open flag, item count, roving position.
  const debug = () => ({
    label,
    open,
    itemCount: items.length,
    activeIndex,
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  bind:this={wrapperEl}
  use:envelope={{ name: 'menu', id, debug }}
  onfocusout={onFocusOut}
>
  <button
    bind:this={triggerRef}
    class="menu-trigger"
    type="button"
    role="menuitem"
    aria-haspopup="menu"
    aria-expanded={open}
    tabindex={active ? 0 : -1}
    onclick={() => (open ? doClose(true) : doOpen())}
    onkeydown={onTriggerKey}
    onfocus={() => onTriggerFocus?.()}
  >
    {label}
  </button>

  {#if open}
    <ul class="menu-popup" role="menu" aria-label={label} onkeydown={onPopupKey}>
      {#each items as item, i (item.label)}
        <MenuItem
          label={item.label}
          icon={item.icon}
          accelerator={item.accelerator}
          {platform}
          active={i === activeIndex}
          disabled={item.disabled}
          onSelect={() => selectAt(i)}
          bind:ref={rowEls[i]}
          id={cid(itemKey(item.label))}
        />
      {/each}
    </ul>
  {/if}
</div>
