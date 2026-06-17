# INV-EXP — Invite-Expiry Replay-Gate: Phase-0 Audit
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & scope

Phase-0 (D-071) audit opening the **INV-EXP fix-arc** — the invite-expiry gate re-firing on federation replay. Surfaced by M8.6 / C8 (the federation catch-up rejecting invite+join); inserted **before M8.7** at Joe's direction; tree-child of M8.6 (its provenance). Output → design → Joe-lock → runbook → Clair.

**The finding in one line.** The `3044 invite_expired` join-acceptance gate lives in `dispatch_event`, checks `Utc::now() > invite.valid_until`, and runs on **every** origin — so a node receiving a *historical* invited join via federation re-evaluates the (absolute, default-14d) window against its *current* wall-clock and rejects it. A peer catching up a Space older than the invite window drops the historical invited memberships → membership diverges from the home node.

## 2. Method

Grounded against live `main`: the gate site (`runtime.rs:1190-1230`), all `dispatch_event` call sites + their `EventOrigin`, the home-node disk-replay path, `EventOrigin` definition, and the `drain_pending_*` re-dispatch. No code in this phase.

## 3. Findings

### IE-A1 — THE BUG: 3044 re-fires on federation replay with raw wall-clock
The gate (`runtime.rs:1208-1230`, inside `dispatch_event`, `MembershipJoin` arm) fires when the joiner has a pending invite present and checks `chrono::DateTime::parse(vu) → Utc::now() > valid_until → Reject 3044`. It runs **regardless of `origin`**. On federation catch-up, the invite event replays first (`apply_invite` repopulates `pending_invites`), then the join re-dispatches → gate sees the pending invite, compares the home-issued absolute `valid_until` (e.g. +14d from issuance) against the receiver's *current* `Utc::now()`. For any Space older than the window → reject → the join never applies on the peer → that member is missing. Membership integrity diverges across federation by elapsed wall-clock — the protocol's core property.

### IE-A2 — restart is SAFE (ingest_event bypasses the gate)
Home-node disk replay (`replay_spaces_from_dir`, app.rs:4089) routes through **`ingest_event`**, not `dispatch_event` — confirmed by the contracts at runtime.rs:1714 / app.rs:4022 / protocol_audit.rs:29 ("replay uses `ingest_event`, never `dispatch_event`"). The 3044 gate is in `dispatch_event` only, so node restart never re-gates its own historical members. The affected surface is **federation receive only**.

### IE-A3 — 3045 (over-ceiling invite) is replay-stable, not buggy
The sibling `3045 invite_validity_exceeds_max` check (`runtime.rs:1148-1163`, `MembershipInvite` arm) compares `valid_until > invite_timestamp + ceiling(tier)` — two fixed values, **no `Utc::now()`**. It yields the same verdict on every replay, so it is not a wall-clock-replay bug. The admission-only logic (IE-A5) *could* still apply to it for consistency — a design sub-decision, not a defect.

### IE-A4 — drain re-dispatch threads origin, but the buffer stores no per-entry origin
`drain_pending_by_identity(identity_id, origin)` (and the uniform / federation-relationship siblings) take an `origin` and re-dispatch released events with it. But the `PendingBuffer` stores **no per-entry origin** (it already drops per-entry `peer_node_id` — "F-3 peer_node_id not stored per buffered entry; passing None skips the F-3 re-check on drain"). So a buffered event is re-dispatched with the **draining trigger's** origin, not its own. Consequence for the fix: a federation join buffered on a missing identity, then drained when the identity arrives **locally**, would be re-dispatched as `LocallySubmitted` → the origin-gate fix (IE-A5) would mis-fire the expiry check on it. The fix must handle the drain path, not only direct dispatch.

### IE-A5 — the fix signal already exists (EventOrigin)
`dispatch_event(event, origin: EventOrigin, peer_node_id)` already carries `EventOrigin { LocallySubmitted, ReceivedViaFederation }` (runtime.rs:132-140), threaded since Phase 4 for the F-3 check. No new plumbing needed: the expiry gate can be conditioned on origin. Fix surface is small — a guard around the 3044 block.

### IE-A6 — the gate reads raw `Utc::now()`, not the injected Clock
The 3044 comparison uses `Utc::now()` directly, **not** the M8.6 `Arc<dyn Clock>`. This site was outside M8.6's six-site federation-stress fence (it is an identity/membership gate, a different subsystem). Two consequences: (1) a deterministic repro/regression test needs `now` controllable here, so the fix should **extend the M8.6 Clock seam to this gate** (route the read through `clock.now_utc()`); (2) this is the **first reuse of the `Clock` trait outside M8.6** → it crosses the promotion threshold flagged at M8.6 close — promote `Clock` to a D-NNN in this arc.

## 4. Design fork (framed, NOT locked)

**Primary — Option A (origin-gate, admission-only):** run the 3044 expiry check **only at live admission** (`origin == LocallySubmitted`); skip it on `ReceivedViaFederation`. Rationale: the home node adjudicates the invite window in real time at admission (correct, preserved); a federation peer **replicates an already-admitted DAG fact** and must not re-adjudicate it against its own clock. Aligned with the F-5 Option-1 pairwise trust model (peers replicate admitted state, they do not re-admit).

**Sub-decisions for design:**
- **D-1 (drain, from IE-A4):** how to stop the drain path mis-firing the gate. Options: (a) store per-entry origin in the buffer so drain re-dispatches with the true origin; (b) a "skip-expiry-on-drain" rule (a drained event is a re-dispatch of an already-buffered event, not a fresh admission). **Open question to resolve first:** is the 3044 gate **upstream or downstream** of the missing-identity HeldPending buffering? If downstream (gate runs at first receipt, passes within-window, then buffers), skip-on-drain is safe. If upstream (buffered before the gate ever runs), skip-on-drain would skip enforcement entirely for missing-identity joins. **Must be grounded in design before locking D-1.**
- **D-2 (3045):** include the over-ceiling check in admission-only too (consistency), or leave it (it's replay-stable, IE-A3)? Lean: include, for a uniform "membership gates are admission-only" rule.
- **D-3 (clock, from IE-A6):** route the gate's `now` through the injected `Clock` (testability + consistency) and **promote `Clock` to a D-NNN** (first cross-arc reuse). Lean: yes.

**The protocol-semantics Joe-lock (the principle the fix encodes):** *a peer trusts the home node's admission decision on a join and does not re-adjudicate invite-expiry on replication.* Reads as clearly right, but it is a real commitment about where invite-window enforcement lives (admission, once) vs. replication (never) — lock it explicitly.

## 5. Repro / regression test design

The isolated repro the C8 trace only suggested (C8 was tangled with the empty-delta `federation_add` issue): home node H admits invite+join **within** the window; advance `now` past `valid_until`; fresh node B federates with H and catches up → assert **B drops the join (3044) and Bob is absent on B while present on H** (the bug), and after the fix **B reconstructs Bob's membership** (H and B converge). Deterministic advancement requires the IE-A6 clock injection at the gate (else a real-time `sleep`, flaky). This is the fix-arc's first test target.

## 6. Open questions / Joe-locks for design

1. **Semantics lock** — peer-trusts-home-admission (§4). 
2. **D-1** — drain handling; resolve the upstream/downstream gate-vs-buffering ordering first.
3. **D-2** — 3045 admission-only (lean: include).
4. **D-3** — extend Clock to the gate + promote `Clock` to D-NNN (lean: yes).

## 7. State & next-active

- Suite **1201/0/2** (audit-only, no code).
- Provenance: surfaced by M8.6 / C8; tree-child of M8.6; sequenced **before M8.7**.
- No DECISIONS change yet (the `Clock` promotion lands in design per D-3).
- **Next-active:** design phase — ground the D-1 ordering question, lock §6, author the design (origin-gate + drain handling + clock extension), then runbook → Clair. Clair stands down until the runbook exists.

---

*End of INV-EXP Phase-0 audit. Status: ACTIVE. The bug is the 3044 join-expiry gate re-firing on federation replay against wall-clock; the fix signal (EventOrigin) already exists; the real design work is the drain-path handling (IE-A4/D-1) and extending the M8.6 Clock seam here (IE-A6/D-3, which promotes the Clock trait).*  
