# MP-F1a — client send-confirm (multi-event delivery) — DESIGN

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

Design phase — locks F1A-D1..D6 (arc-local, D-069) on the five forks Joe locked at Phase-0
(`tasks/MP_F1A_SEND_CONFIRM_AUDIT.md` §8). No code; the runbook is authored after this design
is Joe-locked. The convergence (D-076) check is §6. F1a is **client send-confirm only**;
**MP-F1b** (cross-node DM federation, Joe-locked resolution (iii)) is OUT — separate Phase-0.

**The principle this arc records:** *a verb does not return / close until each event it sent has
been node-confirmed* (or the per-op timeout policy fires). Fire-and-forget borrowed against the
D-070 `EventAccepted` it ignored; F1a makes the client consume it. This is the
"per-verb submit-and-await behaviour is deferred" work the client receive loops already named.

---

## 1. The defect (recap, MP-F1 facet-2 / J-327)

`ops::create_dm_space` sends 3 DAG events (`dm_space_create` → auto-`room_create` →
auto-`invite`) fire-and-forget on one connection, then `goodbye` + drops. The abrupt teardown
(TCP RST → `WSAECONNABORTED`) discards the node's buffered room + invite frames; only event 1
is read. The room never exists → the DM `message.text` is rejected at step 11
(not-a-room-member). Empirically (J-327): forcing all 3 events to land (await per-event signal)
makes the room/invite/message land. `create-dm-space` is the only current multi-event op; every
single-event op is latently exposed (its one event survives only by read-before-abort timing).

---

## 2. Locks

### F1A-D1 — scope (c): all sending ops await their ack (F-1)

Every `ops::*` verb that emits DAG events confirms each event before sending the next and before
`goodbye`. The 9 sending ops (`create_space`, `create_room`, `create_dm_space`, `send`, `join`,
`leave`, `invite`, `ai_delegate`, `ai_revoke`; `register` sends none) all route through the
confirm helper (F1A-D2). The principle is a **durable client-send discipline**, not a DM patch;
the 4 queued thin verbs (ban / room_update / thread×3 / auth-tier-param) inherit it.

### F1A-D2 — `Connection::send_event_confirmed` (F-4)

A single helper on `xgen-core::transport::Connection<S>` is the one source of truth (D-067):

```text
pub async fn send_event_confirmed(
    &mut self,
    event: &Event,
    timeout: Duration,
) -> Result<EventConfirm, TransportError>

pub enum EventConfirm {
    Accepted,                              // EventAccepted for this event_id
    Rejected { code: u32, reason: String },// Error for this event_id (deterministic node outcome)
    TimedOut,                              // no signal within `timeout`
}
```

It sends the event, then drains `recv()` until a `TransportMessage::EventAccepted` **or**
`TransportMessage::Error` whose `event_id` matches the sent event, with a `tokio::time::timeout`
deadline. **Unrelated inbound is skipped** (fan-out `Event`s for other Spaces, pings, stray
sync frames) — matched strictly by `event_id`. A dead connection → `TransportError`.

**Separation of concern (load-bearing):** the helper *observes* the node's outcome; the
**op layer applies policy** (F1A-D3/D4). The helper never decides success/failure — it reports
`Accepted` / `Rejected` / `TimedOut` and lets the caller map that to the verb's contract.

### F1A-D3 — confirm semantics (F-2)

`Accepted` **and** `Rejected` are both "the node took the event deterministically" → the client
may proceed. Mapping at the op:
- `Rejected` of the verb's **own primary event** → surface as the verb's error (honest).
- `Rejected` of an **expected-by-design** event → "confirmed, proceed." (Robustness clause:
  empirically the DM auto-invite is **`Accepted`** — DAG-valid, with an internal DM-constraint
  state-apply no-op that is swallowed during ingest, so it yields `EventAccepted`, not `Error`.
  F-2 still covers any future by-design-reject case so a single mis-frame can't wedge a chain.)

### F1A-D4 — timeout + failure policy (F-5, with the single-event sizing)

The policy **splits by op class**, and the split is **principled, not fork-averaging**: the
discriminator is **whether await-ack has already disambiguated the timeout**. In a multi-event
chain every predecessor is await-ack-confirmed *before* its child is sent, so a mid-chain
timeout cannot be a buffered HeldPending (the predecessor is provably present) — it is
genuinely *lost* → `Err`, abort. A single-event op has *no prior ack* to disambiguate, so its
timeout is irreducibly lost-vs-HeldPending → warn + proceed. F-5 governs the first case, F-3(a)
the second; each follows from the same rule applied to different evidence.

- **Multi-event chain (`create_dm_space`):** await each event's confirm before sending the next.
  On `TimedOut` or `TransportError` → **abort the remaining sends and return `Err` from the
  verb** (fallible-honest). **No client-state record is written on failure** (today's success
  record moves *after* the confirmed send block). A timeout here means the root/chain was lost
  (the chain never goes HeldPending — each predecessor is acked-present before its child is
  sent), so erroring is correct, not conservative.
- **Single-event ops:** await the event's confirm. `Accepted` → ok; `Rejected`(own event) →
  `Err`; **`TimedOut` → log a warning and proceed (return ok-unconfirmed)** — F-3(a). Rationale:
  a single-event timeout is ambiguous between *lost* and *HeldPending* (node has it, buffered,
  will apply); the client cannot distinguish without a held-signal (F1A-D5, routed out), and
  hard-failing every timeout would regress the common HeldPending-eventually-applies case. The
  honest residue is named in §5.
- **Timeout value:** reuse `sync_completion_timeout(data_dir)` =
  `[sync].completion_timeout_seconds` (default 5s) — the existing client-waits-for-node bound,
  configurable, D-067 no-drift (the same timeout `send`/`get_dag_tips` already use). Each op
  resolves it and passes it to the helper; the helper stays config-agnostic.

### F1A-D5 — node-wire-neutral + routed finding (F-3)

F1a adds **no node-side wire surface** — it consumes the existing D-070 `EventAccepted`/`Error`.
`HeldPending` is the one `DispatchOutcome` that emits **no** client signal (Accepted/Duplicate
send `accept_signal`; Rejected sends `reject_signal`; HeldPending is silent). Under F1A-D4's
single-event timeout-proceed, a verb can therefore return ok on a silently-buffered HeldPending
without true confirmation — a **silent-discard shape (D-065 / B3)**, named in §5, **not solved
in F1a**. Routed candidate (not built, not folded in): **"HeldPending positive visibility"** — a
node-emitted held-signal so the client can wait-for-apply instead of proceed-blind. Out of F1a
scope by Joe-lock.

### F1A-D6 — convergence safety + proof (see §6)

Discharged: client send-pacing only; no node ordering/resolution surface touched.

---

## 3. Change surface

- **`xgen-core/src/transport/connection.rs`** — add `EventConfirm` + `send_event_confirmed`
  (sends, then `event_id`-matched drain with timeout; skips unrelated inbound). No change to
  `send_event`/`recv`/`goodbye`.
- **`xgen-client/src/ops.rs`** — the 11 `send_event` sites across 9 ops become
  `send_event_confirmed` + the F1A-D3/D4 policy. `create_dm_space`: confirm each of the 3 in
  order, abort+`Err` on TimedOut/Transport, move the client-state record after the confirmed
  block. Single-event ops: confirm + warn-and-proceed on TimedOut. Each op resolves the timeout
  via `sync_completion_timeout`.
- **Dispatchers** (`xgen-client` CLI + aicontrol/batch) — already propagate `ops::*` `Result`;
  no structural change. `create-dm-space` simply becomes able to return an error
  (fallible-honest) instead of always optimistic-ok.
- **Untouched:** every node crate; the wire format; `dispatch_event`/resolver/ordering;
  `goodbye` semantics; single-event happy paths (byte-identical events, only the client's
  send/close pacing changes).

## 4. Proof plan

- **Unit (`xgen-core`, duplex/stub):** `send_event_confirmed` — (a) `EventAccepted` for the
  id → `Accepted`; (b) `Error` for the id → `Rejected{code,reason}`; (c) no signal → `TimedOut`
  after the deadline; (d) an unrelated `Event` then the matching `EventAccepted` → skips the
  former, returns `Accepted`; (e) connection drop mid-await → `TransportError`.
- **Unit (`xgen-client`):** `create_dm_space` aborts + `Err` + writes no client-state record
  when a confirm times out (stub node that acks event 1, drops 2); single-event op returns
  ok-with-warning on TimedOut.
- **Integration (the facet-2 witness):** a single-node DM flow (or the MP-C-07 node-A
  sub-assertion) — all three `create-dm-space` events land + the DM `message.text` is **applied
  on the home node** (was REJECTED step 11). This is F1a's falsifiable claim. Full MP-C-07 green
  still requires **MP-F1b** (cross-node DM federation) — F1a alone does not flip the whole
  scenario.

---

## 5. Honest boundary (D-065)

- F1a fixes **delivery reliability** + facet-2's **home-node half** (the DM message lands where
  it's sent). It does **NOT** fix **facet-1** (cross-node DM convergence) — confirmed
  empirically (J-327): with all events delivered, membership still diverges because a DM's
  `dm_constraints_active` zeroes `federation_nodes` → no DM event federates. That is **MP-F1b**
  (resolution (iii), gate B first).
- **Named silent-discard residue (F1A-D5):** under the single-event timeout-proceed policy, a
  verb can return ok on a silently-buffered HeldPending without true confirmation. This is a
  D-065/B3 shape, accepted for F1a (node-wire-neutral) and **routed** as the candidate
  "HeldPending positive visibility" node-wire change — *not* solved here, *not* folded in.
- Scope (c) adds one client→node round-trip per event (latency for bulk/multi-event sends) —
  named, accepted for correctness. Single-event happy paths are unchanged in wire bytes.

---

## 6. Convergence (D-076) check

**Discharged.** `send_event_confirmed` changes only *when* the client sends the next event /
closes — it waits for the node's already-produced admission ack. The events are byte-identical
(same `prev_events`, content, signatures); `dispatch_event`, `derive_resolved`,
`state_key_for_event`, ordering, and `now` are untouched; the ack is admission *output* read
only by the client. The change strictly *raises* the probability the node receives every event
and never alters which event wins a conflict. No ordering or convergence surface in the blast
radius (mirrors the MP-F2 / MP-F3 D-076 discharge).

---

## 7. Next

Joe-lock F1A-D1..D6 → author `tasks/MP_F1A_SEND_CONFIRM_IMPL.md` (runbook): C1 the
`Connection::send_event_confirmed` helper + unit tests; C2 the ops.rs retrofit (chain policy +
single-event policy + client-state-record-after-confirm) + the facet-2 integration witness; C3
close. Then **MP-F1b** opens its Phase-0 (resolution (iii), A–E, gate B first).
