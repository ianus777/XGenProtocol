// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Hides the console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use xgen_node_lib::lifecycle::{
    make_node_state_event, NodeLifecycleState, NodeStateEvent,
};
use xgen_common::event_trace::{ExitReason, write_session_footer, write_session_header};

// ── Helpers ────────────────────────────────────────────────────────────────────

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

// ── Flag parsing ───────────────────────────────────────────────────────────────

struct Flags {
    service_mode: bool,
    instance_label: Option<String>,
    port_override: Option<u16>,
}

fn parse_flags() -> Flags {
    let args: Vec<String> = std::env::args().collect();

    let service_mode = args.iter().any(|a| a == "--service");

    let instance_label = args.windows(2)
        .find(|w| w[0] == "--instance")
        .map(|w| w[1].clone());

    let port_override = args.windows(2)
        .find(|w| w[0] == "--port")
        .and_then(|w| w[1].parse::<u16>().ok());

    Flags { service_mode, instance_label, port_override }
}

fn resolve_data_dir(instance_label: &Option<String>) -> PathBuf {
    match instance_label {
        Some(l) => exe_dir().join("instances").join(l),
        None    => exe_dir(),
    }
}

fn maybe_write_default_config(data_dir: &PathBuf, port: u16) {
    let config_path = data_dir.join("xgen-node_config.toml");
    if !config_path.exists() {
        let content = format!(
            "# XGen Node default configuration\nport = {}\n",
            port
        );
        let _ = std::fs::write(&config_path, content);
    }
}

// ── Logging init ───────────────────────────────────────────────────────────────

fn init_logging(data_dir: &PathBuf) {
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

/// Minimal state logging for --service mode (no Tauri).
fn emit_state_service(state: &NodeLifecycleState) {
    tracing::info!(lifecycle_state = state.as_canonical(), "lifecycle transition (service)");
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

async fn run_startup(app: AppHandle) {
    emit_node_state(&app, NodeLifecycleState::Initialising);
    tokio::time::sleep(Duration::from_millis(500)).await;
    emit_node_state(&app, NodeLifecycleState::Ready);
}

// ── Service mode ───────────────────────────────────────────────────────────────

fn run_service_mode() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        emit_state_service(&NodeLifecycleState::Initialising);
        tokio::time::sleep(Duration::from_millis(200)).await;
        emit_state_service(&NodeLifecycleState::Ready);
        tracing::info!("xgen-node running in service mode — waiting for Ctrl+C");
        tokio::signal::ctrl_c().await.ok();
        emit_state_service(&NodeLifecycleState::Closing);
    });
}

// ── Entry point ────────────────────────────────────────────────────────────────

fn main() {
    let flags = parse_flags();
    let data_dir = resolve_data_dir(&flags.instance_label);
    std::fs::create_dir_all(&data_dir).expect("Failed to create instance data directory");

    maybe_write_default_config(&data_dir, flags.port_override.unwrap_or(8080));

    init_logging(&data_dir);

    let started_at = chrono::Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let session_id = format!("{:08x}", rand::random::<u32>());
    write_session_header(
        "node",
        None,
        None,
        None,
        "0.1",
        "0.10.3",
        &session_id,
        &started_at,
    );

    if flags.service_mode {
        tracing::info!("starting in service mode (--service)");
        run_service_mode();
        write_session_footer(ExitReason::Shutdown);
        return;
    }

    // ── Tauri mode ─────────────────────────────────────────────────────────────

    let initial_state = NodeAppState {
        primary: NodeLifecycleState::Initialising,
        degraded: HashSet::new(),
    };
    let initial_event = make_node_state_event(
        NodeLifecycleState::Initialising,
        &HashSet::new(),
    );
    let shared = CurrentNodeState(Arc::new(Mutex::new((initial_state, initial_event))));

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .manage(shared)
        .setup(|app| {
            // ── Systray ────────────────────────────────────────────────────────
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;

            let show_item = MenuItem::with_id(app, "show", "Open Admin Panel", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Shut Down", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let tray_icon = app.default_window_icon()
                .expect("no window icon configured")
                .clone();

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("XGen Node")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
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
                    }
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
            tauri::async_runtime::spawn(async move {
                run_startup(handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_node_state, shut_down])
        .run(tauri::generate_context!())
        .expect("error while running xgen-node-app");

    write_session_footer(ExitReason::Shutdown);
}
