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

## Baseline metrics protocol

Decision recorded with Joe (2026-05-16): run the full Multiparty suite (S1–S5) twice — first with the present `--batch` (= baseline / "A"), then again after the improvements ship (= "B"). Compare statistically. This turns the improvement work into A/B evidence rather than "trust me, it's better now," and protects against silent regressions.

The metric set is defined here so present-version runs capture exactly the same numbers improved-version runs will. If any scenario discovers a metric not in this list during the present pass, **add it here and capture it from that scenario forward** — don't quietly diverge.

### Metric set per scenario

| Category | Metric | Recording shape | Notes |
|---|---|---|---|
| Outcome | PASS / FAIL / PASS-with-caveat | one cell | The verification baseline. Captured in the existing "Verdict" sections. |
| Throughput | Stress-phase wall-clock (s) | one number | First send dispatched → last batch exited. |
| Throughput | Effective messages/sec (accepted ÷ wall-clock) | one number | Normalised throughput. |
| Latency | Outbound → Inbound per-message delay (median / p95 / max, ms) | three numbers | **Unmeasurable under present `--batch` for most cases** — record as "—" with a one-line reason (e.g. "real-time fan-out not observable; connection closes before push arrives"). Present pass establishes "unmeasurable" as a baseline data point itself. |
| Loss | Authored ÷ accepted ratio (e.g. "294/300 = 98%") | one fraction + % | Reliability indicator. |
| Loss | Pending-buffer timeouts during the run | one count | Distinguishes "lost in transit" from "rejected at DAG validation." |
| Errors | ERROR-level log lines, per binary | one count per binary | Should be 0; if non-zero, classify in a side note. |
| Errors | WARN-level log lines, per binary, classified | one count + classification | Some WARN is fine (graceful shutdown). |
| DAG integrity | Duplicate `event_id`s anywhere | one count | Should be 0. |
| DAG integrity | Orphaned events at end of run (events with unknown `prev_events`) | one count | Should be 0. |
| Observability | % of pairing-table `✔` cells that are real-time-observed vs reconstructed-via-sync_request | one % | **The headline metric.** Present-version will be near 0% for most cells (everything reconstructed via `history`). Improved-version should approach 100% (events arrive at currently-connected clients in real time). This single metric tells the improvement story most clearly. |
| Setup cost | Wall-clock from "start scenario" to "verification PASS recorded" | one number | Includes the two-pass-script / ID-substitution / log-scraping friction. Should drop substantially after backreferences + structured replies land. |

### Statistical shape

For each scenario:

- **1 verification run** — must PASS, records the integrity metrics (loss, errors, DAG integrity, outcome).
- **n=3 measurement runs of the stress / concurrent phase only** — record throughput, latency distribution, observability % per run, then aggregate (min / median / max + indicator of variance). If a metric shows wild variance, bump that specific scenario to n=5; document the bump.

Total cost: ~10–15 min per scenario × 5 scenarios = ~1 hour per full pass. Two full passes (baseline + improved) = ~2 hours of dedicated benchmarking, plus the verification runs (which double as PASS/FAIL).

### Recording convention

Each `MULTIPARTY_S{N}_findings.md` gains a **"Metrics"** section, structured as a two-column table: "Present version (baseline)" | "Improved version". The baseline column is filled during the present pass; the improved column is filled during the improved pass.

A short summary table (all 5 scenarios × all metrics × both versions) gets compiled into the journal entry that closes the improvement work — useful for the spec / architecture conversation that follows.

### What to record when a metric is unmeasurable in the present version

Use the literal string `—` (em dash) in the cell, followed by a one-line reason in parentheses. Example:

```
| Latency (median) | — (no real-time observation under present --batch — see BATCH_FLAG_review.md §3) | 23 ms |
```

This makes the "improvement story" visible in the table itself — every `—` in the baseline column that becomes a number in the improved column is one capability the improvements unlocked.

### Friction log (append-only)

Each scenario surfaces new friction with the present `--batch`. Append observations here as they're discovered during the S1 Tauri rerun and S2–S5 runs; they refine the improvement priorities at the end of the present pass.

Format: one bullet per observation, with `[Sn]` tag identifying which scenario surfaced it.

- _(empty for now — populate during the multiparty runs)_

---

## What to do next

The revised sequence (after the 2026-05-16 discussion with Joe):

1. **Re-run MULTIPARTY_S1 through the Tauri path** with the current `--batch` implementation as-is. This validates the deployment shape against today's binary, surfaces any Tauri-shell-specific issues that the CLI bypassed, and captures the S1 baseline metrics per the protocol above. See `tasks/MULTIPARTY_S1_tauri_rerun.md` for the runbook.
2. **Run MULTIPARTY_S2 → S3 → S4 → S5 through Tauri `--batch` as-is.** Each scenario captures the metric set above; each session appends observations to the "Friction log" section. See `tasks/MULTIPARTY_S2_to_S5_present_pass.md` for the cross-scenario runbook.
3. **At the end of the present pass:** review the friction log, refine the improvement priorities in the six points above based on observed (not speculated) pain.
4. **Ship the agreed improvements.** Minimum: point 1 (persistent WS) and point 5 (unified handlers). Likely additionally needed based on S4: point 3 (event observation channel). Other points as the friction log suggests.
5. **Re-run S1 → S5 with the improved `--batch`** (the "B" pass). Capture the same metrics. The two-column tables in each findings file fill in completely.
6. **Compose the closing journal entry** — summary table of all metrics across all scenarios in both versions, plus the qualitative story of what changed.

---

## Out of scope for this review

- The protocol-level fan-out implementation in `xgen-node-lib::fanout` (F-001) — separate work, already done in J-067.
- The Tauri lifecycle state machine itself (Ch2 / Appendix E) — separate, already specified.
- A full MCP server bridging XGen to a chat AI — future, would consume the surface this document describes.
- Cross-platform pipe abstractions — Windows-first is fine for Phase 1/2.

---

*End of `BATCH_FLAG_review.md` (original Clair review).*

---

# Chat Claude addendum — `--aicontrol` design (2026-05-17)

> **Status:** ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-17  
> Language: English  
> Author: JozefN (architectural commitment) + Chat Claude (technical detail)  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools. This addendum was authored by Chat Claude as a follow-up to Clair's review above, after Joe locked the architectural commitment (split `--batch` from a new AI-driver surface) and the flag name `--aicontrol`. All technical decisions in this addendum are Chat Claude's and Clair's call — Joe delegated them explicitly to avoid per-detail approval bottlenecks. Joe retains review authority only on decisions that collide with a bigger XGen unit (protocol surface, Tauri lifecycle, federation, etc.).  
> License: BSL 1.1 (converts to GPL upon project handover)  

## What this addendum is

Clair's review above identified six concrete improvement points and re-framed the diagnosis as a control-plane category mismatch (fire-and-forget script runner vs persistent control session). Chat Claude's review of that review (in conversation, 2026-05-17) concurred with the diagnosis, pushed back on the multiplexed-vs-sister-pipe choice for the event channel, recommended named bindings be mandatory rather than `@last_*`-convenience, and flagged four missing concerns (timeout/cancellation, pipe-level authentication, replay safety, `SpaceState` map in session state).

Joe then made one architectural commitment: rather than evolve `--batch` to meet the AI driver's needs (and degrade the human-readability that was its original design goal), introduce a **second flag** `--aicontrol` designed from the start for AI drivers. `--batch` is preserved verbatim for humans and human-readable automation. The architectural commitment is recorded as D-066. This addendum captures the technical detail under the umbrella of that decision.

**Scope of this addendum.** Everything below is technical implementation detail for `--aicontrol`, deliberately decided by Chat Claude and Clair without per-item Joe approval. Items that *would* require Joe's input (because they collide with a bigger XGen unit) are flagged inline with **[Joe-flag]** so future reviewers can audit the boundary.

## 1. Surface shape — one decision at a time

### 1.1 Naming and invocation

Locked by Joe:

- Flag name: `--aicontrol`. Audience-visible at the CLI surface.
- Binary: `xgen-client` only in this milestone. The `xgen-node --aicontrol` question is deferred to the design phase that schedules this work, on the rationale that the Node-side use cases (federation introspection, hosted-Space state queries, real-time event tap for compliance modules) are different enough from the Client-side use cases (driving an Identity through Space participation) that the two surfaces may diverge in command verb set. Better to design Client `--aicontrol` first, validate it, then re-evaluate whether the Node side wants a structurally similar surface or something else.

Technical decisions taken below this line:

- **Mode, not script-runner.** `xgen-client --aicontrol` opens a persistent control session over a dedicated named pipe. No `--aicontrol script.aib` file-loading flag; scripts are fed via stdin redirection if needed. The session lives as long as the pipe connection lives.
- **File extension** `.aib` (for *AI batch*) is reserved as a *convention* for input files driven via redirection, but the runtime does not enforce or check extensions — the pipe sees bytes, not files. This matches the principle that `--aicontrol` is a live session, not a script runner.

### 1.2 Pipe naming — sister pipe

The existing legacy pipe stays:

```
\\.\pipe\xgen-client[-<instance>]
```

A new sister pipe lands alongside it:

```
\\.\pipe\xgen-client[-<instance>].aicontrol
```

Why sister pipe and not multiplexed-same-pipe:

- The two protocols are genuinely different (line-oriented text vs JSONL). Multiplexing them by first-line-sniffing introduces a parse-the-first-byte branch that adds nothing.
- Sister pipe lets `--batch` and `--aicontrol` evolve independently. Future changes to one cannot regress the other.
- An audit-conscious operator can lock down `--aicontrol` access with a more restrictive ACL than `--batch` without affecting the legacy surface.
- Two pipe names to remember is trivial cost; the deployment shape is symmetric (the legacy pipe is already named, sister pipe just appends `.aicontrol`).

This extends D-043 (named pipe naming convention). Recorded in D-066's relationship-to-other-decisions table.

### 1.3 Concurrency model — strictly serial per connection, multiple connections allowed

One pipe connection = one in-flight command at a time. The driver fires command N, waits for the reply, then fires command N+1. No request IDs, no out-of-order reply matching, no fancy correlation.

Drivers needing concurrency open multiple pipe connections. The long-lived instance accepts repeated connections (per D-043's composability principle); each connection is an independent session with its own variable-binding namespace.

Why strictly serial:

- Simpler protocol. No request ID field on every command, no reply demultiplexing in the driver, no "which command failed?" ambiguity.
- Matches the actual workload shape. AI drivers think sequentially — "create the Space, then create the Room, then send a message." The natural model is sequential.
- Event observation runs on its own channel (§2), so events don't serialize behind commands.
- Drivers wanting parallelism (e.g. fan multiple sends across different Spaces simultaneously) open multiple connections. The instance handles each in parallel without protocol-level concurrency in any one connection.

## 2. Event observation — dedicated event pipe

A third pipe surface, alongside the legacy `--batch` pipe and the `--aicontrol` command pipe:

```
\\.\pipe\xgen-client[-<instance>].events
```

Why a third pipe rather than multiplexing on the `--aicontrol` command pipe:

- **Back-pressure isolation.** A slow event consumer (e.g. an AI driver doing LLM inference between event batches) blocks only its own event stream, not the command channel.
- **Subscribe/unsubscribe model.** The first message on the event pipe is `{"cmd":"subscribe","filter":{...}}`. The filter shape:
  ```json
  {"spaces":["xgen://hash/sha256:abc...","xgen://hash/sha256:def..."], "event_types":["message.text","state.*"]}
  ```
  Either field may be omitted. Empty `spaces` means "all Spaces this Identity is in." `event_types` accepts wildcards (`state.*`). Subsequent events are streamed as JSONL until the connection closes.
- **The same Identity can have one command session and zero-to-N event subscriptions.** A driver wanting separate-handler-per-Space opens one event pipe per Space.

Event record shape:

```json
{
  "type":"event",
  "event":{ /* full Event object as per Appendix I */ },
  "received_at":"2026-05-17T...",
  "space_id":"xgen://hash/sha256:..."
}
```

The `received_at` field is the instance's local-time observation timestamp — distinct from the Event's `timestamp` (sender clock) and from any future home-Node delivery timestamp. The S1 latency metric (Outbound → Inbound delivery time) is `received_at - sender's timestamp`, captured *honestly* for the first time.

Non-Event signals (lifecycle transitions, connection state changes) ride the same pipe with `"type":"signal"`:

```json
{"type":"signal","name":"home_node_disconnected","reason":"transport.goodbye","timestamp":"..."}
{"type":"signal","name":"home_node_reconnected","timestamp":"..."}
{"type":"signal","name":"lifecycle","state":"degraded_federation","timestamp":"..."}
```

This surfaces the Appendix E lifecycle state transitions in real time to the AI driver, addressing Clair's Point 6 (lifecycle-state-blind error reporting) for the observation side.

## 3. Command/reply protocol — JSONL

### 3.1 Command shape

Every command from driver to instance is one JSON object on one line:

```json
{"cmd":"create-space","args":{"name":"Test Space"}}
{"cmd":"send","args":{"space":"$space","room":"$room","text":"hello"}}
```

Fields:

- `cmd` (required, string) — the command verb. The set is the same as the `xgen-client` subcommand surface: `register`, `create-space`, `create-room`, `invite`, `join`, `send`, `history`, `whoami`, `status`, `spaces`, `rooms`, `members`, `federate`, `ai delegate`, `ai revoke`, `ai status`, plus `--aicontrol`-only verbs documented below.
- `args` (required, object) — the named arguments for the command. Argument names match CLI long-flag names with dashes (e.g. `"create-room"` uses `"space"`, `"name"`).
- `id` (optional, string) — driver-supplied correlation ID echoed back in the reply. Useful when driver logs need to thread reply to command across the wire.

### 3.2 Reply shape

Every reply is one JSON object on one line:

```json
{"status":"ok","cmd":"create-space","id":"<echoed>","data":{"space_id":"xgen://hash/sha256:...","event_id":"xgen://hash/sha256:..."}}
{"status":"error","cmd":"send","id":"<echoed>","error":{"code":4002,"name":"predecessor_timeout","message":"...","instance_state":"ready"}}
```

Fields:

- `status` (required, string): `"ok"` or `"error"`. No other values.
- `cmd` (required, string): echoes the command verb.
- `id` (present iff the command included one): echoes the driver's correlation ID.
- `data` (present iff `status == "ok"`, required then): command-specific result fields. The schema per command is part of the canonical `--aicontrol` document (forthcoming, in Ch4 or a dedicated Appendix).
- `error` (present iff `status == "error"`, required then): structured error. Schema below.

### 3.3 Error shape — lifecycle-aware

Every error reply includes the instance's current lifecycle state alongside the error itself:

```json
{"error":{
  "code":"INSTANCE_NOT_READY",
  "category":"lifecycle",
  "message":"register first or complete SETUP in the UI",
  "instance_state":"setup",
  "hint":"run: register --name <your name>"
}}
```

Fields:

- `code` (required, string): the error code. Two categories:
  - **Protocol codes** — numeric codes from the existing XGen error domain (e.g. `4002` for predecessor timeout). Carried as the numeric value cast to string for uniformity.
  - **Control-surface codes** — string codes specific to `--aicontrol`: `INSTANCE_NOT_READY`, `UNKNOWN_COMMAND`, `BAD_ARGUMENT`, `BINDING_NOT_FOUND`, `CONCURRENT_COMMAND_NOT_ALLOWED`, `CONNECTION_LOST`, `TIMEOUT`. Documented exhaustively in the canonical document.
- `category` (required, string): one of `protocol`, `lifecycle`, `argument`, `connection`, `timeout`. Lets the driver branch on broad category without parsing the code.
- `message` (required, string): human-readable description. Not for programmatic parsing.
- `instance_state` (required, string): the instance's current lifecycle state at the time of the error — one of the Appendix E states. Lets the driver reason about whether to retry, wait, or escalate.
- `hint` (optional, string): a suggested next command if applicable. Free-form but stable enough that drivers can match on it.

## 4. Variable bindings — named, mandatory

Clair's Point 4 proposed `@last_space` / `@last_room` convenience plus `let X = @last_room` explicit binding. Chat Claude pushed back: implicit `@last_*` is unsafe for non-interactive scripts because creating two rooms back-to-back makes `@last_room` resolve to the second, which is rarely what the script wanted.

Decision: **named bindings are mandatory; `@last_*` is not implemented in v1**.

```json
{"cmd":"create-space","args":{"name":"Test"},"bind":"space"}
{"cmd":"create-room","args":{"space":"$space","name":"general"},"bind":"room"}
{"cmd":"send","args":{"space":"$space","room":"$room","text":"hello"}}
```

- `bind` (optional, string): names the result of this command. The binding is created when the command succeeds; on failure no binding is written.
- `$<name>` in any argument value: substitutes the named binding before dispatch. Unknown binding → error `BINDING_NOT_FOUND`.
- Bindings are scoped to the pipe connection. New connection = empty binding namespace.
- The `bind` target is whatever the command's primary return value is — `space_id` for `create-space`, `room_id` for `create-room`, `event_id` for `send`, etc. The canonical doc lists the bind value per command.
- For composite results (e.g. `create-space` returns both `space_id` and `event_id`), `bind:"foo"` binds `foo` to the primary return; access other fields via `$foo.event_id` syntax. **[Joe-flag if this syntax conflicts with anything else]** — substring substitution inside the JSON value, simple dot notation only, no expressions.

Why not `@last_*`:

- Determinism for non-interactive scripts. A script that creates 5 Spaces and 5 Rooms cannot reliably address them via `@last_*`; named bindings make every reference explicit.
- Forward-compat. Adding `@last_*` later as a convenience layer over named bindings is easy; removing implicit `@last_*` after drivers depend on it is hard.
- Smaller test surface. Named-only is a simpler protocol to verify than named-plus-implicit.

## 5. Subscription state and binding inspection — `state` command

Clair's Point 6 proposed a `state` command as a structured `whoami`+`status`+`spaces` rolled together. Confirmed and extended:

```json
{"cmd":"state"}
< {"status":"ok","cmd":"state","data":{
    "lifecycle":"ready",
    "identity_id":"xgen://pubkey/ed25519:...",
    "home_node":"ws://127.0.0.1:8080/xgen",
    "home_node_connected":true,
    "connected_since":"2026-05-17T...",
    "spaces":[{"space_id":"...","role":"owner","member_count":3,"room_count":2}],
    "bindings":{"space":"xgen://hash/sha256:...","room":"xgen://hash/sha256:..."},
    "event_subscriptions":1
  }}
```

Key additions over Clair's proposal:

- `bindings` map exposes the current session's binding namespace. Useful for drivers debugging substitution issues.
- `event_subscriptions` count surfaces how many event pipes are currently attached. An AI driver can verify its own subscription is registered.
- `home_node_connected` distinguishes between "instance is up" and "instance is connected to its home Node." A `ready` lifecycle with `home_node_connected: false` means the instance is healthy but currently network-degraded.

## 6. Persistent WebSocket to home Node

Clair's highest-priority point. Confirmed.

When the `xgen-client --service` resident starts (whether for `--batch`, `--aicontrol`, or the regular Tauri shell), it opens **one** authenticated WebSocket to its home Node at startup. The connection is:

- **Reused across all commands** dispatched via any control surface. Per-command WS churn ends.
- **Reconnected automatically** with the spec 3.3.6 backoff if the home Node drops. During disconnection, commands requiring the network fail with `error.category == "connection"` and `error.code == "CONNECTION_LOST"`; the driver can poll `state` to detect reconnection.
- **Multiplexed across Spaces.** A single WS carries traffic for every Space this Identity is a member of. Per-Space `prev_events` is tracked locally in `SessionState.spaces[space_id]` as the most-recent-event-observed-per-Space. **No more per-command `get_dag_tips` over sync_request — the F-003/F-004 bug class is architecturally eliminated.**

The persistent connection is created lazily on first network operation if it isn't already up, with a short timeout (3 seconds default). If creation fails, that first command fails with `CONNECTION_LOST` and subsequent commands retry creation transparently. This means a driver that fires `whoami` (no network) followed by `send` (network) doesn't block on connection time during the offline operation.

## 7. Shared command implementation layer — `ops::*`

Confirmed Clair's Point 5 verbatim, with one constraint Chat Claude added: **migration is atomic per-command-pair, not staged.**

The refactor produces a single source of truth:

```rust
// xgen-client/src/ops.rs (in xgen-client-lib)

pub struct OpContext<'a> {
    pub session: &'a mut SessionState,   // persistent connection, bindings, per-Space cache
    pub data_dir: &'a Path,
    pub log: &'a dyn LogSink,
}

pub async fn create_space(ctx: &mut OpContext<'_>, args: CreateSpaceArgs) -> Result<CreateSpaceResult> { ... }
pub async fn send(ctx: &mut OpContext<'_>, args: SendArgs) -> Result<SendResult> { ... }
// ... one function per command
```

- **`--batch` path:** thin wrapper that constructs a one-shot `SessionState` (fresh connection per call, no bindings), calls `ops::*`, formats the result as `OK\n` / `ERROR: ...\n`, drops the session.
- **`--aicontrol` path:** holds the `SessionState` across the pipe connection lifetime, calls `ops::*`, formats the result as JSONL.
- **Tauri UI path:** same `SessionState`, called via Tauri commands.

**Migration sequencing.** For each command (`register`, `create-space`, `create-room`, ...), the per-command refactor is one atomic commit:

1. Add `ops::cmd_name` function.
2. Replace `cmd_cmd_name` (in `main.rs`) with a thin shim calling `ops::cmd_name`.
3. Replace `exec_cmd_name` (in `batch.rs`) with a thin shim calling `ops::cmd_name`.
4. Delete the duplicated logic from both.

All four steps in the same commit. Partial migration (only `cmd_*` migrated, `exec_*` still has its own implementation) creates a *third* drift surface and is explicitly forbidden. This is non-negotiable.

## 8. Timeout and cancellation

Clair's review did not address this. Chat Claude flagged it as a missing concern. Decision:

- **Per-command timeout** is part of the command's `args` block as an optional `timeout_ms` field. Default per command (in the canonical doc) is conservative (30 seconds for network commands, 5 seconds for state-read commands). Drivers can override.
- **On timeout**, the instance returns `error.code == "TIMEOUT"`, `error.category == "timeout"`. The command may or may not have actually executed remotely — the timeout is a local guard, not a remote cancel. For idempotent commands (`whoami`, `status`), retry is safe. For non-idempotent commands (`send`, `create-space`), the driver must reconcile via subsequent state queries.
- **No explicit cancel command.** A driver wanting to cancel an in-flight command closes the pipe connection. The instance treats connection close as cancellation of any in-flight command and cleans up locally; remote side-effects may already have happened.

This is the simplest model that handles the realistic failure modes without inventing a request-tracking layer the driver doesn't need.

## 9. Pipe-level authentication (deferred)

**[Joe-flag for awareness, not for decision today]**

Today: anyone with permission to open `\\.\pipe\xgen-client[-<label>]` controls the live Identity. That's fine on a single-user dev box but becomes a concern when:

- An MCP server runs as a different OS user than the human user.
- Multiple AI drivers share access to one Identity (intentionally) and want audit trails per driver.
- A compromised AI driver should not be able to read the entire event history of every Space the Identity is in.

None of these scenarios exist today. The pipe surface remains unauthenticated in `--aicontrol` v1; the deployment assumption is "the driver runs as the same OS user as the live instance."

When this needs to change (likely with the MCP server milestone), the natural primitive is per-connection authentication via a token established when the AI driver and the human user paired (similar to the M2/M3 trust assertion model but scoped to the local control plane, not the federation plane). Recorded here so future work has a starting point; not designed today.

## 10. Replay safety (deferred)

If a driver crashes mid-batch and reconnects, what's the contract? Today's answer is "none — fresh connection, fresh binding namespace, do it over." Acceptable for v1 because the typical AI driver is itself robust to per-command failure (it retries based on the resulting state).

If future work needs strong replay safety, the natural extension is **idempotency keys**: each command carries a driver-supplied `idempotency_key`; the instance remembers recently-seen keys and returns the original reply for any duplicate. Not in v1 because the design constraint (which idempotency keys mean "same command" — same `cmd` + `args`? same `cmd` + `args` + binding state?) is non-trivial and adding it later is straightforward (new optional field, default off).

## 11. Sequencing

The revised plan after this addendum:

1. **Run `MULTIPARTY_S1` Tauri rerun and `S2`–`S5` baseline pass with the present `--batch` as-is.** Captures the "A" baseline metrics column per Clair's protocol. No code change.
2. **Refactor `cmd_*` and `exec_*` into shared `xgen-client-lib::ops::*`.** Per-command atomic commits. Touches both legacy `--batch` and (later) `--aicontrol` foundations. Test coverage: existing `cargo test` plus a re-run of the present `--batch` smoke confirms the legacy surface is unaffected.
3. **Land `--aicontrol` v1.** Three pipes (legacy, command, events), persistent WS, JSONL command/reply protocol, named bindings (mandatory), lifecycle-aware errors, `state` command, per-command timeout. The canonical document (Ch4 or new Appendix — to be decided in the design phase) is written in the same pass.
4. **Re-run `S1`–`S5` against `--aicontrol` (the "B" pass).** Capture matching metrics in the second column of every findings file's metrics table.
5. **Closing journal entry** summarising the A/B comparison across all scenarios.

The key insight from this sequencing: step 2 (the `ops::*` refactor) is independent of any `--aicontrol` design detail and benefits both surfaces. It ships first to ensure the baseline pass exercises unified handlers.

## 12. Open items for the design phase

When `--aicontrol` v1 work is scheduled, these items need explicit decisions in the canonical document:

- The full `cmd` verb set with per-command `args` schema and `data` reply schema.
- The exhaustive list of control-surface error codes (§3.3 above lists the categories but the full list grows as commands are added).
- The exact subscription filter grammar for the `.events` pipe (§2 sketches it; the grammar needs a formal definition).
- The `state` command's full output schema (§5 sketches it; the schema needs to be locked).
- The per-command default timeout values (§8 says "conservative defaults" but each command needs an actual number).
- Whether `xgen-node --aicontrol` is in scope for the same milestone or deferred.

None of these block this addendum or D-066 from being landed; they are the design-phase deliverables.

---

*End of Chat Claude addendum.*
