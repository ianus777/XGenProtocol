// dm-draft.svelte.ts — THE DM DRAFT RENDER STATE (M-RP-MEMBER-ACT Leg C-bis-2, §5.1 / runbook v1.3 §3).
//
// 🔑 A SIBLING to `roomLatch`, NOT a third state inside it. `roomLatch`'s header (room-latch.svelte.ts:5-18)
// declares "one predicate, both widgets" — a single latched room drives BOTH R5's render and R6's `canSend`.
// A state meaning "no room, but pretend" would make `canSend` lie, the exact failure `roomLatch` prevents.
// So the draft lives in its OWN store — and (v1.3) it does NOT touch `roomLatch` at all: opening a draft
// leaves the latch where it is, so R7 keeps its scope, roster and fill (§5.1 holds LITERALLY).
//
// 🛑 THE KICKOFF'S R-1/R-2 WERE WITHDRAWN AFTER BEING DRIVEN (2026-08-09). R-2 derived `active` off the latch,
// which forced `roomLatch.clear()` on open — and `members-panel.svelte:57` reads `roomLatch.effectiveSpaceId`
// as its authoritative scope, so the clear EMPTIED R7 to `no-scope` and (app_client.svelte:218) tore down the
// members fill on every open/close. R-1's send-safety was a duplicated guarantee: C-bis-3 puts the draft
// branch ABOVE the composer's early return, so a draft send never consults `roomLatch` at all.
//
// 🔒 `active` IS STORED, NOT DERIVED (v1.3): `active = counterpart != null`. The draft CLOSES on a `room`
// selection via `note(sel)`, called by the shell's existing bus→latch effect (app_client.svelte:202-208 —
// "ONE effect, TWO latches", now three: `dmDraft.note` joins `roomLatch.note` / `spaceLatch.note`, the idiom
// `spaceLatch.note` used when it landed at M-RP-SELECT-ORIENT C-1). `note` READS its argument and never
// `_counterpart` — no self-invalidating read-modify-write (N-136). ⚠️ Any `room` selection closes it,
// including re-selecting the room you were already in, which a latch-value comparison would have missed.
//
// TEXT IS KEYED BY COUNTERPART (§5.3): switching drafts preserves each one's typed text. The map survives
// navigation by construction, and `note()` clears only the OPEN counterpart, NEVER the text map, so
// re-opening a draft restores its text. NO PERSISTENCE (J-598, Joe): the client holds no user data; the map
// dies with the session.
//
// A PURE `$state` STORE (the roomLatch / spaces-state idiom): module-level `$state`, an object literal of
// getters + writers, no self-run effect.
//
// ⚠️ LEG SCOPE: `open` / `note` / `active` / `counterpart` / `setText` / `text` are exercised by C-bis-2/3
// (members-panel opens; the shell effect closes on a room; stream-panel reads active + counterpart; the
// composer routes its text). C-bis-4 wires the last three — the injected `create` transport, `error` (the
// failure surface, §5.4), and `clear` (called by the composer after a successful send). The C-bis-1
// "plumbing before tenant" precedent: the shape landed in step 1, the consumers arrive per-leg.
//
// THE CREATE SEAM IS INJECTED (the echo `SendTransport` precedent). `create_dm_space` is a Tauri command and
// there are ZERO `@tauri-apps` imports under `ui/common` (W-3), so this module never learns Tauri exists —
// the shell supplies a `create_dm_space`-backed function at boot. The verb SIGNS AND SENDS a three-event
// chain and needs a LIVE NODE, so unlike the on-disk reads it CAN fail; `create` returns `null` on failure
// and parks the cause in `error`, leaving the draft open with its text (§5.4, D-065).

import { selection, type Selection } from '$common/stores/selection.svelte';

/** The counterpart being drafted, or `null` when no draft is open. `active` is exactly `!== null`. */
let _counterpart = $state<string | null>(null);
/** Typed text, keyed by counterpart id (§5.3) — switching between drafts keeps each one. */
let _texts = $state<Record<string, string>>({});
/** The last create failure, or `null` (§5.4). A single readable field — the VISIBLE surface Joe rules on
 *  later. C-bis-4 SETS it and leaves it UN-RENDERED (behind this one call site); nothing paints it yet. */
let _error = $state<string | null>(null);

/** The Rust `CreateDmSpaceResult` (`xgen-client/src/ops.rs:827`) carried VERBATIM — the flavour fields are
 *  `#[serde(transparent)]`, so they cross the Tauri boundary as bare strings and there is no mapping layer
 *  (the `SendOutcome` / `KnownSpace` precedent; a mapping layer is a D-067 drift surface). */
export interface CreateDmSpaceResult {
  space_id: string;
  room_id: string;
  event_id: string;
  invitee: string;
  owner_identity_id: string;
}

/** The injected create seam. The shell supplies a `create_dm_space`-backed function (W-3: this module never
 *  imports Tauri). It REJECTS on a node-side failure — the verb signs and sends and needs a live node. */
export type CreateDmTransport = (invitee: string) => Promise<CreateDmSpaceResult>;

let _createTransport: CreateDmTransport | null = null;

export const dmDraft = {
  /** 🔒 STORED, not derived (v1.3): a draft is active iff a counterpart is set. Read by R5 (stream-panel) to
   *  gate the intro mount and swap the stream to empty, and by R6 (composer, C-bis-3) to enable sending. */
  get active(): boolean {
    return _counterpart !== null;
  },
  /** The id of the identity being drafted, or `null`. R5 resolves its display label from this. */
  get counterpart(): string | null {
    return _counterpart;
  },
  /** The typed text for the currently-open draft (`''` with no draft, or none typed yet). Consumed by the
   *  composer (C-bis-3) and the send sequence (C-bis-4); nothing writes it in C-bis-2. */
  get text(): string {
    return _counterpart != null ? (_texts[_counterpart] ?? '') : '';
  },
  /** The last create failure, or `null` (§5.4). The single call site a future VISIBLE surface reads; C-bis-4
   *  populates it and renders nothing (the surface is Joe's to rule on). Cleared on `open` and each `create`. */
  get error(): string | null {
    return _error;
  },
  /** Open (or re-open) a draft for `identityId`. Idempotent — reopening restores any text already typed for
   *  them (§5.3), since `note()` preserved the text map. Clears any stale create error (fresh context). */
  open(identityId: string): void {
    _error = null;
    _counterpart = identityId;
  },
  /** THE BUS-FED CLOSER (v1.3) — the shell's bus→latch effect calls this alongside `roomLatch.note` /
   *  `spaceLatch.note`. A `room` selection CLOSES the draft (the user navigated to a conversation); the text
   *  map is untouched, so re-opening the draft restores its text. An `identity` selection — including the
   *  click that OPENED the draft and wrote the person to the bus (L-7 → R8) — is IGNORED, so opening a draft
   *  and lighting R8 do not fight. Reads its argument, never `_counterpart` (N-136). */
  note(sel: Selection | null = selection.current): void {
    if (sel?.entity.kind === 'room') _counterpart = null;
  },
  /** Update the typed text for the currently-open draft (C-bis-3, the composer). No-op with no draft open.
   *  Reassigns a fresh object (Svelte 5 `$state` tracks the reference). */
  setText(text: string): void {
    if (_counterpart == null) return;
    _texts = { ..._texts, [_counterpart]: text };
  },
  /** The shell injects the `create_dm_space`-backed transport at boot (W-3: this module imports no Tauri).
   *  Same seam shape as `echo.setTransport` — the store stays transport-agnostic and unit-testable. */
  setCreateTransport(t: CreateDmTransport | null): void {
    _createTransport = t;
  },
  /**
   * Run the create for `invitee` (C-bis-4, §5.2). Returns the result on success, or `null` on failure — in
   * which case the draft is LEFT OPEN with its text UNTOUCHED (§5.4, D-065) and `error` carries the cause.
   * The verb signs and sends three events to a live node and CAN fail; the caller latches / sends only on a
   * non-null result, so a failed create latches nothing and NOTHING on screen implies the DM exists. The
   * caller passes the counterpart (captured at click time) rather than reading `_counterpart`, so a
   * navigation mid-create cannot swap the invitee out from under the send.
   */
  async create(invitee: string): Promise<CreateDmSpaceResult | null> {
    _error = null;
    if (!_createTransport) {
      _error = 'no create transport';
      return null;
    }
    try {
      return await _createTransport(invitee);
    } catch (e) {
      _error = String(e);
      return null;
    }
  },
  /** Close the draft after a successful send (C-bis-4). Drops the counterpart AND its text — the text was
   *  sent, and the real DM now takes over (create → latch → the shipped stream). */
  clear(): void {
    if (_counterpart != null) {
      const { [_counterpart]: _drop, ..._rest } = _texts;
      _texts = _rest;
    }
    _counterpart = null;
  },
};

// DEV-only CDP handle (the __XGEN_ROOM__ / __XGEN_MEMBERS__ idiom, N-024) so the verify pass can read the
// draft state directly. Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_DRAFT__?: unknown }).__XGEN_DRAFT__ = dmDraft;
}
