# Task — CLI Flag Precedence Audit (D-068)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-17  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

Enforce the rule locked in **D-068**: every CLI flag — on every binary, without exception — takes precedence over the config file. The known violation `xgen-node --port` (J-078) is one instance of a structural problem; this task closes the structural problem, not just the one symptom.

**This task blocks M6.** M6 (multiparty baseline pass) measures behaviour across many flag-driven invocations. If flag precedence is unreliable, M6 metrics are unreliable. The audit must complete before M6 starts.

**This task covers BOTH binaries with symmetric rigour.** The known violation is on `xgen-node`, but the same audit, the same tests, and the same manual verification are required on `xgen-client`. Fixing only the Node is not sufficient — the Client side must be empirically confirmed compliant or fixed if not, with the same evidence quality.

---

## §1 — Mandatory reading

Read in this order before touching code. The whole audit reads down to two architectural artifacts (D-068 and Appendix F §F.0); everything else is supporting context.

| Source | What it gives | Why read it |
|---|---|---|
| `DECISIONS.md` D-068 | The rule itself, locked. Full reasoning, scope, the named violation, the audit task list. | The authority. Everything else conforms to this. |
| `docs/xgen_appendix_f_en.md` §F.0 (full) | The complete CLI surface — fundamental flags, fundamental subcommands, non-fundamental flags, non-fundamental subcommands, the `spaces` name collision. | Defines the audit surface. Every flag in §F.0.1 and §F.0.3 is a candidate row. |
| `docs/xgen_appendix_f_en.md` §F.0.6 | The user-facing reference for the rule. Already shipped. Lists the 9 flag-with-config-equivalent pairs and the known violation note. | Must remain canonical and aligned with the audit outcome. |
| `docs/xgen_appendix_f_en.md` §F.1 (both config schemas) | The complete `xgen-node_config.toml` and `xgen-client_config.toml` field lists. | Defines which config fields exist — needed to identify which flags shadow which fields. |
| `JOURNAL.md` J-078 | The M5 smoke setup entry where `xgen-node --port` failed to override `[node].listen` on first invocation. The known violation. | The concrete observation that motivates the audit. |
| `CLAUDE.md` MANDATORY behaviour rules (top of file) | Rules 1–7. | Apply throughout. Quote real output (Rule 2), never invent numbers (Rule 5), ask if ambiguous (Rule 6), write JOURNAL last (Rule 4). |

After reading, the model in your head must be: **flag > env > config > default**, universal across both binaries, no exceptions, every shadowing pair empirically confirmed.

---

## §2 — The rule

Verbatim from D-068 — restated here so this task file is self-contained.

**Precedence order, applied to every setting that can be specified in more than one place:**

1. **CLI flag** — highest priority. The flag passed to the binary at startup wins.
2. **Environment variable** — applies where one exists (today: `XGEN_LOG` only).
3. **Config file field** — if no flag and no env override, the config's value applies.
4. **Default value** — if none of the above supplies a value, the binary's built-in default applies.

The rule is **uniform** across `xgen-node` and `xgen-client`. The rule is **structural**, not stylistic. Three reasons:

1. **CLI is the most-recent intent.** A config file was written at some past time; a flag is what the operator typed *right now*. Right-now intent must beat persisted intent.
2. **CLI is visible; config is hidden.** Flags appear in shell history, scripts, process listings. Config fields are buried in TOML files. The most-visible source must be the authoritative one for diagnosis and audit.
3. **The testing model depends on it.** Smoke tests, stress tests, multiparty scenarios all vary flags against a single set of config files. If a flag silently falls back to config, every test that varies that flag is unreliable — silently wrong, not loudly broken.

Reason 3 is the operational urgency. M6 needs this to be true.

---

## §3 — Root-cause the known violation first

Before auditing other flags, root-cause `xgen-node --port`. The mechanism likely tells us whether other flags share the defect.

**Known behaviour (J-078, 2026-05-17):**
- `xgen-node_config.toml` had `listen = "ws://127.0.0.1:8080/xgen"`.
- Operator launched `xgen-node --port 8081` against a separate already-running Node on `8080`.
- Node attempted to bind `127.0.0.1:8080` (config value), failed with `os error 10048` (port already in use).
- Same command on second invocation succeeded — mechanism unclear (timing artefact, retry-path success, race).

**Required investigation steps:**

1. Read `xgen-node/src/main.rs` end-to-end. Identify the clap definition of `--port`.
2. Read the config-loading code path. Identify the order of operations: when is config loaded relative to clap parsing? Where do the two merge? Which is authoritative at the bind call?
3. Identify the call site that constructs the bind address. Trace backwards: which value does it use — the flag, the config, or some merged structure?
4. Identify whether `--port` writes through to `[node].listen` or to a separate field. If it writes to a separate field, identify which field the bind call reads.
5. Determine the mechanism by inspection, not guesswork. Candidates: (a) flag value never reached the merge step, (b) config loaded after flag applied and overwrote it, (c) clap default-value shadowing, (d) bypassed code path entirely.
6. **Report findings in `JOURNAL.md`** before writing any fix code. Include: the mechanism, the file:line references, a one-paragraph explanation, and a stated hypothesis for whether other flags share the defect.

**Joe approves the findings before §5 begins.** Do not proceed to the shared helper design without approval — the helper shape depends on what the root cause turns out to be.

---

## §4 — Exhaustive audit (per-binary, empirical)

Fill in two tables per binary. **All four tables must be filled** — Node Table A, Node Table B, Client Table A, Client Table B. Empirical means: actually run the binary with the conditions described, observe the result, record real output. No assumptions, no extrapolation.

### §4.1 — Table A: Flags with config equivalents

For each row, set up a config with field X = value V1, invoke the binary with the flag set to V2 (≠ V1), observe which value the binary actually uses, record the result.

**xgen-node Table A** — candidate rows (Clair confirms or extends from §F.0.1 and §F.0.3):

| Flag | Config field | Env var | Tested value pair | Observed: which won? | D-068 compliant? | Code location |
|---|---|---|---|---|---|---|
| `--config <path>` | (default search path) | — | flag=`./alt.toml`, default=`./xgen-node_config.toml` | | | |
| `--log-level <lvl>` | `[logging].level` | `XGEN_LOG` | flag=`debug`, env=`warn`, config=`info` | | | |
| `--instance <label>` | (implicit default-instance) | — | flag=`alt`, default=none | | | |
| `--service` | (Tauri shell default) | — | flag set, config absent | | | |
| `--local` | `[node].local_mode` | — | flag set, config=`false` | | | |
| `--port <port>` | `[node].listen` (port component) | — | flag=`8081`, config=`8080` | **FAIL (J-078)** | **NO** | known violation |
| `--quiet` | (default banner) | — | flag set, config absent | | | |

**xgen-client Table A** — candidate rows:

| Flag | Config field | Env var | Tested value pair | Observed: which won? | D-068 compliant? | Code location |
|---|---|---|---|---|---|---|
| `--config <path>` | (default search path) | — | flag=`./alt.toml`, default=`./xgen-client_config.toml` | | | |
| `--log-level <lvl>` | `[logging].level` | `XGEN_LOG` | flag=`debug`, env=`warn`, config=`info` | | | |
| `--instance <label>` | (implicit default-instance) | — | flag=`alt`, default=none | | | |
| `--service` | (Tauri shell default) | — | flag set, config absent | | | |
| `--node <endpoint>` | `[client].node` | — | flag=`ws://127.0.0.1:9999/xgen`, config=`ws://127.0.0.1:8080/xgen` | | | |
| `--quiet` | (default banner) | — | flag set, config absent | | | |
| `--ai-mode` | `[ai].is_ai` (read at startup) | — | flag set, config has `[ai] is_ai = true` (consistent); also test conflict: flag set, config absent `[ai]` (must error cleanly, not silently fall back) | | | |

If Clair discovers a flag with a config equivalent not in the candidate rows above, **add a row** and audit it. If a candidate row turns out to have no config equivalent in current code, mark it "N/A — no config equivalent" and explain in the JOURNAL.

### §4.2 — Table B: Subcommand options that may shadow config or state-file values

This table captures the question Joe raised: it is not only top-level flags that can shadow config — subcommand options can too. `xgen-client send --node ...` is the obvious case; there may be others.

For each row, the question is the same: if both sources are present, which wins, and does it match D-068?

**xgen-node Table B** — candidate rows:

| Subcommand | Option | Could shadow | Tested current behaviour | D-068 compliant? | Code location |
|---|---|---|---|---|---|
| (audit each subcommand listed in §F.0.4 Node-only and §F.0.2 fundamental) | | | | | |

**xgen-client Table B** — candidate rows:

| Subcommand | Option | Could shadow | Tested current behaviour | D-068 compliant? | Code location |
|---|---|---|---|---|---|
| every network subcommand | `--node <endpoint>` | `[client].node` | flag=alt-node, config=default-node | | | |
| `register` | `--name <name>` | (no config equivalent expected) | confirm by inspection | | | |
| `init` | `--passphrase <pw>` | (writes config, doesn't read) | confirm by inspection | | | |
| `init` | `--ai`, `--cap key=value` | (writes config, doesn't read) | confirm by inspection | | | |

Clair walks **every subcommand** in §F.0.2 and §F.0.4 and asks per-option: is there a config field or state-file field this option could be shadowing? If yes, audit. If no, write "no shadowing — confirmed by inspection" in the JOURNAL.

### §4.3 — Recording the results

Each table goes into the JOURNAL.md entry for this task. Each row's "Tested" cell quotes a real terminal command and its real output — Rule 2. Each row's "Observed" cell is a factual statement of which value the binary used, derivable from the output quoted.

**Joe reviews the four completed tables before §5 begins.** This is the second gate.

---

## §5 — Shared helper abstraction

After §3 and §4 are approved, propose the implementation pattern in a JOURNAL entry. Joe approves before any code lands.

**Likely shape** (Clair refines, justifies, or rejects):

```rust
/// Resolve a setting from the four-tier precedence order.
/// flag wins over env wins over config wins over default.
pub fn resolve_setting<T: Clone>(
    flag: Option<T>,
    env: Option<T>,
    config: Option<T>,
    default: T,
) -> T {
    flag.or(env).or(config).unwrap_or(default)
}
```

Lives in `xgen-common` (used by both binaries — required for the "uniform across both binaries" property of D-068). Tested in `xgen-common` directly. Has documentation tying it to D-068 by name.

**Open design questions Clair addresses in the proposal JOURNAL entry:**

1. Does a single generic helper cover every case, or do certain settings (e.g. `--instance` which composes with paths; `--ai-mode` which is a boolean toggle that gates other behaviour) need bespoke handling? Justify either way.
2. How does the helper interact with clap's own default-value mechanism? Specifically: if clap's `default_value` is set on a flag, the flag is *never* `None` from clap's perspective. The helper must distinguish "operator passed `--flag X`" from "operator did not pass the flag, clap supplied the default." Concrete recommendation: do not use clap `default_value` on any flag covered by this helper; let the flag be `Option<T>` and resolve defaults at the helper. State this as a rule in the proposal.
3. Error path: if the flag value is malformed (e.g. `--port abc`), clap rejects at parse time and the helper is never called — that's correct. If the config value is malformed, where is it rejected? At config-load time, before the helper sees it. Confirm by inspection.
4. Should `XGEN_LOG` be the only env var the helper knows about, or should env-var support be generic (so any future env var slots in cleanly)? Recommend generic; defer adding new env vars until requested.

**Joe approves the helper shape before §6 begins.** This is the third gate.

---

## §6 — The fix — atomic commits per concern

After §5 is approved, land the fix in this commit order. Each commit is independently reviewable and passes `cargo test` on its own.

1. **`xgen-common`: add `resolve_setting` helper + its unit tests.** Tests assert the four-tier order, the None-handling, and the generic-over-T behaviour. No call sites touched yet.
2. **`xgen-node`: refactor every Table A and Table B non-compliant call site to use the helper.** Includes the `--port` root-cause fix. No behaviour changes on already-compliant flags (they pass through the helper identically).
3. **`xgen-client`: refactor every Table A and Table B non-compliant call site to use the helper.** Symmetric to commit 2. No behaviour changes on already-compliant flags.
4. **`xgen-node` + `xgen-client`: add integration tests** per §7. Tests live in `xgen-node/tests/` and `xgen-client/tests/` respectively (or in `xgen-common/tests/` if testable there without binary integration).
5. **Doc sync per §9.** Appendix F §F.0.6 reviewed and aligned; doc comments in both `main.rs` files reviewed against §F.0 per D-028.

Each commit message names the commit's scope, the relevant D-numbers (D-068, D-063 for the library-first connection), and the corresponding task section (§5, §6, §7, §9).

---

## §7 — Tests

Per-flag, per-binary, focused integration tests. Every row in Tables A and B gets at least one test — including compliant rows (regression lock).

### §7.1 — Test naming convention

`precedence_<binary>_<flag-or-subcommand>_flag_beats_config`  
`precedence_<binary>_<flag>_flag_beats_env_beats_config` (for `--log-level`)  
`precedence_<binary>_<flag>_config_beats_default` (for compliant baseline cases)  
`precedence_<binary>_<flag>_default_when_neither_set`

### §7.2 — Required tests (xgen-node)

- `precedence_node_port_flag_beats_config` — the J-078 reproduction case.
- `precedence_node_local_flag_beats_config`
- `precedence_node_config_flag_beats_default`
- `precedence_node_loglevel_flag_beats_env_beats_config`
- `precedence_node_instance_flag_beats_default`
- `precedence_node_service_flag_beats_default`
- `precedence_node_quiet_flag_beats_default`
- Per Table B row Clair identifies as auditable.

### §7.3 — Required tests (xgen-client)

- `precedence_client_node_flag_beats_config` (top-level flag)
- `precedence_client_send_node_flag_beats_config` (subcommand-option case)
- `precedence_client_config_flag_beats_default`
- `precedence_client_loglevel_flag_beats_env_beats_config`
- `precedence_client_instance_flag_beats_default`
- `precedence_client_service_flag_beats_default`
- `precedence_client_quiet_flag_beats_default`
- `precedence_client_aimode_flag_with_config_consistent`
- `precedence_client_aimode_flag_without_config_errors_cleanly` (negative test)
- Per Table B row Clair identifies as auditable.

### §7.4 — Running and recording

Run `cargo test`. Quote actual output in JOURNAL.md — exact pass count, exact new-test names visible in output, exact total. **Never invent a number** (Rule 5). Test count before this task is **435** (J-078); the new count is whatever `cargo test` actually reports after this task, with no extrapolation.

---

## §8 — Manual verification

After §6 commits land and §7 tests pass, run the manual verification by hand and quote real output in JOURNAL.md.

### §8.1 — The J-078 reproduction (xgen-node)

1. Configure `xgen-node_config.toml` with `listen = "ws://127.0.0.1:8080/xgen"`.
2. Ensure no other process is using port 8080 or 8081.
3. Launch: `xgen-node --port 8081`.
4. **Required outcome**: the Node binds `127.0.0.1:8081`, not `127.0.0.1:8080`.
5. Quote the actual Node startup log line that confirms which port was bound.
6. Stop the Node cleanly.

### §8.2 — The symmetric Client verification

1. Configure `xgen-client_config.toml` with `node = "ws://127.0.0.1:8080/xgen"`.
2. Have a Node running on `8081`, not `8080`.
3. Launch: `xgen-client --node ws://127.0.0.1:8081/xgen whoami`.
4. **Required outcome**: the Client connects to `8081`, not `8080`. If the Client's `whoami` does not require a network connection, run `register` or `status` instead — any command that surfaces which Node was contacted.
5. Quote the actual log line or output that confirms which Node was used.

### §8.3 — Spot-check matrix

Pick one fundamental flag (e.g. `--log-level`) and one non-fundamental flag per binary (e.g. `--port` on Node, `--node` on Client) and re-run the conflict scenario by hand even where §7 has automated tests. Two reasons: confirm the automated tests are actually testing the behaviour the operator observes, and catch any binary-vs-test-harness divergence early.

---

## §9 — Documentation sync

### §9.1 — Appendix F §F.0.6

The flag-by-flag mapping table in §F.0.6 must match Table A's final state. After §6 commits land:

1. Compare §F.0.6's "Flag-by-flag mapping" table row-by-row against Table A.
2. Any mismatch (flag listed in §F.0.6 but missing from Table A, or vice versa; or a "Flag wins?" cell now stale because the audit found a different reality) is a §F.0.6 edit. Edits are doc-only — no code touched in this step.
3. The "Known violation" note in §F.0.6 must be replaced with a "Locked compliant" or similar note pointing at the J-078 fix commit and this task's completion.

### §9.2 — Rust doc comments per D-028

D-028 says the Rust doc comments on flags in `xgen-node/src/main.rs` and `xgen-client/src/main.rs` MUST match Appendix F exactly. After §9.1, walk both `main.rs` files:

1. Every flag's doc comment that mentions config-override behaviour must match §F.0.6's "Flag wins?" cell verbatim or with neutral paraphrase that doesn't contradict.
2. Any flag whose doc comment is silent on config-vs-flag behaviour, but which has a config equivalent, gets a one-line addition: "Overrides `[section].field` from config (D-068)."
3. No code-behaviour changes in this step — doc comments only.

### §9.3 — DECISIONS.md cross-reference

D-068 references this task file by name (`tasks/CLI_PRECEDENCE_AUDIT.md`). When this task ships:

1. D-068's "Audit task scheduled" subsection gets a follow-up sentence: "Completed in J-NNN (date)."
2. No content edit to D-068's rule statement, reasoning, or scope — those are locked and remain unchanged.

---

## §10 — Definition of Done

Each item is independently verified before being marked complete (Rule 7). Real output, real observation, no shortcuts.

- [ ] §3 root-cause documented in `JOURNAL.md` with file:line references and the named mechanism.
- [ ] §3 findings approved by Joe (link the approval message or just confirm in JOURNAL).
- [ ] §4.1 xgen-node Table A filled in JOURNAL.md with empirical results — every row has real terminal output quoted.
- [ ] §4.1 xgen-client Table A filled in JOURNAL.md with empirical results — every row has real terminal output quoted.
- [ ] §4.2 xgen-node Table B filled in JOURNAL.md.
- [ ] §4.2 xgen-client Table B filled in JOURNAL.md.
- [ ] §4 four tables approved by Joe.
- [ ] §5 helper abstraction proposed in JOURNAL.md.
- [ ] §5 helper abstraction approved by Joe.
- [ ] §6 commit 1 (xgen-common helper + unit tests) lands and passes `cargo test`.
- [ ] §6 commit 2 (xgen-node refactor) lands and passes `cargo test`.
- [ ] §6 commit 3 (xgen-client refactor) lands and passes `cargo test`.
- [ ] §6 commit 4 (integration tests per §7) lands and passes `cargo test`.
- [ ] §7.4 — actual `cargo test` pass count quoted in JOURNAL.md, no fabricated numbers.
- [ ] §8.1 — J-078 reproduction succeeds on `xgen-node`. Real log line quoted.
- [ ] §8.2 — symmetric `xgen-client` verification succeeds. Real log line quoted.
- [ ] §8.3 — spot-check matrix run by hand, results in JOURNAL.md.
- [ ] §9.1 — Appendix F §F.0.6 reviewed and aligned with final Table A state.
- [ ] §9.2 — Rust doc comments in both `main.rs` files reviewed per D-028.
- [ ] §9.3 — DECISIONS.md D-068 closing note added.
- [ ] Task file header updated: Status → COMPLETED, Last updated → completion date.
- [ ] JOURNAL.md entry written **last**, quoting real output throughout (Rule 4).
- [ ] CLAUDE.md updated to reflect this task complete and M6 unblocked.

---

## Out of scope for this task

- Adding new flags or new config fields. The audit covers the current surface, not future additions.
- New environment variables beyond `XGEN_LOG`. The helper is generic over env vars so future ones slot in, but no new env vars are added here.
- The `init` flow's interactive prompts. Those are flow-control, not precedence.
- The `--aicontrol` surface (M7). Designed against the post-audit binary; out of scope until M7.
- M6 itself. M6 runs after this task ships.

---

## Cross-references

- **D-068** — the rule (locked, 2026-05-17).
- **D-063** — library-first principle; the shared helper extends it.
- **D-028** — canonical-source rule; §9.2 doc-comment sync follows it.
- **D-035** — convention-derived paths; sibling rule (paths derived not from flags-or-config, but from working directory).
- **Appendix F §F.0** — full CLI surface taxonomy.
- **Appendix F §F.0.6** — user-facing reference (already shipped; this task confirms or updates).
- **J-078** — the M5 close-out entry that surfaced the violation.
- **CLAUDE.md MANDATORY behaviour rules** — apply throughout (Rules 1–7).
