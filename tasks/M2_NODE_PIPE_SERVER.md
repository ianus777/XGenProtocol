# M2 — Node Pipe Server
> **Status**: PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-16 (created at M1 close-out — J-073 sequel)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

M1 (J-068 through J-073, commits `e864715` → `95ef5e1`) shipped both binaries with all 19 fundamental flags. Five of them on the Node side are stubs that print *"requires the M2 Node pipe server — not yet implemented"* and exit non-zero:

- `xgen-node --ping`
- `xgen-node --health`
- `xgen-node --stop`
- `xgen-node --reload-config`
- `xgen-node --batch <file.xgb>`

M2 makes those five flags real. The Client side has had a working pipe server since J-038 / J-044 (and gained the four single-line control commands in J-069's Phase 4). M2 ports the same pattern to the Node and wires the five Node-side handlers.

M2 is structural: it adds a new resident-only concern (the pipe server) to the Node, with a Node-specific command set for `__BATCH__`. It is **not** a protocol change — D-043's pipe-naming convention (`\\.\pipe\xgen-{node|client}-{label}`) and the four control tokens (`__PING__`, `__HEALTH__`, `__STOP__`, `__RELOAD_CONFIG__`) were locked in M1 and apply unchanged here.

---

## Cross-references

| Source | Relevance |
|---|---|
| `DECISIONS.md` D-043 | Pipe naming convention `\\.\pipe\xgen-{node\|client}-{label}` |
| `DECISIONS.md` D-056 | Application Deployment Model — every resident must host a pipe server |
| `DECISIONS.md` D-062 / D-063 | Library-first dispatch; pipe server lives in `xgen-node-lib` |
| `JOURNAL.md` J-068 → J-073 | M1 closure chain; current state of the binaries |
| `xgen-client/src/batch.rs::start_pipe_server` | The skeleton to port — battle-tested via J-038 / J-044 / J-069 / J-071 |
| `xgen-client/src/batch.rs::{cmd_ping,cmd_health,cmd_stop,cmd_reload_config,pipe_send_control}` | Client-side helpers — Node side wants exactly the same shape |
| `xgen-node/src/main.rs::node_pipe_stub` | The five stub call-sites to flip to real delegators |
| `xgen-node/src/desktop.rs` | Resident-desktop spawn point — pipe server needs to start here |
| `xgen-node/src/app.rs::run_node` | Headless resident (`--service`) — pipe server needs to start here too |

---

## Scope

**In scope for M2:**
1. Port `start_pipe_server` skeleton from Client to Node (`xgen-node/src/pipe.rs` or extend `xgen-node-lib::app`). Same accept loop, same shutdown_rx pattern, same control-command short-circuit.
2. Implement the four control-command handlers on the Node side: `__PING__`, `__HEALTH__`, `__STOP__`, `__RELOAD_CONFIG__`. PING is timestamp-trivial; HEALTH should surface real Node state (number of connections, spaces hosted, federated peers, lifecycle state); STOP exits the process (mirrors Client pattern); RELOAD_CONFIG stays as `NOT_IMPLEMENTED` for M2 (real config reload is a separate concern — name the milestone honestly).
3. Implement `__BATCH__` dispatch on the Node side. The Node command set for batch is whatever Node subcommands make sense to run remotely (`status`, `connections`, `peers`, `spaces`, `identity list`, `version`). Out of scope: any Node subcommand that *mutates* state (none today) or that needs interactive input.
4. Wire the pipe server into both Node resident modes — `desktop::run` (Tauri + WS) and `app::run_node` (--service headless). Same pipe name derived from `--instance` (D-043).
5. Wire the five Node-side pipe-client helpers: `cmd_ping`, `cmd_health`, `cmd_stop`, `cmd_reload_config`, `cmd_batch` — mirroring the Client helpers exactly.
6. Flip the five `node_pipe_stub("--xxx")` call-sites in `xgen-node/src/main.rs` to delegate to the real helpers.
7. PID-file write — already happens in `run_node` (J-069 Phase 4); pipe server should reuse the same data_dir for the pipe-name derivation. No new files needed.

**Out of scope (deferred):**
- Real config reload — `__RELOAD_CONFIG__` responds with the same `NOT_IMPLEMENTED` message the Client does. Reload semantics need their own design pass (which fields are reloadable? does the WS listener restart? do active connections drop?). Separate milestone.
- M3 — AI Client deployment (DM control plane, AI-specific config, designated operator) — completely separate concern.
- Multiparty test redesign — paused per M1 task file; resumes after M2/M3 land.
- Appendix F comprehensive example rewrite — still deferred (waits for M2/M3 stability).
- AttachConsole hybrid-app polish (desktop console flash) — cosmetic, deferred.

---

## Decisions to surface (likely needs Joe's call before / during implementation)

These are flagged so the next session knows what to ask about rather than guess:

1. **`__BATCH__` Node command set.** Which Node subcommands are allowed via pipe-batch? Read-only ones are obviously safe (`status`, `connections`, `peers`, `spaces`, `identity list`, `version`, `whoami`). The Node has no protocol-action subcommands today (no `register` / `send` etc — those are Client-only). So Node `__BATCH__` is essentially "read state and print" — useful for monitoring scripts. Confirm scope.
2. **`__HEALTH__` content.** Client returns `HEALTHY pid=<n>`. Node could return richer info (connection count, peer count, lifecycle state, uptime, last-error-if-any). One-line constraint. Pick a format.
3. **`__STOP__` behaviour during accept loop.** Client `__STOP__` calls `std::process::exit(0)` in the pipe handler. Node could do the same OR signal the WS accept loop to break cleanly. Process-exit is simpler and works in both desktop and --service modes; clean shutdown is post-M2 polish (analogous to AttachConsole on the desktop console flash).
4. **`__RELOAD_CONFIG__` honest stub.** Same `NOT_IMPLEMENTED` message as Client? Or something more specific to Node (e.g. "config reload would require restarting the WS listener — out of scope for M2")? Pick wording.
5. **Pipe server visibility in `--health`.** When the pipe server itself is queried via `--health`, does the response include "pipe alive" info? Likely yes since the response itself proves it. Decide format.

---

## Implementation steps (recommended sequence)

### Phase 0 — Pre-flight
1. **Baseline.** `cargo test --workspace --release` — confirm 391. Quote actual output in journal.

### Phase 1 — Port the pipe-server skeleton

2. **Create `xgen-node/src/pipe.rs`** (or extend `xgen-node-lib::app` — Claire's call based on tidiness). Port `batch::start_pipe_server` from Client. Same shape:
   - `pub async fn start_pipe_server(pipe_name: String, data_dir: PathBuf, shutdown_rx: watch::Receiver<bool>) -> ()`
   - Same accept loop with `first_pipe_instance(true)` on first iteration
   - Same control-command short-circuit before falling through to batch path
   - Same `__PING__` / `__HEALTH__` / `__STOP__` / `__RELOAD_CONFIG__` handlers (specifics per Joe's dispositions above)
   - Same `__BATCH__` collect-lines-until-`__END__` pattern, dispatching to a Node-specific `dispatch_line`

3. **Add `pub fn pipe_name(instance_label: Option<&str>)`** mirroring Client's:
   - `None` → `\\.\pipe\xgen-node`
   - `Some("n1")` → `\\.\pipe\xgen-node-n1`

### Phase 2 — Implement the Node command set for __BATCH__

4. **Build Node `dispatch_line`.** Uses the same pattern as Client: shlex-tokenize, parse via the canonical `Cli` (subset of NodeCommand variants allowed), dispatch to `app::cmd_*`. Allowed initially: `status`, `connections`, `peers`, `spaces`, `identity list`, `version`, `whoami`. Reject anything else with "not supported in pipe-batch mode."

5. **PID file already exists** (J-069 Phase 4). Reuse without changes.

### Phase 3 — Wire into resident modes

6. **`xgen-node/src/desktop.rs`** — spawn the pipe server task alongside the run_node task. Same `_pipe_shutdown_hold` pattern as `xgen-client/src/service.rs::run` (J-071 bug — the watch-channel sender must live until block_on ends, not just the cfg block). The comment in `xgen-client/src/service.rs` explains the rule.

7. **`xgen-node/src/app.rs::run_node`** — spawn the pipe server task before entering the accept loop. Same pattern.

### Phase 4 — Wire the Node-side pipe-client helpers

8. **Create `xgen-node/src/pipe_client.rs`** (or extend `xgen-node-lib::pipe`). Mirror the Client helpers exactly:
   - `pub fn cmd_ping(pipe_name_str: &str) -> i32`
   - `pub fn cmd_health(pipe_name_str: &str) -> i32`
   - `pub fn cmd_stop(pipe_name_str: &str) -> i32`
   - `pub fn cmd_reload_config(pipe_name_str: &str) -> i32`
   - `pub fn cmd_batch(raw_path: &str, pipe_name_str: &str, instance_label: Option<&str>) -> i32`
   - Shared `async fn pipe_send_control(pipe_name_str: &str, control_token: &str) -> Result<String>`

9. **Flip the stubs in `xgen-node/src/main.rs`.** Replace each `exit_with_result(node_pipe_stub("--xxx"))` with the real pipe-client helper invocation. Delete `node_pipe_stub` once unused.

### Phase 5 — Verification

10. **`cargo test --workspace --release`** — green at 391.

11. **Smoke against a running Node.** Per-flag end-to-end against both desktop and --service modes. Suggested smoke script: model after the J-072 walkthrough script (`/c/Users/Joe/AppData/Local/Temp/phase5_matrix.sh`) but Node-side this time:
    - Start `xgen-node --service`
    - `xgen-node --pid` (already works pre-M2 from J-069)
    - `xgen-node --ping` → `pong: <n> ms`
    - `xgen-node --health` → one-line summary with real state info
    - `xgen-node --batch foo.xgb` → executes the batch's Node commands
    - `xgen-node --reload-config` → server replies `NOT_IMPLEMENTED`
    - `xgen-node --stop` → resident terminates

12. **Update the Phase 5 matrix cells.** Cells N14, N16, N17, N18, N19 should flip from "stub message + exit=1" to real handlers. Re-run the matrix walkthrough; expect 49/49 PASS without the M2 stub asterisks.

---

## Definition of Done

- [ ] Baseline captured (391 from J-073).
- [ ] `xgen-node-lib::pipe` (or equivalent) hosts the pipe server with all four control commands implemented + `__BATCH__` Node command dispatch.
- [ ] `pipe_name(instance_label)` derives `\\.\pipe\xgen-node[-<label>]` per D-043.
- [ ] Pipe server wired into both Node resident modes (desktop spawn + `--service` spawn). Same `_pipe_shutdown_hold` pattern as J-071's bug-fix.
- [ ] Five Node-side pipe-client helpers implemented (`cmd_ping`, `cmd_health`, `cmd_stop`, `cmd_reload_config`, `cmd_batch`) mirroring the Client helpers.
- [ ] `node_pipe_stub` deleted; all five `main.rs` call-sites now delegate to real helpers.
- [ ] `cargo build --release --workspace` clean (no new warnings — 44 pre-existing in stress-test code).
- [ ] `cargo test --workspace --release` green at 391.
- [ ] End-to-end smoke against a running Node: all five flags produce expected output and exit codes. Quote actual output in the journal entry.
- [ ] Phase 5 matrix re-run: N14/N16/N17/N18/N19 now real PASS (not stub-message PASS).
- [ ] `JOURNAL.md` entry (J-074?) quoting verification output.
- [ ] `tasks/M2_NODE_PIPE_SERVER.md` header flipped from `PENDING` to `COMPLETED`.
- [ ] `CLAUDE.md` updated to reflect M2 done; the Node side of the deployment model now matches the Client side.

---

## Behaviour rules reminder (from CLAUDE.md)

- **Rule 1** — Never fabricate results. Real output only.
- **Rule 2** — Show actual output. Quote terminal output verbatim in the journal.
- **Rule 3** — Stop and report when a tool fails.
- **Rule 4** — Write the journal entry last, after verification is confirmed.
- **Rule 5** — Never invent numbers. Test counts from `cargo test` only.
- **Rule 6** — When in doubt, do less and ask. Five decisions are pre-flagged above; expect more during implementation.
- **Rule 7** — Definition of Done is a checklist, not a formality.

If `__RELOAD_CONFIG__` ends up wanting to do real work mid-M2 (i.e. someone realises the Node WS listener could be restarted cleanly), **stop and ask Joe** — that's M2+1 scope creep and deserves its own milestone.

---

*End of M2 task file.*
