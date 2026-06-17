# MP-R3 (capstone) — Phase-0 audit (grounded vs live `main`)
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

The D-071 Phase-0 grounding audit for **MP-R3**, the capstone of the Multiparty-tests milestone.
It executes the four-item checklist in `tasks/MP_R3_CAPSTONE_PHASE0_BRIEF.md` §3 **against live
`main`, to file:line** — no code, no runbook, no Joe-locks pre-empted. Its output feeds the design
phase (`tasks/MP_R3_CAPSTONE_DESIGN.md`, R3-D1..Dn → Joe-lock → runbook → RUN) and, specifically,
the **one open Joe-lock** (§6, fix-vs-route for {MP-F11, MP-F13}) — Clair grounds the cost, Chat
frames the forks, Joe locks.

State at open: MP-R1 ✅ CLOSED (J-340), MP-R2 ✅ CLOSED (J-348). The Multiparty milestone stays
🟢 PLAY — R3 is the last sub-pass. R3 inherits the **loop-to-green BOUNDED-gate rerun character**
(R1 J-322 → R2 J-344 → R3): a box-gated RUN surfaces findings → a scope-frozen gate → rerun → close.

**Method (the MP-R2 bar):** surface-and-route (D-065/D-084); **pin-by-observation BEFORE routing**
(three falsifications across the R2 stretch earned this). Every "this exists / this is absent" claim
below is grounded to a live file:line, not inferred from the prior round's prose.

---

## 1. Scope + headline

**Round definition (brief §1).** Capstone = **max the box bears, chaos overlay stacked.** The
handoff's ~1,562-process ceiling is **stale-suspect** and the R2 bench's ~1288 is the more recent
node-only figure — **neither is inherited**; the first RUN step re-benches (R2's D4 lesson).

**Four findings of this audit (the headlines, each grounded below):**

1. **The scale wall is not the box — it is the missing residents-multiplexer.** `RoundDial`
   carries `residents_per_process` ([`dial.rs:75`](../xgen-mptest/src/dial.rs#L75)) and a
   `logical_participants() = clients × residents_per_process` promise
   ([`dial.rs:102`](../xgen-mptest/src/dial.rs#L102)), and the sweep has a
   `SweepAxis::ResidentsPerProcess` ([`sweep.rs:124`](../xgen-mptest/src/sweep.rs#L124)) — but
   **nothing multiplexes.** `ScenarioTemplate::generate` consumes only `dial.nodes`/`dial.clients`
   and writes **one batch file ⇒ one client process per actor**
   ([`sweep.rs:361-393`](../xgen-mptest/src/sweep.rs#L361)); `run_scenario` spawns one
   `ManagedProcess` per actor ([`runner.rs:255-272`](../xgen-mptest/src/runner.rs#L255)). So
   `residents_per_process` is **decorative end-to-end** (the R2 G-1 finding, unchanged). One logical
   participant = one OS process today → the logical-participant ceiling *equals* the process ceiling.
   **This is the load-bearing net-new R3 infra and the round's pivot** (R2 deferred it explicitly).
2. **The bench measures NODE processes only.** `run_microbench` spawns only
   `init_and_spawn_node` ([`bench.rs:172`](../xgen-mptest/src/bench.rs#L172)); the ceiling estimate
   is `budget / mean_node_RSS` ([`bench.rs:58-63`](../xgen-mptest/src/bench.rs#L58)). A real chat
   scenario is **N nodes + M clients**; client RSS is never sampled. So "~1288" is a *node*-process
   ceiling, not a *total*-process one. The re-bench must measure client RSS (or confirm parity) and
   decide whether the ceiling is on `nodes + clients`.
3. **The deps cluster in three directions, and exactly one can balloon the round** (§3): A
   catch-up/federation-depth (MP-F11 + MP-A-08), B multi-node-adversary injector (MP-A-08 + MP-A-06),
   C identity-discovery (MP-F13). C is the deepest (J-278 root).
4. **The chaos overlay is partly built, partly absent.** Flood (`event_flood`, single-target,
   paced) and churn (`run_storm`/`slow_loris`) exist as **standalone drivers**, not composable on
   the scale dial; node **restart** exists (`ManagedProcess::restart`) but is not a director step;
   **no transport-level partition/link-cut primitive exists**; the **convergence oracle already
   serves "convergence-after-heal"** (it just needs a long-enough settle + a heal step to run before it).

---

## 2. Scale spine — re-bench + the residents-multiplexer (brief §3.1)

### 2.1 What the bench is, grounded

`xgen-mptest/src/bench.rs`: `run_microbench(bins, tiers, base_port)` spawns `tier` **node**
processes per tier ([`bench.rs:165-176`](../xgen-mptest/src/bench.rs#L165)), confirms each up via an
aicontrol `state` round-trip, samples RSS+threads (`crate::resource`), and derives
`estimate_ceiling(mean_rss) = budget_bytes / mean_rss` where
`budget = total_ram(32 GB) − reserved(4 GB)` ([`bench.rs:42-63`](../xgen-mptest/src/bench.rs#L42)).
Tiers come from `XGEN_MPTEST_BENCH_TIERS` (the real run = `10,50,100`)
([`bench.rs:145`](../xgen-mptest/src/bench.rs#L145)). The R2 RUN measured (J-344): ceiling ~1288,
mean node RSS ~22 MB, `from_bench` RSS wall ≈1110 MB (50× mean, per
[`sweep.rs:101-112`](../xgen-mptest/src/sweep.rs#L101)). **All node-only.**

### 2.2 The one-process-per-actor model (grounded)

- `run_scenario` spawns one node per `manifest.nodes` ([`runner.rs:190-222`](../xgen-mptest/src/runner.rs#L190))
  and one **client process per non-injector actor** ([`runner.rs:255-272`](../xgen-mptest/src/runner.rs#L255)).
  `injector`-kind actors spawn **no** process ([`runner.rs:244-252`](../xgen-mptest/src/runner.rs#L244)).
- The sweep generator writes one `.jsonl` per actor and one `[[actors]]` per actor, indexing nodes
  round-robin `a % nodes` ([`sweep.rs:378-393`](../xgen-mptest/src/sweep.rs#L378)) — it **never reads
  `residents_per_process`**.
- The client `--ai-mode` resident drives **one** identity (the M4 EchoPlugin); the
  `dial.rs:73-78` doc's "an AI resident drives many logical participants" is aspirational, **not
  built** (R2 audit G-1, [`MP_R2_SCALE_AUDIT.md` §G-1](MP_R2_SCALE_AUDIT.md)).

**Consequence:** total OS processes = `nodes + clients` ([`dial.rs:107`](../xgen-mptest/src/dial.rs#L107)),
and `logical_participants == clients` today. The R2 floor (MP-C-05 GREEN to 64 clients, no
break-point) was process-bound at 64; "max the box bears" without a multiplexer is just
"~ceiling-minus-nodes client processes."

### 2.3 The residents-multiplexer — the F-1 fork carried from R2 (the design-phase pivot)

The R2 audit framed three realizations of a logical participant ([`MP_R2_SCALE_AUDIT.md` §F-1](MP_R2_SCALE_AUDIT.md)),
deferred to R3:
- **(a) AI-resident multiplexer** — one `--ai-mode` client process drives N logical identities.
  **Net-new `xgen-client` capability** (the resident is one-identity today). Honors the two-number
  model (processes = HW wall; logical participants an order cheaper). Largest build; truest to scale.
  **Touches a production crate** → full protocol-change discipline (own Phase-0 sub-grounding in the
  design phase: how N identities share one WS / one keypair-set / one aicontrol pipe; do they share a
  client process's `ClientState`, or N states).
- **(b) Runner actor-expansion** — expand one `ActorSpec { residents = N }` into N client processes.
  Cheap, **but does not reduce process count** → defeats the two-number model, hits the box ceiling
  at the same `logical_participants` as one-per-process. A stopgap that must be named as such.
- **(c) Hybrid / batch-templating** — a manifest actor-template instantiated ×N processes with a
  `{{resident_index}}` substitution.

**Audit lean (NOT a lock — design-phase + Joe):** (a) is the only option that delivers what
`logical_participants()` promises and the only one that makes "capstone scale" mean more than
"~1200 client processes." Its cost (a real `xgen-client` resident change) is the reason this fork is
the capstone's biggest single decision. If the round is scoped to "prove the box ceiling on real
processes" rather than "prove thousands of logical participants," (b)/(c) suffice and (a) defers
again — but that should be a **conscious** scope call, not a default.

### 2.4 Re-bench plan (what the RUN's first step must measure — grounded asks)

1. **Re-run `run_microbench` with `XGEN_MPTEST_BENCH_TIERS=10,50,100`** on the freed box → the live
   node ceiling (replaces both 1562 and 1288). Feeds `CeilingFloors::from_bench`
   ([`sweep.rs:101`](../xgen-mptest/src/sweep.rs#L101)).
2. **Extend the bench (or a sibling) to sample CLIENT RSS** (the gap in §2.1) — `bench.rs` only
   spawns nodes; a capstone ceiling on `nodes + clients` needs the client footprint. If (a)
   multiplexing lands, also measure a **multiplexed** client's RSS vs `residents_per_process` (the
   real "cheaper by an order" claim — is it linear, sub-linear?).
3. **Decide the ceiling denominator** — node-only (today) vs total-process vs (with multiplexing)
   logical-participant. This is a design-phase output that the CEILING classifier's `from_bench`
   floor should reflect.
4. **CEILING-classifier hardening (R2-D4 deferred caveats 2/3/4).** R2 shipped per-process peak-RSS +
   thread-thrash + the None→ceiling-suspect inversion ([`sweep.rs:199-232`](../xgen-mptest/src/sweep.rs#L199)).
   Capstone scale wants the **deferred** enrichments: **aggregate-RSS-vs-box-RAM** and
   **OOM-death-by-exit-code** (`ManagedProcess::try_exit_status` already exists,
   [`process.rs:283`](../xgen-mptest/src/process.rs#L283), unused by the classifier) — at capstone
   scale the real ceiling is aggregate memory + processes dying, not one peak process. This is a
   concrete R3 build item, grounded as not-yet-built.

---

## 3. Dep clustering + per-dep fix cost (brief §3.2)

The four named deps **cluster, not list** — confirmed against live code. For each: the real surface
to file:line, and what a "fix" actually costs.

### Cluster A — catch-up / federation-depth (protocol): MP-F11 + MP-A-08

**MP-F11 — regular-Space content catch-up onto a late-federating third node, F-3-gated.**
Grounded surface:
- The F-3 gate: `dispatch_event` Step-2, **federation-channel events only** (`peer_node_id.is_some()`),
  consults `SpaceState.federation_nodes` ([`runtime.rs:1007-1040`](../xgen-core/src/node/runtime.rs#L1007)).
  A miss → **HeldPending on the federation-relationship trigger** (Phase 7.5 §6 held-not-bypassed),
  emitting `tracing::warn!(event="f3_reject", reason="federation_relationship_missing",
  disposition="held_pending")` ([`runtime.rs:1071-1078`](../xgen-core/src/node/runtime.rs#L1071)) —
  exactly the log the J-346 diagnostic pinned on node C.
- The skip set (Lock B1 + Phase 7.5 §5): only `StateFederationAdd | StateSpaceCreate |
  StateDmSpaceCreate` skip F-3 ([`runtime.rs:1027-1032`](../xgen-core/src/node/runtime.rs#L1027)) —
  `state.room_create`, `membership.*`, `message.text` are all gated. So a late peer that lacks A in
  its `federation_nodes` holds **all** of A's content.
- The DM fix (Design-Z, J-333): `repopulate_dm_federation_nodes` populates `federation_nodes` from
  **members ∪ pending invitees → home nodes**, but is **`dm_constraints_active`-gated** — it
  early-returns for regular Spaces ([`runtime.rs:2004-2007`](../xgen-core/src/node/runtime.rs#L2004)).
  It re-fires at every rebuild site (ingest create / `derive_resolved` arm / incremental apply arm /
  cold-start rehydrate — [`runtime.rs:519`, `665`, `687`, `697`](../xgen-core/src/node/runtime.rs#L519)).
- The drain: `drain_pending_by_federation_relationship` ([`runtime.rs:1805`](../xgen-core/src/node/runtime.rs#L1805))
  releases F-3-held events when the relationship lands; idempotent (unit-proven, `runtime.rs:2793/2863`).
- Regular Spaces populate `federation_nodes` via the vantage-aware `apply_federation_add`
  ([`state.rs:655-700`](../xgen-core/src/space/state.rs#L655)) — fired when a `state.federation_add`
  is applied. **The MP-F11 gap:** a late peer C establishing federation with A for a shared *regular*
  Space S never gets A into C's `federation_nodes[S]` (no `state.federation_add` for S lands on C in
  the right shape, no proactive populate for regular Spaces), so A's pushed content is F-3-held and
  no drain fires (J-346 diagnostic: `apply` count = 0).

**Fix cost (grounded).** Generalize Design-Z from DM-only to **regular Spaces on the establish
path**: on `federation initiate`/establish naming a shared regular Space, populate the late peer's
`federation_nodes[S]` with the establishing peer's node id, then fire
`drain_pending_by_federation_relationship`. The machinery (repopulate hook + the drain + D-091
invariant E) **already exists** — the work is (i) lift the `dm_constraints_active` gate to a
regular-Space populate path keyed on the federation-establish event/handshake, and (ii) prove F-3
stays intact (the J-333 "an unconditional skip would be a hole" lesson — the populate must be a
**legitimate relationship record**, not a skip). **Bounded but non-trivial** (it touches the F-3
surface on a new path — `xgen-core` node runtime + the federation establish hook). This is the
catch-up infra **many R3 rows ride** (the role R2's D5 catch-up machinery played) — see §6.

**MP-A-08 — partition + reconnect storm.** Its expected property ("convergence after heal; no lost
admitted events; no reconnect deadlock", matrix MP-A-08) is the **adversarial stress of the same
catch-up A delivers**: catch up the events *and* the identities registered during the gap (the F9-D2
generalized-establish-trigger was built "R3/MP-A-08-free" with this in view, J-345). **The
orchestrator surface is absent** — see §4.2 (no link-cut primitive); MP-A-08 is the net-new
orchestrator-link-control build (also Cluster B). Fix cost = the partition/heal director step (§4.2)
+ a convergence-after-heal oracle (the existing `convergence_verdict` + a heal step, §4.3).

### Cluster B — multi-node adversary injector (capability): MP-A-08 + MP-A-06

**MP-A-06 — equivocation / fork.** The injector is **single-target**: each `InjectorHandle` carries
one `node_url` ([`runner.rs:135-141`, `244-251`](../xgen-mptest/src/runner.rs#L135)),
and `run_injector_actor` builds one `WireActor` against that one node
([`injector_actor.rs:101-111`, `280-292`](../xgen-mptest/src/injector_actor.rs#L101)). Faithful
equivocation needs **fork-X → A and fork-Y → B** at one frontier — a **multi-target injector** (one
hostile actor connected to ≥2 nodes, presenting conflicting valid events). **Net-new capability.**
- **The oracle is NOT net-new** (a useful sharpening of the brief): equivocation's expected outcome
  is convergence-on-**winner** (both events apply, M8 resolution elects one, no permanent fork) — and
  `convergence_verdict` ([`oracle.rs:229-272`](../xgen-mptest/src/oracle.rs#L229)) **already** asserts
  all-node membership-set equality + cooperative transcript event-id-set equality. After the fork
  propagates + settles, both nodes hold both fork events and resolve identically → `convergence_verdict`
  passes iff convergence-on-winner holds. The net-new is the **injector's multi-target presentation**,
  not the verdict. (A small transcript-level assertion that *both* fork events are present + one is
  resolved-out of state could be added, but the core convergence claim is the existing oracle.)
- **MP-A-08 shares this capability class** (re-routed R2→R3 for exactly this reason, J-341): both need
  net-new two-node / multi-target injection + link control.

**Fix cost (grounded).** A multi-target injector actor (raw-wire `WireActor` per node, present
conflicting events at a shared frontier) in `xgen-mptest` — **test-crate-only** (the
`WireActor`/`injector` builders are pub and already construct crafted events). The convergence oracle
is reused. Bounded; no production crate.

### Cluster C — identity-discovery (protocol, deepest): MP-F13

**MP-F13 — Space `home_node` holds a WS URL, not a node pubkey id (NodeXgid contract violation).**
Grounded surface:
- `migration_initiate` rejects MIG_6010 when `st.home_node.as_str() != rt.node_id.as_str()`
  ([`admin_ops.rs:2096-2099`](../xgen-node/src/admin_ops.rs#L2096)). The check is **correct against
  the intended model** (`home_node : NodeXgid` should be a pubkey id).
- The defect is upstream: `ops::create_space` writes `home_node` from `ctx.session.home_node`
  ([`ops.rs:448-456`](../xgen-client/src/ops.rs#L448)) and wraps it `NodeXgid::from_xgid(Xgid::new(home_node))`
  ([`ops.rs:202`, `230`, `405`](../xgen-client/src/ops.rs#L202)) — but per **J-278 / MP-F1b** the
  client only ever learns the node's **WS URL** (`ws://…/xgen`), never its pubkey id. The node stores
  the signed content **verbatim** (can't rewrite signed content). So **every persisted `home_node`
  is a WS URL violating its NodeXgid contract**; migration is just the first comparator to hit it.
- The in-process Arc F tests dodged it by setting `home_node = node_id` explicitly (the J-347 note,
  `runtime.rs` test fixture) — MP-C-16 is the first real-binary Arc F migration with a client-created
  Space.

**Fix cost (grounded — this is THE balloon risk).** The root fix is the **client writing the node's
pubkey id as `home_node`, which requires the client to *learn* it** — i.e. the J-278 / F1B-D5
"production identity→home-node discovery" arc (already on the ROADMAP horizon). That is **a
founding-philosophy-territory arc, bigger than R3** (the F1B-D5 close note + J-279). Three
fix-shape forks (J-347, string-resolved):
- **(c) deeper J-278 dependency = the root, the route** — solve identity→home-node discovery.
- **(b) migration-resolves/compares-own-URL** (node compares `home_node` against its own known
  listen URL) — a bounded near-term **symptom-fix** that could green MP-C-16 *without* solving J-278;
  **flagged, NOT taken** at J-347 (papers over the NodeXgid violation + leaves the broader
  inconsistency for any other comparator).
- **(a) node-normalizes-home_node-on-ingest** — **blocked** (signed content; node can't rewrite).

**This is the dep that can balloon the round** (brief §2 Cluster C) → the §6 fix-vs-route Joe-lock.

### Cluster summary (for the design-phase map)

| Cluster | Deps | Net-new surface | Production crate? | Balloon risk |
|---|---|---|---|---|
| A catch-up / fed-depth | MP-F11 (+ MP-A-08 stress) | generalize Design-Z to regular Spaces on establish; the F-3 drain | **yes** (`xgen-core` node runtime) | medium — bounded, reuses machinery, touches F-3 |
| B multi-node injector | MP-A-06 (+ MP-A-08) | multi-target raw-wire injector | no (`xgen-mptest`) | low — oracle reused |
| C identity-discovery | MP-F13 | client learns node pubkey id (J-278/F1B-D5) | **yes** (`xgen-client` + node id surface) | **high — founding-philosophy arc** |

---

## 4. Chaos overlay — composition + oracle (brief §3.3)

### 4.1 What "stacked" needs vs what's built

The capstone wants fault-injection **composed on the scale dial** (partition + equivocation +
flood-curve, *while* N×M chat runs), not isolated single-property rows. Grounded inventory:

- **Flood (MP-A-07 intensity):** `event_flood(url, count, pace)`
  ([`churn.rs:118-138`](../xgen-mptest/src/churn.rs#L118)) — a single-target, **paced** member-context
  submit stream (the `pace` knob is the intensity). Standalone driver; the doc itself flags **"a true
  fire-hose (no-drain submit) is an R3 enrichment"** ([`churn.rs:116`](../xgen-mptest/src/churn.rs#L116)).
  The injector also has a per-directive `after_ms` pacing knob
  ([`injector_actor.rs:136-141`](../xgen-mptest/src/injector_actor.rs#L136)). **MP-A-07's
  intensity-*curve* → R3** (the curve, not the R2 liveness witness — matrix MP-A-07).
- **Churn (MP-A-18/19):** `run_storm` (open/drop cycles) + `slow_loris` (held-idle connections)
  ([`churn.rs:84-104`](../xgen-mptest/src/churn.rs#L84)) — raw post-handshake un-authenticated
  connections via `xgen_core::transport::client::connect_url`. Standalone, single-target. Liveness
  asserted by a post-storm aicontrol probe.
- **Restart (MP-C-15):** `ManagedProcess::restart` kills + re-spawns without re-`init` (replay from
  disk) ([`process.rs:239-275`](../xgen-mptest/src/process.rs#L239)). Exists, but is **not a director
  step** — used by a bespoke smoke, not orchestrable mid-scenario from the manifest.

**Gap (grounded): these are isolated drivers, not composable on the dial.** `run_scenario`'s
concurrent phase drives batch actors + injector actors + the director
([`runner.rs:329-353`](../xgen-mptest/src/runner.rs#L329)); the director's only step kinds are
**Link / Clock / Migration** ([`runner.rs:416-421`](../xgen-mptest/src/runner.rs#L416)). There is no
"flood during chat" / "partition mid-window" / "restart node X at T" director step. Composing chaos
on the scale dial is **net-new orchestrator wiring** (a chaos-step kind, or a parallel chaos task
joined into the concurrent phase).

### 4.2 The partition primitive — absent (the MP-A-08 / chaos load-bearing gap)

- **No transport-level link-cut exists.** The orchestrator can `add-peer`/`initiate` (establish,
  [`runner.rs:653-671`](../xgen-mptest/src/runner.rs#L653)), `restart` a whole node (node-down, not a
  partition), or open/drop raw client connections (churn) — but it **cannot sever the A↔B federation
  link while both nodes keep running**, then heal it.
- **`federation defederate` IS aicontrol-exposed** (unfenced, `admin_ops::federation_defederate`,
  [`aicontrol.rs:366`](../xgen-node/src/aicontrol.rs#L366)). So a **relationship-level** partition
  (defederate → re-add-peer → re-initiate) is *reachable as verbs*, but: (i) defederate removes the
  peer from `federation_nodes` (a clean relationship teardown, per the C9 hazard test), which is
  **not** the same as a transient transport cut with the relationship intact; (ii) the director has
  no defederate step + no heal-then-reconverge sequencing.
- **Two partition models — a real grounding fork for the design phase (NOT pre-locked):**
  - **Relationship-level** (defederate ↔ refederate): tests teardown + re-establish + catch-up
    (rides MP-F11's establish-path populate + drain). Reachable via existing verbs; needs director
    wiring only.
  - **Transport-level** (sockets cut, both nodes live, relationship intact): tests in-flight loss +
    the node's **production reconnect scheduler** (`xgen-node/src/reconnect.rs`, J-085 — the
    15/30/60/120-min ladder, the "no reconnect deadlock" the MP-A-08 oracle wants) draining the
    buffer on re-dial. More faithful to "partition + reconnect storm"; **no primitive exists** (would
    need a port-block / proxy-drop seam, or node kill+restart as a coarse approximation).

  The design phase + Joe pick the model; the audit's note: MP-A-08's *named* property ("no reconnect
  deadlock") points at the **transport-level** path (the reconnect scheduler), which is the harder
  build; the relationship-level path is cheaper and rides MP-F11 but tests a different thing.

### 4.3 The oracle under chaos

- **Convergence-after-heal** is **already served** by `convergence_verdict`
  ([`oracle.rs:229-272`](../xgen-mptest/src/oracle.rs#L229)): run the chaos → heal → `settle` →
  assert membership-set + cooperative transcript convergence across all nodes. The only adjustments:
  (i) the `settle` bound (`MAX = 15s`, [`runner.rs:611`](../xgen-mptest/src/runner.rs#L611)) is tuned
  for small cooperative runs — a capstone heal + catch-up may need a longer/elastic quiesce window
  (a design-phase tuning, grounded as a known constraint); (ii) the ≥2-projection precondition
  ([`oracle.rs:234`](../xgen-mptest/src/oracle.rs#L234)) is the MP-F7 oracle-edge — latently fragile
  for a churn scenario that legitimately ends an actor mid-leave (J-348 flagged it for future churn
  rows; capstone churn at scale is exactly that class).
- **Liveness-under-churn** is asserted today by a post-chaos aicontrol probe (the churn/flood smokes'
  pattern), **not** a during-chaos continuous oracle. A capstone "node stays live + honest traffic
  applies under sustained stacked chaos" wants a periodic liveness probe interleaved with the chaos —
  net-new (small).
- **No "lost admitted event" oracle beyond transcript set-equality.** MP-A-08's "no lost admitted
  events" = the cooperative transcript set equality after heal (an event admitted pre-partition must
  be present on all nodes post-heal). The existing set-equality serves it; the design must ensure the
  pre-partition admitted set is captured as the expected baseline.

---

## 5. R3 row enumeration (D-078, production-grounded — brief §3.4)

Enumerated against the matrix R3/PENDING rows + the four deps' witness rows, grounded against live
code (not inferred). Three kinds:

**(I) Capstone-climb rows — GREEN at the R2 floor; R3 climbs scale/width/intensity to the box wall + chaos.**
| Row | R2 state | R3 capstone delta | Gated on |
|---|---|---|---|
| MP-C-05 sustained n×n chat | ✅ GREEN to 64 clients (R2 floor) | climb to box ceiling (residents-multiplexed) | residents-multiplexer (§2.3) |
| MP-C-11 membership churn under load | ✅ GREEN-on-rerun (MP-F7) | churn at capstone scale | residents-multiplexer; oracle ≥2-edge (§4.3) |
| MP-C-14 4–5 node star+mesh topology | PENDING (R2→R3) | wider topology under load | topology generation (FederationPattern is star-only today, [`sweep.rs:286-292`](../xgen-mptest/src/sweep.rs#L286)) |
| MP-A-07 flooding / DoS | ✅ liveness witness (R2) | **intensity curve** (fire-hose enrichment) | `event_flood` no-drain enrichment (§4.1) |
| MP-A-18 connect/disconnect storm | ✅ (R2 C4) | storm at capstone churn | composable-on-dial chaos (§4.1) |

**(II) New-capability rows — net-new orchestrator (the round's build).**
| Row | Mechanism | Net-new | Oracle |
|---|---|---|---|
| MP-A-08 partition + reconnect storm | orchestrator link control (§4.2) | **partition/heal director step** (relationship- or transport-level — fork §4.2) | convergence-after-heal (existing, §4.3) |
| MP-A-06 equivocation / fork | multi-target injector (§3 Cluster B) | **multi-target raw-wire injector** (test-crate) | convergence-on-winner = existing `convergence_verdict` |

**(III) Named-dep witness rows — gated on the carried deps (the §6 Joe-lock decides fix-vs-route).**
| Row | Dep | Witness | If dep fixed | If dep routed |
|---|---|---|---|---|
| MP-A-01(ii) federation-replay membership-preserved | **MP-F11** | `mp_r2_catchup::mp_a_01_ii_*` | greens (regular-Space late-third-node catch-up) | stays red-with-reason (property J-298-proven in-process) |
| MP-C-16 live space migration | **MP-F13** | `mp_r2_fixed::mp_c_16_*` | greens (needs J-278 root or the flagged symptom-fix) | stays red-with-reason (R1 MP-C-06 / R2 MP-C-16 precedent) |

**Standing — explicitly NOT R3 (named homes, do not pull in):** MP-C-06 re-home → M10. MP-F6
(swallowed apply-error) → M10. MP-F12 (departed-signer re-dispatch) → own home. Production
identity→home-node discovery (F1B-D5, now joined by MP-F13) → own arc (unless §6 locks MP-F13
in-round). MP-C-12 (E2E) is built-with-D3-boundary, no longer R3-carried.

**R3 row count: 9** (5 climb + 2 new-capability + 2 dep-witness) + the **chaos overlay** as a
cross-cutting capstone property (composed fault-injection on the scale dial), not a single row.

---

## 6. THE open Joe-lock — fix-vs-route for {MP-F11, MP-F13} (brief §4)

**Decided BEFORE the RUN** — so the loop-to-green bounded gate is reserved for newly-surfaced RUN
findings, not the known-carried deps (the R2 lesson: the gate's scope is frozen; deps are
faced-and-routed to their disposition *before* the gate opens).

**Grounded inputs from this audit (the §3.2 cost the lock needs):**
- **MP-F11** is **bounded, in-round-fixable**: it reuses the shipped Design-Z machinery (repopulate
  hook + `drain_pending_by_federation_relationship` + D-091 invariant E), generalized from DM-only
  ([`runtime.rs:2004-2007`](../xgen-core/src/node/runtime.rs#L2004)) to regular Spaces on the
  federation-establish path. It is the **catch-up infra many R3 rows ride** (Cluster A — MP-A-08 and
  the late-fed witness lean on it). Risk: it touches the F-3 surface on a new path (the J-333 "an
  unconditional skip is a hole" lesson → the populate must be a legitimate relationship record).
- **MP-F13** is **deep**: the root fix is the J-278 / F1B-D5 identity→home-node discovery arc — a
  founding-philosophy arc bigger than R3. The only in-round greener is the **flagged-NOT-taken
  symptom-fix (b)** (migration compares own URL), which papers over the NodeXgid violation.

**Chat's recommendation (recorded by this audit's grounding, NOT locked — Chat frames the forks, Joe locks):**
- **MP-F11 fixed in-round** — it is the catch-up infra the capstone's catch-up/partition rows ride
  (the role R2's D5 played); bounded; reuses existing machinery.
- **MP-F13 carried as a named non-green**, routed to the F1B-D5 home arc (MP-C-16 stays
  red-with-reason — the R1 MP-C-06 / R2 MP-C-16 precedent), so R3 stays **closeable** and the
  capstone does not balloon into "solve identity discovery." The symptom-fix (b) is available if Joe
  wants MP-C-16 green at the cost of an acknowledged paper-over — flagged, not recommended.

This keeps the capstone's close criterion in the proven "all-green-except-{named deps}" shape (R1
MP-C-06, R2 {MP-C-16, MP-A-01(ii)}).

---

## 7. Discipline + next step

- Surface-and-route (D-065/D-084); **pin-by-observation BEFORE routing** (the MP-R2 bar). No code in
  Phase-0; no runbook is Joe-locked yet.
- The fix-vs-route call for {MP-F11, MP-F13} is the open Joe-lock — this audit feeds it (§6 cost
  grounding); Chat frames the forks; Joe locks. Don't decide it here.
- Two-commit close: Clair's audit/design arc-docs commit FIRST (pushed), then Chat's Phase-0-close
  doc-bridge (the J-NNN that flips CLAUDE.md PLAY + ROADMAP). Joe pushes. Chat never pushes.

**Next step: `tasks/MP_R3_CAPSTONE_DESIGN.md`** (R3-D1..Dn → Joe-lock → runbook → RUN). The design
phase locks: (D-1) the residents-multiplexer realization (F-1 (a)/(b)/(c), §2.3 — the round's pivot);
(D-2) the partition model (relationship- vs transport-level, §4.2); (D-3) the multi-target injector
shape (§3 Cluster B); (D-4) chaos-on-the-dial composition (the director chaos-step / parallel chaos
task, §4.1); (D-5) the CEILING-classifier capstone hardening (aggregate-RSS + OOM-exit, §2.4);
(D-6) the fix-vs-route Joe-lock for {MP-F11, MP-F13} (§6); (D-7) the R3 row set + close criterion
(§5 — all-green-except-{named deps}).

**Entry point (Rule 0):** CLAUDE.md PLAY (J-348 MP-R2-CLOSED head) → JOURNAL J-348 →
`tasks/HANDOFF_MP_R3.md` → `tasks/MP_R3_CAPSTONE_PHASE0_BRIEF.md` → this audit →
`tasks/MP_findings.md` (MP-F11/F13) → `docs/tests/MULTIPARTY_TEST_MATRIX.md` §6 →
`docs/ROADMAP.md` Multiparty node.
