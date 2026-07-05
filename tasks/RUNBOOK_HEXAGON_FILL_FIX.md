# Runbook — M-RP5.0d hexagon badge-clip fix (fill-layer refactor)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Fix the PROVISIONAL hexagon bug: `clip-path` on `.entity-avatar[data-shape="hexagon"]` clips all descendants, slicing the status badge + isAi spark. Move the shape onto an inner fill layer so the root stays unclipped and badges sit on true corners. Additive; skin path `ui/assets/skin.css`. No push — Joe pushes.

---

## Root cause

`clip-path`/`border-radius` shape lives on the root `<figure>`, so its clip region also clips the `.status` child + `::after` spark. Current skin only nudges the badge inward to hide the slice — not a real fix, and isAi has the same latent clip.

## Fix (option A — fill layer)

1. **Component** — `entity-avatar.svelte`: add an absolutely-positioned `.ea-fill` layer (behind content) that carries the shape + seed bg/border. Root `<figure>` → `overflow:visible`, transparent, no border/clip; holds initials + badges on top. Applies to all shapes (circle/square/hex become fill-driven, uniform).
2. **Skin** — `ui/assets/skin.css`:
   - move `background:seed`, `border:seed`, `border-radius`/`clip-path` from `.entity-avatar` → `.entity-avatar .ea-fill` (`position:absolute; inset:0`).
   - `[data-shape="square"]` `border-radius` and `[data-shape="hexagon"]` `clip-path` now target `.ea-fill`.
   - **remove** the hexagon `.status` nudge (`right/bottom` override) — badge returns to the standard bottom-right corner, now unclipped.
   - initials/`::after` spark/`.status` stay on root, unaffected.
3. Verify no seed-ring loss: with the ring on `.ea-fill`, the full hexagon border shows (the old "diminished on diagonals" note is resolved too).

## Verify (CDP 9422)

- `vite build` clean; served module confirmed (N-058).
- Assert: hexagon room-with-status → badge on bottom-right corner, **not clipped** (getBoundingClientRect within viewport, not sliced); isAi spark top-right intact on hexagon; circle/square/DM 0-regression (fill layer renders identical); initials centered; registry count unchanged (no new component); 0 orphans. Screenshot the `room-status` cell.

## D-074 close (one commit)

- `ui/docs/xgen-ui-notes.md` → N-081 (fill-layer refactor, resolves M-RP5.0c PROVISIONAL), v-bump.
- `ui/docs/xgen-ui-components.md` → registry v0.52 (avatar internal note; no surface change), v-bump.
- `docs/ROADMAP.md` → M-RP5.0d ✅, v-bump.
- `CLAUDE.md` PLAY → next-J.
- `JOURNAL.md` → entry (last, real CDP).
- this runbook → COMPLETED.
- No DECISIONS.

## DoD

- [x] `.ea-fill` layer added; shape/seed moved to it; root unclipped `overflow:visible`.
- [x] hexagon status badge on bottom-right corner, unclipped; isAi top-right intact.
- [x] circle/square/DM 0-regression. **Note:** the diagonal-hull seed ring stays absent — a CSS `border` only draws on the rectangular border-box, so `clip-path` cuts bare fill on the diagonals regardless of the fill-layer move (the fill carries them). This was flagged before build; the primary deliverable (badge un-sliced) is fixed; the diagonal ring remains PROVISIONAL (Joe HMR-tunes a drawn hull if wanted). "Full ring restored" (step 3's optimistic phrasing) is therefore **not** claimed.
- [x] CDP-verified (9422): badge rect unclipped (`badgeBottomRightCorner:true`, in-viewport), root `clip-path:none`+`overflow:visible`, `.ea-fill` carries the hexagon polygon, isAi `::after` top:-1px/right:-1px, shapes intact, registry unchanged at 185, `count===unique` 0 orphans; screenshot `temp/room-hex-fix.png`.
- [x] records: N-081 (ui-notes v0.65), registry v0.52 (ui-components), ROADMAP v4.37, PLAY→J-468, JOURNAL J-468, runbook→COMPLETED. No DECISIONS.
