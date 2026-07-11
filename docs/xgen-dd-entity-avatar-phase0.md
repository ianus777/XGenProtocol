# XGen UI — dd Phase-0: `entity-avatar` (first data-dependent component)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Phase-0 subsystem audit (D-071) for the **dd track opening**. `entity-avatar` is the first data-dependent component — the one that materializes an address-book entry into a visual. Design-only; no code until Joe-locked and a runbook is authored.

---

## 1. Subsystem audit — the two domain sources

Ground truth read from the reference implementation (D-078 — symbol defs, not inferred), cross-checked against Appendix I (canonical data-structure reference, v1.7).

**`IdentityRecord`** — `xgen-core/src/identity/registry.rs`, spec 3.6.6. Display-relevant fields:

| field | type | note |
|---|---|---|
| `identity_id` | `IdentityXgid` | always present — the stable key |
| `display_name` | `Option<String>` | **may be absent** → fallback mandatory |
| `is_ai` | `bool` | kind-bool (human vs AI) |
| `ai_capabilities` | `Option<AiCapabilities>` | AI only |
| `home_node` | `NodeXgid` | |
| `revoked` | `bool` (+`revoked_at`/`revocation_reason`) | status |
| `trust_assertion` | `Option<Value>` | opaque JSON — **no tier scalar** on the record |
| `devices` | `Vec<DeviceRecord>` | |

**`SpaceState`** — `xgen-core/src/space/state.rs`. Display-relevant fields:

| field | type | note |
|---|---|---|
| `space_id` | `SpaceXgid` | always present |
| `name` | `Option<String>` | **may be absent** → fallback mandatory |
| `topic` | `Option<String>` | |
| `is_dm` | `bool` | kind-bool (DM vs Space) |
| `owner_id` | `IdentityXgid` | |
| `auth_tier` | `u32` | |
| `e2e_encryption` | `bool` | status hint |
| `members` | `HashMap<…>` | count available |
| `federation_nodes` | `Vec<NodeXgid>` | |

**Three findings that drive the design:**

1. **No image, no colour on either type.** An avatar must be **derived** — the `chip` precedent (hash → HSL muted band) for colour + initials from name. Structural, not incidental.
2. **Name is `Option` on both.** A fallback (initials-from-xgid, or a generic glyph) is mandatory when name is `None`.
3. **Each carries one kind-bool + one status:** identity → `is_ai` / `revoked`; space → `is_dm` / `e2e_encryption`. These are the dynamic inputs.

---

## 2. Theory — what makes dd ≠ di

- **di** — rendered *shape* is fixed; data only fills slots (a `textfield` is a textfield at every value).
- **dd** — the rendered *shape itself branches on the data* (identity → circle + AI-badge; space → rounded-square; revoked → greyed/slashed).

A dd's reason to exist is that it **encapsulates the domain → presentation mapping** (kind → shape, kind → badge, name → initials, seed → colour). That mapping is the "materialization." Remove it and it is just a styled box.

### 2.1 Data dwells in the address book

The avatar is **not a data source** — it is a **projection of one address-book entry** (the audited records). The avatar reads the book and materializes one entry at one **amount of information**. Several avatar formats exist; each is a partial read. The **context menu is the 100% read** (later arc) — so no avatar format needs to be complete.

The projection ladder (increasing information amount):

| variant | shows | ~% of record | tier |
|---|---|---|---|
| `presence` | shape + colour seed | ~10% | atomic |
| `list` | + initials / name-cue | ~25% | atomic |
| `labeled` | + name text | ~40% | composite |
| `card` | + one secondary line + status | ~60% | composite |
| *(context menu)* | everything | **100%** | widget (later) |

### 2.2 The primary axis is purpose, not size

"Same size, different content" is real (a 32px avatar is initials-only in a dense list, but name-beside in a nav item). So size does not *determine* content — **purpose does**. The single primary prop is a semantic **`variant`** (purpose/role); **size and content are derived presets per variant**. This is the `led.state` / `textfield.type` discipline — one semantic axis, presentation derived. A raw `size?` override can be added additively later only if a real demand recurs (D-069 four-recurrence bar).

---

## 3. The seam — `EntityDescriptor` (source-agnostic)

The dd consumes a **domain view-model**, not the raw protocol type. `core` never imports `IdentityRecord`/`SpaceState` (keeps the GPL reference lib protocol-free; matches N-057 source-agnostic). The **shell owns the protocol → descriptor map**.

```
IdentityRecord / SpaceState        (Appendix I, GPL protocol types)
        │  shell maps  (owns the protocol coupling)
        ▼
EntityDescriptor { kind, name?, id, flags, image? }   ← source-agnostic seam
        │  entity-avatar materializes  (owns kind → presentation)
        ▼
DOM: circle/square + initials + hashed colour + badge
```

- `EntityDescriptor` is a **domain** view-model (holds `kind` so the dd still branches — stays dd), **not** a presentation one (`{shape,initials,colour}` would demote it to a di).
- `image?` is **reserved-unfed** — no record carries an image today; the slot exists, honestly empty (D-065).
- **`EntityDescriptor` IS the W-11 dd-socket payload.** Designing it here fixes the slot shape `temperature-indicator` and future dd-consumers plug into — forward-consistent, no rework.

---

## 4. Locked framing (Phase-0)

1. dd materializes an **address-book entry**; shape branches on `kind`.
2. Descriptor = a **record projection** `{ kind, name?, id, flags, image? }`; `image` reserved-unfed (D-065 honest gap).
3. Avatar is a **partial read**; the **context menu = 100% escape hatch** (later arc; a menu-trigger seam is reserved on the avatar now).
4. Primary axis = **`variant`** (purpose) → size + content **derived**, not free axes.
5. **M-RP5.0 scope = dd-atomic**, self-contained variants only (`presence`, `list` = glyph / initials); `labeled` + `card` land with `container-list-item` (the first dd-composite). Badge self-drawn (`::after` pseudo), not a nested `led` — keeps the first dd atomic.

---

## 5. Decisions to Joe-lock (design walk, one at a time)

> **✅ LOCKED + BUILT (M-RP5.0, J-462).** All of A–H were Joe-locked in the design walk and realized in `entity-avatar` (the first dd-atomic); this Phase-0 is now COMPLETED. **One correction from the walk:** decision **B** below proposed a `<div>` root — the walk instead chose the semantically-honest **`<figure class="entity-avatar" role="img">`** (`aria-label={name ?? kind}`, `<figcaption>` reserved), and generalized this to **the dd-root rule (N-075):** a dd root is honest HTML for the materialized thing, NOT the di `<div>`=composite litmus. See `ui/docs/xgen-ui-notes.md` N-075 for the built record.

- **A — descriptor shape.** Confirm `EntityDescriptor { kind: 'identity'|'space', name?, id, flags, image? }`; `flags` = `{ isAi?, revoked?, isDm?, e2e? }`. *LOCKED as proposed.*
- **B — dd root convention.** Propose dd-atomic root = `<div class="entity-avatar">` (a materialized object → composite-like shell, unlike di-atomic's native-tag root). *Rec: yes.* **→ LOCKED as `<figure role="img">` instead (N-075 dd-root rule; see the banner above).**
- **C — kind → shape.** identity = circle; space = rounded-square; DM space = circle (people-shaped). *Rec: circle for identity + DM, rounded-square for non-DM space.*
- **D — badges (this milestone).** `isAi` badge + `revoked` overlay (greyed + slash) in-scope; `e2e` lock deferred to a later amendment. *Rec: yes.*
- **E — colour seed.** `hash(name ?? id) → HSL`, reuse the `chip` muted S/L band via a shared helper. *Rec: yes.*
- **F — variant set (v1).** `presence` (xs, glyph) · `list` (sm, glyph+initials). `labeled`/`card` reserved for the composite step. *Rec: yes.*
- **G — registration.** dd-atomic self-registers one getter `{ kind, variant, name, initials, seed, flags }`. *Rec: yes.*
- **H — menu-trigger seam.** Reserve `onActivate?` (or equivalent) now; the `entity-context-menu` widget consumes it later. *Rec: reserve, don't build.*

---

## 6. Roadmap — dd track (M-RP5)

| milestone | component | tier | note |
|---|---|---|---|
| **M-RP5.0** | `entity-avatar` | dd-**atomic** | first dd; `variant` presence/list; kind → shape; descriptor seam; menu-trigger reserved |
| M-RP5.1 | `container-list-item` | dd-**composite** | object-backed row; composes entity-avatar; unlocks `labeled`/`card` variants |
| M-RP5.2 | `spaces-panel` | dd-composite | composes `section` + rows |
| M-RP5.3 | `entity-context-menu` | widget | the 100% read; consumes `EntityDescriptor` |
| M-RP5.4 | `temperature-indicator` | widget | consumes `meter` via the W-11 dd-socket |

> **⚠️ WITHDRAWN (2026-07-11, J-502).** The `meter` + W-11-dd-socket mechanism above is **wrong** — it was written before Ch6 §6.12 was consulted. Temperature is an **existing protocol property** (spec §3.7.13 · Ch6 §6.12 · D-061): the home Node's plugin computes it, the client treats the value as **opaque**, and `xgen-client/src/temperature.rs` already ships the `temperature_update` event + the `data-temp-state` DOM contract. **Real shape: a `$common` store + a skin contract, rendered as content — no `meter`, no dd-socket, no surface.** See `docs/xgen-widget-surfaces-phase0.md` §6.2.

Kind-4 `use:render` stays deferred (D-065). `temperature-indicator` unblocks once a dd-consumer + the descriptor socket exist.

---

*UI-architecture Phase-0 audit. No protocol/data implication — `core` stays protocol-free behind the `EntityDescriptor` seam. Framing locks 1–5 set; decisions A–H await the design walk before a runbook is authored.*
