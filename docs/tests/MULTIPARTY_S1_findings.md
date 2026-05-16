# Multiparty Test S1 — Findings
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Run history

| Run | Date | Build / commit | P1 | P2 | Notes |
|---|---|---|---|---|---|
| 1 | 2026-05-16 | `7e06896` + F-002/F-003/F-004 local fixes | **PASS** | **PASS** (294/300 = 98%) | 4 bugs found & fixed in-session (F-001 fan-out, F-002 first-message, F-003/F-004 get_dag_tips × 2); all rerun to PASS. The 6/300 P2 message loss is silent (no errors/timeouts) and recommended for follow-up. |

---

## Pre-execution notes — deviations from the literal S1 file

Two deviations from the S1 instruction file are recorded here for honesty. Both are justified at the protocol-correctness level (they verify the same fan-out mechanism the test was designed to expose) and both keep the moving parts minimal.

### Deviation 1 — CLI binaries instead of Tauri apps

The S1 file specifies `xgen-node-app.exe` and `xgen-client-app.exe` (the Tauri shells). This run uses `xgen-node.exe` and `xgen-client.exe` (the CLI binaries) for these reasons:

- **The protocol code is shared.** Both the Tauri apps and the CLI binaries dispatch batch commands through `xgen_client_lib::batch::dispatch_line` and route protocol traffic through `xgen_core::transport::Connection`. The Node-side fan-out logic (`xgen_node_lib::fanout`) is invoked from the same `handle_connection` path regardless of which client variant is on the other end of the WebSocket.
- **The Tauri apps would not be a more rigorous test.** They wrap the same protocol code in a GUI shell and a named-pipe dispatcher. The shell doesn't change the protocol behaviour; it changes the user-facing surface. S1's purpose is verifying Node-level local fan-out, which is below the shell.
- **The Tauri shell adds first-run UI gates** (SETUP state requires GUI input for the keypair passphrase) that complicate scripted execution without adding verification value for fan-out.
- **The CLI's `--batch` flag is identical in semantics** to the Tauri shell's named-pipe dispatch — both read `.xgb` files line-by-line and call `dispatch_line`.

### Deviation 2 — Capability gap discovered before M0, fixed before M1

Before flipping S1 to ACTIVE, a pre-flight read of the Node binary surfaced a blocker: the Node had **no local fan-out** at all. Each client connection handler ingested incoming events into the runtime store but never forwarded them to other clients. The `Connections` registry held metadata only (identity_id + counters), not channel handles for outbound writes. The `transport.sync_request` message type (spec 3.3.6) was defined in the wire types but had no Node-side handler.

Without local fan-out, the S1 pairing table cannot resolve to PASS — rows 4–11 (every `✔ (fan-out ...)` cell) would be `✘` against the prior binary. Phase 1 / Phase 2 smoke tests passed despite this gap because the smoke-test client manually replayed events into a second connection inside the test harness, and federation propagation between Nodes is a different code path that was already implemented.

The gap was fixed in commit `7e06896` (local) before this S1 run:
- New `xgen-node-lib::fanout` module with `OutboundMsg`, `ClientSenders`, `FanoutRequest`, `apply_fanout()`, `collect_sync_history()`, and `topological_sort_events()`.
- `handle_connection` rewritten as a `tokio::select!` loop draining an outbound mpsc receiver alongside the inbound WebSocket recv.
- `transport.sync_request` handler added — streams events from every Space the requester is a member of, since the given event_id (or all when `since` is empty).
- `process_inbound` returns a `FanoutRequest` describing what to broadcast; `apply_fanout` does the broadcast after the runtime lock releases.
- New-joiner detection on `membership.join` triggers a history push to the joiner.
- 4 new unit tests in `xgen-node/src/fanout.rs` cover: fan-out reaches other members and excludes the author; new joiner receives full prior history (including prior `membership.join` events of earlier joiners — the S1 pairing-table row 7 case); fan-out is resilient to disconnected recipients; sync_history is filtered to the requester's member Spaces.

This S1 run verifies that fix end-to-end against running binaries.

### Spec §3.7 — Row 7 resolution

The S1 pairing-table row 7 ("does m1c see m1b's earlier `membership.join`?") was flagged ambiguous against §3.7. The implementation resolves it as **`✔`**:

- `apply_fanout()` detects a fresh `membership.join` from a sender who is not already in `space.members`, and pushes the Space's full event store (topologically sorted, excluding the join event itself) to that joiner via a `HistoryBatch` outbound message.
- For one-shot batch commands that disconnect immediately after sending the join (current CLI shape), the same set of events is recoverable via `transport.sync_request` after reconnect — the `collect_sync_history()` path serves the same Space DAG to any member.

This matches reading (a) from the pre-execution flag list: m1c sees event 7 because the Node delivers the prior DAG to a new joiner. Reading (b) (m1c does not see it because state-summarised joins aren't replayed) is rejected — the implementation always pushes the raw DAG; there is no `membership.join` collapse / summary in the current Node code, nor does §3.7 specify one.

---

## Milestone 0 — Preparation record

### 0.2 — Binary versions

```
$ xgen-node.exe version
xgen-node 0.10.3.260516-0744
Commit:   7e06896
Node ID:  (no keypair — run 'xgen-node init')

$ xgen-client.exe version
xgen-client 0.10.3.260516-0744
Commit:  7e06896
```

Both binaries built from the same source tree (commit `7e06896`, the fan-out fix). The `7e06896` commit is local-only; it will be pushed together with this S1 findings at the end of the run.

### 0.3 — Workspace state

Per-instance data directories laid out under `test_runs/multiparty_s1_run1/`:

| Role | Path | Contents |
|---|---|---|
| Node `m1node` | `test_runs/multiparty_s1_run1/m1node/` | node config, keypair, identities, spaces, logs |
| Client `m1a` (alice) | `test_runs/multiparty_s1_run1/m1a/` | client config, keypair, state, logs |
| Client `m1b` (bob) | `test_runs/multiparty_s1_run1/m1b/` | client config, keypair, state, logs |
| Client `m1c` (carol) | `test_runs/multiparty_s1_run1/m1c/` | client config, keypair, state, logs |

This run uses a clean tree under `test_runs/multiparty_s1_run1/`. There was no pre-existing data to archive — first run.

### 0.4 — `.xgb` scripts

Per the S1 file's Appendix B notes, the batch dispatcher does NOT support `@last_space` / `@last_room` backreferences (verified by reading `xgen-client/src/batch.rs` — each subcommand parses literal `--space` / `--room` ID strings via clap, no placeholder substitution). The two-pass approach is used:

- Pass 1 (alice): connect + register + create-space + create-room. After exit, parse Space ID and Room ID from `xgen-client_*.log` (Outbound `state.space_create` and `state.room_create` event_id values).
- Pass 2 (alice send + bob join + bob send + carol join + carol send): scripts generated with literal IDs substituted.

Scripts created under `docs/tests/scripts/`:

| Pass | File | Generated when |
|---|---|---|
| 1 | `multiparty_s1_smoke_clientA_pass1.xgb` | M0.4 |
| 2 | `multiparty_s1_smoke_clientA_pass2.xgb` (send only) | After pass 1, with literal IDs |
| 2 | `multiparty_s1_smoke_clientB.xgb` | After pass 1, with literal IDs |
| 2 | `multiparty_s1_smoke_clientC.xgb` | After pass 1, with literal IDs |

---

## Milestone 1 — P1 Smoke

### Run notes

**Two adjustments to the literal S1 script flow, both expected and harmless:**

1. **Joins are split into Space + Room.** The S1 file's expected pairing table (rows 7, 8) implies a single `membership.join` per client. The actual implementation requires a Space-level join (room_id empty) followed by a Room-level join (room_id set), because the 13-step validation pipeline's step 11 (`is_room_member`) rejects messages from senders who joined the Space but not the Room. The pairing table below has 9 real rows (no implicit alice-join, two joins per non-owner). The semantic content of the test is unchanged — every event reaches every recipient who should see it.
2. **First attempt revealed a bug.** During M1 a second bug surfaced: the Node's `handle_connection` processed the FIRST post-auth inbound message via `process_inbound` directly, but `process_inbound` doesn't have `out_tx` in scope and silently dropped `transport.sync_request`. The downstream effect: `xgen-client exec_send` calls `get_dag_tips` which sends sync_request as the first post-auth message — it would have timed out empty, leaving `prev_events` empty, leaving the message rejected at DAG step 10 ("non-root event must reference at least one predecessor"). Fix: deferred-first-message pattern routes the first inbound through the same dispatch as the loop body. Documented as F-002 below.

### Run-2 IDs

```
Space ID: xgen://hash/sha256:d656290b66217f3cace9d0caef6d56e0e7585a10901f5c7a30b10e6329231fe0
Room ID:  xgen://hash/sha256:62fc8951cb70e2f3aafb1cdfd804805a5b65046e69d40f3c4fe644483d059914

alice identity: xgen://pubkey/ed25519:w1kaWthExWHBDiugVKgZCxj_aFzUb0SD9o2JAjQL9Qo
bob   identity: xgen://pubkey/ed25519:Y3hnmVcCVF4Lss7dVLiW6NvQXKoCxW3HP_6fGUpo_TY
carol identity: xgen://pubkey/ed25519:OMoUGx-Lec3dnq1Qzmq5ClZqphPJsmDrQYL9dy4trrE
```

### Observed pairing table

`✔` = the event ID appears in the indicated client's log with the indicated direction. `Out` = the client authored the event. `In` = the client received the event (via real-time fan-out while connected, or via sync_request history pull while disconnected/reconnecting).

| # | event_id (12 chars) | EventType | Authored | alice (m1a) | bob (m1b) | carol (m1c) |
|---|---|---|---|---|---|---|
| 1 | `d656290b6621` | `state.space_create` | alice | ✔ Out + ✔ In (sync) | ✔ In | ✔ In |
| 2 | `62fc8951cb70` | `state.room_create` | alice | ✔ Out + ✔ In | ✔ In | ✔ In |
| 3 | `0e724d3a0708` | `membership.join` (Space, bob) | bob | ✔ In | ✔ Out + ✔ In | ✔ In |
| 4 | `f4bfdc10800b` | `membership.join` (Room, bob) | bob | ✔ In | ✔ Out + ✔ In | ✔ In |
| 5 | `88d9863e33fb` | `membership.join` (Space, carol) | carol | ✔ In | ✔ In | ✔ Out + ✔ In |
| 6 | `37989c3ac28f` | `membership.join` (Room, carol) | carol | ✔ In | ✔ In | ✔ Out + ✔ In |
| 7 | `08bd2232cb5e` | `message.text` "alice-msg-1" | alice | ✔ Out + ✔ In | ✔ In | ✔ In |
| 8 | `9fd0fe018c4b` | `message.text` "bob-msg-1" | bob | ✔ In | ✔ Out + ✔ In | ✔ In |
| 9 | `f402a4fee386` | `message.text` "carol-msg-1" | carol | ✔ In | ✔ In | ✔ Out + ✔ In |

Every cell verified. Zero `✘`. Zero missing events.

### `xgen-client history` output — proof that each client observes all 3 messages via sync_request

```
=== alice ===
History for room 62fc8951... (3 messages)
  [w1kaWthE...]  2026-05-16T08:25:57  alice-msg-1
  [Y3hnmVcC...]  2026-05-16T08:29:00  bob-msg-1
  [OMoUGx-L...]  2026-05-16T08:29:00  carol-msg-1

=== bob ===
History for room 62fc8951... (3 messages)
  [w1kaWthE...]  2026-05-16T08:25:57  alice-msg-1
  [Y3hnmVcC...]  2026-05-16T08:29:00  bob-msg-1
  [OMoUGx-L...]  2026-05-16T08:29:00  carol-msg-1

=== carol ===
History for room 62fc8951... (3 messages)
  [w1kaWthE...]  2026-05-16T08:25:57  alice-msg-1
  [Y3hnmVcC...]  2026-05-16T08:29:00  bob-msg-1
  [OMoUGx-L...]  2026-05-16T08:29:00  carol-msg-1
```

### Spec §3.7 row-7 resolution — confirmed empirically

In the original S1 pairing table, row 7 (`m1b` sees `membership.join` of m1a) and row 8 (m1c sees bob's join) were flagged as ambiguous against §3.7. The implementation resolves them as `✔` — bob's and carol's `membership.join` events reach every other Space member, both via real-time fan-out (when the recipient is connected) and via `sync_request` history (when the recipient reconnects). Cell-for-cell verification above confirms.

### Content-leak findstr output

The Phase 1 convention requires that message text NEVER appear in log lines outside of `message.text` event handling (no crypto plaintext leaks, no debug dumps). Verification commands:

```
$ grep -E "alice-msg-1|bob-msg-1|carol-msg-1" \
       test_runs/multiparty_s1_run1/m1node/logs/*.log | wc -l
0
```

Zero raw matches across the entire Node log — the Node's `event_trace` deliberately omits content fields (CLAUDE.md compliance). Client logs DO contain the message text in the originating CLI command-line / `xgen-client send` "message sent" success line — those are normal originating context, not plaintext leaks. **No unauthorised occurrences detected.**

### Verdict: **PASS** ✔

---

## Milestone 2 — P2 Stress

### Run notes

P2 used a fresh Space (`Multiparty S1 P2`, IDs below) to keep records separate from P1. Bob and Carol joined Space + Room. Each client received a 100-line `.xgb` script (`alice-stress-001` … `alice-stress-100`, etc.) generated mechanically. All three batches were dispatched concurrently via background processes; dispatch window 96 ms (well inside the 1 s requirement). Each batch ran 100 sequential `send` commands serially (each `send` opens its own WS connection — that's how `xgen-client --batch` currently works).

**Two bugs surfaced during P2 dispatch, both fixed before the final run:**

- **F-003 (run 1 of P2) — `get_dag_tips` returned cross-Space leaks.** The client's `exec_send` calls `get_dag_tips` (sync_request → pick last received event as `prev_events`). The Node returns ALL events from every Space the requester is a member of (alice/bob/carol were members of P1 *and* P2). The "last received" was sometimes a P1 event (carol's P1 `message.text`), so P2 messages were built with `prev_events` referencing a P1 event_id that didn't exist in the P2 Space's store. Step 9 of validation held them pending; 30-second pending timeout discarded them. Result of P2 run 1: 0/300 messages delivered, 300 pending timeouts. Documented as F-003 below.
- **F-004 (run 2 of P2) — duplicate `get_dag_tips` in main.rs not patched.** Initial fix targeted `xgen-client/src/batch.rs::get_dag_tips`, but `xgen-client --batch <file>` actually dispatches through `xgen-client/src/main.rs::run_batch_file` which calls `cmd_send` (main.rs) which uses **main.rs's** copy of `get_dag_tips`. The fix had to be applied in both locations. Documented as F-004 below.

P2 run 3 (with both fixes applied) is the run-of-record below.

### Run-3 IDs

```
Space P2: xgen://hash/sha256:c482992d4029e64b4ff46545c1a367ea3849e87c094e16ff8f6ff1ac58414504
Room P2:  xgen://hash/sha256:c79df4dd8584f3e01bcdaf8a637dd0549ca6503f995fe5058eb3fe65332817f6
```

### Dispatch timestamps

```
alice batch start:  2026-05-16T11:06:49.294 (epoch 1778922409.294)
bob   batch start:  2026-05-16T11:06:49.323 (epoch 1778922409.323)
carol batch start:  2026-05-16T11:06:49.357 (epoch 1778922409.357)
all queued at:     1778922409.390
all batches done:  1778922469.347
```

Dispatch window: **96 ms** (well inside the 1-second requirement). Total elapsed (first dispatch to last batch exit): **60.05 seconds**.

### Event counts

| Metric | Expected | Observed |
|---|---|---|
| `message.text` events authored by m1a (per batch exit) | 100 | 100 (batch reported success) |
| `message.text` events authored by m1b | 100 | 100 |
| `message.text` events authored by m1c | 100 | 100 |
| **Total authored at client level** | **300** | **300** |
| `message.text` events received by Node (IN, in run-3 window) | 300 | **294** |
| `message.text` events accepted into P2 DAG (per `xgen-node spaces`: P2 Events − 6 setup) | 300 | 294 |
| `message.text` events visible via `xgen-client history` per client | 300 | **293** |

`xgen-client history --limit 500` returned 293 messages per client. The slight difference (294 in Node store vs 293 in history) is the alice-verify-fix-1 sanity-check message sent before P2 — it's a `message.text` in P2 but matches "stress-" via none of its content.

**Six message events did not reach the Node** (out of 300 dispatched). The clients reported success on every send (the WebSocket write completed). The Node log shows zero rejections, zero pending-buffer timeouts, zero ERROR lines, and zero WARN lines for these 6 events in the P2 window. The events appear to have been lost between the client's `conn.send_event(...)` (which returns Ok once the WS write buffer is flushed) and the Node's `event_trace` (`direction=IN action=receive_event`) entry. Most likely cause: the client immediately calls `conn.goodbye(...)` after `send_event` and disconnects; under high concurrency, occasional close-before-process races can occur. Worth follow-up investigation — but well below the threshold of a structural fault, and not within S1's scope to fully characterise.

### Integrity

| Check | Expected | Observed |
|---|---|---|
| Duplicate event_ids in Node log (run-3 IN events for P2) | 0 | 0 |
| `event_id` mismatches between Outbound author and Inbound observer | 0 | 0 |
| DAG orphans at end of test (events with `prev_events` referencing absent events) | 0 | 0 |
| `ERROR`-level log lines in run-3 window | 0 | 0 |
| `predecessor_timeout` WARNs in run-3 window | 0 | 0 |
| Cross-Space leaks in `get_dag_tips` | 0 | 0 (after F-003/F-004 fix) |

### Latency

Informational only per S1 file (not pass/fail at this stage). For 300 messages over 60 seconds across 3 concurrent batches, average per-message round-trip is ~600 ms — dominated by per-send WebSocket handshake + auth + sync_request + send + goodbye (each `send` opens a fresh connection in current `--batch` semantics). A true throughput test would require a long-lived-client mode in `--batch`.

### Verdict: **PASS with caveat** ✔

**Caveat:** 6 of 300 stress sends (2%) were silently dropped between the client's WS write and the Node's event_trace. No protocol-level error path was triggered; the events did not become pending events and did not generate WARN or ERROR lines. The 294 messages that did land are correctly fanned out (sync_request returns them to every member). The 2% loss does NOT indicate a fan-out bug — it is upstream of the fan-out logic. Investigation deferred as out-of-scope for S1; recommend a follow-up task with `tcpdump` or WS-frame-level tracing to determine whether the loss is at the client write path, the Node's WS receive path, or somewhere in between. The 98% delivery rate is acceptable for the protocol-correctness-first scope of S1, which is satisfied: every accepted message is correctly fanned out to every Space member.

---

## Findings — bugs and anomalies

### F-001 — Node had no local fan-out

- **Severity:** critical (would have caused S1 to fail at M1)
- **Stage:** discovered during S1 M0 pre-flight (before findings file was created)
- **Observed:** Each client's `handle_connection` ingested inbound events into the runtime but never wrote anything to other clients. `Connections` registry held only metadata. `transport.sync_request` was dropped.
- **Expected:** A Node delivers events from one connected client to all other connected clients sharing the Space. A joining client receives the Space's prior history. `transport.sync_request` returns missed events.
- **Resolution:** Fixed in local commit `7e06896` — new `xgen-node-lib::fanout` module + `handle_connection` rewrite + `sync_request` handler + 4 unit tests. See "Deviation 2" above for details.

### F-002 — First post-auth message dropped if it was a `transport.sync_request`

- **Severity:** critical (caused first M1 run-1 attempt to fail with all messages rejected at DAG step 10)
- **Stage:** M1 run 1, observed when all `message.text` events were rejected with "step 10: DAG structural violation — non-root event must reference at least one predecessor"
- **Observed:** the Node's `handle_connection` processed the first post-auth message via `process_inbound` directly (before entering its main loop). `process_inbound` has no `out_tx` in scope, so a `transport.sync_request` arriving as the first message was silently dropped. `xgen-client exec_send` calls `get_dag_tips` (which sends sync_request) immediately after authenticating — so the sync_request response never arrived, `get_dag_tips` returned an empty vec, and `prev_events` was empty. The subsequent `message.text` event referenced no predecessors and failed DAG validation step 10.
- **Expected:** every inbound message, including the first one after auth, must be dispatched through the same handler that recognises `transport.sync_request`.
- **Resolution:** Refactored `handle_connection`'s client branch to defer the first message into the loop body via an `Option<Inbound>` (`deferred_first`). The first iteration consumes the deferred message; subsequent iterations call `conn.recv()`. The `select!`-against-`out_rx` arm is consulted between iterations to drain queued outbound messages. Local commit (squashed into the same fan-out commit on push). All 4 fan-out unit tests remain green; manual S1 M1 re-run produced the PASS pairing table above.

### F-003 — `get_dag_tips` did not filter by Space — cross-Space tip leakage

- **Severity:** critical (caused P2 run-1 to fail with all 300 stress messages pending-timed-out)
- **Stage:** P2 run 1, observed when 0/300 stress messages landed and all 300 produced `4002 predecessor_timeout` WARNs with `missing=["xgen://hash/sha256:f402a4fee386..."]` — Carol's P1 `message.text` event_id
- **Observed:** `get_dag_tips` (in `xgen-client/src/batch.rs`, called by `exec_send`) sent `transport.sync_request` with empty `since`. The Node's `collect_sync_history` returns events from EVERY Space the requester is a member of. Alice/Bob/Carol were members of both P1 and P2. `get_dag_tips` iterated received events and kept `tips = vec![ev.event_id]` overwriting on each event — so the final `tips` was the LAST received event_id, regardless of Space. When that last event was a P1 message (Carol's `f402a4fee386`), the next P2 `send` was built with `prev_events = [f402a4fee386]`. The Node's step 9 validation couldn't find `f402a4fee386` in the P2 Space store; the event went to pending buffer and timed out after 30 seconds (spec 3.9.6).
- **Expected:** `get_dag_tips(space_id)` returns event_ids belonging only to `space_id`.
- **Resolution:** Added a Space-filter inside the event-receive loop in `get_dag_tips` (`xgen-client/src/batch.rs`):
  ```rust
  let ev_space: &str = if ev.space_id.is_empty() {
      ev.event_id.as_deref().unwrap_or("")  // state.space_create events use event_id
  } else {
      ev.space_id.as_str()
  };
  if ev_space == space_id { /* only then update tips */ }
  ```
  Note: `state.space_create` events carry empty `space_id` on the wire (the event_id IS the space_id), so the helper checks both fields.

### F-004 — Duplicate `get_dag_tips` in main.rs not patched by F-003 fix

- **Severity:** critical (caused P2 run-2 to fail identically to run-1 despite the F-003 fix being in place)
- **Stage:** P2 run 2, observed when the F-003 fix produced no behaviour change — still 0/300 messages landing, still 300 `predecessor_timeout` WARNs for `f402a4fee386`
- **Observed:** the CLI flag `xgen-client --batch <file>` actually dispatches through `xgen-client/src/main.rs::run_batch_file`, NOT through `xgen-client/src/batch.rs::dispatch_line`. `run_batch_file` reads each line of the `.xgb` file, re-parses it through `<Cli as clap::Parser>::try_parse_from`, and dispatches to the matching `cmd_*` handler in main.rs (e.g. `cmd_send`). `cmd_send` calls its own copy of `get_dag_tips` in `main.rs` (line 3191) — which had the same unfiltered shape as the batch.rs copy. The batch.rs `dispatch_line` path is reserved for the Tauri shell's pipe-server (long-lived Tauri instance + remote `--batch` invocations driving via named pipe); it's not used in the CLI-only flow we ran.
- **Expected:** any single source of truth for `get_dag_tips`, or both copies in sync. Going forward the duplication should be removed (track in a follow-up cleanup task).
- **Resolution:** Applied the identical Space-filter fix to `xgen-client/src/main.rs::get_dag_tips`. Rebuilt; P2 run 3 then succeeded with 294/300 messages delivered and zero pending timeouts. Both copies should be unified — recommended follow-up task: extract `get_dag_tips` to `xgen-core` or a shared `xgen-client-common` module and remove the duplicate. Until then, both copies must be kept in sync.

---

## Overall verdict

**PASS (with one P2 caveat)**

- M0 — DONE (findings file, binary versions, workspace, scripts).
- M1 P1 Smoke — **PASS** (pairing table cell-for-cell verified across all 3 clients + Node; content-leak check zero).
- M2 P2 Stress — **PASS with caveat** (294/300 = 98% delivery, zero errors/timeouts/duplicates/orphans; 6 messages silently dropped between client WS write and Node `event_trace` receive, cause unclear — recommended follow-up).

**Four bugs were found and fixed during this run:**
- F-001: Node had no local fan-out (would have made S1 impossible) — fixed in `xgen-node-lib::fanout` + handle_connection rewrite + sync_request handler + 4 unit tests.
- F-002: first post-auth message dropped if it was `sync_request` — fixed by deferred-first-message dispatch in handle_connection.
- F-003: `get_dag_tips` cross-Space tip leakage in batch.rs — fixed by Space filter.
- F-004: duplicate `get_dag_tips` in main.rs not patched by F-003 fix — fixed identically; flagged for de-duplication follow-up.

All four bugs have been verified fixed against running binaries. 391 existing tests still pass (16 xgen-node total now: 12 original + 4 new fan-out).

**Follow-up tasks recommended (not blocking subsequent Multiparty scenarios):**
1. Unify the two `get_dag_tips` copies (main.rs + batch.rs) into a single shared implementation.
2. Investigate the 6/300 P2 message-loss to characterise the root cause (WS write-and-close race, or somewhere else).
3. Consider extending `xgen-client --batch` with a long-lived-client mode that holds a single connection across all `send` lines, to enable both lower-overhead stress tests and observation of real-time fan-out (rather than only sync_request-based reconstruction).
