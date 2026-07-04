# M-RP2.28 — `tag-select` (multi-tag input, chip consumer)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## Goal
`tag-select` — 23rd `core`, 6th di composite. The **chip consumer**: a multi-select tag input that renders selected values as `chip` instances via `{#each}` (N-064, no per-instance registration) and picks candidates from an owned popup (the combobox owned-popup pattern). Passive di, `bind:value → string[]`. Matrix **+2/cell** (composite + `__filter` child; chips don't multiply) → **83 → 89** (3 cells).

## Locks
1. **Structure** — `div.tag-select` (envelope). Passive di. Composes a `textfield` child (query buffer, suffix `__filter`) + own `<ul role="listbox">` + `chip` `{#each}`. Getter `{ values, count }` (select-multiple precedent).
2. **Model** — `value: string[]` `$bindable`, default `[]` (option *values*). Query buffer is local `$state` (`bind:value={query}` on the child), **not** the model; cleared on every pick.
3. **Options** — combobox schema `(string | {value,label,status?,disabled?})[]`, normalized (reuse combobox normalize). Chip label resolved from `options`; freeform (allowCreate) value===label. Source-agnostic (N-057) — sampler literal here; real client TOML `[tags]` deferred to the consumer / M-RP4.3.
4. **Popup (two sections)** — reuse owned-popup. Top **"Selected (N)"** = all picked rows (reachable even when the row collapses to `+N`); main list = `notSelected && matchesQuery` (hide-selected). Pick **stays open**, clears query, refocuses field. Own skin key `.tag-select-list`.
5. **Control row** — fixed `min-height: --ctl-h`; chips `flex-wrap` + inline growing filter input; overflow chips collapse to a `+N` counter pill.
6. **Add / remove / edge behaviour** —
   - Pick option → push `r.value` (stay open, clear query).
   - `allowCreate?` default **false**; true → Enter on non-empty query with **no exact match** creates `{value:q,label:q}`.
   - Dedup: **case-insensitive, silent** (dup pick/create clears query, no-op).
   - `max?` unset default; at cap → picks **no-op + field dims** (input read-only-ish, list hidden).
   - Query empty + **Backspace** → pop last from `value[]`.
   - Arrow-into-chips nav: **deferred** (stays passive; Backspace-last is the essential path).
7. **a11y** — field `role="combobox"` + `aria-expanded`; `<ul role="listbox" aria-multiselectable="true">`; option `aria-selected`; chip `×` keeps `aria-label="Remove {label}"`.
8. **Matrix** — cells `#default` (few options, 1–2 preselected) / `#max` (cap-2, prove no-op+dim) / `#create` (allowCreate) → **83 → 89**.

## Steps
- **A** — `tag-select.svelte`: envelope root `div.tag-select`; props (`value`/`options`/`placeholder`/`disabled`/`allowCreate`/`max`/`id`/`name`); options normalize (combobox shape); `query`/`open` state; derived `selectedRows`/`shown`; getter `{ values, count }`.
- **B** — chips via `{#each}` (N-064, no envelope) + `+N` overflow collapse; `__filter` child wire (`bind:value={query}`); pick / remove / Backspace-last / dedup / max (no-op+dim) logic.
- **C** — popup: two-section owned `<ul>` (selected-on-top + filtered-main), stay-open pick, hide-selected, allowCreate Enter path.
- **D (skin)** — `.tag-select` control row (`--ctl-h`, flex-wrap, growing input, `+N` pill) + own `.tag-select-list` (selected-section divider, compact rows); `data-open` toggles popup; `data-full` dims at cap.
- **E (CDP — self-driven, 9422, both accents)** — `ids()===89`; child `textfield#*__filter` (no collision); `#default` `{values:[...],count:N}`; pick pushes + stays open + clears query + hides-selected; top section lists all selected; `#max` cap-2 → 3rd pick no-op + `data-full`; `#create` Enter creates value===label; Backspace-last pops; dedup silent; screenshots both accents; 0 orphans on 9422/5175.

## DoD
- [x] A: `tag-select.svelte`, options normalize, `query`/`open`, getter `{ values, count }`
- [x] B: chips `{#each}` (no reg) + `+N` overflow, `__filter` wire, pick/remove/backspace/dedup/max
- [x] C: two-section popup, stay-open pick, hide-selected, allowCreate Enter
- [x] D: `.tag-select` + `.tag-select-list` skin, `+N` pill, selected divider, `data-full` dim
- [x] E: CDP both accents, real output (Rule 2), matrix 83→89, `__filter` no-collision, cap no-op+dim, create value===label, 0 orphans
- [x] Records (D-074): N-065 + registry v0.37 + ROADMAP v4.20 + J-451 + CLAUDE PLAY + this runbook → COMPLETED; `[tags]` seed design note logged; `HANDOFF_UI_TIER_DISCUSSION.md` ACTIVE→DEPRECATED
