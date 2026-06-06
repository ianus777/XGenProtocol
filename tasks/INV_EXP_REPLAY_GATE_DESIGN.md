# INV-EXP — Invite-Expiry Replay-Gate: Design
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & scope

Design phase for the INV-EXP fix-arc (Phase-0 audit: `tasks/INV_EXP_REPLAY_GATE_AUDIT.md`). All four §6 locks **confirmed by Joe (2026-06-06)**. This doc is the design of record; the runbook follows. No code in this phase.

## 2. Locked semantics

**A peer trusts the home node's admission decision on a join and does not re-adjudicate invite-expiry on replication.** Invite-window enforcement lives at **live local admission** (once), never at replication. Aligned with the F-5 / D-089 pairwise-trust model (a received-via-federation event is a terminal replicated fact, not a re-admission).

## 3. The fix — origin-gate, forced by the control-flow ordering

**Grounded ordering (the decisive finding).** In `dispatch_event` (`xgen-core/src/node/runtime.rs`): the missing-identity HeldPending buffering is at **1068-1085** (`ValidationOutcome::HeldPending { missing_identity } → .add(…) → return`); the 3044 expiry gate is at **1190-1227**. The gate is **downstream** of the buffering. Consequence: for a join whose identity is missing at receipt, the function returns *before* the gate — so the gate **never runs at first receipt**; it runs only when the join is **drained** (re-dispatched after the identity arrives). The drain *is* that join's first and only admission.

**This rules out "skip-on-drain":** skipping the gate on drain would skip enforcement *entirely* for any missing-identity-buffered join, reopening the unbounded-window hole INV-D6 closed. So the fix keys on **origin**, not on "is this a drain."

**The rule.** Run the 3044 expiry gate (and 3045, per §5) **iff `event` origin is `LocallySubmitted`**; skip it for `ReceivedViaFederation`. Because the gate runs inside `dispatch_event(event, origin, …)`, this is a guard around the existing block. Direct dispatch is already correct (callers pass the right origin — federation sites at runtime.rs:1951/2238/2300, local at 2060/2162). Only the **drain path** carries the wrong origin today (§4).

**Four-path correctness:**

| Path | Origin at gate | Behaviour |
|------|----------------|-----------|
| Local join, identity present (direct) | `LocallySubmitted` | gate runs — live admission, real-time window ✅ |
| Local join, buffered on missing identity → drained | `LocallySubmitted` (stored) | gate runs at drain = its first admission ✅ enforcement preserved |
| Federation join, direct | `ReceivedViaFederation` | gate skipped — replicates home's decision ✅ |
| Federation join, buffered → drained | `ReceivedViaFederation` (stored) | gate skipped — **the bug, fixed** ✅ |

Net: aged-Space federation catch-up reconstructs membership (gate skipped on the replica); the home node still enforces the window in real time.

## 4. Per-entry origin in PendingBuffer (D-1, forced)

**Why per-entry, not the batch param.** `drain_pending_by_identity(id, origin)` currently re-dispatches *every* released event with one batch `origin`: `self.dispatch_event(ev, origin, None)` (runtime.rs:1573). But a single identity-arrival drain can release a **mix** of origins (local + federation joins waiting on the same identity). A per-batch origin cannot distinguish them. Only per-entry origin is correct.

**Mechanism (runbook will wire exact signatures):**
1. `PendingBuffer` entry stores `origin: EventOrigin`. `add()` gains an `origin` parameter (both call sites: F-3 deferral runtime.rs:1004 → `ReceivedViaFederation`; missing-identity/predecessor 1078 → the dispatch's own `origin`).
2. The release/resolve functions (`resolve_identity` and the predecessor / federation-relationship release paths) return the origin alongside each event — `Vec<Event>` → `Vec<(Event, EventOrigin)>`.
3. The drain loops re-dispatch with the **stored per-entry origin**: `self.dispatch_event(ev, stored_origin, None)`.
4. The vestigial batch `origin` param on `drain_pending_by_identity` / `drain_pending_uniform` / `drain_pending_by_federation_relationship` is **removed**; callers stop passing it.

**Orthogonal, unchanged:** the `peer_node_id = None` on drain (skips the F-3 re-check on drain) is a separate pre-existing approximation, untouched here — this arc fixes the *origin* threading for the 3044/3045 gates, not F-3.

## 5. 3045 (D-2) + Clock extension (D-3 / D-090)

**3045 (D-2 — include).** The over-ceiling invite check (runtime.rs:1145-1171) is replay-stable (no `Utc::now()`), so it is not the bug — but it is gated on `LocallySubmitted` too, for the uniform rule "membership admission gates run only at live local admission." A federated peer applies the invite without re-checking the ceiling.

**Clock (D-3 → D-090).** The 3044 comparison migrates from raw `Utc::now()` to the injected `self.clock.now_utc()` (the M8.6 seam, NodeRuntime-resident). This makes the aged-Space repro deterministic and is the **first cross-arc reuse** of the `Clock` trait → promoted to **D-090** (canonical injectable time source). Opportunistic migration of this one gate only; no blanket sweep.

## 6. Test design

**Repro / regression (the headline).** `inv_exp_federation_replay_preserves_membership`: home node H admits invite+join within the window; advance the injected `Clock` past `valid_until`; a fresh peer B federates with H and catches up. **Before fix:** B rejects the join (3044) → Bob absent on B, present on H (divergence). **After fix:** B applies the join → H and B converge on membership. Determinism via the D-3 clock injection (no real sleep).

**Per-path units** (mirror the §3 table): local-direct still rejects an expired join (admission enforcement intact); local-buffered-then-drained still enforces (gate runs at drain); federation-direct skips; federation-buffered-then-drained skips (the per-entry-origin path). Plus a mixed-origin drain test: one local + one federation join waiting on the same identity drain with their *own* origins (guards the §4 per-entry mechanism).

**Placement.** Gate-level units beside the existing INV-D6 tests in `runtime.rs`; the federation repro in the two-Node harness (sibling to `phase9_*`). xgen-core gains the `mock-clock` dev-dep where the clock-driven units need it.

## 7. Locks (confirmed 2026-06-06)

1. **Semantics** — peer-trusts-home-admission (§2). ✅
2. **D-1** — per-entry origin in `PendingBuffer`; 3044 runs iff `LocallySubmitted`; skip-on-drain rejected (gate downstream of buffering). ✅
3. **D-2** — 3045 gated on `LocallySubmitted` too (uniform admission-only rule). ✅
4. **D-3** — 3044 clock via injected `Clock`; **promote `Clock` to D-090**. ✅

## 8. State & next-active

- Suite **1201/0/2** (design-only, no code).
- **DECISIONS change:** D-090 added (Clock promotion) — the first DECISIONS change in this arc.
- **Next-active:** runbook (`tasks/INV_EXP_REPLAY_GATE_IMPL.md`) — sequence the buffer/origin change, the gate guards, the clock migration, and the tests; Joe-lock checkpoints as needed → Clair. Clair stands down until the runbook exists.

---

*End of INV-EXP design. The fix is an origin-gate (3044/3045 run only at `LocallySubmitted`) made correct on the drain path by per-entry origin in `PendingBuffer`, with the 3044 clock migrated to the injected `Clock` (D-090). The control-flow ordering — gate downstream of missing-identity buffering — is what forces per-entry origin over skip-on-drain.*  
