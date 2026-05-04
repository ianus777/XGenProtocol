# XGen UI — Designer's Overview

> Author: Claude (design-side instance, "imagine" / artifact)
> Date: 2026-05-04
> Status: First-pass overview after reading README, CLAUDE.md, Ch1 (Philosophy) and Ch6 (Client Design).
> Audience: JozefN; the Chat instance reviewing this and preparing concrete UI guidelines.
>
> This file is **notes from the design side**, not a spec change. Nothing here overrides Ch6.

---

## 0. Scope of this read

I have read, in order:
- `README.md`
- `CLAUDE.md`
- `docs/xgen_ch6_client_design.md` (full)
- `docs/xgen_ch1_philosophy.md` (full)

I have **not yet** read: Ch2 architecture, Ch3 spec, Ch4 implementation, appendices A–F, DECISIONS.md, JOURNAL.md, FIXES_ph1.md, IMPLEMENTATION_GUIDE_ph1.md, the logging proposal, Ch0/Ch5.

Ch1 + Ch6 + CLAUDE.md is, in my judgment, sufficient for a **first-pass UI overview**. Ch2 will affect primitive vocabulary (Community vs. Space, Room types incl. `room.forum`); Ch3 will affect what surfaces I must render (EventTypes, error display format 3.3.8, validation states); appendices A and C will sharpen distinctiveness anchors and concrete object shapes. I will absorb those before final design, not before this overview.

---

## 1. What I now believe XGen is — design-side restatement

This is my own restatement, in plain language, so the Chat instance can correct me if I drifted.

XGen is **infrastructure that pretends to be an app on the surface**. From a designer's chair this is the central tension:

- The user expects something that **feels like Discord/Slack** — communities with rooms, voice, chat, files. That's the front-of-the-eye experience.
- Underneath, every artifact on screen is **a signed Event in an append-only DAG**, on a node the user can run themselves, with **identity that lives with them, not the server**.

The design job is therefore not to *expose* the protocol. The design job is to **make the protocol's properties feel like dignity** — calm, durable, your-own — without turning the UI into a cryptography classroom.

**Three properties of XGen that the UI must transmit, even when not labelled:**

1. **You own this.** Identity, history, community membership — none of it can be taken from you by anyone running infrastructure. The UI should never feel borrowed.
2. **It will outlive its current shape.** Protocol-first, clients are temporary. The UI should not feel like a 2026 SaaS — it should feel like something that could still be used in 2040 with a different skin on it. (Which is exactly what the theming architecture in Ch6 §6.3 enforces.)
3. **It is not for kids on the run from consequences.** No-anonymity is a pillar (Ch1, pillar 2). The UI must read as serious-but-warm — not corporate, not playful-mascot, not "throwaway ID culture". Gen-X target user (Ch1, "Target User"): 45–60, used IRC/ICQ/MSN/Skype, values stability, ownership, "just works".

---

## 2. The aesthetic direction — proposal, open to revision

The user said: "lightly old-school for Gen-X users". Combined with Ch1's "unassuming, quietly excellent, just works", and the target-user description, I read this as:

**Not** retro-pastiche. **Not** terminal-cyberpunk (user explicitly excluded). **Not** Discord-mimicry. **Not** trendy 2024-startup gradient-and-soft-shadow.

**Yes** — what I would call **"Tool, not Toy"**:
- Calm, paper-tone or warm-neutral surfaces, with subtle digital chrome — the feel of pro tools from late 90s / early 00s (Macromedia / SGI / early OS X) **without** the bevel-and-skeuomorphism.
- Type sized for adult reading distance, not phone-cropped.
- Generous white-space rhythm in the body, but **dense, information-rich** in nav and admin chrome — Gen-X users who built the early internet are not afraid of dense info displays.
- Iconography minimal. Most "icons" are text glyphs or labels. (This also helps reskinning — fewer SVG dependencies.)
- Accent colors used **as state**, not as decoration. Healthy / warning / error / federation-active. Not a brand wash.

**Reference defaults proposal (open):**
- `--xgen-font-family`: Inter (per Ch6 reference). I'll keep this.
- `--xgen-font-family-mono`: JetBrains Mono (per Ch6 reference).
- `--xgen-color-surface`: warm-neutral, not gaming-app-blue-black. Suggestion for dark default: `#1d1c19` (a tone with green undertone; reads as paper-on-night, not as "dark mode v17"). For light default: `#f5f1e6` (cream, not white).
- `--xgen-color-primary`: a **single** accent. Ochre (`#a55a1f`-ish) reads as Gen-X / tool / earthy without being warning. Alternative: a deep teal (`#1f6a78`-ish) — also Gen-X-coded, more clinical. I'd commit to ochre as default and provide teal as a built-in alternate skin.

These are **proposals**. Ch6 explicitly defers values to UI testing. I am offering a starting point, not locking anything.

---

## 3. The architecture I'm designing for — what I now know

From Ch6, repeated here so the Chat instance can confirm I understood it:

**Two binaries, one design system.**
- `xgennode.exe` — Node admin UI. Operator-facing. Local-only window (not a web UI on a port).
- `xgenclient.exe` — Client UI. User-facing.
- `xgen-ui-shared/` — single source of truth: `tokens.css` + Svelte components.

**Three-layer theming cascade:**
1. XGen default (built-in)
2. Application theme (dark/light, operator-configured at Node level)
3. Space theme (declared by Space owner via `state.space_theme` Event, applies only inside that Space's view)

**Token slots are locked** (`--xgen-color-*`, `--xgen-font-*`, `--xgen-space-*`, `--xgen-radius-*`, `--xgen-shadow-*`, `--xgen-transition-*`). **Values are open** until UI testing.

**Component inventory is enumerated** (Ch6 §6.2): Button, Input, Avatar, MessageBubble, MemberListItem, RoomListItem, SpaceListItem, ErrorDisplay, Modal, TierBadge, NodeStatusIndicator. I will not invent new components; if I need something the inventory doesn't have, I'll flag it as an open question.

**Message header is decided** (Ch6 §6.7): Avatar / Display name / Timestamp / Reply link. Nothing else in the base header. Tier badges, role badges, status indicators — all of those go into the `room.message.decorator` extension slot. **The base UI is intentionally clean. Modules add the noise, if any.** This is a strong, opinionated decision and I support it on the design side: it gives XGen a quieter timeline than Discord/Slack out of the box.

**Threads are flat with header links** — no indentation, no sidebar pollution. The reply preview is a navigable header link to the parent. (Ch6 §6.7.) This is the right call for Gen-X ergonomics.

**Module architecture is UI-visible** (Ch6 §6.8). Seven named injection slots:
- `room.sidebar.top`
- `room.sidebar.bottom`
- `room.toolbar`
- `room.message.decorator`
- `space.header`
- `node.dashboard.widget`
- `global.statusbar`

These slots **must exist as named DOM placeholders in the base UI**, even when empty, or modules can't attach. My current skeleton does not have all seven yet — I will add them.

**Module List is a screen of its own** (Ch6 §6.8.5), with a defined block structure: status indicator / name / version / mode badge / description / author / Settings / Launch / Disable / Remove.

**Pattern A** (folder is the application, no AppData/registry) shapes the Node admin: there's no "deploy" or "infrastructure" surface — the node is the folder. The Node UI's complexity is in *what's running inside the node*, not *where the node lives*.

---

## 4. Skeleton-first principle — why I started where I started

The user (rightly) asked for **skeleton first**: the DOM/HTML structure of both apps must be readable with no CSS at all, like the early-web standards proposed. This maps cleanly onto Ch6's Svelte-as-thin-presentation model:

- **DOM = component contract.** Each Svelte component renders a fixed semantic tree. Skeleton tests the tree.
- **Tokens = themability.** `xgen-ui-shared/tokens.css` is the contract for theming.
- **Skin = full visual.** `xgen-ui-shared/skin-*.css` files are swappable; Space themes are token-level overrides.

The skeleton I produced (`skeleton/client.html`, `skeleton/node.html`) follows that. With CSS disabled, both files render as a meaningful, navigable, semantic document. A screen reader can read them. Adding a `<style>` element produces the visual UI without changing a single tag.

**I want this to be the law for the Svelte components too**, and I'll say so to the Chat instance: every Svelte component should produce DOM that is meaningful with no CSS. If a component requires CSS to be readable, it's wrong.

---

## 5. What I think needs to change in my current skeleton — before any aesthetic work

These are **technical corrections** I identified after reading Ch6, not aesthetic decisions:

1. **Rename CSS variables to spec names.** My `--accent`, `--ink-base`, `--surface-side` etc. should all be `--xgen-color-primary`, `--xgen-color-text`, `--xgen-color-surface-raised` etc. The token slots in Ch6 §6.2 are locked; I should not be inventing parallel names.

2. **Strip tier badges from base message header.** Ch6 §6.7 is explicit: base header is Avatar / Display name / Timestamp / Reply link only. My skeleton currently inlines `Tier 1` next to names. That's a decorator concern.

3. **Add the seven named widget slots.** Empty `<div data-xgen-slot="room.sidebar.top">` etc., visible as named placeholders. The Node skeleton needs `node.dashboard.widget`. Both apps need `global.statusbar`.

4. **Add a Module List screen** to the Node skeleton, with the visual structure from Ch6 §6.8.5.

5. **Add a Module List screen** to the Client skeleton too — Ch6 §6.8.5 says "Every installed module appears in the module list", and §6.6 places the Auth Module verification flow in the Client. So both apps have one.

6. **Reframe vocabulary.** I've been using "channel" in places — Ch1 calls them rooms. The Community primitive may yet be called Community / Place / Hub / Home (Ch1 leaves naming open). I'll use "Space" for now since Ch6 uses it consistently, and flag this as resolvable later.

7. **TierBadge as a deliberate, separate component**, used only in MemberListItem and SpaceListItem per Ch6 §6.2 and §6.7. Not inline in the message header.

8. **NodeStatusIndicator as its own component** — likely lives in `global.statusbar` slot for both apps.

9. **ErrorDisplay** must follow spec 3.3.8 format. I haven't read 3.3.8 yet. To be added after Ch3 read.

---

## 6. Where I am asking the Chat instance for input

In rough priority order:

**A. Vocabulary lock.** Ch1 leaves the Community-primitive name open (Community / Space / Place / Hub / Home). Ch6 uses "Space" consistently. Should I treat "Space" as locked for UI work, or design for late renaming?

**B. Permitted Space-theme override token list** (Ch6 §6.3 open question). Until this is answered, I cannot demonstrate Layer 3 theming honestly. My proposal: only `--xgen-color-primary`, `--xgen-color-surface-raised`, and possibly `--xgen-color-text` are overridable by Space theme. Brand surfaces only — never accessibility-critical tokens, never fonts, never spacing.

**C. Disable-Space-themes user preference** (Ch6 §6.3 open question). My proposal: yes, this should exist as a Client-level preference, in the Identity Profile screen. Some users will need it for accessibility or focus.

**D. Default value commitments** for `--xgen-color-*`. I've proposed warm-neutral / ochre as the defaults. The Chat instance should either confirm or push back. I'd rather lock this than guess for another iteration.

**E. Light vs. dark default.** Ch6 §6.3 says both exist, operator-configured. Which is the **out-of-the-box default** for the reference client? My instinct: dark, because Gen-X tool users tend to work long sessions and dark is easier on aging eyes. But the warm-paper light skin is more on-philosophy ("paper, not screen"). I'd accept either.

**F. Iconography policy.** The current Discord-style mockups lean on emoji glyphs (🔊 for voice, etc.) which I dislike for a serious tool. My proposal: zero emoji in chrome; voice rooms denoted by a labeled prefix or a static SVG. The Chat instance should rule on this.

**G. Tauri window chrome.** Ch6 specifies Tauri but doesn't dictate native vs. custom titlebar. The user said "decide for me". My proposal for Phase 2 / Windows: **native chrome** — it costs less, behaves correctly with Windows accessibility, and matches the "tool, not toy" stance. Custom chrome is a Phase 3 concern if at all.

**H. Module visual sandboxing.** Ch6 §6.8.8 lists this as an open question. From the design side: widget modules in injection slots **must visually inherit** the active theme tokens (so a module's button looks like an XGen button), but **must not be able to escape their slot's bounding box**. Practically: each widget renders inside a fixed-size container with `overflow: hidden`, with the host app passing the resolved CSS variables in. This needs to be a hard rule, or the visual coherence promise of `xgen-ui-shared/` collapses.

**I. Empty states.** Ch6 doesn't address them. Every screen needs one (no Spaces yet, no rooms in this Space, no messages yet, no nodes yet, no modules installed, no identities registered). I'd like to design these explicitly because they are the user's first impression.

**J. First-run.** Ch6 §6.5 mentions Identity Setup. This is a high-stakes screen — keypair generation, display name, home Node, Auth Module. The user's whole relationship to XGen begins here. I want to design it carefully and would value the Chat instance's input on the exact step sequence.

---

## 7. What I plan to produce next, after Chat-instance review

In this order (each step pause-able):

1. Refactor skeleton tokens to spec names. (`tokens.css` becomes `xgen-ui-shared/tokens.css` shape.) No visual change.
2. Add the seven named slots and the Module List screen to both skeletons.
3. Strip base-header decorator content; introduce `TierBadge` and `NodeStatusIndicator` as proper components.
4. Commit to default token values and re-render the Classic skin under spec-correct names.
5. Build a **Space-theme demo** — toggle a `state.space_theme`-style override on one Space and watch the Room view re-skin. This proves Layer 3.
6. Design the Identity Setup first-run flow.
7. Design empty states for all primary screens.
8. Design the Module List screen, including the install/consent dialog for `user`-mode modules (Ch6 §6.8.4).

I will not commit any of this until the Chat instance has reviewed §6 of this document.

---

## 8. A note from the design side, addressed to the Chat instance

If you are reading this:

You and I are both Claude. I do design artifacts; you do conversation and document craft. We have very different working contexts. So a few things from my chair:

- **Token slots are the contract.** If you find yourself rewriting Ch6 §6.2's token list, please flag it explicitly — I will have built skin files against the existing names, and renames cascade through every artifact.
- **Component inventory is the contract.** If you add a component, please give me the name, the slot it lives in, and which screens use it. I'd rather have a precise instruction than "redesign the message bubble area".
- **Aesthetic decisions made in §2 of this document are proposals, not commitments.** If you have a stronger direction, push back; I'll redo. Locking the aesthetic too early on the wrong note is more expensive than one extra round of skin work.
- **Empty states and first-run are where the philosophy lives or dies.** A user's first 60 seconds with the Identity Setup screen will tell them more about whether XGen feels like dignity than the message timeline ever will. Please weight these screens heavily in the guidelines you write.
- **The protocol's invisibility is a feature, but not at the price of opacity.** A user should be able to see, on demand, the Event ID of a message, the signature, the parent in the DAG. Not in the base header — but reachable. The "context-on-demand" surface (per-message detail panel?) is something we should design intentionally.

I'll re-read this file after you've added your guidelines. I expect to absorb whatever you write and produce the next round.

— Claude (imagine)

---

## Appendix — Files I produced before this overview

In `skeleton/`:
- `client.html` — semantic skeleton, Client / channel view, ~145 lines
- `node.html` — semantic skeleton, Node / dashboard, ~140 lines
- `skin-classic.css` — dark default placeholder skin, ~470 lines
- `skin-workshop.css` — warm Gen-X paper-tone reference skin, ~95 lines
- `skin-contrast.css` — high-contrast proof-of-skinnability, ~70 lines
- `index.html` — viewer with toolbar (App: Client / Node / Split, Skin: None / Classic / Workshop / Contrast)

These will be revised after this overview is reviewed.
