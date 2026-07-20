// room-latch.svelte.ts — THE SHARED ROOM LATCH (M-RP6.3 Leg D2, §2.1 / lock #12).
//
// 🔑 WHY THIS STORE EXISTS, AND WHY IT IS A LIFT RATHER THAN A SECOND COPY.
//
// R5 (`stream-panel`) has latched the last `kind: 'room'` bus selection since Leg C2. R6 (the composer) needs
// THE SAME ROOM — and the obvious alternative, reading `selection.current` directly, was recommended by Chat
// and REVERSED by the user-impact question (D-121, J-559):
//
//   Reading the bus:  you are reading room X. You click a Space in the tree. The stream keeps showing room X
//                     — it is latched. THE COMPOSER GREYS OUT. You are looking at a conversation you can no
//                     longer type into, on ordinary navigation, with nothing on screen explaining why.
//   Sharing the latch: the composer is enabled exactly when the stream is showing a room, and disabled
//                     exactly when the stream says "select a room". THE MESSAGE ALWAYS GOES TO THE
//                     CONVERSATION YOU ARE LOOKING AT.
//
// ⚠️ THERE ARE TWO LATCHES AND THE NAME HIDES IT. `rooms-panel.svelte:23` latches the SPACE
// (`latchedSpaceId`); the one lifted here is R5's ROOM latch. "The latch" means whichever one you happen to
// be reading. R2's Space latch is NOT touched by this store and must not be folded into it.
//
// REJECTED: duplicating the latch logic inside the composer. It avoids editing a shipped component, at the
// price of two copies of a rule that must agree forever — a D-067 drift surface whose divergence produces
// EXACTLY the greyed-out-composer failure this decision exists to prevent.
//
// APP-LIFETIME, NOT WIDGET-LIFETIME. R5 and R6 are separate region widgets in separate tiles; either can be
// folded or unmounted. A latch owned by R5 would vanish the moment R5 was folded, taking the composer's
// target with it. Same argument as the gaps store (C-10).
//
// A PURE `$state` STORE FED BY THE SHELL — no `.svelte.ts` in this codebase runs its own effect. `note()` is
// the SINGLE WRITER; the shell's `$effect` calls it and the DEV hook exposes THAT SAME function (the J-548
// `tickNow` lesson: a verify seam that skips the mechanism verifies the wrong thing).
//
// ⚠️ `note()` READS ITS ARGUMENT AND WRITES `_latched`; it never READS `_latched`. That is what keeps it free
// of the self-invalidating read-modify-write that N-136 records (and it is why the rooms-panel comment says
// the same thing about its own latch).

import { selection, type Selection } from '$common/stores/selection.svelte';
import { spacesState } from '$common/stores/spaces-state.svelte';

let _latched = $state<string | null>(null);

/** Resolve the latched room in the known-Space tree, returning both ids. Null when nothing is latched or the
 *  latched room no longer resolves (stale-latch guard, N-095 spirit: fall back, never throw). */
function resolveLatched(): { spaceId: string; roomId: string } | null {
  const id = _latched;
  if (id == null) return null;
  for (const s of spacesState.spaces) {
    if (s.rooms.some((r) => r.room_id === id)) return { spaceId: s.space_id, roomId: id };
  }
  return null;
}

export const roomLatch = {
  /** The raw latched id — what was last selected, resolvable or not (verify/debug surface). */
  get latchedRoomId(): string | null {
    return _latched;
  },
  /**
   * The room BOTH R5 and R6 act on: the latched room iff it still resolves in the known-Space tree,
   * otherwise null. An unregistered client has no rooms, so this stays null and both widgets show their
   * honest "select a room" state.
   */
  get effectiveRoomId(): string | null {
    return resolveLatched()?.roomId ?? null;
  },
  /**
   * The Space owning `effectiveRoomId`. R5 never needed this; the composer does — `send_message` takes
   * (spaceId, roomId, text). Derived from the same single resolution, so the pair can never disagree.
   */
  get effectiveSpaceId(): string | null {
    return resolveLatched()?.spaceId ?? null;
  },
  /** 🔒 Lock #12's mechanism: no room latched ⇒ typing yes, SENDING NO. One predicate, both widgets. */
  get canSend(): boolean {
    return resolveLatched() != null;
  },
  /** THE SINGLE WRITER — same fn the shell's `$effect` calls and the DEV hook exposes. Writes only on a
   *  `room` selection, so a later Space (or message) selection KEEPS the room, which is the whole point. */
  note(sel: Selection | null = selection.current): void {
    if (sel?.entity.kind === 'room') _latched = sel.entity.id;
  },
  /** Test/verify affordance — forget the latch (the ingest.clear / gaps.clear precedent). */
  clear(): void {
    _latched = null;
  },
};

// DEV-only CDP handle (N-024 idiom). Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_ROOM__?: unknown }).__XGEN_ROOM__ = roomLatch;
}
