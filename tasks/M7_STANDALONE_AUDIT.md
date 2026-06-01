# M7-standalone — Phase 0 Backing Audit (live config reload)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Scope & method

M7-standalone realises the `--reload-config` Node verb that today returns honest `NOT_IMPLEMENTED` (`pipe.rs:819`). The *what-reloads* classification is already Joe-locked at `docs/xgen_node_admin_ops_design.md §2.6.3` (M6 Phase 0 Pass 3, Joe-lock #2). This audit grounds three things against the **live tree** before any design lock, and lets the findings — not the pre-sketched Q1–Q4 — set the design questions:

1. the config **re-read / validation** path;
2. each **Reloadable** field's **live-apply seam**;
3. whether the §2.6.3 **buckets still hold** against the code as built.

Doc-only, no code. Method: read the canonical tree (`xgen-node/src/`, not `.claude/worktrees/`).

---

## §2 The three grounding targets — verdicts

### T1 — Re-read / validation path → **EXISTS (clean)**
- `try_load_config(path) -> Option<NodeConfig>` (`app.rs:3217`) = `read_to_string` + `toml::from_str`. A reusable, total re-read.
- `cmd_check_config` (`app.rs:3118`) = validate-by-parse precedent; serialise-back via `toml::to_string_pretty` (`app.rs:2854/3138`) exists for the persist side.
- The `__RELOAD_CONFIG__` handler already holds **`config_path: PathBuf`** (`pipe.rs:719`) **and `runtime: Arc<Mutex<NodeRuntime>>`** (`pipe.rs:720`) in scope — the surface is wired to reach disk **and** the live resident; only the body is a stub. The `__HEALTH__` arm shows the reach-into-`runtime` pattern.
- **Caveat (finding F-1):** "validation" today = serde **structural** parse only. There is no semantic check (`[node].listen` socket-addr well-formedness; `[logging].level` directive legality). `EnvFilter` self-validates the level *on apply* (`log_set_level` → `LogSetError`), so logging is safe; `listen`/`local_mode` are not pre-checked.

### T2 — `[logging].level` live-apply → **BACKED (clean)**
- `LOG_RELOAD: OnceLock<reload::Handle<EnvFilter, Registry>>` (`app.rs:318`), set at startup (`app.rs:483`); `admin_ops::log_set_level` (A6-D1) already flips it live (`app.rs:363–382`).
- Reloading the level = reuse that exact handle. Fully backed, self-validating, no new wiring.

### T3 — `[node].local_mode` live-apply → **NOT BACKED (the catch — see §4)**
- `local_mode` is computed **once at startup** (`app.rs:414`, `config.node.local_mode || opts.local_override`) and threaded **by value** as a plain `bool` into ~20 hot-path sites (reconnect handlers, `handle_identity_msg`, `accept_registration`). No `Arc` / atomic / `OnceLock` holds it → **no single live cell to flip**.

---

## §3 Bucket re-validation (does §2.6.3 hold against the live tree?)

§2.6.3 is a **combined Node+Client** table. Filtering to the **Node** config struct (`NodeConfig` = `node{listen, local_mode}`, `paths{keypair_path, spaces_dir}`, `logging{level}`, `sync`, `federation`, `bootstrap`):

| §2.6.3 entry | Node-relevant? | Live-apply seam? | Verdict |
|---|---|---|---|
| `[logging].level` (Reloadable) | yes | reload handle (T2) | **HOLDS** |
| `[node].local_mode` (Reloadable) | yes | none (T3/§4) | **DOES NOT HOLD** |
| `[node].listen` (Restart-required) | yes | n/a (persist-only) | holds |
| `[paths].keypair_path` (Restart-required) | yes | n/a (persist-only) | holds |
| `[ai.behavior].*` (Reloadable) | **no** — Client field | — | out of Node scope |
| `[ai].plugin` / `[ai].is_ai` (Restart-required) | **no** — Client | — | out of Node scope |
| `[client].node` (Restart-required) | **no** — Client | — | out of Node scope |

**Finding F-2 (scope):** the Node-only Reloadable set is exactly **two** fields, not the full table — `{[logging].level (backed), [node].local_mode (not backed)}`. The three `[ai.*]`/`[client].*` Reloadable/Restart entries are Client-side; a Node `--reload-config` v1 cannot touch them. Sections `sync` / `federation` / `bootstrap` are unclassified by §2.6.3 (F-3).

---

## §4 The catch — `[node].local_mode` is classified Reloadable but has no live seam

The premise that the buckets "hold" **inverts here**, mirroring how the last cluster surfaced a catch at every checkpoint:

- **No live cell.** `local_mode` is a frozen startup `bool` threaded by value; a disk change cannot reach the running resident's WS/registration path without a cross-cutting refactor (`AtomicBool` or `Arc<RwLock<bool>>` shared into every handler).
- **Inconsistency (F-4).** Some verbs **re-read `cfg.node.local_mode` fresh from disk per call** (`admin_ops.rs:1744`, federation-add) — those already observe an edited file with no reload mechanism at all — while the resident hot path uses the frozen bool. So "Reloadable" is today **half-true and path-dependent**, not a clean live-apply.
- **Consequence.** Marking `[node].local_mode` "Reloadable (changes apply immediately)" in §2.6.3 was **aspirational** — written in M6 against *planned* future behaviour (§2.6.3 explicitly says the mechanism lives in M7). The audit confirms the seam was never built.

---

## §5 Surface wiring (the stub)

- `--reload-config` is a **control-mode** flag (`main.rs:113/230`) → `cmd_reload_config` (`pipe.rs:1142`) → sends `__RELOAD_CONFIG__` → resident stub (`pipe.rs:819`). Windows-only; non-Windows stub at `pipe.rs:1190`.
- This is the **legacy `pipe.rs` control-flag** family (`__PING__`/`__HEALTH__`/`__STOP__`) — **not** `--batch`, **not** `--aicontrol`. D-066 kept `pipe.rs`/`--batch` untouched through the aicontrol/events arcs; M7-standalone is the arc that **legitimately** reactivates this arm.
- The handler already has `config_path` + `runtime` in scope (§2 T1) — no new plumbing to reach the inputs.

---

## §6 Design questions the audit raises (reshaped — supersede the pre-sketched Q1–Q4)

The audit moves the centre of gravity off "surface/atomicity polish" and onto **the `local_mode` premise break**:

- **DQ-1 (the real lock) — what does v1 do about `[node].local_mode`?**
  - **A. Demote → Restart-required (honest-minimal, zero-refactor):** persist to disk, report "active on restart"; v1 applies live only the genuinely-backed `[logging].level`. Also resolves F-4 by *not* claiming live-apply we don't have. Aligns with *"honest longer work over fast shortcuts."*
  - **B. Build the live seam:** `AtomicBool`/shared-cell refactor across ~20 sites + reconcile F-4. Bigger, touches the hot path, real risk.
  - **C. Split:** v1 = mechanism + logging-live + local_mode-restart-required; a follow-on arc does B if wanted. (Lead recommendation.)
- **DQ-2 — surface placement:** stay on legacy `pipe.rs __RELOAD_CONFIG__` only (already wired), or also expose `reload-config` via `--aicontrol`/`admin_ops`? (One source of truth preferred; legacy arm is the lower-risk v1 home.)
- **DQ-3 — validation depth (F-1):** re-parse only (all-or-nothing; `try_load_config` already returns `None` on bad TOML), or add semantic validation for restart-required fields before persisting? Logging self-validates on apply.
- **DQ-4 — report shape:** the structured reply replacing `NOT_IMPLEMENTED` — `reloaded[]` / `pending-restart[]` / `rejected[]`. Define the wire shape (control-line text vs JSON).
- **DQ-5 — scope confirm (F-2):** Node-only v1; Client `[ai.*]`/`[client].node` reload deferred to its own follow-on?
- **DQ-6 — unclassified sections (F-3):** bucket `sync` / `federation` / `bootstrap` (the bootstrap arc already says config *seeds*, the JSON store is *truth* — likely Restart-required or N/A).

---

## §7 Verdict summary

| Target | Verdict |
|---|---|
| Re-read / validation path | **EXISTS** (`try_load_config`; surface pre-wired with `config_path`+`runtime`); structural-only validation (F-1) |
| `[logging].level` live-apply | **BACKED** (reload handle, A6-D1) |
| `[node].local_mode` live-apply | **NOT BACKED** — classified Reloadable, no live seam, path-dependent (§4, F-4) |
| §2.6.3 buckets hold? | **Partially.** Logging holds; `local_mode` does not; three entries are Client-only (F-2) |

**Net:** the re-read path is the easy half and is already there; the milestone's one genuine design fork is **DQ-1 (`local_mode`)**. The buckets did **not** fully hold — so per Joe's framing, the audit **reshapes** the design phase around DQ-1 rather than confirming a clean Q1–Q4 lock.

**Next-active:** design phase (`tasks/M7_STANDALONE_DESIGN.md`) — lock DQ-1 first (it sets v1's shape), then DQ-2…DQ-6. No code until the design + runbook close. Clair stood down.
