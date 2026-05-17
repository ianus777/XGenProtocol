// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Batch dispatch + named-pipe IPC (D-043). After Phase 3 wider (J-070), all
// command implementations live in `crate::app`; this module is just the pipe
// transport and the canonical `get_dag_tips`. Named-pipe pieces are
// Windows-only.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::Mutex;

use xgen_core::{
    transport::connection::Inbound,
    wire::types::TransportMessage,
};

// ── Resident health state (M4) ────────────────────────────────────────────────

/// Live state surfaced by the pipe server's `__HEALTH__` handler.
/// Populated by the resident's main loop (AI or human) and read by the
/// pipe-server's request handler. Default values are the human-Client
/// shape — the AI service overrides on startup.
#[derive(Debug, Clone)]
pub struct ResidentHealthState {
    /// Short label for the resident mode — `"human"` or `"ai"` — appended
    /// to the `__HEALTH__` reply as `mode=...`.
    pub mode_label: String,
    /// AI-only field: `Some((known, total))` where `known` is the number of
    /// Spaces the AI resolves an operator for and `total` is the count of
    /// Spaces the AI is a member of. `None` for human-mode residents (the
    /// field is omitted from the `__HEALTH__` reply in that case).
    pub operator_known: Option<(usize, usize)>,
}

impl ResidentHealthState {
    /// Default state for a human-Client resident — no AI-specific fields.
    pub fn human_default() -> Self {
        Self {
            mode_label: "human".to_string(),
            operator_known: None,
        }
    }

    /// Default state for an AI-Client resident at startup — before any
    /// Space events have been observed. Shows `operator_known=0/0` rather
    /// than absent so the field is consistently present in AI-mode logs.
    pub fn ai_default() -> Self {
        Self {
            mode_label: "ai".to_string(),
            operator_known: Some((0, 0)),
        }
    }
}

// ── Pipe name — D-043 ──────────────────────────────────────────────────────────

/// Returns the named pipe path for a given instance label (D-043).
/// `None` → `\\.\pipe\xgen-client`
/// `Some("alice")` → `\\.\pipe\xgen-client-alice`
pub fn pipe_name(instance_label: Option<&str>) -> String {
    match instance_label {
        Some(label) => format!(r"\\.\pipe\xgen-client-{}", label),
        None => r"\\.\pipe\xgen-client".to_string(),
    }
}

/// Request DAG tips for a Space via `transport.sync_request` and collect the
/// most recent tip event_id, Space-filtered (closes F-003/F-004 from J-067).
/// The canonical implementation for the entire Client crate.
pub async fn get_dag_tips(
    conn: &mut xgen_core::transport::connection::Connection<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    space_id: &str,
) -> Result<Vec<String>> {
    let req = TransportMessage::SyncRequest {
        protocol_version: "0.1".to_string(),
        since: String::new(),
    };
    conn.send_transport(&req).await?;
    let mut tips: Vec<String> = vec![];
    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_millis(500);
    loop {
        match tokio::time::timeout_at(deadline, conn.recv()).await {
            Ok(Ok(Inbound::Event(ev))) => {
                // Filter to events belonging to the target Space. The Node
                // returns events from every Space the requester is a member of,
                // so cross-Space leaks would corrupt prev_events. Events with
                // empty space_id (state.space_create / state.dm_space_create)
                // identify themselves via event_id == space_id.
                let ev_space: &str = if ev.space_id.is_empty() {
                    ev.event_id.as_deref().unwrap_or("")
                } else {
                    ev.space_id.as_str()
                };
                if ev_space == space_id {
                    if let Some(id) = ev.event_id {
                        tips = vec![id];
                    }
                }
            }
            _ => break,
        }
    }
    Ok(tips)
}

// ── Dispatch ───────────────────────────────────────────────────────────────────

/// Tokenize and dispatch one batch command line.
///
/// Uses shlex for quoting/escaping. Parses against the canonical
/// `crate::app::Cli` (same parser as direct CLI / `--batch` in-process /
/// pipe-server). Dispatches to `crate::app::cmd_*` — there is no parallel
/// command implementation in batch.rs (Phase 3 wider dedup).
///
/// Commands not appropriate for batch dispatch (the long-running smoke /
/// stress tests, `--service`-only modes) are rejected with a clear error
/// rather than silently misbehaving.
pub async fn dispatch_line(line: &str, data_dir: &Path) -> Result<()> {
    use crate::app::{self, Cli, ClientCommand};

    let tokens = shlex::split(line).unwrap_or_else(|| vec![line.to_string()]);
    let mut argv = vec!["xgen-client".to_string()];
    argv.extend(tokens);

    let cli = <Cli as clap::Parser>::try_parse_from(&argv)
        .map_err(|e| anyhow::anyhow!("unrecognised command: {}", e))?;

    let node_override = cli.node.as_deref();
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| data_dir.join("xgen-client_config.toml"));
    let node = app::resolve_node(node_override, &config_path);
    let keypair_path = app::resolve_keypair_path(&config_path);

    match cli.command {
        None => Ok(()),
        Some(ClientCommand::Init(args)) => app::cmd_init(&args, data_dir),
        Some(ClientCommand::Whoami) => {
            // M5 commit 1: pipe arm calls ops::whoami directly. The pipe
            // protocol (D-066-frozen) only needs OK/ERROR — the result data
            // is discarded here. A future --aicontrol surface (M7) will
            // serialise the same WhoamiResult as JSONL.
            let mut session =
                crate::session::SessionState::new(String::new(), data_dir.to_path_buf());
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::whoami(&mut ctx).map(|_| ())
        }
        Some(ClientCommand::Status) => {
            // M5 commit 2: pipe arm calls ops::status directly.
            let mut session =
                crate::session::SessionState::new(String::new(), data_dir.to_path_buf());
            let mut ctx = crate::ops::OpContext {
                session: &mut session,
                data_dir,
                node_override: None,
            };
            crate::ops::status(&mut ctx).map(|_| ())
        }
        Some(ClientCommand::Spaces) => app::cmd_spaces(data_dir),
        Some(ClientCommand::Version) => app::cmd_version(),
        Some(ClientCommand::Register(args)) => {
            let ai = app::load_ai_section(&config_path);
            app::cmd_register(&args, &node, &keypair_path, data_dir, ai.as_ref()).await
        }
        Some(ClientCommand::CreateSpace(args)) => {
            app::cmd_create_space(&args, &node, &keypair_path, data_dir).await
        }
        Some(ClientCommand::CreateRoom(args)) => {
            app::cmd_create_room(&args, &node, &keypair_path, data_dir).await
        }
        Some(ClientCommand::Invite(args)) => app::cmd_invite(&args, &node, &keypair_path).await,
        Some(ClientCommand::Join(args)) => app::cmd_join(&args, &node, &keypair_path).await,
        Some(ClientCommand::Send(args)) => app::cmd_send(&args, &node, &keypair_path).await,
        Some(ClientCommand::History(args)) => {
            app::cmd_history(&args, &node, &keypair_path).await
        }
        Some(ClientCommand::Ai(args)) => match args.command {
            crate::app::AiCommand::Delegate(a) => {
                app::cmd_ai_delegate(&a, &node, &keypair_path).await
            }
            crate::app::AiCommand::Revoke(a) => {
                app::cmd_ai_revoke(&a, &node, &keypair_path).await
            }
            crate::app::AiCommand::Status(a) => {
                app::cmd_ai_status(&a, &node, &keypair_path).await
            }
        },
        Some(ClientCommand::SmokeTest(_))
        | Some(ClientCommand::StressTest(_))
        | Some(ClientCommand::SmokePh2(_))
        | Some(ClientCommand::StressComplete(_)) => {
            anyhow::bail!(
                "long-running test commands cannot be dispatched through --batch / pipe"
            )
        }
    }
}

// ── Named pipe server — M1 (Windows only) ──────────────────────────────────────

/// Backward-compatible entry point — starts the pipe server with a
/// human-Client default `ResidentHealthState`. Existing callers (service::run,
/// desktop::run) keep working unchanged. The M4 AI service uses
/// `start_pipe_server_with_health` to thread its own state in.
#[cfg(target_os = "windows")]
pub async fn start_pipe_server(
    pipe_name_str: String,
    data_dir: PathBuf,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    let health = Arc::new(Mutex::new(ResidentHealthState::human_default()));
    start_pipe_server_with_health(pipe_name_str, data_dir, shutdown_rx, health).await;
}

/// Start the named pipe server with a shared health-state handle.
/// Accepts one connection at a time; reads commands until `__END__`;
/// dispatches each; writes `OK\n` or `ERROR: …\n`; loops.
/// Stops cleanly when `shutdown_rx` delivers `true`.
///
/// The `__HEALTH__` reply incorporates the `ResidentHealthState` so AI-mode
/// residents (M4) report `mode=ai operator_known=N/M` while human-mode
/// residents report `mode=human` (default).
#[cfg(target_os = "windows")]
pub async fn start_pipe_server_with_health(
    pipe_name_str: String,
    data_dir: PathBuf,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    health_state: Arc<Mutex<ResidentHealthState>>,
) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::windows::named_pipe::ServerOptions;

    tracing::info!(pipe = %pipe_name_str, "Pipe server starting");

    let mut first = true;

    loop {
        if *shutdown_rx.borrow() {
            break;
        }

        let server = {
            let create_result = if first {
                first = false;
                ServerOptions::new().first_pipe_instance(true).create(&pipe_name_str)
            } else {
                ServerOptions::new().create(&pipe_name_str)
            };
            match create_result {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(error = %e, "Named pipe create failed — pipe server stopping");
                    break;
                }
            }
        };

        // Wait for a client connection or shutdown signal
        let connected = tokio::select! {
            r = server.connect() => r.is_ok(),
            _ = shutdown_rx.changed() => false,
        };

        if !connected || *shutdown_rx.borrow() {
            break;
        }

        // server is now a connected pipe — split for independent read/write
        let (reader_half, mut writer_half) = tokio::io::split(server);
        let mut reader = BufReader::new(reader_half);
        let mut lines: Vec<String> = Vec::new();
        let mut buf = String::new();

        // Read the first line — may be a control command (handled inline) or
        // the first batch line (collected then drained until __END__).
        let first_line: Option<String> = match reader.read_line(&mut buf).await {
            Ok(0) => None,
            Ok(_) => Some(
                buf.trim_end_matches('\n')
                    .trim_end_matches('\r')
                    .to_string(),
            ),
            Err(e) => {
                tracing::warn!(error = %e, "Pipe read error (first line)");
                None
            }
        };

        // Control commands (Phase 4): single line, single response, no __END__.
        // Recognised tokens: __PING__, __HEALTH__, __STOP__, __RELOAD_CONFIG__.
        if let Some(ref line) = first_line {
            match line.as_str() {
                "__PING__" => {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    let resp = format!("PONG {}\n", now_ms);
                    let _ = writer_half.write_all(resp.as_bytes()).await;
                    let _ = writer_half.flush().await;
                    continue;
                }
                "__HEALTH__" => {
                    // One-line liveness summary. PID + mode label + AI-mode
                    // operator-known count (M4 §7). The structured per-Space
                    // operator map stays on `xgen-client status`.
                    let snapshot = health_state.lock().await.clone();
                    let mut resp = format!(
                        "HEALTHY pid={} mode={}",
                        std::process::id(),
                        snapshot.mode_label
                    );
                    if let Some((known, total)) = snapshot.operator_known {
                        resp.push_str(&format!(" operator_known={}/{}", known, total));
                    }
                    resp.push('\n');
                    let _ = writer_half.write_all(resp.as_bytes()).await;
                    let _ = writer_half.flush().await;
                    continue;
                }
                "__STOP__" => {
                    let _ = writer_half.write_all(b"OK STOPPING\n").await;
                    let _ = writer_half.flush().await;
                    tracing::info!("__STOP__ received over pipe — exiting process");
                    // Brutal exit so the Tauri main loop and any background
                    // tasks go down with us. The merged binary owns the
                    // entire process; clean Tauri shutdown coordination is
                    // post-M1 polish.
                    std::process::exit(0);
                }
                "__RELOAD_CONFIG__" => {
                    let _ = writer_half
                        .write_all(b"NOT_IMPLEMENTED: config reload arrives in a later milestone\n")
                        .await;
                    let _ = writer_half.flush().await;
                    continue;
                }
                _ => { /* fall through to batch path */ }
            }
        }

        // Batch path: not a control command. Collect lines until __END__.
        if let Some(line) = first_line {
            if line != "__END__" {
                lines.push(line);
            }
        }
        loop {
            buf.clear();
            match reader.read_line(&mut buf).await {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = buf
                        .trim_end_matches('\n')
                        .trim_end_matches('\r')
                        .to_string();
                    if trimmed == "__END__" {
                        break;
                    }
                    lines.push(trimmed);
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Pipe read error");
                    break;
                }
            }
        }

        tracing::info!(count = lines.len(), "Batch execution started");

        let mut exec_error: Option<String> = None;
        for line in &lines {
            match dispatch_line(line, &data_dir).await {
                Ok(()) => {}
                Err(e) => {
                    exec_error = Some(format!("{:#}", e));
                    break;
                }
            }
        }

        match exec_error {
            None => {
                tracing::info!("Batch execution completed — OK");
                let _ = writer_half.write_all(b"OK\n").await;
            }
            Some(msg) => {
                tracing::warn!(error = %msg, "Batch execution stopped — ERROR");
                let response = format!("ERROR: {}\n", msg);
                let _ = writer_half.write_all(response.as_bytes()).await;
            }
        }
        let _ = writer_half.flush().await;
        // writer_half and reader_half dropped here → pipe connection closed
    }

    tracing::info!(pipe = %pipe_name_str, "Pipe server stopped");
}

// ── Batch invocation path — M2 (Windows only) ──────────────────────────────────

/// Second-process batch invocation path.
/// Validates the .xgb file, connects to the running instance's pipe,
/// streams commands + __END__, reads result, returns exit code.
/// Creates its own tokio runtime — must NOT be called from within async context.
#[cfg(target_os = "windows")]
pub fn run_batch_client(raw_path: &str, pipe_name_str: &str, instance_label: Option<&str>) -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to create async runtime: {}", e);
            return 2;
        }
    };
    rt.block_on(run_batch_client_async(raw_path, pipe_name_str, instance_label))
}

#[cfg(target_os = "windows")]
async fn run_batch_client_async(raw_path: &str, pipe_name_str: &str, instance_label: Option<&str>) -> i32 {
    use std::io::BufRead as _;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;

    // M2 §2.2 — canonicalize and extension check
    let canonical = match std::fs::canonicalize(raw_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: cannot resolve batch file path {:?}: {}", raw_path, e);
            return 2;
        }
    };

    let ext = canonical
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if !ext.eq_ignore_ascii_case("xgb") {
        eprintln!(
            "error: batch file must have .xgb extension, got {:?}",
            canonical
        );
        return 2;
    }

    // M2 §2.3 — read non-empty, non-comment lines
    let file = match std::fs::File::open(&canonical) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: cannot open {:?}: {}", canonical, e);
            return 2;
        }
    };
    let commands: Vec<String> = std::io::BufReader::new(file)
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    // M2 §2.4 — connect to the running instance
    let mut client = match ClientOptions::new().open(pipe_name_str) {
        Ok(c) => c,
        Err(_) => {
            let start_hint = match instance_label {
                Some(l) => format!("xgen-client.exe --instance {} before running --batch.", l),
                None    => "xgen-client.exe before running --batch.".to_string(),
            };
            eprintln!(
                "error: no running xgen-client instance found at {}\n       Start {}",
                pipe_name_str, start_hint
            );
            return 3;
        }
    };

    // M2 §2.5 — write commands + __END__ sentinel
    for cmd in &commands {
        let line = format!("{}\n", cmd);
        if let Err(e) = client.write_all(line.as_bytes()).await {
            eprintln!("error: failed to write to pipe: {}", e);
            return 1;
        }
    }
    if let Err(e) = client.write_all(b"__END__\n").await {
        eprintln!("error: failed to write sentinel: {}", e);
        return 1;
    }
    if let Err(e) = client.flush().await {
        eprintln!("error: failed to flush: {}", e);
        return 1;
    }

    // Read response
    let mut response = String::new();
    if let Err(e) = client.read_to_string(&mut response).await {
        eprintln!("error: failed to read response: {}", e);
        return 1;
    }

    let response = response.trim();
    if response == "OK" {
        0
    } else if let Some(msg) = response.strip_prefix("ERROR: ") {
        eprintln!("{}", msg);
        1
    } else {
        eprintln!("error: unexpected pipe response: {}", response);
        1
    }
}

// ── Control-command client helpers — Phase 4 (Windows only) ────────────────────

/// Open the pipe, send a single control line, read the single-line response.
/// Returns the trimmed response on success, or an Err containing a
/// human-readable diagnostic on failure.
#[cfg(target_os = "windows")]
async fn pipe_send_control(pipe_name_str: &str, control_token: &str) -> Result<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;

    let mut client = ClientOptions::new()
        .open(pipe_name_str)
        .with_context(|| format!("no resident found at {}", pipe_name_str))?;

    let line = format!("{}\n", control_token);
    client
        .write_all(line.as_bytes())
        .await
        .context("failed to write control command")?;
    client.flush().await.context("failed to flush pipe")?;

    let mut response = String::new();
    client
        .read_to_string(&mut response)
        .await
        .context("failed to read pipe response")?;

    Ok(response.trim().to_string())
}

/// `--ping`: round-trip a __PING__ command and print the latency in ms.
/// Exits 0 on PONG response; non-zero otherwise.
#[cfg(target_os = "windows")]
pub fn cmd_ping(pipe_name_str: &str) -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to create async runtime: {}", e);
            return 2;
        }
    };
    let start = std::time::Instant::now();
    let result = rt.block_on(pipe_send_control(pipe_name_str, "__PING__"));
    let elapsed_ms = start.elapsed().as_millis();
    match result {
        Ok(response) => {
            if response.starts_with("PONG ") {
                println!("pong: {} ms", elapsed_ms);
                0
            } else {
                eprintln!("error: unexpected ping response: {}", response);
                1
            }
        }
        Err(e) => {
            eprintln!("error: {:#}", e);
            1
        }
    }
}

/// `--health`: ask the running resident for its one-line liveness summary.
/// Exits 0 if response starts with HEALTHY, non-zero otherwise.
#[cfg(target_os = "windows")]
pub fn cmd_health(pipe_name_str: &str) -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to create async runtime: {}", e);
            return 2;
        }
    };
    match rt.block_on(pipe_send_control(pipe_name_str, "__HEALTH__")) {
        Ok(response) => {
            println!("{}", response);
            if response.starts_with("HEALTHY") {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("error: {:#}", e);
            1
        }
    }
}

/// `--stop`: signal the running resident to exit gracefully (the resident
/// process terminates itself after responding). Returns 0 on OK STOPPING.
#[cfg(target_os = "windows")]
pub fn cmd_stop(pipe_name_str: &str) -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to create async runtime: {}", e);
            return 2;
        }
    };
    match rt.block_on(pipe_send_control(pipe_name_str, "__STOP__")) {
        Ok(response) => {
            println!("{}", response);
            if response.starts_with("OK") {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("error: {:#}", e);
            1
        }
    }
}

/// `--reload-config`: signal the running resident to reload its config.
/// The resident currently replies NOT_IMPLEMENTED (config-reload semantics
/// land in a later milestone); this command surfaces that honestly.
#[cfg(target_os = "windows")]
pub fn cmd_reload_config(pipe_name_str: &str) -> i32 {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: failed to create async runtime: {}", e);
            return 2;
        }
    };
    match rt.block_on(pipe_send_control(pipe_name_str, "__RELOAD_CONFIG__")) {
        Ok(response) => {
            println!("{}", response);
            if response.starts_with("OK") {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("error: {:#}", e);
            1
        }
    }
}
