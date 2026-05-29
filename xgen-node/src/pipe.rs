// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Named-pipe IPC for `xgen-node` (M2). Ports the Client's `batch::start_pipe_server`
//! and control-command client helpers (`cmd_ping`, `cmd_health`, `cmd_stop`,
//! `cmd_reload_config`, `cmd_batch`) onto the Node side.
//!
//! D-043 pipe-name convention: `\\.\pipe\xgen-node[-<instance>]`.
//! D-056 deployment model: every resident hosts a pipe server.
//!
//! Per Joe's M2 dispositions:
//!   - `__BATCH__` accepts only the Node's read-only subcommand subset
//!     (status, connections, peers, spaces, identity list, version, whoami).
//!   - `__HEALTH__` returns a rich one-line summary with pid / conns / peers /
//!     spaces / uptime / state.
//!   - `__STOP__` calls `std::process::exit(0)` (same as Client).
//!   - `__RELOAD_CONFIG__` returns a Node-specific `NOT_IMPLEMENTED` line
//!     explaining the WS-listener-rebind constraint.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;

use xgen_core::federation::registry::FederationRegistry;
use xgen_core::node::runtime::NodeRuntime;

use crate::admin_ops;
use crate::app;

// ── Pipe name — D-043 ──────────────────────────────────────────────────────────

/// Returns the named pipe path for a given instance label (D-043).
/// `None` → `\\.\pipe\xgen-node`
/// `Some("n1")` → `\\.\pipe\xgen-node-n1`
pub fn pipe_name(instance_label: Option<&str>) -> String {
    match instance_label {
        Some(label) => format!(r"\\.\pipe\xgen-node-{}", label),
        None => r"\\.\pipe\xgen-node".to_string(),
    }
}

// ── Dispatch ───────────────────────────────────────────────────────────────────

/// Tokenize and dispatch one Node batch command line.
///
/// Allowed (read-only): `status`, `connections`, `peers`, `spaces`,
/// `identity list`, `version`, `whoami`. Anything else is rejected explicitly —
/// the Node has no mutating subcommands today, and pipe-batch is intentionally
/// restricted to the safe set per the M2 task file.
pub async fn dispatch_line(
    line: &str,
    data_dir: &Path,
    config_path: &Path,
    runtime: Option<&Arc<tokio::sync::Mutex<NodeRuntime>>>,
    federation_registry: Option<&Arc<tokio::sync::Mutex<FederationRegistry>>>,
) -> Result<()> {
    let tokens = shlex::split(line).unwrap_or_else(|| vec![line.to_string()]);

    // Read-only allowlist (M2) — preserved unchanged.
    match tokens.as_slice() {
        [] => return Ok(()),
        [cmd] if cmd == "status" => return app::cmd_status(data_dir),
        [cmd] if cmd == "connections" => return app::cmd_connections(data_dir),
        [cmd] if cmd == "peers" => return app::cmd_peers(data_dir),
        [cmd] if cmd == "spaces" => return app::cmd_spaces(data_dir),
        [cmd] if cmd == "version" => return app::cmd_version(config_path, data_dir),
        [cmd] if cmd == "whoami" => return app::cmd_whoami(config_path, data_dir),
        [a, b] if a == "identity" && b == "list" => return app::cmd_identity_list(data_dir),
        _ => {}
    }

    // M6 admin verbs (§6) — parse the two-token verb path via the shared clap
    // grouping and dispatch into `admin_ops::*` (D-067; the same layer M7's
    // `--aicontrol` will call). The read-only allowlist above is unchanged.
    // `runtime` is `Some` for the in-resident pipe server (A5 verbs need the
    // live NodeRuntime — P5 decision); `None` only in unit tests of the
    // file-only A6 verbs.
    match admin_ops::AdminCli::try_parse_from(tokens.iter().map(String::as_str)) {
        Ok(cli) => dispatch_admin(cli.command, data_dir, config_path, runtime, federation_registry).await,
        Err(_) => anyhow::bail!(
            "command not supported in pipe-batch mode (allowed reads: status, connections, peers, spaces, identity list, version, whoami; M6 admin verbs: audit query|export|archive, log set-level|show-level, identity show|revoke|set-trust-expiry|manage-replica): {}",
            line
        ),
    }
}

/// The administrator principal recorded as the audit `actor` (§2.6.1). v1:
/// OS-user-equals-administrator — the pipe inherits OS-level access control, so
/// the initiating OS user is the administrator. M7 may carry a distinct principal.
fn current_admin_actor() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .map(|u| format!("os-user:{u}"))
        .unwrap_or_else(|_| "os-user:unknown".to_string())
}

/// Execute one parsed admin command and render its reply for the `--batch` pipe
/// channel (§2.3 — the dispatcher formats; `admin_ops::*` emits nothing). On
/// success: print a human summary to stdout and return `Ok(())` → the pipe
/// server sends `OK`. On `AdminError`: return it as the reply body → the pipe
/// server's `ERROR: <body>` wrapper yields `ERROR: <CODE>: <message>`.
async fn dispatch_admin(
    cmd: admin_ops::AdminCommand,
    data_dir: &Path,
    config_path: &Path,
    runtime: Option<&Arc<tokio::sync::Mutex<NodeRuntime>>>,
    federation_registry: Option<&Arc<tokio::sync::Mutex<FederationRegistry>>>,
) -> Result<()> {
    use admin_ops::{AdminCommand, AuditCommand, FederationCommand, IdentityCommand, LogCommand};

    let actor = current_admin_actor();
    let mut ctx = admin_ops::AdminContext::batch(data_dir, config_path, actor);
    if let Some(rt) = runtime {
        ctx = ctx.with_runtime(Arc::clone(rt));
    }
    if let Some(fr) = federation_registry {
        ctx = ctx.with_federation_registry(Arc::clone(fr));
    }

    match cmd {
        AdminCommand::Audit(AuditCommand::Query(args)) => {
            match admin_ops::audit_query(&mut ctx, args).await {
                Ok(r) => {
                    // Direct-mode stdout view; the pipe channel returns OK only
                    // (rich structured output is M7's --aicontrol surface).
                    for e in &r.entries {
                        if let Ok(j) = serde_json::to_string(e) {
                            println!("{j}");
                        }
                    }
                    println!("audit query: {} matched, {} returned", r.total_matched, r.returned);
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
        AdminCommand::Audit(AuditCommand::Export(args)) => {
            match admin_ops::audit_export(&mut ctx, args).await {
                Ok(r) => {
                    println!("audit export: {} entries → {}", r.exported_count, r.output_path);
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
        AdminCommand::Audit(AuditCommand::Archive(args)) => {
            match admin_ops::audit_archive(&mut ctx, args).await {
                Ok(r) => {
                    println!("audit archive: {} archived → {}", r.archived_count, r.archive_path);
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
        AdminCommand::Log(LogCommand::ShowLevel(args)) => {
            match admin_ops::log_show_level(&mut ctx, args).await {
                Ok(r) => {
                    for e in &r.levels {
                        println!("{}: {}", e.module, e.level);
                    }
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
        AdminCommand::Log(LogCommand::SetLevel(args)) => {
            match admin_ops::log_set_level(&mut ctx, args).await {
                Ok(r) => {
                    println!(
                        "log set-level: {} {} → {} (applied={})",
                        r.module, r.previous_level, r.new_level, r.applied
                    );
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
        AdminCommand::Identity(IdentityCommand::Show(args)) => {
            match admin_ops::identity_show(&mut ctx, args).await {
                Ok(r) => {
                    if let Ok(j) = serde_json::to_string(&r.record) {
                        println!("{j}");
                    }
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
        AdminCommand::Identity(IdentityCommand::Revoke(args)) => {
            match admin_ops::identity_revoke(&mut ctx, args).await {
                Ok(r) => {
                    println!(
                        "identity revoke: {} revoked at {} ({} stale membership space(s))",
                        r.identity_id,
                        r.revoked_at,
                        r.stale_membership_spaces.len()
                    );
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
        AdminCommand::Identity(IdentityCommand::SetTrustExpiry(args)) => {
            match admin_ops::identity_set_trust_expiry(&mut ctx, args).await {
                Ok(r) => {
                    println!(
                        "identity set-trust-expiry: {} {} → {}",
                        r.identity_id,
                        r.previous_expiry.as_deref().unwrap_or("(none)"),
                        r.new_expiry
                    );
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
        AdminCommand::Identity(IdentityCommand::ManageReplica(args)) => {
            match admin_ops::identity_manage_replica(&mut ctx, args).await {
                Ok(r) => {
                    println!(
                        "identity manage-replica: {} → [{}]",
                        r.identity_id,
                        r.replicas.join(", ")
                    );
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
        AdminCommand::Federation(FederationCommand::List(args)) => {
            match admin_ops::federation_list(&mut ctx, args).await {
                Ok(r) => {
                    for rel in &r.relationships {
                        if let Ok(j) = serde_json::to_string(rel) {
                            println!("{j}");
                        }
                    }
                    println!(
                        "federation list: {} matched, {} returned{}",
                        r.total_matched,
                        r.returned,
                        r.next_cursor
                            .as_deref()
                            .map(|c| format!(" (next cursor: {c})"))
                            .unwrap_or_default()
                    );
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
        AdminCommand::Federation(FederationCommand::Defederate(args)) => {
            match admin_ops::federation_defederate(&mut ctx, args).await {
                Ok(r) => {
                    println!(
                        "federation defederate: {} at {} ({} shared space(s) cleaned)",
                        r.peer_node_id,
                        r.defederated_at,
                        r.cleaned_spaces.len()
                    );
                    Ok(())
                }
                Err(e) => anyhow::bail!("{}", e.code_message()),
            }
        }
    }
}

// ── Health snapshot ────────────────────────────────────────────────────────────

/// One-line health summary captured under brief locks (per Joe's M2 disposition:
/// "rich one-line"). Format:
///   `HEALTHY pid=<n> state=RUNNING conns=<n> peers=<n> spaces=<n> uptime=<n>s`
async fn build_health_line(
    runtime: &Arc<tokio::sync::Mutex<NodeRuntime>>,
    connections: &app::Connections,
    started_at_epoch: u64,
) -> String {
    let (spaces, peers) = {
        let rt = runtime.lock().await;
        (rt.spaces.len(), rt.peer_urls.len())
    };
    let conns = {
        let c = connections.lock().await;
        c.len()
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(started_at_epoch);
    let uptime = now.saturating_sub(started_at_epoch);
    format!(
        "HEALTHY pid={} state=RUNNING conns={} peers={} spaces={} uptime={}s",
        std::process::id(),
        conns,
        peers,
        spaces,
        uptime,
    )
}

// ── Named pipe server — M2 (Windows only) ──────────────────────────────────────

/// Start the named pipe server in the running Node instance.
/// Accepts one connection at a time; handles a control token inline or collects
/// batch lines until `__END__`; writes `OK\n` / `ERROR: …\n`; loops until
/// `shutdown_rx` delivers `true` (or its sender is dropped).
#[cfg(target_os = "windows")]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn start_pipe_server(
    pipe_name_str: String,
    data_dir: PathBuf,
    config_path: PathBuf,
    runtime: Arc<tokio::sync::Mutex<NodeRuntime>>,
    federation_registry: Arc<tokio::sync::Mutex<FederationRegistry>>,
    connections: app::Connections,
    started_at_epoch: u64,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
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
                ServerOptions::new()
                    .first_pipe_instance(true)
                    .create(&pipe_name_str)
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

        let connected = tokio::select! {
            r = server.connect() => r.is_ok(),
            _ = shutdown_rx.changed() => false,
        };

        if !connected || *shutdown_rx.borrow() {
            break;
        }

        let (reader_half, mut writer_half) = tokio::io::split(server);
        let mut reader = BufReader::new(reader_half);
        let mut lines: Vec<String> = Vec::new();
        let mut buf = String::new();

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
                    let summary =
                        build_health_line(&runtime, &connections, started_at_epoch).await;
                    let mut resp = summary;
                    resp.push('\n');
                    let _ = writer_half.write_all(resp.as_bytes()).await;
                    let _ = writer_half.flush().await;
                    continue;
                }
                "__STOP__" => {
                    let _ = writer_half.write_all(b"OK STOPPING\n").await;
                    let _ = writer_half.flush().await;
                    tracing::info!("__STOP__ received over pipe — exiting process");
                    // Brutal exit (same pattern as Client). Graceful WS-listener
                    // teardown is post-M2 polish.
                    std::process::exit(0);
                }
                "__RELOAD_CONFIG__" => {
                    let _ = writer_half
                        .write_all(b"NOT_IMPLEMENTED: config reload would require restarting the WS listener - out of scope for M2\n")
                        .await;
                    let _ = writer_half.flush().await;
                    continue;
                }
                _ => { /* fall through to batch path */ }
            }
        }

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
            match dispatch_line(line, &data_dir, &config_path, Some(&runtime), Some(&federation_registry)).await {
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
    }

    tracing::info!(pipe = %pipe_name_str, "Pipe server stopped");
}

// ── Batch invocation path — M2 (Windows only) ──────────────────────────────────

/// Second-process batch invocation path. Validates the .xgb file, connects to
/// the running Node's pipe, streams commands + `__END__`, reads result, returns
/// exit code. Creates its own tokio runtime — must NOT be called from within
/// an async context.
#[cfg(target_os = "windows")]
pub fn cmd_batch(raw_path: &str, pipe_name_str: &str, instance_label: Option<&str>) -> i32 {
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
async fn run_batch_client_async(
    raw_path: &str,
    pipe_name_str: &str,
    instance_label: Option<&str>,
) -> i32 {
    use std::io::BufRead as _;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;

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

    let file = match std::fs::File::open(&canonical) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: cannot open {:?}: {}", canonical, e);
            return 2;
        }
    };
    // `map_while(Result::ok)` stops on the first read error; `filter_map(|l| l.ok())`
    // would skip the error and keep reading — which, on `std::io::Lines`, can
    // spin forever if the underlying reader keeps returning Err. Behaviour-
    // adjacent fix per clippy::lines_filter_map_ok (Rust 1.95 hardening).
    let commands: Vec<String> = std::io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    let mut client = match ClientOptions::new().open(pipe_name_str) {
        Ok(c) => c,
        Err(_) => {
            let start_hint = match instance_label {
                Some(l) => format!("xgen-node.exe --instance {} before running --batch.", l),
                None => "xgen-node.exe before running --batch.".to_string(),
            };
            eprintln!(
                "error: no running xgen-node instance found at {}\n       Start {}",
                pipe_name_str, start_hint
            );
            return 3;
        }
    };

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

// ── Control-command client helpers — M2 (Windows only) ─────────────────────────

/// Open the pipe, send a single control line, read the single-line response.
/// Returns the trimmed response on success, or a human-readable Err on failure.
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

/// `--ping`: round-trip a `__PING__` and print latency in ms.
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

/// `--health`: print the running Node's one-line liveness summary.
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

/// `--stop`: signal the running Node to exit (the resident calls
/// `std::process::exit(0)` itself after responding).
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

/// `--reload-config`: surface the resident's honest `NOT_IMPLEMENTED` response.
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

// ── Non-Windows stubs ──────────────────────────────────────────────────────────

#[cfg(not(target_os = "windows"))]
pub fn cmd_ping(_pipe_name_str: &str) -> i32 {
    eprintln!("error: --ping is Windows-only in M2");
    1
}

#[cfg(not(target_os = "windows"))]
pub fn cmd_health(_pipe_name_str: &str) -> i32 {
    eprintln!("error: --health is Windows-only in M2");
    1
}

#[cfg(not(target_os = "windows"))]
pub fn cmd_stop(_pipe_name_str: &str) -> i32 {
    eprintln!("error: --stop is Windows-only in M2");
    1
}

#[cfg(not(target_os = "windows"))]
pub fn cmd_reload_config(_pipe_name_str: &str) -> i32 {
    eprintln!("error: --reload-config is Windows-only in M2");
    1
}

#[cfg(not(target_os = "windows"))]
pub fn cmd_batch(_raw_path: &str, _pipe_name_str: &str, _instance_label: Option<&str>) -> i32 {
    eprintln!("error: --batch is Windows-only in M2");
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{self, AuditEntry};
    use tempfile::tempdir;

    fn seed(dir: &Path) {
        let conn = audit::open_audit_db(dir).unwrap();
        let e = AuditEntry {
            timestamp: "2026-05-10T00:00:00.000Z".to_string(),
            verb: "federation accept".to_string(),
            actor: "alice".to_string(),
            actor_via: "batch".to_string(),
            target: None,
            args_hash: "h".to_string(),
            outcome: "ok".to_string(),
            error_code: None,
            error_message: None,
            correlation_id: None,
            meta_atts: "{}".to_string(),
        };
        audit::insert_entry(&conn, &e).unwrap();
    }

    #[tokio::test]
    async fn dispatch_routes_audit_query_verb() {
        let dir = tempdir().unwrap();
        seed(dir.path());
        let cfg = dir.path().join("xgen-node_config.toml");
        // Parses "audit query" through the clap grouping and runs admin_ops::audit_query.
        dispatch_line("audit query --actor alice", dir.path(), &cfg, None, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn dispatch_rejects_unknown_verb() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let err = dispatch_line("frobnicate the gizmo", dir.path(), &cfg, None, None)
            .await
            .unwrap_err();
        assert!(format!("{err}").contains("not supported"));
    }

    #[tokio::test]
    async fn dispatch_audit_query_bad_timestamp_surfaces_structured_code() {
        let dir = tempdir().unwrap();
        seed(dir.path());
        let cfg = dir.path().join("xgen-node_config.toml");
        // AdminError code_message bubbles through dispatch as the reply body.
        let err = dispatch_line("audit query --since not-a-ts", dir.path(), &cfg, None, None)
            .await
            .unwrap_err();
        assert!(format!("{err}").contains("AUDIT_5010"));
    }

    #[tokio::test]
    async fn dispatch_routes_identity_verb_to_admin_ops() {
        // `identity show <id>` parses through the clap grouping and reaches
        // admin_ops::identity_show. With no runtime handle, the verb surfaces a
        // structured GENERIC_4000 (require_runtime) — proving it routed to the
        // verb, not the "not supported" catch-all (which would say "not
        // supported"). The in-resident pipe always supplies the runtime.
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let err = dispatch_line(
            "identity show xgen://pubkey/ed25519:AAAA",
            dir.path(),
            &cfg,
            None,
            None,
        )
        .await
        .unwrap_err();
        let s = format!("{err}");
        assert!(s.contains("GENERIC_4000"), "got: {s}");
        assert!(!s.contains("not supported"), "got: {s}");
    }

    #[tokio::test]
    async fn dispatch_routes_federation_verb_to_admin_ops() {
        // `federation list` parses through the clap grouping and reaches
        // admin_ops::federation_list. With no federation-registry handle the
        // verb surfaces GENERIC_4000 (require_federation_registry) — proving it
        // routed to the verb, not the "not supported" catch-all.
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let err = dispatch_line("federation list", dir.path(), &cfg, None, None)
            .await
            .unwrap_err();
        let s = format!("{err}");
        assert!(s.contains("GENERIC_4000"), "got: {s}");
        assert!(!s.contains("not supported"), "got: {s}");
    }

    #[tokio::test]
    async fn dispatch_read_only_allowlist_preserved_for_unknown_admin_subverb() {
        // An unknown audit sub-verb falls through clap parse to the catch-all.
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let err = dispatch_line("audit frobnicate", dir.path(), &cfg, None, None)
            .await
            .unwrap_err();
        assert!(format!("{err}").contains("not supported"));
    }
}
