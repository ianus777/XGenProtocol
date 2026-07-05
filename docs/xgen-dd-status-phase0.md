# XGen UI — dd Phase-0: `status` (self-status component, variant-driven)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Phase-0 subsystem audit (D-071) for **M-RP5.1a** — `status`, a dd-atomic that materializes a self-set status (Track A `state.status`, J-461) into a visual. Variant = display density. Composed as a corner-badge slot on `entity-avatar` so every avatar-bearing surface inherits it. Design-only; no code until Joe-locked + runbook.

---

## 1. Why a component (not an avatar field)

Self-status is personal expression, not connection state — worth carrying onto **every** entity display. Making it its own dd-atomic + composing it as an avatar slot means item/panel/nav/mention all inherit the badge free, and `line`/`full` reuse the same materialization standalone. Same discipline as `entity-avatar`/`entity-item`: purpose → variant; new need → new variant.

## 2. Subsystem input — Track A `state.status`

Ground truth (J-461, `xgen-core/src/status/`): `StatusRecord { emoji?, text?, updated_at, expires_at? }` — emoji = 1 grapheme cap; text ≤128B; lazy expiry; global/identity-scoped. **Presence (online/away) is explicitly NOT this** (Track A deferred it). So `status` renders self-set expression, never here/not-here.

Source-agnostic seam: `status` consumes a view-model `{ emoji?, text?, updatedAt?, expiresAt? }` (shell maps from `StatusRecord`); `core` imports no protocol type.

## 3. Variants (display density)

| variant | shows | use |
|---|---|---|
| `badge` | emoji only, corner overlay | rides every avatar |
| `line` | emoji + text, inline | card / row secondary |
| `full` | emoji + text + relative time | detail / context menu |

- text-absent → emoji only (all variants).
- no inline room / emoji-only → text as tooltip (`title`).
- expired (lazy) → renders empty/absent (no stale status).

## 4. Locked framing (Phase-0)

1. One component, `variant` = density; new need → new variant.
2. Backed by Track A `state.status` via source-agnostic view-model; presence excluded.
3. `badge` variant is the avatar corner-slot payload; `line`/`full` are standalone reads.
4. dd-atomic root per N-075 (honest HTML; class×arity from folder + panel + getter).
5. Emoji 1-grapheme (Track A cap) fits the corner badge exactly.

## 5. Decisions to Joe-lock (walk)

- **A — name.** `status`. *Rec: yes.*
- **B — variant set.** `badge`/`line`/`full` + rules above. *Rec: yes.*
- **C — root.** `badge` = `<span class="status" role="img">` (token-like, emoji glyph); `line`/`full` = `<span class="status">` with text. *Rec: yes.*
- **D — avatar seam.** `entity-avatar` gains a `status?` slot; renders `status variant="badge"` as a positioned corner overlay (bottom-right, like the presence-dot position but self-status). *Rec: yes — amendment to M-RP5.0.*
- **E — tooltip fallback.** emoji-only / no-room → `title` = text (+ maybe expiry). *Rec: yes.*
- **F — expiry display.** lazy: expired → absent; `full` may show relative "updated 5m ago" from `updatedAt`. *Rec: absent-on-expire; relative time full-only.*
- **G — getter.** `{ variant, emoji, hasText, expired }` (never leak full text if we treat it as sensitive — it's public, so text ok; keep `hasText` for parity). *Rec: `{ variant, emoji, hasText, expired }`.*

## 6. Roadmap — dd track (M-RP5, updated)

| milestone | component | tier | note |
|---|---|---|---|
| M-RP5.0 | `entity-avatar` | dd-atomic | ✅ CLOSED (J-462) |
| M-RP5.1 | `entity-item` | dd-composite | ✅ CLOSED (J-463) |
| **M-RP5.1a** | **`status`** | dd-atomic | `badge`/`line`/`full`; backed by `state.status` |
| M-RP5.1b | `entity-avatar` amend | — | +`status?` corner-slot (composes `status badge`) |
| M-RP5.2 | `spaces-panel` | dd-composite | inherits status via avatar |
| M-RP5.3 | `entity-context-menu` | widget | uses `status full` |
| M-RP5.4 | `temperature-indicator` | widget | consumes `meter` via W-11 socket |

Kind-4 `use:render` stays deferred (D-065).

---

*Phase-0 audit. No protocol implication — `core` stays protocol-free behind the status view-model. Framing 1–5 set; decisions A–G await the walk before a runbook.*
