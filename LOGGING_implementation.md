# XGen Protocol — Logging Implementation Instructions
> Document type: Implementation instructions for Claude Code
> Applies to: `xgen-node` and `xgen-client` binaries
> Date: May 2026
> Prepared by: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> Decision record: D-033, D-032
> See also: `docs/xgen_appendix_g_en.md` — Appendix G: Log Line Convention (the format contract)
> See also: `docs/tests/LOGGING_debug_ph1.md` — Phase 1 debug log infrastructure
> See also: `docs/tests/LOGGING_debug_ph2.md` — Phase 2 global Event tracing interface (superseded by this document for implementation details)

---

## Purpose

This document is the definitive implementation guide for XGen debug logging. It supersedes the implementation detail sections of `docs/tests/LOGGING_debug_ph2.md`. The format contract is defined in Appendix G (`docs/xgen_appendix_g_en.md`) — this document tells Mr. Code how to implement it in Rust.

Phase 1 debug log infrastructure (datetime-stamped files, config level switch, subscriber init) is already implemented and must not be re-implemented. This document covers what is still missing.

---

## What Is Already Implemented

Do not re-implement these — they are complete and verified:

- Datetime-stamped log files in `logs/` subfolder, one per session
- `[logging].level` config switch in both binaries
- `XGEN_LOG` env var override
- `with_ansi(false)` file output
- Subscriber init in `main()` of both binaries
- `event_trace.rs` module in `xgen-common/src/` with `trace_event()`, `EventDirection { Inbound, Outbound }`, `SessionContext`, `SpaceRole`
- Role gate: output suppressed unless Owner or Admin is authenticated
- Content field never logged

---

## What Needs to Change

Three things require implementation or update:

1. **`EventDirection` rename** — `Inbound`/`Outbound` → `In`/`Out`, with `Display` producing `IN`/`OUT`. Add `Local` variant producing `LOCAL`.
2. **`action` field** — mandatory on every Event log line. Add to `trace_event()` output and introduce a companion `trace_local()` for LOCAL actions.
3. **Session header and footer** — written to the log file on startup and clean shutdown respectively.

---

## Step 1 — Update `event_trace.rs`

File: `xgen-common/src/event_trace.rs`

### 1a — Update EventDirection

```rust
pub enum EventDirection {
    In,    // Event arriving at this binary from the network
    Out,   // Event leaving this binary to the network
    Local, // Action occurring entirely within this binary — no network crossing
}

impl fmt::Display for EventDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::In    => write!(f, "IN"),
            Self::Out   => write!(f, "OUT"),
            Self::Local => write!(f, "LOCAL"),
        }
    }
}
```

### 1b — Update `trace_event()` — add `action` field

The `action` field is mandatory on every log line per Appendix G. For network Events, action is always `receive_event` (IN) or `send_event` (OUT).

```rust
pub fn trace_event(event: &Event, direction: EventDirection, session: &SessionContext) {
    let role_permits = matches!(
        session.role,
        Some(SpaceRole::Owner) | Some(SpaceRole::Admin)
    );
    if !role_permits {
        return;
    }

    let action = match direction {
        EventDirection::In  => "receive_event",
        EventDirection::Out => "send_event",
        EventDirection::Local => {
            // Local actions must use trace_local() — not this function
            tracing::warn!("trace_event called with Local direction — use trace_local() instead");
            return;
        }
    };

    let event_id = event.event_id.as_deref().unwrap_or("(none)");
    tracing::debug!(
        direction  = %direction,
        action     = %action,
        event_id   = %event_id,
        event_type = %event.event_type,
        sender     = %event.sender,
        space_id   = %event.space_id,
        room_id    = %event.room_id,
        timestamp  = %event.timestamp,
        "Event"
    );
}
```

### 1c — Add `trace_local()`

LOCAL actions do not cross the network and do not carry a full Event context in all cases. They use a dedicated function with a controlled action enum.

```rust
/// Valid LOCAL action values per Appendix G action registry.
pub enum LocalAction {
    CreateEvent,
    StoreEvent,
    ApplyEvent,
    RejectEvent,
}

impl fmt::Display for LocalAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateEvent => write!(f, "create_event"),
            Self::StoreEvent  => write!(f, "store_event"),
            Self::ApplyEvent  => write!(f, "apply_event"),
            Self::RejectEvent => write!(f, "reject_event"),
        }
    }
}

/// Log a LOCAL action at the Event boundary.
///
/// Called for create_event, store_event, apply_event, reject_event.
/// These actions never cross the network — direction is always LOCAL.
/// The content field is never logged. No role gate — LOCAL actions
/// are always logged when the subscriber level permits.
pub fn trace_local(
    action: LocalAction,
    event_id: &str,
    event_type: Option<&str>,
    space_id: Option<&str>,
    error_code: Option<u32>,
) {
    tracing::debug!(
        direction  = "LOCAL",
        action     = %action,
        event_id   = %event_id,
        event_type = event_type.unwrap_or(""),
        space_id   = space_id.unwrap_or(""),
        error_code = error_code.map(|c| c.to_string()).unwrap_or_default(),
        "Event"
    );
}
```

**Note on role gate for LOCAL actions:** LOCAL actions are internal — they contain no sensitive user content and no identity information beyond the event_id. The role gate is therefore not applied to `trace_local`. An operator running at `debug` level sees all LOCAL actions regardless of who is authenticated.

---

## Step 2 — Session Header

The session header must be written to the log file immediately after the subscriber is initialised — before any other log output. It uses the format defined in Appendix G exactly.

Add a `write_session_header()` function in `event_trace.rs`:

```rust
/// Write the session header block to the log.
/// Must be called once, immediately after subscriber init, before any other logging.
/// `started_at` is RFC 3339 UTC.
pub fn write_session_header(
    app_type: &str,           // "node" or "client"
    self_id: &str,            // node_id (node) or identity_id (client)
    endpoint: Option<&str>,   // node listen address — None for client
    connected_node: Option<&str>, // node URL client connected to — None for node
    protocol_version: &str,
    build: &str,
    session_id: &str,
    started_at: &str,
) {
    // Header marker
    tracing::info!("=== XGEN SESSION START ===");
    tracing::info!("app_type={}", app_type);

    match app_type {
        "node"   => tracing::info!("node_id={}", self_id),
        "client" => tracing::info!("identity_id={}", self_id),
        _        => tracing::info!("id={}", self_id),
    }

    if let Some(ep) = endpoint {
        tracing::info!("endpoint={}", ep);
    }
    if let Some(cn) = connected_node {
        tracing::info!("connected_node={}", cn);
    }

    tracing::info!("protocol_version={}", protocol_version);
    tracing::info!("build={}", build);
    tracing::info!("session_id={}", session_id);
    tracing::info!("started_at={}", started_at);

    // Mandatory blank line — body start delimiter per Appendix G
    tracing::info!("");
}
```

**Where to call it — Node (`xgen-node/src/main.rs`):**

```rust
// After subscriber init, before run_node()
let started_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
let session_id = format!("{:08x}", rand::random::<u32>());
event_trace::write_session_header(
    "node",
    &node_id_uri,
    Some(&config.node.listen),
    None,
    "0.1",
    env!("CARGO_PKG_VERSION"),
    &session_id,
    &started_at,
);
```

**Where to call it — Client (`xgen-client/src/main.rs`):**

```rust
// After subscriber init, before any command execution
let started_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
let session_id = format!("{:08x}", rand::random::<u32>());
event_trace::write_session_header(
    "client",
    &identity_id_uri,
    None,
    Some(&node_url),
    "0.1",
    env!("CARGO_PKG_VERSION"),
    &session_id,
    &started_at,
);
```

---

## Step 3 — Session Footer

The session footer must be written on any clean exit. It uses the format defined in Appendix G exactly.

Add a `write_session_footer()` function in `event_trace.rs`:

```rust
/// Valid exit reason values per Appendix G.
pub enum ExitReason {
    Shutdown,
    Restart,
    Error,
}

impl fmt::Display for ExitReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shutdown => write!(f, "shutdown"),
            Self::Restart  => write!(f, "restart"),
            Self::Error    => write!(f, "error"),
        }
    }
}

/// Write the session footer block to the log.
/// Must be called on every clean exit path. Never called on crash or kill —
/// absence of footer is itself the signal of abnormal termination.
pub fn write_session_footer(reason: ExitReason) {
    let ended_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    // Mandatory blank line — body end delimiter per Appendix G
    tracing::info!("");
    tracing::info!("=== XGEN SESSION END ===");
    tracing::info!("ended_at={}", ended_at);
    tracing::info!("reason={}", reason);
}
```

**Where to call it:** at every clean exit point in both `main()` functions. This includes normal shutdown, graceful error exits, and restart paths. Do not call it on panic or from signal handlers — those are abnormal terminations.

```rust
// Normal shutdown
event_trace::write_session_footer(ExitReason::Shutdown);

// Fatal error caught and handled
tracing::error!(reason = %e, "Fatal error");
event_trace::write_session_footer(ExitReason::Error);
```

---

## Step 4 — Wire LOCAL actions

Add `trace_local()` calls at the four LOCAL action points in `xgen-node`. These are in addition to the existing `trace_event()` calls at the IN/OUT boundaries.

```rust
// After constructing an Event locally
event_trace::trace_local(LocalAction::CreateEvent, &event_id, Some(&event_type), Some(&space_id), None);

// After writing Event to DAG store
event_trace::trace_local(LocalAction::StoreEvent, &event_id, None, Some(&space_id), None);

// After applying Event to Space/Room state machine
event_trace::trace_local(LocalAction::ApplyEvent, &event_id, None, Some(&space_id), None);

// After rejecting Event in validation pipeline
event_trace::trace_local(LocalAction::RejectEvent, &event_id, Some(&event_type), Some(&space_id), Some(error_code));
```

---

## Step 5 — Update call sites for EventDirection rename

Find all existing call sites of `trace_event()` in both binaries and update the direction argument:

```rust
// Before
trace_event(&event, EventDirection::Inbound, &session);
trace_event(&event, EventDirection::Outbound, &session);

// After
trace_event(&event, EventDirection::In, &session);
trace_event(&event, EventDirection::Out, &session);
```

---

## Step 6 — Verify

**Test 1 — Header present:**
Start `xgen-node`. Open the log file. Confirm the first line is `=== XGEN SESSION START ===` followed by `key=value` fields and a blank line before the first body line.

**Test 2 — Footer on clean shutdown:**
Stop `xgen-node` with a normal shutdown signal. Confirm the log file ends with a blank line, `=== XGEN SESSION END ===`, `ended_at=...`, `reason=shutdown`.

**Test 3 — No footer on kill:**
Force-kill the process (`kill -9` / Task Manager). Confirm the log file ends mid-body with no footer marker.

**Test 4 — Direction values:**
Send a message as admin. Confirm log lines show `direction=OUT`, `direction=IN`, `direction=LOCAL` — not `Outbound`, `Inbound`.

**Test 5 — action field present:**
Confirm every body line contains `action=receive_event`, `action=send_event`, `action=store_event`, `action=apply_event`, `action=create_event`, or `action=reject_event` as appropriate.

**Test 6 — reject_event includes error_code:**
Trigger a validation failure. Confirm the `reject_event` line includes `error_code=<number>`.

**Test 7 — Pairing still works:**
Send a message as admin. Confirm `event_id` matches between client `direction=OUT action=send_event` line and node `direction=IN action=receive_event` line.

**Test 8 — content field never appears:**
Search log files for any message text. It MUST NOT appear at any log level.

---

## Files Modified

| File | Change |
|---|---|
| `xgen-common/src/event_trace.rs` | Update `EventDirection` (rename + add `Local`); update `trace_event()` to add `action` field; add `trace_local()`, `LocalAction`, `write_session_header()`, `write_session_footer()`, `ExitReason` |
| `xgen-node/src/main.rs` | Call `write_session_header()` after subscriber init; call `write_session_footer()` on all clean exit paths; update `EventDirection` call sites |
| `xgen-client/src/main.rs` | Same as node |
| `xgen-node/src/` (DAG/space modules) | Add `trace_local()` calls at create, store, apply, reject points |

---

## Format Reference

The exact log line format, field definitions, direction values, action registry, session structure, and parsing rules are defined in **Appendix G** (`docs/xgen_appendix_g_en.md`). This document implements that contract. If this document and Appendix G conflict, Appendix G wins.

---

*End of document*
