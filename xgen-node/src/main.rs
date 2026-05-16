// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Thin CLI dispatcher for xgen-node (D-063). Parses arguments, decides which
//! mode to enter, calls into `xgen_node_lib::app` or `::desktop`. No business
//! logic here.
//!
//! Mode selection (highest precedence first):
//!   - any read-only control flag (--check-config, --print-config, --pid)
//!       → handler, exit
//!   - any pipe-dependent control flag (--ping/--health/--stop/--reload-config/--batch)
//!       → stub message, exit (the Node-side pipe server is M2 work)
//!   - any subcommand → control mode handler, exit
//!   - `--service` → headless WS via `app::run_node()`
//!   - default → Tauri desktop shell via `desktop::run()` (spawns run_node
//!       alongside Tauri internally)

use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use xgen_common::build_info;
use xgen_node_lib::{
    app::{self, RunNodeOpts},
    desktop,
};

fn validate_instance_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 64
        && label.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// XGen Protocol Node — federated, identity-verified communication.
///
/// Run without a subcommand to start the Node in desktop mode (Tauri shell).
/// Pass `--service` to start in headless resident mode (WS server, no UI).
/// Use subcommands to initialise, inspect, or query a running Node.
#[derive(Parser)]
#[command(name = "xgen-node", version = build_info::VERSION)]
struct Cli {
    /// Path to config file. Default: <data dir>/xgen-node_config.toml
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Override: start in Local Node mode regardless of config setting
    #[arg(long)]
    local: bool,

    /// Force headless resident mode (no UI). Runs the WS server via
    /// `app::run_node()`. The escape hatch from default Tauri-desktop launch.
    #[arg(long)]
    service: bool,

    /// Instance label — segregates data and logs under <exe_dir>/instances/<label>.
    /// Valid before any subcommand (e.g. `--instance n1 init`) or as a global
    /// flag after a subcommand (`init --instance n1`); clap treats it as global.
    #[arg(long, global = true)]
    instance: Option<String>,

    /// Override the WS listener port. Only consulted when writing a fresh
    /// config file in desktop mode (run_node reads its port from config).
    #[arg(long)]
    port: Option<u16>,

    /// Override the effective logging level for this invocation. Wins over
    /// config and the XGEN_LOG env var. Examples: "info", "debug", "warn".
    #[arg(long, value_name = "LEVEL", global = true)]
    log_level: Option<String>,

    /// Suppress startup chatter on stdout (banner, "Listening on…" line).
    /// Errors still surface on stderr; structured logs are unaffected.
    #[arg(long, global = true)]
    quiet: bool,

    /// Validate the effective config, print OK or the first parse error, exit.
    /// Read-only, no pipe contact.
    #[arg(long)]
    check_config: bool,

    /// Print the effective config as TOML on stdout and exit. Read-only.
    #[arg(long)]
    print_config: bool,

    /// Print the resident PID (from `<data dir>/xgen-node.pid`) and exit.
    #[arg(long)]
    pid: bool,

    /// Round-trip a noop against the running resident's pipe and print the
    /// latency in milliseconds. Requires the M2 Node pipe server (stubbed).
    #[arg(long)]
    ping: bool,

    /// Ask the running resident for a one-line liveness summary.
    /// Requires the M2 Node pipe server (stubbed).
    #[arg(long)]
    health: bool,

    /// Signal the running resident to shut down gracefully.
    /// Requires the M2 Node pipe server (stubbed).
    #[arg(long)]
    stop: bool,

    /// Signal the running resident to reload its config.
    /// Requires the M2 Node pipe server (stubbed).
    #[arg(long)]
    reload_config: bool,

    /// Execute a batch command file (.xgb) against the running resident via
    /// pipe. Requires the M2 Node pipe server (stubbed).
    #[arg(long, value_name = "FILE")]
    batch: Option<PathBuf>,

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

    /// Print this Node's node_id (xgen://pubkey/...) by loading the local keypair.
    /// Does not require the resident to be running.
    Whoami,

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

fn resolve_data_dir(instance: &Option<String>) -> PathBuf {
    match instance {
        Some(label) => {
            if !validate_instance_label(label) {
                eprintln!(
                    "error: --instance label {:?} is invalid. \
                     Use only letters, digits, hyphens, and underscores (max 64 chars).",
                    label
                );
                std::process::exit(1);
            }
            app::exe_dir().join("instances").join(label)
        }
        None => app::exe_dir(),
    }
}

/// Per Joe's Phase 4 disposition: Node-side pipe-dependent flags (--ping,
/// --health, --stop, --reload-config, --batch) print this message and exit
/// non-zero. The Node pipe server lives in M2; the Client side, which already
/// has a pipe server, gets full implementations.
fn node_pipe_stub(flag: &str) -> Result<()> {
    bail!(
        "{} requires the M2 Node pipe server — not yet implemented",
        flag
    );
}

fn exit_with_result(r: Result<()>) -> ! {
    match r {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("{}", app::red(&format!("error: {:#}", e)));
            std::process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let data_dir = resolve_data_dir(&cli.instance);
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| data_dir.join("xgen-node_config.toml"));

    // ── Control-mode read-only flags (exit before any runtime is built) ────────
    if cli.check_config {
        exit_with_result(app::cmd_check_config(&config_path));
    }
    if cli.print_config {
        exit_with_result(app::cmd_print_config(&config_path));
    }
    if cli.pid {
        exit_with_result(app::cmd_pid(&data_dir));
    }
    if cli.ping {
        exit_with_result(node_pipe_stub("--ping"));
    }
    if cli.health {
        exit_with_result(node_pipe_stub("--health"));
    }
    if cli.stop {
        exit_with_result(node_pipe_stub("--stop"));
    }
    if cli.reload_config {
        exit_with_result(node_pipe_stub("--reload-config"));
    }
    if cli.batch.is_some() {
        exit_with_result(node_pipe_stub("--batch"));
    }

    // ── Resident-desktop branch (Tauri shell, synchronous) ─────────────────────
    // Tauri owns the main thread; cannot run under a tokio macro. Dispatched
    // before the tokio runtime is created. Desktop mode spawns `run_node`
    // alongside Tauri internally (D-062 / D-063).
    if cli.command.is_none() && !cli.service {
        desktop::run(
            config_path,
            data_dir,
            cli.port.unwrap_or(8080),
            cli.log_level.clone(),
        );
        return;
    }

    // ── tokio-driven control/headless modes ────────────────────────────────────
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let result = rt.block_on(async {
        match &cli.command {
            None => {
                // --service: headless WS server.
                app::run_node(
                    &config_path,
                    &data_dir,
                    RunNodeOpts {
                        local_override: cli.local,
                        init_logging: true,
                        quiet: cli.quiet,
                        log_level_override: cli.log_level.clone(),
                    },
                )
                .await
            }
            Some(NodeCommand::Init { passphrase }) => {
                app::cmd_init(&data_dir, passphrase.as_deref())
            }
            Some(NodeCommand::Status) => app::cmd_status(&data_dir),
            Some(NodeCommand::Whoami) => app::cmd_whoami(&config_path, &data_dir),
            Some(NodeCommand::Connections) => app::cmd_connections(&data_dir),
            Some(NodeCommand::Spaces) => app::cmd_spaces(&data_dir),
            Some(NodeCommand::Peers) => app::cmd_peers(&data_dir),
            Some(NodeCommand::Identity { action }) => match action {
                IdentityAction::List => app::cmd_identity_list(&data_dir),
            },
            Some(NodeCommand::Version) => app::cmd_version(&config_path, &data_dir),
        }
    });
    if let Err(e) = result {
        eprintln!("{}", app::red(&format!("error: {:#}", e)));
        std::process::exit(1);
    }
}
