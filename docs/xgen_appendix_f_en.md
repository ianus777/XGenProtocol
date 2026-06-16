# Appendix F — CLI Reference and Usage Examples
> **Status:** ACTIVE  
> Version: 1.10  
> Date: May 2026  
> **Last updated**: 2026-06-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

> **XGID discipline notice.** XGID discipline (DECISIONS.md D-072, `docs/xgen_appendix_j_en.md`) applies to all identifiers referenced in this appendix — CLI parameters, batch reply schemas, Node-side identifier surfaces. The five wire-format invariances of Ch3 §3.0.3 — field names, field types (string), canonical form, URI grammar, and string-equality semantics — bind every identifier surface documented here. Full retype of identifier-carrying fields to XGID flavour types **shipped at Retrofit Pass 4**: the xgen-client M5 ops Result schemas, CLI dispatcher arguments, and batch reply schemas now carry typed XGID flavours **in memory** (`IdentityXgid` / `SpaceXgid` / `EventXgid` / `RoomXgid` / `NodeXgid`) and remain **plain `String` on the wire** via serde-transparency, so the five invariances above hold byte-for-byte. CLI arguments are parsed as `String` and projected to typed flavours at the dispatcher arm (Pass 4 §4.3 Option α). Some Node-side surfaces were retyped at Pass 3.  

---

This appendix provides the complete CLI reference for `xgen-node` and `xgen-client`, followed by real-world usage examples covering common operator and user workflows. It is the authoritative reference for CLI syntax. The Rust source doc comments in `xgen-node/src/main.rs` and `xgen-client/src/main.rs` MUST match this appendix exactly (D-028).

---

## F.0 Flag model — fundamental vs non-fundamental

The CLI is organised around two role-specific binaries (`xgen-node`, `xgen-client`) that share a *fundamental* surface — flags and subcommands present on both with identical semantics — plus a *non-fundamental* surface that is binary-specific. The fundamental surface is the operator's common vocabulary; the non-fundamental surface is where Node and Client diverge by role.

The axis is **symmetry between binaries**, not internal dispatch order. A flag or subcommand is fundamental if it exists on both binaries and means the same thing; otherwise it is non-fundamental. This distinction reflects the D-056 deployment model (one binary per role + multi-mode dispatch) made visible at the CLI surface, and the D-063 library-first architecture that makes the shared surface implementable as a single shared command layer.

### F.0.1 Fundamental flags

Present on both `xgen-node` and `xgen-client` with identical semantics. These are the flags an operator can carry from one binary to the other without re-learning anything.

| Flag | Effect | Status |
|---|---|---|
| `--check-config` | Validate the effective config, print OK or first parse error, exit. Read-only, no pipe contact. | Both, functional |
| `--print-config` | Print the effective config as TOML on stdout and exit. Read-only. | Both, functional |
| `--pid` | Print the running resident's PID from `<data dir>/xgen-{node,client}.pid` and exit. | Both, functional |
| `--ping` | Round-trip a noop against the running resident's pipe and print latency in milliseconds. Exits 0 on PONG, non-zero on failure. | Both — Client functional, Node stubs |
| `--health` | Ask the running resident for a one-line liveness summary. Exits 0 if HEALTHY, non-zero otherwise. AI-mode residents extend the reply format — see §F.3. | Both — Client functional, Node stubs |
| `--stop` | Signal the running resident to shut down gracefully via pipe. Resident terminates itself after replying `OK STOPPING`. | Both — Client functional, Node stubs |
| `--reload-config` | Signal the running resident to reload its config via pipe. Currently returns `NOT_IMPLEMENTED` — reload semantics arrive later. | Both — Client functional, Node stubs |
| `--service` | Force headless resident mode (no UI). On both binaries this is the equivalent of pre-M1 "just run the binary" behaviour now that the default opens the Tauri shell. | Both, functional |
| `--instance <label>` | Segregate data and logs under `<exe dir>/instances/<label>/`. Drives the pipe name (D-043). Label rules: alphanumeric, hyphens, underscores; max 64 characters. | Both, functional |
| `--config <path>` | Override the config file path. Default: `./xgen-{node,client}_config.toml`. | Both, functional |
| `--log-level <lvl>` | Override the effective tracing level for this invocation. Wins over config and `XGEN_LOG`. Values: `off`, `error`, `warn`, `info`, `debug`, `trace`. | Both, functional |
| `--quiet` | Suppress startup banner / "Listening on..." line. Structured logs unaffected; errors still surface on stderr. | Both, functional |
| `--help` / `-h` | Print help and exit. | Both, clap-standard |
| `--version` / `-V` | Print version and build metadata, exit. | Both, clap-standard |

"Node stubs" means the flag is recognised but returns `error: <flag> requires the M2 Node pipe server — not yet implemented` and exits non-zero. The Node-side pipe server lands in a later milestone; the flag exists at the CLI surface today for symmetry and forward compatibility.

### F.0.2 Fundamental subcommands

Present on both binaries with the same role and similar output shape. Each binary's deep table (§F.2 / §F.3) re-lists these alongside the binary-specific subcommands for ease of reference.

| Subcommand | Role | Output |
|---|---|---|
| `init` | Generate keypair and default config in the data directory. Safe to re-run — will not overwrite an existing keypair. | Keypair path, config path, Identity/Node ID. |
| `whoami` | Print the local Identity (Client) or Node identity (Node) and display name. Reads local state — no Node connection required. | Identity/Node ID, display name. |
| `status` | Print the local state summary. No Node connection required — reads state file or live process state. | Per-binary state summary (see binary-specific tables). |
| `version` | Print version and build metadata. | Version, git SHA, build date. |

### F.0.3 Non-fundamental flags

Binary-specific. Either exists only on one side, or is meaningful only for one role.

| Flag | Binary | Effect |
|---|---|---|
| `--ai-mode` | Client | Run the `--service` resident as an AI Client (M4, spec 3.6.10). Requires `--service`; clap rejects standalone uses. Loads the plugin named by `[ai] plugin = "..."` in config. |
| `--batch <file>` | Client | Execute a `.xgb` batch file sequentially and exit. Two execution modes — direct in-process (CLI binary) and pipe-dispatched (when a resident is running). See §F.8. |
| `--node <endpoint>` / `-n` | Client | Override the Node WebSocket endpoint for this invocation. Per-subcommand override; the Node doesn't need this (it *is* the Node). |
| `--local` | Node | Start in Local Node mode regardless of config. No Trust Assertion required for registration. |
| `--port <port>` | Node | Override the listen port for this Node instance. Required on the first launch of an additional named instance to avoid port conflicts with the default instance. |

### F.0.4 Non-fundamental subcommands

Binary-specific. Listed in full detail in §F.2 (Node) and §F.3 (Client).

**Node-only:**

| Subcommand | Role |
|---|---|
| `connections` | List currently connected clients and federated peers |
| `spaces` | List every Space this Node hosts (operator view — see §F.0.5 collision note) |
| `peers` | List every known federated peer Node |
| `identity list` | List every registered Identity on this Node |

**Client-only:**

| Subcommand | Role |
|---|---|
| `register` | Register this Identity on the Node |
| `create-space` | Create a new Space; caller becomes Owner |
| `create-room` | Create a Room in a Space |
| `invite` | Invite an Identity to a Space |
| `ban` | Ban a member from a Space (Admin+; cascades across all Rooms) |
| `room-update` | Set a Room's per-Role permission overrides (Admin+; wholesale-replace) |
| `thread create` | Create a Thread in a Room (PG-08; emits `thread.create`) |
| `thread resolve` | Mark a Thread Resolved (Admin+; emits `thread.resolved`) |
| `thread archive` | Mark a Thread Archived (Admin+; emits `thread.archived`) |
| `join` | Join a Space (accept invite or join an open Space) |
| `send` | Send a message to a Room — a `message.text` (`--text`) **or** one-or-more file attachments (`--attach`, repeatable; M12.2a) |
| `history` | Fetch and display Room message history in causal order |
| `fetch` | Fetch a Room's message attachments to a local directory (alias `fetch-attachments`; M12.2a) |
| `self` | Open the personal "Saved Messages" self-thread (create-if-absent; M11, D-021) |
| `spaces` | List Spaces this Identity has joined (membership view — see §F.0.5) |
| `rooms` | List Rooms within a Space (shipped M6 Phase 1, R1) |
| `members` | List members of a Space (**deferred** — not a zero-network local read; `xgen-client_state.json` persists no per-member data. Needs either a Node query or a state-schema expansion; re-enters as its own scoped piece. M6 Phase 1 decision, 2026-05-29) |
| `federate` | Initiate federation for a Space with a peer Node (**deferred to M6 Phase 7** — co-designed with Node-side federation management; A1/R2) |
| `ai delegate` | Transfer the operator role for an AI Identity within a Space (M3) |
| `ai revoke` | Clear an explicit operator delegation (M3) |
| `ai status` | Query the currently resolved operator for an (AI, Space) pair (M3) |
| `smoke-test` | Run the Phase 1 17-step smoke test |
| `smoke-ph2` | Run the Phase 2 integrated smoke test |
| `stress-test` | Run the Phase 1 stress test |
| `stress-complete` | Run the Full Integration Stress Test (Scenarios 0–5) |

### F.0.5 The `spaces` name collision

The `spaces` subcommand exists on both binaries but is **not** a fundamental subcommand despite the shared name. The two implementations answer different questions and read from different data sources.

| Aspect | `xgen-node spaces` | `xgen-client spaces` |
|---|---|---|
| **Role perspective** | Operator — "what is this Node hosting?" | User — "what is this Identity a member of?" |
| **Data source** | Node's local Space directory (every Space whose `home_node` is this Node) | Client state file (`xgen-client_state.json`) — Spaces this Identity has joined |
| **Network required** | No — reads Node storage directly | No — reads local state file |
| **Membership filter** | None — lists every Space the Node hosts regardless of who is a member | Only Spaces this Identity has joined or been invited to |
| **Includes Spaces hosted elsewhere?** | No — only locally-hosted Spaces | Yes — Spaces hosted on any Node, including federated ones the Identity joined |
| **Output focus** | Space ID, Auth Tier, member count, Room count, event count (Node bookkeeping) | Space name, role-in-Space, Room count for this Identity (membership view) |
| **Authority** | Authoritative for hosted Spaces | Cached view — may lag if the state file is not fresh |
| **Typical use** | Capacity planning, audit, operator sanity check | Identity-side navigation, finding a Space ID to act on |

**Why this lands in non-fundamental, not fundamental.** The fundamental axis is *same name AND same semantics*. `spaces` shares the name but not the semantics — running `xgen-node spaces` on a Node and `xgen-client spaces` on a Client gives two genuinely different answers, neither of which is wrong. A Node hosting ten Spaces might show ten entries even if no client on this machine has joined any of them; a Client showing two Spaces might be a member of Spaces hosted across five different Nodes. Treating these as fundamental would mislead operators into thinking the two outputs should agree or be cross-checkable. They aren't and shouldn't be.

**Why the names overlap anyway.** The shared name reflects that both views are about Spaces, and the operator/user reading either output knows which role they are in. Renaming one side (`xgen-node hosted-spaces` vs `xgen-client member-spaces`, for instance) was considered and rejected at CLI design time: the disambiguation by *binary* is sharper than disambiguation by *subcommand suffix*, since you cannot run `xgen-client spaces` thinking it is a Node command — the binary name itself disambiguates the role.

**Related: subcommand pairs with similar names but distinct semantics.** `spaces` is the most visible example. Two others worth noting in the same spirit, though they are not direct collisions:

| Node-side | Client-side | Relationship |
|---|---|---|
| `xgen-node identity list` | `xgen-client whoami` + `xgen-client status` | Both surface Identity information, but one is "every Identity registered here" (operator audit), the other is "this client's own Identity" (user introspection). Different verb shapes, no name overlap. |
| `xgen-node peers` | `xgen-client federate` | Both touch federation, but one is read-only ("what peer Nodes do I have?") and the other is write ("initiate federation for this Space with this peer"). Different verbs, no name overlap. |

These pairs share a *domain* but not a *surface name*, so they do not need the collision treatment that `spaces` does. They are listed here so a reader scanning role differences across binaries sees the full picture in one place.

### F.0.6 CLI flag precedence over config file

For any setting that can be specified both as a CLI flag and as a field in a TOML config file, the **CLI flag takes precedence**. Precedence order:

1. **CLI flag** — highest priority. The flag passed to the binary at startup wins.
2. **Config file field** — if no flag is given, the config's value applies.
3. **Default value** — if neither flag nor config supplies a value, the binary's built-in default applies.

This rule is uniform across both `xgen-node` and `xgen-client` and applies to every flag in §F.0.1 (fundamental) and §F.0.3 (non-fundamental) that has a config equivalent. It is locked in `DECISIONS.md` D-068.

**Flag-by-flag mapping** for flags that have config equivalents:

| Flag | Binary | Config equivalent | Flag wins? |
|---|---|---|---|
| `--config <path>` | both | (default search path) | Yes |
| `--node <endpoint>` | Client | `[client].node` | Yes |
| `--log-level <lvl>` | both | `[logging].level` and `XGEN_LOG` env | Yes (flag > env > config > default) |
| `--instance <label>` | both | (implicit default-instance behaviour) | Yes |
| `--service` | both | (Tauri shell default) | Yes (flag forces headless) |
| `--local` | Node | `[node].local_mode` | Yes (one-way override — flag forces `true`; flag-absent defers to config) |
| `--port <port>` | Node | `[node].listen` (port component) | Yes |
| `--quiet` | both | (default banner behaviour) | Yes |
| `--ai-mode` | Client | `[ai].is_ai` (read at startup) | Yes (flag is the runtime selector; `[ai]` config provides the registration declaration) |

Flags listed in §F.0.1 and §F.0.3 that have *no* config equivalent (e.g. `--check-config`, `--print-config`, `--pid`, `--ping`, `--health`, `--stop`, `--reload-config`, `--batch`, `--help`, `--version`) are dispatch-only flags; they trigger an action rather than selecting a value, and the precedence rule does not apply to them.

**Audit closed — J-079 (2026-05-17).** The CLI Precedence Audit (`tasks/CLI_PRECEDENCE_AUDIT.md`) shipped on this date. Empirical verification across every row above confirms each cell. Five violations were surfaced and fixed:

1. **`xgen-node --port`** — flag was structurally orphaned from the bind path (`cli.port` never threaded into `run_node`). Fixed: `RunNodeOpts` gains `port_override`; bind site resolves via `xgen-common::precedence::resolve_setting`.
2. **`xgen-client --service` log-level** — bespoke subscriber init bypassed `[logging].level`.
3. **`xgen-client --service --ai-mode` log-level** — same defect, same site shape.
4. **`xgen-client` (Tauri shell) log-level** — same.
5. **`xgen-node` (Tauri shell) log-level** — same.

Fix for #2–#5: four parallel subscriber-init blocks converged on the new helper `xgen-common::precedence::resolve_log_level`, which bakes in `XGEN_LOG` awareness and reads `config.logging.level`. The two previously-compliant subscriber-init paths (Node `run_node`, Client short-lived CLI commands) were refactored onto the same helper for consistency, regression-locked by the integration tests in `xgen-node/tests/precedence.rs` and `xgen-client/tests/precedence.rs`. The drift surface that produced these violations is architecturally eliminated.

**Why the rule is locked rather than informal:** the testing model (smoke tests, stress tests, multiparty scenarios) depends on flag overrides being reliable. A flag that silently falls back to config makes every test that varies that flag potentially unreliable. The rule must be enforced uniformly so test results are trustworthy. See D-068 for the full reasoning.

---

## F.1 Configuration file reference

### xgen-node_config.toml

Full example with all supported fields:

```toml
[node]
listen = "ws://127.0.0.1:8080/xgen"
local_mode = true

[paths]
keypair_path = "./xgen-node_keypair.enc"

[logging]
level = "info"
```

| Field | Section | Description |
|---|---|---|
| `listen` | `[node]` | WebSocket endpoint the Node binds to |
| `local_mode` | `[node]` | `true` = no Trust Assertion required for registration |
| `keypair_path` | `[paths]` | Path to the encrypted keypair file. Default: `./xgen-node_keypair.enc` |
| `level` | `[logging]` | Log verbosity: `off` / `error` / `warn` / `info` / `debug` / `trace`. Default: `info` |

All other data paths (`spaces/`, `logs/`, `audit/`) are derived automatically from the Node's working directory and are not configurable (D-035).

### xgen-client_config.toml

Full example with all supported fields, including the M3+M4 `[ai]` family:

```toml
[client]
node = "ws://127.0.0.1:8080/xgen"

[paths]
keypair_path = "./xgen-client_keypair.enc"

[logging]
level = "debug"

# [ai] is present only when this client is staged as an AI Identity
# (see `xgen-client init --ai`). Absent for human clients.
[ai]
is_ai = true
plugin = "echo"           # M4: which AiBehavior impl to load

[ai.capabilities]         # M3: capability flags
dm_initiate = false
spontaneous_post = false

[ai.behavior]             # M4: per-plugin config sub-table
mention_token = "@bob"    # optional; plugin-specific
```

**Core sections:**

| Field | Section | Description |
|---|---|---|
| `node` | `[client]` | Default Node endpoint — used when `--node` is not provided |
| `keypair_path` | `[paths]` | Path to the encrypted keypair file |
| `level` | `[logging]` | Log verbosity: same values as Node (`off`/`error`/`warn`/`info`/`debug`/`trace`) |

All other data paths (`spaces/`, `logs/`, `audit/`, state file) are derived automatically from the Client's working directory and are not configurable (D-035).

**M3+M4 `[ai]` family** (present only when the client is staged as AI — `init --ai`):

| Field | Sub-table | Required when `[ai]` present | Description |
|---|---|---|---|
| `is_ai` | `[ai]` | Yes | The AI declaration itself (spec 3.6.10). Must be `true` for an AI-mode resident to start. |
| `plugin` | `[ai]` | Required for `--ai-mode` | Names the `AiBehavior` implementation loaded at AI-resident startup. M4 ships `"echo"`. Unknown values cause the runtime to refuse to start. |
| `dm_initiate` | `[ai.capabilities]` | No (default `false`) | Capability flag: may this AI initiate DM Spaces? Phase 2 protocol-enforced. |
| `spontaneous_post` | `[ai.capabilities]` | No (default `false`) | Capability flag: may this AI post without being addressed? Phase 2 informational; Node-side enforcement deferred to Phase 3. |
| `<extra capability keys>` | `[ai.capabilities]` | No | Forward-compatibility — unknown capability keys are tolerated, stored verbatim, and round-tripped to the Node. |
| `mention_token` | `[ai.behavior]` | No (default unset) | Optional alias the plugin treats as a mention (e.g. `"@bob"`). The AI's full `identity_id` URI is always a mention regardless; `mention_token` adds a second OR'd rail. Case-sensitive (RFC 3986). |
| `<extra behavior keys>` | `[ai.behavior]` | No | Forward-compatibility — unknown plugin-specific keys are tolerated and round-tripped. |

The split between `plugin = "..."` (in `[ai]`) and `[ai.behavior]` is deliberate: "which plugin" is a single-line toggle; "how that plugin is tuned" lives in its own namespace. Future plugins each document which `[ai.behavior]` keys they consume.

An M3-staged config (no `plugin` field, no `[ai.behavior]` sub-table) keeps working for non-AI-resident operations (e.g. `xgen-client whoami`); the AI resident only errors at startup if `--ai-mode` is set without `plugin`.

---

## F.2 xgen-node — Complete command reference

```
xgen-node [OPTIONS] [COMMAND]
```

When invoked with no subcommand, starts the Node in foreground mode.

**Global options:**

| Option | Short | Description |
|---|---|---|
| `--config <path>` | `-c` | Config file path. Default: `./xgen-node_config.toml` |
| `--local` | | Start in Local Node mode regardless of config |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version and build info |

**Subcommands:**

| Command | Description |
|---|---|
| `init` | Generate keypair and default config in current directory |
| `status` | Print current Node status from state file |
| `connections` | List all connected clients and federated peers |
| `spaces` | List all hosted Spaces with Rooms and event counts |
| `peers` | List all known federated peer Nodes |
| `identity list` | List all registered Identities |
| `version` | Print version and build metadata |

---

## F.3 xgen-client — Complete command reference

```
xgen-client [OPTIONS] [COMMAND]
```

When invoked with no subcommand and no fundamental flag, opens the Tauri desktop shell (M1+, D-056/D-062). Pass `--service` for the equivalent of the pre-M1 "just run the binary" behaviour.

See §F.0 for the full fundamental/non-fundamental flag taxonomy. Tables below are role-deep references repeating the shared fundamental surface alongside Client-specific surface for one-stop reading.

**Fundamental options** (shared with `xgen-node`):

| Option | Short | Description |
|---|---|---|
| `--config <path>` | `-c` | Config file path. Default: `./xgen-client_config.toml` |
| `--instance <label>` | | Named instance. Drives data-dir and pipe name (D-043). |
| `--log-level <lvl>` | | Override the effective tracing level. |
| `--quiet` | | Suppress startup banner. |
| `--check-config` | | Validate config, print result, exit. |
| `--print-config` | | Print effective config as TOML, exit. |
| `--pid` | | Print resident PID from PID file, exit. |
| `--ping` | | Round-trip noop against resident pipe, exit. |
| `--health` | | One-line liveness summary from resident, exit. AI-mode residents extend the format — see below. |
| `--stop` | | Signal resident to shut down, exit. |
| `--reload-config` | | Signal resident to reload config, exit (`NOT_IMPLEMENTED` today). |
| `--service` | | Force headless resident mode (no UI). |
| `--help` / `-h` | | Print help. |
| `--version` / `-V` | | Print version and build metadata. |

**Non-fundamental options** (Client-only):

| Option | Short | Description |
|---|---|---|
| `--node <endpoint>` | `-n` | Node WebSocket endpoint. Overrides config. Global — applies to network subcommands. |
| `--batch <file.xgb>` | | Execute a `.xgb` batch file. See §F.8 for the two execution modes. |
| `--ai-mode` | | Run the `--service` resident as an AI Client (M4). Requires `--service`; clap rejects standalone uses. |

**Subcommands:**

| Command | Arguments | Network? | Description |
|---|---|---|---|
| `init` | `[--passphrase <pw>]` `[--ai]` `[--cap key=value]` | No | Generate keypair and default config. `--ai` stages this Client as an AI Identity, writing the `[ai]` section to config. `--cap` overrides a capability default (repeatable; ignored without `--ai`). |
| `whoami` | — | No | Print local Identity ID and display name from state file. |
| `status` | — | No | Print client state summary: identity, space count. |
| `register` | `--name <name>` | Yes | Register this Identity on the Node. For an AI-staged Client, sends `is_ai = true` and the capability map. Writes state file. |
| `create-space` | `--name <name>` | Yes | Create a new Space. Caller becomes owner. Updates state file. |
| `create-room` | `--space <id>` `--name <name>` | Yes | Create a Room in a Space. Updates state file. |
| `invite` | `--space <id>` `--identity <id>` `--role <role>` | Yes | Invite an Identity to a Space. |
| `ban` | `--space <id>` `--identity <id>` | Yes | Ban a member from a Space (member-initiated `membership.ban`; authority is Admin+ via the `can_ban` gate — a non-admin ban is refused at validation). Space-level: `apply_ban` cascades the removal across every Room. A banned Identity cannot rejoin (the ban dominates at resolution); a banned member's subsequent posts are rejected (non-member). |
| `room-update` | `--space <id>` `--room <id>` `--deny <role>:<perm>`… `--allow <role>:<perm>`… | Yes | Set a Room's per-Role permission overrides (`state.room_update`, PG-12; authority Admin+ via `ChangeInfo` — a non-admin update is refused at validation). `--deny` / `--allow` are repeatable; `<role>` = member\|moderator\|admin\|owner, `<perm>` = send_messages\|invite\|kick\|ban\|change_info. **Wholesale-replace: the flags set the Room's COMPLETE override set — every override not listed in this invocation is CLEARED.** To keep an existing override, repeat it. (Effect: e.g. `--deny moderator:send_messages` makes a Moderator's post in that Room `PermissionDenied`; with no override the same post is permitted by membership default.) |
| `thread create` | `--space <id>` `--room <id>` `[--title <t>]` `[--auth-tier-min <n>]` | Yes | Create a Thread in a Room (`thread.create`, PG-08; starts `Open`). Requires Room membership; `auth_tier_min` is narrow-not-widen (≥ the Room/Space tier) and ≤ the creator's tier. Prints the `thread_id` (an `xgen://thread/sha256:…`) used by `resolve`/`archive`. |
| `thread resolve` / `thread archive` | `--space <id>` `--room <id>` `--thread <id>` | Yes | Mark a Thread `Resolved` / `Archived` (`thread.resolved` / `thread.archived`). Authority is Admin+ (`ChangeInfo`) — a non-admin member's resolve/archive is refused at validation. `--thread` is the id from `thread create`. |
| `join` | `--space <id>` | Yes | Join a Space (accept invite or join an open Space). |
| `send` | `--space <id>` `--room <id>` `[--text <text>]` `[--attach <path>]`… | Yes | Send to a Room. Provide `--text` (a `message.text`) **or** one-or-more `--attach` (a `message.file` carrying the file attachment(s); `--attach` is repeatable for multi-file) — exactly one of the two; combining `--text` with `--attach` is rejected (M12.2a, D3). |
| `history` | `--space <id>` `--room <id>` `[--limit <N>]` | Yes | Fetch and display Room message history in causal order. |
| `fetch` | `--space <id>` `--room <id>` `--out-dir <dir>` | Yes | Fetch every blob attachment from a Room's messages and write each to `<out-dir>`, named from its `Descriptor` filename (overwrite on collision). Alias `fetch-attachments`. The read-side companion to `send --attach` (M12.2a, D2). |
| `self` | — | Yes | Open the personal **"Saved Messages"** thread (M11, D-021): a *self-DM* — a DM whose creator and sole invitee are the same identity, reusing your existing registered identity (no second account, no new registration). No id argument — auto-resolves the session identity. Create-if-absent: creates the `"self"`-labelled self-DM on the first call, opens it (no network round-trip) thereafter. Never federated (`DmFederationNotAllowed`); reachable from any client authenticated as you. Post/read with `send`/`history` against the returned room. |
| `spaces` | — | No | List Spaces this Identity has joined (see §F.0.5 collision note). |
| `rooms` | `--space <id>` | No | List Rooms in a Space. Shipped M6 Phase 1 (R1). |
| `members` | `--space <id>` | No | List members of a Space. **Deferred** (M6 Phase 1, 2026-05-29) — no local data source today; see the §F.3 §F.10 command-table note above. |
| `federate` | `--space <id>` `--peer <endpoint>` | Yes | Initiate federation for a Space with a peer Node. **Deferred to M6 Phase 7** (federation management). |
| `ai delegate` | `--space <id>` `--ai <id>` `--to <member-id>` | Yes | Transfer the operator role for an AI Identity in a Space (M3, D-064). Signer must be Space owner or admin. Emits `state.ai_operator_delegate`. |
| `ai revoke` | `--space <id>` `--ai <id>` | Yes | Clear an explicit operator delegation for an AI Identity in a Space (M3). Resolution falls through to the AI's inviter, then to the Space owner. Signer must be owner or admin. Emits `state.ai_operator_revoke`. |
| `ai status` | `--space <id>` `--ai <id>` | Yes | Print the currently resolved operator for an (AI, Space) pair as seen by the queried Node. Connects via WS, replays the Space's DAG locally, applies the fall-upward resolution function, prints the result with provenance (stored delegation / inviter fallback / owner fallback). |
| `smoke-test` | `--node-a <ep>` `--node-b <ep>` | Yes | Run the Phase 1 17-step smoke test. |
| `smoke-ph2` | `--node-a <ep>` `--node-b <ep>` | Yes | Run the Phase 2 integrated smoke test — all Phase 1 and Phase 2 layers end-to-end. PASS/FAIL output per step. |
| `stress-test` | (see `--help`) | Yes | Run the Phase 1 stress test — concurrent multi-identity load. |
| `stress-complete` | (see `--help`) | Yes | Run the Full Integration Stress Test (Scenarios 0–5). |
| `version` | — | No | Print version and build metadata. |

**Role values for `--role`:** `owner` / `admin` / `moderator` / `member`

### F.3.1 `--health` reply format

For a human-mode resident:

```
HEALTHY pid=<pid> mode=human
```

For an AI-mode resident (M4 extension):

```
HEALTHY pid=<pid> mode=ai operator_known=<known>/<total>
```

Where `<known>` is the number of Spaces this AI is a member of with a resolvable operator (via `resolve_operator`), and `<total>` is the number of Spaces this AI is a member of. A coarse signal — `operator_known=2/3` tells the operator at a glance that one Space is in orphan state without forcing a follow-up `status` call.

The `mode=` field is always present and disambiguates human vs AI residents binding to the same pipe space.

### F.3.2 `init --ai` and `--cap` semantics

`--ai` flips the `init` flow to stage the Client as an AI Identity:

- Writes an `[ai]` section to `xgen-client_config.toml` with `is_ai = true`.
- Defaults `plugin = "echo"` and writes an empty `[ai.behavior]` table.
- Defaults `dm_initiate = false` and `spontaneous_post = false` under `[ai.capabilities]`.
- A subsequent `xgen-client register` sends `is_ai = true` plus the capability map to the Node (spec 3.6.10).

`--cap key=value` overrides a capability default. Repeatable. Form: `--cap dm_initiate=true`. Recognised keys per Phase 2 are `dm_initiate` and `spontaneous_post`; additional keys are tolerated and round-tripped verbatim to the Node (forward compat). `--cap` is ignored unless `--ai` is also given.

Re-running `init --ai` upserts the `[ai]` section without clobbering other config fields. Running `init` (without `--ai`) on a config that already has `[ai]` leaves the section untouched.

---

## F.4 Usage examples — Node operator workflows

### F.4.1 First-time Node setup

```
mkdir E:\XGen\XGenNode_A
cd E:\XGen\XGenNode_A
xgen-node init
```

Output:
```
Generating keypair...
Passphrase: ********
Confirm:    ********
Keypair saved:  ./xgen-node_keypair.enc
Config saved:   ./xgen-node_config.toml
Node ID: xgen://pubkey/ed25519:Cazue8SnVwub0khckedqjHwtwjP8WLGuJOdj6h1bA68
Run 'xgen-node' to start.
```

Then edit `xgen-node_config.toml` to set the listen address and logging level, then:

```
xgen-node
```

### F.4.2 Check Node status while running

In a second terminal, from the Node's working directory:

```
xgen-node status
```

Output:
```
xgen-node status
================
Node ID:      xgen://pubkey/ed25519:Cazue8...
Version:      0.10.3 (build 260429-2152)
Uptime:       0h 14m 22s
Mode:         Local Node
Endpoint:     ws://127.0.0.1:8080/xgen
Connections:  2 clients, 1 federated peer
Spaces:       1 hosted
Events:       47 total across all spaces
```

### F.4.3 List all registered identities on the Node

```
xgen-node identity list
```

Output:
```
Registered Identities (2)

  xgen://pubkey/ed25519:MiB5Ew...   Alice    registered 14m ago   1 device
  xgen://pubkey/ed25519:GRqRup...   Bob      registered  9m ago   1 device
```

### F.4.4 Check active federation

```
xgen-node peers
```

Output:
```
Federated Peers (1)

  Node ID:     xgen://pubkey/ed25519:kU90as...
  Endpoint:    ws://127.0.0.1:8081/xgen
  State:       ACTIVE
  Spaces:      SmokeTestSpace
  Connected:   14m 20s ago
  Last seen:   3s ago
```

### F.4.5 Enable debug logging temporarily

Change `level = "debug"` in `xgen-node_config.toml` and restart, or use the environment override without restarting:

```
set XGEN_LOG=debug
xgen-node
```

Module-specific debug (federation only):
```
set XGEN_LOG=xgen_node_lib::federation=debug,info
xgen-node
```

Restore to normal:
```
set XGEN_LOG=
xgen-node
```

---

## F.5 Usage examples — Identity and Space workflows

### F.5.1 Set up a new client identity

```
mkdir E:\XGen\XGenClient_Alice
cd E:\XGen\XGenClient_Alice
xgen-client init
```

Output:
```
Generating keypair...
Passphrase: ********
Confirm:    ********
Keypair saved:    ./xgen-client_keypair.enc
Config saved:     ./xgen-client_config.toml
Identity ID: xgen://pubkey/ed25519:MiB5EwRXFRzccw...
Run 'xgen-client register --name "Your Name"' to register on a Node.
```

### F.5.2 Register identity on a Node

```
xgen-client --node ws://127.0.0.1:8080/xgen register --name "Alice"
```

Output:
```
Registered.
Identity ID: xgen://pubkey/ed25519:MiB5EwRXFRzccw...
```

Check local identity state:
```
xgen-client whoami
```

Output:
```
Identity ID:    xgen://pubkey/ed25519:MiB5EwRXFRzccw...
Display name:   Alice
Registered on:  ws://127.0.0.1:8080/xgen
Spaces joined:  0
```

### F.5.3 Create a Space and a Room

```
xgen-client --node ws://127.0.0.1:8080/xgen create-space --name "Project Alpha"
```

Output:
```
Space created.
Space ID: xgen://hash/sha256:9ba66d487573c1f64f1f14976efe9b0f8a016c44df805563cf541bb2d35f094c
```

```
xgen-client --node ws://127.0.0.1:8080/xgen create-room \
  --space xgen://hash/sha256:9ba66d487573... \
  --name "general"
```

Output:
```
Room created.
Room ID: xgen://hash/sha256:9cb9acbef9720c6fb0f1846e773c0d8ea97773db85b5d7976c249557938f7853
```

### F.5.4 Invite another Identity to the Space

Alice invites Bob (Bob's identity_id obtained from Node identity list or out-of-band):

```
xgen-client --node ws://127.0.0.1:8080/xgen invite \
  --space xgen://hash/sha256:9ba66d487573... \
  --identity xgen://pubkey/ed25519:GRqRupb9GudeJu5xiXtWiISwPOYo28nNcbkU1sdn3FM \
  --role member
```

Output:
```
Invite sent.
```

### F.5.5 Accept an invitation and join the Space (Bob's side)

From Bob's client directory, on Node B:

```
xgen-client --node ws://127.0.0.1:8081/xgen join \
  --space xgen://hash/sha256:9ba66d487573...
```

Output:
```
Joined Space: Project Alpha
```

### F.5.6 List Spaces, Rooms, and members

```
xgen-client spaces
```

Output:
```
Spaces (1)

  Project Alpha
  ID:      xgen://hash/sha256:9ba66d487573...
  Node:    ws://127.0.0.1:8080/xgen
  Role:    owner
  Rooms:   1
  Members: 2
```

```
xgen-client rooms --space xgen://hash/sha256:9ba66d487573...
```

Output:
```
Rooms in Project Alpha (1)

  general
  ID: xgen://hash/sha256:9cb9acbef972...
```

> **`members` deferred (M6 Phase 1 decision, 2026-05-29).** The output below is the *target shape*, not a currently-shippable command. `members` has no zero-network local data source — `xgen-client_state.json` / `KnownSpace` persist no per-member data (only a Node-side member count). Producing this output needs either a Node query or a client state-schema expansion; the membership-source design is deferred to its own scoped piece, sequenced near Phase 7. See `tasks/M6_CLIENT_MEMBERS_DESIGN.md`.

```
xgen-client members --space xgen://hash/sha256:9ba66d487573...
```

Output (target shape — not yet implemented):
```
Members of Project Alpha (2)

  xgen://pubkey/ed25519:MiB5Ew...   Alice   owner    registered 14m ago
  xgen://pubkey/ed25519:GRqRup...   Bob     member   registered  9m ago
```

### F.5.7 Send messages

Alice sends a message:

```
xgen-client --node ws://127.0.0.1:8080/xgen send \
  --space xgen://hash/sha256:9ba66d487573... \
  --room xgen://hash/sha256:9cb9acbef972... \
  --text "Hello Bob, welcome to Project Alpha!"
```

Output:
```
Message sent.
Event ID: xgen://hash/sha256:e97c46b1e8d842a0cedd9862dabc66a82869744e50c2a43a67ef5d4fadf39434
```

Bob replies from Node B:

```
xgen-client --node ws://127.0.0.1:8081/xgen send \
  --space xgen://hash/sha256:9ba66d487573... \
  --room xgen://hash/sha256:9cb9acbef972... \
  --text "Hello Alice! Glad to be here."
```

Output:
```
Message sent.
Event ID: xgen://hash/sha256:9179066b77712b33a010787c3054b924ed8535edae0052260ee70a0f3d0814c4
```

### F.5.8 Establish federation between two Nodes

Alice (Space owner on Node A) initiates federation so Node B can participate in the Space:

```
xgen-client --node ws://127.0.0.1:8080/xgen federate \
  --space xgen://hash/sha256:9ba66d487573... \
  --peer ws://127.0.0.1:8081/xgen
```

Output:
```
Federation initiated.
Peer Node ID: xgen://pubkey/ed25519:kU90as7POYA8w9yRANpsFdYjjP32LJKpyUCQau6qAUA
Events synced: 4
```

---

## F.6 Usage examples — Two-Node full session

This example shows the complete sequence for a real two-Node session from scratch. It mirrors the Phase 1 smoke test.

**Prerequisites:** Node A running on port 8080, Node B running on port 8081. Alice registered on Node A, Bob registered on Node B.

```bash
# -- Alice (Node A) --

# Create the Space
xgen-client --node ws://127.0.0.1:8080/xgen create-space --name "SmokeTestSpace"
# → Space ID: xgen://hash/sha256:9ba66d48...

# Create a Room
xgen-client --node ws://127.0.0.1:8080/xgen create-room \
  --space xgen://hash/sha256:9ba66d48... \
  --name "general"
# → Room ID: xgen://hash/sha256:9cb9acbe...

# Invite Bob
xgen-client --node ws://127.0.0.1:8080/xgen invite \
  --space xgen://hash/sha256:9ba66d48... \
  --identity xgen://pubkey/ed25519:GRqRup... \
  --role member

# Federate with Node B
xgen-client --node ws://127.0.0.1:8080/xgen federate \
  --space xgen://hash/sha256:9ba66d48... \
  --peer ws://127.0.0.1:8081/xgen

# -- Bob (Node B) --

# Join the Space
xgen-client --node ws://127.0.0.1:8081/xgen join \
  --space xgen://hash/sha256:9ba66d48...

# Join the Room
xgen-client --node ws://127.0.0.1:8081/xgen join \
  --space xgen://hash/sha256:9ba66d48... \
  --room xgen://hash/sha256:9cb9acbe...

# -- Exchange messages --

# Alice sends
xgen-client --node ws://127.0.0.1:8080/xgen send \
  --space xgen://hash/sha256:9ba66d48... \
  --room xgen://hash/sha256:9cb9acbe... \
  --text "Hello Bob"

# Bob replies
xgen-client --node ws://127.0.0.1:8081/xgen send \
  --space xgen://hash/sha256:9ba66d48... \
  --room xgen://hash/sha256:9cb9acbe... \
  --text "Hello Alice"
```

---

## F.7 `--help` output requirements

Every command and subcommand MUST produce useful `--help` output generated from Rust doc comments via `clap` derive API. The following format is required for all subcommands:

```
xgen-client-send
Send a text message to a Room

USAGE:
    xgen-client [OPTIONS] send [SEND OPTIONS]

OPTIONS:
    --space <space-id>    Space ID (xgen://hash/sha256:...)
    --room  <room-id>     Room ID (xgen://hash/sha256:...)
    --text  <text>        Message text to send

EXAMPLES:
    xgen-client --node ws://127.0.0.1:8080/xgen send \
        --space xgen://hash/sha256:9ba66d48... \
        --room  xgen://hash/sha256:9cb9acbe... \
        --text "Hello!"

    -h, --help    Print help information
```

Requirements:
- Every argument has a description
- Every subcommand has at least one `EXAMPLES:` entry
- The `EXAMPLES:` section uses real, runnable commands — not placeholders like `<YOUR_SPACE_ID>`
- The examples in `--help` MUST match the examples in this appendix exactly

---

## F.8 Multi-instance and batch operation

### F.8.1 Named instances — `--instance`

`xgen-client-app.exe` can run as multiple simultaneous named instances. Each instance is a fully independent protocol client with its own keypair, state file, and log directory. From the protocol's perspective, five instances on one machine are identical to five clients on five different machines.

```
xgen-client-app.exe --instance alice
xgen-client-app.exe --instance bob
xgen-client-app.exe --instance bot_01
```

Instance data is stored under `instances/<label>/` relative to the executable. With no `--instance` flag, data is stored in the executable directory (default, backward-compatible).

`xgen-node-app.exe` supports the same flag. Node instances additionally require `--port` on first launch to avoid port conflicts:

```
xgen-node-app.exe --instance node_a --port 8080
xgen-node-app.exe --instance node_b --port 8081
```

**Label rules:** alphanumeric characters, hyphens, and underscores only. Maximum 64 characters. Invalid labels print an error and exit immediately.

### F.8.2 Batch command files — `--batch`

A second invocation of `xgen-client-app.exe` with `--batch <file.xgb>` connects to a running instance via a named pipe, delivers the command file, waits for all commands to complete, and exits. No window is opened.

```
xgen-client-app.exe --instance alice --batch alice_setup.xgb
```

The running instance must already be started. If no running instance is found, exit code 3 is returned with a clear error message.

**Named pipe convention (D-043):**

| Invocation | Pipe |
|---|---|
| `xgen-client-app.exe` | `\\.\pipe\xgen-client` |
| `xgen-client-app.exe --instance alice` | `\\.\pipe\xgen-client-alice` |
| `xgen-node-app.exe --instance node_a` | `\\.\pipe\xgen-node-node_a` |

**Exit codes:**

| Code | Meaning |
|---|---|
| 0 | All commands completed successfully |
| 1 | A command returned an error (execution stopped at first failure) |
| 2 | Batch file path or extension invalid |
| 3 | No running instance found |

### F.8.3 `.xgb` file format

UTF-8 text. One command per line. Same syntax as interactive CLI subcommands — no binary name prefix.

```
# Lines starting with # are comments — ignored.
# Empty lines are ignored.

# Register this identity on the default node
register --name alice

# Create a space
create-space --name "Test Space"

# Send a message (use --node to override if needed)
send --space xgen://hash/sha256:9ba66d48... \
     --room  xgen://hash/sha256:9cb9acbe... \
     --text  "Batch message from alice"
```

**Available batch commands:**

| Command | Network? | What it does |
|---|---|---|
| `whoami` | No | Print Identity ID and display name from state file |
| `status` | No | Print state summary: identity, space count |
| `register --name <name>` | Yes | Auth + `identity.register` event → writes state file |
| `create-space --name <name>` | Yes | Auth + `state.space_create` event → updates state file |
| `create-room --space <id> --name <name>` | Yes | Auth + `state.room_create` event → updates state file |
| `invite --space <id> --identity <id> --role <role>` | Yes | Auth + `membership.invite` event |
| `join --space <id>` | Yes | Auth + `membership.join` event |
| `send --space <id> --room <id> --text <text>` | Yes | Auth + `message.text` event |

The `--node <endpoint>` flag is available on all network commands and overrides the config file.

**Send-confirm semantics (MP-F1a).** As of MP-F1a, every network command awaits the Node's deterministic per-event outcome (`EventAccepted` / `Error`, correlated by `event_id`) before it proceeds to the next command or exits — it no longer fire-and-forgets the event then disconnects. Consequences for batch runs:

- A Node **rejection** of a command's event now surfaces as a command error: the batch stops at first failure and returns **exit 1** (§F.8.2 / §F.8.5). Previously some rejections were acknowledged optimistically and could pass as exit 0; they now report honestly ("honest behaviour over polite behaviour", D-065).
- A command that emits **multiple events** (e.g. a DM-create chain) aborts and errors if any event in the chain is rejected or times out unconfirmed; any local state write happens only after the whole chain is confirmed.
- A single-event command that times out **unconfirmed** — no `EventAccepted` and no `Error` within the confirm window (e.g. a held-pending event, which emits no signal) — warns and proceeds rather than failing, because that timeout is ambiguous (genuinely lost vs. node-held). An explicit rejection always errors.

The confirm window reuses `[sync].completion_timeout_seconds`. This is a **behavioural** change only — no CLI flag or argument changed. Per D-028, the `main.rs` `--help` doc comments for the affected verbs are kept in lockstep with this appendix.

**Reject-surfacing (MP-F5).** As of MP-F5, a single-event reject (the rejection arm of the MP-F1a send-confirm above) carries the Node's **wire reject code and the rejected `event_id` as structured fields** on the `--aicontrol` reply envelope (`reject_code` + `event_id` in the error body) — so an automation driver reads the exact code (e.g. `3030` `tier_mismatch`) instead of parsing it out of the human message text. The reply's surface `code` stays `GENERIC_4000` and `category` stays `protocol` (a control-plane code never represents a verb error — AC-D3d), with the wire semantics on the additive fields; old readers ignore them. This finishes the MP-F2 node-side surfacing (`Error` frame with code + event_id) into the client reply; it is scoped to **locally-submitted single-event** rejects. `--batch` runs are unaffected (they still report failure via exit 1, per the bullets above).



### F.8.4 Stress test setup — full example

Two nodes, two clients, scripted setup — no manual interaction:

```
# Start nodes (leave running in separate terminals)
xgen-node-app.exe --instance node_a --port 8080
xgen-node-app.exe --instance node_b --port 8081

# Start clients (leave running in separate terminals)
xgen-client-app.exe --instance alice
xgen-client-app.exe --instance bob
```

`alice_setup.xgb`:
```
# Register alice on node_a and create a space
register --name "Alice" --node ws://127.0.0.1:8080/xgen
create-space --name "StressSpace" --node ws://127.0.0.1:8080/xgen
create-room --space xgen://hash/sha256:SPACE_ID --name "general" --node ws://127.0.0.1:8080/xgen
invite --space xgen://hash/sha256:SPACE_ID --identity xgen://pubkey/ed25519:BOB_ID --role member --node ws://127.0.0.1:8080/xgen
```

`bob_setup.xgb`:
```
# Register bob on node_b and join alice's space
register --name "Bob" --node ws://127.0.0.1:8081/xgen
join --space xgen://hash/sha256:SPACE_ID --node ws://127.0.0.1:8081/xgen
```

Deliver in parallel:
```
xgen-client-app.exe --instance alice --batch alice_setup.xgb
xgen-client-app.exe --instance bob   --batch bob_setup.xgb
```

Each invocation exits 0 on success. Execution results appear in the respective instance log files under `instances/alice/logs/` and `instances/bob/logs/`.

### F.8.5 CLI binary batch mode — `xgen-client --batch`

`xgen-client.exe` (the non-Tauri CLI binary) supports `--batch` as a **direct sequential executor**. No running instance is required. Each line in the `.xgb` file is parsed as a CLI subcommand and dispatched immediately, in process.

```
xgen-client --node ws://127.0.0.1:8080/xgen --batch test/setup.xgb
```

The global `--node` specified on the invocation is **inherited** by all network commands in the file. Individual lines do NOT repeat `--node` unless they need to override the global for that specific command.

**Example `.xgb` file for CLI binary:**

```
# No --node per line — inherited from invocation --node flag
register --name "Alice"
create-space --name "TestSpace"
whoami
status
```

**Exit codes:**

| Code | Meaning |
|---|---|
| 0 | All commands completed successfully |
| 1 | A command returned an error (stops at first failure) |
| 2 | File not found or extension is not `.xgb` |

**Distinction from Tauri app batch mode (§F.8.2):**

| Property | CLI binary (`xgen-client`) | Tauri app (`xgen-client-app`) |
|---|---|---|
| Running instance required | No | Yes |
| IPC mechanism | None — direct in-process dispatch | Named pipe (`\\.\pipe\xgen-client[-label]`) |
| Window opened | No | No |
| `smoke-ph2` allowed | No (returns error) | No |

The canonical source of argument descriptions and examples is this appendix (F.2, F.3, F.4, F.5, F.6, F.9). The Rust doc comments that generate `--help` output MUST match this appendix. When this appendix changes, the Rust source MUST be updated to match.

---

## F.9 Usage examples — AI Identity workflows

This section walks through the M3+M4 surface for staging, running, and managing an AI Identity. The narrative mirrors the M4 smoke run quoted in J-077: alice (human, on Node A) invites bob (AI) into a Space, bob's `EchoPlugin` replies to mentions, the operator role is initially alice (by inviter fallback), and a third Identity charlie demonstrates explicit delegation.

### F.9.1 Set up an AI Identity

From a fresh data directory for the AI:

```
mkdir E:\XGen\XGenClient_Bob
cd E:\XGen\XGenClient_Bob
xgen-client init --ai
```

Output:
```
Generating keypair...
Passphrase: (empty for Local Node mode)
Keypair saved:    ./xgen-client_keypair.enc
Config saved:     ./xgen-client_config.toml
Staged as AI Identity (spec 3.6.10).
  cap.dm_initiate = false
  cap.spontaneous_post = false
  plugin = "echo"
Identity ID: xgen://pubkey/ed25519:nkRTIqeu...
Run 'xgen-client register --name "Bob (AI)"' to register on a Node.
```

Resulting `xgen-client_config.toml`:

```toml
[client]
node = "ws://127.0.0.1:8080/xgen"

[paths]
keypair_path = "./xgen-client_keypair.enc"

[logging]
level = "debug"

[ai]
is_ai = true
plugin = "echo"

[ai.capabilities]
dm_initiate = false
spontaneous_post = false

[ai.behavior]
```

To override a capability default at init time:

```
xgen-client init --ai --cap dm_initiate=true
```

This would write `dm_initiate = true` under `[ai.capabilities]` instead of the default `false`.

Register bob on the Node (sends `is_ai = true` plus the capability map per spec 3.6.10):

```
xgen-client --node ws://127.0.0.1:8080/xgen register --name "Bob (AI)"
```

Output:
```
Registered as AI Identity.
Identity ID: xgen://pubkey/ed25519:nkRTIqeu...
  is_ai = true
  cap.dm_initiate = false
  cap.spontaneous_post = false
```

From this point bob is a registered AI Identity on the Node. Alice can `invite` bob to a Space using bob's `identity_id`, and bob (via the AI resident in §F.9.2) can `join` once invited. Manual join is required — the AI resident does not auto-join Spaces (§6.15.9).

### F.9.2 Run as an AI resident

Start bob's AI Client resident in a separate terminal:

```
xgen-client --instance m4-bob --ai-mode --service
```

Startup log lines (visible in `logs/xgen-client_<timestamp>.log`):

```
INFO xgen_client_lib::ai_service: ai-service: plugin loaded plugin="echo" mention_token=None identity_id=xgen://pubkey/ed25519:nkRT...
INFO xgen_client_lib::ai_service: ai-service: connecting to home Node home_node=ws://127.0.0.1:8080/xgen
INFO xgen_client_lib::ai_service: ai-service: authenticated identity_id=xgen://pubkey/ed25519:nkRT...
```

In a third terminal, query liveness:

```
xgen-client --instance m4-bob --health
```

Output for an AI-mode resident:

```
HEALTHY pid=40136 mode=ai operator_known=1/1
```

The `operator_known=1/1` means bob is a member of one Space and that Space has a resolvable operator (via M3's `resolve_operator` fall-upward chain). If bob were in three Spaces and only two had resolvable operators, the reply would read `operator_known=2/3` — a coarse signal of orphan state without forcing a `status` follow-up.

Now exercise the plugin. From alice's terminal:

```
xgen-client --node ws://127.0.0.1:8080/xgen send \
  --space xgen://hash/sha256:2ccf... \
  --room  xgen://hash/sha256:d85d... \
  --text  "hello xgen://pubkey/ed25519:nkRT...wBpQk, are you there?"
```

After a few seconds, alice queries history:

```
xgen-client history --space xgen://hash/sha256:2ccf... --room xgen://hash/sha256:d85d...
```

Output:
```
History for room d85d05ed... (2 messages)
  [kFluTpiB...]  2026-05-17T08:29:08  hello xgen://pubkey/ed25519:nkRT...wBpQk, are you there?
  [nkRTIqeu...]  2026-05-17T08:29:08  [echo-plugin] received mention from V_osISzS9wUg
```

Bob's `EchoPlugin` matched alice's `identity_id` URI in the text (rail A of mention detection — always-on, case-sensitive) and replied with the deterministic line specified in Ch6 §6.15.4. `V_osISzS9wUg` is the last 12 characters of alice's `identity_id`.

**Pacing in action (drop, not queue).** Alice sends two more mentions back-to-back:

```
xgen-client --node ws://127.0.0.1:8080/xgen send \
  --space xgen://hash/sha256:2ccf... --room xgen://hash/sha256:d85d... \
  --text "first ping for xgen://pubkey/ed25519:nkRT...wBpQk"
xgen-client --node ws://127.0.0.1:8080/xgen send \
  --space xgen://hash/sha256:2ccf... --room xgen://hash/sha256:d85d... \
  --text "second ping for xgen://pubkey/ed25519:nkRT...wBpQk right after"
```

History after the burst (5 messages total, only 2 from bob):

```
[kFluTpiB...]  2026-05-17T08:29:08  hello xgen://...wBpQk, are you there?
[nkRTIqeu...]  2026-05-17T08:29:08  [echo-plugin] received mention from V_osISzS9wUg
[kFluTpiB...]  2026-05-17T08:29:59  first ping for xgen://...wBpQk
[nkRTIqeu...]  2026-05-17T08:29:59  [echo-plugin] received mention from V_osISzS9wUg
[kFluTpiB...]  2026-05-17T08:30:00  second ping for xgen://...wBpQk right after
```

The second ping at 08:30:00 — 703ms after bob's previous reply — got no reply. Bob's structured log records the drop with the named principle:

```
2026-05-17T08:30:00.081  WARN ai_service: ai-service: dropping reply — pacing cap not yet elapsed (honest behaviour over polite behaviour) ai_pacing_ms=2000
```

This is the Ch6 §6.15.7 contract in action: the AI's pacing cap rejected the second reply, so it is dropped rather than queued. The literal phrase `(honest behaviour over polite behaviour)` makes the principle greppable in production logs (D-065).

Shut down cleanly:

```
xgen-client --instance m4-bob --stop
```

Output:
```
OK STOPPING
```

### F.9.3 Operator delegation flow

When alice invited bob into the Space, alice became bob's operator by inviter fallback (M3 fall-upward resolution — D-064, spec 3.6.10.6). No event was signed for this; the operator is *computed* from current Space membership and the original `membership.invite`. To verify:

```
xgen-client --node ws://127.0.0.1:8080/xgen ai status \
  --space xgen://hash/sha256:2ccf... \
  --ai    xgen://pubkey/ed25519:nkRT...
```

Output:
```
Resolved operator for AI in Space xgen://hash/sha256:2ccf...:
  operator: xgen://pubkey/ed25519:kFluTpiB...   (Alice)
  resolution: inviter fallback (no explicit delegation; AI was invited by this Identity)
```

Now alice delegates bob's operator role to charlie (a third Identity who is already a member of the Space):

```
xgen-client --node ws://127.0.0.1:8080/xgen ai delegate \
  --space xgen://hash/sha256:2ccf... \
  --ai    xgen://pubkey/ed25519:nkRT... \
  --to    xgen://pubkey/ed25519:cHARlieY...
```

Output:
```
Delegation signed and sent.
Event type: state.ai_operator_delegate
Event ID:   xgen://hash/sha256:...
  space:    xgen://hash/sha256:2ccf...
  ai:       xgen://pubkey/ed25519:nkRT...
  new operator: xgen://pubkey/ed25519:cHARlieY...   (Charlie)
```

Charlie verifies the delegation took effect:

```
xgen-client --node ws://127.0.0.1:8080/xgen ai status \
  --space xgen://hash/sha256:2ccf... \
  --ai    xgen://pubkey/ed25519:nkRT...
```

Output:
```
Resolved operator for AI in Space xgen://hash/sha256:2ccf...:
  operator: xgen://pubkey/ed25519:cHARlieY...   (Charlie)
  resolution: stored delegation (state.ai_operator_delegate signed by Alice)
```

If charlie later leaves the Space without an explicit revoke, the fall-upward resolution silently reverts to alice (inviter fallback) — no manual cleanup needed (spec 3.6.10.6, D-064). This is one of the named instances of "honest behaviour over polite behaviour" referenced in Ch6 §6.15.7: the system reports the *currently* resolvable operator, not a stale stored value.

To explicitly clear the delegation (returning to inviter fallback):

```
xgen-client --node ws://127.0.0.1:8080/xgen ai revoke \
  --space xgen://hash/sha256:2ccf... \
  --ai    xgen://pubkey/ed25519:nkRT...
```

Output:
```
Revocation signed and sent.
Event type: state.ai_operator_revoke
Event ID:   xgen://hash/sha256:...
Operator now resolves to: xgen://pubkey/ed25519:kFluTpiB...   (Alice, via inviter fallback)
```

**Authority for `ai delegate` / `ai revoke`.** Signer must be a Space owner or admin (D-064). Attempts by lower-privileged Identities are rejected by the Node with error code 3041 `ai_role_violation`. The delegate target must be a current Space member; non-member targets are also rejected at validation time.

**Cross-Node propagation.** `ai status` returns the queried Node's converged view. Call `ai status` against each Node in a federated Space to verify the delegation has propagated. Resolution is eventual — a freshly-signed delegation appears on the home Node immediately and on federated peers as soon as the delegate event reaches them through normal Event fanout.

---

## F.10 — `xgen-auth-module` (Tier-1 reference Auth Module) and registry-as-trust-source semantics

*(Added M10.2, J-364.)* `xgen-auth-module` is a standalone binary (its own workspace member, own keypair) that
**issues** Tier-1 `TrustAssertion`s, signing them as itself. It is an **offline signer** — it never serves a
live endpoint the Node calls; the Node verifies a presented assertion's signature offline against the trusted
issuer's public key. T1 verification attests **proof-of-key-possession only** (key control), not external KYC.

**CLI (minimal, witness-scope — not a product surface).**

```
xgen-auth-module keygen --out <path>            # generate + save the module keypair (encrypt-at-rest)
xgen-auth-module issue  --key <path> \           # self-sign a Tier-1 TrustAssertion for a subject identity
                        --subject <xgid> [--out <path>]
```

The issued assertion carries the M10.1 module-policy descriptor on `claims.extra`: `module_kind = reference`
and a `module_policy` with `erasability` (a `reference` module issues an erasable Tier-1 claim). The subject
identity attaches the assertion to its `register` message; the Node validates it through the live 7-check
`validate_assertion`.

**Registry is the trust source (M10.2 behaviour change).** As of M10.2, the Node's assertion gate **live-reads
`AuthModuleRegistry`** at validation time — the operator-managed `auth-module` CRUD verbs are now
enforcement-bearing:

- `auth-module register` makes an issuer **trusted for validation** (no separate config edit needed).
- `auth-module revoke` **takes effect immediately, without a Node restart** — a subsequent registration
  presenting that issuer's assertion is rejected.

**Config `trusted_auth_modules` is now a bootstrap-seed (D-065 honesty note).** At startup the Node seeds each
configured issuer into the registry **add-only**: it inserts an issuer only if absent, re-runs idempotently on
every boot, never un-revokes a CRUD-revoked issuer (operator revoke is authoritative), and skips malformed
entries. **Consequence:** **removing an issuer from the config no longer un-trusts it** — once seeded, the
registry rules. To withdraw trust, use `auth-module revoke` (or delete the record); editing the config alone has
no effect on an already-seeded issuer. An empty config + empty registry leaves behaviour byte-for-byte as before
(the `local_mode`/baseline path is untouched).

### F.10.1 Higher-tier mock issuance — `issue --tier <N>` (M10.3)

*(Added M10.3, J-368.)* The `issue` subcommand takes an optional `--tier <N>` flag so the same reference binary
can stand in as a **parameterized higher-tier mock** — the template an institution forks, not a deployable trust
anchor.

```
xgen-auth-module issue --key <path> --subject <xgid> [--tier <N>] [--out <path>]
```

| `--tier` | Behaviour |
|---|---|
| `1` (default / omitted) | Tier-1 **reference** assertion — unchanged from §F.10 (`module_kind = reference`, erasable). |
| `2` \| `3` \| `4` | Auto-**mock**: sets `module_kind = mock`, a grounded TTL (**T2 = 365d, T3 = 180d, T4 = 90d**), and tier-appropriate `module_policy.erasability` (**T2–T3 `erasable`, T4 `retained`**). |

The mock issues exactly what validation consumes — the tier integer, the TTL, and the descriptor; it does **not**
populate richer per-tier claim schemas (no production validator reads them, so populating them would be theater).
The `mock` label is **expression-only**: a mock is honoured only if an operator has trusted its issuer via the
`auth-module` CRUD verbs (the registry gate is the safety mechanism — a mock is never a trust anchor by virtue of
its label).

**Per-issuer tier scope + the `3032` reject (operator-visible).** An operator can scope a trusted issuer to
specific tiers via the issuer's `accepted_tiers`. At registration the Node enforces, per issuer, that the
asserted tier is within that issuer's scope (Step 1.5, **restrictive-only**: an empty/absent `accepted_tiers`
means *unrestricted*). An assertion whose tier is **outside** the issuer's scope is rejected with:

```
Error 3032 (assertion_tier_unauthorized): This Auth Module issuer is not
authorized to attest the asserted Tier.
```

This is **distinct** from `3030 tier_mismatch` ("the asserted tier is below the Space's/Node's required floor"):
3030 is a node-wide ordered floor; 3032 is a per-issuer set-membership check. A T2-scoped issuer's T2 assertion
is accepted; its T3 assertion is rejected with 3032. (3032 was renumbered from a draft 3012 to avoid colliding
with the reserved `3012 watchlist_match` — see ch3 §3.11.7.)

---

## F.11 Session log

### Session 1 — April 2026 (JozefN)
**Covered:** Appendix F written in full covering: F.1 configuration file reference for both binaries including the new `[logging]` section and the removal of `log_path`/`spaces_dir` from config (D-035); F.2 complete xgen-node command reference; F.3 complete xgen-client command reference; F.4 Node operator workflow examples (setup, status, identity list, federation check, debug logging); F.5 Identity and Space workflow examples (init, register, create-space, create-room, invite, join, spaces/rooms/members list, send message, federate); F.6 complete two-Node full session example mirroring the Phase 1 smoke test with real event_ids from J-029; F.7 `--help` output requirements and the canonical source rule (D-028).

### Session 2 — 2026-05-17 (JozefN)
**Covered:** comprehensive M2/M3/M4 documentation sweep. Header reformatted to standard template (Version 1.1, Date, Language, License lines added). New **§F.0 Flag model** introducing the fundamental-vs-non-fundamental axis (symmetry between binaries, not internal dispatch order): F.0.1 fundamental flags table, F.0.2 fundamental subcommands table, F.0.3 non-fundamental flags table, F.0.4 non-fundamental subcommands tables (Node-only and Client-only), F.0.5 the `spaces` name collision (full role-perspective table plus rejected-rename rationale plus related pairs `identity list` vs `whoami`/`status` and `peers` vs `federate`). §F.1 `xgen-client_config.toml` extended with the M3+M4 `[ai]` family (`is_ai`, `plugin`, `[ai.capabilities]` with `dm_initiate`/`spontaneous_post`, `[ai.behavior]` with `mention_token`), including the deliberate split rationale and M3-staged-config forward compat. §F.3 Client command reference fully extended: fundamental options table, non-fundamental options table (`--ai-mode` added with `requires = service` constraint, `--batch` and `--node` clarified), `init` row extended with `--ai` and `--cap`, full `ai delegate` / `ai revoke` / `ai status` rows added (M3 D-064), `history` subcommand documented, `smoke-ph2` / `stress-test` / `stress-complete` added. New §F.3.1 `--health` reply format documenting the M4 `mode=ai operator_known=N/M` extension and the `mode=` field universal presence. New §F.3.2 `init --ai` and `--cap` semantics. New **§F.9 AI Identity workflows** with three subsections: F.9.1 set up AI Identity (full `init --ai` output, resulting config TOML, register flow), F.9.2 run as AI resident (`--ai-mode --service` startup logs, `--health` reply, pacing-drop smoke quoting J-077's real transcript with the literal greppable WARN line and the 703ms inter-reply timing), F.9.3 operator delegation flow (alice as inviter-fallback operator, delegate to charlie, charlie verifies via `ai status`, alice revokes; rationale for fall-upward silent revert as instance of "honest behaviour over polite behaviour"). Duplicate §F.8 heading (collision between Multi-instance/batch and Session log) resolved — Session log renumbered to §F.10. M1 binary-consolidation preamble box removed (its content was absorbed into §F.0 and the per-section refreshes). The previous "preamble only — comprehensive example sweep pending" disclaimer in the header is now resolved — this commit *is* that sweep. Cross-references to D-028, D-035, D-043, D-056, D-062, D-063, D-064, D-065, and Ch6 §6.15 added throughout.

### Session 3 — 2026-06-13 (JozefN)
**Covered:** new **§F.10** documenting the `xgen-auth-module` Tier-1 reference binary (offline signer; `keygen`/`issue` CLI; the M10.1 `module_kind`/`module_policy` descriptor on issued assertions) and the M10.2 **registry-as-trust-source** behaviour change (the gate live-reads `AuthModuleRegistry`; `auth-module register`/`revoke` are now enforcement-bearing; `revoke` bites without restart). D-065 honesty note recorded: config `trusted_auth_modules` is now an add-only/idempotent/revoke-wins **bootstrap-seed**, so removing an issuer from config no longer un-trusts it ("registry rules" — use `auth-module revoke` to withdraw trust); empty config + empty registry = prior behaviour byte-for-byte. Session log renumbered §F.10→§F.11. Cross-refs to D-069/D-083/AMR-D1 and the M10.2 design/audit (J-363/J-364).

### Session 4 — 2026-06-14 (JozefN)
**Covered:** new **§F.10.1** documenting the M10.3 parameterized higher-tier mock issuance — the `issue --tier <N>` flag (N=1 reference default; N∈{2,3,4} auto-sets `module_kind = mock` + grounded TTL T2=365/T3=180/T4=90 + tier-appropriate `erasability`, T4 `retained`), the expression-only nature of the `mock` label (the registry CRUD gate remains the safety mechanism, not the label), and the operator-visible per-issuer **`accepted_tiers` scope** with its **`3032 assertion_tier_unauthorized`** reject (restrictive-only; distinct from the node-floor `3030 tier_mismatch`). D-065 note: the reject code was renumbered **3012 → 3032** at close (J-367 catch) — the M10.3 design's 3012 collided with the reserved `3012 watchlist_match` in ch3 §3.11.7; `watchlist_match` keeps 3012, the auth-tier-authz code moved to 3032 (adjacent to the 3030/3031 tier-authz band). Cross-refs to ch3 §3.11.7 and the M10.3 design/audit (J-366/J-368).

### Session 5 — 2026-06-14 (JozefN)
**Covered:** new `self` Client subcommand added to the command tables (F.0.4 Client-only + F.3 detailed reference) — the M11 (D-021) personal **"Saved Messages"** self-thread: a self-DM reusing the user's existing identity as both endpoints (no second account, no new registration), auto-resolving the session identity (no id argument), create-if-absent (creates the `"self"`-labelled self-DM on the first call, opens thereafter), never federated (`DmFederationNotAllowed`), reachable from any client authenticated as the user; post/read via the existing `send`/`history`. Caught at the M11 close as a missing close deliverable (the thin-verb-arc Appendix-F convention, J-334; the runbook §6 close list had omitted it). Cross-refs to ch6 §6.16 and the M11 design/close (J-377/J-378). (Note: the underlying `create-dm-space` verb remains undocumented in this appendix — a pre-existing gap, out of M11 scope.)

### Session 6 — 2026-06-15 (JozefN)
**Covered:** new `fetch` Client subcommand added to the command tables (F.0.4 Client-only + F.3 detailed reference) — the M12.2a read-side companion to `send --attach`: fetches every blob attachment from a Room's messages and writes each to `--out-dir` (required), named from its `Descriptor` filename (overwrite on collision); alias `fetch-attachments`; wired across all four dispatch arms (CLI / run-path / batch / aicontrol). The `send` rows (F.0.4 + F.3) were updated to the final M12.2a surface: `--text` is now optional and `--attach <path>` is repeatable (multi-file), with exactly-one-of-the-two enforced (combining `--text` and `--attach` is rejected — VC, D-065 no-quiet-data-loss). Part of the M12 attachments arc (M12.2a = fetch verb + `--attach` polish + the F6 blob-size gate; D2+D3+D4). The illustrative `.xgb` batch table (§F.8.3) was not re-tabulated — §F.0.4/§F.3 are the canonical command authority (D-028). Cross-refs to the M12.2 design (J-383) and the M12.2a close (J-384).
