# M12.2b — F9 data-root posture shift: Implementation runbook

> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-16  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Purpose & status

The Clair-authored M12.2b implementation runbook — the **F9 data-root posture shift** (the design
§4 slice). Executes the Joe-LOCKED M12.2 design (`tasks/M12_2_DESIGN.md` v1.1, M12.2-D5 + D6) on
the M12.2 D-071 Phase-0 audit (`tasks/M12_2_FETCH_GATE_DATAROOT_PHASE0_AUDIT.md`, GO; finding
**M12.2-A-04**; forks **FK-4 / FK-5**; ledger). **The LAST M12.2 sub-arc** — its close closes
**M12.2**. M12.2a (the blob-feature trio + the self-thread e2e) SHIPPED + CLOSED at J-384.

**M12.2b = the F9 posture shift (D5 + D6) — a node-ops convention change, NOT a client feature:**
- **D5** — the node's data root defaults **outside** the install folder (a hand-rolled platform
  dir, no `dirs` crate), is operator-overridable via a **`--data-dir` flag + env var** (forced by
  the verified before-config-load ordering), is **startup-validated** (present-or-creatable /
  writable / not-tmp, fail fast), and `--instance` rebases under the **resolved** root.
- **D6** — existing `exe_dir()`-rooted deployments are **leave-as-legacy + named**: documented, with
  a `--data-dir=<old path>` escape; **no auto-migration**; the new default applies to **fresh**
  deployments only.

D-071 arc discipline: this runbook → **Joe-lock** → implement (spine-first; validation = the
RED-on-revert spine) → Chat doc-bridge (J-385 = M12.2b close = M12.2 close) → M12.3. **No code
precedes the runbook lock.** Decisions are arc-local (D-069). Two-seat: Clair commits **code only**;
Chat authors the canonical-record doc-bridge; Joe reassembles + pushes.

**Grounded against `main` @ `30bb18e`** (HEAD, J-384; tree clean). Every anchor in §2 was
**re-confirmed by reading production code this session** (D-078) — `app.rs`/`main.rs` drifted from
the audit's `60cfd8f` anchors across the M12.2a commits; lines below are current.

---

## §2 Grounding ledger (F9 seams, re-confirmed to file:line on `main` @ `30bb18e`)

| # | Seam | Location | M12.2b relevance |
|---|---|---|---|
| F1 | `resolve_data_dir(instance)` = `exe_dir()` / `exe_dir()/instances/<label>`; **no `--data-dir`** | `xgen-node/src/main.rs:173-188` | the default-shift + override + validation site |
| F2 | `data_dir` chosen (`:202`) **before** `config_path` (`:203-206`) + `try_load_config` (`app.rs:562`) | `main.rs:202` | override MUST be flag/env, not a config field (config lives under the root) |
| F3 | node `Cli` (config/local/instance/port/log-level/check-config/print-config/pid/reload-config); **no `--data-dir`** | `main.rs:45-127` | add `--data-dir` (global, like `--instance`) |
| F4 | `NodeConfig::default` roots `keypair_path`/`spaces_dir`/`blobs_dir` at `exe_dir()` | `app.rs:335-352` | the written-default / `unwrap_or_default` fallback root |
| F5 | `cmd_init` writes `NodeConfig::default()` then overrides **only `keypair_path`** → `data_dir`; **spaces/blobs stay exe_dir** | `app.rs:3787-3818` (`:3813` default, `:3815` keypair-only) | **F9 must root spaces/blobs at data_dir** (else the event DAG + blobs stay in the install folder — §3 S-2) |
| F6 | runtime resolution: `spaces_dir`/`blobs_dir` = `config.paths.X.unwrap_or(data_dir.join(...))` (fallback); `keypair_path` = `config.paths.keypair_path` (**no** fallback); identities/federation/queue/policy/bootstrap/state/audit/logs/pid = `data_dir.join(...)` (no config field) | `app.rs:644/759/768/772/953/984/1015/1070/1105/928/667/...` | the ~14 `data_dir.join` consumers **inherit** the resolved root unchanged |
| F7 | **client** `resolve_data_dir` is IDENTICAL (`exe_dir()` / instances; no `--data-dir`); `data_dir` `:55` | `xgen-client/src/main.rs:36-55` | the **scope fork** (§4 VA): node-only vs both binaries |
| F8 | mptest `ManagedProcess`: `run_init` (`--instance <l> init`, `:138`), `init_and_spawn_node` (`--instance --service --port`, `:157`), `init_and_spawn_client` (`:208`), `spawn_client_reusing_keypair` (`:251`); `instance_data_dir = exe_dir(bins)/instances/<label>` (`:78`) | `xgen-mptest/src/process.rs` | **blast radius** — the harness locates data at `exe_dir/instances/<label>`; the default change MUST be pinned with `--data-dir <exe_dir>` here or box-gated tests break (§3 S-3) |
| F9 | in-process `phase9_harness` uses `tempfile::tempdir()` + explicit `spaces_dir`/`blobs_dir`; **never calls `resolve_data_dir`** | `xgen-node/src/tests/phase9_harness.rs:760` | the in-suite **1440/0** suite is UNAFFECTED by the default change |
| F10 | no `dirs`/`directories` crate in any manifest | (grep, empty) | **hand-rolled** platform lookup (D5) |
| F11 | no existing writable/creatable/not-tmp validation (the `tempdir` hits are test-only) | `main.rs`/`app.rs` | startup validation is **net-new** (D5) |
| F12 | Appendix F describes `--instance` as `<exe dir>/instances/<label>` | `docs/xgen_appendix_f_en.md` F.0.1 / F.8.1 | stale text → updates at close (Chat-owned; named here) |
| F13 | `validate_instance_label` + the eprintln/`exit(1)` fail-fast pattern | `main.rs:173-183` | the precedent shape for the new validation's fail-fast |

---

## §3 D-065 surfacings / scope (surfaced, not papered over — confirm at lock)

### S-1 — the scope fork (the one real fork; §4 VA): node-only vs both binaries

F7: `xgen-client` has the **identical** `resolve_data_dir` (exe_dir / instances; no `--data-dir`).
The design framed F9 around the **node** (M12-D7 / M12.2-D5 / audit M12.2-A-04 are node-only). But
the client also roots its data (`xgen-client_state.json` + keypair) at the install folder, and
`--instance` is on both binaries. **Recommend BOTH** (§4 VA) — symmetric, cheap, and consistent
(a user should not get node-data-outside but client-data-inside the install folder). Node-only is a
coherent narrower scope. **Joe locks.**

### S-2 — `cmd_init` leaves spaces/blobs at exe_dir (F9 must root them at data_dir)

F5: `cmd_init` roots **keypair + config** at `data_dir` but **spaces_dir/blobs_dir stay
`exe_dir`** (only `keypair_path` is overridden after `NodeConfig::default()`). So today `--instance`
instances share `exe_dir/spaces`; and under a future `--data-dir`, the **event DAG (spaces) +
attachments (blobs)** — the bulk + the whole F9 "back-up-as-a-unit durable root" point — would
stay in the install folder. **F9 must root spaces_dir + blobs_dir at the resolved data_dir** in the
written default config (C1). This corrects a pre-existing rooting asymmetry; surfaced, not silent.

### S-3 — mptest blast radius (a breaking default change; must be pinned in the same commit)

F8: the mptest harness spawns the real binaries with `--instance <label>` and locates their data at
`instance_data_dir = exe_dir(bins)/instances/<label>`. Changing the binary default to a platform dir
→ the binary writes `<platform>/instances/<label>` while the harness reads `<exe_dir>/instances/...`
→ **all box-gated mptest tests break** (incl. the M12.2a self-thread e2e + the MP-arc tests). **Fix
(C1, same commit as the default change):** the harness pins `--data-dir <exe_dir(bins)>` on every
real-binary spawn (`run_init` + `init_and_spawn_node` [+ `init_and_spawn_client` +
`spawn_client_reusing_keypair` if VA=both]) so `--data-dir <exe_dir> --instance <label>` resolves to
`exe_dir/instances/<label>` = `instance_data_dir` (behaviour-preserving pin). The in-suite suite
(F9) is unaffected.

---

## §4 Runbook-level values — recommend; **Joe locks at the runbook lock**

| Value | Recommendation | Grounding |
|---|---|---|
| **VA — scope (S-1)** | **Both binaries.** The platform-default + validation logic lives in **`xgen-common`** (one impl, no drift, D-067; unit-testable), consumed by both `resolve_data_dir`s. Node-only is the coherent narrower alternative (logic still lands in xgen-common, consumed by node only). | F7 |
| **VB — platform-dir chain + app subdir** | `#[cfg(windows)]` → `%LOCALAPPDATA%` (env `LOCALAPPDATA`; fallback `%USERPROFILE%\AppData\Local`). `#[cfg(not(windows))]` → `$XDG_DATA_HOME` else `$HOME/.local/share` (covers Linux **and** macOS — no macOS special-case; the project is Windows-primary, D-043). App subdir = **`XGenProtocol/`** (shared by both binaries; the `xgen-node_*`/`xgen-client_*` filename prefixes + `--instance` keep files distinct — matches today's exe_dir-shared model). If no platform base resolves (no env, no HOME) → **fail fast** (do NOT silently fall back to exe_dir — that would re-pollute the install folder). | F10; FK-4 |
| **VC — flag + env names + precedence** | `--data-dir <abs path>` (global, like `--instance`) + env **`XGEN_DATA_DIR`**. Precedence **flag > env > platform-default** (no config equivalent — precedes config load, F2; record in Appendix F F.0.6 / D-068 at close). A pure `resolve_data_root_choice(flag, env) -> Source` helper (xgen-common) encodes precedence → unit-testable (W2). | F2/F3 |
| **VD — startup validation (the spine)** | On the resolved root: (1) **creatable** — `create_dir_all`; failure → error; (2) **writable** — a write-probe (create/write/remove a `.xgen-write-test` temp file); failure → error; (3) **not-tmp** — reject if the canonicalised root is under `std::env::temp_dir()` (data under temp is wiped = silent loss; note `%LOCALAPPDATA%/Temp` is a *sibling* of the `XGenProtocol` default, so the default passes). **Fail-fast** = clear stderr message + `exit(1)` (the F13 `validate_instance_label` pattern). Pure `validate_data_dir(path) -> Result<()>` in xgen-common → unit-testable + RED-on-revert (W3). | F11/F13 |
| **VE — D6 existing-data (FK-5)** | **Leave-as-legacy + named** — no auto-migration. **+ a startup NOTICE (not an error):** when the resolved root is the fresh platform default (no `--data-dir`) **and** an old `exe_dir` layout holds node data (e.g. `exe_dir/xgen-node_identities.db` exists), print a one-line stderr notice naming the `--data-dir=<exe_dir>` escape (so an upgrading operator isn't surprised by a "fresh" node ignoring old data). Low cost, makes "named" active. Joe's call whether to build the notice or doc-only. | FK-5 |
| **VF — `--instance` interaction** | `--data-dir X --instance n1` → `X/instances/n1` (compose; `--instance` rebases under the resolved root, D5). Flag/env absent + `--instance` → `<platform-default>/instances/n1`. | F1 |

---

## §5 Build sequence (spine-first; written for the recommended VA..VF resolution)

Two code commits, then Chat's doc-bridge close. Per-commit DoD in §7. Clair commits code; Chat
authors the canonical-record bridge; Joe reassembles + pushes.

- **C1 — the data-root resolution shift (D5 minus validation) + the blast-radius pins.**
  - **xgen-common** (new, e.g. `src/data_dir.rs`, VA): `platform_default_data_dir() -> Option<PathBuf>`
    (VB hand-rolled per-OS) + `resolve_data_root_choice(flag, env) -> RootChoice` (VC precedence).
    Pure; unit-tested (W1/W2).
  - **xgen-node** `resolve_data_dir` (main.rs:173): take the `--data-dir` flag value; resolve
    flag > `XGEN_DATA_DIR` env > platform default (fail fast if no platform base, VB); `--instance`
    rebases under the resolved root (VF). Add `--data-dir` to the `Cli` struct (global).
  - **xgen-node config rooting (S-2):** `cmd_init` roots **spaces_dir + blobs_dir** (not just
    keypair) at the resolved `data_dir` in the written config; the `NodeConfig::default` fallback
    note recorded (no-config nodes fail on keypair at app.rs:644 regardless — the written config is
    the live path). *(Confirm-at-build: set them to `Some(data_dir/…)` vs `None`-for-fallback;
    `Some` is explicit + matches the keypair pattern.)*
  - **xgen-client** (if VA=both): the same `resolve_data_dir` + `--data-dir`/`XGEN_DATA_DIR` over the
    shared xgen-common helpers.
  - **mptest pin (S-3):** `run_init` + `init_and_spawn_node` [+ client spawns if VA=both] pass
    `--data-dir <exe_dir(bins)>` so the harness's `instance_data_dir` assumption holds. **Box-gated
    tests must still pass** (the M12.2a e2e + MP-arc) — re-run a representative box-gated test.
  - Witnesses W1 / W2 / W4 in-suite.

- **C2 — startup validation (the RED-on-revert spine) + D6.**
  - **xgen-common** `validate_data_dir(path) -> Result<()>` (VD: creatable / writable / not-tmp).
  - **xgen-node** `resolve_data_dir` calls it after resolving (fail-fast, exit(1), clear message);
    **xgen-client** likewise (if VA=both).
  - **D6 (VE):** the leave-as-legacy notice (if locked) — at startup, fresh-default + old-exe_dir-data
    → one-line stderr notice naming `--data-dir=<exe_dir>`.
  - Witnesses W3 (RED-on-revert: neuter the not-tmp check → a temp root passes → RED) + W5.

**(Chat) doc-bridge + M12.2b close = M12.2 close (J-385)** — canonical flips (CLAUDE PLAY, JOURNAL
J-385, ROADMAP, design status → M12.2 CLOSED); **Appendix F** F.0.1/F.8.1 `--instance` text →
"<resolved data root>/instances/<label>" + a `--data-dir`/`XGEN_DATA_DIR` entry (F.0.1 + F.0.6
precedence, D-068); M12.3 opens next.

---

## §6 Witnesses (in-suite — config/path resolution; **no box-gated RUN** unlike M12.2a, except the S-3 re-run)

- **W1 — fresh default.** `platform_default_data_dir()` resolves under the platform base (LOCALAPPDATA
  / XDG), ends with `XGenProtocol`, and is **not** `exe_dir` (env set in the test). RED-on-revert:
  fall back to exe_dir → W1 RED.
- **W2 — override + precedence.** `resolve_data_root_choice(Some(flag), _) == flag`;
  `(None, Some(env)) == env`; `(None, None) == platform default`. (flag > env > default, VC.)
- **W3 — validation (spine, RED-on-revert).** `validate_data_dir`: a good tempdir → `Ok`; a path
  under `std::env::temp_dir()` → `Err`; a non-creatable path (under a regular file) → `Err`.
  RED-on-revert: neuter the not-tmp branch → a temp path returns `Ok` → W3 RED.
- **W4 — instance rebase.** resolved-root + `--instance n1` → `<root>/instances/n1` (VF).
- **W5 — legacy (D6).** `resolve_data_root_choice(Some(old_exe_dir), _)` returns the old path (override
  honored → a legacy node starts there, no migration); + (if VE notice built) the notice fires when
  fresh-default + old-data, and does NOT fire otherwise.
- **S-3 box-gated re-run (C1):** re-run one box-gated mptest (e.g. `m12_2a_self_thread_e2e w_multi`)
  after the harness pin to confirm the default change didn't break the real-binary path. **Flag the
  RUN for Joe** (box-gated; needs `--features harness-control` node + client built first).

---

## §7 Definition of Done

**Per-commit gate (C1, C2):** `cargo build --workspace` 0-error
(`CARGO_TARGET_DIR=C:/cargo-targets/XGenProtocol`);
`cargo clippy --workspace --all-targets --all-features -- -D warnings` clean;
`cargo test --workspace` green (baseline **1440/0** + the commit's new in-suite tests); for C2 (the
spine), **RED-on-revert recorded** in the commit message.

**Milestone gate (close):** W1–W5 green in-suite; the C1 S-3 box-gated mptest re-run reported (the
default change did not break the real-binary path); Appendix F `--instance`/`--data-dir` text landed
(Chat); M12.2 closes when M12.2b closes.

*(No "commit pushed" DoD line — `Status: COMPLETED` is the shipped signal. Joe pushes.)*

---

## §8 Out of scope (later sub-arcs / reserved — do NOT pull in)

- **Pattern-A tier→size table + per-Space immutable override** (F6 enrichment) — reserved (M12.2-D4).
- **Auto-migration** of existing `exe_dir` data — declined (D6/VE); the `--data-dir=<old>` escape covers it.
- A **`dirs`/`directories` crate** dependency — declined (VB hand-rolled).
- **M12.3** federation fetch-by-hash + F3 + `10003 blob_unavailable`; **M12.4** `message.redact` +
  F2b retention read + crypto-shred (D3-gated). **M12-D6** universal-E2E invariant stays a flagged
  DECISIONS.md promotion candidate — not this arc.

---

## §9 Sequence + entry (Rule 0)

this runbook → **Joe locks VA..VF + the §5 sequence** → implement C1→C2 (spine-first; validation =
the RED-on-revert spine) → hand Joe the push per commit → Chat doc-bridge (J-385 = M12.2b close =
**M12.2 close**) → M12.3.

**Entry (Rule 0):** `CLAUDE.md` PLAY → `JOURNAL.md` J-384 → `tasks/M12_2_DESIGN.md` v1.1 (§2 D5+D6 /
§4 M12.2b) → `tasks/M12_2_FETCH_GATE_DATAROOT_PHASE0_AUDIT.md` (M12.2-A-04 + FK-4/FK-5 + the ledger)
→ `tasks/M12_ATTACHMENTS_DESIGN.md` v1.1 (M12-D7) → `docs/ROADMAP.md` (M12).
