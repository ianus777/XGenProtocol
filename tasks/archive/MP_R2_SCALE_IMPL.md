# MP-R2 — Multiparty-tests Round 2 (scale + real-clock): Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The implementation runbook for **MP-R2** (scale + real-clock). Executes the J-341 Joe-LOCKED design
(`tasks/MP_R2_SCALE_DESIGN.md`, MP-R2-D1..D6 + the §2 two-climb-mechanism falsification). Sibling-
in-shape to `tasks/MP_R1_DETERMINISTIC_IMPL.md`: a commit plan, grounded per-commit DoD + test
enumeration (D-078, named by symbol), the break-point-per-axis contract, RED-on-revert witnesses,
operational fences, the box-gated RUN order. **Runbook authoring only — no code, no run.** The box
(RUN gate, M-R2.3) stays held; the build commits (C1–C6) land unit-/parse-testable now, the heavy
sweeps + new-capability smokes stay `#[ignore]` and **RUN box-gated**.

**Change surface — test-crate-only (MP-R2-D1 / F-1 defer holds this).** Every commit lands in
`xgen-mptest` + `docs/tests/multiparty_scenarios/<ID>/`. **No production crate is touched** (the one
item that would have — the residents multiplexer — is deferred to R3). If any commit's grounding
shows a production-crate change is required, that is a **surface-and-route stop** (D-065/D-084), not
an in-runbook patch.

---

## 2. Grounding refinements surfaced before locking (D-065 — realization, not intent change)

Live-code grounding of the D-locks surfaced **two realization refinements**. Neither reverses a
locked *intent*; each is the cleaner *realization* of it. Flagged here for Joe's blessing at the
runbook lock (the loop's per-arc grounding catch — here it lands as two small realization
corrections, not a model change like the design's §2 axis falsification).

- **R-1 (refines D3 — where the dial→scale generation happens).** D5 §5 said "run_scenario consumes
  `dial.nodes`/`dial.clients` for sweep rows" + "the runner instantiates a templated actor
  ×`dial.clients`." Grounding `run_scenario` (`runner.rs:166-376`) shows it spawns **one node per
  `manifest.nodes` + one client per `manifest.actors`** and is cleanly manifest-authoritative. The
  **less-invasive realization**: the **sweep layer** generates a *concrete* `Scenario` (a concrete
  manifest with N actors/nodes + their generated paced batches) from a **template + the rung's
  dial**, then hands it to `run_scenario` **unchanged**. So `dial.nodes`/`dial.clients` are consumed
  by the **generator**, not by `run_scenario`'s spawn loop — the topology-authority invariant
  (`runner.rs:46-49`) stays TRUE for every row, the (a)-sweep / (b)-fixed-N boundary needs **no
  branch inside `run_scenario`**, and the (b) tranche is byte-unaffected. (`dial.worker_threads`
  **is** consumed by `run_scenario`/the spawn path — that is a genuine `run_scenario` change, G-6.)
  Net: D3 splits cleanly — **scale → sweep-layer generator; worker_threads → `run_scenario`.**
- **R-2 (refines D4 — bench→floors is a runtime struct, not const mutation).** D4 §6 said "wire
  `bench.rs` output → `RSS_WALL_BYTES`/`THREAD_THRASH_COUNT`." Those are **compile-time consts**
  (`sweep.rs:55,60`) — a runtime bench report cannot mutate them. The realization: a runtime
  **`CeilingFloors { rss_wall_bytes, thread_thrash_count }`** struct; `CeilingFloors::default()` =
  today's coarse consts (preserving R1 behaviour where no bench ran), `CeilingFloors::from_bench(&
  BoxCeilingReport)` = the calibrated floors; `is_resource_exhausted` + `classify_rung` take
  `&CeilingFloors`. Same outcome ("the floors are bench-derived"), realized as a passed struct.

Both are recorded in the per-commit scopes (C1, C2). Neither expands the change surface beyond
`xgen-mptest`.

---

## 3. Commit plan overview

Seven commits — six code (C1–C6) + one doc-only close (C7) — sequenced so the dial-bridge
dependency is explicit (the (a)-tranche gates on C1). The **box-gated RUN phase** (§12) runs between
C6 and C7, after Joe frees the box.

| # | Commit | D-lock | Box? |
|---|--------|--------|------|
| **C1** | Dial→sweep scale generator + batch-gen/pacing + `worker_threads` from dial | D3 + D2 (+ R-1) | No (build + units) |
| **C2** | CEILING calibration: `CeilingFloors` + bench-derived + None→Ceiling-suspect | D4 (+ R-2) | No (build + units) |
| **C3** | Late-federation/catch-up director capability | D5 | No (build + units; smokes `#[ignore]`) |
| **C4** | Connection-churn orchestrator driver | D1(c) | No (build + units; smokes `#[ignore]`) |
| **C5** | Tranche (a): scale/intensity scenarios MP-C-05/11 + MP-A-07 + the multi-rung sweep harness | D1(a) | Build no / **RUN box-gated** |
| **C6** | Tranche (b): new-capability fixed-N — SPLIT C6a–C6e (§9): C6a topology (MP-C-04/14, MP-A-13) · C6b restart+MP-C-15 · C6c migration+MP-C-16 · C6d MP-A-11/A-21 · C6e MP-C-12 (BUILT, injector-path node-blindness, D3-boundary) · + MP-A-18/19 (C4) & MP-A-01(ii) (C3) witnesses. **MP-A-06 → R3** (re-routed, J-342). | D1(b)+(c) | Build no / **RUN box-gated** |
| **C7** | Milestone-close (Chat doc-only, post-RUN) | — | No |

**Per-commit DoD convention (task-file rule).** Each commit's DoD is a checklist; it **does NOT
include "commit pushed"** (unflippable inside the push commit, and Joe pushes). `Status: COMPLETED`
on the arc docs is the shipped signal. Build commits gate on: `cargo build -p xgen-mptest`
+ `cargo build -p xgen-node --features harness-control` + `cargo build -p xgen-client` 0-error;
`cargo clippy -p xgen-mptest --all-targets -- -D warnings` clean; the fast `xgen-mptest` unit suite
0-failed (no process spawn); new heavy smokes present + `#[ignore]`-annotated.

---

## 4. C1 — dial→sweep scale generator + batch-gen/pacing + worker_threads (D3 + D2 + R-1)

**The structural heart. The (a)-tranche gates on this.**

**Scope.** (i) per-line `after_ms` pacing in the batch + injector drivers (D2); (ii) the offline
**scenario generator** (`ScenarioTemplate` → concrete `Scenario` at a given dial; consumes
`dial.nodes`/`dial.clients`, R-1); (iii) `run_sweep` evolves to take a template + generate-per-rung;
(iv) `run_scenario` consumes `dial.worker_threads` → the spawn path (G-6).

**Change surface (file:line, grounded):**
- `batch.rs` — `BatchLine` (`:33-40`) gains `after_ms: Option<u64>`; `parse_batch_lines` (`:45-64`)
  extracts it from the line JSON; `run_actor` (`:107-159`) sleeps `after_ms` (if any) **before**
  `send_line` (`:136`). A line with no `after_ms` sends immediately → **R1 batches byte-unchanged**.
- `injector_actor.rs` — `run_injector_actor` loop (`:116`) reads an optional `after_ms` off the
  directive and sleeps before each attack (gives MP-A-07 its flood pace).
- `process.rs` — `init_and_spawn_node` (`:123`) + `init_and_spawn_client` (`:159`) + `base_command`
  (`:241-248`) take a `worker_threads: Option<u32>`; the env helper `worker_threads()` (`:83-85`)
  becomes the **fallback** when the dial value is `None`. Callers updated: `runner.rs:175,236`,
  `bench.rs:170`.
- `runner.rs` — `run_scenario` (`:166`) reads `dial.worker_threads` and threads it to the two spawn
  calls.
- `sweep.rs` — NEW `ScenarioTemplate` (a manifest-template + per-actor batch-template); NEW
  `generate(&self, dial: &RoundDial) -> Result<Scenario>` (writes a concrete manifest + N paced
  `<actor-i>.jsonl` into a tempdir, returns a loaded `Scenario`); `run_sweep` (`:204`) signature
  evolves to `run_sweep(template: &ScenarioTemplate, sweep, base)` — per rung: `dial =
  axis.apply(base, value)` → `scenario = template.generate(&dial)?` → `run_scenario(&scenario,
  &dial)`. **A `ScenarioTemplate::Fixed(Scenario)` variant keeps the R1 single-rung path working**
  (the existing `mp_r1_sweep.rs` smoke updates to `Fixed`, ignoring the dial — R1 RED-on-revert
  protection).

**Tests (named by symbol, D-078):**
- `batch.rs::tests::parse_batch_lines_reads_after_ms` — a line with `"after_ms":50` parses to
  `Some(50)`; a line without parses to `None` (R1 batches unaffected). *(unit)*
- `batch.rs::tests::after_ms_absent_is_byte_identical_to_r1` — the existing R1 fixtures parse with
  `after_ms: None` (guards the byte-unchanged claim). *(unit)*
- `process.rs::tests::worker_threads_prefers_dial_then_env` — a pure `resolve_worker_threads(Option<u32>)`
  helper returns the dial value when `Some`, else the env/`"2"` default. *(unit — extract the helper
  so it is testable without spawning)*
- `sweep.rs::tests::template_generate_emits_dial_sized_manifest` — `ScenarioTemplate::generate` at
  `dial.clients = 4` writes a manifest with 4 actors + 4 batch files. *(unit — writes to a tempdir,
  no spawn)*
- `sweep.rs::tests::fixed_template_ignores_dial` — `ScenarioTemplate::Fixed` returns its scenario
  regardless of dial (R1 compat). *(unit)*

**RED-on-revert.** None this commit flips no matrix row; the R1 protection is `mp_r1_sweep.rs`
staying green under the `Fixed` template (revert the `Fixed` variant → R1 single-rung smoke fails to
compile/run).

**DoD (no "commit pushed").**
- [ ] `after_ms` parsed + honored in `run_actor` + `run_injector_actor`; absent ⇒ immediate send.
- [ ] R1 fixtures parse byte-identically (`after_ms_absent_is_byte_identical_to_r1` green).
- [ ] `worker_threads` consumed from the dial (env fallback); 3 call sites updated.
- [ ] `ScenarioTemplate` + `generate` + `run_sweep` template form; `Fixed` keeps `mp_r1_sweep.rs` green.
- [ ] 5 named unit tests green; fast `xgen-mptest` suite 0-failed; build (all 3 binaries) 0-error; clippy `-D warnings` clean.

---

## 5. C2 — CEILING calibration (D4 + R-2)

**Scope.** (1) bench-derived floors via a runtime `CeilingFloors` (R-2); (5) failed-rung-no-sample →
**Ceiling-suspect** (the deliberate R1-default reversal). Caveats 2/3/4 deferred to R3.

**Change surface (file:line):**
- `sweep.rs` — NEW `CeilingFloors { rss_wall_bytes: u64, thread_thrash_count: u32 }`;
  `CeilingFloors::default()` = the current consts (`RSS_WALL_BYTES` `:55` / `THREAD_THRASH_COUNT`
  `:60`, retained as the default's source); `CeilingFloors::from_bench(&bench::BoxCeilingReport)`
  derives calibrated floors (e.g. `rss_wall = k × report.reference_mean_rss_bytes()`,
  `thread_thrash` off the measured steady-state — the exact multiplier pinned against the bench
  report at RUN time, recorded in C7). `is_resource_exhausted` (`:149`) + `classify_rung` (`:155`)
  take `&CeilingFloors`. `run_sweep` threads the floors (default, or `from_bench` when a report is
  supplied).
- `sweep.rs::classify_rung` — **the None-branch flip (caveat 5).** Today (`:161-164`) a failed rung
  with `resource == None` ⇒ `LogicFault` (conservative). **Change: ⇒ `Ceiling`**, with the
  `BreakPoint.detail` carrying `"resource-sample-unavailable: ceiling-suspect"`. **Shape pinned:
  flip the None branch (NOT a 4th `RungClass` variant)** — `RungClass` stays `{Green, LogicFault,
  Ceiling}` (a suspect is a `Ceiling` for stop-purposes; a 4th variant would ripple into
  `BreakPoint`/`SweepResult`/stop-logic for no gain); the "suspect" provenance lives in `detail`.
  This **reverses** the R1 single-rung default — recorded as a deliberate R2 change (sampling fails
  most under memory pressure, exactly when a true ceiling should fire).

**Tests (named by symbol, D-078):**
- `sweep.rs::tests::ceiling_suspect_when_fail_with_no_resource_sample` — **replaces** the R1
  `logic_fault_when_fail_with_no_resource_evidence` (`:280-284`): a fail + `None` ⇒ `Ceiling` (the
  reversal). *(unit)* — **the RED-on-revert witness for caveat 5**: revert the None-branch flip →
  this test fails (and the old-named test would pass), proving the change is load-bearing.
- `sweep.rs::tests::floors_from_bench_derive_calibrated_walls` — `CeilingFloors::from_bench(report)`
  yields walls scaled off `report.reference_mean_rss_bytes()`, distinct from `default()`. *(unit)*
- `sweep.rs::tests::is_resource_exhausted_uses_provided_floors` — exhaustion is judged against the
  passed floors, not the consts. *(unit)*
- The existing `green_when_verdict_passes_regardless_of_resource` / `logic_fault_when_fail_with_healthy_resources`
  / `ceiling_when_fail_with_rss_wall` / `ceiling_when_fail_with_thread_thrash` update to pass
  `&CeilingFloors::default()` and stay green (regression).

**DoD (no "commit pushed").**
- [ ] `CeilingFloors` + `default()` (= consts) + `from_bench()`; `classify_rung`/`is_resource_exhausted` take it.
- [ ] None-branch flipped to Ceiling-suspect; `ceiling_suspect_when_fail_with_no_resource_sample` green; revert → RED.
- [ ] bench→floors derivation unit-proven; existing classifier tests updated + green.
- [ ] fast suite 0-failed; build 0-error; clippy `-D warnings` clean.

---

## 6. C3 — late-federation/catch-up director capability (D5)

**Scope.** The late-federation/catch-up director path — a node federates **after** a Space has
history, then catches up. **C3-scope refinement (J-342, Joe-nodded):** C3 ships the **catch-up /
late-federation path ONLY**; its in-commit witness MP-A-01(ii) needs only the late-federation
*ordering*. The **restart + migration primitives MOVED to C6** (C6b/c, §9) where their MP-C-15/16
witnesses immediately exercise them — shipping them untested in C3 was less verifiable. MP-A-01(ii)
(aged-Space replay) is C3's witness. *(The change-surface bullets below that mention restart/migration
reflect the pre-refinement plan; as-shipped restart = C6b, migration = C6c per §9 — full reconciliation
at the C7 close.)*

**Change surface (file:line):**
- `manifest.rs` — `FederationLink` (`:134-139`) gains `#[serde(default)] after: Option<String>` (an
  export key the link waits on before establishing — mirrors `Clock.after` `:211`). `validate`
  (`:252-260`) unchanged (the key is resolved at run time, like clock `after`).
- `runner.rs` — `run_director` (`:390-432`) federation phase: a link with `after = Some(key)` waits
  on `registry.wait_for(key, …)` **before** its re-seed `add-peer` + `initiate` (instead of the
  pre-drive seed at `:259-274`) → the late node federates after the owner's batch (and, for
  MP-A-01(ii), the `[[clock]]` aging) has built the Space; catch-up via the existing sync path.
- `process.rs` — NEW `ManagedProcess::restart(&mut self)` (kill child, re-spawn the **same**
  instance label → same data dir → replay-from-disk; for MP-C-15). Grounded against the kill-on-drop
  + instance-label model (`:201-210`, `:159-183`); restart reuses the label so the data dir persists
  across the kill.
- `runner.rs` — a director hook to drive a node restart (MP-C-15) and the migration verb
  (`migration initiate`, MP-C-16) as director steps (the migration verb is an existing aicontrol
  command; the restart is the new `ManagedProcess::restart`). *(MP-C-15/16 **scenarios AND** their restart/migration **primitives both land in C6** — the C3-scope refinement, J-342; see §9.)*

**Tests (named by symbol, D-078):**
- `manifest.rs::tests::parses_federation_link_after_key` — a `[[federation]]` with `after =
  "space_aged"` parses to `after: Some(...)`; without ⇒ `None`. *(unit)*
- `process.rs::tests::restart_reuses_instance_label_and_data_dir` — `restart` preserves
  `data_dir`/`aicontrol_pipe` (so replay-from-disk targets the same store). *(unit — no live node;
  asserts the label/dir invariants)*
- `tests/mp_r2_catchup.rs::late_federation_catch_up_converges` — **heavy `#[ignore]`**: A builds +
  ages a Space; B federates late (`after`); B catches up; convergence holds. *(box-gated RUN)*
- `tests/mp_r2_catchup.rs::mp_a_01_ii_aged_invite_replay_preserves_membership` — **heavy `#[ignore]`**:
  the MP-A-01(ii) witness (aged-Space catch-up does not re-reject the historical invited-join;
  membership preserved). *(box-gated RUN)*

**DoD (no "commit pushed").**
- [ ] `FederationLink.after` parsed; `run_director` late-establish path; `restart` primitive.
- [ ] 2 unit tests green; 2 heavy catch-up smokes present + `#[ignore]`.
- [ ] fast suite 0-failed; build 0-error; clippy `-D warnings` clean.

---

## 7. C4 — connection-churn orchestrator driver (D1(c))

**Scope.** A net-new raw-connection open/hold/drop driver (no such primitive exists — audit §2). Two
witness rows: MP-A-18 (connect/disconnect storm) + MP-A-19 (slow-loris / held connections). These
are inseparable from the driver, so their scenarios + smokes land **with** the driver here.

**Change surface (file:line):**
- NEW `xgen-mptest/src/churn.rs` (declared `pub mod churn;` in `lib.rs:69-86`) — a raw-WS connection
  driver built on the existing transport (`xgen_core::transport::client::connect_url`, the same entry
  `wireactor.rs:52` uses): `open_n(url, n)`, `drop_all`, a `storm(url, cycles, n)` (open N → drop →
  repeat, MP-A-18), and a `slow_loris(url, n, hold)` (open N, send a partial/no frame, hold, MP-A-19).
  Reuses `injector::inject_malformed_frame` shape for partial writes where useful. Test-crate-only,
  never ships (mirrors `injector.rs`).
- The driver asserts the **node stays live** (a post-storm legitimate `state`/create lands) — the
  M8.6 C4 attempt-gauge property at the binary; no task/handle leak observable from the harness side
  (node liveness is the proxy).

**Tests (named by symbol, D-078):**
- `churn.rs::tests::storm_plan_cycles_open_then_drop` — pure: a storm of `cycles=3, n=10` yields the
  expected open/drop sequence (no live socket). *(unit)*
- `tests/mp_r2_churn.rs::mp_a_18_connect_disconnect_storm_node_stays_live` — **heavy `#[ignore]`**:
  storm against a live node; a post-storm legitimate command still lands. *(box-gated RUN)*
- `tests/mp_r2_churn.rs::mp_a_19_slow_loris_does_not_exhaust_node` — **heavy `#[ignore]`**: held
  partial connections; honest traffic unaffected. *(box-gated RUN)*

**DoD (no "commit pushed").**
- [ ] `churn.rs` driver (open/hold/drop/storm/slow-loris); test-crate-only, never ships.
- [ ] 1 unit test green; 2 heavy churn smokes present + `#[ignore]`.
- [ ] fast suite 0-failed; build 0-error; clippy `-D warnings` clean.

---

## 8. C5 — tranche (a): scale/intensity scenarios + the multi-rung sweep harness (D1(a))

**Scope.** The headline curve+break-point tranche. Authors the templated scenarios + the multi-rung
sweep test harness; the **break-point RUN is box-gated**.

**Change surface:**
- `docs/tests/multiparty_scenarios/MP-C-05/` + `MP-C-11/` — **`ScenarioTemplate`** inputs (a base
  manifest-template + per-actor batch-template the C1 generator expands ×`dial.clients`, with
  `after_ms` pacing for the sustained window). MP-C-05 = sustained n×n (clients + intensity); MP-C-11
  = membership churn (clients + join/leave pacing).
- `docs/tests/multiparty_scenarios/MP-A-07/` — an injector flood batch (paced `after_ms` → high
  rate), the intensity sweep target.
- NEW `tests/mp_r2_sweep.rs` — **heavy `#[ignore]`**: builds a multi-rung `Sweep` over
  `SweepAxis::Clients` (MP-C-05/11) and an intensity sweep (regenerated batches at decreasing
  `after_ms`, MP-A-07); calls `run_sweep`; asserts a `SweepResult` curve + records the break-point.

**Tests (named by symbol, D-078):**
- `tests/mp_r2_sweep.rs::mp_c_05_sustained_chat_clients_sweep_curve` — **heavy `#[ignore]`**:
  multi-rung clients sweep; each rung oracle-checked (convergence + liveness); the curve +
  break-point (or all-GREEN to max) recorded. *(box-gated RUN)*
- `tests/mp_r2_sweep.rs::mp_c_11_membership_churn_clients_sweep_curve` — **heavy `#[ignore]`**.
  *(box-gated RUN)*
- `tests/mp_r2_sweep.rs::mp_a_07_flood_intensity_sweep_curve` — **heavy `#[ignore]`**: intensity
  rungs (decreasing `after_ms`); break-point = the pace at which liveness/back-pressure breaks.
  *(box-gated RUN)*
- `manifest.rs`/`sweep.rs` parse-level units for the new template inputs (named when the template
  shape is fixed in C1).

**Build DoD (now, no box):**
- [ ] MP-C-05/11/MP-A-07 template inputs present + parse via the C1 generator (a parse/generate unit
      per template, no spawn).
- [ ] `tests/mp_r2_sweep.rs` present, all `#[ignore]`; build 0-error; clippy `-D warnings` clean;
      fast suite 0-failed.

**RUN DoD (box-gated, §12):**
- [ ] bench-calibrated floors loaded (C2 `from_bench`).
- [ ] each sweep run produces a `SweepResult` curve; every rung classified GREEN/LOGIC-FAULT/CEILING.
- [ ] the **break-point per axis is recorded** (the deliverable — §11); a CEILING is logged as a
      hardware finding (not a protocol FAIL); a LOGIC-FAULT routes to `MP_findings.md`.
- [ ] **RED-on-revert** (per row that flips PENDING→recorded): the matrix flip is justified by a run
      whose break-point is reproducible (re-run isolated; spawn-timeout flakes classified distinctly
      from protocol RED — §12).
- [ ] matrix rows MP-C-05/11/MP-A-07 flipped PENDING → break-point recorded (Chat doc-bridge).

---

## 9. C6 — tranche (b): new-capability fixed-N (D1(b)+(c)) — SPLIT C6a–C6e (Joe-approved, J-341)

**Scope.** The never-run fixed-N rows. C6 was too large for one honest box-unverifiable commit (~5
net-new mechanisms + 9 scenarios), so it shipped as the **Joe-approved C6a–C6e split**. Build now;
**RUN box-gated.** As-built per sub-commit:

- **C6a — topology/static (ride existing `run_scenario`):** `mp_c_04_three_node_transitive_converges`
  · `mp_c_14_star_topology_converges` · `mp_a_13_anti_transitivity_c_does_not_receive_via_b` — inline
  tempdir scenarios in `tests/mp_r2_fixed.rs` (the `mp_r1_runner.rs` precedent; promotable to committed
  dirs post-RUN). No new mechanism.
- **C6b — restart + MP-C-15:** `ManagedProcess::restart` + `RespawnSpec` (`process.rs`) +
  `tests/mp_r2_restart.rs::mp_c_15_restart_replay_preserves_space`.
- **C6c — migration + MP-C-16:** `[[migration]]` manifest table + the director migration phase +
  `node_migrate` (`migration initiate`, grounded vs `admin_ops::MigrationInitiateArgs`) +
  `tests/mp_r2_fixed.rs::mp_c_16_live_migration_space_rehomes`.
- **C6d — single-node adversarial submits:** `tests/mp_r2_adversarial.rs::mp_a_11_oversized_payload_bounded_node_alive`
  (1 MiB message via `build_member_message` + node-liveness) + `::mp_a_21_stale_mls_commit_no_regression`
  (replay a stale `build_mls_commit_event` against an advanced epoch + node-liveness; the grounding
  gate — a stale commit is constructible test-crate-side without full MLS state — PASSED, state.rs:2140).
- **C6e — MP-C-12 (E2E content-blindness):** **✅ BUILT (C6e `6730f3e`, J-343).** Verdict: the client
  `--aicontrol`/`ops` surface has no e2e verb (production client e2e path D3-deferred, Arc H / J-257)
  → driving e2e via the client verb path is **STOP** (would falsify test-crate-only, §13 → route). The
  test-crate raw-wire injector/`WireActor` **can** construct + submit MLS/encrypted events directly
  (xgen-core builders pub; `app.rs` does so), so the **node-content-blindness core IS test-crate-
  expressible via the injector path** (client-*decrypt* half D3-gated). **BUILT (J-343):**
  `tests/mp_r2_e2e.rs` (box-gated `#[ignore]`) — a black-box replica of the Arc H content-blindness
  proof via `WireActor` (epoch key + `enc:` envelope constructed test-crate-side, node holds no key;
  `encrypt_message_envelope` pub, client_mls.rs:265); asserts node `.events`/store carry ciphertext
  only. **D3 boundary:** the client-*decrypt* half is NOT asserted (D3-gated). No production-crate
  change (the verdict held). RUN result box-gated.

**RE-ROUTED (Joe-locked, J-341 MP-R2 C6d):**
- **MP-A-06 (equivocation) → R3.** Grounding falsified the "fixed-N, rides the existing single-node
  injector" premise: faithful equivocation needs a **two-node / multi-target injector** + a
  convergence-on-winner oracle — the **same multi-node-adversary class as MP-A-08 (already R3)**. Moved
  to R3 to keep R2's mechanism surface lean (F-1 logic). **Not built in R2** (matrix MP-A-06 row updated).

**Build DoD (now, no box):**
- [ ] C6a/b/c/d scenarios + the restart/migration mechanisms present; `tests/mp_r2_{fixed,restart,adversarial}.rs`
      all `#[ignore]`; the manifest/sweep/process parse+unit tests green.
- [ ] build 0-error; clippy `-D warnings` clean; fast suite 0-failed.

**RUN DoD (box-gated, §12):**
- [ ] each fixed-N row (C6a/b/c/d) run to a recorded result (oracle pass/fail); LOGIC-FAULTs route to `MP_findings.md`.
- [ ] MP-A-01(ii) (C3 witness) + MP-A-18/19 (C4 witnesses) run + recorded.
- [ ] **RED-on-revert** per flipped row; spawn-timeout flakes classified distinctly (§12).
- [ ] matrix rows flipped PENDING → result (Chat doc-bridge); MP-A-06 recorded R3-deferred; MP-C-12 per Joe's C6e call.

---

## 10. C7 — milestone-close (Chat doc-only, post-RUN; Joe-locked)

**Scope.** After the box-gated RUN records every break-point + fixed-N result: the matrix roll-up
(§6 recount + each row's R2 result), `tasks/MP_R2_SCALE_{AUDIT,DESIGN,IMPL}.md` → COMPLETED, ROADMAP,
JOURNAL, the canonical `bench.rs` box-ceiling report archived, the C2 `from_bench` multiplier
recorded against the measured ceiling, and the **MP-R2-resumed-vs-fold decision** on any R3-deferred
continuations (MP-C-05/11/14, MP-A-07/18 R3 entry-rungs; MP-A-08 R3). **No self-close — the milestone
close is Joe's lock.** DoD does not include "commit pushed."

---

## 11. The break-point-per-axis contract (the R2 deliverable)

R2's deliverable is **not** pass/fail — it is the **curve + break-point per volume axis**:

- Each sweep runs the scenario **once per rung** (`run_sweep`, `sweep.rs:204`); each rung is
  oracle-checked and classified **GREEN** (climb) / **LOGIC-FAULT** (oracle fail + healthy resources
  → stop + route a finding) / **CEILING** (oracle fail + resource exhaustion, **or** fail + no sample
  per C2 caveat-5 → stop, hardware break-point, **not** a protocol FAIL).
- The break-point is `SweepResult.break_point` (`sweep.rs:177-191`): the rung index + dial + class +
  detail. **A CEILING break-point is a hardware finding** (recorded against the bench ceiling), never
  a protocol defect. **A LOGIC-FAULT break-point is a routed `MP_findings.md` finding** + does not
  block the R2 record.
- The matrix row's R2 result is the **recorded break-point** (e.g. "GREEN to N=K clients; CEILING at
  N=K+step, RSS-wall"), not a bare ✅/❌.

---

## 12. Operational fences + the box-gated RUN order

**RUN gate (held until Joe frees the box).** The build commits (C1–C6) land without the box (units +
`#[ignore]` smokes). The RUN phase is box-gated and ordered:

1. **bench first (calibrate D4 floors + fix the real R2/R3 numbers).** Run the `bench.rs`
   micro-benchmark with `XGEN_MPTEST_BENCH_TIERS=10,50,100` (`bench.rs:19,145`); its
   `BoxCeilingReport` (`estimated_ceiling`, `reference_mean_rss_bytes`) → `CeilingFloors::from_bench`
   (C2) **and** sets the concrete R2 sweep `max` (and the R3 numbers). **No sweep runs before the
   bench.**
2. **(a)-tranche sweeps** (C5): MP-C-05/11 (clients), MP-A-07 (intensity) → break-points.
3. **(b)-tranche fixed-N** (C6) + the C3/C4 witnesses (MP-A-01(ii), MP-A-18/19).
4. Results recorded → C7 close (Chat doc-bridge; Joe locks the milestone close).

**Binary-clobber fence (audit §7 / G-7).** `cargo test --workspace` rebuilds `xgen-node`
default-features over the `harness-control` binary at the pinned target dir → heavy tranches then
fail all-`UNKNOWN_COMMAND` on a fenced verb (`clock`/`add-peer`) — the J-315 fence-holds signal of a
clobbered binary. **Run the workspace check BEFORE the `--features harness-control` build, or rebuild
harness-control after any workspace build, before the heavy tranches.** All-`UNKNOWN_COMMAND` = the
binary, not the code.

**Spawn-timeout flakes ≠ protocol RED (Rule 2).** R2 contention is higher by construction. An
aicontrol pipe-connect / node-startup timeout under peak parallelism (R1's MP-C-10 precedent) is a
harness process-spawn flake, **not** a protocol RED — **confirm before classifying: re-run isolated.**
A break-point or a routed finding requires a reproducible (isolated re-run) signal, not a
parallelism flake.

---

## 13. Scope guard + honest boundary

**In scope:** the dial-bridge generator + pacing (C1), the CEILING calibration (C2), the
late-federation/catch-up + connection-churn infra (C3/C4), the 14 R2 scenario rows + the multi-rung
sweep harness (C5/C6), the box-gated RUN producing break-points (§12), the close (C7).

**Out of scope (NOT built here):** the residents-per-process multiplexer (R3 — F-1 defer; keeps R2
test-crate-only); CEILING caveats 2/3/4 (continuous/aggregate/injector sampling — R3); MP-A-08
partition+reconnect storm (R3); **MP-A-06 equivocation (RE-ROUTED → R3, J-341 C6d — needs a two-node
injector, the same multi-node-adversary class as MP-A-08)**; the R3 entry-rung continuations of
MP-C-05/11/14 + MP-A-07/18. **MP-C-12 (E2E):** the client-verb drive path is out of scope (D3-deferred
production exposure); the injector-path node-blindness core is **built (C6e, J-343, box-gated `#[ignore]`)** with the
client-*decrypt* half D3-gated (boundary recorded, §9).

**Surface-and-route (D-065/D-084).** A row that surfaces a genuine protocol defect under load routes
to `tasks/MP_findings.md` + its own fix-arc; it does not block the R2 break-point record. **No
production crate is touched in this runbook** — if a commit's grounding shows one is required, STOP
and surface (it would mean a D-lock falsified, not a patch to make).

**Honest boundary.** R2 proves the protocol holds under moderate-heavy load + real time and finds
the break-points off the nodes / clients / intensity / connection-churn mechanisms — **not** the
chaos capstone (R3), **not** density-per-process (R3 residents). A green R2 is a scale floor on the
freed box, read against the bench-calibrated ceiling.

---

## 14. Next

Joe-lock this runbook (blessing the §2 R-1/R-2 realization refinements) → Clair implements C1 → C6
(build, box-free) → Joe frees the box → the box-gated RUN (§12, bench first) → C7 close (Joe-locked).
Commit order: Clair's code/arc-doc FIRST, Chat's doc-bridge separate, Joe pushes both. No self-close.

Per D-065 + D-069 + D-071 + D-074 + D-078 + D-084 + MP-R2-D1..D6 + MP-R1-D2 (the inherited sweep
contract) + MP-R1-D8 (honest boundary) + MP-R1-D10 (surface-and-route).
