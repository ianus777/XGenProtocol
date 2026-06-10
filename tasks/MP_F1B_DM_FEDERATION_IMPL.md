# MP-F1b — cross-node DM convergence (membership-driven DM federation) — IMPLEMENTATION RUNBOOK

> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-09  

> **v1.1 note:** §1–§7 below are the **pre-Z plan** (members-only helper + the §3.2 "no new send code"
> premise). The live witness falsified that premise; **Design Z** is what shipped. Read **§9 (as-built,
> SHIPPED)** + design `MP_F1B_DM_FEDERATION_DESIGN.md` §9 for the actual mechanism. §3 (the apply-site
> enumeration) stands as-grounded.
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The Clair-facing build plan for **MP-F1b** under the **Option-2 lock (Joe, J-332)** and
**F1B-D1..D7** (`tasks/MP_F1B_DM_FEDERATION_DESIGN.md`). Ships (iii) membership-driven DM
federation **harness-scoped**: a single idempotent NodeRuntime helper populates a DM Space's
`federation_nodes` from its current resolved members' home nodes at every apply site, the existing
push path federates DM events, and the gate-B production gap is **routed** (F1B-D5), not built.

**Authored before implementing; Joe-lock first (D-071).** Two-commit discipline holds: **Clair's
code FIRST, then Chat's doc-bridge.** This runbook is the code half.

**Deliverables (Joe's task brief):**
1. `repopulate_dm_federation_nodes(&mut SpaceState, &IdentityRegistry)` — the NodeRuntime helper.
2. The apply-site enumeration — **grounded** against live `ingest_event` + `rehydrate_space_from_store` (§3, the open D1 runbook item — done here).
3. Four NodeRuntime units — resolvable-set / omit-unresolvable / regular-Space-unchanged / leave-shrinks.
4. The MP-C-07 (`mp_r1_c4`) witness flip KNOWN-FAIL → harness-green-with-boundary, with RED-on-revert.

**Boundaries (do not cross):** production identity→home-node **discovery** (F1B-D5) is **routed, not
built**. **No production witness** for the un-seeded case (F1B-D4 — not expressible on G-6 rails).
F1b composes with the shipped MP-F4 (frontier-anchor + A1) — reads Space membership, fires **after**
`derive_resolved`, reopens nothing. Invariant E stays a DECISIONS candidate — promote at close.

**Read §7 (grounding surprises) before locking** — the grounding turned up five precision items the
design's prose collapsed; none change the design's intent, but the implementer (and Joe) should see
them.

---

## 2. Commit shape

Two work commits + the close (Clair's half; Chat's doc-bridge is a separate commit per standing
order):

- **C1 — helper + apply-site wiring + 4 units** (xgen-core). The whole production change + its
  NodeRuntime-level proof. Single atomic commit (helper, 4 hook sites, 4 unit tests).
- **C2 — MP-C-07 witness flip** (xgen-mptest): the `mp_r1_c4` doc-comment + `#[ignore]` reason flip
  to harness-green-with-boundary; the stale `mp_r1_c5:187` cross-ref corrected. Heavy `--ignored`
  run + RED-on-revert demonstration recorded.
- **Close (doc-bridge, Chat seat):** `MP_findings.md`, matrix MP-C-07 row, MP_R1 §11 already amended
  (J-332), CLAUDE/JOURNAL/ROADMAP; promote invariant E (D-069); place the discovery arc (F1B-D5) on
  the ROADMAP horizon.

DoD per commit: `cargo build` 0 + clippy clean (default **and** `--features harness-control` where
touched) + the relevant suite green. No "commit pushed" line — Joe pushes.

---

## 3. The apply-site enumeration (the open D1 item — GROUNDED against live `main`)

The helper must re-fire after **every** path that (re)derives or mutates a DM `SpaceState`, because
`derive_resolved` rebuilds a fresh state whose DM `federation_nodes` starts **empty** (the DM
constructors set `Vec::new()`; `apply_federation_add` is rejected for DMs). The single apply
chokepoint is **`ingest_event`** — `dispatch_event` delegates to it (`self.ingest_event(event)`,
runtime.rs:1543), and `replay_spaces_from_dir` calls it directly (app.rs:4192). `rehydrate_space_from_store`
is the **separate** cold-start path. So the hook sites are:

| # | Site | File:line (live) | When it fires | Hook |
|---|---|---|---|---|
| 1 | `ingest_event` **create arm** | runtime.rs:656–662 (`StateSpaceCreate \| StateDmSpaceCreate` → `derive_resolved` → `spaces.insert`) | DM create applies (local **and** federation-received via dispatch→ingest) | populate before `spaces.insert` |
| 2 | `ingest_event` **incremental conflict-rebuild** | runtime.rs:675–680 (`if conflict { derive_resolved → spaces.insert }`) | a genuine concurrent membership conflict rebuilds the DM | populate before `spaces.insert` |
| 3 | `ingest_event` **incremental fast-apply** | runtime.rs:681–683 (`else if let Some(state) = spaces.get_mut { state.apply_event(...) }`) | the common DM `membership.join` / message apply (no conflict) | populate after `apply_event` |
| 4 | `rehydrate_space_from_store` | runtime.rs:517–519 (`derive_resolved → self.spaces.insert`) | cold-start restart (app.rs:4238) + migration cutover (migration_driver.rs:446/467) | populate before `self.spaces.insert` |

**Precision refinement (surface for Joe — §7.1):** the design's "incremental `_ => apply_event`
arm" is in code **two** sub-sites — the conflict-rebuild branch (site 2) **and** the fast-apply
branch (site 3). Both must call the helper. The common DM join path is site 3.

**Not separate hook sites (covered):** `replay_spaces_from_dir` → `ingest_event` (sites 1–3); the
three `drain_pending_*` helpers re-dispatch via `dispatch_event` → `ingest_event` (sites 1–3). One
helper, four call sites; no per-site logic beyond the call → no D-067 drift.

---

## 4. Steps

### S1 — the helper (F1B-D1 / F1B-D2 / F1B-D3)

A free function beside `build_identity_home_nodes` (runtime.rs:1895; same shape — takes the registry,
DM-only, no `&self`):

```rust
/// MP-F1b (F1B-D1/D2/D3) — populate a DM Space's `federation_nodes` from its
/// current resolved members' home nodes. DM-only; `apply_federation_add` stays
/// intact (`DmFederationNotAllowed`) — no third-party node ever receives DM
/// content. Idempotent (full replace), so it re-fires safely at every apply site.
///
/// F1B-D2: the FULL members' home-node set, self-included (cross-node symmetric —
/// both parties' nodes derive the identical set; the push path skips self).
/// F1B-D3: a member whose record is NOT in this node's registry is OMITTED — no
/// crash, no guess, no fabricated home. That omission IS the gate-B boundary
/// (harness-seeded → resolves → federates; production stranger → omitted →
/// deferred behind the discovery arc, F1B-D5).
fn repopulate_dm_federation_nodes(state: &mut SpaceState, registry: &IdentityRegistry) {
    if !state.dm_constraints_active {
        return; // DM-only — regular Spaces use apply_federation_add (untouched).
    }
    let mut nodes: Vec<NodeXgid> = Vec::new();
    for id in state.members.keys() {
        if let Some(rec) = registry.get(id) {          // resolvable → include
            if !nodes.contains(&rec.home_node) {
                nodes.push(rec.home_node.clone());
            }
        }
        // unresolvable → omit (F1B-D3 boundary)
    }
    nodes.sort();                                       // determinism — see §7.6
    state.federation_nodes = nodes;
}
```

Grounded types: `SpaceState.federation_nodes: Vec<NodeXgid>` (state.rs:234); `state.members:
HashMap<IdentityXgid, SpaceMember>` (state.rs:221); `state.dm_constraints_active: bool` (state.rs:239,
`pub`); `IdentityRegistry::get(&IdentityXgid) -> Option<&IdentityRecord>` (registry.rs:110);
`IdentityRecord.home_node: NodeXgid` (registry.rs:47); `NodeXgid` derives `Ord` (flavours.rs:129) → `.sort()` available.

### S2 — wire the four apply sites (§3)

**Sites 1, 2, 4** (populate-before-insert): the destructure binds `identity_registry` (sites 1/2) /
`self.identity_registry` (site 4) disjointly from `spaces`, so populate `state` before the insert:

```rust
// site 1 + site 2 (inside `let NodeRuntime { spaces, identity_registry, .. } = self;`):
if let Some(mut state) = derive_resolved(log, &my_node_id, &ihn) {
    repopulate_dm_federation_nodes(&mut state, identity_registry);
    spaces.insert(state.space_id.clone(), state);
}
// site 4 (rehydrate — not a destructure; `self.identity_registry` is free):
if let Some(mut state) = derive_resolved(events, &my_node_id, &ihn) {
    repopulate_dm_federation_nodes(&mut state, &self.identity_registry);
    self.spaces.insert(state.space_id.clone(), state);
}
```

**Site 3** (populate-after-apply): `state` is `spaces.get_mut(&space_id)`; `identity_registry` is a
sibling destructured binding (disjoint):

```rust
} else if let Some(state) = spaces.get_mut(&space_id) {
    let _ = state.apply_event(&event, &my_node_id);
    repopulate_dm_federation_nodes(state, identity_registry);
}
```

Borrow-checker note: at sites 1/2/3 `identity_registry` is already destructured from `self` (the
existing `let NodeRuntime { spaces, stores, graphs, identity_registry, .. } = self;` at runtime.rs:574).
At site 4 `self.identity_registry` is borrowed immutably by `state` (a local that doesn't borrow
`self`), released before `self.spaces.insert`. All four compile without restructuring.

### S3 — the four NodeRuntime units (proof plan, F1B-D3 / F1B-D7-C)

NodeRuntime-level (per the design proof plan), using the existing test helpers in the runtime.rs test
module: `NodeRuntime::new(node_key)`, `register_identity`, `make_record(key, home_node)` (registers
an identity with a chosen `home_node` — or omit a register to make a member **unresolvable**),
`pubkey_uri`, `build_dm_space_create_event` / `build_membership_event` / `sign_event` (from
`xgen-core::space`). Build node A; ingest a DM create (alice creates DM with bob) + bob's join; assert
`node.spaces[dm_id].federation_nodes`.

1. **resolvable-set** — register alice@`A` (self) + bob@`B` (a distinct home id); ingest DM create +
   bob's join → `federation_nodes == sorted([A, B])` (self-included, F1B-D2). The cross-node
   convergence property in miniature.
2. **omit-unresolvable** (F1B-D3, the honest-by-construction proof) — register alice@`A` only, **do
   not register bob**; ingest DM create + bob's join → `federation_nodes == [A]` (bob's home omitted,
   no panic/guess). This is the gate-B boundary, witnessed at the unit level.
3. **regular-Space-unchanged** (DM-only) — create a **plain** Space (`from_space_create`,
   `dm_constraints_active=false`), members alice+bob both resolvable; ingest membership → the helper
   early-returns → `federation_nodes` stays whatever the non-DM path set (empty here, no
   `state.federation_add`). Asserts the helper never touches regular Spaces.
4. **leave-shrinks** (F1B-D7-C) — resolvable DM {alice, bob}; assert `federation_nodes == [A, B]`;
   ingest bob's `membership.leave`; assert `federation_nodes` shrinks (bob's home gone — `[A]`). The
   helper recomputes the full set each apply, so a leave shrinks it for free.

### S4 — the MP-C-07 witness flip (F1B-D4) — **C2**

`xgen-mptest/tests/mp_r1_c4.rs::mp_c_07_dm_across_nodes_converges` (currently `#[ignore = "KNOWN
FAIL ..."]`, the committed repro). The assertions are unchanged (both DM messages converge A↔B); only
its status + doc-comment flip:

- **`#[ignore]` reason** → the heavy form its green siblings use, so it runs in the `--ignored` suite:
  `#[ignore = "heavy: spawns two harness-control xgen-node + 2 clients; run with --ignored"]` (matches
  `mp_c_02` / `mp_c_03`).
- **Doc-comment** → the F1B-D4 boundary text (replace the "KNOWN FAIL → routed finding" body):
  > MP-C-07 — DM private space across nodes. **Harness-green-with-boundary (MP-F1b).** DM federation
  > forms when members' home nodes are resolvable (the harness G-6-seeds the relationship + replicates
  > identities, so both parties resolve) → both DM messages converge A↔B. **Production convergence to a
  > not-yet-known counterparty is DEFERRED** behind the routed "production identity→home-node
  > discovery" arc (F1B-D5): the un-seeded case is not expressible on current G-6 rails (sibling to
  > MP-A-01(ii) PENDING) → **no production witness is claimed**, by design.
- **Stale cross-ref:** `xgen-mptest/tests/mp_r1_c5.rs:187` ("the federated MP-C-07 (`mp_r1_c4`) stays
  KNOWN-FAIL until MP-F1b") → "harness-green-with-boundary (MP-F1b)."

**The witness mechanism (so the run is understood, not magic — §7.3):** the director's `initiate`
re-add-peers naming the DM Space, so alice's `a2`/`a3` ride the **F-1a tip-exchange catch-up**
(`stream_federation_delta` iterates `shared_spaces`, not `federation_nodes`). bob's join + `b4` reach A
via the **helper-populated B→A push** (`apply_federation_push`, `federation_nodes`): bob joins on B →
helper → B `federation_nodes = {A, B}` → B pushes bob's join + `b4` to A. Both messages on both nodes
→ GREEN.

**RED-on-revert (do, don't assume) — the F1B-D4 genuineness check:** revert the helper (comment out the
four calls) → DM `federation_nodes` stays empty on B → `apply_federation_push` early-returns → B never
pushes bob's join or `b4` to A → the `b4`-on-all-nodes assertion fails on A **and** alice's membership
projection stays `{alice}` (bob's join never propagated) → MP-C-07 **RED** on two counts. Restore →
GREEN. Record the RED output in the commit message; it is not vacuous.

### S5 — verification (do, don't assume)

```text
cargo build -p xgen-core
cargo clippy -p xgen-core --lib --tests --all-features -- -D warnings
cargo test  -p xgen-core                       # 4 new units + the full M8/assert_converges net green
cargo build -p xgen-node --features harness-control && cargo build -p xgen-client
cargo test  -p xgen-mptest --test mp_r1_c4 -- --ignored --nocapture   # mp_c_07 GREEN (+ mp_c_02/03 stay green)
```

D-077 backward-coherence: the full xgen-core + xgen-node + mptest suites stay green (DM-only helper;
the `state.rs:3631–3760` `apply_federation_add` tests are SpaceState-level — the helper is
NodeRuntime-level, never fires there; the xgen-node "federation_nodes empty" sites are regular Spaces,
helper early-returns). `assert_converges` + the 285 integration + M8 net green.

---

## 5. Definition of Done

- [ ] `repopulate_dm_federation_nodes` shipped (S1) — DM-only, full set, self-included, omit-unresolvable, sorted.
- [ ] Four apply sites wired (S2 / §3): ingest create arm + conflict-rebuild + fast-apply + rehydrate.
- [ ] Four NodeRuntime units green (S3): resolvable-set / omit-unresolvable / regular-Space-unchanged / leave-shrinks.
- [ ] MP-C-07 (`mp_r1_c4`) flipped to harness-green-with-boundary (S4); `mp_r1_c5:187` cross-ref corrected.
- [ ] **RED-on-revert demonstrated** (S4) — revert helper → MP-C-07 RED on `b4` + membership projection; output recorded.
- [ ] **No production witness claimed** — recorded honestly (F1B-D4); the un-seeded case is not on current rails.
- [ ] MP-F4 composition holds — helper reads `state.members` after `derive_resolved`; no new `state_key`; frontier-anchor + A1 untouched.
- [ ] `apply_federation_push`/`derive_event_nodes` handle a self-entry gracefully (no panic, no self-delivery) — confirmed (§7.5).
- [ ] Build 0 + clippy clean (default **and** `--features harness-control`); full suites green (D-077).
- [ ] **Routed, not built:** F1B-D5 discovery gap — no discovery code in this arc.
- [ ] No DECISIONS change in-arc (invariant E a candidate; promote at close per D-069 in the doc-bridge).

---

## 6. Scope fence + boundaries (D-065)

- **In scope:** the NodeRuntime population helper (F1B-D1/D2/D3) + the four apply-site hooks + the
  four units + the MP-C-07 witness flip with boundary (F1B-D4).
- **Out of scope, routed:** production identity→home-node **discovery** (F1B-D5) — its own arc; F1b
  does **not** build it. The federated-RTBF historical-copy tension (F1B-D7-C) — acknowledged, not
  solved.
- **Honest boundary:** MP-C-07 converges **in the harness** because G-6 pre-seeds resolution; a
  production DM to a not-yet-known counterparty does **not** converge until the discovery arc lands.
  Recorded, witnessed-with-boundary, **not** dressed as a clean cross-node ✅.

---

## 7. Grounding surprises (surface for Joe-lock — none change design intent)

The grounding turned up five precision items the design's prose collapsed. None alter the locked
F1B-D1..D7; recording them so the implementer doesn't re-discover and Joe sees the real shape.

**§7.1 — the "incremental arm" is two sub-sites, not one.** The design's "incremental `_ => apply_event`
arm" is in code the conflict-rebuild branch (675–680) **and** the fast-apply branch (681–683). Both
need the hook → **four** apply sites total (not three). The common DM join is the fast-apply branch.

**§7.2 — `ingest_event` is the single apply chokepoint.** `dispatch_event` does its own validation/gating
then **delegates** to `self.ingest_event(event)` (runtime.rs:1543); `replay_spaces_from_dir` calls
`ingest_event` directly. So local-submit, federation-receive, and replay all funnel through
`ingest_event`'s three sites; only `rehydrate_space_from_store` is separate. (I initially read
`dispatch_event` as having its own apply path — it does not. The design's apply-site list is correct;
this confirms it.)

**§7.3 — the catch-up streams on `shared_spaces`, not `federation_nodes`.** `stream_federation_delta`
(federation_session.rs:57/84) iterates the relationship's `shared_spaces`. So in MP-C-07, alice's
`a2`/`a3` reach B via the director's `initiate` catch-up (shared_spaces, **helper-independent**); bob's
join + `b4` reach A via the helper-populated B→A push. **This is why RED-on-revert is genuine without
being total:** reverting the helper still lets a3 ride the catch-up, but kills the B→A push of bob's
join + `b4` → A is missing `b4` + alice's view stays `{alice}` → RED. The helper is load-bearing for
exactly the half the catch-up cannot carry (the live B→A direction).

**§7.4 — the re-fire at the JOIN site is load-bearing, not an optimization.** Identity replication can
lag the DM create (the A↔B session establishes at `initiate`, after the create). So the create-arm
helper may omit a member whose record hasn't replicated yet. The **idempotent re-fire at bob's
membership-apply** (site 3, gated by `alice_sent` + the settle window) is when both records are reliably
in the registry → the set lands complete. Re-firing at all sites is what makes the witness robust to
replication timing — not redundancy.

**§7.5 — self-inclusion (F1B-D2) is deliberate + harmless.** The helper includes the local node's home
in `federation_nodes`, so A and B both derive the **identical** `{A, B}` (cross-node `SpaceState`
symmetric). `apply_federation_push` iterates `federation_nodes`, finds no `FederationPeerSenders` entry
for self → `try_send` no-op + log → no self-delivery, no panic. **Confirm-at-impl** (DoD): re-read
`apply_federation_push`/`derive_event_nodes` to verify the self-entry is a graceful no-op (the design
flags this; it appears so from the code but verify).

**§7.6 — sort is required, not cosmetic.** `state.members` is a `HashMap` (non-deterministic iteration,
per-instance random seed). Without `nodes.sort()`, two derivations of the same DM on the same node could
produce different `Vec<NodeXgid>` orders → within-node `assert_converges` (Vec equality) could fail.
`NodeXgid: Ord` (flavours.rs:129) → `.sort()` closes it. (Cross-node the mptest oracle already tolerates
federation-infra asymmetry per MP-R1-D7, but the sorted full set is symmetric anyway.)

---

## 8. Entry point (Rule 0)

CLAUDE PLAY (J-332) → JOURNAL J-332 → `tasks/MP_F1B_DM_FEDERATION_AUDIT.md` (§2 (iii)/A–E, §4 gate-B
verdict) → `tasks/MP_F1B_DM_FEDERATION_DESIGN.md` (§9 Design-Z amendment) → `tasks/MP_R1_DETERMINISTIC_DESIGN.md`
§11 (D10 amended) → this runbook (§9 as-built; §3 apply sites stand). The helper + the apply-site hooks +
the identity-replicate hook + the units + the honest MP-C-07 flip are the code deliverables; the discovery
arc (F1B-D5) is routed, not built.

---

## 9. As-built — Design Z (SHIPPED)

The live witness falsified the §3.2 "no new send code" premise; **Design Z** (Joe-locked) is what shipped.
Full rationale: design `MP_F1B_DM_FEDERATION_DESIGN.md` §9. Deltas from §1–§7:

**C1 (xgen-core + xgen-node):**
- **Helper populates from parties = `members ∪ pending_invites`** (not members only) — invariant E amended
  to "a DM's federation set = its parties' home nodes." This puts the counterparty's home in
  `federation_nodes` **from create** (it's the seeded pending invitee), so the receiving F-3 gate passes
  the bootstrap join **with no skip** and the creator's pre-join message pushes immediately.
- **No F-3 skip.** F-3 stays fully intact (the experimental skip was removed). The spine (F1B-D8) came back
  **falsified** — `apply_join` open-joins (no DM 2-party join gate), so an unconditional skip would be a
  hole; Z avoids the skip entirely. F-3 remains the guard that blocks a 3rd-party federation join.
- **New NodeRuntime hook `repopulate_dm_federation_after_identity`** fired from the identity-replicate
  handler (app.rs, beside `drain_pending_by_identity`): re-populates affected DMs' `federation_nodes` when
  a lagging record lands **and** drains any F-3-held DM join for that peer — reusing
  `drain_pending_by_federation_relationship` **verbatim** (D-076 by inheritance).
- The 4 apply-site hooks (§3) ship as planned (create / conflict-rebuild / fast-apply / rehydrate).

**Units (xgen-core, +6 — was 4):** `..._parties_resolvable` (pending-inclusion: set = `{A,B}` from create) /
`..._omits_unresolvable_party` / `..._regular_space_untouched` / `..._shrinks_on_leave` /
**`..._join_via_federation_passes_f3_no_skip`** (Z bootstrap, `dispatch_event`) /
**`..._third_party_dm_join_via_federation_blocked_by_f3`** (the F1B-D8 hole-closed proof — F-3 blocks a
3rd-party federation join, proving Z needs no skip).

**C2 witness (xgen-mptest):** `mp_r1_c4::mp_c_07` flipped to harness-green-with-boundary; **GREEN A↔B,
stable ×3** (both `a3` + `b4`); **RED-on-revert demonstrated** (neuter Z population → `federation_nodes`
empty → bob's join F-3-held → membership diverges → RED). `mp_r1_c5:187` cross-ref corrected.

**Verification:** xgen-core 689/0, xgen-node 286/0, clippy clean (default + all-features); MP-C-07 GREEN ×3.

**Grounding discharges (both clean):** D-076 by inheritance (same drain, one more trigger); no empty-set
instant across the pending→member transition (`apply_join`'s remove+insert are one apply; the helper reads
only at quiescent points).

---

*Per D-065 + D-067 + D-069 + D-071 + D-075 + D-076 + D-077.*
