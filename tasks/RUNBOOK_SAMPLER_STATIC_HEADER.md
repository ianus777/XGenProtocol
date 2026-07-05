# Runbook — M-RP4.9 sampler static-header + scroll + rename
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Sampler-infra build. No component/registry touch. Design locked (`docs/xgen-sampler-static-header-phase0.md`). Skin path `ui/assets/skin.css`. No push — Joe pushes.

---

## Steps

1. **Static header** — `ui/sampler/app_sampler.svelte`: restructure shell to `flex-column` — fixed block (title + `client|node` toggle + tab bar) / body wrapper `flex:1; overflow-y:auto`. Tabs no longer scroll.
2. **Confined scroll** — `.s-*` skin: scroll lives on the body wrapper only; header block `flex:0 0 auto`. Body = the red-rectangle region under the tabs.
3. **Tab rename** — labels → `DI Atomics · DI Composites · DD Atomics · DD Composites · Widgets` (string-only; keep tab indices/keys stable so N-053 panel routing + CDP unaffected).

## Verify (CDP 9422)

- `vite build` clean; served module confirmed (N-058).
- Assert: tab bar stays put while body scrolls (scrollTop on body wrapper, header offsetTop constant); five tabs new labels; panel routing per tab unchanged (DD·composite still renders entity-item/panel cells); registry count **unchanged** (no component delta); 0 orphans. Screenshot scrolled state.

## D-074 close (one commit)

- `ui/docs/xgen-ui-notes.md` → N-079 (sampler static-header/scroll), v-bump.
- `docs/ROADMAP.md` → M-RP4.9 ✅, v-bump.
- phase0 → COMPLETED.
- `CLAUDE.md` PLAY → J-466.
- `JOURNAL.md` → J-466 (last, real CDP).
- this runbook → COMPLETED.
- No registry/components-doc change (no component delta); no DECISIONS.

## DoD

- [x] static header (title+toggle+tabs fixed); body-only scroll. *(`.sampler-scroll` flex:1/overflow-y:auto/min-height:0; #sampler-root overflow:hidden)*
- [x] five tabs renamed; indices/keys stable; panel routing unchanged. *(labels DI Atomics/DI Composites/DD Atomics/DD Composites/Widgets; ids untouched)*
- [x] CDP-verified: header-fixed-while-body-scrolls, labels, routing, registry unchanged, 0 orphans. *(headerFixed:true @ scrollTop=400, docScrollable:false, registry 173, DD Composites renders entity-panel/item; screenshot static-header-verify.png)*
- [x] records: N-079, ROADMAP, phase0→COMPLETED, PLAY→J-466, JOURNAL, runbook→COMPLETED.
