# M-RP2.31 — `section` (di, collapsible disclosure container)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-04  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

The **27th `core`**. Root native `<section>`. A collapsible disclosure container: optional header (title + badge + chevron) over a body slot of arbitrary content. **di, binding = none** — interprets no domain structure. **Atomic-ish** (self-contained, the chip/star-rating shape): composes NO registering child components — header bits are raw internal elements, body is a `children` snippet — so it registers **one** getter (matrix +1/cell). Does NOT open dd. **Supersedes** the seed `section-header` divider (→ DEPRECATED).

## Skeleton (Joe-locked)
```
<section class="section">
  <h2 class="section-header">Title <span class="section-badge">2/5</span></h2>
  <div class="section-body"> …children (incl. nested <section>)… </div>
</section>
```
Collapsible → the `<h2>` content sits in a `<button aria-expanded>` + chevron; body hidden via `[data-collapsed]` (`display:none`, never `{#if}` — slot stays mounted).

## Locks
1. **Root** `<section class="section">`. `title?` → internal `<h{level}>` (`level?` default 2); no title → bare styled container (the "simple div" use).
2. **Header** solid band: `<h{level} class="section-header">` = title + optional `badge?: string` (programmatic string, e.g. "2/5", `<span class="section-badge">`) + chevron when collapsible. `actions?` slot deferred (D-065).
3. **Body** `<div class="section-body">` wrapping a `children` snippet (any content; **nesting by construction** — a `section` inside the body).
4. **Collapse** `collapsible?` (default false) + `collapsed?` ($bindable, default false). Collapsible header = `<button aria-expanded>`; body `[data-collapsed]` → `display:none`. Self-contained (owns its own body).
5. **Props/getter** `title?`/`badge?`/`collapsible?`/`collapsed?`/`level?`/`id`. Getter `{title, badge, collapsible, collapsed}`.
6. **Skin** `.section` container + `.section-header` (solid band — future bg-colour/picture-mod target) + `.section-badge` + `.section-body`; chevron masked glyph (N-052), rotates on `[data-collapsed]`. Accent-neutral.
7. **Milestone** di track, **M-RP2.31**.

## Deferred (D-065, logged not built)
- Header widget-mods: background colour / picture background.
- `actions?` header slot (extra controls).
- Filter/search — a **panel/widget** concern (data-aware), NOT `section`; it feeds `badge` + hides rows. Not this component.

## Build steps
- **A** `section.svelte` (new, `ui/core/.../data-independent/`): root `<section>`; `children` snippet; conditional `<h{level}>`/button per `collapsible`; getter.
- **B** skin: `.section` / `.section-header` / `.section-badge` / `.section-body` + chevron + `[data-collapsed]`.
- **C** sampler (DI·atomic → Display or a new Container sub-header): cells — `#plain` (title+body), `#badged` ("2/5"), `#collapsible` (collapsed toggle), `#bare` (no title), `#nested` (a `section` in the body).
- **D** CDP verify (9422, real output): getters; `SECTION`/`<h2>`/`.section-body`; `[data-collapsed]` hides body (`display:none`); `aria-expanded` flips; nested registers 2 ids; `.section*` rules in cascade; registry delta; 0 orphans.
- **E** records (D-074): N-073 ui-notes · registry (section row, 27th core) · ROADMAP (M-RP2.31 DONE + seed `section-header` DEPRECATED) · CLAUDE PLAY (→ J-459) · JOURNAL J-459 · this runbook COMPLETED. Also flip the dd-seed `section-header` row → DEPRECATED (superseded by `section`).

## DoD
- [x] `section.svelte` authored (root `<section>`, header/body, collapse, nesting), build clean.
- [x] `.section*` skin added (solid header band, chevron, `[data-collapsed]`).
- [x] Sampler cells incl. nested + collapsible.
- [x] CDP-verified (getters, collapse hides body, aria, nesting, cascade, 0 orphans).
- [x] Records closed; seed `section-header` → DEPRECATED; runbook COMPLETED.

## After this
- **dd track opens (M-RP5.0):** `entity-avatar` — the **true dd opener** (domain-bound, D-071 audit: IdentityRecord/SpaceState) → `container-list-item` → `spaces-panel` (composes `section` + rows + collapse) → `temperature-indicator` widget.
