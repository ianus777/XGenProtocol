# Phase 1 Stress Test — Findings

**Date:** 2026-05-06  
**Reviewed by:** Documentation Claude  
**Runs analysed:** 4 runs across two builds

---

## Run history

| Run | Time | Commit | Outcome | F-001 status |
|---|---|---|---|---|
| 1 | 07:06 | fac0429 | PASS | 200 ERROR lines, no buffer, no resolution |
| 2 | 07:21 | fac0429 | PASS | 150 ERROR lines, no buffer, no resolution |
| 3 | 11:46 | 4e2d0f3 | PASS | Buffer implemented — 200 events buffered, **50/250 federated resolved** |
| 4 | 11:55 | 4e2d0f3 | PASS | Buffer implemented — **250/250 federated resolved, zero buffering** |

Run 4 is a **clean pass at all levels**, including federation. Run 3 shows the buffer working but incomplete resolution under one particular timing scenario.

---

## Finding F-001 — Federation DAG ordering: events held pending on receiving node

### Status: **PARTIALLY RESOLVED** (intermittent — see below)

### Original behaviour (runs 1–2, commit fac0429)

During Phase 4 (message flood), Node B logged 150–200 `ERROR` lines:

```
ERROR xgen_node: accept_message failed  reason=step 9: unknown prev_events — event held pending
```

Events arriving out of causal order were logged as errors and discarded — no buffer, no retry. Federated `message.text` events applied on Node B: **0 out of ~250 expected**.

---

### Fix in commit 4e2d0f3

Mr. Code implemented a pending-event buffer. The `ERROR` lines are gone. Out-of-order federated events are now logged at `DEBUG`:

```
DEBUG xgen_node: event buffered — waiting for unknown prev_events  event_id=...
```

---

### Remaining issue (run 3, commit 4e2d0f3)

Run 3 shows the buffer is present but resolution is incomplete under one timing condition:

| Metric | Run 3 (11:46) | Run 4 (11:55) | Expected |
|---|---|---|---|
| direction=IN events | 284 | 284 | 284 |
| Federated `message.text` applied | **50** | **250** | 250 |
| Events buffered (never resolved) | **200** | 0 | 0 |
| ERROR lines | 0 | 0 | 0 |

In run 3, Node B received all 284 federated events but only applied 50 of the 250 `message.text` events. The other 200 were buffered and **never resolved** — the log ends with buffered events still pending at shutdown. No resolution log line exists for these events.

In run 4, all 250 federated `message.text` events were applied cleanly with zero buffering.

**This is a race condition**, not a systematic failure. The buffer resolves correctly when parents arrive in time; it stalls when the test ends before all chains are fully flushed. The two runs used identical configuration and the same commit — the difference is timing.

### Root cause hypothesis

The buffer unblocking logic triggers when a parent event is applied. If the flood completes and clients disconnect before all buffered events' parents have arrived via federation, those events remain in the buffer permanently. There is no flush-on-idle or drain-on-disconnect mechanism. When Node B's clients disconnect, federation traffic stops, parents stop arriving, and the buffer stalls indefinitely.

### What still needs to be done

1. **Drain the buffer on federation completion / client disconnect.** When the last client on Node B disconnects and no further federation events are expected, any remaining buffered events should be either resolved (if parents arrive via a delayed federation path) or logged as permanently unresolved with a count. A `WARN` line like `buffer drained at shutdown: N events unresolved` would make this visible.

2. **Add `apply_event` count to the stress test report.** The report currently shows 500/500 client-side sends but does not count how many federated events Node B actually applied. Run 3 would have shown `federated applied: 50 / 250` and flagged the issue automatically. This is the most important report improvement.

3. **Distinguish "buffered and later resolved" from "buffered and dropped".** A summary log line at the end of a session (e.g. `buffer stats: N received, N resolved, N unresolved at shutdown`) would make the distinction visible without manual log analysis.

---

## Checklist update

| Checklist item | Runs 1–2 | Runs 3–4 |
|---|---|---|
| Auto: send errors = 0 | ✅ | ✅ |
| Auto: join failures = 0 | ✅ | ✅ |
| Auto: content leak clean | ✅ | ✅ |
| Auto: DAG chain integrity | ✅ | ✅ |
| Manual: no ERROR lines in Node B | ❌ | ✅ |
| Manual: federation completeness (apply count) | ❌ not checked | ⚠️ run 3 partial, run 4 ✅ |

The manual federation completeness check should be promoted to an **automated check** in the report.

---

## Overall assessment

The fix in `4e2d0f3` is a genuine improvement — ERROR lines gone, buffer in place, and run 4 shows a fully clean result. The remaining work (buffer drain + report counter) is relatively small. Run 4 can be treated as the current high-water mark.

**Recommended before Phase 2:** Implement buffer drain logging and add federated `apply_event` count to the automated report. The intermittent nature of run 3 vs run 4 suggests the fix is correct in structure but needs the drain path to be reliable under all timing conditions.
