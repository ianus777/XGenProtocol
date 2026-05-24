# Task — Federation Event Propagation Phase 9 Survey
> **Status**: PENDING  
> Version: 2.0  
> Date: May 2026  
> **Last updated**: 2026-05-24 (J-109 supersession pointer — the body of this survey doc references the pre-Phase-7.5 F-3 outcome shape (permanent Rejected) at multiple sites (§2.6 narrative + §5/§6 honesty-test framing references); these references are now superseded by `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` v1.2 §2.6 + new §2.6.1 contract walkthrough, which captures the Phase 7.5 §6 held-not-bypassed posture (outcome `DispatchOutcome::HeldPending` + `disposition = "held_pending"` field on the existing `f3_reject` G2 trace event + recovery via `drain_pending_by_federation_relationship` arrival hook + 4007 federation_relationship_timeout). This survey doc body is preserved unchanged as historical record of the pre-Phase-7.5 framing per the two-document survey-vs-findings framing locked at original survey close (Joe-lock 2026-05-19); the findings doc is the locked source-of-truth for Scenario 6 contract assertions at Clair's Phase 9 Commit 3b-2-equivalent pickup. Sibling-shape to how `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` audit doc was preserved as PENDING/COMPLETED at v1.1 unchanged when the design + impl docs superseded its specific framing details. Per D-077 bidirectional sustainability discipline + Reading B lock per J-109 framing. Previous content: 2026-05-19 (v2.0 — adversarial rewrite: ten items instead of eight; per-scenario stress dimensions; compound scenarios; failure-mode catalogue + observability audit; flake-handling recommendation walked to "fix both as precondition"; priority restated as "working functions, not done-mark on roadmap")  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Audit the surfaces the Phase 9 DoD scenarios will exercise — *before* any Phase 9 integration-test code is written. Surface drift, structural gaps, harness-shape questions, flake-handling decisions, and **the conditions under which federation can actually break**. Produce a survey-findings document that Joe + Chat Claude can review and lock decisions on; Phase 9 implementation work proceeds only after those locks are in.

**Priority for this milestone close-out is unambiguous.** Phase 9 exists to prove federation *works under conditions that matter*, not to put a green checkmark next to six cherry-picked scenarios so M6 (new) can unblock. A milestone that ships green and turns out to have N2N bugs three weeks later is a milestone that failed at its real job. This survey's job is to make sure Phase 9 catches bugs if they exist — and that means hunting for them, not just verifying the happy path.

**Why this exists.** The Federation Event Propagation milestone has shipped seven implementation phases plus one documentation pass. Each phase had its own per-surface tests. Phase 9 is the **deployment-level adversarial proof** — multiple `xgen-node` binaries, real TCP, end-to-end verification, **and explicit attempts to break the system under representative conditions**. Surfaces that look correct in isolation can compose badly; surfaces that look testable in isolation can be hard to drive from a deployment harness; bugs that are absent at single-event scale appear at queue-depth scale. This survey catches all three before Phase 9 writes assertions against assumed shapes.

**This is the Phase 0 pattern, applied with teeth.** Phases 6, 7, and 8 each did a pre-implementation audit step before code. Phase 9 follows the same discipline but with a wider lens: not just "what does this F-item need to be tested" but "what bugs would Phase 9 catch if they exist, and what bugs would it miss." D-071 ("Subsystem audits precede dependent milestones") and the project principle "Subsystem audits precede dependent milestones" both apply here.

**No code changes in this task.** The deliverable is a survey-findings document and a recommendation set. Phase 9 implementation is a separate task file authored against the locked recommendations.

---

## §1 — Mandatory reading

Read in this order before starting the survey. Sources cited below are the survey's authority; the survey's findings must be code-grounded in these.

| Source | What it gives | Why read it |
|---|---|---|
| `CLAUDE.md` MANDATORY behaviour rules (top of file) | Rules 1–7. | Apply throughout. Quote real code with file:line references (Rule 2); never speculate when uncertain (Rule 1); ask if ambiguous (Rule 6). |
| `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.9 | The Phase 9 scope as Joe-locked at runbook handoff: six DoD scenarios listed verbatim. | The survey's baseline six are anchored here. Phase 9 will likely **grow** beyond six per this survey's compounds (item 9) and observability requirements (item 10). |
| `docs/xgen_federation_propagation_design.md` (v1.0, ACTIVE) | All ten F-items locked. §15 Implementation Complete records the eight shipped phases. | Each scenario maps to one or more F-items; the survey cites the design doc when asserting which F-item a scenario owns. |
| `docs/xgen_propagation_reliability.md` (J-081 audit, ARCHIVED) | The audit that motivated the milestone. §2, §3, §5 are the HIGH-severity findings the six scenarios must close. | Phase 9's purpose is to prove the audit's HIGH-severity gaps are closed at the deployment level. Re-read with adversarial framing: would the audit's bug hunter accept the proposed Phase 9 assertions as proof? |
| `xgen-node/src/tests/federation_relationship_integration.rs` | Phase 7 integration tests — including `state_federation_add_skips_f3_check`, the **honesty-test precedent**. | Survey item 3 (honesty-test framing) is anchored to this file's style. Read it carefully — the negative assertion ("outcome is NOT `federation_relationship_missing`") is the load-bearing pattern. |
| `xgen-node/src/tests/heldpending_identity_integration.rs` | Phase 6 integration tests — F-10 scenarios at NodeRuntime level. | Reference for what "NodeRuntime-level test" means in this codebase, vs deployment-level which Phase 9 needs. |
| `xgen-node/src/tests/reconnect_integration.rs` | Phase 5 integration tests — bilateral A-initiates / B-initiates pair. | Reference for in-memory `FederationRegistry` driving of reconnect surface. |
| `xgen-node/src/tests/federation_push_integration.rs` | Phase 4 integration tests — federation push at NodeRuntime level. | Reference. |
| Existing `stress-complete` test (J-059) | The deployment-level precedent: 3-Node topology, real TCP, spawned `xgen-node` binaries, 6/6 scenarios, 14.6 s runtime. **Critically: J-059 found two real bugs during its live run.** | The strongest precedent for Phase 9's shape AND a proof-point that adversarial deployment testing finds bugs that NodeRuntime tests miss. Locate via: search workspace for `stress_complete`. |
| `xgen-node/src/app.rs::process_inbound`, `xgen-core/src/node/runtime.rs::dispatch_event` | The two surfaces every federation-receiving scenario exercises. | Survey item 1 (preconditions inventory) cites these for each scenario. |
| `xgen-core/src/federation/registry.rs` | `FederationRegistry` + `PeerOperationalRecord` — Phase 5's surface. Drop-and-recover scenarios exercise this directly. | Required reading. |
| `docs/xgen_ch3_specification.md` §3.9.6 + §3.9.8 | Error code 4006 + the predecessor-code-wins sub-rule. | Required reading for F-10 scenarios. |
| `xgen-common/src/state.rs` (`build_node_state`) | What's exported in `xgen-node_state.json`. | Required reading for item 10 (observability audit) — Phase 9 tests are only as strong as what they can observe. |
| **CLAUDE.md "Known test-flake state"** section | The two pre-existing flakes that Phase 9's increased parallelism will stress harder than anything before. | Required reading for item 8 (flake-handling). The survey's recommendation is to fix both as precondition; the reading is to confirm both flakes' signatures match the diagnosis. |

After reading, the model in your head must be: each F-item has an in-process test today; Phase 9 elevates that to a deployment-level test under conditions designed to find bugs; the survey's job is to design the conditions, not just verify the elevations.

---

## §2 — The baseline six scenarios (locked at runbook handoff)

Verbatim from `tasks/FEDERATION_PROPAGATION_COMPLETION.md` §3.9 — restated here so this task file is self-contained.

| # | Scenario | Primary F-item | Cross-phase surfaces |
|---|---|---|---|
| 1 | Two-Node federation push smoke | F-1 (Phase 4) | F-1a handshake (Phase 3), F-2 long-lived session |
| 2 | Three-Node anti-transitivity | F-5 origin gating (Phase 4) | F-1 push, `EventOrigin` enum |
| 3 | Drop-and-recover | F-1b drop (Phase 4) + F-1a recovery (Phase 3) | F-1c per-peer record + reconnect scheduler (Phase 5) |
| 4 | Validation-asymmetry regression | F-4 pipeline unification (Phase 2) | F-7 pagination (Phase 1) |
| 5 | Unknown-signer first-contact | F-10 HeldPending (Phase 6) | F-10a 30s timeout, 4006 error code |
| 6 | Federation-relationship rejection | F-3 verification gate (Phase 7) | `SpaceState.federation_nodes`, B1 skip-rule |

**Phase 9 final scenario count will likely be 10-12, not 6.** The baseline six remain; items 9 and 10 add compounds and observability-driven additions on top. The runbook §3.9 wording "Two-Node smoke test. Three-Node smoke test if affordable." was conservative pre-survey; this survey's job is to recommend the strong version of Phase 9, and Joe locks the final scope after reading findings.

---

## §3 — Survey items (ten total)

For each finding, cite the file and line. "Code-grounded" means: quote the actual code; explain what it does; cite where the assumption originates. Speculation is not acceptable.

### §3.1 Items 1-6 — Per-scenario audit (five sub-items each)

For **each of the six baseline scenarios** in §2, the survey produces a section answering five sub-items.

#### Sub-item A — Preconditions inventory

What shipped surfaces does this scenario exercise? Enumerate:

1. **Code surfaces.** Every function, struct, enum, or wire-message type the scenario touches. Cite file:line.
2. **Configuration surfaces.** What `[*]` config sections, what flags, what env vars must be set per Node binary to drive the scenario? Are they all reachable from a deployment harness today?
3. **State-file surfaces.** What does the scenario expect to observe in `xgen-node_state.json`, `xgen-node_federation.json`, the SQLite identity DB? Are the relevant fields exported?
4. **Operator controls.** Does any scenario require an operator action that doesn't have an automation surface today? (Example: scenario 3 needs to "drop a peer connection" — does that mean killing the process, killing the TCP socket, or something `--batch`-driveable?) If any operator action is missing automation, flag it as a structural gap (feeds into item 7).

#### Sub-item B — Observation strategy

How does the test confirm the outcome? Three observation channels to consider, per scenario:

1. **State files** (`xgen-node_state.json` polling, like J-085's reconnect tests do).
2. **Logs** (parse log files for specific tracing events — quote actual log shapes from existing tests as precedent).
3. **In-process assertions** (where the test harness can reach into the binary's runtime — usually only if the test is in-tree against a `NodeRuntime` rather than against a spawned binary).

For each scenario, recommend the primary observation channel and any secondary cross-checks. Be honest: if observation is hard for a scenario, say so — that's a finding worth surfacing now and feeds item 10.

#### Sub-item C — Honesty test framing

**Per Phase 7 precedent — `state_federation_add_skips_f3_check`.** That test's load-bearing pattern: it asserts the *negative* ("outcome is NOT `federation_relationship_missing`") because the positive outcome (HeldPending from Phase 6's unknown-signer rule) is orthogonal to F-3's skip rule. An assertion that just checked "outcome is HeldPending" would have passed for the wrong reason. The honest assertion isolates F-3.

For each scenario, produce a one-paragraph **honesty check**: does the proposed assertion confirm the F-item the scenario owns, or could it pass for an orthogonal reason? If the latter, propose a sharper assertion shape that isolates the F-item.

Orthogonal-pass risks for the baseline six (starter prompts — the survey may surface more):

- **Scenario 1:** "B receives the event" — could pass because of handshake history dump, not because of Phase 4 push. Honest check: does the assertion distinguish push from dump? (E.g. event arrives at B *after* handshake-ACTIVE timestamp.)
- **Scenario 2:** "C does not receive E" — could pass because C is offline, because C never received the federation_add for the Space, because of a race. Honest check: confirm B *had the chance to forward* and *chose not to* per F-5 — log every B outbound and assert origin filter, not just absence at C.
- **Scenario 3:** "B receives queued events after recovery" — could pass via push retry rather than F-1a tip-exchange recovery. Honest check: distinguish recovery path from late push by observing the handshake message + tip-exchange messages.
- **Scenario 4:** "Forged-signature event is rejected" — could pass because B is offline, because the federation relationship doesn't exist, because of a network glitch. Honest check: confirm rejection happens at the validation pipeline with the right error code.
- **Scenario 5:** "Event eventually ingests after Identity replication" — could pass via timeout-retry rather than F-10's identity-arrival hook. Honest check: confirm resolve happens *on identity arrival*, not on a separate sweep.
- **Scenario 6:** "Event from peer X is rejected" — could pass because X has no events to send, because X's connection drops, because of any other reason than F-3. Honest check: confirm F-3's `federation_relationship_missing` is the recorded rejection cause.

The discipline: **every assertion should fail for exactly one reason.**

#### Sub-item D — Harness fit

Does J-059's `stress-complete` shape work for this scenario? Three possible answers, each with rationale:

1. **Reuse `stress-complete` shape** (spawn binaries, real TCP, observe via state files + logs). Cite which patterns specifically apply.
2. **Adapt** — the shape is mostly right but needs additions (e.g. drop-and-recover needs a "kill connection" affordance the existing stress test doesn't have).
3. **Diverge** — a different shape entirely fits better.

Mixed-shape Phase 9 is acceptable if it's the honest answer. The survey's job is to surface the per-scenario decision; Joe locks it.

#### Sub-item E — Stress dimensions (NEW in v2.0)

For each scenario, enumerate the axes along which the scenario can be made stronger or weaker. The survey recommends a **strong** baseline per axis — biased toward stronger, not toward affordable. "Affordable" is what Phase 9 budgets for runtime; "strong" is what catches bugs.

The honest rule: **the strong version of each scenario is the version where Clair would expect bugs to be found if they exist.** Weak versions only prove the happy path.

Starter axes for the baseline six (the survey may add more per scenario):

- **Scenarios 1, 2 (push smoke, anti-transitivity):** strength comes from **volume + variety**. 1 event proves the wire works; ~100 events at varied sizes (small text, near-max payload, mixed event types from message.text through state.* through membership.*) proves the pipeline works. Recommend ~100-event mix unless harness cost is prohibitive.
- **Scenario 3 (drop-and-recover):** strength comes from **timing edge cases**, not volume. Drop mid-handshake vs mid-stream vs idle vs during outbound push queue drain. 5-10 events queued is enough — but the **drop moment** needs to be varied. Bugs hide in the moments, not in the volume.
- **Scenario 4 (validation asymmetry regression):** strength comes from **coverage of forged-event variants × reject paths**, not volume. One forged signature isn't enough — survey enumerates forged signature, forged sender, forged event_id, malformed prev_events, etc., each across all 5 reject paths in `process_inbound`. Recommend ~20 variants total.
- **Scenarios 5, 6 (unknown-signer, F-3 rejection):** strength comes from **the moment** the unknown/missing condition appears.
  - Scenario 5 axes: identity arrives 1s before F-10 timeout, identity arrives 1ms before timeout, identity arrives 1ms after timeout (must NOT resolve — the timeout error wins), identity arrives twice (must not double-fire `drain_pending_by_identity`).
  - Scenario 6 axes: relationship exists at handshake then revoked mid-session, never existed, exists asymmetrically (A federates B but B doesn't federate A).

For each axis, the survey recommends a specific test parameter (number of events, timing offsets, variant counts) with rationale. "We didn't pick a number" is not a survey output.

---

### §3.2 Item 7 — Cross-scenario structural gaps

Aggregate the structural gaps surfaced across items 1-6 sub-item A point 4. For each gap, the survey records:

1. **What gap.** The operator control, observability surface, or test harness affordance that is missing today.
2. **Which scenarios are blocked or weakened.** A gap that blocks scenario 3 only is less urgent than a gap that weakens scenarios 1, 3, and 5.
3. **Three options for closing the gap:**
   - (a) Add the missing affordance as Phase 9 precondition work (new commit before Phase 9 tests start).
   - (b) Work around the gap in the test harness — accept the awkwardness as a known limitation.
   - (c) Defer the gap to a follow-on milestone — accept that Phase 9 doesn't fully prove that surface.
4. **Recommendation.** Survey picks one of (a)/(b)/(c) per gap with rationale; final lock is Joe's.

**D-069 discipline applies.** Gaps should be surfaced now, not papered over at implementation time. A gap discovered mid-Phase-9 and silently routed around is exactly the failure mode D-069 names.

**Default lean for v2.0:** when in doubt, prefer (a). Phase 9's job is to find bugs; missing observability surfaces ARE bugs (or pre-bugs) and should be closed rather than worked around. (b) is acceptable only when the gap is non-load-bearing for bug-finding. (c) is acceptable only when the gap is clearly outside the federation surface — which by definition no Phase 9 finding can be.

---

### §3.3 Item 8 — Flake-handling (RECOMMENDATION WALKED IN v2.0)

**Context.** Two pre-existing flakes are in scope when Phase 9's `cargo test --workspace` runs:

1. **Precedence env-var race** (introduced at D-068 commit `3e2f311`). Surfaces in ~10-20% of full workspace runs.
2. **`reconnect_with_existing_tip_small_delta_delivered`** (Phase 3 test, surfaced under Phase 4's parallelism increase). Surfaces in ~10% of full workspace runs; 0% in isolated runs.

**Three options remain on the table.** The v2.0 survey's recommendation is option (c) — fix both as Phase 9 precondition. Rationale below; the survey may walk this back if Clair's trace surfaces evidence the recommendation is wrong, but the burden of proof is on walking back, not on walking forward.

(a) **Parallelism-controlled bucket for new Phase 9 tests.** Cost: hours. Benefit: Phase 9 tests don't contribute to flake noise. **Honest assessment: this is a workaround that papers over the underlying issue.**

(b) **Document flake-retry as milestone-close protocol.** Cost: minimal. Benefit: no harness work. **Honest assessment: this is the "ship green on retry" pattern, which is D-065's "polite behaviour over honest behaviour" failure mode applied to test results. A milestone that needs a retry rule to pass is a milestone whose `cargo test --workspace` says PASS by convention, not by truth.**

(c) **Fix both flakes as Phase 9 precondition.** Cost: days (env-var race is structural; reconnect flake is concurrency-shaped). Benefit: clean close-out, no retry rule, **AND** the two flakes are *informative about the codebase's concurrency assumptions*:
- **Precedence env-var race** — env vars are process-global; if a test races on them, the *production code* might race on them too under the right conditions. This isn't just a test bug; it's a signal.
- **`reconnect_with_existing_tip_small_delta_delivered`** — the "passes in isolation, fails under parallelism" pattern almost always means shared state somewhere the production code didn't expect to share. **Phase 9 will *increase* parallelism** (more spawned binaries, more concurrent TCP connections). Shipping Phase 9 on top of a known parallelism-sensitive flake is shipping on a foundation that Phase 9 will stress harder than anything before it.

**Survey recommendation in v2.0: option (c).** Both flakes get fixed before Phase 9 tests are written. If during fix work, either flake turns out to be a genuine production concurrency bug (not just a test bug), that's a finding worth surfacing immediately — federation under load is exactly where production concurrency bugs land.

**The survey may walk this back** if Clair's trace finds either flake is *provably* test-only and has zero overlap with the federation surface. The walk-back must be code-grounded, not "feels like a test-only issue." Until that proof exists, the recommendation stands at (c).

---

### §3.4 Item 9 — Compound scenarios (NEW in v2.0)

Federation bugs in real systems show up at the **intersection of conditions**, not in single-condition tests. The baseline six scenarios each test one F-item in isolation; compounds test what happens when two or more F-item conditions co-occur. **Compounds are Phase 9 scope, not follow-on scope** — deferring them is the "done-mark, not working" trap.

The survey enumerates **6-8 plausible compounds** and recommends **3-4 land in Phase 9**. Starter set (the survey may add or replace based on what the trace surfaces):

#### Compound C1 — F-10 unknown-signer arriving during F-1b drop

Peer A pushes event with unknown signer Identity to peer B → B buffers as HeldPending → connection drops before Identity replicates → reconnect happens → does the HeldPending entry survive? Does it resolve? Does the F-10 timeout fire correctly relative to the reconnect timing? Tests Phase 5 + Phase 6 interaction.

#### Compound C2 — F-5 anti-transitivity under push queue depth

A pushes 100 events to B and C in rapid succession; B and C have no relationship. Does B's outbound push to anyone *ever* contain an event with `EventOrigin::ReceivedViaFederation`? **Honest test:** log every outbound push from B and assert origin filter at the source, not just spot-check at the destination. Origin filter could be technically correct in code but functionally bypassed under queue pressure if the origin field is reset somewhere.

#### Compound C3 — F-3 rejection during F-1a recovery

A and B federate for Space S. B drops. While B is down, A is removed from S's `federation_nodes` (via membership.kick or similar). B comes back, A initiates handshake. Does A push events for S? Does B reject them with `federation_relationship_missing`? Does the handshake itself proceed (per-peer relationship is distinct from per-Space relationship)? Critical because the audit found multiple drift surfaces around `SpaceState.federation_nodes` vs `FederationRegistry.shared_spaces` — compound exercises both.

#### Compound C4 — Phase 5 reconnect scheduler under churn

Drop a peer, recover, drop, recover, drop, recover — 5 cycles in 10 minutes. Does the backoff ladder reset correctly on each handshake-ACTIVE? Does the `peer_records` JSON stay consistent? Does any cycle leak a `tokio::spawn`? Does `peer_records` get out of sync with `relationships`?

#### Compound C5 — Validation asymmetry under load

Phase 9 baseline scenario 4 tests validation asymmetry with one forged event at a time. Compound: send 100 mixed valid+forged events at once via federation push. Does the validation pipeline maintain isolation between events? Does any forged event's rejection state leak into a valid event's acceptance path? This is where pipeline-shaped bugs hide.

#### Compound C6 — F-10 identity-arrival hook under parallel arrivals

Two federation pushes arrive simultaneously, both with unknown signers; both signers' identity records arrive in close succession. Does `drain_pending_by_identity` handle parallel arrivals correctly? Could two arrivals double-drain the same HeldPending entry? Could one arrival's drain skip an entry the other was about to process?

#### Compound C7 — Tip-exchange size limit at boundary

F-1a tip-exchange uses pagination per F-7. What happens when the delta to be exchanged is exactly `batch_size` (1000)? `batch_size + 1`? `batch_size * 2`? Off-by-one bugs hide here. Specifically: does `continue_from` correctly chain across pagination boundaries when the boundary coincides with the delta size?

#### Compound C8 — Bidirectional simultaneous push

A pushes event E_A to B; at the same wall-clock moment, B pushes event E_B to A. Both are valid for the same Space. Do both arrive, both ingest, both reach the other side's local fan-out? Does the long-lived bidirectional session (F-2 + F-2a) handle simultaneous push from both sides without deadlock?

**Per compound, the survey produces:**

1. **What it tests** (one paragraph).
2. **What bug it would catch** (cite plausible bug shapes — the survey should be able to imagine the bug in concrete terms; if it can't imagine a bug the compound would catch, the compound has weak motivation).
3. **Cost estimate** (easy / medium / hard, with rationale grounded in harness work required).
4. **Recommendation** (include in Phase 9 / defer / drop).

**Default recommendation: include all compounds rated "easy" or "medium" cost. Defer only "hard" compounds to a follow-on milestone, and only if the survey is confident the deferred compound can land in M7 or M8 scope.** Phase 9 final scenario count = 6 baseline + 3-4 compounds = 9-10 total.

---

### §3.5 Item 10 — Failure-mode catalogue + observability audit (NEW in v2.0)

Phase 9 tests are only as strong as **what bugs they're designed to catch** and **what they can observe**. Items 1-6 sub-item C ask "what would make this assertion pass dishonestly"; item 10 asks the inverse: "what specific federation bugs are we hunting for, and can we see them if they exist?"

#### Sub-item A — Failure-mode catalogue

Enumerate the bugs Phase 9 is designed to detect. Each entry has the form:

```
Bug: [one-sentence description of the bug]
F-item(s) violated: [which F-item(s) this bug would violate]
Detection: [which Phase 9 scenario/compound catches this; "not caught" is acceptable but flagged]
Severity: [HIGH/MEDIUM/LOW based on protocol impact]
```

Starter catalogue (the survey will likely add more during the trace):

1. **Federation push delivers event but doesn't update local DAG.** Event visible to remote clients, invisible to local clients on the receiving Node. F-1, F-2 violated. Detection: scenario 1 — but only if the assertion *both* checks event arrival via federation push *and* checks local-client visibility on B. Severity: HIGH.

2. **F-1b drops the event but doesn't update `peer_records.lost_connection`.** Reconnect scheduler doesn't schedule recovery. F-1b, F-1c violated. Detection: scenario 3 — but only if the assertion checks `xgen-node_federation.json` state explicitly after drop. Severity: HIGH.

3. **F-10 HeldPending entry survives identity arrival but `drain_pending_by_identity` doesn't fire.** Event stays HeldPending until timeout regardless of identity availability. F-10 violated. Detection: scenario 5 — strong version with identity-arrives-just-before-timeout axis. Severity: HIGH.

4. **F-3 check runs against stale `federation_nodes` snapshot.** Race window where revoked peer can still push for ~one event. F-3 violated. Detection: compound C3 if included. Severity: MEDIUM (the window is short but real).

5. **`EventOrigin::ReceivedViaFederation` leaks into local fan-out path.** Echo loop between Nodes: A pushes to B, B fans out locally (which it should), B's local fan-out re-enters federation push to A (which it should NOT — the origin tag exists to prevent exactly this). F-5 violated. Detection: scenario 1 + scenario 2 — but only if the assertion checks Node A does NOT receive its own event back from B. Severity: HIGH (echo loops can multiply rapidly).

6. **Phase 5 `tokio::spawn` per peer per tick leaks tasks** if reconnect attempts overlap. Resource exhaustion under sustained churn. F-1c violated implicitly. Detection: compound C4. Severity: MEDIUM (would surface as memory growth, not immediate failure).

7. **Continue_from pagination loses events at boundary.** Pagination of delta exchange skips events at the chunk boundary. F-7, F-1a violated. Detection: compound C7. Severity: HIGH.

8. **Bidirectional simultaneous push deadlocks F-2a session.** WS frames blocked on both ends waiting for the other to drain. F-2, F-2a violated. Detection: compound C8. Severity: HIGH.

9. **HeldPending double-drain on parallel identity arrivals** results in event ingested twice (and DAG rejecting the duplicate, which surfaces as a rejection of a valid event). F-10 violated. Detection: compound C6. Severity: MEDIUM.

10. **Validation asymmetry leaks rejection state across events** under load. A forged event in a batch causes a valid event in the same batch to be rejected. F-4 violated. Detection: compound C5. Severity: HIGH.

**The catalogue is not exhaustive.** The survey should extend it during the trace. The discipline: **for every assertion in Phase 9, ask "what bug does this assertion catch" and write that bug into the catalogue.** Assertions without a corresponding bug entry are weak assertions.

**Catalogue entries marked "not caught"** become input to the post-milestone Client-Side Consequences Audit (which is the J-081-shape canonical doc that runs after Phase 9 ships, per memory #14). The catalogue should be honest about what Phase 9 does and does not catch.

#### Sub-item B — Observability audit

For each detection method in the failure-mode catalogue, confirm Phase 9 can actually observe what it needs to observe. Specific things to audit:

1. **`xgen-node_state.json` exports.** Does it export `pending_identity_replication` (Phase 6 added it — verify)? Does it export per-peer connection state (Phase 5 — verify)? Does it export federation push queue depth (probably not — flag)?

2. **Log shape stability.** Phase 9 tests will parse logs. Are the log shapes for federation events documented, or could a future log refactor silently break Phase 9? (Stable tracing event names are the safest pattern; flag any scenario that depends on log message text rather than tracing event names.)

3. **Distinct outcomes are distinctly observable.** Can the test observe the F-3 rejection reason vs the F-10 HeldPending reason as distinct outcomes? Can it distinguish F-1b drop from session goodbye? Each scenario's honesty depends on observability being distinct enough to support the assertion.

4. **Per-peer event flow visibility.** Can the test observe *which* peer pushed *which* event? This is critical for compound C2 (F-5 anti-transitivity at the source). If observability is missing, surface it as a gap and recommend adding it as Phase 9 precondition work.

5. **Timing observability.** Several scenarios depend on observing the order of events (handshake-ACTIVE before event arrival for scenario 1; identity arrival before timeout for scenario 5). Are the relevant timestamps recorded with enough precision?

**Gaps in observability are gaps in the test's strength.** If a bug can occur silently because no test can see it, the test should add the observability hook as part of Phase 9, not work around the gap. **Observability additions are themselves protocol-level improvements** — adding a tracing event or a state.json field is small work that pays off across all future federation work.

---

## §4 — Survey deliverable

The survey produces **one file**:

**`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** (Status: PENDING at first write; flipped to COMPLETED when Joe locks the recommendations).

Structure:

```
# Federation Event Propagation Phase 9 Survey — Findings

[header per project convention, Status: PENDING]

## §1 Summary
- One-paragraph per-scenario verdict (six baseline rows).
- Compound scenarios: recommended for Phase 9 (count: N) + deferred (count: M).
- Cross-scenario structural-gap count + recommended close-out approach.
- Flake-handling recommendation (one of a/b/c) with rationale.
- Failure-mode catalogue summary: N HIGH bugs catalogued, M caught by Phase 9, K "not caught" feeding Client-Side Consequences Audit.
- Observability audit summary: N gaps surfaced, M recommended for close-out before Phase 9.
- Harness convergence assessment (one shape vs mixed-shape).
- Final Phase 9 scenario count recommendation.
- Final Phase 9 runtime estimate (best-guess, with rationale).

## §2 Per-scenario findings (six sections, one per baseline scenario)

### §2.1 Scenario 1 — Two-Node federation push smoke
  A. Preconditions inventory
  B. Observation strategy
  C. Honesty test framing
  D. Harness fit
  E. Stress dimensions (with recommended parameters)
  Recommendation: [strong-version description]

### §2.2 through §2.6 — same shape for the other five baseline scenarios

## §3 Compound scenarios (one subsection per compound)

For each compound (C1 through C8 + any additions):
  - What it tests
  - What bug it would catch
  - Cost estimate
  - Recommendation (include / defer / drop)

Aggregate: which N compounds land in Phase 9.

## §4 Cross-scenario structural gaps

For each gap:
  - What gap
  - Which scenarios affected
  - Options (a)/(b)/(c) with rationale
  - Recommendation

## §5 Flake-handling proposal

Recommendation: one of (a)/(b)/(c) with rationale. Default lean (c) per v2.0;
walk-back permitted only with code-grounded evidence.

## §6 Failure-mode catalogue

Full catalogue: each bug as (description, F-item, detection, severity).
Marked entries: caught by Phase 9 / not caught (feeds follow-on milestone).

## §7 Observability audit

For each gap:
  - What's missing
  - Which scenarios/compounds need it
  - Close-out recommendation (Phase 9 precondition / Phase 9 inline / defer)

## §8 Open questions for Joe

Anything the survey cannot resolve without Joe's input. Each question is one
paragraph + the alternatives Clair considered.

## §9 Survey methodology notes

Brief: what was read, what was traced, what was simulated mentally. Closes
Rule 2 — actual references, not speculation.
```

**The findings document is the survey's only deliverable.** No code changes. No Cargo.toml edits. No test scaffolding. Phase 9 implementation is a separate task file authored after Joe locks the findings.

---

## §5 — What Phase 9 implementation looks like AFTER this survey

This section is **not** Phase 9's task file — it sketches what the survey's findings feed into.

After the survey closes:

1. Joe + Chat Claude review findings.
2. Joe locks: per-scenario harness shapes + stress parameters; compound inclusions; structural-gap decisions; flake-handling decision; observability additions.
3. **If item 8 recommendation (c) is locked: Clair fixes both flakes as a separate commit before Phase 9 tests start.** This is precondition work, not Phase 9 work proper.
4. **If item 7 or item 10 surface observability/operator-control gaps recommended as (a) "Phase 9 precondition work": Clair lands those in separate commits before Phase 9 tests start.** Same pattern as flake fixes.
5. Chat Claude writes `tasks/FEDERATION_PROPAGATION_PHASE_9.md` (implementation task file) per locked decisions.
6. Clair executes Phase 9: writes the locked scenarios (~9-12 total) per locked harness shapes and stress parameters; applies failure-mode-catalogue-driven assertions; verifies observability surfaces work as expected.
7. `cargo test` passes; all scenarios green under sustained workspace parallelism (since flakes are fixed); milestone close commit.
8. CLAUDE.md + ROADMAP.md flip Federation Event Propagation milestone PLAY → DONE in the same commit.
9. M6 (new) unblocks.
10. Client-Side Consequences Audit runs as J-081-shape canonical doc, fed by the failure-mode catalogue's "not caught" entries.

The survey is **the lever** that determines what step 6 looks like. A thorough survey makes step 6 boring; a shallow survey makes step 6 produce surprises during a milestone-closing phase, which is the worst possible time for surprises.

**Time budget framing.** Phase 9's expected runtime grows from ~15s (J-059 baseline) to maybe 1-3 minutes under the strong-version scenarios + compounds. That's acceptable. Phase 9's *implementation* time grows from ~days (six simple scenarios) to ~weeks (12 strong scenarios + flake fixes + observability adds). That's also acceptable. The cost of *not* doing this work is shipping federation that breaks in production, which costs months of debugging plus credibility damage. **Priority is working functions, not done-mark on roadmap.**

---

## §6 — Operating discipline (restated from CLAUDE.md)

These rules apply throughout this survey.

**Rule 1 — Never fabricate results.** If a file or function being audited is not found, report that. Do not describe what it "should" contain.

**Rule 2 — Show actual output, not a description of output.** Every claim cites file:line. Every code reference quotes the actual code.

**Rule 3 — Stop and report when a tool fails.** If a search returns nothing or a file read fails, stop and report.

**Rule 4 — Write the findings document last.** The findings document is written after the per-scenario investigation is complete. Order: read mandatory sources → trace each scenario's surfaces → trace each compound's surfaces → enumerate failure modes → audit observability → form recommendations → write findings document.

**Rule 5 — Never invent numbers.** Don't estimate test runtimes, line counts, or flake rates by feel. Cite the source or say "unknown."

**Rule 6 — When in doubt, do less and ask.** If anything is ambiguous, file it as a §8 open question for Joe rather than deciding silently.

**Rule 7 — Definition of Done is a checklist, not a formality.** This task's DoD is below.

**No `commit pushed` checkbox.** The `Status: COMPLETED` header is the real ship signal.

**Joe pushes; Clair does not.** When the findings document is ready and Joe has reviewed, Joe pushes the commit.

---

## §7 — Definition of Done

- [ ] All mandatory reading in §1 completed.
- [ ] Six per-scenario sections written (§2.1 through §2.6), each with sub-items A/B/C/D/E plus a recommendation.
- [ ] Compound scenarios section written (§3), with at minimum the 8 starter compounds analysed + any additional compounds the trace surfaces; each compound has what-it-tests / what-bug-it-catches / cost / recommendation.
- [ ] Cross-scenario structural-gap section written (§4), with each gap surfacing what/affected-scenarios/options/recommendation.
- [ ] Flake-handling proposal written (§5) with one of (a)/(b)/(c) recommended and rationale; default lean (c).
- [ ] Failure-mode catalogue written (§6) with entries for every assertion Phase 9 will make; each entry has description/F-item/detection/severity.
- [ ] Observability audit written (§7) with gaps and close-out recommendations.
- [ ] §8 Open questions for Joe section written (may be empty if nothing surfaced).
- [ ] §9 Survey methodology notes written.
- [ ] `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` written with full project header and Status: PENDING.
- [ ] No code changes shipped in this task.
- [ ] No test runs needed for this task — but if `cargo test` was run for any reason, the count is quoted from actual output per Rule 5.
- [ ] JOURNAL.md entry written *after* the findings document is complete, summarising what the survey found and the key recommendations.
- [ ] CLAUDE.md updated to reflect "Phase 9 survey complete, awaiting Joe lock on findings."
- [ ] ROADMAP.md updated in same commit if a state change is reflected.

---

## §8 — Cross-references

- **`tasks/FEDERATION_PROPAGATION_COMPLETION.md`** §3.9 — Phase 9 scope as locked at runbook handoff. The baseline six scenarios are anchored here; the survey extends to ~10-12 total.
- **`docs/xgen_federation_propagation_design.md`** (v1.0, ACTIVE) — design doc; §15 records the eight shipped phases.
- **`docs/xgen_propagation_reliability.md`** (J-081 audit, ARCHIVED) — the audit that motivated the milestone. Re-read with adversarial framing.
- **`DECISIONS.md`** D-065 (honest behaviour over polite behaviour — applied to test results here, not just protocol behaviour), D-069 (delegated design discipline; surface gaps in design), D-070 (two events of equal importance), D-071 (subsystem audits precede dependent milestones).
- **CLAUDE.md** — current milestone state, MANDATORY behaviour rules, known-flake state.

---

*End of task file. Survey starts when Clair picks this up. Findings document is the only deliverable. Phase 9's strength as a bug-finding exercise is determined here, in the survey — not later, in the implementation.*  
