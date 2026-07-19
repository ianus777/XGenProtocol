# M-RP6.3 Leg D1 — send-path honesty: the undrained send stops lying
> **Status**: ACTIVE  
> Version: 1.0  
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

`send_message` then calls
`await_send_outcome(reply_rx, crate::resident::SEND_ACK_TIMEOUT).await`. **The
duration is a parameter at the seam and a named constant at the call site**
(D5: shape settable, do not wire settable).

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

### §3.3 — Correct the stale comment

`desktop.rs:59` claims the queue always surfaces honestly. Rewrite to state what
is now true: a **full** queue fails at `try_send`, an **undrained** queue fails
at the bounded wait, and the drain discards an abandoned request. *N-109's
family: a stale honesty note is worse than a missing one, because it was written
by someone being careful.*

### §3.4 — Not changed, deliberately

`SendOutcome`'s four values and shape · `SEND_ACK_TIMEOUT`'s value · the
`pending` sweep · the channel capacity · `ops::send` · anything in `xgen-core`.

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

1. **First, confirm the invoke path is reachable from CDP at all.** If
   `window.__TAURI__` is not exposed in this build, **the live drive is NOT
   available and V-L is recorded as NOT RUN — not faked, not reasoned.** Chat
   settles this before the leg is handed over. *A leg that can silently
   not-run must be named before it is driven.*
2. Node up → `send_message` → expect `accepted` (control: proves the path is
   otherwise intact and that a later `failed` means something).
3. **Kill the node.** Wait for the resident to leave READY.
4. `send_message` again → **must return `failed` within ~10 s**, with a wall-clock
   reading. *Today this call never returns; that difference is the whole leg.*
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
- [ ] `send_message` calls it with `SEND_ACK_TIMEOUT`; no unbounded `.await` remains on the send path
- [ ] the drain skips a request whose `reply` is closed, and **logs it**
- [ ] `desktop.rs:59`'s comment corrected
- [ ] U1–U3 present, U1 covering the `timed_out` pass-through
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
