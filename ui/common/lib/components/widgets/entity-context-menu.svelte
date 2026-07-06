<script lang="ts">
  // entity-context-menu — the SECOND `widget` (M-RP5.3), and the FIRST widget with a real
  // dd-dependency: it is the first genuine consumer of the W-11 dd-socket (binds an
  // `EntityDescriptor` + a self-status view-model). Home ui/common/lib/components/widgets/.
  // Design lock: docs/entity-context-menu-phase0.md (A→H), widget spec ui/docs/xgen-widget-tier.md.
  //
  // WIDGET, not a composite (W-2 discriminator, Phase-0 §2 A). Roving focus alone is NOT the
  // line — `entity-panel` already owns roving focus. Three things push this over W-2:
  //   (1) an overlay mount/unmount LIFECYCLE (appear → focus-trap → dismiss-policy → tear down);
  //   (2) it DISPATCHES host-integrating side effects (message / block / navigate);
  //   (3) it is the first real W-11 dd-socket consumer.
  // Remove it and you lose a BEHAVIOUR, not a layout. Level-2 marked via `data-tier="widget"`.
  //
  // GESTURE-AGNOSTIC (Phase-0 §2 B). The widget exposes `open(anchor)` / `close()`; the CONSUMER
  // wires the gesture (avatar/item `onActivate?`, right-click, long-press) to `open`. The widget
  // owns everything AFTER open, never the gesture itself — which keeps it sampler-testable without
  // a real right-click. `open()`/`close()` are exported instance methods (reached via `bind:this`);
  // the debug registry exposes state only, so the sampler wires a trigger and CDP drives by
  // dispatching real events + reading getter G.
  //
  // I/O SEAM (W-3). Phase A — pure Svelte, no `invoke`. Action handlers are CONSUMER-INJECTED
  // callbacks (the `substitutions-editor` `onApply` precedent); the widget calls the callback,
  // never a shell dep. The sampler passes stubs → pure layer; the real shell backs them.
  //
  // GETTER G (W-4, Phase-0 §2 G). One aggregate {open, variant, purpose, kind, itemCount,
  // activeIndex} — observable task-state ONLY, never payload/entity secrets (N-060 precedent).
  // Child `core` self-register under `<id>__<slot>` (`__avatar`, `__status`).
  //
  // ── STEP 3 (this file): HEADER + the universal `identity` item + dispatch ────────────────────
  // The menu = a HEADER (composes `entity-avatar` + name + `status` **full** — the identity-read
  // surface, Phase-0 §2 E) over the item list. The base ships exactly ONE item — the universal
  // full-entity-read (`identity`), kind-labelled; the space/room labels are RESERVED (provisional
  // here) and the flag-gated per-(variant,purpose) slots are declared-not-populated (W-8). Item
  // rows are widget-owned `<li role="menuitem">` (dispatch is the widget's contract); the header
  // still composes `core` per W-1. Children self-register `__avatar` / `__status` (Phase-0 §2 G).
  //
  // Behaviour machine D (Phase-0 §2 D) — Step 2, retained below:
  //   closed → open(anchored, focus moved in) → navigating(roving) → dispatch(run handler) → closed
  // Dismiss: Esc · outside-click · select-then-close · focus-leaves. Focus returns to the anchor on
  // close; listeners wire on open, tear down on close/unmount (W-5).
  //
  // ── STEP 5 (this file): PORTAL + viewport flip/shift (Phase-0 §2 C, effect layer) ────────────
  // Avatars live inside `overflow` panels/scrollers that would CLIP an inline popup. `portal` (opt-in)
  // relocates the root to `document.body` (escaping every ancestor clip) and switches it to
  // `position: fixed`, then places it against the anchor's viewport rect with flip (below→above on
  // bottom overflow) + shift (clamp into the viewport on right/left overflow). Phase-0's two-layer
  // split is honoured exactly: the SAMPLER pure layer runs `portal={false}` (an inline
  // `position:absolute` popup, no relocation — the verified Step-4 path); the REAL shell runs
  // `portal={true}` (portal + fixed + flip/shift — the effect layer). One skin, gated by `data-portal`.
  import { envelope } from '$common/components/base/envelope';
  import EntityAvatar from '$core/components/data-dependent/entity-avatar.svelte';
  import Status from '$core/components/data-dependent/status.svelte';
  import type { EntityDescriptor } from '$core/components/data-dependent/types';

  /** An anchor is either a live element (rect read lazily) or an explicit viewport point. */
  export type MenuAnchor = HTMLElement | DOMRect | { x: number; y: number };

  /** A single menu row. Label/derivation are populated in Step 3; the machine only needs id+label. */
  type MenuItem = { id: string; label: string; disabled?: boolean };

  let {
    descriptor,
    status,
    variant = 'card',
    purpose = 'default',
    portal = false,
    onSelect,
    onClose,
    id,
  }: {
    /** The entity this menu reads (W-11 dd-socket payload). */
    descriptor: EntityDescriptor;
    /** Optional self-status view-model (Track A) — the header `status full` read. */
    status?: { emoji?: string; text?: string; updatedAt?: string; expiresAt?: string };
    /** Header density (mirrors the avatar axis). Declared; catalogue grows with usage. */
    variant?: 'presence' | 'list' | 'labeled' | 'card';
    /** Action-context (a superset of the avatar axis) — gates the item-set. Declared-not-populated. */
    purpose?: string;
    /** Relocate to `document.body` + `position:fixed` + flip/shift (effect layer). Default inline. */
    portal?: boolean;
    /** Consumer-injected dispatch: an item was chosen (W-3 callback seam). */
    onSelect?: (itemId: string) => void | Promise<void>;
    /** Consumer notification that the menu closed (focus-return is the consumer's to honour). */
    onClose?: () => void;
    id?: string;
  } = $props();

  const kind = $derived(descriptor?.kind);
  const name = $derived(descriptor?.name);

  // Composite-derived stable child ids (`<id>__avatar`, `<id>__status`) — Phase-0 §2 G.
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // ── Item derivation ────────────────────────────────────────────────────────────────────────
  // buildItems() returns f(variant, purpose) gated by (kind, flags). The base seeds ONLY the
  // universal full-read item (`identity`); the per-(variant,purpose) flag-gated slots are reserved
  // (declared-not-populated, W-8). Its label is concrete for identity kind; the space/room labels
  // are RESERVED (provisional here per Phase-0 §2 E). `itemCount`/`activeIndex` in G read off this.
  const READ_LABEL: Record<string, string> = {
    identity: 'View identity',
    space: 'View space', // reserved/provisional
    room: 'View room', // reserved/provisional
  };
  function buildItems(): MenuItem[] {
    if (!kind) return [];
    return [{ id: 'identity', label: READ_LABEL[kind] ?? 'View' }];
  }
  const items = $derived<MenuItem[]>(buildItems());
  const itemCount = $derived(items.length);

  // ── Overlay task-state (W-2 machine) ───────────────────────────────────────────────────────
  let isOpen = $state(false);
  let anchor: MenuAnchor | undefined; // stashed on open (Step 5 reads it for flip/shift)
  let activeIndex = $state(-1); // roving position (transient); −1 = none
  let menuEl: HTMLDivElement | undefined; // root, for containment checks + programmatic focus
  let rowEls: HTMLLIElement[] = []; // per-row refs for roving focus
  let returnFocusEl: HTMLElement | null = null; // focus-return target captured at open

  /**
   * Open the menu anchored to `a` (gesture-agnostic entry point — the consumer's trigger calls it).
   * Captures the focus-return target, seeds the active row, and moves focus in (via the effect below
   * once the rows have rendered). Idempotent re-anchor if already open.
   */
  export function open(a: MenuAnchor): void {
    if (!isOpen) returnFocusEl = document.activeElement as HTMLElement | null;
    anchor = a;
    activeIndex = 0; // clamped to a real row by the focus effect / onKey once items exist
    isOpen = true;
  }

  /** Close the menu, tear the machine down, and return focus to the anchor (Phase-0 §2 D). */
  export function close(): void {
    if (!isOpen) return;
    isOpen = false;
    activeIndex = -1;
    const back = anchor instanceof HTMLElement ? anchor : returnFocusEl;
    back?.focus?.();
    onClose?.();
  }

  function focusRow(i: number): void {
    (rowEls[i] ?? menuEl)?.focus();
  }

  // Dispatch-and-close (Phase-0 §2 D): run the consumer handler, then close + return focus. The
  // handler may be async (navigate/open) — fire-and-forget so the machine tears down promptly.
  function selectAt(i: number): void {
    const it = items[i];
    if (!it || it.disabled) return;
    void onSelect?.(it.id);
    close();
  }

  // F — roving keyboard, centralised on the root so it fires wherever focus sits inside the menu.
  function onKey(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      e.preventDefault();
      close();
      return;
    }
    const n = items.length;
    if (n === 0) return;
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        activeIndex = Math.min(n - 1, activeIndex + 1);
        focusRow(activeIndex);
        break;
      case 'ArrowUp':
        e.preventDefault();
        activeIndex = Math.max(0, activeIndex - 1);
        focusRow(activeIndex);
        break;
      case 'Home':
        e.preventDefault();
        activeIndex = 0;
        focusRow(0);
        break;
      case 'End':
        e.preventDefault();
        activeIndex = n - 1;
        focusRow(n - 1);
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

  // Dismiss: focus-leaves (the "anchor blur" essence) — moving focus OUT of the menu closes it;
  // roving between rows keeps relatedTarget inside → no close.
  function onFocusOut(e: FocusEvent): void {
    const next = e.relatedTarget as Node | null;
    if (next && menuEl && !menuEl.contains(next)) close();
  }

  // ── Portal + placement (effect layer, `portal` opt-in) ─────────────────────────────────────
  // Svelte action: relocate the root to `document.body` on mount so it escapes every ancestor
  // `overflow` clip; remove it on destroy (the component's own teardown still fires). No-op when
  // `portal` is false → the inline sampler path is untouched. `portal` is static per instance.
  function portalAction(node: HTMLElement) {
    if (!portal) return {};
    document.body.appendChild(node);
    return { destroy: () => node.remove() };
  }

  function anchorRect(a: MenuAnchor): { top: number; bottom: number; left: number; width: number } {
    if (a instanceof HTMLElement) {
      const r = a.getBoundingClientRect();
      return { top: r.top, bottom: r.bottom, left: r.left, width: r.width };
    }
    if (typeof DOMRect !== 'undefined' && a instanceof DOMRect) {
      return { top: a.top, bottom: a.bottom, left: a.left, width: a.width };
    }
    const p = a as { x: number; y: number };
    return { top: p.y, bottom: p.y, left: p.x, width: 0 };
  }

  // Place the fixed-positioned portal against the anchor's viewport rect: below-left by default,
  // FLIP above on bottom overflow, SHIFT to clamp into the viewport on right/left overflow.
  function reposition(): void {
    if (!portal || !menuEl || !anchor) return;
    const r = anchorRect(anchor);
    const mw = menuEl.offsetWidth;
    const mh = menuEl.offsetHeight;
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const m = 4;
    let top = r.bottom + m;
    if (top + mh > vh - m && r.top - mh - m >= m) top = r.top - mh - m; // flip above
    let left = r.left;
    if (left + mw > vw - m) left = vw - mw - m; // shift left
    if (left < m) left = m;
    menuEl.style.top = `${Math.max(m, top)}px`;
    menuEl.style.left = `${left}px`;
  }

  // W-5 lifecycle: the outside-click (document mousedown) listener + the focus-move-in are wired
  // when the menu is OPEN and torn down the instant it closes OR the component unmounts (the effect
  // cleanup covers both) → 0 orphans. The anchor element is exempt from outside-click so the
  // trigger's own gesture doesn't immediately re-dismiss.
  $effect(() => {
    if (!isOpen) return;
    const onDocDown = (e: MouseEvent) => {
      const t = e.target as Node | null;
      if (t && menuEl?.contains(t)) return;
      if (t && anchor instanceof HTMLElement && anchor.contains(t)) return;
      close();
    };
    document.addEventListener('mousedown', onDocDown, true);
    // After the rows have rendered: place the portal (before paint — microtask beats paint, no
    // flash) then move focus IN (active row, else the root).
    queueMicrotask(() => {
      if (!isOpen) return;
      reposition();
      focusRow(activeIndex);
    });
    return () => document.removeEventListener('mousedown', onDocDown, true);
  });

  // G — the widget self-registers ONE aggregate getter. State only; no descriptor/handler payload.
  const debug = () => ({
    open: isOpen,
    variant,
    purpose,
    kind: kind ?? null,
    itemCount,
    activeIndex,
  });
</script>

<!-- C — root: `<div class="entity-context-menu" role="menu">`, Level-2 via `data-tier`. The
  registered root ALWAYS mounts (the `status.svelte` mounted-but-empty precedent, N-053) so getter G
  stays CDP-readable while CLOSED (`{open:false}` after Esc) and the registry count stays stable —
  `data-open` reflects state, the skin (Step 4) collapses the closed menu. The popup CONTENT (header
  + items) is what's conditional. `tabindex=-1` makes the root programmatically focusable so focus
  has a home when the item list is empty. Roving keydown + focus-leave dismiss are centralised here.
  Portal-to-body + viewport flip/shift stay for the effect layer (Step 5). -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  bind:this={menuEl}
  use:envelope={{ name: 'entity-context-menu', id, debug }}
  use:portalAction
  data-tier="widget"
  data-open={isOpen || undefined}
  data-portal={portal || undefined}
  role="menu"
  tabindex="-1"
  aria-label={descriptor?.name ?? kind}
  aria-hidden={!isOpen || undefined}
  onkeydown={onKey}
  onfocusout={onFocusOut}
>
  {#if isOpen}
    <!-- Header (Phase-0 §2 E): the identity-read surface. Composes `entity-avatar` (no corner
      status — the `full` line below carries status, and this keeps exactly `__avatar`/`__status`
      children per G) + the name + `status` `full`. `status ?? {}` keeps `__status` always
      registered; the status atomic collapses itself (mounted-but-empty) when there's nothing. -->
    <div class="ecm-header" role="presentation">
      <EntityAvatar {descriptor} {variant} id={cid('avatar')} />
      <div class="ecm-heading">
        <span class="ecm-name">{name ?? kind}</span>
        <Status status={status ?? {}} variant="full" id={cid('status')} />
      </div>
    </div>
    <ul class="ecm-items" role="none">
      {#each items as item, i (item.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <li
          bind:this={rowEls[i]}
          role="menuitem"
          class="ecm-item"
          tabindex={i === activeIndex ? 0 : -1}
          aria-disabled={item.disabled || undefined}
          onclick={() => selectAt(i)}
        >
          {item.label}
        </li>
      {/each}
    </ul>
  {/if}
</div>
