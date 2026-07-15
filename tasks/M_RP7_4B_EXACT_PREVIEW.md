# M-RP7.4b — The exact preview: rehearse the drop, draw the result
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-15  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

M-RP7.4a's preview draws **half of the target's rect as it is NOW**. But `move` does **remove → collapse-degenerate → insert** — the *remove* reflows the grid *before* the region lands. So the preview is right only when the source's removal does not disturb the target, and wrong when it does.

**Measured on the real client (`spaces` → `stream` right, the reflow-heavy case):**

| | left | top | width | height |
|---|---|---|---|---|
| 7.4a preview (½ of stream NOW) | 789 | 164 | 421 | 520 |
| **rendered reality** | **729** | 165 | **459** | 516 |
| **delta** | **60px** | 1 | **38px** | 4 |

Joe: fix it — the preview should show the real size and location **after** the drop.

**This supersedes the filed `M-RP-PREVIEW-EXACT` — it IS that item, built now.**

**Scope: `region-shell.svelte` ONLY.** `move` and `resolveLayout` are already imported (or one import added). **No `mutate.ts` change · no `skin.css` change · no Rust · no schema change (`version` stays 3).** One file.

---

## 1. The approach — "rehearse, then proportion" (measured, not chosen on reasoning)

The preview must equal the rendered result. The rendered result comes from `move` + the browser's flex. **So compute the preview from a dry-run of `move`, not from the current DOM.**

**Why this is both cheap AND exact — three findings, each measured on the client, not argued:**

1. **`move` is pure and total (M-RP7.3), so a dry-run has ZERO side-effects.** Verified: running `move` on the live descriptor left `__XGEN_LAYOUT__.current` byte-identical. The dry-run's only output is *"the tree you'd get."*

2. **The rendered result = half of the target's slot IN THE POST-MOVE TREE.** The moved region lands as a new sibling splitting the target's weight (M-RP7.3 §3.5). Ground truth `l=729,w=459` is exactly half of stream's *post-removal* slot — not half of its current rect.

3. **🔑 THE DECIDER: naive weight-proportional math predicts the browser's real rects within 2px, gaps and all.** Measured: container 1454px, sizes `[1,2,7,2]` → proportional prediction `[121,242,848,242]` vs actual `[121,243,850,243]`, **max delta 2px**. **So NO offscreen render is needed, and NO flexbox reimplementation** — proportional math on the dry-run tree lands within a pixel or two of the real layout.

This is the third time the D2/N-126 discipline applies, and it is why option "simulate the flex geometry from scratch" was **rejected**: that would be a *second model of how the grid lays out*, which drifts the instant a skin gap or min changes. **Proportional-from-weights is not a second model — it is the SAME rule the renderer applies** (`flex: {weight} 1 0` → `weight / total × container`, §3 below). We mirror the renderer's own arithmetic, we do not invent a parallel one.

---

## 2. The algorithm

On each hover with a real `drag.edge` (unchanged guard — D3/D4 inherited):

1. **Dry-run:** `const hypo = move(layout, drag.sourceId, drag.hover.targetId, drag.edge);`
   *(If `isMoveNoop` is true the guard already gave `drag.edge = null`, so we never reach here on a no-op. Belt-and-braces: if `hypo` deep-equals `layout`, return null.)*
2. **Resolve:** `const hypoResolved = resolveLayout(hypo, knownIds);` — the same resolve the render uses, so drops/folds are handled identically.
3. **Locate** the moved region's leaf in `hypoResolved` (by `widgetId === drag.sourceId`), recording its **path**.
4. **Proportion down the path** (§3) → a rect in container space.
5. Draw that rect. On release, the real render produces the same rect (within the ~2px proportional floor).

---

## 3. 🔒 The proportion walk — MIRROR the renderer's two rules exactly, or it drifts

The renderer gives each split child `flex: {weight} 1 0`, **except** a shrink-wrapped split (all children folded across its axis) which gets `flex: 0 0 auto` and **drops out of the weight pool** (`region-node.svelte:75-79`, `splitShrinkWraps`). **The preview math MUST apply both rules** or it drifts on any layout containing a folded-across region.

Starting from the container rect (`.region-shell > :first-child`'s live `getBoundingClientRect`), for each split on the path to the moved leaf:
- Compute each child's effective weight: its descriptor weight, **or 0 if `splitShrinkWraps(child)`** (that child takes its strip size, not a weight share — for an exact match, subtract shrink-wrapped children's measured strip extent first, then proportion the remainder; **if this proves fiddly, see §5 fallback**).
- The moved leaf's fraction along the split's axis = `myWeight / Σ effectiveWeights`.
- Narrow the running rect along `dir` by that fraction and offset by preceding siblings' fractions; the cross-axis extent is the full running rect.
- Subtract the gap contribution is **NOT required** for the ≤2px target — but note it as the known floor (§4).

**🔒 Reuse `splitShrinkWraps` and `carriesMainAxisWeight` from `resolve.ts` — do NOT re-derive the shrink-wrap test.** A concept already in the code is not one to reinvent (Rule 3).

---

## 4. ⚠️ The honesty floor — state it, do not round it away

Proportional math ignores the 4px gaps, so it carries a **~2px error** vs the real render. That is **10× better than 7.4a's 60px and invisible on screen** — but it is a real floor. **Record it as "~2px proportional floor," NOT "pixel-perfect."** *A number rounded before it is recorded cannot be used as a control later* (N-124a). If a future skin makes gaps large, this floor grows and §5 becomes the answer.

---

## 5. Fallback, pre-authorised — if the shrink-wrap proportioning is fiddly

If mirroring the shrink-wrap weight-exclusion in pure math proves error-prone (folded targets giving >5px drift), **do NOT fight it with ever-more-complex math** (that road is the D2 second-model trap). Instead fall back to **measure-the-dry-run**: mount `hypoResolved` in a hidden container sized to the real one, read the moved leaf's `getBoundingClientRect`, draw that. It is more moving parts but cannot drift (it is the real engine). **Flag the switch — do not absorb it silently.** Chat re-drives either way.

---

## 6. Verification — every leg re-driven by Chat, trusted-pointer harness (Rule 5)

The method that gives GROUND TRUTH: capture the preview rect mid-drag; **commit the real move**; read the moved region's rendered rect (fresh query — after a move the tile is a NEW node, N-125, so a stale reference lies — read `matches:1` to confirm); compare; **restore via the saved descriptor.**

| # | leg | expected |
|---|---|---|
| **V1** | **exact on the reflow-heavy case** (`spaces`→`stream` right) | preview rect vs rendered rect **≤ ~2px** on all of l/t/w/h (was 60px/38px) |
| **V2** | **still exact on the local case** (`members`→`stream`, no reflow) | ≤ ~2px (7.4a was already ~1-4px here; must not regress) |
| **V3** | **exact through a FOLDED target** | fold a region across, drop next to it; preview ≤ ~5px (the shrink-wrap path, §3/§5) |
| **V4** | **exact on WRAP vs SIBLING insert** | both insert kinds match reality ≤ ~2px |
| **V5** | **no-op & hole still show nothing** (D3/D4 unregressed) | suppressed edge / hole → no `.region-drop-preview` in the DOM |
| **V6** | **the dry-run has no side-effect** | `__XGEN_LAYOUT__.current` byte-identical before/after a hover that computes a preview; registry **67** |
| **V7** | suites | `npm test` unchanged (or +cases if the proportion walk is extracted & unit-tested) · `vite build` 169 · `cargo test` 1517/0/62 IDENTICAL (case-SENSITIVE — N-117) |
| **V8** | cleanup | overlay+preview gone on release; saved-descriptor probe removed; session ends `location.reload()` (N-123) |

**V1 IS the milestone.** The whole point is that the 60px lie becomes ~2px truth.

---

## 7. ⚠️ Traps

1. **Do NOT reimplement flexbox.** Mirror the renderer's `flex: w 1 0` proportion + the shrink-wrap exclusion, nothing more. Anything fancier is the D2 second-model that drifts.
2. **After a committed move the tile is a NEW DOM node (N-125).** Every ground-truth rect read must be a fresh `querySelector` returning `matches:1` — a held reference reads the pre-move position and lies (this bit Chat twice while measuring this very milestone).
3. **The dry-run must run on a COPY semantics** — `move` returns a new Layout and does not mutate its input (M-RP7.3, total/pure), so passing `layout` directly is safe. Confirm by V6, do not assume.
4. **`resolveLayout` needs `knownIds`** — use the same set the shell already builds for its live `resolved`; do not construct a second one.
5. **Rule 6** (fired on the runbook four milestones running): if the code contradicts this doc, the code is right — flag it.

---

## 8. Definition of done

- [x] `region-shell.svelte`: `previewRect` derives from `move`+`resolveLayout`+proportion walk, not the static half; **shrink-wrap rule mirrored via `carriesMainAxisWeight` (no §5 fallback needed — V3 top 1px proves the strip-exclusion works)**
- [x] no `mutate.ts` / `skin.css` / Rust / schema change
- [x] V1–V8 measured on the real client. **⚠️ The ≤2px bar was NOT met — the FINDING (N-128): the runbook's "≤2px" measured a single split level; the preview walks a multi-level path to a tile, so fixed gaps accumulate (reflow-exact ~2px, up to ~14px on a 3-level wrap). The REFLOW (the actual bug) is fixed: `spaces`→`stream` right left 60px → 2px. Joe reset the bar to OPTICAL correctness — "if the highlighted rectangles are in optically correct positions and size, i am satisfied" — which the ~1–4% residual meets.** Floor recorded as-is, not "pixel-perfect." V6 dry-run purity proven (layout byte-identical while the preview shows). V7 `npm test` 77 · `vite build` 169 · `cargo test` **1517/0/62 IDENTICAL**.
- [x] `M-RP-PREVIEW-EXACT` marked **⬛ superseded by M-RP7.4b** (dock-engine §13)
- [x] Records: `docs/xgen-dock-engine-phase0.md` (§11 row 4b, §13) · `ui/docs/xgen-ui-notes.md` (N-128 + the N-127 correction: conditional 1–120px, not flat 40–100) · `JOURNAL.md` J-527 · `CLAUDE.md` PLAY · `docs/ROADMAP.md` — one atomic commit (D-074)

**⚠️ Deviations from the runbook, flagged not absorbed (Rule 6):** (1) the "≤2px floor" was measured on the wrong quantity (a single split level's column widths, not a multi-level path to a tile) — the real floor is per-level gap accumulation, up to ~14px; recorded honestly, not rounded to the runbook's number. (2) The §5 offscreen-render fallback was NOT taken — the proportional walk (with the strip-exclusion) meets the optical bar Joe set, so the heavier path stays filed. (3) The runbook's headline goal — fix the reflow — IS met (60px → 2px); the ≤2px *everywhere* target was not, and Joe reset it to optical, which is the honest close.

*(`Status: COMPLETED` in this header is the shipped signal. "Commit pushed" is not a DoD item — Joe pushes.)*
