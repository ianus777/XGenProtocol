// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! D-068 — CLI flag precedence over config file. Integration tests for
//! `xgen-node`. Each test spawns the actual binary and asserts on observable
//! behaviour (stdout, log file contents) — the level §4 of the audit ran
//! manually, now codified as regression locks. See
//! `tasks/CLI_PRECEDENCE_AUDIT.md` §7.2 and `JOURNAL.md` J-079.
//!
//! These tests bind real sockets and write real log files; they use
//! `--instance <unique-label>` in a per-test `TempDir` to keep state isolated
//! and high-numbered ports to minimise conflicts.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

const NODE_EXE: &str = env!("CARGO_BIN_EXE_xgen-node");

/// `app::exe_dir()` returns the directory containing the running binary —
/// `--instance LABEL` resolves to `<exe_dir>/instances/LABEL/`, *not*
/// relative to cwd. Mirror that resolution here so tests can locate the
/// per-instance config, logs, keypair, etc.
fn instance_dir_for(label: &str) -> PathBuf {
    Path::new(NODE_EXE)
        .parent()
        .expect("NODE_EXE has parent")
        .join("instances")
        .join(label)
}

fn write_node_config(dir: &Path, listen: &str, level: &str, local_mode: bool) {
    let keypair_path = dir.join("xgen-node_keypair.enc");
    let content = format!(
        "[node]\nlisten = \"{listen}\"\nlocal_mode = {local_mode}\n[paths]\nkeypair_path = '{}'\n[logging]\nlevel = \"{level}\"\n",
        keypair_path.to_string_lossy().replace('\\', "\\\\")
    );
    std::fs::write(dir.join("xgen-node_config.toml"), content).unwrap();
}

fn init_keypair(label: &str) {
    let status = Command::new(NODE_EXE)
        .args(["--instance", label, "init", "--passphrase="])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("xgen-node init");
    assert!(status.success(), "xgen-node init failed");
}

/// Best-effort cleanup of the per-instance dir created by `init_keypair`.
/// Tests share `<exe_dir>/instances/` so unique labels per test are essential.
fn cleanup(label: &str) {
    let _ = std::fs::remove_dir_all(instance_dir_for(label));
}

/// Spawn `xgen-node --service` with the given extra args, wait briefly for
/// the bind + banner emission, then kill it and return captured stdout.
fn run_service_briefly(label: &str, extra_args: &[&str]) -> String {
    let mut args = vec!["--instance", label, "--service"];
    args.extend_from_slice(extra_args);
    let mut child = Command::new(NODE_EXE)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("xgen-node spawn");
    // Wait long enough for bind + banner.
    sleep(Duration::from_millis(1500));
    let _ = child.kill();
    let output = child.wait_with_output().expect("xgen-node wait");
    String::from_utf8_lossy(&output.stdout).to_string()
}

// ── Table A regressions ────────────────────────────────────────────────────

/// J-078 reproduction lock — the case D-068 was written to eliminate.
/// Config says one port, flag says another; flag MUST win. Pre-fix the binary
/// bound the config port; post-fix it binds the flag port.
fn find_latest_log(instance_dir: &Path) -> PathBuf {
    let logs_dir = instance_dir.join("logs");
    std::fs::read_dir(&logs_dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", logs_dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |n| n.starts_with("xgen-node_") && n.ends_with(".log"))
        })
        .expect("log file produced")
}

#[test]
fn precedence_node_port_flag_beats_config() {
    let label = "p-port";
    cleanup(label);
    init_keypair(label);
    let instance_dir = instance_dir_for(label);
    write_node_config(&instance_dir, "ws://127.0.0.1:19591/xgen", "info", true);

    let stdout = run_service_briefly(label, &["--port", "19592"]);
    cleanup(label);
    assert!(
        stdout.contains("Listening on ws://127.0.0.1:19592/"),
        "flag should override config port; stdout was:\n{stdout}"
    );
    assert!(
        !stdout.contains("Listening on ws://127.0.0.1:19591/"),
        "config port must not be bound when flag is set; stdout was:\n{stdout}"
    );
}

/// No flag → config port wins. Locks the regression "the helper still defers
/// to config when flag is absent" — without this, a misfire of the helper
/// (e.g. an accidental `unwrap_or(8080)` in front of the resolver) would not
/// be caught by the J-078 reproduction alone.
#[test]
fn precedence_node_port_config_wins_when_flag_absent() {
    let label = "p-port2";
    cleanup(label);
    init_keypair(label);
    let instance_dir = instance_dir_for(label);
    write_node_config(&instance_dir, "ws://127.0.0.1:19593/xgen", "info", true);

    let stdout = run_service_briefly(label, &[]);
    cleanup(label);
    assert!(
        stdout.contains("Listening on ws://127.0.0.1:19593/"),
        "config port should bind when flag is absent; stdout was:\n{stdout}"
    );
}

/// --config flag picks the file `--print-config` reads. Cheap (no bind).
#[test]
fn precedence_node_config_flag_beats_default() {
    let label = "p-cfg";
    cleanup(label);
    init_keypair(label);
    let instance_dir = instance_dir_for(label);
    write_node_config(&instance_dir, "ws://127.0.0.1:18080/xgen", "info", true);
    let alt = instance_dir.join("alt-config.toml");
    let alt_content = std::fs::read_to_string(instance_dir.join("xgen-node_config.toml"))
        .unwrap()
        .replace("level = \"info\"", "level = \"warn\"");
    std::fs::write(&alt, alt_content).unwrap();

    let default = String::from_utf8(
        Command::new(NODE_EXE)
            .args(["--instance", label, "--print-config"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let flagged = String::from_utf8(
        Command::new(NODE_EXE)
            .args(["--instance", label, "--config", alt.to_str().unwrap(), "--print-config"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    cleanup(label);
    assert!(default.contains("level = \"info\""), "default config not read: {default}");
    assert!(flagged.contains("level = \"warn\""), "--config flag not respected: {flagged}");
}

/// --instance segregates data dir → --print-config reads from instance-local
/// config. Cheap (no bind).
#[test]
fn precedence_node_instance_flag_beats_default() {
    let label = "p-inst";
    cleanup(label);
    init_keypair(label);
    let instance_dir = instance_dir_for(label);
    write_node_config(&instance_dir, "ws://127.0.0.1:18181/xgen", "info", true);

    let out = String::from_utf8(
        Command::new(NODE_EXE)
            .args(["--instance", label, "--print-config"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    cleanup(label);
    assert!(
        out.contains("ws://127.0.0.1:18181/xgen"),
        "--instance did not direct config read to instance dir: {out}"
    );
}

/// D-068 regression lock (Node `run_node` path — was already compliant before
/// the audit; commit 3 refactored it onto the canonical helper). Config
/// level=error, no flag, no env → log file must have no INFO lines.
#[test]
fn precedence_node_loglevel_service_respects_config() {
    let label = "p-log";
    cleanup(label);
    init_keypair(label);
    let instance_dir = instance_dir_for(label);
    write_node_config(&instance_dir, "ws://127.0.0.1:19594/xgen", "error", true);

    let _stdout = run_service_briefly(label, &[]);
    let log_file = find_latest_log(&instance_dir);
    let content = std::fs::read_to_string(&log_file).unwrap();
    cleanup(label);
    let info_count = content.lines().filter(|l| l.contains(" INFO ")).count();
    assert_eq!(
        info_count, 0,
        "config level=error should suppress INFO lines (D-068); log was:\n{content}"
    );
}

/// --log-level flag beats config. Locks the flag tier on Node --service.
#[test]
fn precedence_node_loglevel_flag_beats_config() {
    let label = "p-logf";
    cleanup(label);
    init_keypair(label);
    let instance_dir = instance_dir_for(label);
    write_node_config(&instance_dir, "ws://127.0.0.1:19595/xgen", "error", true);

    let _stdout = run_service_briefly(label, &["--log-level", "info"]);
    let log_file = find_latest_log(&instance_dir);
    let content = std::fs::read_to_string(&log_file).unwrap();
    cleanup(label);
    let info_count = content.lines().filter(|l| l.contains(" INFO ")).count();
    assert!(
        info_count > 0,
        "flag --log-level info should produce INFO lines despite config error (D-068); log was:\n{content}"
    );
}
