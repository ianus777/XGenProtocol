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
        Self {
            node: NodeSection {
                listen: "ws://127.0.0.1:8080/xgen".to_string(),
                local_mode: true,
            },
            paths: PathsSection {
                keypair_path: "./xgen-node_keypair.enc".to_string(),
                log_path: Some("./xgen-node.log".to_string()),
                spaces_dir: Some("./spaces".to_string()),
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
    /// Path to config file. Default: ./xgen-node_config.toml
    #[arg(short, long, default_value = "./xgen-node_config.toml")]
    config: PathBuf,

    /// Override: start in Local Node mode regardless of config setting
    #[arg(long)]
    local: bool,

    #[command(subcommand)]
    command: Option<NodeCommand>,
}

#[derive(Subcommand)]
enum NodeCommand {
    /// Generate a keypair and default config in the current directory, then exit.
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
    let result = match &cli.command {
        None => run_node(&cli),
        Some(NodeCommand::Init) => cmd_init(),
        Some(NodeCommand::Status) => cmd_status(&cli.config),
        Some(NodeCommand::Connections) => cmd_connections(&cli.config),
        Some(NodeCommand::Spaces) => cmd_spaces(&cli.config),
        Some(NodeCommand::Peers) => cmd_peers(&cli.config),
        Some(NodeCommand::Identity { action }) => match action {
            IdentityAction::List => cmd_identity_list(&cli.config),
        },
        Some(NodeCommand::Version) => cmd_version(&cli.config),
    };
    if let Err(e) = result {
        eprintln!("error: {:#}", e);
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
    const KEYPAIR_FILE: &str = "./xgen-node_keypair.enc";
    const CONFIG_FILE: &str = "./xgen-node_config.toml";

    if Path::new(KEYPAIR_FILE).exists() {
        println!("Keypair already exists: {KEYPAIR_FILE}");
        println!("Skipping keypair generation. Delete the file to regenerate.");
    } else {
        println!("Generating keypair...");
        let passphrase = prompt_passphrase()?;
        let signing_key = keypair::generate();
        keypair::save(&signing_key, Path::new(KEYPAIR_FILE), &passphrase)
            .context("failed to save keypair")?;
        println!("Keypair saved:  {KEYPAIR_FILE}");
        println!("Node ID:        {}", pubkey_uri(&signing_key));
    }

    if Path::new(CONFIG_FILE).exists() {
        println!("Config already exists: {CONFIG_FILE} — not overwritten.");
    } else {
        let cfg = NodeConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).context("failed to serialise config")?;
        std::fs::write(CONFIG_FILE, toml_str).context("failed to write config")?;
        println!("Config saved:   {CONFIG_FILE}");
    }

    println!();
    println!("Run 'xgen-node' to start.");
    Ok(())
}

// ── status ─────────────────────────────────────────────────────────────────────

fn cmd_status(config_path: &Path) -> Result<()> {
    let state = load_state(config_path)?;
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
        println!("State file:   WARNING — updated {}s ago (Node may not be running)", age);
    } else {
        println!("State file:   updated {}s ago", age);
    }
    Ok(())
}

// ── connections ────────────────────────────────────────────────────────────────

fn cmd_connections(config_path: &Path) -> Result<()> {
    let state = load_state(config_path)?;

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

fn cmd_spaces(config_path: &Path) -> Result<()> {
    let state = load_state(config_path)?;

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

fn cmd_peers(config_path: &Path) -> Result<()> {
    let state = load_state(config_path)?;

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

fn cmd_identity_list(config_path: &Path) -> Result<()> {
    let identities_path = base_dir(config_path).join("xgen-node_identities.db");

    let registry = IdentityRegistry::load(&identities_path).with_context(|| {
        format!(
            "failed to load identity registry at {}\n  Is the Node initialised? Run 'xgen-node init'.",
            identities_path.display()
        )
    })?;

    let mut all = registry.all();
    // Sort by registered_at for stable output.
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
        .unwrap_or_else(|| "./xgen-node_keypair.enc".to_string());
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

fn base_dir(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

fn try_load_config(path: &Path) -> Option<NodeConfig> {
    let text = std::fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()
}

fn load_state(config_path: &Path) -> Result<NodeState> {
    let path = base_dir(config_path).join("xgen-node_state.json");
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

/// Truncate a full xgen:// URI for display in tables.
/// "xgen://hash/sha256:a3f9b2c1..." → "sha256:a3f9b2c1..."
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

/// "14m 22s ago" — appends " ago".
fn format_ago(secs: i64) -> String {
    format!("{} ago", fmt_duration(secs))
}

/// "14m ago" coarser format for identity registration age.
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
