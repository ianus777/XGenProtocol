# M-RP6.1e-A — `status-bar` core (sb-cell + separator + resize-grip seam) build runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

For Clair. First step of the **M-RP6.1e client frame consolidation** split (Phase-0 J-488 / D-107 / `docs/xgen-client-frame-phase0.md` §4.5, §6, §10; split locked J-493). Per-component design **locked by Joe** ("lock all by your recomms," this session). This is the pure **`status-bar` core component** — UNLIKE the menu family (6.1d), it IS a catalogued `core` component with a **sampler cell** (it's a data-independent display container the sampler can host; only the grip's real Tauri drag graduates to the real client at 6.1e-B). Design captured here; **no code at lock time** (Rule 1/5).

---

## 1. Goal

Build the `status-bar` `core` component — the fixed bottom-pane strip — as a sampler-testable display container, and wire the resize-grip **seam** (the real `startResizeDragging` wiring is 6.1e-B, real client).

- **`status-bar`** — `<div class="status-bar">`, a **di composite**: side-stacking `sb-cell` groups + `separator`s + an always-visible SE resize-grip.
- Ships with **one real cell per side** (Joe-locked): **left cell = a `status-indicator`**, **right cell = the resize-grip**.
- The component imports **no Tauri / no protocol** — the grip exposes an **`onResizeGrip?` seam** the consuming shell wires (6.1e-B).

## 2. Locked design

### 2.1 Structure (§4.5)

- **Root** `<div class="status-bar" use:envelope>` — di composite. Getter G `{ leftCount, rightCount, hasGrip }` (observable config).
- **`sb-cell`** — `<span class="sb-cell">` wrapper, prop `side` `left` | `right`. Flex, two groups (left group `margin-right:auto` or a spacer, right group trailing). The cell owns positioning; the inner display component owns its look. A cell hosts any display component (`status-indicator` / `label` / `meter` / the grip).
- **Left cell** — hosts a `status-indicator` (composed child, self-registers `__status-indicator` → which itself brings `__led` + `__label`; matrix multiplies — **measure, don't predict**, Rule 5).
- **Right cell** — hosts the resize-grip (§2.3).
- **Separators** — real `separator` (§4.2, `orientation="vertical"`) between cells where more than one cell sits on a side. With one cell per side there may be **zero** separators; the demo can add a second left cell (e.g. a `label`) to exercise a vertical separator — Clair's call, note it.

### 2.2 Font tokens (§4.6, additive, no rename)

Add **below** `--fs-0` in `skin.css` (verify the live scale first, Rule 5 — currently `--fs-0:10px; --fs-1:12px; --fs-2:14px`):

- **`--fs-s1: 9px`**
- **`--fs-s2: 8px`**

The status-bar text (the `status-indicator` caption in the bar) defaults to `--fs-s1`. General L2 tokens (other dense spots may want sub-10 too), not status-bar-only.

### 2.3 Resize grip (§4.5) — glyph + seam, NO Tauri

- A `<span class="sb-grip">` in the right cell, **always visible**, drawing our **own** skinnable SE-corner triangle glyph (NOT the native OS triangle). Glyph mechanism = Clair's call: a pure-CSS corner triangle, or an `icon` (6.1a) with a new SE-resize glyph added to `icons.ts` (prefer `icon` if it's clean — consistent with the icon-adoption direction — but a CSS triangle is fine and lighter). Note which you chose.
- **Seam:** the grip's `onpointerdown` calls `onResizeGrip?.(event)`. The `status-bar` does **nothing** with it (no Tauri import). In the real client (6.1e-B) the shell wires `onResizeGrip` → `startResizeDragging` (SE corner = width+height). *(If a future need is to resize an internal pane rather than the OS window, that's dock-engine splitter territory, M-RP7 — out of scope.)*
- **Accessibility honesty:** the grip is **pointer-only** (resize has no keyboard equivalent here; keyboard/OS window-resize is an OS concern). `aria-hidden` or a plain label; note this plainly — don't fake a keyboard affordance.

### 2.4 Sampler cell (this IS catalogued — unlike the menu family)

- Add a `status-bar` cell to the sampler under the **DI Composites** tab (it's a di composite). Populate it with the real left-`status-indicator` + right-grip layout.
- The **sampler catalogue registry GROWS** here (currently 299) — measure the real delta via CDP, do not predict (Rule 5). This is the opposite of 6.1d (the menu family was frame chrome, not a sampler cell; 299 stayed put). State the new count plainly.
- The grip's `onResizeGrip` seam is **inert in the sampler** (no Tauri) — pass a stub (e.g. a counter/log) so the seam is CDP-observable (pointerdown fires the callback) without a real window drag. The real drag is 6.1e-B.

## 3. Files to touch (indicative — Clair confirms exact paths)

1. `ui/core/…/status-bar.svelte` — new `core` (§2.1); composes `sb-cell` + `separator` + grip; `onResizeGrip?` prop; getter G.
2. `ui/core/…/sb-cell.svelte` — new `core` (or an internal sub-part of `status-bar` if cleaner — Clair's call; if it's a standalone catalogued atomic vs an internal part, note the N-020/N-064 reasoning and whether it registers).
3. `ui/assets/skin.css` — `.status-bar` / `.sb-cell` / `.sb-grip` L2 rules; the `--fs-s1`/`--fs-s2` tokens; status-bar text `--fs-s1`. Confirm real tokens (Rule 5).
4. `ui/assets/icons.ts` — **only if** the grip uses an `icon` glyph (a new SE-resize `d`-string).
5. `ui/sampler/…/app_sampler.svelte` (or the DI-Composites panel) — the `status-bar` cell with left `status-indicator` + right grip + a stub `onResizeGrip`.

**NOT this milestone (defer to 6.1e-B):** mounting the status-bar into the real client bottom pane · the `.state-indicator` → `status-indicator` migration · wiring `onResizeGrip` → `startResizeDragging` · the window-config flips (`resizable:true`, drag-region, default/min size) · center-only scroll · removing the logo + Quit. Those are 6.1e-B (real client). The `dialog` + Help→About is 6.1e-C.

## 4. Verify plan — sampler (D-097; Rule 2, quote real output)

Sampler 9422, both accents via skin-swap:

- **Registry:** the `status-bar` cell present; `count===unique`; 0 orphans both directions; **measure the real total** (299 → N) — the composed `status-indicator` brings its `__led`/`__label` (and `__status-indicator` if that's how it self-registers); state the delta and what each id is.
- **Getter G** exact (`{leftCount, rightCount, hasGrip}` or whatever the final shape is — record it).
- **Structure:** root `DIV.status-bar`; left `sb-cell[side=left]` hosts the `status-indicator`; right `sb-cell[side=right]` hosts `.sb-grip`; `separator` present iff >1 cell on a side.
- **Grip seam:** dispatch a `pointerdown` on `.sb-grip` → the stub `onResizeGrip` callback fires (CDP-observable via the stub's counter/flag). The real `startResizeDragging` is NOT tested here (6.1e-B).
- **Tokens:** `--fs-s1`/`--fs-s2` resolve (9px/8px); the status-bar caption computed `font-size` = 9px.
- **Skin cascade:** `.status-bar`/`.sb-cell`/`.sb-grip` rules in cascade (stylesheet-rule inspection, N-042 method if pseudo-heavy).
- **Accent:** the `status-indicator`'s `link` (if any) rides `--accent2` gold↔blue; the `led` caller-colour + the grip stay accent-neutral (confirm the grip glyph colour is a neutral token, not `--accent`).
- **Eye-check** screenshot (both accents): the strip reads as a thin bottom bar, connection light+label left, grip bottom-right.

## 5. Close (D-074 two-commit)

Clair feat first (code-only: §3 files). Then Chat doc-bridge:
- `ui/docs/xgen-ui-components.md` — **catalogue row(s)** for `status-bar` (+ `sb-cell` if standalone) — this IS a sampler cell, so it's a real catalogue entry (unlike the menu trio). Version bump; note the new sampler registry total.
- `ui/docs/xgen-ui-notes.md` **N-087** (the `status-bar` di-composite / side-stacking `sb-cell` / the resize-grip seam pattern / `--fs-s1`/`--fs-s2` tokens / pointer-only grip honesty / grip glyph mechanism chosen).
- `docs/xgen-client-frame-phase0.md` — if any §4.5 detail was refined during build, refine in-place; else just reference. Version bump only if touched.
- `docs/ROADMAP.md` (M-RP6.1e-A ✅ DONE, vX bump, next-active **M-RP6.1e-B** real-client frame consolidation).
- `CLAUDE.md` PLAY (head → new J-494; the new sampler registry total; next-active 6.1e-B).
- `JOURNAL.md` +J-494 (quote the real sampler CDP + the build + the measured registry delta).
- this task → COMPLETED.

**No new D** expected (D-107 extension). Deferred (D-065): full-edge resize, node app's status-bar. Not pushed — Joe pushes.

## 6. Definition of Done

- [ ] `status-bar.svelte` (`core`, di composite) — side-stacking `sb-cell` groups + `separator` + grip; `onResizeGrip?` seam; getter G.
- [ ] `sb-cell` — `side` left|right; hosts any display component (standalone-or-internal decision noted).
- [ ] Left cell = `status-indicator`, right cell = `.sb-grip` (Joe-locked contents).
- [ ] `.sb-grip` draws the SE-corner glyph (mechanism noted), fires `onResizeGrip?` on pointerdown, pointer-only (a11y honesty noted); NO Tauri import.
- [ ] `skin.css` — `.status-bar`/`.sb-cell`/`.sb-grip` + `--fs-s1:9px`/`--fs-s2:8px` (real tokens confirmed, Rule 5).
- [ ] Sampler DI-Composites cell (left status-indicator + right grip + stub `onResizeGrip`).
- [ ] Sampler CDP green (both accents): registry delta **measured** (299 → N, ids stated), `count===unique`, 0 orphans; getter G exact; structure + side-stacking; grip stub fires on pointerdown; `--fs-s1`=9px resolves; grip accent-neutral; screenshot eye-checked.
- [ ] `vite build` clean — module count quoted.
- [ ] Records bridged (§5), task flipped COMPLETED.

---

*End of M-RP6.1e-A runbook.*
