# M-RP6.1e-C2 — `get_about_info` (xgen-common::about + build_info + Tauri read command) build runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-11  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

For Clair. **Second of three steps** in M-RP6.1e-C (Phase-0 J-488 / D-107 / `docs/xgen-client-frame-phase0.md` §4.1, §6, §10.4). Design **locked by Joe** ("go by all recomms", this session, after a grounding pass that **corrected three of Chat's own earlier recommendations** — see §2.0, read it first).

**The C-split:** C1 `dialog` core ✅ CLOSED (J-496) · **C2 `get_about_info` ← this runbook** · C3 Help→About assembly.

**C2 is the Rust half.** It ships **no UI** — no Help menu, no About dialog, no logo. It ships the **data** the About dialog will read, and proves it over CDP in the real client. Design captured here; **no code at lock time** (Rule 1/5).

---

## 1. Goal

Everything in Joe's About below "Built" — build date, commit, Rust/Tauri/Svelte versions, platform, app directory, data/config paths — is **invisible to the frontend**. C2 builds the read path:

- one canonical **`xgen-common::about`** module producing an `AboutInfo`,
- a thin **`#[tauri::command] get_about_info`** in `desktop.rs` returning it,
- **no new build-metadata surface** (see §2.0 — one already exists).

---

## 2. Locked design

### 2.0 ⚠️ READ FIRST — what already exists, and three corrections

**Chat's own earlier recommendations were wrong on three points.** They were caught by grounding the code before writing this runbook (Rule 5), and are corrected here. Do **not** implement the earlier shape.

**(1) `xgen-common::build_info` ALREADY EXISTS — and already does the "Built + SHA" job.**

```rust
// xgen-common/src/build_info.rs — SHIPPED, in use
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
pub const GIT_HASH: &str = env!("BUILD_GIT_HASH");
```

`xgen-common/build.rs` **already** emits `BUILD_TIMESTAMP` + `BUILD_GIT_HASH` (it already shells `git rev-parse --short HEAD`) and already sets `rerun-if-changed=../.git/HEAD` + `../.git/refs/`. It is already consumed in ~6 places (`--version`, `print_banner`, `ops::StatusResult.version`, `ClientState.version`/`.build`).

> **Therefore: DO NOT add a `build.rs` build-metadata emission to `xgen-client` for Built/SHA.** A second build-metadata surface is exactly the **D-067 drift surface** this project exists to eliminate. **Reuse `build_info`.**

**(2) The Svelte version is NOT in `package.json`.** `ui/client/package.json` declares **`"svelte": "^5"`** — a *range*. Reading it would print **`"^5"`**, which is not a version and would be a lie in an About box. The **resolved** version is **`5.55.5`**, and lives in:
- `ui/client/node_modules/svelte/package.json` — **not committed**, absent on a clean checkout, and
- **`ui/client/package-lock.json` — COMMITTED** (confirmed via `git ls-files`).

→ **Read the lockfile** (§2.3).

**(3) A new `#[tauri::command]` needs NO capability grant.** Tauri v2 capabilities gate **`core:` / plugin** commands, not app-defined ones. Proof on disk: `get_state` / `get_pacing_state` / `get_substitutions` / `set_substitutions` / `quit` all work today, and `capabilities/default.json` carries only `core:default`, `process:default`, `core:window:allow-start-resize-dragging`. The J-495 capability lesson was **specific to `core:window:*`**. **Do not add a permission for `get_about_info`** — cargo-culting the last milestone's lesson would add a meaningless grant.

**Also grounded (so you don't have to):** `tauri::VERSION` **exists** (`pub const VERSION: &str = env!("CARGO_PKG_VERSION")`, tauri **2.11.1** resolved) → no build.rs needed for it. `data_dir` is **not** currently Tauri-managed state (only `ConfigPath` is).

### 2.1 `build_info` — ONE new field

Extend the **existing** `xgen-common/build.rs` (not a new one, not a client one):

- emit **`cargo:rustc-env=BUILD_RUSTC_VERSION=…`** by shelling **`rustc -V`** (no new dependency — `std::process::Command`, exactly as the existing `git rev-parse` call does). Fall back to `"unknown"` on failure, mirroring `BUILD_GIT_HASH`'s existing fallback.
- add `pub const RUSTC_VERSION: &str = env!("BUILD_RUSTC_VERSION");` to `build_info.rs`.

**One build-info surface. The node gets it free** — that is the whole point of putting it here.

### 2.2 `xgen-common::about` — the B2 shape (Joe-locked)

New module `xgen-common/src/about.rs`.

```rust
/// The app-agnostic environment block. Identical fields in every XGen app.
pub struct AboutInfo {
    // Identity of the app — PASSED IN (see the ⚠️ below).
    pub name: String,
    pub version: String,
    pub link: String,

    // Build metadata — from build_info (§2.1). NOT re-derived.
    pub built: String,        // build_info::BUILD_TIMESTAMP
    pub commit: String,       // build_info::GIT_HASH
    pub rustc: String,        // build_info::RUSTC_VERSION

    // Toolchain the app itself knows — PASSED IN (common must not depend on tauri/JS).
    pub tauri: String,
    pub svelte: String,

    // Environment.
    pub platform: String,     // std::env::consts::OS + ARCH — honest, no OS-build-number theatre
    pub app_dir: String,      // current_exe()'s parent
    pub data_dir: String,     // PASSED IN — each app resolves its own
    pub config_path: String,  // PASSED IN
}
```

> ⚠️ **`version` MUST be the calling app's own `env!("CARGO_PKG_VERSION")` — NOT `build_info::VERSION`.** `build_info::VERSION` is **xgen-common's** version. Both are `0.10.3` today, so the bug would be **invisible** — and would silently start lying the day the crates' versions diverge. Pass it in from `xgen-client`.

> ⚠️ **`xgen-common` must NOT gain a `tauri` dependency.** It is the protocol-layer crate. The Tauri version is `tauri::VERSION`, read **in the shell** and passed in. Same reasoning as `core`-stays-app-agnostic for components.

**Constructor:** one canonical `pub fn collect(params…) -> AboutInfo` — paths and app facts **passed in, never derived inside common** (the client and node resolve `data_dir` differently, via `--instance`).

**Per-app typed extension (B2, Joe-locked):**

```rust
pub struct ClientAboutInfo { pub common: AboutInfo, /* client-only fields */ }
```

**Rule-6 note, flagged not hidden:** on Joe's field list the client has **zero** client-only fields today, so `ClientAboutInfo` is presently a **zero-extension wrapper**. Keep it anyway — it is the **typed seam**: the node's About *will* differ (listen port, peer count, node XGID, federation role), and with the wrapper in place the command's return type never changes when the first client-only field lands. This is a deliberate, stated choice, not an oversight. If Clair thinks it earns its keep differently, **flag it** — do not silently collapse it.

**Serde:** `#[derive(Serialize)]` (+ `Deserialize` for symmetry with `ops::*Result`), so it crosses the Tauri IPC boundary as plain JSON. Follow the `ops.rs` result-struct conventions.

### 2.3 Svelte version — via the client `build.rs` (option S-A, Joe-locked)

`xgen-client/build.rs` is currently just `tauri_build::build()`. Extend it:

- parse **`ui/client/package-lock.json`** (the committed lockfile, §2.0) for the resolved `node_modules/svelte` → `version`, and emit `cargo:rustc-env=XGEN_SVELTE_VERSION=…`.
- add **`serde_json` as a `[build-dependencies]`** entry — it is already a runtime dep, and a crude string scan of a lockfile is fragile.
- `cargo:rerun-if-changed=../ui/client/package-lock.json`.
- Fallback `"unknown"` on any failure (missing file, unexpected shape) — **never** guess, never print a range like `^5`.

**Why here and not in `xgen-common`:** the Svelte version is a **per-app frontend fact** (the node has its own `package-lock.json`). Common has no UI. The node repeats this pattern in its own `build.rs` at M-RP7.x.

*(The rejected alternative, for the record: injecting it via `vite.config.js` `define:`. Cleaner layering, but then About data arrives from two sources; S-A keeps it one `invoke`.)*

### 2.4 "Built" staleness — LEAVE IT, and label it honestly (Joe-locked)

`xgen-common/build.rs` reruns only when `.git/HEAD` / `.git/refs/` change. So **`BUILD_TIMESTAMP` is the last commit-triggered rebuild, not "when this binary was linked"** — build with uncommitted changes and the timestamp is stale.

**Do not "fix" this.** Forcing a rerun on every build would recompile `xgen-common` — and therefore the **entire workspace** — on every single build. The cost is real and the benefit is cosmetic.

**Instead: be honest.** The `commit` (SHA) is the field that actually identifies a build, and it is exact. The runbook records the caveat; C3 may render `Built` alongside the commit so the pair reads truthfully. **Do not add a "dirty" marker** (noise).

### 2.5 The Tauri command + managed `data_dir`

- **`desktop.rs`**: `#[tauri::command] fn get_about_info(…) -> ClientAboutInfo` — a **thin wrapper** over `xgen_common::about::collect(…)`, exactly the `get_substitutions` shape (that command is 1 line over `app::load_substitutions_section`). **No logic in the command.**
- Register it in the existing `invoke_handler![…]` list.
- **`data_dir` → managed state.** Today only `ConfigPath(PathBuf)` is managed. Add the data dir (a new `DataDir(PathBuf)` managed struct, or widen to an `AboutPaths`-style struct — **Clair's call, note which**). Do **not** reconstruct it as `config_path.parent()`: that is a derivation that silently breaks if the config filename ever moves.
- **No capability change** (§2.0 (3)).
- `app_dir` = `std::env::current_exe()`'s parent; handle the `Err` honestly (`"unknown"`), never `unwrap()`.

---

## 3. Files to touch (indicative — Clair confirms exact paths)

1. `xgen-common/build.rs` — **+`BUILD_RUSTC_VERSION`** (`rustc -V`, `"unknown"` fallback). *Existing file — extend, don't replace.*
2. `xgen-common/src/build_info.rs` — **+`RUSTC_VERSION`** const.
3. `xgen-common/src/about.rs` — **new**: `AboutInfo` + `collect()` + `ClientAboutInfo` (+ `NodeAboutInfo` **only if** it costs nothing to declare; otherwise leave it to M-RP7.x — **your call, flag it**).
4. `xgen-common/src/lib.rs` — register the `about` module.
5. `xgen-client/build.rs` — **+ Svelte version** from `ui/client/package-lock.json` (§2.3).
6. `xgen-client/Cargo.toml` — **+`serde_json` under `[build-dependencies]`**.
7. `xgen-client/src/desktop.rs` — the `get_about_info` command + `data_dir` managed state + `invoke_handler!` registration.

**NOT this milestone:** the Help menu · the About dialog · the logo assets · any `ui/**` file · any `dialog.svelte` change. Those are **C3**. **Scope-clean = no `ui/**` at all.**

**Do NOT touch:** `xgen-client/src/ops.rs`. About-info is **not** a protocol verb — `ops.rs` is the M5/D-067 canonical layer for network+state verbs (CLI / batch / aicontrol dispatchers). About-info has no node round-trip, no mutation, no CLI meaning. **It is not a D-092 four-armed verb** and must not grow four arms.

---

## 4. Verify plan — REAL CLIENT 9222 (D-097; Rule 2, quote real output)

The sampler **cannot** host this: it is a `tauri`+`tauri-build`-only crate with no `xgen-common`/protocol deps and no `get_about_info`. Verify in the **real client** (`run-client.ps1 -Debug`).

1. **`cargo build` / `cargo test`** — workspace green. **Quote the real test count** (Rule 5 — do not carry a remembered number forward).
2. **CDP `invoke`** — in the real client (9222):
   `await window.__TAURI_INTERNALS__.invoke('get_about_info')` → quote the **actual returned JSON**.
3. **Field-by-field truth check** — every field must be *checkable against something else*, not merely present:
   - `version` — equals `xgen-client`'s **own** Cargo.toml version (**0.10.3** today). ⚠️ Prove it is not accidentally `build_info::VERSION`: they are **both 0.10.3 right now**, so a passing-looking value proves nothing. State how you established it (e.g. the call site passes `env!("CARGO_PKG_VERSION")` from `xgen-client`).
   - `commit` — equals `git rev-parse --short HEAD` **at the time the crate was last compiled** (§2.4). Compare against the real `git` output and explain any difference rather than hand-waving it.
   - `rustc` — matches a real `rustc -V` on this machine.
   - `tauri` — **2.11.1** (matches `Cargo.lock`).
   - `svelte` — **5.55.5** (matches `ui/client/package-lock.json`). **If it comes back `^5`, §2.3 was not followed.**
   - `platform` — real OS/arch.
   - `app_dir` / `data_dir` / `config_path` — **real, existing paths**. Prove `config_path` points at a file that actually exists.
4. **No permission denial** — the J-495 error trap (`console.error` + `onerror` + `unhandledrejection`): `errCount:0`, `permissionDenials:[]`. This is what confirms §2.0 (3): the command works with **no** capability grant.
5. **Client registry unchanged** — C2 ships no UI, so the client's own registry stays **7** (J-495's measured number) and the **sampler catalogue stays 313**. **Measure both; never predict** (Rule 5).
6. **`vite build`** — clean (no frontend change expected; prove it).

**PS 5.1 (N-086):** wrap eval returns as a **JSON object**; single-expression evals are the reliable form (N-089). `__TAURI_INTERNALS__.invoke` is **non-configurable** — do not attempt to stub it.

---

## 5. Rule-6 confirm points (ground it, don't guess)

- **`ClientAboutInfo` is a zero-extension wrapper today** (§2.2). Kept deliberately as the typed seam. If you disagree, **flag it** — don't silently collapse or silently keep it.
- **`serde_json` as a `[build-dependencies]`** — confirm it resolves without pulling anything unexpected into the build graph.
- **`rustc -V` in `build.rs`** — confirm it runs in this environment (it should; `git` already does).
- If **any** grounded fact in §2.0 turns out to be wrong on contact with the code, **stop and report** (Rule 3). §2.0 exists because three earlier assumptions were wrong; a fourth is entirely possible.

---

## 6. Definition of Done

- [ ] `xgen-common/build.rs` emits `BUILD_RUSTC_VERSION`; `build_info::RUSTC_VERSION` added. **No new build-metadata surface anywhere else.**
- [ ] `xgen-common/src/about.rs` — `AboutInfo` + `collect()` + `ClientAboutInfo`; paths + app facts **passed in**; **no `tauri` dep in `xgen-common`**.
- [ ] `version` passed from **`xgen-client`'s own** `CARGO_PKG_VERSION` (not `build_info::VERSION`).
- [ ] `xgen-client/build.rs` emits the **resolved** Svelte version from `package-lock.json` (**5.55.5**, not `^5`).
- [ ] `get_about_info` registered + returning; **no capability change**; `data_dir` managed (not derived from the config path).
- [ ] Workspace `cargo test` green — **count quoted from real output**.
- [ ] All 6 verify legs in §4 run against the **real client 9222**, with **actual quoted output**, incl. the full returned JSON and the field-by-field truth check.
- [ ] Client registry **measured 7**; sampler catalogue **measured 313**. Neither predicted.
- [ ] Scope-clean: **no `ui/**`**, no `ops.rs`.
- [ ] Any deviation **flagged, not absorbed** (Rule 6).

*(Per the task-file DoD rule: "commit pushed" is deliberately NOT a checklist item. `Status: COMPLETED` in the header is the real signal.)*

---

## 7. Close (D-074, two commits)

1. **Clair — feat commit** (code only): the 7 files in §3.
2. **Chat — doc-bridge commit**: `JOURNAL.md` (J-series) · `CLAUDE.md` PLAY · `docs/ROADMAP.md` · `docs/xgen-client-frame-phase0.md` (§6 / §10.4 — C2 ✅) · `ui/docs/xgen-ui-notes.md` (N-090 **only if** there is a UI-side lesson; this is a Rust milestone, so possibly none — **do not invent one**) · this file → **COMPLETED**.

Joe pushes both. Chat never pushes.

---

*End of M-RP6.1e-C2 runbook.*
