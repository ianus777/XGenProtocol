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

use crate::app;
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

/// Resolved path to `xgen-client_config.toml` for this instance (M-RP4.2).
/// Held as managed state so `get_substitutions` reads the live config without
/// re-deriving the data-dir — which depends on the `--instance` label resolved
/// at launch. Mirrors the `CurrentState`/`Pacing` managed-state pattern.
struct ConfigPath(PathBuf);

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

/// Returns the raw `[substitutions] rules` string from xgen-client_config.toml
/// (M-RP4.2). Called by the Svelte shell on boot; the `$common` processor store
/// parses it (split on ` | `, first space per pair → find | replace) and feeds
/// every processor-host. Empty string when the section or the file is absent.
#[tauri::command]
fn get_substitutions(config: tauri::State<ConfigPath>) -> String {
    app::load_substitutions_section(&config.0).rules
}

/// Writes the raw `[substitutions] rules` string back to xgen-client_config.toml
/// (M-RP4.3 — the effect half of the `substitutions-editor` widget). Symmetric with
/// `get_substitutions`; the widget's host-injected `onApply` callback invokes this
/// on Apply. Returns `Err(String)` (surfaced to the webview) on a read/parse/write
/// failure rather than clobbering the config (D-065). Session-only under D-101
/// (clean-slate-on-start re-seeds every launch) — surfaced in the editor UI (W-8).
#[tauri::command]
fn set_substitutions(rules: String, config: tauri::State<ConfigPath>) -> std::result::Result<(), String> {
    app::write_substitutions_section(&config.0, &rules).map_err(|e| e.to_string())
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
        // --aicontrol sister server (M7 C2) — second independent spawn.
        let ai_dir = data_dir.clone();
        let ai_pipe = crate::aicontrol::aicontrol_pipe_name(&pipe_name);
        let ai_rx = shutdown_rx.clone();
        let state_lock: crate::aicontrol::StateFileLock =
            std::sync::Arc::new(tokio::sync::Mutex::new(()));
        tauri::async_runtime::spawn(async move {
            // expected_token = None: AC-D4 gate inert in v1 (M7C-D1).
            crate::aicontrol::start_aicontrol_server(ai_pipe, ai_dir, ai_rx, state_lock, None).await;
        });
        // M7-events C5: the client `.events` observer pipe — second WS to the
        // home Node, filter-at-drain.
        let events_dir = data_dir.clone();
        let events_pipe = crate::events_pipe::events_pipe_name(&pipe_name);
        let events_rx = shutdown_rx.clone();
        tauri::async_runtime::spawn(async move {
            crate::events_pipe::start_events_server(events_pipe, events_dir, events_rx).await;
        });
    }

    let config_path = data_dir.join("xgen-client_config.toml");
    let keypair_path = data_dir.join("xgen-client_keypair.enc");

    // D-101 — clean-slate-on-start (phase-scoped). Config is ephemeral this
    // phase: if one exists, wipe it and regenerate from seed BEFORE the
    // first-run read below. This SUSPENDS J-438 seed-once — cleared substitution
    // pairs reappear on relaunch (intended now; no persistent settings surface
    // exists yet). Retired when the client/node UIs gain persistent settings.
    // See DECISIONS.md D-101 for the full why + exit condition.
    app::clean_slate_config(&config_path, &keypair_path);

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
    // D-068 — flag > env (XGEN_LOG) > config (`[logging].level`) > "debug".
    // Pre-J-079 this path fell back to `EnvFilter::new("debug")`, silently
    // ignoring config. The convergence ships in J-079 commit 3.
    let config_path = data_dir.join("xgen-client_config.toml");
    let config_level = app::read_config_log_level(&config_path);
    let env_filter = EnvFilter::new(xgen_common::precedence::resolve_log_level(
        log_level_override.as_deref(),
        config_level.as_deref(),
    ));
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

    // M-RP4.2 — the config path for `get_substitutions`, derived from the same
    // data_dir the startup sequence uses (`run_startup` line ~139).
    let config_path = ConfigPath(data_dir.join("xgen-client_config.toml"));

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .manage(shared_state)
        .manage(PipeShutdown(shutdown_tx))
        .manage(pacing_manager)
        .manage(config_path)
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
        .invoke_handler(tauri::generate_handler![
            get_state,
            get_pacing_state,
            quit,
            get_substitutions,
            set_substitutions
        ])
        .run(tauri::generate_context!())
        .expect("error while running xgen-client desktop shell");

    write_session_footer(ExitReason::Shutdown);
}

#[cfg(test)]
mod pass_4_commit_1_tests {
    //! XGID Retrofit Pass 4 Commit 1 — Surface #4 (Tauri Shell) per-surface
    //! tests T8 + T9 (runbook §6.3, design doc §4.6.b + §4.2 Instance C).
    use super::*;
    use xgen_common::xgid::{IdentityXgid, SpaceXgid, Xgid};

    /// T8 — a Tauri command return (`get_pacing_state` → `Vec<PacingState>`)
    /// crosses the IPC boundary to the JS frontend via serde-transparent
    /// newtypes (§4.2 Instance C): the JS-visible JSON carries identifier
    /// slots as plain strings, not nested objects.
    #[test]
    fn tauri_command_return_serde_transparent_to_js_frontend() {
        let snap = PacingState {
            space_id: SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:S".to_string())),
            sender_identity_id: IdentityXgid::from_xgid(Xgid::new(
                "xgen://pubkey/ed25519:M".to_string(),
            )),
            cap_ms: 2000,
            queue_count: 0,
            time_to_next_send_ms: 0,
            drain_ms: 0,
            is_ai: true,
        };
        let json = serde_json::to_string(&snap).unwrap();
        assert!(json.contains(r#""space_id":"xgen://hash/sha256:S""#), "got {json}");
        assert!(
            json.contains(r#""sender_identity_id":"xgen://pubkey/ed25519:M""#),
            "got {json}"
        );
    }

    /// T9 — `ClientStateEvent` carries no identifier slots (design doc §4.6.b
    /// drift correction): `state` is an enum, `label`/`timestamp` stay
    /// `String`. `make_state_event` produces descriptive Strings.
    #[test]
    fn lifecycle_state_event_descriptive_slots_stay_string() {
        let ev = make_state_event(ClientLifecycleState::Ready);
        let _label: &String = &ev.label; // descriptive, stays String
        let _timestamp: &String = &ev.timestamp; // descriptive, stays String
        assert_eq!(ev.label, "Ready");
        assert!(matches!(ev.state, ClientLifecycleState::Ready));
    }
}
