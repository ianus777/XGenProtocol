# R2-F01 — Client/Node Convergence Alignment — Design (Joe-locked)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

Design phase for the **R2-F01 fix-arc**, locked with Joe on 2026-06-05. It resolves
the audit's §4 open questions and locks the fork. **Doc-only, no code.** Builds on
`tasks/R2_F01_CLIENT_CONVERGENCE_AUDIT.md` v1.0 (the grounded gap + the framed fork).

The arc finishes the **client-side half of state-resolution convergence** that Arc C
(J-241) scoped out under the SR-D3 lock (node-side only). It is a **wiring + design-choice**
arc — the convergence engine (`derive_resolved`) is done and proven (Arc C); the only real
question was sourcing the engine's `identity_home_nodes` input on the client, settled below.

---

## 2. Grounding recap (from the audit + the design-phase code walk)

- `derive_resolved(events, my_node_id, identity_home_nodes)` is `pub` and reachable from
  `xgen-client` (path dep on `xgen-core`). So is `topological_sort`, `conflicts_in_log`, and
  `state_key_for_event` (confirm-at-pickup, CP-4).
- `identity_home_nodes` (`identity_id → home_node`) is consulted by **only** three resolution
  layers — **Layer 3** (home-node assertion), **Layer 5a** (node-priority ordering), **Layer
  5b** (federation recency). Each does `identity_home_nodes.get(sender)?`; on a miss the layer
  **abstains** (returns `None`) and resolution falls through. Layer 1 (ban>join), Layer 4
  (role), and Layer 5c (lexicographic backstop) consult **no** map.
- The map is built on the node by `build_identity_home_nodes(&IdentityRegistry)`. That binding
  (`identity → home_node`) lives in the node's registry (registration + identity replication),
  **NOT in the per-Space DAG** the client drains. The client therefore **cannot reconstruct a
  node-matching map from the drained log** — this is the load-bearing finding that decides Q1.
- The two `ops.rs` sites rebuild SpaceState **from scratch per read** (drain → timestamp-sort →
  `apply_event`). The `ai_service.rs:295` site applies **per-event in receive order** (the only
  incremental site).

---

## 3. Locked decisions (F01-D1 … F01-D6 — arc-local, D-069)

### F01-D1 — Fork = Option A (client re-derives)
The client computes its projection via `derive_resolved` over the drained log, in place of the
timestamp-sort + plain `apply_event` loops. **No new node surface.** Reuses the proven Arc-C
engine; the node stays authoritative (the client view is a local projection — status / members
/ display / AI-context / pacing).

### F01-D2 — Q1 resolution = A-pure (empty `identity_home_nodes`)
The client passes an **empty** `identity_home_nodes` map. Grounded rationale:

- The binding is not in the drained log (§2), so reconstruction is impossible; the only ways to
  get a node-matching map are *fetch it from the node* (a new surface) or *do without it*.
- With an empty map, Layers 3/5a/5b **abstain cleanly** (`get(sender)?` → `None` → fall-through)
  and **Layer 5c always elects a winner** → the client projection is **deterministic and
  self-consistent** (every permutation of the client's log converges to one state).
- The client's resolution can differ from the node's **only** on a conflict that the node
  decides at Layers 3/5a/5b — i.e. **concurrent AND same-event-type (Layer 1 abstains) AND
  same-role (Layer 4 abstains) AND cross-home-node** membership / key-rotation conflicts. This
  is a narrow, characterised class; single-node or same-home-node conflicts cannot reach those
  layers at all. The node remains the source of truth.

**Escalation (named, NOT built): A+thin-fetch.** If the close-phase reachability probe (F01-D5)
surfaces a *realistic client-reachable* conflict that lands on Layers 3/5a/5b, the fix escalates
to sourcing the map from a small node read surface (`identity_id → home_node` for the Space's
participants — a `HashMap<String,String>`, far lighter than serializing a full resolved
`SpaceState`). This is a flagged decision point, not an automatic build — honest deferral per
D-065, and it mirrors the M8 CP-A precedent (settle load-bearingness by measurement, not
assumption).

### F01-D3 — Vantage = `my_node_id = ""`
The client passes `""`, consistent with today's `apply_event(ev, "")`. The resolution layers do
not read `my_node_id` (Layer 5b orders off `space_state.home_node`); only `apply_federation_add`
consults it for the D-075 vantage. **CP-3:** confirm at pickup that `""` does not perturb
`apply_federation_add` resolution for the client's read sites (expected no-op — the client is
not a federation peer).

### F01-D4 — Apply discipline per site type
- **Rebuild-per-read sites** (`ops.rs:1302-1353`, `ops.rs:1538-1564`): straight swap of the
  drain → sort → `apply_event` loop for a single `derive_resolved(drained_log, "", &empty)`
  call. No conflict-gate needed — these already rebuild from scratch, so this mirrors
  `rehydrate_space_from_store` exactly.
- **Incremental site** (`ai_service.rs:295`, per-event receive-order loop): do **not**
  blind-rederive per event (O(N²) over a stream). Lock **batch-then-derive** — accumulate the
  inbound log and call `derive_resolved` at read/flush points — as the default. If the AI loop
  genuinely needs live per-event state, the alternative is to **mirror `ingest_event`'s gate**
  (incremental `apply_event` for the common case; full `derive_resolved` rebuild only when
  `conflicts_in_log` fires; `state_key_for_event` short-circuits message traffic before any log
  scan). **CP-2:** confirm the AI loop's read/flush boundary at pickup and pick batch-vs-gate
  accordingly.

### F01-D5 — Test shape (Q5)
- **Convergence property test** mirroring Arc C's permutation proof: the client's replay of a
  permuted log converges to the node's resolved snapshot (for conflicts decided by Layers
  1/4/5c — i.e. everything except the cross-home-node tie-breaks the empty map intentionally
  abstains on).
- **Reachability probe:** construct/enumerate the conflict shapes the client can actually
  observe and assert whether any lands on Layers 3/5a/5b under the empty map. A **negative**
  result confirms A-pure is sufficient (the divergence class is unreachable in practice). A
  **positive** result is a **flagged finding** that triggers the F01-D2 escalation review —
  it is recorded honestly, not silently shipped.

### F01-D6 — Commit split
- **C1** — the two `ops.rs` rebuild-per-read sites onto `derive_resolved`.
- **C2** — the `ai_service.rs` inbound loop (batch-then-derive or gate per CP-2).
- **Close** — doc-only (D-074): flip R2-F01 🟪→✅ in the Round-2 register (`ROUND_2_AUDIT.md`
  §5), record the reachability-probe result + any escalation flag, ROADMAP + JOURNAL +
  F01-D# eval.

---

## 4. Scope fence (explicitly OUT)

- **No client-side materialization cache** (audit F01-A5 — the client replays fresh per read;
  the SQLite-rebuildability concern is forward-looking, not a current defect). If a client cache
  is added later it rebuilds via whichever resolution path this arc locks.
- **No new node surface** unless the F01-D5 escalation fires.
- **Node authority unchanged** — this arc aligns the client's *local projection*; it does not
  touch the node's resolved view or any wire format.

---

## 5. Confirm-at-pickup checklist (D-078)

- **CP-1** — exact shape of the two `ops.rs` drain → sort → `apply_event` loops (line-level,
  ground against the live files; the audit pinned 1302-1353 and 1538-1564).
- **CP-2** — `ai_service.rs:295` read/flush boundary → batch-then-derive vs `ingest_event`-gate.
- **CP-3** — `my_node_id = ""` does not perturb `apply_federation_add` for the client read sites
  (F01-D3).
- **CP-4** — `derive_resolved`, `topological_sort`, `conflicts_in_log`, `state_key_for_event`
  all reachable from `xgen-client` (path dep); confirm imports.

---

## 6. Status & next-active

Design locked (Joe, 2026-06-05). **No code, no DECISIONS change** (F01-D# arc-local, D-069 —
the arc reuses Arc C's engine; no cross-arc invariant minted). R2-F01 stays 🟪 OPEN in the
Round-2 register (the fix is not done; design is its second phase).

**Next-active: the runbook** (`tasks/R2_F01_CLIENT_CONVERGENCE_IMPL.md`, Joe-approved) → then
Clair implements C1/C2 → doc-only close.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + the two-round audit principle.
