# XGen Protocol — State-Resolution Convergence Audit (Arc C / M8 Phase 0)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Method, sources, scope

### 0.1 Purpose

The D-071 Phase-0 subsystem audit that gates Arc C (M8/M9 multiparty) — the one
target the protocol gap audit (`tasks/PROTOCOL_GAP_AUDIT.md` §2.4) could not
pre-empt. Central question:

> When two or more Nodes apply **concurrent, conflicting State Events to the
> same state key**, does every Node converge on the *same* final state —
> deterministically, regardless of arrival order, respecting causality and
> authority — as ch3 §3.9.2 promises?

This audit walks the spec promise (§1) against the as-built code (§2), states
the gap precisely (§3), splits M8 vs M9 (§4), folds the scope-conditional gaps
(§5), and produces the design-question list that feeds the design beat (§6).
Track-1 doc; no locks (locks come at design). Earns a JOURNAL line only when
acted on (Rule 0 / D-074).

### 0.2 Sources

**Spec:** `docs/xgen_ch3_specification.md` §3.9 (full); ch2 L866/872/2097/2106
(causal/convergent state-res).
**Code:** `xgen-core/src/resolution/{algorithm,conflict,state_key,mod}.rs` ·
`xgen-core/src/space/state.rs` (`apply_event`) · `xgen-core/src/node/runtime.rs`
(dispatch/replay) · workspace-wide caller grep.

### 0.3 Verdict vocabulary

Reuses the gap audit's: **NO-GAP · GAP-CONFIRMED · SPEC-DRIFT · NEEDS-DESIGN ·
N/A**. Finding-IDs **SR-Fn** (append-only). Design questions **SR-Qn**.

---

## §1 — The convergence promise (spec §3.9)

ch3 §3.9 is **written in full** (not a stub). Normative surface:

- **§3.9.1** — state key = (category, key_field); two Events conflict iff same
  state key + no causal ordering. Message Events explicitly excluded.
- **§3.9.2 Convergence Guarantee** (L3013) — strong eventual consistency;
  resolution is a **pure function of Event content**; no timestamps as
  tiebreakers; commutative + associative. "Every Node independently applies the
  same stack to the same DAG and reaches the same winner without communication"
  (L853).
- **§3.9.3 Seven-Layer Resolution Algorithm** (L3027) — Layer 1 EventType
  priority · Layer 2 Auth Tier · Layer 3 Home-Node assertion · Layer 4 Role ·
  Layer 5a manual node ordering · Layer 5b federation recency · Layer 5c
  lexicographic event_id backstop.
- **§3.9.5** — split-brain recovery is **free**, a consequence of §3.9.2 (no
  special protocol).
- **§3.9.7** — snapshot is a *performance optimisation*; the **Event log is
  authoritative**; loser Events stay in the DAG permanently.

**The promise is precise and total: deterministic convergence on every Node via
the seven-layer stack, applied to the authoritative Event log.**

---

## §2 — As-built

### 2.1 The resolution machinery EXISTS and is unit-tested

The gap audit assumed convergent resolution was unbuilt. **It is not.** A
complete implementation is present:

- `resolution/algorithm.rs:37` — `resolve(conflicts, space_state,
  identity_home_nodes) -> &Event`: the full **seven-layer stack**, each layer a
  named fn (`layer1_event_type_priority` … `layer5c_lexicographic_backstop`),
  comments citing "spec 3.9.3". Layer 5c guarantees a unique winner for any
  non-empty set. Extensive `#[cfg(test)]` coverage (ban>join, role tiebreak,
  home-node assertion, lexicographic backstop, etc.).
- `resolution/conflict.rs:28` — `find_conflicts(events) -> Vec<(StateKey,
  Vec<&Event>)>` groups events by shared state key; `:49` `conflicts_with`
  filters by causal ordering (ancestor ⇒ no conflict).
- `resolution/state_key.rs:44` — `state_key_for_event`: the **conflict domain**.
  Keyed types: membership (`join`/`leave` by sender; `invite`/`kick`/`ban`/
  `node_eject`/`node_unban` by `target_identity`), `state.room_update` (by room),
  `state.space_update` (by space), `state.node_priority` (by space),
  `system.key_rotation` (by sender). All others (`message.*`, `federation.*`,
  `migration.*`, `mls.*`, …) → `None` (no resolution).
- `resolution/mod.rs` — `ResolutionError` (4001 `state_conflict_unresolvable`,
  4004 `state_key_invalid`), error-code surface matching §3.9.8.

### 2.2 The apply path is Phase-1 last-write-wins

`space/state.rs:449 apply_event` is the live state-derivation function. Its own
header (L456–457): *"For Phase 1 (no concurrent state changes), the most recent
event wins."* It is a per-type dispatch (`match &event.event_type → apply_*`),
applied **in causal/arrival order** as events arrive or replay. There is **no
conflict detection and no resolution call** anywhere on this path.

### 2.3 The wiring gap — zero production callers

Workspace grep (`find_conflicts` / `conflicts_with` / `resolution::` /
`::resolve(`, excluding `resolution/` itself and `#[cfg(test)]`): **zero
production callers.** (The only `resolve` hits are the unrelated `resolve_cmd`
in `aicontrol`.) The resolution module is built, tested, and **dead** —
identical pattern to PG-13 (tier-gate primitive present, not wired into join).

---

## §3 — Findings

| ID | Verdict | Sev | Finding | Evidence |
|----|---------|-----|---------|----------|
| **SR-F1** | **GAP-CONFIRMED** | **S1** | The §3.9.2 convergence guarantee is **not realised**: the seven-layer algorithm (§3.9.3) is implemented + unit-tested but never invoked. The live apply path applies events in arrival order with no conflict detection. Two Nodes receiving the same concurrent same-key events in different orders are **not guaranteed to converge** — the guarantee rests on code that never runs. | `resolution/*` (built) vs `state.rs:449` (LWW) + zero-caller grep |
| SR-F2 | GAP-CONFIRMED | S2 | `state.room_update` / `state.space_update` are in the conflict domain (`state_key_for_event` keys them) but `apply_event` dispatches both to `=> Ok(())` **no-ops** — even a resolved winner mutates no state. The room/space mutable-state appliers don't exist. | `state.rs` `StateSpaceUpdate \| StateRoomUpdate => Ok(())`; `state_key.rs:72/80` |
| SR-F3 | NEEDS-DESIGN | S2 | §3.9.7 snapshot model unrealised: there is no "apply winner, recompute snapshot on conflicting arrival" mechanism. Today's path mutates the snapshot in arrival order; a late-arriving event that *loses* (or a winner arriving after its loser) has no re-resolution step. Convergence needs the snapshot to be a function of the resolved log, not arrival sequence. | `state.rs:449` (no recompute); §3.9.7 |
| SR-F4 | NO-GAP | — | Causal ordering substrate is sound: `prev_events` DAG + cycle/root validation (`graph.rs`), topo-sort (D-076 v1.1), pending-buffer hold-on-unseen. The convergence layer sits *on top* of this and is the missing piece. | prior arcs (D-075/076) |
| SR-F5 | NEEDS-DESIGN | S2 | Layer inputs (`identity_home_nodes` map, `space_state.node_priority_order`, `federation_nodes`) are passed to `resolve()` by the (absent) caller. Wiring must source these at the apply site — `identity_home_nodes` in particular is algorithm-layer `HashMap<String,String>` not yet threaded from the identity registry. | `algorithm.rs:37` signature; registry seam |

**Headline (SR-F1):** the M8/M9 work is **not** "design + build a convergence
algorithm" (the gap audit's framing). It is **"wire the existing algorithm onto
the apply path + prove convergence end-to-end across Nodes."** The hard part
shifts from algorithm design to (a) the apply/replay integration and (b)
multi-node convergence testing — the integration gap, not the math.

---

## §4 — M8 vs M9 scope split

- **M8 ("improved pass") — the wiring + proof.** Thread `find_conflicts` +
  `resolve` into the apply/replay path (SR-F1); make the snapshot a function of
  the resolved log (SR-F3); supply the missing room/space-update appliers
  (SR-F2); source layer inputs at the apply site (SR-F5). Prove it with
  multi-node convergence tests (concurrent same-key events, divergent arrival
  orders, split-brain merge → identical snapshots). This is the bulk of the
  value and is bounded — the algorithm already exists and is tested in isolation.
- **M9 ("redesign") — conditional.** Only if M8's integration shows the existing
  layered stack can't reach §3.9.2 convergence without structural change
  (e.g. conflicted-vs-unconflicted partitioning à la Matrix state-res-v2, or
  auth-chain rewalk on each merge). M9 is a *contingency the M8 proof either
  triggers or retires* — not a committed second arc.

**Recommendation:** scope Arc C as **M8 = wire + prove**; hold M9 as a named
contingency gated on M8's convergence test results.

---

## §5 — Scope-fold: PG-08 / PG-10 / PG-12

The gap audit left these "scope-conditional on C." Verdict:

- **PG-12 (per-Room permission override)** — **OUT of M8 core.** Independent of
  convergence; it's an enforcement-layer gap (Arc D). M8 doesn't need it; folding
  it in would widen scope without serving the convergence proof.
- **PG-10 (AI capability hard-enforcement / AI-not-owner)** — **OUT.** Pure
  enforcement (Arc D), orthogonal to resolution.
- **PG-08 (Thread primitive)** — **OUT of M8 core, watch at M9.** A Thread is a
  new keyed state category; *if* M9 re-partitions the conflict domain, adding
  `thread.*` is cheaper done then. No M8 dependency.

**Net:** none of PG-08/10/12 are M8 prerequisites. M8 stays tightly scoped to
the convergence wiring. This retires the §2.4 "double-design risk" — the
enforcement/primitive arcs (D, E) are genuinely independent of the resolution
wiring.

---

## §6 — Design-question list (feeds the design beat)

- **SR-Q1** — Apply-path shape: does `apply_event` stay arrival-order and a
  *separate* resolution pass recompute the snapshot on conflicting arrival, or
  does the apply path itself become conflict-aware (detect on ingest → resolve →
  apply winner)? (Bears on SR-F1 + SR-F3.)
- **SR-Q2** — Snapshot recompute granularity: full replay-from-log on any
  detected conflict (simple, correct, slow) vs incremental per-state-key
  re-resolution (fast, more surface). §3.9.7 allows either (snapshot is just an
  optimisation).
- **SR-Q3** — `identity_home_nodes` source + lifetime at the apply site: from the
  identity registry? Cached? (SR-F5.) Determines a possible xgen-core↔registry seam.
- **SR-Q4** — Room/space-update appliers (SR-F2): build the real mutable-state
  appliers now (M8) or confirm they're genuinely empty by design? (They carry
  state keys — likely a real omission, not intentional.)
- **SR-Q5** — Test substrate: in-process multi-`SpaceState` convergence harness
  (cheap, deterministic) vs two-node integration (heavier). The `.events`
  integration test (J-229) precedent suggests in-process for the convergence
  property + one two-node smoke for the seam.
- **SR-Q6** — Layer 2 (Auth Tier) stays inert (single-Tier Phase 1) — confirm
  it's left as future-proofing, not activated in M8.

---

## §7 — Verdict + next step

**One S1 (SR-F1), two S2 design items (SR-F2, SR-F3, SR-F5), one NO-GAP
(SR-F4).** The convergence guarantee is specified and the algorithm is built —
**the gap is the wiring and the end-to-end proof**, not the algorithm. Arc C is
real, bounded, and correctly the next milestone; its risk lives in integration
and testing, not in unsolved math.

**Next step: the design beat** — resolve SR-Q1–Q6, lock the apply-path shape
(arc-local SR-D#, D-069), then runbook → Clair implements (M8). M9 held as a
contingency on M8's convergence-test outcome.

**Audit complete (v1.0).** Status ACTIVE until a recommendation is acted on
(Rule 0 / D-074).
