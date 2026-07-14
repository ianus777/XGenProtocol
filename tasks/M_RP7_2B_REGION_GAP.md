# M-RP7.2b — The region-owned gap model (tile margin, zero-width seam)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-14  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this milestone is, and what it is not

**It is:** the gap stops being *something the grid puts between regions* and becomes **something each region owns**. Joe (2026-07-13): *"those gaps are part of each region and are transparent, like a margin on a block element — they overlap when they meet, not added."*

**It is not:** an appearance pass. `--region-gap: 4px` is Joe's value and is **not touched**. `M-RP-SKIN` remains the discharger for every `PROVISIONAL` in the arc.

**It is not gated on `M-RP-PLATE`.** J-520 claimed it was; **J-521 retracted that** — the model needs the *splits not to paint*, which was a two-line skin move, already shipped. *A dependency you have not tried to break is a dependency you have assumed.*

**Scope: `ui/assets/skin.css` ONLY.** No component, no descriptor, no vitest, no Rust. Gaps and spacing are skin (N-090). If this turns out to need a component change, that is a **FINDING, not a licence** — the component change would be its own milestone.

---

## 1. The model, in coordinates (this is the part to argue with)

Let `G = var(--region-gap)`.

- Every **tile** carries `margin: G` on all four sides.
- Every **split** carries **nothing** — no margin, no padding, and (already locked, J-521) **no background**.
- Every **seam** becomes a **zero-width** flex child with **`−G/2` margin on each side, along the split's axis**.

Walk a row: tileA's border box ends at `x = 0`.

| | |
|---|---|
| tileA `margin-right` | `+G` → margin box ends at `G` |
| seam margin box | starts at `G`, outer size `0 + (−G/2) + (−G/2) = −G` → ends at `0` |
| seam **border box** | sits at `G/2` — **dead centre of the gap** |
| tileB `margin-left` | `+G` → tileB **border box** starts at `G` |

**Result: tileA↔tileB = exactly `G`,** and the seam's zero-width box lands in the middle of it, so the drag survives.

**It composes at any depth** because a split contributes nothing: `tile → seam(−G) → nested split(0 margin) → its first child tile(+G)` = `G − G + 0 + G` = **`G`**.

Four boundaries, one number:

| boundary | today | after |
|---|---|---|
| tile ↔ tile | `G` (seam element) | **`G`** |
| tile ↔ frame edge | `G` (shell padding) | **`G`** (the tile's own margin — `--region-pad` **disappears**) |
| tile ↔ **hole** | **`0`** ⚠️ | **`G`** — *the one boundary that is wrong today* |
| nested tile ↔ uncle | `G` | **`G`** |

---

## 2. The edits (three, all in `skin.css`)

**E1 — `.region-shell`: retire two tokens, delete the padding.**
- `--region-pad` and `--region-seam` are **deleted**. One knob remains: `--region-gap`.
- `.region-shell { padding: var(--region-pad) }` → **removed**. The perimeter is now the outermost tile's own margin.

**E2 — `.region-tile`: `margin: var(--region-gap)`.**

**E3 — `.region-seam`: zero-width, negative margins per axis.**
- `flex: 0 0 0` (was `0 0 var(--region-seam)`).
- `[data-dir="row"]` → `margin-inline: calc(-0.5 * var(--region-gap))`
- `[data-dir="col"]` → `margin-block: calc(-0.5 * var(--region-gap))`

---

## 3. 🔒 M1 — the grab zone must be re-derived, because the seam is now zero-width

Today's `--region-seam-hit: max(1px, calc((8px - var(--region-gap)) / 2))` exists because the seam element **is** `--region-gap` wide (N-122: at gap 0 the seam was 0px and a fixed 1px expansion left a 2px target — alive and ungrabbable). **That premise is now false: the seam is ALWAYS zero-width.** The entire target is the `::before`.

**New:** `--region-seam-hit: max(4px, calc(var(--region-gap) / 2))` → grab zone `= 2 × hit = max(8px, G)`.

- **Never smaller than a finger** (8px floor).
- **Never smaller than the gap the user can see** (at `G = 20` the whole 20px reads as draggable, not a middle strip of it).

`calc()`/`max()` is legal here **only because this token is never read by JS** (N-122). `--region-min` and `--region-snap` **stay plain values** — `getComputedStyle` does not resolve `calc()` in a custom property; it returns the raw token stream and `parseFloat` gives `NaN`, and **a clamp of NaN is no clamp, and it passes.**

## 4. 🔒 M2 — `.region-split` must never get a background, and it is now load-bearing

Already locked (J-521, when the backdrop moved to `.region-shell`). Under this model a **nested split's box overlaps its neighbour's margin**, so a background on it would **paint into the gap and eat it**. It stops being a tidiness rule and becomes a correctness rule. It is a verified leg (V2), not a comment.

## 5. 🔒 N-119 still holds, and matters more

`.region-seam[data-live="true"] { z-index: 1 }` **stays**. The `::before` now reaches **into both neighbours' border boxes** at small `G` — and *pointer-events decides whether an element is hit; paint order decides which.* Verified by sweeping `elementFromPoint`, not by reading the CSS (V4).

---

## 6. ⚠️ THE ONE CLAIM THIS MILESTONE MUST NOT ASSERT

**A margin sits OUTSIDE the flex item's border box, so `flex: n 1 0` still distributes free space proportionally and `[1,2,7,2]` survives EXACT.**

That is the design's central assumption. It is **coherent** — and N-121 is the reason coherence is not enough:

> a real `border` broke the arc's baseline to `[1, 1.97, 6.90, 1.97]`, because a border is **inside** the box: flex distributed free space to the **content** boxes and then added a **constant** `2 × border` to every **border** box. **Every rect JS can measure is a border box** — and the splitter computes its drag fraction from `getBoundingClientRect()`, so it would have biased the resize arithmetic **systematically, silently, forever, and nothing would ever fail.**

**A margin should behave differently — but "should" is not a verification.** V1 is the look.

**🔒 If V1 does not come back EXACT, the model does not ship as designed.** It is reported as a finding, not tuned into looking right.

---

## 7. Verification — every leg re-driven by Chat on the real client (Rule 5)

Client `9222`. **Reload before measuring a baseline** — a client mid-selection reads 71, not 67 (N-112/N-115).

| # | leg | expected |
|---|---|---|
| **V1** | **split ratios, measured border boxes** | **`[1,2,7,2]` EXACT** — the N-121-class check |
| **V2** | the four gaps, measured | tile↔tile `G` · tile↔edge `G` · **tile↔hole `G` (today 0)** · nested-at-depth `G` |
| **V3** | the drag still commits | **MID**-drag (`-MidExpression`, button down) descriptor still `[1,2,7,2]`; **AFTER** = integers, pair total invariant, untouched siblings ×100 |
| **V4** | grab zone, `elementFromPoint` swept at `--region-gap` 0 / 4 / 20 | zone `≥ 8px` at every setting; the seam wins over **both** neighbours |
| **V5** | registry | **67** (quiescent · empty store · **no selection** · nothing folded · **zero saved UI states**); the seam still does not register |
| **V6** | clamp | stops at **22px**, `data-collapsed`/`data-fold-mode` null — *it stops, it does not fold*; folded strip 22px, zero overflow |
| **V7** | suites | `npm test` **59** · `vite build` **169** · `cargo test` **1517 / 0 / 62 IDENTICAL** — which *proves* the zero-Rust claim rather than asserting it (case-**SENSITIVE** grep — N-117) |
| **V8** | **cleanup (N-123)** | no inline style left on `.region-shell`; the session ends with `location.reload()` |
| **V9** | **Joe looks at it** | the instrument that has paid four times (M-RP7.1 the hole · M-RP7.1b the chevron · J-520 the two-valued gap · J-521 the two-surfaced perimeter) |

⚠️ **Re-measure coordinates before every gesture.** A rect is not a constant, and **this milestone moves every rect in the grid.**

---

## 8. Definition of done

- [ ] E1 · E2 · E3 applied to `skin.css`; `--region-pad` and `--region-seam` **gone from the file** (grep, don't assume)
- [ ] V1–V8 measured on the real client; numbers recorded from Chat's own reads
- [ ] V9 — Joe has looked at it
- [ ] `docs/xgen-dock-engine-phase0.md` §4.5.2 rewritten (the gap model ships; the `M-RP-PLATE` gate stays retracted)
- [ ] `ui/docs/xgen-ui-notes.md` — the N-note (whatever the measurement actually teaches, not what this runbook predicts)
- [ ] `JOURNAL.md` J-523 · `CLAUDE.md` PLAY · `docs/ROADMAP.md` — one atomic commit (D-074)

*(`Status: COMPLETED` in this header is the shipped signal. "Commit pushed" is not a DoD item — Joe pushes.)*

---

## 9. ✅ CLOSE (J-523) — what actually happened, and §6 is why this section exists

**Every mechanical leg came back green. §6's warning fired on the milestone's OWN PREMISE instead.**

### ⚠️ The fourth boundary was a ghost

§1's table claimed **tile↔hole = `G`, "today ZERO — the one boundary that is wrong"**. Chat wrote that into this runbook, into `skin.css` and into three canonical records **before measuring it**. A/B-measured (inject the OLD geometry live, fold a tile ACROSS to build a real hole):

| | NEW | **OLD (injected)** |
|---|---|---|
| folded tile insets | 4 / 4 | **4 / 4** |
| distance to the hole | 774.8px | **774.8px** |
| adjacent-gap census | all **4** | **all 4** |
| perimeter | **4** | **4** |

**Since J-521 moved the backdrop to `.region-shell`, a hole and a gap are the SAME SURFACE.** A tile "butting onto a hole" is indistinguishable from a tile with a gap — *there is nothing on the far side of that gap to be separated from.* **The model has ZERO VISUAL DELTA.** → **N-124.**

**Joe shipped it anyway, on the honest reason:** it **deletes a mechanism, not a token** (perimeter-as-shell-padding + inter-tile-as-seam-thickness = two mechanisms that had already drifted apart once), and **a gap that lives on the REGION travels with it when M-RP7.4 moves it.**

### §6 called it, and §6's own claim also failed

§6 said `[1,2,7,2]` must come back **EXACT** or the model does not ship. **It came back `[1, 2.000112, 7.000224, 2.000112]` — and the OLD geometry returns `2.000114`.** The 0.004px is **pre-existing Chromium sub-pixel, in both models**, which is exactly what **proves the margin adds no bias**. The arc's "EXACT" was exact to *display precision*. → **N-124a.** *(The clamp likewise stops at **22.29px**, not "exactly 22" — integer-weight rounding; the OLD geometry rounds to **21.89**, i.e. just BELOW the floor.)*

### Measured (Chat re-drove every leg, Rule 5 — client 9222)

| leg | result |
|---|---|
| V1 ratios | `2.000112` (new) vs `2.000114` (old) — **no bias** |
| V2 gaps | perimeter **4/4/4/4** · **14 adjacent tile pairs all exactly 4**, incl. pairs crossing a split boundary → **it composes at depth** |
| V3 drag | **MID (button down)** `[1,2,7,2]` untouched while tiles painted 216/118 · **AFTER** `[194,106,700,200]` — pair total 300 invariant, untouched siblings ×100 |
| V4 grab zone | **9px @ gap 0 · 9px @ gap 4 · 20px @ gap 20**; the seam wins at x=113–115 — *inside* the neighbour's border box (**N-119's `z-index` re-proven**) |
| V5 registry | **67** quiescent; the zero-width seam registers nothing |
| V6 clamp | **22.29px**, `data-collapsed`/`data-fold-mode` **null** — *it stops, it does not fold*; `--region-min` still reads `22px` on the seam |
| V7 | `npm test` **59** · `vite build` **169** · zero Rust **by scope** (`git diff` = `ui/assets/skin.css` only) |
| V8 | probes removed, no inline style, client reloaded quiescent |
| V9 | **Joe drove it by hand** — registry 71 (selection active) and a hand-dragged descriptor `[100,220,680,200]` |

### ⚠️ Chat's own defect, recorded not absorbed

The skin edit was five line-index splices, and **two ran top-down** — so an earlier splice shifted the file and the next one **deleted `min-height: 0` from `.region-shell`** instead of the `padding` line. **Caught by the grep verification, not by the diff.** Fixed, re-verified, and the rule written down: **splice DESCENDING, and guard every splice with an assertion on the line it is about to replace.** → **N-124b.**
