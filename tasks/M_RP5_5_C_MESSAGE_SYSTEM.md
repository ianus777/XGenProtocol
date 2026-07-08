# M-RP5.5 C — `system` kind + full `isOwn` verify (message dd-composite)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-08  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Goal

Third + final build step of the `message` dd-composite: the second kind — `system` (authorless centered notice) — plus a full both-sides `isOwn` verify. Closes the **message family v1**. Ground truth = `docs/xgen-dd-message-family-phase0.md` v1.0 (§2 types, §3 map, §7 step C).

## Scope (C only)

IN: `system` kind (no avatar / no header / no name / no details / no edited / no tombstone — just a centered `paragraph` notice); getter normalized on `system`; `.message[data-kind="system"]` skin; 2 system sampler cells (plain + long/wrapping); re-assert the `text` `isOwn` mirror both sides. Getter unchanged on `text`.
OUT (later): `bodyExtras` (reserved-unfed, D-065); `message-stream` (M-RP5.6); R5 system-widget wrap (M-RP6.x).

## Render rules

- **`system`** — a distinct branch, NOT the `text` grid. Author is absent → no `entity-avatar`. Render ONE centered `paragraph` (self-registers `__body`); no `.msg-header`, no `label` (`__name`), no `details`, no `edited` marker, no tombstone path. Root `<article data-kind="system">`, **no `data-own`**.
- **`text`** — unchanged from A/B. `isOwn` → `data-own` mirror (avatar side + alignment flip), already built; C only re-asserts both sides in the sampler.
- **Kind branch** — a single top-level split on `kind`: `system` → centered-notice sub-tree; `text` → the existing A/B sub-tree (avatar column + header guard + body/tombstone). Keep the two paths visibly separate so `system` reads none of the text-only fields.

## Getter (Option A — normalized on `system`)

On `kind:'system'`, the getter forces the text-only fields off (they are structurally meaningless — system is authorless/centered):

```
{ kind:'system', author:null, hasBody, detailsCount:0,
  isOwn:false, grouped:false, edited:false, deleted:false }
```

On `kind:'text'`, the getter is **verbatim**, unchanged from A/B. Add a one-line comment at the normalization marking it deliberate (getter tracks RENDER truth, not descriptor truth — the `deleted → detailsCount:0` precedent, J-479). No fixture needs to exercise a stray-field system descriptor; the normalization is a guard, not a feature.

## Files

- `ui/core/lib/components/data-dependent/message.svelte` — top-level `kind` branch; `system` centered-notice sub-tree (centered `paragraph`, `__body` only); getter `system` normalization + comment.
- `ui/assets/skin.css` — `.message[data-kind="system"]`: collapse the avatar-column grid → full-width, `text-align:center`, muted "special-adjust" line (no new token if `--t3`/`--fs-*` suffice); ensure centered wrap stays symmetric.
- `ui/sampler/…` — cells: `message#system-notice` (plain, e.g. `"alice joined the room"`), `message#system-long` (wrapping, e.g. a rename notice). Confirm the existing `text` own/other cells still present for the mirror re-assert (no new text cell needed if A's own+other already stand).

## Build steps

1. `message.svelte` — introduce the top-level `kind` split; move the existing A/B markup under the `text` arm untouched; add the `system` arm (centered `paragraph`, `__body`, no avatar/header/details). Getter: normalize on `system`, verbatim on `text`, add the deliberate-normalization comment.
2. Skin — `.message[data-kind="system"]` centered full-width rule; verify no avatar column reserved; wrap check.
3. Sampler — the 2 system cells above.
4. CDP verify (9422, both accents) — see checklist.

## CDP checklist (9422)

- `message#system-notice`: root `data-kind="system"`, **no** `data-own`; `__avatar` NOT registered, `__name` NOT registered, `__body` registered; getter = `{kind:'system',author:null,detailsCount:0,isOwn:false,grouped:false,edited:false,deleted:false}`; computed-style `text-align:center`, full-width (no avatar column).
- `message#system-long`: as above + the notice wraps to ≥2 lines, stays centered, no overflow, symmetric.
- `text` mirror re-assert: other-side grid `28px 288px`, own-side `288px 28px` (`data-own`), from the A cells.
- `count===unique`, 0 orphans **both directions**.
- Both accents (`--accent2` `#c28840` ↔ `#3a7ab0`) — system is accent-neutral geometry; note if the centered line picks up any accent (it should not).

## Registry note

Each `system` cell registers `message + __body` = **2** (no `__avatar`, no `__name`). Two system cells → **+4**. Start 215 → expected **219** at close; record the actual post-build count (Rule 5 — do not treat the estimate as verified). No orphans.

## DoD

- 2 system cells CDP-verified (9422), both accents.
- `system`: no avatar / no header / no name / no details / no edited / no tombstone; centered `paragraph` only; `__body` registered, `__avatar`/`__name` absent.
- Getter normalized on `system` (all text-only fields false, `author:null`, `detailsCount:0`); verbatim on `text`; comment present.
- Long system notice wraps centered, symmetric, no overflow.
- `text` `isOwn` mirror re-asserted both sides.
- Registry `count===unique`, 0 orphans both directions; actual delta recorded.
- `.md` header rule on touched docs; registry doc bumped.
- **Message family v1 CLOSED** stated at close.
- `Status: COMPLETED` header = the done signal.

## Close (D-074, two commits)

1. **feat** (Clair): `message.svelte` + `skin.css` + sampler.
2. **docs** (Chat): registry bump (v0.55→v0.56), `JOURNAL` J-NNN, `docs/ROADMAP.md` (M-RP5.5 C ✅ + family v1 CLOSED), CLAUDE.md PLAY, this runbook → COMPLETED.

Joe pushes both.

## Close record

CLOSED at **J-480**. feat `09e9cbe` (3 files: `message.svelte` top-level `kind` split + system arm + Option-A getter normalization; `skin.css` `.message[data-kind="system"]` grid→`1fr` + centered; `app_sampler.svelte` 2 system fixtures/cells). Doc-bridge = second commit (Joe pushes).

**CDP (9422, both accents) — real output:** `ids()` **215→219** (`count===219`, `unique===219`, 0 orphans both directions). Each system cell = `message#<id>` + `paragraph#<id>__body` (2 entries; NO `__avatar`/`__name`). System getters both cells = `{kind:'system', author:null, hasBody:true, detailsCount:0, isOwn:false, grouped:false, edited:false, deleted:false}` (Option-A normalized). `system-notice`: `data-kind=system`, `grid-template-columns:324px` (single track), `text-align:center`, no `.msg-header`, no `.entity-avatar`, h=26 (one line). `system-long`: same, h=62 (wraps, centered symmetric). Text mirror re-asserted: `text-other` `28px 288px` (own=false) / `text-own` `288px 28px` (own=true). Accent-neutral: system line `rgb(138,136,128)`=`--t3` identical client↔node; `--accent2` `#c28840` ↔ `#3a7ab0` (swap live). Screenshot `temp/m-rp5-5-c-system.png`. `vite build` clean (158 modules; two pre-existing meter/entity-avatar warnings only).

**Message family v1 CLOSED** (2 kinds `text`/`system` + `grouped`/`edited`/`deleted` states + `details` socket). Doc-bridge: registry v0.56, ROADMAP v4.49 (M-RP5.5 ✅ DONE + C ✅), CLAUDE PLAY, this runbook. Next → **M-RP5.6** `message-stream` (Phase-0 addendum → A shell+grouping+dividers → B scroll).
