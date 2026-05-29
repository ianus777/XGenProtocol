# M6 Phase 5 — A5 Identity registry administration
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Ship the four A5 verbs — Node-local administration of the Identity records this
Node stores (D-082: scoped to what this Node hosts, never an Identity's standing
elsewhere). Authoritative spec: `docs/xgen_node_admin_ops_design.md` §6.A5 +
Appendix K.2.2. Follows the Phase-4 `admin_ops` verb pattern (Args/Result +
`AdminError`/`Stage` + `record_action` audit-the-auditor + clap `AdminCommand`
variant + pipe dispatch arm).

## Verbs (4 — locked Block 4; `identity list` already ships from M2)

| Verb | Class | Audited | Notes |
|---|---|---|---|
| `identity show` | READ | no (A5-D3) | display one stored record (live registry read) |
| `identity revoke` | DESTRUCTIVE | **yes** | block-only (A5-D1); mark revoked, persist, report inert Spaces |
| `identity set-trust-expiry` | WRITE | **yes** | set/replace the `expiry` inside the Trust Assertion |
| `identity manage-replica` | WRITE (add/remove) | **yes** (add/remove); no (list) | thin-scope (A5-D2): declare/list replica-holding Nodes; no active push |

Propagation interaction = **none** for all A5 verbs (no protocol event; cascade
deferred per A5-D1).

## P5 decision — AdminContext widens to runtime-aware (Joe-locked 2026-05-29)

**The load-bearing decision of this phase.** A5's mutating verbs must reach
*live* Node state, not just disk:

- **A5-D1 commits `identity revoke` to "immediate, security-critical."** A
  disk-only write to `xgen-node_identities.db` would leave the running
  resident's in-memory `identity_registry` stale (its auth check reads memory),
  and the next registration-save would clobber the file — a revoked Identity
  could keep authenticating until restart. That is a security window, not
  cosmetic lag; disk-only does not implement A5-D1.
- **`ReplicaRegistry` is in-memory only** (`replication.rs` — "rebuilt on
  restart"). `manage-replica` has *no disk backing* — it can only operate on the
  live runtime.

So this category reaches into the resident, exactly as A6-D1's `log set-level`
reaches the live `tracing-subscriber` reload handle. `AdminContext` gains an
optional `runtime: Option<Arc<Mutex<NodeRuntime>>>` (and an `identities_path()`
helper); the pipe server already holds that `Arc` (used for the health line) and
now threads it through `dispatch_line` → `dispatch_admin`. `batch_with_runtime`
is the runtime-aware constructor; `batch` (runtime `None`) stays for the
file-only A6 verbs and for unit tests.

**Recorded as a decision, not an implicit one** (per Joe's note): this sets the
precedent that `AdminContext` carries a runtime handle for *all* live-mutating
verb categories (A1/A2/A4 later phases inherit it). M7's `--aicontrol`
dispatcher provides the handle the same way the pipe does. Not a D-NNN — it is
an implementation realisation within design §6.5 latitude, recorded here.

## Commit sequence

| # | Scope | Status |
|---|---|---|
| 1 | xgen-core registry: `revoked`/`revoked_at`/`revocation_reason` on `IdentityRecord` (serde-default, backward-compatible); `revoke`/`is_revoked`/`set_trust_expiry` methods + `RegistryError::AlreadyRevoked`; ~22 fixture literal sites swept; 8 unit tests | ✅ (core lib 457→465) |
| 2 | `AdminContext` runtime handle + `batch_with_runtime`/`identities_path`/`require_runtime`; `identity_show`/`revoke`/`set_trust_expiry`/`manage_replica` verbs (Args/Result, `IDENT_*` codes, audit-the-auditor); clap `IdentityCommand`/`ReplicaAction`; 4 verb tests | ✅ (folded — see below) |
| 3 | pipe `dispatch_line`/`dispatch_admin` thread the runtime + 4 identity arms; revoke auth gate in `handle_connection` (immediate deny via live `is_revoked`); 1 dispatch-routing test | ✅ (node lib 117→122) |
| 4 | Phase close: this file → COMPLETED + Ch3 §3.6.6 revocation-field doc-sync + JOURNAL J-155 + CLAUDE PLAY flip (→ Phase 6) + ROADMAP | ✅ |

Per Joe's prior "fold it" cadence for M6, the phase lands as one folded commit
(Commits 1–3 above are logical groupings, verified together). Joe pushes.

## Error-code bands (A5, §2.7 / Appendix K.5)

`IDENT_6001` identity not found · `IDENT_6002` already revoked · `IDENT_6010`
malformed/invalid expiry · `IDENT_6020` invalid/missing node_id (add/remove) ·
`IDENT_6021` replica already present / not present · `GENERIC_4000` bad args /
no live runtime.

## Definition of Done

- [x] `identity show` returns the stored record; `IDENT_6001` on unknown; not audited.
- [x] `identity revoke` marks revoked in the **live** registry, persists to disk, reports `stale_membership_spaces`; `IDENT_6001`/`IDENT_6002`; DESTRUCTIVE → audited.
- [x] Revoked Identity denied session-open immediately (auth gate reads live `is_revoked`).
- [x] `identity set-trust-expiry` validates RFC-3339 (`IDENT_6010`), sets/replaces expiry, reports `previous_expiry`; WRITE → audited.
- [x] `identity manage-replica` add/remove/list against the live `ReplicaRegistry`; `IDENT_6001/6020/6021`; add/remove audited, list not.
- [x] `IdentityRecord` revocation fields are serde-default (active records byte-identical to pre-M6; pre-A5 JSON deserialises).
- [x] clap `identity` grouping routes all four verbs via `dispatch_line`; M2 read-only allowlist (incl. `identity list`) unchanged.
- [x] `cargo test --workspace` green (685 lib + 25 integration, 0 failed); clippy `-D warnings` clean; build all-targets 0 errors.

## Verification (close)

- `cargo test --workspace`: **685 lib** (63 client + 35 common + 465 core + 122 node) + 25 integration; 0 failed. +13 lib vs the Phase-4 672: +8 core registry (Commit 1), +5 node (4 A5 verb tests + 1 dispatch-routing test).
- `cargo clippy --workspace --lib --tests --all-features -- -D warnings`: clean.
- `cargo build --workspace --all-targets`: 0 errors.

## Scope honesty (D-065)

- The `--batch` pipe reply stays OK/ERROR (M2-frozen); `dispatch_admin` prints a
  human summary (or the record JSON for `show`) to resident stdout and returns
  OK. Rich structured verb output is M7's `--aicontrol` job.
- The revoke auth gate is a 4-line check over the unit-tested
  `IdentityRegistry::is_revoked`; it is covered by that unit test + the
  registry round-trip tests rather than a full two-process register→revoke→
  reconnect TCP scenario (disproportionate scaffolding for a straight-line gate
  — same call made in Phase 2 for the accept/reject signal helpers).
- `manage-replica` is A5-D2 thin-scope: it records the replica relationship in
  the in-memory `ReplicaRegistry` (not persisted — rebuilt on restart by the
  replication subsystem); active replication push is out of M6.
- No DECISIONS.md change. The category locks live in the canonical M6 doc
  (D-069); the AdminContext-widening is recorded above as a P5 implementation
  lock, not a D-NNN.

## Next

Phase 6 — A3 Bootstrap configuration (Appendix K.2.3, 5 verbs). **Phase 9 stays
design-gated** (the A4 `membership.node_eject` wire sub-design precedes it).

---

*End of Phase 5 plan.*
