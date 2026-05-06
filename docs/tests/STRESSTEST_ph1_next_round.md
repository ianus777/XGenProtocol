# Phase 1 Stress Test — Next Round Instructions for Claude Code

> Prepared by: JozefN + Documentation Claude  
> Date: 2026-05-06  
> Based on: 4-run stress test analysis, findings in `STRESSTEST_ph1_findings.md`  
> Commit under review: `4e2d0f3`

---

## Context

Four stress test runs have been completed across two commits. The latest commit (`4e2d0f3`) fixed the original F-001 issue — ERROR log lines are gone, and a pending-event buffer is in place. Run 4 (11:55) is a fully clean result: 250/250 federated events applied, zero buffering, zero errors.

However, run 3 (11:46) on the same commit shows 200 events buffered and never resolved at shutdown — an intermittent race condition where the buffer stalls if the test ends before all parent events have arrived via federation. The fix is structurally correct but needs a drain path to be deterministic.

This round has **three tasks** plus **one documentation clarification**. All are small and tightly scoped.

---

## Task 1 — Buffer drain on shutdown (required for Phase 1 sign-off)

**Problem:** When the node shuts down cleanly (Ctrl+C), any events still in the pending-event buffer are silently abandoned. There is no log line indicating how many were unresolved. This makes run 3 vs run 4 indistinguishable from the report alone.

**Required change:** In the clean shutdown path (where `write_session_footer(ExitReason::Shutdown)` is called), before writing the footer, emit a summary log line for the pending buffer:

- If the buffer is empty: no line needed — silence is the success signal.
- If the buffer has unresolved entries, emit one `WARN` line per space (or one global summary — whichever is simpler):

```
WARN xgen_node: pending_buffer_at_shutdown space_id=xgen://hash/sha256:... unresolved=12
```

This line must appear **before** the session footer blank line, so it is part of the body.

**Behaviour change:** None. The buffer logic itself does not change. This is logging only.

**Acceptance:** A run that stalls (like run 3) must now show the `WARN` line with a nonzero count. A clean run (like run 4) must show no such line.

---

## Task 2 — Federated `apply_event` count in the stress test report (required for Phase 1 sign-off)

**Problem:** The automated report shows 500/500 client-side sends but does not count how many federated events the receiving node actually applied. Run 3 passed the report while leaving 200 events permanently buffered. This gap must be closed before Phase 2.

**Required change:** After the test completes, scan both node log files and count `apply_event` entries where `event_type="message.text"`. Add a new section to the printed report and the saved `.txt` report file:

```
Federation Completeness (message events applied on receiving node)
----------------------------------------------------
  Node A applied  (from Node B members):   250 / 250  ✓
  Node B applied  (from Node A members):   250 / 250  ✓
```

Expected values: each node should apply exactly `(members_per_node × messages)` federated `message.text` events. With default config: 5 × 50 = 250 per node.

Add a corresponding automated checklist entry:

```
[auto]   Federation completeness Node A:   250 / 250  ✓
[auto]   Federation completeness Node B:   250 / 250  ✗  (got 50)   ← example failure
```

If either node's count is below expected, the overall test outcome should be `PARTIAL` rather than `PASS`.

**Implementation note:** The node log files are already referenced in the report. This task reads them after the test and counts matching lines — no changes to the node binary are required.

---

## Task 3 — Log level audit: `event buffered` line (small, one line)

**Problem:** In `4e2d0f3`, the old `ERROR` for "held pending" was correctly replaced with a `DEBUG` line:

```
DEBUG xgen_node: event buffered — waiting for unknown prev_events  event_id=...
```

This is correct behaviour. Please confirm that this line uses `DEBUG` and not any higher level. No change needed if it is already `DEBUG` — this is a verification task only.

If it is `INFO` or higher, downgrade it to `DEBUG`. The buffering of out-of-order events under concurrent load is a normal transient condition, not a noteworthy event.

---

## Task 4 — Documentation clarification: Appendix G, Parsing Rules (no code change)

**Instruction from JozefN:**

Add the following rule to the **Parsing Rules** section of `docs/xgen_appendix_g_en.md`, after the existing rule 10 ("Unknown fields MUST be silently ignored"):

```
11. Field value matching MUST be case-insensitive. The capitalisation of field
    values carries no semantic meaning and exists solely for human readability.
    For example: `direction=IN`, `direction=in`, and `direction=In` are
    equivalent. `action=ApplyEvent` and `action=apply_event` are equivalent.
    Parsers and analyzers MUST NOT treat capitalisation differences as distinct
    values.
```

This is a clarification to the format contract, not a code change. The Rust implementation already produces consistent casing (`IN`, `OUT`, `LOCAL`, lowercase action names) — this rule documents the intent for any future third-party parser or AI log analyzer consuming XGen logs.

Also update the version line in the Appendix G header from `Version: 1.0` to `Version: 1.1` and update `Last edited` to the current date.

---

## Acceptance Criteria for This Round

- [ ] Two consecutive stress test runs both show `PASS` with federation completeness 250/250 on both nodes
- [ ] A run that stalls (if reproducible) now shows the `WARN pending_buffer_at_shutdown` line
- [ ] The `event buffered` log line is confirmed at `DEBUG` level
- [ ] Appendix G Parsing Rules updated with rule 11, version bumped to 1.1

When all four are done, Phase 1 can be declared clean.
