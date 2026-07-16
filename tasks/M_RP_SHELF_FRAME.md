# M-RP-SHELF-FRAME — fixed-height shelves
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

## Goal

Both shelves (top favourites · bottom system) hold a **fixed height** whether empty or full, so an empty favourites strip no longer collapses and shifts the centre grid. Joe: a calmer, non-reflowing grid frame.

## What changed (skin-only, 1 file)

`ui/assets/skin.css` — the `.shelf[data-empty]` collapse rule (`min-height/padding/border → 0`) was neutralised. Both shelves already share the base `.shelf` rule (`min-height: var(--ctl-h)` + `padding: 0 var(--sp-2)` + a position hairline: top `border-bottom`, bottom `border-top`), so removing the collapse makes an empty shelf hold that same frame — no hardcoded pixel, DPR-safe by construction. `data-empty` stays **emitted** (a skin hook, no JS reader — grepped) but no longer zeroes the box. The shelf docblock was corrected in place (the "`data-empty` collapses … the 6.1j pre-pinning look" note → the fixed-frame note).

No component, registry, catalogue, or schema change. Zero Rust. PROVISIONAL skin (Joe HMR-tunes).

## Verification (CDP, live client 9222 — Chat drove, Rule 5)

Applied under the running `tauri dev` session (HMR), measured in place:

| shelf | before | after |
|---|---|---|
| bottom (button) | 28.8px, `data-empty:false` | 28.8px — unchanged |
| top (favourites) | 0px (collapsed, `data-empty:true`) | 28px, `data-empty:true` (attr retained), `min-height:28px`, hairline 0.8px |

The collapse is gone; the top strip went 0 → 28px, so the grid no longer reflows between an empty and a populated favourites shelf.

**Measured residual (accepted, not a defect):** top 28 vs bottom 28.8 = 0.8px (one device pixel) — `box-sizing:border-box` + `min-height: var(--ctl-h)` means a populated shelf's faces push its border-box 0.8px past the min while an empty shelf sits exactly at min. Sub-pixel; Joe accepted against the optical bar (N-128). The exact-equality alternative — `.shelf { height: var(--ctl-h) }` → both 28px — is filed-not-taken.

## DoD

- [x] `.shelf[data-empty]` no longer collapses; both shelves render `--ctl-h` + hairline empty or full.
- [x] Top shelf measured 0 → 28px on 9222; bottom unchanged at 28.8px.
- [x] `data-empty` still emitted (skin hook), no JS reader relies on the collapse.
- [x] Skin docblock corrected in place.
- [x] Records: JOURNAL J-530 · N-130 · ROADMAP (v5.01) · CLAUDE.md PLAY.

## Notes

The node inherits this free at **M-RP7.7** (one shared skin → both apps frame identically). No new D.
