# XGen Federation Event Propagation — Design

> **Status**: PENDING  
> Version: 0.3  
> Date: May 2026  
> **Last updated**: 2026-05-18 (Pass 2 in progress; F-1, F-2, F-3 Joe-confirmed in conversation; F-3 section written with layered-authority reasoning)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools. This document is the deliverable of the Federation Event Propagation milestone's Joe-locked design phase, per the D-069 canonical-document rule.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This document is the canonical design for the Federation Event Propagation completion milestone. It specifies the mechanism by which an event accepted into one XGen Node's DAG reaches federated peer Nodes — and what guarantees that mechanism provides.

The milestone exists because the Propagation Reliability Audit (J-081, `docs/xgen_propagation_reliability.md`) found that Node-to-Node federation event propagation does not exist as a production mechanism in the current implementation. The federation surface today is one-time history dump on peer-initiated handshake, then connection close. No persistent peer session, no outbound event push, no DAG-tip reconciliation, no gap-recovery mechanism. This document specifies the mechanism that closes that gap.

It is the canonical document for the Federation Event Propagation milestone per the D-069 canonical-document rule. Future edits to the design land here, not in `tasks/` addenda or in DECISIONS.md notes. The implementation runbook (`tasks/FEDERATION_PROPAGATION_COMPLETION.md`) — when written, after Pass 3 closes — is a runbook against this design, not an alternative design surface.

**This document is partly protocol, partly reference implementation.** The protocol-level additions it specifies — new wire messages, validation rule changes — will land in Chapter 3 in the implementing commit. The Node-implementation pieces (per-peer record, reconnect scheduling, admin UI surfaces) are reference-implementation specification and belong alongside Chapter 4 and the M6 admin design doc, not in Chapter 3. Each section flags which layer it operates at.

### 1.1 Phase 0 design provenance

This document is produced by the milestone's Phase 0 design phase. The phase runs in three passes per the D-069 discipline:

- **Pass 1 — audit current state.** Already done. J-081 is the audit; this milestone inherits it. No re-audit.
- **Pass 2 — proposals + Joe-lock markers.** Surface design alternatives with trade-offs; mark every framework decision with `[JOE-LOCK]`; surface decisions one at a time, not as a wall. **This document is the Pass 2 working draft.**
- **Pass 3 — lock framework decisions + canonical doc.** Walk all `[JOE-LOCK]` markers; lock; promote `Status` to `ACTIVE`; mark Pass 2 work superseded.

Pass 2 is governed by `tasks/FEDERATION_PROPAGATION_DESIGN.md`. After Pass 3 closes, that task file is marked COMPLETED and the runbook task file is written.

---

## 2. Background

### 2.1 What the audit established

The audit traced the propagation lifecycle from event submission through to federated-peer delivery and identified, with code-grounded evidence, where the mechanism breaks down. The headline findings:

- **Stage 6 (Node-to-Node federation propagation) is architecturally absent.** Three independent traces converged: (a) the production `xgen-node/src/` codebase contains zero callers of `run_initiating`; (b) no pull mechanism exists Node-to-Node (`space.join_request` is only received in production, never sent); (c) the stress-test "Federation Completeness" check measures only each Node's local-clients delivery, not cross-Node propagation. Audit §2 verdict: GAP IDENTIFIED — severity HIGH.

- **The `process_inbound` validation pipeline applies asymmetrically across event types.** Path A (message events via `accept_message`) runs the full 13-step validation including signature verification, timestamp checks, and HeldPending buffering for unknown predecessors. Paths B (membership.join) and C (other state events) bypass signature and timestamp verification and have no HeldPending integration. Severity LOW today (locally-authenticated submission is the only entry point) but HIGH the moment a federation push channel exists, because federation propagation is the exact vector that would make Paths B/C reachable with unverified events. Audit §3 sub-finding.

- **The existing `transport.sync_request` mechanism has documented gaps that become relevant when the design extends it Node-to-Node.** The spec-defined `sync_response` and `sync_complete` reply shapes (Ch3 §3.3.6) are unimplemented; the client uses a 500ms quiet-time timeout for end-of-stream detection. No pagination on `collect_sync_history`. Unknown-`since` returns silent-empty with no signal back. Audit §4 sub-findings.

- **Documentation drift.** `docs/xgen_node_admin_ops_design.md` §4.2 and `docs/xgen_ch4_implementation.md` lines 779, 825-827 describe Node-to-Node federation and `transport.sync_request` mechanisms that do not exist. The audit recorded these for correction in this milestone's documentation pass.

### 2.2 Why this milestone exists, not a code patch

The gap is not a bug in a known mechanism — it is the absence of a load-bearing protocol-implementation surface. Future protocol promises depend on Stage 6 working: the M6 `EventAccepted` G2 semantic is meaningful only when DAG-resident events reliably reach the rest of the system; multiparty deployments in M8/M9 require cross-Node propagation; production federated topologies require all of it.

Closing the gap by patching is not appropriate. A push channel needs wire-protocol shape decisions, a session model, identity-authority decisions, and validation-pipeline changes. None of those are local fixes. The milestone exists to make those decisions as canonical design before any code is written, per D-069.

### 2.3 The validation asymmetry is a precondition, not parallel work

Closing federation push without closing the validation asymmetry would land a vulnerability: a peer could push membership or state events purporting to come from any Identity, and the receiving Node's Paths B/C would accept and persist them without signature verification. The two pieces of work belong in the same milestone, in the same coordinated design phase, in the same implementation runbook. This document treats them as one body of work.

---

## 3. Scope and non-scope

### 3.1 In scope

- **Federation event push.** The new mechanism by which Node A's accepted events reach federated peer Node B.
- **Federation gap recovery.** How peer Nodes detect and recover from missed events (handshake-time, mid-stream, on-reconnect).
- **`process_inbound` validation asymmetry closure.** Lifting Paths B and C to the same signature + timestamp + HeldPending discipline that Path A has today.
- **Federation handshake evolution.** What the handshake produces in the new model (tip exchange replacing the today's full-history dump as the post-handshake handoff to push).
- **Per-peer record at the Node-implementation layer.** Persistent state about every known peer Node, including operational metadata (lost-connection flag, last-seen, reconnect schedule) and operator-set custom settings. Read by reconnect scheduling and the admin UI; invisible on the wire.
- **Documentation correction.** Update `docs/xgen_node_admin_ops_design.md` §4.2 and `docs/xgen_ch4_implementation.md` lines 779, 825-827 to describe the new mechanism in place of the absent one.

### 3.2 Out of scope

- **M6 (new) admin verb work.** Blocked behind this milestone's ACTIVE flip; that is its own milestone.
- **Wire-layer rejection signal for event acceptance.** The M6 (new) Phase 2 envelope-level `event_id` work, Joe-locked direct at audit close. Coordinated with this milestone (the validation-pipeline changes here may surface events that need rejection signalling), but its design is in `docs/xgen_node_admin_ops_design.md` §6.5, not here.
- **Transitive federation.** Audit §3.5 flagged as MEDIUM-deferred. The design phase may sketch as future work but does not lock.
- **MLS operationalisation.** Independent parallel workstream (D3 in the project roadmap), not affected by this milestone.
- **Compaction / event-store eviction.** No compaction mechanism exists today; this milestone does not add one. The unknown-`since` silent-empty behaviour from audit §4.4 is recorded as a future scaling concern, not solved here.
- **Test plan and runbook.** Pass 3 closes; then a separate runbook task file is written for Clair.

### 3.3 Non-scope decisions explicitly recorded

These are out-of-scope but are recorded here so that future readers know they were considered and deferred, not forgotten:

| Item | Why deferred | Where it lands |
|---|---|---|
| Compaction-aware sync | No compaction exists; can't design recovery against an absent mechanism | Future scaling milestone |
| Pagination on `collect_sync_history` | Audit §4 sub-finding; scope decision pending in F-7 below | Possibly in scope (see F-7) |
| Cross-Space topological order | Audit §4 LOW sub-finding; pre-existing M4 carry-over | Existing carry-over, not this milestone |
| Transitive federation wire | Audit §3.5; locking by accident in either direction is worse than not locking | Future milestone or this one's "future considerations" appendix |

---

## 4. Framework decision F-1 — Federation push direction

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

### 4.1 The question

When event E is accepted into Node A's DAG (audit lifecycle Stage 4), by what mechanism does E reach peer Node B (a federation peer with shared Space S)?

### 4.2 Options considered

**Option 1 — Push from home.** Node A holds a persistent outbound WebSocket connection to each federated peer. On Stage-4 accept, the same mechanism that calls `apply_fanout` for local clients also pushes E to each peer Node sharing the relevant Space. The peer receives, validates, ingests, fans out to its own local clients.

- ✅ Low latency; symmetric with local fan-out; originating side knows when to push.
- ⚠️ Connection-state management on both sides; buffering question on peer-down; validation-asymmetry exposure.

**Option 2 — Pull from peer.** Each peer periodically asks the home Node for new events since their last known tip. No push; everything is request/response.

- ✅ Connection initiative is recipient-side; reuses the existing `sync_request` mental model; no outbound queue on home.
- ❌ Latency floor = polling interval; burst load on home Node when peer comes back; peer doesn't know what it doesn't know.

**Option 3 — Hybrid.** Default mode is push (Option 1). On peer-side detection of a gap, peer issues a pull request (Option 2) to recover. Persistent peer session for push, sync-request shape for pull-recovery.

- ✅ Best of both: low latency in steady state, pull-on-gap recovery; no mandatory outbound buffer; reuses existing HeldPending and `sync_request` infrastructure.
- ⚠️ Two mechanisms to design and maintain; decision points proliferate.

### 4.3 Decision — Option 3 (hybrid)

**Push for steady state, pull for gap recovery.** Node A maintains a persistent peer session with each federated peer B. On Stage-4 accept of an event E that belongs to a Space B participates in, A pushes E over the peer session. B receives, validates, ingests, fans out to its own local clients. If B's validation discovers E has unresolved predecessors (HeldPending case), B issues a pull request to A for the missing predecessor range — reusing the gap-detection mechanism that already exists for `accept_message` Path A. If the peer session is unhealthy or partitioned, recovery falls to pull-on-reconnect (see §4.5).

**Reasoning recorded.** Two of the three pieces already exist in some form. Pull lives client-to-Node in `transport.sync_request` (audit §4.1). Gap detection lives in the HeldPending buffer pattern (audit §3.2 Path A). Push is the genuinely new piece. Hybrid composes the existing pieces and adds one; pure-push or pure-pull would each require building one new mechanism while ignoring infrastructure that exists for a reason.

### 4.4 Sub-decision F-1a — Initial handshake produces tip exchange, not full dump

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

**Today's behaviour.** When peer Node B initiates a federation handshake to home Node A, A snapshots Space S's full event history under the runtime lock, streams it in topological order, then closes the connection (audit §2.1). The session is one-shot.

**Decision.** The handshake is reshaped to a **tip exchange**. Peer B sends its current tip per shared Space (or empty if new to the relationship); home A responds with the delta — events that follow B's tip in the DAG, up to A's current tip. For a brand-new relationship where B has no tip, the delta is the full history. For recovery after a short downtime, the delta is small. After delta delivery completes, the session **stays open** as the persistent push channel established by F-1.

**Reasoning recorded.** Tip exchange is symmetric for first-contact and recovery cases — first-contact is just "the biggest possible gap" — and that symmetry fits the hybrid principle (push for steady state, pull for gap recovery, "first contact" is the largest gap). Option B is more consistent with F-1c (both sides maintain per-peer records) because the peer's tip is exactly the kind of per-peer state F-1c specifies. The marginal implementation cost is lower than it first looks because the validation-asymmetry work in F-4 is going to touch the handshake code regardless.

### 4.5 Sub-decision F-1b — Buffering on peer-down: drop, recover via pull

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

**The question.** Home Node A pushes event E to peer Node B. B is unreachable (connection dropped, network partition, B is restarting). What does A do with E?

**Decision.** **Drop the push at the protocol layer. No outbound queue. Recovery is the peer's job via pull on the next successful session establishment.** When B comes back, B's handshake tip exchange (F-1a) picks up everything A has accepted since B was last in sync.

**Reasoning recorded.** Consistent with the rest of the design's posture. Stage 5 (local fan-out, audit §1) already runs on best-effort `try_send` + sync-on-reconnect recovery; making Stage 6 best-effort + pull-on-reconnect is the same shape applied to the Node-to-Node case. Hybrid was chosen precisely because pull-on-gap exists as the recovery mechanism — so leaning on it for peer-down is using the design as intended, not abandoning the peer. A bounded outbound queue (the rejected Option β) would buy a small UX win for very-short outages at a real complexity cost; pull-on-reconnect is a normal flow, not an exception path. A durable outbound queue (the rejected Option γ) would contradict the rest of the system's "Stage X is best-effort, recovery is sync's job" posture.

### 4.6 Sub-decision F-1c — Node-implementation per-peer record

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

**The question.** Given F-1b drops outbound events on peer-down, how does Node A's implementation manage reconnection attempts and operator visibility for unreachable peers?

**Decision.** Each Node maintains a **persistent per-peer record** at the Node-implementation layer (not protocol-visible). The record exists for every peer Node ever known to this Node — peers currently active, peers currently unreachable, peers from past sessions that haven't reconnected. Fields include:

- **Operational state.** Lost-connection flag, last-seen timestamp, last successful session timestamp, next scheduled reconnect attempt.
- **Operator-set custom settings.** Per-peer overrides (priority, retry-cadence overrides, operator notes, future fields not yet defined).

The record is **persisted in the federation registry** (the existing storage surface for federation relationships per audit §2.1 and Ch4 §4.11.2). Whether F-1c adds columns to an existing table or a new sibling table is left to the implementation runbook — Clair's latitude per the M6 precedent, with the criterion *cleaner is better*.

**Reconnect scheduling.** Node A's implementation reads the F-1c record to decide when to attempt outbound reconnection to a peer flagged lost-connection. **Global backoff schedule in v1** (e.g. 15 / 30 / 60 / 120 min capped), with per-peer override as a future enhancement if it surfaces as needed. The reconnect attempt itself constructs an outbound `FederationMessage::Hello` — which means `run_initiating` (today used only by tests and the stress relay per audit §2.2) gains its first production caller in `xgen-node/src/`. No new wire-protocol message is needed; the existing handshake is the reconnection mechanism.

**Admin UI surface.** The F-1c record is the source of truth for any admin-facing display of peer status. "Peer B has been unreachable for 17 minutes, next reconnect attempt in 8 minutes" comes from reading the F-1c record. The exact UI design is out of scope here; this design phase locks that *the record exists and is queryable*.

**Operator capability — opportunistic vs. peer-initiated.** The mechanism is **bilateral**: a peer that comes back online can initiate handshake to its home Node as today, OR the home Node can initiate outbound when its F-1c record says it's time to retry. Either side's success establishes the session. This is genuinely new behaviour — today's federation is peer-initiated only (audit §2.2 zero production callers of `run_initiating` in `xgen-node/src/`) — but it's behaviour at the Node-implementation layer, not new protocol.

**Reasoning recorded.** Layering keeps the protocol simple (Option α has no queue, no retry semantics, no peer-lifecycle wire shape) while letting the Node implementation do operationally useful work the protocol doesn't need to know about. The forward-compat note in conversation — "every node will have to save some mention about other past nodes for some custom settings" — is captured here: F-1c is the canonical home for any "what does this Node remember about that peer Node" question, present or future.

### 4.7 Implementation-runbook notes from F-1

These are not design decisions; they are pieces of context the runbook author should keep in view when writing the Clair-facing task file later:

- The F-1c record likely overlaps with the existing `peer_announcements` table in the federation registry (Ch4 §4.11.2). Schema decision (extend or sibling table) is the runbook's call.
- `run_initiating` gaining its first production caller in `xgen-node/src/` is a meaningful test-coverage delta. The runbook should include integration tests that exercise Node-initiated reconnection.
- The F-1a tip exchange replaces the existing `handle_federation_incoming` history-dump logic. Existing tests that depend on the dump shape may need updates.
- The push-on-Stage-4 hook integrates at or near `app.rs:637` (the existing `apply_fanout` call site). Federation push is a sibling of local fan-out, not a wrapper around it.

---

## 5. Framework decision F-2 — Session model

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

### 5.1 The question

F-1 established hybrid push-with-pull-recovery over "persistent peer sessions." This section pins down what "persistent" means operationally: when the session is open, when it closes, who hosts it, and what happens at session boundaries.

### 5.2 Options considered

**Option 1 — Long-lived continuous session.** Once the handshake succeeds, the WebSocket stays open indefinitely. The transport-layer keepalive (Ch3 §3.3.6: 30s ping, 10s pong timeout) detects deadness. F-1c kicks in when the session is observed lost.

- ✅ Simplest mental model; minimum latency; reuses existing keepalive.
- ⚠️ N peers = N open connections (same shape as any production WS server with persistent clients — rounding noise on Node load).

**Option 2 — Periodic reconciliation, no continuous session.** Brief session every N seconds, push accumulated events, close.

- ❌ Re-introduces polling-latency under "reconciliation window" framing — contradicts F-1 hybrid lock.
- ❌ Re-handshake cost per interval.
- ❌ Effectively pull-with-extra-steps.

**Option 3 — Ephemeral-per-batch.** Session opens when a push is needed, holds briefly, closes after idle window.

- ❌ First-event-in-burst pays full handshake latency — user-visible "messages feel slow when room has been quiet."
- ❌ Connection churn during bursty traffic.

### 5.3 Decision — Option 1 (long-lived continuous session)

**Sessions are persistent for as long as both Nodes are reachable.** Once handshake completes (per F-1a tip exchange), the WebSocket stays open. Push events flow as they are accepted into the home Node's DAG. Health is detected by the existing transport-layer keepalive — no new mechanism.

**Reasoning recorded.** F-1 chose hybrid specifically to avoid the polling-latency floor that Option 2 would re-introduce under a different name. Option 3's first-event-in-burst penalty would surface as user-visible UX slowness in low-traffic Spaces. Option 1 reuses the existing transport keepalive (which exists for client connections and works equally well here) and the "many open connections" cost is the same cost a Node already pays for its client connections — federation peer count is rounding noise relative to client count at any realistic deployment scale.

### 5.4 Sub-decision F-2 lifecycle — Session boundaries

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

**Session opens** on successful handshake completion. The handshake can be initiated by either side: peer-initiated (today's mechanism, audit §2.1) or home-initiated via F-1c reconnect scheduling. The side that wins the handshake race becomes the host of that session; the host role has no semantic consequence beyond owning the lifecycle.

**Session closes** on any of:

| Event | Effect |
|---|---|
| Either side sends `goodbye` (clean shutdown, peer restart, operator action) | Session closes. F-1c lost-connection flag set on the side that observes the goodbye. |
| Either side detects keepalive failure (Ch3 §3.3.6: ping sent, no pong within 10s) | Session is treated as dead. F-1c lost-connection flag set. |
| Either side detects WebSocket-layer error (TCP reset, transport-level failure) | Session closes. F-1c lost-connection flag set. |
| TLS / network partition with no clean signal | Resolved by keepalive timeout (above). No special-case handling. |

**Re-establishment is a fresh session, not a resumption.** When the connection is re-established (either side initiating per F-1c), the receiving side runs a full handshake from scratch including F-1a tip exchange. There is no "resume previous session" concept. Recovery of missed events happens via the F-1a tip exchange delta, not via session-state preservation. This matches the rest of the protocol — sessions are stateless about their own history.

**No mid-session state to preserve.** Because F-1b drops outbound events on push failure (Option α), there is no "in-flight" queue to deal with when the session ends. The session has no obligations to its own events; once a push succeeds or fails it is done.

### 5.5 Sub-decision F-2a — Session topology per federated pair

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

**The question.** A pair of federation peer Nodes typically shares multiple Spaces, with each Space hosted on one of the two Nodes. Events must flow in both directions: Node A's events for Space S1 (which A hosts) flow A→B; Node B's events for Space S2 (which B hosts) flow B→A. Does this two-way flow ride on **one shared WebSocket** between A and B, or **two separate WebSockets** (one A→B, one B→A)?

**Options considered.**

- **(i) One WebSocket per pair, bidirectional event flow.** Both A and B push their respective Space events over the same connection. Whoever wins the handshake race owns the lifecycle.
- **(ii) Two WebSockets per pair, each Node hosts its own outbound.** Symmetric but doubles the connection count.

**Decision — (i) one WebSocket per peer-pair, bidirectional event flow.** Whoever wins the handshake race owns the lifecycle. The host role can shift across re-establishments (Node A's process restarts → its connection closes → Node B re-initiates per F-1c → B is now the host) without any semantic consequence.

**Reasoning recorded.** One connection per pair is simpler to reason about, debug, and observe. Connection count scales linearly with peer count, not multiplicatively. There is no semantic content to "which side hosts the session" — both sides push events for Spaces they host, both sides receive events for Spaces the other hosts, and the validation pipeline on receipt is the same regardless of session host.

### 5.6 Implementation-runbook notes from F-2

- The transport-layer keepalive (Ch3 §3.3.6) is the sole connection-health mechanism. No federation-specific heartbeat is added.
- Session "host" is a runtime concept (who currently owns the WS lifecycle), not a persisted concept. F-1c records the federation relationship and lost-connection flag; it does not record "who was the host last time."
- The push-on-Stage-4 hook needs to look up "is there an active session to this peer for this Space?" — a routing question that is implementation detail. If yes, send over the existing session; if no, push fails per F-1b and the F-1c reconnect schedule handles recovery.
- Both sides must be prepared to receive events over a session they initiated, not just events for Spaces they host. The connection is bidirectional by design.

---

## 6. Framework decision F-3 — Identity authority on the federation channel

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

### 6.1 The question

When Node A pushes event E to Node B over the persistent F-2 session, what *authority* is asserting the push? Put differently: when B's `process_inbound` receives E, what does the receiver-side validation pipeline use as the trust anchor for "yes, this event really comes from where it claims, and yes, this peer is entitled to be relaying it"?

This is the question that protects against a compromised or malicious peer Node injecting events into a receiver's DAG.

### 6.2 The three identity signals available on the federation channel

When event E arrives at B over a federation session, B has access to three identity signals:

1. **Event-level signature** (E.signature). The Identity (e.g. Alice's keypair) signed E. Verifiable if B has Alice's Identity record (her public key) locally.
2. **Federation-session authority** (the WS session itself). Established by the handshake — A authenticated with its Node keypair, so B knows "this session is to Node A" with cryptographic certainty for the lifetime of the session.
3. **Stage-4 acceptance** (implicit). A is asserting "I accepted E into my DAG." This is not a separate signature; it is the act of pushing on the session.

Different combinations of these signals give different authority models.

### 6.3 Options considered

**Option 1 — Trust the event signature only. Federation session is plumbing.**

B verifies E.signature against the author's Identity record. If verification passes, E is accepted. The fact that A sent it over the federation session is operationally useful (it is how E got here) but cryptographically irrelevant.

- ✅ Cryptographically the strongest baseline. Every event independently verifiable by Identity public key.
- ❌ Permits any peer that happens to have valid events to push them. A defederated Node X could push events for Space S to B over an unrelated session, and B would accept them as long as the signatures verified. The event itself is authentic, but X had no business relaying it.

**Option 2 — Trust event signature + require federation relationship for the Space.**

B verifies E.signature against the author's Identity record AND requires that the federation session A sent E over corresponds to a federation relationship established between A and B for the Space E belongs to.

- ✅ Same Identity-signature guarantee as Option 1.
- ✅ Adds federation-relationship gating: B rejects events for Space S from peers that do not have an established federation relationship with B for Space S.
- ✅ Composes two cryptographic and authorisation checks that already exist in the protocol into a coherent ingestion gate.

**Option 3 — Add federation-layer signing on each push.**

Same as Option 2, plus A wraps E in a federation envelope and signs the envelope with A's Node keypair. Each pushed event has two signatures: the event's own and A's federation-push signature on the envelope.

- ❌ Significant overhead. Every pushed event costs an extra signature on both ends.
- ❌ Redundant with TLS. Production federation runs over `wss://`; TLS already prevents session hijack. The federation signature adds nothing TLS does not already provide.
- ❌ Does not address the real attack. The real threat is not TLS hijack; it is the peer Node itself being compromised. A federation signature signed by the compromised Node does not defend against a compromised Node.

### 6.4 Decision — Option 2 (event signature + federation relationship verification)

**B's `process_inbound` ingestion gate for federation-channel events runs two independent checks:**

1. **Event-level signature verification** against the author's Identity record in B's local registry. Same logic as today's `accept_event` step 12 (audit §3.2 Path A) — extended to apply to all event types, not just messages (F-4 closes that asymmetry).
2. **Federation-relationship verification** against B's federation registry: the peer that delivered the event must have an established federation relationship with B for the Space the event belongs to.

If either check fails, the event is rejected per the receive-side rejection policy (F-4 specifies how rejection surfaces to the sender and the local observability layer).

### 6.5 Why two checks are not redundant with session authentication

The federation session is authenticated at handshake time: A signs `federation.hello` with A's Node keypair, B verifies, the session is established with cryptographic certainty about *which Node is at the other end of the WebSocket*. That fact is durable for the lifetime of the session — TLS protects the channel and the original Node-keypair signature established who is there.

A reasonable question: if the session already authenticates A, why does each event need its own check?

**Because the session authenticates a Node, but events are authored by Identities.**

The federation session proves: *this WebSocket is to Node A*. That is a fact about which Node process is at the other end.

The event E proves: *Alice made me*. The event's `sender` field is Alice's pubkey URI; the signature is computed with Alice's private key. The event's authority claim is about an Identity, not about a Node.

These are independent claims. The session proves the first; only the event signature proves the second. Alice can host her Identity on any Node — she might be a member of Space S on Node A today, register a replica on Node C tomorrow, leave A entirely later. The fact that her event arrived via the session to Node A says nothing about whether Alice actually authored it. A is the messenger, not the author.

If the system collapsed the two checks ("session is to A, so trust whatever A says about its members"), three failure modes would surface:

- A compromised Node A could push events claiming to be from any Identity, including ones A does not host.
- A misconfigured Node A could relay events from outside its own member list and B would accept them.
- An Identity's protocol authority would derive transitively from whichever Node is currently relaying their events. An Identity would be only as cryptographically strong as the weakest Node it ever federates through.

The event-level signature exists precisely so the author's authority is independent of the messenger's authority. Alice's event is signed by Alice; if Alice's private key is intact, no compromised peer Node can forge an event from her — even a Node that has a valid federation relationship with the receiver.

The federation-relationship check (the second part of Option 2) answers a different question: *is this Node entitled to be relaying events for this Space at all?* This is an authorisation question, not a cryptographic-authenticity one. A Node X with no federation relationship with B for Space S has no business pushing Space-S events to B, even if those events themselves are cryptographically authentic. The federation registry is consulted to answer this question.

Two questions, two checks, neither substitutable for the other:

| Question | Check | Data source |
|---|---|---|
| Is this event real (did the claimed author make it)? | Event-level signature verification | Identity record in B's local registry |
| Is this Node entitled to relay events for this Space? | Federation-relationship lookup | Federation registry in B's local store |

The session authentication answers neither of these. It answers *which Node process is at the other end of the WebSocket*, which is fresh once per session, not fresh per event.

### 6.6 Dependency on Identity replication

Both Option 1 and Option 2 require B to have the relevant Identity records to verify signatures. If B receives an event from a Space member whose Identity record B does not hold, the signature cannot be verified and the event is rejected.

The existing Identity replication subsystem (audit §2.3, Layer 18 / replica registry) is responsible for getting Identity records to B. This milestone does not change Identity replication, but it does *depend* on it working. The implementation runbook should include a verification step: when a federation event push lands, Identity records for the event's author must already be present, or the event arrival surfaces a sync problem.

This is an ordering constraint that needs verification. The runbook should include integration tests where the receiver does and does not have the relevant Identity record, and confirm the rejection-with-reason path is correctly traversed in the negative case.

### 6.7 Implication for F-4 (validation asymmetry)

Option 2 pre-commits to F-4 fixing the Path B/C asymmetry. There is no way to honour Option 2's first check (event-level signature verification) without lifting `process_inbound` Paths B (membership.join) and C (other state events) to the same signature-verification discipline that Path A (message events via `accept_message`) has today. F-3's lock therefore constrains F-4's design space: F-4 must produce a path where all three event-type families share the same verification pipeline on the receive side. Whether they share a code path or run separate parallel paths is the actual F-4 question.

### 6.8 Implementation-runbook notes from F-3

- The two-check ingestion gate composes existing logic (event-signature verification from `accept_event`, federation-relationship lookup from the federation registry). It is not new cryptographic machinery; it is wiring existing checks into a single gate.
- The federation registry lookup is in the hot path for every federation-received event. The runbook should consider caching or in-memory indexing if profiling shows the lookup is expensive at scale. Phase 1 / Phase 2 scale will not stress this.
- Rejection due to either check failing must produce a clear log line (Node-side observability) and, in coordination with M6 (new) Phase 2 envelope-`event_id` work, a wire-layer signal back to the originating peer so the sender can correlate. The exact form of that signal is M6 Phase 2's design, not this document's; this document only flags that F-3 rejection paths are one of the populating contexts for it.

---

## Pass 2 — remaining framework decisions to surface

The following framework decisions are queued for Pass 2 conversation. None is locked yet.

- **F-4 — Validation asymmetry closure.** How `process_inbound` Paths B and C gain signature and timestamp verification. Same code path as Path A or separate.
- **F-5 — Transitive federation.** Locked-out / locked-in / opt-in (initial spec).
- **F-6 — 500ms quiet-time fallback (audit §4).** Fold in or defer.
- **F-7 — No-pagination on `collect_sync_history` (audit §4).** Fold in or defer.
- **F-8 — Ch4 lines 779/825-827 correction.** Now or at Pass 3.
- **F-9 — `docs/xgen_node_admin_ops_design.md` §4.2 correction.** Now or at Pass 3.
- **F-10 — "DAG hole" semantics when validation fails on a federated event.** What does the receiving Node do when a state event arrives whose predecessors are unknown and whose signature can't be verified due to missing Identity records.

---

*End of document (Pass 2 in progress).*  
