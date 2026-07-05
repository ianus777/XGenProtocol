# XGen UI — Phase-0: sampler static-header + scroll reorg + tab rename
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Phase-0 for **M-RP4.9** — sampler-infra only (no component/registry impact). Three changes: static tab bar, scroll region confined to the panel body, tab-header rename. Design-only; no code until Joe "go".

---

## 1. Scope

Sampler shell (`ui/sampler/app_sampler.svelte` + `.s-*` skin in `ui/assets/skin.css`). **Zero component touch, zero registry delta.** Pure test-bed ergonomics.

## 2. Changes

- **A — static tab bar.** Tab headers (`DI·atomic … WIDGET`) + the shell/mode toggle (`client|node`) become **fixed** (outside the scroller). Currently the whole body under the title scrolls, taking the tabs with it.
- **B — confined scroll region.** Only the panel body (the red-rectangle region: from under the tab bar to the window bottom) scrolls vertically. Layout: `flex-column` shell — fixed header block (title + tabs) / `flex:1; overflow-y:auto` body.
- **C — tab rename.** `DI · atomic → DI Atomics` · `DI · composite → DI Composites` · `DD · atomic → DD Atomics` · `DD · composite → DD Composites` · `WIDGET → Widgets`. String-only.

## 3. Framing / risk

- CDP harness reads via `data-debug-id` on components, not tab DOM → rename is **safe** (no probe by tab label). Confirm no test greps the old tab strings.
- Scroll reorg is CSS structural; the four-panel keyed rendering (N-053) is unchanged — only the scroll container boundary moves.
- No `.md` beyond sampler notes; N-note optional (sampler ergonomics, not a component contract).

## 4. Decisions to lock (trivial)

- **A** static header (title+tabs+toggle fixed). *Rec: yes.*
- **B** body-only scroll (`overflow-y:auto` on body wrapper). *Rec: yes.*
- **C** tab labels as §2C. *Rec: yes.*

## 5. Roadmap

| ms | scope | tier |
|---|---|---|
| **M-RP4.9** | sampler static-header + scroll + rename | sampler infra |
| M-RP5.0c | room kind (avatar+descriptor) | dd-atomic amend |
| M-RP5.3 | `entity-context-menu` | widget |
| M-RP5.4 | `temperature-indicator` | widget |

Rec: land M-RP4.9 before M-RP5.0c (cleaner test-bed for the new room cells).

---

*Sampler-infra Phase-0. No protocol/component/registry implication.*
