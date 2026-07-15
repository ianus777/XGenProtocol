<script lang="ts">
  // shelf — data-independent chrome (M-RP6.1i): the ordered command STRIP that frames the widget
  // grid. Top shelf = user favourites; bottom shelf = system commands. Frame chrome OUTSIDE the
  // Layout descriptor (S-1, the menu-bar/status-bar posture, D-107): the controls that govern the
  // grid must not be dockable by the grid. Verified in the sampler here; the real client mount is
  // M-RP6.1j.
  //
  // WHAT IT IS NOT (region-dock §0 / surfaces §1–2):
  //   - NOT a grid (S-3): no leaf/split/tabs. A 1-D ordered strip. It never takes a Layout descriptor.
  //   - NOT a status-bar: a status-bar is passive DISPLAY; a shelf is an active COMMAND surface →
  //     ARIA role="toolbar" + roving tabindex + arrow traversal.
  //   - NOT a menu-bar: no popups, no dismiss policy, no focus-return. A shelf has NO owned-popup
  //     machine (which is why N-086's shared-W-2 extraction stays not-triggered).
  //   - NOT a second dock (S-4): a face is icon + click, never a shrunken widget.
  //
  // THE KEYBOARD MACHINE is menu-bar's LINEAR rove, copied and nothing else (D3): activeIndex + a
  // faces[] ref array + ArrowLeft/ArrowRight/Home/End, wrap-around, preventDefault, focus() on the
  // target. Enter/Space activate the focused face NATIVELY (the <button> does this) → not handled
  // here. The menu.svelte half (open/dismiss/outside-click/focus-return) has NO shelf counterpart —
  // a face has no popup — so it is not imported, adapted, or seamed for. Roving does NOT skip
  // disabled faces: a disabled toolbar button stays focusable + reachable (its aria-label must be
  // readable), and at 6.1j the bottom shelf is ENTIRELY disabled — skipping would make it
  // keyboard-unreachable.
  //
  // Faces dispatch COMMANDS, never a bare onclick (S-7): `onCommand` is byte-for-byte the menu-bar
  // seam (app_client wires onCommand → commandTable). A shelf button and a menu item become one
  // command with two triggers, and an accelerator is free. The shelf imports no Tauri, no store,
  // no command table — it emits a commandId and forgets.
  //
  // EMPTY = MOUNTED, not absent (D4 / N-053): the root always renders (never {#if} a registered
  // root — the registry stays complete); `data-empty` reflects items.length === 0 and the skin
  // collapses it (the status.svelte expired→mounted-but-empty precedent). This is what lets the top
  // shelf mount empty at 6.1j without a dead control and without a hole in the registry.
  //
  // NO badge this milestone (D6): nothing produces a count, and unread counts have no protocol
  // mechanism (the read-marker gap, J-503) — a badge prop with no feeder is the N-097 shape. No
  // prop, no skin rule, no reserved slot.
  //
  // The type-class is supplied by `envelope` (N-023); no `class` hardcoded. No component <style>:
  // all appearance is skin, keyed by `.shelf` in the one skin file (N-025 / N-090).
  import { envelope } from '$common/components/base/envelope';
  import ShelfFace from './shelf-face.svelte';

  type ShelfItemDef = {
    /** Glyph name (an icons.ts key). Required — a face IS an icon. */
    icon: string;
    /** Accessible name. Required — an icon-only button with no label is unreachable. */
    label: string;
    /** The opaque command id dispatched via onCommand on activate. */
    command: string;
    disabled?: boolean;
    /** Toggle-latch state (M-RP7.6) → the face's aria-pressed. Absent ⇒ not a toggle. */
    pressed?: boolean;
  };

  let {
    items = [],
    position = 'bottom',
    ariaLabel,
    onCommand,
    id,
  }: {
    /** Ordered faces, left→right. */
    items?: ShelfItemDef[];
    /** 'top' (favourites) | 'bottom' (system) → reflects to data-position. */
    position?: 'top' | 'bottom';
    /** The toolbar's accessible name. */
    ariaLabel?: string;
    /** Dispatch an activated face's command id up to the shell's command table. */
    onCommand?: (command: string | undefined) => void;
    id?: string;
  } = $props();

  let activeIndex = $state(0); // the roving-focused face (roving tabindex)
  let faces: HTMLButtonElement[] = []; // face <button> refs, for Left/Right focus moves

  const cid = (i: number) => (id ? `${id}__${i}` : undefined);

  function focusFace(i: number) {
    faces[i]?.focus?.();
  }

  function onStripKey(e: KeyboardEvent) {
    // Only rove when a face itself is focused. (No popups here, so this is defensive parity with
    // menu-bar; every child of the strip is a face.)
    if (!faces.includes(e.target as HTMLButtonElement)) return;
    const n = items.length;
    if (n === 0) return; // empty strip: nothing to focus, never throw
    switch (e.key) {
      case 'ArrowRight':
        e.preventDefault();
        activeIndex = (activeIndex + 1) % n;
        focusFace(activeIndex);
        break;
      case 'ArrowLeft':
        e.preventDefault();
        activeIndex = (activeIndex - 1 + n) % n;
        focusFace(activeIndex);
        break;
      case 'Home':
        e.preventDefault();
        activeIndex = 0;
        focusFace(0);
        break;
      case 'End':
        e.preventDefault();
        activeIndex = n - 1;
        focusFace(n - 1);
        break;
      default:
        break;
    }
  }

  // G — aggregate getter (task-state only, N-060): the strip's edge, its face count, and which face
  // has roving focus. The per-face command/disabled/active ride each shelf-face's own getter.
  const debug = () => ({ position, itemCount: items.length, activeIndex });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  use:envelope={{ name: 'shelf', id, debug }}
  role="toolbar"
  aria-orientation="horizontal"
  aria-label={ariaLabel}
  data-position={position}
  data-empty={items.length === 0 || undefined}
  onkeydown={onStripKey}
>
  {#each items as item, i (item.command)}
    <ShelfFace
      icon={item.icon}
      label={item.label}
      command={item.command}
      disabled={item.disabled ?? false}
      pressed={item.pressed ?? false}
      active={activeIndex === i}
      onActivate={() => onCommand?.(item.command)}
      bind:ref={faces[i]}
      id={cid(i)}
    />
  {/each}
</div>
