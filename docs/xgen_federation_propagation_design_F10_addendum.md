# XGen Federation Event Propagation — Design (F-10 addendum)

> **Status**: PENDING  
> Version: 0.1  
> Date: May 2026  
> **Last updated**: 2026-05-18 (F-10 surfaced and Joe-confirmed in Pass 2 conversation; final framework decision of Pass 2)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools. Pass 2 addendum to `docs/xgen_federation_propagation_design.md`; merged into the canonical document at Pass 3 per the D-069 canonical-document rule.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## About this addendum

Third and final Pass 2 addendum to `docs/xgen_federation_propagation_design.md`. Covers F-10 (DAG hole semantics on validation failure with unknown signer Identity). With F-10 locked, Pass 2 is complete.

Pass 2 addenda manifest:
- `docs/xgen_federation_propagation_design_F7_addendum.md` (F-7 pagination)
- `docs/xgen_federation_propagation_design_F8_F9_addendum.md` (F-8 + F-9 documentation correction timing)
- `docs/xgen_federation_propagation_design_F10_addendum.md` (this file; F-10 DAG hole semantics)

Pass 3 consolidates all addenda into the canonical design doc in a single careful rewrite. After Pass 3, all addenda are deleted.

---

## 13. Framework decision F-10 — DAG hole semantics on validation failure with unknown signer Identity

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

### 13.1 The question

F-3 locked: every federated event must pass both event-signature verification AND federation-relationship verification. F-4 locked: HeldPending applies uniformly to all event families when predecessors are unknown. But the audit (§3.3 Scenario C) identified a specific combination neither F-3 nor F-4 explicitly resolves:

**What does Node B do when a federated event arrives with BOTH (a) unknown predecessors (HeldPending case from F-4) AND (b) unknown sender Identity (signature cannot be verified because B does not hold the relevant Identity record)?**

This is the natural state for a new federation peer relationship's first events. When Node B accepts its first federation relationship for Space S:

- Node A pushes events for Space S to B.
- B has no Identity records for any of Space S's members yet (Identity replication is its own async pipeline).
- B has no predecessor events for Space S yet (whatever history exists at A).

Both checks F-3 requires fail simultaneously. F-10 specifies what B does in this state.

### 13.2 What "DAG hole" means

A "DAG hole" would be the state where B has accepted into local storage events whose validation status is undecidable because B is missing the data needed to validate them. The events exist in B's DAG but B cannot confirm they are authentic.

F-10's job is to decide whether DAG holes are permitted (and if so, how they get resolved), or whether the design must prevent them entirely by some other mechanism.

### 13.3 Options considered

**Option 1 — Reject.** F-3's signature check fails (no Identity record means no verification possible) → event is rejected. Recovery path: Identity replication delivers the record, then F-1a tip exchange or subsequent push re-delivers the event.

- ✅ Simplest; most strictly consistent with F-3.
- ✅ No tentative state in the DAG; storage only contains validated events.
- ❌ First-contact recovery is slow. A new federation relationship's first delivery wave hits this case for every event — every event is rejected and must be re-pulled after Identity replication catches up. Doubles bandwidth.
- ❌ Does not compose well with F-1a tip exchange. The tip exchange delivers history; under Option 1, all of that history is rejected on first delivery (no Identity records) and re-fetched after Identity replication. Two-pass instead of one-pass.

**Option 2 — HeldPending (extend F-4's mechanism).** Apply HeldPending uniformly: buffer the event pending arrival of predecessors AND Identity records. F-4's existing mechanism already handles unknown predecessors; F-10 generalises the trigger to include unknown Identity records.

- ✅ Reuses an existing mechanism. No new buffer, no new state, no new retry path.
- ✅ Handles first-contact gracefully. Events arrive, get buffered, Identity records arrive via Identity replication, signature verification passes, events get ingested. One-pass.
- ✅ Preserves the "every event in storage has been validated" invariant.
- ⚠️ HeldPending was designed for unknown predecessors. Extending it to also handle unknown signers widens its responsibility — but only conceptually; the data structure is the same.
- ⚠️ Memory bound. A flood of first-contact events all in HeldPending pending Identity replication holds meaningful memory until either the records arrive or the 30s timeout fires.
- ⚠️ Identity replication is async and out of F-10's scope. If Identity replication is consistently slower than the HeldPending timeout in some deployment, events get dropped via timeout and re-pulled on next sync. Correct, but slow.

**Option 3 — Tentative storage with retroactive validation.** Store the event in B's DAG marked unvalidated. Do not fan out to local clients. When the Identity record arrives, validate retroactively. If validation passes, promote and fan out. If validation fails, delete from storage.

- ✅ Best preservation of first-contact event order; predecessors form naturally without HeldPending churn.
- ❌ Introduces a "tentatively unvalidated" state in storage. New conceptual state, new failure modes, new queries.
- ❌ Retroactive deletion is a sharp edge. If a tentative event is later deleted because signature verification failed, anything that referenced it (a HeldPending child event, a downstream consumer) needs cleanup too. Cascade complexity.
- ❌ Breaks the "every event in storage has been validated" invariant. Downstream code (state machine, queries, federation push) trusts that invariant. Breaking it cascades.

### 13.4 Decision — Option 2 (extend HeldPending to handle unknown signer Identity)

**HeldPending's trigger condition is generalised from "unknown predecessor" to "unknown predecessor OR unknown signer Identity OR both."** When an event arrives at `process_inbound` and either dependency is missing, the event enters HeldPending. The buffer waits for both dependencies to arrive. When all dependencies are satisfied, the event is re-routed through the validation core (F-4 §7.4), passes signature and timestamp checks, and is ingested normally.

If the timeout fires before all dependencies arrive, the event is discarded. Recovery is via F-1a tip exchange on the next session re-establishment.

The data structure for HeldPending stays the same; only the retry-trigger condition gains one more arrival event to watch for (Identity record arrival, in addition to predecessor arrival).

**Reasoning recorded.**

1. **Reuses an existing mechanism.** F-4 already specified HeldPending applies uniformly to all event families. Extending the trigger condition is a small generalisation — the buffer, the timer, and the retry path are the same. No new state machine, no new storage shape, no new query path.

2. **Handles first-contact naturally.** New federation relationships hit this case at full volume. Option 1 would force every first-contact event through a reject-then-re-pull cycle (double bandwidth). Option 2 lets events queue, Identity records flow in via replication, and the queued events validate and ingest as records arrive — one-pass.

3. **Preserves the "every event in storage has been validated" invariant.** This is a load-bearing invariant for downstream code. Option 3 would break it; Option 2 keeps it because events stay in HeldPending (not in storage) until validated.

### 13.5 Sub-decision F-10a — Timeout policy for the Identity-missing case

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

**Decision — same as F-4a: 30 seconds uniform in v1, implementation-configurable.**

The reference implementation defaults to 30 seconds uniform across all HeldPending cases (unknown predecessors, unknown signer Identity, or both). Implementation-configurable via the same field F-4a specified (whatever the runbook chooses — `[validation].heldpending_timeout_seconds` or similar).

**v2 evolution path.** If deployment data shows Identity-missing cases need a longer window than predecessor-missing cases — for example, if Identity replication is consistently slower than DAG predecessor delivery in some federation topology — per-trigger-condition configuration can be added in v2 without requiring a protocol change. The mechanism stays the same; only the timeout granularity changes.

**Reasoning recorded.** Same as F-4a:

1. HeldPending is a short-window optimisation, not a durability guarantee. The durability mechanism is the F-1a tip exchange on next reconnect.
2. Uniform timeout means one timer, one set of edge cases to test, simpler observability.
3. v2 configurability path exists for the case where data justifies it. v1 stays simple.

### 13.6 What happens in the "Identity record never arrives" case

If Identity replication is stalled, broken, or never delivers the missing Identity record, the HeldPending timeout fires after 30s and the buffered event is discarded. The next sync_request or F-1a tip exchange re-delivers the event; if Identity replication is still broken at that point, the event hits HeldPending again and times out again — a loop.

This is correct behaviour. The loop continues until Identity replication is fixed (operator intervention or upstream debugging). At no point are events silently accepted without validation; at no point is B's storage corrupted; the failure is loud (events keep dropping into HeldPending, observability shows the buildup) and recoverable.

The audit should flag this as a v1 limitation: F-10 + Identity replication assumes Identity replication is operationally healthy. If it is not, federation event ingestion stalls until it is.

### 13.7 Implementation-runbook notes from F-10

- HeldPending's trigger condition is the only thing that changes. The buffer, the timer, the retry path, the data structure, the discard-on-timeout behaviour — all unchanged.
- The retry trigger needs to watch for two arrival events: predecessor arrival (existing, from F-4) AND Identity record arrival (new). The Identity-arrival hook needs to fire when a new Identity record lands via replication.
- Integration test coverage should include: (a) Identity record arrives within timeout → event validates and ingests; (b) predecessors arrive within timeout, Identity record arrives later but still within timeout → event validates on second retry; (c) Identity record never arrives, timeout fires, event discarded, next sync re-delivers; (d) both predecessors and Identity record missing → event waits for both, validates when both arrive.
- The "Identity replication health" concern (§13.6) should be surfaced in Node-side observability. A metric like "events currently in HeldPending pending Identity record" exposed to the admin UI lets operators see when Identity replication is the bottleneck. Exact metric design is runbook's call.
- This decision implicitly couples the federation push milestone to Identity replication's reliability. The runbook should explicitly call this out so it does not surprise anyone debugging later.

---

## F-10 lock state

| Sub-item | Decision |
|---|---|
| F-10 | Option 2 — extend HeldPending trigger to handle unknown signer Identity. Reuse F-4 mechanism. |
| F-10a | Same as F-4a: 30s uniform in v1, implementation-configurable, per-trigger-condition path available in v2 |

Pass 3 folds this section into the canonical design doc as §13, then deletes this addendum file.

---

## Pass 2 — complete

With F-10 locked, all ten framework decisions are confirmed:

| F-item | Topic | Decision summary | Location |
|---|---|---|---|
| F-1 | Push direction | Hybrid (push for steady state, pull for gap recovery) | Main design doc §4 |
| F-1a | Initial handshake | Tip exchange replaces full history dump | Main §4.4 |
| F-1b | Buffering on peer-down | Drop, recover via pull | Main §4.5 |
| F-1c | Per-peer record | Node-implementation persistent state, global backoff reconnect | Main §4.6 |
| F-2 | Session model | Long-lived continuous | Main §5 |
| F-2 lifecycle | Session boundaries | Opens on handshake, closes on goodbye/keepalive/error, fresh on re-establishment | Main §5.4 |
| F-2a | Session topology | One WebSocket per pair, bidirectional | Main §5.5 |
| F-3 | Identity authority | Event signature + federation relationship verification | Main §6 |
| F-4 | Validation asymmetry closure | Unified validation core + per-type handlers | Main §7 |
| F-4a | HeldPending timeout | 30s uniform v1, configurable v2 | Main §7.5 |
| F-4b | Pre-check placement | Structural before validation, semantic after | Main §7.6 |
| F-5 | Transitive federation | Locked-out v1, Option 3 v2 evolution path documented | Main §8 |
| F-6 | sync_complete | Fold in (`SyncComplete { since, new_tip }`) | Main §9 |
| F-6b | Safety-net timeout | 5s default, configurable | Main §9.5 |
| F-7 | Pagination | Fold in (response-size pagination with cursor) | F-7 addendum |
| F-7a | Page size | 1000 default, configurable | F-7 addendum |
| F-8 | Ch4 correction timing | Correct at Pass 3, forward-reference design doc | F-8+F-9 addendum |
| F-9 | Admin-ops doc correction timing | Same as F-8 | F-8+F-9 addendum |
| F-10 | DAG hole semantics | Extend HeldPending to include unknown signer Identity | This addendum |
| F-10a | Identity-missing timeout | Same as F-4a (30s uniform v1, configurable v2) | This addendum |

Pass 2 is complete. Pass 3 follows: consolidate all addenda into the canonical design doc, walk every `[JOE-LOCK]` marker for final lock, flip Status from PENDING to ACTIVE, correct the Ch4 and admin-ops-doc drifted text in the same commit, then write the implementation runbook (`tasks/FEDERATION_PROPAGATION_COMPLETION.md`) for Clair, then flip CLAUDE.md and ROADMAP.md to ACTIVE for the milestone.

---

*End of F-10 addendum.*  
