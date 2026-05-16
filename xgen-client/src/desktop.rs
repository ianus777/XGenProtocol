// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Tauri desktop shell for xgen-client (D-062, D-063). Migrated into the
//! library crate by M1 Phase 2a so the single `xgen-client` binary can
//! initialise the UI when launched without `--service`.
//!
//! 2a status: runs first-run detection, pipe server, and the existing
//! auto-connect lifecycle scaffold (Initialising → Setup OR Connecting →
//! Authenticating → Ready/Disconnected). The full long-lived client resident
//! (sustained WS to home Node, history sync, real-time fan-out) is wired in
//! 2b / M3.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};

use crate::lifecycle::{make_state_event, ClientLifecycleState, ClientStateEvent};
use crate::pacing::{PacingManager, PacingState};
use crate::temperature::TemperatureUpdate;
use xgen_common::event_trace::{write_session_footer, write_session_header, ExitReason};

// ── Shared state ───────────────────────────────────────────────────────────────

struct CurrentState(Arc<Mutex<ClientStateEvent>>);

/// Holds the watch sender so the quit command can signal the pipe server.
struct PipeShutdown(tokio::sync::watch::Sender<bool>);

/// Outbound pacing queue manager (Ch6 §6.14). The Tauri webview reads the
/// current snapshot via `get_pacing_state`.
struct Pacing(Arc<Mutex<PacingManager>>);

fn emit_state(app: &AppHandle, state: ClientLifecycleState) {
    let canonical = state.as_canonical();
    tracing::info!(lifecycle_state = canonical, "lifecycle transition");
    let payload = make_state_event(state);
    if let Ok(mut stored) = app.state::<CurrentState>().0.lock() {
        *stored = payload.clone();
    }
    let _ = app.emit("xgen-client-state-changed", &payload);
}

// ── Tauri commands ─────────────────────────────────────────────────────────────

/// Returns the current lifecycle state. Called by Svelte on mount so it gets
/// the live state regardless of when the webview finished loading.
#[tauri::command]
fn get_state(state: tauri::State<CurrentState>) -> ClientStateEvent {
    state.0.lock().unwrap().clone()
}

/// Read-only snapshot of the outbound pacing queue for a Space (Ch6 §6.14.4).
/// Returns one entry per sender that has activity recorded. The Svelte layer
/// projects this into `data-pacing-state` / `--xgen-pacing-*` custom property
/// values (Ch6 §6.14.3, §6.14.4).
#[tauri::command]
fn get_pacing_state(space_id: String, pacing: tauri::State<Pacing>) -> Vec<PacingState> {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    pacing
        .0
        .lock()
        .map(|m| m.snapshots_for_space(&space_id, now_ms))
        .unwrap_or_default()
}

/// Emit a temperature update to the Svelte layer (spec 3.7.13, Ch6 §6.12).
/// Invoked by the Node event ingest pipeline when an incoming Event carries
/// `xgen.room_temperature` or `xgen.member_temperature` in its meta_atts. The
/// Svelte layer projects this into `data-temp-state` and the
/// `--xgen-*-temperature` custom properties.
///
/// Not yet called from the current scaffold (no Phase 2 ingest path wired
/// into the Tauri shell yet); reserved as the API surface the ingest will
/// use once it lands.
#[allow(dead_code)]
fn emit_temperature_update(app: &AppHandle, update: &TemperatureUpdate) {
    let _ = app.emit("xgen-temperature-update", update);
}

#[tauri::command]
fn quit(app: AppHandle) {
    emit_state(&app, ClientLifecycleState::Closing);
    // Signal the pipe server to shut down before exiting.
    let _ = app.state::<PipeShutdown>().0.send(true);
    write_session_footer(ExitReason::Shutdown);
    app.exit(0);
}

// ── Startup sequence ───────────────────────────────────────────────────────────

async fn run_startup(
    app: AppHandle,
    data_dir: PathBuf,
    pipe_name: String,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    // Always emit INITIALISING first, regardless of first-run state.
    emit_state(&app, ClientLifecycleState::Initialising);

    // M1 §1.1 — start the named pipe server on a dedicated task.
    #[cfg(target_os = "windows")]
    {
        let data_dir_clone = data_dir.clone();
        let pipe_name_clone = pipe_name.clone();
        let rx_clone = shutdown_rx.clone();
        tauri::async_runtime::spawn(async move {
            crate::batch::start_pipe_server(pipe_name_clone, data_dir_clone, rx_clone).await;
        });
    }

    let config_path = data_dir.join("xgen-client_config.toml");
    let keypair_path = data_dir.join("xgen-client_keypair.enc");

    // First-run detection: neither config nor keypair exists.
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

// ── Entry point ────────────────────────────────────────────────────────────────

/// Launch the Tauri desktop shell. Returns when the user shuts the app down.
///
/// `data_dir` — Tier-1 runtime files live here. Caller resolves it from the
/// `--instance` label (or `exe_dir()` when no label is set).
/// `instance_label` — for pipe name derivation. None → default pipe name.
pub fn run(
    data_dir: PathBuf,
    instance_label: Option<String>,
    log_level_override: Option<String>,
) {
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    let log_dir = data_dir.join("logs");
    std::fs::create_dir_all(&log_dir).expect("failed to create logs/");
    let now = chrono::Local::now();
    let log_filename = format!("xgen-client_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));
    let log_path = log_dir.join(&log_filename);
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("failed to open log file");

    use tracing_subscriber::{fmt, EnvFilter};
    // Precedence: --log-level > XGEN_LOG > "debug".
    let env_filter = if let Some(ref lvl) = log_level_override {
        EnvFilter::new(lvl)
    } else {
        EnvFilter::try_from_env("XGEN_LOG").unwrap_or_else(|_| EnvFilter::new("debug"))
    };
    fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_ansi(false)
        .with_writer(log_file)
        .init();

    let started_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let session_id = format!("{:08x}", rand::random::<u32>());
    write_session_header(
        "client",
        None,
        None,
        None,
        "0.1",
        env!("CARGO_PKG_VERSION"),
        &session_id,
        &started_at,
    );

    // M1 §1.2 — derive pipe name from instance label.
    let pipe_name_str = crate::batch::pipe_name(instance_label.as_deref());

    // Write the PID file so `--pid` can find this resident.
    crate::app::write_pid_file(&data_dir);

    // Shutdown channel: sender stored as Tauri state, receiver passed to pipe server.
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Initial stored state — Initialising until run_startup determines actual state.
    let initial = make_state_event(ClientLifecycleState::Initialising);
    let shared_state = CurrentState(Arc::new(Mutex::new(initial)));

    let pacing_manager = Pacing(Arc::new(Mutex::new(PacingManager::new())));

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .manage(shared_state)
        .manage(PipeShutdown(shutdown_tx))
        .manage(pacing_manager)
        .setup(move |app| {
            let handle = app.handle().clone();
            let dir = data_dir.clone();
            let pn = pipe_name_str.clone();
            let rx = shutdown_rx.clone();
            tauri::async_runtime::spawn(async move {
                run_startup(handle, dir, pn, rx).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_state, get_pacing_state, quit])
        .run(tauri::generate_context!())
        .expect("error while running xgen-client desktop shell");

    write_session_footer(ExitReason::Shutdown);
}
