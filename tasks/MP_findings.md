# Multiparty-tests — Findings
> **Status**: ACTIVE  
> Version: 1.2  
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

- **Surfaced:** MP-A-05 + MP-A-15, C7 (J-321). **Status: RESOLVED (J-324, fix-arc shipped).** Severity: low
  (observability/contract, not security — rejections work + events correctly absent).
- **RESOLUTION (J-324, MP-F2-D1..D6):** `DispatchOutcome::Rejected(String)` widened to
  `Rejected(RejectInfo { code, name, reason })` (D1, 1-tuple-of-struct — minimal blast radius; the
  ~21 `Rejected(_)` wildcards survived unchanged); each of the 15 gates supplies its already-known
  code (D2); `reject_signal` plumbs `info.code` to the `Error` frame, deleting the hardcoded 4000
  (D4); origin gate unchanged (D5). Reason strings FROZEN byte-identical (additive code field) so
  the ~37 reason-assertion tests stayed green (D-077 backward-coherence). **Payoff verified
  end-to-end:** `mp_r1_c7::mp_a_15_clock_skew_rejected` now asserts **wire `error_code == 3046`**
  (was 4000). Build 0 + clippy clean (default + `--features harness-control`); fast suite 0-failed
  (xgen-core 667, xgen-node 72, integration 285). D-076 discharged: the `Rejected` arm returns
  `FanoutRequest::none()`, `info.code` is read only by `reject_signal` (observability) — no
  admission/ordering surface touched. Arc docs `tasks/MP_F2_REJECT_WIRE_CODE_{DESIGN,IMPL}.md`
  → COMPLETED.
- **Residual → MP-F2-followon (named, not absorbed):** (a) the 7 unmapped event-validation variants
  (signature / membership / permission) stay generic-4000 — so **MP-A-05 still delivers 4000**
  (boundary encoded as a test, `mp_a_05_*` asserts `==4000`); closing them needs an
  event-validation code-assignment decision (spec's 3001/3002 signature codes are
  registration-scoped, not event-scoped). (b) the **3030-vs-3010 tier-code spec drift** (code emits
  `3030 tier_mismatch`; spec §3.11.7 lists `3010 auth_tier_insufficient`). (c) optional cosmetic
  prose de-dup (the `(3030)` text left in some reason strings — harmless, not a drift surface).
- **Original symptom (now fixed):** `validate_event` rejections delivered an `Error` frame with
  `error_code=4000` (generic) + the reason in the message string, not the
  `ExchangeError::to_wire_code` value, so a peer/client couldn't programmatically distinguish *why*.
  MP-A-15 → was `(4000, "… exceeds now + 300s skew ceiling")` despite
  `TimestampOutOfBounds::to_wire_code()` = `3046` (exchange.rs:139). Root = a two-boundary code-drop
  (`dispatch_event` flattening to `Rejected(String)` via `err.to_string()`, runtime.rs:1086; +
  `reject_signal` hardcoding 4000, app.rs:2395). MP-F2 was the deferred completion of D-070's
  transport contract (J-081 named refinement).

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
