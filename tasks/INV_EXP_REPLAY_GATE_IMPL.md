# INV-EXP — Invite-Expiry Replay-Gate: Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose, scope, entry point

Implementation runbook for INV-EXP. Executes the J-296 Joe-LOCKED design (`tasks/INV_EXP_REPLAY_GATE_DESIGN.md`) — no new decisions; D-090 already landed. **Entry point for Clair (Rule 0):** CLAUDE PLAY → JOURNAL J-296 → design §3–§7 → this runbook §2–§3 (then audit §3 for the findings). Clair picks up at **checkpoint #1**.

Four commits, two checkpoints. The plumbing (C1) is split from the gate behaviour (C2) for bisectability — same discipline as M8.6's seam-before-tests split.

## 2. Commit plan

### Commit 1 — per-entry origin in `PendingBuffer` (structural origin correctness)
File surface: `xgen-core/src/dag/pending.rs` + the three drain fns + `add()` call sites in `xgen-core/src/node/runtime.rs`.
- `BufferedEntry` gains `origin: EventOrigin` (import `EventOrigin` into `pending.rs` from the node module — same crate).
- `PendingBuffer::add(…)` (pending.rs:151) gains an `origin: EventOrigin` parameter; stores it in `BufferedEntry`. (Param placement Clair's call — surfaces at checkpoint #1.)
- `try_release` (pending.rs:380, the private core all three triggers funnel through) returns the entry's stored origin alongside the event; `resolve_identity` (243) + `resolve_federation_relationship` (286) + the predecessor-release path propagate → return **`Vec<(Event, EventOrigin)>`** instead of `Vec<Event>`.
- The two `add()` call sites in `dispatch_event` pass the function's own `origin`: F-3 deferral (runtime.rs:1004) and missing-identity/predecessor (1078).
- The three `drain_pending_*` loops (`drain_pending_by_identity` runtime.rs:1527, `drain_pending_uniform` 1447, `drain_pending_by_federation_relationship` 1605): iterate `Vec<(Event, EventOrigin)>` and re-dispatch with the **stored per-entry origin** — `self.dispatch_event(ev, stored_origin, None)` (replaces the batch-`origin` call at 1573 and siblings).
- **Remove** the batch `origin` param from the three `drain_pending_*` signatures; update callers (identity-arrival, predecessor-drain, federation-relationship-drain hooks in app.rs/runtime).

**Behaviour note (NOT neutral — D-065).** C1 corrects the origin a *drained* event carries. Today a federation-buffered-then-drained event re-dispatches with the trigger's batch origin, which can wrongly stamp it `LocallySubmitted` — a latent D-089 anti-transitivity / federation-push-eligibility hazard. C1 fixes that to the true stored origin. The 3044/3045 gates still run on **every** origin at C1 (the headline bug is fixed in C2, not here). Suite should stay green; **if any test depends on the batch-origin behaviour, surface it as a finding** (do not paper over).

### Commit 2 — the gate fix (origin-guard + clock migration)
File surface: `xgen-core/src/node/runtime.rs`.
- Guard the **3044** join-expiry block (1190–1227) and the **3045** invite-over-ceiling block (1145–1171) on `origin == EventOrigin::LocallySubmitted` — skip both on `ReceivedViaFederation`.
- Migrate the 3044 expiry comparison from raw `Utc::now()` to `self.clock.now_utc()` (D-090; NodeRuntime-resident clock from M8.6). 3045 has no clock read.
- This is the headline fix: a federation replica neither re-adjudicates expiry nor re-checks the ceiling; it applies the already-admitted membership fact.

### Commit 3 — tests
File surface: gate units in `runtime.rs` (beside the INV-D6 tests); the federation repro in the two-Node harness (sibling to `phase9_*`). Add `xgen-common` `mock-clock` to **xgen-core `[dev-dependencies]`** (not yet present — C6 was time-independent).
- **Headline repro** `inv_exp_federation_replay_preserves_membership`: H admits invite+join in-window; advance the injected `Clock` past `valid_until`; fresh peer B federates + catches up. Pre-fix: B rejects (3044) → member absent on B, present on H. Post-fix: B applies → H/B converge. Deterministic via injected Clock (no real sleep).
- **Per-path units** (mirror design §3 table): local-direct still rejects expired (enforcement intact); local-buffered→drained still enforces (gate at drain); federation-direct skips; federation-buffered→drained skips.
- **Mixed-origin drain test**: one local + one federation join waiting on the same identity → drain re-dispatches each with its **own** stored origin (guards the C1 mechanism; the §4 correctness that a single drain can release mixed origins).

### Commit 4 — close
Doc-only. Flip AUDIT/DESIGN/IMPL → COMPLETED; JOURNAL close entry (record the witness + any C1 findings); PLAY → CLOSED; ROADMAP tree-child → ✅, NEXT-ACTIVE → M8.7. D-074 atomic.

## 3. Checkpoints

- **Checkpoint #1 (after C1, before C2).** Clair surfaces the final signature shape — `BufferedEntry.origin`; the `add()` param placement; the `try_release`/`resolve_*` return type; the three `drain_pending_*` signatures with the batch param removed + the updated caller list — and confirms the suite stayed green. Joe confirms. Light: no value-locks (all decided in design); this is a ripple-confirmation gate.
- **Checkpoint #2 (sensitivity witness, before close).** Demonstrate the headline repro **RED** with the C2 origin-guard reverted (federation replica re-adjudicates → member dropped), **GREEN** restored (file net-unchanged) — proves the test catches the bug, not green-always. Record in the close JOURNAL entry. Mirror of M8.6's C4 witness.

## 4. Named test targets
1. `inv_exp_federation_replay_preserves_membership` (two-Node repro)
2. `inv_exp_local_direct_expired_join_rejected` (enforcement intact)
3. `inv_exp_local_buffered_drain_still_enforces`
4. `inv_exp_federation_direct_skips_expiry`
5. `inv_exp_federation_buffered_drain_skips_expiry`
6. `inv_exp_mixed_origin_drain_preserves_per_entry_origin`
(Names indicative; Clair may refine, surfaces the final list at checkpoint #1.)

## 5. Definition of Done
- `BufferedEntry` carries `origin`; `add()` + `try_release` + `resolve_*` threaded; three `drain_pending_*` re-dispatch the stored origin, batch param removed, all callers updated.
- 3044 + 3045 guarded on `LocallySubmitted`; 3044 clock via `self.clock.now_utc()`.
- Six named tests green; the headline repro proven sensitive (checkpoint #2 witness).
- `ingest_event` (restart) path untouched (IE-A2); `peer_node_id=None`-on-drain F-3 approximation untouched.
- Build all-targets 0; clippy `-D warnings` clean both feature sets; suite green (≥ 1201 + new tests).
- Canonical docs updated atomically (D-074). *(No "commit pushed" line — the COMPLETED header is the shipped signal.)*

## 6. Constraints / do-not-drift
- **Origin keys the gate, not "is-drain"** — the gate is downstream of buffering, so skip-on-drain would skip enforcement entirely. Per-entry origin is the mechanism.
- **Opportunistic clock migration only** — the 3044 gate, nothing else (D-090; no blanket `Utc::now()` sweep).
- **MockClock stays test-only** behind `mock-clock`.
- **Don't restructure** the drain F-3 `peer_node_id=None` approximation or the `ingest_event` restart path.
- **Never push** — Joe pushes.

---

*End of INV-EXP runbook. C1 = per-entry origin in `PendingBuffer` (corrects drained-event origin; not behaviour-neutral — surfaces a latent anti-transitivity hazard); C2 = 3044/3045 origin-guard + 3044 clock→`self.clock` (the headline fix); C3 = repro + per-path + mixed-origin tests; C4 = close. Two checkpoints: ripple-confirm after C1, sensitivity witness before close.*  
