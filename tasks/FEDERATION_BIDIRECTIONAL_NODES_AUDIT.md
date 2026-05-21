# Federation Bidirectional `federation_nodes` Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-21 (audit opened in response to the bidirectional `federation_nodes` finding surfaced during Phase 9 Commit 3a Scenario 1 diagnostic run; sibling to J-081 Propagation Reliability Audit precedent (pre-design subsystem audit) and to Phase 7.5 design-phase precedent (dependent work surfaces protocol gap, closes in its own phase before dependent work resumes). Per D-071 audit-precedes-dependent-design discipline. This document is the canonical record of the audit finding; the subsequent design phase produces its own task file with locked framework decisions and is the implementation precondition.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This document is the canonical record of a protocol-level audit finding surfaced during Phase 9 Commit 3a's Scenario 1 diagnostic run: **after a successful brand-new federation bootstrap from Node A to Node B for Space S, B's `SpaceState.federation_nodes[S]` ends up containing B's own Node ID instead of A's, causing the F-3 federation-relationship gate on B to reject every post-bootstrap A→B push event.**

It is a subsystem audit, per D-071 — produced before any design-phase work begins, so the design phase walkthrough that follows has a code-grounded shared baseline to walk against. It is sibling-in-shape to J-081 (Propagation Reliability Audit, which preceded the Federation Event Propagation design phase) and Phase 7.5's design task file (which preceded the Phase 7.5 implementation runbook). The pattern is: dependent work surfaces a load-bearing protocol gap → audit documents the gap with code-grounded evidence and candidate fix shapes → design phase walks the candidates and locks a fix → implementation runbook ships it → dependent work resumes.

This document does not lock a fix. Locking is the design phase's job. This document's job is to surface the gap precisely, ground it in the actual code, and frame the option space cleanly enough that the design walkthrough can run efficiently.

### 1.1 Position in the milestone

This audit sits between Phase 9 Commit 3a (regression witness shipped 2026-05-21) and the bidirectional `federation_nodes` design phase (next-active). Phase 9 Commit 3b (Scenarios 2 + 3, plus the compound scenarios) is paused inside the Federation Event Propagation milestone scope until the protocol fix lands. When the fix ships, Scenario 1's `#[ignore]` annotation lifts and the scenario becomes the activating regression lock; Commit 3b then proceeds.

Pass 1 implementation (`tasks/XGID_RETROFIT_PASS_1_IMPL.md`, Status: ACTIVE v2.0) remains downstream of Federation Event Propagation milestone closure. Whether the bidirectional fix affects Pass 1's runbook scope depends on which fix shape is locked — see §7.5 below.

---

## 2. Provenance — how the gap surfaced

Phase 9 Commit 3a's Scenario 1 ("two-Node push smoke") was the first in-process integration test designed to exercise the full bidirectional federation push path with real `NodeRuntime` instances on both sides. The pre-existing federation-side test surface had two structural gaps that allowed this protocol-level bug to remain hidden through all eight Federation Event Propagation milestone phases plus Phase 7.5:

1. **`cold_start_bootstrap_integration` tests** build the bootstrap stream manually. The fixtures construct `state.federation_add` events directly with `fed_add(content.node_id=peer_a_id)` on B's side — that is, the tests inject what they EXPECT B to end up with, rather than what production actually delivers. The tests then verify the gate behaviour given the assumed state, not given the production state-arrival path.

2. **`federation_push_integration` tests** use a one-sided harness. Node B is a wire reader with no real `NodeRuntime` — it consumes the wire frames A sends and asserts on their content, but it never runs `process_inbound` or `apply_federation_add` on B's side. Anything that depends on B's local state being populated correctly from ingested events is outside the harness's scope.

Phase 9 Scenario 1 was the first test in the project's history to spawn two real `NodeRuntime` instances and let them perform a real bootstrap end-to-end through the production code path. Within minutes of the harness coming up green, the assertion failed in a way that pointed past Scenario 1's specific check at a structural protocol problem.

### 2.1 The observed cascade

The pattern recorded by Clair's diagnostic run:

1. Node B (brand-new for Space S) initiates federation handshake to Node A.
2. F-1a tip exchange completes. A's `stream_federation_delta` ships Space S's full history to B, plus a freshly-built `state.federation_add` event under the a-i symmetry rule (runbook §3.3.1 Lock 2).
3. B's `process_inbound` accepts the bootstrap stream. The Phase 7.5 P7.5-A skip rule lets `state.space_create` through F-3 and F-4 step 1, so the Space gets created locally on B. P7.5-B's third HeldPending trigger correctly holds subsequent events until federation-relationship arrives.
4. `state.federation_add` arrives. Phase 7's B1 + B3 skip rules let it through F-3 and the F-4 step 9/11/13 gates. Phase 7.5's idempotent arrival hook fires, draining the HeldPending federation-relationship trigger.
5. **At this point the held events re-validate through the unified validation core — and F-3 rejects them all because B's `federation_nodes` for Space S does not actually contain A.**

The events accumulate in HeldPending again via P7.5-B's trigger (third HeldPending trigger), reach the 180s federation-relationship timeout, and get discarded with error code 4007. Every subsequent A→B push event walks the same path.

**Bootstrap appears to succeed but the relationship never actually establishes from B's view.** A's view is correct; B's view is broken.

---

## 3. The mechanism, code-verified

The gap exists in the interaction between three pieces of code that, individually, behave per the locked design. The problem only surfaces when all three execute in sequence on the receiver side.

### 3.1 Sender side — event construction

`xgen-node/src/federation_session.rs::stream_federation_delta` lines 112-127:

```rust
// §3.3.1 Lock 2 — a-i symmetry rule: this side builds state.federation_add
// for `space_id` exactly when the peer's tips map shows that Space absent
// AND we have events for it. Deterministic from wire-visible tips maps;
// both sides compute the same answer from the same data.
if peer_absent && we_have_events {
    let fed_add_ev = sign_event(
        build_federation_add_event(
            node_keypair,           // A's Node keypair → sender = A
            space_id,
            our_local_tips,
            peer_node_id,           // B's Node ID → content.node_id = B
            session_id,
            negotiated_version,
            negotiated_serialisation,
        ),
        node_keypair,
    );
    // ... ingest locally, persist, append to delta
}
```

The builder at `xgen-core/src/space/state.rs::build_federation_add_event` (lines 910-933):

```rust
pub fn build_federation_add_event(
    key: &SigningKey,
    space_id: &str,
    prev_events: Vec<String>,
    peer_node_id: &str,
    session_id: &str,
    negotiated_version: &str,
    negotiated_serialisation: &str,
) -> Event {
    Event::new(
        EventType::StateFederationAdd,
        sender_id(key),             // signing-key-derived → A
        String::new(),
        space_id.to_string(),
        prev_events,
        now(),
        json!({
            "node_id": peer_node_id, // verbatim B's ID
            "session_id": session_id,
            // ...
        }),
    )
}
```

**The constructed event is unambiguous and asymmetric:** `event.sender = A`, `event.content.node_id = B`. The semantic is unidirectional: "A is asserting that A approves B as a federation peer for Space S." There is one logical assertion encoded in one wire event.

### 3.2 Receiver side — event application

`xgen-core/src/space/state.rs::apply_federation_add` lines 351-363:

```rust
fn apply_federation_add(&mut self, event: &Event) -> Result<(), SpaceError> {
    if self.dm_constraints_active {
        return Err(SpaceError::DmFederationNotAllowed);
    }
    let node_id = event.content["node_id"]   // always extracts B from this event
        .as_str()
        .ok_or(SpaceError::MissingField("node_id"))?
        .to_string();
    if !self.federation_nodes.contains(&node_id) {
        self.federation_nodes.push(node_id);  // pushes B regardless of which Node reads
    }
    Ok(())
}
```

**The application logic is unconditional and symmetric:** whoever applies this event pushes `content.node_id` onto their `federation_nodes`. The function does not consult `event.sender`, does not consult any per-Node context, does not have any notion of "is this event about me or about the other party."

The event ingestion path is shared between local-author and federation-receiver via Phase 4's `EventOrigin` parameter, but `apply_federation_add` itself does not see `EventOrigin` — the state-apply layer is downstream of the origin-aware dispatcher and operates as a pure function of the event.

### 3.3 The combined outcome

When A and B both ingest the single `state.federation_add` event A produced:

| Node | Applies | Resulting `federation_nodes` | Correctness |
|---|---|---|---|
| A | `content.node_id = B` → push B | `{..., B}` | ✅ A learns "I am federated with B" |
| B | `content.node_id = B` → push B | `{..., B}` | ❌ B should learn "I am federated with A" |

### 3.4 What F-3 then does on B

`xgen-core/src/node/runtime.rs::dispatch_event` runs the Phase 7 Lock A1 + B1 check on every federation-channel event:

```rust
// Phase 7 Lock A1
if !state.federation_add && peer_node_id.is_some() {
    let peer = peer_node_id.unwrap();
    let in_federation = self.spaces.get(&space_id)
        .map(|s| s.federation_nodes.iter().any(|n| n == peer))
        .unwrap_or(false);
    if !in_federation {
        return DispatchOutcome::Rejected(
            format!("federation_relationship_missing: peer {peer} ...")
        );
    }
}
```

Wire-authenticated `peer = A`. `B.federation_nodes = {B}`. The `any(|n| n == "A")` returns false. F-3 rejects. Every post-bootstrap A→B push event walks this path. Phase 7.5's P7.5-B third HeldPending trigger fires, the events hold for 180s, then time out with error code 4007.

The F-3 check itself is correct per the locked design. The check is reading the right data source (Phase 7 Lock A1: `SpaceState.federation_nodes`). The bug is that the data source has the wrong value because `apply_federation_add` produced the wrong value when B ingested A's event.

---

## 4. The relationship to the design as locked

The federation propagation design phase considered chicken-and-egg cases between gates and events that establish the gate's data. Phase 7 Lock A1 explicitly anticipated a related class of bug (`xgen_federation_propagation_design.md` §6.4):

> "The two reads must agree, or an update race between handshake-time refresh and event-time `state.federation_add` ingestion would produce a system that pushes events but rejects them on receipt."

**The bug found is a different shape of the same general class.** The design's anticipated failure mode was: *two read paths* (handshake-time `FederationRegistry.shared_spaces` vs. event-time `SpaceState.federation_nodes`) might disagree on the same underlying question, producing inconsistent push-vs-reject behaviour. The design closed that gap by mandating both reads consult the same source (`SpaceState.federation_nodes`), eliminating the two-source-drift surface.

This bug is not a race between two reads. It is a semantic asymmetry between sender's view and receiver's view of the same ingested event. Both sides read `SpaceState.federation_nodes` via the same code path. Both sides write `SpaceState.federation_nodes` via the same code path. **The shared code produces different correctness on the two sides because the event's semantic is asymmetric ("A approves B") while the application logic is symmetric (push `content.node_id`).**

### 4.1 Why the locked design did not catch this

Three structural reasons:

1. **The "two reads" framing implicitly assumed the event's symmetric application would populate both sides correctly.** §6.4 worried about whether two readers see the same value; it did not interrogate whether the writers were producing the same value.

2. **Phase 3's a-i symmetry rule was framed as a SENDER-side rule** ("the side that has events builds it"). The locked design (runbook §3.3.1 Lock 2) ensures the SENDER-side decision is deterministic from wire-visible state. It does not address what the RECEIVER does with the resulting event.

3. **The test surface had two structural gaps** (per §2 above) that prevented bidirectional bootstrap from being exercised end-to-end before Phase 9. Phase 7.5's six NodeRuntime-level integration tests covered HeldPending semantics with manual fixtures; the Phase 4 + Phase 5 integration tests covered the wire path on one side with the other side as a wire reader. Until Phase 9 Scenario 1, no test combined "real `NodeRuntime` on both sides" with "real federation handshake from cold start" with "post-bootstrap event push." That combination is what surfaces this bug.

### 4.2 The principle the bug reveals

The bug reveals an unstated principle in the protocol's design that the design phase did not surface as a Joe-lock-threshold question:

> **The event semantic and the event application semantic must be congruent across all ingestors of the event.**

For `state.federation_add` they are not. The event's semantic ("A approves B") has a sender-implicit party (A, encoded in `event.sender`) and a receiver-explicit party (B, encoded in `content.node_id`). The application logic ignores the sender and pushes the content party — which is correct on A's side (A wants B added) and incorrect on B's side (B wants A added, not itself).

Every other state-event family the audit walked has congruent semantics: `state.space_create`, `state.room_create`, `membership.invite`, `membership.join`, `state.dm_promote`, all have an application semantic that is symmetric across ingestors (everyone who applies the event ends up with the same Space state because the event itself names the symmetric piece of new state, not a directional relationship). `state.federation_add` is the only event in the current Ch3 event registry whose semantic is genuinely directional.

This is not necessarily a design flaw in `state.federation_add`'s schema — federation IS a directional relationship at the protocol layer (A approves B does not automatically mean B approves A; mutual federation is two approvals, not one). The flaw is that the application layer does not honour the directionality.

---

## 5. Scope

### 5.1 In scope for the design phase that follows

- **Determine whether `state.federation_add`'s semantic is one logical assertion or two.** (See §6 Q1.)
- **Determine the correct shape of the fix** — wire-format change, application-logic change, sender-side rule change, or hybrid. The four candidate fix shapes are presented in §7.
- **Confirm or revise the Pass 1 runbook scope** based on the locked fix shape's structural footprint. (See §7.5.)
- **Coordinate the fix with all existing Federation Event Propagation milestone locks** — Phase 3 a-i symmetry rule, Phase 4 origin gating, Phase 7 A1 + B1 + B3, Phase 7.5 P7.5-A + P7.5-B + P7.5-C + P7.5-D. The fix must not regress any of these.
- **Preserve the HeldPending P7.5-B trigger semantics** — the third trigger condition fires correctly today; the bug is that resolving it doesn't help because the data the trigger waits for is wrong on arrival.

### 5.2 Out of scope for this audit and the design phase

- **Generalising the principle in §4.2 across other event families.** No other event in the current registry has directional semantics; the principle is recorded for future event-design discipline (D-073 territory or sibling) but does not produce work in this milestone.
- **Wire-protocol changes beyond `state.federation_add`'s content schema.** If the locked fix is wire-format-bearing, the change is bounded to one event type.
- **Test-surface restructuring beyond what the fix's regression coverage requires.** The structural test-surface gaps in §4.1.3 are real but their closure is Phase 9 Commit 3a's regression-witness scope; this milestone does not extend Phase 9's test plan.
- **Stress-test follow-on coverage.** The four deferred compound scenarios in `tasks/FEDERATION_STRESS_FOLLOWON.md` remain blocked on the clock-injection seam and are unaffected by this audit.

### 5.3 Non-scope decisions explicitly recorded

| Item | Why deferred | Where it lands |
|---|---|---|
| Audit of other state events for directional-semantic congruence | No current evidence of similar bugs in other event families; opportunistic audit would expand scope without clear benefit | Future event-design discipline note |
| Renaming `state.federation_add` to reflect directionality (e.g. `state.federation_approve`) | Wire-protocol-visible rename touches every persisted event in production archives | Out-of-scope; not justified by the bug |
| Symmetric mutual-federation event (single event approving both directions) | Different protocol semantic from today's pairwise approval; out of scope | Future protocol discussion if mutual federation is ever a first-class concept |

---

## 6. Audit-phase questions for the design walkthrough

Two structural questions surface before any fix shape can be properly evaluated. The design phase walkthrough must answer both.

### 6.1 Q1 — Is `state.federation_add` ONE event or TWO logical events?

The bug exists because the event has a single `node_id` field that means different things to different readers. Two readings of what the event fundamentally IS:

**Reading (i) — One event, asymmetric interpretation.** The event is one logical assertion ("A approves B as a federation peer for Space S"). The current schema records only the "other" party (the party-not-self-from-sender's-perspective). The fix is to teach the receiver to derive the OTHER party from somewhere — most naturally, from `event.sender`, which IS A from B's perspective too (the signature is preserved across the wire and verified by both sides).

**Reading (ii) — Two events, one per side.** A federation relationship is genuinely bidirectional; each side should produce its own `state.federation_add` naming the other. A produces one ("A federates with B"); B produces one ("B federates with A"). Both events live in their respective Nodes' DAGs as audit records of relationship establishment.

The design implicitly assumed (i) — the a-i symmetry rule (runbook §3.3.1 Lock 2) says "the side that has events builds it," singular. But the schema and applier were built as if (ii) were true (both sides naively apply `content.node_id`).

This reading must be resolved before any fix shape can be evaluated, because it determines which fix shapes are even on the table. Reading (i) admits Shape A and Shape D (see §7). Reading (ii) admits Shape B and Shape C.

### 6.2 Q2 — Does the existing event signature carry enough information to derive the receiver's correct value?

If Reading (i) is selected, the fix depends on B being able to derive A's Node ID from the event B receives. The event's `sender` field carries A's Node URI by construction:

- `build_federation_add_event` calls `sender_id(key)` where `key = node_keypair` (A's Node keypair).
- Phase 4 Q3 overload allows `event.sender` to be either an Identity URI or a Node URI.
- For `state.federation_add`, the sender is always a Node URI by construction (Phase 4 §3.4.1 Q3-overload code-comment block).

**So the answer is yes: A's Node ID is available to B via `event.sender`.** This is verified, not assumed.

This means Reading (i) is implementable via origin-aware apply logic on the receiver side: if "this is my own event" → use `content.node_id`; if "this is a federation-received event" → use `event.sender`. The discriminator is `EventOrigin`, which Phase 4 already threads to the dispatcher.

Reading (ii) does not depend on this question because each side authors its own event with its own `content.node_id`.

### 6.3 Other questions the design phase will need to surface

These are listed for completeness but are subordinate to Q1's resolution:

- **Q3 — Replay semantics.** If the fix depends on `EventOrigin` at apply time, what happens on Node restart when federation_add events are replayed from disk? The current `EventOrigin` enum (`LocallySubmitted`, `ReceivedViaFederation`) does not have a `ReplayedFromDisk` variant. Either origin must be persisted (wire-bearing or sidecar), or a third variant must exist, or the apply logic must reconstruct origin from event context. (Applies to Shape A only.)
- **Q4 — Idempotency on duplicate ingest.** If the fix involves minting a reciprocal event on receipt (Shape B), what prevents the reciprocal from being re-minted on every replay? (Applies to Shape B only.)
- **Q5 — Idempotency on the existing a-i symmetry rule.** If the fix changes the sender-side rule to mint two events (Shape C), the existing a-i symmetry rule's three-case reasoning needs to be re-walked to confirm the new rule is still deterministic and doesn't produce duplicate events on reconnect. (Applies to Shape C only.)
- **Q6 — Migration of existing federation_add events in archives.** Production deployments don't exist yet, but the test fixtures and any in-progress development-build state files contain federation_add events under the current schema. A wire-format change (Shape D) needs a migration story or a clean-break point. (Applies to Shape D only.)

---

## 7. Candidate fix shapes

Four candidate fix shapes were captured in JOURNAL J-092 Commit 3a sub-entry. This audit refines each with code-grounded cost/benefit analysis and surfaces the Pass-1 impact for each.

The shapes are presented in order of structural depth (smallest change first). The design phase walkthrough picks one with Joe-lock; this audit does NOT pre-decide.

### 7.1 Shape A — Origin-aware applier (JOURNAL shape c, refined)

**Mechanism.** `apply_federation_add` becomes origin-aware. When called with `EventOrigin::LocallySubmitted`, push `content.node_id` as today. When called with `EventOrigin::ReceivedViaFederation`, push `event.sender` instead. Zero wire-format change. The event semantic remains "one logical assertion" per Q1 Reading (i).

**Code shape (illustrative, not locked).**

```rust
fn apply_federation_add(
    &mut self,
    event: &Event,
    origin: EventOrigin,        // new parameter
) -> Result<(), SpaceError> {
    if self.dm_constraints_active { ... }
    let node_to_add = match origin {
        EventOrigin::LocallySubmitted => event.content["node_id"].as_str()...,
        EventOrigin::ReceivedViaFederation => event.sender.as_str(),
    };
    if !self.federation_nodes.contains(&node_to_add.to_string()) {
        self.federation_nodes.push(node_to_add.to_string());
    }
    Ok(())
}
```

**Cost.**

- Threads `EventOrigin` through the apply chain. `apply_event` and its dispatcher gain an `origin` parameter, or `apply_federation_add` specifically gains one and a re-dispatch shim handles the other event types.
- Breaks the existing principle that state-apply is a pure function of the event. Today every `apply_*` is `(SpaceState, &Event) → Result<(), SpaceError>`. After Shape A, `apply_federation_add` is `(SpaceState, &Event, EventOrigin) → Result<(), SpaceError>`.
- Requires a verbatim code-comment block at the apply site citing this audit, since the asymmetry is otherwise invisible to a future reader.

**Benefit.**

- Smallest possible code change. One function gains a parameter and a match arm.
- No wire-format change. Existing federation_add events on disk stay valid (subject to Q3 replay concern below).
- No protocol-level new concept. The fix lives entirely in receiver-side apply logic.

**Q3 replay concern (load-bearing).** When B's Node restarts, the persisted federation_add events are replayed from disk to reconstruct `SpaceState`. The current `EventOrigin` enum (`xgen-core::node::runtime::EventOrigin`) has two variants — `LocallySubmitted` and `ReceivedViaFederation`. Replay from disk does not fit either: the events came FROM federation originally but the replay itself is a local operation.

Three sub-options for handling replay under Shape A:

- **Shape A.1 — Persist origin alongside the event.** Add a sidecar field (e.g. `origin: EventOrigin` in the SQLite event-store schema, NOT in the wire-format `Event` struct). At replay time, the persisted origin tells `apply_federation_add` which arm to take. Cost: schema change in event store. Benefit: clean semantic, no apply-time ambiguity.
- **Shape A.2 — Introduce `EventOrigin::ReplayedFromDisk` variant with apply-time fallback rule.** Replay uses the new variant; the apply rule for `ReplayedFromDisk` matches whichever rule applied at original ingest time. But how does it know? Either (a) check whether `event.sender == self_node_id` (replay's "is this my own event" discriminator) → equivalent to checking origin without an explicit field, or (b) add another field/heuristic. Cost: heuristic logic, fragile. Benefit: no schema change.
- **Shape A.3 — Reconstruct origin from event content.** At replay time, check `event.sender == self_node_id`. If yes, replay as `LocallySubmitted`. If no, replay as `ReceivedViaFederation`. This is the same heuristic as A.2 but framed as origin-reconstruction at the replay boundary. Cost: receiver must know its own Node ID at apply time (already true). Benefit: no schema change, no new enum variant.

Shape A.3 is the cleanest of the three, but it makes the "originally local" vs "originally federation" discriminator structurally dependent on `event.sender == self_node_id`. That discriminator is fragile if Node ID ever changes (Node-keypair rotation, hypothetical). It also means the discriminator at ingest time (Phase 4's `EventOrigin` parameter) and the discriminator at replay time (sender-equality check) are different mechanisms answering the same question — a drift surface under D-067.

**Pass 1 impact.** None. `SpaceState.federation_nodes` and `FederationRegistry` shapes are unchanged. `apply_federation_add`'s signature changes, but that's not an XGID retype.

### 7.2 Shape B — Reciprocal event minted on ingestion (JOURNAL shape a, refined)

**Mechanism.** When B's `apply_federation_add` runs on A's event, B ALSO mints a second `state.federation_add` event signed by B's Node keypair, naming A as `content.node_id`. B ingests its own minted event locally. After both events apply, B's federation_nodes contains A. Q1 Reading (ii): two events, one per side.

**Code shape (illustrative, not locked).**

```rust
fn apply_federation_add(&mut self, event: &Event) -> Result<(), SpaceError> {
    if self.dm_constraints_active { ... }
    let node_id = event.content["node_id"].as_str()...;
    if node_id == self_node_id {
        // This event names me as the federated party.
        // Mint a reciprocal naming the sender.
        let reciprocal = build_federation_add_event(
            &self_keypair, ..., peer_node_id: event.sender, ...
        );
        // ... ingest, persist, fan out
    } else {
        // Standard case (this is our own event, or A's view of A's relationship)
        self.federation_nodes.push(node_id.to_string());
    }
    Ok(())
}
```

**Cost.**

- Introduces a new pattern: "ingesting an event causes a new event to be authored." Nothing else in the protocol works this way today. Future contributors reading `apply_federation_add` would need a comment block explaining why this one event family does so.
- The reciprocal event's `prev_events` need defining. The current event has prev_events = A's tips for the Space. The reciprocal should logically reference the original federation_add as its predecessor — but the original was just ingested, so the predecessor relationship is intra-bootstrap-batch. This needs careful design.
- The reciprocal propagates back to A on A's next pull. A sees a federation_add naming A (its own Node ID). Either A's apply logic special-cases "ignore federation_add naming self" (re-introduces the asymmetry-by-special-case pattern Shape A's critique applies to), or A's federation_nodes ends up containing A itself — exactly the same bug shape mirrored.
- Doubles the federation_add count per relationship in archives.
- Idempotency (Q4): on Node restart, the persisted reciprocal replays correctly. But during a re-handshake after disconnect, if A re-runs the a-i symmetry rule (B's tips for S still absent because A's view of B's view hasn't refreshed), A could mint a fresh federation_add for the same relationship. B re-mints its reciprocal. Duplicates accumulate unless the a-i rule itself is tightened with an "already federated" check.

**Benefit.**

- No wire-format change.
- `apply_federation_add` stays a pure function of the event (no `EventOrigin` parameter).
- Each Node's federation_nodes is built purely from events it ingested directly, with no "apply asymmetry."

**Pass 1 impact.** None. Same shapes as today, just twice as many events.

### 7.3 Shape C — Two events at handshake, one per side (JOURNAL shape b, refined)

**Mechanism.** The a-i symmetry rule changes: instead of "the side that has events builds one event," both sides independently build their own `state.federation_add` at handshake time, each naming the other party. Q1 Reading (ii): two events, one per side. The bilateral exchange at handshake provides the wire visibility for both sides to mint without coordination.

**Code shape (illustrative, not locked).** The Phase 3 a-i rule in `stream_federation_delta` extends:

```rust
// Old rule: A builds federation_add when B's tips[S] absent AND A has events.
// New rule: A builds federation_add naming B (as today), AND there is now
// a sibling rule on B's side that builds federation_add naming A.
//
// Both events go into both sides' DAGs via the standard bilateral exchange.
```

Concretely, during the bilateral tip exchange (Phase 3 §3.3 Locked wire shape), both sides' `stream_federation_delta` runs. Today only the side with events mints federation_add. Shape C extends both sides to mint when their counterparty's tips show the Space absent, regardless of whether the local side has events for that Space.

Wait — this needs more care. If B has no events for Space S (true at brand-new bootstrap), B has nothing to put as `prev_events` on a federation_add. The federation_add needs to land in B's DAG with valid prev_events.

Two sub-options for resolving the prev_events question on B's side:

- **Shape C.1 — B's federation_add references A's federation_add as predecessor.** After B ingests A's federation_add for Space S, B mints its own federation_add with `prev_events = [A's federation_add.event_id]`. The DAG structure encodes the relationship: A's event creates the Space-locally-on-B (per P7.5-A), A's federation_add adds B to A's view, B's federation_add (riding on A's federation_add) adds A to B's view.
- **Shape C.2 — B's federation_add references the Space-create event as predecessor.** Same structural shape but anchors directly to `state.space_create` instead of A's federation_add. Cleaner topology but loses the explicit "this is the reciprocal of that" link.

**Cost.**

- Re-opens the locked a-i symmetry rule (Phase 3 §3.3.1 Lock 2). The rule's three-case reasoning needs to be re-walked. The current rule has worked through "A has events but B doesn't," "B has events but A doesn't," "neither has events" — Shape C adds a fourth case shape: "the brand-new side mints a federation_add on receipt that references the other side's federation_add."
- Doubles federation_add count per relationship (same as Shape B).
- The new rule on B's side runs at ingestion time, not at handshake time. The bilateral nature of the handshake doesn't actually help here — by the time B is minting its own federation_add, the handshake is over and the events are flowing through process_inbound. So this is functionally close to Shape B (mint-on-ingest) but with the minting integrated into the federation_session loop instead of into apply_federation_add. The distinction may be a clean abstraction or may be a meaningless code-organisation difference.

**Benefit.**

- Pure apply semantics preserved (no `EventOrigin` parameter in apply).
- The "two events" interpretation is honoured structurally — each side's DAG records the relationship from its own perspective with its own signed assertion.
- Reciprocity is visible in the DAG topology (under C.1).

**Pass 1 impact.** None. Same shapes as today.

### 7.4 Shape D — Wire-format change: extend `content` schema (JOURNAL shape d, refined)

**Mechanism.** `state.federation_add`'s content schema is extended so the event self-describes both parties. The application logic generalises to "push whichever node_id is not our own." Q1 Reading: ambiguous — could be (i) with extended content or (ii) collapsed into one event with symmetric content.

Two sub-options for the schema shape:

- **Shape D.1 — Add `peer_node_id` field.** `content = {node_id: <as today>, peer_node_id: <sender's own ID>, session_id, ...}`. The combination of (`node_id`, `peer_node_id`) fully describes the directional relationship. Receiver picks whichever isn't its own.
- **Shape D.2 — Symmetric `{a_node, b_node}`.** `content = {a_node: <first>, b_node: <second>, session_id, ...}`. Sort by some convention (e.g. lexicographic) so the pair is canonical. Receiver picks whichever isn't its own.

**Code shape (illustrative, D.1).**

```rust
// builder
json!({
    "node_id": peer_node_id,    // as today
    "peer_node_id": sender_id(key),  // NEW — sender's own ID
    // ...
})

// applier
let candidates = [
    event.content["node_id"].as_str()...,
    event.content["peer_node_id"].as_str()...,
];
for n in candidates {
    if n != self_node_id && !self.federation_nodes.contains(n) {
        self.federation_nodes.push(n.to_string());
    }
}
```

**Cost.**

- Wire-format change to `state.federation_add` content schema. Existing federation_add events in archives (test fixtures, dev-build state files) become incompatible unless `#[serde(default)]` is used with a sensible default.
- If `#[serde(default)]` is used with default = empty string, the applier needs to fall back to "if peer_node_id is missing, derive from event.sender" — which is Shape A's heuristic, with wire-format scaffolding on top.
- Appendix C primitive schemas and Appendix I content-struct field tables need updating to reflect the new field.

**Benefit.**

- Cleanest possible semantics. The event self-describes both parties; the application logic is uniform across all ingestors; the directional-vs-symmetric question (Q1) is resolved by the schema itself.
- No `EventOrigin` plumbing in apply. No reciprocal-event minting. The fix lives in the event's own definition.
- Future-proof. If new state events with directional semantics ever exist, this is the schema pattern.

**Pass 1 impact (LOAD-BEARING).** This is the only shape with Pass 1 implications. The Pass 1 runbook (`tasks/XGID_RETROFIT_PASS_1_IMPL.md`) explicitly retypes:

- Appendix C primitive schemas — affected if the schema gains a field.
- Appendix I content-struct field tables — affected if the field is added.
- `event_content_struct_xgid_roundtrip` invariance test — affected because the test must round-trip the new field.

If Shape D is locked, Pass 1's sub-question 4 invariance test list may need extending, and the documentation-pass scope grows by one field row in two appendices. The runbook's current Joe-locked sub-questions remain valid; the structural addition is one new row in Pass 1's coverage table.

If any other shape is locked, Pass 1 proceeds unmodified.

### 7.5 Summary table

| Shape | Q1 reading | Wire change | Apply purity | Replay concern | Pass 1 impact |
|---|---|---|---|---|---|
| A — Origin-aware applier | (i) one event | None | Broken | Yes (Q3) | None |
| B — Reciprocal mint on ingest | (ii) two events | None | Preserved | Yes (Q4 + duplicates) | None |
| C — Two events at handshake | (ii) two events | None (rule change) | Preserved | Limited (Q5) | None |
| D — Schema extension | (i) or (ii) | Yes (content field) | Preserved | Migration needed (Q6) | Yes — new field row in Appx C + I |

Cost-benefit is genuinely distributed across the four shapes. No single shape dominates the others. The design phase walkthrough makes the trade-off explicitly.

---

## 8. Phase-9 + milestone implications

### 8.1 Phase 9 Commit 3a regression witness

Scenario 1 at `xgen-node/src/tests/phase9_two_node_smoke.rs` is `#[ignore]`-annotated with an inline doc comment naming this audit's finding. When the fix lands, the `#[ignore]` lifts. The scenario then becomes the activating regression lock for the bug — any future change that re-introduces the asymmetry will fail Scenario 1.

The scenario stays on disk as authored; no modifications needed when the fix ships. The fix's implementation runbook should include "remove #[ignore] from `phase9_two_node_smoke.rs::scenario_1_two_node_push_smoke` and verify the test passes" as a DoD item.

### 8.2 Phase 9 Commit 3b (Scenarios 2 + 3)

Both remain paused inside the Federation Event Propagation milestone scope. Scenario 2 (anti-transitivity) requires three-Node federation, which requires bidirectional bootstrap to work for at least two of the three pairs. Scenario 3 (drop-and-recover) requires a working federation to drop FROM.

Both unblock when the fix ships. They do not unblock incrementally per fix-shape; any of the four shapes correctly closes the gap as far as Phase 9's other scenarios are concerned.

### 8.3 Compound scenarios C2 / C3 / C5 / C7 / C9 / C10

All six compound scenarios from the Phase 9 survey (`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`) presuppose working bidirectional federation. All six are paused behind the same gate.

The four deferred compounds (C1 / C4 / C6 / C8) in `tasks/FEDERATION_STRESS_FOLLOWON.md` remain blocked on the clock-injection seam, independently of this fix. They are unaffected by the bidirectional fix decision.

### 8.4 M6 (new) Node admin write path

Remains 🟡 PENDING behind Federation Event Propagation milestone closure. The bidirectional fix is now part of the milestone closure dependency chain. M6 (new) is unaffected in scope by this audit; the dependency chain just gained one more node.

### 8.5 Pass 1 implementation

Status: ACTIVE v2.0 with all four sub-questions Joe-locked. The Pass 1 runbook is unmodified by Shapes A, B, or C. Shape D requires extending Pass 1's coverage by one field row in Appendix C + Appendix I. The decision is downstream of the design-phase Joe-lock; until then, Pass 1 stays at v2.0.

---

## 9. Discipline notes

This audit is a worked instance of three project-management principles already in DECISIONS.md:

- **D-069 (canonical-document rule).** This document is the canonical record of the bidirectional `federation_nodes` audit. Future references to the finding cite this document.
- **D-071 (subsystem audits precede dependent milestones).** The audit runs before the design phase that fixes the gap. The design phase has this document as its Pass 1 input, mirroring how Phase 7.5's design phase had its own design task file as input to its implementation runbook.
- **D-074 (milestone-close commits include JOURNAL).** When the design phase closes, its closure commit will include a JOURNAL entry. When the implementation runbook closes, that closure commit will also include a JOURNAL entry.

It is also a worked instance of the "honest longer work over fast shortcuts" principle (Cross-cutting principles in ROADMAP.md). Phase 9 Commit 3a could have shipped without surfacing the finding (Scenario 1 stripped from disk instead of `#[ignore]`-annotated). It could have shipped with the finding privately noted and Scenarios 2+3 attempted as workarounds. Neither would have been honest about the protocol's actual state. The path taken — ship the regression witness, stand down the milestone, document the audit, walk the design phase, ship the fix — is longer than the workarounds and shorter than rediscovering the bug from a production deployment.

The shape is sibling to the Phase 7.5 precedent. Phase 7.5 stood down Phase 9 at Commit 3 boundary, ran its own design + implementation, then Phase 9 resumed. This audit's design phase stands down Phase 9 at Commit 3a boundary, runs its own design + implementation, then Phase 9 resumes.

---

## 10. Cross-references

### 10.1 Design documents

- **`docs/xgen_federation_propagation_design.md`** (Status: ACTIVE, v1.0) — the canonical Federation Event Propagation design. §6.4 Phase 7 Lock A1 + B1, §6.4.1 Phase 7.5 P7.5-A through P7.5-D, §15 Implementation Complete table.
- **`tasks/FEDERATION_PROPAGATION_COMPLETION.md`** (Status: ACTIVE, v1.0) — the implementation runbook. §3.3 Phase 3 wire shape, §3.3.1 Lock 2 (a-i symmetry rule), §3.4.1 Phase 4 implementation locks, §3.7.1 Phase 7 implementation locks.
- **`tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md`** (Status: COMPLETED v1.0) — Phase 7.5 design task file. Sibling-in-shape to this audit's downstream design phase.
- **`tasks/FEDERATION_PROPAGATION_PHASE_7_B3_AMENDMENT.md`** (Status: COMPLETED v1.0) — Phase 7 B3 amendment. Sibling shape: design gap surfaced during implementation, closed mid-milestone.
- **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (Status: ACTIVE v1.0) — Phase 9 implementation task file. Scope intact; Commit 3a shipped; Commit 3b paused.

### 10.2 Code surfaces

- `xgen-node/src/federation_session.rs::stream_federation_delta` (lines 75-175) — sender-side `state.federation_add` construction under the a-i symmetry rule.
- `xgen-core/src/space/state.rs::build_federation_add_event` (lines 910-933) — event builder.
- `xgen-core/src/space/state.rs::apply_federation_add` (lines 351-363) — receiver-side application logic.
- `xgen-core/src/node/runtime.rs::dispatch_event` — Phase 7 F-3 check; Phase 7.5 P7.5-B HeldPending third trigger.
- `xgen-node/src/tests/phase9_two_node_smoke.rs` — Phase 9 Scenario 1 regression witness (`#[ignore]`-annotated).
- `xgen-node/src/tests/phase9_harness.rs` — Phase 9 in-process two-Node harness module.

### 10.3 JOURNAL

- **J-092 Commit 3a sub-entry** — the originating record of the finding, with the four candidate fix shapes that this audit refines.
- J-088 (Phase 7 closure) — F-3 implementation, sibling locks A1 + B1.
- J-093 (Phase 7.5 design closure) — sibling precedent for the design-phase pattern this audit feeds into.
- J-094 (Phase 7.5 implementation closure) — sibling precedent for the implementation-phase pattern that follows the design phase.

### 10.4 DECISIONS

- **D-065** — Honest behaviour over polite behaviour. Cited in §9; informs the standdown-and-fix posture over workarounds.
- **D-067** — Single source of truth. Cited in §7.1 Shape A.3 critique.
- **D-069** — Canonical-document rule. This document is the canonical home for the audit finding.
- **D-071** — Subsystem audits precede dependent milestones. The discipline this document follows.
- **D-074** — Milestone-close commits include JOURNAL. Forward-binding for the design + implementation phases that follow.

---

*End of audit document. Design phase walkthrough is next-active for Chat Claude + Joe. The walkthrough resolves Q1 + Q2 (§6.1, §6.2) and picks one of the four candidate fix shapes (§7) with Joe-lock. The locked design phase produces its own task file (sibling to `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md`) that captures the framework decisions. After the design phase closes, the implementation runbook is authored (sibling to `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md`) and handed off to Clair.*  
