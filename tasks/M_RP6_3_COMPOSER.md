# M-RP6.3 — live messaging: R6 composer + narrow-B send path
> **Status**: ACTIVE  
> Owes: M-RP6.4 room-history backfill · M-RP6.7 resident pong timeout · M-RP6.8 view-latch persistence  
> Version: 2.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-22  
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

> **⚠️ AMENDED IN PLACE (J-546) — THE FIRST CLAUSE IS IMPRECISE, corrected by
> measurement.** `PING_INTERVAL` does **NOT** bound dead-peer detection
> generally. There is **no pong timeout**: `run_session` sets `pending_ping` and
> **never acts on a pong that fails to arrive**. So against a peer whose kernel
> still ACKs TCP — a suspended process, a paused VM, a frozen host — the ping
> **write keeps succeeding** into the send buffer and the resident stays
> **READY indefinitely**. Measured (Clair, J-546): **27 s across 2+ ping
> intervals**, `bytes_out` growing at the ~10 s and ~20 s marks, state never
> leaving READY. **Detection fires ONLY where the WRITE fails or `recv()`
> errors** — i.e. an abrupt kill/RST, not a silent freeze.
>
> *The second clause (`CONNECT_TIMEOUT`) stands unchanged, and the design
> consequence — the countdown must hand off to `connecting…` — is unaffected.*
> **Leg C consequence: a frozen-peer outage produces NO lifecycle transition at
> all, so the gap item will not appear. Do not describe the connection indicator
> as detecting “the node is down”; it detects a broken socket.**

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
- [x] Leg B: non-blocking drain proven under load; ack correlation live
- [x] Leg B: live ingest wired (the M-RP6.6 deferral closes here)
- [x] Leg C1 (`core`): `message-stream` fills its host; divider `now` is live;
      the `status` row kind exists in `StreamRow` — sampler-verified
- [x] Leg C2 (shell): R5 wrapped live; gap item counts, hands off to
      `connecting…`, collapses on recovery; grace period exercised with a
      sub-2s blip
- [ ] Leg D: composer sends live; D6 behaviour exercised during a real outage
- [x] No message ever renders as delivered when it is not (D6 binding rule)
- [x] D2 invariant holds: no blob / `send_event_confirmed` / `get_dag_tips` on
      the resident socket — verified against the diff
- [x] `cargo test` moves off the 1524 floor (the honest signal Rust landed)
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

**⚠️ One Leg-A claim was later found unearned (J-546).** §7's *“terminal state +
`[Reconnect]` + auto-resume, exercised”* stands. But the `attempt:0` reading
that accompanied it was taken after a **terminal park**, whose exit resets the
counter explicitly — so it could not test the *“0 while live”* contract, which
was in fact broken for the ordinary reconnect path. Fixed at J-546. *Recorded
here so §7 is not read as stronger than it was.*

---

## §8 — Leg B close (J-546)

**Leg B is CLOSED.** Outbound multiplexer + ack correlation + live ingest, over
the resident socket (D1), with the drain as single reader **and** writer (D3).

**Shipped:** the drain's outbound `select!` arm (build → sign → write) ·
`event_id → oneshot` correlation reusing `TransportMessage::event_id()` and
`EventConfirm` (D-067, no parallel outcome type) · a 1 s sweep resolving entries
past `SEND_ACK_TIMEOUT` as `timed_out` · teardown resolving every survivor to
`failed` **with a stated reason** · **`TipTracker`** (the B-5 frontier) ·
`send_message` + a control-plane re-anchor on its own short-lived connection ·
live ingest → `xgen-event` → the `$common` `ingest` store · `service.rs` threaded
with an inert message plane (one spine, one code path).

**Gates:** `cargo test` **1541/0/62** · vite **184** · npm **77** · sampler **328**.

### Three things this leg settled

1. **B-5 — tips come from the resident, not from a query per message.** A send
   needs `prev_events`; `get_dag_tips` owns the drain and is D2-forbidden here,
   so there was **no way to build a sendable event over the resident socket at
   all**. Resolved by a resident-held frontier, re-anchored **once per session
   per Space** over a control connection. *Stale tips are not corruption — they
   yield an event that reads as concurrent, which resolution already settles.*
2. **`SendOutcome` is four-way, never a boolean.** `timed_out` is its own
   outcome; folding it into success or failure is the D6 lie.
3. **An `accepted` outcome proves the anchor was VALID, never OPTIMAL.**
   Chaining is observable only in the tips. **Standing rule** — do not read a
   wall of green accepts as proof the frontier is tracked.

### Verification, per seat

| leg | seat |
|---|---|
| V1 accepted round-trip | two-seat (Chat, then Clair on her own inputs) |
| V2 frontier anchoring (`tip_count=1` = msg1) | **Chat, single-seat** |
| V3 live ingest | two-seat, independently re-driven |
| V4 in-flight failure on drop | Clair |
| V5 drain non-stall under load | Clair |
| V6 re-anchor connection hygiene | Clair |

**Honest scope note:** V5 passed under a **sustained trickle**, not under the
40-send burst — that burst was confounded by the re-anchor herd (below). Once the
herd is bounded, a genuinely heavy load test would be cleaner.

### Filed, not fixed

- **Concurrent-first-send re-anchor herd.** N concurrent `send_message` on an
  un-anchored Space → up to N control connections. No leak (V6). Leg D's composer
  sends one at a time, so it is not hit in practice.

### Carried into Leg C

- R5 reads the `$common` `ingest` store (Events **verbatim** — the
  `MessageDescriptor` projection is Leg C's, and must not be duplicated earlier).
- The gap item reads the Leg-A status surface. **`terminal` is durable only while
  the app is ignored** (any `focus` resumes it) — do not draw it as permanent.
- **A frozen peer produces no lifecycle transition at all** (§0 G-4 as amended),
  so the gap item will not appear for that class of outage.

---

## §9 — Leg C Phase-0 (grounded against HEAD, 2026-07-18)

Authority: Joe granted full autonomy on this leg **except** visual appearance
(Ms Design, §5 unchanged) and fundamental architecture (the pragmatic-vs-unified
socket class of question). Every decision below is Chat's, taken under that
grant, and each one is grounded rather than derived.

### §9.1 — The re-ground, six findings

R5's records were five milestones old (M-RP5.6 closed J-485). Re-read against
HEAD: `message-stream.svelte` · `message.svelte` · `data-dependent/types.ts` ·
`stream/grouping.ts` · `$common/plugins/registry.ts` · `layout-default.ts` ·
`rooms-panel.svelte` · `ingest.svelte.ts` · `xgen-common/src/wire.rs`.

**F-1 — there is no R5 widget, and the register path has moved.**
`buildWidgetRegistry` maps `stream → RegionPlaceholder`. The current mechanism is
a `CLIENT_PLUGINS` descriptor (`surface: 'region'`, `regionId`, `component`) that
`layout-default` **derives** the registry from (N-096, one source several
readers). There is no `app_client` register line any more. R5 is therefore a new
`$common` widget plus one descriptor row — the `spaces-panel` / `rooms-panel`
shape exactly.

**F-2 — `message-stream` is boxed.** Its own scoped `<style>` carries
`min-height: 64px; max-height: 340px`. That is a fixture-bench constraint. A
region leaf must FILL its tile and self-scroll (J-499 D5). **This is a `core`
change, and it is not optional.**

**F-3 — `const now = new Date()` is captured ONCE at mount.** Divider labels
freeze for the life of the process. Invisible on a sampler fixture mounted for
thirty seconds; wrong on a resident that runs for days — “Today (Jul 18)” is
still “Today” on the 19th. `core`.

**F-4 — `StreamRow` is a closed union** (`message | divider`). The live mutable
item is a THIRD row kind, and it lives in `grouping.ts` + `message-stream.svelte`.
`core` again.

**F-5 — the ingest store is flat and unscoped.** `IngestEvent[]`,
`INGEST_CAP = 500` **global** across every room, space and event type, with a
**global** `dropped`. R5 shows one room ⇒ a busy Room B can evict Room A's
messages inside the cap, and R5 cannot honestly report completeness per room.

**F-6 — R5 inherits R2's latch problem, one level deeper.** `rooms-panel`
already latches the last **space** because its own click moves the bus to a room
(D3, N-136 avoided). R5's shipped `onSelect` moves the bus to a **message**, so
R5 needs a **room latch** on the identical shape. Nothing else holds “the active
room” — it exists only on the selection bus.

> **⚠️ F-2, F-3 and F-4 are all `core`.** Three `core` edits inside a shell
> milestone is exactly what made the M-RP6.1k registry delta unreadable (the
> `dialog` footer case, correctly filed-not-built). Hence C1/C2 below.

### §9.2 — C-1 · LEG C SPLITS IN TWO

| leg | scope | verified in |
|---|---|---|
| **C1 — `message-stream` region fitness (`core`)** | drop the height box → fill + self-scroll · live `now` for dividers · the third `StreamRow` kind as a SHAPE | **sampler 9422** — catalogue 328 is the readable signal |
| **C2 — `stream-panel` widget + live projection (shell / `$common`)** | new widget + `CLIENT_PLUGINS` row · room latch · projection · gap-item feed + grace | **real client 9222 + live node** |

C1 first, and C2 touches **zero** `core`. A registry delta that mixes a `core`
change with a shell change cannot be attributed to either.

**C2's runbook is deliberately NOT authored yet.** Its anchors are whatever C1
ships, and this arc has already paid twice for a runbook written against a
generation-stale anchor (M-RP6.2; N-116). C2 is authored from C1's close.

### §9.3 — C-2 · THE EVENT → `MessageDescriptor` PROJECTION

Grounded against `xgen-common/src/wire.rs` (≈45 `EventType` variants) and the
shipped body key `content["text"]` (`ai_behavior.rs:110`).

An **explicit allowlist with a `default: ignore` arm.** Not a denylist: a
denylist admits every future protocol type into the stream by default, and this
wire type already ships an `Unknown(String)` catch-all precisely because new
types are expected.

| event type | → |
|---|---|
| `message.text` | `kind: 'text'` — `id = event_id`, `body = content.text`, `timestamp`, `isOwn = sender === selfState xgid` |
| `membership.join` · `leave` · `kick` · `ban` · `node_eject` | `kind: 'system'` centred notice |
| `message.redact` | **not a row** — mutates the referenced descriptor's `deleted` (a shipped M-RP5.5 B state) |
| `message.file` · `message.reaction` | ignore — these are `bodyExtras` / `details`, reserved-unfed (D-065) |
| everything else (`state.*`, `mls.*`, `migration.*`, `bootstrap.*`, `identity.*`, `dm_promote.*`, `thread.*`, `reputation.*`, `system.*`, `Unknown`) | ignore, silently and by design |

`membership.invite` / `mute` / `node_unban` are **out of v1**: none of them is an
arrival or a departure the room witnesses. They are listed here so their absence
reads as a decision rather than an oversight.

> **🔑 C-3 · MEMBERSHIP IS TWO-LEVEL, AND THIS NEARLY BECAME A PHANTOM DEFECT.**
> Grounded, not assumed: `state_key.rs:252–253` builds a **Space** join with
> `room_id = ""` and a **Room** join with a real `room_id`; `derive.rs:1026–1030`
> emits the pair. So a Space-level `membership.join` carries an **EMPTY**
> `room_id` and is **correctly excluded** by a room-scoped filter.
> **Clair's very first live ingest at Leg B was a `membership.join`** — so the
> event class most likely to be driven first at verify is the one the filter
> legitimately drops. **Anyone verifying C2 must drive a ROOM-level join, not a
> Space-level one, or they will read a working seam as a dead one.** This is the
> J-546 fan-out-excludes-the-author trap in a second costume: *zero was correct
> there too.*

### §9.4 — C-4 · PROJECT ON READ, NEVER A MIRROR STORE

R5 takes `messages: MessageDescriptor[]` as a prop and `rows` is `$derived`. The
widget therefore derives its array off `ingest.events`, filtered by the latched
room and mapped through §9.3. No append API, no second store, no reconciliation
— grouping, dividers and the scroll machine all recompute free (proven J-485).

A projected mirror store is **rejected**: it would put the projection in a second
place, which is the exact reason Leg B refused to project at all.

> **⚠️ AMENDED IN PLACE AT §9.11.2 (Leg D Phase-0, J-552).** This lock governs
> **INBOUND** and is unchanged for it. **OUTBOUND** is a named exception: the
> node excludes the author from fan-out *by identity* (`fanout.rs:305`), so the
> user's own sends have **no wire fact to project from** and must come from a
> session-mortal, never-federated echo store. Read §9.11.2 before applying this
> lock to anything the user sent.

### §9.5 — C-5 · THE ROOM LATCH, AND WHAT R5 DOES WITH AN OFF-ROOM EVENT

R5 latches the last `kind: 'room'` selection, on the `rooms-panel` D3 shape: the
effect reads `selection.current` and WRITES the latch, never reads it (the N-136
self-invalidating read-modify-write avoided). An event for another room is
filtered out, stays in `ingest`, and appears if the user switches to that room.

Two honest empty states, distinct copy for distinct truths (N-091):
no room latched → *select a room*; a latched room with nothing projected →
*no messages*.

> **⚠️ AMENDED IN PLACE (J-549) — BOTH EMPTY STATES ARE NOW WIDGET-COMPOSED.**
> C-9 (§9.10) prepends a permanent head-marker row, and `message-stream`'s own
> empty branch is `showEmpty = count === 0 && !backgroundDeclared`. A
> permanently-present marker makes `count ≥ 1` forever, so **`core`'s "No
> messages yet" can never fire again from this widget.** The second empty state
> is therefore a **second synthetic `system` row** the widget composes, one
> level up from where C1 put it. *The truths and the distinct-copy requirement
> are unchanged; only who renders them moved.*

### §9.6 — C-6 · ⚠️ THERE IS NO BACKFILL, AND R5 MUST SAY SO

Switching to a room shows only what arrived over the resident **this session**.
There is no history load anywhere in the client. **An empty room and a
not-yet-loaded room therefore render identically**, which is a message-loss
illusion in the D6 family — *a stream may never look complete when it is not.*

**Decision:** the stream carries a **head marker row** — always present when a
room is latched, always the first row — stating that the view begins at session
start. It has **two states**, and the second is what makes it earn its keep:

1. normal — the view begins at session start;
2. `ingest.dropped > 0` — *and part of this session was discarded* (F-5).

This is the cheapest honest answer to F-5 without restructuring the store, and
the marker **is deleted, not softened, by the milestone that lands backfill** —
per N-109, its removal is written into that milestone's DoD, not left to be
noticed. Its *appearance* is Ms Design's; its *existence and meaning* are
specified here.

History load is filed as **M-RP6.4 — room history backfill**; it is not scoped
here and nothing in Leg C reserves a slot for it.

### §9.7 — C-7 · THE LIVE ITEM: WHERE IT LIVES, AND WHAT IT BREAKS

- The item is a **third `StreamRow` kind** (C1), NOT a `MessageDescriptor` with
  `kind: 'system'`. A system message is immutable, carries an author-shaped
  descriptor and participates in grouping; this row is **mutable, collapses in
  place, and carries a live countdown**. Forcing it through `MessageDescriptor`
  would make `computeRows` break grouping runs as a side effect of a connection
  event — a coupling nobody asked for.
- **It breaks a grouping run.** A continuation rendered across a visible
  disconnect reads wrong.
- **The ~2 s grace timer lives in the widget** (C2), not in `core` and not in
  Rust. The Leg-A status surface already publishes the transition; the widget
  starts the timer on leaving `READY` and materialises the row only if it fires.
  Nothing is added to `resident.rs`.
  > **⚠️ AMENDED IN PLACE (J-549) — THE TIMER AND THE EPISODE MOVE TO A
  > `$common` STORE (C-10, §9.10).** A widget-local tracker **loses all outage
  > history when the tile is folded, the layout changes, or the plugin is
  > toggled**, and restarts the grace timer on remount — a still-live outage
  > would blink out and return two seconds later. A second mount would give two
  > views of one connection two disagreeing stories. **The binding half of this
  > clause — not `core`, not Rust, nothing added to `resident.rs` — is
  > unchanged.**
- **Copy constraints, binding (§0 G-4 as amended):** the row describes a **broken
  socket**, never “the node is down” — a frozen peer produces no transition at
  all. And the terminal state is **not permanent**: any window `focus` resumes a
  parked resident, so it may not be drawn as a dead end.

### §9.8 — C-8 · AUTHOR NAMES SHIP ABSENT

Nothing in the client resolves an XGID → display name: `spacesState` carries no
members and R7 is unbuilt. So `author = { kind: 'identity', id: sender }` with
**no `name`**, and `entity-avatar` falls back to its xgid-tail initials — the
shipped, already-verified path.

This is the correct render, not a gap to paper over (W-8). It is stated here so
Ms Design knows the stream ships **initials-only** author avatars, and so nobody
fabricates a name map to make the panel look finished (the J-501 rule: *do not
invent fields to make a panel look substantial*).

### §9.9 — Filed here, built elsewhere

| item | successor |
|---|---|
| Room history backfill (the C-6 marker's discharger) | **M-RP6.4 — room history backfill** |
| Per-room ingest scoping / cross-room cap eviction (F-5) | **M-RP6.4** (same store touch) |
| Resident pong timeout — frozen-peer detection (§0 G-4) | **M-RP6.7 — resident pong timeout**, FILED, unscoped, Joe's to schedule |
| Concurrent-first-send re-anchor herd | unchanged, filed at §8 |
| XGID → display-name resolution (C-8) | R7 members / **M-RP-PLUGINS-NODE** |
| ~~**`IngestEvent.event_type` → `type`**~~ — **DONE (J-551)**, Chat micro-change ahead of Leg D. Three readers, not one: the declaration, `wireType()`'s fallback (which stopped type-checking, not merely stopped being load-bearing), and two stale comments + a test title. `wireType()` and its `event_type` fallback are KEPT. | **closed — J-551** |
| **No `ui/**` package runs a typecheck** — so no gate can fail on a type error; the frontend floors prove runtime + module graph only (N-138) | **M-RP-TYPECHECK — admit a typecheck to the frontend gate set**, filed, unscoped, Joe's to schedule |
| **View-latch behaviour across remount + the unattributed bus move** (§8.8/§8.9 of the C2 runbook) | **M-RP6.8 — view-latch persistence** |
| Per-message send state (“not delivered yet” / pending / failed) — C2 has no outbound rows | **Leg D**, via `MessageDescriptor.details` |

**M-RP6.7 is filed, not started.** Filing an arc is not deciding to build it;
the frozen-peer gap is named so Leg C cannot paper over it, per §0.

---

## §9.10 — Leg C2 Phase-0 (grounded against HEAD, 2026-07-19; Joe-locked)

Three questions were grounded against HEAD before the C2 runbook was authored,
because a `core`-shaped answer to any of them would have changed the
milestone's shape and finding that out after the runbook is the expensive
order. **All three came back shell-only. There is no C1b.**

### §9.10.1 — C-9 · THE HEAD MARKER IS A SYNTHETIC `system` DESCRIPTOR

**The question:** C-6's head marker is neither a message nor a divider, and
“the view begins at session start” is not one of C1's four status phases. A
fifth phase or a fourth `StreamRow` kind would be a `core` change C2 is
forbidden to make.

**Grounded** in `data-dependent/types.ts`: `MessageKind = 'text' | 'system'`,
and `system` is an **authorless centred notice** — `author` optional, `body`
optional, `id` free. §9.3 already routes `membership.*` through that render, so
it is a **shipped, verified path**, not a new one.

**LOCKED:** the head marker is a synthetic `MessageDescriptor{kind:'system'}`
that the **widget prepends** to `messages`. Zero `core`. `computeRows` needs no
change — a system row already breaks a grouping run, and as row 0 it breaks
nothing.

Four consequences, stated rather than discovered:

1. **It makes `core`'s empty state unreachable** → C-5 amended in place (§9.5).
2. **⚠️ Getter `count` is no longer the message count** — it becomes
   `projected + 1` (or `+2` when empty). Anyone verifying C2 must subtract, or
   an empty room reads as two phantom messages. *The J-548 hidden-element
   family: a number that is right about the wrong quantity.*
3. **Reserved id prefix** (`__head__`) so a synthetic row can never collide with
   an `event_id` and `onSelect` can filter it.
4. **A session crossing midnight** puts a day divider between the marker and the
   first message. **That is the truthful render, not a bug.**

*Rejected:* a leading `divider` row (`computeRows` mints dividers itself —
injection needs a `core` prop) and a fifth `status` phase (`core`, forbidden).

### §9.10.2 — C-10 · EPISODE IDENTITY IS MINTED CLIENT-SIDE, IN A `$common` STORE

**The question:** C1's V6 depends on a stable `id` across a phase transition,
and `resolvedAfterMs` is retrospective. Neither has a source in the resident.

**Grounded:** `ResidentStatus` is exactly
`{ attempt, max_attempts, next_attempt_in_ms, terminal, connect_timeout_ms,
ping_interval_ms }`. **No episode id, no gap start time, no outage duration.**
The lifecycle state arrives on a separate channel (`selfState.connection`).

**LOCKED:** a new `$common` store (`gaps.svelte.ts`) mints episode identity,
records the start time, owns the grace timer, derives the four phases, and
publishes a `StreamStatus[]`. **This amends C-7's “in the widget” clause**
(§9.7, amended in place); C-7's binding half — not `core`, not Rust, nothing
added to `resident.rs` — is untouched.

**The single-writer rule, the `tickNow` lesson applied forward:** ONE function
ingests an observation; the reactive effect calls it and the DEV hook exposes
**that same function**. No setter that injects a finished episode — *a verify
seam that skips the mechanism verifies the wrong thing* (J-548).

**Honest limits, binding on the close:** episode START is **event-timed** and
accurate; the countdown **numbers are poll-sampled at 2 s** and may be that
stale — say which is which. A **frozen peer produces no transition, hence no
episode, hence no row** (§0 G-4 amended). `terminal` is **not permanent**
(any `focus` resumes a parked resident). Episode history is **session-scoped
and in-memory** by design.

### §9.10.3 — C-11 · THE RESOLVED ROW MARKS A DISCONTINUITY, NOT AN ALL-CLEAR

There is no backfill, so messages other people sent **during** a gap are gone
and nothing will fill the hole. A resolved row meaning only *connection
restored · 8s* is true and **incomplete** — the same D6 rule as the head marker
(*the stream may never look complete when it isn't*), applied to the middle of
the stream instead of the top.

**LOCKED:** the resolved episode row asserts **“the record is discontinuous
here.”** It costs nothing — `resolvedAfterMs` already ships and the row already
renders in the right position. **Wording and appearance remain Ms Design's;
only the meaning is locked here.**

**Filed, NOT built → Leg D: per-message send state.** “Not delivered yet” /
pending / failed indicators belong on **outbound** rows, and C2 has none: every
projected row is inbound (delivered by definition) and the node excludes the
author from fan-out, so your own sends never ingest. `MessageDescriptor.details`
is the reserved socket (its own comment names *send-status led*) and Leg B
already ships the four-way `SendOutcome` behind it.

### §9.10.4 — F-6 CORRECTED: THE LATCH IS SAFER THAN J-547 ASSUMED

F-6 recorded that “R5's shipped `onSelect` moves the bus to a **message**.”
**That is not true at HEAD.** `message-stream.svelte` sets its own `$bindable
selected` and calls `onSelect?.(mid)` — an explicitly **reserved hook**. It
never writes the selection bus; the consumer decides.

**LOCKED:** C2 does **not** wire `onSelect` to `selection.set`. The room latch
is then a byte-for-byte copy of the `rooms-panel` D3 shape with `'room'`, and
the N-136 self-invalidating read-modify-write is **not merely avoided — it is
unreachable**, because nothing this widget does moves the bus off a room.

**And a message selection is not expressible on the bus today:**
`EntityDescriptor.kind` is `'identity' | 'space' | 'room'` — there is no message
kind. Wiring R5 to the bus is therefore a `core` descriptor-union change, not a
deferred C2 decision. **Filed, not built.**

**Filter scope:** `room_id` alone is sufficient — room ids are hash-derived
`xgen://` globals, so `space_id` is redundant. No space latch this leg.

---

## §9.11 — Leg D Phase-0 (grounded against HEAD, 2026-07-19; Joe-locked)

> ⚠️ **RE-GROUND NOTE (J-559, added after M-RP6.9 and M-RP-TYPECHECK landed).** The **substance of
> §9.11 stands**; **four references below have shifted and one section is in the wrong tense.**
> Read the substance, not the line numbers:
> - `resident.rs:706` (the outcome struct) → now **`743–761`**
> - `fanout.rs:305` (author exclusion by identity) → now **`303` · `1067` · `1116`**
> - `desktop.rs:331` (`try_send`) → now **`346`**; the bounded wait is **`363`**
> - `message.svelte:71` (widget resolution) → now **`:56`** (`widgets?: Record<string, Component>`) and **`:63`** (`isOwn`)
> - `message.svelte:29` — *"`bodyExtras` declared but never rendered"* is **SUPERSEDED: M-RP6.9 BUILT IT** (J-556)
>
> ⚠️ **AND §9.11.4 IS WRITTEN IN THE PRESENT TENSE ABOUT A BUG THAT IS CLOSED** — see its own
> heading note. **Do not re-fix it.**

Driven before any runbook, per the standing order that a `core`- or Rust-shaped
answer changes the milestone's shape and finding that out mid-runbook is the
expensive order. **Unlike C2, the answers did NOT come back shell-only.**

### §9.11.1 — Two corrections to the brief, both material

**The four-way is `accepted` · `rejected` · `timed_out` · `failed`**
(`resident.rs:706`, a struct: `{event_id: Option<String>, status: &'static str,
code: Option<u32>, reason: Option<String>}`). The session brief listed
*in-flight* as the fourth. **In-flight is not an outcome — it is the state
before one exists**, and `rejected` is a real one: a deterministic node refusal
carrying a wire `code`, a message that will NEVER arrive no matter how long the
user waits. That is a different thing to tell a user than *timed out*.

**`event_id` arrives WITH the outcome, not before it.** `send_message` awaits
the correlated reply and resolves once. At the moment the user presses Enter the
frontend does not know the `event_id`, so **an echo cannot be keyed by it at
creation** — the brief's proposed shape is not constructible. **LOCKED: the echo
is keyed by a CLIENT-MINTED local id; the real `event_id` is stitched on when
the outcome returns.**

### §9.11.2 — D-1 · C-4 IS AMENDED: OUTBOUND GETS AN ECHO STORE

**The problem, grounded:** the node excludes the author from fan-out **by
IDENTITY, not by connection** — `fanout.rs:305`, `EV-D2`, in terms. Combined
with C-4 (`stream-panel` projects ONLY `ingest`, on read), a send round-trips,
returns `accepted`, and **the stream does not change.** Your own words render
nowhere.

**⚠️ The brief proposed "a mirror with a defined death — dropped when the real
event arrives." THE REAL EVENT NEVER ARRIVES.** There is no server-sent fact to
reconcile your own messages against, ever. The echo is the sole record they
exist, for the whole session.

**LOCKED — C-4 amended in place, narrowly:** C-4 continues to govern **inbound**
unchanged (project on read, no mirror, no reconciliation). **Outbound** gets a
separate, explicitly **session-mortal, never-federated** echo store in
`$common`. Not a general mirror — a named exception with a stated reason and a
stated death. C-4's original rationale (do not put the projection in two places)
is untouched: there is still exactly one projection path per source of truth.
*The awkwardness — a mirror that never dies within a session — becomes a
DOCUMENTED property rather than a leak.*

### §9.11.3 — D-2 · THE TWELVE USER-FACING LOCKS

> ### 🔒 **THE VERIFIER COLUMN — ADDED AT `M-RP-LOCK-RECHECK` LEG C (J-574, 2026-07-22). NO LOCK IN THIS TABLE IS WITHOUT A NAMED VERIFIER.**
> 🔑 **WHY THE COLUMN EXISTS.** J-560 closed Leg D2 claiming *"ONE LOCK UNMET"*. **Two were.** And **lock #5 appeared nowhere in that close at all** — not met, not unmet, not deferred. ⇒ **A LOCK WITH NO VERIFICATION LEG IS UNFALSIFIABLE: it cannot fail, so it cannot be trusted, and a green milestone says nothing about it either way.** ⚠️ The pass filed to investigate that then found **#11 in the same condition** — *one is an oversight; the second, found by the pass filed to investigate the first, is a mechanism.*
> **MACHINE verdicts** were driven at `b477bae` (Leg A, J-569), **each with a positive control** — *"the bad thing is absent" and "nothing happened" are the same string.* Every probe and its control is tabulated at `tasks/M_RP_LOCK_RECHECK.md` §10.
> **EYE verdicts** are Joe's alone and are quoted verbatim at `tasks/M_RP_LOCK_RECHECK.md` §11. ⚠️ Both were **RE-CONFIRMED on 2026-07-22, NOT captured live** — judged in session, write-up lost to an MCP failure, re-confirmed afterwards. *Recorded as recovered because a verdict from memory and a verdict from a live reading are the same sentence carrying different reliability.*
> ⚠️ **A VERDICT IN THIS TABLE IS NOT A WIRE PROOF.** Two Leg-A legs drove outcomes through a **stub transport** (`echo.setTransport`). That verifies the store and the render rules and **verifies nothing about the wire** — and no lock among the twelve claims the wire.

| # | lock | verdict | verifier |
|---|---|---|---|
| 1 | A local echo exists. C-4 amended (§9.11.2). | ✅ MET | **machine** — `b477bae`, Leg A (J-569): echo count 0→1 read in the SAME eval as the send click, so the row exists before the network is consulted. *Control:* pre-click 0 measured three times. |
| 2 | The echo lives in a **`$common` store**, not the widget — the C-10 argument, stronger here: a lost outage row is an omission; a lost sentence you just typed is the app eating your words in front of you. | ✅ MET | **machine** — `b477bae`, Leg A (J-569): no widget holds an echo array; the store is `$common/stores/echo-state.svelte.ts`. *Control:* the grep proven live by finding 8 other `$state` decls in the same widgets. |
| 3 | Keyed by a **client-minted local id**; `event_id` stitched on at outcome. | ✅ MET | **machine** — `b477bae`, Leg A (J-569): same `localId` across `pending`→`accepted`, `eventId` ABSENT→stitched. *Control:* the ABSENT pre-state was measured, so the stitch is a transition, not a pre-existing value. |
| 4 | The echo's timestamp is **client-minted and stays that way** — see §9.11.5. | ✅ MET | **machine** — `b477bae`, Leg A (J-569): `sentAt` byte-identical before and after the outcome. *Control:* the same outcome demonstrably changed OTHER fields on that row — invariance, not a dead read. |
| 5 | **Self is special-cased**: the user does not see their own six-char hash tail. `isOwn` already ships. Wording/appearance = Ms Design. | ❌ **UNMET** | **SPLIT — both halves now named.** **machine** — `b477bae`, Leg A (J-569): own rows render the FULL XGID as the author name (rendered name === self `identity_id`, 65 chars, byte-equal), avatar `name:null` / `initials:"GC"`. **eye — Joe, 2026-07-22 (RE-CONFIRMED):** the name becomes **"Self"** — default, customisable later, visually distinguished. ⚠️ **FIX FILED TO `M-RP-SELF-SURFACE`; NOT DONE HERE.** |
| 6 | Send-status has **THREE visual states, not two**: sent (`accepted`) · **unresolved** (`timed_out`) · not sent (`rejected` + `failed`, same state, different copy). Collapsing `timed_out` either way is the D6 lie verbatim. | ✅ MET | **SPLIT — both halves now named.** **machine** — `b477bae`, Leg A (J-569): four outcomes → three tones + `pending`; registry getter `tone` === painted `data-tone` on all four; labels distinct strings. **eye — Joe, 2026-07-22 (RE-CONFIRMED), verbatim:** *"they read as 3 states"*. |
| 7 | **Retry policy by status:** `failed` → retry freely (never reached the wire) · `rejected` → no retry (it will be refused again) · `timed_out` → **no retry affordance at all**. ⚠️ **AMENDED IN PLACE AT LEG C (J-574).** This row previously read *"`timed_out` → retry only behind an explicit warning, because the node may hold it and the user may double-post."* That was **narrowed deliberately at D2 §3.1** and shipped as **no retry**, enforced in BOTH the store's refusal and the widget's button (one predicate, N-126). **The behaviour was always right; the table never received the amendment** — the J-566 shape again, *a decision applied in code and not in the record.* | ✅ MET | **machine** — `b477bae`, Leg A (J-569), **measured against the shipped narrowing, not against the stale text**: retry offered on `failed` only, refused at the STORE as well as the widget. *Control:* a bypass call on `rejected`/`timed_out` resolved `null` + unchanged, while the SAME call on `failed` MUTATED it. |
| 8 | The echo dies at **exactly one stated moment**: session end / reload. **The C-6 head marker must cover the user's OWN sends**, or its confession is partial. *They never read the messages they lost; they wrote the one they lost.* | ✅ MET | **machine** — `b477bae`, Leg A (J-569): no persistence path reaches the store; head marker names own sends; live echo count 4→0 across a reload. *Control:* the 4 measured immediately prior. |
| 9 | Echoes are real `MessageDescriptor`s, so **grouping and dividers come free**. | ✅ MET | **machine** — `b477bae`, Leg A (J-569): `groupedCount: 12`, `dividerCount: 0`. *Control:* the "missing" 13th is ARITHMETIC, not hand-waving — two group heads 425,026 ms ≈ 7.1 min apart, past the 5-minute window. |
| 10 | **Auto-scroll on your own send, always** — the one action where it is unambiguous. | ❌ **UNMET — accepted deviation** | **machine** — `b477bae`, Leg A (J-569): scrolled to top, sent, `scrollTop` stayed 0 / `atBottom` false. *Control:* the row demonstrably landed (count 15→16, max 639→705) **and** the probe demonstrably sees scroll (639.2→0). ⚠️ **Joe accepted this deviation at J-560 with his eyes open; re-confirmed, NOT re-tested into a pass.** |
| 11 | **N windows, one device** — consistent, falls out of #2. Not "two": stated as N so nobody special-cases a pair. | ✅ **MET at VIEW scope** | **machine** — `b477bae`, Leg A (J-569): the whole tile grid unmounted and all 4 echoes survived. *Control:* `composerMounted` 1→0 and the grid emptied, so the unmount really happened. **Scope fixed by D-122 (J-571 / J-572)**, not by editing this lock. ⚠️ **BOUNDARY, RECORDED BECAUSE IT IS NOT OBVIOUS: this does NOT extend to a separate window.** Module `$state` is per-webview and #8 makes the echo session-mortal, so two separate windows are **two independent stores** — **#8 and #11 CONTRADICT at separate-window scope**, and #11 is true only at the scope named here. |
| 12 | **No room latched ⇒ typing yes, sending no.** Silently accepting a sentence that goes nowhere is the worst of the three options. | ✅ MET | **machine** — `b477bae`, Leg A (J-569): with no room, typing yes (`disabled:false`, 18 chars accepted) and sending no (button `disabled:true`). *Control:* latching a room flipped ONLY the button — the draft survived at 18 chars, so the refusal is caused by the latch. |

⚠️ **TWO THINGS THIS PASS DELIBERATELY DID NOT DO.**
- **#11's wording was NOT edited.** Leg A found the lock unfalsifiable at OS-window scope and offered a rewording; **Joe took a different route** — D-122 fixed the *vocabulary*, and the lock resolves at VIEW scope with its boundary recorded above. *Rewording a lock is a records decision about Joe's own design, and was not Chat's to take.*
- **#5 was NOT fixed, and #10 was NOT reopened.** *A verification pass that also fixes things cannot tell you whether it verified or created.*

📌 **FILED, NOT FIXED — AN ORPHANED APPEARANCE OWNER SITTING INSIDE ROW #5.** That lock's text still assigns *"Wording/appearance = Ms Design"* — a seat **retired at J-568** and superseded by **D-123**, under which appearance is Joe's. The row is left **verbatim** because editing design text is not Chat's call. ⇒ **candidate for `M-RP-SEAT-ORPHANS`**, where ten such items are already known and *ten is a floor*.

### §9.11.4 — ✅ **CLOSED AT LEG D1 (J-553, `53dab37`) — READ IN THE PAST TENSE** · ⚠️ D-3 · THE SILENT QUEUE WAS ALREADY SHIPPED. LEG D IS NOT ZERO-RUST.

> ✅ **FIXED.** `desktop.rs:363` now calls `await_send_outcome(reply_rx, SEND_QUEUE_TIMEOUT)`, and
> `SEND_QUEUE_TIMEOUT` is **DERIVED** (`SEND_ACK_TIMEOUT + PENDING_SWEEP_INTERVAL + 5`) with a test
> asserting it outlasts the drain's worst case. **The description below is of the pre-D1 state and is
> retained as the reasoning that produced the fix — it is NOT a live defect. Do not re-fix it.**

**`send_message` never checks link state.** It `try_send`s onto the outbound
channel (`desktop.rs:331`) and awaits the reply. And `io.outbound_rx.recv()` is
polled **only inside `run_session`'s `select!`** (`resident.rs:418`). Between
sessions — i.e. for the entire duration of an outage — **nothing drains that
queue.**

So a send during an outage: `try_send` succeeds · **the promise never resolves**
· the message sits queued · and when the link returns the drain pops it and
**sends it**, minutes later, unannounced.

**`SEND_ACK_TIMEOUT` (10 s, swept at 1 s) does not save this** — it ages only
sends already WRITTEN and awaiting an ack. A queued-but-never-written send has
**no timer at all**.

***That is precisely the silent queue D6 forbids, live at HEAD, plus a promise
that never resolves.*** Leg B is scrupulous about D6 everywhere it writes
(`resident.rs:449`, *"Never written → say so"*) — the gap is the state where it
never gets to write. **Nobody was careless: it is the seam between two correct
pieces, and it only becomes reachable once a composer exists to hit it.**

**LOCKED (D1):** bound the wait in `send_message` —
`tokio::time::timeout(SEND_ACK_TIMEOUT, reply_rx)` — and on expiry return
**`failed`**, NOT `timed_out`. `failed` means *never reached the wire*, which is
exactly true of a send that was never drained; `timed_out` would claim the node
may hold it, which is the D6 lie inverted. A frontend lifecycle guard blocks the
common case, **but the guard is the nicety and the timeout is the guarantee**
(§0 G-4: a half-open socket defeats any guard).

***Rejected: draining the queue on session teardown and failing everything in
it.*** Tidier, but it destroys a message the user may still want offered under
D6's explicit *"send when reconnected"*.

**Consequence: C2's zero-Rust shape does NOT repeat. Leg D changes `.rs`, and
`cargo test` moves off 1541 by design.**

### §9.11.5 — D-4 · THE SOCKETS, AND WHY SEND-STATUS IS NOT HEADER FURNITURE

**`details` is live, not reserved-unfed** — `message.svelte:71` resolves each
mount against a consumer registry, drops unknown ids (W-13), and `detailsCount`
reports RENDER truth. `message-stream` takes `widgets?: Record<string,
Component>` from the shell. So a status widget needs **zero `core`** to mount.

**⚠️ But grouped continuation rows suppress the whole header line — name AND
`details`** (`message.svelte:149`; grouping is decided in `core`'s
`grouping.ts`: same author, within `GROUP_WINDOW_MS` = 5 min, no divider
between). **Three sends in a row ⇒ rows 2 and 3 cannot show send-status. A
`failed` third message would look exactly like the delivered first one.** D6's
mirror, broken by a `core` render rule, in the most common send pattern there
is. *Locks #6 and #9 were in direct tension and this is where it surfaced.*

**REJECTED: perforating grouping so `details` survives.** Suppression is
**correct** — repeating the author's name and time on every row of a run is
precisely what grouping exists to remove. Punching a hole in a right rule to
admit one tenant is the wrong repair.

**LOCKED: send-status belongs in `bodyExtras`, and the socket gets built.**
`bodyExtras` sits BELOW the body, **outside** the `{#if !grouped}` block, so it
is **grouping-immune by position** — and `types.ts:69` already names it for
*attachments / reactions*. It is declared but **never rendered**
(`message.svelte:29`), so this is a `core` change — but it is **building a
socket that was always designed**, not perforating a rule that was always right.

**`types.ts:67–69` is corrected as part of this:** *send-status led* moves from
the `details` list to `bodyExtras`, with the reason recorded. That line was
written before a composer existed, when nobody had asked whether send-status is
**header chrome or per-row state**. It is per-row state — the same category as a
reaction, not the same category as an author name.

**⚠️ NO PROVISIONAL-TIMESTAMP FIELD, and the reason is not cost.**
`MessageDescriptor.timestamp` is a bare `string` with no provenance, and the
message formats its own. Marking it provisional is a `core` change — but
**`SendOutcome` carries no timestamp**, so even on `accepted` there is nothing
to correct the time TO. *A provisional marker that can never resolve is worse
than none.* Provenance rides the status widget instead. **FILED:** `accepted`
returns no authoritative timestamp, so a user's own rows keep a client-minted
time for the session and **may order differently for them than for everyone
else in the room**. Real, permanent within a session, not Leg D's to fix.

### §9.11.6 — D-5 · THE LEG SPLIT, AND WHY A MILESTONE IS INSERTED

**Joe's call (2026-07-19):** build the `bodyExtras` container **fixture-driven
first**, before either tenant needs it — the R5 precedent exactly (M-RP5.6 built
the stream on fixtures; Leg C wrapped it live four milestones later).

**The argument, and it is Joe's:** a socket designed against a single static led
will hold a single static led. **Reactions are the far richer consumer** — N
mounts per row, added and removed at runtime, interactive, on continuation rows.
Design against the strong consumer and send-status is trivially a second tenant;
design against the weak one and it is paid for twice. **And this is the cheap
moment**: one message component, one stream, one registry.

**What the build teaches that a design walk cannot:** `cid()` behaviour under
**N mounts × M rows**. The client registry has sat at **134 quiescent** through
C2 and every verify leg on this arc keys off it. Three mounts on twenty rows is
sixty new ids — **or sixty collisions**, and which one is not known. Answer it
on a fixture bench, not during a composer drive where it would look like the
composer's fault. Same for W-13 drop-unknown when socket membership changes at
runtime: `details` has only ever been handed a static list.

| leg | tier | scope |
|---|---|---|
| **D1 — send-path honesty (Rust)** | `.rs` | §9.11.4. **First**, because it is a LIVE D6 violation and depends on nothing else here. |
| **M-RP6.9 — `bodyExtras`: the per-row message container (`core`)** | `core` | Sampler-fixture-driven. **ZERO protocol, ZERO store, ZERO federation.** |
| **D2 — R6 composer + echo store (`$common`)** | `$common` | §9.11.2 + §9.11.3. User value lands here. |
| **D3 — send-status widget** | `$common` | Second tenant of a proven container; `details`-vs-`bodyExtras` already answered by measurement. |

**⚠️ THE FENCE ON M-RP6.9 — BINDING.** The container renders `WidgetMount[]` and
**never learns what a tag or a reaction IS**. The sampler passes fixture mounts;
a future arc passes real ones; **the component does not change between those two
events.** That is what makes it a container to complete rather than a stub to
replace.

- **Fixtures live in the SAMPLER only.** In the real client the container renders
  **nothing** until something feeds it. A client rendering invented tags is fake
  data on screen — N-091 / D-065, the thing this project refuses most
  consistently. The sampler catalogue moving off **386** is the honest signal it
  shipped.
- **Fixture assets are locally bundled, never remote URLs** — so not even a
  fixture establishes the precedent **D-111** forbids.
- **NO `ReactionDescriptor`, no wire shape, no protocol, no attribution.** *A
  data shape invented before the protocol exists is a shape the protocol then
  has to satisfy* — and **who reacted is identity data, which sits on the
  no-anonymity core: Joe's.*

### §9.11.7 — ⚠️ #11b · MULTI-DEVICE: NAMED, NOT SOLVED, AND NOT LEG D'S

Author exclusion is **by identity** (`fanout.rs:305`), and the loop below it
delivers to **every connection** of each recipient. So other people's messages
reach all of their devices — **and your own reach none of yours.** The echo is
process-local; a second device is a second process with its own resident. You
type on the laptop and **the sentence does not exist on the phone.** Not "until
reload" — at all.

**Not solvable in Leg D and not pretended otherwise.** It is the intersection of
EV-D2, the no-backfill window (C-6) and one identity spanning devices — the last
of which sits on the no-anonymity core. **Joe: "this awaits us from day one, and
I keep it in mind the whole time."**

**The encouraging half:** the node DID persist the event. Fan-out is a
live-delivery optimisation, **not the record.** A device that can ask the node
for room history sees the user's own messages perfectly well — so **the
multi-device hole closes at M-RP6.4, not at Leg D.** **BINDING CONSTRAINT ON
M-RP6.4, written now while it is still free:** backfill reads the EVENT STORE
and must return the requesting identity's OWN events. *If backfill were ever
built by replaying fan-out, the hole would silently persist.*

### §9.11.8 — Filed here, built elsewhere (Leg D additions)

| item | successor |
|---|---|
| Blob-backed custom reactions — custom sets, animated, above 16px, not unicode | **M-RP-REACTIONS**, filed unscoped, no design record; discussion deferred to the real interface |
| `accepted` carries no authoritative timestamp ⇒ own rows may order differently than for others | filed, unscoped |
| Multi-device self-visibility (§9.11.7) | **M-RP6.4**, with the event-store constraint above |
| **⚠️ Outage send latency is `re-anchor + SEND_QUEUE_TIMEOUT`, NOT the bound alone** — measured **19 155 ms** at Leg D1's V-L, decomposed as **3.143 s failed re-anchor + 16.013 s bounded wait** (the bound itself accurate to 13 ms). **Leg D2's lifecycle guard should normally stop the call reaching this path at all** | **Leg D2** — carry it, so nobody later reads ~19 s as a bug in the bound |

