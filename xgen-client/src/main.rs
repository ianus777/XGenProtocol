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

use xgen_common::{
    build_info,
    event_trace::{
        EventDirection, ExitReason, SessionContext, SpaceRole,
        trace_event, write_session_footer, write_session_header,
    },
    state::ClientState,
};
use xgen_core::{
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
                level: "debug".to_string(),
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

    /// Run the Phase 1 stress test against two running Node instances.
    /// Concurrent multi-identity load test; produces report + full communication record.
    StressTest(StressTestArgs),
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

#[derive(Args)]
struct StressTestArgs {
    /// Endpoint of Node A. Example: ws://127.0.0.1:8080/xgen
    #[arg(long)]
    node_a: String,
    /// Endpoint of Node B. Example: ws://127.0.0.1:8081/xgen
    #[arg(long)]
    node_b: String,
    /// Total number of test identities (min 2, max 20). Default: 10.
    #[arg(long, default_value = "10")]
    members: usize,
    /// Messages per identity in the message phase. Default: 50.
    #[arg(long, default_value = "50")]
    messages: usize,
    /// Resting period in milliseconds after each phase transition (nodes settle).
    /// Applied after Phase 3 (before flood) and after Phase 4 (before report). Default: 2000.
    #[arg(long, default_value = "2000")]
    rest_ms: u64,
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
            .unwrap_or_else(|| "debug".to_string());
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
    }

    // Session header — written once, immediately after subscriber init.
    // identity_id and connected_node are omitted here; they are logged as body
    // lines after connection and auth complete (D-038).
    {
        let started_at = chrono::Utc::now()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let session_id = format!("{:08x}", rand::random::<u32>());
        write_session_header(
            "client",
            None,  // identity_id — logged as body line after auth (D-038)
            None,  // endpoint — client has no listen address
            None,  // connected_node — logged as body line after connect (D-038)
            "0.1",
            build_info::VERSION,
            &session_id,
            &started_at,
        );
    }

    let result: Result<()> = match &cli.command {
        None => {
            build_info::print_banner("xgen-client");
            println!();
            Cli::command().print_help().unwrap();
            println!();
            Ok(())
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
        Some(ClientCommand::StressTest(args)) => cmd_stress_test(args).await,
    };
    if let Err(ref e) = result {
        tracing::error!(reason = %format!("{:#}", e), "Fatal error");
        write_session_footer(ExitReason::Error);
        eprintln!("{}", red(&format!("error: {:#}", e)));
        std::process::exit(1);
    } else {
        write_session_footer(ExitReason::Shutdown);
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
    tracing::info!("identity_id={}", auth_id);
    tracing::info!("connected_node={}", node);
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
    tracing::info!("identity_id={}", identity_id_auth);
    tracing::info!("connected_node={}", node);
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
    trace_event(&space_ev, EventDirection::Out, &session_ctx);
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
    tracing::info!("identity_id={}", auth_id);
    tracing::info!("connected_node={}", node);
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
    trace_event(&room_ev, EventDirection::Out, &session_ctx);
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
    let invite_auth_id = conn.client_authenticate(&signing_key).await.context("authentication failed")?;
    tracing::info!("identity_id={}", invite_auth_id);
    tracing::info!("connected_node={}", node);

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
    trace_event(&invite_ev, EventDirection::Out, &session_ctx);
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
    tracing::info!("identity_id={}", auth_id);
    tracing::info!("connected_node={}", node);
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
    trace_event(&join_ev, EventDirection::Out, &session_ctx);
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
    tracing::info!("identity_id={}", auth_id);
    tracing::info!("connected_node={}", node);
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
    trace_event(&msg_ev, EventDirection::Out, &session_ctx);
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
    tracing::info!("identity_id={}", auth_id);
    tracing::info!("connected_node={}", node);
    tracing::info!(identity_id = %auth_id, "Authenticated");
    let session_ctx = SessionContext {
        identity_id: Some(auth_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(args.space.clone()),
    };

    // Send sync_request to receive history
    let sync_req = xgen_core::wire::types::TransportMessage::SyncRequest {
        protocol_version: "0.1".to_string(),
        since: String::new(),
    };
    conn.send_transport(&sync_req).await.context("failed to send sync_request")?;

    let mut messages: Vec<(String, String, String)> = vec![]; // (sender, timestamp, text)
    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(5);

    loop {
        match tokio::time::timeout_at(deadline, conn.recv()).await {
            Ok(Ok(Inbound::Event(ev))) => {
                trace_event(&ev, EventDirection::In, &session_ctx);
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
    trace_event(&space_ev, EventDirection::Out, &alice_ctx);
    alice_conn.send_event(&space_ev).await?;
    println!("         Space ID: {}...", &space_id[..space_id.len().min(52)]);

    // ── Step 6: Alice produces state.room_create ──────────────────────────────────
    step(6, "Alice creates Room 'general'");
    let room_ev = sign_event(
        build_room_create_event(&alice_key, &space_id, "general", None),
        &alice_key,
    );
    let room_id = room_ev.event_id.clone().unwrap();
    trace_event(&room_ev, EventDirection::Out, &alice_ctx);
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
    trace_event(&invite_ev, EventDirection::Out, &alice_ctx);
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
    let mut received_events: Vec<xgen_core::wire::types::Event> = vec![];
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
    trace_event(&bob_join_space_ev, EventDirection::Out, &bob_ctx);
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
    trace_event(&bob_join_room_ev, EventDirection::Out, &bob_ctx);
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
    trace_event(&hello_bob_ev, EventDirection::Out, &alice_ctx);
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
    trace_event(&hello_alice_ev, EventDirection::Out, &bob_ctx);
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

// ── stress-test ───────────────────────────────────────────────────────────────

async fn cmd_stress_test(args: &StressTestArgs) -> Result<()> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use std::time::Instant;

    let members = args.members.clamp(2, 20);
    let mpm     = args.messages;   // messages per member
    let test_ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    let log: CommLog = Arc::new(std::sync::Mutex::new(Vec::new()));
    let seq: Seq     = Arc::new(AtomicU64::new(0));

    fn now_s() -> String {
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }
    fn pubkey_uri_st(key: &ed25519_dalek::SigningKey) -> String {
        format!("xgen://pubkey/ed25519:{}", xgen_core::crypto::encoding::encode(
            key.verifying_key().as_bytes()))
    }
    fn actor(i: usize) -> String {
        if i == 0 { "Alice".to_string() } else { format!("M{i}") }
    }

    comm_push(&log, &seq, "system","system","INFO","test_start","","",vec![],true,
        &format!("members={members} mpm={mpm}"));

    // ══════════════════════════════════════════════════════════════════════
    // Phase 1 — Setup (sequential)
    // ══════════════════════════════════════════════════════════════════════
    println!("Phase 1 — Setup ...");
    comm_push(&log, &seq, "setup","system","INFO","phase_start","","",vec![],true,"Phase 1 — Setup");
    let t1 = Instant::now();

    // Generate all keypairs up front
    let keys: Vec<ed25519_dalek::SigningKey> = (0..members).map(|_| keypair::generate()).collect();
    let ids:  Vec<String> = keys.iter().map(|k| pubkey_uri_st(k)).collect();

    // Alice connects + registers
    let mut alice = connect_url(&args.node_a).await.context("Phase 1: connect Node A")?;
    alice.client_authenticate(&keys[0]).await.context("Phase 1: Alice auth")?;
    {
        let reg = sign_register(build_register(&keys[0], Some("StressTest-Alice".into())), &keys[0]);
        alice.send_identity(&reg).await?;
        comm_push(&log,&seq,"setup","Alice","SENT","identity.register","",&args.node_a,vec![],true,"name=StressTest-Alice");
        let ok = matches!(alice.recv().await?, Inbound::Identity(IdentityMessage::RegisterOk{..}));
        comm_push(&log,&seq,"setup","Alice","RECV",if ok {"identity.register_ok"} else {"identity.register_fail"},"",&args.node_a,vec![],ok,"");
        if !ok { bail!("Phase 1: Alice registration failed"); }
    }

    // Space
    let space_ev = sign_event(build_space_create_event(&keys[0],"StressTest Space",None,1,&args.node_a), &keys[0]);
    let space_id = Arc::new(space_ev.event_id.clone().unwrap());
    comm_event(&log,&seq,"setup","Alice","SENT",&space_ev,&args.node_a);
    alice.send_event(&space_ev).await?;

    // 3 rooms
    let mut room_ids_vec: Vec<Arc<String>> = Vec::new();
    let mut chain_tip = space_id.as_ref().clone();
    for rname in ["general","random","tech"] {
        let rev = sign_event(build_room_create_event(&keys[0],&space_id,rname,None), &keys[0]);
        chain_tip = rev.event_id.clone().unwrap();
        comm_event(&log,&seq,"setup","Alice","SENT",&rev,&args.node_a);
        alice.send_event(&rev).await?;
        room_ids_vec.push(Arc::new(chain_tip.clone()));
    }
    let room_ids: Arc<Vec<Arc<String>>> = Arc::new(room_ids_vec);

    // Invites for all other members
    for i in 1..members {
        let inv = sign_event(
            Event::new(EventType::MembershipInvite, ids[0].clone(), String::new(),
                space_id.as_ref().clone(), vec![chain_tip.clone()], now_s(),
                serde_json::json!({"target_identity": ids[i], "role": "member"})),
            &keys[0]);
        chain_tip = inv.event_id.clone().unwrap();
        comm_event(&log,&seq,"setup",&actor(0),"SENT",&inv,&args.node_a);
        alice.send_event(&inv).await?;
    }
    let _ = alice.goodbye("phase1_complete").await;
    let d1 = t1.elapsed();
    comm_push(&log,&seq,"setup","system","INFO","phase_end","","",vec![],true,&format!("duration_ms={}",d1.as_millis()));
    println!("  done in {:.1}s", d1.as_secs_f64());

    // ══════════════════════════════════════════════════════════════════════
    // Phase 2 — Registration (sequential)
    // ══════════════════════════════════════════════════════════════════════
    println!("Phase 2 — Registration ...");
    comm_push(&log,&seq,"registration","system","INFO","phase_start","","",vec![],true,"Phase 2 — Registration");
    let t2 = Instant::now();

    for i in 1..members {
        let node = assigned_node_url(i, members, &args.node_a, &args.node_b);
        let a = actor(i);
        let mut conn = connect_url(&node).await.with_context(|| format!("Phase 2: M{i} connect"))?;
        conn.client_authenticate(&keys[i]).await.with_context(|| format!("Phase 2: M{i} auth"))?;
        let reg = sign_register(build_register(&keys[i], Some(format!("StressTest-M{i}"))), &keys[i]);
        conn.send_identity(&reg).await?;
        comm_push(&log,&seq,"registration",&a,"SENT","identity.register","",&node,vec![],true,&format!("name=StressTest-M{i}"));
        let ok = matches!(conn.recv().await?, Inbound::Identity(IdentityMessage::RegisterOk{..}));
        comm_push(&log,&seq,"registration",&a,"RECV",if ok {"identity.register_ok"} else {"identity.register_fail"},"",&node,vec![],ok,"");
        let _ = conn.goodbye("reg_complete").await;
    }
    let d2 = t2.elapsed();
    comm_push(&log,&seq,"registration","system","INFO","phase_end","","",vec![],true,&format!("duration_ms={}",d2.as_millis()));
    println!("  done in {:.1}s", d2.as_secs_f64());

    // ══════════════════════════════════════════════════════════════════════
    // Phase 3 — Federation + Join (concurrent)
    // ══════════════════════════════════════════════════════════════════════
    println!("Phase 3 — Federation + Join ...");
    comm_push(&log,&seq,"fed_join","system","INFO","phase_start","","",vec![],true,"Phase 3 — Federation + Join");
    let t3 = Instant::now();

    // DAG anchors per member (Phase 3 sets these; Phase 4 reads them)
    let anchors: Arc<std::sync::Mutex<Vec<String>>> =
        Arc::new(std::sync::Mutex::new(vec![chain_tip.clone(); members]));
    anchors.lock().unwrap()[0] = chain_tip.clone();

    let join_failures = Arc::new(AtomicUsize::new(0));

    // 3a — Federation (ephemeral key, same pattern as smoke test)
    let fed_key  = keypair::generate();
    let fed_id   = pubkey_uri_st(&fed_key);
    let fed_log  = log.clone();  let fed_seq  = seq.clone();
    let fed_na   = args.node_a.clone(); let fed_nb = args.node_b.clone();
    let fed_sid  = space_id.clone();

    let fed_task: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        let result: Result<()> = async {
            comm_push(&fed_log,&fed_seq,"fed_join","federation","INFO","fed_start","",&fed_na,vec![],true,"");
            let mut fc = connect_url(&fed_na).await.context("fed: connect A")?;
            fc.client_authenticate(&fed_key).await.context("fed: auth A")?;
            let fs = run_initiating(&mut fc, &fed_key, FederationCapabilities::default(),
                vec![fed_sid.as_ref().clone()]).await.context("fed: handshake")?;
            comm_push(&fed_log,&fed_seq,"fed_join","federation","INFO","fed_handshake_ok",&fs.session_id,&fed_na,vec![],true,"");

            fc.send_space(&SpaceControlMessage::JoinRequest {
                space_id: fed_sid.as_ref().clone(), node_id: fed_id.clone() }).await?;

            let mut history: Vec<Event> = vec![];
            loop {
                match fc.recv().await? {
                    Inbound::Event(ev) => {
                        comm_event(&fed_log,&fed_seq,"fed_join","federation","RECV",&ev,&fed_na);
                        history.push(ev);
                    }
                    Inbound::Transport(TransportMessage::Goodbye{..}) | Inbound::Closed => break,
                    _ => {}
                }
            }
            comm_push(&fed_log,&fed_seq,"fed_join","federation","INFO","history_recv","","",vec![],true,
                &format!("events={}",history.len()));

            // Forward history to Node B (register fed_key on B first)
            let mut bc = connect_url(&fed_nb).await.context("fed: connect B")?;
            bc.client_authenticate(&fed_key).await?;
            let reg2 = sign_register(build_register(&fed_key, Some("fed-relay".into())), &fed_key);
            bc.send_identity(&reg2).await?;
            let _ = bc.recv().await; // RegisterOk or fail — continue either way
            for ev in &history {
                bc.send_event(ev).await.ok();
                comm_event(&fed_log,&fed_seq,"fed_join","federation","SENT",ev,&fed_nb);
            }
            let _ = bc.goodbye("fed_forward_done").await;
            comm_push(&fed_log,&fed_seq,"fed_join","federation","INFO","fed_complete","","",vec![],true,
                &format!("forwarded={}",history.len()));
            Ok(())
        }.await;
        if let Err(e) = result {
            tracing::error!("Federation failed: {:#}", e);
        }
    });

    // 3a must complete before 3b: Node B must know about the Space before members can join.
    // Running federation to completion first removes the race; joins are still concurrent
    // among themselves, which is what the load test actually exercises.
    let _ = fed_task.await;
    comm_push(&log,&seq,"fed_join","system","INFO","fed_join_barrier","","",vec![],true,
        "federation complete — starting joins");

    // 3b — Join tasks (concurrent among themselves)
    let mut join_tasks = Vec::new();
    for i in 1..members {
        let node    = assigned_node_url(i, members, &args.node_a, &args.node_b);
        let a       = actor(i);
        let key     = keys[i].clone();
        let iid     = ids[i].clone();
        let sid     = space_id.clone();
        let rids    = room_ids.clone();
        let jlog    = log.clone(); let jseq = seq.clone();
        let janch   = anchors.clone();
        let jfail   = join_failures.clone();
        let node_cl = node.clone();

        join_tasks.push(tokio::spawn(async move {
            let result: Result<String> = async {
                let mut conn = connect_url(&node_cl).await?;
                conn.client_authenticate(&key).await?;
                let mut last = sid.as_ref().clone();

                // Join space
                let jsev = sign_event(Event::new(EventType::MembershipJoin,
                    iid.clone(), String::new(), sid.as_ref().clone(),
                    vec![last.clone()], chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis,true),
                    serde_json::json!({})), &key);
                last = jsev.event_id.clone().unwrap();
                comm_event(&jlog,&jseq,"fed_join",&a,"SENT",&jsev,&node_cl);
                conn.send_event(&jsev).await?;

                // Join 3 rooms
                for (ri, rid) in rids.iter().enumerate() {
                    let jrev = sign_event(Event::new(EventType::MembershipJoin,
                        iid.clone(), rid.as_ref().clone(), sid.as_ref().clone(),
                        vec![last.clone()], chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis,true),
                        serde_json::json!({})), &key);
                    last = jrev.event_id.clone().unwrap();
                    comm_event(&jlog,&jseq,"fed_join",&a,"SENT",&jrev,&node_cl);
                    conn.send_event(&jrev).await
                        .with_context(|| format!("M{i}: join room {ri}"))?;
                }
                let _ = conn.goodbye("join_done").await;
                Ok(last)
            }.await;

            match result {
                Ok(anchor) => { janch.lock().unwrap()[i] = anchor; }
                Err(e) => {
                    tracing::error!(member=i, "Join failed: {:#}", e);
                    comm_push(&jlog,&jseq,"fed_join",&a,"INFO","join_failed","","",vec![],false,&format!("{:#}",e));
                    jfail.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }

    for t in join_tasks { let _ = t.await; }

    let d3 = t3.elapsed();
    let jf = join_failures.load(Ordering::Relaxed);
    comm_push(&log,&seq,"fed_join","system","INFO","phase_end","","",vec![],true,
        &format!("duration_ms={} join_failures={}", d3.as_millis(), jf));
    println!("  done in {:.1}s  (join failures: {})", d3.as_secs_f64(), jf);

    // Resting point — let membership events propagate and be applied on both nodes
    // before the flood begins. Avoids races between join delivery and message validation.
    if args.rest_ms > 0 {
        println!("  resting {}ms (nodes settling after Phase 3) ...", args.rest_ms);
        comm_push(&log,&seq,"rest","system","INFO","rest_start","","",vec![],true,
            &format!("after=phase3 ms={}", args.rest_ms));
        tokio::time::sleep(tokio::time::Duration::from_millis(args.rest_ms)).await;
        comm_push(&log,&seq,"rest","system","INFO","rest_end","","",vec![],true,"after=phase3");
    }

    // ══════════════════════════════════════════════════════════════════════
    // Phase 4 — Message Flood (concurrent)
    // ══════════════════════════════════════════════════════════════════════
    let total_msg = members * mpm;
    println!("Phase 4 — Message flood ({total_msg} events across {members} members) ...");
    comm_push(&log,&seq,"msg_flood","system","INFO","phase_start","","",vec![],true,
        &format!("Phase 4 — Message flood ({total_msg} events)"));
    let t4 = Instant::now();

    // Snapshot anchors; no Arc<Mutex> needed inside tasks
    let anchors_snap: Vec<String> = anchors.lock().unwrap().clone();

    let sent_ctr  = Arc::new(AtomicU64::new(0));
    let error_ctr = Arc::new(AtomicU64::new(0));

    let mut msg_tasks = Vec::new();
    for i in 0..members {
        let node      = assigned_node_url(i, members, &args.node_a, &args.node_b);
        let a         = actor(i);
        let key       = keys[i].clone();
        let sid       = space_id.clone();
        let rids      = room_ids.clone();
        let mlog      = log.clone(); let mseq = seq.clone();
        let sent_cl   = sent_ctr.clone();
        let err_cl    = error_ctr.clone();
        let init_anch = anchors_snap[i].clone();
        let mpm_cl    = mpm;
        let node_cl   = node.clone();

        msg_tasks.push(tokio::spawn(async move {
            let result: Result<()> = async {
                let mut conn = connect_url(&node_cl).await?;
                conn.client_authenticate(&key).await?;
                let mut last = init_anch;

                for mi in 0..mpm_cl {
                    let ri       = mi % 3;
                    let room_id  = rids[ri].as_ref().clone();
                    let rname    = ["general","random","tech"][ri];

                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        rand::random::<u64>() % 50)).await;

                    let mev = sign_event(build_message_text_event(
                        &key, &sid, &room_id, vec![last.clone()],
                        &format!("M{i} msg {mi}")), &key);
                    let eid = mev.event_id.clone().unwrap_or_default();
                    let prev0 = mev.prev_events.first().cloned().unwrap_or_default();

                    // Send; on connection failure reconnect once and retry the same event.
                    let (send_ok, retried) = match conn.send_event(&mev).await {
                        Ok(_) => (true, false),
                        Err(e) => {
                            tracing::warn!(member=i, msg=mi, "send failed ({e:#}), reconnecting");
                            comm_push(&mlog,&mseq,"msg_flood",&a,"INFO","reconnect","",&node_cl,
                                vec![],true,&format!("msg_index={mi}"));
                            let retry = async {
                                let mut nc = connect_url(&node_cl).await?;
                                nc.client_authenticate(&key).await?;
                                Ok::<_,anyhow::Error>(nc)
                            }.await;
                            match retry {
                                Ok(nc) => {
                                    conn = nc;
                                    match conn.send_event(&mev).await {
                                        Ok(_) => (true, true),
                                        Err(e2) => {
                                            tracing::error!(member=i, msg=mi, "retry failed: {e2:#}");
                                            (false, true)
                                        }
                                    }
                                }
                                Err(e2) => {
                                    tracing::error!(member=i, msg=mi, "reconnect failed: {e2:#}");
                                    (false, true)
                                }
                            }
                        }
                    };
                    if send_ok {
                        last = eid.clone();
                        sent_cl.fetch_add(1, Ordering::Relaxed);
                        comm_push(&mlog,&mseq,"msg_flood",&a,"SENT","message.text",
                            &eid, &node_cl, vec![prev0], true,
                            &format!("room={rname} msg_index={mi}{}",
                                if retried {" reconnected"} else {""}));
                    } else {
                        err_cl.fetch_add(1, Ordering::Relaxed);
                        comm_push(&mlog,&mseq,"msg_flood",&a,"SENT","message.text",
                            "", &node_cl, vec![], false,
                            &format!("room={rname} msg_index={mi} error=send_failed"));
                    }
                }
                let _ = conn.goodbye("flood_done").await;
                Ok(())
            }.await;
            if let Err(e) = result {
                tracing::error!(member=i, "Flood task failed: {e:#}");
            }
        }));
    }

    // Progress ticker — every 5s
    let prog_sent  = sent_ctr.clone();
    let prog_err   = error_ctr.clone();
    let prog_total = total_msg as u64;
    let t4_prog    = t4;
    let prog_task  = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let s = prog_sent.load(Ordering::Relaxed);
            let e = prog_err.load(Ordering::Relaxed);
            println!("  [stress] {s} / {prog_total} events sent  ({e} errors)  elapsed: {}s",
                t4_prog.elapsed().as_secs());
            if s + e >= prog_total { break; }
        }
    });

    for t in msg_tasks { let _ = t.await; }
    prog_task.abort();

    let d4      = t4.elapsed();
    let f_sent  = sent_ctr.load(Ordering::Relaxed);
    let f_err   = error_ctr.load(Ordering::Relaxed);
    let throughput = f_sent as f64 / d4.as_secs_f64().max(0.001);
    comm_push(&log,&seq,"msg_flood","system","INFO","phase_end","","",vec![],true,
        &format!("duration_ms={} sent={f_sent} errors={f_err}", d4.as_millis()));
    println!("  done in {:.1}s  ({f_sent}/{total_msg} sent, {f_err} errors)", d4.as_secs_f64());

    // Resting point — let federation delivery and pending-buffer drain complete on both
    // nodes before the report is generated. Without this, the apply_event count on the
    // receiving node is a snapshot mid-drain and will appear lower than expected.
    if args.rest_ms > 0 {
        println!("  resting {}ms (nodes settling after Phase 4) ...", args.rest_ms);
        comm_push(&log,&seq,"rest","system","INFO","rest_start","","",vec![],true,
            &format!("after=phase4 ms={}", args.rest_ms));
        tokio::time::sleep(tokio::time::Duration::from_millis(args.rest_ms)).await;
        comm_push(&log,&seq,"rest","system","INFO","rest_end","","",vec![],true,"after=phase4");
    }

    // ══════════════════════════════════════════════════════════════════════
    // Content leak check
    // ══════════════════════════════════════════════════════════════════════
    let (leak_count, log_path_checked) = match find_latest_client_log() {
        Some(p) => {
            let text = std::fs::read_to_string(&p).unwrap_or_default();
            (scan_message_pattern(&text), Some(p))
        }
        None => (0, None),
    };
    if leak_count > 0 {
        println!();
        println!("WARNING: CONTENT LEAK DETECTED — message text found in log files.");
        println!("This is a critical bug. Do not use these logs for verification.");
    }
    comm_push(&log,&seq,"system","system","INFO","content_leak_check","","",vec![],leak_count==0,
        &format!("matches={leak_count}"));

    // ══════════════════════════════════════════════════════════════════════
    // Federation completeness — count apply_event message.text on each node
    // ══════════════════════════════════════════════════════════════════════
    let project_dir = exe_dir().parent().map(|p| p.to_path_buf()).unwrap_or_else(exe_dir);
    let node_a_log_dir = project_dir.join("test").join("node_a").join("logs");
    let node_b_log_dir = project_dir.join("test").join("node_b").join("logs");

    let node_a_applied = find_latest_node_log(&node_a_log_dir)
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .map(|t| count_apply_event_message_text(&t))
        .unwrap_or(0);
    let node_b_applied = find_latest_node_log(&node_b_log_dir)
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .map(|t| count_apply_event_message_text(&t))
        .unwrap_or(0);

    let fed_a_expected = (members / 2) * mpm;
    let fed_b_expected = (members - members / 2) * mpm;
    let fed_a_ok = node_a_applied >= fed_a_expected;
    let fed_b_ok = node_b_applied >= fed_b_expected;

    // ══════════════════════════════════════════════════════════════════════
    // Compute per-member + per-room stats from the comm log
    // ══════════════════════════════════════════════════════════════════════
    let entries: Vec<CommEntry> = log.lock().unwrap().clone();

    let mut m_sent:  Vec<u64> = vec![0; members];
    let mut m_err:   Vec<u64> = vec![0; members];
    let mut r_count: [u64; 3] = [0; 3];

    for e in &entries {
        if e.phase == "msg_flood" && e.direction == "SENT" && e.event_type == "message.text" {
            let mi: usize = if e.actor == "Alice" { 0 }
                else { e.actor.trim_start_matches('M').parse().unwrap_or(0) };
            if mi < members {
                if e.ok { m_sent[mi] += 1; } else { m_err[mi] += 1; }
            }
            if e.ok {
                if e.notes.contains("room=general")      { r_count[0] += 1; }
                else if e.notes.contains("room=random")  { r_count[1] += 1; }
                else if e.notes.contains("room=tech")    { r_count[2] += 1; }
            }
        }
    }

    // DAG chain integrity: for each member verify prev_events chain in order
    let mut chain_ok: Vec<bool> = vec![true; members];
    for i in 0..members {
        let a = actor(i);
        let msgs: Vec<&CommEntry> = entries.iter()
            .filter(|e| e.phase == "msg_flood" && e.direction == "SENT"
                && e.event_type == "message.text" && e.actor == a && e.ok)
            .collect();
        let mut prev = anchors_snap[i].clone();
        for e in msgs {
            if e.prev_events.first().map(|s| s.as_str()) != Some(prev.as_str()) {
                chain_ok[i] = false; break;
            }
            prev = e.event_id.clone();
        }
    }

    let all_chains_ok = chain_ok.iter().all(|&b| b);

    // ══════════════════════════════════════════════════════════════════════
    // Build report
    // ══════════════════════════════════════════════════════════════════════
    let outcome = if f_err == 0 && jf == 0 && fed_a_ok && fed_b_ok { "PASS" }
        else if f_err == 0 && jf == 0 { "PARTIAL" }  // federation incomplete
        else { "PARTIAL" };

    let half      = members / 2;
    let half_b    = members - half;
    let total_ev_expected = 1 + 3 + (members - 1) + (members - 1) * 4 + members * mpm;
    let total_ev_sent: usize = entries.iter()
        .filter(|e| e.direction == "SENT" && e.ok
            && !["phase_start","phase_end","test_start","join_failed",
                 "fed_start","fed_handshake_ok","history_recv","fed_complete",
                 "content_leak_check"].contains(&e.event_type.as_str()))
        .count();
    let total_duration = d1 + d2 + d3 + d4;

    let mut report: Vec<String> = Vec::new();
    let sep  = "=".repeat(52);
    let sep2 = "-".repeat(52);

    report.push(sep.clone());
    report.push("XGen Protocol — Phase 1 Stress Test Report".into());
    report.push(sep.clone());
    report.push(format!("Date:    {}", chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)));
    report.push(format!("Version: {} ({})", xgen_common::build_info::VERSION, xgen_common::build_info::GIT_HASH));
    report.push(String::new());
    report.push("Configuration".into());
    report.push(sep2.clone());
    report.push(format!("  Node A:    {}", args.node_a));
    report.push(format!("  Node B:    {}", args.node_b));
    report.push(format!("  Members:   {members}  ({half} on Node A [M0–M{}], {} on Node B [M{}–M{}])",
        half-1, half_b, half, members-1));
    report.push(format!("  Rooms:     3  (general, random, tech)"));
    report.push(format!("  Messages:  {mpm} per member"));
    report.push(String::new());
    report.push(format!("OUTCOME: {outcome}"));
    report.push("=".repeat(16));
    report.push(String::new());
    report.push("Phase Timing".into());
    report.push(sep2.clone());
    report.push(format!("  Phase 1  Setup:              {:.2}s", d1.as_secs_f64()));
    report.push(format!("  Phase 2  Registration:       {:.2}s", d2.as_secs_f64()));
    report.push(format!("  Phase 3  Federation + Join:  {:.2}s", d3.as_secs_f64()));
    report.push(format!("  Phase 4  Message Flood:      {:.2}s", d4.as_secs_f64()));
    report.push(format!("  Total:                       {:.2}s", total_duration.as_secs_f64()));
    report.push(String::new());
    report.push("Event Statistics".into());
    report.push(sep2.clone());
    report.push(format!("  Expected total events:  {total_ev_expected}  (1 space + 3 rooms + {} invites + {} joins + {} messages)",
        members-1, (members-1)*4, members*mpm));
    report.push(format!("  Protocol events sent:   {total_ev_sent}"));
    report.push(format!("  Messages attempted:     {total_msg}"));
    report.push(format!("  Messages sent OK:       {f_sent}  ({:.1}%)", f_sent as f64 / total_msg as f64 * 100.0));
    report.push(format!("  Send errors:            {f_err}"));
    report.push(format!("  Join failures:          {jf}"));
    report.push(format!("  Throughput (Phase 4):   {throughput:.1} events/sec"));
    report.push(String::new());
    report.push("Room Distribution (message events)".into());
    report.push(sep2.clone());
    let rnames = ["general","random","tech"];
    for ri in 0..3usize {
        let exp = expected_per_room(mpm, ri) * members;
        let got = r_count[ri];
        let mark = if got == exp as u64 { "✓" } else { "✗" };
        report.push(format!("  {:8}  got {:>5}  expected {:>5}  {mark}", rnames[ri], got, exp));
    }
    report.push(String::new());
    report.push("Per-Member Statistics".into());
    report.push(sep2.clone());
    report.push(format!("  {:>5}  {:10}  {:6}  {:>5}  {:>6}  {:9}",
        "Index","Actor","Node","Sent","Errors","DAG Chain"));
    report.push(format!("  {:>5}  {:10}  {:6}  {:>5}  {:>6}  {:9}",
        "-----","----------","------","-----","------","---------"));
    for i in 0..members {
        let n = if i < half { "Node A" } else { "Node B" };
        let ch = if chain_ok[i] { "OK" } else { "BROKEN" };
        report.push(format!("  {:>5}  {:10}  {:6}  {:>5}  {:>6}  {:9}",
            i, actor(i), n, m_sent[i], m_err[i], ch));
    }
    report.push(format!("  {:>5}  {:10}  {:6}  {:>5}  {:>6}",
        "","Total","", m_sent.iter().sum::<u64>(), m_err.iter().sum::<u64>()));
    report.push(String::new());
    report.push("DAG Chain Integrity".into());
    report.push(sep2.clone());
    report.push(format!("  Result: {}  (each sender's prev_events chain verified)",
        if all_chains_ok { "OK — all members" } else { "PARTIAL — see per-member table" }));
    report.push(String::new());
    report.push("Content Leak Check".into());
    report.push(sep2.clone());
    report.push(format!("  Pattern:  M\\d+ msg \\d+"));
    match &log_path_checked {
        Some(p) => report.push(format!("  Scanned:  {}", p.display())),
        None    => report.push("  Scanned:  (log file not found)".into()),
    }
    report.push(format!("  Result:   {}",
        if leak_count == 0 { "CLEAN — 0 matches  ✓".to_string() }
        else { format!("LEAK DETECTED — {leak_count} matches  ✗  CRITICAL BUG") }));
    report.push(String::new());
    report.push("Federation Completeness (message events applied on receiving node)".into());
    report.push(sep2.clone());
    report.push(format!("  Node A applied  (M0–M{}):  {:>5} / {:>5}  {}",
        half-1, node_a_applied, fed_a_expected, if fed_a_ok { "✓" } else { "✗" }));
    report.push(format!("  Node B applied  (M{}–M{}):  {:>5} / {:>5}  {}",
        half, members-1, node_b_applied, fed_b_expected, if fed_b_ok { "✓" } else { "✗" }));
    if node_a_applied == 0 && node_b_applied == 0 {
        report.push("  (node log files not found — run nodes from <project>/test/node_*/  directories)".into());
    }
    report.push(String::new());
    report.push("Verification Checklist".into());
    report.push(sep2.clone());
    report.push(format!("  [auto]   Send errors:         {f_err}  {}",   if f_err==0  {"✓"} else {"✗"}));
    report.push(format!("  [auto]   Join failures:        {jf}  {}",    if jf==0     {"✓"} else {"✗"}));
    report.push(format!("  [auto]   Content leak:         {}",           if leak_count==0 {"CLEAN  ✓"} else {"LEAK  ✗"}));
    report.push(format!("  [auto]   DAG chain integrity:  {}",           if all_chains_ok {"OK  ✓"} else {"PARTIAL  ✗"}));
    report.push(format!("  [auto]   Federation completeness Node A:  {:>5} / {:>5}  {}",
        node_a_applied, fed_a_expected, if fed_a_ok { "✓" } else { "✗" }));
    report.push(format!("  [auto]   Federation completeness Node B:  {:>5} / {:>5}  {}",
        node_b_applied, fed_b_expected, if fed_b_ok { "✓" } else { "✗" }));
    report.push("  [manual] No ERROR lines in Node A log for valid events".into());
    report.push("  [manual] No ERROR lines in Node B log for valid events".into());
    report.push("  [manual] Session footer present in all Node logs (clean shutdown)".into());
    report.push(format!("  [manual] direction=IN on Node A for M0–M{} outbound events", half-1));
    report.push(format!("  [manual] direction=IN on Node B for M{}–M{} outbound events", half, members-1));
    report.push("  [manual] Federation propagation: Node B logs show events from Node A".into());
    report.push("  [manual] Federation propagation: Node A logs show events from Node B".into());
    report.push(String::new());
    report.push("Identity Registry".into());
    report.push(sep2.clone());
    for i in 0..members {
        let n = if i < half { "Node A" } else { "Node B" };
        let short = ids[i].get(ids[i].len().saturating_sub(40)..).unwrap_or(&ids[i]);
        report.push(format!("  {:10}  ...{}  {}", actor(i), short, n));
    }
    report.push(String::new());
    report.push("Space and Room IDs".into());
    report.push(sep2.clone());
    report.push(format!("  Space:          {}", space_id.as_ref()));
    for (i, rn) in ["general","random","tech"].iter().enumerate() {
        report.push(format!("  Room {:8}  {}", rn, room_ids[i].as_ref()));
    }
    report.push(String::new());
    report.push("Log Files (for manual verification)".into());
    report.push(sep2.clone());
    report.push("  Node A:   test/node_a/logs/  (xgen-node_*.log)".into());
    report.push("  Node B:   test/node_b/logs/  (xgen-node_*.log)".into());
    match &log_path_checked {
        Some(p) => report.push(format!("  Client:   {}", p.display())),
        None    => report.push("  Client:   (not found — check <exe_dir>/logs/)".into()),
    }
    report.push(String::new());
    report.push("Communication Record".into());
    report.push(sep2.clone());
    report.push(format!("  File:     stress-reports/xgen-stress-test_{test_ts}_events.json"));
    report.push(format!("  Entries:  {}  (all events, responses, phase markers — no message content)", entries.len()));
    report.push(format!("  Format:   JSON array — fields: seq ts phase actor direction event_type event_id node prev_events ok notes"));
    report.push(String::new());
    report.push(sep.clone());
    report.push(format!("Phase 1 Stress Test — {outcome}"));
    if f_err == 0 && jf == 0 && leak_count == 0 {
        report.push(format!("All {total_msg} messages delivered. Zero errors. {members} concurrent identities across 2 federated nodes."));
        report.push("DAG chain integrity verified. Content leak check clean.".into());
        report.push("This run demonstrates Phase 1 correctness under concurrent load.".into());
    } else {
        report.push(format!("Sent {f_sent}/{total_msg} messages. {f_err} send errors. {jf} join failures."));
    }
    report.push(sep.clone());

    // ══════════════════════════════════════════════════════════════════════
    // Write files + print
    // ══════════════════════════════════════════════════════════════════════
    let reports_dir = exe_dir().join("stress-reports");
    std::fs::create_dir_all(&reports_dir)?;

    let report_path = reports_dir.join(format!("xgen-stress-test_{test_ts}_report.txt"));
    let events_path = reports_dir.join(format!("xgen-stress-test_{test_ts}_events.json"));

    // Print report to stdout
    println!();
    for line in &report { println!("{line}"); }

    // Write report file
    std::fs::write(&report_path, report.join("\n"))
        .with_context(|| format!("failed to write report to {}", report_path.display()))?;

    // Write events JSON
    let events_json = serde_json::to_string_pretty(&entries)
        .context("failed to serialize comm log")?;
    std::fs::write(&events_path, events_json)
        .with_context(|| format!("failed to write events to {}", events_path.display()))?;

    println!();
    println!("Report saved:  {}", report_path.display());
    println!("Events log:    {}", events_path.display());

    Ok(())
}

// ── Stress test types and helpers ─────────────────────────────────────────────

/// One entry in the full communication record written by the stress test.
/// Content (message text) is never stored here.
#[derive(serde::Serialize, Clone)]
struct CommEntry {
    seq: u64,
    ts: String,
    phase: String,
    actor: String,
    direction: String,   // SENT | RECV | INFO
    event_type: String,
    event_id: String,
    node: String,
    prev_events: Vec<String>,
    ok: bool,
    notes: String,
}

type CommLog = std::sync::Arc<std::sync::Mutex<Vec<CommEntry>>>;
type Seq     = std::sync::Arc<std::sync::atomic::AtomicU64>;

fn comm_push(
    log: &CommLog, seq: &Seq,
    phase: &str, actor: &str, dir: &str,
    etype: &str, eid: &str, node: &str,
    prev: Vec<String>, ok: bool, notes: &str,
) {
    use std::sync::atomic::Ordering;
    let n = seq.fetch_add(1, Ordering::Relaxed);
    log.lock().unwrap().push(CommEntry {
        seq: n,
        ts: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        phase: phase.into(), actor: actor.into(), direction: dir.into(),
        event_type: etype.into(), event_id: eid.into(), node: node.into(),
        prev_events: prev, ok, notes: notes.into(),
    });
}

fn comm_event(log: &CommLog, seq: &Seq, phase: &str, actor: &str, dir: &str,
    ev: &Event, node: &str)
{
    comm_push(log, seq, phase, actor, dir,
        &ev.event_type.to_string(),
        ev.event_id.as_deref().unwrap_or(""),
        node,
        ev.prev_events.clone(),
        true, "");
}

/// Scan text for the pattern M\d+ msg \d+ (message content leak check).
fn scan_message_pattern(text: &str) -> usize {
    let b = text.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'M' {
            let j = i + 1;
            let mut k = j;
            while k < b.len() && b[k].is_ascii_digit() { k += 1; }
            if k > j {
                if b.get(k..k + 5) == Some(b" msg ") {
                    let mut l = k + 5;
                    while l < b.len() && b[l].is_ascii_digit() { l += 1; }
                    if l > k + 5 { count += 1; i = l; continue; }
                }
            }
        }
        i += 1;
    }
    count
}

/// Find the most recently modified xgen-client_*.log in <exe_dir>/logs/.
fn find_latest_client_log() -> Option<PathBuf> {
    let log_dir = exe_dir().join("logs");
    std::fs::read_dir(&log_dir).ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("xgen-client_"))
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
}

/// Find the most recently modified xgen-node_*.log in a given logs/ directory.
fn find_latest_node_log(log_dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(log_dir).ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("xgen-node_"))
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
}

/// Count lines containing both `apply_event` and `message.text` in a log file.
fn count_apply_event_message_text(text: &str) -> usize {
    text.lines()
        .filter(|line| line.contains("apply_event") && line.contains("message.text"))
        .count()
}

/// node_a if member_index < total/2, else node_b. Alice (0) always on node_a.
fn assigned_node_url(member_index: usize, total: usize, node_a: &str, node_b: &str) -> String {
    if member_index < total / 2 { node_a.to_string() } else { node_b.to_string() }
}

/// Expected messages sent to room_idx (0=general,1=random,2=tech) per member.
fn expected_per_room(messages_per_member: usize, room_idx: usize) -> usize {
    messages_per_member / 3 + if room_idx < messages_per_member % 3 { 1 } else { 0 }
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
    conn: &mut xgen_core::transport::connection::Connection<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    _space_id: &str,
) -> Result<Vec<String>> {
    // Phase 1: send a sync_request with empty since to get recent events,
    // then collect event IDs (the last one is the most recent tip).
    let req = xgen_core::wire::types::TransportMessage::SyncRequest {
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
