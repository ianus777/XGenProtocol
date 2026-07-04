# M-RP2.31a — `section` `width?` (settable width, additive)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-04  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Small **additive amendment** to `section` (M-RP2.31, J-459) — a settable width, the `meter` `width?` mechanism (M-RP2.30). No breaking change.

## Locked design (Joe)
- **`width?`** (string, e.g. `"320px"`/`"24rem"`) → inline `width` on the root `<section>`. Unset → **100%** (fills container).
- **`min-width: 160px`** floor in `.section` (a titled box needs more than meter's 80px).
- Getter gains `width` → `{title, badge, collapsible, collapsed, width}` (meter precedent).

## Build steps
- **A** `section.svelte`: add `width?: string`; inline `style={width ? \`width: ${width}\` : undefined}` on root; getter adds `width`.
- **B** skin: `.section { min-width: 160px; }` (+ the skin already has no explicit width → the block default fills; confirm 100% holds).
- **C** sampler: add `section#fixed` cell (`width="320px"`).
- **D** CDP verify (9422): getter carries `width`; `#fixed` computed `width:320px`; a default cell fills; `min-width` present; registry +1; 0 orphans.
- **E** records (D-074): N-074 ui-notes · registry v0.46 (getter note) · ROADMAP v4.29 (M-RP2.31a) · CLAUDE PLAY (→ J-460) · JOURNAL J-460 · this runbook → COMPLETED.

## DoD
- [x] `width?` prop + inline style + getter, build clean.
- [x] `.section` min-width:160px.
- [x] Sampler `section#fixed` cell.
- [x] CDP-verified (getter, width:320px, min-width, registry delta, 0 orphans).
- [x] Records closed, runbook COMPLETED.

## After this
- The **dd track opens (M-RP5.0):** `entity-avatar` (domain-bound, D-071 audit: IdentityRecord/SpaceState) → `container-list-item` → `spaces-panel`.
