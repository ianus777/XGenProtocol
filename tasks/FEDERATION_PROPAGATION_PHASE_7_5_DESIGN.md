# Federation Event Propagation — Phase 7.5 Design (Cold-Start Bootstrap)
> **Status**: ACTIVE  
> Version: 0.1 (draft, awaiting Joe-lock walkthrough)  
> Date: May 2026  
> **Last updated**: 2026-05-19 (initial draft — single-pass design task file for the Phase 7.5 cold-start bootstrap fix surfaced during Phase 9 Scenario 1 setup)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This document is the design task file for **Phase 7.5 of the Federation Event Propagation milestone** — a small, focused milestone phase that closes a protocol-level gap surfaced during Phase 9 Scenario 1 implementation: **brand-new federation bootstrap is dead-locked under the post-Phase-7 F-3 gate**.

Phase 7.5 sits between Phase 8 (documentation pass, closed in J-089) and Phase 9 (deployment integration tests, currently paused). It is a precondition for Phase 9 because three of Phase 9's six DoD scenarios (push smoke, anti-transitivity, drop-and-recover) require brand-new federation bootstrap to work end-to-end. Without Phase 7.5, those scenarios cannot be tested except in artificial setups that pre-load Space state on the receiver — which would ratify nothing about production cold-start behaviour.

This is a design task file, not an implementation runbook. The runbook (Clair-facing, with code-level commit sequence) is a follow-on artefact produced after this design phase closes per the D-069 canonical-document rule.

---

## 2. Provenance — how the gap surfaced

Phase 9 Scenario 1 ("two-Node push smoke") was the first scenario Clair attempted to implement after Phase 9 IMPL went LOCKED in J-091. The scenario requires: spawn two Nodes, establish federation between them, post a message on Node A, verify it propagates to Node B.

When Clair set up the federation handshake from a brand-new Node B to Node A for a Space A hosts, B's `process_inbound` stream produced this cascade of rejections during the F-1a delta stream:

```
ERROR event_rejected event_type=state.space_create    reason="federation_relationship_missing: peer <A> has no federation relationship for Space <S>"
ERROR event_rejected event_type=state.room_create     reason="space not found: <S>"
ERROR event_rejected event_type=membership.invite     reason="space not found: <S>"
ERROR event_rejected event_type=state.federation_add  reason="space not found: <S>"
ERROR event_rejected event_type=membership.join       reason="space not found: <S>"
```

The chain:

1. A's `stream_federation_delta` ships Space S's full history to B per Phase 3's F-1a tip exchange. `state.federation_add` (freshly built by A under the a-i symmetry rule) is last in topological order because it references the latest tip as `prev_events`.
2. B receives `state.space_create` first. Phase 7's F-3 gate consults `SpaceState.federation_nodes` for Space S. Space S doesn't exist on B yet (this is bootstrap), so the lookup returns None, mapped to `relationship_ok = false`. F-3 rejects with `federation_relationship_missing`.
3. Every subsequent event in the stream gets "space not found" from F-4 step 1 because Space S was never created (the `state.space_create` that would have created it got rejected upstream).
4. `state.federation_add` — the one event Phase 7's B1 skip rule was designed to let through F-3 — also fails F-4 step 1 for "space not found" before F-3 even runs.

Net result: **a brand-new federation peer can never bootstrap a Space from a remote.** The path that Phase 5's reconnect scheduler + Phase 3's bilateral tip exchange + Phase 4's federation push + Phase 7's F-3 gate all assumed works does not work for the cold-start case.

This matches failure-mode catalogue entry M5 ("F-3 brand-new federation bootstrap dead-locked") from the Phase 9 survey (J-090). Scenario 1 walked into it as the survey predicted, just earlier in the implementation than expected — during harness setup rather than under the test assertion phase.

---

## 3. What the existing design considered, and what it missed

The federation propagation design phase (Pass 2 + Pass 3, J-081-precedent through canonical doc v1.0 ACTIVE) was thorough. It explicitly considered three chicken-and-egg cases between gates and events that establish the gate's data:

| Chicken-and-egg | Resolved by | Locked at |
|---|---|---|
| F-3 vs. `state.federation_add` (sender not in `federation_nodes` yet — that's what the event is about to add) | B1 skip rule: F-3 skips for `state.federation_add` events | Phase 7 Lock B1 |
| F-4 step 2 (predecessor presence) vs. delivery order | F-4a HeldPending: buffer event pending predecessor arrival, 30s timeout, recovery via F-1a on reconnect | F-4 (Phase 2) |
| F-3 vs. unknown signer Identity (Identity replication is async and may lag federation push) | F-10 HeldPending generalisation: buffer event pending Identity arrival, same timeout policy | F-10 (Phase 6) |

The design did NOT explicitly consider a fourth case:

| Chicken-and-egg | Status |
|---|---|
| **F-4 step 1 (Space exists locally) AND F-3 vs. `state.space_create` (Space doesn't exist yet — that's what the event is about to create)** | **Not resolved. Gap surfaced in J-091 Phase 9 Scenario 1 setup.** |

The design partially anticipated something like this — F-10 §13.3 explicitly noted that for a new federation relationship "B has no Identity records for any of Space S's members yet" and "B has no predecessor events for Space S yet." But the framing was about events that depend on prior events or unknown signers, not about the Space itself not existing. F-3's data source (`SpaceState.federation_nodes`) presupposes Space S exists locally; nobody asked what happens when it doesn't.

Phase 7's B1 skip is narrowly scoped to `state.federation_add`. It doesn't cover `state.space_create`. And even if F-3 were skipped for `state.space_create`, F-4 step 1 still rejects events for unknown Spaces, including `state.space_create` itself.

**This is a genuine design gap, not an implementation error.** Phase 7.5 closes it.

---

## 4. Scope

### 4.1 In scope

- **Resolve the cold-start bootstrap chicken-and-egg** for `state.space_create` arriving via federation to a Node that has no local record of the target Space.
- **Coordinate the resolution with F-3 (Phase 7), F-4 (Phase 2), and F-10 (Phase 6)** so the existing gates remain correct for all other event flows.
- **Preserve the audit's HIGH-severity finding closure** — F-3's per-event federation-relationship enforcement must remain operationally active for non-bootstrap event flows. Phase 7.5 cannot weaken F-3's posture.
- **Document the lock decisions** in the canonical design doc (`docs/xgen_federation_propagation_design.md`) as a sibling subsection to F-3's existing §6.4 + Lock B1 + Lock B2 framing.

### 4.2 Out of scope

- **Reordering of `stream_federation_delta`'s output.** Phase 3's a-i symmetry rule and topological ordering stay as shipped. Sender-side behaviour is unchanged.
- **Multi-tip DAG semantics for `state.federation_add`.** Phase 7.5 does not introduce DAG-shape changes; `state.federation_add` keeps its current `prev_events` semantics (references the latest tip at delta-stream build time).
- **Wire-shape changes.** No additions to `TransportMessage` variants, no changes to F-1a tip exchange shape.
- **Identity replication coupling.** F-10's Identity-replication coverage is unchanged. The Phase 7.5 trigger is independent of F-10's triggers (they may compound in a single HeldPending entry; see §6.4 for the combination rule).
- **Phase 9 implementation.** Phase 9 stays paused until Phase 7.5 closes and ships. Phase 9 scenarios then exercise the fix as end-to-end verification.

### 4.3 Non-scope decisions explicitly recorded

These are deliberately deferred to keep Phase 7.5 narrow:

| Item | Why deferred | Where it would land |
|---|---|---|
| Sender-side stream reordering (Option Y from design conversation) | Would create multi-tip DAG per federation; couples to F-1a / F-6a wire-shape clarification; high implication footprint | If ever needed: future protocol-evolution milestone, separate scope |
| Session-flag bootstrap window (Option X.b from design conversation) | Bypasses F-3 during bootstrap window; weakens post-audit trust model | Not pursued; documented in §5.4 for posterity |
| Per-trigger HeldPending timeout granularity at the data-structure level | F-10a forecasted this as a v2 evolution; Phase 7.5 uses uniform timeout per Lock D below | Future scaling milestone if data justifies |

---

## 5. Framework decision P7.5-A — narrow skip rule for Space-create EventTypes at F-4 step 1 + F-3

`[JOE-LOCK: pending — awaiting walkthrough]`

### 5.1 The question

`state.space_create` and `state.dm_space_create` arriving over a federation session must be allowed to land at a receiver that does not yet host the target Space — otherwise the Space can never come into local existence and the bootstrap chain cannot start.

The narrowest possible accommodation: extend the existing B1 skip pattern (Phase 7) to cover these two additional EventTypes at both F-4 step 1 and F-3.

### 5.2 What the rule says

The unified validation core in `NodeRuntime::dispatch_event` (per F-4 §7.7) executes these gates in order. Phase 7.5 introduces narrow skip rules at the first two:

**F-4 step 1 (structural pre-check, currently rejects unknown Space).** Skip the "Space exists locally" check when the incoming event type is `state.space_create` or `state.dm_space_create`. By structural necessity, these events bring the Space into existence; the check cannot apply to them.

**F-3 (federation-relationship verification, currently rejects unknown peer).** Skip the `SpaceState.federation_nodes` check when the incoming event type is `state.space_create` or `state.dm_space_create`. By structural necessity, no `federation_nodes` list exists for the Space yet (the Space doesn't exist locally). The check cannot apply to them.

Both skips are applied only for these two specific EventTypes. All other event types continue to flow through F-4 step 1 and F-3 unchanged. Phase 7's B1 skip for `state.federation_add` is preserved unchanged.

### 5.3 Authority for the skip

`state.space_create` arriving via a federation session carries two authority claims:

1. **Session-level handshake auth.** The sender (peer Node A) authenticated with A's Node keypair during the F-1a handshake. The receiver knows with cryptographic certainty that this session is to Node A.
2. **Event-level signature.** The event is signed by its author (typically the Space's founding Identity). The receiver can verify this signature **only if** it holds the relevant Identity record — which it may not, at cold-start time. If the Identity record is not yet replicated, the event enters HeldPending via F-10's existing mechanism (Identity-arrival trigger), where it waits for Identity replication to deliver the record. **Signature verification is not skipped — only the structural Space-existence and federation-relationship checks are skipped.** Once the Identity arrives, the event re-validates and the signature is checked.

This is the same authority basis Phase 7 used for B1's skip of `state.federation_add`. The rule extends naturally: events that bring the structure they reference into existence carry intrinsic authority for the structure-creation, while remaining subject to signature verification through F-10's existing buffering.

### 5.4 Options considered

**Option α — skip at F-4 step 1 + F-3 for Space-create EventTypes (selected).** As described in §5.2. Narrow scope, sibling to B1, structurally clean. F-3's enforcement for all other event types remains active. Signature verification still applies (via F-10 if Identity unknown). Decision locked at §5.6 below.

**Option β — broad skip ("DAG-root events").** Apply the skip to all DAG-root EventTypes (`state.space_create`, `state.dm_space_create`, `state.room_create`). Rejected: `state.room_create` is a DAG root but creates a Room nested under a Space; if the Space doesn't exist locally, `state.room_create` SHOULD be rejected. The discriminator is not "DAG root" — it's "creates the Space it references." Only `state.space_create` and `state.dm_space_create` satisfy that.

**Option γ — session-flag bootstrap window (Option X.b from design conversation).** Track per-(peer, space) handshake-in-progress state; bypass F-3 for all events during the window. Rejected: weakens F-3 to pre-audit semantics during exactly the moment trust most matters (first contact). Couples the handshake protocol to receiver-side trust state in a load-bearing way. Documented here for posterity but not pursued.

**Option δ — sender-side stream reordering (Option Y from design conversation).** Mint `state.federation_add` with `prev_events = [state.space_create.event_id]` so it lands as a structural sibling of the Space root, allowing it to be sent immediately after Space-create. Rejected: introduces multi-tip-per-Space DAGs as a normal feature; propagates implications through F-1a / F-6a wire shapes; opens "DAG-root-referencing events" as a precedent pattern; non-reversible (federation_add events with non-tip `prev_events` would persist in archives). Documented here for posterity but not pursued.

### 5.5 Why Option α beats the alternatives

The choice was made by comparing implications across four dimensions: trust model, DAG topology, wire shape, reversibility.

- **Trust model.** Option α preserves F-3's enforcement for every event flow except the two structurally-special EventTypes. Option γ weakens F-3 during the bootstrap window. Option α is strictly stronger.
- **DAG topology.** Option α leaves DAG topology unchanged. Option δ makes multi-tip-per-Space normal. Option α keeps existing mental models.
- **Wire shape.** Option α requires no wire changes. Option δ requires clarifying `BTreeMap<space_id, tip>` semantics in F-1a (does "tip" mean content tip or any tip?). Option α stays clean.
- **Reversibility.** Option α is a receiver-side implementation rule; if it ever needs to change, the change is internal. Option δ embeds non-standard `prev_events` references in events that would persist in archives.

### 5.6 Decision

**Option α (skip at F-4 step 1 + F-3 for `state.space_create` and `state.dm_space_create` EventTypes).** Locked at §5.2's rule. Verbatim code-comment block at both skip sites, citing Phase 7.5 §5.

`[JOE-LOCK: pending — awaiting walkthrough]`

---

## 6. Framework decision P7.5-B — HeldPending third trigger for missing federation relationship

`[JOE-LOCK: pending — awaiting walkthrough]`

### 6.1 The question

After P7.5-A lets `state.space_create` land, the Space exists locally on the receiver. But all the events that arrive next in the bootstrap stream — `state.room_create`, `membership.invite`, `state.federation_add`, `membership.join`, `message.*` — pass F-4 step 1 (Space now exists) and then **fail F-3** because the peer is not yet in `SpaceState.federation_nodes` for the new Space. (Phase 7's B1 skip lets `state.federation_add` through F-3, but the other events have no such skip.)

These events cannot be naively rejected — they are legitimate bootstrap content arriving in topological order. They also cannot bypass F-3 — that would weaken the post-audit trust model exactly during first contact.

The right answer is the same shape F-4 and F-10 already use: **HeldPending the events until the missing dependency (federation relationship) arrives.**

### 6.2 What the rule says

The HeldPending data structure (currently in `xgen-core/src/dag/pending.rs`) gains a third trigger condition alongside the two F-10 generalised it to:

| Trigger | Resolved by | Existing in |
|---|---|---|
| Missing predecessors | Predecessor event arrival | F-4 (Phase 2) |
| Missing Identity record | Identity replication arrival | F-10 (Phase 6) |
| **Missing federation relationship for (peer, space)** | **`state.federation_add` event ingestion** | **Phase 7.5 (this lock)** |

When F-3 fails for a federation-channel event because the peer is not in `SpaceState.federation_nodes[space]`, the event is held in HeldPending instead of being rejected. The held entry records the (peer_node_id, space_id) pair it is waiting for.

When `state.federation_add` is later ingested locally for that (peer, space) pair, an arrival hook fires (analogous to F-10's Identity-arrival hook). All HeldPending events for that pair re-validate through the unified validation core. F-3 now passes (the peer is in `federation_nodes`). Events flow through the rest of the pipeline normally — semantic pre-checks, ingest, fan-out.

### 6.3 Combination with F-10 (events missing both federation relationship AND Identity record)

During cold-start bootstrap, an event commonly arrives with BOTH missing dependencies:

- Federation relationship not yet established (this lock's trigger).
- Identity record for the signer not yet replicated (F-10's trigger).

The HeldPending entry's data structure already supports `Option<...>` fields for each trigger condition (F-10's `missing_identity: Option<String>`). Phase 7.5 adds an analogous `missing_federation_relationship: Option<(String, String)>` (peer_node_id, space_id). An event missing both has both fields populated; resolution requires both arrivals.

The unified validation core (F-4 §7.4) is the re-validation entry point regardless of which trigger originally caused the HeldPending. Order of arrivals does not matter — federation_add can arrive before Identity, or after, or simultaneously; the buffer waits for both.

**Predecessor-code-wins rule extension (per F-10 §13's locked sub-rule for predecessor + Identity overlap).** If the event also has unknown predecessors AND the HeldPending entry times out, the timeout error code is `4002 predecessor_timeout` (predecessor takes precedence). If only the Identity is missing at timeout, the code is `4006 identity_record_timeout` (F-10's lock). If only the federation relationship is missing at timeout, the code is **`4007 federation_relationship_timeout`** (Phase 7.5 — new code in domain 4000-4999 state resolution).

### 6.4 Why this is not weakening F-3

A subtle point worth recording explicitly: HeldPending'ing an event that failed F-3 is not the same as bypassing F-3. The event is not accepted into storage, not fanned out, not visible to anything downstream until F-3 passes on re-validation. The buffer is a holding cell, not a back-channel.

The trust model is identical to F-10's: events arrive, dependencies are missing, the buffer holds them, dependencies arrive, the buffer drains, events validate cleanly. At no point is an event accepted without its full validation passing.

The only "weakening" relative to a strict rejection policy is that bootstrap delays validation rather than rejecting on first try. But the validation that eventually runs is the same validation, with the same data sources, producing the same yes/no answer. F-3 is not skipped; it is **deferred until its data source is populated.**

### 6.5 Decision

**Add a third HeldPending trigger condition: "missing federation relationship for (peer, space)".** Resolved by `state.federation_add` arrival hook. New error code `4007 federation_relationship_timeout` for the case where the federation_add never arrives within the timeout window. Combination semantics with F-10's Identity trigger and F-4's predecessor trigger as described in §6.3.

`[JOE-LOCK: pending — awaiting walkthrough]`

---

## 7. Framework decision P7.5-C — HeldPending timeout for the federation-relationship trigger

`[JOE-LOCK: pending — awaiting walkthrough]`

### 7.1 The question

F-4a locked the predecessor-trigger timeout at 30 seconds uniform across event families. F-10a locked the Identity-trigger timeout at the same 30 seconds, with v2 evolution path to per-trigger configuration "if Identity replication is consistently slower than predecessor delivery in some federation topology."

The federation-relationship trigger surfaces a new question: **is 30 seconds the right timeout for waiting for `state.federation_add` during bootstrap?**

Two considerations differentiate this trigger from the existing two:

1. **Bootstrap streams can be large.** A Space with months of history can have thousands of events. The full delta might take 30+ seconds to deliver across realistic WAN latency, especially with F-7 pagination at 1000 events per batch. The early events in the stream would time out before `state.federation_add` (last in topological order) arrives.

2. **`state.federation_add` arrival is bounded by stream delivery, not by an independent async pipeline.** F-10's Identity-replication trigger waits for an unrelated async system to deliver records; the wait could be unboundedly long if Identity replication is broken. The federation-relationship trigger waits for a specific event arriving in the same stream that delivered the held events. If the stream completes (`SyncComplete` received), `state.federation_add` either arrived in it or didn't; there is no "still pending" state after stream completion.

### 7.2 Options considered

**Option α — uniform 30s timeout (matches F-4a / F-10a).** Predecessor + Identity triggers stay at 30s; federation-relationship trigger also at 30s. Rejected: realistic bootstrap streams routinely exceed 30 seconds. Early-arriving events would time out before federation_add lands.

**Option β — extended uniform timeout (e.g., 120s) for all three triggers.** All HeldPending timeouts extend. Rejected: predecessor and Identity triggers don't need extension; extending them masks slow-Identity-replication failure modes that the 30s timeout currently surfaces.

**Option γ — per-trigger timeout, federation-relationship gets a longer default (e.g., 120s) configurable.** F-4a's predecessor trigger and F-10a's Identity trigger keep 30s. Federation-relationship trigger gets 120s default with config field. Selected.

**Option δ — bind timeout to F-1a session lifetime.** Held entries waiting for federation-relationship don't time out as long as the F-1a delta stream is in progress with the relevant peer. Timeout fires only after `SyncComplete` arrives (or session disconnects) without `state.federation_add` having landed. Rejected: more elegant but couples HeldPending to session state in a way that complicates the buffer's data structure and adds a new arrival hook ("session ended without federation_add"). Option γ achieves the same operational outcome with simpler mechanics.

### 7.3 Decision

**Per-trigger timeout configuration, federation-relationship trigger defaults to 120 seconds, configurable via new field `[sync].federation_relationship_timeout_seconds`.** F-4a's predecessor timeout and F-10a's Identity timeout remain at 30 seconds each (unchanged from their respective locks).

This is the v2 evolution path F-10a forecasted, brought forward to v1 by Phase 7.5's introduction of a third trigger with materially different timing characteristics.

**Reasoning recorded.**

1. **Bootstrap is the case where waiting is normal, not pathological.** Unlike predecessor or Identity arrival (which should be fast in steady state), federation_add arrival is bounded by stream-delivery time during cold-start. A 30s timeout misclassifies normal bootstrap as failure.

2. **Per-trigger granularity is cheaper to implement now than to retrofit.** F-10a's evolution path documented this; Phase 7.5 is the natural moment.

3. **120s default is generous but bounded.** A bootstrap stream that hasn't delivered `state.federation_add` within 120 seconds either (a) hit a real failure (sender crashed, session dropped) or (b) is delivering a multi-tens-of-thousands-event Space history at slow throughput. Case (a) wants the timeout to fire; case (b) wants a longer config override. Both are served by a configurable 120s default.

`[JOE-LOCK: pending — awaiting walkthrough]`

---

## 8. Framework decision P7.5-D — Observability for the new HeldPending trigger

`[JOE-LOCK: pending — awaiting walkthrough]`

### 8.1 The question

F-10 added `pending_identity_replication: usize` to `xgen-node_state.json` so operators can detect "Identity replication is the bottleneck" by polling the state file. Phase 7.5's third trigger needs an analogous counter.

### 8.2 Decision

Add `pending_federation_relationship: usize` to `NodeState` (in `xgen-common/src/state.rs`), populated by `build_node_state` summing each Space's HeldPending count for the federation-relationship trigger condition. `#[serde(default)]` for forward-compat with pre-Phase-7.5 state files.

Phase 9's observability preconditions (G1, G2, G3 from the survey) and Phase 7.5's counter together give operators visibility into the bootstrap state: "this Node is currently receiving a bootstrap stream from peer X for Space S; N events are held pending federation_add arrival." Plus the existing log lines from F-4 / F-10 / F-3 cover the events themselves.

No new trace events beyond what Phase 9 Commit 1 already plans (G2's `f3_reject` trace event is the relevant hook; Phase 7.5 changes its semantic from "rejected" to "held pending" but the trace point is the same).

`[JOE-LOCK: pending — awaiting walkthrough]`

---

## 9. Implementation runbook handoff

After Phase 7.5 design locks (this document promoted to canonical, all four `[JOE-LOCK]` markers walked to final form), the implementation runbook is produced as a separate Clair-facing task file.

### 9.1 Expected runbook scope

Roughly five commits, in order:

1. **Doc-pass commit.** This task file flipped to COMPLETED. Canonical design doc (`docs/xgen_federation_propagation_design.md`) gains a new §6.4.1 sibling subsection covering Phase 7.5's locks alongside Phase 7's B1 + B2 framing. §15 Implementation Complete table gains a Phase 7.5 row.
2. **F-4 step 1 + F-3 skip implementation.** Two skip sites in `NodeRuntime::dispatch_event` and the validation core. Verbatim code-comment blocks at both. Unit tests for both EventTypes (`state.space_create`, `state.dm_space_create`).
3. **HeldPending third trigger implementation.** Data structure extension (new `missing_federation_relationship: Option<(String, String)>` field on the held entry). New arrival hook in `state.federation_add` ingestion path. New `drain_pending_by_federation_relationship` analogous to F-10's `drain_pending_by_identity`. New `[sync].federation_relationship_timeout_seconds` config field. New error code `4007 federation_relationship_timeout` (next-free after `4006`). Observability counter in `xgen-node_state.json`.
4. **Integration tests at NodeRuntime level.** Cold-start bootstrap end-to-end (the scenario Clair could not get past). Mid-bootstrap session drop and resume (events held, session dies, new session re-delivers, events drain on second federation_add arrival). Combination with F-10: bootstrap event with both missing Identity and missing federation relationship, validates when both arrive.
5. **Phase 7.5 close commit.** ROADMAP.md Past entry created. CLAUDE.md PLAY block reflects Phase 7.5 ✅ + Phase 9 ready to resume. Test count updated.

Phase 9 then resumes with Scenario 1 unblocked.

### 9.2 Joe-lock items the runbook itself may surface

Some implementation-level questions could surface during runbook authoring or Clair's implementation work. None are anticipated at design time, but the canonical-document discipline applies: any wire-shape-visible or trust-model-visible decision that surfaces during implementation requires Joe-lock before code lands. Implementation-internal decisions (data-structure field names, internal function signatures, test helper shapes) are Clair's latitude.

---

## 10. Definition of Done

Phase 7.5 design phase is complete when:

- [ ] All four framework decisions (P7.5-A through P7.5-D) have their `[JOE-LOCK]` markers walked from `pending — awaiting walkthrough` to final form `[JOE-LOCK: locked YYYY-MM-DD]`.
- [ ] Canonical design doc (`docs/xgen_federation_propagation_design.md`) updated with the new locks recorded as §6.4.1 or sibling structure.
- [ ] CLAUDE.md PLAY block updated to reflect Phase 7.5 design closed, implementation runbook authored, awaiting Clair pickup.
- [ ] ROADMAP.md updated: Present section reflects Phase 7.5 implementation ready; Past section gains Phase 7.5 design entry; Phase 9 still paused until Phase 7.5 ships.
- [ ] Implementation runbook task file (`tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` or similar — exact path locked at runbook authoring time) authored at Status: ACTIVE, v1.0.
- [ ] This task file flipped to Status: COMPLETED in the same commit that ships the canonical design doc updates.

No code changes during Phase 7.5 design phase. Test count unchanged at 519.

---

## 11. Cross-references

- `docs/xgen_federation_propagation_design.md` — Canonical design doc (v1.0 ACTIVE). Phase 7.5 locks land as §6.4.1 sibling subsection.
- `tasks/FEDERATION_PROPAGATION_PHASE_9.md` — Phase 9 implementation task file (ACTIVE v1.0). Paused pending Phase 7.5 closure.
- `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` — Phase 9 survey (COMPLETED v1.1). Failure-mode catalogue M5 named this gap; Phase 7.5 is its closure.
- `docs/xgen_propagation_reliability.md` — J-081 audit. Phase 7.5 inherits the audit-precedes-dependency discipline (D-071) one phase deeper.
- D-065 (honest behaviour over polite behaviour) — informs Lock P7.5-B (held-not-bypassed posture).
- D-069 (Joe-locked design phase + canonical-document rule) — discipline that produced this document.
- D-071 (subsystem audits precede dependent milestones) — extends to "design gaps surface during dependent work and close before the dependent work proceeds."

---

*End of document. Design phase pending Joe-lock walkthrough of P7.5-A through P7.5-D.*  
