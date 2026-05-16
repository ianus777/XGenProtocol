# Multiparty Test S1 — Multiple Clients on One Node
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Operation

This is file **1 of 5** in the **Multiparty** test operation — a sequenced suite of multi-party scenario tests that exercise the protocol beyond the single-node, single-client paths covered by Phase 1 / Phase 2 smoke and stress tests.

**Full sequence (locked execution order):**

1. **`MULTIPARTY_S1_multiclient_one_node.md`** — this file — multiple clients per Node
2. `MULTIPARTY_S2_concurrent_send.md` — DAG under concurrent writes
3. `MULTIPARTY_S3_federation_topology.md` — 3+ Node federation, transitive
4. `MULTIPARTY_S4_n_clients_n_nodes.md` — N clients across N Nodes
5. `MULTIPARTY_S5_client_rebind.md` — one client across multiple Nodes

Each file in the suite must be COMPLETED before the next begins.

---

## Purpose

Verify that a single Node correctly fans events out to **multiple concurrently-connected clients** sharing the same Space and Room. The Phase 1 smoke test established that two Nodes federate correctly with one client each. This test establishes that one Node correctly delivers events to N clients on its own — the local fan-out path that every real deployment relies on.

**What this test proves:**

- A single `xgen-node` process accepts and maintains multiple concurrent client WebSocket connections.
- Events authored by any one client are delivered to all other clients connected to the same Node.
- The Node's outbound event dispatch to its clients is correct and complete (no drops, no duplicates).
- Three independent Identities can coexist on one Node without state corruption.
- The DAG order is consistent across all clients' views.

**What this test does NOT prove:**

- Federation behaviour (covered by S3).
- Concurrent / simultaneous send ordering (covered by S2).
- Behaviour under sustained high load (this test uses small message counts; load-style testing is part of P2 Stress but with modest volume — true load testing is out of scope for the Multiparty suite).
- Multi-Node topology (covered by S3, S4).

---

## Prerequisites

This test depends on the following being COMPLETED:

- Phase 1 smoke test (`SMOKETEST_ph1.md`) — establishes the baseline single-client/single-node path.
- `BATCH_FLAG_ph2.md` — provides the `--batch` mechanism that drives the test.
- Phase 2 integration test (J-058, all 60 steps PASS) — establishes that the current binaries handle real multi-event sequences correctly.

**Required binaries (must build cleanly from current `main`):**

- `xgen-node-app.exe`
- `xgen-client-app.exe`

**Required spec sections (read before execution):**

- Ch3 §3.7 — Space and Room protocol (event flow, membership)
- Ch3 §3.7.11 — Phase 1 smoke test (for comparison)
- Ch6 §6.9 — Console / batch operation

---

## Scope

### In scope

- 1 Node, 3 Clients, 1 Space, 1 Room.
- Each client registers a distinct Identity on the Node.
- Each client joins the Space and Room.
- Each client sends a message; all clients observe all messages.
- P1 — exhaustive event pairing across all clients (smoke).
- P2 — modest sustained concurrent sending from all 3 clients (stress).

### Out of scope

- Federation (no second Node).
- Identity rebinding to a different Node (S5).
- Concurrent send race conditions (S2).
- Permission and role testing (separate work).
- Encryption tier testing (Phase 2 E2E tests, separate).
- True high-volume load testing (this test caps at modest counts; the Multiparty suite is correctness-first, not throughput-first).

---

## Architecture Constraints — Non-Negotiable

These rules apply before any other implementation decision. An implementation that violates any of them is non-compliant.

**Use only existing infrastructure.** Do not add new CLI commands, new event types, or new protocol surfaces for this test. The test must run against the binaries as they exist when this file becomes ACTIVE. If a required capability is missing, stop and report — do not improvise.

**No shell invocation.** All client orchestration runs through the existing `--batch` mechanism via named pipes (D-043). Scripts are `.xgb` files passed by path. No shell concatenation, no `&&`, no PowerShell pipelines that depend on stdout parsing.

**Three distinct instance labels.** The three client instances MUST use `--instance` labels that differ from each other and differ from any other instance in active use. Suggested labels: `m1a`, `m1b`, `m1c`. This guarantees three distinct named pipes (`\\.\pipe\xgen-client-m1a`, `\\.\pipe\xgen-client-m1b`, `\\.\pipe\xgen-client-m1c`) and three distinct data directories.

**Three distinct keypair files.** Each client instance has its own keypair file. Do not share keypairs across instances under any circumstance.

**Stop on first failure.** If any P1 milestone fails, stop the entire test, record the failure in `MULTIPARTY_S1_findings.md`, and do not proceed to P2. P2 only runs after P1 is fully green.

**Honesty.** Per CLAUDE.md Rules 1–7: never fabricate results. If a step does not produce the expected output, report the actual output verbatim and stop. Do not invent event counts, pairing matches, or PASS verdicts.

**Findings file is the write surface during execution.** This instruction file is read-only during execution except for the Definition of Done checklist at the end. All runtime observations, bugs, anomalies, and re-run logs go to `MULTIPARTY_S1_findings.md` (created at the start of execution, see Milestone 0 below).

---

## Topology

```
              ┌─────────────────────────────┐
              │       xgen-node-app         │
              │   --instance m1node         │
              │   ws://127.0.0.1:8080/xgen  │
              │                             │
              │   Space: S_multiparty_1     │
              │   Room:  R_general          │
              └──────────┬──────────────────┘
                         │
       ┌─────────────────┼─────────────────┐
       │                 │                 │
   ┌───┴───┐         ┌───┴───┐         ┌───┴───┐
   │client │         │client │         │client │
   │  m1a  │         │  m1b  │         │  m1c  │
   │alice  │         │bob    │         │carol  │
   └───────┘         └───────┘         └───────┘
```

Three clients (`m1a` → alice, `m1b` → bob, `m1c` → carol) all connecting to one Node (`m1node`) at `ws://127.0.0.1:8080/xgen`. All three join the same Space and Room.

---

## Test Data and Identifiers

| Item | Value |
|---|---|
| Node instance label | `m1node` |
| Node endpoint | `ws://127.0.0.1:8080/xgen` |
| Client A instance label | `m1a` |
| Client A display name | `alice` |
| Client A passphrase (test) | `m1a-pass-1234` |
| Client B instance label | `m1b` |
| Client B display name | `bob` |
| Client B passphrase (test) | `m1b-pass-1234` |
| Client C instance label | `m1c` |
| Client C display name | `carol` |
| Client C passphrase (test) | `m1c-pass-1234` |
| Space name (P1) | `Multiparty S1 P1` |
| Room name (P1) | `general` |
| Space name (P2) | `Multiparty S1 P2` |
| Room name (P2) | `general` |

Passphrases are test fixtures only. Never reuse outside this test.

---

## Milestone 0 — Preparation

Before executing P1 or P2, create the findings file and verify environment.

### Tasks

**0.1 — Create findings file.**

Create `docs/tests/MULTIPARTY_S1_findings.md` from the template at the end of this document (Appendix A). Status starts as `ACTIVE`.

**0.2 — Verify binary versions.**

Run and record in the findings file:

```
xgen-node-app.exe --version
xgen-client-app.exe --version
```

Both binaries must come from the same build. Record the exact strings.

**0.3 — Clean workspace.**

For each of `m1node`, `m1a`, `m1b`, `m1c`, ensure no prior data exists at the per-instance data directory. If prior data exists, archive it under `test_runs/multiparty_s1_<timestamp>_pre/` before deletion. Record the archive path (or "no prior data") in findings.

**0.4 — `.xgb` scripts present and validated.**

Verify the following script files exist and are syntactically valid (per `BATCH_FLAG_ph2.md`):

- `docs/tests/scripts/multiparty_s1_smoke_clientA.xgb`
- `docs/tests/scripts/multiparty_s1_smoke_clientB.xgb`
- `docs/tests/scripts/multiparty_s1_smoke_clientC.xgb`
- `docs/tests/scripts/multiparty_s1_stress_clientA.xgb`
- `docs/tests/scripts/multiparty_s1_stress_clientB.xgb`
- `docs/tests/scripts/multiparty_s1_stress_clientC.xgb`

Their exact contents are specified in Appendix B (P1 scripts) and Appendix C (P2 scripts) below. If any script is missing or differs from the spec, create or correct it before proceeding.

### Definition of Done — Milestone 0

- [ ] Findings file `MULTIPARTY_S1_findings.md` created with header in correct format and status `ACTIVE`.
- [ ] Binary version strings recorded in findings.
- [ ] Workspace clean (or prior data archived) and recorded.
- [ ] All six `.xgb` scripts verified present and matching Appendix B / C.

---

## Milestone 1 — P1 Smoke

**Goal:** Three clients on one Node successfully register, join a common Space and Room, exchange one message each, and observe each other's messages. Pairing table proves correct fan-out.

### Sequence

The execution sequence is **serialised** for P1 — no client moves to its next step until the previous step is verified. This produces a deterministic pairing table.

**Step P1.1 — Start Node.**

```
xgen-node-app.exe --instance m1node
```

Wait for the Node to reach `READY` (state file shows `READY`; log shows the WebSocket listener bound to `127.0.0.1:8080`). Record the timestamp in findings.

**Step P1.2 — Start Client A.**

```
xgen-client-app.exe --instance m1a
```

Wait for Client A to reach `INITIALISED` (state file written, no Identity yet). Record timestamp.

**Step P1.3 — Run Client A smoke script.**

```
xgen-client-app.exe --instance m1a --batch docs/tests/scripts/multiparty_s1_smoke_clientA.xgb
```

This script (see Appendix B.1) performs: `connect`, `register alice`, `create-space "Multiparty S1 P1"`, `create-room general`, `send` of one message. Wait for exit code 0. Record the Space ID and Room ID from the client log.

**Step P1.4 — Start Client B and run smoke script.**

```
xgen-client-app.exe --instance m1b
xgen-client-app.exe --instance m1b --batch docs/tests/scripts/multiparty_s1_smoke_clientB.xgb
```

Script (Appendix B.2): `connect`, `register bob`, `join <Space ID from P1.3>`, `send` of one message. Wait for exit code 0.

**Step P1.5 — Start Client C and run smoke script.**

```
xgen-client-app.exe --instance m1c
xgen-client-app.exe --instance m1c --batch docs/tests/scripts/multiparty_s1_smoke_clientC.xgb
```

Script (Appendix B.3): `connect`, `register carol`, `join <Space ID from P1.3>`, `send` of one message. Wait for exit code 0.

**Step P1.6 — Settle.**

Wait 3 seconds to allow any in-flight events to land in all three client logs.

**Step P1.7 — Shut down all four processes cleanly.**

In order: Clients A, B, C, then Node. Use the standard shutdown command (per Ch6 §6.9); do not kill processes.

### Expected event pairing

After P1.5 completes and P1.6 settles, the Node log and all three client logs must contain the following events. Each event is uniquely identified by its `event_id` (an `xgen://hash/sha256:...` URI). For every event, the pairing table records whether each log contains it.

| # | EventType | Authored by | Expected in Node log | Expected in m1a log | Expected in m1b log | Expected in m1c log |
|---|---|---|---|---|---|---|
| 1 | `identity.register` | m1a | ✔ | ✔ (own) | — | — |
| 2 | `identity.register` | m1b | ✔ | — | ✔ (own) | — |
| 3 | `identity.register` | m1c | ✔ | — | — | ✔ (own) |
| 4 | `state.space_create` | m1a | ✔ | ✔ (own) | ✔ (fan-out on join) | ✔ (fan-out on join) |
| 5 | `state.room_create` | m1a | ✔ | ✔ (own) | ✔ (fan-out on join) | ✔ (fan-out on join) |
| 6 | `membership.join` (m1a, implicit at space create) | m1a | ✔ | ✔ (own) | ✔ (fan-out on join) | ✔ (fan-out on join) |
| 7 | `membership.join` (m1b) | m1b | ✔ | ✔ (fan-out) | ✔ (own) | ✔ (fan-out on join — see note) |
| 8 | `membership.join` (m1c) | m1c | ✔ | ✔ (fan-out) | ✔ (fan-out) | ✔ (own) |
| 9 | `message.text` (alice) | m1a | ✔ | ✔ (own) | ✔ (fan-out) | ✔ (fan-out) |
| 10 | `message.text` (bob) | m1b | ✔ | ✔ (fan-out) | ✔ (own) | ✔ (fan-out) |
| 11 | `message.text` (carol) | m1c | ✔ | ✔ (fan-out) | ✔ (fan-out) | ✔ (own) |

**Note on event 7:** the row "✔ (fan-out on join — see note)" depends on the spec's definition of historical event replay on join. If the spec at §3.7 currently states that a joining client receives **all prior events including membership events of earlier joiners**, then m1c receives event 7 at join time. If the spec states that only state-relevant events are replayed and `membership.join` of others is not, then this cell is `—`. **Cross-check spec §3.7 before execution and record the resolved expectation in findings before running.** Do not run P1 with this row ambiguous.

### Pairing table format (findings file)

The findings file records the **observed** table in this exact format:

```
| # | event_id (short, 12 chars) | EventType | Authored | Node log | m1a log | m1b log | m1c log |
|---|---|---|---|---|---|---|---|
| 1 | abc123def456 | identity.register | m1a | ✔ | ✔ | — | — |
| ... |
```

A cell is `✔` only if the actual event_id appears with the expected `direction` (Outbound for the author's client, Inbound for the Node and for other clients).

### Content-leak check

Per the Phase 1 convention. Run:

```
findstr /S /M /R "alice-msg-1\|bob-msg-1\|carol-msg-1" *.log
```

Across all client and Node logs. Each message text appears only on the **`MessageEnvelope`** level (i.e. logged as part of normal `message.text` event handling). It MUST NOT appear in any line tagged as cryptographic plaintext exposure, debug dump, or pre-encryption leak. The strict rule: zero occurrences of message text outside `message.text` event-handling log lines.

Record the findstr output verbatim.

### Definition of Done — Milestone 1

- [ ] Node started cleanly, reached `READY`, timestamp recorded.
- [ ] All three clients started cleanly, reached `INITIALISED`, timestamps recorded.
- [ ] All three smoke scripts ran with exit code 0.
- [ ] All four processes shut down cleanly.
- [ ] Observed pairing table built from log inspection, matches expected table cell-for-cell.
- [ ] Event 7 expectation resolved against spec §3.7 before run; resolution recorded in findings.
- [ ] Content-leak findstr returned zero unauthorised occurrences.
- [ ] P1 verdict recorded in findings: PASS or FAIL.
- [ ] If FAIL: stop the entire test, do not proceed to Milestone 2.

---

## Milestone 2 — P2 Stress

**Goal:** Three clients on one Node, each sending a modest stream of messages concurrently, with the Node correctly fanning all events out to all clients. Verify DAG integrity, no drops, no duplicates, and acceptable latency.

P2 runs only after P1 verdict is PASS.

### Sequence

**Step P2.1 — Clean workspace and start Node.**

Same as P1.1, but with fresh data directory. Use a new Space (`Multiparty S1 P2`) to keep P1 and P2 records separate.

**Step P2.2 — Start all three clients in parallel.**

Launch m1a, m1b, m1c without waiting for each to finish startup before the next. All three reach `INITIALISED` independently.

**Step P2.3 — Bootstrap state.**

Run sequentially (not concurrently — bootstrap must be serialised):

```
xgen-client-app.exe --instance m1a --batch docs/tests/scripts/multiparty_s1_stress_clientA.xgb
```

The Client A stress script (Appendix C.1) does setup only: `connect`, `register alice`, `create-space "Multiparty S1 P2"`, `create-room general`. Wait for exit 0. Capture Space ID and Room ID.

Then sequentially:

```
xgen-client-app.exe --instance m1b --batch docs/tests/scripts/multiparty_s1_stress_clientB.xgb
xgen-client-app.exe --instance m1c --batch docs/tests/scripts/multiparty_s1_stress_clientC.xgb
```

Each script (Appendix C.2, C.3) does: `connect`, `register`, `join <Space ID>`. Wait for exit 0 from each.

**Step P2.4 — Concurrent send phase.**

This is the actual stress phase. For each of the three clients, prepare and dispatch a **second** batch script that sends 100 messages back-to-back. Dispatch all three within a 1-second window.

Each per-client send script (Appendix C.4, C.5, C.6) contains 100 `send` lines with distinct message texts:

- m1a: `alice-stress-001` through `alice-stress-100`
- m1b: `bob-stress-001` through `bob-stress-100`
- m1c: `carol-stress-001` through `carol-stress-100`

**Concurrency requirement:** The three send scripts MUST be dispatched within a 1-second window. Use whatever orchestration is appropriate (a wrapper that fires three named-pipe sends within a tight loop is fine — but no shell concatenation). Record the dispatch timestamps in findings.

**Step P2.5 — Drain.**

Wait 30 seconds for all in-flight events to settle. The Node log and all three client logs should stop growing.

**Step P2.6 — Shutdown.**

Clean shutdown of all four processes in order: Clients A, B, C, then Node.

### Metrics to capture

In the findings file, record:

**Event counts:**

| Metric | Expected | Observed |
|---|---|---|
| `message.text` events authored by m1a | 100 | _ |
| `message.text` events authored by m1b | 100 | _ |
| `message.text` events authored by m1c | 100 | _ |
| **Total authored** | **300** | _ |
| `message.text` events in Node log (Inbound) | 300 | _ |
| `message.text` events in m1a log (Inbound + Outbound) | 300 | _ |
| `message.text` events in m1b log | 300 | _ |
| `message.text` events in m1c log | 300 | _ |

**Integrity:**

| Check | Expected | Observed |
|---|---|---|
| Duplicate event_ids in Node log | 0 | _ |
| Duplicate event_ids in any client log | 0 | _ |
| `event_id` mismatches between Outbound author and Inbound observer | 0 | _ |
| DAG orphans at end of test (events with `prev_events` referencing absent events) | 0 | _ |
| `ERROR`-level log lines | 0 | _ |
| `WARN`-level log lines (non-shutdown) | 0 expected; record any | _ |

**Latency (informational, not pass/fail at this stage):**

| Metric | Observed |
|---|---|
| Median Outbound→Inbound delivery time (author client → other clients) | _ |
| p95 Outbound→Inbound delivery time | _ |
| Max Outbound→Inbound delivery time | _ |

Latency is informational for S1. It becomes a pass/fail criterion in later scenarios where the Multiparty suite has accumulated baselines.

### Definition of Done — Milestone 2

- [ ] All three clients sent 100 messages each (300 total authored).
- [ ] All four logs show all 300 `message.text` events with correct direction tags.
- [ ] Zero duplicates across all logs.
- [ ] Zero orphaned DAG events at end of test.
- [ ] Zero `ERROR` log lines across all four logs.
- [ ] `WARN` lines (if any) recorded in findings with classification.
- [ ] Latency metrics recorded.
- [ ] P2 verdict recorded in findings: PASS or FAIL.

---

## Definition of Done — Test S1 as a whole

- [ ] Milestone 0 (Preparation) all items ticked.
- [ ] Milestone 1 (P1 Smoke) all items ticked, verdict PASS.
- [ ] Milestone 2 (P2 Stress) all items ticked, verdict PASS.
- [ ] Findings file `MULTIPARTY_S1_findings.md` status set to `COMPLETED` with overall verdict.
- [ ] JOURNAL.md entry written summarising the S1 run (event counts, verdict, any anomalies).
- [ ] This instruction file's header status updated from `ACTIVE` to `COMPLETED`.
- [ ] If any bugs surfaced and required fixes during execution, the FIX records are linked from the findings file.

After all items above are ticked, sequence advances to file 2/5 (`MULTIPARTY_S2_concurrent_send.md`).

---

## Appendix A — Findings file template

When Milestone 0.1 creates `docs/tests/MULTIPARTY_S1_findings.md`, use this template:

```markdown
# Multiparty Test S1 — Findings
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: YYYY-MM-DD  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Run history

| Run | Date | Build / commit | P1 | P2 | Notes |
|---|---|---|---|---|---|
| 1 | YYYY-MM-DD | _commit hash_ | _PASS/FAIL_ | _PASS/FAIL_ | _short note_ |

---

## Milestone 0 — Preparation record

- Binary versions: _paste version strings_
- Workspace state: _clean / archived to ..._
- Scripts verified: _list_
- Spec §3.7 cross-check for event 7: _resolved as ✔ or — with reasoning_

---

## Milestone 1 — P1 Smoke

### Observed pairing table

_(insert table per the instruction file's format)_

### Content-leak findstr output

```
_(paste verbatim)_
```

### Verdict: _PASS/FAIL_

_Notes:_

---

## Milestone 2 — P2 Stress

### Metrics

_(insert tables per the instruction file's format)_

### Verdict: _PASS/FAIL_

_Notes:_

---

## Findings — bugs and anomalies

_For each bug found during execution:_

### F-001 — _short title_
- **Severity:** _critical / major / minor_
- **Stage:** _P1 step / P2 step_
- **Observed:** _what happened_
- **Expected:** _what should have happened_
- **Resolution:** _link to FIXES file or commit, or "open"_

---

## Overall verdict

_PASS / FAIL / BLOCKED_
```

---

## Appendix B — P1 Smoke `.xgb` scripts

### B.1 — `docs/tests/scripts/multiparty_s1_smoke_clientA.xgb`

```
# Multiparty S1 P1 — Client A (alice) smoke script
# Phase: register, create space, create room, send one message

connect ws://127.0.0.1:8080/xgen
register --name alice --passphrase m1a-pass-1234
create-space --name "Multiparty S1 P1"
create-room --space @last_space --name general
send --space @last_space --room @last_room --text "alice-msg-1"
status
```

**Note on `@last_space` / `@last_room`:** these are placeholders for the IDs returned by the immediately preceding `create-space` / `create-room` commands. If the current batch dispatcher does NOT support such backreferences (verify against `BATCH_FLAG_ph2.md`), this script must be split into two passes:

- First pass: just `connect` + `register` + `create-space` + `create-room`, then capture IDs from log.
- Second pass: substitute literal IDs into `send` and run.

Resolve this against the binary's current behaviour and record the resolution in findings before execution. If `@last_*` placeholders are not supported, the script files below MUST be regenerated with literal IDs after step P1.3 captures them — and the regenerated scripts saved alongside (do not overwrite the templates).

### B.2 — `docs/tests/scripts/multiparty_s1_smoke_clientB.xgb`

```
# Multiparty S1 P1 — Client B (bob) smoke script
# Phase: register, join existing space, send one message
# Space ID and Room ID MUST be substituted from P1.3 output before run

connect ws://127.0.0.1:8080/xgen
register --name bob --passphrase m1b-pass-1234
join --space <SPACE_ID_FROM_P1.3>
send --space <SPACE_ID_FROM_P1.3> --room <ROOM_ID_FROM_P1.3> --text "bob-msg-1"
status
```

### B.3 — `docs/tests/scripts/multiparty_s1_smoke_clientC.xgb`

```
# Multiparty S1 P1 — Client C (carol) smoke script
# Phase: register, join existing space, send one message
# Space ID and Room ID MUST be substituted from P1.3 output before run

connect ws://127.0.0.1:8080/xgen
register --name carol --passphrase m1c-pass-1234
join --space <SPACE_ID_FROM_P1.3>
send --space <SPACE_ID_FROM_P1.3> --room <ROOM_ID_FROM_P1.3> --text "carol-msg-1"
status
```

---

## Appendix C — P2 Stress `.xgb` scripts

### C.1 — `docs/tests/scripts/multiparty_s1_stress_clientA.xgb` (bootstrap)

```
# Multiparty S1 P2 — Client A (alice) stress bootstrap
# Sets up the Space and Room for the stress phase. No sends here.

connect ws://127.0.0.1:8080/xgen
register --name alice --passphrase m1a-pass-1234
create-space --name "Multiparty S1 P2"
create-room --space @last_space --name general
status
```

### C.2 — `docs/tests/scripts/multiparty_s1_stress_clientB.xgb` (bootstrap)

```
# Multiparty S1 P2 — Client B (bob) stress bootstrap
# Joins the Space created by Client A. No sends here.

connect ws://127.0.0.1:8080/xgen
register --name bob --passphrase m1b-pass-1234
join --space <SPACE_ID_FROM_P2.3>
status
```

### C.3 — `docs/tests/scripts/multiparty_s1_stress_clientC.xgb` (bootstrap)

```
# Multiparty S1 P2 — Client C (carol) stress bootstrap
# Joins the Space created by Client A. No sends here.

connect ws://127.0.0.1:8080/xgen
register --name carol --passphrase m1c-pass-1234
join --space <SPACE_ID_FROM_P2.3>
status
```

### C.4 — `docs/tests/scripts/multiparty_s1_stress_clientA_send.xgb`

100 `send` commands. Generated mechanically — the first three and last three shown; lines 4–97 follow the same pattern.

```
# Multiparty S1 P2 — Client A (alice) stress send
# 100 messages sent back-to-back

send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-001"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-002"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-003"
# ... lines 4 through 97 follow the same pattern, incrementing the suffix ...
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-098"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-099"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-100"
```

### C.5 — `docs/tests/scripts/multiparty_s1_stress_clientB_send.xgb`

Same shape as C.4 but with `bob-stress-NNN` texts.

### C.6 — `docs/tests/scripts/multiparty_s1_stress_clientC_send.xgb`

Same shape as C.4 but with `carol-stress-NNN` texts.

---

## Sequence Position

| | |
|---|---|
| **This file** | 1 of 5 |
| **Previous** | — (first in the suite) |
| **Next** | `MULTIPARTY_S2_concurrent_send.md` |

Do not advance to the next file until this one's Definition of Done is fully ticked.

---

*End of MULTIPARTY_S1_multiclient_one_node.md*
