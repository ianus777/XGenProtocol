# M-RP6.3 Leg D1 — send-path honesty: the undrained send stops lying
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-19  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this leg is, and what it is deliberately not

**It is:** two Rust changes on the client send path, closing a **live D6
violation** that has been shipped since Leg B and is unreachable today only
because no composer exists to hit it.

**It is NOT:** the composer · the echo store · the status indicator · any
frontend work at all. **Zero `ui/**` files.** The frontend lifecycle guard
belongs to Leg D2 and is deliberately not here — *the guard is the nicety, this
leg is the guarantee* (§0 G-4: a half-open socket defeats any guard).

**Why it goes first:** it depends on nothing else in Leg D, and building a
composer on top of a send path that can hang would mean discovering the hang
through the composer — where it looks like the composer's fault.

**⚠️ This leg MOVES `cargo test` off 1541.** That is the honest signal Rust
landed. A leg that changes `.rs` and leaves the count identical has not shipped
its tests.

---

## §1 — Scope

| in | out |
|---|---|
| `xgen-client/src/desktop.rs` — `send_message` bounded wait | anything in `ui/**` |
| `xgen-client/src/resident.rs` — the extracted awaiter + its tests; the drain's abandoned-request check | `SendOutcome`'s shape (unchanged) |
| new unit tests | `xgen-core` (GPL core untouched, the Leg B/C precedent) |
| | the node, the sampler, `skin.css` |

**No new dependency.** See §2.4 — the obvious test approach needs one and is
rejected for that reason.

---

## §2 — Grounded anchors (verified against HEAD, 2026-07-19)

### §2.1 — The hole, at source

`desktop.rs` queues and waits:

- `try_send(OutboundRequest{...})` → on failure returns `SendOutcome::failed("not queued: …")`
- then `reply_rx.await`, with **no timeout of any kind**

`resident.rs:418` — `Some(req) = io.outbound_rx.recv()` sits **inside
`run_session`'s `select!`**. `run_session` exists only while a session exists.
**Between sessions nothing polls that receiver.**

Therefore, during an outage: the request is accepted into the channel, **the
future never resolves**, and when the link returns the drain pops it and
**writes it**, however much later.

### §2.2 — Why the existing timeout does not cover it

`SEND_ACK_TIMEOUT = Duration::from_secs(10)`, swept on a 1 s interval, ages
entries in `pending` — the map populated at `resident.rs:446`, **after
`conn.send_event(&ev).await` has already succeeded.** A request that was never
written never enters `pending` and **has no timer at all**.

### §2.3 — The channel

`desktop.rs:828` — `mpsc::channel::<OutboundRequest>(64)`. So `try_send` only
fails once 64 requests are already queued; the first 63 sends of an outage all
succeed into the queue and hang. **`desktop.rs:59`'s comment already claims
*"the queue surfaces as an honest `failed`, never as a silent drop"* — that is
true of the FULL queue and false of the undrained one.** The comment is
corrected as part of this leg (D-065: a comment asserting a property the code
does not have is a false record).

### §2.4 — ⚠️ The test is constructible, but NOT the obvious way

`tokio = { features = ["full"] }`, and `#[tokio::test]` is already in use
(`aicontrol.rs:818` and following) — **so the async test harness exists.**

**But `full` does NOT include `test-util`**, so `tokio::time::pause()` is
unavailable and a test of a 10-second timeout would **sleep ten real seconds**.

**Resolved by extraction, not by a new dev-dependency** (§3.1): the wait becomes
a free function taking its duration as a parameter, so the test passes
milliseconds. This follows Leg A's own precedent — *pure schedule logic
unit-tested, no clock, no socket*.

*Recorded because a runbook that demands an unsatisfiable test is the J-548
defect, and it was Chat's error last time.*

---

## §3 — The build

### §3.1 — Bound the wait, and return `failed`

Extract into `resident.rs`, beside `SendOutcome`:

```rust
/// Await the drain's verdict, bounded. An expiry here means the request was
/// QUEUED BUT NEVER WRITTEN — the drain only polls `outbound_rx` inside
/// `run_session`, so an outage leaves it unread with no other timer over it
/// (`SEND_ACK_TIMEOUT` ages only sends already on the wire).
///
/// The expiry is `failed`, NOT `timed_out`. `failed` means *never reached the
/// wire*, which is exactly what happened. `timed_out` would claim the node may
/// be holding the event — the D6 lie inverted, and it would also make the
/// message ineligible for a free retry that is in fact perfectly safe.
pub async fn await_send_outcome(
    reply_rx: tokio::sync::oneshot::Receiver<SendOutcome>,
    timeout: Duration,
) -> SendOutcome { … }
```

Three arms, all explicit:

| | result |
|---|---|
| reply arrives | that `SendOutcome`, verbatim |
| channel closed without a reply | `failed("resident stopped before the send resolved")` — the shipped string, preserved |
| timeout elapses | `failed("not sent — no link")` — **wording provisional, Ms Design owns user-facing copy** |

**⚠️ THE CALLER'S BOUND IS NOT `SEND_ACK_TIMEOUT`, AND v1.0 GOT THIS WRONG**
(Clair, before implementation — Rule 6). `pending.insert` stamps
`Instant::now()` **at WRITE time** (`resident.rs:446`) and the sweep
(`interval(Duration::from_secs(1))`, `MissedTickBehavior::Delay`,
`resident.rs:334`) resolves at *write* + `SEND_ACK_TIMEOUT` + up to one tick.
The caller's wait starts **strictly earlier than the write**. So an outer bound
of `SEND_ACK_TIMEOUT` expires first **always, by construction, for every
δ > 0** — `timed_out` becomes **dead on this path**, and a send that genuinely
reached the wire and is genuinely ambiguous is reported *never reached the
wire*. ***That is the mirror-lie §3.2 exists to prevent, re-introduced by §3.1's
own call site.*** *U1 would still have passed: it proves the pass-through at
the seam while the call site makes it unreachable in production — the exact
"silently not-run" shape this runbook names for U3, and it was in the runbook's
own instructions.*

So: **name the sweep literal** (`resident.rs:334` is unnamed, and a derivation
cannot cite what has no name), and **derive the caller's bound**:

```rust
pub const PENDING_SWEEP_INTERVAL: Duration = Duration::from_secs(1);

/// The ceiling on the CALLER's wait. It is a BACKSTOP, not the primary
/// mechanism: a send that reaches the wire is always resolved by the drain
/// (ack, or the sweep at SEND_ACK_TIMEOUT). This bound exists solely for the
/// request that was queued and NEVER WRITTEN, which has no other timer over it.
/// It must exceed the drain's worst case, measured from a write that begins
/// AFTER this wait does — or a written-but-unacked send surfaces as `failed`,
/// "never reached the wire" about a message that did.
pub const SEND_QUEUE_TIMEOUT: Duration = Duration::from_secs(
    SEND_ACK_TIMEOUT.as_secs() + PENDING_SWEEP_INTERVAL.as_secs() + 5,
);
```

`send_message` calls `await_send_outcome(reply_rx, SEND_QUEUE_TIMEOUT).await`.
**The duration stays a parameter at the seam and a named constant at the call
site** (D5: shape settable, do not wire settable).

**⚠️ 16 s, and the margin is deliberately GENEROUS rather than tight.** δ — the
delay before the drain pops *this* request — **is bounded by nothing**: the
channel holds 64 and a backlog makes it arbitrarily large, so **no constant is
provably correct.** What matters is the direction of the error: too large costs
an outage a few extra seconds before it says *not sent*; **too small relabels an
ambiguous send as a definite failure**, which is the D6 lie. With Leg D2's
lifecycle guard the common outage never reaches this call at all — this path is
the half-open and racy case, so generosity is cheap.

**A DERIVED constant, not a chosen one, and it is checked by a test (U4):** a
hand-picked `13` would go silently wrong the day someone tunes
`SEND_ACK_TIMEOUT` up. *A comment cannot fail; a test can.*

### §3.2 — ⚠️ The drain must SKIP an abandoned request — and this half is not optional

**Without it, §3.1 creates a NEW D6 violation and it is the mirror of the one
being fixed.** We tell the user *not sent*; two minutes later the link returns,
the drain pops that same request and writes it; the message **appears in the
room**. *A message that WAS sent must not look lost* — D6's binding rule, §1.

`send_message` returning drops `reply_rx`, which makes the paired
`oneshot::Sender` **closed**. So the drain already has the signal it needs:

```rust
Some(req) = io.outbound_rx.recv() => {
    // The caller has gone: `send_message` timed out and has ALREADY told the
    // user this message was not sent. Writing it now would put a message on the
    // wire that the user was told failed — D6's mirror. Drop it, and say so.
    if req.reply.is_closed() {
        tracing::info!(space_id = %req.space_id, "resident: dropping abandoned outbound (caller timed out)");
        continue;
    }
    …existing build → sign → write…
}
```

**Honest limit, stated not hidden:** if the timeout fires at the same instant
the drain begins writing, the write proceeds. The guarantee is *the drain never
KNOWINGLY writes an abandoned request*, *not* a perfect mutual exclusion. **The
window is one `select!` branch wide, and naming it is cheaper than a lock that
would have to be held across an `.await`.**

**This does not conflict with D6's *"send when reconnected"* offer.** That offer
is an explicit user action producing a **new** request. The abandoned one dies —
which is exactly what makes the offer honest rather than a silent queue wearing
a button.

### §3.3 — Correct the stale comments — THREE, not one

v1.0 named only the first. Clair grounded the other two, **both sitting directly
on the code this leg changes**:

| where | the false claim |
|---|---|
| `desktop.rs:59` | the queue *"surfaces as an honest `failed`, never as a silent drop"* — true of a **full** queue, false of an **undrained** one |
| `resident.rs:691` | `OutboundRequest.reply` is *"**Always resolved exactly once**"* — false for the undrained queue, **and false by design after §3.2**, which drops an abandoned request's sender unresolved |
| `desktop.rs:347` | *"The drain ALWAYS resolves the reply … so a closed channel here means the resident itself is gone"* — both halves false |

Rewrite all three to state what is now true: a **full** queue fails at
`try_send`; an **undrained** queue fails at the bounded wait; a **written** send
is resolved by the drain (ack, sweep, or teardown); an **abandoned** request is
discarded and logged.

*N-109's family, three at once: a stale honesty note is worse than a missing
one, because it was written by someone being careful — so the next reader
trusts it. And §3.2 does not merely expose these as stale, it makes the
strongest of them **newly** false, which is exactly when a comment must move in
the same commit as the code.*

### §3.4 — Not changed, deliberately

`SendOutcome`'s four values and shape · **`SEND_ACK_TIMEOUT`'s VALUE** · the
`pending` sweep's behaviour · the channel capacity · `ops::send` · anything in
`xgen-core`.

*Naming the sweep's `Duration::from_secs(1)` as `PENDING_SWEEP_INTERVAL` (§3.1)
is **not** a change to the sweep: same value, same behaviour, one literal given
a name so `SEND_QUEUE_TIMEOUT` can be derived from it instead of guessing at
it.*

---

## §4 — Verification

### §4.1 — Unit (the load-bearing half)

Three `#[tokio::test]`s over `await_send_outcome`, all with a **millisecond**
timeout (§2.4):

- **U1** reply delivered before expiry → that outcome verbatim, **including a
  `timed_out` reply passing through untouched.** *This is the leg that fails if
  anyone later folds the queue-expiry into `timed_out`: it proves the two are
  distinguishable at the seam, and a test that only checked "expiry is failed"
  would still pass after that mistake.*
- **U2** sender dropped without a reply → `failed`, the shipped string.
- **U3** no reply at all → `failed`, and **`status == "failed"` asserted, not
  merely "not accepted"** — a negative assertion passes for the wrong reason.
- **U4 — the ordering invariant, and it is the leg that outlives this milestone:**
  assert `SEND_QUEUE_TIMEOUT > SEND_ACK_TIMEOUT + PENDING_SWEEP_INTERVAL`. A
  **pure const check — no clock, no async.** *It exists because v1.0's defect was
  invisible to every other test here: U1 proves `timed_out` passes through the
  seam and says nothing about whether the call site can ever produce one. U4 is
  what fails the day someone tunes `SEND_ACK_TIMEOUT` up and silently re-creates
  the squeeze.*

**⚠️ U3 can silently not-run:** given a generous timeout it proves nothing but
that the test itself waited. Use single-digit milliseconds and assert the call
returned **before** a control duration — *the expiry must be shown to be the
cause, not a coincidence of ordering.*

One sync test for §3.2's predicate if it can be expressed without a live drain;
if it cannot, **say so in the handback rather than inventing a harness** — the
live drive below is then its only evidence, and that must be stated as such
(N-092a: name the proxy as a proxy).

### §4.2 — Live (one drive, real client 9222 + node)

There is **no composer yet**, so the command is driven directly:

1. **Confirm the invoke path first.** `window.__TAURI__` is a **FALSE-NEGATIVE
   probe — this build has no `withGlobalTauri`**, so it reports unreachable on a
   path that works (Clair, grounded). The working entry is
   **`__TAURI_INTERNALS__.invoke`**, the J-492 / J-497 precedent. **V-L is
   therefore expected to be drivable and is NOT pre-recorded as not-run** —
   confirm it live rather than claiming it, and if it genuinely fails, record
   the reason rather than faking the drive.
2. Node up → `send_message` → expect `accepted` (control: proves the path is
   otherwise intact and that a later `failed` means something).
3. **Kill the node.** Wait for the resident to leave READY.
4. `send_message` again → **must return `failed` within ~16 s** (`SEND_QUEUE_TIMEOUT`),
   with a wall-clock reading. *Today this call never returns; that difference is
   the whole leg.* **⚠️ Report the measured seconds** — a return at ~10 s would
   mean the v1.0 squeeze is still live somewhere.
5. **Restart the node, wait for READY, and confirm the abandoned message does
   NOT appear in the room** — driven from `bob`'s client or the stream panel.
   **This is §3.2's only end-to-end proof and the most important reading in the
   leg.** A missing message is an absence, so it needs the step-2 control to
   mean anything.

---

## §5 — Gates

| gate | floor | expectation |
|---|---|---|
| `cargo test` | 1541/0/62 | **MUST MOVE** — floor + exactly the new tests, counted, not estimated |
| `npm test` | 114 | unchanged (zero `ui/**`) |
| `vite build` | 192 client / 169 sampler | unchanged |
| sampler catalogue | 386 | unchanged |
| client registry | 134 quiescent | unchanged |

**⚠️ N-117:** the dev client **holds the exe** — stop both apps before any
`cargo` command, or the run dies on `failed to remove file …xgen-client.exe`.
A `0/0/0` result is **inconclusive, not a pass**.

**N-108:** state which numbers were **seen** and which were **derived**.

---

## §6 — Definition of Done

- [ ] `await_send_outcome` extracted, three arms, expiry → `failed`
- [ ] `PENDING_SWEEP_INTERVAL` named; `SEND_QUEUE_TIMEOUT` **derived from it and `SEND_ACK_TIMEOUT`**, not hand-picked
- [ ] `send_message` calls it with `SEND_QUEUE_TIMEOUT`; no unbounded `.await` remains on the send path
- [ ] the drain skips a request whose `reply` is closed, and **logs it**
- [ ] **all THREE** stale comments corrected (`desktop.rs:59`, `desktop.rs:347`, `resident.rs:691`)
- [ ] U1–U4 present; U1 covering the `timed_out` pass-through, U4 the ordering invariant
- [ ] `cargo test` moved off 1541 by **exactly** the new test count
- [ ] the four frontend floors unchanged
- [ ] V-L driven, **or explicitly recorded as not-run with the reason**
- [ ] deviations listed under Rule 6, not absorbed

*(Per house rule, "commit pushed" is NOT a DoD item — `Status: COMPLETED` is the
real signal.)*

---

## §7 — Handback

Report, in this order: what shipped · the `cargo test` count with its arithmetic
shown · every frontend floor · V-L's result **or its named absence** · Rule 6
deviations · anything found that this runbook did not anticipate.

**Do not report a number this runbook asked for if you did not measure it.**
Three handbacks on this arc were right in conclusion and wrong in evidence; two
reproduced exactly. **The distinction is the point of the seat.**

---

## §8 — Close (J-553)

### §8.1 — What reproduced

Chat re-drove the gates independently. **`cargo test` 1546/0/62 across 56
terminator lines — Clair's handback reproduces exactly**, the third consecutive
two-seat leg on this arc that does. Scope confirmed from `git show --stat
53dab37`: **`xgen-client/src/desktop.rs` + `xgen-client/src/resident.rs` only,
+241/−11** — zero `ui/**`, zero `xgen-core`, zero node, zero sampler, exactly
§1.

All five tests confirmed present by name at source, and
**`SEND_QUEUE_TIMEOUT` confirmed DERIVED, not hardcoded** (`resident.rs:812`:
`SEND_ACK_TIMEOUT.as_secs() + PENDING_SWEEP_INTERVAL.as_secs() + 5`), with
`PENDING_SWEEP_INTERVAL` wired into the sweep at `:334` — so the literal is gone
from the code, not merely shadowed by a named copy.

### §8.2 — ⚠️ THE FIRST GATE RUN WAS INCONCLUSIVE, NOT A PASS — N-117 LIVE

Chat's first `cargo test` returned **exit 101, ZERO terminator lines,
`passed=0 failed=0 ignored=0`** — *"failed to remove file
`…debug\xgen-client.exe`, Access is denied (os error 5)"*. **Reported here
because `0/0/0` is INCONCLUSIVE and reads exactly like a clean run to any
script that parses totals.** Had it been quoted as a result it would have been
a floor of zero.

**And the holder was NOT Clair's leftovers:** `xgen-client` PID 40432 and
`xgen-node` PID 19580, both started **14:02**, roughly **three hours after** her
11:02 V-L drive. Stopped, re-run, clean. *Recorded so the handback is not read
as having left the machine dirty — the processes postdate the drive.*

### §8.3 — 🔑 THE RUNBOOK WAS WRONG AND THE IMPLEMENTER CAUGHT IT FIRST

v1.0 §3.1 instructed `await_send_outcome(reply_rx, SEND_ACK_TIMEOUT)`. Grounded
against the drain, that constant makes `timed_out` **unreachable by
construction** (§3.1, v1.1). Clair stopped **before writing a line**, grounded
it, and brought it back — the third runbook on this arc caught by the
implementer reading it whole first, and **the second that was Chat's error**
(J-499, J-548, this).

**The class is stable across all three: internally consistent section by
section, contradictory ACROSS sections.** Here §3.1 said *bound the wait* and
§4.1 said *prove `timed_out` passes through*; both were correct alone, and only
the call site made one unreachable. ***The warning about exactly that shape was
three paragraphs above the instruction that violated it.***

### §8.4 — U4 WAS MUTATION-TESTED, NOT ASSUMED

Clair set `SEND_QUEUE_TIMEOUT = SEND_ACK_TIMEOUT` — **the v1.0 instruction** —
and U4 failed with *"SEND_QUEUE_TIMEOUT (10s) must exceed the drain's worst
case (11s)"*; reverted, re-confirmed green. ***A test that has never failed is
not yet known to be able to.*** **This is the strongest single act in the
handback:** U4 exists because v1.0's defect was invisible to every other test in
the file, and it was proven capable of catching that exact defect rather than
assumed to be.

### §8.5 — ⚠️ V-L MEASURED 19 155 ms, NOT ~16 s — AND WAS DECOMPOSED, NOT ROUNDED

§4.2 step 4 expected ~16 s. The drive returned **19 155 ms**. Clair did not
report it as "about right": from the client log, **t0 10:58:18.818 → re-anchor
fails 10:58:21.961 (+3.143 s, connection refused) → bounded wait 16.013 s.**
**The bound is therefore measured accurate to 13 ms**, and the control
corroborates the model — its 1142 ms contains a 1.13 s successful re-anchor.

**→ The runbook did not anticipate this: outage latency is `re-anchor +
SEND_QUEUE_TIMEOUT`, not `SEND_QUEUE_TIMEOUT`.** Filed to COMPOSER §9.11.8 so
nobody later reads ~19 s as a bug in the bound. *A number that disagrees with
the runbook and is then EXPLAINED is worth more than one that agrees.*

### §8.6 — V-L step 5: the absence proven against a positive control

`resident: dropping abandoned outbound (caller timed out)` logged at 11:02:44;
the node's persisted store then showed **control present (1) · abandoned absent
(0)**, resident READY for minutes afterward. **The same grep returned 1 for the
control** — so the 0 means something. *§3.2's only end-to-end evidence, and it
carried it.*

### §8.7 — The v1.0 probe would have recorded a false NOT-RUN

Settled live: `window.__TAURI__` → `"undefined"`; `__TAURI_INTERNALS__.invoke`
→ `"function"`. **v1.0's probe was a false negative on a path that works**, and
would have produced a defensible-looking *"V-L NOT RUN"*. → **N-139**.

### §8.8 — Deviations (Rule 6), all accepted

1. **§3.2's predicate test is a PROXY and is named as one** (N-092a). It proves
   the signal the drain branches on; it does **not** exercise the drain's
   `continue`, which needs a live session. **V-L step 5 is that half's only
   end-to-end evidence.** *Naming the proxy is the deviation done right.*
2. **U1 widened** beyond the runbook to loop `Accepted`/`TimedOut`/`Rejected`
   and assert `timed_out` separately by name — **`Rejected` carries a `code` the
   others do not, and "verbatim" is a claim about all four fields.** Accepted:
   the runbook under-specified it.
3. **Absence proven from the node's persisted store, not `bob`'s client** — a
   second identity would have proven **fan-out**, not **non-writing**. Correct
   instrument for the claim.

### §8.9 — Gates at close

| gate | floor | at close | how |
|---|---|---|---|
| `cargo test` | 1541/0/62 | **1546/0/62**, 56 terminators | **SEEN twice** — Clair, then Chat independently |
| new tests | — | **+5**, named | counted, not inferred from the delta |
| `npm test` | 114 | 114 | seen (Clair) |
| `vite build` | 192 / 169 | 192 / 169 | seen (Clair) |
| client registry | 134 | 134 === unique 134 | seen after full reload; `sel: null`, 0 saved states |
| sampler catalogue | 386 | **NOT re-measured** | **grounded BY SCOPE** — zero `ui/**` in the diff (J-497 precedent) |

**N-108 honoured throughout:** the one gate not measured is named as not
measured, with its warrant.

### §8.10 — Test environment, left changed

`LegBSpace` / `general` gained one control message (`D1-CONTROL-nodeup`). Both
apps stopped (and stopped again by Chat at §8.2). CDP probe globals cleared by
reload (N-123).

