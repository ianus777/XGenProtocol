# Runbook — M-RP5.0c `room` kind (avatar + descriptor amendment)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Build runbook for the `room` kind. Design locked (`docs/xgen-dd-room-kind-phase0.md` v1.1, A–E). Additive amendment to `entity-avatar` (M-RP5.0) + `EntityDescriptor`. Skin path `ui/assets/skin.css`. Rec: land after M-RP4.9. No push — Joe pushes.

---

## Locked design (A–E)

- **A** `kind: 'room'` (own kind). **B** hexagon via `clip-path`. **C** status badge nudged onto hex hull, bottom-right kept. **D** initials centered. **E** sampler room cells (avatar + item + panel).

## Steps

1. **Descriptor** — `EntityDescriptor.kind` union += `'room'` (source-agnostic; no protocol import).
2. **Avatar shape** — `entity-avatar.svelte`: shape branch += `room → 'hexagon'` (`data-shape="hexagon"`); ring/seed/initials/status inherit. No structure change.
3. **Skin** — `ui/assets/skin.css`: `.entity-avatar[data-shape="hexagon"] { clip-path: polygon(...) }`; nudge `.entity-avatar[data-shape="hexagon"] .status` inset so the corner sits on the hull. Identity/space/DM rules untouched.
4. **Sampler** — add room cells: DD·atomic avatar (room, variants presence/list) + DD·composite an `entity-item` room row + an `entity-panel` cell with a room in the list.

## Verify (CDP 9422)

- `vite build` clean; served module confirmed (N-058).
- Assert: room cell `data-shape="hexagon"`; `clip-path` in computed style; initials centered; status badge on-hull (not clipped) for a room-with-status cell; identity/space/DM cells **unchanged** (0-regression); item/panel room rows render hex avatars with no item/panel code change; registry delta = +room cells; 0 orphans. Screenshot.

## D-074 close (one commit)

- `ui/docs/xgen-ui-notes.md` → N-080 (room kind), v-bump.
- `ui/docs/xgen-ui-components.md` → registry v0.51 (avatar kind taxonomy += room), v-bump.
- `docs/ROADMAP.md` → M-RP5.0c ✅, v-bump.
- phase0 → COMPLETED.
- `CLAUDE.md` PLAY → J-467 (or next).
- `JOURNAL.md` → entry (last, real CDP).
- this runbook → COMPLETED.
- No DECISIONS.

## DoD

- [ ] `EntityDescriptor` kind += room; no protocol import.
- [ ] avatar `room → hexagon`; ring/seed/initials/status inherit; identity/space/DM 0-regression.
- [ ] `clip-path` hexagon skin + status hull-nudge.
- [ ] sampler room cells (avatar + item + panel).
- [ ] CDP-verified: hexagon shape, clip-path, centered initials, status on-hull, item/panel free ripple, registry delta, 0 orphans.
- [ ] records: N-080, registry v0.51, ROADMAP, phase0→COMPLETED, PLAY→entry, JOURNAL, runbook→COMPLETED.
