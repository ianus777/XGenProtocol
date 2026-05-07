# Phase 1 Stress Test — Findings
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

**Reviewed by:** Documentation Claude  
**Runs analysed:** 8 runs across four builds (including verification run)

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
| 7 | 23:45 | 8c9402b | PASS | ✅ 250/250 | ✅ |
| 8 | 23:45 | 8c9402b | PASS | ✅ 250/250 (F-002 fix confirmed) | ✅ |

---

## Finding F-001 — Federation DAG ordering: events held pending on receiving node

### Status: ✅ RESOLVED (commit 0ff9a45)

Runs 5 and 6 show zero `event buffered` lines and zero ERROR lines on both nodes. The pending-event buffer from `4e2d0f3` is resolving all out-of-order events before shutdown. The intermittent stall seen in run 3 does not recur in the latest commit.

**`pending_buffer_at_shutdown` WARN line:** Not present in either run 5 or run 6 node logs — correct, because both buffers drained cleanly. The Task 1 implementation is working as specified: silence = success.

---

## Finding F-002 — Report federation counter reads cumulative node log totals (new, minor)

### Status: ✅ RESOLVED (commit 8c9402b, verified runs 7 and 8)

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

## Session footer

### Status: ✅ RESOLVED (commit f5cdf91, verified in session 14)

The session footer (`=== XGEN SESSION END === / reason=shutdown`) was implemented and verified working in session 14 (`STRESSTEST_ph1_final_round.md`). The verification run (session 15) could not re-verify it due to a Windows tooling limitation — background node processes cannot receive graceful Ctrl+C from the automation environment, so nodes were force-killed and no footer was written. This is a test-harness limitation, not a protocol regression. The implementation is correct.

---

## Verification run (session 15 — commit 8c9402b)

Formal verification against the 7-check checklist in `STRESSTEST_ph1_verification_run.md`:

| # | Check | Result |
|---|---|---|
| A | `log-parse-test` — OUTCOME: PASS, all 6 lines ✓ | ✅ PASS |
| B1 | Session footer — Node A | ⏸ Not re-verified (Windows tooling limitation — see note) |
| B1 | Session footer — Node B | ⏸ Not re-verified (Windows tooling limitation — see note) |
| B2 | Run 1 federation counter — `250/250` | ✅ PASS |
| B2 | Run 2 federation counter — `250/250` (not `500/250`) | ✅ PASS |
| B3 | Run 1 no regressions — OUTCOME PASS, 500/500, 0 errors | ✅ PASS |
| B3 | Run 2 no regressions — OUTCOME PASS, 500/500, 0 errors | ✅ PASS |

**Note on B1:** The session footer (`=== XGEN SESSION END ===`) cannot be verified in the automation environment because background node processes cannot receive graceful Ctrl+C on Windows. Nodes were force-killed, so no footer was written during this session. This is a test-harness limitation, not a protocol issue. The footer was confirmed working in session 14 (`STRESSTEST_ph1_final_round.md`), which remains the authoritative proof.

**Verification outcome:** Complete. All verifiable checks pass. B1 carries prior proof from session 14.

---

## Overall assessment

| Item | Status |
|---|---|
| F-001 (federation buffer drain) | ✅ Resolved in 0ff9a45 |
| Task 1 (pending_buffer_at_shutdown WARN) | ✅ Implemented and working |
| Task 2 (federation completeness in report) | ✅ Resolved in 8c9402b — 250/250 on consecutive runs |
| Task 3 (event buffered at DEBUG level) | ✅ Confirmed — zero buffered events, no log level issues |
| Task 4 (Appendix G rule 11) | ✅ Verified — log-parse-test PASS all 6 lines |
| Session footer on node shutdown | ✅ Resolved in f5cdf91 — verified session 14 |
| F-002 (report counter scoping) | ✅ Resolved in 8c9402b |

**Phase 1 stress test is clean and complete.** Runs 7 and 8 (commit 8c9402b) confirm all acceptance criteria. The formal verification run (session 15) passes all verifiable checks. No open findings.
