# Multiparty Test S3 — Findings (M8 / Wave 2 / C3)
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

The **C3** result of M8 Wave 2 (runbook `tasks/M8_MULTIPARTY_IMPL.md` §4 C3; design
`tasks/M8_MULTIPARTY_DESIGN.md` §3 S3 row). S3 covers three federation aspects across ≥3
Nodes: **transitive propagation**, **jurisdiction policy** (Arc G), and **migration**
(Arc F). First baseline (no historical "A"; M8-D3). B stamp: `8b14aa8` (≡ `676b9c1`).

---

## Headline finding (M8-D4 → M9 input): federation is anti-transitive, not chain-transitive

**The S3/S0 premise of *transitive* propagation to a *non-adjacent* Node does not match the
built model.** Federation is **anti-transitive**: the F-5 guard at the top of
`apply_federation_push` (`xgen-node/src/federation_session.rs:266-279`) returns immediately
for any `EventOrigin::ReceivedViaFederation` event — a Node **never re-forwards** a
federation-received event to its other peers. So in a **chain** topology (A↔B, B↔C federated,
A↔C **not**), Alice's event on A reaches B (direct push) but **does not** reach C — B does not
relay it. A multi-Node Space is therefore served by **full-mesh** federation + anti-transitive
delivery (every hosting Node pushes directly to every peer; no Node duplicates by
re-forwarding), NOT by chain-with-transitive-forward.

This is a **clarification, not a bug**: anti-transitivity is the *correct* behaviour under a
full mesh (it prevents duplicate delivery). It surfaces the long-standing **spec §3.2
"forward on accept" gap** the S0 file already flagged (S0 §"S3 — spec gap"). Per **M8-D4**, a
surfaced-weakness/clarification is a **success** that feeds M9 (the multiparty redesign should
make the topology model — mesh + anti-transitive — explicit and reconcile the S3/S0
"transitive (non-adjacent)" framing). Recorded; **not redesigned in-arc.**

What *does* hold (and is proven): in a topology where A is federated with **both** B and C
(`A↔B`, `A↔C`), Alice's event on A reaches **both** B and C by direct push, and B↔C
anti-transitivity holds — proven by `phase9_three_node_anti_transitivity.rs` (assertion #2:
"every event posted by Alice on A appears in C's runtime store"; assertions #1+#3: the F-5
no-leak side). S3 references that proof for the 3-Node delivery + anti-transitivity aspects
rather than duplicating it.

---

## Aspect coverage

| Aspect | Source | Verdict |
|---|---|---|
| 3-Node multi-node delivery (A → {B, C} direct) | `phase9_three_node_anti_transitivity.rs` assertion #2 (referenced) | **PASS** (existing) |
| Anti-transitivity / F-5 (chain B↔C no-leak) | `phase9_three_node_anti_transitivity.rs` assertions #1 + #3 (referenced) | **PASS** (existing) — and the headline M9 finding above |
| Transitive (non-adjacent, chain A→C via B) | F-5 guard `federation_session.rs:266-279` | **NOT a capability** → M8-D4 / M9 input (headline) |
| Jurisdiction reject (Arc G PG-04) | `federation_policy_enforcement.rs::inbound_jurisdiction_drops_excluded_space_event` (referenced) | **PASS** (existing) |
| **Migration multiparty convergence (Arc F PG-11)** | **`m8_s3_federation.rs` (NEW, this commit)** | **PASS** |

---

## Jurisdiction (Arc G PG-04) — referenced proof

Cross-jurisdiction containment is proven end-to-end by the existing workspace test
`inbound_jurisdiction_drops_excluded_space_event`: two federated Nodes; A hosts a Space with
`jurisdiction = "RU"` and another with `"SK"`; B sets `FederationPolicy.allowed_jurisdictions
= ["SK"]` for A; A pushes a message in each Space on one ordered channel; **B ingests the SK
message and DROPS the RU message pre-apply** (`jurisdiction_permits` AND-composed with
`policy_permits` at both inbound + outbound sites; `federation_policy.rs:178-209`). Undeclared
Spaces fail a restrictive set (strict undeclared-denied; DM Spaces declare `None`, AG-D4). M8
references this as the S3 jurisdiction proof (no duplicate authored).

---

## Migration (Arc F PG-11) — new proof (`m8_s3_federation.rs`)

The `state.space_migrate` cutover applier (`apply_space_migrate`,
`xgen-core/src/space/state.rs:1104`) flips `SpaceState.home_node` source→dest under the AF-D2
authority gate (`sender == home_node`) and is reachable via the plain `ingest` →
`derive_resolved` → `apply_event` path (the 12-message migration driver is the *transfer*
orchestration; the cutover applier is independent). Two tests:

| Test | What it proves | Verdict |
|---|---|---|
| `s3_migration_flips_home_node_and_converges_across_three_nodes` | A creates a Space (home=A) with a Room + member; an A-authored cutover flips `home_node` to B; after all three Nodes ingest it, **`home_node == B` on all three**, every Node's resolved `SpaceState` is **byte-identical**, and the migrated members/rooms survive. | **PASS** |
| `s3_stale_source_remigrate_rejected_on_all_nodes` | After A→B, a stale A-signed re-migrate (A no longer home) is **rejected** by the applier (`PermissionDenied`) on every Node — `home_node` stays B, both Nodes still converge. The AF-D2 self-protecting authority gate, multiparty. | **PASS** |

```
$ cargo test -p xgen-node --lib m8_s3
running 2 tests
test tests::m8_s3_federation::tests::s3_stale_source_remigrate_rejected_on_all_nodes ... ok
test tests::m8_s3_federation::tests::s3_migration_flips_home_node_and_converges_across_three_nodes ... ok
test result: ok. 2 passed; 0 failed; ...
```

Full workspace after Wave 2: `cargo test --workspace` → **1162 passed / 0 failed / 2 ignored**;
0 build errors; clippy clean default **and** `--all-features`.

---

## The four metrics (M8-D2)

- **M1 — Delivery completeness.** Characterized: 3-Node direct delivery (A→{B,C}) holds
  (existing anti-transitivity test assertion #2). Transitive/chain delivery is **not a
  capability** (F-5) — the headline finding, an M9 input, not a delivery loss within the
  built model.
- **M2 — Convergence correctness.** **CONVERGED** for migration: byte-identical resolved
  `SpaceState` across all three Nodes pre- and post-cutover (`home_node` flip preserves
  convergence); jurisdiction filtering converges to the policy-correct per-Node view.
- **M3 — Integrity.** Zero orphans/duplicates by construction; the stale-remigrate test
  confirms a rejected cutover does not corrupt or fork state. No `ERROR`/unexpected `WARN`.
- **M4 — Latency (informational; throughput NOT measured).** Migration cutover + jurisdiction
  filtering are in-process/deterministic — no network latency to report. Binary-level
  federation latency reference is the S1-A baseline.

---

## CP-4 placement + binary-half scope

Per CP-4/M8-D6, the rigorous federation/jurisdiction/migration **correctness** proofs are
workspace-homed (deterministic; real processes add no signal — and the convergence math is the
same property C2 proved exhaustively). The operator-realistic **binary-level** S3 run (real
federated `xgen-node.exe` + `migration initiate` CLI verb, `MIG_6010/6011`) is scoped to the
binary suite-execution pass. Honest note (D-065): two-fresh-Node federation has no single clean
CLI initiation verb (auto-establishes via handshake/reconnect; `federation initiate` is
known-peer only, J-177) — a federation-orchestration item for the binary pass, itself an
M9/operator-tooling input, **not a C3 blocker**. Migration *is* CLI-reachable
(`migration initiate`), so the binary migration run is the most readily runnable S3 binary
piece.

---

## Definition of Done — C3

- [x] Transitive/anti-transitivity recorded — F-5 anti-transitive model is the headline M8-D4
  finding (M9 input); 3-Node direct delivery referenced (existing proof).
- [x] Jurisdiction reject — referenced existing end-to-end proof (cross-jurisdiction Space
  dropped at peer per `allowed_jurisdictions`).
- [x] **Migration `home_node` flip verified on all Nodes + post-migration convergence** (new
  test) + AF-D2 stale-remigrate rejection.
- [x] M1 characterized · M2 CONVERGED · M3 zero · M4 recorded (informational).
- [x] CP-4 placement + binary-half scoped.
- [x] `cargo test --workspace` 1162/0/2; clippy clean both feature sets.

---

*End of MULTIPARTY_S3_findings.md — C3 complete.*
