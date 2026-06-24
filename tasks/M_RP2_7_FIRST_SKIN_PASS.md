# M-RP2.7 — First skin pass (N-031 CSS source stack + L2 vocabulary founding)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-24  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Goal

Stand up the N-031 CSS source stack and found the L2 token+treatment vocabulary; skin the three built components (`toggle` / `button` / `textfield`) + shell chrome; close the N-028/N-029 global `button{}` / `input{}` wrinkle. **One shared `skin.css`, per-shell accent** (Q2-locked).

This is a **vocabulary-founding pass**, not a quick three-component skin (N-031 saturation): get the L2 primitives right once so later components assemble from them.

## Locked design walk (J-411 discussion → this runbook)

- **Q1** ✅ skin home `ui/assets/`; new `$assets` Vite alias; relocate `modern-normalize.css` into `ui/assets/`.
- **Q2** ✅ one shared `skin.css` for BOTH apps; per-shell `--accent*` alias set in each `app.css`.
- **Q3** ✅ wire the local pristine `modern-normalize.css` as L0 (first import, both shells); drop the hand-rolled `app.css` reset.
- **Q4** ✅ chrome-boundary allocation per the table below; `#core-ui-pane` look stays chrome for pass 1.
- **Q5** ✅ LOCKED skin-only (J-412 eye-check) — switch via `appearance:none` + `::before` thumb, `:checked` drives `translateX`; renders cleanly as a pill+thumb in both apps; `toggle.svelte` stays `<style>`-free, L1 empty.
- Focus ring ✅ accent-tinted · Seat ✅ Ms Design leads (mechanical wiring rides along) · ✅ `-Mode screenshot` added to `cdp-debug.ps1`.

## Phase-0 audit findings (state at open, 2026-06-24)

- `modern-normalize.css` is NOT wired into either live shell (only the retired `ui/templates/dev_core_ui/svelte/main.js` references the npm package). Live shells import only `./app.css`. Local pristine copy unreferenced at `ui/modern-normalize.css` (v3.0.1).
- Both `ui/{client,node}/src/app.css` are near-identical: same reset, byte-identical `:root` token block, same chrome, same generic `button{}` appearance. Only divergence: `button.primary-client-button`→`--pr` (gold) vs `button.primary-node-button`→`--inf` (blue).
- Those `primary-*-button` rules are DEAD (no button carries the class). Live accent difference today is only the state-dot (`dotColor` → `--pr`/`--inf`), drawn from shared tokens.
- Three components carry ZERO `<style>` blocks (skin-only, confirmed in source comments).
- Vite aliases `$common`→`../common/lib`, `$core`→`../core/lib` confirmed in both shells. No `$assets`.
- `ui/assets/` holds fonts + logos only (no CSS). Fonts also duplicated per-shell at `ui/{client,node}/src/assets/`; `@font-face` points at the per-shell copy.

## Chrome-boundary allocation (Q4)

| Rule(s) | Destination |
|---|---|
| `*{box-sizing/margin/padding}`, `main{display:block}`, `p{}`, `img{}` floor, `button{appearance:none;background:transparent;border:none;font:inherit;color:inherit;cursor;text-align}` | **xgen-normalize.css (L0)** |
| `:root` semantic palette, `@font-face`, `html,body{font-family/background}`, `.button`/`.toggle`/`.textfield` appearance, hover/pressed/focus/disabled/invalid/switch treatments, accent fill | **skin.css (L2)** |
| `body` centering, `#core-ui-pane`, `.state-indicator`/`.state-dot`/`.pulse`/`@keyframes dot-pulse`, `.button-pane`, `img#app-logo`, per-shell `:root{--accent*}` | **app.css (shell chrome)** |

Wrinkle fix: generic `button{…appearance…}` re-keys to **`.button`** (type-class) so bare `<button>` no longer inherits the skin — closes N-028 finding 2 / N-029. Dead `primary-*-button` rules retire into the `--accent` button treatment. State-dot stays on semantic tokens (NOT accent) — a degraded node is amber regardless of brand.

---

## Phase 1 — Infrastructure (mechanical)

Each file op `Filesystem:get_file_info`-verified before the next (no prose-then-batch). `Filesystem:edit_file` with `dryRun:True` before any live edit.

1. **Relocate** `ui/modern-normalize.css` → `ui/assets/modern-normalize.css` (`Filesystem:move_file`). Pristine, never edited.
2. **Create** `ui/assets/xgen-normalize.css` (L0 adapted floor): migrate the reset from app.css — `*{box-sizing/margin/padding}`, `main{display:block}`, `p{}`, `img{}` floor, `button{appearance:none;background:transparent;border:none;font:inherit;color:inherit;cursor;text-align}`. Deviations-from-upstream recorded in-file.
3. **Create** `ui/assets/skin.css` (L2) — content per Phase 2.
4. **Add** `$assets`→`../assets` alias to both `ui/{client,node}/vite.config.js`.
5. **Rewire** both `main.js` import chain, in order:
   `$assets/modern-normalize.css` → `$assets/xgen-normalize.css` → `$assets/skin.css` → `./app.css` → `App`.
6. **Gut** both `app.css` to chrome only (per the allocation table) + add the per-shell accent block:
   client `:root{--accent:var(--pr);--accent2:var(--pr2);--accent-ink:var(--pr-ink)}` · node `--inf*`.

## Phase 2 — Vocabulary founding (the weight; Ms Design, taste)

In `skin.css`, define-once then assemble:

- **Tokens:** semantic palette (`--pr*`/`--inf*`/`--ok`/`--err` — moved here, canonical), `@font-face` + `html,body` typography/surface, radius/spacing scale, `--accent*` consumed (set per-shell, not here).
- **Treatments:** focus ring (accent-tinted), disabled-grey, `:invalid`→`--err`, pressed bevel (`.button[aria-pressed="true"]` toggle-mode + momentary `:active`), switch (`.toggle[role="switch"]` `appearance:none` + `::before` thumb + `:checked` translateX), accent button fill/hover via `--accent`.
- **Component keys:** `.button` (re-keyed off bare `button` — the wrinkle fix), `.toggle`, `.textfield`.

## Phase 3 — Verify (Chat self-drives per N-028 working mode)

- **Wrinkle-clearance (mechanical, CDP `getComputedStyle`):** classless `<button>` reads normalize-flat; `.button` reads skinned; assert modern-normalize stylesheet loaded.
- **`-Mode screenshot`** added to `cdp-debug.ps1` (`Page.captureScreenshot`); capture both apps.
- **Eye-check (Joe):** both apps coherent; **Q5 switch-shape verdict** (skin-only stands, or L1 scaffold needed).
- Confirm zero L1 regressions in the three components; clean teardown (ports 9222/9322/5173/5174, no orphans).

## Phase 4 — Close (J-412, records-only)

- New `ui/docs/xgen-ui-notes.md` N-033 (skin vocabulary founded).
- `ui/docs/xgen-ui-components.md` skin notes on the three components.
- `docs/ROADMAP.md` RP node: M-RP2.7 ✅, frontier advance, version bump.
- `CLAUDE.md` PLAY: M-RP2.7 ✅ CLOSED, Next → `select` (di·A); entry pointer J-411→J-412.
- This task file Status ACTIVE→COMPLETED, DoD checked.
- Joe pushes.

---

## Definition of Done

- [x] N-031 stack wired: modern-normalize (L0, relocated) → xgen-normalize (L0) → skin.css (L2) → app.css (chrome), both shells, correct import order.
- [x] `$assets` alias added to both vite configs.
- [x] Wrinkle closed: bare `<button>` normalize-flat vs `.button` skinned, verified by computed-style probe.
- [x] Three components + shell chrome skinned coherently; one shared skin, per-shell `--accent*`.
- [x] Switch shape eye-verified (Q5 final verdict recorded); L1 still empty OR minimal scaffold justified.
- [x] Both apps visually eye-verified (Joe) + screenshots captured.
- [x] J-412 records landed (N-033, components registry, ROADMAP, PLAY, this file COMPLETED).
