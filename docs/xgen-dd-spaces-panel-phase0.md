# XGen UI — dd Phase-0: `spaces-panel` (roving-focus entity list)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Phase-0 subsystem audit (D-071) for **M-RP5.2** — `spaces-panel`, the dd-composite that composes `section` (group chrome) + `entity-item ×N` (rows) and owns roving keyboard focus + selection. Last dd-composite before the widget tier. Design-only; no code until Joe-locked + runbook.

---

## 1. What it composes

- `section` (di, M-RP2.31) — collapsible header (title + badge) + body slot. The group chrome.
- `entity-item ×N` (dd-composite, M-RP5.1) — the rows, fed `EntityDescriptor[]`.
- panel-owned layer — roving focus, selection, empty/loading. **This** is why it's its own composite, not a bare `section`.

Every row inherits `entity-avatar` (+ its status corner-slot, M-RP5.1b) for free.

## 2. Locked framing

1. **Wrap, not beside.** `section` is the root; the panel fills its body slot with rows. `section` stays the honest group container; panel is the data + focus layer inside it.
2. **Panel owns roving focus** (decision D deferred it out of `entity-item`): one tabstop for the list, arrows move active row, Home/End, Enter/Space activates. Rows are `role="option"`, panel `role="listbox"` (single-select) — or `role="list"` + buttons if multi-activate. *Rec: listbox.*
3. **Data in, source-agnostic.** Panel takes `items: EntityDescriptor[]` (+ per-item secondary/status/meta), not protocol types.
4. dd-composite; composes real `section` + `entity-item` children (self-register; matrix multiplies).

## 3. Decisions to Joe-lock (walk)

- **A — name.** `spaces-panel` (or `entity-panel` — it lists any entities, not only spaces). *Rec: `entity-panel`, entity-generic; `spaces-panel` = a preset/consumer.*
- **B — root/compose.** wrap `section`; `<section>`-rooted, panel body = `<ul role="listbox">`. *Rec: yes.*
- **C — focus model.** roving tabindex, listbox semantics, arrows/Home/End/Enter, `selected` → active row. *Rec: yes.*
- **D — selection.** single active row `bind:selected` (id); click + keyboard. *Rec: single-select v1; multi deferred.*
- **E — empty/loading.** empty → in-body message (di `paragraph`/`label`); loading → skeleton rows deferred. *Rec: empty message v1, loading deferred (D-065).*
- **F — collapse.** inherit `section` `collapsible?`/`collapsed?` pass-through; collapsed hides rows (slot stays mounted). *Rec: pass-through.*
- **G — getter.** `{ count, selected, collapsed, hasEmpty }`. *Rec: yes.*
- **H — avatar corner-fix.** fold in the isAi/status corner de-collision (status = bottom-right, `isAi` → top-right) so panel rows inherit clean avatars. *Rec: bake in (option A from the M-RP5.1b close).*

## 4. Roadmap — dd track (M-RP5)

| ms | component | tier | note |
|---|---|---|---|
| M-RP5.0/.1/.1a/.1b | avatar / item / status / slot | — | ✅ CLOSED |
| **M-RP5.2** | `entity-panel` (`spaces-panel`) | dd-composite | wrap `section` + rows; roving focus; +corner-fix |
| M-RP5.3 | `entity-context-menu` | widget | the 100% read |
| M-RP5.4 | `temperature-indicator` | widget | `meter` via W-11 socket |

Closes the dd-composite tier; widget tier (5.3/5.4) next. Kind-4 `use:render` deferred (D-065).

---

*Phase-0 audit. No protocol implication — source-agnostic behind `EntityDescriptor`. Framing 1–4 set; decisions A–H await the walk before a runbook.*
