// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};

use xgen_common::{build_info, state::ClientState};

// ── Client config ──────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct ClientConfig {
    client: ClientSection,
    paths: PathsSection,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ClientSection {
    node: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PathsSection {
    keypair_path: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            client: ClientSection {
                node: "ws://127.0.0.1:8080/xgen".to_string(),
            },
            paths: PathsSection {
                keypair_path: "./xgen-client_keypair.enc".to_string(),
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

    /// Path to config file. Default: ./xgen-client_config.toml
    #[arg(short, long, default_value = "./xgen-client_config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: ClientCommand,
}

#[derive(Subcommand)]
enum ClientCommand {
    /// Generate a keypair and default config in the current directory, then exit.
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

    /// Register this Identity on the Node. Requires --node.
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

fn main() {
    let cli = Cli::parse();
    let result = match &cli.command {
        ClientCommand::Init => cmd_init(),
        ClientCommand::Whoami => cmd_whoami(&cli.config),
        ClientCommand::Status => cmd_status(&cli.config),
        ClientCommand::Spaces => cmd_spaces(&cli.config),
        ClientCommand::Version => cmd_version(),
        ClientCommand::Register(_)
        | ClientCommand::CreateSpace(_)
        | ClientCommand::CreateRoom(_)
        | ClientCommand::Invite(_)
        | ClientCommand::Join(_)
        | ClientCommand::Send(_)
        | ClientCommand::History(_)
        | ClientCommand::SmokeTest(_) => cmd_network_stub(&cli.command),
    };
    if let Err(e) = result {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

// ── init ───────────────────────────────────────────────────────────────────────

fn cmd_init() -> Result<()> {
    const KEYPAIR_FILE: &str = "./xgen-client_keypair.enc";
    const CONFIG_FILE: &str = "./xgen-client_config.toml";

    if Path::new(KEYPAIR_FILE).exists() {
        println!("Keypair already exists: {KEYPAIR_FILE}");
        println!("Skipping keypair generation. Delete the file to regenerate.");
    } else {
        println!("Generating keypair...");
        let passphrase = prompt_passphrase()?;
        let signing_key = keypair::generate();
        keypair::save(&signing_key, Path::new(KEYPAIR_FILE), &passphrase)
            .context("failed to save keypair")?;
        println!("Keypair saved:    {KEYPAIR_FILE}");
        println!("Identity ID: {}", keypair::pubkey_uri(&signing_key));
    }

    if Path::new(CONFIG_FILE).exists() {
        println!("Config already exists: {CONFIG_FILE} — not overwritten.");
    } else {
        let cfg = ClientConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).context("failed to serialise config")?;
        std::fs::write(CONFIG_FILE, toml_str).context("failed to write config")?;
        println!("Config saved:     {CONFIG_FILE}");
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
        println!("State file:    WARNING — updated {}s ago", age);
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

// ── network command stub ───────────────────────────────────────────────────────

fn cmd_network_stub(cmd: &ClientCommand) -> Result<()> {
    let name = match cmd {
        ClientCommand::Register(_) => "register",
        ClientCommand::CreateSpace(_) => "create-space",
        ClientCommand::CreateRoom(_) => "create-room",
        ClientCommand::Invite(_) => "invite",
        ClientCommand::Join(_) => "join",
        ClientCommand::Send(_) => "send",
        ClientCommand::History(_) => "history",
        ClientCommand::SmokeTest(_) => "smoke-test",
        _ => unreachable!(),
    };
    eprintln!("'{name}' requires a running xgen-node — available in Phase 2.");
    eprintln!("  To test the full protocol in Phase 1: cargo test smoke");
    std::process::exit(4);
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn base_dir(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

fn load_client_state(config_path: &Path) -> Result<ClientState> {
    let path = base_dir(config_path).join("xgen-client_state.json");
    if !path.exists() {
        bail!(
            "state file not found: {}\n  Run 'xgen-client init' and 'xgen-client register' first.",
            path.display()
        );
    }
    let json = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read state file: {}", path.display()))?;
    serde_json::from_str(&json).context("state file is corrupt or has an unexpected format")
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

// ── Inline keypair module ──────────────────────────────────────────────────────
//
// Phase 1: keypair logic is duplicated here because xgen-client does not depend
// on xgen-node. Phase 2 (D-022): this moves to xgen-core and is imported by both.

mod keypair {
    use std::path::Path;

    use anyhow::{Context, Result};
    use argon2::{Algorithm, Argon2, Params, Version};
    use base64::{engine::general_purpose, Engine};
    use chacha20poly1305::{
        aead::{Aead, KeyInit},
        ChaCha20Poly1305, Key, Nonce,
    };
    use ed25519_dalek::SigningKey;
    use rand::{rngs::OsRng, RngCore};
    use serde::{Deserialize, Serialize};

    const KDF_M_COST: u32 = 65536;
    const KDF_T_COST: u32 = 3;
    const KDF_P_COST: u32 = 1;

    #[derive(Serialize, Deserialize)]
    struct KeypairFile {
        version: u32,
        algorithm: String,
        kdf: String,
        salt: String,
        nonce: String,
        ciphertext: String,
    }

    pub fn generate() -> SigningKey {
        let mut secret = [0u8; 32];
        OsRng.fill_bytes(&mut secret);
        SigningKey::from_bytes(&secret)
    }

    pub fn save(signing_key: &SigningKey, path: &Path, passphrase: &str) -> Result<()> {
        let mut salt = [0u8; 32];
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);

        let enc_key = derive_key(passphrase, &salt)?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&enc_key));
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce_bytes),
                signing_key.as_bytes().as_ref(),
            )
            .map_err(|_| anyhow::anyhow!("encryption failed"))?;

        let file = KeypairFile {
            version: 1,
            algorithm: "ed25519".to_string(),
            kdf: "argon2id".to_string(),
            salt: encode(&salt),
            nonce: encode(&nonce_bytes),
            ciphertext: encode(&ciphertext),
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("failed to create keypair directory")?;
        }
        let json =
            serde_json::to_string_pretty(&file).context("failed to serialise keypair file")?;
        std::fs::write(path, json).context("failed to write keypair file")?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn load(path: &Path, passphrase: &str) -> Result<SigningKey> {
        let json = std::fs::read_to_string(path).context("failed to read keypair file")?;
        let file: KeypairFile =
            serde_json::from_str(&json).context("keypair file is corrupt")?;

        if file.version != 1 {
            anyhow::bail!("unsupported keypair file version: {}", file.version);
        }

        let salt = decode(&file.salt).context("invalid salt")?;
        let nonce_bytes = decode(&file.nonce).context("invalid nonce")?;
        let ciphertext = decode(&file.ciphertext).context("invalid ciphertext")?;

        let enc_key = derive_key(passphrase, &salt)?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&enc_key));
        let plaintext = cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("decryption failed — wrong passphrase?"))?;

        let key_bytes: [u8; 32] = plaintext
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid key bytes in file"))?;
        Ok(SigningKey::from_bytes(&key_bytes))
    }

    pub fn pubkey_uri(signing_key: &SigningKey) -> String {
        let encoded = encode(signing_key.verifying_key().as_bytes());
        format!("xgen://pubkey/ed25519:{}", encoded)
    }

    fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32]> {
        let params = Params::new(KDF_M_COST, KDF_T_COST, KDF_P_COST, Some(32))
            .map_err(|_| anyhow::anyhow!("invalid Argon2 parameters"))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(passphrase.as_bytes(), salt, &mut key)
            .map_err(|_| anyhow::anyhow!("key derivation failed"))?;
        Ok(key)
    }

    fn encode(bytes: &[u8]) -> String {
        general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    #[allow(dead_code)]
    fn decode(s: &str) -> Result<Vec<u8>> {
        general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .context("base64url decode failed")
    }
}
