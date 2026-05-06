# Phase 1 Stress Test — Findings

**Date:** 2026-05-06  
**Reviewed by:** Documentation Claude  
**Runs analysed:** two runs, both `v0.10.3 (fac0429)`

---

## Summary

Both runs report **PASS** at the client level: 500/500 messages delivered, zero send errors, zero join failures, DAG chain integrity OK, content leak clean. Throughput is consistent at ~275 events/sec.

However, manual inspection of the Node B logs reveals a persistent issue that the report's automated checks do not capture: **federated events arrive at Node B with unknown `prev_events` during the concurrent message flood**, causing the node to reject and hold them pending. This happened in both runs and is a structural behaviour, not a fluke.

---

## Finding F-001 — Federation DAG ordering: events held pending on receiving node

### Classification
**Severity:** Medium — does not prevent message delivery to senders, but federation consistency is unverified.  
**Type:** Protocol behaviour / missing implementation.  
**Affects:** Node B receiving federated events from Node A members during concurrent flood.

### Observed behaviour

During Phase 4 (message flood), Node B logs the following for every federated `message.text` event whose parent has not yet been received:

```
ERROR xgen_node: accept_message failed  reason=step 9: unknown prev_events — event held pending
```

Counts across both runs:

| Run | Node B ERROR lines | reject_event trace entries |
|---|---|---|
| 07:06 (run 1) | 200 | 0 (event_trace not yet instrumented) |
| 07:21 (run 2) | 150 | 150 |

All rejections carry the same reason: `step 9: unknown prev_events — event held pending`.

The error message says "held pending", implying the node intends to buffer and re-process these events once their parents arrive. **However, there is no evidence in either log that the held events were subsequently applied.** The `apply_event` count in run 2 (134 entries) is far below what full federation would require (~250 federated message events expected on Node B from Node A's 5 members × 50 messages). No retry, no resolution, no drop log line is visible.

### Root cause (hypothesis)

XGen uses a per-sender DAG chain where each event's `prev_events` references that sender's own last event. During a concurrent flood, Node A's 5 members send 50 messages each in parallel with random jitter. Federation to Node B is asynchronous. Events from a single sender frequently arrive at Node B out of order — message N+1 arrives before message N. Node B's `accept_message` (step 9) requires all `prev_events` to be known before accepting an event, and has no implementation for resolving out-of-order delivery.

This is a **federation ordering gap**: the node correctly validates the DAG but has no pending-event buffer with retry logic.

### Why the report shows PASS

The automated checks verify the **client's** view: did the sender receive an OK response from its own home node? Node A accepted all 500 sends without error — the client's DAG chains are intact. The report does not query Node B's internal state or count how many federated events it actually applied to persistent storage.

### What needs to be verified / implemented

1. **Verify:** Does `accept_message` step 9 actually buffer the event, or does it log "held pending" and discard? Check the implementation — if it discards, the effective federation rate under concurrent load is significantly below 100%.

2. **Implement (if not already):** A pending-event buffer on the receiving node. When an event arrives with unknown `prev_events`, buffer it keyed by the missing parent `event_id`. When any event is successfully applied, check the buffer for events that are now unblocked and process them recursively.

3. **Add to stress test report:** A count of `apply_event` entries on each node log, compared against expected federation counts. This makes the gap visible automatically rather than requiring manual log inspection.

4. **Downgrade log level:** `ERROR` is too high for a recoverable "waiting for parent" condition. Once a buffer+retry is in place, this should log at `DEBUG` or `TRACE`. Using `ERROR` for an expected transient state makes log review harder.

---

## Secondary observation — event_trace instrumentation added between runs

Run 1 (07:06) Node B log has 249 lines and zero `event_trace` entries for `reject_event` or `apply_event`. Run 2 (07:21) has 817 lines and full trace coverage. This confirms that `event_trace` instrumentation for the node-side apply/reject path was added between the two runs today — which is good progress, but means run 1 cannot be used for federation completeness analysis.

---

## Checklist update

The manual verification checklist in `STRESSTEST_ph1.md` currently includes:

```
[manual] Federation propagation: Node B logs show events from Node A
[manual] Federation propagation: Node A logs show events from Node B
```

These items should be strengthened to also check:

```
[manual] No ERROR lines in Node logs for valid events  ← currently fails
[auto]   apply_event count on Node B ≥ expected federated messages
```

---

## Status

| Item | Status |
|---|---|
| F-001 identified | ✅ |
| F-001 root cause confirmed | ⚠️ Hypothesis — needs code review |
| F-001 fix implemented | ❌ Pending |
| Report auto-check for federation completeness | ❌ Pending |

**Recommended action before Phase 2:** Confirm whether held-pending events are buffered or discarded, implement retry buffer if missing, and add federation completeness count to the automated report.
