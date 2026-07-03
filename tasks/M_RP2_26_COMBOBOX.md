# M-RP2.26 — `combobox` (rich, owned-popup)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## Goal
`combobox` — 21st `core`, 5th di composite. **Owned-popup** (own `<ul role="listbox">`, not native datalist) so rows can be styled: compact, left-aligned, no balloon, rich rows. Passive di (owns only `open` — a UI flag, like password-field's `revealed`, NOT a widget). Reusable owned-popup pattern → later powers color-picker. Matrix **74→80**.

## Locks
1. **Structure** — `div.combobox` (envelope, owns `open`) + `textfield` child (`cid('input')`, collision-safe vs password-field `__field`) + own `<ul role="listbox">`. `bind:value`=text; `options`=`{value,label,status?,disabled?,icon?}[]` (back-compat `string[]`); `icon?` declared, unwired until icon primitive. Getter `{value, open, count}`.
2. **Open/close** — open on focus/click; close on blur (next-tick, so select lands), on select, on `esc`.
3. **Filter** — case-insensitive substring on `label`; empty = all.
4. **Visual** — own `<ul>`: compact rows, left-aligned, square-ish (thin border/shadow, no balloon). Chevron −5px. Icon swap: collapsed **chevron** (`--tri`), expanded **closed triangle** (`--tri-open`), stroke-only masked sw2, on `.combobox[data-open]::after`.
5. **Disabled/selection** — disabled row unselectable+dimmed; disabled composite inert; select sets `value` + closes.
6. **Matrix** — cells default / preset / disabled → +2/cell → **74→80**.

## Steps
- **A** — `combobox.svelte`: envelope root, `open` state, textfield child (`cid('input')`), `<ul role="listbox">` from normalized rows, filter derived, select handler. Getter `{value,open,count}`.
- **B (skin)** — `.combobox` block: relative anchor, `[data-open]` popup show, compact left-aligned `<ul>`/`<li>`, `--tri`/`--tri-open` swap, chevron −5px, disabled dims.
- **C (sampler)** — DI·composite panel: 3 cells (default / preset / disabled). `vite build`.
- **D (CDP — self-driven, 9422, both accents)** — `ids()===80`; composite + `__input` child; open-on-focus sets `data-open` + shows `<ul>`; filter narrows rows; select sets value + closes; disabled inert; ▼ swaps chevron↔triangle; accent gold↔blue; screenshots (closed + open); 0 orphans on 9422/5175.

## DoD
- [ ] A: `combobox.svelte`, `cid('input')`, getter aggregate
- [ ] B: `.combobox` skin, owned `<ul>`, ▼ swap, no balloon
- [ ] C: 3 sampler cells, `vite build` clean
- [ ] D: CDP both accents, real output (Rule 2), matrix 74→80, 0 orphans
- [ ] Records (D-074): N-063 + registry v0.35 + ROADMAP + J-449 + CLAUDE PLAY + this runbook → COMPLETED
