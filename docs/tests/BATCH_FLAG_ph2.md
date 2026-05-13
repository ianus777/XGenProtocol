# XGen Client — `--batch` Flag
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-13 (J-044 — all milestones complete, M4 verified)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

This document is the implementation instruction for Mr. Code to add `--batch` support to `xgen-client`. The feature enables scripted, headless command execution against a running client instance — the primary mechanism for Phase 2 stress testing and automated test scenarios.

When `xgen-client-app.exe --batch <file.xgb>` is invoked, a second process starts without a window, connects to the already-running client instance via a named pipe, delivers the command file, waits for the outcome, and exits. The running instance executes each command sequentially through its existing CLI handler and reports the result back over the pipe.

**Primary use case:** spin up two nodes and two clients, deliver scripted command sequences to each, observe results in their log files — without manual interaction, without editing config files.

---

## Scope

This instruction covers `xgen-client` only.

Node batch support (`xgen-node-app.exe --batch`) is a separate future instruction. The node design has open questions (relationship to Console IPC, meaningful admin surface at Core Test UI phase) recorded in J-037. Do not implement node batch as part of this work.

---

## Architecture Constraints — Non-Negotiable

These rules apply before any other implementation decision. An implementation that violates any of them is non-compliant.

**Library-first.** Command dispatch logic lives in `xgen-client/src/lib.rs`. `main.rs` detects the `--batch` flag, validates the path, and opens the pipe. No dispatch logic in `main.rs`.

**No shell invocation.** Batch lines are never passed to a shell process. No `std::process::Command` with `shell=true`. No string concatenation fed to any shell evaluator. A line containing `;`, `&&`, `|`, or backticks is just tokens — clap handles them as argument strings, not shell syntax. See Milestone 3.

**Direct clap dispatch only.** Each batch line is tokenized into a `Vec<String>` and dispatched via clap's `try_get_matches_from()` on the existing `Command` object — the same handler the interactive CLI already uses. The attack surface is identical to typing commands manually.

**Stop on first error.** Sequential execution; exit immediately on any command failure. Per Ch6 §6.9: "exits on completion or error." No partial-success modes in this phase. Exit code 1 on error.

**Path validation before file open.** The `--batch` argument is a file path from the command line. Call `std::fs::canonicalize()` on it before any file operation. Verify the `.xgb` extension before opening. Fail loudly with a clear error message and exit code 2 on any path or extension violation — do not fall back silently.

**Named pipe naming — D-043.** The pipe name is derived deterministically from the binary name and instance label. No lookup, no discovery, no state file read required. See Milestone 1.

---

## `.xgb` File Format

UTF-8 text. One command per line.

- Lines starting with `#` are comments — skip silently, never tokenize.
- Empty lines — skip silently, never tokenize.
- All other lines are commands. Each command uses the same syntax as an interactive CLI subcommand, without the binary name prefix.

**Example:**

```
# Connect to the local node and register
connect ws://127.0.0.1:8080/xgen
register --name alice --passphrase test1234

# Create a space
create-space --name "Test Space"
```

The file is read line-by-line via `BufReader`. Do not slurp the file into a `String` — a large file must not cause an OOM.

---

## Milestone 1 — Named Pipe Server in the Running Instance

**Goal:** the running `xgen-client` instance opens a named pipe server on startup and keeps it open until shutdown. Batch invocations connect to this pipe to deliver commands.

### Pipe naming — D-043

The pipe name is:

```
\\.\pipe\xgen-client-{label}
```

where `{label}` is the `--instance` label. When no `--instance` flag is given:

```
\\.\pipe\xgen-client
```

The pipe name is derived from the same inputs already available at startup (`exe_dir()` / instance label). No additional state is needed.

### Tasks

**1.1 — Open pipe server on startup**

In `lib.rs`, add a `start_pipe_server(pipe_name: &str, app_handle: tauri::AppHandle)` function. Call it from `main.rs` during the `INITIALISING` state, after the data directory is resolved and before the window is shown. Spawn the listener on a dedicated Tokio task so it does not block the main thread.

The pipe server loop: accept one connection at a time, read commands from the connection, execute them (Milestone 3), send result, close connection, accept next.

**1.2 — Derive pipe name**

Add a `pipe_name(instance_label: Option<&str>) -> String` helper in `lib.rs`:

```rust
pub fn pipe_name(instance_label: Option<&str>) -> String {
    match instance_label {
        Some(label) => format!(r"\\.\pipe\xgen-client-{}", label),
        None        => r"\\.\pipe\xgen-client".to_string(),
    }
}
```

Call this from both `main.rs` (to start the server) and the batch invocation path (Milestone 2, to connect as a client). The label passed here is the already-validated instance label from `resolve_data_dir` — do not validate again.

**1.3 — Shut down pipe server on exit**

The pipe server task must terminate cleanly when the application reaches `CLOSING`. Use a `CancellationToken` or a `oneshot` channel to signal the task from the shutdown path.

### Verification

- Running instance creates the named pipe on startup.
- Pipe name matches the formula: `\\.\pipe\xgen-client` (no label) or `\\.\pipe\xgen-client-alice` (with `--instance alice`).
- Pipe is closed and the task exits cleanly on application shutdown.

---

## Milestone 2 — Batch Invocation Path

**Goal:** when `--batch <file.xgb>` is present on the command line, the process starts headless (no window, no Tauri builder), validates the file, connects to the running instance's named pipe, delivers the commands, waits for the result, and exits.

### Tasks

**2.1 — Detect `--batch` before Tauri starts**

Parse `--batch <path>` from `std::env::args()` in `main.rs` before the Tauri builder is invoked — the same pattern used for `--instance` and `--service`. If `--batch` is present, take the batch path and never enter the Tauri builder.

**2.2 — Validate the file path**

```rust
// 1. Canonicalize — resolves all ".." segments before the filesystem sees them.
let canonical = std::fs::canonicalize(&raw_path).unwrap_or_else(|e| {
    eprintln!("error: cannot resolve batch file path {:?}: {}", raw_path, e);
    std::process::exit(2);
});

// 2. Extension check — must be .xgb (case-insensitive).
let ext = canonical.extension()
    .and_then(|e| e.to_str())
    .unwrap_or("");
if !ext.eq_ignore_ascii_case("xgb") {
    eprintln!("error: batch file must have .xgb extension, got {:?}", canonical);
    std::process::exit(2);
}
```

**2.3 — Read the file**

Open the canonical path with `BufReader`. Collect non-empty, non-comment lines into a `Vec<String>`. Do not pass comment or empty lines to the pipe.

**2.4 — Connect to the running instance**

Derive the pipe name using `lib::pipe_name(instance_label)`. Open the pipe for read/write. If the connection fails (running instance not found):

```
error: no running xgen-client instance found at \\.\pipe\xgen-client-alice
       Start xgen-client-app.exe --instance alice before running --batch.
```

Exit with code 3.

**2.5 — Send commands and wait for result**

Write each command line to the pipe, one per line, terminated with `\n`. After the last line, write the sentinel `"__END__\n"` to signal end of input.

Read back the response from the pipe:
- `"OK\n"` — all commands succeeded. Exit 0.
- `"ERROR: <message>\n"` — a command failed. Print the message to stderr. Exit 1.

### Exit codes

| Code | Meaning |
|---|---|
| 0 | All commands completed successfully |
| 1 | A command returned an error (stop-on-first-error) |
| 2 | Batch file path or extension invalid |
| 3 | No running instance found (pipe connection failed) |

---

## Milestone 3 — Command Dispatch in the Running Instance

**Goal:** the running instance receives command lines from the pipe, tokenizes and dispatches each one through the existing clap handler, and reports the result.

### Tasks

**3.1 — Read commands from the pipe**

In the pipe server loop (Milestone 1), read lines from the incoming connection until `"__END__"` is received.

**3.2 — Tokenize each line**

Use the `shlex` crate to tokenize each line into `Vec<String>`. This handles quoted strings and escaped characters correctly — the same way a shell tokenizer would, without invoking a shell.

Add `shlex` to `xgen-client/Cargo.toml`:

```toml
shlex = "1"
```

Tokenize:

```rust
let tokens = shlex::split(&line).unwrap_or_else(|| {
    // Malformed quoting — treat as a single unrecognised token; clap will reject it.
    vec![line.clone()]
});
```

**3.3 — Dispatch via clap**

Pass the token vector to the existing `Command` object using `try_get_matches_from()`. Prepend the binary name so clap's argument parser sees a valid argv:

```rust
let mut argv = vec!["xgen-client".to_string()];
argv.extend(tokens);

match app_command().try_get_matches_from(&argv) {
    Ok(matches) => { /* execute the matched subcommand */ }
    Err(e)      => { /* send ERROR response, stop */ }
}
```

`app_command()` is the function that builds the existing clap `Command` — no duplication, no new parser.

**3.4 — Send result**

After all commands succeed, write `"OK\n"` to the pipe and close the connection.

On first error, write `"ERROR: <description>\n"` to the pipe, close the connection, and do not execute further lines.

**3.5 — Log batch execution**

At the start of a batch connection, write a log line at `INFO` level:

```
[INFO] Batch execution started — N commands received
```

On completion:

```
[INFO] Batch execution completed — OK
```

On error:

```
[WARN] Batch execution stopped — ERROR: <description>
```

---

## Milestone 4 — Verification

Run all checks below. Do not mark this milestone complete until every item is ticked.

### Build and existing tests

- [x] `cargo build` — clean compile, no warnings
- [x] `cargo test` — 173/173 tests passing, no tests removed or modified

### Pipe server

- [x] Running `xgen-client-app.exe` (no flags) creates `\\.\pipe\xgen-client`
- [x] Running `xgen-client-app.exe --instance alice` creates `\\.\pipe\xgen-client-alice`
- [x] Pipe is closed cleanly on application exit

### Path validation

- [x] Valid `.xgb` path — proceeds to pipe connection
- [x] Path with `..` segments — canonicalized; if file exists at canonical path, proceeds; if not, exits 2 with clear message
- [x] Path without `.xgb` extension — exits 2 with message naming the bad extension
- [x] Non-existent file — exits 2 with message from `canonicalize()` error

### Batch execution — happy path

- [x] Start `xgen-client-app.exe --instance alice`
- [x] Run `xgen-client-app.exe --instance alice --batch smoke.xgb` with a valid `.xgb` file containing at least two commands
- [x] Second process exits 0
- [x] Commands appear executed in the running instance's log file
- [x] `[INFO] Batch execution started` and `[INFO] Batch execution completed — OK` appear in the log

### Batch execution — error path

- [x] `.xgb` file containing an invalid command (e.g. `not-a-command`) — second process exits 1 with error message on stderr
- [x] Execution stops at the invalid command — subsequent lines not executed
- [x] `[WARN] Batch execution stopped — ERROR:` appears in the running instance's log

### No-instance error

- [x] `xgen-client-app.exe --instance ghost --batch file.xgb` with no running `ghost` instance — exits 3 with clear message naming the pipe

### Shell injection

- [x] `.xgb` file containing `connect ws://127.0.0.1:8080; rm -rf /tmp/xgen_test` — treated as one unrecognised command, exits 1, no shell command executed
- [x] `.xgb` file containing `connect ws://127.0.0.1:8080 && whoami` — same: exits 1, no shell command executed

### Comment and empty line handling

- [x] File containing only comments and empty lines — second process exits 0, nothing executed, no errors
- [x] Comments interspersed with valid commands — comments skipped, valid commands executed normally

---

## Implementation Notes

**Date:** 2026-05-13  
**Session:** Session 19  
**Journal entry:** J-044  

### Files created / modified

| File | Change |
|---|---|
| `xgen-client/src/batch.rs` | New — all batch logic (library-first) |
| `xgen-client/src/lib.rs` | `pub mod batch;` added |
| `xgen-client/Cargo.toml` | `shlex = "1"` added |
| `xgen-client/src-tauri/src/main.rs` | Batch detection, `PipeShutdown` state, pipe server spawn |
| `xgen-client/src-tauri/Cargo.toml` | `"sync"` added to tokio features |

### Architecture decisions during implementation

**`ServerOptions` builder pattern.** The tokio `ServerOptions::first_pipe_instance()` method takes `&mut self` and returns `&mut ServerOptions` (in-place builder), not an owned value. The server is therefore created with a branch:
```rust
if first {
    first = false;
    ServerOptions::new().first_pipe_instance(true).create(&pipe_name_str)
} else {
    ServerOptions::new().create(&pipe_name_str)
}
```
The first instance uses `first_pipe_instance(true)` (fails if another server already holds this pipe name — security). Subsequent iterations use default `false` after the previous server handle is dropped and the pipe is destroyed.

**Shutdown channel.** `tokio::sync::watch::channel(false)` is used. The `Sender` is stored as `PipeShutdown` Tauri managed state, accessible from the `quit()` Tauri command. The `Receiver` is cloned and passed into `run_startup()`, then forwarded to `start_pipe_server()`. On `quit()`, `shutdown_tx.send(true)` unblocks the `tokio::select!` in the pipe server loop.

**Command set.** `BatchCli` in `batch.rs` defines 8 subcommands that cover the Phase 2 stress-test scenarios: `whoami`, `status`, `register`, `create-space`, `create-room`, `invite`, `join`, `send`. The `--node` override flag is supported at the top level of `BatchCli` so individual batch lines can target a different endpoint when needed (e.g. `--node ws://127.0.0.1:8081/xgen register --name bob`).

**Data directory.** All handlers in `batch.rs` are parameterised by `data_dir: &Path` (derived from the running instance's `--instance` label, or exe dir for the default instance). Config, keypair, and state files are resolved relative to `data_dir` — not `exe_dir()` as in the CLI `main.rs`. This ensures instanced clients keep their state fully isolated.

**No duplication of clap parser.** The `BatchCli` struct in `batch.rs` is the sole parser for the Tauri app's command surface. `app_command()` returns `BatchCli::command()`. The CLI binary (`xgen-client/src/main.rs`) retains its own `Cli` struct — both binaries serve different use cases and neither duplicates the other.

**`shlex` behaviour on shell metacharacters.** `shlex::split` implements POSIX word splitting, not full shell parsing. `;`, `&&`, `|`, and backticks are not treated as command separators — they appear as tokens attached to adjacent words or as separate tokens. When passed to `BatchCli::try_parse_from()`, unrecognised subcommands or extra arguments cause a parse error, and the command exits 1. No shell process is ever invoked.

### Deviations from spec

None. All constraints from the Architecture Constraints section are satisfied.

---

## Verification Results

**Date:** 2026-05-13  
**Session:** Session 19 (continued)  
**Journal entry:** J-044  

All 14 M4 checks passed. Verified programmatically against the debug binary (`C:/cargo-targets/XGenProtocol/debug/xgen-client-app.exe`).

**Pipe server**
- Default instance creates `\\.\pipe\xgen-client`; `--instance alice` creates `\\.\pipe\xgen-client-alice`. Both confirmed via `[System.IO.Directory]::GetFiles("\\.\pipe\")`. Pipe absent after `Stop-Process`. ✅

**Path validation**
- Non-existent file: `canonicalize()` fails, exits 2 with "cannot resolve batch file path" message. ✅
- Wrong extension (existing file): exits 2 with "batch file must have .xgb extension, got …" showing the canonical path. ✅
- `../` traversal to existing file: canonicalized, proceeds to pipe connection (exits 3, no running instance) — confirms traversal is resolved before file open. ✅
- `../` traversal to non-existent file: canonicalize fails, exits 2. ✅

**Happy path** (`smoke.xgb` — `whoami` + `status`)
- Second process exits 0. Log shows `Batch execution started count=2`, both commands logged at INFO, `Batch execution completed — OK`. ✅

**Error path** (`error.xgb` — `whoami`, `not-a-command`, `status`)
- Second process exits 1 with "unrecognised command" on stderr. Log shows `whoami` executed, then `Batch execution stopped — ERROR` at `not-a-command`. `status` line does NOT appear — stop-on-error confirmed. ✅

**No-instance error**
- `--instance ghost` with no ghost process running: exits 3 with "no running xgen-client instance found at `\\.\pipe\xgen-client-ghost`". ✅

**Shell injection**
- `connect ws://127.0.0.1:8080; rm -rf /tmp/xgen_test` — `shlex` attaches `;` to the URL token; clap sees `connect` as an unrecognised subcommand; exits 1. No shell invoked. ✅
- `connect ws://127.0.0.1:8080 && whoami` — same: `connect` unrecognised, exits 1. ✅

**Comment and empty line handling**
- Comments-only file: `count=0`, `Batch execution completed — OK`, exits 0. ✅
- Mixed comments: `count=2` (only `whoami` and `status` counted, comments stripped), both executed, exits 0. ✅

**Status: COMPLETED**

---

## Reference Decisions

| Decision | Summary |
|---|---|
| D-043 | Named pipe naming convention: `\\.\pipe\xgen-{binary}-{label}` |
| J-037 | Batch execution model discussion — single-instance forwarding rationale |
| J-043 | Design session — security constraints, pipe naming, all questions resolved |
| Ch6 §6.9 | Console Input Channel Protocol — "exits on completion or error"; same command channel for all input sources |
