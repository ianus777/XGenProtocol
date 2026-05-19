# Task — Federation Event Propagation Phase 9 Implementation
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-19 (initial — authored from Joe-locked findings in `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` §8)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Ship Phase 9 of the Federation Event Propagation milestone — the deployment-level adversarial proof that federation works under conditions that matter. Twelve scenarios across mixed harness shapes; three observability preconditions; one flake-fix precondition; one milestone-close commit. After this task ships, the milestone flips PLAY → DONE and M6 (new) unblocks.

**This task implements the locked findings.** The survey is `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY.md` (COMPLETED). The findings are `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` (COMPLETED, v1.1). The four Joe-locks recorded in the findings §8 are the load-bearing scoping decisions; this task does not re-litigate them.

**Priority anchor (from survey v2.0).** Priority is working functions, not done-mark on roadmap. A milestone that ships green and turns out to have federation bugs three weeks later is a milestone that failed at its real job. Every scenario in this task is designed to *find bugs if they exist*, not to put a green checkmark next to an F-item.

---

## §1 — Mandatory reading

Read in this order before starting implementation.

| Source | What it gives | Why read it |
|---|---|---|
| `CLAUDE.md` MANDATORY behaviour rules | Rules 1–7. | Apply throughout. Quote actual `cargo test` output (Rule 5); never fabricate test counts (Rule 1); stop and report on tool failure (Rule 3); write JOURNAL last (Rule 4). |
| `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` (COMPLETED v1.1) | Full survey findings + four Joe-locks in §8. | Authoritative scope for this task. Per-scenario stress dimensions, honesty assertions, harness choices, observability gaps all locked. |
| `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.9 | Original Phase 9 scope at runbook handoff. | Reference — but the locked scope in the findings supersedes the runbook's "Two-Node smoke + Three-Node if affordable" language. |
| `docs/xgen_federation_propagation_design.md` v1.0 ACTIVE | All ten F-items locked; §15 Implementation Complete record. | Authoritative protocol behaviour the tests assert against. |
| `xgen-client/src/app.rs` `cmd_stress_complete` (~line 3597+) | The harness precedent for deployment-level scenarios. | All "stress-complete shape" references in this task refer to this implementation. |
| Phase 5/6/7 integration tests (cited in survey §1) | Style + observability patterns for NodeRuntime-level tests. | Phase 9's NodeRuntime-level scenarios (4, C5, C7, C9, C10) extend the patterns shown there. |

---

## §2 — Locked decisions from survey findings §8

These are restated here so this task is self-contained.

**Q1 Lock — 12 scenarios.** 6 baseline (1, 2, 3, 4, 5, 6) + 6 compounds (C2, C3, C5, C7, C9, C10). Deferred: C1, C4, C6, C8 → `federation-stress` follow-on milestone (`tasks/FEDERATION_STRESS_FOLLOWON.md`).

**Q2 Lock — G4 (audit log for F-3) deferred to M6.** Phase 9 uses transient log parsing for F-3 rejection observation. M6 (new) Phase 0 owns the protocol audit-log schema.

**Q3 Lock — Flake fix option (i) first.** Add `#[serial_test::serial]` to both flake sites. Escalation criterion (option (ii) walk-back) defined in §6 of this task.

**Q4 Lock — Multi-commit Phase 9.** Expected shape (~5-7 commits) sequenced in §3 below.

---

## §3 — Commit sequence

Phase 9 ships as **5-7 atomic commits**, each independently reviewable, each with its own JOURNAL sub-entry, each with quoted `cargo test` output. Final commit closes the milestone.

### Commit 1 — Observability preconditions (G1 + G2 + G3)

**Scope.** Three observability gaps closed in one commit.

**G1 — `xgen-node_state.json::peers` field.** Currently hard-coded to `vec![]` at [`xgen-node/src/app.rs:1775`](xgen-node/src/app.rs:1775). Read `FederationRegistry::peer_records` (already loaded at startup, wrapped in `Arc<Mutex<>>`), render each `PeerOperationalRecord` into the `FederatedPeer` shape that `xgen-common/src/state.rs::FederatedPeer` already exports. ~50 lines.

**Fields to render per peer:**
- `peer_node_id` — straight pass-through.
- `lost_connection` — straight pass-through.
- `last_successful_session` — straight pass-through (RFC 3339 string, already present).
- `next_reconnect_attempt` — straight pass-through.

If `FederatedPeer` schema in `xgen-common/src/state.rs` is missing fields the test scenarios need, extend the schema (add fields with `#[serde(default)]` for forward-compat per Phase 5 precedent). Schema extensions are CLAUDE.md Rule 6 territory — if any extension feels load-bearing, stop and ask Joe.

**G2 — Stable structured trace events for F-1 push + F-3 reject paths.** Today's `tracing::warn!`/`error!` calls carry free-form message text. Add `event = "..."` field with stable identifier. Affected sites:

| Site | File:line | New trace event name |
|---|---|---|
| F-1 push success | `xgen-node/src/federation_session.rs:201+` | `event = "federation_push_sent"` |
| F-1b drop (queue full) | `xgen-node/src/federation_session.rs:245` | `event = "federation_push_dropped_full"` |
| F-1b drop (peer unregistered) | `xgen-node/src/federation_session.rs:256` | `event = "federation_push_dropped_unregistered"` |
| F-5 guard fired | `xgen-node/src/federation_session.rs:209` | `event = "federation_push_skipped_origin"` |
| F-3 reject | `xgen-core/src/node/runtime.rs:378` | `event = "f3_reject"` |
| Co-located rejection log | `xgen-node/src/app.rs:1441` | `event = "event_rejected", reason = %reason` |
| Validation reject (per F-4) | inside `process_inbound` reject arms | `event = "validation_reject"` |

Each trace event includes structured fields (peer_node_id, space_id, event_id where relevant). Free-form message text is *additionally* allowed for human readability but the `event` field is the load-bearing identifier for Phase 9 tests.

**G3 — Fan-out trace event.** `xgen-node/src/fanout.rs::apply_fanout` success path adds:
- `event = "fanout_delivered", client_id = X, event_id = E` for each successful delivery.
- `event = "fanout_dropped_channel_full", client_id = X, event_id = E` for the existing try_send failure path.

Pairs with Scenario 1's honesty check #2 and Scenario 2's destination-side absence assertion.

**Verification.** `cargo test --workspace` passes (existing 519 tests unchanged). New trace events visible in test logs when run with `XGEN_LOG=info`. Commit message quotes actual test output.

**DoD for Commit 1:**
- [ ] G1 implemented: `xgen-node_state.json::peers` populated from `FederationRegistry::peer_records` at `build_node_state` call.
- [ ] G2 implemented: 7 trace event additions across federation_session.rs, runtime.rs, app.rs with stable `event` field.
- [ ] G3 implemented: 2 trace event additions in fanout.rs.
- [ ] `cargo test --workspace` passes; test count quoted.
- [ ] No public API breakage (Phase 5 + Phase 6 + Phase 7 tests still pass).
- [ ] JOURNAL sub-entry written.

---

### Commit 2 — Flake fixes (option (i))

**Scope.** `#[serial_test::serial]` applied at two sites.

**Flake #1 site.** Four tests in `xgen-common/src/precedence.rs`:
- `resolve_log_level_*` family at lines 148-178 per survey trace.
- Add `serial_test = "..."` to `xgen-common/Cargo.toml` `[dev-dependencies]` if not already present.

**Flake #2 site.** `xgen-node/src/tests/federation_delta_integration.rs` — `#[serial_test::serial]` on the test module or on individual tests in the module. Decision shape: if the module has 1-3 tests, serialise individually; if 4+ tests, the module attribute is cleaner. Verify by counting — `cargo test federation_delta_integration --list` is the canonical source.

**Escalation criterion (Q3 walk-back to option (ii)).** If during Commit 3-N (per-scenario test additions) any new Phase 9 integration test exhibits either:
1. A `127.0.0.1:0` bind race or "address already in use" failure under workspace parallelism that isn't explained by the test's own logic, OR
2. WS frame-ordering inconsistency where the same test passes in isolation but fails under `--workspace`,

then STOP per CLAUDE.md Rule 3. Report to Joe. Walk back to option (ii) — investigate the underlying tokio/WS race. Do NOT silently add more `#[serial_test::serial]` annotations across new tests as a workaround. The diagnostic signal IS Phase 9's deployment stress; suppressing it defeats the purpose.

**Verification.** Run `cargo test --workspace` 10 times. Per CLAUDE.md Rule 5, quote each run's PASS/FAIL outcome. Acceptable: 10/10 PASS. Anything else is a signal to escalate.

**DoD for Commit 2:**
- [ ] `#[serial_test::serial]` applied to both flake sites.
- [ ] Cargo.toml updates if needed.
- [ ] 10 consecutive `cargo test --workspace` runs all PASS; runs quoted.
- [ ] JOURNAL sub-entry written.

---

### Commit 3 — Baseline deployment scenarios (1, 2, 3)

**Scope.** Three deployment-level scenarios via `stress-complete` harness shape.

**Scenario 1 — Two-Node federation push smoke.**
- 2 Nodes spawned as separate `xgen-node` binaries.
- 100 events from Alice on A, 5 event types, mixed payload sizes (100B/10KB/100KB), 2 concurrent clients on A.
- Honesty assertions per findings §2.1 sub-item C:
  - Alice's post timestamp > `handshake_active_at_B_ts`.
  - Each event arrives on B's wire after `handshake_active_at_B_ts`.
  - Each event's `apply_federation_push` invocation observed on A via G2 trace event `federation_push_sent`.
  - F-5 guard did NOT fire (no `federation_push_skipped_origin` trace events for these 100 events).
- File location: `xgen-node/src/tests/phase9_two_node_smoke.rs` (new file).

**Scenario 2 — Three-Node anti-transitivity.**
- 3 Nodes spawned. A↔B and A↔C federated; B↔C explicitly NOT federated.
- 100 events from A, observed at B and C in parallel.
- Source-side honesty (load-bearing per findings §2.2): G2 trace event `federation_push_skipped_origin` fired on B for E-from-A (origin = `ReceivedViaFederation`). Zero peers iterated.
- Destination-side honesty: E appears in C's CommLog with `from=A`, never with `from=B`.
- File location: `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs` (new file).

**Scenario 3 — Drop-and-recover.**
- 2 Nodes spawned. A↔B federated. 10 events queued, drop mid-stream, 2 sequential drop-recover cycles.
- Adapt harness: spawn B as `tokio::process::Child`; `child.kill().await` for drop; respawn with same `data_dir`.
- Honesty assertions per findings §2.3 sub-item C:
  - Assertion 1: R14 log lines (now `event = "federation_push_dropped_unregistered"` per G2) fire on A for all queued events.
  - Assertion 2: F-1a tip-exchange handshake observed on B's startup logs; `peer_records[B].last_successful_session` advances; `state.federation_add` NOT re-streamed in delta.
  - Assertion 3: Bob receives queued events in topological order.
- File location: `xgen-node/src/tests/phase9_drop_and_recover.rs` (new file).

**Verification.** `cargo test --workspace` passes; baseline 519 + Commit 1/2 unchanged + 3 new tests = expected ~522 minimum. Quote actual count.

**DoD for Commit 3:**
- [ ] 3 new test files implemented.
- [ ] All honesty assertions per findings §2.1-§2.3 sub-item C satisfied.
- [ ] `cargo test --workspace` passes; test count quoted.
- [ ] JOURNAL sub-entry written.

---

### Commit 4 — Baseline scenario 5 + 6 + B1 honesty test

**Scope.** Two more baseline scenarios + the Phase 7 B1 honesty test elevated to binary level.

**Scenario 5 — Unknown-signer first-contact.**
- Stress-complete shape: A↔B federated; Bob's Identity on A but NOT replicated to B.
- 4 timing variants × 4 missing variants per findings §2.5 sub-item E — but only the load-bearing case at binary level: identity arrives 1ms before timeout (resolves) and 1ms after timeout (must NOT resolve).
- Observation: `xgen-node_state.json::pending_identity_replication` polling; sub-second timing via tracing event timestamps.
- Honesty: `pending_identity_replication` decrements within 100ms of identity-replicate hook firing (proves hook ran, not periodic sweep).
- File location: `xgen-node/src/tests/phase9_unknown_signer_first_contact.rs` (new file).

**Scenario 6 — Federation-relationship rejection.**
- Stress-complete shape: Node X federates with B (session-level) but is NOT in any of B's Spaces' `federation_nodes`. X attempts to push event for Space S.
- 3 relationship-state variants × 3 event families per findings §2.6 sub-item E — but compress to load-bearing cases at binary level: never-existed (the canonical case) + asymmetric.
- Honesty assertions per findings §2.6 sub-item C:
  - F-3 reject log line includes exactly `event = "f3_reject"` (G2 trace event) with `reason = federation_relationship_missing`, peer X, Space S.
  - Event E NOT in B's DAG (verify via `xgen-node_state.json::spaces[].event_count`).
  - B's local fan-out NOT invoked for E (no `fanout_delivered` G3 trace event with E's event_id).
- **Lock B1 honesty test (binary-level).** X sends `state.federation_add` event. Assert outcome is NOT `federation_relationship_missing` (the negative assertion per Phase 7 precedent). Per findings §2.6: HeldPending due to F-10 unknown-signer is orthogonal to F-3 skip; the assertion isolates F-3.
- File location: `xgen-node/src/tests/phase9_federation_relationship_rejection.rs` (new file).

**Verification.** `cargo test --workspace` passes; expected ~525 minimum.

**DoD for Commit 4:**
- [ ] 2 new test files (Scenarios 5 + 6 plus B1 binary-level test).
- [ ] All honesty assertions per findings §2.5-§2.6 sub-item C satisfied.
- [ ] `cargo test --workspace` passes; test count quoted.
- [ ] JOURNAL sub-entry written.

---

### Commit 5 — Compound deployment scenarios (C2, C3)

**Scope.** Two compound scenarios at deployment level.

**Compound C2 — F-5 anti-transitivity under push queue depth.**
- Extends Scenario 2 setup: 3 Nodes, A↔B and A↔C federated, B↔C not federated.
- A pushes 100 events to B and C in rapid succession.
- Load-bearing assertion: log every outbound push from B (G2 `federation_push_sent` for B→A, `federation_push_skipped_origin` for B's attempts to forward A-origin events). Assert zero outbound pushes from B that carry an event with origin = `ReceivedViaFederation`. Source-side, not destination-side — catches bug catalogue M5 even if a hypothetical bypass affects only some events.
- File location: `xgen-node/src/tests/phase9_compound_c2_anti_transitivity_at_load.rs` (new file).

**Compound C3 — F-3 rejection during F-1a recovery.**
- Setup: A↔B federate for Space S. B drops (kill binary). While B is down, A applies `state.federation_remove` for itself on S (via client-side action on A — this needs a `state.federation_remove` event-emit code path; verify the event type exists or extend if needed).
- B comes back; A initiates F-1a handshake.
- Honesty assertions: handshake completes (per-peer relationship distinct from per-Space relationship); subsequent A→B push events for S are rejected with `event = "f3_reject"`, `reason = federation_relationship_missing`.
- **Note for Clair:** if `state.federation_remove` event type doesn't exist in the spec/code today, STOP and ask Joe before extending. Adding a new event type is a spec change, not a Phase 9 test addition. Likely alternatives: use `membership.kick` if the framing fits, or scope the test to "A's federation_nodes membership for S revoked via direct state edit on A's data dir while B is down" (lower fidelity but Phase 9-shippable).
- File location: `xgen-node/src/tests/phase9_compound_c3_f3_during_recovery.rs` (new file).

**Verification.** `cargo test --workspace` passes; expected ~527 minimum.

**DoD for Commit 5:**
- [ ] 2 new test files for compounds C2 + C3.
- [ ] If `state.federation_remove` ambiguity surfaces in C3: stopped and asked Joe per Rule 6 BEFORE implementing.
- [ ] `cargo test --workspace` passes; test count quoted.
- [ ] JOURNAL sub-entry written.

---

### Commit 6 — Compound NodeRuntime scenarios (4, C5, C7, C9, C10)

**Scope.** Five NodeRuntime-level scenarios. All in `xgen-core` test directory since they exercise `NodeRuntime::dispatch_event` directly without TCP.

**Scenario 4 — Validation asymmetry regression (NodeRuntime-level).**
- 6 forgery variants × 5 event families = 30 forgery test cases per findings §2.4 sub-item E.
- For each case: construct forged event, call `runtime.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id))`, assert `DispatchOutcome::Rejected` with the specific validation-error reason.
- File location: `xgen-core/src/tests/phase9_validation_asymmetry.rs` or `xgen-core/src/node/tests/phase9_validation_asymmetry.rs` (Clair chooses based on existing test organisation in xgen-core).

**Compound C5 — Validation asymmetry under load.**
- 100 mixed valid+forged events fed via `dispatch_event` calls in random order.
- Assert per-event outcome independently: no forged event's rejection state affects a valid event's acceptance.
- File location: `xgen-core/src/tests/phase9_compound_c5_validation_under_load.rs`.

**Compound C7 — `continue_from` pagination at boundary.**
- 4 test cases per findings §3.7: Space with N=999, N=1000, N=1001, N=2000 events.
- Federate, observe delta size; assert all events arrive on the receiving Node's DAG (no boundary loss).
- NodeRuntime-level for the assertion sharpness; pair with one end-to-end smoke test (can be folded into one of the deployment scenarios if budget tight).
- File location: `xgen-core/src/tests/phase9_compound_c7_pagination_boundary.rs`.

**Compound C9 — F-3 drain-time approximation hazard.**
- Setup: federation event from peer X for Space S buffers in B's PendingBuffer (missing predecessor). X removed from S's `federation_nodes`. Predecessor arrives. Drain re-dispatches with `peer_node_id: None`.
- Assert: event ingests (the approximation accepts) — and document the behaviour. Bound assertion: event was buffered for ≤ 30 s (F-4a window). If bound exceeds 30 s, that's an unrecorded bug.
- File location: `xgen-core/src/tests/phase9_compound_c9_drain_time_hazard.rs`.

**Compound C10 — Identity-replicate hook serialisation under lock contention.**
- 3 concurrent federation peers push events for unknown Bob to B; 3 concurrent identity-replicate messages for Bob.
- Assert: no event is drained twice (no duplicate-ingest DAG rejection); each buffered event drains exactly once.
- File location: `xgen-core/src/tests/phase9_compound_c10_identity_lock_contention.rs`.

**Verification.** `cargo test --workspace` passes; expected ~565 minimum (Scenario 4 adds ~30 tests, others add ~5 each).

**DoD for Commit 6:**
- [ ] 5 new test files for Scenario 4 + compounds C5, C7, C9, C10.
- [ ] All honesty assertions per findings §2.4 and §3.5/§3.7/§3.9/§3.10 satisfied.
- [ ] `cargo test --workspace` passes; test count quoted.
- [ ] JOURNAL sub-entry written.

---

### Commit 7 — Milestone close

**Scope.** Final close-out. No new tests; updates to CLAUDE.md + ROADMAP.md + JOURNAL.md + design doc.

**Updates:**
1. **CLAUDE.md.** Federation Event Propagation milestone block flips 🟢 PLAY → ✅ DONE. M6 (new) block flips 🟡 PENDING → ACTIVE (or whatever the natural next state is). Last updated bumped.
2. **ROADMAP.md.** Same state-transition reflected. Last updated bumped.
3. **`docs/xgen_federation_propagation_design.md` §15** updated from "eight implementation phases shipped" to "nine implementation phases shipped" with Phase 9 line added. Last updated bumped on file header.
4. **`tasks/FEDERATION_PROPAGATION_COMPLETION.md`** flipped from ACTIVE to COMPLETED. Last updated bumped.
5. **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (this file) flipped from ACTIVE to COMPLETED. Last updated bumped.
6. **JOURNAL.md.** J-### consolidated entry covering all 6 prior commits' sub-entries plus close-out. Test count quoted from actual `cargo test --workspace` output. Milestone shipping summary.

**Verification.** Final `cargo test --workspace` run quoted in commit message. CLAUDE.md milestone block reflects DONE state. ROADMAP.md in sync. Design doc §15 lists Phase 9.

**DoD for Commit 7:**
- [ ] CLAUDE.md updated: milestone PLAY → DONE.
- [ ] ROADMAP.md updated: same.
- [ ] Design doc §15 updated: Phase 9 line added.
- [ ] Runbook + this file flipped to COMPLETED.
- [ ] JOURNAL J-### consolidated entry written.
- [ ] Final `cargo test --workspace` output quoted.
- [ ] Joe pushes the commit.

---

## §4 — Aggregate Definition of Done — Phase 9 milestone

These items must hold at Commit 7 close:

- [ ] All 7 commits shipped sequentially per §3.
- [ ] 12 scenarios implemented (6 baseline + 6 compounds: C2, C3, C5, C7, C9, C10).
- [ ] 3 observability gaps closed (G1, G2, G3).
- [ ] 2 pre-existing flakes serialised (option (i) — escalation criterion not triggered, OR escalation triggered and handled per §6).
- [ ] Test count grew from baseline 519 to ≥ 565 (best estimate; actual quoted).
- [ ] All `cargo test --workspace` runs across 7 commits passed; outputs quoted in respective commit messages.
- [ ] Failure-mode catalogue from findings §6: 11 of 14 bugs catalogued as "caught by Phase 9" actually have a Phase 9 test that would detect them. Confirm by inspection at close.
- [ ] JOURNAL.md has a consolidated J-### entry summarising the milestone close, with sub-entries for each of the 7 commits.
- [ ] CLAUDE.md milestone block flipped PLAY → DONE.
- [ ] ROADMAP.md reflects same state.
- [ ] Design doc §15 lists nine shipped phases.
- [ ] `tasks/FEDERATION_STRESS_FOLLOWON.md` exists and is in PENDING state (created at this milestone close per Step 3 of session plan).
- [ ] Client-Side Consequences Audit identified as the next J-081-shape canonical doc (per memory #14); placeholder task file optional but recommended.

**The 4 catalogue bugs NOT caught by Phase 9** (M6, M8, M13 per findings §6) explicitly carry forward to either:
- `federation-stress` follow-on milestone (M6, M8 via deferred compounds C4, C8, C6).
- Client-Side Consequences Audit (M13 — F-1c registry consistency).

This is intentional. The honest framing of milestone close-out names what was and was not proven.

---

## §5 — Coordination with M6 (new)

M6 (new) Phase 2 lands the envelope-level `event_id` on `TransportMessage::Error` per D-070. Phase 9 produces the rejection paths that M6 Phase 2 will wire — specifically the G2 trace events at `f3_reject`, `validation_reject`, and the co-located rejection log at app.rs:1441. M6 Phase 2 will not change the trace events; it adds the wire-layer rejection signal alongside.

**No M6 work in this task.** M6 unblocks at Phase 9 close. The locked Q2 (defer G4 audit-log to M6) is the explicit hand-off point.

---

## §6 — Escalation rules

Beyond CLAUDE.md Rules 1-7 (which always apply), Phase 9 has one escalation rule unique to this task:

**Flake escalation rule (per Q3 lock).** If during any commit in §3, any new Phase 9 integration test exhibits:
1. A `127.0.0.1:0` bind race or "address already in use" failure under `cargo test --workspace` that isn't explained by the test's own logic, OR
2. WS frame-ordering inconsistency where the same test passes in isolation but fails under `--workspace`,

then:
- STOP per Rule 3.
- Report to Joe with: which test, what symptom, what `cargo test --workspace` output (quoted per Rule 2).
- Walk back to option (ii) per Q3 lock — investigate the underlying tokio/WS race shape.
- Do NOT silently add more `#[serial_test::serial]` annotations. Suppressing the diagnostic signal defeats the purpose; Phase 9's deployment stress IS the signal.

This escalation rule is documented here so it's not lost across the multi-commit cadence.

---

## §7 — Cross-references

- **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY.md`** (COMPLETED v2.0) — the survey task that produced the findings.
- **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** (COMPLETED v1.1) — Joe-locked findings; authoritative scope for this task.
- **`tasks/FEDERATION_PROPAGATION_COMPLETION.md`** §3.9 (ACTIVE — flipped to COMPLETED at Commit 7) — original Phase 9 scope at runbook handoff.
- **`tasks/FEDERATION_STRESS_FOLLOWON.md`** (PENDING — created at Commit 7) — deferred compounds C1, C4, C6, C8 + clock injection + parallelism investigation if Q3 escalation didn't fire.
- **`docs/xgen_federation_propagation_design.md`** (ACTIVE v1.0) — canonical design; §15 records nine shipped phases at Commit 7.
- **`docs/xgen_propagation_reliability.md`** (J-081 audit, ARCHIVED) — the audit that motivated the milestone; M13 carries forward to the Client-Side Consequences Audit per its precedent.
- **`DECISIONS.md`** D-065 (honest behaviour over polite behaviour — applied to test results here), D-069 (delegated design discipline + canonical-document rule), D-070 (two events of equal importance), D-071 (subsystem audits precede dependent milestones; Phase 9 survey instantiates D-071 retroactively).
- **CLAUDE.md** — current milestone state, MANDATORY behaviour rules, known-flake state.

---

*End of Phase 9 implementation task file. Implementation starts when Clair picks this up. Milestone closes at Commit 7.*  
