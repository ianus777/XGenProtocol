# Multiparty Test S4 — N Clients Across N Nodes (Real Chat-Room)
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

This is file **4 of 5** in the **Multiparty** test operation.

**Full sequence (locked execution order):**

1. `MULTIPARTY_S1_multiclient_one_node.md` — multiple clients per Node
2. `MULTIPARTY_S2_concurrent_send.md` — DAG under concurrent writes
3. `MULTIPARTY_S3_federation_topology.md` — 3+ Node federation, transitive
4. **`MULTIPARTY_S4_n_clients_n_nodes.md`** — this file — N clients across N Nodes
5. `MULTIPARTY_S5_client_rebind.md` — one client across multiple Nodes

Each file in the suite must be COMPLETED before the next begins.

---

## Purpose

Verify that XGen behaves correctly in the **realistic chat-room scenario** — multiple Nodes, multiple Clients per Node, all sharing one Space and Room, all sending messages concurrently. This combines every dimension previous tests covered separately:

- **Local fan-out (S1):** multiple Clients on the same Node must each receive Events authored by their Node-neighbours.
- **Federation topology (S3):** Events must propagate across all federated Nodes (full mesh in this test).
- **Concurrent writes (S2):** Clients on different Nodes author Events at overlapping times; DAG ordering and convergence must hold.

This is the closest test to what a real XGen deployment looks like in production — and the most likely place where bugs hidden by smaller scenarios surface.

**What this test proves:**

- 4 Nodes federated full mesh, each accepting multiple Client WebSocket connections, do not deadlock, leak memory, or drop Events under normal sustained chat load.
- 6 Clients across the 4 Nodes (uneven distribution) all converge to identical Event histories.
- Concurrent sending from all 6 Clients across all 4 Nodes produces a DAG that resolves to the same ordering on every Node per §3.9.
- Local fan-out and federation fan-out compose correctly — a Client receives an Event from a Node-neighbour Client as quickly as from any other Client via federation.
- No duplicate event_ids in any Node's store despite the multiple federation paths available in full mesh.

**What this test does NOT prove:**

- Identity portability across Nodes (S5).
- Scalability beyond 4 Nodes / 6 Clients — this test fixes those counts.
- Network partition / heal scenarios.
- E2E encryption tier verification.
- Performance under genuine load (this is correctness-first, modest volume).
- Behaviour under adversarial conditions (malicious Nodes, signature attacks, replay).

---

## Prerequisites

This test depends on the following being COMPLETED:

- `MULTIPARTY_S1_multiclient_one_node.md` — local fan-out works.
- `MULTIPARTY_S2_concurrent_send.md` — concurrent DAG writes work across 2 Nodes.
- `MULTIPARTY_S3_federation_topology.md` — transitive and full-mesh federation works across 3 Nodes.
- Phase 1 smoke test, Phase 2 integration test — both COMPLETED.
- `BATCH_FLAG_ph2.md` — `--batch` available.

**Required binaries:**

- `xgen-node-app.exe`
- `xgen-client-app.exe`

**Required spec sections (read before execution):**

- Ch3 §3.2 — Event Specification (DAG, validation pipeline).
- Ch3 §3.4 — Federation Handshake.
- Ch3 §3.7 — Space and Room Protocol.
- Ch3 §3.9 — State Resolution Algorithm (the convergence guarantee carries the test).
- `MULTIPARTY_S3_findings.md` — review for any topology-related anomalies observed.

---

## Scope

### In scope

- 4 Nodes federated **full mesh** (every Node ↔ every Node). 6 federation channels total.
- 6 Clients distributed across the Nodes: **2 + 2 + 1 + 1** (Node A has 2, Node B has 2, Node C has 1, Node D has 1). The uneven distribution is intentional — equal distributions mask bugs that depend on load asymmetry.
- 1 Space, 1 Room, all 6 Clients members.
- P1 — single round: each Client sends one message. Verify pairing across all 6 Clients × 4 Nodes.
- P2 — sustained chat: each Client sends 50 messages, all 6 dispatched concurrently for a fixed duration. 300 total messages.

### Out of scope

- More than 4 Nodes; more than 6 Clients.
- Identity rebinding (S5).
- Concurrent writes from arbitrarily many authors (the count is fixed at 6).
- Mixed topology cases — full mesh only for this test. Chain topology with multiple clients per node is theoretically a sub-case; not tested here.
- High-volume load (300 messages is correctness-first volume; load testing is separate work).
- Long-running stability beyond test duration.

---

## Architecture Constraints — Non-Negotiable

**Use only existing infrastructure.** No new CLI commands, no new event types.

**No shell invocation.** `--batch` and named pipes only.

**Distinct instance labels.** Nodes: `m4nA`, `m4nB`, `m4nC`, `m4nD`. Clients: `m4a1`, `m4a2` (on Node A); `m4b1`, `m4b2` (on Node B); `m4c1` (on Node C); `m4d1` (on Node D). Ten distinct named pipes total. Distinct data directories and keypairs for every instance.

**Full-mesh topology is enforced, not assumed.** Six federation channels must be confirmed in each Node's federation registry before P1 begins. Verify each Node's registry contains the other three Nodes.

**Volume is modest by design.** 300 messages in P2 is below true-load territory. This test verifies that **correctness composes** across all three dimensions (fan-out × federation × concurrency), not that the system handles thousands of messages per second. If P2 reveals load-related bugs, that's a finding worth recording — but the test is not designed as a load test.

**Concurrent dispatch is wall-clock measurable.** P2 requires all 6 Clients to begin their send phase within a fixed dispatch window (target ≤ 2 s). Recorded timestamps must demonstrate this.

**Stop on first failure.** P1 failure halts the test; do not run P2. Within P2, a partial failure (e.g. one Node missing 3 Events) is still a failure — the test does not have "soft pass" modes.

**Honesty.** Per CLAUDE.md Rules 1–7. No invented event counts. No claimed convergence without byte-level diffs.

**Findings file is the write surface.** All runtime data goes to `MULTIPARTY_S4_findings.md`.

---

## Topology

```
                          ┌─── full-mesh federation: 6 channels ───┐

  ┌────────────────────┐     ┌────────────────────┐     ┌────────────────────┐     ┌────────────────────┐
  │   xgen-node-app    │◀═══▶│   xgen-node-app    │◀═══▶│   xgen-node-app    │◀═══▶│   xgen-node-app    │
  │  --instance m4nA   │◀═══▶│  --instance m4nB   │◀═══▶│  --instance m4nC   │     │  --instance m4nD   │
  │ ws://...8080/xgen  │ ┌──▶│ ws://...8081/xgen  │     │ ws://...8082/xgen  │◀═══▶│ ws://...8083/xgen  │
  └────────┬───────────┘ │   └────────┬───────────┘     └────────┬───────────┘ ▲   └────────┬───────────┘
           │             │            │                          │             │            │
           │             └────────────┼──────────────────────────┼─────────────┘            │
           │                          │                          │                          │
           │                          │                  ┌───────┴───────┐                  │
       ┌───┴───┐  ┌───┐           ┌───┴───┐  ┌───┐       │    client     │              ┌───┴───┐
       │client │  │   │           │client │  │   │       │     m4c1      │              │client │
       │ m4a1  │  │m4a│           │ m4b1  │  │m4b│       │    carol      │              │ m4d1  │
       │alice1 │  │ 2 │           │ bob1  │  │ 2 │       └───────────────┘              │ dave  │
       └───────┘  └───┘           └───────┘  └───┘                                       └───────┘

  Node A: 2 Clients          Node B: 2 Clients          Node C: 1 Client                Node D: 1 Client
```

Full-mesh federation. Client distribution: 2, 2, 1, 1 (total 6).

(The ASCII art is approximate — the test harness verifies the actual topology programmatically in M0 and P1.2.)

---

## Test Data and Identifiers

| Item | Value |
|---|---|
| Node A | `m4nA` at `ws://127.0.0.1:8080/xgen` |
| Node B | `m4nB` at `ws://127.0.0.1:8081/xgen` |
| Node C | `m4nC` at `ws://127.0.0.1:8082/xgen` |
| Node D | `m4nD` at `ws://127.0.0.1:8083/xgen` |
| Client A1 | `m4a1` (alice1, `m4a1-pass-1234`) — connects to Node A |
| Client A2 | `m4a2` (alice2, `m4a2-pass-1234`) — connects to Node A |
| Client B1 | `m4b1` (bob1, `m4b1-pass-1234`) — connects to Node B |
| Client B2 | `m4b2` (bob2, `m4b2-pass-1234`) — connects to Node B |
| Client C1 | `m4c1` (carol, `m4c1-pass-1234`) — connects to Node C |
| Client D1 | `m4d1` (dave, `m4d1-pass-1234`) — connects to Node D |
| Space name (P1) | `Multiparty S4 P1` |
| Space name (P2) | `Multiparty S4 P2` |
| Room name (both) | `general` |
| P2 messages per Client | 50 |
| P2 total messages | 300 |
| P2 dispatch window | ≤ 2 s |
| Settle wait after final send | 60 s |

Note: display names `alice1` and `alice2` are intentionally similar but distinct. They are different Identities with different keypairs. Same convention for `bob1` / `bob2`. This catches bugs that confuse Clients by display-name proximity.

---

## Milestone 0 — Preparation

**0.1 — Create findings file.**

Create `docs/tests/MULTIPARTY_S4_findings.md` from Appendix A. Status `ACTIVE`.

**0.2 — Record binary versions.**

```
xgen-node-app.exe --version
xgen-client-app.exe --version
```

**0.3 — Spec cross-check.**

Confirm spec sections listed in Prerequisites are present and unchanged. Quote §3.9.2 (convergence guarantee) into findings.

Cross-reference `MULTIPARTY_S3_findings.md`: was the spec gap on "forward on accept" (§3.2) recorded? If yes, note here. If S3 produced any FIXES, confirm those fixes are in the current build before S4 begins.

**0.4 — Clean workspace.**

All 10 `m4*` data directories cleared or archived. Record paths.

**0.5 — Validate scripts.**

Confirm all P1 scripts (Appendix B, 12 scripts: 6 setup + 6 send) and P2 scripts (Appendix C, 12 scripts: 6 setup + 6 send) present at `docs/tests/scripts/`.

**0.6 — Measure baseline RTT.**

Single-hop federation RTT between any two Nodes on the harness. Record. Confirm the P2 settle wait (60 s) is at least 30× the single-hop RTT to give convergence ample time.

### Definition of Done — Milestone 0

- [ ] Findings file created, status `ACTIVE`.
- [ ] Binary versions recorded.
- [ ] Spec cross-check complete; S3 findings cross-referenced.
- [ ] All 10 workspaces clean.
- [ ] All 24 scripts validated.
- [ ] RTT measured; settle wait confirmed sufficient.

---

## Milestone 1 — P1 Smoke

**Goal:** All 6 Clients across all 4 Nodes successfully register, join the Space, and exchange one message each. The resulting 6 messages appear on all 4 Nodes. Each Client sees all 6 messages (own + 5 others). Pairing matrix proves it.

### Sequence

**Step P1.1 — Start all 4 Nodes.**

```
xgen-node-app.exe --instance m4nA
xgen-node-app.exe --instance m4nB
xgen-node-app.exe --instance m4nC
xgen-node-app.exe --instance m4nD
```

Wait for all four to reach `READY`. Record timestamps.

**Step P1.2 — Federate full mesh.**

Establish 6 federation channels: A↔B, A↔C, A↔D, B↔C, B↔D, C↔D.

Verify each Node's federation registry contains the other 3 Nodes. Record verification in findings. If any channel is missing, abort and restart with clean workspace.

**Step P1.3 — Start Client A1 on Node A, register, create Space and Room.**

```
xgen-client-app.exe --instance m4a1
xgen-client-app.exe --instance m4a1 --batch docs/tests/scripts/multiparty_s4_smoke_clientA1_setup.xgb
```

Setup script (Appendix B.1): connect to Node A, `register alice1`, `create-space "Multiparty S4 P1"`, `create-room general`. Capture Space ID and Room ID.

**Step P1.4 — Wait for state propagation.**

Wait 15 s for `state.space_create` and `state.room_create` Events to propagate across all 4 Nodes. Confirm each Node shows the Space and Room.

**Step P1.5 — Start and bootstrap the remaining 5 Clients in sequence.**

For each of m4a2 (Node A), m4b1 (Node B), m4b2 (Node B), m4c1 (Node C), m4d1 (Node D):

```
xgen-client-app.exe --instance <label>
xgen-client-app.exe --instance <label> --batch docs/tests/scripts/multiparty_s4_smoke_<label>_setup.xgb
```

Each setup script (Appendix B.2 through B.6): connect to the appropriate Node, register, join the Space (Space ID from P1.3). Wait for exit 0.

After all 5 join, wait 15 s for membership to propagate.

**Step P1.6 — Confirm membership on all Nodes.**

Each of the 4 Nodes must show all 6 Identities as Space members. Record confirmation in findings.

**Step P1.7 — Each Client sends one message, in sequence.**

In this order, sequential (not concurrent — concurrency is P2's domain):

| Order | Client | Node | Message |
|---|---|---|---|
| 1 | m4a1 | A | `alice1-smoke-1` |
| 2 | m4a2 | A | `alice2-smoke-1` |
| 3 | m4b1 | B | `bob1-smoke-1` |
| 4 | m4b2 | B | `bob2-smoke-1` |
| 5 | m4c1 | C | `carol-smoke-1` |
| 6 | m4d1 | D | `dave-smoke-1` |

Wait 5 s between sends. After the last send, wait 30 s for settlement.

Send scripts (Appendix B.7 through B.12): each script contains one `send` command.

**Step P1.8 — Pairing matrix.**

For each of the 6 messages, verify presence in each of the 4 Node stores. Also verify each Client log shows all 6 messages (own as Outbound, 5 others as Inbound).

Pairing matrix (24 cells total for the 6 messages × 4 Nodes; plus a separate 36-cell matrix for 6 Clients × 6 messages):

**Node-presence matrix:**

| # | event_id (12-char) | EventType | Author | Authored Node | In A | In B | In C | In D |
|---|---|---|---|---|---|---|---|---|
| 1 | _aaa..._ | message.text (alice1-smoke-1) | m4a1 | A | ✔ | ✔ | ✔ | ✔ |
| 2 | _bbb..._ | message.text (alice2-smoke-1) | m4a2 | A | ✔ | ✔ | ✔ | ✔ |
| 3 | _ccc..._ | message.text (bob1-smoke-1) | m4b1 | B | ✔ | ✔ | ✔ | ✔ |
| 4 | _ddd..._ | message.text (bob2-smoke-1) | m4b2 | B | ✔ | ✔ | ✔ | ✔ |
| 5 | _eee..._ | message.text (carol-smoke-1) | m4c1 | C | ✔ | ✔ | ✔ | ✔ |
| 6 | _fff..._ | message.text (dave-smoke-1) | m4d1 | D | ✔ | ✔ | ✔ | ✔ |

All 24 cells = ✔ required.

**Client-presence matrix:**

| Message | m4a1 sees | m4a2 sees | m4b1 sees | m4b2 sees | m4c1 sees | m4d1 sees |
|---|---|---|---|---|---|---|
| alice1-smoke-1 | ✔ (Out) | ✔ (In) | ✔ (In) | ✔ (In) | ✔ (In) | ✔ (In) |
| alice2-smoke-1 | ✔ (In) | ✔ (Out) | ✔ (In) | ✔ (In) | ✔ (In) | ✔ (In) |
| bob1-smoke-1 | ✔ (In) | ✔ (In) | ✔ (Out) | ✔ (In) | ✔ (In) | ✔ (In) |
| bob2-smoke-1 | ✔ (In) | ✔ (In) | ✔ (In) | ✔ (Out) | ✔ (In) | ✔ (In) |
| carol-smoke-1 | ✔ (In) | ✔ (In) | ✔ (In) | ✔ (In) | ✔ (Out) | ✔ (In) |
| dave-smoke-1 | ✔ (In) | ✔ (In) | ✔ (In) | ✔ (In) | ✔ (In) | ✔ (Out) |

All 36 cells = ✔ required. "Out" = Outbound (own send), "In" = Inbound (received).

**Step P1.9 — Cross-Node convergence check.**

For each pair of Nodes, dump the full Event list for the Space and diff:

- A vs B, A vs C, A vs D, B vs C, B vs D, C vs D (6 pairwise diffs).
- All must be empty.

Record diff outputs.

**Step P1.10 — Content-leak check.**

```
findstr /S /M /R "alice1-smoke-1\|alice2-smoke-1\|bob1-smoke-1\|bob2-smoke-1\|carol-smoke-1\|dave-smoke-1" *.log
```

Zero unauthorised occurrences.

**Step P1.11 — Clean shutdown.**

Order: all 6 Clients first (any order), then all 4 Nodes.

### Definition of Done — Milestone 1

- [ ] 4 Nodes started cleanly.
- [ ] Full-mesh federation established and verified.
- [ ] All 6 Clients started, registered, joined the Space.
- [ ] All 4 Nodes show all 6 Identities as members after P1.5 + P1.6 wait.
- [ ] All 6 messages sent in sequence.
- [ ] Node-presence matrix: 24/24 ✔.
- [ ] Client-presence matrix: 36/36 ✔.
- [ ] All 6 cross-Node diffs empty.
- [ ] Content-leak check clean.
- [ ] Zero `ERROR` log lines across 10 log files (4 Nodes + 6 Clients).
- [ ] P1 verdict recorded: PASS or FAIL.
- [ ] If FAIL: stop, do not proceed to P2.

---

## Milestone 2 — P2 Stress

**Goal:** Sustained concurrent chat from all 6 Clients across all 4 Nodes. Each Client sends 50 messages, all 6 dispatched within a 2-second window. 300 messages total. Verify convergence on all 4 Nodes, no duplicates from full-mesh redundancy, no drops, identical state at end.

### Sequence

**Step P2.1 — Clean workspace, Node startup, full-mesh federation.**

Fresh data directories. Repeat P1.1–P1.2 with verification. Use a new Space (`Multiparty S4 P2`).

**Step P2.2 — Client bootstrap.**

Sequential: m4a1 creates Space and Room. Capture IDs. Then m4a2, m4b1, m4b2, m4c1, m4d1 each connect, register, and join in sequence. Wait 30 s for state propagation across all 4 Nodes. Confirm all 6 members visible on all 4 Nodes.

**Step P2.3 — Concurrent send dispatch.**

Each Client dispatches a single `.xgb` script containing 50 `send` lines. All 6 dispatches MUST occur within a 2 s wall-clock window.

Message texts:

- m4a1: `alice1-stress-01` through `alice1-stress-50`
- m4a2: `alice2-stress-01` through `alice2-stress-50`
- m4b1: `bob1-stress-01` through `bob1-stress-50`
- m4b2: `bob2-stress-01` through `bob2-stress-50`
- m4c1: `carol-stress-01` through `carol-stress-50`
- m4d1: `dave-stress-01` through `dave-stress-50`

Record dispatch timestamps for all 6 Clients in findings. The window must be ≤ 2 s.

**Step P2.4 — Drain.**

Wait 60 s for all 300 Events to settle across all 4 Nodes. The Node logs should stop growing after this window.

**Step P2.5 — Convergence verification.**

For each of the 4 Nodes, dump the full sorted Event list for the Space and Room. Pairwise diff all 6 Node pairs (A-B, A-C, A-D, B-C, B-D, C-D). All diffs must be empty.

Record diffs verbatim.

**Step P2.6 — Duplicate scan.**

Per-Node: group Events by `event_id`, verify no duplicates. Critical because full-mesh creates 6 federation paths between any two Nodes — an Event from one Node could be heard by another Node via multiple paths if deduplication is broken.

**Step P2.7 — Per-Client log verification.**

For each Client, verify its log shows:

- 50 Outbound `message.text` events (its own sends).
- 250 Inbound `message.text` events (the other 5 Clients × 50 each).
- Total: 300 `message.text` events per Client log.

**Step P2.8 — Clean shutdown.**

All 6 Clients, then all 4 Nodes.

### Metrics to capture

**Per-Node event counts:**

| Metric | Expected | A | B | C | D |
|---|---|---|---|---|---|
| `message.text` total in store | 300 | _ | _ | _ | _ |
| Duplicate `event_id`s | 0 | _ | _ | _ | _ |
| Orphans at end | 0 | _ | _ | _ | _ |

**Per-Client event counts:**

| Client | Outbound expected | Outbound observed | Inbound expected | Inbound observed | Total expected | Total observed |
|---|---|---|---|---|---|---|
| m4a1 | 50 | _ | 250 | _ | 300 | _ |
| m4a2 | 50 | _ | 250 | _ | 300 | _ |
| m4b1 | 50 | _ | 250 | _ | 300 | _ |
| m4b2 | 50 | _ | 250 | _ | 300 | _ |
| m4c1 | 50 | _ | 250 | _ | 300 | _ |
| m4d1 | 50 | _ | 250 | _ | 300 | _ |

**Cross-Node convergence:**

| Pair | Diff result |
|---|---|
| A ↔ B | _ |
| A ↔ C | _ |
| A ↔ D | _ |
| B ↔ C | _ |
| B ↔ D | _ |
| C ↔ D | _ |

All must be empty.

**Concurrent dispatch verification:**

| Client | Dispatch timestamp (ms) |
|---|---|
| m4a1 | _ |
| m4a2 | _ |
| m4b1 | _ |
| m4b2 | _ |
| m4c1 | _ |
| m4d1 | _ |
| **Max - min (window)** | **≤ 2000** |

**Latency (informational):**

| Metric | Same-Node (local fan-out) | Different-Node (federation) |
|---|---|---|
| Median Outbound → Inbound | _ | _ |
| p95 | _ | _ |
| Max | _ | _ |

Comparison is informative: same-Node fan-out (e.g. m4a1 → m4a2 on Node A) should be measurably faster than cross-Node delivery (e.g. m4a1 → m4d1 via federation).

**Log hygiene:**

| Metric | Expected | Observed |
|---|---|---|
| `ERROR` across all 10 logs | 0 | _ |
| `WARN` (non-shutdown) | 0 expected; record any | _ |
| Pending buffer events at shutdown (F-001 regression) | 0 | _ |
| Federation re-handshake events during test | 0 | _ |

### Definition of Done — Milestone 2

- [ ] Full-mesh federation established and verified.
- [ ] All 6 Clients bootstrapped, all visible as members on all 4 Nodes.
- [ ] Dispatch window ≤ 2 s; timestamps recorded.
- [ ] Each of 4 Nodes contains 300 `message.text` events; zero duplicates; zero orphans.
- [ ] Each of 6 Client logs contains 300 `message.text` events (50 Outbound + 250 Inbound).
- [ ] All 6 pairwise Node diffs empty.
- [ ] Zero `ERROR` log lines.
- [ ] `WARN` lines (if any) classified.
- [ ] Latency metrics recorded.
- [ ] P2 verdict recorded: PASS or FAIL.

---

## Definition of Done — Test S4 as a whole

- [ ] Milestone 0 (Preparation) all items ticked.
- [ ] Milestone 1 (P1 Smoke) all items ticked, verdict PASS.
- [ ] Milestone 2 (P2 Stress) all items ticked, verdict PASS.
- [ ] Findings file status set to `COMPLETED` with overall verdict.
- [ ] JOURNAL.md entry written summarising the S4 run.
- [ ] This instruction file's header status updated from `ACTIVE` to `COMPLETED`.
- [ ] Any bugs that surfaced linked from findings file.

After all items above are ticked, sequence advances to file 5/5 (`MULTIPARTY_S5_client_rebind.md`).

---

## Appendix A — Findings file template

When M0.1 creates `docs/tests/MULTIPARTY_S4_findings.md`, use this template:

```markdown
# Multiparty Test S4 — Findings
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
- Single-hop RTT measured: _N ms_
- Spec §3.9.2 quote: _..._
- S3 findings cross-referenced: yes/no; any pending FIXES applied? yes/no

---

## Milestone 1 — P1 Smoke

### Federation verification (full mesh)

| Node | Federated peers listed in registry |
|---|---|
| A | _ |
| B | _ |
| C | _ |
| D | _ |

All four show the other three: yes/no

### Node-presence matrix

_(insert 6 × 4 matrix per instruction file format)_

### Client-presence matrix

_(insert 6 × 6 matrix)_

### Cross-Node diffs

| Pair | Result |
|---|---|
| A ↔ B | empty / non-empty (paste) |
| A ↔ C | _ |
| A ↔ D | _ |
| B ↔ C | _ |
| B ↔ D | _ |
| C ↔ D | _ |

### Content-leak findstr

```
_(paste verbatim)_
```

### Verdict: _PASS/FAIL_

_Notes:_

---

## Milestone 2 — P2 Stress

### Dispatch window

| Client | Dispatch timestamp |
|---|---|
| m4a1 | _ |
| m4a2 | _ |
| m4b1 | _ |
| m4b2 | _ |
| m4c1 | _ |
| m4d1 | _ |
| Window (max − min) | _ ms |

### Per-Node and per-Client event counts

_(insert tables)_

### Cross-Node diffs

_(insert)_

### Latency metrics

_(insert)_

### Verdict: _PASS/FAIL_

_Notes:_

---

## Findings — bugs and anomalies

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

### B.1 — `docs/tests/scripts/multiparty_s4_smoke_m4a1_setup.xgb`

```
# Multiparty S4 P1 — Client m4a1 (alice1) setup
# Creates the Space and Room

connect ws://127.0.0.1:8080/xgen
register --name alice1 --passphrase m4a1-pass-1234
create-space --name "Multiparty S4 P1"
create-room --space @last_space --name general
status
```

### B.2 — `docs/tests/scripts/multiparty_s4_smoke_m4a2_setup.xgb`

```
# Multiparty S4 P1 — Client m4a2 (alice2) setup
# Same Node as m4a1; joins existing Space

connect ws://127.0.0.1:8080/xgen
register --name alice2 --passphrase m4a2-pass-1234
join --space <SPACE_ID_FROM_P1.3>
status
```

### B.3 — `docs/tests/scripts/multiparty_s4_smoke_m4b1_setup.xgb`

```
# Multiparty S4 P1 — Client m4b1 (bob1) setup

connect ws://127.0.0.1:8081/xgen
register --name bob1 --passphrase m4b1-pass-1234
join --space <SPACE_ID_FROM_P1.3>
status
```

### B.4 — `docs/tests/scripts/multiparty_s4_smoke_m4b2_setup.xgb`

```
# Multiparty S4 P1 — Client m4b2 (bob2) setup
# Same Node as m4b1

connect ws://127.0.0.1:8081/xgen
register --name bob2 --passphrase m4b2-pass-1234
join --space <SPACE_ID_FROM_P1.3>
status
```

### B.5 — `docs/tests/scripts/multiparty_s4_smoke_m4c1_setup.xgb`

```
# Multiparty S4 P1 — Client m4c1 (carol) setup

connect ws://127.0.0.1:8082/xgen
register --name carol --passphrase m4c1-pass-1234
join --space <SPACE_ID_FROM_P1.3>
status
```

### B.6 — `docs/tests/scripts/multiparty_s4_smoke_m4d1_setup.xgb`

```
# Multiparty S4 P1 — Client m4d1 (dave) setup

connect ws://127.0.0.1:8083/xgen
register --name dave --passphrase m4d1-pass-1234
join --space <SPACE_ID_FROM_P1.3>
status
```

### B.7 — `multiparty_s4_smoke_m4a1_send.xgb`

```
send --space <SPACE_ID> --room <ROOM_ID> --text "alice1-smoke-1"
```

### B.8 — `multiparty_s4_smoke_m4a2_send.xgb`

```
send --space <SPACE_ID> --room <ROOM_ID> --text "alice2-smoke-1"
```

### B.9 — `multiparty_s4_smoke_m4b1_send.xgb`

```
send --space <SPACE_ID> --room <ROOM_ID> --text "bob1-smoke-1"
```

### B.10 — `multiparty_s4_smoke_m4b2_send.xgb`

```
send --space <SPACE_ID> --room <ROOM_ID> --text "bob2-smoke-1"
```

### B.11 — `multiparty_s4_smoke_m4c1_send.xgb`

```
send --space <SPACE_ID> --room <ROOM_ID> --text "carol-smoke-1"
```

### B.12 — `multiparty_s4_smoke_m4d1_send.xgb`

```
send --space <SPACE_ID> --room <ROOM_ID> --text "dave-smoke-1"
```

---

## Appendix C — P2 Stress `.xgb` scripts

### C.1 through C.6 — Setup scripts

Same shape as B.1 through B.6 but with Space name `"Multiparty S4 P2"` instead of `"Multiparty S4 P1"`. m4a1's script creates the Space; the other 5 join.

Filename pattern: `multiparty_s4_stress_<instance>_setup.xgb`.

### C.7 through C.12 — Send scripts (50 messages per Client)

Each script contains 50 `send` lines. First three and last three of m4a1's script shown; lines 4–47 follow the same pattern.

`multiparty_s4_stress_m4a1_send.xgb`:

```
# Multiparty S4 P2 — Client m4a1 50 messages

send --space <SPACE_ID> --room <ROOM_ID> --text "alice1-stress-01"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice1-stress-02"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice1-stress-03"
# ... lines 4 through 47 ...
send --space <SPACE_ID> --room <ROOM_ID> --text "alice1-stress-48"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice1-stress-49"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice1-stress-50"
```

The other 5 stress send scripts follow the same shape with author-prefixed text:

- `multiparty_s4_stress_m4a2_send.xgb`: 50 lines, texts `alice2-stress-01` through `alice2-stress-50`.
- `multiparty_s4_stress_m4b1_send.xgb`: `bob1-stress-01` through `bob1-stress-50`.
- `multiparty_s4_stress_m4b2_send.xgb`: `bob2-stress-01` through `bob2-stress-50`.
- `multiparty_s4_stress_m4c1_send.xgb`: `carol-stress-01` through `carol-stress-50`.
- `multiparty_s4_stress_m4d1_send.xgb`: `dave-stress-01` through `dave-stress-50`.

All 6 send scripts may be generated mechanically.

---

## Sequence Position

| | |
|---|---|
| **This file** | 4 of 5 |
| **Previous** | `MULTIPARTY_S3_federation_topology.md` |
| **Next** | `MULTIPARTY_S5_client_rebind.md` |

Do not advance to the next file until this one's Definition of Done is fully ticked.

---

*End of MULTIPARTY_S4_n_clients_n_nodes.md*
