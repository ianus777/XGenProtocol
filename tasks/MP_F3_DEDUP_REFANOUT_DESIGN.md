# MP-F3 — duplicate re-fan-out — DESIGN

> **Status**: ACTIVE
> Version: 1.0
> Date: Jun 2026
> **Last updated**: 2026-06-08
> Language: English
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

---

## 1. What this is

MP-F3 (routed at MP-R1 C7, J-321; `tasks/MP_findings.md`) is the **second production-crate**
fix-arc of the loop-to-green (MP-R1-D10), after MP-F2 (J-324). The defect: a re-submitted
duplicate `event_id` is **applied once** (DAG/store/disk dedup holds) but **re-broadcast** —
`dispatch_event` returns `Accepted` for the duplicate, so `process_inbound` fans it out again
and members/observers receive it twice. Mild amplification, no state corruption.

The fix is **dedup-at-dispatch**: `dispatch_event` already has the fact it needs (the store
knows the event is a duplicate via `EventStore::contains`); surface it as a new
`DispatchOutcome::Duplicate` outcome that `process_inbound` maps to no-fan-out, while still
sending the originator a truthful idempotent ack.

Phase-0 audit: `tasks/MP_F3_DEDUP_REFANOUT_AUDIT.md` (ACTIVE). Forks F3-A..F3-D Joe-locked
(this session) exactly as the audit recommended, plus one pinned proof obligation (§5).

---

## 2. Grounding (against live `main`, from the Phase-0 audit)

- **Fan-out chokepoint:** `process_inbound` (`xgen-node/src/app.rs:2434`) calls
  `rt.dispatch_event(...)` (2582), then `match outcome` (2593): `Accepted` →
  `FanoutRequest { event: Some(event), new_joiner }` (2687) = **fans out**; `HeldPending` /
  `Rejected` → `FanoutRequest::none()` = no fan-out. The returned request drives **both**
  `apply_fanout` (local members + `.events` observers, `xgen-node/src/fanout.rs:213`) **and**
  `apply_federation_push` (federation peers); both early-return when `req.event` is `None`
  (fanout.rs:219-222). So `FanoutRequest::none()` suppresses both deliveries in one move.
- **No dedup gate:** `dispatch_event` (`xgen-core/src/node/runtime.rs:919`) passes a duplicate
  through `validate_event` (no dedup check there) and the semantic gates, re-runs
  `ingest_event`, and returns `Accepted` (1538).
- **The fact is already known + dropped:** `InMemoryEventStore::insert` returns
  `Err(StoreError::DuplicateEventId)` on a duplicate id (`xgen-core/src/dag/store.rs:250-258`),
  but `ingest_event` discards it (`let _ = store.append(...)`, runtime.rs:629). The clean
  read primitive is `EventStore::contains(&EventXgid)` (store.rs:89/315). `event_id` is the
  SHA-256 canonical-content hash, so "same event_id" ⟺ a genuine duplicate (no false-positive).

---

## 3. Locked decisions (F3-D1..D5)

### F3-D1 (F3-A) — dedup check location: `dispatch_event`, post-`ensure_store`

Insert the dedup check in `dispatch_event` after Step 3 `ensure_store` succeeds
(runtime.rs:1080-1085), **before** `validate_event` (1101). The store handle is guaranteed to
exist there, the effective `space_id` is already resolved (`space_id_of`, 943), and this is
the single function that produces the outcome driving fan-out (no-drift, D-067). Skipping a
known duplicate before `validate_event` is also a small free win (no re-validation).

```text
// after ensure_store(&space_id) succeeds, before validate_event:
if let Some(eid) = event.event_id.as_ref() {
    if self.stores.get(&space_id).map(|s| s.contains(eid)).unwrap_or(false) {
        return DispatchOutcome::Duplicate;
    }
}
```

`event_id == None` ⇒ skip the dedup check (an unsigned/idless event is rejected by
`validate_event` downstream anyway; dedup keys on the content hash, which an idless event
lacks). Rejected alternative A2 (dedup in `process_inbound`) duplicates `space_id` resolution
and splits the logic across the crate boundary.

### F3-D2 (F3-B) — outcome shape: new `DispatchOutcome::Duplicate` variant

Add a unit variant `DispatchOutcome::Duplicate` (sibling to `HeldPending` — a
"do-not-fan-out, not-an-error" outcome). Chosen over reusing `Accepted { ..., novel: bool }`:

- Adding a *variant* (not changing an existing one's payload) leaves every
  `matches!(_, DispatchOutcome::Accepted { .. })` and `matches!(_, Rejected(_))` wildcard
  **intact** — the inverse of the MP-F2 shape call (MP-F2 *changed* `Rejected`'s payload, so a
  1-tuple minimised wildcard churn; MP-F3 *adds* a variant, so wildcards survive and a new
  variant is the lower-churn choice).
- Only genuinely-exhaustive `match` blocks need a `Duplicate` arm. **7 production sites**, all
  compiler-caught, all trivial:
  - `xgen-core/src/node/runtime.rs` drain re-dispatch ×3 (1623, 1707, 1791) — fold into the
    no-op arm: `DispatchOutcome::HeldPending | DispatchOutcome::Rejected(_) | DispatchOutcome::Duplicate => {}`.
    (A drained event was buffered, not stored, so it ingests fresh → `Accepted`; `Duplicate`
    is not expected on this path but is handled as a safe no-op.)
  - `xgen-node/src/app.rs:2593` (process_inbound) — **the load-bearing arm** (F3-D3 below).
  - `xgen-node/src/admin_ops.rs:4013, 6030` — admin-submitted dispatch; `Duplicate` ⇒
    treat as already-applied (the admin op's effect is already present), no error.
  - `xgen-node/src/migration_driver.rs:264` — migration dispatch; `Duplicate` arm, no-op
    sibling of the success path.
  - Plus the test-module exhaustive matches (`federation_relationship_integration.rs:294`,
    `m8_s7_privilege.rs:98`, and any in the `runtime.rs` test module) — compiler-caught;
    each gets a trivial `Duplicate` arm. (`app.rs:1447` `match first` is on `Inbound`, **not**
    `DispatchOutcome` — not in scope.)
- More honest than `Accepted{novel}`: a duplicate is neither freshly-accepted nor rejected,
  and `matches!(outcome, Accepted { .. })` staying false for a duplicate keeps the ~140 test
  assertions that use that matcher meaningful.

### F3-D3 (F3-C) — `process_inbound` mapping: idempotent ack, never fan out

In `process_inbound`'s `match outcome` (app.rs:2593), the new arm:

```text
DispatchOutcome::Duplicate => {
    // truthful idempotent ack — the event WAS accepted (at first ingest);
    // acking stops a retrying LocallySubmitted client. Fan-out is the only
    // thing that must not repeat.
    if let Some(sig) = accept_signal(origin, &event_id, Utc::now()...) {
        let _ = conn.send_transport(&sig).await;
    }
    FanoutRequest::none()   // suppresses local fan-out AND federation push (F3-D4)
}
```

`accept_signal` already gates on `origin == LocallySubmitted` (app.rs:2680), so a
federation-received duplicate sends no ack (no originator to ack) — same rule as `Accepted`.
**No persist** on the `Duplicate` arm (the event is already on disk from first ingest;
`persist_event`'s own guard would no-op anyway, but we skip the call). The `Duplicate` arm
does **not** carry `new_joiner` (a duplicate join's joiner is already a member; `Accepted`
already returns `new_joiner: None` for an existing member today — behaviour preserved).

### F3-D4 (F3-D) — suppression scope: both deliveries, via `FanoutRequest::none()`

Returning `FanoutRequest::none()` for `Duplicate` suppresses **both** local member broadcast
(`apply_fanout`) and federation peer push (`apply_federation_push`), because both key off
`FanoutRequest.event`. A duplicate must neither re-broadcast locally nor re-push to peers —
both achieved with no extra work. Falls out of F3-D3; confirmed.

### F3-D5 — side-effect-skip safety (the early-return loses nothing)

The dedup early-return (F3-D1) skips the re-run of `ingest_event` + Steps 6/7 on the
duplicate. Each effect is idempotent or already-fired (Phase-0 §6, re-confirmed here as a
build-time obligation):

1. `graph.add_event` — set semantics; re-call is a no-op.
2. `store.append` — returns `DuplicateEventId`, already swallowed.
3. `apply_event` / `derive_resolved` — state is a pure function of the log; the log already
   contains the event, so the second apply is a pure idempotent no-op. **Confirmed by the
   convergence regression suite (285) + the F3-D7 unit.**
4. `record_key_package` (MlsKeyPackage hook) — keyed by (identity, device), overwrites
   identically; first ingest already stored it.
5. drain hooks (Steps 6/7) — keyed on the event's own id; the first ingest already fired any
   drain waiting on `event_id`. A duplicate unblocks nothing new.

---

## 4. Change surface

- `xgen-core/src/node/runtime.rs`:
  - `DispatchOutcome` — add unit variant `Duplicate` (+ doc-comment, kept with the enum to
    avoid the MP-F2 `doc_lazy_continuation` clippy trap).
  - `dispatch_event` — the F3-D1 dedup check (post-`ensure_store`, pre-`validate_event`).
  - drain re-dispatch ×3 (1623/1707/1791) — `Duplicate` folded into the no-op arm.
- `xgen-node/src/app.rs`:
  - `process_inbound` `match outcome` (2593) — the F3-D3 `Duplicate` arm.
- `xgen-node/src/admin_ops.rs` (4013/6030) + `xgen-node/src/migration_driver.rs` (264) —
  trivial `Duplicate` arms (compiler-caught).
- Test modules — trivial `Duplicate` arms where an exhaustive `match` exists; the new units
  (§5) + the MP-A-09 flip.

No frozen-string change (no reason strings touched — `Duplicate` is not an error). The only
wire-observable behaviour is the F3-D3 idempotent ack, which is the locked, intended change.

---

## 5. Proof plan (and the pinned proof obligation)

**Pinned success criterion (Joe-lock):** flip the MP-A-09 harness assertion
(`xgen-mptest/tests/mp_r1_c7.rs:154`, `mp_a_09_duplicate_dedup_holds`) from the tolerant
`assert n >= 1` (which tolerates the `n > 1` re-emit and carries the measurement-gap note) to
**"the duplicate event is fanned out exactly once"** — the duplicate's `event_id` appears
exactly once per recipient transcript. This is what makes MP-F3 **falsifiable** (a regression
that re-broadcasts a duplicate fails it) and retires the MP-F3 measurement-gap note, turning
MP-A-09 from *PASS-on-property + routed finding* into a **clean PASS**.

- **F3-D6 (xgen-core unit):** submit event E → `Accepted`; submit E again → `Duplicate`;
  assert `SpaceState` + store length are byte-identical after the second submit (the
  duplicate changed nothing). Covers F3-D1 + F3-D2.
- **F3-D7 (xgen-core unit):** the side-effect-skip safety — a duplicate's apply is a no-op
  (state identical) and no pending event is spuriously drained. Covers F3-D5.
- **F3-D8 (xgen-node unit):** the `Duplicate` arm yields `FanoutRequest::none()` (no local
  fan-out, no federation push) AND, for `LocallySubmitted`, still sends `EventAccepted`;
  for `ReceivedViaFederation`, sends no ack. Covers F3-D3 + F3-D4.
- **MP-A-09 flip (close deliverable):** the exactly-once assertion above.
- **Convergence regression net:** the 285 binary-convergence integration tests + xgen-core
  (667) + xgen-node (72) stay green — the D-076 discharge (§6) at the suite level.

---

## 6. Safety — D-076 discharge (the dedup decision is convergence-neutral by construction)

A duplicate `event_id` is, by definition, **already in the store and already applied** — its
first arrival ran the full pipeline. The node's DAG, EventStore, and derived `SpaceState` are
byte-identical whether the duplicate is re-applied idempotently (today) or dropped at dispatch
(this fix); both leave the log containing the event exactly once. The only behavioural
difference is the fan-out emission, which is delivery/observability — never read by
`derive_resolved` / `state_key_for_event` / the resolver (same class as MP-F2's reject code).

Cross-node: a node that drops a duplicate and a node that re-applies-idempotently both hold the
event exactly once → `derive_resolved` over their logs is identical → they converge
identically. The required invariant — *a node that drops a duplicate must converge identically
to one that re-applies-idempotently* — holds because state is a pure function of the log and
the log is identical in both. F3-D5's idempotency audit is the load-bearing part; the 285
convergence tests are the net.

---

## 7. Scope fence + honest boundary (D-065)

**In scope:** dedup-at-dispatch for **already-accepted (already-stored)** duplicates; the
`Duplicate` outcome; the no-fan-out + idempotent-ack mapping; the MP-A-09 flip.

**Surfaced-not-chased (out of scope, recorded in Phase-0 §9):**
1. **Drained events do not fan out** — `process_inbound`'s `FanoutRequest` carries only the
   triggering event, not `additional_persisted`. A separate drain→fanout seam, not MP-F3.
2. **Re-submit-while-pending** — an event in the pending buffer (not the store) that re-arrives
   is not caught by store-based dedup; it is already benign (`PendingBuffer::add` is idempotent
   by `event_id`). Store-based dedup deliberately scopes to already-accepted duplicates.
3. **Harness oracle gap** — the `.events` transcript measures fan-out emissions, not the DAG.
   MP-F3 makes the fan-out count an honest exactly-once proxy in the no-pending case; the
   oracle still cannot directly read the DAG (a harness-machinery item, not a production fix).

No DECISIONS change (F3-D# arc-local, D-069).

---

## 8. Entry point (Rule 0)

CLAUDE PLAY → JOURNAL J-324 → `tasks/MP_findings.md` MP-F3 → this design (`§3` locks, `§5`
proof obligation) → Phase-0 audit `tasks/MP_F3_DEDUP_REFANOUT_AUDIT.md`. Next: Joe records the
lock → runbook (`tasks/MP_F3_DEDUP_REFANOUT_IMPL.md`) → implement → close (MP-A-09 → clean
PASS). Build 0 + clippy clean on default **and** `--features harness-control` at each step.

---

*Per D-065 + D-067 + D-069 + D-071 + D-076.*
