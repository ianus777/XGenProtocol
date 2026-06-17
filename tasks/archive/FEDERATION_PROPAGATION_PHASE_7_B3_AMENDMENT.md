# Federation Event Propagation — Phase 7 B3 Amendment
> **Status**: COMPLETED  
> Version: 1.0 (Joe-lock walkthrough closed 2026-05-20; lock walked to final form on both §4.1 and §4.2; Commit 3.5 implementation shipped 2026-05-20)  
> Date: May 2026  
> **Last updated**: 2026-05-20 (Status flipped ACTIVE → COMPLETED at Commit 3.5 close. Implementation shipped: `validate_event` gained `fed_add_via_federation: bool` parameter; `dispatch_event` derives it as `peer_node_id.is_some() && event.event_type == StateFederationAdd`; federation-relationship arrival hook lifted from `xgen-node::app::process_inbound` into `dispatch_event` Step 7 so every caller exercises it under the runtime lock; `resolve_federation_relationship` gained `reindex_after_partial_release` helper to prevent buffer-entry orphaning when sibling drain-released events haven't landed in the store yet; canonical design doc §6.4 gained the B3 paragraph; runbook §2 reflects the new Commit 3.5 slot; B3 unit tests + paused Commit 4 integration tests both green at 556 tests.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This is the **Joe-locked B3 amendment** to Phase 7's existing B-series (B1, B2) covering `state.federation_add` event-validation behaviour. B1 (locked 2026-05-18) skipped the F-3 federation-relationship gate for `state.federation_add` so federation could bootstrap. B3 (locked 2026-05-20) closes the remaining downstream validation gates that block federation_add from completing ingestion on the receiver.

This is not a P7.5-A extension. P7.5-A (locked 2026-05-19) covers `state.space_create` and `state.dm_space_create` at F-3 + F-4 step 1. The federation_add validation gates are a sibling concern, structurally identical to B1's reasoning: events that bring a relationship into existence cannot be gated by checks that presuppose the relationship.

Phase 7.5 Commit 4 (NodeRuntime-level integration tests) is paused on this amendment. After B3 locks, the runbook gains a new commit (3.5 — implement B3) before Commit 4, and Commit 4 ships.

---

## 2. Why this is a B-series amendment, not a P7.5-A extension

Phase 7 (F-3) and Phase 7.5 (cold-start bootstrap) are distinct phases with distinct lock series:

- **B1 (Phase 7, locked 2026-05-18)** — F-3 federation-relationship check skips `state.federation_add` arriving over a federation session. Authority: session-level handshake auth (Node-keypair) + event-level signature (same keypair).
- **B2 (Phase 7, deferred)** — narrowing the B1 skip to "sender == wire-authenticated peer == federation_add.adds_node". Not done at v1; layers on top of B1 cleanly if a future threat model justifies.
- **P7.5-A (Phase 7.5, locked 2026-05-19)** — narrow skip rule extension for `state.space_create` + `state.dm_space_create` at F-3 + F-4 step 1. Sibling to B1 — but B1 covered a different gate, and P7.5-A covers a different EventType set.

B3 belongs to the B-series because:

1. **Same EventType as B1.** `state.federation_add`. B1 already named this EventType as structurally special.
2. **Same structural reasoning as B1.** "Events that bring a relationship into existence cannot be gated by checks that presuppose the relationship." B1 articulated this for F-3; B3 extends it to F-4 step 9 / step 11 / step 13.
3. **Latent since J-088 (Phase 7 close).** Pre-Phase-7.5 no production path exercised federation_add through `validate_event` — the Phase 7 test `state_federation_add_skips_f3_check` explicitly notes "Other outcomes (...other rejections from downstream validation steps) are all consistent with B1 being applied. The test does not constrain the downstream outcome." Phase 7 surfaced this exact uncertainty and bounded its claim to F-3 only. B3 is the closure Phase 7 anticipated.
4. **Not P7.5-A-shaped.** P7.5-A is about `state.space_create` / `state.dm_space_create` at gates that presuppose the Space exists. B3 is about `state.federation_add` at gates that presuppose Space membership or predecessor ordering. Different EventTypes, different gates, different authority surfaces.

**Production-gap framing.** This has been a latent gap since Phase 7 closed. The Phase 9 survey (J-090) named F-3 cold-start bootstrap (M5) as the explicit blocker; it did not catalogue the federation_add downstream-validation gap because the F-3 reject (M5) blocked the test path before federation_add validation could even run. Phase 7.5 Commit 4 is the first production-shaped test path to exercise federation_add through `validate_event`. Worth recording for the same reason J-081 recorded the federation-push absence (D-071 "subsystem audits precede dependent milestones" — design gaps surface during dependent work and close before the dependent work proceeds).

---

## 3. The two validation gates that block federation_add

Two distinct gates fail when `state.federation_add` arrives via federation on a cold receiver. Both surfaced during Phase 7.5 Commit 4 diagnostic tests.

### 3.1 Step 9 predecessor presence (predecessor-chain deadlock)

`state.federation_add` is topologically last in the bootstrap stream — its `prev_events` references the latest tip on the sender side, which (per [federation_session.rs:120](../xgen-node/src/federation_session.rs:120) — `our_local_tips`) is typically the most recent membership.join or message.text event.

On the receiver, all the events between `state.space_create` and `state.federation_add` (`state.room_create`, `membership.invite`, `membership.join`, …) fail F-3 and enter HeldPending on Phase 7.5's federation-relationship trigger. They are NOT in the local store — they sit in `PendingBuffer::events`.

When `state.federation_add` then arrives at the validation core, [exchange.rs:426-432](../xgen-core/src/message/exchange.rs:426) checks:

```rust
let unknown: Vec<String> = event
    .prev_events
    .iter()
    .filter(|id| !store.contains(id.as_str()))
    .cloned()
    .collect();
if !unknown.is_empty() {
    return ValidationOutcome::HeldPending { missing_predecessors: unknown, missing_identity: None };
}
```

The predecessors are NOT in the store. They are in the pending buffer. `validate_event` returns `HeldPending` on missing predecessors.

Net deadlock:

| Event | Held on trigger | Waiting for |
|---|---|---|
| `state.room_create` | federation-relationship | `state.federation_add` arrival |
| `membership.invite` | federation-relationship | `state.federation_add` arrival |
| `membership.join` | federation-relationship | `state.federation_add` arrival |
| `state.federation_add` | predecessor (`membership.join.event_id`) | `membership.join` arrival in store |

`state.federation_add` can never escape because the only event that can satisfy its predecessor is itself held waiting on `state.federation_add`. After 30 s (predecessor-trigger timeout, F-4a) the federation_add times out with 4002; after 180 s the rest time out with 4007.

**Captured failure output** (xgen-node/src/tests/cold_start_bootstrap_integration.rs, `diagnostic_predecessor_chain_deadlock`):

```
DIAGNOSTIC: federation_add HeldPending — predecessor-chain deadlock
```

### 3.2 Step 11 sender-membership (Node URI not a Space member)

Bypass the predecessor-chain issue (ingest predecessors directly via `ingest_event` test-shortcut so step 9 passes), and the next downstream gate fires. `state.federation_add`'s sender is the peer Node's URI per Phase 4 §3.4.1 Q3 overload — [federation_session.rs:117](../xgen-node/src/federation_session.rs:117) constructs `build_federation_add_event(node_keypair, ...)`, so the sender field is the Node-keypair pubkey URI.

`validate_event` step 11 ([exchange.rs:461-484](../xgen-core/src/message/exchange.rs:461)) lists the EventTypes that skip the membership check:

```rust
let skip_membership = matches!(
    event.event_type,
    EventType::MembershipJoin
        | EventType::StateSpaceCreate
        | EventType::StateDmSpaceCreate
);
if !skip_membership {
    let space = ...;
    if !space.is_member(sender) {
        return ValidationOutcome::Rejected(ExchangeError::NotASpaceMember);
    }
    ...
}
```

`StateFederationAdd` is NOT in the skip list. The sender (a Node URI) is NOT in `space.members` (members are Identities, not Nodes). `validate_event` returns `Rejected(NotASpaceMember)`.

**Captured failure output** (`diagnostic_federation_add_step_11_sender_membership`):

```
DIAGNOSTIC: federation_add Rejected at validation core: step 11: sender is not a Space member
```

### 3.3 Step 13 permission (symmetric concern)

Step 13 ([exchange.rs:493-499](../xgen-core/src/message/exchange.rs:493)) is gated by the same `skip_membership` flag and runs `check_permission(event, space)` for non-skipped EventTypes. For `state.federation_add` the permission check would check whether the (non-member) sender holds a role allowing federation-add — which they don't (no role). Even if step 11 were bypassed, step 13 would fail.

Step 13's `skip_membership` reuse means a single fix at the flag covers both, but the design proposal should name step 13 explicitly so the implementation runbook lists both as DoD items.

---

## 4. Proposed B3 lock

### 4.1 What the rule says

`[JOE-LOCK: locked 2026-05-20]`

`state.federation_add` events arriving via a federation channel (i.e., `peer_node_id.is_some()` at `dispatch_event` entry) bypass the following validation-core gates in addition to F-3 (already skipped per B1):

- **Step 9 (predecessor presence).** Predecessor check skipped — federation_add may reference predecessors that are still in the receiver's HeldPending buffer (which will drain on federation_add's own ingestion via the Phase 7.5 §6 arrival hook). Without this skip, the predecessor-chain deadlock in §3.1 fires.
- **Step 11 (sender registration + sender membership — full skip).** Both halves of step 11 skipped. The first-half check (`IdentityRegistry::contains(sender)` — F-10 unknown-signer trigger) is registry-keyed by `identity_id` ([xgen-core/src/identity/registry.rs:96](../xgen-core/src/identity/registry.rs:96)); the registry's populating paths (`IdentityMessage::Register` and `handle_incoming_replicate`) NEVER insert Node URIs. A federation_add signed by a Node-keypair (per Phase 4 Q3 overload) would F-10-buffer forever waiting for an Identity record that will never arrive. The second-half check (`SpaceState::is_member(sender)`) fails because Node URIs are by design not Space members. **Both fail for the same Q3-overload reason: federation_add is Node-authored, not Identity-authored.** Authority for the signer is established entirely by step 12 (signature verification — see below); no registry lookup is needed.
- **Step 13 (sender permission).** Permission check skipped — symmetric concern with step 11's second half (no member role = no permission for any state event).

The following steps are NOT skipped:

- **Step 8 (event_id hash).** Canonical-form hash check fires unchanged.
- **Step 10 (DAG structure).** root/non-root DAG-shape check fires unchanged. `state.federation_add` is non-root; `prev_events` must be non-empty (currently enforced).
- **Step 12 (signature verification).** Fires unchanged. `verify_event_signature` ([xgen-core/src/space/state.rs:684](../xgen-core/src/space/state.rs:684)) is pure crypto: it decodes the pubkey from the `xgen://pubkey/ed25519:` URI prefix and verifies via `ed25519_dalek`. The Q3 overload is transparent at this layer — Identity URIs and Node URIs share the same wire shape, so step 12 verifies federation_add signatures correctly without any registry-side adjustment.

The authority for a federation_add arriving via federation channel after B3 is:

1. **Session-level handshake auth.** Node-keypair authenticated the wire session at handshake time.
2. **Event-level signature.** Same keypair signed the event content; step 12 verifies cryptographically against the pubkey embedded in the sender URI.
3. **Structural sanity.** Step 8 + step 10 confirm the event is well-formed and references at least one predecessor (the F-1a tip).

The implicit Q3-overload bridge is recorded here for posterity: the validation core treats `event.sender` as an opaque pubkey URI; only step 11-first-half coupled it to the Identity registry. Skipping that coupling for federation_add closes the F-10-deadlock for Node-authored events without weakening cryptographic authority.

### 4.2 Scope: narrow to federation channel

`[JOE-LOCK: locked 2026-05-20]`

The skip narrows to "federation_add arriving via federation channel" (`peer_node_id.is_some()` at `dispatch_event`), not all federation_add events. Locally-submitted federation_add (M6 admin write path) may have a different sender shape (admin Identity, not Node URI) and should retain full validation. M6 will revisit if needed; this amendment does not preempt that scope.

### 4.3 Why this is not weakening F-4

Same reasoning shape as B1's "deferred not weakened" argument (Phase 7.5 §6.4):

- **Authority chain intact.** Session handshake + event signature provide cryptographic authority. The Q3-overload trace in §4.1 shows step 12 is sufficient to verify a federation_add signed by a Node keypair, since the pubkey is encoded in the sender URI. The Identity-registry lookup at step 11-first-half added nothing for Node-authored events (and in fact prevented them from ever ingesting).
- **Structural necessity.** federation_add IS the relationship-establishing event. Step 9 / step 11 / step 13 presuppose the relationship exists (predecessors landed, sender is a member, sender has a member role). Applying them to federation_add is logically equivalent to applying F-3 to it — which B1 already correctly skipped.
- **Bounded blast radius.** The skip applies to one EventType, on one channel direction (incoming federation), gated by a wire-authenticated `peer_node_id`. A malicious peer would need a valid Node-keypair signature on the event to pass step 12; key access is the operator-controlled trust boundary.

### 4.4 DoS consideration

Same envelope as P7.5-A's analysis (Phase 7.5 §5.3 — DoS surface); cited by reference rather than restated.

In summary: (1) federation peers are operator-authorised (not anonymous) and operator removal terminates the surface immediately; (2) content-determinism of any structurally-meaningful identifier (Space IDs, event IDs) prevents collision attacks; (3) misbehaving-peer cleanup is operator-driven via peer removal + future M6 admin write-path tooling (rate limiting deferred to M6). B3 adds no new surface beyond what P7.5-A §5.3 already analysed — both touch the same "what can a wire-authenticated peer make a cold receiver do" boundary, and the answer is unchanged: a peer can introduce structurally-novel events into the receiver's local store, bounded by signature + handshake auth and cleanable by operator action.

See P7.5-A §5.3 for the full operator-authorised + content-determinism + SpaceLocalMetadata-triage triple analysis.

### 4.5 Implementation site

Cleanest implementation is at `dispatch_event` rather than `validate_event` — same site as the existing B1 skip and Phase 7.5 §5 skips:

```rust
// Hypothetical Commit 3.5 shape — confirm during implementation.
let skip_validation_core = matches!(event.event_type, EventType::StateFederationAdd)
    && peer_node_id.is_some();
if skip_validation_core {
    // Inline verification of the gates we DO want: F-10 unknown-signer +
    // signature. Sketch only; final shape is Clair's latitude.
    if !self.identity_registry.contains(&event.sender) {
        // F-10: buffer on Identity trigger.
        ...
    }
    if !verify_event_signature(&event) {
        return DispatchOutcome::Rejected("signature verification failed".to_string());
    }
    // Skip step 9 (predecessor), step 11 (membership), step 13 (permission).
    self.ingest_event(event);
    return DispatchOutcome::Accepted { new_joiner: None };
}
```

Alternative: keep `validate_event` as the single dispatcher and add `StateFederationAdd` to `skip_membership` plus a parallel predecessor-skip flag. Either shape is acceptable; the design lock is the behaviour (skip step 9 + step 11 + step 13 for federation-channel federation_add), not the code layout.

---

## 5. Where this lands in the commit sequence

Phase 7.5 implementation runbook ([tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md](FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md)) gets a new commit slotted between Commit 3 (HeldPending third trigger, shipped) and Commit 4 (integration tests, paused):

| # | Commit | Status |
|---|---|---|
| 1 | Doc-pass | ✅ Shipped (12cfe5a) |
| 2 | F-4 step 1 + F-3 skip + SpaceLocalMetadata | ✅ Shipped (aa2433f) |
| 3 | HeldPending third trigger + 4007 + counter | ✅ Shipped (1be7189) |
| **3.5** | **B3 amendment: skip step 9 + 11 + 13 for federation_add via federation** | **🟡 PROPOSED (this document)** |
| 4 | NodeRuntime cold-start bootstrap integration tests | ⏸ PAUSED pending Commit 3.5 |
| 5 | Phase 7.5 close | ⏸ PENDING |

Commit 3.5 ships:
- The skip logic at `dispatch_event` (or equivalent).
- Verbatim B3 code-comment block citing this proposal's §4.1.
- A handful of dispatcher-level unit tests in `xgen-core/src/node/runtime.rs::phase_7_5_tests` covering: federation_add via federation Accepts on cold receiver (when signature + identity preconditions satisfied); federation_add via federation HeldPending on unknown signer (F-10 still applies); locally-submitted federation_add still hits full validation (narrowness regression).

Commit 4 then ships with the original six scenarios A-F passing.

---

## 6. Open implementation question (small, Clair-latitude after this lock)

Code-layout choice: skip-in-dispatch_event (sketch above) vs. extend-skip_membership-and-add-predecessor-skip-flag-in-validate_event. The DoD captured here is behavioural — both layouts produce the same Accepted/Rejected outcomes for the same inputs. Clair picks the cleaner layout at Commit 3.5 implementation time and documents the choice in the journal entry. The lock above does not constrain the choice.

---

## 7. Definition of Done

B3 amendment design phase is complete when:

- [x] Joe-lock walkthrough closes with §4.1's behaviour locked. (Closed 2026-05-20.)
- [x] `[JOE-LOCK: locked YYYY-MM-DD]` marker walked to final form on §4.1 + §4.2. (Both walked to `[JOE-LOCK: locked 2026-05-20]`.)
- [ ] Phase 7.5 implementation runbook updated with Commit 3.5 entry (§5 table). (Pending — Clair updates runbook at Commit 3.5 start.)
- [ ] Canonical design doc (`docs/xgen_federation_propagation_design.md`) §6.4 gains a sibling B3 paragraph following B1's framing (at runbook Commit 3.5's doc-pass step, not at this design lock — same discipline as B1's "Pass 3 promotion" pattern).
- [ ] This document flipped to Status COMPLETED at the same time as the runbook update + design-doc paragraph land in Commit 3.5.
- [ ] Status ACTIVE through Commit 3.5; flips to COMPLETED in the same commit that ships the implementation + canonical-doc update.

---

## 8. Cross-references

- [tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md](FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md) — Implementation runbook (ACTIVE v1.0). Gains Commit 3.5 entry once B3 locks.
- [tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md](FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md) — Phase 7.5 design (COMPLETED v1.0). B3 is sibling, not contained.
- [docs/xgen_federation_propagation_design.md](../docs/xgen_federation_propagation_design.md) §6.4 — F-3 framework with Phase 7 B1 + B2. B3 lands here as a sibling paragraph.
- [xgen-core/src/message/exchange.rs:395-502](../xgen-core/src/message/exchange.rs) — `validate_event` function; step 9 + step 11 + step 13.
- [xgen-node/src/federation_session.rs:111-145](../xgen-node/src/federation_session.rs:111) — a-i symmetry rule producing federation_add events with Node-keypair sender.
- [xgen-node/src/tests/cold_start_bootstrap_integration.rs](../xgen-node/src/tests/cold_start_bootstrap_integration.rs) — Paused Phase 7.5 Commit 4 tests + the two diagnostic tests that captured the failure outputs in §3.1 + §3.2.
- D-065 (honest behaviour over polite behaviour) — federation_add should land on a cold receiver, not pretend to via test-only ingest shortcuts.
- D-069 (Joe-locked design phase + canonical-document rule) — discipline that produced this document.
- D-071 (subsystem audits precede dependent milestones) — extends to "design gaps surface during dependent work and close before the dependent work proceeds"; B3 is Phase 7.5 Commit 4's instance.

---

*End of document. Joe-lock walkthrough closed 2026-05-20. Phase 7.5 Commit 3.5 implementation is the next step per §5's table.*  
