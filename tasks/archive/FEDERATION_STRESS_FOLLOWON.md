# Task — Federation Stress Follow-On Milestone
> **Status**: COMPLETED (M8.6 shipped the four deferred compounds C1/C4/C6/C8 + the clock-injection seam in-milestone; closed at J-294, 2026-06-06)  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-19 (initial stub created at Phase 9 design-lock; scope is the four compounds deferred from Phase 9 per `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` §3.11 + the clock-injection seam they depend on)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Ship the four federation-stress compound scenarios deferred from Phase 9 of the Federation Event Propagation milestone, plus the structural enabler (clock-injection seam) those compounds depend on.

**This is a placeholder task file at PENDING status.** Scope is locked at creation; design phase + implementation work happens when this milestone goes ACTIVE. The file exists to give the deferred work a named home so it does not get lost in the post-milestone roadmap shuffle.

**When does this go ACTIVE?** After:
1. Federation Event Propagation milestone closes (Phase 9 ships).
2. M6 (new) ships (envelope `event_id` on `TransportMessage::Error` plus all admin write-path verbs).
3. M7 ships (`--aicontrol` over both binaries).
4. Client-Side Consequences Audit ships (the J-081-shape canonical doc that runs against the Phase 9 catalogue's "not caught" entries plus the consequences of federation-actually-working that surface to client-side surfaces).

Roughly speaking: after the M-series trunk catches up to where federation propagation is being exercised under production-like operator workflows. The bugs this milestone hunts for are *bugs at scale*; running this milestone before the system reaches that scale would produce green checkmarks against unrepresentative load.

---

## Scope — locked at creation

### Compound C1 — F-10 unknown-signer arriving during F-1b drop

**Deferred from Phase 9 per findings §3.1.** Peer A pushes event with unknown signer Bob's Identity to peer B → B buffers as HeldPending → A's connection to B drops mid-stream → reconnect happens via F-1a → does HeldPending survive? Does it resolve? Does F-10's 30 s timeout fire correctly when wall-clock spans the drop period?

**Catalogue bugs hunted.** M3 (HeldPending survives identity arrival but drain_pending_by_identity doesn't fire) AND M6 (Phase 5 tokio::spawn leak) — but the component bugs are caught in Phase 9 in their primary surfaces. C1 catches the *compound* failure where both surfaces interact: F-1a recovery handshake re-streams Bob's join event but B's HeldPending still holds the prior version → duplicate-ingest hazard.

**Cost.** Medium-Hard. Requires clock injection (the 30 s F-10 timeout otherwise makes parallel test runs impractical).

### Compound C4 — Phase 5 reconnect scheduler under churn

**Deferred from Phase 9 per findings §3.4.** Drop peer, recover, drop, recover, drop, recover — 5 cycles in 10 minutes. Does the backoff ladder reset correctly on each handshake-ACTIVE? Does `peer_records` JSON stay consistent? Does any cycle leak a `tokio::spawn`? Does `peer_records` get out of sync with `relationships`?

**Catalogue bugs hunted.** M6 (Phase 5 `tokio::spawn` per peer per tick leaks). Plus invariant checks on `mark_active` / `mark_lost` API under cycling.

**Cost.** Hard without clock injection. The backoff ladder is 15/30/60/120 minutes wall-clock; five cycles = 5+ hours wall-clock unless we inject a clock.

### Compound C6 — F-10 identity-arrival hook under parallel arrivals

**Deferred from Phase 9 per findings §3.6.** Two federation pushes arrive simultaneously, both with unknown signers; both signers' identity records arrive in close succession. Does `drain_pending_by_identity` handle parallel arrivals correctly?

**Catalogue bugs hunted.** M9 (HeldPending double-drain on parallel identity arrivals).

**Cost.** Medium. Race-window-sensitive. C10 in Phase 9 covers the single-identity-multiple-replicate variant; C6 covers the two-identity-parallel-arrival variant. Different shapes; both worth proving.

### Compound C8 — Bidirectional simultaneous push

**Deferred from Phase 9 per findings §3.8.** A pushes E_A to B simultaneously with B pushing E_B to A. Do both arrive, both ingest, both reach local fan-out? Does F-2 + F-2a handle simultaneous push without deadlock?

**Catalogue bugs hunted.** M8 (bidirectional simultaneous push deadlocks F-2a session).

**Cost.** Medium-Hard. Engineering simultaneous wall-clock push is non-trivial. Bug is improbable (try_send is non-blocking) but not impossible; deserves a real test rather than a confidence assertion.

### Structural enabler — Clock-injection seam

**Required for C1 and C4.** Phase 5 did NOT ship a clock-injection seam: `scheduler_tick` accepts a runtime + senders + paths, not a clock. F-10's 30 s timeout and F-1c's 15/30/60/120 min backoff ladder both use `std::time::SystemTime::now()` (or `chrono::Utc::now()` per actual code) directly.

**Design phase deliverable.** Add a `Clock` trait to `xgen-common` with `now()`-style methods; thread an `Arc<dyn Clock>` (or generic parameter) through `scheduler_tick`, the F-10 timeout sweep, and any other federation surface that depends on wall-clock. Default implementation reads real time; test-only `MockClock` provides controllable advancement. Pair with `tokio::time::pause()` + `tokio::time::advance()` where the tokio runtime is already involved.

**Why this is its own deliverable.** It's not test-only code; it's a structural change to how federation reads time. Threading a Clock affects production code (every wall-clock call site rewires). The design choices (trait vs generic; Arc vs &dyn; sync vs async API; thread-safety guarantees; serde behaviour if Clock is in any persisted struct) need their own Joe-lock pass per D-069 discipline.

**Estimated cost.** 2-3 days for the trait + threading + tests + design doc update.

---

## Pre-milestone Phase 0 (design) work

Before going ACTIVE, this milestone needs a Phase 0 design pass per D-071 ("subsystem audits precede dependent milestones") and the project principle ("every milestone's Phase 0 includes a subsystem audit"):

1. **Audit clock-call-site surface.** Trace every wall-clock read in the federation surface (xgen-core/src/federation, xgen-node/src/reconnect, runtime.rs timeout sweep, registry.rs serialised timestamps, etc.). Findings document shape: J-081 / Phase 9 survey shape — code-grounded, every claim cited file:line.
2. **Joe-lock the Clock trait shape.** Sync vs async API; trait object vs generic; persistence behaviour. Same Joe-lock discipline as F-items.
3. **Design phase Joe-locks for C1, C4, C6, C8 harness shapes.** Each compound's deployment-vs-NodeRuntime decision + observability + honesty assertions per Phase 9 survey precedent.
4. **Survey for any new compounds surfaced during the audit.** Phase 9 survey added C9 and C10 from trace; the clock-injection audit may add C11+.

---

## Coordination with adjacent milestones

- **Federation Event Propagation milestone:** This milestone is downstream. Phase 9 ships first; this milestone exists because Phase 9's locked scope explicitly deferred these compounds.
- **M6 (new):** This milestone is downstream of M6. M6 ships the admin write-path verbs that include drop-peer affordances (Phase 9 survey Gap G5 deferred here). C4's drop-recover cycles can use the M6 drop-peer verb instead of the kill-the-binary approximation Phase 9 used.
- **M7 (`--aicontrol`):** Probably parallel to this milestone, not a hard dependency.
- **Client-Side Consequences Audit:** Should ship before this milestone. CSCA produces the post-Phase-9 understanding of which production scale the system has reached; this milestone tests *at* that scale.

---

## Definition of Done — placeholder

The DoD is filled in during the Phase 0 design pass. At PENDING-status creation, the DoD is the scope above plus generic Phase 9-shape close-out:

- [ ] Clock-injection seam shipped.
- [ ] All four deferred compounds (C1, C4, C6, C8) implemented and passing.
- [ ] Catalogue bugs M6 + M8 + any new bugs surfaced during this milestone's audit caught.
- [ ] CLAUDE.md + ROADMAP.md state-transitions reflected.
- [ ] JOURNAL J-### consolidated entry for milestone close.

---

## Cross-references

- **`tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md`** (COMPLETED v1.1) — the survey that deferred C1, C4, C6, C8 to this milestone. §3.11 compound aggregate locked the deferrals.
- **`tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (ACTIVE / shipping now) — the milestone-closing implementation task that ships the Phase 9 scenarios.
- **`tasks/FEDERATION_PROPAGATION_COMPLETION.md`** — runbook for the parent milestone.
- **`docs/xgen_federation_propagation_design.md`** v1.0 ACTIVE — all ten F-items locked.
- **DECISIONS.md** D-069 (canonical-document rule + design Joe-lock discipline), D-071 (subsystem audits precede dependent milestones — this milestone's Phase 0 follows the same shape).

---

*End of placeholder task file. Status: PENDING. Goes ACTIVE after Federation Event Propagation milestone closes, M6 ships, M7 ships, Client-Side Consequences Audit ships. Phase 0 design pass + Joe-locks happen at activation.*  
