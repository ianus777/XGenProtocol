# M9.1 — Event Timestamp-Bound Validation (F1 / gap G6) — Design

> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

Close finding **F1 / gap G6** (`tasks/M9_findings.md`): the inbound validation core admits an
event with an arbitrary timestamp — surfaced live by the M9 injector (MP-A-15 ClockSkew). M9.1
adds a **timestamp admission bound** to the production validation path. Doc-only design; executes
on a single small Clair commit afterward. No DECISIONS change (M9.1-D# arc-local per D-069).

---

## 2. Grounding correction (D-065 — supersedes audit §3.1 / F-D)

The Phase-0 audit and J-308 cite the gap in **`validate_steps_8_13`** at
**`xgen-node/src/message/exchange.rs`**. Live `main` corrects this on two counts:

1. **Wrong crate path.** The file is `xgen-core/src/message/exchange.rs`; the Node runtime is
   `xgen-core/src/node/runtime.rs`. There is no `xgen-node/src/message/`.
2. **Wrong (deprecated) function.** `validate_steps_8_13` is `#[deprecated]` (J-125); its only
   callers are xgen-core test fixtures and the equally-deprecated `accept_event`. Patching it
   would **not** close the live hole. The production inbound gate is the **F-4 unified core
   `validate_event`** (`exchange.rs:466`), called live at **`runtime.rs:1060`** inside
   `dispatch_event`.

This is the same class of error as the J-307 R1 `ingest_event`→`validate_event` correction.
**M9.1 lands in `validate_event`.**

**Grounded facts the design relies on:**
- `Event.timestamp: String` (RFC3339, e.g. `2026-04-30T10:00:00.000Z`); no existing step parses
  or bounds it.
- `validate_event(event, space, id_registry, store, fed_add_via_federation)` is **pure** — it
  holds no clock. Its early checks (step 8 event_id hash, step 10 DAG structure) need neither
  store nor space.
- At the live call site (`dispatch_event`, `runtime.rs:1060`), both **`now`** (already computed
  as `self.clock.now_utc()`, D-090) and the event **`origin`** / `peer_node_id` are in scope.
- `validate_event` call sites: the def (exchange.rs:466), **one** test (exchange.rs:1255), and
  the **one** live caller (runtime.rs:1060). Threading a `now` param touches only these.

---

## 3. Locked decisions (M9.1-D1 … M9.1-D5)

### M9.1-D1 — Bound = future-skew ceiling **only** (F-A)
Reject an event whose timestamp is too far **ahead** of the receiver's `now`:

```
parse(event.timestamp) > now + MAX_FUTURE_SKEW   ⇒   reject
```

Far-**past** events are **legitimate** (federation catch-up, replay-from-disk on an append-only
log) and are **not** bounded. A malformed/unparseable timestamp is rejected (this is also the
first step that validates timestamp parseability at all).

**Causal monotonicity (`timestamp ≥ max(prev_events.timestamp)`) considered and deferred.** It is
clock-free and convergence-safe, but (a) its marginal coverage over D1 is narrow — only past-dated
events whose stamp falls below their own parents — and (b) a strict `≥` rejects honest replies
authored on a lagging clock, so it would need its own `SKEW_TOLERANCE` parameter and dedicated
tests, smuggling a second invariant into a single-commit fix. If past-skew coverage is wanted
later, it is its own small arc (or folds into M9.2 surface work).

### M9.1-D2 — Origin-gating = **both origins**, convergence-safe by margin (F-B — the crux)
The bound runs on **every** event regardless of origin (LocallySubmitted **and**
ReceivedViaFederation). It is **not** INV-EXP-style local-only.

This is safe **because a future ceiling is fundamentally different from invite-expiry** (see §4):
the invite gate is a *past-deadline* check that catch-up flips accept→reject; a future ceiling is
*monotone under catch-up* and never does. Both-origins is what actually **closes the federated
injector** (audit A5) instead of declaring it out of scope.

### M9.1-D3 — Where + how (F-D)
A new **"Step 8.5 — timestamp future-bound"** inside `validate_event`, placed with the other pure
structural checks (after step 8 event_id, alongside step 10 DAG structure, **before** step 9 /
HeldPending — a future-skewed event is rejected outright, never buffered).

- `validate_event` gains a `now: DateTime<Utc>` parameter. The live caller passes
  `self.clock.now_utc()`; unit tests pass a fixed `now` (the check is then fully deterministic
  with no MockClock needed at the unit level).
- A new error variant `ExchangeError::TimestampOutOfBounds` (covers both unparseable and
  over-ceiling; message distinguishes). Its wire code is assigned in the runbook with an explicit
  collision check against the spec's existing codes (the 6009-vs-6007 lesson) — provisional, not
  fixed here.
- **D-076 non-interference (hard constraint):** the check only rejects-or-passes. Accepted events
  flow through the existing path **unchanged**; the timestamp is **never** read by
  `state_key_for_event`, the resolver, or any ordering decision. Ordering stays wire-order.

### M9.1-D4 — Window value (F-E)
`MAX_FUTURE_SKEW` is a **named constant**, not a magic number. Recommended default **5 minutes** —
two orders of magnitude above realistic NTP skew (sub-second to low seconds) so honest skew never
flips a verdict, and far below a useful future-dating attack window. Named so it is the spec, not
tuned to pass a test.

### M9.1-D5 — Convergence safety is gated by proof (F-C)
The design is not done until the §5 repros are green. Two-node same-verdict under honest skew +
catch-up leniency + a sensitivity witness are **mandatory**, not optional.

---

## 4. Convergence-safety argument (why both-origins is safe)

INV-EXP (J-298) made the invite/join expiry gates **local-only** because invite-expiry is a
**past-deadline** check: it compares a fixed `valid_until` against a *moving, increasing* `now`.
A peer catching up an aged Space sees `now > valid_until` and would reject a historical join the
home node accepted while the invite was fresh → **divergence**. Local-only was the fix.

A **future-skew ceiling is the opposite shape** and the INV-EXP constraint does not transfer:

1. **Monotone under catch-up.** Catch-up only moves `now` **forward**. So `event.timestamp − now`
   only **shrinks** (and goes negative) as an event ages on the wire. An event that passed the
   ceiling at its home node passes it **more** comfortably at any later-arriving peer. The verdict
   can never flip accept→reject with time — the exact failure mode INV-EXP had cannot occur here.

2. **Live-receipt skew is margin-bounded.** The only moment two honest nodes can disagree is at
   near-simultaneous live receipt, and then only by their clock-skew δ. With `MAX_FUTURE_SKEW ≫ δ`
   (5 min vs sub-second NTP skew), the verdict is identical on all honest nodes except for an event
   deliberately stamped inside the δ-wide boundary — which is the malicious-peer case M9 already
   lists out of scope.

A node whose **own** clock is skewed ahead by more than `MAX_FUTURE_SKEW` could accept what an
honest node rejects — but that is a locally-broken clock, not a protocol divergence, and it fails
**open** (admits a slightly-future event), never closed against honest traffic.

**Net:** the bound yields the same accept/reject on every honest node, and is *more* lenient (never
stricter) on catch-up — so it is convergence-safe on both origins. This is the core design claim,
proven by §5(b)/(c).

---

## 5. Proof plan (in-process; gates the design — M9.1-D5)

All in xgen-core (`validate_event` is here; MockClock already a xgen-core dev-dependency from
INV-EXP for the dispatch-level repros).

**(a) Unit — `validate_event` bound (deterministic, fixed `now`):**
- timestamp `= now` → accept; `= now + 4 min` (under 5) → accept.
- timestamp `= now + 10 min` (over ceiling) → `Rejected(TimestampOutOfBounds)`.
- timestamp far **past** (`now − 30 days`) → **accept** (catch-up/replay legitimacy).
- unparseable timestamp (`"not-a-date"`) → `Rejected(TimestampOutOfBounds)`.

**(b) Honest-skew same-verdict (two nodes):** two `NodeRuntime`s, clocks differing by a small δ
(e.g. 2 s), ingest the **same** event with `timestamp = base + 10 min`. Both reject (margin
protects → identical verdict). A `timestamp = base` event → both accept.

**(c) Catch-up leniency (the monotonicity property):** event with a legitimate `timestamp ≈ base`
is accepted live at Node A (`A.now ≈ base`); the **same** event is ingested at Node B with B's
MockClock advanced (`B.now = base + 2 days`) → B **accepts** (now has moved past it). Proves an
aged event is never newly-rejected on catch-up.

**(d) Sensitivity witness:** revert the Step 8.5 arm → the over-ceiling injector event from (a)/(b)
is **admitted** (RED — the F1 hole reopens, and the absence-oracle would show it landing in
`.events`); restore → rejected (GREEN). This is the test a no-op fix would leave green.

---

## 6. Honest boundary / coverage ledger (D-065)

- **What M9.1 closes:** future-dated events from **any** origin (local liar / skewed local clock
  **and** the federated injector MP-A-15, given future-direction skew). The audit's feared
  "federated skewed event stays admitted by design" outcome is **avoided** — both-origins is safe,
  so the injector is caught, not excused.
- **What M9.1 does NOT close (named, not glossed):** **past**-dated events. A coherently-backdated
  event (old timestamp *and* old/absent parents) is indistinguishable from legitimate catch-up;
  no wall-clock gate can reject it without breaking federation catch-up. The only convergence-safe
  past-side check is causal monotonicity (D1, deferred). If MP-A-15 turns out to skew **into the
  past**, D1 alone does not close it — see §7 runbook-grounding item.
- **Scope:** M9.1 touches only the timestamp admission bound. It does not touch resolution/ordering
  (D-076), the invite-expiry gates (INV-EXP), or any sibling finding (F2/F3/F4 → M9.2).

---

## 7. Scope, risks, next-active

**Change surface (small):** `exchange.rs` (`validate_event` signature + Step 8.5 + the new
`ExchangeError` variant + `to_wire_code` arm); `runtime.rs:1060` (pass `self.clock.now_utc()`); the
one test call site (exchange.rs:1255) passes a fixed `now`; tests per §5. Single commit (D-074).

**Runbook-grounding item (must resolve before Clair):** confirm the **skew direction** of the M9
injector's MP-A-15 ClockSkew attack. If future → D1 closes it (assert rejection + absence). If
past → record in §6 as out-of-scope for M9.1 (and a candidate for the deferred monotonicity arc),
and have the sensitivity witness (5d) drive the bound with a *synthetic* future-skewed event so the
fix is still proven. Either way the fix is correct; this only determines which test asserts MP-A-15
specifically.

**Next-active: M9.1 runbook** (`tasks/M9_1_TIMESTAMP_VALIDATION_IMPL.md`) — Step 8.5 + error
variant + wire-code collision check + caller wiring + §5 tests + sensitivity witness → Clair (one
commit + doc-only close). Then **M9.2** (F2/F3/F4 fenced seams) → Multiparty-tests.

**Entry point (Rule 0):** CLAUDE PLAY → JOURNAL J-308 → `tasks/M9_1_TIMESTAMP_VALIDATION_AUDIT.md`
§3+§4 → this design §3+§4 → `tasks/M9_findings.md` (F1).

Per D-065 + D-069 + D-071 + D-074 + D-076 + D-090.
