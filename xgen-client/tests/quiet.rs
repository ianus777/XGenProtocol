// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Appendix F §F.0.1 — `--quiet` semantics: suppress chatty stdout
//! ("Connecting to ..." line on per-subcommand invocations). Result lines
//! and structured logs are unaffected. See J-080 (CARRY_OVER pass) for the
//! gating change.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CLIENT_EXE: &str = env!("CARGO_BIN_EXE_xgen-client");

/// M12.2b (F9, S-3) — a `Command` for the client binary with the data root pinned
/// to the exe dir (`XGEN_DATA_DIR`), so the binary's new platform default does
/// not move `<exe_dir>/instances/<label>` out from under these tests.
fn client_cmd() -> Command {
    let mut c = Command::new(CLIENT_EXE);
    c.env("XGEN_DATA_DIR", Path::new(CLIENT_EXE).parent().unwrap());
    c
}

fn instance_dir_for(label: &str) -> PathBuf {
    Path::new(CLIENT_EXE)
        .parent()
        .expect("CLIENT_EXE has parent")
        .join("instances")
        .join(label)
}

fn init_client(label: &str) {
    let status = client_cmd()
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

fn write_min_config(dir: &Path, node: &str) {
    let keypair_path = dir.join("xgen-client_keypair.enc");
    let content = format!(
        "[client]\nnode = \"{node}\"\n[paths]\nkeypair_path = '{}'\n[logging]\nlevel = \"info\"\n",
        keypair_path.to_string_lossy().replace('\\', "\\\\")
    );
    std::fs::write(dir.join("xgen-client_config.toml"), content).unwrap();
}

/// With `--quiet`, the per-subcommand "Connecting to ..." line MUST NOT appear
/// in stdout. The subcommand still attempts the network call (and will fail
/// against the unreachable port); we only assert on the suppressed line.
#[test]
fn quiet_suppresses_connecting_to_line() {
    let label = "q-on";
    cleanup(label);
    init_client(label);
    write_min_config(&instance_dir_for(label), "ws://127.0.0.1:19999/xgen");

    let stdout = String::from_utf8(
        client_cmd()
            .args(["--instance", label, "--quiet", "register", "--name", "X"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    cleanup(label);

    assert!(
        !stdout.contains("Connecting to"),
        "--quiet should suppress the Connecting-to line; stdout was:\n{stdout}"
    );
}

/// Without `--quiet`, the "Connecting to ..." line MUST appear. Mirror of the
/// negative case to lock the gate's positive direction. (The precedence.rs
/// suite also covers this via `--node` flag observation, but we keep this
/// here so the quiet contract is testable as a unit.)
#[test]
fn no_quiet_emits_connecting_to_line() {
    let label = "q-off";
    cleanup(label);
    init_client(label);
    write_min_config(&instance_dir_for(label), "ws://127.0.0.1:19999/xgen");

    let stdout = String::from_utf8(
        client_cmd()
            .args(["--instance", label, "register", "--name", "X"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    cleanup(label);

    assert!(
        stdout.contains("Connecting to ws://127.0.0.1:19999/xgen"),
        "without --quiet, the Connecting-to line should appear; stdout was:\n{stdout}"
    );
}
