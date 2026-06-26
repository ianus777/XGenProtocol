# XGen UI — Component Index
> **Status**: ACTIVE  
> Version: 0.17  
> Date: Jun 2026  
> **Last updated**: 2026-06-25  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

This file records the data-independent **catalogue** (the intended control vocabulary) and a **seed** of the data-derived spine (concept-locked rows that extend during stock-take + testing); both become the live registry of built components as the library is laid down.

---

## Built components (live registry)

The live registry of components actually authored in the tree (N-019). **Tier** marks the home crate-mirror (N-026): `common` = shared substrate, `core` = the reference component library. **Phase** marks the build-layer a component's binding demands (N-028): A = pure Svelte · B = Svelte + Tauri · C = all three layers — orthogonal to the di/dd class axis. The catalogue/seed tables below remain the *intended* vocabulary; a row graduates here once built.

| Component | Tier | Phase | Class · semantic | Root | Path | Debug | Ref |
|---|---|---|---|---|---|---|---|
| base (substrate) | `common` | — | foundation | — | `ui/common/lib/components/base/{logic,envelope,debug}.ts` | provides `use:envelope` + `window.__XGEN_DEBUG__` registry | N-023/N-024 |
| toggle | `core` | A | data-independent · boolean-toggle | `<input type="checkbox">` | `ui/core/lib/components/data-independent/toggle.svelte` | `() => $state.snapshot({ checked })` | N-022/N-024/N-030 |
| button | `core` | A | data-independent · action-trigger | `<button>` | `ui/core/lib/components/data-independent/button.svelte` | `() => $state.snapshot({ clicks, disabled, pressed })` | N-022/N-024/N-028/N-030 |
| textfield | `core` | A | data-independent · string-input family (`type`) | `<input type=…>` | `ui/core/lib/components/data-independent/textfield.svelte` | `() => $state.snapshot({ type, value })` | N-022/N-024/N-029/N-038/N-039 |
| select | `core` | A | data-independent · single-select | `<select>` | `ui/core/lib/components/data-independent/select.svelte` | `() => $state.snapshot({ value })` | N-022/N-024/N-034 |
| label | `core` | A | data-independent · display-kind caption | `<label>` | `ui/core/lib/components/data-independent/label.svelte` | `() => $state.snapshot({ text })` | N-022/N-024/N-032/N-035 |
| paragraph | `core` | A | data-independent · display-kind prose | `<p>` | `ui/core/lib/components/data-independent/paragraph.svelte` | `() => $state.snapshot({ text })` | N-022/N-024/N-032/N-036 |
| image | `core` | A | data-independent · display-kind image | `<img>` | `ui/core/lib/components/data-independent/image.svelte` | `() => $state.snapshot({ src, alt })` | N-022/N-024/N-032/N-037 |

First built `core` component, authored at M-RP2.3 as the substrate proof: verified live in **both** apps (client 9222 / node 9322) — `snapshot()` returned real `{checked:false}`, flip → `{checked:true}` confirmed live reactive reads.

`button` (M-RP2.4, J-405) is the second `core` component and a pipeline-tuning pass — action-trigger (event-out `onclick`, no `bind`) over the same N-023/N-024 envelope substrate, proving it generalizes beyond the toggle's bind-in path. Registry-verified live in both apps: `snapshot()` returned `button#quit` → `{clicks:0,disabled:false}` (client 9222) and `button#shutdown` → `{clicks:0,disabled:false}` (node 9322); both buttons function as the window close affordance, retiring the throwaway `Button.svelte` in both shells (N-019 reuse, second instance). Terminal-action note (N-028): clicking Quit/Shut-Down exits the app, so the `clicks` 0→1 delta cannot be self-redumped — the live-reactive-read proof is inherited from `toggle`; `clicks` here is registration-and-baseline observable. Pre-skin the button is **not** bare — it inherits a global `button {}` rule already in each shell (an N-025 wrinkle for the skin pass), not the normalize-only baseline.

`textfield` (M-RP2.5, J-407) is the third `core` component and the **string bind-in** path (`bind:value`) — completing the three envelope binding shapes (toggle boolean-in, button event-out, textfield string-in). Atomic native `<input>`; `type` is a **constrained prop** (M-RP2.12, → D-096) — `text`(default)`|search|email|url|tel|password` fold into this one component (all share the `<input>` root, string `bind:value`, `.textfield` skin), each with a very-weak-grey per-type inset icon (N-039); getter now `{type,value}`. `number`/`range`/`date`/`color`/`file` remain own atomics; the password reveal toggle is the `password-field` composite. Native-state surface only: `value`/`placeholder`/`disabled`/`readonly`/`id`/`pattern`/`name`; template matching via native `pattern`→`:invalid` (consumer owns the rule, skin owns the look); processor-**ready** (open to a future `common` `use:` text-processor action shared with `<textarea>`, not built here). Registry-verified live in both apps — baseline `{value:""}`, then a dispatched `input` event drove `textfield#demo` → `{value:"hello"}` (client) / `{value:"world"}` (node), **re-landing the live-reactive-read delta on the bind-in path** that the terminal button could not self-redump (N-028). Verify subtlety (N-029): driving `bind:value` over CDP needs a real dispatched `input` event, not a bare `el.value=` assignment.

`select` (M-RP2.8, J-413) is the fourth `core` component and the **first content-carrying** di — atomic native `<select>`, the bind-in `bind:value` (string) path again, but additionally carrying its `<option>` list via a normalized **`options`** prop (`string[]` or `{value,label,disabled?}[]` → `{value,label,disabled}[]`; optional `placeholder` → leading disabled `<option value="">`). The options-prop (not slotted children) keeps the root atomic + data-independent and is the surface the dd layer will later feed (N-034) — the precedent for all content-carrying components. No `multiple` (separate semantic). First component **authored and skinned in one pass** (the L2 vocabulary now exists): `.select` assembles from L2 + an `appearance:none` inline-SVG arrow, L1 empty. Registry-verified live in both apps — baseline `{value:""}` → dispatched `change` → `{value:"beta"}` (client) / `{value:"gamma"}` (node); the open option-list popup is engine-rendered, left native (N-034 / Q3). Verify subtlety: driving `bind:value` needs a dispatched `change` (N-029 restated).

`label` (M-RP2.9, J-414) is the fifth `core` component and the **first display-kind** di — value-carrying but **read-only** (the display half of the di model, N-032), vs the four interactive di. Atomic native `<label>`; value prop **`text`** (plain, **not** `$bindable` — display-di take a *semantic* value-name: label/paragraph = `text`, image = `src`); `id`; getter `{text}`; zero `<style>`. **No `for`** — association is a composite concern (N-032), wired by `textfield-group` (implicit nesting); a standalone label is valid-but-inert. Skinned by *assembling* L2 (`color: var(--t2)`, `font-size: 12px`, `line-height: 1.5`) — **no new token** (the `--fs-*` type scale is deferred to `paragraph`, where two text components justify founding it). Registry-verified live in both apps — `{text:"Demo label"}` (client 9222 / node 9322); computed-style `color: rgb(200,196,188)`=`--t2`, `12px`. Founds the display-di **verify pattern**: read-only has no event to dispatch — verify = render + computed-style (vs the interactive set's dispatch-then-delta). Finding: the rendered `display:block` is flex-item blockification from the shell's flex layout, not a skin rule (the skin sets no `display`; the inline default holds).

`paragraph` (M-RP2.10, J-415) is the sixth `core` component and the **second display-kind** di — atomic native `<p>`, single paragraph of prose; reuses the read-only display-di pattern (N-035) verbatim. Value prop **`text`** (text-node body `{text}`, never `{@html}`); getter `{text}`; zero `<style>`. The reserved inline-mark formatter (N-032) is deferred to a future `common` `use:render` action (the render-side counterpart to the edit-side `use:processor`) that owns delimiter map + whitelist + sanitisation — **not built** (D-065); the component never opens `{@html}`. This milestone also **founded the `--fs-*` type scale** (deferred here from M-RP2.9): `--fs-1: 12px` (control/caption) + `--fs-2: 14px` (body) + `--lh: 1.5`, with the four shipped skins retro-keyed (`12px`→`var(--fs-1)`, `1.5`→`var(--lh)`; no `--fs-3`/`--fs-4` seed, D-065). `.paragraph` = `--fs-2` + `color: var(--t)` (brightest — prose is content, vs label's caption `--t2`) + `--lh` + `--sp-3` spacing. Registry-verified both apps — `{text:"Demo paragraph of prose."}`; computed-style `.paragraph 14px/21px rgb(236,233,225)`=`--fs-2`/`--t`, and the four retro-keyed re-verified non-regressive at 12px/18px (N-036).

`image` (M-RP2.11, J-416) is the seventh `core` component and the **third/final display-di** — the trio (label/paragraph/image) is complete. Atomic native `<img>`, a **void element**: the first display-di whose value lives in an **attribute** (`src`), not a text-node body. Props `src` + `alt` both **required** (typed non-optional, no default — the consumer must consciously pass `alt`, including `alt=""` for decorative; a DEV-only `console.warn` catches omission, no prod throw). Getter `{src, alt}` (two fields — `alt` is part of the contract; precedent: a display-di getter carries what the semantic demands). `.image` = `border-radius: var(--rad)` only — an image's look is intrinsic, the skin just frames; no new token; sizing is a consumer concern. Demo backed by a new bundled asset `ui/assets/img-placeholder.svg` (neutral grey placeholder), imported via `$assets` — the first asset-backed demo; Vite inlined the sub-threshold SVG as a data-URI. Registry-verified both apps — `{src:"data:image/svg+xml,…",alt:"Image placeholder"}`; computed-style `border-radius 6px`/`display block`/`complete true` (N-037).

**Shape families & the M-RP2.6 retrofit (N-030 design; semantics built M-RP2.6 / J-410, visuals = M-RP2.7 skin).** `toggle` admits **checkbox / switch** shapes (skin, same component) — switch-shape now reflects `role="switch"` + `aria-checked` (built); `button` gained additive **`ariaLabel`** (→ `aria-label`) + **`mode`** (`momentary` default / `toggle`) with bind-out **`pressed`** and toggle-mode-only `aria-pressed`, getter now `{clicks,disabled,pressed}` (icon-button = a button *skin shape*; the button-style boolean toggle = button toggle-mode — neither is a new component). The *semantics* shipped M-RP2.6 (CDP-verified both apps: pressed-latch self-redump; `role="switch"` persists, `aria-checked` reflects `checked`; momentary Quit/Shut-Down carry no `aria-pressed`); the *visual* shapes (icon, switch pill, pressed bevel `[aria-pressed]`) render with the first skin file (M-RP2.7). Queued-but-unbuilt display-di (identities locked N-032): `label` (root `<label>`, short caption — association is a composite concern) **built M-RP2.9/J-414** + `paragraph` (root `<p>`, single paragraph of prose, render-side formatter seam reserved) **built M-RP2.10/J-415** + `image` (root `<img>`, `src` value + required `alt`) **built M-RP2.11/J-416** — the **display-di trio is complete** (label/paragraph/image), all value-carrying + **read-only** (display half of the di model). And `combobox` (a di composite of `textfield` + `datalist`, *not* `textfield` + `select`).

**CSS source stack (N-031, locked 2026-06-23).** Components draw from a 4-source ordered cascade: **L0** `modern-normalize.css` (pristine upstream cleaner, per-tag) → **L0** `xgen-normalize.css` (our adapted element-generic floor, per-tag, deviations in-file) → **L1** scoped `<style>` in each `.svelte` (construction/structural, per-component, as-needed, appearance-neutral — frequently empty) → **L2** one `skin.css` (all appearance, keyed by type-class, the single removable layer + live-swap target). Litmus: remove a rule — breaks function → baseline (L0/L1); only goes plain → skin. The first skin pass (M-RP2.7) founds the L2 token+treatment vocabulary; thereafter new components mostly *assemble* skin from defined vocabulary (N-019 reuse applied to styling).

**First skin pass shipped (M-RP2.7, J-412).** The N-031 stack is stood up (`ui/assets/{modern-normalize,xgen-normalize,skin}.css`; `$assets` Vite alias; `main.js` import chain; `app.css` gutted to shell chrome + per-shell `--accent*`) and the **L2 vocabulary founded** (radius/spacing scales, accent-tinted focus ring, disabled/invalid/pressed treatments, switch via `appearance:none` + `::before` thumb). `button` / `toggle` / `textfield` are now skinned, keyed `.button`/`.toggle`/`.textfield`; the N-028/N-029 global `button{}` wrinkle is **closed** (appearance re-keyed off bare `<button>` — a classless `<button>` renders the normalize-flat floor). **One shared `skin.css`, per-shell `--accent*`** (client gold/`--pr`, node blue/`--inf`). Switch shape is **skin-only** (Q5 locked) — all three components stay zero-`<style>` (L1 empty). Verified live in both apps via CDP computed-style probe + screenshots (N-033).

---

## Data-independent components

| Interaction semantic | Native element | Svelte bind | Shape variants |
|---|---|---|---|
| action-trigger | `<button>` | `onclick` | text button · icon button · icon+text · link-styled |
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

> **Build note (D-096):** the string-valued input semantics above — free-text (single line), constrained text, secret, and the search-field shape — are all served by the **one built `textfield`** via its constrained `type` prop (`text|search|email|url|tel|password`). These catalogue rows remain as the *semantic vocabulary*; they are not separate components. `number`/`range`/`date`/`color`/`file` stay their own atomics (value-type / structure differ); the password reveal toggle is the `password-field` composite.

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
- Status ACTIVE: component authoring is open since M-RP2.3 (`toggle`); M-RP2.4 added the second `core` component (`button`, action-trigger), registry-verified live in both apps and retiring the throwaway `Button.svelte`; M-RP2.5 added the third (`textfield`, free-text single-line), completing the three envelope binding shapes; M-RP2.6 added the `button` retrofit + `toggle` switch `role` (J-410); M-RP2.7 founded the L2 skin vocabulary + closed the `button{}` wrinkle (J-412); M-RP2.8 added the fourth (`select`, single-select, first content-carrying di) (J-413); M-RP2.9 added the fifth (`label`, first display-kind di, read-only caption) (J-414); M-RP2.10 added the sixth (`paragraph`, second display-di) + founded the `--fs-*` type scale (J-415); M-RP2.11 added the seventh (`image`, third/final display-di — trio complete) (J-416). New components register in **Built components** (above) when authored, carrying their Tier + Phase (N-019/N-026/N-028).
