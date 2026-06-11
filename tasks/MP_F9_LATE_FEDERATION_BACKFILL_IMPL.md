# MP-F9 — late-federation identity catch-up — IMPLEMENTATION RUNBOOK (carries MP-F10)

> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Discipline note

Implementation runbook for the **Joe-LOCKED** MP-F9 design (`..._DESIGN.md` v1.1, terminal-state **A:
fix-in-fix-phase**). **Production-crate fix (`xgen-node`) — full arc discipline: NO code until this
runbook is Joe-locked.** F9-D1/D2/D3 + F10-D1 are locked; the within-A micro-decisions are runbook
detail below. Sequencing (Joe, ii): the trace is strong enough to design against, so the runbook is
authored now with the **confirming traced re-run as exec step 1** — box-gated, it **gates the fix, not
the authoring**.

**Two holds carried from the design (Joe, D-065):**
1. **F9-D3 behavior is HARD (Joe, runbook-lock).** C2 ships signer-set = **distinct senders of the
   backfilled delta** regardless of test budget — the behavior **never** narrows to members-only. Only
   the **departed-signer witness** may flex (carried + flagged for Joe if C2's budget is tight, §3 C2);
   the *behavior* does not. The risk closed: the test slipping and the behavior quietly becoming
   members-only because "the test isn't there to catch it." **Lock the behavior; let only the witness
   flex.** (R3-correctness: a reconnecting peer must validate historical events whose signers may have
   **since left**.)
2. **Terminal-A is conditional** — holds **iff** exec step 1 nails the `state.space_create`
   HeldPending discriminator **and** no real disclosure-scoping fork appears. **If the re-run
   contradicts the trace, or a genuine fork surfaces → STOP, surface it, revert to route-to-R3; do NOT
   shrink the fix.** MP-F10 still lands either way (harness, R2-internal).

**Scope guard (J-344):** gate item is F9+F10 only — no drift to F8/F7; anything new is faced-and-routed
to its own home, does not re-open the four-item gate.

---

## 1. The locked design (recap, file:line)

- **The gap is the trigger** (not machinery). Receiver auto-drains: `handle_identity_replicate_msg`
  (app.rs:2913) → `drain_pending_by_identity` (app.rs:2975 → runtime.rs:1717) releases F-10-held events
  on a record landing, **inside the upsert lock**. Sender send-path exists:
  `push_identity_to_peers` (app.rs:3118) — connect → authenticate → `Replicate` → ack → `add_replica`,
  whole `IdentityRecord` serialized (app.rs:3135). The peer is already in `peer_urls` at establish on
  **both** sides — initiator via `federation add-peer`→`record_peer_url` (admin_ops.rs:1941), receiver
  via `record_peer_url` at handshake (app.rs:1982).
- **F9-D1** fix = a backlog-push trigger on establish, reuse the above. **F9-D2** trigger on establish,
  both sides, symmetric. **F9-D3** signer set = distinct senders of the shared Spaces' backfilled
  history (NOT current members). **F9-D5** disclosure = the registration-precedent (whole record incl.
  `home_node` already replicated to federated peers), NOT D5.
- **F10-D1** dependency-ordered single-owner director: only `ClockPlan` carries `publishes`
  (runner.rs:165); the internal edges are `clock.publishes → step.after`; order steps so a fed
  link/migration/clock-step whose `after` matches a clock step's `publishes` runs **after** it.
  Preserves the `&mut [NodeHandle]` single-owner model (runner.rs:429-518); no deadlock.

---

## 2. Exec step 1 (GATE, not a commit) — the confirming traced re-run

**Box-gated — coordinate with Joe to run** (freed box + a `--features harness-control` `xgen-node`
build; `--test-threads=1`, background-spawned per the RUN discipline; a spawn/connect timeout is a
flake → re-run isolated, Rule 2). **This gates the fix (C3 below), not the authoring of this runbook.**

Run: `cargo test -p xgen-mptest --test mp_r2_catchup late_federation_catch_up_converges -- --ignored
--nocapture` against a harness-control node, with temporary instrumentation (a `tracing::debug!` at the
step-11 hold site if not already traced).

**PASS signature (verdict nailed → proceed to the fix):**
1. On **B**, `HeldPending { missing_identity: Some(alice) }` for **every** backfilled event —
   **including `state.space_create`** (the discriminator; even the F-3-skipping create holds,
   exchange.rs:629).
2. On **B**, the 4006 `identity_record_timeout` sweep eventually fires on those held events (no
   identity ever arrives).
3. On **A**, `push_identity_to_peers` takes the empty-`peer_urls` early-return (app.rs:3131-3133) at
   alice's `a1` registration.
4. On **B**, `id_registry` never contains alice (the session never replicated her).

**FAIL signature (revert to route-to-R3 — §0 hold-2):** if the create instead *applies* on B (non-empty
transcript / partial delivery), or B's registry already contains alice (some replication path I missed),
the localization shifts → **STOP, surface, route MP-F9 to R3.** (The zero-event symptom rules this out;
this is the honest-boundary close, not an expected branch.)

---

## 3. Commit plan (per-commit; NO code until Joe-locks this runbook)

Two code commits (different crates / concerns, D-074 per-commit atomicity), then the box-gated R2
rerun + Chat doc-bridge close. **C2 first** (production fix, gates both C3 rows); **C3 second**
(harness reorder, additionally gates Smoke 2).

### C2 — F9: late-federation identity catch-up (`xgen-node`, production)

**Change.**
1. **New helper** `replicate_space_signers_to_peer(runtime, node_keypair, peer_node_id, peer_url,
   shared_spaces)` (app.rs, sibling to `push_identity_to_peers`):
   - For each Space in `shared_spaces`, read `store.range(0)` and collect **distinct `ev.sender`**
     (F9-D3 — signers, not `SpaceState.members`); dedup across Spaces.
   - For each distinct signer, `identity_registry.get(sender)` → **if present**, send
     `IdentityReplicateMessage::Replicate` to `peer_url` (factor + reuse the
     `connect_url`→`client_authenticate`→`send_identity_replicate`→await-ack→`add_replica` inner of
     `push_identity_to_peers`); **if absent, omit** (honest-by-construction — F1B-D3 sibling; no
     crash/guess). Records are version-guarded → **idempotent** on re-establish.
   - Targets the **one** newly-established peer (not a re-spam of all `peer_urls`).
2. **Trigger at both establish hooks (F9-D2, symmetric):**
   - **Initiator** — `reconnect::attempt_reconnect` post-ACTIVE (reconnect.rs, after the handshake
     reaches ACTIVE / `drop(attempt_guard)` :480, around the post-handshake spawn :489): replicate A's
     own known signers of `shared_spaces` to the peer.
   - **Receiver** — `handle_federation_incoming` post-seal (app.rs:1980-1983, after `record_peer_url`,
     before `run_federation_session_post_handshake` :2000): replicate B's own known signers to the peer.
   - **Ordering-forgiving:** the push may run concurrently with / after the delta stream — the
     receiver's `drain_pending_by_identity` releases held events as each record lands (no
     before/after-the-stream constraint).

**Named tests (D-078) — new `xgen-node/src/tests/late_federation_identity_catchup.rs`** (in-process
two-node, sibling to `reconnect_integration.rs` / `phase9_*`; the per-commit witness — the box-gated
Smoke 1 is the R2-rerun witness):
- `late_federated_peer_catches_up_history_and_signers` — A registers alice + builds a Space with
  history (create + room + post); B federates **late** via the reconnect/establish path; assert B's
  store carries the Space's events **AND** B's `identity_registry` contains alice. **RED-on-revert:**
  neuter the trigger (skip `replicate_space_signers_to_peer`) → B's events stay F-10-held / store empty
  for the Space (the zero-event reproduction in-process).
- `late_catch_up_replicates_departed_signer` (F9-D3 correctness witness) — a signer who posted then
  **left** the Space still gets their record replicated (their historical event validates on B). Guards
  against the members-only drift (hold 1). *(If the in-process harness can't cleanly model a leave +
  historical event in C2's budget, record it as a known F9-D3 obligation carried to the box-gated
  rerun + flag for Joe — do not silently narrow to members-only.)*

**DoD (C2):**
- `cargo build -p xgen-node` 0-error (default **and** `--features harness-control` **and**
  `--all-features`); `cargo clippy -p xgen-node --lib --tests --all-features -- -D warnings` clean.
- `cargo test -p xgen-node` 0-failed; the two new tests GREEN; **RED-on-revert** confirmed on
  `late_federated_peer_catches_up_history_and_signers`.
- **Prime invariant (early-federation byte-identical):** existing federation/reconnect tests
  (`reconnect_integration`, `federation_*`, `phase9_*`) stay green — the trigger is **additive +
  idempotent**, no change to the early-federation path or to any wire shape (D-077 backward-coherence;
  no new `IdentityReplicateMessage` variant, no new receive handler).
- No production behavior change beyond the additive establish-trigger.

**Greens:** Smoke 1 (`late_federation_catch_up_converges`) at the R2 rerun; **precondition for Smoke 2's
row** (bob's identity + historical join catch up onto the late node C).

### C3 — F10-D1: dependency-ordered single-owner director (`xgen-mptest`, test-crate)

**Change.** In `run_director` (runner.rs:437-518), replace the fixed federation → clock → migration
sequence with a **dependency-ordered single pass** over all director steps:
- Build the internal published-key set = `{ clock.publishes for each ClockPlan }` (only `ClockPlan`
  publishes — runner.rs:165).
- Order steps so any step (LinkPlan / MigrationPlan / ClockPlan) whose `after` ∈ internal-published-keys
  runs **after** the clock step that publishes it (topological over the `clock.publishes → step.after`
  edges); steps with no internal dependency keep their manifest-relative order; **external** `after`
  keys (`history_ready`, `bob_join_ready`, `space_id` — published by the concurrent actor drive /
  `PRIMARY_SPACE_KEY`) are `wait_for`'d as today.
- Execute the ordered list **sequentially** over `&mut nodes` (single owner preserved — no borrow
  refactor). Migration still runs after its `space_key` resolves.

**Named test (D-078) — unit, no spawn** (in `runner.rs`/`manifest.rs` test module):
- `director_orders_fed_link_after_its_clock_gate` — a manifest with a fed link `after =
  "clock_advanced"` + a clock step `publishes = "clock_advanced"` → the computed director order places
  the clock step **before** the fed link. **RED-on-revert:** the fixed-order (federation-all-first)
  director places the fed link first → the dependency is unsatisfiable (the deadlock the fixed order
  produces).

**DoD (C3):**
- `cargo build -p xgen-mptest` 0-error; `cargo clippy -p xgen-mptest --tests --all-features -- -D
  warnings` clean.
- `cargo test -p xgen-mptest` fast suite 0-failed (the new ordering unit + existing
  manifest/director/dial units); ordering unit GREEN + RED-on-revert.
- The early-link path (no `after`, or `after` = an external key) is **unchanged** — the existing
  passing scenarios (MP-C-05 sweep, the C6 fixed-N rows that use the director) keep their behavior
  (prime invariant for the harness).

**Greens:** Smoke 2's **deadlock** (`mp_a_01_ii_aged_invite_replay` no longer times out at the
director). The row **fully greens only with C2 also** (the identity+join catch-up).

---

## 4. The R2 rerun + close (witness gate)

After C2 + C3, **box-gated R2 rerun** (R1's `a9fbd98` precedent — re-run the affected smokes to
green-to-criterion; coordinate with Joe / freed box):
- `cargo test -p xgen-mptest --test mp_r2_catchup -- --ignored --nocapture` (both smokes), against a
  freshly-built `--features harness-control` node (**rebuild harness-control AFTER any
  `--workspace` build** — the J-315/J-340 clobber fence).
- **Witness:** Smoke 1 GREEN (B catches up the Space — events + signers) → **MP-F9 GREEN-on-rerun
  (terminal-state A satisfied)**; Smoke 2 GREEN (no deadlock + aged-invite join preserved on C) →
  **MP-F10 GREEN-on-rerun** (and confirms C2's identity catch-up onto C).
- **Gate ledger:** MP-F9 → GREEN-on-rerun; MP-F10 → GREEN-on-rerun. Two of the four gate items
  terminal. (MP-F8, MP-F7 follow per the J-344 sequence.)

**Close = Chat seat doc-bridge** (`MP_findings.md` MP-F9/MP-F10 → RESOLVED + gate ledger, JOURNAL,
ROADMAP, matrix C3 rows). The arc docs (AUDIT / DESIGN / this IMPL) flip to COMPLETED at the doc-bridge
close. **Commit order (standing):** Clair's code FIRST (C2, C3), then Chat's doc-bridge; Joe pushes.

---

## 5. Scope guard / honest boundary / what this runbook does NOT do

- **No code until Joe-locks this runbook.** Production-crate fix → full arc discipline.
- **F9-D3 behavior is hard** (hold 1) — C2 ships delta-signers regardless of test budget; the behavior
  never narrows to members-only. Only the departed-signer *witness* may flex (carried + flagged), never
  the behavior.
- **Terminal-A is conditional** (hold 2): exec step 1 must nail the discriminator; a contradicting
  re-run or a genuine disclosure-scoping fork → STOP + route-to-R3, not shrink-to-fit.
- **Does NOT** touch F8/F7 or re-open the four-item gate. The within-A micro-decisions (delivery-path
  reuse vs over-session, batching, departed-signer modelling) are C2 detail, not gate items.
- **MP-F10 is independent** — it lands (C3) regardless of the MP-F9 terminal state; only its
  contribution to Smoke 2's *row* green depends on C2.

**Confirm-at-pickup (D-078) — resolve against live `main` at C2/C3 start, don't guess:**
- C2: the exact factor boundary of `push_identity_to_peers`' inner send (one-record-to-one-peer reuse);
  the initiator hook line in `attempt_reconnect` (post-ACTIVE, around :480-489); whether the in-process
  test harness (`reconnect_integration` template) can drive a late establish + a registered identity +
  Space history without box-gating.
- C3: the precise `run_director` rewrite that keeps the `&mut nodes` single-owner model + the
  external-vs-internal `after`-key partition (manifest `[[exports]]`/actor keys vs `clock.publishes`).

---

*Per D-065 (surface, don't shrink-to-fit) + D-069 (arc-local F9-D#/F10-D#) + D-071 (runbook follows the
locked design) + D-074 (per-commit atomicity) + D-077 (backward-coherence / prime invariant) + D-078
(confirm-at-pickup) + D-084 (route, don't patch in-tranche) + MP-R1-D8 (honest boundary) + the J-344
BOUNDED-gate criterion.*
