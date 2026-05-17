# M4 — AI Client (resident mode of xgen-client)
> **Status**: PENDING  
> Version: 0.2 (v0.1 review-pass landed; architecture LOCKED on seven of seven sections; impl-level decisions still PROPOSED)  
> Date: May 2026  
> **Last updated**: 2026-05-17 (v0.2 incorporates Joe's review of v0.1)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

M3 (J-075) landed every protocol primitive an AI Identity needs to live inside a Space: `is_ai` registration, capability flags, operator role with fall-upward resolution, delegate/revoke handlers, AI-owned-Space rejection, and the Client CLI to drive all of it. What's missing is the **consumer** — a long-running process that registers as an AI, joins Spaces, receives events through a sustained WebSocket, and acts on them under the existing pacing and temperature constraints.

M4 is that consumer. It is **deliberately a reference implementation**, not a production AI. The deliverable is to prove the loop end-to-end — wire format, pacing, temperature visibility, mute handling, operator-on-record — using a trivial reference behaviour. Real LLM hookups, sophisticated dialog policies, multi-Space coordination, etc. layer on top of this binary as plugin implementations in later milestones.

The pattern: M4 establishes "the AI Client framework"; later milestones add "what the AI Client does" as plugins.

---

## Sequencing — M4 lands AFTER D-056 consolidation

**Locked at v0.1 review (Joe, 2026-05-17).**

D-056 (Application Deployment Model — DECISIONS.md:1921) names three implementation follow-on tasks pulling current code into compliance with the "one binary per role, multi-mode dispatch" target. They are tracked separately from the architectural decision itself; M4 is not implemented until they land:

| # | D-056 follow-on task | Status |
|---|---|---|
| 1 | **Node-side `--batch` implementation.** Port the BATCH_FLAG_ph2.md pattern to the Node side: same library-first rule, same pipe-naming convention, same clap dispatch shape, with the Node's own command set. | Open (J-075 baseline) |
| 2 | **Collapse `*-app.exe` into the single product binaries.** Merge `xgen-{node,client}/src/main.rs` with `xgen-{node,client}/src-tauri/src/main.rs` into one entry point per role; extract shared resident-mode logic into the library crate; eliminate the two parallel `--batch` implementations on the Client side. | M1 covered the dual-binary collapse + the parallel `--batch` dedup; `src-tauri/` leftover dirs noted as housekeeping in J-073. Joe to confirm at lock pass whether anything else remains here. |
| 3 | **Pipe server in resident mode for both binaries.** Every resident-mode invocation hosts the pipe server. | M2 (`app::run_node`) added the Node-side pipe server reaching both `--service` and the Tauri-desktop path; Client-side Tauri variant already had one pre-M1. Joe to confirm at lock pass whether item #3 is satisfied or whether a Client-side `--service`-only pipe-server wiring is still open. |

**Why this sequencing.** Adding the AI Client as a fourth binary with its own M2-style pipe server before consolidation completes gives consolidation more surface to chew through — and the right place to put the AI Client falls out of consolidation, not the other way around. Per section 1 below, M4 is **a mode of `xgen-client`**, not a separate binary; that decision rests on the post-consolidation shape. Designing M4 against the post-consolidation surface costs nothing extra at design time and avoids retrofit later.

This sequencing also means M4's task file is fully written and locked, but implementation does not begin until D-056's open items above are confirmed done. The architectural decisions in this file stand independent of that scheduling.

---

## Architectural foundation — LOCKED (per Joe, v0.1 review pass)

> All seven sections below were marked PROPOSED in v0.1 and went through Joe's review on 2026-05-17. The outcomes (LOCK / AMEND) are recorded inline. Implementation-level details (data shapes, function signatures, file locations) remain to be locked in a second pass — see "Implementation decisions" below.

### 1. Binary identity — LOCKED: `xgen-client --ai-mode` (resident sub-mode)

The AI Client is a **mode of `xgen-client`**, not a separate binary. Two binaries total: `xgen-node`, `xgen-client`. Three roles for `xgen-client`:

| Invocation | Role |
|---|---|
| `xgen-client <subcommand>` | One-shot human Client (existing) |
| `xgen-client --service` | Long-running human-Client resident (existing) |
| `xgen-client --ai-mode --service` | **NEW:** Long-running AI-Client resident |

**Why (Joe's reasoning, v0.1 review).** The Node's headless mode is `--service`, not a separate `xgen-node-service` binary. By symmetry, an AI Client is a client — same Identity registration, same Space membership, same event emission, same `[ai]` config staging — just with behaviour coming from a plugin instead of a keyboard. Consistency with the resident/control pattern wins. Yes, dispatch complexity grows, but adding a mode is cheaper than maintaining a parallel binary's lifecycle, config loading, and pipe server. Aligns with the D-056 consolidation direction (one binary per role) — picking a separate binary would put M4 in conflict with the architecture it should be following.

**Config implications.** Single config file `xgen-client_config.toml` (already in place; M3 added the `[ai]` section). Plugin section names defined in §5 below. Init flow stays `xgen-client init --ai [--cap k=v]` (existing M3 surface; no new init verb).

**Pipe-name implications.** Pipe naming follows existing convention `\\.\pipe\xgen-client[-<instance>]`. AI-mode resident binds to this same pipe — control commands (`--ping/--health/--stop/--reload-config`) work the same way; the resident's `__HEALTH__` reply tells the caller whether it's a human-mode or AI-mode resident.

**Dispatch shape.** `xgen-client/src/main.rs` gains an `--ai-mode` flag (Boolean, requires `--service`). When `--ai-mode --service` is detected, dispatch goes into `service::run_ai_service` (new entry point in `xgen-client-lib`) rather than `service::run_ws_loop`. The two functions share most of the connect/auth/keep-alive scaffolding; M4's job is to factor the shared parts cleanly into the library.

### 2. Plugin model — LOCKED: `AiBehavior` trait + one reference plugin shipped

The AI Client's *behaviour* (what it does on receipt of an event) is a **plugin trait**, not hard-coded logic. The binary ships **one** reference plugin: `echo-on-mention`.

**Why locked now (Joe's reasoning, v0.1 review).** Trait surface is small enough that getting it wrong now is cheap; getting it wrong after a real LLM plugin exists is expensive. Lock the shape during M4 so future plugin work consumes a stable interface.

```rust
// xgen-ai/src/behavior.rs (proposed shape)
pub trait AiBehavior: Send + Sync {
    /// Called when the AI receives an inbound Event. Return `None` for no
    /// response, or `Some(text)` to emit a `message.text` reply.
    fn on_event(&mut self, ctx: &EventContext) -> Option<String>;

    /// Called once at startup, after the AI is connected and joined to all
    /// known Spaces. Plugins may use this to initialise state.
    fn on_start(&mut self, ctx: &StartContext) {}

    /// Identifier surfaced in logs / status output. Static for the plugin's
    /// lifetime.
    fn name(&self) -> &'static str;
}
```

### 3. Reference behaviour — LOCKED with AMENDED reply text

The reference plugin:
- Watches `message.text` events.
- If the AI's `identity_id` substring (or its config-selected `mention_token`, default unset) appears in the event content, replies after waiting the Space's `ai_pacing_ms`.
- **Reply text MUST be deterministic and obviously artificial:** `[echo-plugin] received mention from <sender_id_short>`, where `<sender_id_short>` is the last 12 characters of the mentioning Identity's `identity_id`. No configurable reply text in M4 — the format is fixed for grep-ability in smoke tests and for unambiguity in early demos.
- Honours `active_mutes` from SpaceState — does not reply during cooldown.
- Logs every received event at INFO; replies at INFO with the outbound event_id.

**Why locked deterministic (Joe's reasoning, v0.1 review).** Smoke tests need to grep for the reply, and nobody should mistake the artefact for a real reply during early demos. Free-text replies are a future-plugin concern; the reference plugin's job is to be unmistakeably the reference.

That's the entire reference behaviour. It proves: WebSocket loop, registration as AI, event reception, pacing compliance, mute compliance, reply via `message.text`. **Nothing more.** Real LLM hookups become future plugins.

**Plugin loading.** Phase 1: static (the reference plugin is the only choice, compiled in). Phase 2+: dynamic / config-selected. M4 lands the trait + reference plugin; the loader is trivial (matches the configured plugin name to the only available impl) but the architecture is in place.

### 4. Lifecycle — LOCKED: long-running resident, M2-style pipe server

The AI Client runs as a **long-running resident** under `xgen-client --ai-mode --service`. There is no "one-shot AI command". Lifecycle:

- `xgen-client init --ai [--cap dm_initiate=true ...]` (existing M3 surface). Generates keypair, writes config with the `[ai]` section.
- `xgen-client register --name "Bot Name"` (existing M3 surface). Registers on the Node with `is_ai=true` and the capability map.
- `xgen-client --ai-mode --service` — the resident. Starts a sustained WebSocket to the home Node, runs the auth handshake, enters the event loop. Reuses the existing service-mode scaffold.
- `xgen-client --ping` / `--health` / `--stop` / `--reload-config` — existing pipe-server control commands. The AI-mode resident binds to the same pipe (`\\.\pipe\xgen-client` / `\\.\pipe\xgen-client-<instance>`); `__HEALTH__` reply identifies the resident as AI-mode.

**No Tauri / no systray / no UI.** Headless by design. If a future milestone wants an admin UI, it's a separate deliverable.

**Why locked (Joe, v0.1 review).** Mirrors M2 cleanly. The control-surface verbs are already in place from M2; AI-mode reuses them without adding new control commands.

### 5. Configuration — LOCKED with AMENDED layout: split "which plugin" from "plugin config"

The AI Client reads `xgen-client_config.toml` (same file as the human Client; one binary, one config file shape). It uses the existing `[client]/[paths]/[logging]/[ai]` sections from M3.

**Plugin selection lives in `[ai]`; plugin-specific config keys live in `[ai.behavior]`:**

```toml
[client]
node = "ws://127.0.0.1:8080/xgen"

[paths]
keypair_path = "..."

[logging]
level = "info"

[ai]
is_ai = true
plugin = "echo"                # ← which plugin to load

[ai.capabilities]
dm_initiate = false
spontaneous_post = false

[ai.behavior]                  # ← plugin's own config (echo's, in this case)
mention_token = "@bob"          # ← optional; default unset, identity_id substring only
```

**Why split (Joe's reasoning, v0.1 review).** Keeps the two concerns separable. When a second plugin exists, `plugin = "..."` is a single-line toggle; the per-plugin config in `[ai.behavior]` swaps in tandem but lives in its own namespace. Mixing both into a single sub-table (the v0.1 proposal) makes plugin selection harder to script and visually conflates "what plugin" with "how that plugin is tuned."

**Open-enum on plugin name.** Unknown values are tolerated by config parsing; runtime plugin loader rejects them with a clear error at startup. M4 only ships `"echo"`; future plugins add their own names.

**`[ai.behavior]` schema.** Each plugin documents which keys it consumes from `[ai.behavior]`. The echo plugin's keys for M4: just `mention_token` (Option<String>, default `None`). Unknown keys in `[ai.behavior]` are tolerated (forward compat).

### 6. Pacing — LOCKED: respect `ai_pacing_ms`, drop late replies

Per D-060, each Space carries `ai_pacing_ms` (default 2000ms) — minimum interval between consecutive AI Events. M4's AI Client respects it:

- The runtime maintains a per-Space `last_send_at` timestamp.
- Before sending a reply, check `now - last_send_at >= ai_pacing_ms`. If not, **drop the reply** (do not queue). The plugin produced a reply, but pacing rejected it.
- Log dropped replies at WARN with reason.

**Why drop instead of queue — locked at v0.1 review.** Queueing produces stale replies. By the time the cooldown expires, the conversation has moved on; the queued reply now misrepresents the AI's *current* state rather than reflecting it. Dropping is honest: "I had something to say at the moment, but you set a rate limit; I respected it and the moment passed."

**Recurring principle — "honest behaviour over polite behaviour" (Joe, v0.1 review).** This is an instance of a broader XGen design principle worth naming: when a system has the choice between behaviour that *misrepresents its current state* (polite — "I'll keep your thought for you and deliver it eventually") and behaviour that *honestly reflects its current state* (honest — "I can't say this right now and the moment is gone"), XGen picks the honest option. The same logic shows up elsewhere in the protocol — the fall-upward operator resolution returns the *currently-resolvable* operator rather than a stale stored value; the Node drops events it can't validate rather than queueing them indefinitely; mute is a wall, not a delay. M4's pacing-drop is a clean instance worth a journal note when the implementation lands.

The existing `PacingManager` in `xgen-client/src/pacing.rs` already implements the wait/drop logic per Ch6 §6.14.2. M4 reuses it.

### 7. Temperature — LOCKED out of scope for M4

Per D-061, Spaces carry `xgen.room_temperature` / `xgen.member_temperature` meta_atts surfaced by a plugin on the Node side; the math is plugin-owned. The Client receives `temperature.update` and displays the value.

M4 does **not** participate in temperature beyond what's implicit (the AI Client receives the meta_atts like any client). It does not surface its own temperature, does not react to room temperature thresholds. Reason: temperature is conversational-dynamics work that needs careful design; the M4 deliverable is "the loop works", not "the AI dances with temperature."

Auto-mute via `auto_temperature` reason (3.7.13.6) is already enforced Node-side; M4's AI Client respects `active_mutes` regardless of reason, so no extra work needed for compliance.

### 8. Operator control plane — LOCKED out of scope for M4

M3 records who the operator is and provides the resolution function. M4 does not surface "the operator can instruct the AI" semantics. Reasons:

- No protocol-level operator-signed events exist (per the M3 architecture lock). Designing M4 around them loads weight on something unbuilt.
- The operator command surface is its own protocol-level design conversation (DM commands? Special EventTypes? Out-of-band IPC?) — should not be pre-empted by M4's reference plugin.
- The reference plugin (echo-on-mention) has no controllable parameters that would benefit from operator instruction; future plugins will.

**Locked at v0.1 review.**

What M4 DOES expose: the resident's `__HEALTH__` reply includes a coarse `operator_known=true|false` signal; `xgen-client status` (offline-local, existing M3 surface — but with the resident running, the state file is fresh) prints the resolved operator for the AI in each Space it's a member of. Informational only.

---

## Cross-references

| Source | Relevance |
|---|---|
| `tasks/M3_AI_OPERATOR_ROLE.md` (COMPLETED) | The protocol primitives M4 consumes. |
| `JOURNAL.md` J-075 | M3 close-out — full surface map. |
| `DECISIONS.md` D-059 | AI Identity + capabilities. M4 honours the wire shapes. |
| `DECISIONS.md` D-060 | Pacing. M4's runtime respects `ai_pacing_ms`. |
| `DECISIONS.md` D-061 | Temperature. M4 is a passive recipient (no surfacing). |
| `DECISIONS.md` D-062 | Tauri-in-binary model. **Does not apply** — M4 is headless. |
| `DECISIONS.md` D-063 | Library-first dispatch. M4 follows this — runtime in `xgen-client-lib`, binary is a thin shell. |
| `xgen-client/src/pacing.rs` | Existing `PacingManager` — M4 reuses. |
| `xgen-client-lib::service::run_ws_loop` | The natural attachment point for the AI Client runtime — long-running WS loop already exists for `xgen-client --service`. M4 forks or extends it. |
| `xgen-node/src/pipe.rs` | M2's Node pipe server. M4's pipe server mirrors its shape. |

---

## Scope

### In scope for M4

1. **`xgen-ai` binary crate** — thin `main.rs` + small AI-runtime module. Library-first per D-063: all real logic in `xgen-client-lib` (extended for the AI runtime loop where the existing one differs) or a new `xgen-ai-lib` if separation is cleaner.
2. **`AiBehavior` trait** + the reference `echo-on-mention` plugin. Trait lives in `xgen-client-lib` (or `xgen-ai-lib`); reference plugin is the only impl shipped.
3. **AI Client runtime loop** — sustained WS to home Node, auth handshake, optional `transport.sync_request` to catch up missed events on startup, event-receive loop, plugin invocation per event, reply emission honouring pacing and mute.
4. **`xgen-ai init` / `register` / `--service`** commands + the M2-style pipe server (`--ping` / `--health` / `--stop` / `--reload-config`).
5. **`xgen-ai status`** (offline-local, like xgen-client status) printing the local Identity, joined Spaces, the resolved operator per Space, and last-N reply timestamps.
6. **Unit tests** for the runtime loop (event in → plugin call → reply out with pacing) using mock connections.
7. **Single-Node end-to-end smoke** — alice (human admin) creates a Space, invites bob (AI Client running on the same machine), bob auto-joins, alice sends `@bob hello`, bob replies after `ai_pacing_ms`, alice sees the reply.
8. **`docs/xgen_ch6_client_design.md`** gets a new section on the AI Client architecture (or a new appendix — Joe's call at lock time).
9. **`DECISIONS.md`** entry capturing the M4 architecture once Joe locks it.

### Out of scope (deferred)

- **Real LLM hookup** — future plugin. M4 ships one trivial reference plugin to prove the loop.
- **Multiple plugins / plugin selection at runtime** — Phase 2+ for the AI Client.
- **Operator command surface** — separate protocol-level design.
- **Temperature surfacing / room-temperature reaction** — design unclear; defer.
- **Auto-join of Spaces by invite** — Phase 2 if needed. M4: operator manually joins via `xgen-client join` against the AI's keypair? Or AI auto-joins all `pending_invites` on startup? **Decision needed at lock time** — proposing the latter (auto-join all pending) for testability.
- **Cross-Space coordination** — out of scope.
- **Multi-device AI Client** — single device per AI Identity in M4.
- **Tauri / UI surface** — explicitly not in M4.
- **DPI resistance, alternative transports** — Phase 3 across the board.

---

## Phase 0 — Pre-flight inventory (REQUIRED before any code)

1. **Capture baseline.** `cargo test --workspace --release` — confirm **411**. Quote actual output.
2. **Read `xgen-client-lib::service::run_ws_loop` end-to-end.** Map what the human Client's `--service` resident does on inbound events today. Today it drops them (per J-072 / J-075). M4 needs to consume them. Decide whether to fork the function or extend it.
3. **Read `xgen-client/src/pacing.rs::PacingManager`.** Confirm the wait/drop API matches M4's needs. Note any edge cases (clock skew, missing `is_ai`, missing rules, cap-of-zero) already handled.
4. **Read `xgen-node/src/pipe.rs`** and `xgen-client/src/batch.rs::start_pipe_server`. M4's pipe server mirrors one of these (proposing the Node's shape since it's newer / cleaner).
5. **Inspect xgen-client-lib for AI-mode hooks.** Anything pre-staged in M3 that M4 should extend, vs. add fresh? Specifically: how does `xgen-client register` know to send `is_ai=true`? That code path is M4's template for the AI binary.
6. **Confirm `xgen-client/src/main.rs` dispatch.** M4 adds `xgen-ai/src/main.rs` modelled on it; understanding the existing flow before duplicating saves friction.

Phase 0 produces a short findings note in the journal entry — not a separate document.

---

## Implementation decisions — PROPOSED (awaiting Joe's second lock pass)

The architectural foundation above answers the *what*. These are the *how* details, to be settled in a second review pass before implementation begins. v0.1's auto-join proposal (#4) is amended pre-lock per Joe's signal at v0.1 review — see below.

1. **Library home.** AI runtime lives in `xgen-client-lib::ai_service` (new module alongside the existing `service`); shared scaffold (connect, auth, keep-alive) extracted into a common helper consumed by both `run_ws_loop` and `run_ai_service`. **No new crate.** Dependencies are identical to existing `xgen-client-lib`; a new crate boundary adds friction without value.
2. **`AiBehavior` trait location.** `xgen-client-lib::ai_behavior` module. Public so future plugin crates can implement it.
3. **Reference plugin name.** `EchoPlugin` (struct), config key `"echo"`. (Plugin name is the short form used in the config `plugin = "echo"`; struct name carries the longer descriptor in code.)
4. **Join behaviour — PRE-LOCKED MANUAL per Joe (v0.1 review).** The AI Client does **not** auto-join Spaces on startup. The operator drives the join via the existing `xgen-client --instance <ai-label> join --space <id>` (one-shot, runs against the same keypair the resident uses — both processes happen to share the keypair file; the resident reloads `pending_invites` from store on next event). **Why:** auto-join would make an AI Identity's first observable behavior in a Space config-driven rather than chosen, muddying the trust model. Manual join keeps presence as an explicit, auditable event in the DAG. Testing convenience is real but solvable with a one-line CLI helper in the smoke script.
5. **Reply event prev_events.** Same `get_dag_tips`-based discovery as `cmd_send`. No special path for AI.
6. **Mention detection.** Simple substring check on `content.text` for the AI's `identity_id` (full URI) — and a config-selectable nickname token from `[ai.behavior] mention_token = "@bob"`. Default: identity_id substring only.
7. **Operator surfacing on `__HEALTH__`.** Pipe reply for an AI-mode resident includes `mode=ai operator_known=true|false` after the standard fields. The structured per-Space operator map is available via `xgen-client status` (offline-local, existing) when the resident's state file is fresh.

---

## Implementation steps (recommended sequence)

### Phase 1 — `--ai-mode` flag + dispatch

1. Add `--ai-mode` flag to `xgen-client/src/app.rs::Cli`. Requires `--service` (clap validation; error if either is used without the other in a way that's nonsensical — to be decided at impl time).
2. `xgen-client/src/main.rs` dispatch: when `--ai-mode --service` is detected, call `xgen_client_lib::ai_service::run_ai_service` instead of `service::run_ws_loop`. All other invocations preserve their existing behavior.
3. `cargo build --release --workspace` clean.

### Phase 2 — `AiBehavior` trait + reference plugin

4. Trait in `xgen-client-lib::ai_behavior`. Public so future plugins can implement it.
5. `EchoPlugin` (config key `"echo"`) in `xgen-client-lib::ai_behavior::echo`.
6. Unit tests for the plugin's `on_event` decision-making (mention detected via identity_id, mention detected via `mention_token`, no mention, mute active).

### Phase 3 — AI runtime loop

7. New `xgen-client-lib::ai_service::run_ai_service` factored out of (or alongside) `service::run_ws_loop`. Shared scaffold extracted to a common helper; the two services differ only in what they do on inbound Events. AI-service loop:
   - Loads `[ai] plugin = "..."` + instantiates the named plugin.
   - On inbound `Event`, calls `plugin.on_event(ctx)`. If `Some(reply_text)`, runs through pacing + mute checks, then sends a `message.text` to the originating Room.
   - Maintains per-Space `last_send_at` for pacing.
   - Drops late replies (does not queue).
8. **Manual join (no auto-join).** The runtime makes no `membership.join` decisions on its own — joins are operator-driven via `xgen-client --instance <ai-label> join --space <id>` (existing CLI). The resident reloads its SpaceState on subsequent events.

### Phase 4 — Pipe server + observability

9. Existing pipe server in resident mode handles `__PING__` / `__HEALTH__` / `__STOP__` / `__RELOAD_CONFIG__`. M4 extends `__HEALTH__` reply with `mode=ai` and `operator_known=true|false` when the resident is in AI mode.
10. `xgen-client status` (existing M3 surface) already prints local Identity + joined Spaces. M4 adds a section showing resolved operator per Space when the local Identity is an AI (read from xgen-client_state.json which the resident keeps fresh).

### Phase 5 — Single-Node end-to-end smoke

11. Smoke script: start `xgen-node --instance m4-smoke --service`. Init + register alice (human) and bob (AI with `plugin = "echo"`). Alice creates Space, invites bob, runs `xgen-client --instance m4-bob join --space <id>` (manual join, per impl decision #4). Start `xgen-client --instance m4-bob --ai-mode --service` in background. Alice sends `message.text` containing bob's identity_id substring. Verify bob's reply (`[echo-plugin] received mention from <hash>`) lands on alice's side within `ai_pacing_ms + jitter`. Transcript quoted in journal.

### Phase 6 — Spec + DECISIONS

12. `docs/xgen_ch6_client_design.md` new section "AI Client architecture" — runtime model, plugin trait, lifecycle, pacing/mute contract. Or a new appendix per Joe's call at lock time.
13. `DECISIONS.md` D-065 (or next available) capturing locked M4 architecture. Include the "honest behaviour over polite behaviour" principle note from §6.

---

## Definition of Done

- [ ] Phase 0 baseline captured (`cargo test` quoted in journal).
- [ ] Phase 0 inventory done; findings folded into the journal entry.
- [ ] `--ai-mode` flag added to `xgen-client` CLI; dispatch routes `--ai-mode --service` to `run_ai_service`.
- [ ] `AiBehavior` trait + `EchoPlugin` reference plugin implemented with unit tests covering: mention via identity_id, mention via `mention_token`, no mention, mute active.
- [ ] AI runtime loop (`run_ai_service`) implemented: sustained WS, plugin invocation per inbound event, reply emission under pacing + mute, drops late replies.
- [ ] Manual join model preserved (no auto-join logic in the runtime).
- [ ] Existing pipe server's `__HEALTH__` reply extended with `mode=ai operator_known=…` for AI-mode residents.
- [ ] `xgen-client status` surfaces resolved operator per Space for AI Identities.
- [ ] `cargo build --release --workspace` clean (no new warnings beyond M3's baseline).
- [ ] `cargo test --workspace --release` green at the new total (expected ~430–450).
- [ ] Single-Node end-to-end smoke runs green; transcript quoted in journal. Smoke uses the deterministic reply text from §3.
- [ ] `docs/xgen_ch6_client_design.md` (or new appendix) AI Client section landed.
- [ ] `DECISIONS.md` D-065 added (includes the "honest behaviour over polite behaviour" principle note).
- [ ] `JOURNAL.md` entry written (J-076 if M4 lands in one session) quoting actual verification output.
- [ ] `tasks/M4_AI_CLIENT_BINARY.md` header flipped from `PENDING` to `COMPLETED`.
- [ ] `CLAUDE.md` updated; next session entry point reset.

---

## What's still open

The eight architectural sections above are all LOCKED as of v0.2 (Joe's review pass of v0.1). The remaining open items for the second review pass:

- **Seven implementation-level decisions** (above) — including impl #4 already pre-locked to **manual join** at v0.1 review. The other six need confirmation.
- **D-056 consolidation status** — confirm at second-pass review which of the three follow-on tasks remain open and need to land before M4 implementation can begin. Specifically, whether M2 satisfied item #3 ("pipe server in resident mode for both binaries") and whether anything besides housekeeping remains on item #2 after M1.
- **Spec home for the AI Client architecture** — `xgen_ch6_client_design.md` new section vs. a new appendix? Either is defensible; Joe's call.

---

## Behaviour rules reminder (from CLAUDE.md)

- **Rule 1** — Never fabricate results. Real output only.
- **Rule 2** — Show actual output. Quote terminal output verbatim in the journal.
- **Rule 3** — Stop and report when a tool fails.
- **Rule 4** — Write the journal entry last, after verification is confirmed.
- **Rule 5** — Never invent numbers. Test counts from `cargo test` only.
- **Rule 6** — When in doubt, do less and ask. The architecture above is PROPOSED until Joe locks it; do not silently extend.
- **Rule 7** — Definition of Done is a checklist, not a formality.

If anything in Phase 0 inventory contradicts the locked architecture (e.g. `run_ws_loop` is too entangled with the human Client to extend cleanly), **stop and surface it** rather than redesigning unilaterally.

---

*End of M4 task file v0.2. Architecture locked. Awaiting impl-level second review pass + D-056 consolidation completion before implementation.*
