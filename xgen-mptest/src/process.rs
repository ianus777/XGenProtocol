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

/// Number of tokio worker threads to pin per spawned binary (M9-D7), from the
/// `XGEN_MPTEST_WORKER_THREADS` env var (default `2`). This is the **fallback**
/// when the dial supplies no value (see [`resolve_worker_threads`]).
fn worker_threads_env() -> String {
    std::env::var("XGEN_MPTEST_WORKER_THREADS").unwrap_or_else(|_| "2".to_string())
}

/// Resolve the worker-thread count to pin (MP-R2-D3 / G-6): the dial's value
/// wins when `Some`, else the env fallback ([`worker_threads_env`]). Pure, so it
/// is unit-tested without spawning.
pub fn resolve_worker_threads(dial_value: Option<u32>) -> String {
    match dial_value {
        Some(n) => n.to_string(),
        None => worker_threads_env(),
    }
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
    /// What [`ManagedProcess::restart`] (MP-R2 C6b) needs to re-spawn this exact
    /// instance without re-`init` (so the data dir survives ⇒ replay-from-disk).
    respawn: RespawnSpec,
}

/// The re-spawn parameters [`ManagedProcess::restart`] replays — the binary +
/// its `--service` args, minus `init` (the data dir already exists).
enum RespawnSpec {
    Node {
        exe: PathBuf,
        port: u16,
        local: bool,
        worker_threads: Option<u32>,
    },
    Client {
        exe: PathBuf,
        node_url: String,
        ai_mode: bool,
        worker_threads: Option<u32>,
    },
}

impl ManagedProcess {
    /// Run the binary's `init` subcommand (generates keypair + default config in
    /// the instance data dir) with an empty passphrase. Short-lived; waits.
    fn run_init(exe: &Path, label: &str) -> Result<()> {
        // `init` is short-lived (generate keypair + config, then exit); the worker
        // count is irrelevant, so it uses the env default (None).
        let status = base_command(exe, None)
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
        worker_threads: Option<u32>,
    ) -> Result<ManagedProcess> {
        Self::run_init(&bins.node, label)?;
        // Align the node's *advertised* endpoint with its `--port` bind. `--port`
        // overrides the bind (D-068), but the federation handshake advertises
        // `[node].listen` (admin_ops `federation_initiate` dial-back hint), which
        // `init` leaves at the default `:8080`. Without this, a peer records the
        // wrong URL and `push_identity_to_peers` (identity re-home replication,
        // MP-C-06) dials an unreachable address. A real node configures `listen`
        // to its reachable address; the harness does the same here.
        set_node_listen_in_config(&instance_data_dir(bins, label), port)?;
        let mut cmd = base_command(&bins.node, worker_threads);
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
            respawn: RespawnSpec::Node {
                exe: bins.node.clone(),
                port,
                local,
                worker_threads,
            },
        })
    }

    /// `init` then `--service` spawn a client resident. `node_url` is the home
    /// node WS endpoint; `ai_mode` runs the AI resident (`--ai-mode --service`,
    /// the cooperative crowd).
    ///
    /// **Grounded (D-065):** the client `--aicontrol` ops resolve the node from
    /// **config** (`resolve_node(None, config)` — see `ops`/`aicontrol.rs`), NOT
    /// the launch `--node` flag (the plain `--service` resident's own WS reads
    /// config too; only the `--ai-mode` resident threads `--node` per C9). So the
    /// harness writes `[client] node = <url>` into the freshly-`init`ed config
    /// before spawning, and *also* passes `--node` (helps the ai-mode path).
    pub fn init_and_spawn_client(
        bins: &Binaries,
        label: &str,
        node_url: &str,
        ai_mode: bool,
        worker_threads: Option<u32>,
    ) -> Result<ManagedProcess> {
        Self::run_init(&bins.client, label)?;
        set_client_node_in_config(&instance_data_dir(bins, label), node_url)?;
        let mut cmd = base_command(&bins.client, worker_threads);
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
            respawn: RespawnSpec::Client {
                exe: bins.client.clone(),
                node_url: node_url.to_string(),
                ai_mode,
                worker_threads,
            },
        })
    }

    /// Spawn a client resident **reusing an existing actor's keypair** (M10.5 C1c /
    /// MP-C-06 re-home rail, D6 — key continuity + per-phase node retarget). Unlike
    /// [`init_and_spawn_client`] this does **not** run `init` (which would generate
    /// a *fresh* keypair = a different `identity_id`); it creates the new instance
    /// data dir, copies the keypair (and config) from `source_data_dir` so the
    /// re-home client presents the **same `identity_id`**, retargets
    /// `[client] node` to `node_url` (the new home), and spawns `--service`. The
    /// re-home batch then runs `register --re-registration` over its `.aicontrol`.
    /// The `source` actor's `ManagedProcess` must still be alive (its data dir not
    /// yet dropped) when this is called, so the keypair is present to copy.
    pub fn spawn_client_reusing_keypair(
        bins: &Binaries,
        label: &str,
        source_data_dir: &Path,
        node_url: &str,
        worker_threads: Option<u32>,
    ) -> Result<ManagedProcess> {
        let data_dir = instance_data_dir(bins, label);
        std::fs::create_dir_all(&data_dir)
            .with_context(|| format!("creating re-home data dir {}", data_dir.display()))?;
        // The keypair is the one artifact carrying the identity (same identity_id).
        let keypair = "xgen-client_keypair.enc";
        let src_kp = source_data_dir.join(keypair);
        std::fs::copy(&src_kp, data_dir.join(keypair)).with_context(|| {
            format!("copying re-home keypair from {}", src_kp.display())
        })?;
        // Copy the (full, init-shaped) config too, then retarget the node below —
        // robust against any required config field the bare node-only write omits.
        let config = "xgen-client_config.toml";
        let src_cfg = source_data_dir.join(config);
        if src_cfg.exists() {
            std::fs::copy(&src_cfg, data_dir.join(config)).with_context(|| {
                format!("copying re-home config from {}", src_cfg.display())
            })?;
        }
        set_client_node_in_config(&data_dir, node_url)?;
        let mut cmd = base_command(&bins.client, worker_threads);
        cmd.args(["--instance", label, "--service", "--node", node_url]);
        let child = cmd.spawn().with_context(|| {
            format!("spawning re-home client `{label}` --service → {node_url}")
        })?;
        Ok(ManagedProcess {
            kind: Kind::Client,
            label: label.to_string(),
            data_dir,
            aicontrol_pipe: aicontrol_pipe(Kind::Client, label),
            child,
            keep_artifacts: false,
            respawn: RespawnSpec::Client {
                exe: bins.client.clone(),
                node_url: node_url.to_string(),
                ai_mode: false,
                worker_threads,
            },
        })
    }

    /// Kill + re-spawn this exact instance **without re-`init`** (MP-R2 C6b /
    /// D5). The instance label, data dir, and `.aicontrol` pipe are preserved, so
    /// the node replays its persisted Spaces from disk (the MP-C-15 restart+replay
    /// durability path; the catch-up shape composes with C3's late-federation for
    /// cross-node rejoin). The new child replaces the old — a fresh `.aicontrol`
    /// connection is needed after (the old pipe server died with the old child).
    pub fn restart(&mut self) -> Result<()> {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let child = match &self.respawn {
            RespawnSpec::Node {
                exe,
                port,
                local,
                worker_threads,
            } => {
                let mut cmd = base_command(exe, *worker_threads);
                cmd.args(["--instance", &self.label, "--service", "--port"])
                    .arg(port.to_string());
                if *local {
                    cmd.arg("--local");
                }
                cmd.spawn()
                    .with_context(|| format!("restart node `{}`", self.label))?
            }
            RespawnSpec::Client {
                exe,
                node_url,
                ai_mode,
                worker_threads,
            } => {
                let mut cmd = base_command(exe, *worker_threads);
                cmd.args(["--instance", &self.label, "--service", "--node", node_url]);
                if *ai_mode {
                    cmd.arg("--ai-mode");
                }
                cmd.spawn()
                    .with_context(|| format!("restart client `{}`", self.label))?
            }
        };
        self.child = child;
        Ok(())
    }

    /// Retain the instance data dir on drop (for capture / debugging a failure).
    pub fn keep_artifacts(&mut self) {
        self.keep_artifacts = true;
    }

    /// Has the child already exited? (`Some(code)` if so.)
    pub fn try_exit_status(&mut self) -> Option<std::process::ExitStatus> {
        self.child.try_wait().ok().flatten()
    }

    /// The OS process id of the spawned binary (for resource sampling, C3).
    pub fn pid(&self) -> u32 {
        self.child.id()
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

/// Set `[node] listen = ws://127.0.0.1:<port>/xgen` in a node's
/// `xgen-node_config.toml`, preserving any other config. `init` writes the
/// default `:8080`; the node binds to `--port` (D-068) but advertises
/// `[node].listen` in the federation handshake (the `federation_initiate`
/// dial-back hint), so peers must record the reachable address — otherwise
/// `push_identity_to_peers` dials `:8080` and fails (MP-C-06 re-home replication).
fn set_node_listen_in_config(data_dir: &Path, port: u16) -> Result<()> {
    let path = data_dir.join("xgen-node_config.toml");
    let mut doc: toml::Value = match std::fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text)
            .unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new())),
        Err(_) => toml::Value::Table(toml::map::Map::new()),
    };
    let table = doc
        .as_table_mut()
        .ok_or_else(|| anyhow!("node config root is not a table: {}", path.display()))?;
    let node = table
        .entry("node".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let node_tbl = node
        .as_table_mut()
        .ok_or_else(|| anyhow!("[node] is not a table in {}", path.display()))?;
    node_tbl.insert(
        "listen".to_string(),
        toml::Value::String(format!("ws://127.0.0.1:{port}/xgen")),
    );
    let serialized = toml::to_string(&doc)
        .with_context(|| format!("serializing node config {}", path.display()))?;
    std::fs::write(&path, serialized)
        .with_context(|| format!("writing node config {}", path.display()))?;
    Ok(())
}

/// Set `[client] node = <url>` in a client's `xgen-client_config.toml`,
/// preserving any other config (e.g. an `[ai]` section from `init --ai`). The
/// client `--aicontrol` ops read the node from config, not the launch flag.
fn set_client_node_in_config(data_dir: &Path, node_url: &str) -> Result<()> {
    let path = data_dir.join("xgen-client_config.toml");
    let mut doc: toml::Value = match std::fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text)
            .unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new())),
        Err(_) => toml::Value::Table(toml::map::Map::new()),
    };
    let table = doc
        .as_table_mut()
        .ok_or_else(|| anyhow!("client config root is not a table: {}", path.display()))?;
    let client = table
        .entry("client".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let client_tbl = client
        .as_table_mut()
        .ok_or_else(|| anyhow!("[client] is not a table in {}", path.display()))?;
    client_tbl.insert("node".to_string(), toml::Value::String(node_url.to_string()));
    let serialized = toml::to_string(&doc)
        .with_context(|| format!("serializing client config {}", path.display()))?;
    std::fs::write(&path, serialized)
        .with_context(|| format!("writing client config {}", path.display()))?;
    Ok(())
}

/// A base [`Command`] for a binary: inherits no stdin, pins worker threads
/// (`worker_threads` from the dial, else the env fallback — MP-R2-D3 / G-6), and
/// silences child stdout/stderr (the harness reads state over pipes, not logs).
fn base_command(exe: &Path, worker_threads: Option<u32>) -> Command {
    let mut cmd = Command::new(exe);
    cmd.env("TOKIO_WORKER_THREADS", resolve_worker_threads(worker_threads))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // M12.2b (F9, S-3) — pin the data root to the exe dir. The binaries' new
    // default roots data at a platform dir (outside the install folder); the
    // harness, however, locates + tears down instances at
    // `<exe_dir>/instances/<label>` (`instance_data_dir`). Passing `--data-dir
    // <exe_dir>` (a global flag, valid before any subcommand) keeps the spawned
    // binary rooted where the harness expects — `exe.parent()` == `exe_dir(bins)`
    // for both binaries (same build dir). Single chokepoint: every spawn +
    // restart goes through here, so the pin can't be forgotten.
    if let Some(dir) = exe.parent() {
        cmd.arg("--data-dir").arg(dir);
    }
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
    base_command(Path::new(exe.as_ref()), None)
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

    #[test]
    fn worker_threads_prefers_dial_then_env() {
        // MP-R2-D3 / G-6: the dial's value wins when present.
        assert_eq!(resolve_worker_threads(Some(4)), "4");
        assert_eq!(resolve_worker_threads(Some(1)), "1");
        // None ⇒ the env fallback (default "2" when the var is unset). Set it
        // explicitly so the assertion does not depend on ambient process env.
        std::env::set_var("XGEN_MPTEST_WORKER_THREADS", "3");
        assert_eq!(resolve_worker_threads(None), "3");
        std::env::remove_var("XGEN_MPTEST_WORKER_THREADS");
        assert_eq!(resolve_worker_threads(None), "2");
    }
}
