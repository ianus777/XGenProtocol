<script lang="ts">
  // entity-item — the FIRST data-dependent COMPOSITE (M-RP5.1, dd-composite). It materializes ONE
  // address-book entry (identity ∪ space ∪ DM) as a full DISPLAY UNIT — avatar + name + secondary
  // line + trailing meta/status — one tier up from `entity-avatar` (the dd-atomic glyph, M-RP5.0).
  //
  // ONE composite, PURPOSE selected by `variant` (the "purpose → variant, presentation derived"
  // discipline, applied one tier up from the avatar): `row` (list entry) · `card` (prominent) ·
  // `nav` (sidebar entry) · `inline` (mention/presence). A genuinely new entity-display need becomes
  // a NEW variant, not a new component (D-069 recurrence bar). The slot surface is DERIVED per
  // variant (not free props): `row` = name + meta · `card` = name + secondary + status · `nav` =
  // name · `inline` = name.
  //
  // SINGLE-KNOB rule (lock): the consumer sets ONE `entity-item` `variant`; the composite DERIVES
  // the inner `entity-avatar` variant internally (AVATAR_VARIANT below). The two variant axes never
  // fight — the caller never touches the avatar's variant.
  //
  // Source-agnostic: consumes the same `EntityDescriptor` seam as the avatar (the W-11 dd-socket);
  // `core` imports NO `IdentityRecord`/`SpaceState`. The secondary line, status, and meta are
  // CALLER-SUPPLIED slots (the shell maps protocol → these strings; the shell maps Track A
  // `state.status` → the `status` slot). The composite owns LAYOUT, not protocol reads.
  //
  // COMPOSITE-REGISTRATION (the status-indicator precedent): the composite root registers ONE
  // aggregate getter (F); the real `entity-avatar` child self-registers under the composite-derived
  // stable id `<id>__avatar` (zero changes to the atomic beyond its pre-announced variant widening).
  // A composite cell therefore yields MULTIPLE registry entries and the matrix multiplies.
  import { envelope } from '$common/components/base/envelope';
  import EntityAvatar from './entity-avatar.svelte';
  import type { EntityDescriptor } from './types';

  let {
    descriptor,
    variant = 'row',
    secondary,
    status,
    meta,
    selected = false,
    onActivate,
    width,
    id,
  }: {
    descriptor: EntityDescriptor;
    variant?: 'row' | 'card' | 'nav' | 'inline';
    secondary?: string; // topic / last-message / handle — caller string, source-agnostic
    status?: { emoji?: string; text?: string }; // Track A self-status slot (shell-mapped)
    meta?: string; // unread count / timestamp — caller string
    selected?: boolean;
    onActivate?: () => void;
    width?: string;
    id?: string;
  } = $props();

  // B — single-knob derive-map: entity-item.variant → entity-avatar.variant. LITERAL to the lock.
  const AVATAR_VARIANT = {
    row: 'list',
    card: 'card',
    nav: 'labeled',
    inline: 'presence',
  } as const;

  const kind = $derived(descriptor.kind);
  const avatarVariant = $derived(AVATAR_VARIANT[variant]);

  // Name TEXT for the composite's own column (the avatar owns its initials fallback separately).
  // Absent name → the id, honestly (matches the avatar getter's `name: name ?? null` convention).
  const displayName = $derived(descriptor.name ?? descriptor.id);

  // C — slot surface, DERIVED per variant (not free): render-truth, so the getter reports what is
  // actually shown, not merely what was passed.
  const showSecondary = $derived(variant === 'card' && !!secondary);
  const showStatus = $derived(
    variant === 'card' && !!(status && (status.emoji || status.text)),
  );
  const showMeta = $derived(variant === 'row' && !!meta);

  // Composite-derived stable child id (so the self-registering avatar reads cleanly, not ordinal).
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // F — aggregate getter. `hasSecondary`/`hasStatus` are the RENDER truth (slot-applicable AND
  // value present), so CDP reads them straight off the composite.
  const debug = () => ({
    variant,
    kind,
    name: descriptor.name ?? null,
    hasSecondary: showSecondary,
    hasStatus: showStatus,
    selected: !!selected,
  });

  function activate() {
    onActivate?.();
  }
  function onKeydown(e: KeyboardEvent) {
    // The item exposes activate-on-Enter/Space; the LIST/PANEL (M-RP5.2) owns roving focus +
    // tabindex/role — this item sets neither (D lock). Dead until the panel makes it focusable.
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      activate();
    }
  }
</script>

<!-- G — dd-composite root = `<div class="entity-item">` (N-075 dd-root, panel-disambiguated). The
  real `entity-avatar` child is derived to its inner variant (single-knob) and self-registers. -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  use:envelope={{ name: 'entity-item', id, debug }}
  data-variant={variant}
  data-kind={kind}
  data-selected={selected || undefined}
  style={width ? `width: ${width}` : undefined}
  onclick={activate}
  onkeydown={onKeydown}
>
  <EntityAvatar {descriptor} variant={avatarVariant} id={cid('avatar')} />
  <div class="ei-body">
    <span class="ei-name">{displayName}</span>
    {#if showSecondary}
      <span class="ei-secondary">{secondary}</span>
    {/if}
  </div>
  {#if showStatus}
    <span class="ei-status">
      {#if status?.emoji}<span class="ei-status-emoji">{status.emoji}</span>{/if}
      {#if status?.text}<span class="ei-status-text">{status.text}</span>{/if}
    </span>
  {/if}
  {#if showMeta}
    <span class="ei-meta">{meta}</span>
  {/if}
</div>
