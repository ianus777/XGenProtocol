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
  import type { MessageDescriptor } from '$core/components/data-dependent/types';
  import { ingest, type IngestEvent } from '$common/stores/ingest.svelte';
  import { selfState } from '$common/stores/self-state.svelte';
  import { selection } from '$common/stores/selection.svelte';
  import { spacesState } from '$common/stores/spaces-state.svelte';
  import { gaps } from '$common/stores/gaps.svelte';
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
  const SESSION_START = 'Showing messages received since you connected.';
  const SESSION_START_DROPPED = 'Showing messages received since you connected — some were dropped.';
  const NO_MESSAGES = 'No messages in this room yet.';
  const SELECT_ROOM = 'Select a room to see its messages.';

  // Session start — captured once at mount; the head marker anchors here (it precedes the first message).
  const sessionStart = Date.now();

  // ── The room latch (C-5, F-6) — the rooms-panel D3 shape with 'room'. Reads selection, writes the latch,
  // never reads the latch. `onSelect` is NOT wired to selection.set (§3.3) so nothing here moves the bus. ──
  let latchedRoomId = $state<string | null>(null);
  $effect(() => {
    const c = selection.current;
    if (c?.entity.kind === 'room') latchedRoomId = c.entity.id;
  });

  // Stale-latch guard (N-095 spirit): a latched room that no longer resolves in the known-Space tree falls
  // back to the "select a room" state — never throw. (An unregistered client has no rooms → effectiveRoomId
  // stays null → "select a room", honest.)
  const roomKnown = $derived(
    latchedRoomId != null &&
      spacesState.spaces.some((s) => s.rooms.some((r) => r.room_id === latchedRoomId)),
  );
  const effectiveRoomId = $derived(roomKnown ? latchedRoomId : null);

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

  const projected = $derived(
    effectiveRoomId == null
      ? []
      : ingest.events
          .filter((e) => e.room_id === effectiveRoomId)
          .map((e) => projectEvent(e, selfId, redactedIds))
          .filter((m): m is MessageDescriptor => m !== null),
  );

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
    syntheticCount: messages.length - projected.length,
    streamCount: messages.length,
    episodeCount: statusArr.length,
    dropped: ingest.dropped,
    emptyState: effectiveRoomId == null ? 'no-room' : projected.length === 0 ? 'no-messages' : null,
  });
</script>

<!-- Widget root (the rooms-panel / spaces-panel precedent: `data-tier="widget"` + envelope). MessageStream
  is ALWAYS mounted (no conditional mount → no registry churn); every empty state is a composed `system` row,
  so `core`'s own "No messages yet" is never reached. `onSelect` is deliberately NOT wired to the bus (§3.3). -->
<div class="stream-panel" data-tier="widget" use:envelope={{ name: 'stream-panel', id, debug }}>
  <MessageStream messages={messages} status={statusArr} id={cid('stream')} />
</div>

<style>
  /* Structural only (the fill chain; appearance is Ms Design's, N-090). This is the FIRST region widget that
     must FILL its tile and self-scroll (the others are short cards) — so, like `message-stream` itself, it
     carries a minimal structural <style> (N-094: a per-component structural block is permitted; the question
     is "could a skinner retune this?" — a fill contract is not a look). `height:100%` resolves against the
     flex `.region-tile-body`; `min-height:0` rides every level so the scrollbar stays inside the leaf, never
     migrating to the document (the J-499 D5 failure mode). No skin.css rule was added. */
  .stream-panel {
    height: 100%;
    min-height: 0;
  }
</style>
