# XGen Protocol — State-Resolution Convergence Design (Arc C / M8)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is

The design beat for Arc C / M8, following `tasks/STATE_RESOLUTION_AUDIT.md`
(v1.0). The audit's headline (SR-F1): the seven-layer convergence algorithm is
built + unit-tested (`xgen-core/src/resolution/`) but has **zero production
callers** — the live apply path (`space/state.rs:449 apply_event`) is Phase-1
last-write-wins. M8 = **wire the algorithm onto the node apply path + prove
convergence end-to-end**; the algorithm itself is not redesigned.

Decisions **SR-D1–SR-D6 LOCKED** (Joe, 2026-06-03), arc-local per D-069 —
promotion to DECISIONS.md (if any) evaluated at close. No DECISIONS/ROADMAP/
JOURNAL change at design (D-074: those land when acted on, i.e. at C1+).

---

## §1 — Locked decisions

### SR-D1 — Apply-path shape: pure applier + ingest conflict gate (SR-Q1)

`apply_event` stays the pure per-event applier it is today — **unchanged,
well-tested, not disturbed.** A **conflict-detection gate** is added on the
node derivation path: for each incoming/replayed State Event, check
`conflicts_with` against the events already applied to that state key.

- **No conflict** (the common case) → incremental `apply_event`, exactly today's
  behaviour. Fast-path, zero new cost, regression-safe.
- **Conflict detected** → trigger re-resolution (SR-D2).

Rejected: making `apply_event` itself conflict-aware (forces rollback of
already-applied losers; membership set-ops aren't cleanly invertible across
arbitrary interleavings).

### SR-D2 — Recompute model: full snapshot rebuild from the resolved log (SR-Q2)

On a detected conflict, rebuild the affected Space snapshot from the
authoritative Event log:

```
causal_sort(log) → find_conflicts → resolve(each conflict set) → 
    replay winners through apply_event → new snapshot
```

Losers stay in the DAG permanently (§3.9.7). This is **provably correct** —
`snapshot ≡ fold(resolve(causal_sort(log)))` *is* the §3.9.2 convergence
statement — and avoids the cross-key dependency trap (a winning ban must also
strip room membership, etc.). Conflicts are rare, so full rebuild cost is
acceptable. **Incremental per-key recompute is a deferred optimisation, not
v1** (D-065: correctness before speed).

### SR-D3 — Resolution is node-side in M8 (SR-Q3 + sub-decision 1)

The gate + recompute live on the **node** derivation path (`node/runtime.rs`
replay + dispatch). **Clients consume node-resolved state** and do not resolve
independently in M8 — a client lacking home-node data would resolve via a
different layer (5c instead of 3) and **diverge from the node**, violating
§3.9.2. Client apply sites (`xgen-client/src/{ai_service,ops}.rs`) stay
unchanged (display derivation). `identity_home_nodes` is sourced from the
identity registry at the node apply site; the algorithm's
`HashMap<String,String>` parameter is **unchanged** (good seam — don't widen it
here; Pass-2 XGID widening is its own arc). Client-side resolution is a
post-M8 possibility once identity replication (§3.13) guarantees the client has
the same layer inputs — explicitly out of scope.

### SR-D4 — M8 proof scope: membership-core; room/space-update minimal-or-deferred (SR-Q4 + sub-decision 2 + SR-Q4a)

The convergence proof centres on **membership conflicts** — they exercise all
seven layers (Layer 1 type priority, Layer 3 home-node, Layer 4 role, 5a/5b/5c
backstops) and are the richest conflict domain.

**SR-Q4a resolved:** ch3 carries **no content schema** for `state.room_update`/
`state.space_update` — they are code-only EventTypes (keyed in
`state_key_for_event` but applied as `=> Ok(())` no-ops, SR-F2). Therefore M8
does **not** build rich room/space-update appliers. Default = **defer** the
SR-F2 appliers to a follow-on (or to whenever ch3 schemas the update events);
M8 may add a trivial name/topic applier only if it costs nothing and helps a
test. The proof does not depend on them.

### SR-D5 — Test substrate: in-process permutation property + one two-node smoke (SR-Q5)

- **Primary (the proof)** — in-process convergence property tests: N
  `SpaceState`s, the same concurrent same-key event set fed in **different
  permutations**, assert **byte-identical final snapshots**. Cheap,
  deterministic, many permutations — this is where §3.9.2 is actually proven.
- **Secondary (the seam)** — one two-node integration smoke reusing the phase9
  harness, guarding the live `runtime.rs` wiring. (J-229 layering precedent.)

### SR-D6 — Layer 2 (Auth Tier) stays inert (SR-Q6)

Single-Tier Phase 1 → all events tie at Layer 2 → falls through. Already
acknowledged in `algorithm.rs`. **No M8 work**; left as future-proofing for
cross-tier Spaces.

---

## §2 — Integration sites (grounded)

Node derivation apply sites — the wiring targets (exact handles =
confirm-at-pickup, D-078):

- `xgen-core/src/node/runtime.rs` — **:326** (rehydrate/replay), **:470 / :501 /
  :509** (dispatch apply). These are the node-side `let _ = state.apply_event(&ev,
  &my_node_id)` sites. The gate wraps these.
- `xgen-core/src/message/exchange.rs` — **:981 / :1894** derivation loops
  (`let _ = s.apply_event(ev, "")`). Confirm at pickup whether these are
  on the node derivation path (gate them) or are isolated builders (leave).
- **Out of scope (unchanged):** `xgen-client/src/{ai_service,ops}.rs`,
  `space/mod.rs` builders, all `#[cfg(test)]` sites.

The gate is one chokepoint function (e.g. `apply_event_resolving` or a
`SpaceState`-level "apply with conflict check") so the wiring is a single new
seam the four runtime sites route through — not four edits with drift risk.

---

## §3 — The convergence test (what "proven" means)

The property test asserts the §3.9.2 guarantee directly:

1. Construct a conflict scenario: e.g. concurrent `membership.ban(target=X)` by
   an admin vs `membership.join(X)` by X, no causal ordering (shared
   `prev_events` tip).
2. For **every permutation** of arrival order, build a fresh `SpaceState` and
   feed the events through the resolving apply path.
3. Assert all permutations yield **identical** final snapshots (member set,
   roles, bans, rooms) — and that the winner matches the layer the scenario
   targets (Layer 1 ban>join, Layer 4 role tiebreak, Layer 5c lexicographic).
4. Split-brain case (§3.9.5): two divergent partial logs merged in either order
   → identical snapshot, no special recovery path.

Scenarios span the layers exercised by membership (1, 3, 4, 5a, 5b, 5c).

---

## §4 — Confirm-at-pickup (D-078, for Clair)

1. **Gate home + signature** — `SpaceState` method vs free fn in `resolution`;
   how `conflicts_with` gets "events already applied to this key" (the snapshot
   doesn't retain per-key event lists today — may need the log, which the node
   has via the store).
2. **`identity_home_nodes` sourcing** at the runtime apply site — from the
   identity registry handle; cached vs per-call.
3. **exchange.rs :981 / :1894** — node-path (gate) or builder (leave)?
4. **Full-rebuild trigger granularity** — rebuild the one Space on any conflict
   in it; confirm the store exposes the Space's full event log for replay
   (EventStore `range`/`collect_sync_history` from J-228).

---

## §5 — Scope boundary + M9 contingency

**M8 = wire + prove** (SR-D1–D6). **M9 = named contingency**, triggered only if
M8's convergence tests show the layered stack can't reach §3.9.2 without
structural change (conflicted/unconflicted partitioning, auth-chain rewalk on
merge). Not a committed second arc. **PG-08/10/12 are NOT M8 prerequisites**
(audit §5) — Arcs D/E stay independent.

---

## §6 — Next step

Runbook → `tasks/STATE_RESOLUTION_IMPL.md`: commit plan (gate seam + recompute
→ runtime wiring → convergence property tests → two-node smoke → close), all
green-on-landing, confirm-at-pickup §4 resolved at pickup. Then Clair
implements M8. Doc-only at design — suite unchanged at 1035/0/2.

**Design complete (v1.0). SR-D1–D6 LOCKED.**
