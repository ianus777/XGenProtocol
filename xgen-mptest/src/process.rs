// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Process lifecycle (M9-D3 / Checkpoint #1).
//!
//! Spawns the real `xgen-node`/`xgen-client` binaries as separate OS processes,
//! with **kill-on-drop** so a panicking test never leaks a node, and
//! **instance-label namespacing** for per-run isolation.
//!
//! ## Isolation — grounded correction to "temp data dirs" (Checkpoint #1)
//! The design/runbook say "temp data dirs (`tempfile`)", but the binaries derive
//! their data dir from `exe_dir()/instances/<label>` (node `main.rs::resolve_data_dir`,
//! client likewise) — the **instance label is the single namespacing key** for
//! both the data dir *and* the named-pipe name. A `std::env::temp_dir()` path
//! cannot be injected without a binary code change (out of scope). So isolation
//! is achieved by a **unique instance label per actor per run**
//! (`mp-<scenario>-<role>-<nonce>`); the data lands under
//! `<exe_dir>/instances/<label>/` and RAII teardown deletes exactly that dir.
//! This needs zero binary change and matches how pipe names already work.
//!
//! ## Worker-thread pinning (Checkpoint #1)
//! The binaries build their runtime with `Builder::new_multi_thread()` and do
//! **not** call `.worker_threads()` — so tokio honors the `TOKIO_WORKER_THREADS`
//! env var. The harness sets it (default 2) on every spawned binary to avoid
//! scheduler thrash at scale (M9-D7).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::binloc::Binaries;
use crate::Result;
use anyhow::{anyhow, Context};

/// Which binary an actor maps to (drives the pipe-name prefix).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Node,
    Client,
}

impl Kind {
    fn pipe_prefix(self) -> &'static str {
        match self {
            // Mirror of `xgen-node::pipe::pipe_name` / `xgen-client::batch::pipe_name`
            // (D-043). Kept in sync by the Round-0 smokes connecting successfully.
            Kind::Node => r"\\.\pipe\xgen-node",
            Kind::Client => r"\\.\pipe\xgen-client",
        }
    }
}

/// The `.aicontrol` pipe path for a given binary kind + instance label.
/// Mirror of `aicontrol_pipe_name(pipe_name(Some(label)))` in both binaries.
pub fn aicontrol_pipe(kind: Kind, label: &str) -> String {
    format!("{}-{label}.aicontrol", kind.pipe_prefix())
}

/// The `.events` pipe path (`.aicontrol` + `.events`). Used by the C2 oracle.
pub fn events_pipe(kind: Kind, label: &str) -> String {
    format!("{}.events", aicontrol_pipe(kind, label))
}

/// The exe directory (parent of the located binaries). Instance data dirs live
/// under `<exe_dir>/instances/<label>` (mirror of `app::exe_dir()`).
pub fn exe_dir(bins: &Binaries) -> PathBuf {
    bins.node
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// The data dir a binary will use for a given instance label.
pub fn instance_data_dir(bins: &Binaries, label: &str) -> PathBuf {
    exe_dir(bins).join("instances").join(label)
}

/// Number of tokio worker threads to pin per spawned binary (M9-D7).
fn worker_threads() -> String {
    std::env::var("XGEN_MPTEST_WORKER_THREADS").unwrap_or_else(|_| "2".to_string())
}

/// A spawned binary process with kill-on-drop and instance-dir cleanup.
///
/// Construct with [`ManagedProcess::init_and_spawn_node`] /
/// [`ManagedProcess::init_and_spawn_client`]. On drop the child is killed and
/// (unless [`ManagedProcess::keep_artifacts`] was set) the instance data dir is
/// removed.
pub struct ManagedProcess {
    pub kind: Kind,
    pub label: String,
    pub data_dir: PathBuf,
    /// The `.aicontrol` pipe path for this process.
    pub aicontrol_pipe: String,
    child: Child,
    keep_artifacts: bool,
}

impl ManagedProcess {
    /// Run the binary's `init` subcommand (generates keypair + default config in
    /// the instance data dir) with an empty passphrase. Short-lived; waits.
    fn run_init(exe: &Path, label: &str) -> Result<()> {
        let status = base_command(exe)
            .args(["--instance", label, "init", "--passphrase", ""])
            .status()
            .with_context(|| format!("spawning `{} ... init`", exe.display()))?;
        if !status.success() {
            return Err(anyhow!(
                "`{} --instance {label} init` exited with {status}",
                exe.display()
            ));
        }
        Ok(())
    }

    /// `init` then `--service` spawn a node. `port` sets the WS listen port
    /// (`--port`); `local` forces Local-Node mode (`--local`, default-on for
    /// cooperative Round-0).
    pub fn init_and_spawn_node(
        bins: &Binaries,
        label: &str,
        port: u16,
        local: bool,
    ) -> Result<ManagedProcess> {
        Self::run_init(&bins.node, label)?;
        let mut cmd = base_command(&bins.node);
        cmd.args(["--instance", label, "--service", "--port"])
            .arg(port.to_string());
        if local {
            cmd.arg("--local");
        }
        let child = cmd
            .spawn()
            .with_context(|| format!("spawning node `{label}` --service on port {port}"))?;
        Ok(ManagedProcess {
            kind: Kind::Node,
            label: label.to_string(),
            data_dir: instance_data_dir(bins, label),
            aicontrol_pipe: aicontrol_pipe(Kind::Node, label),
            child,
            keep_artifacts: false,
        })
    }

    /// `init` then `--service` spawn a client resident. `node_url` is the home
    /// node WS endpoint (`--node`); `ai_mode` runs the AI resident
    /// (`--ai-mode --service`, the cooperative crowd).
    pub fn init_and_spawn_client(
        bins: &Binaries,
        label: &str,
        node_url: &str,
        ai_mode: bool,
    ) -> Result<ManagedProcess> {
        Self::run_init(&bins.client, label)?;
        let mut cmd = base_command(&bins.client);
        cmd.args(["--instance", label, "--service", "--node", node_url]);
        if ai_mode {
            cmd.arg("--ai-mode");
        }
        let child = cmd
            .spawn()
            .with_context(|| format!("spawning client `{label}` --service → {node_url}"))?;
        Ok(ManagedProcess {
            kind: Kind::Client,
            label: label.to_string(),
            data_dir: instance_data_dir(bins, label),
            aicontrol_pipe: aicontrol_pipe(Kind::Client, label),
            child,
            keep_artifacts: false,
        })
    }

    /// Retain the instance data dir on drop (for capture / debugging a failure).
    pub fn keep_artifacts(&mut self) {
        self.keep_artifacts = true;
    }

    /// Has the child already exited? (`Some(code)` if so.)
    pub fn try_exit_status(&mut self) -> Option<std::process::ExitStatus> {
        self.child.try_wait().ok().flatten()
    }
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        // Kill-on-drop: never leak a spawned binary, even on panic.
        let _ = self.child.kill();
        let _ = self.child.wait();
        if !self.keep_artifacts {
            let _ = std::fs::remove_dir_all(&self.data_dir);
        }
    }
}

/// A base [`Command`] for a binary: inherits no stdin, pins worker threads, and
/// silences child stdout/stderr (the harness reads state over pipes, not logs).
fn base_command(exe: &Path) -> Command {
    let mut cmd = Command::new(exe);
    cmd.env("TOKIO_WORKER_THREADS", worker_threads())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd
}

/// A short unique-enough nonce for instance labels, derived from the process id
/// and a monotonic counter (no `Date::now`/`rand` dep needed; the harness only
/// needs uniqueness within a run).
pub fn label_nonce() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{n}", std::process::id())
}

/// Build an instance label `mp-<scenario>-<role>-<nonce>`, sanitised to the
/// binaries' label rules (letters/digits/hyphens/underscores, ≤64 chars).
pub fn instance_label(scenario: &str, role: &str) -> String {
    let raw = format!("mp-{scenario}-{role}-{}", label_nonce());
    let cleaned: String = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();
    cleaned.chars().take(64).collect()
}

/// Helper for callers that build their own [`Command`] (e.g. the micro-benchmark
/// in C3) — exposes the base configuration.
pub fn configured_command(exe: impl AsRef<OsStr>) -> Command {
    base_command(Path::new(exe.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aicontrol_pipe_names_match_binary_convention() {
        assert_eq!(
            aicontrol_pipe(Kind::Node, "a"),
            r"\\.\pipe\xgen-node-a.aicontrol"
        );
        assert_eq!(
            aicontrol_pipe(Kind::Client, "alice"),
            r"\\.\pipe\xgen-client-alice.aicontrol"
        );
    }

    #[test]
    fn events_pipe_appends_events_suffix() {
        assert_eq!(
            events_pipe(Kind::Node, "a"),
            r"\\.\pipe\xgen-node-a.aicontrol.events"
        );
    }

    #[test]
    fn instance_label_is_sanitised_and_bounded() {
        let l = instance_label("MP-C-02", "alice");
        assert!(l.starts_with("mp-MP-C-02-alice-"));
        assert!(l.len() <= 64);
        assert!(l
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn label_nonce_is_unique_per_call() {
        let a = label_nonce();
        let b = label_nonce();
        assert_ne!(a, b);
    }
}
