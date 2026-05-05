# XGen Protocol — Logging Implementation Record
> Document type: Implementation record
> Applies to: `xgen-node` and `xgen-client` binaries
> Date: May 2026
> Prepared by: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> Status: COMPLETE — implemented by Claude Code, verified May 2026
> Decision record: D-033, D-032, D-038
> See also: `docs/xgen_appendix_g_en.md` — Appendix G: Log Line Convention (the format contract)
> See also: `docs/tests/LOGGING_debug_ph1.md` — Phase 1 debug log infrastructure
> See also: `docs/tests/LOGGING_debug_ph2.md` — Phase 2 global Event tracing interface

---

## Status

All logging implementation is complete. This document records what was built and where. It is not a to-do list.

---

## What Was Implemented

### `xgen-common/src/event_trace.rs`

The central logging module. Contains all logging types and functions used by both binaries.

**`EventDirection`** — `In` / `Out` / `Local`, Display produces `IN` / `OUT` / `LOCAL` per Appendix G.

**`SpaceRole`** — `Owner`, `Admin`, `Moderator`, `Member`. Used by role gate.

**`SessionContext`** — holds `identity_id`, `role`, `space_id` for the active session.

**`trace_event(event, direction, session)`** — logs a single network Event (IN or OUT). Role gate: suppressed unless Owner or Admin. `action` field derived from direction (`receive_event` for IN, `send_event` for OUT). Content field never logged. Local direction variant logs a warning and returns — use `trace_local()` instead.

**`LocalAction`** — enum of valid LOCAL actions: `CreateEvent`, `StoreEvent`, `ApplyEvent`, `RejectEvent`. Display produces lowercase per Appendix G action registry.

**`trace_local(action, event_id, event_type, space_id, error_code)`** — logs a LOCAL action. No role gate — LOCAL actions contain no sensitive content.

**`write_session_header(app_type, self_id, endpoint, connected_node, protocol_version, build, session_id, started_at)`** — writes the Appendix G session header block followed by a mandatory blank line. All parameters accept `Option<&str>` — None fields are omitted from the header. See D-038 for the client exception.

**`ExitReason`** — `Shutdown`, `Restart`, `Error`. Display produces lowercase per Appendix G.

**`write_session_footer(reason)`** — writes a mandatory blank line followed by the Appendix G session footer block. Called on every clean exit path. Never called on crash or kill — absence of footer signals abnormal termination.

---

### `xgen-node/src/main.rs` — wiring

- Keypair loaded **before** subscriber init so `node_id` is available for a complete session header
- Subscriber init: datetime-stamped file in `logs/`, `with_ansi(false)`, config level switch, `XGEN_LOG` env var override
- `write_session_header()` called immediately after subscriber init with all fields populated
- `trace_event()` called at the inbound Event boundary in `handle_connection()` before `process_inbound()`
- `trace_event()` called at the outbound boundary in `handle_federation_incoming()` before each `conn.send_event()`
- `trace_local()` called for `CreateEvent`, `StoreEvent`, `ApplyEvent`, `RejectEvent` in `handle_federation_incoming()` and `process_inbound()`
- `write_session_footer(ExitReason::Shutdown)` called on clean Ctrl+C shutdown

---

### `xgen-client/src/main.rs` — wiring

- Subscriber init: same pattern as node
- `write_session_header()` called immediately after subscriber init with `identity_id=None` and `connected_node=None` — see D-038
- `identity_id` and `connected_node` logged as structured body lines inside each command handler after auth completes:
  ```rust
  tracing::info!("identity_id={}", auth_id);
  tracing::info!("connected_node={}", node_url);
  ```
- `trace_event()` called at outbound boundary before each `conn.send_event()` in all command handlers
- `trace_event()` called at inbound boundary in `cmd_history()` on received Events
- `write_session_footer(ExitReason::Shutdown)` called on clean exit
- `write_session_footer(ExitReason::Error)` called before `std::process::exit(1)` on fatal error

---

## D-038 Exception — Client Header Partial Fields

The client session header omits `identity_id` and `connected_node` because those values are not available at subscriber init time. Full rationale in D-038 (`DECISIONS.md`). This is a CLI client limitation. The Tauri UI client (Phase 2) will write a complete header.

---

## Appendix G Compliance

| Requirement | Status |
|---|---|
| `key=value` format throughout | ✅ |
| `IN` / `OUT` / `LOCAL` direction values | ✅ |
| `action` field mandatory on every body line | ✅ |
| Session header with marker and blank line | ✅ |
| Session footer with marker, `ended_at`, `reason` | ✅ |
| Blank line before footer | ✅ |
| No footer on crash/kill | ✅ (not called from panic/signal handlers) |
| Content field never logged | ✅ |
| Role gate on `trace_event` | ✅ |
| No role gate on `trace_local` | ✅ |
| `LOCAL` action list: create, store, apply, reject | ✅ |
| Client header partial fields (D-038) | ✅ documented exception |

---

*End of document*
