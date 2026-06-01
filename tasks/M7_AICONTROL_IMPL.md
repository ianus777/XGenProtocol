# M7 `--aicontrol` — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Framing

M7 ships `--aicontrol` v1 on both binaries: a persistent JSONL control session over a sister named pipe, plus a dedicated events pipe. It is **adapter work** — it wraps the already-shipped `xgen-client-lib::ops::*` (14 fns, M5/D-067) and `xgen-node-lib::admin_ops::*` (M6) surfaces; it adds **no new business logic**. The design is fully locked in `tasks/M7_AICONTROL_DESIGN.md` (AC-D1–AC-D6); the canonical spec is `docs/xgen_aicontrol_implementation.md` v1.2. This runbook is the Clair-facing build plan under those locks.

**The seam already exists.** Both binaries have a `dispatch_line` that resolves a verb, calls the shared command layer, and currently renders **plain text** (`OK\n`/`ERROR …\n`), discarding the structured result — with in-code comments naming M7 as the consumer that will serialise it as JSONL:
- Client: `xgen-client/src/batch.rs` — `pipe_name(label)` + `dispatch_line(line, data_dir)` → arms call `crate::ops::*` via `OpContext`.
- Node: `xgen-node/src/pipe.rs` — `pipe_name(label)` + `dispatch_line` → `dispatch_admin` + `start_pipe_server` (tokio `ServerOptions`); `ActorVia::AiControl` ("aicontrol") already exists for audit attribution.

The `--aicontrol` arm is a **sister to these**: same `ops::*`/`admin_ops::*` calls, but (a) JSONL command in, (b) `cmd` resolved per AC-D1, (c) `$`-bindings substituted, (d) result serialised into the AC-D2 envelope instead of discarded, (e) per-command timeout (AC-D3a). The `--batch` arm is **untouched** (D-066 boundary).

**Hard rule for the whole milestone (D-065):** if a verb is not already in `ops::*`/`admin_ops::*`, M7 does **not** add it. The three client verbs `create-dm-space`/`leave`/`members` are deferred (AC-D5). `config-reload` is **not** M7-`--aicontrol` (routes to M7-standalone). Touching business logic = scope breach; stop and surface.

**Build order (audit Q2):** envelope-first (Node-dominated — already done in the AC-D2 design lock), **client-first build** (validate the locked envelope + bindings + `state` on the simpler, stable `ops::*` before applying it to the heavier admin verbs).

## 2. Sequence overview

| Commit | Scope | Checkpoint |
|---|---|---|
| **C1** | Shared substrate — JSONL codec + AC-D2 envelope types + AC-D1 `cmd`-resolver + binding engine + AC-D3d catalogue + AC-D3a timeout helper. **Pure, no pipe, no dispatch.** | — (design pins it; proceeds directly) |
| **C2** | **Client** `.aicontrol` command pipe + dispatch arm wrapping `ops::*` + `state` (AC-D3c client) + per-command timeout | **#1 before C2** (pipe/codec/dispatch-arm wiring) |
| **C3** | **Client** `.events` pipe + `subscribe`/`unsubscribe` + filter grammar (AC-D3b) + event-observation hook | **#2 before C3** (event-broadcast seam code-trace) |
| **C4** | **Node** `.aicontrol` command pipe + dispatch arm wrapping `admin_ops::*` + node `state` + `ActorVia::AiControl` attribution | — (symmetric to C2) |
| **C5** | **Node** `.events` pipe (+ Node-only `nodes` filter) | — (symmetric to C3) |
| **C6** | Close — canonical-doc SHIPPED banners + version bump, ROADMAP, CLAUDE PLAY, JOURNAL, this runbook → COMPLETED (D-074 atomic) | — |

**Split triggers (D-065 honest-scope):** (a) any verb named in the design but absent from `ops::*`/`admin_ops::*` → STOP, surface, do not implement (AC-D5 boundary). (b) the event-observation seam (C3) requires new runtime instrumentation rather than tapping an existing broadcast → STOP at checkpoint #2, surface; do not build new instrumentation in an adapter milestone. (c) any single commit exceeds ~600 lines diff → propose a family-boundary split.

**Two Joe-lock checkpoints only** (the substrate is design-pinned; the node side is symmetric to the client side). Everything else proceeds directly.

## 3. Commit 1 — shared substrate (pure; no checkpoint)

The pieces both binaries need, with **no pipe and no business-logic calls** — so it is pure and fully unit-testable in isolation.

- **Envelope types (AC-D2).** Reply `{status:"ok", cmd, id?, data}` / error `{status:"error", cmd?, id?, error:{code, category, message, instance_state, stage?, hint?}}`. `category` is a closed enum (`protocol·lifecycle·argument·connection·timeout·permission`); `stage` is the 6-variant `Stage` re-exported from `admin_ops` (or its `as_str` forms). Serde-serialises with the optional-by-source fields `skip_serializing_if`.
- **`cmd` resolver (AC-D1).** Split on the first space → `[category, verb]` (node) or `[verb]` (client). Reserved control verbs (`state`; `subscribe`/`unsubscribe` belong to the events pipe) checked **before** CLI-path resolution.
- **Binding engine (AC-D1/§5).** `bind` names a result; `$name` / `$name.field` substitution before dispatch; unknown binding → `BINDING_NOT_FOUND`; per-connection namespace.
- **Control-surface catalogue (AC-D3d).** The 9 codes + their categories as constants; `MALFORMED_COMMAND` for pre-parse failure (no `cmd` to echo). Invariant in code: control codes never carry `category: protocol`.
- **Timeout helper (AC-D3a).** The 3-tier classifier (read 5 s / write 30 s / federation 180 s) keyed off a verb-class tag, pinned by name to `AUTH_MODULE_PROBE_TIMEOUT_SECS` / `PENDING_TIMEOUT_SECS` / `FEDERATION_RELATIONSHIP_TIMEOUT_SECS`; `timeout_ms` override honored-as-is, floor-validated → `BAD_ARGUMENT`.

**Confirm-at-pickup — shared home.** Where the pure substrate lives is a structural choice (it must be reachable by both `xgen-client` and `xgen-node`): lean is `xgen-common` (both already depend on it; lowest-friction), alternative is a new `xgen-aicontrol` crate. The **per-binary dispatch arm** (which calls that binary's own `ops::*`/`admin_ops::*`) always lives in the binary. Resolve at C1 pickup; surface if `xgen-common` pulls an unwanted dependency.

**Tests (pure):** envelope serde round-trips (ok + each error source shape); `cmd` split incl. the `auth-module register` two-hyphen case + reserved-verb precedence; binding substitution incl. dot-notation + `BINDING_NOT_FOUND`; timeout classifier per tier + `timeout_ms` floor-validation; `MALFORMED_COMMAND` on non-JSON / missing `cmd`.

## 4. Joe-lock checkpoint #1 (before C2) + Commit 2 — client command pipe

**Checkpoint #1 — pin the pipe/codec/dispatch-arm wiring** (the load-bearing infra lock; fires after C1 ships, before any pipe code):
1. **Sister-pipe naming + spawn.** `.aicontrol` appended to `pipe_name()`; how the second server loop spawns alongside the existing `--batch` server without disturbing it. Code-trace the client's current pipe-server setup.
2. **Dispatch-arm reuse, not fork.** Confirm the `--aicontrol` arm calls the *same* `ops::*` fns the `batch.rs` arm calls — the only difference is envelope-out vs plain-text-discard. No business logic duplicated.
3. **Serial-per-connection model (§2.3).** One in-flight command per connection; `CONCURRENT_COMMAND_NOT_ALLOWED` if violated; binding namespace is per-connection (fresh connection = empty).
4. **Substrate home** ratified (the C1 confirm-at-pickup) + envelope realisation reviewed.

**Commit 2 (after #1 locks):** the client `.aicontrol` command pipe + dispatch arm wrapping `ops::*`; the `state` control verb (AC-D3c client core — `lifecycle`/`identity_id`/`display_name`/`is_ai`/`home_node`/`version`/`spaces[]` + live `home_node_connected` + control-owned `bindings`/`event_subscriptions`); per-command timeout. `--batch` untouched.

**Tests:** a happy-path command round-trips through the envelope; `bind` + `$`-substitution across two commands; `state` returns the locked core; `UNKNOWN_COMMAND` / `BAD_ARGUMENT` / `MALFORMED_COMMAND` shapes; an `ops::*` error maps to `category: protocol`, message-only (AC-D2 client mapping); serial-model rejection.

## 5. Joe-lock checkpoint #2 (before C3) + Commit 3 — client events pipe

**Checkpoint #2 — the event-observation seam** (the riskiest unknown; fires before C3): code-trace whether the client runtime exposes an **existing event-broadcast seam** the `.events` pipe can subscribe to (e.g. the channel/fan-out the resident already drives) **without new instrumentation** (AC-D3c guardrail). If the only path is to add new taps/counters → STOP and surface (split trigger b) — that is feature work, not adapter work.

**Commit 3 (after #2 locks):** the client `.events` pipe + `subscribe`/`unsubscribe` (first message on the pipe) + filter grammar (AC-D3b: AND-across/OR-within, empty==no-restriction, two wildcard forms raw-prefix on `EventType::as_str()`, entitlement-is-ceiling/inert, malformed→`BAD_ARGUMENT` pre-stream) + the event record + signal shapes (§3). `event_subscriptions` count now reflects attached pipes.

**Tests:** filter combination (AND/OR), empty==all-entitled, wildcard prefix match + illegal-wildcard rejection, out-of-entitlement space inert (no events, no error), malformed-filter rejection pre-stream; an observed event serialises to the §3 record shape with `received_at`.

## 6. Commit 4 — node command pipe (symmetric to C2)

The node `.aicontrol` command pipe + dispatch arm wrapping `admin_ops::*` (reusing `dispatch_admin`), built on the C1 substrate and the C2 pattern. Node `state` (AC-D3c node core — `lifecycle`/`node_id`/`operator_display_name`/`endpoint`/`auth_tiers_served` + store-derived `federated_peers`/`hosted_spaces` + control-owned `bindings`/`event_subscriptions`). **Audit attribution:** audited admin verbs invoked via `--aicontrol` tag `ActorVia::AiControl` (already in the enum — wire it through the arm). Node verb errors carry the band code + `stage` (AC-D2 node mapping). `--batch` (`pipe.rs` plain-text path) untouched. No checkpoint (symmetric to the locked C2 wiring).

**Tests:** a node admin verb round-trips with band code + stage on error; `state` node core; `ActorVia::AiControl` recorded on an audited write; reuse of `dispatch_admin` confirmed (no forked admin logic).

## 7. Commit 5 — node events pipe (symmetric to C3)

The node `.events` pipe, reusing the C3 hook + filter engine, plus the **Node-only `nodes` filter** dimension (Client rejected it with `BAD_ARGUMENT` at C3; Node honors it). Node-side signals (e.g. `federation_request_pending`, §3). No checkpoint (symmetric).

**Tests:** `nodes` filter narrows the node event stream; the Client-side `BAD_ARGUMENT` for `nodes` (added at C3) still holds; a node signal serialises.

## 8. Commit 6 — close (D-074 atomic)

Doc-only. Same-commit atomic close: `docs/xgen_aicontrol_implementation.md` SHIPPED banners on §3/§4/§6/§7/§8/§9/§10 + version bump (with honest as-built deltas per D-065 where code diverged from the v1.2 spec); `tasks/M6_*`-style backing not needed; ROADMAP (visual-tree M7 row ✅ + Present arc-CLOSED + version bump); CLAUDE PLAY → next milestone; JOURNAL milestone-close entry; this runbook + `tasks/M7_AICONTROL_DESIGN.md` + `tasks/M7_AICONTROL_AUDIT.md` → COMPLETED. DECISIONS.md: AC-D# are arc-local (D-069) — promote the `cmd` verb-exposure model to a global `D-###` here **only if** Joe calls it at close; otherwise no DECISIONS change.

## 9. Per-commit DoD + verification rigour

Each code commit (C1–C5): `cargo test --workspace` green (report the count delta); `cargo build --workspace --all-targets` 0/0; `cargo clippy --workspace --lib --tests --all-features -- -D warnings` clean. C1 is package-scoped to wherever the substrate lands + its consumers. Explicit `git add <file>` per file; `git status` before commit; multi-`-m` commit messages; Joe pushes. No "commit pushed" in any DoD checklist (the `Status: COMPLETED` header / green verification is the signal).

## 10. Confirm-at-pickup (consolidated, D-078)

- **Substrate home** — `xgen-common` (lean) vs new `xgen-aicontrol` crate (C1; checkpoint #1 ratifies).
- **AC-D3b raw-prefix predicate** — exact match on `EventType::as_str()` with the trailing `.` retained in the prefix; verify against the real type strings.
- **AC-D3c keep-or-drop fields** — client `connected_since`/`member_count`/`room_count`; node `uptime_seconds`/`active_connections`/`registered_identities`. Keep iff already cheaply available in the runtime, else drop (no new instrumentation).
- **§7.1 node read-verb names** — reconcile the canonical doc's read-verb list against the actual M2 read subset exposed in `admin_ops`/`pipe.rs`.
- **Event-observation seam** — the existence + shape of the runtime broadcast the `.events` pipe taps (checkpoint #2; gates C3).

## 11. Discipline notes

- **Adapter, not feature (D-065).** No new business logic; no new verbs beyond the shipped `ops::*`/`admin_ops::*`. AC-D5's three verbs + `config-reload` stay out.
- **`--batch` untouched (D-066).** The legacy plain-text path is preserved verbatim; `--aicontrol` is a sister, never a replacement.
- **Reserved trio inert (AC-D4).** The `authorize` stage, `PERMISSION_DENIED`, and the per-connection token are dormant in v1; do not wire per-verb gating. Pipe-access == administrator (D-082), OS-ACL-delegated.
- **AC-D# arc-local (D-069).** No DECISIONS.md change during implementation unless Joe promotes the verb model at close.
- **Stop-and-surface (Rule 3).** Any split trigger, any business-logic temptation, any missing seam → stop and surface to Joe, do not work around.

## 12. Cross-references

- `tasks/M7_AICONTROL_DESIGN.md` (v1.0) — the AC-D1–AC-D6 locks this runbook implements.
- `tasks/M7_AICONTROL_AUDIT.md` (v1.0) — Phase-0 drift-reconciliation + the build-order recommendation.
- `docs/xgen_aicontrol_implementation.md` (v1.2) — canonical spec (§2 surface, §3 events, §4 protocol, §5 bindings, §8 codes, §9 state, §10 timeout).
- `xgen-client/src/batch.rs` (`pipe_name`/`dispatch_line`) + `xgen-client/src/ops.rs` (14 fns) — the client seam.
- `xgen-node/src/pipe.rs` (`pipe_name`/`dispatch_line`/`dispatch_admin`/`start_pipe_server`) + `xgen-node/src/admin_ops.rs` (`AdminError`/`Stage`/`ActorVia::AiControl`) — the node seam.
- DECISIONS.md: D-063 (library-first multi-dispatch), D-066 (`--batch`/`--aicontrol` split), D-067 (`ops::*`), D-069, D-074, D-078, D-082.

---

*Runbook ACTIVE v1.0. C1 proceeds directly; checkpoint #1 fires before C2, checkpoint #2 before C3. Clair entry point: CLAUDE PLAY + JOURNAL J-199 per Rule 0, then §1–§3 here, then `tasks/M7_AICONTROL_DESIGN.md`.*
