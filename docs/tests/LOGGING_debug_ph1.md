# XGen Protocol — Debug Logging Implementation Instructions
> Document type: Implementation instructions for Claude Code  
> Applies to: `xgen-node` and `xgen-client` binaries — **debug log only**  
> Date: April 2026  
> Prepared by: JozefN  
> See also: `docs/tests/LOGGING_audit_ph2.md` for audit log implementation (Phase 2)
> See also: `docs/tests/LOGGING_debug_ph2.md` for global Event tracing interface (Phase 2 — first implementation task)
> See also: `docs/xgen_appendix_g_en.md` for the log line format convention (Appendix G)

---

## Important: two separate log types

XGen has two independent log systems. This document covers only the **debug log**.

| | This document | LOGGING_audit_ph2.md |
|---|---|---|
| What | Debug log | Audit log |
| Purpose | Diagnose problems | Prove accountability |
| Audience | Developer, operator | Auditor, regulator |
| Controlled by | `[logging].level` in config | Always on — cannot be disabled |
| Location | `logs/xgen-node_YYYY-MM-DD_HH-MM-SS.log` | `audit/protocol_audit_YYYY-MM.jsonl` |
| Retention | Operator deletes when done | Never auto-deleted |

Do not merge these two logs or write audit events into the debug log. The audit log is Phase 2 — see `LOGGING_audit_ph2.md`.

---

## Design decision

Logging is permanent infrastructure. Every run of `xgen-node` or `xgen-client` creates a new log file with a datetime suffix. Log files accumulate in a `logs/` subfolder relative to the executable's working directory. Operators control log verbosity via the config file. No environment variables required for normal use.

---

## Log file naming and location

### Node

Log files are written to a `logs/` subfolder relative to the Node's working directory (the folder where `xgen-node_config.toml` lives). The existing `log_path` field in `[paths]` is **removed** and replaced by a `[logging]` section (see Step 1).

Each run creates a new file:
```
logs/xgen-node_2026-04-29_14-35-22.log
logs/xgen-node_2026-04-30_09-12-04.log
```

Pattern: `logs/xgen-node_YYYY-MM-DD_HH-MM-SS.log`

The datetime is the moment the process starts, in local time, formatted as `YYYY-MM-DD_HH-MM-SS`. Log files are never overwritten — each run produces a new file. Old log files accumulate and are not automatically deleted (log rotation is out of scope for Phase 1).

### Client

Same pattern, relative to the client's working directory:
```
logs/xgen-client_2026-04-29_14-35-40.log
```

Pattern: `logs/xgen-client_YYYY-MM-DD_HH-MM-SS.log`

The `logs/` subfolder is created automatically on first run if it does not exist.

---

## Log line format

Each log line uses the following fixed format:

```
YYYY-MM-DD HH:MM:SS.mmm [LEVEL] target: message key=value key=value
```

**Example lines:**

```
2026-04-29 14:35:22.401 [INFO ] xgen_node_lib::node::runtime: Node started node_id=xgen://pubkey/ed25519:Cazue8... endpoint=ws://127.0.0.1:8080/xgen
2026-04-29 14:35:31.014 [INFO ] xgen_node_lib::identity::registration: Identity registered identity_id=xgen://pubkey/ed25519:mvHwAL...
2026-04-29 14:35:31.902 [WARN ] xgen_node_lib::space::state: Event rejected step=10 reason=space_not_found sender=xgen://pubkey/ed25519:mvHwAL...
2026-04-29 14:35:32.118 [ERROR] xgen_node_lib::transport::server: Client disconnected identity_id=xgen://pubkey/ed25519:mvHwAL... reason=connection_reset
2026-04-29 14:35:33.220 [DEBUG] xgen_node_lib::federation::handshake: Handshake state transition state=CAPS_SENT
```

**Format rules for Claude Code:**

- Timestamp: local time, `YYYY-MM-DD HH:MM:SS.mmm` (millisecond precision)
- Level: fixed-width 5 chars in brackets: `[INFO ]`, `[WARN ]`, `[ERROR]`, `[DEBUG]`, `[TRACE]`
- Target: the Rust module path (provided automatically by `tracing` via `with_target(true)`)
- Message: short description, no trailing punctuation
- Fields: space-separated `key=value` pairs after the message — use `%` sigil (Display) for URIs and strings, `?` for debug types
- No ANSI colour codes in file output (`with_ansi(false)`)
- No blank lines between log lines
- UTF-8 encoding

---

## Step 1 — Update xgen-node config

### 1a — Update the NodeConfig struct in xgen-node/src/main.rs

Remove `log_path` from `PathsSection`. Add a `LoggingSection`:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct LoggingSection {
    /// Log level: "off" | "error" | "warn" | "info" | "debug" | "trace"
    level: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PathsSection {
    keypair_path: String,
    spaces_dir: String,
    // log_path REMOVED — logging config is now in [logging]
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeConfig {
    node: NodeSection,
    paths: PathsSection,
    logging: LoggingSection,  // ADD
}
```

### 1b — Update both test config files

**test/node_a/xgen-node_config.toml** — replace existing content with:

```toml
[node]
listen = "ws://127.0.0.1:8080/xgen"
local_mode = true

[paths]
keypair_path = 'test/node_a\xgen-node_keypair.enc'
spaces_dir = 'G:\My Drive\Projects\XGenProtocol\bin\spaces'

[logging]
level = "info"
```

**test/node_b/xgen-node_config.toml** — replace existing content with:

```toml
[node]
listen = "ws://127.0.0.1:8081/xgen"
local_mode = true

[paths]
keypair_path = 'test/node_b\xgen-node_keypair.enc'
spaces_dir = 'G:\My Drive\Projects\XGenProtocol\bin\spaces'

[logging]
level = "info"
```

---

## Step 2 — Add logging to xgen-client config

The client currently has no config file. Add a `[logging]` section to the client config struct and template.

### 2a — Add LoggingSection to ClientConfig struct in xgen-client/src/main.rs

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct LoggingSection {
    level: String,
}

// Add to existing ClientConfig:
struct ClientConfig {
    // ... existing fields ...
    logging: LoggingSection,
}
```

### 2b — Add `[logging]` to the client config template / default config

Wherever the client generates or expects its config file, add:

```toml
[logging]
level = "info"
```

---

## Step 3 — Implement log file initialisation

Add the following dependencies to both `Cargo.toml` files if not already present:

**xgen-node/Cargo.toml** — ensure `env-filter` feature on `tracing-subscriber`:
```toml
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**xgen-client/Cargo.toml** — add:
```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 3a — Log initialisation code for xgen-node/src/main.rs

Place this block **after** config is loaded, **before** any other startup logic:

```rust
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::{fmt, EnvFilter};

// Determine log directory relative to working directory
let log_dir = PathBuf::from("logs");
fs::create_dir_all(&log_dir)
    .expect("Failed to create logs/ directory");

// Build filename: xgen-node_YYYY-MM-DD_HH-MM-SS.log
let now = chrono::Local::now();
let log_filename = format!(
    "xgen-node_{}.log",
    now.format("%Y-%m-%d_%H-%M-%S")
);
let log_path = log_dir.join(&log_filename);

// Open log file in append mode (safe even if file already exists)
let log_file = fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(&log_path)
    .expect("Failed to open log file");

// Determine effective level: config default, no env var override in production
// (XGEN_LOG env var still works for development if set)
let env_filter = if std::env::var("XGEN_LOG").is_ok() {
    EnvFilter::from_env("XGEN_LOG")
} else {
    EnvFilter::new(&config.logging.level)
};

// Initialise subscriber with file output and fixed format
fmt()
    .with_env_filter(env_filter)
    .with_target(true)
    .with_ansi(false)
    .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
        "%Y-%m-%d %H:%M:%S%.3f".to_string()
    ))
    .with_level(true)
    .with_writer(log_file)
    .init();

tracing::info!("Log file opened: {}", log_path.display());
```

### 3b — Identical pattern in xgen-client/src/main.rs

Same code, with `xgen-client` as the filename prefix:

```rust
let log_filename = format!(
    "xgen-client_{}.log",
    now.format("%Y-%m-%d_%H-%M-%S")
);
```

Everything else identical.

---

## Step 4 — Add structured log points to xgen-node

Replace existing `println!` / `eprintln!` operational output with `tracing::` calls.

**Rule:** user-facing CLI output (startup banner printed to stdout, command results like spaces list) stays as `println!`. Internal operational events become `tracing::` calls written to the log file.

**Log level discipline:**

| Macro | Use for |
|---|---|
| `tracing::error!` | Something failed — request, connection, validation |
| `tracing::warn!` | Unexpected but recoverable |
| `tracing::info!` | Normal milestone an operator cares about |
| `tracing::debug!` | Internal detail for debugging |
| `tracing::trace!` | Step-by-step internals (use sparingly) |

**Minimum required log points — xgen-node/src/main.rs and library modules:**

```rust
// Startup — first log line after subscriber init
tracing::info!(node_id = %node_id, endpoint = %endpoint, "Node started");

// Identity registered
tracing::info!(identity_id = %identity_id, "Identity registered");

// Identity registration rejected
tracing::warn!(identity_id = %identity_id, reason = %reason, "Identity registration rejected");

// Client authenticated (transport auth succeeded)
tracing::info!(identity_id = %identity_id, "Client authenticated");

// Client disconnected
tracing::info!(identity_id = %identity_id, reason = %reason, "Client disconnected");

// Space created
tracing::info!(space_id = %space_id, name = %name, "Space created");

// Space not found — Fix 16 error (was raw println!, now structured)
tracing::error!(space_id = %space_id, step = 10, "accept_message failed: space not found");

// Event accepted
tracing::debug!(event_id = %event_id, event_type = %event_type, sender = %sender, "Event accepted");

// Event rejected — any validation step
tracing::warn!(step = %step, reason = %reason, sender = %sender, "Event rejected");

// Federation handshake complete
tracing::info!(peer_node_id = %peer_node_id, shared_spaces = %shared_spaces, "Federation established");

// Federation handshake failed
tracing::error!(peer_node_id = %peer_node_id, reason = %reason, "Federation failed");

// Node shutdown
tracing::info!("Node shutting down");
```

**Library module debug points** (invisible at `info` level):

```rust
// transport — connection lifecycle
tracing::debug!(peer_addr = %addr, "Incoming connection accepted");
tracing::debug!(peer_addr = %addr, "Connection closed");

// identity/registration.rs — pipeline
tracing::debug!(step = %step, "Registration pipeline step passed");
tracing::warn!(step = %step, reason = %reason, "Registration pipeline failed");

// space/state.rs — event validation
tracing::debug!(step = %step, event_id = %event_id, "Validation step passed");
tracing::warn!(step = %step, event_id = %event_id, reason = %reason, "Validation failed");

// federation/handshake.rs — state machine
tracing::debug!(state = %state_name, "Handshake state transition");
```

---

## Step 5 — Add structured log points to xgen-client

```rust
tracing::info!(node_url = %url, "Connecting to Node");
tracing::info!(identity_id = %identity_id, "Authenticated");
tracing::error!(reason = %reason, "Authentication failed");
tracing::info!(space_id = %space_id, "Joined Space");
tracing::info!(event_id = %event_id, room = %room, "Message sent");
tracing::error!(reason = %reason, "Message send failed");
tracing::info!(peer_node_url = %peer_url, "Federation initiated");
```

---

## Step 6 — Verify

**Test 1 — Log file created on startup:**
Start `xgen-node`. Confirm `logs/xgen-node_YYYY-MM-DD_HH-MM-SS.log` created in the Node's working directory.

**Test 2 — New file per run:**
Stop and restart `xgen-node`. Confirm a second log file is created with a new datetime suffix. First file is untouched.

**Test 3 — Log line format:**
Open a log file. Confirm lines match:
```
2026-04-29 14:35:22.401 [INFO ] xgen_node_lib::...: Node started node_id=... endpoint=...
```

**Test 4 — Level off:**
Set `level = "off"` in config, restart Node. Confirm log file is created but receives no entries (empty after the file-open line).

**Test 5 — Level debug:**
Set `level = "debug"`, restart. Confirm debug lines appear for transport connections and validation steps.

**Test 6 — Client log:**
Run `xgen-client spaces`. Confirm `logs/xgen-client_YYYY-MM-DD_HH-MM-SS.log` created in client's working directory with connect/auth entries.

**Test 7 — Fix 16 error is now structured:**
Reproduce Fix 16 (create Space, restart Node, send message). Confirm the log contains:
```
YYYY-MM-DD HH:MM:SS.mmm [ERROR] xgen_node_lib::space::state: accept_message failed: space not found space_id=xgen://hash/sha256:...
```

---

## Important constraints

- **No secrets in logs at any level** — never log private key material, passphrase, or raw keypair bytes
- **`with_ansi(false)` mandatory for file output** — no colour codes in log files
- **Append mode** — always `OpenOptions::append(true)`, never truncate
- **`logs/` folder created automatically** — do not require operator to create it manually
- **Structured key=value fields** — use `%` (Display) for URIs and strings, `?` (Debug) for complex types; no interpolated format strings in the message text itself

---

## Files modified

| File | Changes |
|---|---|
| `xgen-node/Cargo.toml` | Add `env-filter` feature to `tracing-subscriber` |
| `xgen-client/Cargo.toml` | Add `tracing` and `tracing-subscriber` with `env-filter` |
| `xgen-node/src/main.rs` | Remove `log_path` from PathsSection; add `LoggingSection`; add log init block; add log points |
| `xgen-client/src/main.rs` | Add `LoggingSection` to ClientConfig; add log init block; add log points |
| `test/node_a/xgen-node_config.toml` | Remove `log_path` from `[paths]`; add `[logging]` section |
| `test/node_b/xgen-node_config.toml` | Remove `log_path` from `[paths]`; add `[logging]` section |
| `xgen-node/src/transport/` | Add debug-level connection log points |
| `xgen-node/src/identity/` | Add debug-level registration pipeline log points |
| `xgen-node/src/space/` | Add debug-level event validation log points |
| `xgen-node/src/federation/` | Add debug-level handshake state log points |

---

*End of logging instructions*
