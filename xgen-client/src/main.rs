// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Thin CLI dispatcher for xgen-client (D-063). Parses arguments via the lib's
//! `Cli` (which is also used by the batch-file executor), decides mode, calls
//! into `xgen_client_lib::app` or `::desktop`. No business logic here.
//!
//! Post-Phase-2a mode selection:
//!   - any subcommand                → control mode handler, exit
//!   - `--batch <file>`              → in-process batch executor, exit
//!   - `--service`                   → headless resident (stub until 2b/M3)
//!   - otherwise                     → Tauri desktop shell via `desktop::run()`

use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use xgen_client_lib::app::{self, Cli, ClientCommand};
use xgen_client_lib::{ai_service, desktop, service};
use xgen_common::{
    build_info,
    event_trace::{write_session_footer, ExitReason},
};

fn validate_instance_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 64
        && label.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
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

fn main() {
    let cli = Cli::parse();
    let data_dir = resolve_data_dir(&cli.instance);
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| data_dir.join("xgen-client_config.toml"));

    // ── Control-mode read-only flags (exit before any runtime is built) ───────
    if cli.check_config {
        match app::cmd_check_config(&config_path) {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("{}", app::red(&format!("error: {:#}", e)));
                std::process::exit(1);
            }
        }
    }
    if cli.print_config {
        match app::cmd_print_config(&config_path) {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("{}", app::red(&format!("error: {:#}", e)));
                std::process::exit(1);
            }
        }
    }
    if cli.pid {
        match app::cmd_pid(&data_dir) {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("{}", app::red(&format!("error: {:#}", e)));
                std::process::exit(1);
            }
        }
    }

    // ── Pipe-based control flags (read or signal the running resident) ────────
    #[cfg(target_os = "windows")]
    {
        let pipe = xgen_client_lib::batch::pipe_name(cli.instance.as_deref());
        if cli.ping {
            std::process::exit(xgen_client_lib::batch::cmd_ping(&pipe));
        }
        if cli.health {
            std::process::exit(xgen_client_lib::batch::cmd_health(&pipe));
        }
        if cli.stop {
            std::process::exit(xgen_client_lib::batch::cmd_stop(&pipe));
        }
        if cli.reload_config {
            std::process::exit(xgen_client_lib::batch::cmd_reload_config(&pipe));
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if cli.ping || cli.health || cli.stop || cli.reload_config {
            eprintln!("error: --ping/--health/--stop/--reload-config require Windows named pipes");
            std::process::exit(2);
        }
    }

    // ── Desktop branch (Tauri, synchronous, owns main thread) ─────────────────
    // Dispatched before any tokio runtime is created. Tauri builds its own.
    if cli.command.is_none() && !cli.service && cli.batch.is_none() {
        desktop::run(data_dir, cli.instance.clone(), cli.log_level.clone());
        return;
    }

    // ── Service mode — headless resident ──────────────────────────────────────
    // Long-running. Owns its own tokio runtime, tracing subscriber, PID file,
    // pipe server (Windows), and a best-effort sustained WS connection to the
    // home Node. Stays alive until Ctrl+C or `__STOP__` over the pipe.
    //
    // M4: `--ai-mode --service` routes to `ai_service::run` instead — the AI
    // Client resident. Same scaffold (logging, PID, pipe server, ctrl_c) but
    // a different inner loop that consumes inbound events via the configured
    // `AiBehavior` plugin and emits replies under pacing + mute constraints.
    if cli.service {
        if cli.ai_mode {
            ai_service::run(data_dir, cli.instance.clone(), cli.log_level.clone());
        } else {
            service::run(data_dir, cli.instance.clone(), cli.log_level.clone());
        }
        return;
    }

    // ── Logging init for control/batch modes ───────────────────────────────────
    // Desktop branch above does its own init via `desktop::run()`. Calling the
    // global tracing subscriber init in both branches would panic.
    app::init_logging(&config_path, cli.log_level.as_deref());
    app::write_client_session_header();

    // ── Build the tokio runtime for the remaining async paths ──────────────────
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    // ── Batch mode ─────────────────────────────────────────────────────────────
    if let Some(batch_path) = &cli.batch {
        let exit_code = rt.block_on(app::run_batch_file(
            batch_path,
            cli.node.as_deref(),
            &config_path,
            &data_dir,
        ));
        write_session_footer(ExitReason::Shutdown);
        std::process::exit(exit_code);
    }

    // cmd_stress_complete is a 900-line async fn whose state machine
    // exhausts the default tokio thread stack (2 MB). Dispatch it on a
    // dedicated OS thread with a 32 MB stack before the borrowing match below.
    if let Some(ClientCommand::StressComplete(args)) = cli.command {
        // Drop the runtime built above — the spawned thread builds its own.
        drop(rt);
        let result: Result<()> = std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("tokio runtime")
                    .block_on(app::cmd_stress_complete(&args))
            })
            .expect("thread spawn")
            .join()
            .expect("thread join");
        if let Err(ref e) = result {
            tracing::error!(reason = %format!("{:#}", e), "Fatal error");
            write_session_footer(ExitReason::Error);
            eprintln!("{}", app::red(&format!("error: {:#}", e)));
            std::process::exit(1);
        } else {
            write_session_footer(ExitReason::Shutdown);
        }
        return;
    }

    let result: Result<()> = rt.block_on(async {
        match &cli.command {
            None => {
                if !cli.quiet {
                    build_info::print_banner("xgen-client");
                    println!();
                    Cli::command().print_help().unwrap();
                    println!();
                }
                Ok(())
            }
            Some(ClientCommand::Init(args)) => app::cmd_init(args, &data_dir),
            Some(ClientCommand::Whoami) => app::cmd_whoami(&data_dir),
            Some(ClientCommand::Status) => app::cmd_status(&data_dir),
            Some(ClientCommand::Spaces) => app::cmd_spaces(&data_dir),
            Some(ClientCommand::Version) => app::cmd_version(),
            Some(ClientCommand::Register(args)) => {
                let node = app::resolve_node(cli.node.as_deref(), &config_path);
                let keypair_path = app::resolve_keypair_path(&config_path);
                let ai = app::load_ai_section(&config_path);
                app::cmd_register(args, &node, &keypair_path, &data_dir, ai.as_ref()).await
            }
            Some(ClientCommand::CreateSpace(args)) => {
                let node = app::resolve_node(cli.node.as_deref(), &config_path);
                let keypair_path = app::resolve_keypair_path(&config_path);
                app::cmd_create_space(args, &node, &keypair_path, &data_dir).await
            }
            Some(ClientCommand::CreateRoom(args)) => {
                let node = app::resolve_node(cli.node.as_deref(), &config_path);
                let keypair_path = app::resolve_keypair_path(&config_path);
                app::cmd_create_room(args, &node, &keypair_path, &data_dir).await
            }
            Some(ClientCommand::Invite(args)) => {
                let node = app::resolve_node(cli.node.as_deref(), &config_path);
                let keypair_path = app::resolve_keypair_path(&config_path);
                app::cmd_invite(args, &node, &keypair_path, &data_dir).await
            }
            Some(ClientCommand::Join(args)) => {
                let node = app::resolve_node(cli.node.as_deref(), &config_path);
                let keypair_path = app::resolve_keypair_path(&config_path);
                app::cmd_join(args, &node, &keypair_path, &data_dir).await
            }
            Some(ClientCommand::Send(args)) => {
                let node = app::resolve_node(cli.node.as_deref(), &config_path);
                let keypair_path = app::resolve_keypair_path(&config_path);
                app::cmd_send(args, &node, &keypair_path).await
            }
            Some(ClientCommand::History(args)) => {
                let node = app::resolve_node(cli.node.as_deref(), &config_path);
                let keypair_path = app::resolve_keypair_path(&config_path);
                app::cmd_history(args, &node, &keypair_path).await
            }
            Some(ClientCommand::SmokeTest(args)) => app::cmd_smoke_test(args).await,
            Some(ClientCommand::StressTest(args)) => app::cmd_stress_test(args).await,
            Some(ClientCommand::SmokePh2(args)) => app::cmd_smoke_ph2(args).await,
            Some(ClientCommand::StressComplete(_)) => unreachable!("handled above"),
            Some(ClientCommand::Ai(args)) => {
                let node = app::resolve_node(cli.node.as_deref(), &config_path);
                let keypair_path = app::resolve_keypair_path(&config_path);
                match &args.command {
                    xgen_client_lib::app::AiCommand::Delegate(a) => {
                        app::cmd_ai_delegate(a, &node, &keypair_path).await
                    }
                    xgen_client_lib::app::AiCommand::Revoke(a) => {
                        app::cmd_ai_revoke(a, &node, &keypair_path).await
                    }
                    xgen_client_lib::app::AiCommand::Status(a) => {
                        app::cmd_ai_status(a, &node, &keypair_path).await
                    }
                }
            }
        }
    });
    if let Err(ref e) = result {
        tracing::error!(reason = %format!("{:#}", e), "Fatal error");
        write_session_footer(ExitReason::Error);
        eprintln!("{}", app::red(&format!("error: {:#}", e)));
        std::process::exit(1);
    } else {
        write_session_footer(ExitReason::Shutdown);
    }
}
