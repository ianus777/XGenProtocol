# M-RP7.5 — The session layout feeder (writes `session.layout`) + M-RP-RESTART (Restart · Revert UI)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-15  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

The grid finally **persists**. Until now every fold/resize/move mutated `layout` **in memory only**; `loadLayout()` reads `store.session.layout` but nothing ever wrote it, so it always fell to `DEFAULT_LAYOUT`. This milestone adds the **write half** — and its companion verbs (M-RP-RESTART: **Restart** + **Revert UI**), which ship together because the feeder *creates* the state those verbs act on.

**🔑 The load half is already shipped.** `loadLayout()` (layout-default.ts) already does `store.session.layout → migrateLayout(layout, DEFAULT_LAYOUT) → render`. Do **not** touch it. This milestone gives it a value to read.

**Scope:** `ui/client/src/uistate.svelte.ts` · `ui/client/src/app_client.svelte` · `xgen-client/src/desktop.rs` (Restart plugin wire ONLY) · `xgen-client/src/capabilities/*.json` (one grant) · `Cargo.toml`/builder (plugin init). **No `ui/core/**`, no sampler, no `mutate.ts`/`resolve.ts`, no schema change (`version` stays 3).**

---

## 1. 🔒 Locks (Joe, 2026-07-15)

- **Object name:** `uiStateStore.setSessionLayout(layout)` — the debounced per-key session writer. Peer to `save`/`load`/`remove`.
- **N-107 one level deeper:** the write MUST merge **per key inside `session`** — read the on-disk `session` fresh, spread it, override only `layout`. A whole-object `session:` write **eats `geometry`** (Rust writes `session.geometry`; the frontend writes `session.layout`).
- **Debounce ~400ms** on session writes (§12 mandates it; note each mutation is already a single discrete commit — resize writes once on `pointerup`, J-519 — so the debounce only coalesces rapid sequences).
- **M-RP-RESTART decomposed (Joe): two parts of one idea.**
  - **Restart** = bounce the process. Wire the **already-declared-but-dead `tauri-plugin-process`** (`restart()`). Reload naturally re-reads the autosave via `loadLayout()`. **This is the ONLY Rust in the milestone.**
  - **Revert UI** = the frontend half, standalone (its own interactive element): live-reload `session.layout` from disk and reassign the grid, **no process bounce**. "Renew UI from last autosave." Zero Rust.
- **Reset-to-default is OUT OF SCOPE** — filed to the (unbuilt) settings surface; "buried deep, not an everyday control" (Joe). This milestone ships NO reset-to-`DEFAULT_LAYOUT` verb.

**⚠️ Supersedes the filed J-520 wording** for Revert (*"load last automatic save"*): grounded against the continuous feeder, "last autosave ≈ live grid", so Revert-UI is a **reload/refresh** (renew from disk), not an undo. The J-520 note's own "greyed-100%-today" caveat is retired by the feeder existing.

---

## 2. ⚠️ cargo test IS EXPECTED TO MOVE — and that is the inverse proof

Through M-RP7.1–7.4 the discipline was `cargo test` **1517/0/62 IDENTICAL** = proof no Rust landed. **M-RP7.5 breaks that on purpose**, but ONLY via the Restart plugin wire.

- The **feeder** (Legs A/B) and **Revert UI** (Leg D) are **zero-Rust** — if they alone moved `cargo test`, something is wrong.
- The **Restart** wire (Leg C) adds the `tauri_plugin_process` surface → the delta must be **exactly** that plugin's tests, nothing in `xgen-core`/`space`/`node`. **Grep `git diff --stat` for `.rs` files: the only `.rs` change is `desktop.rs` (builder `.plugin(...)`). No `xgen-core/**`, no `space/state.rs`.**

If Leg C is deferred (see §7), `cargo test` stays IDENTICAL and that is the correct signal.

---

## 3. Leg A — `setSessionLayout` + the N-107-correct `persist()` merge (`uistate.svelte.ts`)

**A1.** Add a method on the store:
```ts
/** M-RP7.5 — feed the SESSION arrangement. Debounced per-key write; never a whole-`session` write. */
setSessionLayout(layout: Layout): void {
  _store.session = { ...(_store.session ?? {}), layout };
  scheduleSessionPersist();   // ~400ms debounce → persist()
}
```

**A2.** `persist()` becomes the ONE N-107-correct write path. Today it merges `named`/`active` over `onDisk` and never writes `session`. Extend the merge to carry `session.layout` **into the on-disk session bag**:
```ts
const onDiskSession = (onDisk.session && typeof onDisk.session === 'object') ? onDisk.session : {};
const layout = _store.session?.layout;
const merged = {
  ...onDisk,
  version: 1,
  named: _store.named,
  active: _store.active,
  // Per-key merge INSIDE session: preserve geometry (Rust's) + unknown keys; override layout only.
  ...(layout ? { session: { ...onDiskSession, layout } } : {}),
};
```
- `layout` undefined (pre-first-mutation) ⇒ no `session` key written ⇒ geometry untouched, loadLayout still DEFAULTs. Guarded above.
- `onDisk.session` is read FRESH each write (the existing `get_ui_state` at the top of `persist()`), so a geometry Rust wrote since our last read survives.

**A3.** Debounce helper (module-level; mirrors the Rust 500ms geometry throttle in spirit):
```ts
let _sessionTimer: ReturnType<typeof setTimeout> | null = null;
function scheduleSessionPersist(): void {
  if (_sessionTimer) clearTimeout(_sessionTimer);
  _sessionTimer = setTimeout(() => { _sessionTimer = null; persist(); }, 400);
}
```

**⚠️ Rule-6 check for Clair:** the named `save`/`load`/`remove` paths already call `persist()` immediately. If a named op and a debounced session write interleave, both read `onDisk` and both write — last-write-wins per key, which is correct because both now merge per-key. Do NOT add a second write path; route everything through `persist()`.

---

## 4. Leg B — feed on every mutation (`app_client.svelte`)

`handleFold` / `handleResize` / `handleMove` each already reassign `layout`. After each reassignment add:
```ts
uiStateStore.setSessionLayout($state.snapshot(layout));
```
`$state.snapshot` — the same de-proxy the named `save` uses (line ~242). Three call sites, one line each. Nothing else.

---

## 5. Leg C — Restart (the ONLY Rust): wire `tauri-plugin-process`

**C1.** Ground first (do NOT assume): confirm `tauri-plugin-process` is in `xgen-client/Cargo.toml` and confirm there is **no** existing `.plugin(tauri_plugin_process::init())` in `desktop.rs`. The ROADMAP (J-520) says declared-never-wired — verify, then wire:
```rust
.plugin(tauri_plugin_process::init())
```
**C2.** Capability grant — plugin commands need one (J-498: the permission lesson never generalises; ground it per command class). Add the process-restart permission to `xgen-client/src/capabilities/default.json` (verify the exact permission id against the plugin's manifest — likely `process:default` or `process:allow-restart`).

**C3.** Frontend — a **File ▸ Restart** command via the existing menu-bar + command table (M-RP6.1c/d), id **`app.restart`**:
```ts
import { relaunch } from '@tauri-apps/plugin-process';
// command handler: await relaunch();
```
(Confirm the JS import path against the installed plugin — `@tauri-apps/plugin-process` is the v2 shape.)

**⚠️ Named risk (J-520):** after live messaging (M-RP6.3) a restart tears down a WS that may hold unsent messages. Not this milestone's concern — but do NOT add a "save drafts" hook; just the bounce.

---

## 6. Leg D — Revert UI (zero Rust): standalone renew-from-autosave

**D1.** A command id **`layout.revert`** wired the same way (menu item and/or a dedicated interactive element — Joe owns the graphical form; you own the wiring, §0 autonomy):
```ts
async function handleRevertUi() {
  const next = await loadLayout();   // re-reads session.layout from disk, migrates, never null (N-095)
  layout = next;                     // reassign → shell re-resolves
}
```
- Uses the **shipped** `loadLayout()` unchanged — the renew is literally "read the autosave again."
- **Never** assign null/undefined (the J-499/N-095 blank-centre trap). `loadLayout()` already guarantees non-null.
- No `setSessionLayout` call after revert — reloading the same disk state and re-writing it is a no-op; skip it.

---

## 7. Sequence / dependency

A → B (feeder complete, verifiable alone: mutate, read disk). Then D (Revert reloads what the feeder wrote). C (Restart) is independent and may be **deferred to a C-tail commit** if you want a zero-Rust feat first (then `cargo test` stays IDENTICAL for A/B/D, moves only at C — the §2 proof, split cleanly). Recommended: **A+B+D as one feat, C as a second** so the Rust delta is isolated in its own `git show --stat`.

---

## 8. Verify — Chat re-drives EVERY leg on the real client 9222 (Rule 5)

Baseline: relaunch to quiescent, measure registry (was 67 at M-RP7.4b — re-measure, N-105/N-108; do not quote).

- **V1 feeder writes + N-107 holds (the core proof).** Mutate (fold R3 or move a tile) → read `xgen-client_uistate.json` on disk → `session.layout` present and matches the descriptor → **`session.geometry` byte-intact**. Then move the OS window (Rust writes geometry) → mutate again → re-read: **both** `session.layout` (new) AND `session.geometry` (new, from Rust) present. *This is the N-107 proof: neither writer ate the other.* Split state-change and disk-read across two steps (N-099).
- **V2 the "unreachable today" that becomes reachable — persistence across relaunch.** Arrange the grid (a move + a fold) → wait out the debounce → **Restart** (or manual relaunch) → grid returns **arranged**, not DEFAULT. First load of a saved workspace, driven not reasoned.
- **V3 load survives a past-build descriptor.** Inject a `session.layout` carrying a **retired widgetId** (the N-120 ghost) via the store, relaunch/reload → resolve drops it (`droppedCount≥1`), the rest renders, no crash, no blank centre. Then a malformed shape → `loadLayout` falls to DEFAULT (N-095). *No new code — this verifies the shipped drop/migrate path.*
- **V4 Revert UI renews from disk.** Mutate → confirm autosave → Revert UI → grid reloads to the autosaved arrangement (a refresh, not an undo — assert it matches the LAST autosave, not a pre-mutation state).
- **V5 Restart is the only Rust.** `git diff --stat`: the sole `.rs` change is `desktop.rs` (`.plugin`). `cargo test` moves by exactly the `tauri_plugin_process` surface; **zero** `xgen-core`/`space`/`node` delta. If C deferred: `cargo test` 1517/0/62 IDENTICAL for A/B/D.
- **V6 build/tests.** `npm test` (re-measure) · `vite build` (re-measure) · `cargo test` (record the new number — it is EXPECTED to move; §2).
- **V7 clean quiescent** — no inline residue, harness swatches cleared, `location.reload()` (N-123).

**Harness:** trusted-pointer `cdp-debug.ps1` for the drags; `__XGEN_LAYOUT__` for the descriptor; `__XGEN_UISTATE__` for the store; ground-truth rects via **fresh** `querySelector` returning `matches:1` (N-125 — a moved tile is a new DOM node). Re-measure coordinates before each gesture.

---

## 9. Definition of Done

- [ ] `setSessionLayout` + per-key `session` merge in `persist()` (N-107) — geometry survives, proven on disk (V1).
- [ ] Feeder wired in the 3 mutation callbacks (V1/V2).
- [ ] Grid persists across a relaunch (V2).
- [ ] Load survives a retired widgetId / malformed shape via the shipped drop+migrate path (V3).
- [ ] Revert UI renews from the last autosave, non-null, no blank centre (V4).
- [ ] Restart bounces the process (`tauri-plugin-process` wired + granted); the ONLY Rust; delta isolated (V5).
- [ ] `cargo test` new number recorded (expected move) · `npm test` · `vite build` re-measured (V6).
- [ ] No `ui/core/**`, no sampler, no schema change; every canonical number Chat-measured, not Clair-quoted (Rule 5).

*(`Status: COMPLETED` is the shipped signal — "commit pushed" is not a DoD item. Joe pushes.)*

---

## 10. Filed, NOT in this milestone

- **Reset to default** (`DEFAULT_LAYOUT` from deep in settings) — the experiment-nuke; separate, out of scope (Joe: "buried deep, not everyday").
- **Revert-to-active-named** (reload the active named state's layout rather than DEFAULT/autosave) — a richer revert; filed.
- **M-RP7.6 grid lock** — the `locked` flag also lives in `session`, so it lands AFTER this (N-107 per-key inside `session` must be shipped first).
- Unsent-message-safe restart (post-M-RP6.3) — J-520 named risk; not now.
