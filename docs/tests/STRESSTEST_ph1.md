# XGen Protocol — Phase 1 Stress Test
> **Status:** COMPLETED  
> **Last updated:** 2026-05-06  
> Document type: Implementation instructions for Claude Code  
> Applies to: `xgen-client/src/main.rs`  
> Date: May 2026
> Prepared by: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> Decision record: D-033, D-038
> See also: `docs/tests/SMOKETEST_ph1.md` — Phase 1 smoke test (functional baseline)
> See also: `docs/xgen_appendix_g_en.md` — Appendix G: Log Line Convention

---

## Purpose

The smoke test verifies protocol correctness in a controlled 2-identity, 17-step sequence. The stress test verifies behaviour under concurrent load — multiple identities sending simultaneously across two federated nodes. It exposes race conditions in DAG tip resolution, connection handling, and federation propagation that sequential testing cannot find.

The verification artifact is the log files. No automated DAG integrity check is performed — that requires query infrastructure not yet available in Phase 1.

---

## CLI

Add a `StressTest` subcommand to `xgen-client` alongside `SmokeTest`:

```
xgen-client stress-test --node-a <url> --node-b <url> [--members <n>] [--messages <n>]
```

**Arguments:**

| Argument | Default | Description |
|---|---|---|
| `--node-a` | required | WebSocket endpoint of Node A |
| `--node-b` | required | WebSocket endpoint of Node B |
| `--members` | `10` | Total number of test identities (minimum 2, maximum 20) |
| `--messages` | `50` | Messages per identity in the message phase |

Rooms are fixed at 3 (`general`, `random`, `tech`) — not configurable.

---

## Test Structure

The test runs in four sequential phases. Concurrency is introduced only in phases 3 and 4.

### Phase 1 — Setup (sequential)

Alice (member 0) performs all structural operations in order:

1. Connect to Node A, authenticate with ephemeral keypair
2. Register identity with display name `"StressTest-Alice"`
3. Create Space — name `"StressTest Space"`
4. Create Room `general`
5. Create Room `random`
6. Create Room `tech`
7. For each other member (1 through N-1): send `membership.invite` with role `member`
8. Disconnect

All other members generate their ephemeral keypairs during this phase but do not connect yet.

**Member distribution:** first half register on Node A, second half on Node B. With default 10 members: members 0–4 on Node A, members 5–9 on Node B. Alice (member 0) is always on Node A.

### Phase 2 — Registration (sequential)

Each member (1 through N-1) registers on their assigned node in order. Sequential to avoid overwhelming the node with simultaneous registration requests before federation is established.

Each member registers with display name `"StressTest-M{n}"` where `n` is the member index.

### Phase 3 — Federation + Join (concurrent)

Two things happen concurrently:

**3a — Federation:** Alice connects to Node A and runs the federation handshake to bring Node B into the Space. This is the same federation sequence as the smoke test (Steps 8–11).

**3b — Joins:** All members (1 through N-1) join the Space and all 3 Rooms concurrently via `tokio::spawn`. Each member:
1. Connects to their assigned node
2. Authenticates
3. Sends `membership.join` for the Space
4. Sends `membership.join` for Room `general`
5. Sends `membership.join` for Room `random`
6. Sends `membership.join` for Room `tech`
7. Records their last `event_id` (the Room tech join) as their DAG anchor

Wait for all join tasks and the federation task to complete before proceeding to Phase 4. Use `tokio::join!` or `futures::join_all`.

**Failure policy:** if any join task fails, log the error and continue — do not abort the test. Report failures in the summary.

### Phase 4 — Message Flood (concurrent)

All N members send messages concurrently. Each member runs in its own `tokio::spawn` task:

```
for i in 0..messages {
    let room = pick_room(i);           // deterministic rotation: general → random → tech → general ...
    let text = format!("M{member_index} msg {i}");
    build and sign message.text event with prev_events = [last_event_id]
    send to assigned node
    last_event_id = sent event_id
    sleep random 0–50ms jitter
}
```

**Room rotation:** deterministic round-robin (`i % 3`) — not random. This ensures even distribution across rooms and makes the expected event count per room calculable for verification.

**`prev_events`:** each sender tracks only its own chain. Use the sender's last sent `event_id` as the sole `prev_events` entry. Cross-identity DAG merges happen at the node level.

**Jitter:** `tokio::time::sleep(Duration::from_millis(rand::random::<u64>() % 50))` before each send. This spreads the load and produces a realistic DAG shape rather than a pure burst.

**Error handling:** on send failure, log the error, increment the error counter, and continue with the next message. Do not abort the task.

Wait for all message tasks to complete.

---

## Progress Output

Print a live progress line to stdout during the message phase. Update every 5 seconds:

```
  [stress] 142 / 500 events sent  (4 errors)  elapsed: 14s
```

At phase boundaries, print a header line:

```
Phase 1 — Setup ...
Phase 2 — Registration ...
Phase 3 — Federation + Join ...
Phase 4 — Message flood (500 events across 10 members) ...
```

---

## Summary Report

After all phases complete, print the following to stdout:

```
============================================
STRESS TEST COMPLETE
============================================

Configuration
  Nodes:        Node A (ws://...) + Node B (ws://...)
  Members:      10  (5 on Node A, 5 on Node B)
  Rooms:        3  (general, random, tech)
  Messages:     50 per member
  Total events: ~535  (setup + joins + messages)

Results
  Messages sent:    498 / 500
  Send errors:      2
  Join failures:    0
  Elapsed:          38.4s
  Throughput:       13.0 events/sec

Expected event distribution (messages only)
  Room general:   ~167  (members × messages ÷ 3)
  Room random:    ~167
  Room tech:      ~166

Log files — review for verification
  Node A:  test/node_a/logs/xgen-node_YYYY-MM-DD_HH-MM-SS.log
  Node B:  test/node_b/logs/xgen-node_YYYY-MM-DD_HH-MM-SS.log
  Client:  <exe dir>/logs/xgen-client_YYYY-MM-DD_HH-MM-SS.log

Verification checklist (manual — inspect log files)
  [ ] No send errors in Node logs (no ERROR lines for valid events)
  [ ] No content leaks — message text must not appear in any log file
  [ ] Session footer present in all log files (clean shutdown)
  [ ] direction=IN entries on Node A for members 0-4 outbound events
  [ ] direction=IN entries on Node B for members 5-9 outbound events
  [ ] Federation propagation — Node B logs show events originated on Node A
  [ ] Node A logs show events originated on Node B

Test outcome: PASS / PARTIAL / FAIL
  (PASS = zero errors, all tasks completed)
  (PARTIAL = some send errors, all tasks completed)
  (FAIL = one or more tasks panicked or could not connect)
============================================
```

Compute `Total events` as: 1 (space_create) + 3 (room_create) + (N-1) invites + (N-1)×4 joins (space + 3 rooms) + N×messages.

---

## Content Leak Check

After the test completes, before printing the summary, search all log files for any message text matching the pattern `"M\d+ msg \d+"`. If any match is found, print a prominent warning:

```
WARNING: CONTENT LEAK DETECTED — message text found in log files.
This is a critical bug. Do not use these logs for verification.
```

Implement this as a post-test scan of the client log file only (the client log is in the same process — scan it directly by reading the file). Node log scanning is left to manual verification.

---

## Implementation Notes

### Keypair generation

Generate all N keypairs at the start of `cmd_stress_test` before Phase 1 begins. Store them in a `Vec<SigningKey>`. Each member's index in the Vec is their permanent identity index for the duration of the test.

### Connection model

Each member opens a new connection per phase operation — same pattern as the smoke test and all existing commands. Do not attempt persistent connections or connection pooling.

### Node assignment

```rust
fn assigned_node<'a>(member_index: usize, total: usize, node_a: &'a str, node_b: &'a str) -> &'a str {
    if member_index < total / 2 { node_a } else { node_b }
}
```

Alice (index 0) is always on Node A regardless of `total`.

### Space ID and Room IDs

Alice produces these in Phase 1. They must be passed to all subsequent phases. Use shared `Arc<String>` values or pass them directly into spawned tasks via move closures.

### DAG anchor initialisation

Each member's initial DAG anchor (before sending any messages) is their last join event_id from Phase 3. If a member's join failed, use the space_id as the fallback anchor — same approach as the existing commands.

### Timing

Record `Instant::now()` at the start of Phase 4 and at completion. Compute elapsed and throughput for the summary.

### `rand` usage

The stress test uses `rand::random::<u64>() % 50` for jitter. This is already a dependency — no new crates needed.

---

## Test Environment

Same as smoke test:

| Component | Location | Port |
|---|---|---|
| Node A | `test/node_a/` | `ws://127.0.0.1:8080/xgen` |
| Node B | `test/node_b/` | `ws://127.0.0.1:8081/xgen` |
| Binaries | `bin/xgen-node.exe`, `bin/xgen-client.exe` | — |

Both nodes must be running with `level = "debug"` in their configs before the test is started. The stress test does not start or stop nodes — that is the operator's responsibility.

**Run from `bin/` directory:**
```
xgen-client stress-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen
```

With custom parameters:
```
xgen-client stress-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen --members 20 --messages 100
```

---

## Files Modified

| File | Change |
|---|---|
| `xgen-client/src/main.rs` | Add `StressTest` subcommand, `StressTestArgs`, `cmd_stress_test()` |

No other files require modification.

---

## Relationship to Smoke Test

The stress test does not replace the smoke test. They serve different purposes:

| | Smoke test | Stress test |
|---|---|---|
| Purpose | Protocol correctness | Behaviour under load |
| Identities | 2 (ephemeral) | 10 (ephemeral, configurable) |
| Events | ~17 (sequential) | ~535 (concurrent) |
| Verification | Automated pairing report | Manual log review |
| Run frequency | Every significant code change | Before milestones and releases |

---

*End of document*
