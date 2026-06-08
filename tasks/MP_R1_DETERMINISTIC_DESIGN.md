# MP-R1 — Multiparty-tests Round 1 (deterministic correctness floor): Design
> **Status**: ACTIVE  
> Version: 1.4  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The design phase for **MP-R1**, the deterministic correctness floor of the Multiparty-tests
milestone. Executes the J-316 Joe-LOCKED structure and grounds the six forks from
`tasks/MP_R1_DETERMINISTIC_AUDIT.md` §4 into the decisions **MP-R1-D1..D6** (arc-local, D-069).
Feeds the runbook. NO code authored here.

MP-R1 builds the **general scenario runner** the harness lacks (audit G-1), un-stales the dial +
adds scenario clock-control (G-2/G-3), fixes the oracle contract for state isolation (G-4),
encodes the federation bootstrap sequence (G-6), authors the R1 batch set (22 scenarios), and
runs R1 to a recorded matrix result per scenario — surfacing (never patching) defects.

---

## 2. MP-R1-D1 (F-A) — the `run_scenario` orchestrator

**Decision: one generic runner, two actor kinds.** A single
`async fn run_scenario(scenario: &Scenario, dial: &RoundDial) -> Result<ScenarioOutcome>` covers
both families. The existing per-actor `run_actor` (batch.rs — connected client + lines + shared
`Registry` + exports/waits) is reused unchanged; the new code is the top orchestrator the Round-0
smokes hand-wired.

**Flow (the canonical sequence):**
1. **Spawn nodes** — one `ManagedProcess` node per `manifest.nodes` (label → run-nonce instance
   label, port, `local` flag), `--init` keypair, kill-on-drop. Connect each node's `.aicontrol`.
2. **Establish federation** — for each `manifest.federation` link, run the **G-6 bootstrap
   sequence** (MP-R1-D1a below). This is the runner's job, not each scenario's.
3. **Spawn actors** — one client `ManagedProcess` per `manifest.actors` entry (→ its node's
   `--node`/url, `ai_mode` flag), connect each actor's `.aicontrol`.
4. **Attach observation** — start an `EventCollector` per node before driving (events are
   live-only; attach-at-start, audit/M9-D4).
5. **Drive concurrently** — one tokio task per actor calling `run_actor` against the **shared
   `Registry`**. Concurrency is **required**: cross-actor `{{exports}}` and `[[waits]]` only
   resolve if producers and consumers run simultaneously (a sequential drive deadlocks the first
   consumer). Clock-director steps (MP-R1-D3) run as a sibling task ordered on the same registry.
6. **Join + settle** — await all actor tasks; a bounded settle window for federation replication
   to quiesce (poll-until-stable, not a fixed sleep where avoidable).
7. **Oracle** — query `members` per node + read each `EventCollector`; run the Space-scoped
   verdict (MP-R1-D4). Emit `ScenarioOutcome { verdict, per_actor_runs, resource_sample }`.

**MP-R1-D1a — the G-6 federation bootstrap sequence (encoded once, in the runner).** A cross-node
Space requires this order (established by the M9.2 F2 smoke; harness ordering, not a binary
change): **(i)** `add-peer` each direction (seed the relationship, empty shared-spaces) **before**
any identity registers — so `push_identity_to_peers` (app.rs:2824) replicates a registering
identity to the peer; **(ii)** actors register (identities replicate); **(iii)** the Space-owning
actor creates the Space; **(iv)** re-`add-peer` naming the now-existing Space id in shared-spaces;
**(v)** `federation initiate` from the link's `from` node → dials + replicates. The runner
sequences (i)/(iv)/(v) around the actor drive; (ii)/(iii) are actor-batch commands gated by the
export graph. Seam verbs are the M9.2 fenced `federation add-peer {node_id,url,spaces}` /
`federation initiate {peer_node_id}` — **R1 runs require a `--features harness-control` node
build** (documented in the runbook + the smoke headers).

**Two actor kinds.** A *batch actor* drives a `.jsonl` via `run_actor` (cooperative + logic
adversarial). An *injector actor* is the test-only raw-wire client (`injector.rs`, F4/MP-A-05/12)
— it does not go through `run_actor`; the manifest marks it so the runner routes it to the
injector path. The runner is one function; the actor kind is a per-actor dispatch.

---

## 3. MP-R1-D2 (F-B) — the sweep contract (locked now; R2/R3 inherit)

**Decision: a thin `Sweep` over `RoundDial`.** Today `RoundDial` is a single point (dial.rs); the
sweep is a new layer that yields a **sequence** of dials:

```text
Sweep { axis: SweepAxis, start: usize, step: usize, max: usize, stop_on_fail: bool }
SweepAxis ∈ { Nodes, Clients, ResidentsPerProcess, MessageRate, … }   // R1 uses none / a 1-rung sweep
```

The runner runs the scenario once per rung (each rung = a concrete `RoundDial`), recording
`SweepRung { dial, verdict: OracleVerdict, resource: ResourceSample }`. The run **result is a
curve + break-point**, not a bool: `SweepResult { rungs: Vec<SweepRung>, break_point: Option<…> }`.

**The mandatory distinction (audit §5, D-065) is built into the stop condition.** A rung's outcome
is one of: **GREEN** (oracle pass) → climb; **LOGIC-FAULT** (oracle fail — non-convergence / lost
admitted event / wrong rejection) → stop, route a finding; **CEILING** (oracle inconclusive +
`resource.rs` shows OOM / RSS wall / thread-thrash / process death-by-resource) → stop, record as
a **hardware** break-point, **not** a protocol FAIL. The runner must consult the `ResourceSample`
before labelling any non-GREEN rung, or it mislabels "ran out of RAM" as "protocol broke."

**R1's use:** a degenerate single-rung sweep (the smallest viable dial: 2–3 nodes, MockClock,
fixed seeds) through the **same** `SweepResult` type — so R2/R3 inherit the contract with no
retrofit. R1 builds the type + the single-rung path; the multi-rung climb mechanics are exercised
by R2/R3 (the type supports them, R1 does not stress them).

---

## 4. MP-R1-D3 (F-C) — scenario clock-control

**Decision: a manifest `[[clock]]` ordered step list.** New optional manifest table; per-actor
JSONL stays purely actor-driven (the clock is a scenario-director action, not an actor command):

```toml
[[clock]]
node  = "a"            # topology node label
op    = "advance"      # "advance" | "set"
value = "15d"          # duration (advance) | RFC3339 instant (set)
after = "invite_ready" # export key OR barrier name this step waits on (optional)
```

The runner runs clock steps as a director task: each step blocks on its `after` key (reusing the
`Registry::wait_for` ordering already in batch.rs), then sends the F3 verb (`clock advance
{duration}` / `clock set {timestamp}`) on the named node's `.aicontrol`. Steps with no `after` run
at scenario start. This unblocks **MP-A-01** (invite → advance clock past `valid_until` → replay).
MockClock operability comes from the `harness-control` build (un-stales dial.rs G-2: `ClockMode::
Mock` becomes valid when the node build supports it; the runner asserts the build via a probe verb
or the F3 reply, failing loud like the smokes).

**Determinism note.** R1 is MockClock + fixed seeds. Where a scenario has no clock step, the node
runs its startup-installed MockClock at a fixed base instant (the `harness-control` startup
install, M9.2-D3) — deterministic by construction, the property R1 exists to exploit.

---

## 5. MP-R1-D4 (F-D) — Space-scoped oracle as the contract

**Decision: every R1 scenario asserts Space-scoped; no binary change.** The oracle (oracle.rs) is
**already** Space-scoped — `convergence_verdict(projections, transcripts, space_id)`,
`Transcript::event_ids_for_space`, membership by `owner_id` + `(identity_id, role)` set (excluding
node-local `joined_at`/`invited_by`). Both Round-0 smokes proved the pattern. The contract:

- A scenario asserts on **its own freshly-created unique Space** — the Space's `event_id` set and
  that Space's membership projection across the hosting nodes — **never** absolute node counts
  (`hosted_spaces` etc.), which the shared default `spaces_dir` (`<exe_dir>/spaces`, G-4) pollutes
  across runs.
- **Rejection** = the offending `event_id` absent from every node's transcript (`rejection_verdict`)
  AND absent from membership. For **logic** rejections (MP-A-02/03/04/14/16/17/20) the assertion is
  the reply's **error code/category** captured by `run_actor` (`ActorRun` replies carry the
  `Reply`); a small `rejection_category_verdict` helper reads it (the data already exists, the
  helper is new). For **equivocation/convergence-on-winner** outcomes (not R1 — MP-A-06 is R2) the
  signal is convergence, not absence; flagged for R2, not built here.

The shared `spaces_dir` is recorded as a known harness constraint (not fixed in R1). An optional
per-instance `spaces_dir` override is a possible later hardening, explicitly **out of MP-R1 scope**.

---

## 6. MP-R1-D5 (F-E) — the R1 scenario set + authoring cadence

**The 22 R1 scenarios** (audit §6; matrix Round = R1). Authored + run in **four
mechanism-grouped tranches**, each a runbook commit, run fix→rerun before the next:

- **Tranche 1 — cross-node cooperative core:** MP-C-02 (true A↔B, promoting the Round-0 single-node
  PASS), MP-C-03 (concurrent send under conflict), MP-C-07 (DM space). *Exercises the new runner +
  G-6 bootstrap + multi-node convergence oracle — the riskiest path, first.*
- **Tranche 2 — membership-lifecycle cooperative:** MP-C-01 (local fan-out), MP-C-06 (identity
  re-home), MP-C-08 (multi-room + per-room overrides), MP-C-09 (ban → converge → post-rejected),
  MP-C-10 (leave & rejoin), MP-C-13 (thread create/resolve/archive).
- **Tranche 3 — logic-adversarial (batch-expressible):** MP-A-02 (over-ceiling/expired invite at
  submission), MP-A-03 (tier-gate join refusal), MP-A-04 (unauthorized/non-member send), MP-A-14
  (ban-evasion via new identity), MP-A-16 (forged invite), MP-A-17 (wrong-space_id), MP-A-20
  (privilege escalation). *All assert error code/category (MP-R1-D4).*
- **Tranche 4 — wire/injector + clock:** MP-A-05 (signature forgery — Round-0 PASS, re-run in the
  runner), MP-A-09 (duplicate-event_id dedup), MP-A-10 (causal gap / missing-parent), MP-A-12
  (malformed frame — F4 raw client), MP-A-15 (clock-skew — confirm M9.1 wire-3046 rejection at the
  binary), MP-A-01 (expired-invite federation replay — exercises the MP-R1-D3 `[[clock]]` step).

Each scenario lands its `docs/tests/multiparty_scenarios/<ID>/` dir (`<actor>.jsonl` +
`manifest.toml`) and its matrix Result moves PENDING → PASS (or FAIL→routed finding).

---

## 7. MP-R1-D6 (F-F) — defect policy

**Decision: surface-and-route, not patch** (M9 D-065/D-084 discipline). A scenario FAIL that is a
real system defect becomes a routed finding in a **new `tasks/MP_findings.md`** (mirroring
`M9_findings.md`: id, symptom, grounded rejection/divergence point, route to a fix-arc). A FAIL
does **not** block MP-R1 close. The deliverable is the **recorded result per scenario** (PASS, or
FAIL→routed finding), not all-green. Binary/protocol changes route out to their own arcs; MP-R1
touches only `xgen-mptest` + the scenario dirs + the canonical records.

---

## 7.5 MP-R1-D7 (added J-319, proven at C4) — federated-convergence oracle scope

**Decision: the per-Space `event_id`-set convergence comparison excludes federation-bootstrap
infra events by an explicit kind list; membership convergence is retained as the positive
"federation formed + state converged" assertion.** Surfaced by Clair at C4 (D-065) and grounded
three ways before locking — a scope correction to match the property the oracle verifies
(cooperative content converges cross-node), **not** a weakening.

- **Exclusion list (narrow, by explicit kind):** `INFRA_EVENT_KINDS = ["state.federation_add"]`,
  that kind only. The full `EventType` surface was checked: the other federation/bootstrap kinds
  (`bootstrap.register`/`keepalive`/`deregister`, `reputation.defederation_signal`,
  `migration.federation_notify`) are protocol/transport messages, not Space-DAG events, so they
  never enter a Space transcript. `state.federation_add` is the **sole** federation-infra event
  that lands in a Space DAG; no `state.federation_remove` exists (defederate is registry-only).
- **The asymmetry is registry-vs-DAG, directional, and benign (grounded, not the symmetric-pair
  guess):** the M9.2 `add-peer` on the **initiating** node A is a registry upsert only
  (`FederationRegistry::upsert` + `record_peer_url`, admin_ops.rs:1906) — it deliberately emits **no**
  Space-DAG event (the M9.2-D2′ fenced-relationship boundary); node **B** materializes the
  `state.federation_add` Space-DAG event on receipt. So the Space DAG holds **A:0 / B:1** — by the
  harness mechanism, not a lost event. `federation list` on both nodes confirms each registry holds
  the active link (each side formed its link); bidirectional convergence (invite A→B, join B→A)
  confirms events flow both ways.
- **The guardrail (not blind-ignore):** membership convergence stays the positive assertion — a
  genuinely-failed federation surfaces as membership divergence (a member absent on the far node),
  which the oracle still catches (and did: MP-C-07 / MP-F1 is exactly such a divergence, correctly
  flagged FAIL). The doc-comment on the filter cites the directional A:0/B:1 mechanism so a future
  reader does not "correct" it back to a wrong symmetric model; a unit test encodes that the
  exclusion makes an A:0/B:1 distribution converge.

Inherited by every cross-node scenario (C4–C7) and R2/R3. Arc-local (D-069).

---

## 8. Scope, change surface, proof

**Change surface (all in `xgen-mptest` + scenario dirs):** the `run_scenario` orchestrator + the
G-6 bootstrap helper (MP-R1-D1); the `Sweep`/`SweepResult` types + single-rung path (D2);
manifest `[[clock]]` parse + the clock-director task (D3); the `rejection_category_verdict` helper
+ un-stale `dial.rs` `ClockMode::Mock` (D4/G-2); the 22 scenario dirs (D5). **Untouched:** every
production crate (xgen-common/core/node/client) — findings route out (D6).

**Proof (the runbook details):** the runner drives Tranche 1 green end-to-end (the new-machinery
proof); each subsequent tranche runs to recorded results; the sweep type carries a unit test for
the GREEN/LOGIC-FAULT/CEILING classification; the `[[clock]]` step drives MP-A-01 deterministically.
Heavy entry points stay `#[ignore]`/out-of-band (spawn real binaries); the fast unit suite does
not spawn processes (audit/lib.rs hard constraint).

**Honest boundary (D-065):** R1 proves correctness under **no load** — a green R1 is the floor, not
scale (R2/R3) and not coverage. The runner is the deliverable + the 22 recorded results.

Suite 1271/0/11 (no code this phase). No DECISIONS change (MP-R1-D# arc-local, D-069). Next:
the MP-R1 runbook (`tasks/MP_R1_DETERMINISTIC_IMPL.md`) — the runner + types commit, then the four
scenario tranches → Clair.

---

## 9. MP-R1-D8 (added J-320) — close criterion amended + test-debt ledger

**Decision: MP-R1 closes when every R1 scenario carries a recorded outcome ∈ {PASS · FAIL→routed ·
BLOCKED}; all-22-PASS is NOT the close bar and is unreachable in R1.** Forced by the C5 + C6/C7
pre-flight findings (J-320): several scenarios depend on client authoring verbs / harness
capabilities that do not exist today (deferred to the UI/authoring-verb pass) and cannot be driven
without an out-of-scope production change (§8 / MP-R1-D6).

**BLOCKED is a valid terminal outcome** — a *coverage finding about the binary's authoring surface*,
not a defect (so NOT a `MP_findings.md` entry) and not a non-result. Surfacing these gaps is the
milestone delivering its intended value: a test pass over a happy-path-only driving surface that
came back all-green would mean the matrix wasn't probing anything new. The matrix legend carries
`🚧 BLOCKED — no authoring/harness capability; untested, not a defect, not closed`.

**What a green MP-R1 certifies (honest boundary, sharpened):** the happy-path cooperative core +
the adversarial-logic/wire core, under no load. It does **NOT** certify the admin / lifecycle /
threads / re-home surface — that is untested by capability gap, not by omission.

**Expected final R1 shape** (≈22 = ~15 PASS-eligible + ≥1 routed + 6 BLOCKED; the C6/C7 PASS set
firms up as those tranches run; MP-A-16 + MP-A-01 are finding-candidates that may route):

**Test-debt ledger — the deferred coverage the UI / authoring-verb pass inherits** (6 BLOCKED
scenarios across 5 root causes; each becomes runnable when its enabling capability ships):

- **member-ban verb gap** (`build_membership_event` builder-only) → **MP-C-09** (member-admin ban),
  **MP-A-14** (ban-evasion — depends on a banned-user precondition).
- **room-override verb gap** (PG-12, `build_room_update_event` builder-only) → **MP-C-08**.
- **thread verb gap** (PG-08, `build_thread_*` builders-only) → **MP-C-13**.
- **auth-tier-unsettable** (`ops::create_space` hardcodes `auth_tier=1`, ops.rs:357; PG-13 gate is a
  Tier-1 no-op today) → **MP-A-03** (tier-gate join refusal).
- **re-home gap** (harness: no key continuity across `--init` clients + aicontrol drops
  `node_override`; production: `home_changed` client broadcast deferred, J-278 CP-5 / J-279) →
  **MP-C-06**.

When the authoring verbs + re-home notify ship, these six become runnable — to be resumed as a small
**MP-R1-resumed** sweep or folded into the MP-R2 pass (structure decided post-C7, not here). The
debt is **owed and recorded**, not closed.

**MP-A-20 (reframe, J-320):** authored via the role-gate path (a non-privileged member attempts an
owner/admin-gated *client* verb — `invite` / `ai delegate` → `PermissionDenied`, category=permission,
the real `can_invite` gate), NOT the matrix's originally-named node-admin verbs (not client-issuable;
a `UNKNOWN_COMMAND` control-parse error would be the wrong category). Same escalation property, a
different valid instance of it — distinct from the MP-C-09/`node_eject` wrong-path substitution
(which exercised a *different* mechanism). Recorded as an as-authored matrix note.

Arc-local (D-069); amends the §8 close bar + the IMPL §5 milestone-close line.

---

## 10. MP-R1-D9 (added J-321, proven at C6) — logic-adversarial rejection oracle is path-split

**Decision: batch logic-adversarial scenarios assert rejection via a PAIRED oracle — the offending
`event_id` absent from every node's transcript AND the protected state unchanged. Category-level
assertion (`PermissionDenied`, 3045, 4000, …) is NOT batch-observable and lives on the C7 wire
path only.** Surfaced by Clair at C6 (D-065), grounded against live `connection.rs`/`ops.rs`. This
**amends MP-R1-D4** (whose §5 "logic rejections assert error code/category via
`rejection_category_verdict`" premise was wrong on the live rails) and **supersedes the MP-A-20
category instruction** (J-320, "assert category=permission").

- **Why batch can't see the category:** the client ops that get rejected — `invite` / `join` /
  `send` / `leave` — are `send_event` + goodbye with **no recv** (`Connection::send_event` is
  `to_vec` + `send_bytes`, write-only, connection.rs:120). The node's accept/reject (`EventAccepted`
  / `Error` per D-070) never returns to the aicontrol reply, so `run_actor` captures
  `{status:ok, event_id}` **regardless** of whether the node admitted or rejected the event. A
  category helper reading `ActorRun.replies` would see OK and mis-verdict. (Same fire-and-forget
  mechanism as MP-F1 facet-2 — `send` returns ok+event_id even when the event never lands — now
  understood as a *general property of the batch path*, not a DM-specific quirk.)
- **What batch CAN prove (the paired oracle):** the fire-and-forget op returns a locally-computed
  `event_id`; the node rejects it; that id is **absent** from every node's transcript
  (`rejection_verdict`) **and** the protected state is **unchanged** (target never gains membership
  / message never lands / no cross-space leak). The two halves together prove *the adversarial
  action had no effect* — the right security property for an R1 floor. Absence alone is too weak
  (benign reasons exist for an absent event); the state-unchanged half is required.
- **Where category lives:** the C7 injector/`WireActor` path submits the event and **recvs** the
  node's `Error` frame on the same connection (how MP-A-05 Round-0 read `Error(4000, …)`).
  `rejection_category_verdict` finds its real home in C7, asserting the specific code/category.
- **Consequence — every R1 "rejection" property splits by path:** batch (C6) proves
  *effect-absence*; wire (C7) proves *category*. C7 is where adversarial coverage gets its teeth.

Proven at C6: MP-A-02/04/17/20 all PASS under the paired oracle (4/4). Arc-local (D-069).

**MP-A-16 reclassified C6→C7 (J-321).** The batch form was **mis-premised**, not a defect: XGen
Spaces are **open-join by default** — an uninvited batch `join` legitimately succeeds and grants
member role (runtime.rs:1244 "an open join (no pending invite at all) is untouched"; J-275 INV-EXP
close). The `join` verb takes no invite-reference arg (the pending-invite bootstrap is node-side),
so a batch join *cannot* "reference a never-issued invite" — it can only do a legitimate open-join.
The genuine attack (a `membership.join` whose `prev_events` reference a fabricated/never-issued
invite predecessor → missing-predecessor / HeldPending → joiner never a member) is **injector-only**
→ C7, alongside MP-A-09/10. C7 assertion: node rejects/holds on the wire AND joiner never a member;
if the injector cannot reference a fake predecessor, or the node *accepts* it → route a finding.

**Confirmed-property breadcrumb (not a finding, not action — for the auth-tier work).** C6
empirically confirmed the open-join model: an uninvited join grants full member role on any Space,
by design (J-275). Combined with auth-tier being Tier-1-only today (MP-A-03 BLOCKED;
`ops::create_space` hardcodes `auth_tier=1`), there is currently **no join gate on the happy path**.
Intended — but a property the PG-13/auth-tier work (and the M10 auth-module pass) should weigh when
it lands, rather than rediscover. Recorded so it is visible there.

---

## 11. MP-R1-D10 (added J-323, Joe-LOCKED) — fix-and-rerun strategy; close criterion resolved

**Decision: MP-R1 does NOT close at C7. It enters a fix-and-rerun loop — fix the surfaced
production-code issues, then re-run R1, until all-green EXCEPT the one explicitly-deferred
scenario (MP-C-06 re-home). This supersedes MP-R1-D8** ("BLOCKED is a valid terminal outcome /
all-22-PASS unreachable"). The tests were the *instrument*; the deliverable is a corrected
protocol (Joe, J-322: "this is the purpose of that test, to fix production code").

**Grounded by the BLOCKED-sizing pass (Clair, J-323): 5 of 6 BLOCKED are THIN, 1 is a real arc.**
The sizing disproved the assumption the hybrid rested on (that the BLOCKEDs were UI-era feature
work to avoid building twice). The opposite is true: for ban / room-override / thread / auth-tier
the entire core path — `EventType` + builder + applier + permission gate + M8 convergence — already
shipped and is convergence-proven; the only gap is the **client `ops::*` verb** to drive it.
Building a thin driver verb is not building UI surface twice (the hard part is done + proven), so
the "build twice" risk that favored the hybrid evaporates → **loop-to-green wins for the 5 thin.**

**Root cause (the pattern, not 6 isolated gaps).** As the core grew (Arc D room-overrides, Arc E
threads, ban, auth-tier), each addition stopped at "EventType + builder + applier + convergence
proven" and never added the **client verb** to drive it — because nothing *needed* to drive it until
a test tried. The client-verb surface silently lagged the core surface; the unit tests passed while
the multiparty harness couldn't reach the feature. **Forward rule:** a core addition checks its CLI
equivalent at the time it ships, not a year later. (This is why MP-R1 found them — the discovery
function working.)

**The strategy (Joe-LOCKED):**
1. **Fix the 3 findings** — MP-F2 → MP-F3 → MP-F1, each its own production fix-arc (D-071 Phase-0).
2. **Build the 4 thin verbs** — `ban` (also unblocks MP-A-14), `room_update`, `thread` (×3), and a
   `create-space --auth-tier` param — each a small `xgen-client` arc over an existing core builder.
   Unblocks 5 scenarios (MP-C-08/09/13, MP-A-03/14).
3. **DEFER MP-C-06 (re-home)** — the sole real arc: needs the unbuilt `home_changed` client
   broadcast (J-278 CP-5 / J-279) **plus** harness keypair-relocation/per-command `--node`. Not
   load-bearing for R2/R3. The **one surviving test-debt ledger item**, carried to the
   M10 / re-home-notify era.
4. **Re-run R1** as fixes/verbs land; loop toward green.

**Close criterion (resolved from D8's "in flux"):** MP-R1 closes when R1 re-runs **all-green
except MP-C-06** — i.e. every scenario PASS, save the one explicitly-deferred re-home scenario
(recorded BLOCKED/deferred, not a fudge). "Thin" = low-risk shim, **not** zero-process: each verb
is a production-crate change that signs + sends real events, so each gets full per-arc discipline
(Phase-0 → lock → runbook → implement → close). Total remaining ≈ 3 finding-fixes + 4 verb-arcs +
R1 rerun + 1 deferred — a substantial body of work (weeks, not days), correctly sized as such.

**Each fix/verb arc is now a PRODUCTION arc** (`xgen-common`/`core`/`node`/`client`), unlike
C1–C7 which were `xgen-mptest`-only — protocol-change discipline applies (convergence-safety,
D-076 ordering caution, grounding-first).

Arc-local (D-069); supersedes the D8 close bar + the IMPL §5 milestone-close line.

---

## 12. MP-F2 fix-arc — Phase-0 verdict + design forks (Joe-LOCKED leans, J-323)

Phase-0 (Clair, J-323) **confirmed the gap**, low-severity (observability/contract, not security).
Root = a **two-boundary code-drop**, not a wire-shape gap: the `Error` wire frame already carries
`error_code: u32` (wire/types.rs); the value is dropped at (1) `dispatch_event` flattening
`ValidationOutcome::Rejected(ExchangeError)` → `DispatchOutcome::Rejected(String)` via
`err.to_string()` (runtime.rs:1086), and (2) `reject_signal` hardcoding `error_code: 4000`
(app.rs:2395) because it only receives the opaque string. MP-F2 is the deferred completion of
D-070's transport contract (D-070 shipped accept/reject + envelope-correlation; the specific
reject code was the named refinement, J-081). **D-076 discharged:** the reject code is admission
*output*, never input — derived from an already-produced `ExchangeError` on an already-rejected
(DAG-absent) event, written only onto the `Error` frame; it never feeds `state_key_for_event`,
the resolver, ordering, or `now`. No ordering surface in the blast radius.

**Forks (Joe-LOCKED leans for the design phase):**
- **F-1 carrier (the crux): (a) widen `DispatchOutcome::Rejected`** to carry the structured code
  (`{ code, name, reason }` or the `ExchangeError`/`Option<(u32,&str)>`) — single source of truth,
  unifies all reject sites. **NOT (b)** embed-and-re-parse a formatted string (the drift surface
  D-067 exists to prevent). Touches the ~13 `Rejected(...)` sites + ~10 test arms — worth it.
- **F-2 taxonomy scope: propagate-the-already-computed-code THIS arc only** (closes MP-A-15's
  3046). The unmapped variants (signature / membership / permission — `to_wire_code` maps only 5 of
  ~13) + the **spec drift Clair caught** (code emits `3030 tier_mismatch` vs spec §3.11.7
  `3010 auth_tier_insufficient`; ch3 §3.9/§3.6.5 signature-code scoping) are a **named follow-on**
  (its own finding/arc — MP-F2-followon), NOT silently absorbed. Consequence recorded: **MP-A-05
  stays generic-4000 until the follow-on** (fine — it already PASSes on-property).
- **F-3 origin gate: unchanged** — keep `LocallySubmitted`-only emission; orthogonal to the code.

Next (Track B): Clair authors the MP-F2 **design** on these leans → Joe-lock → runbook → implement
→ close. (Track A's verb-arcs follow the finding-fixes per the D10 strategy.)

Per D-065 + D-069 + D-071 + D-074 + D-084.
