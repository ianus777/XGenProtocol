# Phase 1 Stress Test — Findings
> **Status:** ACTIVE  
> **Last updated:** 2026-05-06  

**Date:** 2026-05-06  
**Reviewed by:** Documentation Claude  
**Runs analysed:** 6 runs across three builds

---

## Run history

| Run | Time | Commit | Outcome | Federation completeness | Node log clean |
|---|---|---|---|---|---|
| 1 | 07:06 | fac0429 | PASS | ❌ 0/250 (no buffer, silent discard) | ❌ 200 ERRORs |
| 2 | 07:21 | fac0429 | PASS | ❌ 0/250 (no buffer, silent discard) | ❌ 150 ERRORs |
| 3 | 11:46 | 4e2d0f3 | PASS | ⚠️ 50/250 (buffer stalled at shutdown) | ✅ |
| 4 | 11:55 | 4e2d0f3 | PASS | ✅ 250/250 | ✅ |
| 5 | 16:44 | 0ff9a45 | PASS | ✅ 250/250 | ✅ |
| 6 | 16:44 | 0ff9a45 | PASS | ⚠️ 500/250 (report counter bug — see below) | ✅ |

---

## Finding F-001 — Federation DAG ordering: events held pending on receiving node

### Status: ✅ RESOLVED (commit 0ff9a45)

Runs 5 and 6 show zero `event buffered` lines and zero ERROR lines on both nodes. The pending-event buffer from `4e2d0f3` is resolving all out-of-order events before shutdown. The intermittent stall seen in run 3 does not recur in the latest commit.

**`pending_buffer_at_shutdown` WARN line:** Not present in either run 5 or run 6 node logs — correct, because both buffers drained cleanly. The Task 1 implementation is working as specified: silence = success.

---

## Finding F-002 — Report federation counter reads cumulative node log totals (new, minor)

### Status: ⚠️ Open — report bug, no correctness impact

**Observed in run 6 (16:44:28):**

```
Federation Completeness (message events applied on receiving node)
  Node A applied  (M0–M4):    500 /   250  ✓
  Node B applied  (M5–M9):    500 /   250  ✓
```

`500 / 250` — the counter exceeds the expected value. This happens because runs 5 and 6 were executed with the **nodes kept running between runs** (both node logs contain two `SESSION START` markers — one for Node A's federation connection and one for the test client connection). The report scans the entire node log file and counts all `apply_event message.text` entries since the node started — it does not scope the count to the current test run.

When nodes are restarted between runs, each log is fresh and the counter is correct (run 5 is clean). When nodes stay up across multiple runs, the counter accumulates. In run 6, each node had 500 message.text applied entries: 250 from run 5 + 250 from run 6.

**No correctness impact** — the actual federation behaviour is fine. This is purely a report scoping issue.

**Required fix:** The report's `apply_event` counter must be scoped to the current test run only. The simplest approach: note the node log file size (or last line number) at the start of the test, then count only lines appended after that point. Alternatively, use the test start timestamp and filter log lines by `timestamp=` field.

**Checklist impact:** The current check passes `500 / 250` as `✓` because it only tests `>=` expected. The check should also fail when the count exceeds expected by more than a small tolerance, or better, scope the count correctly so the check becomes an exact match.

---

## Session footer — still absent

Both new node logs end without a `=== XGEN SESSION END ===` footer. The nodes are being stopped between test sessions but the footer is not being written. This was noted in earlier runs and has not been addressed yet. Absence of footer = abnormal termination per Appendix G — even if the node is being killed intentionally during development, the footer should appear on clean Ctrl+C.

This is a pre-existing observation, not a regression. Tracked here for completeness.

---

## Overall assessment

| Item | Status |
|---|---|
| F-001 (federation buffer drain) | ✅ Resolved in 0ff9a45 |
| Task 1 (pending_buffer_at_shutdown WARN) | ✅ Implemented and working |
| Task 2 (federation completeness in report) | ✅ Implemented — minor scoping bug (F-002) |
| Task 3 (event buffered at DEBUG level) | ✅ Confirmed — zero buffered events, no log level issues |
| Task 4 (Appendix G rule 11) | Not yet verified — requires doc review |
| Session footer on node shutdown | ⚠️ Still absent |

Two consecutive clean runs (5 and 6) with 250/250 federation completeness on correct runs. F-001 is resolved. The remaining items are small.

**Phase 1 is clean enough to declare with one small caveat:** F-002 (report counter scoping) should be fixed before the report is used as a formal artifact. It does not affect correctness but it does affect report trustworthiness — a `500/250 ✓` line looks wrong to any reader.
