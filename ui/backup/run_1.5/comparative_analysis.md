# UI Skeletons — Comparative Analysis (miss skeleton vs chat mockups)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-08  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Executive summary — the surprising finding

The original audit (`skeleton_audit.md`) analyzed the chat mockups in `ui/backup/fixed_samples/` and laid out a long list of div-to-semantic upgrade candidates. That audit is correct in its details, but it's framed against the wrong reference. The right reference is the comparison between two existing artifacts:

- **`ui/backup/skeleton/`** — the **miss skeleton** (Run 1). Heavily semantic HTML. Almost textbook. Falls short visually.
- **`ui/backup/fixed_samples/`** — the **chat mockups** (Run 2 visual reference). Heavily div-based. Visually polished.

**The miss skeleton already implements ~95% of the semantic structure the original audit recommended.** It has `<header role="banner">`, `<nav aria-label>` with `<ol><li><a>`, `<main>`, `<aside>`, `<footer>`, `<article>` per message, `<form>` for compose and Console prompt, `<dl>/<dt>/<dd>`, `<time datetime>`, `<details><summary>`, ARIA labels throughout. It is structurally well-formed.

So the gap between the miss skeleton and the chat mockups is **not "divs vs semantic"**. The gap is:

1. **CSS reset rigour** — the chat mockups' `* { margin:0; padding:0; box-sizing:border-box }` plus inline styling neutralizes everything. The miss skeleton relies on external `tokens.css` + `skin-classic.css` which appears to leave UA defaults partially intact for semantic tags.

2. **Visual treatment** — the chat mockups carry deliberate spacing, type scale, color tokens, micro-decisions for every container. The miss skeleton has structure but lacks the visual coding density.

3. **Run 2 evolutions** — the chat mockups apply D-038 (no tier badges in messages or member list), D-039 (action buttons in nav-footer), Run 2 Change 1 (Space rail initials + hover tooltips). The miss skeleton predates these decisions and still shows tier badges in messages, no nav-footer buttons, and full text labels in the rail.

The implication: **Phase B is not a div-to-semantic conversion**. It's a merge of the miss skeleton's structure with the chat mockups' visual treatment, plus application of the Run 2 changes. Much smaller scope than the original audit suggested.

---

## What the miss skeleton already gets right

These items appear correctly in the miss skeleton. They do **not** require conversion work:

| Concern | Miss skeleton implementation |
|---|---|
| App banner | `<header role="banner" aria-label="Application">` |
| Space switcher | `<nav aria-label="Spaces">` containing `<ol>` of `<li><a>` |
| Side navigation | `<nav aria-label="Rooms in {space}">` |
| Nav sections | `<section aria-labelledby>` with `<h3>` |
| Nav items | `<a href>` inside `<li>` inside `<ol>` |
| Nav footer | `<footer aria-label>` |
| Main content | `<main aria-labelledby>` |
| Page header inside main | `<header>` inside `<main>` with `<h1>` |
| Search | `<form role="search">` with `<label>` and `<input>` |
| Messages list | `<ol aria-label="Messages">` |
| Each message | `<li><article aria-labelledby><header><p>…</p></header><p>body</p></article></li>` |
| Timestamps | `<time datetime="…">` |
| Event attachment | `<dl aria-label>` with `<dt>`/`<dd>` |
| Compose box | `<form aria-label>` with `<label>`, `<textarea>`, `<button type="submit">` |
| Sidebar | `<aside aria-label>` |
| Status cards | `<article aria-labelledby>` with `<dl>` |
| Status bar | `<footer data-xgen-slot>` |
| Dashboard cards (Node) | `<ol>` of `<li><article aria-labelledby>` |
| Hosted spaces (Node) | `<ol>` of `<li><article>` |
| Live log (Node) | `<ol aria-label>` of `<li><article data-level>` |
| Action buttons | `<button type="button">` and `<button type="submit">` |
| Console session header | `<header role="banner" aria-label="Console session">` |
| Console preferences | `<nav aria-label>` with `<ol>` of `<li>` containing `<label>` and `<select>` |
| Console session metadata | `<details><summary>` with `<dl>` |
| Console log stream | `<main>` with `<ol aria-label="Log entries">` |
| Console prompt | `<form aria-label="Issue command">` with `<label><code>xgen></code></label>`, `<input>`, `<button type="submit">` |
| State reference dropdown | `<dl>` with `<dt data-state>`/`<dd>` |

Effectively the entire structural backbone of the original audit's "Conversion conventions" section is **already implemented**. The work that audit described was based on a flawed premise (that we'd be starting from div-heavy and adding semantics). We're starting from semantic-heavy and adding visual polish.

---

## What the miss skeleton actually still needs (semantic refinements)

A small set of items where the miss skeleton's semantics are not quite right. These are the genuine semantic-conversion items:

1. **Reply links use `<a href="#" data-field="reply-link">`** — replies are actions, not navigation. Should be `<button type="button">`. Minor but real.

2. **Console state-dropdown `<dt>` items are clickable** — `<dt>` is a definition term, not interactive. Wrap each clickable in `<button>`, or restructure as `<menu>` of `<button>` with `<dt>` semantics preserved as data attributes or ARIA.

3. **`~ close` link in Console status bar uses `<a href="#">`** — close is an action, should be `<button>`. Currently uses anchor with click handler.

4. **`xgen-state-indicator` (state pill) is a clickable `<span>`** in client/node skeletons (not focusable) — should be `<button>` if interactive, or `<output aria-live="polite">` if purely passive. The console version handles this slightly differently (clickable span with explicit script binding) but has the same focus-ability issue.

5. **Member entries in client.html have no `<article>` wrapper** — the messages do, but members don't. Minor consistency item.

6. **Node `<section aria-labelledby="actions-h">` wraps buttons in `<p>`** — `<p>` is paragraph content, not a button group. Could be `<menu>` or just stay as `<p>` (technically valid but conceptually odd).

That's the entire structural diff. Six items, all minor.

---

## What the miss skeleton needs for Run 2 alignment

These are not semantic issues — they're applications of decisions made after the miss skeleton was authored:

### D-038 — Tier badge placement

Miss skeleton currently shows `<span class="xgen-tier-badge">` in three places that D-038 removes:

- Inside `room.message.decorator` slot per message (client.html lines 105–107, 129–131, 157–159)
- Next to member names in the online list (client.html lines 187, 194, 201)
- In the nav-footer next to local user name (client.html line 69)

These need to be removed. The chat mockups already implement this correctly. Tier badges remain in: Console status bar, Node status panel in client sidebar, Node admin dashboard.

### D-039 — Nav-footer action buttons

Miss skeleton's nav-footer contains only identity/health text. Chat mockups add Disconnect/Exit (Client) and Restart/Stop (Node) buttons. Add these.

### Run 2 Change 1 — Space rail thumbnails

Miss skeleton shows full Space names as text labels (`<a>XGen Protocol</a>`, `<a>Gardening</a>`). Chat mockups show two-letter initials (XP, G, AL, R) inside thumbnail buttons with hover tooltips containing full name + node address + ping + member count. Apply this to both client.html (Spaces) and node.html (Hosted spaces).

### Run 2 Change 3 — Visual quality refinement

The chat mockups are the visual reference for: message layout, member list compactness, status bar rendering, tier glyph styling. The miss skeleton's structure is correct; the visual treatment via CSS needs to match the chat mockups.

---

## Where the gap actually lives — CSS reset + visual treatment

The miss skeleton looks less polished than the chat mockups primarily because:

### CSS reset coverage

The chat mockups use embedded `<style>` with:

```css
* { box-sizing: border-box; margin: 0; padding: 0; }
```

…plus hand-written rules for every container, fully defining every property they care about.

The miss skeleton uses external `tokens.css` + `skin-classic.css`. Inspection of those files would reveal whether the reset is comprehensive. The expected gaps for an external-skin approach:

- `<h1>` through `<h6>` retain UA `font-size`/`margin` unless explicitly reset
- `<ol>`/`<ul>` retain `list-style: disc` and `padding-left: 40px` unless reset
- `<button>` retains UA chrome unless reset
- `<fieldset>` retains its default border and padding
- `<dl>` retains its default block layout

These accumulating UA defaults are exactly what the user's "documents vs applications" intuition was identifying. The miss skeleton renders semantic tags with their default document-style appearance; the chat mockups render every tag from a flat baseline.

The fix is a Preflight-equivalent reset added to `tokens.css` (or a new `reset.css`):

```css
h1, h2, h3, h4, h5, h6 { font-size: inherit; font-weight: inherit; }
ol, ul { list-style: none; }
nav ol, nav ul, aside ol, aside ul, main ol, main ul,
header ol, header ul, footer ol, footer ul,
article ol, article ul { padding: 0; }
button { background: none; border: 0; font: inherit; color: inherit; cursor: pointer; }
fieldset { border: 0; padding: 0; min-width: 0; }
legend { padding: 0; }
input, textarea, select { font: inherit; color: inherit; }
table { border-collapse: collapse; }
img, picture, video, canvas, svg { display: block; max-width: 100%; }
```

After this reset is in place, the miss skeleton's semantic tags will render as flatly as the chat mockups' divs do. From there, visual polish is added back through `skin-*.css` rules targeting specific classes/attributes — same approach the chat mockups use, just applied to semantic tags instead of div+class combinations.

### Visual coding density

Beyond reset, the chat mockups have deliberate styling for every visual element: Space rail thumbnails, tooltip popovers, message gutter alignment, member list compactness, status bar typography, state pill rendering, tier glyph dimensions. The miss skeleton's `skin-classic.css` likely has fewer rules and lower density.

This is the genuine visual-design work — and it's already done in the chat mockups. The path is to extract those decisions and rewrite them as CSS targeting the miss skeleton's semantic selectors (e.g. `nav[aria-label="Spaces"] li > a` instead of `.space-btn`).

---

## Where the gap is NOT

To correct course on the original audit:

- **It is not a div-to-semantic restructuring task.** The structure is already there.
- **It is not a layer-by-layer port to Svelte.** The miss skeleton is already a viable Svelte template; the visual gap is the only barrier.
- **It is not a question of whether semantic HTML can match div-based polish.** The miss skeleton proves the structure exists; the chat mockups prove the visual quality is achievable. The merge is what hasn't been done.

---

## Corrected roadmap

### Milestone 1 — Convention freeze and review

This document and `skeleton_audit.md` together. JozefN reviews and signs off, or requests changes, before any further work.

### Milestone 2 — Visual extraction from chat mockups

Extract the visual decisions embedded in the three chat mockups (`ui/backup/fixed_samples/xgen-mockup-*.html`) into structured CSS rules. Rewrite the rules to target semantic selectors (matching the miss skeleton) rather than div+class selectors.

Output: an updated `tokens.css` and revised skin files (`skin-classic.css` rewritten, plus possibly a new `skin-dark.css`) that, when loaded by the miss skeleton, produce the chat mockups' visual quality.

### Milestone 3 — Apply Run 2 changes to the miss skeleton

In a copy of the miss skeleton (or in run_1.5):
- Remove tier badges from message decorator slot, member list, and nav-footer (D-038)
- Add Disconnect/Exit and Restart/Stop buttons to nav-footers (D-039)
- Convert Space rail labels to initials + hover tooltips (Run 2 Change 1)
- Apply the six minor semantic refinements listed in "What the miss skeleton actually still needs"

### Milestone 4 — Phase A: model new element types

Per JozefN's plan: model the unmodelled element types (message kinds, status variants, etc.) from the Run 2 briefing list. **Two paths** worth deciding between:

- **Path A1 — model in chat-mockup div/span style first**, then convert. Matches JozefN's stated preference for "see how it looks on a pure clean UI table" before semantic discipline. Lower upfront constraint; visual decisions stay free.
- **Path A2 — model directly in semantic-with-reset style**, since the miss skeleton + reset CSS already proves visual quality is achievable from semantic markup. No conversion step needed afterward.

Recommendation: A1 if the new elements are visually exploratory and decisions aren't yet locked; A2 if the design pattern is well-understood from existing elements (just a new variant). Decide per-element.

### Milestone 5 — Visual regression check

Side-by-side render: miss skeleton (with new CSS) vs chat mockups. Confirm visual quality matches. Document any acceptable deltas.

### Milestone 6 — Reset CSS and conventions become permanent

The reset rules become a stable `reset.css` (or a dedicated `@layer reset` in `tokens.css`). The conventions in `skeleton_audit.md` become `docs/ui_semantic_conventions.md`.

### Milestone 7 — Svelte port

The miss skeleton (now with chat-mockup visual quality applied) is the Svelte port template. Component-by-component conversion. The visual treatment carries over via CSS; the semantic structure carries over via component markup.

---

## Open questions

1. **Should the miss skeleton (with Run 2 changes applied) become the new active reference**, replacing both the current `xgen-mockup-*.html` files and the role of fixed_samples? Recommendation: yes, after Milestones 2–4 are complete. The current `xgen-mockup-*.html` files are a partial merge attempt; they can be deprecated.

2. **Where should the new (Run 2-aligned + chat-mockup-styled) miss skeleton live?** Options: in `ui/run_1.5/`, in `ui/` directly (replacing the current xgen-mockup files), or in a new `ui/skeleton-v2/` folder. Recommendation: `ui/run_1.5/` for the work-in-progress version; promote to `ui/` once validated.

3. **Should the reset CSS approach use `@layer` (CSS Cascade Layers, supported in all Tauri webviews)?** Modern browsers support `@layer reset { … }` for explicit cascade isolation. This would let `tokens.css` declare reset rules in a layer that's guaranteed to lose to subsequent rules. Worth considering for clean precedence semantics.

4. **For Phase A (Milestone 4) — A1 or A2?** This is genuinely a per-element call. Decide as new element types are taken on.

5. **The `<select>` rendering inconsistency across Tauri webviews remains.** Console preferences use real `<select>` in both miss skeleton and chat mockups. Acceptable as-is; flagged for awareness.

---

## Reference

- `ui/backup/skeleton/` — miss skeleton (Run 1 semantic structure)
- `ui/backup/fixed_samples/` — chat mockups (Run 2 visual reference)
- `ui/run_1.5/skeleton_audit.md` — original audit (chat mockups analysis; correct in detail, framed against wrong reference)
- `ui/docs/xgen-ui-run-2_BRIEFING.md` — Run 2 changes (D-038, D-039, Space rail)
- `DECISIONS.md` D-038, D-039
