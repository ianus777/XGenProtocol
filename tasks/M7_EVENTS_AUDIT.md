# M7-events arc — Phase-0 Audit (reality map)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

The M7-events arc Phase-0 audit. The arc adds the deferred client + node `.events` pipes from M7 `--aicontrol` v1 (closed command-pipes-only at J-205). M7 v1 deferred them because the events pipe **cannot be built as a pure adapter** in the current tree: the checkpoint-#2 trace (J-203) found no in-process broadcast to tap, and the dedicated-`.events`-WS alternative collides with the Node's one-sender-per-identity fan-out registry.

This audit centres on that **gating Node multi-connection-per-identity fan-out change** (`ClientSenders` → `Vec<(conn_id, Sender)>`), maps the live sites it touches, carries the J-203 Q1/Q2/Q3 findings so they are not re-derived, and lists the open questions the design phase must lock. Doc-only — no code.

Builds on J-203 (checkpoint-#2 / reshape), the M7 design doc's deferred §AC-D3b (subscription-filter grammar) + §AC-D3c (`state` schema), and the canonical `docs/xgen_aicontrol_implementation.md` §3 (events, marked DEFERRED at C6).

---

## 1. Reality map (live trace, xgen-node)

### 1.1 The fan-out registry type

`xgen-node/src/fanout.rs`:

```
pub type ClientSenders =
    Arc<Mutex<HashMap<IdentityXgid, mpsc::Sender<OutboundMsg>>>>;
```

The doc-comment is explicit Phase-1 framing: *"one device per Identity, so one channel per Identity. On disconnect the entry is removed; on reconnect a new entry is installed."* This single-sender-per-identity assumption is the collision source.

Sibling registry `FederationPeerSenders = Arc<Mutex<HashMap<NodeXgid, mpsc::Sender<OutboundMsg>>>>` has the identical single-sender shape (one WS per peer pair, F-2a). Out of v1 scope, but symmetric — flagged under Q5 below.

`OutboundMsg` variants: `Event(Event)` · `HistoryBatch { events }` · `SyncComplete { since, new_tip, continue_from }`.

### 1.2 The three live sites

| Site | Location | Current behaviour |
|---|---|---|
| **Register** | `handle_connection`, client branch (`app.rs`) | `mpsc::channel(1024)` then `senders.insert(identity_id.clone(), out_tx)` — **unconditional insert = overwrite** |
| **Drain** | same handler's `select!` loop | `Some(out_msg) = out_rx.recv()` → `conn.send_event` / `send_transport` |
| **Remove** | same handler, on disconnect | `senders.remove(&identity_id)` — **remove-by-identity** |

The overwrite + remove-by-identity pair is the J-203 collision: a second same-identity WS clobbers the first sender on insert, and either side's disconnect removes the shared key.

### 1.3 `apply_fanout` — the consumer

`fanout.rs::apply_fanout(req, author_id, runtime, client_senders)`:

1. Locks runtime briefly → `recipients = space.members.keys()` (+ joiner history if `new_joiner`), then drops the runtime lock.
2. Locks `client_senders`; for each `rid != author_id`: `senders.get(rid)` → `tx.try_send(OutboundMsg::Event(..))`.
   - Success → `fanout_delivered` trace. Channel full → `fanout_dropped_channel_full` trace (the event is **dropped**, cap-1024).
3. Joiner history: `senders.get(joiner_id)` → `try_send(HistoryBatch)`.

Called from **three sites**, all passing `&client_senders`: the client recv path (`handle_connection`), the federation catch-up drain, and the F-2 steady-state loop (`run_federation_session_post_handshake`).

The two `.get(rid)` / `.get(joiner_id)` lookups are the exact points the retype touches on the consumer side.

---

## 2. The gating change (shape)

Retype the registry value to a per-connection vector:

```
HashMap<IdentityXgid, Vec<(ConnId, mpsc::Sender<OutboundMsg>)>>
```

| Site | Change |
|---|---|
| Register | push `(conn_id, tx)`; create the entry if absent (no overwrite) |
| Remove | drop the matching `conn_id` from the Vec; remove the identity key only when its Vec empties |
| `apply_fanout` | both `.get(rid)` / `.get(joiner_id)` become *iterate the Vec, `try_send` to each*; per-`(rid, conn_id)` `fanout_delivered` / `fanout_dropped_channel_full` traces preserved |

This is a **node mechanism change, out of adapter scope** (J-203) — the reason it gates the events pipes and was split out of M7 v1. It is the load-bearing commit of the arc.

**Prime invariant (C1 regression):** with exactly one connection per identity, a Vec-of-one fans out byte-for-byte identically to today. The whole existing fan-out + federation suite must stay green with the retype in place.

---

## 3. Carried J-203 findings (do not re-derive)

- **Q1 — registry collision + Vec-sender shape.** Documented in §1–§2 above; this *is* the gating change.
- **Q2 — subscription = from-now-forward live.** Fan-out registration alone delivers; **no `SyncRequest`** path; historical catch-up is the command pipe's `history` verb, not the events pipe. The events pipe is a live tail.
- **Q3 — gaps visible across reconnect; process-wide `event_subscriptions`.** No silent replay; cap-1024 mpsc drops on full (existing `fanout_dropped_channel_full` behaviour, now per-connection). A **process-wide `event_subscriptions` registry** threaded to both servers is the C-item that lets the `state` verb report a real count (it ships honest `0` in v1 on both binaries).

---

## 4. Open questions for the design phase (EV-D# candidates)

1. **`ConnId` type + source.** Monotonic per-process counter vs uuid; minted where (accept loop / per resident entry point). Must be unique for the lifetime of a connection and cheap to compare.
2. **Adapter-after-retype thesis (the central question).** Confirm that once fan-out is multi-sender, the `.events` pipe is *"a second registered `(conn_id, sender)` + an AC-D3b filter view"* rather than new instrumentation — i.e. the J-203 split-trigger (b) blocker is dissolved by the retype, and the rest of the arc is adapter work again.
3. **Events-pipe registration seam.** Where the `.events` consumer registers/deregisters its `(conn_id, sender)`. For the **client** side this is the resident opening a *second* same-identity WS to its home Node (J-203 reading B) and tailing its fan-out to the pipe; for the **node** side, the resident is itself the fan-out hub (a local registration / in-process tap).
4. **Filter application point.** AC-D3b grammar (AND-across / OR-within; two wildcard forms; entitlement-is-ceiling; `nodes` Node-only) applied at the events-pipe drain as a **view** over what fan-out already delivers — never an access request. Confirm AC-D3b is reused as-locked or revised.
5. **Node-side scope + `FederationPeerSenders`.** Does the node `.events` pipe (and the `nodes` filter) need `FederationPeerSenders` to take the same Vec retype, or does that registry stay single-sender / out of v1? Lean: ClientSenders-only for v1; federation-peer fan-out unchanged.
6. **`event_subscriptions` registry.** Process-wide, threaded to both servers; the data behind the `state` field that ships `0` in v1. Confirm shape (per-connection subscription records) + the `state.event_subscriptions` count source (AC-D3c).

---

## 5. Proposed arc shape (sequence, not locked)

- **Phase 0** — this audit.
- **Design** — `EV-D#` locks (arc-local, D-069): `ConnId`; the `ClientSenders` retype contract; the events-pipe registration seam (client + node); `event_subscriptions` registry; reuse-or-revise AC-D3b / AC-D3c.
- **Runbook + commits (≈4–5), 1 checkpoint:**
  - **C1** `ClientSenders` retype + the 3 `apply_fanout` call-site updates + register/remove rewrite — **Joe-lock checkpoint before C1** (load-bearing mechanism change). Prime-invariant regression: single-connection fan-out byte-for-byte.
  - **C2** process-wide `event_subscriptions` registry + `state` wiring (both binaries).
  - **C3** client `.events` pipe + `subscribe`/`unsubscribe` + AC-D3b filter.
  - **C4** node `.events` pipe + Node-only `nodes` filter.
  - **C5** close (D-074 atomic, doc-only).

---

## Cross-refs

- JOURNAL J-203 (checkpoint-#2 reshape; the Q1/Q2/Q3 source) + J-205 (M7 v1 close, command-pipes-only).
- `tasks/M7_AICONTROL_DESIGN.md` — §AC-D3b (subscription-filter grammar) + §AC-D3c (`state` schema), both deferred here.
- `docs/xgen_aicontrol_implementation.md` §3 (events, DEFERRED banner at C6) — the canonical spec the pipes implement.
- `xgen-node/src/fanout.rs` — `ClientSenders`, `apply_fanout`, `OutboundMsg`, `FederationPeerSenders`.
- `xgen-node/src/app.rs` — `handle_connection` (register/drain/remove) + the 3 `apply_fanout` call sites.
- DECISIONS.md: D-065 (adapter-not-feature), D-066 (`--batch`/`--aicontrol` split), D-069 (arc-local IDs), D-074 (atomic milestone close), D-078 (confirm-at-pickup), D-082 (administrator vs operator).

---

*Phase-0 audit ACTIVE. Next: open `tasks/M7_EVENTS_DESIGN.md` and lock the EV-D# set with Joe — `ConnId` + the retype contract first.*
