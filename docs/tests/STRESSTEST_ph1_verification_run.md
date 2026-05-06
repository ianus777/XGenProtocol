# Phase 1 — Final Round Verification Test Run

> **Status:** ACTIVE  
> **Last updated:** 2026-05-06  
> Prepared by: Documentation Claude  
> Date: 2026-05-06  
> Branch: claude/beautiful-joliot-953f69  
> Commits: f5cdf91, 288eb85, 8c9402b  
> Purpose: Verify all three acceptance criteria from `STRESSTEST_ph1_final_round.md`

---

## Before you start

Pull the branch and build:

```
git pull
git checkout claude/beautiful-joliot-953f69
cargo build --release
```

Copy the new binaries to `bin/`:

```
copy C:\cargo-targets\XGenProtocol\release\xgen-node.exe bin\xgen-node.exe
copy C:\cargo-targets\XGenProtocol\release\xgen-client.exe bin\xgen-client.exe
```

---

## Test A — `log-parse-test` (no nodes needed)

From the `bin/` directory, run:

```
xgen-client.exe log-parse-test
```

**Expected output:**

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

**Pass criterion:** OUTCOME: PASS, all six lines show ✓.

---

## Test B — Session footer verification + stress test run 1

This test also covers the F-002 counter scoping fix (first of two consecutive runs).

**Step 1 — Start both nodes** (two separate terminals, from `bin/`):

Terminal 1:
```
cd G:\My Drive\Projects\XGenProtocol\test\node_a
..\..\bin\xgen-node.exe
```

Terminal 2:
```
cd G:\My Drive\Projects\XGenProtocol\test\node_b
..\..\bin\xgen-node.exe
```

Wait for both nodes to print their startup lines before continuing.

**Step 2 — Run stress test (run 1)**, from `bin/`:

```
xgen-client.exe stress-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen
```

Wait for the report to print completely.

**Step 3 — Run stress test again immediately (run 2), nodes still running:**

```
xgen-client.exe stress-test --node-a ws://127.0.0.1:8080/xgen --node-b ws://127.0.0.1:8081/xgen
```

**Step 4 — Stop both nodes** with Ctrl+C in each terminal.

---

## Test B — What to check

### B1 — Session footer (Task 2)

After stopping the nodes, open the two new log files created during this session:

- `test\node_a\logs\xgen-node_<today's timestamp>.log`
- `test\node_b\logs\xgen-node_<today's timestamp>.log`

**Expected at the end of each file:**

```

=== XGEN SESSION END ===
ended_at=...
reason=shutdown
```

**Pass criterion:** Both node logs end with the footer block. Blank line before marker, `ended_at` and `reason=shutdown` fields present.

### B2 — Federation completeness counter scoping (Task 1 / F-002)

Check both stress test reports in `bin\stress-reports\`:

**Run 1 report** — both nodes freshly started, log files are new:
```
Federation Completeness (message events applied on receiving node)
  Node A applied  (M0–M4):    250 /   250  ✓
  Node B applied  (M5–M9):    250 /   250  ✓
```

**Run 2 report** — nodes still running, same log files now contain two runs' worth of data. With the F-002 fix the counter must be scoped to run 2 only:
```
Federation Completeness (message events applied on receiving node)
  Node A applied  (M0–M4):    250 /   250  ✓
  Node B applied  (M5–M9):    250 /   250  ✓
```

**Pass criterion:** Run 2 shows `250 / 250`, not `500 / 250`. This is the specific scenario that was broken before the fix. If it still shows `500 / 250` — counter scoping is not working.

### B3 — No regressions

Both reports must show:
- `OUTCOME: PASS`
- `Messages sent OK: 500 (100.0%)`
- `Send errors: 0`
- `DAG chain integrity: OK`
- `Content leak: CLEAN`

---

## Summary checklist

| # | Check | Pass condition |
|---|---|---|
| A | `log-parse-test` | OUTCOME: PASS, all 6 lines ✓ |
| B1 | Session footer — Node A | `=== XGEN SESSION END ===` + `reason=shutdown` at end of log |
| B1 | Session footer — Node B | same |
| B2 | Run 1 federation counter | `250 / 250` on both nodes |
| B2 | Run 2 federation counter | `250 / 250` on both nodes (not `500 / 250`) |
| B3 | Run 1 no regressions | OUTCOME: PASS, 500/500, no errors |
| B3 | Run 2 no regressions | OUTCOME: PASS, 500/500, no errors |

All 7 checks must pass. Report results back and I will do the final log verification before giving the merge go-ahead.
