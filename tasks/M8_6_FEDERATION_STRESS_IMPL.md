# M8.6 — Federation Stress: Implementation Runbook (Clair)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Framing

Clair-facing implementation runbook for **M8.6 — Federation stress**. Consumes the design (`tasks/M8_6_FEDERATION_STRESS_DESIGN.md` v1.0) + audit (`tasks/M8_6_FEDERATION_STRESS_AUDIT.md` v1.0). Reading order per Rule 0: CLAUDE.md PLAY → JOURNAL J-289 → design §2–§7 → audit §3–§6 → this runbook.

**What ships (one milestone, three commits):** the clock-injection seam (Fork A) + the C4 attempt-task gauge + a **reconnect connect-timeout** (C4 prerequisite, grounded below) + the four compound tests C1/C4/C6/C8 — built and run in-milestone.

**What does NOT ship:** any change to the B4 detached-spawn posture (gauge instruments entry/exit only); any operator-observability surface for the gauge (test-only); any change to `connect_url`'s other callers (the timeout wraps the reconnect call site only); any of the ~90 out-of-fence stamping reads.

**Grounded prerequisite (D-065).** `connect_url` (`xgen-core/src/transport/client.rs:32`) awaits `connect_async` **unbounded** — no connect timeout. Against a non-responsive peer the reconnect attempt task hangs forever = the C4 leak vector. This runbook adds a timeout at the `attempt_reconnect` call site (§3.4); without it the gauge can never return to 0 and C4 cannot pass.

## 2. Commit sequence + Joe-lock checkpoints

| Commit | Scope |
|--------|-------|
| **1 — seam + gauge + connect-timeout** | `Clock` trait + `RealClock`/`MockClock` (xgen-common); thread `Arc<dyn Clock>` through NodeRuntime + scheduler; `PendingBuffer::add(now: Instant)` param; connect-timeout wrap; `Arc<AtomicUsize>` gauge inc/dec; seam-level unit tests. Production behaviour change → its own commit. |
| **2 — four compounds** | C6 (core-unit), C4 (scheduler-direct), C1 + C8 (two-Node). The eight §6 named tests land here (minus the two seam units already in Commit 1). |
| **3 — milestone close** | Doc flips (design/runbook → COMPLETED; audit → COMPLETED), CLAUDE/ROADMAP/JOURNAL per D-074, promotion eval. |

**Split trigger:** if Commit 2 exceeds ~600 lines diff or any single compound file > ~400 lines, split per family boundary (C1/C8 two-Node vs C4 scheduler vs C6 unit) — surface at checkpoint #2.

**Joe-lock checkpoints:**
- **#1 — pre-Commit-1 + connect-timeout value.** Confirm the `Clock`/`MockClock` API as designed; **lock the connect-timeout value + placement** (recommendation: a new `CONNECT_TIMEOUT_SECS` const, call-site wrap in `attempt_reconnect`, value aligned with or below the handshake `WAIT_TIMEOUT_SECS` — Clair surfaces the actual `WAIT_TIMEOUT_SECS` value and proposes; Joe locks).
- **#2 — pre-Commit-2 named-test list + C8 interleaving count.** Clair surfaces the eight §6 test names verbatim + the **C8 bounded-channel capacity + interleaving permutation count**; Joe approves before the compounds are written.
- **#3 — post-Commit-2 verification.** Suite green across both feature sets; **C4 gauge-returns-to-0 demonstrably red without the connect-timeout** (prove the test is sensitive: temporarily revert the timeout, watch C4 fail, restore) — the sensitivity witness, recorded honestly.

## 3. Commit 1 — seam + gauge + connect-timeout

### 3.1 `Clock` trait + `RealClock` + `MockClock` (`xgen-common`)
Per design §3.1–§3.2. New module (e.g. `xgen-common/src/clock.rs`). `trait Clock: Send + Sync { fn now_utc() -> DateTime<Utc>; fn now_instant() -> Instant; }`. `RealClock` reads real time. `MockClock { base_utc, base_instant, cursor: Mutex<Duration> }` + `advance(d)`; both derived reads add the cursor. `MockClock` is `#[cfg(test)]` or a test-support module (Clair's call; if cross-crate test reuse is needed, a non-test-gated `mock` module behind a feature — surface at #1).

### 3.2 Thread `Arc<dyn Clock>` through NodeRuntime + scheduler
Store `clock: Arc<dyn Clock>` on `NodeRuntime`, constructed once (`RealClock`) at the production entry points. Pass into `spawn_reconnect_scheduler` → `scheduler_tick` → `attempt_reconnect` alongside the existing `Arc` params. Replace the three W-domain reads: `reconnect.rs:150`, `app.rs:2309`, `app.rs:2183` → `clock.now_utc()`.

### 3.3 `PendingBuffer::add(now: Instant)` (xgen-core, clock-free)
Add a `now: Instant` param to `add()` (symmetric with `drain_timed_out(now, …)`); store it as `received_at`. The buffer does **not** take a `Clock`. The NodeRuntime ingest caller passes `clock.now_instant()`. Update the existing `add()` call sites + the buffer's own unit-test fixtures to pass an explicit `Instant` (the buffer tests already construct `Instant::now() + Duration` — they pass `Instant::now()` at add).

### 3.4 Connect-timeout (C4 prerequisite)
Wrap the `connect_url(&peer_url)` call in `attempt_reconnect` (`reconnect.rs`) with `tokio::time::timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS), …)`; on elapse, log + return (attempt fails cleanly, gauge decrements). New const `CONNECT_TIMEOUT_SECS` (value locked at checkpoint #1). **Do not** modify `connect_url` itself (other callers unaffected). This makes every attempt path time-bounded.

### 3.5 Gauge (`Arc<AtomicUsize>`)
Per design §4. `Arc<AtomicUsize>` "outstanding attempt-phase tasks", threaded like the clock. **inc** at the `tokio::spawn` in `scheduler_tick`. **dec** on every attempt-resolution path in `attempt_reconnect`: each early `return` (connect-timeout, connect fail, auth fail, handshake fail, peer-mismatch abort) **and** at the handshake-ACTIVE transition (just before `run_federation_session_post_handshake` — the task ceases to be an attempt). Use a guard type (Drop-based dec) to avoid missing a path. Session tasks are not counted.

### 3.6 Seam unit tests (land in Commit 1)
- `clock_mock_advances_utc_and_instant_in_lockstep`
- `clock_real_reads_are_monotonic_and_wall`
- `pending_add_takes_injected_instant_and_drain_uses_it`

### 3.7 Verification (Commit 1)
`cargo build --workspace --all-targets` 0; `cargo clippy --all-targets --all-features -- -D warnings` clean both feature sets; full suite green (no regression from the threading). The seam is behaviour-neutral in production (RealClock == today's reads); the only production behaviour change is the connect-timeout (intended).

## 4. Commit 2 — the four compounds

Per design §5. Each test's **red condition** is the sensitivity contract — it MUST be able to fail.

- **C6 — `xgen-core/src/dag/pending.rs` `#[cfg(test)]`** — `c6_drain_by_identity_isolates_to_named_identity`. Two entries on signers A,B; `resolve_identity(A)` releases only A's; B stays. Red: A's drain releases B's entry.
- **C4 — `xgen-node/src/tests/m8_6_c4_reconnect_churn.rs` (NEW)** — `c4_churn_x5_ladder_resets_and_peer_records_consistent` + `c4_attempt_task_gauge_returns_to_zero_after_timeouts`. `scheduler_tick`-direct + stub black-hole peer + MockClock + `tokio::time::advance` + gauge. Assert ladder reset on ACTIVE, peer_records/relationships consistency, cursor invariants, **gauge → 0** after timeouts. Red: gauge stuck > 0 (leak), ladder mis-reset, peer_records drift.
- **C1 — `xgen-node/src/tests/m8_6_c1_held_pending_reconnect.rs` (NEW)** — `c1_held_pending_drains_cleanly_across_f1a_restream`. Two in-process Nodes (beside `phase9_drop_and_recover.rs` scaffolding) + controllable drop + MockClock. Assert buffer empty (no orphan), Bob a member once, join in store once. Red: orphan, Bob absent, or drain didn't fire.
- **C8 — `xgen-node/src/tests/m8_6_c8_bidirectional_push.rs` (NEW)** — `c8_bidirectional_push_completes_under_provoked_backpressure`. Two Nodes + bidirectional F-2 + **bounded-channel-full + N interleavings** (capacity + N locked at checkpoint #2). Assert both events land both sides, no hang. Honest framing in the test doc-comment: strong stress test, not a deadlock-freedom proof.

## 5. Commit 3 — milestone close

- `tasks/M8_6_FEDERATION_STRESS_IMPL.md` Status ACTIVE → COMPLETED; `..._DESIGN.md` → COMPLETED; `..._AUDIT.md` → COMPLETED.
- `tasks/FEDERATION_STRESS_FOLLOWON.md` Status PENDING → COMPLETED (its scope is now shipped) or a close note pointing here.
- CLAUDE.md PLAY flip → M8.6 CLOSED, next-active = M8.7 (D3 MLS). JOURNAL J-### close entry. ROADMAP M8.6 🟢→✅ + version bump.
- Promotion eval: `Clock` trait — promote to a D-NNN **if** M9's harness reuses it (the likely second instance); else stays arc-local promotion-watch. Gauge + connect-timeout arc-local.

## 6. Verification rigour (milestone-bearing boundary)

5 isolated runs (`cargo clean` between) + 3 consecutive workspace runs = 8 green minimum at the Commit 2 / Commit 3 boundary. `clippy -D warnings` clean both feature sets. The **C4 sensitivity witness** (checkpoint #3): demonstrate C4 red without the connect-timeout, then restore — proves the test is not green-always. Record the witness in the JOURNAL close entry (D-065).

## 7. Definition of Done

- [ ] `Clock` trait + `RealClock` + `MockClock` shipped (xgen-common); single-cursor offset model; `advance_all` lockstep helper.
- [ ] Six clock sites threaded; `PendingBuffer::add(now: Instant)`; buffer stays clock-free.
- [ ] Connect-timeout wrapped at `attempt_reconnect` (value Joe-locked); `connect_url` untouched.
- [ ] Gauge inc/dec across all attempt-resolution paths (Drop-guard); session tasks uncounted; B4 unchanged.
- [ ] Eight §6 tests present and passing; each compound's red condition verified reachable.
- [ ] C4 sensitivity witness recorded.
- [ ] Suite green both feature sets; clippy `-D warnings` clean; build all-targets 0.
- [ ] Canonical docs flipped per D-074 (design/runbook/audit COMPLETED; CLAUDE/ROADMAP/JOURNAL).

*(No "commit pushed" line — the COMPLETED header is the shipped signal; Joe pushes.)*

## 8. Discipline notes

- **One writer per file per atom**; write each file to disk before the next; `Filesystem:*` for E:\, never the sandbox.
- **D-074 atomic:** each commit's canonical-record changes ship with the work they record.
- **D-065:** the connect-timeout finding + the C4 sensitivity witness are surfaced, not papered over. C8 is labelled a stress test, not a proof, in its own doc-comment.
- **Seam reuse:** the `Clock`/`MockClock` are built so M9's harness can reuse them — keep them general (no federation-specific assumptions in the trait).

## 9. Cross-references

- `tasks/M8_6_FEDERATION_STRESS_DESIGN.md` v1.0 (the locked design — §2 decisions, §3 seam, §4 gauge, §5 harnesses, §6 named tests, §7 open items resolved here).
- `tasks/M8_6_FEDERATION_STRESS_AUDIT.md` v1.0 (Phase-0 grounding — site map, domains, fence).
- `tasks/FEDERATION_STRESS_FOLLOWON.md` (the original deferred-compound stub).
- `xgen-core/src/dag/pending.rs` (buffer), `xgen-node/src/reconnect.rs` (scheduler/ladder), `xgen-core/src/transport/client.rs:32` (`connect_url`), `xgen-node/src/tests/phase9_drop_and_recover.rs` (two-Node precedent).
- DECISIONS D-065 / D-069 / D-071 / D-074.

---

*End of M8.6 implementation runbook. Status: ACTIVE. Clair pickup: checkpoint #1 (Clock API + connect-timeout value) → Commit 1 (seam + gauge + connect-timeout) → checkpoint #2 (test list + C8 interleavings) → Commit 2 (four compounds) → checkpoint #3 (sensitivity witness) → Commit 3 (close). Clair stands down until Joe approves this runbook.*  
