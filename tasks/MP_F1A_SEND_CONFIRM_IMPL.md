# MP-F1a — client send-confirm — IMPLEMENTATION RUNBOOK

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

Executes the Joe-locked design (`tasks/MP_F1A_SEND_CONFIRM_DESIGN.md`, F1A-D1..D6). Production
arc (`xgen-core` transport + `xgen-client`). Three commits: **C1** the helper + units → **C2**
the ops retrofit + facet-2 witness → **C3** doc-only close. D-076 discharged (design §6 — client
send-pacing only). No node-side wire change (F1A-D5). **MP-F1b** (cross-node DM federation) is
OUT. Canonical records (CLAUDE/JOURNAL/ROADMAP/DECISIONS) are the doc-bridge seat; code commits
first, doc-bridge after (standing order).

---

## 1. What ships

The recorded principle (F1A-D1): **a verb does not return / close until each event it sent is
node-confirmed** (or its op-class timeout policy fires). The client finally consumes the D-070
`EventAccepted`/`Error` ack it currently ignores. Fixes the deterministic `create-dm-space`
multi-event loss (facet-2 home-node half) **and** the latent single-event class. Does **not**
fix facet-1 (MP-F1b).

---

## 2. Commit plan

### C1 — `Connection::send_event_confirmed` + unit tests (`xgen-core`)

**File:** `xgen-core/src/transport/connection.rs`.

1. Add the outcome enum (module-public):
   ```text
   pub enum EventConfirm { Accepted, Rejected { code: u32, reason: String }, TimedOut }
   ```
2. Add the helper on `impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S>`:
   ```text
   pub async fn send_event_confirmed(&mut self, event: &Event, timeout: Duration)
       -> Result<EventConfirm, TransportError>
   ```
   - `self.send_event(event).await?`.
   - Resolve the sent `event_id` (`event.event_id`); a `None` event_id is a programming error
     on a signed event — return a `TransportError` (don't silently match nothing).
   - `tokio::time::timeout(timeout, …)` around a `recv()` loop:
     - `Inbound::Transport(EventAccepted { event_id, .. })` where `event_id == sent` → `Accepted`.
     - `Inbound::Transport(Error { event_id: Some(id), error_code, error_string, .. })` where
       `id == sent` → `Rejected { code: error_code, reason: error_string }`.
     - any other `Inbound` (fan-out `Event` for another Space, ping/pong, stray sync frame,
       `EventAccepted`/`Error` for a *different* id) → **skip, keep looping**.
     - `Inbound::Closed` / `recv()` `Err` → `TransportError`.
   - timeout elapses → `Ok(EventConfirm::TimedOut)` (a TimedOut is an outcome, not an error — the
     op layer applies F1A-D4 policy).
   - **Do not touch** `send_event` / `recv` / `goodbye` (F1A-D2 single-source; the existing
     fire-and-forget primitives stay for the handshake/federation paths that legitimately use
     them).
3. Imports: `std::time::Duration`, `tokio::time::timeout`.
4. **Unit tests** (`#[cfg(test)]`, in-process WS pair via `tokio::io::duplex` +
   `tokio_tungstenite::{client_async, accept_async}` → two `Connection<DuplexStream>`; both deps
   already present). Drive the server half to elicit each branch:
   - (a) server replies `EventAccepted{id}` → `Accepted`.
   - (b) server replies `Error{event_id:Some(id), error_code, error_string}` → `Rejected{code,reason}`.
   - (c) server stays silent → `TimedOut` after a short (e.g. 200 ms) timeout.
   - (d) server sends an unrelated `Event` (different id), *then* `EventAccepted{id}` → skips the
     former, returns `Accepted`.
   - (e) server drops the connection mid-await → `TransportError`.

**DoD C1:** `cargo build -p xgen-core` 0; `cargo test -p xgen-core` green (+5 helper units);
clippy clean. No xgen-client/xgen-node change yet (the helper is additive; existing callers
untouched).

### C2 — ops retrofit + facet-2 witness (`xgen-client`)

**File:** `xgen-client/src/ops.rs` (+ `xgen-client/src/tests/`, + one mptest scenario).

Retrofit all **9** sending ops per the §3 table. Each op resolves the timeout via the existing
`sync_completion_timeout(ctx.data_dir)` (`[sync].completion_timeout_seconds`, default 5s — D-067
no-drift) and passes it to `send_event_confirmed`. Policy (F1A-D3/D4):

- **`create_dm_space` (multi-event chain → error-on-timeout, F-5):** confirm each of the 3
  events in order. `Accepted` → next. `TimedOut`/`TransportError` → **abort the remaining sends,
  `goodbye`, and return `Err`** (fallible-honest); the predecessor was acked-present so a
  timeout = genuinely lost (design F1A-D4 rationale). `Rejected` of `dm_space_create`/`room` →
  return that as the verb error; `Rejected` of the auto-invite is by-design-OK (empirically it
  is `Accepted` — DAG-valid, internal DM-constraint state no-op swallowed — but accept-either
  per F1A-D3). **Move the client-state record (`state.spaces.push(...)` + `write_client_state`)
  to AFTER the confirmed send block** so a failed create writes no success record.
- **Single-event ops (proceed-on-timeout, F-3(a)):** confirm the one event. `Accepted` → ok.
  `Rejected`(own event) → return `Err` (honest — the node rejected it; this is the beneficial
  side-effect below). `TimedOut` → **`tracing::warn!` and proceed** (return ok-unconfirmed); the
  ambiguity is irreducible without a held-signal (F1A-D5).

**Beneficial side-effect to verify (D-077 backward-coherence):** surfacing `Rejected` as the
verb's `Err` means single-event ops become *fallible on node-reject* (today they swallow it —
the MP-R1-D9 / J-081 §5 "batch ops are write-only, rejections invisible" gap). This is correct
(rejections finally reach the client) but is a contract change. Run the full `xgen-client` +
`xgen-mptest` suites; any cooperative test that relied on a silently-swallowed reject is a
**finding to surface, not work around**. Adversarial mptest scenarios that previously could not
see a rejection may now observe it — confirm they still assert correctly.

**D-028 lockstep check (CP-3, at C2):** grep the `xgen-client` `main.rs` per-verb doc comments
for any that describe the old exit/contract semantics (e.g. "always succeeds" / optimistic-ack
wording for `create-dm-space` or the single-event verbs). If the retrofit shifts any per-verb
doc comment, D-028 requires Appendix F to move in lockstep — record which comments shifted so
the C3 Appendix F edit covers them. (CLI **syntax** is unchanged; only exit-code/contract
semantics shift.)

**Witnesses:**
1. **Policy (deterministic, stub-WS, `xgen-client/src/tests/send_confirm_integration.rs`):**
   mirror the `reregistration_integration.rs` ephemeral-WS-node pattern. A controllable stub
   that, per test, acks / rejects / stays silent for specific event_ids:
   - `create_dm_space` against a stub that acks event 1 then goes silent → verb returns `Err`
     **and writes no client-state record** (assert the state file unchanged).
   - `create_dm_space` against a stub that acks all 3 → verb `Ok`, record written.
   - a single-event op against a silent stub → `Ok` with a warning (proceed).
   - a single-event op against a stub that `Error`-rejects its event → `Err`.
2. **End-to-end facet-2 (real binaries, `#[ignore]`):** a **single-node DM** mptest scenario
   `docs/tests/multiparty_scenarios/MP-C-07-LOCAL/` (template: the single-node MP-C-01 manifest
   — one node `a`, two actors both `node = "a"`, **no `[[federation]]`**). alice
   `create-dm-space` with bob; bob `join` (space + room); both `send`. Assert: both messages +
   the room land in node `a`'s `.events` and the 2-party membership/content **converge on the
   single node**. This isolates facet-2 (no federation ⇒ no facet-1). RED before C2, GREEN after.
   Add the runner in the appropriate `mp_r1_c*` test file (sibling to the MP-C-01 invocation in
   `mp_r1_c5.rs` / `mp_r1_runner.rs`). The existing federated `MP-C-07` smoke **stays `#[ignore]`
   / known-FAIL** (it needs F1b) — note that in its doc-comment, don't flip it.

**DoD C2:** `cargo build -p xgen-client` 0 (+ `cargo build -p xgen-node --features
harness-control` for the scenario); clippy clean; `cargo test -p xgen-client` green (+ stub
tests); `cargo test -p xgen-mptest` fast suite green; the new single-node-DM `#[ignore]` witness
PASSES out-of-band and the federated MP-C-07 still reproduces its known FAIL. Record actual
numbers.

### C3 — close (doc-only)

Flip `MP_F1A_SEND_CONFIRM_{AUDIT,DESIGN,IMPL}.md` → COMPLETED. `tasks/MP_findings.md` MP-F1:
mark facet-2 RESOLVED (home-node delivery fixed; cross-node convergence remains under MP-F1b);
record the routed "HeldPending positive visibility" candidate (F1A-D5). The CLAUDE/JOURNAL/
ROADMAP doc-bridge is the Chat seat (not this runbook's commits).

**Appendix F / D-028 (CP-3 — HARD C3 deliverable, gates the close; Chat owns the spec-doc
edit).** F1a changes command **exit/contract** semantics, not CLI syntax: `create-dm-space`
becomes **fallible on confirm-timeout** (was always optimistic-ok), and the single-event
`Rejected → Err` (CP-1) now **surfaces node rejections** to the command/batch layer. Per D-028
(spec ↔ CLI in lockstep), at C3 update **`docs/xgen_appendix_f_en.md`**:
- the **exit-code** section (a verb can now exit non-zero on confirm-timeout / surfaced reject);
- the **batch-reply schema** section (a batch reply can now carry an error where it previously
  always reported success);
- any **affected usage examples** (create-dm-space + the single-event verbs);
- plus any per-verb prose flagged shifted by the C2 D-028 lockstep check.
This is not optional — it is a gating C3 deliverable so the spec doesn't silently lag the
contract change.

---

## 3. Per-op retrofit table

| op | events | class | TimedOut policy | Rejected(own) |
|---|---|---|---|---|
| `create_dm_space` | 3 (dm_space_create→room→invite) | **chain** | abort + `Err` (F-5) | `Err` (primary); invite by-design-OK |
| `create_space` | 1 | single | warn + proceed | `Err` |
| `create_room` | 1 | single | warn + proceed | `Err` |
| `send` | 1 | single | warn + proceed | `Err` |
| `join` | 1 | single | warn + proceed | `Err` |
| `leave` | 1 | single | warn + proceed | `Err` |
| `invite` | 1 | single | warn + proceed | `Err` |
| `ai_delegate` | 1 | single | warn + proceed | `Err` |
| `ai_revoke` | 1 | single | warn + proceed | `Err` |
| `register` | 0 (identity msg, not a DAG event) | n/a | unchanged | unchanged |

`register` sends an `IdentityMessage::Register` (not `send_event`) and already awaits its
`RegisterOk` — out of scope, untouched.

---

## 4. Checkpoints (Joe-lock)

None blocking — the design locks F1A-D1..D6 fully. Two confirm-at-pickup items (D-078), surface
if they diverge from the design, don't re-litigate:
- **CP-1:** the single-event `Rejected → Err` contract change blast radius (C2) — if the suite
  surfaces a cooperative test/scenario relying on a swallowed reject, surface it (finding), don't
  paper over.
- **CP-2:** the `send_event_confirmed` WS-pair test scaffolding (C1) — if `tokio::io::duplex` +
  `client_async`/`accept_async` is awkward in `xgen-core`, fall back to proving the helper
  behaviour via the C2 stub-WS tests and ship C1 with the matching/timeout logic only;
  surface the choice.
- **CP-3 (Appendix F / D-028 — HARD C3 deliverable, NOT optional):** F1a shifts command
  exit/contract semantics (create-dm-space fallible on confirm-timeout; single-event
  `Rejected→Err` surfaces rejections), so `docs/xgen_appendix_f_en.md` exit-code +
  batch-reply-schema sections + affected usage examples update at C3 (Chat owns the spec edit).
  At **C2**, run the D-028 lockstep grep of `main.rs` per-verb doc comments and record any that
  shifted so the C3 edit covers them. CLI syntax is unchanged.

---

## 5. Scope fence

- **IN:** `Connection::send_event_confirmed` + the 9-op retrofit + the policy + the facet-2
  single-node witness + stub-WS policy tests.
- **OUT:** any node-side change (F1a is wire-neutral, F1A-D5); HeldPending positive visibility
  (routed, not solved); **MP-F1b** cross-node DM federation (separate Phase-0, resolution (iii),
  gate B first); the federated MP-C-07 smoke (stays known-FAIL until F1b).

---

## 6. Definition of Done

- [ ] C1: `EventConfirm` + `send_event_confirmed` shipped; 5 helper units green; xgen-core build
      0 + clippy clean; existing callers untouched.
- [ ] C2: 9 ops retrofitted per §3; client-state record moved after confirm in `create_dm_space`;
      stub-WS policy tests green; single-node-DM witness PASSES out-of-band (RED→GREEN); federated
      MP-C-07 still known-FAIL; full xgen-client + xgen-mptest fast suites green (actual numbers
      recorded); the `Rejected→Err` backward-coherence sweep done (findings surfaced, not worked
      around).
- [ ] C2: D-028 lockstep grep of `main.rs` per-verb doc comments run; shifted comments recorded
      for the C3 Appendix F edit.
- [ ] C3: AUDIT/DESIGN/IMPL → COMPLETED; `MP_findings.md` MP-F1 facet-2 RESOLVED + HeldPending
      visibility routed.
- [ ] C3 (CP-3, GATING): `docs/xgen_appendix_f_en.md` exit-code + batch-reply-schema sections +
      affected usage examples updated (D-028 lockstep; Chat owns the spec edit). The close does
      not complete until this lands.
- [ ] No node-wire change; D-076 untouched; no DECISIONS change (F1A-D# arc-local, D-069).
