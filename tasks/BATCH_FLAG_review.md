# `--batch` Flag — Design Review and Improvement Proposal
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools. This review was authored by an AI working as the *user* of the `--batch` surface, drawing from real friction encountered during MULTIPARTY_S1 (J-067).  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this file is

A design review of the current `--batch` flag implementation, written from the perspective of an AI driving XGen operations against the running `xgen-node-app` and `xgen-client-app` Tauri executables. It identifies what works, what's weak, and proposes concrete improvements ordered by leverage.

The motivation is twofold. First, the `--batch` flag is explicitly the AI control surface — humans drive the Tauri GUI, AIs drive the named-pipe protocol. Second, J-067 (MULTIPARTY_S1) revealed that the current `--batch` design forced a CLI shortcut that bypassed the deployment shape, and that two `--batch` issues (F-003 / F-004) emerged from architectural drift between parallel implementations. Both pressures justify a focused review.

Cross-reference: the original spec is `docs/tests/BATCH_FLAG_ph2.md`. This document is a layer on top — it does not deprecate the original.

---

## Context — what `--batch` is today

### Architecture (per `BATCH_FLAG_ph2.md`, D-043)

```
xgen-client-app.exe --instance m1a                       ← long-lived (Tauri window + pipe server)
   └─ Tauri runtime + start_pipe_server task
       └─ named pipe \\.\pipe\xgen-client-m1a
           └─ accept() loop: for each connection, read lines, dispatch each via batch::dispatch_line
               └─ for each line: parse via clap (BatchCli), route to exec_* in batch.rs
                   └─ each exec_* opens its own WebSocket → Node → closes

xgen-client-app.exe --instance m1a --batch script.xgb    ← short driver
   └─ early-exit in main.rs detects --batch, calls batch::run_batch_client(file, pipe_name, label)
       └─ open \\.\pipe\xgen-client-m1a
       └─ write each line of the .xgb, read OK\n / ERROR: …\n per line
       └─ send __END__, exit
```

The Node side mirrors the same shape: `xgen-node-app.exe --instance m1node` is long-lived (systray icon, no window in `--service` mode), `xgen-node-app.exe --instance m1node --batch X.xgb` is the short driver (Node has fewer batch commands than the client — mostly inspection).

### Codebase touch-points (current)

- `xgen-client/src/batch.rs` — pipe server, `dispatch_line`, `exec_*` handlers, `get_dag_tips` (copy A).
- `xgen-client/src/main.rs` — CLI flow, `run_batch_file`, `cmd_*` handlers, `get_dag_tips` (copy B), `--batch` short-circuit in the Tauri binary's main.
- `xgen-client/src-tauri/src/main.rs` — spawns `start_pipe_server` in a Tauri async task, intercepts `--batch` for early exit.
- `xgen-node/src/main.rs` — symmetric on the Node side.

---

## Where it works well

1. **The conceptual shape is right.** A long-lived process + remote control via IPC is exactly what an AI agent needs. Better than a REST API (no auth dance per request), better than repeated CLI invocations (no process spawn cost, no auth re-handshake per command), better than SSH (no shell). The `--batch` flag is a coherent control plane.

2. **Per-instance isolation via labels.** `\\.\pipe\xgen-client-{label}` cleanly separates instances. Multiple Tauri instances coexist on one machine without naming collision. The data-dir layout (`<exe_dir>/instances/<label>/`) is similarly clean.

3. **One pipe per binary kind, multiple sessions over time.** The pipe server accepts repeated connections — the same `.xgb` driver can connect again, or a different driver can connect. This is the right composability.

4. **Windows-native.** Named pipes are the correct primitive on Windows; cross-platform abstractions would have introduced complexity for no benefit (Phase 1 is Windows-first).

5. **Exit codes communicate batch outcome clearly.** 0 = all succeeded, 1 = first failure halts, 2 = file-not-found / parse error. Adequate for human users and CI.

---

## Where it falls short

Each issue below is real friction encountered during J-067 (MULTIPARTY_S1) or anticipated for subsequent multiparty tests.

### 1. Per-command WebSocket churn

**Symptom.** Each `exec_send` opens a fresh WS, authenticates, calls `sync_request` to fetch `prev_events`, sends the event, closes. For 100 sends, that's 100 handshakes × 100 auth challenge-responses × 100 sync_request round trips.

**Consequences.**
- **Slow.** S1 P2 took 60 seconds for 300 messages (~600 ms per send). Most of that is handshake / auth / sync_request overhead, not protocol work.
- **Hides real-time fan-out.** Fan-out events the Node pushes back after the send arrive *after* the connection has closed. The driver can never observe real-time fan-out — only the eventually-consistent state via subsequent `sync_request` history pulls.
- **Causes F-003 / F-004.** Each `exec_send` independently re-derives `prev_events` via `get_dag_tips` against a sync_request stream — which is exactly where the cross-Space tip leakage bug lived. With a persistent connection holding session state, `prev_events` is just "the last event I observed in this Space" — no sync_request, no bug.
- **Doesn't match real operational mode.** A real user's Tauri client doesn't reconnect on every keystroke. The deployed shape is one persistent connection per (instance, home Node) pair, multiplexed across all Spaces the instance is a member of.

**Proposal.** The long-lived Tauri instance should hold **one** authenticated WebSocket connection to its home Node, opened at startup (after SETUP completes) and re-established on disconnect with the existing reconnection backoff (spec 3.3.6). Every batch command dispatched through the pipe runs over that connection. Per-Space `prev_events` is tracked locally as state inside the running instance.

This is the single highest-leverage change. It eliminates F-003 / F-004 entirely (no per-`send` `get_dag_tips`), drops 100-send latency from ~60 s to under 1 s, enables real-time event observation (see point 3), and matches the deployment shape.

### 2. Unstructured replies

**Symptom.** The pipe protocol replies `OK\n` or `ERROR: <message>\n` per dispatched line. The actual data produced by the command (created `space_id`, new `event_id`, etc.) is only available by scraping the running instance's log file.

**Consequences.**
- **Two-pass `.xgb` execution is mandatory** for anything that depends on previous commands. S1 had to: run alice's `create-space`, scrape the log for `Space ID:`, generate a new `.xgb` with that literal ID, run it. Repeat for `create-room`. This is exactly what `@last_space` was meant to solve in spec — but absent backrefs (point 4), the workaround is log-scraping.
- **AI-hostile.** AIs work best with structured data. Plain-text `OK` is fine for humans tailing a log; it's the worst case for tool output parsing.
- **Brittle log parsing.** The current workaround relies on stable log-line formats (`Space ID: <id>`). Any log refactor breaks every test.

**Proposal.** Each dispatched line returns a single JSON object on the pipe:

```json
{"status":"ok","cmd":"create-space","data":{"space_id":"xgen://hash/sha256:...","event_id":"xgen://hash/sha256:...","timestamp":"2026-05-16T..."}}
{"status":"ok","cmd":"send","data":{"event_id":"xgen://hash/sha256:...","accepted":true}}
{"status":"error","cmd":"send","error":{"code":4002,"name":"predecessor_timeout","message":"..."}}
```

One JSON object per line, terminated by `\n`. The driver parses it; humans can pipe through `jq` or any pretty-printer. The existing `OK\n` / `ERROR: ...\n` could remain as a fallback compatibility format selectable via a flag (e.g. `--reply-format=jsonl|legacy`), but the AI-default should be JSONL.

### 3. No real-time event observation channel

**Symptom.** A long-lived Tauri instance receives fan-out events in real time (per F-001 implementation in J-067). A batch driver currently has no way to subscribe to those events; it can only call `history` after the fact, which uses `sync_request` to fetch the eventually-consistent DAG state.

**Consequences.**
- **The S1 pairing-table verification is a reconstruction, not an observation.** S1's expected table specifies real-time delivery (`✔ (fan-out)` means "this event arrived at this client in real time as the Node fanned it out"). Without an observation channel, the only thing we can verify is "the event eventually appears in history" — strictly weaker.
- **No timing/latency metrics for real-time delivery.** S1 P2's latency table ("median Outbound→Inbound delivery time") cannot be filled in honestly without a real-time receive channel.
- **The temperature mechanism design (Ch6 §6.12) and pacing queue design (Ch6 §6.14) assume the client observes events in real time.** Those features can't be tested end-to-end without this.

**Proposal.** A streamed observation surface alongside the command surface. Two viable shapes:

- **Multiplexed on the same pipe.** Each JSONL line carries a `type` tag: `{"type":"reply",...}` for command replies, `{"type":"event",...}` for fan-out events. The driver demultiplexes.
- **Sister pipe.** `\\.\pipe\xgen-client-{label}.events` — pure event stream, line-delimited JSON, separate connection.

I lean toward multiplexed (one less moving part, ordering with respect to commands is well-defined). The trade-off is parser complexity in the driver: it needs to interleave reply-waiting with event-stream handling. For an AI driver, that's trivial; for a hand-written human tool, it's slightly harder.

Per-Space filtering on subscribe (`{"cmd":"subscribe","space":"xgen://..."}`) is a useful follow-on so a driver doesn't drown in events from every Space the instance is a member of.

### 4. No backreferences / variable bindings

**Symptom.** The original `BATCH_FLAG_ph2.md` proposed `@last_space` / `@last_room` placeholders. The dispatcher does not implement them. Every `--space` / `--room` argument must be a literal `xgen://hash/sha256:...` ID. For scenarios that create a Space and Room, then act on them, this forces the two-pass dance described in point 2.

**Consequences.**
- **Test scripts are not self-contained.** Every scenario file in `docs/tests/` that involves creating-then-using a Space needs an external mechanism to substitute IDs after creation.
- **Re-runs are painful.** Each fresh test run produces new IDs; the substituted scripts have to be regenerated each time. (S1 generated `_pass1`, `_pass1b`, `_pass2` and then a separate set when bob/carol joined Room.)

**Proposal.** Per-pipe-session variable map maintained by the long-lived instance.

```
register --name alice                   # creates @last_identity
create-space --name "Test"              # creates @last_space, @spaces["Test"]
create-room --space @last_space --name general
                                        # creates @last_room, @rooms["general"]
send --space @last_space --room @last_room --text "hello"
```

Reserved names: `@last_identity`, `@last_space`, `@last_room`, `@last_event`. Explicit bindings: `let X = @last_room` (optional, for readability). Variable expansion happens server-side, in the long-lived instance — the driver doesn't need to know IDs. Variables are session-scoped (cleared when the pipe connection closes), unless an explicit `persist` keyword is used.

This eliminates the two-pass dance entirely. A test's `.xgb` becomes a single deterministic script regardless of what IDs it produces at runtime.

### 5. Duplicate command implementations (cmd_* vs exec_*)

**Symptom.** Every batch-compatible command has two implementations:

| Command | CLI handler (`main.rs`) | Tauri-pipe handler (`batch.rs`) |
|---|---|---|
| `register` | `cmd_register` | `exec_register` |
| `create-space` | `cmd_create_space` | `exec_create_space` |
| `create-room` | `cmd_create_room` | `exec_create_room` |
| `invite` | `cmd_invite` | `exec_invite` |
| `join` | `cmd_join` | `exec_join` |
| `send` | `cmd_send` | `exec_send` |
| `history` | `cmd_history` | (none — `history` is not in batch) |
| `get_dag_tips` (helper) | duplicated | duplicated |
| `whoami` | `cmd_whoami` | `exec_whoami` |
| `status` | `cmd_status` | `exec_status` |

The CLI `--batch` path (in `xgen-client/src/main.rs::run_batch_file`) dispatches to the `cmd_*` set. The Tauri-pipe path (in `xgen-client/src/batch.rs::dispatch_line`) dispatches to the `exec_*` set. **They are different code that happens to do similar things.**

**Consequences.**
- **Fixes drift.** F-003 was applied to `exec_*::get_dag_tips`. The next test run hit F-004 — the *other* `get_dag_tips`. Future bug fixes will keep paying this tax until the two sets are merged.
- **Verification gaps.** A CLI-only test (like the J-067 S1 run) exercises only `cmd_*`. The deployment-shape `exec_*` is unverified. The opposite is also true (a Tauri-only test leaves `cmd_*` unverified).
- **Cognitive overhead.** Adding a new command means adding it in two places; missing one is silent.

**Proposal.** Extract a single set of command handlers, parameterised by execution context, into a shared module. Strawman:

```rust
// xgen-client/src/ops.rs (or xgen-client-common::ops)

pub struct OpContext<'a> {
    conn: &'a mut Connection,         // either fresh-per-call or persistent
    data_dir: &'a Path,
    runtime_state: &'a mut SessionState, // holds @last_*, persistent WS, etc.
}

pub async fn send(ctx: &mut OpContext, args: SendArgs) -> Result<SendResult> { ... }
pub async fn create_space(ctx: &mut OpContext, args: CreateSpaceArgs) -> Result<CreateSpaceResult> { ... }
// ...
```

- CLI `--batch` path: creates a fresh `OpContext` with a one-shot connection per call.
- Tauri pipe path: creates the `OpContext` once at session start, persists the connection, retains `runtime_state` across calls.
- Both call the same `send` / `create_space` etc. functions.

This is the structural fix that makes F-003 / F-004-style drift impossible to recur. It's a non-trivial refactor (touches every command handler) but pays for itself the first time it prevents a duplicate-fix bug.

### 6. Lifecycle-state-blind error reporting

**Symptom.** Errors surface in their lowest-level form. "keypair not found" is what a failed `register` says when the instance is in SETUP and has no keypair. "connection refused" is what a failed `send` says when the home Node is down. The driver has to interpret these into operational reality.

**Consequences.**
- **AI agents have to reason from raw error text.** That works but it's brittle (error strings drift). It also doesn't expose information the long-lived instance has that the driver doesn't: the instance's current lifecycle state, last connect attempt, current home Node, last successful send.
- **No "is it ready yet?" affordance.** A driver wanting to wait until the instance is `READY` before dispatching has to poll the state file every 5 seconds (the file's write cadence). A query would be cleaner.

**Proposal.** Add lifecycle metadata to error responses, and add a `state` command:

```json
{"status":"error","cmd":"send","error":{"code":"INSTANCE_NOT_READY","instance_state":"setup","hint":"register first or complete SETUP in the UI"}}

// New command:
{"cmd":"state"}
< {"status":"ok","cmd":"state","data":{"lifecycle":"ready","home_node":"xgen://...","connected_since":"...","known_spaces":[...]}}
```

The `state` command is essentially a structured `whoami`+`status`+`spaces` rolled into one, addressed at the live instance state rather than the on-disk state file.

---

## The `--batch` flag as the AI surface

Up to here I've treated each issue as a discrete improvement. Stepping back: the current `--batch` is implemented as a **fire-and-forget script runner**. The `.xgb` file is a static sequence of commands; the driver runs them, collects OK/ERROR, exits.

What an AI agent actually wants is a **persistent control session**:

- Open a session to a long-lived instance.
- Issue commands incrementally (one in, one out).
- Observe events as they arrive.
- React to state.
- Close the session.

The static `.xgb` is just one client of that session protocol — a non-interactive one. A future Claude-driven MCP server bridging XGen to a chat AI would be another client, wearing the same surface. So would an MCP-style live agent that lives in a Space and moderates it. The same primitives serve all of them.

That framing also makes the priorities clear:

| Improvement | Priority | Reason |
|---|---|---|
| 1. Persistent WS in the long-lived instance | **highest** | Unblocks everything else (real-time fan-out, no per-send sync, no F-003/F-004 class of bugs, matches deployment) |
| 5. Single source of truth for handlers | **high** | Prevents drift; required for any non-trivial refactor not to regress |
| 2. Structured JSONL replies | **high** | The AI-ergonomic primitive; required for backreferences (point 4) to be useful programmatically |
| 4. Backreferences / variable bindings | **medium-high** | Eliminates two-pass dance entirely; relies on persistent session (point 1) and structured replies (point 2) |
| 3. Event observation channel | **medium-high** | Required to verify real-time fan-out (S1 onward). Best implemented after point 1 (persistent connection holds the subscription) |
| 6. Lifecycle-aware errors + `state` command | **medium** | Quality of life; not blocking |

---

## What to do next

A sensible sequence:

1. **Re-run MULTIPARTY_S1 through the Tauri path** with the current `--batch` implementation as-is (no improvements yet). This validates the deployment shape against today's binary and surfaces any Tauri-shell-specific issues that the CLI bypassed. See `tasks/MULTIPARTY_S1_tauri_rerun.md` for the runbook.
2. **Decide which improvements to ship before continuing with S2.** My recommendation: at minimum, points 1 (persistent WS) and 5 (unify handlers). These two together close F-003 / F-004-class bugs, make S2's "concurrent send" scenario meaningfully testable, and align the implementation with the deployment shape. Points 2 / 3 / 4 / 6 can follow as a second pass.
3. **Add structured replies (point 2) and the event observation channel (point 3) before MULTIPARTY_S4** (the realistic chat-room scenario), since S4's verification depends on real-time event timing across multiple Nodes.
4. **Backreferences (point 4) and lifecycle-aware errors (point 6)** can land anywhere; they're additive.

---

## Out of scope for this review

- The protocol-level fan-out implementation in `xgen-node-lib::fanout` (F-001) — separate work, already done in J-067.
- The Tauri lifecycle state machine itself (Ch2 / Appendix E) — separate, already specified.
- A full MCP server bridging XGen to a chat AI — future, would consume the surface this document describes.
- Cross-platform pipe abstractions — Windows-first is fine for Phase 1/2.

---

*End of `BATCH_FLAG_review.md`*
