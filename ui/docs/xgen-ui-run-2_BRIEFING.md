# XGen UI — Run 2 Briefing for Design Claude

> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

This document is the Run 2 feedback and instruction set for design Claude, based on JozefN's review of the Run 1 skeleton output. Three changes required. All are documented here with full rationale. Update the relevant skeleton files accordingly.

---

## Change 1 — Space rail thumbnails: initials + hover tooltip

**Current behaviour:** Space thumbnails in the left rail show abbreviated text (XG, G, AL, R).

**Required behaviour:** Show initials derived from the Space name. When a Space has a custom icon uploaded, show the icon instead. Initials are the permanent fallback.

**Hover tooltip** — on hover, show a compact tooltip containing:
- Full Space name
- Node address (e.g. `ws://127.0.0.1:8080`)
- Ping / latency (e.g. `4ms`)
- Member count (e.g. `4 members`)

The tooltip surfaces useful context without cluttering the rail. It is the only place this information appears in the rail — no text labels, no subtitles under the thumbnails.

**Initials derivation rule:** take the first letter of each word in the Space name, up to two letters. "XGen Protocol" → XP. "Gardening" → G. "Audio Lab" → AL. "Research" → R. All uppercase.

**Note:** this is the same pattern as user avatars. Consistent mental model — Spaces behave like entities with identities.

---

## Change 2 — Tier badge: remove from messages and member list

**Current behaviour:** `[T1]` / `[T2]` tier glyphs appear:
- Below each message in the `room.message.decorator` slot
- Next to each member name in the online members list
- In the navigation footer next to the local user's name

**Required behaviour:** Remove tier badges from all of the above locations.

**Rationale (D-038):** The Auth tier is a property of the **Node**, not of an individual member or message. It describes what authentication level the Node requires and enforces. A user authenticated at Tier 1 on one Node may be Tier 2 on another — the tier is session-scoped, not identity-scoped. Displaying it on individual messages or member entries implies it is a permanent attribute of the person, which is architecturally incorrect.

**Where tier badges remain correct:**
- **Console status bar** — `Joe / @joe [T1] · Space › #Room` — this correctly shows the current session's auth level on the connected Node. Keep as-is.
- **Node status panel** in the client sidebar — "Auth tier: T1 Community" — correctly describes the connected Node's requirement. Keep as-is.
- **Node admin dashboard** — the Node's own tier prominently displayed. Keep as-is.

**What to do:**
- Remove `xgen-tier-badge` from `room.message.decorator` slots in `client.html`
- Remove `xgen-tier-badge` from member list entries in `client.html`
- Remove `xgen-tier-badge` from the navigation footer in `client.html`
- Keep `xgen-tier-badge` in the Console status bar (`console.html`)
- Keep `xgen-tier-badge` in the Node status panel aside section

**Empty decorator slots:** once tier badges are removed, the `room.message.decorator` slot will be empty for all messages. Leave the slot in place — it is the injection point for third-party modules. Empty slots render a dashed placeholder per the existing token rule.

---

## Change 3 — Visual quality reference

JozefN noted that the Chat instance's mockup renderings have stronger visual precision than the current skeleton output — tighter spacing, more refined component rendering, better color application.

The Chat instance mockup (rendered during the Run 2 review session) is the visual reference for the following:
- Message layout — avatar, name/time/reply row, body, decorator slot spacing
- Member list — avatar + name + role layout, compact and consistent
- Status bar — monospace, left/right division, state pill rendering
- Tier glyph — compact inline square, color-coded, same height as surrounding text

**The semantic HTML structure from Run 1 is correct and must be preserved.** This is a visual refinement pass only — do not restructure the skeleton, do not remove slots, do not change token names. Apply visual precision on top of the existing structure.

If visual precision requires CSS additions or refinements to `skin-classic.css` or `tokens.css`, make them. Token slot names are locked — values are revisable.

---

## No new open questions

All questions from Run 1 remain as previously answered. No new architectural decisions are required for Run 2. The three changes above are self-contained.

---

## Formal decision recorded

**D-038** — Tier badge placement. See `DECISIONS.md`.

---

## Session log

### Session 1 — May 2026 (JozefN + Documentation Claude)
Run 2 briefing written. Three changes identified from JozefN's review of Run 1 output: Space rail thumbnail initials + hover tooltip; tier badge removed from messages and member list (D-038 — tier is a Node property, not a member property); visual quality reference pass against Chat instance mockup. Semantic structure from Run 1 preserved.
