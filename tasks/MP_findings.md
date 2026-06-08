# Multiparty-tests — Findings
> **Status**: ACTIVE  
> Version: 1.3  
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

- **Surfaced:** MP-A-09, C7 (J-321). **Status: RESOLVED (J-326, fix-arc shipped).** Severity: minor
  (mild fan-out amplification; no state corruption — clients dedup on apply).
- **RESOLUTION (J-326, F3-D1..D5):** new unit variant `DispatchOutcome::Duplicate`; a
  `store.contains(event_id)` dedup gate in `dispatch_event`; `process_inbound` maps `Duplicate` →
  idempotent `EventAccepted` (`LocallySubmitted` only — truthful, it *was* accepted at first ingest)
  + `FanoutRequest::none()`, which suppresses **both** local fan-out and federation push. 7
  production exhaustive arms got a trivial `Duplicate` arm (migration/test sites already had
  catch-alls; all compiler-caught). **Payoff verified end-to-end:** `mp_r1_c7::mp_a_09_*` flipped
  from the tolerant `n >= 1` to **"fanned out exactly once"** (max occurrences in any single
  transcript == 1) and PASSES against a real `--features harness-control` node — the
  harness-measurement-gap note is retired, the assertion is now falsifiable, and MP-A-09's recording
  flips from PASS-on-property + routed-finding to **clean PASS**. Build 0 + clippy clean (default +
  harness-control); xgen-core 669/0 (+2 units F3-D6/D7), xgen-node 286/0 (convergence/M8 net green),
  xgen-mptest 72/0. D-076 discharged: state is a pure function of an identical log (drop vs
  idempotent-reapply → identical convergence); the §6 side-effect-skip audit confirmed all five
  re-run effects idempotent/already-fired. Arc docs `tasks/MP_F3_DEDUP_REFANOUT_{AUDIT,DESIGN,IMPL}.md`
  → COMPLETED.
- **In-arc correction (D-065, the one real finding of the arc):** the locked F3-D1 gate placement
  ("before `validate_event`") was **wrong** and `phase9_compound_c5` caught it — a signature-forgery
  reusing a *stored* event's content+id (event_id = content hash, which **excludes** the signature)
  was mis-classified `Duplicate` instead of `Rejected(step 12)`, losing the forgery signal. "Same
  `event_id`" only means "genuine duplicate" once the event is **confirmed valid**. Fix: moved the
  gate to **after `validate_event` passes**, before the Step-4 semantic gates (still in
  `dispatch_event`, still `store.contains`, still `Duplicate`) — a genuine duplicate re-validates
  cleanly, and placing it before the semantic gates is correct because an already-stored event is
  already fully accepted (gate-drift, e.g. a since-expired invite, must not un-accept it). Recorded
  as design §8a / runbook §4a.
- **Out-of-scope catches (surfaced-not-chased, design §7):** (1) drained events don't fan out
  (`FanoutRequest` carries only the triggering event; drain-recovered events reach the DAG but not
  live-broadcast — members get them via sync; separate seam). (2) re-submit-while-pending isn't
  caught by store-based dedup (the event is in the `PendingBuffer`, not the store) but is already
  benign (`PendingBuffer::add` idempotent by event_id). Store-based dedup correctly scopes to
  already-accepted duplicates.
- **Original symptom (now fixed):** `dispatch_event` returned `Accepted` for a re-submitted
  duplicate → `process_inbound` re-ran `apply_fanout` → members/observers received it twice (DAG/
  store/disk dedup held 3 ways, so applied-once was never in question; the gap was fan-out
  amplification, no state corruption).

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
