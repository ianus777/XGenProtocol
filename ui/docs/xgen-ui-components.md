# XGen UI — Component Index
> **Status**: ACTIVE  
> Version: 0.5  
> Date: Jun 2026  
> **Last updated**: 2026-06-21  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

This file records the data-independent **catalogue** (the intended control vocabulary) and a **seed** of the data-derived spine (concept-locked rows that extend during stock-take + testing); both become the live registry of built components as the library is laid down.

---

## Built components (live registry)

The live registry of components actually authored in the tree (N-019). **Tier** marks the home crate-mirror (N-026): `common` = shared substrate, `core` = the reference component library. The catalogue/seed tables below remain the *intended* vocabulary; a row graduates here once built.

| Component | Tier | Class · semantic | Root | Path | Debug | Ref |
|---|---|---|---|---|---|---|
| base (substrate) | `common` | foundation | — | `ui/common/lib/components/base/{logic,envelope,debug}.ts` | provides `use:envelope` + `window.__XGEN_DEBUG__` registry | N-023/N-024 |
| toggle | `core` | data-independent · boolean-toggle | `<input type="checkbox">` | `ui/core/lib/components/data-independent/toggle.svelte` | `() => $state.snapshot({ checked })` | N-022/N-024 |

First built `core` component, authored at M-RP2.3 as the substrate proof: verified live in **both** apps (client 9222 / node 9322) — `snapshot()` returned real `{checked:false}`, flip → `{checked:true}` confirmed live reactive reads.

---

## Data-independent components

| Interaction semantic | Native element | Svelte bind | Shape variants |
|---|---|---|---|
| action-trigger | `<button>` | `on:click` | text button · icon button · icon+text · link-styled |
| boolean-toggle | `<input type="checkbox">` | `bind:checked` | checkbox · on/off switch |
| tri-state boolean | `<input type="checkbox">` + `indeterminate` | `bind:checked` | tri-state checkbox |
| single-select (small set) | `<input type="radio">` group | `bind:group` | radio group · segmented control · button group |
| single-select (any size) | `<select>` | `bind:value` | native dropdown · styled select |
| multi-select | checkbox group / `<select multiple>` | `bind:group` | checkbox list · multi-dropdown |
| free-text (single line) | `<input type="text">` | `bind:value` | textfield · search field |
| free-text (multi line) | `<textarea>` | `bind:value` | textarea · auto-grow textarea |
| constrained text | `<input type="url\|email\|tel">` | `bind:value` | validated textfield |
| secret | `<input type="password">` | `bind:value` | password field |
| numeric (exact) | `<input type="number">` | `bind:value` | number field · stepper |
| numeric (bounded) | `<input type="range">` | `bind:value` | slider |
| date / time | `<input type="date\|time\|datetime-local">` | `bind:value` | native picker |
| color | `<input type="color">` | `bind:value` | swatch picker |
| file-select | `<input type="file">` | `bind:files` | file button |

### Composites

Data-independent composites (N-022 amendment): several native controls assembled into one control point, still keyed to a single semantic, binding = none. Schema: the header line carries name · annotations · `→ <root tag class>`; a `<div class="type">` root means composite and the `├──` child lines are its composed-of members (N-020/N-022). Children are named bare — each child's own catalogue entry defines its root.

```
combobox         keyed: single-select · binding: none → <div class="combobox">
├── textfield     free-text constituent (filter / value)
├── icon-button   list toggle (tabindex -1)
└── select-list   single-select constituent (filtered set)

tag-select       keyed: multi-select · binding: none → <div class="tag-select">
├── chip × N      selected items  (chip = sub-composite; schema TBD)
├── textfield     free-text entry / filter
└── select-list   multi-select constituent (suggestions)

star-rating      keyed: single-select · binding: none → <div class="star-rating">
└── icon-button × N   ordinal stars; value = highest lit

password-field   keyed: secret · binding: none → <div class="password-field">
├── secret-field  <input type="password"> (value-bearing)
└── icon-button   show / hide toggle (presentational only — not a second semantic)
```

*Not yet classified (deferred):* read-only display primitives (`<progress>`, `<meter>`, `<output>`) — likely data-derived.

---

## Data-derived components

UI representations (materializations) of a defined data structure, or ungrounded UI constructs (binding = none) — N-022. **Seed stage:** the rows below are concept-locked in the notes (N-pointer per row); the mockup stock-take + empirical testing *extend and refine* this registry — rows are added, never the table redrawn. Status PENDING until authoring is formally opened.

One row per component. Root tag discriminates atomic (no sub-components) vs composite (`<div class="type">` with a composed-of list) — same rule as the di table. The table is a thin spine of fixed columns; a component needing more than a row holds gets a titled paragraph under **Component detail**, pointed to from its Purpose cell.

| Component | Root | Binding | Composed-of | Purpose |
|---|---|---|---|---|
| entity-avatar | `<div class="entity-avatar">` | `IdentityRecord \| SpaceState` | — | identity/locality visual token; dynamic by kind (N-011/N-018) |
| container-list-item | `<div class="container-list-item">` | `SpaceState \| RoomState` | — | one row in a container list; dynamic by kind (N-013) |
| section-header | `<div class="section-header">` | none | — | labelled divider within a panel; ungrounded (N-022) |
| visit-card | `<div class="visit-card">` | `IdentityRecord` (tiers) | — | public profile render; tier-relative decay (N-010) |
| contact-entry | `<div class="contact-entry">` | reference + cached visit-card + private annotations | *kind TBD* | one contact-book person, three owned strata (N-009) |
| spaces-panel | `<div class="spaces-panel">` | `[SpaceState]` | container-list-item ×N + section-header | joined-Spaces panel (N-022 worked example) |
| outbox-card | `<div class="outbox-card">` | event catalog §I.2 | icon + title + description + accent + action-row | one unresolved event, friendly register (N-017) |
| console | `<div class="console">` | O + G (D-056) | scrollback + input-line | tilde-invoked CLI surface (N-015/N-016) |

*Read-only display primitives (`<progress>` / `<meter>` / `<output>`) remain deferred from the di side; likely land here as small data-derived rows when first needed.*

## Component detail

*None yet — authored per-component as a row graduates to needing depth (full composed-of tree, binding field-map, kind rationale, open questions). Pointed to from the row's Purpose cell; the spine table is never widened to hold it.*

---

## How to use this file

- Consult before authoring any UI element (N-019): exists → import and reuse; genuinely absent → create in the library and register here, same step.
- Component identity is one string in three places (N-020): component name = file name = root type-class (kebab-case).
- Data-independent rows are keyed to an interaction semantic; one semantic admits several shape variants.
- Data-derived rows (seeded 2026-06-21) carry a data-structure binding (Appendix I / G / O / none) and, for composites, a composed-of membership list (N-022). Layout: structural (function-critical, appearance-neutral) CSS lives with the component; all appearance lives in the one skin file, keyed by type-class — never here (N-025). The dd table is a thin spine of fixed columns; deeper per-component detail goes to a titled paragraph under **Component detail**, pointed to from the Purpose cell — the table is never widened.
- Reading a component schema: text up to `→` is name + annotations; the `<tag class>` after `→` is the root. A native tag (`<button>`/`<input>`/`<select>`/`<textarea>`) ⇒ single-element (atomic); a `<div class="type">` ⇒ composite, with `├──` child lines as composed-of members (N-022). Same semantic can be either kind — the root tag is the discriminator (plain `checkbox` = `<input>` atomic; `checkbox-group` = `<div>` composite).
- Status ACTIVE: component authoring is formally open as of M-RP2.3 — the first `core` component (`toggle`) is built and registry-verified live in both apps. New components register in **Built components** (above) when authored (N-019).
