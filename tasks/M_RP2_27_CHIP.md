# M-RP2.27 — `chip` (removable token)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## Goal
`chip` — 22nd `core`, a standalone di token (atomic-ish `<span class="chip">`). Prerequisite for `tag-select` (M-RP2.28): built standalone + reusable (dd facets, tier/`is_ai` badges, entity tokens); the registry already reserves the name. Uppercase label + optional removable `×`. Colour computed from the label (hash→HSL, fixed muted S/L band) so it's shell-independent + deterministic. Matrix **80→83** (3 sampler cells).

## Locks
1. **Structure** — `span.chip` (envelope). Props: `label` (raw value stored; uppercase is display-only), `removable?` (default true), `onRemove?`, `id`. Getter `{label, removable}`. Atomic-ish (no self-registering child components).
2. **Remove** — `×` on the **right**; `×`-only remove (whole chip stays inert-selectable for later); `onRemove?` fired on `×` click; `×` is a masked stroke glyph (N-052 lineage, `--chip-x`).
3. **Colour (computed)** — `hash(label)` → hue; fixed muted **S/L band** (never bright/white); fill = the hue at that band, text = same hue darkened, thin same-hue border. No palette table, no per-tag config. Rides inline CSS vars the `.chip` skin reads (the `led` colour-var precedent, N-034 lineage — first di whose colour is neither accent-derived nor caller-supplied but self-computed).
4. **Visual** — 8px uppercase, `letter-spacing: 0.03em`, compact padding, small radius. Long labels truncate with ellipsis (fixed max-width). Non-removable = no `×`, same fill.
5. **Matrix** — cells default / non-removable / long-label → **80→83**.

## Steps
- **A** — `chip.svelte`: envelope root `<span class="chip">`, props, `hash→HSL` compute (inline `--chip-bg`/`--chip-fg`/`--chip-bd` vars), conditional `×` button (`removable`), `onRemove?` wire. Getter `{label, removable}`.
- **B (skin)** — `.chip` block: inline-flex, 8px uppercase + `letter-spacing`, padding/radius, reads the 3 computed vars, `--chip-x` masked `×` glyph, ellipsis truncation (max-width).
- **C (sampler)** — DI·atomic panel: 3 cells (default / non-removable / long-label). `vite build`.
- **D (CDP — self-driven, 9422, both accents)** — `ids()===83`; `#default {label, removable:true}`; `#static {removable:false}` (no `×`); computed fill/fg differ per label (deterministic, same under both accents — colour is self-computed, not accent-derived); `×` click fires `onRemove` (spy); long-label ellipsis; screenshots; 0 orphans on 9422/5175.

## DoD
- [x] A: `chip.svelte`, hash→HSL vars, conditional `×`, getter `{label, removable}`
- [x] B: `.chip` skin, computed-colour vars read, `×` mask, uppercase + ellipsis
- [x] C: 3 sampler cells, `vite build` clean (142 modules)
- [x] D: CDP both accents, real output (Rule 2), matrix 80→83, colour-self-computed proof, `onRemove` spy, 0 orphans
- [x] Records (D-074): N-064 + registry v0.36 + ROADMAP v4.19 + J-450 + CLAUDE PLAY + this runbook → COMPLETED
