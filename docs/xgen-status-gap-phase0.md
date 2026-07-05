# XGen — Self-Set Status: Protocol Gap + Two-Track Phase-0
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Companion to `xgen-dd-entity-avatar-phase0.md`. Surfaces a fundamental protocol gap found while opening the dd track, and splits the work into two parallel tracks.

---

## 1. The gap

Self-set status (emoji + description line, per the Discord reference shot) is **not modelled anywhere in the protocol**. Audit of `IdentityRecord` / `SpaceState` (D-078, symbol defs):

- **Exists, system-derived:** `revoked` (lifecycle), `is_ai` (kind), `member_temperature` (3.7.13, moderation heat, visibility-gated), `active_mutes.cooldown_until` (transient).
- **Missing, greenfield:** self-set status `{ emoji, text }`, presence (online/typing/dots).

A federated identity protocol without a **status primitive** is missing something every peer implementation will expect. It belongs in the spec.

## 2. Data locations (dissection)

- **User records** — node holds authoritative `IdentityRecord`; client holds its projection (`xgen-client_state.json` + address book = client-side seen-records).
- **Avatar data** — none stored. Avatar is **derived** from a record (initials + hashed colour) + reserved-unfed `image?`.
- **Status data** — nowhere yet (the gap).

## 3. How status must travel

- **Self-set status** = durable identity trait → rides identity federation (`IdentityRecord` `update_version`, 3.6.8). **Protocol concern — must be specced.**
- **Presence** (online/typing) = ephemeral → a lightweight separate channel, **not** the durable record. Likely out-of-band, deferred.

## 4. Two-track plan

**Track A — status protocol arc (Chat, priority).** Spec a home for self-set `{ emoji, text }` on the federated identity path: field vs `state.*` event, `update_version` propagation, size caps, visibility. Gates all status-bearing UI.

**Track B — `entity-avatar` / `avatar-entry` core (UI, parallel).** Multi-format display block, **zero status dependency**. Minimum one purpose variant: **`list`** (glyph + initials + name-cue). Derives shape from `kind`, colour from `hash(name ?? id)`. Buildable now.

**Converge:** status-bearing avatar variants (status-emoji overlay + status text line) land only after Track A ships.

## 5. Roadmap

| track | milestone | scope |
|---|---|---|
| A | **PROTO-STATUS.0** | Phase-0 audit of identity federation path; locate status field/event home |
| A | PROTO-STATUS.1 | Spec self-set `{ emoji, text }` + propagation + visibility |
| B | **M-RP5.0** | `entity-avatar` core — `list` variant, `kind`→shape, derived colour, child-free |
| B | M-RP5.1 | second variant (`presence`) + `EntityDescriptor` seam finalized |
| B→A | M-RP5.2 | status-bearing variants (gated on PROTO-STATUS.1) |
| B | M-RP5.3+ | `container-list-item` → `spaces-panel` → context-menu → `temperature-indicator` |

## 6. Open (next locks)

- Track A: field-on-record vs separate `state.*` event.
- Track B: confirm `list` as the first variant; confirm `<div class="entity-avatar">` as the dd block root (dd axis may differ from di's `<div>`=composite rule).
- Presence dots / corner-emoji semantics: **parked** (Discord-example only, not spec).

---

*Recommendation: open Track A now as priority design; build Track B core in parallel. They converge at the status-bearing variant.*
