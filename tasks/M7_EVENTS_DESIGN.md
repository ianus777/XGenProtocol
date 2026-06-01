# M7-events arc — Design (decision log)
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

The M7-events arc design decision log. Locks are `EV-D#` (arc-local per D-069), resolved one-by-one with Joe. Builds on `tasks/M7_EVENTS_AUDIT.md` (v1.0), J-203 (the Q1/Q2/Q3 findings), and the M7 design doc's deferred §AC-D3b (subscription-filter grammar) + §AC-D3c (`state` schema). The arc adds the client + node `.events` pipes deferred from M7 v1, on top of the gating Node multi-connection-per-identity fan-out change. No code until the runbook.

## Lock order

| # | Decision | Status |
|---|---|---|
| EV-D1 | `ConnId` type + source | 🔒 **LOCKED** (A — `ConnId(u64)` over a process-global atomic) |
| EV-D2 | `ClientSenders` retype contract | 🔒 **LOCKED** (`Vec<(ConnId, Sender)>`, kind-agnostic) |
| EV-D3 | Events-pipe registration seam (client + node) | 🔒 **LOCKED** (client second-WS rides the retype; node = observer registry in `apply_fanout`) |
| EV-D4 | Filter application point (AC-D3b reuse-or-revise) | 🔒 **LOCKED** (reuse AC-D3b; shared `matches` predicate; node filters-before-send, client filters-at-drain) — **amended v1.1 at C2**: `matches` is 3-param (`event_nodes` caller-supplied) |
| EV-D5 | `FederationPeerSenders` scope (v1 in or out) | 🔒 **LOCKED** (out of scope; stays single-sender) |
| EV-D6 | `event_subscriptions` registry + `state` count (AC-D3c) | 🔒 **LOCKED** (shared registry both servers; node folds into the EV-D3 observer registry; `state` = process-wide count) |

---

## EV-D1 — `ConnId` type + source — 🔒 LOCKED (A)

**Decision: a `ConnId(u64)` newtype, minted from one process-global `AtomicU64` at each registration entry point.**

- `Copy`, zero-dep, sortable, log-friendly. `fetch_add(1, Relaxed)` at each accept path (the primary client WS + the new events-pipe WS). Uniqueness holds for process lifetime (u64 does not realistically wrap).
- Carries **no connection-kind tag** in v1 — `apply_fanout` stays kind-agnostic (see EV-D2). Type-carries-contract: the id identifies a connection, nothing more.
- Rejected: UUID (heavier + a dep, overkill for an in-process map key); reusing an existing id (none exists — client connections aren't keyed today; `session_id` is federation-only).
- Tests pass literal ids.

---

## EV-D2 — `ClientSenders` retype contract — 🔒 LOCKED (kind-agnostic `Vec`)

**Decision: retype the registry value to `Vec<(ConnId, mpsc::Sender<OutboundMsg>)>`.**

```
HashMap<IdentityXgid, Vec<(ConnId, mpsc::Sender<OutboundMsg>)>>
```

- **Inner = `Vec`, not `HashMap<ConnId,_>`** — N-per-identity is tiny (1–2), iteration is the hot path, remove is O(n) on a trivially small n.
- **Register** → push `(conn_id, tx)`; create the identity key if absent (no overwrite).
- **Remove** → drop the matching `conn_id`; **prune the identity key when its Vec empties** (keeps "is this identity connected?" honest for any other reader).
- **`apply_fanout`** → iterate the recipient's Vec, `try_send(Event)` to each; per-`(rid, conn_id)` `fanout_delivered` / `fanout_dropped_channel_full` traces preserved (cap-1024 drop-on-full unchanged).

**Two consequence calls (locked), both keeping `apply_fanout` connection-kind-agnostic:**

1. **Author exclusion stays by *identity*, not `conn_id`.** The author's own events-pipe connection does not see its own posted event echoed. Coherent: the command pipe's `EventAccepted` ack already confirms the send; the events pipe shows *others'* traffic. Preserves today byte-for-byte.
2. **Joiner `HistoryBatch` goes to *all* of the joiner's connections** at the registry layer — but the events-pipe drain forwards only filtered `Event` and ignores `HistoryBatch` / `SyncComplete`. So Q2's "live-only, no history" lives at the **events-pipe drain**, not in the registry. `apply_fanout` need not know a connection's kind.

**Why this confirms the adapter-after-retype thesis (audit Q2 / J-203 split-trigger b).** Because the retype is mechanical + kind-agnostic, all events-pipe specialness (live-only, filtering) sits at its own consumer — so the rest of the arc is adapter work, not new fan-out instrumentation, on the client side. (Node side asymmetry resolves at EV-D3.)

**Prime invariant (C1 regression):** with exactly one connection per identity, a Vec-of-one fans out byte-for-byte identically to today. The existing fan-out + federation suite must stay green with the retype in place.

---

## EV-D3 — events-pipe registration seam — 🔒 LOCKED (client second-WS; node observer registry)

**Decision: the two sides are asymmetric.** Client rides the EV-D2 retype as a pure adapter; node needs an observation tap broader than membership.

**Client `.events` (pure adapter).** The client resident opens a *second* same-identity WS to its home Node, registers as a second `(conn_id, sender)` on the node's multi-sender `ClientSenders`, receives normal **member-scoped** fan-out, and streams filtered `Event`s down the `.events` pipe to the AI driver. Member-scoping is correct for a client — it sees only its own Spaces' traffic. No node-side mechanism beyond the retype.

**Node `.events` (observer registry — the asymmetry).** The node resident *is* the fan-out hub, and `apply_fanout` only sends to `space.members.keys()` — so a `ClientSenders` registration would see only one identity's member Spaces, not all hosted-Space traffic. A node operator/AI watching `.events` wants everything the node fans out. Resolution: a **node-observer registry** — `Vec<(ConnId, filter, Sender)>` of node-level observers — consulted **inside `apply_fanout` after the member loop**, fed every fanned event regardless of membership.

- Reuses the single fan-out chokepoint; the node observes exactly what it fans out (the right grain for "what's happening in my Spaces").
- Node-internal mechanism, in arc scope (sibling to the retype). The observer concept is confined to the **node hub** — the client path stays kind-agnostic / pure adapter.
- **Grain (honest flag):** Option A observes **fan-out output** (events delivered to members), not the accept/persist chokepoint. Rejected/zero-member events are NOT observed in v1. Seeing those would be the accept-chokepoint tap (broader than the J-203 framing) — explicitly out of v1.

**Rejected:** node observer as a synthetic member of every hosted Space (pollutes membership state); accept/persist-chokepoint tap (second lower tap point, couples to persistence, broader than v1 needs).

## EV-D4 — filter application point — 🔒 LOCKED (reuse AC-D3b; node filters-before-send, client at-drain)

**Decision: AC-D3b's grammar is reused verbatim** (AND-across / OR-within; the two wildcard forms `*` and `state.*`; `empty == all`; entitlement-is-ceiling; `nodes` Node-only; malformed → `BAD_ARGUMENT` pre-stream, the `subscribe` being the first message). This arc decides only *where* it runs — the homes EV-D3 already created:

- **Client** — the parsed filter lives on the `.events` pipe session; applied at the **pipe drain** (the second-WS delivers member-scoped `Event`s; the drain forwards only matches). `nodes` present → `BAD_ARGUMENT`.
- **Node** — the parsed filter is the `filter` field of the EV-D3 observer record `(ConnId, filter, Sender)`; applied **before `try_send`** inside `apply_fanout`'s observer loop — only matches enter the observer channel. `nodes` is meaningful here.

**Shared predicate (D-067, no drift):** a pure `fn matches(&Filter, &Event, event_nodes: &[NodeXgid]) -> bool` used by both sites (signature amended at C2 — see v1.1 below).

**v1.1 amendment (C2, 2026-06-01) — `matches` is 3-param; the node set is caller-supplied.** The original v1.0 signature `matches(&Filter, &Event) -> bool` was **unimplementable as written** for the `nodes` dimension: an `Event` carries no uniform node field. Node provenance is partial and non-uniform — node-authored events (`membership.node_eject`/`node_unban`, `state.node_priority`) put the node in `sender`; `state.federation_add` (+ likely `migration.federation_notify`, `reputation.defederation_signal`) in untyped `content["node_id"]`; and **every other event's** node association is the Space's `home_node`, which is **runtime state** (`SpaceState`), not on the event. This is exactly EV-D5's "event's own provenance **+ runtime state**." Resolution (Option 3, Joe-endorsed): the predicate takes `event_nodes: &[NodeXgid]`, the set of nodes the event involves; the `nodes` arm is `filter.nodes.is_empty() || filter.nodes ∩ event_nodes ≠ ∅`. The **caller** derives `event_nodes` — the **C3 node side** from runtime (`SpaceState.home_node` + `federation_nodes` + sender-if-Node + `content["node_id"]`); the **client** passes `&[]` (and rejects a non-empty `nodes` filter at the C5 call site, so the arm is vacuously "all"). This honors EV-D4's single-pure-predicate intent *better* than the literal 2-param form — one shared predicate, pure, with the runtime-sourced dimension parameterized rather than smuggled into a fake event field. Lives at `xgen-common/src/aicontrol/filter.rs` (C2). **Create-event resolution (folded in):** the `spaces` arm uses the canonical `Event::effective_space_id()` (empty `space_id` → `event_id`; `xgen-common/src/wire.rs`), so a `spaces:[S]` filter also sees S's own `state.space_create`; `xgen-node::fanout::event_space_id` converges onto this helper in C3 (no lasting two-copy drift).

**Why the node filters before send (A), not at-drain (B):** filter-before-send keeps non-matching events out of the bounded cap-1024 observer channel — narrow subscriptions don't flood the buffer, so the Q3 drop-on-full risk does not worsen at the high-volume hub. The client has no choice but at-drain (its member-scoped WS already narrowed entitlement; the hub re-filter isn't available to it).

**Entitlement-as-ceiling holds by construction** on both: client = its member Spaces, node = its hosted Spaces; the filter only narrows. The node-vs-client asymmetry mirrors EV-D3 and keeps the bounded channel clean where volume is highest.

## EV-D5 — `FederationPeerSenders` scope — 🔒 LOCKED (out of scope; stays single-sender)

**Decision: `FederationPeerSenders` is NOT retyped; it stays `HashMap<NodeXgid, Sender>` and is untouched by this arc.**

- `apply_fanout` is the **superset chokepoint** — every accepted event passes through it (locally-submitted *and* federation-received, since the F-2 loop calls `apply_fanout` for received events). The EV-D3 node observer, fed inside `apply_fanout`, already sees all accepted node traffic regardless of origin.
- `FederationPeerSenders` is the **outbound transport to peers** (one WS per peer pair, F-2a) — a delivery channel, not an observation source. It carries nothing the observer needs that `apply_fanout` doesn't already see.
- The `nodes` filter dimension (AC-D3b, Node-only) is satisfied from the event's own provenance + runtime state, not from the peer-senders registry.
- No "two residents per peer" collision exists — the events pipe is a client/operator observer surface, never a federation peer — so the multi-sender retype has no reason to reach this registry.

**Honest check:** no accepted event reaches peers (`apply_federation_push`) without also passing `apply_fanout` (which runs for every accepted event, even zero-local-member hosted Spaces) — so the fan-out hub is the complete observation view.

## EV-D6 — `event_subscriptions` registry + `state` — 🔒 LOCKED (shared registry; node folds into observer registry; process-wide count)

**Decision: the field becomes live (it shipped honest `0` in M7 v1 with no events pipe). Three points:**

1. **Single source of truth, threaded to both servers (per binary).** A shared `Arc<Mutex<…>>` written by the **events-pipe server** (subscribe / unsubscribe / events-conn close) and read by the **command-pipe server** (for `state`). This is the J-203 "process-wide, threaded to both servers": `state` lives on the command pipe; subscriptions live on the events pipe.
2. **On the node, the registry IS the EV-D3 observer registry** — not a second structure. `Vec<(ConnId, Filter, Sender)>` is already process-wide + reachable from `apply_fanout`; the events-pipe server pushes/prunes entries, `apply_fanout` reads it (new param through the 3 call sites), command-pipe `state` reads `.len()`. On the **client** there is no `apply_fanout`, so the registry is the set of active `.events` sessions `(ConnId, Filter, <ws handle>)` — same shared-Arc pattern.
3. **`state.event_subscriptions` = process-wide *count*, not a per-driver list.** A driver's command pipe and events pipe are separate connections with separate `ConnId`s and **no session link in v1** (AC-D4 token deferred) — so `state` cannot honestly attribute subscriptions to the caller. The truthful v1 value is the process-wide active-subscription count (replacing the shipped `0`). Named consequence: two drivers each holding a subscription → each `state` reads `2` (live-tail count on this process), correct for an operator surface. A per-driver list is the session-link feature → belongs to the `--aicontrol` hardening arc with the AC-D4 token, not here.

---

## Cross-refs

- `tasks/M7_EVENTS_AUDIT.md` (v1.0) — the Phase-0 reality map this design acts on.
- JOURNAL J-203 (Q1/Q2/Q3) + J-205 (M7 v1 close).
- `tasks/M7_AICONTROL_DESIGN.md` — §AC-D3b (filter grammar) + §AC-D3c (`state` schema).
- `xgen-node/src/fanout.rs` — `ClientSenders`, `apply_fanout`, `OutboundMsg`, `FederationPeerSenders`.
- `xgen-node/src/app.rs` — `handle_connection` register/drain/remove + the 3 `apply_fanout` call sites.
- DECISIONS.md: D-065, D-066, D-069, D-074, D-078, D-082.

---

*All EV-D# locked (EV-D1 · EV-D2 · EV-D3 · EV-D4 · EV-D5 · EV-D6). Design phase complete. Doc stays ACTIVE through implementation, flips to COMPLETED at arc close (per the FAC / auth-module precedent). Next: the implementation runbook `tasks/M7_EVENTS_IMPL.md` for Clair.*
