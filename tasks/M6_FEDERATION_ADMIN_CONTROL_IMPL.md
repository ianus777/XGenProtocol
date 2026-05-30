# Federation-Admin-Control 2a — Approval & Queue — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Framing

Implementation runbook for **federation-admin-control sub-arc 2a (approval & queue)**.
Design locked at `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md` v1.0 (FAC-D1/D1a/D2,
J-172); audit at `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md`; verb spec at
`docs/xgen_node_admin_ops_design.md` §6.A1.

Reading order at pickup (Rule 0): CLAUDE.md PLAY → latest JOURNAL → this runbook
§1–§2 → the 2a design doc → §3+ per commit.

Scope is **2a only**: `federation accept` / `reject` / `initiate` + the pending-request
queue + the `FederationState` model. The policy verbs (`set-policy`/`show-policy`) +
store + enforcement are **2b** (`tasks/M6_FEDERATION_POLICY_DESIGN.md`, PENDING) — do
NOT build them here.

**Default-off invariant (FAC-D1).** With `require_approval = false` (the default),
every code path below must behave **byte-for-byte as today** — auto-establish on valid
handshake. The queue, the pause-point, and the state transitions are reachable only
when the operator sets `require_approval = true`. The shipped `list`/`defederate`
subset is untouched. This is the single most important property to preserve.

**Code-trace grounding (J-173).**
- `FederationRelationship` (`xgen-core/src/federation/registry.rs:39-56`) already uses
  the `#[serde(default)]` + `skip_serializing_if` pattern (see `peer_url`), and the
  registry already has a backward-compat precedent test
  (`load_old_format_without_peer_records_field_works`). So FAC-D2's new `state` field
  with `#[serde(default)]` → old records load as the default variant. **`Active` is the
  default** → existing implicitly-active records stay Active. Migration is free.
- Federation reject codes: **2001** (no_common_capabilities) + **2002**
  (version_incompatible) are taken in `handshake.rs`; **2003 is free** → the new
  "approval pending, retry later" code (checkpoint #2).
- The ACTIVE transition in `run_receiving` (`handshake.rs`, after `recv accept`) is the
  pause-point site; `run_initiating` is operator-outbound (no gate, FAC-D1a asymmetry).

## §2 Sequence overview

| Commit | Scope | Crate(s) | Class |
|---|---|---|---|
| **1 — State model + migration** | `FederationState` enum (`Active` default) as `#[serde(default)]` field on `FederationRelationship`; `mark_pending`/`mark_rejected` registry methods; backward-compat load test | xgen-core | foundational |
| **2 — Queue + config flag** | `require_approval` on `NodeConfig` (default false); pending-request queue store (persisted JSON, D-035) + add/remove/get/list | xgen-core + xgen-node | net-new |
| **3 — Pause-point (FAC-D1a)** | wire `run_receiving`: on `require_approval && peer-not-Active` → enqueue + `Reject` code 2003; default-off path untouched | xgen-core + xgen-node | LOAD-BEARING |
| **4 — Verbs** | `federation accept` / `reject` / `initiate` in `admin_ops` + clap + pipe arms | xgen-node | mechanical |
| **5 — Close** | §6.A1 marks the 3 verbs SHIPPED; backing-audit row; runbook COMPLETED; PLAY → 2b | docs | close |

**Joe-lock checkpoints:**
- **#1 (pre-Commit-1):** the `FederationState` variant set (`Active`/`Pending`/`Rejected`/`Revoked`) + **`Active`-as-serde-default** confirmed against the existing persisted shape.
- **#2 (pre-Commit-3):** the new reject code **2003** + the exact pause condition (**"peer not already `Active` in the registry"** — approval state only, NOT policy; policy is 2b).
- **#3 (pre-Commit-4):** `reject` tombstone retention/expiry + `initiate`'s no-self-approval asymmetry (operator-outbound establishes as today even when `require_approval = true`).

**Verification rigour (every commit):** `cargo test --workspace` green, `cargo clippy --workspace --lib --tests --all-features -- -D warnings` clean, `cargo build --workspace --all-targets` 0 errors. Commits 1 + 3 (the migration + the load-bearing gate) also get isolated re-runs of their new tests, and an **explicit default-off regression test** (a full handshake with `require_approval = false` still auto-establishes).

**Discipline:** no "commit pushed" in any DoD (D-074); `Status: COMPLETED` is the signal. ROADMAP + CLAUDE update in the same commit as any state change.

## §3 Commit 1 — State model + migration (FAC-D2)

`FederationState` enum in `registry.rs` (`#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`, `#[serde(rename_all = "lowercase")]`): `Active` / `Pending` / `Rejected` / `Revoked`. `impl Default` → `Active`.

Add to `FederationRelationship`:
```
#[serde(default)]
pub state: FederationState,
```
`#[serde(default)]` makes old JSON records (no `state` key) deserialize as `Active` —
the implicitly-active records the audit found stay active. Do NOT add
`skip_serializing_if` (we always want `state` written going forward).

Registry methods (beside `mark_active` at `registry.rs:185`):
- `mark_pending(&mut self, peer_node_id, now)` — sets `state = Pending` (creating the record if absent, mirroring `mark_active`'s create-if-absent shape).
- `mark_rejected(&mut self, peer_node_id, now)` — sets `state = Rejected` (tombstone).
- `mark_active` — additionally sets `state = Active` (it already creates/updates the operational record; extend it to set the state field too).

**Tests:** `federation_state_defaults_active`; `load_old_relationship_without_state_field_is_active` (the FAC-D2 backward-compat proof, sibling to `load_old_format_without_peer_records_field_works`); `mark_pending_then_active_transitions`; `mark_rejected_sets_tombstone`; save/load round-trip carries `state`.

## §4 Commit 2 — Queue + config flag

**Config.** `require_approval: bool` on the federation section of `NodeConfig`
(`#[serde(default)]` → false). Default-off = today's behaviour. (Confirm the exact
config struct + section at pickup — sibling to existing federation config fields.)

**Queue store.** New `PendingFederationRequest { peer_node_id, peer_url: Option<String>, received_at: String (RFC 3339), shared_spaces, negotiated_version, negotiated_serialisation }` (the handshake-derived facts needed to complete the relationship on `accept`). A `PendingFederationQueue` store, persisted JSON at the D-035-convention path (sibling to the registry file; confirm resolver at pickup), with `add` / `remove(peer)` / `get(peer)` / `all` / `save` / `load`. Keep it a sibling store, not a field on the registry — the queue is pre-relationship.

**Tests:** queue add/get/remove/all; save/load round-trip; `require_approval` defaults false; old config without the field loads false.

## §5 Commit 3 — Pause-point (FAC-D1a) — LOAD-BEARING

Wire the gate into the receiving handshake. The gate fires **only** when
`require_approval == true` AND the peer is **not already `Active`** in the registry
(an already-Active peer reconnecting is not a new approval — it proceeds). On gate:
record a `PendingFederationRequest` in the queue, send `Reject` with **code 2003**
(`error_string` e.g. `"approval_pending"`), and return a new
`HandshakeError::ApprovalPending` (do NOT transition to ACTIVE). The peer disconnects;
after the operator `accept`s, the peer's normal reconnect / an operator `initiate`
establishes the now-approved session.

**Threading.** `run_receiving` needs `require_approval` + a registry read + a queue
handle. Pin the exact signature shape at pickup — prefer the established pattern (the
caller in `xgen-node` that drives `run_receiving` already holds the registry; pass the
flag + a queue handle the same way, or gate in the node-side caller around the
handshake rather than deep inside the core fn if that keeps `xgen-core` free of the
queue type). **Decision for pickup:** does the gate live inside `run_receiving`
(xgen-core gains a queue-callback) or in the `xgen-node` caller wrapping it? Lean
**node-side wrap** — keeps the approval/queue policy in the node layer and `xgen-core`'s
handshake a pure protocol primitive. Confirm against the actual call site.

**Default-off regression test is mandatory here:** `require_approval = false` → a full
receiving handshake still reaches ACTIVE + persists, unchanged.

**Tests:** gate enqueues + rejects 2003 when on & peer absent; already-Active peer
bypasses the gate when on; **default-off auto-establishes (regression)**; pending peer
not double-enqueued on retry.

## §6 Commit 4 — Verbs

`admin_ops::federation_accept` / `_reject` / `_initiate` + clap `FederationCommand`
variants + pipe arms (mirror the shipped `list`/`defederate`).
- **`accept <peer>`** — peer must be in the queue (else error). Removes from queue,
  `mark_active` (Pending→Active + upsert + schedule reconnect via the existing
  scheduler path), returns the established relationship summary. WRITE → A6 trail.
- **`reject <peer>`** — peer must be in the queue. Removes from queue, `mark_rejected`
  (tombstone). DESTRUCTIVE → A6 trail. Tombstone retention per checkpoint #3.
- **`initiate <peer-url>`** — operator-outbound: calls `run_initiating` (today's path).
  **No self-approval even when `require_approval = true`** (FAC-D1a asymmetry — the
  operator initiating *is* the approval). WRITE → A6 trail.

**Errors:** peer-not-in-queue (accept/reject); bad peer-url (initiate); reuse the A1
error family. **Tests:** accept completes + clears queue; reject tombstones + clears
queue; accept/reject unknown peer → error; initiate establishes regardless of flag.

## §7 Commit 5 — Close

§6.A1 marks `accept`/`reject`/`initiate` SHIPPED; `tasks/M6_BACKING_AUDIT.md` A1 row
updated (3 of the 5 deferred verbs ship; `set-policy`/`show-policy` remain → 2b).
Runbook ACTIVE → COMPLETED. CLAUDE PLAY → **2b (federation policy)**. ROADMAP arc row
2a 🟢 → ✅, 2b 🟡 → 🟢-next. JOURNAL close entry. Full verification + isolated re-runs
of Commit 1 + Commit 3 tests + the default-off regression.

## §8 Discipline notes

- **Default-off is the prime invariant** — every commit keeps `require_approval = false`
  behaving exactly as today; the explicit regression test guards it (D-065 honesty: we
  don't quietly change federation's default posture).
- **2a ≠ 2b** — no policy types/store/enforcement here. If a verb seems to want policy,
  it's out of scope; stop and flag.
- **Backward-compat load (FAC-D2)** — the `#[serde(default)] state = Active` migration
  must be proven by a test loading a pre-state-field record (D-067 no-drift: existing
  federated Nodes don't break on upgrade).
- **`initiate` asymmetry** — operator-outbound is not gated; the gate is inbound-only.
- **D-078 grounding** — verify config struct shape, the `run_receiving` call site, and
  the D-035 store path against live code at pickup; report mismatches at the relevant
  checkpoint rather than guessing.

## §9 Cross-refs

- Design: `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md` v1.0 (2a). Audit:
  `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md`. Sub-arc 2b:
  `tasks/M6_FEDERATION_POLICY_DESIGN.md` (PENDING).
- Spec/verb: `docs/xgen_node_admin_ops_design.md` §6.A1 + Appendix K.2.4.
- Code: `xgen-core/src/federation/registry.rs` (`FederationRelationship` :39-56,
  `mark_active` :185, save/load :299-308, the backward-compat test precedent),
  `xgen-core/src/federation/handshake.rs` (`run_receiving` ACTIVE transition; reject
  codes 2001/2002, 2003 free), `xgen-node/src/reconnect.rs` (scheduler — `accept`'s
  reconnect path), `xgen-node/src/admin_ops.rs` (A1 `list`/`defederate` to mirror).
- D-071 / D-069 / D-065 / D-067 / D-074 / D-078.

---

*Implementation runbook. Clair's sequence: Commit 1 (state+migration) → 2 (queue+flag) → 3 (pause-point) → 4 (verbs) → 5 (close). Default-off byte-for-byte today is the prime invariant.*
