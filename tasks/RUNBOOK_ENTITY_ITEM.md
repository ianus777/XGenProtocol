# Runbook — M-RP5.1 `entity-item` (variant-driven dd-composite)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Build runbook for `entity-item` — the first dd-**composite**. Design locked in Phase-0 (`docs/xgen-dd-entity-item-phase0.md` v1.1, decisions A–G). Session-open order still applies (Rule 0). No push — Joe pushes.

---

## Locked design (A–G + width)

- **A** name `entity-item`.
- **B** variants `row`/`card`/`nav`/`inline`; derive-map row→avatar `list`, card→`card`, nav→`labeled`, inline→`presence`; new need → new variant (D-069 bar).
- **C** slots: row = name+meta · card = name+secondary+status · nav = name · inline = name.
- **D** `onActivate?` + `selected?` (skin state); panel owns roving focus.
- **E** `status?: { emoji?, text? }` caller-supplied slot (source-agnostic, shell maps Track A `state.status`).
- **F** getter `{ variant, kind, name, hasSecondary, hasStatus, selected }`.
- **G** root `<div class="entity-item">`.
- **Width (N-076 global rule):** `width?` — unset = 100%, set = value, `min-width` = slot-composition floor.

## Step 1 — type + component

- `ui/core/lib/components/data-dependent/entity-item.svelte`:
  - root `<div class="entity-item">`, `data-variant`, `data-kind`, `data-selected`.
  - props: `descriptor: EntityDescriptor` (reuse M-RP5.0 type), `variant: 'row'|'card'|'nav'|'inline'` (default `'row'`), `secondary?: string`, `status?: { emoji?: string; text?: string }`, `meta?: string` (unread/timestamp — caller string), `selected?: boolean`, `onActivate?`, `width?: string`, `id`.
  - composes real `entity-avatar` child — inner variant **derived** from the derive-map (single-knob); child self-registers (`__avatar`).
  - slot surface per variant (C) — conditional render, not free props.
  - `onActivate?` on root (click/Enter); `selected?` → `data-selected` skin state. No roving focus here (panel).
  - getter F via `use:envelope`.
- DoD: no protocol imports; single-knob derive verified; secondary/status/meta are caller slots only.

## Step 2 — skin

- `.entity-item` in `ui/skin.css`: flex layout (avatar · text-column {name / secondary} · trailing meta+status); per-variant sizing/density (`[data-variant]`); `[data-selected]` highlight; hover state; **width rule** (`width:100%` default via unset, inline `width` override, `min-width` floor).
- Accent posture: `selected`/hover may use `--accent`; entity colour stays seed-driven (avatar owns it). PROVISIONAL.

## Step 3 — sampler (DD·composite panel)

- Populate the **DD·composite** panel (empty placeholder, N-053). Cells:
  - each variant × representative kind: `row` (identity), `card` (space w/ secondary+status), `nav` (DM), `inline` (identity).
  - edge: `selected` cell, absent-secondary, status-bearing cell, fixed-`width` cell.
- Stable ids (`entity-item#row-identity`, etc.); avatar children register `__avatar`.

## Step 4 — CDP verify (sampler 9422)

- `vite build` clean; kill zombies + confirm served module contains `entity-item#…` BEFORE probing (N-058).
- Assert: getter F per cell; inner avatar variant matches derive-map per composite variant; slot surface per C (row no secondary, card has secondary+status, etc.); `data-selected` highlight; width unset→100% / set→value / `min-width` floor present; child `__avatar` registered (matrix multiplies); root `DIV.entity-item`; `.entity-item*` rules in cascade; registry delta; **0 orphans**.
- Quote real CDP output (Rule 2). Screenshot `temp/entity-item-verify.png`.

## Step 5 — D-074 atomic close (one commit)

- `ui/docs/xgen-ui-notes.md` → **N-076** (global width rule) + entity-item note (first dd-composite; single-knob derive), version bump.
- `ui/docs/xgen-ui-components.md` → registry **v0.48** (entity-item row; DD·composite panel populated; width-rule note; retro-ref meter/section), version bump.
- `docs/ROADMAP.md` → M-RP5.1 ✅ DONE, RP node + tree tail, v-bump.
- `docs/xgen-dd-entity-item-phase0.md` → Status → COMPLETED (per Joe).
- `CLAUDE.md` PLAY → entry head → J-463.
- `JOURNAL.md` → J-463 (written last, real CDP output).
- this runbook → **Status: COMPLETED**, version bump.
- No `DECISIONS.md` touch (arc-local; width rule is a component contract → N-note, not DECISIONS).

---

## Definition of Done

- [x] `entity-item.svelte` built; root `<div>`; composes `entity-avatar` with derived inner variant; getter F; `onActivate?`/`selected?`; no protocol imports.
- [x] slot surface per variant (C) correct. *(CDP: card-space hasSecondary+hasStatus true; row/nav/inline false; card-plain absent-secondary false)*
- [x] width rule (N-076) implemented (unset=100%, set=value, min-width floor). *(CDP: row no inline style → 180px floor; fixed inline 280px; inline min-width:0)*
- [x] `.entity-item` skin; per-variant density; selected/hover states. *(14 `.entity-item*` rules in cascade; card border+`--s2`; `[data-selected]` gold bar)*
- [x] sampler DD·composite panel populated (variants × kinds + edge cells). *(7 cells: 4 variant×kind + selected + card-plain + fixed)*
- [x] CDP-verified: getter, derive-map, slots, selected, width, child registration, registry delta, **0 orphans** — real output quoted. *(registry 124→138, count===unique===138; screenshot temp/entity-item-verify.png)*
- [x] records closed atomically (D-074): ui-notes N-076, registry v0.48, ROADMAP, phase0→COMPLETED, CLAUDE PLAY→J-463, JOURNAL J-463, runbook→COMPLETED.
