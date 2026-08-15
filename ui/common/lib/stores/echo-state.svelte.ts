// echo-state.svelte.ts — the OUTBOUND ECHO STORE (M-RP6.3 Leg D2, §9.11.2 / locks #1–#4, #7, #8, #11).
//
// WHY IT EXISTS AT ALL. The node excludes the author from fan-out BY IDENTITY, not by connection
// (`fanout.rs:303`, EV-D2). Combined with C-4 (`stream-panel` projects ONLY `ingest`, on read), a send
// round-trips, returns `accepted`, and THE STREAM DOES NOT CHANGE. Your own words render nowhere.
//
// ⚠️ AND THE OBVIOUS FIX DOES NOT WORK. The brief proposed "a mirror with a defined death — dropped when
// the real event arrives". THE REAL EVENT NEVER ARRIVES. There is no server-sent fact to reconcile your own
// messages against, ever. So this is NOT a placeholder awaiting confirmation: for the whole session it is
// THE ONLY RECORD THOSE MESSAGES EXIST.
//
// C-4 IS AMENDED NARROWLY, NOT REPEALED (§9.11.2): C-4 still governs INBOUND unchanged (project on read, no
// mirror, no reconciliation). OUTBOUND gets this separate, explicitly SESSION-MORTAL, NEVER-FEDERATED store.
// There is still exactly one projection path per source of truth. The awkwardness — a mirror that never dies
// within a session — is a DOCUMENTED property, not a leak.
//
// A `$common` store, not widget state (lock #2), and the argument is stronger than C-10's: a lost outage row
// is an omission; A LOST SENTENCE YOU JUST TYPED IS THE APP EATING YOUR WORDS IN FRONT OF YOU. R5 (which
// renders echoes) and R6 (which creates them) are SEPARATE region widgets in SEPARATE tiles — either can be
// folded, moved or unmounted independently — so widget-local state would lose the record on a layout change.
// Lock #11 (N windows, one device) falls out of the same placement for free.
//
// 🔒 SESSION-MORTAL — THE ONE STATED MOMENT OF DEATH (lock #8): module state, wiped by reload / app exit.
// It is NEVER persisted (no uistate key, no disk, no `session.*`) and NEVER federated. That is deliberate,
// not an omission: a persisted echo would outlive the outcome that gave it meaning and would assert, after a
// relaunch, that a message exists which nothing on the network has. ⚠️ The C-6 head marker must therefore
// cover the user's OWN sends too — they never read the messages they lost; they wrote the one they lost.
//
// A PURE `$state` STORE with an INJECTED transport (the gaps / self-state precedent: no `.svelte.ts` in this
// codebase runs its own effect, and W-3 forbids `$common` importing a shell dep — there are ZERO
// `@tauri-apps` imports under `ui/common`, verified). The shell injects `sendMessage` at boot; this module
// never learns that Tauri exists.

/** The Rust `SendOutcome` (resident.rs:733) carried VERBATIM — snake_case, no mapping layer (the
 *  ConnTraffic / ResidentStatus precedent; a mapping layer is a drift surface, D-067). `status` is typed as
 *  the bare wire string because that is what crosses the boundary; narrowing happens once, below. */
export interface SendOutcome {
  event_id: string | null;
  status: string;
  code: number | null;
  reason: string | null;
}

// The pure status rules live in `./echo-status` (a plain `.ts`) so they can be unit-tested — the node
// vitest harness cannot load a `.svelte.ts`. That is the derive.ts / mounts.ts / grouping.ts precedent, and
// it matters here because those three functions encode locks #6 and #7.
//
// `accepted` (node validated + durably persisted) · `rejected` (deterministic refusal carrying a wire
// `code` — a message that will NEVER arrive, no matter how long the user waits) · `timed_out` (no
// correlated signal, GENUINELY AMBIGUOUS, the node may hold it) · `failed` (never reached the wire).
// Collapsing `timed_out` into either success or failure is precisely the lie D6 forbids.
import { narrowStatus, isRetryable, type SendStatus } from './echo-status';
// Re-exported so a consumer imports the status vocabulary from the store it is reading, not from two places.
export { narrowStatus, toneOf, isRetryable } from './echo-status';
export type { SendStatus } from './echo-status';

// M-RP-INTRO Leg 2a — the intro payload's shape, TYPE-ONLY (erased at build), so this store stays free of
// runtime imports. One declaration for both directions lives beside the key and the validator (`derive.ts`).
import type { MessageIntro } from '$common/components/widgets/stream/derive';

/** The injected send seam. The shell supplies a `send_message`-backed function; this module stays
 *  transport-agnostic and therefore unit-testable and W-3-clean.
 *  M-RP-INTRO Leg 2a widens it with a TRAILING OPTIONAL `intro`, so every existing implementation and every
 *  existing call keeps its exact current shape — including the DEV bridge's zero-arg stub. */
export type SendTransport = (
  spaceId: string,
  roomId: string,
  text: string,
  intro?: MessageIntro | null,
) => Promise<SendOutcome>;

/**
 * ONE outbound message as this client knows it.
 *
 * 🔑 KEYED BY A CLIENT-MINTED `localId`, NOT BY `event_id` (lock #3, §9.11.1). `send_message` awaits the
 * correlated reply and resolves ONCE, so the real `event_id` arrives WITH the outcome. At the moment the
 * user presses Enter the frontend does not know it — an echo keyed by `event_id` is not constructible.
 * `eventId` is STITCHED ON when the outcome returns, onto the SAME row.
 *
 * `sentAt` is CLIENT-MINTED AND STAYS THAT WAY (lock #4). `SendOutcome` carries no timestamp, so even on
 * `accepted` there is nothing to correct it to. FILED (§12): own rows may therefore order differently for
 * the author than for everyone else in the room. Real, permanent within a session, not Leg D's to fix.
 */
export interface EchoMessage {
  localId: string;
  spaceId: string;
  roomId: string;
  text: string;
  sentAt: number;
  status: SendStatus;
  /** Stitched at outcome (lock #3). Absent while `pending`, and absent forever on `failed` (nothing was
   *  ever written, so there is no id to carry) — absent-not-empty-string, the absent-not-zero discipline. */
  eventId?: string;
  /** The wire refusal code — `rejected` only. */
  code?: number;
  /** Human-readable cause — `rejected` / `failed`. */
  reason?: string;
  /** M-RP-INTRO Leg 2a / §4.5 — the intro this send carried, when it carried one.
   *
   *  🔑 IT IS STORED, NOT ONLY PASSED, AND THAT IS THE WHOLE OF §4.5. Own rows never reach `projectEvent`
   *  (`stream-panel.svelte:118-133` maps echoes through `echoToDescriptor` instead), so without a copy HERE
   *  the SENDER never sees their own intro — and `I2`'s "symmetry is free" was ARGUED on the initiator
   *  seeing their intro as message one.
   *
   *  ABSENT-NOT-EMPTY, the `eventId` discipline two fields up: an ordinary send has no `intro` key at all,
   *  never an empty object (`N-182`). A retry re-reads this row, so a retried intro rides along unchanged. */
  intro?: MessageIntro;
}

let _echoes = $state<EchoMessage[]>([]);
let _transport: SendTransport | null = null;
let _counter = 0; // monotonic; epoch + counter = a stable, collision-free local id (the gaps id shape)

/** Mutate one row in place and reassign, so `$derived` consumers re-run (the gaps `_episodes` shape). */
function patch(localId: string, apply: (e: EchoMessage) => void): void {
  const i = _echoes.findIndex((e) => e.localId === localId);
  if (i < 0) return;
  apply(_echoes[i]);
  _echoes = [..._echoes];
}

/** Stitch an outcome onto an existing row (lock #3). The SINGLE place a status leaves `pending`. */
function resolve(localId: string, outcome: SendOutcome): SendStatus {
  const status = narrowStatus(outcome.status);
  patch(localId, (e) => {
    e.status = status;
    // `event_id` is present on accepted / rejected / timed_out; null on failed. Absent stays ABSENT.
    if (outcome.event_id) e.eventId = outcome.event_id;
    if (outcome.code != null) e.code = outcome.code;
    if (outcome.reason) e.reason = outcome.reason;
  });
  return status;
}

/** Drive one row through the transport. Shared by `send` (new row) and `retry` (existing row). */
async function dispatch(e: EchoMessage): Promise<SendStatus> {
  if (!_transport) {
    // No transport injected (browser dev preview, or a boot-order fault). NEVER silently succeed.
    return resolve(e.localId, {
      event_id: null,
      status: 'failed',
      code: null,
      reason: 'no send transport',
    });
  }
  try {
    // The intro rides FROM THE ROW, not from the caller — so a `retry` replays exactly what the first
    // attempt carried, and a row without one passes `undefined` (no key on the wire, `N-182`).
    return resolve(e.localId, await _transport(e.spaceId, e.roomId, e.text, e.intro));
  } catch (err) {
    // The transport itself threw — the call never completed, so nothing reached the wire.
    return resolve(e.localId, { event_id: null, status: 'failed', code: null, reason: String(err) });
  }
}

export const echo = {
  /** Every echo this session, oldest first. Reactive. */
  get all(): EchoMessage[] {
    return _echoes;
  },
  /** Whether a transport has been injected (verify affordance — distinguishes "offline" from "unwired"). */
  get wired(): boolean {
    return _transport != null;
  },
  /** Echoes for ONE room, oldest first — what R5 merges into its projection (lock #9). */
  forRoom(roomId: string | null): EchoMessage[] {
    if (roomId == null) return [];
    return _echoes.filter((e) => e.roomId === roomId);
  },
  /** Look one row up (the send-status widget reads its own row by id). */
  byLocalId(localId: string): EchoMessage | null {
    return _echoes.find((e) => e.localId === localId) ?? null;
  },
  /** The shell injects the `send_message`-backed transport at boot (W-3: this module never imports Tauri). */
  setTransport(t: SendTransport | null): void {
    _transport = t;
  },
  /**
   * THE ONE WRITER. Mints the local id, appends a `pending` row IMMEDIATELY (so the sentence is on screen
   * before the network is consulted), then stitches the outcome onto that same row.
   *
   * Returns the final status so a caller can react; the RENDER never depends on the return value — it reads
   * the store, which is what makes an unawaited call safe.
   *
   * M-RP-INTRO Leg 2a — `intro` is a TRAILING OPTIONAL, appended AFTER `at` rather than before it. `at` is
   * lock #4's client-minted timestamp and reordering a documented parameter to save one explicit argument at
   * the single call site that needs both is the kind of silent meaning-change that bites later.
   * 🛑 `text` IS UNCHANGED AND STILL REQUIRED (1-bis): the intro is composed independently and never
   * populates, replaces or derives the sentence.
   */
  async send(
    spaceId: string,
    roomId: string,
    text: string,
    at: number = Date.now(),
    intro?: MessageIntro | null,
  ): Promise<SendStatus> {
    _counter += 1;
    const row: EchoMessage = {
      localId: `echo-${at}-${_counter}`,
      spaceId,
      roomId,
      text,
      sentAt: at,
      status: 'pending',
    };
    // Absent-not-null: the key is written ONLY when there is an intro, so an ordinary send's echo record is
    // byte-identical to today's and `echoToDescriptor` (§4.5) has nothing to mount.
    if (intro) row.intro = intro;
    _echoes = [..._echoes, row];
    return dispatch(row);
  },
  /**
   * Retry ONE row. 🔒 `failed` ONLY (lock #7 as narrowed at §3.1).
   *
   * ⚠️ THE GUARD IS HERE, NOT ONLY IN THE UI, AND THAT IS THE POINT. `rejected` will be refused again;
   * `timed_out` is the ONLY status where a retry can put a SECOND COPY of the same sentence on the federated
   * network, permanently, attributed to the user's identity — because the node may already be holding the
   * first. A refusal that lives only in a disabled button is one stray caller away from not existing
   * (the M-RP7.6 handler-refusal lesson: the layer a bridge cannot walk around).
   */
  async retry(localId: string): Promise<SendStatus | null> {
    const row = _echoes.find((e) => e.localId === localId);
    // THE SAME predicate the retry button reads (N-126) — not a second comparison that must agree with it.
    if (!row || !isRetryable(row.status)) return null;
    patch(localId, (e) => {
      e.status = 'pending';
      e.reason = undefined;
      e.code = undefined;
    });
    const fresh = _echoes.find((e) => e.localId === localId);
    return fresh ? dispatch(fresh) : null;
  },
  /** Test/verify affordance — drop every echo without pretending the session restarted (ingest.clear precedent). */
  clear(): void {
    _echoes = [];
  },
};

// DEV-only CDP handle (mirrors __XGEN_INGEST__ / __XGEN_GAPS__, N-024) so the verify pass can read and drive
// the echo store directly. Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_ECHO__?: unknown }).__XGEN_ECHO__ = echo;
}
