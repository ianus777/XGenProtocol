# XGen UI — dd Phase-0: `entity-item` (variant-driven entity composite)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Phase-0 subsystem audit (D-071) for **M-RP5.1** — `entity-item`, the first dd-**composite**. A single variant-driven composite that materializes one address-book entry as a full display unit (avatar + text + meta), replacing the earlier per-shape plan (`container-list-item`, `entity-card`, …). Design-only; no code until Joe-locked + runbook authored.

---

## 1. Why one composite, not many

The earlier roadmap named `container-list-item` as a standalone. That path multiplies bespoke "avatar + something" composites (row, card, nav entry, mention). Instead: **one composite, purpose selected by `variant`** — the same "purpose → variant, presentation derived" discipline that governs `entity-avatar`, applied one tier up. A genuinely new entity-display need becomes a **new variant**, not a new component; standalone only when it can't fit the variant model (D-069 recurrence bar).

## 2. Relation to `entity-avatar` (two tiers, one knob)

- `entity-avatar` (dd-atomic, M-RP5.0) = one entity's glyph; its own `variant` (`presence`/`list`/`labeled`/`card`).
- `entity-item` (dd-composite) = a layout container that **composes** `entity-avatar` + name + secondary line + trailing meta + row behaviour.
- **Single-knob rule (lock):** the consumer sets **one** `entity-item` `variant`; the composite **derives** the inner avatar variant internally. The two variant axes never fight.

## 3. Subsystem inputs

Same source as M-RP5.0 — the `EntityDescriptor` seam (source-agnostic, `core` imports no protocol types). Additional display inputs the composite surfaces beyond the avatar:
- name (from descriptor `name?`, fallback already handled by avatar).
- secondary line — topic / last-message / handle (caller-supplied, source-agnostic; NOT read from protocol here).
- trailing meta — unread count, timestamp, **self-status** (emoji+text from Track A `state.status`, J-461).
- row behaviour — activate, selected/hover, keyboard nav (list context).

Status is now unblocked (Track A shipped `state.status`), so a status-bearing variant is buildable this milestone if in scope.

## 4. Locked framing (Phase-0)

1. One composite, `variant` = purpose; new need → new variant.
2. Single-knob: `entity-item.variant` derives `entity-avatar.variant`.
3. Entity-generic (identity ∪ space ∪ DM) — driven by descriptor `kind`, not a per-kind component.
4. Secondary line + meta are **caller-supplied slots** (source-agnostic); composite owns layout, not protocol reads.
5. dd-composite root per N-075 (honest HTML; class×arity from folder + panel + getter). Composes the real `entity-avatar` child (self-registers, matrix multiplies).

## 5. Decisions to Joe-lock (design walk, one at a time)

- **A — name.** `entity-item` (mirrors `entity-avatar`, purpose-neutral). *Rec: yes.*
- **B — variant set (v1).** `row` (dense list line) · `card` (richer tile) · `nav` (sidebar entry) · `inline` (compact mention/token). + the derive-inner-avatar-variant map + the "new-need → new-variant" rule. *Rec: yes; confirm the four.*
- **C — slot surface per variant.** which of {name, secondary, meta/status} each variant shows (derived, not free). *Rec: row=name+meta · card=name+secondary+status · nav=name · inline=name.*
- **D — row behaviour.** `onActivate?`; `selected?`/hover as skin state; keyboard nav owned by the *list/panel* (M-RP5.2), not the item. *Rec: item exposes `onActivate?`+`selected?`, panel owns roving focus.*
- **E — status wiring.** consume `state.status` as a caller-supplied `status?: {emoji?,text?}` slot (source-agnostic, shell maps from Track A). *Rec: yes, slot only.*
- **F — getter.** `{ variant, kind, name, hasSecondary, hasStatus, selected }`. *Rec: yes.*
- **G — root element.** honest dd-composite root (`<div class="entity-item">` acceptable here — dd-composite, panel-disambiguated; or `<article>`/`<li>` if a list-semantic is wanted). *Rec: decide at walk; lean `<div>`.*

## 6. Roadmap — dd track (M-RP5, updated)

| milestone | component | tier | note |
|---|---|---|---|
| M-RP5.0 | `entity-avatar` | dd-atomic | ✅ CLOSED (J-462) |
| **M-RP5.1** | **`entity-item`** | dd-**composite** | variant-driven; single-knob; renamed from `container-list-item` |
| M-RP5.2 | `spaces-panel` | dd-composite | composes `section` + `entity-item ×N`; owns roving focus |
| M-RP5.3 | `entity-context-menu` | widget | the 100% read |
| M-RP5.4 | `temperature-indicator` | widget | consumes `meter` via W-11 socket |

Kind-4 `use:render` stays deferred (D-065).

---

*Phase-0 audit. No protocol implication — `core` stays protocol-free behind `EntityDescriptor` + caller-supplied slots. Framing locks 1–5 set; decisions A–G await the walk before a runbook.*
