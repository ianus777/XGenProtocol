<script lang="ts">
  // status-bar — data-independent, COMPOSITE (M-RP6.1e-A): the fixed bottom-pane strip and
  // the first component of the M-RP6.1e client-frame consolidation split (D-107). Root IS
  // `<div class="status-bar">` — the N-020/N-022 composite marker. Binding none; **di** — the
  // caller supplies the connection state→colour map + caption (and an optional secondary
  // label); the bar interprets no domain structure.
  //
  // STRUCTURE (frame Phase-0 §4.5, Joe-locked contents): two `sb-cell` groups —
  //   left  → a composed `status-indicator` (led + label; the connection light + caption),
  //           and OPTIONALLY a vertical `separator` + a secondary `label` when `secondaryText`
  //           is set (e.g. a version tag — this is what exercises the vertical separator).
  //   right → the always-visible SE resize-grip.
  // `sb-cell` is an internal, NON-registering layout part (N-064); only the composed REAL
  // atomics register, under composite-derived stable ids (the status-indicator precedent):
  //   status-indicator#<id>__status-indicator  (+ its own __led / __label children)
  //   separator#<id>__sep + label#<id>__secondary   (only when secondaryText is set)
  // A status-bar row therefore yields MULTIPLE registry entries — measure, don't predict
  // (Rule 5); the CDP re-drive states the real delta.
  //
  // RESIZE-GRIP SEAM (§4.5) — the bar imports NO Tauri. The grip's `onpointerdown` calls
  // `onResizeGrip?.(event)`; the bar does nothing else with it. In the real client (6.1e-B)
  // the shell wires `onResizeGrip` → Tauri `startResizeDragging` (SE corner = width+height).
  // A11y honesty: the grip is POINTER-ONLY — window resize has no keyboard equivalent here
  // (that is an OS concern), so the grip is `aria-hidden` and NOT a focusable/keyboard control;
  // it is not faking an affordance it does not have.
  //
  // The grip glyph is drawn in `skin.css` (a neutral-token SE-corner triangle, NOT accent) —
  // appearance is skin-only (N-025), CSS not an `icon` (the grip is a positioned corner
  // affordance, not an inline-with-text glyph; a CSS triangle keeps it wholly in the skin with
  // no icons.ts churn — noted for the icon-adoption backlog).
  import { envelope } from '$common/components/base/envelope';
  import SbCell from './sb-cell.svelte';
  import StatusIndicator from './status-indicator.svelte';
  import Separator from './separator.svelte';
  import Label from './label.svelte';

  let {
    // Forwarded to the left status-indicator (the connection light + caption).
    states = {},
    state,
    pulse = false,
    caption = '',
    linkHref,
    linkText,
    linkExternal = false,
    onLinkClick,
    // Optional 2nd left cell — a secondary label (e.g. a version tag) preceded by a vertical
    // separator. Unset → the left cell holds only the status-indicator (leftCount 1).
    secondaryText,
    // The SE resize-grip (right cell). `grip=false` drops it (a non-resizable host).
    grip = true,
    onResizeGrip,
    id,
  }: {
    states?: Record<string, string>;
    state?: string;
    pulse?: boolean;
    caption?: string;
    linkHref?: string;
    linkText?: string;
    linkExternal?: boolean;
    onLinkClick?: (e: MouseEvent) => void;
    secondaryText?: string;
    grip?: boolean;
    onResizeGrip?: (e: PointerEvent) => void;
    id?: string;
  } = $props();

  // Composite-derived stable child ids (the status-indicator precedent).
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // N-024 opt-in. Aggregate of what the COMPOSITE owns: item counts per side + grip presence.
  // `leftCount` counts hosted display components (status-indicator + optional secondary label;
  // the divider is not content). `rightCount` counts right-cell items (the grip). Colours /
  // captions live on the child entries, not duplicated here.
  const debug = () => ({
    leftCount: secondaryText ? 2 : 1,
    rightCount: grip ? 1 : 0,
    hasGrip: grip,
  });
</script>

<div use:envelope={{ name: 'status-bar', id, debug }}>
  <SbCell side="left">
    <StatusIndicator
      {states}
      {state}
      {pulse}
      {caption}
      {linkHref}
      {linkText}
      {linkExternal}
      {onLinkClick}
      id={cid('status-indicator')}
    />
    {#if secondaryText}
      <Separator orientation="vertical" id={cid('sep')} />
      <Label text={secondaryText} id={cid('secondary')} />
    {/if}
  </SbCell>
  <SbCell side="right">
    {#if grip}
      <span
        class="sb-grip"
        aria-hidden="true"
        onpointerdown={(e) => onResizeGrip?.(e)}
      ></span>
    {/if}
  </SbCell>
</div>
