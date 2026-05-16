# Multiparty Test S2 — DAG Under Concurrent Writes
> **Status**: PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Operation

This is file **2 of 5** in the **Multiparty** test operation.

**Full sequence (locked execution order):**

1. `MULTIPARTY_S1_multiclient_one_node.md` — multiple clients per Node
2. **`MULTIPARTY_S2_concurrent_send.md`** — this file — DAG under concurrent writes
3. `MULTIPARTY_S3_federation_topology.md` — 3+ Node federation, transitive
4. `MULTIPARTY_S4_n_clients_n_nodes.md` — N clients across N Nodes
5. `MULTIPARTY_S5_client_rebind.md` — one client across multiple Nodes

Each file in the suite must be COMPLETED before the next begins.

---

## Purpose

Verify that the protocol's append-only DAG remains coherent when two or more clients on **different Nodes** author events simultaneously. The Phase 1 smoke test and S1 covered sequential authorship, where each event has a clean causal parent. This test introduces the harder case: two events authored at essentially the same wall-clock moment, on different Nodes, both descending from the same set of DAG tips.

**What this test proves:**

- When two clients send concurrently from different Nodes, both events are accepted by both Nodes.
- The resulting DAG has both events as siblings (sharing one or more parents), with `prev_events` correctly pointing to the tips that were known at authoring time.
- Federation correctly propagates concurrent events in both directions without deadlock, livelock, or message loss.
- Deterministic ordering rules (lexicographic on `event_id` after timestamp, per spec) produce identical resolved order on both Nodes for any observer.
- No event is lost, duplicated, or rewritten on either Node.
- The `prev_events` fanin limit (10, Phase 1) is respected and the test does not approach it.

**What this test does NOT prove:**

- Behaviour with 3+ concurrent authors (covered by S4).
- Transitive federation hops (covered by S3).
- Identity portability across Nodes (covered by S5).
- Network partition / heal scenarios (out of scope for the Multiparty suite — separate Phase 3 work).
- Performance under sustained concurrent load above the modest counts used here.

---

## Prerequisites

This test depends on the following being COMPLETED:

- `MULTIPARTY_S1_multiclient_one_node.md` — verifies single-Node fan-out works before adding federation.
- Phase 1 smoke test (`SMOKETEST_ph1.md`) — verifies 2-Node federation handshake.
- `BATCH_FLAG_ph2.md` — provides `--batch` mechanism.
- Phase 2 integration test (J-058, 60/60 PASS) — verifies multi-event Phase 2 protocol surfaces work.
- `STRESSTEST_ph1_findings.md` — informs expected behaviour under stress (F-001 federation DAG ordering resolved).

**Required binaries:**

- `xgen-node-app.exe`
- `xgen-client-app.exe`

**Required spec sections (read before execution):**

- Ch3 §3.5 — DAG event store (append-only invariant, `prev_events`, tips, pending buffer)
- Ch3 §3.6 — Federation handshake
- Ch3 §3.7 — Space and Room protocol (membership, message events)
- Ch3 §3.9 — State resolution (Phase 2 — deterministic ordering rules under concurrent writes)
- `STRESSTEST_ph1_findings.md` F-001 — known-good behaviour of the pending-event buffer

**Spec cross-check required before P1 execution:**

- **Concurrent-write ordering rule.** Locate the canonical statement in spec §3.9 that defines how two events with overlapping `prev_events` and similar timestamps are ordered. Common conventions: lexicographic sort on `event_id` after timestamp tiebreaker. The exact rule must be quoted into the findings file at preparation time so the observed ordering can be verified, not guessed.

---

## Scope

### In scope

- 2 Nodes federated, 2 Clients (one per Node).
- 1 Space, 1 Room, both clients members.
- P1 — single concurrent-send pair: both clients send "at the same time", DAG inspection confirms sibling structure and identical ordering on both Nodes.
- P2 — sustained concurrent sending from both clients for a fixed duration; DAG integrity, no duplicates, no orphans.

### Out of scope

- 3+ concurrent authors (S4).
- Transitive federation (S3).
- Identity rebinding (S5).
- Single-Node fan-out (S1).
- Network partitions, dropped packets, or recovery scenarios.
- E2E encryption tier verification.
- True load testing beyond the modest counts specified.

---

## Architecture Constraints — Non-Negotiable

These rules apply before any other implementation decision. An implementation that violates any of them is non-compliant.

**Use only existing infrastructure.** No new CLI commands, no new event types, no new protocol surfaces. Test runs against current binaries. If a capability is missing, stop and report — do not improvise.

**No shell invocation.** Orchestration via `--batch` and named pipes (D-043) only.

**Distinct Node and Client instance labels.** Use `m2nA`, `m2nB` for Nodes; `m2a`, `m2b` for Clients. Distinct named pipes, distinct data directories, distinct keypairs.

**"Concurrent" is a wall-clock requirement, not a vibe.** The concurrent-send phase requires two `send` commands to be dispatched within a measured window. The window must be **smaller than the federation round-trip time** for the test to be meaningful. Record the dispatch timestamps to sub-millisecond resolution. If the harness cannot guarantee dispatch within the window, the test is invalid.

**Sub-second dispatch window — target 50 ms or tighter.** This is the wall-clock budget between the first and last `send` command leaving the client process. On localhost loopback, federation round-trips run in single-digit milliseconds; a 50 ms window is comfortably below that. If localhost RTT differs significantly, adjust the window downward, not upward.

**Stop on first failure.** P1 failure halts the test; do not run P2.

**Honesty.** Per CLAUDE.md Rules 1–7. No fabricated event counts, no invented DAG inspections, no PASS verdict without verified evidence.

**Findings file is the write surface.** All runtime data goes to `MULTIPARTY_S2_findings.md`. This instruction file is read-only during execution except for the final DoD checklist.

---

## Topology

```
   ┌──────────────────────────┐         ┌──────────────────────────┐
   │      xgen-node-app       │◀═══════▶│      xgen-node-app       │
   │      --instance m2nA     │  fed.   │      --instance m2nB     │
   │  ws://127.0.0.1:8080/xgen│         │  ws://127.0.0.1:8081/xgen│
   │                          │         │                          │
   │   Space: S_multiparty_2  │         │   Space: S_multiparty_2  │
   │   Room:  R_general       │         │   Room:  R_general       │
   └────────────┬─────────────┘         └────────────┬─────────────┘
                │                                    │
            ┌───┴───┐                            ┌───┴───┐
            │client │                            │client │
            │  m2a  │                            │  m2b  │
            │alice  │                            │bob    │
            └───────┘                            └───────┘
```

Two Nodes federated over `ws://`. One client per Node. Both clients are members of the same Space and Room (created on Node A, joined on Node B via federation).

---

## Test Data and Identifiers

| Item | Value |
|---|---|
| Node A instance label | `m2nA` |
| Node A endpoint | `ws://127.0.0.1:8080/xgen` |
| Node B instance label | `m2nB` |
| Node B endpoint | `ws://127.0.0.1:8081/xgen` |
| Client A instance label | `m2a` |
| Client A display name | `alice` |
| Client A passphrase (test) | `m2a-pass-1234` |
| Client A connects to | Node A |
| Client B instance label | `m2b` |
| Client B display name | `bob` |
| Client B passphrase (test) | `m2b-pass-1234` |
| Client B connects to | Node B |
| Space name (P1) | `Multiparty S2 P1` |
| Room name (P1) | `general` |
| Space name (P2) | `Multiparty S2 P2` |
| Room name (P2) | `general` |
| P1 concurrent dispatch window | ≤ 50 ms |
| P2 messages per client | 50 |
| P2 concurrent dispatch window | ≤ 1 s (per round) |
| P2 number of rounds | 10 |

Passphrases are test fixtures only.

---

## Milestone 0 — Preparation

**0.1 — Create findings file.**

Create `docs/tests/MULTIPARTY_S2_findings.md` from the template in Appendix A. Status `ACTIVE`.

**0.2 — Record binary versions.**

```
xgen-node-app.exe --version
xgen-client-app.exe --version
```

Both binaries from the same build. Record exact strings.

**0.3 — Spec cross-check: concurrent-write ordering rule.**

Open `docs/xgen_ch3_specification.md` §3.9 (state resolution). Locate the rule that defines deterministic ordering between two events with overlapping `prev_events`. Quote it verbatim into the findings file under "Preparation record → Ordering rule". This rule is the expected behaviour the test verifies.

If §3.9 does not specify the rule, **stop and escalate**. Do not run P1 with the ordering rule undefined — the test would have no pass criterion.

**0.4 — Clean workspace.**

For each of `m2nA`, `m2nB`, `m2a`, `m2b`: ensure no prior data exists. Archive prior data to `test_runs/multiparty_s2_<timestamp>_pre/` if present. Record paths in findings.

**0.5 — Validate `.xgb` scripts.**

Verify all scripts in Appendix B (P1) and Appendix C (P2) exist at `docs/tests/scripts/` with contents matching the spec.

**0.6 — Measure baseline federation RTT.**

Before running the test, measure the federation round-trip time on the harness machine:

1. Start Node A and Node B.
2. Federate them (per Phase 1 smoke test step).
3. From Node A's log, measure the time between dispatching a single test event and receiving its acknowledgment from Node B.

Record the RTT in findings. The P1 dispatch window (50 ms) must be at least 5× smaller than the RTT for the concurrency to be meaningful. If RTT is below 10 ms, narrow the dispatch window proportionally and record the adjusted value.

### Definition of Done — Milestone 0

- [ ] Findings file `MULTIPARTY_S2_findings.md` created with correct header, status `ACTIVE`.
- [ ] Binary versions recorded.
- [ ] Spec §3.9 concurrent-ordering rule quoted into findings.
- [ ] Workspace clean or archived; paths recorded.
- [ ] All scripts present and validated.
- [ ] Federation RTT measured and recorded; dispatch window confirmed to be at least 5× smaller.

---

## Milestone 1 — P1 Smoke

**Goal:** Two clients on two federated Nodes each send one message within a tight dispatch window. The resulting DAG must show both events as siblings (sharing parents), present on both Nodes, and ordered identically by the spec's rules on both Nodes.

P1 is the warm-up. Single round, exhaustive inspection.

### Sequence

**Step P1.1 — Start Node A.**

```
xgen-node-app.exe --instance m2nA
```

Wait for `READY`. Record timestamp.

**Step P1.2 — Start Node B.**

```
xgen-node-app.exe --instance m2nB
```

Wait for `READY`. Record timestamp.

**Step P1.3 — Federate A ↔ B.**

Use the standard federation command (the same used in Phase 1 smoke test). Wait for `state.federation_add` to appear in both Node logs with matching `event_id`. Record federation event_id in findings.

**Step P1.4 — Start Client A, register, create Space and Room.**

```
xgen-client-app.exe --instance m2a
xgen-client-app.exe --instance m2a --batch docs/tests/scripts/multiparty_s2_smoke_clientA_setup.xgb
```

Setup script (Appendix B.1): `connect ws://127.0.0.1:8080/xgen`, `register alice`, `create-space "Multiparty S2 P1"`, `create-room general`. Capture Space ID and Room ID. Wait for exit 0.

**Step P1.5 — Start Client B, register on Node B, join the Space.**

```
xgen-client-app.exe --instance m2b
xgen-client-app.exe --instance m2b --batch docs/tests/scripts/multiparty_s2_smoke_clientB_setup.xgb
```

Setup script (Appendix B.2): `connect ws://127.0.0.1:8081/xgen`, `register bob`, `join <Space ID>`. Wait for exit 0.

**Step P1.6 — Settle.**

Wait 3 seconds for the Space state to fully propagate. Confirm both Nodes show both clients as members (via `status` command or log inspection).

**Step P1.7 — Concurrent send.**

This is the key step. Both clients send one message each, dispatched within the measured window (target ≤ 50 ms, adjusted per M0.6).

Dispatch:

```
xgen-client-app.exe --instance m2a --batch docs/tests/scripts/multiparty_s2_smoke_clientA_send.xgb
xgen-client-app.exe --instance m2b --batch docs/tests/scripts/multiparty_s2_smoke_clientB_send.xgb
```

These two commands MUST be initiated by the harness within the dispatch window. The harness records the exact wall-clock timestamp of each `--batch` invocation start (not when the script begins execution inside the running client — when the second process is spawned). Record both timestamps to sub-millisecond resolution.

Each send script (Appendix B.3, B.4) is a single `send` command:

- m2a sends: `alice-concurrent-1`
- m2b sends: `bob-concurrent-1`

Wait for both exit codes (0 expected).

**Step P1.8 — Settle.**

Wait 5 seconds for both events to propagate to both Nodes and settle in their respective stores.

**Step P1.9 — DAG inspection.**

Read the DAG state from both Nodes (via existing inspection commands — `status`, log analysis, or direct SQLite query of the event store; verify which is appropriate per Ch3 §3.5). For each of the two concurrent events, record:

- `event_id`
- `prev_events` array
- Authored-by Identity
- Timestamp
- Containing Node store (A, B, or both)

**Step P1.10 — Shutdown.**

Clean shutdown: Clients A and B, then Nodes A and B.

### Expected DAG structure after P1.7

The two concurrent message events are expected to be **siblings** in the DAG:

```
        ... earlier events: membership joins, room create, etc. ...
                            │
                       ┌────┴────┐
                       │ tip(s)  │
                       └────┬────┘
                            │
              ┌─────────────┴─────────────┐
              │                           │
   ┌──────────┴──────────┐    ┌──────────┴──────────┐
   │ message.text        │    │ message.text        │
   │ "alice-concurrent-1"│    │ "bob-concurrent-1"  │
   │ event_id: 0xAAA...  │    │ event_id: 0xBBB...  │
   │ prev_events: [tip]  │    │ prev_events: [tip]  │
   └─────────────────────┘    └─────────────────────┘
```

**Both events** must share at least one parent in `prev_events`. The exact `prev_events` set may differ between the two if the two clients had different tip views at authoring time — but the intersection must be non-empty (both must descend from the same Space state).

**Both Nodes** (A and B) must show both events in their stores after settle.

**Ordering:** by the rule quoted from spec §3.9 in M0.3, the two events have a deterministic order when iterated. Both Node A and Node B must produce the same order. Record the order observed on each Node in the findings file.

### Pairing table — concurrent events

| event_id (12-char prefix) | Author | Authored on Node | In Node A store | In Node B store | Resolved order rank on A | Resolved order rank on B |
|---|---|---|---|---|---|---|
| _aaa..._ | m2a (alice) | A | ✔ | ✔ | _N_ | _N_ |
| _bbb..._ | m2b (bob) | B | ✔ | ✔ | _N+1_ | _N+1_ |

The "resolved order rank" must be identical between A and B for each event.

### Content-leak check

```
findstr /S /M /R "alice-concurrent-1\|bob-concurrent-1" *.log
```

Across all client and Node logs. Message text appears only in `message.text` event-handling log lines. Zero unauthorised occurrences.

### Definition of Done — Milestone 1

- [ ] Both Nodes started cleanly, federated, both in `READY`.
- [ ] Both Clients started, registered, joined the Space.
- [ ] Concurrent send dispatch window measured; recorded; within the budget set in M0.6.
- [ ] Both events present in both Node stores (4 ✔ in pairing table).
- [ ] Both events share at least one parent in `prev_events` (recorded).
- [ ] Resolved order identical on A and B (recorded).
- [ ] Content-leak check clean.
- [ ] No `ERROR` or unexpected `WARN` log lines.
- [ ] P1 verdict recorded: PASS or FAIL.
- [ ] If FAIL: stop, do not proceed to P2.

---

## Milestone 2 — P2 Stress

**Goal:** Sustained concurrent sending from both clients across multiple rounds. Verify the DAG remains coherent — no orphans, no duplicates, identical resolved order on both Nodes — over a meaningful volume of events.

### Sequence

**Step P2.1 — Workspace and Node startup.**

Same as P1.1–P1.3 but with fresh data directories. Federate. Use a new Space (`Multiparty S2 P2`).

**Step P2.2 — Client bootstrap.**

Sequentially run the setup scripts (Appendix C.1, C.2): Client A creates Space and Room; Client B joins. Wait 3 seconds for state propagation.

**Step P2.3 — Concurrent send rounds.**

Run **10 rounds** of concurrent dispatch. Each round consists of:

- Both clients dispatch a `send` `.xgb` script within a ≤ 1 s window.
- Each script sends 5 messages back-to-back (so 10 messages per round, 5 per client).
- After dispatch, wait 2 seconds before starting the next round.

Round message texts (deterministic, so we can grep by exact value):

- Round R (R = 1..10), Client A messages: `alice-stress-R-1` through `alice-stress-R-5`
- Round R, Client B messages: `bob-stress-R-1` through `bob-stress-R-5`

Total: **100 messages** authored (50 per client) across **10 concurrent-write rounds**.

Send scripts (Appendix C.3 through C.22 — one per client per round) are listed at the end of this document.

Record per-round dispatch timestamps for both clients in findings.

**Step P2.4 — Drain.**

Wait 30 seconds after the final round for all events to settle on both Nodes.

**Step P2.5 — DAG integrity scan.**

For each Node, programmatically check:

- Total `message.text` event count = 100 (zero loss).
- Zero duplicate `event_id`s.
- Zero orphan events (`prev_events` references that don't resolve to a stored event).
- DAG is acyclic (per the protocol invariant, this should be impossible to violate, but verify).
- Every `message.text` event from the test exists in **both** Node stores.

For the **resolved iteration order**: dump the full set of 100 `message.text` events in the order produced by the spec §3.9 rule on each Node. The two dumps must be **byte-identical**. Diff them; record diff output (empty = pass).

**Step P2.6 — Shutdown.**

Clean shutdown order: Clients A, B; Nodes A, B.

### Metrics to capture

**Event counts (per Node):**

| Metric | Expected | Observed on A | Observed on B |
|---|---|---|---|
| `message.text` events authored by m2a | 50 | _ | _ |
| `message.text` events authored by m2b | 50 | _ | _ |
| Total `message.text` in store | 100 | _ | _ |
| Duplicate `event_id`s | 0 | _ | _ |
| Orphan events at end | 0 | _ | _ |

**Cross-Node consistency:**

| Check | Expected | Observed |
|---|---|---|
| Iteration order on A matches iteration order on B (diff) | empty | _ |
| Every event in A's store also in B's store | yes | _ |
| Every event in B's store also in A's store | yes | _ |

**Latency (informational):**

| Metric | Observed |
|---|---|
| Median federation delivery time (Outbound on origin Node → Inbound on remote Node) | _ |
| p95 federation delivery time | _ |
| Max federation delivery time | _ |

**Log hygiene:**

| Metric | Expected | Observed |
|---|---|---|
| `ERROR` log lines across both Node logs | 0 | _ |
| `ERROR` log lines across both Client logs | 0 | _ |
| `WARN` log lines (non-shutdown) | 0 expected; record any | _ |
| Pending buffer events stuck at shutdown (F-001 regression check) | 0 | _ |

### Definition of Done — Milestone 2

- [ ] All 10 concurrent-send rounds completed.
- [ ] 100 messages authored total (50 per client).
- [ ] Both Node stores contain all 100 events.
- [ ] Zero duplicates on either Node.
- [ ] Zero orphans on either Node.
- [ ] Cross-Node iteration order diff is empty.
- [ ] Zero `ERROR` log lines.
- [ ] `WARN` lines (if any) classified in findings.
- [ ] F-001 regression check: no events stuck in pending buffer at shutdown.
- [ ] Latency metrics recorded.
- [ ] P2 verdict recorded: PASS or FAIL.

---

## Definition of Done — Test S2 as a whole

- [ ] Milestone 0 (Preparation) all items ticked, including spec §3.9 rule quoted.
- [ ] Milestone 1 (P1 Smoke) all items ticked, verdict PASS.
- [ ] Milestone 2 (P2 Stress) all items ticked, verdict PASS.
- [ ] Findings file status set to `COMPLETED` with overall verdict.
- [ ] JOURNAL.md entry written.
- [ ] This instruction file's header status updated from `ACTIVE` to `COMPLETED`.
- [ ] Any bugs that surfaced linked from findings file.

After all items above are ticked, sequence advances to file 3/5 (`MULTIPARTY_S3_federation_topology.md`).

---

## Appendix A — Findings file template

When M0.1 creates `docs/tests/MULTIPARTY_S2_findings.md`, use this template:

```markdown
# Multiparty Test S2 — Findings
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
- Federation RTT measured: _N ms_
- Dispatch window adjusted to: _N ms_
- Spec §3.9 concurrent-ordering rule (verbatim quote):

  > _quote here_

---

## Milestone 1 — P1 Smoke

### Federation event
- federation event_id: _xgen://hash/sha256:..._

### Concurrent dispatch
- m2a dispatch timestamp: _HH:MM:SS.mmm_
- m2b dispatch timestamp: _HH:MM:SS.mmm_
- Window: _N ms_

### Two concurrent events
- alice-concurrent-1 event_id: _..._
  - prev_events: _[...]_
  - Authored on: Node A
  - In Node A store: ✔/✘
  - In Node B store: ✔/✘
- bob-concurrent-1 event_id: _..._
  - prev_events: _[...]_
  - Authored on: Node B
  - In Node A store: ✔/✘
  - In Node B store: ✔/✘
- Shared parents (intersection of prev_events): _[...]_

### Resolved order
- Order on Node A: _alice first / bob first / other_
- Order on Node B: _alice first / bob first / other_
- Match: yes/no

### Content-leak findstr output

```
_(paste verbatim)_
```

### Verdict: _PASS/FAIL_

_Notes:_

---

## Milestone 2 — P2 Stress

### Per-round dispatch timestamps

| Round | m2a dispatch | m2b dispatch | Window (ms) |
|---|---|---|---|
| 1 | _ | _ | _ |
| 2 | _ | _ | _ |
| ... |

### Event counts

_(insert tables per the instruction file's format)_

### Cross-Node consistency

- Order diff: empty / non-empty (paste diff if non-empty)

### Latency metrics

_(insert table)_

### Verdict: _PASS/FAIL_

_Notes:_

---

## Findings — bugs and anomalies

### F-001 — _short title_
- **Severity:** _critical / major / minor_
- **Stage:** _P1 step / P2 step / round N_
- **Observed:** _what happened_
- **Expected:** _what should have happened_
- **Resolution:** _link to FIXES file or commit, or "open"_

---

## Overall verdict

_PASS / FAIL / BLOCKED_
```

---

## Appendix B — P1 Smoke `.xgb` scripts

### B.1 — `docs/tests/scripts/multiparty_s2_smoke_clientA_setup.xgb`

```
# Multiparty S2 P1 — Client A (alice) setup
# Connect to Node A, register, create Space and Room

connect ws://127.0.0.1:8080/xgen
register --name alice --passphrase m2a-pass-1234
create-space --name "Multiparty S2 P1"
create-room --space @last_space --name general
status
```

### B.2 — `docs/tests/scripts/multiparty_s2_smoke_clientB_setup.xgb`

```
# Multiparty S2 P1 — Client B (bob) setup
# Connect to Node B, register, join the Space created on Node A
# Space ID must be substituted from P1.4 output before run

connect ws://127.0.0.1:8081/xgen
register --name bob --passphrase m2b-pass-1234
join --space <SPACE_ID_FROM_P1.4>
status
```

### B.3 — `docs/tests/scripts/multiparty_s2_smoke_clientA_send.xgb`

```
# Multiparty S2 P1 — Client A (alice) concurrent send
# Single message, dispatched within the concurrent window

send --space <SPACE_ID> --room <ROOM_ID> --text "alice-concurrent-1"
```

### B.4 — `docs/tests/scripts/multiparty_s2_smoke_clientB_send.xgb`

```
# Multiparty S2 P1 — Client B (bob) concurrent send
# Single message, dispatched within the concurrent window

send --space <SPACE_ID> --room <ROOM_ID> --text "bob-concurrent-1"
```

**Note on `@last_space` / `@last_room` placeholders:** same caveat as S1. Verify whether the batch dispatcher supports backreferences. If not, regenerate scripts with literal IDs after the setup phase captures them.

---

## Appendix C — P2 Stress `.xgb` scripts

### C.1 — `docs/tests/scripts/multiparty_s2_stress_clientA_setup.xgb`

```
# Multiparty S2 P2 — Client A (alice) setup
# Same as B.1 but with P2 Space name

connect ws://127.0.0.1:8080/xgen
register --name alice --passphrase m2a-pass-1234
create-space --name "Multiparty S2 P2"
create-room --space @last_space --name general
status
```

### C.2 — `docs/tests/scripts/multiparty_s2_stress_clientB_setup.xgb`

```
# Multiparty S2 P2 — Client B (bob) setup
# Same as B.2 with P2 Space

connect ws://127.0.0.1:8081/xgen
register --name bob --passphrase m2b-pass-1234
join --space <SPACE_ID_FROM_P2.2>
status
```

### Per-round send scripts (C.3 through C.22)

Twenty scripts, two per round (one for each client), for 10 rounds.

Naming pattern:

- `multiparty_s2_stress_clientA_round_R_send.xgb` — Client A's 5 messages for round R
- `multiparty_s2_stress_clientB_round_R_send.xgb` — Client B's 5 messages for round R

Each script contains 5 `send` lines. Example for round 1, Client A:

```
# Multiparty S2 P2 — Client A round 1
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-1-1"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-1-2"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-1-3"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-1-4"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-1-5"
```

Round 1 Client B:

```
# Multiparty S2 P2 — Client B round 1
send --space <SPACE_ID> --room <ROOM_ID> --text "bob-stress-1-1"
send --space <SPACE_ID> --room <ROOM_ID> --text "bob-stress-1-2"
send --space <SPACE_ID> --room <ROOM_ID> --text "bob-stress-1-3"
send --space <SPACE_ID> --room <ROOM_ID> --text "bob-stress-1-4"
send --space <SPACE_ID> --room <ROOM_ID> --text "bob-stress-1-5"
```

**Generation rule:** for round R (R = 1..10) and client X ∈ {A, B}, lines 1..5:

```
send --space <SPACE_ID> --room <ROOM_ID> --text "{author}-stress-{R}-{N}"
```

where `{author}` is `alice` for A and `bob` for B, `{R}` is the round number (1..10), `{N}` is the message index within the round (1..5).

All 20 scripts can be generated programmatically. Alternatively, two parameterised scripts can be reused if the batch dispatcher supports variable substitution (verify against `BATCH_FLAG_ph2.md` — if not supported, scripts must be expanded to literals).

---

## Sequence Position

| | |
|---|---|
| **This file** | 2 of 5 |
| **Previous** | `MULTIPARTY_S1_multiclient_one_node.md` |
| **Next** | `MULTIPARTY_S3_federation_topology.md` |

Do not advance to the next file until this one's Definition of Done is fully ticked.

---

*End of MULTIPARTY_S2_concurrent_send.md*
