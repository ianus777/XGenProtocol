# XGen UI — dd Phase-0: `entity-panel` (roving-focus entity list)
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Phase-0 subsystem audit (D-071) for **M-RP5.2** — `entity-panel` (a.k.a. the `spaces-panel` consumer preset), the dd-composite that composes `section` (group chrome) + `entity-item ×N` (rows) and owns roving keyboard focus + selection. Last dd-composite before the widget tier. Decisions A–H **LOCKED** (2026-07-05). No code until runbook + Joe "go".

---

## 1. What it composes

- `section` (di, M-RP2.31) — collapsible header (title + badge) + body slot. The group chrome.
- `entity-item ×N` (dd-composite, M-RP5.1) — the rows, fed `EntityDescriptor[]`.
- panel-owned layer — roving focus, selection, empty state. **This** is why it's its own composite, not a bare `section`.

Every row inherits `entity-avatar` (+ status corner-slot, M-RP5.1b) for free.

## 2. Locked framing

1. **Wrap, not beside.** `section` is the root; panel fills its body slot with rows. `section` stays the honest group container; panel is the data + focus layer inside it.
2. **Panel owns roving focus** (decision D deferred it out of `entity-item`): one tabstop, arrows move active row, Home/End, Enter/Space activates. Panel `role="listbox"`, rows `role="option"` (single-select).
3. **Data in, source-agnostic.** `items: EntityDescriptor[]` (+ per-item secondary/status/meta), not protocol types.
4. dd-composite; composes real `section` + `entity-item` children (self-register; matrix multiplies).

## 3. Decisions — LOCKED (2026-07-05)

- **A — name.** `entity-panel` (entity-generic; lists any entities). `spaces-panel` = a consumer preset/label, not the component name. ✅
- **B — root/compose.** wrap `section`; `<section>`-rooted, panel body = `<ul role="listbox">`. ✅
- **C — focus model.** roving tabindex + listbox semantics + arrows/Home/End/Enter; `selected` → active row. ✅
- **D — selection.** single active row `bind:selected` (id); click + keyboard. Multi-select deferred. ✅
- **E — empty/loading.** empty → in-body message (di `paragraph`/`label`); loading skeleton deferred (D-065). ✅
- **F — collapse.** inherit `section` `collapsible?`/`collapsed?` pass-through (collapsed hides rows, slot stays mounted). ✅
- **G — getter.** `{ count, selected, collapsed, hasEmpty }`. ✅
- **H — avatar corner-fix.** fold in the isAi/status de-collision — **status = bottom-right, `isAi` spark → top-right** — so panel rows inherit clean avatars (option A from the M-RP5.1b close). ✅

## 4. Roadmap — dd track (M-RP5)

| ms | component | tier | note |
|---|---|---|---|
| M-RP5.0/.1/.1a/.1b | avatar / item / status / slot | — | ✅ CLOSED |
| **M-RP5.2** | `entity-panel` | dd-composite | wrap `section` + rows; roving focus; +corner-fix |
| M-RP5.3 | `entity-context-menu` | widget | the 100% read |
| M-RP5.4 | `temperature-indicator` | widget | `meter` via W-11 socket |

Closes the dd-composite tier; widget tier (5.3/5.4) next. Kind-4 `use:render` deferred (D-065).

---

*Phase-0 audit. Source-agnostic behind `EntityDescriptor`. Framing 1–4 set; decisions A–H LOCKED; runbook next.*
