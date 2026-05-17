# M5 — `ops::*` refactor (single source of truth for Client command implementations)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-17 (closed by J-078 — 12 atomic commits, 435 tests, 17/17 smoke PASS)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

`xgen-client` today has two parallel implementations of every batch-compatible command. The CLI dispatcher (`xgen-client/src/main.rs`) defines a `cmd_*` set; the pipe dispatcher (`xgen-client/src/batch.rs`) defines a separate `exec_*` set. The two sets do similar work but are different code that has drifted multiple times — J-067 documented F-003 and F-004 as two manifestations of the same `get_dag_tips` bug class, where a fix landed on `exec_*::get_dag_tips` and the *other* `get_dag_tips` (in the `cmd_*` path) silently kept the bug.

D-063 (library-first architecture rule) moved dispatch out of `main.rs` into the library. M5 extends that principle one level deeper: command *implementations* move from per-dispatcher copies into a single shared layer `xgen-client-lib::ops::*` that every dispatcher (CLI, pipe, Tauri UI, and future `--aicontrol`) calls into. The drift surface that produced F-003/F-004 becomes architecturally impossible — there is one implementation; nobody can fix one copy and miss the other.

M5 is a **pure refactor**. No new behaviour, no new commands, no new wire shape. Pre- and post-condition both build the same product; the difference is structural.

M5 is also a **prerequisite for `--aicontrol` v1 (D-066)**. The `--aicontrol` surface needs a shared command layer to dispatch through; designing `--aicontrol` against the current two-set duplicate would either inherit the drift problem or force the refactor to happen mid-implementation under feature pressure. Doing the refactor first ships `--aicontrol` against a clean foundation.

---

## Sequencing — open

This task is the first step of the four-step roadmap locked after D-066:

1. **M5 (this task): `ops::*` refactor.** ← you are here.
2. Multiparty test suite baseline pass (S1 Tauri rerun + S2–S5 present-`--batch`).
3. `--aicontrol` v1 implementation.
4. Multiparty improved pass (A/B metrics fill in).

D-066's relationship table notes M5 as the structural enabler for steps 3 and 4. Step 2 also benefits because the baseline pass exercises unified handlers rather than the drift-prone duplicates.

Phase 3 MLS work (D3 in the roadmap) is an independent workstream and does not block on M5.

---

## Scope — pure refactor, locked

M5 ships exactly these structural changes:

1. **New module: `xgen-client-lib::ops`.** One function per Client command verb. Each function takes an `OpContext` (defined in §3 below) and command-specific arguments; returns `Result<CommandResult>` where `CommandResult` is a command-specific result struct.

2. **New types: `SessionState`, `OpContext`, per-command `CommandResult` variants.** These live in `xgen-client-lib::ops` alongside the command functions. `SessionState` carries the WebSocket connection (one-shot in M5 — persistent comes in M7), the binding namespace (empty in M5 — populated in M7), and the per-Space cache (empty in M5 — populated in M7).

3. **Migration of every `cmd_*` and `exec_*` to thin shims.** Each existing command handler in `main.rs` and `batch.rs` becomes a one-screen wrapper that builds an `OpContext`, calls the matching `ops::*` function, and formats the result for its respective output channel (CLI stdout for `cmd_*`, pipe `OK\n` / `ERROR: ...\n` for `exec_*`).

4. **Deletion of all duplicate logic.** The two `get_dag_tips` copies become one. Any other helper currently duplicated between `main.rs` and `batch.rs` becomes one. After M5, `grep -r "fn get_dag_tips"` in the workspace returns exactly one match.

What M5 explicitly does **NOT** include (these are M7's scope, per D-066 addendum):

- Persistent WebSocket to the home Node.
- Named bindings or `$<name>` substitution.
- JSONL command/reply protocol.
- The `--aicontrol` flag itself.
- The `.events` pipe.
- Lifecycle-aware structured errors.
- The `state` command.
- Per-command timeout fields.

The `SessionState` and `OpContext` types are designed in M5 to *accommodate* M7's extensions without re-design (the fields M7 needs are stubbed or empty in M5), but no M7 functionality lands in M5.

---

## Architectural foundation — LOCKED

### 1. The shared layer: `xgen-client-lib::ops`

The new module lives in the library crate at `xgen-client/src/ops.rs` (single file in M5; may split into submodules in M7 if the command count grows). Visibility: `pub` for all command functions, `pub(crate)` for internal helpers shared across commands.

```rust
// xgen-client/src/ops.rs (skeleton)

pub mod ops {
    use crate::session::SessionState;
    
    pub struct OpContext<'a> {
        pub session: &'a mut SessionState,
        pub data_dir: &'a Path,
        pub node_override: Option<&'a str>,  // CLI --node flag
    }
    
    // One function per command verb. Each takes (&mut OpContext, ArgsStruct)
    // and returns Result<CommandResultStruct>.
    pub async fn register(ctx: &mut OpContext<'_>, args: RegisterArgs) -> Result<RegisterResult> { ... }
    pub async fn create_space(ctx: &mut OpContext<'_>, args: CreateSpaceArgs) -> Result<CreateSpaceResult> { ... }
    pub async fn create_room(ctx: &mut OpContext<'_>, args: CreateRoomArgs) -> Result<CreateRoomResult> { ... }
    pub async fn invite(ctx: &mut OpContext<'_>, args: InviteArgs) -> Result<InviteResult> { ... }
    pub async fn join(ctx: &mut OpContext<'_>, args: JoinArgs) -> Result<JoinResult> { ... }
    pub async fn send(ctx: &mut OpContext<'_>, args: SendArgs) -> Result<SendResult> { ... }
    pub async fn history(ctx: &mut OpContext<'_>, args: HistoryArgs) -> Result<HistoryResult> { ... }
    pub async fn whoami(ctx: &mut OpContext<'_>) -> Result<WhoamiResult> { ... }
    pub async fn status(ctx: &mut OpContext<'_>) -> Result<StatusResult> { ... }
    pub async fn spaces(ctx: &mut OpContext<'_>) -> Result<SpacesResult> { ... }
    pub async fn rooms(ctx: &mut OpContext<'_>, args: RoomsArgs) -> Result<RoomsResult> { ... }
    pub async fn members(ctx: &mut OpContext<'_>, args: MembersArgs) -> Result<MembersResult> { ... }
    pub async fn federate(ctx: &mut OpContext<'_>, args: FederateArgs) -> Result<FederateResult> { ... }
    pub async fn ai_delegate(ctx: &mut OpContext<'_>, args: AiDelegateArgs) -> Result<AiDelegateResult> { ... }
    pub async fn ai_revoke(ctx: &mut OpContext<'_>, args: AiRevokeArgs) -> Result<AiRevokeResult> { ... }
    pub async fn ai_status(ctx: &mut OpContext<'_>, args: AiStatusArgs) -> Result<AiStatusResult> { ... }
}
```

The argument structs (`RegisterArgs` etc.) already exist in `xgen-client/src/app.rs` as clap `#[derive(Args)]` types. M5 reuses them directly — no parallel arg-struct hierarchy. The result structs are new in M5 and carry the data the dispatcher needs to format for its output channel.

**Result struct shape (example for `create-space`):**

```rust
pub struct CreateSpaceResult {
    pub space_id: String,        // xgen://hash/sha256:...
    pub event_id: String,        // xgen://hash/sha256:... of the state.space_create event
    pub timestamp: String,       // RFC 3339 UTC
}
```

Result structs are flat, `pub` field-by-field, no methods beyond `Default` where useful. They are the data the formatters consume; format-specific concerns (pretty-printing, JSONL serialisation) stay in the dispatchers.

### 2. `SessionState` — one-shot in M5, extension points for M7

`SessionState` lives in `xgen-client/src/session.rs` (new file). M5's shape is minimal:

```rust
pub struct SessionState {
    pub conn: Option<Connection>,          // WebSocket connection — None until first network op
    pub identity: Option<ClientIdentity>,  // loaded keypair + cached identity_id
    pub home_node: String,                 // resolved from --node, [client].node config, in that order
    pub data_dir: PathBuf,
    
    // ── M7 extension points (present but unused in M5) ─────────────────
    pub bindings: HashMap<String, String>,      // M7: $<name> substitution map; empty in M5
    pub spaces: HashMap<String, SpaceCache>,    // M7: per-Space last-event cache; empty in M5
    // pub persistent: bool,                    // M7: true for --aicontrol, false for --batch
}
```

The M7 extension fields are present in M5 so that the type signature doesn't change between M5 and M7. M5 leaves them initialised to empty defaults; the M5 dispatchers ignore them; M5 tests verify they stay empty across operations (defensive — confirms M5 doesn't accidentally start populating them).

**One-shot semantics in M5.** Each `cmd_*` and `exec_*` invocation constructs a fresh `SessionState`, calls the relevant `ops::*` function (which opens a WebSocket as needed, does the work, leaves the connection in `SessionState.conn`), then drops `SessionState`. The Drop impl on `SessionState` closes any open WebSocket. This is exactly the same network behaviour as today — M5 does not change *when* connections open or close, only *where* the code lives.

### 3. The atomic-commit migration contract

**This contract is non-negotiable.** Partial migration (one command migrated, another not, both calling shared `ops::*`) creates a *third* drift surface and is explicitly forbidden.

For each command verb (`register`, `create-space`, `create-room`, ...), one atomic commit performs all four of these steps:

1. **Add the `ops::*` function** with the consolidated implementation. Take the better of the two current implementations (or the merger of both if they diverge in useful ways — record the merge decision in the commit message).
2. **Replace `cmd_<verb>` in `main.rs`** with a thin shim that builds an `OpContext`, calls `ops::<verb>`, formats the result for CLI stdout.
3. **Replace `exec_<verb>` in `batch.rs`** with a thin shim that builds an `OpContext`, calls `ops::<verb>`, formats the result for the pipe (`OK\n` plus result data on subsequent lines, or `ERROR: <message>\n`).
4. **Delete any helper code in `main.rs` / `batch.rs` that is now unused** because both dispatchers route through `ops::<verb>`. In particular: the two `get_dag_tips` copies vanish into one `ops::*`-private helper at this step.

All four steps in the same commit. The post-condition for that commit: `cargo build` clean, `cargo test --workspace --release` green at no fewer tests than baseline, and `grep "fn cmd_<verb>"` returns one match (the thin shim) and `grep "fn exec_<verb>"` returns one match (the thin shim).

**Migration order.** Migrate commands in this order to surface design issues early:

1. `whoami` — simplest, no network. Validates the `OpContext` plumbing and the `Result<X>` flow without WebSocket complexity. Establishes the pattern.
2. `status` — same shape as `whoami`. Confirms the pattern works for a second offline command.
3. `spaces` — also offline. Last of the read-only-from-state-file commands.
4. `register` — first network command. Validates the WebSocket-opening flow inside `ops::*`.
5. `create-space` — first state-changing network command. Validates result-struct extraction (`space_id`, `event_id`).
6. `create-room` — second state-changing command. Confirms the pattern.
7. `invite` — first membership command.
8. `join` — second membership command.
9. `send` — the command where the `get_dag_tips` duplicate-bug lived. **This is the headline migration** — when it lands, F-003/F-004 class is architecturally eliminated.
10. `history` — read-only with pagination.
11. `rooms`, `members`, `federate` — remaining state-reading and federation commands.
12. `ai delegate`, `ai revoke`, `ai status` — the M3 surface.

Twelve to thirteen commits total (`rooms`/`members`/`federate` may be combined if the diffs are trivially small; `ai *` may be one combined commit). Each commit is independently reviewable and revertable.

### 4. Helpers and shared utilities

Several internal helpers are currently duplicated. M5 consolidates them into `ops`-private (or `xgen-client-lib`-private) helpers:

- **`get_dag_tips`** — currently in both `main.rs` and `batch.rs`. Becomes a single function in `ops::send` (or a helper module shared by `send` and `create-room`). The bug fix that landed for F-003 on one copy applies to all callers automatically after M5.
- **`connect_and_authenticate`** — currently inlined per-command. Becomes a private `SessionState` method that opens the WebSocket lazily on first network operation.
- **`load_keypair`** — currently inlined per-command. Becomes a private `SessionState::ensure_identity` method.
- **`build_event` helpers** — already in `xgen-core::space::state`; no change needed.

Helpers that are not shared (e.g. a function used only by one command) stay private to that command's implementation in `ops::*`.

### 5. The Tauri UI path

`xgen-client/src-tauri/src/main.rs` defines Tauri commands (`#[tauri::command]` functions) that are invoked from the Svelte frontend. Today these duplicate-yet-again the `cmd_*` / `exec_*` logic.

M5 migrates the Tauri commands to call `ops::*` directly. Same atomic-commit contract: when `cmd_send` and `exec_send` become thin shims, the Tauri-side `send_message` Tauri command also becomes a thin shim calling `ops::send`. The Tauri shell is the third dispatcher; D-063 said "library-first," and the Tauri commands are part of that library-callable surface.

This is the third reason atomicity matters: leaving the Tauri commands on the old implementation while `cmd_*` and `exec_*` migrate creates a third drift surface, which would make M5 worse than the current state.

---

## Implementation decisions

These are decisions Chat Claude and Clair lock together during M5; they do not require Joe's input but are recorded here so the rationale survives.

### D-M5-1: Where `OpContext` is constructed

Each dispatcher constructs its own `OpContext`. The CLI dispatcher builds it once per process invocation; the pipe dispatcher builds it once per pipe connection in M5 (per connection, not per command — even in the one-shot M5 model, a single pipe connection may dispatch multiple lines, and reusing the `OpContext` across them keeps the cached identity load and config parse out of the per-line hot path).

This naturally extends to M7: `--aicontrol` keeps one `OpContext` for the lifetime of the connection (now actually persistent, not just per-command-batch), populates the binding namespace and per-Space cache across calls.

### D-M5-2: Error type — `anyhow::Result` in M5, structured in M7

M5 keeps `anyhow::Result<T>` as the return type from `ops::*` functions. This is the simplest change that keeps M5 a pure refactor — error formatting in the dispatchers stays exactly what it is today (`anyhow::Error::to_string()` for CLI, same for pipe).

M7 introduces a structured `ControlError` enum carrying the `category` / `code` / `instance_state` / `hint` fields from the D-066 addendum §3.3. M5 does not introduce this type — M7 wraps `anyhow::Error` into `ControlError` at the dispatcher boundary as part of the `--aicontrol` work.

### D-M5-3: `SessionState` Drop behaviour

The `Drop` impl on `SessionState` performs a best-effort `goodbye` on the WebSocket if one is open, then closes the socket. Failures during Drop are logged at WARN but not propagated (per Rust's "don't panic in Drop" rule). This matches today's per-command teardown behaviour — each command currently closes its own connection at the end of its work.

### D-M5-4: Test strategy

Three layers of test coverage:

1. **Existing tests stay green.** The baseline `cargo test --workspace --release` count (429 after M4) must not drop. New tests may push it higher, but no test may be removed in M5.
2. **New unit tests for `ops::*` functions** where the function's logic is non-trivial. `ops::send` warrants unit tests for the `get_dag_tips` path; `ops::register` warrants unit tests for the AI capability path; offline commands (`whoami`, `status`, `spaces`) get smoke tests.
3. **Integration test: full smoke against running Nodes** at the end of M5, before the close-out commit. Run `xgen-client smoke-test --node-a ws://... --node-b ws://...` (the existing Phase 1 17-step smoke). PASS confirms M5's refactor preserves end-to-end protocol behaviour. If it fails, the close-out commit does not land — the regression is investigated first.

The smoke test result is quoted in the closing journal entry (matching M2/M3/M4 rhythm).

### D-M5-5: No backwards-compat shim layer

M5 does not introduce a "transitional" set of functions that both old and new dispatchers can call during migration. The atomic-commit contract means each command moves in one commit; there is no in-between state where some old and some new exist simultaneously. The simplicity of the migration is bought by the contract; weakening the contract for transitional ease would re-introduce drift risk for short-term convenience and is rejected.

---

## Definition of Done

- [ ] Phase 0 baseline captured: `cargo test --workspace --release` quoted in the close-out journal entry.
- [ ] `xgen-client-lib::ops` module exists with one function per Client command verb (the 13–16 verbs listed in §1).
- [ ] `xgen-client-lib::session::SessionState` exists with M5 minimum shape (conn, identity, home_node, data_dir) plus M7 extension fields stubbed (bindings, spaces).
- [ ] `xgen-client-lib::ops::OpContext` exists.
- [ ] Every command verb's three dispatchers (`cmd_<verb>` in main.rs, `exec_<verb>` in batch.rs, the Tauri command in src-tauri/src/main.rs) are thin shims calling `ops::<verb>`.
- [ ] `grep -r "fn get_dag_tips" xgen-client/` returns exactly one match (inside the `ops` module or its helpers).
- [ ] No duplicate command logic anywhere in `xgen-client/src/`. The two-set drift surface is gone.
- [ ] `cargo build --release --workspace` clean — no new warnings beyond the existing baseline.
- [ ] `cargo test --workspace --release` green at no fewer tests than the M4 baseline (429).
- [ ] Integration smoke test against running Nodes (`smoke-test --node-a ... --node-b ...`) passes end-to-end.
- [ ] `DECISIONS.md` updated with D-067 capturing the structural outcome (single source of truth for Client commands; M7 prerequisite met).
- [ ] `JOURNAL.md` close-out entry written, quoting cargo output, listing the per-command commits, and confirming the smoke test result.
- [ ] `tasks/M5_OPS_REFACTOR.md` status flipped from PENDING to COMPLETED.
- [ ] `CLAUDE.md` updated; next session entry point reset to point at M6 (Multiparty baseline pass).

(Definition of Done does NOT include "commit pushed" per the project task-file convention — `Status: COMPLETED` is the canonical signal that the work shipped.)

---

## Next session entry point

M6 — Multiparty baseline pass (S1 Tauri rerun + S2–S5 with present `--batch`). Task file `tasks/MULTIPARTY_S1_tauri_rerun.md` and `tasks/MULTIPARTY_S2_to_S5_present_pass.md` already exist and are the baseline runbooks; both reference Clair's `BATCH_FLAG_review.md` for the metrics protocol that M6 captures into the "A" column of every scenario's findings file.

---

*End of `M5_OPS_REFACTOR.md`*
