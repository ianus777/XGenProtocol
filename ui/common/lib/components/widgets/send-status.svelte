<script lang="ts">
  // send-status — M-RP6.3 Leg D3. The per-row send indicator, and the SECOND tenant of the `bodyExtras`
  // socket M-RP6.9 built. A `$common` widget: it reads the `$common` echo store directly by `localId`
  // (a `WidgetMount.props` value), so nothing has to pass a function through a props bag.
  //
  // 🔒 IT SHIPS WITH D2 AND THAT IS A LOCK, NOT A PREFERENCE (§3). Without it, an echo row looks EXACTLY
  // like a delivered message — same component, same descriptor shape, same styling — and it may have been
  // `rejected`, refused by the node, never arriving, ever. ⚠️ AND IT NEVER SELF-CORRECTS: the node excludes
  // the author from fan-out by identity, so the message never returns from the server. There is no later
  // moment when the truth arrives and the row updates. The echo is the ONLY record the message exists, and
  // it asserts success purely by being there.
  //
  // WHY `bodyExtras` AND NOT `details` (§9.11.5, M-RP6.9 D-5): a grouped continuation row suppresses the
  // WHOLE header line — name AND `details`. Three sends in a row and a `failed` third would look identical
  // to a delivered first, in the most common send pattern there is. `bodyExtras` sits below the body,
  // OUTSIDE the header guard, so it is GROUPING-IMMUNE BY POSITION.
  //
  // 🔒 THREE VISUAL STATES, NOT TWO (lock #6) — plus `pending`, which is the state BEFORE an outcome, not a
  // fourth outcome (§9.11.1):
  //     accepted           -> sent
  //     timed_out          -> UNRESOLVED  (its own state: the node MAY hold it)
  //     rejected + failed  -> not sent    (one state, DIFFERENT copy — the causes are not the same)
  // Collapsing `timed_out` into either neighbour is the D6 lie verbatim, so it gets its own state and its
  // own words.
  //
  // 🔒 RETRY BY STATUS (lock #7, NARROWED at §3.1): `failed` retries freely — it never reached the wire, so
  // a retry cannot duplicate anything. `rejected` gets NO retry: it will be refused again. `timed_out` gets
  // NO RETRY AFFORDANCE AT ALL — it is the ONLY status where one click can place a SECOND COPY of the same
  // sentence on the federated network, permanently, attributed to the user's identity, because the node may
  // already be holding the first. The store enforces this too; a refusal that lives only in a hidden button
  // is one stray caller away from not existing.
  import { envelope } from '$common/components/base/envelope';
  import { echo, toneOf, isRetryable, type SendStatus } from '$common/stores/echo-state.svelte';

  // ⚠️ `localId` IS OPTIONAL, AND THE TYPE GATE IS WHY — it caught this on the first run of the milestone.
  // The socket types its registry as `Record<string, Component>`, i.e. components taking NO props, while
  // `WidgetMount.props` is `Record<string, unknown>` — so NOTHING type-checks that a mount supplies this
  // value. Declaring it required was a promise the socket cannot enforce; declaring it optional states the
  // truth and makes the "Unknown" render below a REACHABLE branch rather than dead code (N-091).
  // (The prop-less registry type is a `core` defect of the same shape M-RP-TYPECHECK fixed in registry.ts.
  // Fixing it is a `core` change and is FLAGGED, not folded into a `$common` milestone.)
  let { localId = '', id }: { localId?: string; id?: string } = $props();

  const row = $derived(localId ? echo.byLocalId(localId) : null);
  const status = $derived<SendStatus | null>(row?.status ?? null);

  // Copy is FUNCTIONAL and PROVISIONAL (appearance and final phrasing -> M-RP-SKIN, the stream-panel
  // `SESSION_START` precedent). The MEANING is locked: each line says what is true, and `timed_out` never
  // borrows either neighbour's words.
  const LABEL: Record<SendStatus, string> = {
    pending: 'Sending…',
    accepted: 'Sent',
    timed_out: 'Unresolved — the node may have it',
    rejected: 'Not sent — refused',
    failed: 'Not sent',
  };

  // The three-state grouping the skin keys on (`pending` renders as its own quiet fourth) — the pure,
  // unit-tested rule, not a local copy.
  const tone = $derived(status ? toneOf(status) : 'pending');

  // Retry is offered for `failed` ONLY (§3.1) — and this is THE SAME predicate `echo.retry` enforces, so
  // the visible affordance and the committed action cannot drift apart (N-126).
  const canRetry = $derived(status != null && isRetryable(status));

  // The refusal cause, when the node gave one. Shown for `rejected`/`failed` so "not sent" is never a bare
  // assertion the user cannot act on.
  const detail = $derived(
    row && (row.status === 'rejected' || row.status === 'failed')
      ? [row.code != null ? `#${row.code}` : null, row.reason ?? null].filter(Boolean).join(' ')
      : '',
  );

  function onRetry(): void {
    if (localId) void echo.retry(localId);
  }

  const debug = () => ({
    localId,
    status,
    tone,
    canRetry,
    hasDetail: detail !== '',
    eventId: row?.eventId ?? null,
  });
</script>

<!-- Root is ALWAYS mounted while the row exists, so a status is never silently absent; an unresolvable
  `localId` renders the honest "unknown" state rather than nothing (N-091: an empty render and a missing
  widget are indistinguishable on screen). -->
<span class="send-status" data-tone={tone} use:envelope={{ name: 'send-status', id, debug }}>
  <span class="send-status-dot" aria-hidden="true"></span>
  <span class="send-status-label">{status ? LABEL[status] : 'Unknown'}</span>
  {#if detail}<span class="send-status-detail">{detail}</span>{/if}
  {#if canRetry}
    <button class="send-status-retry" type="button" onclick={onRetry}>Retry</button>
  {/if}
</span>
