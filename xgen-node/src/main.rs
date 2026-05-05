// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use clap::{Parser, Subcommand};
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
use xgen_node_lib::{
    crypto::encoding,
    federation::handshake::{negotiate_serialisation, negotiate_version, sign_msg, verify_msg},
    identity::{
        keypair,
        registration::accept_registration,
        registry::IdentityRegistry,
    },
    node::runtime::NodeRuntime,
    space::state::{build_federation_add_event, sign_event},
    transport::{
        connection::{Connection, Inbound},
        server::Server,
    },
    wire::types::{
        Event, EventType, FederationCapabilities, FederationMessage, IdentityDeviceEntry,
        IdentityMessage, NegotiatedCapabilities, SpaceControlMessage, TransportMessage,
    },
};

// ── Node config ────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeConfig {
    node: NodeSection,
    paths: PathsSection,
    logging: LoggingSection,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeSection {
    listen: String,
    local_mode: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PathsSection {
    keypair_path: String,
    spaces_dir: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LoggingSection {
    level: String,
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
                level: "info".to_string(),
            },
        }
    }
}

// ── Connection tracking ────────────────────────────────────────────────────────

struct ConnectedClientInfo {
    identity_id: String,
    display_name: String,
    connected_at: String,
    events_received: u64,
}

type Connections = Arc<tokio::sync::Mutex<Vec<ConnectedClientInfo>>>;

// ── CLI ────────────────────────────────────────────────────────────────────────

/// XGen Protocol Node — federated, identity-verified communication.
///
/// Run without a subcommand to start the Node in foreground mode.
/// Use subcommands to initialise, inspect, or query a running Node.
#[derive(Parser)]
#[command(name = "xgen-node", version = build_info::VERSION)]
struct Cli {
    /// Path to config file. Default: <exe dir>/xgen-node_config.toml
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Override: start in Local Node mode regardless of config setting
    #[arg(long)]
    local: bool,

    #[command(subcommand)]
    command: Option<NodeCommand>,
}

#[derive(Subcommand)]
enum NodeCommand {
    /// Generate a keypair and default config next to the config file, then exit.
    /// Safe to run multiple times — will not overwrite an existing keypair.
    /// Prompts for a passphrase. Use empty passphrase for Local Node mode (Phase 1).
    Init {
        /// Use this passphrase instead of prompting (for scripts and CI).
        #[arg(long)]
        passphrase: Option<String>,
    },

    /// Print the current Node status from xgen-node_state.json.
    /// The Node must be running for this file to exist and be current.
    /// A warning is shown if the file is older than 30 seconds.
    Status,

    /// List all currently connected clients and federated peers.
    /// Reads from xgen-node_state.json.
    Connections,

    /// List all Spaces hosted on this Node with their Rooms and event counts.
    /// Reads from xgen-node_state.json.
    Spaces,

    /// List all known federated peer Nodes (active and previously connected).
    /// Reads from xgen-node_state.json.
    Peers,

    /// Identity management subcommands.
    Identity {
        #[command(subcommand)]
        action: IdentityAction,
    },

    /// Print version, build metadata, and Node ID if a keypair exists.
    Version,
}

#[derive(Subcommand)]
enum IdentityAction {
    /// List all Identities registered on this Node.
    /// Reads from xgen-node_identities.db in the Node's application folder.
    List,
}

// ── Entry point ────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| exe_dir().join("xgen-node_config.toml"));
    // data_dir: Tier-1 runtime files live here.
    // No --config → exe_dir() (guaranteed by GetModuleFileNameW on Windows).
    // --config /path/to/cfg.toml → /path/to/ (user chose that deployment dir).
    let data_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(exe_dir);

    let result = match &cli.command {
        None => run_node(&cli, &config_path, &data_dir).await,
        Some(NodeCommand::Init { passphrase }) => cmd_init(&data_dir, passphrase.as_deref()),
        Some(NodeCommand::Status) => cmd_status(&data_dir),
        Some(NodeCommand::Connections) => cmd_connections(&data_dir),
        Some(NodeCommand::Spaces) => cmd_spaces(&data_dir),
        Some(NodeCommand::Peers) => cmd_peers(&data_dir),
        Some(NodeCommand::Identity { action }) => match action {
            IdentityAction::List => cmd_identity_list(&data_dir),
        },
        Some(NodeCommand::Version) => cmd_version(&config_path, &data_dir),
    };
    if let Err(e) = result {
        eprintln!("{}", red(&format!("error: {:#}", e)));
        std::process::exit(1);
    }
}

// ── run (no subcommand — starts the Node server) ───────────────────────────────

async fn run_node(cli: &Cli, config_path: &Path, data_dir: &Path) -> Result<()> {
    // Load config (fall back to default if missing)
    let config = try_load_config(config_path).unwrap_or_default();
    let local_mode = config.node.local_mode || cli.local;

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

    // Initialise debug log — one file per run, datetime-stamped
    {
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
        let env_filter = if std::env::var("XGEN_LOG").is_ok() {
            EnvFilter::from_env("XGEN_LOG")
        } else {
            EnvFilter::new(&config.logging.level)
        };
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

    // Session header — written once, immediately after subscriber init.
    let started_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let session_id = format!("{:08x}", rand::random::<u32>());
    write_session_header(
        "node",
        Some(&node_id_uri),
        Some(&config.node.listen),
        None,
        "0.1",
        build_info::VERSION,
        &session_id,
        &started_at,
    );

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

    // Startup banner
    build_info::print_banner("xgen-node");
    println!();
    println!("Node ID:    {}", runtime.node_id);
    println!("Endpoint:   {}", config.node.listen);
    println!("Mode:       {}", if local_mode { "local" } else { "production" });
    println!("Identities: {} registered", runtime.identity_registry.len());
    println!();
    tracing::info!(node_id = %runtime.node_id, endpoint = %config.node.listen, "Node started");

    // Parse listen address from ws://host:port/path
    let listen_addr = parse_ws_addr(&config.node.listen)?;

    // Shared state
    let node_id = runtime.node_id.clone();
    let node_keypair = Arc::new(signing_key);
    let runtime = Arc::new(tokio::sync::Mutex::new(runtime));
    let connections: Connections = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // State writer task — writes xgen-node_state.json every 5 seconds
    {
        let rt = Arc::clone(&runtime);
        let conns = Arc::clone(&connections);
        let state_path = data_dir.join("xgen-node_state.json");
        let node_id_w = node_id.clone();
        let endpoint = config.node.listen.clone();
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

    // Bind WebSocket server
    let mut server = Server::bind(listen_addr)
        .await
        .with_context(|| format!("failed to bind on {listen_addr}"))?;
    println!("Listening on {} — press Ctrl+C to stop", config.node.listen);
    println!();

    // Accept loop
    loop {
        tokio::select! {
            result = server.accept() => {
                match result {
                    Ok(conn) => {
                        let rt = Arc::clone(&runtime);
                        let conns = Arc::clone(&connections);
                        let home = node_id.clone();
                        let lm = local_mode;
                        let ids = identities_path.clone();
                        let kp = Arc::clone(&node_keypair);
                        let sdir = spaces_dir.clone();
                        tokio::spawn(async move {
                            handle_connection(conn, rt, conns, kp, home, lm, ids, sdir).await;
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
            &config.node.listen,
            if local_mode { "local" } else { "production" },
            &started_at,
        );
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = std::fs::write(data_dir.join("xgen-node_state.json"), json);
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
    node_keypair: Arc<SigningKey>,
    home_node_id: String,
    local_mode: bool,
    identities_path: PathBuf,
    spaces_dir: PathBuf,
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

            // Process the first message then loop
            if let Inbound::Event(ref ev) = first_msg {
                trace_event(ev, EventDirection::In, &session_ctx);
            }
            process_inbound(
                &mut conn,
                first_msg,
                &identity_id,
                &home_node_id,
                local_mode,
                &runtime,
                &identities_path,
                &spaces_dir,
            )
            .await;

            loop {
                match conn.recv().await {
                    Ok(Inbound::Transport(TransportMessage::Goodbye { .. })) | Ok(Inbound::Closed) => break,
                    Ok(Inbound::Ping(_)) | Ok(Inbound::Pong(_)) | Ok(Inbound::Transport(_)) => {}
                    Ok(msg) => {
                        if let Inbound::Event(ref ev) = msg {
                            trace_event(ev, EventDirection::In, &session_ctx);
                        }
                        events_received += 1;
                        process_inbound(
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
                        // Update event counter in the tracker
                        let mut conns = connections.lock().await;
                        if let Some(c) = conns.iter_mut().find(|c| c.identity_id == identity_id) {
                            c.events_received = events_received;
                        }
                    }
                    Err(_) => break,
                }
            }

            // Remove from active connections on disconnect
            tracing::info!(identity_id = %identity_id, "Client disconnected");
            let mut conns = connections.lock().await;
            if let Some(pos) = conns.iter().position(|c| c.identity_id == identity_id) {
                conns.remove(pos);
            }
        }
    }
}

// ── Federation incoming handler ────────────────────────────────────────────────

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

    let (peer_node_id, peer_caps, peer_version) = match hello {
        FederationMessage::Hello {
            node_id,
            capabilities,
            protocol_version,
            ..
        } => (node_id, capabilities, protocol_version),
        _ => unreachable!(),
    };

    // Negotiate capabilities — "json" is the mandatory baseline
    let our_caps = FederationCapabilities::default();
    let serial = negotiate_serialisation(&our_caps.serialisation, &peer_caps.serialisation)
        .unwrap_or_else(|| "json".to_string());
    let neg_version =
        negotiate_version("0.1", &peer_version).unwrap_or_else(|| "0.1".to_string());

    // Send federation.capabilities (signed with node keypair)
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
            timestamp: ts,
            signature: None,
        },
        &node_keypair,
    );
    if conn.send_federation(&caps_msg).await.is_err() {
        return;
    }

    // Receive federation.accept — verify signature
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

    // Receive space.join_request
    let (req_space_id, req_node_id) = match conn.recv().await {
        Ok(Inbound::Space(SpaceControlMessage::JoinRequest { space_id, node_id })) => {
            (space_id, node_id)
        }
        _ => return,
    };

    tracing::info!(space_id = %req_space_id, peer_node_id = %peer_node_id, "Federation join request");

    // Snapshot history and tips atomically before building federation_add
    let (history, tips) = {
        let rt = runtime.lock().await;
        (rt.all_events(&req_space_id), rt.dag_tips(&req_space_id))
    };

    // Build and sign federation_add event
    let fed_add_ev = sign_event(
        build_federation_add_event(
            &node_keypair,
            &req_space_id,
            tips,
            &req_node_id,
            &session_id,
            &neg_version,
            &serial,
        ),
        &node_keypair,
    );

    let fed_add_id = fed_add_ev.event_id.as_deref().unwrap_or("(none)").to_string();
    let fed_add_type = fed_add_ev.event_type.to_string();
    trace_local(LocalAction::CreateEvent, &fed_add_id, Some(&fed_add_type), Some(&req_space_id), None);

    // Ingest federation_add on this node and persist to disk.
    {
        let mut rt = runtime.lock().await;
        rt.ingest_event(fed_add_ev.clone());
    }
    persist_event(&spaces_dir, &req_space_id, &fed_add_ev);
    trace_local(LocalAction::StoreEvent, &fed_add_id, None, Some(&req_space_id), None);
    trace_local(LocalAction::ApplyEvent, &fed_add_id, None, Some(&req_space_id), None);

    // Send history (topological order) then federation_add then goodbye
    let fed_session_ctx = SessionContext {
        identity_id: Some(peer_node_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(req_space_id.clone()),
    };
    let total = history.len() + 1;
    for ev in &history {
        trace_event(ev, EventDirection::Out, &fed_session_ctx);
        if conn.send_event(ev).await.is_err() {
            return;
        }
    }
    trace_event(&fed_add_ev, EventDirection::Out, &fed_session_ctx);
    if conn.send_event(&fed_add_ev).await.is_err() {
        return;
    }
    let _ = conn.goodbye("history_sync_complete").await;

    tracing::info!(peer_node_id = %peer_node_id, events_sent = total, "Federation established");
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
) {
    match msg {
        Inbound::Identity(im) => {
            handle_identity_msg(conn, im, identity_id, home_node_id, local_mode, runtime, identities_path).await;
        }
        Inbound::Event(event) => {
            let event_id = event.event_id.as_deref().unwrap_or("(none)").to_string();
            let event_type_str = event.event_type.to_string();
            let mut rt = runtime.lock().await;
            // Resolve space_id: space_create events have empty space_id; their event_id is the space_id.
            let space_id = if event.space_id.is_empty() {
                event.event_id.clone().unwrap_or_default()
            } else {
                event.space_id.clone()
            };
            match event.event_type {
                EventType::MessageText
                | EventType::MessageFile
                | EventType::MessageReaction
                | EventType::MessageRedact => {
                    match rt.accept_message(&space_id, event) {
                        Ok(_) => {
                            trace_local(LocalAction::ApplyEvent, &event_id, Some(&event_type_str), Some(&space_id), None);
                        }
                        Err(e) => {
                            tracing::error!(space_id = %space_id, reason = %e, "accept_message failed");
                            trace_local(LocalAction::RejectEvent, &event_id, Some(&event_type_str), Some(&space_id), None);
                        }
                    }
                }
                EventType::MembershipJoin => {
                    // Reject membership.join for an unknown Space (Fix 16 — secondary requirement).
                    if !rt.spaces.contains_key(&space_id) {
                        tracing::error!(space_id = %space_id, step = 10, "accept_message failed: space not found");
                        trace_local(LocalAction::RejectEvent, &event_id, Some(&event_type_str), Some(&space_id), Some(10));
                        return;
                    }
                    rt.ingest_event(event.clone());
                    persist_event(spaces_dir, &space_id, &event);
                    trace_local(LocalAction::StoreEvent, &event_id, None, Some(&space_id), None);
                    trace_local(LocalAction::ApplyEvent, &event_id, None, Some(&space_id), None);
                }
                _ => {
                    rt.ingest_event(event.clone());
                    persist_event(spaces_dir, &space_id, &event);
                    trace_local(LocalAction::StoreEvent, &event_id, None, Some(&space_id), None);
                    trace_local(LocalAction::ApplyEvent, &event_id, None, Some(&space_id), None);
                }
            }
        }
        _ => {}
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
                    {
                        let mut rt = runtime.lock().await;
                        let _ = rt.identity_registry.register(record);
                        let _ = rt.identity_registry.save(identities_path);
                    }
                    let ok = IdentityMessage::RegisterOk {
                        protocol_version: "0.1".to_string(),
                        identity_id: authenticated_id.to_string(),
                        registered_at: ts,
                    };
                    let _ = conn.send_identity(&ok).await;
                    tracing::info!(identity_id = %authenticated_id, "Identity registered");
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

fn cmd_init(data_dir: &Path, passphrase_arg: Option<&str>) -> Result<()> {
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

fn cmd_status(data_dir: &Path) -> Result<()> {
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

fn cmd_connections(data_dir: &Path) -> Result<()> {
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

fn cmd_spaces(data_dir: &Path) -> Result<()> {
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

fn cmd_peers(data_dir: &Path) -> Result<()> {
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

fn cmd_identity_list(data_dir: &Path) -> Result<()> {
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

fn cmd_version(config_path: &Path, data_dir: &Path) -> Result<()> {
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

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Directory of the running executable (Tier 1 files co-located with binary).
///
/// On Windows calls GetModuleFileNameW directly — this is immune to CWD, PATH
/// lookup, symlinks, and any shell wrapper tricks. On other platforms uses the
/// standard library's current_exe(). Panics if the path cannot be determined;
/// the binary cannot operate safely without knowing where it lives.
fn exe_dir() -> PathBuf {
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
fn persist_event(spaces_dir: &Path, space_id: &str, event: &Event) {
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
fn red(s: &str) -> String {
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
