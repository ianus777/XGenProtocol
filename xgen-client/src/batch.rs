// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Batch command dispatch and named-pipe IPC (Milestone 1–3).
// Named pipe server and batch client are Windows-only (D-043).

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use clap::{Args, CommandFactory, Parser, Subcommand};

use xgen_common::{
    build_info,
    event_trace::{EventDirection, SessionContext, SpaceRole, trace_event},
    state::{ClientState, KnownRoom, KnownSpace},
};
use xgen_node_lib::{
    identity::{
        keypair,
        registration::{build_register, identity_id_from_key, sign_register},
    },
    message::exchange::build_message_text_event,
    space::state::{build_room_create_event, build_space_create_event, sign_event},
    transport::{client::connect_url, connection::Inbound},
    wire::types::{Event, EventType, IdentityMessage, TransportMessage},
};

// ── Config (local parse, mirrors ClientConfig in main.rs) ──────────────────────

#[derive(serde::Deserialize)]
struct Config {
    client: ConfigClient,
    paths: ConfigPaths,
}

#[derive(serde::Deserialize)]
struct ConfigClient {
    node: String,
}

#[derive(serde::Deserialize)]
struct ConfigPaths {
    keypair_path: String,
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

// ── Clap definitions (M3: "existing Command object", no duplication) ───────────

#[derive(Parser)]
#[command(name = "xgen-client", version = build_info::VERSION)]
struct BatchCli {
    /// Node WebSocket endpoint. Overrides config.
    #[arg(short, long)]
    node: Option<String>,

    #[command(subcommand)]
    command: BatchCommand,
}

#[derive(Subcommand)]
enum BatchCommand {
    /// Print local identity ID and display name.
    Whoami,
    /// Print local client state summary.
    Status,
    /// Register this identity on the Node.
    Register(RegisterArgs),
    /// Create a new Space on the Node.
    CreateSpace(CreateSpaceArgs),
    /// Create a new Room within a Space.
    CreateRoom(CreateRoomArgs),
    /// Invite an identity to a Space.
    Invite(InviteArgs),
    /// Join a Space or a Room.
    Join(JoinArgs),
    /// Send a message to a Room.
    Send(SendArgs),
}

#[derive(Args)]
struct RegisterArgs {
    /// Display name to register. Max 128 characters.
    #[arg(long)]
    name: String,
}

#[derive(Args)]
struct CreateSpaceArgs {
    /// Display name for the Space.
    #[arg(long)]
    name: String,
}

#[derive(Args)]
struct CreateRoomArgs {
    /// Space ID (xgen://hash/sha256:...)
    #[arg(long)]
    space: String,
    /// Display name for the Room.
    #[arg(long)]
    name: String,
}

#[derive(Args)]
struct InviteArgs {
    /// Space ID
    #[arg(long)]
    space: String,
    /// Identity ID to invite
    #[arg(long)]
    identity: String,
    /// Role: owner, admin, moderator, member
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
    /// Message text.
    #[arg(long)]
    text: String,
}

/// Returns the clap Command object used for batch dispatch (M3 §3.3).
pub fn app_command() -> clap::Command {
    BatchCli::command()
}

// ── Local helpers ───────────────────────────────────────────────────────────────

fn resolve_node(node_override: Option<&str>, data_dir: &Path) -> String {
    if let Some(n) = node_override {
        return n.to_string();
    }
    let path = data_dir.join("xgen-client_config.toml");
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(cfg) = toml::from_str::<Config>(&text) {
            return cfg.client.node;
        }
    }
    "ws://127.0.0.1:8080/xgen".to_string()
}

fn resolve_keypair_path(data_dir: &Path) -> PathBuf {
    let path = data_dir.join("xgen-client_config.toml");
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(cfg) = toml::from_str::<Config>(&text) {
            let p = PathBuf::from(&cfg.paths.keypair_path);
            if p.is_absolute() {
                return p;
            }
        }
    }
    data_dir.join("xgen-client_keypair.enc")
}

fn load_keypair(data_dir: &Path) -> Result<ed25519_dalek::SigningKey> {
    let path = resolve_keypair_path(data_dir);
    if !path.exists() {
        bail!("keypair not found: {}\n  Run init first.", path.display());
    }
    keypair::load(&path, "")
        .with_context(|| format!("failed to load keypair from {}", path.display()))
}

fn load_state(data_dir: &Path) -> Result<ClientState> {
    let path = data_dir.join("xgen-client_state.json");
    if !path.exists() {
        bail!("state file not found: {}\n  Run register first.", path.display());
    }
    let json = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&json).context("state file is corrupt")
}

fn load_or_default_state(data_dir: &Path, node: &str) -> Result<ClientState> {
    let path = data_dir.join("xgen-client_state.json");
    if path.exists() {
        if let Ok(json) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<ClientState>(&json) {
                return Ok(state);
            }
        }
    }
    let sk = load_keypair(data_dir)?;
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

fn write_state(data_dir: &Path, state: &ClientState) -> Result<()> {
    let path = data_dir.join("xgen-client_state.json");
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(&path, json)
        .with_context(|| format!("failed to write {}", path.display()))
}

async fn get_dag_tips(
    conn: &mut xgen_node_lib::transport::connection::Connection<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    _space_id: &str,
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
                if let Some(id) = ev.event_id {
                    tips = vec![id];
                }
            }
            _ => break,
        }
    }
    Ok(tips)
}

// ── Command handlers ────────────────────────────────────────────────────────────

async fn exec_whoami(data_dir: &Path) -> Result<()> {
    let state = load_state(data_dir)?;
    tracing::info!(
        identity_id = %state.identity_id,
        display_name = %state.display_name,
        "whoami"
    );
    Ok(())
}

async fn exec_status(data_dir: &Path) -> Result<()> {
    let state = load_state(data_dir)?;
    tracing::info!(
        identity_id = %state.identity_id,
        spaces = state.spaces.len(),
        "status"
    );
    Ok(())
}

async fn exec_register(
    args: RegisterArgs,
    node_override: Option<&str>,
    data_dir: &Path,
) -> Result<()> {
    let node = resolve_node(node_override, data_dir);
    let signing_key = load_keypair(data_dir)?;
    let identity_id = identity_id_from_key(&signing_key);

    let mut conn = connect_url(&node).await.context("failed to connect to Node")?;
    let auth_id = conn
        .client_authenticate(&signing_key)
        .await
        .context("authentication failed")?;
    tracing::info!(identity_id = %auth_id, "authenticated");

    let reg = sign_register(
        build_register(&signing_key, Some(args.name.clone())),
        &signing_key,
    );
    conn.send_identity(&reg)
        .await
        .context("failed to send registration")?;

    match conn.recv().await.context("no response from Node")? {
        Inbound::Identity(IdentityMessage::RegisterOk { registered_at, .. }) => {
            tracing::info!(
                identity_id = %identity_id,
                registered_at = %registered_at,
                "registered"
            );
            let state = ClientState {
                identity_id,
                display_name: args.name,
                version: build_info::VERSION.to_string(),
                build: build_info::GIT_HASH.to_string(),
                home_node: node,
                updated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                spaces: vec![],
            };
            write_state(data_dir, &state)?;
        }
        Inbound::Identity(IdentityMessage::RegisterFail {
            error_code,
            error_string,
            ..
        }) => {
            bail!("registration rejected (code {}): {}", error_code, error_string);
        }
        other => bail!("unexpected response: {:?}", other),
    }
    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

async fn exec_create_space(
    args: CreateSpaceArgs,
    node_override: Option<&str>,
    data_dir: &Path,
) -> Result<()> {
    let node = resolve_node(node_override, data_dir);
    let signing_key = load_keypair(data_dir)?;
    let identity_id = identity_id_from_key(&signing_key);

    let mut conn = connect_url(&node).await.context("failed to connect")?;
    let auth_id = conn
        .client_authenticate(&signing_key)
        .await
        .context("auth failed")?;
    tracing::info!(identity_id = %auth_id, "authenticated");

    let space_ev = sign_event(
        build_space_create_event(&signing_key, &args.name, None, 1, &node),
        &signing_key,
    );
    let space_id = space_ev.event_id.clone().unwrap();

    let ctx = SessionContext {
        identity_id: Some(identity_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(space_id.clone()),
    };
    trace_event(&space_ev, EventDirection::Out, &ctx);
    conn.send_event(&space_ev)
        .await
        .context("failed to send space_create")?;
    tracing::info!(space_id = %space_id, name = %args.name, "space created");

    let mut state = load_or_default_state(data_dir, &node)?;
    state.spaces.push(KnownSpace {
        space_id,
        name: args.name,
        node_endpoint: node,
        role: "owner".to_string(),
        rooms: vec![],
    });
    state.updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    write_state(data_dir, &state)?;

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

async fn exec_create_room(
    args: CreateRoomArgs,
    node_override: Option<&str>,
    data_dir: &Path,
) -> Result<()> {
    let node = resolve_node(node_override, data_dir);
    let signing_key = load_keypair(data_dir)?;

    let mut conn = connect_url(&node).await.context("failed to connect")?;
    let auth_id = conn
        .client_authenticate(&signing_key)
        .await
        .context("auth failed")?;
    tracing::info!(identity_id = %auth_id, "authenticated");

    let room_ev = sign_event(
        build_room_create_event(&signing_key, &args.space, &args.name, None),
        &signing_key,
    );
    let room_id = room_ev.event_id.clone().unwrap();

    let ctx = SessionContext {
        identity_id: Some(auth_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(args.space.clone()),
    };
    trace_event(&room_ev, EventDirection::Out, &ctx);
    conn.send_event(&room_ev)
        .await
        .context("failed to send room_create")?;
    tracing::info!(room_id = %room_id, name = %args.name, "room created");

    let mut state = load_or_default_state(data_dir, &node)?;
    if let Some(space) = state.spaces.iter_mut().find(|s| s.space_id == args.space) {
        space.rooms.push(KnownRoom {
            room_id,
            name: args.name,
            joined: true,
        });
    }
    state.updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    write_state(data_dir, &state)?;

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

async fn exec_invite(
    args: InviteArgs,
    node_override: Option<&str>,
    data_dir: &Path,
) -> Result<()> {
    let node = resolve_node(node_override, data_dir);
    let signing_key = load_keypair(data_dir)?;
    let sender = identity_id_from_key(&signing_key);

    let mut conn = connect_url(&node).await.context("failed to connect")?;
    let auth_id = conn
        .client_authenticate(&signing_key)
        .await
        .context("auth failed")?;
    tracing::info!(identity_id = %auth_id, "authenticated");

    let invite_ev = sign_event(
        Event::new(
            EventType::MembershipInvite,
            sender,
            String::new(),
            args.space.clone(),
            vec![args.space.clone()],
            Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            serde_json::json!({ "target_identity": args.identity, "role": args.role }),
        ),
        &signing_key,
    );

    let ctx = SessionContext {
        identity_id: Some(auth_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(args.space.clone()),
    };
    trace_event(&invite_ev, EventDirection::Out, &ctx);
    conn.send_event(&invite_ev)
        .await
        .context("failed to send invite")?;
    tracing::info!(
        space_id = %args.space,
        target = %args.identity,
        "invite sent"
    );

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

async fn exec_join(
    args: JoinArgs,
    node_override: Option<&str>,
    data_dir: &Path,
) -> Result<()> {
    let node = resolve_node(node_override, data_dir);
    let signing_key = load_keypair(data_dir)?;
    let sender = identity_id_from_key(&signing_key);

    let mut conn = connect_url(&node).await.context("failed to connect")?;
    let auth_id = conn
        .client_authenticate(&signing_key)
        .await
        .context("auth failed")?;
    tracing::info!(identity_id = %auth_id, "authenticated");

    let join_ev = sign_event(
        Event::new(
            EventType::MembershipJoin,
            sender,
            args.room.clone().unwrap_or_default(),
            args.space.clone(),
            vec![args.space.clone()],
            Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            serde_json::json!({}),
        ),
        &signing_key,
    );

    let ctx = SessionContext {
        identity_id: Some(auth_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(args.space.clone()),
    };
    trace_event(&join_ev, EventDirection::Out, &ctx);
    conn.send_event(&join_ev)
        .await
        .context("failed to send join")?;
    tracing::info!(space_id = %args.space, "joined");

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

async fn exec_send(
    args: SendArgs,
    node_override: Option<&str>,
    data_dir: &Path,
) -> Result<()> {
    let node = resolve_node(node_override, data_dir);
    let signing_key = load_keypair(data_dir)?;

    let mut conn = connect_url(&node).await.context("failed to connect")?;
    let auth_id = conn
        .client_authenticate(&signing_key)
        .await
        .context("auth failed")?;
    tracing::info!(identity_id = %auth_id, "authenticated");

    let prev_events = get_dag_tips(&mut conn, &args.space)
        .await
        .unwrap_or_else(|_| vec![args.space.clone()]);

    let msg_ev = sign_event(
        build_message_text_event(
            &signing_key,
            &args.space,
            &args.room,
            prev_events,
            &args.text,
        ),
        &signing_key,
    );

    let ctx = SessionContext {
        identity_id: Some(auth_id.clone()),
        role: Some(SpaceRole::Owner),
        space_id: Some(args.space.clone()),
    };
    trace_event(&msg_ev, EventDirection::Out, &ctx);
    conn.send_event(&msg_ev)
        .await
        .context("failed to send message")?;
    tracing::info!(room = %args.room, "message sent");

    let _ = conn.goodbye("client_disconnect").await;
    Ok(())
}

// ── Dispatch (M3: tokenize → clap → execute) ───────────────────────────────────

/// Tokenize and dispatch one batch command line.
/// Uses shlex for quoting/escaping; dispatches via clap; no shell invocation.
pub async fn dispatch_line(line: &str, data_dir: &Path) -> Result<()> {
    let tokens = shlex::split(line).unwrap_or_else(|| vec![line.to_string()]);
    let mut argv = vec!["xgen-client".to_string()];
    argv.extend(tokens);

    let cli = BatchCli::try_parse_from(&argv)
        .map_err(|e| anyhow::anyhow!("unrecognised command: {}", e))?;

    let node_override = cli.node.as_deref();

    match cli.command {
        BatchCommand::Whoami => exec_whoami(data_dir).await,
        BatchCommand::Status => exec_status(data_dir).await,
        BatchCommand::Register(a) => exec_register(a, node_override, data_dir).await,
        BatchCommand::CreateSpace(a) => exec_create_space(a, node_override, data_dir).await,
        BatchCommand::CreateRoom(a) => exec_create_room(a, node_override, data_dir).await,
        BatchCommand::Invite(a) => exec_invite(a, node_override, data_dir).await,
        BatchCommand::Join(a) => exec_join(a, node_override, data_dir).await,
        BatchCommand::Send(a) => exec_send(a, node_override, data_dir).await,
    }
}

// ── Named pipe server — M1 (Windows only) ──────────────────────────────────────

/// Start the named pipe server in the running instance.
/// Accepts one connection at a time; reads commands until `__END__`;
/// dispatches each; writes `OK\n` or `ERROR: …\n`; loops.
/// Stops cleanly when `shutdown_rx` delivers `true`.
#[cfg(target_os = "windows")]
pub async fn start_pipe_server(
    pipe_name_str: String,
    data_dir: PathBuf,
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

        // Read command lines until __END__ or EOF
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
pub fn run_batch_client(raw_path: &str, pipe_name_str: &str) -> i32 {
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
    rt.block_on(run_batch_client_async(raw_path, pipe_name_str))
}

#[cfg(target_os = "windows")]
async fn run_batch_client_async(raw_path: &str, pipe_name_str: &str) -> i32 {
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
            eprintln!(
                "error: no running xgen-client instance found at {}\n\
                       Start xgen-client-app.exe before running --batch.",
                pipe_name_str
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
