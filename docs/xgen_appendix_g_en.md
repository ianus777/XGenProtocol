# XGen Protocol — Appendix G: Log Line Convention
> Status: active
> Version: 1.0
> Date: May 2026
> Last edited: May 2026
> Language: English
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

This appendix defines the log line format, session structure, and field contract for all XGen debug log output. It applies to both `xgen-node` and `xgen-client`. It covers format only — what gets logged and when is specified in `docs/tests/LOGGING_debug_ph1.md` and `docs/tests/LOGGING_debug_ph2.md`. The audit log is a separate system defined in `docs/tests/LOGGING_audit_ph2.md` and is never mixed with debug log output.

---

## Design Principles

### 1. Event-Centric Correlation

All cross-node pairing is performed using `event_id`.

- `event_id` is the only globally reliable identifier
- timestamps are local and non-authoritative
- log analysis MUST NOT rely on time correlation

A valid XGen log allows full reconstruction of Event flow across nodes using `event_id` alone.

### 2. Session-Scoped Identity

Each log file represents a single execution session of one binary. A session header MUST appear at the top of every log file. A session footer MUST appear at the bottom on clean exit. Absence of a footer indicates abnormal termination.

### 3. One Action Per Line

Each log line represents exactly one atomic action.

- no multiline entries
- no embedded blocks
- no implicit continuation

Compatible with streaming analysis, line-based tools (`grep`, `awk`), and AI ingestion without preprocessing.

### 4. Key=Value Structure Throughout

Every field in every line — header, body, and footer — uses the same flat format:

```
key=value key=value key=value
```

Rules:
- keys are lowercase `snake_case`
- values are unquoted unless they contain spaces (quote with double quotes if needed)
- no free-text sentences
- no positional meaning
- no alternative formats (no colon syntax, no JSON, no nested structures)

### 5. Stable Field Names

Field names are part of the logging contract. They MUST remain stable across versions. New fields may be added. Existing fields MUST NOT be renamed or removed. Unknown fields MUST be silently ignored by analyzers.

### 6. Explicit Semantics

All meaning MUST be explicit in field values.

Forbidden:
```
received event
sending message
operation successful
```

Required:
```
direction=IN action=receive_event event_id=...
direction=OUT action=send_event event_id=...
direction=LOCAL action=store_event event_id=...
```

### 7. No Decorative Output

Logs MUST NOT contain emojis, ANSI colour codes, formatting symbols, natural language commentary, or redundant success messages. Logs are a data structure, not a user interface.

---

## Session Structure

Every log file has three parts:

```
[session header]
[blank line]
[body lines]
[blank line]
[session footer]
```

The blank lines are mandatory. They delimit the header and footer blocks from the body and are part of the format contract.

---

## Session Header

The session header MUST appear exactly once, at the top of the file, before any body lines. It MUST be static — never updated during runtime.

Opening marker line (mandatory, literal):
```
=== XGEN SESSION START ===
```

Header fields follow immediately on the next lines. All fields are mandatory.

**Node header:**
```
=== XGEN SESSION START ===
app_type=node
node_id=xgen://pubkey/ed25519:AAA...
endpoint=127.0.0.1:8080
protocol_version=0.1
build=0.6.2-260430-1724
session_id=9f3a7c12
started_at=2026-04-30T17:24:50.123Z

```

**Client header:**
```
=== XGEN SESSION START ===
app_type=client
identity_id=xgen://pubkey/ed25519:BBB...
connected_node=127.0.0.1:8080
protocol_version=0.1
build=0.6.2-260430-1724
session_id=a81c2d44
started_at=2026-04-30T17:24:51.002Z

```

Note the mandatory blank line after the last header field. This blank line is the body start delimiter.

**Header field reference:**

| Field | Present in | Description |
|---|---|---|
| `app_type` | both | `node` or `client` |
| `node_id` | node only | Ed25519 pubkey URI of this Node |
| `identity_id` | client only | Ed25519 pubkey URI of the authenticated identity |
| `endpoint` | node only | WebSocket listen address |
| `connected_node` | client only | WebSocket address of the Node this client connected to |
| `protocol_version` | both | XGen protocol version |
| `build` | both | Binary build string |
| `session_id` | both | Random hex identifier for this session, unique per run |
| `started_at` | both | RFC 3339 UTC datetime of process start |

---

## Session Footer

The session footer MUST appear at the end of the file on any clean exit. It MUST NOT appear if the process terminates abnormally — absence of a footer is itself a signal of crash or kill.

A mandatory blank line MUST precede the footer marker. This blank line is the body end delimiter.

Opening marker line (mandatory, literal):
```
=== XGEN SESSION END ===
```

Footer fields follow immediately. All fields are mandatory.

```

=== XGEN SESSION END ===
ended_at=2026-04-30T18:15:00.000Z
reason=shutdown
```

**Footer field reference:**

| Field | Values | Description |
|---|---|---|
| `ended_at` | RFC 3339 UTC datetime | Moment of clean exit |
| `reason` | `shutdown` \| `restart` \| `error` | Exit classification |

**`reason` value semantics:**

| Value | Meaning |
|---|---|
| `shutdown` | Normal operator-initiated shutdown |
| `restart` | Intentional restart (e.g. config reload) |
| `error` | Caught fatal error — process exited cleanly after logging the error |
| *(no footer)* | Crash or external kill — abnormal termination |

---

## Body Lines

Body lines appear between the header blank line and the footer blank line. Each line represents one atomic action.

**Mandatory fields on every body line:**

| Field | Description |
|---|---|
| `timestamp` | RFC 3339 UTC datetime with millisecond precision |
| `direction` | `IN`, `OUT`, or `LOCAL` (see below) |
| `action` | Operation being performed (see action registry below) |
| `event_id` | XGen event URI — present on all Event-related lines |

**Optional fields (include when available and meaningful):**

| Field | Description |
|---|---|
| `event_type` | EventType string (e.g. `message.text`, `state.space_create`) |
| `sender` | Identity pubkey URI of the Event author |
| `space_id` | Space identifier URI |
| `room_id` | Room identifier URI — omit or leave empty for Space-level Events |
| `target` | Destination node or identity URI — outbound only, when specific |
| `error_code` | Protocol error code — on rejection or error actions only |

---

## Direction Values

| Value | Meaning |
|---|---|
| `IN` | Event arrived at this binary from the network |
| `OUT` | Event left this binary to the network |
| `LOCAL` | Action occurred entirely within this binary — no network crossing |

---

## Action Registry

All valid `action` values. This list is the contract — no other values may be used.

**Network actions:**

| Action | Direction | Description |
|---|---|---|
| `receive_event` | `IN` | Event received and deserialized from network |
| `send_event` | `OUT` | Event serialized and sent to network |

**Local actions:**

| Action | Direction | Description |
|---|---|---|
| `create_event` | `LOCAL` | Event constructed locally before sending |
| `store_event` | `LOCAL` | Event written to the DAG store |
| `apply_event` | `LOCAL` | Event applied to Space or Room state machine |
| `reject_event` | `LOCAL` | Event rejected by validation pipeline — nothing sent |

Note: a single real-world scenario may produce multiple lines. Example — incoming Event that fails validation and triggers an error response:
```
timestamp=... direction=IN action=receive_event event_id=...
timestamp=... direction=LOCAL action=reject_event event_id=... error_code=3042
timestamp=... direction=OUT action=send_event event_id=...
```
Each line records one distinct action. This is correct and expected.

---

## Complete Example

```
=== XGEN SESSION START ===
app_type=node
node_id=xgen://pubkey/ed25519:AAA...
endpoint=127.0.0.1:8080
protocol_version=0.1
build=0.6.2-260430-1724
session_id=9f3a7c12
started_at=2026-04-30T17:24:50.123Z

timestamp=2026-04-30T17:24:50.201Z direction=LOCAL action=create_event event_id=xgen://hash/sha256:abc... event_type=state.space_create space_id=xgen://hash/sha256:def...
timestamp=2026-04-30T17:24:50.456Z direction=OUT action=send_event event_id=xgen://hash/sha256:abc... target=xgen://pubkey/ed25519:CCC...
timestamp=2026-04-30T17:24:50.789Z direction=IN action=receive_event event_id=xgen://hash/sha256:abc... event_type=state.space_create sender=xgen://pubkey/ed25519:BBB... space_id=xgen://hash/sha256:def...
timestamp=2026-04-30T17:24:50.812Z direction=LOCAL action=store_event event_id=xgen://hash/sha256:abc... space_id=xgen://hash/sha256:def...
timestamp=2026-04-30T17:24:50.815Z direction=LOCAL action=apply_event event_id=xgen://hash/sha256:abc... space_id=xgen://hash/sha256:def...

=== XGEN SESSION END ===
ended_at=2026-04-30T18:15:00.000Z
reason=shutdown
```

---

## Parsing Rules

For implementors writing log analyzers or AI ingestion pipelines:

1. A line starting with `=== XGEN SESSION START ===` opens a header block
2. Header fields are `key=value` lines immediately following the marker
3. A blank line after header fields closes the header block and opens the body
4. Body lines always begin with `timestamp=`
5. A blank line after body lines closes the body
6. A line starting with `=== XGEN SESSION END ===` opens a footer block
7. Footer fields are `key=value` lines immediately following the marker
8. EOF closes the footer block
9. EOF without a footer marker — abnormal termination
10. Unknown fields MUST be silently ignored

---

## What This Appendix Does Not Cover

- **Role gate** — debug output is suppressed for non-owner/admin sessions. Defined in D-033 and implemented in `event_trace.rs`. Not a format concern.
- **Audit log** — a separate, always-on, append-only JSON Lines system. Defined in `docs/tests/LOGGING_audit_ph2.md`. Never mixed with debug log output.
- **Log file naming and rotation** — defined in `docs/tests/LOGGING_debug_ph1.md`.
- **Subscriber initialisation** — defined in `docs/tests/LOGGING_debug_ph1.md` and `docs/tests/LOGGING_debug_ph2.md`.
- **JSON export mode** — possible future extension. Any such mode MUST preserve the `event_id`-centric correlation model.

---

## Non-Goals

This appendix intentionally avoids:

- JSON logging — unnecessary complexity for the primary debug format
- binary log formats
- human-oriented narrative formatting
- transport-level tracing duplication
- schema registries or parser libraries — the format is self-describing

---

*End of appendix*
