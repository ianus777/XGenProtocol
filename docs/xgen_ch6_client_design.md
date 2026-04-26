# XGen Protocol — Chapter 6: Client Design

> Status: preliminary — confirmed architectural decisions written; sections requiring Phase 1 implementation experience marked as pending  
> Version: 0.1  
> Date: April 2026  
> Author: JozefN  

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

**New EventTypes likely needed:**
- `state.space_theme` — Space theme declaration (referenced in 6.3 above)
- `message.thread_start` — if threads are added to the UI
- `message.edit` — if message editing is supported in the UI

**New state fields likely needed:**
- Space: `theme` object in Space state
- Room: `notification_level` per-user preference (may be client-side only)

**Phase 2 spec sections directly informed by Chapter 6:**
- 3.9 State Resolution — how conflicting `state.space_theme` Events are resolved
- 3.10 E2E Encryption — encrypted Room visual indicator in Room View
- 3.13 Identity Replication — multi-device sync visible in Identity Profile screen

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
