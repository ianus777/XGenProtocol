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
// ⚠️ LEG SCOPE (C-bis-2, reported): `open` / `note` / `active` / `counterpart` are exercised THIS leg
// (members-panel opens; the shell effect closes on a room; stream-panel reads active + counterpart to gate
// the intro and swap the stream to empty). `text` / `setText` / `clear` complete the store shape runbook
// step 1 declares, and their consumers land later within this milestone — the composer wires `setText`/`text`
// (C-bis-3), the send sequence calls `clear` (C-bis-4). The C-bis-1 "plumbing before tenant" precedent.

import { selection, type Selection } from '$common/stores/selection.svelte';

/** The counterpart being drafted, or `null` when no draft is open. `active` is exactly `!== null`. */
let _counterpart = $state<string | null>(null);
/** Typed text, keyed by counterpart id (§5.3) — switching between drafts keeps each one. */
let _texts = $state<Record<string, string>>({});

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
  /** Open (or re-open) a draft for `identityId`. Idempotent — reopening restores any text already typed for
   *  them (§5.3), since `note()` preserved the text map. */
  open(identityId: string): void {
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
