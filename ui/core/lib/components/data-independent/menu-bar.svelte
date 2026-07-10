<script lang="ts">
  // menu-bar — data-independent chrome (M-RP6.1d): the top-pane horizontal strip
  // (`<div role="menubar">`) that hosts the `menu` triggers and owns roving Left/Right across them.
  // Top of the `core` menu trio (menu-bar / menu / menu-item); mounted into the client's fixed top
  // pane (frame chrome OUTSIDE the Layout descriptor — region-dock model §2, D-107). 6.1e adds the
  // bottom status-bar, 6.1f the center region.
  //
  // Single-open mutual exclusion lives HERE: menu-bar owns `openIndex` (at most one popup open) and
  // `activeIndex` (which trigger has roving focus). Each `menu` is `open={openIndex === i}` and
  // requests transitions back — so opening File closes any other. For the minimal File-only bar this
  // is trivially satisfied, but the model is correct for the populated menus to come.
  //
  // Roving Left/Right (runbook §2.1): handled on the menubar (keydown bubbles up from the focused
  // trigger). It fires ONLY when a trigger itself is the event target — an OPEN popup routes its own
  // Up/Down/Enter/Esc and must not move the bar. Home/End jump to the first/last trigger. Down opens
  // the focused menu (handled inside `menu`'s trigger keydown).
  //
  // FRAME CHROME, not a sampler cell (Joe-locked, runbook §4): no `app_sampler.svelte` grid cell; the
  // sampler catalogue registry stays 299. Appearance + behaviour are verified in the real client
  // (9222) via CDP structure + eyecheck (§5).
  //
  // The type-class is supplied by `envelope` (N-023); no `class` hardcoded. All appearance is skin,
  // keyed by `.menu-bar` in the one skin file (N-025).
  import { envelope } from '$common/components/base/envelope';
  import Menu from './menu.svelte';
  import type { Accelerator, Platform } from '$common/keymap/accelerator';

  type MenuItemDef = {
    label: string;
    icon?: string;
    accelerator?: Accelerator;
    command?: string;
    disabled?: boolean;
  };
  type MenuDef = { label: string; items?: MenuItemDef[] };

  let {
    menus = [],
    platform = 'win',
    onCommand,
    id,
  }: {
    menus?: MenuDef[];
    platform?: Platform;
    /** Dispatch a chosen item's command id up to the shell's command table. */
    onCommand?: (command: string | undefined) => void;
    id?: string;
  } = $props();

  let openIndex = $state(-1); // the single open menu (mutual exclusion); -1 = all closed
  let activeIndex = $state(0); // the roving-focused trigger (roving tabindex)
  let triggers: HTMLButtonElement[] = []; // trigger `<button>` refs, for Left/Right focus moves

  const cid = (s: string) => (id ? `${id}__${s}` : undefined);
  const menuKey = (s: string) => s.toLowerCase().replace(/\s+/g, '-');

  function focusTrigger(i: number) {
    triggers[i]?.focus?.();
  }

  function onBarKey(e: KeyboardEvent) {
    // Only rove triggers when a trigger itself is focused — an open popup owns its own keys.
    if (!triggers.includes(e.target as HTMLButtonElement)) return;
    const n = menus.length;
    if (n === 0) return;
    switch (e.key) {
      case 'ArrowRight':
        e.preventDefault();
        activeIndex = (activeIndex + 1) % n;
        focusTrigger(activeIndex);
        break;
      case 'ArrowLeft':
        e.preventDefault();
        activeIndex = (activeIndex - 1 + n) % n;
        focusTrigger(activeIndex);
        break;
      case 'Home':
        e.preventDefault();
        activeIndex = 0;
        focusTrigger(0);
        break;
      case 'End':
        e.preventDefault();
        activeIndex = n - 1;
        focusTrigger(n - 1);
        break;
      default:
        break;
    }
  }

  // G — aggregate getter (task-state only): how many triggers, which has roving focus, which is open.
  const debug = () => ({
    items: menus.length,
    activeIndex,
    openIndex,
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  use:envelope={{ name: 'menu-bar', id, debug }}
  role="menubar"
  aria-label="Application"
  onkeydown={onBarKey}
>
  {#each menus as m, i (m.label)}
    <Menu
      label={m.label}
      items={m.items ?? []}
      {platform}
      open={openIndex === i}
      active={activeIndex === i}
      onRequestOpen={() => (openIndex = i)}
      onRequestClose={() => {
        if (openIndex === i) openIndex = -1;
      }}
      onTriggerFocus={() => (activeIndex = i)}
      {onCommand}
      bind:triggerRef={triggers[i]}
      id={cid(menuKey(m.label))}
    />
  {/each}
</div>
