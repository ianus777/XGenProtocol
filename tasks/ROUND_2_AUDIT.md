# Round 2 — Whole-Codebase Coherence Audit (UI Gate)
> **Status**: COMPLETE  
> Version: 1.3  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

Round 2 is the **GATE** before UI: a single additive, semi-redundant sweep across the
entire shipped codebase, run **after every Round-1 D-071 arc closed** (Arc H / PG-05
interface-locked at J-257; suite 1153/0/2; gap register Open 1/13). It is **not an arc**:
it builds no fixes. Output = this audit doc + the findings register (§5) + the UI go/no-go
verdict (§6). Any finding needing work spawns its own arc, or feeds M10. Read-only and
additive — that is what the locked two-round principle requires.

**Locked scope (ROADMAP, recorded 2026-06-04, Joe).** Re-checking already-audited surfaces
is the accepted price of catching what per-arc audits structurally cannot: **cross-arc
interactions** (visible only once all arcs exist) and the **client / adjacent-crate
surfaces** that per-arc audits under-ground. Absorbs as in-scope subsets the **M8
client-side impact** (SR-D3 node-side resolution with client apply sites untouched) and the
**Arc-E client/node check**. Principle: prefer semi-redundant checks over gluing
cross-cutting concerns into an already-locked arc; the only non-gluing lever on foundational
concerns is **arc ordering**, decided at selection time.

**Out of scope.** **M10** (Tier-1 Auth Module reference set) — unstarted/unbuilt, nothing to
audit; it is downstream of this gate. Round 2 audits **shipped code only**.

**Parked items this gate must absorb** (grounded homes): the M8 number collision (R2-F05),
the operator-terminology correction (R2-F06), the multi-device seam (R2-F09), and the
Arc-F/Arc-G "Round-2-homed" carry-ins (R2-F07).

---

## 2. Surfaces swept (Joe-confirmed 2026-06-05)

Crates: `xgen-common` · `xgen-core` · `xgen-node` · `xgen-client` · `xgen-store-sqlite`.
Plus `docs/` canonical + `tasks/` record coherence.

Axes (each grounded by grep/read against the live tree on `main`):
1. Cross-arc state-mutation coherence — every `apply_event` arm + `state_key_for_event`
   conflict-domain + their interaction under M8 `derive_resolved`.
2. Client/node seam — the SR-D3 client-trusts-node assumption vs all post-M8 arcs.
3. Wire-code register integrity — the renumber history across all bands.
4. Dormant-but-correct inventory — every "live hook, no-op until X".
5. Doc-vs-code drift — ch3/ch4/Appendix C/I as-built since M6.
6. Terminology + numbering hygiene.

---

## 3. Per-axis findings (grounded)

### 3.1 Cross-arc state-mutation coherence
`apply_event` (`xgen-core/src/space/state.rs:561-614`) and `state_key_for_event`
(`xgen-core/src/resolution/state_key.rs:44-120`) were read in full. **Conflict domains are
non-overlapping:** `membership` (space:identity-or-target) · `state.room_update` (room) ·
`state.space_update` (space; applier is the SR-F2 no-op) · `thread.status` (thread) ·
`state.node_priority` (space) · `state.mls_group_init` (room) · `system.key_rotation`
(identity). The apply chokepoint is explicit and inert for forward-compat: `Unknown(_) =>
Ok(())` and `_ => Ok(())` (state.rs:613-614).

- **Set-once Space fields ride M8 cleanly.** `jurisdiction` (AG-D3) and `e2e_encryption`
  (AH-D2) carry no applier / no `state_key` arm; they ride `derive_resolved` via SpaceState
  `PartialEq`. Convergence pinned at each arc's close. **No interaction.**
- **`RoomState.mls_epoch` has no `state_key` arm** (AH-D4): epoch-advance rides membership
  resolution; a single-committer linear chain folds deterministically. This is correct for
  Phase-2, but is **identity-membership-shaped** — see R2-F09 (multi-device seam).
- **`state.space_migrate` has no `state_key` arm** (AF-D1/D2): causally-terminal singleton,
  self-protecting `sender == home_node` gate. Terminal-by-construction; no conflict domain.
- **POSITIVE RESULT — e2e/mls × migration is clean.** `migration_driver.rs` transfers via
  `range(0)` → `append` → `rehydrate_space_from_store` (which uses `derive_resolved`, C2).
  `mls_group_init` / `mls_commit` events live in the DAG, so `mls_epoch` + `e2e_encryption`
  rebuild correctly on the destination. No cross-arc bug.

### 3.2 Client/node seam (M8 client-side impact — absorbed)
The client does **not** consume node-resolved snapshots; it **replays the DAG locally**.
The replay (`xgen-client/src/ops.rs:1304-1353`) sorts events by **timestamp** (root-first)
and applies each via **plain `apply_event`** (Phase-1 last-write-wins) — it does **not**
call `topological_sort`, `find_conflicts`, `resolve`, or `derive_resolved`. Same pattern at
`ai_service.rs:295` and the projection helper `ops.rs:1503-1564`. This is the SR-D3 lock
("clients consume node-resolved state; client apply sites untouched") surfacing as a real
seam: under genuine concurrent same-key conflict — or clock skew, since the client orders by
timestamp not causal DAG — the client's local SpaceState can **diverge** from the node's
resolved view. Node stays authoritative; impact is client-local views (pacing, AI-context,
ops reads). → **R2-F01**.

### 3.3 Wire-code register integrity
Bands **30xx** (3010/3011/3020/3030/3041/3042/3043), **40xx** (4001-4007 resolution),
**50xx** (5001/5002 MLS KeyPackage; 5003-5005 spec-only/dormant) are **clean — no
collisions, no orphans**. The **60xx migration band** drifts:
- Spec ch3 §3.12.11 (`docs/xgen_ch3_specification.md:4246-4254`): 6001-6006 + **6007
  `migration_verification_failed`** + **6008 `migration_in_progress`** + **6009
  `migration_authority`**.
- Code: `state_machine.rs:67-72` emits 6001-6006; `exchange.rs:116` emits **6009**
  (matches); but `verification.rs:30-31` emits verification failures as **6010
  `EventCountMismatch` + 6011 `TipsMismatch`** — **not** the spec's 6007. No emitter for
  6007 or 6008 anywhere in code.
- The Arc-F close added 6009 to the spec table but never reconciled the code's 6010/6011, nor
  retired/repurposed the orphaned 6007. → **R2-F02**. Residues: stale `// wire 6007` comment
  at `phase_arcf_migration_e2e.rs:168` (→ **R2-F03**); numeric 6010/6011 (`verification.rs`)
  vs prefixed string codes `MIG_6010`/`MIG_6011` (`admin_ops.rs:1860-1867`) reuse the same
  integers across distinct namespaces (→ **R2-F04**).

### 3.4 Dormant-but-correct inventory
Each verified honestly **inert, not silently broken** (catalogue, no finding): jurisdiction
federation hook (no-op until an operator declares `allowed_jurisdictions`) · tier-gate
(`verify(1,1)=Ok` Tier-1 no-op) · KeyPackage durability (replay re-adds a consumed package,
fenced D3) · dormant migration admission (6003/4/5) · assertion trusted-list (empty default
+ Local-Node bypass). All documented + tested as such at their arc closes.

### 3.5 Doc-vs-code drift
The per-arc closes reconciled ch3/ch4/Appendix C/I as-built. The one residual drift found is
the 60xx band (§3.3 → R2-F02). No other normative drift surfaced in the swept surfaces.

### 3.6 Terminology + numbering hygiene
- **M8 number collision** — two live ROADMAP "M8" entries (closed state-resolution
  convergence J-241; pending multiparty A/B-metrics placeholder, ROADMAP L753). ROADMAP L229
  already reads "A/B metrics → M9", so the placeholder looks already-absorbed into M9. →
  **R2-F05**.
- **Operator terminology** — "operator" governed by D-082's four-sense classifier — only Sense D (the `--batch` admin principal) renames to administrator/admin; the infra/custodian sense (C) KEEPS "operator" (D-082 §1 forbids an owner/admin alias). [F06-A1 correction, 2026-06-05: this line previously misstated the rule as "collapses to owner/admin"]. Blast radius: **~133 code
  identifiers** (excl. `ai_operator`) + **~194 doc occurrences**. Large + mixed. → **R2-F06**.

---

## 4. Carry-ins absorbed (Arc-F / Arc-G → "Round-2-homed")
Named at the Arc-F and Arc-G closes as homed to this gate, captured here as **R2-F07**:
(1) migration **sibling-drift** (the migration terminal singleton vs sibling Spaces);
(2) the **deferred Arc-G federation-block**. Note from the Arc-F close: `federation
show-policy` was already patched and is **NOT** part of the carry-in. Exact characterization
to be expanded against the Arc-F/Arc-G close notes when a fix-arc opens.

---

## 5. Findings register

Severity: S1 (critical) · S2 (significant) · S3 (moderate) · S4 (minor).
Status: 🟪 OPEN · ✅ DONE (gradually updated as fix-arcs land).

| ID | Sev | Status | Finding | Disposition |
|----|-----|--------|---------|-------------|
| R2-F01 | S2 | ✅ DONE | Client/node resolution divergence — client replayed via timestamp-ordered plain `apply_event` (`ops.rs`, `ai_service.rs`), not `derive_resolved`/topo-sort; could diverge from node-resolved state under concurrency or clock skew. | **CLOSED at J-264 (fix-arc, A-pure).** All three client sites now re-derive via the proven Arc-C `derive_resolved` with an empty `identity_home_nodes` map + vantage `""` (C1 `ops.rs` read paths J-262 · C2 `ai_service.rs` AI-inbound gate J-263). **F01-D5 reachability probe = positive-in-principle:** under the empty map Layers 3/5a/5b structurally abstain, so a concurrent + same-event-type + same-role + cross-home-node membership/key-rotation conflict (reachable only in a federated multi-home Space; single-home = negative) is decided 5c-lexicographic on the client vs 3/5a/5b on the node. Node stays authoritative; client view = local projection. The named, UNBUILT **A+thin-fetch** escalation is **flagged** for a future decision (a flagged decision, not an auto-build — D-065/F01-D2). |
| R2-F02 | S3 | ✅ DONE | 60xx migration code/spec drift — spec §3.12.11 has 6007/6008; code emits verification failures as 6010/6011 (`verification.rs:30-31`), no 6007/6008 emitter; 6009 added to spec at Arc-F close, 6010/6011 never reconciled, 6007 orphaned. | Doc-reconcile (dormant subsystem; implementer-facing). |
| R2-F03 | S4 | ✅ DONE | Stale `// wire 6007` comment at `phase_arcf_migration_e2e.rs:168` (actual 6009; assertion checks only `Rejected(_)`, behaviour correct). | Fold into R2-F02. |
| R2-F04 | S4 | ✅ DONE | 60xx numbering reuse — numeric 6010/6011 (`verification.rs`) vs string `MIG_6010`/`MIG_6011` (`admin_ops.rs:1860-1867`); distinct namespaces, no functional collision. | Note; optionally renumber MIG_ strings under R2-F02. |
| R2-F05 | S3 | ✅ DONE | M8 number collision — closed state-resolution convergence (J-241) vs pending multiparty A/B-metrics placeholder (ROADMAP L753); record already routes A/B metrics → M9. | Doc-only; rename placeholder → M9-pass or retire (Joe's call). |
| R2-F06 | S3 | ✅ DONE | Operator-terminology correction — repurpose "operator" → delegated AI-running user; old node-custodian sense → owner/admin. ~133 code + ~194 doc occurrences. | **CLOSED at J-266 (zero-rename).** Classification ledger (audit + design) found the active corpus already D-082-compliant: Sense-D was renamed by J-150 (admin-ops + aicontrol); the 3 audit-flagged candidates (ch6:342, appendix_e:92, lifecycle_states:141) were NOT renamed (full context = console/node-operator sense; a piecemeal rename would break chapter consistency; `lifecycle_states.md` is a superseded draft). Joe ruling: console-operator added as D-082 Sense E (keep). Arc = F06-A1 register correction + D-082 Sense-E refinement. No code. |
| R2-F07 | S4 | ✅ DONE | Arc-F/Arc-G "Round-2-homed" carry-ins — migration sibling-drift + deferred Arc-G federation-block (`federation show-policy` already patched, not part of carry-in). | Absorb; expand against close notes when a fix-arc opens. |
| R2-F09 | S3 | ⤴ PULLED | Multi-device seam — AH-D4 epoch-advance is identity-membership-shaped with no own `state_key`; device-level add/remove is a real seam a future multi-device arc breaks (downstream of D3). | **PULLED from the gate (2026-06-05, Joe).** Not a UI blocker (D3-gated); the multi-device seam is real but downstream of everything in the locked M8 → M9 → Multiparty-tests → M10 → UI chain. **Relocated to a future multi-device arc**, to be motivated by the UI prototype when it exercises device-level add/remove. Round 2 closes COMPLETE with this pulled, not left unresolved. |

*(R2-F08 intentionally unallocated — the dormant-but-correct inventory of §3.4 is a clean
catalogue, not a finding.)*

---

## 6. UI go/no-go verdict — **GO** (Round 2 COMPLETE 2026-06-05)

The codebase is **coherent**: state-mutation conflict domains are non-overlapping and
convergent under M8; cross-arc interactions check clean (notably e2e/mls × migration);
wire-code bands are clean except the 60xx doc-drift; dormant features are honestly inert.

**Only R2-F01 touches UI correctness** — UI will read the same client projection that can
diverge from node-resolved state under concurrency. Everything else is doc-only (F02-F05,
F07), a large-but-non-blocking terminology pass (F06), or D3-downstream/dormant (F09).

**Verdict: UI may start, but R2-F01 should be a named fix-arc that lands before — or runs
alongside — correctness-sensitive UI views.** Suggested post-gate ordering: Round 2 →
**R2-F01 fix-arc** → M10 → UI (R2-F01 may run parallel to M10). Doc-only findings
(F02-F05, F07) can be cleared in a single housekeeping pass at any time.

**R2-F01 CLOSED at J-264 (2026-06-05).** The UI-correctness blocker is resolved: all three
client sites re-derive through the node's own `derive_resolved` engine (A-pure), so the client
projection converges with the node's resolved view for the L1/L4/L5c-decided conflict classes a
single-home Space can produce. The residual — a cross-home-node 3/5a/5b-decided conflict in a
federated multi-home Space — is the **flagged, UNBUILT A+thin-fetch** escalation, not a UI
blocker (node authoritative; client = local projection). UI may now proceed past this gate.

---

## 7. Status & next-active

Round 2 (gate) **COMPLETE** (closed 2026-06-05, Joe); audit complete, register §5 tracked.

**Doc-housekeeping pass closed F02/F03/F04/F05/F07 (J-259, 2026-06-05).** Resolutions: **F02** — ch3 §3.12.11 gained a dormant/target as-built note (the spec table is the target scheme; the migration subsystem is dormant and emits free-text `reason` strings; internal codes `error_code` 6001–6006 + verification 6010/6011 are provisional; only 6009 is wired live) — **annotation, not a table rewrite**, and **no code renumber** (that is a future code arc when migration activates). **F03** — the stale `// wire 6007` test comment corrected to 6009 (zero-behavior comment-only edit). **F04** — the 6010/6011-vs-`MIG_6010/6011` reuse documented + accepted (distinct namespaces, no functional collision). **F05** — milestone naming stabilised: M8 = A/B metrics, M9 = Multiparty Redesign, multiparty test = deliberately unnumbered; the closed Arc-C entry's borrowed M8 label vacated; a milestone-naming tree folded into ROADMAP's visual-tree section. **F07** — the Arc-F/Arc-G carry-ins homed in §4 here; their reconcile (federation verb corpus vs `admin_ops`; migration sibling-drift) rides a future migration/federation doc-arc. **The gate itself shipped no code; this pass made one zero-behavior test-comment fix (F03).**

**R2-F01 fix-arc CLOSED at J-264 (2026-06-05).** A-pure client re-derive shipped across C1 (`ops.rs` read paths, J-262) + C2 (`ai_service.rs` AI-inbound gate, J-263) + this doc-only close; all three client sites route through `derive_resolved`. F01-D5 reachability probe recorded **positive-in-principle** (federated multi-home only) → A+thin-fetch flagged as the named UNBUILT escalation (not auto-built; D-065). Suite 1156/0/2 (+3 over the gate's 1153). No DECISIONS change (F01-D# arc-local, D-069). Task docs (audit/design/runbook) → COMPLETED.

**Register at the R2-F06 close was Open 1/9** — R2-F09 (multi-device seam, D3-gated) only (R2-F06 CLOSED — see below). **Next-active then: M10** → UI. This register is the durable artifact; statuses flip 🟪→✅ as fix-arcs close.

**R2-F06 fix-arc CLOSED at J-266 (2026-06-05).** Zero-rename. The classification ledger (audit + design) found the active corpus already D-082-compliant. Sense-D was renamed by J-150 (admin-ops + aicontrol); the three audit-flagged candidates (ch6:342, appendix_e:92, lifecycle_states:141) were NOT renamed — full context showed they carry the console/node-operator sense (a piecemeal rename would make the chapters internally inconsistent), and `lifecycle_states.md` is a superseded draft. Arc = two non-rename edits: F06-A1 register correction (this doc, §3.6 + §5) + a D-082 Sense-E refinement (console-operator keep-sense). No code, no further renames. Suite unchanged 1156/0/2. Task docs (audit/design) → COMPLETED.

**Round 2 (gate) CLOSED COMPLETE — 2026-06-05 (Joe).** All UI-gating findings resolved; the one remaining item, **R2-F09 (multi-device seam), was PULLED from the gate** (D3-gated, not a UI blocker) and relocated to a future multi-device arc to be motivated by the UI prototype. Gate-open count: **0**. **Locked post-gate chain: M8 (A/B metrics) → M9 (Multiparty Redesign) → Multiparty tests → M10 (Auth Module reference set) → UI.** Rationale (Joe): keep the multiparty redesign and its tests back-to-back; M10 sits last (most UI-adjacent); UI is built on a **clean table**, not alongside pre-UI work — UI integration regularly kicks issues back into lower layers, so those layers are settled and multiparty-green first. Note: the multiparty tests and the Auth Module (M10) tests are **independent test surfaces, deliberately not entangled** — the multiparty suite exercises convergence/federation under N clients; M10 carries its own exhaustive auth-module test battery. Keeping them separate **eases both milestones** (neither's tests depend on the other's); the ordering (multiparty tests before M10) follows from that separation, not from a compromise. **Next-active: M8.**

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + the two-round audit principle (2026-06-04).
