# R2-F01 — Client/Node Convergence Alignment — Phase-0 Audit
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

Phase-0 audit for the **R2-F01 fix-arc** — the highest-severity Round-2 finding (S2) and
the only one touching UI correctness. **Doc-only, no code.** This audit grounds the gap and
frames the one real design fork; it does **not** lock the design (that is the next phase,
Joe-lock). The arc finishes the **client-side half of state-resolution convergence** that
Arc C (J-241) deliberately scoped out under the SR-D3 lock (node-side only).

**The gap (from Round 2 §3.2).** The node resolves state via `derive_resolved` (Arc C, the
seven-layer convergent engine). The **client does not** — it replays the DAG via
**timestamp-ordered plain `apply_event`** (Phase-1 last-write-wins). Under genuine concurrent
same-key conflict, or clock skew (the client orders by wall-clock timestamp, not causal
DAG), the client's local SpaceState can **diverge** from the node's resolved view. UI reads
this client projection, so the divergence is a UI-correctness concern.

---

## 2. Grounded findings

### F01-A1 — The three client replay sites (the gap, confirmed)
All replay via `sort_by(timestamp)` then plain `apply_event(ev, "")` — no `topological_sort`,
no `derive_resolved`, no conflict resolution:
- `xgen-client/src/ops.rs:1302-1353` — the `ai_status` path: `drain_space_events` → timestamp
  sort (root-first) → `apply_event`.
- `xgen-client/src/ops.rs:1538-1564` — the projection helper (same shape).
- `xgen-client/src/ai_service.rs:295` — the AI service inbound loop: per-event `apply_event`
  as events arrive (receive-order, not even timestamp-sorted).

Consumers: ops reads (status/members/display), AI-behaviour context, pacing. Node stays
authoritative; impact is **client-local views**.

### F01-A2 — `derive_resolved` is reachable from the client (Option A is available today)
`pub use derive::derive_resolved` (`resolution/mod.rs:16`); `xgen-client` depends on
`xgen-core` by path. So the client **can** call the proven engine — no new crate, no new
node surface required for the re-derive option.

### F01-A3 — …but `derive_resolved` needs inputs the client may not have (the real crux)
Signature: `derive_resolved(events, my_node_id: &str, identity_home_nodes: &HashMap<String,
String>)`.
- **events** — the client has them (`drain_space_events`).
- **my_node_id** — the client is not a Node; today it passes `""` to `apply_event` (the D-075
  vantage falls to the non-Node branch). Passing `""` to `derive_resolved` is consistent with
  current client behaviour, but must be confirmed not to perturb `apply_federation_add`
  resolution.
- **identity_home_nodes** — the **crux**. The node builds this per-rebuild from its
  `IdentityRegistry` (Arc C CP-C, `build_identity_home_nodes`). The client has **no equivalent
  registry**. If it passes an empty/incomplete map, its resolution can differ from the node's
  in the tie-break layers that consult home-nodes → **divergence persists even with
  `derive_resolved`**. Sourcing a node-matching `identity_home_nodes` on the client is the
  load-bearing design question for Option A.

### F01-A4 — The node exposes the raw event log, not a resolved snapshot (Option B cost)
The client gets state by draining the **raw event log** (`drain_space_events`) and replaying;
there is no node read-surface that returns a resolved `SpaceState`. So the "client trusts a
node-pushed resolved snapshot" option (Option B) requires a **new node read surface + wire
format** — more surface than Option A.

### F01-A5 — No client-side materialization cache today (SQLite concern is forward-looking)
The client replays the drained log **fresh per read** — there is no persistent client-side
SQLite/display cache to rebuild. The absorbed M8 "SQLite display-cache rebuildability" concern
is therefore **forward-looking**, not a current defect; today the only question is *which
resolution the per-read replay uses*. (If a client cache is added later, it rebuilds from the
synced log via whichever resolution path this arc locks.)

### F01-A6 — Honest scope note
This is a **wiring + design-choice arc**, not a from-scratch build — the convergence
algorithm is done and proven (Arc C). The risk is the input-sourcing decision (F01-A3) and
the choice of fork below, not the math.

---

## 3. The design fork (framed for Joe-lock — NOT locked here)

**Option A — client re-derives via `derive_resolved`.** Swap the three timestamp-sort +
`apply_event` loops for `derive_resolved` over the drained log. Reuses the proven engine; no
new node surface. **Cost/crux:** source a node-matching `identity_home_nodes` on the client
(F01-A3) — either reconstruct it from the drained events, fetch it from the node, or accept a
characterised divergence in home-node-dependent tie-breaks. Resolve the `my_node_id=""` vantage
question.

**Option B — node exposes a resolved snapshot; client trusts it.** Matches the SR-D3 spirit
("clients consume node-resolved state"). The client stops replaying; the node serves its
already-resolved `SpaceState`. **Cost:** a new node read surface + wire format (F01-A4); the
client loses any offline/optimistic local derivation.

**Option C — hybrid** (noted, likely overkill): node snapshot for display, thin client
re-derive for offline/optimistic paths. Adds both costs; only worth it if offline use is a
real requirement.

**Audit lean (for design to weigh, not a lock):** Option A is the minimal, lowest-surface fix
that directly closes the divergence (same engine + same inputs → same output), *provided*
F01-A3's input-sourcing is solvable cleanly. Option B is architecturally purer per SR-D3 but
adds a node surface. The `identity_home_nodes` sourcing question (F01-A3) is what design must
settle first — it may itself tip the choice toward B if the client genuinely cannot reconstruct
a node-matching map.

---

## 4. Open questions for the design phase
1. **F01-A3 resolution** — how does the client obtain a node-matching `identity_home_nodes`
   (reconstruct from events · fetch from node · accept characterised divergence)? This gates
   Option A.
2. **Vantage** — is `my_node_id=""` correct for client-side `derive_resolved`, or does the
   client need the connected node's id?
3. **Fork choice** — Option A vs B vs C (likely falls out of Q1).
4. **AI service site** — `ai_service.rs:295` applies per-event in receive-order; does it
   re-derive on each event (expensive) or batch + re-derive (latency)? A separate sub-question
   from the ops read paths.
5. **Test shape** — the convergence proof for the client mirrors Arc C's permutation property
   tests (client replay of permuted logs converges to the node's resolved snapshot).

---

## 5. Status & next-active

Phase-0 audit complete; **no code, no design locks** (audit precedes design, D-071). R2-F01
stays 🟪 OPEN in the Round-2 register (the fix is not done; this is its first phase).
**Next-active: the design phase** — resolve §4 Q1 (the `identity_home_nodes` crux) first, then
lock the fork (Option A/B/C) with Joe, then a runbook, then Clair implements. Reuses Arc C's
proven `derive_resolved` engine.

Per Rule 0 + D-065 + D-069 + D-071 (audit-precedes-dependent-design) + the two-round audit
principle.
