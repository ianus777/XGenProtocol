# M8.7 — Concurrent-Commit Resolution: Design
> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & scope

Design for **M8.7 — concurrent-commit resolution** (the **R** of the audit's S/L/R split). Closes the A4 gap: a concurrent `mls.commit` race at one frontier has no conflict domain, so fold order silently decides the canonical commit. This design makes `mls.commit` a first-class conflict domain so every node converges deterministically on the same winning commit.

**Locked scope (CC-D1, supersedes audit §6 "R+S"):** **M8.7 = R only.** S (real crypto primitive swap) folds into the L arc — the real MLS key schedule is produced by the openmls group object (ratchet/secret tree), inseparable from the production-client lifecycle. R is crypto-agnostic: it resolves the **opaque Node-tracked epoch state** (`RoomState.mls_epoch` + the new `mls_commit_tip`), a DAG-level property independent of the key schedule.

**R = three small things (CC-D2 + CC-D5):** a `MlsCommit` arm in `state_key_for_event`, a `RoomState.mls_commit_tip` field, and `apply_mls_commit` recording the resolved winner's id. **This is design-only.** No code, no DECISIONS change (arc-local, CC-D#, D-069). Authored + Joe-LOCKED; CC-D5 added at J-301; next-active = runbook.

---

## 2. The problem (A4, grounded)

`state_key_for_event` (`xgen-core/src/resolution/state_key.rs`) returns `None` for `EventType::MlsCommit` (its comment even claims epoch advances introduce no state key). `apply_mls_commit` (`state.rs:825`) takes `content["epoch"]` and sets `mls_epoch = Some(epoch)` unconditionally; its comment fences the race to D3. Under two members committing `N → N+1` at one frontier, `derive_resolved` applies both with no arbitration.

**The subtlety that drives CC-D5 (vacuity):** two *honest* concurrent commits both read epoch `N` and advance to `N+1`, so they carry the **same** `target_epoch`. `mls_epoch` therefore lands at `N+1` under **either** fold order — the counter alone can never diverge for honest commits, so a counter-only proof is **vacuous** (green with or without the fix). The real divergence is **which commit is canonical** for the transition — an identity currently unobservable in Node state. CC-D5 makes it observable.

---

## 3. The resolution mechanism (grounded — R reuses it, adds no new resolution code)

`derive_resolved` (`derive.rs:76`): groups the log by `state_key_for_event` → restricts each group to its causal **frontier** (`frontier_of`) → `resolve()` (`algorithm.rs`) picks the winner → winners + unconflicted events fold into the convergent `SpaceState`; losers are excluded (CP-A). The node builds state from `derive_resolved` (`runtime.rs:460`, `:602`) — cold-start convergent. Tiebreak with no semantic priority = **Layer-5c lexicographic by `event_id`** (`derive.rs:36/59`); `event_id` is a content hash → identical on every node → deterministic convergence.

**Consequence:** giving `MlsCommit` a state key routes commits through the proven membership/room-update path with zero new resolution code.

---

## 4. The fix (CC-D2 / CC-D3 / CC-D5) + change surface

**CC-D2 — state key = `(room, target_epoch)`.** Add an `MlsCommit` arm to `state_key_for_event`: category `"state.mls_commit"`, key_field `"{room_id}:{target_epoch}"` (`target_epoch = content["epoch"].as_u64()`; absent/malformed ⇒ `None`, matching the applier's silent-no-op). **Why target_epoch, not per-room:** per-room would group *all* commits, so a later `2 → 3` commit and a losing `1 → 2` commit (mutually frontier-concurrent) would compete and the tiebreak could pick the epoch-2 loser — an **epoch regression**. Keying by target epoch means only same-transition commits group; sequential advances are different keys. (The frontier filter is a second line of defence; the key already eliminates the hazard.)

**CC-D3 — Layer-5c lexicographic tiebreak, no Layer-1.** Two concurrent epoch advances carry no semantic priority; the existing lexicographic-by-`event_id` tiebreak is the deterministic, cross-node-convergent winner rule.

**CC-D5 — record the canonical winning commit (the observable).** Add `RoomState.mls_commit_tip: Option<EventXgid>` next to `mls_epoch`. `apply_mls_commit` sets `room.mls_commit_tip = event.event_id.clone()` alongside `mls_epoch = Some(epoch)`. Because `derive_resolved` admits only the resolved **winner** to the applied set, the tip records the winner; the loser is excluded and never applied. The field is an order-independent scalar, so it **rides `RoomState`'s existing `PartialEq`/`Eq` M8 convergence oracle additively** (mirrors `mls_epoch`). **`RoomState` is NOT serialized** (`#[derive(Debug, Clone, PartialEq, Eq)]` only; rebuilt from the log each load), so **no `serde(default)` / persistence migration is needed.**

**`apply_mls_commit` — sets tip; comment corrected.** Logic gains only the tip write; the loser-exclusion is automatic via the resolved set. The D3-fencing comment (`state.rs:810-824`) and the stale `state_key.rs` epoch-advance comment + the `_ => None` "mls.\*" comment are corrected (D-065 honesty: `mls.commit` now has a state key; `mls.welcome`/`mls.proposal` still do not).

**Grounded change surface (one commit):**
- `xgen-core/src/resolution/state_key.rs` — `MlsCommit` arm (CC-D2) + two comment corrections.
- `xgen-core/src/space/state.rs` — `RoomState.mls_commit_tip` field (+ `None` at the RoomState constructors and at `state.mls_group_init` genesis) + `apply_mls_commit` tip write (CC-D5) + comment correction.
- One impl-time verification: confirm `apply_mls_commit` is invoked solely for the resolved winner on the live node path (joins the membership resolved-application path; a one-line trace confirms it). **Checkpoint.**

---

## 5. In-process proof plan (observable + sensitive; no openmls client)

- **state_key units:** two commits with the same `target_epoch` share a key; different `target_epoch` do not; **regression guard** — a `2 → 3` commit and a losing `1 → 2` commit do **not** share a key.
- **resolution unit:** a frontier of two same-`target_epoch` commits resolves to one deterministic (lexicographic) winner.
- **headline two-`NodeRuntime` convergence repro:** two members commit concurrently `1 → 2` in one Room; both nodes (independent ingest order) converge — asserted via the **`RoomState` `Eq` oracle**: same `mls_epoch = 2` **and** same `mls_commit_tip` (the winning `event_id`); the loser is excluded on both.
- **sensitivity witness (records why the design needed CC-D5):** revert the `MlsCommit` arm → both commits become unconflicted → both apply in fold order → `mls_commit_tip` = last-applied → the two nodes' `RoomState` diverge on the tip (**RED**); restore (**GREEN**). This is the test that would have stayed green under a counter-only design — the proof that CC-D5 earns its place.

---

## 6. §-locks (Joe-LOCKED)

- **CC-D1** — M8.7 = **R only**; S folds into the L (production openmls-client) arc. Supersedes audit §6 "R+S".
- **CC-D2** — `mls.commit` state key = `(room, target_epoch)` (`category "state.mls_commit"`, `key_field "{room}:{epoch}"`).
- **CC-D3** — Layer-5c lexicographic-by-`event_id` tiebreak; **no** Layer-1 priority rule.
- **CC-D4** — home-DS commit serialization is **L-side** (live-delivery optimization), not an R convergence requirement. F-B "hybrid" splits: DAG-tiebreak = R, home-DS-serialize = L.
- **CC-D5** *(added J-301)* — `apply_mls_commit` records the resolved winner in `RoomState.mls_commit_tip: Option<EventXgid>` (rides the M8 `Eq` oracle; `RoomState` unserialized → no migration). Makes R observable + the witness sensitive; without it the counter-only proof is vacuous.

---

## 7. Coverage ledger / honest boundary (D-065)

- **`mls_commit_tip` is R/Node-side, not L.** It records the DAG-level *identity* of the authoritative commit — which every federated node must agree on (convergence). The Node never interprets the commit's crypto; it tracks which one won.
- **Loser rollback-and-replay is NOT exercised by R.** R proves every node *agrees on the winner* (no permanent fork). The loser client detecting its loss and rebuilding (re-derive from the winner, re-apply its proposal → epoch N+2) is **L** (real openmls client). The in-process proof demonstrates convergence-on-winner, not loser-recovery — the Arc H C1 Finding 1 analogue. Named, not glossed.
- **S (real key schedule / HPKE) + home-DS serialization deferred to L** (CC-D1 / CC-D4).

---

## 8. Out of scope / untouched

- Real RFC 9420 / openmls crypto, group-state persistence, KeyPackage generation, Credential↔XGID, `ops::send` live-encrypt, loser-rebuild — all **L**.
- Validation of commit well-formedness (honest commits target current+1); a divergent `target_epoch` is a validation concern orthogonal to R's convergence.
- `MlsGroupInit` state key + the `mls_epoch=Some(0)` genesis — unchanged (genesis leaves `mls_commit_tip = None`).
- Node DS blindness invariant — preserved.

---

Per D-065 (vacuity caught → CC-D5; R-only scope; loser-rebuild honesty) + D-069 + D-071 + D-074. Next-active: runbook (`tasks/M8_7_CONCURRENT_COMMIT_IMPL.md`) → Clair. Not pushed — Joe pushes.
