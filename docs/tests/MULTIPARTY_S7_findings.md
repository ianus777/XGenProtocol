# Multiparty Test S7 — Findings (M8 / Wave 3 / C6)
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

The **C6** result of M8 Wave 3 (runbook `tasks/M8_MULTIPARTY_IMPL.md` §5 C6; design §3 S7
row). Privilege enforcement (Arc D), **multiparty behaviour only — NOT the auth-tier matrix
(M8-A7)**: a synthetic Tier-1/Local-Node setup; no Auth Module ref set. First baseline (no
historical "A"; M8-D3). B stamp: `8b14aa8` (≡ `676b9c1`).

---

## Proofs (new, `m8_s7_privilege.rs`)

| Test | What it proves | Verdict |
|---|---|---|
| `s7_tier_gated_join_refused_on_all_nodes` | A Tier-1 (invited) joiner is **refused** entry to a Tier-2 Space on **every** Node — the dispatch step-4 gate returns `Rejected` carrying wire **3030 `tier_mismatch`** — and the joiner is **absent** from the resolved membership on every Node (the rejected join never enters the DAG). | **PASS** |
| `s7_per_room_override_blocks_moderator_send_and_converges` | A `(Moderator, SendMessages, Deny)` override on `#announcements` **blocks** a Moderator's message (`check_permission` → `PermissionDenied`) on **every** Node, and the override **converges** — identical `permission_overrides` on every Node (it rides state-keyed `state.room_update`, the C2 Layer-4 convergence). The "Mods can't post in #announcements" case, multiparty-observed. | **PASS** |

```
$ cargo test -p xgen-node --lib m8_s7
running 2 tests
test ...s7_tier_gated_join_refused_on_all_nodes ... ok
test ...s7_per_room_override_blocks_moderator_send_and_converges ... ok
test result: ok. 2 passed; 0 failed; ...
```
Full workspace after Wave 3: **1167 passed / 0 failed / 2 ignored**; clippy clean both feature
sets.

---

## "Observed by all members" — the multiparty angle (M2/G-ALIGN)

Both decisions are agreed by every member's Node:
- **Tier-gate refusal:** the rejected join is a dispatch-time reject (never stored), so there
  is **no event to disagree on** — the joiner is uniformly absent. Proven on two Nodes.
- **Per-Room override:** rides state-keyed `state.room_update`, so every Node resolves the
  **identical** override (the C2 Layer-4 convergence applies) and makes the same accept/reject
  decision. Asserted: identical `permission_overrides` on both Nodes + identical block.

**Honest scope (M8-A7):** S7 tests the *multiparty behaviour* of the tier-gate and the
per-Room override — NOT the auth-tier matrix. The gate is an honest Tier-1 no-op at Tier 1
(`verify(1,1)=Ok`); it bites here because the synthetic Space declares `auth_tier=2`. No Auth
Module reference set is required or built (that is M10's battery).

---

## The four metrics (M8-D2)

- **M1 — Delivery.** N/A as a fan-out metric — S7 is about enforcement decisions, not delivery.
- **M2 — Convergence.** **CONVERGED** — the per-Room override resolves identically on every
  Node; the tier-gate refusal is uniform (no DAG entry).
- **M3 — Integrity.** Zero — the rejected join produces no member/event anywhere; no
  `ERROR`/unexpected `WARN`.
- **M4 — Latency (informational; throughput NOT measured).** In-process; no network latency.

---

## CP-4 placement

Both enforcement decisions are deterministic (the gate + `check_permission` are pure given
resolved state) — real processes add no signal (M8-D6) — so S7 is workspace-homed. A
binary-level operator-realistic S7 would exercise the same gates over real WS and is scoped to
the suite-execution pass (no new correctness signal). *(Note: the binary invitee-join bug
recorded in `MULTIPARTY_S8_findings.md` would currently block a binary tier-gate join setup —
an additional reason the workspace home is correct.)*

---

## Definition of Done — C6

- [x] **Tier-gated join refusal** multiparty-visible (3030 on every Node; joiner absent
  everywhere).
- [x] **Per-Room override** enforced + converged (Deny blocks the Moderator's send on every
  Node; identical `permission_overrides`).
- [x] Synthetic Tier-1/Local-Node; no Auth Module ref set (M8-A7 honored).
- [x] M1–M4 recorded; CP-4 placement noted.
- [x] `cargo test --workspace` 1167/0/2; clippy clean both feature sets.

---

*End of MULTIPARTY_S7_findings.md — C6 complete.*
