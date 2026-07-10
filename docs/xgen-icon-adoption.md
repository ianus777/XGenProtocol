# M-RP-ICON-ADOPT — Icon adoption / glyph consolidation (milestone, theory-open)
> **Status**: PENDING  
> Version: 0.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-09  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Backlog milestone (not the current task — parked behind the M-RP6.1 frame arc). Goal: retire the inline hardcoded field glyphs in favour of the `icon` component (M-RP6.1a) + its registry, so every glyph is **one system**: tint-skinnable, CDP-inspectable, single provenance. **Theory is deliberately unresolved below** — the milestone shape is provisional pending Joe's decisions on §3.

Milestone number **TBD** — candidate slot after the frame arc / live-wiring (M-RP6.x). Wants its own D-071 Phase-0 (touches ~6 existing field components).

---

## 1. Inventory (embedded glyphs to consolidate)

Currently inline `<svg>` inside their field components:

| Glyph | Home component | Likely style | State |
|---|---|---|---|
| mail envelope | email text-field | outline (stroke?) | static |
| calendar | date-picker | fill/stroke? | static |
| clock | time-picker | fill/stroke? | static |
| gear | (settings) | fill, multi-path | static |
| palette | color-picker | **multi-colour** | static |
| eye / eye-off | password-field | stroke? | **2-state toggle** |
| file / browse | file-field | fill/stroke? | static |
| chevron + caret-down | combo-box | **CSS triangle (`--tri`), not svg** | static |

Exact styles are a Phase-0 audit output, not assumed here.

## 2. What's already solved vs what isn't

- **Tint (colour): solved.** Any glyph routed through `icon` inherits `fill: var(--icon-tint, currentColor)` — colour-skinnable the moment a field renders `<Icon>` instead of an inline `<svg>`. No new mechanism needed.
- **Shape (geometry): the open question.** Where does the path data live, and can a *theme* redraw it — see §3.

## 3. Theory — the aspects to think through

### 3a. Shape-skinnability: three technical routes
- **(A) Registry `d`-strings (current 6.1a model).** Shape lives in `icons.ts`; component renders `<path d=…>`. Pro: shape = content, matches the L2 rule ("skin owns appearance, not content"); CDP-inspectable; tree-shaken. Con: a theme can't swap the *glyph itself* from skin.css (only its colour).
- **(B) CSS `mask-image` (data-URI glyph + `background: var(--tint)`).** Glyph as a mask in skin.css; colour via background. Pro: a **theme could ship its own glyph set** entirely in skin.css. Con: not real DOM `<path>` (no per-path control, weaker CDP story), pushes *content* into skin.css.
- **(C) CSS `d: path("…")` property (Chromium/WebView2 supports it).** A `<path>` whose geometry is overridden from CSS. Pro: real element + theme-swappable geometry. Con: same L2 violation (shape in skin.css); niche, brittle across path formats.

**Crux question for Joe:** *Is per-theme glyph replacement an actual goal?* If **no** → **(A)** wins outright (shape stays registry/content, skin.css stays appearance-only). If **yes** (a theme can reskin the whole icon set, not just recolour) → **(B)** is the only clean route, and it means consciously relaxing the "skin.css = appearance only" line for glyphs. My lean is **(A)** — consistent iconography + themeable colour is almost always what's wanted; whole-glyph theme swaps are rare and costly — but this is yours to settle.

### 3b. Multi-colour glyphs break the single-tint model (biggest aspect)
The palette / colour-picker icon is inherently multi-hued. `fill: var(--icon-tint)` forces one colour. Options:
- **(i)** Registry entries may carry per-path baked fills; `--icon-tint` applies only to paths that *don't* specify one. Keeps them in `icon`, but they're no longer purely tintable.
- **(ii)** Multi-token glyphs: each path → its own `var(--icon-slotN)`. Powerful, more complex.
- **(iii)** **Re-open D-096:** a genuinely pictorial multi-colour mark may be an **`image`, not an `icon`** — the exact axis D-096 draws. The palette icon might simply belong to `image`. Worth deciding per-glyph, not globally.

### 3c. Stroke vs fill
`icon` is fill-only today (stroke variant deferred, D-065 / N-083). Several field glyphs (eye, outline mail) are likely stroke-based. Adoption forces the **stroke-variant** decision: add a `stroke` mode to `icon` (`stroke: var(--icon-tint); fill:none`), or re-author those glyphs as fill. Phase-0 classifies each; the extension (if any) locks before adoption.

### 3d. Stateful glyphs (eye/eye-off)
Trivial but note the pattern: two registry entries (`eye` / `eye-off`), and the **component** swaps `name` off its own state — the skin never toggles state. Same pattern for any future 2-state glyph.

### 3e. Chevron/caret reconciliation
Combo-box draws its triangle via a **CSS `--tri` triangle**, not an svg glyph. Options: fold into the registry as real glyphs for one unified system, or leave the CSS triangle as-is (it's cheap and not really an icon). Decide fold-vs-leave; not free either way.

### 3f. Provenance / licence (audit gate)
BSL→GPL means every shipped glyph needs a clean licence. Where did the 8 embedded svgs originate? A permissive set (Lucide MIT, Material Apache-2.0, Heroicons MIT) is fine **with attribution**; unknown-origin glyphs must be re-sourced or re-drawn. This is a hard Phase-0 gate, not optional.

## 4. Provisional milestone shape (pending §3)

- **Phase-0 (D-071 audit):** enumerate every embedded glyph across field components; classify each fill/stroke/multi-colour; licence-source each (§3f); decide per-glyph **icon-vs-image** (D-096, §3b); resolve §3a–3e. Output: the locked adoption spec + any `icon` extensions.
- **Phase-1:** extract glyphs into `icons.ts`; land any Phase-0-approved `icon` extensions (stroke mode / multi-fill).
- **Phase-2:** per-field adoption — mail · date · time · gear · palette · file · eye — field-by-field, each CDP-verified against its prior render.
- **Phase-3:** chevron/caret reconcile (only if §3e = fold).
- **Gate:** starts after the frame arc is functional; not before.

## 5. Explicit open questions for Joe

1. Per-theme glyph replacement — a goal? (decides §3a A-vs-B).
2. Palette/multi-colour marks — `icon` with baked fills, multi-token, or reclassify as `image`? (§3b).
3. Add a `stroke` mode to `icon`, or re-author stroke glyphs as fill? (§3c).
4. Fold the combo-box CSS triangle into the registry, or leave it? (§3e).
5. Provenance of the 8 current glyphs — known-permissive, or need re-sourcing? (§3f).

## 6. Non-goals / deferred

- No CSS-side shape (routes B/C) unless §3a Q1 = yes.
- No work starts before the frame arc is functional.
- `temperature-indicator` and other post-frame widgets are separate (M-RP6.5).

---

*Draft — theory open. Milestone number + phase locks pending Joe's §5 answers.*
