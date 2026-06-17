# Phase-0 Audit — Thin-verb Arc 4: `thread`×3 (MP-C-13 / PG-08)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The D-071 Phase-0 audit for the **fourth and last** thin-verb arc (order
Joe-LOCKED J-334: auth-tier → MP-F5 → ban → room_update → **thread×3**). Grounds
the verb surface + the grounding asks, frames forks for Joe-lock. No code,
nothing pre-decided. After thread×3 closes → **R1 rerun** (close criterion:
all-green-except-MP-C-06, MP-C-07 harness-green-with-boundary).

Arc goal: ship the client `thread` verbs (`create` / `resolve` / `archive`) over
the shipped Arc E (PG-08) core — builders, `ThreadState` applier, the
`thread.status` shared state-key (M8 Layer-5c convergence), and the tier +
ChangeInfo gates all exist; **only the client verbs are missing**. Unblocks
**MP-C-13**.

---

## 2. Verb surface (thin — client verbs only; mirror the `ai` subcommand group)

The core (Arc E):
- `build_thread_create_event(key, space, room, prev_events, title: Option<&str>, auth_tier_min: u32)` ([state.rs:1618](../xgen-core/src/space/state.rs#L1618)) → `ThreadCreate`.
- `build_thread_resolved_event(key, space, room, thread_id, prev_events)` + `build_thread_archived_event(…)` ([state.rs:1648/1660](../xgen-core/src/space/state.rs#L1648)) → `ThreadResolved` / `ThreadArchived`, content `{"thread": thread_id}`.
- **thread_id derivation:** the Thread's id = `thread_id_from_event_id(create_event_id)` (pub, [state.rs:1413](../xgen-core/src/space/state.rs#L1413); hash-prefix swap → `xgen://thread/sha256:`). `apply_thread_create` derives the same from its own event_id ([state.rs:905](../xgen-core/src/space/state.rs#L905)) — so the client computes thread_id from the signed create event and resolve/archive reference it.
- appliers: `apply_thread_create` (Open) + `apply_thread_status` (Resolved/Archived); `state_key` for ThreadResolved|ThreadArchived = `("thread.status", thread)` (shared → Layer-5c picks one winner under concurrency; [state_key.rs:108](../xgen-core/src/resolution/state_key.rs#L108)).

No client `thread` verb exists. **Verb shape (fork F-TH-1, lean: subcommand
group):** mirror the existing **`ai`** group (`ClientCommand::Ai(AiCommand)` with
`delegate`/`revoke`/`status`) → `ClientCommand::Thread(ThreadCommand)` with
`Create`/`Resolve`/`Archive`. Each routable sub-action goes through **all four
dispatch arms (D-092)** — one outer `Thread(t)` arm per dispatcher (main CLI ·
run-path · batch · aicontrol) inner-routing the three actions, exactly as the `ai`
arm does (aicontrol.rs:476). Args:
- `thread create --space --room [--title] [--auth-tier-min <u32>]` → `ThreadCreateResult { event_id, thread_id, space_id, room_id }` (must expose `thread_id` for the scenario export).
- `thread resolve --space --room --thread <thread_id>` ; `thread archive --space --room --thread <thread_id>`.

**Wire-neutral** (builders shipped Arc E). **send-confirm** via
`apply_single_event_confirm` (the MP-F5 site) — a refused thread op surfaces
structurally.

---

## 3. Grounding asks

### Ask (a) — 4 arms per verb (D-092) + subcommand-vs-flat → **subcommand group (F-TH-1 lean)**

One `Thread(ThreadCommand)` outer arm in each of the 4 dispatchers, inner-routing
create/resolve/archive (mirror `ai`). D-092 satisfied — every action is routable
via all four paths (incl. the aicontrol `Box::pin` routing, the harness path).

### Ask (b) — gate teeth → **SPLIT: create-tier mostly Tier-1 no-op; resolve/archive ChangeInfo HAS TEETH**

- **ThreadCreate:** dispatch step-4 (runtime.rs) enforces narrow-not-widen
  (`thread_tier < space.auth_tier` → wire 3030) + participation
  (`verify_tier_assertion(creator, thread_tier)`). Narrow-not-widen has teeth at
  Tier-2+ (now creatable via the auth-tier verb; unit-proven
  `pg08_thread_auth_tier_below_room_rejected`); participation is a Tier-1 no-op
  (the PG-13 pattern). Plus Room membership (validate step-11). A Tier-1 member
  can create a thread → **green-eligible**, no route.
- **ThreadResolved / ThreadArchived:** ChangeInfo gate (Admin+,
  [exchange.rs:851](../xgen-core/src/message/exchange.rs#L851) maps these →
  `RoomPermission::ChangeInfo`). **Real teeth** — a non-admin member's
  resolve/archive → `PermissionDenied` → rejected. This gate is **not yet
  witnessed by any MP scenario** (room_update's MP-C-08 exercised the per-Room
  *override* layer, not the ChangeInfo *authority* threshold).

So MP-C-13 is green-eligible (no finding to route); the resolve/archive ChangeInfo
teeth are an available enforcement witness (fork F-TH-2).

### Ask (c) — MP-C-13 oracle → **both-halves (F-TH-2 lean), inherits MP-F5 for the enforcement half**

The row is positive ("thread state transitions converge; rides M8 Layer-5c").
But: (1) the harness has **no `ThreadState` projection** (projections are
membership-only) — so the positive convergence is asserted via the **transcript**
(the create/resolve/archive events all land in every node's cooperative event set;
the Layer-5c winner-selection is unit-proven at state_key.rs:374); (2) the
resolve/archive ChangeInfo teeth (Ask b) are otherwise unwitnessed and cheap to
exercise. **Lean: both-halves —**
- **positive:** alice (owner) `thread create` → `resolve` → `archive`; all three
  events present + converged on the node (final status deterministically Archived);
- **enforcement (assert-the-reject, MP-F5):** a non-admin member (bob) attempts
  `thread resolve` → ChangeInfo `PermissionDenied` → Error with `reject_code`
  (PermissionDenied → **4000**, pin empirically, MP-F2-followon) + `event_id`,
  the op absent everywhere.

Positive-only (drop the enforcement half) is the matrix-minimal fallback if Joe
prefers to keep MP-C-13 strictly to the row's stated convergence claim. Lean
both-halves (loop-consistent with auth-tier/ban/room_update; witnesses the thread
ChangeInfo teeth; inherits MP-F5, the reason this arc follows F5).

### Ask (d) — topology → **single-node (lean)**

Layer-5c resolution is node-local; the convergence + the ChangeInfo reject are
both node-local. Cross-node convergence rides proven MP-C-02 machinery. Lean
single-node (the standing call, as the last three arcs).

---

## 4. Pre-fold gate (the precondition class — ground before folding the design)

MP-C-13's preconditions are all **authorable thinly**:
- parent Room → `create-room` (shipped);
- a Thread → `thread create` (this arc's verb);
- a seated member for the enforcement half → `invite` + `join` (shipped);
- resolve/archive reference the thread_id → exposed by `thread create`'s result
  (`thread_id_from_event_id`, pub) + cross-actor export.

No precondition needs an unbuilt verb → **the arc stays thin.** (The gate that bit
room_update [moderator-seating] and ban [the 4th arm] — checked here: nothing
missing.)

---

## 5. Forks for Joe-lock (recommendations; none pre-decided)

- **F-TH-1 — verb shape:** `thread` **subcommand group** (`create`/`resolve`/`archive`, mirror `ai`); 4 dispatch arms per D-092 (one outer arm per dispatcher). *(lean)* vs three flat verbs.
- **F-TH-2 — oracle:** **both-halves** (positive event-convergence + non-admin-resolve assert-the-reject, ChangeInfo teeth, inherits MP-F5). *(lean)* vs positive-only (matrix-minimal).
- **F-TH-3 — topology:** single-node. *(lean)*
- **F-TH-4 — positive-convergence observation:** assert via **transcript** (the 3 thread events present + converged on every node) since the harness exposes no ThreadState projection. *(lean — the only available rail)* The Layer-5c winner-selection is unit-proven; the integration witness proves the verbs author the events + they converge.

---

## 6. Phase-0 DoD

- [x] Verb surface enumerated: 3 client sub-verbs (subcommand group, mirror `ai`); 4 dispatch arms per D-092; thread_id mechanism grounded; wire-neutral.
- [x] Ask (a) 4-arm/subcommand shape: subcommand group, one outer arm × 4 dispatchers.
- [x] Ask (b) gate teeth: **split** — create-tier mostly Tier-1 no-op (green-eligible); resolve/archive ChangeInfo **has teeth** (enforcement witness available).
- [x] Ask (c) oracle: **both-halves** lean (positive transcript convergence + ChangeInfo assert-the-reject, MP-F5); positive-only fallback noted; decided at Phase-0.
- [x] Ask (d) topology: single-node.
- [x] Pre-fold gate: all MP-C-13 preconditions authorable thinly — arc stays thin.
- [x] Forks framed (F-TH-1 shape · F-TH-2 oracle · F-TH-3 topology · F-TH-4 observation); nothing pre-decided.

**Next:** design phase — lock F-TH-1..4, author the folded runbook, impl (3 verbs
+ MP-C-13 witness, RED-on-revert) → close. Appendix F `thread` entries + the
MP-C-13 matrix flip are required close deliverables (J-323). Then **R1 rerun**.
No DECISIONS change (TH-D# arc-local, D-069; D-092 already promoted).

---

Per D-065 + D-069 + D-071 + D-074 + D-092 (4 dispatch arms). MP-R1-D9
(assert-the-reject, inherited from MP-F5) + MP-R1-D10 (loop-to-green) govern.
MP-F6 (runtime.rs:691 swallowed-apply-error) is already routed — if a thread op
touches that path, note against MP-F6, don't re-route.
