# M8.6 — Federation Stress: Design (clock seam + C4 gauge + four compound harnesses)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & scope

Design phase for **M8.6 — Federation stress** (D-069/D-071). Consumes the Phase-0 audit (`tasks/M8_6_FEDERATION_STRESS_AUDIT.md` v1.0). Locks: the `Clock` trait + `MockClock` API, the threading map across the six clock sites, the **C4 attempt-task gauge** (a deliberate surface beyond the clock fence), and the four compound harness specs **with the test-quality reframes** that came out of the design-review discussion. Output → runbook → Clair.

**Headline from the test-quality review (D-065):** the four compounds are not equally strong as first framed. Two needed reframing to stay *sensitive* (a test that would actually go red if the bug were present); two needed a real-coverage decision. All four are resolved below.

## 2. Locked decisions (this session)

| Lock | Decision |
|------|----------|
| **Q1 fork** | **Fork A** — minimal `Clock` trait (`now_utc` + `now_instant`) + `tokio::time` for timers. |
| **Q2 fence** | Seam = the six audit §3.2 clock sites **+ one C4 surface** (the attempt-task gauge, §4). The ~90 stamping reads stay out. |
| **Q3 mechanics** | `Arc<dyn Clock>`, sync, two methods, home `xgen-common`. `PendingBuffer` stays clock-free: `add()` gains `now: Instant`. |
| **Q4 harness** | Two families: two-Node in-process (C1, C8) + single-runtime/scheduler-direct (C4, C6). All MockClock-driven. |
| **C1 test reframe** | Assert **buffer↔drain consistency across reconnect**, NOT "no store dup" (store dedup is already defended — insensitive). |
| **C6 test reframe** | Assert **sequential cross-identity isolation**, NOT "parallel" (the buffer is mutex-guarded; a true race can't occur or be tested). |
| **C4 coverage** | **Real spawn-leak detection via a scoped attempt-task gauge** (not proxy invariants alone). B4 detached-spawn posture unchanged. |
| **C8 coverage** | **Strengthen** — bounded-channel-full / interleaving provocation, NOT a single-run liveness assertion. |

## 3. The clock seam

### 3.1 `Clock` trait (`xgen-common`)
```rust
pub trait Clock: Send + Sync {
    fn now_utc(&self) -> chrono::DateTime<chrono::Utc>;
    fn now_instant(&self) -> std::time::Instant;
}
```
Threaded as `Arc<dyn Clock>`. Sync (both reads are non-blocking). `xgen-common` already depends on chrono (`event_trace.rs`), so both return types are in scope. The `T` domain (tokio sleeps/timeouts) is **not** in the trait — handled by `tokio::time::pause/advance` in tests (Fork A).

### 3.2 `RealClock` + `MockClock` (single-cursor offset model, Joe-locked)
```rust
pub struct RealClock;
impl Clock for RealClock {
    fn now_utc(&self) -> DateTime<Utc> { Utc::now() }
    fn now_instant(&self) -> Instant { Instant::now() }
}

// test-only
pub struct MockClock {
    base_utc: DateTime<Utc>,
    base_instant: Instant,
    cursor: Mutex<Duration>,   // one knob
}
impl MockClock {
    pub fn advance(&self, d: Duration) { *self.cursor.lock() += d; }
}
impl Clock for MockClock {
    fn now_utc(&self) -> DateTime<Utc> { self.base_utc + chrono::Duration::from_std(*self.cursor.lock()).unwrap() }
    fn now_instant(&self) -> Instant { self.base_instant + *self.cursor.lock() }
}
```
One `cursor` drives both derived reads (Joe's "1 + offsets"). Harness helper advances the cursor **and** `tokio::time::advance(d)` together so W/M/T move in lockstep from one call:
```rust
async fn advance_all(clock: &MockClock, d: Duration) { clock.advance(d); tokio::time::advance(d).await; }
```
(`base_instant` is captured at MockClock construction; `std::time::Instant` has no arbitrary constructor, so the mock anchors at a real base + offsets — this is why the model is base+cursor, not absolute set.)

### 3.3 Threading map (the six clock sites)

| Site | Before | After |
|------|--------|-------|
| `reconnect.rs:150` scheduler_tick | `let now = Utc::now()` | `let now = clock.now_utc()` |
| `app.rs:2309` mark_lost | `mark_lost(&peer, Utc::now())` | `mark_lost(&peer, clock.now_utc())` |
| `app.rs:2183` mark_active stamp | `let now = Utc::now()` | `let now = clock.now_utc()` |
| `pending.rs:192` `received_at` | `Instant::now()` in `add()` | `add(…, now: Instant)` param; caller passes `clock.now_instant()` |
| `reconnect.rs:106` tick cadence | `tokio::time::sleep(60s)` | unchanged; tests use `tokio::time::pause/advance` |
| `handshake.rs:426` handshake timeout | `tokio::time::timeout(WAIT_TIMEOUT_SECS)` | unchanged; tests drive the tokio clock |

`Arc<dyn Clock>` is stored as a `NodeRuntime` field, constructed once (`RealClock` in production), and passed into `spawn_reconnect_scheduler` (threaded through `scheduler_tick` + `attempt_reconnect` alongside the existing `Arc` params). It reaches `pending.add()` via the NodeRuntime ingest path.

### 3.4 `PendingBuffer` stays clock-free (Q3 sub-lock)
The buffer is a pure DAG-logic component (xgen-core). It does **not** take a `Clock`. `drain_timed_out(now: Instant, …)` already takes `now`; we add the symmetric `add(…, now: Instant)` so the single production `Instant::now()` (pending.rs:192) moves to the caller. NodeRuntime (which holds the `Clock`) supplies both. Keeps xgen-core tokio-free and Clock-free.

## 4. The C4 attempt-task gauge (new surface beyond the clock fence)

**Why it exists.** A `tokio::spawn` leak cannot be counted from inside a test; proxy invariants (cursor/peer_records) stay green even if a task leaks → insensitive to the named M6 bug. The gauge makes C4 a *real* detector.

**Shape.** `Arc<AtomicUsize>` "outstanding attempt-phase tasks", threaded like the Clock seam:
- **inc** at `tokio::spawn` of an attempt (`scheduler_tick` spawn site).
- **dec** when the attempt phase resolves: on any failure return (connect/auth/handshake) **and** on the handshake-ACTIVE transition (the task ceases to be an *attempt* and becomes a long-lived session via `run_federation_session_post_handshake`).
- **Session tasks are NOT counted** — a successful reconnect's task is long-lived by design, so the gauge must scope to the attempt phase only, else a healthy session reads as a "leak".

**Invariant (the sensitive assertion).** After the MockClock + tokio clock advance past all connect/handshake timeouts against a non-responsive peer, the gauge **returns to 0**. A hung attempt task (no termination) keeps it > 0 → test red.

**Connect-timeout dependency (flag for the runbook).** The gauge only returns to 0 if every attempt path is bounded in time. The handshake has `tokio::time::timeout` (handshake.rs:426); the **TCP connect (`connect_url`) timeout is unconfirmed**. If absent, a black-hole peer leaks the attempt task — which is a genuine leak vector, and C4's gauge would *expose* it (gauge never returns to 0). Runbook step: confirm/establish a connect timeout; if it must be added, that is C4 doing its job, not scope creep.

**B4 unchanged.** Still one detached `tokio::spawn` per due peer per tick. The gauge instruments entry/exit; it does not restructure into a `JoinSet` (that would change the Joe-locked B4 posture — explicitly out of scope).

## 5. Compound harness specs (with reframes + sensitivity)

### C1 — F-10 unknown-signer during F-1b drop
- **Harness:** two in-process Nodes + controllable transport (drop mid-stream) + MockClock. Precedent: `phase9_drop_and_recover.rs`.
- **Sequence:** A pushes Bob's join (unknown signer) → B buffers HeldPending → connection drops mid-stream → F-1a reconnect re-streams → Bob's identity arrives via replication.
- **Assertion (reframed — sensitive):** after the full sequence, (a) PendingBuffer is empty (no orphan), (b) Bob is a member exactly once, (c) the join is in the store exactly once. **NOT** "no store dup" alone — store dedup is already defended (`insert` rejects `DuplicateEventId`, ingest swallows it), so that assertion is green-always. **Red condition:** orphaned buffer entry, or Bob absent, or drain failed to fire (the actual M3 bug).
- **Clock role:** advance the 30 s F-10 window across the drop without real waiting.

### C4 — Phase-5 reconnect scheduler under churn (×5)
- **Harness:** `scheduler_tick`-direct loop (it is already extracted "callable without sleeping the 60 s tick") + stub black-hole peer + MockClock + `tokio::time::advance` + the §4 gauge.
- **Sequence:** mark_lost → tick → ladder fires → drop/recover ×5 across the 15/30/60/120-min ladder (advanced, not real-clock).
- **Assertions:** (a) ladder resets to the 15-min initial delay on each handshake-ACTIVE; (b) `peer_records` stays consistent with `relationships`; (c) cursor invariants (§3.6 audit — post-restart aggressive-probe semantics honoured); **(d) the gauge returns to 0** after timeouts elapse (the real spawn-leak detector).
- **Red condition:** gauge stuck > 0 (leak), ladder mis-resets, or peer_records drift.

### C6 — identity-arrival drain isolation (reframed)
- **Harness:** single `NodeRuntime` + two-identity buffer + two sequential `drain_pending_by_identity` calls + MockClock.
- **Sequence:** two events buffered HeldPending on two different unknown signers (A, B); A's identity arrives, then B's, in close succession.
- **Assertion (reframed — sensitive):** `drain_pending_by_identity(A)` releases **only** A's entries; B's entry stays until B's drain. **NOT** "parallel" — the buffer is mutex-guarded so a true race cannot occur (or be tested); the realistic M9 bug is *logical* cross-identity contamination, which sequential close-succession arrival exercises. **Red condition:** A's drain wrongly releases B's entry (double-drain / contamination).

### C8 — bidirectional simultaneous push (strengthened)
- **Harness:** two in-process Nodes + bidirectional F-2 sessions + **provocation** (NOT single-run liveness).
- **Strengthening (Joe-locked):** drive the channel toward the blocking path deliberately — bounded-channel-full back-pressure + multiple send/receive interleavings — so the F-2a path is exercised under the condition the M8 deadlock would manifest, rather than a single happy-path run.
- **Assertion:** under provoked back-pressure across N interleavings, both events land at both Nodes and neither session hangs (bounded completion under the MockClock/tokio clock).
- **Honest framing (D-065):** this is a strong regression + stress test, not a formal deadlock-freedom proof; `try_send` non-blocking makes the bug improbable, and the strengthened harness raises the bar from "didn't happen once" to "didn't happen under provoked contention across interleavings." Exact interleaving count → runbook (§7).

## 6. Named test inventory (concrete targets for the runbook)

- `clock_mock_advances_utc_and_instant_in_lockstep` (MockClock unit)
- `clock_real_reads_are_monotonic_and_wall` (RealClock unit, light)
- `pending_add_takes_injected_instant_and_drain_uses_it` (buffer clock-free param)
- `c1_held_pending_drains_cleanly_across_f1a_restream` (buffer↔drain consistency)
- `c4_churn_x5_ladder_resets_and_peer_records_consistent`
- `c4_attempt_task_gauge_returns_to_zero_after_timeouts` (the leak detector)
- `c6_drain_by_identity_isolates_to_named_identity` (cross-identity isolation)
- `c8_bidirectional_push_completes_under_provoked_backpressure`

## 7. Open items for the runbook / design-review Joe-lock

1. **C4 connect-timeout** — confirm `connect_url` is time-bounded; if not, add one (C4 prerequisite, §4).
2. **C8 interleaving count + provocation mechanism** — exact bounded-channel capacity + interleaving permutations.
3. **Harness crate placement** — `xgen-node/src/tests/` (two-Node: C1, C8) vs `xgen-core` unit (C6 buffer) vs scheduler-direct (C4); lean per assertion target.
4. **Gauge production-visibility** — is the attempt-task gauge a pure test seam, or also surfaced as operator observability? (Lean: test seam only for M8.6; observability is a later concern.)

## 8. Promotion eval & state

- `Clock` trait → **promotion-watch** (arc-local now; promote to a D-NNN if it recurs as a cross-arc time-injection pattern, e.g. reused by M9's harness as expected).
- C4 gauge → arc-local; no DECISIONS change anticipated.
- Suite **1193/0/2** (no code in this phase).
- **Next-active:** design-review Joe-lock (§2 table + §7 items) → runbook (`tasks/M8_6_FEDERATION_STRESS_IMPL.md`) → Clair (seam + gauge first, then C1/C4/C6/C8 as one pack). Clair stands down until the runbook exists.

---

*End of M8.6 design. Status: ACTIVE. Fork A + single-cursor MockClock locked; C4 gains a real spawn-leak gauge (B4 unchanged); C8 strengthened; C1/C6 reframed for sensitivity. The seam + gauge are the only production touches; the four compounds test interactions on already-unit-tested components.*  
