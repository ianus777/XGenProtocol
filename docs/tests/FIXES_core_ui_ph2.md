# Core Test UI — Bug Fixes (Phase 2)
> **Status**: PENDING  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

Four bugs identified during code review of the completed Core Test UI (Client + Node). Fix them in the order listed, run the verification checklist after all four are applied, and append results at the bottom of this file.

---

## Fix 1 — Client startup sequence skips INITIALISING on first run

**File:** `xgen-client/src-tauri/src/main.rs`  
**Function:** `run_startup`

**What is wrong:**

The current code goes directly to `SETUP` on first run, bypassing `INITIALISING`:

```rust
// CURRENT — wrong order
if !config_path.exists() && !keypair_path.exists() {
    emit_state(&app, ClientLifecycleState::Setup);
    return;  // INITIALISING is never emitted on first run
}
emit_state(&app, ClientLifecycleState::Initialising);  // only reached on non-first run
```

The spec (CLIENT_CORE_UI_ph2.md §2.5) requires: always emit `INITIALISING` first, then
transition to `SETUP` if it is a first run. This also explains the screenshot showing
"Setting up" with no prior "Initialising" flash.

**Fix:**

Rewrite `run_startup` so `INITIALISING` is always the first emitted state and paths are
derived from `data_dir` (which also fixes Fix 2 — see below):

```rust
async fn run_startup(app: AppHandle, data_dir: std::path::PathBuf) {
    // Always start here, regardless of whether this is a first run.
    emit_state(&app, ClientLifecycleState::Initialising);

    let config_path = data_dir.join("xgen-client_config.toml");
    let keypair_path = data_dir.join("xgen-client_keypair.enc");

    // First-run detection: neither config nor keypair exists yet.
    if !config_path.exists() && !keypair_path.exists() {
        emit_state(&app, ClientLifecycleState::Setup);
        return;
    }

    // Auto-connect: attempt ws://127.0.0.1:8080/xgen with 2-second timeout.
    emit_state(&app, ClientLifecycleState::Connecting);

    let connect_result = tokio::time::timeout(
        tokio::time::Duration::from_millis(2000),
        tokio_tungstenite::connect_async("ws://127.0.0.1:8080/xgen"),
    )
    .await;

    match connect_result {
        Ok(Ok(_stream)) => {
            emit_state(&app, ClientLifecycleState::Authenticating);
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            emit_state(&app, ClientLifecycleState::Ready);
        }
        _ => {
            emit_state(&app, ClientLifecycleState::Disconnected);
        }
    }
}
```

---

## Fix 2 — Client run_startup ignores data_dir; uses exe_dir() for config/keypair paths

**File:** `xgen-client/src-tauri/src/main.rs`  
**Functions:** `run_startup`, setup closure in `main`

**What is wrong:**

The first-run detection looks in the executable directory regardless of the `--instance` flag:

```rust
// CURRENT — ignores --instance, always checks exe dir
let config_path = exe_dir().join("xgen-client_config.toml");
let keypair_path = exe_dir().join("xgen-client_keypair.enc");
```

When running with `--instance <label>`, the data directory is `instances/<label>/` and the
config and keypair live there. Looking in the exe directory means the first-run check always
returns "first run" for named instances even after they have been set up.

Additionally, `data_dir` is computed in `main()` and captured in the setup closure, but
is never passed to `run_startup`. It is silently discarded with `let _ = dir;`:

```rust
// CURRENT — data_dir available but not forwarded
tauri::async_runtime::spawn(async move {
    run_startup(handle).await;
    let _ = dir;  // ← data_dir is here, run_startup never sees it
});
```

**Fix:**

Fix 1 above already adds `data_dir: std::path::PathBuf` as a parameter to `run_startup`
and uses it for the config/keypair paths. The only remaining change is in the setup
closure — pass `dir` to `run_startup` and remove the dead `let _ = dir;` line:

```rust
// FIXED setup closure
.setup(move |app| {
    let handle = app.handle().clone();
    let dir = data_dir.clone();
    tauri::async_runtime::spawn(async move {
        run_startup(handle, dir).await;
    });
    Ok(())
})
```

Fixes 1 and 2 are applied together in a single edit to `run_startup` and its call site.

---

## Fix 3 — Hardcoded version string "0.10.3" in both Tauri main.rs files

**Files:**
- `xgen-client/src-tauri/src/main.rs`
- `xgen-node/src-tauri/src/main.rs`

**What is wrong:**

Both files pass a hardcoded version literal to `write_session_header`:

```rust
write_session_header(
    "client",  // or "node"
    None,
    None,
    None,
    "0.1",
    "0.10.3",  // ← hardcoded, will silently go stale as the project advances
    &session_id,
    &started_at,
);
```

**Fix:**

Replace `"0.10.3"` with `env!("CARGO_PKG_VERSION")` in both files:

```rust
write_session_header(
    "client",  // or "node"
    None,
    None,
    None,
    "0.1",
    env!("CARGO_PKG_VERSION"),
    &session_id,
    &started_at,
);
```

`env!("CARGO_PKG_VERSION")` is resolved at compile time from the crate's `Cargo.toml`
`version` field. It will always match the actual binary version without any manual update.

---

## Fix 4 — Node admin window starts visible on launch (D-037 violation)

**File:** `xgen-node/src-tauri/tauri.conf.json`

**What is wrong:**

```json
"windows": [
  {
    "title": "XGen Node",
    "label": "main",
    "width": 420,
    "height": 260,
    "decorations": false,
    "resizable": false,
    "center": true,
    "visible": true      ← window opens immediately on launch
  }
]
```

Per D-037 the Node is process-centric: the systray icon is the entry point and the admin
window is on-demand. Opening the window automatically on launch breaks this model.

**Fix:**

Change `"visible": true` to `"visible": false`:

```json
"windows": [
  {
    "title": "XGen Node",
    "label": "main",
    "width": 420,
    "height": 260,
    "decorations": false,
    "resizable": false,
    "center": true,
    "visible": false
  }
]
```

No other changes required. The existing systray "Open Admin Panel" handler (`window.show()`)
and the window hide-on-close handler are already correct.

---

## Verification

Apply all four fixes, then run from the workspace root:

```
cargo build
cargo test
```

Confirm: clean compile with no warnings, 173/173 tests passing.

Then verify manually:

**Fix 1 + Fix 2 — Client startup sequence:**

*First-run path (no instance):*
- Delete or rename `xgen-client_config.toml` and `xgen-client_keypair.enc` from the exe
  directory to simulate a first run
- Launch `xgen-client` — state indicator must show `Initialising` first, then transition
  to `Setting up`

*Normal path (config present, no Node running):*
- Restore a config file in the exe directory
- Launch `xgen-client` with no Node running — state must show `Initialising`, then
  `Connecting`, then `Disconnected` after the 2-second timeout

*Instanced first-run path:*
- Ensure `instances/test/` does not exist (or contains no config/keypair)
- Launch `xgen-client --instance test`
- State must show `Initialising`, then `Setting up`
- The instance directory `instances/test/` must be created and the log written there
- No config/keypair lookup must happen in the exe directory

**Fix 3 — Version in log headers:**
- Launch either binary and open its log file
- The session header line must show the version from the crate's `Cargo.toml`, not `0.10.3`

**Fix 4 — Node window hidden on launch:**
- Launch `xgen-node` — no admin window must appear on screen
- The systray icon must appear
- Click "Open Admin Panel" — the admin window must open
- Close the window — it hides, systray icon remains
- Click "Open Admin Panel" again — window re-opens
- "Shut Down" from systray must terminate the process cleanly

---

## Checklist

- [ ] Fix 1 applied — `INITIALISING` emitted before `SETUP` on first run
- [ ] Fix 2 applied — `run_startup` receives `data_dir`, uses it for all path lookups
- [ ] Fix 3 applied — `env!("CARGO_PKG_VERSION")` in both `main.rs` files
- [ ] Fix 4 applied — `"visible": false` in `xgen-node/src-tauri/tauri.conf.json`
- [ ] `cargo build` — clean compile, no warnings
- [ ] `cargo test` — 173/173 tests passing
- [ ] Fix 1+2 verified — client first-run shows `Initialising → Setting up`
- [ ] Fix 1+2 verified — instanced first-run reads/writes instance directory correctly
- [ ] Fix 3 verified — session header shows correct version in log
- [ ] Fix 4 verified — node window hidden on launch, shown via systray

---

## Verification Results

*(To be filled in by Mr. Code)*
