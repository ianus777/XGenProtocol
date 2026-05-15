# XGen Protocol — Full Integration Stress Test
> **Status:** PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated:** 2026-05-15  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  
> See also: `docs/xgen_ch4_implementation.md` §4.19 — summary and scenario table  
> See also: `docs/xgen_appendix_h_en.md` §H.2 — full output record (filled after run)  
> See also: `docs/tests/INTEGRATION_TEST_ph2.md` — smoke test instruction file  

---

## Purpose

This is the instruction file for the full integration stress test. Its role is to prove that the complete XGen reference implementation — Phase 1 and Phase 2 together — is correct under concurrent load and adversarial conditions. The smoke test (`smoke-ph2`) proved functional correctness step by step. This test proves correctness when many things happen at the same time.

The Phase 1 stress test proved message delivery at load (500 messages, 10 identities, 2 nodes). This test extends that proof across all Phase 2 protocol layers: E2E encryption at message load, state resolution under concurrent conflict, space migration while traffic is flowing, identity replication to a three-node topology, and DM promotion during an active message flood.

**What this test adds beyond the smoke test and Phase 1 stress test:**

- Concurrency within Phase 2 features — not just one thing at a time
- A third Node (Node C) doubling as a Bootstrap Node — the first test of the full 3-node topology
- Volume: more events, more members, more simultaneous operations
- Adversarial sequences: conflicting state events, migration during active traffic, replica queries without the home node

---

## Mandatory Rules

All CLAUDE.md mandatory rules apply. In particular:

- Do not write the journal entry or mark the Definition of Done until all scenarios are confirmed PASS with actual terminal output.
- Do not paraphrase the output. Paste it verbatim into Appendix H §H.2.
- If any scenario fails, stop, report the exact error, and do not continue to the next scenario.
- The comm record (`stress_complete_events.json`) must be written before the Definition of Done is marked complete.

---

## Prerequisites

Before starting:

- [ ] `xgen-node.exe` and `xgen-client.exe` are built from the latest source (`cargo build` or `build.sh`)
- [ ] All 300 unit tests pass (`cargo test --workspace`)
- [ ] `smoke-ph2` passes 60/60 (the full integration smoke test must be green before the stress test begins)
- [ ] Three test node directories exist and are clean (no stale state files, no stale databases):
  - `test/node_a/` — Node A
  - `test/node_b/` — Node B
  - `test/node_c/` — Node C (also Bootstrap Node)
- [ ] All three nodes have been initialised (`xgen-node --config <path> init`) with fresh keypairs
- [ ] Node C config has `[bootstrap] enabled = true` (see Environment Setup below)
- [ ] No other processes are bound to ports 9080, 9081, 9082

---

## Environment Setup

### Node directories

| Node | Directory | Port | Role |
|---|---|---|---|
| Node A | `test/node_a/` | `ws://127.0.0.1:9080/xgen` | Standard node |
| Node B | `test/node_b/` | `ws://127.0.0.1:9081/xgen` | Standard node |
| Node C | `test/node_c/` | `ws://127.0.0.1:9082/xgen` | Standard node + Bootstrap Node |

All config files follow the standard `xgen-node_config.toml` schema. Node C requires one additional section:

```toml
# test/node_c/xgen-node_config.toml — additional section for Bootstrap capability
[bootstrap]
enabled = true
directory_url = "http://127.0.0.1:9082/bootstrap"
accepts_registrations = true
region = "local-test"
operator = "stress-test"
```

The Bootstrap HTTP endpoint (port 9082) is served alongside the WebSocket endpoint. Record the port separation decision in DECISIONS.md if not already recorded (see IMPLEMENTATION_GUIDE_ph2.md §Layer 17).

### Log level

All three nodes run at `debug` level for the duration of the stress test. This is required for the direction=IN verification and the content leak check. Restore to `info` after the test.

### Starting the nodes

Open three terminal windows. From `bin/`:

```
Terminal 1: xgen-node --config ..\test\node_a\xgen-node_config.toml
Terminal 2: xgen-node --config ..\test\node_b\xgen-node_config.toml
Terminal 3: xgen-node --config ..\test\node_c\xgen-node_config.toml
```

Wait for all three to emit their startup log line before invoking the client.

---

## The `stress-complete` Subcommand

### What to implement

Add `stress-complete` as a new subcommand to `xgen-client`, alongside the existing `smoke-test`, `smoke-ph2`, and `stress-test` subcommands. The handler lives in `xgen-client/src/commands/stress_complete.rs`. Register it in `main.rs` under the existing command dispatch.

### Flags

| Flag | Short | Default | Description |
|---|---|---|---|
| `--node-a <url>` | `-a` | required | WebSocket endpoint for Node A |
| `--node-b <url>` | `-b` | required | WebSocket endpoint for Node B |
| `--node-c <url>` | `-c` | required | WebSocket endpoint for Node C (Bootstrap) |
| `--members <n>` | `-m` | `10` | Total members for Scenario 0 and 1 (split evenly across Node A and Node B) |
| `--messages-per-member <n>` | | `50` | Messages per member in Scenario 0 and 1 |
| `--log-level <level>` | | `info` | Client-side log level |

### Invocation

```
xgen-client stress-complete \
  --node-a ws://127.0.0.1:9080/xgen \
  --node-b ws://127.0.0.1:9081/xgen \
  --node-c ws://127.0.0.1:9082/xgen
```

### Output format

The output banner follows the same style as `smoke-ph2`. Each scenario is a named section. Each step within a scenario prints `[PASS]` or `[FAIL]` with a descriptive label. Scenario summaries print at the end of each scenario. A final result block prints at the end of the run.

```
════════════════════════════════════════════════════════════
STRESS-COMPLETE — Full Integration Stress Test
════════════════════════════════════════════════════════════
Node A:  ws://127.0.0.1:9080/xgen
Node B:  ws://127.0.0.1:9081/xgen
Node C:  ws://127.0.0.1:9082/xgen (Bootstrap)
Members: 10  Messages/member: 50
── Scenario 0: Phase 1 Regression ──────────────────────────
...
── Scenario 0 RESULT ────────────────────────────────────────
Sent: 500/500  Errors: 0  Join failures: 0  Duration: Xs
[PASS] Scenario 0
── Scenario 1: E2E Encryption Flood ────────────────────────
...
[PASS] Scenario 1
...
════════════════════════════════════════════════════════════
STRESS-COMPLETE RESULTS
════════════════════════════════════════════════════════════
Scenario 0 — Phase 1 Regression          PASS
Scenario 1 — E2E Encryption Flood        PASS
Scenario 2 — State Conflict Storm        PASS
Scenario 3 — DM Promotion Under Load     PASS
Scenario 4 — Space Migration Under Traffic PASS
Scenario 5 — Identity Replication        PASS
────────────────────────────────────────────────────────────
TOTAL  6/6 PASS
Node A: ws://127.0.0.1:9080/xgen
Node B: ws://127.0.0.1:9081/xgen
Node C: ws://127.0.0.1:9082/xgen
Duration: Xs
════════════════════════════════════════════════════════════
STRESS-COMPLETE PASSED — 6/6 scenarios
```

If any scenario fails the run halts immediately, prints the failure detail, and exits with a non-zero exit code.

### Comm record

Write a JSON comm record to `docs/tests/stress_complete_events.json` at the end of a successful run. Follow the same schema as `STRESSTEST_ph1_events.json` (seq, ts, phase, actor, direction, event_type, event_id, node, prev_events, ok, notes). The `phase` field uses the scenario name string (e.g. `"s0_regression"`, `"s1_encryption"`, `"s2_conflict_storm"`, `"s3_dm_promotion"`, `"s4_migration"`, `"s5_replication"`).

Message content is never stored in any field of the comm record. The `notes` field records room and message index for flood events only.

---

## Implementation Notes

### Code structure

All scenario logic lives in `stress_complete.rs`. Do not put scenario logic in `main.rs`. The file structure within the handler mirrors the scenario order: one async function per scenario, called in sequence from the top-level `run_stress_complete()` function.

Shared setup (node connections, federation establishment, identity generation) is handled in a `setup()` function called before Scenario 0. Each scenario receives the setup context as a parameter.

### Connection management

Each member holds one persistent WebSocket connection per node it is registered on. The same reconnect-once-and-retry pattern from the Phase 1 stress test applies: on connection failure, reconnect, re-authenticate, retry the same event (same event_id and prev_events), then count as error if the retry also fails.

### Federation ordering

Establish all federation connections before spawning any member joins. Member joins must not begin until all relevant federation sessions are ACTIVE. This is the same ordering rule as Phase 1 — do not relax it.

### Three-node federation

For scenarios involving Node C, federate A↔C and B↔C before beginning the scenario. A↔B federation is established during setup (used from Scenario 0 onward). C is brought in only when a scenario requires it (Scenario 5). Scenarios 0–4 use only Node A and Node B.

---

## Scenario 0 — Phase 1 Regression

**Purpose:** Confirm that Phase 2 code has not broken Phase 1 message delivery under concurrent load. This is the identical workload as the Phase 1 stress test proof run — zero tolerance for regressions.

**Setup (shared with Scenario 1):**

- Alice (M0) registers on Node A and creates one Space with 3 Rooms: `general`, `random`, `tech`
- M1–M4 register on Node A; Alice invites them all
- M5–M9 register on Node B
- Alice invites M5–M9 (federation must be ACTIVE before invitations are sent)
- All members join the Space and all 3 Rooms
- MLS is **not** active in this scenario — plain-text messages only

**Message flood:**

- All 10 members send `--messages-per-member` messages concurrently
- Room assignment: `msg_index % 3` (round-robin across `general`, `random`, `tech`)
- Each member maintains a sequential `prev_events` chain

**Pass criteria for Scenario 0:**

| Check | Method | Threshold |
|---|---|---|
| Messages sent | Comm record | 500/500 (or `members × mpm`) |
| Send errors | Comm record | 0 |
| Join failures | Comm record | 0 |
| Reconnects | Comm record | 0 (report if any occur, do not fail) |
| DAG chain integrity | Comm record scan | OK for all members |
| Content leak — client log | Log scan (`M\d+ msg \d+`) | 0 matches |
| Content leak — Node A log | Log scan | 0 matches |
| Content leak — Node B log | Log scan | 0 matches |
| direction=IN — Node A | Log grep | count > 0 (report exact) |
| direction=IN — Node B | Log grep | count > 0 (report exact) |
| Federation propagation | Node B log grep for `state.space_create` direction=IN | present |
| ERROR lines — Node A log | Log grep | 0 (excluding known `event held pending` WARN lines) |
| ERROR lines — Node B log | Log grep | 0 (excluding known `event held pending` WARN lines) |

Print the Scenario 0 result block with all counts before proceeding.

---

## Scenario 1 — E2E Encryption Flood

**Purpose:** Prove that the MLS delivery pipeline is stable under the same concurrent message load as Scenario 0. MLS epoch management must not break under 10 concurrent senders.

**Setup (continues from Scenario 0 — same Space, same members, same Rooms):**

- Each member generates a fresh MLS KeyPackage and uploads it via `mls.key_package` event to their home Node for each Room
- Alice creates the MLS group for each Room (sends `mls.welcome` + `mls.commit`)
- All non-Alice members receive their `mls.welcome` and initialise their local MLS group state (epoch 0)
- Verify epoch 0 is established before starting the flood (check: `mls.welcome` received by all members for all 3 Rooms)

**Message flood:**

- Same 10 members, same `--messages-per-member`, same round-robin room assignment
- Every `message.text` event carries an encrypted content blob — `enc:` prefix on the `content` field
- Each member encrypts using their current MLS group state for the target Room before sending

**Pass criteria for Scenario 1:**

| Check | Method | Threshold |
|---|---|---|
| Messages sent | Comm record | 500/500 |
| Send errors | Comm record | 0 |
| enc: prefix on all message.text events | Comm record scan | 100% of message.text events |
| Content leak — all logs | Log scan (any plaintext pattern) | 0 matches |
| Node stores enc: blob only | Node A/B log scan | plaintext never appears |
| Epoch stability | Comm record — no unexpected `mls.commit` during flood | 0 unexpected epoch advances |
| Forward secrecy spot-check | Remove M9 from group mid-flood (after msg 25); attempt decrypt with M9's epoch-0 key on a post-removal message | Must fail |
| ERROR lines — all Node logs | Log grep | 0 |

The forward secrecy spot-check is implemented as: after M9 has sent 25 messages, M0 (Alice) sends a Remove Proposal + Commit for M9. The client-side test then attempts to decrypt a message sent after the epoch advance using M9's pre-removal key material. The decryption must fail (return an error, not silently produce garbage). Print the outcome as a named check in the Scenario 1 output.

Print the Scenario 1 result block before proceeding.

---

## Scenario 2 — State Conflict Storm

**Purpose:** Prove that the state resolution algorithm converges correctly under concurrent conflicting events. Both nodes must arrive at identical membership state after the storm, and losing events must remain in the DAG.

**Setup:**

- New Space on Node A (fresh — not the Scenario 0/1 Space)
- 5 members registered on Node A: Carol, Dave, Eve, Frank, Grace (roles: all `member` initially)
- Alice (owner) and Bob (admin) also in the Space
- Federate Node A ↔ Node B so both nodes receive all events

**Conflict group 1 — membership ban vs invite (5 pairs):**

For each of Carol, Dave, Eve, Frank, Grace:
- Alice (owner) sends `membership.ban` for that member
- Bob (admin) sends `membership.invite` for the same member with `role=member`
- Both events are sent as close to simultaneously as possible (spawn both tasks, `join_all`)
- Expected winner: `membership.ban` (Layer 1 — ban beats invite)

After all 5 pairs are sent, wait 500ms for propagation, then query membership state on both Node A and Node B.

**Conflict group 2 — room rename by different roles (3 rooms):**

- Create 3 Rooms in the same Space: `alpha`, `beta`, `gamma`
- For each Room, concurrently send `state.room_name` from Alice (owner), Bob (admin), and Carol (member — if Carol is banned from group 1, use a separate member identity `Member6` with role=member registered before the ban storm)
- Names sent: `<room>-owner`, `<room>-admin`, `<room>-member` (e.g. `alpha-owner`, `alpha-admin`, `alpha-member`)
- Expected winner: owner's name (Layer 4 — highest role wins)

After all 3 rename triplets are sent, wait 500ms, then query Room state on both nodes.

**Pass criteria for Scenario 2:**

| Check | Method | Threshold |
|---|---|---|
| All 5 members banned on Node A | State query | Carol, Dave, Eve, Frank, Grace status = `banned` |
| All 5 members banned on Node B | State query | Same |
| Node A = Node B membership state | Comparison | Identical |
| Losing invite events in Node A DAG | DAG query by event_id | 5 invite events present |
| Losing invite events in Node B DAG | DAG query by event_id | 5 invite events present |
| Room names = owner's choice on Node A | State query | `alpha-owner`, `beta-owner`, `gamma-owner` |
| Room names = owner's choice on Node B | State query | Same |
| Losing rename events in both DAGs | DAG query by event_id | 6 losing events present (2 per room) |
| ERROR lines — all Node logs | Log grep | 0 |

Print Scenario 2 result block before proceeding.

---

## Scenario 3 — DM Promotion Under Load

**Purpose:** Prove that DM Space lifecycle (constraint enforcement → message exchange → promotion) works correctly while unrelated traffic is flowing on other Spaces.

**Setup:**

- Use the Scenario 0/1 Space on Node A/B as the background load Space (it can be idle at this point — no active flood needed, but it exists with all its members)
- Two new identities: Eve2 and Frank2, both registered on Node A
- Eve2 creates a DM Space on Node A

**DM constraint enforcement:**

- Attempt to invite a third member (Grace2) to the DM Space — Node A must reject
- Attempt to create a second Room in the DM Space — Node A must reject
- Record both rejections as [PASS] checks

**DM message exchange:**

- Eve2 and Frank2 exchange 50 encrypted messages in the DM Space (round-trip, alternating senders, MLS active)
- All 50 messages must send and receive without error

**DM promotion:**

- Eve2 sends `dm.promote_propose`
- Frank2 sends `dm.promote_confirm`
- Node A produces `state.dm_promote` event — verify it is signed by Node A (not by Eve2 or Frank2)
- Verify `dm_constraints_active = false` after promotion

**Post-promotion invite:**

- Invite Grace2 to the promoted Space — must succeed (constraints lifted)
- Grace2 joins — must succeed

**Background load check:**

- While the above is running, launch 3 members from the Scenario 0/1 Space sending 20 additional messages each to the `general` Room on Node A
- These messages must all deliver without error
- Verify the two workloads do not interfere (zero errors on both)

**Pass criteria for Scenario 3:**

| Check | Method | Threshold |
|---|---|---|
| Third-member invite rejected | Response code | Error returned (DM constraint) |
| Second Room creation rejected | Response code | Error returned (DM constraint) |
| 50 DM messages sent | Comm record | 50/50, 0 errors |
| state.dm_promote in DM Space DAG | DAG query | Present |
| state.dm_promote signed by Node | Signature check | Signer = Node A keypair |
| dm_constraints_active = false | State query | false |
| Grace2 invite accepted post-promotion | Response | Accept |
| Grace2 join accepted | Response | Accept |
| Background flood: 60 messages | Comm record | 60/60, 0 errors |
| DM pre-promotion message count | DAG query | 50 events present and intact |
| ERROR lines — all Node logs | Log grep | 0 |

Print Scenario 3 result block before proceeding.

---

## Scenario 4 — Space Migration Under Traffic

**Purpose:** Prove that Space migration correctly captures in-flight messages via the tail batch mechanism when migration is initiated while senders are actively flooding the Space.

**Setup:**

- New Space on Node A with at least 20 pre-existing events (create Space + Room + 15 messages from Alice)
- 3 concurrent senders: Alice (M0), M1, M2 — all registered on Node A

**Migration under flood:**

1. Start all 3 senders concurrently, each sending 30 messages to the `general` Room (90 total flood messages)
2. After the first 10 messages per sender are confirmed sent (30 events on the wire), Alice sends `migration.request` targeting Node B
3. Migration proceeds: `migration.propose` → `migration.accept` → `migration.event_batch` transfers → tail batch captures events produced during transfer → `migration.verified` → `state.space_migrate`
4. The 3 senders continue sending their remaining 20 messages each during migration — these fall into the tail batch
5. After `state.space_migrate` is committed, all 3 senders send 10 additional messages directed at Node B (post-migration)

**Event count verification:**

| Batch | Expected count |
|---|---|
| Pre-migration historical events | 20 (setup) + 30 (first phase of flood) = 50 |
| Tail batch (events during transfer) | Up to 60 (remaining flood messages) — exact count from comm record |
| Post-migration messages | 30 (3 senders × 10) |
| **Total on Node B** | Pre + tail + post |

The exact tail batch size depends on timing and will vary between runs. The check is: `pre + tail + post = total events sent` with zero missing events. This is verified by comparing the Node B event count after migration with the total events in the comm record.

**Merkle root verification:**

Node B's `migration.verified` message carries a Merkle root computed over all transferred event IDs. This must match the root declared by Node A in `migration.complete`. Print both roots and the match result as named checks.

**Post-migration access:**

- Query a pre-migration event from Node B by event_id — must return the event
- Send one message from M5 (on Node B) to the migrated Space on Node B — must succeed
- Query the post-migration message from Node A — Node A must redirect or return `state.space_migrate` indicating the new host

**Pass criteria for Scenario 4:**

| Check | Method | Threshold |
|---|---|---|
| Total flood messages sent | Comm record | 90/90, 0 errors |
| migration.verified received | Comm record | Present |
| Merkle root match | Root comparison | Match |
| state.space_migrate in DAG | DAG query | Present |
| Total events on Node B | Event count | pre + tail + post (no gaps) |
| Pre-migration event accessible on Node B | Query by event_id | Returns event |
| Post-migration message accepted by Node B | Response | Accept |
| ERROR lines — Node A/B logs | Log grep | 0 |

Print Scenario 4 result block before proceeding.

---

## Scenario 5 — Identity Replication and Bootstrap Discovery

**Purpose:** Prove identity replication across a 3-node topology and Bootstrap Node directory-based peer discovery. This is the only scenario that uses Node C.

**Part A — Identity Replication:**

**Setup:**

- Federate Node A ↔ Node C and Node B ↔ Node C (in addition to the existing A↔B federation)
- Register 20 new identities on Node A in 4 concurrent batches of 5 (`tokio::spawn` 5 registration tasks, `join_all`, repeat 4 times with a 200ms gap between batches)

**Replication wait:**

After the last batch completes, wait 2 seconds for replication propagation. This is the maximum expected propagation delay for a 3-node localhost topology. If any replica query fails after 2s, report the specific identity and the node queried — do not retry silently.

**Replica queries:**

- Query all 20 identities from Node B (none are registered on B — they must come from B's replica store)
- Query all 20 identities from Node C (same — replica-only)
- Every query must return the correct `display_name` and `public_key`

**Pass criteria for Part A:**

| Check | Method | Threshold |
|---|---|---|
| 20 registrations on Node A | Comm record | 20/20, 0 errors |
| All 20 resolved from Node B | Query results | 20/20 correct records |
| All 20 resolved from Node C | Query results | 20/20 correct records |
| Replication factor respected | identity.replicate events in comm record | ≤ N=3 replicate events per identity |

**Part B — Bootstrap Node Discovery:**

**Setup:**

- Node C is running with `[bootstrap] enabled = true` (from Environment Setup)
- Node B registers itself with Node C's Bootstrap directory by sending `bootstrap.node_register` to Node C
- Node C acknowledges with `bootstrap.node_register_ack`

**Discovery:**

- Start a fresh client session with no prior knowledge of Node A
- Query Node C's Bootstrap directory for available peers (`bootstrap.node_lookup`)
- Node C returns a peer list from its directory — Node A must appear in the list
- Use the returned Node A endpoint to open a WebSocket connection and run the Phase 1 transport challenge-response handshake
- Register a new identity on Node A via the Bootstrap-discovered endpoint

**Directory signature verification:**

- Fetch Node C's HTTP directory endpoint (`GET http://127.0.0.1:9082/bootstrap`)
- Verify the returned JSON is signed by Node C's keypair
- Verify Node A and Node B both appear in the directory (Node B registered in setup; Node A was announced during federation)

**Pass criteria for Part B:**

| Check | Method | Threshold |
|---|---|---|
| bootstrap.node_register_ack received | Comm record | Present |
| bootstrap.node_lookup returns Node A | Lookup response | Node A endpoint present |
| Connection to Bootstrap-discovered Node A | Handshake | Completes successfully |
| Identity registration via discovered endpoint | Comm record | Success |
| HTTP directory signed by Node C | Signature verify | Valid |
| Node A and Node B in HTTP directory | Directory content | Both present |
| ERROR lines — all Node logs | Log grep | 0 |

Print Scenario 5 result block, then print the final STRESS-COMPLETE RESULTS block.

---

## Verification Checklist

After all 6 scenarios pass, complete this checklist. Each item must be independently confirmed with actual output — do not mark complete by inference.

### Automated checks (confirmed by comm record and client output)

- [ ] Scenario 0: 500/500 messages, 0 errors, 0 join failures
- [ ] Scenario 0: DAG chain integrity — all member chains unbroken
- [ ] Scenario 1: 500/500 messages, 0 errors, all enc: prefix
- [ ] Scenario 1: Forward secrecy spot-check — post-removal decryption fails
- [ ] Scenario 2: All 5 members banned on both nodes
- [ ] Scenario 2: Room names = owner's choice on both nodes
- [ ] Scenario 2: 11 losing events present in both DAGs (5 invites + 6 renames)
- [ ] Scenario 3: 50 DM messages, 0 errors; 60 background messages, 0 errors
- [ ] Scenario 3: state.dm_promote signed by Node A
- [ ] Scenario 3: Post-promotion invite and join succeed
- [ ] Scenario 4: 90 flood messages, 0 errors
- [ ] Scenario 4: Merkle root match confirmed
- [ ] Scenario 4: Total events on Node B = pre + tail + post (no gaps)
- [ ] Scenario 5A: 20/20 identities resolved from Node B via replica
- [ ] Scenario 5A: 20/20 identities resolved from Node C via replica
- [ ] Scenario 5B: Bootstrap discovery → connection → registration succeeds
- [ ] Scenario 5B: HTTP directory signed and correct

### Manual checks (confirmed by node log inspection)

- [ ] Content leak — Node A log: 0 plaintext message matches in any scenario
- [ ] Content leak — Node B log: 0 plaintext message matches in any scenario
- [ ] Content leak — Node C log: 0 plaintext message matches
- [ ] ERROR lines — Node A log: 0 (excluding expected WARN lines)
- [ ] ERROR lines — Node B log: 0
- [ ] ERROR lines — Node C log: 0
- [ ] direction=IN — Node A: count reported (do not check against expected — record actual)
- [ ] direction=IN — Node B: count reported
- [ ] direction=IN — Node C: count reported
- [ ] Session footer present in all 3 node logs (nodes stopped with Ctrl+C, not SIGKILL)

---

## Post-Run Documentation Requirements

These four items are part of the Definition of Done. Do them in this order after all scenarios pass and all verification checklist items are confirmed.

### 1. Write the comm record

Write `docs/tests/stress_complete_events.json`. Schema is identical to `STRESSTEST_ph1_events.json`. Use the `phase` values: `"s0_regression"`, `"s1_encryption"`, `"s2_conflict_storm"`, `"s3_dm_promotion"`, `"s4_migration"`, `"s5_replication"`.

### 2. Fill Appendix H §H.2

Open `docs/xgen_appendix_h_en.md`. Replace the §H.2 placeholder block (currently: `Status: PENDING — to be filled after the stress test run completes`) with the complete terminal output from the run:

- Full per-scenario step output (every `[PASS]` line)
- Full per-scenario result blocks
- The final `STRESS-COMPLETE RESULTS` banner
- The verification checklist results (automated items from comm record)
- The per-scenario timing table
- The node log direction=IN breakdown (same format as `STRESSTEST_ph1_results.md`)

Add the run metadata at the top of §H.2: date, binary version, environment, duration, commit hash.

Update the §H.2 header: remove `Status: PENDING`, add `Result: PASS — 6/6 scenarios — <date>`.

### 3. Update Ch4 §4.19

Open `docs/xgen_ch4_implementation.md`. In §4.19, replace `**Result: PENDING**` with the result line, timing, and reference:

```
**Result: PASS — 6/6 scenarios — <date> (J-0XX)**

Environment: Node A `ws://127.0.0.1:9080/xgen`, Node B `ws://127.0.0.1:9081/xgen`,
Node C `ws://127.0.0.1:9082/xgen` (Bootstrap), debug build. Duration: Xs.
300/300 unit tests confirmed passing.

Full scenario output: **Appendix H §H.2**. Instruction file: `docs/tests/STRESS_TEST_complete.md`.
```

Update the §4.19 status in the Phase 2 section skeleton table from `⏳ Pending` to `✅ Complete — 6/6 PASS`.

Update the Ch4 header `**Last updated**` line.

### 4. Write the journal entry

Write a new JOURNAL.md entry (J-0XX — increment from the last entry). Follow the standard journal format. The entry must include:

- Scope: what was done
- Work performed: the test run, any issues encountered and resolved, the exact commands used
- Verification: the actual terminal output (the final STRESS-COMPLETE RESULTS banner)
- Definition of Done checklist with all items confirmed

Update the JOURNAL.md header `**Last updated**` line.

---

## Definition of Done

- [ ] All 6 scenarios report PASS in the `stress-complete` terminal output
- [ ] All automated verification checklist items confirmed from comm record
- [ ] All manual verification checklist items confirmed from node log inspection
- [ ] `docs/tests/stress_complete_events.json` written
- [ ] Appendix H §H.2 filled with full verbatim output
- [ ] Ch4 §4.19 updated with result, date, and reference to Appendix H §H.2
- [ ] Ch4 §4.19 status in section skeleton table updated to `✅ Complete`
- [ ] JOURNAL.md entry written with actual terminal output quoted
- [ ] CLAUDE.md updated: stress test complete, next priority set
- [ ] This file (`STRESS_TEST_complete.md`) header updated: Status `PENDING` → `COMPLETED`

---

*End of document*
