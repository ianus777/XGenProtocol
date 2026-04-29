// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;

use xgen_common::{build_info, state::NodeState};
use xgen_node_lib::{
    crypto::encoding,
    identity::{keypair, registry::IdentityRegistry},
};

// ── Node config ────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeConfig {
    node: NodeSection,
    paths: PathsSection,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeSection {
    listen: String,
    local_mode: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PathsSection {
    keypair_path: String,
    log_path: Option<String>,
    spaces_dir: Option<String>,
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
                log_path: Some(
                    dir.join("xgen-node.log").to_string_lossy().to_string(),
                ),
                spaces_dir: Some(
                    dir.join("spaces").to_string_lossy().to_string(),
                ),
            },
        }
    }
}

// ── CLI ────────────────────────────────────────────────────────────────────────

/// XGen Protocol Node — federated, identity-verified communication.
///
/// Run without a subcommand to start the Node in foreground mode (Phase 2).
/// Use subcommands to initialise, inspect, or query the running Node.
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
    /// Generate a keypair and default config next to the executable, then exit.
    /// Safe to run multiple times — will not overwrite an existing keypair.
    /// Prompts for a passphrase. Use empty passphrase for Local Node mode (Phase 1).
    Init,

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

fn main() {
    let cli = Cli::parse();
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| exe_dir().join("xgen-node_config.toml"));

    let result = match &cli.command {
        None => run_node(&cli),
        Some(NodeCommand::Init) => cmd_init(),
        Some(NodeCommand::Status) => cmd_status(&config_path),
        Some(NodeCommand::Connections) => cmd_connections(&config_path),
        Some(NodeCommand::Spaces) => cmd_spaces(&config_path),
        Some(NodeCommand::Peers) => cmd_peers(&config_path),
        Some(NodeCommand::Identity { action }) => match action {
            IdentityAction::List => cmd_identity_list(),
        },
        Some(NodeCommand::Version) => cmd_version(&config_path),
    };
    if let Err(e) = result {
        eprintln!("{}", red(&format!("error: {:#}", e)));
        std::process::exit(1);
    }
}

// ── run (no subcommand) ────────────────────────────────────────────────────────

fn run_node(_cli: &Cli) -> Result<()> {
    build_info::print_banner("xgen-node");
    println!();
    println!("Node runtime is Phase 2.");
    println!("  Use 'xgen-node init'   to initialise a new Node folder.");
    println!("  Use 'xgen-node status' to check if a Node is running.");
    Ok(())
}

// ── init ───────────────────────────────────────────────────────────────────────

fn cmd_init() -> Result<()> {
    let dir = exe_dir();
    let keypair_file = dir.join("xgen-node_keypair.enc");
    let config_file = dir.join("xgen-node_config.toml");

    if keypair_file.exists() {
        println!("Keypair already exists: {}", keypair_file.display());
        println!("Skipping keypair generation. Delete the file to regenerate.");
    } else {
        println!("Generating keypair...");
        let passphrase = prompt_passphrase()?;
        let signing_key = keypair::generate();
        keypair::save(&signing_key, &keypair_file, &passphrase)
            .context("failed to save keypair")?;
        println!("Keypair saved:  {}", keypair_file.display());
        println!("Node ID:        {}", pubkey_uri(&signing_key));
    }

    if config_file.exists() {
        println!("Config already exists: {} — not overwritten.", config_file.display());
    } else {
        let cfg = NodeConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).context("failed to serialise config")?;
        std::fs::write(&config_file, toml_str).context("failed to write config")?;
        println!("Config saved:   {}", config_file.display());
    }

    println!();
    println!("Run 'xgen-node' to start.");
    Ok(())
}

// ── status ─────────────────────────────────────────────────────────────────────

fn cmd_status(config_path: &Path) -> Result<()> {
    let state = load_state()?;
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
    let _ = config_path;
    Ok(())
}

// ── connections ────────────────────────────────────────────────────────────────

fn cmd_connections(_config_path: &Path) -> Result<()> {
    let state = load_state()?;

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

fn cmd_spaces(_config_path: &Path) -> Result<()> {
    let state = load_state()?;

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

fn cmd_peers(_config_path: &Path) -> Result<()> {
    let state = load_state()?;

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

fn cmd_identity_list() -> Result<()> {
    let identities_path = exe_dir().join("xgen-node_identities.db");

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

fn cmd_version(config_path: &Path) -> Result<()> {
    println!("xgen-node {}", build_info::full_version());
    println!("Commit:   {}", build_info::GIT_HASH);

    let cfg = try_load_config(config_path);
    let keypair_path = cfg
        .map(|c| c.paths.keypair_path)
        .unwrap_or_else(|| exe_dir().join("xgen-node_keypair.enc").to_string_lossy().to_string());
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

/// Directory of the running executable (D-020: Tier 1 files co-located with binary).
fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn try_load_config(path: &Path) -> Option<NodeConfig> {
    let text = std::fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()
}

/// Load the Node state file from the exe directory (Tier 1 — always co-located).
fn load_state() -> Result<NodeState> {
    let path = exe_dir().join("xgen-node_state.json");
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
