# MP-F3 — duplicate re-fan-out — IMPLEMENTATION RUNBOOK

> **Status**: COMPLETED
> Version: 1.1
> Date: Jun 2026
> **Last updated**: 2026-06-08
> Language: English
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

---

## 1. What this is

Executes the J-325 Joe-LOCKED design (`tasks/MP_F3_DEDUP_REFANOUT_DESIGN.md`, F3-D1..D5). The
second production-crate fix-arc of the loop-to-green (MP-R1-D10), after MP-F2. A re-submitted
duplicate `event_id` is applied once but re-broadcast; the fix adds a `DispatchOutcome::Duplicate`
outcome (dedup-at-dispatch via `store.contains`) that maps to no-fan-out + an idempotent ack.

Production-arc discipline: build 0 + clippy clean on **default AND `--features harness-control`**;
the D5 side-effect-skip audit is a **build-time obligation** (confirm each of the five re-run
effects is genuinely idempotent/already-fired as implemented — don't just trust the design); the
**285-convergence net is the D-076 discharge** and MUST stay green.

---

## 2. Commit shape

Single atomic commit (small, cohesive — variant + dedup check + match arms + tests + MP-A-09
flip), same shape as MP-F2. **Commit order (corrects the MP-F2 doc-first inversion):** hand Joe
the code-staging command FIRST; the doc-bridge (Chat seat) is written after, so history reads
code→record. The arc docs (`*_AUDIT.md`, `*_DESIGN.md`, this `*_IMPL.md` → COMPLETED) ride the
code commit per the writer-per-file rule.

---

## 3. Steps

### S1 — `DispatchOutcome::Duplicate` variant (F3-D2)

`xgen-core/src/node/runtime.rs` — add unit variant `Duplicate` to `DispatchOutcome` (after
`HeldPending`, before `Rejected`). Doc-comment kept **with the enum body** (MP-F2 lesson — avoid
`doc_lazy_continuation`). Sibling framing: "do-not-fan-out, not-an-error" (like `HeldPending`).

### S2 — dedup check in `dispatch_event` (F3-D1)

`xgen-core/src/node/runtime.rs` — after Step 3 `ensure_store(&space_id)` succeeds
(≈1080-1085), before the `validate_event` block (≈1101):

```rust
// MP-F3-D1 — dedup-at-dispatch. The store knows an already-accepted event
// (insert returns DuplicateEventId, swallowed at ingest). Surface it here as
// Duplicate → process_inbound maps to no-fan-out (kills local + federation
// re-broadcast) + an idempotent ack. event_id is the content hash, so
// contains() is an exact-duplicate test (no false positive). event_id == None
// skips (validate_event rejects an idless event downstream anyway).
if let Some(eid) = event.event_id.as_ref() {
    if self.stores.get(&space_id).map(|s| s.contains(eid)).unwrap_or(false) {
        return DispatchOutcome::Duplicate;
    }
}
```

(`EventStore::contains` takes `&EventXgid`; `event.event_id` is `Option<EventXgid>`.)

### S3 — drain re-dispatch arms ×3 (F3-D2)

`xgen-core/src/node/runtime.rs` (1623, 1707, 1791) — fold `Duplicate` into the existing no-op
arm: `DispatchOutcome::HeldPending | DispatchOutcome::Rejected(_) | DispatchOutcome::Duplicate => {}`.
(A drained event was buffered, not stored, so it ingests fresh → `Accepted`; `Duplicate` here is
a safe no-op, not expected.)

### S4 — `process_inbound` `Duplicate` arm (F3-D3 + F3-D4)

`xgen-node/src/app.rs` — in `match outcome` (2593), add after the `HeldPending` arm:

```rust
DispatchOutcome::Duplicate => {
    // MP-F3-D3 — idempotent ack (the event WAS accepted at first ingest;
    // acking stops a retrying LocallySubmitted client). No persist (already
    // on disk), no new_joiner. FanoutRequest::none() suppresses BOTH local
    // fan-out and federation push (F3-D4 — both key off req.event).
    tracing::debug!(
        space_id = %space_id_for_persist,
        event_id = %event_id,
        event_type = %event_type_str,
        "duplicate event — applied once already; ack re-sent, fan-out suppressed"
    );
    if let Some(sig) = accept_signal(
        origin,
        &event_id,
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
    ) {
        let _ = conn.send_transport(&sig).await;
    }
    FanoutRequest::none()
}
```

(`accept_signal` already gates on `origin == LocallySubmitted`, so a federation-received
duplicate sends no ack — same rule as `Accepted`.)

### S5 — remaining production exhaustive arms (F3-D2, compiler-caught)

`xgen-node/src/admin_ops.rs` (4013, 6030) + `xgen-node/src/migration_driver.rs` (264) — trivial
`Duplicate` arms (treat as already-applied / no-op sibling of the success path). Let the compiler
list them; handle each honestly.

### S6 — test-module exhaustive arms (compiler-caught)

`xgen-node/src/tests/federation_relationship_integration.rs:294`,
`xgen-node/src/tests/m8_s7_privilege.rs:98`, and any `DispatchOutcome` exhaustive `match` in the
`runtime.rs` test module — trivial `Duplicate` arms. (`app.rs:1447` is a `match` on `Inbound`,
NOT `DispatchOutcome` — out of scope.) Surface anything the compiler flags beyond this list.

### S7 — new units (F3-D6/D7/D8)

- **F3-D6 (xgen-core, runtime.rs test module):** submit E → `Accepted`; submit E again →
  `Duplicate`; assert `SpaceState` + store length byte-identical after the second submit.
- **F3-D7 (xgen-core):** side-effect-skip safety — the duplicate's apply is a no-op (state
  identical) and no pending event is spuriously drained. **This is the D5 build-time obligation
  encoded as a test.**
- **F3-D8 (xgen-node):** the `Duplicate` arm → `FanoutRequest::none()` (no local fan-out, no
  federation push) AND `EventAccepted` sent for `LocallySubmitted` / not for
  `ReceivedViaFederation`.

### S8 — MP-A-09 flip (the pinned proof obligation, F3-D / §5)

`xgen-mptest/tests/mp_r1_c7.rs:154` (`mp_a_09_duplicate_dedup_holds`) — flip from tolerant
`assert n >= 1` (re-emit tolerated + measurement-gap note) to **"the duplicate is fanned out
exactly once"**: the duplicate `event_id` appears exactly once per recipient transcript. Rewrite
the doc-comment to drop the re-emit-tolerance / measurement-gap prose. Run `--ignored` under
`--features harness-control` and verify end-to-end.

### S9 — D5 build-time confirmation (do, don't assume)

As implemented, re-confirm each of the five re-run effects the early-return skips is genuinely
idempotent/already-fired (design §F3-D5): (1) `graph.add_event` set-semantics; (2) `store.append`
dup-Err swallowed; (3) `apply_event`/`derive_resolved` pure-function-of-log; (4) `record_key_package`
keyed-overwrite; (5) drain hooks keyed on the event's own id. Record the confirmation in the
close note. If any turns out NOT idempotent, STOP and surface (Rule 3) — the early-return would
then lose a needed effect.

---

## 4. Definition of Done

- [ ] S1–S8 implemented per F3-D1..D5.
- [ ] `cargo build --workspace --all-targets` — 0 errors (default).
- [ ] `cargo build --workspace --all-targets --features harness-control` — 0 errors.
- [ ] `cargo clippy --workspace --lib --tests` — clean (default).
- [ ] `cargo clippy --workspace --lib --tests --features harness-control` — clean.
- [ ] xgen-core fast suite green; xgen-node fast suite green; **285-convergence integration green
      (D-076 discharge)**.
- [ ] F3-D6/D7/D8 units pass.
- [ ] **MP-A-09 passes the exactly-once assertion** (`--ignored`, `--features harness-control`),
      verified end-to-end.
- [ ] S9 — five-effect idempotency re-confirmed as-built (recorded in the close note).
- [ ] Arc docs → COMPLETED (AUDIT/DESIGN/this IMPL).
- [ ] **No "commit pushed" line** — hand Joe the code-staging command first (code→record order).

---

## 4a. As-built (J-326) — SHIPPED

- **F3-D1 placement corrected** (design §8a): the dedup gate moved from "before `validate_event`"
  to "**after `validate_event` passes, before the Step 4 semantic gates**" — `phase9_compound_c5`
  caught that a signature-forgery reusing a stored `event_id` (id excludes the signature) was
  mis-classified `Duplicate` instead of `Rejected(step 12)`. Validation must precede dedup so a
  forgery is rejected and only a confirmed-valid already-stored event is a true duplicate.
  Surfaced per D-065, not a silent override.
- **S9 five-effect idempotency re-confirmed as-built** (the gate skips Step 4 + `ingest_event` +
  drains for a duplicate): (1) `graph.add_event` set-semantics no-op; (2) `store.append`
  `DuplicateEventId` swallowed; (3) `apply_event`/`derive_resolved` pure-function-of-log —
  **F3-D6 asserts SpaceState byte-identical**; (4) `record_key_package` keyed-overwrite by
  (identity, device); (5) drains keyed on the event's own id, already fired at first ingest —
  **F3-D7 asserts no second drain + empty buffer**. Plus the new skip of the Step 4 semantic
  gates: correct because an already-stored event is already fully accepted (gate drift must not
  un-accept it). The early-return loses nothing.
- **Verification (actual output):** `cargo build --workspace --all-targets` 0 (default) +
  `--features harness-control` 0; `cargo clippy --workspace --lib --tests` clean both feature
  sets; xgen-core lib **669/0** (+2: F3-D6/D7), xgen-node lib **286/0** (convergence/M8 net
  green — one workspace-parallelism flake on a re-run, green isolated + on retry, the documented
  `reconnect`/`drop_and_recover` flake, not a regression), xgen-mptest lib **72/0**.
  **MP-A-09 (`mp_a_09_duplicate_fanned_out_exactly_once`, `--ignored --features harness-control`)
  PASS end-to-end** — the duplicate is fanned out exactly once (max-per-transcript == 1).
- **Commit order:** code-staging command handed to Joe first; doc-bridge (Chat seat) after.

---

## 5. Entry point (Rule 0)

CLAUDE PLAY → JOURNAL J-325 → `tasks/MP_findings.md` MP-F3 → design `tasks/MP_F3_DEDUP_REFANOUT_DESIGN.md`
§3 (locks) + §5 (proof obligation) → this runbook §3.

---

*Per D-065 + D-067 + D-069 + D-071 + D-076.*
