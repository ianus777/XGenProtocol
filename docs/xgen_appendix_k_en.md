# Appendix K — M6 Node Admin Verb + Schema Reference
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## K.1 Purpose

This appendix is the at-a-glance reference for every Node admin verb introduced by **M6 (new) — the Node admin write path**. The authoritative design — per-verb audit implications, failure-stage semantics, propagation behaviour, and the full reasoning behind each Block 4 lock — lives in `docs/xgen_node_admin_ops_design.md` §6; this appendix mirrors it as a schema index.

M6 ships **33 admin verbs** across seven categories (+ 5 deferred). All route through `xgen-node-lib::admin_ops::*` (D-067), dispatched via the `xgen-node --batch` pipe; two-token naming per the M6 design doc §2.6.6. Error-code prefixes and bands per §2.7. The verbs were locked verb-by-verb in Block 4 of M6 Phase 0 (2026-05-29; JOURNAL J-151).

---

## K.2 Verb reference by category

### K.2.1 — A6 Logging & audit (Phase 4) — 5 verbs
| Verb | Class | Args → Result | Errors |
|---|---|---|---|
| `log set-level` | WRITE | `{module?, level}` → `{module, previous_level, new_level, applied}` | `LOG_5101/5102` |
| `log show-level` | READ | `{module?}` → `{levels[]}` | `GENERIC_4000` |
| `audit archive` | DESTRUCTIVE | `{before, output?}` → `{archived_count, archive_path, oldest_ts, newest_ts}` | `AUDIT_5001/5002/5010` |
| `audit query` | READ | `{actor?, verb?, since?, until?, outcome?, limit?}` → `{entries[], total_matched, returned}` | `AUDIT_5010` |
| `audit export` | READ | `{…filters, output, format?}` → `{exported_count, output_path, format}` | `AUDIT_5010/5020` |

### K.2.2 — A5 Identity registry (Phase 5) — 4 verbs
| Verb | Class | Args → Result | Errors |
|---|---|---|---|
| `identity show` | READ | `{identity_id}` → `{record}` | `IDENT_6001` |
| `identity revoke` | DESTRUCTIVE | `{identity_id, reason?}` → `{identity_id, revoked_at, stale_membership_spaces[]}` | `IDENT_6001/6002` |
| `identity set-trust-expiry` | WRITE | `{identity_id, expiry}` → `{identity_id, previous_expiry, new_expiry}` | `IDENT_6001/6010` |
| `identity manage-replica` | WRITE | `{identity_id, action, node_id?}` → `{identity_id, replicas[]}` | `IDENT_6001/6020/6021` |

(`identity list` ships from M2.)

### K.2.3 — A3 Bootstrap configuration (Phase 6) — 5 verbs
| Verb | Class | Args → Result | Errors |
|---|---|---|---|
| `bootstrap show` | READ | `{bootstrap_id?}` → `{registrations[], bootstrap_info, auth_tiers_served[]}` | `GENERIC_4000` |
| `bootstrap register` | WRITE | `{bootstrap_url}` → `{bootstrap_id, registered_at, advertised_tiers[]}` | `BOOT_7001/7002/7010` |
| `bootstrap deregister` | DESTRUCTIVE | `{bootstrap_id}` → `{bootstrap_id, deregistered_at}` | `BOOT_7003/7010` |
| `bootstrap set-info` | WRITE | `{display_name?, description?, contact?}` → `{bootstrap_info, re_advertised_to[]}` | `BOOT_7020` |
| `bootstrap set-tiers` | WRITE | `{tiers[]}` → `{auth_tiers_served[], re_advertised_to[]}` | `BOOT_7021` |

### K.2.4 — A1 Federation management (Phase 7) — 7 verbs
| Verb | Class | Args → Result | Errors |
|---|---|---|---|
| `federation list` | READ | `{state?, limit?, cursor?}` → `{relationships[], total_matched, returned, next_cursor}` | `FED_3001` |
| `federation accept` | WRITE | `{peer_node_id, endpoint?}` → `{peer_node_id, federated_at, state}` | `FED_3002/3003/3010` |
| `federation reject` | WRITE | `{peer_node_id, reason?}` → `{peer_node_id, rejected_at}` | `FED_3002` |
| `federation initiate` | WRITE | `{peer_node_id, endpoint}` → `{peer_node_id, state, initiated_at}` | `FED_3003/3010/3011` |
| `federation defederate` | DESTRUCTIVE | `{peer_node_id, reason?}` → `{peer_node_id, defederated_at, cleaned_spaces[]}` | `FED_3004/3010` |
| `federation set-policy` | WRITE | `{peer_node_id, mode, allowed_spaces?, rate_limit?}` → `{peer_node_id, policy}` | `FED_3004/3020` |
| `federation show-policy` | READ | `{peer_node_id?}` → `{policies[]}` | `GENERIC_4000` |

### K.2.5 — A2 Auth Module management (Phase 8) — 5 verbs
| Verb | Class | Args → Result | Errors |
|---|---|---|---|
| `auth-module list` | READ | `{revoked?}` → `{modules[]}` | `GENERIC_4000` |
| `auth-module register` | WRITE | `{url, public_key, tiers[]}` → `{module_id, accepted_tiers[], registered_at}` | `AUTH_2001/2002/2003/2021` |
| `auth-module revoke` | DESTRUCTIVE | `{module_id, reason?}` → `{module_id, revoked_at, note}` | `AUTH_2004/2005` |
| `auth-module set-tiers` | WRITE | `{module_id, tiers[]}` → `{module_id, accepted_tiers[]}` | `AUTH_2004/2021` |
| `auth-module test` | READ | `{module_id}` → `{module_id, reachable, response_time_ms?, reported_tiers?}` | `AUTH_2004` |

### K.2.6 — A4 Space & Room admin (Phase 9) — 5 verbs
| Verb | Class | Args → Result | Errors |
|---|---|---|---|
| `space list-hosted` | READ | `{name_filter?}` → `{spaces[]}` | `GENERIC_4000` |
| `space force-eject` | DESTRUCTIVE | `{space_id, identity_id, reason?}` → `{space_id, identity_id, ejected_at, event_id}` | `SPACE_8001/8002/8003/8004` |
| `space set-node-policy` | WRITE | `{space_id, policy}` → `{space_id, policy}` | `SPACE_8001/8020` |
| `space show-node-policy` | READ | `{space_id}` → `{space_id, policy}` | `SPACE_8001` |
| `space audit-events` | READ | `{space_id, event_type?, since?, until?, limit?, cursor?}` → `{events[], returned, next_cursor}` | `SPACE_8001/8010` |

`force-eject` is the only M6 verb that emits a Space-DAG event (`membership.node_eject`, Node-signed — A4-D1; detailed wire sub-design opens Phase 9).

### K.2.7 — A7 Plugin management (Phase 10) — 2 verbs
| Verb | Class | Args → Result | Errors |
|---|---|---|---|
| `plugin list` | READ | `{}` → `{plugins[]}` | `GENERIC_4000` |
| `plugin status` | READ | `{plugin_name}` → `{name, version, status, kind, events_consumed?, last_activity?}` | `PLUGIN_9001` |

---

## K.3 Deferred verbs (re-enter post-M6)

| Verb | Category | Reason |
|---|---|---|
| `federation signal-defederation` | A1 | Bootstrap reputation-consumer surface unbuilt (A1-D3) |
| `space migrate-as-source` | A4 | §3.12 migration flow heavy / unbuilt (A4-D2) |
| `plugin load` / `configure` / `unload` | A7 | Single no-op plugin; no extensible surface (A7-D1) |

---

## K.4 Cross-cutting Block 4 locks

- **Revocations don't cascade** (A2-D1 / A5-D1): `identity revoke` and `auth-module revoke` are block-only; existing memberships / Trust Assertions go inert or age out at natural expiry. Retroactive cascade is deferred (depends on the A4 signing machinery).
- **Two audit logs, two audiences:** `audit *` (A6) targets the SQLite admin trail (§2.6.4); `space audit-events` (A4) targets the §3.11.8 protocol audit log. Distinct stores, distinct purposes.
- **Audit-the-auditor** (A6-D4): READ verbs are not audited; WRITE / DESTRUCTIVE verbs and the data-extracting `audit export` write audit entries.
- **Accept signal:** only `force-eject` emits a Space-DAG event; it is pipe-originated, so no `EventAccepted` wire message is sent — the verb result is the G2-boundary analog (returns after validate + persist; fan-out / federation async).

---

## K.5 Error-code bands (harmonised)

| Prefix | Category | Band |
|---|---|---|
| `AUTH_*` | Auth Module management | 2xxx |
| `FED_*` | Federation management | 3xxx |
| `GENERIC_*` | Verb-agnostic | 4000 |
| `AUDIT_*` | Audit administration | 5000s |
| `LOG_*` | Logging administration | 51xx |
| `IDENT_*` | Identity registry | 6xxx |
| `BOOT_*` | Bootstrap configuration | 7xxx |
| `SPACE_*` | Space / Room admin | 8xxx |
| `PLUGIN_*` | Plugin management | 9xxx |

Each prefix owns a distinct band; `GENERIC_4000` is the single cross-cutting code. The `--aicontrol` JSONL surface (M7) will carry the same codes in structured form, so they propagate forward without renaming.

---

*End of Appendix K.*
