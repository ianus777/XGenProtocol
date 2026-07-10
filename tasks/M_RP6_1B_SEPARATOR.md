# M-RP6.1b — `separator` (core) build runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

For Clair. Second frame prerequisite of the M-RP6.1 client-UI-frame arc (Phase-0 J-488 / D-107 / `docs/xgen-client-frame-phase0.md` §4.4, §6). Per-component design **locked by Joe "go"** (Chat design walk, this session). `separator` = the **29th `core`** component, a **di** display-kind primitive (no data-dependency), and the **first value-less component** in the library (getter is config-only). One component, used **everywhere** — the menu-divider and the status-bar cell-divider are the same thing (Phase-0 §4.4 "built once"; D-096 fold cleared, **no new D**). Registry **293** at handoff — build raises it; **CDP-measure the real new total, do not predict** (Rule 5).

---

## 1. Goal

A pure visual divider `<div role="separator">` with **zero content, no value, no binding, no interaction**. Orientation `horizontal|vertical`; appearance `line|double|gap` — **all appearance (thickness, style, colour) lives in `skin.css`** (L2); component `<style>` empty.

## 2. Locked design (Joe "go")

- **Root — `<div role="separator">`.** NOT `<hr>`. Chosen deliberately so the *same* component is valid both in the flex status-bar AND as a direct child of a future `<ul role="menu">` (an `<hr>` is not a valid `<ul>` child; a `role="separator"` div is) → one root, every context, no branch ever. `use:envelope`, `id`.
- **Orientation.** `orientation?: 'horizontal' | 'vertical'`, default **`horizontal`** (canonical). Reflected to **both** `data-orientation` (skin hook) and `aria-orientation` (a11y).
- **Variant.** `variant?: 'line' | 'double' | 'gap'`, default **`line`**. Reflected to `data-variant` (skin hook).
- **Prop surface (leanest in the library).** `orientation?` · `variant?` · `id`. **No** value / binding / label / interaction / inline-style / tint / thickness props — every visual is skin-owned.
- **Getter G — `{ orientation, variant }`** (config-only; the first value-less getter).
- **Appearance = `skin.css` only (border-based).** A `<div>` draws its rule via a **border** (not `background`) — because `border-style: double` gives the two-line rule natively (thickness ≥3px shows line/gap/line); `background` can't express `double`. Colour is a skin token (`--s5` or the live hairline token — **confirm against the real `skin.css`, Rule 5**), carried inside the border shorthand alongside thickness + style. Per orientation: horizontal → `border-top`; vertical → `border-left`. `gap` → `border:0` (pure spacing, box still present). Spacing *around* the separator is the **consumer's** concern (status-bar cell layout / menu), not this component's.

  Locked skin block (adapt token/px to the real `skin.css`, keep the shape):
  ```
  .separator { border: 0; box-sizing: border-box; }
  .separator[data-orientation="horizontal"] { width: 100%; }
  .separator[data-orientation="vertical"]   { align-self: stretch; }

  .separator[data-variant="line"][data-orientation="horizontal"]   { border-top:  1px solid  var(--s5); }
  .separator[data-variant="line"][data-orientation="vertical"]     { border-left: 1px solid  var(--s5); }
  .separator[data-variant="double"][data-orientation="horizontal"] { border-top:  3px double var(--s5); }
  .separator[data-variant="double"][data-orientation="vertical"]   { border-left: 3px double var(--s5); }
  /* data-variant="gap" → base rule only, border stays 0 */
  ```
  Note (recorded, not a defect): with `border-style: double` the token paints **both** lines and the middle gap shows the panel background behind it (transparent) — a distinctly-coloured middle gap is a two-layer technique, deferred (D-065).

## 3. Files to touch

1. `ui/core/…/separator.svelte` — new component (`<div role="separator">`, `orientation`/`variant` props → `data-*` + `aria-orientation`, envelope getter `{orientation,variant}`, **empty `<style>`**). Mirror an existing `core` di (`label`/`icon`) for placement + envelope convention.
2. `ui/assets/skin.css` — the `.separator` L2 rules (§2). Confirm the real hairline colour token + base px (Rule 5).
3. `ui/sampler/app_sampler.svelte` — DI Atomics cells (§4).

## 4. Sampler cells (DI Atomics panel)

- `separator#horizontal` — default (`line`, horizontal), in a block context (width fills).
- `separator#vertical` — `orientation="vertical"`, placed **inside a short flex row** (e.g. two labels with the separator between) so `align-self:stretch` has a height to stretch to.
- `separator#double` — `variant="double"` (horizontal) — the two-line rule.
- `separator#gap` — `variant="gap"` — pure spacing, no visible line.

## 5. Verify plan (CDP, sampler 9422, both accents; Rule 2 — quote real output)

Single-line evals, bare `querySelectorAll` + filter in JS (no quoted `[data-debug-id="…"]` selectors).

- **Registry** — `ids()` includes the 4 separator cells; **measured** new total + delta over 293 with cause; `count===unique`; **0 orphans both directions**.
- **Getter G** — `{orientation, variant}` on each cell (`#horizontal` `{horizontal,line}`, `#vertical` `{vertical,line}`, `#double` `{…,double}`, `#gap` `{…,gap}`).
- **Element** — root `tag=div`, `role="separator"`, `aria-orientation` present + correct on both; `data-orientation`/`data-variant` reflected.
- **Skin cascade** — `.separator` rules present (stylesheet-rule inspection, N-042 method if computed-style is masked); component `<style>` empty.
- **Variant looks (computed-style)** — `#double` computed `border-top-style: double` + width ≥3px; `#gap` `border-*-width: 0`; `#horizontal` `border-top-style: solid` 1px; `#vertical` draws on `border-left`.
- **Accent-neutral** — separator colour is `--s5`-derived chrome; confirm it does **NOT** swap gold↔blue on skin-swap (the `led`/`meter` no-accent precedent) — computed border-colour identical client↔node.
- **Eye-check** — geometry-covered (double shows two lines, gap shows none, vertical stretches to row height); screenshot only if the harness cooperates (flaky path, optional).

## 6. Close (D-074 two-commit)

Clair feat first (code-only: the 3 files in §3). Then Chat doc-bridge: `ui/docs/xgen-ui-components.md` (registry vX→vX+1, separator = 29th core, measured count), `docs/ROADMAP.md` (M-RP6.1b ✅ DONE, vX bump, next-active 6.1c), `ui/docs/xgen-ui-notes.md` **N-084** (separator / leanest di / first value-less component / `<div role=separator>` root chosen for menu+status-bar reuse / border-based double via skin / accent-neutral chrome), `CLAUDE.md` PLAY (head → new J, next-active M-RP6.1c `Accelerator`), `JOURNAL.md` +J. **No new D** — D-096 fold cleared at Phase-0 §4.4. Not pushed — Joe pushes.

## 7. Definition of Done

- [ ] `separator.svelte` authored — `<div role="separator">`, `orientation`/`variant` → `data-*`+`aria-orientation`, getter `{orientation,variant}`, empty `<style>`.
- [ ] `.separator` L2 rules in `skin.css` (border-based; real hairline token confirmed).
- [ ] 4 DI Atomics sampler cells added (§4).
- [ ] `vite build` clean — module count quoted.
- [ ] CDP verify all §5 legs green — real output quoted; new registry total measured + delta cause recorded; accent-neutral confirmed.
- [ ] Records bridged (§6), task flipped COMPLETED.

---

*End of M-RP6.1b runbook.*
