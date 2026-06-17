# Federation Bidirectional `federation_nodes` Design
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-21 (Status flipped ACTIVE → COMPLETED at implementation runbook Commit 1 — design phase's role as "active input" ends as the runbook lands canonical-doc §6.4.2 + §15 row. Three Joe-locks (Q1 Reading (i), Shape A, sub-option A.1) and D-075 promotion preserved as authoritative record. Locked content below is the canonical historical record of the framework decisions made during the design walkthrough closed same-day as the audit doc shipped. Sibling-in-shape to `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` (COMPLETED v1.0). Per D-069 (canonical document) + D-071 (audit precedes dependent design) + D-074 (forward-binding) + D-075 (the protocol-design principle this design phase locked). Implementation runbook at `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (ACTIVE v1.0) carries the four-commit Clair sequence.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This document is the design task file for the **bidirectional `federation_nodes` phase** — a small, focused milestone phase that closes a protocol-level gap surfaced during Phase 9 Commit 3a's Scenario 1 diagnostic run. It sits between the audit doc (`tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md`, Status ACTIVE v1.0 — flips COMPLETED in implementation Commit 1) and the implementation runbook (`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`, Status ACTIVE v1.0).

This is a design task file, not an implementation runbook. The runbook (Clair-facing, with code-level commit sequence) is the follow-on artefact produced after this design phase closes per the D-069 canonical-document rule.

### 1.1 Position in the milestone

The phase sits as a sibling to Phase 7.5 inside the Federation Event Propagation milestone — same pattern, same shape: dependent work (Phase 9) surfaced a protocol gap that closes in its own design + implementation phase before the dependent work resumes. Phase 9 Commit 3a's Scenario 1 stays on disk as `#[ignore]`-annotated regression witness; when the fix lands, `#[ignore]` lifts and the scenario becomes the activating regression lock.

Pass 1 implementation (`tasks/XGID_RETROFIT_PASS_1_IMPL.md`, Status ACTIVE v2.0) remains downstream of Federation Event Propagation milestone closure. **All three Joe-locks below are Pass-1-neutral**: wire format unchanged, no new fields, no schema changes. Pass 1's coverage table and sub-question locks are unaffected.

### 1.2 Reading order on session start

For Chat Claude + Joe re-entering this conversation:

1. This document, §2 (audit summary in one paragraph) — refresh the gap shape.
2. This document, §3 (Q1 lock + reasoning) — the event-model decision.
3. This document, §4 (Shape A lock + sub-option A.1 lock + verification result) — the fix shape.
4. This document, §5 (cross-references to D-075 + implementation runbook) — what ships next.

For the implementation runbook author (Chat Claude in a future session, or Clair on direct read):

1. This document end-to-end — the design is what to build.
2. The audit doc (`tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md`) — code-grounded mechanism evidence at §3 (sender/receiver/combined-outcome), Q1 framing at §6.1, Shape A + A.1 sub-options at §7.1.
3. DECISIONS.md D-075 — the principle the locks instantiate.

---

## 2. Audit summary (one paragraph)

Phase 9 Commit 3a's Scenario 1 ("two-Node push smoke") was the first test in the project's history to spawn two real `NodeRuntime` instances and let them perform a real federation bootstrap end-to-end through the production code path. The diagnostic run surfaced this cascade: A's `stream_federation_delta` ships Space S's full history to B including a freshly-built `state.federation_add` event (`sender = A`, `content.node_id = B`, semantics "A approves B as federation peer for Space S"). B's `apply_federation_add` reads `content.node_id` verbatim and pushes B onto its own `federation_nodes`. Result: A's view correct (`{B}`); B's view incorrect (`{B}` — should be `{A}`). F-3 then rejects every post-bootstrap A→B push event because the wire-authenticated peer A is not in B's `federation_nodes`. Events accumulate in HeldPending via Phase 7.5 P7.5-B's third trigger, hit the 180s federation-relationship timeout, get discarded with error code 4007. **Bootstrap appears to succeed but the relationship never actually establishes from B's view.** Full code-grounded evidence at `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` §3 with file:line references.

The audit framed two structural questions (Q1 + Q2) and presented four candidate fix shapes (A/B/C/D) with code-grounded cost/benefit. This design task file walks the questions and locks the choice.

---

## 3. Q1 lock — Reading (i): one event, asymmetric interpretation

### 3.1 The Joe-lock

**Q1 lock: Reading (i).** `state.federation_add` is one event recording one party's act, not a two-sided relationship object. The event has one signer (the asserter), one signature, one DAG slot, one logical assertion: "A declares that A federates with B for Space S." The asymmetry between `event.sender` (the asserter) and `event.content.node_id` (the other party) is the event's structure, not a redundancy or a bug.

`SpaceState.federation_nodes[S]` is a derived projection over `state.federation_add` events with a vantage-aware derivation rule: for each such event E, the local entry adds the **other party** — `event.sender` if I am `event.content.node_id`, else `event.content.node_id`. A's and B's `federation_nodes[S]` end up as mirrors as a consequence of correct application, not as a structural property of the event store.

### 3.2 Reasoning

Three reasons, in order of weight:

**Reason 1 — Precedent matters more than aesthetic symmetry.** Every relationship-shaped event in the current registry follows the one-party-assertion + derived-projection pattern: `membership.invite`, `membership.join`, `state.dm_promote`, `state.space_create`, `state.room_create`, `state.ai_operator_delegate`, `state.ai_operator_revoke`, `membership.kick`, `membership.ban`, `membership.mute`. The event records what one party did; the resulting data object (members list, room state, federation_nodes, pending_invites, ai_operator_delegations, banned set, active_mutes) is a derived projection. Reading (ii) (two events, one per side) would have introduced the first event family in the registry whose semantic completeness requires two signed assertions of the same type, one per party. That is a real precedent departure — not necessarily wrong, but a deliberate departure that must be justified by federation being genuinely a different shape of relationship from membership.

**Reason 2 — The "peer equality" argument for Reading (ii) is weaker than it looks.** Federation bootstrap *is* genuinely asymmetric: one side has Space history, the other is joining as a federation peer. Phase 3's a-i symmetry rule (runbook §3.3.1 Lock 2) already produces symmetric outcomes from asymmetric inputs (tip exchange → each side mints what the other needs). Peer equality emerges from the *outcome*, not the *event shape*. Reading (i) honours this; Reading (ii) would have pretended the bootstrap asymmetry doesn't exist by encoding peer-equality at the event-shape layer.

**Reason 3 — Operational simplicity.** Reading (ii) admits "half-federated" intermediate states (one side's event present, the other's missing), replay edge cases where one side's event is lost in event-store migration, and reciprocal-mint timing concerns. Reading (i) has none of these — the event store is the source of truth, the applier reads it correctly, done.

### 3.3 What Reading (ii) would have been (and was rejected)

Recorded for completeness so the rejection is explicit and the principle promoted to D-075 has clear contrast:

Under Reading (ii), a federation relationship for Space S between A and B would be constituted by **two signed assertions**, one per direction: E_A (signer=A, content names B) and E_B (signer=B, content names A). Each event would be fully symmetric to the other. The applier would become a straight read: "add `content.node_id` to my federation_nodes." `SpaceState.federation_nodes[S]` would be a straight projection of "all my own federation_add events for this Space."

This was rejected for the three reasons above. The rejection is itself the protocol's commitment to the one-party-assertion + derived-projection pattern as the default discipline for relationship-shaped events. D-075 promotes that commitment to a project-wide principle so future contributors do not silently drift toward Reading (ii) when adding new relationship-shaped events.

### 3.4 The principle promoted to D-075

The Joe-lock above instantiates a more general protocol-design principle, promoted to `DECISIONS.md` D-075 in the same commit as this design task file:

> Every relationship-shaped event in the protocol records **one party's act**, not a two-sided relationship object. The resulting data object is a **derived projection** with vantage-aware applier logic when the event has sender-vs-other-party asymmetry.

D-075 is sibling-distinct from D-070 (D-070 is about the transport-layer acceptance-vs-rejection signal pair on `TransportMessage`; D-075 is about the event-model layer — what `state.federation_add` IS as a DAG event). Future readers seeing "two events" in either decision should consult the layer named in the decision title to disambiguate.

### 3.5 Q2 — Signature carries enough information to derive the receiver's value

The audit doc §6.2 already code-verified the answer: yes. `build_federation_add_event` calls `sender_id(key)` where `key = node_keypair` (A's Node keypair); `event.sender` carries A's Node URI by construction; Phase 4 Q3 overload allows `event.sender` to be a Node URI on `state.federation_add` events specifically. So Reading (i) is implementable via vantage-aware apply logic on the receiver side, deriving the asserter from `event.sender` and the other-party from `content.node_id`, with the local Node's vantage (`my_node_id`) telling the applier which side it is on.

No additional walkthrough needed at Q2 — the audit's code trace settles it.

---

## 4. Shape lock — Shape A + sub-option A.1

### 4.1 The Joe-lock

**Shape lock: Shape A** (origin-aware applier). Wire format unchanged. Event construction unchanged. The applier `apply_federation_add` in `xgen-core/src/space/state.rs` gains a parameter — the local Node ID — and derives the relevant peer with vantage-awareness:

```rust
fn apply_federation_add(
    &mut self,
    event: &Event,
    my_node_id: &str,    // NEW: local Node's vantage
) -> Result<(), SpaceError> {
    if self.dm_constraints_active {
        return Err(SpaceError::DmFederationNotAllowed);
    }
    let content_node_id = event.content["node_id"]
        .as_str()
        .ok_or(SpaceError::MissingField("node_id"))?;
    // D-075 vantage-aware applier: pick the OTHER party from this Node's view.
    // If I am content.node_id (someone else's federation_add naming me),
    // the relevant peer is event.sender (the asserter).
    // Else (my own federation_add, OR someone else's naming a third party),
    // the relevant peer is content.node_id.
    let peer_to_add = if content_node_id == my_node_id {
        event.sender.as_str()
    } else {
        content_node_id
    };
    let peer_string = peer_to_add.to_string();
    if !self.federation_nodes.contains(&peer_string) {
        self.federation_nodes.push(peer_string);
    }
    Ok(())
}
```

Both branches are needed because A's own DAG also contains the `state.federation_add` event A authored — A applies it through its own applier and falls into the `else` branch (content.node_id = B, not A's own ID), adding B. B receives the same event via federation, applies it, and falls into the `if` branch (content.node_id = B = its own ID), adding A's sender. Symmetric outcomes through asymmetric branches, driven by vantage. This is D-075 in code.

**Sub-option lock: A.1** (re-derive on load). `SpaceState` is fully derived from the event store at every load — there is no persisted `federation_nodes` field to migrate, no fixup pass to run, no version marker to add. The fix lands and the next time any Node starts (or any Space's state is reconstructed during test setup), every `SpaceState.federation_nodes` automatically rebuilds correctly because the events themselves are correct and the applier now reads them correctly.

### 4.2 Verification — A.1 native fit confirmed

The audit doc § noted that A.1 needed verification against the actual `SpaceState` load model. The verification was performed at design close and resolved as follows:

**Result: A.1 is a perfect architectural fit, not a deviation.**

Code trace through `xgen-core/src/node/runtime.rs`:

- `NodeRuntime::spaces: HashMap<String, SpaceState>` is the only home for `SpaceState`. There is no persisted form.
- `NodeRuntime::new(keypair: SigningKey)` (line 125-148) initialises `spaces: HashMap::new()` (line 134). Every NodeRuntime starts with no SpaceStates.
- The companion fields at the same struct level carry the sibling comment "Not persisted — rebuilt from local state on restart (Phase 2 simplification)" at line 108 (`replica_registry`) and "Not persisted; discarded on Node restart or when proposal resolves" at line 105 (`dm_proposals`). The "not persisted, rebuild on restart" model is the deliberate Phase 2 design across all NodeRuntime-resident derived state.
- `SpaceState` is constructed via `SpaceState::from_space_create` (line 162 of state.rs) or `SpaceState::from_dm_space_create` (line 211 of state.rs) when the originating `state.space_create` / `state.dm_space_create` event lands; subsequent state events are applied via `SpaceState::apply_event` (line 354). The event store is the source of truth; `SpaceState` is the runtime cache.

**Implication.** No migration code, no version marker, no fixup pass, no schema change anywhere. The implementation runbook does not carry an "existing event stores need rebuilding" deliverable because there is no such surface — the rebuild happens organically on every Node start.

This means the architectural-cleanliness argument for A.1 is not just a future-proofing argument; it matches the model that's already shipped. A.1 is the existing model applied to a bug whose fix happens to need it.

### 4.3 What Shapes B, C, D would have been (and were rejected)

Recorded for completeness so the rejections are explicit:

**Shape B — Reciprocal mint on ingest.** When B's `apply_federation_add` runs on A's event, B also mints a second `state.federation_add` event signed by B's Node keypair, naming A. B ingests its own minted event locally. Rejected because (1) it introduces a "ingesting an event causes a new event to be authored" pattern that doesn't exist elsewhere in the protocol, (2) the reciprocal propagates back to A on next pull and either requires special-case "ignore federation_add naming self" handling (which re-introduces the asymmetry-by-special-case the audit was trying to remove) or produces the same bug mirrored on A's side, (3) idempotency concerns on re-handshake risk duplicate reciprocals accumulating in the DAG.

**Shape C — Two events at handshake.** Each side independently builds its own `state.federation_add` at handshake time, naming the other party. Rejected because (1) re-opens Phase 3 §3.3.1 Lock 2 a-i symmetry rule for the four-case re-walk Reading (ii) implies, (2) Reading (ii) was rejected at Q1 — Shape C is structurally a Reading (ii) shape regardless of whether the prev_events sub-option (C.1 reciprocal-references-A's / C.2 anchors-to-space-create) is taken, (3) doubles federation_add count per relationship across all archives.

**Shape D — Wire format extension.** Add a content field (D.1 `peer_node_id` duplicating sender, D.2 symmetric `{a_node, b_node}`) so the event self-describes both parties. Rejected at design close because:

- D.1 duplicates `sender` into content — worst-of-both-worlds (pay the wire cost AND keep the vantage logic in the applier). Information duplication invites future bugs ("what if sender ≠ peer_node_id? is it malformed or is one authoritative?"). Rejected on its own merits.
- D.2 (symmetric content) is the philosophically cleanest schema, but it is **secretly Reading (ii) smuggled into Reading (i)**. A symmetric event ("the relationship is the object, not the act") structurally represents Reading (ii), regardless of whether one signer signs it. The schema change re-introduces the philosophical commitment Q1 rejected; pursuing D.2 would have inconsistency with the Q1 lock and would require re-opening that decision.
- D.2's additional cost: wire-format change touching Appendix C + Appendix I + Pass 1 coverage table. Pass-1-impacting (only shape with Pass-1 impact among A/B/C/D). Without compensating semantic clarity over Shape A (which it doesn't provide given the Reading (ii) leak), the cost is not justified.

The chain Q1 Reading (i) → Shape A → A.1 is internally consistent at every step: one event, vantage-aware applier, derive-on-load. The chain has no precedent departures, no wire impact, no Pass-1 impact, and matches the existing `SpaceState` non-persistence model.

### 4.4 Replay sub-option survey (A.1 vs A.2 vs A.3)

The audit doc §7.1 named three sub-options for replay under Shape A. The verification result above (§4.2) makes A.1 the natural fit. For completeness:

- **A.1 (locked).** Re-derive on load. No replay/origin-tracking machinery needed because `SpaceState` is non-persisted; every Node start fresh-builds from events. Zero replay surface to worry about.
- **A.2** (introduce `EventOrigin::ReplayedFromDisk` variant with apply-time fallback rule) — rejected because there is no replay surface in the first place. A.2 was a candidate under the assumption `SpaceState` was persisted, which the verification refuted.
- **A.3** (reconstruct origin from `event.sender == self_node_id` heuristic at replay time) — rejected for the same reason. The verification showed there is no replay-from-disk path for `SpaceState`; the heuristic has nothing to discriminate against. A.3 would also have created the drift surface flagged in the audit doc §7.1 (origin discriminator at ingest = `EventOrigin` parameter, origin discriminator at replay = sender-equality check, two mechanisms answering the same question).

The `EventOrigin` enum stays unchanged in Shape A + A.1. The applier becomes vantage-aware via the new `my_node_id` parameter — that is independent of `EventOrigin`'s `LocallySubmitted` / `ReceivedViaFederation` distinction. `apply_federation_add` does NOT need to be `EventOrigin`-aware; it needs to be `my_node_id`-aware. This is a subtler distinction than the audit doc's framing but resolves cleanly: `my_node_id` is local Node context (always known, always the same value across LocallySubmitted and ReceivedViaFederation), not event-origin context.

---

## 5. Scope, ordering, and downstream coordination

### 5.1 In scope for the implementation that follows

- **Code:** `apply_federation_add` gains `my_node_id: &str` parameter and vantage-aware branching; `SpaceState::apply_event` dispatch threads `my_node_id` through; production callers at `NodeRuntime::ingest_event` (xgen-core/src/node/runtime.rs:189 + :197) pass `&self.node_id`.
- **Tests:** six suggested unit tests in `xgen-core/src/space/state.rs::tests` covering both vantage branches (sender-vantage and content.node_id-vantage), mirror property (A and B end with mirrored `federation_nodes`), DM constraint preserved, missing-field rejection preserved, and idempotency on duplicate ingest.
- **Phase 9 Scenario 1 resurrection:** `#[ignore]` removed from `xgen-node/src/tests/phase9_two_node_smoke.rs::scenario_1_two_node_push_smoke`; scenario passes against the fixed applier; serves as the activating integration-level regression lock.
- **Documentation:** Canonical design doc gains §6.4.2 sibling subsection (sibling to Phase 7.5's §6.4.1) summarising the locks + pointing at this design task file + at D-075. §15 Implementation Complete table gains a row.
- **Same-commit discipline:** Per D-074, the milestone-close commit includes JOURNAL.md alongside CLAUDE.md, ROADMAP.md, this design task file Status flip (ACTIVE → COMPLETED), and the audit doc Status flip (ACTIVE → COMPLETED).

### 5.2 Out of scope for this design phase and the implementation that follows

- **Wire-format changes.** Shape D was the only candidate that touched the wire; rejected at design close.
- **`EventOrigin` enum changes.** Shape A + A.1 does not require new variants or signature changes on `EventOrigin`.
- **Schema changes to `state.federation_add` content.** No new fields. Existing content schema is correct as-authored.
- **Migration of existing events.** No surface — `SpaceState` is non-persisted; events themselves are correct.
- **Generalising the vantage-aware-applier pattern to other event families.** D-075 names the pattern as a default for future relationship-shaped events but does NOT trigger a sweep of existing events. The only event currently needing a vantage-aware applier is `state.federation_add` (the audit walked the registry — every other event is symmetric across ingestors).

### 5.3 Hard ordering (no parallelism within the implementation)

The four implementation commits are sequential, not parallel:

1. **Commit 1 — Doc-pass.** Canonical design doc §6.4.2 + §15 row; design task file Status flipped COMPLETED; audit doc Status flipped COMPLETED. No code. The design state of record settles before any code lands.
2. **Commit 2 — Origin-aware applier + plumbing + unit tests.** The fix itself. Tests at unit level lock in correctness for both vantage branches.
3. **Commit 3 — Phase 9 Scenario 1 resurrection.** `#[ignore]` lifts; scenario passes; integration-level regression lock activates.
4. **Commit 4 — Milestone close.** CLAUDE.md + ROADMAP.md + JOURNAL.md + status flips per D-074.

Commit 2 → 3 ordering is load-bearing: the integration test passes only after the applier fix, so attempting to lift `#[ignore]` before the fix lands would produce a failing test in the regression-witness commit. The doc-pass commit before code is the D-069 + D-074 same-commit-discipline applied in advance — the canonical record reflects the locked design before any code references it.

### 5.4 Downstream coordination

**Phase 9 Commit 3b unblocks** when Commit 3 of this implementation ships. The remaining scenarios (2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10) per `tasks/FEDERATION_PROPAGATION_PHASE_9.md` proceed as planned. The four deferred compounds (C1, C4, C6, C8) in `tasks/FEDERATION_STRESS_FOLLOWON.md` remain blocked on the clock-injection seam independently — unaffected by this fix.

**Federation Event Propagation milestone closes** when Phase 9 closes. The closure commit's PLAY block flip (currently 🟢 PLAY → ✅ DONE) is when the M6 (new) implementation track + the XGID Retrofit Pass 1 implementation track simultaneously unblock per the existing dependency chain.

**Pass 1 implementation is Pass-1-neutral on this fix.** The applier signature change is a Rust function-signature change, not an XGID retype. The `my_node_id: &str` parameter at Shape A is a String reference; if Pass 3 widens it to `&NodeXgid` at the dispatch site, the change collapses naturally (the applier uses it as a comparator against `event.content["node_id"]` and `event.sender`, both of which are String at v1 and become XGID flavours through the Pass-series progression). No coordination flag with Pass 1's runbook is needed.

**D-075 is the principle the implementation instantiates.** The verbatim code-comment block at the applier site in Commit 2 cites D-075 + this design task file §3.1 by name; future readers can trace from applier to principle.

### 5.5 Cross-references

- **DECISIONS.md D-075** — the protocol-design principle this design phase locked. Authoritative home for the "event records one party's act; data object is derived projection with vantage-aware applier" rule.
- **DECISIONS.md D-070** — sibling-distinct "two events" principle at the transport-layer signal pair. Distinct from D-075 (transport-layer outcome signals vs. event-model semantics). Future readers seeing "two events" in either decision should consult the layer named in the decision title to disambiguate.
- **DECISIONS.md D-069** — canonical-document rule. The audit doc, this design task file, the implementation runbook, the canonical design doc's §6.4.2 summary, and D-075 itself form the four-document chain per D-069.
- **DECISIONS.md D-071** — audit-precedes-dependent-design discipline. This design phase is the worked instance of that discipline at the design-phase boundary.
- **DECISIONS.md D-074** — milestone-close commits include JOURNAL.md. Applies to Commit 4 of the implementation runbook.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md`** (Status ACTIVE v1.0, flips COMPLETED at implementation Commit 1) — code-grounded audit doc. §3 mechanism, §6.1 Q1 framing, §6.2 Q2 verification, §7.1–§7.4 four candidate fix shapes, §7.5 summary table.
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`** (Status ACTIVE v1.0) — implementation runbook. Four-commit sequence for Clair.
- **`docs/xgen_federation_propagation_design.md`** §6.4 — Phase 7 F-3 framework (Lock A1, Lock B1) that established `SpaceState.federation_nodes` as the F-3 data source. Implementation Commit 1 adds §6.4.2 as sibling subsection summarising this design phase's locks.
- **`tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md`** (Status COMPLETED v1.0) — sibling design task file. Same lifecycle: ACTIVE during design walkthrough, flips COMPLETED at implementation runbook's Commit 1.
- **`xgen-core/src/node/runtime.rs::NodeRuntime`** — the runtime struct whose `spaces: HashMap<String, SpaceState>` field carries the (non-persisted) state. Verification site for the A.1 sub-option lock.
- **`xgen-core/src/space/state.rs::apply_federation_add`** (lines 351-363 pre-fix) — the function whose signature gains `my_node_id` and whose body becomes vantage-aware in implementation Commit 2.

---

## 6. Discipline notes

This design phase is a worked instance of three project-management principles already in DECISIONS.md, plus one mid-design verification step worth recording explicitly:

- **D-069 (canonical-document rule).** Authoritative homes settled at design close: D-075 in DECISIONS.md (principle); this design task file in `tasks/` (locks + reasoning); audit doc in `tasks/` (code-grounded mechanism); implementation runbook in `tasks/` (commit sequence). Four documents, one authority each, explicit forward-references between them.
- **D-071 (subsystem audits precede dependent milestones).** The audit doc shipped before this design phase opened; the design phase's first read on every session is the audit's §3 + §6 + §7. This design phase produces no new code-grounded evidence — that's the audit's job; this phase's job is to walk the option space the audit framed and lock the choice.
- **D-074 (milestone-close commits include JOURNAL).** Forward-binding for implementation Commit 4. Recorded here so the future Chat Claude or Clair authoring that commit doesn't omit JOURNAL.md from the changed-files list — the cross-doc state-flip discipline is load-bearing per D-074.
- **Mid-design verification step.** The A.1 sub-option lock was contingent on confirming `SpaceState`'s non-persistence model. The verification (§4.2 above) was performed inside this design phase, not deferred to implementation. This is the same pattern Phase 7.5 used at its §6.3 idempotent-hook clarification (verify mechanism before locking shape). Recording here as worked instance: when a sub-option lock depends on a code-state assumption, verify before locking; don't defer the assumption to implementation discovery.

This is also a worked instance of the "honest longer work over fast shortcuts" principle (ROADMAP.md cross-cutting principles). Phase 9 Commit 3a could have shipped without standing down the milestone — strip Scenario 1 from disk, ship Commits 1 + 2 + harness, attempt Scenarios 2 + 3 as workarounds. Or strip Scenario 1, note the finding in JOURNAL only, defer the fix indefinitely. Both would have been faster than the audit + design + implementation chain. Neither would have been honest about the protocol's actual state. The path taken — ship the regression witness, stand down the milestone, document the audit, walk the design phase, ship the fix — is longer than the workarounds and shorter than rediscovering the bug from a production deployment, which is the cost-of-comparison the principle is built against.

The shape recurses Phase 7.5's pattern (which itself recursed the J-081 → Federation design → Federation implementation pattern). The bidirectional `federation_nodes` design phase is the third instance in the project's history of "dependent work surfaces a load-bearing gap → audit → design → implementation → dependent work resumes." Each instance adds confidence that the pattern is durable across milestone shapes.

---

## 7. Out-of-scope decisions explicitly recorded

For each item that came up during the walkthrough and was deferred:

| Item | Why deferred | Where it lands |
|---|---|---|
| Audit of other state events for directional-semantic congruence | No current evidence of similar bugs; D-075 names the discipline as a default for future events but does not trigger a retroactive sweep | Future event-design discipline; opportunistic, not scheduled |
| Renaming `state.federation_add` to reflect directionality (e.g. `state.federation_approve`) | Wire-protocol-visible rename touches every persisted event in production archives | Out of scope; not justified by the bug |
| Symmetric mutual-federation event (single event approving both directions, no asymmetric interpretation needed) | Different protocol semantic from today's pairwise approval; Reading (ii) rejected | Future protocol discussion if mutual federation is ever a first-class concept |
| Persisting `SpaceState` (which would re-open the A.1 vs A.3 sub-option question) | Phase 2 simplification model deliberately makes `SpaceState` non-persisted; persistence would be its own milestone | Future milestone if fast-start becomes a constraint |
| Generalising `my_node_id` plumbing to all `apply_*` functions | Only `apply_federation_add` currently needs vantage-awareness; threading the parameter everywhere on speculation would be premature abstraction | Per-applier audit if/when other vantage-aware appliers surface |
| Widening `my_node_id: &str` to `&NodeXgid` at Shape A's call sites | Pass 3 territory — when xgen-node-side dispatch widens to XGID flavours, the parameter type widens with it; the v1 fix uses `&str` for surface-level neutrality | XGID Retrofit Pass 3 |

---

## 8. Open items at design close

None.

All three Joe-lock thresholds (Q1, Shape, sub-option) were resolved in the walkthrough; the verification check for sub-option A.1 was performed inside the design phase against `xgen-core/src/node/runtime.rs` and resolved cleanly. D-075 promotion happened same-session; the implementation runbook authoring happened same-session. No open questions are deferred to implementation.

The implementation runbook (`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`, Status ACTIVE v1.0) is the next-active artefact for Clair.

---

*End of design task file. Status flips ACTIVE → COMPLETED in Commit 1 of the implementation runbook per the established design-task-file lifecycle (sibling to `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` v1.0 → COMPLETED at Phase 7.5 implementation runbook Commit 1). Locked content above is preserved as authoritative record of the three framework decisions Q1 Reading (i) + Shape A + sub-option A.1.*  
