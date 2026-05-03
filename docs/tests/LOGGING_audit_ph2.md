# XGen Protocol — Audit Log Implementation Instructions
> Document type: Implementation instructions for Claude Code  
> Applies to: `xgen-node` binary — protocol audit log  
> Date: April 2026  
> Prepared by: JozefN  
> See also: `LOGGING_debug_ph1.md` for debug log implementation  
> Spec reference: 3.11.8 Audit Log Requirements  
> **Phase: Phase 2 — implement alongside Tier 2+ Auth Module work**

---

## When to implement this

Do not implement the audit log during Phase 1. The audit log is Phase 2 infrastructure — it is part of the Tier 1+ (meaning Tier 2 and above) implementation. Implement it when Tier 2+ Auth Module work begins in Phase 2. The audit log has nothing meaningful to exercise until Tier 2+ Spaces and their membership Events exist.

The debug log (`LOGGING_debug_ph1.md`) is the current priority.

---

## Important: two separate log types

XGen has two independent log systems. This document covers only the **audit log**.

| | LOGGING_debug_ph1.md | This document |
|---|---|---|
| What | Debug log | Audit log |
| Purpose | Diagnose problems | Prove accountability |
| Audience | Developer, operator | Auditor, regulator |
| Controlled by | `[logging].level` in config | Always on — cannot be disabled |
| Location | `logs/xgen-node_YYYY-MM-DD_HH-MM-SS.log` | `audit/protocol_audit_YYYY-MM.jsonl` |
| Retention | Operator deletes when done | Never auto-deleted |

Do not merge these two logs or write debug events into the audit log.

---

## What the audit log is

The protocol audit log is a permanent, append-only, machine-readable record of all membership and state-change Events that occur on this Node. It exists to enable compliance audits, not to help debug the software. It cannot be disabled via config and must never be auto-deleted.

It is not a copy of the DAG. It is a structured summary of protocol-level facts — who joined what Space, when, under whose authority. The full Event is always recoverable from the DAG via `event_id` if the auditor needs it.

---

## Log file location and rotation

Audit log files live in an `audit/` subfolder relative to the Node's working directory (the folder where `xgen-node_config.toml` lives).

The Node MUST create the `audit/` folder automatically on first run if it does not exist.

**One file per calendar month:**
```
audit/protocol_audit_2026-04.jsonl
audit/protocol_audit_2026-05.jsonl
```

Pattern: `audit/protocol_audit_YYYY-MM.jsonl`

On the first Event of a new calendar month, the Node opens a new file. The previous month's file is closed and left untouched. Files accumulate indefinitely — the Node MUST NOT delete them automatically under any circumstance.

**File format:** JSON Lines — one complete JSON object per line, UTF-8, no trailing comma, newline `\n` at end of each line. This format is directly importable into any log aggregation system (Elasticsearch, Splunk, ClickHouse, etc.).

---

## Log line format

Every audit log line is a flat JSON object with no nesting. All fields are present on every line — no optional fields are omitted (use `null` only if a field is genuinely not applicable for that EventType).

**Mandatory fields on every line:**

| Field | Type | Description |
|---|---|---|
| `ts` | string | RFC 3339 UTC timestamp with millisecond precision — `"2026-04-29T14:35:31.014Z"` |
| `event_type` | string | XGen EventType string — e.g. `"membership.join"` |
| `event_id` | string | XGen event_id hash URI — links this entry back to the full Event in the DAG |
| `node_id` | string | The Node that produced this audit entry (this Node's pubkey_uri) |

**EventType-specific fields — add the following for each EventType:**

| EventType | Additional fields |
|---|---|
| `membership.join` | `identity_id`, `space_id`, `approving_node_id` |
| `membership.leave` | `identity_id`, `space_id` |
| `membership.invite` | `inviter_id`, `invitee_id`, `space_id` |
| `membership.kick` | `kicker_id`, `kicked_id`, `space_id`, `reason` (string or null) |
| `membership.ban` | `banner_id`, `banned_id`, `space_id`, `reason` (string or null) |
| `state.space_create` | `creator_id`, `space_id`, `auth_tier` (integer) |
| `state.room_create` | `creator_id`, `room_id`, `space_id` |
| `state.federation_add` | `initiating_node_id`, `receiving_node_id`, `space_id` |
| `state.federation_remove` | `departing_node_id`, `space_id`, `reason` (string or null) |
| `identity.register` | `identity_id`, `home_node_id`, `tier_verified` (integer) |
| `system.key_rotation` | `identity_id`, `old_key_hash`, `new_key_hash` |

**Example lines:**

```jsonl
{"ts":"2026-04-29T14:35:31.014Z","event_type":"membership.join","event_id":"xgen://hash/sha256:a3f9b2c1...","node_id":"xgen://pubkey/ed25519:CCCC...","identity_id":"xgen://pubkey/ed25519:AAAA...","space_id":"xgen://hash/sha256:b2c3d4e5...","approving_node_id":"xgen://pubkey/ed25519:CCCC..."}
{"ts":"2026-04-29T14:36:02.881Z","event_type":"state.space_create","event_id":"xgen://hash/sha256:d4e5f6a7...","node_id":"xgen://pubkey/ed25519:CCCC...","creator_id":"xgen://pubkey/ed25519:AAAA...","space_id":"xgen://hash/sha256:b2c3d4e5...","auth_tier":1}
{"ts":"2026-04-29T14:40:15.203Z","event_type":"membership.ban","event_id":"xgen://hash/sha256:e5f6a7b8...","node_id":"xgen://pubkey/ed25519:CCCC...","banner_id":"xgen://pubkey/ed25519:BBBB...","banned_id":"xgen://pubkey/ed25519:DDDD...","space_id":"xgen://hash/sha256:b2c3d4e5...","reason":"repeated_harassment"}
```

---

## Implementation instructions

### Step 1 — Create the audit writer module

Create a new module `xgen-node/src/audit/mod.rs` (or `xgen-node/src/audit.rs` if preferred). This module is the single write path for all audit log entries. No other code writes to the audit log directly.

The module exposes one public function:

```rust
pub async fn write_audit_entry(entry: AuditEntry) -> Result<()>
```

Where `AuditEntry` is a struct covering all possible fields:

```rust
#[derive(serde::Serialize)]
pub struct AuditEntry {
    pub ts: String,                          // RFC 3339 UTC millisecond precision
    pub event_type: String,                  // XGen EventType string
    pub event_id: String,                    // hash URI
    pub node_id: String,                     // this Node's pubkey_uri
    
    // EventType-specific fields — all Option<String> or Option<i64>
    // Use None for fields not applicable to this EventType
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inviter_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kicker_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kicked_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banned_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approving_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiating_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiving_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub departing_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_key_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_key_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_tier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier_verified: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
```

The `write_audit_entry` function:
1. Serialises the entry to a single JSON line using `serde_json::to_string`
2. Appends the line plus `\n` to the current month's audit file
3. Checks whether the current month has changed since the last write — if so, closes the current file handle and opens a new one for the new month
4. On file open error: logs the error to the debug log via `tracing::error!` and returns an error — **does not panic, does not silently drop the entry**

### Step 2 — Initialise the audit writer in xgen-node/src/main.rs

After config is loaded and before the network listener opens, initialise the audit module:

```rust
// Create audit/ directory if it does not exist
let audit_dir = PathBuf::from("audit");
fs::create_dir_all(&audit_dir)
    .expect("Failed to create audit/ directory");

// Initialise audit writer (opens current month's file)
audit::init(audit_dir, node_id.clone()).await
    .expect("Failed to initialise audit log — cannot start without audit capability");
```

**The Node MUST NOT start if the audit log cannot be initialised.** An audit-incapable Node is not a compliant Node.

### Step 3 — Call write_audit_entry at each relevant point

In the event acceptance pipeline (`space/state.rs` or wherever Events are validated and committed to the DAG), add a call to `audit::write_audit_entry` immediately after an Event passes the full 13-step validation pipeline and is committed to the SQLite store.

Add calls for every EventType listed in the format table above. The call pattern for each:

```rust
// Example: membership.join
audit::write_audit_entry(AuditEntry {
    ts: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    event_type: event.event_type.to_string(),
    event_id: event.event_id.clone(),
    node_id: node_id.clone(),
    identity_id: Some(event.sender.clone()),
    space_id: Some(event.space_id.clone()),
    approving_node_id: Some(node_id.clone()),
    ..Default::default()
}).await?;
```

The `..Default::default()` pattern fills all unused optional fields with `None`, which are then omitted from the JSON output by `skip_serializing_if`.

### Step 4 — Verify

**Test 1 — File created on first Event:**
Start `xgen-node`. Register an Identity, create a Space. Confirm `audit/protocol_audit_YYYY-MM.jsonl` created in Node's working directory.

**Test 2 — Correct format:**
Open the audit file. Confirm each line is valid JSON with `ts`, `event_type`, `event_id`, `node_id` present. Confirm no blank lines, no trailing commas, valid RFC 3339 timestamps.

**Test 3 — Append-only:**
Stop and restart the Node. Run more Events. Confirm the same monthly file gains new lines appended — it is not overwritten.

**Test 4 — Cannot be disabled:**
Set `[logging].level = "off"` in config, restart. Confirm the debug log produces nothing. Confirm the audit log continues to receive entries.

**Test 5 — All 11 EventTypes covered:**
Run through a full smoke test sequence. Confirm entries appear for: `identity.register`, `state.space_create`, `state.room_create`, `membership.join`, and `state.federation_add`. The remaining EventTypes (`membership.leave/kick/ban/invite`, `state.federation_remove`, `system.key_rotation`) can be verified when the corresponding CLI commands are implemented.

---

## Constraints

- **Append only:** always open with `.append(true)` — never truncate
- **Never auto-delete:** no rotation that deletes files, no TTL on audit files
- **No secrets:** never write private key material or passphrases to the audit log
- **No debug events in audit log:** the audit log contains only the 11 protocol EventTypes listed — not transport events, connection events, or internal errors
- **Panic on init failure, error on write failure:** failing to initialise the audit log is fatal — the Node should not run without audit capability. A write failure during operation should log to the debug log and return an error, but not crash the Node.
- **Client does not write an audit log:** the audit log is a Node responsibility only. The client produces a debug log only.

---

## Files created / modified

| File | Change |
|---|---|
| `xgen-node/src/audit/mod.rs` | New module — AuditEntry struct, write_audit_entry function, month-rotation logic |
| `xgen-node/src/main.rs` | Add audit module import; add audit init block; create `audit/` directory |
| `xgen-node/src/space/state.rs` | Add audit::write_audit_entry calls after Event commit for all 11 EventTypes |
| `xgen-node/src/lib.rs` | Expose `pub mod audit` |

---

*End of audit log instructions*
