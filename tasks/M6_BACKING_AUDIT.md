# M6 Backing-Map Audit — Verb-to-Subsystem Reality Check
> **Status**: ACTIVE  
> Version: 1.6  
> Date: May 2026  
> **Last updated**: 2026-05-31  
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

### A1 — Federation management (Phase 7) — SHIPPED ✅ (arc CLOSED)
Backing: `FederationRegistry` (`federation/registry.rs`) — real: `upsert`/`remove`/`get`/`all`/`mark_active`/`mark_lost`/`save`/`load` + F-1c reconnect lifecycle.

| Verb | Backing | Note |
|---|---|---|
| `federation list` | **BACKED** | reads `all()`; `--state` now filters the real FAC-D2 `FederationState` (J-177) |
| `federation defederate` | **BACKED** | `remove()` + local cleanup + persist |
| `federation accept` | **SHIPPED** | 2a arc (J-177): dequeue + upsert `Active` + schedule reconnect; FED_3005 |
| `federation reject` | **SHIPPED** | 2a arc (J-177): dequeue + permanent `Rejected` tombstone (suppresses re-enqueue); FED_3005 |
| `federation set-policy` | **SHIPPED** | 2b arc (J-182): per-peer `FederationPolicy` sibling store + `policy_permits` enforcement (both sites, default-permit); FED_3008 |
| `federation show-policy` | **SHIPPED** | 2b arc (J-182): reads the policy store or the default with `is_default` |
| `federation initiate` | **SHIPPED** | 2a arc (J-177): known-peer outbound via `attempt_reconnect`; FED_3006/3007 |

→ **ALL 7 shipped.** `list` + `defederate` (M6 honest-subset) + `accept` + `reject` + `initiate` (federation-admin-control **2a**, J-174→J-178) + `set-policy` + `show-policy` (federation-admin-control **2b**, J-179→J-183). The federation-admin-control arc is **CLOSED**.

### A2 — Auth Module management (Phase 8) — SHIPPED ✅ (auth-module-registry arc, J-185→J-189; arc CLOSED)
Originally ABSENT: backing was only Tier *claim* types (`tiers.rs`) + the `AuthModuleUntrusted`(3006) wire error; no registry of trusted modules existed. The **auth-module-registry** D-071 arc built it: `AuthModuleXgid` (7th XGID flavour, **D-083**) + `AuthModuleRecord`/`AuthModuleRegistry` (`xgen-core/src/auth/module_registry.rs`, persisted at `xgen-node_auth_modules.json`) + the 5 verbs in `admin_ops` (AMR-D1 standalone — store + verbs, no runtime consumer yet; the 3006/registration consultation is its own future arc).

| Verb | Backing | Note |
|---|---|---|
| `auth-module list` | **SHIPPED ✅** (J-187) | enumerates `AuthModuleRegistry` (revoked included, flagged) |
| `auth-module register` | **SHIPPED ✅** (J-187) | `--pubkey` derives `module_id` (AMR-D3); WRITE/audited |
| `auth-module revoke` | **SHIPPED ✅** (J-187) | block-only-retains (A2-D1); unknown → `AUTHMOD_6101` |
| `auth-module set-tiers` | **SHIPPED ✅** (J-187) | replaces tier set; `AUTHMOD_6101`/`6103` |
| `auth-module test` | **SHIPPED ✅** (J-188) | connectivity-only probe (5 s); unreachable = result, not error |

→ **SHIPPED (all 5):** *auth-module-registry* arc, CLOSED J-189. (The deferred enforcement consultation — registration steps / 3006 — remains a future arc per AMR-D1.)

### A3 — Bootstrap configuration (Phase 6) — SHIPPED ✅ (bootstrap-client arc CLOSED, J-190→J-195)
Original verdict (2026-05-29): ABSENT — the client send-path was a placeholder (`bootstrap/client.rs`) and there was no `[bootstrap]` config / local store. The *bootstrap-client* arc built the client: `[bootstrap]` config seed + `BootstrapRegistrationStore` (combined `xgen-node_bootstrap.json`, BC-D1/D2) + `bootstrap/signing.rs` (sign/verify `BootstrapMessage`) + `bootstrap_client.rs` (framed send-path over the normal transport, BC-D3 — NOT HTTP) + `bootstrap_keepalive.rs` (TTL scheduler + best-effort re-advertise).

| Verb | Backing | Note |
|---|---|---|
| `bootstrap show` | **SHIPPED** ✅ | reads the registrations store + self-info (J-193) |
| `bootstrap register` | **SHIPPED** ✅ | `--url`/`--pubkey`; drives the framed send-path, stores the ack (J-193) |
| `bootstrap deregister` | **SHIPPED** ✅ | sends signed Deregister, removes from store (J-193) |
| `bootstrap set-info` | **SHIPPED** ✅ | local write + best-effort re-advertise (A3-D2, J-193/J-194) |
| `bootstrap set-tiers` | **SHIPPED** ✅ | local self-info only — no wire field carries tiers (Checkpoint #1(d), J-193) |

→ **SHIPPED (all 5):** *bootstrap-client* arc CLOSED. New `BOOT_71xx` admin error block (distinct from the spec §3.14.8 server-side wire codes). (Directory-fetch HTTP + operating *as* a Bootstrap-Node server remain separately deferred — out of A3-D1 client-only scope.)

### A4 — Space & Room admin (Phase 9) — SHIPPED + SPLIT

> **Amended 2026-05-30 (J-163).** A4 shipped J-157→J-160; the cells below are updated from the J-157 pre-implementation snapshot (`force-eject`/`unban` SHIPPED; `audit-events` deferred to the *protocol-audit-log* arc).
> **Amended 2026-05-30 (J-169).** The *protocol-audit-log* arc closed: `audit-events` ABSENT → **SHIPPED** (J-167) + a new `audit-rebuild` verb (J-168, PAL-D3). The §3.11.8 protocol log is now built (`xgen-node/src/protocol_audit.rs`; writer hook inside `persist_event`). A4 is fully resolved bar the 2 node-policy verbs.
> **Amended 2026-05-31 (J-197).** The *node-policy* arc closed: `set-node-policy` + `show-node-policy` ABSENT → **SHIPPED** (J-197, C1). The `NodePolicy` store is now built (`xgen-core/src/space/node_policy.rs`, persisted `xgen-node_node_policy.json`; Fork X — stored inert, enforcement deferred to the temperature-plugin arc). **A4 is now fully resolved.**

Backing: hosted-Space state is real (`accept_registration` writes locally-hosted records). The `membership.node_eject` + `membership.node_unban` EventTypes **shipped** (J-159; Node-signed, wire 3043). The §3.11.8 protocol audit log **shipped** (protocol-audit-log arc, J-166→J-168). The `NodePolicy` store **shipped** (node-policy arc, J-197).

| Verb | Backing | Note |
|---|---|---|
| `space list-hosted` | **SHIPPED** | reads hosted-Space state (J-157) |
| `space audit-events` | **SHIPPED** | reads the §3.11.8 protocol audit log (protocol-audit-log arc, J-167) |
| `space audit-rebuild` | **SHIPPED** | rebuild-from-DAG, PAL-D3 (protocol-audit-log arc, J-168) |
| `space show-node-policy` | **SHIPPED** | reads the `NodePolicy` store (node-policy arc, J-197) |
| `space set-node-policy` | **SHIPPED** | writes the `NodePolicy` store, Fork X inert (node-policy arc, J-197) |
| `space force-eject` | **SHIPPED** | `membership.node_eject` (J-159); live fan-out + federation push (J-160) |
| `space unban` | **SHIPPED** | `membership.node_unban` (J-159) — reversal of node_eject |

→ **Shipped (all 7):** `list-hosted` (J-157) + `force-eject` + `unban` (J-159 Option A; J-160 Option B live fan-out) + `audit-events` (J-167) + `audit-rebuild` (J-168, protocol-audit-log arc) + `set-node-policy` + `show-node-policy` (J-197, node-policy arc).

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
| A1 Federation | 7 | SHIPPED ✅ | all 7 ✅ — `list`+`defederate`+`accept`+`reject`+`initiate` (2a, J-178) + `set-policy`+`show-policy` (2b, J-182) | — (arc CLOSED) |
| A4 Space/Room | 9 | SHIPPED ✅ | all 7 — `list-hosted` + `force-eject` + `unban` + `audit-events` + `audit-rebuild` (protocol-audit-log) + `set-node-policy` + `show-node-policy` (node-policy, J-197) | — (all arcs CLOSED) |
| A7 Plugin | 10 | BACKED | 2 reads ✅ | (writes already deferred, A7-D1) |
| A3 Bootstrap | 6 | SHIPPED ✅ | `show`/`register`/`deregister`/`set-info`/`set-tiers` (bootstrap-client arc, CLOSED) | — |
| A2 Auth Module | 8 | SHIPPED ✅ | 5 (J-185→J-189) | — (arc CLOSED) |

**M6's real shipping write-path:** A5 (4) + A6 (5, shipped J-154) + A1 subset (2) + A4 subset (3: `list-hosted` + `force-eject` + `unban`) + A7 (2) = **16 verbs** (all shipped J-154→J-160). **18 verbs route to four post-M6 D-071 subsystem arcs + node-policy**, not M6 verb phases. **Update (J-182):** the *federation-admin-control* arc (2a + 2b) is now CLOSED — A1's 5 deferred verbs all shipped, so the A1 row above is fully ✅. The remaining D-071 arcs are *auth-module-registry* (A2, 5) + *bootstrap-client* (A3, 5); *protocol-audit-log* closed at J-169; *node-policy* (2) is the fifth deferral. **Update (J-189):** the *auth-module-registry* arc (A2, 5) is now CLOSED — the A2 row above is fully ✅. The only remaining D-071 verb arc is *bootstrap-client* (A3, 5); *node-policy* (2) is the fifth deferral. **Update (J-195):** the *bootstrap-client* arc (A3, 5) is now CLOSED — the A3 row above is fully ✅. **All four D-071 verb arcs have shipped** (federation-admin-control · protocol-audit-log · auth-module-registry · bootstrap-client); only *node-policy* (2 verbs — `set/show-node-policy`) remains as the fifth deferral before M7 `--aicontrol`. **Update (J-197):** the *node-policy* arc is now CLOSED (C1 shipped the store + both verbs + threading; C2 doc-close). **All M6 deferrals are now closed** — every M6 admin verb across all seven categories has shipped. Next is **M7 `--aicontrol`** (reuses `admin_ops::*`).

## Consequence for the phase plan

- **Phases 6 (A3) and 8 (A2) have no shippable verbs** as designed → both deferred to subsystem arcs. With A4 now shipped (J-157→J-160), M6's backed write-path was **complete (16 verbs)**; the remaining work — the four D-071 subsystem arcs + node-policy — has now **all shipped** (last: node-policy, J-197). Phase order in §5.1 amended accordingly.
- **Four post-M6 D-071 arcs** named: *federation-admin-control*, *bootstrap-client*, *auth-module-registry*, *protocol-audit-log* (`audit-events`) — **all CLOSED**. **node-policy** (`set/show-node-policy`) was the separate fifth deferral — **CLOSED at J-197**. Each was its own audit→design→impl arc, not a verb phase.
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
  - `tasks/M6_NODE_POLICY_AUDIT.md` (the fifth deferral; A4 set/show-node-policy — arc CLOSED J-197).
- Arc design stubs (Joe-reserved, follow the audit phase): `tasks/M6_FEDERATION_ADMIN_CONTROL_DESIGN.md`, `tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md`, `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md`, `tasks/M6_PROTOCOL_AUDIT_LOG_DESIGN.md`.

---

*End of audit.*
