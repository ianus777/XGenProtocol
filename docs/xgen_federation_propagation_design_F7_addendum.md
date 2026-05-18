# XGen Federation Event Propagation — Design (F-7 addendum)

> **Status**: PENDING  
> Version: 0.1  
> Date: May 2026  
> **Last updated**: 2026-05-18 (F-7 surfaced and Joe-confirmed in Pass 2 conversation)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools. Pass 2 addendum to `docs/xgen_federation_propagation_design.md`; merged into the canonical document at Pass 3 per the D-069 canonical-document rule.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## About this addendum

This file is a Pass 2 addendum to `docs/xgen_federation_propagation_design.md`. As the canonical design doc grew through F-1 to F-6, full-file rewrites for each subsequent F-item became disproportionately expensive. F-7 through F-10 are therefore written as standalone addenda; Pass 3 consolidates all addenda into the canonical document in a single careful rewrite.

Reading order: open the main design doc first to absorb F-1 through F-6, then read this addendum for F-7. At Pass 3 the addenda are deleted after their content is folded in.

---

## 10. Framework decision F-7 — Pagination on `collect_sync_history`

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

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

`[JOE-LOCK: confirmed in Pass 2 conversation 2026-05-18; formal promotion at Pass 3]`

**Decision — implementation-configurable, reference-implementation default of 1000 events per batch. NOT protocol-fixed.**

Following F-6b's precedent. The protocol mandates the *mechanism* (paginated responses, `limit` and `continue_from` fields); the *value* is an implementation default with operator override.

**Concrete configuration.** Config field `[sync].batch_size` in both `xgen-node_config.toml` and `xgen-client_config.toml`. Default 1000 events per batch. Operator may override.

**Choice of 1000 as the default.**

- Large enough that most realistic catch-up cases finish in one batch (typical Space activity over hours-to-days is well under 1000 events).
- Small enough that individual response sizes stay well within reasonable WebSocket frame limits and serialise quickly.
- Round number with no magical significance — operators reading the config see "this is a sizing knob" rather than "this is a protocol constant."

**Reasoning recorded.** Same as F-6b. Hardcoding a page size into the protocol would force every deployment to compromise: LAN-only deployments could safely run with larger batches; constrained-bandwidth deployments might want smaller. Configurable from day one prevents repeating the "magic number bake-in" problem the milestone is already fixing for the 500ms quiet-time heuristic.

### 10.5 Wire-shape additions

The F-6 + F-7 wire changes compose as follows:

```
TransportMessage::SyncRequest {
    since: String,            // existing
    limit: Option<u32>,       // F-7 — optional; absent means implementation default
}

TransportMessage::SyncComplete {
    since: String,            // F-6 — echo of the request's since cursor
    new_tip: String,          // F-6 — responder's current DAG tip for the relevant Space
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

## F-7 lock state

| Sub-item | Decision |
|---|---|
| F-7 | Option 1 — fold in: implement response-size pagination in this milestone |
| F-7a | Implementation-configurable, default 1000 events per batch, `[sync].batch_size` in config |

Pass 2 Pass 3 will fold this section into the canonical design doc as §10, update §3.1 scope to include the pagination item, update §3.3 non-scope table to remove the "pagination — possibly in scope" line, and delete this addendum file.

---

*End of F-7 addendum.*  
