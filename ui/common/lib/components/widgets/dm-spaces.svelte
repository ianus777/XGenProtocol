<script lang="ts">
  // dm-spaces — the DM home (M-RP-MEMBER-ACT Leg E-1). A `kind: system` region widget (W-13,
  // non-removable); the swap is DERIVED via a `CLIENT_PLUGINS` descriptor (`surface: 'region'`,
  // `regionId: 'dm-spaces'`), exactly like spaces/rooms/members-panel — `buildWidgetRegistry` maps the
  // leaf onto it, NOT an `app_client` register line.
  //
  // The FOURTH `entity-panel` over the container umbrella (N-013: Spaces and Rooms are *containers* in
  // UI scope). R1 (`spaces-panel`) lists NON-DM containers; this lists DM containers. Same component,
  // same row shape, COMPLEMENTARY predicate — the shared `isDmSpace` export (spaces-state) is the one
  // source both readers use, so E-1 and E-3 (the R1 filter) can never drift (D-067).
  //
  // A VIEW, never a STORE (Joe: "the client is just reader-sender, doesn't hold any users data"). It
  // derives from `spacesState` + `addressBook` and persists nothing (D-121 lens ②). No draft row
  // (N-091), no controls (§5.7 census — no verb, no control).
  //
  // ⚠️ E-1 builds the home; it does NOT build the R1 filter (that is E-3). Until E-3 lands, a DM Space
  // renders in BOTH R1 and here — expected, temporary, not a defect (Phase-0 §5: the home ships before
  // the filter so no commit ever carries the filter without the home).
  import { envelope } from '$common/components/base/envelope';
  import { spacesState, isDmSpace, type KnownSpace } from '$common/stores/spaces-state.svelte';
  import { selection } from '$common/stores/selection.svelte';
  import { roomLatch } from '$common/stores/room-latch.svelte';
  import { spaceLatch } from '$common/stores/space-latch.svelte';
  import { dmDraft } from '$common/stores/dm-draft.svelte';
  import { selfState } from '$common/stores/self-state.svelte';
  import { addressBook } from '$common/stores/address-book.svelte';
  import type { EntityDescriptor } from '$core/components/data-dependent/types';
  import EntityPanel from '$core/components/data-dependent/entity-panel.svelte';

  // The leaf mount passes ONLY `regionId` (region-node.svelte). Derive the envelope id via the region-*
  // convention `id = "region-" + regionId` (→ `dm-spaces#region-dm-spaces`, panel child
  // `region-dm-spaces__panel`).
  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // D-142 fallback: where no display name is known, an identity shows `…` + the last 8 characters of its
  // key — never the raw long pubkey. COPIED from members-panel (L-8's copied-not-shared precedent: four
  // independent impls before J-508's extraction bar; do NOT refactor members-panel). The counterpart is
  // an XGID.
  const tail8 = (xgid: string) => (xgid ? `…${xgid.slice(-8)}` : '');

  // The self identity — the self thread's counterpart is the SESSION identity (spaces-state:32-34, backfilled
  // by K3 at ops.rs:94), so it is what pins the self row FIRST (§3.1). `null` before `get_self_state`
  // returns (a boot tick).
  const selfId = $derived(selfState.identity?.identity_id ?? null);

  // Leg E ③ = L2: resolve `counterpart` through the address book to a display name; fall back to `tail8`.
  // ONE label function, used for BOTH the row descriptor AND the sort key (§3.2). `descriptorFromId`
  // (members-panel:136) is COPIED, not lifted — the book is the GLOBAL `identity_id → SeenRecord` map
  // (get_address_book, NOT Space-scoped), so a counterpart absent from any roster can still resolve a name.
  // NEVER `KnownSpace.name` (L3, refused on D-143: a user-writable display string must not be a lookup key,
  // which is the whole reason `counterpart` exists).
  function labelFor(s: KnownSpace): string {
    const cp = s.counterpart ?? '';
    return addressBook.book[cp]?.display_name ?? tail8(cp);
  }

  // KnownSpace → descriptor. `kind: 'space'` + `flags.isDm: true` → entity-avatar draws a CIRCLE
  // (people-shaped, entity-avatar:56-62). This is the FIRST consumer to feed `isDm` — the flag has been
  // drawn but never fed until now (§3.1).
  function descriptorFor(s: KnownSpace): EntityDescriptor {
    return { kind: 'space', id: s.space_id, name: labelFor(s), flags: { isDm: true } };
  }

  // The DM rows: every `counterpart != null` Space (isDmSpace), self thread FIRST, the rest sorted by
  // resolved label case-insensitively (§3.1). `.filter().map()` copies before sort (never mutate the store
  // array). `dmRows` carries the resolved `space` so onActivate has no second lookup on the label.
  const dmRows = $derived.by(() => {
    const list = spacesState.spaces.filter(isDmSpace).map((s) => ({ space: s, label: labelFor(s) }));
    list.sort((a, b) => {
      const aSelf = a.space.counterpart === selfId;
      const bSelf = b.space.counterpart === selfId;
      if (aSelf !== bSelf) return aSelf ? -1 : 1; // self thread pinned first (§3.1 row 1)
      return a.label.toLocaleLowerCase().localeCompare(b.label.toLocaleLowerCase());
    });
    return list;
  });

  const items = $derived(dmRows.map(({ space }) => ({ descriptor: descriptorFor(space) })));

  // Highlight the open DM — the COMPLEMENT of R1 (spaces-panel:58-63). R1 suppresses its highlight when the
  // browsing latch resolves to a DM; this lights a DM row iff it resolves to a DM. Same `spaceLatch` source,
  // same `isDmSpace` predicate, INVERTED — so R1 and this panel can never both light (or both dark) for one
  // Space. Derived (not clicked) with `selectOnActivate={false}`, so it stays authoritative — the
  // M-RP-SELECT-ORIENT drift designed out (a clicked highlight drifts from the latch).
  // 🔓 NOTE (Rule 6): the runbook did not specify a highlight. This mirrors R1's shipped pattern, inverted,
  // for coherence with the arc's "panels keep saying where you are" goal. Flagged for Joe in the hand-back;
  // to drop it, remove `{selected}` and `selectOnActivate={false}` below.
  const selected = $derived.by(() => {
    const latched = spaceLatch.latchedSpaceId;
    if (latched == null) return undefined;
    const s = spacesState.spaces.find((x) => x.space_id === latched);
    return s != null && isDmSpace(s) ? latched : undefined;
  });

  // Activation (§3.3) — reuse the members-panel existing-DM shape (:267-268): latch the DM's ROOM (R5/R6)
  // AND move the browsing SPACE latch (R2 lists the DM's room, R1 goes unlit via its DM suppression), close
  // any open draft (KEEPING its text), and write the bus (R8 shows the DM Space). Both latches ALWAYS — a
  // lone room latch strands spaceLatch on the previous Space (the J-708 ① split; C-bis-6 fixed exactly this).
  // The self row is NOT excluded (§3.3, F5): the self thread is a real Space with a real room and this panel
  // is its ONLY GUI door — do NOT copy members-panel:249's `!== selfId` guard (that refuses a self *draft*).
  function onActivate(spaceId: string): void {
    const s = spacesState.spaces.find((x) => x.space_id === spaceId);
    if (!s) return;
    // A DM Space carries exactly one room. An empty `rooms` is a FINDING, not a case to paper over —
    // NO-OP and report, no invented fallback (§3.3; findDm at members-panel:234 guards it this way).
    const roomId = s.rooms[0]?.room_id;
    if (roomId == null) {
      if (import.meta.env.DEV)
        console.warn(`[dm-spaces] DM Space ${spaceId} has no room — activation is a no-op (finding)`);
      return;
    }
    roomLatch.latch(roomId);
    spaceLatch.latch(s.space_id);
    dmDraft.close(); // close(), NOT clear() — a half-typed message to someone else must survive (§5.3)
    selection.set(regionId, descriptorFor(s));
  }

  // Aggregate getter G (W-4): what the panel owns. `selfFirst` is `null` when empty (no first row to check).
  // Names/ids ride each row's own getter (no republish, N-060).
  const debug = () => ({
    count: dmRows.length,
    selfFirst: dmRows.length > 0 ? dmRows[0].space.counterpart === selfId : null,
    selectedId: selected ?? null,
    hasEmpty: dmRows.length === 0,
  });
</script>

<!-- No literal `class` on the wrapper: `use:envelope` STAMPS the type-class from `name` (`dm-spaces`), and
  `mergeClasses` does NOT dedupe (logic.ts:19), so a literal `class="dm-spaces"` would render
  `class="dm-spaces dm-spaces"` (the M-RP7.1-filed doubling). Writing new code correctly rather than adding a
  sixth instance of that defect; the stamped single `.dm-spaces` is still skinnable by Joe. -->
<div data-tier="widget" use:envelope={{ name: 'dm-spaces', id, debug }}>
  <EntityPanel
    {items}
    {selected}
    {onActivate}
    selectOnActivate={false}
    emptyText="No direct messages"
    id={cid('panel')}
  />
</div>
