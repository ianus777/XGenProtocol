# M-RP7.4 — Drag to dock: the algebra gets a pointer
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-14  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

The loudest milestone in the arc: a user rearranges the grid **by hand** for the first time. **No new tree algebra** — `move(source, target, edge)` is done, pure, total, live (M-RP7.3, J-524). 7.4 wires **trusted pointer gestures** onto it: activate the grip, draw drop bands, call `move` on release.

**Everything downstream assumes this:** M-RP7.5 persists the arrangement, M-RP7.6 locks it. Neither means anything until a user can produce an arrangement.

**Scope.** `ui/core/lib/components/layout/` (grip handler, the drop-band overlay, prop threading) + `ui/client/src/app_client.svelte` (wire `handleMove` to the gesture, mount the overlay). **No Rust. No sampler. No schema change (`version` stays 3). No persistence** — the moved layout lives in memory until 7.5. **No `skin.css` VALUES** — band colour, ghost opacity, highlight, cursor are M-RP-SKIN's; this leg ships them PROVISIONAL and functional, not tuned.

---

## 1. What is already true — grounded, do not rebuild

- **The grip exists, painted and DEAD.** `region-tile.svelte`: `<span class="region-tile-move" aria-hidden="true">`, no handler. *Joe's own comment: "only with this grip the region can be moved."* **7.4 ACTIVATES it — it does not add a handle.** Drop `aria-hidden`; it becomes an interactive control.
- **`move` is live and total.** `app_client.svelte:87` `handleMove(sourceId, targetId, edge)` already calls `move(layout, …)` and reassigns `layout`. **The gesture's only job is to call `handleMove` with the right three arguments.** `edge ∈ {top,bottom,left,right}`.
- **A tile already knows its own address.** `region-node` threads `srcIndex`/`path`; a rendered tile's `targetId` is its own `widgetId`. **A drop band does not compute an address — it reads one already present.**
- **The threading path is proven.** `onFold`/`onResize` flow shell → node → tile as props (`region-shell.svelte:57`, `region-node.svelte`). **`onMove` follows the identical path.** Do not invent a context/store; match the existing shape (D-067: one way to do a thing).
- **🔒 N-119 is load-bearing here.** The seam's hit overlay works *only* because `.region-seam[data-live] { z-index }` lifts it above the tile that paints after it (skin.css:2863). *pointer-events decides WHETHER an element is hit; paint order decides WHICH.* **Every band edge over a tile boundary is this same fight.**
- **🔒 D-116.** A target tile is an ADDRESS; **a hole is not a drop target.**

---

## 2. The gesture, end to end

1. **pointerdown on `.region-tile-move`** → `setPointerCapture`, record `sourceId = this tile's widgetId`. Nothing moves yet.
2. **past a ~4px threshold** → **drag active**. The source tile gets `data-dragging` (skin dims it — PROVISIONAL). A drag ghost follows the pointer.
3. **pointer over a rendered tile** → four **drop bands** on that tile (top/bottom/left/right, inert centre). `elementFromPoint` picks the band; it highlights.
4. **pointerup on a band** → `handleMove(sourceId, targetId, edge)`. **pointerup anywhere else** (centre · source itself · outside the grid · **Esc**) → **no-op, clean teardown.**

Teardown (release capture, drop `data-dragging`, hide the overlay) runs on **every** end, drop or not. A drag that changes nothing leaves **no trace** in the DOM.

---

## 3. 🔒 The five locked decisions (Joe, 2026-07-14 — "locked")

### D1 — ONE grid-level overlay, not per-tile

A single overlay element, mounted once, painted **last**, at a `z-index` above every tile. It computes the four bands from the **hovered tile's rect**. **This designs the entire N-119 class out in one stroke** — one element above everything cannot be painted over by a sibling. Per-tile overlays would re-fight the seam's battle on every tile. The drag ghost lives here too.

### D2 — the band under the pointer is chosen by HIT-TEST, never geometry

`elementFromPoint(x, y)` → read `data-edge` off the band element. **Do NOT compute the quadrant from the tile rect.** Geometry math disagrees with what actually paints exactly at the boundaries that matter (sub-pixel; the −G/2 seam margins). *Hit-test the thing that paints; do not run a second model of the truth that can drift* (N-124). **The verification leg IS this mechanism, not a parallel one.**

### D3 — a hole offers no band because a band attaches only to a rendered `.region-tile`

**Not "suppress bands over holes" — bands cannot EXIST on a non-tile.** A hole has no `regionId` → no `targetId` → `move` could not address it even if a band appeared. The centre of a folded-across region (its hole) simply produces no band. *A guard you can forget to write is a guard; an impossibility is free* — the same shape as 7.3's unsayable cycle (D-116).

### D4 — a band that would be a no-op does NOT light up

`move` already no-ops a drop that reproduces the source's current position (7.3, §3.6). **7.4 must not HIGHLIGHT such a band** — a band that lights and then does nothing on release is the painted-dead chrome this project refuses (J-500). Compute per hovered tile: suppress the (up to two) edges whose drop would place the source where it already is (source is already that tile's neighbour on that side, in a split of that axis). **This is the one leg with real per-hover cost, and Joe took it in-scope: a lying affordance is a correctness bug, not an appearance one.**

### D5 — keyboard move is FILED, not built

The grip becomes focusable (it takes a handler and `tabindex` this leg), so no dead key ships. But a full keyboard move protocol (pickup → choose target without a pointer → drop → where does focus land) is its own design. **→ `M-RP-MOVE-KBD`, filed in §13.** Named, not silent.

---

## 4. What this is NOT

No tear-off to OS windows (M-RP8). **No entity/content drag** — this drags REGIONS, not what is inside them (`M-RP-ENTITY-DRAG`, filed). No persistence (M-RP7.5). No touch tuning beyond pointer events working. **No band appearance tuning** (M-RP-SKIN).

---

## 5. ⚠️ Traps, named up front

1. **N-119 / D2 is the whole milestone's spine.** If you find yourself computing an edge from coordinates, stop — that is the drift D2 forbids. Read `data-edge` off `elementFromPoint`.
2. **`button: "none"` on button-up moves** (harness rule): a hover that reports `buttons=1` would poison band selection. The drag itself holds the button DOWN — but any post-release hover read must be button-up.
3. **Re-measure coordinates before every gesture** (harness rule): a `move` relocates every tile; a band rect from before the drop is stale.
4. **`setPointerCapture` means move/up events fire on the GRIP, not the tile under the pointer** — that is correct and required (you get a continuous stream). Use `elementFromPoint`, not `event.target`, to find the hovered band.
5. **Rule 6 has fired on the runbook three milestones running, and every time the runbook was wrong.** If the code contradicts this doc, **the code is right — say so, do not absorb it.**
6. **The leaf-count invariant (N-125) is your cheapest tripwire.** A gesture that desyncs the registry is a bug even if the picture looks right.

---

## 6. Verification — every leg re-driven by Chat on the real client, with the TRUSTED-POINTER harness (Rule 5)

**This milestone is a POINTER milestone — the DEV `move()` handle is NOT the proof. `cdp-debug.ps1 -Mode drag` from grip to band IS.** Reload before any baseline (a client mid-selection reads 71 — N-112).

| # | leg | expected |
|---|---|---|
| **V1** | **hit-test agrees with outcome** (the D2 spine) | sweep `elementFromPoint` across all four band edges of a tile; the `data-edge` read at each point matches the edge `move` acts on when released there. **No coordinate the picture shows as one edge resolves to another.** |
| **V2** | **the live relocation** | trusted drag: grip of `spaces` → the RIGHT band of `stream`. `spaces` leaves the left column and appears as `stream`'s right sibling **on screen** (rect moves; title intact; N-125 keys hold). |
| **V3** | **leaf-count invariant** | registry **67/67 unique**, 8 leaves, through a sequence of real drags (sibling · wrap · relocate). |
| **V4** | **a hole offers no band** | fold a region across to make a hole; drag another region over the hole; **no band appears on the hole**, and a release there is a **no-op** with clean teardown. |
| **V5** | **the no-op band does not light** (D4) | hover the source over the edge that reproduces its current position; **the band does not highlight**; release is a no-op. |
| **V6** | **teardown is total** | Esc mid-drag · release outside the grid · release on the source's own centre → each leaves **no `data-dragging`, no overlay, no orphaned capture**; registry unchanged. |
| **V7** | **threshold** | a pointerdown+up on the grip **under 4px** of travel is a CLICK, not a move — no band phase, no `move` call. |
| **V8** | suites | `npm test` **≥ 75 (+ any new cases)** · `vite build` **169** · `cargo test` **1517 / 0 / 62 IDENTICAL** (case-SENSITIVE grep — N-117; kill the client first — it locks the exe). |
| **V9** | cleanup | no inline residue; session ends `location.reload()` (N-123). |

**Test enumeration is production-grounded (D-078):** grep the gesture handler's exported/observable surface; do not infer the suite from this prose.

---

## 7. Definition of done

- [x] `.region-tile-move` activated: `onpointerdown` handler + `role="button"`/`tabindex="0"`; `aria-hidden` dropped. (Keyboard activation deferred → `M-RP-MOVE-KBD`, D5.)
- [x] `onMoveStart` + `draggingId` threaded shell → node → tile (the `onFold` shape). **⚠️ Rule-6 flag: the runbook said "onMove threaded to the tile"; grounded, the TILE only needs a START trigger — under D1 the loop + overlay + the `onMove` completion live at the grid level (region-shell), so the tile gets `onMoveStart`, and `onMove` (→ `handleMove`) is the shell↔app callback. The runbook's wording was off; the code is right.**
- [x] one grid-level overlay: drag ghost + four `data-edge` bands + inert centre, `z-index: 4000` above all tiles (D1)
- [x] band selection by `elementFromPoint`/`data-edge` (D2); **no geometry quadrant math** — the rects only POSITION the hit targets
- [x] bands attach only to `.region-tile[data-region-id]` (D3); no-op edges suppressed via `isMoveNoop` and never highlighted (D4, N-126)
- [x] `app_client.svelte`: the gesture calls the existing `handleMove` via `onMove`
- [x] `M-RP-MOVE-KBD` filed (D5) — §13
- [x] V1–V9 measured on the real client 9222 with the trusted-pointer harness. V1 sweep `top:stream/bottom:stream/left:stream/right:stream/center:-`; V2 spaces relocated far-left→right-of-stream; V3 registry 67/unique 67/leaf 8/**stampMismatches 0** through a drag sequence (N-125 tripwire clean); V4 hole → `bandsDrawn:0`, release no-op; V5 no-op band `data-noop=true active=no`; V6 Esc/outside/source-centre all clean teardown; V7 sub-threshold = click; V8 `npm test` **77** · `vite build` **169** · `cargo test` **1517/0/62 IDENTICAL**; V9 clean quiescent 67
- [x] Records: `docs/xgen-dock-engine-phase0.md` (§11 row 4 CLOSED, §13 M-RP-MOVE-KBD) · `ui/docs/xgen-ui-notes.md` (N-126) · `JOURNAL.md` J-525 · `CLAUDE.md` PLAY · `docs/ROADMAP.md` — one atomic commit (D-074)

*(`Status: COMPLETED` in this header is the shipped signal. "Commit pushed" is not a DoD item — Joe pushes.)*
