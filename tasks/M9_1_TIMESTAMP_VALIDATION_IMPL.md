# M9.1 — Event Timestamp-Bound Validation (F1 / gap G6) — Implementation Runbook

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

Execute the J-309 Joe-LOCKED design (`tasks/M9_1_TIMESTAMP_VALIDATION_DESIGN.md`, M9.1-D1…D5):
add a **future-skew timestamp admission bound** to the live F-4 validation core `validate_event`,
closing F1 / gap G6 (the M9 injector MP-A-15 silently admitted). **Single commit** (D-074),
crypto-agnostic, admission-only. Clair may pick up.

**Locks recap:** D1 future-skew ceiling only · D2 both origins (convergence-safe by catch-up
monotonicity) · D3 in `validate_event` "Step 8.5", threaded `now`, never feeds ordering (D-076) ·
D4 `MAX_FUTURE_SKEW` = 5 min, named · D5 proof-gated.

---

## 2. Grounded surfaces (so Clair does not re-discover)

All in **`xgen-core`** (the audit's `xgen-node`/`validate_steps_8_13` citation is wrong — corrected
at J-309).

- **Target fn:** `validate_event` — `xgen-core/src/message/exchange.rs:466`. Pure (no clock).
  Current signature: `validate_event(event: &Event, space: Option<&SpaceState>, id_registry:
  &IdentityRegistry, store: &dyn EventStore, fed_add_via_federation: bool) -> ValidationOutcome`.
- **Event field:** `Event.timestamp: String` (RFC3339, e.g. `2026-04-30T10:00:00.000Z`). No step
  parses it today.
- **Call sites — exactly three** (so the new `now` param is a small touch):
  1. the def (exchange.rs:466);
  2. **live caller** `dispatch_event` — `xgen-core/src/node/runtime.rs:1060`; `self.clock.now_utc()`
     (D-090) and `origin`/`peer_node_id` are already in scope here;
  3. **one test** — exchange.rs:1255 (`node_eject_from_non_home_node_rejected_with_3043`); its event
     is `Utc::now()`-dated.
  - The many other exchange.rs tests call the **deprecated** `validate_steps_8_13` — **not**
    `validate_event` — and are unaffected.
- **Wire code (collision-checked):** 30xx = the admission family. Taken: 3030, 3040, 3041, 3042,
  3043, 3044, 3045. **3046 is free** → `event_timestamp_out_of_bounds` (sits right after the
  3044/3045 invite-admission gates — same clock/expiry-admission family).
- **MP-A-15 direction:** the injector's `build_clock_skew_event` stamps **`2099-01-01T00:00:00.000Z`**
  (`xgen-mptest/src/injector.rs`) — **far-future**. So D1 closes it; assert rejection. (No
  out-of-scope branch needed — the favorable case.)
- **MockClock:** already a `xgen-core` `[dev-dependencies]` (INV-EXP) for the dispatch-level repros.

---

## 3. Commit plan (single commit)

### Step 1 — constant + error variant + wire code (`exchange.rs`)
- Add a module const: `pub const MAX_FUTURE_SKEW_SECS: i64 = 300;` (5 min — named, D4). Doc-comment:
  "≫ realistic NTP skew, ≪ a useful future-dating window; the bound, not tuned to pass a test."
- Add `ExchangeError::TimestampOutOfBounds` (carries a `String` for the human reason —
  unparseable vs over-ceiling). `#[error("event timestamp out of bounds: {0}")]`.
- Add the `to_wire_code` arm: `Self::TimestampOutOfBounds(_) => Some((3046,
  "event_timestamp_out_of_bounds"))`.

### Step 2 — `now` param + "Step 8.5" (`validate_event`, exchange.rs)
- Add a parameter `now: DateTime<Utc>` (last positional, after `fed_add_via_federation`). `chrono`
  is already imported in this module.
- Insert **Step 8.5** immediately after Step 8 (event_id hash), **before** Step 10 / Step 9 — a
  pure check on the event's own field, cheapest, and a skewed event must be rejected outright, not
  HeldPending-buffered:
  ```
  // Step 8.5 — timestamp future-skew bound (M9.1-D1/D3). Admission-only; the
  // accepted event's timestamp is NEVER read by state_key_for_event / the
  // resolver / ordering (D-076 — ordering is wire-order). Runs on BOTH origins
  // (M9.1-D2): a future ceiling is monotone under catch-up, so it is convergence-
  // safe everywhere (design §4). Far-past is legitimate (catch-up + replay).
  let ts = match chrono::DateTime::parse_from_rfc3339(&event.timestamp) {
      Ok(t) => t.with_timezone(&Utc),
      Err(_) => return ValidationOutcome::Rejected(
          ExchangeError::TimestampOutOfBounds(format!(
              "unparseable RFC3339 timestamp: {}", event.timestamp))),
  };
  if ts > now + chrono::Duration::seconds(MAX_FUTURE_SKEW_SECS) {
      return ValidationOutcome::Rejected(ExchangeError::TimestampOutOfBounds(
          format!("timestamp {} exceeds now + {}s skew ceiling",
                  event.timestamp, MAX_FUTURE_SKEW_SECS)));
  }
  ```
- **Do NOT** add any lower bound (no past floor — D1).

### Step 3 — caller wiring
- **runtime.rs:1060** — pass `self.clock.now_utc()` as the new `now` arg (it is already computed in
  `dispatch_event`; reuse the existing local if present, else read it).
- **exchange.rs:1255** test — pass `Utc::now()` (the eject event is `Utc::now()`-dated → ≈ now →
  passes Step 8.5; the test still asserts `NodeEjectAuthority` reached after step 12).

### Step 4 — tests (§4).

### Step 5 — checkpoint (§5) → doc-only close.

---

## 4. Tests (mirrors design §5 (a)–(d))

**Unit, in `exchange.rs` tests (fixed `now`, fully deterministic — pass any `now`):**
- `ts8_5_now_and_under_ceiling_accept` — timestamp `= now` and `= now + 4 min` → not
  `TimestampOutOfBounds` (reaches a later step / `Validated`).
- `ts8_5_over_ceiling_rejected_3046` — timestamp `= now + 10 min` →
  `Rejected(TimestampOutOfBounds)`; assert `to_wire_code() == Some((3046, …))`.
- `ts8_5_far_past_accepted` — timestamp `= now − 30 days` → **not** rejected (catch-up/replay
  legitimacy).
- `ts8_5_unparseable_rejected` — `timestamp = "not-a-date"` → `Rejected(TimestampOutOfBounds)`.

**Dispatch-level, in the runtime.rs test module (MockClock):**
- `m9_1_honest_skew_same_verdict` — two `NodeRuntime`s, clocks differing by δ ≈ 2 s; ingest the
  **same** event with `timestamp = base + 10 min` → **both** Rejected; a `timestamp = base` event →
  **both** accepted. (Same-verdict under honest skew — the margin protects.)
- `m9_1_catchup_leniency` — event with legitimate `timestamp ≈ base` accepted live at A
  (`A.now ≈ base`); the **same** event ingested at B with B's MockClock advanced (`B.now =
  base + 2 days`) → **B accepts** (now moved past it — the monotonicity property that makes
  both-origins safe).
- `m9_1_sensitivity_witness` (the D-065 witness) — with Step 8.5 present, the `+10 min` /
  2099-class event is rejected and **absent** from the applied log; document that reverting the
  Step 8.5 arm flips it to admitted (RED). Recorded at the checkpoint, file net-unchanged.

---

## 5. Checkpoint (one, light — surface, don't work around: D-065/D-084)

1. **D-076 non-interference:** confirm an **accepted** event flows through unchanged — Step 8.5 only
   rejects-or-passes; `state_key_for_event` / the resolver / ordering never read `event.timestamp`.
   (Grep-confirm no new timestamp read outside Step 8.5.)
2. **Backward-coherence (D-077):** run the **full** workspace suite. If any existing test ingests a
   `> 5 min`-future event through `dispatch_event` expecting **acceptance**, that is a real
   interaction → **surface it as a finding**, do not widen the ceiling to pass. (Expected: none —
   fixtures are `Utc::now()` or past-dated; the only future-dated event is the adversarial 2099
   injector.)
3. **MP-A-15 closed:** the 2099-class event is rejected (3046) and absent from the applied log.
4. Suite green; build 0; clippy clean (default **and** `--all-features`).

---

## 6. Definition of Done

- Step 8.5 in `validate_event`; `MAX_FUTURE_SKEW_SECS` const; `TimestampOutOfBounds` + 3046 arm.
- Both callers wired (runtime.rs `self.clock.now_utc()`; the one test `Utc::now()`).
- The §4 tests added and green; sensitivity witness recorded at the checkpoint.
- `cargo build --workspace --all-targets` 0; `cargo clippy --workspace --lib --tests -- -D warnings`
  clean on default + `--all-features`; `cargo test --workspace` green (1262 + new).
- Docs at close (doc-only): AUDIT + DESIGN + this IMPL → COMPLETED; **`tasks/M9_findings.md` F1 →
  marked resolved** (closed by M9.1); JOURNAL + ROADMAP (v2.98→v2.99, M9.1 🟢→⚫/✅) + CLAUDE PLAY.
  Optional courtesy: refresh the now-stale "would be accepted (gap-G6)" doc-comment in
  `xgen-mptest/src/injector.rs` (test-crate doc only — Clair's call, low priority).
- **No "commit pushed" line** (Status COMPLETED is the signal).

---

## 7. Scope guards + honest boundary (D-065)

- **In scope:** the future-skew admission bound only. **Not** touched: ordering/resolution (D-076),
  the invite-expiry gates (INV-EXP 3044/3045), the deprecated `validate_steps_8_13`, any sibling
  finding (F2/F3/F4 → M9.2).
- **Closes:** future-dated events from **any** origin (local liar / skewed local clock **and** the
  federated injector — MP-A-15 is 2099, caught).
- **Does NOT close (named):** **past**-dated events — indistinguishable from legitimate catch-up; no
  wall-clock gate can reject them without breaking catch-up. That is the deferred causal-monotonicity
  territory (D1), a candidate future arc, not M9.1.

---

## 8. Next-active

**Clair** — Step 1 → … → Step 5 → doc-only close (one commit, D-074). Then **M9.2** (F2/F3/F4 fenced
seams; its own D-071 Phase-0) → Multiparty-tests.

**Entry point for Clair (Rule 0):** CLAUDE PLAY → JOURNAL J-309 →
`tasks/M9_1_TIMESTAMP_VALIDATION_DESIGN.md` §3+§4 → this runbook §2+§3 → `tasks/M9_findings.md` (F1).

Per D-065 + D-069 + D-071 + D-074 + D-076 + D-077 + D-090.
