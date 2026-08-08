<script lang="ts">
  // rooms-panel — R2 Rooms (M-RP6.2, D8; M-RP-SELECT-ORIENT C-1/C-3). A `kind: system` region widget; the
  // swap is DERIVED via a `CLIENT_PLUGINS` descriptor (`surface: 'region'`, `regionId: 'rooms'`), exactly like
  // spaces-panel. No new invoke — the rooms ride EMBEDDED in `spacesState` (D1).
  //
  // It WRITES the bus on room activation (the bus's THIRD writer) but no longer READS it. Both the data scope
  // and the highlight come from TWO app-lifetime latches the SHELL drives (one effect, two latches — C-1):
  // `spaceLatch` scopes which Space's rooms to list, and `roomLatch.effectiveRoomId` lights the current room
  // so a later identity/member selection KEEPS it lit (D-146) rather than blanking it. Reading a latch instead
  // of `selection.current` is why R2's own room-activation (moving the bus to `kind: 'room'`) does not blank
  // R2's own list.
  import { envelope } from '$common/components/base/envelope';
  import { type KnownRoom } from '$common/stores/spaces-state.svelte';
  import { selection } from '$common/stores/selection.svelte';
  import { spaceLatch } from '$common/stores/space-latch.svelte';
  import { roomLatch } from '$common/stores/room-latch.svelte';
  import type { EntityDescriptor } from '$core/components/data-dependent/types';
  import EntityPanel from '$core/components/data-dependent/entity-panel.svelte';

  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // R2's data scope = the last SPACE selection, now held in the LIFTED `spaceLatch` `$common` store (C-1). It
  // was component `$state` here until this milestone; folding R2 unmounts the widget and DESTROYED that state,
  // so on unfold R2 read "Select a space" while roomLatch still targeted the room (Clair's F2). An
  // app-lifetime store survives the unmount. The shell drives `spaceLatch.note()` (one effect, two latches);
  // that writer writes `_latched` and never reads it, so there is no self-invalidating read-modify-write
  // (the N-136 trap avoided). R1 (`spaces-panel`) reads the SAME latch for its highlight (D4 opt-2, D-146).
  const scopedSpace = $derived(spaceLatch.scopedSpace);
  const rooms = $derived(scopedSpace?.rooms ?? []);

  function toDescriptor(r: KnownRoom): EntityDescriptor {
    return { kind: 'room', id: r.room_id, name: r.name };
  }
  // v1: descriptor only; `secondary`/`meta` UNFED (D6/D-065). `kind: 'room'` -> entity-avatar hexagon (J-501).
  const items = $derived(rooms.map((r) => ({ descriptor: toDescriptor(r) })));

  // `selected` is DERIVED FROM THE ROOM LATCH (`roomLatch.effectiveRoomId`, C-3), NOT the bus. A later identity
  // selection (a member row, the R3 self card) now KEEPS the room lit (D-146). `effectiveRoomId` — not the raw
  // `latchedRoomId` — is the room R5 renders and R6 targets, so the highlight can never disagree with them
  // (Phase-0 §4). It is `string | null`; the prop wants `string | undefined` (entity-panel:63), hence `?? undefined`.
  const selected = $derived(roomLatch.effectiveRoomId ?? undefined);

  function onActivate(roomId: string): void {
    const r = rooms.find((x) => x.room_id === roomId);
    if (r) selection.set(regionId, toDescriptor(r));
  }

  // Two honest empty states (N-091): no space scope -> "Select a space"; a scoped space with zero rooms ->
  // "No rooms". Distinct copy — different truths. entity-panel renders one `emptyText`; pick it by scope.
  const emptyText = $derived(scopedSpace === null ? 'Select a space' : 'No rooms');

  const debug = () => ({
    count: rooms.length,
    latchedSpaceId: spaceLatch.latchedSpaceId,
    selectedId: selected ?? null,
    hasEmpty: rooms.length === 0,
  });
</script>

<div class="rooms-panel" data-tier="widget" use:envelope={{ name: 'rooms-panel', id, debug }}>
  <EntityPanel {items} {selected} {onActivate} {emptyText} id={cid('panel')} />
</div>
