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
  // `selected` is DERIVED FROM THE LIFTED SPACE LATCH (`spaceLatch.latchedSpaceId`, C-1), NOT the bus, and
  // highlights the Space you are BROWSING — so selecting a room KEEPS its Space lit (D4 opt-1 -> opt-2, D-146:
  // the M-RP6.2 bus-pure lock is superseded now the latch exists to read). R1 and R2 read ONE value, so they
  // always agree; the bus is still WRITTEN on activation (onActivate), it is just no longer READ here for the
  // highlight.
  //
  // C-bis-6 (Joe's rule): R1 highlights only NON-DM Spaces. When the browsing latch resolves to a DM
  // (`KnownSpace.counterpart != null`), the highlight is SUPPRESSED — entering a DM UNSELECTS the tree rather
  // than lighting a DM row. It is A3's render-only DM filter in miniature (F-D): the SUPPRESSION lives in the
  // `selected` $derived, NEVER the store — a store-side DM filter would make every DM unsendable, since
  // `resolveLatched`/`canSend` both read `spacesState`. R2 still lists the DM's rooms (its scope is the same
  // latch), so a DM shows a one-row `dm` column with R1 dark — "A tells you where you are".
  import { envelope } from '$common/components/base/envelope';
  import { spacesState, isDmSpace, type KnownSpace } from '$common/stores/spaces-state.svelte';
  import { selection } from '$common/stores/selection.svelte';
  import { spaceLatch } from '$common/stores/space-latch.svelte';
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

  // E-3 (OQ3 = A3, Joe J-709): DM Spaces leave R1's RENDER — but ONLY the render. `visible` is the rendered set
  // (non-DM Spaces); `spaces` stays the FULL set. Both are used deliberately: `visible` feeds `items` and the
  // aggregate getter (LOCK 2/F2); `spaces` still feeds `selected` (C-bis-6 needs a DM PRESENT to recognise and
  // suppress it — filtering here would invert the guard, LOCK 1/F1) and `onActivate`. The store itself is NEVER
  // filtered (F3) — a store-side filter would break the DM home, both latches, `canSend` and member activation.
  // Complement of `dm-spaces` via the shared `isDmSpace` export, so R1 and the DM home can never disagree about
  // one Space (D-067).
  const visible = $derived(spaces.filter((s) => !isDmSpace(s)));

  // v1: only the descriptor is fed. `secondary`/`meta` ship UNFED (D6/D-065) — no faked topic / last-message
  // / unread (the read-marker gap has no protocol mechanism yet).
  const items = $derived(visible.map((s) => ({ descriptor: toDescriptor(s) })));

  // Highlight from the LIFTED latch (C-1, D-146), not the bus. The latch holds the last `space` selection, so
  // a later room/identity selection KEEPS the Space lit (D4 opt-2). `latchedSpaceId` is `string | null`; the
  // prop wants `string | undefined` (entity-panel:63), hence `undefined` for "no highlight".
  // C-bis-6: SUPPRESS the highlight when the latched Space is a DM (`counterpart != null`) — R1 lights only
  // non-DM Spaces (Joe's rule). A stale id that no longer resolves in `spaces` keeps highlighting the raw id,
  // exactly as before (undefined `s` → not suppressed).
  const selected = $derived.by(() => {
    const id = spaceLatch.latchedSpaceId;
    if (id == null) return undefined;
    const s = spaces.find((x) => x.space_id === id);
    return s?.counterpart != null ? undefined : id;
  });

  function onActivate(spaceId: string): void {
    const s = spaces.find((x) => x.space_id === spaceId);
    if (s) selection.set(regionId, toDescriptor(s));
  }

  // Aggregate getter G (W-4): what the panel owns. Names/ids ride each row's own getter (no republish, N-060).
  // `count`/`hasEmpty` follow the RENDER (`visible`), not the store — after E-3 the panel owns the visible rows,
  // so an unfiltered getter would report 7 while 4 render (F2 — the primary verify surface must not lie).
  const debug = () => ({
    count: visible.length,
    selectedId: selected ?? null,
    hasEmpty: visible.length === 0,
  });
</script>

<div class="spaces-panel" data-tier="widget" use:envelope={{ name: 'spaces-panel', id, debug }}>
  <EntityPanel {items} {selected} {onActivate} emptyText="No spaces yet" id={cid('panel')} />
</div>
