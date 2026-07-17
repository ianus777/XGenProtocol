<script lang="ts">
  // rooms-panel — R2 Rooms (M-RP6.2, D8). A `kind: system` region widget; the swap is DERIVED via a
  // `CLIENT_PLUGINS` descriptor (`surface: 'region'`, `regionId: 'rooms'`), exactly like spaces-panel. It
  // both READS the bus (to scope which Space's rooms to list) and WRITES it (activating a room = the bus's
  // THIRD writer). No new store, no new invoke — the rooms ride EMBEDDED in `spacesState` (D1).
  //
  // THE ONE NEW MECHANIC (D3, the latch): R2 cannot read its data scope from `selection.current`, because
  // R2's OWN room-activation moves the bus to a `kind: 'room'`. So it LATCHES the last SPACE selection and
  // KEEPS it while the bus holds a room. Without the latch, clicking a room would blank R2's own list.
  import { envelope } from '$common/components/base/envelope';
  import { spacesState, type KnownRoom } from '$common/stores/spaces-state.svelte';
  import { selection } from '$common/stores/selection.svelte';
  import type { EntityDescriptor } from '$core/components/data-dependent/types';
  import EntityPanel from '$core/components/data-dependent/entity-panel.svelte';

  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // The last SPACE selection = R2's data scope. Written ONLY when the bus holds a space; KEPT when R2's own
  // room-activation moves the bus to a room (D3). The effect reads `selection.current` and writes
  // `latchedSpaceId` — it never READS `latchedSpaceId`, so there is no self-invalidating read-modify-write
  // (the N-136 trap avoided).
  let latchedSpaceId = $state<string | null>(null);
  $effect(() => {
    const c = selection.current;
    if (c?.entity.kind === 'space') latchedSpaceId = c.entity.id;
  });

  // The scoped Space — null when nothing is latched OR the latched Space is gone between hydrations
  // (stale-latch guard, N-095 spirit: never throw; fall back to the "Select a space" empty state).
  const scopedSpace = $derived(
    spacesState.spaces.find((s) => s.space_id === latchedSpaceId) ?? null,
  );
  const rooms = $derived(scopedSpace?.rooms ?? []);

  function toDescriptor(r: KnownRoom): EntityDescriptor {
    return { kind: 'room', id: r.room_id, name: r.name };
  }
  // v1: descriptor only; `secondary`/`meta` UNFED (D6/D-065). `kind: 'room'` -> entity-avatar hexagon (J-501).
  const items = $derived(rooms.map((r) => ({ descriptor: toDescriptor(r) })));

  // Read the bus BACK (D5), room facet.
  const selected = $derived(
    selection.current?.entity.kind === 'room' ? selection.current.entity.id : undefined,
  );

  function onActivate(roomId: string): void {
    const r = rooms.find((x) => x.room_id === roomId);
    if (r) selection.set(regionId, toDescriptor(r));
  }

  // Two honest empty states (N-091): no space scope -> "Select a space"; a scoped space with zero rooms ->
  // "No rooms". Distinct copy — different truths. entity-panel renders one `emptyText`; pick it by scope.
  const emptyText = $derived(scopedSpace === null ? 'Select a space' : 'No rooms');

  const debug = () => ({
    count: rooms.length,
    latchedSpaceId,
    selectedId: selected ?? null,
    hasEmpty: rooms.length === 0,
  });
</script>

<div class="rooms-panel" data-tier="widget" use:envelope={{ name: 'rooms-panel', id, debug }}>
  <EntityPanel {items} {selected} {onActivate} {emptyText} id={cid('panel')} />
</div>
