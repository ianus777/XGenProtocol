# XGen Client — Core Test UI
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-13 (Task 1.4 implemented; npm installed; M1–M3 done)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Implementation Status — Read First

| Milestone | Status | Notes |
|---|---|---|
| Milestone 1 — Tauri Scaffold | ✅ Done | All tasks 1.1–1.4 complete. Task 1.4 (`--instance` flag) implemented 2026-05-13. |
| Milestone 2 — Lifecycle State Machine | ✅ Done | All tasks complete. `cargo build` clean, 173/173 tests pass. |
| Milestone 3 — State Indicator Wired | ✅ Done | Svelte event listener + dot colour mapping implemented. `npm install` complete. |
| Milestone 4 — Verification | ⏳ Not started | Manual walkthrough required — start Tauri dev build and verify checklist. |

**Mr. Code's immediate task:**
1. Run Milestone 4 verification checklist (manual UI walkthrough)

---

## Purpose

This document is the implementation instruction for Mr. Code to produce the `xgen-client` Core Test UI — the first real Tauri window for the client binary. The result is a minimal but functional application window that:

- Displays the XGen Client identity (logo, app name)
- Shows the current lifecycle state in real time (Appendix E — 11 states)
- Provides the Quit action
- Establishes the Tauri scaffold and Svelte build pipeline for Phase 2 Track 1

This is the first UI deliverable of Phase 2. It is intentionally minimal — a test instrument, not the final UI. The full client UI (Spaces, Rooms, Console, message stream) follows in later phases.

---

## Design Reference Files

The visual design is owned by the project. Mr. Code must preserve:

- **Graphical look**: colors, typography, spacing, border radius — as in the reference files below
- **UI texts**: app name, button label, state labels (from Appendix E display labels)
- **Logo**: use `ui/dev_core_ui/shared_assets/logo_client_64.png`

The HTML/CSS/Svelte structure may be changed freely. Mr. Code must also verify that attribute names, CSS class names, and CSS IDs in Joe's reference files are aligned with spec conventions — flag and correct any discrepancy.

**Primary reference (Svelte concept, authoritative for visual intent):**
```
ui/templates/dev_core_ui/svelte/app_client.svelte
ui/templates/dev_core_ui/svelte/lib/Button.svelte
ui/templates/dev_core_ui/svelte/app.css
ui/templates/dev_core_ui/svelte/main.js
```

**Secondary reference (flat HTML, cross-check for visual consistency):**
```
ui/templates/dev_core_ui/xgen-client-core-test-ui.html
ui/xgen-client-core-test-ui.html
```

**CSS token set** — `app.css` defines the canonical token set for the Core Test UI. The amber primary palette (`--pr`, `--pr2`, `--pr-ink`) identifies the client binary. Do not change it.

---

## Architecture Constraints — Non-Negotiable

These rules apply before any other implementation decision:

**Library-first.** All lifecycle state machine logic lives in `xgen-client/src/lib.rs`. The Tauri `main.rs` is a thin shell — argument parsing, window creation, event wiring. No business logic in `main.rs`. No protocol logic in Svelte.

**State machine in lib.rs.** The `ClientLifecycleState` enum and all transition logic are implemented in `xgen-client/src/lib.rs`. The Tauri backend calls into the library; it does not define state transitions itself.

**Svelte calls Tauri commands only.** The frontend has no direct access to the filesystem, state files, or protocol state. All data flows through Tauri commands (`invoke`) and Tauri events (`listen`).

**Spec is authoritative.** State names, display labels, transition rules, and severity ordering are defined in `docs/xgen_appendix_e_en.md` — Appendix E, section E.2. Implement exactly what is specified there. Do not add states, rename states, or alter transition rules.

**DECISIONS.md before advancing.** Any implementation choice beyond spec prescription must be recorded in `DECISIONS.md` before proceeding to the next milestone. D-042 is already recorded — read it.

---

## Milestone 1 — Tauri Scaffold

**Goal:** `xgen-client` opens a window. The core test UI renders inside it. Nothing is wired yet — state is static placeholder.

### Tasks

**1.1 — Tauri project setup** ✅ Done

Set up a Tauri project for `xgen-client`. The Tauri crate integrates with the existing `xgen-client` Cargo workspace crate — do not create a parallel Rust project. The Tauri `main.rs` wraps `xgen-client/src/lib.rs`.

Svelte + Vite is the frontend build system per the reference files in `ui/templates/dev_core_ui/svelte/`. Wire Vite to build the Svelte frontend and Tauri to load it.

**1.2 — Window: no native titlebar** ✅ Done

The window must use Option 2 custom chrome (specified in CLAUDE.md Phase 2 Track 1 step 1): no native titlebar, application icon + name only. The window is not resizable at this stage.

**1.3 — Render the core test UI** ✅ Done

The Svelte root for `xgen-client` is `app_client.svelte`. The rendered UI must match the visual reference:

- Logo: `logo_client_64.png`, 48 × 48 px, centred
- State indicator placeholder: a static dot (grey) + text `"Initialising"` — will be wired in Milestone 3
- Button: label `"Quit"`, amber primary style (`--pr` palette)
- Container: `#core-ui-pane`, centred, 320 px wide, dark surface `--s2`, `1px solid --s5` border, `--rad` border-radius
- Font: XGen UI Sans (Inter-Regular.woff2 from the assets folder), 12 px body

**1.4 — `--instance` flag and data directory** 🆕 New — implement this before proceeding to Milestone 3

`xgen-client` must support running as multiple named instances simultaneously — required for multi-client stress testing and scripted test scenarios.

Parse `--instance <label>` from command-line args **before** the Tauri builder runs. The label is an arbitrary string (e.g. `alice`, `bot_1`). Derive all data paths from it:

```rust
let data_dir = match instance_label {
    Some(label) => std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("instances")
        .join(&label),
    None => exe_dir(), // default — backward compatible
};
std::fs::create_dir_all(&data_dir).expect("Failed to create instance data directory");
```

All data paths — keypair file, config file, log directory — must be derived from `data_dir`, not from `exe_dir()` directly. The `instances/` subdirectory is created automatically on first run.

When no `--instance` flag is given, behaviour is unchanged from the current default (data files in the executable's directory). This keeps single-instance usage backward compatible.

The named pipe for single-instance detection and `--batch` command delivery is **not** part of this milestone — it is implemented in `BATCH_FLAG_ph2.md`. For now, implement the flag parsing and data directory derivation only.

**1.5 — Verify** ✅ Items 1–5 done · ⏳ Items 6–7 pending (require Task 1.4)

- Window opens on `xgen-client` launch
- Logo, button, placeholder state indicator render correctly
- Quit button closes the window and terminates the process cleanly
- No native titlebar visible
- No console errors
- `xgen-client --instance alice` creates `instances/alice/` in the executable directory and writes all data files there
- `xgen-client --instance bob` creates `instances/bob/` independently — both instances run simultaneously without conflict

---

## Milestone 2 — Lifecycle State Machine in Rust ✅ Done

**Goal:** The 11 Client lifecycle states from Appendix E are implemented in `xgen-client/src/lib.rs`. State transitions emit Tauri events. No UI wiring yet — verify via log output only.

### Tasks

**2.1 — ClientLifecycleState enum**

Add to `xgen-client/src/lib.rs`:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientLifecycleState {
    Setup,
    Initialising,
    Connecting,
    Authenticating,
    Ready,
    DegradedAuth,
    DegradedFederation,
    DegradedNode,
    Reconnecting,
    Disconnected,
    Closing,
}
```

Serialises to the canonical uppercase underscore form: `"SETUP"`, `"INITIALISING"`, etc.

**2.2 — Display label**

Implement `Display` for `ClientLifecycleState` returning the Appendix E display label (title case):

| State | Display label |
|---|---|
| `Setup` | Setting up |
| `Initialising` | Initialising |
| `Connecting` | Connecting |
| `Authenticating` | Authenticating |
| `Ready` | Ready |
| `DegradedAuth` | Auth degraded |
| `DegradedFederation` | Federation degraded |
| `DegradedNode` | Node degraded |
| `Reconnecting` | Reconnecting |
| `Disconnected` | Disconnected |
| `Closing` | Closing |

**2.3 — Tauri event payload**

Define a serialisable payload struct in `lib.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClientStateEvent {
    pub state: ClientLifecycleState,   // canonical enum value
    pub label: String,                 // display label from Appendix E
    pub timestamp: String,             // UTC RFC 3339 with milliseconds
}
```

**2.4 — State transition and emission**

Implement a `transition_state(new_state: ClientLifecycleState, app_handle: &tauri::AppHandle)` function (or equivalent mechanism consistent with the library-first rule). This function:

1. Updates the current state
2. Logs the transition at `INFO` level: `tracing::info!("lifecycle_state={}", new_state)`
3. Emits a Tauri event named `"xgen-client-state-changed"` with `ClientStateEvent` as payload

The `AppHandle` must be passed in from `main.rs` — the library does not hold a reference to Tauri internals.

**2.5 — Wire startup sequence**

Wire the startup state progression in `main.rs` / `lib.rs` startup code:

1. On process start: transition to `Initialising`
2. If no keypair or config exists (first run): transition to `Setup`
3. After config and keypair loaded: if `auto_connect_local = true`, begin silent local scan
4. On connection attempt: transition to `Connecting`
5. On WebSocket connected, handshake in progress: transition to `Authenticating`
6. On handshake success: transition to `Ready`
7. On connection failure: transition to `Disconnected`
8. On window close initiated: transition to `Closing`

Do not implement reconnect logic or degraded states at this milestone — those are Phase 2 protocol work. Implement the state enum and transitions for the happy path only. The remaining states must exist in the enum for completeness but are not triggered yet.

**2.6 — Verify**

Run `xgen-client` and confirm via the log file:

- `lifecycle_state=INITIALISING` appears immediately on startup
- `lifecycle_state=READY` appears after auto-connect succeeds (requires a running `xgen-node`)
- `lifecycle_state=DISCONNECTED` appears when no local Node is found (auto-connect timeout)
- `lifecycle_state=CLOSING` appears on window close
- No state transitions appear in the log that are not listed above

---

## Milestone 3 — State Indicator Wired to UI ⏳ Not started

**Goal:** The Svelte frontend listens for `"xgen-client-state-changed"` events and updates the state indicator in real time.

### Tasks

**3.1 — State indicator component**

The state indicator renders inside `#core-ui-pane`, between the logo and the Quit button:

```
[ logo ]
  ● Ready          ← state dot + label
[ Quit ]
```

The indicator consists of:
- A status dot: 8 px circle, coloured by state severity (see colour mapping below)
- The Appendix E display label in `--t2` text colour
- Displayed inline, horizontally centred

**State dot colour mapping:**

| State(s) | Dot colour | Animation |
|---|---|---|
| `SETUP`, `CLOSING` | `--t4` (muted grey) | none |
| `INITIALISING` | `--t3` (grey) | slow pulse |
| `CONNECTING`, `AUTHENTICATING`, `RECONNECTING` | `--inf` (blue) | slow pulse |
| `READY` | green — add token `--ok: #2d7a3a` | none |
| `DEGRADED_AUTH`, `DEGRADED_FEDERATION`, `DEGRADED_NODE` | `--pr` (amber) | none |
| `DISCONNECTED` | red — add token `--err: #8a2a2a` | none |

Add `--ok` and `--err` to the `:root` token set in `app.css`. Do not use ad-hoc hex values in component styles — only tokens.

**3.2 — Event listener in Svelte**

In `app_client.svelte`, listen for `"xgen-client-state-changed"` via Tauri's event API (`@tauri-apps/api/event`). On every event:

- Update the reactive state variable
- The dot colour and label update automatically via reactive binding

Initial state before the first event is received: `INITIALISING` dot (grey pulsing), label `"Initialising"`.

**3.3 — Verify**

With a running `xgen-node` on `ws://127.0.0.1:8080/xgen`:

1. Launch `xgen-client` — indicator shows `Initialising` (grey pulsing)
2. Auto-connect succeeds — indicator transitions to `Ready` (green dot, no animation)
3. Click Quit — indicator briefly shows `Closing` before window closes

Without a running Node:

1. Launch `xgen-client` — indicator shows `Initialising`, then `Disconnected` (red, no animation) after 2-second timeout

---

## Milestone 4 — Verification ⏳ Not started

Full manual walkthrough. Do not mark this milestone complete until all items pass.

**Checklist:**

- [ ] Window opens without native titlebar
- [ ] Logo renders at correct size and position
- [ ] State indicator visible between logo and Quit button
- [ ] `INITIALISING` state shown on launch (grey pulsing dot)
- [ ] `READY` state shown after successful auto-connect (green dot)
- [ ] `DISCONNECTED` state shown when no Node available (red dot, after 2 s timeout)
- [ ] `CLOSING` state shown briefly on Quit
- [ ] Each state transition appears in the client log file at `INFO` level
- [ ] Tauri event `"xgen-client-state-changed"` payload contains correct `state`, `label`, and `timestamp` fields (verify via browser dev tools or log)
- [ ] Quit button terminates the process cleanly — log session footer (`=== XGEN SESSION END ===`) is written
- [ ] No native titlebar visible
- [ ] No console errors in browser dev tools
- [ ] `xgen-client --instance alice` creates `instances/alice/` and writes all data files there
- [ ] Two instances with different labels run simultaneously without conflict
- [ ] No `--instance` flag — default behaviour unchanged, data files in executable directory
- [ ] `cargo test` — 173/173 tests still passing after all changes
- [ ] Clean compile, no warnings

**Record results** in a brief note appended to this document under `## Verification Results`.

---

## Files to Produce

| File | Description |
|---|---|
| `xgen-client/src-tauri/` | Tauri crate (or equivalent structure) |
| `xgen-client/src/lib.rs` | `ClientLifecycleState`, `ClientStateEvent`, `transition_state` added |
| `xgen-client/src/main.rs` | Tauri shell, startup sequence wired |
| `ui/dev_core_ui/client/` | Svelte frontend source for the client Core Test UI |
| `DECISIONS.md` | Any new decisions beyond D-042 recorded before advancing |
| `JOURNAL.md` | Entry added on completion |

The Svelte build output (dist) must not be committed to the repository. Add it to `.gitignore`.

---

## Related Documents

| Document | Purpose |
|---|---|
| `docs/xgen_appendix_e_en.md` | Authoritative lifecycle state definitions — E.2 Client states |
| `docs/xgen_ch6_client_design.md` | Full UI architecture (Phase 2 reference) |
| `DECISIONS.md` D-042 | Tauri event emission for lifecycle state changes |
| `CLAUDE.md` Phase 2 Track 1 | Phase 2 UI implementation order |
| `ui/templates/dev_core_ui/svelte/` | Joe's design concept — visual reference |
| `NODE_CORE_UI_ph2.md` | Node Core Test UI instruction (follows this one) |
