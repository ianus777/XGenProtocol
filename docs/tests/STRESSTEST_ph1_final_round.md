# Phase 1 Stress Test — Final Cleanup Round Instructions for Claude Code

> Prepared by: JozefN + Documentation Claude  
> Date: 2026-05-06  
> Based on: 6-run stress test analysis, findings in `STRESSTEST_ph1_findings.md`  
> Commit under review: `0ff9a45`  
> Previous instructions: `STRESSTEST_ph1_next_round.md`

---

## Context

Phase 1 stress testing is functionally complete. Commit `0ff9a45` resolves all correctness issues: zero ERROR lines, zero buffered events, 250/250 federated events applied on both nodes in clean runs. The `pending_buffer_at_shutdown` WARN line and the federation completeness section in the report are both implemented and working.

This is a **cosmetic and tooling-only round**. No protocol logic, no architecture changes. Three small fixes and one new test.

---

## Task 1 — Fix federation completeness counter scoping (F-002)

**Problem:** The report's `apply_event` counter reads the entire node log file. When nodes stay running between consecutive test runs, the counter accumulates across all runs in that session, producing nonsense values like `500 / 250 ✓`. Run 6 (16:44:28) demonstrated this exactly.

**Required change:** Scope the counter to the current test run only. At the start of the stress test (before Phase 1 Setup), record the current byte offset or line count of each node log file. After Phase 4 completes, count only `apply_event message.text` lines that appear after that recorded offset.

The simplest reliable approach: record the file size in bytes at test start using `std::fs::metadata`, then after the test open the file, seek to that offset, and count from there.

**Acceptance:** Running two consecutive stress tests with nodes kept alive between them must produce `250 / 250` in both reports, not a cumulative total in the second.

**Checklist change:** The auto-check should also flag when the count *exceeds* the expected value (not just when it falls short), as an indicator that scoping may be broken:

```
[auto]   Federation completeness Node A:    250 /   250  ✓   ← exact match
[auto]   Federation completeness Node A:    500 /   250  ✗   ← exceeds expected — log scoping error
[auto]   Federation completeness Node A:     50 /   250  ✗   ← below expected — buffer issue
```

---

## Task 2 — Session footer on node shutdown

**Problem:** Node logs end without `=== XGEN SESSION END ===`. Per Appendix G, absence of a footer signals abnormal termination (crash or kill). The nodes are being stopped cleanly during test runs but the footer is not being written.

**Required change:** Verify that `write_session_footer(ExitReason::Shutdown)` is being called on the Ctrl+C signal handler path in `xgen-node/src/main.rs`. If the signal handler is terminating the process before the footer call is reached, restructure the shutdown sequence so the footer is written before exit.

This is a one-area check — the footer call is already implemented (`LOGGING_implementation.md` confirms it), so this is likely a signal handling ordering issue rather than a missing implementation.

**Acceptance:** Both node log files end with:

```

=== XGEN SESSION END ===
ended_at=...
reason=shutdown
```

The manual checklist item `Session footer present in all Node logs (clean shutdown)` must pass without exception.

---

## Task 3 — Case-insensitivity test (`log-parse-test` subcommand)

**Background:** Appendix G Parsing Rule 11 (added in version 1.1) states that field value matching MUST be case-insensitive. Capitalisation of field values carries no semantic meaning. This rule exists to protect any future third-party parser or AI log analyzer from treating `direction=IN` and `direction=in` as different values.

**Required change:** Add a `log-parse-test` subcommand to `xgen-client` (alongside `smoke-test` and `stress-test`). This subcommand does not connect to any node — it is a self-contained parser contract test.

The test constructs a synthetic log snippet in memory containing field values in deliberately varied casing, then passes it through the same log parsing logic used by the stress test report (the `apply_event` counter, the `direction` filter, etc.) and asserts that results are identical regardless of casing.

**Minimum test cases:**

| Input | Must match same as |
|---|---|
| `direction=IN` | `direction=in`, `direction=In`, `direction=iN` |
| `direction=OUT` | `direction=out`, `direction=Out` |
| `direction=LOCAL` | `direction=local`, `direction=Local` |
| `action=apply_event` | `action=Apply_Event`, `action=APPLY_EVENT`, `action=Apply_event` |
| `action=reject_event` | `action=REJECT_EVENT`, `action=Reject_Event` |
| `event_type=message.text` | `event_type=Message.Text`, `event_type=MESSAGE.TEXT` |

**Output on pass:**
```
XGen Log Parse Test — Parsing Rule 11 (case-insensitive field values)
----------------------------------------------------
  direction=IN / in / In / iN          ✓
  direction=OUT / out / Out            ✓
  direction=LOCAL / local / Local      ✓
  action=apply_event variants          ✓
  action=reject_event variants         ✓
  event_type=message.text variants     ✓

OUTCOME: PASS — all field value comparisons are case-insensitive
```

**Output on fail:** list the specific comparison that failed and the values that were treated as distinct.

**Note:** This test does not require the Rust log parser to actually *produce* mixed-case output — the production code always emits consistent casing. The test verifies that the *consumer* side (the report's grep/filter logic) is case-insensitive. If the report uses direct string comparison rather than case-folded matching, this test will catch it.

---

## Acceptance Criteria for This Round

- [x] Two consecutive stress test runs with nodes kept alive show `250 / 250` in both reports (not cumulative)
- [x] Both node logs end with `=== XGEN SESSION END ===` and `reason=shutdown`
- [x] `xgen-client log-parse-test` runs and reports PASS
- [x] No regressions in existing smoke-test or stress-test behaviour

**All four criteria implemented. See J-033 in JOURNAL.md.**

Phase 1 stress test final round: ✅ Complete — commit `f5cdf91`.
