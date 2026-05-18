// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! D-035 — Short-lived `xgen-client` CLI subcommands write their log file
//! under `<data_dir>/logs/`, not `<exe_dir>/logs/`. Pre-J-080 this site wrote
//! to `<exe_dir>/logs/` unconditionally, mixing logs from every `--instance`
//! invocation into one shared directory. The fix makes the short-lived CLI
//! symmetric with `--service`, `--ai-mode --service`, and the Tauri shell —
//! all of which already used `<data_dir>/logs/`. See J-080 carry-over #2.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CLIENT_EXE: &str = env!("CARGO_BIN_EXE_xgen-client");

fn instance_dir_for(label: &str) -> PathBuf {
    Path::new(CLIENT_EXE)
        .parent()
        .expect("CLIENT_EXE has parent")
        .join("instances")
        .join(label)
}

fn init_client(label: &str) {
    let status = Command::new(CLIENT_EXE)
        .args(["--instance", label, "init", "--passphrase="])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("xgen-client init");
    assert!(status.success(), "xgen-client init failed");
}

fn cleanup(label: &str) {
    let _ = std::fs::remove_dir_all(instance_dir_for(label));
}

fn count_logs(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map_or(false, |n| n.starts_with("xgen-client_") && n.ends_with(".log"))
                })
                .count()
        })
        .unwrap_or(0)
}

/// A short-lived CLI subcommand run with `--instance <label>` writes its log
/// under `<data_dir>/logs/` (which for `--instance` is
/// `<exe_dir>/instances/<label>/logs/`). The `init` subcommand goes through
/// the same `app::init_logging` codepath as every other short-lived CLI
/// subcommand (per `main.rs`), so we use it as the test trigger to avoid
/// additional setup. Asserting on count-difference for `<exe_dir>/logs/`
/// rather than equality, so any pre-existing files there from other test
/// runs don't break the assertion.
#[test]
fn short_lived_cli_log_lands_in_instance_logs_dir() {
    let label = "lp-cli";
    cleanup(label);

    let instance_logs = instance_dir_for(label).join("logs");
    let exe_logs = Path::new(CLIENT_EXE).parent().unwrap().join("logs");
    let exe_count_before = count_logs(&exe_logs);

    init_client(label);

    let instance_count = count_logs(&instance_logs);
    let exe_count_after = count_logs(&exe_logs);
    cleanup(label);

    assert!(
        instance_count >= 1,
        "short-lived CLI must write log under <data_dir>/logs/ ({}); found {} log files",
        instance_logs.display(),
        instance_count
    );
    assert_eq!(
        exe_count_after, exe_count_before,
        "short-lived CLI must NOT write to <exe_dir>/logs/ when --instance is set; \
         exe_dir log count changed from {} to {}",
        exe_count_before, exe_count_after
    );
}
