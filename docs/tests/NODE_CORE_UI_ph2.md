# XGen Node — Core Test UI
> **Status**: PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Prerequisite

**`CLIENT_CORE_UI_ph2.md` Milestones 1 and 2 must be done before starting this instruction.** This condition is already met. The Tauri scaffold, Svelte build pipeline, `--instance` flag, and lifecycle state pattern established for the client are the direct template for this work.

Both Core Test UIs are developed in parallel from this point. Milestones 3 and 4 of the client and Milestones 1–4 of the node proceed simultaneously — the goal is to bring both exes to the same verified state before the next phase begins.

---

## Purpose

This document is the implementation instruction for Mr. Code to produce the `xgen-node` Core Test UI — the first real Tauri window for the Node binary. The result is:

- A **systray icon** that reflects Node lifecycle state at all times
- A detachable **admin window** (the core test UI) that can be opened and closed without affecting the running Node process
- A **Shut Down** action in both the systray menu and the admin window
- A `--service` headless mode with no systray and no window

This mirrors the client Core Test UI in visual language but follows the Node's fundamentally different deployment model (process-centric, not window-centric).

---

## Key Architectural Difference from the Client

The client is **window-centric**: closing the window ends the session and terminates the process.

The Node is **process-centric**: the Node process owns its own lifecycle. The systray icon and admin window are observers of process state — they do not create it. Closing the admin window does **not** stop the Node. The Node continues running until a deliberate Shut Down action is taken or an OS signal is received.

This distinction must be implemented correctly. Read D-037 in `DECISIONS.md` before touching any Node window or systray code.

---

## Design Reference Files

The visual design must match Joe's reference files:

- **Graphical look**: colors, typography, spacing, border radius — as in the reference files below
- **UI texts**: app name, button label (`"Shut Down"`), state labels (from Appendix E display labels)
- **Logo**: use `ui/dev_core_ui/shared_assets/logo_node_64.png`

The HTML/CSS/Svelte structure may be changed freely. Mr. Code must verify that attribute names, CSS class names, and CSS IDs are aligned with spec conventions — flag and correct any discrepancy.

**Primary reference (Svelte concept, authoritative for visual intent):**
```
ui/templates/dev_core_ui/svelte/app_node.svelte
ui/templates/dev_core_ui/svelte/lib/Button.svelte
ui/templates/dev_core_ui/svelte/app.css
```

**Secondary reference (flat HTML):**
```
ui/templates/dev_core_ui/xgen-node-core-test-ui.html
ui/xgen-node-core-test-ui.html
```

**CSS token set** — `app.css` is the canonical token set. The blue info palette (`--inf`, `--inf2`, `--inf-ink`) identifies the Node binary. The amber palette (`--pr`) is for the client only — do not use it here.

---

## Architecture Constraints — Non-Negotiable

**Library-first.** All Node lifecycle state machine logic lives in `xgen-node/src/lib.rs`. The Tauri `main.rs` is a thin shell — startup, window/systray wiring, event plumbing. No business logic in `main.rs`. No protocol logic in Svelte.

**State machine in lib.rs.** The `NodeLifecycleState` enum and all transition logic are implemented in `xgen-node/src/lib.rs`. The Tauri backend calls into the library; it does not define state transitions itself.

**Degraded states stack.** The Node can be in multiple `DEGRADED_*` conditions simultaneously. The state machine must track a set of active degraded conditions, not a single state value. The UI shows the highest-severity active condition (severity order: DEGRADED_STORAGE > DEGRADED_AUTH > DEGRADED_FEDERATION). Both transitions are logged.

**Closing the admin window does not stop the Node.** The window close event must be intercepted and the window hidden, not the process terminated. Only an explicit Shut Down action (systray menu or button) terminates the process.

**Spec is authoritative.** State names, display labels, transition rules, severity ordering, and systray icon mapping are defined in `docs/xgen_appendix_e_en.md` — Appendix E, section E.1. Implement exactly what is specified there.

**DECISIONS.md before advancing.** Any implementation choice beyond spec prescription must be recorded in `DECISIONS.md` before proceeding to the next milestone.

---

## Milestone 1 — Tauri Scaffold (Node)

**Goal:** `xgen-node` shows a systray icon and opens an admin window. The core test UI renders in the admin window. Nothing is wired yet — state is static placeholder. Closing the window hides it; the process keeps running.

### Tasks

**1.1 — Tauri project setup**

Set up a Tauri project for `xgen-node`. The Tauri crate integrates with the existing `xgen-node` Cargo workspace crate — do not create a parallel Rust project. The Tauri `main.rs` wraps `xgen-node/src/lib.rs`.

Svelte + Vite is the frontend build system, consistent with the client. The node frontend lives in `ui/dev_core_ui/node/`.

**1.2 — Systray icon**

Implement a system tray icon using Tauri's tray API. Systray menu items at this milestone:

- **Open Admin Panel** — shows the admin window (creates it if not open, focuses it if already open)
- **Shut Down** — transitions to `CLOSING`, flushes, terminates the process

The initial icon at this milestone is a static grey icon (placeholder). The icon will update dynamically in Milestone 3. Provide the icon assets in at least 16×16 and 32×32 sizes.

**1.3 — Admin window**

The admin window renders the Node core test UI. It must use Option 2 custom chrome (no native titlebar), consistent with the client window.

Window close behaviour: intercept the close event and **hide the window** rather than destroying it. The process must remain running. The window can be re-shown via "Open Admin Panel" from the systray.

**1.4 — `--service` flag**

When `xgen-node` is launched with `--service`:
- No systray icon is created
- No admin window is opened
- The process runs headless
- All other behaviour (logging, state machine, protocol) is unchanged

**1.5 — `--instance` flag, `--port` flag, and data directory**

`xgen-node` must support running as multiple named instances simultaneously — required for multi-node stress testing (e.g. two federated nodes on different ports).

Parse `--instance <label>` and `--port <port>` from command-line args **before** the Tauri builder runs. Derive all data paths from the label:

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

All data paths — keypair file, config file, log directory, spaces directory — must be derived from `data_dir`.

**`--port` at first launch:** when `--port` is given and no config exists yet in `data_dir`, write a config with that port into the instance directory. On subsequent runs, the config is loaded from the instance directory automatically — `--port` need not be repeated. If `--port` is omitted and no config exists, default to `8080`.

```
xgen-node.exe --instance node_a --port 8080   ← first launch: creates instances/node_a/, writes config with port 8080
xgen-node.exe --instance node_a               ← subsequent launches: reads instances/node_a/config, uses port 8080
xgen-node.exe --instance node_b --port 8081   ← independent instance on 8081
```

When no `--instance` flag is given, behaviour is unchanged from the current default. This keeps single-instance usage backward compatible.

The named pipe for single-instance detection and `--batch` command delivery is **not** part of this milestone — it is implemented in `BATCH_FLAG_ph2.md`. For now, implement the flag parsing, data directory derivation, and `--port` config write only.

**1.6 — Render the core test UI**

The Svelte root for `xgen-node` is `app_node.svelte`. The rendered UI must match the visual reference:

- Logo: `logo_node_64.png`, 48 × 48 px, centred
- State indicator placeholder: a static dot (grey) + text `"Initialising"` — will be wired in Milestone 3
- Button: label `"Shut Down"`, blue primary style (`--inf` palette)
- Container: `#core-ui-pane`, centred, 320 px wide, dark surface `--s2`, `1px solid --s5` border, `--rad` border-radius
- Font: XGen UI Sans (Inter-Regular.woff2), 12 px body

**1.7 — Verify**

- `xgen-node` launches and a systray icon appears
- "Open Admin Panel" opens the admin window with correct visual
- Closing the admin window hides it — systray icon remains
- "Open Admin Panel" again shows the window
- "Shut Down" from systray terminates the process cleanly
- `xgen-node --service` starts without systray or window
- No native titlebar on the admin window
- No console errors
- `xgen-node --instance node_a --port 8080` creates `instances/node_a/` and starts on port 8080
- `xgen-node --instance node_b --port 8081` creates `instances/node_b/` and starts on port 8081 — both run simultaneously without conflict
- Subsequent launch of `xgen-node --instance node_a` (no `--port`) reads port from instance config and starts correctly
- No `--instance` flag — default behaviour unchanged

---

## Milestone 2 — Lifecycle State Machine in Rust

**Goal:** The 7 Node lifecycle states from Appendix E are implemented in `xgen-node/src/lib.rs`. State transitions emit Tauri events. Degraded state stacking is implemented. No UI wiring yet — verify via log output only.

### Tasks

**2.1 — NodeLifecycleState enum**

Add to `xgen-node/src/lib.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NodeLifecycleState {
    Initialising,
    Ready,
    DegradedFederation,
    DegradedStorage,
    DegradedAuth,
    Maintenance,
    Closing,
}
```

Serialises to the canonical uppercase underscore form: `"READY"`, `"DEGRADED_STORAGE"`, etc.

**2.2 — Display label**

Implement `Display` for `NodeLifecycleState` returning the Appendix E display label (title case):

| State | Display label |
|---|---|
| `Initialising` | Initialising |
| `Ready` | Ready |
| `DegradedFederation` | Federation degraded |
| `DegradedStorage` | Storage degraded |
| `DegradedAuth` | Auth module degraded |
| `Maintenance` | Maintenance |
| `Closing` | Closing |

**2.3 — Degraded state set**

The Node tracks a `HashSet<NodeLifecycleState>` of active degraded conditions alongside the primary state. Only the three `DEGRADED_*` variants are valid set members. Implement a `active_display_state()` method that returns:

- `Closing` if shutting down
- `Maintenance` if in maintenance
- The highest-severity active degraded state if any are present (DEGRADED_STORAGE > DEGRADED_AUTH > DEGRADED_FEDERATION)
- `Ready` if no degraded conditions active and not in maintenance
- `Initialising` during startup

This is what the UI and systray display. The full set is available for future admin dashboard detail views.

**2.4 — Tauri event payload**

Define in `lib.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeStateEvent {
    pub state: NodeLifecycleState,     // active display state (highest severity)
    pub label: String,                  // display label from Appendix E
    pub active_degraded: Vec<String>,   // all active degraded condition names, empty if none
    pub timestamp: String,              // UTC RFC 3339 with milliseconds
}
```

**2.5 — State transition and emission**

Implement `transition_state(new_state: NodeLifecycleState, app_handle: &tauri::AppHandle)` and `resolve_degraded(condition: NodeLifecycleState, app_handle: &tauri::AppHandle)` in `lib.rs`. Both:

1. Update the state / degraded set
2. Log the transition at `INFO` level: `tracing::info!("lifecycle_state={}", active_display_state())`
3. Emit `"xgen-node-state-changed"` with `NodeStateEvent` as payload

The `AppHandle` is passed from `main.rs`. In `--service` mode the handle is absent — emit should be skipped gracefully (the systray/window do not exist).

**2.6 — Wire startup sequence**

Wire the startup state progression in `main.rs` / `lib.rs`:

1. On process start: transition to `Initialising`
2. After config, keypair, DAG stores, identity registry loaded successfully: transition to `Ready`
3. On init failure: log error, exit process
4. On Shut Down action: transition to `Closing`, flush, exit

Do not implement degraded states or MAINTENANCE at this milestone — those are Phase 2 protocol work. The enum must contain them for completeness but they are not triggered yet.

**2.7 — Verify**

Run `xgen-node` and confirm via log file:

- `lifecycle_state=INITIALISING` appears immediately on startup
- `lifecycle_state=READY` appears after successful startup
- `lifecycle_state=CLOSING` appears on Shut Down
- `xgen-node --service` produces identical log output with no Tauri-related errors

---

## Milestone 3 — State Indicator and Systray Icon Wired

**Goal:** The admin window and systray icon both reflect the current Node lifecycle state in real time.

### Tasks

**3.1 — State indicator in the admin window**

Identical pattern to the client: dot + label, between logo and Shut Down button, horizontally centred. Use the same dot colour token system established for the client, plus the Node-specific mappings:

| State(s) | Dot colour | Animation |
|---|---|---|
| `INITIALISING`, `CLOSING` | `--t3` (grey) | slow pulse on INITIALISING |
| `READY` | `--ok` (green) | none |
| `DEGRADED_STORAGE` | `--err` (red) | none |
| `DEGRADED_AUTH`, `DEGRADED_FEDERATION` | `--pr` (amber) | none |
| `MAINTENANCE` | `--inf` (blue) | none |

The dot colour follows `active_display_state()` — the highest-severity active condition.

**3.2 — Systray icon update**

Update the systray icon dynamically based on `active_display_state()`, per the Appendix E systray icon mapping:

| Condition | Icon | Tooltip |
|---|---|---|
| `INITIALISING` | Grey, animated | Initialising… |
| `READY` | Green | Ready |
| Any `DEGRADED_*` | Amber | Degraded — click for details |
| `MAINTENANCE` | Blue | Maintenance mode |
| `CLOSING` | Grey | Shutting down… |

Provide icon assets for each colour/state. Animated icons (INITIALISING) can use a simple frame sequence if Tauri's tray API supports it, or a static grey icon as a fallback.

**3.3 — Svelte event listener**

In `app_node.svelte`, listen for `"xgen-node-state-changed"` via Tauri's event API. On every event, update the reactive state variable. Initial state before the first event: `INITIALISING` (grey pulsing dot).

**3.4 — Verify**

With no special conditions:

1. Launch `xgen-node` — systray shows grey animated icon, admin window shows `Initialising`
2. Startup completes — systray turns green, admin window shows `Ready`
3. Shut Down from systray — systray shows grey, admin window shows `Closing`, process exits

---

## Milestone 4 — Verification

Full manual walkthrough. Do not mark this milestone complete until all items pass.

**Checklist:**

- [ ] Systray icon appears on launch
- [ ] Admin window opens via "Open Admin Panel" systray menu item
- [ ] Closing admin window hides it — process continues running
- [ ] Re-opening via systray works
- [ ] No native titlebar on admin window
- [ ] Logo, button, state indicator render correctly
- [ ] `INITIALISING` state shown on launch (grey pulsing dot, grey systray)
- [ ] `READY` state shown after startup (green dot, green systray)
- [ ] `CLOSING` state shown briefly on Shut Down (grey dot, grey systray)
- [ ] Shut Down terminates process cleanly — log session footer written
- [ ] `xgen-node --service` starts headless — no systray, no window, correct log output
- [ ] Each state transition appears in the node log at `INFO` level
- [ ] `"xgen-node-state-changed"` event payload contains correct `state`, `label`, `active_degraded`, and `timestamp` fields
- [ ] `xgen-node --instance node_a --port 8080` creates `instances/node_a/` and starts on port 8080
- [ ] `xgen-node --instance node_b --port 8081` creates `instances/node_b/` and starts on port 8081 — both run simultaneously without conflict
- [ ] Subsequent `xgen-node --instance node_a` (no `--port`) reads port from instance config
- [ ] No `--instance` flag — default behaviour unchanged
- [ ] `cargo test` — 173/173 tests still passing
- [ ] Clean compile, no warnings

**Record results** in a brief note appended to this document under `## Verification Results`.

---

## Files to Produce

| File | Description |
|---|---|
| `xgen-node/src-tauri/` | Tauri crate for the Node |
| `xgen-node/src/lib.rs` | `NodeLifecycleState`, `NodeStateEvent`, degraded set, `transition_state`, `resolve_degraded` added |
| `xgen-node/src/main.rs` | Tauri shell, systray, window hide-on-close, `--service` flag, startup sequence |
| `ui/dev_core_ui/node/` | Svelte frontend source for the Node Core Test UI |
| `DECISIONS.md` | Any new decisions recorded before advancing |
| `JOURNAL.md` | Entry added on completion |

The Svelte build output (dist) must not be committed. Add to `.gitignore` if not already present.

---

## Related Documents

| Document | Purpose |
|---|---|
| `docs/xgen_appendix_e_en.md` | Authoritative lifecycle state definitions — E.1 Node states |
| `DECISIONS.md` D-037 | Node deployment model — systray singleton, detachable window, `--service` flag |
| `DECISIONS.md` D-042 | Tauri event emission pattern (established for client, same pattern here) |
| `docs/xgen_ch6_client_design.md` | Full UI architecture (Phase 2 reference) |
| `CLIENT_CORE_UI_ph2.md` | Client Core Test UI — prerequisite, establishes the scaffold and pattern |
