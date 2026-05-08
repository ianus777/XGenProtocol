# UI Skeleton Audit — Run 1.5
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-08  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

> **Note (2026-05-08):** This audit analyzes the chat mockups in isolation. After also reviewing the miss skeleton in `ui/backup/skeleton/`, the framing changed — the miss skeleton already implements ~95% of the semantic structure recommended below. See `comparative_analysis.md` for the corrected take. This document remains useful as a detailed conversion reference and rule set; it is **not** a description of work to be done from scratch.

---

## Purpose

Audit the visual reference mockups in `ui/backup/fixed_samples/` (`xgen-mockup-client.html`, `xgen-mockup-node.html`, `xgen-mockup-console.html`) to inventory their use of `<div>`/`<span>` versus semantic HTML, identify upgrade candidates for the Phase 2 Svelte port, and define the conversion conventions to apply uniformly across all skeletons — including the new element types yet to be modelled.

The fixed_samples are the visual reference. They were built div/span-heavy on purpose to achieve visual precision (no UA defaults to fight). The audit's job is to plan how to preserve that visual quality while restoring structural truth.

---

## Methodology

For each of the three primary mockups: high-level tag inventory, classification of every `<div>`/`<span>` into one of three buckets, specific issues, and a recommended conversion table.

The three buckets:

- **Justified** — pure visual scaffolding (flex row, divider, positioning anchor, glow layer, slot anchor). Stays as `<div>` or `<span>`. No action.
- **Upgrade candidate** — playing the role of a real semantic element. Convert during Svelte port.
- **Ambiguous** — could go either way; flagged for deliberate decision.

---

## File 1 — `xgen-mockup-client.html`

### Tag inventory

**Semantic tags already in correct use:** `<h1>`, `<h2>`, `<h3>`, `<p>`, `<time>`, `<a>` (with caveats — see issues), `<dl>`, `<dt>`, `<dd>`, `<input>`, `<button>`, `<code>`, `<strong>`. The leaves of the document are largely correct.

**Layout-level structure is all `<div>` with classes:** `.wrap`, `.rail`, `.sidenav`, `.main`, `.aside`, `.statusbar`. These are landmarks pretending to be generic containers.

**Repeating-list structure is all `<div>` with classes:** spaces, nav-items, messages, members, status-rows, hosted spaces. None wrapped in `<ol>`/`<ul>`.

### Upgrade candidates

| Current | Proposed | Reason |
|---|---|---|
| `<div class="wrap">` | drop, or stylistic `<div>` | Layout grid root. `<body>` can be the grid root directly. |
| `<div class="rail">` | `<header role="banner">` containing `<nav aria-label="Spaces">` | Application identity + space switcher. |
| `<div class="space-btn">` | `<a href="…">` inside `<li>` | Each is clickable navigation. |
| `<div class="tooltip">` | `<span data-xgen-tooltip>` | Inline-by-purpose; data attribute carries kind. |
| `<div class="sidenav">` | `<nav aria-label="Rooms in {space}">` | Rooms list = navigation. |
| `<div class="sidenav-title">` | `<h2>` | Names the navigation section. |
| `<div class="nav-section">` | `<section aria-labelledby="…">` | Grouped navigation subsections. |
| `<div class="nav-label">` | `<h3>` (inside the section) | Section heading. |
| `<div class="nav-item">` | `<a href="…">` inside `<li>` inside `<ol>` | Each = navigation link in equal-rank list. |
| `<div class="nav-footer">` | `<footer>` | Footer of the navigation panel. |
| `<div class="main">` | `<main aria-labelledby="room-title">` | Document main content. |
| `<div class="room-header">` | `<header>` (inside `<main>`) | Room title bar. |
| `<div class="room-toolbar">` | `<div data-xgen-slot="…">` | Slot — keep as div; Phase 2 injection point. |
| `<div class="messages">` | `<ol aria-label="Messages">` | Ordered list of messages. |
| `<div class="msg">` | `<li><article aria-labelledby="…">` | Self-contained content unit in ordered list. |
| `<div class="avatar">` | `<span data-avatar data-kind="…">` | Inline visual element with initial. |
| `<div class="msg-meta">` | `<header>` (inside the `<article>`) | Message header — author, time, action. |
| `<div class="msg-body">` | `<p>` | Message text = paragraph content. |
| `<div class="system-msg">` | `<p data-kind="system">` with `role="status"` | Visual variant of message; status semantics. |
| `<div class="event-card">` (wraps `<dl>`) | drop wrapper, keep `<dl>` | Wrapper is just chrome. |
| `<div class="compose">` | `<form aria-label="Compose">` | Submission needs `<form>` for Enter-to-send. |
| `<div class="compose-actions">` | `<div>` (justified) | Pure flex-row grouping. |
| `<div class="aside">` | `<aside aria-label="Members and node status">` | Sidebar landmark. |
| `<div class="member">` | `<li><article>` inside `<ol>` | List item. |
| `<div class="member-av">` | `<span data-avatar>` | Inline avatar. |
| `<div class="member-name">` | `<p><strong>` | Name. |
| `<div class="member-role">` | `<p>` | Role. |
| `<div class="status-card">` | `<article>` or `<section>` | Self-contained block. |
| `<div class="status-row">` | `<div>` (justified) | HTML5-valid `<dl>`/`<div>` grouping for grid layout. |
| `<div class="statusbar">` | `<footer data-xgen-slot="global.statusbar">` | Global footer. |

### Specific issues

1. **Reply links use `<a href="#">` but are actions, not navigation.** Should be `<button type="button">`. As-is, screen readers announce them as links and they pollute history.
2. **Compose box is a `<div>`, not `<form>`** — Enter does not submit because there's no `<form>` ancestor.
3. **The `tier-pill` and `state-pill` are `<span>`s without ARIA.** Add `aria-label` (e.g. `"Authentication tier 1"`) and consider `<output aria-live="polite">` for state-pill so transitions are announced.
4. **`<a href="#">~ console</a>`** — semantically OK if production behaviour is "navigate to Console view"; convert to `<button>` if it's a toggle action.
5. **The `system-msg` block** — could carry `role="status"` for assistive tech.

---

## File 2 — `xgen-mockup-node.html`

### Tag inventory

Same overall pattern as client. Dashboard cards correctly use `<h3>` for titles. `<dl>` is used cleanly for Node identity.

### Upgrade candidates

(Mirrors client.html — listing only what's distinct.)

| Current | Proposed | Reason |
|---|---|---|
| `<div class="cards">` | `<ol aria-label="Overview">` | Grid of dashboard cards = ordered list. |
| `<div class="card">` | `<li><article aria-labelledby="…">` | Each card is self-contained in a list. |
| `<div class="val">` | `<p><strong>` | Numeric value. |
| `<div class="sub">` | `<p>` | Subtitle. |
| `<div class="spaces">` | `<ol aria-label="Hosted spaces">` | List of spaces. |
| `<div class="space-row">` | `<li><article>` | Self-contained space card. |
| `<div class="log">` | `<section>` containing `<ol aria-label="Recent log lines">` with `aria-live="polite"` | Live log. |
| `<div class="log-line">` | `<li><article data-level="…">` | Each entry is a self-contained content unit. Inline spans (ts/lvl/code/msg) stay as `<span>`. |
| `<div class="actions">` | `<menu>` or `<div>` (justified) | Action button row. |
| `<div class="identity-card">` | `<article>` or `<div>` (justified) | Just chrome around a `<dl>`. |

### Specific issues

1. **"Manage" buttons on space rows are `<button>` already** — correct.
2. **The state-pill in nav-footer (`<span class="state-pill-nav">`) is decorative.** Should be `<output aria-live="polite">` since it reflects live Node state.
3. **`.cards .card .val`** — large numeric value; semantic emphasis (`<p><strong>`) recommended over plain `<div>`.

---

## File 3 — `xgen-mockup-console.html`

### Tag inventory

Console form controls (`<select>`, `<option>`, `<label>`, `<input>`, `<button>`) are correct semantic tags inside div containers. Layout-level structure and the log stream are div-soup.

### Upgrade candidates

| Current | Proposed | Reason |
|---|---|---|
| `<div class="console-head">` | `<header aria-label="Console session">` | Session header landmark. |
| `<span class="console-title">` | `<strong>` inside the `<header>` | Console name. |
| `<span class="console-session">` | `<span data-field="session-id">` | Session ID — span fine. |
| `<div class="console-prefs">` | `<form aria-label="Console preferences">` | Contains form controls. |
| `<div class="log-stream">` | `<main aria-label="Console log stream">` containing `<ol aria-label="Log entries">` with `aria-live="polite"` | Main content; ordered list. |
| `<div class="log-line">` | `<li><article data-level="…">` | Self-contained entry. |
| `<div class="prompt-area">` | `<form aria-label="Console prompt">` | Submit-on-Enter requires `<form>`. |
| `<span class="prompt-label">xgen&gt;</span>` | `<label for="prompt-input">` | Labels the input. |
| `<div class="prompt-hint">` | `<p>` | Help text. |
| `<div class="status-bar">` | `<footer data-xgen-slot="global.statusbar">` | Global footer. |
| `<span class="state-pill">` (clickable) | `<button>` | Currently span with click handler — not focusable. |
| `<div class="state-dropdown">` | `<div role="dialog" aria-label="State reference">` | Modal/dialog popup. |
| `<dt>` (clickable items in dropdown) | `<button>` inside the `<dt>`, or restructure as `<menu>` of `<button>` | `<dt>` is a definition term, not interactive. |
| `<span class="status-close">~ close</span>` | `<button>` | Click-to-close action. |

### Specific issues

1. **`prompt-label` is `<span>`, not `<label>`** — input is unlabelled to assistive tech. The visible "xgen>" is a perfect `<label>`.
2. **Prompt area is a `<div>`, not `<form>`** — Enter does not submit.
3. **State pill is a clickable `<span>`** — not focusable, not announced as a button.
4. **Dropdown `<dt>` items are clickable** — `<dt>` is not interactive. Wrap each in `<button>` or use `<menu>` of `<button>`.
5. **`status-close` is a span with click handler** — should be `<button>`.
6. **`<select>` for color scheme is genuinely a `<select>`** — this is the one place we hit Tauri three-webview rendering inconsistency. Acceptable: low-stakes preference setting.

---

## Cross-cutting findings

### What's already right

- Heading hierarchy (`<h1>` → `<h2>` → `<h3>`) is sensible.
- `<dl>`/`<dt>`/`<dd>` used correctly for key/value displays (Node identity, Auth tier, status cards).
- Native form controls (`<input>`, `<select>`, `<button>`) used where they belong — except `<form>` wrappers are missing in three places.
- `<time>` for timestamps. `<code>` for monospace identifiers. `<strong>` for emphasis.

### What's consistently wrong

1. **Layout regions are unsemantic divs.** Same pattern in all three files: `.wrap` grid root with `.rail` / `.sidenav` / `.main` / `.aside` / `.statusbar` children. Conversion to `<header>` / `<nav>` / `<main>` / `<aside>` / `<footer>` is mechanical and uniform.

2. **Repeating equal-rank items are loose divs.** Spaces, nav-items, messages, members, dashboard cards, hosted spaces, log lines — flat `<div>` repetitions. Conversion: container becomes `<ol>` (or `<ul>` if order irrelevant), each item becomes `<li>`, content wraps in `<article>` if self-contained.

3. **Clickable elements aren't buttons or links.** `.space-btn`, `.nav-item`, `.state-pill`, `.status-close`, dropdown `<dt>` items, "Reply" pseudo-links — all interactive without being focusable interactive elements. Actions → `<button>`, navigation → `<a href>`.

4. **Form controls live outside `<form>`.** Compose box, Console prompt, Console prefs — all have inputs and submit buttons inside plain `<div>`. Wrap each in `<form>`.

### What stays as `<div>` / `<span>` (justified)

- `.tooltip` containers (inline visual chrome)
- `.compose-actions`, `.actions`, `.statusbar-left`, `.statusbar-right` — pure flex-row groupings inside semantic ancestors
- `.event-card` outer wrapper (the `<dl>` inside is the real content; wrapper is just chrome)
- `.status-row` wrappers inside `<dl>` (HTML5-valid `<dl>`/`<div>` grouping)
- All inline visual spans: `.tier-pill`, `.state-dot`, avatar initials, `.status-sep`
- Slot anchors: `<div data-xgen-slot="…">` — Phase 2 injection points with no inherent semantics
- The body-level `.wrap` grid root (replaceable with `<body>` directly, or kept as stylistic root)

---

## Conversion conventions

These rules apply uniformly across all skeletons — existing three plus any new element types modelled in Phase A.

### Layout

| Region | Tag |
|---|---|
| App root | `<body>` directly (drop `.wrap`) or stylistic `<div>` |
| Top rail / app banner | `<header role="banner">` |
| Space switcher inside rail | `<nav aria-label="Spaces">` containing `<ol>` |
| Side navigation | `<nav aria-label="…">` |
| Nav sections | `<section aria-labelledby="…">` with `<h3>` |
| Main content area | `<main aria-labelledby="…">` |
| Sidebar / right panel | `<aside aria-label="…">` |
| Status bar | `<footer data-xgen-slot="global.statusbar">` |
| Page header inside main | `<header>` (inside `<main>`) |

### Repeating items

- Container with N equal-rank items → `<ol>` (or `<ul>` if order irrelevant)
- Each item → `<li>`
- Each item's content (if self-contained) → `<article>` inside the `<li>`

Applies to: spaces, nav items, messages, members, dashboard cards, hosted spaces, log lines, and any new repeating element type added later.

### Interactive elements

- In-page action → `<button type="button">`
- Navigation (changes URL or active section) → `<a href="…">`
- Form submission → `<button type="submit">` inside `<form>`
- Clickable text → never a `<span>` or `<dt>` with click handler

### Forms

- Any container with `<input>`/`<select>`/`<textarea>` plus a submit button → `<form>`
- Every `<input>` has a `<label for="…">` (visible or `aria-label`)
- Compose box, Console prompt, Console prefs all become `<form>`

### Live regions

- State pill → `<button>` if interactive (clicking opens dropdown), wrapping a `<span>` with `aria-live="polite"` for the textual state
- Live log stream → `<ol>` with `aria-live="polite"` or `role="log"`
- System / status messages → `role="status"`

### Inline visual spans

Stay as `<span>`. Use `data-*` attributes for kind (`data-avatar data-kind="primary"`, `data-tier="1"`, `data-state="READY"`). Avatars carry `aria-label` if the initial isn't self-explanatory.

### Slots

`<div data-xgen-slot="…">` stays as `<div>`. Slots are Phase 2 module injection points and have no inherent semantics until something is mounted into them.

---

## Visual preservation guarantee

The conversion will not lose the visual quality the chat mockups achieve, IF the reset is extended to cover the new semantic tags. The existing `* { margin:0; padding:0; box-sizing:border-box }` covers most of it, but specific UA defaults need additional zeroing:

```css
h1, h2, h3, h4, h5, h6 { font-size: inherit; font-weight: inherit; }
ol, ul { list-style: none; }
nav ol, nav ul, aside ol, aside ul, main ol, main ul,
header ol, header ul, footer ol, footer ul { padding: 0; }
button { background: none; border: 0; font: inherit; color: inherit; cursor: pointer; }
fieldset { border: 0; padding: 0; }
article, section, nav, aside, header, footer, main { display: block; }
```

This is essentially Tailwind's Preflight philosophy: neutralize UA defaults for semantic tags so they render as flat as `<div>`s would, then add back type scale and spacing through scoped component styles. End result: **semantically rich HTML that visually matches the div-based reference**.

---

## Roadmap

### Milestone 1 — Convention freeze (this document)
Audit complete. Conversion conventions defined. JozefN reviews and signs off (or requests changes) before any conversion or new-element modelling begins.

### Milestone 2 — Phase A: model new element types in pure div/span mode
The element types from the Run 2 briefing list (message kinds, status variants, etc.) get modelled in the same div/span flat-tag environment as the existing fixed_samples mockups. No semantic constraints — visual decisions only. New mockups go in `ui/run_1.5/` alongside this audit.

### Milestone 3 — Phase B: semantic conversion pass
Apply the conversion conventions from this document to ALL skeletons together (existing three + new ones from Milestone 2). Output: a parallel set of semantic skeletons, naming TBD. The fixed_samples versions stay as the visual reference; the semantic versions become the Svelte port template.

### Milestone 4 — Visual regression check
Side-by-side render of fixed_samples vs semantic versions. Confirm visual quality is preserved. Document any acceptable deltas (form control rendering, scrollbar variance, focus rings).

### Milestone 5 — Reset CSS extracted
The semantic-tag-neutralization rules from "Visual preservation guarantee" become a permanent stylesheet. All future skeleton work and the Svelte port import this reset.

### Milestone 6 — Svelte port begins
With conventions frozen and semantic skeletons validated, component-by-component conversion to Svelte starts. Each Svelte component matches its semantic skeleton.

---

## Open questions

1. **Folder layout for new mockups.** Should Phase A new-element mockups live in `ui/run_1.5/` or a separate folder? `run_1.5` works if it's understood as the umbrella for the convention-freeze + new-element work.
2. **Where do semantic versions of existing mockups land?** Recommendation: parallel folder; fixed_samples stays as visual reference.
3. **`<select>` rendering inconsistency across Tauri webviews — accept or replace?** Recommendation: accept for Console preferences (low-stakes). Reconsider if a `<select>` appears on a high-visibility surface.
4. **Reply "links" — `<button>` or `<a>`?** Recommendation: `<button>`. Replies are actions, not navigation.
5. **Should this audit's conversion conventions be lifted into a separate, permanent doc** (e.g. `docs/ui_semantic_conventions.md`) so it survives outside the run_1.5 folder context? Recommendation: yes, after sign-off.
