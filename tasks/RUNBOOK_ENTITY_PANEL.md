# Runbook — M-RP5.2 `entity-panel` (roving-focus dd-composite)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Build runbook for `entity-panel` — the last dd-composite. Design locked in Phase-0 (`docs/xgen-dd-spaces-panel-phase0.md` v1.1, A–H). Session-open order applies (Rule 0). Skin path is `ui/assets/skin.css`. No push — Joe pushes.

---

## Locked design (A–H)

- **A** name `entity-panel` (`spaces-panel` = consumer preset).
- **B** wrap `section` (root `<section>`), body `<ul role="listbox">`.
- **C** roving tabindex + listbox + arrows/Home/End/Enter/Space; `selected`→active row.
- **D** single-select `bind:selected` (id); click + keyboard. Multi deferred.
- **E** empty → in-body message; loading deferred.
- **F** `collapsible?`/`collapsed?` pass-through to `section`.
- **G** getter `{ count, selected, collapsed, hasEmpty }`.
- **H** avatar corner-fix: status bottom-right, `isAi` spark → **top-right**.

## Step 1 — avatar corner-fix (H, do first)

- `ui/assets/skin.css`: move the `entity-avatar` `isAi` spark `::after` from bottom-right → **top-right**; status corner-slot stays bottom-right. Additive skin-only; no `.svelte` change.
- Re-verify avatar/item/status cells 0-regression (isAi + status now non-overlapping).

## Step 2 — `entity-panel` component

- `ui/core/lib/components/data-dependent/entity-panel.svelte`:
  - props: `items: EntityDescriptor[]` (+ per-item `secondary?`/`status?`/`meta?` — an `EntityItemInput[]` view-model), `title?`, `badge?`, `collapsible?`, `collapsed?` ($bindable), `selected?` ($bindable, id), `onActivate?`, `emptyText?`, `id`.
  - root composes `section` (title/badge/collapse pass-through); body `<ul role="listbox">` of `entity-item` rows (`variant="row"`), each `role="option"` `aria-selected`.
  - roving tabindex: `tabindex=0` on active, `-1` others; ArrowUp/Down move, Home/End jump, Enter/Space → `onActivate(id)`; click selects+activates.
  - empty (`items.length===0`) → `emptyText` message in body (compose di `paragraph` or `label`).
  - children self-register (`section` `__section`, each row `entity-item#<panelid>-<itemid>` + its `__avatar`/`__status`); getter G via `use:envelope`.
- DoD: no protocol import; single tabstop; selection + keyboard nav verified.

## Step 3 — skin

- `.entity-panel` in `ui/assets/skin.css`: `<ul>` reset (no bullets/margin), row focus ring on `[aria-selected]`/`:focus-visible`, empty-message muted. Width rule (N-076) inherited via rows. PROVISIONAL.

## Step 4 — sampler (DD·composite panel)

- Cells: a spaces preset (rounded-square avatars) + a DMs preset (circle avatars, one with status badge); an empty-panel cell; a collapsed cell. Deterministic `items` arrays.
- Stable ids; verify child registration multiplies.

## Step 5 — CDP verify (sampler 9422)

- `vite build` clean; kill zombies + confirm served module (N-058).
- Assert: getter G; `role=listbox`/`option`; roving tabindex (one `0`, rest `-1`); Arrow/Home/End move active; Enter → `onActivate`; `bind:selected` reflects `aria-selected`; empty cell shows message + `hasEmpty:true`; collapse hides rows (slot mounted); child rows + `__avatar`/`__status` registered; corner-fix (isAi top-right, status bottom-right, no overlap); registry delta; **0 orphans**. Quote real output (Rule 2). Screenshot.

## Step 6 — D-074 atomic close (one commit)

- `ui/docs/xgen-ui-notes.md` → N-078 (`entity-panel` + corner-fix), v-bump.
- `ui/docs/xgen-ui-components.md` → registry v0.50 (`entity-panel` row; avatar corner-fix note), v-bump.
- `docs/ROADMAP.md` → M-RP5.2 ✅ DONE, v-bump.
- `docs/xgen-dd-spaces-panel-phase0.md` → COMPLETED.
- `CLAUDE.md` PLAY → J-465.
- `JOURNAL.md` → J-465 (last, real CDP output).
- this runbook → COMPLETED, v-bump.
- No DECISIONS touch (arc-local).

---

## Definition of Done

- [ ] avatar corner-fix (H): isAi top-right, status bottom-right, 0-regression.
- [ ] `entity-panel.svelte` built; wraps `section`; listbox + roving focus; single-select `bind:selected`; empty message; no protocol import.
- [ ] `.entity-panel` skin (ul reset, focus ring, empty muted).
- [ ] sampler cells (spaces / DMs+status / empty / collapsed).
- [ ] CDP-verified: getter, listbox roles, roving tabindex, keyboard nav, selection, empty, collapse, child registration, corner-fix, registry delta, **0 orphans** — real output quoted.
- [ ] records closed atomically (D-074): N-078, registry v0.50, ROADMAP, phase0→COMPLETED, PLAY→J-465, JOURNAL J-465, runbook→COMPLETED.
