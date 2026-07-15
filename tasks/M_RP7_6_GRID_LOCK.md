# M-RP7.6 — The Grid Lock: freeze arrangement, keep function
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-15  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Read first: the grounding, and the one sentence

**The whole milestone in a sentence (Joe):** *the lock that only hides buttons is not a lock — the guard is in the algebra's callers, and the suppression is how you keep it honest.*

**Grounding done before this runbook (grep the code yourself if you doubt a line — Rule 6 has fired on the runbook six straight milestones; this is written to be argued with):**

- **G1 — the seam resize already has a gate.** `region-node.svelte` wires `onpointerdown={live ? (e) => startResize(e, i - 1) : undefined}` — a dead seam gets no listener. The locked-resize gate rides the SAME mechanism: `live && !locked`. A locked seam becomes a dead seam. NOT a new pattern.
- **G2 — `[aria-pressed]` is a free `.shelf-face` skin selector.** The shipped `.shelf-face` rules are `:hover / :focus / :focus-visible / [aria-disabled="true"]` only (skin.css ~2549–2577). Roving `active` drives `tabindex`, not a skin rule. `aria-pressed` and `active` are different axes on the same `<button>` — no collision.
- **G3 — each shelf-face registers WITH its `__icon` child** (`icon.svelte` uses `use:envelope`). A 4th bottom face is therefore **+2 registry** (face + icon). Predicted new quiescent baseline **67 → 69**. ⚠️ **MEASURE it on the live client — do not carry 67, do not assert 69.**
- **G4 — the three gestures and where the lock threads:**
  - **Move grip** lives in `region-tile.svelte` (`onpointerdown={(e) => onMoveStart?.(regionId, e)}`) — plain DOM, NO envelope → element-absent has ZERO registry cost.
  - **Fold buttons** live in `region-tile.svelte` (`.region-title-buttons` span, two `.region-tile-fold` `<button>`s) — plain DOM.
  - **Splitter seam** lives in `region-node.svelte` (`startResize`, the `live` gate above).
  - **Drop-preview + edge bands** live in `region-shell.svelte`, driven by `drag?.active`. `drag` STARTS at the grip. A suppressed grip ⇒ `onMoveStart` never fires ⇒ `drag` stays null ⇒ bands/preview never render. They go inert **transitively**; `handleMove`'s early-return is the backstop. (This is the D1 caveat, closed.)
- **G5 — ZERO RUST (N-116 shape — do not repeat 7.5's miss).** `locked` is a `session` key; Rust round-trips the ui-state blob OPAQUE (D-114). No Rust touches this. Prove it: `git diff --stat` shows no `.rs`, `cargo test` **1517/0/62 IDENTICAL by construction**.

---

## §1 — The lock model (D1 — the load-bearing decision)

One boolean, `locked`. It lives in **`session.locked`** (per-key merge, N-107).

**The lock is the handler refusal (the real guard).** `handleFold` / `handleResize` / `handleMove` in `app_client.svelte` each early-return `if (locked) return;` at the top. This is the layer a command-bridge, a future keyboard-move path (`M-RP-MOVE-KBD`), or a stray `handleMove` call cannot walk around — *an access rule is only real if its callers enforce it* (the D-116 / N-107 shape).

**The suppression is the honesty (so no live-looking dead control ships).** The affordances go **element-absent** when locked (the house convention: a folded tile ships its resize triangle absent, not greyed — J-500 / D8.2). Grip absent, fold buttons absent, seam dead.

**Neither half alone is correct.** Refusal-alone leaves painted-dead chrome (the thing this project keeps refusing). Suppression-alone leaves the algebra reachable. Ship both; the refusal is load-bearing.

---

## §2 — The face (D2 — the FIRST stateful shelf face)

`shelf-face` has `active` (roving) + `disabled` (guard) and no toggle concept. Add one, additively:

- **`shelf-face.svelte`** — new prop `pressed?: boolean` → render `aria-pressed={pressed || undefined}` on the `<button>`; getter becomes `{ command, hasIcon, disabled, active, pressed }`.
- **`shelf.svelte`** — `ShelfItemDef` gains `pressed?: boolean`; thread `pressed={item.pressed ?? false}` into `<ShelfFace … />`.
- **Single glyph + `[aria-pressed="true"]` skin state.** A lock/unlock glyph-swap is an appearance decision → **deferred to `M-RP-SKIN` (§0 autonomy: appearance is Joe's).** Do not swap glyphs here.

---

## §3 — Persistence (D3 — mirror the shipped N-107 pattern, do not re-derive)

- **`uistate.svelte.ts` — new method** mirroring `setSessionLayout` verbatim:
  ```
  setSessionLocked(locked: boolean): void {
    _store.session = { ...(_store.session ?? {}), locked };
    scheduleSessionPersist();
  }
  ```
- **`persist()` — two-key session merge.** The clause currently writes `session` only when `layout` is defined and forwards only `layout`. `locked` is an INDEPENDENT key — extend it so a lock-toggle with no layout change still persists, and neither key clobbers the other or `geometry`:
  ```
  const layout = _store.session?.layout;
  const locked = _store.session?.locked;
  const sessionOut = {
    ...onDiskSession,
    ...(layout ? { layout } : {}),
    ...(locked !== undefined ? { locked } : {}),
  };
  // include `session` in `merged` when EITHER key is present:
  ...(layout || locked !== undefined ? { session: sessionOut } : {}),
  ```
- **`hydrate()` / read-back.** On launch, read `session.locked`; **default `false` when the key is absent** (a past-build store has no `locked` key — the migrate-tolerance you already proved at 7.5 V3). The shell seeds its `locked` `$state` from `uiStateStore.session()?.locked ?? false`.

---

## §4 — Wiring (D4)

**`app_client.svelte`:**
- `let locked = $state(false);` — seed from `uiStateStore.session()?.locked ?? false` after `hydrate()`.
- `commandTable['layout.lock'] = () => { locked = !locked; uiStateStore.setSessionLocked(locked); };` (single toggle — one boolean, one command; two commands would be two sources of truth for one bit, D-067).
- `SHELF_BOTTOM` becomes a **`$derived`** array so the 4th face's `pressed` tracks `locked`:
  `{ icon: 'lock', label: 'Lock layout', command: 'layout.lock', pressed: locked, disabled: false }` (4th, after gear/diskette/load).
- `handleFold` / `handleResize` / `handleMove` — add `if (locked) return;` as the first line.
- `<RegionShell … locked={locked} />`.

**`region-shell.svelte`:** new `locked?: boolean` prop → thread to `<RegionNode … locked={locked} />`. (Bands/preview need no explicit gate — they are downstream of `drag`, which is downstream of the grip; G4.)

**`region-node.svelte`:** new `locked?: boolean` prop → seam becomes `onpointerdown={live && !locked ? (e) => startResize(e, i - 1) : undefined}`; thread `locked` down to `<RegionTile … />` and recursively to child `<RegionNode … />`.

**`region-tile.svelte`:** new `locked?: boolean` prop → wrap the move grip in `{#if !locked}` (element-absent) and the `.region-title-buttons` span in `{#if !locked}` (element-absent).

---

## §5 — The glyph (D5 — provisional, Joe's lane)

`icons.ts` has no `lock`. Add a **`lock`** entry with a PROVISIONAL path (the gear/diskette/load precedent). Name = `core` (D-108); shape → `M-RP-SKIN`. Do not spend effort on the shape — it is re-done at the skin pass.

---

## §6 — Scope (exact files)

`core`: `shelf-face.svelte` · `shelf.svelte` · `region-shell.svelte` · `region-node.svelte` · `region-tile.svelte` · `icons.ts`
`assets`: `skin.css` (`.shelf-face[aria-pressed="true"]` state; any locked-affordance hook if needed)
`client`: `uistate.svelte.ts` · `app_client.svelte`

**ZERO Rust · ZERO sampler · no schema change** — `locked` is a `session` key, NOT a `Layout` field, so the Layout `version` **stays 3** (no `migrateLayout` touch).

---

## §7 — DoD (Chat re-drives every leg on the live client 9222, Rule 5 — numbers not personally measured do not enter the record)

- **V1 — THE LOAD-BEARING LEG: the lock is the algebra's caller, not the chrome.** With `locked=true`, drive `__XGEN_LAYOUT__.move(...)` and `.fold(...)` (the DEV bridge — it calls `handleMove`/`handleFold` DIRECTLY, bypassing the grip) → **layout byte-identical**. This proves the refusal, not merely the suppression.
- **V2 — suppression.** locked ⇒ grip element-absent, `.region-title-buttons` element-absent, seams carry no `pointerdown` listener. unlocked ⇒ all present and live.
- **V3 — bands/preview inert.** locked ⇒ a grip press cannot start (grip absent) ⇒ `drag` null ⇒ region-shell getter shows 0 bands / preview null.
- **V4 — face toggle.** lock face `aria-pressed` flips true/false; shelf-face getter `pressed` tracks; roving `active` unaffected (different axis; G2).
- **V5 — persistence, N-107.** toggle lock ⇒ on-disk `session.locked=true` while `session.geometry` byte-identical AND `session.layout` untouched; survives `location.reload()`. A fold committed while unlocked persists both `layout` and `locked` with neither clobbering the other or geometry. ⚠️ Any disk probe write is **BOM-FREE** (`UTF8Encoding($false)`) or Rust `get_ui_state` chokes → N-095 DEFAULT fallback masquerading as a clean load (N-124).
- **V6 — migrate-tolerance.** a past-build store with no `locked` key hydrates ⇒ `locked=false`, no crash.
- **V7 — content untouched.** locked ⇒ the widget inside each tile is still interactive (scroll a leaf, interact within it). "Keep function" proven, not asserted.
- **V8 — registry.** MEASURE the new quiescent baseline (predict 69 = 67 + face + icon); `count===unique`.
- **V9 — zero-Rust + clean.** `git diff --stat` no `.rs`; `cargo test` **1517/0/62 IDENTICAL**; `npm test`; `vite build`.

**No "commit pushed" line — `Status: COMPLETED` is the shipped signal.**

---

## §8 — Deviations (Rule 6 — flag, do not absorb)

Write-back / flag anything below that grounding contradicts BEFORE building:
- The **+2 registry** prediction assumes the face's `__icon` child registers on the bottom shelf exactly as it does elsewhere. If it comes back +1, that is a finding — MEASURE, report, do not retro-fit.
- The **element-absent grip** assumes the grip does not register (grep: it uses no `envelope`). If absence perturbs the registry, that is a finding.
- If any locked gate turns out to need a **component** change beyond threading a prop (e.g. the seam `live` predicate cannot compose `!locked` cleanly), that is a finding, not a licence — surface it.

---

## Records to change ON CLOSE

`shelf-face.svelte` · `shelf.svelte` · `region-shell.svelte` · `region-node.svelte` · `region-tile.svelte` · `icons.ts` · `skin.css` · `uistate.svelte.ts` · `app_client.svelte` (the code) · `docs/xgen-dock-engine-phase0.md` §11 row 6 → ✅ CLOSED · `ui/docs/xgen-ui-components.md` (shelf-face gains `pressed`; registry bump) · `ui/docs/xgen-ui-notes.md` (an N-note **only if** a real UI lesson surfaces — D-065, do not invent one) · `CLAUDE.md` PLAY · `docs/ROADMAP.md` (paired with CLAUDE.md, D-074) · this task → `Status: COMPLETED`. **No new D expected** (D-116 / N-107 extension).
