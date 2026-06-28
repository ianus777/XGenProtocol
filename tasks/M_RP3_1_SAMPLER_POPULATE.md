# M-RP3.1 — Populate the Sampler (class×state matrix + polished skin-swap)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-27  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Goal

Turn the M-RP3.0 scaffold (one `button#smoke`) into the actual tuning surface: all **10 built `core` components** mounted live in a **semantic-group × state** grid, each cell a real `envelope`-registered instance, with a polished client↔node skin-swap. This is what makes the sampler usable for tuning `date` and everything after (D-097). Frontend-only — the `xgen-sampler` crate is untouched.

## Locks (design walk, Joe-locked 2026-06-27 "go by your recomms")

1. **IA = semantic-group × state, not class×phase.** Phase-0: all 10 are **di·A** today (no dd, no Phase B/C), so N-028's class×phase axes are degenerate. v1 groups by **Interactive** (toggle, button, textfield, select, textarea, number, range) and **Display** (label, paragraph, image); each component is a row, its **applicable** states are cells across. Class/phase columns activate later when dd/B/C exist.
2. **Ragged state-map (honest, not a forced uniform grid):**
   - **default** — all 10
   - **disabled** — the 7 interactive only (display-di have no disabled)
   - **invalid** — only `textfield` (bad email) + `number` (out-of-range); the rest have no `:invalid` in their skin and get no faked column
   - **teaching variants** — `toggle` checked, `textfield` password (icon), `textarea` multi-line value
3. **No focus column.** Focus is transient; a static focus cell would be a lie. The focus ring is verified live/CDP, not exhibited in the grid.
4. **Live instances, not snapshots.** Every cell is a real mounted component (type/toggle/drag), registered via `envelope` as **`{type}#{state}`** (e.g. `toggle#disabled`, `number#invalid`, `textfield#password`) so CDP `ids()` enumerates the matrix.
5. **Skin-swap = polished segmented control, kept as TOOL CHROME.** Replace the bare button with a `client | node` segmented control in the sticky bar, styled in the sampler's own `<style>`/`app.css` — NOT a sampled `core` component (preserves the N-028 tool-chrome vs sampled-component line). It flips `:root[data-shell]`; with accent-prominent components now present (toggle `accent-color`, focus rings), the swap visibly re-themes (the two shell screenshots genuinely differ — unlike the smoke-only scaffold).
6. **Scope guard.** OUT (deferred growth): prop editors / per-instance controls, A/B/C phase columns, search/filter, standalone-exe live-reload. v1 is the tuning surface, not an IDE.
7. **No `DECISIONS.md` touch** — no new principle; this applies D-097/D-098/N-028.

## The matrix (instance list — ~22 live cells)

**Interactive**
- `toggle#default` (unchecked) · `toggle#checked` · `toggle#disabled`
- `button#default` · `button#disabled` · `button#toggle` (mode=toggle, shows the pressed latch)
- `textfield#default` (type text, seeded value) · `textfield#disabled` · `textfield#invalid` (type=email, bad value → real `:invalid`) · `textfield#password` (type password, reveal/lock icon)
- `select#default` (options + placeholder + value) · `select#disabled`
- `textarea#default` (multi-line `\n` value) · `textarea#disabled`
- `number#default` (seeded value) · `number#disabled` · `number#invalid` (value outside `[min,max]` → `--err`)
- `range#default` (value 50, min 0 max 100) · `range#disabled`

**Display**
- `label#default` · `paragraph#default` · `image#default` (small bundled asset or data-URI placeholder — confirm an `$assets` image exists at implement; else inline data-URI)

(Exact prop surfaces re-read per component at implement before wiring — `value`/`checked` bindings, `options` shape, `min`/`max`/`step`, `text`, `src`/`alt`, `mode`/`pressed`, `type`.)

## Phases

**Phase 1 — layout** (`ui/sampler/src/app.css`): replace the v0 `.sampler-smoke` block with a grid — section headers (Interactive / Display), a row per component, cells laid out with their `{type}#{state}` id label above/beside each live instance. Sticky `.sampler-bar`. Segmented-control styles (`.sampler-seg`, active state) as tool chrome. No new tokens; consume the skin.

**Phase 2 — matrix** (`ui/sampler/src/app_sampler.svelte`): rewrite v0 → import all 10 from `$core/components/data-independent/*`; mount each instance per the list above with its seeded props + `id`; bare `$state` for each bound value (plain-JS shell, N-041). Replace the bare swap button with the segmented `client | node` control (active-state reactive, flips `document.documentElement.dataset.shell`, default client).

**Phase 3 — image asset:** confirm a usable image in `ui/assets` (logos) or drop a tiny inline data-URI into `image#default` so the cell renders without a network fetch.

**Phase 4 — CDP verify (Chat self-drives, real `tauri dev` + CDP 9422):**
- `run-sampler.ps1 -Debug` → boots; `__XGEN_DEBUG__.ids()` lists **all ~22** `{type}#{state}` ids (assert count + spot-presence of `toggle#disabled`, `number#invalid`, `textfield#password`).
- **disabled** spot-check: computed-style on `number#disabled` / `toggle#disabled` shows the disabled treatment (greyed / `cursor:not-allowed` / reduced opacity per skin).
- **invalid** spot-check: `number#invalid` + `textfield#invalid` border-color = `--err`.
- **skin-swap**: flip `data-shell` client↔node → `--accent` gold↔blue on a toggle; screenshot diff is real (accent-prominent cells change).
- **full-matrix screenshots** both shells (client + node) — eye-check the grid renders, states read correctly, accent re-themes.
- clean teardown (5175/9422 free, 0 orphans).

**Phase 5 — records (D-074):**
- `ui/docs/xgen-ui-notes.md` **N-045** — sampler populated: the semantic-group×state IA (class/phase deferred), the ragged state-map + no-focus-column rationale, live `{type}#{state}` instances, the segmented skin-swap, the matrix verify.
- `docs/ROADMAP.md` — M-RP3.1 ✅; frontier M-RP3.0→M-RP3.1; version bump (v4.02).
- `CLAUDE.md` PLAY → M-RP3.1 ✅ CLOSED; Next = resume component track (`date`) in the sampler; pointer J-422→J-423.
- `ui/docs/xgen-ui-components.md` — one line: sampler populated with all 10 (live matrix).
- `tasks/M_RP3_1_SAMPLER_POPULATE.md` → COMPLETED.
- `JOURNAL.md` **J-423** (written last, real CDP output quoted).
- Two commits (implementation: app_sampler.svelte + app.css [+ asset]; then records-only). Joe pushes.

## Definition of Done

- [x] `app.css`: semantic-group×state grid + section headers + segmented-control (tool-chrome) styles; sticky bar; no new tokens
- [x] `app_sampler.svelte`: all 10 components imported; 22 live `{type}#{state}` instances per the matrix list with seeded props; segmented `client|node` skin-swap (default client) replacing the bare button
- [x] `image#default` renders from an inline data-URI (no network fetch)
- [x] CDP: `ids()` lists all 22 instances (exact matrix list confirmed in J-423)
- [x] CDP: disabled cells `cursor:not-allowed`; `number#invalid` + `textfield#invalid` border = `--err` `rgb(138,42,42)` (default stays `--s5`)
- [x] CDP: skin-swap flips toggle `accent-color` `rgb(154,106,48)`↔`rgb(42,96,144)` / `--accent` `#9a6a30`↔`#2a6090`; client + node full-matrix screenshots differ
- [x] clean teardown (5175/9422 free, 0 orphans)
- [x] records (N-045, ROADMAP v4.02 M-RP3.1 ✅, CLAUDE PLAY, components note, JOURNAL J-423, task COMPLETED) — no DECISIONS touch

**Finding (verify):** `toggle` has **no `disabled` prop** (only `checked`/`id`/`shape`) — `toggle#disabled` was replaced with **`toggle#switch`** and logged (N-045) as an atomic gap for the di resume. Final matrix = 22 cells (not 23).
