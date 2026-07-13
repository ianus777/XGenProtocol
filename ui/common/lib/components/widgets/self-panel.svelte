<script lang="ts">
  // self-panel — R3 Self/connection: the FIRST REAL SYSTEM WIDGET (M-RP6.1g, D4). `kind: system` (W-13,
  // non-removable), it replaces the `self` placeholder in the layout registry — ONE registry entry swapped,
  // no rewrite (exactly what renderer A's prop-injected registry was built for, N-093). Level-2 marked via
  // `data-tier="widget"` on the root (the `substitutions-editor` shape).
  //
  // Composes SHIPPED `core` UNMODIFIED: an `entity-item` (card) for the identity + a `status-indicator` for
  // the connection light — both fed from the ONE `$common` self-state store (D3), NEVER from shell props
  // (the leaf mount gives only `regionId`; a common widget can't import a shell dep, W-3).
  //
  // FIRST WRITER of the selection bus (D5): activating the identity `entity-item` sets `selection`. The
  // bus's "no writer yet" note is retired this milestone.
  //
  // HONEST PHASE-LIMITS (D6 / W-8, nothing faked to pass a leg):
  //   ① unregistered → the entity-item shows the REAL keypair-derived XGID with a "not registered"
  //      secondary line (entity-item's name-falls-back-to-id contract does the rest) — NO fake name;
  //   ② Track-A `status` ships ABSENT (the `status` prop is simply not passed) — no client status read
  //      path exists yet, so an empty slot is the truth, not a faked value.
  import { envelope } from '$common/components/base/envelope';
  import { selfState, STATE_COLOURS, PULSING_STATES } from '$common/stores/self-state.svelte';
  import { selection } from '$common/stores/selection.svelte';
  import type { EntityDescriptor } from '$core/components/data-dependent/types';
  import EntityItem from '$core/components/data-dependent/entity-item.svelte';
  import StatusIndicator from '$core/components/data-independent/status-indicator.svelte';

  // The leaf mount (`region-node`) passes a widget ONLY `regionId` — NOT `{regionId, id}` as an earlier
  // runbook draft assumed (grounded against region-node.svelte:36). Derive the envelope id from it using the
  // EXACT `region-placeholder` convention `id = "region-" + regionId` (Joe amendment): so the registry delta
  // reads as a clean swap in place — `section#region-self` OUT, `self-panel#region-self` IN, children
  // `region-self__section` / `__item` (+ `__item__avatar`) / `__status` (+ `__status__led`/`__status__label`).
  // One convention, eight regions.
  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  const conn = $derived(selfState.connection);
  const identity = $derived(selfState.identity);

  // The identity descriptor (D6 ①). Real XGID when the keypair read succeeded, `'—'` when even that is
  // absent; `name` absent → the entity-item falls back to the id (its shipped contract) — NO fake name.
  const descriptor = $derived<EntityDescriptor>({
    kind: 'identity',
    id: identity?.identity_id ?? '—',
    name: identity?.display_name ?? undefined,
  });

  // Secondary line: the home node when registered, else the honest "not registered" (D6 ①).
  const secondary = $derived(
    identity?.registered ? (identity.home_node ?? undefined) : 'not registered',
  );

  // Selection = read the bus BACK (D5, Joe amendment). self-panel is the bus's writer AND a reader:
  // without this the entity-item's `selected` prop is a constant false — an unfed branch that strands the
  // shipped `.entity-item[data-selected]` skin affordance. Matched on ENTITY id (the item's own identity),
  // so it is render-truth for "is THIS entity the current selection".
  const selected = $derived(selection.current?.entity.id === descriptor.id);

  // Aggregate getter G (W-4 — observable task-state, never payload/secrets). The display name and XGID are
  // already CDP-readable off the composed `entity-item#…__item` getter, so republishing them here would be
  // duplication AND a payload leak (the N-060 `hasValue` precedent) — G reports only what it itself owns.
  const debug = () => ({
    registered: !!identity?.registered,
    hasIdentity: !!identity?.identity_id,
    connectionState: conn.state,
    selected,
  });
</script>

<!-- M-RP7.1 (D3): the <Section title="Self"> wrapper is UNWRAPPED — the tile frame draws the title now,
  so a Section here would be a second title. The rows render directly; `section#region-self__section`
  leaves the registry, the item/status children keep their own ids. -->
<div class="self-panel" data-tier="widget" use:envelope={{ name: 'self-panel', id, debug }}>
  <EntityItem
    {descriptor}
    variant="card"
    {secondary}
    {selected}
    onActivate={() => selection.set(regionId, descriptor)}
    id={cid('item')}
  />
  <StatusIndicator
    states={STATE_COLOURS}
    state={conn.state}
    pulse={PULSING_STATES.includes(conn.state)}
    caption={conn.label}
    id={cid('status')}
  />
</div>
