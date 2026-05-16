# Multiparty Test S3 — 3+ Node Federation, Transitive
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

This is file **3 of 5** in the **Multiparty** test operation.

**Full sequence (locked execution order):**

1. `MULTIPARTY_S1_multiclient_one_node.md` — multiple clients per Node
2. `MULTIPARTY_S2_concurrent_send.md` — DAG under concurrent writes
3. **`MULTIPARTY_S3_federation_topology.md`** — this file — 3+ Node federation, transitive
4. `MULTIPARTY_S4_n_clients_n_nodes.md` — N clients across N Nodes
5. `MULTIPARTY_S5_client_rebind.md` — one client across multiple Nodes

Each file in the suite must be COMPLETED before the next begins.

---

## Purpose

Verify that Events propagate correctly across **multi-Node federation topologies** where not every Node is directly connected to every other Node. The Phase 1 smoke test and S1/S2 covered two-Node federation only. This test exercises **transitive propagation**: when Nodes A↔B and B↔C are federated but A↔C is not, an Event authored on Node A must still reach Node C via Node B.

**What this test proves:**

- A Node that receives an Event from one federated peer forwards it to all its other federated peers that share the Space, regardless of which peer originally produced it.
- Transitive propagation completes in bounded time — there is no relay loop, no dropped relay, no event_id rewriting at intermediate Nodes.
- The convergence guarantee in §3.9.2 holds across 3-Node and full-mesh topologies — every Node ends with byte-identical state.
- Federation does not deduplicate falsely (Node B doesn't refuse to forward to A an Event it received from A).
- A full-mesh topology (every Node connected to every other) behaves identically to a chain topology — same final state, no duplicate-event storage.

**What this test does NOT prove:**

- Behaviour with N clients across N Nodes — covered by S4.
- Identity portability across Nodes — covered by S5.
- Network partition recovery — out of scope for the Multiparty suite (separate Phase 3 work).
- Performance scaling beyond 3 Nodes — out of scope; the count is fixed at 3 for this test.
- Long-running federation stability beyond the test duration.

---

## Prerequisites

This test depends on the following being COMPLETED:

- `MULTIPARTY_S1_multiclient_one_node.md` — single-Node fan-out verified.
- `MULTIPARTY_S2_concurrent_send.md` — 2-Node concurrent DAG behaviour verified.
- Phase 1 smoke test, Phase 2 integration test, `STRESSTEST_ph1_findings.md` — all COMPLETED.
- `BATCH_FLAG_ph2.md` — `--batch` available.

**Required binaries:**

- `xgen-node-app.exe`
- `xgen-client-app.exe`

**Required spec sections (read before execution):**

- Ch3 §3.2.2 — Event propagation, forward-compatibility rule for unknown EventTypes.
- Ch3 §3.2.6 — Event Validation Pipeline (step 9: pending buffer for unknown predecessors — central to transitive propagation).
- Ch3 §3.4 — Federation Handshake (bilateral; one channel per Node pair covers all shared Spaces).
- Ch3 §3.5.5 — Announcement Propagation (relay rule for Node announcements).
- Ch3 §3.7 — Space and Room Protocol.
- Ch3 §3.9 — State Resolution Algorithm (convergence guarantee §3.9.2).

---

## Spec gap to record in findings (informational, not blocking)

The current spec does **not** include a canonical sentence such as:

> "A Node MUST forward each accepted Event to all federated peers that share the Space, including peers from which it did not originate."

The transitive propagation behaviour is **implied** by the convergence guarantee in §3.9.2 — every Node holding the same Event set computes the same state — but the explicit "forward on accept" rule is not written as a single normative MUST. This is not a blocker for S3 because the implementation has to do transitive propagation for §3.9.2 to hold; the test will reveal whether the implementation does.

**Action for M0:** record this gap in the findings file. If S3 PASSES, propose a one-sentence spec addition to §3.2 (likely §3.2.2 or a new §3.2.6.1) for a future spec pass. If S3 FAILS due to missing transitive propagation in the implementation, the spec gap and the bug get filed together — fix the implementation, write the spec sentence.

---

## Scope

### In scope

- 3 Nodes total: A, B, C.
- Two topologies tested in sequence:
  - **P1 — chain topology:** A↔B and B↔C federated; A↔C NOT federated. Transitive case.
  - **P2 — full mesh:** all three Node pairs federated. Should behave identically to chain in final state.
- 1 Client per Node (3 Clients total).
- 1 Space, 1 Room, all three Clients members.
- Both topologies: each Client sends 1 message in P1, 20 messages each in P2 (60 total).
- Verification: all three Nodes converge to identical Event sets and identical state.

### Out of scope

- More than 3 Nodes.
- Multiple clients per Node (S1 covers this; combined with topology in S4).
- Concurrent writes from 3+ Clients (S4 territory).
- Identity portability across Nodes (S5).
- Network partition / heal scenarios.
- Federation latency measurement under load.

---

## Architecture Constraints — Non-Negotiable

**Use only existing infrastructure.** No new CLI commands, no new event types. Test runs against current binaries.

**No shell invocation.** `--batch` and named pipes only.

**Distinct instance labels.** Nodes: `m3nA`, `m3nB`, `m3nC`. Clients: `m3a`, `m3b`, `m3c`. Distinct named pipes, data directories, keypairs.

**Topology is enforced, not assumed.** The chain topology (P1) requires that A and C have NO federation channel between them. The test harness MUST verify this explicitly — by inspecting each Node's federation registry — before sending the test message. If A and C are inadvertently federated (e.g. via a stale registry entry), the test is invalid and must be restarted with a clean workspace.

**No timeout cheating.** The pending-buffer timeout in §3.9.6 is 30 seconds. For transitive propagation tests, the harness MUST wait at least 60 seconds after the final send before declaring "Event did not arrive at Node C". Premature failure declarations are bugs in the test, not bugs in the protocol.

**Stop on first failure.** P1 failure halts the test; do not run P2.

**Honesty.** Per CLAUDE.md Rules 1–7. No fabricated event counts, no claimed convergence without byte-level verification.

**Findings file is the write surface.** All runtime data goes to `MULTIPARTY_S3_findings.md`.

---

## Topology

**P1 — Chain topology (transitive case):**

```
   ┌──────────────────────┐         ┌──────────────────────┐         ┌──────────────────────┐
   │     xgen-node-app    │◀═══════▶│     xgen-node-app    │◀═══════▶│     xgen-node-app    │
   │    --instance m3nA   │         │    --instance m3nB   │         │    --instance m3nC   │
   │ws://127.0.0.1:8080/  │   fed.  │ws://127.0.0.1:8081/  │   fed.  │ws://127.0.0.1:8082/  │
   └──────────┬───────────┘         └──────────┬───────────┘         └──────────┬───────────┘
              │                                │                                │
              │       ╳  (A↔C NOT federated — this is the transitive case)      │
              │                                                                 │
              │                                                                 │
          ┌───┴───┐                        ┌───┴───┐                        ┌───┴───┐
          │client │                        │client │                        │client │
          │  m3a  │                        │  m3b  │                        │  m3c  │
          │alice  │                        │bob    │                        │carol  │
          └───────┘                        └───────┘                        └───────┘
```

A↔B and B↔C are federated. A↔C is NOT. Node B is the only path between A and C for Events.

**P2 — Full mesh topology:**

All three Node pairs federated (A↔B, B↔C, A↔C). Same clients as P1, fresh data, new Space.

---

## Test Data and Identifiers

| Item | Value |
|---|---|
| Node A | `m3nA` at `ws://127.0.0.1:8080/xgen` |
| Node B | `m3nB` at `ws://127.0.0.1:8081/xgen` |
| Node C | `m3nC` at `ws://127.0.0.1:8082/xgen` |
| Client A | `m3a` (alice, passphrase `m3a-pass-1234`) — connects to Node A |
| Client B | `m3b` (bob, passphrase `m3b-pass-1234`) — connects to Node B |
| Client C | `m3c` (carol, passphrase `m3c-pass-1234`) — connects to Node C |
| Space name (P1) | `Multiparty S3 P1 chain` |
| Space name (P2) | `Multiparty S3 P2 mesh` |
| Room name (both) | `general` |
| P1 transitive-propagation wait | 60 s (≥ 2× §3.9.6 timeout) |
| P2 messages per client | 20 |
| P2 total messages | 60 |

---

## Milestone 0 — Preparation

**0.1 — Create findings file.**

Create `docs/tests/MULTIPARTY_S3_findings.md` from Appendix A. Status `ACTIVE`.

**0.2 — Record binary versions.**

```
xgen-node-app.exe --version
xgen-client-app.exe --version
```

**0.3 — Record spec cross-check.**

Open `docs/xgen_ch3_specification.md`. Confirm the following sections are present and unchanged from this file's prerequisites list:

- §3.2.2 — propagation rule for unknown EventTypes
- §3.2.6 — validation pipeline, step 9 (pending buffer)
- §3.4 — federation handshake
- §3.9.2 — convergence guarantee
- §3.9.6 — pending event timeout (30 s)

Quote §3.9.2 verbatim into findings. Also record the **spec gap** noted above: there is no explicit "forward on accept" canonical sentence; transitive propagation is implied by §3.9.2 convergence. This gap is informational, not blocking.

**0.4 — Clean workspace.**

All four `m3*` data directories cleared or archived. Record paths.

**0.5 — Validate scripts.**

Confirm all P1 scripts (Appendix B) and P2 scripts (Appendix C) present at `docs/tests/scripts/`.

**0.6 — Measure baseline federation RTT.**

Same as S2 M0.6: measure the time for an Event to travel A→B (single hop) on the harness machine. Record. Then estimate **two-hop RTT** (A→B→C) as approximately 2× the single-hop RTT. The P1 transitive-propagation wait (60 s) must be much larger than the two-hop RTT.

### Definition of Done — Milestone 0

- [ ] Findings file created, status `ACTIVE`.
- [ ] Binary versions recorded.
- [ ] Spec cross-check complete; §3.9.2 quoted; spec gap on transitive propagation recorded.
- [ ] Workspace clean.
- [ ] All scripts validated.
- [ ] Single-hop RTT measured; two-hop wait confirmed sufficient.

---

## Milestone 1 — P1 Smoke (Chain Topology)

**Goal:** Establish a chain topology (A↔B↔C, A↔C not federated). Three Clients (one per Node) join the same Space. Each Client sends one message. Verify all three messages arrive on all three Nodes — proving transitive propagation works.

### Sequence

**Step P1.1 — Start all three Nodes.**

```
xgen-node-app.exe --instance m3nA
xgen-node-app.exe --instance m3nB
xgen-node-app.exe --instance m3nC
```

Wait for all three to reach `READY`. Record timestamps.

**Step P1.2 — Federate A↔B and B↔C ONLY.**

Initiate federation A↔B from one side (per the standard federation command — same as in Phase 1 smoke test).
Then federate B↔C from one side.
**Do NOT federate A↔C.**

Verify in each Node's federation registry:

- Node A's federation registry contains: B only.
- Node B's federation registry contains: A and C.
- Node C's federation registry contains: B only.

Record verification output in findings. **If A↔C is inadvertently federated, abort and restart with clean workspace.**

**Step P1.3 — Start Client A, register, create Space.**

```
xgen-client-app.exe --instance m3a
xgen-client-app.exe --instance m3a --batch docs/tests/scripts/multiparty_s3_smoke_clientA_setup.xgb
```

Setup script (Appendix B.1): connect to Node A, `register alice`, `create-space "Multiparty S3 P1 chain"`, `create-room general`. Capture Space ID and Room ID. Wait for exit 0.

**Step P1.4 — Wait for Space state to propagate to Nodes B and C.**

After step P1.3 completes, wait 10 seconds for the `state.space_create` and `state.room_create` Events to propagate A→B and then B→C transitively. Confirm both Nodes B and C show the Space in their stores by reading their state files or running `xgen-node-app.exe --instance m3nB --status` (or equivalent).

If the Space is not visible on Node C after 60 seconds, the test fails at this step. Record actual propagation latency to Node C (`state.space_create` Event timestamp on Node A vs. Inbound time on Node C).

**Step P1.5 — Start Client B, register on Node B, join Space.**

```
xgen-client-app.exe --instance m3b
xgen-client-app.exe --instance m3b --batch docs/tests/scripts/multiparty_s3_smoke_clientB_setup.xgb
```

Setup script (Appendix B.2): connect to Node B, `register bob`, `join <Space ID>`. Wait for exit 0.

**Step P1.6 — Start Client C, register on Node C, join Space.**

```
xgen-client-app.exe --instance m3c
xgen-client-app.exe --instance m3c --batch docs/tests/scripts/multiparty_s3_smoke_clientC_setup.xgb
```

Setup script (Appendix B.3): connect to Node C, `register carol`, `join <Space ID>`. Wait for exit 0.

**Step P1.7 — Wait for membership to propagate.**

Wait 30 seconds. Confirm all three Nodes show all three Clients as Space members (via state file inspection or status command).

**Step P1.8 — Each Client sends one message.**

Run sequentially (not concurrently — this is the smoke phase, isolating transitive propagation, not concurrent ordering):

```
xgen-client-app.exe --instance m3a --batch docs/tests/scripts/multiparty_s3_smoke_clientA_send.xgb
```

m3a sends `alice-chain-1`. Wait 30 seconds.

```
xgen-client-app.exe --instance m3b --batch docs/tests/scripts/multiparty_s3_smoke_clientB_send.xgb
```

m3b sends `bob-chain-1`. Wait 30 seconds.

```
xgen-client-app.exe --instance m3c --batch docs/tests/scripts/multiparty_s3_smoke_clientC_send.xgb
```

m3c sends `carol-chain-1`. Wait 60 seconds.

**Step P1.9 — Verify all three messages on all three Nodes.**

For each of the three messages and each of the three Nodes, verify the message Event is present in the Node's Event store. Build the pairing table (below).

The transitive cases (where propagation must hop through B) are:

- `alice-chain-1` reaching Node C: Authored on A → forwarded to B (direct) → forwarded to C (transitive via B).
- `carol-chain-1` reaching Node A: Authored on C → forwarded to B (direct) → forwarded to A (transitive via B).

The direct cases (single hop) are:

- `alice-chain-1` reaching Node B (direct).
- `bob-chain-1` reaching Nodes A and C (direct from B, single hop each).
- `carol-chain-1` reaching Node B (direct).

**Step P1.10 — Cross-Node state diff.**

For each Node, dump the full sorted Event list for the test Space (using `event_id` as the sort key, or the spec's resolved order per §3.9). Diff the three dumps:

- A vs B → expected empty
- B vs C → expected empty
- A vs C → expected empty (the critical transitive check)

Record diff outputs verbatim in findings.

**Step P1.11 — Clean shutdown.**

Clients C, B, A; then Nodes C, B, A.

### Expected pairing table — P1

| event_id (12-char prefix) | EventType | Authored | Hop count to A | Hop count to B | Hop count to C | In A store | In B store | In C store |
|---|---|---|---|---|---|---|---|---|
| _setup events_ | various | m3a | 0 | 1 | 2 (transitive) | ✔ | ✔ | ✔ |
| _aaa..._ | message.text (alice-chain-1) | m3a | 0 (own) | 1 (direct) | 2 (transitive) | ✔ | ✔ | ✔ |
| _bbb..._ | message.text (bob-chain-1) | m3b | 1 (direct) | 0 (own) | 1 (direct) | ✔ | ✔ | ✔ |
| _ccc..._ | message.text (carol-chain-1) | m3c | 2 (transitive) | 1 (direct) | 0 (own) | ✔ | ✔ | ✔ |

All ✔ required for PASS.

### Content-leak check

```
findstr /S /M /R "alice-chain-1\|bob-chain-1\|carol-chain-1" *.log
```

Across all Node and Client logs. Zero unauthorised occurrences.

### Definition of Done — Milestone 1

- [ ] Three Nodes started cleanly.
- [ ] Chain topology established and verified: A↔B and B↔C only; A↔C NOT federated.
- [ ] Federation registry contents recorded in findings for all three Nodes.
- [ ] Three Clients started, registered on respective Nodes, joined the Space.
- [ ] Space and Room visible on all three Nodes after Step P1.4 / P1.7 waits.
- [ ] All three messages sent (one per Client).
- [ ] Pairing table fully ✔ — all three messages present in all three Node stores.
- [ ] All three pairwise Event-set diffs (A↔B, B↔C, A↔C) are empty.
- [ ] Content-leak check clean.
- [ ] No `ERROR` or unexpected `WARN` log lines.
- [ ] P1 verdict recorded: PASS or FAIL.
- [ ] If FAIL: stop, do not proceed to P2.

---

## Milestone 2 — P2 Stress (Full Mesh)

**Goal:** Re-run the same scenario as P1 but with all three Node pairs federated (full mesh). Larger volume — 20 messages per Client, 60 total. Verify final state convergence is identical across all three Nodes, despite the additional propagation paths.

P2 verifies that **full mesh does not break convergence** — each Event is delivered exactly once to each Node's store regardless of how many paths it could have taken.

### Sequence

**Step P2.1 — Clean workspace and Node startup.**

Fresh data directories for all `m3*` instances. Start all three Nodes. Wait for `READY`.

**Step P2.2 — Federate full mesh.**

Federate A↔B, B↔C, and A↔C. Verify each Node's federation registry contains the other two.

**Step P2.3 — Client bootstrap.**

Run sequentially: Client A creates Space `Multiparty S3 P2 mesh` and Room. Capture IDs. Clients B and C join. Wait 30 seconds for propagation. Confirm all three Nodes show all three Clients as members.

**Step P2.4 — Send phase.**

Each Client sends 20 messages back-to-back via a single `.xgb` script. Dispatch may overlap in time, but is NOT a concurrency test — this is volume-and-convergence. The three send scripts may be dispatched approximately in parallel; record actual dispatch order if it matters.

Message texts:

- m3a: `alice-mesh-01` through `alice-mesh-20`
- m3b: `bob-mesh-01` through `bob-mesh-20`
- m3c: `carol-mesh-01` through `carol-mesh-20`

Total: 60 messages.

**Step P2.5 — Drain.**

Wait 60 seconds for all Events to settle on all three Nodes.

**Step P2.6 — Convergence verification.**

For each Node, dump the full sorted Event list for the Space and Room. Compare pairwise:

- A's dump vs B's dump → must be byte-identical
- B's dump vs C's dump → must be byte-identical
- A's dump vs C's dump → must be byte-identical

Record all three diffs.

**Step P2.7 — Duplicate check.**

In each Node's store, group Events by `event_id` and verify no `event_id` appears more than once. This catches the failure mode where full-mesh causes a Node to receive the same Event via multiple paths and store duplicates.

**Step P2.8 — Shutdown.**

Clean shutdown: Clients C, B, A; Nodes C, B, A.

### Metrics to capture

**Event counts per Node:**

| Metric | Expected | Observed on A | Observed on B | Observed on C |
|---|---|---|---|---|
| `message.text` authored by m3a | 20 | _ | _ | _ |
| `message.text` authored by m3b | 20 | _ | _ | _ |
| `message.text` authored by m3c | 20 | _ | _ | _ |
| **Total `message.text` in store** | **60** | _ | _ | _ |
| Duplicate `event_id`s | 0 | _ | _ | _ |
| Orphans at end | 0 | _ | _ | _ |

**Cross-Node convergence:**

| Pair | Diff result | PASS criterion |
|---|---|---|
| A ↔ B | _ | empty |
| B ↔ C | _ | empty |
| A ↔ C | _ | empty (key check — was the transitive path also redundant via mesh?) |

**Latency (informational):**

| Metric | Direct (1-hop) | Transitive (2-hop) chain | Full mesh (1-hop available) |
|---|---|---|---|
| Median delivery time | _ | _ | _ |
| p95 delivery time | _ | _ | _ |

Comparing chain (P1) vs mesh (P2) latency for the same A→C path is informative — mesh should be faster because there's a direct edge.

**Log hygiene:**

| Metric | Expected | Observed |
|---|---|---|
| `ERROR` lines across all Node logs | 0 | _ |
| `ERROR` lines across all Client logs | 0 | _ |
| Pending buffer entries at shutdown | 0 | _ |
| Spurious federation re-handshake events | 0 | _ |

### Definition of Done — Milestone 2

- [ ] Full mesh topology established and verified.
- [ ] 60 `message.text` Events authored (20 per Client).
- [ ] All three Node stores contain all 60 Events.
- [ ] Zero duplicate `event_id`s on any Node.
- [ ] All three pairwise diffs empty.
- [ ] Zero `ERROR` log lines.
- [ ] Latency metrics recorded.
- [ ] P2 verdict recorded: PASS or FAIL.

---

## Definition of Done — Test S3 as a whole

- [ ] Milestone 0 (Preparation) all items ticked, including spec-gap note recorded.
- [ ] Milestone 1 (P1 Smoke — chain) all items ticked, verdict PASS.
- [ ] Milestone 2 (P2 Stress — mesh) all items ticked, verdict PASS.
- [ ] Findings file status set to `COMPLETED` with overall verdict.
- [ ] JOURNAL.md entry written.
- [ ] This instruction file's header status updated from `ACTIVE` to `COMPLETED`.
- [ ] Spec-gap recommendation (one-sentence addition to §3.2 for "forward on accept") raised to Joe in the JOURNAL entry. Do NOT silently amend the spec — propose only.

After all items above are ticked, sequence advances to file 4/5 (`MULTIPARTY_S4_n_clients_n_nodes.md`).

---

## Appendix A — Findings file template

When M0.1 creates `docs/tests/MULTIPARTY_S3_findings.md`, use this template:

```markdown
# Multiparty Test S3 — Findings
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

| Run | Date | Build / commit | P1 (chain) | P2 (mesh) | Notes |
|---|---|---|---|---|---|
| 1 | YYYY-MM-DD | _commit hash_ | _PASS/FAIL_ | _PASS/FAIL_ | _short note_ |

---

## Milestone 0 — Preparation record

- Binary versions: _paste version strings_
- Workspace state: _clean / archived to ..._
- Scripts verified: _list_
- Single-hop RTT measured: _N ms_
- Spec §3.9.2 (convergence) verbatim quote:

  > _quote here_

- **Spec gap recorded:** no explicit "forward on accept" canonical sentence in §3.2. Transitive propagation is implied by §3.9.2 convergence. _(See test description for full text.)_

---

## Milestone 1 — P1 Smoke (chain)

### Federation registry verification

- Node A federation registry: _list of federated peers_
- Node B federation registry: _list_
- Node C federation registry: _list_
- A↔C absence confirmed: yes/no

### Pairing table

_(insert table per instruction file format)_

### Cross-Node Event diff

- A vs B: empty / non-empty (paste diff)
- B vs C: empty / non-empty
- A vs C: empty / non-empty (KEY CHECK)

### Content-leak findstr

```
_(paste verbatim)_
```

### Verdict: _PASS/FAIL_

_Notes:_

---

## Milestone 2 — P2 Stress (mesh)

### Federation registry verification

- Each Node lists the other two: yes/no

### Event counts

_(insert tables)_

### Cross-Node diffs

_(insert)_

### Duplicate check

- Duplicates found on any Node: yes/no

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

## Spec-gap recommendation

Proposed addition to §3.2 (location TBD — likely §3.2.2 or new §3.2.6.1):

> _draft sentence here, e.g._
> "A Node that accepts an Event MUST forward it to all federated peers that share the Space containing the Event, including peers from which the Event did not originate. Forwarding propagation is transitive: every accepted Event reaches every federated Node that shares the relevant Space, by direct or relay paths, subject to §3.9.6 pending event timeout."

Status: proposal only. Do not amend spec without Joe's approval.

---

## Overall verdict

_PASS / FAIL / BLOCKED_
```

---

## Appendix B — P1 Smoke `.xgb` scripts

### B.1 — `docs/tests/scripts/multiparty_s3_smoke_clientA_setup.xgb`

```
# Multiparty S3 P1 — Client A (alice) setup
# Connect to Node A, register, create Space and Room

connect ws://127.0.0.1:8080/xgen
register --name alice --passphrase m3a-pass-1234
create-space --name "Multiparty S3 P1 chain"
create-room --space @last_space --name general
status
```

### B.2 — `docs/tests/scripts/multiparty_s3_smoke_clientB_setup.xgb`

```
# Multiparty S3 P1 — Client B (bob) setup
# Connect to Node B, register, join Space (Space ID substituted from P1.3)

connect ws://127.0.0.1:8081/xgen
register --name bob --passphrase m3b-pass-1234
join --space <SPACE_ID_FROM_P1.3>
status
```

### B.3 — `docs/tests/scripts/multiparty_s3_smoke_clientC_setup.xgb`

```
# Multiparty S3 P1 — Client C (carol) setup
# Connect to Node C, register, join Space (Space ID substituted from P1.3)
# Note: Node C is NOT federated with Node A directly — relies on B for relay

connect ws://127.0.0.1:8082/xgen
register --name carol --passphrase m3c-pass-1234
join --space <SPACE_ID_FROM_P1.3>
status
```

### B.4 — `docs/tests/scripts/multiparty_s3_smoke_clientA_send.xgb`

```
# Multiparty S3 P1 — Client A single send

send --space <SPACE_ID> --room <ROOM_ID> --text "alice-chain-1"
```

### B.5 — `docs/tests/scripts/multiparty_s3_smoke_clientB_send.xgb`

```
# Multiparty S3 P1 — Client B single send

send --space <SPACE_ID> --room <ROOM_ID> --text "bob-chain-1"
```

### B.6 — `docs/tests/scripts/multiparty_s3_smoke_clientC_send.xgb`

```
# Multiparty S3 P1 — Client C single send
# This Event must reach Node A via Node B (transitive case)

send --space <SPACE_ID> --room <ROOM_ID> --text "carol-chain-1"
```

---

## Appendix C — P2 Stress `.xgb` scripts

### C.1 — `docs/tests/scripts/multiparty_s3_stress_clientA_setup.xgb`

```
# Multiparty S3 P2 — Client A setup (mesh)

connect ws://127.0.0.1:8080/xgen
register --name alice --passphrase m3a-pass-1234
create-space --name "Multiparty S3 P2 mesh"
create-room --space @last_space --name general
status
```

### C.2 — `docs/tests/scripts/multiparty_s3_stress_clientB_setup.xgb`

```
# Multiparty S3 P2 — Client B setup

connect ws://127.0.0.1:8081/xgen
register --name bob --passphrase m3b-pass-1234
join --space <SPACE_ID_FROM_P2.3>
status
```

### C.3 — `docs/tests/scripts/multiparty_s3_stress_clientC_setup.xgb`

```
# Multiparty S3 P2 — Client C setup

connect ws://127.0.0.1:8082/xgen
register --name carol --passphrase m3c-pass-1234
join --space <SPACE_ID_FROM_P2.3>
status
```

### C.4 — `docs/tests/scripts/multiparty_s3_stress_clientA_send.xgb`

20 `send` lines, first three and last three shown; lines 4–17 follow the same pattern.

```
# Multiparty S3 P2 — Client A 20 messages

send --space <SPACE_ID> --room <ROOM_ID> --text "alice-mesh-01"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-mesh-02"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-mesh-03"
# ... lines 4–17 ...
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-mesh-18"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-mesh-19"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-mesh-20"
```

### C.5 — `docs/tests/scripts/multiparty_s3_stress_clientB_send.xgb`

Same shape as C.4 but with `bob-mesh-NN` texts (01–20).

### C.6 — `docs/tests/scripts/multiparty_s3_stress_clientC_send.xgb`

Same shape as C.4 but with `carol-mesh-NN` texts (01–20).

---

## Sequence Position

| | |
|---|---|
| **This file** | 3 of 5 |
| **Previous** | `MULTIPARTY_S2_concurrent_send.md` |
| **Next** | `MULTIPARTY_S4_n_clients_n_nodes.md` |

Do not advance to the next file until this one's Definition of Done is fully ticked.

---

*End of MULTIPARTY_S3_federation_topology.md*
