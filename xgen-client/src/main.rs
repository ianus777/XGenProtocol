// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Thin CLI dispatcher for xgen-client (D-063). Parses arguments via the lib's
//! `Cli` (which is also used by the batch-file executor), decides mode, calls
//! into `xgen_client_lib::app`. No business logic here.

use anyhow::Result;
use clap::{CommandFactory, Parser};

use xgen_common::{
    build_info,
    event_trace::{ExitReason, write_session_footer},
};
use xgen_client_lib::app::{
    self, Cli, ClientCommand,
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| app::exe_dir().join("xgen-client_config.toml"));

    app::init_logging(&config_path);
    app::write_client_session_header();

    // ── Batch mode ─────────────────────────────────────────────────────────────
    if let Some(batch_path) = &cli.batch {
        let exit_code = app::run_batch_file(batch_path, cli.node.as_deref(), &config_path).await;
        write_session_footer(ExitReason::Shutdown);
        std::process::exit(exit_code);
    }

    // cmd_stress_complete is a 900-line async fn whose state machine
    // exhausts the default tokio thread stack (2 MB). Dispatch it on a
    // dedicated OS thread with a 32 MB stack before the borrowing match below.
    if let Some(ClientCommand::StressComplete(args)) = cli.command {
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

    let result: Result<()> = match &cli.command {
        None => {
            build_info::print_banner("xgen-client");
            println!();
            Cli::command().print_help().unwrap();
            println!();
            Ok(())
        }
        Some(ClientCommand::Init(args)) => app::cmd_init(args),
        Some(ClientCommand::Whoami) => app::cmd_whoami(&config_path),
        Some(ClientCommand::Status) => app::cmd_status(&config_path),
        Some(ClientCommand::Spaces) => app::cmd_spaces(&config_path),
        Some(ClientCommand::Version) => app::cmd_version(),
        Some(ClientCommand::Register(args)) => {
            let node = app::resolve_node(cli.node.as_deref(), &config_path);
            let keypair_path = app::resolve_keypair_path(&config_path);
            app::cmd_register(args, &node, &keypair_path).await
        }
        Some(ClientCommand::CreateSpace(args)) => {
            let node = app::resolve_node(cli.node.as_deref(), &config_path);
            let keypair_path = app::resolve_keypair_path(&config_path);
            app::cmd_create_space(args, &node, &keypair_path).await
        }
        Some(ClientCommand::CreateRoom(args)) => {
            let node = app::resolve_node(cli.node.as_deref(), &config_path);
            let keypair_path = app::resolve_keypair_path(&config_path);
            app::cmd_create_room(args, &node, &keypair_path).await
        }
        Some(ClientCommand::Invite(args)) => {
            let node = app::resolve_node(cli.node.as_deref(), &config_path);
            let keypair_path = app::resolve_keypair_path(&config_path);
            app::cmd_invite(args, &node, &keypair_path).await
        }
        Some(ClientCommand::Join(args)) => {
            let node = app::resolve_node(cli.node.as_deref(), &config_path);
            let keypair_path = app::resolve_keypair_path(&config_path);
            app::cmd_join(args, &node, &keypair_path).await
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
    };
    if let Err(ref e) = result {
        tracing::error!(reason = %format!("{:#}", e), "Fatal error");
        write_session_footer(ExitReason::Error);
        eprintln!("{}", app::red(&format!("error: {:#}", e)));
        std::process::exit(1);
    } else {
        write_session_footer(ExitReason::Shutdown);
    }
}
