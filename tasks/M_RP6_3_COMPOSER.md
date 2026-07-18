# M-RP6.3 — live messaging: R6 composer + narrow-B send path
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-18  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Grounding verdict (read against `main` @ `aa3cc3e`, 2026-07-18)

Six findings. Three of them made the milestone **smaller**, one made it **different**.

**G-1 — Retry cap does not exist.** `resident.rs::run_resident` loops forever.
`Backoff { attempt: u32 }` is private, has no getter, and `saturating_add`s
indefinitely; the schedule is `1·2·4·8·16·30·30…` (`BACKOFF_MAX_SHIFT = 5`,
`BACKOFF_CAP_SECS = 30`). → **A `(2/10)` counter is NOT expressible today.**
There is no `10`, and no way to read the `2`.

**G-2 — Next-attempt-time is not published.** The delay is computed and slept
inside `run_resident`; nothing escapes. → **A countdown is not expressible
today.**

**G-3 — The lifecycle sink cannot carry either.** `F: FnMut(ClientLifecycleState)`
is a bare enum, no payload. Attempt number and next-attempt-time have nowhere to
ride. A second published surface is required — mirroring the Leg-C
`TrafficCounters` + `get_conn_stats` shape already shipped (**one pattern, not a
new mechanism**, D-067).

**G-4 — The detection window is ~10 s, twice over.** `PING_INTERVAL = 10s`
(so a silently-dead peer goes unnoticed up to 10 s) and `CONNECT_TIMEOUT = 10s`
(so each retry dial can take up to 10 s). → **The countdown MUST hand off to a
`connecting…` state.** A countdown that reaches zero followed by up to ten
seconds of silence reads as frozen — the exact failure the animation exists to
prevent.

**G-5 — `send_event_confirmed` owns the connection and eats the drain.**
`xgen-core/src/transport/connection.rs:186` takes `&mut self`, sends, then drains
`recv()` until its `event_id` correlates — and **discards every non-correlated
inbound** (`_ => continue`, `:243`). Under a unified socket that means fan-out
events are swallowed while a send awaits its ack: **directly incompatible with
live ingest.** It is a family, not a method — `upload_blob` (`:256`),
`fetch_blob` (`:323`) and `batch.rs::get_dag_tips` share the shape. Call-site
sweep across `xgen-client/src`: ~114 hits on `send_event_confirmed` /
`connect_url` / `get_dag_tips` / `.recv()`.
→ **"Move correlation into the drain" as a general rule is a crate-wide
refactor, far past this milestone.**

**G-6 — There are no auth tiers, and no session lifetime.**
`xgen-core/src/transport/auth.rs`: nonce challenge → Ed25519 signature → verify.
`CHALLENGE_EXPIRY_SECS = 30` bounds the **challenge**, not the session. There is
**no session expiry, no re-auth, no scopes, no roles, no privilege levels**
anywhere in the transport auth layer. Authentication is binary and permanent for
the life of the socket. `resident.rs::ws_authority` documents Phase 1 as
**`ws://` only — no TLS**.

---

## §1 — Locked decisions

**D1 — NARROW B.** The resident socket is the **control / message plane only**.
Outbound chat messages multiplex over the resident's one socket; the `ops::*`
verbs keep their own short-lived connections, which is already correct under
**D-056** (they are control-mode, not resident). B was decided for the *user's
messaging experience* — that is the composer, not `create_space`.
*Rationale:* confines G-5's correlation work to one path and leaves the ~114
call sites untouched. Same user-facing win, a fraction of the blast radius.

**D2 — THREE PLANES, named as a standing invariant.**

| plane | traffic | connection |
|---|---|---|
| control / message | small, latency-sensitive, ordered | **the resident socket (B)** |
| bulk data | large, throughput-sensitive, cancellable | **own connection per transfer** |
| real-time media | loss-tolerant, latency-over-reliability | **out of scope; separate transport** |

**INVARIANT (binding on all future milestones):** the own-connection-and-drain
family (`send_event_confirmed`, `upload_blob`, `fetch_blob`, `get_dag_tips`)
**may never run on the resident socket.** A blob upload on the resident socket
blinds the client to inbound fan-out for the entire transfer.

**D3 — NON-BLOCKING DRAIN is a constraint, not a style.** Send drops into an
outbound queue and returns; the drain writes it and correlates the ack **without
ever stalling inbound**. This is what makes B strictly better for the user rather
than a trade — it captures A's one real edge (inbound stays crisp under a jammed
uplink). WebSocket has **no multiplexing** (unlike HTTP/2 or QUIC): head-of-line
blocking is inherent to one socket, and chunk-granularity fairness in the drain
is the only mitigation. `ops::send` stays in the tree as a **software** fallback
(for a multiplexer bug), **never a network fallback**.

**D4 — BOUNDED RETRY + TERMINAL STATE + AUTO-RESUME.**
Retry attempts are **capped**; on exhaustion the resident enters a **terminal
`disconnected` state with an explicit `[Reconnect]` action** — not a greyed-out
spinner. **Auto-resume on window-focus / network-return** keeps the
sleeping-laptop case recovering without infinite retry.
*Rationale:* an uncapped resident cannot express a counter and cannot offer a
terminal state; a capped one without auto-resume fails the laptop-wake case.
Both halves are required.

**D5 — SHAPE SETTABLE, DO NOT WIRE SETTABLE.** All tunables are **named
constants in one place** (`RECONNECT_MAX_ATTEMPTS`, `RECONNECT_BASE_DELAY`,
`GRACE_PERIOD_MS`, plus the existing `PING_INTERVAL` / `CONNECT_TIMEOUT` /
`BACKOFF_*`), and their **values travel to the UI as published data, never
hardcoded a second time in Svelte**. No behaviour keys on a specific number.
**Explicit NON-GOAL: no config-plumbing layer is built.** Migration to
settings-fed values waits on **J-513** (M-RP-SETTINGS). *A named constant
migrates in one line; a premature config layer charges now for a decision not
yet made.*

**D6 — FAIL-FAST DEFAULT, with an explicit queue offer.** While the link is
down, **send is blocked, typing is not** (never disable the input — killing the
textarea mid-thought is the worst response to a network blip). If the outage
outlasts the reconnect window, the UI **offers** "send when reconnected" as an
explicit user choice — never a silent queue.
**BINDING RULE: a message may never LOOK sent when it is not.**
*Honest limit (G-4):* a half-open socket means the first send after a drop can
fail **even with the guard in place** — the banner arrives after the failure.
Blocking sends is a nicety, never a guarantee; per-message failure state is still
required.

---

## §2 — Leg split

**Leg A — resident status publication (Rust).**
Cap + attempt counter + next-attempt-time escape `run_resident` on a published
surface mirroring `TrafficCounters` / `get_conn_stats` (G-3). Terminal state on
exhaustion; `[Reconnect]` command; auto-resume trigger on focus / network-return
(D4). Pure schedule logic unit-tested, no clock, no socket.
*DoD:* kill the node → attempt count and next-attempt-time observable via the
command; exhaust the cap → terminal state; `[Reconnect]` re-arms; a live
reconnect resets.

**Leg B — outbound multiplexer + ack correlation (Rust).**
An outbound queue + a drain that owns `recv()`, writes queued sends, and
correlates `event_id → oneshot` **without stalling inbound** (D3). Scoped to the
message path only (D1). Live ingest — the M-RP6.6 deferral — wires here, since
the drain is now the single reader.
*DoD:* a send under simulated inbound load does not stall the drain; an ack
correlates to its sender; a dropped connection fails in-flight sends
deterministically.

**Leg C — R5 stream wrap + live item contract (frontend).**
R5 `message-stream` exists fixture-driven (M-RP5.6) → this is a **wrap**, fed
live from Leg B's ingest. Adds the **live mutable item class** R5 does not have
today: a connection-gap item that counts, animates, and **resolves in place**.
- live: `reconnecting (2/10) · retrying in 6…` → hands off to `connecting…`
  across the dial window (G-4)
- resolved: **collapses in place** to `connection restored · 8s` — the live
  widget *is* the historical marker, matured
- exhausted: replaced by the terminal `disconnected — [Reconnect]` item (D4)
- **grace period ~2 s before anything appears** — a blip that recovers inside the
  window is never shown. *Silence is the correct UI for a blip.*
*Appearance is NOT specified here — see §5.*

**Leg D — R6 composer (frontend). BUILD, not a wrap.**
R6 does not exist. Textarea + send, wired to Leg B. Implements D6: send blocked
while down, typing never blocked, explicit queue offer on a long outage,
per-message failure state.
*DoD:* send round-trips live to the node; a send during an outage behaves per D6
and never renders as delivered.

---

## §3 — Accepted risks, with named successors

Filed, **not** solved here. None blocks M-RP6.3 shipping; all block **the
resident talking to anything beyond loopback**. The gate is on deployment, not
on this milestone.

| # | risk | successor |
|---|---|---|
| R-1 | **No TLS.** `ws://` only (`resident.rs::ws_authority`). Plaintext on the wire. | **M-SEC-TLS** |
| R-2 | **No session lifetime.** One authentication holds indefinitely; B extends the hijack window from seconds to days. | **M-SEC-AUTHSESS** |
| R-3 | **No auth scopes or tiers.** M6 admin verbs and a chat send carry byte-identical authentication. Mitigated *by convention* under D1 (admin stays control-mode), not *by mechanism*. | **M-SEC-AUTHSESS** |
| R-4 | **Metadata exposure.** MLS protects content; who / when / how-much rides in clear — and Leg C of M-RP6.6 now counts exactly those bytes. | **M-SEC-TLS** (partial) |
| R-5 | **Bulk transfers have no cancel, resume or progress**, and pay +33% base64 over WS. | **M-BLOB-PLANE** |

**GATE: `wss://` is a hard prerequisite before the resident dials anything
non-loopback.** Stated as a gate, not a preference.

**⚠️ M-SEC-AUTHSESS is not a technical scoping call.** "What is an identity
authorized to do, and for how long" sits directly on the no-anonymity core. It
is **Joe's to lock**, and it may need spec/doc work before any code. It is
flagged here as possibly large enough to deserve its own grounding session before
it is even scoped.

---

## §4 — Sequence

1. **M-RP6.3** — this milestone (unblocked; loopback-scoped)
2. **M-SEC-TLS** — `wss://` transport + certificate handling *(gate before non-loopback)*
3. **M-SEC-AUTHSESS** — session lifetime, re-auth, auth scopes *(Joe-locked; largest)*
4. **M-BLOB-PLANE** — bulk transfer connections: cancel, resume, progress, binary framing

*Rationale:* TLS before auth-session because it is cheapest and unblocks the
most; auth-session before the blob plane so the blob plane inherits a settled
session model rather than being retrofitted twice.

---

## §5 — Out of scope

- **Visual appearance of every element in Leg C and Leg D — Ms Design's lane.**
  This document specifies *what states exist, what they mean, and when they
  appear*; it specifies **no** colour, type, spacing, glyph or animation choice.
  Note only: the animation should read as "not frozen", not as the loudest thing
  in the stream, and an hourglass is wrong (it implies a *known* wait).
- Widening B beyond the message plane (D1/D2 invariant).
- Real-time media transport.
- Settings-fed tunables (D5, behind J-513).
- The M-RP6.6 ConnStats row-swap (still owed, Ms Design).

---

## §6 — Definition of Done

- [x] Leg A: cap, attempt count, next-attempt-time published and observable
- [x] Leg A: terminal state + `[Reconnect]` + auto-resume, exercised
- [ ] Leg B: non-blocking drain proven under load; ack correlation live
- [ ] Leg B: live ingest wired (the M-RP6.6 deferral closes here)
- [ ] Leg C: R5 wrapped live; gap item counts, hands off to `connecting…`,
      collapses on recovery; grace period exercised with a sub-2s blip
- [ ] Leg D: composer sends live; D6 behaviour exercised during a real outage
- [ ] No message ever renders as delivered when it is not (D6 binding rule)
- [ ] D2 invariant holds: no blob / `send_event_confirmed` / `get_dag_tips` on
      the resident socket — verified against the diff
- [ ] `cargo test` moves off the 1524 floor (the honest signal Rust landed)
- [ ] CDP-verified on real client 9222 + live node, every leg re-driven

*(Per house rule, "commit pushed" is NOT a DoD item — the `Status: COMPLETED`
header is the real signal.)*

---

## §7 — Leg A close (J-545)

**Leg A is CLOSED.** Shipped constants + published surface + terminal park:
`RECONNECT_MAX_ATTEMPTS = 10` · `Backoff::attempt()` / `exhausted()` ·
`ResidentStatus` (the `TrafficCounters` atomics shape) · `get_resident_status`
(the `get_conn_stats` shape, folded into the existing 2 s poll) ·
`resume_resident(source)` + a `watch::Sender<u64>` resume channel (the
`PipeShutdown` shape) · `selfState.resident` · the `resident.reconnect` command
and the `focus` / `online` auto-resume triggers.

**Gates:** `cargo test` **1530/0/62** (floor 1524 + exactly 6 new tests) ·
`vite build` **183** · `npm test` **77** · sampler catalogue **328** unchanged.

**Three things this leg settled that the Phase-0 left open:**

1. **The cap is 10, grounded not chosen.** 181 s of sleeping + up to 10 ×
   `CONNECT_TIMEOUT` ⇒ ≈6 min to terminal. Under D5 it is a named constant
   published as data; migrating it to settings is a one-line change.
2. **`next_attempt_in_ms` is remaining time**, stored internally as an absolute
   deadline and **cleared the instant the sleep ends** — a countdown left armed
   through the dial window would sit at zero and read as frozen (G-4). `None` is
   the signal Leg C hands off to `connecting…` on.
3. **`terminal` is durable only while the app is ignored.** Any `focus` resumes a
   parked resident. This is the intended sleeping-laptop behaviour, but it is
   **not** a general "stop trying" guarantee — Leg C must not render the terminal
   item as if it were permanent.

**Carried into Leg C (appearance is Ms Design's, §5 unchanged):** the
`[Reconnect]` **verb** is live (`resident.reconnect`); its **element** is
deliberately absent, awaiting Leg C — the `layout.revert` precedent, and the
opposite of shipping painted-dead chrome.

**Open, unexplained, instrumented:** during the first live park the resident left
the terminal state with no attributable trigger (`focus`/`online` counters read
zero). The cause is still unknown; `resume_resident` now logs a `source` for
every caller, so a recurrence is self-identifying. Recorded rather than guessed
at (J-545).
