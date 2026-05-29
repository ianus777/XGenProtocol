# Handoff — M6 (new) Node Admin Write Path: Implementation
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-29  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Chat Claude → **Clair** handoff. M6 (new) — the Node admin write path — has completed its full design phase (Phase 0: Pass 1 + Pass 2 + Pass 3 + Block 4; JOURNAL J-151). The implementation gate is cleared. This note is Clair's entry point to begin implementing.

## Gate status — CLEARED

- **Propagation Reliability Audit: COMPLETED** (`docs/xgen_propagation_reliability.md`, closed 2026-05-18). This was the named gate on M6 implementation (M6 design doc §4.3 / §5.3). Its §5 finding — the symmetric rejection signal — is already locked into the Phase 2 design (envelope-level `event_id`).
- **Before starting:** confirm the workspace is green at the current baseline — `cargo test --workspace` (expect 637) and `cargo build --workspace --all-targets` (0 errors).

## Read first (canonical sources)

1. `docs/xgen_node_admin_ops_design.md` — the canonical M6 design. §1–§5 (principles, the `EventAccepted` addition, propagation lifecycle, phase plan); §6 (per-category verb specs).
2. `docs/xgen_appendix_k_en.md` — Appendix K, the at-a-glance verb + schema index (33 verbs, error-code bands).
3. `docs/xgen_propagation_reliability.md` §6.5 — the locked rejection-signal design (envelope `event_id`).
4. `DECISIONS.md` — D-067 (single source `admin_ops::*`), D-070 (accept/reject symmetry), D-082 (administrator terminology).

## Phase order (M6 design §5.1)

```
Phase 1 — Client gap patches (R1/R2/R3 — confirm with Joe first; see Gates)
Phase 2 — admin_ops::* scaffolding + TransportMessage envelope event_id + EventAccepted + rejection paths
Phase 3 — Read-only completions on existing --batch
Phase 4 — A6 Logging/audit (the audit primitive lands here; all later phases consume it)
Phase 5 — A5 Identity registry
Phase 6 — A3 Bootstrap configuration
Phase 7 — A1 Federation management
Phase 8 — A2 Auth Module management
Phase 9 — A4 Space/Room admin   [DESIGN-GATED — see Gates]
Phase 10 — A7 Plugin management
```

Each of Phases 3–10 adds one category's verbs against the Phase 2 scaffolding. Per-verb schemas: §6 + Appendix K.

## Phase 2 — the first implementation chunk (§5.2 + audit §6.5)

1. `xgen-node-lib::admin_ops` module skeleton.
2. `AdminContext` + `AdminError` types (mirror `OpContext` / the Client error pattern).
3. `TransportMessage` envelope **`event_id: Option<String>`** + `EventAccepted { accepted_at }` — the **only** new variant. `Error` covers rejection by populating envelope `event_id`. **No `EventRejected` variant.**
4. Client-side `EventAccepted` handling + envelope-`event_id` correlation against in-flight submissions.
5. `xgen-node-lib::audit` module skeleton (SQLite `audit_entries`, §2.6.4; empty table on first start).
6. `pipe::dispatch_line` routes new write verbs into `admin_ops::*` (read-only allowlist preserved unchanged).
7. Rejection paths in `process_inbound` (`xgen-node/src/app.rs:846–934` region) emit `Error` with `event_id: Some(...)`.
8. Confirm `#[serde(skip_serializing_if = "Option::is_none")]` on envelope `event_id` for pre-M6 client backward-compat.

**Latitude:** the Rust realisation of envelope `event_id` (wrapping struct vs flattened field vs tagged union) is Clair's call — *cleaner is better*. Wire-format-visible changes beyond the locked `event_id` require Joe-lock.

## Gates / open items (NOT pure-Clair)

1. **Phase 9 is design-gated.** A4-D1 locked the *direction* — `force-eject` emits a new `membership.node_eject` EventType, Node-keypair-signed — but the **wire/validation sub-design** (exact event shape, who-may-emit rule, Ch3 §3.3 registry entry, Appendix I entry, federation-validation interaction) is a **Chat-Claude-+-Joe session that must run before Phase 9 codes**. Do not implement Phase 9 until it lands. (Same pattern as the `EventAccepted` shape before Phase 2.)
2. **Phase 1 R1/R2/R3 not yet confirmed.** Block 4 walked the §6 categories, not Phase 1's Client gap-patches (`rooms` / `members` Client commands; `federate` deferred to Phase 7). Confirm scope with Joe at Phase 1 start — Phase 1 may collapse to near-zero (R3).

## Do NOT build (deferred verbs — Appendix K.3)

- `federation signal-defederation` (A1-D3) — Bootstrap reputation-consumer surface unbuilt.
- `space migrate-as-source` (A4-D2) — §3.12 migration flow unbuilt.
- `plugin load` / `configure` / `unload` (A7-D1) — single no-op plugin; no extensible surface.

## Implementation disciplines

- **`admin_ops::*` is the single source** (D-067); the `--batch` dispatcher and the future `--aicontrol` (M7) both call it. No parallel implementations.
- **Terminology (D-082):** "administrator" (prose) / "admin" (code, CLI, error-codes, config) for the runtime principal; never "operator" (reserved for the AI-operator role).
- **Error-code bands** per Appendix K.5: `AUTH_2xxx` · `FED_3xxx` · `GENERIC_4000` · `AUDIT_5xxx` + `LOG_51xx` · `IDENT_6xxx` · `BOOT_7xxx` · `SPACE_8xxx` · `PLUGIN_9xxx`.
- **Revocations don't cascade** (A2-D1 / A5-D1): `identity revoke` / `auth-module revoke` are block-only.
- **Two audit logs:** `audit *` → SQLite admin trail (§2.6.4); `space audit-events` → §3.11.8 protocol log. Don't conflate.
- **Commit discipline:** per-file `git add`; DoD checklists must NOT include "commit pushed" (the `Status: COMPLETED` header is the real signal); write each file to disk before the next.
- **Phase task files:** one `tasks/M6_PHASE_N_IMPL.md` per phase as you go, following the existing `*_IMPL.md` shape.

## Definition of Done (this handoff)

This handoff is consumed once Clair opens Phase 2 (or Phase 1, if R1/R2/R3 is confirmed with Joe). It flips to `Status: COMPLETED` when M6 implementation is underway with its own per-phase task files tracking progress.

---

*End of handoff.*
