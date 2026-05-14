# XGen Protocol — Phase 2 Integrated Stress Test
> **Status:** PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated:** 2026-05-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## ⚠️ PROJECT ROOT — Read before opening any file

All work in this task file is performed in the project root:

**`E:\Projects\XGenProtocol`**

Before starting any work, confirm your working directory:

```
echo %CD%
```

Expected output: `E:\Projects\XGenProtocol`

**If your shell shows `.claire\worktrees\`, `.claude\worktrees\`, or any other path — stop.
Navigate to `E:\Projects\XGenProtocol` before touching a single file.**

All file paths in this document are relative to `E:\Projects\XGenProtocol`. Do not create,
edit, or compile from worktrees, temp directories, or any subdirectory unless explicitly stated.

---

## Purpose

This test verifies XGen Protocol behaviour under load with Phase 2 features active.
Where the integration smoke test (INTEGRATION_TEST_ph2.md) checks correctness of each
feature once, this test checks that they hold under concurrent, high-volume conditions.

Target: surface race conditions, DAG consistency failures, state resolution divergence,
and epoch management breakdowns that only appear under sustained concurrent load.

The test produces a full communication record (comm_log) and a summary report.
Both are pasted into `JOURNAL.md` as the Phase 2 stress test evidence.

---

## Prerequisites

- `INTEGRATION_TEST_ph2.md` COMPLETE — integration smoke test must pass before stress test runs
- `cargo test` passing: 300/300 ✅
- Both Node binaries compiled and present in `bin/`
- Both Node instances running before the command is invoked
- Sufficient local ports available (Nodes on 8080 and 8081)

---

## Part A — Extend `stress-test` Command

The existing `stress-test` command tests Phase 1 features with up to 20 members and
a configurable message count. Extend it to support Phase 2 stress scenarios.

### Changes to `StressTestArgs` in `xgen-client/src/main.rs`

**Raise the members cap from 20 to 50:**

```rust
/// Total number of test identities (min 2, max 50). Default: 10.
#[arg(long, default_value = "10")]
members: usize,
```

Change `.clamp(2, 20)` to `.clamp(2, 50)` in `cmd_stress_test`.

**Add `--phase2` flag:**

```rust
/// Enable Phase 2 stress scenarios: concurrent state conflicts, E2E encryption
/// under load, and space migration during active message traffic.
/// Requires --members >= 4 and --messages >= 100.
#[arg(long)]
phase2: bool,
```

**Add `--conflicts` flag:**

```rust
/// Number of concurrent conflicting membership events to generate in Phase 5.
/// Default: 100. Only used when --phase2 is set.
#[arg(long, default_value = "100")]
conflicts: usize,
```

**Add `--epochs` flag:**

```rust
/// Number of MLS epoch changes (member add/remove cycles) to perform in Phase 6.
/// Default: 100. Only used when --phase2 is set.
#[arg(long, default_value = "100")]
epochs: usize,
```

---

## Part B — Phase 2 Stress Phases

When `--phase2` is set, the existing 4-phase stress test runs first (unchanged), then
the following three phases are appended. All new phases use the same `comm_log` and
`seq` counter as existing phases, producing one unified communication record.

---

### Phase 5 — Concurrent State Conflicts (--conflicts events)

**Goal:** verify that state resolution produces the same winner on both nodes under
concurrent load. Divergence between Node A and Node B state after resolution is a failure.

**Setup:** reuse the Space and member set from Phase 1 of the existing stress test.
Select the first 4 members as the conflict participants: Alice (owner), M1 (admin),
M2 (moderator), M3 (member).

**Step sequence:**

1. Generate `--conflicts` pairs of conflicting events. Each pair targets a different
   test identity Mx and consists of:
   - Event A: `membership.ban` targeting Mx (sent by Alice, role=owner)
   - Event B: `membership.invite` targeting Mx (sent by M1, role=admin, same `prev_events`)
   Both events share the same `prev_events` tip — genuine DAG fork on the same state key.

2. Send all events concurrently using Tokio tasks. Split evenly: half sent to Node A,
   half to Node B. Record send timestamps and returned event IDs.

3. Wait for propagation (2 × `rest_ms` after last send).

4. Query final membership state from both Node A and Node B for each targeted identity.

5. For each target:
   - Both nodes must agree on the winner (same membership status)
   - The winner must be `banned` (Layer 1 of resolution: ban beats invite)
   - Both events must be present in the DAG on both nodes

**Failure conditions:**
- Node A and Node B disagree on winner for any target → `STATE_DIVERGENCE` error
- Winner is not `banned` → `WRONG_WINNER` error
- Either event missing from DAG → `DAG_MISSING_EVENT` error

**Comm_log entries:** one entry per conflict pair with `phase=conflict`, recording
both event IDs, the target identity, and the resolution result.

---

### Phase 6 — E2E Encryption Under Load (--epochs cycles)

**Goal:** verify MLS epoch management stays consistent under sustained member churn.

**Setup:** Alice and M1 share a dedicated E2E room (created at Phase 6 start).
Both upload KeyPackages. Alice initialises the MLS group.

**Step sequence:**

1. Run `--epochs` add/remove cycles. Each cycle:
   - Add M2 to the group (Alice fetches M2's KeyPackage, sends Welcome + Commit)
   - Send 10 encrypted messages (alternating Alice and M2 as sender)
   - Remove M2 from the group (Alice sends Remove Commit, epoch advances)
   - Verify M2 cannot decrypt the first message of the next cycle (wrong epoch key)

2. After all cycles, verify:
   - Final epoch number equals `--epochs` × 2 (one advance per add, one per remove)
   - All encrypted message content fields in event_trace are empty (Node never saw plaintext)
   - Total messages sent: `--epochs × 10`. All accepted by Node A.

3. Concurrent load during epoch cycling: while epoch cycles run, M1 continues sending
   unencrypted messages to a separate room in the same Space (from the existing Phase 4
   message flood). Verify the two streams do not interfere.

**Failure conditions:**
- Wrong epoch count → `EPOCH_COUNT_MISMATCH`
- M2 decrypts post-removal message → `FORWARD_SECRECY_VIOLATION` (critical failure)
- Any plaintext visible in event_trace → `PLAINTEXT_LEAK` (critical failure)
- Message delivery failure rate > 1% → `HIGH_DELIVERY_FAILURE_RATE`

**Comm_log entries:** one entry per cycle with `phase=e2e_epoch`, recording cycle number,
epoch before/after, messages sent, and forward secrecy check result.

---

### Phase 7 — Space Migration Under Traffic (single migration)

**Goal:** verify that Space migration completes correctly while messages are actively
being sent to the Space being migrated.

**Setup:** use the main Space from Phase 1. Ensure at least 5 members are active
and sending messages.

**Step sequence:**

1. Start a background message flood: 5 members each sending 1 message every 100ms
   to the Space, concurrently. Record all sent event IDs.

2. While the flood is running (after 200ms), Alice initiates migration:
   `migration.request` → `migration.propose` → `migration.accept` →
   event batch transfer → `migration.verified` → `state.space_migrate`.

3. After `state.space_migrate` is committed, redirect the message flood to Node B.

4. After migration completes, verify:
   - All events sent before migration are present in Node B's DAG
   - Events sent after migration are accepted by Node B
   - Event count on Node B ≥ event count on Node A at migration cutover

5. Verify the final DAG on Node B is internally consistent:
   - No cycles
   - All `prev_events` references resolve
   - No duplicate event IDs

**Failure conditions:**
- Pre-migration events missing from Node B → `MIGRATION_DATA_LOSS` (critical)
- Post-migration events rejected by Node B → `POST_MIGRATION_REJECTION`
- DAG inconsistency → `DAG_INTEGRITY_FAILURE` (critical)
- Migration did not complete (timed out after 30s) → `MIGRATION_TIMEOUT`

**Comm_log entries:** migration lifecycle events plus a final DAG consistency report.

---

## Recommended Run Parameters

For a thorough stress test that completes in under 3 minutes on modern hardware:

```powershell
bin\xgen-client stress-test `
  --node-a ws://127.0.0.1:8080/xgen `
  --node-b ws://127.0.0.1:8081/xgen `
  --members 50 `
  --messages 1000 `
  --phase2 `
  --conflicts 100 `
  --epochs 100 `
  --rest-ms 2000
```

Total events generated (approximate):
- Phase 1–4 (Ph1 baseline): 50 members × 1000 messages = 50,000+ events
- Phase 5 (conflicts): 100 × 2 conflicting events = 200 events
- Phase 6 (E2E): 100 cycles × 10 messages = 1,000 encrypted events + 200 epoch changes
- Phase 7 (migration under traffic): ~500 concurrent messages + migration lifecycle

**Total: approximately 52,000 events across the session.**

If the run is too slow, reduce `--messages 500 --conflicts 50 --epochs 50` for a faster
pass (~26,000 events). Do not reduce below these values for the final documented run.

---

## Output Format

After all phases complete, print a summary block:

```
════════════════════════════════════════════════════════════
STRESS-TEST-PH2 RESULTS
════════════════════════════════════════════════════════════
Members:           50
Messages/member:   1000
Conflicts:         100
E2E epochs:        100

Phase 1  Setup                     PASS   (1.2s)
Phase 2  Registration              PASS   (8.4s)
Phase 3  Federation + Join         PASS  (11.2s)
Phase 4  Message Flood             PASS  (38.7s)
Phase 5  Concurrent Conflicts      PASS   (4.1s)   100/100 correct winners
Phase 6  E2E Encryption Load       PASS  (22.3s)   0 forward-secrecy violations
Phase 7  Migration Under Traffic   PASS  (14.8s)   0 events lost
────────────────────────────────────────────────────────────
Total events sent:     52,247
Total events accepted: 52,247
Delivery failure rate: 0.00%
DAG integrity:         OK (both nodes)
State divergence:      NONE
Duration:              100.7s
════════════════════════════════════════════════════════════
```

**This full output block must be pasted verbatim into `JOURNAL.md`.** Do not paraphrase.
Do not summarise. Paste the actual numbers.

The comm_log file is saved to `logs/stress_ph2_<timestamp>.json`. Include the file path
in the journal entry. Do not embed the full comm_log in the journal — just the summary
block and the log file path.

---

## Running the Test

```powershell
# Terminal 1 — Node A
cd E:\XGen\NodeA
xgen-node

# Terminal 2 — Node B
cd E:\XGen\NodeB
xgen-node

# Terminal 3 — Run the stress test
cd E:\Projects\XGenProtocol
bin\xgen-client stress-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen --members 50 --messages 1000 --phase2 --conflicts 100 --epochs 100 --rest-ms 2000
```

---

## Verification

After the command exits 0:

1. `cargo test` — must still show 300/300 (no regressions)
2. `cargo build --release` — clean, no warnings
3. Paste full summary block into `JOURNAL.md`
4. Record comm_log file path in journal entry
5. Confirm: 0 forward-secrecy violations, 0 migration data loss, 0 state divergence

---

## Definition of Done

- [ ] `stress-test` extended: `--members` cap raised to 50, `--phase2`, `--conflicts`, `--epochs` flags added
- [ ] Phase 5 implemented: concurrent conflict generation, winner verification, divergence check
- [ ] Phase 6 implemented: E2E epoch cycling under concurrent load, forward-secrecy check
- [ ] Phase 7 implemented: migration under active message traffic, post-migration DAG integrity check
- [ ] `cargo test` — 300/300 passing, 0 warnings
- [ ] `cargo build --release` — clean
- [ ] Stress test exits 0 with recommended parameters (50 members, 1000 msg, 100 conflicts, 100 epochs)
- [ ] Full summary block pasted verbatim into `JOURNAL.md`
- [ ] comm_log file path recorded in `JOURNAL.md`
- [ ] `CLAUDE.md` updated: stress test marked COMPLETE with journal reference
- [ ] Committed and pushed
