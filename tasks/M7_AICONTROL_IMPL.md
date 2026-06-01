# M7 `--aicontrol` — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.2  
> Date: May 2026  
> **Last updated**: 2026-06-01  
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
| **C2** | **Client** `.aicontrol` command pipe + dispatch arm wrapping `ops::*` + `state` (AC-D3c client) + per-command timeout — ✅ SHIPPED (J-202) | **#1 before C2** (pipe/codec/dispatch-arm wiring) — closed |
| ~~**C3**~~ | ~~Client `.events` pipe + filter grammar~~ — **DEFERRED → M7-events arc** (checkpoint #2 fired split-trigger (b): reading (B) collides with the Node's one-sender-per-identity `ClientSenders` registry; the fix is a node mechanism change, out of adapter scope — see §5) | **#2 before C3** — fired → STOP |
| **C4** | **Node** `.aicontrol` command pipe + dispatch arm wrapping `admin_ops::*` + node `state` + `ActorVia::AiControl` attribution | — (symmetric to C2) |
| ~~**C5**~~ | ~~Node `.events` pipe~~ — **DEFERRED → M7-events arc** (paired with C3 + the node multi-connection enhancement) | — |
| **C6** | Close — canonical-doc SHIPPED banners (command pipes only) + events sections marked deferred + version bump, ROADMAP, CLAUDE PLAY, JOURNAL, this runbook → COMPLETED (D-074 atomic) | — |

**M7 v1 reshape (checkpoint #2, 2026-06-01): command-pipes-only = C1 + C2 + C4 + C6.** The events pipe (C3/C5) + the prerequisite node multi-connection-per-identity fan-out change defer to a named follow-on, the **M7-events arc** (distinct from the `--aicontrol` hardening arc — AC-D4 token + AC-D6 idempotency). Findings carried into that arc so they are not re-derived: **Q1** the `ClientSenders: HashMap<IdentityXgid, Sender>` collision (second same-identity WS clobbers/removes the resident's sender, breaking AI-resident fan-out) + the required `HashMap<IdentityXgid, Vec<(conn_id, Sender)>>` shape in `fanout.rs` + `app.rs`; **Q2** subscription = from-now-forward live (no `SyncRequest` — fan-out registration alone delivers; history via the command pipe's `history` verb); **Q3** gaps-visible-across-reconnect (no silent replay) + a process-wide `event_subscriptions` registry threaded to both servers as a C3 item.

**Split triggers (D-065 honest-scope):** (a) any verb named in the design but absent from `ops::*`/`admin_ops::*` → STOP, surface, do not implement (AC-D5 boundary). (b) the event-observation seam requires new runtime instrumentation rather than tapping an existing broadcast → STOP, surface; do not build new instrumentation in an adapter milestone — **FIRED at checkpoint #2 → C3/C5 deferred**. (c) any single commit exceeds ~600 lines diff → propose a family-boundary split.

**Two Joe-lock checkpoints** (both closed): #1 before C2 (pipe/codec/dispatch wiring — locked, applied at C2); #2 before C3 (event-seam code-trace — fired split-trigger (b), C3/C5 deferred). C1/C4/C6 proceed directly.

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

## 5. Joe-lock checkpoint #2 (before C3) + Commit 3 — client events pipe — ⛔ DEFERRED (M7-events arc)

**⛔ DEFERRED at checkpoint #2 (2026-06-01, J-203).** The code-trace found **no existing in-process event-broadcast seam** to tap (both resident recv loops are single-consumer/discard; the tee is named-but-unbuilt in `service.rs`). The only no-resident-instrumentation alternative — a dedicated authenticated `.events` WS consuming the Node's `apply_fanout` push (reading B) — **collides with the Node's one-sender-per-identity `ClientSenders` registry**: a second same-identity WS clobbers/removes the resident's sender (`app.rs:1238`/`:1407`), breaking the AI resident's fan-out. The clean fix (multi-connection-per-identity in `fanout.rs` + `app.rs`) is a **node mechanism change, out of adapter scope → split-trigger (b)**. C3 + C5 + the node enhancement defer to the **M7-events arc** (see §2 reshape for the carried Q1/Q2/Q3 findings). The original C3 spec is retained below as the arc's starting point.

**Checkpoint #2 — the event-observation seam** (the riskiest unknown; fires before C3): code-trace whether the client runtime exposes an **existing event-broadcast seam** the `.events` pipe can subscribe to (e.g. the channel/fan-out the resident already drives) **without new instrumentation** (AC-D3c guardrail). If the only path is to add new taps/counters → STOP and surface (split trigger b) — that is feature work, not adapter work.

**Commit 3 (after #2 locks):** the client `.events` pipe + `subscribe`/`unsubscribe` (first message on the pipe) + filter grammar (AC-D3b: AND-across/OR-within, empty==no-restriction, two wildcard forms raw-prefix on `EventType::as_str()`, entitlement-is-ceiling/inert, malformed→`BAD_ARGUMENT` pre-stream) + the event record + signal shapes (§3). `event_subscriptions` count now reflects attached pipes.

**Tests:** filter combination (AND/OR), empty==all-entitled, wildcard prefix match + illegal-wildcard rejection, out-of-entitlement space inert (no events, no error), malformed-filter rejection pre-stream; an observed event serialises to the §3 record shape with `received_at`.

## 6. Commit 4 — node command pipe (symmetric to C2) — ✅ SHIPPED (J-204)

**✅ SHIPPED (J-204, 2026-06-01).** New `xgen-node/src/aicontrol.rs` (sister to `pipe.rs`; `--batch` untouched, D-066). **Marshaling — Option A (locked, J-204):** the client's reconstruct-argv does **not** port — node admin verbs mix **positional** (required IDs: `space_id`, `peer_node_id`, …) + **flag** (`--reason`, `--state`, …) args. So the ~33 `*Args` structs derive `serde::Deserialize` (+ `#[serde(default)]` on the 5 Vec fields; `ReplicaAction` enum too, `rename_all="lowercase"`) and the arm marshals `serde_json::from_value::<XxxArgs>(args)` (wire key = Rust field name, AC-D1). **Additive + inert** to clap/CLI/`--batch` (no `default_value`/`ArgAction` to lose). A single `cmd`-string-keyed match deserializes + calls the SAME `admin_ops::*` `dispatch_admin` calls (no forked logic); missing required field → `BAD_ARGUMENT`, unmatched verb → `UNKNOWN_COMMAND`. `ActorVia::AiControl` set on the ctx; node verb errors → band code + `stage` + `category: protocol` (AC-D2 node). **No separate state-file lock** (C2-rider analog): the admin stores are `Arc<Mutex<…>>` and the **same** Arcs thread to both servers → cross-connection + cross-server mutations serialize on the stores' own mutexes. **§7.1 as-built:** the surface = the `admin_ops::*` verbs + `state`, **not** the 7 M2 print-only reads (`status`/`connections`/`peers`/`spaces`/`whoami`/`version`/`identity list` are `app::cmd_*` with no structured Result). **Node `state` as-built:** keeps `uptime_seconds`/`active_connections`/`registered_identities` (cheap from threaded deps) + `node_id`/`endpoint`/`auth_tiers_served`/`federated_peers`/`hosted_spaces`; **drops `operator_display_name`** (not in local config per `cmd_whoami`); `event_subscriptions` honest `0` (events deferred). **Asymmetry recorded:** client stays reconstruct-argv (all-flag Args), node uses serde (positional+flag) — the mechanism differs because the surfaces genuinely do. Verification: `cargo test --workspace` **898**/0/1 (+7); build all-targets 0/0; clippy `-D warnings` clean.

*(Original plan retained below.)*

The node `.aicontrol` command pipe + dispatch arm wrapping `admin_ops::*` (reusing `dispatch_admin`), built on the C1 substrate and the C2 pattern. Node `state` (AC-D3c node core — `lifecycle`/`node_id`/`operator_display_name`/`endpoint`/`auth_tiers_served` + store-derived `federated_peers`/`hosted_spaces` + control-owned `bindings`/`event_subscriptions`). **Audit attribution:** audited admin verbs invoked via `--aicontrol` tag `ActorVia::AiControl` (already in the enum — wire it through the arm). Node verb errors carry the band code + `stage` (AC-D2 node mapping). `--batch` (`pipe.rs` plain-text path) untouched. No checkpoint (symmetric to the locked C2 wiring).

**Tests:** a node admin verb round-trips with band code + stage on error; `state` node core; `ActorVia::AiControl` recorded on an audited write; reuse of `dispatch_admin` confirmed (no forked admin logic).

## 7. Commit 5 — node events pipe (symmetric to C3) — ⛔ DEFERRED (M7-events arc)

**⛔ DEFERRED with C3 (2026-06-01).** Paired with the client events pipe + the node multi-connection-per-identity enhancement; deferred to the M7-events arc. Original spec retained below as the arc's starting point.

The node `.events` pipe, reusing the C3 hook + filter engine, plus the **Node-only `nodes` filter** dimension (Client rejected it with `BAD_ARGUMENT` at C3; Node honors it). Node-side signals (e.g. `federation_request_pending`, §3). No checkpoint (symmetric).

**Tests:** `nodes` filter narrows the node event stream; the Client-side `BAD_ARGUMENT` for `nodes` (added at C3) still holds; a node signal serialises.

## 8. Commit 6 — close (D-074 atomic)

Doc-only. **Command-pipes-only close** (events pipe deferred per the §2 reshape). Same-commit atomic close: `docs/xgen_aicontrol_implementation.md` SHIPPED banners on the command-pipe sections (§4 protocol / §6 client verbs / §7 node verbs / §8 codes / §9 state / §10 timeout) + the **events sections (§3) marked DEFERRED → M7-events arc** + version bump (with honest as-built deltas per D-065 where code diverged from the v1.2 spec); ROADMAP (visual-tree M7 row ✅ command-pipes-only + Present arc-CLOSED + version bump); CLAUDE PLAY → next milestone; JOURNAL milestone-close entry; this runbook + `tasks/M7_AICONTROL_DESIGN.md` + `tasks/M7_AICONTROL_AUDIT.md` → COMPLETED. **Required canonical-doc notes (as-built, D-065):** (1) §8 — `CONCURRENT_COMMAND_NOT_ALLOWED` is a **wired safety-net that is structurally non-firing in v1's sequential per-connection handler** (the handler never reads the next line until the current reply is written → serial by construction; rejection reserved for a future pipelined model). (2) Marshaling asymmetry — the **client** arm uses reconstruct-argv (all-flag `ops::*` Args) while the **node** arm uses serde `from_value` on `Deserialize`-derived `admin_ops::*` Args (positional+flag mix); the mechanism differs because the surfaces do. (3) §7.1 — node `--aicontrol` surface = the `admin_ops::*` verbs + `state`, **not** the 7 M2 print-only `app::cmd_*` reads. (4) §9 — node `state` drops `operator_display_name` (not in local config); `event_subscriptions` is honest `0` on both binaries (events pipe deferred). DECISIONS.md: AC-D# are arc-local (D-069) — promote the `cmd` verb-exposure model to a global `D-###` here **only if** Joe calls it at close; otherwise no DECISIONS change.

## 9. Per-commit DoD + verification rigour

Each code commit (C1 · C2 · C4; C3/C5 deferred): `cargo test --workspace` green (report the count delta); `cargo build --workspace --all-targets` 0/0; `cargo clippy --workspace --lib --tests --all-features -- -D warnings` clean. C1 is package-scoped to wherever the substrate lands + its consumers. Explicit `git add <file>` per file; `git status` before commit; multi-`-m` commit messages; Joe pushes. No "commit pushed" in any DoD checklist (the `Status: COMPLETED` header / green verification is the signal).

## 10. Confirm-at-pickup (consolidated, D-078)

- ✅ **Substrate home** — resolved at C1: `xgen-common/src/aicontrol/` (J-201; no new crate).
- ⛔ **AC-D3b raw-prefix predicate** — DEFERRED with the events pipe (M7-events arc).
- ✅ **AC-D3c keep-or-drop fields** — client `connected_since`/`member_count`/`room_count` dropped at C2 (need resident-WS instrumentation); **node (C4, J-204): keep** `uptime_seconds`/`active_connections`/`registered_identities` (cheap from threaded deps), **drop** `operator_display_name` (not in local config per `cmd_whoami`).
- ✅ **§7.1 node read-verb names (C4, J-204)**: the 7 M2 reads are `app::cmd_*` print-only (no structured Result) → node `--aicontrol` surface = the `admin_ops::*` verbs + `state` (not those 7); `state` + the structured admin reads cover the ground.
- ✅ **Event-observation seam** — resolved at checkpoint #2 (J-203): absent → C3/C5 deferred to the M7-events arc (split-trigger b).

## 11. Discipline notes

- **Adapter, not feature (D-065).** No new business logic; no new verbs beyond the shipped `ops::*`/`admin_ops::*`. AC-D5's three verbs + `config-reload` stay out.
- **`--batch` untouched (D-066).** The legacy plain-text path is preserved verbatim; `--aicontrol` is a sister, never a replacement.
- **Reserved trio inert (AC-D4).** The `authorize` stage, `PERMISSION_DENIED`, and the per-connection token are dormant in v1; do not wire per-verb gating. Pipe-access == administrator (D-082), OS-ACL-delegated.
- **AC-D# arc-local (D-069).** No DECISIONS.md change during implementation unless Joe promotes the verb model at close.
- **`CONCURRENT_COMMAND_NOT_ALLOWED` is a wired safety-net, structurally non-firing in v1 (C2 as-built, J-202).** The sequential per-connection handler never reads the next line until the current reply is written → serial by construction; the in-flight guard + rejection code exist but the v1 loop cannot reach them. Reserved for a future pipelined handler. **C6 obligation:** record this in canonical-doc §8 (see §8 above).
- **Stop-and-surface (Rule 3).** Any split trigger, any business-logic temptation, any missing seam → stop and surface to Joe, do not work around.

## 12. Cross-references

- `tasks/M7_AICONTROL_DESIGN.md` (v1.0) — the AC-D1–AC-D6 locks this runbook implements.
- `tasks/M7_AICONTROL_AUDIT.md` (v1.0) — Phase-0 drift-reconciliation + the build-order recommendation.
- `docs/xgen_aicontrol_implementation.md` (v1.2) — canonical spec (§2 surface, §3 events, §4 protocol, §5 bindings, §8 codes, §9 state, §10 timeout).
- `xgen-client/src/batch.rs` (`pipe_name`/`dispatch_line`) + `xgen-client/src/ops.rs` (14 fns) — the client seam.
- `xgen-node/src/pipe.rs` (`pipe_name`/`dispatch_line`/`dispatch_admin`/`start_pipe_server`) + `xgen-node/src/admin_ops.rs` (`AdminError`/`Stage`/`ActorVia::AiControl`) — the node seam.
- DECISIONS.md: D-063 (library-first multi-dispatch), D-066 (`--batch`/`--aicontrol` split), D-067 (`ops::*`), D-069, D-074, D-078, D-082.

---

*Runbook ACTIVE v1.2. M7 v1 reshaped to command-pipes-only (C1 ✅ J-201 · C2 ✅ J-202 · C4 next · C6 close); C3/C5 + the node multi-connection enhancement deferred to the M7-events arc (checkpoint #2, J-203). Both checkpoints closed. Clair entry point: CLAUDE PLAY + the latest JOURNAL entry per Rule 0, then §6 (C4) here, then `tasks/M7_AICONTROL_DESIGN.md`.*
