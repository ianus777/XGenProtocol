# Multiparty Test S2 — Findings (M8 / Wave 1 / C2)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## What this records

The **C2** result of M8 Wave 1 (runbook `tasks/M8_MULTIPARTY_IMPL.md` §3 C2; design
`tasks/M8_MULTIPARTY_DESIGN.md` §3 S2 row + §4). Per **M8-D1**, S2 is *extended* from
concurrent **message** sends to concurrent **state** events that genuinely conflict, and the
headline metric **M2 (convergence correctness)** is proven: byte-identical resolved
`SpaceState` across all Nodes AND every client projection (**G-ALIGN**), under **every
arrival permutation**.

This is the **first** baseline for S2 — there is no historical "A" (S2 never ran before;
M8-D3). M8 *establishes* the S2 baseline.

---

## Run history

| Run | Date | Build / commit | M2 (convergence) | G-ALIGN | Notes |
|---|---|---|---|---|---|
| 1 | 2026-06-05 | `0.10.3.260605-1454` / `8b14aa8` (≡ code of B target `676b9c1`) | **CONVERGED** | **HOLDS** | Workspace exhaustive proof, 3 conflict layers (1/4/5c), every arrival permutation, cross-node live seam. key-rotation substituted by thread.status (M8-D4 finding). |

B stamp per CP-3 (`MULTIPARTY_M8_readiness.md`): `xgen-node 0.10.3.260605-1454` /
`xgen-client 0.10.3.260605-1454`, commit `8b14aa8` — code-identical to the M8-D3 B target
`676b9c1` (intervening commits doc-only).

---

## Placement decision (CP-4 / M8-D6) — why M2/G-ALIGN is workspace-homed

M2 (byte-identical convergence) and G-ALIGN are **deterministic pure properties** of the
state-resolution algorithm. The strongest possible statement is "**every** arrival permutation
yields one identical resolved state," and `derive_resolved` topo-sorts internally, so a
workspace integration test can enumerate **all** permutations of the conflict log
(`permutations()` / Heap's algorithm) and assert byte-identical output via `SpaceState:
PartialEq`. A binary-level run cannot do this: the CLI exposes no `prev_events` control, so a
controlled conflict requires genuine concurrent dispatch (timing-dependent), and real WS
delivery can realise only a couple of arrival orders — strictly weaker than the exhaustive
workspace proof, and (per M8-D6) **real processes add no signal** to a deterministic
convergence proof. The M2/G-ALIGN headline therefore lives at the workspace level
(`xgen-node/src/tests/m8_s2_convergence.rs`, 3 tests). The operator-realistic concurrent
*federation* send (delivery realism / latency under real 2-Node concurrency) is the S2
binary-half — scoped below.

---

## Conflict matrix (M2 — the headline)

Three conflict cases, one per client-reachable resolution layer (Layers 1/4/5c — the
G-ALIGN-safe layers; Layers 3/5a/5b consult the home-node map and are deliberately avoided
per the R2-F01 A-pure finding). All **live + buildable on B** (builder + applier present).
Each case asserts: (1) every permutation → one identical `SpaceState`; (2) two real Nodes
ingesting the concurrent pair in opposite order converge to that same state at the live
SR-D1 seam; (3) the client A-pure projection (`derive_resolved(log, "", &empty)`, exactly
what `xgen-client::ops::members_projection` calls) equals the Node's resolved view.

| Case | Conflict | Layer | Winner (semantic) | Permutations | Cross-node | G-ALIGN | Verdict |
|---|---|---|---|---|---|---|---|
| ban-vs-join | `MembershipBan(bob)` ∥ `MembershipJoin(bob)`, both ref create root | **1** (removal precedence) | ban wins → bob banned, not a member | 3! = 6, all identical | A==B==canonical | holds | **CONVERGED** |
| role precedence | owner `state.room_update` ∥ admin `state.room_update`, same Room, different overrides | **4** (role) | owner's override (Member·SendMessages·Deny) wins | 6! = 720, all identical | A==B==canonical | holds | **CONVERGED** |
| thread status | `thread.resolved` ∥ `thread.archived`, same thread | **5c** (lexicographic) | lower event_id's terminal status wins (deterministic) | 5! = 120, all identical | A==B==canonical | holds | **CONVERGED** |

Test: `xgen-node/src/tests/m8_s2_convergence.rs` —
`s2_layer1_ban_vs_join_converges`, `s2_layer4_role_precedence_converges`,
`s2_layer5c_thread_status_converges`. All 3 pass.

```
$ cargo test -p xgen-node --lib m8_s2
running 3 tests
test tests::m8_s2_convergence::tests::s2_layer1_ban_vs_join_converges ... ok
test tests::m8_s2_convergence::tests::s2_layer5c_thread_status_converges ... ok
test tests::m8_s2_convergence::tests::s2_layer4_role_precedence_converges ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 265 filtered out; finished in 0.10s
```

Full workspace after C2: `cargo test --workspace` → **1159 passed / 0 failed / 2 ignored**
(1156 baseline + 3 C2 tests); 0 build errors; clippy clean on default **and** `--all-features`.

---

## The four metrics (M8-D2)

- **M1 — Delivery completeness.** *Characterized at the convergence layer (M8-D2 allows
  zero-or-characterized).* The resolution is loss-free: every accepted event remains in the
  DAG; conflict **losers stay in the DAG** (only the resolved snapshot reflects the winner —
  `algorithm.rs` "loser stays in DAG" contract), so no event is dropped by resolution and no
  orphan is produced. End-to-end fan-out **delivery** under real concurrency (the S1-A
  294/300 surface) is a binary-level property — see the binary-half scope below; the S1-A
  baseline (98%, 6/300 WS write-close race) stands as the delivery reference (M8-D3).
- **M2 — Convergence correctness (headline).** **CONVERGED.** Byte-identical resolved
  `SpaceState` across Nodes + every client projection (G-ALIGN), every arrival permutation,
  across Layers 1/4/5c. (See matrix.)
- **M3 — Integrity.** **Zero by construction.** No duplicate `event_id` (event_ids are
  content hashes; `derive_resolved` de-duplicates by id in topo-sort), zero orphans (losers
  retained, not dangling), no unexpected pending-timeouts (the workspace proof feeds complete
  logs; the live-seam ingest accepts both events). No `ERROR`/unexpected `WARN`. (E2E
  content-blindness is S6, not S2.)
- **M4 — Latency (informational; throughput NOT measured, M8-D2).** The convergence proof is
  deterministic/in-process — no network latency to measure. Binary-level per-connection
  latency reference is the S1-A baseline (~600 ms/round-trip, one WS per send). No throughput
  measured (M8-D2 non-goal; blocked on the unbuilt long-lived client mode).

---

## M8-D4 finding (recorded; an M9-scoping input, not an in-arc fix)

**`system.key_rotation` is a dormant forward-ready EventType.** It has a
`state_key_for_event` arm (`xgen-core/src/resolution/state_key.rs:111`, keyed on the
rotating sender) but **no builder and no `apply_event` arm** (confirmed: no
`build_*key_rotation`; no `SystemKeyRotation` arm in `space/state.rs::apply_event`). A
concurrent key-rotation conflict is therefore **not buildable on B without adding wire
surface**, which M8 must not do. The C2 third conflict case substitutes **thread.status
resolved-vs-archived**, which exercises the **same resolution layer (5c)** the key-rotation
case would have. Per **M8-D4**, the unbuilt key-rotation path is a **success-shaped finding**
that feeds M9 (multiparty redesign) / the Arc-H real-crypto track — not an in-arc redesign.

---

## S2 binary-half (operator-realistic federation send) — scope

The deterministic M2/G-ALIGN headline is fully delivered at the workspace level (above; the
M8-D6-correct home). The remaining S2 binary-half — two **federated** real `xgen-node.exe`
with a client each, concurrently emitting conflicting state events, asserting byte-identical
resolved state on both Nodes + a real `xgen-client members` G-ALIGN check + M1 delivery /
M4 latency under real concurrency — is recorded as a **scoped binary-level run** for the
suite-execution pass. Rationale (honest, D-065): (a) the convergence claim it would make is
already proven exhaustively + order-independently here, so real processes add no convergence
signal (M8-D6); (b) the CLI exposes no `prev_events` control, so a *controlled* binary
conflict depends on genuine concurrent dispatch timing (the S1-A 6/300 WS-race surface) —
its unique value is delivery/latency realism, which the S1-A baseline already characterizes;
(c) two-fresh-node federation initiation has no single clean CLI verb today (federation
auto-establishes via handshake/reconnect; `federation initiate` is known-peer only, J-177) —
a federation-orchestration item appropriate to the binary suite-execution pass, not a C2
blocker. This is a **placement decision, not a coverage gap**: the convergence headline is
complete.

---

## Definition of Done — C2

- [x] Conflict matrix recorded (3 layers, all CONVERGED).
- [x] **M2 verdict**: CONVERGED (byte-identical across Nodes + client projections, every
  arrival permutation; Layers 1/4/5c).
- [x] **G-ALIGN verdict**: HOLDS (client A-pure projection == Node resolved view, all 3
  cases; structurally — these layers don't consult the home-node map).
- [x] **M1** characterized (loss-free resolution; losers retained in DAG) + **M3** zero by
  construction.
- [x] **M4** recorded (informational; deterministic proof has no network latency; S1-A
  reference noted). Throughput not measured (M8-D2).
- [x] key-rotation substitution recorded as an M8-D4 finding (M9 input).
- [x] CP-4 placement rationale recorded; S2 binary-half scoped.
- [x] `cargo test --workspace` 1159/0/2; clippy clean both feature sets.

---

*End of MULTIPARTY_S2_findings.md — C2 complete. Wave 1 (C1 + C2) done → Joe-lock
checkpoint #1.*
