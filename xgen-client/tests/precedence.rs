// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! D-068 — CLI flag precedence over config file. Integration tests for
//! `xgen-client`. See `tasks/CLI_PRECEDENCE_AUDIT.md` §7.3 and `JOURNAL.md`
//! J-079. Tests share `<exe_dir>/instances/` (per `app::exe_dir()` semantics)
//! with unique labels.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

const CLIENT_EXE: &str = env!("CARGO_BIN_EXE_xgen-client");

fn instance_dir_for(label: &str) -> PathBuf {
    Path::new(CLIENT_EXE)
        .parent()
        .expect("CLIENT_EXE has parent")
        .join("instances")
        .join(label)
}

fn write_client_config(dir: &Path, node: &str, level: &str, ai_section: Option<&str>) {
    let keypair_path = dir.join("xgen-client_keypair.enc");
    let mut content = format!(
        "[client]\nnode = \"{node}\"\n[paths]\nkeypair_path = '{}'\n[logging]\nlevel = \"{level}\"\n",
        keypair_path.to_string_lossy().replace('\\', "\\\\")
    );
    if let Some(ai) = ai_section {
        content.push_str(ai);
    }
    std::fs::write(dir.join("xgen-client_config.toml"), content).unwrap();
}

fn init_client(label: &str, ai: bool) {
    let mut args = vec!["--instance", label, "init", "--passphrase="];
    if ai {
        args.push("--ai");
    }
    let status = Command::new(CLIENT_EXE)
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("xgen-client init");
    assert!(status.success(), "xgen-client init failed");
}

fn cleanup(label: &str) {
    let _ = std::fs::remove_dir_all(instance_dir_for(label));
}

fn run_service_briefly(label: &str, extra_args: &[&str]) -> String {
    let mut args = vec!["--instance", label, "--service"];
    args.extend_from_slice(extra_args);
    let mut child = Command::new(CLIENT_EXE)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("xgen-client spawn");
    sleep(Duration::from_millis(1500));
    let _ = child.kill();
    let output = child.wait_with_output().expect("xgen-client wait");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn find_latest_log(instance_dir: &Path) -> Option<PathBuf> {
    let logs_dir = instance_dir.join("logs");
    let mut candidates: Vec<(PathBuf, std::time::SystemTime)> = std::fs::read_dir(&logs_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("xgen-client_") && n.ends_with(".log"))
        })
        .filter_map(|e| {
            let path = e.path();
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((path, mtime))
        })
        .collect();
    candidates.sort_by_key(|(_, mtime)| *mtime);
    candidates.pop().map(|(p, _)| p)
}

// ── Table A regressions ────────────────────────────────────────────────────

/// `--node <endpoint>` global flag overrides `[client].node` in config —
/// observable in the "Connecting to ..." line printed by any network
/// subcommand. Uses a deliberately unreachable port so the subcommand fails
/// fast.
#[test]
fn precedence_client_node_flag_beats_config() {
    let label = "p-node";
    cleanup(label);
    init_client(label, false);
    let instance_dir = instance_dir_for(label);
    write_client_config(&instance_dir, "ws://127.0.0.1:18080/xgen", "info", None);

    let with_flag = String::from_utf8(
        Command::new(CLIENT_EXE)
            .args([
                "--instance",
                label,
                "--node",
                "ws://127.0.0.1:19999/xgen",
                "register",
                "--name",
                "X",
            ])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let no_flag = String::from_utf8(
        Command::new(CLIENT_EXE)
            .args(["--instance", label, "register", "--name", "X"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    cleanup(label);

    assert!(
        with_flag.contains("Connecting to ws://127.0.0.1:19999/xgen"),
        "flag should override config node; stdout was:\n{with_flag}"
    );
    assert!(
        no_flag.contains("Connecting to ws://127.0.0.1:18080/xgen"),
        "config node should apply when flag absent; stdout was:\n{no_flag}"
    );
}

/// `--config <path>` reads the file the flag points at, not the default.
#[test]
fn precedence_client_config_flag_beats_default() {
    let label = "p-cfg";
    cleanup(label);
    init_client(label, false);
    let instance_dir = instance_dir_for(label);
    write_client_config(&instance_dir, "ws://127.0.0.1:18080/xgen", "info", None);
    let alt = instance_dir.join("alt-config.toml");
    std::fs::write(
        &alt,
        std::fs::read_to_string(instance_dir.join("xgen-client_config.toml"))
            .unwrap()
            .replace("level = \"info\"", "level = \"warn\""),
    )
    .unwrap();

    let default = String::from_utf8(
        Command::new(CLIENT_EXE)
            .args(["--instance", label, "--print-config"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let flagged = String::from_utf8(
        Command::new(CLIENT_EXE)
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

/// `--instance` directs `--print-config` at the per-instance config.
#[test]
fn precedence_client_instance_flag_beats_default() {
    let label = "p-inst";
    cleanup(label);
    init_client(label, false);
    let instance_dir = instance_dir_for(label);
    write_client_config(&instance_dir, "ws://127.0.0.1:18181/xgen", "info", None);

    let out = String::from_utf8(
        Command::new(CLIENT_EXE)
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

/// **The headline fix.** `xgen-client --service` log-level: config respected.
/// Pre-J-079 the binary ignored config and fell back to hardcoded "debug"
/// (§4.3.2 — 9 INFO lines despite `level = "error"`). Post-fix: 0 INFO lines.
#[test]
fn precedence_client_service_loglevel_respects_config() {
    let label = "p-log";
    cleanup(label);
    init_client(label, false);
    let instance_dir = instance_dir_for(label);
    write_client_config(&instance_dir, "ws://127.0.0.1:19999/xgen", "error", None);

    let _stdout = run_service_briefly(label, &[]);
    let log_file = find_latest_log(&instance_dir).expect("log file produced");
    let content = std::fs::read_to_string(&log_file).unwrap();
    cleanup(label);
    let info_count = content.lines().filter(|l| l.contains(" INFO ")).count();
    assert_eq!(
        info_count, 0,
        "config level=error must suppress INFO lines on --service (D-068 #2); log was:\n{content}"
    );
}

/// `--log-level` flag beats config on `--service` mode.
#[test]
fn precedence_client_service_loglevel_flag_beats_config() {
    let label = "p-logf";
    cleanup(label);
    init_client(label, false);
    let instance_dir = instance_dir_for(label);
    write_client_config(&instance_dir, "ws://127.0.0.1:19999/xgen", "error", None);

    let _stdout = run_service_briefly(label, &["--log-level", "info"]);
    let log_file = find_latest_log(&instance_dir).expect("log file produced");
    let content = std::fs::read_to_string(&log_file).unwrap();
    cleanup(label);
    let info_count = content.lines().filter(|l| l.contains(" INFO ")).count();
    assert!(
        info_count > 0,
        "flag --log-level info should produce INFO lines despite config error (D-068); log was:\n{content}"
    );
}

// ── Table B / negative-test row ────────────────────────────────────────────

/// `--ai-mode` without `--service` is a clap-level error — `requires =
/// "service"` rejects the invocation at parse time.
#[test]
fn precedence_client_aimode_requires_service() {
    let label = "p-ai1";
    cleanup(label);
    init_client(label, false);

    let output = Command::new(CLIENT_EXE)
        .args(["--instance", label, "--ai-mode"])
        .output()
        .unwrap();
    cleanup(label);
    assert!(!output.status.success(), "clap should reject --ai-mode without --service");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--service") || stderr.contains("required"),
        "clap error should mention the unmet requirement; stderr was:\n{stderr}"
    );
}

/// `--ai-mode --service` with no `[ai]` section in config must error rather
/// than silently fall back to a non-AI mode (D-068 spirit: the flag is the
/// runtime selector; config supplies the data the mode needs; if data is
/// missing, error cleanly). Observable via the WARN line in the log file.
#[test]
fn precedence_client_aimode_without_config_errors_cleanly() {
    let label = "p-ai2";
    cleanup(label);
    init_client(label, false); // init WITHOUT --ai, so no [ai] section
    let instance_dir = instance_dir_for(label);

    let _stdout = run_service_briefly(label, &["--ai-mode"]);
    let log_file = find_latest_log(&instance_dir).expect("log file produced");
    let content = std::fs::read_to_string(&log_file).unwrap();
    cleanup(label);
    assert!(
        content.contains("ai-mode requires [ai] section"),
        "expected the missing-[ai] WARN; log was:\n{content}"
    );
}
