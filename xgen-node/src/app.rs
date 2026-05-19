// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Node resident-mode logic (D-063). All long-running protocol behaviour —
//! WebSocket accept loop, connection handlers, identity registration,
//! federation handshake, fan-out, persistence — lives here so that every
//! entry point (CLI dispatcher, Tauri shell, future control-mode commands)
//! routes through one shared command layer (D-056).

use std::{
    collections::{BTreeMap, HashMap},
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use tokio::net::TcpStream;

use xgen_common::{
    build_info,
    event_trace::{
        EventDirection, ExitReason, LocalAction, SessionContext, SpaceRole,
        trace_event, trace_local, write_session_footer, write_session_header,
    },
    state::{ConnectedClient, HostedRoom, HostedSpace, NodeState},
};
use crate::{
    crypto::encoding,
    federation::handshake::{negotiate_serialisation, negotiate_version, sign_msg, verify_msg},
    identity::{
        keypair,
        registration::accept_registration,
        registry::{IdentityRecord, IdentityRegistry},
        replication::handle_incoming_replicate,
    },
    node::runtime::{DispatchOutcome, NodeRuntime},
    transport::{
        client::connect_url,
        connection::{Connection, Inbound},
        server::Server,
    },
    wire::types::{
        Event, FederationCapabilities, FederationMessage, IdentityDeviceEntry,
        IdentityMessage, IdentityReplicateMessage, NegotiatedCapabilities,
        TransportMessage,
    },
};

// Fan-out types live in `crate::fanout` so they are unit-testable
// without spawning real sockets.
use crate::fanout::{
    apply_fanout, collect_sync_history, ClientSenders, FanoutRequest, OutboundMsg,
};
use crate::federation_session::stream_federation_delta;

// ── Node config ────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NodeConfig {
    pub node: NodeSection,
    pub paths: PathsSection,
    pub logging: LoggingSection,
    /// F-6b / F-7a sync-pipeline tuning. Absent in pre-F-6/F-7 configs;
    /// `#[serde(default)]` keeps existing on-disk configs parsing.
    #[serde(default)]
    pub sync: SyncSection,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NodeSection {
    pub listen: String,
    pub local_mode: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PathsSection {
    pub keypair_path: String,
    pub spaces_dir: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoggingSection {
    pub level: String,
}

/// `[sync]` config section (F-6b + F-7a). Both fields are reference-implementation
/// defaults the operator may override; neither is protocol-fixed. Pattern
/// matches `[logging]` — protocol prescribes the mechanism, not the values.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncSection {
    /// F-6b safety-net timeout in seconds. When a sync_request issues, the
    /// requester waits up to this long for the peer's `SyncComplete` before
    /// surfacing a "peer never said done" error. Default 5; configurable per
    /// deployment (LAN can lower it; satellite-link can raise it).
    #[serde(default = "default_completion_timeout_seconds")]
    pub completion_timeout_seconds: u64,
    /// F-7a default page size. The Node returns at most `batch_size` events
    /// per `sync_request` response, with `continue_from` cursor on the trailing
    /// `SyncComplete` when more events remain. Default 1000.
    #[serde(default = "default_sync_batch_size")]
    pub batch_size: u32,
}

fn default_completion_timeout_seconds() -> u64 {
    5
}

fn default_sync_batch_size() -> u32 {
    1000
}

impl Default for SyncSection {
    fn default() -> Self {
        Self {
            completion_timeout_seconds: default_completion_timeout_seconds(),
            batch_size: default_sync_batch_size(),
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        let dir = exe_dir();
        Self {
            node: NodeSection {
                listen: "ws://127.0.0.1:8080/xgen".to_string(),
                local_mode: true,
            },
            paths: PathsSection {
                keypair_path: dir
                    .join("xgen-node_keypair.enc")
                    .to_string_lossy()
                    .to_string(),
                spaces_dir: Some(dir.join("spaces").to_string_lossy().to_string()),
            },
            logging: LoggingSection {
                level: "debug".to_string(),
            },
            sync: SyncSection::default(),
        }
    }
}

// ── Connection tracking ────────────────────────────────────────────────────────

pub(crate) struct ConnectedClientInfo {
    pub(crate) identity_id: String,
    pub(crate) display_name: String,
    pub(crate) connected_at: String,
    pub(crate) events_received: u64,
}

pub(crate) type Connections = Arc<tokio::sync::Mutex<Vec<ConnectedClientInfo>>>;

// ── run (no subcommand — starts the Node server) ───────────────────────────────

/// Options controlling how `run_node` initialises itself.
#[derive(Debug, Clone)]
pub struct RunNodeOpts {
    /// Force Local Node mode regardless of the config setting. `--local` flag.
    pub local_override: bool,
    /// Override the WS listener port per D-068 (precedence: flag > config >
    /// default). When `Some(port)`, replaces the port component of
    /// `config.node.listen` before binding. Host and path components remain
    /// from config. `None` → use the port from config (or `8080` if config
    /// is missing). `--port` flag.
    pub port_override: Option<u16>,
    /// Install the global tracing subscriber + write session header. Set false
    /// when the Tauri desktop shell has already done both.
    pub init_logging: bool,
    /// Suppress startup chatter on stdout (banner, "Listening on…"). Errors
    /// still surface; structured logs are unaffected. `--quiet` flag.
    pub quiet: bool,
    /// Override the effective logging level. Wins over config and XGEN_LOG.
    /// Only consulted when `init_logging` is true. `--log-level` flag.
    pub log_level_override: Option<String>,
    /// Instance label (from `--instance`). Used to derive the named-pipe name
    /// (D-043). `None` → `\\.\pipe\xgen-node`; `Some("n1")` → `xgen-node-n1`.
    pub instance_label: Option<String>,
}

impl Default for RunNodeOpts {
    fn default() -> Self {
        Self {
            local_override: false,
            port_override: None,
            init_logging: true,
            quiet: false,
            log_level_override: None,
            instance_label: None,
        }
    }
}

/// Resident-mode entry point. Long-running. Owns the lifecycle, binds the
/// WebSocket server, accepts connections, runs until Ctrl+C.
pub async fn run_node(
    config_path: &Path,
    data_dir: &Path,
    opts: RunNodeOpts,
) -> Result<()> {
    // Load config (fall back to default if missing)
    let config = try_load_config(config_path).unwrap_or_default();
    let local_mode = config.node.local_mode || opts.local_override;
    // F-7a: per-Node sync page size, resolved at startup. Used by every
    // sync_request handler in this Node process. No flag tier today — config
    // is the only override surface — so we don't go through `resolve_setting`
    // here. If a future `--sync-batch-size` flag lands it slots in via the
    // canonical helper, matching `--port` / `--log-level`.
    let sync_batch_size: usize = config.sync.batch_size as usize;

    // Load keypair before subscriber init so node_id is available for the session header.
    let keypair_path = PathBuf::from(&config.paths.keypair_path);
    if !keypair_path.exists() {
        bail!(
            "no keypair found at {}\n  Run 'xgen-node init' to initialise this Node folder.",
            keypair_path.display()
        );
    }
    let signing_key = keypair::load(&keypair_path, "").with_context(|| {
        format!(
            "failed to load keypair from {}\n  If passphrase-protected, use empty passphrase for Phase 1.",
            keypair_path.display()
        )
    })?;
    let node_id_uri = pubkey_uri(&signing_key);

    // Initialise debug log — one file per run, datetime-stamped.
    // Skipped when the desktop shell has already installed the subscriber.
    if opts.init_logging {
        use std::fs;
        use tracing_subscriber::{fmt, EnvFilter};

        let log_dir = data_dir.join("logs");
        fs::create_dir_all(&log_dir).expect("Failed to create logs/ directory");
        let now = chrono::Local::now();
        let log_filename = format!("xgen-node_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));
        let log_path = log_dir.join(&log_filename);
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to open log file");
        // D-068 — flag > env (XGEN_LOG) > config (`[logging].level`) >
        // "debug". Pre-J-079 this site already implemented the chain manually
        // but ad-hoc; converged on the canonical helper for consistency with
        // the four other entry-points and as a regression lock.
        let env_filter = EnvFilter::new(xgen_common::precedence::resolve_log_level(
            opts.log_level_override.as_deref(),
            Some(config.logging.level.as_str()),
        ));
        fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_ansi(false)
            .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
                "%Y-%m-%d %H:%M:%S%.3f".to_string(),
            ))
            .with_level(true)
            .with_writer(log_file)
            .init();
    }

    let started_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let session_id = format!("{:08x}", rand::random::<u32>());

    // Resolve the effective listen address per D-068 — `--port` overrides the
    // port component of `config.node.listen` (host and path remain from
    // config). Computed once here so banner output, session header, tracing,
    // state-file writes, and the actual bind all show the same value. See
    // xgen-common::precedence::resolve_setting and tasks/CLI_PRECEDENCE_AUDIT.md
    // for the rule.
    let listen_addr_from_config = parse_ws_addr(&config.node.listen)?;
    let resolved_port = xgen_common::precedence::resolve_setting(
        opts.port_override,
        None,
        Some(listen_addr_from_config.port()),
        8080u16,
    );
    let mut listen_addr = listen_addr_from_config;
    listen_addr.set_port(resolved_port);
    let effective_endpoint: String = if resolved_port == listen_addr_from_config.port() {
        config.node.listen.clone()
    } else {
        rewrite_url_port(&config.node.listen, resolved_port)
    };

    // Session header — written once, immediately after subscriber init.
    // Skipped when the desktop shell has already emitted one (with no
    // node_id, since the keypair wasn't yet loaded at that point).
    if opts.init_logging {
        write_session_header(
            "node",
            Some(&node_id_uri),
            Some(&effective_endpoint),
            None,
            "0.1",
            build_info::VERSION,
            &session_id,
            &started_at,
        );
    } else {
        // In desktop mode, the node_id wasn't known when the session header
        // was written. Log it now as a body line so it's still traceable.
        tracing::info!(node_id = %node_id_uri, endpoint = %effective_endpoint, "Node identity loaded");
    }

    // Spaces directory — Tier 2 (default: <data_dir>/spaces, overridable via config)
    let spaces_dir = config.paths.spaces_dir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("spaces"));
    let _ = std::fs::create_dir_all(&spaces_dir);

    // Load identity registry
    let identities_path = data_dir.join("xgen-node_identities.db");
    let mut runtime = NodeRuntime::new(signing_key.clone());
    if identities_path.exists() {
        match IdentityRegistry::load(&identities_path) {
            Ok(reg) => runtime.identity_registry = reg,
            Err(e) => {
                eprintln!("{}", yellow(&format!("warning: identity registry load failed: {e}")));
                tracing::warn!(reason = %e, "Identity registry load failed");
            }
        }
    }

    // Replay Space event logs from disk — MUST complete before network listener opens (spec 4.8.5).
    let replayed = replay_spaces_from_dir(&mut runtime, &spaces_dir);
    if replayed > 0 {
        eprintln!("Replayed {} Space event store(s) from disk.", replayed);
        tracing::info!(count = replayed, "Space event stores replayed from disk");
    }

    // Startup banner (suppressed under --quiet).
    if !opts.quiet {
        build_info::print_banner("xgen-node");
        println!();
        println!("Node ID:    {}", runtime.node_id);
        println!("Endpoint:   {}", effective_endpoint);
        println!("Mode:       {}", if local_mode { "local" } else { "production" });
        println!("Identities: {} registered", runtime.identity_registry.len());
        println!();
    }
    tracing::info!(node_id = %runtime.node_id, endpoint = %effective_endpoint, "Node started");

    // `listen_addr` and `effective_endpoint` already resolved above per D-068.

    // Shared state
    let node_id = runtime.node_id.clone();
    let node_keypair = Arc::new(signing_key);
    let runtime = Arc::new(tokio::sync::Mutex::new(runtime));
    let connections: Connections = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let client_senders: ClientSenders = Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    // State writer task — writes xgen-node_state.json every 5 seconds
    {
        let rt = Arc::clone(&runtime);
        let conns = Arc::clone(&connections);
        let state_path = data_dir.join("xgen-node_state.json");
        let node_id_w = node_id.clone();
        let endpoint = effective_endpoint.clone();
        let mode_str = if local_mode { "local" } else { "production" }.to_string();
        let started = started_at.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                let rt_guard = rt.lock().await;
                let conns_guard = conns.lock().await;
                let state =
                    build_node_state(&rt_guard, &conns_guard, &node_id_w, &endpoint, &mode_str, &started);
                drop(rt_guard);
                drop(conns_guard);
                if let Ok(json) = serde_json::to_string_pretty(&state) {
                    let _ = std::fs::write(&state_path, json);
                }
            }
        });
    }

    // Pending buffer timeout sweep — every 5 s, discard events that have waited
    // longer than PENDING_TIMEOUT_SECS for a missing predecessor (spec 3.9.6, E004002).
    {
        let rt = Arc::clone(&runtime);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                let mut rt_guard = rt.lock().await;
                let now = std::time::Instant::now();
                for (space_id, buf) in &mut rt_guard.pending {
                    for entry in buf.drain_timed_out(now) {
                        tracing::warn!(
                            space_id = %space_id,
                            event_id = %entry.event_id,
                            missing = ?entry.missing_predecessors,
                            error_code = 4002,
                            "4002 predecessor_timeout: pending event discarded after timeout"
                        );
                    }
                }
            }
        });
    }

    // Bind WebSocket server
    let mut server = Server::bind(listen_addr)
        .await
        .with_context(|| format!("failed to bind on {listen_addr}"))?;
    // Write the PID file now that the bind succeeded. Used by `--pid`.
    write_pid_file(data_dir);
    if !opts.quiet {
        println!("Listening on {} — press Ctrl+C to stop", effective_endpoint);
        println!();
    }

    // Named-pipe server (M2) — hosts the four control commands and the Node's
    // read-only `__BATCH__` subset. D-043 pipe-name convention. The watch-
    // channel sender must remain alive for the lifetime of this function's
    // async block — J-071 lesson; if the binding scope ends, the receiver's
    // `.changed()` returns Err immediately and the server breaks.
    let pipe_name_str = crate::pipe::pipe_name(opts.instance_label.as_deref());
    let started_at_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    #[cfg(target_os = "windows")]
    let _pipe_shutdown_hold = {
        let (tx, rx) = tokio::sync::watch::channel(false);
        let pipe_name_owned = pipe_name_str.clone();
        let pipe_data_dir = data_dir.to_path_buf();
        let pipe_config_path = config_path.to_path_buf();
        let pipe_runtime = Arc::clone(&runtime);
        let pipe_connections = Arc::clone(&connections);
        tokio::spawn(async move {
            crate::pipe::start_pipe_server(
                pipe_name_owned,
                pipe_data_dir,
                pipe_config_path,
                pipe_runtime,
                pipe_connections,
                started_at_epoch,
                rx,
            )
            .await;
        });
        tx
    };
    #[cfg(not(target_os = "windows"))]
    {
        let _ = pipe_name_str;
        tracing::warn!(
            "named pipe server is Windows-only; --ping/--health/--stop/--reload-config/--batch will fail on this platform"
        );
    }

    // Accept loop
    loop {
        tokio::select! {
            result = server.accept() => {
                match result {
                    Ok(conn) => {
                        let rt = Arc::clone(&runtime);
                        let conns = Arc::clone(&connections);
                        let senders = Arc::clone(&client_senders);
                        let home = node_id.clone();
                        let lm = local_mode;
                        let ids = identities_path.clone();
                        let kp = Arc::clone(&node_keypair);
                        let sdir = spaces_dir.clone();
                        let sbs = sync_batch_size;
                        tokio::spawn(async move {
                            handle_connection(conn, rt, conns, senders, kp, home, lm, ids, sdir, sbs).await;
                        });
                    }
                    Err(e) => {
                        eprintln!("{}", red(&format!("accept error: {e}")));
                        tracing::error!(reason = %e, "Connection accept error");
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down...");
                tracing::info!("Node shutting down");
                break;
            }
        }
    }

    // Final state write on shutdown
    {
        let rt = runtime.lock().await;
        let conns = connections.lock().await;
        let state = build_node_state(
            &rt,
            &conns,
            &node_id,
            &effective_endpoint,
            if local_mode { "local" } else { "production" },
            &started_at,
        );
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = std::fs::write(data_dir.join("xgen-node_state.json"), json);
        }
    }

    // Warn about any events still buffered (pending prev_events that never arrived).
    {
        let rt = runtime.lock().await;
        for (space_id, buf) in &rt.pending {
            if !buf.is_empty() {
                tracing::warn!(space_id = %space_id, unresolved = buf.len(), "pending_buffer_at_shutdown");
            }
        }
    }

    write_session_footer(ExitReason::Shutdown);
    Ok(())
}

// ── Connection handler ─────────────────────────────────────────────────────────

async fn handle_connection(
    mut conn: Connection<TcpStream>,
    runtime: Arc<tokio::sync::Mutex<NodeRuntime>>,
    connections: Connections,
    client_senders: ClientSenders,
    node_keypair: Arc<SigningKey>,
    home_node_id: String,
    local_mode: bool,
    identities_path: PathBuf,
    spaces_dir: PathBuf,
    sync_batch_size: usize,
) {
    // Transport challenge-response authentication
    let identity_id = match conn.server_authenticate().await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(reason = %e, "Transport authentication failed");
            return;
        }
    };

    // Build session context — Phase 1 local mode: all authenticated sessions are Owner-level.
    // Phase 2 will resolve role from the space registry per space_id.
    let session_ctx = SessionContext {
        identity_id: Some(identity_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: None,
    };

    // Read first message — determines whether this is a federation or client connection
    let first = match conn.recv().await {
        Ok(m) => m,
        Err(_) => return,
    };

    match first {
        // ── Federation connection (another Node connecting) ───────────────
        Inbound::Federation(fm) if matches!(&fm, FederationMessage::Hello { .. }) => {
            tracing::info!(peer_node_id = %identity_id, "Incoming federation connection");
            handle_federation_incoming(
                &mut conn,
                fm,
                runtime,
                node_keypair,
                home_node_id,
                spaces_dir,
            )
            .await;
        }

        // ── Client connection ─────────────────────────────────────────────
        first_msg => {
            tracing::info!(identity_id = %identity_id, "Client authenticated");
            let display_name = {
                let rt = runtime.lock().await;
                rt.identity_registry
                    .get(&identity_id)
                    .and_then(|r| r.display_name.clone())
                    .unwrap_or_default()
            };
            let connected_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let mut events_received: u64 = 0;

            // Register in the active-connection tracker
            {
                let mut conns = connections.lock().await;
                conns.push(ConnectedClientInfo {
                    identity_id: identity_id.clone(),
                    display_name,
                    connected_at,
                    events_received: 0,
                });
            }

            // Outbound channel — the fan-out path and history-push path send
            // OutboundMsg into this channel; the select! arm below drains it
            // and writes to the WebSocket. Capacity is generous so a slow
            // client cannot block other clients' fan-out for long; if full,
            // try_send drops the message (the client will catch up via
            // transport.sync_request after reconnect).
            let (out_tx, mut out_rx) =
                tokio::sync::mpsc::channel::<OutboundMsg>(1024);
            {
                let mut senders = client_senders.lock().await;
                senders.insert(identity_id.clone(), out_tx.clone());
            }

            // Process the first message via the same dispatch as the loop's
            // recv arm — otherwise a sync_request arriving as the first
            // post-auth message is silently dropped (process_inbound has no
            // out_tx in scope).
            let mut deferred_first: Option<Inbound> = Some(first_msg);

            // Main loop: select between inbound recv and outbound drain.
            loop {
                // Drain the deferred first message first, otherwise call recv.
                let incoming: Result<Inbound, _> = if let Some(m) = deferred_first.take() {
                    Ok(m)
                } else {
                    tokio::select! {
                        biased;
                        r = conn.recv() => r,
                        Some(out_msg) = out_rx.recv() => {
                            match out_msg {
                                OutboundMsg::Event(ev) => {
                                    trace_event(&ev, EventDirection::Out, &session_ctx);
                                    if conn.send_event(&ev).await.is_err() {
                                        break;
                                    }
                                }
                                OutboundMsg::HistoryBatch { events } => {
                                    for ev in events {
                                        trace_event(&ev, EventDirection::Out, &session_ctx);
                                        if conn.send_event(&ev).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                OutboundMsg::SyncComplete {
                                    since,
                                    new_tip,
                                    continue_from,
                                } => {
                                    // F-6: explicit end-of-batch signal. Replaces
                                    // the 500ms quiet-time heuristic; the requester
                                    // waits for this message instead of guessing
                                    // via inter-event silence.
                                    let msg = TransportMessage::SyncComplete {
                                        protocol_version: "0.1".to_string(),
                                        since,
                                        new_tip,
                                        continue_from,
                                    };
                                    if conn.send_transport(&msg).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            continue;
                        }
                    }
                };

                // Dispatch the inbound (whether deferred-first or fresh recv).
                match incoming {
                    Ok(Inbound::Transport(TransportMessage::Goodbye { .. }))
                    | Ok(Inbound::Closed) => break,
                    Ok(Inbound::Ping(_)) | Ok(Inbound::Pong(_)) => {}
                    // transport.sync_request is the only Transport variant
                    // the Node responds to from a client (spec 3.3.6 + F-6 + F-7).
                    // After delivering the batch we emit an explicit SyncComplete
                    // with an optional `continue_from` pagination cursor — replaces
                    // the 500ms quiet-time heuristic for end-of-stream detection
                    // (D-065, honest behaviour over polite behaviour).
                    Ok(Inbound::Transport(TransportMessage::SyncRequest {
                        since,
                        limit,
                        ..
                    })) => {
                        // F-7a: per-request limit (None → config default 1000).
                        // sync_batch_size is resolved at run_node startup from
                        // [sync].batch_size (config → default 1000).
                        let effective_limit =
                            limit.map(|n| n as usize).unwrap_or(sync_batch_size);
                        let (events, continue_from) = collect_sync_history(
                            &runtime,
                            &identity_id,
                            &since,
                            effective_limit,
                        )
                        .await;
                        // new_tip: whole-batch model — last delivered event_id,
                        // or echo `since` when the page is empty (caller is
                        // caught up at the position they asked from).
                        let new_tip = events
                            .last()
                            .and_then(|e| e.event_id.clone())
                            .unwrap_or_else(|| since.clone());
                        let _ = out_tx
                            .send(OutboundMsg::HistoryBatch { events })
                            .await;
                        let _ = out_tx
                            .send(OutboundMsg::SyncComplete {
                                since: since.clone(),
                                new_tip,
                                continue_from,
                            })
                            .await;
                    }
                    Ok(Inbound::Transport(_)) => {}
                    Ok(msg) => {
                        if let Inbound::Event(ref ev) = msg {
                            trace_event(ev, EventDirection::In, &session_ctx);
                        }
                        events_received += 1;
                        let fanout = process_inbound(
                            &mut conn,
                            msg,
                            &identity_id,
                            &home_node_id,
                            local_mode,
                            &runtime,
                            &identities_path,
                            &spaces_dir,
                        )
                        .await;
                        apply_fanout(fanout, &identity_id, &runtime, &client_senders).await;
                        let mut conns = connections.lock().await;
                        if let Some(c) =
                            conns.iter_mut().find(|c| c.identity_id == identity_id)
                        {
                            c.events_received = events_received;
                        }
                    }
                    Err(_) => break,
                }
            }

            // Remove from active connections on disconnect
            tracing::info!(identity_id = %identity_id, "Client disconnected");
            {
                let mut senders = client_senders.lock().await;
                senders.remove(&identity_id);
            }
            let mut conns = connections.lock().await;
            if let Some(pos) = conns.iter().position(|c| c.identity_id == identity_id) {
                conns.remove(pos);
            }
        }
    }
}

// ── Federation incoming handler ────────────────────────────────────────────────

/// F-1a federation handshake receiver (runbook §3.3 Locked wire shape +
/// §3.3.1 Locks 1-7). Phase 3 reshape: replaces the pre-F-1a
/// "handshake → space.join_request → state.federation_add → dump → goodbye"
/// flow with "handshake (with bilateral tips) → bilateral delta delivery via
/// stream_federation_delta → session stays open as the F-2 persistent push
/// channel (Phase 4 push lands on top of this)."
///
/// The peer's `shared_spaces` and `tips` from Hello are the source of truth
/// for what to deliver. `SpaceControlMessage::JoinRequest` is no longer part
/// of the post-handshake flow; the receiver determines delta scope from the
/// peer's wire-visible tips per the locked semantics in runbook §3.3.
async fn handle_federation_incoming(
    conn: &mut Connection<TcpStream>,
    hello: FederationMessage,
    runtime: Arc<tokio::sync::Mutex<NodeRuntime>>,
    node_keypair: Arc<SigningKey>,
    home_node_id: String,
    spaces_dir: PathBuf,
) {
    // Verify hello signature
    if let Err(e) = verify_msg(&hello) {
        tracing::error!(reason = %e, "Federation hello: invalid signature");
        return;
    }

    // §3.3 Locked wire shape (Option 3 bilateral tips): extract peer's
    // shared_spaces + tips from Hello — these drive both the Capabilities
    // reply (our tips for the same Spaces) and the delta-delivery iteration
    // domain inside stream_federation_delta.
    let (peer_node_id, peer_caps, peer_version, peer_endpoint, peer_shared_spaces, peer_tips) =
        match hello {
            FederationMessage::Hello {
                node_id,
                capabilities,
                protocol_version,
                node_endpoint,
                shared_spaces,
                tips,
                ..
            } => (
                node_id,
                capabilities,
                protocol_version,
                node_endpoint,
                shared_spaces,
                tips,
            ),
            _ => unreachable!(),
        };

    // Negotiate capabilities — "json" is the mandatory baseline.
    let our_caps = FederationCapabilities::default();
    let serial = negotiate_serialisation(&our_caps.serialisation, &peer_caps.serialisation)
        .unwrap_or_else(|| "json".to_string());
    let neg_version =
        negotiate_version("0.1", &peer_version).unwrap_or_else(|| "0.1".to_string());

    // Build our local tips for each Space the peer declares as shared. A Space
    // we don't host (no entry in stores) yields no tip; a Space with multiple
    // DAG tips (concurrent forks) picks the lexicographically smallest for
    // wire-shape determinism — Phase 1/2 DAGs are single-tip in practice, but
    // the rule is total. Empty `our_tips` is valid (Locked semantics: "I
    // participate in zero shared Spaces"); absent entry under a non-empty
    // shared_spaces means "send full history" — handled by stream_federation_delta.
    let our_tips: BTreeMap<String, String> = {
        let rt = runtime.lock().await;
        peer_shared_spaces
            .iter()
            .filter_map(|space_id| {
                let local_tips = rt.dag_tips(space_id);
                local_tips.into_iter().min().map(|tip| (space_id.clone(), tip))
            })
            .collect()
    };

    // Send federation.capabilities (signed with node keypair) — carries our tips.
    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let caps_msg = sign_msg(
        FederationMessage::Capabilities {
            protocol_version: "0.1".to_string(),
            node_id: home_node_id.clone(),
            capabilities: our_caps,
            negotiated: NegotiatedCapabilities {
                serialisation: serial.clone(),
                protocol_version: neg_version.clone(),
            },
            tips: our_tips,
            timestamp: ts,
            signature: None,
        },
        &node_keypair,
    );
    if conn.send_federation(&caps_msg).await.is_err() {
        return;
    }

    // Receive federation.accept — verify signature.
    let accept_msg = match conn.recv().await {
        Ok(Inbound::Federation(fm @ FederationMessage::Accept { .. })) => fm,
        _ => return,
    };
    if verify_msg(&accept_msg).is_err() {
        return;
    }
    let session_id = match accept_msg {
        FederationMessage::Accept { session_id, .. } => session_id,
        _ => return,
    };

    tracing::info!(
        peer_node_id = %peer_node_id,
        shared_spaces_count = peer_shared_spaces.len(),
        "Federation handshake reached ACTIVE"
    );

    // Store the peer's endpoint URL so we can push identity replicas to it later.
    if let Some(url) = peer_endpoint {
        let mut rt = runtime.lock().await;
        rt.record_peer_url(&peer_node_id, url);
    }

    // F-1a bilateral delta delivery — applies the a-i symmetry rule for
    // state.federation_add per Space where peer's tips[S] is absent and we
    // have events for S (runbook §3.3.1 Lock 2).
    if let Err(e) = stream_federation_delta(
        conn,
        &runtime,
        &peer_shared_spaces,
        &peer_tips,
        &peer_node_id,
        &session_id,
        &neg_version,
        &serial,
        &node_keypair,
        &spaces_dir,
    )
    .await
    {
        tracing::warn!(
            peer_node_id = %peer_node_id,
            error = %e,
            "Federation delta delivery failed; session terminating"
        );
        return;
    }

    tracing::info!(
        peer_node_id = %peer_node_id,
        "Federation delta delivery complete; session stays open"
    );

    // Phase 4 plugs in the federation-push sender here; intentionally unused
    // in Phase 3 — channel exists to avoid restructuring the loop at Phase 4
    // ship. Sized 1024 to match the client-connection precedent at
    // app.rs:622-734 (runbook §3.3.1 Lock 3).
    let (_out_tx, mut out_rx) = tokio::sync::mpsc::channel::<OutboundMsg>(1024);

    // F-2 long-lived continuous session — minimal Phase 3 scope: drain
    // inbound, exit on Goodbye/Closed. Phase 4 wires the receive-side event
    // dispatch + fan-out into the Ok(Inbound::Event(_)) arm; for now Phase 3
    // discards because federation push doesn't exist on the sending side yet,
    // so receiving Events on a federation session is unexpected.
    loop {
        tokio::select! {
            biased;
            r = conn.recv() => {
                match r {
                    Ok(Inbound::Transport(TransportMessage::Goodbye { .. }))
                    | Ok(Inbound::Closed)
                    | Err(_) => break,
                    Ok(Inbound::Ping(_)) | Ok(Inbound::Pong(_)) => {}
                    Ok(_) => {
                        // Phase 4: dispatch Inbound::Event through process_inbound
                        // + apply_fanout here.
                    }
                }
            }
            Some(_out_msg) = out_rx.recv() => {
                // Reserved for Phase 4 — no sender clones of _out_tx are
                // registered in Phase 3, so this arm is unreachable.
                unreachable!("federation push not enabled in Phase 3");
            }
        }
    }

    tracing::info!(peer_node_id = %peer_node_id, "Federation session ended");
}

// ── Inbound message processor ──────────────────────────────────────────────────

async fn process_inbound(
    conn: &mut Connection<TcpStream>,
    msg: Inbound,
    identity_id: &str,
    home_node_id: &str,
    local_mode: bool,
    runtime: &Arc<tokio::sync::Mutex<NodeRuntime>>,
    identities_path: &Path,
    spaces_dir: &Path,
) -> FanoutRequest {
    match msg {
        Inbound::Identity(im) => {
            handle_identity_msg(conn, im, identity_id, home_node_id, local_mode, runtime, identities_path).await;
            FanoutRequest::none()
        }
        Inbound::IdentityReplicate(irm) => {
            handle_identity_replicate_msg(conn, irm, runtime).await;
            FanoutRequest::none()
        }
        Inbound::Event(event) => {
            // F-4 unified dispatcher (Phase 2 of Federation Event Propagation).
            // Replaces the pre-F-4 three-path branching (Path A messages via
            // `accept_message`; Path B `MembershipJoin` direct `ingest_event`
            // with no validation; Path C state.* direct `ingest_event` with no
            // validation). After F-4 every event family routes through
            // `NodeRuntime::dispatch_event`, which runs the validation core
            // (signature + timestamp + predecessor presence + DAG structure +
            // sender / membership / permission per F-4b) before semantic
            // pre-checks (AI role / capability / operator target) and ingest.
            //
            // HeldPending now applies uniformly to all event families per
            // F-4a (30 s timeout, shared `PendingBuffer`). Drain re-dispatches
            // unblocked events through the full pipeline — see
            // `NodeRuntime::drain_pending_uniform`.
            let event_id = event.event_id.as_deref().unwrap_or("(none)").to_string();
            let event_type_str = event.event_type.to_string();
            let space_id_for_persist = if event.space_id.is_empty() {
                event.event_id.clone().unwrap_or_default()
            } else {
                event.space_id.clone()
            };

            let mut rt = runtime.lock().await;
            let outcome = rt.dispatch_event(event.clone());
            drop(rt);

            match outcome {
                DispatchOutcome::Accepted { new_joiner } => {
                    persist_event(spaces_dir, &space_id_for_persist, &event);
                    trace_local(
                        LocalAction::StoreEvent,
                        &event_id,
                        Some(&event_type_str),
                        Some(&space_id_for_persist),
                        None,
                    );
                    trace_local(
                        LocalAction::ApplyEvent,
                        &event_id,
                        Some(&event_type_str),
                        Some(&space_id_for_persist),
                        None,
                    );
                    FanoutRequest {
                        event: Some(event),
                        new_joiner,
                    }
                }
                DispatchOutcome::HeldPending => {
                    tracing::debug!(
                        space_id = %space_id_for_persist,
                        event_id = %event_id,
                        event_type = %event_type_str,
                        "event buffered — waiting for unknown prev_events"
                    );
                    FanoutRequest::none()
                }
                DispatchOutcome::Rejected(reason) => {
                    tracing::error!(
                        space_id = %space_id_for_persist,
                        event_id = %event_id,
                        event_type = %event_type_str,
                        reason = %reason,
                        "process_inbound: event rejected"
                    );
                    trace_local(
                        LocalAction::RejectEvent,
                        &event_id,
                        Some(&event_type_str),
                        Some(&space_id_for_persist),
                        None,
                    );
                    FanoutRequest::none()
                }
            }
        }
        _ => FanoutRequest::none(),
    }
}


// ── Identity message handler ───────────────────────────────────────────────────

async fn handle_identity_msg(
    conn: &mut Connection<TcpStream>,
    msg: IdentityMessage,
    authenticated_id: &str,
    home_node_id: &str,
    local_mode: bool,
    runtime: &Arc<tokio::sync::Mutex<NodeRuntime>>,
    identities_path: &Path,
) {
    match msg {
        IdentityMessage::Register { .. } => {
            let already = {
                let rt = runtime.lock().await;
                rt.identity_registry.contains(authenticated_id)
            };
            let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            match accept_registration(&msg, authenticated_id, already, local_mode, home_node_id, &ts) {
                Ok(record) => {
                    let identity_id_str = authenticated_id.to_string();
                    let node_keypair_clone = {
                        let mut rt = runtime.lock().await;
                        let _ = rt.identity_registry.register(record.clone());
                        let _ = rt.identity_registry.save(identities_path);
                        rt.node_keypair.clone()
                    };
                    let ok = IdentityMessage::RegisterOk {
                        protocol_version: "0.1".to_string(),
                        identity_id: identity_id_str.clone(),
                        registered_at: ts,
                    };
                    let _ = conn.send_identity(&ok).await;
                    tracing::info!(identity_id = %identity_id_str, "Identity registered");
                    // Replicate to peer Nodes asynchronously (spec 3.13.1).
                    let rt_clone = Arc::clone(runtime);
                    tokio::spawn(async move {
                        push_identity_to_peers(record, rt_clone, node_keypair_clone).await;
                    });
                }
                Err(e) => {
                    let (code, msg_str) = e.to_registration_code();
                    let fail = IdentityMessage::RegisterFail {
                        protocol_version: "0.1".to_string(),
                        error_code: code,
                        error_string: msg_str.to_string(),
                        timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    };
                    let _ = conn.send_identity(&fail).await;
                    tracing::warn!(identity_id = %authenticated_id, reason = %msg_str, "Identity registration rejected");
                }
            }
        }
        IdentityMessage::Get { identity_id, .. } => {
            let rt = runtime.lock().await;
            let response = match rt.identity_registry.get(&identity_id) {
                Some(record) => IdentityMessage::Record {
                    protocol_version: "0.1".to_string(),
                    identity_id: record.identity_id.clone(),
                    display_name: record.display_name.clone(),
                    registered_at: record.registered_at.clone(),
                    devices: record
                        .devices
                        .iter()
                        .map(|d| IdentityDeviceEntry {
                            device_id: d.device_id.clone(),
                            device_name: d.device_name.clone(),
                            authorised_at: d.authorised_at.clone(),
                        })
                        .collect(),
                    home_node: record.home_node.clone(),
                },
                None => IdentityMessage::NotFound {
                    protocol_version: "0.1".to_string(),
                    identity_id,
                },
            };
            drop(rt);
            let _ = conn.send_identity(&response).await;
        }
        _ => {}
    }
}

// ── Identity replication — inbound (spec 3.13.4) ──────────────────────────────

/// Handle an incoming `identity.replicate` message on this Node (acting as a replica).
/// Accepts or rejects the record, then sends `identity.replicate_ack` or a transport error.
async fn handle_identity_replicate_msg(
    conn: &mut Connection<TcpStream>,
    msg: IdentityReplicateMessage,
    runtime: &Arc<tokio::sync::Mutex<NodeRuntime>>,
) {
    let (identity_id, identity_record_value, update_version) = match msg {
        IdentityReplicateMessage::Replicate {
            identity_id,
            identity_record,
            update_version,
            ..
        } => (identity_id, identity_record, update_version),
        // replicate_ack arriving here would be a protocol error — ignore silently.
        IdentityReplicateMessage::ReplicateAck { .. } => return,
    };

    // Deserialise identity_record Value → IdentityRecord.
    let record: IdentityRecord = match serde_json::from_value(identity_record_value) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(identity_id = %identity_id, reason = %e, "identity.replicate: invalid record payload");
            return;
        }
    };

    let result = {
        let mut rt = runtime.lock().await;
        handle_incoming_replicate(record, &mut rt.identity_registry)
    };

    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    match result {
        Ok(()) => {
            let ack = IdentityReplicateMessage::ReplicateAck {
                protocol_version: "0.1".to_string(),
                identity_id: identity_id.clone(),
                update_version,
                timestamp: ts,
                signature: None,
            };
            let _ = conn.send_identity_replicate(&ack).await;
            tracing::info!(identity_id = %identity_id, update_version = update_version, "Identity replica accepted");
        }
        Err(e) => {
            tracing::warn!(identity_id = %identity_id, reason = %e, "identity.replicate: rejected");
            // Send transport error 3020 so the home Node can handle the stale-version case.
            let err_msg = TransportMessage::Error {
                protocol_version: "0.1".to_string(),
                error_code: e.error_code(),
                error_string: e.to_string(),
                timestamp: ts,
            };
            let _ = conn.send_transport(&err_msg).await;
        }
    }
}

// ── Identity replication — outbound (spec 3.13.1) ─────────────────────────────

/// Push a newly registered Identity record to all known peer Nodes (spec 3.13.1).
/// Runs asynchronously after registration — failures are logged but not fatal.
async fn push_identity_to_peers(
    record: IdentityRecord,
    runtime: Arc<tokio::sync::Mutex<NodeRuntime>>,
    node_keypair: ed25519_dalek::SigningKey,
) {
    // Snapshot peer_urls under the lock, then release before any I/O.
    let peer_urls: Vec<(String, String)> = {
        let rt = runtime.lock().await;
        rt.peer_urls.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    };

    if peer_urls.is_empty() {
        return;
    }

    let identity_record_value = match serde_json::to_value(&record) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(reason = %e, "push_identity_to_peers: serialise failed");
            return;
        }
    };
    let identity_id = record.identity_id.clone();
    let update_version = record.update_version;

    for (peer_node_id, url) in peer_urls {
        let iid = identity_id.clone();
        let val = identity_record_value.clone();
        let kp = node_keypair.clone();
        let rt = Arc::clone(&runtime);

        tokio::spawn(async move {
            match connect_url(&url).await {
                Err(e) => {
                    tracing::warn!(peer = %peer_node_id, url = %url, reason = %e, "replication: connect failed");
                    return;
                }
                Ok(mut conn) => {
                    if conn.client_authenticate(&kp).await.is_err() {
                        tracing::warn!(peer = %peer_node_id, "replication: authenticate failed");
                        return;
                    }
                    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
                    let replicate = IdentityReplicateMessage::Replicate {
                        protocol_version: "0.1".to_string(),
                        identity_id: iid.clone(),
                        identity_record: val,
                        update_version,
                        timestamp: ts,
                        signature: None,
                    };
                    if conn.send_identity_replicate(&replicate).await.is_err() {
                        tracing::warn!(peer = %peer_node_id, "replication: send failed");
                        return;
                    }
                    // Wait for ack (best-effort; timeout is handled by recv() WebSocket layer).
                    match conn.recv().await {
                        Ok(Inbound::IdentityReplicate(IdentityReplicateMessage::ReplicateAck { .. })) => {
                            tracing::info!(identity_id = %iid, peer = %peer_node_id, "Replication ack received");
                            let mut rt_guard = rt.lock().await;
                            rt_guard.replica_registry.add_replica(&iid, &peer_node_id);
                        }
                        other => {
                            tracing::warn!(identity_id = %iid, peer = %peer_node_id, msg = ?other, "replication: unexpected response");
                        }
                    }
                }
            }
        });
    }
}

// ── State file builder ─────────────────────────────────────────────────────────

fn build_node_state(
    rt: &NodeRuntime,
    conns: &[ConnectedClientInfo],
    node_id: &str,
    endpoint: &str,
    mode: &str,
    started_at: &str,
) -> NodeState {
    let spaces = rt
        .spaces
        .values()
        .map(|space| {
            let store = rt.stores.get(&space.space_id);
            let total_events = store.map(|s| s.len() as u64).unwrap_or(0);

            let rooms = space
                .rooms
                .values()
                .map(|room| {
                    let room_events = store
                        .map(|s| s.values().filter(|e| e.room_id == room.room_id).count() as u64)
                        .unwrap_or(0);
                    HostedRoom {
                        room_id: room.room_id.clone(),
                        name: room.name.clone(),
                        event_count: room_events,
                        last_activity: String::new(),
                    }
                })
                .collect();

            HostedSpace {
                space_id: space.space_id.clone(),
                name: space.name.clone().unwrap_or_else(|| {
                    space.space_id[..space.space_id.len().min(20)].to_string()
                }),
                member_count: space.members.len(),
                event_count: total_events,
                rooms,
            }
        })
        .collect();

    let clients = conns
        .iter()
        .map(|c| ConnectedClient {
            identity_id: c.identity_id.clone(),
            display_name: c.display_name.clone(),
            connected_at: c.connected_at.clone(),
            events_sent: 0,
            events_received: c.events_received,
        })
        .collect();

    NodeState {
        node_id: node_id.to_string(),
        version: build_info::VERSION.to_string(),
        build: build_info::GIT_HASH.to_string(),
        started_at: started_at.to_string(),
        mode: mode.to_string(),
        endpoint: endpoint.to_string(),
        updated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        clients,
        peers: vec![],
        spaces,
    }
}

// ── init ───────────────────────────────────────────────────────────────────────

pub fn cmd_init(data_dir: &Path, passphrase_arg: Option<&str>) -> Result<()> {
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("failed to create data directory: {}", data_dir.display()))?;

    let keypair_file = data_dir.join("xgen-node_keypair.enc");
    let config_file = data_dir.join("xgen-node_config.toml");

    if keypair_file.exists() {
        println!("Keypair already exists: {}", keypair_file.display());
        println!("Skipping keypair generation. Delete the file to regenerate.");
    } else {
        println!("Generating keypair...");
        let passphrase = match passphrase_arg {
            Some(p) => p.to_string(),
            None => prompt_passphrase()?,
        };
        let signing_key = keypair::generate();
        keypair::save(&signing_key, &keypair_file, &passphrase)
            .context("failed to save keypair")?;
        println!("Keypair saved:  {}", keypair_file.display());
        println!("Node ID:        {}", pubkey_uri(&signing_key));
    }

    if config_file.exists() {
        println!("Config already exists: {} — not overwritten.", config_file.display());
    } else {
        let mut cfg = NodeConfig::default();
        // Point keypair_path to this data_dir, not exe_dir
        cfg.paths.keypair_path = keypair_file.to_string_lossy().to_string();
        let toml_str = toml::to_string_pretty(&cfg).context("failed to serialise config")?;
        std::fs::write(&config_file, toml_str).context("failed to write config")?;
        println!("Config saved:   {}", config_file.display());
    }

    println!();
    println!("Run 'xgen-node --config {}' to start.", config_file.display());
    Ok(())
}

// ── status ─────────────────────────────────────────────────────────────────────

pub fn cmd_status(data_dir: &Path) -> Result<()> {
    let state = load_state(data_dir)?;
    let age = age_seconds(&state.updated_at);

    let total_events: u64 = state.spaces.iter().map(|s| s.event_count).sum();

    println!("xgen-node status");
    println!("================");
    println!("Node ID:      {}", state.node_id);
    println!("Version:      {}", state.version);
    println!("Uptime:       {}", uptime_str(&state.started_at));
    println!("Mode:         {}", state.mode);
    println!("Endpoint:     {}", state.endpoint);
    println!(
        "Connections:  {} client{}, {} federated peer{}",
        state.clients.len(),
        plural(state.clients.len()),
        state.peers.len(),
        plural(state.peers.len()),
    );
    println!("Spaces:       {} hosted", state.spaces.len());
    println!("Events:       {} total across all spaces", total_events);
    if age > 30 {
        println!(
            "State file:   {}",
            yellow(&format!("WARNING — updated {}s ago (Node may not be running)", age))
        );
    } else {
        println!("State file:   updated {}s ago", age);
    }
    Ok(())
}

// ── connections ────────────────────────────────────────────────────────────────

pub fn cmd_connections(data_dir: &Path) -> Result<()> {
    let state = load_state(data_dir)?;

    println!(
        "Connections ({} client{}, {} peer{})",
        state.clients.len(),
        plural(state.clients.len()),
        state.peers.len(),
        plural(state.peers.len()),
    );

    if state.clients.is_empty() && state.peers.is_empty() {
        println!("\n  No active connections.");
        return Ok(());
    }

    if !state.clients.is_empty() {
        println!();
        println!("CLIENTS");
        println!(
            "  {:<44}  {:<16}  {:<14}  {:<12}  {}",
            "Identity", "Display name", "Connected", "Events sent", "Received"
        );
        for c in &state.clients {
            println!(
                "  {:<44}  {:<16}  {:<14}  {:<12}  {}",
                short_id(&c.identity_id),
                c.display_name,
                format_ago(age_seconds(&c.connected_at)),
                c.events_sent,
                c.events_received,
            );
        }
    }

    if !state.peers.is_empty() {
        println!();
        println!("FEDERATED PEERS");
        println!(
            "  {:<44}  {:<30}  {:<10}  {}",
            "Node ID", "Endpoint", "State", "Since"
        );
        for p in &state.peers {
            println!(
                "  {:<44}  {:<30}  {:<10}  {}",
                short_id(&p.node_id),
                p.endpoint,
                p.state,
                format_ago(age_seconds(&p.connected_at)),
            );
        }
    }
    Ok(())
}

// ── spaces ─────────────────────────────────────────────────────────────────────

pub fn cmd_spaces(data_dir: &Path) -> Result<()> {
    let state = load_state(data_dir)?;

    println!("Spaces ({})", state.spaces.len());

    if state.spaces.is_empty() {
        println!("\n  No hosted Spaces.");
        return Ok(());
    }

    for space in &state.spaces {
        println!();
        println!("  Space: {}", space.name);
        println!("  ID:    {}", space.space_id);
        println!(
            "  Rooms: {}   Members: {}   Events: {}",
            space.rooms.len(),
            space.member_count,
            space.event_count
        );
        for room in &space.rooms {
            let activity = if room.last_activity.is_empty() {
                "no activity yet".to_string()
            } else {
                format!("{} ago", fmt_duration(age_seconds(&room.last_activity)))
            };
            println!();
            println!("    Room: {}", room.name);
            println!("    ID:   {}", room.room_id);
            println!("    Events: {}   Last activity: {}", room.event_count, activity);
        }
    }
    Ok(())
}

// ── peers ──────────────────────────────────────────────────────────────────────

pub fn cmd_peers(data_dir: &Path) -> Result<()> {
    let state = load_state(data_dir)?;

    println!("Federated Peers ({})", state.peers.len());

    if state.peers.is_empty() {
        println!("\n  No known federated peers.");
        return Ok(());
    }

    for peer in &state.peers {
        println!();
        println!("  Node ID:     {}", peer.node_id);
        println!("  Endpoint:    {}", peer.endpoint);
        println!("  State:       {}", peer.state);
        println!("  Session ID:  {}", peer.session_id);
        println!("  Version:     {} / {}", peer.version, peer.protocol);
        if !peer.shared_spaces.is_empty() {
            println!("  Spaces:      {}", peer.shared_spaces.join(", "));
        }
        println!("  Connected:   {}", format_ago(age_seconds(&peer.connected_at)));
        println!("  Last seen:   {}", format_ago(age_seconds(&peer.last_seen_at)));
    }
    Ok(())
}

// ── identity list ──────────────────────────────────────────────────────────────

pub fn cmd_identity_list(data_dir: &Path) -> Result<()> {
    let identities_path = data_dir.join("xgen-node_identities.db");

    let registry = IdentityRegistry::load(&identities_path).with_context(|| {
        format!(
            "failed to load identity registry at {}\n  Is the Node initialised? Run 'xgen-node init'.",
            identities_path.display()
        )
    })?;

    let mut all = registry.all();
    all.sort_by(|a, b| a.registered_at.cmp(&b.registered_at));

    println!("Registered Identities ({})", all.len());

    if all.is_empty() {
        println!("\n  No identities registered.");
        return Ok(());
    }

    println!();
    for record in all {
        let name = record.display_name.as_deref().unwrap_or("<no name>");
        let age = fmt_registration_age(&record.registered_at);
        println!(
            "  {}   {:<20}  registered {}   {} device{}",
            record.identity_id,
            name,
            age,
            record.devices.len(),
            plural(record.devices.len()),
        );
    }
    Ok(())
}

// ── version ────────────────────────────────────────────────────────────────────

pub fn cmd_version(config_path: &Path, data_dir: &Path) -> Result<()> {
    println!("xgen-node {}", build_info::full_version());
    println!("Commit:   {}", build_info::GIT_HASH);

    let cfg = try_load_config(config_path);
    let keypair_path = cfg
        .map(|c| c.paths.keypair_path)
        .unwrap_or_else(|| data_dir.join("xgen-node_keypair.enc").to_string_lossy().to_string());
    let keypair_path = PathBuf::from(&keypair_path);

    if keypair_path.exists() {
        match keypair::load(&keypair_path, "") {
            Ok(signing_key) => println!("Node ID:  {}", pubkey_uri(&signing_key)),
            Err(_) => println!(
                "Node ID:  (keypair is passphrase-protected — use 'xgen-node status' when running)"
            ),
        }
    } else {
        println!("Node ID:  (no keypair — run 'xgen-node init')");
    }
    Ok(())
}

// ── whoami ─────────────────────────────────────────────────────────────────────

/// Prints `node_id` (xgen://pubkey/...) by loading the keypair. The
/// operator_display_name lives on the NodeAnnouncement record, not the local
/// config today; that field will surface here once announcement metadata is
/// stored locally (post-M1).
pub fn cmd_whoami(config_path: &Path, data_dir: &Path) -> Result<()> {
    let cfg = try_load_config(config_path);
    let keypair_path = cfg
        .map(|c| c.paths.keypair_path)
        .unwrap_or_else(|| {
            data_dir
                .join("xgen-node_keypair.enc")
                .to_string_lossy()
                .to_string()
        });
    let keypair_path = PathBuf::from(&keypair_path);
    if !keypair_path.exists() {
        bail!(
            "no keypair found at {}\n  Run 'xgen-node init' to initialise this Node folder.",
            keypair_path.display()
        );
    }
    let signing_key = keypair::load(&keypair_path, "")
        .with_context(|| format!("failed to load keypair from {}", keypair_path.display()))?;
    println!("Node ID:                 {}", pubkey_uri(&signing_key));
    println!("operator_display_name:   (not in local config — see NodeAnnouncement metadata)");
    Ok(())
}

// ── check-config / print-config ───────────────────────────────────────────────

/// `--check-config`: parse the config (defaults if missing), print OK / first
/// validation error, exit 0 on success and non-zero on failure.
pub fn cmd_check_config(config_path: &Path) -> Result<()> {
    if !config_path.exists() {
        println!(
            "config OK: {} (file absent — defaults will apply)",
            config_path.display()
        );
        return Ok(());
    }
    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("cannot read {}", config_path.display()))?;
    let _: NodeConfig = toml::from_str(&content)
        .with_context(|| format!("invalid TOML in {}", config_path.display()))?;
    println!("config OK: {}", config_path.display());
    Ok(())
}

/// `--print-config`: serialise the effective config (file merged with
/// defaults) to TOML on stdout. Read-only, no pipe contact.
pub fn cmd_print_config(config_path: &Path) -> Result<()> {
    let cfg = try_load_config(config_path).unwrap_or_default();
    let s = toml::to_string_pretty(&cfg).context("failed to serialise config to TOML")?;
    print!("{}", s);
    Ok(())
}

// ── pid ────────────────────────────────────────────────────────────────────────

const PID_FILE_NAME: &str = "xgen-node.pid";

/// Write the current process PID into `<data_dir>/xgen-node.pid`. Called by
/// `run_node` immediately after the WS bind succeeds. Silent on I/O failure
/// (the PID file is a convenience for the `--pid` flag, not load-bearing).
fn write_pid_file(data_dir: &Path) {
    let pid = std::process::id();
    let path = data_dir.join(PID_FILE_NAME);
    let _ = std::fs::write(&path, pid.to_string());
}

/// `--pid`: read `<data_dir>/xgen-node.pid` and print its contents.
/// No liveness check — a stale file remains until overwritten by the next
/// resident or removed manually.
pub fn cmd_pid(data_dir: &Path) -> Result<()> {
    let path = data_dir.join(PID_FILE_NAME);
    let pid_str = std::fs::read_to_string(&path)
        .with_context(|| format!("no resident PID file at {}", path.display()))?;
    println!("{}", pid_str.trim());
    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Directory of the running executable (Tier 1 files co-located with binary).
///
/// On Windows calls GetModuleFileNameW directly — this is immune to CWD, PATH
/// lookup, symlinks, and any shell wrapper tricks. On other platforms uses the
/// standard library's current_exe(). Panics if the path cannot be determined;
/// the binary cannot operate safely without knowing where it lives.
pub fn exe_dir() -> PathBuf {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let mut buf: Vec<u16> = vec![0u16; 260]; // start at MAX_PATH
        loop {
            // NULL module handle → path of the running exe
            let n = unsafe {
                windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW(
                    std::ptr::null_mut(), // NULL → path of the running exe
                    buf.as_mut_ptr(),
                    buf.len() as u32,
                )
            };
            if n == 0 {
                panic!("GetModuleFileNameW failed — cannot determine executable path");
            }
            if n < buf.len() as u32 {
                let path = PathBuf::from(OsString::from_wide(&buf[..n as usize]));
                return path
                    .parent()
                    .expect("executable path has no parent directory")
                    .to_path_buf();
            }
            // Buffer was full — double it and retry (handles paths beyond MAX_PATH)
            let new_len = buf.len() * 2;
            buf.resize(new_len, 0);
        }
    }

    #[cfg(not(windows))]
    {
        std::env::current_exe()
            .expect("cannot determine executable path")
            .parent()
            .expect("executable path has no parent directory")
            .to_path_buf()
    }
}

fn try_load_config(path: &Path) -> Option<NodeConfig> {
    let text = std::fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()
}

/// Read `[logging].level` from `xgen-node_config.toml`, returning `None` if
/// the file is missing or fails to parse. Used by both Node entry-points
/// (`run_node` here for `--service`, `desktop::init_logging` for the Tauri
/// shell) so both feed `xgen_common::precedence::resolve_log_level` (D-068).
pub fn read_config_log_level(config_path: &Path) -> Option<String> {
    try_load_config(config_path).map(|c| c.logging.level)
}

/// Parse a ws://host:port/path URL to a SocketAddr for binding.
fn parse_ws_addr(url: &str) -> Result<SocketAddr> {
    let stripped = url
        .strip_prefix("ws://")
        .or_else(|| url.strip_prefix("wss://"))
        .with_context(|| format!("expected ws:// or wss:// URL, got: {url}"))?;
    let host_port = stripped.split('/').next().unwrap_or(stripped);
    host_port
        .parse::<SocketAddr>()
        .with_context(|| format!("invalid address in WebSocket URL: {host_port}"))
}

/// Rewrite the port component of a `ws://host:port/path` URL. Used to
/// reconstruct the effective endpoint string after `--port` override per
/// D-068 — so banner output, session headers, state-file writes, and
/// tracing all show the actual bound port rather than the config-as-written
/// value. Returns the original `url` unchanged if it does not start with
/// `ws://` or `wss://`.
pub(crate) fn rewrite_url_port(url: &str, new_port: u16) -> String {
    let (scheme, rest) = if let Some(r) = url.strip_prefix("ws://") {
        ("ws", r)
    } else if let Some(r) = url.strip_prefix("wss://") {
        ("wss", r)
    } else {
        return url.to_string();
    };
    let (host_port, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    let host = host_port.rsplit_once(':').map(|(h, _)| h).unwrap_or(host_port);
    format!("{scheme}://{host}:{new_port}{path}")
}

/// Load the Node state file from the data directory (Tier 1 — co-located with config).
fn load_state(data_dir: &Path) -> Result<NodeState> {
    let path = data_dir.join("xgen-node_state.json");
    if !path.exists() {
        bail!(
            "state file not found: {}\n  Is the Node running? Start it with 'xgen-node'.",
            path.display()
        );
    }
    let json = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read state file: {}", path.display()))?;
    serde_json::from_str(&json).context("state file is corrupt or has an unexpected format")
}

fn pubkey_uri(signing_key: &SigningKey) -> String {
    let encoded = encoding::encode(signing_key.verifying_key().as_bytes());
    format!("xgen://pubkey/ed25519:{}", encoded)
}

/// Truncate a full xgen:// URI for table display.
fn short_id(uri: &str) -> String {
    let rest = uri
        .strip_prefix("xgen://hash/")
        .or_else(|| uri.strip_prefix("xgen://pubkey/"))
        .unwrap_or(uri);
    if let Some((scheme, key)) = rest.split_once(':') {
        let trunc: String = key.chars().take(8).collect();
        format!("{scheme}:{trunc}...")
    } else {
        let trunc: String = uri.chars().take(20).collect();
        format!("{trunc}...")
    }
}

/// Seconds since the given RFC 3339 timestamp, or i64::MAX on parse error.
fn age_seconds(timestamp: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_seconds())
        .unwrap_or(i64::MAX)
}

/// "2h 14m 38s" — used for uptime display.
fn uptime_str(started_at: &str) -> String {
    let secs = age_seconds(started_at);
    if secs <= 0 {
        return "unknown".to_string();
    }
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

/// "14m 22s" — used in table columns.
fn fmt_duration(secs: i64) -> String {
    if secs <= 0 {
        return "0s".to_string();
    }
    if secs < 60 {
        return format!("{}s", secs);
    }
    let m = secs / 60;
    let s = secs % 60;
    if m < 60 {
        return format!("{}m {:02}s", m, s);
    }
    let h = m / 60;
    let m = m % 60;
    format!("{}h {}m", h, m)
}

fn format_ago(secs: i64) -> String {
    format!("{} ago", fmt_duration(secs))
}

fn fmt_registration_age(timestamp: &str) -> String {
    let secs = age_seconds(timestamp);
    if secs < 0 {
        return "just now".to_string();
    }
    if secs < 120 {
        return format!("{}s ago", secs);
    }
    let m = secs / 60;
    if m < 120 {
        return format!("{}m ago", m);
    }
    let h = m / 60;
    if h < 48 {
        return format!("{}h ago", h);
    }
    format!("{}d ago", h / 24)
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn prompt_passphrase() -> Result<String> {
    let pass =
        rpassword::prompt_password("Passphrase: ").context("failed to read passphrase")?;
    let confirm =
        rpassword::prompt_password("Confirm:    ").context("failed to read passphrase")?;
    if pass != confirm {
        bail!("Passphrases do not match.");
    }
    Ok(pass)
}

// ── Space event persistence (Fix 16) ──────────────────────────────────────────

/// Filename-safe representation of a space_id for use as a JSON store filename.
fn space_file_name(space_id: &str) -> String {
    let clean = space_id
        .strip_prefix("xgen://hash/sha256:")
        .map(|h| format!("sha256_{}", h))
        .unwrap_or_else(|| space_id.replace(['/', ':', '.'], "_"));
    format!("{}.json", clean)
}

/// Append one Event to the per-Space JSON store.
/// Idempotent — won't write duplicates (matched by event_id).
pub(crate) fn persist_event(spaces_dir: &Path, space_id: &str, event: &Event) {
    if space_id.is_empty() {
        return;
    }
    let _ = std::fs::create_dir_all(spaces_dir);
    let path = spaces_dir.join(space_file_name(space_id));
    let mut events: Vec<Event> = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    // Avoid duplicate entries.
    if let Some(id) = &event.event_id {
        if events.iter().any(|e| e.event_id.as_deref() == Some(id.as_str())) {
            return;
        }
    }
    events.push(event.clone());
    if let Ok(json) = serde_json::to_string(&events) {
        let _ = std::fs::write(&path, json);
    }
}

/// Scan `spaces_dir` for *.json Space event stores and replay all events through
/// `NodeRuntime::ingest_event`. Returns the number of Space files replayed.
///
/// This MUST be called before the network listener opens (spec 4.8.5).
fn replay_spaces_from_dir(runtime: &mut NodeRuntime, spaces_dir: &Path) -> usize {
    let entries = match std::fs::read_dir(spaces_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let events: Vec<Event> = match std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            Some(e) => e,
            None => continue,
        };
        if events.is_empty() {
            continue;
        }
        for event in events {
            runtime.ingest_event(event);
        }
        count += 1;
    }
    count
}

/// ANSI red — applied only when stderr is a terminal.
pub fn red(s: &str) -> String {
    use std::io::IsTerminal;
    if std::io::stderr().is_terminal() {
        format!("\x1b[31m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

/// ANSI yellow — applied only when stderr is a terminal.
fn yellow(s: &str) -> String {
    use std::io::IsTerminal;
    if std::io::stderr().is_terminal() {
        format!("\x1b[33m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // rewrite_url_port — shipped from the CLI Precedence Audit (D-068, J-079)
    // alongside the `--port` flag threading. Exercises the port-substitution
    // helper used to reconstruct the effective endpoint string after override.

    #[test]
    fn rewrite_url_port_replaces_port_in_ws_url_with_path() {
        let r = rewrite_url_port("ws://127.0.0.1:8080/xgen", 9192);
        assert_eq!(r, "ws://127.0.0.1:9192/xgen");
    }

    #[test]
    fn rewrite_url_port_replaces_port_in_wss_url() {
        let r = rewrite_url_port("wss://example.com:443/xgen", 8443);
        assert_eq!(r, "wss://example.com:8443/xgen");
    }

    #[test]
    fn rewrite_url_port_preserves_path_with_multiple_segments() {
        let r = rewrite_url_port("ws://127.0.0.1:8080/xgen/v1", 9192);
        assert_eq!(r, "ws://127.0.0.1:9192/xgen/v1");
    }

    #[test]
    fn rewrite_url_port_handles_url_with_no_path() {
        let r = rewrite_url_port("ws://127.0.0.1:8080", 9192);
        assert_eq!(r, "ws://127.0.0.1:9192");
    }

    #[test]
    fn rewrite_url_port_returns_unchanged_for_unrecognised_scheme() {
        let r = rewrite_url_port("http://example.com:80/", 8080);
        assert_eq!(r, "http://example.com:80/");
    }
}
