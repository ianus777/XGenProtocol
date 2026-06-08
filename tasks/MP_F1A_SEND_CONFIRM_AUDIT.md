# MP-F1a — client send-confirm (multi-event delivery) — D-071 PHASE-0 AUDIT

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

Phase-0 grounding only — **no code, no design**. This audit grounds the client send path
against live `main` + the empirical traces captured during the MP-F1 facet-2 grounding pass
(J-327), confirms the gap, surfaces the forks with recommendations, and runs the D-076
convergence check. The design is authored only after Joe locks the forks (§8).

MP-F1 was split (Joe, J-327): **MP-F1a** = client send-confirm (facet-2, this audit);
**MP-F1b** = cross-node DM federation (facet-1, resolution (iii), separate Phase-0 next).
This audit is F1a only. F1a does **not** fix facet-1 (see §5).

---

## 1. The gap (confirmed empirically, J-327)

`ops::create_dm_space` sends **three** DAG events (`state.dm_space_create` →
auto-`state.room_create` → auto-`membership.invite`) over **one** connection, fire-and-forget
(`Connection::send_event`, no ack awaited), then `goodbye` + drops the connection. On Windows
localhost the abrupt teardown (TCP RST → `WSAECONNABORTED 10053`) discards the node's
buffered-but-unread frames, so the node reads + processes **only event 1**; the room + invite
are **never read** (node-side trace: `room_create`/`membership.invite` count = 0 across both
nodes). With the room absent, the subsequent DM `message.text` is rejected at
`validate_event` step 11 ("sender is not a member of room …") — which is the surface MP-F1
facet-2 recorded as "DM messages absent from `.events`".

**Root cause:** the client never awaits the node's `EventAccepted` (D-070) before sending the
next event or closing. Fire-and-forget borrowed against a confirmation it ignored. The race is
**deterministic** for the multi-event `create-dm-space` (event 1 processed slowly — dispatch +
disk persist + ack + fan-out — while events 2–3 are RST-discarded), and **latent** for every
single-event op (their one event survives only because it is read before the abort —
timing-dependent, not guaranteed under load).

**Fix validated this session:** a throwaway (A)-prototype (`create_dm_space` awaits the
per-event signal before next-send + before goodbye) made **all three events land** — node A
then carried `dm_space_create + room_create + invite + alice's message`, and alice's message
flipped from REJECTED to applied (in `.events`). So the confirm mechanism is proven end-to-end.

---

## 2. Surfaces grounded (live `main`)

- **`xgen-client/src/ops.rs`** — 11 `send_event` sites across 9 ops. `create_dm_space` is the
  **only** multi-event-per-connection op (3 sends); all others send exactly 1
  (`create_space`/`create_room`/`send`/`join`/`leave`/`invite`/`ai_delegate`/`ai_revoke`),
  `register` 0. Every op is `send_event …` (fire-and-forget) then `goodbye`.
- **The D-070 ack already exists.** `process_inbound` (`xgen-node/src/app.rs`) on a
  LocallySubmitted event sends `TransportMessage::EventAccepted { event_id }` on accept and
  `TransportMessage::Error { event_id, error_code }` on reject — both carry the originating
  `event_id` for correlation. **No new node wire surface is required for F1a** (caveat: F-3
  HeldPending, below).
- **Ack ordering is correct for chains.** `accept_signal` is sent **after** persist, before
  `apply_fanout` (app.rs comment: "local fan-out has not yet begun"). So when the client holds
  event N's ack, event N is durable + in the store → event N+1's predecessor check passes. The
  `create-dm-space` chain (dm_space ← room ← invite) is **self-protecting** under await-ack: no
  event goes HeldPending because its predecessor is acked-present before it is sent.
- **Signal coverage per `DispatchOutcome`:** Accepted → signal; Duplicate → signal (idempotent
  ack, F3-D3); Rejected → signal; **HeldPending → SILENT** (no `accept_signal`/`reject_signal`).
  HeldPending is the one outcome a confirm-wait cannot observe (see F-3).
- **The client currently ignores the ack for ops.** The `--service`/resident receive loops in
  `xgen-client/src/app.rs` *recognise* `EventAccepted`/`Error` but only log them; the carried
  comment is verbatim *"per-verb submit-and-await behaviour is deferred"*. **F1a is exactly that
  deferred work** — foreseen + named, not a surprise.
- **`Connection::goodbye`** (`xgen-core/src/transport/connection.rs`) = send `Goodbye` +
  `ws.close(None)` + drop; it does **not** await the peer draining the stream, so the close
  races as an RST that discards buffered frames. (Confirming "clean close drains" is option (B)
  in the design space and is the wrong direction per Joe — it lowers loss probability without
  giving confirmation.)

---

## 3. Forks (for Joe-lock at design)

- **F-1 — retrofit scope (the crux).** Joe directed *retrofit in-scope*.
  - (a) `create-dm-space` only — minimal; fixes the deterministic break.
  - (b) all multi-event ops — today identical to (a) (`create-dm-space` is the only one).
  - (c) **ALL ops await their event's ack before next-send / before goodbye** — the principled
    client-model fix: every submitted event is confirmed durable before the client proceeds or
    closes; eliminates the latent single-event class too.
  - **Lean: (c)** per Joe ("retrofit in-scope"). The recorded principle becomes
    "a verb does not return / close until the node has confirmed each event it sent." Future
    multi-event verbs inherit it for free.
- **F-2 — confirm semantics.** What counts as "the node took it"? **`EventAccepted` OR
  `Error`(reject) for the matching `event_id`** — both are deterministic node outcomes; the
  client may proceed on either. (The DM auto-invite is a legitimate by-design reject under DM
  constraints — proceeding on its `Error` is correct, not a failure.) **Lean: accept either
  signal as "confirmed", surface a real `Error` to the caller only when it is the verb's own
  primary event.**
- **F-3 — HeldPending (the silent outcome).** Only relevant to scope (c) for ops whose
  `prev_events` can reference an event the node lacks (e.g. a `send` racing sync). The
  `create-dm-space` chain never hits this (§2). Options: (a) timeout-then-proceed with a logged
  warning (honest, no wire change); (b) node emits a new "held" signal so the client knows
  (adds a node wire surface); (c) treat timeout as soft-success. **Lean: (a) for F1a** (keep
  F1a node-wire-neutral); flag (b) as a named follow-on if the general retrofit needs positive
  held-visibility. **This is the only fork that could pull a node-side change into F1a — keep it
  out unless Joe wants (b).**
- **F-4 — where the confirm lives.** A thin `Connection` helper
  (`send_event_confirmed` / `await_event_outcome`) vs an ops-layer helper. The await loop must
  skip unrelated inbound (fan-out events for other Spaces the client is in, pings) and match by
  `event_id`. **Lean: a `Connection`-level confirm helper** (single source of truth, reusable,
  D-067 no-drift) consumed by the ops layer.
- **F-5 — timeout + failure policy.** A per-event confirm needs a deadline (the (A)-prototype
  used 5 s). On timeout: error the op vs proceed. For the multi-event chain a lost event = a
  broken DM, so **timeout-should-error is defensible** (turns `create-dm-space` from
  infallible-optimistic to fallible-honest). **Lean: error on confirm-timeout for the chain;
  the exact value + whether single-event ops also hard-error is a design sizing call.**

---

## 4. D-076 / convergence check

**Discharged.** Send-confirm is a **client-side delivery-pacing** change: the client waits for
the node's already-produced ack before sending the next event / closing. The events sent are
byte-identical; their `prev_events`, content, signatures, and the node's
`dispatch_event`/`derive_resolved`/`state_key_for_event`/ordering/`now` are all untouched. The
ack is admission *output*, read only by the client. There is **no ordering or convergence
surface** in the blast radius (mirrors the MP-F2 / MP-F3 D-076 discharge). The only behavioural
delta is *when* the client sends/closes — which strictly *increases* the chance the node
receives every event, never changes which event wins a conflict.

---

## 5. Honest boundary (D-065)

- F1a fixes **delivery reliability** (the client ensures the node took each event before
  proceeding/closing). It makes alice's DM message land on the **home node** (facet-2's
  home-node half).
- F1a does **NOT** fix **facet-1** (cross-node DM convergence). Confirmed empirically (J-327):
  even with all three `create-dm-space` events delivered, membership still diverges
  (alice-view `{alice}` vs bob-view `{alice,bob}`) because a DM's `dm_constraints_active`
  rejects `apply_federation_add` → `federation_nodes = 0` (all 7 DM push attempts logged
  `federation_nodes=0`) → no DM event federates either direction. That is **MP-F1b**
  (Joe-locked resolution (iii), separate Phase-0).
- F1a reuses the existing `EventAccepted`/`Error` wire (no new protocol surface) **unless** F-3
  picks (b). Scope (c) adds one client→node round-trip per event (latency for bulk sends) —
  named, accepted for correctness.

---

## 6. Proof obligation

- **Integration:** the committed MP-C-07 known-FAIL smoke is the witness for facet-2's home-node
  half — with F1a, alice's a3 lands on node A (proven by the (A)-prototype). The full MP-C-07
  green still requires F1b; F1a's falsifiable claim is *"all create-dm-space events land + the
  DM message is applied on the home node"* (assertable on a single-node DM, or as a node-A
  sub-assertion of MP-C-07).
- **Unit (client):** the `Connection` confirm helper — ack-matches → proceed; reject(`Error`)
  matches → proceed (or surface, per F-2); unrelated inbound skipped; timeout → policy (F-5).

---

## 7. Scope fence

- **F1a = client send-confirm only** (`xgen-client` + a thin `xgen-core::transport::Connection`
  helper). No node-side change required **unless** F-3 = (b).
- **F1b** (cross-node DM federation, resolution (iii)) is OUT — separate Phase-0, gate B first.
- The 4 queued thin verbs (ban / room_update / thread×3 / create-space `--auth-tier`) are each
  single-event; under scope (c) they inherit the confirm discipline but introduce no new
  multi-event surface.

---

## 8. Next

Joe locks **F-1..F-5** (F-1 retrofit scope + F-3 HeldPending node-wire-neutrality are the
load-bearing calls) → design (`tasks/MP_F1A_SEND_CONFIRM_DESIGN.md`) → runbook → implement →
close. Then **MP-F1b** opens its Phase-0 (resolution (iii), A–E, gate B first).
