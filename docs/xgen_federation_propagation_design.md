# XGen Federation Event Propagation — Design

> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-21 (Bidirectional `federation_nodes` design phase RECORDED — new §6.4.2 sibling subsection (sibling to §6.4.1 Phase 7.5 cold-start bootstrap) summarises three Joe-locks: Q1 Reading (i) (one event, asymmetric interpretation; D-075 promoted), Shape A (origin-aware applier with new `my_node_id: &str` parameter; wire format unchanged), sub-option A.1 (re-derive on load; native fit verified against `SpaceState` non-persistence model at `xgen-core/src/node/runtime.rs:134`). New §15 row records the four-commit implementation sequence. The bidirectional gap was surfaced during Phase 9 Commit 3a Scenario 1 diagnostic run: B's `apply_federation_add` populated `federation_nodes` with B's own URI instead of A's, causing F-3 to reject every post-bootstrap push event. Wire-format-neutral and Pass-1-neutral. Per D-069 + D-075. Implementation runbook at `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (ACTIVE v1.0) is the next-active artefact for Clair. Previous 2026-05-20 update note (Phase 7.5 milestone CLOSED) stands authoritative in the body below.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools. This document is the canonical deliverable of the Federation Event Propagation milestone's Joe-locked design phase, per the D-069 canonical-document rule.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this document is

This document is the canonical design for the Federation Event Propagation completion milestone. It specifies the mechanism by which an event accepted into one XGen Node's DAG reaches federated peer Nodes — and what guarantees that mechanism provides.

The milestone exists because the Propagation Reliability Audit (J-081, `docs/xgen_propagation_reliability.md`) found that Node-to-Node federation event propagation does not exist as a production mechanism in the current implementation. The federation surface today is one-time history dump on peer-initiated handshake, then connection close. No persistent peer session, no outbound event push, no DAG-tip reconciliation, no gap-recovery mechanism. This document specifies the mechanism that closes that gap.

It is the canonical document for the Federation Event Propagation milestone per the D-069 canonical-document rule. Future edits to the design land here, not in `tasks/` addenda or in DECISIONS.md notes. The implementation runbook (`tasks/FEDERATION_PROPAGATION_COMPLETION.md`) is a runbook against this design, not an alternative design surface.

**This document is partly protocol, partly reference implementation.** The protocol-level additions it specifies — new wire messages, validation rule changes — will land in Chapter 3 in the implementing commit. The Node-implementation pieces (per-peer record, reconnect scheduling, admin UI surfaces) are reference-implementation specification and belong alongside Chapter 4 and the M6 admin design doc, not in Chapter 3. Each section flags which layer it operates at.

### 1.1 Phase 0 design provenance

This document is produced by the milestone's Phase 0 design phase. The phase ran in three passes per the D-069 discipline:

- **Pass 1 — audit current state.** Done. J-081 is the audit; this milestone inherits it. No re-audit.
- **Pass 2 — proposals + Joe-lock markers.** Surface design alternatives with trade-offs; mark every framework decision with `[JOE-LOCK]`; surface decisions one at a time, not as a wall. Pass 2 ran in conversation over 2026-05-18 and produced this document at v0.6 (F-1 through F-6 inline) plus three addenda (F-7 pagination, F-8/F-9 documentation correction timing, F-10 DAG hole semantics). The addendum pattern was a Pass 2 efficiency move: full-file rewrite per F-item became disproportionately expensive once the doc grew past ~70KB.
- **Pass 3 — lock framework decisions + canonical doc.** Walk all `[JOE-LOCK]` markers to final form; consolidate addenda into the canonical document; flip Status to ACTIVE; execute the F-8 and F-9 documentation corrections in the same commit; write the implementation runbook for Clair. Pass 3 closed 2026-05-18 in the same-day session that follows Pass 2.

This document is the Pass 3 canonical artefact. All ten framework decisions are locked. Pass 2 task file (`tasks/FEDERATION_PROPAGATION_DESIGN.md`) is marked COMPLETED in the same commit that ships this v1.0 document; the implementation runbook (`tasks/FEDERATION_PROPAGATION_COMPLETION.md`) is created in the same commit and handed off to Clair as the next-active task.

---

## 2. Background

### 2.1 What the audit established

The audit traced the propagation lifecycle from event submission through to federated-peer delivery and identified, with code-grounded evidence, where the mechanism breaks down. The headline findings:

- **Stage 6 (Node-to-Node federation propagation) is architecturally absent.** Three independent traces converged: (a) the production `xgen-node/src/` codebase contains zero callers of `run_initiating`; (b) no pull mechanism exists Node-to-Node (`space.join_request` is only received in production, never sent); (c) the stress-test "Federation Completeness" check measures only each Node's local-clients delivery, not cross-Node propagation. Audit §2 verdict: GAP IDENTIFIED — severity HIGH.

- **The `process_inbound` validation pipeline applies asymmetrically across event types.** Path A (message events via `accept_message`) runs the full 13-step validation including signature verification, timestamp checks, and HeldPending buffering for unknown predecessors. Paths B (membership.join) and C (other state events) bypass signature and timestamp verification and have no HeldPending integration. Severity LOW today (locally-authenticated submission is the only entry point) but HIGH the moment a federation push channel exists, because federation propagation is the exact vector that would make Paths B/C reachable with unverified events. Audit §3 sub-finding.

- **The existing `transport.sync_request` mechanism has documented gaps that become relevant when the design extends it Node-to-Node.** The spec-defined `sync_response` and `sync_complete` reply shapes (Ch3 §3.3.6) are unimplemented; the client uses a 500ms quiet-time timeout for end-of-stream detection. No pagination on `collect_sync_history`. Unknown-`since` returns silent-empty with no signal back. Audit §4 sub-findings.

- **Documentation drift.** `docs/xgen_node_admin_ops_design.md` §4.2 and `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 describe Node-to-Node federation and `transport.sync_request` mechanisms that do not exist. The audit recorded these for correction in this milestone's documentation pass. Pass 3 of the design phase executes the corrections (F-8 + F-9, §11 + §12 of this document).

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
- **`sync_complete` wire shape implementation (F-6).** Implement the spec-defined `transport.sync_complete` message that today's code uses a 500ms quiet-time fallback for; migrate all four production callers to the explicit signal.
- **Pagination on `collect_sync_history` (F-7).** Implement response-size pagination with `continue_from` cursor; bound response size at the protocol level; pair with `sync_complete` to make catch-up flows predictable across WAN latency.
- **DAG hole prevention (F-10).** Generalise HeldPending to handle the unknown-signer-Identity case so first-contact federation events queue rather than reject-then-re-pull.
- **Documentation correction.** Update `docs/xgen_node_admin_ops_design.md` §4.2 and `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 to describe the deferred-to-this-milestone state in place of the absent mechanism. Performed at Pass 3 (F-8 + F-9).

### 3.2 Out of scope

- **M6 (new) admin verb work.** Blocked behind this milestone's ACTIVE flip; that is its own milestone.
- **Wire-layer rejection signal for event acceptance.** The M6 (new) Phase 2 envelope-level `event_id` work, Joe-locked direct at audit close. Coordinated with this milestone (the validation-pipeline changes here may surface events that need rejection signalling), but its design is in `docs/xgen_node_admin_ops_design.md` §6.5, not here.
- **Transitive federation as a v1 feature.** F-5 locks transitive federation OUT of v1 with a known evolution path to peer-by-peer opt-in (Option 3 in §8.3) in v2 if scaling pressure surfaces.
- **MLS operationalisation.** Independent parallel workstream (D3 in the project roadmap), not affected by this milestone.
- **Compaction / event-store eviction.** No compaction mechanism exists today; this milestone does not add one. The unknown-`since` silent-empty behaviour from audit §4.4 is recorded as a future scaling concern, not solved here.
- **Test plan and runbook.** The implementation runbook (`tasks/FEDERATION_PROPAGATION_COMPLETION.md`) is written at Pass 3 close and is a Clair-facing artefact; the canonical design (this document) is its source of truth.

### 3.3 Non-scope decisions explicitly recorded

These are out-of-scope but are recorded here so that future readers know they were considered and deferred, not forgotten:

| Item | Why deferred | Where it lands |
|---|---|---|
| Compaction-aware sync | No compaction exists; can't design recovery against an absent mechanism | Future scaling milestone |
| Cross-Space topological order | Audit §4 LOW sub-finding; pre-existing M4 carry-over | Existing carry-over, not this milestone |
| Transitive federation (v1 feature) | F-5 locked OUT for v1; v2 evolution path documented | This document §8.6 |

---

## 4. Framework decision F-1 — Federation push direction

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

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

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

**Today's behaviour.** When peer Node B initiates a federation handshake to home Node A, A snapshots Space S's full event history under the runtime lock, streams it in topological order, then closes the connection (audit §2.1). The session is one-shot.

**Decision.** The handshake is reshaped to a **tip exchange**. Peer B sends its current tip per shared Space (or empty if new to the relationship); home A responds with the delta — events that follow B's tip in the DAG, up to A's current tip. For a brand-new relationship where B has no tip, the delta is the full history. For recovery after a short downtime, the delta is small. After delta delivery completes, the session **stays open** as the persistent push channel established by F-1.

**Reasoning recorded.** Tip exchange is symmetric for first-contact and recovery cases — first-contact is just "the biggest possible gap" — and that symmetry fits the hybrid principle (push for steady state, pull for gap recovery, "first contact" is the largest gap). Option B is more consistent with F-1c (both sides maintain per-peer records) because the peer's tip is exactly the kind of per-peer state F-1c specifies. The marginal implementation cost is lower than it first looks because the validation-asymmetry work in F-4 is going to touch the handshake code regardless.

### 4.5 Sub-decision F-1b — Buffering on peer-down: drop, recover via pull

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

**The question.** Home Node A pushes event E to peer Node B. B is unreachable (connection dropped, network partition, B is restarting). What does A do with E?

**Decision.** **Drop the push at the protocol layer. No outbound queue. Recovery is the peer's job via pull on the next successful session establishment.** When B comes back, B's handshake tip exchange (F-1a) picks up everything A has accepted since B was last in sync.

**Reasoning recorded.** Consistent with the rest of the design's posture. Stage 5 (local fan-out, audit §1) already runs on best-effort `try_send` + sync-on-reconnect recovery; making Stage 6 best-effort + pull-on-reconnect is the same shape applied to the Node-to-Node case. Hybrid was chosen precisely because pull-on-gap exists as the recovery mechanism — so leaning on it for peer-down is using the design as intended, not abandoning the peer. A bounded outbound queue (the rejected Option β) would buy a small UX win for very-short outages at a real complexity cost; pull-on-reconnect is a normal flow, not an exception path. A durable outbound queue (the rejected Option γ) would contradict the rest of the system's "Stage X is best-effort, recovery is sync's job" posture.

### 4.6 Sub-decision F-1c — Node-implementation per-peer record

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

**The question.** Given F-1b drops outbound events on peer-down, how does Node A's implementation manage reconnection attempts and operator visibility for unreachable peers?

**Decision.** Each Node maintains a **persistent per-peer record** at the Node-implementation layer (not protocol-visible). The record exists for every peer Node ever known to this Node — peers currently active, peers currently unreachable, peers from past sessions that haven't reconnected. Fields include:

- **Operational state.** Lost-connection flag, last-seen timestamp, last successful session timestamp, next scheduled reconnect attempt.
- **Operator-set custom settings.** Per-peer overrides (priority, retry-cadence overrides, operator notes, future fields not yet defined).

The record is **persisted in the federation registry** (the JSON-backed `FederationRegistry` in `xgen-core/src/federation/registry.rs` per Ch4 §4.11.2). The implementation runbook's Phase 5 Joe-lock A3 placed `peer_records: HashMap<peer_node_id, PeerOperationalRecord>` as a sibling field alongside `relationships` inside the same `FederationRegistry` struct — one JSON file, one save site, type-clean separation. (The pre-Phase-5 audit §2.1 description of the registry as SQLite-backed with a `peer_announcements` table was documentation drift; no such table ever existed in code — see §15 Implementation Complete for the post-milestone shape.)

**Reconnect scheduling.** Node A's implementation reads the F-1c record to decide when to attempt outbound reconnection to a peer flagged lost-connection. **Global backoff schedule in v1** (e.g. 15 / 30 / 60 / 120 min capped), with per-peer override as a future enhancement if it surfaces as needed. The reconnect attempt itself constructs an outbound `FederationMessage::Hello` — which means `run_initiating` (today used only by tests and the stress relay per audit §2.2) gains its first production caller in `xgen-node/src/`. No new wire-protocol message is needed; the existing handshake is the reconnection mechanism.

**Admin UI surface.** The F-1c record is the source of truth for any admin-facing display of peer status. "Peer B has been unreachable for 17 minutes, next reconnect attempt in 8 minutes" comes from reading the F-1c record. The exact UI design is out of scope here; this design phase locks that *the record exists and is queryable*.

**Operator capability — opportunistic vs. peer-initiated.** The mechanism is **bilateral**: a peer that comes back online can initiate handshake to its home Node as today, OR the home Node can initiate outbound when its F-1c record says it's time to retry. Either side's success establishes the session. This is genuinely new behaviour — today's federation is peer-initiated only (audit §2.2 zero production callers of `run_initiating` in `xgen-node/src/`) — but it's behaviour at the Node-implementation layer, not new protocol.

**Reasoning recorded.** Layering keeps the protocol simple (Option α has no queue, no retry semantics, no peer-lifecycle wire shape) while letting the Node implementation do operationally useful work the protocol doesn't need to know about. The forward-compat note in conversation — "every node will have to save some mention about other past nodes for some custom settings" — is captured here: F-1c is the canonical home for any "what does this Node remember about that peer Node" question, present or future.

### 4.7 Implementation-runbook notes from F-1

These are not design decisions; they are pieces of context the runbook author should keep in view when writing the Clair-facing task file later:

- The F-1c record sits alongside the existing federation-relationship record in the JSON-backed `FederationRegistry` (Ch4 §4.11.2). Schema decision (extend `FederationRelationship` directly, sibling type in a separate JSON file, or sibling field inside `FederationRegistry`) was the runbook's call — Phase 5 Joe-lock A3 picked the sibling-field-inside shape.
- `run_initiating` gaining its first production caller in `xgen-node/src/` is a meaningful test-coverage delta. The runbook should include integration tests that exercise Node-initiated reconnection.
- The F-1a tip exchange replaces the existing `handle_federation_incoming` history-dump logic. Existing tests that depend on the dump shape may need updates.
- The push-on-Stage-4 hook integrates at or near `app.rs:637` (the existing `apply_fanout` call site). Federation push is a sibling of local fan-out, not a wrapper around it.

---

## 5. Framework decision F-2 — Session model

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

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

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

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

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

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

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

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

**Data source for the relationship check (Phase 7 Lock A1, runbook §3.7.1).** The "federation registry" phrasing used throughout this section is shorthand for **`SpaceState.federation_nodes`** — the per-Space list of federated peer Node IDs built up by `state.federation_add` events under the a-i symmetry rule (F-1a). This is the single source of truth for "is peer X federated with us for Space S?" The standalone `FederationRegistry` (per-peer protocol-level + F-1c operational state) is a different store; it is NOT consulted on the per-event hot path. Phase 4 Q2 locked this for the symmetric outbound push decision (`apply_federation_push`); Phase 7 confirmed the same source for the inbound F-3 check. The two reads must agree, or an update race between handshake-time refresh and event-time `state.federation_add` ingestion would produce a system that pushes events but rejects them on receipt.

**B's `process_inbound` ingestion gate for federation-channel events runs two independent checks:**

1. **Event-level signature verification** against the author's Identity record in B's local registry. Same logic as today's `accept_event` step 12 (audit §3.2 Path A) — extended to apply to all event types, not just messages (F-4 closes that asymmetry).
2. **Federation-relationship verification** against `SpaceState.federation_nodes` for the event's target Space: the peer that delivered the event must appear in the federation_nodes list.

If either check fails, the event is rejected per the receive-side rejection policy (F-4 specifies how rejection surfaces to the sender and the local observability layer).

**Implementation note (Phase 7 Lock B1, runbook §3.7.1).** `state.federation_add` events arriving over a federation session bypass the F-3 relationship check. The federation_add event IS the relationship-establishing event itself: at dispatch time the sender Node is by definition NOT yet in `federation_nodes` for the target Space (that's what the event will add once ingested). Without this skip, federation could never bootstrap. The event's authority is intrinsic — the session-level handshake auth (peer Node-keypair) and the event-level signature (same keypair) cover the relevant authority claims. The skip is **NOT** narrowed to "sender == wire-authenticated peer == federation_add.adds_node" — that is the B2 alternative explicitly rejected at v1 (the threat model doesn't justify the additional check at this layer). If a future threat model justifies tightening, B2 layers on top of B1 cleanly.

**Implementation note (Phase 7 Lock B3, locked 2026-05-20, sibling to B1).** B1 covered F-3 only. Downstream validation-core gates also presuppose Space membership — step 9 (predecessor presence), step 11 (sender registration in IdentityRegistry, plus sender Space membership), and step 13 (sender permission). For `state.federation_add` arriving via federation channel these gates fail for the same structural reason F-3 fails: federation_add is Node-authored under Phase 4 §3.4.1 Q3 overload (sender = Node URI, not Identity URI), Node URIs are not Space members, and `IdentityRegistry` has no production path that inserts Node URIs. B3 widens the skip set: `state.federation_add` with `peer_node_id.is_some()` at `dispatch_event` skips step 9 + step 11 (both halves) + step 13. Step 8 (event_id hash), step 10 (DAG structure), and step 12 (signature verification) are preserved — the signer's pubkey is encoded in the sender URI and step 12's `verify_event_signature` decodes it directly via pure crypto (no registry lookup, Q3-overload transparent at this layer). Without B3, the bootstrap stream described in §6.4.1's Phase 7.5 framing dead-locks: federation_add HeldPending on missing predecessor (predecessor-chain deadlock — its predecessors are the very events held on the federation-relationship trigger), or alternatively HeldPending on F-10 Identity trigger (Node-URI Identity never replicates). Full proposal at `tasks/FEDERATION_PROPAGATION_PHASE_7_B3_AMENDMENT.md` with captured production-gap evidence at §3 of that document. As with B1, the v2-tightening path B2 (sender-wire-peer match) layers cleanly on top of B3 if a future threat model justifies; B3 does not preempt that scope. Narrow to federation channel: locally-submitted federation_add (M6 admin write path, future) retains full validation per the amendment §4.2.

### 6.4.1 Phase 7.5 — Cold-Start Bootstrap (sibling locks to B1)

Phase 7.5 closes a cold-start bootstrap chicken-and-egg that surfaced during Phase 9 Scenario 1 setup (J-093). When a brand-new Node B initiates federation handshake to Node A for Space S, the F-1a delta stream delivers `state.space_create` first — but B has no local record of S yet, so `SpaceState.federation_nodes[S]` returns None and F-3 rejects. Every subsequent event in the stream then fails F-4 step 1 ("space not found"). Even `state.federation_add`, which Phase 7's B1 skip rule lets through F-3, fails F-4 step 1 for the same "space not found" reason. Net: brand-new federation peers cannot bootstrap a Space — the cold-start path the Phase 1-7 design assumed works does not work end-to-end. Phase 7.5 is the formal closure of this gap. Full design lives at `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` (COMPLETED v1.0); this subsection records the four locks at the canonical-design depth alongside Phase 7's B1 + B2 framing.

**P7.5-A — Narrow skip rule extension for Space-create EventTypes** `[JOE-LOCK: locked 2026-05-19]`. The B1 skip pattern (Phase 7) is extended to cover `state.space_create` and `state.dm_space_create` at both F-4 step 1 ("Space exists locally") and F-3 (federation-relationship verification). Authority is preserved via event-level signature verification through F-10's HeldPending for unknown-signer cases; only the structural Space-existence and federation-relationship checks are skipped. The skip is narrow — it does NOT extend to `state.room_create` (which creates a Room nested under a Space; if the Space doesn't exist locally, room_create SHOULD be rejected) or to other DAG-root EventTypes. The discriminator is "creates the Space it references", not "DAG root". DoS-surface implication: any peer with an active federation session can introduce a new Space on the receiver's local store; this surface is bounded by federation peers being operator-authorised (not anonymous), content-determinism preventing identifier collisions with existing Spaces, and the new `SpaceLocalMetadata` sibling structure in `xgen-common` carrying `introducer_node_id: Option<String>` (populated once at Space-create ingestion via federation, never modified afterward, persisted to a dedicated local SQLite table). The field name `introducer_node_id` is retained through any future XGID-typing pass — the field name carries the role, the type carries the contract. `SpaceLocalMetadata` is a sibling to `SpaceState` rather than a field on it, preserving SpaceState's invariant that all its content is derived from federated events.

**P7.5-B — Third HeldPending trigger condition for missing federation relationship** `[JOE-LOCK: locked 2026-05-19]`. After P7.5-A lets `state.space_create` land, subsequent events in the bootstrap stream (`state.room_create`, `membership.invite`, `state.federation_add`, `membership.join`, `message.*`) pass F-4 step 1 (Space now exists) but fail F-3 because the peer is not yet in `SpaceState.federation_nodes` for the new Space. These events cannot be naively rejected (legitimate bootstrap content in topological order) nor bypass F-3 (weakens trust model exactly during first contact). The HeldPending data structure gains a third trigger condition: "missing federation relationship for (peer, space)", resolved by an idempotent `state.federation_add` arrival hook (fires on every successful ingestion, not only the first — mirrors F-10's Identity-arrival hook semantics, no "already-drained pairs" tracking required). New error code `4007 federation_relationship_timeout` for the case where the federation_add never arrives within the timeout window. Held-not-bypassed posture: events sit in HeldPending until F-3's data source is populated by federation_add arrival, then re-validate cleanly through the unified validation core. F-3 is not skipped; it is deferred until its data source is populated. Combination semantics with F-4a (predecessor) and F-10 (Identity) via existing struct-variant Option fields — an event missing multiple dependencies has multiple fields populated; resolution requires all arrivals. Two-stage cascade case (where `state.federation_add` itself enters HeldPending on F-10's Identity trigger) resolves naturally without special handling: Identity arrival fires F-10's hook, federation_add re-validates and ingests, federation_add ingestion fires P7.5-B's hook, dependent events drain. Each hook responds to its own trigger; no cross-hook coordination is needed.

**Precedence at timeout (P7.5-B sub-rule extending F-10's predecessor-code-wins)** `[JOE-LOCK: locked 2026-05-19]`. If a HeldPending entry times out with multiple missing dependencies, the emitted error code follows the precedence: predecessor (`4002`) > federation-relationship (`4007`) > Identity (`4006`). Rationale: federation-relationship is the most upstream blocker in the dependency chain because Identity replication is conditionally downstream of federation establishment (Identity events themselves flow over federation transport). Reporting the most upstream blocker directs the operator to the right diagnostic question. Verbatim code-comment block at the timeout-emit site, sibling to Phase 6's block at the same site.

**P7.5-C — Per-trigger HeldPending timeout** `[JOE-LOCK: locked 2026-05-19]`. F-4a's predecessor trigger and F-10a's Identity trigger remain at 30 seconds each. The federation-relationship trigger defaults to **180 seconds** with new config field `[sync].federation_relationship_timeout_seconds`. This is the v2 evolution path F-10a forecasted, brought forward to v1 by Phase 7.5's introduction of a third trigger with materially different timing characteristics: bootstrap streams can be large (a Space with months of history may take tens of seconds to deliver across realistic WAN latency, especially with F-7 pagination at 1000 events per batch), and `state.federation_add` arrival is bounded by stream delivery rather than an independent async pipeline (unlike F-10's Identity-replication trigger, which waits for an unrelated async system). The 180-second default is generous but bounded — a bootstrap stream that hasn't delivered `state.federation_add` within 180 seconds either hit a real failure (sender crashed, session dropped) or is delivering a multi-tens-of-thousands-event Space history at slow throughput. Both are served by a configurable default. The default was raised from a draft 120s during the Joe-lock walkthrough to give meaningful headroom over the medium-WAN-degraded case before operators need to discover and tune the config field; practical experience may revise this value in a future tuning pass.

**P7.5-D — Observability for the new HeldPending trigger** `[JOE-LOCK: locked 2026-05-19]`. New `pending_federation_relationship: usize` counter on `NodeState` (in `xgen-common/src/state.rs`), populated by `build_node_state` summing each Space's HeldPending count for the federation-relationship trigger condition. `#[serde(default)]` for forward-compat with pre-Phase-7.5 state files. Sibling to Phase 6's `pending_identity_replication`. The existing `f3_reject` trace event (shipped J-092 Commit 1 per Phase 9 observability preconditions) is **retained** — not renamed — and gains a new `disposition` field with value `rejected` (the dominant non-bootstrap case, F-3 fails permanently) or `held_pending` (Phase 7.5's narrow new path, F-3 defers via HeldPending). Three reasons for retention over rename: (1) Phase 9 Commits 1+2 are already shipped, renaming would be follow-up code change touching trace plumbing; (2) the name `f3_reject` is still accurate for the vast majority of fires — the held-pending case is the narrow new path, not the dominant case; (3) "reject" in trace-event vocabulary often means "did not accept on first try" rather than "permanently refused", and the disposition field clarifies which variant. The `introducer_node_id` field introduced in P7.5-A's `SpaceLocalMetadata` is **NOT exposed in the state file** — the state file is reserved for high-level health counters; per-Space details are queryable via direct SQL until M6 (new) admin work provides an operator CLI verb.

**Out-of-scope decisions explicitly recorded for Phase 7.5.** Sender-side stream reordering (Option Y from design conversation — minting `state.federation_add` with `prev_events = [state.space_create.event_id]` so it lands as a structural sibling of the Space root) was rejected: introduces multi-tip-per-Space DAGs as a normal feature, propagates implications through F-1a / F-6a wire shapes, opens "DAG-root-referencing events" as a precedent pattern, non-reversible (federation_add events with non-tip `prev_events` would persist in archives). Session-flag bootstrap window (Option X.b from design conversation — tracking per-(peer, space) handshake-in-progress state and bypassing F-3 during the window) was rejected: weakens F-3 to pre-audit semantics during exactly the moment trust most matters (first contact), couples handshake protocol to receiver-side trust state in a load-bearing way. Phase 7.5 uses the held-not-bypassed posture (P7.5-B) instead, which preserves F-3's enforcement for every event flow except the two structurally-special EventTypes (P7.5-A) and defers F-3 (rather than weakening it) for the events that arrive before its data source is populated.

### 6.4.2 Bidirectional `federation_nodes` — vantage-aware applier (sibling to B1)

Phase 7.5's locks closed the cold-start bootstrap chicken-and-egg by letting `state.space_create` land structurally (P7.5-A) and HeldPending-buffering subsequent events until `state.federation_add` arrived (P7.5-B). Phase 9 Commit 3a's Scenario 1 ("two-Node push smoke") — the first integration test in the project to spawn two real `NodeRuntime` instances and drive a real federation bootstrap end-to-end — surfaced a second gap one layer down: after `state.federation_add` lands on the receiver, the receiver's `SpaceState.federation_nodes` ends up containing the wrong Node. The applier `apply_federation_add` read `content.node_id` verbatim from every vantage; the receiver (B) populated its `federation_nodes` with B's own URI instead of the asserter's (A's). F-3 then rejected every post-bootstrap A→B push event because the wire-authenticated peer was not in `federation_nodes`. The bidirectional `federation_nodes` design phase closed this gap with three Joe-locks recorded at `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` (COMPLETED v1.0).

**Q1 Reading (i) — one event, asymmetric interpretation** `[JOE-LOCK: locked 2026-05-21]`. `state.federation_add` is one event recording one party's act: "A declares that A federates with B for Space S." The event has one signer (the asserter), one signature, one DAG slot, one logical assertion. The asymmetry between `event.sender` (the asserter) and `event.content.node_id` (the other party) is the event's structure, not a redundancy. `SpaceState.federation_nodes[S]` is a derived projection with a vantage-aware derivation rule: for each `state.federation_add` event E, the local entry adds the **other party** — `event.sender` if I am `event.content.node_id`, else `event.content.node_id`. A and B's `federation_nodes[S]` end up as mirrors as a *consequence* of correct application, not as a structural property of the event store. The rejected Reading (ii) — two events per relationship, one per direction — was preserved in the design walkthrough's reasoning for transparency, but rejected on precedent grounds (would have been the first event family in the registry requiring two signed assertions of the same type) and operational-complexity grounds (intermediate "half-federated" states, replay edge cases, reciprocal-mint timing concerns). Q1's lock is the protocol-event-layer principle promoted to DECISIONS.md as D-075.

**Shape A — origin-aware applier** `[JOE-LOCK: locked 2026-05-21]`. The applier `apply_federation_add` in `xgen-core/src/space/state.rs` gains a `my_node_id: &str` parameter and derives the relevant peer with vantage-awareness: if `content.node_id == my_node_id`, the relevant peer to add is `event.sender` (the asserter); else, the relevant peer is `content.node_id`. Both branches are needed because A's own DAG also contains the `state.federation_add` event A authored — A applies it through its own applier and falls into the `else` branch (content.node_id = B, not A's own ID), adding B. B receives the same event via federation, applies it, and falls into the `if` branch (content.node_id = B = its own ID), adding `event.sender = A`. Symmetric outcomes through asymmetric branches, driven by vantage. **Wire format unchanged.** Event construction unchanged. The `SpaceState::apply_event` dispatch threads `my_node_id` to `apply_federation_add` only; other arms ignore the parameter (other appliers do not currently require vantage-awareness per D-075's on-demand scope). Rejected alternatives: Shape B (reciprocal mint on ingest — introduces "ingesting causes new event authoring" pattern absent elsewhere in the protocol), Shape C (two events at handshake — structurally a Reading (ii) shape regardless of prev_events sub-option), Shape D (wire-format extension — D.1 worst-of-both-worlds duplication, D.2 secretly re-introduces Reading (ii) thinking and pays the only Pass-1 cost in the option space). Shape A is wire-format-neutral and Pass-1-neutral.

**Sub-option A.1 — re-derive on load** `[JOE-LOCK: locked 2026-05-21]`. `SpaceState` is fully derived from the event store at every NodeRuntime construction — there is no persisted `federation_nodes` field to migrate, no fixup pass to run, no version marker to add. Verification performed at design close against `xgen-core/src/node/runtime.rs`: `NodeRuntime::new(keypair)` initialises `spaces: HashMap::new()` (line 134); companion fields at the same struct level carry sibling comments "Not persisted; rebuilt on restart" (Phase 2 simplification). `SpaceState` is constructed via `SpaceState::from_space_create` or `SpaceState::from_dm_space_create` when the originating creation event lands; subsequent state events are applied via `SpaceState::apply_event`. The event store is the source of truth; `SpaceState` is the runtime cache. The fix lands and self-heals on next Node start. No migration code, no version marker, no fixup pass, no schema change. A.1 is not a deviation from the existing model; it IS the existing model applied to a bug whose fix happens to need it. Sub-options A.2 (introduce `EventOrigin::ReplayedFromDisk`) and A.3 (sender-equality heuristic at replay) were considered and rejected as a consequence of A.1's verification result: there is no replay-from-disk surface for `SpaceState`, so a replay-discriminator construct has nothing to discriminate against.

**Code surfaces touched in implementation Commit 2.** Three files: `xgen-core/src/space/state.rs` (`apply_federation_add` signature + body; `SpaceState::apply_event` dispatch; six suggested unit tests at unit level), `xgen-core/src/node/runtime.rs` (two `ingest_event` call sites pass `&self.node_id` through). The Phase 9 Scenario 1 resurrection at implementation Commit 3 — `#[ignore]` lifted from `xgen-node/src/tests/phase9_two_node_smoke.rs::scenario_1_two_node_push_smoke` — is the activating regression lock at integration level (the unit tests are the regression lock at unit level).

**Cross-references.** D-075 (the protocol-design principle this phase instantiates; sibling-distinct from D-070's transport-layer "two events" principle); `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` (locks + rejected-alternative reasoning); `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` (code-grounded mechanism evidence at file:line granularity); `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (four-commit Clair runbook).

**What this phase does NOT change.** Wire format unchanged (no new fields on `state.federation_add` content; no schema migration). `EventOrigin` enum unchanged. `state.federation_add` content schema unchanged. Existing federation_add events on disk (test fixtures, dev-build state files) stay valid as-authored — the bug was in the applier reading them, not in the events themselves. Pass-1-neutral: the `my_node_id: &str` parameter at Shape A is a String reference; when Pass 3 widens dispatch to XGID flavours, the parameter widens naturally. No coordination flag with `tasks/XGID_RETROFIT_PASS_1_IMPL.md` needed.

### 6.4.3 Topological-sort wire-order determinism (sibling to §6.4.2)

Phase 7.5's locks (§6.4.1) closed the cold-start bootstrap chicken-and-egg by letting `state.space_create` land structurally and HeldPending-buffering subsequent events until `state.federation_add` arrived. Bidirectional `federation_nodes` (§6.4.2) closed the receiver-vantage applier bug that surfaced on top of that. Phase 9 Commit 4's verification of the bidirectional fix surfaced a *third* layer one step further upstream: the federation-delta wire ordering itself is non-deterministic across senders with identical Space state. `topological_sort_events` in `xgen-node/src/fanout.rs:193` preserves input-vector order when tie-breaking ready siblings (events with all predecessors already emitted, including DAG roots with empty `prev_events`). Its caller `compute_federation_delta_for_space` at `xgen-node/src/fanout.rs:321` feeds it via `store.values().cloned().collect()` — `EventStore` is `HashMap<String, Event>` with randomized iteration per instance. Two `xgen-node` processes with identical Space state produce different federation-delta wire orderings ~50% of runs. When `state.room_create` (DAG root, empty `prev_events`) wins the race against `state.space_create` (also DAG root), B's `dispatch_event` Step 1 rejects with "space not found"; cascading rejections produce 2 Accepted / 2 Rejected / 101 HeldPending against the passing-run baseline of 102 Accepted / 3 HeldPending. Full code-grounded evidence at `tasks/FEDERATION_TOPOSORT_AUDIT.md` (Status COMPLETED v1.0) §3 with file:line references; design walkthrough at `tasks/FEDERATION_TOPOSORT_DESIGN.md` (Status COMPLETED v1.0) closed this gap with three Joe-locks.

**Q3.ii — Canonical wire ordering required** `[JOE-LOCK: locked 2026-05-22]`. Wire-order determinism is a sender-side normative property for Node-to-Node federation, not implementation latitude. Two senders with identical Space history MUST produce byte-identical federation deltas (modulo signature-bearing fields that vary by author and time). The rejected alternative (Q3.i — each receiver runs its own deterministic topological sort; wire ordering is implementation latitude; the receiver's local DAG is the protocol's contract) was rejected for four reasons in order of weight: wire-format analogue of the no-drift-surface discipline family (D-067 + D-070 + D-075 align — locking explicitly rather than trusting emergence from local primitives); MLS coupling (Ch3 §3.10 + D3 parallel workstream require canonical wire ordering at the application layer; locking Q3.i would surface as late-stage MLS discovery exactly the shape D-071 was created to prevent); cross-Node debugging benefit is immediate ("do these two senders' deltas match byte-for-byte" becomes a yes/no operator-level question available today); catalogue alignment (the audit's failure-mode-row naming already implicitly assumed Q3.ii framing). Q3 was walked first because it was load-bearing for shape admissibility — under Q3.ii, Shapes B (timestamp sort) and D.2 (`IndexMap` insertion order) are structurally disqualified (non-canonical across senders). Q3.ii is the principle the topological-sort phase promotes to DECISIONS.md as **D-076**.

**Q2 middle + Q2.γ — fix the primitive's contract once with explicit forward-binding** `[JOE-LOCK: locked 2026-05-22]`. Fix `topological_sort_events` so it produces canonical output regardless of input ordering — the primitive's contract changes from "respects causality" to "canonical, given a fixed event set". Plus fix the sibling Site 1 at `compute_federation_delta_for_space:321` so federation delta is Q3.ii-compliant end-to-end. The Q2.γ forward-binding flags two Node-to-Client sender-output sites as Q3.ii-analogues that the principle applies to but that don't get fixed in this milestone: `collect_sync_history` (client-to-Node sync_request flow; same `HashMap.values()` feed pattern) and `apply_fanout` history-push (Node-to-Client history delivery; same pattern). Both await their own consumer-pressure framing in a future scheduling pass; the forward-binding language ensures the principle is inherited at that future site's design phase rather than re-litigated. The rejected alternatives were Q2 narrow (federation-only with no forward-binding language — discipline-pattern-consistency grounds; D-067 + D-070 + D-075 all forward-bound siblings explicitly) and Q2 wide (codebase-wide `HashMap` iteration audit — milestone-scoping grounds; this milestone is a targeted fix, not a sweep).

**Q1 Shape A v1 + sibling Site 1 fix** `[JOE-LOCK: locked 2026-05-22]`. Tie-break source: event_id lexicographic sort at `topological_sort_events`, applied to ready siblings at each iteration of the outer loop. Sibling Site 1 fix: sort the `Vec<Event>` at `compute_federation_delta_for_space:321` before passing to the primitive — belt-and-braces explicit canonical-ordering chain end-to-end matching Q2 middle's letter ("primitive fixed + feed canonical"). Pass-1 posture: v1 `&str` sort with code-comment block at the sort site flagging Pass 3 retype to `EventXgid` when xgen-node-side dispatch widens to XGID flavours; Pass-1-neutral, preserves `tasks/XGID_RETROFIT_PASS_1_IMPL.md` Status ACTIVE v2.0 unchanged. The mandatory code-comment block at the sort site (verbatim shape at `tasks/FEDERATION_TOPOSORT_DESIGN.md` §5.3) cites D-076 + Appendix J's content-hash framing: event_id is content-hash-derived per Appendix J, so the lex sort key is byte-stable across senders with identical Space history, which is exactly what D-076's "two senders with identical state produce byte-identical federation deltas" contract obligates. Rejected alternatives: Shape A v2 (typed `EventXgid` from outset — Pass-1 coupling grounds; the wrap-or-comment precedent from XGID Adoption v1 Commit 2 and bidirectional Commit 2 establishes "ship &str at v1, retype under Pass 3 with code-comment marker"); Shape C (canonical-event-bytes sort — Shape A's ordering ≈ Shape C's for distinct event_ids; Shape C's additional benefit concentrated in a hypothetical duplicate-event_id edge case the protocol does not currently emit, paying cross-crate dep + per-comparison serialisation cost for hypothetical benefit); Shape D.1 (BTreeMap at EventStore — milestone-scoping; D.1's right home is a separate `EventStore` canonical-iteration discipline milestone if ever scheduled); Shape D.1 + Shape A (belt-and-braces along two redundant dimensions — bidirectional-precedent scoping discipline argues against bundling).

**Code surfaces touched in implementation Commit 2.** One file with two named edits: `xgen-node/src/fanout.rs::topological_sort_events` (lines 193-220 — adds the primitive `events.sort_by(|a, b| a.event_id.cmp(&b.event_id))` line at the top of each outer-loop iteration plus the verbatim code-comment block) and `xgen-node/src/fanout.rs::compute_federation_delta_for_space` (line ~321 — adds `all.sort_by(|a, b| a.event_id.cmp(&b.event_id))` after the `HashMap.values().cloned().collect()` before passing to `topological_sort_events`). Three-to-five unit tests land at the same commit covering deterministic output across input permutations, stable tie-break for ready siblings with empty `prev_events`, no-op-equivalence for already-canonically-ordered input, and the wire-order-determinism witness — `compute_federation_delta_byte_identical_across_two_senders` — that exercises D-076's full contract at unit level (sibling-in-shape to bidirectional's `apply_federation_add_two_vantages_mirror`). The Phase 9 Scenario 1 second `#[ignore]` lift at implementation Commit 3 — `#[ignore]` lifted from `xgen-node/src/tests/phase9_two_node_smoke.rs::two_node_federation_push_smoke_100_messages` — is the activating regression lock at integration level for both D-075 (bidirectional vantage-aware applier) and D-076 (topological-sort wire-order determinism). The unit tests above are the regression lock at unit level for D-076.

**Cross-references.** D-076 (the protocol-design principle this phase instantiates; fourth member of the no-drift-surface discipline family, sibling-distinct from D-067 at the code-organisation layer + D-075 at the event-model layer, pairing with D-070 at the transport layer); `tasks/FEDERATION_TOPOSORT_DESIGN.md` (locks + rejected-alternative reasoning); `tasks/FEDERATION_TOPOSORT_AUDIT.md` (code-grounded mechanism evidence at file:line granularity; the `compute_federation_delta_for_space:321` HashMap-feed and `topological_sort_events:193` primitive form Sites 1 and 2 of the compounding bug); `tasks/FEDERATION_TOPOSORT_IMPL.md` (four-commit Clair runbook). Canonical sibling sort precedent at `xgen-core/src/node/runtime.rs::topological_sort` (lines 859-912) uses Kahn's algorithm with explicit `queue_vec.sort()` for stable tie-breaking — the design phase deliberately did NOT consolidate the two implementations; the drift surface between them was the D-067 instance D-076 generalises, but the consolidation itself is a separate D-067-flavoured audit phase if ever scheduled.

**What this phase does NOT change.** Wire format unchanged. `Event` struct unchanged. `EventStore` container type unchanged (`HashMap<String, Event>` retained — Shape D.1's BTreeMap change was rejected on milestone-scoping grounds). `state.federation_add` content schema unchanged. The fix is purely sender-side serialisation discipline; receivers observe a more canonical wire order but their dispatch logic is untouched. Existing federation deltas on disk (test fixtures, dev-build state files) stay valid — they were never persisted in a wire-canonical form to begin with; the fix produces canonical wire output going forward. Pass-1-neutral: `tasks/XGID_RETROFIT_PASS_1_IMPL.md` (Status ACTIVE v2.0) is unaffected. The two known Q3.ii-analogue sites (`collect_sync_history` + `apply_fanout` history-push) remain Q2.γ-flagged for future scheduling; this milestone does not touch them. D-076 itself is not promoted to a spec-normative Ch3 statement in this milestone — DECISIONS.md is the canonical lock; spec-level promotion is separate doc-pass work if a future contributor needs spec-level reference.

### 6.4.4 Persistence amendment (drain-without-persist gap closure, sibling to §6.4.3)

The bidirectional vantage-aware applier (§6.4.2) and topological-sort wire-order determinism (§6.4.3) closed two layers of cold-start federation regressions. Phase 9 Commit 3b-1's Scenario 3 (drop-and-recover with relationship state) implementation at J-104 surfaced a third regression one layer further upstream of both: relationship-state events drained out of HeldPending by `dispatch_event`'s three drain helpers (`drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship` in `xgen-core/src/node/runtime.rs`) were re-dispatched via internal recursive `dispatch_event` calls whose Accepted outcomes were silently dropped via `let _ = self.dispatch_event(...)`; `xgen-node::app::process_inbound` persisted only the explicitly-passed event, never the drained ones. On Node restart, `replay_spaces_from_dir` at `xgen-node/src/app.rs:2628` saw only the persisted events; the un-persisted released events were unreplayable; in-flight federation-relationship state was lost across restart. The gap is structurally distinct from the drift surfaces §6.4.2 and §6.4.3 closed — those were applier-correctness and wire-format-determinism layers; this one is a persistence-contract layer between dispatch's drain hook and the storage-write site. The secondary silent-error surface at `xgen-core/src/node/runtime.rs:181` (`graph.add_event` returning `UnknownPrevEvent` swallowed via `let _ =`) compounds the primary gap by hiding any DAG-corruption signal even when the persist site is fixed. Full audit at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` (COMPLETED v1.1) §3 + §4; design walkthrough at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` (COMPLETED v1.1 at this Commit 1) closed the gap with four Joe-locks across Q1→Q4 with no re-walk firing (recorded explicitly per D-065 honest-framing: the non-firing of a re-walk under Lock #2 discipline is itself a load-bearing record).

**Q1 — `ingest_event` Result-handling + sort-on-replay (a.ii + a.iii.β + candidate D-NNN flag)** `[JOE-LOCK: locked 2026-05-23]`. The `graph.add_event` silent-discard at `xgen-core/src/node/runtime.rs:181` served two production-call masters: `dispatch_event` (where `validate_event` Step 9 already guaranteed predecessor presence; an `UnknownPrevEvent` here is a load-bearing bug, not an expected case) and `replay_spaces_from_dir` (where on-disk store-iteration order does NOT respect the DAG; predecessor-missing can fire legitimately under HashMap iteration randomization). Q1's lock answers both callers with sibling-distinct mechanisms appropriate to each call site's context: `ingest_event` signature changes to `pub fn ingest_event(&mut self, event: Event) -> Result<(), GraphError>` (a.iii.β, type-level Result-propagation forcing compile-time caller-handling at every call site), AND `replay_spaces_from_dir` sorts events topologically via xgen-core's `topological_sort` re-export before passing each to `ingest_event` (a.ii, defensive layer preventing the legitimate replay case from triggering the error). The mid-walk sustainability question — "is type-level future-proof?" — surfaced three drift surfaces log-level vigilance does NOT catch (future caller bypasses validate_event without noticing; disk format changes ordering guarantees; future async-predecessor protocol revision) and forced the lock from (a).iii.α (log-level `tracing::error!`) to (a).iii.β. A second-order question — "is (a).iii.β the future-proof solution?" — forced honesty that nothing is future-proof in absolute terms; rungs above (a).iii.β (ValidatedEvent wrapper compiler-forcing the correct path via type-constructor discipline; sealed traits + visitor pattern constraining new-caller shape; formal verification with machine-checked invariants) were named explicitly and flagged as **candidate D-NNN "ingest path invariant encoding"** for future walk at design doc §8 — not promoted to DECISIONS.md at this milestone, kept available via canonical-record pointer. Sibling-shape to D-076's v1 → v1.1 amendment lifecycle (v1 surfaced; gap surfaced at implementation; amendment named the second load-bearing property explicitly) — same pattern at the meta-level for layered protocol-invariant fixes.

**Q2 — `DispatchOutcome::Accepted` return-vector aggregation** `[JOE-LOCK: locked 2026-05-23]`. The Accepted variant gains an `additional_persisted: Vec<Event>` field. `dispatch_event` aggregates the drained-event vectors returned by the three drain helpers (per Q3) at three drain call sites and emits them in the Accepted outcome. `process_inbound` at `xgen-node/src/app.rs` persists the explicitly-passed initial event (existing behaviour) AND iterates `additional_persisted` for the drained events (new persist-loop block immediately after the existing initial-event persist call) with a sibling block in `handle_identity_replicate_msg` for the Identity-arrival drain path. Layer separation preserved — xgen-core stays I/O-free; the persist authority remains `xgen-node`'s storage-write site, not the in-memory dispatch outcome. Pairs cleanly with Q1's type-chain (the Result-propagation surface from Q1 composes with the return-vector surface from Q2 without contradiction; the no-re-walk-fired outcome at design close traces to this composition stability). Rejected alternatives: Q2(b) call-back into `process_inbound` from the drain helpers (rejected on xgen-core-I/O-free-discipline grounds; calling back through the layer boundary would have inverted the existing Phase 6 / Phase 7.5 Identity-hook architecture); Q2(c) explicit persist-vector parameter threaded through the recursion (rejected on signature-complexity grounds; Shape β2 return-vector is the self-documenting honest shape).

**Q3 — all three drain helpers return `Vec<Event>`** `[JOE-LOCK: locked 2026-05-23]`. The same gap pattern surfaced at all three drain helper sites: `drain_pending_uniform` (line ~670, Phase 4 / F-4a uniform timeout drain), `drain_pending_by_identity` (line ~745, Phase 6 / F-10 Identity-arrival drain), `drain_pending_by_federation_relationship` (line ~795, Phase 7.5 / F-3 relationship-arrival drain). All three drain a HeldPending entry, re-dispatch via internal `dispatch_event`, and currently drop the outcome via `let _ =`. Same-family-same-atomic-close — sibling-shape to topo-sort Commit 2a's layered-B3 atomic close at J-101 where one primary fix surfaced two validator-companion encodings closed atomically per D-067 Option E (second project-wide layered-B3 instance recorded at design close; not yet a durable pattern, three would be). Each drain helper's signature changes to return `Vec<Event>` containing the drained events whose Accepted outcomes need persisting; the recursion pattern is Shape β2 (each helper returns; `dispatch_event` aggregates via concatenation; the initial event stays with `process_inbound`'s existing persist site rather than the returned vector) chosen over Shape β1 (accumulator threaded through recursion) on five grounds: self-documenting signatures; easier code-review (each function's contract is visible at its signature); bounded recursion depth makes Vec allocation cost negligible at protocol traffic levels; sibling-shape to the existing `drain_pending_messages` recursion pattern in runtime.rs; avoids the "outer caller forgets to thread the accumulator" footgun Shape β1 introduces.

**Q4 — sentinel-tree in-scope at milestone close (Commit 3b-1 collapse)** `[JOE-LOCK: locked 2026-05-23]`. The four sentinel-tree files at `xgen-node/src/tests/` (`phase9_harness.rs`, `phase9_three_node_anti_transitivity.rs`, `phase9_drop_and_recover.rs`, `mod.rs`) currently uncommitted as Phase 9 Commit 3b-1 in-flight work ship atomic at this milestone's close as the activating regression lock at integration level for the persistence fix. Scenario 3 (drop-and-recover with relationship state) transitions FAIL → PASS, verifying the fix end-to-end. Q4(a) lock effectively collapses Phase 9 Commit 3b-1 INTO this sub-amendment milestone close — Phase 9 resumes at Commit 3b-2-equivalent (Scenarios 2 + compound scenarios C2/C3/C5/C7/C9/C10) after this milestone close, dependency depth unchanged in shape. Rejected alternative: Q4(b) sentinel-tree ships in a separate Phase 9 Commit 3b-1 sequence after this sub-amendment milestone close (rejected on activating-regression-lock-at-close grounds; verifying the fix at integration level inside the milestone that introduces it is the bidirectional + topo-sort precedent shape, and decoupling them would lose the integration-level lock at the close commit).

**Code surfaces touched in implementation Commit 2 + Commit 2a + Commit 3.** Commit 2: `xgen-core/src/node/runtime.rs::ingest_event` signature change with the verbatim code-comment block at the signature site (four locked structural elements plus rungs-list bullet per `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` §4.3 Joe-lock checkpoint #4); `xgen-core/src/dag/graph.rs::GraphError` visibility widening to `pub` if not already public; `xgen-core/src/node/runtime.rs::topological_sort` re-export as `pub` (single source of truth per D-067 + D-076 no-drift-surface family); `xgen-node/src/app.rs::replay_spaces_from_dir` sort-on-replay logic at line ~2628 with `tracing::warn!` for any residual GraphError surface. Commit 2a: `DispatchOutcome::Accepted` variant change adding `additional_persisted: Vec<Event>`; three drain helpers' signatures change to return `Vec<Event>`; `dispatch_event` aggregates at three drain call sites; `process_inbound` adds the persist-loop block immediately after existing initial-event persist call + sibling block in `handle_identity_replicate_msg`. Commit 3: four sentinel-tree files refined per the refinement-vs-rework distinction at runbook §5.2 (routine refinement folds into Commit 3; structural rework escalates to Joe-lock checkpoint #5) + Scenario 3 transition FAIL → PASS + verification rigour 5 isolated + 3 workspace = 8 green runs minimum, sibling-shape to topo-sort J-101 verification rigour.

**Cross-references.** `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` (audit doc, COMPLETED v1.1 at J-105 — code-grounded evidence at file:line granularity; the silent-discard pattern at runtime.rs:181 plus the three drain-helper sites form the compounding-bug surface); `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` (design doc, COMPLETED v1.1 at this Commit 1 — Q1→Q4 walk + four Joe-locks + candidate D-NNN flag at §8); `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` (five-commit Clair runbook, ACTIVE v1.0 at this Commit 1). Sibling-shape precedent at §6.4.3 (topological-sort) — same milestone-internal sub-amendment shape (audit → design → runbook → impl → close); same one-Joe-lock-per-question lock pattern; same code-comment-block-at-load-bearing-site discipline; same verification-rigour 5+3=8-green-runs minimum at integration close. Candidate D-NNN "ingest path invariant encoding" flagged at design doc §8 stays flagged-not-promoted; promotion gated on (a) dependent work surfacing a concrete drift instance (a).iii.β did NOT catch, OR (b) Joe-lock on philosophical/strategic grounds independent of a surfacing gap. Per D-069 audit-vs-design boundary discipline: this milestone's audit named the questions, design locked the answers, runbook translated locks into Clair-executable form, implementation ships at code level — the four-phase D-071 shape sibling to Phase 7.5 cold-start, bidirectional `federation_nodes`, and topological-sort wire-order determinism.

**What this phase does NOT change.** Wire format unchanged. `Event` struct unchanged. `EventStore` container type unchanged. `state.federation_add` content schema unchanged. The fix is purely receive-side persistence discipline; sender behaviour is untouched. Existing on-disk events stay valid — events that landed via `process_inbound`'s pre-Commit-2a path before this milestone close were always persisted explicitly; the gap was only in drained events not surviving restart, and any in-flight drain state at upgrade time replays from the HeldPending buffer (which itself rebuilds from on-disk events through the unchanged `process_inbound` pre-existing-event path). Pass-1-neutral: `tasks/XGID_RETROFIT_PASS_1_IMPL.md` (Status ACTIVE v2.0) is unaffected — the `additional_persisted: Vec<Event>` field uses the existing `Event` type. The four other silent-discard sites in `ingest_event` (event_id-missing-return for unsigned events; store.insert silent under duplicate-event idempotency; two apply_event silents in StateSpaceCreate and default branches) stay UNTOUCHED per Q1 narrow-scope discipline; they belong to candidate D-NNN's future-walk question and broadening scope to them requires Joe-lock at a future audit phase. D-NNN itself is not promoted to DECISIONS.md in this milestone — design doc §8 is the canonical flag.

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

The federation-relationship check (the second part of Option 2) answers a different question: *is this Node entitled to be relaying events for this Space at all?* This is an authorisation question, not a cryptographic-authenticity one. A Node X with no federation relationship with B for Space S has no business pushing Space-S events to B, even if those events themselves are cryptographically authentic. `SpaceState.federation_nodes[S]` is consulted to answer this question.

Two questions, two checks, neither substitutable for the other:

| Question | Check | Data source |
|---|---|---|
| Is this event real (did the claimed author make it)? | Event-level signature verification | Identity record in B's local registry |
| Is this Node entitled to relay events for this Space? | Federation-relationship lookup | `SpaceState.federation_nodes[S]` in B's local store (Phase 7 Lock A1) |

The session authentication answers neither of these. It answers *which Node process is at the other end of the WebSocket*, which is fresh once per session, not fresh per event.

### 6.6 Dependency on Identity replication

Both Option 1 and Option 2 require B to have the relevant Identity records to verify signatures. If B receives an event from a Space member whose Identity record B does not hold, the signature cannot be verified and the event is rejected — except where F-10 generalises HeldPending to buffer the event pending Identity record arrival (see §13).

The existing Identity replication subsystem (audit §2.3, Layer 18 / replica registry) is responsible for getting Identity records to B. This milestone does not change Identity replication, but it does *depend* on it working. The implementation runbook should include a verification step: when a federation event push lands, Identity records for the event's author must already be present, or the event arrival surfaces a sync problem (handled by F-10's extended HeldPending).

This is an ordering constraint that needs verification. The runbook should include integration tests where the receiver does and does not have the relevant Identity record, and confirm both the F-10 HeldPending path and the rejection-with-reason path are correctly traversed.

### 6.7 Implication for F-4 (validation asymmetry)

Option 2 pre-commits to F-4 fixing the Path B/C asymmetry. There is no way to honour Option 2's first check (event-level signature verification) without lifting `process_inbound` Paths B (membership.join) and C (other state events) to the same signature-verification discipline that Path A (message events via `accept_message`) has today. F-3's lock therefore constrains F-4's design space: F-4 must produce a path where all three event-type families share the same verification pipeline on the receive side. Whether they share a code path or run separate parallel paths is the actual F-4 question.

### 6.8 Implementation-runbook notes from F-3

- The two-check ingestion gate composes existing logic (event-signature verification from `accept_event`, federation-relationship lookup against `SpaceState.federation_nodes`). It is not new cryptographic machinery; it is wiring existing checks into a single gate.
- The `SpaceState.federation_nodes` lookup is in the hot path for every federation-received event. The runbook should consider caching or in-memory indexing if profiling shows the lookup is expensive at scale. Phase 1 / Phase 2 scale will not stress this; the existing `Vec::contains` over typically 1-3 federated peers per Space is sufficient.
- Rejection due to either check failing must produce a clear log line (Node-side observability) and, in coordination with M6 (new) Phase 2 envelope-`event_id` work, a wire-layer signal back to the originating peer so the sender can correlate. The exact form of that signal is M6 Phase 2's design, not this document's; this document only flags that F-3 rejection paths are one of the populating contexts for it.

---

## 7. Framework decision F-4 — Validation asymmetry closure

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

### 7.1 The question

F-3 locked Option 2: every event arriving on the federation channel must pass event-level signature verification against the author's Identity record. The audit (§3.2) established that today's `process_inbound` only does this for **Path A** (message events via `accept_message`). **Path B** (membership.join) and **Path C** (other state events) bypass signature verification, bypass timestamp checks, and have no HeldPending integration.

F-4 asks: **how does `process_inbound` get reshaped so all three paths apply the same verification discipline?**

This is the precondition the audit flagged: closing federation push without closing the asymmetry would land a vulnerability. F-4 closes it.

### 7.2 The three paths today

From audit §3.2, restated for context:

| Path | Triggered by | What runs today | What's missing |
|---|---|---|---|
| **A** | `MessageText`, `MessageFile`, `MessageReaction`, `MessageRedact` | `accept_message` → full 13-step pipeline → HeldPending on unknown predecessor | Nothing — this is the reference implementation |
| **B** | `MembershipJoin` | Two pre-checks (Space exists, new-joiner detection), then `ingest_event` directly | Signature verification, timestamp check, HeldPending, full pipeline |
| **C** | All other state events (`state.*`, etc.) | Two pre-checks (AI role violation, AI operator target/permission), then `ingest_event` directly | Signature verification, timestamp check, HeldPending, full pipeline |

Path A is the model. Paths B and C are the asymmetry. The three paths exist for legitimate reasons — joins need new-joiner detection logic that messages do not, AI operator events have permission checks that other events do not — but the *validation core* (signature, timestamp, predecessor handling) should be the same for all three. Today it is not.

### 7.3 Options considered

**Option 1 — Unify into a single code path: shared validation core + per-event-type post-validation handlers.**

Refactor `process_inbound` so all three paths route through a single validation function. The path-specific logic (new-joiner detection, AI checks) becomes pre- or post-processing around the shared validation, not a substitute for it. There is no way to reach event-handling code without passing through the validation core first.

- ✅ One source of truth for validation. The drift surface that produced the audit finding cannot recur — there is nowhere to silently bypass the pipeline because there is only one pipeline.
- ✅ Matches the M5 `ops::*` precedent and D-067. That refactor architecturally eliminated drift between dispatchers; this is the M5-shaped fix for the validation-pipeline layer.
- ✅ HeldPending applies uniformly. The unknown-predecessor case for membership/state events gets the same buffer-and-retry treatment messages already have. Closes audit §3.3 Sub-finding "Scenario A non-message."
- ⚠️ Larger refactor surface. Touches three arms of `process_inbound`, the `accept_message` / `accept_event` boundary, and likely `runtime.ingest_event`. More tests to update.

**Option 2 — Keep three paths, add the same checks to B and C in parallel.**

Each path independently calls signature verification + timestamp check + HeldPending logic. The path structure stays the same; each arm just gains the missing checks.

- ✅ Smaller refactor. Less code moved.
- ❌ Re-creates the exact drift surface the audit just surfaced. Three independent verification implementations is *how* the asymmetry happened in the first place. Adding the missing checks in three places preserves the structural condition that allowed the gap to exist.
- ❌ HeldPending gets implemented three times. The buffer-and-retry logic is non-trivial; duplicating it across paths is the M5-pre-refactor pattern that D-067 was written to eliminate.
- ❌ Future audit risk. The next audit looks at these three paths and finds drift. Not "if" but "when."

**Option 3 — Hybrid: shared validation function called from each path.**

Extract the validation core into a new function. Each of the three paths in `process_inbound` calls it before doing its path-specific work. The path structure stays; the validation is shared.

- ✅ One source of truth for validation. Same as Option 1.
- ✅ Less structural upheaval than Option 1.
- ⚠️ Two seams instead of one. The validation function has a clean contract, but the three paths each need to integrate it correctly. Still more attack surface than Option 1 for "did this path remember to call the validator?"
- ⚠️ `accept_message` / `accept_event` already does something like this internally. Risk of producing a redundant abstraction if the refactor does not carefully understand the existing layering.

### 7.4 Decision — Option 1 (unify into a single code path)

**`process_inbound` is refactored into a dispatcher with the following shape:**

1. **Common validation core** runs first, regardless of event type. The equivalent of today's `accept_event` 13-step pipeline: signature verification, timestamp check, predecessor presence (with HeldPending on miss), structural checks. Returns `Validated(event)`, `Rejected(reason)`, or `HeldPending`.
2. **Event-type-specific post-validation handlers** then run on the validated event. `MessageText` → message-handling logic (state machine apply, etc.). `MembershipJoin` → new-joiner detection + ingest. `state.ai_operator_delegate` → AI permission check + ingest. And so on.

The validation core has one implementation. The post-validation handlers are still per-event-type because they legitimately differ. The asymmetry the audit found was that *validation* was per-path, not that handling was per-path. Option 1 fixes the validation half while leaving the handling differences intact.

**Reasoning recorded.** Three reasons Option 1 beats Options 2 and 3:

1. **Eliminates drift architecturally, not by discipline.** Option 2 adds the same checks in three places — the same condition that produced the original asymmetry. Option 3 has three call sites to a shared function, which is better than three implementations but still leaves "did this path remember to call the validator?" as a future-bug surface. Option 1 has no such surface — there is no way to reach event-handling code without passing through validation first.
2. **Matches the M5 / D-067 precedent.** That refactor architecturally eliminated the drift surface in `xgen-client-lib::ops`. The same principle applies here: one canonical function per concern, dispatchers are thin shims.
3. **HeldPending becomes a property of the validation core, not of `accept_message` specifically.** This closes audit §3.3 Sub-finding "Scenario A non-message" (the case where a non-message event arrives with unknown predecessors and gets silently ingested with state-machine no-op). Under Option 1, that case becomes "validation core returns HeldPending → event buffered → retried when predecessors arrive" — same as messages today.

### 7.5 Sub-decision F-4a — HeldPending timeout policy for state events

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

**The question.** Today's HeldPending buffers messages whose `prev_events` reference unknown predecessors, retries when the predecessors arrive, and discards after a 30-second timeout (audit §3.2 Path A; Ch4 §4.12.3). Under Option 1, this behaviour extends to membership/state events. What's the timeout policy for state events?

**Decision — same as messages (30 seconds), uniform across all event families in v1, per-family configurable in v2 if needed.**

**Reasoning recorded.** HeldPending is a short-window optimisation, not a durability guarantee. If a state event's predecessors do not arrive within 30 seconds, the real recovery mechanism is the F-1a tip exchange on the next session re-establishment — not "wait forever in memory." Keeping the timeout uniform across event families means one buffer, one timer, one set of edge cases to test. Defaulting longer or to no-timeout would either hold memory for events that the tip-exchange will recover anyway, or hold memory unboundedly — both contradicting the rest of the design's "best-effort + sync recovery" posture. If a deployment surfaces a real case where state events need a longer window (slow cross-continental federation, etc.), v2 can add per-family configuration without breaking the design.

### 7.6 Sub-decision F-4b — Pre-validation check placement

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

**The question.** Some checks today happen *before* validation in the path-specific arms: "Space exists" check in Path B, "AI role violation" and "AI operator target/permission" in Path C. Under Option 1, these can stay before validation (cheap fail-fast) or move after (consistency at the cost of doing crypto on events that would be rejected anyway). Which goes where?

**Decision — structural pre-checks (Space exists) before validation; semantic pre-checks (AI role violation, AI operator permission) after.**

**Reasoning recorded.** "Space exists locally" is a cheap structural check (HashMap lookup). Running it before signature verification avoids wasting Ed25519 crypto on events for Spaces this Node does not host — same shape as how `accept_message` checks Space existence early today. "AI role violation" and "AI operator permission" are semantic checks — they answer "is this validated event also permitted?" not "is this event structurally well-formed?" Conceptually they sit *after* validation passes. Moving them after also makes the audit trail cleaner: the trace shows a validation-passed event being rejected for a permission reason, not a permission-rejected event whose validation status is ambiguous.

### 7.7 Final pipeline shape

The Option 1 + F-4a + F-4b decisions produce the following pipeline shape for `process_inbound`. This is a sketch of the dispatcher, not a code spec — the actual implementation is Clair's work in the runbook phase.

```
process_inbound(event, peer_session) →

  # 1. Structural pre-checks (cheap; fail-fast; F-4b)
  if event references a Space this Node does not host → reject
  
  # 2. Federation-relationship check (F-3 second check; only for federation-channel events)
  if event arrived via federation AND no relationship for (peer, space_id) → reject
  
  # 3. Validation core (F-4 Option 1 unified path)
  match validation_core(event):
    Validated(event) → continue
    HeldPending → buffer with 30s timeout (F-4a; trigger generalised per F-10); return
    Rejected(reason) → log; emit rejection signal per M6 Phase 2; return
  
  # 4. Semantic pre-checks (F-4b)
  match event.type:
    StateSpaceCreate | StateDmSpaceCreate if sender is AI → reject (AI role violation)
    StateAiOperatorDelegate | StateAiOperatorRevoke → check target + permission; reject if fails
    _ → continue
  
  # 5. Event-type-specific post-validation handler
  match event.type:
    MessageText | MessageFile | MessageReaction | MessageRedact → message handler
    MembershipJoin → new-joiner-detection + ingest
    state.* → ingest
  
  # 6. Fan-out (Stage 5 local + Stage 6 federation push per F-1)
  apply_fanout(...)
  apply_federation_push(...)  # new — but ONLY for locally-submitted events (F-5)
```

Note that the validation core (step 3) handles signature verification, timestamp check, and predecessor presence uniformly for all event types. Today, Path A does this internally via `accept_message → accept_event`; Paths B and C skip it. After F-4, the core is reached by every event regardless of type.

The `apply_federation_push` call in step 6 is gated by F-5: it runs only when the event was locally submitted to this Node (not received via federation). See §8.

### 7.8 Implementation-runbook notes from F-4

- The validation core is the conceptual equivalent of today's `accept_event`. Whether the refactor renames it, splits it, or keeps the existing function as-is is the runbook's call — what matters is that there is exactly one of it and every event family passes through it.
- The `accept_message` boundary may evaporate as a separate function under Option 1, becoming just the message-handler arm of the unified dispatcher. Alternatively, `accept_message` may remain as a thin wrapper around the validation core for backward-compat with existing callers. Clair's latitude.
- HeldPending today lives inside `accept_message` / runtime. Moving it to the validation core means the buffer needs to be reachable from all three event families' code paths. The buffer's identity (one per Node, one per Space, etc.) is a runbook detail; the design only requires that buffer behaviour applies uniformly. F-10 extends the buffer's trigger condition; the buffer itself is the same.
- Existing tests for `accept_message`'s HeldPending behaviour serve as the test template for the extended coverage. The runbook should explicitly include integration tests for the three Scenario-A cases the audit identified (Path B unknown predecessor, Path C unknown predecessor, plus Path A regression).
- The rejection signal in step 3 (when validation fails) is the wire-layer signal M6 (new) Phase 2 is designing. F-4's contribution is to ensure the rejection paths exist consistently across all three event families; M6 Phase 2 wires them to the `Error` variant with envelope `event_id`.
- The federation-relationship check in step 2 is technically F-3's work, not F-4's, but it lives in the same dispatcher and is implementation-coupled. The runbook treats them as one unit.

---

## 8. Framework decision F-5 — Transitive federation

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

### 8.1 The question

The audit (§3.5) found that `apply_fanout` operates exclusively against `ClientSenders` — the map of locally-connected client identities. It has no notion of peer Nodes, no federation-relationship lookup, no forwarding to peer connections. Even with F-1's push channel landing, the implementation must explicitly decide what happens when a Node *receives* an event from a federation peer: does it re-propagate that event to its *other* federation peers, or does it stop?

Concretely: Node A federates with Node B for Space S. Node B also federates with Node H and Node R for Space S. A and H, A and R have no direct relationship. Alice on A posts E in Space S. A pushes E to B. Now: does B re-propagate E to H and R?

If yes (in any form), that's transitive federation. If no, propagation stops one hop from origin.

This question is not optional. Even "we don't support transitive federation" is a design call — without an explicit lock, an implementer could accidentally enable it (or accidentally break it) and the protocol's behaviour would drift on it without anyone noticing.

### 8.2 Why this matters

The three frames matter for different concerns:

- **Scaling.** Transitive federation lets a Space exist across N Nodes without each Node needing direct relationships with every other Node. Hub-and-spoke topologies become viable; mesh topologies become unnecessary. For N participating Nodes, pairwise (Option 1) requires N*(N-1)/2 relationships; transitive (Option 2) requires only enough for connectivity.
- **Censorship resistance.** A Space-S participant whose home Node defederates from one peer can still reach the Space via other peers, if transitivity is permitted.
- **Authority chains.** Every additional hop is an additional Node that can fail, be compromised, or drop events. Transitive federation extends the trust chain in ways the receiver cannot independently audit.

### 8.3 Options considered

**Option 1 — Locked-out (no transitive federation in v1).**

Federation is pairwise only. Node B receives events from A for Spaces A and B share. B does not re-propagate them to B's other peers, even if those other peers also share those Spaces.

Operationally: every Space that needs to span more than two Nodes requires a direct federation relationship between every pair of participating Nodes (mesh topology). For N participating Nodes, that's N*(N-1)/2 relationships.

- ✅ **Simplest authority model.** Every event arriving at a Node came directly from another Node that the receiver has a federation relationship with for the relevant Space. No transitivity, no "this event came through 3 hops" question.
- ✅ **Authority chain is one hop.** Each receiver can independently verify the immediate sender's federation relationship. Compromised peers cannot launder events through other peers.
- ✅ **No deduplication problem.** Idempotent ingest via `event_id` dedup is unnecessary at the federation layer (events arrive exactly once per direct relationship).
- ✅ **No cycle risk.** Without re-propagation, federation cycles cannot form.
- ❌ **Mesh scaling cost.** For a Space hosted across many Nodes, every pair needs a direct relationship. Operationally expensive at scale.
- ⚠️ **Defederation cuts off cleanly but absolutely.** If A defederates from B for Space S, B receives nothing from A. There is no fallback path.

**Option 2 — Locked-in (transitive federation by default).**

When B receives event E from A for Space S, B's `apply_fanout`-equivalent also pushes E to B's other federation peers that share Space S. Events propagate transitively through the federation graph until every Node that holds a relationship for S has received E.

- ✅ **Scales naturally.** Hub-and-spoke topologies viable.
- ✅ **Resilience to defederation.** A defederation between two specific Nodes does not sever a Space across the wider topology.
- ❌ **Authority chain extends without bound.** Receiver B has no way to verify whether the event A pushed actually originated at A or arrived at A via another transitive hop. The F-3 federation-relationship check verifies "A is allowed to relay events for S" — but A might be relaying an event from peer X that A trusts via different rules than B trusts X. Trust is not transitive in this way.
- ❌ **Duplicate-delivery problem.** If A and B both have relationships with C, and A pushes E to B and to C, B also pushes E to C. Idempotent ingest handles correctness, but doubles federation bandwidth.
- ❌ **Cycle risk.** A → B → C → A is a federation cycle. Requires explicit cycle detection in push logic.
- ❌ **Premature commitment.** Locking transitive in v1 means we're adding protocol complexity for a scaling pressure that does not yet exist.

**Option 3 — Opt-in (peer-by-peer transitivity flag).**

Pairwise by default (Option 1), but a federation relationship can be marked "this peer is allowed to relay transitively" by mutual agreement. When B receives E from A under a transitive-marked relationship, B re-propagates to *only those peers* B has also marked transitive with.

- ✅ Operator control over the trust chain.
- ✅ Limits the authority-chain extension to explicit-agreement peers.
- ⚠️ Adds protocol surface (relationships gain a flag; wire shape grows).
- ⚠️ Cycle risk still present.
- ⚠️ Adds operational complexity to a feature without yet-pressured scaling needs. Operators must make per-peer decisions before they have evidence of which peers should be transitive.

### 8.4 Decision — Option 1 (locked-out, no transitive federation in v1)

**Events propagate exactly one hop from their origin.** A Node only federation-pushes events that were locally submitted to it. Events received via federation are accepted into the local DAG and fanned out to local clients (Stage 5), but are NOT re-pushed to other federation peers.

**Worked example.** Node A federates with B. Node B also federates with H and R for the same Space. Alice on A posts E:

- A → B ✅ (direct relationship; A pushes via F-1)
- B → H ❌ (B does not re-propagate; B is endpoint for received-via-federation events)
- B → R ❌ (same)
- A → H, A → R ✅ if those Nodes have their own direct relationships with A; otherwise they don't see E

H and R need their own direct federation relationship with the originating Node (A) to receive E. Federation is pairwise; there is no transitive relay.

**Reasoning recorded.**

1. **Current pressure does not justify the complexity.** Phase 2/Phase 3 deployments and the M6/M7/M8/M9 roadmap do not have a use case where mesh-topology cost is the limiting factor. Locking transitive federation now would add authority-chain complexity, duplicate-delivery handling, and cycle prevention that all want their own design rigour, without solving a present problem.
2. **Option 2's authority-chain extension contradicts F-3.** F-3 locked "the federation relationship is per-Space and per-peer." Transitive federation by default lets event authority propagate through Nodes the receiver has no direct relationship with, weakening the F-3 gate. The per-event signature check still holds, but the federation-relationship check becomes "this peer claims a relationship to a peer with a relationship for this Space" — a weaker authorisation than F-3 intended.
3. **Locked-out is the easiest position to relax later.** If a future milestone surfaces a real scaling need, v2 can add Option 3 (peer-by-peer opt-in) without breaking v1 deployments. Going the other way — locking transitive in now and discovering we do not want it — would mean a protocol break.

### 8.5 Implementation requirement — explicit origin gating

The implementation MUST explicitly check, before calling `apply_federation_push`, that the event being pushed was **locally submitted** to this Node. Events that arrived via federation (over a peer session) MUST NOT enter the federation-push code path.

This is the line that prevents accidental transitivity. Without it, a future refactor that "fixes" `apply_fanout` to be more uniform across senders could inadvertently enable transitive propagation. The runbook should make this requirement prominent — likely as a guard at the top of `apply_federation_push`, with a comment citing F-5 and this section.

A plausible implementation marker: events carry an in-memory "origin" indicator (locally-submitted vs. received-via-federation) that the federation-push function inspects before forwarding. The marker is implementation detail, not wire-visible — it lives only in the Node's runtime processing of an event, not in the event itself.

### 8.6 Evolution path to v2 (Option 3, if scaling pressure surfaces)

This section is forward-looking, not a v1 commitment. It exists to make clear that "no transitive federation in v1" is not "no transitive federation forever."

If a future deployment surfaces a real need for transitive federation (a Space wants to span 20+ Nodes; mesh-relationship cost becomes operationally painful; hub-and-spoke topology is the natural answer), the path forward is **Option 3 — peer-by-peer opt-in transitivity**:

- Federation relationships gain a `transitive_relay` flag (default `false`).
- The flag is negotiated at handshake or set via admin tooling (M6 admin verb territory).
- When set to `true` on a relationship, B accepts that events from this peer may have been relayed from elsewhere, and B is willing to re-relay events received from this peer to *other* relationships also marked `true`.
- Cycle detection: a "via" or "hop count" field in the federation-envelope tracks the propagation path; events with too many hops or a cycle are dropped.

This evolution is non-breaking for v1 deployments because:
- Existing relationships default to `transitive_relay: false` (= today's locked-out behaviour).
- No wire-shape change for v1 events; the `via` / `hop_count` field is added only on the federation envelope, not on Event itself.
- Implementations that don't honour the flag fail safely toward Option 1's behaviour.

The decision to take this evolution path, and the design specifics, are out of scope here. The note exists only to preserve the option.

### 8.7 Implementation-runbook notes from F-5

- The origin-gating check (§8.5) is the load-bearing piece. The runbook should treat it as a hard requirement and include a regression test that confirms a federation-received event does NOT trigger `apply_federation_push`.
- The "origin indicator" implementation may already exist in some form (Connection type carries Node-vs-Client information that distinguishes inbound channels). Clair's latitude on whether to add a new explicit marker or reuse existing context.
- Identity replication push (audit §2.3, `push_identity_to_peers`) is a separate subsystem and not affected by F-5. Identity records propagating Node-to-Node is its own mechanism with its own rules; F-5 governs Space event propagation only.
- The Stage-5 local fan-out is unaffected. A federation-received event still gets fanned out to local clients normally; F-5 only blocks the federation-push side of the post-ingest path.

---

## 9. Framework decision F-6 — `sync_complete` wire shape and the 500ms quiet-time fallback

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

### 9.1 The question

Audit §4.2 found that the existing `transport.sync_request` mechanism has no protocol-level end-of-stream signal. The spec-defined `transport.sync_complete` (Ch3 §3.3.6) is unimplemented. Today, the client detects "the response stream is over" by **500ms of silence on the channel** — a quiet-time heuristic. All four production `SyncRequest` callers (`batch.rs:83`, `ai_service.rs:224`, `ops.rs:721`, `ops.rs:939` per audit §4.5) use this pattern.

The audit flagged this as **LOW today, MEDIUM at scale**:

- **Latency floor failure mode.** Under high WAN latency, the peer might take >500ms between consecutive events when delivering a large catch-up. The requester times out mid-stream, believes catch-up is complete, and proceeds — having missed events still in flight.
- **High-volume failure mode.** Under a large catch-up, the stream may exceed 500ms gaps between batches as the peer serialises and writes events. Same outcome.

Both failure modes are silent — the requester does not know it has prematurely terminated catch-up. The recovery is "you'll get the missed events on the next sync_request" — but if the application has already proceeded under "catch-up done," it may have committed actions based on incomplete state.

This milestone extends `sync_request` Node-to-Node (per F-1's pull-on-gap recovery). The 500ms heuristic was acceptable client-to-Node on a single host or LAN; extending it to inter-Node federation across WANs makes the failure modes substantially more likely. **Federation pull-on-gap is the exact code path that will hit the WAN-latency failure mode first.**

F-6 asks: does this milestone fix the heuristic, or does it inherit the heuristic and accept the failure mode for v1?

### 9.2 Options considered

**Option 1 — Fold in: implement the spec-defined `sync_complete` wire shape now.**

Implement `TransportMessage::SyncComplete`. The peer sends it after the last event of a sync response stream. Requesters wait for `SyncComplete` instead of guessing via quiet-time. Existing client-side callers migrate to use the explicit signal.

- ✅ **Fixes the failure mode at the protocol level.** No more guessing; the peer tells the requester when the stream is done.
- ✅ **Matches the spec.** Ch3 §3.3.6 already calls for this shape. The implementation has been deferred since Phase 1; folding it in here brings code and spec into alignment.
- ✅ **Federation pull-on-gap inherits a reliable mechanism.** When F-1's gap-recovery code path issues a sync_request to a peer Node, it gets a definitive completion signal rather than guessing across a WAN.
- ⚠️ **Adds work to this milestone.** Wire shape + Node-side emission logic + client-side migration + tests. Not large in absolute terms but real.
- ⚠️ **Touches the four existing production callers.** Each needs to migrate from quiet-time to explicit-signal. Each call site is a small surgical change but they need to be done together.

**Option 2 — Defer: keep the 500ms heuristic, accept the failure mode for v1.**

Leave the existing mechanism in place. Federation pull-on-gap inherits the heuristic. v2 (or a separate "Sync Mechanism Hardening" milestone) fixes it later.

- ✅ Smaller milestone.
- ❌ **Lands a known failure mode in federation push.** WAN latency between federation peers is exactly the case where this fails. The audit said "becomes relevant when the design extends it Node-to-Node" — this milestone is the moment of extension.
- ❌ **Discoverable failure.** A production deployment with federation across regions would hit this within days, not months. The fix would then need to be a hotfix milestone.
- ❌ **Contradicts D-065** ("honest behaviour over polite behaviour"). A timeout-based heuristic that silently terminates catch-up is the polite-but-incorrect behaviour the project rejects elsewhere.

**Option 3 — Partial fold-in: implement `sync_complete` but keep quiet-time as a fallback for backward compatibility.**

Add `sync_complete`; clients still accept silence as end-of-stream for older Nodes that do not emit the signal. Migration is gradual.

- ✅ Backward-compatible.
- ⚠️ Preserves the failure mode in mixed-version deployments.
- ⚠️ More complex to test (both modes coexist).

### 9.3 Decision — Option 1 (fold in)

**Implement `transport.sync_complete` in this milestone.** All four production callers migrate to wait for the explicit signal. Federation pull-on-gap uses the explicit signal from day one. The 500ms quiet-time heuristic is removed.

**Reasoning recorded.** Three reasons:

1. **Federation push is the exact code path that surfaces this failure mode.** The audit flagged the heuristic as "LOW today, MEDIUM at scale." This milestone is the scaling event — it is not a future hypothetical, it is the current design intent (Node-to-Node pull across WANs in F-1's gap-recovery path). Inheriting the heuristic into federation would be knowingly landing a defect.
2. **The wire shape is already specced.** Ch3 §3.3.6 has called for `sync_complete` since Phase 1. The implementation was deferred. F-1's design depends on reliable sync; implementing the spec's existing answer is cheaper than designing a workaround.
3. **D-065 alignment.** "Honest behaviour over polite behaviour" applies directly: a quiet-time guess is polite-but-incorrect. The honest answer is "I told you when I was done."

Option 3 was tempting for "backward compatibility" framing, but this milestone is the first to ship Node-to-Node `sync_request` — there are no "old Nodes" to be backward-compatible with at the federation layer. Backward compatibility with the *existing* client-to-Node use is preserved by Option 1 too: the migration is internal to `xgen-client`; old behaviour disappears entirely.

### 9.4 Sub-decision F-6a — `SyncComplete` wire shape

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

**Decision.** The wire shape is:

```
TransportMessage::SyncComplete {
    since: String,                  // echo of the request's since cursor
    new_tip: String,                // the responder's current DAG tip for the relevant Space
    continue_from: Option<String>,  // F-7 — non-null when more events are available (see §10)
}
```

**Reasoning recorded.** Three shape options were considered for the F-6 portion (`since` + `new_tip`):

- **Minimal** — `SyncComplete { since }`. Confirms stream end; nothing else.
- **With count** — `SyncComplete { since, count }`. Also reports number of events delivered.
- **With new tip** — `SyncComplete { since, new_tip }`. Also reports the responder's current DAG tip.

The with-new-tip variant is chosen because the new tip is exactly what the requester needs to make its *next* sync_request authoritative without a separate query. For federation pull-on-gap specifically, knowing the peer's current tip after a sync response means the requester knows whether it is now caught up or whether there is more to chase — directly relevant to F-1a's tip exchange semantic. The count was rejected as marginal; the new tip closes an ergonomic gap that exists in the wire protocol today.

The `since` field echoes the request's cursor so the requester can correlate the completion signal with the request that triggered it (in case multiple sync_requests are in flight).

The `continue_from` field is added by F-7 (see §10). Its presence in the F-6 wire shape reflects the design decision in F-7 to compose pagination on top of the same `SyncComplete` message rather than introducing a separate pagination signal.

### 9.5 Sub-decision F-6b — Safety-net timeout for missing `SyncComplete`

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

**The question.** Under Option 1, the requester waits for `sync_complete` as the authoritative stream-end signal. But a timeout still needs to exist for the case where the peer crashes mid-stream, the network silently drops the `sync_complete` message, or the peer is buggy. What is the timeout, and is it protocol-fixed or implementation-configurable?

**Decision — implementation-configurable, reference-implementation default of 5 seconds. NOT protocol-fixed.**

**Reasoning recorded.**

1. **It is an operational question, not a correctness question.** A LAN deployment can use 1 second safely. A satellite-link deployment may need 60 seconds. The protocol-correct behaviour is the same in both cases — wait for `sync_complete` — but the operationally-appropriate patience differs. Hardcoding it into the protocol would force every deployment to make compromises.

2. **The spec follows this pattern elsewhere.** Connection keepalive (Ch3 §3.3.6) specifies the *mechanism* (ping/pong) but the *intervals* (30s/10s) are recommendations the implementation can override. Same shape for sync timeouts: protocol mandates the mechanism, implementation defaults provide reasonable values, operators can tune.

3. **Avoids repeating the "magic number bake-in" problem.** Today's 500ms is exactly the bug F-6 exists to fix — a number embedded in code that became a de-facto protocol constant nobody decided. Doing the same with a new number would repeat the mistake. Configurable from day one prevents the repeat.

**Concrete configuration.** Config field `[sync].completion_timeout_seconds` in both `xgen-node_config.toml` and `xgen-client_config.toml`. Default 5 seconds. Operator may override.

**Choice of 5 seconds as the default.**

- Long enough that no realistic WAN handoff between `sync_complete`-aware peers should hit it under normal conditions.
- Short enough that a hung peer or dropped message does not stall the requester for an unacceptable duration.
- 10x today's 500ms heuristic, so the failure mode that motivated F-6 (high-latency multi-second gaps between events) is comfortably accommodated.

### 9.6 The shift in role of the timeout — before vs. after F-6

The 500ms heuristic today plays the role of "the primary completion signal." After F-6, the timeout (now 5s configurable) plays a fundamentally different role: it is the **safety net for when the peer fails to send `sync_complete` at all**. This distinction matters because the failure modes are different:

| Aspect | Today (500ms heuristic) | After F-6 (5s configurable) |
|---|---|---|
| Role | Primary completion signal | Safety net for missing `sync_complete` |
| Failure mode if exceeded | Silent premature catch-up termination | Visible "peer never said done; giving up" — surfaces in logs |
| Value | 500ms, hardcoded | 5s default, configurable |
| Protocol-visible | Implicit (the heuristic *is* the mechanism) | Not protocol-visible (the *mechanism* is the wire message; the timeout is implementation-side) |

The "silent premature termination" failure mode disappears because the timeout is no longer making a positive assertion ("the stream is complete"). It is making a negative assertion ("the peer never told me it is complete; something is wrong"). The latter surfaces as an error in logs; the former silently corrupts state.

### 9.7 Implementation-runbook notes from F-6

- The four production callers (`batch.rs:83`, `ai_service.rs:224`, `ops.rs:721`, `ops.rs:939`) all need to migrate from the quiet-time pattern to waiting for `SyncComplete`. The change shape is mechanical: replace `tokio::time::timeout_at(deadline, conn.recv())` loop with a loop that breaks on `Inbound::Transport(TransportMessage::SyncComplete { .. })`. The runbook should sequence this as a single refactor commit before federation push lands so the four call sites stay in sync.
- The Node-side emission point is the end of `collect_sync_history` delivery in `app.rs:613-619`. After the last event of the history batch is sent, send `SyncComplete { since: request.since, new_tip: <current tip per Space>, continue_from: <cursor or null per F-7> }`.
- Cross-Space behaviour: a single `sync_request` with empty `since` covers all Spaces the requester is a member of (audit §4.3). The `new_tip` field in the response is therefore ambiguous if events span multiple Spaces. Two options for the runbook to consider: (a) emit one `SyncComplete` per Space with that Space's tip; (b) emit one `SyncComplete` for the whole batch with a map of `space_id → tip`. The design does not lock this — Clair's latitude with a recommendation that the runbook flag the choice explicitly with rationale. The same choice interacts with F-7 pagination (see §10.6).
- The 5-second default for `[sync].completion_timeout_seconds` is the reference-implementation default. The runbook should document the configurability and ensure both the Node and Client configs surface the field.
- Spec update: Ch3 §3.3.6 needs to be updated to reflect that `sync_complete` is no longer "deferred" but "shipped in this milestone." The runbook handles that in the documentation pass.

---

## 10. Framework decision F-7 — Pagination on `collect_sync_history`

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

### 10.1 The question

Audit §4.3 found that `collect_sync_history` (the Node-side handler for `sync_request`) returns **the complete event set in a single response** with no pagination, no size limit, no cursor for resumption.

For Phase 1 / Phase 2 scale, this works. A Space with 50 events sends 50 events; a Space with 500 sends 500. A reconnecting client whose Spaces contain millions of events would receive millions in a single response.

The audit flagged this as **LOW today, MEDIUM at scale**. Like F-6, this surfaces when the design extends `sync_request` Node-to-Node — federation pull-on-gap (F-1's gap-recovery mechanism) and the F-1a tip-exchange handshake both rely on `collect_sync_history` to deliver event ranges.

Two cases this milestone introduces will exercise the no-pagination behaviour:

1. **F-1a tip exchange at handshake.** For a brand-new relationship to a Space with long history, the response is the full history in one shot. Same shape as today's `handle_federation_incoming` dump, just under a different code path.
2. **F-1 pull-on-gap recovery.** Usually small (a few events between HeldPending gap and current tip) but pathological when a peer was offline for an extended period.

In both cases, response size is bounded by "everything since the requester's tip" — bounded by deployment activity, not enforced.

### 10.2 Options considered

**Option 1 — Fold in: implement response-size pagination in this milestone.**

Add a `limit` field to `SyncRequest` (or use an implementation-default if not specified) and a `continue_from` cursor in the response. The Node returns at most N events; if more are available, includes the cursor so the requester can fetch the next batch via a follow-up `sync_request`.

- ✅ **Bounds response size at the protocol level.** No more "million events in one message" pathological case.
- ✅ **Federation pull-on-gap gets predictable behaviour.** Large recovery cases are paginated; each batch is bounded.
- ✅ **Composable with `SyncComplete` (F-6).** `SyncComplete` becomes a per-batch signal; if `continue_from` is non-null, the requester knows to keep pulling.
- ⚠️ Adds protocol surface (two fields). Wire-shape change is small but real.
- ⚠️ Migration cost: the four existing `sync_request` callers need pagination loops.
- ⚠️ Page-size sizing decision.

**Option 2 — Defer: keep unbounded responses.**

Leave `collect_sync_history` as-is. Federation pull-on-gap inherits the unbounded behaviour. Future scaling milestone fixes it.

- ✅ Smaller milestone.
- ⚠️ Pathological case lands in federation (WebSocket frame size, memory pressure, connection blocked during delivery).
- ⚠️ The 5-second F-6b safety-net timeout becomes the bottleneck — a large response might take >5s to deliver, triggering the timeout for what's really a size problem, not a network problem.
- ⚠️ Pairs badly with F-6: F-6 closes the silent-failure mode; F-7-deferred re-opens a different silent failure (response too large, connection drops, requester sees F-6b timeout).

**Option 3 — Partial fold-in: size-bounded streaming, no explicit cursor.**

Node responds progressively; requester re-issues `sync_request` with the latest event_id as the new `since` if the previous response felt incomplete.

- ❌ Re-introduces "felt incomplete" heuristic — same shape of bug as F-6's 500ms quiet-time.
- ❌ Contradicts D-065. Honest behaviour wants the responder to say "here's the batch; here's the cursor to get more."

### 10.3 Decision — Option 1 (fold in)

**Implement response-size pagination in this milestone.** `SyncRequest` gains an optional `limit` field. The Node returns at most `limit` (or implementation-default) events per response. If more events are available, the `SyncComplete` message that ends the batch carries a non-null `continue_from` cursor. The requester issues a follow-up `SyncRequest` with `since: <continue_from>` to fetch the next batch. The loop continues until `SyncComplete` arrives with a null `continue_from`, meaning catch-up is complete.

**Reasoning recorded.**

1. **F-6 and F-7 are conceptually paired.** Both address "what happens when the response stream is bigger than the simple case." Solving F-6 alone leaves a related sharp edge unaddressed: a pathologically large response can hit F-6b's 5-second safety-net timeout for what's really a size problem.
2. **Coordinating the wire-protocol changes is cheaper now than later.** F-6 and F-7 both touch `SyncRequest` and `SyncComplete`. Adding the pagination fields in the same commit as the completion signal reduces churn on the four migration call sites.
3. **Pagination is the kind of thing that's much easier to design when the protocol is still forming.** Today there are four `sync_request` callers; in v2 there may be many more. Retrofitting later would require coordinating across all callers and risking missed migrations.

The scope cost is real — F-7 grows the milestone — but the cost is bounded (two wire fields, a pagination loop in four sites, one sizing default with config override) and pairs with work F-6 is already doing.

### 10.4 Sub-decision F-7a — Page-size policy

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

**Decision — implementation-configurable, reference-implementation default of 1000 events per batch. NOT protocol-fixed.**

Following F-6b's precedent. The protocol mandates the *mechanism* (paginated responses, `limit` and `continue_from` fields); the *value* is an implementation default with operator override.

**Concrete configuration.** Config field `[sync].batch_size` in both `xgen-node_config.toml` and `xgen-client_config.toml`. Default 1000 events per batch. Operator may override.

**Choice of 1000 as the default.**

- Large enough that most realistic catch-up cases finish in one batch (typical Space activity over hours-to-days is well under 1000 events).
- Small enough that individual response sizes stay well within reasonable WebSocket frame limits and serialise quickly.
- Round number with no magical significance — operators reading the config see "this is a sizing knob" rather than "this is a protocol constant."

**Reasoning recorded.** Same as F-6b. Hardcoding a page size into the protocol would force every deployment to compromise: LAN-only deployments could safely run with larger batches; constrained-bandwidth deployments might want smaller. Configurable from day one prevents repeating the "magic number bake-in" problem the milestone is already fixing for the 500ms quiet-time heuristic.

### 10.5 Wire-shape additions and pagination flow

The F-6 + F-7 wire changes compose as follows. The `SyncComplete` shape was already shown in §9.4 with the `continue_from` field; this section is the full picture including `SyncRequest`:

```
TransportMessage::SyncRequest {
    since: String,            // existing
    limit: Option<u32>,       // F-7 — optional; absent means implementation default
}

TransportMessage::SyncComplete {
    since: String,                  // F-6 — echo of the request's since cursor
    new_tip: String,                // F-6 — responder's current DAG tip for the relevant Space
    continue_from: Option<String>,  // F-7 — non-null when more events are available
}
```

**Pagination flow:**

1. Requester sends `SyncRequest { since: "X", limit: 1000 }`.
2. Responder sends up to 1000 events.
3. Responder sends `SyncComplete { since: "X", new_tip: "Y", continue_from: "Z" }` where `Z` is the event_id of the last event delivered (or null if all caught up).
4. If `continue_from` is non-null, requester sends `SyncRequest { since: "Z", limit: 1000 }` and the loop continues.
5. When the responder has nothing more to send, `SyncComplete.continue_from` is null and the requester knows catch-up is complete.

The `new_tip` field is informational — the requester uses it to confirm they're caught up to the responder's current state. The `continue_from` field is the authoritative pagination signal: null = done, non-null = call again.

### 10.6 Implementation-runbook notes from F-7

- Pagination flows through the same four call sites F-6 already touches (`batch.rs:83`, `ai_service.rs:224`, `ops.rs:721`, `ops.rs:939`). The runbook should sequence F-6 and F-7 as a single coordinated wire-protocol change rather than two separate commits — adding `SyncComplete` and the pagination fields together reduces churn and keeps the four migration call sites in sync.
- Each call site needs a pagination loop. Rough shape: `while continue_from is non-null: SyncRequest(since=continue_from, limit=...); collect batch; check SyncComplete.continue_from`. Reasonable to factor into a helper.
- Node-side: `collect_sync_history` needs to honour `limit` and emit `continue_from` correctly when the available event set exceeds it. Cursor semantics: `continue_from` is the event_id of the last event in the current batch; the next request asks for events after it.
- Cross-Space behaviour interacts with the F-6 cross-Space ambiguity (one `SyncComplete` per Space vs. one for the whole batch). If the runbook chooses per-Space `SyncComplete`, pagination is per-Space too and each Space has its own `continue_from`. If the runbook chooses whole-batch `SyncComplete`, pagination is across the combined event stream. The choice is Clair's latitude with a recommendation that whichever shape lands, the runbook documents it clearly.
- The 1000-events default for `[sync].batch_size` is the reference-implementation default. The runbook should document the configurability and surface the field in both Node and Client configs.
- No spec update needed beyond what F-6 already requires for Ch3 §3.3.6. The pagination fields are added in the same spec update that covers `sync_complete`.

---

## 11. Framework decision F-8 — Ch4 §4.11.3 + §4.12.3 correction timing

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

### 11.1 The question

The audit (§4.6 and §6.2) identified two specific paragraphs in `docs/xgen_ch4_implementation.md` that describe mechanisms that do not exist in code:

- **§4.11.3 "Event Fan-out"** — the paragraph beginning "Fan-out to federated peers wraps the Event in a transport frame..." describes a per-peer outbound queue and reconnect-driven `transport.sync_request` flow that the audit confirmed does not exist.
- **§4.12.3 "Pending Event Buffer"** — the paragraph beginning "Events that fail validation step 9..." describes Node-to-peer `transport.sync_request` for missing predecessors that the audit confirmed does not exist either.

Both are factual drifts where Ch4 describes what was intended rather than what exists. The correction itself is not in question; the question is when.

**Note on citations.** The audit doc cites these locations by line number (Ch4 lines 779 and 825-827 in its §4.6 and §6.2). Those line numbers reflect Ch4 as it stood at audit time and have shifted since due to subsequent edits. Pass 3 locates the drift by content match against the unique phrases "per-peer outbound queue" (§4.11.3) and "Node sends `transport.sync_request` to its peers for the missing predecessors" (§4.12.3). The audit doc's line-number citation is preserved as a historical record of where the drift was at J-081 close; Pass 3's task file and this section record the section-heading citations as the durable reference.

### 11.2 Options considered

**Option 1 — Correct during Pass 2.** Replace the drifted text now with either a reservation note or accurate "today's-behaviour" text. Two Ch4 edits across the milestone — once now, once when implementation lands.

**Option 2 — Correct at Pass 3.** When Pass 3 promotes the design doc to ACTIVE, do the Ch4 correction in the same commit. The Ch4 text gets replaced with text that forward-references the canonical design doc (`xgen_federation_propagation_design.md`) and acknowledges the mechanism is deferred to its implementing milestone.

**Option 3 — Correct at implementation runbook phase.** Leave Ch4 untouched through Pass 2 and Pass 3. The correction is part of the runbook's "documentation pass" step when the actual code lands.

### 11.3 Decision — Option 2 (correct at Pass 3, same commit as design doc ACTIVE promotion)

**The Ch4 correction is performed in the same commit that flips this document from PENDING to ACTIVE.** The corrected text becomes a forward-reference to this canonical design doc, honest about the implementation state ("specified in the federation propagation design; implementation follows in the corresponding milestone").

**Reasoning recorded.**

1. **Pass 3 is the natural publication moment.** That is when the design doc flips to ACTIVE and gets cross-referenced by everything else. Folding the Ch4 correction into the same commit means the cross-reference (Ch4 → design doc) is alive from the moment the design doc itself becomes authoritative.
2. **"Describes a deferred mechanism" is better than "describes a mechanism that does not exist."** Today's Ch4 text is misleading because it does not say "this isn't built yet." A Pass 3 correction that explicitly forward-references the design doc and acknowledges the deferred state is honest about the project's posture — consistent with D-065 (honest behaviour over polite behaviour).
3. **Pass 2 already has enough scope.** Adding Ch4 edits during Pass 2 means design-discussion turns also include "and now let me edit Ch4" detours. Better to keep Pass 2 focused on decisions and batch the documentation fix at Pass 3.

Option 3 has the appeal of "one move, end state is accurate" but the cost is real: weeks of misleading text in a load-bearing document. Option 1 fragments the Ch4 edits across multiple phases without strong benefit.

### 11.4 Correction principles

The exact rewrite is performed at Pass 3 close (the same commit that ships this document at v1.0 ACTIVE). The principles:

- **§4.11.3 (Event Fan-out) drift paragraph.** Replace the paragraph describing the per-peer outbound queue and reconnect-driven sync_request with a forward-reference: federation fan-out is specified in this canonical design doc; implementation lands in the federation propagation completion milestone; today's federation fan-out behaviour is the absent-mechanism state recorded in J-081 audit §2.
- **§4.12.3 (Pending Event Buffer) drift paragraph.** Replace the paragraph describing Node-to-peer sync_request for missing predecessors with a forward-reference: HeldPending recovery via federation pull-on-gap is specified in this document (F-1 + F-10); implementation lands in the federation propagation completion milestone; today's HeldPending behaviour is the local-client-recovery-only state recorded in J-081 audit §3.

The principle is: forward-reference the canonical design doc, acknowledge the deferred state, never describe behaviour as if implemented when it is not.

---

## 12. Framework decision F-9 — `xgen_node_admin_ops_design.md` §4.2 correction timing

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

### 12.1 The question

Audit §6.2 identified that `docs/xgen_node_admin_ops_design.md` §4.2 describes Node-to-Node federation in a way that does not match the actual code. The drifted text suggests federation push exists in some form; the audit confirmed it does not.

The structural question is identical to F-8: when is the correction made?

### 12.2 Options considered

The same three options as F-8 — correct during Pass 2, correct at Pass 3, correct at implementation runbook phase — apply here with the same trade-offs.

### 12.3 Decision — Option 2 (correct at Pass 3, same commit as design doc ACTIVE promotion)

**Same as F-8.** The §4.2 correction is performed in the same commit that promotes this document from PENDING to ACTIVE. The corrected text becomes a forward-reference to this canonical design doc.

**Reasoning recorded.** Structurally identical to F-8 §11.3:

1. Pass 3 is the natural publication moment for cross-references to the newly canonical design.
2. "Describes a deferred mechanism" is better than "describes a mechanism that does not exist," and consistent with D-065.
3. Pass 2 stays focused on decisions, not documentation-cleanup edits to adjacent docs.

### 12.4 Correction principles

`xgen_node_admin_ops_design.md` §4.2 currently describes Node-to-Node federation propagation (Stage 6) as "existing machinery." The audit confirmed this mechanism is architecturally absent. The Pass 3 rewrite replaces the Stage-6 sub-bullet with:

- A statement that Node-to-Node federation event propagation is specified in this canonical design doc (F-1 through F-7, F-10).
- A statement that implementation lands in the federation propagation completion milestone.
- A statement that, until that milestone closes, federation propagation does not occur as a production mechanism — peers receive a one-time history dump on handshake and the connection then closes.
- Where §4.2's surrounding context refers to specific federation-relationship admin verbs (M6 territory), retain those references — they belong in this doc as admin-ops design, and they couple correctly to the federation propagation work.

Exact phrasing is performed at Pass 3 close. Principle is the same as F-8: forward-reference, acknowledge deferred state, never describe behaviour as implemented when it is not.

---

## 13. Framework decision F-10 — DAG hole semantics on validation failure with unknown signer Identity

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

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

`[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`

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

This is recorded as a v1 limitation: F-10 + Identity replication assumes Identity replication is operationally healthy. If it is not, federation event ingestion stalls until it is. The implementation runbook surfaces this in Node-side observability so operators can see when Identity replication is the bottleneck.

### 13.7 Implementation-runbook notes from F-10

- HeldPending's trigger condition is the only thing that changes. The buffer, the timer, the retry path, the data structure, the discard-on-timeout behaviour — all unchanged.
- The retry trigger needs to watch for two arrival events: predecessor arrival (existing, from F-4) AND Identity record arrival (new). The Identity-arrival hook needs to fire when a new Identity record lands via replication.
- Integration test coverage should include: (a) Identity record arrives within timeout → event validates and ingests; (b) predecessors arrive within timeout, Identity record arrives later but still within timeout → event validates on second retry; (c) Identity record never arrives, timeout fires, event discarded, next sync re-delivers; (d) both predecessors and Identity record missing → event waits for both, validates when both arrive.
- The "Identity replication health" concern (§13.6) should be surfaced in Node-side observability. A metric like "events currently in HeldPending pending Identity record" exposed to the admin UI lets operators see when Identity replication is the bottleneck. Exact metric design is runbook's call.
- This decision implicitly couples the federation push milestone to Identity replication's reliability. The runbook explicitly calls this out so it does not surprise anyone debugging later.

---

## 14. Pass 3 closure notes

This document is the canonical Pass 3 artefact of the Federation Event Propagation milestone's Joe-locked design phase. It supersedes Pass 2's working state, which spanned a main doc at v0.6 plus three addenda (F-7, F-8/F-9, F-10).

**Pass 2 ran in conversation over 2026-05-18.** All ten framework decisions surfaced with `[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]` markers. The split into addenda was a Pass 2 efficiency move: once the main doc grew past ~70KB, full-file rewrite per F-item became disproportionately expensive, so F-7, F-8/F-9, and F-10 were written as standalone addenda.

**Pass 3 ran in the same-day session that followed Pass 2.** Five deliverables shipped in one coordinated commit:

1. Addenda consolidated into this canonical document as §10 (F-7), §11 (F-8), §12 (F-9), §13 (F-10). Addendum files deleted. Version bumped from v0.6 to v1.0 (first canonical version). Status flipped from PENDING to ACTIVE.
2. All `[JOE-LOCK]` markers walked to final form: `[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`.
3. F-8 and F-9 documentation corrections executed: `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 forward-referenced to this document; `docs/xgen_node_admin_ops_design.md` §4.2 forward-referenced to this document. All in the same commit.
4. Implementation runbook for Clair written at `tasks/FEDERATION_PROPAGATION_COMPLETION.md`. Status: ACTIVE on creation. Becomes the next-active task once this milestone block flips to ACTIVE in CLAUDE.md and ROADMAP.md.
5. CLAUDE.md and ROADMAP.md updated to reflect Pass 3 closure: Pass 2 task file (`tasks/FEDERATION_PROPAGATION_DESIGN.md`) flipped to COMPLETED; Pass 3 task file (`tasks/FEDERATION_PROPAGATION_PASS_3.md`) flipped to COMPLETED at session close; this milestone block's status flipped in the same commit as the rest of the work.

**No code changes during Pass 3.** Test count stays at 468. The next state change is Clair's Phase 1 commit (per the runbook), which flips Federation Event Propagation implementation from 🟡 PENDING to 🟢 PLAY.

**Cross-references at Pass 3 close:**
- `tasks/FEDERATION_PROPAGATION_PASS_3.md` — Pass 3 task file (COMPLETED at this commit).
- `tasks/FEDERATION_PROPAGATION_COMPLETION.md` — Implementation runbook for Clair (created at this commit, Status ACTIVE).
- `docs/xgen_propagation_reliability.md` (J-081) — Audit doc, archival; its line-number citations for Ch4 reflect pre-edit state and are durable as historical record.
- `docs/xgen_node_admin_ops_design.md` §4.2 — Forward-references this document (corrected at this commit per F-9).
- `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 — Forward-reference this document (corrected at this commit per F-8).
- D-065 (honest behaviour over polite behaviour) — Cited multiple times throughout (F-6, F-7, F-10).
- D-069 (Joe-locked design phase, canonical-document rule) — The discipline that produced this document.

---

## 15. Implementation Complete

The Federation Event Propagation milestone shipped across eight implementation phases plus one documentation pass between 2026-05-18 and the doc-pass close. Each phase produced one Clair commit (implementation) preceded by one Chat-Claude runbook subsection commit (implementation locks, Phases 3 onward) where Joe-lock-threshold questions surfaced. Per-phase shipped state:

| Phase | JOURNAL | Headline shipped |
|---|---|---|
| 1 | J-082 (2026-05-18) | `TransportMessage::SyncComplete` wire shape (F-6); `collect_sync_history` pagination + `continue_from` (F-7); four production-caller migrations to SyncComplete-driven pagination loops with F-6b 5 s safety-net timeout; `[sync]` config section on both binaries. 476 tests. |
| 2 | J-083 (2026-05-18) | F-4 `process_inbound` validation pipeline unification — Path A / Path B / Path C all reach event-handling code only via the unified validation core. HeldPending moved from `accept_message` to the validation core. 480 tests. |
| 3 | J-084 (2026-05-19) | F-1a federation handshake reshape to bilateral tip exchange (`tips` on Hello + Capabilities); `stream_federation_delta` builds + ingests `state.federation_add` under the a-i symmetry rule; receiver-side production path operational, initiator-side production caller reserved for Phase 5. 488 tests. |
| 4 | J-085 (2026-05-19) | F-1 federation event push (`apply_federation_push` sibling of `apply_fanout`); F-1b drop-on-peer-down with no outbound queue; F-5 origin gating via `EventOrigin::ReceivedViaFederation` parameter; `FederationPeerSenders` registry mirroring `ClientSenders`; R12-R15 Clair-latitude items (try_send / R14 log lines / register-on-ACTIVE / deregister-on-exit). 491 tests. |
| 5 | J-086 (2026-05-19) | F-1c per-peer operational record (`peer_records: HashMap<String, PeerOperationalRecord>` inside `FederationRegistry`, Joe-lock A3); reconnect scheduler in new `xgen-node::reconnect` module (60s tick + 15/30/60/120-min ladder capped + parallel detached `tokio::spawn` per due peer); `run_federation_session_post_handshake<S>` helper extracted to share between receiver-side `handle_federation_incoming` and new initiator-side `attempt_reconnect`; `process_inbound`, `stream_federation_delta`, identity-message handlers made generic over `S: AsyncRead+AsyncWrite+Unpin` to bridge `TcpStream` (server-accept) and `MaybeTlsStream<TcpStream>` (outbound connect); `run_initiating` gains its first production caller (closing audit §2.2). 505 tests. |
| 6 | J-087 (2026-05-19) | F-10 HeldPending generalisation: `ValidationOutcome::HeldPending { missing_predecessors, missing_identity }` struct variant; per-`PendingBuffer` `waiting_for_identity` secondary index with `NodeRuntime::drain_pending_by_identity` cross-Space fan-out (Lock A2); `pending_identity_replication` counter in `xgen-node_state.json` (Lock C2); new error code `4006 identity_record_timeout` (next-free after 4001-4005 — Step 6 namespace verification) with predecessor-code-wins sub-rule (Lock D); legacy `validate_steps_8_13` confirmed test-only-reachable. 516 tests. |
| 7 | J-088 (2026-05-19) | F-3 federation-relationship verification gate operational; `dispatch_event` gains `peer_node_id: Option<&str>` parameter (Lock C1); check consults `SpaceState.federation_nodes` (Lock A1, same source `apply_federation_push` uses on the outbound side, closing the symmetric pair); `state.federation_add` events skip F-3 (Lock B1) with verbatim code-comment block to allow relationship bootstrap. 519 tests. |
| 7.5 | J-093 (design, 2026-05-19) + J-094 (implementation, 2026-05-20) | Cold-Start Bootstrap milestone SHIPPED across five commits (1: doc-pass; 2: F-3 + F-4-step-1 skip + `SpaceLocalMetadata`; 3: HeldPending third trigger + `drain_pending_by_federation_relationship` + 4007 + counter + `f3_reject` disposition; 3.5: **Phase 7 B3 amendment** at `ecbbf19` closing two latent Phase-7 gaps surfaced by Commit 4 integration tests — predecessor-chain deadlock + step-11 sender-membership rejection for `state.federation_add` arriving via federation channel; B3 framed sibling-to-B1 not P7.5-A-extension; skip set widened to step-11 in full after Q3-overload code trace; 4: folded into 3.5 — six NodeRuntime-level integration tests green; 5: milestone-internal close). All four `[JOE-LOCK: locked 2026-05-19]` framework decisions (P7.5-A skip rule extension for `state.space_create` + `state.dm_space_create`; P7.5-B third HeldPending trigger; P7.5-C 180s federation-relationship timeout; P7.5-D `pending_federation_relationship` counter + `f3_reject` disposition extension) shipped per `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` (COMPLETED v1.0) and `tasks/FEDERATION_PROPAGATION_PHASE_7_5_IMPL.md` (COMPLETED v1.0). B3 amendment at `tasks/FEDERATION_PROPAGATION_PHASE_7_B3_AMENDMENT.md` (COMPLETED v1.0). Architectural notes: federation-relationship arrival hook lifted from `xgen-node::app::process_inbound` into `dispatch_event` Step 7 (mirror of Phase 6's Identity-hook architecture); `resolve_federation_relationship` gained `reindex_after_partial_release` helper preventing buffer-entry orphan under HashSet-order non-determinism (bug shape that surfaces only with three HeldPending triggers); `drain_timed_out` dropped implicit `.max(default_timeout)` so operator's configured timeout wins. 519 → 556 tests (+37). Failure-mode catalogue M5 structurally closed. |
| 8 | J-089 (doc-pass) | Six accumulated doc-vs-code drift surfaces closed (Ch4 §4.11.2 JSON-shape rewrite; Ch4 §4.11.3 + §4.12.3 + admin-ops §4.2 forward-references → implementation-complete; runbook §3.5 stale framing corrected; CLAUDE.md Tier-1 file table; this section §14/§15 implementation-complete note; spec §3.3.6 wire-shape rewrite + §3.9.6 + §3.9.8 4006 entry; this §6.4 + §6.5 + §6.8 federation_nodes clarification + B1 implementation note). Test count unchanged at 519 (documentation only per Phase 8 DoD). |
| Bidirectional `federation_nodes` | J-096 (2026-05-21) | Sibling to Phase 7.5: closes the second cold-start gap surfaced by Phase 9 Commit 3a Scenario 1 — `apply_federation_add` populated `federation_nodes` with the wrong Node on the receiver vantage, causing F-3 to reject every post-bootstrap push event. Three Joe-locks per `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` (COMPLETED v1.0): Q1 Reading (i) (one event, asymmetric interpretation; D-075 promoted to DECISIONS.md), Shape A (origin-aware applier with new `my_node_id: &str` parameter; wire format unchanged), sub-option A.1 (re-derive on load; native fit verified against `SpaceState` non-persistence model). Four atomic commits per `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`: doc-pass → applier + plumbing + unit tests → Phase 9 Scenario 1 resurrection (`#[ignore]` lifted) → milestone close. Wire-format-neutral, Pass-1-neutral. Test count baseline 571 (post-J-095 XGID Adoption v1) + N new unit tests + 1 resurrected scenario = 572 + N at milestone close. |
| Topological-sort wire-order determinism | J-101 (2026-05-23) | Sibling to Phase 7.5 and Bidirectional `federation_nodes`: closes a separate pre-existing wire-order non-determinism surfaced by Phase 9 Commit 3a Scenario 1's post-bidirectional verification (J-096 Finding 2). `topological_sort_events` at `xgen-node/src/fanout.rs:193` preserved input-vector order for ready siblings (DAG roots with empty `prev_events`); its caller `compute_federation_delta_for_space:321` fed it via `HashMap.values().cloned().collect()` with randomized iteration. Two senders with identical Space state produced different federation-delta wire orderings ~50% of runs; when `state.room_create` raced `state.space_create`, B's dispatch Step 1 rejected with "space not found" and bootstrap cascaded. Three Joe-locks per `tasks/FEDERATION_TOPOSORT_DESIGN.md` (COMPLETED v1.0): Q3.ii canonical wire ordering required (sender-side normative property; two senders with identical state MUST produce byte-identical federation deltas modulo signature-bearing fields), Q2 middle + Q2.γ (fix the primitive's contract once; forward-bind to Node-to-Client siblings `collect_sync_history` + `apply_fanout` history-push for future scheduling), Q1 Shape A v1 + sibling Site 1 fix (event_id lex sort at the primitive + sort `Vec<Event>` at `compute_federation_delta_for_space:321` before passing; v1 `&str` sort with code-comment block flagging Pass 3 retype to `EventXgid`). **D-076 promoted to DECISIONS.md** as the protocol-design principle the locks instantiate; fourth member of the no-drift-surface discipline family (D-067 code-organisation + D-070 transport-layer + D-075 event-model + D-076 wire-format). Four atomic commits per `tasks/FEDERATION_TOPOSORT_IMPL.md`: doc-pass → primitive + sibling fix + unit tests → Phase 9 Scenario 1 second `#[ignore]` lift (same scenario that surfaced the finding becomes the activating regression lock for both D-075 and D-076 at integration level) → milestone close. Wire-format-neutral, Pass-1-neutral. Three-to-five unit tests at unit level including `compute_federation_delta_byte_identical_across_two_senders` as the wire-order-determinism witness (sibling-in-shape to bidirectional's `apply_federation_add_two_vantages_mirror`). Test count baseline 577 (post-J-096 bidirectional milestone close) + N new unit tests + 1 resurrected scenario = 578 + N at milestone close. Phase 9 Commit 3b unblocks at milestone close (Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10, ~5-7 atomic commits in their own sequence); Federation Event Propagation milestone stays PLAY until Phase 9 completes. |
| Persistence amendment | [J-108] (2026-05-24) | Drain-without-persist gap closure across `xgen-core::node::runtime`'s three drain helpers (`drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship`) and `xgen-node::app::process_inbound`'s persist site. `NodeRuntime::ingest_event` keeps binary-void signature + `tracing::error!` at the `graph.add_event` silent site (Q1 a.iii.α — reverted from (a).iii.β at re-walk Track 1 J-107 per Y-lock on cross-milestone Phase 7 B3 amendment dependency: B3's `state.federation_add` arriving via federation channel intentionally has missing predecessors, and B3 implicitly relied on the silent-discard at this site as a feature; Result-propagation would have broken B3 at the SpaceState mutation layer); `replay_spaces_from_dir` at `xgen-node/src/app.rs:2628` sorts events topologically before ingest (Q1 a.ii defensive layer, retained). `DispatchOutcome::Accepted` gains `additional_persisted: Vec<Event>` (Q2 return-vector aggregation); all three drain helpers return `Vec<Event>` and `dispatch_event` aggregates at three call sites (Q3 same-family-same-atomic-close, layered-B3 second project-wide instance). Sentinel-tree (four files at `xgen-node/src/tests/`: `phase9_harness.rs`, `phase9_three_node_anti_transitivity.rs`, `phase9_drop_and_recover.rs`, `mod.rs`) ships atomic at Commit 3 + flips at this milestone close as activating integration-level regression lock (Q4(a)); Commit 3b-1 collapses into this sub-amendment milestone close per Q4(a). Five-commit Clair-facing sequence per `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` (COMPLETED v1.2): Commit 1 doc-pass (`0ca29e6`) → Commit 2 Q1 ingest-path (`f4f0e4e`) → Commit 2a Q2+Q3 dispatch+persist (`c88fd73`) → Commit 3 sentinel-tree refinement + verify (`a677244`, 5 isolated + 3 workspace = 8 green runs minimum) → Commit 4 milestone close (this commit). Plus re-walk Track 1 (`b9a30da` at J-107) which amended in-place to (a).iii.α + promoted D-077 "bidirectional sustainability discipline" to DECISIONS.md (at every silent-discard / conditional-mutation / fallible-discard pattern, sustainability question MUST be asked in both directions — forward-drift AND backward-coherence — before locking any fix; meta-layer discipline above D-067 + D-070 + D-075 + D-076 v1.1 protocol-layer no-drift-surface family). Candidate D-NNN "ingest path invariant encoding under bidirectional sustainability discipline" stays flagged-not-promoted in this milestone at design doc §8 (rungs above (a).iii.β — ValidatedEvent wrapper, sealed traits + visitor pattern, formal verification); scope expanded at J-107 to cover five `ingest_event` silents + three drain helpers + M6 reject paths + Phase 7 B3 apply_event dependency. Wire-format-neutral, Pass-1-neutral. Workspace test delta across milestone: +7 (Commit 2 +2; Commit 2a +5; Commit 3 +0 unit tests because §3 work is harness-refinement, sentinel tests themselves are the regression locks). 8/8 GREEN runs at Commit 3 verification (xgen-core 431 / xgen-node 68 / xgen-common 24 / xgen-client 47 / integration buckets 7,6,5,2,1,1). |
| 9 | (pending) | Deployment-level integration tests covering the six DoD scenarios (two-Node push smoke, three-Node anti-transitivity, drop-and-recover, validation-asymmetry regression, unknown-signer first-contact, federation-relationship rejection). |

**Architectural outcomes recorded.**

- The "missing mechanism" verdict from audit J-081 §2 (Stage 6 federation propagation architecturally absent) is closed. Federation event push exists as a production mechanism with proper F-5 anti-transitivity, F-1b drop-on-peer-down, F-1c reconnect scheduling, and F-3 receive-side gating.
- The validation-asymmetry concern from audit J-081 §3 (Paths B/C bypassing signature + timestamp checks) is closed. All event families now flow through the F-4 unified validation core; the F-10 generalisation handles federation first-contact events whose Identity records have not yet replicated.
- Two protocol-design principles surfaced and landed in DECISIONS.md alongside the implementation: D-070 ("Two events of equal importance, opposite direction" — acceptance + rejection signals with envelope `event_id`, coordinated with M6 Phase 2's wire-layer signal) and D-071 ("Subsystem audits precede dependent milestones" — the discipline that produced the J-081 audit before this implementation milestone went ACTIVE).

**Cross-references at milestone close:**

- `tasks/FEDERATION_PROPAGATION_COMPLETION.md` — Implementation runbook (per-phase §3.1 through §3.9 + Joe-lock subsections §3.3.1 / §3.4.1 / §3.5.1 / §3.6.1 / §3.7.1).
- `docs/xgen_propagation_reliability.md` (J-081 audit) — Pre-implementation audit, archival reference.
- `docs/xgen_ch3_specification.md` §3.3.6 + §3.9.6 + §3.9.8 — Spec-side wire-shape + error-code definitions updated to shipped state at Phase 8 doc-pass.
- `docs/xgen_ch4_implementation.md` §4.11.2 + §4.11.3 + §4.12.3 — Implementation-doc-side coverage updated to shipped state at Phase 8 doc-pass.
- `docs/xgen_node_admin_ops_design.md` §4.2 — Admin-ops doc-side coverage updated to shipped state at Phase 8 doc-pass.
- JOURNAL entries J-082 through J-089 — Per-phase shipped-state record. J-088 closes Phase 7; J-089 closes Phase 8 (this section); J-090 will close Phase 9 when it ships.

---

*End of document. Pass 3 complete; design phase closed; runbook handoff to Clair effective at commit. Implementation Complete recorded post-Phase-8 doc-pass.*  
