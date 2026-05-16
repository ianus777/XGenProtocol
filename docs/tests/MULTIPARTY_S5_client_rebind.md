# Multiparty Test S5 — One Client Across Multiple Nodes (Identity Portability)
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

This is file **5 of 5** in the **Multiparty** test operation — the final scenario.

**Full sequence (locked execution order):**

1. `MULTIPARTY_S1_multiclient_one_node.md` — multiple clients per Node
2. `MULTIPARTY_S2_concurrent_send.md` — DAG under concurrent writes
3. `MULTIPARTY_S3_federation_topology.md` — 3+ Node federation, transitive
4. `MULTIPARTY_S4_n_clients_n_nodes.md` — N clients across N Nodes
5. **`MULTIPARTY_S5_client_rebind.md`** — this file — one client across multiple Nodes

This is the last file in the suite. After S5 COMPLETES, the entire Multiparty operation is closed.

---

## Purpose

Verify that an Identity is portable across home Nodes. The previous tests (S1–S4) all assumed each Client stays bound to its registering Node for the duration of the test. Real deployments need more: a Client must be able to **re-home** to a different Node when its original home Node goes offline permanently, while keeping the same `identity_id`, the same signature continuity, and the same membership in Spaces it has joined.

The spec mechanism is **orphaned Identity recovery (§3.13.8)** — a client whose original home Node is unavailable can re-register on a new Node using the existing keypair, broadcasting `identity.home_changed` to notify the network. Because `identity_id` is the pubkey URI of the keypair, re-homing produces the same ID; previously signed Events remain verifiable.

**What this test proves:**

- A Client whose home Node is shut down can re-register on a different Node using its existing keypair, producing the same `identity_id` (§3.13.8 key continuity).
- The new Node accepts the registration with `re_registration: true` and the keypair challenge-response succeeds.
- An `identity.home_changed` notification reaches federated peers; they update their replica records with the new home node URL.
- After re-homing, the Client can send and receive Events in Spaces it was previously a member of — without re-joining.
- Events signed by this Identity before re-homing remain verifiable on every Node (signature continuity).
- An Identity record retrieved from a replica Node during the orphan window is current and serviceable for signature verification.

**What this test does NOT prove:**

- Concurrent re-homing (two Clients re-homing at once) — not a meaningful scenario.
- Adversarial re-homing (a malicious party trying to re-home someone else's Identity) — out of scope; defended by the keypair challenge-response.
- Trust Assertion renewal at re-home — touched on briefly per §3.13.8 ("Trust Assertion continuity") but not deeply exercised; Tier 1 has empty assertions for this test.
- Multi-device Identity (Phase 2+ feature beyond Phase 1 single-device Identity).
- The full N=3 replication factor — the test uses a minimum-viable replica count to keep the topology comprehensible.

---

## Spec capability gate — verify before writing scripts

S5 depends on three protocol surfaces that may or may not be fully wired into the current CLI:

1. **`re_registration: true` flag in `identity.register`** — defined in §3.6 registration schema. Must be settable from `xgen-client-app.exe`. If not exposed via CLI, S5 is blocked pending a small CLI extension.
2. **`identity.replicate` push from home Node to replica(s)** — §3.13.4. Must occur automatically on registration. Verifiable by inspecting the replica Node's Identity registry after registration.
3. **`identity.home_changed` notification** — §3.13.8. Must be emitted by the Client (or the new home Node) after re-registration. Verifiable by inspecting peer Node logs.

**Action for M0.3:** Clair MUST verify these three capabilities exist in the current binaries before writing any `.xgb` scripts beyond setup. If any is missing, S5 splits into two paths:

- **Path A (capability present):** proceed with this test as specified.
- **Path B (capability missing):** record the gap in findings, mark S5 as `BLOCKED` (not FAIL), and queue a tasks file requesting Clair implement the missing CLI surface as a prerequisite. Do not invent workarounds. Do not fabricate.

This gate exists because identity portability is the highest-risk dependency in the Multiparty suite — most other tests use only Phase 1 surfaces, but S5 requires Phase 2 §3.13 to be implemented end-to-end including CLI exposure.

---

## Prerequisites

This test depends on the following being COMPLETED:

- `MULTIPARTY_S1` — local fan-out works.
- `MULTIPARTY_S2` — concurrent DAG works.
- `MULTIPARTY_S3` — transitive federation works.
- `MULTIPARTY_S4` — full mesh chat-room works.
- Phase 1 smoke test, Phase 2 integration test — both COMPLETED.
- `BATCH_FLAG_ph2.md` — `--batch` available.

**Required binaries:**

- `xgen-node-app.exe`
- `xgen-client-app.exe`

**Required spec sections (read before execution):**

- Ch3 §3.6 — Identity Registration Protocol (the `re_registration` flag, lines 1668).
- Ch3 §3.13 — Identity Replication Parameters (the entire section, especially §3.13.1, §3.13.4, §3.13.8).

Quote §3.13.1 (replication model and authority) and §3.13.8 (orphan recovery) into the findings file at M0.3.

---

## Scope

### In scope

- 3 Nodes: A (original home), B (replica, becomes new home), C (federation peer / replica).
- 1 Client (alice). 1 Space, 1 Room. Other ambient Identities for replica observation purposes only.
- P1 — single re-home cycle: register on A, send messages, shut down A, re-home to B, send more messages from B. Verify signature continuity and Space membership preservation.
- P2 — sustained chat across a re-home boundary: send 30 messages from A, re-home to B, send 30 more from B. 60 messages total. Verify all 60 events appear on all Nodes with consistent ordering.

### Out of scope

- More than 1 re-home per Client (the Identity changes home Node once; not a roam-back test).
- Re-homing without orphan condition (live home Node migration — Phase 3 work).
- Multiple Clients re-homing simultaneously.
- Multi-device Identity scenarios.
- Trust Assertion renewal at re-home (out of scope for Tier 1).
- Identity replication factor stress testing (separate test if needed).

---

## Architecture Constraints — Non-Negotiable

**Use only existing infrastructure.** No new event types. CLI extensions for `re_registration: true` exposure may be required (gate at M0.3); if so, S5 BLOCKS and a separate task file requests Clair add the surface.

**No shell invocation.** `--batch` and named pipes only.

**Distinct instance labels.** Nodes: `m5nA`, `m5nB`, `m5nC`. Client: `m5alice`. One Client only — the test is about one Identity moving, not many.

**Keypair preservation is mandatory.** The Client's keypair MUST be preserved across the re-home step. The whole point of re-homing is that the same Ed25519 keypair produces the same `identity_id`. If the test harness regenerates the keypair, the test is invalid.

The mechanism: the Client's keypair file (`xgen-client_keypair.enc` per the file placement rules) MUST persist between the "register on A" phase and the "re-register on B" phase. The Client's data directory is the boundary — keep the keypair file; the Identity state can be re-read or re-derived.

If `xgen-client-app.exe` does not currently support pointing to an existing keypair file when targeting a new Node, that's another capability gate finding for M0.3.

**No timeout cheating.** §3.9.6 pending event timeout is 30 s. §3.13.6 replica refresh interval is 7 days (irrelevant for this test's duration but worth noting in case of unexpected interactions). Allow ample settle time — minimum 60 s after re-home before declaring success or failure of post-re-home sends.

**Stop on first failure.** P1 failure halts the test; do not run P2. Capability gate failure at M0.3 BLOCKS the test (different from FAIL — it's a precondition not met, not a protocol bug).

**Honesty.** Per CLAUDE.md Rules 1–7. Do not invent `re_registration` mechanisms that don't exist. Do not fabricate `identity.home_changed` propagation. If the spec mechanism isn't there in the binaries, say so plainly.

**Findings file is the write surface.** All runtime data goes to `MULTIPARTY_S5_findings.md`.

---

## Topology

**Initial state (before re-home):**

```
   ┌──────────────────────┐         ┌──────────────────────┐         ┌──────────────────────┐
   │     xgen-node-app    │◀═══════▶│     xgen-node-app    │◀═══════▶│     xgen-node-app    │
   │    --instance m5nA   │   fed.  │    --instance m5nB   │   fed.  │    --instance m5nC   │
   │ws://127.0.0.1:8080/  │         │ws://127.0.0.1:8081/  │         │ws://127.0.0.1:8082/  │
   │  (alice's HOME)      │◀════════│  (alice's REPLICA)   │═══════▶ │  (alice's REPLICA)   │
   └──────────┬───────────┘         └──────────────────────┘         └──────────────────────┘
              │
              │  alice originally registered here
              │
          ┌───┴───────┐
          │  client   │
          │ m5alice   │
          │  alice    │
          └───────────┘
```

**After re-home (Node A is gone; alice re-homed to Node B):**

```
                                   ┌──────────────────────┐         ┌──────────────────────┐
                                   │     xgen-node-app    │◀═══════▶│     xgen-node-app    │
       (Node A shut down)          │    --instance m5nB   │   fed.  │    --instance m5nC   │
                                   │ws://127.0.0.1:8081/  │         │ws://127.0.0.1:8082/  │
                                   │  (alice's NEW HOME)  │◀═══════▶│  (alice's REPLICA)   │
                                   └──────────┬───────────┘         └──────────────────────┘
                                              │
                                              │  alice now connects here
                                              │  same keypair, same identity_id
                                              │
                                          ┌───┴───────┐
                                          │  client   │
                                          │ m5alice   │
                                          │  alice    │
                                          └───────────┘
```

The Client's `identity_id` is unchanged. Previously signed Events (created while Node A was alive) remain on Nodes B and C as replicas and are verifiable. New Events flow through Node B → C.

---

## Test Data and Identifiers

| Item | Value |
|---|---|
| Node A | `m5nA` at `ws://127.0.0.1:8080/xgen` — alice's original home |
| Node B | `m5nB` at `ws://127.0.0.1:8081/xgen` — becomes alice's new home |
| Node C | `m5nC` at `ws://127.0.0.1:8082/xgen` — federation peer / replica |
| Client | `m5alice` (display name `alice`, passphrase `m5alice-pass-1234`) |
| Space name (P1) | `Multiparty S5 P1` |
| Space name (P2) | `Multiparty S5 P2` |
| Room name (both) | `general` |
| Pre-rehome message count (P2) | 30 |
| Post-rehome message count (P2) | 30 |
| Settle wait after re-home | 60 s |

---

## Milestone 0 — Preparation

**0.1 — Create findings file.**

Create `docs/tests/MULTIPARTY_S5_findings.md` from Appendix A. Status `ACTIVE`.

**0.2 — Record binary versions.**

```
xgen-node-app.exe --version
xgen-client-app.exe --version
```

**0.3 — CAPABILITY GATE — must pass before writing send scripts.**

Verify the three required protocol surfaces are wired through the CLI:

1. **`re_registration` flag exposure.**
   - Inspect `xgen-client-app.exe register --help`. Look for `--re-registration` or equivalent flag (CLI naming may differ from wire field name).
   - If absent, **STOP and mark S5 as BLOCKED.** Open a separate task file at `tasks/MULTIPARTY_S5_BLOCKER_re_registration.md` describing the missing CLI surface and the §3.6 / §3.13.8 spec requirement.

2. **Replica observation.**
   - On a clean 3-Node federation (any test setup will do), register an Identity on Node A. Wait 10 s. Inspect Node B's and Node C's identity registries (via `xgen-node-app.exe identity list` or equivalent, or by direct SQLite query of the registry file). The newly registered Identity SHOULD appear as a replica.
   - If replicas are not visible, **mark S5 as BLOCKED.** §3.13.4 `identity.replicate` push is not occurring. Open a blocker task file.

3. **`identity.home_changed` observability.**
   - Verify that when an `identity.home_changed` Event is emitted (manually constructed or via re-registration if (1) is wired), peer Node logs record it.
   - This is a softer gate — the Event MAY be implemented at the wire level even if (1) isn't yet at the CLI level. Record the state regardless.

Record all three findings in the M0 section of `MULTIPARTY_S5_findings.md`. **Do not proceed past M0 if (1) or (2) fail.** Stop and report.

**0.4 — Spec quotes.**

Quote §3.13.1 (replication model) and §3.13.8 (orphan recovery) verbatim into findings.

**0.5 — Clean workspace.**

All 4 `m5*` data directories cleared or archived. Record paths.

**0.6 — Validate scripts.**

Confirm P1 scripts (Appendix B) and P2 scripts (Appendix C) present at `docs/tests/scripts/`.

### Definition of Done — Milestone 0

- [ ] Findings file created, status `ACTIVE`.
- [ ] Binary versions recorded.
- [ ] Capability gate 0.3: all three items verified. (1) and (2) PASS, or test marked BLOCKED with blocker task filed.
- [ ] §3.13.1 and §3.13.8 quoted in findings.
- [ ] All 4 workspaces clean.
- [ ] All scripts validated.

---

## Milestone 1 — P1 Smoke (single re-home cycle)

**Goal:** alice registers on Node A, joins a Space, sends a message. Node A is then shut down. alice re-registers on Node B with `re_registration: true`. After re-home, alice sends another message and it appears on both Nodes B and C with the same `identity_id` as the pre-re-home message.

### Sequence

**Step P1.1 — Start Node A, Node B, Node C.**

```
xgen-node-app.exe --instance m5nA
xgen-node-app.exe --instance m5nB
xgen-node-app.exe --instance m5nC
```

Wait for all three `READY`. Record timestamps.

**Step P1.2 — Federate full mesh (A↔B, A↔C, B↔C).**

Verify each Node's federation registry contains the other two. Record.

**Step P1.3 — Start Client, register on Node A, create Space.**

```
xgen-client-app.exe --instance m5alice
xgen-client-app.exe --instance m5alice --batch docs/tests/scripts/multiparty_s5_smoke_pre_rehome.xgb
```

Pre-rehome script (Appendix B.1): connect to Node A, `register alice`, `create-space "Multiparty S5 P1"`, `create-room general`, `send "alice-pre-rehome-1"`. Capture Identity ID, Space ID, Room ID. Wait for exit 0.

**Step P1.4 — Wait for replication.**

Wait 15 s for Nodes B and C to receive the Identity replica record (`identity.replicate` per §3.13.4) and for the Space/Room/message Events to propagate. Confirm:

- Node B's identity registry shows alice as a replica (home_node = m5nA's node_id).
- Node C's identity registry shows alice as a replica.
- All three Nodes' Space stores contain the `state.space_create`, `state.room_create`, `membership.join`, and `message.text` Events.

Record alice's `identity_id` from each Node's view — must be byte-identical across A, B, C.

**Step P1.5 — Preserve keypair, shut down Node A.**

**Critical:** before shutting down Node A, verify that `m5alice`'s keypair file is preserved in the Client's data directory and is independent of Node A's lifecycle. The Client's keypair is on the Client side, not the Node side — this is a sanity check.

Shut down `xgen-node-app.exe --instance m5nA` (graceful shutdown). Node A is now offline.

Do NOT shut down the Client. The Client is still running but its WebSocket to Node A is broken; it should detect this and enter a disconnected state per Appendix E lifecycle states.

**Step P1.6 — Re-register on Node B.**

```
xgen-client-app.exe --instance m5alice --batch docs/tests/scripts/multiparty_s5_smoke_rehome.xgb
```

Re-home script (Appendix B.2):

- `connect ws://127.0.0.1:8081/xgen` (switch to Node B).
- `register --name alice --passphrase m5alice-pass-1234 --re-registration` (use the same passphrase to unlock the same keypair; flag indicates re-home).
- `send --space <SPACE_ID> --room <ROOM_ID> --text "alice-post-rehome-1"`.
- `status`.

Wait for exit 0.

The CLI flag name is provisional — match whatever name `xgen-client-app.exe register --help` reveals from the M0.3 gate.

**Step P1.7 — Wait for `identity.home_changed` propagation.**

Wait 30 s. Confirm:

- Node B's identity registry shows alice with `home_node = m5nB` (changed from m5nA).
- Node C's identity registry shows the same.
- The `identity.home_changed` Event appears in Node C's log.

**Step P1.8 — Verify continuity.**

For each surviving Node (B and C):

- alice's `identity_id` is byte-identical to what was recorded in P1.4 (same keypair = same ID).
- The pre-re-home message `alice-pre-rehome-1` is still in the store and its signature still verifies. (Verifiable manually if direct signature-check tools exist, or implicitly by the fact that the Node's state has not rejected it.)
- The post-re-home message `alice-post-rehome-1` is in the store and is correctly attributed to the same `identity_id` as the pre-re-home one.
- alice's Space membership has not been duplicated — there is one membership record, not two.

**Step P1.9 — Pairing check.**

| Message | event_id | Author identity_id | In Node B store | In Node C store |
|---|---|---|---|---|
| alice-pre-rehome-1 | _aaa..._ | _alice_id_ | ✔ | ✔ |
| alice-post-rehome-1 | _bbb..._ | _alice_id_ (same) | ✔ | ✔ |

The two messages must have the **same `Author identity_id`** column value. If they differ, re-homing failed — re-registration produced a new ID.

**Step P1.10 — Content-leak check.**

```
findstr /S /M /R "alice-pre-rehome-1\|alice-post-rehome-1" *.log
```

Zero unauthorised occurrences.

**Step P1.11 — Clean shutdown.**

Client first, then Node B, then Node C. (Node A was shut down at P1.5.)

### Definition of Done — Milestone 1

- [ ] All three Nodes started cleanly, federated, all visible.
- [ ] Client registered on Node A; Identity replicated to B and C (verified in M0 gate + P1.4).
- [ ] Pre-re-home message sent and present on all three Nodes' stores.
- [ ] Node A shut down cleanly.
- [ ] Re-registration on Node B succeeded with same `identity_id`.
- [ ] `identity.home_changed` propagated to Node C (verified in P1.7 logs).
- [ ] Post-re-home message sent and present on Nodes B and C with same `identity_id` as pre-re-home.
- [ ] alice's Space membership not duplicated.
- [ ] Pairing check ✔.
- [ ] Content-leak check clean.
- [ ] Zero `ERROR` log lines on Nodes B and C (Node A's log is closed; record its final state for reference but it should also be clean before shutdown).
- [ ] P1 verdict recorded: PASS or FAIL.
- [ ] If FAIL: stop, do not proceed to P2.

---

## Milestone 2 — P2 Stress

**Goal:** Sustained chat across a re-home boundary. alice sends 30 messages on Node A, re-homes, sends 30 more on Node B. All 60 messages must appear on all surviving Nodes (B and C) attributed to the same `identity_id`. The DAG must remain coherent across the re-home transition.

### Sequence

**Step P2.1 — Workspace, Nodes, federation.**

Fresh data. Same setup as P1.1–P1.2 with a new Space (`Multiparty S5 P2`).

**Step P2.2 — Register on Node A, create Space, send 30 messages.**

```
xgen-client-app.exe --instance m5alice
xgen-client-app.exe --instance m5alice --batch docs/tests/scripts/multiparty_s5_stress_pre_rehome.xgb
```

Pre-rehome stress script (Appendix C.1): connect, register, create-space, create-room, then 30 sends: `alice-stress-pre-01` through `alice-stress-pre-30`. Wait for exit 0.

**Step P2.3 — Wait for replication and propagation.**

Wait 30 s. Confirm replica records on B and C; confirm all 30 messages on all three Nodes.

**Step P2.4 — Shut down Node A.**

Graceful shutdown of `m5nA`.

**Step P2.5 — Re-home and send 30 more.**

```
xgen-client-app.exe --instance m5alice --batch docs/tests/scripts/multiparty_s5_stress_post_rehome.xgb
```

Post-rehome stress script (Appendix C.2): switch to Node B, `register --re-registration`, then 30 sends: `alice-stress-post-01` through `alice-stress-post-30`. Wait for exit 0.

**Step P2.6 — Drain.**

Wait 60 s for all 30 post-re-home messages to settle on Node B and propagate to Node C. Confirm `identity.home_changed` appears in Node C's log.

**Step P2.7 — Verify final state.**

For each of Node B and Node C:

- Total `message.text` Events in store = 60 (30 pre + 30 post).
- All 60 attributed to the same `identity_id`.
- Zero duplicate `event_id`s.
- alice's `identity_id` is byte-identical to what was recorded pre-re-home.
- `home_node` field for alice's record = `m5nB` (changed from `m5nA`).
- Diff B's Event list against C's Event list — must be empty.

**Step P2.8 — Clean shutdown.**

Client, then Node B, then Node C.

### Metrics to capture

**Event counts (B and C only — A is gone):**

| Metric | Expected | Observed on B | Observed on C |
|---|---|---|---|
| Pre-re-home `message.text` (alice-stress-pre-NN) | 30 | _ | _ |
| Post-re-home `message.text` (alice-stress-post-NN) | 30 | _ | _ |
| **Total** | **60** | _ | _ |
| Duplicates | 0 | _ | _ |
| Orphans | 0 | _ | _ |

**Identity continuity:**

| Check | Expected | Observed |
|---|---|---|
| identity_id pre-re-home vs post-re-home | byte-identical | _ |
| Pre-re-home messages attributed to identity_id | yes | _ |
| Post-re-home messages attributed to same identity_id | yes | _ |
| alice's `home_node` field on B | m5nB | _ |
| alice's `home_node` field on C | m5nB | _ |
| `identity.home_changed` Event in Node C log | present | _ |

**Cross-Node consistency:**

| Check | Expected | Observed |
|---|---|---|
| B's Event list vs C's Event list diff | empty | _ |

**Log hygiene:**

| Metric | Expected | Observed |
|---|---|---|
| `ERROR` lines on Node B | 0 | _ |
| `ERROR` lines on Node C | 0 | _ |
| `ERROR` lines on Client log | 0 (disconnect from A is not an error; record any genuine errors) | _ |
| Membership duplication anomalies | 0 | _ |

### Definition of Done — Milestone 2

- [ ] 60 `message.text` Events authored across the re-home boundary.
- [ ] All 60 present on Node B; all 60 present on Node C.
- [ ] Zero duplicates on either Node.
- [ ] alice's `identity_id` unchanged across re-home.
- [ ] All 60 messages attributed to the same `identity_id`.
- [ ] alice's `home_node` updated correctly on both Nodes after re-home.
- [ ] `identity.home_changed` Event observed in Node C log.
- [ ] B↔C Event list diff empty.
- [ ] Zero `ERROR` log lines on B and C.
- [ ] P2 verdict recorded: PASS or FAIL.

---

## Definition of Done — Test S5 as a whole

- [ ] Milestone 0 (Preparation) all items ticked, including capability gate 0.3 either PASS or test marked BLOCKED with blocker task file.
- [ ] Milestone 1 (P1 Smoke) all items ticked, verdict PASS.
- [ ] Milestone 2 (P2 Stress) all items ticked, verdict PASS.
- [ ] Findings file status set to `COMPLETED` with overall verdict.
- [ ] JOURNAL.md entry written summarising the S5 run.
- [ ] This instruction file's header status updated from `ACTIVE` to `COMPLETED`.

---

## Definition of Done — Entire MULTIPARTY operation

After S5 COMPLETES, the entire 5-file Multiparty suite is closed. Final wrap-up:

- [ ] All five `MULTIPARTY_S{1,2,3,4,5}_*.md` instruction files set to `COMPLETED`.
- [ ] All five `MULTIPARTY_S{1,2,3,4,5}_findings.md` files set to `COMPLETED`.
- [ ] A consolidated JOURNAL.md entry summarises the entire Multiparty operation: what was tested, what was found, what bugs surfaced, what fixes were applied, what spec gaps were recorded.
- [ ] Any DECISIONS.md entries arising from findings (e.g. the §3.2 "forward on accept" sentence raised in S3) are filed.
- [ ] CLAUDE.md updated to reference the Multiparty suite as a permanent regression artifact: when major protocol or transport changes land in future, the suite can be re-run as a high-level integration check.
- [ ] If any FIXES files were created during the suite, they are linked from the Multiparty operation summary.

This file is the last instruction in the suite. After all items above are ticked, Multiparty is COMPLETE.

---

## Appendix A — Findings file template

When M0.1 creates `docs/tests/MULTIPARTY_S5_findings.md`, use this template:

```markdown
# Multiparty Test S5 — Findings
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

| Run | Date | Build / commit | M0 gate | P1 | P2 | Notes |
|---|---|---|---|---|---|---|
| 1 | YYYY-MM-DD | _commit hash_ | _PASS/BLOCKED_ | _PASS/FAIL/N-A_ | _PASS/FAIL/N-A_ | _short note_ |

---

## Milestone 0 — Preparation record

- Binary versions: _paste version strings_
- Workspace state: _clean / archived to ..._
- Scripts verified: _list_

### Capability gate (M0.3) results

**Gate 1 — `re_registration` flag exposure:**
- CLI help output: _paste verbatim_
- Flag present: yes/no
- Result: PASS / BLOCKED (blocker file: _path_)

**Gate 2 — Replica observation:**
- Test setup: _describe_
- Identity registered on Node A: _identity_id_
- Replica visible on Node B after 10 s: yes/no
- Replica visible on Node C after 10 s: yes/no
- Result: PASS / BLOCKED (blocker file: _path_)

**Gate 3 — `identity.home_changed` observability:**
- Method used to test: _describe_
- Event observed in peer logs: yes/no
- Result: PASS / soft / BLOCKED

### Spec quotes

§3.13.1: _verbatim_

§3.13.8: _verbatim_

---

## Milestone 1 — P1 Smoke

### Identity continuity

- alice's identity_id (pre-re-home, recorded on A): _..._
- alice's identity_id (pre-re-home, recorded on B replica): _..._
- alice's identity_id (pre-re-home, recorded on C replica): _..._
- alice's identity_id (post-re-home, new home on B): _..._
- Byte-identical across all four: yes/no

### Pairing check

_(insert table per instruction file format)_

### `identity.home_changed` observation
- Event observed in Node C log at: _timestamp_
- Verbatim log line: _paste_

### Content-leak findstr

```
_(paste verbatim)_
```

### Verdict: _PASS/FAIL_

_Notes:_

---

## Milestone 2 — P2 Stress

### Event counts

_(insert tables)_

### Identity continuity

_(insert table)_

### B↔C diff result

_(insert)_

### Log hygiene

_(insert table)_

### Verdict: _PASS/FAIL_

_Notes:_

---

## Findings — bugs and anomalies

### F-001 — _short title_
- **Severity:** _critical / major / minor_
- **Stage:** _M0.3 gate / P1 step / P2 step_
- **Observed:** _what happened_
- **Expected:** _what should have happened_
- **Resolution:** _link to FIXES file or commit, or "open" or "blocker task filed at..."_

---

## Overall verdict

_PASS / FAIL / BLOCKED_
```

---

## Appendix B — P1 Smoke `.xgb` scripts

### B.1 — `docs/tests/scripts/multiparty_s5_smoke_pre_rehome.xgb`

```
# Multiparty S5 P1 — alice pre-rehome on Node A
# Register, create Space and Room, send one message

connect ws://127.0.0.1:8080/xgen
register --name alice --passphrase m5alice-pass-1234
create-space --name "Multiparty S5 P1"
create-room --space @last_space --name general
send --space @last_space --room @last_room --text "alice-pre-rehome-1"
status
```

### B.2 — `docs/tests/scripts/multiparty_s5_smoke_rehome.xgb`

```
# Multiparty S5 P1 — alice re-home to Node B
# Switch endpoint, re-register with same passphrase (and therefore same keypair),
# send another message into the same Space.
# Space ID and Room ID substituted from P1.3 capture.
# The --re-registration flag name is provisional — match the actual CLI surface
# discovered in the M0.3 capability gate.

connect ws://127.0.0.1:8081/xgen
register --name alice --passphrase m5alice-pass-1234 --re-registration
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-post-rehome-1"
status
```

---

## Appendix C — P2 Stress `.xgb` scripts

### C.1 — `docs/tests/scripts/multiparty_s5_stress_pre_rehome.xgb`

```
# Multiparty S5 P2 — alice pre-rehome stress on Node A
# Register, create Space and Room, send 30 messages

connect ws://127.0.0.1:8080/xgen
register --name alice --passphrase m5alice-pass-1234
create-space --name "Multiparty S5 P2"
create-room --space @last_space --name general
send --space @last_space --room @last_room --text "alice-stress-pre-01"
send --space @last_space --room @last_room --text "alice-stress-pre-02"
send --space @last_space --room @last_room --text "alice-stress-pre-03"
# ... lines 4 through 27 follow the same pattern ...
send --space @last_space --room @last_room --text "alice-stress-pre-28"
send --space @last_space --room @last_room --text "alice-stress-pre-29"
send --space @last_space --room @last_room --text "alice-stress-pre-30"
status
```

### C.2 — `docs/tests/scripts/multiparty_s5_stress_post_rehome.xgb`

```
# Multiparty S5 P2 — alice post-rehome stress on Node B
# Switch endpoint, re-register, send 30 messages into the same Space.

connect ws://127.0.0.1:8081/xgen
register --name alice --passphrase m5alice-pass-1234 --re-registration
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-post-01"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-post-02"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-post-03"
# ... lines 4 through 27 follow the same pattern ...
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-post-28"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-post-29"
send --space <SPACE_ID> --room <ROOM_ID> --text "alice-stress-post-30"
status
```

---

## Sequence Position

| | |
|---|---|
| **This file** | 5 of 5 (final) |
| **Previous** | `MULTIPARTY_S4_n_clients_n_nodes.md` |
| **Next** | — (end of Multiparty operation) |

After this file's Definition of Done is fully ticked, the Multiparty operation is closed. See "Definition of Done — Entire MULTIPARTY operation" above for the final wrap-up checklist.

---

*End of MULTIPARTY_S5_client_rebind.md*
