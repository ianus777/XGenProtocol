# M-RP2.26 — `combobox` (5th di composite)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## Goal
`combobox` — the **21st `core`** and **5th di composite**. Passive (Path A, native `<datalist>` — browser owns popup + filtering). `<input list>` + `<datalist>` + a decorative ▼. Matrix **74→80**.

## Locks (this session)
1. **Icon** — `--tri` = stroke-only rounded down-triangle SVG (`fill='none' stroke stroke-width='2' stroke-linejoin='round'`), masked like the eye (N-052 lineage), 18px, scoped on `.combobox`.
2. **textfield `list?` prop** — Step-A additive (default-absent → 44-cell registry unchanged; `redactValue` precedent), forwarded to `<input list>`. **Own commit.**
3. **▼ is decorative** — `.combobox::after`, NOT a button (no reveal action; native datalist not reliably click-openable). The one graphical divergence from `password-field`.
4. **Shape A** — `<div class="combobox">` = real `textfield` child (`__field`, self-registers, `redactValue` off) + raw `<datalist id>` + `::after` ▼; `list` wires field→datalist by id.
5. **`options` prop** — N-034 `select` normalized shape (`string[] | {value,label,disabled?}[]`) → `<option value>` in the datalist.
6. **Getter + matrix** — composite `{value, count}` (count = options length); child field `{type:"text", value}`. Cells default / disabled / preset-value → **+2/cell → 74→80**.

## Steps
- **A (own commit)** — `textfield.svelte`: add `list?: string` prop, forward to `<input list>`. Registry behaviour-unchanged. `vite build` gate.
- **B** — `combobox.svelte`: `<div>` root via `envelope`, aggregate getter `{value, count}`; compose `Textfield` (`bind:value`, `list={cid('list')}`, `id={cid('field')}`) + `<datalist id={cid('list')}>` from normalized `options`. `disabled` passthrough.
- **C (skin)** — `.combobox` block in `skin.css`: flex, field `flex:1`, `position:relative`; `--tri` var + `::after` masked glyph (18px, `--t3` neutral, right-anchored, `pointer-events:none`); `[aria-disabled]` dims.
- **D (sampler)** — DI·composite panel: 3 cells (default / disabled / preset-value). `vite build`.
- **E (CDP verify — Chat self-drives, sampler 9422, both accents)** — `ids()===80`; composite + `__field` baseline; datalist `<option>` count; type-to-filter round-trips `bind:value`; disabled inert; `::after` 18px + mask set + `--t3`; accent gold↔blue; `.combobox` rules in cascade; screenshots (▼ outlined, left/right placement); 0 orphans.

## DoD
- [ ] A: `textfield` `list?` forwarded, build clean, registry unchanged — own commit
- [ ] B: `combobox.svelte` built, getter aggregate, children self-register
- [ ] C: `.combobox` skin + `--tri` stroke-only ▼, no-reflow
- [ ] D: 3 sampler cells, `vite build` clean
- [ ] E: CDP both accents, real output (Rule 2), matrix 74→80, 0 orphans
- [ ] Records (D-074 atomic): N-063 + registry v0.35 + ROADMAP + J-449 + CLAUDE PLAY + this runbook → COMPLETED
