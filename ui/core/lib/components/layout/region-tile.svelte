<script lang="ts">
  // region-tile — the tile frame (M-RP7.1, D1). The chrome moves from the widget into the renderer:
  // `region-node` now mounts THIS around a leaf's widget instead of a bare `.region-leaf` div. It owns
  //   - a title STRIPE: [move-grip · title · fold button]
  //   - the BODY slot (the widget renders inside it; the body is the scroller — D5, ex-`.region-leaf`)
  //   - an SE corner RESIZE grip — the shipped `status-bar` clip-path triangle, reused (D7)
  //
  // `core` tier (D1): the NODE app inherits the grid at M-RP7.6, exactly the `region-shell`/`menu-bar`/
  // `status-bar` precedent. NO Tauri, NO protocol import. Zero component-local <style> (D9/N-090): every
  // pixel — stripe height, grip size, triangle, the folded/rotated form, hover states — lives in skin.css
  // keyed off `.region-tile*`, so Joe retunes the whole look without a component edit.
  //
  // THIS LEG IS FRAME-ONLY. Nothing drags, nothing resizes — the fold button's click is the ONLY live
  // gesture (D7). The move grip and the resize triangle are PAINTED AND DEAD (aria-hidden, no handler);
  // their dischargers are named (move → M-RP7.4, resize → M-RP7.2). They carry no claim (no role, no
  // tabindex, no hover cursor), so they say nothing to the user that later becomes false.
  //
  // FOLD, along the parent split's AXIS (D8):
  //   - parent `col` (divides height) → collapse HEIGHT → the tile shrinks to the horizontal stripe.
  //   - parent `row` (divides width)  → collapse WIDTH  → the stripe rotates to a vertical side strip.
  // `collapsed` is ONE boolean; the axis is DERIVED from the parent's `dir` (passed as `axis`), NEVER
  // stored — a stored direction would go stale the instant the tile is dragged (D-067 in miniature).
  // Both are reflected as `data-collapsed` / `data-axis` and the SKIN does the rotation (N-090); the
  // component branches on the axis ONLY to reflect the attribute. The folded strip keeps the SAME content
  // and DOM order (grip · title · fold) — the skin's one `writing-mode` property picks CW (default) vs CCW.
  //
  // A folded tile shows NO resize triangle (D8.2): a folded tile has no resizable dimension of its own
  // (its collapse axis is pinned at stripe size by definition, its cross-axis belongs to the parent), so
  // the grip is ELEMENT-ABSENT — not greyed (J-500: the absent slot ships absent, not faked).
  import { envelope } from '$common/components/base/envelope';
  import type { Snippet } from 'svelte';

  let {
    regionId,
    title,
    collapsed = false,
    axis = 'col',
    flex,
    onFold,
    children,
  }: {
    /** The leaf widgetId — the tile's durable registry handle is `region-${regionId}` (N-096, D4). */
    regionId: string;
    /** Display title from the registry (D2); the widget no longer draws its own. */
    title: string;
    /** Fold state — render truth (D10). Absent/false ⇒ expanded. */
    collapsed?: boolean;
    /** The PARENT split's `dir` (D8). Derived, never stored; drives the collapse axis via the skin. */
    axis?: 'row' | 'col';
    /** Descriptor weight from the parent split (the N-090 DATA carve-out — inline, not skin). */
    flex?: number;
    /** Fold toggle seam — the shell mutates the descriptor (D6). */
    onFold?: (regionId: string, collapsed: boolean) => void;
    /** The widget body (rendered only while expanded). */
    children?: Snippet;
  } = $props();

  // The inline flex weight (the ONE N-090 exception: `sizes[]` is DATA). It is OMITTED while collapsed so
  // the skin can pin the tile to stripe size — an inline `flex: {n} 1 0` would out-specify any skin rule,
  // so a folded tile could never stop growing along its collapse axis (§4.2). Omitting it lets the skin's
  // `.region-tile[data-collapsed] { flex: 0 0 auto }` take over; the tile still fills its parent's
  // cross-axis via the split's default `align-items: stretch`, so no hole is ever left.
  const flexStyle = $derived(!collapsed && flex !== undefined ? `flex: ${flex} 1 0` : undefined);

  // Getter G (D10) — `collapsed`/`axis` are RENDER TRUTH (what actually painted / reflected), which is
  // what makes a fold CDP-provable (the `message.detailsCount` precedent).
  const debug = () => ({ regionId, title, collapsed, axis });
</script>

<!-- No literal `class="region-tile"`: `use:envelope` supplies the type-class from `name` (N-023), and
  mergeClasses does not dedupe — a literal would concatenate to `region-tile region-tile`. -->
<div
  data-axis={axis}
  data-collapsed={collapsed || undefined}
  style={flexStyle}
  use:envelope={{ name: 'region-tile', id: `region-${regionId}`, debug }}
>
  <div class="region-tile-stripe">
    <!-- Move grip (Joe's walk: "only with this grip the region can be moved"). PAINTED + DEAD this leg;
      the drag lands at M-RP7.4 (D7). -->
    <span class="region-tile-move" aria-hidden="true"></span>
    <span class="region-tile-title">{title}</span>
    <!-- The ONLY live control. Toggles the descriptor's `collapsed` via the shell (D6). -->
    <button
      type="button"
      class="region-tile-fold"
      aria-label={collapsed ? 'Unfold region' : 'Fold region'}
      aria-expanded={!collapsed}
      onclick={() => onFold?.(regionId, !collapsed)}
    ></button>
  </div>

  {#if !collapsed}
    <div class="region-tile-body">
      {@render children?.()}
    </div>
    <!-- SE resize grip — the reused status-bar triangle (D7). PAINTED + DEAD; resize lands at M-RP7.2.
      Element-ABSENT when folded (D8.2), not greyed. -->
    <span class="region-tile-resize" aria-hidden="true"></span>
  {/if}
</div>
