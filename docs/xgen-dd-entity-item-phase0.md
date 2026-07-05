# XGen UI — dd Phase-0: `entity-item` (variant-driven entity composite)
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Phase-0 subsystem audit (D-071) for **M-RP5.1** — `entity-item`, the first dd-**composite**. A single variant-driven composite that materializes one address-book entry as a full display unit (avatar + text + meta), replacing the earlier per-shape plan (`container-list-item`, `entity-card`, …). Decisions A–G **LOCKED** (2026-07-05 design walk). No code until runbook authored + Joe "go".

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

Status is now unblocked (Track A shipped `state.status`), so a status-bearing variant is buildable this milestone.

## 4. Locked framing (Phase-0)

1. One composite, `variant` = purpose; new need → new variant.
2. Single-knob: `entity-item.variant` derives `entity-avatar.variant`.
3. Entity-generic (identity ∪ space ∪ DM) — driven by descriptor `kind`, not a per-kind component.
4. Secondary line + meta are **caller-supplied slots** (source-agnostic); composite owns layout, not protocol reads.
5. dd-composite root per N-075 (honest HTML; class×arity from folder + panel + getter). Composes the real `entity-avatar` child (self-registers, matrix multiplies).
6. **Global width rule (N-076, new standing contract).** No `width` set → **100%** (fills container); `width` set → that value; **`min-width` = the component's intrinsic composition floor** (its slot layout's natural minimum). Generalizes the `meter`/`section` `width?` precedent to a default contract for all width-bearing components; retro-referenced by `meter`/`section`.

## 5. Decisions — LOCKED (2026-07-05)

- **A — name.** `entity-item`. ✅
- **B — variant set (v1) + rules.** `row` · `card` · `nav` · `inline`. Derive-map: `row`→avatar `list` · `card`→`card` · `nav`→`labeled` · `inline`→`presence`. Rule: new entity-display need → **new variant** (standalone only if it can't fit, D-069 bar). ✅
- **C — slot surface per variant** (derived, not free): `row` = name + meta · `card` = name + secondary + status · `nav` = name · `inline` = name. ✅
- **D — row behaviour.** item exposes `onActivate?` + `selected?` (skin state); the list/panel (M-RP5.2) owns roving keyboard focus, not the item. ✅
- **E — status wiring.** caller-supplied `status?: { emoji?, text? }` slot, source-agnostic (shell maps from Track A `state.status`). ✅
- **F — getter.** `{ variant, kind, name, hasSecondary, hasStatus, selected }`. ✅
- **G — root element.** `<div class="entity-item">` (dd-composite, panel-disambiguated per N-075). ✅
- **Width.** `width?` per the N-076 global rule (unset = 100%; set = value; `min-width` = slot-composition floor). ✅

## 6. Roadmap — dd track (M-RP5)

| milestone | component | tier | note |
|---|---|---|---|
| M-RP5.0 | `entity-avatar` | dd-atomic | ✅ CLOSED (J-462) |
| **M-RP5.1** | **`entity-item`** | dd-**composite** | variant-driven; single-knob; renamed from `container-list-item` |
| M-RP5.2 | `spaces-panel` | dd-composite | composes `section` + `entity-item ×N`; owns roving focus |
| M-RP5.3 | `entity-context-menu` | widget | the 100% read |
| M-RP5.4 | `temperature-indicator` | widget | consumes `meter` via W-11 socket |

Kind-4 `use:render` stays deferred (D-065).

---

*Phase-0 audit. No protocol implication — `core` stays protocol-free behind `EntityDescriptor` + caller-supplied slots. Framing locks 1–6 set; decisions A–G LOCKED; runbook next.*
