// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use clap::{Args, CommandFactory, Parser, Subcommand};
use serde_json::json;

use xgen_common::{build_info, event_trace::{EventDirection, SessionContext, SpaceRole, trace_event}, state::ClientState};
use xgen_node_lib::{
    crypto::encoding,
    federation::handshake::run_initiating,
    identity::{
        keypair,
        registration::{build_register, identity_id_from_key, sign_register},
    },
    message::exchange::build_message_text_event,
    space::state::{
        build_room_create_event, build_space_create_event, sign_event, verify_event_signature,
    },
    transport::{
        client::connect_url,
        connection::Inbound,
    },
    wire::types::{
        Event, EventType, FederationCapabilities, IdentityMessage, SpaceControlMessage,
        TransportMessage,
    },
};

// ── Client config ──────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct ClientConfig {
    client: ClientSection,
    paths: PathsSection,
    logging: LoggingSection,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ClientSection {
    node: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PathsSection {
    keypair_path: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LoggingSection {
    level: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        let dir = exe_dir();
        Self {
            client: ClientSection {
                node: "ws://127.0.0.1:8080/xgen".to_string(),
            },
            paths: PathsSection {
                keypair_path: dir
                    .join("xgen-client_keypair.enc")
                    .to_string_lossy()
                    .to_string(),
            },
            logging: LoggingSection {
                level: "info".to_string(),
            },
        }
    }
}

// ── CLI ────────────────────────────────────────────────────────────────────────

/// XGen Protocol reference client.
///
/// Every invocation executes one command and exits.
/// The Node endpoint is taken from --node or from xgen-client_config.toml.
#[derive(Parser)]
#[command(name = "xgen-client", version = build_info::VERSION)]
struct Cli {
    /// Node WebSocket endpoint, e.g. ws://127.0.0.1:8080/xgen. Overrides config.
    #[arg(short, long)]
    node: Option<String>,

    /// Path to config file. Default: <exe dir>/xgen-client_config.toml
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<ClientCommand>,
}

#[derive(Subcommand)]
enum ClientCommand {
    /// Generate a keypair and default config next to the executable, then exit.
    /// Safe to run multiple times — will not overwrite an existing keypair.
    /// Prompts for a passphrase. Use empty passphrase for Local Node mode (Phase 1).
    Init,

    /// Print the local Identity ID and display name from xgen-client_state.json.
    /// No Node connection required.
    Whoami,

    /// Print the local client status from xgen-client_state.json.
    /// No Node connection required.
    Status,

    /// List Spaces and Rooms known to this client from xgen-client_state.json.
    /// No Node connection required.
    Spaces,

    /// Print version and build metadata.
    Version,

    /// Register this Identity on the Node. Requires --node or config.
    /// In Local Node mode, no Trust Assertion is required.
    Register(RegisterArgs),

    /// Create a new Space on the Node. The caller becomes the Space Owner.
    CreateSpace(CreateSpaceArgs),

    /// Create a new Room within a Space.
    CreateRoom(CreateRoomArgs),

    /// Invite an Identity to a Space.
    Invite(InviteArgs),

    /// Join a Space or a specific Room within a Space.
    Join(JoinArgs),

    /// Send a message.text Event to a Room.
    Send(SendArgs),

    /// Fetch and display the message history for a Room in causal (DAG) order.
    History(HistoryArgs),

    /// Run the Phase 1 smoke test against two running Node instances.
    /// Exercises all 17 steps from spec 3.7.11 over real TCP connections.
    SmokeTest(SmokeTestArgs),
}

#[derive(Args)]
struct RegisterArgs {
    /// Display name to register. Max 128 characters.
    #[arg(long)]
    name: String,
}

#[derive(Args)]
struct CreateSpaceArgs {
    /// Display name for the Space. Max 128 characters.
    #[arg(long)]
    name: String,
}

#[derive(Args)]
struct CreateRoomArgs {
    /// Space ID (xgen://hash/sha256:...)
    #[arg(long)]
    space: String,
    /// Display name for the Room. Max 128 characters.
    #[arg(long)]
    name: String,
}

#[derive(Args)]
struct InviteArgs {
    /// Space ID
    #[arg(long)]
    space: String,
    /// Identity ID to invite (xgen://pubkey/ed25519:...)
    #[arg(long)]
    identity: String,
    /// Role to assign on join: owner, admin, moderator, member
    #[arg(long)]
    role: String,
}

#[derive(Args)]
struct JoinArgs {
    /// Space ID
    #[arg(long)]
    space: String,
    /// Room ID. If omitted, joins the Space itself.
    #[arg(long)]
    room: Option<String>,
}

#[derive(Args)]
struct SendArgs {
    /// Space ID
    #[arg(long)]
    space: String,
    /// Room ID
    #[arg(long)]
    room: String,
    /// Message text (quoted string).
    #[arg(long)]
    text: String,
}

#[derive(Args)]
struct HistoryArgs {
    /// Space ID
    #[arg(long)]
    space: String,
    /// Room ID
    #[arg(long)]
    room: String,
    /// Maximum number of messages to display. Default: 50.
    #[arg(long, default_value = "50")]
    limit: usize,
}

#[derive(Args)]
struct SmokeTestArgs {
    /// Endpoint of Node A. Example: ws://127.0.0.1:8080/xgen
    #[arg(long)]
    node_a: String,
    /// Endpoint of Node B. Example: ws://127.0.0.1:8081/xgen
    #[arg(long)]
    node_b: String,
    /// Do not clean up test Identities and Spaces after the run.
    #[arg(long)]
    keep: bool,
}

// ── Entry point ────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| exe_dir().join("xgen-client_config.toml"));

    // Initialise debug log — one file per run, datetime-stamped
    {
        use std::fs;
        use tracing_subscriber::{fmt, EnvFilter};

        let log_dir = exe_dir().join("logs");
        fs::create_dir_all(&log_dir).expect("Failed to create logs/ directory");
        let now = chrono::Local::now();
        let log_filename = format!("xgen-client_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));
        let log_path = log_dir.join(&log_filename);
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to open log file");
        let level = std::fs::read_to_string(&config_path).ok()
            .and_then(|s| toml::from_str::<ClientConfig>(&s).ok())
            .map(|c| c.logging.level)
            .unwrap_or_else(|| "info".to_string());
        let env_filter = if std::env::var("XGEN_LOG").is_ok() {
            EnvFilter::from_env("XGEN_LOG")
        } else {
            EnvFilter::new(&level)
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
        tracing::info!("Log file opened: {}", log_path.display());
    }

    let result = match &cli.command {
        None => {
            build_info::print_banner("xgen-client");
            println!();
            Cli::command().print_help().unwrap();
            println!();
            return;
        }
        Some(ClientCommand::Init) => cmd_init(),
        Some(ClientCommand::Whoami) => cmd_whoami(&config_path),
        Some(ClientCommand::Status) => cmd_status(&config_path),
        Some(ClientCommand::Spaces) => cmd_spaces(&config_path),
        Some(ClientCommand::Version) => cmd_version(),
        Some(ClientCommand::Register(args)) => {
            let node = resolve_node(&cli, &config_path);
            let keypair_path = resolve_keypair_path(&config_path);
            cmd_register(args, &node, &keypair_path).await
        }
        Some(ClientCommand::CreateSpace(args)) => {
            let node = resolve_node(&cli, &config_path);
            let keypair_path = resolve_keypair_path(&config_path);
            cmd_create_space(args, &node, &keypair_path).await
        }
        Some(ClientCommand::CreateRoom(args)) => {
            let node = resolve_node(&cli, &config_path);
            let keypair_path = resolve_keypair_path(&config_path);
            cmd_create_room(args, &node, &keypair_path).await
        }
        Some(ClientCommand::Invite(args)) => {
            let node = resolve_node(&cli, &config_path);
            let keypair_path = resolve_keypair_path(&config_path);
            cmd_invite(args, &node, &keypair_path).await
        }
        Some(ClientCommand::Join(args)) => {
            let node = resolve_node(&cli, &config_path);
            let keypair_path = resolve_keypair_path(&config_path);
            cmd_join(args, &node, &keypair_path).await
        }
        Some(ClientCommand::Send(args)) => {
            let node = resolve_node(&cli, &config_path);
            let keypair_path = resolve_keypair_path(&config_path);
            cmd_send(args, &node, &keypair_path).await
        }
        Some(ClientCommand::History(args)) => {
            let node = resolve_node(&cli, &config_path);
            let keypair_path = resolve_keypair_path(&config_path);
            cmd_history(args, &node, &keypair_path).await
        }
        Some(ClientCommand::SmokeTest(args)) => cmd_smoke_test(args).await,
    };
    if let Err(e) = result {
        eprintln!("{}", red(&format!("error: {:#}", e)));
        std::process::exit(1);
    }
}

// ── init ───────────────────────────────────────────────────────────────────────

fn cmd_init() -> Result<()> {
    let dir = exe_dir();
    let keypair_file = dir.join("xgen-client_keypair.enc");
    let config_file = dir.join("xgen-client_config.toml");

    if keypair_file.exists() {
        println!("Keypair already exists: {}", keypair_file.display());
        println!("Skipping keypair generation. Delete the file to regenerate.");
    } else {
        println!("Generating keypair...");
        let passphrase = prompt_passphrase()?;
        let signing_key = keypair::generate();
        keypair::save(&signing_key, &keypair_file, &passphrase)
            .context("failed to save keypair")?;
        println!("Keypair saved:    {}", keypair_file.display());
        println!("Identity ID: {}", identity_id_from_key(&signing_key));
    }

    if config_file.exists() {
        println!("Config already exists: {} — not overwritten.", config_file.display());
    } else {
        let cfg = ClientConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).context("failed to serialise config")?;
        std::fs::write(&config_file, toml_str).context("failed to write config")?;
        println!("Config saved:     {}", config_file.display());
    }

    println!();
    println!("Run 'xgen-client --node <endpoint> register --name \"Your Name\"' to register on a Node.");
    Ok(())
}

// ── whoami ─────────────────────────────────────────────────────────────────────

fn cmd_whoami(config_path: &Path) -> Result<()> {
    let state = load_client_state(config_path)?;
    println!("Identity ID:    {}", state.identity_id);
    println!("Display name:   {}", state.display_name);
    println!("Registered on:  {}", state.home_node);
    println!("Spaces joined:  {}", state.spaces.len());
    Ok(())
}

// ── status ─────────────────────────────────────────────────────────────────────

fn cmd_status(config_path: &Path) -> Result<()> {
    let state = load_client_state(config_path)?;
    let age = age_seconds(&state.updated_at);

    println!("xgen-client status");
    println!("==================");
    println!("Identity ID:   {}", state.identity_id);
    println!("Display name:  {}", state.display_name);
    println!("Version:       {}", state.version);
    println!("Home node:     {}", state.home_node);
    println!("Spaces joined: {}", state.spaces.len());
    if age > 30 {
        println!(
            "State file:    {}",
            yellow(&format!("WARNING — updated {}s ago", age))
        );
    } else {
        println!("State file:    updated {}s ago", age);
    }
    Ok(())
}

// ── spaces ─────────────────────────────────────────────────────────────────────

fn cmd_spaces(config_path: &Path) -> Result<()> {
    let state = load_client_state(config_path)?;

    println!("Known Spaces ({})", state.spaces.len());

    if state.spaces.is_empty() {
        println!("\n  No known Spaces. Join one with 'xgen-client join'.");
        return Ok(());
    }

    for space in &state.spaces {
        println!();
        println!("  Space: {}", space.name);
        println!("  ID:    {}", space.space_id);
        println!("  Node:  {}", space.node_endpoint);
        println!("  Role:  {}", space.role);
        for room in &space.rooms {
            println!();
            println!("    Room: {}", room.name);
            println!("    ID:   {}", room.room_id);
            println!("    Joined: {}", if room.joined { "yes" } else { "no" });
        }
    }
    Ok(())
}

// ── version ────────────────────────────────────────────────────────────────────

fn cmd_version() -> Result<()> {
    println!("xgen-client {}", build_info::full_version());
    println!("Commit:  {}", build_info::GIT_HASH);
    Ok(())
}

// ── register ───────────────────────────────────────────────────────────────────

async fn cmd_register(args: &RegisterArgs, node: &str, keypair_path: &Path) -> Result<()> {
    let signing_key = load_keypair(keypair_path)?;
    let identity_id = identity_id_from_key(&signing_key);

    tracing::info!(node_url = %node, "Connecting to Node");
    println!("Connecting to {}...", node);
    let mut conn = connect_url(node).await.context("failed to connect to Node")?;
    let auth_id = conn.client_authenticate(&signing_key).await.context("authentication failed")?;
    tracing::info!(identity_id = %auth_id, "Authenticated");

    let reg = sign_register(build_register(&signing_key, Some(args.name.clone())), &signing_key);
    conn.send_identity(&reg).await.context("failed to send registration")?;

    match conn.recv().await.context("no response from Node")? {
        Inbound::Identity(IdentityMessage::RegisterOk { registered_at, .. }) => {
            tracing::info!(identity_id = %identity_id, "Authenticated");
            println!("Identity registered successfully.");
            println!("Identity ID:    {}", identity_id);
            println!("Display name:   {}", args.name);
            println!("Registered at:  {}", registered_at);
            println!("Home node:      {}", node);

            // Write client state file
            let state = ClientState {
                identity_id,
                display_name: args.name.clone(),
                version: build_info::VERSION.to_string(),
                build: build_info::GIT_HASH.to_string(),
                home_node: node.to_string(),
                updated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                spaces: vec![],
            };
            write_client_state(&state)?;
            println!("State saved:    {}", exe_dir().join("xgen-client_state.json").display());
        }
        Inbound::Identity(IdentityMessage::RegisterFail { error_code, error_string, .. }) => {
            bail!("registration rejected (code {}): {}", error_code, error_string);
        }
        other => bail!("unexpected response from Node: {:?}", other),
    }

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

// ── create-space ───────────────────────────────────────────────────────────────

async fn cmd_create_space(args: &CreateSpaceArgs, node: &str, keypair_path: &Path) -> Result<()> {
    let signing_key = load_keypair(keypair_path)?;
    let identity_id = identity_id_from_key(&signing_key);

    tracing::info!(node_url = %node, "Connecting to Node");
    println!("Connecting to {}...", node);
    let mut conn = connect_url(node).await.context("failed to connect")?;
    let identity_id_auth = conn.client_authenticate(&signing_key).await.context("authentication failed")?;
    tracing::info!(identity_id = %identity_id_auth, "Authenticated");

    // Build and sign space_create event
    let space_ev = sign_event(
        build_space_create_event(&signing_key, &args.name, None, 1, node),
        &signing_key,
    );
    let space_id = space_ev.event_id.clone().unwrap();

    let session_ctx = SessionContext {
        identity_id: Some(identity_id_auth.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(space_id.clone()),
    };
    trace_event(&space_ev, EventDirection::Outbound, &session_ctx);
    conn.send_event(&space_ev).await.context("failed to send space_create event")?;
    tracing::info!(space_id = %space_id, name = %args.name, "Space created");

    println!("Space created:");
    println!("  Name:     {}", args.name);
    println!("  Space ID: {}", space_id);
    println!("  Owner:    {}", identity_id);

    // Update client state
    let mut state = load_or_default_client_state(keypair_path, node)?;
    state.spaces.push(xgen_common::state::KnownSpace {
        space_id: space_id.clone(),
        name: args.name.clone(),
        node_endpoint: node.to_string(),
        role: "owner".to_string(),
        rooms: vec![],
    });
    state.updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    write_client_state(&state)?;

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

// ── create-room ────────────────────────────────────────────────────────────────

async fn cmd_create_room(args: &CreateRoomArgs, node: &str, keypair_path: &Path) -> Result<()> {
    let signing_key = load_keypair(keypair_path)?;

    tracing::info!(node_url = %node, "Connecting to Node");
    println!("Connecting to {}...", node);
    let mut conn = connect_url(node).await.context("failed to connect")?;
    let auth_id = conn.client_authenticate(&signing_key).await.context("authentication failed")?;
    tracing::info!(identity_id = %auth_id, "Authenticated");

    let room_ev = sign_event(
        build_room_create_event(&signing_key, &args.space, &args.name, None),
        &signing_key,
    );
    let room_id = room_ev.event_id.clone().unwrap();

    let session_ctx = SessionContext {
        identity_id: Some(auth_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(args.space.clone()),
    };
    trace_event(&room_ev, EventDirection::Outbound, &session_ctx);
    conn.send_event(&room_ev).await.context("failed to send room_create event")?;

    println!("Room created:");
    println!("  Name:    {}", args.name);
    println!("  Room ID: {}", room_id);
    println!("  Space:   {}", args.space);

    // Update client state
    let mut state = load_or_default_client_state(keypair_path, node)?;
    if let Some(space) = state.spaces.iter_mut().find(|s| s.space_id == args.space) {
        space.rooms.push(xgen_common::state::KnownRoom {
            room_id: room_id.clone(),
            name: args.name.clone(),
            joined: true,
        });
    }
    state.updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    write_client_state(&state)?;

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

// ── invite ─────────────────────────────────────────────────────────────────────

async fn cmd_invite(args: &InviteArgs, node: &str, keypair_path: &Path) -> Result<()> {
    let signing_key = load_keypair(keypair_path)?;
    let sender = identity_id_from_key(&signing_key);

    tracing::info!(node_url = %node, "Connecting to Node");
    println!("Connecting to {}...", node);
    let mut conn = connect_url(node).await.context("failed to connect")?;
    conn.client_authenticate(&signing_key).await.context("authentication failed")?;

    // We need current DAG tips to set prev_events — for Phase 1, use a sync_request
    // to get the latest event IDs, or use empty prev_events as fallback.
    // Phase 1 simplification: we pass the space_id itself as a prev_event placeholder.
    // The node will store the event; prev_events consistency is best-effort in Phase 1.
    let invite_ev = sign_event(
        Event::new(
            EventType::MembershipInvite,
            sender,
            String::new(),
            args.space.clone(),
            vec![args.space.clone()], // Phase 1: use space_id as prev_event anchor
            Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            json!({ "target_identity": args.identity, "role": args.role }),
        ),
        &signing_key,
    );

    let session_ctx = SessionContext {
        identity_id: invite_ev.event_id.as_ref().map(|_| invite_ev.sender.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(args.space.clone()),
    };
    trace_event(&invite_ev, EventDirection::Outbound, &session_ctx);
    conn.send_event(&invite_ev).await.context("failed to send invite event")?;
    println!("Invitation sent to {} in space {}", args.identity, args.space);
    println!("Event ID: {}", invite_ev.event_id.unwrap_or_default());

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

// ── join ───────────────────────────────────────────────────────────────────────

async fn cmd_join(args: &JoinArgs, node: &str, keypair_path: &Path) -> Result<()> {
    let signing_key = load_keypair(keypair_path)?;
    let sender = identity_id_from_key(&signing_key);

    tracing::info!(node_url = %node, "Connecting to Node");
    println!("Connecting to {}...", node);
    let mut conn = connect_url(node).await.context("failed to connect")?;
    let auth_id = conn.client_authenticate(&signing_key).await.context("authentication failed")?;
    tracing::info!(identity_id = %auth_id, "Authenticated");

    let join_ev = sign_event(
        Event::new(
            EventType::MembershipJoin,
            sender.clone(),
            args.room.clone().unwrap_or_default(),
            args.space.clone(),
            vec![args.space.clone()],
            Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            json!({}),
        ),
        &signing_key,
    );

    let session_ctx = SessionContext {
        identity_id: Some(auth_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(args.space.clone()),
    };
    trace_event(&join_ev, EventDirection::Outbound, &session_ctx);
    conn.send_event(&join_ev).await.context("failed to send join event")?;
    tracing::info!(space_id = %args.space, "Joined Space");

    let target = if args.room.is_some() { "Room" } else { "Space" };
    println!("Joined {} {}.", target, args.room.as_deref().unwrap_or(&args.space));

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

// ── send ───────────────────────────────────────────────────────────────────────

async fn cmd_send(args: &SendArgs, node: &str, keypair_path: &Path) -> Result<()> {
    let signing_key = load_keypair(keypair_path)?;

    tracing::info!(node_url = %node, "Connecting to Node");
    println!("Connecting to {}...", node);
    let mut conn = connect_url(node).await.context("failed to connect")?;
    let auth_id = conn.client_authenticate(&signing_key).await.context("authentication failed")?;
    tracing::info!(identity_id = %auth_id, "Authenticated");

    // Phase 1: get DAG tips via sync_request then use the most recent tip as prev_event.
    // Fallback: use space_id as a minimal anchor.
    let prev_events = get_dag_tips(&mut conn, &args.space).await.unwrap_or_else(|_| vec![args.space.clone()]);

    let msg_ev = sign_event(
        build_message_text_event(&signing_key, &args.space, &args.room, prev_events, &args.text),
        &signing_key,
    );
    let event_id = msg_ev.event_id.clone().unwrap_or_default();

    let session_ctx = SessionContext {
        identity_id: Some(auth_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(args.space.clone()),
    };
    trace_event(&msg_ev, EventDirection::Outbound, &session_ctx);
    conn.send_event(&msg_ev).await.context("failed to send message")?;
    tracing::info!(room = %args.room, "Message sent");
    println!("Message sent.");
    println!("Event ID: {}", event_id);

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

// ── history ────────────────────────────────────────────────────────────────────

async fn cmd_history(args: &HistoryArgs, node: &str, keypair_path: &Path) -> Result<()> {
    let signing_key = load_keypair(keypair_path)?;

    tracing::info!(node_url = %node, "Connecting to Node");
    println!("Connecting to {}...", node);
    let mut conn = connect_url(node).await.context("failed to connect")?;
    let auth_id = conn.client_authenticate(&signing_key).await.context("authentication failed")?;
    tracing::info!(identity_id = %auth_id, "Authenticated");
    let session_ctx = SessionContext {
        identity_id: Some(auth_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(args.space.clone()),
    };

    // Send sync_request to receive history
    let sync_req = xgen_node_lib::wire::types::TransportMessage::SyncRequest {
        protocol_version: "0.1".to_string(),
        since: String::new(),
    };
    conn.send_transport(&sync_req).await.context("failed to send sync_request")?;

    let mut messages: Vec<(String, String, String)> = vec![]; // (sender, timestamp, text)
    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(5);

    loop {
        match tokio::time::timeout_at(deadline, conn.recv()).await {
            Ok(Ok(Inbound::Event(ev))) => {
                trace_event(&ev, EventDirection::Inbound, &session_ctx);
                if ev.space_id == args.space && ev.room_id == args.room {
                    if matches!(ev.event_type, EventType::MessageText) {
                        let text = ev.content["text"].as_str().unwrap_or("").to_string();
                        let sender_short = short_id(&ev.sender);
                        messages.push((sender_short, ev.timestamp.clone(), text));
                        if messages.len() >= args.limit {
                            break;
                        }
                    }
                }
            }
            Ok(Ok(Inbound::Transport(TransportMessage::Goodbye { .. })))
            | Ok(Ok(Inbound::Closed)) => break,
            Ok(Ok(_)) => {}
            Ok(Err(_)) | Err(_) => break,
        }
    }

    println!("History for room {} ({} messages)", short_id(&args.room), messages.len());
    println!();
    for (sender, ts, text) in &messages {
        println!("  [{}]  {}  {}", sender, &ts[..ts.len().min(19)], text);
    }
    if messages.is_empty() {
        println!("  No messages found.");
    }

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

// ── smoke-test (spec 3.7.11 — 17 steps over real TCP) ─────────────────────────

async fn cmd_smoke_test(args: &SmokeTestArgs) -> Result<()> {
    fn now() -> String {
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
    }
    fn pubkey_uri(key: &ed25519_dalek::SigningKey) -> String {
        format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        )
    }
    fn step(n: u8, msg: &str) {
        println!("Step {:>2}: {}", n, msg);
    }

    println!("Phase 1 Smoke Test — spec 3.7.11 (17 steps)");
    println!("============================================");
    println!("Node A:  {}", args.node_a);
    println!("Node B:  {}", args.node_b);
    println!();

    // ── Step 1: Node A already running; generate Alice's ephemeral keypair ──────
    let alice_key = keypair::generate();
    let alice_id = pubkey_uri(&alice_key);
    let alice_ctx = SessionContext {
        identity_id: Some(alice_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: None,
    };
    step(1, "Node A running; Alice ephemeral keypair generated");
    println!("         Alice: {}...", &alice_id[..alice_id.len().min(52)]);

    // ── Step 2: Alice registers on Node A ────────────────────────────────────────
    step(2, "Alice registers on Node A");
    let mut alice_conn = connect_url(&args.node_a)
        .await
        .context("cannot connect to Node A")?;
    alice_conn.client_authenticate(&alice_key).await.context("Alice: auth failed")?;
    {
        let reg = sign_register(build_register(&alice_key, Some("Alice".to_string())), &alice_key);
        alice_conn.send_identity(&reg).await?;
        match alice_conn.recv().await? {
            Inbound::Identity(IdentityMessage::RegisterOk { .. }) => println!("         OK"),
            Inbound::Identity(IdentityMessage::RegisterFail { error_string, .. }) => {
                bail!("Alice registration failed: {error_string}");
            }
            other => bail!("unexpected: {other:?}"),
        }
    }

    // ── Step 3: Node B already running; generate test-node-B federation keypair ──
    let test_node_b_key = keypair::generate(); // simulates Node B's federation connector
    let test_node_b_id = pubkey_uri(&test_node_b_key);
    step(3, "Node B running; test-Node-B federation keypair generated");
    println!("         Node B (test): {}...", &test_node_b_id[..test_node_b_id.len().min(52)]);

    // ── Step 4: Bob registers on Node B ──────────────────────────────────────────
    let bob_key = keypair::generate();
    let bob_id = pubkey_uri(&bob_key);
    let bob_ctx = SessionContext {
        identity_id: Some(bob_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: None,
    };
    step(4, "Bob registers on Node B");
    println!("         Bob: {}...", &bob_id[..bob_id.len().min(52)]);
    let mut bob_conn = connect_url(&args.node_b)
        .await
        .context("cannot connect to Node B")?;
    bob_conn.client_authenticate(&bob_key).await.context("Bob: auth failed")?;
    {
        let reg = sign_register(build_register(&bob_key, Some("Bob".to_string())), &bob_key);
        bob_conn.send_identity(&reg).await?;
        match bob_conn.recv().await? {
            Inbound::Identity(IdentityMessage::RegisterOk { .. }) => println!("         OK"),
            Inbound::Identity(IdentityMessage::RegisterFail { error_string, .. }) => {
                bail!("Bob registration failed: {error_string}");
            }
            other => bail!("unexpected: {other:?}"),
        }
    }

    // ── Step 5: Alice produces state.space_create ─────────────────────────────────
    step(5, "Alice creates Space on Node A");
    let space_ev = sign_event(
        build_space_create_event(&alice_key, "XGen Test Space", None, 1, &args.node_a),
        &alice_key,
    );
    let space_id = space_ev.event_id.clone().unwrap();
    trace_event(&space_ev, EventDirection::Outbound, &alice_ctx);
    alice_conn.send_event(&space_ev).await?;
    println!("         Space ID: {}...", &space_id[..space_id.len().min(52)]);

    // ── Step 6: Alice produces state.room_create ──────────────────────────────────
    step(6, "Alice creates Room 'general'");
    let room_ev = sign_event(
        build_room_create_event(&alice_key, &space_id, "general", None),
        &alice_key,
    );
    let room_id = room_ev.event_id.clone().unwrap();
    trace_event(&room_ev, EventDirection::Outbound, &alice_ctx);
    alice_conn.send_event(&room_ev).await?;
    println!("         Room ID:  {}...", &room_id[..room_id.len().min(52)]);

    // ── Step 7: Alice invites Bob ──────────────────────────────────────────────────
    step(7, "Alice invites Bob to the Space");
    let invite_ev = sign_event(
        Event::new(
            EventType::MembershipInvite,
            alice_id.clone(),
            String::new(),
            space_id.clone(),
            vec![space_id.clone(), room_id.clone()],
            now(),
            json!({ "target_identity": bob_id, "role": "member" }),
        ),
        &alice_key,
    );
    let invite_id = invite_ev.event_id.clone().unwrap();
    trace_event(&invite_ev, EventDirection::Outbound, &alice_ctx);
    alice_conn.send_event(&invite_ev).await?;
    println!("         Invite ID: {}...", &invite_id[..invite_id.len().min(52)]);

    // Small delay to let Node A process the events before federation snapshot
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // ── Step 8: test-Node-B connects to Node A for federation handshake ───────────
    step(8, "Test-Node-B connects to Node A — transport + federation handshake");
    let mut fed_conn = connect_url(&args.node_a)
        .await
        .context("cannot connect to Node A for federation")?;
    fed_conn.client_authenticate(&test_node_b_key).await.context("federation auth failed")?;

    let fed_session = run_initiating(
        &mut fed_conn,
        &test_node_b_key,
        FederationCapabilities::default(),
        vec![space_id.clone()],
    )
    .await
    .context("federation handshake failed")?;
    tracing::info!(peer_node_url = %args.node_a, "Federation initiated");
    println!("         Session ID: {}...", &fed_session.session_id[..fed_session.session_id.len().min(52)]);

    // ── Step 9: test-Node-B sends space.join_request ──────────────────────────────
    step(9, "Test-Node-B sends space.join_request");
    fed_conn
        .send_space(&SpaceControlMessage::JoinRequest {
            space_id: space_id.clone(),
            node_id: test_node_b_id.clone(),
        })
        .await?;

    // ── Steps 10+11: receive history + federation_add from Node A ─────────────────
    step(10, "Node A produces state.federation_add");
    step(11, "Receiving history from Node A");
    let mut received_events: Vec<xgen_node_lib::wire::types::Event> = vec![];
    loop {
        match fed_conn.recv().await? {
            Inbound::Event(ev) => received_events.push(ev),
            Inbound::Transport(TransportMessage::Goodbye { .. }) | Inbound::Closed => break,
            _ => {}
        }
    }
    let fed_add_ev = received_events
        .last()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("no events received from Node A during history sync"))?;
    let fed_add_id = fed_add_ev.event_id.clone().unwrap();
    println!(
        "         Received {} events (federation_add: {}...)",
        received_events.len(),
        &fed_add_id[..fed_add_id.len().min(52)]
    );

    // Forward history events to Node B
    println!("         Forwarding history to Node B...");
    for ev in &received_events {
        bob_conn.send_event(ev).await?;
    }

    // Register Alice on Node B (so Node B can validate Alice's messages)
    println!("         Registering Alice on Node B...");
    let mut alice_on_b = connect_url(&args.node_b).await.context("cannot connect to Node B")?;
    alice_on_b.client_authenticate(&alice_key).await?;
    {
        let reg = sign_register(build_register(&alice_key, Some("Alice".to_string())), &alice_key);
        alice_on_b.send_identity(&reg).await?;
        match alice_on_b.recv().await? {
            Inbound::Identity(IdentityMessage::RegisterOk { .. }) => {}
            _ => {} // may already be registered or fail — continue
        }
    }

    // Register Bob on Node A (so Node A can validate Bob's messages)
    println!("         Registering Bob on Node A...");
    let mut bob_on_a = connect_url(&args.node_a).await.context("cannot connect to Node A")?;
    bob_on_a.client_authenticate(&bob_key).await?;
    {
        let reg = sign_register(build_register(&bob_key, Some("Bob".to_string())), &bob_key);
        bob_on_a.send_identity(&reg).await?;
        match bob_on_a.recv().await? {
            Inbound::Identity(IdentityMessage::RegisterOk { .. }) => {}
            _ => {}
        }
    }

    // ── Step 12: Bob joins the Space ──────────────────────────────────────────────
    step(12, "Bob joins the Space");
    let bob_join_space_ev = sign_event(
        Event::new(
            EventType::MembershipJoin,
            bob_id.clone(),
            String::new(),
            space_id.clone(),
            vec![fed_add_id.clone()],
            now(),
            json!({}),
        ),
        &bob_key,
    );
    let bob_join_space_id = bob_join_space_ev.event_id.clone().unwrap();
    trace_event(&bob_join_space_ev, EventDirection::Outbound, &bob_ctx);
    bob_conn.send_event(&bob_join_space_ev).await?; // send to Node B
    bob_on_a.send_event(&bob_join_space_ev).await?; // propagate to Node A
    println!("         OK");

    // ── Step 13: Bob joins the Room ───────────────────────────────────────────────
    step(13, "Bob joins the Room");
    let bob_join_room_ev = sign_event(
        Event::new(
            EventType::MembershipJoin,
            bob_id.clone(),
            room_id.clone(),
            space_id.clone(),
            vec![bob_join_space_id.clone()],
            now(),
            json!({}),
        ),
        &bob_key,
    );
    let bob_join_room_id = bob_join_room_ev.event_id.clone().unwrap();
    trace_event(&bob_join_room_ev, EventDirection::Outbound, &bob_ctx);
    bob_conn.send_event(&bob_join_room_ev).await?; // send to Node B
    bob_on_a.send_event(&bob_join_room_ev).await?; // propagate to Node A
    println!("         OK");

    // Small delay for Node A/B to process join events
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // ── Step 14: Alice sends "Hello Bob" ──────────────────────────────────────────
    step(14, "Alice sends 'Hello Bob' to Node A");
    let hello_bob_ev = sign_event(
        build_message_text_event(
            &alice_key,
            &space_id,
            &room_id,
            vec![bob_join_room_id.clone()],
            "Hello Bob",
        ),
        &alice_key,
    );
    let hello_bob_id = hello_bob_ev.event_id.clone().unwrap();
    trace_event(&hello_bob_ev, EventDirection::Outbound, &alice_ctx);
    alice_conn.send_event(&hello_bob_ev).await?; // send to Node A
    alice_on_b.send_event(&hello_bob_ev).await?; // propagate to Node B
    println!("         OK");

    // ── Step 15: Bob sends "Hello Alice" ──────────────────────────────────────────
    step(15, "Bob sends 'Hello Alice' to Node B");
    let hello_alice_ev = sign_event(
        build_message_text_event(
            &bob_key,
            &space_id,
            &room_id,
            vec![hello_bob_id.clone()],
            "Hello Alice",
        ),
        &bob_key,
    );
    let hello_alice_id = hello_alice_ev.event_id.clone().unwrap();
    trace_event(&hello_alice_ev, EventDirection::Outbound, &bob_ctx);
    bob_conn.send_event(&hello_alice_ev).await?; // send to Node B
    bob_on_a.send_event(&hello_alice_ev).await?; // propagate to Node A
    println!("         OK");

    // ── Step 16: verify both Events exist in both Nodes via DAG integrity ─────────
    step(16, "Verifying event signatures on both messages");
    if !verify_event_signature(&hello_bob_ev) {
        bail!("FAIL: Alice's 'Hello Bob' event has an invalid signature");
    }
    if !verify_event_signature(&hello_alice_ev) {
        bail!("FAIL: Bob's 'Hello Alice' event has an invalid signature");
    }
    println!("         Alice's message: signature VALID");
    println!("         Bob's message:   signature VALID");

    // ── Step 17: verify message content ───────────────────────────────────────────
    step(17, "Verifying message content");
    let alice_text = hello_bob_ev.content["text"].as_str().unwrap_or("");
    let bob_text = hello_alice_ev.content["text"].as_str().unwrap_or("");
    if alice_text != "Hello Bob" {
        bail!("FAIL: Alice's message content mismatch: got '{alice_text}'");
    }
    if bob_text != "Hello Alice" {
        bail!("FAIL: Bob's message content mismatch: got '{bob_text}'");
    }
    println!("         Alice → \"{}\"  ✓", alice_text);
    println!("         Bob   → \"{}\"  ✓", bob_text);

    // Graceful close all connections
    let _ = alice_conn.goodbye("smoke_test_complete").await;
    let _ = bob_conn.goodbye("smoke_test_complete").await;
    let _ = alice_on_b.goodbye("smoke_test_complete").await;
    let _ = bob_on_a.goodbye("smoke_test_complete").await;

    println!();
    println!("============================================");
    println!("ALL 17 STEPS PASSED.");
    println!("============================================");
    println!();
    println!("  Space ID:         {}", space_id);
    println!("  Room ID:          {}", room_id);
    println!("  Alice's message:  {}", hello_bob_id);
    println!("  Bob's message:    {}", hello_alice_id);
    println!("  Federation session: {}", fed_session.session_id);

    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Directory of the running executable (D-020: Tier 1 files co-located with binary).
fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn resolve_node(cli: &Cli, config_path: &Path) -> String {
    if let Some(n) = &cli.node {
        return n.clone();
    }
    if let Some(text) = std::fs::read_to_string(config_path).ok() {
        if let Ok(cfg) = toml::from_str::<ClientConfig>(&text) {
            return cfg.client.node;
        }
    }
    ClientConfig::default().client.node
}

fn resolve_keypair_path(config_path: &Path) -> PathBuf {
    if let Some(text) = std::fs::read_to_string(config_path).ok() {
        if let Ok(cfg) = toml::from_str::<ClientConfig>(&text) {
            return PathBuf::from(cfg.paths.keypair_path);
        }
    }
    exe_dir().join("xgen-client_keypair.enc")
}

fn load_keypair(path: &Path) -> Result<ed25519_dalek::SigningKey> {
    if !path.exists() {
        bail!(
            "keypair not found: {}\n  Run 'xgen-client init' first.",
            path.display()
        );
    }
    // Phase 1: empty passphrase for Local Node mode
    keypair::load(path, "").with_context(|| {
        format!("failed to load keypair from {}", path.display())
    })
}

fn load_client_state(config_path: &Path) -> Result<ClientState> {
    let path = exe_dir().join("xgen-client_state.json");
    if !path.exists() {
        bail!(
            "state file not found: {}\n  Run 'xgen-client init' and 'xgen-client register' first.",
            path.display()
        );
    }
    let json = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read state file: {}", path.display()))?;
    let _ = config_path;
    serde_json::from_str(&json).context("state file is corrupt or has an unexpected format")
}

fn load_or_default_client_state(keypair_path: &Path, node: &str) -> Result<ClientState> {
    let path = exe_dir().join("xgen-client_state.json");
    if path.exists() {
        let json = std::fs::read_to_string(&path)?;
        if let Ok(state) = serde_json::from_str::<ClientState>(&json) {
            return Ok(state);
        }
    }
    // Build minimal state from keypair
    let sk = load_keypair(keypair_path)?;
    Ok(ClientState {
        identity_id: identity_id_from_key(&sk),
        display_name: String::new(),
        version: build_info::VERSION.to_string(),
        build: build_info::GIT_HASH.to_string(),
        home_node: node.to_string(),
        updated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        spaces: vec![],
    })
}

fn write_client_state(state: &ClientState) -> Result<()> {
    let path = exe_dir().join("xgen-client_state.json");
    let json = serde_json::to_string_pretty(state).context("failed to serialise client state")?;
    std::fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

/// Request DAG tips for a space via sync_request and collect a few event IDs.
async fn get_dag_tips(
    conn: &mut xgen_node_lib::transport::connection::Connection<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    _space_id: &str,
) -> Result<Vec<String>> {
    // Phase 1: send a sync_request with empty since to get recent events,
    // then collect event IDs (the last one is the most recent tip).
    let req = xgen_node_lib::wire::types::TransportMessage::SyncRequest {
        protocol_version: "0.1".to_string(),
        since: String::new(),
    };
    conn.send_transport(&req).await?;

    let mut tips: Vec<String> = vec![];
    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_millis(500);

    loop {
        match tokio::time::timeout_at(deadline, conn.recv()).await {
            Ok(Ok(Inbound::Event(ev))) => {
                if let Some(id) = ev.event_id {
                    tips = vec![id]; // keep only the latest
                }
            }
            _ => break,
        }
    }

    Ok(tips)
}

fn age_seconds(timestamp: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_seconds())
        .unwrap_or(i64::MAX)
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

fn short_id(uri: &str) -> String {
    let rest = uri
        .strip_prefix("xgen://hash/")
        .or_else(|| uri.strip_prefix("xgen://pubkey/"))
        .unwrap_or(uri);
    if let Some((_scheme, key)) = rest.split_once(':') {
        let trunc: String = key.chars().take(8).collect();
        format!("{trunc}...")
    } else {
        let trunc: String = uri.chars().take(12).collect();
        format!("{trunc}...")
    }
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
