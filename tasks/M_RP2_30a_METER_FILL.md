# M-RP2.30a — `meter` `fill?` (custom bar colour, additive)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-04  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Small **additive amendment** to the shipped `meter` (M-RP2.30, J-457) — an optional custom fill colour, the `led`/`chip` data-coloured-via-inline-var mechanism. No breaking change.

## Locked design (Joe, option A)

- **`fill?` prop** — hex or `var(--token)`. Set → an inline `--meter-fill` var; the skin reads `background: var(--meter-fill, <semantic>)` on all three value pseudos, so a fixed colour **overrides the optimum/sub/over semantics entirely** (the true custom progress bar).
- **Unset** → falls back to the existing `--ok` / `--warn` / `--err` semantic fills (no behaviour change for current callers).
- Bonus: closes the no-optimum-reads-green gap — a neutral bar is now `fill="var(--t3)"`.
- Getter gains `fill` → `{value, min, max, optimum, fill}`.

## Build steps

- **A — `meter.svelte`**: add `fill?: string` prop; when set, add `--meter-fill: {fill}` to the root inline `style` (compose with the existing `width?` style — one `style` string); getter adds `fill`.
- **B — skin (`.meter`)**: change each value-pseudo `background: var(--ok|--warn|--err)` → `background: var(--meter-fill, var(--ok|--warn|--err))`.
- **C — sampler**: add `meter#custom` cell (`fill="var(--accent2)"` — proves accent-gold bar + override) and/or `fill="var(--t3)"` neutral.
- **D — CDP verify** (9422, real output): getter carries `fill`; the inline `--meter-fill` var present on `#custom`; `.meter::-webkit-meter-*` rules now show `var(--meter-fill, …)` in cascade; registry +1; 0 orphans.
- **E — records (D-074)**: N-072 (ui-notes, fill amendment) · registry v0.44 (getter note) · ROADMAP v4.27 (M-RP2.30a) · CLAUDE PLAY (→ J-458) · JOURNAL J-458 · this runbook → COMPLETED.

## Definition of Done

- [x] `fill?` prop added, composed with `width?` in one inline style, getter carries `fill`, `vite build` clean.
- [x] `.meter` value-pseudos read `var(--meter-fill, <semantic>)`.
- [x] Sampler `meter#custom` cell added.
- [x] CDP-verified (getter, inline var, cascade, registry delta, 0 orphans).
- [x] Records closed atomically, runbook → COMPLETED.

## After this
- The **dd track opens (M-RP5.0):** `section-header` (ungrounded warm-up) → `entity-avatar` (first domain-bound, D-071 audit on IdentityRecord/Appendix I).
