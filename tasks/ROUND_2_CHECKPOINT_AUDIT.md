# Round-2 Checkpoint — Post-Multiparty Coherence Sweep
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-12  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

A **Round-2-style whole-codebase coherence sweep**, run as a **post-multiparty
checkpoint** — *not* the final pre-UI gate. The sole Round-2 on record
(`tasks/ROUND_2_AUDIT.md` v1.3, J-258, COMPLETE) ran 2026-06-05 at the Arc-H close
(suite 1153), **before the entire Multiparty-tests milestone**. That milestone (R1 J-340 ·
R2 J-348 · R3 J-356) landed a large body of production-crate change the J-258 audit never
saw — so J-258 stands only as a historical record, and this checkpoint re-sweeps the
shipped surface as it stands at the multiparty close.

It is **read-only and additive** — it builds no fixes. Output = this doc + the findings
register (§5) + the checkpoint verdict (§6). Any finding needing work spawns its own arc or
feeds an already-named home.

**Why a checkpoint and not the gate (sequencing, Joe-locked 2026-06-12).** Round-2 is
*defined* as the pre-UI gate, run after **all** pre-UI work, immediately before UI. Per the
reconciled post-multiparty chain — **M10 → M11 → M12 → UI** — the real gate sits **after
M12**. This checkpoint cannot be that gate: M10/M11/M12 are unbuilt, so they are out of
scope here (§2). Its purpose is narrower and disjoint from the final gate: confirm the
multiparty production surface is coherent **before M10 is built on top of it** (D-071 —
audit precedes dependent milestones), while the MP-F# changes are fresh in the record. The
final pre-UI gate (after M12) then sweeps the whole tree **including** M10/M11/M12 and their
cross-arc interactions — the only point those become visible.

## 2. Scope (Joe-locked 2026-06-12)

**In scope — shipped code only**, the surface as it stands at HEAD `5709df3` (J-356):
crates `xgen-common` · `xgen-core` · `xgen-node` · `xgen-client` · `xgen-store-sqlite`, plus
`docs/` canonical coherence. Emphasis on the multiparty production deltas (MP-F2 reject
plumbing · MP-F5 client-reply codes · MP-F4 frontier + room-scoped membership `state_key` ·
MP-F7 client causal anchoring · MP-F1b/MP-F11 Design-Z `federation_nodes` · MP-F14
cooperative-frontier infra-exclusion · the four thin verbs + D-092) **× each other × the
pre-multiparty arcs** (Arc-H E2E, Arc-F migration, Arc-E primitives).

**Out of scope — unbuilt, recorded as a boundary, not audited:** **M10** (Auth Module
reference set), **M11** (`self` account), **M12** (attachments). There is no code to sweep
for them; they are downstream of this checkpoint. Their audit happens at the **final pre-UI
gate after M12**. (Same posture J-258 took for M10: "unstarted/unbuilt, nothing to audit.")

## 3. Axes swept (Joe-locked 2026-06-12) — grounded against HEAD `5709df3`

Six axes, each grounded by grep/read against the live tree on `main`.

### 3.1 Cross-arc state-mutation coherence — CLEAN
`state_key_for_event` (`xgen-core/src/resolution/state_key.rs:44-180`) read in full. Conflict
domains are **non-overlapping** and the multiparty additions are coherent:
- **MP-F4 room-scoping (A1)** — `membership_scope_key` keys a space-level join/leave
  (empty `room_id`) room-agnostically and a room-level one with a `room:` infix, so
  space-membership and room-membership of one identity never alias. Kick is scope-aware on
  the same basis (keyed on target); invite / ban / node_eject / node_unban stay space-level
  (no room-level applier branch). Proven by the MP-F4 test block (space-vs-room-join distinct;
  room-kick shares with room-join not space-join; room-leave shares with room-join;
  invite/ban room-agnostic even with a `room_id`).
- **`thread.status`** (resolved/archived share a category → genuine conflict → Layer-5c),
  **`state.mls_group_init`** (per-room), **`state.mls_commit`** (per-`(room, target_epoch)`,
  epoch-regression-guarded, M8.7 CC-D2) all keyed correctly; `mls.welcome`/`proposal`/
  `key_package` keyless; message events keyless.
- **Design-Z `federation_nodes` population** (`xgen-core/src/node/runtime.rs`): two symmetric
  rebuilt-not-persisted paths — `repopulate_dm_federation_nodes` (DM, from members, F1B) +
  `repopulate_regular_federation_nodes` (regular Space, from `federation_relationships`,
  MP-F11/R3-D6) — fired at cold-start (L533-537), rebuild (L688-692, L713-717), and
  membership change (L723-727). The F-3 peer-gate (L1069) is intact; third parties stay
  blocked (the J-333 hole-closed property, generalized at MP-F11). No cross-arc conflict.

### 3.2 Client/node seam — CLEAN (R2-F01 closure survives multiparty)
Both `ops.rs` read paths still route through `derive_resolved(events, "", &{})`
(`ops.rs:1908` + `:2081`) — the R2-F01 A-pure closure (empty `identity_home_nodes`, vantage
`""`). `ai_service.rs` uses the gated `apply_or_rebuild` (`:543`): conflict →
`derive_resolved` rebuild (`:546`), else the incremental fast path (`:550`, the lone plain
`apply_event`, proven byte-identical when no conflict). **MP-F7's `last_local_events`** is
send-anchoring (rejoin `prev_events` selection), **orthogonal to the read/resolution seam** —
no regression of R2-F01. No new client read path bypasses `derive_resolved`.

### 3.3 Wire-code register integrity — ONE FINDING (RC-F-01)
The multiparty-era codes are internally consistent and distinct: `3044 invite_expired` /
`3045 invite_validity_exceeds_max` / `3046 event_timestamp_out_of_bounds`; the 4000-band
generic reject; the 60xx migration band (unchanged since J-258's R2-F02 doc-reconcile). The
one drift is a **3010/3011 double-definition** in ch3 — see §5 RC-F-01 — which is the
already-named MP-F2-followon family and lands in M10's lap.

### 3.4 Dispatch-arm completeness (D-092) — CLEAN
The 4-arm rule (CLI / run-path / batch / **aicontrol**) holds for every multiparty verb-add.
`xgen-client/src/aicontrol.rs` carries the `Box::pin` dispatch arm for **Ban** (L446),
**RoomUpdate** (L452), **Thread** (L458), and **CreateSpace** (L422, the `--auth-tier`
carrier). The empirically-caught failure mode (ban missing its aicontrol arm at J-337) did
not recur.

### 3.5 Dormant-but-correct + multiparty M10-deferrals — CLEAN catalogue
The J-258 §3.4 dormant inventory (jurisdiction federation hook · tier-gate Tier-1 no-op ·
KeyPackage durability · dormant migration admission 6003/4/5 · assertion trusted-list empty
default) is unchanged. Multiparty added named, homed deferrals — **not silently broken**:
MP-C-06 (re-home → M10) · MP-C-16 / MP-F13 (home-node discovery, J-278 → M10) · MP-F6
(swallowed apply-error, runtime.rs:691 → M10) · MP-F12 (departed-signer, own home) ·
MP-F2-followon (unmapped variants flatten to 4000 → M10). All catalogued at the multiparty
close (`tasks/HANDOFF_MP_R3.md` §3 / `tasks/MP_findings.md` v1.20).

### 3.6 Doc-vs-code drift — CLEAN except RC-F-01
Appendix F is current: `ban` (F.x), `room-update`, `thread create/resolve/archive` all
documented with authority + semantics. ch3 §3.6.5 / §3.12.x reject tables carry the
multiparty codes (3044/3045/3046). The one drift is RC-F-01 (§5).

## 4. Carry-forward to the final pre-UI gate
The **final Round-2 gate runs after M12**, immediately before UI, and must:
(1) re-sweep the **whole** tree **including M10/M11/M12** + their cross-arc interactions with
the settled base; (2) **verify RC-F-01 is resolved** when M10 builds the auth-tier codes;
(3) re-confirm this checkpoint's clean axes against the M10/M11/M12 deltas.

## 5. Findings register

Severity: S1 (critical) · S2 (significant) · S3 (moderate) · S4 (minor).
Status: 🟪 OPEN · ✅ DONE.

| ID | Sev | Status | Finding | Disposition |
|----|-----|--------|---------|-------------|
| RC-F-01 | S3 | 🟪 OPEN → M10 | **3010/3011 wire-code double-definition in ch3.** §3.6.5 region (`xgen_ch3_specification.md` L1911-1912) + **code** (`registration.rs:120-122`): `3010 = assertion_identity_mismatch`, `3011 = assertion_claims_insufficient` (Arc E PG-03). §3.11.7 Auth-Module region (L3833-3834) + the L3829 reservation note: `3010 = auth_tier_insufficient`, `3011 = kyc_verification_pending`. Same integers, two meanings. The "3030-vs-3010" question (code uses 3030 `tier_mismatch` for tier-insufficiency; §3.11.7 says 3010) is the same root. | **Confirms + sharpens MP-F2-followon → M10.** Doc-internal + dormant: the §3.11.7 codes are unbuilt M10 (Auth-Module tier) territory; the code is self-consistent (no runtime bug). M10 owns the 3010-3016 auth-module band and reconciles the assertion-vs-auth-tier codes when it builds the tier codes. **Not a UI blocker, not new work beyond what M10 already carries.** Note: J-258 §3.3 marked "30xx clean — no collisions"; that was stale on this — multiparty's MP-F2 work (MP-A-03→3030) is what surfaced it. |

*No other finding. Axes 3.1 / 3.2 / 3.4 / 3.5 / 3.6 are clean catalogues, not findings.*

## 6. Checkpoint verdict — **GO** (Round-2 checkpoint COMPLETE 2026-06-12)

The shipped-through-multiparty surface is **coherent**: state-mutation conflict domains
non-overlapping and convergent under M8 (MP-F4 room-scoping + MlsCommit clean); the client
seam's R2-F01 closure survives MP-F7/MP-F4; the D-092 4-arm rule holds for every multiparty
verb; dormant + M10-deferred items are named and homed; docs are current.

**The single finding (RC-F-01) is doc-internal, dormant, and already-homed to M10** — no new
work, no UI-correctness impact. **The base is coherent to build M10 on.** This checkpoint
shipped no code.

**The final pre-UI gate (after M12) remains the real UI gate** — it sweeps M10/M11/M12 and
verifies RC-F-01's resolution.

## 7. Status & next-active

Round-2 **checkpoint COMPLETE** (J-357, 2026-06-12); register §5 tracked (RC-F-01 OPEN → M10).
**Next-active: M10** (Auth Module reference set — opens its own D-071 Phase-0).

Post-multiparty chain (reconciled J-357): **M10 → M11 → M12 (attachments) → Round-2 final
pre-UI gate → UI → Streams (standalone, post-UI plane).**

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + the two-round audit principle (2026-06-04) +
the checkpoint/final-gate split (2026-06-12).
