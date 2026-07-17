<script lang="ts">
  // spaces-panel — R1 Spaces (M-RP6.2, D8). A `kind: system` region widget (W-13, non-removable) that
  // replaces the `spaces` placeholder in the layout registry. The swap is DERIVED, not registered: this
  // widget is a `PluginDescriptor` in `CLIENT_PLUGINS` (`surface: 'region'`, `regionId: 'spaces'`), and
  // `buildWidgetRegistry(installed.mounted)` maps the leaf onto it — the self/inspector precedent (M-RP6.1l),
  // NOT an `app_client` register line.
  //
  // Renders an `entity-panel` of the known Spaces (from the `spacesState` $common store, D1) and WRITES the
  // selection bus on activation — the bus's SECOND writer (R3 self was the first). Selecting a Space here
  // repopulates R2 (rooms-panel latches the space) and R8 (inspector reads the bus) — the milestone's real
  // cross-region flow.
  //
  // PROJECTION IN THE WIDGET (D7): the store carries raw `KnownSpace[]`; the `KnownSpace -> EntityDescriptor`
  // map lives here (the self-panel precedent). `core` imports no protocol type (W-3).
  //
  // `selected` is DERIVED FROM THE BUS (D5, the self-panel standing rule) and highlights a Space ONLY while
  // the bus holds THAT space (D4 opt-1, bus-pure). When R2 selects a room the bus becomes a room, so R1
  // un-highlights while you browse its rooms — one selection, one meaning, no second latch.
  import { envelope } from '$common/components/base/envelope';
  import { spacesState, type KnownSpace } from '$common/stores/spaces-state.svelte';
  import { selection } from '$common/stores/selection.svelte';
  import type { EntityDescriptor } from '$core/components/data-dependent/types';
  import EntityPanel from '$core/components/data-dependent/entity-panel.svelte';

  // The leaf mount passes ONLY `regionId` (region-node.svelte). Derive the envelope id via the region-*
  // convention `id = "region-" + regionId` (→ `spaces-panel#region-spaces`, panel child `region-spaces__panel`).
  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // KnownSpace -> descriptor. `kind: 'space'` -> entity-avatar draws a rounded-square (or circle when a
  // future `flags.isDm` is set, J-501); no isDm here -> the Space shape. Name falls back to the id in the
  // avatar/row if empty (the entity-item contract) — no fake name.
  function toDescriptor(s: KnownSpace): EntityDescriptor {
    return { kind: 'space', id: s.space_id, name: s.name };
  }

  const spaces = $derived(spacesState.spaces);
  // v1: only the descriptor is fed. `secondary`/`meta` ship UNFED (D6/D-065) — no faked topic / last-message
  // / unread (the read-marker gap has no protocol mechanism yet).
  const items = $derived(spaces.map((s) => ({ descriptor: toDescriptor(s) })));

  // Read the bus BACK (D5). R1 owns the 'space' facet of the one selection; a room selection (R2) leaves
  // this undefined -> R1 un-highlights (D4 opt-1).
  const selected = $derived(
    selection.current?.entity.kind === 'space' ? selection.current.entity.id : undefined,
  );

  function onActivate(spaceId: string): void {
    const s = spaces.find((x) => x.space_id === spaceId);
    if (s) selection.set(regionId, toDescriptor(s));
  }

  // Aggregate getter G (W-4): what the panel owns. Names/ids ride each row's own getter (no republish, N-060).
  const debug = () => ({
    count: spaces.length,
    selectedId: selected ?? null,
    hasEmpty: spaces.length === 0,
  });
</script>

<div class="spaces-panel" data-tier="widget" use:envelope={{ name: 'spaces-panel', id, debug }}>
  <EntityPanel {items} {selected} {onActivate} emptyText="No spaces yet" id={cid('panel')} />
</div>
