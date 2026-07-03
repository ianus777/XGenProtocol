<script lang="ts">
  // star-rating — data-independent, COMPOSITE (M-RP2.24): the 19th `core` component and the
  // THIRD di composite (after status-indicator N-054, password-field N-060). SHAPE B (Joe-lock
  // 2026-07-03): SELF-CONTAINED — the root IS `<div class="star-rating">` (the composite
  // root-marker via `envelope`, N-020/N-022), but it renders its stars INTERNALLY in an `{#each}`
  // rather than composing built atomic components. This REFINES the composite definition: a
  // di-composite is a `<div class="type">` assembly; composing child atomics (status/password) is
  // the common case, not a requirement. The matrix multiplies FLAT +1 (one aggregate getter, no
  // self-registering children — unlike the child-composite precedents). → D-069 promotion-watch
  // (definition refinement; note only unless it recurs).
  //
  // di + PASSIVE: the caller supplies `max`/`value`; the component interprets no domain structure
  // (di). Hover-preview is transient presentational `$state` (the button `:active` precedent), not
  // load/save/validate/host-I/O → clears the widget bar (N-059). Stays a passive composite.
  //
  // VALUE (Lock 1): `value: number`, `$bindable`, default 0 (= unrated), numeric bind-out; `max`
  // default 5; getter `{ value, max }`.
  // A11Y (Lock 2): root `role="radiogroup"`; each star `role="radio"` + `aria-checked`; roving
  // `tabindex` (the checked star, or star 1 when unrated, is the tab stop); arrows move+select
  // (selection-follows-focus radiogroup model), Home=1 / End=max. readonly/disabled drop interaction.
  // HOVER + CLEARABLE (Lock 3): `hovered` transient preview restores on `mouseleave`; `clearable`
  // (default true) → clicking the active star zeroes it.
  // GLYPH (all-next): ★ via a currentColor `mask` placeholder (the password-field eye pattern,
  // N-052; the `--star` var is skin-scoped, SVG placeholder until the `icon` primitive). Filled =
  // `--accent2` (re-themes gold/blue per shell), empty = `--t4`. Whole-star only v1; a half-star
  // average is a future readonly shape.
  import { envelope } from '$common/components/base/envelope';

  let {
    value = $bindable(0),
    max = 5,
    readonly = false,
    disabled = false,
    clearable = true,
    id,
    name,
    ariaLabel,
  }: {
    value?: number;
    max?: number;
    readonly?: boolean;
    disabled?: boolean;
    clearable?: boolean;
    id?: string;
    name?: string;
    ariaLabel?: string;
  } = $props();

  let hovered = $state(0);                          // transient preview; 0 = none
  let stars = $state<HTMLElement[]>([]);            // refs for roving-focus after keyboard move
  const active = $derived(disabled || readonly);    // interaction suppressed
  const shown = $derived(hovered || value);         // fill target (preview wins while hovering)

  function set(i: number) {
    if (active) return;
    value = clearable && i === value ? 0 : i;       // click active star → clear
  }
  function preview(i: number) {
    if (!active) hovered = i;
  }
  function clearPreview() {
    hovered = 0;
  }
  function onKey(e: KeyboardEvent) {
    if (active) return;
    let next = value;
    switch (e.key) {
      case 'ArrowRight':
      case 'ArrowUp':   next = Math.min(max, (value || 0) + 1); break;
      case 'ArrowLeft':
      case 'ArrowDown': next = Math.max(1, (value || 1) - 1); break;
      case 'Home':      next = 1; break;
      case 'End':       next = max; break;
      default: return;
    }
    e.preventDefault();
    value = next;
    stars[next - 1]?.focus();                       // roving focus follows selection
  }

  // Roving tab stop: the checked star, or star 1 when unrated. -1 for the rest.
  const tabFor = (i: number) => (active ? undefined : (value === i || (value === 0 && i === 1) ? 0 : -1));

  // N-024 opt-in. $state.snapshot de-proxies for CDP returnByValue. Reports only what the
  // composite owns (value + max); no children to aggregate (Shape B).
  const debug = () => $state.snapshot({ value, max });
</script>

<div
  use:envelope={{ name: 'star-rating', id, debug }}
  role="radiogroup"
  aria-label={ariaLabel || undefined}
  aria-disabled={disabled || undefined}
  data-readonly={readonly || undefined}
  onmouseleave={clearPreview}
  onkeydown={onKey}
>
  {#each Array(max) as _, idx (idx)}
    {@const i = idx + 1}
    <span
      bind:this={stars[idx]}
      class="star"
      role="radio"
      aria-checked={i === value}
      aria-label={`${i} star${i === 1 ? '' : 's'}`}
      data-filled={i <= shown || undefined}
      tabindex={tabFor(i)}
      onclick={() => set(i)}
      onmouseenter={() => preview(i)}
    ></span>
  {/each}
  {#if name}<input type="hidden" {name} value={value} />{/if}
</div>
