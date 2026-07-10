<script lang="ts">
  // separator — data-independent, DISPLAY-kind (N-032): a pure visual divider. The 29th
  // `core` component and the SECOND frame prerequisite of the M-RP6.1 client-UI-frame arc
  // (D-107). A display-di sibling of label/paragraph/image/led/icon — but the LEANEST in
  // the library and the FIRST VALUE-LESS component: it carries no value at all (not even a
  // read-only one), so its getter is CONFIG-ONLY. Atomic (N-020): the root IS a <div>.
  //
  // WHY <div role="separator"> AND NOT <hr> (locked, Joe "go"): chosen deliberately so the
  // SAME component is valid in every context with no branch — a flex status-bar cell divider
  // AND a direct child of a future <ul role="menu"> (an <hr> is NOT a valid <ul> child; a
  // role="separator" div is). One root, every context, built once (frame Phase-0 §4.4).
  // D-096 fold cleared at Phase-0 — no new decision.
  //
  // API: `orientation` (horizontal | vertical, default horizontal — canonical) reflects to
  // BOTH `data-orientation` (the skin hook) and `aria-orientation` (a11y). `variant`
  // (line | double | gap, default line) reflects to `data-variant` (the skin hook). NO
  // value / binding / label / interaction / inline-style / tint / thickness props — every
  // visual (thickness, style, colour) is skin-owned, keyed by `.separator` in the one skin
  // file (N-025). Spacing AROUND the divider is the consumer's concern (status-bar cell
  // layout / menu), not the atom's.
  //
  // APPEARANCE = skin.css only, BORDER-based: a <div> draws its rule via a border (not a
  // background) because `border-style: double` gives the two-line rule natively — a
  // background cannot express `double`. horizontal → border-top; vertical → border-left;
  // `gap` → border:0 (pure spacing, the box still present). The colour is the shared --s5
  // hairline token (the same 1px border colour every bordered control uses), so the
  // separator is accent-NEUTRAL chrome (the led/meter no-accent precedent).
  //
  // The type-class is supplied by `envelope` (N-023), so no `class` is hardcoded. No local
  // CSS: empty <style>, all appearance is skin (N-025 / N-021 layer 2).
  import { envelope } from '$common/components/base/envelope';

  let {
    orientation = 'horizontal',
    variant = 'line',
    id,
  }: {
    orientation?: 'horizontal' | 'vertical';
    variant?: 'line' | 'double' | 'gap';
    id?: string;
  } = $props();

  // N-024 opt-in. Value-LESS: the getter reports config only (the first such getter); the
  // registry stays uniform (N-030 §4). Plain-return (the icon/led precedent) — orientation
  // and variant are plain strings, no proxy to de-reference.
  const debug = () => ({ orientation, variant });
</script>

<div
  use:envelope={{ name: 'separator', id, debug }}
  role="separator"
  aria-orientation={orientation}
  data-orientation={orientation}
  data-variant={variant}
></div>

<style></style>
