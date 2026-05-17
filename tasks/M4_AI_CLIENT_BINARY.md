# M4 — AI Client Binary (reference implementation)
> **Status**: PENDING  
> Version: 0.1 (draft for Joe's review — architecture PROPOSED, not LOCKED)  
> Date: May 2026  
> **Last updated**: 2026-05-17 (drafted at M3 close-out — J-075 sequel)  
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

## Architectural foundation — PROPOSED (awaiting Joe's lock pass)

> All seven sections below are open for re-design at task-file review time. Once Joe runs the lock pass (separate session), they become non-negotiable for the implementation pass. M3 followed this exact rhythm.

### 1. Binary identity — PROPOSED: separate `xgen-ai` binary

Three binaries: `xgen-node`, `xgen-client`, **`xgen-ai`**. The AI Client gets its own thin shell on top of `xgen-client-lib` (which already carries most of the connect/auth/sync surface). The library crate stays canonical; the new binary is a few hundred lines of dispatch + the AI runtime loop.

**Why not `xgen-client --ai-mode` (or new subcommand)?**
- xgen-client's run modes (default desktop, `--service`, `--batch`, individual subcommands) already require careful dispatch in `main.rs`. Adding an AI-flavoured resident mode grows that complexity for the wrong reason — the AI loop is **not** a flavour of the human Client, it's a different program that happens to reuse the same protocol library.
- The configuration surfaces differ. Human Client uses `[client]/[paths]/[logging]` + optional `[ai]` (added in M3 for the AI-staged human-driven case). AI Client adds `[ai.behavior]` + plugin config; mixing those into the same dispatcher means a single config file describes either a human client or an AI client depending on which mode the binary is launched in. Two binaries, two clean config shapes.
- M1 collapsed four binaries to two because the displaced binaries shared **identical** code (just stub vs real). xgen-client and the AI Client share only their library dependency — they do different things at runtime.

**Why not a separate crate?**
- The new binary depends on `xgen-client-lib` for everything WebSocket/auth/identity-related. A separate crate would either duplicate that or pull in xgen-client-lib as a dependency anyway. A new binary crate `xgen-ai/` mirroring `xgen-client/` (thin main + small AI-runtime module) is cheaper than a fresh middle layer.

**Build implications.**
- `xgen-ai.exe` produced by `cargo build --release` and copied to `bin/` by `build.sh`.
- BSL 1.1 license header — same as xgen-client/xgen-node, transitioning to GPL on handover. Library crate remains GPL.

### 2. Reference behaviour — PROPOSED: plugin model, ship one trivial reference plugin

The AI Client's *behaviour* (what it does on receipt of an event) is a **plugin trait**, not hard-coded logic. The binary ships **one** reference plugin: `echo-on-mention`.

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

The reference plugin:
- Watches `message.text` events.
- If the AI's `identity_id` (or its `xgen.nick` if surfaced) appears in the event content, replies with a canned line ("Bot here — I see you mentioned me.") after waiting the Space's `ai_pacing_ms`.
- Honours `active_mutes` from SpaceState — does not reply during cooldown.
- Logs every received event at INFO; replies at INFO with the outbound event_id.

That's the entire reference behaviour. It proves: WebSocket loop, registration as AI, event reception, pacing compliance, mute compliance, reply via `message.text`. **Nothing more.** Real LLM hookups become future plugins.

**Plugin loading.** Phase 1: static (the reference plugin is the only choice, compiled in). Phase 2+: dynamic / config-selected. M4 lands the trait + reference plugin; the loader is trivial (always returns the reference plugin) but the architecture is in place.

### 3. Lifecycle — PROPOSED: long-running daemon, M2-style pipe server

The AI Client runs as a **long-running daemon**. There is no "one-shot AI command". Lifecycle:

- `xgen-ai init [--passphrase=…]` — same shape as `xgen-client init`. Generates keypair, writes config, marks `[ai] is_ai = true` automatically.
- `xgen-ai register --name "Bot Name"` — one-shot, registers on the Node with `is_ai=true` and the M3 capability map. Identical to `xgen-client register` when the Client config has `[ai]`.
- `xgen-ai --service` — the resident. Starts a sustained WebSocket to the home Node, runs the auth handshake, optionally joins Spaces the AI is invited to (or already a member of via prior sessions), enters the event loop.
- `xgen-ai --ping` / `--health` / `--stop` / `--reload-config` — pipe-server control commands, same pattern as M2's Node pipe server (`\\.\pipe\xgen-ai` / `\\.\pipe\xgen-ai-<label>`).

**No Tauri / no systray / no UI.** Headless by design. If a future milestone wants an admin UI, it's a separate deliverable.

### 4. Configuration — PROPOSED: extend the existing `xgen-client_config.toml` schema

The AI Client reads `xgen-ai_config.toml` (same shape as `xgen-client_config.toml`, just file-named for the AI binary). It uses the existing `[client]/[paths]/[logging]/[ai]` sections from M3, plus a new `[ai.behavior]` section:

```toml
[client]
node = "ws://127.0.0.1:8080/xgen"

[paths]
keypair_path = "..."

[logging]
level = "info"

[ai]
is_ai = true

[ai.capabilities]
dm_initiate = false
spontaneous_post = false

[ai.behavior]
plugin = "echo-on-mention"
# Plugin-specific keys live in sub-tables, e.g.:

[ai.behavior.echo-on-mention]
reply_text = "Bot here — I see you mentioned me."
```

Open-enum on `[ai.behavior.<plugin-name>]` — each plugin owns its config sub-table; unknown keys are tolerated.

**Why not a separate file?** Two reasons: (a) the AI is configured by the same operator who configured the Identity (no need for separate ownership), (b) splitting state across files creates "where does this go?" friction during operation.

### 5. Pacing — PROPOSED: respect `ai_pacing_ms`, drop late replies

Per D-060, each Space carries `ai_pacing_ms` (default 2000ms) — minimum interval between consecutive AI Events. M4's AI Client respects it:

- The runtime maintains a per-Space `last_send_at` timestamp.
- Before sending a reply, check `now - last_send_at >= ai_pacing_ms`. If not, **drop the reply** (do not queue). The plugin produced a reply, but pacing rejected it.
- Log dropped replies at WARN with reason.

**Why drop instead of queue?** Queueing produces stale replies — by the time the cooldown expires, the conversation has moved on. Dropping is honest: "I had something to say but you set a rate limit; I respected it." A queued-with-staleness-check model is more complex than M4 needs. Future milestones may add it.

The existing `PacingManager` in `xgen-client/src/pacing.rs` already implements the wait/drop logic per Ch6 §6.14.2. M4 reuses it.

### 6. Temperature — PROPOSED: out of scope for M4

Per D-061, Spaces carry `xgen.room_temperature` / `xgen.member_temperature` meta_atts surfaced by a plugin on the Node side; the math is plugin-owned. The Client receives `temperature.update` and displays the value.

M4 does **not** participate in temperature beyond what's implicit (the AI Client receives the meta_atts like any client). It does not surface its own temperature, does not react to room temperature thresholds. Reason: temperature is conversational-dynamics work that needs careful design; the M4 deliverable is "the loop works", not "the AI dances with temperature."

Auto-mute via `auto_temperature` reason (3.7.13.6) is already enforced Node-side; M4's AI Client respects `active_mutes` regardless of reason, so no extra work needed for compliance.

### 7. Operator control plane — PROPOSED: out of scope for M4

M3 records who the operator is and provides the resolution function. M4 does not surface "the operator can instruct the AI" semantics yet. Reasons:

- No protocol-level operator-signed events exist (per the M3 architecture lock).
- The operator command surface is its own protocol-level design conversation (DM commands? Special EventTypes? Out-of-band IPC?) — should not be pre-empted by M4's reference plugin.
- The reference plugin (echo-on-mention) has no controllable parameters that would benefit from operator instruction; future plugins will.

What M4 DOES expose: `xgen-ai status` (or pipe-server `__HEALTH__`) prints the resolved operator for the AI in each Space it's a member of — informational only.

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

## Implementation decisions — TO BE LOCKED (at task-file review time, before Phase 1)

The architectural foundation above answers the *what*. These are the *how* details, to be settled at task-file review before implementation begins.

1. **Library home.** New crate `xgen-ai-lib` mirroring `xgen-client-lib`, OR put the AI runtime in `xgen-client-lib::ai_service`? **Proposing the latter** — avoids new crate boundary; the dependencies are identical.
2. **`AiBehavior` trait location.** `xgen-client-lib::ai_behavior` module. Public so future plugin crates can implement it.
3. **Reference plugin name.** `EchoOnMention` (struct), config key `"echo-on-mention"`.
4. **Auto-join behaviour.** On startup, the AI scans its `xgen-ai_state.json` for known Spaces with `pending_invites` containing its `identity_id`, and signs `membership.join` for each. **Proposing this** (testability win) but Joe to confirm.
5. **Reply event prev_events.** Same `get_dag_tips`-based discovery as `cmd_send`. No special path for AI.
6. **Mention detection.** Simple substring check on `content.text` for the AI's `identity_id` (full URI) — and a config-selectable nickname token from `[ai.behavior.echo-on-mention] mention_token = "@bob"`. Default: identity_id substring only.
7. **Operator surfacing on `--health`.** Pipe `__HEALTH__` returns one-line summary including `operator_known=true|false` (a coarse signal — the structured per-Space operator map is on `xgen-ai status`).

---

## Implementation steps (recommended sequence)

### Phase 1 — Crate scaffold + binary skeleton

1. Create `xgen-ai/` crate with `Cargo.toml`, `src/main.rs`, BSL header. Add to workspace.
2. `xgen-ai/src/main.rs` — clap-based CLI with `init`, `register`, and pipe-server control flags (`--service`, `--ping`, `--health`, `--stop`, `--reload-config`, `--instance`). Mirror `xgen-client/src/main.rs` shape.
3. `build.sh` copies `xgen-ai.exe` alongside the others.
4. `cargo build --release --workspace` clean.

### Phase 2 — AiBehavior trait + reference plugin

5. Trait in `xgen-client-lib::ai_behavior` (or wherever lock decision #2 lands).
6. `EchoOnMention` plugin in `xgen-client-lib::ai_behavior::echo_on_mention`.
7. Unit tests for the plugin's `on_event` decision-making (mention detected, no mention, mute active).

### Phase 3 — AI runtime loop

8. New `xgen-client-lib::ai_service::run_ai_service` modelled on `service::run_ws_loop`. Differences:
   - Loads `[ai.behavior]` config + instantiates the plugin.
   - On inbound `Event`, calls `plugin.on_event(ctx)`. If Some(reply_text), runs through pacing+mute check, then sends.
   - Maintains per-Space `last_send_at` for pacing.
9. Auto-join logic (per lock decision #4): on connect, scan known Spaces, sign `membership.join` for any pending-invite where the AI is the target.

### Phase 4 — Pipe server + observability commands

10. AI pipe server `\\.\pipe\xgen-ai[-<label>]` mirroring `xgen-node/src/pipe.rs`. Four control commands.
11. `__HEALTH__` returns AI-specific summary including connected duration, spaces joined, replies sent, mute state, operator known per Space (boolean).
12. `xgen-ai status` (offline-local) prints local Identity + joined Spaces + resolved operator per Space + last-N reply timestamps from `xgen-ai_state.json`.

### Phase 5 — Single-Node end-to-end smoke

13. Smoke script (`tasks/M4_smoke.sh` or inline): start xgen-node, init+register alice (human) and bob (AI), alice creates Space, invites bob, alice manually starts `xgen-ai --service` for bob's instance, bob auto-joins, alice sends `@bob test`, bob replies after `ai_pacing_ms`, verify reply landed.

### Phase 6 — Spec + DECISIONS

14. `docs/xgen_ch6_client_design.md` new section "AI Client architecture" — runtime model, plugin trait, lifecycle, pacing/mute contract. Or a new appendix per Joe's call.
15. `DECISIONS.md` D-065 (or next available) capturing locked M4 architecture.

---

## Definition of Done

- [ ] Phase 0 baseline captured (`cargo test` quoted in journal).
- [ ] Phase 0 inventory done; findings folded into the journal entry.
- [ ] `xgen-ai` binary crate exists in the workspace, builds clean, copied by `build.sh`.
- [ ] `AiBehavior` trait + `EchoOnMention` reference plugin implemented with unit tests.
- [ ] AI runtime loop (`run_ai_service`) implemented: sustained WS, plugin invocation per inbound event, reply emission under pacing + mute.
- [ ] Auto-join behaviour live (per lock decision #4).
- [ ] AI pipe server live; `--ping` / `--health` / `--stop` / `--reload-config` all real.
- [ ] `xgen-ai status` surfaces local Identity, joined Spaces, resolved operator per Space.
- [ ] `cargo build --release --workspace` clean (no new warnings beyond M3's baseline).
- [ ] `cargo test --workspace --release` green at the new total (expected ~430–450).
- [ ] Single-Node end-to-end smoke runs green; transcript quoted in journal.
- [ ] `docs/xgen_ch6_client_design.md` (or new appendix) AI Client section landed.
- [ ] `DECISIONS.md` D-065 added.
- [ ] `JOURNAL.md` entry written (J-076 if M4 lands in one session) quoting actual verification output.
- [ ] `tasks/M4_AI_CLIENT_BINARY.md` header flipped from `PENDING` to `COMPLETED`.
- [ ] `CLAUDE.md` updated; next session entry point reset.

---

## What Joe needs to decide at lock time

The seven architectural sections above are PROPOSED. Joe's lock pass settles each as LOCKED or AMENDED. Specific calls to make:

1. **Binary identity** — separate `xgen-ai`, OR `xgen-client --ai-mode`, OR `xgen-client ai-resident` subcommand? (Proposed: separate binary.)
2. **Plugin model** — confirm trait-based, ship one reference plugin. Or hardcode the reference behaviour and add plugin trait later? (Proposed: trait now.)
3. **Reference plugin behaviour** — confirm "echo-on-mention with canned text". Want something more interesting (e.g. tiny scripted dialog)? (Proposed: trivial echo.)
4. **Auto-join on startup** — confirm AI auto-joins all `pending_invites` where it's the target. Or require manual join via the operator? (Proposed: auto-join.)
5. **Pacing on full** — drop late replies, or queue with staleness check? (Proposed: drop.)
6. **Temperature participation** — confirm out of scope for M4. (Proposed: out.)
7. **Operator control plane** — confirm out of scope. (Proposed: out.)

Plus the seven implementation-level decisions listed above for locking-in-pass-two (same rhythm as M3).

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

*End of M4 task file draft. Ready for Joe's review and lock pass.*
