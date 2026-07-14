# M-RP7.2 — Splitter resize on the seam
> **Status**: COMPLETED  
> Version: 2.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

**Implementer: Clair.** Design authority: none — every open question below is already closed. **If a step is wrong, say so and stop** (Rule 6: at M-RP7.1b the deviation was correct and the runbook was the thing that was wrong).

> ## ✅ **CLOSED — J-519. Code commit `9faa38c` (8 files, +512/−28). Do not re-execute.**
>
> **MEASURED (Chat re-drove every leg, Rule 5):** registry **67** (quiescent · empty store · no selection · nothing folded · zero saved UI states; seam does not register) · `cargo test` **1517/0/62 IDENTICAL** · `npm test` **59** · `vite build` **169** · `--region-min` reads **22px on the seam** · 7 seams · clamp stops at **exactly 22px** without folding · **MID-drag: descriptor untouched at `[1,2,7,2]` while the tile painted 74→176px; AFTER: `[237,63,700,200]`.**
>
> **⚠️ RULE 6 FIRED TWICE, AND BOTH TIMES THE RUNBOOK WAS THE THING THAT WAS WRONG.**
> **① N-119 — §4's `::before` hit area did not work.** The next tile **paints over** the far half of the overlay; **half of the seam's own VISIBLE pixel was unclickable.** Fixed with `z-index: 1` on a live seam — **Clair's deviation, correct, accepted.** *`pointer-events` decides WHETHER an element is hit; **paint order decides WHICH**. Expanding a hit area is two facts and the CSS states one.*
> **② §5's premise was misattributed** — `resolve.ts` did **not** own the flex decision. The **intent** (one source, D-067) was right; the **fact** was wrong. She exported the predicates and deduped three call-sites instead of adding a fourth.
>
> **🔑 AND ONE DEFECT THIS MILESTONE SHIPPED, NOW OWNED BY M-RP7.3 — N-120.** `path` is threaded over the **RESOLVED** tree; `resizeSplit` walks the **DESCRIPTOR**. `resolve.ts` drops unknown leaves → **the index spaces diverge**. **Reached, not argued:** with one ghost widget, dragging a seam **right to enlarge a panel HALVED it**, and **55% of the row's weight went to a widget that does not exist**. **Fold is drop-safe (addresses by `regionId`); resize is drop-fragile (addresses by position).** *Unreachable in today's build; reachable the first time a widget id is retired.* → **a REQUIRED LEG of M-RP7.3, not a filed item.**

**Read first:** `docs/xgen-dock-engine-phase0.md` v1.5 (§4.2, §6, §7 — **§4.1-H is HISTORY, do not implement it**) · `ui/docs/xgen-region-dock-model.md` v2.1 · `ui/core/lib/components/layout/` (all of it) · **`ui/docs/xgen-ui-notes.md` N-118 — it is not optional, it is a correctness requirement in leg 4.**

---

## 0. What this milestone actually is

The seam between two tiles becomes a drag handle. Weights snap to a quantum. **The arithmetic is the cheap part.**

> **⚠️ THE EXPENSIVE PART IS ALREADY DONE, AND IT IS NOT YOURS: leg 0, the trusted-mouse harness (Chat, shipped).** A synthetic `MouseEvent` from `eval` is **untrusted** and fires no native defaults (J-496). `cdp-debug.ps1` now has **`-Mode click`** and **`-Mode drag`**, driving real `Input.dispatchMouseEvent` — and a **`-MidExpression`** that is evaluated **while the button is still down**, which is the only way to tell a live preview apart from a descriptor written on every move. **Verified on the real client: coordinates are CSS pixels (DPR 1.25 does NOT apply), `isTrusted=true`, `buttons=1` through the moves, three consecutive drags byte-identical.**

**You do not need to build any of that. You need to not break it.**

---

## 1. 🔒 Design locks — taken under §0 autonomy. Do not re-open, do not re-derive.

| # | lock |
|---|---|
| **L1** | **`mutate.ts` is BORN in this milestone**, beside `resolve.ts`, holding **`resizeSplit` only**. M-RP7.3 adds `move` and pulls `fold` out of the shell. *The arc table put the algebra after the first mutation; that cannot be true — the first mutation IS algebra.* |
| **L2** | **Weights stay INTEGERS. Never a float in the descriptor** (§7). Resolution comes from an **exact integer scale-up**, not from rounding. |
| **L3** | **Live preview during the drag; integers written ONCE, on `pointerup`.** A transient local override renders; the descriptor is not touched until release. |
| **L4** | **A seam is draggable iff BOTH neighbours carry a main-axis weight** — i.e. neither is `flex: 0 0 auto`. One predicate; it covers folded-along and shrink-wrapped without a special case. |
| **L5** | **Addressing is a derived `path: number[]`. NO schema change, no split id.** *A key nothing writes is a key nobody has round-tripped.* |
| **L6** | **The clamp reads the skin.** The minimum is a CSS custom property; the component owns the clamp, the skin owns the number (N-090). **It STOPS. It never auto-folds** (§4.2). |
| **L7** | **No keyboard resize.** `role="separator"` + arrows would put a tab stop on **all 7 seams** that nobody asked for, and it would dodge the harness this milestone exists to land. **Filed.** |
| **L8** | **The seam does NOT register.** `.region-split` has no getter either — the seam is *its* chrome. Proof is the tree diff + painted geometry (§10), not a getter. **Registry stays 67.** |

---

## 2. Leg 1 — `mutate.ts` (pure, no DOM, no Svelte)

New file `ui/core/lib/components/layout/mutate.ts`.

```
resizeSplit(layout: Layout, path: number[], seamIndex: number, fraction: number): Layout
```

- **`path`** — child indices from the root **to the SPLIT node**. `[]` = root.
- **`seamIndex`** — `i`, the seam between `children[i]` and `children[i+1]`.
- **`fraction`** — where the pair's boundary now sits, as a fraction of **the pair's combined weight**. Caller clamps in pixels; you clamp defensively to at least one unit a side.

**Algorithm — and every step of it is load-bearing:**

1. Walk `path`. **If the target is not a split, or `seamIndex` is out of range, or `path` is bad → return `layout` UNCHANGED. Never throw.** (N-095's temperament: a bad input is not a crash.)
2. `total = sum(sizes)`.
3. **`k` = the smallest power of ten such that `total * k >= 1000`.** `[1,2,7,2]` (total 12) → `k = 100` → `[100,200,700,200]`. **If `total >= 1000` already, `k = 1` — an already-scaled split is NEVER rescaled again.**
   > **🔑 WHY A POWER OF TEN AND NOT `ceil(1000/total)`.** Both are exact. `ceil` gives `k=84` → `[84,168,588,168]`, which is correct and unreadable. **A saved workspace is a file a human opens.** Round numbers cost nothing here and the descriptor stays legible.
   > **🔑 WHY SCALE AT ALL.** Total 12 across ~1200px means **one weight unit = 100px** — an unusable drag resolution. Integer scaling multiplies every sibling by the same `k`, so **untouched siblings keep their proportions to the byte.** No rounding, no drift, no float. *This is the whole reason §7's no-floats lock survives contact with a mouse.*
4. `pair = scaled[i] + scaled[i+1]`.
5. `a = round(pair * fraction)`, clamped to `[1, pair - 1]`. `b = pair - a`.
6. New `sizes` = `scaled` with `[i] = a`, `[i+1] = b`. **The pair's total is invariant, so no other tile moves and the split's own total is invariant too.**
7. Return a **new** tree. **Do not mutate the input.** `version`, `dir`, `widgetId`, `collapsed` all pass through untouched.

### Tests — `mutate.test.ts` (vitest, no DOM)

Enumerate by **grepping what you exported**, not from this list (D-078). At minimum:

1. `[1,2,7,2]` → first resize scales to total **1200**; ratios of the *untouched* siblings are **byte-identical**.
2. **Only the pair changes.** Every other entry is `===` its scaled self.
3. **Pair total invariant**; split total invariant.
4. **A second resize does NOT rescale** (`k = 1`).
5. `fraction = 0` and `fraction = 1` → each side still ≥ 1 unit.
6. Bad path · non-split target · `seamIndex` out of range → **returns the input unchanged, throws nothing.**
7. **Input not mutated** — pass a deep-frozen layout.
8. `row` and `col` both.
9. `collapsed` and `version` survive a resize.

---

## 3. Leg 2 — `path` threading

- `region-node.svelte` gains `path: number[]` (default `[]`), recursing as `[...path, i]`.
- `region-shell.svelte` mounts the root with `path={[]}`.
- **Purely additive and purely derived.** M-RP7.4's `move` needs the identical addressing, so this is paid once.

---

## 4. Leg 3 — the seam element

`.region-split`'s `gap` becomes **0**; a `<div class="region-seam">` is rendered **between** children (`n-1` seams for `n` children).

> **🔒 THERE IS NO LEADING OR TRAILING SEAM.** The perimeter (`--region-pad`) is **not** a seam and can never become one. *A cursor that promises a drag which does not exist is a lie the skin tells.*

**Skin (`skin.css`, all PROVISIONAL, discharger `M-RP-SKIN`):**

- The seam is **`flex: 0 0 var(--region-seam)`** and **transparent** — the split's `--s5` background shows through, so **the hairline looks EXACTLY as it does today.** Do not repaint it.
- **Hit area:** a `::before` overlay, `position:absolute`, expanded by **`--region-seam-hit`** (new, `3px`) along the split's axis only. **The visible seam does not change size.**
- `cursor: col-resize` (row split) / `row-resize` (col split). Drive it off a `data-dir` attribute on the seam.
- > ### ⚠️ **`user-select: none` ON THE SEAM. THIS IS CORRECTNESS, NOT POLISH — READ N-118.**
  > A drag over selectable text leaves a **selection**; the **next** drag presses on that selection, and **Chromium opens a native HTML5 drag that swallows every subsequent `mousemove` AND the `mouseup`.** The tile then sticks to the cursor with nothing to end it. **A real user reproduces this by dragging a splitter twice.** Also set `touch-action: none`.
- **Dead seam** (L4): `data-live="false"` → **no cursor change, no hit expansion, no listeners.** The element **stays** (it is still the visual divider) — this is not painted-dead chrome, because the hairline was never a control affordance. **Only the interaction is absent.**

---

## 5. Leg 4 — the gesture (in `region-node`)

- **`pointerdown`** on a live seam → **`preventDefault()`** (belt to N-118's braces) → **`setPointerCapture(e.pointerId)`**. Capture: the two neighbour elements' rects, and **`--region-min`** read once via `getComputedStyle`.

> ### ⚠️ **`--region-min` DOES NOT EXIST YET, AND THE OBVIOUS WAY TO ADD IT IS A TRAP. READ THIS BEFORE YOU WRITE THE CLAMP.**
> **The number already exists and must not be typed twice: `--region-stripe: 22px`**, whose own comment says *"stripe thickness: height (horizontal) / **width (folded vertical)**"*. **That IS the folded size, and §4.2 says the minimum IS the folded size.** A literal `22px` anywhere in this milestone is a second source of truth (**D-067** — the J-499 drift, and the thing Clair correctly refused at M-RP7.1b).
>
> **🔒 BUT `--region-stripe` IS DECLARED ON `.region-tile`, AND THE SEAM LIVES ON `.region-split` — ITS PARENT.** **Custom properties inherit DOWN, never UP.** So `getComputedStyle(seam).getPropertyValue('--region-stripe')` returns the **empty string**, `parseFloat` gives **`NaN`**, and a careless clamp becomes **no clamp at all** — which looks like it works, right up until a drag squashes a tile to nothing. ***A minimum that silently evaluates to zero is worse than no minimum, because it passes.***
>
> **→ HOIST IT.** Move `--region-stripe` (and only its **declaration**, not its value) from `.region-tile` up to **`.region-shell`**, and add **`--region-min: var(--region-stripe)`** beside it. `.region-tile` still resolves it to `22px` by inheritance — **behaviour-neutral, verify it** — and now the seam can see it too. **ONE number, one place.**
>
> *This is the exact reasoning `skin.css` already applies to `--region-fold-rotate` three lines away* (*"it lives at `:root`, NOT on `.region-tile`, precisely so an override REACHES it — a local default out-specifies an inherited value and silently shadows it"*). **Same trap, opposite direction.**
>
> **⚠️ AND ASSERT THE READ.** If `--region-min` resolves to nothing, **fail loudly** — do not fall back to a literal.
- **`pointermove`** → new boundary px → **snap to `--region-snap`** (new, `8px`, PROVISIONAL) → **clamp so both neighbours stay ≥ `--region-min`. It stops. It does not auto-fold.** Write a transient local `dragSizes` used for the **inline flex of the pair only**. **`node.sizes` is NOT touched.**
- **`pointerup`** → `fraction = a_px / (a_px + b_px)` from the final clamped boundary → `onResize(path, seamIndex, fraction)` → clear `dragSizes` → release capture.
- **`lostpointercapture` / `Escape`** → clear `dragSizes`, **call nothing.** A cancelled drag leaves the descriptor exactly as it was.

**Liveness (L4):** `resolve.ts` **already** decides which children get an inline flex and which are `flex: 0 0 auto` (that is how fold and shrink-wrap work). **REUSE that value. If it is not exported, export it. Do NOT re-derive it** — a second source of truth is the D-067 drift that J-499 killed and that M-RP7.1b's `migrateLayout` nearly repeated.

---

## 6. Leg 5 — shell wiring

`app_client.svelte`: `handleResize(path, i, fraction) { layout = resizeSplit(layout, path, i, fraction) }`, threaded `region-shell → region-node` exactly the way `onFold` already is.

**In memory only.** `session.layout` still has **no writer** until M-RP7.5. Do not add one.

---

## 7. Definition of Done

- [ ] `mutate.ts` + `mutate.test.ts`; **`npm test` count RECORDED, not predicted.**
- [ ] `path` threaded; seam element; gesture; shell wiring.
- [ ] **`user-select: none` + `preventDefault` on the seam, and a drag driven TWICE from the same point without a reload** (N-118's exact reproducer).
- [ ] **`--region-stripe` hoisted to `.region-shell`; `--region-min: var(--region-stripe)` beside it.** **Verify the hoist is behaviour-neutral** — the folded strip still measures **22px** — and **verify the seam can actually READ `--region-min`** (an empty read is `NaN`, and a clamp of `NaN` is no clamp).
- [ ] Clamp exercised: a drag that would push a neighbour below `--region-min` **stops** and **does not fold**.
- [ ] A **shrink-wrapped** split's seam, and a seam beside a **folded-along** tile, are **inert** — proven, not asserted.
- [ ] **Fold still works after a resize**, and a resize after a fold.
- [ ] **`cargo test` 1517/0/62 IDENTICAL** — this is what *proves* zero Rust.
- [ ] **Client registry 67** (quiescent · empty store · no selection · nothing folded · **zero saved UI states** — N-115: one saved state and it reads 71 forever, and 71 is ambiguous).
- [ ] `vite build` module count **recorded**. It will likely rise by one (a new module). **Record it; do not assert it is unchanged.**

**Chat re-drives every non-destructive leg on the real client (Rule 5). Numbers Chat did not personally measure do not enter the record.**

---

## 8. Out of scope — do not drift into these

Persistence (**M-RP7.5**) · `move` / drag-to-dock (**M-RP7.3 / 7.4**) · keyboard resize (**filed**, L7) · the `mergeClasses` dedupe sweep (**N-113 — its own milestone, NEVER a rider**) · appearance tuning (**`M-RP-SKIN`**) · the hole raster (**`M-RP-PLATE` deletes it; do not touch it**).
