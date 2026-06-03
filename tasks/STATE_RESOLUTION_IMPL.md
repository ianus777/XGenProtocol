# XGen Protocol — State-Resolution Convergence Runbook (Arc C / M8)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-03  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — What this is

The implementation runbook for Arc C / M8, executing
`tasks/STATE_RESOLUTION_DESIGN.md` (SR-D1–D6 LOCKED). M8 = **wire the existing
seven-layer `resolve()` onto the node apply path + prove convergence**. The
algorithm (`xgen-core/src/resolution/`) is not redesigned. Suite baseline
(J-236): **1035/0/2**. Build output `C:/cargo-targets/XGenProtocol`.

**Grounding that shapes the plan (confirm-at-pickup #1 resolved):** the node
already rebuilds `SpaceState` from the resolved log via
`topological_sort(store.range(0))` — see `ingest_event`'s `StateSpaceCreate`
arm (out-of-order replay), `rehydrate_space_from_store` (`runtime.rs:291`), and
`all_events` (`runtime.rs`). The resolving rebuild reuses this machinery: it
reads the Space's full event log from the store, so the snapshot does **not**
need to retain per-key event lists for `conflicts_with`.

---

## §2 — The resolving-derivation core (the shape both code commits build on)

A pure derivation that turns a Space's event log into a convergent snapshot:

```
fn derive_resolved(events: Vec<Event>, my_node_id, identity_home_nodes) -> SpaceState:
    sorted   = topological_sort(events)            // causal order (existing fn)
    groups   = find_conflicts(&sorted)             // [(StateKey, Vec<&Event>)]
    losers   = {}                                  // HashSet<event_id>
    for (_key, set) in groups:                     // set.len() >= 2
        winner = resolve(set, &auth_state, identity_home_nodes)   // seven layers
        losers += set \ {winner}                   // losers stay in DAG (§3.9.7)
    state = SpaceState::from_*create(first sorted create event)
    for ev in sorted:
        if ev.event_id not in losers: state.apply_event(ev, my_node_id)
    return state
```

The winner of each conflict set applies; losers are skipped (not deleted — they
remain in the DAG). No conflict → `losers` empty → identical to today's replay.
This **is** the §3.9.2 statement: `snapshot ≡ fold(resolve(causal_sort(log)))`.

### CP-A (confirm-at-pickup, **the key semantic question** grounding surfaced)

`resolve()` consults `space_state` for Layer 4 (role) / 5a (node priority) / 5b
(federation recency). **Which state snapshot is the auth basis during a
rebuild?** Recommended v1 (state-res-v2's shape): derive an **unconflicted-state
basis** first — fold only the events whose state key has *no* conflict — then
resolve each conflict set against that basis, then apply winners. This avoids
the circularity of resolving against a snapshot that itself depends on conflict
outcomes. If this needs locking it becomes **SR-D7** at pickup. Simpler
fallback if the unconflicted basis proves fiddly: resolve against the
current best-effort snapshot (accepted as a v1 approximation, flagged). Clair
picks at pickup with the membership-conflict tests as the oracle.

---

## §3 — Commit plan

All commits green-on-landing (build + clippy `-D warnings` + suite). D-074
per-commit canonical-record discipline applies at C2+; C1 is code+tests only.

### C1 — Resolving-derivation core + convergence property tests (xgen-core, no wiring)

- Add `derive_resolved` (home: `resolution/` or a `SpaceState` assoc fn —
  CP-B). Reuses `topological_sort` + `find_conflicts` + `resolve` + `apply_event`.
- Resolve CP-A (auth basis) here, proven by the tests.
- **The proof (SR-D5 primary):** in-process permutation property tests — N
  `SpaceState`s, same concurrent same-key set in **every arrival permutation**,
  assert **byte-identical** snapshots. Scenarios spanning the membership layers:
  Layer 1 (ban>join on same target), Layer 4 (role tiebreak, two admins),
  Layer 5c (lexicographic backstop, two equal-role senders), split-brain
  (§3.9.5: two partial logs merged either way → identical).
- **Not wired into ingest yet** — `derive_resolved` has no production caller at
  C1 (sibling to how `resolve` sits unused today; this commit makes it *usable*
  and *proven*, C2 makes it *used*). Green on landing.

### C2 — Wire into the node apply path (xgen-core runtime)

- Route node state-derivation through the resolving core. Two sites:
  - `rehydrate_space_from_store` (`runtime.rs:291`) — replace its
    `topological_sort → apply each` with `derive_resolved`. Pure win:
    cold-start now convergent.
  - `ingest_event` (`runtime.rs`) `_ =>` arm — the incremental apply. Add the
    **conflict gate (SR-D1):** if the incoming state-keyed event conflicts with
    the stored log for its key (`conflicts_with` over `store.range(0)`),
    trigger a `derive_resolved` rebuild of that Space; else incremental
    `apply_event` (today's fast-path, unchanged). The create-arms' existing
    replay also routes through `derive_resolved`.
- Source `identity_home_nodes` from `identity_registry` at the site (CP-C).
- Client apply sites (`xgen-client`) **untouched** (SR-D3). `exchange.rs:981/1894`
  per CP-D.
- One chokepoint so the four runtime apply sites share one seam (no drift).
- D-074: this is the code change; canonical-record edits ride at close.

### C3 — Two-node integration smoke + SR-F2 decision (xgen-node)

- One two-node convergence smoke reusing the phase9 harness (SR-D5 secondary):
  two nodes receive the same concurrent same-key events in different orders →
  identical resolved snapshots. Guards the live seam.
- SR-F2 (room/space-update appliers): **deferred by default** (SR-D4 — ch3 has
  no content schema for them). Add a trivial name/topic applier only if a test
  needs it; otherwise leave the no-op and note the deferral at close.

### Close — doc-only, D-074 atomic

- `STATE_RESOLUTION_AUDIT.md` + `STATE_RESOLUTION_DESIGN.md` + this runbook →
  COMPLETED.
- ROADMAP: M8 (state-resolution convergence) → done entry; version bump.
- CLAUDE.md PLAY → M8 CLOSED; next-active = Joe selects next arc (D/E/F/G/H/I or
  M9 if triggered).
- JOURNAL J-237 (or next free).
- DECISIONS.md: evaluate SR-D# promotion. Likely **arc-local (D-069)** — M8
  *implements* the §3.9.2 guarantee, it doesn't establish a new cross-cutting
  discipline. If CP-A's auth-basis rule generalises, consider one promotion.
  Joe's call at close.
- **M9 disposition:** record whether M8's convergence tests passed cleanly
  (M9 retired) or surfaced a structural limit (M9 triggered, scoped from the
  failing scenarios).

---

## §4 — Confirm-at-pickup (D-078)

- **CP-A** *(the important one)* — auth basis for `resolve()` during rebuild:
  unconflicted-state basis (recommended) vs best-effort-snapshot (fallback).
  May lock as SR-D7. §2.
- **CP-B** — home of `derive_resolved`: `resolution/` free fn vs `SpaceState`
  assoc fn. (Leans `resolution/` — it orchestrates `find_conflicts`+`resolve`,
  both there; but it builds a `SpaceState`, so an assoc fn reads well too.)
- **CP-C** — `identity_home_nodes` sourcing: `IdentityRecord.home_node`
  (`NodeXgid`) is per-record; confirm `identity_registry` exposes an
  iterate/lookup to build the `HashMap<String,String>` the algorithm wants.
  Cache vs per-rebuild.
- **CP-D** — `exchange.rs:981` / `:1894` derivation loops: on the node
  derivation path (route through the gate) or isolated builders (leave)?
- **CP-E** — conflict-detection granularity in `ingest_event`: per-incoming
  cheap check (does this state-keyed event share a key with a non-causal event
  in the store?) → rebuild only on hit; confirm `conflicts_with` is the right
  primitive over `store.range(0)`.

---

## §5 — Out of scope (recorded)

PG-08 / PG-10 / PG-12 are **not** M8 prerequisites (audit §5) — Arcs D/E stay
independent. Client-side resolution is post-M8 (needs §3.13 identity
replication to give clients the same layer inputs). Layer 2 (Auth Tier) stays
inert (SR-D6). Incremental per-key recompute (vs full rebuild) is a deferred
optimisation (SR-D2).

**Runbook complete (v1.0).** Next: Clair picks up C1 (resolve CP-A + CP-B
first). Per Rule 0 + D-065 + D-069 + D-074 + D-078.
