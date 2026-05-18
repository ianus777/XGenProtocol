# XGen Propagation Reliability Audit
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-18 (audit closed; all five sections + close-out summary verdict-locked by Joe)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this document is

This document is the canonical reference for how an accepted DAG event propagates through the XGen reference implementation, and what guarantees that propagation actually provides. It is the deliverable of the Propagation Reliability Audit milestone, opened on 2026-05-18 at M6 Phase 0 Pass 3 close.

It exists because Pass 3 of M6 Phase 0 (see `docs/xgen_node_admin_ops_design.md` §3) locked `TransportMessage::EventAccepted` with a **G2 semantic** — *"event is in home Node's authoritative DAG store."* The G2 claim is meaningful only if events that reach DAG-resident status in the home Node reliably propagate to the rest of the system: other locally-connected members, federation peers, disconnected clients on reconnect. Before M6 ships the accept signal, the propagation mechanism it implicitly trusts must be verified.

The audit walks the five stages after originator submission, codified in the design doc §4.1 lifecycle. Each stage section answers the questions enumerated in `tasks/PROPAGATION_RELIABILITY_AUDIT.md` §3 with **code-grounded evidence** — file:line citations and quoted code — and closes with an explicit verdict of one of:

- **VERIFIED WORKING** — mechanism exists, is correct as far as the audit can tell, consistent with what the design doc claims.
- **GAP IDENTIFIED** — mechanism does not exist or is incomplete; gap description + recommended follow-on.
- **PARTIALLY VERIFIED** — some questions answered with confidence, others uncertain; uncertain pieces explicitly named.

**This document is written section-by-section under a Joe-approval gate** (`tasks/PROPAGATION_RELIABILITY_AUDIT.md` §5.3). Each stage section is written, the verdict drafted, then the document pauses for Joe to approve the verdict before the next section is written. A `PARTIALLY VERIFIED` or `GAP IDENTIFIED` verdict is preferable to a falsely-claimed `VERIFIED WORKING`.

**This document is not a fix milestone.** No code changes ship as part of this audit. Where a gap is identified, the recommended follow-on is described but not implemented; the fix lands in a separate downstream milestone.

---

## The lifecycle being audited

Verbatim from `docs/xgen_node_admin_ops_design.md` §4.1:

```
[1] Originator submits Event over WS to home Node
[2] Home Node runs 13-step validation pipeline (Ch3 §3.7)
[3] Home Node writes Event to local event store
[4] Home Node sends TransportMessage::EventAccepted → originator   ← G2 boundary (M6 ships this)
        ╔═════════════════════════════════════════════════════════════════╗
        ║   Stages 5+ are asynchronous from the originator's perspective. ║
        ║   The originator's G2 claim ("event is in home Node's store")   ║
        ║   is true and stable from this point.                           ║
        ╚═════════════════════════════════════════════════════════════════╝
[5] Home Node fans out to other locally-connected members (apply_fanout)
[6] Home Node propagates to federated peer Nodes
[7] Federated peers ingest into their own DAGs and fan out to their members
[8] Disconnected clients catch up on next sync_request
```

This audit confirms or refutes the reliability of stages 5, 6, 7, and 8 — plus §6 on `TransportMessage::Error` propagation scope. Stages 1–4 are M6's own work; the audit assumes they will be implemented per the design doc.

---

## §1 Stage 5 — Local fan-out

**Scope of the question.** When the home Node accepts an event into its DAG, what guarantees does the system provide that the event reaches other connected members of the event's Space?

### 1.1 What `apply_fanout` does

The Stage-5 entry point is [`apply_fanout`](xgen-node/src/fanout.rs:81) in `xgen-node/src/fanout.rs:81-137`. The runtime calls it once per accepted inbound event at [`app.rs:637`](xgen-node/src/app.rs:637), immediately after the connection's `process_inbound` returns a `FanoutRequest`:

```rust
let fanout = process_inbound( /* ... */ ).await;
apply_fanout(fanout, &identity_id, &runtime, &client_senders).await;
```

The function (fanout.rs:81-137) executes in three discrete steps:

1. **Resolve the Space ID** ([`fanout.rs:91-94`](xgen-node/src/fanout.rs:91)). `state.space_create` and `state.dm_space_create` events carry empty `space_id` and use their own `event_id`; all other events carry `space_id` explicitly. If neither lookup yields a Space ID, fan-out returns silently.
2. **Snapshot recipients + optional history under the runtime lock** ([`fanout.rs:97-117`](xgen-node/src/fanout.rs:97)). Recipients = `space.members.keys()`. For brand-new joiners, the function additionally snapshots the Space's full event history in topological order, excluding the join event itself.
3. **Push to each non-author recipient under the senders lock** ([`fanout.rs:119-137`](xgen-node/src/fanout.rs:119)). For each recipient identity, look up the `mpsc::Sender<OutboundMsg>` in the shared `ClientSenders` map; if present, call `tx.try_send(OutboundMsg::Event(event.clone()))`. For new joiners with a non-empty history, push a single `OutboundMsg::HistoryBatch { events: history }`.

The runtime lock is dropped before the senders lock is acquired — a deliberate ordering chosen to keep critical sections short and prevent fan-out from blocking other connection handlers (fanout.rs:78-80 docstring).

The consumer side of each per-connection mpsc lives in [`app.rs:580-604`](xgen-node/src/app.rs:580): the connection handler's `tokio::select!` loop drains `out_rx` and writes each `OutboundMsg::Event` (or each event in a `HistoryBatch`) directly to the WebSocket via `conn.send_event(&ev)`.

### 1.2 Channel-full behaviour

Per-connection mpsc bound is **1024** ([`app.rs:562-563`](xgen-node/src/app.rs:562)):

```rust
let (out_tx, mut out_rx) =
    tokio::sync::mpsc::channel::<OutboundMsg>(1024);
```

The fan-out push at [`fanout.rs:126`](xgen-node/src/fanout.rs:126) uses `try_send` and discards the result:

```rust
if let Some(tx) = senders.get(rid) {
    let _ = tx.try_send(OutboundMsg::Event(event.clone()));
}
```

`try_send` returns `Err(TrySendError::Full)` when the buffer is at capacity. Because the result is bound to `_`, the error is silently dropped — **the event is not delivered to that recipient on this Stage-5 attempt**, no retry happens at Stage 5, no log line is emitted, and no metric is incremented. The same pattern applies to the joiner-history push at [`fanout.rs:133`](xgen-node/src/fanout.rs:133).

The recovery path for a dropped event is Stage 8 (sync catch-up on reconnect) — the missed event is in the home Node's DAG and will be served on the recipient's next `sync_request`. So the *long-run* delivery guarantee is preserved by the sync layer; the *real-time* delivery guarantee is "best-effort, drop on backpressure, no observability." Audited separately as §4.

This is consistent with D-065 ("honest behaviour over polite behaviour") in form — the system does not queue indefinitely or block the writer — but the silent drop pattern weakens the "honest" half: a Node operator inspecting logs sees no record of which events were dropped to which recipients. This is a minor observability gap, not a correctness gap.

### 1.3 Disconnected-recipient behaviour

A recipient that has no live WebSocket connection has no entry in the `ClientSenders` map. The map is mutated by the connection handler at [`app.rs:566`](xgen-node/src/app.rs:566) (insert on authentication) and [`app.rs:653`](xgen-node/src/app.rs:653) (remove on disconnect):

```rust
let mut senders = client_senders.lock().await;
senders.remove(&identity_id);
```

`apply_fanout` iterates `recipients` (the Space's member list — from the persistent SpaceState, not from connection state) and gates each push with `if let Some(tx) = senders.get(rid)`. A member without a current `Sender` entry is silently skipped — no error, no buffer, no retry. This is the intended behaviour and is locked by the test [`fanout_skips_disconnected_recipients`](xgen-node/src/fanout.rs:495) at lines 495-520.

The disconnected recipient catches up via Stage 8 sync on reconnect, identically to the channel-full case.

### 1.4 Retry mechanism

**None at Stage 5.** Search of `xgen-node/src/fanout.rs` and the surrounding fan-out call site in `app.rs` shows no retry queue, no per-recipient backoff, no pending-delivery store. The "PendingBuffer" mentioned in M5-era project memory ([`project_session_12.md`](C:/Users/Joe/.claude/projects/E--Projects-XGenProtocol/memory/project_session_12.md)) is a *runtime* pending buffer for events whose `prev_events` reference unresolved predecessors during ingest — that is an ingest-side mechanism (NodeRuntime), not a fan-out retry mechanism. Stage 5 is fire-and-forget by design; per-event delivery loss is recovered by Stage 8.

### 1.5 Author-exclusion rationale (J-080 finding, re-confirmed)

The fan-out loop excludes the event's author at [`fanout.rs:121-124`](xgen-node/src/fanout.rs:121):

```rust
for rid in &recipients {
    if rid == author_id {
        continue;
    }
    if let Some(tx) = senders.get(rid) {
        let _ = tx.try_send(OutboundMsg::Event(event.clone()));
    }
}
```

**There is no inline comment at this code site explaining why the author is excluded.** The exclusion is locked behaviourally by the test [`message_fans_out_to_other_members_and_excludes_author`](xgen-node/src/fanout.rs:339) at lines 339-380, added in J-067 (F-001) as part of the original local fan-out implementation.

J-080 (carry-over of `cmd_create_space` optimistic-ack) searched the codebase for the rationale and found it recorded only in a code comment inside a *different* test — [`new_joiner_receives_full_history_push`](xgen-node/src/fanout.rs:382), at lines 467-469:

```rust
// Carol receives one HistoryBatch with prior events (Space, Room,
// Bob invite, Bob join, Alice message, Carol invite). The join event
// itself is excluded (Carol's client already has its own outbound copy).
```

The "Carol's client already has its own outbound copy" rationale was authored to explain why the *joiner's own join event* is filtered out of the history push at [`fanout.rs:108-110`](xgen-node/src/fanout.rs:108) — a different code path from the main author-exclusion loop. The Pass-3 input addendum in `tasks/NODE_ADMIN_PASS2_PROPOSALS.md` §"Pass-3 input: missing protocol accept signal" then *infers* the same rationale extends by analogy to the main author-exclusion ("duplicate-avoidance UX, not protocol-correctness"). The inference is reasonable — the author already has the event locally because they just sent it — but it is an inference layered onto a test comment about a different filter, not a comment at the point of the main exclusion code.

**Implication.** The rationale is genuinely *unrecorded at the protocol layer*. It is enforced by a test name. The Pass 3 framing of D-070 (proposed) treats this as an asymmetry-by-accident that M6's `EventAccepted` closes: the author had no positive signal because nobody designed one, and the no-self-echo behaviour is the by-design consequence of that absence. Once `EventAccepted` ships, the author-exclusion behaviour is the right default (the originator gets a dedicated accept message, not a self-echo of their own event); the design call should be recorded explicitly at that point, not left as a test name.

### 1.6 Verdict

**PARTIALLY VERIFIED.**

| Question | Status |
|---|---|
| What `apply_fanout` does | VERIFIED — code-grounded, well-tested, behaviour matches design-doc claim. |
| Channel-full behaviour | VERIFIED with caveat — fire-and-forget, silent drop, no observability. Recovery via Stage 8. |
| Disconnected-recipient behaviour | VERIFIED — silent skip, recovery via Stage 8. |
| Retry mechanism at Stage 5 | VERIFIED ABSENT — by design; no retry exists, recovery is Stage 8. |
| Author-exclusion rationale | PARTIALLY VERIFIED — behaviour is correct and locked by a test, but the documented rationale lives only in a code comment about a *different* code path. The main author-exclusion has no inline rationale and no DECISIONS.md entry. |

**The Stage-5 mechanism itself is correct.** Events delivered through `apply_fanout` to recipients whose channels have headroom are reliably written to their per-connection WebSocket; the data-shape and routing decisions are unit-tested. Three real-time loss surfaces exist (channel-full silent drop, disconnected recipient silent skip, author exclusion); two are by-design and rely on Stage 8 to provide eventual delivery, and the third is intentional (originator does not self-echo).

**Two observations** that do not affect the verdict but are recorded here for downstream attention:

1. **Observability gap.** The silent-drop pattern in §1.2 leaves no record of dropped Stage-5 deliveries. A future operator-facing diagnostic ("which recipients missed which real-time deliveries?") is not currently possible from logs. Severity: **LOW** — Stage 8 reliably recovers; this is a debuggability concern, not a correctness one. Recommended follow-on: a counter on `try_send` failure paths (no DECISIONS.md entry needed; implementation detail).

2. **Documentation gap on author-exclusion.** The rationale should be recorded somewhere durable — either as an inline comment at [`fanout.rs:121-124`](xgen-node/src/fanout.rs:121), or (preferably, given the D-070 framing) folded into the D-070 promotion text that ships when this audit closes. Severity: **LOW** — behaviour is locked by tests; the gap is documentation, not behaviour. Recommended follow-on: include this rationale in the D-070 promotion entry.

Both observations are flagged here so Joe can decide severity classification per `tasks/PROPAGATION_RELIABILITY_AUDIT.md` §4.2 before any follow-on tracking is filed.

---

## §2 Stage 6 — Node-to-Node federation propagation

**Scope of the question.** When the home Node accepts an event into its DAG, what guarantees does the system provide that the event reaches *federated peer Nodes* — separate Node processes that share the Space?

This section is the audit's PRIMARY investigation per `tasks/PROPAGATION_RELIABILITY_AUDIT.md` §3.2. Joe has locked the direction in advance: whatever the audit finds, the project will close any gap properly before M6 proceeds (per the 2026-05-18 "honest longer work over fast shortcuts" framing). The traces below describe reality so the gap can be addressed, not so a path forward can be picked.

### 2.1 The federation surface that exists today

The Node implementation contains one and only one production federation-handler code path: [`handle_federation_incoming`](xgen-node/src/app.rs:665) in `xgen-node/src/app.rs:665-799`. The handler runs when an inbound WebSocket carries a `FederationMessage::Hello` ([`app.rs:519`](xgen-node/src/app.rs:519) match arm). Its full lifecycle:

1. Verify the peer's signed `hello` ([`app.rs:674`](xgen-node/src/app.rs:674)).
2. Negotiate capabilities + version, send signed `federation.capabilities` ([`app.rs:691-715`](xgen-node/src/app.rs:691)).
3. Receive signed `federation.accept` ([`app.rs:718-728`](xgen-node/src/app.rs:718)).
4. Receive `space.join_request` carrying the peer's target `space_id` ([`app.rs:730-738`](xgen-node/src/app.rs:730)).
5. Snapshot the Space's full event history + DAG tips atomically under the runtime lock ([`app.rs:741-744`](xgen-node/src/app.rs:741)).
6. Build + sign a `federation_add` event ([`app.rs:746-758`](xgen-node/src/app.rs:746)) — ingest it locally, persist it.
7. Stream the snapshotted history in topological order, followed by the `federation_add` event, then `goodbye("history_sync_complete")` ([`app.rs:779-790`](xgen-node/src/app.rs:779)):

```rust
let total = history.len() + 1;
for ev in &history {
    trace_event(ev, EventDirection::Out, &fed_session_ctx);
    if conn.send_event(ev).await.is_err() {
        return;
    }
}
trace_event(&fed_add_ev, EventDirection::Out, &fed_session_ctx);
if conn.send_event(&fed_add_ev).await.is_err() {
    return;
}
let _ = conn.goodbye("history_sync_complete").await;
```

8. Optionally record the peer's WebSocket endpoint URL in `NodeRuntime::peer_urls` ([`app.rs:792-796`](xgen-node/src/app.rs:792)) — for later use by the identity-replication subsystem (§2.3 below).
9. Emit a single `tracing::info!` line: *"Federation established"*, *"events_sent = total"*.

**The peer connection closes after step 7.** The connection is not held open. The peer is given the Space's history-at-handshake-time and the federation relationship is recorded — and that is the end of the propagation contract. No outbound queue is established, no per-peer event subscription is registered, no callback into `apply_fanout` knows that this peer exists.

The `FederationRegistry` (`xgen-core/src/federation/registry.rs`) stores the relationship persistently — `peer_node_id`, `shared_spaces`, `session_id`, `last_connected`, `peer_url` — so the *fact* of federation survives Node restart. But the registry is consulted only by `push_identity_to_peers` (§2.3); no event-emitting code path reads it.

### 2.2 Trace 1 — Does the home Node ever re-initiate the handshake?

**No production code path calls `run_initiating`.**

A grep of `xgen-node/src/` for `run_initiating` returns:

| Location | Kind |
|---|---|
| [`xgen-node/src/tests/smoke.rs:190`](xgen-node/src/tests/smoke.rs:190) | Test (Phase-1 17-step smoke) |
| [`xgen-node/src/tests/federation_integration.rs:46`](xgen-node/src/tests/federation_integration.rs:46) | Test |
| [`xgen-node/src/tests/federation_integration.rs:93`](xgen-node/src/tests/federation_integration.rs:93) | Test |

A grep for outbound `FederationMessage::Hello` construction in `xgen-node/src/` returns:

| Location | Kind |
|---|---|
| [`xgen-node/src/app.rs:680`](xgen-node/src/app.rs:680) | Pattern-match of *received* Hello (in `handle_federation_incoming`) |

The Node's production code never constructs an outbound `FederationMessage::Hello`. Federation handshake initiation is exclusively driven by external actors — in production: by other Nodes connecting in; in tests: by the test harness running `run_initiating` in-process. The home Node is purely the receiving side.

There is no scheduler, no periodic task, no on-event hook, no on-tip-change hook, no `tokio::spawn` anywhere in `xgen-node/src/app.rs` that would trigger an outbound handshake re-establishment after the initial peer connection closes. The Node has the `peer_urls` data ([`xgen-core/src/node/runtime.rs:56`](xgen-core/src/node/runtime.rs:56)) it would need to re-connect — but no code that does so for event-federation purposes.

### 2.3 Trace 2 — Is there any code path where peer Nodes pull from home?

**No event-pull mechanism between Nodes exists.**

Two outbound-to-peer code paths exist in `xgen-node/src/app.rs`, neither of which moves Space events:

1. **[`push_identity_to_peers`](xgen-node/src/app.rs:1100)** at lines 1100-1170. Triggered when an identity registration completes ([`app.rs:988`](xgen-node/src/app.rs:988)). For each peer URL in `runtime.peer_urls`, spawns a task that opens a *client* WebSocket to the peer (`connect_url` + `client_authenticate`), sends an `IdentityReplicateMessage::Replicate`, waits for `ReplicateAck`, records the replica. This is the identity-replication subsystem (Layer 18 / replica registry). It is the only production "Node sends WS to another Node" code path. **It does not handle Space events. It does not establish a federation session. It does not call `run_initiating` or any federation-handshake function.** It uses the regular client-authentication path with the Node's keypair acting as a client.

2. **`handle_federation_incoming`** (already traced in §2.1). Receiving side only; sends a one-time history dump then closes.

A grep for `space.join_request` / `SpaceControlMessage::JoinRequest` in `xgen-node/src/` returns:

| Location | Kind |
|---|---|
| [`xgen-node/src/app.rs:732`](xgen-node/src/app.rs:732) | *Receiving* a JoinRequest in `handle_federation_incoming` |
| [`xgen-node/src/tests/smoke.rs:156`](xgen-node/src/tests/smoke.rs:156) | Test (receive) |
| [`xgen-node/src/tests/smoke.rs:201`](xgen-node/src/tests/smoke.rs:201) | Test (send) |

No production code in `xgen-node` *sends* a `JoinRequest`. No "fetch events since X" or "give me your tips" Node-to-Node request type exists at the wire-protocol level either. The pull-from-home direction does not exist.

### 2.4 Trace 3 — What does the stress test's "Federation Completeness" actually measure?

**The stress test simulates federation via a one-time client-side history relay at setup, and its completeness metric measures only each Node's local-clients delivery, not cross-Node propagation.**

The relay code lives in `xgen-client/src/app.rs:2742-2786` (`cmd_stress_test` Phase 3a). The pattern:

```rust
// Spawn an ephemeral "fed_key" client. Connect to Node A, authenticate, run
// run_initiating handshake, send space.join_request. Receive history events
// into a local `history` Vec until goodbye/Closed.
let fs = run_initiating(&mut fc, &fed_key, FederationCapabilities::default(),
    vec![fed_sid.as_ref().clone()], None).await.context("fed: handshake")?;
// ... receive loop fills `history` ...

// Then: connect to Node B with the same ephemeral key, register identity,
// replay each historical event as a CLIENT send_event.
let mut bc = connect_url(&fed_nb).await.context("fed: connect B")?;
bc.client_authenticate(&fed_key).await?;
// ... register fed_key on B ...
for ev in &history {
    bc.send_event(ev).await.ok();
    comm_event(&fed_log,&fed_seq,"fed_join","federation","SENT",ev,&fed_nb);
}
let _ = bc.goodbye("fed_forward_done").await;
```

This is a Client-side relay, not a Node-to-Node propagation. The "federation" happens because a test client opens connections to both Nodes and copies events from one to the other. Once the relay's `goodbye("fed_forward_done")` fires, the relay disappears.

Phase 4 (message flood, lines 2900+) then has members send their `message.text` events to whichever Node they were assigned to (`assigned_node_url(i, members, ...)` splits members 50/50 between Nodes A and B). **There is no mechanism in the test, and no mechanism in the Node implementation, that propagates a Phase-4 message sent by a member on Node A to Node B.**

The "Federation Completeness" check at [`xgen-client/src/app.rs:3042-3045`](xgen-client/src/app.rs:3042) confirms this:

```rust
let fed_a_expected = (members / 2) * mpm;
let fed_b_expected = (members - members / 2) * mpm;
let fed_a_ok = node_a_applied >= fed_a_expected;
let fed_b_ok = node_b_applied >= fed_b_expected;
```

`fed_a_expected = (members / 2) * mpm` is the count of messages originated by the half of members assigned to Node A. The check is `node_a_applied >= fed_a_expected` — Node A's `apply_event message.text` log-line count must be ≥ the count of messages from Node A's own locally-connected members. If federation propagated Node B's messages onto Node A, the count would be roughly `members * mpm` (double the expected). The check does not require that.

So a PASS on "Federation Completeness" in the J-059 stress run (6/6 PASS, 43/43 checks per `docs/tests/STRESS_TEST_complete.md` and `JOURNAL.md` J-059) is consistent with — and indeed expected from — a system with no ongoing Node-to-Node event propagation. The label is misleading; the metric measures local-clients delivery completeness.

The stress test does report `direction=IN Node A: {n} events applied` (e.g. `cmd_stress_complete` Scenario 0 check at [`app.rs:3918-3921`](xgen-client/src/app.rs:3918)) as an info-only `sc_check!(0, ..., true, "")` — the count is recorded but no threshold is asserted. So even a count of "zero peer-originated events on Node A" would not fail the test.

### 2.5 Synthesis — answers to the design doc's §4.3 questions

`docs/xgen_node_admin_ops_design.md` §4.3 lists three open questions that this audit was opened to answer. The three traces above resolve them:

**Q1. Federation send buffering — does the Node buffer outbound federation events across WS reconnects, or are events emitted during disconnect lost from the federation path?**

Neither. There is **no outbound federation-event path at all** to which the buffering question applies. Events emitted in any direction (during connect, during disconnect, during steady state) are not forwarded to peer Nodes by any production code. The buffering question is downstream of an outbound-push mechanism that does not exist.

**Q2. DAG-tip reconciliation between federated peers — sync_request-style at the Node-to-Node layer, or pure real-time push?**

Neither. There is no real-time push (Q1) and no reconciliation mechanism. After the initial `handle_federation_incoming` history dump completes, neither Node has a code path that compares tips with the other, requests missing events, or re-syncs on any trigger. The `FederationRegistry` stores enough information (`peer_url`, `session_id`, `shared_spaces`) for a future reconciler to operate against, but no consumer of this data for reconciliation exists.

**Q3. Recovery from peer-DAG gap — if a peer's DAG ends up missing events, what mechanism brings it back into sync?**

None. Consequence of Q1 + Q2. The only mechanism that puts events on a peer's DAG today is the initial-handshake history dump in `handle_federation_incoming`. If a peer is added to a federation relationship at time T, it gets the Space history up to T at that moment, and never receives any event emitted on the home Node after T (other than identity-replication events, which travel through a separate non-federation subsystem and do not enter the Space DAG). To re-sync, the peer would need to issue a fresh `space.join_request` to the home Node — and the home Node's response would again be a one-time history dump followed by `goodbye`.

The peer can manually re-trigger this by re-initiating a fresh federation handshake, but the home Node itself has no logic that detects "peer is out of sync, push it the missing events." There is no detection signal and no push channel.

**Q4 (Pass-3 follow-up) — Federation peer connection lifecycle.**

- First-time connect-after-downtime: peer-initiated only; the home Node never reaches out. The peer's `run_initiating` triggers a full history dump from the home Node (history-tips-snapshot at handshake time), then connection closes.
- Peer goes offline mid-stream (sending to other peers, those are offline): no queues exist on either side. Events emitted during this window are lost from the federation path entirely.
- Stress-test evidence: per Trace 3 above, the test does not actually exercise ongoing federation propagation; the metric labelled "Federation Completeness" does not measure it.

### 2.6 What this means for the M6 `EventAccepted` G2 semantic

The G2 semantic locked in `docs/xgen_node_admin_ops_design.md` §2.1 reads:

> *"After the originator receives `EventAccepted`, they may claim the event is in the Node's authoritative DAG store."*

§2.1 is careful to scope the claim: G2 says nothing about federation propagation; the design doc §3.4 lists "Federation peers know about the event" as an explicit ❌-may-not-claim. The G2 claim is intentionally limited to the home Node.

This audit confirms that, at the present implementation state, the G2 claim is the *strongest* claim the system can honestly support — anything beyond it (claim of fan-out completion, claim of federation receipt, claim of any peer Node holding the event) cannot be made honestly because the propagation mechanism beyond Stage 5 does not exist.

The design doc §4.2 ("Stages M6 does NOT modify") states *"Federation propagation (Stage 6): The home Node forwards the event to peer Nodes over the federation session WS established by the federation handshake."* This sentence describes a mechanism that does not exist in the codebase. The federation-handshake session does not persist after the initial history dump; there is no "federation session WS" over which events are forwarded.

### 2.7 Verdict

**GAP IDENTIFIED — severity HIGH.**

Stage 6 (Node-to-Node federation propagation) is **architecturally absent** from the current implementation. The federation surface that exists is one-time history dump on peer-initiated handshake, then connection close. No persistent peer session, no outbound event push, no DAG-tip reconciliation, no gap-recovery mechanism. Three independent traces (no `run_initiating` callers in production; no pull mechanism; stress-test "completeness" measures local delivery only) converge on this finding.

**Implications for M6 and beyond:**

- The M6 `EventAccepted` G2 semantic remains *correct* as scoped — it makes no claim about federation propagation, and indeed nothing in the current implementation could back such a claim.
- The design doc §4.2 sentence ("the home Node forwards the event to peer Nodes over the federation session WS established by the federation handshake") **describes a mechanism that does not exist** and should be corrected when the federation-completion milestone closes; today's reality is closer to "no forwarding mechanism exists for ongoing events; initial handshake produces a one-time history dump."
- Per Joe's 2026-05-18 direction, the appropriate response is to close the federation gap properly before M6 proceeds. The shape of that completion milestone (whether it is push-from-home, pull-from-peer, or a hybrid; whether peer sessions persist or whether reconciliation is periodic; what the wire-protocol additions look like) is downstream design work; the audit's job ends at "describe reality."

**Severity HIGH justification.** The gap is not an observability or documentation concern (those are LOW per §1's pattern). It is the absence of a load-bearing protocol-implementation surface that the design doc claims exists and that future protocol promises (`EventAccepted` G2 propagation reliability, multiparty correctness in M9, federated multi-Node deployments) all depend on. M6 itself can ship without closing the gap (G2 is scoped narrowly enough) — but the project direction is to address the gap before M6 proceeds, per Joe's 2026-05-18 lock.

**Recommended follow-on placement (per `tasks/PROPAGATION_RELIABILITY_AUDIT.md` §4.2 Severity HIGH).** The gap blocks M6 (new) going ACTIVE per Joe's direction. It does not fit inside this audit (the audit is design-time; closing the gap is a new milestone). Recommended: file as a standalone milestone — provisional name *Federation Event Propagation* — to slot between this audit and M6 (new) Phase 1. Specific scope and design decisions are out of this audit; that is Chat Claude + Joe's design work in a subsequent session.

---

## §3 Stage 7 — Federated peer ingestion and re-fan-out

**Scope of the question — as narrowed by Joe.** Given §2's finding that no ongoing federation event propagation exists, Stage 7 reduces to: what happens to the events a peer Node receives during the *one-time initial-handshake history dump*? Does the peer run them through validation? Does it fan them out to its own local clients? Does it re-propagate to *its* federation peers (transitive federation)? And — the question kept explicit at Joe's request — what happens if validation fails on a history-dump event (unknown prev_events, timestamp out of bound, signature unverifiable because the originator's identity record hasn't been replicated)?

### 3.1 Production peer-side ingestion does not exist either

A grep of the full repository for `run_initiating` returns 11 production callers — every one of which is in `xgen-client/src/app.rs`:

| Site | Subcommand | Purpose |
|---|---|---|
| [app.rs:945](xgen-client/src/app.rs:945) | `smoke-test` | Phase-1 17-step smoke harness |
| [app.rs:1117](xgen-client/src/app.rs:1117) | `smoke-test` | Second federation hop |
| [app.rs:2415](xgen-client/src/app.rs:2415) | `smoke-ph2` | Phase-2 60-step smoke |
| [app.rs:2747](xgen-client/src/app.rs:2747) | `stress-test` | Phase 3 federation relay |
| [app.rs:3693](xgen-client/src/app.rs:3693) | `stress-complete` | Equivalent relay |
| [app.rs:4891](xgen-client/src/app.rs:4891) | `stress-complete` | A↔C federation |
| [app.rs:4919](xgen-client/src/app.rs:4919) | `stress-complete` | B↔C federation |

`xgen-node/src/` has zero non-test callers (already confirmed in §2.2). So in the present implementation:

- The only entities that *initiate* a federation handshake are test/diagnostic Client subcommands.
- These Clients authenticate first via `client_authenticate` (not as Nodes), then call `run_initiating` over the already-authenticated WebSocket.
- The handshake's receiving side (`handle_federation_incoming` in the real Node) responds with a one-time history dump.
- The initiating Client receives those events into a local `Vec<Event>` via a `loop { match fc.recv().await? }` pattern, then either: (a) discards them (smoke tests, after assertion checks), or (b) opens a *second* Client connection to a different Node and replays the events via `send_event` (stress-test relay pattern at [app.rs:2768-2778](xgen-client/src/app.rs:2768)).

```rust
// xgen-client/src/app.rs:2754-2778 — the relay receive-then-forward pattern
let mut history: Vec<Event> = vec![];
loop {
    match fc.recv().await? {
        Inbound::Event(ev) => {
            comm_event(&fed_log,&fed_seq,"fed_join","federation","RECV",&ev,&fed_na);
            history.push(ev);
        }
        Inbound::Transport(TransportMessage::Goodbye{..}) | Inbound::Closed => break,
        _ => {}
    }
}
// ... open second connection to Node B as a client ...
for ev in &history {
    bc.send_event(ev).await.ok();
    // ...
}
```

**The receive loop never calls `rt.ingest_event` or `rt.accept_message`.** No production code path takes events received via `run_initiating`'s response and ingests them into a local DAG. The data is held in a Vec and forwarded (in the stress relay) or asserted-against (in smoke tests). So Stage-7 "peer ingestion" is not exercised by any production receive-side code today; the only Stage-7-shaped pathway that *does* run validation+ingestion is the stress-relay's secondary `bc.send_event` call, which arrives at Node B as a regular client event and goes through `process_inbound` along with every other client-delivered event.

### 3.2 What `process_inbound` does with an event (the path stress-relay events take)

When the stress relay's `send_event` reaches Node B, the event arrives as `Inbound::Event(event)` and is dispatched by [`process_inbound`](xgen-node/src/app.rs:803) at `xgen-node/src/app.rs:803-944`. The dispatch is **heterogeneous by event type** — there are three distinct ingestion paths.

**Path A — Message events** (`MessageText`, `MessageFile`, `MessageReaction`, `MessageRedact`) at [app.rs:832-852](xgen-node/src/app.rs:832):

```rust
match rt.accept_message(&space_id, event.clone()) {
    Ok(_) => {
        trace_local(LocalAction::ApplyEvent, ...);
        FanoutRequest { event: Some(event), new_joiner: None }
    }
    Err(ExchangeError::HeldPending(_)) => {
        tracing::debug!(... "event buffered — waiting for unknown prev_events");
        FanoutRequest::none()
    }
    Err(e) => {
        tracing::error!(... reason = %e, "accept_message failed");
        trace_local(LocalAction::RejectEvent, ...);
        FanoutRequest::none()
    }
}
```

`accept_message` ([xgen-core/src/node/runtime.rs:147](xgen-core/src/node/runtime.rs:147)) runs the **full 13-step validation pipeline** via `accept_event` — signature verification, identity-registry lookup, prev_events presence check, timestamp check, state-machine permission check, the lot. Three outcomes:

- **Ok** → event stored in DAG + state machine updated + `drain_pending_messages` runs (cascade-resolves any events that were buffered waiting for this one). FanoutRequest emitted for local fan-out.
- **HeldPending** → event has unresolvable `prev_events`; buffered in `PendingBuffer` ([runtime.rs:174-179](xgen-core/src/node/runtime.rs:174)) keyed on the missing predecessor IDs. Will be auto-re-processed if those predecessors later arrive and are accepted. No fan-out. No log line beyond a `tracing::debug!`. **No signal back to the sender that the event is buffered.**
- **Err(other)** → validation failed (signature, identity, timestamp, permission, etc.). `trace_local(LocalAction::RejectEvent, ...)` logs it. **No fan-out. No signal back to the sender.** Event is dropped silently from the receiver's perspective; the sender sees nothing (this is the §5 `TransportMessage::Error` question — see §5 below for how the protocol actually surfaces this back to the sender via the `accept_message` arm in `handle_identity_msg` etc.).

**Path B — Membership join** (`MembershipJoin`) at [app.rs:853-872](xgen-node/src/app.rs:853):

Direct `rt.ingest_event` call after two pre-checks: (1) Space must exist locally — otherwise reject with `step=10`; (2) new-joiner detection by comparing sender to current Space members. **No HeldPending handling.** **No full validation pipeline** (no signature verification step, no timestamp check, no full state-machine permission check — only the Space-exists check). If the join's `prev_events` reference an unknown predecessor, `ingest_event` is called regardless. The runtime's `ingest_event` at [runtime.rs:130-136](xgen-core/src/node/runtime.rs:130) for state-machine events applies the event to SpaceState via `state.apply_event(&event)` if the Space exists — and silently does nothing if it doesn't (state machine has no concept of "buffer this for later"). FanoutRequest is emitted with `new_joiner` populated, triggering history push to the new member.

**Path C — All other event types** (state.*, etc.) at [app.rs:873-943](xgen-node/src/app.rs:873):

Direct `rt.ingest_event` + `persist_event`, gated by two pre-checks: (1) AI role violation check for `StateSpaceCreate`/`StateDmSpaceCreate` (rejects if sender is AI); (2) AI operator target+permission check for `StateAiOperatorDelegate`/`StateAiOperatorRevoke`. **No HeldPending handling.** **No full validation pipeline.** **No signature verification at this layer.** Unknown `prev_events` are silently tolerated. If `ingest_event` reaches a Space that doesn't exist locally, the state-machine apply is a no-op (same silence as Path B). FanoutRequest emitted unconditionally.

The runtime.rs doc comment at line 143 advertises that PendingBuffer handles "out-of-order federation delivery" — but this is **only true for the `accept_message` path (Path A)**. Paths B and C have no pending-buffer integration; their `prev_events` checks are weaker and their failure modes are "silent ingest with state-machine no-op" rather than "buffer and retry."

### 3.3 Joe's question — validation failure on a history-dump event

Concretely, three failure scenarios for a history-dump event that the stress-relay forwards to Node B via `bc.send_event`:

**Scenario A — `prev_events` references an event Node B doesn't yet have.**

- **Message event:** Path A → `accept_message` returns `HeldPending(missing)`. Event buffered in `PendingBuffer`. When the missing predecessor arrives (later in the same history dump, or never), the buffered event auto-resolves and runs through validation again. If the predecessor never arrives, the event sits in PendingBuffer indefinitely. No log line (other than `tracing::debug!` at receive time). No signal to sender.
- **Membership.join:** Path B → no HeldPending. `ingest_event` is called regardless; the state machine's apply is a no-op if context is insufficient (e.g. invite not yet ingested). The event lands in the EventStore but may not have any state-machine effect. **DAG hole at the state-machine layer, silent.**
- **Other state events:** Path C → same as B. Silent ingest; may or may not have state effect.

**Scenario B — Timestamp check fails (e.g. clock skew, or future-timestamp guard rejects).**

- **Message event:** Path A → validation step (timestamp check is inside the 13-step pipeline) fails → `Err(other)` → `trace_local(LocalAction::RejectEvent, ...)`, drop, no signal back. Event is lost from this Node's DAG.
- **Membership.join + other state events:** Paths B/C — **no timestamp check is run at this layer.** The event is ingested. The runtime does not enforce timestamp bounds for non-message events at the `process_inbound` boundary. Whether downstream code (state-machine, eventual consistency layer) enforces bounds is out of scope here, but at the ingestion boundary the protection is one-sided (message events only).

**Scenario C — Signature check fails because the originator's identity record isn't replicated to this peer.**

- **Message event:** Path A → `accept_event` step 4 (signature verification) requires identity-registry lookup for the sender. If the sender isn't in `identity_registry`, the lookup fails before signature verification can proceed; the call returns `Err`. `trace_local(LocalAction::RejectEvent, ...)`, drop, no signal back. **The history-dump event is silently lost from this peer's DAG**, even though it is a valid event in the originator's home Node.
- **Membership.join + other state events:** Paths B/C — **no signature verification is run at this layer.** The event is ingested regardless of whether its sender is known. This is a *correctness gap distinct from the federation gap* — the protocol's signature guarantee is enforced only for message events on the `process_inbound` path, not for state or membership events.

**Note on the signature-check asymmetry.** The 13-step validation pipeline is comprehensive when it runs (Path A), but Paths B and C bypass it. This is not a federation-specific issue — it would be true for any event ingested via `process_inbound`, regardless of whether the event originated locally or came via a future federation push. It surfaced during the Stage 7 trace because the question "what happens if signature can't be verified on a history-dump event" forced the question, and the answer for membership/state events is "no check runs." The audit records this for downstream consideration; the fix belongs in a separate code-correctness milestone, not in the federation-completion milestone, and is severity-deferred to Joe's judgment (likely MEDIUM — a real correctness gap, but not the one this audit was opened to verify).

### 3.4 Re-fan-out to the peer's local clients

This part works. `process_inbound` returns a `FanoutRequest`; the calling site at [app.rs:637](xgen-node/src/app.rs:637) immediately calls `apply_fanout(fanout, &identity_id, &runtime, &client_senders)`. So events that successfully ingest into a peer Node — regardless of whether they came from a local client or from the stress-relay's `bc.send_event` forwarding — *do* get fanned out to the peer Node's locally-connected members.

The fan-out is the same Stage-5 mechanism audited in §1. All the §1 caveats apply (silent drop on channel full, silent skip on disconnected recipient, no retry — recovery via Stage 8). For a Node that actually received federation events via some future mechanism, the re-fan-out to its own local clients would inherit Stage-5's strengths and weaknesses.

### 3.5 Transitive federation — does the protocol or implementation support it?

**No.** `apply_fanout` operates exclusively against `ClientSenders` — `Arc<Mutex<HashMap<String, mpsc::Sender<OutboundMsg>>>>` ([fanout.rs:40](xgen-node/src/fanout.rs:40)) — which is the map of *locally-connected client identities*. It has no notion of peer Nodes; it does not look up federation relationships; it does not forward to any kind of peer connection. Even if the gap §2 identifies were closed (events propagated to direct federation peers), those peers would not in turn propagate to *their* federation peers via `apply_fanout`. Transitive federation is unimplemented at every layer — no wire shape, no code path, no metric.

Whether the protocol-as-specified intends transitive federation is a question for the design-completion milestone, not for this audit. The audit records that the current implementation does not provide it.

### 3.6 Verdict

**GAP IDENTIFIED — severity HIGH (consequence of §2).**

The Stage-7 question is downstream of Stage 6. With Stage 6 architecturally absent, Stage 7 has no production exercise: no production code receives federation events; no production code ingests them. The infrastructure that *would* run if federation events did flow — `process_inbound` + `apply_fanout` — partially works (Path A for messages with full validation + HeldPending) and partially does not (Paths B and C bypass signature/timestamp validation; no transitive federation).

**Internal consistency with §2.** §3's finding that no production peer-side ingestion path exists is the same finding as §2 viewed from the other side: §2 said "Node-to-Node federation propagation doesn't exist as an outbound mechanism"; §3 confirms there is no peer-side ingestion path to receive what the outbound side would push. This is internal consistency, not additional rot — both directions confirm the same architectural absence.

**Sub-findings recorded for downstream attention:**

| Sub-finding | Severity | Placement |
|---|---|---|
| Production peer-side ingestion path doesn't exist | HIGH | Same root cause as §2; closed by the federation-completion milestone |
| `process_inbound` validation asymmetry — Paths B and C skip signature + timestamp verification | LOW *today*, HIGH *on federation landing* — must close as **precondition of** federation completion (see below) | No separate task file; CLAUDE.md PENDING block for federation completion notes validation asymmetry as a precondition, coordinated within that milestone's design phase |
| Transitive federation unimplemented at every layer | MEDIUM, deferred | Out of audit scope. The current code answers the transitive-federation question *by accident* — there's no relay because there's no federation push at all. The design call (pairwise-only / transitive-by-default / transitive-opt-in) lands with the federation-completion milestone's design phase. Document in audit only; don't pre-decide. |
| HeldPending silent buffer (no log, no signal-back, can sit forever) | LOW | Observability concern; pairs with §1's silent-drop pattern. Future contributor adds metric/log when next touching pending-buffer code. |

**Severity-elevation note for sub-finding 2.** The `process_inbound` validation asymmetry (Paths B and C skipping signature verification and timestamp checks) is *today* a latent gap — severity LOW — because no production code path reaches Paths B and C with events whose signatures haven't already been verified at submission time. Locally-submitted events carry the originator's authenticated WebSocket session as implicit provenance; the bypass in Paths B/C does not currently create an exploitable surface.

**This changes the moment federation event push lands.** Federation propagation is the exact vector that would make the asymmetry exploitable: as soon as a peer Node can push `MembershipJoin` or other state events to this Node, those events would flow through Paths B/C and reach the DAG without signature verification. A malicious or compromised peer could inject membership or state events purporting to come from any Identity, and the receiving Node would accept and persist them.

**Timing constraint — explicit.** The validation asymmetry MUST be addressed *before* — or as a coordinated precondition of — the federation-completion milestone. Closing federation propagation without first closing the validation asymmetry would land a vulnerability. The federation-completion design phase (Chat Claude + Joe, post-audit) addresses both as coordinated work, not parallel milestones.

**Within the narrowed scope Joe approved:** validation failure on a history-dump event produces three distinct silent-loss patterns (Scenario A buffers for messages / silently ingests with no state effect for non-messages; Scenario B drops messages on timestamp failure / no check for non-messages; Scenario C drops messages on unknown-sender / no check for non-messages). None of these signal back to the originating Node or to any operator-visible surface beyond `trace_local(LocalAction::RejectEvent, ...)` for the message-event paths. The "DAG hole" outcome Joe asked about is real and occurs in Scenarios A (non-message), B (non-message), and C (non-message) — events ingested into the EventStore but with state-machine no-op effects, leaving the SpaceState diverged from the DAG's nominal content.

The federation-completion milestone will inherit these sub-findings: any real federation push mechanism must decide how to handle ingestion failures at the peer (HeldPending buffering for non-messages? Cross-Node sender-identity lookup before signature check fails? Notify-back-to-sender on validation failure? A peer-side "request missing predecessor" follow-up?). These design questions are out of scope here.

---

## §4 Stage 8 — Sync catch-up on reconnect

**Scope of the question.** When a client disconnects and reconnects, how does it discover and ingest the events it missed during the gap? Is the mechanism reliable? Does it ever get used Node-to-Node (which would be the closest existing analogue to federation reconciliation)?

### 4.1 Wire shape and protocol

The wire-level message is `TransportMessage::SyncRequest` at [`xgen-core/src/wire/types.rs:91-95`](xgen-core/src/wire/types.rs:91):

```rust
/// Request missed Events since a given event_id.
#[serde(rename = "transport.sync_request")]
SyncRequest {
    protocol_version: String,
    since: String,
},
```

Two fields. `since` is an `event_id` URI; the requester asks the Node to send events that follow `since` in the DAG. An empty `since` means "send me everything from every Space I'm a member of."

The spec (Ch3 §3.3.6 lines 1017-1041, per FIXES_ph1.md Fix 05) defines two reply wire shapes: `transport.sync_response` (carrying a batched event list) and `transport.sync_complete` (signalling end-of-batch). **Neither is implemented.** The Phase-1 implementation deliberately deferred them — the comment at [`xgen-node/src/fanout.rs:25-26`](xgen-node/src/fanout.rs:25):

```rust
/// Phase 1 keeps this minimal — Events only. `transport.sync_complete` /
/// `transport.sync_response` wrappers from spec 3.3.6 are deferred; the client
/// reads the streamed Events directly until quiet.
```

The Node's actual response is the same `OutboundMsg::HistoryBatch { events: Vec<Event> }` shape it uses for joiner-history push (§1.1). The connection handler at [`app.rs:592-599`](xgen-node/src/app.rs:592) unwraps the batch and writes each event individually to the WebSocket via `conn.send_event(&ev)`. There is no in-band signal that "the batch is complete."

### 4.2 Client-side completion detection — timeout, not signal

Because no `sync_complete` is sent, the requesting client must guess when the response stream has ended. The canonical pattern is at [`xgen-client/src/batch.rs:83-91`](xgen-client/src/batch.rs:83):

```rust
let req = TransportMessage::SyncRequest {
    protocol_version: "0.1".to_string(),
    since: String::new(),
};
conn.send_transport(&req).await?;
let mut tips: Vec<String> = vec![];
let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_millis(500);
loop {
    match tokio::time::timeout_at(deadline, conn.recv()).await {
        // ...
    }
}
```

**500ms quiet-time = "stream complete."** All four production `SyncRequest` callers (`batch.rs:83`, `ai_service.rs:224`, `ops.rs:721`, `ops.rs:939`) follow this pattern. There is no protocol-level handshake that says "all caught-up events have been delivered" — the client gives up after no inbound has arrived for the timeout duration.

This works under low-latency conditions and small-volume catch-ups (Phase 1 / Phase 2 testing), but has known failure modes: under high WAN latency, a slow stream may exceed the 500ms quiet threshold between events; under high catch-up volume, the stream may not finish within reasonable bounded time and the client cannot know whether the last event received was the last event the Node will send.

### 4.3 Server-side handler and `collect_sync_history` semantics

The Node handles `SyncRequest` at [`app.rs:613-619`](xgen-node/src/app.rs:613):

```rust
Ok(Inbound::Transport(TransportMessage::SyncRequest { since, .. })) => {
    let events =
        collect_sync_history(&runtime, &identity_id, &since).await;
    let _ = out_tx
        .send(OutboundMsg::HistoryBatch { events })
        .await;
}
```

[`collect_sync_history`](xgen-node/src/fanout.rs:178) at `xgen-node/src/fanout.rs:178-207` implements the gathering logic:

```rust
let rt = runtime.lock().await;
let mut out: Vec<Event> = Vec::new();
for (space_id, space) in &rt.spaces {
    if !space.is_member(requester_id) {
        continue;
    }
    if let Some(store) = rt.stores.get(space_id) {
        let all: Vec<Event> = store.values().cloned().collect();
        let sorted = topological_sort_events(all);
        if since.is_empty() {
            out.extend(sorted);
        } else {
            let mut past = false;
            for ev in sorted {
                if past {
                    out.push(ev);
                } else if ev.event_id.as_deref() == Some(since) {
                    past = true;
                }
            }
        }
    }
}
out
```

Semantic properties:

| Property | Status |
|---|---|
| **Per-Space membership filter** ([fanout.rs:186-188](xgen-node/src/fanout.rs:186)) | ✅ Correct. Requester gets events only from Spaces they are a member of; cross-Space leak is prevented at this layer. |
| **`since=""`** | Returns ALL events from EVERY Space the requester is a member of, topologically sorted per Space. No pagination, no limit. |
| **`since=<known_id>`** | Returns all events that follow `since` in topological order. The boundary event itself is excluded (the `past = false` flag flips on match, then subsequent events are pushed). |
| **`since=<unknown_id>`** | Returns an empty list. Silent. `past` never flips. The client receives no events and detects end-of-stream via the 500ms timeout. **No signal-back that the `since` cursor was invalid.** |
| **Topological sort** | Applied per-Space at [fanout.rs:191](xgen-node/src/fanout.rs:191). Across Spaces, no ordering invariant — events from different Spaces interleave by `HashMap` iteration order over `rt.spaces` (non-deterministic). |
| **Pagination / response limits** | None. A reconnecting client whose Spaces contain millions of events would receive millions of events in a single response. |

### 4.4 Joe's explicit Stage-8 questions

`tasks/PROPAGATION_RELIABILITY_AUDIT.md` §3.4 enumerates four questions; answers:

**Q1. Protocol message — wire-level shape?** `TransportMessage::SyncRequest { protocol_version, since }` (above §4.1). Two fields only. The spec-defined response shapes (`sync_response`, `sync_complete`) are unimplemented; the actual response is a stream of bare `Event` messages on the same channel.

**Q2. What does the Node return — complete event list, DAG-tip diff, something else?** A complete event list, topologically sorted per Space, for every Space the requester is a member of, filtered by the `since` cursor (events strictly after `since`). Not a tip diff. Not paginated. Cross-Space membership filter is enforced.

**Q3. Client's known tip is far behind — pagination, time-window limit?** Neither. The full event set is returned in one stream. This is workable for Phase-1/Phase-2-scale data sizes; will not scale to long-running production deployments without a follow-on pagination/limit mechanism.

**Q4. Client's `since` references events the Node no longer has (e.g. compaction)?** No compaction mechanism exists today, so the scenario does not occur in practice. *If* it did occur, the behaviour is silent: `collect_sync_history` returns an empty list, the client receives nothing, the 500ms timeout fires, the client believes it is up-to-date. There is no `sync_error` or `since_unknown` signal-back.

### 4.5 Joe's specific addition — is sync_request used Node-to-Node?

**No.** Construction sites for `TransportMessage::SyncRequest { ... }` in the full repository:

| Location | Caller |
|---|---|
| [`xgen-client/src/batch.rs:83`](xgen-client/src/batch.rs:83) | `get_dag_tips` — client-side tip lookup before sending |
| [`xgen-client/src/ai_service.rs:224`](xgen-client/src/ai_service.rs:224) | AI resident's initial-connect catch-up |
| [`xgen-client/src/ops.rs:721`](xgen-client/src/ops.rs:721) | `ops::*` library helper |
| [`xgen-client/src/ops.rs:939`](xgen-client/src/ops.rs:939) | `ops::*` library helper |

Zero `xgen-node/src/` constructors. Confirms client-to-Node only; no Node-to-Node use. There is no existing mechanism in the codebase that resembles peer-to-peer DAG reconciliation. The pattern `SyncRequest` provides — "give me everything since X for the Spaces I'm a member of" — is fundamentally a client-recovery shape (the requester is identified by their authenticated session as a Space member); applying it Node-to-Node would require either (a) the requesting Node authenticating as a member identity (which it does not own), or (b) a new Node-to-Node variant that uses federation-relationship authority instead of membership.

### 4.6 Documentation-vs-reality observation

Three documents describe behaviour at the Stage-6/Stage-8 boundary that does not exist in code:

- `docs/xgen_node_admin_ops_design.md` §4.2 sentence on federation push (already flagged in §2.6 of this document).
- `docs/xgen_ch4_implementation.md:779`: *"Fan-out to federated peers wraps the Event in a transport frame and sends it over the active Node-to-Node WebSocket connection. If the connection to a peer is temporarily down, the Event is held in a per-peer outbound queue. When the peer reconnects, the queue is flushed and the peer sends a `transport.sync_request` to catch up on any Events it missed."*
- `docs/xgen_ch4_implementation.md:825-827`: *"Events that fail validation step 9 (unknown predecessor) are held in a per-Room in-memory pending buffer. The Node sends `transport.sync_request` to its peers for the missing predecessor IDs. ... A reconnecting peer will re-send Events via `transport.sync_request`."*

Both Ch4 §implementation passages describe a Node-to-Node `transport.sync_request` flow (Node-as-requester, peer Node-as-responder) that has no implementation. This is consistent with the §2 finding: the implementation guide was written with a federation surface in mind that did not survive into the present code. The corrections belong with the federation-completion milestone's design phase, not with this audit.

### 4.7 Verdict

**PARTIALLY VERIFIED.**

| Question | Status |
|---|---|
| `SyncRequest` wire shape and dispatch path | VERIFIED — code-grounded, exercised in production by all four constructors, served correctly by the Node handler. |
| Per-Space membership filter on response | VERIFIED — `space.is_member(requester_id)` guard at fanout.rs:186-188; locked by the test `collect_sync_history_returns_only_member_spaces` ([fanout.rs:522-565](xgen-node/src/fanout.rs:522)). |
| Unknown-`since` behaviour | PARTIALLY VERIFIED — behaviour is "return empty silently." Correct under "no compaction exists" but fragile if any future code path causes events to disappear from the store. No signal-back to the client. |
| Pagination / response limits | VERIFIED ABSENT — by design at Phase 1; this is a known follow-on for a scaling milestone. |
| Spec compliance — `sync_response` / `sync_complete` wire shapes | GAP IDENTIFIED — specced in Ch3 §3.3.6 (per FIXES_ph1.md Fix 05) but unimplemented. The implementation uses bare `Event` messages on the same channel and a 500ms client-side timeout for end-of-stream detection. |
| Node-to-Node use | VERIFIED ABSENT — zero `xgen-node` callers; no Node-to-Node reconciliation pattern exists. |

**The mechanism that exists works** for the client-recovery case it was built for — small-volume catch-up after a short disconnect, with bounded Spaces and a low-latency network. Stage 8 is the recovery path that §1 depends on for delivery-loss recovery (channel-full silent drops, disconnected-recipient silent skips); it succeeds at this role because the typical Phase-1/Phase-2 use case stays within the conditions that make timeout-based stream-end detection reliable.

**Sub-findings recorded for downstream attention:**

| Sub-finding | Severity (proposed) | Placement |
|---|---|---|
| `sync_response` / `sync_complete` wire shapes specced but unimplemented (client uses 500ms quiet-time fallback) | LOW *today* (works for tested workloads), MEDIUM as deployment surface grows (high-latency / large-catch-up failure modes) | Document in audit only; landing the specced shapes is a future protocol-hardening milestone, not the federation milestone |
| Unknown-`since` returns silent-empty with no signal-back | LOW | Document in audit only; future contributor adds a `since_unknown` signal when the scaling milestone touches this code |
| No pagination / response-size limit | LOW today, MEDIUM at scale | Known Phase-1 simplification per `fanout.rs:25-26` comment. Address in scaling milestone, not this audit |
| Cross-Space topological ordering is HashMap-iteration-order (non-deterministic) | LOW | Acceptable because clients demultiplex by `space_id`; each Space's internal order is preserved. Already noted in earlier project memory ("EventStore HashMap iteration determinism" carry-over) |
| Documentation-vs-reality gap in Ch4 §implementation (Node-to-Node `sync_request` flow described, not implemented) | LOW | Documentation gap, paired with the §2.6 design-doc correction. Both close in the federation-completion milestone's documentation pass. |

**Pattern observation across §2–§4.** This is the third drift surface the audit has surfaced across its first three sections. §2 found that `docs/xgen_node_admin_ops_design.md` §4.2 describes a federation push mechanism that does not exist; §3 found that `process_inbound` runs full validation only for messages, with state/membership events bypassing signature and timestamp checks; §4 now finds that Ch4 §implementation describes a Node-to-Node `transport.sync_request` flow that is not implemented and that the specced `sync_response` / `sync_complete` wire shapes are absent from the implementation. The pattern is consistent: where the audit looks, it finds drift between specification/design documents and the implementation that was supposed to back them. None of these drift surfaces are the audit's "primary" finding (Stage 6 absence is) — but their collective presence suggests that the post-audit documentation pass needs to be substantive, and that the federation-completion milestone should not assume the existing implementation guides are accurate maps of what's there to extend.

---

## §5 `TransportMessage::Error` propagation scope

**Scope of the question.** Per `tasks/PROPAGATION_RELIABILITY_AUDIT.md` §3.5 — confirm that `TransportMessage::Error` is sent only to the originator's connection, never broadcast, never federated, never persisted in the DAG. This was expected to be the audit's shortest and most-confirmatory section. What the trace actually finds is substantially different from what the design doc §2.1 and §3.5, and earlier session text in J-080, both implicitly claim.

### 5.1 Wire shape

[`xgen-core/src/wire/types.rs:75-82`](xgen-core/src/wire/types.rs:75) — the actual definition:

```rust
/// General transport error.
#[serde(rename = "transport.error")]
Error {
    protocol_version: String,
    error_code: u32,
    error_string: String,
    timestamp: String,
},
```

**Four fields. No `event_id`. No `reason` field.**

Compare to the shape sketched in `docs/xgen_node_admin_ops_design.md` §3.1 line 204:

```rust
Error { event_id: String, reason: String, /* ... */ },
```

The design doc shows a different field set — `event_id` and `reason` — that does not match the implementation. The drafted shape is what the design doc *imagines* `Error` to look like when it functions as a per-event rejection signal. The actual wire shape has no per-event correlation field. **A client receiving `transport.error` cannot identify which submitted event it pertains to.** This is the fourth drift surface this audit has surfaced: §2 (Ch3 §3.15 federation propagation), §3 (process_inbound validation asymmetry), §4 (Ch4 §implementation `sync_request` flow + unimplemented `sync_response`/`sync_complete` shapes), and now §5 (design doc's `Error` shape vs. implementation's `Error` shape).

### 5.2 Production emit sites

A grep of the entire repository for `TransportMessage::Error {` returns exactly one production construction site:

| Location | Trigger |
|---|---|
| [`xgen-node/src/app.rs:1085`](xgen-node/src/app.rs:1085) | Identity-replicate failure (inside `handle_identity_replicate_msg`) |

Context at app.rs:1082-1092:

```rust
Err(e) => {
    tracing::warn!(identity_id = %identity_id, reason = %e, "identity.replicate: rejected");
    // Send transport error 3020 so the home Node can handle the stale-version case.
    let err_msg = TransportMessage::Error {
        protocol_version: "0.1".to_string(),
        error_code: e.error_code(),
        error_string: e.to_string(),
        timestamp: ts,
    };
    let _ = conn.send_transport(&err_msg).await;
}
```

This is the only path. The error is emitted when an inbound `IdentityReplicate` from a peer Node fails to apply (e.g. version mismatch); it is sent back to the peer Node's connection.

**Notably absent:** the event-acceptance failure paths. Searching `process_inbound` at [`xgen-node/src/app.rs:803-944`](xgen-node/src/app.rs:803):

| Failure path | What it does | Sends `Error`? |
|---|---|---|
| Path A — `accept_message` returns `Err(other)` ([app.rs:846-851](xgen-node/src/app.rs:846)) | `tracing::error!` + `trace_local(LocalAction::RejectEvent, ...)` | **No** |
| Path B — `membership.join` for unknown Space ([app.rs:855-858](xgen-node/src/app.rs:855)) | `tracing::error!` + `trace_local(LocalAction::RejectEvent, ...)` | **No** |
| Path C — AI role violation ([app.rs:885-897](xgen-node/src/app.rs:885)) | `tracing::error!` + `trace_local(LocalAction::RejectEvent, ...)` | **No** |
| Path C — AI operator target/permission check ([app.rs:913-921](xgen-node/src/app.rs:913), [app.rs:926-934](xgen-node/src/app.rs:926)) | `tracing::error!` + `trace_local(LocalAction::RejectEvent, ...)` | **No** |

**None of the event-acceptance rejection paths send `TransportMessage::Error` to the originator.** All five paths log to the Node's tracing layer and emit a `trace_local(LocalAction::RejectEvent, ...)` line — both of which are Node-side observability surfaces visible only to the Node operator. The originating client receives nothing on the wire.

### 5.3 What this means for the design-doc and J-080 framing

The design doc `docs/xgen_node_admin_ops_design.md` §2.1 names the principle (D-070 proposed) as:

> *"The asymmetry that existed before M6 — `Error` exists, no acceptance signal — was a structural-by-accident, not by-design property; M6 closes it."*

And §3.5 of the design doc claims:

> *"`Error` is sent only to the originator's connection. `Error` does not propagate to other members (a rejection means the event never entered the DAG — there is nothing to fan out)."*

The earlier J-080 carry-over framing (`JOURNAL.md` lines 110-113) said:

> *"The Client cannot detect **acceptance** at all today — only **rejection** (via `TransportMessage::Error`) and *absence of rejection* (which is the silent-Node-hang ambiguity)."*

And the Pass-3 input section of `tasks/NODE_ADMIN_PASS2_PROPOSALS.md` line 410:

> *"`TransportMessage::Error` citing the originator's `event_id` — **Yes** (M3 3041 path, validation failures) — Rejected"*

**All four passages share an assumption that this audit refutes:** that `TransportMessage::Error` *exists as a rejection signal for event-acceptance failures, citing the originator's event_id*. The implementation does neither. The wire shape lacks `event_id`. The event-acceptance failure paths do not emit `Error` at all. The single production emit site is identity replication, not event acceptance.

The accurate picture is symmetric in a different way than D-070 imagined: **neither direction has a wire-layer signal for event acceptance/rejection.** The originator gets silence on success (no `EventAccepted` because none exists) and silence on validation failure (no `Error` because the rejection paths don't emit it). The only observable signal in either direction is "no signal at all, distinguishable from a hung Node only by timeout."

### 5.4 Joe's three confirmations

The task file §3.5 asks for three explicit confirmations:

**Confirmation 1 — "every emit site sends to the originator's connection only, not to fan-out."**

VERIFIED. The single emit site at app.rs:1085 uses `conn.send_transport(&err_msg).await` on the local `Connection` handle (the inbound peer's connection). There is no broadcast loop, no `client_senders` iteration, no fan-out. The error is delivered on the same WebSocket the failing `IdentityReplicate` arrived on.

**Confirmation 2 — "`Error` is never broadcast."**

VERIFIED. No code path constructs `TransportMessage::Error` and iterates over `ClientSenders` or `peer_urls` or any kind of distribution map. The single emit site sends to exactly one connection.

**Confirmation 3 — "`Error` is never federated."**

VERIFIED. `Error` is a `TransportMessage`, not an `Event`. It cannot enter the DAG; the DAG store accepts only `Event` values. There is no federation forwarder for transport messages, and as §2 established, no production federation event push exists at all.

The three confirmations hold — but they hold *vacuously for event-acceptance failures*, because `Error` is not sent for event-acceptance failures in the first place.

### 5.5 D-070 grounding — revised

Joe asked for §5 to provide empirical grounding for the D-070 principle ("two events of equal importance, opposite direction"). The original framing was: §5 confirms `Error`'s scope; §2 establishes that `EventAccepted` is the missing symmetric partner. With the actual §5 findings, the grounding sentence must be revised.

**The empirical situation, stated honestly:** D-070's framing assumed `Error` exists as the rejection signal for event acceptance. The audit refutes this assumption. The implementation has *no* wire-layer signal for either acceptance or rejection of event submissions. The asymmetry D-070 was opened to close (rejection has signal, acceptance doesn't) is not the actual asymmetry. The actual situation is more uniformly broken: neither direction has a usable signal.

**This strengthens, not weakens, the D-070 principle.** The principle says: where the protocol could speak the truth in both directions, it must. The audit's finding is that the protocol currently speaks the truth in *neither* direction for event acceptance. Applying D-070 honestly means M6 ships not just `EventAccepted` but also a corresponding event-correlated rejection signal — either an extension to `Error` that carries `event_id` (and wired into the acceptance failure paths in `process_inbound`), or a new `EventRejected` wire shape symmetric to `EventAccepted`. Either choice closes the actual asymmetry, not the imagined one.

Picking between extending `Error` and adding `EventRejected` is a design call for the M6 (new) Phase 2 work — and it is now informed by an audit finding, not an assumption. The audit records the choice as open; the design phase decides which form D-070 takes when both signals ship together.

### 5.6 Verdict

**GAP IDENTIFIED — severity HIGH (different from §2's HIGH; this one blocks M6's `EventAccepted` from being a useful primitive on its own).**

The three originator-only / never-broadcast / never-federated confirmations all hold, but they hold against an implementation where `Error` is not the rejection signal the design doc and earlier session framing assumed. The wire shape has no `event_id` field. The event-acceptance failure paths do not emit `Error` at all. The single production emit site (identity replicate failure) has nothing to do with event acceptance.

**Sub-findings recorded for downstream attention:**

| Sub-finding | Severity (proposed) | Placement |
|---|---|---|
| `TransportMessage::Error` wire shape lacks `event_id` (cannot correlate to originator's submitted event) | HIGH | Closed by M6 (new) Phase 2 — either extend `Error` shape or add `EventRejected`. Coordinated with `EventAccepted` design. |
| Event-acceptance failure paths in `process_inbound` do not emit any wire-layer signal (only Node-side trace_local logs) | HIGH | Same. Wiring the rejection signal into `accept_message`'s `Err` arms is the symmetric counterpart of wiring `EventAccepted` into the `Ok` arm. |
| Design doc §3.1 line 204 sketches an `Error` shape (`event_id` + `reason`) that does not match the implementation | LOW (documentation) | Document-pass correction during M6 (new) Phase 2 or post-audit doc sync |
| J-080 journal entry and Pass-3 input addendum both assert `Error` is the rejection signal — incorrect | LOW (historical record) | No correction needed in the journal/addendum (they remain as historical record of what was believed); the canonical doc going forward is this audit |

**Implication for M6 (new) Phase 2.** Phase 2 was specified in design doc §5.2 to ship `TransportMessage::EventAccepted` on the accept path. The audit's §5 finding means Phase 2 must also ship the symmetric rejection-side wiring — either extending `Error` to carry `event_id` and wiring it into the `process_inbound` reject paths, or adding `EventRejected`. The decision is small relative to the full M6 milestone scope but cannot be deferred: shipping `EventAccepted` without its symmetric partner would itself be an instance of the asymmetry D-070 was named to prevent.

**This is the fourth drift surface across the audit's five sections.** §2, §3, §4, and now §5 each surfaced a place where the implementation diverges from documents that purported to describe it. The pattern across all four: the documents describe behaviour that was reasonable to assume but never verified. The audit's job — verifying — was overdue, and its delivery surfaces a coherent set of related gaps rather than four unrelated rot pockets. All four close in coordinated work between the federation-completion milestone (§2, §3, parts of §4) and M6 (new) Phase 2 (§5 + §3's validation-asymmetry precondition).

---

## §6 Close-out summary

### 6.1 Verdicts at a glance

| § | Stage | Verdict |
|---|---|---|
| §1 | Local fan-out (Stage 5) | PARTIALLY VERIFIED — mechanism correct; two LOW-severity documentation/observability gaps |
| §2 | Node-to-Node federation propagation (Stage 6) | **GAP IDENTIFIED — severity HIGH.** Stage 6 architecturally absent |
| §3 | Federated peer ingestion and re-fan-out (Stage 7) | GAP IDENTIFIED — severity HIGH (consequence of §2), plus one elevation-on-federation-landing finding |
| §4 | Sync catch-up on reconnect (Stage 8) | PARTIALLY VERIFIED — mechanism works for current workloads; spec-vs-impl gaps and scale-fragility |
| §5 | `TransportMessage::Error` propagation scope | **GAP IDENTIFIED — severity HIGH** (different shape than §2's HIGH). Three originator-only confirmations hold vacuously |

### 6.2 The pattern across §2–§5 — drift surfaces, recorded as fact

The audit surfaced documentation-vs-implementation drift in four of its five sections. §2 found the design doc `docs/xgen_node_admin_ops_design.md` §4.2 describes a federation push mechanism that does not exist. §3 found that `process_inbound` runs full validation only for messages, with state/membership events bypassing signature and timestamp checks (a finding that is not itself federation drift, but surfaced as the federation question forced the trace). §4 found that Ch4 §implementation lines 779 and 825-827 describe a Node-to-Node `transport.sync_request` flow that is not implemented, and that the specced `sync_response` / `sync_complete` wire shapes are absent from the implementation. §5 found that `TransportMessage::Error`'s actual wire shape (no `event_id`) does not match the design doc's drafted shape, and that the event-acceptance reject paths do not emit `Error` at all — refuting framing that was sustained across multiple sessions.

Recorded as fact, without editorialising: where the audit looked, it found drift. The implication — that more drift may exist elsewhere in the codebase — is a project-management conversation Chat Claude + Joe will have post-audit. A new project principle is forming ("subsystem audits precede dependent milestones") that will be formalised in that conversation; this audit's role is to provide the empirical motivation.

### 6.3 Consolidated sub-findings

| Sub-finding | Section | Severity | Closure path |
|---|---|---|---|
| Silent `try_send` drops have no observability surface | §1 | LOW | Document in audit only; counter added when next touching `fanout.rs` |
| Author-exclusion rationale unrecorded at point of code | §1 | LOW | Fold into D-070 promotion text post-audit |
| Production peer-side ingestion path doesn't exist | §3 | HIGH | Same root cause as §2 → Federation Completion milestone |
| `process_inbound` validation asymmetry (Paths B/C skip signature + timestamp) | §3 | LOW today, HIGH on federation landing | **Precondition** of Federation Completion milestone — must close together |
| Transitive federation unimplemented at every layer | §3 | MEDIUM, deferred | Design call for Federation Completion milestone's design phase |
| HeldPending silent buffer (no log, no signal-back, can sit forever) | §3 | LOW | Document in audit only |
| `sync_response` / `sync_complete` specced-but-unimplemented; 500ms quiet-time fallback | §4 | LOW today, MEDIUM at scale | Related concern for Federation Completion design phase (large catch-up workload) |
| Unknown-`since` returns silent-empty | §4 | LOW | Flag for compaction work if/when it lands |
| No pagination / response-size limit on sync | §4 | LOW today, MEDIUM at scale | Pairs with previous; scaling milestone |
| Cross-Space topological order is HashMap-iteration-order | §4 | LOW | Cross-references existing M4 carry-over note in CLAUDE.md |
| Ch4 §implementation describes Node-to-Node sync flow that doesn't exist | §4 | LOW | Documentation pass during Federation Completion milestone |
| `TransportMessage::Error` wire shape lacks `event_id` | §5 | HIGH | **Closed by M6 (new) Phase 2** — Joe-locked design (§6.5 below) |
| Event-acceptance reject paths emit no wire-layer signal | §5 | HIGH | Closed by M6 (new) Phase 2 — same Joe-locked design |
| Design doc §3.1 line 204 `Error` shape mismatches implementation | §5 | LOW | Documentation correction during M6 (new) Phase 2 |
| J-080 + Pass-3 addendum assert `Error` is rejection signal — refuted | §5 | LOW (historical record) | No correction needed; audit supersedes without revising journal |

No follow-on task files filed as part of this audit close-out. Per Joe's D-069 discipline lock at 2026-05-18, downstream milestones go through their own Joe-locked design phase (Pass 1 → Pass 2 → Pass 3) before being declared ACTIVE — pre-filing a placeholder task file would create exactly the "drafted but not Joe-locked" ambiguity D-069 was written to prevent.

### 6.4 Post-audit work — Federation Event Propagation milestone (PENDING, design-phase needed)

The collective HIGH-severity findings (§2 Stage 6 absence, §3 peer-side ingestion absence) close in one downstream milestone:

**Provisional name:** Federation Event Propagation completion.  
**Status:** PENDING. Goes ACTIVE only after its own Joe-locked design phase (Pass 1 / Pass 2 / Pass 3) following the D-069 discipline.  
**Scope (audit-derived, not Joe-locked):** close the ongoing-event federation push gap. Either push-from-home, pull-from-peer, or hybrid. Persistent peer sessions vs. periodic reconciliation. New wire-protocol additions. The §3 validation-asymmetry sub-finding closes as a **precondition** of this milestone, not parallel — landing federation push without the validation hardening would land a vulnerability. Several §4 findings (500ms quiet-time fallback, no pagination) are related concerns; design phase decides whether to fold them in.  
**Blocks:** M6 (new) ACTIVE flip.

### 6.5 Post-audit work — M6 (new) Phase 2 scope adjustment (Joe-locked direct, no new design pass needed)

The §5 finding requires M6 (new) Phase 2 to add a symmetric rejection-side signal alongside the planned `TransportMessage::EventAccepted`. Joe locked the design call directly during the audit close-out conversation, eliminating the need for a Phase-2 design pass:

**Locked design.** A new field `event_id: Option<String>` is added at the `TransportMessage` envelope level (base of the transport-message hierarchy), populated when the message pertains to a specific event. **`EventAccepted` is the only new variant.** `Error` covers rejection by populating envelope `event_id`. **No new `EventRejected` variant.** The `error_code` namespace already encodes semantic meaning; envelope `event_id` provides correlation.

**Reasoning.** This mirrors the existing protocol architecture (Primitive base + SignedPrimitive extension; shared base + variant body). One well-placed field at the right layer beats adding structure elsewhere.

**Practical effect on M6 (new) Phase 2.** Original 6 deliverables in `docs/xgen_node_admin_ops_design.md` §5.2 stand. Add:

- `event_id: Option<String>` on the `TransportMessage` envelope.
- Event-rejection paths in `process_inbound` ([`xgen-node/src/app.rs:846-851`](xgen-node/src/app.rs:846), [`855-858`](xgen-node/src/app.rs:855), [`885-897`](xgen-node/src/app.rs:885), [`913-921`](xgen-node/src/app.rs:913), [`926-934`](xgen-node/src/app.rs:926)) emit `Error` with `event_id: Some(...)`.
- Client-side handling correlates envelope `event_id` against in-flight submissions.
- Confirm during implementation: serde derive handles `Option<String>` as omittable for backward-compat with pre-M6 clients (likely yes via `#[serde(skip_serializing_if = "Option::is_none")]`).

**No Pass 4 design session needed.** The design doc receives an edit-only update post-audit (§3.1 Error shape correction, §3.2–§3.4 envelope reference, new short §3.6 describing rejection path, §9 D-070 framing aligned). That edit is Chat Claude work, not Clair work.

**Structural realisation around the envelope-level `event_id`** (Rust type design, serde derives, module organisation, internal refactors that preserve wire shape) is delegated to Clair with the criterion that *cleaner is better*. Wire-format-visible changes beyond the locked `event_id` addition require Joe-lock. Threshold: would a future contributor reading the change ask "why was this decided?" — if yes, pause for Joe; if no, ship it as normal engineering judgment.

### 6.6 D-070 promotion is Chat Claude work (post-audit)

The D-070 draft in `docs/xgen_node_admin_ops_design.md` §9 should be promoted to DECISIONS.md as a numbered decision, with the corrected framing this audit established: D-070 now requires **both** `EventAccepted` AND a wire-layer rejection signal (the envelope-level `event_id` per §6.5) in M6 (new) Phase 2. The principle's empirical grounding is strengthened by this audit — the asymmetry runs deeper than the original framing assumed, and the principle's response (both signals as equal first-class primitives) is what's needed regardless. Promotion is a separate atomic action for Chat Claude + Joe after this audit closes.

### 6.7 What this audit did not do — and what stayed out of scope

- No code changes (audit, not a fix milestone — `tasks/PROPAGATION_RELIABILITY_AUDIT.md` §5.1).
- No follow-on task files filed (held per Joe's D-069 discipline lock).
- No Ch3 specification revisions (any spec change is a separate filing).
- No M6 admin verb work (deferred to M6 (new) Phase 2+).
- No edits to historical records (J-080, Pass-3 addendum, design doc §3.1) — the canonical doc going forward is this audit; corrections to the historical records are a documentation-pass task for Chat Claude post-audit.
- No tests added — pure code-trace audit. The 468-test baseline from J-080 is unchanged by this milestone.

### 6.8 Entry point for the next session

1. **Chat Claude + Joe:** D-070 promotion to DECISIONS.md with corrected framing per §6.6.
2. **Chat Claude + Joe:** documentation pass — correct `docs/xgen_node_admin_ops_design.md` §3.1 / §3.2 / §3.4 / §4.2 / §9 per §6.5 and §6.6; correct `docs/xgen_ch4_implementation.md` §implementation passages on Node-to-Node `sync_request` per §4.6.
3. **Chat Claude + Joe:** open the Federation Event Propagation milestone (Pass 1 / Pass 2 / Pass 3 design phase), with validation asymmetry as a precondition per §6.4.
4. **Clair (when M6 (new) Phase 2 ACTIVE):** implement envelope-level `event_id` per §6.5 locked design, with internal-realisation latitude as specified.

---

*End of audit.*

---

*End of document (in progress).*
