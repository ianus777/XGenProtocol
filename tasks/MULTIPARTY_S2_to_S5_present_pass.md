# MULTIPARTY S2–S5 — Present-version pass (runbook for fresh sessions)
> **Status**: PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

After J-067 (MULTIPARTY_S1 via CLI shortcut), Joe and I agreed (2026-05-16):

1. **Run all five Multiparty scenarios (S1–S5) through Tauri `--batch` with the present implementation.** This is the "A" leg of the planned A/B comparison.
2. **Capture baseline metrics** per the protocol in `tasks/BATCH_FLAG_review.md` §"Baseline metrics protocol".
3. **Maintain a friction log** in the same file, appending observations as they surface during each scenario.
4. **At the end of the present pass:** review the friction log, refine the six improvement points in the review based on observed (not speculated) pain.
5. **Ship the improvements**, then re-run the full suite as the "B" leg.

This file is the meta-runbook covering S2–S5 (S1's Tauri rerun has its own runbook, `tasks/MULTIPARTY_S1_tauri_rerun.md`). It captures cross-scenario conventions that aren't in the individual S2/S3/S4/S5 scenario files, plus the explicit decisions Joe and I reached that the original scenario files don't reflect.

---

## Reading order for each fresh session

For each scenario S<N> in order (S2 → S3 → S4 → S5):

1. **`CLAUDE.md`** — global behaviour rules, current state.
2. **`docs/tests/MULTIPARTY_S0_intro.md`** — operation conventions.
3. **`docs/tests/MULTIPARTY_S{N}_*.md`** — the scenario itself.
4. **Findings files from prior scenarios** in the suite (lower N). Their friction log entries and observed-vs-expected discrepancies may affect what to watch for.
5. **`tasks/BATCH_FLAG_review.md`** — §"Baseline metrics protocol" + §"Friction log" (append observations from this scenario here).
6. **This file** — cross-scenario decisions.

---

## Cross-scenario decisions (Joe + AI, 2026-05-16)

These apply to every scenario S2–S5 (and were also applied to S1 Tauri rerun). They are decisions, not options — the future-session should not re-litigate them mid-run without flagging to Joe.

### Decision 1 — Tauri `--batch` only, not the CLI shortcut

Every scenario runs against the long-lived Tauri executables driven via named-pipe `--batch`. No `xgen-client.exe --batch` (the CLI shortcut). Reason: the deployment shape and the AI control surface are both the Tauri `--batch` path; testing the CLI path leaves `batch.rs::exec_*` and the named-pipe IPC unverified.

The CLI run of S1 (J-067) is grandfathered as the only exception. From here forward, every Multiparty scenario uses Tauri.

### Decision 2 — present `--batch` is good enough for verification, weak for observation

The current `--batch` implementation has known limitations documented in `BATCH_FLAG_review.md` §1–6. The most important for these scenarios:

- **Per-`send` WS churn** (§1): each command opens its own WebSocket. Real-time fan-out arrives at the client *after* the connection has closed. The client driver cannot observe real-time delivery — only reconstruct eventually-consistent state via `sync_request`.
- **No backreferences** (§4): every `--space` / `--room` argument is a literal `xgen://hash/sha256:...` ID; two-pass script generation is mandatory.
- **No structured replies** (§2): created IDs are scraped from log files.

The decision: run scenarios anyway. Verification claims (PASS/FAIL on protocol invariants) are unaffected. Observation claims (real-time delivery timing, causal-order witness) are weaker — that's an expected baseline metric, not a test failure. The improved pass will close those gaps; the present pass establishes the baseline.

### Decision 3 — capture baseline metrics per the protocol

Each scenario captures the metric set in `BATCH_FLAG_review.md` §"Baseline metrics protocol":

- Outcome, throughput, latency (median/p95/max), loss, errors, DAG integrity, observability %, setup cost.
- 1 verification run + n=3 measurement runs of the stress / concurrent phase.
- Metrics land in a "Metrics" section in each scenario's `MULTIPARTY_S{N}_findings.md`, in a two-column table (Present | Improved).
- Unmeasurable metrics in the present version use `—` with a one-line reason.

### Decision 4 — append observations to the friction log as they happen, not after the fact

The friction log lives in `BATCH_FLAG_review.md` §"Friction log". Format: `[S2] one-line observation with enough context to act on later`. Append during execution, not retrospectively — friction is easy to forget after a PASS.

### Decision 5 — do NOT fix `--batch` issues during the present pass

If a scenario surfaces a `--batch` shortcoming, **log it and work around it** (two-pass scripts, log scraping, whatever). Do NOT improve `--batch` during the present pass — the whole point is to measure the present version uniformly. Improvements get batched after S5 completes.

**Exception:** if a scenario reveals a **protocol-level bug** (not a `--batch` shortcoming) like F-001 / F-002 / F-003 / F-004 in J-067, that's a blocker and gets fixed. The distinction:

- "`--batch` is awkward to use here" → friction log, do not fix.
- "the Node accepts a malformed event" → fix immediately, write findings, continue.

### Decision 6 — Findings files document run history; CLI run-1 of S1 is grandfathered

Each `MULTIPARTY_S{N}_findings.md` has a "Run history" table. The Tauri-batch present pass adds a row. The improved pass adds another row. S1's findings file already has run 1 (CLI, J-067); run 2 (Tauri present) and run 3 (Tauri improved) get appended.

S2–S5 will each have run 1 = Tauri present and run 2 = Tauri improved. Cleaner numbering because they never had a CLI run.

---

## Per-scenario heads-ups

Things specific to each scenario that may bite — sourced from S1 experience + close reading of the S2–S5 files.

### S2 (`MULTIPARTY_S2_concurrent_send.md`) — DAG under concurrent writes, 2 Nodes

- **Federation handshake adds setup work.** Each Node needs to know about the other before federation propagates events. Bootstrap order: start both Nodes, federate them, then start clients on each side.
- **The "concurrent send within 50 ms" requirement** (per the S0 intro) is much tighter than S1's 1-second window. With per-`send` WS churn, achieving 50 ms is harder. May need to dispatch via something faster than separate `--batch` invocations. Worth flagging if achievable timing exceeds 50 ms — record actual dispatch window in findings either way.
- **Causal-order verification under concurrent writes is exactly the case the present `--batch` can't observe in real time.** S2's pairing table assumes you can witness which event arrived at which Node first. With present `--batch`, you can only verify the post-merge state — the two Nodes converge to the same DAG. Document the limitation in the "Observability %" metric for S2 (probably 0% — pure reconstruction).

### S3 (`MULTIPARTY_S3_federation_topology.md`) — 3 Nodes, chain + mesh, transitive

- **The spec gap.** Per S0 intro §"Spec gaps and capability gates": §3.2 lacks an explicit "forward on accept to all federated peers" canonical sentence. Transitive propagation is implied by §3.9.2 (convergence guarantee) but not written as a normative MUST. S3's M0 must record this in findings. If transitive propagation works, propose the one-sentence spec addition (raise to Joe in the journal entry — do not silently amend §3.2).
- **Topology verification is critical.** S0 explicitly warns: verify A↔C is NOT federated by inspecting each Node's federation registry, not assuming it. Stale federation entries from prior runs WILL break the test if the federation registry isn't cleaned between runs.
- **Wait at least 60 s after the final send before concluding "the event did not arrive"** — at least 2× the §3.9.6 pending timeout. Premature failure declarations are test bugs, not protocol bugs.

### S4 (`MULTIPARTY_S4_n_clients_n_nodes.md`) — 4 Nodes × 6 Clients

- **This is the test where present `--batch` hurts most.** 24+ client batch invocations (6 clients × multiple batches each), 4 Tauri Node instances, sustained chat with real-time-expectations on the pairing table. The observability % metric will probably be 0% for most cells; document it cleanly.
- **2-second concurrent dispatch window** for the chat-phase sends. Wider than S2's 50 ms; should be achievable with current `--batch` even if tight.
- **6 client windows on the desktop.** Visual clutter. The `--service` flag exists for the Node but not the Client. Joe can choose to ignore; the windows aren't necessary for the test.
- **Two pairing matrices** (events × Nodes, events × Clients) rather than one. Findings file gets twice the table-building work.

### S5 (`MULTIPARTY_S5_client_rebind.md`) — Identity portability

- **Capability gate per S0 intro.** S5 depends on three protocol surfaces that may not be fully wired through the CLI / batch: `re_registration` flag on `identity.register`, `identity.replicate` push from home Node to replicas, `identity.home_changed` event observability. S5's M0.3 verifies all three.
- **If any of the three is missing:** mark S5 as **BLOCKED**, file a separate task `tasks/MULTIPARTY_S5_BLOCKER_re_registration.md` describing the missing surface. Do NOT invent workarounds. The blocker task becomes the prerequisite for completing S5 (in either the present-version pass or the improved-version pass — possibly only the improved pass if `--batch` improvements bundle the CLI surface for these too).
- **S5 is one client across multiple Nodes** — no concurrency, no fan-out matrix, much simpler structurally. The metrics that matter for S5 are different from S1–S4 (re-bind latency, replica retention, key continuity). Adapt the metric set rather than forcing the S1–S4 shape.

---

## Pre-flight, per scenario

Same as `MULTIPARTY_S1_tauri_rerun.md`'s pre-flight, applied per scenario:

1. **Per-instance data dirs intact** for the scenario's instance labels (`m{N}a`, `m{N}nA`, etc. per the S0 intro's instance-label table). Each instance dir has a config + keypair generated via `xgen-client init --passphrase ""` (or `xgen-node init --passphrase ""`).
2. **Tauri binaries are current** — `xgen-client-app.exe version` and `xgen-node-app.exe version` should embed the latest commit (or rebuild). Note: as of 2026-05-16 the Tauri apps don't actually have a `version` subcommand — see `BATCH_FLAG_review.md` §"Things to flag" — use `cargo metadata` or check the binary's build date as a workaround.
3. **Port 8080 (and others used by the scenario) are free.** Check `netstat -ano | findstr ":8080"`. Kill any lingering `xgen-node-app.exe` or `xgen-node.exe` from earlier sessions before starting.
4. **No leftover named pipes** — Windows auto-cleans on process exit, but a crashed Tauri instance may briefly hold one. Symptom: `start_pipe_server` log line reports a bind failure. Fix: kill any orphan Tauri processes (`Get-Process xgen-*-app`).
5. **Workspace archived if pre-existing.** Each scenario's M0.3 step requires archiving prior data to `test_runs/multiparty_s{N}_<timestamp>_pre/`. Per `.gitignore`, `test_runs/` is not committed — the archive is local-only.

---

## What success of the present pass looks like

Each `MULTIPARTY_S{N}_findings.md` has:

- Status: **COMPLETED**
- "Run history" row for the Tauri present-pass run, marked PASS or PASS-with-caveat.
- "Metrics" section with the "Present version (baseline)" column fully filled, "Improved version" column with `—` placeholders.
- F-NNN bug entries for any protocol-level bugs found during this scenario (separate from `--batch` shortcomings, which go to the friction log).

And `tasks/BATCH_FLAG_review.md` "Friction log" section has accumulated observations from S2 / S3 / S4 / S5 (in addition to whatever S1 Tauri rerun contributed). The list refines the improvement priorities for the post-present-pass `--batch` improvement work.

When all five findings files are COMPLETED with present-pass metrics, write a single journal entry summarising the present pass and proposing the concrete `--batch` improvement work (informed by the friction log) for the next phase.

---

*End of `MULTIPARTY_S2_to_S5_present_pass.md`*
