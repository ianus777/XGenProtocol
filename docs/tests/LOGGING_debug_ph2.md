# XGen Protocol — Debug Logging Phase 2: Global Event Tracing Interface
> Document type: Implementation instructions for Claude Code  
> Phase: Phase 2 — implement FIRST, before any Phase 2 protocol features  
> Applies to: `xgen-node` and `xgen-client` binaries  
> Date: April 2026  
> Prepared by: JozefN  
> Decision record: D-033  
> Supersedes: enumerated `tracing::` calls in `LOGGING_debug_ph1.md` as the primary mechanism  

---

## Why this exists

The Phase 1 logging implementation added `tracing::info!` calls one by one, per command handler. This is fragile:
- New commands added in Phase 2 will produce no log output unless someone remembers to add a call
- No guarantee that a client log entry and a Node log entry can be paired
- Sensitive conversations can leak if a future developer adds a log call on the `content` field

The correct solution is a **global Event tracing interface** — a single function that every inbound and outbound Event passes through automatically. Logging is not something that happens in individual handlers. It happens at the Event boundary, always, for every Event, with a role gate controlling visibility.

This is the **first implementation task of Phase 2**, before any Phase 2 protocol features are built.

---

## Architecture

### The single chokepoint

Every Event that enters or leaves the Node or client passes through one function:

```rust
pub fn trace_event(
    event: &XgenEvent,
    direction: EventDirection,
    session: &SessionContext,
)
```

This function is called in exactly two places per binary:
- **Node:** once in the inbound message handler (after deserialization, before validation), and once in the outbound send path (before serialization)
- **Client:** once in the outbound send path (before serialization), and once in the inbound receive path (after deserialization)

No other code calls `trace_event`. No handler, no command function, no protocol module adds its own Event log call. The global interface is the sole source of Event log entries.

### EventDirection

```rust
pub enum EventDirection {
    Inbound,   // Event arriving at this binary from the network
    Outbound,  // Event leaving this binary to the network
}
```

### SessionContext

```rust
pub struct SessionContext {
    pub identity_id: Option<String>,  // authenticated Identity, if any
    pub role: Option<SpaceRole>,      // their role in the relevant Space
    pub space_id: Option<String>,     // Space context if known
}

pub enum SpaceRole {
    Owner,
    Admin,
    Moderator,
    Member,
}
```

---

## Role gate — sensitive content protection

Debug log output for Events is suppressed unless the authenticated session holds **owner** or **admin** role in the relevant Space.

```rust
pub fn trace_event(
    event: &XgenEvent,
    direction: EventDirection,
    session: &SessionContext,
) {
    // Role gate — suppress for non-admin sessions
    let role_permits = matches!(
        session.role,
        Some(SpaceRole::Owner) | Some(SpaceRole::Admin)
    );
    if !role_permits {
        return;
    }

    // Safe fields only — content field is NEVER logged
    tracing::debug!(
        direction = %direction,
        event_id  = %event.event_id,
        event_type = %event.event_type,
        sender    = %event.sender,
        space_id  = %event.space_id,
        room_id   = %event.room_id,
        timestamp = %event.timestamp,
        "Event"
    );
}
```

**Rules:**
- `Owner` and `Admin` — debug output produced
- `Moderator` and `Member` — debug output suppressed, even if `level = "debug"` in config
- No authenticated session (unauthenticated transport connection) — suppressed
- The `content` field is **never** logged at any level by any code — not in `trace_event`, not anywhere

**Why moderator is excluded:** moderators can manage members but are not responsible for system-level operations. Admins and owners are the accountability boundary for system visibility.

---

## Pairing guarantee

Every Event carries a globally unique `event_id` (SHA-256 content hash). Because `trace_event` logs `event_id` on both the client (Outbound) and the Node (Inbound), any two log files from the same session can be joined by `event_id`:

```
# xgen-client log
2026-04-30 14:35:22.401 [DEBUG] xgen_client::transport: Event direction=Outbound event_id=xgen://hash/sha256:a3f9... event_type=message.text sender=xgen://pubkey/ed25519:AAAA... space_id=xgen://hash/sha256:b2c3... room_id=xgen://hash/sha256:c3d4... timestamp=2026-04-30T14:35:22.401Z

# xgen-node log — same event_id, direction=Inbound
2026-04-30 14:35:22.418 [DEBUG] xgen_node::transport::server: Event direction=Inbound event_id=xgen://hash/sha256:a3f9... event_type=message.text sender=xgen://pubkey/ed25519:AAAA... space_id=xgen://hash/sha256:b2c3... room_id=xgen://hash/sha256:c3d4... timestamp=2026-04-30T14:35:22.401Z
```

The 17ms difference is the network round trip. The `event_id` is the join key. No coordination needed between the two binaries — the content hash does the pairing automatically.

---

## Implementation steps

### Step 1 — Create the trace_event module

In `xgen-common/src/` (or `xgen-node/src/` if xgen-common is not yet the right home), create:

```
xgen-common/src/tracing/mod.rs
```

Containing:
- `EventDirection` enum (derive `Display`)
- `SessionContext` struct
- `SpaceRole` enum
- `trace_event()` function as specified above

Export from `xgen-common/src/lib.rs`:
```rust
pub mod tracing;
```

Both `xgen-node` and `xgen-client` depend on `xgen-common` — they both get `trace_event` from the same source. This guarantees the log format is identical between the two binaries.

### Step 2 — Wire into xgen-node

Find the two Event boundary points in `xgen-node`:

**Inbound boundary** — where a received WebSocket message is deserialized into an `XgenEvent`. This is the single point in `transport/server.rs` or `message/exchange.rs` where raw bytes become a typed Event. Add immediately after deserialization, before validation:

```rust
common::tracing::trace_event(&event, EventDirection::Inbound, &session_context);
```

**Outbound boundary** — where an `XgenEvent` is serialized and sent to a peer or client. Add immediately before serialization:

```rust
common::tracing::trace_event(&event, EventDirection::Outbound, &session_context);
```

These are the only two additions needed in `xgen-node`. No other files require changes for Event tracing.

### Step 3 — Wire into xgen-client

Same two boundary points in `xgen-client`:

**Outbound boundary** — where the client serializes an Event before sending. Single location in the transport send path.

**Inbound boundary** — where the client receives and deserializes a message from the Node.

```rust
common::tracing::trace_event(&event, EventDirection::Outbound, &session_context);
// ... serialize and send
```

```rust
// ... receive and deserialize
common::tracing::trace_event(&event, EventDirection::Inbound, &session_context);
```

### Step 4 — Build SessionContext at the connection level

The `SessionContext` must be constructed when authentication completes and passed through the connection's lifetime. It is not reconstructed per Event — it is established once per session and held for the session's duration.

In the Node's connection handler, after the challenge-response auth succeeds:

```rust
let session_context = SessionContext {
    identity_id: Some(authenticated_identity_id.clone()),
    role: space_registry.role_of(&authenticated_identity_id, &space_id),
    space_id: Some(space_id.clone()),
};
```

In the client, after authentication:

```rust
let session_context = SessionContext {
    identity_id: Some(my_identity_id.clone()),
    role: Some(my_role_in_space.clone()),
    space_id: Some(target_space_id.clone()),
};
```

### Step 5 — Remove enumerated Event log calls

Remove `tracing::info!` / `tracing::debug!` calls from individual command handlers that duplicate what `trace_event` now covers. Specifically — any call that logs an Event's fields (`event_id`, `event_type`, `sender`, etc.) in a command handler is now redundant and should be removed.

**Keep** the non-Event operational log calls: `Node started`, `Identity registered`, `Client authenticated`, `Client disconnected`, `Federation established`, `Node shutting down` — these are lifecycle events, not protocol Events, and they are not covered by `trace_event`.

### Step 6 — Verify

**Test 1 — Admin session produces output:**
```
set XGEN_LOG=debug
xgen-client send --space TestSpace --room RoomA --text "hello" (as admin)
```
Expected: `[DEBUG] ... Event direction=Outbound event_id=... event_type=message.text ...` appears in client log. Corresponding `direction=Inbound` line appears in node log with same `event_id`.

**Test 2 — Member session produces no Event output:**
```
set XGEN_LOG=debug
xgen-client send --space TestSpace --room RoomA --text "hello" (as member)
```
Expected: no `Event direction=` lines in either log. Operational lines (`Connecting to Node`, `Authenticated`, `Message sent`) still appear.

**Test 3 — content field never appears:**
Search both log files for the message text "hello". It MUST NOT appear anywhere in any log file at any level.

**Test 4 — Pairing works:**
Send a message as admin. Take the `event_id` from the client log. Search the node log for the same `event_id`. Confirm it appears with `direction=Inbound` and matching fields.

**Test 5 — No Event log calls outside trace_event:**
```
grep -r "tracing::.*event_id" xgen-node/src/ xgen-client/src/
```
Result should show only the two `trace_event` call sites (in transport boundary files), no other files.

---

## What stays from Phase 1

The Phase 1 debug log infrastructure is fully retained:
- Datetime-stamped files in `logs/` subfolder
- Config `[logging].level` switch
- `XGEN_LOG` env var override
- `with_ansi(false)` file output
- Subscriber init in `main()` of both binaries
- Operational lifecycle log calls (Node started, Identity registered, etc.)

Only the Event log point mechanism changes — from enumerated to global.

---

## Files created / modified

| File | Change |
|---|---|
| `xgen-common/src/tracing/mod.rs` | New — `EventDirection`, `SessionContext`, `SpaceRole`, `trace_event()` |
| `xgen-common/src/lib.rs` | Add `pub mod tracing` |
| `xgen-node/src/transport/server.rs` | Add `trace_event` call at inbound boundary |
| `xgen-node/src/message/exchange.rs` | Add `trace_event` call at outbound boundary (or wherever outbound send occurs) |
| `xgen-client/src/main.rs` or transport module | Add `trace_event` calls at both boundaries |
| `xgen-node/src/main.rs` | Remove redundant enumerated Event log calls |
| `xgen-client/src/main.rs` | Remove redundant enumerated Event log calls |

---

## Note on LOGGING_debug_ph1.md

`LOGGING_debug_ph1.md` specified the Phase 1 enumerated approach. That document was correct for its scope. This document supersedes it for Event tracing. The infrastructure sections of `LOGGING_debug_ph1.md` (subscriber init, file format, config) remain valid. The enumerated log point sections are superseded by the global interface defined here.

---

*End of document*
