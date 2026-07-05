# XGen UI — dd Phase-0: `room` kind (entity-avatar + descriptor amendment)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Phase-0 subsystem audit (D-071) for **M-RP5.0c** — add `room` as a third entity kind: the second location representation (a room/channel inside a space, peer to the space itself). Amends `EntityDescriptor` + `entity-avatar` shape branch; ripples free through `entity-item`/`entity-panel`. Design-only; no code until Joe "go".

---

## 1. Why a kind, not a flag

A room is a first-class location entity — a peer to `space`, not a variant of it ("second type of location representation"). Modeling it as `kind: 'room'` (Option A) keeps the shape branch honest and lets downstream (item/panel) treat it uniformly. Rejected: `flags.isRoom` sub-flag (Option B) — hides a location peer inside space, muddies the kind taxonomy.

## 2. Kind taxonomy (after amendment)

| kind | shape | note |
|---|---|---|
| `identity` | circle | person / AI |
| `space` | rounded-square | a community/server |
| **`room`** | **hexagon** | a room/channel inside a space |
| (DM) | circle | `space` + `flags.isDm` (people-shaped, unchanged) |

Hexagon = visually distinct from circle (identity/DM) and rounded-square (space); reads as "structured location."

## 3. Subsystem touch

- `EntityDescriptor.kind` union += `'room'` (source-agnostic seam; `core` still protocol-free).
- `entity-avatar.svelte` shape branch += `room → hexagon` (CSS `clip-path` polygon; ring/seed/status corner all inherit unchanged).
- Additive: identity/space/DM shapes untouched → M-RP5.0/5.1/5.2 cells 0-regression.
- `entity-item`/`entity-panel` need **no code change** — they pass `descriptor` through; room shape appears free.

## 4. Decisions to lock

- **A — model.** `kind: 'room'` (Option A). *Rec: yes.*
- **B — shape.** hexagon via `clip-path`. *Rec: yes; confirm vs alt (tag/pennant).*
- **C — status corner + hexagon.** bottom-right badge on a hex clip — verify the corner sits on the hull, not clipped. *Rec: nudge inset if needed; keep bottom-right contract.*
- **D — initials on hexagon.** same initials render, centered. *Rec: yes.*
- **E — sampler.** add room cells to DD·atomic (avatar) + DD·composite (item/panel with a room row). *Rec: yes.*

## 5. Roadmap

| ms | scope | tier |
|---|---|---|
| M-RP4.9 | sampler infra | sampler |
| **M-RP5.0c** | `room` kind (avatar+descriptor) | dd-atomic amend |
| M-RP5.3 | `entity-context-menu` | widget |
| M-RP5.4 | `temperature-indicator` | widget |

---

*Phase-0 audit. Source-agnostic behind `EntityDescriptor`; `core` protocol-free. Decisions A–E await the walk before a runbook.*
