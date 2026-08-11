<script lang="ts">
  // members-panel — R7 Members (M-RP-MEMBERS Leg B). A `kind: system` region widget (W-13); the swap is
  // DERIVED via a `CLIENT_PLUGINS` descriptor (`surface: 'region'`, `regionId: 'members'`), exactly like
  // spaces/rooms-panel — `buildWidgetRegistry` maps the leaf onto it, NOT an `app_client` register line.
  //
  // Shows WHO IS IN THIS CONVERSATION: the members of the latched Space's room (from the address-book store,
  // fed by the shell's fill), with each member's name resolved from the address-book cache. Reads THREE
  // stores — `roomLatch` (the scope, L1), `addressBook` (roster + book + phase), `selfState` (the self
  // fixture + the connection, for state ⑤). W-3: `core` imports no protocol type; the SHELL owns the fill.
  //
  // ⚠️ NOT A SELECTION SURFACE (Phase-0 line 284). The rows are INERT (`interactive={false}`,
  // M-RP-PANEL-INERT): no click, no keyboard, no bus write. The ONLY highlight is the DM counterpart (L16),
  // fed one-way through `selected` — the row RENDERS state, it never PRODUCES it. R7 must NEVER call
  // `selection.set()` (L15): writing the global bus would silently change what R8 inspector displays.
  //
  // ⚠️ SUPERSEDED, ANNOTATED NOT REPAIRED (D-131, J-675 + M-RP-MEMBER-ACT L-7).
  // The "must NEVER call selection.set()" rule above is REVERSED by L-7 (2026-08-06, Joe
  // uttered): LMC does BOTH — opens the DM and writes the bus, so R8 shows the member's
  // card. The REASON was inverted, not discarded: J-591 objected that the bus write would
  // be SILENT; under L-7 it is the point. Wiring lands in Leg C — the rows are still inert
  // here. This comment also overstates its own source (J-675); M-RP-PANEL-INERT recorded
  // inertness as DEFERRED, not rejected, so `interactive` is being USED, not overridden.
  //
  // ✅ NOW SHIPPED, ANNOTATED NOT REPAIRED (D-131, M-RP-MEMBER-ACT Leg C-3). The two blocks
  // above are HISTORY. The panel below is now `interactive={true}` with `selectOnActivate={false}`
  // and `onActivate={onMemberActivate}`, so the rows are NO LONGER inert and R7 DOES call
  // `selection.set()` — exactly the reversal L-7 named. The statements "the rows are still inert
  // here" and "`interactive={false}`" above are FALSE as of C-3; kept, not erased, so the reversal's
  // reasoning stays legible. `selectOnActivate={false}` is what keeps `counterpart` — the DERIVED DM
  // highlight (L16) — authoritative: a click writes the bus and latches the DM's room, but the panel
  // never overwrites `selected`. That is the M-RP-SELECT-ORIENT defect designed out (L-5).
  import { envelope } from '$common/components/base/envelope';
  import { roomLatch } from '$common/stores/room-latch.svelte';
  import { spaceLatch } from '$common/stores/space-latch.svelte';
  import { spacesState } from '$common/stores/spaces-state.svelte';
  import { selection } from '$common/stores/selection.svelte';
  import { addressBook, type MemberEntry } from '$common/stores/address-book.svelte';
  import { dmDraft } from '$common/stores/dm-draft.svelte';
  import { selfState } from '$common/stores/self-state.svelte';
  import type { EntityDescriptor } from '$core/components/data-dependent/types';
  import EntityPanel from '$core/components/data-dependent/entity-panel.svelte';
  import Paragraph from '$core/components/data-independent/paragraph.svelte';

  // The leaf mount passes ONLY `regionId` (region-node.svelte). Derive the envelope id via the region-*
  // convention `id = "region-" + regionId` (→ `members-panel#region-members`, panel child `region-members__panel`).
  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // D-142: where no display name is known, an identity shows `…` + the last 8 characters of its
  // key — never the raw long pubkey. The ellipsis is part of the STRING, not the skin: only this
  // widget knows the name is a fallback (a resolved member may simply have no display_name, and
  // an erased member may still carry a cached one), and no CSS selector can express that.
  const tail8 = (xgid: string) => (xgid ? `\u2026${xgid.slice(-8)}` : '');

  // ── The scope (L1) ────────────────────────────────────────────────────────────────────────────
  // `roomLatch.effectiveSpaceId` — `null` until a ROOM is latched (selecting a Space alone does NOT populate
  // R7, the deliberate B1 cost). The widget's authoritative scope; the store's own `spaceId` may lag it for a
  // tick, which is exactly what `rosterKnown` guards below.
  const scope = $derived(roomLatch.effectiveSpaceId);
  const connState = $derived(selfState.connection.state);

  // The roster is KNOWN iff the store holds one AND it is FOR the current scope — belt to the store's own
  // late-guard (§3.5), so a lagging roster from a previous scope never renders under a new heading.
  const rosterKnown = $derived(addressBook.roster !== null && addressBook.spaceId === scope);

  // ── The five states, one tree (§4) ────────────────────────────────────────────────────────────
  // ⑤ (offline) is distinguished from ④ (failed) by the CONNECTION, not the phase (L5 / §4 line 201): a
  // disconnected client that could not reach its node reads ⑤; a fill rejection while online reads ④.
  type PanelState = 'no-scope' | 'known' | 'inflight' | 'failed' | 'offline';
  const panelState: PanelState = $derived(
    scope == null
      ? 'no-scope' // ① self only, no message — no room picked, so there are no "others" to blame the net for
      : rosterKnown
        ? 'known' // ② self + all in room
        : connState === 'DISCONNECTED'
          ? 'offline' // ⑤ self + "I cannot see the others"
          : addressBook.phase === 'failed'
            ? 'failed' // ④ self + "I cannot reach the others"
            : 'inflight', // ③ self + "I am waiting for the others"
  );

  // Joe's copy, verbatim. ① and ② carry NO message (①: blaming the network for a scope never chosen is a
  // second false statement; ②: the members ARE the answer). ④ ("reach") vs ⑤ ("see") is nuance the
  // connection led distinguishes, not the verb (§4 line 201).
  const NOTE: Record<PanelState, string | null> = {
    'no-scope': null,
    known: null,
    inflight: 'I am waiting for the others',
    failed: 'I cannot reach the others',
    offline: 'I cannot see the others',
  };
  const message = $derived(NOTE[panelState]);

  // ── The rows ──────────────────────────────────────────────────────────────────────────────────
  const selfId = $derived(selfState.identity?.identity_id ?? null);

  // Self (L2/L17): resolved from `selfState`, NEVER the book. Always present, always FIRST, UNMARKED (its
  // position is the only mark). `null` only before `get_self_state` returns (a transient boot tick).
  const selfDescriptor: EntityDescriptor | null = $derived(
    selfState.identity
      ? {
          kind: 'identity',
          id: selfState.identity.identity_id ?? '',
          name:
            selfState.identity.display_name ?? tail8(selfState.identity.identity_id ?? ''),
          flags: {}, // self carries no isAi/revoked here
        }
      : null,
  );

  // A member → descriptor: name resolved from the book (last-resort xgid-tail), `isAi` from the book (L9 —
  // ONLY isAi; `revoked` SHIPS UNFED because the wire never sets it, so feeding it would light a shipped
  // badge from a constant false). `secondary`/`meta`/`status`/`role`/`joined_at` all UNFED/discarded (L10).
  function toDescriptor(m: MemberEntry): EntityDescriptor {
    const rec = addressBook.book[m.identity_id];
    return {
      kind: 'identity',
      id: m.identity_id,
      name: rec?.display_name ?? tail8(m.identity_id), // §5-iii + D-142: `…` + last-8 fallback
      // §5-iii (D): a live-added member (the `unresolved` marker, Leg A) asserts NO `isAi`. `isAi?` is
      // optional and an absent book record must render UNKNOWN, never DEFINITELY-NOT-AN-AI — defaulting
      // `false` from a missing record is the N-097 trap inverted (an AI joiner would render as human). Fill
      // members are never marked, so their branch is byte-for-byte unchanged.
      flags: m.unresolved ? {} : { isAi: rec?.is_ai ?? false },
    };
  }

  // A descriptor from a BARE identity id (no MemberEntry) — used for the C-bis-7 synthesised DM counterpart
  // row, which comes from the Space record, not the fill. `_book` is the GLOBAL address book
  // (`get_address_book` returns the bare `identity_id → SeenRecord` map, NOT Space-scoped —
  // address-book.svelte.ts:148), so a counterpart absent from the roster can still resolve a `display_name`;
  // otherwise the D-142 `…` + last-8 fallback. No book record ⇒ `flags {}` (UNKNOWN), NEVER `{ isAi: false }`:
  // defaulting from a missing record is the N-097 trap (an AI would render as human). This row is NEVER
  // stamped `unresolved` — that marker means "reached the roster via a live delta" and a Space-record
  // fabrication must not masquerade as one (§5a / M-RP-LIVEFEED-REFRESH Leg A).
  function descriptorFromId(idOf: string): EntityDescriptor {
    const rec = addressBook.book[idOf];
    return {
      kind: 'identity',
      id: idOf,
      name: rec?.display_name ?? tail8(idOf),
      flags: rec ? { isAi: rec.is_ai } : {},
    };
  }

  // ── The DM counterpart (L16 highlight + C-bis-7 row) ────────────────────────────────────────────
  // The counterpart's id comes from the SPACE RECORD (`KnownSpace.counterpart`), NOT the roster's non-self
  // member. OQ8-K3 writes it at DM creation, so a DM whose counterpart never joined the room still knows who
  // it is with — the fill's roster would hold only self, but the Space record names them (J-711). `null` for
  // a group Space ⇒ `undefined` here (no highlight, no synthesised row). ⚠️ `KnownSpace.counterpart` is the
  // SESSION identity for the self thread (spaces-state:32-34); self is NEVER the counterpart (L17 — self
  // stays unmarked), so exclude it explicitly — a bare field match would mark self's own thread.
  // `scope` non-null ⇒ its Space is in `spacesState.spaces` by construction (roomLatch resolves scope FROM
  // that list), so the `.find` cannot miss when scoped.
  const counterpart = $derived.by(() => {
    const cp = spacesState.spaces.find((s) => s.space_id === scope)?.counterpart;
    return cp != null && cp !== selfId ? cp : undefined;
  });

  // §5/§5a — the ③ set for the current roster. `notFoundIds` is a small array; a Set keeps the
  // per-row test O(1) and reads as the membership question it is.
  const notFound = $derived(new Set(addressBook.notFoundIds));

  // State ② members: the roster MINUS self (L4), MINUS state ③ — except the DM counterpart,
  // which is NEVER hidden (§5a E2, J-648). ⚠️ `counterpart` is `undefined` outside a DM
  // (G-B9), so `=== counterpart` IS the DM exception; do NOT add a separate `isDm` test.
  // B-1: `_roster` stays complete — this filters at RENDER, never in the store.
  const memberRows = $derived(
    panelState === 'known' && addressBook.roster
      ? addressBook.roster
          .filter((m) => m.identity_id !== selfId)
          .filter((m) => !notFound.has(m.identity_id) || m.identity_id === counterpart)
          .map((m) => ({
            descriptor: toDescriptor(m),
            unresolved: notFound.has(m.identity_id)
              ? ('erased' as const)
              : m.unresolved
                ? ('unasked' as const)
                : undefined,
          }))
      : [],
  );

  // C-bis-7 — the DM counterpart may be a participant of the stream you are looking at WITHOUT being in the
  // fill's roster: a DM whose counterpart never joined the room returns a roster of self only (J-711), but the
  // Space record still names them (`counterpart` above). Joe's rule — R7 shows the participants of the stream
  // you are looking at, WHENEVER THEY EXIST — so synthesise the counterpart row from the Space record when the
  // roster did not already supply it. In an existing DM whose counterpart DID join, `memberRows` already
  // carries them ⇒ no synthesis (never duplicate). In a DRAFT `counterpart` is `undefined` (the scope is the
  // group Space, whose `counterpart` is `null`) ⇒ no synthesis: the DM does not exist yet, and inventing two
  // rows would be a roster no fill produced (D-065). `memberCount` (the fill count) stays as-is while `rowCount`
  // gains this row — that disagreement is the TRUTH, not a defect (trap ①): the second row is from the Space
  // record, not the fill, and inflating `memberCount` would make a frontend count masquerade as a wire count.
  const counterpartRow = $derived(
    panelState === 'known' &&
      counterpart != null &&
      !memberRows.some((r) => r.descriptor.id === counterpart)
      ? { descriptor: descriptorFromId(counterpart) }
      : null,
  );

  // The rendered rows: self FIRST (present in all five states, L2), then the other members (state ② only),
  // then the synthesised DM counterpart if the roster did not carry it (C-bis-7).
  const rows = $derived(
    (selfDescriptor ? [{ descriptor: selfDescriptor }] : [])
      .concat(memberRows)
      .concat(counterpartRow ? [counterpartRow] : []),
  );

  // ── Interaction (M-RP-MEMBER-ACT Leg C-3 / C-bis-2) ─────────────────────────────────────────────
  // R7 is a selection surface: `interactive={true}`, `selectOnActivate={false}`. Clicking a member opens
  // that member's EXISTING DM (L-1/L-7) and leaves the IDENTITY on the bus. As of Leg C-bis-2, a member
  // with NO existing DM opens a DRAFT instead (J-692 option B) — the row is no longer a dead control.

  // L-8: the DM lookup — a NAMED LOCAL FUNCTION, deliberately not inlined and not extracted to a shared
  // helper. Copied-not-shared follows J-508's roving precedent (four independent impls before extraction
  // earned its own milestone). The anticipated second caller is M-RP-PEOPLE (a people-panel over the
  // address book); before any lift, settle "is a book entry enough to OFFER a DM?" — a book entry is not
  // proof a DM exists, and this lookup answers only the latter.
  function findDm(identityId: string): { space_id: string; room_id: string } | null {
    // Self is never a DM target. `KnownSpace.counterpart` holds the session identity for the SELF-THREAD
    // space (spaces-state:32-34), so a bare field match would resolve self's own thread. R7 already never
    // marks self (L17), but the lookup must NOT rely on that — exclude it explicitly (§4).
    if (identityId === selfId) return null;
    // A field match, not a search: `counterpart` names the DM's other party, and `rooms` rides embedded
    // (G7/G8) — no second fetch. `null` counterparts (real Spaces) never equal a member id.
    const space = spacesState.spaces.find((s) => s.counterpart === identityId);
    if (!space) return null; // no existing DM — §5-b: the click is a silent no-op (J-692 unruled).
    // A DM space carries exactly one room. An empty `rooms` here should not happen; if it does, do nothing
    // — it is a FINDING, not a case to paper over (§4). No invented fallback.
    // Return BOTH ids from the space already in hand (C-bis-6): the caller latches the ROOM and moves the
    // browsing SPACE latch, and both come from THIS self-excluded match — no second finder to drift from it
    // (room-latch:20-22 rejects that D-067 shape by name).
    const room_id = space.rooms[0]?.room_id;
    if (room_id == null) return null;
    return { space_id: space.space_id, room_id };
  }

  // On a member click, one of three outcomes: an existing DM latches, a member with no DM opens a draft,
  // or a no-op (self / the empty-rooms finding). Where writes fire they are to INDEPENDENT stores and are
  // NOT ordering-dependent — do not "fix" them into a coupling. (N2 was refused precisely because it wrote
  // the SAME store twice, coalescing into one $effect run that never observed the room; these write
  // DIFFERENT stores, so no such coalescing exists.)
  function onMemberActivate(identityId: string): void {
    // Self is never a DM target NOR a draft target. `findDm` already returns null for self, but the
    // no-DM branch below now OPENS A DRAFT on "no DM" — so without this guard a self-click (the self row is
    // clickable) would open a self-draft. This preserves Leg C's self-click no-op. Self IS in `_roster`
    // (memberRows filters it out only for RENDER), so the `!m` guard below does NOT catch it.
    if (identityId === selfId) return;
    // N-171 (fixed here because this leg opens this function): the roster lookup goes ABOVE every write.
    // `latch()`/`clear()` are unconditional while `selection.set()` sat behind an `if (m)` guard, so the
    // pair could HALF-APPLY (R5 shows the DM while R8 shows the old card). Doing the lookup first means
    // either BOTH writes fire or NEITHER does. The locked write ORDER is untouched — a LOOKUP moves, not a
    // WRITE.
    const m = addressBook.roster?.find((x) => x.identity_id === identityId);
    if (m == null) return; // not a rendered roster row — nothing to open, nothing to select.
    const descriptor = toDescriptor(m);

    const dm = findDm(identityId);
    if (dm != null) {
      // Existing DM (Leg C): latch its ROOM — R5 shows it, R6 targets it (L-1/L-2). C-bis-6 (Joe's rule):
      // ALSO move the BROWSING Space latch to the DM's Space, so R2 lists the DM's one room and R1 (which
      // suppresses DM highlights) goes unlit — `roomLatch.latch()` alone left `spaceLatch` on the previous
      // Space, so R2 drew that Space's rooms while highlighting the DM's room, a dangling highlight (J-708 ①).
      // Both ids come from `findDm`'s single self-excluded match — one lookup, two writes to INDEPENDENT
      // stores (the shell's one-effect-both-note()s shape), NOT a coupling.
      roomLatch.latch(dm.room_id);
      spaceLatch.latch(dm.space_id);
      // C-bis-7 / J-713: opening an existing DM is navigating to a conversation, so CLOSE any open draft —
      // otherwise `stream-panel:231` keeps suppressing this DM's stream (`streamMessages = []` while
      // `dmDraft.active`) and R5 stays captioned for the OLD draft while R7/R2 have already moved. The draft's
      // shipped close trigger is a `room` selection (`dm-draft:note`), but this path writes an IDENTITY (L-7)
      // and latches directly, which `note()` correctly ignores — so the caller does what the bus effect would
      // have (the C-bis-6 shape). 🛑 `close()`, NOT `clear()`: `clear()` DELETES the draft's text (post-send);
      // `close()` KEEPS it, so a half-typed message to someone else survives the visit (§5.3).
      dmDraft.close();
      // Leave the person on the bus so R8 shows the card (L-7).
      selection.set(regionId, descriptor);
      return;
    }
    // `dm` is null. Self is excluded above, so this is either NO DM space at all (→ open a draft) or a
    // DM space that exists but has an empty `rooms` (the finding `findDm` guards). The latter is an
    // EXISTING (malformed) DM: drafting on it would mint a DUPLICATE space on send, which the milestone's
    // dedup design (K3, gate 4b) exists to prevent — so it stays a no-op, NOT a draft.
    if (spacesState.spaces.some((s) => s.counterpart === identityId)) return;
    // No DM space exists — J-692 option B: open a DRAFT.
    // 🛑 v1.3: NO `roomLatch.clear()`. The kickoff's R-1 (clear the latch on open) was WITHDRAWN after it was
    // driven: R7's scope IS `roomLatch.effectiveSpaceId` (`:57`), so clearing emptied R7 to `no-scope` and
    // (app_client.svelte:218) tore down its fill on every open/close. `active` is now STORED
    // (`counterpart != null`), so opening the draft needs no clear; the draft CLOSES on a room selection via
    // `dmDraft.note` in the shell effect. The send-to-stale-room defence R-1 duplicated already lives in
    // C-bis-3 (the draft branch ABOVE the composer's early return). ⇒ the latch STAYS, R7 keeps its group
    // roster and fill, and `canSend` is true during a draft — harmless, because C-bis-3's draft branch never
    // consults the latch.
    dmDraft.open(identityId);
    // R8 shows the person you are drafting (L-7). An `identity` selection is ignored by BOTH `roomLatch.note`
    // and `dmDraft.note`, so writing the bus neither moves the latch nor closes the draft it just opened.
    selection.set(regionId, descriptor);
  }

  // Aggregate getter G (W-4). ⚠️ `memberCount` derives from the ROSTER, never from rendered rows (L4) — the
  // self fixture is not a roster entry. Names/ids ride each row's own getter (no republish, N-060).
  const debug = () => ({
    state: panelState,
    scope: scope ?? null,
    memberCount: addressBook.roster?.length ?? null,
    rowCount: rows.length,
    erasedHidden: (addressBook.roster ?? []).filter(
      (m) => m.identity_id !== selfId && notFound.has(m.identity_id) && m.identity_id !== counterpart,
    ).length,
    isDm: addressBook.isDm,
    counterpart: counterpart ?? null,
    hasMessage: message !== null,
    phase: addressBook.phase,
  });
</script>

<div class="members-panel" data-tier="widget" use:envelope={{ name: 'members-panel', id, debug }}>
  <!-- ACTIVE (M-RP-MEMBER-ACT Leg C-3, was Inert): clicking a member opens its existing DM and writes the
    bus. `selectOnActivate={false}` keeps the highlight DERIVED (L16) — never a click write (L-5); `onActivate`
    opens the DM + writes the bus (L-1/L-7). C-bis-7 EXTENDS L16 to the DRAFT: `dmDraft.counterpart` highlights
    the drafted-to row (during a draft the stream IS about that person), still derived, still one-way. `??
    undefined` coerces the store's `string | null` to the prop's `string | undefined` (null and undefined both
    mean "no selection"). In a DM `counterpart` wins; in a draft it is `undefined` and the draft target shows. -->
  <EntityPanel
    items={rows}
    selected={counterpart ?? dmDraft.counterpart ?? undefined}
    interactive={true}
    selectOnActivate={false}
    onActivate={onMemberActivate}
    id={cid('panel')}
  />
  {#if message}
    <Paragraph text={message} id={cid('note')} />
  {/if}
</div>
