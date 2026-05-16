# M1 — Binary Consolidation
> **Status**: PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Why this exists

D-056 (locked 2026-05-16) names the target deployment shape: **one binary per role** (`xgen-node.exe`, `xgen-client.exe`), each with two mode categories (resident + control), dispatched at startup. The current repo has four binaries — `xgen-node.exe` + `xgen-node-app.exe` + `xgen-client.exe` + `xgen-client-app.exe` — with two parallel `--batch` implementations on the Client side (F-003 / F-004 in J-067). D-056 is the spec target; M1 is the implementation pass that brings the code in line.

M1 is structural: it consolidates the binary topology AND lands the fundamental-flag contract on both binaries. After M1, both binaries are symmetric on the 19 fundamental flags; role-specific flags remain on their respective binaries.

All multiparty tests (S1 Tauri rerun and the S2–S5 present pass) are postponed and will be **redesigned from scratch** after M1/M2 land, not carried over. This is a clean reset, not a rerun against the new shape.

---

## Cross-references

| Source | Relevance |
|---|---|
| `DECISIONS.md` D-056 | Architectural target — locks the deployment model M1 implements |
| `DECISIONS.md` D-043 | Pipe naming convention `\\.\pipe\xgen-{node\|client}-{label}` |
| `DECISIONS.md` D-035 | Removed `log_path` / `spaces_dir` from CLI config — referenced by `--log-level` decision |
| `JOURNAL.md` J-067 | F-001 / F-002 / F-003 / F-004 record. F-003/F-004 closed permanently as a byproduct of M1's single `--batch` path |
| `docs/xgen_appendix_e_en.md` | Lifecycle states — clarified by D-056 to apply to resident mode only |
| `docs/xgen_ch2_architecture.md` | Application Deployment Model section (rewritten in D-056) |
| `tasks/BATCH_FLAG_review.md` | Context on the `--batch` surface and the duplicate `get_dag_tips` problem |
| `docs/tests/BATCH_FLAG_ph2.md` | Original `--batch` spec (Client side) |

---

## Scope

**In scope for M1:**
1. Merge `xgen-node/src-tauri/src/main.rs` into `xgen-node/src/main.rs` (one binary: `xgen-node.exe`).
2. Merge `xgen-client/src-tauri/src/main.rs` into `xgen-client/src/main.rs` (one binary: `xgen-client.exe`).
3. Extract resident-mode logic (`run_node_server`, `start_client_session`, and friends) into the library crates (`xgen-node/src/lib.rs`, `xgen-client/src/lib.rs`).
4. Tauri always compiled in (option b — D-057 reserved for this decision). Runtime dispatch chooses whether to initialise the UI.
5. Eliminate the two parallel `--batch` implementations on the Client side. After M1, exactly one `--batch` code path on each binary, calling the shared library command layer. `get_dag_tips` lives in one place. F-003 / F-004 dedup is verified.
6. Implement all 19 fundamental flags on both binaries (full list below).
7. Per-binary verification matrix executed against the merged binaries.

**Out of scope (deferred to later milestones):**
- Pipe server in Node resident mode → M2
- AI Client deployment (DM control plane, AI-specific config, designated operator, etc.) → M3
- Multiparty test suite (redesigned from scratch, not migrated) → after M2/M3 land
- Long-lived `--batch` mode and the `BATCH_FLAG_review.md` leverage proposals → after multiparty redesign
- Characterisation of the 6/300 P2 message loss observed in J-067 → after multiparty redesign

---

## The fundamental flag contract (post-M1)

After M1, both `xgen-node.exe` and `xgen-client.exe` support all 19 of the following flags. Role-specific flags remain on their respective binaries unchanged.

| # | Flag / command | Mode category | Purpose |
|---|---|---|---|
| 1 | `--instance <label>` | Modifies any resident mode | Deployment-shape modifier — derives data-dir, log file naming, pipe naming. D-043. |
| 2 | `--config <path>` | Modifies any mode | Path override for the config file. |
| 3 | `--service` | Selects resident headless | Headless variant of resident mode. No UI. Pipe server active. |
| 4 | `init [--passphrase X]` | Control mode (one-shot) | Keypair + config generation. Exits cleanly. |
| 5 | `--batch <file>` | Control mode | Sequential dispatch of `.xgb` commands against a running resident via pipe. |
| 6 | `--stop` | Control mode | Graceful shutdown signal to a `--service` resident via pipe. |
| 7 | `--reload-config` | Control mode | Reload config in a `--service` resident via pipe. |
| 8 | `--check-config` | Control mode (read-only) | Validate config syntax/semantics, exit. No side effects, no pipe contact. |
| 9 | `--print-config` | Control mode (read-only) | Print effective config (defaults + file + flags merged), exit. |
| 10 | `--health` | Control mode (read-only) | Read-only liveness probe of a running resident via pipe. Alias / overlap with `status`. |
| 11 | `--pid` | Control mode (read-only) | Print the resident PID to stdout, exit. |
| 12 | `--ping` | Control mode (read-only) | Open the pipe, send noop, receive reply. Print roundtrip latency in ms. Exit 0 on success. |
| 13 | `--log-level <lvl>` | Modifier | Override the effective logging level for this invocation. |
| 14 | `--quiet` | Modifier | Suppress startup chatter on stdout. Errors still surface. |
| 15 | `--help` / `-h` | Control mode (read-only) | clap default. Print usage, exit. |
| 16 | `--version` / `-V` | Control mode (read-only) | clap default short form. Print version string, exit. |
| 17 | `version` *(subcommand)* | Control mode (read-only) | Long-form version + build metadata + Node/Identity ID. |
| 18 | `status` *(subcommand)* | Control mode (read-only) | Read the binary's `*_state.json` file, print summary. Role-specific content. |
| 19 | `whoami` *(subcommand)* | Control mode (read-only) | Print the Identity / Node ID that this binary owns. Role-specific content (Client = Identity + display name; Node = Node ID + operator_display_name). |

### What stays role-specific (not promoted to fundamental)

**Node-only:**
- `--local` — Force Local Node mode regardless of config
- `--port <n>` — Override the WS listener port
- `connections`, `peers`, `identity list` — Node-specific introspection subcommands

**Client-only:**
- `--node <url>` — Specify the home Node WS endpoint
- `register`, `create-space`, `create-room`, `invite`, `join`, `send` — Client's protocol-action subcommands

### `spaces` subcommand — ambiguous case

Both binaries currently have `spaces` as a subcommand with role-specific content (Node = Spaces hosted; Client = Spaces I'm a member of). This is effectively fundamental in shape but role-specific in semantics, like `status` and `whoami`. M1 preserves both — same name, different output. No rename. Document the divergence in `xgen_appendix_f_en.md` when it's next updated.

---

## Decisions M1 introduces (D-NNN entries to record on completion)

Two new decisions land with M1. These numbers are reserved; Claire records the full entries in `DECISIONS.md` when she ships.

**D-057 — Tauri inclusion model: compiled into product binary, runtime dispatch chooses UI initialisation.**
- Choice between (a) Cargo feature flag `tauri` (build-time variant) and (b) always-compiled-in (runtime variant) was made in favour of (b).
- Rationale: fewer error-classes. Under (a) a packager forgetting `--features tauri` would ship a GUI-less binary to a desktop user — a real packaging-mistake category. (b) eliminates that class entirely at the cost of slightly larger binary size and longer build time for headless deployments.
- Acknowledged costs: server-shape deployment carries WebView2/Tauri runtime dependencies it never invokes; `cargo build` time grows with the UI rather than the protocol; CI runs one build instead of two and so cannot independently classify "this break is UI-side" vs "this break is protocol-side." All accepted.
- This decision is the literal Rust expression of "one binary per role, multiple mode variants" from D-056.

**D-058 — Resident-mode logic moves to the library crate.**
- `run_node_server()`, `start_client_session()`, and the shared infrastructure they call now live in `xgen-node/src/lib.rs` and `xgen-client/src/lib.rs`.
- The binary's `main.rs` becomes a thin dispatcher: parse flags, decide mode, call the corresponding library function. No business logic in `main.rs`.
- Required by D-056's shared command layer — all input channels (Tauri UI button clicks, Console typed commands, `--batch` piped commands, future control-mode flags) must dispatch through the same command layer; that layer must live somewhere that all entry points can call.
- The library-first architecture rule from `CLAUDE.md` is now applied to the merged binary structure.

---

## Pre-flight reading

Before starting the implementation, Claire reads:

1. `CLAUDE.md` — current behaviour rules, especially the "Never fabricate results / Show actual output / Stop and report when a tool fails" set.
2. `DECISIONS.md` D-056, D-043, D-037 — architectural targets and pipe naming convention.
3. `tasks/BATCH_FLAG_review.md` — context on the `--batch` surface, including the friction points the merge resolves.
4. The four current entry-point files:
   - `xgen-node/src/main.rs`
   - `xgen-node/src-tauri/src/main.rs`
   - `xgen-client/src/main.rs`
   - `xgen-client/src-tauri/src/main.rs`
5. `xgen-client/src/batch.rs` and any pipe-server code in the Tauri shell — the model for how M1 wires the dispatcher.

---

## Implementation steps (recommended sequence)

### Phase 0 — Pre-flight

1. **Baseline.** Run `cargo test --workspace` and `cargo build --release --workspace`. Record the exact test count. Quote the actual output in the journal entry. This number is the M1 acceptance baseline (expected: 391 per J-067 — verify, don't assume).

### Phase 1 — Library-crate extraction (D-058)

2. **Extract Node resident-mode logic.** Move `run_node()` and its helpers from `xgen-node/src/main.rs` into `xgen-node/src/lib.rs` as `run_node_server()` (or similar — name TBD by Claire). Both the future CLI dispatcher and the future Tauri dispatcher will call this. The CLI binary keeps working through this step; tests stay green.

3. **Extract Client resident-mode logic.** Move the Client's main protocol surface (connect, register, send, history fetch) from `xgen-client/src/main.rs` into `xgen-client/src/lib.rs` as a coherent API. The current CLI subcommands (`register`, `create-space`, etc.) call into this. Tests stay green.

### Phase 2 — Merge

4. **Node binary merge.** Collapse `xgen-node/src-tauri/src/main.rs` into `xgen-node/src/main.rs`. Single `[[bin]]` entry. Dispatch logic in `main.rs`:
   - Parse flags (clap)
   - Detect mode: control mode (any control-mode flag present), resident headless (`--service`), or resident desktop (default).
   - Control mode → dispatch to the corresponding handler, exit.
   - Resident desktop → initialise Tauri + start library-crate `run_node_server()`.
   - Resident headless → start library-crate `run_node_server()`, no Tauri init.

5. **Client binary merge.** Same shape for `xgen-client/src/main.rs`. Eliminates `xgen-client-app.exe` as a separate target.

6. **Cargo.toml cleanup.** Remove `*-app` binary targets. The `src-tauri` directories may stay as source containers (Tauri assets, icons, config) but they no longer produce a separate `[[bin]]`. Adjust `tauri.conf.json` paths as needed.

### Phase 3 — Single `--batch` path on Client (closes F-003/F-004)

7. **Unify Client `--batch`.** The current two `--batch` implementations (`xgen-client/src/main.rs::run_batch_file` and `xgen-client/src-tauri/src/batch.rs`) collapse into one, living in the library crate (`xgen-client/src/lib.rs::batch` module — current `batch.rs` becomes the library module). `get_dag_tips` lives in exactly one place. The pipe-server logic from the Tauri shell is preserved as part of the resident-mode startup.

8. **Verify F-003 / F-004 dedup.** Search the codebase: `get_dag_tips` must return exactly one match. No duplicate Space-filter logic.

### Phase 4 — Fundamental flag implementation

9. **Inventory existing flags per binary.** Confirm what's already there, what's missing. (Pre-implementation inventory expected: Node missing `--batch`, `whoami`; Client missing `--service`; both missing `--stop`, `--reload-config`, `--check-config`, `--print-config`, `--health`, `--log-level`, `--quiet`, `--pid`, `--ping`.)

10. **Implement missing flags on both binaries.** Add clap definitions, route through the shared library command layer where applicable. Order suggestion (least to most complex):
    - `--quiet`, `--log-level` (modifiers, simple to add to existing init paths)
    - `--check-config`, `--print-config` (read-only, no pipe contact)
    - `--pid`, `--ping`, `--health` (control-mode pipe reads)
    - `--stop`, `--reload-config` (control-mode pipe writes — depends on M2 for Node pipe; for Client, the existing pipe server handles it)
    - `version`, `status`, `whoami` on Node side (Node already has `version`, `status` — add `whoami` printing `node_id_uri + operator_display_name`)
    - `--service` on Client side (the AI / headless deployment shape — Tauri-less long-lived process with pipe server)
    - `--batch` on Node side (port the Client pattern, Node command set)

    **Note for `--stop` / `--reload-config` / `--health` / `--ping` on Node side:** these require a pipe server in the Node resident mode, which is M2's work. M1's Node-side implementation may stub these (parse the flag, return "not yet implemented — requires M2") OR M1 may include a minimal pipe server on the Node resident side, folding part of M2 into M1. Claire's call based on implementation cost; document the choice in the journal.

11. **`--ping` latency semantics.** `--ping` returns the milliseconds elapsed between opening the pipe, sending a noop, and receiving the reply. This is local-pipe RTT, not protocol-level RTT to a remote Node. Documented behaviour. Print form: `pong: <n> ms` followed by exit 0; on pipe failure print `no resident found at <pipe-name>` and exit non-zero.

12. **`--check-config` and `--print-config` semantics.** `--check-config` parses the config (including defaults merge and any `--config` override), validates it, prints `config OK: <path>` or the first validation error, exits with 0/non-0. `--print-config` parses the effective config and prints it as TOML to stdout, exits 0. Neither contacts the pipe.

13. **`--health` semantics.** Opens the pipe, sends a `health` request, receives a structured response (state, uptime, last-error-if-any), prints a one-line summary, exits 0 if healthy and non-0 if degraded. Overlap with `status` is acknowledged — `status` reads the state file (works without a running resident, may be stale), `--health` queries the live process (requires the resident to be running and the pipe to respond).

### Phase 5 — Verification

14. **`cargo test --workspace` — green.** Test count matches the Phase 0 baseline (391 expected). No regressions.

15. **`cargo build --release --workspace` — clean.** No warnings. Exactly two binaries produced: `xgen-node.exe` and `xgen-client.exe`. No `*-app.exe` artefacts.

16. **Per-binary verification matrix.** Execute every cell, quote actual output in the journal entry.

---

## Per-binary verification matrix

Each cell is a single check against the merged binary. Quote actual output for each row.

### `xgen-node.exe`

| # | Invocation | Expected behaviour |
|---|---|---|
| N1 | `xgen-node.exe` (no flags) | Tauri window opens; systray icon appears; lifecycle states wire through; WS server binds on configured port. Functional equivalence to current `xgen-node-app.exe`. |
| N2 | `xgen-node.exe --instance n1` | Same as N1 + instance label honoured. Data-dir at `<exe_dir>/instances/n1/`. Log file at `instances/n1/logs/xgen-node_<timestamp>.log`. |
| N3 | `xgen-node.exe --service` | No UI; WS server binds; pipe server present (if M1 includes minimal Node pipe server) or not (if M1 defers Node pipe to M2). Document choice. |
| N4 | `xgen-node.exe --service --instance n1` | Same as N3 + instance label honoured. |
| N5 | `xgen-node.exe init [--passphrase X]` | Keypair + config generated, exits cleanly. No UI initialised. |
| N6 | `xgen-node.exe init --instance n1 [--passphrase X]` | Same + writes to instance-specific data dir. |
| N7 | `xgen-node.exe status` | Reads `xgen-node_state.json`, prints role-specific summary. |
| N8 | `xgen-node.exe whoami` | Prints `node_id` (xgen://pubkey/...) + `operator_display_name` from config. |
| N9 | `xgen-node.exe version` | Long-form version + build metadata + Node ID. |
| N10 | `xgen-node.exe --version` | Short form (clap default). |
| N11 | `xgen-node.exe --help` | Usage. |
| N12 | `xgen-node.exe --check-config` | Validates config, prints OK or first error, exits 0/non-0. |
| N13 | `xgen-node.exe --print-config` | Prints effective config as TOML, exits 0. |
| N14 | `xgen-node.exe --batch script.xgb` | Connects via pipe to a running Node resident, dispatches Node command set, exits cleanly. (Requires the Node pipe server — see N3 note.) |
| N15 | `xgen-node.exe --pid` | Prints resident PID. |
| N16 | `xgen-node.exe --ping` | Prints `pong: <n> ms`, exits 0. |
| N17 | `xgen-node.exe --health` | Prints liveness summary, exits 0 / non-0. |
| N18 | `xgen-node.exe --stop` | Signals resident to exit gracefully. |
| N19 | `xgen-node.exe --reload-config` | Signals resident to reload config. |
| N20 | `xgen-node.exe --log-level debug` (with resident launch) | Resident logs at debug level for this invocation. |
| N21 | `xgen-node.exe --quiet` (with resident launch) | Startup chatter suppressed. |
| N22 | `xgen-node.exe connections` | (Existing Node-specific subcommand, must still work.) |
| N23 | `xgen-node.exe peers` | (Existing Node-specific subcommand, must still work.) |
| N24 | `xgen-node.exe spaces` | (Existing Node-specific subcommand, must still work.) |
| N25 | `xgen-node.exe identity list` | (Existing Node-specific subcommand, must still work.) |

### `xgen-client.exe`

| # | Invocation | Expected behaviour |
|---|---|---|
| C1 | `xgen-client.exe` (no flags) | Tauri window opens. Functional equivalence to current `xgen-client-app.exe`. |
| C2 | `xgen-client.exe --instance c1` | Same + instance label honoured. |
| C3 | `xgen-client.exe --service` | No UI; pipe server active; WS connection to home Node established and maintained. **New deployment shape.** |
| C4 | `xgen-client.exe --service --instance c1` | Same as C3 + instance label honoured. |
| C5 | `xgen-client.exe init [--passphrase X]` | Keypair + config generated, exits cleanly. No UI initialised. |
| C6 | `xgen-client.exe init --instance c1 [--passphrase X]` | Same + writes to instance-specific data dir. |
| C7 | `xgen-client.exe status` | Reads `xgen-client_state.json`, prints role-specific summary. |
| C8 | `xgen-client.exe whoami` | Prints local Identity ID + display name. |
| C9 | `xgen-client.exe version` | Long-form version + build metadata + Identity ID. |
| C10 | `xgen-client.exe --version` | Short form (clap default). |
| C11 | `xgen-client.exe --help` | Usage. |
| C12 | `xgen-client.exe --check-config` | Validates config, prints OK or first error. |
| C13 | `xgen-client.exe --print-config` | Prints effective config as TOML. |
| C14 | `xgen-client.exe --batch script.xgb` | Connects to running resident via pipe, dispatches Client command set, exits cleanly. **Single `--batch` code path** (F-003/F-004 dedup verified). |
| C15 | `xgen-client.exe --instance c1 --batch script.xgb` | Same + instance label resolves correctly to pipe target. |
| C16 | `xgen-client.exe --pid` | Prints resident PID. |
| C17 | `xgen-client.exe --ping` | Prints `pong: <n> ms`. |
| C18 | `xgen-client.exe --health` | Liveness summary. |
| C19 | `xgen-client.exe --stop` | Signals resident to exit gracefully. |
| C20 | `xgen-client.exe --reload-config` | Signals resident to reload config. |
| C21 | `xgen-client.exe --log-level debug` (with resident launch) | Resident logs at debug. |
| C22 | `xgen-client.exe --quiet` (with resident launch) | Startup chatter suppressed. |
| C23 | `xgen-client.exe spaces` | (Existing Client-specific subcommand, must still work.) |
| C24 | `xgen-client.exe register / create-space / create-room / invite / join / send` | (Existing Client-specific subcommands, must still work — exercise at least one to confirm.) |

---

## Definition of Done

Each item independently verified with actual output in the journal entry. Do not mark complete based on assumption.

- [ ] **Baseline captured.** `cargo test --workspace` count recorded pre-M1 (expected 391).
- [ ] **Library-crate extraction (D-058) complete.** `run_node_server` / Client resident logic live in `lib.rs`; `main.rs` is a thin dispatcher on both binaries.
- [ ] **Single Cargo `[[bin]]` per role.** No `*-app.exe` build targets remain.
- [ ] **`cargo build --release --workspace` clean** with zero warnings; exactly `xgen-node.exe` and `xgen-client.exe` produced.
- [ ] **`cargo test --workspace` green** at the Phase 0 baseline count (no regressions, no new test count required by M1 unless implementing `--service` Client adds incidental tests).
- [ ] **Single `--batch` code path on Client.** `get_dag_tips` exists in exactly one location in the codebase. Grep result quoted in journal.
- [ ] **All 19 fundamental flags implemented on both binaries.** Each flag exercised against each binary per the verification matrix above. Output quoted per row.
- [ ] **`xgen-client.exe --service` mode operational.** Headless Client launches, registers/connects, pipe server active, stays running until `--stop`. New code, exercise carefully.
- [ ] **Node-side `--batch`, `--stop`, `--health`, `--ping`, `--reload-config` posture documented.** Either: (a) M1 includes a minimal Node pipe server (folding part of M2 into M1) and these work end-to-end; or (b) M1 stubs them with clear "not yet implemented — requires M2 pipe server" messages. Choice documented in journal with rationale.
- [ ] **D-057 and D-058 recorded** in `DECISIONS.md` with full text per the rationale sketched above.
- [ ] **`JOURNAL.md` entry** quoting all verification output (test count, build output, every matrix row's actual output). No paraphrasing.
- [ ] **`CLAUDE.md` Status section updated** to reflect M1 complete; M2 (Node pipe server) and M3 (AI Client deployment) reframed as the next milestones.
- [ ] **`xgen_appendix_f_en.md` updated** (or scheduled for update — note in journal) to reflect the post-M1 CLI surface on both binaries. The 19 fundamental flags documented; the role-specific flags clarified.

Note: per the May 2026 convention update, **commit pushed is not a DoD item**. The `Status: COMPLETED` header on this file is the signal that the work shipped.

---

## What this milestone makes obsolete

After M1 lands, the following files no longer match reality and need disposition. **Do not** mark them DEPRECATED as part of M1 — leave them for the multiparty redesign conversation after M2/M3, where the full impact will be clearer.

Files to revisit later:
- `tasks/MULTIPARTY_S1_tauri_rerun.md` — premise (rerun against the existing Tauri shape) becomes moot once M1 collapses the shapes.
- `tasks/MULTIPARTY_S2_to_S5_present_pass.md` — the "present pass" leg of the A/B comparison is against pre-M1 shape.
- `docs/tests/MULTIPARTY_S0_intro.md` through `MULTIPARTY_S5_*.md` — invocation patterns reference `*-app.exe` binaries that no longer exist.
- `docs/tests/scripts/multiparty_s1_*.xgb` — `.xgb` scripts whose Tauri-shell invocation context is obsolete.
- `tasks/BATCH_FLAG_review.md` — much of its content describes pre-M1 friction; some proposals carry forward, some are resolved by M1.

These remain in place during M1 because Claire reads `BATCH_FLAG_review.md` during the merge (it has the right context) and the others don't actively confuse her.

---

## Out-of-scope clarifications

To prevent scope creep, the following are **explicitly not** M1 concerns even though they are mentioned in passing here:

- AI Client deployment configuration schema, designated-operator binding, DM control plane — all M3.
- Long-lived `--batch` mode (persistent WS connection driving multiple `--batch` invocations) — post-multiparty-redesign.
- The 6/300 P2 message loss characterisation from J-067 — post-multiparty-redesign.
- Renaming `init`-as-subcommand to `--init`-as-flag for D-056 grammatical alignment — separate small decision later if Joe wants it; M1 preserves the existing subcommand grammar.
- Unifying `spaces` semantics across Node and Client — out of scope; M1 preserves the role-specific divergence.

---

## Behaviour rules reminder (from CLAUDE.md)

Claire reviews these before starting:
- **Rule 1** — Never fabricate results. Real output only.
- **Rule 2** — Show actual output. Quote terminal output verbatim in the journal entry.
- **Rule 3** — Stop and report when a tool fails. Do not work around silently.
- **Rule 4** — Write the journal entry last. After all verification is confirmed.
- **Rule 5** — Never invent numbers. Test counts come from `cargo test` output.
- **Rule 6** — When in doubt, do less and ask. Ambiguous instructions → ask Joe.
- **Rule 7** — Definition of Done is a checklist, not a formality. Each item independently verified.

If Phase 4 step 10's stub-vs-implement question for Node pipe server is ambiguous in practice, **stop and ask Joe** before deciding. The trade-off is folding part of M2 into M1 vs leaving five flags partially implemented on the Node side; either is defensible, but the call belongs to Joe.

---

*End of M1 task file.*
