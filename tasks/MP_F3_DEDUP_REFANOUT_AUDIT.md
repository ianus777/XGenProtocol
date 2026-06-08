# MP-F3 — duplicate re-fan-out — D-071 PHASE-0 AUDIT

> **Status**: ACTIVE
> Version: 1.0
> Date: Jun 2026
> **Last updated**: 2026-06-08
> Language: English
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

---

## 0. Discipline note

Phase-0 grounding only — **no code, no design**. This audit grounds the dispatch→fan-out
path against live `main`, surfaces the forks with recommendations, and runs the D-076
convergence check. The design is authored only after Joe locks the forks (§10). This is the
second production-crate fix-arc of the loop-to-green (MP-R1-D10), after MP-F2 (J-324). Same
discipline: ground-before-conclude, frozen-string / regression-net awareness, build 0 +
clippy clean (default + `--features harness-control`) at each step.

---

## 1. The finding (recap, `tasks/MP_findings.md` MP-F3)

- **Surfaced:** MP-A-09, C7 (J-321). Severity **minor** — mild fan-out amplification, no state
  corruption.
- **Symptom:** the same `event_id` submitted twice is **applied once** (DAG/store/disk dedup
  holds) but **re-broadcast** via `apply_fanout` on the second submit — members/observers
  receive it twice.
- **Root (as routed):** `dispatch_event` returns `Accepted` for a re-submitted duplicate →
  `process_inbound` builds a fan-out request → re-broadcast. A duplicate could be dropped at
  dispatch before fan-out.
- **Route:** a dedup-at-dispatch / fan-out-suppression arc. Production change (xgen-core /
  xgen-node).

---

## 2. Grounding — the dispatch→fan-out path (live `main`)

### 2.1 The accept/fan-out chokepoint

`process_inbound` (`xgen-node/src/app.rs:2434`) is the single inbound chokepoint. It calls
`rt.dispatch_event(...)` (app.rs:2582), drops the runtime lock, and `match`es the outcome
(app.rs:2593):

| outcome | what process_inbound returns | fan-out result |
|---|---|---|
| `Accepted { new_joiner, additional_persisted }` | `FanoutRequest { event: Some(event), new_joiner }` (app.rs:2687) | **fans out** |
| `HeldPending` | `FanoutRequest::none()` (app.rs:2699) | no fan-out |
| `Rejected(info)` | `reject_signal` + `FanoutRequest::none()` (app.rs:2701+) | no fan-out |

The returned `FanoutRequest` then drives **both** delivery paths at the caller
(app.rs:1714 + the federation-push sibling):
- `apply_fanout(fanout, ...)` (`xgen-node/src/fanout.rs:213`) — local members + `.events`
  observers.
- `apply_federation_push(...)` — federation peers (origin == LocallySubmitted).

**Both** key solely off `req.event` being `Some`: `apply_fanout` early-returns when
`req.event` is `None` (fanout.rs:219-222). So **`FanoutRequest::none()` suppresses local
fan-out AND federation push in one move** — this is what the `HeldPending`/`Rejected` arms
already rely on. There is no dedup inside `apply_fanout` itself; it is a pure broadcaster.

### 2.2 `dispatch_event` has no dedup gate

`dispatch_event` (`xgen-core/src/node/runtime.rs:919`) runs: Step 1 space-exists pre-check →
Step 2 F-3 federation-relationship gate → Step 3 `ensure_store` + `validate_event` → Step 4
semantic gates → Step 5 `ingest_event` → Steps 6/7 drains → `Accepted` (runtime.rs:1538).
**Nowhere does it ask "is this event_id already in the store?"** A duplicate of an
already-accepted event passes Step 1 (space exists), passes `validate_event` (which checks
structure / causality / membership / signature / timestamp — **not** dedup), passes the
semantic gates, runs `ingest_event` again, and returns `Accepted`.

### 2.3 Where dedup *currently* happens — and why it is silent

Dedup holds at three lower layers, all inside `ingest_event` (runtime.rs:528) and persist,
all **silent** (they swallow the duplicate rather than signalling it upward):

1. **graph** — `graph.add_event` (runtime.rs:612) is structurally idempotent (tips/successors
   are sets); a duplicate is a no-op (any `Err` is logged + continued, per the (a).iii.α
   silent-discard).
2. **store** — `store.append(event.clone())` (runtime.rs:629) is `let _ = ...`; the
   underlying `InMemoryEventStore::insert` **returns `Err(StoreError::DuplicateEventId)` on a
   duplicate id** (`xgen-core/src/dag/store.rs:250-258`) — but the `Err` is discarded.
3. **disk** — `persist_event` has a per-event duplicate-guard.

The store already exposes the **clean dedup primitive**: `EventStore::contains(&EventXgid)`
(store.rs:315) — a true/false membership check on the appended set. Because `event_id` is the
SHA-256 canonical-content hash, "same event_id" ⟺ "same canonical event" ⟺ a genuine
duplicate (no false-positive risk).

**The gap is purely that the dedup signal never reaches the outcome that drives fan-out.** The
store *knows* the event is a duplicate (`insert` returns `DuplicateEventId`); `dispatch_event`
throws that knowledge away and reports `Accepted`. This is structurally the same shape as
MP-F2 (the wire code existed but was dropped before the `Error` frame) — a known fact dropped
before the layer that needed it.

---

## 3. The clean dedup decision point

The natural, no-drift location is **inside `dispatch_event`**: it is the one function that
(a) already resolves the effective `space_id`, (b) already ensures the store exists, and
(c) produces the `DispatchOutcome` that drives fan-out. A `store.contains(event_id)` check
there — right after Step 3 `ensure_store` succeeds (runtime.rs:1080-1085), before
`validate_event` — returns a "this is a known duplicate" outcome that `process_inbound` maps
to `FanoutRequest::none()`. Placing it post-`ensure_store` also skips re-validating a known
duplicate (a small free win) and guarantees the store handle exists.

Rejected alternative: dedup in `process_inbound` (peek the store before calling
`dispatch_event`). This duplicates `space_id` resolution and splits the dedup logic across the
crate boundary — a drift surface (D-067). The dispatcher is the single source of truth for
"what happened to this event."

---

## 4. Forks

### F3-A — where the dedup check lives  *(recommend: `dispatch_event`, post-`ensure_store`)*

- **A1 (rec):** in `dispatch_event`, after Step 3 `ensure_store`, before `validate_event`.
  Minimal, no-drift, reuses the existing `space_id` + store handle.
- **A2:** in `process_inbound` before dispatch. Rejected — duplicates resolution, splits logic.
- **A3:** in `ingest_event`. Rejected — `ingest_event` returns `()` and is also the replay
  path; it is not the fan-out decision point.

### F3-B — outcome shape  *(recommend: new `DispatchOutcome::Duplicate` variant)*

- **B1 (rec): new variant `DispatchOutcome::Duplicate`** (sibling to `HeldPending` — a
  "do-not-fan-out, not-an-error" outcome). `process_inbound` collapses it with `HeldPending`
  for the no-fan-out path: `HeldPending | Duplicate => FanoutRequest::none()` (modulo the
  F3-C ack choice). **Semantically honest:** a duplicate is neither freshly-`Accepted` nor
  `Rejected`.
  - **Blast radius:** adding a *variant* (not changing an existing one's shape) leaves every
    `matches!(_, Accepted { .. })` and `matches!(_, Rejected(_))` wildcard **intact** — only
    genuinely-exhaustive `match outcome {}` blocks need a new arm. Grounded count of those:
    **~10 sites**, all trivial (`Duplicate => {}` or fold into the no-op arm):
    `runtime.rs` drain re-dispatch ×3 (1623/1707/1791), `app.rs` process_inbound (2593) +
    drain `match first` (1447), `admin_ops.rs` ×2 (4013/6030), `migration_driver.rs` (264),
    test exhaustive ×2 (`federation_relationship_integration.rs:294`,
    `m8_s7_privilege.rs:98`). Smaller than MP-F2's ~37 binders, all compiler-caught.
- **B2:** reuse `Accepted` with a discriminator: `Accepted { new_joiner, additional_persisted,
  novel: bool }` (novel=false for a duplicate). No new variant / no new match arms, but every
  `Accepted { .. }` **construction** site churns (1538/1624/1708/1792 + test builders), and it
  is semantically muddier — `matches!(outcome, Accepted { .. })` would still be true for a
  duplicate, masking the distinction in the ~140 test assertions that use it. Rejected on
  honesty + the masking risk.
- **B3:** thread "was-novel" back through an out-param. Rejected — not clean.

The MP-F2 "1-tuple to minimize wildcard churn" lesson does **not** push toward B2 here:
MP-F2 *changed an existing variant's payload* (every `Rejected(_)` matcher cared), whereas
MP-F3 *adds a new variant* (existing wildcards are untouched). B1 is the lower-churn AND
more-honest choice — the rare case where they align.

### F3-C — ack to the originator on a duplicate  *(recommend: idempotent ack — yes — for LocallySubmitted; needs Joe-lock — wire-observable)*

Today `accept_signal` (`EventAccepted`) fires only for `LocallySubmitted` `Accepted` events
(app.rs:2680). A duplicate from a retrying client should arguably still get an ack — we *did*
accept the event (the first time), so acking "I have your event" is truthful and stops a
client's retry loop. The thing that must not repeat is the **fan-out**, not the ack.

- **C1 (rec):** `Duplicate` from a `LocallySubmitted` origin still sends `EventAccepted`
  (idempotent ack), but returns `FanoutRequest::none()`. Truthful; retry-friendly.
- **C2:** silent (no ack on duplicate). Simpler, but a client that retried on a dropped ack
  keeps retrying.

This is **wire-observable behaviour**, so it is a genuine Joe-lock (not an implementation
detail). Note: MP-A-09's duplicate is an *injector* re-submit (adversarial) — the real-client
retry case is the one C1 serves; both are covered identically.

### F3-D — scope of suppression  *(falls out — confirm)*

Because `apply_fanout` and `apply_federation_push` both key off `FanoutRequest.event`,
returning `FanoutRequest::none()` for a `Duplicate` suppresses **both** local member
broadcast and federation peer push in one move. A duplicate should neither re-broadcast
locally nor re-push to peers — both are achieved with no extra work. Confirm, no fork.

---

## 5. D-076 convergence check (the load-bearing argument)

**The dedup decision is convergence-neutral by construction, and cannot perturb cross-node
convergence.**

- A duplicate `event_id` is, by definition, **already in the store and already applied** — its
  first arrival ran the full pipeline. The node's DAG, EventStore, and derived `SpaceState`
  are byte-identical whether the duplicate is (a) re-applied idempotently (today) or
  (b) dropped at dispatch (proposed). Both leave the log containing the event exactly once.
- The only behavioural difference is the **fan-out emission**, which is delivery/observability
  — never read by `derive_resolved` / `state_key_for_event` / the resolver. Same class as
  MP-F2's reject code (D-076 discharged there because the code is read only by the delivery
  layer). MP-F3's `Duplicate` outcome returns `FanoutRequest::none()` — it touches no
  admission/ordering/resolution surface.
- **Cross-node:** a node that drops a duplicate and a node that re-applies-idempotently both
  hold the event exactly once → `derive_resolved` over their logs is identical → they converge
  identically. The requested invariant — *"a node that drops a duplicate must still converge
  identically to one that re-applies-idempotently"* — holds because state is a pure function of
  the log and the log is identical in both. The 285 binary-convergence integration tests are
  the regression net.

**Confirm in design:** the dedup keys on `store.contains(event_id)` (exact hash membership),
never a heuristic — so it can never drop a legitimately-new event (which by definition has a
different content hash → different `event_id`).

---

## 6. Side-effect-skip audit (does dropping the duplicate before `ingest_event` skip anything needed?)

A dedup early-return (F3-A) skips the entire re-run of `ingest_event` + Steps 6/7 on the
duplicate. Each effect that re-runs today is confirmed safe to skip:

1. **`graph.add_event`** (runtime.rs:612) — set semantics; the re-call is already a no-op.
2. **`store.append`** (runtime.rs:629) — returns `DuplicateEventId`, already swallowed;
   skipping = no change.
3. **`apply_event` / `derive_resolved` re-run** (runtime.rs:633-675) — the duplicate re-applies
   to `SpaceState` today. Skipping it is safe because state is a pure function of the log and
   the log already contains the event (first apply established final state; a second apply is a
   pure idempotent no-op). **Design phase confirms `apply_event` idempotency** via the existing
   convergence suite + a targeted unit; the pure-function-of-log argument is the strong default.
4. **`record_key_package`** (MlsKeyPackage hook, runtime.rs:683-685, 694) —
   `key_package_store.store(...)` is keyed by (identity, device) and overwrites identically →
   idempotent. First ingest already stored it; skip-safe.
5. **Drain hooks (Steps 6/7)** (runtime.rs:1515-1536) — keyed on the event's **own** id
   (`drain_pending_uniform(&space_id, eid)`) + the fed-add pair. A duplicate `event_id` cannot
   unblock any pending event that the *first* arrival didn't already unblock (the first ingest
   of `event_id` already fired its drain). Skip-safe.

All five re-run effects are idempotent / already-fired; the early-return loses nothing.

---

## 7. Blast radius + regression nets

- **Code:** ~10 trivial exhaustive-match arms (§F3-B), all compiler-caught; one dedup check in
  `dispatch_event`; one `Duplicate` arm in `process_inbound` (+ the F3-C ack choice).
- **No frozen-string concern** (no reason strings change — `Duplicate` is not an error).
- **Wire-format:** the only wire-observable is the F3-C ack-or-not choice; the `Duplicate`
  outcome itself is internal.
- **Regression nets:** xgen-core lib (667), xgen-node lib (72), integration convergence (285),
  + the MP-A-09 harness smoke. Build 0 + clippy clean on default **and** `--features
  harness-control`.

---

## 8. Test / close plan (turns MP-A-09 into a clean PASS)

- **Unit (xgen-core):** `dispatch_event` returns `Duplicate` for a re-submitted stored event;
  the first submit returns `Accepted`; `SpaceState` + store are byte-identical after both.
- **Unit (xgen-node):** the `Duplicate` arm yields `FanoutRequest::none()` (no fan-out, no
  federation push); per F3-C, `EventAccepted` still sent for `LocallySubmitted`.
- **MP-A-09 flip (the close deliverable):** the C7 test (`xgen-mptest/tests/mp_r1_c7.rs:154`,
  `mp_a_09_duplicate_dedup_holds`) today asserts `n >= 1` and **tolerates** `n > 1` (the
  re-emit), with a "measurement-gap / re-emit-tolerated" note. After the fix the duplicate is
  fanned out **exactly once** → flip the assertion to "duplicate appears once per recipient
  transcript" and drop the re-emit-tolerance prose. This is what turns MP-A-09's recording
  from *PASS-on-property + routed finding* into a **clean PASS**.

---

## 9. Out of scope / surfaced-not-chased (D-065)

- **Drained events do not fan out.** `process_inbound`'s `FanoutRequest` carries only the
  *triggering* event, not `additional_persisted` (the drained ones) — drain-recovered events
  reach the DAG but are not live-broadcast to members (members get them via sync). This is a
  separate observation about the drain→fanout seam, **not** MP-F3. Surfaced, not chased.
- **Re-submit-while-pending.** An event in the pending buffer (not yet in the store) that
  re-arrives is not caught by a store-based dedup (`store.contains` is false). It is benign:
  `PendingBuffer::add` (pending.rs:165) is keyed by `event_id` (`self.events.insert(eid, …)`)
  and uses HashSet `waiting_for*` inserts, so re-buffering is idempotent; on drain it fans out
  once. Store-based dedup deliberately scopes to *already-accepted* duplicates (the finding's
  scope). Noted.
- **Harness oracle gap.** The `.events` transcript measures fan-out emissions, not the DAG.
  MP-F3 makes the fan-out count an honest proxy for "applied once" (in the no-pending case),
  but the oracle still cannot directly read the DAG — a harness-machinery item, not a
  production fix. Unchanged by this arc.

---

## 10. Recommendation summary + Joe-lock asks

| Fork | Recommendation | Joe-lock? |
|---|---|---|
| **F3-A** where the dedup check lives | `dispatch_event`, post-`ensure_store`, `store.contains(event_id)` | confirm |
| **F3-B** outcome shape | new `DispatchOutcome::Duplicate` variant (sibling of `HeldPending`) | **lock** |
| **F3-C** ack on a duplicate | idempotent `EventAccepted` for `LocallySubmitted`; never fan out | **lock (wire-observable)** |
| **F3-D** suppression scope | `FanoutRequest::none()` suppresses local + federation (falls out) | confirm |
| **D-076** convergence | discharged by construction (state is a pure function of an identical log) | confirm |

**Verdict:** GAP CONFIRMED — minor (observability/amplification, not security, not
convergence). The fix is a clean dedup-at-dispatch: the store already knows the event is a
duplicate; surface that as a `Duplicate` outcome and map it to no-fan-out. Blast radius small
and compiler-caught; convergence-safe by construction.

**Next:** Joe-lock the forks (F3-B + F3-C are the real decisions) → author
`tasks/MP_F3_DEDUP_REFANOUT_DESIGN.md` → runbook → implement → close (MP-A-09 → clean PASS).
Holding for Joe-lock per Phase-0 discipline.

---

*Per D-065 + D-067 + D-069 + D-071 + D-076.*
