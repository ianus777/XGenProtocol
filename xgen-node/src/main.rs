// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Thin CLI dispatcher for xgen-node (D-063). Parses arguments, decides which
//! mode to enter, calls into `xgen_node_lib::app`. No business logic here.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use xgen_common::build_info;
use xgen_node_lib::app;

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| app::exe_dir().join("xgen-node_config.toml"));
    // data_dir: Tier-1 runtime files live here.
    // No --config → exe_dir() (guaranteed by GetModuleFileNameW on Windows).
    // --config /path/to/cfg.toml → /path/to/ (user chose that deployment dir).
    let data_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(app::exe_dir);

    let result = match &cli.command {
        None => app::run_node(&config_path, &data_dir, cli.local).await,
        Some(NodeCommand::Init { passphrase }) => app::cmd_init(&data_dir, passphrase.as_deref()),
        Some(NodeCommand::Status) => app::cmd_status(&data_dir),
        Some(NodeCommand::Connections) => app::cmd_connections(&data_dir),
        Some(NodeCommand::Spaces) => app::cmd_spaces(&data_dir),
        Some(NodeCommand::Peers) => app::cmd_peers(&data_dir),
        Some(NodeCommand::Identity { action }) => match action {
            IdentityAction::List => app::cmd_identity_list(&data_dir),
        },
        Some(NodeCommand::Version) => app::cmd_version(&config_path, &data_dir),
    };
    if let Err(e) = result {
        eprintln!("{}", app::red(&format!("error: {:#}", e)));
        std::process::exit(1);
    }
}
