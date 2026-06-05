# R2-F01 — Client/Node Convergence Alignment — Implementation Runbook
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

Implementation runbook for the **R2-F01 fix-arc**, built on the Joe-locked design
(`tasks/R2_F01_CLIENT_CONVERGENCE_DESIGN.md` v1.0, F01-D1…D6). **Awaiting Joe approval before
Clair picks up** (per the audit→design→lock→runbook→implement lifecycle). Reuses Arc C's proven
`derive_resolved` engine; the convergence math is done, the work is wiring + the AI-site gate.

The fix aligns the **client's local projection** with the node's resolved view by replacing the
three timestamp-sort + plain `apply_event` replay sites with the node's own resolution
discipline. The node stays authoritative throughout; no wire format and no node surface change.

---

## 2. Grounding (done at runbook authoring — the CPs are pre-resolved)

- **CP-4 (imports) — CONFIRMED CLEAN.** `xgen_core::resolution` is `pub mod`; `derive_resolved`
  and `state_key_for_event` are re-exported at `resolution/mod.rs:16-17`; `conflicts_in_log` is
  `pub` at `resolution::derive`; `topological_sort` is `pub` at `node::runtime`. `xgen-client`
  has the `xgen-core` path dep already — each site adds a `use`.
- **Site 1 — `ops.rs` `ai_status`** (`xgen-client/src/ops.rs`): finds the create root → seeds
  `SpaceState::from_space_create` → **bails on DM** (`"ai status against a DM Space is not
  supported in M3"`) → `sorted.sort_by(timestamp, root-first)` → `state.apply_event(ev, "")`.
- **Site 2 — `ops.rs` `members_projection`**: pure over `&[Event]`; finds the root → seeds
  `from_dm_space_create_node` (DM) or `from_space_create` (regular) → same timestamp-sort →
  `apply_event(ev, "")`. Handles both Space kinds.
- **Site 3 — `ai_service.rs` `run_ai_loop`**: a **live, long-running, incremental** per-Space
  `HashMap<String, SpaceState>`; each inbound Event seeds (create) or `apply_event(ev, "")`
  (else) in **receive order**; keeps only the derived state + `last_event_in_space`, **not the
  log**. DM creates are skipped today (no creator key).

---

## 3. Commit C1 — ops read paths (`xgen-client/src/ops.rs` only)

**One writer, one file.** Replace the manual find-root + timestamp-sort + `apply_event` blocks
with `derive_resolved`.

### 3.1 `members_projection` (clean drop-in)
- Add `use xgen_core::resolution::derive_resolved;` (or fully-qualify).
- Replace the find-root → seed → sort → apply block with:
  `derive_resolved(events.to_vec(), "", &std::collections::HashMap::new())`
  → map `None` to the existing `anyhow!("no state.space_create event observed for {space}")`.
- `derive_resolved` dispatches `from_dm_space_create_node` / `from_space_create` internally —
  byte-for-byte the current dual-seed — so DM coverage is preserved.
- Preserve `events_replayed = events.len()` and the deterministic `members.sort_by(identity_id)`.

### 3.2 `ai_status` (keep the DM guard)
- **Preserve the early DM bail** (`StateDmSpaceCreate` → `"ai status against a DM Space is not
  supported in M3"`). This is an operator-resolution scope limit, NOT a convergence concern;
  swapping to `derive_resolved` MUST NOT silently enable DM here (**CP-1a**).
- For the non-DM path, replace the seed + sort + apply loop with
  `derive_resolved(events.clone(), "", &empty)`; `None` → the existing no-create error.
- Vantage `""` is threaded by `derive_resolved` to `apply_event` internally — identical to today
  (F01-D3 / **CP-3**: confirm no `apply_federation_add` perturbation; expected no-op — the client
  is not a federation peer and the drained log is single-home).

### 3.3 C1 tests
- Existing `members_projection_*` tests must stay green unchanged (regression guard — the swap is
  behaviour-preserving for the no-conflict path; `derive_resolved` == plain replay when there are
  no conflicts, already proven in `derive.rs`).
- Add a **permutation-convergence test** (Arc-C mirror): a hand-built concurrent same-key
  conflict log, asserted to derive one identical membership under every arrival permutation via
  `members_projection`, and to match the node's `derive_resolved` winner for an L1/L4/L5c-decided
  conflict (ban>join is the simplest — needs no home-node map).

---

## 4. Commit C2 — AI inbound (`xgen-client/src/ai_service.rs` only)

**One writer, one file.** CP-2 is **resolved by grounding to the gate, not batch-then-derive** —
the loop is incremental + long-running with no clean flush point, so it mirrors `ingest_event`.

### 4.1 The gate
- Add a per-Space accumulated log: `let mut space_logs: HashMap<String, Vec<Event>> =
  HashMap::new();` alongside the existing `spaces` map.
- On each `Inbound::Event`, after the space_id resolution, **append the event to
  `space_logs[space_id]`**, then derive:
  - **create (regular `StateSpaceCreate`)** → seed as today (or `derive_resolved` over the
    one-event log); **DM create** → preserve the current skip (out of scope; **CP-2a**).
  - **non-create, state-keyed, `conflicts_in_log(&event, &space_logs[space_id])` true** → full
    `derive_resolved(space_logs[space_id].clone(), "", &empty)` rebuild → replace `spaces[space_id]`.
  - **else** → incremental `spaces[space_id].apply_event(&event, "")` (today's fast path).
    Message traffic short-circuits inside `conflicts_in_log` via `state_key_for_event`, so the
    common case pays no rebuild cost.
- `last_event_in_space` chaining, health refresh, and plugin dispatch are **unchanged** — only
  the SpaceState-derivation step changes.

### 4.2 Honest residue (D-065)
The resident now **retains the per-Space event log for its lifetime** (a new allocation, bounded
by Space activity). Record this in the C2 commit message + the close note — it is the cost of
giving the AI loop the node's gate.

### 4.3 Testability
The loop needs a live WS, so **factor the per-event derivation into a pure helper** —
`fn apply_or_rebuild(log: &[Event], state: SpaceState, ev: &Event) -> SpaceState` (or similar) —
and unit-test the helper without the network (mirrors how `members_projection` is pure/testable
while `members` needs the WS). Test: a concurrent conflict over the accumulated log converges to
the same snapshot a full `derive_resolved` of the log produces, and matches the node winner.

---

## 5. Close — doc-only (D-074)

1. **Run the F01-D5 reachability probe** — enumerate the conflict shapes the client can observe
   and assert whether any lands on Layers 3/5a/5b under the empty map. **Record the result.**
   - Negative → A-pure confirmed sufficient; close clean.
   - Positive → **flag** the F01-D2 escalation (A+thin-fetch) as a follow-up decision; do NOT
     auto-build it in this arc.
2. Flip **R2-F01 🟪→✅** in `tasks/ROUND_2_AUDIT.md` §5 (and a §6 verdict note); record the probe
   result + any escalation flag.
3. **ROADMAP** version bump + **JOURNAL** close entry + **CLAUDE.md PLAY** flip (same commit,
   D-074) + **F01-D# eval** (arc-local, D-069 — no DECISIONS change unless the probe escalates).
4. Task docs (audit / design / this runbook) → **COMPLETED**.

---

## 6. Confirm-at-pickup checklist (D-078) — residual

- **CP-1a** — `ai_status` keeps its DM bail; `members_projection` is the clean swap. Confirm the
  exact blocks against the live files at pickup (line numbers drift).
- **CP-2a** — AI-site DM creates stay skipped (parity with today); the gate covers regular Spaces.
- **CP-3** — `""` vantage causes no `apply_federation_add` perturbation at the client read sites.
- (CP-4 pre-confirmed in §2.)

---

## 7. Definition of Done

- C1: both `ops.rs` sites route through `derive_resolved`; `ai_status` DM-bail preserved;
  existing tests green + the permutation-convergence test added.
- C2: AI loop gated (`conflicts_in_log` → `derive_resolved` rebuild; else incremental); per-Space
  log retained; pure derivation helper unit-tested.
- Close: reachability probe run + recorded; R2-F01 ✅ in the Round-2 register; ROADMAP + JOURNAL +
  PLAY + F01-D# eval; task docs COMPLETED.
- `cargo test --workspace` green (baseline 1153/0/2 + C1/C2 additions); build all-targets 0;
  clippy clean (default **and** `--all-features`).

*(Per task-file convention, "commit pushed" is NOT a DoD item — the `Status: COMPLETED` header is
the shipped signal; Joe pushes.)*

---

## 8. Status & next-active

Runbook written; **awaiting Joe approval**. On approval → Clair implements C1 → C2 → doc-only
close, one writer per file per commit (C1 = `ops.rs`, C2 = `ai_service.rs`; no overlap).

Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-078 + the two-round audit principle.
