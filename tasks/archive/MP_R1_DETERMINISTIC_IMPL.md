# MP-R1 — Multiparty-tests Round 1 (deterministic correctness floor): Implementation Runbook
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The implementation runbook for **MP-R1**, executing the J-317 Joe-LOCKED design
(`tasks/MP_R1_DETERMINISTIC_DESIGN.md`, MP-R1-D1..D6). Clair may pick up at C1. The work is
**all in `xgen-mptest` + `docs/tests/multiparty_scenarios/`** — no production crate is touched;
real system defects become routed findings (MP-R1-D6), never patches.

Execution discipline: arc-local (D-069); atomic commits (D-074 — a scenario tranche's matrix
Result ships in the same commit as the scenario dirs + run); heavy entry points stay `#[ignore]`
/ out-of-band (they spawn real binaries — lib.rs hard constraint); the fast unit suite never
spawns a process. **R1 runs require a `--features harness-control` node build** (the F2/F3 seams
are fenced, M9.2-D1) — every scenario smoke header states the two-build invocation.

---

## 2. Grounded surfaces (so Clair does not re-discover)

The **hand-wired template** is `tests/c5_mp_c_02.rs` — it does, inline, exactly what
`run_scenario` generalizes: load → spawn node → attach collector → spawn clients → connect
aicontrol → `tokio::join!` the `run_actor`s against a shared `Registry` → quiesce → `members`
query → `convergence_verdict` → `Capture`. C1 lifts this into a generic function over N
nodes/actors + the federation step. `tests/m9_2_f2_add_peer.rs` is the federation-bootstrap
template (G-6); `tests/m9_2_f3_clock.rs` the clock-verb template; `tests/c4_injector.rs` +
`tests/m9_2_f4_malformed_frame.rs` the injector templates.

Confirmed API (live `xgen-mptest`):
- `binloc::locate() -> Result<Bins>`.
- `process::{instance_label(scenario, role), Kind, ManagedProcess, aicontrol_pipe(Kind,&label),
  events_pipe(Kind,&label)}`; `ManagedProcess::init_and_spawn_node(&bins,&label,port,local)`,
  `::init_and_spawn_client(&bins,&label,&url,ai_mode)` (kill-on-drop).
- `aicontrol::{AicontrolClient::connect(&pipe,timeout), DEFAULT_CONNECT_TIMEOUT}`;
  `client.send(&Command)`, `client.send_line(&raw)`.
- `wire::{Command::new(verb), Reply}`; `reply.is_ok()`, `reply.data()`, `reply.data_str(field)`.
- `manifest::{Scenario::load(dir), Manifest, ActorSpec, NodeSpec, FederationLink}`;
  `m.actor(name)`, `m.nodes`, `m.federation`, `scenario.batch_path(spec)`.
- `batch::{parse_batch_lines(text), run_actor(name,&lines,&mut ctl,m,&registry,timeout) ->
  ActorRun}`; `ActorRun::{all_ok(), reply_for(id), replies}`.
- `resolve::Registry::{new(), get(key).await, wait_for(key,timeout).await}` (publish is internal
  to `run_actor` via the manifest `exports`).
- `oracle::{convergence_verdict(&[proj],&[transcript],space_id), rejection_verdict(&[transcript],
  offending_id), MembershipProjection::from_members_data(node,&data), Transcript::from_values(node,
  &values), OracleVerdict}`.
- `events::{EventCollector::start(node,&events_pipe,Filter::all()).await, .snapshot().await}`.
- `dial::{RoundDial, ClockMode, RampProfile}`; `resource.rs` (RSS/thread sampling); `capture::Capture`.
- The F2/F3 seam verbs (harness-control build): `federation add-peer {node_id,url,spaces}`,
  `federation initiate {peer_node_id}`, `clock advance {duration}`, `clock set {timestamp}`.

---

## 3. Commit plan

### C1 — the `run_scenario` orchestrator + G-6 bootstrap + un-stale dial (D1/D1a)
- New `runner.rs`: `async fn run_scenario(scenario: &Scenario, dial: &RoundDial) ->
  Result<ScenarioOutcome>` generalizing the `c5_mp_c_02` flow over `manifest.nodes`/`actors`.
- `ScenarioOutcome { verdict: OracleVerdict, actor_runs: Vec<ActorRun>, resource: ResourceSample }`.
- **G-6 bootstrap helper** `establish_federation(&manifest, &node_ctls, &registry)`: for each
  `manifest.federation` link, run the J-315 sequence — `add-peer` both directions (empty spaces)
  at scenario start (before any register) → [actors register + the owner creates the Space, via
  their batches] → after the Space-id exports, `add-peer` again naming the Space → `initiate` from
  `link.from`. The runner interleaves the seam steps with the concurrent actor drive using the
  shared `Registry` (the post-Space re-seed waits on the owner's exported `space_id`).
- **Actor-kind dispatch**: a `manifest.actors` entry flagged injector (new optional `kind =
  "injector"` field, default batch) routes to the injector path (C7) instead of `run_actor`.
- **Un-stale `dial.rs` (G-2)**: `ClockMode::Mock` becomes valid; `RoundDial::validate()` accepts
  Mock; drop the "not operable" rejection + the stale module/"initiate does not exist yet"
  comments. (Operability now comes from the harness-control build; the runner asserts the build via
  the F3 reply shape, failing loud like the smokes.)
- **Proof**: re-express the Round-0 MP-C-02 (single-node) **and** stand up the true cross-node
  A↔B form through `run_scenario` (the C4/T1 deliverable depends on it; C1 proves the machinery on
  the single-node path + a 2-node federation smoke).
- DoD: build 0; clippy clean (default + `--features harness-control`); the existing `#[ignore]`
  smokes still pass; the new runner smoke passes out-of-band.

### C2 — the sweep contract (D2)
- New `sweep.rs`: `Sweep { axis: SweepAxis, start, step, max, stop_on_fail }`, `SweepAxis` enum;
  `SweepRung { dial, verdict, resource }`; `SweepResult { rungs, break_point: Option<BreakPoint> }`.
- `run_sweep(scenario, sweep) -> SweepResult` iterates rungs, each a `RoundDial`, calling
  `run_scenario`; classifies each rung **GREEN / LOGIC-FAULT / CEILING** by consulting the
  `OracleVerdict` **and** the `ResourceSample` (the D-065 distinction — a non-GREEN rung is CEILING
  iff resources show OOM/RSS-wall/thread-thrash/resource-death, else LOGIC-FAULT). Stop on
  LOGIC-FAULT (route a finding) or CEILING (record hardware break-point) or `max`.
- R1 uses a degenerate single-rung sweep; C2 builds the type + single-rung path + a **unit test**
  for the classifier (synthetic verdict×resource → expected GREEN/LOGIC-FAULT/CEILING). No process
  spawn in the unit test.
- DoD: build 0; clippy clean; classifier unit test green; fast suite +N (unit only).

### C3 — scenario clock-control + oracle rejection-category helper (D3/D4)
- Manifest `[[clock]]` table (manifest.rs): `Clock { node, op: "advance"|"set", value, after:
  Option<String> }`; `deny_unknown_fields`; validate `node` resolves. Parse-test.
- Clock-director task in `runner.rs`: each step blocks on `after` via `Registry::wait_for`, then
  sends the F3 verb on the named node's `.aicontrol`; no-`after` steps run at start. Runs as a
  sibling task to the actor drive.
- `oracle::rejection_category_verdict(run: &ActorRun, command_id, expected: ErrorCategory|code)`:
  reads the captured `Reply` for the command and asserts the error code/category (data already in
  `ActorRun.replies`). Unit-tested with synthetic replies.
- DoD: build 0; clippy clean; manifest + verdict unit tests green.

### C4 — Tranche 1: cross-node cooperative core (D5)
- Author `docs/tests/multiparty_scenarios/{MP-C-02,MP-C-03,MP-C-07}/` (per §4). MP-C-02 promotes
  the committed single-node batches to true **A↔B** (manifest gains a 2nd node + the `[[federation]]`
  link; batches unchanged bar node assignment).
- One `#[ignore]` smoke per scenario calling `run_scenario`; oracle = Space-scoped convergence.
- Update `MULTIPARTY_TEST_MATRIX.md` Result PENDING→PASS (or FAIL→`MP_findings.md`) for the three.
- DoD: build 0; clippy clean; the three smokes run out-of-band to a recorded result; matrix updated
  in the same commit (D-074).

### C5 — Tranche 2: membership-lifecycle cooperative (D5)
- Author `{MP-C-01,MP-C-06,MP-C-08,MP-C-09,MP-C-10,MP-C-13}/`; one smoke each; matrix Results.
- DoD as C4.

### C6 — Tranche 3: logic-adversarial (D5)
- Author `{MP-A-02,MP-A-03,MP-A-04,MP-A-14,MP-A-16,MP-A-17,MP-A-20}/`; each asserts the reply
  error **code/category** via `rejection_category_verdict` (C3). Matrix Results.
- DoD as C4.

### C7 — Tranche 4: wire/injector + clock (D5)
- Author `{MP-A-05,MP-A-09,MP-A-10,MP-A-12,MP-A-15,MP-A-01}/`. MP-A-05/09/10/12/15 use the
  injector actor-kind (C1 dispatch + `injector.rs`, grounded against `c4_injector.rs` +
  `m9_2_f4_malformed_frame.rs`); MP-A-01 uses a `[[clock]]` step (C3) to advance past `valid_until`.
  MP-A-05 re-runs the Round-0 forgery through `run_scenario`; MP-A-15 confirms the M9.1 wire-3046
  rejection at the binary. Matrix Results.
- DoD as C4.

### C8 — close (doc-only)
- `MULTIPARTY_TEST_MATRIX.md` roll-up final (R1 set all PASS or FAIL→routed); `tasks/MP_findings.md`
  authored iff any finding surfaced (mirror `M9_findings.md`); AUDIT/DESIGN/IMPL → COMPLETED;
  JOURNAL + ROADMAP + CLAUDE PLAY; suite reconciliation. Next-active = MP-R2 (its own Phase-0).

---

## 4. Scenario-authoring contract (Space-scoped, MP-R1-D4)

Each scenario dir holds one `<actor>.jsonl` per actor + `manifest.toml`. Ground the exact JSONL
verb/arg/binding shape against the **committed** `docs/tests/multiparty_scenarios/MP-C-02/` files
(the authoritative, real-arg-surface template) and matrix §2/§5: `register`→`identity_id`;
`create-space`→`space_id`; `create-room`→`room_id`; `invite` requires `role`→`event_id`; `join`
takes `{space, room?}` (no `invite_event` — pending-invite bootstrap + `prev_events`)→`space_id`;
`send` requires `room`→`event_id`. Cross-actor values via `{{key}}` from manifest `[[exports]]`;
non-data ordering via `[[waits]]`; clock actions via `[[clock]]` (C3).

**The contract (D-065 / D-G4):** every scenario asserts on its **own freshly-created unique
Space** — that Space's `event_id` set (`convergence_verdict`/`rejection_verdict`) and that Space's
membership projection across hosting nodes — **never** absolute node counts (`hosted_spaces`),
which the shared default `spaces_dir` pollutes across runs.

---

## 5. Definition of Done (per commit + milestone)

Per work commit: `cargo build` 0 errors; `cargo clippy` clean (default **and**
`--features harness-control` where the commit touches the fenced path); the fast unit suite green
(grows by new unit tests only); any new `#[ignore]` scenario smoke run out-of-band with its result
recorded in the matrix **in the same commit**. No "commit pushed" line (the `Status: COMPLETED`
header is the shipped signal). Milestone close (amended J-320, design §9 / MP-R1-D8): every R1 scenario carries a recorded
outcome ∈ {PASS · FAIL→routed · BLOCKED} — all-22-PASS is NOT the bar and is unreachable; findings
routed; BLOCKED scenarios logged in the test-debt ledger (design §9); the three canonical records
updated.

**Suite trajectory (not a frozen number):** baseline 1271/0/11 grows — fast count +the C2/C3 unit
tests; ignored count +the ~20 new scenario smokes. Reconciled explicitly at C8 (the C5/M9.2 pattern).

---

## 6. As-built notes + scope guard

- **Scope guard:** MP-R1 builds the runner + types + the 22 R1 scenarios + runs them. NOT the
  multi-rung sweep climb (R2/R3 stress it; C2 builds only the type + single-rung path), NOT
  `residents_per_process` multiplexing (G-5, R2/R3), NOT any binary/protocol change (findings route
  out, MP-R1-D6), NOT the R2/R3 scenarios (the other 13 matrix rows).
- **Determinism:** R1 = MockClock (harness-control startup install, fixed base instant) + fixed
  seeds; a scenario with no `[[clock]]` step runs at the pinned base instant.
- **Federation reality (honest, D-065):** the G-6 bootstrap uses the fenced `add-peer` that
  fabricates a pre-established relationship — acceptable only because un-buildable in release; it is
  harness wiring, not production peer-discovery (the M9.2 boundary, unchanged).
- **Honest boundary:** a green R1 is the correctness floor under no load — not scale, not coverage.

Suite 1271/0/11 at authoring (no code this phase). No DECISIONS change (MP-R1-D# arc-local, D-069).
Next-active: **Clair** — C1 → C2 → C3 → C4 → C5 → C6 → C7 → C8.

Per D-065 + D-069 + D-071 + D-074 + D-084.
