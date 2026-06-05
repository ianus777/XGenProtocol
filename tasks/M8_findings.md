# M8 — Findings (Milestone Diagnostic Output)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

The consolidated output of milestone **M8** (strong multiparty test). M8's charter was
diagnostic: exercise all functional aspects of the binary under multiparty load and surface
what the strong/strategic tests would later hit. This document is that diagnostic — the
**input that scopes M8.5 (finalization) and M9 (test-harness build)**. Authored at the M8
close; M8 is CLOSED.

Per-scenario run results live in `docs/tests/MULTIPARTY_Sn_findings.md`; this is the
milestone-level consolidation. Implementation seat: Clair (Waves 1–4). Spec/diagnostic/
canonical seat: Chat Claude.

---

## 2. Proven-green spine (what M8 verified on B = HEAD `676b9c1` / build `8b14aa8`)

11 workspace tests, suite **1167/0/2**, clippy clean (default + `--all-features`).

- **S2 — convergence-under-conflict (headline M2).** Byte-identical resolved `SpaceState`
  across all Nodes AND every client projection (G-ALIGN), under **every arrival permutation**,
  for the three client-reachable resolution layers: ban-vs-join (Layer 1, 6 perms), owner-vs-
  admin room_update (Layer 4, 720 perms), thread resolved-vs-archived (Layer 5c, 120 perms).
- **S3 — federation/migration.** `state.space_migrate` flips `home_node` A→B and converges
  byte-identical across 3 Nodes; stale source re-migrate rejected on every Node (AF-D2 gate);
  jurisdiction reject + 3-Node anti-transitivity referenced.
- **S4 — durability.** Restart → `replay_spaces_from_dir` → replayed `SpaceState` byte-
  identical, zero orphans. Vanilla store suffices (validates the D-080 EventStore split).
- **S6 — E2E content-blindness.** N-member encrypted Space: zero plaintext in Node-visible
  surfaces; KeyPackage pool consume + replenish; epoch advance on `mls.commit`.
- **S7 — privilege.** Tier-gate join refusal multiparty-visible on every Node; per-Room
  override (`Deny`) enforced + converged. Multiparty behaviour only (M8-A7).
- **S1** referenced (historical A, commit `7e06896`); M8-D3 A/B = historical-A / measured-B.

Metrics: M1 delivery loss-free at resolution (conflict losers retained in DAG); M2 as above;
M3 zero by construction; M4 informational (deterministic proofs, no network latency).
Throughput not measured (M8-D2 non-goal).

---

## 3. Findings — routed

### → M8.5 (finalization box — last big correctness fixes before the strategic test build)
- **INV — invitee membership-bootstrap (headline; general, not AI-specific).** A one-shot
  invitee `join` cannot complete. Root cause (sharpened across Waves 3–4): the invitee cannot
  source the invite tip (`collect_sync_history` serves only full members), so the join falls
  back to referencing the create-root (`[space_id]`) → it is **causally concurrent with the
  invite**, shares state key `membership:space:bob`, and `derive_resolved` Layer 4 elects the
  Owner's invite over the Member's join → join dropped. The contained empty-`prev_events`
  sub-bug (`ops.rs:770` `Ok(empty)` vs the `Err`-only fallback) is real and was fixable, but
  fixing it alone does not make the invitee a member. The correct fix is a membership-
  bootstrap design (see §4). **Wave-4 C8 attempt reverted under the entanglement guard.**
- **F-5 — federation anti-transitivity (propagation-model decision).** A Node never
  re-forwards a federation-received event (`federation_session.rs:266-279` returns early on
  `ReceivedViaFederation`). Chain A-B-C does **not** deliver A→C via B; multi-Node Spaces rely
  on full mesh. This contradicts the spec §3.2 "forward on accept" premise and the S3/S0
  "transitive (non-adjacent)" assumption. **Decision fork for M8.5:** make it transitive
  (re-forward with loop/dedup/propagation policy) **or** formally commit to full-mesh and
  rewrite spec §3.2. Genuine redesign — at home in the finalization box by charter.
- **S5 — identity-rebind (missing wire surface).** Not a bug: `re_registration` is not a CLI
  flag (`RegisterArgs` has only `--name`) and `identity.home_changed` has no EventType (only
  `identity.replicate` is wired — 1 of 3 surfaces). M8.5 builds the 2 missing surfaces. NB:
  this is *build-new-surface* work, heavier in shape than INV/F-5; M8.5's audit scopes it.

### → M9 (build the strategic test harness)
- **AI-resident load-test harness friction list** — captured in `MULTIPARTY_S8_findings.md`.
  Feeds the M9 harness build (the AI-resident-driven multiparty load test). Rationale: M8
  established that **load-bearing multiparty faults (topology/convergence) only surface with
  many genuine concurrent participants**, and the AI resident (`--ai-mode --service`, M4 —
  holds a live membership, A-pure G-ALIGN apply path) is the mechanism that supplies that
  concurrency without mustering a human crowd. The strategic suite should be AI-resident-
  driven **by design**. (First articulation — recorded, not promoted; see §6.)

### Fixed-in-M8
- **C9 — AI resident now honors `--node`.** `main.rs` threads `--node` → `ai_service::run` →
  `resolve_node` (flag > config, D-068); re-run red→green, no config workaround. Pushed
  `a51b556`.

### D3-fenced (not M8.5)
- **PG-05 real-crypto.** Live production MLS encrypt is D3; M8 exercised the envelope/store/
  KeyPackage/epoch interfaces (interface-locked).

### Dormant / conditional
- **`system.key_rotation`** is a forward-ready `EventType` with a `state_key` arm but no
  builder/applier — a concurrent key-rotation conflict is not buildable on B without new wire
  surface (Wave-1 finding; S2 substituted `thread.status`, same Layer 5c). Flag for M8.5's
  audit only if a rotation-conflict needs to be buildable.

---

## 4. INV — leading candidate direction for M8.5 (Joe, design input — NOT locked)

**Inviter-authorized, acceptance-time bootstrap.** The invite is a **time-boxed capability**
(expiration field) addressed to the invitee's public identity. It conveys *authorization*,
not secrets. At **acceptance**, the invitee's join references the invite event (stable,
immutable ID — never stale); the Node, holding the invite and seeing it unexpired, accepts
the join and serves current sync **on the strength of the valid invite, not prior
membership**. The invitee needs only to **name** the invite event, not **read** the Space.

- No secret key travels — identity **public** key + MLS **public** KeyPackage only ("it's
  public, so what").
- Data is fetched **at acceptance** (fresh), not carried as a snapshot at invite-time (stale).
- Real-life parallel: an invitation is "you're on the list until Friday," not the venue's keys
  mailed ahead; entry is checked live against the list.
- **Likely decouples INV from F-5:** INV is tip-visibility across the *membership* boundary;
  F-5 is event-forwarding across the *node* boundary. Settling them together in M8.5 (same
  milestone) avoids fixing one against an assumption the other overturns.

M8.5 owns the lock (its own audit → design → Joe-lock → runbook → implement → close).

---

## 5. Chain change (locked 2026-06-05) + rationale

**Before (J-267):** M8 → M9 (Multiparty Redesign) → Multiparty tests → M10 → UI.
**After:** **M8 (diagnostic, CLOSED) → M8.5 (finalization: INV + F-5 + S5) → M9 (build the
strategic test harness) → Multiparty tests (strategic milestone, no M# designation) → M10 →
UI.**

- **M8.5 = the finalization box** — "the last big correctness fixes before we build the
  strategic test." Holds INV (design-resolved candidate), F-5 (propagation decision), S5
  (missing identity surface). Justified by **volume/nature of work**: too large and too
  redesign-shaped to retrofit into M8's diagnostic charter or into M9's test-build.
- **M9 = the build process** of the strategic multiparty test harness (AI-resident-driven).
- **Multiparty tests = the strategic milestone**, deliberately **unnumbered** (strategic gate,
  not an operative M# step) — runs on a finalized binary.
- This dissolves the old "Multiparty Redesign vs Multiparty tests" label ambiguity: fixes are
  M8.5, the harness build is M9, the strategic run is the unnumbered gate.
- **Clean-table principle** (as applied to UI at J-267): build the strategic test on a
  finalized binary, not alongside the fixes it would otherwise re-surface.

**M8.5 opens in a fresh session** (Rule 0 from a clean entry point), not trailing this one.

---

## 6. M8-D# evaluation

All M8 decisions (M8-D1…D6) are arc-local test-design choices (D-069); none recurred as a
protocol invariant → **no DECISIONS.md promotion** (four-recurrence-durable principle; nothing
at 3+ genuine instances). The "load-bearing multiparty testing needs real concurrent
participants; AI residents are the mechanism" insight is **first articulation** — recorded
here (§3 → M9) and not promoted.

---

## 7. Status & next-active

**M8 CLOSED** (diagnostic complete, Waves 1–4). Suite 1167/0/2. No DECISIONS change.
**Next-active: M8.5 (finalization)** — opens fresh session: Phase-0 audit grounding INV
(against the invite/join/sync code + the §4 candidate), F-5 (the propagation fork), and S5
(the 2 missing identity surfaces), then design → Joe-lock → runbook → Clair.

**Entry point:** CLAUDE.md PLAY → JOURNAL J-269 → this document §3 + §4 + §5 per Rule 0.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-078 + the two-round audit principle.
