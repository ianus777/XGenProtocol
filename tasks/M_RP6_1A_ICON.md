# M-RP6.1a — `icon` (core) build runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-09  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

For Clair. First frame prerequisite of the M-RP6.1 client-UI-frame arc (Phase-0 J-488 / D-107 / `docs/xgen-client-frame-phase0.md` §4.3, §6). Per-component design **locked by Joe "go by recomms"** (Chat design walk, this session). `icon` = the **28th `core`** component, a **di** display-kind primitive (no data-dependency — sibling of `label`/`image`/`led`), the **first shape-definition value-type**. Design-lock captured here; no code was written at lock time (Rule 1/5). Registry **286** at handoff — build raises it; **CDP-measure the real new total, do not predict** (Rule 5).

---

## 1. Goal

An inline-SVG, tintable, token-scaled square UI glyph. Cleared vs `image` by D-096 on two axes (value-type = shape definition not `src`; surface = tintable glyph not raster). No `{@html}`, no network, no `.ico`.

## 2. Locked design (D1–D7, by recomms)

- **D1 — API (both, `name` primary).** `name` keys into a small bundled registry (`icons.ts`, `d`-strings). Optional raw `path` override (`string | string[]`) for one-offs. Unknown `name` **and** no `path` → DEV-warn + render empty (`<svg>` with no `<path>`) — the W-13 unknown-id drop precedent (still registers, so the empty state is CDP-observable).
- **D2 — multi-path, no `{@html}`.** A registry entry is a `d` string **or `d[]`**; render `{#each paths as d}<path d={d} />{/each}`. Covers ~all real glyphs (1–3 paths). Arbitrary inner-svg markup (`<circle>`/`<g>`/gradients via `{@html}`) is **deferred** (D-065) — keeps `icon` XSS-free, consistent with the anti-`{@html}` lean (N-032).
- **D3 — prop surface + defaults.**
  - `name?: string` — registry key.
  - `path?: string | string[]` — raw `d` override / escape hatch (wins over `name` if both set).
  - `size?: 16 | 20 | 24` — **default `16`** (first consumers menu-item/status-bar are dense).
  - `tint?: string` — hex or `var(--token)`; **default = `currentColor`** (inherits surrounding text colour). Mechanism: inline `--icon-tint` custom prop, skin reads `fill: var(--icon-tint, currentColor)` (the led/chip/meter inline-var precedent).
  - `label?: string` — **decorative by default** (`aria-hidden="true"`, no `role`); when set → `role="img"` + `aria-label={label}` (a11y-correct: a glyph beside a text label must not double-announce).
  - `id: string`, plus the standard `use:envelope`.
  - **No** `src` / `border-radius` / `disabled` (icons are inert display; interactivity belongs to the consumer's `button`/`link`).
- **D4 — value-type / getter.** New "shape-definition" value-type. Getter **`{ name, size, tint, decorative }`** — `name` (or `'(path)'` when the raw override is used) is the CDP-checkable shape identity; `tint` reports the resolved value (`'currentColor'` when unset); `decorative` = `label == null`.
- **D5 — grid + skin (fill-based).** All glyphs authored on a **24×24 viewBox**, rendered at 16/20/24 via `width`/`height` attributes on the `<svg>`. Glyphs are **fill-based** (Material-style solid shapes) so `fill` tinting works; stroke-based (Lucide-style) deferred until a glyph needs it (D-065 — would add a `stroke` variant then). One `.icon` L2 rule in `ui/assets/skin.css`: `display:inline-block; flex:none; vertical-align:middle; fill: var(--icon-tint, currentColor);` + width/height ride the svg attrs (or a `[data-size]` hook if cleaner). **Component `<style>` stays empty** (skin owns look).
- **D6 — registry home + seed.** `icons.ts` (`{ [name]: d | d[] }`) **co-located in `core`** (same dir as `label.svelte`/`image.svelte`), imported at build (tree-shaken, no runtime fetch). `ui/assets/icons/` holds the **source `.svg`** design files for provenance. Seed **3** demonstrative fill-based glyphs only (D-065) — frame consumers pull their own as they land. Suggested starters (24-grid, fill-based; swap for real Material/Lucide-fill paths if preferred — the mechanism is what 6.1a proves):
  - `caret-down`: `M6 9l6 6 6-6z`
  - `dot`: `M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z`
  - `square`: `M5 5h14v14H5z`
- **D7 — sampler placement.** `icon` is a **di** → the **DI Atomics** panel in `app_sampler.svelte`. Cells:
  - `icon#default` — `name="caret-down"`, size 16, tint unset (inherits).
  - `icon#s16` / `icon#s20` / `icon#s24` — size row (one glyph at each token).
  - `icon#tinted` — `tint="var(--accent2)"` (proves the accent swap gold↔blue).
  - `icon#labelled` — `label="collapse"` (proves `role="img"` + `aria-label`).
  - `icon#raw` — `path="M5 5h14v14H5z"` (raw override, no `name`; getter `name` reports `'(path)'`).

## 3. Files to touch

1. `ui/core/…/icon.svelte` — new component (svg root, `name`→registry lookup / `path` override, `d[]` render, envelope getter G).
2. `ui/core/…/icons.ts` — new registry, 3 seed glyphs.
3. `ui/assets/icons/*.svg` — 3 source svgs (provenance).
4. `ui/assets/skin.css` — one `.icon` L2 rule.
5. `ui/sampler/app_sampler.svelte` — DI Atomics cells (§2 D7).

(Place the component/registry per the existing `core` di convention — mirror `label`/`image`.)

## 4. Verify plan (CDP, sampler 9422, both accents; Rule 2 — quote real output)

Single-line evals, bare `querySelectorAll` + filter in JS (no quoted `[data-debug-id="…"]` selectors).

- **Registry integrity** — `ids()` includes all icon cells; `count === unique`; **0 orphans both directions**; record the **measured** new total + the delta over 286 with cause (Rule 5).
- **Getter G shape** — `{ name, size, tint, decorative }` on `icon#default`; `icon#raw` → `name:'(path)'`.
- **Skin cascade** — `.icon` rule present (stylesheet-rule inspection, N-042 method if computed-style is UA-masked); component `<style>` empty.
- **Element** — root `tag=svg`, `viewBox="0 0 24 24"`, `<path>` count matches the glyph (1 for the seeds).
- **Size union** — `icon#s16`/`#s20`/`#s24` render at 16/20/24 px box (`getBoundingClientRect`).
- **Tint** — `icon#default` computed `fill` resolves to the inherited text colour (currentColor); `icon#tinted` `fill` = `--accent2`, and **swaps gold `#c28840` ↔ blue `#3a7ab0`** on skin-swap (the accent proof); default cell is accent-neutral.
- **a11y** — `icon#default` `aria-hidden="true"`, no `role`; `icon#labelled` `role="img"` + `aria-label="collapse"`.
- **Eye-check** — screenshot: three glyphs render, sizes visibly step, tinted glyph accent-coloured.

## 5. Close (D-074 two-commit)

Clair feat commit first (code-only: the 5 files in §3). Then Chat doc-bridge: `ui/docs/xgen-ui-components.md` (registry vX→vX+1, icon = 28th core, measured count), `docs/ROADMAP.md` (M-RP6.1a ✅ DONE, vX bump), `ui/docs/xgen-ui-notes.md` **N-083** (icon / first shape-definition value-type / registry-keyed / anti-`{@html}` multi-path / fill-based tint), `CLAUDE.md` PLAY (head → new J, next-active M-RP6.1b `separator`), `JOURNAL.md` +J. **No new D** — D-096 already cleared at Phase-0 §4.3. Not pushed — Joe pushes.

## 6. Definition of Done

- [ ] `icon.svelte` authored — svg root, `name`/`path` resolve, `d[]` render, no `{@html}`, size/tint/label props, envelope getter `{name,size,tint,decorative}`.
- [ ] `icons.ts` registry with 3 seed glyphs (24-grid, fill-based).
- [ ] 3 source `.svg` under `ui/assets/icons/`.
- [ ] `.icon` L2 rule in `skin.css`; component `<style>` empty.
- [ ] DI Atomics sampler cells added (§2 D7).
- [ ] `vite build` clean — module count quoted.
- [ ] CDP verify all §4 legs green — real output quoted; new registry total measured + delta cause recorded.
- [ ] Records bridged (§5), task flipped COMPLETED.

---

*End of M-RP6.1a runbook.*
