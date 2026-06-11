# MP-R3 (capstone) — design (R3-D1..D7)
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

The design beat for **MP-R3**, the capstone of the Multiparty-tests milestone. It consumes the
grounded Phase-0 audit (`tasks/MP_R3_CAPSTONE_AUDIT.md`) and locks the round's decisions as
**R3-D1..R3-D7** (arc-local, D-069 — none clears the global-principle bar for DECISIONS promotion;
the standing promotion candidates are noted in §7). **No code, no runbook** — the runbook
(`tasks/MP_R3_CAPSTONE_IMPL.md`) is authored after Joe's mechanical locks.

State at open: MP-R1 ✅ CLOSED (J-340), MP-R2 ✅ CLOSED (J-348). The Multiparty milestone stays
🟢 PLAY — R3 is the last sub-pass, inheriting the **loop-to-green BOUNDED-gate rerun character**
(R1 J-322 → R2 J-344 → R3): a box-gated RUN surfaces findings → a scope-frozen gate → rerun → close.

**Three scope forks are Joe-locked (by-recomms) and are recorded as locked here — not re-opened:**
**R3-D1** (residents-multiplexer routed; ceiling = the re-benched process wall), **R3-D2** (partition
= relationship-level), **R3-D6** (MP-F11 fixed in-round; MP-F13 carried red-with-reason). The
design-mechanical work below is **R3-D3 / R3-D4 / R3-D5 / R3-D7** (recommendations for Joe to lock,
in the MP-R2-D1..D6 by-recomms pattern).

**Method (the MP-R2 bar):** surface-and-route (D-065/D-084); **pin-by-observation BEFORE routing**.
Honest boundaries recorded where a row is witnessed with a scoped limit (D-065).

---

## 1. The three Joe-locked scope forks (recorded as locked)

### R3-D1 — residents-multiplexer → own production `xgen-client` arc; R3 ceiling = the re-benched process wall (LOCKED, Joe by-recomms)

The residents-multiplexer (audit §2.3 F-1 option (a): one `--ai-mode` client process driving N
logical identities) is a **net-new production `xgen-client` capability** and is **routed to its own
arc**, NOT built in R3. Consequence for the round: **"max the box bears" = the honestly-measured
process wall** (one logical participant = one OS process today — `logical_participants == clients`,
grounded at [`sweep.rs:361-393`](../xgen-mptest/src/sweep.rs#L361) + [`runner.rs:255-272`](../xgen-mptest/src/runner.rs#L255)).
The climb rows (MP-C-05/11/14, MP-A-07/18) **climb to the re-benched process wall, not to a
multiplexed logical-participant count.** This is a conscious scope call (audit §2.3): R3 proves the
box ceiling on **real processes** + the chaos overlay; thousands-of-logical-participants is the
routed multiplexer arc's job. The `residents_per_process` dial field + `SweepAxis::ResidentsPerProcess`
stay **decorative** (do not exercise them in R3); the routed arc bridges them later.

### R3-D2 — partition = relationship-level (defederate↔refederate); transport-level routed (LOCKED, Joe by-recomms)

MP-A-08's partition (audit §4.2) is modelled at the **relationship level**:
`federation defederate` (sever) ↔ `federation add-peer` + `federation initiate` (heal). All three
verbs are aicontrol-reachable today (`federation defederate` unfenced at
[`aicontrol.rs:366`](../xgen-node/src/aicontrol.rs#L366); add-peer/initiate the G-6 path), so this is
**director-wiring only** — no new transport seam. It **rides MP-F11**: the heal (refederate) is a
late-establish onto a peer that holds Space content during the gap, which is exactly the regular-Space
catch-up MP-F11's fix delivers (the populate-on-establish + F-3 drain). **The transport-level seam
(sockets cut, both nodes live, relationship intact, the production reconnect scheduler `xgen-node/src/reconnect.rs`)
is routed separately** — pursued only if the relationship-level model proves insufficient for
MP-A-08's oracle. **Honest boundary (D-065, recorded now):** the relationship-level partition
witnesses MP-A-08's **convergence-after-heal + no-lost-admitted-events** properties; it does **NOT**
exercise the **"no reconnect deadlock"** property (that is the transport-level reconnect scheduler).
So MP-A-08 at relationship-level is **harness-green-with-boundary** (sibling to MP-C-07 / MP-A-01(ii))
— the reconnect-deadlock half is named-and-routed to the transport-level seam, not silently claimed.

### R3-D6 — MP-F11 fixed in-round; MP-F13 carried red-with-reason (LOCKED, Joe by-recomms)

Decided **before the RUN** so the loop-to-green bounded gate is reserved for newly-surfaced RUN
findings, not the known-carried deps. **MP-F11 is fixed in-round** (the catch-up infra many R3 rows
ride — the role R2's D5 played); **MP-F13 is carried as a named non-green** (MP-C-16 stays
red-with-reason, routed to the J-278/F1B-D5 home). The fix-shape detail for both is §5.

---

## 2. R3-D3 — multi-target raw-wire injector (test-crate; drives MP-A-06, serves MP-A-08)

**The gap (audit §3 Cluster B).** The injector is single-target: one `node_url` per `InjectorHandle`
([`runner.rs:135-141`](../xgen-mptest/src/runner.rs#L135)), one `WireActor` per node
([`injector_actor.rs:101-111`](../xgen-mptest/src/injector_actor.rs#L101)). MP-A-06 equivocation needs
**fork-X → A, fork-Y → B** at one frontier — one hostile identity reaching ≥2 nodes.

**R3-D3 (recommended lock): a multi-target injector actor (test-crate-only).**

- **Shape:** extend the injector actor model so one injector holds a **`Vec<(node_label, WireActor)>`**
  (one raw-wire connection per target node, **all sharing one signing key** — the same hostile
  identity), instead of the single `url`/`WireActor` today. Manifest: an injector spec gains a
  **`nodes` list** (≥1; a single-element list = today's behaviour, backward-compatible). New attack
  directives target node labels; the existing single-target directives keep working against
  `nodes[0]`. **No production crate** — the `WireActor` + `injector` builders are already pub and
  construct crafted events; this is `xgen-mptest` wiring (`injector_actor.rs` + the `InjectorHandle`/
  `ActorSpec`).
- **Member context across nodes:** the hostile identity **registers once** (its `IdentityRecord`
  replicates to the federated peers via G-6) and **open-joins** the target Space (imported via
  `{{space_id}}`; XGen Spaces are open-join by default, J-275) so its events pass F-4 step-11 on
  every node it presents to.
- **The equivocation payload (candidate, pinned at the runbook by observation — NOT locked here):**
  two events that **share one `state_key` and conflict at the same frontier**, one presented to each
  node. The grounded candidate is a **self-membership fork** — a `membership.leave` (→ node A) and a
  `membership.join` (→ node B) of the hostile identity, both anchored to the **same** frontier tip, so
  they collide on `membership:{space}:{self}` and M8 resolution deterministically elects **leave**
  (Layer-1 leave>join, proven by MP-F7/J-348). Both events are valid (correctly signed); the
  equivocation is presenting **different** ones to different nodes. The runbook authors the exact
  pair + frontier by observation (other candidates: conflicting `thread.status`, or — if the hostile
  identity is seated owner/admin — conflicting `state.room_update`); the design does not over-specify
  the payload.
- **Oracle = the existing `convergence_verdict` (NOT net-new).** Equivocation's expected outcome is
  **convergence-on-winner** (both events apply, resolution elects one, no permanent fork), and
  `convergence_verdict` ([`oracle.rs:229-272`](../xgen-mptest/src/oracle.rs#L229)) already asserts
  all-node membership-set equality + cooperative transcript event-id-set equality. After both forks
  propagate (A receives Y via federation, B receives X) and `settle`, both nodes hold both fork
  events and resolve identically → the verdict passes **iff** convergence-on-winner holds. The
  runbook MAY add a transcript-level assertion that **both** fork events are present on both nodes and
  exactly one is resolved-out of state (sharpens "no permanent fork"), but the core convergence claim
  is the existing oracle.
- **"Serves MP-A-08" (the shared capability):** the multi-node-reach capability (one actor, N node
  targets) is the same capability MP-A-08's partition-side load wants (admit distinct events on each
  side of a partition, then heal + assert convergence). MP-A-08's load is primarily **batch actors
  per node** + the R3-D2 partition step; the multi-target injector is available for the adversarial
  variant. Not a hard coupling — D-3 is built for MP-A-06; MP-A-08 reuses the connection model.

---

## 3. R3-D4 — chaos-on-the-dial composition

**The gap (audit §4.1).** The director's only step kinds are Link / Clock / Migration
([`runner.rs:416-421`](../xgen-mptest/src/runner.rs#L416)); the load drivers (flood `event_flood`,
churn `run_storm`/`slow_loris`, restart `ManagedProcess::restart`) are **standalone, not composable
on the scale dial**. "Stacked chaos" = fault-injection running **while** the N×M chat drive runs.

The grounded ownership split decides the seam: a **partition (relationship-level, R3-D2)** + a
**node restart** need node aicontrol / `ManagedProcess` access (the director / runner own these); a
**flood / connect-storm / slow-loris** uses raw WS (`connect_url`), needs **no** node-conn borrow.

### R3-D4a — composition = hybrid two-seam (recommended lock)

- **A parallel chaos task** joined into the concurrent phase (alongside the actor drive + injector
  drive + director at [`runner.rs:329-353`](../xgen-mptest/src/runner.rs#L329)) owns the chaos
  **timeline** + the **raw-WS load drivers** (flood / storm / slow-loris — no `&mut nodes`). It
  schedules actions over a wall-clock window and **publishes timeline keys** to the shared `Registry`
  (e.g. `chaos_partition`, `chaos_heal`) so node-conn actions can gate on them.
- **Director chaos-steps** — a new `DirectorStep::Chaos` kind added to the F10-D1 ordered worklist
  ([`runner.rs:439-501`](../xgen-mptest/src/runner.rs#L439)) — execute the **node-conn actions**
  (relationship-level partition = `federation defederate`; heal = `add-peer` + `initiate`), gated on
  the keys the chaos task publishes, reusing the single-owner `&mut nodes` model (no borrow refactor).
- **Restart-during-chaos ownership wrinkle (flagged for the runbook):** the `ManagedProcess` vector
  is owned by `run_scenario`, not the director ([`runner.rs:190-222`](../xgen-mptest/src/runner.rs#L190)).
  A restart triggered mid-chaos must reach the process handle. The runbook resolves the plumbing
  (candidate: a restart action the runner performs at a gated point, or a shared handle/channel from
  the chaos task) — a design constraint named now, not a code decision.

### R3-D4b — during-chaos liveness probe (recommended lock; net-new, small)

A periodic **liveness probe** task in the concurrent phase: at intervals during the chaos window it
sends a benign aicontrol round-trip (`state`) to each node and, for the honest-traffic property,
optionally has a cooperative member post a message and verifies it lands post-heal. Records
responsiveness; an unresponsive node or honest-traffic-not-applying = a finding (faced-and-routed).
This is the capstone's "node stays live + honest traffic applies under sustained stacked chaos"
witness — net-new but small (a sibling of the churn/flood smokes' post-storm probe, made periodic +
interleaved rather than post-hoc).

### R3-D4c — elastic settle window (recommended lock)

The `settle` `MAX = 15s` ([`runner.rs:611`](../xgen-mptest/src/runner.rs#L611)) is tuned for small
cooperative runs and **will not hold a capstone heal + catch-up**. Lock: **parameterize the settle
bound** — a `settle_max_secs` on the dial or scenario (default 15 = today; capstone/chaos rows set
higher), keeping the existing **poll-until-stable** termination (stable-for-2-rounds → return early,
[`runner.rs:617-629`](../xgen-mptest/src/runner.rs#L617)) so a quiesced run still returns promptly.
The window extends the *ceiling*, not the *typical* settle.

### R3-D4d — the MP-F7 ≥2-projection oracle-edge for churn-at-scale (recommended lock)

`convergence_verdict` requires ≥2 **per-actor client `members`** projections
([`oracle.rs:234`](../xgen-mptest/src/oracle.rs#L234)); at capstone churn scale, actors legitimately
mid-leave at sample time return no projection (the J-348-flagged edge), spuriously failing the
precondition. Lock the **quiesce-then-sample** discipline + a **per-node** (not per-actor) convergence
key for churn-at-scale: gather the membership projection via a **stable client per node** (one
non-churning reader per node) so the convergence comparison is **node-to-node** and decoupled from
which churning client is mid-leave. The transcript half of `convergence_verdict` (node `.events`
set-equality) is already per-node; this aligns the membership half. (Recorded as the churn-at-scale
oracle adjustment; the small per-actor client projection stays available for non-churn rows.)

---

## 4. R3-D5 — CEILING-classifier capstone hardening

**The gap (audit §2.4).** R2 shipped a **per-process** ceiling signal (peak-RSS-wall + thread-thrash
+ None→ceiling-suspect, [`sweep.rs:199-232`](../xgen-mptest/src/sweep.rs#L199)); the R2-D4 caveats
2/3/4 (aggregate-RSS, OOM-exit, continuous sampling) were deferred. At capstone scale the wall is
**aggregate memory + processes dying**, not one peak process.

### R3-D5a — aggregate-RSS-vs-box-RAM (recommended lock)

Extend `peak_resource` ([`runner.rs:642-649`](../xgen-mptest/src/runner.rs#L642), which samples all
pids and takes the max) to also return the **sum** across all spawned pids; `is_resource_exhausted`
([`sweep.rs:202`](../xgen-mptest/src/sweep.rs#L202)) gains an **aggregate-RSS ≥ `BoxSpec::budget_bytes`**
check ([`bench.rs:52`](../xgen-mptest/src/bench.rs#L52)). This is the true capstone wall — the sum of
all node + client footprints against the box budget.

### R3-D5b — OOM-death-by-exit-code (recommended lock)

`ManagedProcess::try_exit_status` ([`process.rs:283`](../xgen-mptest/src/process.rs#L283)) exists and
is **unused by the classifier**. Add an "any spawned process exited unexpectedly (non-zero / OOM)"
signal to the rung outcome → the classifier treats it as **Ceiling** evidence (a process dying under
load is the clearest hardware-wall signal, the D-065 logic-vs-hardware split's strongest discriminator).

### R3-D5c — re-bench feeds the ceiling denominator: sample CLIENT RSS, not node-only (recommended lock)

`run_microbench` spawns only nodes ([`bench.rs:172`](../xgen-mptest/src/bench.rs#L172)); the
`from_bench` floor ([`sweep.rs:101`](../xgen-mptest/src/sweep.rs#L101)) derives from **node** mean
RSS. A real chat scenario is **N nodes + M clients**, and the climb wall (R3-D1) is on **total
processes**. Lock: extend the re-bench to spawn a representative **client** mix (or a sibling
client-bench) and derive the ceiling from the **combined (node + client) footprint**; the CEILING
denominator = total-process RSS budget. The re-bench is the **first RUN step** (R2's D4 discipline —
no scale number inherited; neither 1562 nor 1288).

---

## 5. R3-D6 fix shapes — MP-F11 (in-round) + MP-F13 (routed) [detail for the runbook]

### 5.1 MP-F11 — generalize Design-Z to regular Spaces on the federation-establish path (the in-round protocol fix)

**Grounded mechanism (audit §3 Cluster A).** The DM fix `repopulate_dm_federation_nodes` populates
`federation_nodes` from **members ∪ pending invitees → home nodes** but is **`dm_constraints_active`-gated**
(early-returns for regular Spaces, [`runtime.rs:2004-2007`](../xgen-core/src/node/runtime.rs#L2004));
the F-3 gate holds a late peer's content because that peer is absent from `federation_nodes`
([`runtime.rs:1007-1040`](../xgen-core/src/node/runtime.rs#L1007), `f3_reject` at
[`runtime.rs:1071-1078`](../xgen-core/src/node/runtime.rs#L1071)); the drain
`drain_pending_by_federation_relationship` ([`runtime.rs:1805`](../xgen-core/src/node/runtime.rs#L1805))
releases held events when the relationship lands.

**Fix shape (locked direction, R3-D6; exact code = runbook + Clair):** on the **federation-establish
path for a shared regular Space**, populate the late peer's `federation_nodes[S]` with the
establishing peer's node id as a **legitimate relationship record** (NOT an F-3 skip — the J-333 "an
unconditional F-3 skip is a hole" lesson: a non-party's node must never enter the set, or F-3 stops
blocking third parties), then fire `drain_pending_by_federation_relationship` to release the
F-3-held content. The machinery exists; the work is (i) a regular-Space populate path keyed on the
federation-establish event/handshake (lift the `dm_constraints_active` gate to a sibling regular-Space
path — DM keeps its members∪invitees rule; regular Spaces populate from the **established federation
relationship**, the real authority), and (ii) prove F-3 stays intact.

**The spine (proven, not assumed — D-076 family):** a RED-on-revert witness that the populate is a
*legitimate relationship record* (third-party content from a non-federated peer is still F-3-held)
**and** that the shared-Space content from the established peer now drains (MP-A-01(ii) flips
red→green). The spine test lives in `xgen-core` (the populate + F-3-intact assertion), mirroring
MP-F1b's hole-closed unit (`mp_f1b_third_party_dm_join_via_federation_blocked_by_f3`).

**Witness:** **MP-A-01(ii)** (`mp_r2_catchup::mp_a_01_ii_*`) flips KNOWN-FAIL → GREEN (the
real-binary regular-Space late-third-node catch-up); RED-on-revert = revert the regular-Space
populate → C's `federation_nodes` stays empty → content F-3-held → row red. **This is also what makes
R3-D2's relationship-level partition heal converge** (refederate = late-establish = the same
catch-up). So MP-F11 is load-bearing for **both** MP-A-01(ii) and MP-A-08's heal.

**Discipline:** a production `xgen-core` change → protocol-change discipline (convergence-safety,
D-076 ordering caution, RED-on-revert spine). Folded into R3 per R3-D6 (no separate milestone), with
its own per-commit DoD in the runbook.

### 5.2 MP-F13 — carried red-with-reason (routed)

**Grounded (audit §3 Cluster C).** Root = J-278: the client only ever learns the node's **WS URL**,
never its pubkey id, so `ops::create_space` writes a WS URL into the `NodeXgid`-typed `home_node`
([`ops.rs:448-456`, `405`](../xgen-client/src/ops.rs#L448)); `migration_initiate`'s
`st.home_node == rt.node_id` check ([`admin_ops.rs:2096`](../xgen-node/src/admin_ops.rs#L2096)) is
correct against the intended model → MIG_6010. The root fix is the **J-278/F1B-D5 identity→home-node
discovery arc** (a founding-philosophy arc bigger than R3). **MP-C-16 stays red-with-reason** (the
R1 MP-C-06 / R2 {MP-C-16, MP-A-01(ii)} "all-green-except" precedent). The symptom-fix **(b)**
(migration compares the node's own listen URL) is **NOT taken** (papers over the NodeXgid violation +
leaves the broader inconsistency for any other `home_node` comparator). Routed to the F1B-D5 home;
not chased in R3.

---

## 6. R3-D7 — R3 row set, close criterion, named build items, RUN structure

### 6.1 The R3 row set (the 9 + chaos overlay cross-cutting)

**(I) Capstone-climb rows — GREEN at the R2 floor; climb to the re-benched process wall + chaos (R3-D1).**
| Row | R3 capstone delta | Net-new build |
|---|---|---|
| MP-C-05 sustained n×n chat | climb to the process wall | re-bench (R3-D5c) |
| MP-C-11 membership churn under load | churn at the wall | churn-at-scale oracle (R3-D4d) |
| MP-C-14 4–5 node star+mesh topology | wider topology under load | **topology generator star→star+mesh** (§6.3) |
| MP-A-07 flooding / DoS | **intensity curve** | **flood no-drain enrichment** (§6.3) |
| MP-A-18 connect/disconnect storm | storm at capstone churn | chaos-on-dial composition (R3-D4a) |

**(II) New-capability rows — the round's build.**
| Row | Mechanism | Oracle |
|---|---|---|
| MP-A-08 partition + reconnect storm | relationship-level partition (R3-D2) — director chaos-step | convergence-after-heal (existing) — **boundary: no-reconnect-deadlock half routed** |
| MP-A-06 equivocation / fork | multi-target injector (R3-D3) | convergence-on-winner = existing `convergence_verdict` |

**(III) Named-dep witness rows.**
| Row | Dep | Disposition |
|---|---|---|
| MP-A-01(ii) federation-replay membership-preserved | **MP-F11** (fixed in-round, R3-D6) | **greens** once MP-F11 lands |
| MP-C-16 live space migration | **MP-F13** (routed, R3-D6) | **red-with-reason** (J-278/F1B-D5) |

**Chaos overlay** = cross-cutting capstone property (composed fault-injection on the scale dial,
R3-D4) — not a single row; witnessed by the climb + new-capability rows running **under** the
stacked chaos task + liveness probe.

**Count: 9 rows + the chaos overlay.**

### 6.2 Close criterion

**All-green-except-MP-C-16** — the R1 MP-C-06 / R2 {MP-C-16, MP-A-01(ii)} "all-green-except" shape,
narrowed to a **single** carve-out:
- **MP-A-01(ii) greens** once MP-F11 lands (R3-D6 — fixed in-round).
- **MP-C-16 stays red-with-reason** (MP-F13 routed to J-278/F1B-D5).
- **MP-A-08 is harness-green-with-boundary** (convergence-after-heal + no-lost-events green; the
  no-reconnect-deadlock half named-and-routed to the transport-level seam, R3-D2) — a recorded
  boundary, not a non-green.

What a green MP-R3 certifies (honest): the box-measured process wall under sustained stacked chaos +
the multi-node-adversary properties (equivocation convergence-on-winner, relationship-level
partition convergence-after-heal) + MP-F11's regular-Space late-catch-up — **NOT** multiplexed
logical-participant scale (R3-D1 routed), **NOT** the transport-level reconnect-deadlock property
(R3-D2 routed), **NOT** identity→home-node discovery (MP-F13 routed).

### 6.3 Named build items (grounded)

- **Topology generator star→star+mesh (MP-C-14):** `FederationPattern` is `None`/`StarFromFirst` only
  ([`sweep.rs:286-292`](../xgen-mptest/src/sweep.rs#L286)). Add a `Mesh` / `StarPlusMesh` pattern (the
  generator emits the extra `[[federation]]` cross-links, [`sweep.rs:371-377`](../xgen-mptest/src/sweep.rs#L371)).
- **Flood no-drain enrichment (MP-A-07 intensity curve):** `event_flood` is a paced ack-drained
  submit stream; the doc flags the true fire-hose (no-drain submit) as an R3 enrichment
  ([`churn.rs:116-138`](../xgen-mptest/src/churn.rs#L116)). Add the no-drain variant + a rate-curve
  driver (sweep the inter-send pace / fire-hose) → the intensity **curve**, not the R2 liveness point.
- Plus the R3-D3 (multi-target injector), R3-D4 (chaos composition + liveness probe + elastic settle +
  churn-at-scale oracle), R3-D5 (CEILING aggregate-RSS + OOM-exit + client-RSS re-bench), and the
  R3-D6 MP-F11 protocol fix.

### 6.4 RUN structure (mirrors R2 build-then-RUN; the runbook authors the commit plan)

1. **Build phase (box-free):** the net-new infra (R3-D3 injector, R3-D4 chaos composition, §6.3
   topology + flood, R3-D5 CEILING hardening) + the **MP-F11 protocol fix (R3-D6)**, unit-proven /
   in-process-proven, box-gated `#[ignore]` smokes authored. Mirrors R2's C1–C6 box-free build.
2. **Box-gated RUN:** first step = the **re-bench** (R3-D5c, client RSS included) → the climb rows to
   the wall + the new-capability rows + the chaos overlay + the dep-witness rows.
3. **Fix-phase (loop-to-green BOUNDED gate):** the RUN surfaces findings → the gate is **frozen at
   whatever surfaces** → each → GREEN-on-rerun or Joe-routed-with-reason → **rerun** → the true close.
   The carried deps (MP-F11 fixed in-round, MP-F13 routed) are decided **before** the RUN (R3-D6) →
   **not** gate items. Newly-occurring bugs faced-and-routed; the gate's scope is frozen (the R2
   discipline). R3 close = the consolidated R1+R2+R3 ledger (the standing HANDOFF deliverable).

---

## 7. R3-D# ledger + DECISIONS posture

| # | Decision | Status |
|---|---|---|
| R3-D1 | residents-multiplexer routed to own `xgen-client` arc; ceiling = re-benched process wall | **LOCKED** (Joe by-recomms) |
| R3-D2 | partition = relationship-level (defederate↔refederate); transport-level routed; MP-A-08 harness-green-with-boundary | **LOCKED** (Joe by-recomms) |
| R3-D3 | multi-target raw-wire injector (test-crate); oracle = existing `convergence_verdict` | recommended → Joe-lock |
| R3-D4 | chaos-on-dial: (a) hybrid two-seam (parallel task + director chaos-step) · (b) liveness probe · (c) elastic settle · (d) churn-at-scale per-node oracle | recommended → Joe-lock |
| R3-D5 | CEILING hardening: (a) aggregate-RSS · (b) OOM-exit · (c) client-RSS re-bench denominator | recommended → Joe-lock |
| R3-D6 | MP-F11 fixed in-round (generalize Design-Z populate+drain to regular Spaces on establish, F-3 intact); MP-F13 carried red-with-reason | **LOCKED** (Joe by-recomms) |
| R3-D7 | R3 row set (9 + chaos overlay); close = all-green-except-MP-C-16; named build items | recommended → Joe-lock |

**All arc-local (D-069).** No DECISIONS promotion in this design. **Standing promotion candidates
(Joe's call, the global-principle bar):** the **loop-to-green-with-a-bounded-gate round-close
discipline** (now three instances: R1 J-322, R2 J-344, R3) and **pin-by-observation-before-routing**
(the MP-R2 bar) — both flagged at J-348 as candidates, neither promoted here.

---

## 8. Discipline + next step

- Surface-and-route (D-065/D-084); **pin-by-observation BEFORE routing**. No code in the design beat;
  no runbook is Joe-locked yet.
- Two-commit close: this design (Clair's seat) commits FIRST (Joe pushes), then Chat's Phase-0-close
  doc-bridge (the J-NNN that flips CLAUDE.md PLAY + ROADMAP). Joe pushes; Chat never pushes.
- Honest boundaries recorded: MP-A-08 relationship-level (no-reconnect-deadlock half routed),
  MP-C-16 red-with-reason (MP-F13 routed), R3-D1 ceiling = process wall (not multiplexed).

**Next step: `tasks/MP_R3_CAPSTONE_IMPL.md`** (the runbook) — after Joe's mechanical locks on
R3-D3/D4/D5/D7. The runbook authors: the build-phase commit plan (R3-D3 injector → R3-D4 chaos
composition → §6.3 topology + flood → R3-D5 CEILING hardening → R3-D6 MP-F11 fix, each box-free with
unit/in-process proof + box-gated `#[ignore]` smoke) → the box-gated RUN order (re-bench first) → the
fix-phase bounded gate → the close (+ the consolidated R1+R2+R3 ledger, the standing HANDOFF
deliverable). The runbook resolves the named runbook-level mechanics: the restart-during-chaos
process-handle ownership (R3-D4a), the equivocation payload pinned by observation (R3-D3), the
MP-F11 spine + RED-on-revert witness (R3-D6/§5.1).

**Entry point (Rule 0):** CLAUDE.md PLAY (J-348 MP-R2-CLOSED head) → JOURNAL J-348 →
`tasks/HANDOFF_MP_R3.md` → `tasks/MP_R3_CAPSTONE_PHASE0_BRIEF.md` → `tasks/MP_R3_CAPSTONE_AUDIT.md`
→ this design → `tasks/MP_findings.md` (MP-F11/F13) → `docs/tests/MULTIPARTY_TEST_MATRIX.md` §6.
