# XGen Protocol — Chapter 6: Client Design
> **Status**: ACTIVE  
> Version: 0.4  
> Date: May 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Overview

Chapter 6 specifies the XGen client applications — what they look like, how they behave, and how UI decisions feed back into Phase 2 protocol requirements.

Two applications are specified here: the **Node admin UI** (`xgen-node.exe`) and the **Client UI** (`xgen-client.exe`). Both share a common design system and component library. Both are single executables following the Pattern A deployment model (spec: `IMPLEMENTATION_GUIDE_ph1.md`).

Chapter 6 is written in two passes. The first pass (this document) captures confirmed architectural decisions made before Phase 1 implementation. The second pass fills in the detailed screen specifications, component inventory, and protocol implications after Phase 1 experience is available. The second pass must be complete before Phase 2 specification begins.

---

## 6.1 Client Architecture

### Technology Stack

Both `xgen-node.exe` and `xgen-client.exe` are built using **Tauri** as the desktop application framework with **Svelte** as the frontend framework.

**Tauri** wraps a Rust backend — the XGen protocol logic, cryptography, networking, and file storage — with a web-based frontend rendered in the operating system's native webview (WebView2 on Windows, WebKit on macOS and Linux). The result is a single self-contained executable, Pattern A compliant. No Electron, no Node.js runtime, no separate web server.

**Svelte** was chosen as the frontend framework for three reasons. First, it is the least JavaScript-heavy of modern frontend frameworks — components are written as HTML files with a minimal `<script>` block and a `<style>` block, a structure immediately familiar to developers with strong HTML/CSS backgrounds. Second, CSS works exactly as standard, including CSS custom properties (variables) for theming. Third, Svelte has no complex framework concepts to learn — no virtual DOM, no Redux state management, no React hooks. The learning curve is shallow relative to alternatives.

**JavaScript scope is deliberately minimal.** The XGen UI is not a JavaScript application with a thin HTML layer — it is an HTML/CSS interface with a thin JavaScript layer. All protocol logic, cryptography, state management, and data processing lives in Rust. The frontend calls Rust functions via Tauri's `invoke()` API and reacts to events pushed from Rust. A typical frontend interaction:

```javascript
// Call Rust backend — one line
const event = await invoke('send_message', { roomId, text });

// Reactive variable — Svelte syntax, not raw JS
let messages = [];
$: sortedMessages = messages.sort((a, b) => a.timestamp - b.timestamp);
```

**Library-first backend structure — mandatory.**

The Rust backend in both `xgen-node/` and `xgen-client/` is structured as a library with a thin CLI shell on top from Phase 1 day one. This is not a Phase 2 concern — it is a Phase 1 implementation requirement documented in `IMPLEMENTATION_GUIDE_ph1.md`.

```
Phase 1:  main.rs (thin CLI shell)  →  lib.rs (all protocol logic)
Phase 2:  Tauri entry point         →  lib.rs (unchanged)
```

In Phase 2, the CLI shell (`main.rs`) is replaced by the Tauri entry point. `lib.rs` and all protocol logic are untouched. Every function the CLI called in Phase 1 is available to the Svelte frontend via `invoke()` in Phase 2 — no refactoring required. This is the architectural decision that makes Phase 2 UI integration seamless.

```
XGenProtocol/
  xgen-node/              ← Rust backend (Node binary)
  xgen-client/            ← Rust backend (Client binary)
  xgen-ui-shared/         ← shared design system + Svelte components
    base.css              ← always-loaded app normalize (D-057, D-058)
    tokens.css            ← CSS variable values (color, spacing, type)
    skin-dark.css         ← default visual skin; self-contained (D-057)
    components/           ← independently-editable Svelte components
  xgen-node-ui/           ← Svelte frontend for Node admin UI
  xgen-client-ui/         ← Svelte frontend for Client UI
```

The Tauri build process bundles the Svelte frontend into the Rust binary at compile time. The frontend assets are embedded in the executable — no separate asset folder, no web server. The executable extracts and serves the frontend from memory when the application window opens.

### Deployment

Pattern A applies without exception. Each executable creates and manages its own folder. The Tauri webview state (window size, position) is stored in the application folder alongside protocol data. No AppData, no registry, no system-level integration.

```
C:\XGenClient\
  xgen-client.exe          ← binary with embedded frontend
  client_config.json
  known_nodes.json
  webview_state.json      ← window geometry, persisted by Tauri
  logs\
    xgenclient.log
```

**Keypair exception — key files are NOT required to be in the application folder.**

Both the Node private key and the client Identity private key may be stored anywhere the operator or user chooses. Cloud storage (Google Drive, OneDrive) is explicitly supported — the key file is always encrypted at rest, making cloud storage safe without the decryption passphrase. The path is declared via `keypair_path` in the respective config file. This is a permanent architectural principle, not a Phase 1 limitation.

**Full Pattern A exception taxonomy**

Two categories of exception to the folder-is-the-application rule exist. Both are defined before implementation so they are never discovered as surprises during coding.

*Structural exceptions — physically cannot live in the application folder:*

| Exception | Reason |
|---|---|
| Cryptographic key files | Operator may store in secure cloud, network share, or HSM — `keypair_path` config field |
| Hardware Security Module (HSM) | Physical device — key never touches the filesystem |
| OS keystore (Windows Credential Manager, macOS Keychain) | Managed by OS — Phase 2, platform-specific |
| Tauri webview internal cache | WebView2/WebKit manages its own storage — partially configurable via Tauri API |

*Operational exceptions — can live in the application folder but operators may route elsewhere:*

| Exception | Reason |
|---|---|
| TLS certificates | System-managed by certbot, nginx, or OS certificate store |
| Log output | System log aggregation (syslog, Windows Event Log, Datadog) — app folder logging remains default |
| Shared Identity registry | HA deployments with primary/standby Nodes sharing one registry |

### Cross-Platform

The same Tauri + Svelte codebase produces executables for Windows, macOS, and Linux with minimal platform-specific work. Phase 1 targets Windows. Phase 2 adds macOS and Linux. The Pattern A folder structure applies identically on all three platforms — only the executable extension differs.

---

## 6.2 Shared Design System

### Principle

One design system, two applications. `xgen-ui-shared/` is the single source of truth for all visual tokens and reusable components. Both `xgen-node-ui/` and `xgen-client-ui/` import from it. A change to a CSS variable in `xgen-ui-shared/` propagates to both applications immediately.

### Design Token System

All visual properties are expressed as CSS custom properties (variables). No hardcoded colors, sizes, or font names anywhere in component code. Every visual decision is a token.

Token categories:

```css
/* xgen-ui-shared/tokens.css */

/* Color — base palette */
--xgen-color-primary:        /* main brand color */
--xgen-color-primary-hover:  /* hover state */
--xgen-color-surface:        /* background surfaces */
--xgen-color-surface-raised: /* elevated cards, panels */
--xgen-color-border:         /* borders and dividers */
--xgen-color-text:           /* primary text */
--xgen-color-text-muted:     /* secondary text */
--xgen-color-text-inverse:   /* text on dark backgrounds */
--xgen-color-error:          /* error states */
--xgen-color-warning:        /* warning states */
--xgen-color-success:        /* success states */

/* Typography */
--xgen-font-family:          /* primary typeface */
--xgen-font-family-mono:     /* monospace for IDs, code */
--xgen-font-size-xs:         /* ~10px — timestamps, below-caption labels (D-058) */
--xgen-font-size-sm:         /* 11px — captions, secondary labels */
--xgen-font-size-base:       /* 13px — root body size, set on html element (D-058) */
--xgen-font-size-lg:         /* 15px — prominent labels, section headings */
--xgen-font-size-xl:         /* 18px — large display text only */
--xgen-font-weight-normal:
--xgen-font-weight-medium:
--xgen-font-weight-bold:
--xgen-line-height-tight:
--xgen-line-height-base:
--xgen-line-height-relaxed:

/* Spacing scale — 4px base unit */
--xgen-space-1:   4px
--xgen-space-2:   8px
--xgen-space-3:   12px
--xgen-space-4:   16px
--xgen-space-6:   24px
--xgen-space-8:   32px
--xgen-space-12:  48px
--xgen-space-16:  64px

/* Border */
--xgen-radius-sm:   4px
--xgen-radius-md:   8px
--xgen-radius-lg:   16px
--xgen-radius-full: 9999px

/* Shadow */
--xgen-shadow-sm:
--xgen-shadow-md:
--xgen-shadow-lg:

/* Motion */
--xgen-transition-fast:   100ms ease
--xgen-transition-base:   200ms ease
--xgen-transition-slow:   350ms ease
```

The actual token values (colors, typeface choices) are defined in Chapter 6 second pass — after Phase 1 implementation and visual iteration. The token names and categories are locked now.

**Font tokens — reference implementation:**

```css
--xgen-font-family:       "XGen UI Sans";   /* proportional — reference implementation: Inter (SIL OFL 1.1) */
--xgen-font-family-mono:  "XGen UI Mono";   /* monospace    — reference implementation: JetBrains Mono (SIL OFL 1.1) */
```

The token names are functional descriptions — `XGen UI Sans` and `XGen UI Mono` — not locked to specific typefaces. The reference implementation bundles Inter and JetBrains Mono as the defaults. Both are licensed under SIL Open Font License 1.1, fully compatible with XGen's BSL 1.1 / GPL licensing model, and may be bundled inside the Tauri binary without runtime internet dependency. Operators and module authors may substitute any typeface that satisfies the slot (proportional sans-serif / monospace). Final typeface choices are confirmed during UI testing — the token names in this document are permanent, the values are not.

**Color token values** and all other specific token values are deferred to UI testing. The token categories are locked; the values are filled in once the application is running and visual decisions can be made in context.

### Theming

Two levels of theming exist:

**Application theme** — the default visual appearance of XGen. Dark and light variants. Operator-configurable at Node level. Applied globally.

**Space theme** — a Space owner may declare a theme for their Space via a `state.space_theme` Event. The client reads the theme from the Space's state and applies it as CSS variable overrides for that Space's context. The Space theme overrides application theme tokens within the Space view only.

```json
{
  "type": "state.space_theme",
  "content": {
    "color_primary": "#4f6ef7",
    "color_surface": "#1a1d2e",
    "color_text": "#e8eaf6"
  }
}
```

Only a defined subset of tokens may be overridden by a Space theme — the ones that affect brand identity without affecting readability or accessibility. The permitted override list is specified in Chapter 6 second pass.

### Shared Component Inventory

*Full component specifications in Chapter 6 second pass. Preliminary list:*

- `Button` — primary, secondary, ghost, danger variants
- `Input` — text, password, search
- `Avatar` — Identity avatar, Space avatar, Room avatar
- `MessageBubble` — text, image reference, file reference, reaction, redacted
- `MemberListItem` — Identity display with role badge
- `RoomListItem` — Room name, last message preview, unread count
- `SpaceListItem` — Space name, member count, Tier badge
- `ErrorDisplay` — error code + string + description (spec 3.3.8 display format)
- `Modal` — confirmation dialogs, forms
- `TierBadge` — visual indicator of Space Auth Tier (1–4)
- `NodeStatusIndicator` — connection state, federation health

**Component independence principle:** each Svelte component in `xgen-ui-shared/components/` is self-contained. Components consume tokens from `tokens.css` via CSS custom properties and call Rust via `invoke()` — they do not import from each other. A developer editing `MessageBubble.svelte` has no dependency on `MemberListItem.svelte` and no risk of cascading breakage. This independence is the property that makes module UI development predictable and makes the component library extensible without central coordination.

The component boundary maps to a named slot in the XGen UI shell. The slot inventory (§6.8.3) is the canonical reference for which components are independently injectable.

---

### CSS Layer Architecture

The `xgen-ui-shared/` folder implements a four-layer CSS architecture. Each layer has one job. The layers load in order; each can override the previous. Decision record: D-057, D-058.

```
Layer 1 — base.css        (always loaded, skin-independent)
Layer 2 — tokens.css      (CSS variable values — colors, spacing, type)
Layer 3 — skin-dark.css   (visual identity — bundles own token overrides)
Layer 4 — components/     (Svelte component <style> blocks)
```

**Layer 1 — `base.css` (always loaded)**

`base.css` is the application-level foundation. It loads unconditionally, before any skin. It has one job: establish the structural baseline that gives the application a coherent compact visual character even if no skin is present.

What `base.css` covers — exactly and no more:

- Universal box-model reset (`*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }`)
- Root type scale: `html { font-size: 13px; line-height: 1.35; }` (D-058)
- Minimal resets for browser-aggressive elements only: `button`, `input`, `a` — strip default browser appearance so components can restyle them from scratch
- `body { -webkit-font-smoothing: antialiased; }` — subpixel rendering

`base.css` does not contain colors, font names, spacing values, or any visual opinion. It is approximately 50 lines. It is stable — it changes rarely, if ever.

**Why not a standard browser normalize?** A browser normalize like Normalize.css is written for the full HTML element set — paragraphs, blockquotes, tables, form elements in all their varieties. The XGen UI is a Svelte component application. It does not use most of those elements. Importing a full normalize introduces hundreds of lines of rules that apply to elements that do not exist in the application, increasing CSS surface area and specificity complexity for no benefit. `base.css` resets only the elements the application actually uses. (D-057)

**Degradation behaviour:** if a skin fails to load, the application renders with `base.css` only — a compact, structured interface with clean box model and readable proportions, but no colors or visual identity. This is a legible degraded state, not raw unstyled HTML. (D-057 — corrects D-041's "reset coupled to skin" statement)

**Layer 2 — `tokens.css`**

CSS custom property definitions. Two categories:

*Spacing scale* (D-058) — named steps derived from 4px root unit:
```css
--xgen-space-1:  4px;
--xgen-space-2:  8px;
--xgen-space-3:  12px;
--xgen-space-4:  16px;
--xgen-space-6:  24px;
--xgen-space-8:  32px;
--xgen-space-12: 48px;
--xgen-space-16: 64px;
```

The 4px base unit is the standard used by Discord, Slack, and VS Code. It enables systematic accessibility rescaling — multiply all spacing tokens by 1.5 and the layout scales coherently.

*Typography tokens* (D-058):
```css
--xgen-font-size-xs:   0.77rem;   /* ~10px — timestamps, below-caption */
--xgen-font-size-sm:   0.85rem;   /* 11px — captions, secondary labels */
--xgen-font-size-base: 1rem;      /* 13px — body, set by base.css html rule */
--xgen-font-size-lg:   1.15rem;   /* 15px — prominent labels */
--xgen-font-size-xl:   1.38rem;   /* 18px — large display text */
--xgen-line-height-tight:   1.2;
--xgen-line-height-base:    1.35;
--xgen-line-height-relaxed: 1.55;
```

Typography is component-scoped. Component `<style>` blocks set font-size and line-height for their own elements using these tokens. No global font-size cascade rules beyond the `html` root set in `base.css`. (D-058)

Color token names are defined in `tokens.css` as empty custom properties; their values are set by the active skin. This means `tokens.css` is skin-independent — it defines the vocabulary, not the values.

**Layer 3 — `skin-dark.css` (default skin)**

The skin file does two things: it provides all color token values, and it declares its own font family. The skin is self-contained — it can be replaced wholesale by substituting a different `.css` file. No skin-specific logic lives outside the skin file.

The default skin is `skin-dark.css`. The fallback chain on skin failure: requested skin → `skin-dark.css` → base.css only (legible degraded state). (D-041, D-057)

**Layer 4 — `components/`**

Each `.svelte` file in `components/` carries its own `<style>` block. Component styles consume only tokens from layers 1–3 via `var(--xgen-*)` references. No hardcoded pixel values, no hardcoded color values, no cross-component style imports anywhere. (D-058)

---

## 6.3 Theming Model

*Preliminary — full specification in Chapter 6 second pass.*

**Note on terminology:** §6.2 describes a four-layer *CSS architecture* (base → tokens → skin → components). Section 6.3 describes a separate, application-level *theming cascade* — three levels at which CSS token values can be overridden at runtime. These are different concepts. The CSS architecture is a build-time file structure; the theming cascade is a runtime layering of token value overrides. The CSS architecture enables the theming cascade, but they should not be conflated.

Three-layer theming cascade, each layer overriding the previous:

```
Layer 1 — XGen default theme      (built into the application)
    ↓ overridden by
Layer 2 — Application theme       (dark/light, operator-configured at Node level)
    ↓ overridden by
Layer 3 — Space theme             (declared by Space owner in state.space_theme Event)
```

The client applies Layer 3 overrides only within the active Space context. Switching Spaces switches the active theme. The Room view inherits the Space theme; the global Space list uses the application theme.

**Open questions for second pass:**
- Which specific CSS tokens may a Space owner override?
- Can a user disable Space themes entirely (accessibility preference)?
- Does the Node admin UI support Space theme previewing?

---

## 6.4 Node Admin UI

*Preliminary screen inventory — detailed specifications in Chapter 6 second pass.*

The Node admin UI is the operator-facing interface for managing a running XGen Node. It opens as a desktop window when `xgen-node.exe` is launched. It is not a web interface served on a port — it is the Tauri application window itself, accessible only on the machine running the Node.

**Screens:**

**Dashboard** — Node status at a glance. Node ID (truncated pubkey_uri), uptime, connected clients count, federated Node count, recent error log, announcement validity status.

**Identity Registry** — list of registered Identities. Search by display name or identity_id. View individual Identity records. Trust Assertion status and expiry. Action: revoke registration.

**Federation** — list of federated Nodes with connection status, session ID, shared Spaces, negotiated capabilities. Action: disconnect, view handshake log.

**Spaces** — list of hosted Spaces with member count, Room count, Auth Tier, federation status. Action: view Space state, view Room Event log.

**Auth Modules** — list of trusted Auth Modules with validity status. Action: add new Auth Module (requires Auth Module public record), remove.

**Log Viewer** — filterable operational log. Filter by error code range (1xxx, 2xxx, 3xxx), by severity, by timestamp. Error display follows spec 3.3.8 format.

**Configuration** — view and edit `node_config.json` fields with validation. Restart required indicator for changes that need a restart.

---

## 6.5 Client UI

*Preliminary screen inventory — detailed specifications in Chapter 6 second pass.*

The Client UI is the user-facing interface for participating in XGen Spaces and Rooms.

**Screens:**

**Identity Setup** (first run) — generate keypair, choose display name, select home Node, complete Auth Module verification flow, register Identity.

**Space List** — all Spaces the user is a member of. Space avatar, name, Tier badge, unread message count. Action: join Space, create Space, create DM.

**Room View** — the primary messaging interface. Message history in chronological order, message input, member list sidebar (collapsible), Room name and topic in header. Message types: text, image reference, file reference, reaction, redacted placeholder.

**Member List** — all members of the current Space or Room. Display name, Identity ID (truncated), role badge, online/offline indicator. Action (if permitted by role): invite, kick, ban.

**DM View** — identical to Room View but without the Space context chrome. The two-participant header replaces the Room name.

**Identity Profile** — view own Identity: display name, identity_id, trust assertion status and expiry, connected devices (Phase 2), home Node.

**Node Selection** — choose which Node to connect to. Known Nodes list with connection status. Action: add new Node endpoint.

---

## 6.6 Auth Module UI

*Preliminary — detailed specification in Chapter 6 second pass.*

The verification flow is embedded in the Client UI as part of the Identity Setup screen and the Trust Assertion renewal flow.

**Verification flow screens:**
1. Select verification method (email, phone, or both — depending on Node's required Tier 1 state)
2. Enter contact detail (email address and/or phone number)
3. Enter verification code(s) received by email/SMS
4. Confirmation — assertion received, display validity period

The Auth Module UI is not a separate window — it is a modal dialog sequence within the Client application. The client communicates with the Auth Module over HTTPS; the Rust backend handles the network call and passes results to the Svelte frontend.

---

## 6.7 Protocol Implications

*To be written in Chapter 6 second pass, after Phase 1 implementation.*

This section will document what UI requirements feed back into Phase 2 protocol specification. Preliminary items identified:

**Message header structure — decided:**

The base message header contains four elements in this order:

```
[Avatar]  [Display name]  [Timestamp]  [Reply link — if this message is a reply]
```

The message body follows immediately below. No decoration, no role badges, no status indicators in the base header. Clean and minimal.

The message header is an **extension slot** (`room.message.decorator`). Modules may inject additional elements into the header area — custom tags, role badges, status indicators, or other decorators — without modifying the core message rendering. Base XGen ships with nothing in this slot. The slot is populated entirely by modules.

This is the same injection slot mechanism defined in 6.8.3. The message decorator slot is the message-level equivalent of the sidebar and toolbar slots.

Final visual layout and spacing of the header is tuned during UI implementation.

**New EventTypes likely needed:**
- `state.space_theme` — Space theme declaration (referenced in 6.3 above)
- `message.edit` — message editing (see below)
- `message.delete` — message deletion / redaction (see below)

**Message editing model — decided:**

Editing a message replaces the displayed content in place — the message appears at its original position in the timeline showing the latest version, with a small "edited" marker. The full edit history is accessible via the marker (click to view all previous versions). The original message is never hidden from the history view.

Protocol mechanism: the original `message.text` Event is immutable in the DAG and never modified. An edit produces a new `message.edit` Event referencing the original via `original_event_id`. The UI renders the latest `message.edit` version at the original message position. If multiple edits exist, the most recent one wins.

**Message deletion model — decided:**

Deleting a message replaces it with a placeholder in the timeline:

```
[This message was deleted by Alice / by Admin]
```

The placeholder preserves the message's position in the timeline so the reply chain below it remains coherent. All replies to a deleted message stay visible and untouched — the DAG is append-only and reply branches cannot be removed.

Protocol mechanism: deletion produces a `message.delete` Event referencing the original via `original_event_id`. The UI renders the placeholder at the original position. The original Event content remains in the DAG and is accessible to Node operators — deletion is a UI-level redaction, not a protocol-level erasure.

**Permissions:**

| Action | Who can perform it |
|---|---|
| Edit message | Sender only |
| Delete message | Sender, Space admin, Space owner |

**Space-level policy (configurable):**

A Space owner may restrict editing and deletion via Space state configuration:

- `allow_message_edit: true/false` — whether members can edit their own messages
- `edit_window_seconds: null / integer` — `null` means no time limit; an integer restricts editing to within N seconds of the original send
- `allow_message_delete: true/false` — whether senders can delete their own messages (admin/owner delete is always permitted regardless of this setting)

These policies are declared in `state.space_create` and may be updated by the Space owner via a state Event. They apply to all Rooms in the Space.

At Tier 3+ compliance Spaces, `allow_message_edit` and `allow_message_delete` would typically be set to `false` to maintain an immutable audit record.

**User experience summary (same as Discord/Slack for the user):**
- Edited messages appear in the same position showing the latest version with an "edited" marker
- Deleted messages show a placeholder preserving timeline position
- Reply chains below edited or deleted messages are unaffected
- Edit history visible on click

**Threading model — decided:**

XGen uses a **flat timeline with header links** threading model. Replies are not indented or nested — the Room timeline is always a clean flat list regardless of conversation depth. A reply message carries a clickable reference to its logical parent in the message header. The parent preview (sender name + message excerpt) appears above the reply, not as visual indentation but as a navigable link that jumps to the parent message in the timeline.

This model was chosen specifically to avoid Discord’s problem of threads escaping the Room and becoming sub-channels in the navigation sidebar. In XGen, threads are a UI rendering of the existing DAG structure — no new navigation entries, no sidebar pollution, no separate thread concept.

**Protocol implication:** no new EventType required. A reply is a `message.text` (or other message type) Event whose `prev_events` includes the parent message’s event_id. The UI reads this relationship from the DAG and renders the header link. Optionally, the `content` field may carry a `reply_to_event_id` field as a rendering hint to avoid DAG traversal on display — this is an implementation optimisation, not a protocol requirement.

Final visual treatment of the header link (exact layout, preview length, interaction) is tuned during UI implementation.

**New state fields likely needed:**
- Space: `theme` object in Space state
- Room: `notification_level` per-user preference (may be client-side only)

**Phase 2 spec sections directly informed by Chapter 6:**
- 3.9 State Resolution — how conflicting `state.space_theme` Events are resolved
- 3.10 E2E Encryption — encrypted Room visual indicator in Room View
- 3.13 Identity Replication — multi-device sync visible in Identity Profile screen

---

---

## 6.8 Module Architecture (resolves OQ-01)

Decision record: D-036.

XGen modules extend the Node, the Client, or both. They are first-class participants in the protocol — they connect to the Event stream, speak native XGen, and are managed through a unified module list. There is no separate plugin API, no separate IPC protocol, no special build system.

---

### 6.8.1 Communication Model

Modules communicate via **Event subscription + `meta_atts`**.

A module connects to the Node or Client via a standard WebSocket connection and subscribes to the Event stream. It receives Events as they flow through the system and may produce Events in response. Module-specific payload travels in the `meta_atts` field of Events — the existing extension channel on every XGen Event (spec 3.2.1).

**`meta_atts` conventions for modules:**

```json
"meta_atts": {
  "xgen.module.compliance-reporter.retention_class": "7year",
  "xgen.module.summariser.summary_ready": "true"
}
```

- Keys are namespaced: `xgen.module.<module_id>.<key>`
- Values are strings or JSON-serialisable objects
- Core Nodes and Clients that do not recognise a `meta_atts` key silently ignore it (open enum principle, 3.4.3)
- `meta_atts` is strictly an extension channel — core protocol data never travels through it

This model means modules speak the same language as every other participant in the network. No new protocol is required. A module written in any language that can open a WebSocket connection can participate.

---

### 6.8.2 Module Package and Manifest

A module is distributed as a **package** — one folder containing a manifest file plus any number of handlers, assets, and UI components.

```
my-module/
  xgen-module.json    ← manifest (required)
  main.py             ← entry point (any language)
  ui/                 ← UI assets (if any)
    widget.html
    window.html
  README.md
```

**Manifest schema — `xgen-module.json`:**

```json
{
  "id": "compliance-reporter",
  "version": "1.0.0",
  "name": "Compliance Reporter",
  "description": "Produces SOX-compliant audit reports from Space Event history.",
  "author": "Example Org",
  "author_url": "https://example.org",
  "xgen_protocol_min": "0.1",
  "identity_mode": "system",
  "ui_forms": ["window"],
  "capabilities": ["xgen.module.compliance-reporter"],
  "event_subscriptions": [
    "membership.join",
    "membership.leave",
    "membership.ban",
    "state.space_create"
  ],
  "entry_point": "main.py",
  "settings_schema": {
    "retention_years": { "type": "integer", "default": 7 },
    "report_format": { "type": "string", "enum": ["pdf", "csv"], "default": "pdf" }
  }
}
```

| Field | Required | Description |
|---|---|---|
| `id` | yes | Unique module identifier — reverse-domain style recommended |
| `version` | yes | Semantic version |
| `name` | yes | Human-readable title shown in module list |
| `description` | yes | Shown in module list entry |
| `author` | yes | Author name |
| `author_url` | no | Author or project URL |
| `xgen_protocol_min` | yes | Minimum XGen protocol version required |
| `identity_mode` | yes | `system` or `user` — see 6.8.4 |
| `ui_forms` | yes | Array: any combination of `headless`, `widget`, `window`. Must include at least one. |
| `capabilities` | no | Capability strings to add to the Node/Client announcement when this module is active |
| `event_subscriptions` | no | EventTypes this module subscribes to. Empty = subscribes to nothing (headless utility) |
| `entry_point` | yes | The file to execute when the module starts |
| `settings_schema` | no | JSON Schema fragment defining configurable settings. Rendered automatically in the module list settings panel. |

**Package installation:** the user places the module folder in the `modules/` subdirectory of the Node or Client working directory. The Node/Client scans `modules/` on startup and loads all valid manifests. Hot-loading (without restart) is a Phase 3 consideration.

```
<working_dir>/
  modules/
    compliance-reporter/
      xgen-module.json
      main.py
    summariser/
      xgen-module.json
      main.rs
```

---

### 6.8.3 Module UI Forms

Three UI forms, declared in the manifest. A single module package may declare more than one.

**Headless**

No UI beyond the module list entry. Runs as a background process. No window, no widget. Used for compliance reporters, bridges, aggregators, notification forwarders.

The module list entry remains the management interface: enable/disable, view status, access settings.

**Widget**

A UI component injected into a named slot in the XGen application shell. The widget is an HTML file rendered in an isolated webview embedded in the shell. The widget communicates with its module backend via a local WebSocket.

Widget modules declare their target slot in the manifest:
```json
"widget_slot": "room.sidebar.bottom"
```

**Defined injection slots (preliminary — full inventory in second pass):**

| Slot name | Location |
|---|---|
| `room.sidebar.top` | Top of the member list sidebar in Room view |
| `room.sidebar.bottom` | Bottom of the member list sidebar in Room view |
| `room.toolbar` | Message input toolbar (additional action buttons) |
| `room.message.decorator` | Inline decorator attached to individual messages |
| `space.header` | Below the Space name in the Space header |
| `node.dashboard.widget` | A panel on the Node admin dashboard |
| `global.statusbar` | A small indicator in the application status bar |

The XGen shell renders widget slots as named placeholder elements. A widget module fills its declared slot. If no module occupies a slot, the slot is invisible. Multiple modules targeting the same slot stack vertically.

**Window**

A full separate desktop window launched from the module list entry. The window is a Tauri webview containing the module's `window.html`. It has its own independent lifecycle — it can be opened, minimised, and closed without affecting the main application window.

Used for the Auth Module verification flow, compliance dashboards, administrative tools, and any module whose UI is too substantial to be a widget.

The module list entry carries a **Launch** button for window modules. Clicking it opens (or focuses if already open) the module window.

---

### 6.8.4 Module Identity Modes

Declared in the manifest as `identity_mode`:

**`system` mode**
- The module has its own keypair, generated at installation
- Its identity_id is derived from its keypair pubkey — self-certifying, same as any XGen Identity
- It signs Events as itself — its identity_id appears in the `sender` field
- It may register on the Node as a distinct Identity
- Other participants see it as a separate actor (e.g. a bot that posts summaries)
- No user consent required beyond installing the module

**`user` mode**
- The module acts on behalf of the authenticated user
- It produces Events signed by the user's private key
- The `sender` field of Events it produces carries the user's identity_id
- **Explicit user consent is required at install time** — the install dialog displays: "This module will produce Events signed as you. Do you consent?"
- The user may revoke consent at any time from the module list, which stops the module immediately
- A `user`-mode module that attempts to sign as a different Identity than the authenticated user is rejected by the Node

---

### 6.8.5 Module List — Universal Registry

Every installed module appears in the module list, regardless of its UI form or identity mode. The module list is the single place a user discovers, enables, disables, configures, and removes modules.

**Module list entry — visual structure (stacked block):**

```
┌────────────────────────────────────────────────────────────────┐
│ ○  Compliance Reporter          v1.0.0  [system]  ● Running  │
│     Produces SOX-compliant audit reports from Space Event    │
│     history. Author: Example Org                            │
│                                                              │
│     [Settings]  [Launch]  [Disable]  [Remove]               │
└────────────────────────────────────────────────────────────────┘
```

Every entry contains:

| Element | Description |
|---|---|
| Status indicator | ● Running (green) / ○ Stopped (grey) / ⚠ Error (amber) |
| Name | From manifest `name` |
| Version | From manifest `version` |
| Mode badge | `[system]` or `[user]` — always visible |
| Description | From manifest `description` |
| Author | From manifest `author` |
| Settings button | Opens settings panel rendered from manifest `settings_schema`. Present for all modules, greyed out if no settings defined. |
| Launch button | Opens module window. Only present if `ui_forms` includes `window`. |
| Disable/Enable toggle | Stops or starts the module without removing it |
| Remove button | Uninstalls the module — requires confirmation for `user`-mode modules |

Modules are listed in alphabetical order by name. The user may not reorder them manually — the list is not a priority indicator.

---

### 6.8.6 Capability Advertisement

When a module with declared `capabilities` is active, the Node or Client adds those capability strings to its `capabilities` array in its node announcement (3.5.2). When the module is disabled or removed, the capabilities are removed from the next announcement.

This means the network learns about module capabilities automatically through the existing announcement mechanism. No separate module discovery protocol is needed.

```json
// Node announcement with a module active
"capabilities": [
  "json",
  "msgpack",
  "xgen.federation",
  "xgen.module.compliance-reporter"   ← added by active module
]
```

---

### 6.8.7 Auth Module as a Module

The XGen Auth Module (spec 3.8) is the reference implementation of a Window-form module. It demonstrates all three aspects of the module architecture:

- **Communication:** subscribes to `identity.register` Events, responds via `auth.verify_request` / `trust_assertion` (existing spec 3.8.2 interface)
- **Identity mode:** `system` — the Auth Module has its own keypair, signs Trust Assertions as itself
- **UI form:** `window` — the verification flow (email/phone entry, code confirmation) runs in its own window, launched from the module list
- **Capability:** `xgen.auth.tier1` (and higher tiers for Tier 2+ Auth Modules)

The Auth Module shipping with XGen as a built-in is not special — it uses the same manifest format and the same module list entry as any third-party module. The only difference is that it is bundled with the distribution. A third-party institution may replace it with their own Auth Module by installing a different module package.

---

### 6.8.8 Open Questions for Phase 2 Implementation

- **Hot-loading:** Can modules be installed and activated without restarting the Node/Client? Phase 3 consideration.
- **Module signing:** Should module packages be cryptographically signed by their authors? Required for institutional deployments. Phase 2 design question.
- **Module permissions:** Beyond `identity_mode`, should modules declare what Node data they can access (identity registry, space state, federation registry)? Phase 2 design question.
- **Widget sandboxing:** Widget webviews must be isolated from each other and from the main application. What CSP and iframe sandboxing apply? Phase 2 implementation question.
- **Module-to-module communication:** Can two modules communicate directly, or only via Events? Phase 2 design question.

---

## 6.9 Console Input Channel Protocol

*Status: open question for Phase 2 design — not yet specified*

The Console accepts commands from three sources: keyboard (interactive), batch file (`--batch` flag), and IPC (programmatic, from an AI agent or external process). All three use the same underlying command channel. The log stream flows back to whoever is reading.

This is documented as a formal open question because the IPC interface and the batch file format require deliberate design before Phase 2 implementation begins.

**Three operation modes to specify:**

**Mode 1 — Batch file**
The `.exe` accepts a `--batch <file>` flag. The file contains one command per line (suggested extension: `.xgb` — XGen Batch). The client or node executes each command sequentially and exits on completion or error. No UI window required. Used by Claude Code for test automation and deployment sequences.

**Mode 2 — AI-assisted interactive operation**
A human is present, watching the Console overlay. An AI agent injects commands via IPC, reads the log stream back, and makes decisions. The human can intervene at any point — take back the keyboard, override, redirect. The Console is the shared surface where both human and agent are visible and neither is hidden.

**Mode 3 — Checkpoint-driven admin processes**
Long-running, complex, multi-step administrative workflows driven by an AI agent — Space migration, bulk identity management, federation setup, compliance audit generation. Too complex for manual operation, too sensitive for full automation. The process pauses at defined decision points and requires explicit human confirmation before proceeding. The agent drives; the human approves at checkpoints.

**Questions to resolve in Phase 2 design:**
- IPC mechanism — named pipe, local socket, or stdin/stdout redirect?
- Batch file format — plain text one-command-per-line, or structured (comments, variables, conditionals)?
- Checkpoint handshake protocol — how does the agent signal a pause? How does the human signal confirm / abort / redirect?
- Log stream format for programmatic consumption — structured JSON lines alongside the human-readable stream?
- Authentication of the IPC channel — should a connecting agent authenticate, or is local process ownership sufficient?

**Philosophical grounding:** this is documented in Ch1 — Human and Agent Operation. The Console IPC model is the same architectural principle as algorithm agility and open enums: the interface is stable, what uses it can evolve. A batch file written today works in 2040. An AI agent in 2026 uses the same interface as a human operator.

---

## 6.10 Message Compose — Text Substitution

*Status: noted for second pass — low priority, no implementation dependency*

The message compose area in the Client UI SHALL support a configurable text substitution list. Character sequences entered by the user are substituted in real time as they type, before the message is sent.

**Purpose:** improve typing ergonomics for common typographic and symbolic inputs without requiring special key combinations or character pickers.

**Examples:**

| Typed sequence | Substituted with |
|---|---|
| `->` | `→` |
| `<-` | `←` |
| `=>` | `⇒` |
| `...` | `…` |
| `:)` | `🙂` |
| `:(` | `🙁` |
| `--` | `—` |

**Design rules:**
- Substitution happens on-the-fly as the user types, triggered by a trailing space or punctuation
- The substitution list is user-configurable — the defaults above are a suggested starting set
- Any substitution can be undone immediately with a single Backspace after it fires
- The substitution list is stored in client config, not in any protocol Event — it is a local UI preference only
- The substitution list has no protocol implications

**Implementation note:** this is a client-side input processing concern only. It is implemented in the Svelte compose component and has no Rust backend dependency.

---

## 6.11 Console

The Console is a first-class surface in both `xgen-client.exe` and `xgen-node.exe`. It is not a debug add-on — it is the canonical command surface and, for the Client, the lifecycle host that the stateless CLI invocation model does not provide.

**Full name:** XGen Client Console / XGen Node Console.

---

### 6.11.1 Purpose and role

**Client side:** `xgen-client.exe` has no persistent process between CLI invocations. Each call is stateless — logs fragment, there is no continuity to debug against. The Console window solves this by being the lifecycle host. Opening the window starts a session; closing it ends it. All Events within that window's lifetime are grouped under one session, with one log, one session ID. Without the Console, meaningful Phase 2 client-side testing is not possible.

**Node side:** `xgen-node.exe` has a natural process lifecycle and does not need the Console as a lifecycle host. The Console on the Node side is an operator command surface — a first-class interface for issuing commands and observing the live log stream, equivalent in status to the admin dashboard.

**Both sides:** the Console provides a prompt-driven command interface that is not a replacement for the GUI but a complement to it. It is always available, always honest, and never hides infrastructure state.

---

### 6.11.2 Display model

The Console is an **in-app overlay**. It slides down from the top of the application window over the existing content. The application remains fully visible and active underneath — the user can observe Room messages arriving, Space state updating, and Node activity while typing commands.

This is intentional: the Console is not a modal that interrupts the application. It is a transparent layer that augments it.

**Toggle:** physical top-left key — `Backquote` scancode (`KeyboardEvent.code = "Backquote"`). Position-based, not character-based. Layout-independent across all keyboard locales. Pressing the key opens the Console if closed and closes it if open.

**Transparency:** semi-transparent background, value configurable by the user. Final default value deferred to UI testing.

**Undocking:** planned future capability — the overlay can be undocked to a separate OS window for extended work sessions. Not Phase 2.

---

### 6.11.3 Visual design

The Console has its own visual lane, separate from the `xgen-ui-shared/` skin cascade. It uses a terminal emulator aesthetic.

**Default color scheme:** green-on-dark (VT220 / classic terminal). This is locked as the default.

**User-selectable schemes:**
- `green-black` — VT220, default
- `amber-black` — IBM 3270 / Hercules monochrome
- `white-black` — VGA console
- `black-white` — paper terminal
- `xgen` — uses active `xgen-ui-shared` skin tokens for visual continuity

**Font:** JetBrains Mono (per Ch6 §6.2 reference implementation). System monospace available as fallback toggle.

**Font size:** user-configurable (12 / 14 / 16 / 18px).

The terminal aesthetic is deliberately separate from the application skin. The Console reads as a different kind of surface — one that is closer to the protocol than to the UI.

---

### 6.11.4 Structure

The Console has three zones, top to bottom:

**1 — Status bar**

A single line of persistent status information. Left/right division:

- Left: `XGen Client Console · ● STATE` — app name and current lifecycle state
- Right: `DisplayName / @SpaceNick [Tn] · Space › #Room · ~ close`

Where `[Tn]` is the **tier glyph** — a compact inline square at line height, color-coded by tier:

| Glyph | Tier | Color |
|---|---|---|
| `T1` | Basic verified identity | Green |
| `T2` | Institutional verified | Blue |
| `T3` | Corporate / compliance | Amber |
| `T4` | Government / high security | Red |

The tier glyph in the Console status bar reflects the current session's auth level on the connected Node. **This is the only correct placement of the tier glyph on the Client side.** The tier is a Node property — it describes what authentication level the Node requires and enforces for the current session. It is session-scoped, not identity-scoped. Displaying tier badges on individual messages or member list entries is architecturally incorrect and must not be done (D-038).

The tier glyph also appears in the Node admin dashboard and the Node status panel in the client sidebar — both correctly describing the Node's own tier requirement.

The breadcrumb (`Space › #Room`) reflects the active context when the Console was opened — the Space and Room the user was in. If opened from the Space list with no active Room, only the Space name appears.

**What is NOT in the status bar:** Node URL, session ID, identity fingerprint, DAG head, last error. All available via `state get` command.

**State indicator interaction:** shows one state at a time — the current active state. Click → dropdown showing the full state set (from Appendix E) with current state highlighted. The dropdown is a built-in reference — the operator never needs to look up what a state name means. When multiple degraded states are active, the highest-severity state is shown with a `+N` badge; the dropdown reveals all active states.

**Infrastructure transparency principle:** the lifecycle state in the status bar is not merely a widget. It is a statement about infrastructure ownership. A user connected to a Node in `MAINTENANCE` state sees that immediately — no mystery timeouts. A user seeing `DEGRADED_FEDERATION` knows local Space works but cross-Node delivery is impaired. XGen surfaces infrastructure state because users are participants in infrastructure they own, not tenants on a platform that hides its internals.

**2 — Log stream**

Single chronological stream. Dense, monospace. One line per entry: timestamp / level / subsystem tag / message. All CLI invocations within the session append to this stream — it is never split per-call into separate files.

Log levels rendered with distinct colors within the active scheme: `info`, `warn`, `error`, `cmd` (user command echo), `out` (command output), `hint` (suggested follow-up commands).

Filter rail and search are deferred — not rejected, not in first pass.

**3 — Prompt**

Bottom-anchored `xgen>` prompt line. Standard readline behaviour:
- Up/Down arrows: command history
- Tab: completion of known commands
- `?` or `help`: lists available commands (per existing CLI spec)
- Ctrl+L: clear view

No Ctrl-K command palette. The `Backquote` key is the single access point for the Console itself.

---

### 6.11.5 Session lifecycle (Client)

The Console window is the session host for the Client. Full lifecycle definition is in **Appendix E — Application Lifecycle States**.

Key rules:
- Opening the window = session starts, log begins, `SETUP` or `INITIALISING` state entered
- Closing the window = `CLOSING` state, log archived, session ends
- The session ID is window-bound — it does not persist across window open/close cycles
- `SETUP` (first run) is a formal top-level state, logged from window open — not a pre-lifecycle screen

---

### 6.11.6 Session lifecycle (Node)

The Node Console does not own the Node's lifecycle — the process does. The Console window observes and displays the Node's process-level states. Closing the Node Console window does not affect the running Node.

The Node Console's own session (its log stream) begins when the window opens and ends when it closes, but the Node process lifecycle is independent.

---

### 6.11.7 Relationship to other screens

The Console is accessible from any screen in both applications via the `Backquote` toggle. It is not a screen in the navigation hierarchy — it is a persistent overlay available everywhere.

It does not replace the admin dashboard (Node) or the main client UI (Client). It complements them with a command-driven, log-visible surface that is always honest about what the application is doing.

---

## 6.12 Temperature Property

Temperature is a numeric property attached to two kinds of subject in the client UI: a Room (the collective rhythm of a Room's recent traffic) and a Member-in-a-Room (an individual member's accumulated overpass of Space pacing rules). The Room's home Node computes both values and publishes them; the client displays them.

> **XGID typing (Retrofit Pass 4).** The client `TemperatureUpdate` payload carries `space_id` as `SpaceXgid` and `room_id` as `RoomXgid` (typed in memory, plain `String` on the wire via serde-transparency). `subject_id` **stays `String`** (D-061): it is a union of a member `IdentityXgid` **or** the non-XGID `SUBJECT_ROOM` sentinel (§6.12.3), so it cannot carry a single XGID flavour.

The mechanism behind the numbers is intentionally outside the protocol. Different communities will moderate at different rhythms — a meditation Space, a fast-chat Space, and a regulated compliance Space each have legitimate but incompatible definitions of "hot". The protocol carries the temperature value and the bucket thresholds; the home Node's plugin chooses how to compute the value. Cross-references: Ch1 §"Visible Self-Correcting Feedback", D-059 (AI as first-class member — informs why AI is muted rather than kicked), D-060 (space pacing rules — the input signal the temperature plugin observes), D-061 (the temperature mechanism decision itself).

---

### 6.12.1 What the client receives

Two numeric values per subject, both floats in the closed range `[0.0, 1.0]`, both delivered as `meta_atts` keys on protocol events from the home Node:

| Key | Subject | Meaning |
|---|---|---|
| `xgen.room_temperature` | Room | The Room's collective heat — visible to every member |
| `xgen.member_temperature` | Member-in-Room | One member's individual heat — visibility gated, see §6.12.5 |

Absence of a key means the home Node is not publishing temperature for that subject. The client renders nothing for that subject. A Room may publish room temperature without member temperature, or vice versa.

Both values are **opaque** at the client level — the client does not know how they were computed, does not attempt to re-derive them, and treats them as authoritative.

---

### 6.12.2 Threshold table

The home Node publishes a threshold table once at room-open time as part of the Room metadata response, separate from per-event `meta_atts`:

```json
"temperature_thresholds": {
  "warm":  0.30,
  "hot":   0.55,
  "fiery": 0.80
}
```

The `cool` state is implicit at the bottom of the range (any value below `warm`). A Room that does not publish a threshold table falls back to the Ch6 defaults:

| State | Default lower bound |
|---|---|
| `cool` | 0.00 |
| `warm` | 0.25 |
| `hot` | 0.50 |
| `fiery` | 0.75 |

The client stores the threshold table for the duration of the Room session and reuses it for every temperature update during that session. When the Node sends a new threshold table (e.g. the Space owner changed the temperature configuration), the client adopts the new values and re-derives the bucket for any currently-displayed temperature value.

---

### 6.12.3 Client-side derivation: float → state

On every temperature update, the client performs one comparison per subject to derive the state bucket:

```
if temperature >= fiery_threshold:   state = "fiery"
elif temperature >= hot_threshold:    state = "hot"
elif temperature >= warm_threshold:   state = "warm"
else:                                 state = "cool"
```

Derivation happens **once per update**, when the new `xgen.*_temperature` value arrives, not on every render frame. The derived state is written to the DOM as a data attribute alongside the float; both values then remain in place until the next update.

---

### 6.12.4 DOM contract

Temperature is exposed to CSS through two attributes on each subject's representative DOM element. Skin CSS reads these and renders accordingly.

**For a Room** — applied to the Room's representative element (Room header, Room list entry, or both, depending on skin choice):

```html
<div class="xgen-room-banner"
     data-temp-state="warm"
     style="--xgen-room-temperature: 0.42">
  ...
</div>
```

**For a Member-in-Room** — applied to the member's avatar element wherever it appears (member list, message attribution, hover card):

```html
<div class="xgen-avatar"
     data-temp-state="hot"
     style="--xgen-member-temperature: 0.61">
  ...
</div>
```

Skin CSS may target either or both:

```css
/* Smooth gradient driven by the float */
.xgen-room-banner {
  background: linear-gradient(
    to right,
    var(--xgen-color-surface),
    color-mix(in srgb, var(--xgen-color-warning) calc(var(--xgen-room-temperature, 0) * 100%), transparent)
  );
}

/* Discrete state styling */
.xgen-avatar[data-temp-state="warm"]  { box-shadow: 0 0 0 1px var(--xgen-color-warm); }
.xgen-avatar[data-temp-state="hot"]   { box-shadow: 0 0 0 2px var(--xgen-color-hot); }
.xgen-avatar[data-temp-state="fiery"] { box-shadow: 0 0 0 3px var(--xgen-color-fiery); animation: xgen-pulse 2s infinite; }
```

A skin that wants temperature indicators silent does nothing — the data attributes are present, no rule consumes them, no visual change. A skin that wants only the gradient ignores `data-temp-state`. A skin that wants categorical-only ignores the float and styles the data attribute. All three are valid.

---

### 6.12.5 Visibility policy

`xgen.room_temperature` is **visible to every member of the Room**. The Room's collective state is shared awareness; rendering it for the whole membership is the default and not configurable.

`xgen.member_temperature` is **moderator-visibility by default**. The home Node publishes the per-member value only to clients whose authenticated identity holds a moderator-or-higher role in the Space. Other members receive no `xgen.member_temperature` key for members other than themselves. A member always sees their own temperature regardless of role (the home Node always publishes the value to the subject's own client).

The visibility default is configurable per Space:

| Space setting | Effect |
|---|---|
| `member_temperature_visibility: "moderator"` | Default. Moderators and above see member temperatures. |
| `member_temperature_visibility: "everyone"` | All members see all member temperatures — transparent communities. |
| `member_temperature_visibility: "self_only"` | Even moderators see only their own; auto-moderation runs entirely Node-side. |

The setting is declared in the Space state. The home Node enforces visibility — clients receive only what their role permits, regardless of UI implementation. The client does not implement filtering; it renders what arrives.

---

### 6.12.6 Auto-moderation consequences

The home Node may issue signed membership events when its plugin determines that a member's temperature requires intervention. These are protocol-level events recorded in the DAG, distinct from the temperature value itself:

- `membership.kick` with `reason = "auto_temperature"` — applied to human members at the home Node's chosen action threshold. The member is removed from the Room with a `cooldown_until` timestamp. Default cooldown: 2 hours, Space-configurable.
- `membership.mute` with `reason = "auto_temperature"` — applied to AI members (`is_ai = true`) at the equivalent threshold. The member retains membership and Room context but cannot post until `cooldown_until` elapses. Default cooldown: 15 minutes, Space-configurable.

The asymmetry (human kick vs AI mute) is a recommendation to plugin authors, not a protocol mandate. The protocol distinguishes `membership.kick` from `membership.mute` and makes `is_ai` observable on every Identity; what a plugin chooses to do with that information is the plugin's responsibility. A Space whose plugin treats both identically is valid; a Space whose plugin uses no auto-moderation at all is valid.

The cooldown timestamps land on the signed event at issue time. If the Space owner changes the cooldown defaults later, only future auto-temperature events carry the new value; existing cooldowns in the DAG are immutable.

---

### 6.12.7 Component touch-points

The Avatar component (§6.2 component inventory) reads `--xgen-member-temperature` and `data-temp-state` and surfaces them to its CSS. No JavaScript logic in the component beyond writing the attributes when the meta_atts update arrives.

A planned Room banner / Room header component carries the room-level attributes. A Room list entry component may also carry them — the design choice between "show temperature on the list entry" and "show only inside the Room" is left to skin authors.

When the member list, message attribution, and hover-card UI all share an Avatar component (the avatar-as-first-class-object principle from earlier UI design notes), they all inherit temperature rendering uniformly. This is the intended outcome — one avatar object, one temperature surface, every appearance of a member updates together.

---

### 6.12.8 What is NOT in this section

- **The mathematical model.** Outside the protocol; lives in the home Node's plugin.
- **The action threshold.** Outside the protocol; lives in the home Node's plugin. The display thresholds (§6.12.2) and the action thresholds are independent — a member may render as `fiery` for some time before any auto-moderation fires, and that asymmetry is correct.
- **The decay model.** Outside the protocol; the home Node decides when to publish updated temperature values, the client just renders what arrives.
- **Persistence across restarts.** Temperature is computed live by the home Node. If the Node restarts, it recomputes from the recent Event stream. The client treats a temperature update arriving after reconnect identically to any other update.
- **Cross-Node temperature.** A Room has exactly one home Node (the authoritative one). Other federated Nodes may receive temperature values via `meta_atts` on relayed events; they do not recompute. If a client reads a federated copy of the Room's events from a non-home Node, the temperature values are still the home Node's values — relayed, not derived locally.

---

### 6.12.9 Phase 2 protocol implications

The following must exist in Ch3 (already partially specified in J-063):

- `xgen.room_temperature` and `xgen.member_temperature` reserved as `meta_atts` keys in the `xgen.*` namespace.
- `temperature_thresholds` field on the Room metadata response (Node-to-client session message, not a DAG event).
- `member_temperature_visibility` field on Space state — open enum, three Ch6 values defined above; Node enforces visibility on outgoing meta_atts.
- `auto_temperature` permitted as a reason value on `membership.kick` and `membership.mute` events.
- Default cooldown values for `auto_temperature` kicks and mutes — Ch3 defines the protocol fields, Ch6 defines the UI defaults; the home Node's plugin chooses what to write.

No new EventType is introduced for temperature itself. The mechanism rides existing `meta_atts` and existing membership events. This is deliberate — temperature is a UI signal, not a protocol-level state primitive.

---

## 6.13 AI Member Badge

AI Identities are first-class members of Spaces and Rooms (D-059, Ch1 §"AI as a First-Class Member"). The UI treats them identically to human members in every visual respect except one: a small, unobtrusive **AI badge** marks the member as non-human wherever the member's identity is surfaced. This section specifies the DOM contract and the placement rules; visual rendering belongs to skin CSS.

The badge exists for one reason: members reading a room need to know whether they are addressing a human or an AI. Beyond that single transparency need, no other UI distinction applies — same avatar, same name, same message bubble, same place in the member list. The badge is a label, not a separator.

Cross-references: Ch1 §"AI as a First-Class Member", D-059 (AI Identity model), §6.12 (Temperature Property — relies on `is_ai` for the kick-vs-mute asymmetry).

---

### 6.13.1 Source of truth

The `is_ai` field on the Identity record (declared at registration, immutable thereafter — D-059, Ch3 §3.6) is the single source. When the client renders any element representing an Identity — avatar, name, message attribution, member list entry, hover card — it reads `is_ai` from the cached Identity record and writes a DOM attribute accordingly.

If `is_ai` is unknown (Identity not yet replicated to the client's home Node), the badge is omitted. The badge appears only when `is_ai = true` is positively confirmed. False or absent → no badge.

---

### 6.13.2 DOM contract

The badge is a data attribute on the Identity's representative element, identical in pattern to the temperature attributes in §6.12.4:

```html
<div class="xgen-avatar" data-is-ai="true">
  ...
</div>
```

The attribute appears on:

- Avatar element (universal — every appearance of the member's avatar carries it)
- Member list entry element (for skin freedom to badge the row, not just the avatar)
- Hover card element (rich identity surface)

The attribute is **absent** for human members, not set to `"false"`. Skins target `[data-is-ai="true"]` as the styling selector.

A skin that wants no AI badge does nothing — the attribute is present, no rule consumes it, no visual change. A skin that wants the badge styles the selector. A skin that wants different badges per surface (e.g. small dot on avatar, full label on hover card) styles each selector independently.

---

### 6.13.3 Default rendering — reference skin

The default skin's reference rendering of the AI badge is intentionally minimal:

- **On the avatar:** a small circular indicator overlaid on the avatar's bottom-right corner. Size and colour are skin tokens (`--xgen-ai-badge-size`, `--xgen-ai-badge-color`).
- **On the member list entry:** the avatar's badge is sufficient; no additional decoration on the row.
- **On the hover card:** an explicit label — "AI" — next to the display name.
- **On message attribution:** no badge by default. Messages from AI use the same shape as human messages (D-059). The badge on the avatar accompanying the message is the only visual signal at the message level.

This is the reference rendering only. Skins may render the badge differently — a coloured ring around the entire avatar, an icon next to the name, a tag below the avatar, no badge at all. The DOM contract is fixed; the visual treatment is open.

---

### 6.13.4 What the badge does NOT signal

The AI badge signals one thing: this Identity is an AI. It does **not** signal:

- **Trust level.** AI Identities carry the same Auth Tier as humans in the same Space (D-059). The Tier glyph (§6.11.4) appears in the Console status bar, not on the member.
- **Operator identity.** Who runs the AI is a separate piece of information, surfaced in the hover card or member detail screen — not in the badge.
- **AI capabilities.** Whether the AI has `dm_initiate` or `spontaneous_post` enabled is operational metadata, not a visual signal on the badge.
- **Online/offline status.** Same as for humans, handled by a separate presence indicator.
- **Temperature.** Member temperature (§6.12) is rendered through its own DOM contract.

These remain independently styleable surfaces. The badge does not absorb them.

---

### 6.13.5 Plugin slot interaction

The AI badge default rendering may be replaced by a module-supplied widget. A module that wants to render a richer AI indicator (e.g. showing the model name, the operator's avatar, a tuning state) may register against the `member.ai_decoration` slot (preliminary slot name — final inventory in Ch6 second pass, §6.8.3).

When a module occupies the slot, the default badge is suppressed and the module's widget renders in its place. This follows the same pattern as the `room.message.decorator` slot (§6.7) — the protocol surface is fixed, the rendering is replaceable.

If multiple modules target the same slot, the user chooses which renders in the module list (§6.8.5).

---

### 6.13.6 Phase 2 protocol implications

None. The badge is a UI surface that reads an existing Identity field (`is_ai`, already specified in J-063 Ch3 §3.6.6) and renders accordingly. No new protocol fields, no new EventTypes, no new wire format. The entire section is client-side rendering policy.

The single dependency is on Identity replication (Layer 15 / D-049) carrying `is_ai` correctly across federated Nodes — already specified in J-063 Ch3 §3.6.10 as part of the AI Identity Extension.

---

## 6.14 Pacing Queue

Every Space declares two pacing rules: `human_pacing_ms` (minimum interval between sends for human members) and `ai_pacing_ms` (minimum interval for AI members). The rules are space-level configuration (D-060, Ch3 §3.7.12), applied per Space and equal in authority to the Space's Auth Tier requirement and federation list. This section specifies how the client enforces pacing on outbound messages — the queue mechanism, the asymmetry between human and AI rendering, and the DOM contract for operator-visible AI clients.

Enforcement is **client-side only in Phase 2** (D-060). The Node does not validate pacing on incoming events. A misbehaving client that bypasses pacing shows up clearly in event timestamps and is subject to moderator action and to the temperature mechanism (§6.12). Phase 3+ may add Node-side enforcement if abuse becomes practical; until then, well-behaved clients are trusted in the same way they are trusted for role permissions client-side before Node-side validation.

Cross-references: D-060 (pacing rules decision), Ch3 §3.7.6 (Space state fields), Ch3 §3.7.12 (Pacing Rules on Spaces), D-059 (AI as first-class member — defines `is_ai` used to select which cap applies), §6.12 (Temperature Property — fed by pacing overpasses).

> **XGID typing (Retrofit Pass 4).** The client `PacingManager` keys its per-Space rules by `SpaceXgid` and its per-(space, sender) queues by the composite `(SpaceXgid, IdentityXgid)`; the public `PacingState` snapshot carries `space_id: SpaceXgid` and `sender_identity_id: IdentityXgid` (typed in memory, plain `String` on the wire via serde-transparency). Lookup-by-`&str` is preserved through `Borrow<str>` (Pass 1 additive API), so the enforcement API stays ergonomic.

---

### 6.14.1 Selecting the applicable cap

On every outbound send, the client selects the cap based on the sender's `is_ai` field:

- `is_ai = false` or absent → `human_pacing_ms`
- `is_ai = true` → `ai_pacing_ms`

A single client may host multiple Identities (rare in practice) but each Identity's `is_ai` is immutable (D-059), so the selection is deterministic per sender. A client signing as a human Identity never falls under `ai_pacing_ms` and vice versa.

The selection happens at queue-entry time, not at queue-release time. If the Space owner updates the pacing rules while the queue holds messages, those messages release under the cap that was active when they entered the queue. The new cap applies only to messages enqueued after the update.

---

### 6.14.2 Outbound queue mechanism

The client maintains one outbound queue per (Space, sender) pair. The queue is in-memory only — not persisted across client restart. If the client closes with messages still queued, those messages are lost; the user must retype.

On each send attempt:

1. The client records the timestamp of the current send attempt as `attempt_at`.
2. The client looks up `last_send_at` — the timestamp of the most recent successfully sent message in this (Space, sender) pair, or `0` if none.
3. The client computes `elapsed = attempt_at − last_send_at`.
4. If `elapsed >= cap_ms`, the message is released immediately. `last_send_at` is updated to the actual send time.
5. If `elapsed < cap_ms`, the message is enqueued with a `release_at` timestamp of `last_send_at + cap_ms`. A timer fires at `release_at` to release the message and update `last_send_at`.

Multiple messages enqueued in succession release sequentially, each `cap_ms` after the previous one's release. A burst of N messages takes `(N − 1) × cap_ms` to fully drain.

The queue is FIFO. Reordering of queued messages is not supported — the client sends in compose order.

---

### 6.14.3 Human enforcement — silent throttle

For human senders, the queue operates **silently** by default. The user types and hits send; the message either goes out immediately or waits the necessary fraction of a second; the user typically does not notice.

The silent rule applies only when the wait is short enough to be invisible to a human — a 500 ms default cap (D-060) is below the threshold of perceptible delay for normal typing. When the queue holds messages whose total release time exceeds a threshold (Ch6 default: 2 seconds; skin-configurable via `--xgen-pacing-visible-threshold-ms`), the UI surfaces a small indicator showing pending message count and estimated drain time.

This matches the D-060 principle: humans should not be aware of pacing they have not actually exceeded. The indicator appears only when the queue meaningfully exists.

DOM contract for the human queue indicator:

```html
<div class="xgen-compose" data-pacing-state="throttled"
     style="--xgen-pacing-queue-count: 3; --xgen-pacing-drain-ms: 4500">
  ...
</div>
```

Values for `data-pacing-state`:

- `cool` (or attribute absent) — queue empty or wait below threshold; no UI signal
- `throttled` — queue holds messages over the visibility threshold; soft indicator appears

A skin styles `[data-pacing-state="throttled"]` to show a pill, dot, or text near the compose input. A skin that wants no human pacing indicator at all leaves the selector unstyled.

---

### 6.14.4 AI enforcement — visible operator surface

For AI senders, the queue is **always visible**. The AI client's UI is operator-facing; operators are tuning the AI's behaviour and need to see the constraints under which it is operating.

The AI client surfaces:

- Current `ai_pacing_ms` cap for the active Space
- Time until the next send is permitted (countdown)
- Pending queue length and estimated drain time
- Per-Space breakdown if the AI is active in multiple Spaces simultaneously

DOM contract for the AI operator surface:

```html
<div class="xgen-ai-operator-panel" data-pacing-state="holding"
     style="--xgen-pacing-cap-ms: 2000;
            --xgen-pacing-next-send-ms: 850;
            --xgen-pacing-queue-count: 4;
            --xgen-pacing-drain-ms: 7150">
  ...
</div>
```

Values for `data-pacing-state` (AI client only):

- `clear` — cap satisfied, next send goes immediately
- `holding` — next send waits for the cap to elapse; countdown active
- `queueing` — multiple messages waiting; full operator surface visible

The AI client may also expose the queue programmatically (read-only) via the same IPC interface used by `--batch` (§6.9) so that operators can observe and tune pacing behaviour from automation scripts. The mechanism is the same; only the audience differs.

---

### 6.14.5 Interaction with temperature

The pacing queue is the input signal for the temperature mechanism (§6.12). A pacing overpass — a send that the queue had to delay — is reported to the home Node's temperature plugin as a temperature event. The plugin's accumulator (§6.12.8 "What is NOT in this section" — the mathematical model is plugin business) folds the overpass into the member's temperature value.

The client does not compute temperature. The client only reports the overpass: this Identity attempted to send before the cap had elapsed, in this Space, at this timestamp. What the home Node does with that signal is the plugin's responsibility.

A well-behaved client whose queue is doing its job produces zero overpasses — every send waits its turn. A misbehaving client that bypasses the queue and sends faster than the cap produces visible overpasses (because the home Node observes the timestamps and can compute the delta itself). Temperature accumulates on the misbehaving client's sender, leading eventually to `auto_temperature` consequences (§6.12.6).

This closes the loop: client-side pacing is trusted, but trust is verified by the temperature mechanism. A client that violates pacing accumulates temperature; a client that respects pacing stays cool. The honest path is also the fast path.

---

### 6.14.6 Edge cases

**Clock skew.** If the client's system clock jumps backward (NTP correction, manual change), `last_send_at` may briefly exceed `attempt_at`, making `elapsed` negative. The client treats negative `elapsed` as `0` — the first send after a backward clock jump goes immediately, subsequent sends respect the cap from the new timeline. No queue corruption; no replay attack risk because the Node validates send timestamps independently.

**No `is_ai` available.** If the Identity record has not yet been retrieved at send time (extreme edge case — first send after first registration with a slow home Node), the client falls back to `human_pacing_ms`. The conservative default is the lighter throttle; AI clients in this state are by definition not yet authenticated to the home Node and therefore have no sends to enqueue.

**No pacing rules in Space state.** If `human_pacing_ms` or `ai_pacing_ms` is absent (e.g. legacy Space created before the field existed), the client applies the Ch6 defaults (500 ms / 2000 ms per D-060). The client does not generate state events to fill in missing fields; that is the Space owner's decision.

**Cap of zero.** A Space owner who sets a pacing cap to `0` disables throttling for that member class. The queue passes messages through immediately with no delay. This is a valid configuration (a fast-chat Space might set `human_pacing_ms: 0`); the client honours it.

---

### 6.14.7 Phase 2 protocol implications

None. The pacing fields (`human_pacing_ms`, `ai_pacing_ms`) are already specified in J-063 Ch3 §3.7.6 and §3.7.12. The temperature signal channel (overpass reporting to the home Node's plugin) is internal Node business; the client's only protocol-visible behaviour is the send timestamp, which is already part of every event.

The entire section is client-side implementation policy. The DOM contracts (`data-pacing-state` and the `--xgen-pacing-*` custom properties) are Ch6 conventions consistent with §6.12 and §6.13, applied to the compose component and the AI operator panel respectively.

---

## 6.15 AI Client (resident mode)

The AI Client is a *mode of `xgen-client`*, dispatched when the CLI receives `--ai-mode --service`. It is a long-running resident — same binary, same protocol library, same Node connection mechanics as the human Client — that consumes inbound events through a configurable plugin and emits replies under the existing pacing and mute constraints. This section documents how the AI Client mode is built, the plugin contract, the runtime loop, and the lifecycle.

For the protocol-level surface that the AI Client consumes (AI Identity declaration, `is_ai` immutability, capability flags, operator role, fall-upward resolution, AI-owned-Space prohibition), see Ch3 §3.6.10 and its sub-sections — that is the spec-side material; this section is the implementation-side counterpart.

### 6.15.1 Mode selection and dispatch

`xgen-client` exposes three top-level modes:

| Invocation | Role |
|---|---|
| `xgen-client <subcommand>` | One-shot human Client |
| `xgen-client --service` | Long-running human-Client resident |
| `xgen-client --ai-mode --service` | Long-running AI-Client resident |

The `--ai-mode` flag is meaningful only in combination with `--service`; clap rejects standalone uses. The dispatch in `xgen-client/src/main.rs` routes `--ai-mode --service` to `ai_service::run` (sibling of `service::run`), preserving the existing scaffold for logging, PID file, pipe server, and Ctrl+C handling.

A single Identity may be staged as either a human or an AI client. The decision is recorded in `[ai] is_ai = true|false` in `xgen-client_config.toml` (M3 surface; `init --ai` writes this). The AI mode refuses to start without `is_ai = true` and a named plugin.

### 6.15.2 Configuration

The AI Client reads `xgen-client_config.toml` (same file as the human Client). M4 adds two pieces to the existing `[ai]` section from M3:

```toml
[ai]
is_ai = true
plugin = "echo"            # which plugin to load

[ai.capabilities]
dm_initiate = false
spontaneous_post = false

[ai.behavior]              # per-plugin config; each plugin owns its keys
mention_token = "@bob"      # optional, plugin-specific
```

The split between `plugin = "..."` (in `[ai]`) and `[ai.behavior]` (plugin's own config sub-table) is deliberate: "which plugin to load" is a single-line toggle, while "how that plugin is tuned" lives in its own namespace. When a second plugin lands, swapping plugins is a one-line edit; the `[ai.behavior]` table contents swap in tandem but stay isolated from the selection itself.

Open-enum on plugin name — unknown values are tolerated by config parsing but rejected at startup by the runtime loader. Each plugin documents which `[ai.behavior]` keys it consumes; unknown keys are tolerated (forward compat).

### 6.15.3 The `AiBehavior` trait

Plugins implement the `AiBehavior` trait in `xgen_client_lib::ai_behavior`:

> **XGID typing (Retrofit Pass 4).** `EventContext.ai_identity_id` is a `&IdentityXgid` (typed in memory); mention-detection reads it via `.as_str()`. The `on_event` return is reply *text* (`Option<String>`, descriptive — not an identifier). This is Pass 4's typed-XGID annotation scope only; the M7 `--aicontrol` v1 redesign is a separate milestone.

```rust
pub trait AiBehavior: Send {
    fn on_event(&mut self, ctx: &EventContext) -> Option<String>;
    fn name(&self) -> &'static str;
}

pub struct EventContext<'a> {
    pub event: &'a Event,
    pub ai_identity_id: &'a str,
    pub mention_token: Option<&'a str>,
}
```

The plugin receives one inbound `Event` at a time and returns either `Some(text)` to reply with `text` as `message.text`, or `None` for "no reply." The runtime takes it from there — pacing, mute enforcement, prev_events chaining, and WebSocket I/O are all runtime concerns, not plugin concerns. The plugin sees only what it needs to make a decision.

Implementations MUST be fast and non-blocking. Long-running compute (LLM inference, web requests) is a future-plugin concern that will require its own architecture; the M4 trait is intentionally narrow.

### 6.15.4 Reference plugin: `echo`

M4 ships one plugin: `EchoPlugin`, registered under the config key `"echo"`. Its job is to prove the loop end-to-end, not to be useful. It:

- Watches `message.text` events.
- Detects mentions via two OR'd rails (see §6.15.5).
- Replies with the deterministic line `[echo-plugin] received mention from <sender_id_short>`, where `sender_id_short` is the last 12 characters of the mentioning Identity's `identity_id`.
- Returns `None` for its own events, non-`message.text` events, and events without a matching mention.

The reply text is **not configurable** in M4. The format is fixed for grep-ability in smoke tests and for unambiguity in early demos — nobody should mistake the artefact for a real reply. Configurable reply text is a future-plugin concern.

### 6.15.5 Mention detection (two-rail OR)

The reference plugin (and any future plugin choosing to use the same convention) detects mentions through two independent rails:

1. **Rail A — always-on:** substring match for the AI's full `identity_id` URI in `content.text`. Deterministic, no config needed.
2. **Rail B — optional:** substring match for a `mention_token` (e.g. `"@bob"`) read from `[ai.behavior]`. Default unset.

The rails are **OR'd, not sequenced.** Either match independently counts as a mention. The implementation MUST NOT interpret "always + optionally" as "fall through to optional if always-rail misses" — both rails evaluate independently and any match triggers a reply.

**Case sensitivity:** both rails are case-sensitive by default. URIs are case-sensitive per RFC 3986; the token follows the same convention for predictability. A future config knob `mention_case_insensitive` may be added if a real use case appears.

### 6.15.6 Runtime loop

The AI resident's inner loop lives in `xgen_client_lib::ai_service::run_ai_loop`. On startup it:

1. Loads keypair + node URL from config.
2. Reads `[ai]`, refuses to start if `is_ai = false` or `plugin = …` is missing.
3. Loads the named plugin via `load_plugin()`.
4. Connects to the home Node, authenticates via the standard challenge-response, sends `transport.sync_request` to catch up on Space history.
5. Enters the receive loop.

On each inbound event:

1. Apply to the local per-Space `SpaceState` (initialised from `state.space_create`, updated by every applicable event). The runtime maintains this so it can consult `active_mutes` and `ai_pacing_ms`.
2. Track the most recent event ID per Space — replies chain to it via `prev_events`.
3. Invoke `plugin.on_event(ctx)`. If the plugin returns `Some(reply_text)`, run it through the mute and pacing gates (§6.15.7) before emitting.
4. Refresh the health-state snapshot the pipe server reads.

The loop exits on Goodbye or connection error; the pipe server stays up so the operator can still `--stop` the process cleanly.

### 6.15.7 Pacing and mute — drop, don't queue

Pacing is per-Space. Each Space carries `ai_pacing_ms` (default 2000) — the minimum interval between AI events. The AI runtime tracks `last_send_at_ms` per Space; before emitting a reply, it checks `now - last_send_at_ms >= ai_pacing_ms`. If not, the reply is **dropped** (not queued).

Why drop instead of queue: queueing produces *stale* replies. By the time the cooldown expires, the conversation has moved on; a queued reply would now misrepresent the AI's current state rather than reflecting it. Dropping is honest: "I had something to say at the moment, but you set a rate limit; I respected it and the moment passed."

This is an instance of a recurring XGen design principle named at M4 review: **honest behaviour over polite behaviour.** When a system can choose between behaviour that misrepresents its current state (polite — "I'll deliver this thought eventually") and behaviour that honestly reflects its current state (honest — "I can't say this right now"), XGen picks honest. The same logic appears in the fall-upward operator resolution (returns the currently-resolvable operator, not a stale stored value), in the Node's event-acceptance pipeline (drops events it can't validate rather than queueing them indefinitely), and in mute semantics (mute is a wall, not a delay). See D-065 for the named principle and its other instances.

Drops are logged at WARN with the literal phrase `dropping reply — pacing cap not yet elapsed (honest behaviour over polite behaviour)` so the principle is greppable in production logs.

Mute is enforced on the same path. Before pacing, the runtime checks `SpaceState.active_mutes` for the AI's `identity_id`; a present entry causes the reply to be dropped silently (no special log line). Auto-temperature mutes (spec 3.7.13.6) follow the same path with no AI-specific treatment.

### 6.15.8 Lifecycle, control commands, and observability

The AI resident exposes the standard pipe-server control commands inherited from M2:

| Command | Behaviour for an AI-mode resident |
|---|---|
| `__PING__` | Returns `PONG <ms>` immediately. |
| `__HEALTH__` | Extended for AI mode (see below). |
| `__STOP__` | Exits the process via `std::process::exit(0)`. |
| `__RELOAD_CONFIG__` | Returns `NOT_IMPLEMENTED` (same as the human Client). |

The pipe name follows the existing convention `\\.\pipe\xgen-client[-<instance>]` — the AI resident binds to the same pipe space as the human resident, distinguished by the `mode=` field in the `__HEALTH__` reply rather than by a separate pipe.

`__HEALTH__` reply format for an AI-mode resident:

```
HEALTHY pid=<pid> mode=ai operator_known=<known>/<total>
```

Where `<known>` is the number of Spaces the AI is a member of for which `resolve_operator` returns `Some(...)`, and `<total>` is the number of Spaces the AI is a member of. A coarse signal: `operator_known=2/3` tells the operator at a glance that one Space is in orphan state without forcing a follow-up `status` call. The structured per-Space operator map stays on `xgen-client status` (offline-local) — `--health` is the one-liner, `status` is the detailed view.

For the human-Client resident, the same handler returns `HEALTHY pid=<pid> mode=human` (no `operator_known` field) — the format is consistent across modes; AI-only fields are appended.

### 6.15.9 Manual join — no auto-join

The AI Client does **not** auto-join Spaces on startup. Joins are operator-driven via the existing `xgen-client --instance <ai-label> join --space <id>` (one-shot CLI). The resident's WebSocket loop receives the resulting `membership.join` event like any other event; it never originates a join itself.

Rationale: auto-join would make an AI Identity's first observable behaviour in a Space config-driven rather than chosen, muddying the trust model. Manual join keeps presence as an explicit, auditable event in the DAG. The operator stays in the loop for the question "did this AI actually want to be here?"

Testing convenience is preserved by the fact that the operator drives the join from the same machine the AI resident runs on — both processes share the keypair file, and the one-shot join CLI invocation is a one-line addition to any smoke script.

### 6.15.10 Out of scope (forward-references)

The M4 deliverable explicitly excludes:

- **Real LLM hookups.** Future plugins as additional `AiBehavior` impls.
- **Operator command surface.** No protocol-level operator-signed events exist yet (see Ch3 §3.6.10.6 LOCKED on this); designing the AI Client around them would load weight on something unbuilt. When that protocol surface lands, it'll layer on the existing trait.
- **Temperature surfacing or room-temperature reaction.** The AI Client receives `temperature.update` like any client but doesn't emit temperature or react to thresholds. Conversational-dynamics design conversation; defer.
- **Cross-Space coordination, multi-device AI Client, Tauri/UI surface.** All future milestones.

Forward-reference: the protocol primitives this section consumes — `is_ai` declaration, capability flags, operator role and fall-upward resolution, `state.ai_operator_delegate` / `_revoke`, AI-owned-Space prohibition with error 3041 `ai_role_violation` — live in Ch3 §3.6.10. Read that for the protocol semantics; read this section for how the client mode is built on top.

---

## 6.16 The `self` thread (Saved Messages)

A personal single-user thread for notes-to-self — text messages with full chronological history, surfaced to the user as **"Saved Messages."** It is realised as a **self-DM**: a DM Space whose creator and sole invitee are the same identity, built on the existing Space/Room/Event/DAG machinery with no new protocol surface.

**Reuses the user's existing identity.** The `self` thread is not a separate account and has no second keypair — both endpoints of the DM are the user's own already-registered identity. There is no new registration and no synthetic local-only key; `self` is *you*, addressed to yourself. This is the anchor that keeps the feature from drifting into "a second account."

**Never federated, never broadcast.** A self-DM inherits the DM non-federation guarantee structurally: the same `DmFederationNotAllowed` rule that walls every DM off from federation applies, so a `self` thread's events never leave its home Node. Privacy here is a structural property of the DM primitive, not a configurable default.

**Reach.** The thread is reachable from any client authenticated as the user — their own devices. Because it lives Node-side (not in device-local storage), a second device authenticating as the same identity sees the same thread and its full history. It is Node-resident, not device-local.

**Attachments** are an inherited capability: when M12 lands, the `self` thread carries attachments through the same event/blob mechanism as any DM. M11 ships text-first.

**Boundary.** The `self` thread is not an account, not a Node-side service, and introduces no new wire type, event kind, or reject code. The entire protocol/applier delta is the existing DM creation path with a single construction-time guard that skips the (vestigial) self-invite when the invitee is the creator; the client adds a thin `self` convenience verb (create-if-absent → open) and a "Saved Messages" label over the existing send/history surface.

*Decision record: D-021 (reconciled — registered via the existing identity, never federated; the pre-machinery "never registered" clause relaxed, its spirit preserved). Milestone M11. No Phase 2 protocol implications.*

---

## Session Log

### Session 1 — April 2026 (JozefN)
**Covered:** Chapter 6 preliminary written. Confirmed architectural decisions: Tauri + Svelte for both Node and Client executables, Pattern A compliant, single shared design system in `xgen-ui-shared/`. CSS token system defined with full category list (color, typography, spacing, border, shadow, motion). Three-layer theming cascade confirmed (default → application → Space). Preliminary screen inventories for Node Admin UI (7 screens) and Client UI (6 screens). Auth Module UI as embedded modal flow in Client. `state.space_theme` EventType identified as likely new protocol requirement. JavaScript scope deliberately minimal — all logic in Rust, Svelte handles only presentation. Keypair exception added: key files may be stored anywhere (cloud storage explicitly supported), `keypair_path` config field declares location.

**Pending for second pass (after Phase 1 implementation):**
- Actual token values (colors, typeface)
- Permitted Space theme override token list
- Full component specifications
- Detailed screen wireframes
- Complete protocol implications list

### Session 2 — April 2026 (JozefN)
**Covered:** Section 6.8 Module Architecture written in full (resolves OQ-01, D-036). Eight subsections: 6.8.1 Communication Model (Event subscription + meta_atts; keys namespaced `xgen.module.<id>.<key>`; any language that speaks WebSocket can write a module); 6.8.2 Module Package and Manifest (one folder = one package regardless of internal complexity; full manifest schema with 12 fields including settings_schema rendered automatically; `modules/` subfolder in working directory); 6.8.3 UI Forms (headless = background only; widget = HTML injected into named slot; window = full Tauri webview launched from module list; preliminary injection slot inventory: 7 slots); 6.8.4 Identity Modes (`system` = own keypair, signs as itself; `user` = signs as authenticated user, requires explicit consent at install, revocable at any time); 6.8.5 Module List (universal registry; every module appears regardless of form; stacked block visual structure with status indicator, mode badge, settings, launch, disable, remove); 6.8.6 Capability Advertisement (active module capabilities added to node announcement automatically via open enum mechanism); 6.8.7 Auth Module as Reference Implementation (demonstrates all three aspects: Event subscription, system identity, window UI form; not special — same manifest as any third-party module); 6.8.8 Open Questions (hot-loading Phase 3; module signing Phase 2; widget sandboxing Phase 2; module permissions Phase 2; module-to-module communication Phase 2).

### Session 3 — May 2026 (JozefN)
**Covered:** Section 6.9 Console Input Channel Protocol written as a formal open question for Phase 2 design. Three operation modes defined: Mode 1 batch file (`--batch` flag, `.xgb` format, no UI required), Mode 2 AI-assisted interactive (human present, agent injects via IPC, human can intervene), Mode 3 checkpoint-driven admin processes (agent drives, human approves at decision points). Five design questions documented. Philosophical grounding cross-referenced to Ch1 Human and Agent Operation section. Section 6.11 Console written in full (renumbered from 6.10 to accommodate 6.9). Seven subsections covering purpose, display model, visual design, structure, client and node session lifecycle, and relationship to other screens. Infrastructure transparency principle documented. Tier glyph color coding defined.

### Session 4 — 2026-05-15 (JozefN)
**Covered:** CSS architecture and component library design discussion. Two decisions recorded (D-057, D-058) and reflected in this document.

D-057 — Custom app base CSS layer model. Traditional browser normalize (Normalize.css or similar) explicitly rejected: the HTML element model it covers is incompatible with a Svelte component application that does not use most of those elements. Replaced with a minimal `base.css` (~50 lines) covering only: universal box-model reset, root type scale, and resets for the three browser-aggressive elements the app actually uses (`button`, `input`, `a`). Four-layer CSS architecture formalised: `base.css` (always loaded) → `tokens.css` (variable values) → `skin-dark.css` (visual identity) → `components/` (Svelte `<style>` blocks). Degradation chain corrected from D-041 ("reset coupled to skin") — correct behaviour is: requested skin → default skin → base.css-only structured layout (legible, not raw HTML).

D-058 — 4px root spacing unit and 13px/1.35 app type scale. All spacing expressed in named steps (`--xgen-space-1` through `--xgen-space-16`) derived from a 4px root unit. 13px set as `html { font-size: 13px }` in `base.css` — all font-size tokens expressed in `rem` to support accessibility rescaling. Line-height locked at 1.35 app-wide. Typography is component-scoped — components set font-size and line-height for their own elements; no global cascade rules beyond the html root. No hardcoded pixel or color values anywhere in component code.

Component independence principle documented: each component in `components/` is self-contained with no cross-component imports. The slot inventory in §6.8.3 maps to the independently injectable component set.

`xgen-ui-shared/` folder structure updated to reflect all four layers. §6.2 font size token scale comments corrected to match the 13px root (previously listed "base = 15px equivalent" — now correctly "base = 13px root, D-058"). Clarifying note added to §6.3 distinguishing CSS architecture layering (build-time, §6.2) from application theming cascade (runtime, §6.3).

### Session 5 — 2026-05-15 (JozefN)
**Covered:** §6.12 Temperature Property written in full as the Ch6 first pass on D-061. Nine subsections: 6.12.1 what the client receives (two `meta_atts` keys, opaque floats); 6.12.2 threshold table (Node-supplied at room-open, Ch6 defaults if absent); 6.12.3 client-side derivation (one comparison per update, not per frame); 6.12.4 DOM contract (data attribute + CSS custom property pair on Room banner / Avatar elements, skin styles freely); 6.12.5 visibility policy (room temperature public, member temperature moderator-default with `member_temperature_visibility` Space setting carrying three values: `moderator` / `everyone` / `self_only`); 6.12.6 auto-moderation consequences (`auto_temperature` reason on kick/mute, AI vs human asymmetry as plugin recommendation not protocol mandate, default cooldowns 2h / 15min); 6.12.7 component touch-points (Avatar component as universal temperature surface); 6.12.8 explicit non-scope (math model, action threshold, decay, persistence, cross-Node — all outside protocol); 6.12.9 Phase 2 protocol implications summary. No new EventType introduced — mechanism rides existing `meta_atts` and existing membership events. The mathematical model lives in the home Node's plugin and is intentionally not specified by either protocol or client.

### Session 6 — 2026-05-15 (JozefN)
**Covered:** §6.13 AI Member Badge written as the Ch6 first pass on the D-059 UI surface. Six subsections: 6.13.1 source of truth (`is_ai` Identity field is the single input; absent or false → no badge); 6.13.2 DOM contract (`data-is-ai="true"` attribute on avatar / member list entry / hover card; absent for humans rather than set to false); 6.13.3 default reference-skin rendering (small corner indicator on avatar, explicit "AI" label on hover card, no message-level distinction — matches D-059); 6.13.4 explicit non-scope (badge does NOT signal Tier, operator, capabilities, presence, or temperature — each has its own independent surface); 6.13.5 plugin slot interaction (`member.ai_decoration` slot preliminary; module widget replaces default badge when registered); 6.13.6 Phase 2 protocol implications (none — entirely client-side, reads existing Identity field). No new protocol surface, no new EventTypes, no wire format change.

### Session 7 — 2026-05-15 (JozefN)
**Covered:** §6.14 Pacing Queue written as the Ch6 first pass on the D-060 client surface. Seven subsections: 6.14.1 cap selection (`is_ai` chooses between `human_pacing_ms` and `ai_pacing_ms`; selected at queue-entry, not queue-release); 6.14.2 outbound queue mechanism (FIFO, in-memory, per (Space, sender) pair, deterministic release scheduling); 6.14.3 human silent throttle (queue invisible by default; appears via `data-pacing-state="throttled"` only when total drain time exceeds a configurable visibility threshold, Ch6 default 2 seconds); 6.14.4 AI visible operator surface (always-visible operator panel with countdown, queue depth, per-Space breakdown; `data-pacing-state` carries `clear` / `holding` / `queueing`; queue exposed programmatically via the §6.9 IPC channel); 6.14.5 interaction with temperature (overpass reporting to home Node feeds the temperature plugin; well-behaved clients produce zero overpasses; misbehaving clients accumulate temperature, closing the trust loop); 6.14.6 edge cases (clock skew, missing `is_ai`, missing pacing rules, cap of zero); 6.14.7 Phase 2 protocol implications (none — entirely client-side, reads existing Space state fields and produces existing send-timestamp signal).

### Session 8 — 2026-05-17 (JozefN)
**Covered:** §6.15 AI Client (resident mode) written as Ch6's first-pass client-side implementation home for the M4 AI Client (D-065). Ten subsections: 6.15.1 mode selection and dispatch (three top-level modes for `xgen-client`; `--ai-mode` requires `--service`; clap rejects standalone uses); 6.15.2 configuration (the M3 `[ai] is_ai = true` + `[ai.capabilities]` extended by M4 with `plugin = "echo"` and a `[ai.behavior]` sub-table; deliberate split between "which plugin" and "how that plugin is tuned"); 6.15.3 the `AiBehavior` trait (`on_event` returning `Option<String>`, fast-and-non-blocking contract, runtime owns pacing/mute/prev_events/I-O); 6.15.4 reference plugin `echo` with deterministic reply format (`[echo-plugin] received mention from <last-12>`) and explicit non-configurability rationale; 6.15.5 mention detection (two-rail OR'd: identity_id substring always-on plus optional `mention_token` from `[ai.behavior]`; case-sensitive per RFC 3986); 6.15.6 runtime loop (load → connect → sync → receive loop applies events to per-Space SpaceState, tracks last-event-per-Space for prev_events chaining, invokes plugin, gates replies); 6.15.7 pacing and mute — drop, don't queue, articulating the named recurring principle "honest behaviour over polite behaviour" with the literal greppable WARN line (`dropping reply — pacing cap not yet elapsed (honest behaviour over polite behaviour)`) and the cross-instance list (D-064 fall-upward operator resolution, Node event-acceptance pipeline, mute-as-wall, the cmd_create_space optimistic-ack UX carry-over); 6.15.8 lifecycle and control commands inherited from M2 with `__HEALTH__` extended to `HEALTHY pid=<pid> mode=ai operator_known=<known>/<total>`; 6.15.9 manual join (no auto-join — trust-model rationale: AI presence stays explicit and auditable through `membership.join`); 6.15.10 out-of-scope forward-references (real LLM hookups, operator command surface, temperature reaction, multi-device, Tauri surface — all future milestones). No new EventTypes, no new wire shape — the section is pure client-side implementation reading existing M3 protocol primitives (`is_ai`, `ai_capabilities`, `ai_pacing_ms`, `active_mutes`, `resolve_operator`). Ch3 §3.6.10 cross-reference list extended with D-064, D-065, and the forward link to §6.15. Header bumped to 0.3 / 2026-05-17.

### Session 9 — 2026-06-14 (JozefN)
**Covered:** §6.16 The `self` thread (Saved Messages) written as the M11 close deliverable (D-021). Documents the self-DM shape (shape B): a personal single-user thread reusing the user's existing registered identity as both DM endpoints — not a separate account, no second keypair, no new registration. Never federated / never broadcast by structural inheritance of `DmFederationNotAllowed`; reachable from any client authenticated as the user (their own devices; Node-resident, not device-local); attachments inherited at M12, text-first at M11. Boundary recorded: no new wire type, event kind, or reject code — the entire protocol/applier delta is a construction-time guard skipping the vestigial self-invite when invitee == creator, plus a thin client `self` convenience verb and "Saved Messages" label. D-021 reconciled (relaxed the pre-machinery "never registered" clause; spirit preserved). No Phase 2 protocol implications. Header bumped to 0.4 / 2026-06-14.
