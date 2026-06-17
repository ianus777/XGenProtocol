# M8.7 — Concurrent-Commit Resolution: Implementation Runbook
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose

Executes the J-300/J-301 Joe-LOCKED design (`tasks/M8_7_CONCURRENT_COMMIT_DESIGN.md` v1.1). Makes `mls.commit` a conflict domain so concurrent commits at one frontier resolve to a single deterministic winner that every federated node agrees on. **R only** (CC-D1); S + home-DS serialization are L. Crypto-agnostic, no openmls dependency.

**Clair: read first (Rule 0):** CLAUDE PLAY → JOURNAL J-301 → design §3–§7 → this runbook §2–§4.

---

## 2. Grounded change surface

- `xgen-core/src/resolution/state_key.rs` — the `MlsGroupInit` arm sits at ~lines 100-108 (keyed per-Room); the catch-all `_ => None` with the "mls.\*" comment is just below. The new `MlsCommit` arm goes adjacent to `MlsGroupInit`.
- `xgen-core/src/space/state.rs` — `RoomState.mls_epoch: Option<u64>` at line 134 (struct `RoomState`, `#[derive(Debug, Clone, PartialEq, Eq)]` at 112 — **not** `Serialize`). `apply_mls_commit` at 825 (doc comment 810-824). RoomState constructors initialise fields at ~397 and ~788; `state.mls_group_init` sets `mls_epoch = Some(0)` at ~805.
- Test helpers (reuse, do not duplicate): `build_mls_group_init_event(key, space_id, room_id, room_create_event_id)` and `build_mls_commit_event(key, space_id, room_id, prev_events: Vec<String>, epoch)` in `state.rs` (1457, 1486); both emit `content {epoch, nonce}` — the `nonce` gives two same-epoch commits distinct `event_id`s. Existing single-Node `mls_commit_ingest_advances_room_epoch` in the runtime.rs Arc H C2 test module (1918) is the pattern for the two-Node repro.

---

## 3. Single commit — ordered steps

One commit (the core change is small; no family boundary). Order:

**Step 1 — `RoomState.mls_commit_tip` field (CC-D5).**
- Add `pub mls_commit_tip: Option<EventXgid>` to `RoomState` immediately after `mls_epoch` (line 134), with a doc comment mirroring `mls_epoch`'s: Node-readable canonical commit *identity* for the current epoch (no key material); `None` until the first `mls.commit`; order-independent scalar that rides `RoomState`'s `PartialEq`/`Eq` M8 convergence oracle additively.
- Initialise `mls_commit_tip: None` at every `RoomState { … }` construction site (the ~397 and ~788 constructors; `state.mls_group_init` leaves it `None` — genesis has no commit). Compiler will flag any missed site.
- Confirm `EventXgid` is in scope in `state.rs` (it is — `RoomXgid`/`SpaceXgid`/`EventXgid` already imported for the build helpers); no new import expected.

**Step 2 — `apply_mls_commit` records the winner (CC-D5) + comment.**
- In `apply_mls_commit` (825), alongside `room.mls_epoch = Some(epoch)`, add `room.mls_commit_tip = event.event_id.clone();` (inside the same `if let Some(room)` block; after the `epoch` is read). No other logic change — loser-exclusion is automatic via the resolved set.
- Correct the doc comment (810-824): the concurrent commit-race is now resolved by the `(room, target_epoch)` conflict domain; drop the "fenced to D3 / this applier does not resolve it" wording.

**Step 3 — `MlsCommit` state-key arm (CC-D2) + comment corrections.**
- Add to `state_key_for_event` (adjacent to the `MlsGroupInit` arm):
  ```
  EventType::MlsCommit => {
      let target_epoch = event.content["epoch"].as_u64()?;
      Some(StateKey::new(
          "state.mls_commit",
          format!("{}:{}", event.room_id.as_str(), target_epoch),
      ))
  }
  ```
- Correct the stale sentence in the `MlsGroupInit` comment ("Epoch *advances* … introduce no new state key of their own") → point to the new `MlsCommit` arm (M8.7 CC-D2).
- Correct the `_ => None` comment: `mls.commit` now has a state key; only `mls.welcome` / `mls.proposal` remain keyless among `mls.*`.

**Step 4 — tests (§4).** Add in the same commit.

**Step 5 — checkpoint (§5), then verify (§7) and flip `Status: COMPLETED`.**

---

## 4. Tests

- **`state_key.rs` units** (in the existing tests mod, beside `mls_group_init_keyed_per_room`):
  - same `target_epoch` on one Room → equal keys; different `target_epoch` → unequal; different Room → unequal.
  - **regression guard:** a `2 → 3` commit and a `1 → 2` commit on the same Room do **not** share a key (proves CC-D2's per-epoch keying prevents epoch regression).
  - absent `epoch` → `None`.
- **resolution unit** (runtime.rs Arc H module or a `derive_resolved` test): a frontier of two same-`target_epoch` commits (distinct nonces → distinct ids) resolves so exactly one is applied; the applied tip equals the lexicographically-winning `event_id`.
- **headline two-`NodeRuntime` convergence repro** (runtime.rs Arc H module): build a Space + Room + `mls.group_init`; build two `mls.commit` events both `prev=[group_init id]`, both `epoch=1`, distinct nonces (A, B). Ingest into `rt_x` in order [A, B] and into `rt_y` in order [B, A]. Assert both rooms' `(mls_epoch, mls_commit_tip)` are **equal across the two NodeRuntimes** (the `RoomState` `Eq` oracle), and equal to the lexicographic winner.
- **sensitivity witness (checkpoint #-close, recorded at close):** with the Step-3 `MlsCommit` arm reverted, the repro's two NodeRuntimes diverge on `mls_commit_tip` (**RED**); restore (**GREEN**). Records why CC-D5 + the arm are load-bearing (a counter-only design would have stayed green).

---

## 5. Checkpoint (one, light)

Before close, Clair surfaces:
1. The final `apply_mls_commit` shape (tip write placement) + the `RoomState` field + all constructor sites touched.
2. **The one design-flagged verification:** confirm on the live node path that `apply_mls_commit` is invoked **only for the resolved winner** (not both concurrent commits) — i.e. the tip reflects the winner, not the last-folded event. If the live path applies pre-resolution, that is a finding (surface it, do not work around silently — D-065/D-084).
3. Suite green. No value-locks at the checkpoint.

---

## 6. Definition of Done

- [ ] `RoomState.mls_commit_tip` field added + all constructor sites initialise it.
- [ ] `apply_mls_commit` writes the tip; D3-fencing comment corrected.
- [ ] `MlsCommit` state-key arm added; `MlsGroupInit` + `_ => None` comments corrected.
- [ ] state_key units (incl. regression guard) + resolution unit + two-Node convergence repro added and green.
- [ ] Sensitivity witness demonstrated (arm reverted → RED; restored → GREEN) and recorded at close.
- [ ] `cargo build --workspace --all-targets` 0; `cargo clippy --workspace --lib --tests -- -D warnings` clean on default **and** `--all-features`.
- [ ] `cargo test --workspace` green (baseline 1207 + new tests).
- [ ] Canonical records updated same-commit (D-074): JOURNAL, CLAUDE PLAY, ROADMAP, this runbook → COMPLETED, design → COMPLETED, audit → COMPLETED.

*(No "commit pushed" line — the `Status: COMPLETED` header is the shipped signal; Joe pushes.)*

---

## 7. Verify / scope discipline

- Untouched: `MlsGroupInit` key, genesis `mls_epoch=Some(0)`, `mls.welcome`/`mls.proposal`, commit well-formedness validation, all crypto. No DECISIONS change (arc-local CC-D#, D-069).
- Honest boundary recorded at close: loser rollback-and-replay is L (the proof shows convergence-on-winner only — Arc H C1 Finding 1 analogue).

---

Per D-065 + D-069 + D-074. Next-active: Clair — Step 1 → … → checkpoint → close. Clair stands down until Joe approves this runbook. Not pushed — Joe pushes.
