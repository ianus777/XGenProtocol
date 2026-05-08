# XGen UI — Design Brainstorm
> **Status**: ACTIVE  
> Version: 0.1  
> Date: May 2026  
> **Last updated**: 2026-05-08  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

This document captures the ongoing UI design brainstorm process. It is a living document — points are added progressively as design and development needs emerge. Each point represents a design decision area, not a completed specification.

---

## Reading note for Design Claude

Points captured here are exploratory and may not yet be reconciled with locked Run briefings, the existing skeleton, or prior `DECISIONS.md` entries. If anything below conflicts with a Run briefing or a previous decision, **surface the conflict and ask Joe before acting**. We are still in the phase of identifying and thinking about specific UI elements; rigidity has not yet crystallised. Asking is correct behaviour, not delay.

This applies to scope as well as to content: if a brainstorm point is not explicitly pulled into a Run briefing, treat it as future work, not current scope — unless asking confirms otherwise.

---

## Context

The skeleton UI (Phase 2, May 2026) was designed by Design Claude and consists of two CSS layers:

- `tokens.css` — slot contract: all named CSS custom properties (colors, type, spacing, layout)
- `skin-classic.css` — structural grid + default dark visual; operator-level theme override
- `skin-workshop.css`, `skin-contrast.css` — token-only alternate skins loading on top of the above

The skeleton HTML pages are intentionally readable without any CSS — plain semantic HTML that degrades gracefully. This is a deliberate design principle that must be preserved.

---

## Point 1 — CSS file responsibility audit: AI-editor friendliness

### Problem

Both `tokens.css` and `skin-classic.css` are not practically editable by hand, and are not structured for safe editing by AI editors either. The cascade logic, safe/unsafe zones, and ownership of each rule are implicit — they exist in the developer's head, not in the files.

### Direction

Conduct a **file responsibility audit** and separate files into two categories:

**Category A — Customization targets**
Files intended to be edited for appearance customization. These receive full AI-orientation treatment:
- Top-level map comment block: "if you want to change X, find section Y"
- Named section banners with explicit ownership statements
- `SAFE TO MODIFY` and `STRUCTURAL — DO NOT MODIFY` zone markers
- Impact warnings on high-ripple tokens (e.g. `--xgen-color-primary` affects N locations)
- Dependency annotations where relevant

**Category B — Infrastructure files**
Files that must not be touched during customization. These receive a single hard banner at the top:
```
/* !! DO NOT EDIT — infrastructure file. Changes here will break the cascade. !!  */
```
Nothing else. No invitation to explore.

### Goal

Even an AI editor with no prior knowledge of the project (e.g. Gemini) can open a Category A file, read the map, and make a targeted appearance change safely without touching Category B files.

### Open questions
- Which existing files fall into which category? (requires audit session)
- Does `skin-classic.css` need to be split — its layout/grid structure is Category B, its color values may be Category A?

---

## Point 2 — Avatar as a first-class UI object

### Problem

The current skeleton uses a CSS `::before` pseudo-element as an avatar placeholder. This is not addressable in the DOM, cannot be hovered, cannot hold content, and cannot carry interactive behaviour.

### Direction

The avatar must be a **proper DOM element** — likely a `<button>` wrapping inner structure — present wherever a user is represented in the UI: message stream, member list, any other context.

**Structure:**
- Round background circle as placeholder
- Abbreviated initials displayed inside placeholder (for visual distinction when no image)
- Both sub-elements replaced by the member's own image/icon if defined
- Hover on avatar opens a **rich context menu** specific to that user in that context:
  - User information (filtered by what the member has chosen to share)
  - Utilitarian actions appropriate to context

**Self/user variant:**
The local user's avatar is a richer variant of the same object. Visually distinguished (more saturated color, as seen in sketch). Action set is different — allows acting on own identity (change picture, display name, etc.).

**Consistency:**
Same structural object used everywhere. Context (message vs member list vs elsewhere) may affect the depth of information shown in the context menu, but the avatar element itself is identical.

---

## Point 3 — Message stream: event type visual design

### Direction: three-column message grid

The message stream uses a **strict three-column CSS grid**:

| Left avatar column | Centre text column | Right avatar column |
|---|---|---|
| Fixed width | Stretches with window | Fixed width |
| Members only | All text, all messages | Self/user only |

**Member message:** avatar in left column, header elements (name, message details) left-aligned in centre column.

**User/self message:** avatar in right column, header elements right-aligned in centre column. Message text remains in centre column, unshifted.

The distinguishing gesture is **which avatar column is occupied** and **which direction the header elements align**. The text content never moves. Same structural HTML object, mirrored by CSS.

The user avatar is visually distinct from member avatars — more saturated color (attention/self tone vs muted member tone).

*Reference: sketch provided by Joe, 2026-05-08.*

### Open: full event type list

Every event type displayed in the message stream needs individual visual design. Baseline list (to be confirmed and expanded):

**Member-originated:**
- Regular message (text)
- Edited message
- Deleted message
- Reply (flat thread, parent link in message header)
- File/attachment

**Self (local user):** same types as above, mirrored layout

**System / protocol events:**
- Member joined / left room
- Member role changed
- Room created
- Room topic/description changed
- Space migration notice
- Federation status change (peer connected / disconnected)
- Node state change (DEGRADED, RECONNECTING, etc.)
- Auth tier change
- Key rotation notice

**Module-injected:** decorator slot annotations on any message (tier badges, reactions, etc.)

---

## Point 4 — UI element design as a living list

UI element design is not a one-time deliverable. Discrete UI objects must be identified across all areas of the UI — not only the message stream — and defined individually. The list expands progressively as development needs emerge.

This document (or a companion document in `ui/docs/`) serves as the ongoing record of that process.

---

## Document history

| Date | Event |
|---|---|
| 2026-05-08 | Initial brainstorm session. Four points captured. |
