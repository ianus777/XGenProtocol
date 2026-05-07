# XGen UI — Chat Instance Briefing for Design Claude

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

This document is the Chat instance's formal reply to the design Claude's open questions from `ui/docs/xgan-ui-overview.md` (§6) and `ui/docs/xgan-ui-debug-console-questions.md`. It records all decisions made by JozefN and the Chat instance, and provides the guidelines the design Claude requested before proceeding with skeleton iteration.

Read this document in full before touching any skeleton file. Section 7 of your overview document listed what you planned to produce next — this briefing is the gate you asked for.

---

## New documents to read before proceeding

The following documents were written after your overview and contain decisions you need:

- `docs/xgen_appendix_e_en.md` — **Application Lifecycle States** (Appendix E). This is the lifecycle proposal document you were waiting for (your Q7). Read it in full. It defines all state names, transition rules, and the `auto_connect_local` behaviour.
- `docs/xgen_ch2_architecture.md` — Session 18 added a new section: **Application Deployment Model & Lifecycle States**. The Node deployment model (systray singleton, detachable admin window, service mode) is now in Ch2. Read that section.
- `DECISIONS.md` — D-037: Node deployment model. One paragraph, worth reading for the architectural horizon note.

---

## Answers to your open questions (§6 of your overview)

### A — Vocabulary lock

**"Space" is locked** for all UI work. Ch6 uses it consistently. Do not design for late renaming — treat it as permanent.

---

### B — Permitted Space-theme override token list

The permitted override list for Layer 3 (Space theme) is:

```css
--xgen-color-primary
--xgen-color-primary-hover
--xgen-color-surface
--xgen-color-surface-raised
--xgen-color-border
```

**Never overridable by Space theme:** fonts, spacing, radius, motion, or any accessibility-critical token (`--xgen-color-error`, `--xgen-color-warning`, `--xgen-color-success`, `--xgen-color-text`, `--xgen-color-text-muted`). Brand surfaces only. Readability and accessibility are not negotiable by Space owners.

---

### C — Disable Space themes user preference

**Yes.** A Client-level preference exists: "Disable Space themes". Lives in the Identity Profile screen. When enabled, Layer 3 overrides are ignored and the application theme applies everywhere. Required for accessibility and focus needs.

---

### D — Color system

This is the most important decision in this document. Read carefully.

**The color system is derived from the application icons** (`logo/logo_proto_01_client_hd.png`, `logo/logo_proto_01_node_hd.png`). Look at them before doing any color work.

**Shared base palette — both Node and Client:**
- White family: broken whites, warm off-white, very light cream-yellow. Never pure `#ffffff`.
- Black family: broken blacks, very dark blue-black. Never pure `#000000`.
- Blue family: steel blue variants derived from the Node icon. Primary accent for infrastructure, protocol, operator surfaces, and federation status.

**Client adds:**
- Orange family: desaturated/muted amber-ochre variants as the default working color — warm, earthy, not aggressive.
- Full saturated logo orange reserved for attention-demanding moments only: unread counts, notifications, critical CTA buttons, primary actions. Not for general use.

**Design logic — blue and orange are not just two accent colors. They encode meaning:**
- Blue = infrastructure layer, Node surfaces, federation, protocol status
- Orange = identity layer, personal surfaces, the user's own messages, Spaces, and actions
- The Client lives at the intersection of user and protocol — it uses both
- The Node is pure infrastructure — it uses blue only

**Your ochre proposal** from the overview (§2) was close to the right direction. The difference is that we are not choosing a single brand accent — we are choosing a semantic color system where blue and orange each have a defined domain.

**Color values** remain deferred to UI testing per Ch6. The semantic logic above is locked, not the hex values.

---

### E — Default theme

**Dark.** Light theme exists and is operator-configurable per Ch6 §6.3, but it is not the out-of-the-box default. The warm-paper light skin from your `skin-workshop.css` is a valid alternate skin, not the primary.

---

### F — Iconography policy

**Zero emoji in chrome.** Static SVG icons only, with text labels alongside in all nav and admin surfaces. Voice rooms get a microphone SVG + "Voice" label, not 🔊. The icon set should be minimal — prefer labeled text where space allows. Fewer SVG dependencies means easier reskinning.

---

### G — Tauri window chrome

**Option 2 — full custom chrome.** Rationale: guarantees theme consistency regardless of OS settings. A dark app theme is dark top-to-bottom on every OS, every Windows configuration, every user setting. Native chrome (Option 4) can produce a light titlebar on a dark app depending on the user's OS theme — this is explicitly unacceptable.

**Titlebar content by default: app icon + app name + window controls only.** Nothing else. The titlebar is a resource, not a requirement. It is available for future use but not pre-spent.

**Window controls** follow platform conventions per OS — native traffic lights on macOS, custom on Windows.

**The lifecycle state indicator** shown in the earlier mockup is a *possibility* for future use, not a default. Do not put it in the titlebar now.

---

### H — Module visual sandboxing

This is now a hard rule:

Widget modules in injection slots **must visually inherit** the active theme tokens — a module's button looks like an XGen button, a module's text uses XGen text colors. The host app passes resolved CSS variables into the widget's webview.

Widget modules **must not escape their slot's bounding box.** Each widget renders inside a fixed-size container with `overflow: hidden`. No exceptions.

This rule must be reflected in the slot DOM structure: every injection slot is a bounded container, not an open div.

---

### I — Empty states

**Deferred to Ch6 second pass.** You cannot finalize empty states until the UI has a complete shape. The tone direction (minimal, dignified, short text, no cheerfulness, no illustrations) is noted as a starting point — not a commitment. Leave empty state content as clearly labeled placeholders in the skeleton.

---

### J — First-run flow (Client)

This was redesigned significantly. Read carefully — it is different from what Ch6 §6.5 currently implies.

**First-run is purely local. Zero network traffic.**

Step 1: Display name  
Step 2: Passphrase  
Step 3: Keypair generation (explicit confirmation — this is the moment of identity creation)  
Done. The client is alive with an identity. No Node address, no connection, no Auth Module flow.

**Rationale:** identity exists before any server knows about it. Connecting to a Node is an independent recurring activity — same action whether it is the first Space or the tenth. It is not an onboarding step.

**Auto-connect local:** after every subsequent start, the client silently scans `ws://127.0.0.1:8080/xgen` in the background (non-blocking, 2 second timeout). If a local Node responds, it connects automatically. If nothing responds, the client reaches `READY` (unconnected) silently — no error, no notification. Configurable: `auto_connect_local = true/false` in `client_config.toml`.

**SETUP state** in Appendix E reflects this — it is purely local. Node discovery was removed from the SETUP definition.

**Dev phase:** the full first-run UI can be minimal — three fields and a confirm button. The navigation design for real users will be revisited after the skeleton is stable.

---

## Console decisions (from `xgan-ui-debug-console-questions.md`)

### Q10 — Name

**Ratified.** "Console" is the name for both sides. Full names: **XGen Client Console** and **XGen Node Console**. Used in window titles, file names, and all documentation.

### Q4 — Console toggle and display

**Toggle key:** physical top-left key — `Backquote` scancode (`KeyboardEvent.code = "Backquote"`). Position-based, not character-based. Layout-independent — works on Slovak, US, German, and any other keyboard layout. The character the key produces is irrelevant.

**Display:** in-app overlay. Slides down from the top of the application window. Semi-transparent background — the app content is visible underneath. The user can see messages arriving in the Room while typing a CLI command. This is intentional and a core part of the design vision.

**Transparency:** configurable setting. Value deferred to UI testing. Do not set a final value now — leave it as a CSS variable or config parameter.

**Undocking:** planned future capability — the overlay can be undocked to a separate window for extended work sessions. Not Phase 2. Design the overlay first; undocking is an extension.

**Prompt model:** bottom-anchored `xgen>` prompt. History on Up/Down arrows. Tab completion. `?` or `help` lists commands (already in the spec). No Ctrl-K palette — the tilde key is the single access point.

**Console color scheme:** the green-on-dark VT220 scheme you proposed is **locked as the default**. JozefN specifically confirmed he likes the console colors exactly as rendered. The five scheme options (VT220, 3270 amber, VGA white, paper, xgen-tokens) remain as user-selectable alternatives.

### Q5 — Console status bar

The status bar has a **left/right division**. This layout is locked.

**Left side:** `XGen Client Console · ● STATE`

**Right side:** `DisplayName / @SpaceNick [Tn] · Space › #Room · ~ close`

Where `[Tn]` is the **tier glyph** — a compact inline square at line height, color-coded by tier:
- `T1` — green (basic verified identity)
- `T2` — blue (institutional verified)
- `T3` — amber (corporate/compliance)
- `T4` — red (government/high security)

The glyph is graphical but compact — same height as the surrounding text, fits naturally in the monospace line. Final visual design is yours.

**State indicator behavior:**
- Shows one state at a time — the current active state
- Click → dropdown showing the full state set with current state highlighted
- The dropdown doubles as a built-in reference — the operator never needs to look up what `DEGRADED_STORAGE` means
- When multiple degraded states are active: shows highest-severity state with `+N` badge, dropdown reveals all active states

**What is NOT in the status bar:** Node URL, session ID, identity fingerprint, DAG head, last error. All of these are available via `state get` command. The bar is for at-a-glance health only.

**Philosophical note for the design:** the lifecycle state in the status bar is not just a status widget. It is a statement about infrastructure transparency. A user connected to a Node in `MAINTENANCE` state sees that immediately — no mystery timeouts, no silent failures. A user seeing `DEGRADED_FEDERATION` knows their local Space works but cross-Node messages are impaired. XGen surfaces infrastructure state because users are participants in infrastructure they own, not tenants on a platform that hides its internals. Design the state indicator with this weight in mind.

---

## Technical corrections to your existing skeletons (your §5)

These were already identified by you. Confirming all five are correct:

1. **Rename CSS variables to spec names** — `--xgen-color-primary`, `--xgen-color-text`, `--xgen-color-surface-raised` etc. The token slots in Ch6 §6.2 are the contract. Your parallel names (`--accent`, `--ink-base` etc.) must go.

2. **Strip tier badges from base message header** — Ch6 §6.7 is explicit: Avatar / Display name / Timestamp / Reply link only. Tier badges are a `room.message.decorator` slot concern.

3. **Add the seven named widget slots** — empty `<div data-xgen-slot="room.sidebar.top">` etc. in both skeletons. The Node skeleton needs `node.dashboard.widget`. Both need `global.statusbar`.

4. **Add Module List screen** to both skeletons — Client and Node. Structure from Ch6 §6.8.5.

5. **Fix vocabulary** — "channel" → "room" throughout.

Additionally: **bind the Console skeleton's lifecycle state indicator** to the state names from Appendix E. Replace your placeholder labels (`state-1`, `state-2` etc.) with the real names: `INITIALISING`, `READY`, `DEGRADED_FEDERATION` etc.

---

## What to produce next

In the order you proposed in §7 of your overview, now unblocked:

1. Refactor skeleton CSS variables to spec names. No visual change — just rename.
2. Add the seven named widget slots and Module List screen to both skeletons.
3. Strip base-header decorator content. Introduce `TierBadge` and `NodeStatusIndicator` as proper components.
4. Bind Console skeleton state indicator to Appendix E state names.
5. Commit to starting token values (dark surface, blue, orange families per §D above) and re-render the Classic skin under spec-correct names.
6. Build the Space-theme Layer 3 demo — proves the override cascade works.
7. Design the Identity Setup first-run flow (three steps, purely local per §J above).
8. Empty states — leave as labeled placeholders, final content deferred.
9. Module List screen including the install/consent dialog for `user`-mode modules (Ch6 §6.8.4).

---

## One thing we want from you

Your note in §8 of the overview raised "context-on-demand" — the ability for a user to see, on demand, the Event ID of a message, its signature, its DAG parent. You flagged this as something to design intentionally.

We agree. A per-message detail panel — reachable but not visible by default — is the right answer. We do not have a specification for it yet. When you reach the message timeline in your skeleton iteration, please propose what this surface looks like and hand the proposal back to us. We will formalize it into Ch6.

---

## Session log

### Session 1 — May 2026 (JozefN + Documentation Claude)
Briefing document written covering all open questions from design Claude's overview (Q-A through Q-J) and console Q&A (Q4, Q5, Q10). All questions resolved except I (empty states, deferred to Ch6 second pass). Key decisions: Space locked as vocabulary; color system derived from application icons with blue/orange semantic split; dark default theme; full custom chrome (Option 2) for guaranteed theme consistency; first-run flow redesigned as purely local with zero network traffic; Console overlay confirmed with VT220 green-on-dark scheme locked; status bar left/right division with tier glyph locked; lifecycle state transparency framed as philosophical statement about infrastructure ownership.
