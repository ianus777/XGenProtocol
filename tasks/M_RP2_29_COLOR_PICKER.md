# M-RP2.29 — `color-picker` (compact themed picker, `#rrggbbaa`)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## Goal
`color-picker` — 24th `core`, 7th di composite. The themeable, compact answer to the native `<input type=color>` picker (native dialog is OS-painted, zero CSS/JS reach past the swatch — N-047, reconfirmed). Combobox-shaped owned-popup (N-063): anchor `textfield` (`__hex`) with a **palette icon** in the chevron slot; expanding reveals a dense picker surface. Passive di, owns only `open`. `bind:value → string` canonical `#rrggbbaa` (8-digit). More capable than native (native emits no alpha). Matrix **+4/cell** (composite + `__hex` + `__hue` + `__alpha`) → **89 → 97** (2 cells; exact count verified at build, Rule 5).

## Locks (all Joe-locked "go by your recomms", J-452 pending)
1. **Structure** — `div.color-picker` (envelope). Passive di, owns `open` only (combobox order). Composes `textfield` (`__hex`) + `range` ×2 (`__hue`, `__alpha`) + own SV surface + own recents grid + eyedropper button. Getter `{ value }` (canonical hex; `model` is view-state, not published).
2. **Value** — `value: string` `$bindable`, canonical `#rrggbbaa` lowercase, always-valued (default `#000000ff`). SV/hue/alpha operate on an internal HSVA source of truth; `value` is derived on commit. All inputs (hex field, RGBA/HSVA fields, sliders, SV, recents, eyedropper) write the internal state → re-derive `#rrggbbaa`.
3. **Anchor row** — `textfield __hex` (editable, type-to-set) + swatch chip + palette icon (masked glyph, N-052; `--pal`/`--pal-open`, swaps/tints on `.color-picker[data-open]`, finger cursor, click focuses/opens — the combobox `.chev` mechanic). Hex field accepts 6-digit → pad `ff`; invalid → `:invalid` skin, no commit.
4. **Popup surface** (dense, ~230px, own skin `.color-picker-pop`):
   - **SV surface** — CSS-gradient `div` (`linear-gradient(to top,#000,transparent), linear-gradient(to right,#fff, hsl(H 100% 50%))`) + absolutely-positioned thumb; pointer x→S, y→V (pointerdown + pointermove capture). NOT `<canvas>` (CDP-readable, N-042).
   - **hue** — `range __hue` 0–360, rainbow track skin.
   - **alpha** — `range __alpha` 0–255, checkerboard-behind-gradient track skin.
   - **model selector** — segmented `HEXA · RGBA · HSVA`, default **HEXA**, local `$state`, reset each mount. Swaps the numeric input row only (surface/sliders identical across models).
   - **numeric row** — HEXA: one `#rrggbbaa` field. RGBA: R G B A (0–255). HSVA: H (0–360) S V (0–100) A (0–255).
   - **eyedropper** — button → `new EyeDropper().open()` (recycled native); result `#rrggbb` keeps current alpha; API absent → button hidden (`'EyeDropper' in window`).
   - **recents** — grid of **8**, most-recent-first, dedup, empty = checkerboard; click sets value. Committed **on close** (the value at close pushes if new). Held local `$state`; persistence (store/TOML) deferred.
5. **Alpha** — units **0–255** (1:1 with `aa` byte). Independent 4th channel appended to RGB; SV/hue math is RGB/HSV only.
6. **Children suffixes** — `__hex` (textfield), `__hue` / `__alpha` (range ×2). Collision-safe vs existing `__field`/`__input`/`__filter`.
7. **a11y** — anchor `role="combobox"` + `aria-expanded`; SV `role="slider"` (2-axis, `aria-label`); model toggle `role="radiogroup"`; recents `role="listbox"`.
8. **Deferred (log, not built)** — colorspace attr; persistence of recents + model; alpha-as-% toggle; keyboard nav on SV surface (pointer-only v1, like combobox has no key subsystem).
9. **Matrix** — cells `#default` (a seeded colour + recents) / `#disabled` (inert, greyed, no open) → **89 → 97**.

## Steps
- **A — component.** `ui/core/lib/components/data-independent/color-picker.svelte`. Fork combobox skeleton (envelope root, `bind:this`, focusin/focusout blur-next-tick, `data-open`, Escape-close). Internal HSVA `$state` + hex⇄rgba⇄hsva conversion helpers (pure, top of `<script>`). Wire children + SV pointer handlers + model swap + eyedropper + recents. Getter `{ value }`.
- **B — skin.** Append `.color-picker*` block to `ui/assets/skin.css` (L2 only): anchor row (reuse `.combobox` metrics), palette icon mask, `.color-picker-pop` popover, SV surface + thumb, `.cp-hue`/`.cp-alpha` range track overrides (rainbow / checkerboard), model segmented control, recents grid, eyedropper button, `:disabled` greyed. No component `<style>`.
- **C — sampler.** Import `ColorPicker` into `app_sampler.svelte`; add `#default` + `#disabled` cells to the DI·composite panel.
- **D — build gate.** `vite build` clean (note module count).
- **E — CDP self-verify** (Chat self-drives, sampler + CDP 9422, both accents, real output — Rule 2). Launch `run-sampler.ps1 -Debug`, poll 9422, retry snapshot until mounted.

## Verify (E — checklist, real output only)
- [ ] `ids().length === 97`; children `color-picker#{default,disabled}__{hex,hue,alpha}` present, no suffix collision.
- [ ] `#default` getter `{ value: "#rrggbbaa" }` (8-digit, lowercase) matches seed.
- [ ] Open on focus → `data-open`, popup mounts (SV + hue + alpha + model + numeric + recents + eyedropper).
- [ ] Set hue via `range __hue` `input` event → value re-derives; SV gradient right-stop tracks hue.
- [ ] Set alpha via `range __alpha` → `aa` byte in value changes.
- [ ] Hex field: 6-digit entry pads `ff`; bad hex → `:invalid`, no commit.
- [ ] Model swap HEXA→RGBA→HSVA reformats numeric row; value unchanged across swaps.
- [ ] Recents: pick sets value; on close a new colour pushes to slot 0, dedup holds; grid caps at 8.
- [ ] Eyedropper present iff `'EyeDropper' in window` (report actual WebView2 result).
- [ ] `#disabled`: `aria-disabled`, no open, greyed; hex field disabled.
- [ ] Accent swap gold `rgb(194,136,64)` ↔ blue `rgb(58,122,176)` on accent-derived chrome; SV/hue/alpha are colour-native (unchanged across shells by design — note it).
- [ ] `.color-picker*` rules in cascade (stylesheet-rule inspection, N-042). Screenshots both accents eye-checked. Teardown 0 orphans on 9422/5175.

## Close (D-074, two-commit atomic) — after E confirmed
1. **feat commit** — `color-picker.svelte` + skin block + sampler cells.
2. **docs commit** — N-066 (ui-notes) + components registry v0.38 (24th `core`, 7th di composite, matrix 97) + ROADMAP v4.21 (M-RP2.29 ✅) + JOURNAL J-452 + CLAUDE PLAY (next-active flip) + this runbook → COMPLETED.
Joe supplies both PowerShell commit+push blocks. Joe pushes.

## DoD
- [ ] Component + skin + sampler built; `vite build` clean.
- [ ] All Verify(E) items CDP-confirmed with real output.
- [ ] N-066 + registry v0.38 + ROADMAP v4.21 + J-452 + CLAUDE PLAY written atomically.
- [ ] Runbook `Status: COMPLETED`.
- [ ] Next-active flipped → `widget` definition (N-059→spec) → M-RP4.3 → M-RP4.1.
