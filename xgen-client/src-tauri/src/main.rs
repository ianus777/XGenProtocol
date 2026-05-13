// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Hides the console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};
use xgen_client_lib::lifecycle::{ClientLifecycleState, ClientStateEvent, make_state_event};
use xgen_common::event_trace::{ExitReason, write_session_footer, write_session_header};

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn validate_instance_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 64
        && label.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

// ── Shared state ───────────────────────────────────────────────────────────────

struct CurrentState(Arc<Mutex<ClientStateEvent>>);

/// Holds the watch sender so the quit command can signal the pipe server.
struct PipeShutdown(tokio::sync::watch::Sender<bool>);

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
            xgen_client_lib::batch::start_pipe_server(
                pipe_name_clone,
                data_dir_clone,
                rx_clone,
            )
            .await;
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

// ── Instance data directory ────────────────────────────────────────────────────

fn resolve_data_dir() -> (PathBuf, Option<String>) {
    let args: Vec<String> = std::env::args().collect();
    let label = args
        .windows(2)
        .find(|w| w[0] == "--instance")
        .map(|w| w[1].clone());

    if let Some(ref l) = label {
        if !validate_instance_label(l) {
            eprintln!(
                "error: --instance label {:?} is invalid. \
                 Use only letters, digits, hyphens, and underscores (max 64 chars).",
                l
            );
            std::process::exit(1);
        }
    }

    let dir = match &label {
        Some(l) => exe_dir().join("instances").join(l),
        None => exe_dir(),
    };
    (dir, label)
}

// ── Entry point ────────────────────────────────────────────────────────────────

fn main() {
    let (data_dir, instance_label) = resolve_data_dir();

    // M2 §2.1 — detect --batch before the Tauri builder is invoked.
    // If present, run the batch client path (no window, no Tauri).
    {
        let args: Vec<String> = std::env::args().collect();
        if let Some(idx) = args.iter().position(|a| a == "--batch") {
            let raw_path = args.get(idx + 1).map(|s| s.as_str()).unwrap_or("");
            if raw_path.is_empty() || raw_path.starts_with('-') {
                eprintln!("error: --batch requires a file path argument");
                std::process::exit(2);
            }
            let pn = xgen_client_lib::batch::pipe_name(instance_label.as_deref());
            #[cfg(target_os = "windows")]
            std::process::exit(xgen_client_lib::batch::run_batch_client(raw_path, &pn, instance_label.as_deref()));
            #[cfg(not(target_os = "windows"))]
            {
                eprintln!("error: --batch is only supported on Windows");
                std::process::exit(2);
            }
        }
    }

    std::fs::create_dir_all(&data_dir).expect("Failed to create instance data directory");

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

    use tracing_subscriber::fmt;
    fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("XGEN_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
        )
        .with_target(true)
        .with_ansi(false)
        .with_writer(log_file)
        .init();

    let started_at = chrono::Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
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
    let pipe_name_str = xgen_client_lib::batch::pipe_name(instance_label.as_deref());

    // Shutdown channel: sender stored as Tauri state, receiver passed to pipe server.
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Initial stored state — Initialising until run_startup determines actual state.
    let initial = make_state_event(ClientLifecycleState::Initialising);
    let shared_state = CurrentState(Arc::new(Mutex::new(initial)));

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .manage(shared_state)
        .manage(PipeShutdown(shutdown_tx))
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
        .invoke_handler(tauri::generate_handler![get_state, quit])
        .run(tauri::generate_context!())
        .expect("error while running xgen-client-app");

    write_session_footer(ExitReason::Shutdown);
}
