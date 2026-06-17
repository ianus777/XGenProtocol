# XGen Protocol — Application Lifecycle States

> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Overview

This document defines the named lifecycle states for both XGen applications: the Client (`xgen-client.exe`) and the Node (`xgen-node.exe`). These states are the authoritative vocabulary for:

- The Console status indicator in both applications
- The systray icon health signal in `xgen-node.exe`
- Phase 2 testing scripts and test harness assertions
- Claude Code implementation of state machine logic
- Design Claude's skeleton UI status indicators

**Relationship to Ch6:** this document is a prerequisite for the Ch6 second pass. Section 6.7 (Protocol Implications) and the Console screen specification will reference these states directly. Nothing in this document overrides Ch6 — it extends it.

**Relationship to D-037b:** the Node deployment model (systray singleton, detachable admin window, headless service mode) is defined in D-037b. This document defines what states the Node process moves through regardless of which deployment mode is active.

---

## Design principles

**States are process-level for the Node, session-level for the Client.**

The Node process owns its own lifecycle. The Node's states reflect what the process is doing — they exist whether or not any UI window is open. The systray icon and admin window *observe and display* these states; they do not create them.

The Client has no persistent process between invocations. The Console window *is* the lifecycle host — opening the window starts the session, closing it ends it. The Client's states are therefore session-scoped: they begin at window open and terminate at window close.

**States name categories, logs explain causes.**

A degraded state tells the operator *what area* is affected. The reason — the specific error, the failing component, the underlying cause — belongs in the log and in the status indicator tooltip. This keeps the state set stable as the system grows: new failure causes within a category do not require new states.

**Every failure mode is named.**

No grey zones. If something can go wrong and an operator needs to know about it, it has a state. The cost of naming a state is near zero; the cost of an unnamed failure mode is a debugging session without vocabulary.

---

## Client lifecycle states

The Client session arc: window opens → config loads → connects to Node → authenticates → operates → optionally degrades → disconnects → window closes.

| State | Label | Meaning |
|---|---|---|
| `SETUP` | Setting up | First run only. No keypair or config exists yet. Guided initialisation: auto-discover local Node, collect display name and passphrase, generate keypair on user confirmation. Logged from session start. |
| `INITIALISING` | Initialising | Config and keypair loading. Runs on every subsequent start after first run. Brief — typically sub-second. |
| `CONNECTING` | Connecting | WebSocket connection attempt to the configured Node endpoint in progress. |
| `AUTHENTICATING` | Authenticating | WebSocket connected. Identity handshake with the Node in progress — presenting keypair, awaiting Node acceptance. |
| `READY` | Ready | Fully operational. Can join Spaces, send and receive Events, participate in Rooms. |
| `DEGRADED_AUTH` | Auth degraded | Trust assertion expiring, expired, or renewal failed. The client can still connect and operate but identity verification is at risk. Operator action required. |
| `DEGRADED_FEDERATION` | Federation degraded | The connected Node's federation links are down or unhealthy. The client itself is fine; cross-Node functionality is impaired. |
| `DEGRADED_NODE` | Node degraded | The connected Node is reporting internal issues — high load, storage pressure, or other self-reported health problems. |
| `RECONNECTING` | Reconnecting | Connection lost. Automatic reconnection in progress (only if `auto_reconnect: true` in client config). Uses exponential backoff up to configured max retries. |
| `DISCONNECTED` | Disconnected | No active connection. Either deliberate (user action), auto-reconnect exhausted, or `auto_reconnect: false`. Manual action required to reconnect. |
| `CLOSING` | Closing | Window close initiated. Clean shutdown in progress: flushing pending Events, archiving session log. Transitions to session end on completion. |

### Client state transitions

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
                                                   │
                                          max retries exhausted
                                                   │
                                                   ▼
                                             DISCONNECTED

  Any state ──► CLOSING ──► [session end, window closes]
```

### Auto-reconnect configuration

Auto-reconnect behaviour is user-configurable in `client_config.json`:

```toml
[connection]
auto_reconnect = true               # true = attempt reconnect; false = go straight to DISCONNECTED
reconnect_max_retries = 5           # number of attempts before giving up
reconnect_backoff_base_ms = 1000    # initial wait between attempts (doubles each retry)
reconnect_backoff_max_ms = 30000    # ceiling on backoff interval
```

Default: `auto_reconnect = true`, 5 retries, 1s base backoff, 30s ceiling.

---

## Node lifecycle states

The Node process arc: process starts → loads config → becomes ready → operates indefinitely → optionally degrades → operator shuts down cleanly.

The Node does not have `SETUP`, `CONNECTING`, or `AUTHENTICATING` states — first-run initialisation is handled by the `init` command (Phase 1, D-028), and the Node does not connect to other Nodes as a client: it accepts connections and establishes federation peer relationships as a peer, not as a dependent.

| State | Label | Meaning |
|---|---|---|
| `INITIALISING` | Initialising | Process started. Loading config, keypair, Space DAG stores, identity registry. Pre-flight checks running. |
| `READY` | Ready | Fully operational. Accepting client connections, processing Events, federation active. |
| `DEGRADED_FEDERATION` | Federation degraded | One or more federation peer links are down or unhealthy. The Node itself is operating normally; cross-Node Event delivery is impaired. |
| `DEGRADED_STORAGE` | Storage degraded | Data persistence is at risk. Causes include: disk full, disk I/O errors, SQLite lock contention (cf. Google Drive locking issue — Phase 1), DAG integrity check failure, `spaces_dir` becoming unavailable (e.g. unmounted network share). The specific cause is in the log. |
| `DEGRADED_AUTH` | Auth module degraded | The configured Auth Module is unreachable, timing out, or returning errors. New identity registrations and trust assertion renewals are failing. Existing authenticated sessions are unaffected. |
| `MAINTENANCE` | Maintenance | Operator-initiated state. The Node is running and processing existing sessions but is refusing new incoming connections. Used for upgrades, backups, and configuration changes that require a quiet period. Cannot be entered automatically. |
| `CLOSING` | Closing | Operator-initiated or OS signal received. Clean shutdown in progress: draining active sessions, flushing DAG writes, writing session footer to log. |

### Node state transitions

```
  [process start] ──► INITIALISING ──► READY ◄──────────────────────────────────┐
                             │            │                                       │
                      init failed         ├──► DEGRADED_FEDERATION               │
                             │            ├──► DEGRADED_STORAGE           (condition resolved)
                             ▼            ├──► DEGRADED_AUTH                     │
                          [exit]          │              │                        │
                                          │              └────────────────────────┘
                                          │
                                          ├──► MAINTENANCE ──► READY (operator lifts)
                                          │
                                          └──► CLOSING ──► [process exit]
```

**Degraded states are non-terminal.** A Node in any `DEGRADED_*` state continues operating. When the underlying condition resolves, the Node returns to `READY` automatically. The transition is logged in both directions.

**`MAINTENANCE` is operator-initiated only.** It cannot be entered automatically. It is exited by explicit operator action (via admin UI or CLI command), not automatically.

---

## Shared conventions

**State names are uppercase with underscores** — `READY`, `DEGRADED_AUTH`. This is the canonical form used in code, logs, and config.

**Display labels are title case** — "Ready", "Auth Degraded". These are what appear in the UI status indicator and systray tooltip.

**Degraded states stack.** A Node or Client can be in multiple degraded conditions simultaneously — e.g. `DEGRADED_FEDERATION` and `DEGRADED_STORAGE` at the same time. The UI status indicator shows the highest-severity active state; the full list of active degraded conditions is available in the console or log view.

**Severity order (highest first):**

| Priority | State |
|---|---|
| 1 (highest) | `DEGRADED_STORAGE` (Node) |
| 2 | `DEGRADED_AUTH` |
| 3 | `DEGRADED_FEDERATION` |
| 4 | `DEGRADED_NODE` (Client) |

**`CLOSING` is always reachable** from any state. A process or session can be shut down at any point regardless of current state.

---

## Systray icon mapping (Node, desktop deployment)

| Condition | Icon colour | Tooltip |
|---|---|---|
| `INITIALISING` | Grey, animated | Initialising… |
| `READY` | Green | Ready |
| Any `DEGRADED_*` | Amber | Degraded — click for details |
| `MAINTENANCE` | Blue | Maintenance mode |
| `CLOSING` | Grey | Shutting down… |

---

## Document status and next steps

This document is the input the design Claude is waiting for (referenced in `ui/docs/xgan-ui-debug-console-questions.md`, Q7). Once this document is stable:

1. Design Claude binds the Console skeleton's lifecycle status indicator to these state names
2. Ch6 second pass references this document in the Console screen specification
3. Claude Code implements the state machine in `xgen-client/src/lib.rs` and `xgen-node/src/lib.rs` during Phase 2

---

## Session log

### Session 1 — May 2026 (JozefN + Documentation Claude)
Defined full client and Node lifecycle state sets from first principles. Key decisions: `SETUP` is a formal top-level client state (not a pre-lifecycle screen) so first-run events are session-logged from window open; `DEGRADED` is split into named categories (`DEGRADED_AUTH`, `DEGRADED_FEDERATION`, `DEGRADED_NODE` for client; `DEGRADED_AUTH`, `DEGRADED_FEDERATION`, `DEGRADED_STORAGE` for Node) for diagnostic resolution; auto-reconnect is a user-configurable setting with configurable backoff, not fixed behaviour; `MAINTENANCE` added as a Node-only operator-initiated state. Node has 7 states; Client has 11 states.
