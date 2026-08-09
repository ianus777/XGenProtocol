<script lang="ts">
  // stream-panel — R5 Message stream (M-RP6.3 Leg C2). A `kind: system` region widget; the swap is DERIVED
  // via a `CLIENT_PLUGINS` descriptor (`surface: 'region'`, `regionId: 'stream'`), exactly like
  // spaces-panel / rooms-panel (F-1: no `app_client` register line — `buildWidgetRegistry` picks it up).
  //
  // It PROJECTS the live `ingest` store into `MessageDescriptor[]` on READ (C-4: never a mirror store), feeds
  // the shipped `message-stream` core (C1: region-fitting height model, live dividers, the status row kind),
  // and feeds the connection-gap `status` array from the `gaps` $common store (C-10). It touches ZERO `core`
  // and ZERO Rust — C1 shipped every `core` change this milestone needs.
  //
  // THE ROOM LATCH (C-5, F-6): R5 scopes to the last `kind: 'room'` bus selection and KEEPS it (its own
  // click would move the bus to a message — except `onSelect` is a RESERVED hook that never writes the bus,
  // so the latch is a byte-for-byte copy of the rooms-panel D3 shape and the N-136 trap is UNREACHABLE). The
  // effect reads `selection.current` and WRITES the latch; it never READS the latch (no self-invalidating
  // read-modify-write).
  //
  // THE HEAD MARKER (C-9) is a synthetic `MessageDescriptor{kind:'system'}` the widget PREPENDS — NOT a
  // fifth status phase, NOT a fourth row kind (both `core`, forbidden). `system` is an authorless centred
  // notice (`message.svelte`), the same render the membership rows use. There is NO backfill, so the marker
  // says "the view begins at session start" (C-6) and its second state says part of the session was dropped
  // (F-5). It makes `core`'s "No messages yet" unreachable (count ≥ 1 forever), so BOTH empty states are
  // widget-composed as system rows (C-5 amended).
  import { envelope } from '$common/components/base/envelope';
  import MessageStream from '$core/components/data-dependent/message-stream.svelte';
  import { resolveMounts } from '$core/components/data-dependent/mounts';
  import type { MessageDescriptor, WidgetMount } from '$core/components/data-dependent/types';
  import { ingest, type IngestEvent } from '$common/stores/ingest.svelte';
  import { selfState } from '$common/stores/self-state.svelte';
  import { gaps } from '$common/stores/gaps.svelte';
  // M-RP6.3 Leg D2 — the lifted room latch (§2.1) and the outbound echo store (§9.11.2).
  import { roomLatch } from '$common/stores/room-latch.svelte';
  import { echo, type EchoMessage } from '$common/stores/echo-state.svelte';
  import SendStatus from './send-status.svelte';
  // The DM draft "start of conversation" page — a socket tenant (below), Joe's file (skin.css precedent).
  import DmIntro from './dm-intro.svelte';
  // The pure R5 projection (the allowlist + the wire-field finding) — colocated + unit-tested (the
  // stream/grouping.ts precedent), so the map is verified in vitest, not only live (§5).
  import { wireType, projectEvent } from './stream/derive';

  let { regionId, id = `region-${regionId}` }: { regionId: string; id?: string } = $props();
  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // Reserved synthetic-row ids (C-9) — a `__`-prefix so they can never collide with an `event_id`.
  const HEAD_ID = '__head__';
  const NOMSG_ID = '__nomsg__';
  const NOROOM_ID = '__noroom__';

  // Functional copy (Ms Design's WORDING is PROVISIONAL, the spaces-panel `emptyText` precedent — appearance
  // and final phrasing → M-RP-SKIN). The MEANING is locked (§3.4/C-6): honest about the no-backfill window.
  // ⚠️ WORDING WIDENED AT LEG D2 (lock #8), and the reason is not style. Leg C2's copy said "messages
  // RECEIVED since you connected", which was complete when the only rows were inbound. Now the user's OWN
  // sends live here too, and they are just as session-mortal — the echo store dies on reload. A marker that
  // confesses only the inbound half would be partial in the direction that hurts most: they never read the
  // messages they lost; THEY WROTE the one they lost.
  const SESSION_START = 'Showing this session only — earlier messages, including your own, are not loaded.';
  const SESSION_START_DROPPED =
    'Showing this session only — earlier messages, including your own, are not loaded; some were dropped.';
  const NO_MESSAGES = 'No messages in this room yet.';
  const SELECT_ROOM = 'Select a room to see its messages.';

  // Session start — captured once at mount; the head marker anchors here (it precedes the first message).
  const sessionStart = Date.now();

  // ── The room latch (C-5, F-6) — LIFTED to `$common` at Leg D2 (§2.1). The logic is UNCHANGED: it still
  // latches the last `kind: 'room'` bus selection, still keeps it while the bus holds something else, and
  // still falls back to "select a room" when the latched room no longer resolves (stale-latch guard, N-095
  // spirit — never throw). What moved is WHERE it lives, so R6 (the composer) acts on the SAME room this
  // stream is showing; a second copy of the rule is the D-067 drift that would grey the composer out on the
  // conversation the user is reading. The shell drives the single writer (`roomLatch.note`) app-lifetime,
  // so folding either tile no longer forgets the room. `onSelect` is still NOT wired to selection.set
  // (§3.3), so nothing here moves the bus. ──
  const latchedRoomId = $derived(roomLatch.latchedRoomId);
  const effectiveRoomId = $derived(roomLatch.effectiveRoomId);

  // ── The projection (C-2, C-4) — project on READ off `ingest.events`, filtered by the latched room. The
  // allowlist + wire-field reading are the pure `./stream/derive` module (unit-tested). ──
  const selfId = $derived(selfState.identity?.identity_id ?? null);

  // Redacted target ids (this room) — a `message.redact` carries `content.target_event_id` (grounded,
  // node/runtime.rs:576); it is NOT a row (C-2), it mutates the referenced message's `deleted`.
  const redactedIds = $derived(
    new Set(
      ingest.events
        .filter((e) => wireType(e) === 'message.redact' && e.room_id === effectiveRoomId)
        .map((e) => e.content?.target_event_id as string | undefined)
        .filter((x): x is string => typeof x === 'string'),
    ),
  );

  const inbound = $derived(
    effectiveRoomId == null
      ? []
      : ingest.events
          .filter((e) => e.room_id === effectiveRoomId)
          .map((e) => projectEvent(e, selfId, redactedIds))
          .filter((m): m is MessageDescriptor => m !== null),
  );

  // ── The outbound echo merge (M-RP6.3 Leg D2, lock #9) ──────────────────────────────────────────────
  // Your own words render NOWHERE without this: the node excludes the author from fan-out by identity, so a
  // send returns `accepted` and `ingest` never sees it. C-4 still governs INBOUND unchanged (project on
  // read, no mirror); this is the narrow, documented outbound exception (§9.11.2).
  //
  // 🔑 THE ROW'S `id` IS THE `localId` AND STAYS THE `localId` — NEVER the stitched `eventId`. The stream
  // keys its rows by descriptor id, so flipping the id when the outcome lands would make Svelte destroy and
  // recreate the row: the send-status mount would remount, its state reset, and V3's "the row is the SAME
  // row" would be false in the only sense that matters (N-125, the index-key lesson one socket over).
  // `eventId` rides the echo record instead, where the status widget reads it.
  //
  // Echoes are real `MessageDescriptor`s, so grouping, dividers and own-side alignment come free (lock #9),
  // and the send-status widget rides `bodyExtras` — BELOW the body, OUTSIDE the header guard, so it survives
  // a grouped continuation row (M-RP6.9 D-5; in `details` a failed third message in a run would look
  // identical to a delivered first).
  function echoToDescriptor(e: EchoMessage): MessageDescriptor {
    return {
      kind: 'text',
      id: e.localId,
      author: { kind: 'identity', id: selfId ?? '' }, // NO name (C-8) — xgid-tail initials, same as inbound
      body: e.text,
      timestamp: new Date(e.sentAt).toISOString(), // client-minted and stays that way (lock #4)
      isOwn: true,
      // `mountKey` is a CONSTANT, not localId-composed: `resolveMounts` scopes keys PER ROW, so one mount
      // per row is already unique, and M-RP6.9's duplicate-key crash is a within-one-each-block condition.
      // A row-unique key here would only lengthen the registry id it anchors (`…__x-send-status`).
      bodyExtras: [{ widgetId: 'send-status', mountKey: 'send-status', props: { localId: e.localId } }],
    };
  }

  const outbound = $derived(echo.forRoom(effectiveRoomId).map(echoToDescriptor));

  // Interleave by timestamp so an echo sits where it was written relative to what arrived around it.
  // ⚠️ `accepted` carries NO authoritative timestamp (§9.11.5), so own rows keep their client-minted time
  // and MAY order differently for the author than for everyone else in the room. Real, permanent within a
  // session, FILED — not concealed by sorting them to the end.
  const projected = $derived(
    [...inbound, ...outbound].sort((a, b) => Date.parse(a.timestamp) - Date.parse(b.timestamp)),
  );

  // The `bodyExtras` consumer registry (W-13): an id the host cannot resolve is DROPPED on render, so a
  // future tenant is additive here and a retired one degrades to nothing rather than crashing the row.
  const widgets = { 'send-status': SendStatus };

  // ── The `above` socket (Leg C-bis-1) ─────────────────────────────────────────────────────────────────
  // A fixed-size layer rendered ABOVE the stream, inside the same flex column (§2 of the runbook). J-704
  // measured that the stream FILLS its tile (18px of content in 544px) and will not collapse, so a NAIVE
  // sibling above it would OVERFLOW rather than share — the flex column is what makes them share.
  //
  // STORE-MEDIATED, NOT PROP-FED (N-096): a region widget receives only `regionId`/`id`, so there is no
  // prop channel — Leg C-bis-2 wires `above` to the `dmDraft` store. C-bis-1 lands it EMPTY on purpose:
  // this is the plumbing, not the tenant, so nothing mounts and the registry floor is unmoved. The
  // registry is LOCAL (the `widgets`/send-status shape one block up), because a socket registry cannot
  // arrive as a prop either. `dm-intro` is Joe's file — a structural placeholder here.
  // ⚠️ Reactivity (D-119 paid for it once): `resolveMounts` re-derives on the `aboveWidgets` REFERENCE, so
  // it is a stable const, never mutated in place. Unknown ids are DROPPED (W-13), so `aboveMountCount` is
  // the render truth (a drop-unknown proof), exactly like `backgroundMountCount`.
  const aboveWidgets = { 'dm-intro': DmIntro };
  const above: WidgetMount[] = []; // Leg C-bis-2 replaces this with a `$derived` off `dmDraft`.
  const resolvedAbove = $derived(resolveMounts(above, aboveWidgets, cid('a-')));

  // The head marker anchors at (or before) the first message so it sits first and puts no spurious divider
  // between itself and the first message (a session crossing midnight still puts a truthful divider, §3.4).
  function headTimestamp(): string {
    let ms = sessionStart;
    if (projected.length) {
      const t = Date.parse(projected[0].timestamp);
      if (!Number.isNaN(t)) ms = Math.min(ms, t);
    }
    return new Date(ms).toISOString();
  }

  // The composed message array (C-5 amended — both empty states widget-composed as `system` rows):
  //   no room latched  → [ "Select a room" ]                (NO head marker — nothing to mark)
  //   room, no messages → [ head marker, "No messages" ]     (count 2 synthetic — the §4 measurement note)
  //   room, messages    → [ head marker, …projected ]
  const messages = $derived.by((): MessageDescriptor[] => {
    if (effectiveRoomId == null) {
      return [{ kind: 'system', id: NOROOM_ID, body: SELECT_ROOM, timestamp: new Date(sessionStart).toISOString() }];
    }
    const head: MessageDescriptor = {
      kind: 'system',
      id: HEAD_ID,
      body: ingest.dropped > 0 ? SESSION_START_DROPPED : SESSION_START,
      timestamp: headTimestamp(),
    };
    if (projected.length === 0) {
      return [
        head,
        { kind: 'system', id: NOMSG_ID, body: NO_MESSAGES, timestamp: new Date(sessionStart + 1).toISOString() },
      ];
    }
    return [head, ...projected];
  });

  // The connection-gap episodes (C-7/C-10) — pass straight into C1's `status` prop (do NOT re-order/re-map).
  const statusArr = $derived(gaps.episodes);

  // Aggregate getter G. `projectedCount` is the REAL message count (synthetic rows excluded) so a verifier
  // never mistakes a composed empty state for phantom messages (§4 measurement precondition, the J-548
  // right-about-the-wrong-quantity family). `streamCount` = what the stream's own `count` reads.
  const debug = () => ({
    latchedRoomId,
    effectiveRoomId,
    projectedCount: projected.length,
    // Split out at Leg D2 so a verifier can tell an echo from an arrival — the two are deliberately
    // indistinguishable in the RENDER (lock #9: an echo is a real message row), which is exactly why the
    // getter must distinguish them.
    inboundCount: inbound.length,
    outboundCount: outbound.length,
    syntheticCount: messages.length - projected.length,
    streamCount: messages.length,
    episodeCount: statusArr.length,
    dropped: ingest.dropped,
    // The `above` socket render truth (Leg C-bis-1): 0 while the socket has no tenant, and a dropped
    // unknown id lowers it — so it is the drop-unknown proof, not merely a declared count.
    aboveMountCount: resolvedAbove.length,
    emptyState: effectiveRoomId == null ? 'no-room' : projected.length === 0 ? 'no-messages' : null,
  });
</script>

<!-- Widget root (the rooms-panel / spaces-panel precedent: `data-tier="widget"` + envelope). MessageStream
  is ALWAYS mounted (no conditional mount → no registry churn); every empty state is a composed `system` row,
  so `core`'s own "No messages yet" is never reached. `onSelect` is deliberately NOT wired to the bus (§3.3). -->
<div class="stream-panel" data-tier="widget" use:envelope={{ name: 'stream-panel', id, debug }}>
  <!-- The `above` socket (Leg C-bis-1): fixed-size (`flex:0 0 auto`), rendered BEFORE the stream. Empty in
    C-bis-1 (0 height); C-bis-2 mounts `dm-intro` here when a draft is active. Unknown ids DROPPED (W-13). -->
  <div class="stream-above">
    {#each resolvedAbove as a (a.key)}
      {@const W = a.component}
      <W id={a.id} {...a.props} />
    {/each}
  </div>
  <!-- The stream fills the rest (`flex:1 1 0; min-height:0`) and is ALWAYS mounted (§198: no conditional
    mount → no registry churn; the registry floor is the proof). -->
  <div class="stream-body">
    <MessageStream messages={messages} status={statusArr} {widgets} id={cid('stream')} />
  </div>
</div>

<style>
  /* Structural only (the fill chain; appearance is Ms Design's, N-090). This is the FIRST region widget that
     must FILL its tile and self-scroll (the others are short cards) — so, like `message-stream` itself, it
     carries a minimal structural <style> (N-094: a per-component structural block is permitted; the question
     is "could a skinner retune this?" — a fill contract is not a look). `height:100%` resolves against the
     flex `.region-tile-body`; `min-height:0` rides every level so the scrollbar stays inside the leaf, never
     migrating to the document (the J-499 D5 failure mode). No skin.css rule was added.
     Leg C-bis-1 makes the panel a FLEX COLUMN so the `above` socket and the stream SHARE the tile rather
     than the socket stealing height (J-704: a naive sibling above a fill-its-tile stream overflows). This
     is still the fill contract, not a look — the socket's fixed sizing and the stream's flex-grow are
     structural (N-094); the socket's CONTENT (dm-intro) is skinned in skin.css, which is Joe's. */
  .stream-panel {
    height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  /* The socket takes exactly the space its tenant needs (0 when empty); the stream fills the rest. */
  .stream-above {
    flex: 0 0 auto;
  }
  .stream-body {
    flex: 1 1 0;
    min-height: 0;
  }
</style>
