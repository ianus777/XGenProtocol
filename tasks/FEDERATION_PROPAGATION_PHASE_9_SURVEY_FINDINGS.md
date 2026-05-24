# Federation Event Propagation Phase 9 Survey — Findings
> **Status**: COMPLETED  
> Version: 1.2  
> Date: May 2026  
> **Last updated**: 2026-05-24 (J-109 — §2.6 Scenario 6 substantive rewrite + new §2.6.1 Phase 7.5 §6 contract-walkthrough sub-block; version bump v1.1 → v1.2. Cause: survey staleness surfaced at Clair's Pre-Commit-3b-2-equivalent verification read — §2.6 sub-item C step 3 claimed F-3 produces `DispatchOutcome::Rejected` with `reason = federation_relationship_missing`, but Phase 7.5 §6 "held-not-bypassed posture" (J-093 / J-094, locked 2026-05-19, shipped 2026-05-20) amended F-3's outcome shape to `DispatchOutcome::HeldPending` with new `disposition = "held_pending"` field on the existing `f3_reject` G2 trace event, recovery path via `drain_pending_by_federation_relationship` arrival hook, and 4007 federation_relationship_timeout sweep (default 180s via `[sync].federation_relationship_timeout_seconds`). Phase 7.5 §5 also extended the F-3 skip set from Lock B1's single `StateFederationAdd` type to three types (adds `StateSpaceCreate` + `StateDmSpaceCreate`). Survey was authored before Phase 7.5 shipped; survey findings v1.1 froze the pre-Phase-7.5 contract; the v1.1 §2.6 line ref `[federation_relationship_integration.rs:166]` was renamed at Phase 7.5 to `peer_without_relationship_held_pending_on_federation_relationship` at `:174`. **Sibling-shape to J-099 audit-doc + design-doc §11 in-place amendments at re-walk Step 2** (canonical-record drift surfaced at dependent-milestone implementation time; second project instance of "canonical document staleness surfaces at dependent-milestone implementation time" pattern — two instances not yet durable pattern, three would be). **D-077 bidirectional sustainability discipline applied at amendment authoring**: pre-amendment grep across federation canonical docs surfaced six file candidates with F-3-contract references; per-surface lock decisions recorded inline in J-109 sub-section 4 — three amended (findings v1.2, Phase 9 task drift correction, survey doc supersession pointer), three preserved unchanged with reasoning (completion runbook records Phase 7's locks correctly for their scope; design doc §6.4.1 already incorporates Phase 7.5; Compound C3 paragraphs acknowledged-stale in dropped-scenario context). **Reading B locked over Reading A + Reading C** per J-109 framing — amend canonical source FIRST in Chat Claude Track 1 atom (sibling-shape to J-107 Track 1 amending canonical records before Clair Track 2 pickup), then Clair picks up against the amended contract. Reading A (write-against-current-behaviour + note-staleness) rejected as no-drift-surface anti-pattern; Reading C (split Scenario 6 into HeldPending + recovery + 4007 timeout tests) rejected as scope-creep over Joe's GO for the original framing (if the amended survey surfaces a natural two-test framing, the call belongs in the survey amendment, not Clair's test-writing decision-space). M15 catalogue row added at topological-sort milestone-close commit per J-101 + runbook §6.3 exact phrasing. M15 covers wire-order non-determinism + causal-DAG-construction lie as one bug class with two layers (determinism layer + causality layer under amended D-076 v1.1); closed by topological-sort milestone J-101 across Commit 2 (determinism layer sort fix) + Commit 2a (causality layer Path B fix). Catches count now 12 of 15 (was 11 of 14 — Scenario 1 second `#[ignore]` lift at J-101 catches M15 in addition to its existing D-075 + D-076 v1.1 determinism locks; "Not caught" set unchanged at M6 + M8 + M13). Previous 2026-05-19: Joe-locked all four §8 open questions: Q1 → 12 scenarios; Q2 → defer G4 to M6; Q3 → option (i) serial-test first, escalate on Phase 9 signal only; Q4 → multi-commit Phase 9. See §8 for locked decisions inline.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Summary

This survey audits the Phase 9 deployment-level surface against the six baseline scenarios locked in [`tasks/FEDERATION_PROPAGATION_COMPLETION.md`](tasks/FEDERATION_PROPAGATION_COMPLETION.md:823) §3.9, eight starter compounds, the failure-mode catalogue, and the observability surface. No code was changed. Findings document follows the §4 structure of [`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY.md`](tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY.md:1).

**Per-scenario verdicts.** All six baseline scenarios are reachable from a `stress-complete`-shape harness with one structural addition (per-peer outbound trace observability — see §7 Gap G2). Scenarios 1, 2, 3, 5 fit the harness as-is. Scenario 4 (validation-asymmetry regression) needs a "forge a federation-channel event" affordance currently only exercised at the `NodeRuntime` level. Scenario 6 needs the §7 Gap G2 observability surface to distinguish the F-3 rejection from any other reject cause; honesty discipline (CLAUDE.md Rule 5 applied to assertions) requires it.

**Compounds — recommended for Phase 9.** Four of eight compounds rated `easy`/`medium`: **C2** (F-5 anti-transitivity at queue depth), **C3** (F-3 rejection during F-1a recovery), **C5** (validation asymmetry under load), **C7** (`continue_from` pagination at boundary). Three compounds rated `medium-hard` but defensible: **C1** (F-10 during F-1b drop), **C6** (F-10 parallel identity arrivals), **C8** (bidirectional simultaneous push). One compound rated `hard`/`defer`: **C4** (Phase 5 reconnect under churn — backoff ladder is wall-clock; deterministic exercise needs a clock-injection seam Phase 5 did not ship). Recommendation: **include C2 + C3 + C5 + C7 in Phase 9; defer C1 + C4 + C6 + C8 to a follow-on `federation-stress` milestone.** Two additional compounds surfaced during the trace and are added below as **C9** (F-3 + drain-time approximation hazard) and **C10** (identity-replicate hook serialisation under lock contention) — both recommended for Phase 9.

**Cross-scenario structural gaps.** Five gaps surfaced (§4 below). Four recommended as Phase 9 precondition (option `a`): **G1** observability — `xgen-node_state.json::peers` is hard-coded to `vec![]` in [`xgen-node/src/app.rs:1775`](xgen-node/src/app.rs:1775); the F-1c registry IS persisted to disk but never reflected in the operator-facing state.json. **G2** observability — no stable structured trace event for "F-1 federation push attempted, peer=X, outcome=Y"; the existing log lines are R14 prose. **G3** harness — no in-process way to assert "B's local fan-out reached client C on B"; the existing CommLog in stress-complete records SENT/RECV at WS boundaries but doesn't observe fan-out outcomes. **G5** harness — no operator-controlled "drop this TCP connection" affordance; scenarios 3, C1, C3 currently model drop by killing the binary. **G4** (audit log emission for F-3 rejection) recommended as deferred (option `c`) — within M6 (new) scope per cross-reference in [`docs/xgen_federation_propagation_design.md`](docs/xgen_federation_propagation_design.md:1095) D-070.

**Flake handling.** **Recommendation: (c) fix both flakes as Phase 9 precondition** — confirms the v2.0 default lean. Flake #1 (`XGEN_LOG` env-var race at [`xgen-common/src/precedence.rs:140`](xgen-common/src/precedence.rs:140)) is code-grounded test-only (production reads `XGEN_LOG` once during `init_logging` — verified by grep, single read site), but `#[serial_test::serial]` is the right fix because four unit tests share process-global state. Flake #2 (`reconnect_with_existing_tip_small_delta_delivered` at [`xgen-node/src/tests/federation_delta_integration.rs:326`](xgen-node/src/tests/federation_delta_integration.rs:326)) is parallelism-sensitive and Phase 9 will *increase* the number of concurrent integration tests that bind `127.0.0.1:0` and stream WS frames — shipping Phase 9 on top of this flake is shipping on a foundation Phase 9 will stress harder. The walk-back permitted at survey §3.3 does not apply: the federation-delta integration test directly overlaps Phase 9's surface.

**Failure-mode catalogue (§6).** 15 bugs catalogued (M15 added at topological-sort milestone J-101). 12 HIGH-severity (federation-pipeline + protocol-integrity). 12 caught by recommended Phase 9 set (6 baseline + 4 compounds + 2 new compounds = 12 scenarios; M15 caught by Scenario 1 second `#[ignore]` lift at J-101). 3 entries marked `not caught` and feed the Client-Side Consequences Audit per CLAUDE.md memory entry "Phase 9 close — milestone shipped" line in [`tasks/FEDERATION_PROPAGATION_COMPLETION.md`](tasks/FEDERATION_PROPAGATION_COMPLETION.md:949).

**Observability audit (§7).** Two gaps already named in `G1` and `G2`. Three additional sub-gaps surfaced in the trace: stable tracing event names for F-1 push and F-3 reject paths (currently free-form log message text, drift risk during refactor); per-peer event flow visibility (C2 hard to assert without it); timing observability for F-10's "identity-arrives-just-before-timeout" axis (current state.json snapshot writes every 5 s — too coarse for sub-second F-10 timing axes).

**Harness convergence.** Mixed-shape Phase 9 is the honest answer. Scenarios 1, 2, 3, 5, 6 reuse the `stress-complete` shape (spawn binaries, real TCP, observe via state files + parsed logs). Scenario 4 + compounds C5 are `NodeRuntime`-level integration tests (forging federation-channel events requires in-process control). Compound C7 lives between: `NodeRuntime`-level for the pagination assertions, deployment-level for the end-to-end. **Per-scenario harness choice is the honest answer; trying to drive every scenario through one shape would weaken the assertions or skip honesty checks.**

**Final Phase 9 scenario count.** Recommend **12 scenarios**: 6 baseline + 4 starter compounds (C2, C3, C5, C7) + 2 trace-surfaced compounds (C9, C10). Adjustable down to 10 if Joe locks defer on C9 and C10; not below 10 without losing the load-bearing honesty checks documented in §6 catalogue.

**Final Phase 9 runtime estimate.** Best-guess (per CLAUDE.md Rule 5 — call it "best-guess" not "measured"): 60-120 s for `cargo test --workspace` with all 12 scenarios + 4 flake fixes serialised + Phase 8's 519 existing tests. Reference points: `stress-complete` reported 14.6 s for 6 scenarios at 4 members (J-059), but stress-complete runs as a bin, not via `cargo test`. Acceptable on the cargo-test path provided each scenario's TCP setup amortises across multiple assertions per scenario.

---

## §2 Per-scenario findings

### §2.1 Scenario 1 — Two-Node federation push smoke

**Owns F-item:** F-1 (Phase 4 federation push); cross-surfaces F-1a (Phase 3 handshake), F-2 (long-lived session).

#### A. Preconditions inventory

1. **Code surfaces.** `NodeRuntime::dispatch_event` at [`xgen-core/src/node/runtime.rs:317`](xgen-core/src/node/runtime.rs:317) (F-4 unified pipeline). `apply_federation_push` at [`xgen-node/src/federation_session.rs:201`](xgen-node/src/federation_session.rs:201) with F-5 guard at line 209. `FederationPeerSenders` registry mirrored on `ClientSenders`. `process_inbound` at [`xgen-node/src/app.rs:1350`](xgen-node/src/app.rs:1350) with F-3 lookup at line 1402-1405. `run_federation_session_post_handshake<S>` for the receiver-side session (R12 register-on-ACTIVE).
2. **Configuration surfaces.** Per-binary `[sync].completion_timeout_seconds` (default 5) + `[sync].batch_size` (default 1000) per CLAUDE.md Phase 1 close. `[logging]` level per D-068. Federation peer URL discovered via `federation.hello.node_endpoint` — no operator pre-configuration. All reachable from a `--service`-launched binary.
3. **State-file surfaces.** `xgen-node_state.json::clients` (per-client connection state). `xgen-node_state.json::peers` — **GAP G1: hard-coded `vec![]` at [`xgen-node/src/app.rs:1775`](xgen-node/src/app.rs:1775).** `xgen-node_federation.json` carries the live `FederationRelationship` + `PeerOperationalRecord` (Phase 5). DAG events persisted under `data_dir/spaces/<space_id>/events/` (via `persist_event` at [`xgen-node/src/app.rs:1411`](xgen-node/src/app.rs:1411)).
4. **Operator controls.** Federation initiated by client-side `xgen-client federate` command (or equivalent batch) per stress-complete pattern at [`xgen-client/src/app.rs:3743`](xgen-client/src/app.rs:3743). Adequate — no missing affordance for this scenario.

#### B. Observation strategy

Primary: **logs**. Parse for F-1 push lines from `apply_federation_push` (currently free-form `tracing::warn!` text — Gap G2). Stable shape would let the scenario assert "B's connection observed Event E via `Inbound::Event(...)` *after* `handshake-ACTIVE timestamp on B*".

Secondary: **state files**. Poll `xgen-node_state.json::spaces[].event_count` on B; assert it incremented by 1 after Alice's post. Coarse — confirms ingest happened, doesn't isolate via-push vs via-dump.

Tertiary: **direct client observation via WebSocket**. Stress-complete's existing pattern: a Bob-client connected to Node B subscribes to the Space; Bob's CommLog records `Inbound::Event` arrival keyed by `event_id`. Strong because it also confirms G3 (local fan-out reached Bob on B).

#### C. Honesty test framing

**Risk:** "B has event E in its DAG" could pass via handshake history dump (F-1a), not via push (F-1). The two paths land events through the same `dispatch_event` and produce identical DAG state.

**Sharper assertion shape:**
1. Establish handshake → record `handshake_active_at_B_ts`.
2. Alice's post on A happens *after* `handshake_active_at_B_ts` (assertion: `alice_post_ts > handshake_active_at_B_ts + ε`).
3. Event E arrives on B's wire after `handshake_active_at_B_ts` (assertion: receipt observed via Bob-client connected to B).

The temporal ordering isolates "B got E via push during the active session", not "B got E in the handshake dump". The original "B receives the event" assertion was the wrong-reason-pass risk; this one fails for exactly one reason.

**Additional honesty check:** assert that A's `apply_federation_push` was invoked once for E (Gap G2 — needs a stable structured trace event), and that the F-5 guard did NOT fire. Distinguishes "push delivered" from "fan-out delivered locally on A only".

#### D. Harness fit

**Reuse `stress-complete` shape.** Scenario 1 is the simplest case of the existing Federation Completeness scenario in [`xgen-client/src/app.rs:3599+`](xgen-client/src/app.rs:3599). Two Nodes spawned as separate binaries, real TCP between them, client connections on both sides — already proven in J-059's 6/6 PASS. No adaptation needed beyond Gap G2 (stable trace events).

#### E. Stress dimensions

| Axis | Weak | Strong (recommended) | Rationale |
|---|---|---|---|
| Event count | 1 | **100** | Volume proves the pipeline; 100 is stress-complete's M5 ballpark and finishes in seconds |
| Event types | message.text only | **5 types: message.text, message.attachment-meta, state.room_create, membership.invite, membership.kick** | Each event family exercises different `dispatch_event` semantic branches at [`runtime.rs:425-455`](xgen-core/src/node/runtime.rs:425) |
| Payload size | small | **mixed: 100B / 10KB / 100KB** | Pipeline isolation per-event; payload size affects serialisation cost not pipeline shape, so small mix is enough |
| Concurrency | Alice serial | **2-3 clients on A posting in parallel** | Surfaces races on A's local-submit → push ordering |

**Recommendation:** 100 events, 5 types, mixed payload, 2 concurrent clients on A. Strong version finishes in ~5-10 s.

---

### §2.2 Scenario 2 — Three-Node anti-transitivity

**Owns F-item:** F-5 (Phase 4, design doc §8.5); cross-surfaces F-1, F-1a, `EventOrigin` enum.

#### A. Preconditions inventory

1. **Code surfaces.** F-5 guard at [`xgen-node/src/federation_session.rs:209`](xgen-node/src/federation_session.rs:209) — `if matches!(origin, EventOrigin::ReceivedViaFederation) { return; }`. `EventOrigin::ReceivedViaFederation` set at [`xgen-node/src/app.rs:1402`](xgen-node/src/app.rs:1402) when `process_inbound` runs with the federation peer origin. Per F-5 lock, the guard is the ONLY origin-aware site on the push path.
2. **Configuration.** Three Node binaries with three distinct `data_dir`s and ports.
3. **State-file surfaces.** Same as Scenario 1 + the `peers: vec![]` Gap G1 means observing "C has no relationship with B" needs `xgen-node_federation.json` parsing on each Node.
4. **Operator controls.** Need to federate A↔B and A↔C but NOT B↔C. Reachable via the existing stress-complete federation pattern with the third Node added.

#### B. Observation strategy

Primary: **direct client observation on C**. Carol-client connected to C asserts she does NOT receive Alice's event E via fan-out within a bounded window. *But* — see honesty risk below — Carol-no-receive could pass for orthogonal reasons.

Secondary: **B-side trace observation via Gap G2** — need a stable trace event "B's `apply_federation_push` was called for E with origin=ReceivedViaFederation, returned at the F-5 guard, zero peers iterated". This is the **source-side** check at the F-5 guard, complementary to the destination-side absence check.

#### C. Honesty test framing

**Risk (named in §3.1.5 of survey task file):** Carol-no-receive could pass because Carol is offline, because C never received the relevant `state.federation_add` for the Space, because of a race, because B's `FederationPeerSenders` doesn't have C (no B↔C session). **Each of these would mask an F-5 bug.**

**Sharper assertion shape:**
1. Setup: A↔B and A↔C federated for Space S; B and C verified online; B↔C explicitly NOT federated.
2. Assertion on B's source side (via Gap G2 stable trace event): `apply_federation_push` was invoked for E with `EventOrigin::ReceivedViaFederation` and short-circuited at the guard. **This is the load-bearing check** — proves the guard fired.
3. Assertion on C's destination side: C's WS connection received E exactly once, via the direct A→C push, NOT via B→C. Distinguish by source: E should appear in C's CommLog with `from=A` (the federation peer node_id at receipt), never with `from=B`.

**Why both assertions.** The source-side check proves the F-5 mechanism is correct. The destination-side check proves the mechanism is correctly wired through the broader system. Removing either weakens the honesty check.

#### D. Harness fit

**Reuse `stress-complete` shape** with three-Node topology already proven by J-059's stress test. The third Node binary spawn is incremental; no novel harness work.

#### E. Stress dimensions

| Axis | Weak | Strong (recommended) | Rationale |
|---|---|---|---|
| Event count | 1 | **100 from A, observe all on B and C** | F-5 is a guard; 100 invocations is sufficient to surface a guard race or off-by-one logic |
| Event flow direction | A→B only | **A→{B,C} parallel, then B→A back, then C→A** | Bidirectional fan-out surfaces guard-state contamination across iteration orders |
| Topology | static | **with mid-test defederation A↔B** | Tests F-5 holds during a peer transition (overlap with C3) |

**Recommendation:** 100 events from A; B and C are observed parallel; one mid-stream A↔B defederation. C9 below subsumes the defederation axis if a separate compound is preferred.

---

### §2.3 Scenario 3 — Drop-and-recover

**Owns F-item:** F-1b (drop on peer down) + F-1a (recovery via tip-exchange); cross-surfaces F-1c (Phase 5 reconnect scheduler).

#### A. Preconditions inventory

1. **Code surfaces.** R14 drop log lines at [`xgen-node/src/federation_session.rs:245`](xgen-node/src/federation_session.rs:245) and `:256`. `FederationRegistry::mark_lost` at [`xgen-core/src/federation/registry.rs:200`](xgen-core/src/federation/registry.rs:200). `scheduler_tick` at [`xgen-node/src/reconnect.rs:112`](xgen-node/src/reconnect.rs:112). `BACKOFF_LADDER_MINUTES = [15, 30, 60, 120]` at [`reconnect.rs:55`](xgen-node/src/reconnect.rs:55) — the wall-clock dimension that makes deterministic exercise hard.
2. **Configuration.** Tick interval is hard-coded at 60 s; initial reconnect delay is 15 min. **GAP G5: scheduler timing is not configurable from any settings file** — Phase 9 deterministic exercise of the reconnect path needs either a clock-injection seam or direct `scheduler_tick` invocation (which is what `reconnect_integration.rs` does for unit-style integration).
3. **State-file surfaces.** `xgen-node_federation.json::peer_records[].lost_connection`, `.next_reconnect_attempt`, `.last_successful_session`. Strong observability — directly readable from disk.
4. **Operator controls.** "Drop the peer" today is "kill the binary" — same `--stop` flag M2 shipped. Reasonable but coarse. **Gap G5 — finer drop affordance (drop just the TCP connection, leave the binary running) would distinguish "session crashed" from "process crashed".** Defer to follow-on per option `c` (modelling drop as binary kill is acceptable for v1).

#### B. Observation strategy

Primary: **`xgen-node_federation.json`** polling. Watch `peer_records[B].lost_connection` flip true after kill; watch `next_reconnect_attempt` populate; after B comes back, watch flip back to false + `last_successful_session` advance.

Secondary: **client observation on Bob**. Bob is a member of Space S on Node B; while B is down, Alice on A posts E1, E2, E3; after B restarts, Bob should observe E1, E2, E3 (in topological order) within a bounded window after the reconnect handshake completes.

Tertiary: **log parsing**. R14 lines for the dropped events on A; F-1a tip-exchange handshake observed on B's startup logs.

#### C. Honesty test framing

**Risk:** "B receives queued events after recovery" could pass via late-push retry rather than via F-1a tip-exchange recovery. Today A has no outbound queue (F-1b explicitly: no queue, drop on peer-down), so there's no late-push path. But Phase 9's test should NOT depend on this internal property being unchanged — a future Phase 11 might add a transient outbound queue.

**Sharper assertion shape:**
1. After A pushed E1/E2/E3 with B down, assert R14 log lines fired on A for all three.
2. After B comes up, assert F-1a tip-exchange handshake happened — `xgen-node_federation.json::peer_records[B].last_successful_session` advances *and* the `state.federation_add` re-emission is NOT in the delta (Lock 2 a-i symmetry — B's tip is present after the first session, so federation_add should NOT re-stream per the assertion in `reconnect_with_existing_tip_small_delta_delivered`).
3. Bob receives E1/E2/E3 in topological order.

**Why this isolates F-1a recovery.** Assertion 1 proves F-1b dropped; assertion 2 proves the handshake completed AND followed the optimistic-no-resend rule; assertion 3 proves delta caught up. Each assertion fails for one reason.

#### D. Harness fit

**Adapt `stress-complete` shape.** The scenario needs a "kill binary B, observe state, restart binary B" loop that stress-complete's current flow doesn't have. Practical adaptation: spawn B as a `tokio::process::Child`, `child.kill().await`, sleep, respawn with same `data_dir`. The Phase 5 reconnect-integration test uses `scheduler_tick` directly which is the wrong shape for Phase 9 — Phase 9 wants the binary level.

**Important harness affordance needed:** binary needs to restart with the same `data_dir` and read back its `xgen-node_federation.json`. Already supported (Phase 5 loads at startup) but the test should explicitly verify by reading the file pre- and post-restart.

#### E. Stress dimensions

| Axis | Weak | Strong (recommended) | Rationale |
|---|---|---|---|
| Queued event count | 1 | **5-10** | F-1b is a guard; 5-10 events confirms the guard handles back-to-back drops |
| Drop moment | idle | **{idle, mid-handshake, mid-stream, mid-pushqueue-drain}** | Bugs hide in the moments not in the volume — bug catalogue entry M2 in §6 |
| Recovery wait | tight | **wait for `next_reconnect_attempt` to fire naturally OR manual reconnect via Phase 5 scheduler tick** | Tests both paths |
| Cycles | 1 | **2 sequential drop-recover cycles** | Verifies `mark_lost` idempotency under cycling, partial coverage of C4 |

**Recommendation:** 10 events queued, drop mid-stream (the most realistic operator scenario), 2 cycles. The drop-during-handshake axis is the hardest to engineer — flag as optional.

---

### §2.4 Scenario 4 — Validation asymmetry regression

**Owns F-item:** F-4 (Phase 2 pipeline unification); cross-surfaces F-7 (pagination — handshake dump is the most-stressful entry point for asymmetry).

#### A. Preconditions inventory

1. **Code surfaces.** `dispatch_event` at [`runtime.rs:317`](xgen-core/src/node/runtime.rs:317) — the unified pipeline. Validation core inside `validate_event` at [`xgen-core/src/message/exchange.rs`](xgen-core/src/message/exchange.rs) (signature, timestamp, predecessor, DAG, membership). Origin-uniform per F-4 — same code path for `LocallySubmitted` and `ReceivedViaFederation`.
2. **Configuration.** None scenario-specific.
3. **State-file surfaces.** None — assertions live in the rejection path observability.
4. **Operator controls.** This scenario requires forging an event with a bad signature and getting Node B to ingest it via a federation channel. **Real-binary harness limitation:** the only producer of federation-channel events in production is another Node's `apply_federation_push` — which would never produce a forged event because Alice's signature is computed correctly there. **A direct-WS forgery harness is needed.**

#### B. Observation strategy

Primary: **B's rejection log lines**. `process_inbound` emits `tracing::error!` with `reason = %reason` at [`xgen-node/src/app.rs:1441`](xgen-node/src/app.rs:1441) — Gap G2 would standardise this as a stable trace event.

Secondary: **B's DAG state** — assert E never landed in B's `xgen-node_state.json::spaces[].event_count`.

Tertiary: **B's metrics** (none today) — would be valuable for Phase 9+; flag as a follow-on observability surface.

#### C. Honesty test framing

**Risk:** "Forged event rejected" could pass because B was offline, because the federation relationship doesn't exist, because of a network glitch, because the event was never delivered.

**Sharper assertion shape:**
1. Setup: A↔B federated successfully (verified via Gap G1 surface or by handshake-complete log).
2. Forge an event E with bad signature on the wire from A's connection to B.
3. Assert B's rejection log emits the *specific* validation-error reason (signature-verification-failed) — distinct from F-3's `federation_relationship_missing` or HeldPending's `missing_identity`.
4. Assert the rejection happened *for* E (envelope `event_id` per D-070, when M6 Phase 2 wires it — for Phase 9, parse the event_id from the log).

**Why this isolates F-4.** The error-code namespace tells the test which path rejected. F-4's validation core rejects on signature → distinct code; F-3 rejects on relationship → `federation_relationship_missing`; F-10 buffers → `HeldPending`. Distinct rejection causes are distinctly observable via the reason field.

#### D. Harness fit

**Diverge from `stress-complete` shape.** This scenario needs a "send a federation-channel `FederationMessage::Event` with a forged signature" affordance that no production code path produces. Implementation choices:

- **In-process forging**: write a test that spins up Node B as `NodeRuntime` in-process, opens a federation session via the test, then sends a forged `wire::types::FederationMessage::Event` directly through the WS connection.
- **`NodeRuntime`-level test**: skip TCP entirely. Construct a forged event, call `runtime.dispatch_event(forged, EventOrigin::ReceivedViaFederation, Some(peer_id))`. Lighter and faster, but misses the wire layer.

**Recommendation: NodeRuntime-level test (the lighter option).** F-4's surface is `dispatch_event`; the WS layer adds nothing to the assertion. Pair with one `stress-complete` integration sanity check that one forged event over real TCP gets rejected end-to-end.

#### E. Stress dimensions

| Axis | Weak | Strong (recommended) | Rationale |
|---|---|---|---|
| Forgery variants | bad-signature | **{bad-signature, mutated-sender, mutated-event_id, malformed-prev_events, future-timestamp, past-timestamp}** | F-4's surface has multiple reject paths; one variant per path |
| Reject paths | step 12 only | **steps 8-13 enumerated** | Coverage; bugs hide in the rare reject paths |
| Event families | message.text | **5 families: message, membership.join, membership.kick, state.federation_add (skip-F-3 path), state.room_create** | F-4's whole point is family-uniformity |
| Volume | 1 | **20 mixed forged + 100 valid mixed in random order** | Catches catalogue bug M10 (asymmetry leak under load — overlaps C5) |

**Recommendation:** 6 forgery variants × 5 event families = 30 NodeRuntime-level forgery tests. Plus 1 end-to-end stress-complete-shape test with 20 forged + 100 valid mixed. **C5 below is the strong version** — Phase 9 should land C5 in place of an independent Scenario 4 stress-only test.

---

### §2.5 Scenario 5 — Unknown-signer first-contact

**Owns F-item:** F-10 (Phase 6 HeldPending generalisation); cross-surfaces F-10a (30 s timeout), 4006 error code.

#### A. Preconditions inventory

1. **Code surfaces.** `ValidationOutcome::HeldPending { missing_predecessors, missing_identity }` at [`xgen-core/src/message/exchange.rs`](xgen-core/src/message/exchange.rs). `PendingBuffer::waiting_for_identity` index at [`xgen-core/src/dag/pending.rs`](xgen-core/src/dag/pending.rs). `NodeRuntime::drain_pending_by_identity` at [`runtime.rs:551`](xgen-core/src/node/runtime.rs:551). `handle_identity_replicate_msg` hook at [`app.rs:1592`](xgen-node/src/app.rs:1592). Timeout sweep at [`app.rs:456-491`](xgen-node/src/app.rs:456) with branch on `missing_predecessors.is_empty()` for the 4002/4006 selection. `PENDING_TIMEOUT_SECS = 30`.
2. **Configuration.** None scenario-specific. The 30 s timeout is hard-coded.
3. **State-file surfaces.** `xgen-node_state.json::pending_identity_replication` at [`app.rs:1777`](xgen-node/src/app.rs:1777) — Phase 6 added it specifically for F-10 observability. Strong.
4. **Operator controls.** Need to delay Identity replication. Reachable by federating A↔B *with* a Space-member Bob whose Identity record is on A but NOT yet replicated to B. Achievable in stress-complete-shape harness by sending a federation push BEFORE the identity-replicate message lands on B.

#### B. Observation strategy

Primary: **`xgen-node_state.json::pending_identity_replication`** polling. Should increment when the federation push lands and Bob's Identity is missing; should decrement when Bob's Identity replicates and `drain_pending_by_identity` fires.

Secondary: **log lines**. 4006 timeout line at [`app.rs:484`](xgen-node/src/app.rs:484) for the never-arrives case. F-4a's `tracing::debug!("event buffered — waiting for unknown prev_events")` at [`app.rs:1432`](xgen-node/src/app.rs:1432) for the hold path.

Tertiary: **Bob-client observation**. The released event should eventually surface to Bob via fan-out after the identity arrives.

#### C. Honesty test framing

**Risk:** "Event eventually ingests after Identity replication" could pass via timeout-retry rather than via F-10's identity-arrival hook. (Note: there is no timeout-retry today — the timeout DISCARDS, doesn't retry. But the assertion should still be sharp enough that a regression introducing a sweep-retry path would not silently pass.)

**Sharper assertion shape:**
1. Assert event is in HeldPending state — observe `pending_identity_replication = N` where N > 0 and increases by 1.
2. Trigger Identity replication for Bob's Identity to B (e.g. via `xgen-client identity-replicate`).
3. Assert `pending_identity_replication` decremented within X ms of the identity-replicate hook firing — proves the hook ran, not a periodic sweep. X ≤ 100 ms (the hook is synchronous within `handle_identity_replicate_msg`).
4. Assert the released event landed in B's DAG and reached Bob-client.

**Why this isolates F-10.** Step 3's timing window is sub-second; the sweep task runs every 5 s. If the assertion passes within 100 ms, the hook fired; if it took longer than 100 ms but less than 5 s, the test fails for a known shape; if it took > 5 s, the sweep-retry was the culprit (which today doesn't exist — regression signal).

#### D. Harness fit

**Reuse `stress-complete` shape.** The scenario fits the existing pattern with one addition: stress-complete needs a "send the federation Event before sending the identity Replicate" sequencing. Achievable by spawning Bob's connection on B *first*, then on A having Bob's federation send happen before the replicate.

#### E. Stress dimensions

| Axis | Weak | Strong (recommended) | Rationale |
|---|---|---|---|
| Identity arrival timing | 1 s before timeout | **{1 s before timeout, 100 ms before timeout, 1 ms before timeout, 1 ms after timeout}** | F-10's 30 s window edge cases; bugs hide in the moments |
| Predecessor co-missing | identity-only missing | **{identity-only, predecessor-only, both, both-with-identity-first-arrival}** | Phase 6 already has these as `heldpending_identity_integration.rs` tests; Phase 9 promotes the strongest cases to binary level |
| Volume | 1 event held | **5 events held parallel** | Surfaces buffer-coordination bugs; tractable size |
| Identity duplicates | single arrival | **identity-replicate fires twice for same Bob** | Catalogue bug M9 — double-drain on parallel arrivals (C6) |

**Recommendation:** 4 timing variants × 4 missing variants = 16 NodeRuntime-level cases (already covered by Phase 6 tests except 1 ms timing). One binary-level test covers timing-1ms-before-timeout. The 1 ms after timeout case is the load-bearing honesty check — MUST not release.

---

### §2.6 Scenario 6 — Federation-relationship deferral via HeldPending (post-Phase-7.5 §6 contract; J-109 amendment)

**Owns F-item:** F-3 (Phase 7 verification gate, as amended by Phase 7.5 §6 held-not-bypassed posture); cross-surfaces `SpaceState.federation_nodes`, Phase 7 Lock B1 + Phase 7.5 §5 extended skip set, the third HeldPending trigger condition (federation-relationship), the `drain_pending_by_federation_relationship` arrival hook, the 4007 federation_relationship_timeout sweep, and the `pending_federation_relationship: usize` counter on `NodeState`.

**See §2.6.1 below for the full Phase 7.5 §6 contract walkthrough — single source of truth for the assertion shapes herein.**

#### A. Preconditions inventory

1. **Code surfaces.** F-3 check + Phase 7.5 §5 extended skip set + Phase 7.5 §6 HeldPending buffer at [`runtime.rs:514-555`](xgen-core/src/node/runtime.rs:514). Phase 7.5 §6 federation-relationship arrival hook `drain_pending_by_federation_relationship` at [`runtime.rs:~800`](xgen-core/src/node/runtime.rs) (function in same file, see §2.6.1 for full file:line refs). Federation channel origin propagation in `process_inbound` at [`app.rs:1402-1406`](xgen-node/src/app.rs:1402). The G2 stable trace event `event = "f3_reject"` with new `disposition = "held_pending"` field at [`runtime.rs:540-548`](xgen-core/src/node/runtime.rs:540) (existing trace event name retained per Phase 7.5 P7.5-D Joe-lock; disposition field added). 4007 timeout sweep at [`app.rs:~520`](xgen-node/src/app.rs) (sibling to 4006's sweep block; see §2.6.1 for precedence rules at multi-trigger timeouts).
2. **Configuration.** New `[sync].federation_relationship_timeout_seconds` config field (Phase 7.5 P7.5-C Joe-lock); default **180 seconds** (raised from draft 120s during Phase 7.5 design walk to give operators meaningful headroom over medium-WAN-degraded bootstrap). F-4a's 30s and F-10a's 30s remain unchanged. Per-trigger timeouts make the 4007 timing axis distinct from 4006's axis.
3. **State-file surfaces.** `xgen-node_state.json::pending_federation_relationship` counter (Phase 7.5 P7.5-D Joe-lock; sibling to Phase 6's `pending_identity_replication`) at [`xgen-common/src/state.rs`](xgen-common/src/state.rs) with `#[serde(default)]` for forward-compat. Increments when F-3 defers an event via HeldPending on the third trigger; decrements when the arrival hook fires OR the 4007 timeout sweep fires. `xgen-node_federation.json::peer_records` shows X as a peer if a session was established. `xgen-node_state.json::spaces[].event_count` MUST NOT increment by E's count while E is HeldPending; MUST increment by E's count after federation_add arrival fires the drain hook and E re-validates cleanly.
4. **Operator controls.** Need a Node X that federates with B at the session level but is NOT a member of any of B's Spaces' `federation_nodes`. Achievable: spawn X, complete federation handshake with B, but never include X in any Space's `state.federation_add`. Then X attempts to push an event for Space S that B hosts but X is not federated for. Setup is unchanged from pre-Phase-7.5; only the outcome shape (HeldPending vs Rejected) changes. **Recovery axis** (new under Phase 7.5): for the recovery-path assertions, the harness needs a way to land `state.federation_add` on B at a controlled moment after the deferred event — reachable by having A push the federation_add subsequently in the same session-arc, OR by direct test-driven ingest at `NodeRuntime` level for the unit-test sibling.

#### B. Observation strategy

Primary observability splits into three sub-paths corresponding to the three possible outcomes under Phase 7.5 §6:

- **Defer path (load-bearing first observation):** The G2 trace event `event = "f3_reject"` fires with `disposition = "held_pending"`, `reason = "federation_relationship_missing"`, `peer_node_id = X`, `space_id = S`, `event_id = E.event_id`. The `pending_federation_relationship` counter on `xgen-node_state.json` increments by 1. The event is NOT in B's DAG yet (`xgen-node_state.json::spaces[S].event_count` unchanged). B's local fan-out is NOT invoked for E (no `fanout_delivered` G3 trace event with E's event_id).
- **Recovery path:** When `state.federation_add` for (X, S) lands on B — either via A's subsequent push or via direct ingest — `drain_pending_by_federation_relationship` fires. The deferred event E re-enters `dispatch_event` via the drain hook; F-3 now passes (X is in `federation_nodes` for S); E ingests cleanly. `pending_federation_relationship` decrements by 1 within ~100ms of federation_add ingestion. B's local fan-out IS invoked for E.
- **Timeout path:** If no `state.federation_add` for (X, S) arrives within 180s (or whatever `[sync].federation_relationship_timeout_seconds` is configured to), the 4007 federation_relationship_timeout sweep emits the timeout log line and discards E from the buffer. `pending_federation_relationship` decrements by 1. E never enters B's DAG. (Phase 9 uses a config override to a short timeout — ~3-5s typical for test runs — to keep the timeout-path scenario tractable.)

Secondary: **`peer_records`** shows X as `lost_connection: false` (session is fine across all three sub-paths). The structural assertion "deferral happens at F-3, not at session" stands on `peer_records` evidence.

Tertiary: **assertion on X's side** — if M6 Phase 2 ships the envelope-level `event_id` signal, X may observe the deferral wire-side. Not available in Phase 9 (M6 is blocked behind this milestone). The disposition field's `held_pending` value will inform M6 Phase 2's eventual signal-shape design (whether deferral surfaces wire-side or stays Node-internal).

#### C. Honesty test framing

**Risk under Phase 7.5 §6 (revised from v1.1 framing):** "Event from peer X is deferred via HeldPending" could pass for many wrong reasons — X has no events to send, X's connection drops, the event itself is malformed, the event lands in HeldPending on a different trigger (F-4a predecessor missing, F-10 Identity missing). The honesty assertion must isolate the **third trigger** (federation-relationship) specifically.

**Sharper assertion shape (Phase 7.5 §6 contract):**
1. Assert handshake A_session_X↔B completes (`peer_records` shows X active, session is fine).
2. Assert event E from X to B is structurally valid (would pass F-4 in isolation; missing only the federation-relationship gate, not predecessor or Identity).
3. Assert `event = "f3_reject"` G2 trace event fires with **all four fields exactly**: `disposition = "held_pending"` (load-bearing distinguisher from any future `disposition = "rejected"` permanent-reject path Phase 7.5 reserved but does not emit at v1); `reason = "federation_relationship_missing"` (unique to F-3 path); `peer_node_id = X`; `space_id = S`. The `disposition` field is what distinguishes Phase 7.5's held-not-bypassed posture from any future-tightening permanent-reject path — asserting on it is what makes the test a load-bearing regression lock for the Phase 7.5 P7.5-B Joe-lock specifically.
4. Assert `xgen-node_state.json::pending_federation_relationship` increments by 1 within ~100ms of the f3_reject trace event fire (sub-second window proves the counter is wired to the buffer-add site, not to a periodic sweep).
5. Assert event E is NOT in B's DAG (`xgen-node_state.json::spaces[S].event_count` unchanged) AND B's local fan-out NOT invoked for E (no `fanout_delivered` G3 trace event with E's event_id).
6. **Recovery-path assertion (new under Phase 7.5):** Push a valid `state.federation_add` for (X, S) to B. Within ~100ms: `drain_pending_by_federation_relationship` fires (observable via a debug-level trace event TBD per §2.6.1's source-code reference); E re-dispatches; F-3 now passes; E ingests; `pending_federation_relationship` decrements by 1; `event_count` increments by 1; B's local fan-out IS invoked for E. The recovery-path assertion is what proves Phase 7.5 §6 is held-not-bypassed (deferred, not weakened) — if the recovery-path assertion fails but step 3-5 pass, the buffer is a black hole, not a holding cell.
7. **(Optional, harness-cost-permitting) Timeout-path assertion:** With a test-configured short timeout (~3-5s), assert that if no `state.federation_add` for (X, S) arrives within the window, 4007 federation_relationship_timeout sweep fires; `pending_federation_relationship` decrements by 1; E never enters B's DAG. The timeout-path assertion is what proves the 4007 code path is reachable; without it, the timeout sweep could regress silently.

**Why this isolates F-3 under Phase 7.5 §6.** Three properties layer:
- The `f3_reject` G2 trace event name with `disposition = "held_pending"` is unique to the F-3 third-trigger path.
- The `reason = "federation_relationship_missing"` string distinguishes from any future F-3 sub-paths (per Phase 7.5 P7.5-D reserved future disposition values).
- The `pending_federation_relationship` counter is the third-trigger-specific observability surface (sibling to `pending_identity_replication` for F-10's second trigger and the F-4a predecessor-trigger's counter).

Together, an event hitting all three properties is unambiguously in F-3's third-trigger HeldPending state, not in F-4a or F-10's parallel paths.

**Lock B1 + Phase 7.5 §5 honesty check.** A second test verifies the **extended skip set** (Phase 7.5 §5 widened Lock B1's single `StateFederationAdd` to three types adding `StateSpaceCreate` + `StateDmSpaceCreate`). Three sibling sub-assertions:
- X sends `state.federation_add` (Lock B1, Phase 7 original): outcome MUST NOT contain `disposition = "held_pending"` AND `reason = "federation_relationship_missing"` (the conjunction is what distinguishes "skipped F-3" from "deferred via F-3"). Outcome may legitimately be HeldPending on F-10 Identity trigger if X's Identity isn't on B yet — but the disposition+reason conjunction is the load-bearing negative assertion. The Phase 7.5 §6 contract recovery path makes the original B1 skip-rule precedent (negative assertion against `federation_relationship_missing`) sharper rather than obsolete — the negative assertion now distinguishes "never entered F-3 deferral" from "entered and then drained," which the v1.1 framing could not.
- X sends `state.space_create` (Phase 7.5 §5 new skip member): same negative-assertion shape.
- X sends `state.dm_space_create` (Phase 7.5 §5 new skip member): same negative-assertion shape.

Narrowness regression check: X sends `state.room_create` against a Space that exists locally but where X is not federated. This event type is NOT in the Phase 7.5 §5 skip set (the skip discriminator is "creates the Space it references", and room_create references an existing parent Space). Outcome MUST be `disposition = "held_pending"` on the federation-relationship trigger — i.e. F-3 still fires for room_create. The Phase 7.5 unit test `f3_does_not_skip_state_room_create` at [`runtime.rs:1006-1042`](xgen-core/src/node/runtime.rs:1006) is the upstream regression lock; Phase 9 elevates the same narrowness check to the deployment level over real TCP.

Line-ref correction from v1.1: the renamed Phase 7.5 integration test `peer_without_relationship_held_pending_on_federation_relationship` lives at [`federation_relationship_integration.rs:174`](xgen-node/src/tests/federation_relationship_integration.rs:174) (was `:166` in v1.1 under the old `peer_without_relationship_rejects_with_federation_relationship_missing` name). The Phase 7 precedent `state_federation_add_skips_f3_check` at [`federation_relationship_integration.rs:279`](xgen-node/src/tests/federation_relationship_integration.rs:279) still exists and the negative-assertion pattern is unchanged — the precedent survives the Phase 7.5 amendment because Lock B1's skip-rule survives (now widened, not removed).

#### D. Harness fit

**Reuse `phase9_harness::InProcessNode` shape** for the main deferral + recovery test (per Phase 9 §3.0 Lock #2 uniform-in-process-harness decision). The Phase 7.5 §6 recovery axis adds a fourth sequencing step to the harness setup:
1. Spawn 2 Nodes (A, B); complete handshake; do NOT federate any Space.
2. A creates Space S; X (third node) attempts to push event E for S — expect HeldPending defer.
3. A pushes `state.federation_add(X, S)` to B — expect drain hook fires, E ingests.
4. (Optional, separate test or test variant) Repeat steps 1-2 but with X's connection idle; wait `[sync].federation_relationship_timeout_seconds` (configured short for tests); expect 4007 timeout sweep fires.

The sequencing requires the harness to expose enough timing primitives to assert ~100ms windows around hook-fire vs sweep-fire — same primitive set as Scenario 5 (Phase 6 F-10 timing windows). No new harness work beyond what Scenario 5 also needs.

The Phase 7.5 unit tests at [`runtime.rs:1006`](xgen-core/src/node/runtime.rs:1006) onwards (`f3_does_not_skip_state_room_create`, `f3_fail_buffers_event_on_federation_relationship_trigger`, `drain_pending_by_federation_relationship_drains_buffered_events`, `drain_pending_by_federation_relationship_idempotent`) cover the contract at the `NodeRuntime` level. Phase 9 elevates the deferral + recovery assertions to the deployment level over real TCP, and adds the timeout-path assertion which has no `NodeRuntime`-level sibling (4007 sweep is xgen-node-side).

#### E. Stress dimensions

| Axis | Weak | Strong (recommended) | Rationale |
|---|---|---|---|
| Relationship state at the moment of push | none | **{never-existed, was-defederated-mid-session, asymmetric (A→B yes, B→A no)}** | F-3 surface — the three operationally-possible states (unchanged from v1.1) |
| Event type for deferral path | message.text | **state.room_create AND membership.* AND message.*** | All three event families that the F-3 third-trigger path treats uniformly (state.space_create + state.dm_space_create + state.federation_add are in the Phase 7.5 §5 skip set; room_create is the narrowness regression lock for the skip set) |
| Event type for skip-rule path | state.federation_add only | **state.federation_add + state.space_create + state.dm_space_create** | Phase 7.5 §5 widened the skip set from one type to three; all three need the negative-assertion treatment |
| Recovery timing | not tested | **{federation_add arrives 1s before 4007 timeout, 100ms before, 1ms before, 1ms after}** | 4007 window edge cases sibling to F-10a's 30s edge cases at §2.5 (Phase 7.5 P7.5-C raised this trigger's timeout to 180s default; bugs hide at the boundary) |
| Volume | 1 | **20 from X** | Surfaces guard contamination under load (unchanged from v1.1) |
| Drain-time approximation | not tested | **HeldPending event whose peer X gets defederated before drain** | C9 below; catches bug catalogue M11 (unchanged from v1.1) |

**Recommendation:** 3 relationship-state variants × 3 event families × 4 recovery-timing variants = 36 scenarios at NodeRuntime level (mostly covered by Phase 7.5 unit tests at `runtime.rs:1006+`). At Phase 9 binary level, compress to the load-bearing cases:
- 1 deferral + recovery test (never-existed relationship, message.text, federation_add arrives 1s before timeout) — the canonical case.
- 1 narrowness regression test (room_create, never-existed relationship, no recovery push) — proves the skip set is narrow.
- 1 timeout-path test (never-existed relationship, message.text, no recovery push) — proves 4007 sweep fires.
- 3 Lock B1 + Phase 7.5 §5 negative-assertion tests (one per skip-set member).

The 20-volume axis can fold into C2 (anti-transitivity-at-load) since both involve queue-depth on B's outbound. The drain-time approximation axis is C9's territory.

---

### §2.6.1 Phase 7.5 §6 contract walkthrough — single source of truth for §2.6 assertions (J-109 amendment)

**Why this sub-block exists.** §2.6 above asserts against a contract that was authored after the survey's original v1.0 close. Rather than scatter Phase 7.5-specific references throughout §2.6's narrative (the v1.1 → v1.2 transition's smallest-change shape), this sub-block consolidates the contract once, near §2.6, with file:line refs to the live code. Future test authors implementing Scenario 6 should read this sub-block as the contract reference, not infer from §2.6's narrative alone. **The ultimate source of truth is the code at `xgen-core/src/node/runtime.rs:514-555` plus its sibling drain hook and timeout sweep; this sub-block is a snapshot summary, not the canonical contract.**

**Phase 7.5 §6 contract elements** (sibling-shape to how §2.5 references the Phase 6 F-10 contract):

1. **Outcome enum.** Pre-Phase-7.5: F-3 fail emitted `DispatchOutcome::Rejected("federation_relationship_missing: ...")`. Post-Phase-7.5: F-3 fail emits `DispatchOutcome::HeldPending` AND adds the event to `PendingBuffer` with a secondary index entry on the **federation-relationship trigger** keyed by `(peer_node_id, space_id)`. The shift from permanent reject to deferred buffer is the "held-not-bypassed posture" Joe-lock at Phase 7.5 P7.5-B. F-3 is NOT weakened — it is deferred until its data source (`SpaceState.federation_nodes`) is populated. The buffer is a holding cell, not a back-channel: the event is not accepted into storage, not fanned out, not visible downstream until F-3 passes on re-validation.

2. **G2 trace event shape (Phase 7.5 P7.5-D Joe-lock).** The existing `event = "f3_reject"` G2 trace event name is **retained** — not renamed — and gains a new `disposition` field. Three reasons for retention over rename per P7.5-D: (a) Phase 9 Commits 1+2 are already shipped, renaming would touch trace plumbing; (b) the name `f3_reject` is still accurate for the vast majority of fires — the held-pending case is the narrow new path under Phase 7.5; (c) "reject" in trace-event vocabulary often means "did not accept on first try" rather than "permanently refused", and the disposition field clarifies which variant. **Current disposition values:** `held_pending` (Phase 7.5's narrow new path, the only value v1 emits). **Reserved-but-not-emitted disposition value:** `rejected` (reserved for any future permanent-reject path; v1 does NOT emit this value). Asserting against the `held_pending` value specifically (rather than asserting the trace event fires at all) is what makes a Phase 9 test a load-bearing regression lock for the Phase 7.5 P7.5-B Joe-lock.

3. **PendingBuffer secondary index.** Phase 7.5 adds a third secondary index on `PendingBuffer` for the federation-relationship trigger, keyed by `(peer_node_id, space_id)`. Sibling to Phase 4's predecessor-trigger index (keyed by `event_id`) and Phase 6's Identity-trigger index (keyed by `identity_id`). The index supports O(1) lookup of buffered entries when an arrival hook fires. Observability via `PendingBuffer::pending_federation_relationship_count()` and per-`NodeState::pending_federation_relationship: usize` counter (sums across Spaces). The counter has `#[serde(default)]` for forward-compat with pre-Phase-7.5 state files.

4. **Drain hook — `drain_pending_by_federation_relationship`.** Phase 7.5 §6 federation-relationship arrival hook. Function at `xgen-core/src/node/runtime.rs` — see source for exact line (search for `pub fn drain_pending_by_federation_relationship`). Fires from `xgen-node::app::process_inbound` after every successful `state.federation_add` ingestion. **Idempotent**: subsequent fires for the same `(peer, space)` pair are no-ops because the secondary index is drained on first fire (mirrors F-10's Identity-arrival hook semantics; no "already-drained pairs" tracking required). Cross-Space fan-out via iteration over all Spaces' `PendingBuffer`s, same pattern as `drain_pending_by_identity` (Phase 6 Lock A2; deployment scale ~1-10 Spaces per Node makes the iteration cost negligible). Released events re-enter `dispatch_event` through the same shape as predecessor-arrival drain; F-3 re-check is implicit (the (peer, space) is now in `federation_nodes` by definition — federation_add just ingested), so re-dispatch passes F-3 cleanly. **Phase 7.5 persistence-amendment Q3 (J-108 milestone close):** the drain hook now returns `Vec<Event>` of drained Accepted events for caller-side persistence by `process_inbound` — see `DispatchOutcome::Accepted { additional_persisted, .. }` at the dispatcher and the runtime.rs documentation at the drain hook function.

5. **Timeout sweep — 4007 federation_relationship_timeout.** Phase 7.5 P7.5-C Joe-lock introduced new error code 4007 federation_relationship_timeout. **Default 180 seconds** via new config field `[sync].federation_relationship_timeout_seconds` (raised from draft 120s during the Joe-lock walkthrough to give meaningful headroom over the medium-WAN-degraded case before operators need to discover and tune). F-4a's 30s and F-10a's 30s remain unchanged — the 180s value reflects that bootstrap streams can be large (a Space with months of history may take tens of seconds to deliver) and `state.federation_add` arrival is bounded by stream delivery rather than independent async pipeline. **Precedence at multi-trigger timeout** (Phase 7.5 P7.5-B sub-rule extending F-10's predecessor-code-wins): if a HeldPending entry times out with multiple missing dependencies, the emitted error code follows: predecessor (`4002`) > federation-relationship (`4007`) > Identity (`4006`). Rationale: federation-relationship is the most upstream blocker in the dependency chain because Identity replication is conditionally downstream of federation establishment (Identity events themselves flow over federation transport). Reporting the most upstream blocker directs the operator to the right diagnostic question. Verbatim code-comment block at the timeout-emit site, sibling to Phase 6's block at the same site.

6. **Phase 7.5 §5 extended skip set.** Pre-Phase-7.5 (Phase 7 Lock B1): F-3 was skipped only for `EventType::StateFederationAdd` (the relationship-establishing event itself — chicken-and-egg). Phase 7.5 §5 widened the skip set to three types: `StateFederationAdd` + `StateSpaceCreate` + `StateDmSpaceCreate`. The discriminator is **"creates the Space it references"**, not "DAG root". `state.room_create` is a DAG root but references an existing parent Space — NOT in the skip set; F-3 still fires for room_create. The Phase 7.5 §5 narrowness is load-bearing for Scenario 6's narrowness regression assertion: any future widening to room_create would silently weaken F-3's coverage of the "federated peer creates rooms in Spaces it shouldn't have access to" attack surface.

7. **Two-stage cascade case.** If `state.federation_add` itself enters HeldPending on F-10's Identity trigger (signer's Identity unknown), the cascade resolves naturally without special handling: Identity arrival fires F-10's hook, federation_add re-validates and ingests, federation_add ingestion fires P7.5-B's hook, dependent events drain. Each hook responds to its own trigger; no cross-hook coordination is needed. The two-stage cascade is itself a Phase 7.5 design surface (recorded in Phase 7.5 spec docs); Phase 9 tests it implicitly via the recovery-path assertion's setup if X's Identity arrives mid-test, explicitly via Compound C9 if Joe wants the cascade as its own scenario.

8. **Out-of-scope decisions (Phase 7.5 explicit non-decisions).** Sender-side stream reordering (Option Y from Phase 7.5 design — mint `state.federation_add` with `prev_events = [state.space_create.event_id]` so it lands as a structural sibling of the Space root) was **rejected**: introduces multi-tip-per-Space DAGs as a normal feature, propagates implications through F-1a / F-6a wire shapes, non-reversible. Session-flag bootstrap window (Option X.b — tracking per-(peer, space) handshake-in-progress state and bypassing F-3 during the window) was **rejected**: weakens F-3 to pre-audit semantics during exactly the moment trust most matters. Phase 7.5 uses the held-not-bypassed posture (P7.5-B) instead.

**Source-of-truth pointers.** Code: `xgen-core/src/node/runtime.rs:514-555` (F-3 buffer-add site) + `xgen-core/src/node/runtime.rs` `drain_pending_by_federation_relationship` (drain hook; ~line 800) + `xgen-node/src/app.rs` ~line 520 (4007 timeout sweep) + `xgen-common/src/state.rs` (`pending_federation_relationship` counter). Spec: `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` (COMPLETED v1.0) §6 + `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` (COMPLETED v1.0) Commit 3 sub-section. Canonical design doc: `docs/xgen_federation_propagation_design.md` §6.4.1 P7.5-B + P7.5-C + P7.5-D paragraphs. JOURNAL: J-103 retrospective entry for the milestone close + J-108 for the persistence amendment.

---

## §3 Compound scenarios

### §3.1 C1 — F-10 unknown-signer arriving during F-1b drop

**What it tests.** Peer A pushes event with unknown signer Bob's Identity to peer B → B buffers as HeldPending → A's connection to B drops mid-stream → reconnect happens via F-1a → does HeldPending survive? Does it resolve? Does F-10's 30 s timeout fire correctly when wall-clock spans the drop period?

**What bug it would catch.** Catalogue bug M3 (HeldPending entry survives identity arrival but `drain_pending_by_identity` doesn't fire) AND M6 (Phase 5 reconnect leaks `tokio::spawn`). Specifically, if the F-1a recovery handshake re-streams Bob's join event but B's HeldPending still holds the prior version, the test catches a duplicate-ingest path.

**Cost estimate.** **Medium-Hard.** Requires the binary kill/restart pattern from Scenario 3 AND the identity-replicate delay pattern from Scenario 5. The 30 s F-10 timeout makes the assertion window long; running 20 of these in a `cargo test` is impractical without clock injection.

**Recommendation:** **Defer to follow-on milestone.** The two component bugs (M3 and M6) are caught by Scenario 5 and Scenario 3 respectively in their own surface; the compound's incremental coverage doesn't justify the 30 s wall-clock cost.

---

### §3.2 C2 — F-5 anti-transitivity under push queue depth

**What it tests.** A pushes 100 events to B and C in rapid succession; B and C have no relationship. Verify B's outbound push to anyone *ever* contains an event with `EventOrigin::ReceivedViaFederation`.

**What bug it would catch.** Catalogue bug M5 (`EventOrigin::ReceivedViaFederation` leaks into local fan-out path → echo loop). Today the guard is correct at the source; the failure mode is regression — a future code change adding a queue or batch handler might re-tag events as `LocallySubmitted` somewhere along the way.

**Cost estimate.** **Easy.** Three Nodes (already proven), 100 events (no novel infra), assertion via Gap G2 source-side trace event.

**Recommendation:** **Include in Phase 9.** Strong version of Scenario 2.

---

### §3.3 C3 — F-3 rejection during F-1a recovery

**What it tests.** A and B federate for Space S. B drops. While B is down, A is removed from S's `federation_nodes` (via `state.federation_remove` or membership.kick). B comes back, A initiates F-1a handshake. Does A push events for S? Does B reject them with `federation_relationship_missing`? Does the handshake itself proceed (per-peer relationship distinct from per-Space relationship)?

**What bug it would catch.** Catalogue bug M4 (F-3 reads stale `federation_nodes` snapshot — race window where revoked peer can still push for ~one event). Plus the "two-source drift" concern from the audit (FederationRegistry vs SpaceState.federation_nodes diverging).

**Cost estimate.** **Medium.** Setup is non-trivial — need a Node that can apply a `state.federation_remove` event while a peer is offline, then verify on reconnect.

**Recommendation:** **Include in Phase 9.** This compound directly exercises the design doc §6.4 "single source of truth" claim that was the audit's primary finding about federation_nodes — Phase 9 is the right place to deployment-prove it.

---

### §3.4 C4 — Phase 5 reconnect scheduler under churn

**What it tests.** Drop peer, recover, drop, recover, drop, recover — 5 cycles in 10 minutes. Does the backoff ladder reset correctly on each handshake-ACTIVE? Does `peer_records` JSON stay consistent? Does any cycle leak a `tokio::spawn`? Does `peer_records` get out of sync with `relationships`?

**What bug it would catch.** Catalogue bug M6 (`tokio::spawn` per peer per tick leaks). Plus invariant checks on the `mark_active` / `mark_lost` API.

**Cost estimate.** **Hard.** Backoff ladder is wall-clock — 15 min initial, 30 min, 60 min, 120 min. 5 cycles even with full recovery = 5+ hours wall-clock unless we inject a clock. Phase 5 did NOT ship a clock-injection seam (`scheduler_tick` accepts a runtime + senders + paths, not a clock).

**Recommendation:** **Defer to follow-on milestone.** Block on clock-injection structural work. Recommend filing a `federation-stress` task that includes clock injection + C1 + C4 + C6.

---

### §3.5 C5 — Validation asymmetry under load

**What it tests.** Phase 9 baseline scenario 4 tests validation asymmetry with one forged event. Compound: send 100 mixed valid+forged events at once via federation push. Does the validation pipeline maintain isolation between events? Does any forged event's rejection state leak into a valid event's acceptance path?

**What bug it would catch.** Catalogue bug M10 (validation asymmetry leaks rejection state across events under load — a forged event in a batch causes a valid event to be rejected).

**Cost estimate.** **Easy-Medium.** NodeRuntime-level test with 100 events fed via direct `dispatch_event` calls. Simple to write. Asserts per-event ingest result independently.

**Recommendation:** **Include in Phase 9** as the strong version of Scenario 4. Replaces the smaller Scenario 4 stress dimension axis directly.

---

### §3.6 C6 — F-10 identity-arrival hook under parallel arrivals

**What it tests.** Two federation pushes arrive simultaneously, both with unknown signers; both signers' identity records arrive in close succession. Does `drain_pending_by_identity` handle parallel arrivals correctly?

**What bug it would catch.** Catalogue bug M9 (HeldPending double-drain on parallel identity arrivals).

**Cost estimate.** **Medium.** Requires careful sequencing of two identity-replicate messages on B. Tractable but needs harness control over timing.

**Recommendation:** **Defer to follow-on milestone.** The single-arrival path covers Phase 9's primary F-10 surface. The parallel-arrival bug is real but secondary; pair with C1/C4 in a `federation-stress` follow-on.

---

### §3.7 C7 — Tip-exchange size limit at boundary

**What it tests.** F-1a tip-exchange uses pagination per F-7. What happens when the delta is exactly `batch_size` (1000)? `batch_size + 1`? `batch_size * 2`? Specifically: does `continue_from` correctly chain across pagination boundaries when the boundary coincides with the delta size?

**What bug it would catch.** Catalogue bug M7 (continue_from pagination loses events at boundary). Off-by-one bugs hide here; the design doc explicitly cautions about pagination edge cases in F-7.

**Cost estimate.** **Easy-Medium.** Build a Space with exactly N events, federate, observe delta size. Three test cases (N=999, N=1000, N=1001, N=2000) are enough.

**Recommendation:** **Include in Phase 9.** Pagination boundary bugs are notorious; protocol-correctness depends on getting this right.

---

### §3.8 C8 — Bidirectional simultaneous push

**What it tests.** A pushes event E_A to B; at the same wall-clock moment, B pushes event E_B to A. Both are valid for the same Space. Do both arrive, both ingest, both reach local fan-out on the other side? Does the long-lived bidirectional session (F-2 + F-2a) handle simultaneous push from both sides without deadlock?

**What bug it would catch.** Catalogue bug M8 (bidirectional simultaneous push deadlocks F-2a session).

**Cost estimate.** **Medium-Hard.** Engineering simultaneous wall-clock push is non-trivial. Tokio `select!` schedules with a deterministic bias (`biased;` in [`xgen-node/src/tests/federation_push_integration.rs:292`](xgen-node/src/tests/federation_push_integration.rs:292)) — actual deadlock would require both sides to block on outbound, which the `try_send` non-blocking pattern prevents. Bug is improbable but not impossible.

**Recommendation:** **Defer to follow-on milestone.** Improbable bug + non-trivial harness cost.

---

### §3.9 C9 (new from trace) — F-3 and the drain-time approximation hazard

**What it tests.** A federation event from peer X for Space S arrives at B; B HeldPends it (missing predecessor). While the event is in PendingBuffer, X is removed from S's `federation_nodes` (operator-driven defederation). Predecessor arrives. The drain-time re-dispatch passes `peer_node_id: None` per the explicit approximation at [`runtime.rs:535`](xgen-core/src/node/runtime.rs:535) ("F-3 not re-checked on drain"). **Does the event ingest or reject?**

**What bug it would catch.** This is a known design-doc-disclosed hazard at [`runtime.rs:529-535`](xgen-core/src/node/runtime.rs:529): "a buffered federation event whose peer relationship was torn down within the 30 s HeldPending window slips through." Phase 9 should *verify* the hazard's bound — that it's at most 30 s (the F-4a window) — and document the exact behaviour.

**Cost estimate.** **Easy.** NodeRuntime-level test.

**Recommendation:** **Include in Phase 9.** Not because the hazard is HIGH-severity (the design doc explicitly accepts it), but because Phase 9 should *prove* the hazard's bound by test, not just by claim. If the bound exceeds 30 s, that's an unrecorded bug.

---

### §3.10 C10 (new from trace) — Identity-replicate hook serialisation under lock contention

**What it tests.** `handle_identity_replicate_msg` at [`app.rs:1592`](xgen-node/src/app.rs:1592) calls `rt.drain_pending_by_identity` inside the same runtime-lock critical section as the identity registration. Under high concurrent federation push load (many incoming events for the same Identity), can a hook fire while another hook is mid-flight? Are buffered events for the same Identity drained twice?

**What bug it would catch.** Catalogue bug M9 (parallel arrivals double-drain) AND a new bug M14 (lock-contention-induced ordering bug — see §6).

**Cost estimate.** **Medium.** Three concurrent federation peers pushing events for the same unknown Bob, then three concurrent identity-replicate messages for Bob. Race-window-sensitive but achievable.

**Recommendation:** **Include in Phase 9** as a stronger replacement for C6. Single Identity, multiple replicate paths converge — simpler to reason about than C6's two-identity case.

---

### §3.11 Compound aggregate

| Compound | Cost | Phase 9? | Replaces / extends |
|---|---|---|---|
| C1 | Medium-Hard | Defer | — |
| C2 | Easy | **Include** | Strong version of Scenario 2 |
| C3 | Medium | **Include** | Extends Scenario 6 |
| C4 | Hard | Defer | Blocked on clock injection |
| C5 | Easy-Medium | **Include** | Strong version of Scenario 4 stress axis |
| C6 | Medium | Defer (superseded by C10) | — |
| C7 | Easy-Medium | **Include** | Strong version of Scenario 3 |
| C8 | Medium-Hard | Defer | — |
| C9 | Easy | **Include** | Drain-time hazard verification |
| C10 | Medium | **Include** | Replaces C6 |

**Phase 9 final compounds:** C2, C3, C5, C7, C9, C10 (six compounds).

**Total Phase 9 scenario count:** 6 baseline + 6 compounds = **12 scenarios**. Of these:
- 7 are deployment-level (binary spawn): 1, 2, 3, 5, 6, C2, C3 (and a final smoke test of C5).
- 5 are NodeRuntime-level: 4, C5 (NodeRuntime body), C7 (NodeRuntime body), C9, C10.

The mixed-shape approach is honest and recommended.

---

## §4 Cross-scenario structural gaps

### §4.1 Gap G1 — `xgen-node_state.json::peers` hard-coded to `vec![]`

**What gap.** [`xgen-node/src/app.rs:1775`](xgen-node/src/app.rs:1775) emits `peers: vec![]` unconditionally. The F-1c per-peer operational record IS persisted (separate JSON at `xgen-node_federation.json`), but the operator-facing aggregated state file omits it.

**Which scenarios are affected.** Scenarios 1, 3, 6; compounds C2, C3, C9.

**Options:**
- (a) Add as Phase 9 precondition. Cost: small (read `xgen-node_federation.json::peer_records`, render into `FederatedPeer` shape).
- (b) Work around by parsing `xgen-node_federation.json` directly in the test harness.
- (c) Defer.

**Recommendation:** **(a)**. Small enough to land as precondition; pays off for all operator-facing observability, not just Phase 9.

---

### §4.2 Gap G2 — No stable structured trace events for F-1 push and F-3 reject paths

**What gap.** Current logs are `tracing::warn!`/`tracing::error!` with free-form message text. Phase 9 tests would parse log message text; a future refactor changing the message text silently breaks Phase 9 — same drift surface D-068 named for logging-level config.

**Which scenarios are affected.** Scenarios 1, 2, 6; compounds C2, C5.

**Options:**
- (a) Add stable structured trace events (e.g. `event = "federation_push", peer = X, outcome = "sent"|"dropped_full"|"dropped_unregistered"`) at all federation rejection / push / accept sites as Phase 9 precondition.
- (b) Match on log message text and accept future-refactor fragility.
- (c) Defer.

**Recommendation:** **(a)**. Phase 9 needs these regardless; downstream M6 + M7 + Client-Side Consequences Audit will also benefit. The drift-prevention argument is the same D-068 honoured.

---

### §4.3 Gap G3 — No in-process way to assert "B's local fan-out reached client C on B"

**What gap.** The stress-complete CommLog records WS-level SENT/RECV. There's no instrument that lets a test assert "Node B's `apply_fanout` reached the connection serving Bob with event E". Today the only proxy is "Bob's WS connection received E off the wire", which couples the assertion to client-side state.

**Which scenarios are affected.** Scenario 1 (honesty check #2), Scenario 2 (Carol's no-receive needs strong reason), compound C2.

**Options:**
- (a) Add a structured trace event at `apply_fanout`'s success path as Phase 9 precondition.
- (b) Use the existing client-side WS receipt and document the coupling.
- (c) Defer.

**Recommendation:** **(a)**, paired with G2.

---

### §4.4 Gap G4 — Audit log for F-3 rejection

**What gap.** F-3 rejection currently emits a `tracing::error!` log line but no protocol audit log per §3.11.8 (the audit-log facility specced for M6 (new) Phase 0). Operator-facing post-mortem of "why did B reject events from X?" needs an audit-log surface, not just transient logs.

**Which scenarios are affected.** Scenario 6 (secondary observation); also future Client-Side Consequences Audit work.

**Options:**
- (a) Add as Phase 9 precondition.
- (b) Work around by parsing transient logs.
- (c) **Defer to M6 (new).**

**Recommendation:** **(c)**. Audit-log emission is M6 (new) scope per the canonical design doc at [`docs/xgen_node_admin_ops_design.md`](docs/xgen_node_admin_ops_design.md:1) — not federation scope. Working around with transient log parsing is fine for Phase 9.

---

### §4.5 Gap G5 — No operator-controlled "drop just the TCP connection" affordance

**What gap.** Scenarios 3, C1, C3 model "drop the peer" as "kill the binary." Killing the binary tests F-1b + F-1a recovery in the "Node process restarted" case, but doesn't test "TCP connection died but Node process is healthy" (network blip, NAT timeout, etc.).

**Which scenarios are affected.** Scenario 3 strong-version; compounds C1, C3.

**Options:**
- (a) Add a `--batch` verb `__DROP_PEER__ <node_id>` that closes just the federation session.
- (b) Work around — for Phase 9, "kill the binary" is the only drop modality.
- (c) Defer to M6 (new) (which is the natural home for admin write-path verbs).

**Recommendation:** **(c)** with a (b)-shape workaround for Phase 9. Drop-this-peer is a natural M6 verb; Phase 9 documents the "binary-kill" approximation as a known coverage limitation.

---

## §5 Flake-handling proposal

**Recommendation: (c) fix both flakes as Phase 9 precondition.**

The v2.0 default lean stands. Code-grounded confirmation:

**Flake #1 — precedence env-var race.** Code at [`xgen-common/src/precedence.rs:139-146`](xgen-common/src/precedence.rs:139). The four `resolve_log_level_*` tests at lines 148-178 all use `with_xgen_log` which does `std::env::remove_var("XGEN_LOG")` / `set_var` bracketing. Process-global env vars + parallel tests = race. **Production code reads `XGEN_LOG` exactly once per Node lifetime** (verified: grep for `XGEN_LOG` shows only one production read site, in the binary's init_logging path — the rest are test-only). So the flake is provably test-only.

**Fix:** Add `#[serial_test::serial]` attribute to the four `resolve_log_level_*` tests. Five-line change. Verifies cleanly with one workspace test run.

**Flake #2 — `reconnect_with_existing_tip_small_delta_delivered`.** Code at [`xgen-node/src/tests/federation_delta_integration.rs:326`](xgen-node/src/tests/federation_delta_integration.rs:326). Binds `127.0.0.1:0` (OS-chosen port), spawns server task, client connects, exchanges WS frames. Pattern is correct in isolation; under parallelism it stresses `tokio::spawn` scheduling.

**Code-grounded production overlap check.** The test uses `Server::bind` + `run_receiving` + `stream_federation_delta` — the same code paths Phase 9 will exercise in every binary-level scenario. If the flake represents a real production parallelism bug, Phase 9 would re-discover it. Phase 9 has more integration tests than any prior milestone (12 scenarios × N tokio tasks each). Shipping Phase 9 on this flake is the worst time for a foundation bug to surface.

**Fix:** Either (i) `#[serial_test::serial]` on the federation_delta_integration module — pragmatic, accepts the parallelism limit; or (ii) investigate the actual race shape (likely on `tokio::spawn` interleaving with `Server::bind`'s polling on a busy runtime). Recommend (i) for ship speed; (ii) for production confidence — if the investigation reveals a production race, that's a Phase 9 precondition bug fix in its own right.

**Walk-back? No.** The v2.0 task file permits walking back to (a) or (b) only if Clair's trace proves "zero overlap with the federation surface." The federation_delta_integration test directly tests the same surface Phase 9 will. The overlap is total, not zero.

**Cost:** Estimated **half-day** for the four-test serial fix + the federation_delta_integration parallelism investigation. Acceptable as precondition.

---

## §6 Failure-mode catalogue

| # | Bug | F-item(s) violated | Detection | Severity |
|---|---|---|---|---|
| M1 | Federation push delivers event but doesn't update local DAG (visible to remote clients, invisible to local clients on the receiver) | F-1, F-2 | Scenario 1 (honesty check #3) | HIGH |
| M2 | F-1b drops the event but doesn't update `peer_records.lost_connection`; reconnect scheduler doesn't schedule recovery | F-1b, F-1c | Scenario 3 (assertion 1) | HIGH |
| M3 | F-10 HeldPending entry survives identity arrival but `drain_pending_by_identity` doesn't fire; event stays HeldPending until timeout | F-10 | Scenario 5 (assertion 3) | HIGH |
| M4 | F-3 check runs against stale `federation_nodes` snapshot — race window where revoked peer can still push for ~one event | F-3 | C3 | MEDIUM |
| M5 | `EventOrigin::ReceivedViaFederation` leaks into local fan-out path — echo loop between Nodes | F-5 | C2 (source-side trace assertion) | HIGH (echo loops multiply) |
| M6 | Phase 5 `tokio::spawn` per peer per tick leaks tasks if reconnect attempts overlap | F-1c | C4 (deferred) | MEDIUM (memory growth not immediate failure) — **not caught by Phase 9** |
| M7 | `continue_from` pagination loses events at boundary | F-7, F-1a | C7 | HIGH |
| M8 | Bidirectional simultaneous push deadlocks F-2a session | F-2, F-2a | C8 (deferred) | HIGH — **not caught by Phase 9** |
| M9 | HeldPending double-drain on parallel identity arrivals | F-10 | C10 (replaces C6) | MEDIUM |
| M10 | Validation asymmetry leaks rejection state across events under load | F-4 | C5 | HIGH |
| M11 | F-3 drain-time approximation: peer X defederated mid-buffer slips through within the 30 s F-4a window | F-3 (design-disclosed) | C9 | LOW (window-bounded, design-disclosed) — verify bound |
| M12 (new) | Lock B1 skip-rule misapplied — a non-federation_add event with type-confusion reaches the F-3 skip path | F-3 (Lock B1) | Scenario 6 (Lock B1 honesty test) | HIGH |
| M13 (new) | F-1c registry consistency: `relationships` upserts under one path but `peer_records.mark_active` runs under another; can produce a peer in `relationships` but missing from `peer_records` | F-1c | Not caught by Phase 9 — **flagged for Client-Side Consequences Audit** | MEDIUM |
| M14 (new) | Lock contention in `handle_identity_replicate_msg`: drain_pending_by_identity called inside the runtime lock; under high concurrency, two replicate calls for the same identity can interleave their drain | F-10 | C10 | MEDIUM |
| M15 (new) | Wire-order non-determinism + causal-DAG-construction lie. **Determinism layer:** `topological_sort_events` preserves input-vector order for ready siblings (DAG roots with empty `prev_events`); its caller `compute_federation_delta_for_space` feeds it via `HashMap.values()` iteration with `RandomState`-randomised order. **Causality layer:** `build_room_create_event` constructs `state.room_create` with `prev_events: vec![]` despite its doc-comment claiming `space_id` is the event_id of the parent `state.space_create`; the event-DAG layer treats `state.room_create` as a root regardless of the protocol-level parent-child relationship the doc-comment claims. Two senders with identical Space state produce different federation-delta wire orderings ~50% of runs even with determinism-layer fix alone; cascading bootstrap rejections when `state.room_create` precedes `state.space_create` in canonical order | F-1, F-3 (cascading) | Phase 9 Scenario 1 honesty check (initial determinism finding J-096); design-phase re-walk + Clair Commit 3 verification surfaced causality framing gap (J-099); LOCKED by **D-076 v1.1 amended in place** (two complementary properties — byte-identical determinism + causal-DAG-respecting order; neither sufficient alone); closed by topological-sort milestone J-101 across Commit 2 (determinism layer, sort fix at `xgen-node/src/fanout.rs:193` + `:321`) + Commit 2a (causality layer, Path B fix at `xgen-core/src/space/state.rs:797`) | HIGH |
| M16 (new) | Drain-without-persist gap. Three drain helpers in `xgen-core::node::runtime` (`drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship`) re-dispatch released events INSIDE `dispatch_event` via internal recursive calls and silently drop the Accepted outcome via `let _ = self.dispatch_event(...)`; `xgen-node::app::process_inbound` persists only the explicitly-passed initial event; on Node restart, `replay_spaces_from_dir` at `xgen-node/src/app.rs:2628` sees only the persisted events and in-flight federation-relationship state established mid-drain is un-replayable. Compounding silent-error surface at `xgen-core/src/node/runtime.rs:181` (`graph.add_event` returning `UnknownPrevEvent` swallowed via `let _ =`) hides any DAG-corruption signal even when the persist site is fixed — the same site that B3 federation-bootstrap path (J-088, 2026-05-20) implicitly relies on as a feature (forward-drift vs backward-coherence — D-077 first worked instance) | F-1, F-3 (cascading), F-1c (relationship-state durability) | Phase 9 Commit 3b-1 Scenario 3 (drop-and-recover with relationship state) at J-104; Commit 3b-1 PAUSED; sub-amendment milestone arc opened per D-071 (audit → design → impl → close shape); closed by persistence-amendment sub-amendment milestone close [J-108] across Commit 2 (Q1 (a).iii.α — binary-void signature + `tracing::error!` at the silent site + Q1 (a).ii defensive sort-on-replay at `replay_spaces_from_dir`; reverted from (a).iii.β at re-walk Track 1 J-107 per Y-lock on B3 dependency) + Commit 2a (Q2 (a) return-vector `DispatchOutcome::Accepted { additional_persisted: Vec<Event> }` + Q3 all-three drain helpers return `Vec<Event>` + `dispatch_event` aggregates at three call sites + `process_inbound` persist-loop block) + Commit 3 (sentinel-tree four files activating Scenario 3 transition FAIL → PASS). **D-077 promoted** (bidirectional sustainability discipline at silent-discard / conditional-mutation / fallible-discard sites — meta-layer above D-067 + D-070 + D-075 + D-076 v1.1 protocol-layer no-drift-surface family). **Layered-B3 second project-wide instance** (sibling-shape to topo-sort Commit 2a layered-B3 at J-101: primary fix at drain-hook layer + secondary silent-error surface at runtime.rs:181 closed atomically within the milestone). Activating regression lock at integration level: Scenario 3 transition FAIL → PASS (sibling-shape to Scenario 1's role for D-075 + D-076 v1.1 at J-101). Candidate D-NNN "ingest path invariant encoding under bidirectional sustainability discipline" stays flagged-not-promoted at design doc §8 with scope expanded at J-107 | HIGH |

**Caught by Phase 9 set:** 13 of 16 (M15 caught by Scenario 1 second `#[ignore]` lift at topological-sort milestone J-101 — the same scenario that surfaced the determinism finding at J-096 became the activating regression lock for both layers; M16 caught by Scenario 3 transition FAIL → PASS at persistence-amendment milestone J-108 — same Phase 9 Commit 3b-1 work that surfaced the drain-without-persist gap at J-104 became the activating regression lock).
**Not caught (feeds follow-on milestones):** M6 (C4 deferred), M8 (C8 deferred), M13 (no current detection path) → all three feed the Client-Side Consequences Audit per the project's J-081-shape precedent.

---

## §7 Observability audit

### Sub-item A — Findings against the audit checklist

1. **`xgen-node_state.json` exports.**
   - `pending_identity_replication` — **present** (Phase 6, [`app.rs:1777`](xgen-node/src/app.rs:1777)). Verified.
   - Per-peer connection state — **GAP G1**, `peers: vec![]` hard-coded.
   - Federation push queue depth — **NOT present**. Per F-1b, there is no outbound queue (try_send drops; no buffer). The "queue depth" question is therefore "channel capacity utilisation on `FederationPeerSenders::Sender`" — observable as a tracing gauge but not as state.json field. Not strictly Phase 9 — flag for `federation-stress` follow-on.

2. **Log shape stability.** **GAP G2**. Today's F-1, F-1b, F-3, F-4 reject paths emit `tracing::warn!`/`error!` with free-form message text. Stable structured tracing events are the safest pattern. Recommend precondition work as part of Phase 9.

3. **Distinct outcomes are distinctly observable.**
   - F-3 reject reason includes the unique string `federation_relationship_missing` (line 379 in runtime.rs).
   - F-10 HeldPending has a distinct path (`tracing::debug!` "event buffered" line in app.rs:1432).
   - F-1b drop has its own R14 line.
   - **Co-located rejection log site at app.rs:1441** uses `reason = %reason` which carries the unique string forward — distinct enough.
   - **Verified distinct.**

4. **Per-peer event flow visibility.** **GAP G2 subset** — observing "which peer pushed which event" requires a stable trace event at the source side of `apply_federation_push`. Currently only the drop branches emit, not the success branch. **Add a stable trace event for the success path** in Phase 9 precondition work.

5. **Timing observability.** State.json updates every 5 s — too coarse for F-10's "identity-arrives-just-before-timeout" axis (sub-second). **Use tracing-event timestamps** (microsecond-resolution by default in `tracing-subscriber`) for the F-10 timing axes. Phase 9 tests parse the trace event timestamps directly, not the state.json file.

### Sub-item B — Aggregate gap close-out plan

| Gap | Close-out option | Approx cost |
|---|---|---|
| G1 (peers field) | Phase 9 precondition (option a) | ~50 lines in `build_node_state` |
| G2 (stable trace events) | Phase 9 precondition (option a) | ~10 trace event additions across federation_session.rs, app.rs, runtime.rs |
| G3 (fan-out trace event) | Phase 9 precondition (option a) | ~3 trace event additions in fanout.rs |
| G4 (audit log for F-3) | Defer to M6 (option c) | M6 scope |
| G5 (drop-peer affordance) | Workaround for Phase 9 + defer to M6 (option c) | M6 scope |

Total precondition observability work: ~1-2 days of Clair effort. Pays back across all federation-touching observability work going forward.

---

## §8 Open questions for Joe — LOCKED 2026-05-19

### Q1 — Scope of compound count

The recommendation is 12 scenarios (6 baseline + 6 compounds). The original runbook §3.9 wording was "Two-Node smoke test; Three-Node smoke test if affordable" — the 6 baseline came from §3.9's enumeration but the runbook didn't anticipate the strong-version per-axis stress dimensions or the compound additions.

**Alternative scopes if 12 is too large:**
- **Minimal 8:** 6 baseline + C2 + C5. Drops C3, C7, C9, C10. Loses M4, M7, M9, M14 coverage.
- **Recommended 12:** 6 baseline + C2, C3, C5, C7, C9, C10. Catches 12 of 15 catalogue bugs (M15 added at topological-sort milestone J-101 — caught by Scenario 1 second `#[ignore]` lift).
- **Maximal 16:** add C1, C4, C8 if clock injection lands as precondition. Adds M6, M8 coverage but with a separate harness precondition.

**Question for Joe:** lock 8, 12, or 16. Default recommendation is 12 unless time pressure on M6 (new) makes the smaller count attractive.

**LOCKED — 12 scenarios.** Rationale: Minimal-8 drops C3 (catches M4, the audit's primary structural drift concern) and C7 (catches M7, the kind of bug that ships silently to production and surfaces months later when a Space hits 1000 events). Both are too good a cost-benefit ratio to drop. Maximal-16 needs clock injection, which is its own harness subsystem and creates scope-creep risk in a milestone-closing phase. Clock injection + C1/C4/C6/C8 land in the `federation-stress` follow-on milestone (task stub to be filed at Phase 9 close per Step 3 below). Priority lock: working functions over done-mark; 12 is the bug-finding-leverage sweet spot.

### Q2 — Should the audit-log-emission for F-3 rejection (Gap G4) come forward as Phase 9 precondition?

The recommendation defers to M6, but Joe may want F-3 audit logging in Phase 9 specifically because federation rejection without audit trail is operationally weak. Cost: small (~30 lines at the rejection emit site). Trade-off: schema for the protocol audit log is M6 (new) Phase 0 territory; emitting prematurely creates a documentation drift surface.

**Question for Joe:** include in Phase 9 precondition, or hold for M6?

**LOCKED — defer to M6.** Rationale: protocol audit log schema is M6 (new) Phase 0 territory. Emitting F-3 audit lines in Phase 9 with a placeholder schema creates a documentation drift surface that some future M6 implementer would have to clean up — exactly the "premature canonicalisation" failure mode D-069 names. Operationally weak today? Yes — but Phase 9's transient log parsing proves the behaviour is correct; M6 makes the audit-trail visibility production-grade. Two different concerns, two different milestones. Working around with transient log parsing is fine for Phase 9.

### Q3 — Flake fix scope

Option (c) recommends fixing both flakes as precondition. Flake #1 is trivial (`#[serial_test::serial]`); Flake #2 may turn into a multi-day investigation if option (ii) is chosen (investigate the real race). 

**Question for Joe:** start with option (i) `#[serial_test::serial]` and only walk back to (ii) if Phase 9's runs surface new symptoms; or commit upfront to the investigation?

**LOCKED — option (i) `#[serial_test::serial]` first; escalation to option (ii) triggered only by Phase 9 signal.** Rationale: investigating a tokio/WS parallelism race upfront could cost 3-5 days with uncertain outcome. Phase 9 implementation is ~1-2 weeks of work; if Flake #2 doesn't surface during Phase 9 stress, the investigation was wasted. If it DOES surface (federation_delta path is heavily exercised by C7 and the deployment-level scenarios), Phase 9's failure logs will be the best diagnostic input we'll ever have. **Escalation criterion to be documented in the Phase 9 implementation task file** so it's not lost: if any new Phase 9 integration test exhibits a `127.0.0.1:0` bind race or WS frame-ordering inconsistency under workspace parallelism that isn't explained by the test's own logic, walk back to option (ii) and investigate the underlying race before continuing.

### Q4 — Phase 9 implementation cadence

The recommended 12 scenarios + observability gap close-outs + flake fixes is ~1-2 weeks of Clair work, not days. The original runbook §3.9 DoD line "Phase-9 commit pushed by Joe" implied a single commit; Phase 9 should probably ship as **multiple commits** — observability preconditions first (with their own JOURNAL entry), then flake fixes, then per-scenario test additions.

**Question for Joe:** approve multi-commit Phase 9 (canonical for milestone close) vs single mega-commit (the runbook's letter)?

**LOCKED — multi-commit Phase 9.** Rationale: mirrors how M5 (J-078, 12 atomic commits) and CLI Audit (J-079, 5 atomic commits) shipped — both similarly multi-surface milestone-shape work. Single-mega-commit pattern works for phases that are genuinely one logical change; Phase 9 isn't. Each commit independently reviewable; each commit has actual `cargo test` output quoted; each commit has its own JOURNAL sub-entry within the J-### journal entry for milestone close. Expected commit shape (to be sequenced in the Phase 9 implementation task file):

1. **Commit 1 — Observability preconditions (G1 + G2 + G3).** Fills `peers` field in state.json; adds stable structured trace events at federation push/reject/accept sites + fan-out success path. ~60-80 lines across `app.rs`, `federation_session.rs`, `runtime.rs`, `fanout.rs`.
2. **Commit 2 — Flake fixes (option (i)).** `#[serial_test::serial]` on the four `resolve_log_level_*` tests + on the `federation_delta_integration` module.
3. **Commits 3-N — Per-scenario test additions.** Grouped by harness type and locked thematically (e.g. one commit for deployment-level baseline scenarios 1-3, another for NodeRuntime-level scenarios 4 + C5 + C7 + C9 + C10, another for the three-Node compounds C2 + C3, another for scenarios 5 + 6 + final integration smoke).
4. **Final commit — milestone close.** CLAUDE.md + ROADMAP.md flip Federation Event Propagation milestone PLAY → DONE; JOURNAL J-### consolidates the sub-entries; `cargo test` output for full milestone-close quoted.

---

## §9 Survey methodology notes

**Sources actually read (per CLAUDE.md Rule 2):**

- [`tasks/FEDERATION_PROPAGATION_COMPLETION.md`](tasks/FEDERATION_PROPAGATION_COMPLETION.md:823-879) §3.9 — Phase 9 scope and DoD.
- [`xgen-node/src/tests/federation_relationship_integration.rs`](xgen-node/src/tests/federation_relationship_integration.rs) — full file.
- [`xgen-node/src/tests/heldpending_identity_integration.rs`](xgen-node/src/tests/heldpending_identity_integration.rs) — full file.
- [`xgen-node/src/tests/reconnect_integration.rs`](xgen-node/src/tests/reconnect_integration.rs) — full file.
- [`xgen-node/src/tests/federation_push_integration.rs`](xgen-node/src/tests/federation_push_integration.rs) — full file.
- [`xgen-core/src/federation/registry.rs`](xgen-core/src/federation/registry.rs) — full file.
- [`xgen-core/src/node/runtime.rs`](xgen-core/src/node/runtime.rs:40-602) — `DispatchOutcome`, `EventOrigin`, `dispatch_event`, drain helpers.
- [`xgen-node/src/app.rs`](xgen-node/src/app.rs:440-491) — timeout sweep; lines 1340-1461 — `process_inbound`; lines 1700-1780 — `build_node_state`.
- [`xgen-node/src/federation_session.rs`](xgen-node/src/federation_session.rs:170-267) — `apply_federation_push` + F-5 guard.
- [`xgen-common/src/state.rs`](xgen-common/src/state.rs) — full file.
- [`xgen-common/src/precedence.rs`](xgen-common/src/precedence.rs:61-180) — `resolve_log_level` + the four `with_xgen_log` tests.
- [`xgen-node/src/tests/federation_delta_integration.rs`](xgen-node/src/tests/federation_delta_integration.rs:325-450) — `reconnect_with_existing_tip_small_delta_delivered`.
- [`xgen-client/src/app.rs`](xgen-client/src/app.rs:3597-3760) — `cmd_stress_complete` opening section (setup pattern, 4-50 members).
- [`docs/xgen_federation_propagation_design.md`](docs/xgen_federation_propagation_design.md:1075-1108) §15 — Implementation Complete record, plus §6.4 lines 300-345 (F-3 single-source-of-truth).
- [`docs/xgen_ch3_specification.md`](docs/xgen_ch3_specification.md:3046-3108) — §3.9.6 + §3.9.8 (4002/4006 codes, predecessor-code-wins rule).
- CLAUDE.md (currently loaded context) — MANDATORY behaviour rules, known-flake state, milestone status.

**What was traced (not just read):**

- F-5 guard at the source side (F-3 + apply_federation_push) — confirmed the guard is the *only* origin-aware site on the push path.
- F-3 data source (`SpaceState.federation_nodes` vs `FederationRegistry.shared_spaces`) — confirmed the design doc §6.4 "single source of truth" claim matches the shipped code at runtime.rs:373.
- F-10 timeout branching (4002 vs 4006) — confirmed app.rs:469-486 implements the predecessor-code-wins rule per the spec §3.9.6.
- `xgen-node_state.json::peers` — confirmed hard-coded to `vec![]` at app.rs:1775. Identified as Gap G1.
- F-10 hook timing — confirmed `drain_pending_by_identity` runs synchronously inside the runtime lock at app.rs:1592, providing a < 100 ms assertion window.

**What was simulated mentally:**

- Each scenario's honesty-test risks — walked through "what other reason could this assertion pass for" per the survey's §3.1 sub-item C requirement.
- Compound interactions — for each compound, named the specific catalogue bug it would catch (or admitted weak motivation if no concrete bug came to mind).
- Failure-mode catalogue — extended from the §3.5 starter set by tracing the code paths each F-item exercises; M12, M13, M14 are the new entries from the trace.

**What was NOT done (per Rule 6):**

- No `cargo test` was run. The 519-test baseline is taken from CLAUDE.md and the design doc §15 record; survey adds no new measurement.
- No clock-injection investigation for C4 — the conclusion (deferred to follow-on) is grounded in the observed absence of a clock-injection seam, not an attempted implementation.
- No `stress-complete` test was run to validate the 14.6 s J-059 reference — taken from CLAUDE.md as the recorded reference, not re-measured.

**Confidence note.** The findings are code-grounded for the surfaces directly inspected. Compound C8 (bidirectional simultaneous push) is the lowest-confidence finding — "deadlocks improbable due to try_send" is a reasoning step about an absent failure mode rather than a code-grounded observation. Phase 9 implementation work should test the claim; if a real deadlock is found, escalate to a HIGH-severity entry in the catalogue.

---

*End of findings document. All four §8 open questions Joe-locked 2026-05-19. Phase 9 implementation task file (`tasks/FEDERATION_PROPAGATION_PHASE_9.md`) and federation-stress follow-on stub (`tasks/FEDERATION_STRESS_FOLLOWON.md`) authored against these locks.*  
