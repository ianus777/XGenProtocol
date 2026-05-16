# MULTIPARTY_S1 — Tauri rerun (runbook for a fresh session)
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

MULTIPARTY_S1 was executed in J-067 (commit `2cb2893`) but via the **CLI `--batch` path** (`xgen-client.exe --batch <file>`), not the **Tauri `--batch` path** specified by the S1 file (`xgen-client-app.exe --instance <label>` long-lived + `xgen-client-app.exe --instance <label> --batch <file>` driver). Justified by setup speed, but it left the deployment shape unverified — see `tasks/BATCH_FLAG_review.md` §5 on the duplicate `cmd_*` / `exec_*` implementations.

This file is the runbook for re-running S1 through the **Tauri path** in a fresh session.

---

## Reading order for the fresh session

1. **`CLAUDE.md`** — current state, behaviour rules.
2. **`docs/tests/MULTIPARTY_S0_intro.md`** — operation conventions.
3. **`docs/tests/MULTIPARTY_S1_multiclient_one_node.md`** — the test itself. (Status is COMPLETED from J-067; that's fine — this rerun is run 2 in the findings file's run history table.)
4. **`docs/tests/MULTIPARTY_S1_findings.md`** — what was already verified in run 1 (CLI path) and the four bugs that were fixed. The Tauri rerun appends a new row to the "Run history" table and a new entry in the verdict, not a rewrite of the whole file.
5. **`tasks/BATCH_FLAG_review.md`** — context on the `--batch` surface, including known limitations that may bite again.
6. **This file** — operational sequence.

---

## What's already done — DO NOT redo

| Item | Where | Status |
|---|---|---|
| Local fan-out implementation | `xgen-node-lib::fanout`, `xgen-node/src/main.rs` | DONE (commit `7e06896`) |
| F-002 fix (first-message dispatch) | `xgen-node/src/main.rs` | DONE (commit `2cb2893`) |
| F-003 fix (`get_dag_tips` Space filter, batch.rs) | `xgen-client/src/batch.rs` | DONE (commit `2cb2893`) |
| F-004 fix (`get_dag_tips` Space filter, main.rs) | `xgen-client/src/main.rs` | DONE (commit `2cb2893`) |
| `xgen-client init --passphrase` flag | `xgen-client/src/main.rs` | DONE (commit `2cb2893`) |
| `.xgb` smoke scripts | `docs/tests/scripts/multiparty_s1_smoke_*.xgb` | DONE (commit `2cb2893`) — reuse as-is, only the literal IDs will be regenerated per run |
| `.xgb` stress scripts | `docs/tests/scripts/multiparty_s1_stress_*.xgb` | DONE — reuse as-is |
| Findings file | `docs/tests/MULTIPARTY_S1_findings.md` | DONE for run 1; **append** run 2 |

The Node-side protocol code is the same on both paths. The fan-out and sync_request paths are exercised regardless of which client path drives them. So the Tauri rerun verifies the **client-side `exec_*` handlers + named-pipe IPC + Tauri lifecycle**, not the Node-side fan-out.

---

## What changes vs. the CLI run

The differences, top to bottom:

| Aspect | CLI run 1 (DONE) | Tauri rerun (TODO) |
|---|---|---|
| Client binary | `xgen-client.exe` | `xgen-client-app.exe` |
| Node binary | `xgen-node.exe` (no GUI, just listening) | `xgen-node-app.exe --service` (no GUI, systray suppressed) — or with systray, same thing |
| Instance launch | not needed — each batch invocation is a fresh process | **3 long-lived `xgen-client-app.exe --instance m1<X>` processes must be started first** and remain running for the duration of the test |
| Per-instance data dir | `test_runs/multiparty_s1_run1/m1<X>/` (CLI binary co-located) | The Tauri app uses `<exe_dir>/instances/<label>/` by default (see `xgen-client/src-tauri/src/main.rs::resolve_data_dir`). For test isolation we need to put the binaries in a clean test_runs dir so the instance dir layout is predictable. |
| Bootstrap: keypair + config | `xgen-client init --passphrase ""` per instance, run from the instance dir (so `exe_dir()` resolves there). DONE in run 1 — the existing keypairs in `test_runs/multiparty_s1_run1/m1<X>/` can be reused. | Same — the keypair files are at the per-instance data dir. **However:** the Tauri app's first-run flow enters SETUP state if neither config nor keypair exists. If we reuse the existing keypairs (recommended), SETUP is skipped and the instance proceeds straight to `INITIALISING → CONNECTING → READY` (or `DISCONNECTED` if no Node is running yet). |
| Driving a command | `xgen-client.exe --batch X.xgb` (one process, runs to completion, exits) | `xgen-client-app.exe --instance m1<X> --batch X.xgb` — the binary's `main()` detects `--batch` early and calls `batch::run_batch_client(file, pipe_name, label)`, which opens `\\.\pipe\xgen-client-<label>` and dispatches lines to the *already-running* m1<X> instance. **The long-lived instance must be up first.** |
| First-run UI gate | none (CLI) | SETUP if first launch. Reusing the existing keypairs in `test_runs/multiparty_s1_run1/m1<X>/` skips SETUP. Confirm before starting. |
| `prev_events` derivation | `cmd_send → cmd::get_dag_tips` (main.rs, F-004 fix) | `exec_send → batch::get_dag_tips` (batch.rs, F-003 fix). **Both fixes are in `2cb2893` — both paths now filter by Space.** |
| Number of windows on the desktop | 0 | 1 per long-lived client = 3 client windows; Node systray icon (no window unless detached) |

The single most important thing the Tauri path verifies that the CLI path didn't: the `batch.rs::exec_*` handlers and the named-pipe `dispatch_line` plumbing. Same protocol effect, different code surface.

---

## Pre-flight — confirm before starting the run

Before flipping any status, the next session should confirm:

1. **The instance data dirs and their keypairs from run 1 are intact** (`test_runs/multiparty_s1_run1/m1node/`, `m1a/`, `m1b/`, `m1c/`). If not, regenerate via the same procedure as J-067 used (init with `--passphrase ""`).
2. **`xgen-client-app.exe` and `xgen-node-app.exe` are built and locatable** — `C:/cargo-targets/XGenProtocol/release/xgen-*-app.exe`. Versions should embed commit `2cb2893` or later (`xgen-client-app.exe version` after copying it to an instance dir). If the version is older, rebuild.
3. **Port 8080 is free** — `netstat -ano | findstr ":8080"`. Kill any lingering xgen-node from earlier sessions.
4. **No leftover named pipes from prior runs.** Windows auto-cleans pipes on process exit but if a Tauri instance was crashed mid-run, the pipe name may briefly be claimed. The first `start_pipe_server` call after that handles it — but if the pipe-server-bind fails, kill any orphan `xgen-client-app.exe` / `xgen-node-app.exe` first.
5. **The two `get_dag_tips` copies are still in sync.** Quick diff: `git diff xgen-client/src/main.rs xgen-client/src/batch.rs` should show no semantic difference in the get_dag_tips bodies. If they have drifted, sync them before running (or — preferably — see `tasks/BATCH_FLAG_review.md` §5 and consider unifying them as a precondition).

---

## Operational sequence (the actual run)

### M0 (prep)

- **Append a new row to `docs/tests/MULTIPARTY_S1_findings.md`'s "Run history" table** for run 2 (Tauri).
- **Update the "Pre-execution notes" section** with a Tauri-rerun note: same as run 1, with the deviation now corrected (Tauri path used as S1 specifies).
- **Record binary versions** for `xgen-node-app.exe` and `xgen-client-app.exe` (these don't have a `version` subcommand the way the CLI does — see point on improvement below).
- **Confirm the pre-flight checklist above passed.**

### M1 — Tauri P1 Smoke

1. **Start the Node** in the background:
   ```
   cd test_runs/multiparty_s1_run1/m1node
   ./xgen-node-app.exe --service > m1node_console.log 2>&1 &
   ```
   The `--service` flag suppresses the systray (verify against `NODE_CORE_UI_ph2.md` if behaviour differs). Wait for the state file `xgen-node_state.json` to appear with `lifecycle: "ready"`, or grep the log for "Listening on ws://127.0.0.1:8080/xgen".
2. **Start the three client Tauri instances** in the background. Each will open a window — that's expected:
   ```
   cd test_runs/multiparty_s1_run1/m1a
   ./xgen-client-app.exe --instance m1a > m1a_console.log 2>&1 &
   # ditto for m1b, m1c with their own dirs
   ```
   Each will progress through `INITIALISING → CONNECTING → READY` (since the keypairs exist and Node is up). Wait until the state file in each instance dir reads `READY` (poll the file or sleep ~3 s).
3. **Verify the named pipes are bound:**
   ```
   ls /proc/sys/fs/binfmt_misc/.../ 2>/dev/null || PowerShell:
   Get-ChildItem \\.\pipe\ | Where-Object Name -like 'xgen-client-m1*'
   ```
   Should show `xgen-client-m1a`, `xgen-client-m1b`, `xgen-client-m1c`.
4. **Run alice pass 1 via Tauri-batch:**
   ```
   ./xgen-client-app.exe --instance m1a --batch docs/tests/scripts/multiparty_s1_smoke_clientA_pass1.xgb
   ```
   Wait for exit 0. The output goes to stdout; the long-lived m1a instance's log records the actual events.
5. **Capture Space ID + Room ID from m1a's log** (the run-2 Tauri log will be at `test_runs/multiparty_s1_run1/m1a/logs/xgen-client-app_*.log`, not the CLI log). Same scraping approach as run 1.
6. **Regenerate pass-1b / pass-2 / bob / carol scripts** with the literal IDs. Same procedure as J-067.
7. **Dispatch each batch script** through its instance's pipe:
   ```
   ./xgen-client-app.exe --instance m1a --batch docs/tests/scripts/multiparty_s1_smoke_clientA_pass1b.xgb
   ./xgen-client-app.exe --instance m1a --batch docs/tests/scripts/multiparty_s1_smoke_clientA_pass2.xgb
   ./xgen-client-app.exe --instance m1b --batch docs/tests/scripts/multiparty_s1_smoke_clientB.xgb
   ./xgen-client-app.exe --instance m1b --batch docs/tests/scripts/multiparty_s1_smoke_clientB_join_room.xgb
   ./xgen-client-app.exe --instance m1b --batch docs/tests/scripts/multiparty_s1_smoke_clientB_send.xgb
   # ditto m1c
   ```
8. **Run `xgen-client-app.exe history`** from each instance? — **PROBLEM:** the batch dispatcher in `batch.rs` doesn't support `history` (see `BATCH_FLAG_review.md` table). The CLI run worked around this by invoking `xgen-client.exe history` directly outside the batch flow. For the Tauri rerun, options:
   - **Option A:** invoke `xgen-client.exe history` (CLI, not Tauri) from the same data dir as a temporary inspection — this works because the data dir has the same keypair + state file. Less pure but practical.
   - **Option B:** add `history` to the batch dispatcher's `BatchCommand` enum and re-implement `exec_history` in batch.rs (small change). Worth doing as part of the Tauri rerun work.
   - **Recommended:** Option B if the work is small (it is, ~50 lines mirroring `cmd_history` from main.rs). Otherwise Option A with a note in findings.
9. **Build the pairing table** the same way as run 1: extract event_ids from Node and per-instance logs, classify Out/In/–, check cell by cell.
10. **Content-leak check** with `grep` over all logs.
11. **Record results** in the findings file under run 2's section.

### M2 — Tauri P2 Stress

The same shape as M1 but using the stress scripts. **Key difference from CLI run:** each of the 300 sends will dispatch through the long-lived Tauri instance over its named pipe. **Each send still opens its own WebSocket to the Node** (under the current `--batch` implementation, see `BATCH_FLAG_review.md` §1). Expect:

- Total time similar to CLI run (~60 seconds for 300 sends, dominated by WS handshakes — the pipe layer adds <1 ms per command).
- 6/300 silent message loss might recur (or might be slightly different — the pipe driver's read-OK-after-each-line shape changes the timing of `goodbye`). Investigate if the count changes meaningfully.
- The pipe driver should report exit 0 if all 100 lines dispatched without `ERROR:` (the long-lived instance's exec failures will be `ERROR:` on the pipe even though the WS write may have succeeded).

### Shutdown

1. Three batch invocations of `quit` (via the Tauri shell's `quit` Tauri-command, OR just `taskkill /F` if quit-via-batch isn't wired).
2. `Get-Process xgen-*` to confirm no lingering processes.

---

## Things to flag to Joe before / during the run

These are decisions worth surfacing rather than silently making:

1. **Should `history` be added to the batch dispatcher's command set?** Currently it's not in `BatchCommand` enum (see `xgen-client/src/batch.rs:80-99`). The S1 verification needs it. Either add it (~30 lines), or use the CLI fallback (Option A above) with a note. **Recommendation: add it.**
2. **Verify version embedding in `xgen-client-app.exe`.** The Tauri shell does NOT have a `version` subcommand mirrored to the CLI's. Confirm what `xgen-client-app.exe --help` shows (likely opens a Tauri window since `--help` isn't intercepted before Tauri startup). If there's no version path, document the build via cargo metadata instead. **Possibly worth adding a `--version` short-circuit in the Tauri main, similar to `--batch`.**
3. **The 6/300 silent loss in P2** — same investigation worth doing here: capture the precise count, see if it correlates with pipe-server slowness (any handler delays?), see if increasing the mpsc channel capacity helps. Goal: characterise, not eliminate, on this run. Elimination is its own follow-up.
4. **The `--service` flag on the Node Tauri** — verify it actually suppresses GUI / systray and just runs headless. The S1 file says "headless mode for the Node". If `--service` opens the systray with no admin window, that's fine. If it opens an admin window, kill the test and document.

---

## Reference — what `BATCH_FLAG_review.md` says about doing this right

The current Tauri `--batch` has known limitations documented in `tasks/BATCH_FLAG_review.md`. The most relevant for this rerun:

- **Per-command WS churn** (§1): every `exec_send` opens its own WS. Real-time fan-out happens but can't be observed (connection is gone before the Node pushes back). The pairing table for run 2 will, like run 1, be a reconstruction-from-history rather than an observation of real-time delivery.
- **No backreferences** (§4): the two-pass script regeneration is still mandatory.
- **No structured replies** (§2): we still scrape log lines for IDs.
- **No event observation channel** (§3): real-time fan-out is unverifiable from the batch driver's side.

The Tauri rerun doesn't fix any of those — it verifies that the deployment shape works as well as the CLI shape did. If the rerun PASSES with the same pairing table and 98% P2 delivery, that's the green light. If it surfaces something new (a Tauri-shell-specific bug, a pipe ordering issue, a lifecycle-state race), that's its own F-NNN to file and fix.

---

## What success looks like

Run 2 of the findings file shows:

```
| Run | Date | Build / commit | P1 | P2 | Notes |
| 2 | 2026-05-1? | <commit hash> | PASS | PASS (98% or thereabouts) | Tauri path used per S1 spec; verifies exec_* + pipe IPC deployment shape. (Same Node-side fan-out as run 1.) |
```

And the overall verdict at the bottom of the findings file is unchanged ("PASS with caveat") or stronger if the 6/300 loss is gone.

If run 2 PASSES, S1 is fully closed and S2 can begin.

---

*End of `MULTIPARTY_S1_tauri_rerun.md`*
