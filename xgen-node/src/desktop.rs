// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Tauri desktop shell for xgen-node (D-062, D-063). Migrated into the
//! library crate by M1 Phase 2a so the single `xgen-node` binary can
//! initialise the UI when launched without `--service`.
//!
//! 2b status: desktop mode spawns the real `app::run_node()` WS server
//! alongside Tauri in the same process. Tauri owns the main thread; the WS
//! server runs as a background tokio task in Tauri's async runtime. Logging
//! is initialised here and `run_node` is told to skip its own init (via the
//! `init_logging=false` flag). On `run_node` failure (missing keypair, port
//! conflict, etc.) the window stays open and the lifecycle transitions to
//! `DegradedStorage` so the user can see something is wrong without a crash.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::app::{self, NodeConfig, RunNodeOpts};
use crate::lifecycle::{make_node_state_event, NodeLifecycleState, NodeStateEvent};
use xgen_common::event_trace::{write_session_footer, write_session_header, ExitReason};

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Build the TOML content for a first-launch default config rooted at
/// `data_dir` with the operator's `--port` value (or 8080 default). All
/// `NodeConfig` schema fields are populated so the file roundtrips through
/// `toml::from_str::<NodeConfig>` cleanly. Path fields point under `data_dir`
/// so per-instance segregation works (the previous bogus `port = N`-only form
/// failed to parse, and run_node silently fell back to exe_dir paths).
pub(crate) fn default_config_toml(data_dir: &Path, port: u16) -> String {
    let mut cfg = NodeConfig::default();
    cfg.node.listen = format!("ws://127.0.0.1:{}/xgen", port);
    cfg.paths.keypair_path = data_dir
        .join("xgen-node_keypair.enc")
        .to_string_lossy()
        .to_string();
    cfg.paths.spaces_dir = Some(data_dir.join("spaces").to_string_lossy().to_string());
    let body = toml::to_string_pretty(&cfg).expect("NodeConfig serialises");
    format!("# XGen Node default configuration\n{body}")
}

fn maybe_write_default_config(data_dir: &Path, port: u16) {
    let config_path = data_dir.join("xgen-node_config.toml");
    if !config_path.exists() {
        let content = default_config_toml(data_dir, port);
        let _ = std::fs::write(&config_path, content);
    }
}

/// D-101 clean-slate-on-start (phase-scoped). Config is treated as ephemeral
/// while the settings logic is still in development: wipe any config found at
/// launch, then regenerate the default from seed BEFORE it is read (`init_logging`
/// and `run_node` both read it after this returns), so no launch inherits a
/// stale (or another binary's) file. The node has no substitutions consumer —
/// this keeps the discipline uniform across all three binaries.
///
/// **This SUSPENDS J-438 seed-once for the phase** on the client side (the node
/// has no such seed); the discipline itself — regenerate-from-seed every launch —
/// is retired when the client/node UIs are rewritten with persistent settings.
/// See DECISIONS.md D-101 for the full why + exit condition.
fn clean_slate_config(data_dir: &Path, port: u16) {
    let config_path = data_dir.join("xgen-node_config.toml");
    if config_path.exists() {
        let _ = std::fs::remove_file(&config_path);
    }
    maybe_write_default_config(data_dir, port);
}

// ── Logging init ───────────────────────────────────────────────────────────────

fn init_logging(data_dir: &Path, log_level_override: Option<&str>) {
    let log_dir = data_dir.join("logs");
    std::fs::create_dir_all(&log_dir).expect("failed to create logs/");
    let now = chrono::Local::now();
    let log_filename = format!("xgen-node_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));
    let log_path = log_dir.join(&log_filename);
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("failed to open log file");

    use tracing_subscriber::{fmt, EnvFilter};
    // D-068 — flag > env (XGEN_LOG) > config (`[logging].level`) > "debug".
    // Pre-J-079 this path fell back to `EnvFilter::new("debug")`, silently
    // ignoring config. The convergence ships in J-079 commit 3.
    let config_path = data_dir.join("xgen-node_config.toml");
    let config_level = app::read_config_log_level(&config_path);
    let env_filter = EnvFilter::new(xgen_common::precedence::resolve_log_level(
        log_level_override,
        config_level.as_deref(),
    ));
    fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_ansi(false)
        .with_writer(log_file)
        .init();
}

// ── Shared state ───────────────────────────────────────────────────────────────

struct NodeAppState {
    primary: NodeLifecycleState,
    degraded: HashSet<NodeLifecycleState>,
}

struct CurrentNodeState(Arc<Mutex<(NodeAppState, NodeStateEvent)>>);

// ── Emit helpers ───────────────────────────────────────────────────────────────

fn emit_node_state(app: &AppHandle, primary: NodeLifecycleState) {
    let shared = app.state::<CurrentNodeState>();
    let payload = {
        let mut guard = shared.0.lock().unwrap();
        guard.0.primary = primary;
        let evt = make_node_state_event(guard.0.primary.clone(), &guard.0.degraded);
        guard.1 = evt.clone();
        evt
    };
    tracing::info!(lifecycle_state = payload.state.as_canonical(), "lifecycle transition");
    let _ = app.emit("xgen-node-state-changed", &payload);
}

/// Add a degraded condition without overwriting the primary state. The
/// lifecycle module's display logic (`active_display_state`) expects degraded
/// states to live in the degraded set, not as primary.
fn emit_node_degraded(app: &AppHandle, kind: NodeLifecycleState) {
    debug_assert!(kind.is_degraded(), "emit_node_degraded called with non-degraded state");
    let shared = app.state::<CurrentNodeState>();
    let payload = {
        let mut guard = shared.0.lock().unwrap();
        guard.0.degraded.insert(kind);
        let evt = make_node_state_event(guard.0.primary.clone(), &guard.0.degraded);
        guard.1 = evt.clone();
        evt
    };
    tracing::info!(lifecycle_state = payload.state.as_canonical(), "lifecycle transition (degraded)");
    let _ = app.emit("xgen-node-state-changed", &payload);
}

// ── Tauri commands ─────────────────────────────────────────────────────────────

#[tauri::command]
fn get_node_state(state: tauri::State<CurrentNodeState>) -> NodeStateEvent {
    state.0.lock().unwrap().1.clone()
}

#[tauri::command]
fn shut_down(app: AppHandle, _state: tauri::State<CurrentNodeState>) {
    emit_node_state(&app, NodeLifecycleState::Closing);
    write_session_footer(ExitReason::Shutdown);
    app.exit(0);
}

// ── Startup sequence (Tauri mode) ──────────────────────────────────────────────

/// Spawn `run_node` and drive lifecycle transitions from it.
///
/// `run_node` does not currently emit lifecycle events itself, so the
/// transition out of Initialising is coarse — we sleep briefly to give the
/// bind a chance and then emit Ready unconditionally. On failure (no
/// keypair, port in use, etc.) the spawned task inserts DegradedStorage into
/// the degraded set; once primary moves to Ready the display logic surfaces
/// DEGRADED_STORAGE (per `active_display_state`). Surfacing failures inside
/// `run_node`'s accept loop after the initial bind is 2c / future work.
async fn run_startup(
    app: AppHandle,
    config_path: PathBuf,
    data_dir: PathBuf,
    port_override: Option<u16>,
    instance_label: Option<String>,
) {
    emit_node_state(&app, NodeLifecycleState::Initialising);

    // Spawn run_node as a background task. It owns the WS bind + accept loop
    // and returns when ctrl_c fires or an error occurs.
    let app_for_task = app.clone();
    tauri::async_runtime::spawn(async move {
        let desktop_opts = RunNodeOpts {
            init_logging: false,
            quiet: true,
            port_override,
            instance_label,
            ..RunNodeOpts::default()
        };
        match app::run_node(&config_path, &data_dir, desktop_opts).await {
            Ok(()) => {
                tracing::info!("run_node returned Ok — node shut down cleanly");
            }
            Err(e) => {
                tracing::error!(reason = %format!("{:#}", e), "run_node failed");
                emit_node_degraded(&app_for_task, NodeLifecycleState::DegradedStorage);
            }
        }
    });

    // Always exit Initialising after the timer; the degraded set carries the
    // story if `run_node` failed before this point. If `run_node` fails after
    // this point, the spawned task above will still flip the degraded set.
    tokio::time::sleep(Duration::from_millis(500)).await;
    emit_node_state(&app, NodeLifecycleState::Ready);
}

// ── Entry point ────────────────────────────────────────────────────────────────

/// Launch the Tauri desktop shell. Returns when the user shuts the app down.
///
/// `config_path` — path to the Node config file. Passed through to `run_node`.
/// `data_dir` — Tier-1 runtime files live here. Caller resolves it from the
/// `--instance` label (or `exe_dir()` when no label is set).
/// `port_override` — `--port` flag value if the operator passed one (D-068).
/// Threaded into `RunNodeOpts.port_override` and resolved at the bind site
/// against `[node].listen` from config. Also used as the value written into a
/// fresh `xgen-node_config.toml` if one doesn't already exist (the
/// `maybe_write_default_config` path); the broken-non-schema-field detail of
/// that path is out of scope for D-068 (init flow) — see J-079 §4 carry-overs.
pub fn run(
    config_path: PathBuf,
    data_dir: PathBuf,
    port_override: Option<u16>,
    log_level_override: Option<String>,
    instance_label: Option<String>,
) {
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    // D-101 clean-slate-on-start (phase-scoped) — wipe any config found, then
    // regenerate the default from seed before it is read below. See D-101.
    clean_slate_config(&data_dir, port_override.unwrap_or(8080));

    init_logging(&data_dir, log_level_override.as_deref());

    let started_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let session_id = format!("{:08x}", rand::random::<u32>());
    write_session_header(
        "node",
        None,
        None,
        None,
        "0.1",
        env!("CARGO_PKG_VERSION"),
        &session_id,
        &started_at,
    );

    let initial_state = NodeAppState {
        primary: NodeLifecycleState::Initialising,
        degraded: HashSet::new(),
    };
    let initial_event =
        make_node_state_event(NodeLifecycleState::Initialising, &HashSet::new());
    let shared = CurrentNodeState(Arc::new(Mutex::new((initial_state, initial_event))));

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .manage(shared)
        .setup(move |app| {
            // ── Systray ────────────────────────────────────────────────────────
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;

            let show_item =
                MenuItem::with_id(app, "show", "Open Admin Panel", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Shut Down", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let tray_icon = app
                .default_window_icon()
                .expect("no window icon configured")
                .clone();

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("XGen Node")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                    "quit" => {
                        emit_node_state(app, NodeLifecycleState::Closing);
                        write_session_footer(ExitReason::Shutdown);
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // ── Window hide-on-close ───────────────────────────────────────────
            let window = app.get_webview_window("main").unwrap();
            let win2 = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    win2.hide().ok();
                }
            });

            // ── Startup sequence ───────────────────────────────────────────────
            let handle = app.handle().clone();
            let cp = config_path.clone();
            let dd = data_dir.clone();
            let po = port_override;
            let il = instance_label.clone();
            tauri::async_runtime::spawn(async move {
                run_startup(handle, cp, dd, po, il).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_node_state, shut_down])
        .run(tauri::generate_context!())
        .expect("error while running xgen-node desktop shell");

    write_session_footer(ExitReason::Shutdown);
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// The fresh-init config written by `default_config_toml` MUST parse back
    /// through `NodeConfig`. The pre-J-080 form wrote a non-schema `port = N`
    /// field that `toml::from_str::<NodeConfig>` rejected, after which
    /// `run_node` silently fell back to `NodeConfig::default()` (losing the
    /// per-instance `paths.keypair_path` derivation).
    #[test]
    fn default_config_roundtrips_through_nodeconfig() {
        let data_dir = Path::new("/tmp/xgen-node-test");
        let toml_str = default_config_toml(data_dir, 9081);
        let cfg: NodeConfig =
            toml::from_str(&toml_str).expect("default config must parse as NodeConfig");
        assert_eq!(cfg.node.listen, "ws://127.0.0.1:9081/xgen");
        assert!(cfg.paths.keypair_path.contains("xgen-node-test"));
        assert!(cfg.paths.keypair_path.ends_with("xgen-node_keypair.enc"));
        assert!(cfg.paths.spaces_dir.as_deref().is_some_and(|s| s.ends_with("spaces")));
        assert!(!cfg.logging.level.is_empty());
    }

    /// Port override threads through to the rendered `listen` URL; the rest
    /// of the URL shape stays default.
    #[test]
    fn default_config_honours_port_override() {
        let data_dir = Path::new("/tmp/xgen-node-test-port");
        let toml_str = default_config_toml(data_dir, 18080);
        assert!(toml_str.contains("ws://127.0.0.1:18080/xgen"));
    }

    /// D-101 clean-slate-on-start — a pre-existing (stale / edited) config is
    /// wiped + regenerated to the default seed on the startup path. The old
    /// contents must be gone and the fresh file must parse as `NodeConfig` with
    /// the requested port.
    #[test]
    fn clean_slate_wipes_and_regenerates_node_config() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("xgen-node_config.toml");

        // A stale config the operator (or a prior launch) left behind.
        std::fs::write(&config_path, "# stale marker\nrandom garbage not a NodeConfig\n").unwrap();

        clean_slate_config(dir.path(), 9099);

        let regenerated = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            !regenerated.contains("random garbage"),
            "the stale config is wiped, not merged"
        );
        let cfg: NodeConfig =
            toml::from_str(&regenerated).expect("regenerated config parses as NodeConfig");
        assert_eq!(cfg.node.listen, "ws://127.0.0.1:9099/xgen");
    }

    /// D-101 — on a genuine first run (no config), clean-slate still produces a
    /// fresh default config (nothing to wipe → regenerate).
    #[test]
    fn clean_slate_creates_config_on_first_run() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("xgen-node_config.toml");
        assert!(!config_path.exists());

        clean_slate_config(dir.path(), 8080);

        assert!(config_path.exists(), "a fresh default config is generated");
        let cfg: NodeConfig =
            toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(cfg.node.listen, "ws://127.0.0.1:8080/xgen");
    }
}
