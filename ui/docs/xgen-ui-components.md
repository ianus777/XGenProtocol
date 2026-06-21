# XGen UI — Component Index
> **Status**: PENDING  
> Version: 0.3  
> Date: Jun 2026  
> **Last updated**: 2026-06-20  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

The authoritative component list (N-019): consult before authoring any UI element; reuse, never rebuild. Classified by relation to data structures (N-022): **data-independent** (control points over native HTML, keyed to an interaction semantic) and **data-derived** (representations of a data structure). This file currently records the data-independent **catalogue** — the intended vocabulary; it becomes the live registry of built components as the library is laid down.

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

*Deferred — table to be populated in the future (N-022 binding column: Appendix I / G / O / none; composed-of for composites).*

---

## How to use this file

- Consult before authoring any UI element (N-019): exists → import and reuse; genuinely absent → create in the library and register here, same step.
- Component identity is one string in three places (N-020): component name = file name = root type-class (kebab-case).
- Data-independent rows are keyed to an interaction semantic; one semantic admits several shape variants.
- Data-derived rows (future) carry a data-structure binding (Appendix I / G / O / none) and, for composites, a composed-of membership list (N-022). Layout: structural (function-critical, appearance-neutral) CSS lives with the component; all appearance lives in the one skin file, keyed by type-class — never here (N-025).
- Reading a component schema: text up to `→` is name + annotations; the `<tag class>` after `→` is the root. A native tag (`<button>`/`<input>`/`<select>`/`<textarea>`) ⇒ single-element (atomic); a `<div class="type">` ⇒ composite, with `├──` child lines as composed-of members (N-022). Same semantic can be either kind — the root tag is the discriminator (plain `checkbox` = `<input>` atomic; `checkbox-group` = `<div>` composite).
- Status PENDING: the CDP-over-WebView2 debug gate is now cleared (CDP harness built + verified), so component authoring is unblocked; Status stays PENDING until authoring is formally opened.
