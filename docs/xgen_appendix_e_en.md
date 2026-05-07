# XGen Protocol — Appendix E: Application Lifecycle States

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

This appendix is the exhaustive reference for the named lifecycle states of both XGen applications: `xgennode.exe` and `xgenclient.exe`. It defines the complete state sets, transition rules, configuration parameters, systray behaviour, and design conventions used by both the UI implementation and the Phase 2 test harness.

**Canonical summary** is in Ch2 — Application Deployment Model & Lifecycle States. This appendix provides the full detail that Ch2 summarises. Nothing here overrides Ch2 or Ch6 — it extends them.

**Consumers of this document:**
- Design Claude — binding the Console skeleton's lifecycle status indicator to these state names
- Claude Code — implementing the state machine in `xgen-client/src/lib.rs` and `xgen-node/src/lib.rs`
- Ch6 second pass — Console screen specification references these states directly
- Phase 2 test harness — test scripts assert against these state names

---

## Design principles

**States are process-level for the Node, session-level for the Client.**

The Node process owns its own lifecycle. Its states exist whether or not any UI window is open. The systray icon and admin window observe and display these states; they do not create them.

The Client has no persistent process between invocations. The Console window is the lifecycle host — opening the window starts the session, closing it ends it. Client states are session-scoped: they begin at window open and terminate at window close.

**States name categories, logs explain causes.**

A degraded state identifies what area is affected. The specific cause — the failing component, the underlying error — belongs in the log and in the status indicator tooltip. This keeps the state set stable as the system grows: new failure causes within a category do not require new states.

**Every failure mode is named.**

No grey zones. If something can go wrong and an operator needs to know about it, it has a state. The cost of naming a state is near zero; the cost of an unnamed failure mode is a debugging session without vocabulary.

**State names are uppercase with underscores.** `READY`, `DEGRADED_AUTH`. This is the canonical form in code, logs, and config.

**Display labels are title case.** "Ready", "Auth Degraded". Used in UI status indicators and systray tooltips.

---

## E.1 — Node lifecycle states

The Node process arc: process starts → loads config → becomes ready → operates indefinitely → optionally degrades → operator shuts down cleanly.

The Node does not have `SETUP`, `CONNECTING`, or `AUTHENTICATING` states. First-run initialisation is handled by the `init` command (Phase 1, D-028). The Node does not connect to other Nodes as a client — it accepts connections and establishes federation as a peer.

### State definitions

| State | Display label | Meaning |
|---|---|---|
| `INITIALISING` | Initialising | Process started. Loading config, keypair, Space DAG stores, identity registry. Pre-flight checks running. |
| `READY` | Ready | Fully operational. Accepting client connections, processing Events, federation active. |
| `DEGRADED_FEDERATION` | Federation degraded | One or more federation peer links are down or unhealthy. The Node itself is operating normally; cross-Node Event delivery is impaired. |
| `DEGRADED_STORAGE` | Storage degraded | Data persistence is at risk. Causes include: disk full, disk I/O errors, SQLite lock contention (cf. Google Drive locking — Phase 1), DAG integrity check failure, `spaces_dir` becoming unavailable (e.g. unmounted network share). Specific cause is in the log. |
| `DEGRADED_AUTH` | Auth module degraded | The configured Auth Module is unreachable, timing out, or returning errors. New identity registrations and trust assertion renewals are failing. Existing authenticated sessions are unaffected. |
| `MAINTENANCE` | Maintenance | Operator-initiated. The Node is running and processing existing sessions but is refusing new incoming connections. Used for upgrades, backups, and configuration changes requiring a quiet period. |
| `CLOSING` | Closing | Operator-initiated or OS signal received. Clean shutdown in progress: draining active sessions, flushing DAG writes, writing session footer to log. |

### State transition diagram

```
  [process start] ──► INITIALISING ──► READY ◄──────────────────────────────────┐
                             │            │                                       │
                      init failed         ├──► DEGRADED_FEDERATION               │
                             │            ├──► DEGRADED_STORAGE        (condition resolved)
                             ▼            ├──► DEGRADED_AUTH                     │
                          [exit]          │              │                        │
                                          │              └────────────────────────┘
                                          │
                                          ├──► MAINTENANCE ──► READY (operator lifts)
                                          │
                                          └──► CLOSING ──► [process exit]
```

### Transition rules

**Degraded states are non-terminal.** A Node in any `DEGRADED_*` state continues operating. When the underlying condition resolves, the Node returns to `READY` automatically. Both the entry and exit transitions are written to the log.

**Degraded states stack.** A Node can be in multiple degraded conditions simultaneously. The status indicator shows the highest-severity active state; the full list of active degraded conditions is shown in the admin dashboard and console log view.

**`MAINTENANCE` is operator-initiated only.** It cannot be entered automatically. It is exited by explicit operator action via the admin UI or CLI command — not automatically.

**`CLOSING` is reachable from any state.** A process can be shut down regardless of current state.

### Severity order (highest first)

| Priority | State |
|---|---|
| 1 (highest) | `DEGRADED_STORAGE` |
| 2 | `DEGRADED_AUTH` |
| 3 | `DEGRADED_FEDERATION` |

### Systray icon mapping (desktop deployment)

| Condition | Icon colour | Tooltip |
|---|---|---|
| `INITIALISING` | Grey, animated | Initialising… |
| `READY` | Green | Ready |
| Any `DEGRADED_*` | Amber | Degraded — click for details |
| `MAINTENANCE` | Blue | Maintenance mode |
| `CLOSING` | Grey | Shutting down… |

---

## E.2 — Client lifecycle states

The Client session arc: window opens → config loads → connects to Node → authenticates → operates → optionally degrades → disconnects → window closes.

### State definitions

| State | Display label | Meaning |
|---|---|---|
| `SETUP` | Setting up | First run only. No keypair or config exists. Guided initialisation: auto-discover local Node on default port, collect display name and passphrase, generate keypair on user confirmation. Logged from session start — this is a formal state, not a pre-lifecycle screen. |
| `INITIALISING` | Initialising | Config and keypair loading. Runs on every subsequent start after first run. Typically sub-second. |
| `CONNECTING` | Connecting | WebSocket connection attempt to the configured Node endpoint in progress. |
| `AUTHENTICATING` | Authenticating | WebSocket connected. Identity handshake with the Node in progress — presenting keypair, awaiting Node acceptance. |
| `READY` | Ready | Fully operational. Can join Spaces, send and receive Events, participate in Rooms. |
| `DEGRADED_AUTH` | Auth degraded | Trust assertion expiring, expired, or renewal failed. The client can still connect and operate but identity verification is at risk. User action required. |
| `DEGRADED_FEDERATION` | Federation degraded | The connected Node's federation links are down or unhealthy. The client itself is fine; cross-Node functionality is impaired. |
| `DEGRADED_NODE` | Node degraded | The connected Node is reporting internal issues — high load, storage pressure, or other self-reported health problems. |
| `RECONNECTING` | Reconnecting | Connection lost. Automatic reconnection in progress (only if `auto_reconnect: true` in client config). Uses exponential backoff up to configured max retries. |
| `DISCONNECTED` | Disconnected | No active connection. Cause: deliberate user action, auto-reconnect retries exhausted, or `auto_reconnect: false`. Manual action required to reconnect. |
| `CLOSING` | Closing | Window close initiated. Clean shutdown: flushing pending Events, archiving session log. Session ends on completion. |

### State transition diagram

```
                    ┌──────────────────────────────────────────┐
                    │                                          │
         first run  │                                          ▼
  [window open] ──► SETUP ──► INITIALISING ──► CONNECTING ──► AUTHENTICATING ──► READY
                                   ▲                │                │               │
                              (subsequent           │ failed         │ failed        ├──► DEGRADED_AUTH
                               starts)              ▼                ▼               ├──► DEGRADED_FEDERATION
                                             DISCONNECTED     DISCONNECTED           └──► DEGRADED_NODE
                                                   │                                         │
                                          auto_reconnect=true                                │
                                                   │◄────────────────────────────────────────┘
                                                   ▼
                                            RECONNECTING
                                            │         │
                                   success  │         │  max retries exhausted
                                            ▼         ▼
                                       CONNECTING  DISCONNECTED

  Any state ──► CLOSING ──► [session end, window closes]
```

### Transition rules

**`SETUP` is a formal top-level state.** It is not a pre-lifecycle screen. The Console session and its log begin at window open, before keypair generation. This ensures first-run events are captured.

**`SETUP` runs once.** After a keypair and config exist, subsequent starts go directly to `INITIALISING`.

**Degraded states are non-terminal.** The client continues operating in all three `DEGRADED_*` states. When the underlying condition resolves, the client returns to `READY` automatically. Both transitions are logged.

**Degraded states stack.** The status indicator shows the highest-severity active state; the full list is visible in the console log.

**`RECONNECTING` leads back to `CONNECTING`** on each attempt, not directly to `READY`. The full handshake runs again after each reconnection.

**`CLOSING` is reachable from any state.**

### Severity order (highest first)

| Priority | State |
|---|---|
| 1 (highest) | `DEGRADED_AUTH` |
| 2 | `DEGRADED_FEDERATION` |
| 3 | `DEGRADED_NODE` |

### Auto-reconnect configuration

Auto-reconnect behaviour is user-configurable in `client_config.toml`:

```toml
[connection]
auto_reconnect = true               # true = attempt reconnect on drop; false = go to DISCONNECTED immediately
reconnect_max_retries = 5           # number of attempts before giving up (0 = unlimited)
reconnect_backoff_base_ms = 1000    # initial wait between attempts in milliseconds
reconnect_backoff_max_ms = 30000    # ceiling on backoff interval in milliseconds
```

**Defaults:** `auto_reconnect = true`, 5 retries, 1 000 ms base, 30 000 ms ceiling.

**Backoff formula:** `min(base_ms * 2^attempt, max_ms)`. After 5 retries with default settings: 1s, 2s, 4s, 8s, 16s — total wait approximately 31 seconds before giving up.

---

## E.3 — Shared conventions

**Logging.** Every state transition is written to the application log as a structured log line at `INFO` level. Format follows Appendix G log line convention. Example:

```
timestamp=2026-05-07T10:24:31Z level=INFO subsystem=lifecycle action=state_transition previous=CONNECTING current=AUTHENTICATING
```

**Console status indicator.** The Console chrome displays the current state as a labelled indicator using the display label from the tables above. When multiple degraded states are active, the highest-severity state is shown with a count badge (e.g. "Federation degraded +1"). The full list of active degraded states is accessible by clicking the indicator.

**Test harness.** Phase 2 test scripts assert against state names in canonical uppercase form. Example assertion: `assert_client_state(READY)`. State names must not change without a versioned migration in the test harness.

**State names are stable identifiers.** Once defined in this document, a state name is a public interface. Renaming requires a deprecation cycle.

---

## E.4 — Relationship to other documents

| Document | Relationship |
|---|---|
| Ch2 — Application Deployment Model & Lifecycle States | Canonical summary. This appendix is the full detail. |
| Ch6 — Client Design | Console screen specification (second pass) will reference these states for the status indicator component. |
| D-037 | Node deployment model decision record. Desktop vs. service deployment. Architectural horizon. |
| `ui/docs/xgan-ui-debug-console-questions.md` | Design Claude's Q7 — the lifecycle proposal this document fulfils. |
| `docs/xgen_lifecycle_states.md` | Superseded working draft. This appendix replaces it. The working draft may be deleted once Appendix E is confirmed stable. |
| Appendix G | Log line convention used for state transition log entries. |

---

## Session log

### Session 1 — May 2026 (JozefN + Documentation Claude)
Appendix E written as the exhaustive lifecycle reference, superseding the working draft `docs/xgen_lifecycle_states.md`. Fulfils the lifecycle proposal document that design Claude was waiting for (Q7 in `ui/docs/xgan-ui-debug-console-questions.md`). Full state definitions, transition diagrams, transition rules, severity ordering, auto-reconnect configuration reference, systray icon mapping, shared logging and test harness conventions documented. Ch2 Session 18 entry added simultaneously.
