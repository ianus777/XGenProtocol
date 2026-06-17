# MP-F7 — leave→rejoin convergence: open rejoin anchors to root → dropped — FOLDED ARC-DOC (audit · design · runbook)

> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Discipline note

The **last** MP-R2 fix-phase gate item (gate {MP-F7}). Audit + design + thin runbook folded into one
doc (MP-F8 precedent). **Kind LOCKED = (b)** real leave→rejoin convergence fault; **Fork A LOCKED**
(client-side causal anchoring). The **D-076 spine** (§2.2) is the centerpiece — *proven, not assumed*:
this is exactly where MP-F4's A1 was falsified mid-arc (separating keys did NOT order the joins). **No
code until §3 is Joe-locked.** Terminal = **GREEN-on-rerun** (MP-C-11 gets *fixed*, not routed) → R2
rerun → MP-R2 close.

**Scope guard:** MP-F7 only. R2 close criterion stays **all-green-except-{MP-C-16, MP-A-01(ii)}** (both
R3-routed); MP-C-11 greens, is **not** a third carve-out.

---

## 1. Audit — kind PINNED by observation = (b)

**Symptom (RUN, `mp_r2_sweep` rung-0 / 2 clients):** `LogicFault "convergence needs ≥2 node
projections, got 1"`. The churning client a1 returns no membership view at oracle-sample time, so only a0
projects (oracle.rs:234 requires ≥2 *per-actor* projections; runner.rs:368-386 gathers them best-effort).

**Observed (bounded throwaway diagnostic — `members` replies + the node Space store; node exe untouched,
reverted after):**
- **a0** (stable): `members` = `{a0:owner}`, `events_replayed=10` — a0's **authoritative** view resolves
  a1 as **NOT a member**.
- **a1**: `members` = `None` — no view (downstream: a non-member's member-gated sync returns empty).
- **The node DAG (the smoking gun):** a1's events on key `membership:{space}:a1` —
  `join(j) prev=[space_create]` · `leave(lv) prev=[..a0-tip]` (causally after j) · **`rejoin(rj)
  prev=[space_create]`** — the rejoin is anchored to the **root**, making it **concurrent with the
  leave**. `derive_resolved`'s `frontier_of` sees `{lv, rj}` (size-2) → `resolve()` elects one (observed:
  **leave wins**) → a1 non-member.

**Root mechanism:** `ops::join` builds `prev_events` via `get_invite_bootstrap` → `get_dag_tips` →
fallback `vec![space_id]` (root) (ops.rs:1285-1306). A just-left rejoiner is a **non-member** → no
pending invite **and** the member-gated sync starves it of tips → `get_dag_tips` returns empty → the
**root fallback fires**. (The *leave* anchored to a real tip — a1 was still a member then; the asymmetry
is the pin.)

**The three candidates resolved:**
- **(a) a1 drops its client view** — **FALSIFIED.** `ops::leave` doesn't touch `state.spaces`; `members`
  re-derives from **node-drained** events. a1's `None` is because a1 is a non-member (member-gated sync),
  not a dropped cache.
- **(b) the open rejoin doesn't re-establish membership** — **CONFIRMED** (above). A real protocol/client
  convergence fault — any user leaving + rejoining an open Space stays non-member.
- **(c) the ≥2-actor oracle precondition is wrong** — **NOT the root.** The oracle correctly flagged
  *genuine* non-convergence (a0's authoritative view confirms). Fix (b) → a1 a stable member → projects →
  ≥2 satisfied → passes.

**Family:** the **MP-F4 / J-331** membership-concurrency family (a membership event dropped because bad
`prev_events` made it concurrent) + the J-241 open-join-concurrent edge + the MP-F5 member-gated-sync
bootstrap. **Engine sound, scenario-specific:** MP-C-05 (no churn) runs the identical sweep/oracle/derive
machinery green-to-8 — its actors never leave, so they never hit the rejoin-anchor path.

**Oracle-edge note (record, not a fix):** the ≥2-all-actors precondition is latently fragile for a future
churn row that *legitimately* ends an actor mid-leave (it would trip ≥2 despite correct convergence) —
known edge for future churn rows; **not this case** (a1 ends rejoined-intended, a genuine failure).

---

## 2. Design — Fork A LOCKED

### 2.1 The fix (Fork A — client-side causal anchoring)

The open rejoiner anchors its join **after its own leave**, not the root, so `rj` causally descends from
`lv` and is no longer concurrent on `membership:{space}:a1`.

- **MP-F7-D1 — wrinkle LOCKED = Option 2 (separate map, NOT a `KnownSpace` field):** add
  `ClientState.last_local_events: HashMap<String, String>` (space_id → event_id, `#[serde(default)]`,
  backward-compat). Rationale: the anchor is **causal-DAG bookkeeping, not membership state** — keeping it
  out of `KnownSpace` means **zero joined-list consumer sweep** (no `whoami`/`spaces`/joined-list
  pollution, no per-reader left-filter audit — the D-092-shape risk avoided), a **purely additive D-077
  surface**, and it models what the data is (a last local event holds regardless of membership).
- **MP-F7-D2** — `ops::leave` writes `ClientState.last_local_events[space] = leave_event_id` (+
  `write_client_state`). (`leave` writes no client state today — this adds the one write.)
- **MP-F7-D3** — `ops::join` fallback: when `get_invite_bootstrap` → None **and** `get_dag_tips` → empty,
  read `last_local_events[space]` — **present** ⇒ `prev=[it]` (rejoin, anchor after the leave);
  **absent** ⇒ `vec![space_id]` (first join, root — correct as today). The presence-check is **both** the
  rejoin-vs-first-join distinguisher **and** the safety fallback.
- **MP-F7-D4 (best-effort, not load-bearing)** — the anchor map *improves* the rejoin path when present;
  its **absence must degrade to today's behavior** (`get_dag_tips` → root), **never error**. A fresh
  client / cleared state / a leave that didn't persist must still first-join correctly. **Missing anchor ≠
  failure.**

**Fork B (node scoped-fetch for an open rejoiner)** — declined as primary; the **fallback only** if A
can't reconstruct the anchor client-side (it can — the client holds its own leave `event_id`). **Fork C
(resolution tiebreak: concurrent rejoin beats concurrent leave)** — **rejected outright** (would break
ban/kick dominance; resolution must not guess intent from concurrent membership events; the fix is the
causal **edge**, not the tiebreak).

### 2.2 The D-076 spine (PROVEN, not assumed — the centerpiece)

**Claim.** Anchoring `rj` with `prev=[lv]` makes `membership:{space}:a1` a **linear causal chain
`j → lv → rj`** (each on-key event's ancestry includes the prior on-key event). `derive_resolved` groups
by state-key then computes the causal **frontier** (`frontier_of`): the frontier is `{rj}` (lv, j are
ancestors of rj) → the resolved membership event for a1 is `rj` (a **join**) → **a1 converges to member,
under every arrival ordering** (the causal edges live in the signed `prev_events`, so `topological_sort`
+ `frontier_of` are order-independent). **No new concurrency is manufactured on the key:** rj's only
on-key relationship is to its ancestor lv; rj vs a0's message events are different state-keys (messages
carry no membership key), so cross-key concurrency is irrelevant to a1's membership resolution.

**RED-on-revert (genuine).** Revert the anchor to `prev=[space_id]` (root) → rj and lv share no on-key
ancestry → `frontier_of = {lv, rj}` (size-2) → `resolve()` elects the leave → **a1 non-member** (the
exact observed bug).

**Why this is the spine (the MP-F4 lesson).** MP-F4's A1 (room-scope the membership key) was **falsified
mid-arc** because separating the keys did **not** order the joins — the room-join still sorted concurrent
with the space-join. **Key-separation ≠ ordering.** Fork A creates the causal **edge** directly
(rj prev=[lv]); the spine **proves** the edge produces the ordering — exactly where MP-F4 went wrong.
Prove it (RED-on-revert genuine), do not assume it.

---

## 3. Runbook — thin (Joe-LOCKED 2026-06-11)

### C1 — client-side causal anchoring + the spine proof

**Change (xgen-common + xgen-client):**
1. `ClientState.last_local_events: HashMap<String, String>` (`#[serde(default)]`) — MP-F7-D1.
2. `ops::leave` — after `apply_single_event_confirm` Ok, `state.last_local_events.insert(space,
   leave_event_id)` + `write_client_state` (MP-F7-D2).
3. `ops::join` — the fallback arm (ops.rs:1294-1305): read `last_local_events.get(space)`; present ⇒
   `prev=[it]`, absent ⇒ root (MP-F7-D3/D4 — never error). (Optionally refresh on a successful rejoin so a
   leave→rejoin→leave→rejoin chain stays linear — confirm at pickup.)

**Named tests (D-078):**
- **Spine (xgen-core, the load-bearing test):** construct the `j → lv → rj` DAG with `rj prev=[lv]`; run
  `derive_resolved` across **permuted ingest orders**; assert a1 ∈ members every time. **RED-on-revert:**
  `rj prev=[space_id]` (root) → assert a1 ∉ members (frontier `{lv, rj}`, leave elected). This proves
  §2.2.
- **Client (xgen-client):** `leave` persists `last_local_event`; `join` fallback reads it and anchors
  `prev=[last_local_event]` when present, root when absent (first-join unchanged).

**DoD (C1):**
- `cargo build` 0-error (xgen-common, xgen-client, xgen-core); `cargo clippy --all-features -- -D
  warnings` clean.
- `cargo test` 0-failed; the spine test GREEN + **RED-on-revert** genuine; client unit tests GREEN.
- **Prime invariant:** first-join path byte-unchanged (no `last_local_event` ⇒ root, as today); existing
  join/leave/membership tests green; `KnownSpace` serde backward-compat (old state loads).

### The witness (box-gated R2 rerun)

**MP-C-11 rung-0 GREEN-on-rerun:** a1 ends a stable member (rj anchored after lv → converges) → a1's sync
works → a1 projects → ≥2 satisfied → the rung converges GREEN. RED-on-revert genuine (revert the anchor →
rung-0 LogicFault returns). Rebuild as the sweep needs (single-node real-clock; a harness-control node
works too); coordinate on the box.

---

## 4. Scope / honest boundary / confirm-at-pickup

- **No code until §3 is Joe-locked.** Production-crate fix (xgen-client + xgen-common) → arc discipline.
- **Fork A is locked**; Fork B is fallback-only; Fork C rejected. Do not touch the resolution tiebreak.
- **The spine is proven, not assumed** (§2.2) — RED-on-revert genuine, the MP-F4-falsification guard.
- **Does NOT** touch the node member-gated-sync boundary (that's Fork B / a deeper arc), the resolution
  layers, or any wire shape. Does **not** re-open the gate (MP-F7 is the last item; greening it runs the
  R2 rerun → MP-R2 close).
- **MP-C-11 is fixed, not routed** — GREEN-on-rerun terminal.

**Confirm-at-pickup (D-078) — resolve against live `main` at C1 start:**
- Whether to refresh `last_local_events[space]` on a successful join (for repeated leave→rejoin cycles).
- The exact `derive_resolved` / `frontier_of` / membership-`state_key` entry points for the spine test
  (mirror the MP-F4 + SR-D# convergence tests).

---

*Per D-065 (surface the wrinkle, prove the spine) + D-069 (arc-local MP-F7-D#) + D-071 (runbook follows
the locked design) + D-074 (per-commit atomicity) + D-076 (the spine: causal-DAG-respecting order, proven
across orderings) + D-077 (prime invariant: first-join unchanged) + D-078 (confirm-at-pickup) + the J-344
BOUNDED-gate criterion (GREEN-on-rerun terminal, last gate item).*
