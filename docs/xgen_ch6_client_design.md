# XGen Protocol — Chapter 6: Client Design

> **Status:** ACTIVE  
> Version: 0.1  
> **Last updated**: 2026-05-07  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

---

## Overview

Chapter 6 specifies the XGen client applications — what they look like, how they behave, and how UI decisions feed back into Phase 2 protocol requirements.

Two applications are specified here: the **Node admin UI** (`xgennode.exe`) and the **Client UI** (`xgenclient.exe`). Both share a common design system and component library. Both are single executables following the Pattern A deployment model (spec: `IMPLEMENTATION_GUIDE_ph1.md`).

Chapter 6 is written in two passes. The first pass (this document) captures confirmed architectural decisions made before Phase 1 implementation. The second pass fills in the detailed screen specifications, component inventory, and protocol implications after Phase 1 experience is available. The second pass must be complete before Phase 2 specification begins.

---

## 6.1 Client Architecture

### Technology Stack

Both `xgennode.exe` and `xgenclient.exe` are built using **Tauri** as the desktop application framework with **Svelte** as the frontend framework.

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
  xgen-node-ui/           ← Svelte frontend for Node admin UI
  xgen-client-ui/         ← Svelte frontend for Client UI
```

The Tauri build process bundles the Svelte frontend into the Rust binary at compile time. The frontend assets are embedded in the executable — no separate asset folder, no web server. The executable extracts and serves the frontend from memory when the application window opens.

### Deployment

Pattern A applies without exception. Each executable creates and manages its own folder. The Tauri webview state (window size, position) is stored in the application folder alongside protocol data. No AppData, no registry, no system-level integration.

```
C:\XGenClient\
  xgenclient.exe          ← binary with embedded frontend
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
--xgen-font-size-xs:         /* 11px equivalent */
--xgen-font-size-sm:         /* 13px equivalent */
--xgen-font-size-base:       /* 15px equivalent */
--xgen-font-size-lg:         /* 18px equivalent */
--xgen-font-size-xl:         /* 22px equivalent */
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

---

## 6.3 Theming Model

*Preliminary — full specification in Chapter 6 second pass.*

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

The Node admin UI is the operator-facing interface for managing a running XGen Node. It opens as a desktop window when `xgennode.exe` is launched. It is not a web interface served on a port — it is the Tauri application window itself, accessible only on the machine running the Node.

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

The Console is a first-class surface in both `xgenclient.exe` and `xgennode.exe`. It is not a debug add-on — it is the canonical command surface and, for the Client, the lifecycle host that the stateless CLI invocation model does not provide.

**Full name:** XGen Client Console / XGen Node Console.

---

### 6.11.1 Purpose and role

**Client side:** `xgenclient.exe` has no persistent process between CLI invocations. Each call is stateless — logs fragment, there is no continuity to debug against. The Console window solves this by being the lifecycle host. Opening the window starts a session; closing it ends it. All Events within that window's lifetime are grouped under one session, with one log, one session ID. Without the Console, meaningful Phase 2 client-side testing is not possible.

**Node side:** `xgennode.exe` has a natural process lifecycle and does not need the Console as a lifecycle host. The Console on the Node side is an operator command surface — a first-class interface for issuing commands and observing the live log stream, equivalent in status to the admin dashboard.

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
