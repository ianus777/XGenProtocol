# XGen UI — dd Phase-0: `room` kind (entity-avatar + descriptor amendment)
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Phase-0 subsystem audit (D-071) for **M-RP5.0c** — add `room` as a third entity kind: the second location representation (a room/channel inside a space, peer to the space itself). Amends `EntityDescriptor` + `entity-avatar` shape branch; ripples free through `entity-item`/`entity-panel`. Decisions A–E **LOCKED** (2026-07-05). No code until runbook + Joe "go".

---

## 1. Why a kind, not a flag

A room is a first-class location entity — a peer to `space`, not a variant of it. `kind: 'room'` (Option A) keeps the shape branch honest and lets item/panel treat it uniformly. Rejected: `flags.isRoom` (Option B) — hides a location peer inside space, muddies the kind taxonomy.

## 2. Kind taxonomy (after amendment)

| kind | shape | note |
|---|---|---|
| `identity` | circle | person / AI |
| `space` | rounded-square | a community/server |
| **`room`** | **hexagon** | a room/channel inside a space |
| (DM) | circle | `space` + `flags.isDm`, unchanged |

## 3. Decisions — LOCKED (2026-07-05)

- **A — model.** `kind: 'room'` (Option A, own kind). ✅
- **B — shape.** hexagon via CSS `clip-path` polygon. ✅
- **C — status corner.** bottom-right badge nudged onto the hex hull (inset tweak); bottom-right contract kept. ✅
- **D — initials.** same render, centered on the hexagon. ✅
- **E — sampler.** room cells in DD·atomic (avatar) + DD·composite (item + panel row). ✅

## 4. Subsystem touch

- `EntityDescriptor.kind` union += `'room'` (source-agnostic; `core` protocol-free).
- `entity-avatar.svelte` shape branch += `room → hexagon`; ring/seed/status/initials inherit.
- Additive → identity/space/DM 0-regression; `entity-item`/`entity-panel` need **no** code change (pass-through).

## 5. Roadmap

| ms | scope | tier |
|---|---|---|
| M-RP4.9 | sampler infra | sampler |
| **M-RP5.0c** | `room` kind (avatar+descriptor) | dd-atomic amend |
| M-RP5.3 | `entity-context-menu` | widget |
| M-RP5.4 | `temperature-indicator` | widget |

---

*Phase-0 audit. Source-agnostic behind `EntityDescriptor`; `core` protocol-free. Decisions A–E LOCKED; runbook next.*
