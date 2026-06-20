# XGen UI — Component Index
> **Status**: PENDING  
> Version: 0.1  
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

*Not yet classified (deferred):* read-only display primitives (`<progress>`, `<meter>`, `<output>`) — likely data-derived; composite controls keyed to one semantic but assembled from several (combobox, tag/chip select, star rating, password show/hide) — pending the N-022 composition amendment.

---

## Data-derived components

*Deferred — table to be populated in the future (N-022 binding column: Appendix I / G / O / none; composed-of for composites).*

---

## How to use this file

- Consult before authoring any UI element (N-019): exists → import and reuse; genuinely absent → create in the library and register here, same step.
- Component identity is one string in three places (N-020): component name = file name = root type-class (kebab-case).
- Data-independent rows are keyed to an interaction semantic; one semantic admits several shape variants.
- Data-derived rows (future) carry a data-structure binding (Appendix I / G / O / none) and, for composites, a composed-of membership list (N-022). Layout lives in the component's own `.css`, never here.
- Status PENDING: component authoring is parked until the CDP-over-WebView2 debug mechanism is resolved.
