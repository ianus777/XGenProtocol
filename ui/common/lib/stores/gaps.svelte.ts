// gaps.svelte.ts — the connection-gap episode store (M-RP6.3 Leg C2, C-10). It mints episode identity,
// records each gap's start time, owns the grace timer, derives the four phases, and publishes a
// `StreamStatus[]` for R5 (`stream-panel` → `message-stream`).
//
// WHY A STORE, NOT THE WIDGET (§9.10.2, amends C-7). A widget-local tracker loses all outage history when
// the tile is folded / the layout changes / the plugin is toggled, and restarts the grace timer on remount
// (a still-live outage would blink out and return two seconds later). A second mount (M-RP-SETTINGS'
// `surface: 'window'`) would give two views of one connection two disagreeing stories. So the episodes live
// here, app-lifetime. C-7's binding half — nothing added to `resident.rs`, nothing in `core` — is untouched.
//
// A PURE `$state` STORE, fed by the shell (the self-state / ingest / spaces-state precedent exactly — no
// `.svelte.ts` in this codebase runs its own effect). ONE writer: `apply()`. The shell's reactive `$effect`
// (app_client) calls it, and the DEV hook exposes THAT SAME function — a verify seam that skips the
// mechanism verifies the wrong thing (the J-548 `tickNow` lesson, applied forward). No setter that injects
// a finished episode.
//
// The published type is C1's `StreamStatus`, imported from `core` — NOT redeclared (a second copy is a
// D-067 drift surface). Type-only import, so no layering line is crossed (the `selection.svelte.ts`
// EntityDescriptor precedent).

import type { StreamStatus } from '$core/components/data-dependent/stream/grouping';
import type { Connection, ResidentStatus } from '$common/stores/self-state.svelte';
import { phaseFor } from '$common/components/widgets/stream/derive';

/** The grace window (§9.7, D5 — the ONE place this tunable lives; nothing hardcodes a second copy). A gap
 *  shorter than this is a blip and is NEVER shown — silence is the correct UI for a blip. */
export const GRACE_MS = 2_000;

// The PUBLISHED array (what the widget reads), oldest first. Resolved history + the live gap once past grace.
let _episodes = $state<StreamStatus[]>([]);

// Non-reactive bookkeeping — read/written only inside `apply`, never a widget dependency.
let _lastState: string | null = null; // previous connection state (to detect LEAVING ready)
let _counter = 0; // monotonic, for id minting (start epoch + counter = a stable, mutation-free id)
let _liveId: string | null = null; // the currently-open episode (pending OR published)
let _pending: { ep: StreamStatus; timer: ReturnType<typeof setTimeout> } | null = null; // grace-window holder
let _paused = false; // verify affordance: gate the automatic feed so a controlled test can drive `apply`
                     // in isolation (the `ingest.clear()` precedent). The live path (V11) runs unpaused.

/** Refresh an open episode's phase + poll-sampled numbers from the latest resident. Mutates in place.
 *  `phaseFor` (the phase mapping) is the pure, unit-tested `./stream/derive` module. */
function fillNumbers(ep: StreamStatus, res: ResidentStatus | null): void {
  ep.phase = phaseFor(res);
  ep.attempt = res?.attempt;
  ep.maxAttempts = res?.max_attempts;
  ep.remainingMs = res?.next_attempt_in_ms ?? null;
}

/**
 * The ONE writer. The shell's `$effect` calls it on every `selfState.connection` / `selfState.resident`
 * change; the DEV hook exposes the same function. Episode START is event-timed (the connection listen
 * drives the leave-READY branch), so `timestamp` and `resolvedAfterMs` are accurate; the countdown NUMBERS
 * ride the 2 s resident poll and may be up to ~2 s stale (§3.1 honest-limit 1).
 */
export function apply(conn: Connection, res: ResidentStatus | null, at: number = Date.now()): void {
  const state = conn.state;
  const wasReady = _lastState === 'READY';
  const isReady = state === 'READY';

  if (wasReady && !isReady) {
    // LEAVE READY → open a PENDING episode (not yet published — the grace window). Event-timed start.
    _counter += 1;
    const id = `gap-${at}-${_counter}`;
    const ep: StreamStatus = {
      id,
      phase: phaseFor(res),
      timestamp: at,
      attempt: res?.attempt,
      maxAttempts: res?.max_attempts,
      remainingMs: res?.next_attempt_in_ms ?? null,
    };
    _liveId = id;
    const timer = setTimeout(() => {
      // Grace fired and READY did not return first → publish. (`_pending` guards a discard that already ran.)
      if (_pending && _pending.ep.id === id) {
        _episodes = [..._episodes, _pending.ep];
        _pending = null; // now published; further updates mutate it in `_episodes`
      }
    }, GRACE_MS);
    _pending = { ep, timer };
  } else if (_liveId != null) {
    if (isReady) {
      // RETURN TO READY → resolve (if published) or discard (if still in the grace window — a blip).
      if (_pending && _pending.ep.id === _liveId) {
        clearTimeout(_pending.timer);
        _pending = null; // discarded — never shown
      } else {
        const i = _episodes.findIndex((e) => e.id === _liveId);
        if (i >= 0) {
          const e = _episodes[i];
          e.phase = 'resolved';
          e.remainingMs = null;
          e.resolvedAfterMs = at - e.timestamp; // retrospective, event-timed → accurate
          _episodes = [..._episodes]; // reassign so the widget's $derived re-runs (elements are the same proxies)
        }
      }
      _liveId = null;
    } else {
      // STILL DOWN → refresh the live episode's phase/numbers from the latest poll.
      if (_pending && _pending.ep.id === _liveId) {
        fillNumbers(_pending.ep, res); // pending: update in place (published with the freshest phase when grace fires)
      } else {
        const i = _episodes.findIndex((e) => e.id === _liveId);
        if (i >= 0) {
          fillNumbers(_episodes[i], res);
          _episodes = [..._episodes];
        }
      }
    }
  }
  _lastState = state;
}

export const gaps = {
  /** The published episodes (oldest first) — resolved history plus the live gap once past grace. Reactive. */
  get episodes(): StreamStatus[] {
    return _episodes;
  },
  /** Whether the automatic feed is paused (verify affordance). */
  get paused(): boolean {
    return _paused;
  },
  /** The single writer — same fn the shell's $effect calls (single-writer rule). */
  apply,
  /** Pause the automatic feed so a controlled test can drive `apply` in isolation (V10). Live path runs unpaused. */
  pause(): void {
    _paused = true;
  },
  /** Resume the automatic feed. */
  resume(): void {
    _paused = false;
  },
  /** Test/verify affordance — reset all episode state (the `ingest.clear()` precedent). */
  clear(): void {
    if (_pending) clearTimeout(_pending.timer);
    _pending = null;
    _liveId = null;
    _episodes = [];
    _lastState = null;
  },
};

// DEV-only CDP handle (mirrors __XGEN_INGEST__ / __XGEN_SELF__, N-024) — so the verify pass can drive/read
// the gap store directly. Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_GAPS__?: unknown }).__XGEN_GAPS__ = gaps;
}
