# M-RP7.4a — The division preview: show the real post-drop rect
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

Joe's UI note on the shipped M-RP7.4 drag: the drop preview should show **the real region that is about to be created** — its true size and location after the drop — not a fixed slab floating mid-tile. *"The orange preview should be the size of half of that region and on the place where it will be moved."*

**The insight:** M-RP7.4 conflates two things in one `bands` array — the **hit targets** (the `f=0.3` edge strips + inert centre that DECIDE which edge the pointer is on) and the **thing the user sees**. This milestone splits them. The hit geometry is untouched (that is D2's detection); a **separate preview rect** is drawn = the exact half the moved region will occupy.

**Scope: `region-shell.svelte` + `ui/assets/skin.css`. Two files.** No `move`/`mutate.ts` change · no Rust · no schema change (`version` stays 3). The preview DRAWS what `move` already commits; it computes no new geometry.

---

## 1. 🔒 The five locks (Joe, 2026-07-15 — "locked by recomms")

### D1 — the preview rect = the drop-half of the hovered tile

From `drag.hover.rect` `{left, top, width, height}` and the chosen `drag.edge`:

| edge | preview rect |
|---|---|
| `top` | `{ left, top, width, height/2 }` |
| `bottom` | `{ left, top: top + height/2, width, height/2 }` |
| `left` | `{ left, top, width/2, height }` |
| `right` | `{ left: left + width/2, top, width/2, height }` |

**This is `move`'s own 50/50** (M-RP7.3 §3.5, double-then-bisect). *The preview draws the geometry the algebra already decides — it does not run a parallel computation that can drift* (D2 / N-126, the third time this discipline applies). **Full width on a top/bottom drop; full height on a left/right drop; flush to the target's real edges** — exactly the screenshot's ask.

### D2 — one preview at a time, the hovered edge only

Renders **iff** `drag.edge` is a real, non-suppressed edge (`drag.edge !== null && drag.edge !== 'center'`). Joe's lock: not four faint halves — **one solid half, the honest picture of the one result.**

### D3 — 🔒 detection is UNCHANGED; the preview is read-only over the hit layer

The four `f=0.3` strips + inert centre in the `bands` derive **stay exactly as they are** — they decide the edge by hit-test (D2 of M-RP7.4). The new preview rect carries **`pointer-events: none`**; it is PAINT, never a target. *Letting the preview become a hit target would re-introduce D2's forbidden "second model of the truth."* The preview never feeds detection — it only reads `drag.edge` after detection has set it.

### D4 — a no-op edge shows NO preview, a hole shows NO preview

Free from the existing wiring: `drag.edge` is already `null` when the hovered edge is suppressed (M-RP7.4 D4) or when there is no tile under the pointer (D3, hole). The preview's render guard is simply `drag.edge` — so D3/D4 are inherited, not re-implemented. **Do not add a second suppression check.**

### D5 — appearance stays PROVISIONAL (M-RP-SKIN)

Fill, opacity, border — skin's call later. Ship it **geometrically correct, not tuned.** `BAND_FRAC` (0.3) is **detection depth and stays** — it is not the preview.

---

## 2. The change, concretely

**`region-shell.svelte`:**
1. Add a `$derived` `previewRect` (or fold into the existing `bands` block) that returns the drop-half from `drag.hover.rect` + `drag.edge`, or `null` when `drag.edge` is null/`'center'`.
2. In the overlay markup, add **one** element after the `{#each bands}` loop and before/after the ghost:
   ```svelte
   {#if previewRect}
     <div class="region-drop-preview"
          style="left:{previewRect.left}px;top:{previewRect.top}px;width:{previewRect.width}px;height:{previewRect.height}px"></div>
   {/if}
   ```
3. The `bands` array and every `data-edge`/`data-active`/`data-noop`/hit rect stay **byte-identical** — detection is not touched.

**`skin.css`:**
- `.region-drop-preview { position: absolute; pointer-events: none; /* PROVISIONAL fill — M-RP-SKIN */ }` plus a provisional fill (reuse the band's current highlight look so it reads immediately; the tuned appearance is M-RP-SKIN's).
- **`pointer-events: none` is not appearance — it is the D3 correctness lock. It is mandatory, not provisional.**

---

## 3. ⚠️ Traps

1. **Do NOT let the preview drive detection.** If you find yourself reading the preview rect in `hitTest` or `onWinUp`, stop — that is the D2 second-model trap. Detection reads the `f`-strips; the preview reads `drag.edge`. One-way.
2. **Wrap-insert is where preview can lie (§V3).** When the target's parent runs the *other* axis, `move` wraps the target and its siblings don't move. The preview (half the target rect) is still visually correct — but "half the target rect" and "the actual rendered rect after a wrap" are two computations that must AGREE, and proving they do is the milestone's point. Not an assumption — a measured leg.
3. **Re-measure the tile rect live.** The preview is derived from `drag.hover.rect`, captured on tile-enter (M-RP7.4). That is correct — but a resize mid-drag would stale it. Out of scope here (no resize during a move), noted so it is not assumed away.
4. **Rule 6:** if the code contradicts this runbook, the code is right — flag it. (This has now fired on four consecutive milestones.)

---

## 4. Verification — every leg re-driven by Chat, trusted-pointer harness (Rule 5)

Reload before baseline (a client mid-selection reads 71 — N-112).

| # | leg | expected |
|---|---|---|
| **V1** | **the preview IS the drop-half** | hover each of the four edges mid-drag (button held); the `.region-drop-preview` rect measures **exactly half** the target on the drop axis, full extent on the other, **flush to the target's real edges** |
| **V2** | 🔒 **preview = reality, sibling insert** | drop on an edge whose parent runs the drop axis; the moved region's **rendered** rect after the drop matches the preview rect that was showing (sub-pixel) |
| **V3** | 🔒 **preview = reality, WRAP insert** | same, for a target whose parent runs the OTHER axis — the case that can silently disagree |
| **V4** | **preview never captures the pointer** (D3) | `elementFromPoint` over the preview returns the hit strip / tile beneath, not the preview; detection unaffected |
| **V5** | **no-op & hole show no preview** (D4/D3) | a suppressed edge → no `.region-drop-preview` in the DOM; over a hole → none |
| **V6** | suites | `npm test` unchanged (unless the half-calc is extracted + unit-tested — then +cases) · `vite build` 169 · `cargo test` 1517/0/62 IDENTICAL (case-SENSITIVE grep — N-117) |
| **V7** | cleanup | overlay + preview gone on release; no inline residue; session ends `location.reload()` (N-123) |

**V2 and V3 ARE the milestone.** A preview that promises a rect the drop does not deliver is worse than the honest-but-vague slab it replaces. The pass/fail is literally *"does the picture match the result."*

---

## 5. Definition of done

- [ ] `region-shell.svelte`: `previewRect` derived; one `.region-drop-preview` element; **`bands`/detection byte-identical**
- [ ] `skin.css`: `.region-drop-preview` with **`pointer-events: none`** (correctness) + PROVISIONAL fill
- [ ] V1–V7 measured on the real client with the trusted-pointer harness; **numbers Chat did not personally measure do not enter the record**
- [ ] Records: `docs/xgen-dock-engine-phase0.md` (§7 / arc row) · `ui/docs/xgen-ui-notes.md` (the N-note) · `JOURNAL.md` · `CLAUDE.md` PLAY · `docs/ROADMAP.md` — one atomic commit (D-074)

*(`Status: COMPLETED` in this header is the shipped signal. "Commit pushed" is not a DoD item — Joe pushes.)*
