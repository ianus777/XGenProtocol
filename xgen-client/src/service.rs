// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Headless resident mode for `xgen-client` — the `--service` flag's
//! implementation (M1 Phase 4 finalize; C3 in the verification matrix).
//!
//! Composition:
//!   - own tracing subscriber + per-run log file (no Tauri)
//!   - PID file write (so `--pid` finds this resident)
//!   - named-pipe server (same `batch::start_pipe_server` the desktop uses,
//!     so `--ping` / `--health` / `--stop` / `--reload-config` work the same)
//!   - best-effort sustained WS connection to the home Node:
//!       * connect once with a 10-second timeout
//!       * authenticate as the local Identity
//!       * drain inbound on `conn.recv()` (events ignored at this layer for
//!         M1 MVP — real ingest wiring is M3 work)
//!       * if connect / auth fails, log and keep the pipe server alive so
//!         operators can still query and stop the process cleanly
//!       * if the WS drops mid-flight, log + stop draining (do not reconnect
//!         in M1; reconnect-on-drop is M3)
//!
//! Shutdown: `__STOP__` over the pipe calls `std::process::exit(0)` inside
//! the pipe handler; Ctrl+C from the controlling terminal does the same.
//! Either path is brutal-but-effective for M1; clean cancellation tokens
//! are post-M1 polish.

use std::path::PathBuf;

use anyhow::Result;
use tokio::sync::watch;

use xgen_common::{
    build_info,
    event_trace::{write_session_footer, write_session_header, ExitReason},
};

use crate::app;
use crate::lifecycle::ClientLifecycleState;

// ── Logging init ───────────────────────────────────────────────────────────────

fn init_logging(data_dir: &std::path::Path, log_level_override: Option<&str>) {
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

// ── WS connection task ────────────────────────────────────────────────────────

/// Connect to the home Node, authenticate, and drain inbound events until
/// either the WS drops or the process is told to exit. Returns Ok on a clean
/// loop exit, Err if any setup step (config load, keypair load, connect, auth)
/// fails. The pipe server keeps running regardless — operators always have a
/// channel to query and stop the process.
async fn run_ws_loop(data_dir: PathBuf, mut shutdown_rx: watch::Receiver<bool>) -> Result<()> {
    let config_path = data_dir.join("xgen-client_config.toml");
    let node = app::resolve_node(None, &config_path);
    let keypair_path = app::resolve_keypair_path(&config_path);
    let signing_key = app::load_keypair(&keypair_path)?;

    tracing::info!(home_node = %node, "service: connecting to home Node");

    // M-RP6.6 (D1) — drive the SHARED resident spine. The service is headless,
    // so its lifecycle sink logs rather than emitting a Tauri event; the desktop
    // shell passes an `emit_state` sink to the SAME `run_session`. Single-session
    // here (the service stays connect-once, matching pre-M-RP6.6 M1 behaviour —
    // reconnect is the desktop resident's Leg B); `__STOP__` over the pipe still
    // exits the process directly, so `shutdown_rx` is effectively inert in this
    // path but is threaded for spine-signature uniformity.
    let mut sink = |state: ClientLifecycleState| {
        tracing::info!(lifecycle_state = state.as_canonical(), "service: lifecycle");
    };
    // Headless: the counters are maintained but not surfaced (no `get_conn_stats`
    // command in service mode). Single-session (reconnect is the desktop resident).
    let traffic = crate::resident::TrafficCounters::default();
    let outcome =
        crate::resident::run_session(&node, &signing_key, &traffic, &mut shutdown_rx, &mut sink)
            .await;
    tracing::info!(?outcome, "service: WS session ended");
    Ok(())
}

// ── Entry point ────────────────────────────────────────────────────────────────

/// Headless resident entry. Owns the tokio runtime; runs until Ctrl+C, an
/// `__STOP__` over the pipe (which calls `std::process::exit` internally), or
/// process termination.
pub fn run(
    data_dir: PathBuf,
    instance_label: Option<String>,
    log_level_override: Option<String>,
) {
    std::fs::create_dir_all(&data_dir).expect("failed to create data directory");

    init_logging(&data_dir, log_level_override.as_deref());

    let started_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let session_id = format!("{:08x}", rand::random::<u32>());
    write_session_header(
        "client",
        None,
        None,
        None,
        "0.1",
        build_info::VERSION,
        &session_id,
        &started_at,
    );

    app::write_pid_file(&data_dir);

    let pipe_name_str = crate::batch::pipe_name(instance_label.as_deref());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime for --service mode");

    rt.block_on(async move {
        // M-RP6.6 (D1) — ONE watch channel for the whole resident block. The pipe
        // servers (Windows) and the WS resident both take a receiver; the sender
        // is held until block_on ends — dropping it early makes every receiver's
        // `.changed()` return Err immediately (the pre-M-RP6.6 `_pipe_shutdown_hold`
        // lesson). `__STOP__` inside the pipe handler still calls `process::exit`
        // directly, so this channel is the clean-shutdown convenience, not the
        // sole stop path.
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        #[cfg(target_os = "windows")]
        {
            let pipe_data_dir = data_dir.clone();
            let pipe_name = pipe_name_str.clone();
            // --batch server (unchanged — D-066).
            let batch_rx = shutdown_rx.clone();
            tokio::spawn(async move {
                crate::batch::start_pipe_server(pipe_name, pipe_data_dir, batch_rx).await;
            });
            // --aicontrol sister server (M7 C2): independent accept loop on the
            // `.aicontrol` pipe, its own shutdown receiver + state-file lock.
            let ai_dir = data_dir.clone();
            let ai_pipe = crate::aicontrol::aicontrol_pipe_name(&pipe_name_str);
            let ai_rx = shutdown_rx.clone();
            let state_lock: crate::aicontrol::StateFileLock =
                std::sync::Arc::new(tokio::sync::Mutex::new(()));
            // M7-events C5: the client `.events` observer pipe.
            let events_dir = data_dir.clone();
            let events_pipe = crate::events_pipe::events_pipe_name(&pipe_name_str);
            let events_rx = shutdown_rx.clone();
            tokio::spawn(async move {
                // expected_token = None: AC-D4 gate inert in v1 (M7C-D1; the
                // privilege-model arc supplies a real source).
                crate::aicontrol::start_aicontrol_server(ai_pipe, ai_dir, ai_rx, state_lock, None).await;
            });
            tokio::spawn(async move {
                crate::events_pipe::start_events_server(events_pipe, events_dir, events_rx).await;
            });
        }
        #[cfg(not(target_os = "windows"))]
        {
            // Pipe server is Windows-only (D-043). On other platforms the
            // service mode runs without pipe control — only Ctrl+C can stop
            // it. M1 ships Windows-first; cross-platform pipe is post-M1.
            let _ = pipe_name_str;
            tracing::warn!(
                "service: named pipe server is Windows-only; control flags (--ping/--health/--stop) will fail on this platform"
            );
        }

        // Held until block_on ends so every receiver above (and the WS resident
        // below) stays live.
        let _pipe_shutdown_hold = shutdown_tx;

        // WS resident task — best-effort. If setup fails, log and keep the pipe
        // server up so operators can query and stop the process.
        let ws_data_dir = data_dir.clone();
        let ws_rx = shutdown_rx.clone();
        let ws_task = tokio::spawn(async move {
            if let Err(e) = run_ws_loop(ws_data_dir, ws_rx).await {
                tracing::warn!(reason = %format!("{:#}", e), "service: WS task ended");
            }
        });

        // Block waiting for Ctrl+C. The pipe server's `__STOP__` handler
        // calls `std::process::exit(0)` directly and bypasses this select.
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("service: Ctrl+C received — exiting");
            }
            _ = ws_task => {
                // The WS task ended (either cleanly or with an error already
                // logged inside). Keep the process alive on the pipe server
                // until Ctrl+C — operators can still query and stop us.
                tracing::info!("service: WS task ended; pipe server still active — waiting for Ctrl+C or pipe __STOP__");
                let _ = tokio::signal::ctrl_c().await;
                tracing::info!("service: Ctrl+C received — exiting");
            }
        }
    });

    write_session_footer(ExitReason::Shutdown);
}
