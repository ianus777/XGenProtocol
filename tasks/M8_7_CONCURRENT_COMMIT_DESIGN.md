# M8.7 — Concurrent-Commit Resolution: Design
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

Design for **M8.7 — concurrent-commit resolution** (the **R** of the audit's S/L/R split). Closes the A4 gap: a concurrent `mls.commit` race at one frontier has no conflict domain, so fold order silently decides `mls_epoch`. This design makes `mls.commit` a first-class conflict domain so every node converges deterministically on the same winning commit and the same epoch.

**Locked scope (CC-D1, supersedes audit §6 "R+S"):** **M8.7 = R only.** **S (real crypto primitive swap) folds into the L arc** — the real MLS key schedule is produced by the openmls group object (ratchet tree / secret tree), inseparable from the production-client lifecycle, so S cannot ship in an L-less M8.7. R is crypto-agnostic: it resolves the **opaque Node-tracked epoch counter** (`RoomState.mls_epoch`), a DAG-level property independent of whether the key schedule is Phase-2 or real MLS.

**This is design-only.** No code, no DECISIONS change (arc-local per D-069; labels CC-D#). Authored + Joe-LOCKED this session; next-active = runbook.

---

## 2. The problem (A4, grounded)

`state_key_for_event` (`xgen-core/src/resolution/state_key.rs`) returns `None` for `EventType::MlsCommit` — the stale comment even states "Epoch *advances* … introduce no new state key of their own." So `mls.commit` events never form a conflict group. `apply_mls_commit` (`state.rs:825`) takes `content["epoch"]` and sets `mls_epoch = Some(epoch)` unconditionally; its own comment fences the race to D3 ("This applier does not attempt to resolve it"). Under two members committing `N → N+1` at the same frontier, `derive_resolved` applies both with no arbitration → the surviving `mls_epoch` is fold-order-dependent → **two nodes catching up independently can land on different winners** (divergence).

---

## 3. The resolution mechanism (grounded — R reuses it, adds no new machinery)

`derive_resolved` (`derive.rs:76`): groups the log by `state_key_for_event` → restricts each group to its **causal frontier** (`frontier_of` — same-key events with no same-key descendant in the group) → `resolve()` (`algorithm.rs`) picks the winner → winners + unconflicted events fold into the convergent `SpaceState`; losers are excluded (CP-A). The node builds state from `derive_resolved` (`runtime.rs:460`, `:602`) — cold-start convergent. The tiebreak with no semantic priority is **Layer-5c lexicographic by `event_id`** (`derive.rs:36/59`); `event_id` is a content hash → identical on every node → deterministic convergence.

**Consequence:** giving `MlsCommit` a state key is the entire core change. The proven membership / room-update / thread-status path then resolves commits with zero new resolution code.

---

## 4. The fix (CC-D2 / CC-D3) + change surface

**CC-D2 — state key = `(room, target_epoch)`.** Add an `MlsCommit` arm to `state_key_for_event`:
- category `"state.mls_commit"`, key_field `"{room_id}:{target_epoch}"`, where `target_epoch = content["epoch"].as_u64()` (the epoch the commit advances *to*). Absent/malformed `epoch` ⇒ `None` (no conflict domain; matches the applier's silent-no-op contract).
- **Why target_epoch, not per-room (the load-bearing catch):** a per-room key would group *all* commits for a Room, so a later `2 → 3` commit and a losing `1 → 2` commit (mutually concurrent at the frontier) would compete, and the lexicographic tiebreak could pick the epoch-2 loser over the epoch-3 commit — an **epoch regression**. Keying by target epoch means only same-transition commits (both `→ 2`) ever group; sequential advances are different keys and never collide. (The frontier filter is a second line of defence, but the key already eliminates the hazard.)

**CC-D3 — Layer-5c lexicographic tiebreak, no Layer-1 rule.** Two concurrent epoch advances carry no semantic priority (both advance by one), so R adds **no** Layer-1 priority pair. The existing lexicographic-by-`event_id` tiebreak is the honest, deterministic, cross-node-convergent winner rule.

**`apply_mls_commit` — unchanged logic; corrected comment.** It already applies only to events `derive_resolved` admits, so the loser drops out with no applier change. Its D3-fencing comment (`state.rs:810-824`) is corrected (D-065 honesty): the race is now resolved by the conflict domain.

**Stale-comment correction (D-065).** The `state_key.rs` comment claiming epoch advances introduce no state key is rewritten to describe the new arm.

**Grounded change surface (for the runbook):**
- `xgen-core/src/resolution/state_key.rs` — the `MlsCommit` arm (+ comment).
- `xgen-core/src/space/state.rs` — correct the `apply_mls_commit` doc comment only (no logic change).
- One impl-time verification (flagged honest): confirm `apply_mls_commit` is invoked solely for the resolved winner on the live node path (it joins the membership/room-update resolved-application path; a one-line trace confirms it).

---

## 5. In-process proof plan (test targets — runbook fills exact names)

- **state_key units:** two concurrent commits with the same `target_epoch` share a key; with different `target_epoch` do not; **regression guard** — a `2 → 3` commit and a losing `1 → 2` commit do **not** share a key.
- **resolution unit:** a frontier of two same-`target_epoch` commits resolves to one deterministic (lexicographic) winner.
- **headline two-`NodeRuntime` convergence repro:** two members commit concurrently `1 → 2` in one Room; both nodes (independent catch-up order) converge on the **same** `mls_epoch = 2` **and** the same winning `event_id`; the loser is excluded on both.
- **sensitivity witness (project discipline):** revert the `MlsCommit` arm → the two nodes can diverge on winner / `mls_epoch` (RED); restore (GREEN). Recorded at close.

No live openmls client is required for any of these — they exercise the opaque epoch counter and the DAG resolution.

---

## 6. §-locks (Joe-LOCKED this session)

- **CC-D1** — M8.7 = **R only**; S folds into the L (production openmls-client) arc. Supersedes audit §6 "R+S".
- **CC-D2** — `mls.commit` state key = `(room, target_epoch)` (`category "state.mls_commit"`, `key_field "{room}:{epoch}"`).
- **CC-D3** — Layer-5c lexicographic-by-`event_id` tiebreak; **no** Layer-1 priority rule for commits.
- **CC-D4** — the **home-DS commit serialization** is an **L-side** live-delivery optimization (which winner online clients see first, minimizing how often loser-rebuild fires), **not** an R convergence requirement. The DAG `(room, target_epoch)` tiebreak alone gives deterministic convergence; the F-B "hybrid" lock splits cleanly along the R/L seam: **DAG-tiebreak = R, home-DS-serialize = L.**

---

## 7. Coverage ledger / honest boundary (D-065)

- **Loser rollback-and-replay is NOT exercised by R.** R proves every node *agrees on the winner* (no permanent fork). The loser client detecting its loss and rebuilding group state (re-deriving from the winner, re-applying its proposal → epoch N+2) is **L** (real openmls client). The in-process proof therefore demonstrates *convergence on the winner*, not the loser's recovery — the Arc H C1 Finding 1 analogue. Named here, not glossed.
- **S (real key schedule / HPKE) deferred to L** (CC-D1) — R resolves the opaque counter only.
- **Home-DS serialization deferred to L** (CC-D4).

---

## 8. Out of scope / untouched

- Real RFC 9420 / openmls crypto, group-state persistence, KeyPackage generation, Credential↔XGID, `ops::send` live-encrypt — all **L**.
- Validation of commit well-formedness (honest commits target current+1); a divergent `target_epoch` is a validation concern, orthogonal to R's convergence. Not changed here.
- `MlsGroupInit` state key (already present, per-Room) and the `mls_epoch=Some(0)` genesis — unchanged.
- Node DS blindness invariant — preserved.

---

Per D-065 (R-only scope correction; loser-rebuild honesty) + D-069 + D-071 + D-074. Next-active: runbook (`tasks/M8_7_CONCURRENT_COMMIT_IMPL.md`) → Clair. Clair stands down until the runbook exists. Not pushed — Joe pushes.
