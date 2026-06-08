# Multiparty-tests — Findings
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

Routed defects surfaced by Multiparty-tests runs (MP-R1 onward). Per MP-R1-D6 (surface-and-route,
not patch *inside the test tranche* — the M9 D-065/D-084 discipline): a scenario FAIL that is a real
system defect is recorded here, **not** patched mid-tranche, and routes to its own fix-arc. Each
entry: symptom, grounded divergence/rejection point, contrast proof, code anchor, route. Mirrors
`tasks/M9_findings.md`.

**Fix-phase note (J-322, Joe-directed).** MP-R1 does **not** close at C7. The surfaced backlog
(these findings + the BLOCKED capability gaps) is now treated as the **purpose** of the test pass,
not a deferral: production code gets fixed, then R1 is re-run, looping toward all-green. Findings
are therefore moving from *routed/deferred* to *actively-worked fix-arcs* (each its own D-071
Phase-0, protocol-change discipline). Strategy for the BLOCKED set (fix-all-now vs catalogue across
rounds) is pending the BLOCKED-sizing pass; see J-322.

---

## MP-F1 — DM cross-node does not converge + DM messages not transcript-observable

- **Surfaced:** MP-C-07 (DM space across nodes), C4 (J-319). Status: **OPEN — routed.**
- **Repro:** `docs/tests/multiparty_scenarios/MP-C-07/` + the known-FAIL smoke
  `mp_r1_c4::mp_c_07_dm_across_nodes_converges` (stays RED until the fix-arc). Run with
  `--test-threads=1` against a `--features harness-control` node build.
- **Policy:** surface-and-route (MP-R1-D6); both facets are binary changes → out of MP-R1 scope.

**Facet 1 — convergence: DM `membership.join` applies on B but never propagates B→A.**
- Evidence: bob-view (node B) = `{alice:owner, bob:member}`; alice-view (node A) = `{alice:owner}`.
  Bob is a member on B, absent on A.
- Contrast proof (rules out federation/timing/harness): MP-C-02 (regular Space) converges B→A under
  the **identical** director / G-6 federation / settle — bob's join reaches A there. So the gap is
  **DM-specific**, not the shared machinery.
- `.events` corroboration: node A transcript = `state.dm_space_create` only; node B =
  `dm_space_create` + `state.federation_add` + 2× `membership.join`. Bob's joins are in B's stream,
  never in A's.
- Not authoring: the initial scenario (bob sends without joining) was corrected first — the DM seeds
  the creator as sole Owner-member and the invitee as a **pending invite** (so the counterparty must
  join, like MP-C-02). The divergence **survives** the corrected join flow, which is what makes it a
  real finding rather than a scenario bug.

**Facet 2 — observability (OPEN QUESTION, needs a dig): DM `message.text` not transcript-observable.**
- Both DM sends return `ok` with an `event_id` (alice a3, bob b4), but neither appears in either
  node's `.events`. Regular-Space messages **do** appear (MP-C-03 asserts both message `event_id`s
  in both nodes' cooperative sets — PASS). So it is DM-specific.
- One grep ruled out an obvious DM filter in `ops::send` / `apply_fanout`. Unresolved between "DM
  messages aren't emitted to the event stream at all" vs "optimistic-ack then rejected/not-applied"
  — a trace for the fix-arc, not a grep answer. **Consequence:** even if Facet 1 is fixed, DM
  message-convergence would not be transcript-observable until this is understood (it changes how any
  future DM scenario can be asserted).

**Code anchor (the DM model that grounds the scenario shape):**
`xgen-core/src/space/state.rs::from_dm_space_create` (state.rs:342) — seeds the creator as sole
Owner-member (`members.insert(creator…)`, :381), puts the invitee in `pending_invites` (:419, not
`members`), and auto-produces a room + invite. DM = pending-invite bootstrap.

**Route:** a DM-cross-node fix-arc (own Phase-0). Scope: trace why DM `membership.join` does not
replicate B→A (contrast the regular-Space join path that does), and characterize Facet 2 (DM
message emission to the event stream). Both are protocol/binary work, outside Multiparty-tests.

---

## MP-F2 — reject path delivers generic 4000, not the specific 30xx code

- **Surfaced:** MP-A-05 + MP-A-15, C7 (J-321). Status: **OPEN — routed.** Severity: low
  (observability/contract, not security — rejections work + events correctly absent).
- **Symptom:** `validate_event` rejections deliver an `Error` frame with `error_code=4000`
  (generic) + the specific reason in the message string, **not** the `ExchangeError::to_wire_code`
  value. A federated peer or client cannot programmatically distinguish *why* an event was rejected.
- **Grounded:** MP-A-05 → `(4000, "step 12: signature verification failed")`; MP-A-15 →
  `(4000, "event timestamp out of bounds: … exceeds now + 300s skew ceiling")` — yet
  `TimestampOutOfBounds::to_wire_code()` = `(3046, "event_timestamp_out_of_bounds")` (exchange.rs:139).
  The specific code is computed internally, not put on the wire.
- **Root:** `DispatchOutcome::Rejected` → `process_inbound` emits a generic-4000 `Error` + reason
  string (the J-081 / D-070-pending shape — specific-code-on-reject was deferred).
- **Repro:** `mp_r1_c7::mp_a_05_*` / `mp_a_15_*` (PASS-on-property: rejection + absence; the 4000≠
  specific-code gap is this finding, not a smoke failure).
- **Route:** a reject-path error-code arc (deliver the 30xx on the `Error` frame; resolve the
  D-070-pending wire-code contract). Production/binary change.
- **Note:** the matrix MP-A-15 row's Expected/F1 narrative is stale (M9.1 J-311 closed F1) — to be
  rewritten when MP-A-15's row is next touched.

---

## MP-F3 — node re-fans-out a re-submitted duplicate

- **Surfaced:** MP-A-09, C7 (J-321). Status: **OPEN — routed.** Severity: minor (mild fan-out
  amplification; no state corruption — clients dedup on apply).
- **Symptom:** the same `event_id` submitted twice is **applied once** (DAG dedup holds) but
  **re-broadcast** via `apply_fanout` on the second submit — members/observers receive it twice.
- **Grounded (DAG-dedup holds, 3 ways):** `graph.add_event` is structurally idempotent
  (tips/successors are sets, no error on a duplicate); `store` ignores a duplicate insert
  (runtime.rs:578); `persist_event` has a per-event duplicate-guard (runtime.rs:86). So
  DAG/store/disk dedup holds by construction.
- **Root:** `dispatch_event` returns `Accepted` for a re-submitted duplicate → `process_inbound`
  runs `apply_fanout` → re-broadcast. A duplicate could be dropped at dispatch before fan-out.
- **Harness limitation (recorded):** the `.events` transcript measures **fan-out emissions, not the
  DAG** — it cannot directly confirm DAG dedup (which is why MP-A-09 is recorded on the
  grounded-in-code DAG-dedup-holds, not a fan-out count). A future oracle could add a DAG/state read.
- **Repro:** `mp_r1_c7::mp_a_09_*` (recorded on grounded DAG-dedup; the 2× fan-out is this finding).
- **Route:** a dedup-at-dispatch / fan-out-suppression arc. Production/binary change.

---

## MP-A-01(ii) — federation-replay membership-preserved (NOT a finding; PENDING harness machinery)

- **Status:** PENDING (recorded for completeness; the property is **proven in-process at J-298**,
  INV-EXP close). Not a defect.
- **Why PENDING:** the INV-EXP/J-298 regression guard (an aged invite+join arriving
  `ReceivedViaFederation` skips the 3044 admission gate → membership **preserved**) needs a
  late-federation / catch-up repro — B federating with A *after* A's clock has aged the Space past
  `valid_until`. The G-6 bootstrap establishes federation **early** (before the clock phase), so that
  timing is not reachable on current harness rails.
- **Route:** late-federation/catch-up ordering machinery in `xgen-mptest` (harness, not production),
  or accept the in-process J-298 proof. (MP-A-01(i) local-expiry rejection PASSED at C7.)
