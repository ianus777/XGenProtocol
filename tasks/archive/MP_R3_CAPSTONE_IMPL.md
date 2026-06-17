# MP-R3 (capstone) — implementation runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

The Clair-facing build runbook for **MP-R3**, the capstone of the Multiparty-tests milestone. It
executes the Joe-LOCKED design (`tasks/MP_R3_CAPSTONE_DESIGN.md` v1.1, R3-D1..D7 all LOCKED), mirroring
the **R2 build-then-RUN** shape: a box-free build phase (each commit lib/unit/in-process-proven + a
box-gated `#[ignore]` smoke authored) → the box-gated RUN (re-bench FIRST) → the loop-to-green
BOUNDED-gate fix-phase → close + the consolidated R1+R2+R3 ledger.

State at open: MP-R1 ✅ CLOSED (J-340), MP-R2 ✅ CLOSED (J-348). Phase-0 + design Joe-LOCKED (J-349).
The Multiparty milestone stays 🟢 PLAY — R3 is the last sub-pass.

**No code until this runbook is Joe-locked.** Per-commit DoD is grounded against live code (D-078 —
the file:line anchors are confirm-at-pickup; Clair re-greps exact lines before touching). Each named
test is named below. Clair commits first; **Joe pushes**; Chat writes the next bridge at the
build/RUN boundary. Pin-by-observation BEFORE routing (the MP-R2 bar).

**Locked inputs (do not re-open):** R3-D1 ceiling = re-benched **process** wall (no multiplexer);
R3-D2 partition = relationship-level (MP-A-08 harness-green-with-boundary); R3-D3 multi-target
injector (oracle reused); R3-D4 chaos two-seam + liveness probe + elastic settle + churn-at-scale
per-node oracle; R3-D5 CEILING aggregate-RSS + OOM-exit + client-RSS re-bench; R3-D6 MP-F11 fixed
in-round / MP-F13 routed; R3-D7 row set + close = all-green-except-MP-C-16.

---

## 1. Build-then-RUN shape + commit overview

The build phase is **box-free** (no spawned binaries in the fast suite; the smokes are `#[ignore]`,
run only at the box-gated RUN). Six build commits. **R3-D6 (MP-F11) is the only production-crate
change** (`xgen-core`) — full protocol-change discipline (spine-first, RED-on-revert); C1–C4 are
`xgen-mptest` test-crate only.

| Commit | Scope (R3-D#) | Crate | Box-gated smoke authored |
|---|---|---|---|
| **C1** | multi-target raw-wire injector (R3-D3) | `xgen-mptest` | `mp_r3_equivocation` (MP-A-06) |
| **C2a** | chaos composition seam (R3-D4a: `DirectorStep::Chaos` + parallel chaos task + `[[chaos]]` + elastic settle R3-D4c) | `xgen-mptest` | `mp_r3_partition` (MP-A-08) · `mp_r3_chaos` (overlay) |
| **C2b** | oracle hardening (R3-D4b liveness probe + R3-D4d churn-at-scale per-node oracle) | `xgen-mptest` | (feeds the churn climb; unit-proven) |
| **C3** | topology star→star+mesh (§6.3) + flood no-drain enrichment (§6.3) | `xgen-mptest` | `mp_r3_topology` (MP-C-14) · `mp_r3_flood` (MP-A-07 curve) |
| **C4** | CEILING hardening (R3-D5: aggregate-RSS + OOM-exit + client-RSS re-bench) | `xgen-mptest` | (feeds the RUN re-bench; unit-proven) |
| **C5** | MP-F11 protocol fix (R3-D6) — spine-first | `xgen-core` (+ witness) | `mp_r2_catchup::mp_a_01_ii_*` flips red→green |

**Ordering note (D-078).** C1–C4 are independent test-crate work and may be authored in any order;
this sequence puts the adversary capability (C1) + the chaos seam it composes into (C2) first, then
the generators (C3), then the classifier (C4), then the one production fix (C5). The box-gated smokes
are authored (not run) during build, so a smoke that depends on C5 (MP-A-01(ii), MP-A-08's heal) is
*authored* before C5 lands and *passes* only at the RUN once C5 is in. **The MP-F11 fix (C5) is
load-bearing for MP-A-08's relationship-level heal (R3-D2) and for MP-A-01(ii)** — both green only
after C5.

**Standing per-commit DoD (every commit):** `cargo build --workspace --all-targets` 0-error; `cargo
clippy --workspace --lib --tests --all-features -- -D warnings` clean (default **and**
`--features harness-control` where the commit touches a fenced path); the touched crate's fast lib
suite green; the named unit/in-process tests added + green; the box-gated `#[ignore]` smoke compiles
+ is authored (run deferred to the RUN); **no "commit pushed" line** (Clair commits, Joe pushes); the
prime invariant held (the additive change leaves the existing R1/R2 smokes byte-unaffected).

---

## 2. Build-phase commits

### C1 — multi-target raw-wire injector (R3-D3) [`xgen-mptest`]

**Grounded surfaces (confirm-at-pickup).** `ActorKind::Injector` + `ActorSpec { name, node, batch,
ai_mode, kind }` with `#[serde(deny_unknown_fields)]` ([`manifest.rs:107-136`](../xgen-mptest/src/manifest.rs#L107));
`InjectorHandle { name, url, lines }` ([`runner.rs:135-141`](../xgen-mptest/src/runner.rs#L135)) +
its build at [`runner.rs:244-252`](../xgen-mptest/src/runner.rs#L244); `run_injector_actor(actor,
node_url, lines, registry, timeout)` building one `WireActor` ([`injector_actor.rs:101-111`](../xgen-mptest/src/injector_actor.rs#L101),
`ensure_member` :280-292); the crafted-event builders ([`injector.rs`](../xgen-mptest/src/injector.rs),
imported [`injector_actor.rs:51-58`](../xgen-mptest/src/injector_actor.rs#L51)).

**The change.**
1. `ActorSpec` gains `#[serde(default)] pub nodes: Vec<String>` (additive; `deny_unknown_fields`-safe).
   Resolution rule: a multi-target injector lists `nodes = ["a","b",…]`; a single-target injector
   keeps `node = "a"` (the runner reads `nodes` if non-empty, else `[node]` — backward-compatible, the
   R1/R2 injector smokes unchanged).
2. `InjectorHandle` carries `targets: Vec<(String /*label*/, String /*url*/)>` instead of one `url`.
3. `run_injector_actor` holds `member: Vec<(String, WireActor)>` (one per target, **all from one
   `fresh_key`** — the same hostile identity); a directive's optional `target` field names which node
   label it presents to (default `targets[0]`). Member-context setup (`register` / `create_space` /
   open-`join`) runs against the first target; the identity replicates + the open-join federates so
   the actor is a member on every target.
4. New directive **`equivocate`** — builds two conflicting events at one frontier and submits one to
   each named target (`to_a` / `to_b`), capturing both crafted `event_id`s.

**Equivocation payload — pinned by observation (R3-D3, the named mechanic).** The candidate (design
§2) is a **self-membership fork**: a `membership.leave` (→ target A) and a `membership.join` (→ target
B) of the hostile identity, both anchored to the **same** frontier tip → collide on
`membership:{space}:{self}` → resolution deterministically elects **leave** (Layer-1 leave>join, proven
MP-F7/J-348). **Pin discipline at C1:** author the directive, run the box-gated `mp_r3_equivocation`
smoke once (Joe-authorized, keep-artifacts), and **observe on both nodes**: (i) both fork events land
(both apply — equivocation is not a rejection), (ii) `convergence_verdict` passes (both nodes resolve
the same winner), (iii) no permanent fork. **If the observation falsifies the candidate** (e.g. the
forks don't actually share a state_key, or the loser doesn't federate to the other node) → pin the
real conflicting pair before recording (other candidates: conflicting `thread.status`; or
owner/admin-seated conflicting `state.room_update`). Record the observed winner in the smoke's
doc-comment. **Do not pre-lock the payload in code** until the observation confirms it.

**Oracle (reused, NOT net-new).** `convergence_verdict` ([`oracle.rs:229-272`](../xgen-mptest/src/oracle.rs#L229))
asserts membership-set + cooperative transcript event-id-set equality across nodes = convergence-on-winner.
The smoke MAY add a transcript assertion that **both** fork event_ids are present on both nodes and
exactly one is resolved-out of state (sharpens "no permanent fork").

**Unit/in-process proof (named).** In `injector_actor.rs::tests` (or a sibling): `equivocate_targets_two_nodes_with_one_key`
(the directive builds two events sharing a state_key + frontier, presents one per target — pure
construction assertion, no sockets); `injector_nodes_list_defaults_to_single_node` (backward-compat:
`node`-only spec yields one target); `injector_shares_one_signing_key_across_targets` (the multi-target
member uses one `fresh_key`).

**Box-gated smoke (authored, `#[ignore]`).** NEW `tests/mp_r3_equivocation.rs` —
`mp_a_06_equivocation_converges_on_winner`: 2 federated nodes A↔B; hostile injector registers +
open-joins the shared Space (owned by a cooperative batch actor); `equivocate` presents fork-X→A,
fork-Y→B; settle; assert `convergence_verdict` passes (convergence-on-winner) + both forks present +
one resolved-out + no permanent fork.

**DoD.** Standing DoD + the three units green + `mp_r3_equivocation` authored + the R1/R2 injector
smokes (`mp_r1_c7`) byte-unaffected (the `nodes`-default path).

---

### C2a — chaos composition seam (R3-D4a + R3-D4c) [`xgen-mptest`]

**Grounded surfaces.** The director's step kinds `DirectorStep::{Link, Clock, Migration}`
([`runner.rs:416-421`](../xgen-mptest/src/runner.rs#L416)) + `order_director_steps` (F10-D1
dependency ordering, [`runner.rs:439-501`](../xgen-mptest/src/runner.rs#L439)) + `run_director`
([`runner.rs:515-604`](../xgen-mptest/src/runner.rs#L515)); the concurrent phase
`tokio::join!(drive, inj_drive, direct)` ([`runner.rs:353`](../xgen-mptest/src/runner.rs#L353)); the
`Manifest` collections (`federation`/`clock`/`migration`, partitioned into plans at
[`runner.rs:279-327`](../xgen-mptest/src/runner.rs#L279)); the raw-WS load drivers `run_storm` /
`slow_loris` / `event_flood` ([`churn.rs:84-138`](../xgen-mptest/src/churn.rs#L84)); the `settle`
`MAX = 15s` ([`runner.rs:610-611`](../xgen-mptest/src/runner.rs#L610)) + its poll-until-stable loop
([`runner.rs:614-630`](../xgen-mptest/src/runner.rs#L614)); `federation defederate` aicontrol verb
(unfenced, [`aicontrol.rs:366`](../xgen-node/src/aicontrol.rs#L366)).

**The change (the hybrid two-seam, R3-D4a).**
1. Manifest gains `#[serde(default)] pub chaos: Vec<ChaosSpec>` — `ChaosSpec { kind: ChaosKind, nodes:
   Vec<String>, after: Option<String>, publishes: Option<String>, params: ChaosParams }`. `ChaosKind`
   = `Partition` / `Heal` (node-conn → director) | `Flood` / `Storm` / `SlowLoris` (raw-WS → parallel
   task). The runner **partitions** ChaosSpecs the same way it partitions federation/clock/migration
   plans (node-conn vs raw-WS).
2. **Director chaos-steps** — add `DirectorStep::Chaos(idx)` to the `order_director_steps` worklist
   (reusing the F10-D1 publish→wait ordering verbatim — a partition/heal step gates on `after` and
   MAY `publishes` a heal-done key) + a `run_director` arm: `Partition` = `federation defederate` on
   the named nodes (relationship-level sever, R3-D2); `Heal` = `add-peer`(naming the Space) +
   `initiate` (re-establish — the catch-up that **rides MP-F11**). Uses the single-owner `&mut nodes`
   (no borrow refactor — same as Link/Clock/Migration).
3. **Parallel chaos task** — a new `run_chaos(targets, specs, registry, timeout)` task joined into the
   concurrent phase: `tokio::join!(drive, inj_drive, direct, chaos)`. It owns the chaos **timeline**
   (sleep → publish the `after`/`publishes` keys the director chaos-steps gate on) + drives the raw-WS
   load (`Flood`/`Storm`/`SlowLoris` via `connect_url`/`event_flood` — **no `&mut nodes` borrow**, so
   it composes with the director cleanly). The director executes node-conn actions gated on the keys
   the chaos task publishes.
4. **Elastic settle (R3-D4c)** — `settle` takes a `settle_max_secs` (a new `RoundDial`/`Scenario`
   field, `#[serde(default = "15")]`); capstone/chaos rows set it higher. The poll-until-stable
   termination ([`runner.rs:621-628`](../xgen-mptest/src/runner.rs#L621)) is unchanged (stable-for-2
   → return early), so a quiesced run still returns promptly; only the *ceiling* extends.

**Restart-during-chaos ownership (R3-D4a, the named mechanic) — RESOLVED: not needed for R3.** The
R3 chaos overlay = **partition (relationship-level, director) + flood/storm/slow-loris (parallel task,
raw WS) + equivocation (C1 injector)**. **No R3 row requires a node *restart* during chaos** (MP-A-08
is relationship-level partition, not node-kill; MP-A-18 is connection storm; MP-C-15 restart+replay is
an R2 row, already green, its own bespoke smoke). So the flagged `ManagedProcess`-ownership wrinkle
(processes owned by `run_scenario`, [`runner.rs:190-222`](../xgen-mptest/src/runner.rs#L190), not the
director) **does not arise** — neither the director chaos-steps (node-conn) nor the parallel task
(raw-WS) touch `ManagedProcess::restart`. Recorded honestly (D-065): if a future chaos variant needs
restart-during-chaos, it rides the **routed transport-level seam** (R3-D2), where node-kill lives —
not R3.

**Unit/in-process proof (named).** `director_orders_chaos_step_after_its_publishing_clock` (the
F10-D1 ordering extends to `DirectorStep::Chaos` — a heal gated on a published key runs after its
publisher; sibling to `director_orders_fed_link_after_its_clock_gate`,
[`runner.rs:770-789`](../xgen-mptest/src/runner.rs#L770)); `chaos_specs_partition_into_director_and_raw_ws`
(Partition/Heal → director-step set, Flood/Storm/SlowLoris → parallel-task set); `settle_max_secs_defaults_to_15`
+ `elastic_settle_returns_early_when_stable` (the ceiling extends, the stable-2 termination is intact).

**Box-gated smokes (authored, `#[ignore]`).**
- NEW `tests/mp_r3_partition.rs` — `mp_a_08_relationship_partition_converges_after_heal`: 2 federated
  nodes; cooperative chat; a `Partition` chaos-step severs A↔B (defederate) while each side admits
  distinct events; a `Heal` step (gated on the chaos-task timeline) refederates; settle (elastic);
  assert `convergence_verdict` passes (convergence-after-heal) + **no lost admitted events** (the
  pre-partition admitted set is present on both nodes after heal). **Boundary recorded in the
  doc-comment (R3-D2):** this witnesses convergence-after-heal + no-lost-events; the
  **no-reconnect-deadlock** half is the routed transport-level seam — harness-green-with-boundary. (Green only after C5 MP-F11.)
- NEW `tests/mp_r3_chaos.rs` — `chaos_overlay_liveness_and_convergence`: a small N×M cooperative chat
  running **under** a stacked chaos task (partition + flood) + the C2b liveness probe; assert the node
  stays live throughout + converges after the chaos window. (The capstone overlay witness.)

**DoD.** Standing DoD + the four units green + both smokes authored + the existing director-ordering
units (`director_order_tests`) byte-unaffected (no-edge → original phase order preserved).

---

### C2b — oracle hardening (R3-D4b + R3-D4d) [`xgen-mptest`]

**Grounded surfaces.** `convergence_verdict`'s ≥2-projection precondition
([`oracle.rs:234`](../xgen-mptest/src/oracle.rs#L234)); the per-actor projection gather
([`runner.rs:369-386`](../xgen-mptest/src/runner.rs#L369), one `members` query per batch actor); the
`AicontrolClient` round-trip (the liveness probe vehicle).

**The change.**
1. **During-chaos liveness probe (R3-D4b)** — a `run_liveness_probe(nodes, interval, registry)` task
   (joined into the concurrent phase) that at intervals sends a benign `state` round-trip to each node
   (and, for the honest-traffic property, optionally has a cooperative member post + verifies it lands
   post-heal). Records a `LivenessReport { node, samples, unresponsive_at }`; an unresponsive node =
   a finding surfaced at the RUN (faced-and-routed). Small, additive.
2. **Churn-at-scale per-node oracle (R3-D4d)** — the MP-F7 ≥2-projection edge: at capstone churn a
   churning actor legitimately returns no `members` view at sample time (the J-348-flagged fragility).
   Fix: gather the membership projection via a **stable reader per node** (one non-churning client per
   node) so the convergence key is **node-to-node**, decoupled from which churning client is mid-leave.
   Add `convergence_verdict_per_node(...)` (or a `gather_node_projections` that queries a designated
   stable reader per node) — the transcript half is already per-node ([`oracle.rs:251-265`](../xgen-mptest/src/oracle.rs#L251));
   this aligns the membership half. The existing per-actor `convergence_verdict` stays for non-churn rows.

**Unit/in-process proof (named).** `per_node_projection_ignores_mid_leave_actor` (a node-level
projection converges even when a churning client returns no view — the MP-F7 edge defeated);
`liveness_report_flags_unresponsive_node` (a probe sample with no reply marks `unresponsive_at`);
`per_actor_oracle_unchanged_for_non_churn` (the existing `convergence_verdict` path byte-identical).

**DoD.** Standing DoD + the three units green + the existing `oracle.rs::tests` byte-unaffected.

---

### C3 — topology star→star+mesh + flood no-drain enrichment (§6.3) [`xgen-mptest`]

**Grounded surfaces.** `FederationPattern { None, StarFromFirst }`
([`sweep.rs:286-292`](../xgen-mptest/src/sweep.rs#L286)) + the generator's federation-link emission
([`sweep.rs:371-377`](../xgen-mptest/src/sweep.rs#L371)); `event_flood(url, count, pace)` paced
ack-drained, with the no-drain-fire-hose flagged R3 ([`churn.rs:116-138`](../xgen-mptest/src/churn.rs#L116));
`WireActor::submit` (the per-submit ack-drain, [`churn.rs:130-132`](../xgen-mptest/src/churn.rs#L130)).

**The change.**
1. **Topology (MP-C-14):** `FederationPattern::StarPlusMesh` — the generator emits the star links
   (`n1..` → `n0`) **plus** cross-links among the leaves (the mesh). Confirm the F-5/D-089 pairwise
   model holds under mesh (no transitive relay — MP-A-13 is the anti-transitivity guard, already
   green).
2. **Flood no-drain enrichment (MP-A-07 intensity *curve*):** add a `event_flood_firehose(url, count)`
   variant that submits **without draining each ack** (the true fire-hose the `churn.rs:116` note
   defers to R3) + a **rate-curve driver** that sweeps the inter-send pace (or fire-hose burst size)
   across rungs → the intensity **curve**, not the R2 single liveness point. The honest-traffic
   liveness is asserted by the C2b probe.

**Unit/in-process proof (named).** `star_plus_mesh_emits_leaf_cross_links` (the generated manifest has
the star + mesh `[[federation]]` links for a given node count); `firehose_submits_without_draining_acks`
(pure: the no-drain variant's send loop omits the ack-drain); `flood_rate_curve_sweeps_pace` (the
curve driver yields the rung sequence).

**Box-gated smokes (authored, `#[ignore]`).**
- NEW `tests/mp_r3_topology.rs` — `mp_c_14_star_plus_mesh_converges`: 4–5 nodes star then star+mesh;
  a Space spanning all; assert `convergence_verdict` + `.events` across all nodes (pairwise model holds
  under both topologies).
- NEW `tests/mp_r3_flood.rs` — `mp_a_07_intensity_curve_liveness`: the fire-hose rate-curve against a
  node + the C2b liveness probe; assert the node stays live + honest traffic applies across the curve
  (the intensity *curve* recorded, break-point per-rate if any).

**DoD.** Standing DoD + the three units green + both smokes authored + the existing sweep generator
tests (`template_generate_emits_dial_sized_manifest`, [`sweep.rs:672-691`](../xgen-mptest/src/sweep.rs#L672))
byte-unaffected.

---

### C4 — CEILING-classifier capstone hardening (R3-D5) [`xgen-mptest`]

**Grounded surfaces.** `peak_resource(nodes, actors)` (samples all pids, takes the **max**,
[`runner.rs:642-649`](../xgen-mptest/src/runner.rs#L642)); `is_resource_exhausted(sample, floors)`
(per-process RSS-wall + thread-thrash, [`sweep.rs:202-204`](../xgen-mptest/src/sweep.rs#L202));
`classify_rung` ([`sweep.rs:209-232`](../xgen-mptest/src/sweep.rs#L209)); `ManagedProcess::try_exit_status`
(exists, **unused by the classifier**, [`process.rs:283`](../xgen-mptest/src/process.rs#L283));
`run_microbench` (spawns **nodes only**, [`bench.rs:165-176`](../xgen-mptest/src/bench.rs#L165));
`CeilingFloors::from_bench` (derives the RSS wall from node mean RSS, [`sweep.rs:101-112`](../xgen-mptest/src/sweep.rs#L101));
`BoxSpec::budget_bytes` ([`bench.rs:52`](../xgen-mptest/src/bench.rs#L52)).

**The change.**
1. **Aggregate-RSS-vs-box-RAM (R3-D5a):** extend `peak_resource` to also return the **sum** of all
   sampled pids' RSS (an `AggregateResource { peak, total }`); the rung classifier gains an
   **aggregate-RSS ≥ `BoxSpec::budget_bytes`** → Ceiling check (the true capstone wall — sum of all
   node+client footprints vs the box budget).
2. **OOM-death-by-exit-code (R3-D5b):** the runner records whether any spawned process exited
   unexpectedly mid-run (`try_exit_status` → `Some(non-zero / OOM)`); the rung outcome carries a
   `process_died: bool`; `classify_rung` treats it as **Ceiling** evidence (the clearest hardware-wall
   discriminator — the D-065 logic-vs-hardware split).
3. **Client-RSS re-bench denominator (R3-D5c):** extend the re-bench (or a sibling
   `run_microbench_with_clients`) to spawn a representative **client** mix so the ceiling is derived
   from the **combined (node + client)** footprint; `from_bench` calibrates the wall against the
   total-process mean, not node-only. The re-bench is the **first RUN step** (§3).

**Unit/in-process proof (named).** `aggregate_rss_over_budget_is_ceiling` (sum ≥ budget → Ceiling, even
when no single process trips the per-process wall); `process_died_is_ceiling_evidence` (a non-zero exit
→ Ceiling regardless of RSS); `from_bench_uses_combined_node_client_mean` (the denominator includes
client RSS); the existing classifier units (`logic_fault_when_fail_with_healthy_resources`,
`ceiling_suspect_when_fail_with_no_resource_sample`, [`sweep.rs:504-525`](../xgen-mptest/src/sweep.rs#L504))
byte-unaffected (the new checks are additive Ceiling paths, the None→ceiling-suspect inversion intact).

**DoD.** Standing DoD + the three units green + the existing `sweep.rs::tests` + `bench.rs::tests`
byte-unaffected.

---

### C5 — MP-F11 protocol fix (R3-D6) [`xgen-core` + the box-gated witness] — spine-first

**The one production-crate change.** Full protocol-change discipline: **spine-first** (prove the
D-076/F-3 spine RED-on-revert in `xgen-core` *before* the witness), convergence-safety, no new ordering
surface.

**Grounded mechanism (audit §3 Cluster A / design §5.1).** `repopulate_dm_federation_nodes` populates
`federation_nodes` from members ∪ pending invitees → home nodes but is **`dm_constraints_active`-gated**
(early-returns for regular Spaces, [`runtime.rs:2004-2007`](../xgen-core/src/node/runtime.rs#L2004));
the F-3 gate holds a late peer's content when that peer is absent from `federation_nodes`
([`runtime.rs:1007-1040`](../xgen-core/src/node/runtime.rs#L1007), skip set :1027-1032, `f3_reject`
:1071-1078); the drain `drain_pending_by_federation_relationship` releases held events when the
relationship lands ([`runtime.rs:1805`](../xgen-core/src/node/runtime.rs#L1805)); regular Spaces
populate via the vantage-aware `apply_federation_add` ([`state.rs:655-700`](../xgen-core/src/space/state.rs#L655));
the repopulate call-sites ([`runtime.rs:519`, `665`, `687`, `697`](../xgen-core/src/node/runtime.rs#L519)).

**The fix (locked direction, R3-D6; exact code = Clair).** On the **federation-establish path for a
shared regular Space**, populate the late peer's `federation_nodes[S]` with the **established
federation relationship's** node id as a **legitimate relationship record** (NOT an F-3 skip — the
J-333 hole lesson: a non-party's node must never enter the set, or F-3 stops blocking third parties),
then fire `drain_pending_by_federation_relationship` to release the F-3-held content. Lift the
`dm_constraints_active` gate to a **sibling regular-Space path**: DM keeps its members∪invitees rule;
regular Spaces populate from the **established federation relationship** (the real authority — the peer
the node has actually federated with for S), not from membership.

**Spine-first (proven, not assumed — D-076/F-3; named tests in `xgen-core`).**
- `mp_f11_regular_space_populate_on_establish_drains` — a regular Space late-federating onto a peer:
  the populate-on-establish puts the established peer in `federation_nodes[S]` → the F-3-held content
  drains (`drain_pending_by_federation_relationship` fires) → the events apply. RED-on-revert: revert
  the regular-Space populate → content stays F-3-held → apply count 0.
- `mp_f11_third_party_regular_space_content_blocked_by_f3` — the **hole-closed** assertion (mirroring
  MP-F1b's `mp_f1b_third_party_dm_join_via_federation_blocked_by_f3`): a non-federated third party's
  content for S stays F-3-held (the populate is a *legitimate relationship record*, not a blanket
  skip — F-3 still blocks third parties). RED-on-revert: an over-broad populate (adding a non-party)
  → the third-party content wrongly applies → this test red.

**Box-gated witness (the real-binary flip).** `mp_r2_catchup::mp_a_01_ii_aged_invite_replay_preserves_membership`
([`mp_r2_catchup.rs:202`](../xgen-mptest/tests/mp_r2_catchup.rs#L202)) flips **KNOWN-FAIL → GREEN**
(the 3-node aged-invite: late node C federates after A's clock ages the Space; C's `federation_nodes`
now gets A on establish → A's content drains → membership preserved). RED-on-revert genuine (revert the
regular-Space populate → C's set stays empty → content F-3-held → row red).

**DoD.** `cargo build --workspace --all-targets` 0-error; clippy `--all-features` clean; `xgen-core`
fast suite green (+ the 2 spine tests); `xgen-node` fast suite green; **the existing MP-F1b DM-federation
spine + the J-298 INV-EXP regression byte-unaffected** (the DM path keeps its members∪invitees rule;
the regular-Space path is the additive sibling); D-076 discharged (no new ordering surface — the
populate is a derived projection from the established relationship; the F-3 drain reuses the verbatim
existing hook). The box-gated MP-A-01(ii) witness is authored to assert GREEN at the RUN.

---

## 3. The box-gated RUN order (re-bench FIRST)

Run only after the build phase is Joe-locked + complete (all six commits in, fast suites green). The
node build for federated/chaos/clock smokes is `--features harness-control` (the seam verbs are
fenced); rebuild it **after** any `cargo test --workspace` (the J-340 binary-clobber hazard — workspace
build clobbers the harness-control binary with default features).

1. **Re-bench FIRST (R3-D5c).** `XGEN_MPTEST_BENCH_TIERS=10,50,100`, **client RSS included** (the
   combined node+client footprint) → the live total-process ceiling (replaces 1562 **and** 1288) →
   `CeilingFloors::from_bench`. **No scale number inherited** (R2's D4 discipline). Record the ceiling
   + the calibrated floors.
2. **Climb rows → the re-benched process wall (R3-D1):** MP-C-05 (sustained chat, higher sweep `max`),
   MP-C-11 (churn, the C2b per-node oracle), MP-C-14 (`mp_r3_topology` star+mesh), MP-A-07
   (`mp_r3_flood` intensity curve), MP-A-18 (connection storm at capstone churn). Each climbs to the
   wall; the break-point-per-axis is the deliverable (the curve, not a bool) — CEILING (aggregate-RSS
   / OOM-exit, R3-D5) vs LogicFault distinguished.
3. **New-capability rows:** MP-A-08 (`mp_r3_partition`, relationship-level — harness-green-with-boundary),
   MP-A-06 (`mp_r3_equivocation`, convergence-on-winner).
4. **Chaos overlay:** `mp_r3_chaos` (the climb running under stacked partition + flood + the liveness
   probe).
5. **Dep-witness rows:** MP-A-01(ii) (`mp_r2_catchup::mp_a_01_ii_*` → **GREEN** via C5 MP-F11),
   MP-C-16 (`mp_r2_fixed::mp_c_16_*` → **red-with-reason**, MP-F13 routed — recorded, not chased).

---

## 4. Fix-phase (loop-to-green BOUNDED gate)

The RUN surfaces findings → the gate is **frozen at whatever surfaces** → each reaches a terminal
state: **(a) GREEN-on-rerun** (fixed + the smoke passes) or **(b) Joe-routed-with-reason**. The
**carried deps are decided pre-RUN (R3-D6)** — MP-F11 fixed in-round, MP-F13 routed — so they are
**NOT gate items**. Newly-occurring bugs (incl. any the rerun surfaces) are **faced-and-routed** to
their natural homes (own arc / a horizon / M10) but do **NOT** re-open or extend the gate (the R2
discipline, J-344). Pin-by-observation BEFORE routing (Joe-authorized bounded diagnostics, keep-
artifacts, reverted after). When the gate is terminal, **rerun** the affected smokes to
green-to-criterion → the true close.

**Close criterion (R3-D7):** **all-green-except-MP-C-16.** MP-A-01(ii) green (MP-F11); MP-A-08
harness-green-with-boundary (no-reconnect-deadlock half routed); MP-C-16 red-with-reason (MP-F13). The
R1 MP-C-06 / R2 {MP-C-16, MP-A-01(ii)} "all-green-except" shape, narrowed to a single carve-out.

---

## 5. Close + the consolidated R1+R2+R3 ledger (the milestone deliverable)

At the R3 true close — i.e. when MP-R1, MP-R2, AND MP-R3 are all green/closed — the whole
**Multiparty-tests milestone** closes. Deliverables (Chat's seat for the doc-bridge):

1. **The consolidated R1+R2+R3 ledger (HANDOFF §3, the standing deliverable).** Every scenario row
   (`MP-C-##` + `MP-A-##`) across all three rounds with its FINAL status + the complete findings table
   (`MP-F#`) — the J-348 R1+R2 ledger format (cooperative table, adversarial table, findings table,
   net summary), extended with R3.
2. **The §3.1 breadcrumb sweep (HANDOFF §3.1):** **MP-F2-followon → M10** (the 7 unmapped
   event-validation wire codes still generic-4000 — routed to M10's auth-module/reject-honesty era,
   per the MP-A-05/17/20 boundary); **D-091 tidy verified** (invariant E — a DM's federation set is
   exactly its parties' home nodes — confirm the R3 regular-Space populate (C5) did not perturb the DM
   invariant; the DM path keeps members∪invitees, the regular path is the additive sibling).
3. **Canonical-record flips (Chat):** `MULTIPARTY_TEST_MATRIX.md` §6 (R3 RUN-record + per-row final
   flips); `MP_findings.md` (MP-F11 → RESOLVED/in-round; MP-F13 → routed-with-reason; any RUN-surfaced
   findings); ROADMAP (Multiparty milestone 🟢→✅; MP-R3 ✅); JOURNAL (the R3 close + milestone-close
   bridge); CLAUDE.md PLAY. The Multiparty milestone flips to ✅ — the horizon's next node opens
   (Round-2 whole-codebase audit / UI gate, per the ROADMAP chain).

---

## 6. Resolved runbook-level mechanics (the three named)

1. **Restart-during-chaos process-handle ownership (R3-D4a)** → **RESOLVED: not needed for R3** (§C2a).
   No R3 row requires a node restart during chaos (partition is relationship-level; MP-C-15 is an R2
   row). The `ManagedProcess`-ownership wrinkle does not arise. A future restart-during-chaos variant
   rides the routed transport-level seam.
2. **Equivocation payload (R3-D3)** → **pinned by observation at C1** (§C1): the self-membership
   leave/join fork is the candidate; author the directive, run the smoke once (keep-artifacts), observe
   convergence-on-winner on both nodes, record the observed winner; falsify-and-re-pin if the candidate
   doesn't conflict/propagate. Not pre-locked in code.
3. **MP-F11 spine + RED-on-revert (R3-D6)** → **spine-first at C5** (§C5): two `xgen-core` tests
   (`mp_f11_regular_space_populate_on_establish_drains` + `mp_f11_third_party_regular_space_content_blocked_by_f3`)
   prove the populate is a *legitimate relationship record* (F-3 intact) RED-on-revert, before the
   box-gated MP-A-01(ii) witness flips red→green.

---

## 7. Scope guard + discipline

**NOT in R3 (named homes — do not pull in):** the residents-multiplexer (R3-D1 → own `xgen-client`
arc); the transport-level partition seam + the reconnect-deadlock property (R3-D2 → routed); MP-F13's
root (J-278/F1B-D5 identity→home-node discovery → own arc); MP-C-06 re-home (M10); MP-F6 (M10); MP-F12
(own home); MP-F2-followon (M10, swept at close). **No production-crate change beyond C5 (MP-F11).**

**Discipline.** Per-commit DoD grounded against live code (D-078 — confirm the file:line anchors at
pickup before touching). Each named test by name (above). Surface-and-route (D-065/D-084);
pin-by-observation BEFORE routing. Spine-first for the one protocol change (C5). No "commit pushed"
line — Clair commits, Joe pushes. The box-gated smokes are `#[ignore]` (the fast suite never spawns
binaries); they run only at the box-gated RUN. Honest boundaries recorded (MP-A-08, MP-C-16, R3-D1
ceiling = process wall).

**Next step.** Joe-lock this runbook → Clair builds C1 → C2a → C2b → C3 → C4 → C5 (box-free) → Joe
frees the box → the box-gated RUN (re-bench first) → the bounded-gate fix-phase → close + the
consolidated ledger. Chat writes the next bridge at the build/RUN boundary.

**Entry point (Rule 0):** CLAUDE.md PLAY (J-349 head) → JOURNAL J-349 → `tasks/HANDOFF_MP_R3.md` →
`tasks/MP_R3_CAPSTONE_AUDIT.md` → `tasks/MP_R3_CAPSTONE_DESIGN.md` → this runbook →
`tasks/MP_findings.md` (MP-F11/F13) → `docs/tests/MULTIPARTY_TEST_MATRIX.md` §6.
