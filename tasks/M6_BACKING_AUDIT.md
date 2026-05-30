# M6 Backing-Map Audit — Verb-to-Subsystem Reality Check
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

M6 Block 4 designed 33 admin verbs across 7 categories from the **spec's** view of each subsystem. Implementation (Phases 5–7) revealed that several verbs were designed against subsystems that **do not yet exist in code** — the verb is specified, but its backing store / state machine / network path is a placeholder or absent. This surfaced reactively, one category at a time: A3 (bootstrap-client placeholder), then A1 (no federation approval queue / policy store).

This audit is the **D-071 discipline applied to M6's own write path** ("subsystem audits precede dependent milestones"): a single read-only pass mapping every M6 write verb to its real backing, so the remaining deferrals are **deliberate** rather than rediscovered category-by-category. Verbs were checked against the live `xgen-core` tree (not worktrees) on 2026-05-29.

This is an audit artifact, not a design doc — it records what *is*, and routes gaps to named follow-up arcs. The canonical M6 design remains `xgen_node_admin_ops_design.md`; §5.1 / §6 there are amended **from** this map.

## Legend

- **BACKED** — the subsystem the verb operates on exists and is wired; verb is implementable now.
- **ABSENT** — the backing store / state machine / network path does not exist; verb presupposes unbuilt infrastructure.
- **SPLIT** — some verbs in the category are backed, others absent.
- **SELF-BUILDING** — a foundation phase that *creates* its own backing (not a consumer of pre-existing state).

## The map

### A1 — Federation management (Phase 7) — SPLIT
Backing: `FederationRegistry` (`federation/registry.rs`) — real: `upsert`/`remove`/`get`/`all`/`mark_active`/`mark_lost`/`save`/`load` + F-1c reconnect lifecycle.

| Verb | Backing | Note |
|---|---|---|
| `federation list` | **BACKED** | reads `all()`; `--state` honest (active/all — no `pending` state exists) |
| `federation defederate` | **BACKED** | `remove()` + local cleanup + persist |
| `federation accept` | **ABSENT** | no approval queue — handshake isn't admin-gated, completes straight to ACTIVE |
| `federation reject` | **ABSENT** | nothing to reject (no pending-request store) |
| `federation set-policy` | **ABSENT** | no per-peer policy store / enforcement layer |
| `federation show-policy` | **ABSENT** | reads a policy store that doesn't exist |
| `federation initiate` | **ABSENT** | would have to admin-gate the federation handshake (changes the flow) |

→ **Shipped:** `list` + `defederate` (honest-subset, this milestone). **Deferred (5):** → *federation-admin-control* arc.

### A2 — Auth Module management (Phase 8) — ABSENT (registry not built)
Backing: only Tier *claim* types (`tiers.rs`: `AuthTier`, `Tier2/3/4Claims`, `verify_tier_assertion`) + a wire error (`AuthModuleUntrusted`, code 3006). `flavours.rs` explicitly states the auth-module **surfaces are Pass-2-owned** — i.e. not yet built. There is **no Auth Module registry** to register / revoke / set-tiers / test against.

| Verb | Backing | Note |
|---|---|---|
| `auth-module list` | **ABSENT** | no registry to enumerate |
| `auth-module register` | **ABSENT** | no registry to write to |
| `auth-module revoke` | **ABSENT** | no registry; nothing to mark untrusted (the *trust check* exists; the *registry* doesn't) |
| `auth-module set-tiers` | **ABSENT** | no per-module record to hold tiers |
| `auth-module test` | **ABSENT** | no module endpoint records to probe |

→ **Deferred (all 5):** → *auth-module-registry* arc. (Tier verification logic existing ≠ a registry of trusted modules existing.)

### A3 — Bootstrap configuration (Phase 6) — ABSENT (client send-path placeholder)
Backing: server-side surface is real (`directory.rs`, `reputation.rs`, `capability.rs`, `bootstrap.register`/`_ack` wire types). The **client send-path is a placeholder** (`bootstrap/client.rs` ~0.8 KB, comment: "Phase 2: placeholder"), and there's **no `[bootstrap]` config section / local registrations store** for A3's (client-only, A3-D1) verbs.

| Verb | Backing | Note |
|---|---|---|
| `bootstrap show` | **ABSENT** | no local registrations store to read |
| `bootstrap register` | **ABSENT** | client send-path is placeholder; no store |
| `bootstrap deregister` | **ABSENT** | nothing registered to remove |
| `bootstrap set-info` | **ABSENT** | no local self-info store; re-advertise path absent |
| `bootstrap set-tiers` | **ABSENT** | same |

→ **Deferred (all 5):** → *bootstrap-client* arc. (Server-side directory machinery is real but is a *different* subsystem than A3's client verbs need.)

### A4 — Space & Room admin (Phase 9) — SPLIT
Backing: hosted-Space state is real (`accept_registration` writes locally-hosted records). The `membership.node_eject` EventType is **absent** (already design-gated, A4-D1). `NodePolicy` store is **absent**.

| Verb | Backing | Note |
|---|---|---|
| `space list-hosted` | **BACKED** | reads hosted-Space state |
| `space audit-events` | **BACKED** | reads §3.11.8 protocol log (A4-D3) |
| `space show-node-policy` | **ABSENT** | no `NodePolicy` store |
| `space set-node-policy` | **ABSENT** | no `NodePolicy` store / enforcement |
| `space force-eject` | **ABSENT** | needs `membership.node_eject` EventType (design-gated, A4-D1) |

→ **Backed subset:** `list-hosted` + `audit-events`. **Deferred:** `force-eject` → the A4-D1 wire sub-design (already scheduled, Chat-Claude+Joe pre-Phase-9); `set-node-policy`/`show-node-policy` → a *node-policy* arc (or fold into the force-eject session — Joe's call).

### A5 — Identity registry (Phase 5) — BACKED ✅
Backing: `IdentityRegistry::revoke` / `set_trust_expiry` (`identity/registry.rs`) + `replication.rs` (`add_replica`/`remove_replica`/`get_replicas`) all real. This is why Phase 5 shipped cleanly. All 4 verbs backed.

### A6 — Logging & audit (Phase 4) — SHIPPED ✅ (J-154)
Backing: **built and load-bearing as of J-154.** `audit.rs` ships `open_audit_db` / `insert_entry` / the `audit_entries` SQLite schema + indexes + the archive `DELETE`; `app.rs` installs the live `reload::Layer` handle (`LOG_RELOAD`); `admin_ops::record_action` writes entries. Phases 5 and 7 already consume this backing (every `record_action` → `audit_entries`; the 713-green run exercises it).

| Verb | Backing | Note |
|---|---|---|
| `audit query` / `export` / `archive` | **BACKED** ✅ | SQLite `audit_entries` store shipped J-154 |
| `log set-level` | **BACKED** ✅ | live `reload::Layer` handle shipped J-154 |
| `log show-level` | **BACKED** ✅ | reads the installed reload handle |

→ **Shipped (J-154).** *(Correction 2026-05-29: this row originally read "SELF-BUILDING — no store / no reload handle yet," which described the pre-J-154 tree. A6 was the foundation phase and it has already been built; nothing self-building remains. Corrected per Rule 6 / D-065 — Clair flagged the staleness against the current tree.)*

### A7 — Plugin management (Phase 10) — BACKED (as scoped) ✅
Backing: only the no-op temperature plugin trait (`temperature.rs`, `mod.rs`: "Phase 2 ships only the temperature plugin trait"). A7-D1 already scoped M6 to the 2 READ verbs (`list`/`status`); the WRITE verbs were already deferred. The 2 reads are backed against the single-plugin registry.

## Summary

| Category | Phase | Status | Ships in M6 | Deferred → arc |
|---|---|---|---|---|
| A5 Identity | 5 | BACKED | 4 verbs ✅ (shipped) | — |
| A6 Logging/audit | 4 | SHIPPED ✅ | 5 verbs ✅ (shipped J-154) | — |
| A1 Federation | 7 | SPLIT | `list` + `defederate` ✅ (shipped) | 5 → federation-admin-control |
| A4 Space/Room | 9 | SPLIT | `list-hosted` + `audit-events` | `force-eject` → A4-D1 session; 2 policy → node-policy |
| A7 Plugin | 10 | BACKED | 2 reads ✅ | (writes already deferred, A7-D1) |
| A3 Bootstrap | 6 | ABSENT | — | 5 → bootstrap-client |
| A2 Auth Module | 8 | ABSENT | — | 5 → auth-module-registry |

**M6's real shipping write-path:** A5 (4) + A6 (5, shipped J-154) + A1 subset (2) + A4 subset (2) + A7 (2) = **15 verbs**, plus `force-eject` pending its A4-D1 wire session. **18 verbs route to four post-M6 D-071 subsystem arcs**, not M6 verb phases.

## Consequence for the phase plan

- **Phases 6 (A3) and 8 (A2) have no shippable verbs** as designed → both deferred to subsystem arcs. With A6(4)/A5(5)/A1-subset(7) all shipped, M6's only remaining backed code step is **A4-subset (9)** — `list-hosted` + `audit-events`. Phase order in §5.1 amended accordingly.
- **Four post-M6 D-071 arcs** named: *federation-admin-control*, *bootstrap-client*, *auth-module-registry*, *node-policy* (the last may merge into the A4-D1 force-eject session). Each is its own audit→design→impl arc, not a verb phase.
- The pattern is now **deliberate**: M6 ships the admin write-path *for subsystems that exist*; admin surfaces for subsystems that don't are explicitly downstream of building those subsystems.

## Cross-refs

- `docs/xgen_node_admin_ops_design.md` — canonical M6 design; §5.1 / §6.A1 / §6.A3 amended from this map.
- `docs/xgen_appendix_k_en.md` — verb index; the deferred verbs stay listed (specified) but are not M6-shipping.
- D-071 — "subsystem audits precede dependent milestones" (this audit is that discipline applied reflexively to M6).
- **Per-arc backing audits (audit phase of each D-071 arc, J-161, 2026-05-30)** — this map's per-category rows are deepened into a dedicated, evidence-cited audit per arc:
  - `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md` (A1 deferred 5)
  - `tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md` (A3 deferred 5)
  - `tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md` (A2 deferred 5)
  - `tasks/M6_PROTOCOL_AUDIT_LOG_AUDIT.md` (A4 `audit-events`)
  - *(node-policy = the separate fifth deferral, not yet audited; A4 set/show-node-policy.)*
- Arc design stubs (Joe-reserved, follow the audit phase): `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md`, `tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md`, `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md`, `tasks/M6_PROTOCOL_AUDIT_LOG_DESIGN.md`.

---

*End of audit.*
