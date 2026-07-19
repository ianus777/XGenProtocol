// ingest.svelte.ts — the live inbound-event store (M-RP6.3 Leg B).
//
// The M-RP6.6 deferral closes here: the resident's drain is the single reader of the socket, and every
// inbound protocol Event it reads is emitted as `xgen-event`. The shell listens ONCE and writes this
// store; R5 (`message-stream`) reads it at Leg C. ONE channel, one source — the
// `xgen-client-state-changed` precedent (D1: no second emit channel, D-067).
//
// A `$common` store because R5 lives in `ui/common/lib/components/widgets/` and W-3 forbids a `common`
// widget importing a shell dep — the leaf mount passes a widget only its `regionId`, so anything a
// widget needs is store-mediated. The self-state / spaces-state precedent exactly.
//
// DELIBERATELY NOT a message projection. This carries protocol Events VERBATIM (snake_case as Rust
// serialises them, no mapping layer — a mapping layer is a drift surface). Turning an Event into a
// `MessageDescriptor` is R5's job at Leg C, and doing it here would put the projection in a second
// place before the first one exists.

/** A protocol Event as it arrives over the `xgen-event` Tauri channel. Only the fields the UI reads are
 *  named; the payload is carried whole, so nothing is lost to an early projection. */
export interface IngestEvent {
  event_id?: string;
  /** The event kind. This is the WIRE name. `wire.rs:476` carries
   *  `#[serde(rename = "type")] pub event_type: EventType` — the RUST field is `event_type`, and it is
   *  named that way only because `type` is a Rust keyword. This interface declared the Rust name until
   *  J-551 and now declares the wire name; do NOT "correct" it back. */
  type?: string;
  sender?: string;
  space_id?: string;
  room_id?: string;
  timestamp?: string;
  content?: Record<string, unknown>;
  prev_events?: string[];
}

/** How many events are retained in memory. A resident runs for days; an unbounded array is a slow leak.
 *  The cap is a NAMED CONSTANT (D5) and the store reports `dropped` so the UI can never quietly imply it
 *  is showing everything when it is not. */
export const INGEST_CAP = 500;

let _events = $state<IngestEvent[]>([]);
let _received = $state(0);
let _dropped = $state(0);

export const ingest = {
  /** The retained events, oldest first. */
  get events(): IngestEvent[] {
    return _events;
  },
  /** Total events observed this run — including any the cap dropped. */
  get received(): number {
    return _received;
  },
  /** How many were evicted by the cap. Non-zero ⇒ `events` is a window, not the whole run. */
  get dropped(): number {
    return _dropped;
  },
  /** The most recent event, or null before anything arrives. */
  get latest(): IngestEvent | null {
    return _events.length ? _events[_events.length - 1] : null;
  },
  /** Written by the shell from the `xgen-event` listen. */
  push(ev: IngestEvent): void {
    _received += 1;
    const next = [..._events, ev];
    if (next.length > INGEST_CAP) {
      _dropped += next.length - INGEST_CAP;
      next.splice(0, next.length - INGEST_CAP);
    }
    _events = next;
  },
  /** Test/verify affordance — clears the window without pretending the run restarted. */
  clear(): void {
    _events = [];
  },
};

// DEV-only CDP handle (mirrors __XGEN_SELF__ / __XGEN_SEL__, N-024) so the verify pass can read the live
// ingest without a rendered surface — Leg B ships the SEAM, and R5 renders it at Leg C.
// Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_INGEST__?: unknown }).__XGEN_INGEST__ = ingest;
}
