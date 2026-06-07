// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Exe-location strategy (M9-D3 / Checkpoint #1).
//!
//! The design names `CARGO_BIN_EXE_*`, but that env is only injected for the
//! integration tests of the crate that *defines* the binary — `xgen-mptest` is
//! a separate workspace member, so those vars are **not** present here. The
//! grounded strategy instead resolves the build target directory and finds the
//! two `.exe`s inside it:
//!
//! 1. `XGEN_MPTEST_BIN_DIR` env — an explicit override (a directory holding the
//!    two `.exe`s). Highest priority; lets a runner point at `bin/` or a copy.
//! 2. `CARGO_TARGET_DIR` env + profile — if cargo's target-dir override is set.
//! 3. The workspace `.cargo/config.toml` `[build] target-dir` + profile — the
//!    project pins `C:/cargo-targets/XGenProtocol` there.
//! 4. The fallback known path `C:/cargo-targets/XGenProtocol` + profile.
//!
//! Profile defaults to `debug`; `XGEN_MPTEST_PROFILE=release` selects release.
//! The harness does **not** build the binaries itself (the runner is expected
//! to `cargo build -p xgen-node -p xgen-client` first) — it only *locates* them
//! and errors clearly if they are absent.

use std::path::{Path, PathBuf};

use crate::Result;
use anyhow::{anyhow, Context};

/// The two binaries the harness drives.
#[derive(Debug, Clone)]
pub struct Binaries {
    /// Absolute path to `xgen-node.exe` (or `xgen-node` on non-Windows).
    pub node: PathBuf,
    /// Absolute path to `xgen-client.exe` (or `xgen-client`).
    pub client: PathBuf,
}

/// The exe file stem → platform file name (`xgen-node` → `xgen-node.exe` on
/// Windows, `xgen-node` elsewhere).
fn exe_file_name(stem: &str) -> String {
    if cfg!(windows) {
        format!("{stem}.exe")
    } else {
        stem.to_string()
    }
}

/// The active build profile sub-directory (`debug` | `release`).
fn profile() -> String {
    std::env::var("XGEN_MPTEST_PROFILE").unwrap_or_else(|_| "debug".to_string())
}

/// Read the workspace `.cargo/config.toml` `[build] target-dir`, if present.
/// `manifest_dir` is `xgen-mptest`'s `CARGO_MANIFEST_DIR`; the workspace root is
/// its parent.
fn target_dir_from_cargo_config(manifest_dir: &Path) -> Option<PathBuf> {
    let workspace_root = manifest_dir.parent()?;
    let cfg = workspace_root.join(".cargo").join("config.toml");
    let text = std::fs::read_to_string(&cfg).ok()?;
    let value: toml::Value = text.parse().ok()?;
    let dir = value.get("build")?.get("target-dir")?.as_str()?;
    Some(PathBuf::from(dir))
}

/// Resolve the target directory (the dir that contains `debug/` and
/// `release/`), following the priority chain above. `manifest_dir` is normally
/// `env!("CARGO_MANIFEST_DIR")`.
fn resolve_target_dir(manifest_dir: &Path) -> PathBuf {
    if let Ok(d) = std::env::var("CARGO_TARGET_DIR") {
        return PathBuf::from(d);
    }
    if let Some(d) = target_dir_from_cargo_config(manifest_dir) {
        return d;
    }
    PathBuf::from("C:/cargo-targets/XGenProtocol")
}

/// Locate the built binaries. `manifest_dir` is the `xgen-mptest` crate dir
/// (`env!("CARGO_MANIFEST_DIR")`); the public [`locate`] passes it for you.
pub fn locate_in(manifest_dir: &Path) -> Result<Binaries> {
    // 1. Explicit bin-dir override.
    if let Ok(bin_dir) = std::env::var("XGEN_MPTEST_BIN_DIR") {
        let dir = PathBuf::from(bin_dir);
        return assemble(&dir);
    }
    // 2–4. target-dir + profile.
    let dir = resolve_target_dir(manifest_dir).join(profile());
    assemble(&dir)
}

/// Build a [`Binaries`] from a directory, verifying both exist.
fn assemble(dir: &Path) -> Result<Binaries> {
    let node = dir.join(exe_file_name("xgen-node"));
    let client = dir.join(exe_file_name("xgen-client"));
    if !node.exists() {
        return Err(anyhow!(
            "xgen-node binary not found at {} — run `cargo build -p xgen-node` \
             (or set XGEN_MPTEST_BIN_DIR / XGEN_MPTEST_PROFILE)",
            node.display()
        ));
    }
    if !client.exists() {
        return Err(anyhow!(
            "xgen-client binary not found at {} — run `cargo build -p xgen-client`",
            client.display()
        ));
    }
    Ok(Binaries { node, client })
}

/// Locate the built binaries using this crate's `CARGO_MANIFEST_DIR`.
pub fn locate() -> Result<Binaries> {
    locate_in(Path::new(env!("CARGO_MANIFEST_DIR")))
        .context("locating built xgen-node/xgen-client binaries")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exe_file_name_is_platform_correct() {
        let n = exe_file_name("xgen-node");
        if cfg!(windows) {
            assert_eq!(n, "xgen-node.exe");
        } else {
            assert_eq!(n, "xgen-node");
        }
    }

    #[test]
    fn cargo_config_target_dir_is_read_from_workspace_root() {
        // This crate's manifest dir is <workspace>/xgen-mptest; the project
        // pins target-dir in <workspace>/.cargo/config.toml. We assert the
        // reader finds *some* target-dir (the project ships one) — grounding
        // the priority chain step 3 without hard-coding the path value.
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dir = target_dir_from_cargo_config(manifest);
        assert!(
            dir.is_some(),
            "expected [build] target-dir in <workspace>/.cargo/config.toml"
        );
    }

    #[test]
    fn missing_bin_dir_errors_clearly() {
        let r = assemble(Path::new("Z:/definitely/not/here"));
        assert!(r.is_err());
        let msg = format!("{:#}", r.unwrap_err());
        assert!(msg.contains("xgen-node binary not found"), "msg was: {msg}");
    }
}
