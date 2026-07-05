# Runbook — M-RP5.0 `entity-avatar` (first dd-atomic)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Build runbook for `entity-avatar` — the **first data-dependent component** (dd-atomic). Design locked in Phase-0 (`docs/xgen-dd-entity-avatar-phase0.md`) + the design walk (J-461 session). Ground-truth read order still applies at session open (Rule 0). No push — Joe pushes.

---

## Locked design (A–H + B)

- **A — descriptor.** `EntityDescriptor { kind: 'identity'|'space', name?: string, id: string, flags: { isAi?: boolean, revoked?: boolean, isDm?: boolean, e2e?: boolean }, image?: string }`. `image` reserved-unfed (D-065). Source-agnostic seam = the W-11 dd-socket payload; `core` never imports `IdentityRecord`/`SpaceState`.
- **B — dd-root rule (N-075).** dd does **not** inherit the di `<div>=composite` litmus. dd root = **honest HTML** for the materialized thing; class×arity is read from folder (`data-dependent/`) + sampler panel + getter. `entity-avatar` root = **`<figure class="entity-avatar" role="img">`**, `aria-label={name ?? kind}`; `<figcaption>` **reserved** (unused v1; the seam for `labeled`/`card`).
- **C — kind → shape.** identity = circle · non-DM space = rounded-square · DM space = circle (people-shaped).
- **D — badges.** `isAi` badge + `revoked` overlay (greyed + slash), self-drawn via `::after`/`::before` pseudos (NOT a nested `led` — keeps the first dd atomic). `e2e` lock **deferred**.
- **E — colour seed.** `seedColour(name ?? id) → HSL`, shared helper factored from `chip`'s hash.
- **F — variants v1.** `presence` (xs, glyph only) · `list` (sm, glyph + initials). `labeled`/`card` reserved for `container-list-item` (M-RP5.1).
- **G — getter.** `{ kind, variant, name, initials, seed, flags }`.
- **H — menu seam.** reserve `onActivate?` (don't build; `entity-context-menu` consumes it, M-RP5.3).

---

## Step 1 — shared `seedColour` helper (refactor from `chip`)

- Extract `chip`'s `hash(label) → hue` + muted S/L band into a shared pure helper (DOM-free, `logic.ts` posture). Proposed home: `ui/common/lib/components/base/seed-colour.ts` (or the nearest existing shared util — confirm at build; keep `chip` importing the same fn, zero behaviour change).
- Signature: `seedColour(key: string): { bg: string; fg: string; bd: string }` (chip's existing triple) **or** a single `hsl(...)` if the avatar only needs the fill — decide at build against chip's actual shape; refactor must leave `chip`'s output **byte-identical** (re-verify chip's sampler cells unchanged, 0 regression).
- DoD: `chip` still green in sampler (per-label fills unchanged); helper is the single source.

## Step 2 — type + component

- `EntityDescriptor` type — colocate with the component or a `data-dependent/types.ts`; exported for downstream dd consumers.
- `ui/core/lib/components/data-dependent/entity-avatar.svelte`:
  - root `<figure class="entity-avatar" role="img">`, `aria-label={name ?? kind}`, `data-variant`, `data-kind`, `data-shape`.
  - props: `descriptor: EntityDescriptor`, `variant: 'presence'|'list'` (default `'presence'`), `onActivate?`, `id`.
  - derived: `shape` = kind+isDm → `'circle'|'square'`; `initials` = from `name` (1–2 graphemes, `unicode-segmentation`-equivalent JS: `Intl.Segmenter` or a grapheme-safe slice) else xgid-derived fallback; `seed` = `seedColour(name ?? id)`.
  - `variant` drives size + content preset (presence = glyph/colour only; list = + initials). Size/content are **derived per variant**, not free props.
  - badges: `isAi` → `::after` badge glyph; `revoked` → greyed + slash overlay pseudo. `e2e` NOT drawn (deferred).
  - `onActivate?` reserved seam (wire the handler, no menu).
  - getter G via `use:envelope`.
- DoD: no import of protocol types; `<figcaption>` present-but-unused seam noted in-file.

## Step 3 — skin

- `.entity-avatar` in `ui/skin.css`: shape (`border-radius` circle vs `--rad`-ish square), per-variant sizes, initials type, `--seed-*` inline-var read (led/chip mechanism), `isAi` badge + `revoked` slash/grey. PROVISIONAL.
- Accent posture: seed-coloured (shell-independent, like `chip`), NOT accent-derived — confirm no `--accent` dependency.

## Step 4 — sampler (DD·atomic panel)

- Populate the **DD·atomic** panel (currently empty placeholder, N-053). Rows/cells:
  - identity × {presence, list}; space (non-DM) × {presence, list}; DM space × {presence, list}.
  - edge cells: absent-name (fallback initials/glyph), `revoked` identity, `isAi` identity.
- Stable ids per cell (`entity-avatar#id-presence`, etc.).

## Step 5 — CDP verify (sampler 9422)

- `vite build` clean; kill zombies + confirm served module contains `entity-avatar#…` BEFORE probing registry (N-058 lesson).
- Assert: getter `{kind,variant,name,initials,seed,flags}` per cell; `data-shape` circle for identity + DM, square for non-DM space; `isAi` badge present, `revoked` greyed+slash; absent-name → fallback initials; seed shell-independent (client↔node identical, no accent swap); root `FIGURE`/`role=img`; `.entity-avatar*` rules in cascade; registry delta = +N cells; **0 orphans**.
- Quote real CDP output (Rule 2). Screenshot `temp/entity-avatar-verify.png`.

## Step 6 — D-074 atomic close (all records same commit)

- `ui/docs/xgen-ui-notes.md` → **N-075** (dd-root rule + entity-avatar the first dd-atomic), version bump.
- `ui/docs/xgen-ui-components.md` → registry **v0.47** (entity-avatar row, first `data-dependent/` occupant; DD·atomic panel populated; new dd-root schema note), version bump.
- `docs/ROADMAP.md` → M-RP5.0 ✅ DONE, RP node + tree tail, **v-bump**.
- `docs/xgen-dd-entity-avatar-phase0.md` → decisions A–H marked LOCKED (Status stays ACTIVE or → COMPLETED per Joe).
- `CLAUDE.md` PLAY → entry head → J-462.
- `JOURNAL.md` → J-462 (written last, real CDP output quoted).
- this runbook → **Status: COMPLETED**, version bump.
- No `DECISIONS.md` touch (B is registry/N-note, arc-local; D-069 bar not met).

---

## Definition of Done

- [x] `seedColour` helper shared (`ui/common/lib/components/base/seed-colour.ts`); `chip` re-verified 0 regression (per-label fills byte-identical: rust→h307, svelte→h216, long→h60).
- [x] `entity-avatar.svelte` built; root `<figure role="img">`; getter G `{kind,variant,name,initials,seed,flags}`; `onActivate?` wired + `<figcaption>` reserved (in-file comment seam); no protocol imports (`EntityDescriptor` seam in `data-dependent/types.ts`).
- [x] `.entity-avatar` skin; seed-coloured (`--seed-*`), no accent dependency (verified `seedMatch:true` client↔node).
- [x] sampler DD·atomic panel populated (identity/space/DM × presence/list + absent/revoked/isAi = 9 cells).
- [x] CDP-verified: getter fields, shape-per-kind, isAi/revoked badges, absent→"AZ" fallback, seed shell-independence, registry **115→124**, **0 orphans** — real output quoted in J-462.
- [x] records closed atomically (D-074): ui-notes N-075 (v0.59), registry v0.47, ROADMAP v4.31, phase0 A–H LOCKED, CLAUDE PLAY→J-462, JOURNAL J-462, runbook + handoff→COMPLETED.
