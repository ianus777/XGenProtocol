# Bootstrap-Client — Backing Audit (D-071 arc, audit phase)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-30  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

The **bootstrap-client** arc is one of the four post-M6 D-071 subsystem arcs
(`tasks/M6_BACKING_AUDIT.md`). Per D-071 each arc opens with a backing audit — a
read-only, evidence-cited pass mapping the deferred verbs to what exists, so the
design phase starts from reality. Audit phase only; gaps are routed, not designed.
Verified against the live tree on 2026-05-30; absences grep-confirmed.

## Scope — the deferred verbs

A3 (Phase 6) shipped **zero** verbs in M6 (deferred at J-156). All **5** route here:

`bootstrap show` · `bootstrap register` · `bootstrap deregister` ·
`bootstrap set-info` · `bootstrap set-tiers`

A3-D1 lock: bootstrap **client**-only in M6 (this Node registers *itself* with
Bootstrap Nodes; operating *as* a Bootstrap Node is separately deferred). Design
source: `docs/xgen_node_admin_ops_design.md` §6.A3 + Appendix K.2.3.

## What EXISTS (verified)

- **Wire types** (`xgen-core/src/wire/types.rs`, `BootstrapMessage` enum) —
  `bootstrap.register` / `register_ack` / `keepalive` / `keepalive_ack` /
  `deregister`, fully defined + serde round-trip tested. The wire shape is
  **specified but not produced** in production.
- **Server-side bootstrap machinery** (`xgen-core/src/bootstrap/`) — real but a
  *different* subsystem than the client verbs need:
  - `directory.rs` — `DirectoryEntry`, `BootstrapDirectory` (register_node /
    remove_node / sorted_by_reputation), `sign_directory` / `verify_directory`.
  - `reputation.rs` — `ReputationComponents`, `ReputationRegistry`.
  - `capability.rs` — `BootstrapInfo`, `declare_bootstrap` (a Node declaring its
    *own* advertised capability; not a client-side store).
  - `http.rs` — `BOOTSTRAP_HTTP_PORT`.
- **`verify_directory`** — the one client-relevant server-side function (a client
  *would* call it on a fetched directory) — but the fetch path that feeds it is
  absent (see below).

## What is ABSENT (the gap, verified)

- **Client send-path is a placeholder.** `xgen-core/src/bootstrap/client.rs`
  (verified, the whole file is 17 lines) declares one constant
  (`DIRECTORY_MAX_AGE_SECS = 3600`) and a comment: *"Phase 2: this module is a
  placeholder. The actual reqwest HTTP client calls are implemented in xgen-node"*
  — but no such implementation exists.
- **No production sender.** Grep for `BootstrapMessage::Register|Keepalive|Deregister`
  across `xgen-node/src/**/*.rs` returns **no matches** (verified). Nothing sends
  `bootstrap.register` in production; the only mentions are test-scenario comments.
- **No `[bootstrap]` config section.** `NodeConfig` (`xgen-node/src/app.rs`) has
  `node` / `paths` / `logging` / `sync` only. Grep for `bootstrap` in `app.rs`
  returns comments only (verified) — no config struct, no section.
- **No local registrations store.** Nothing records *which Bootstrap Nodes this
  Node is registered with* (no `BootstrapRegistration` type / store).
- **No local self-info store.** `BootstrapInfo` exists in xgen-core but is used
  server-side to *advertise*; there is no persisted local self-info this Node
  reads/updates and re-advertises from.

## Per-verb backing

| Verb | Class | Backing | Evidence |
|---|---|---|---|
| `bootstrap show` | READ | **ABSENT** | no local registrations store to read |
| `bootstrap register` | WRITE | **ABSENT** | client send-path placeholder; no store |
| `bootstrap deregister` | DESTRUCTIVE | **ABSENT** | nothing registered to remove |
| `bootstrap set-info` | WRITE | **ABSENT** | no self-info store; no re-advertise path |
| `bootstrap set-tiers` | WRITE | **ABSENT** | no advertised-tiers store; no re-advertise path |

## Verdict

**GAP IDENTIFIED — HIGH (whole-subsystem).** All 5 verbs are absent-backed. The
gap is **wider than "just the send-path"** that the J-156 framing emphasised: the
send-path is a placeholder *and* there is no `[bootstrap]` config section, no
registrations store, and no self-info store. The server-side directory/reputation
machinery is real but is the Bootstrap-Node-*server* role — orthogonal to the
client verbs. M6 backing-map assumption (all 5 absent) is **confirmed**, with the
refinement that the gap spans the send-path **and** local persistence, not the
send-path alone.

## What the design phase must build (inputs to the design arc — NOT the design)

1. **Client HTTP send-path** — real (reqwest) `bootstrap.register` /
   `keepalive` / `deregister` send + `*_ack` receive, in `xgen-node` (the shell
   crate, per the placeholder's own note), driving the existing `BootstrapMessage`
   wire types.
2. **`[bootstrap]` config section** on `NodeConfig` (D-035 file convention) — the
   bootstrap nodes to register with, the advertised self-info, advertised tiers.
3. **Local registrations store** — per-Bootstrap-Node records (bootstrap_id, url,
   directory_url, registered_at, expires_at) with add/remove/get/list/update.
   Consumed by `show` (read), `register` (add), `deregister` (remove), and by
   re-advertise (iterate).
4. **Local self-info store** — the `BootstrapInfo` + advertised tiers this Node
   publishes; consumed by `set-info` / `set-tiers` and the re-advertise loop.
5. **Re-advertise + keepalive scheduling** — best-effort fan-out of updated
   self-info/tiers to all registered Bootstrap Nodes (A3-D2: local update succeeds
   even if a re-advertise fails — honest per D-065), plus a TTL keepalive task.

A design decision the design phase must take: **where local bootstrap state
lives** — TOML config section vs a sibling JSON/SQLite store — and how registration
TTL/keepalive scheduling composes with the existing reconnect-scheduler pattern.

## Carry-overs & cross-refs

- `docs/xgen_node_admin_ops_design.md` §6.A3 (verb specs, A3-D1/A3-D2) + Appendix K.2.3.
- Spec §3.14.3 / §3.14.4 / §3.14.7 (bootstrap register / directory fetch / keepalive).
- `tasks/M6_BACKING_AUDIT.md` A3 row. Future design stub:
  `tasks/M6_BOOTSTRAP_CLIENT_DESIGN.md` (Joe-reserved).
- The J-081 pattern (a "missing mechanism" whose wire shape exists but has zero
  production callers) recurs here — the bootstrap client is to A3 what federation
  push was to Stage 6 in the Propagation Reliability Audit.
- D-071 / D-069 / D-065. Sibling arc audits:
  `tasks/M6_FEDERATION_ADMIN_CONTROL_AUDIT.md`,
  `tasks/M6_AUTH_MODULE_REGISTRY_AUDIT.md`, `tasks/M6_PROTOCOL_AUDIT_LOG_AUDIT.md`.

---

*End of audit (audit phase). Design + implementation are the subsequent arc steps.*
