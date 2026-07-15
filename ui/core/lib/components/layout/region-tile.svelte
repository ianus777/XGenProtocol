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
  // FOLD — the USER'S CHOICE, TWO BUTTONS (M-RP7.1b, §4.1). `collapsed` names the DIMENSION collapsed
  // ('width' → vertical strip / 'height' → horizontal stripe), NOT where the strip goes (it parks at the
  // leading edge). The user picks it, so it is STORED — it is a fact about what they asked for, which cannot
  // go stale, unlike M-RP7.1's derived axis (a fact about the tree, which could).
  //
  // But the tile must still know whether that fold runs ALONG or ACROSS its parent split's axis, because
  // the two lay out completely differently (§4.1). A `col` split divides height, so folding HEIGHT is ALONG
  // it (siblings absorb, NO hole); folding WIDTH is ACROSS it (nobody absorbs → a hole, §4.5). `foldMode` is
  // DERIVED at render from `collapsed` + the parent's `axis` — the direction is stored, the mode is not.
  //   - along  → drop the tile's main-axis flex share → the skin pins it to stripe size, siblings absorb.
  //   - across → KEEP the main-axis flex share (inline) → only the cross dimension collapses; the freed
  //              cross-space becomes a hole the split paints (§4.5).
  //
  // Two `<button>`s (`data-fold="width" / "height"`), ALWAYS PRESENT so the rotated strip's content is
  // identical to the unfolded stripe's (§4.3). When folded, the OTHER-axis button is `aria-disabled`
  // (the shelf-face/menu-item pattern — NOT native `disabled`, so it stays keyboard-reachable; and NOT a
  // countdown, N-109 — it flips back the moment the tile unfolds); the MATCHING button unfolds. Appearance
  // (the two chevron orientations, the rotated strip, CW/CCW via --region-fold-rotate) is ALL skin (N-090).
  //
  // A folded tile shows NO resize triangle (D8.2): a folded tile has no resizable dimension of its own
  // (its collapse axis is pinned at stripe size by definition, its cross-axis belongs to the parent), so
  // the grip is ELEMENT-ABSENT — not greyed (J-500: the absent slot ships absent, not faked).
  import { envelope } from '$common/components/base/envelope';
  import type { Snippet } from 'svelte';
  import type { FoldAxis } from './types';
  import { isFoldAlong } from './resolve'; // one source for the along-fold predicate (M-RP7.2, L4)

  let {
    regionId,
    title,
    collapsed,
    axis = 'col',
    flex,
    onFold,
    onMoveStart,
    locked = false,
    dragging = false,
    children,
  }: {
    /** The leaf widgetId — the tile's durable registry handle is `region-${regionId}` (N-096, D4). */
    regionId: string;
    /** Display title from the registry (D2); the widget no longer draws its own. */
    title: string;
    /** Fold state — render truth (D10). Absent ⇒ expanded; a `FoldAxis` ⇒ the dimension the user folded. */
    collapsed?: FoldAxis;
    /** The PARENT split's `dir` (§4.1). Drives `foldMode` (along/across), never the direction itself. */
    axis?: 'row' | 'col';
    /** Descriptor weight from the parent split (the N-090 DATA carve-out — inline, not skin). */
    flex?: number;
    /** Fold toggle seam — the shell mutates the descriptor (D6). `undefined` ⇒ unfold. */
    onFold?: (regionId: string, collapsed: FoldAxis | undefined) => void;
    /** Move-grip pointerdown seam (M-RP7.4). The grip only STARTS the gesture; the grid-level overlay in
     *  `region-shell` (D1) owns capture, the drag loop, band hit-testing and the release → `move`. */
    onMoveStart?: (regionId: string, e: PointerEvent) => void;
    /** Grid lock (M-RP7.6): when true, the move grip and the fold buttons go ELEMENT-ABSENT (the house
     *  convention — the folded tile's resize triangle ships absent, not greyed, J-500/D8.2). Keeps the
     *  arrangement frozen while the widget stays interactive; the shell handler is the real guard. */
    locked?: boolean;
    /** True while THIS tile is the one being dragged (M-RP7.4) — the shell threads its own drag `sourceId`
     *  down; the skin dims the source (PROVISIONAL). Reflected as `data-dragging`. */
    dragging?: boolean;
    /** The widget body (rendered only while expanded). */
    children?: Snippet;
  } = $props();

  // foldMode — DERIVED, never stored (§4.1). 'along' the parent's dividing axis ⇒ siblings absorb, no hole;
  // 'across' ⇒ nobody absorbs ⇒ a hole. (A `col` split divides height, so folding height is ALONG it.)
  const foldMode = $derived<'none' | 'along' | 'across'>(
    collapsed === undefined ? 'none' : isFoldAlong(collapsed, axis) ? 'along' : 'across',
  );

  // The inline flex weight (the ONE N-090 exception: `sizes[]` is DATA). OMITTED for an ALONG fold so the
  // skin's `[data-fold-mode="along"] { flex: 0 0 auto }` pins the tile to stripe size and siblings absorb.
  // KEPT for 'none' AND for an ACROSS fold — an across-folded tile still owns its share of the parent's
  // MAIN axis; only its CROSS axis collapses (the skin does that via align-self + a cross-size pin, §4.1).
  const flexStyle = $derived(foldMode !== 'along' && flex !== undefined ? `flex: ${flex} 1 0` : undefined);

  // Per-button state (§4.3): expanded ⇒ each button folds its own axis; folded ⇒ the matching axis unfolds,
  // the other is disabled. Derived so a re-fold repaints correctly.
  const widthState = $derived<'fold' | 'unfold' | 'disabled'>(
    collapsed === undefined ? 'fold' : collapsed === 'width' ? 'unfold' : 'disabled',
  );
  const heightState = $derived<'fold' | 'unfold' | 'disabled'>(
    collapsed === undefined ? 'fold' : collapsed === 'height' ? 'unfold' : 'disabled',
  );

  function clickFold(dir: FoldAxis, state: 'fold' | 'unfold' | 'disabled'): void {
    if (state === 'disabled') return;
    onFold?.(regionId, state === 'fold' ? dir : undefined); // fold this axis, or unfold
  }

  // Getter G (D10) — `collapsed`/`axis`/`foldMode` are RENDER TRUTH (what actually painted / reflected),
  // which is what makes a fold CDP-provable (the `message.detailsCount` precedent).
  const debug = () => ({ regionId, title, collapsed, axis, foldMode });
</script>

<!-- No literal `class="region-tile"`: `use:envelope` supplies the type-class from `name` (N-023), and
  mergeClasses does not dedupe — a literal would concatenate to `region-tile region-tile`. -->
<div
  data-axis={axis}
  data-collapsed={collapsed || undefined}
  data-fold-mode={foldMode !== 'none' ? foldMode : undefined}
  data-region-id={regionId}
  data-dragging={dragging || undefined}
  style={flexStyle}
  use:envelope={{ name: 'region-tile', id: `region-${regionId}`, debug }}
>
  <div class="region-tile-stripe">
    <!-- Move grip (Joe's walk: "only with this grip the region can be moved") — ACTIVATED at M-RP7.4.
      pointerdown STARTS the gesture; the grid-level overlay (region-shell, D1) owns the rest. Focusable so
      it is not a dead control, but keyboard-driven move (pickup → target → drop) is its own protocol,
      FILED as M-RP-MOVE-KBD (D5) — Enter/Space are deliberately not wired yet.
      M-RP7.6: ELEMENT-ABSENT while locked (D1) — no live-looking dead grip; the shell handler is the guard. -->
    {#if !locked}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span
        class="region-tile-move"
        role="button"
        tabindex="0"
        aria-label="Move region (drag)"
        onpointerdown={(e) => onMoveStart?.(regionId, e)}
      ></span>
    {/if}
    <span class="region-tile-title">{title}</span>
    <!-- Two fold buttons — the user picks the axis (§4.1). ALWAYS PRESENT (§4.3); DOM order is fixed
      [move · title · [fold-width · fold-height]], which now matters (it survives the rotated strip). The
      matching-axis button unfolds; the other-axis button is aria-disabled (NOT native disabled →
      keyboard-reachable) while folded. Glyph + rotation are skin (N-090). Both buttons are wrapped in one
      addressable `.region-title-buttons` span so the skin can target/position them as a group. -->
    <!-- M-RP7.6: the whole fold group is ELEMENT-ABSENT while locked (D1) — the arrangement is frozen, so no
      fold affordance is offered; the shell's `handleFold` early-return is the load-bearing guard. -->
    {#if !locked}
      <span class="region-title-buttons">
        <button
          type="button"
          class="region-tile-fold"
          data-fold="width"
          aria-label={widthState === 'unfold' ? 'Unfold region' : 'Fold region to the left'}
          aria-expanded={collapsed === undefined}
          aria-disabled={widthState === 'disabled' || undefined}
          onclick={() => clickFold('width', widthState)}
        ></button>
        <button
          type="button"
          class="region-tile-fold"
          data-fold="height"
          aria-label={heightState === 'unfold' ? 'Unfold region' : 'Fold region to the top'}
          aria-expanded={collapsed === undefined}
          aria-disabled={heightState === 'disabled' || undefined}
          onclick={() => clickFold('height', heightState)}
        ></button>
      </span>
    {/if}
  </div>

  {#if collapsed === undefined}
    <div class="region-tile-body">
      {@render children?.()}
    </div>
    <!-- SE resize grip — the reused status-bar triangle (D7). PAINTED + DEAD; resize lands at M-RP7.2.
      Element-ABSENT when folded (D8.2), not greyed. -->
    <span class="region-tile-resize" aria-hidden="true"></span>
  {/if}
</div>
