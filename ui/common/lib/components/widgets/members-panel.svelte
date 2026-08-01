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
  import { envelope } from '$common/components/base/envelope';
  import { roomLatch } from '$common/stores/room-latch.svelte';
  import { addressBook, type MemberEntry } from '$common/stores/address-book.svelte';
  import { selfState } from '$common/stores/self-state.svelte';
  import type { EntityDescriptor } from '$core/components/data-dependent/types';
  import EntityPanel from '$core/components/data-dependent/entity-panel.svelte';
  import Paragraph from '$core/components/data-independent/paragraph.svelte';

  // The leaf mount passes ONLY `regionId` (region-node.svelte). Derive the envelope id via the region-*
  // convention `id = "region-" + regionId` (→ `members-panel#region-members`, panel child `region-members__panel`).
  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // Last-resort name: the xgid's final path segment (the `entity-panel` itemKey shape). Used when the book
  // holds no `display_name` — v1 renders the last resort of the name chain for everybody (Phase-0).
  const tail = (xgid: string) => xgid.split('/').pop() || xgid;

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
            selfState.identity.display_name ?? tail(selfState.identity.identity_id ?? ''),
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
      name: rec?.display_name ?? tail(m.identity_id), // §5-iii: NAME unchanged — tail() as today
      // §5-iii (D): a live-added member (the `unresolved` marker, Leg A) asserts NO `isAi`. `isAi?` is
      // optional and an absent book record must render UNKNOWN, never DEFINITELY-NOT-AN-AI — defaulting
      // `false` from a missing record is the N-097 trap inverted (an AI joiner would render as human). Fill
      // members are never marked, so their branch is byte-for-byte unchanged.
      flags: m.unresolved ? {} : { isAi: rec?.is_ai ?? false },
    };
  }

  // ── The DM counterpart highlight (L16) ────────────────────────────────────────────────────────
  // The counterpart's id iff this is a DM, else `undefined` (NO highlight in a group room). The counterpart
  // is the non-self member; `selected` flows one-way to the inert row's highlight (M-RP-PANEL-INERT). Self is
  // never the counterpart (L17 — self stays unmarked).
  const counterpart = $derived(
    panelState === 'known' && addressBook.isDm && addressBook.roster
      ? addressBook.roster.find((m) => m.identity_id !== selfId)?.identity_id
      : undefined,
  );

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

  // The rendered rows: self FIRST (present in all five states, L2), then the other members (state ② only).
  const rows = $derived(
    (selfDescriptor ? [{ descriptor: selfDescriptor }] : []).concat(memberRows),
  );

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
  <!-- Inert (M-RP-PANEL-INERT): render-only, no click/keyboard/bus. `selected` = the DM counterpart, one-way
    (L16). No `onActivate` → no bus write (L15 — R7 is not a selection surface, Phase-0 line 284). -->
  <EntityPanel items={rows} selected={counterpart} interactive={false} id={cid('panel')} />
  {#if message}
    <Paragraph text={message} id={cid('note')} />
  {/if}
</div>
