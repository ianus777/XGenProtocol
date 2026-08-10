// space-latch.svelte.ts — THE SHARED SPACE LATCH (M-RP-SELECT-ORIENT C-1, OQ-1 = option A).
//
// 🔑 WHY THIS STORE EXISTS, AND WHY IT IS A LIFT RATHER THAN A SECOND COPY.
//
// R2 (`rooms-panel`) has always latched the last `kind: 'space'` bus selection, to KEEP its data scope while
// its own room-activation moves the bus to a `kind: 'room'`. Now R1 (`spaces-panel`) needs THE SAME Space —
// so selecting a room KEEPS the browsed Space lit (D4 opt-1 -> opt-2, D-146). If R1 read the bus for its
// highlight, a room selection would blank R1's highlight while you are still looking at that Space's rooms;
// if R1 grew its own private copy, two latches would have to agree forever — the D-067 drift the roomLatch
// header rejects by name.
//
//   R1 reads the LIFTED latch:  select a room -> the Space you are browsing stays lit; R1 and R2 always agree
//                               because they read ONE value. The behaviour Joe asked for ("we will be able to
//                               orient ourselves").
//
// ⚠️ THIS IS NOT `roomLatch.effectiveSpaceId` (option B, REFUSED). That is the Space you are TALKING IN; Joe
// asked for the Space you are BROWSING. They diverge the moment you click a different Space while latched to a
// room. The write rule below is SPACE-SELECTION ONLY — no room-resolves-to-Space arm (Phase-0 §5).
//
// ⚠️ THERE ARE TWO LATCHES AND THE NAMES HIDE IT. `roomLatch` latches the last `kind: 'room'` selection; this
// one latches the last `kind: 'space'` selection. R2's Space latch (formerly private to `rooms-panel`) is
// what lifted here; roomLatch is NOT touched by this store and must not be folded into it (its own header
// forbids the reverse, G6).
//
// APP-LIFETIME, NOT WIDGET-LIFETIME. R1 and R2 are separate region widgets in separate tiles; either can be
// folded or unmounted. When R2's latch lived in `rooms-panel` as component `$state`, folding R2 DESTROYED it:
// on unfold R2 read "Select a space" while roomLatch still targeted the room (measured, Clair's F2). An
// app-lifetime store survives the unmount — the same argument `room-latch.svelte.ts:24` makes for R5/R6.
//
// A PURE `$state` STORE FED BY THE SHELL — no `.svelte.ts` in this codebase runs its own effect. THREE
// functions write `_latched`: `note()` (bus-fed — the shell's `$effect` calls it, the same effect that drives
// `roomLatch.note`), `clear()` (the test/verify reset), and `latch()` (the DIRECT writer, added at C-bis-6 for
// the DM-open paths). 📌 The pre-C-bis claim that `note()` was "THE SINGLE WRITER" was ALREADY false — `clear()`
// has always written `_latched` too; C-bis-6 corrects a standing inaccuracy, it does not break a claim (the
// roomLatch header made the identical correction at Leg C-2).
//
// ⚠️ `note()`/`latch()` READ THEIR ARGUMENT AND WRITE `_latched`; they never READ `_latched`. That keeps them
// free of the self-invalidating read-modify-write N-136 records (the reason the former rooms-panel comment said
// the same about its own private latch).

import { selection, type Selection } from '$common/stores/selection.svelte';
import { spacesState, type KnownSpace } from '$common/stores/spaces-state.svelte';

let _latched = $state<string | null>(null);

export const spaceLatch = {
  /** The raw latched id — what Space was last selected, resolvable or not (verify/debug surface). */
  get latchedSpaceId(): string | null {
    return _latched;
  },
  /**
   * The scoped Space — the latched Space iff it still resolves in the known-Space tree, otherwise null. Null
   * when nothing is latched OR the latched Space is gone between hydrations (stale-latch guard, N-095 spirit:
   * fall back, never throw). R2 reads this for its room list and its "Select a space" empty state.
   */
  get scopedSpace(): KnownSpace | null {
    const id = _latched;
    if (id == null) return null;
    return spacesState.spaces.find((s) => s.space_id === id) ?? null;
  },
  /** THE BUS-FED WRITER — the same fn the shell's `$effect` calls (one of THREE writers of `_latched`; see the
   *  header). Writes only on a `space` selection, so a later room (or identity, or message) selection KEEPS the
   *  Space, which is the whole point. */
  note(sel: Selection | null = selection.current): void {
    if (sel?.entity.kind === 'space') _latched = sel.entity.id;
  },
  /** THE DIRECT WRITER (C-bis-6) — the DM-open paths call this to move the BROWSING latch to the DM's Space
   *  while the BUS still carries the clicked IDENTITY (L-7). It mirrors `roomLatch.latch()`, which latches the
   *  DM's ROOM the same way: opening a DM is the one navigation the user drives WITHOUT clicking the Space, so
   *  `note()` — which latches only a `space` selection — cannot follow it. Without it, `roomLatch.latch()` moves
   *  the ROOM latch while this one stays on the previous Space (the J-708 ① split: R2 draws the old Space's
   *  rooms yet R5/R6 target the DM's room, and R1 highlights the wrong Space). Like `note()`/`clear()`, WRITES
   *  `_latched` and never READS it — no read-modify-write (N-136). ⚠️ This does NOT couple the two latch stores
   *  (line 20 stands): the CALLER moves both latches, exactly as the shell's one effect calls both `note()`s —
   *  `roomLatch` is still never touched by this store, nor this by it. */
  latch(spaceId: string): void {
    _latched = spaceId;
  },
  /** Test/verify affordance — forget the latch (the roomLatch.clear / ingest.clear precedent). */
  clear(): void {
    _latched = null;
  },
};

// DEV-only CDP handle (N-024 idiom). Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_SPACE__?: unknown }).__XGEN_SPACE__ = spaceLatch;
}
