// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M7 `--aicontrol` — client command-pipe surface (Commit C2).
//!
//! A **sister** to `batch.rs`'s `--batch` named-pipe server (D-066: `--batch`
//! is untouched). It exposes the persistent JSONL control protocol over a
//! `…\.aicontrol` named pipe and wraps the **same** `xgen-client-lib::ops::*`
//! functions the batch arm calls — the only divergence is **envelope-out**
//! (the AC-D2 `Reply`) instead of plain-text-discard. No business logic is
//! forked: arguments are reconstructed into an argv and fed through the
//! existing `crate::app::Cli` clap parser (identical validation to the CLI /
//! `--batch`), then the parsed `ClientCommand` is dispatched to `ops::*` with
//! the structured result captured and serialised.
//!
//! Realisation of the checkpoint-#1 locks:
//! - **Sister pipe, second spawn.** `start_aicontrol_server` runs an
//!   independent accept loop on `pipe_name(label) + ".aicontrol"`; the three
//!   resident entry points spawn it alongside the existing `--batch` server.
//! - **Per-connection handler tasks.** The accept loop spawns one handler per
//!   connection and immediately creates the next pipe instance, so multiple
//!   `.aicontrol` connections are served concurrently (§2.3 "multiple
//!   connections allowed").
//! - **Serial per connection.** Each handler is a sequential read→dispatch→
//!   reply loop, so a single connection never has two commands in flight; an
//!   explicit in-flight guard backs the [`ControlCode::ConcurrentCommandNotAllowed`]
//!   path (the sequential loop makes it the safety-net, not the common path).
//! - **State-file serialization (locked C2-rider resolution).** The three
//!   `ops::*` verbs that read-modify-write `xgen-client_state.json`
//!   (`register` / `create-space` / `create-room`) run under a shared
//!   `Arc<tokio::Mutex<()>>` so two concurrent connections cannot lose an
//!   update; reads and non-mutating network verbs stay lock-free / concurrent.
//!
//! The pure dispatch helpers ([`dispatch_one`], [`build_state_data`],
//! [`reconstruct_argv`], …) are not `cfg`-gated so they unit-test on every
//! platform; the named-pipe server is Windows-only (D-043).

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde_json::{json, Map, Value};
use tokio::sync::Mutex;

use xgen_common::aicontrol::{
    parse_command, resolve_cmd, resolve_timeout_ms, substitute, Bindings, CmdResolution,
    ControlCode, ControlError, ControlVerb, ErrorBody, Reply, TimeoutTier,
};

use crate::ops::OpContext;
use crate::session::SessionState;

/// Shared state-file serialization lock. Held across any `ops::*` verb that
/// mutates `xgen-client_state.json` so concurrent `.aicontrol` connections
/// cannot race the file (locked C2-rider resolution).
pub type StateFileLock = Arc<Mutex<()>>;

/// Append `.aicontrol` to the `--batch` pipe name to form the sister pipe.
pub fn aicontrol_pipe_name(batch_pipe_name: &str) -> String {
    format!("{batch_pipe_name}.aicontrol")
}

// ── Dispatch error → envelope mapping (AC-D2) ──────────────────────────────────

/// An error produced while handling one command. Distinct from
/// [`ControlError`] because a *verb* error (an `ops::*` `anyhow`) maps to the
/// `protocol` category with the `GENERIC_4000` code (client `ops::*` is
/// `anyhow`-based — message-only, no `stage`), which a control-surface code
/// can never represent (AC-D3d invariant).
enum DispatchError {
    /// Control-surface failure (parse / binding / timeout / unknown verb / …).
    Control(ControlError),
    /// A client verb (`ops::*`) error — `anyhow` text only (AC-D2 client map).
    ClientVerb(String),
}

impl DispatchError {
    fn into_body(self, instance_state: &str) -> ErrorBody {
        match self {
            DispatchError::Control(ce) => ce.into_body(instance_state),
            DispatchError::ClientVerb(message) => ErrorBody {
                code: "GENERIC_4000".to_string(),
                category: xgen_common::aicontrol::Category::Protocol,
                message,
                instance_state: instance_state.to_string(),
                stage: None,
                hint: None,
            },
        }
    }
}

impl From<ControlError> for DispatchError {
    fn from(ce: ControlError) -> Self {
        DispatchError::Control(ce)
    }
}

// ── Verb classification ─────────────────────────────────────────────────────────

/// AC-D3a timeout tier for a client verb. The client has no federation verbs,
/// so only Read (local state/disk reads) and Write (home-Node round-trips)
/// apply; new verbs inherit their tier from this classification.
fn verb_tier(cmd: &str) -> TimeoutTier {
    match cmd {
        // Local reads of xgen-client_state.json — no network.
        "whoami" | "status" | "spaces" | "rooms" => TimeoutTier::Read,
        // Everything else is a home-Node round-trip (incl. history / ai status,
        // which read over the wire, and all writes).
        _ => TimeoutTier::Write,
    }
}

/// True for the verbs whose `ops::*` call read-modify-writes
/// `xgen-client_state.json` (the verbs that call `write_client_state`). These
/// run under the [`StateFileLock`] (locked C2-rider resolution). Kept precise
/// — over-locking would needlessly serialise concurrent network reads. Revisit
/// if `ops::*` grows a new state-file writer.
fn mutates_state_file(cmd: &str) -> bool {
    matches!(cmd, "register" | "create-space" | "create-room")
}

/// The result field a `bind` names as the bare-`$name` primary value (§5/§6).
/// Verbs without a documented primary fall back to the whole result object.
fn primary_field(cmd: &str) -> Option<&'static str> {
    match cmd {
        "register" => Some("identity_id"),
        "create-space" => Some("space_id"),
        "create-room" => Some("room_id"),
        "send" => Some("event_id"),
        "invite" | "join" => Some("space_id"),
        _ => None,
    }
}

// ── cmd + args → argv (reuse the clap parser) ───────────────────────────────────

/// Reconstruct a CLI argv from the JSONL `cmd` path + `args` object so the
/// existing `crate::app::Cli` clap parser does the verb + argument validation
/// (no forked parsing). `cmd` tokens lead; each `args` entry becomes a
/// `--kebab-flag value` pair (a `true` bool → a bare flag; `false`/`null` →
/// omitted; numbers → their compact form). Snake-case wire keys map to clap's
/// kebab long flags.
pub fn reconstruct_argv(cmd: &str, args: &Map<String, Value>) -> Vec<String> {
    let mut argv = vec!["xgen-client".to_string()];
    argv.extend(cmd.split_whitespace().map(str::to_string));
    for (key, value) in args {
        let flag = format!("--{}", key.replace('_', "-"));
        match value {
            Value::Bool(true) => argv.push(flag),
            Value::Bool(false) | Value::Null => {}
            Value::String(s) => {
                argv.push(flag);
                argv.push(s.clone());
            }
            other => {
                argv.push(flag);
                argv.push(other.to_string());
            }
        }
    }
    argv
}

/// Map a clap parse failure onto a control-surface error: an unrecognised
/// verb → `UNKNOWN_COMMAND`, anything else (missing / invalid argument) →
/// `BAD_ARGUMENT`.
fn map_clap_error(e: &clap::Error) -> ControlError {
    use clap::error::ErrorKind;
    match e.kind() {
        ErrorKind::InvalidSubcommand
        | ErrorKind::UnknownArgument
        | ErrorKind::MissingSubcommand
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            ControlError::new(ControlCode::UnknownCommand, "unrecognised command verb")
        }
        _ => ControlError::new(
            ControlCode::BadArgument,
            format!("invalid arguments: {}", e.kind()),
        ),
    }
}

// ── `state` control verb (AC-D3c client core) ──────────────────────────────────

/// The client lifecycle as derivable locally (no new instrumentation): a
/// configured + keyed instance is `ready`; otherwise it is still in `setup`.
/// Reuses the same on-disk first-run signals the desktop shell checks.
pub fn instance_lifecycle(data_dir: &Path) -> &'static str {
    let config = data_dir.join("xgen-client_config.toml");
    let keypair = data_dir.join("xgen-client_keypair.enc");
    if config.exists() && keypair.exists() {
        "ready"
    } else {
        "setup"
    }
}

/// Build the client `state` reply `data` (AC-D3c locked core). Composes from
/// on-disk `ClientState` + the `[ai]` config + the connection's live binding
/// namespace. Fields requiring the resident's live home-Node connection state
/// (`home_node_connected`, `connected_since`, per-space `member_count` /
/// `room_count`) are **dropped** in v1 — they are not threaded to the pipe
/// handler and adding them would be new instrumentation (AC-D3c guardrail);
/// recorded as a documented follow-up.
pub fn build_state_data(data_dir: &Path, bindings: &Bindings) -> Value {
    let config = data_dir.join("xgen-client_config.toml");
    let is_ai = crate::app::load_ai_section(&config)
        .map(|a| a.is_ai)
        .unwrap_or(false);

    let mut data = Map::new();
    data.insert("lifecycle".into(), json!(instance_lifecycle(data_dir)));
    data.insert("is_ai".into(), json!(is_ai));

    if let Ok(state) = crate::app::load_client_state(data_dir) {
        data.insert("identity_id".into(), json!(state.identity_id));
        data.insert("display_name".into(), json!(state.display_name));
        data.insert("home_node".into(), json!(state.home_node));
        data.insert("version".into(), json!(state.version));
        data.insert(
            "spaces".into(),
            serde_json::to_value(&state.spaces).unwrap_or(Value::Array(vec![])),
        );
    }

    // Control-owned, always present.
    data.insert("bindings".into(), Value::Object(bindings.snapshot()));
    // EV-D6 — live process-wide count of active client `.events` sessions
    // (C5); `0` until a driver subscribes.
    data.insert(
        "event_subscriptions".into(),
        json!(crate::events_pipe::active_session_count()),
    );

    Value::Object(data)
}

// ── Per-command dispatch ────────────────────────────────────────────────────────

/// Handle one JSONL command line and return the envelope [`Reply`] (the
/// transport-free core — unit-testable without a pipe). Parse → resolve →
/// substitute bindings → reconstruct argv → reuse clap → dispatch to `ops::*`
/// under the per-command timeout → serialise into the AC-D2 envelope. On a
/// successful command with `bind`, the result is recorded in `bindings`.
pub async fn dispatch_one(
    line: &str,
    data_dir: &Path,
    bindings: &mut Bindings,
    state_lock: &StateFileLock,
) -> Reply {
    let instance_state = instance_lifecycle(data_dir);

    // Parse. A malformed line has no cmd/id to echo (AC-D3d).
    let command = match parse_command(line) {
        Ok(c) => c,
        Err(ce) => return Reply::error(None, None, ce.into_body(instance_state)),
    };
    let cmd = command.cmd.trim().to_string();
    let id = command.id.clone();

    match dispatch_resolved(&command, &cmd, data_dir, bindings, state_lock).await {
        Ok(data) => {
            if let Some(name) = &command.bind {
                record_bind(bindings, name, &cmd, &data);
            }
            Reply::ok(cmd, id, data)
        }
        Err(de) => Reply::error(Some(cmd), id, de.into_body(instance_state)),
    }
}

async fn dispatch_resolved(
    command: &xgen_common::aicontrol::Command,
    cmd: &str,
    data_dir: &Path,
    bindings: &Bindings,
    state_lock: &StateFileLock,
) -> Result<Value, DispatchError> {
    // Reserved control verbs first (AC-D1).
    if let CmdResolution::Control(ControlVerb::State) = resolve_cmd(cmd) {
        return Ok(build_state_data(data_dir, bindings));
    }

    // Substitute $-bindings into a working copy of args, then peel off the
    // control-level `timeout_ms` (clap must not see it).
    let mut args = command.args.clone();
    substitute(&mut args, bindings)?;
    let timeout_override = args.remove("timeout_ms");
    let tier = verb_tier(cmd);
    let timeout_ms = resolve_timeout_ms(tier, timeout_override.as_ref())?;

    // Reuse the clap parser to validate the verb + arguments (no forked parse).
    let argv = reconstruct_argv(cmd, &args);
    let cli = <crate::app::Cli as clap::Parser>::try_parse_from(&argv)
        .map_err(|e| map_clap_error(&e))?;
    let client_cmd = cli
        .command
        .ok_or_else(|| ControlError::new(ControlCode::UnknownCommand, "no command verb"))?;

    run_cli_command(client_cmd, cmd, data_dir, timeout_ms, state_lock).await
}

/// Dispatch the parsed `ClientCommand` to the matching `ops::*`, capturing the
/// structured result as JSON. Wraps the call in the per-command timeout
/// (AC-D3a); state-mutating verbs additionally hold the [`StateFileLock`].
async fn run_cli_command(
    command: crate::app::ClientCommand,
    cmd: &str,
    data_dir: &Path,
    timeout_ms: u64,
    state_lock: &StateFileLock,
) -> Result<Value, DispatchError> {
    use crate::app::{AiCommand, ClientCommand};

    let dd = data_dir.to_path_buf();
    let config_path = dd.join("xgen-client_config.toml");
    let node = crate::app::resolve_node(None, &config_path);
    let keypair_path = crate::app::resolve_keypair_path(&config_path);

    // Each arm builds its own session (matches the batch arm) and returns the
    // result serialised to JSON. `to_value` is infallible for the `ops::*`
    // Result structs (all `Serialize`).
    let op: Pin<Box<dyn Future<Output = anyhow::Result<Value>> + Send + '_>> = match command {
        ClientCommand::Whoami => Box::pin(async move {
            let mut session = SessionState::new(String::new(), dd.clone());
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::whoami(&mut ctx)?)?)
        }),
        ClientCommand::Status => Box::pin(async move {
            let mut session = SessionState::new(String::new(), dd.clone());
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::status(&mut ctx)?)?)
        }),
        ClientCommand::Spaces => Box::pin(async move {
            let mut session = SessionState::new(String::new(), dd.clone());
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::spaces(&mut ctx)?)?)
        }),
        ClientCommand::Rooms(a) => Box::pin(async move {
            let mut session = SessionState::new(String::new(), dd.clone());
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::rooms(&mut ctx, &a)?)?)
        }),
        ClientCommand::Register(a) => Box::pin(async move {
            let ai = crate::app::load_ai_section(&config_path);
            let mut session = SessionState::new(node, dd.clone());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::register(&mut ctx, &a, ai.as_ref()).await?)?)
        }),
        ClientCommand::CreateSpace(a) => Box::pin(async move {
            let mut session = SessionState::new(node, dd.clone());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::create_space(&mut ctx, &a).await?)?)
        }),
        ClientCommand::CreateRoom(a) => Box::pin(async move {
            let mut session = SessionState::new(node, dd.clone());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::create_room(&mut ctx, &a).await?)?)
        }),
        ClientCommand::Invite(a) => Box::pin(async move {
            let mut session = SessionState::new(node, dd.clone());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::invite(&mut ctx, &a).await?)?)
        }),
        ClientCommand::Join(a) => Box::pin(async move {
            let mut session = SessionState::new(node, dd.clone());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::join(&mut ctx, &a).await?)?)
        }),
        ClientCommand::Leave(a) => Box::pin(async move {
            let mut session = SessionState::new(node, dd.clone());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::leave(&mut ctx, &a).await?)?)
        }),
        ClientCommand::Send(a) => Box::pin(async move {
            let mut session = SessionState::new(node, dd.clone());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::send(&mut ctx, &a).await?)?)
        }),
        ClientCommand::History(a) => Box::pin(async move {
            let mut session = SessionState::new(node, dd.clone());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::history(&mut ctx, &a).await?)?)
        }),
        ClientCommand::Members(a) => Box::pin(async move {
            let mut session = SessionState::new(node, dd.clone());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            Ok(serde_json::to_value(crate::ops::members(&mut ctx, &a).await?)?)
        }),
        ClientCommand::Ai(a) => Box::pin(async move {
            let mut session = SessionState::new(node, dd.clone());
            session.ensure_identity(&keypair_path)?;
            let mut ctx = OpContext { session: &mut session, data_dir: &dd, node_override: None };
            let v = match a.command {
                AiCommand::Delegate(x) => serde_json::to_value(crate::ops::ai_delegate(&mut ctx, &x).await?)?,
                AiCommand::Revoke(x) => serde_json::to_value(crate::ops::ai_revoke(&mut ctx, &x).await?)?,
                AiCommand::Status(x) => serde_json::to_value(crate::ops::ai_status(&mut ctx, &x).await?)?,
            };
            Ok(v)
        }),
        // Not exposed by --aicontrol v1 (not in the 14 ops::* verbs, AC-D5):
        // init / version / the long-running test commands.
        _ => {
            return Err(DispatchError::Control(ControlError::new(
                ControlCode::UnknownCommand,
                "command is not available over --aicontrol",
            )))
        }
    };

    let needs_lock = mutates_state_file(cmd);
    let timeout = tokio::time::Duration::from_millis(timeout_ms);
    let guarded = async move {
        let _state_guard = if needs_lock {
            Some(state_lock.lock().await)
        } else {
            None
        };
        op.await
    };
    match tokio::time::timeout(timeout, guarded).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(DispatchError::ClientVerb(format!("{e:#}"))),
        Err(_) => Err(DispatchError::Control(ControlError::new(
            ControlCode::Timeout,
            format!("command exceeded its {timeout_ms} ms timeout"),
        ))),
    }
}

/// Record a successful command's result under `name` (§5): `primary` is the
/// verb's documented primary field (else the whole object), `fields` is the
/// full result object for `$name.field` access.
fn record_bind(bindings: &mut Bindings, name: &str, cmd: &str, data: &Value) {
    let fields = data.as_object().cloned().unwrap_or_default();
    let primary = primary_field(cmd)
        .and_then(|k| fields.get(k).cloned())
        .unwrap_or_else(|| data.clone());
    bindings.set(name, primary, fields);
}

// ── Named-pipe server (Windows only, D-043) ─────────────────────────────────────

/// Start the `.aicontrol` command-pipe server. Independent of the `--batch`
/// server (D-066): its own accept loop on the sister pipe name, spawning one
/// handler task per connection so multiple drivers connect concurrently. Stops
/// when `shutdown_rx` delivers `true`.
#[cfg(target_os = "windows")]
pub async fn start_aicontrol_server(
    pipe_name_str: String,
    data_dir: PathBuf,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    state_lock: StateFileLock,
) {
    use tokio::net::windows::named_pipe::ServerOptions;

    tracing::info!(pipe = %pipe_name_str, "aicontrol pipe server starting");
    let mut first = true;

    loop {
        if *shutdown_rx.borrow() {
            break;
        }
        let server = {
            let create = if first {
                first = false;
                ServerOptions::new()
                    .first_pipe_instance(true)
                    .create(&pipe_name_str)
            } else {
                ServerOptions::new().create(&pipe_name_str)
            };
            match create {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(error = %e, "aicontrol pipe create failed — server stopping");
                    break;
                }
            }
        };

        let connected = tokio::select! {
            r = server.connect() => r.is_ok(),
            _ = shutdown_rx.changed() => false,
        };
        if !connected || *shutdown_rx.borrow() {
            break;
        }

        // Spawn the per-connection handler and immediately loop to create the
        // next pipe instance (multiple connections served concurrently).
        let handler_dir = data_dir.clone();
        let handler_lock = state_lock.clone();
        tokio::spawn(async move {
            handle_aicontrol_connection(server, handler_dir, handler_lock).await;
        });
    }

    tracing::info!(pipe = %pipe_name_str, "aicontrol pipe server stopped");
}

/// One persistent JSONL session: read a line, dispatch it, write the envelope
/// reply, repeat until the connection closes. Owns its own binding namespace
/// and serial in-flight guard (the sequential loop keeps the connection
/// strictly serial; the guard backs `CONCURRENT_COMMAND_NOT_ALLOWED`).
#[cfg(target_os = "windows")]
async fn handle_aicontrol_connection(
    server: tokio::net::windows::named_pipe::NamedPipeServer,
    data_dir: PathBuf,
    state_lock: StateFileLock,
) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader_half, mut writer_half) = tokio::io::split(server);
    let mut reader = BufReader::new(reader_half);
    let mut bindings = Bindings::new();
    let in_flight = AtomicBool::new(false);
    let mut buf = String::new();

    loop {
        buf.clear();
        match reader.read_line(&mut buf).await {
            Ok(0) => break, // connection closed by driver
            Ok(_) => {
                let line = buf.trim_end_matches('\n').trim_end_matches('\r');
                if line.trim().is_empty() {
                    continue;
                }
                // Serial guard. The sequential loop never reads ahead, so this
                // is a safety-net for any future pipelined handler rather than
                // a path the v1 loop reaches.
                let reply = if in_flight.swap(true, Ordering::SeqCst) {
                    Reply::error(
                        None,
                        None,
                        ControlError::new(
                            ControlCode::ConcurrentCommandNotAllowed,
                            "a command is already in flight on this connection",
                        )
                        .into_body(instance_lifecycle(&data_dir)),
                    )
                } else {
                    let r = dispatch_one(line, &data_dir, &mut bindings, &state_lock).await;
                    in_flight.store(false, Ordering::SeqCst);
                    r
                };
                let mut out = reply.to_line();
                out.push('\n');
                if writer_half.write_all(out.as_bytes()).await.is_err() {
                    break;
                }
                let _ = writer_half.flush().await;
            }
            Err(e) => {
                tracing::warn!(error = %e, "aicontrol pipe read error");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn lock() -> StateFileLock {
        Arc::new(Mutex::new(()))
    }

    fn write_state_fixture(dir: &Path) {
        // A minimal valid xgen-client_state.json so local read ops succeed.
        let json = r#"{"identity_id":"xgen://pubkey/ed25519:ID","display_name":"Alice","version":"0.10.3","build":"test","home_node":"ws://127.0.0.1:8080/xgen","updated_at":"2026-06-01T00:00:00Z","spaces":[{"space_id":"xgen://hash/sha256:S","name":"Demo","node_endpoint":"ws://127.0.0.1:8080/xgen","role":"owner","rooms":[]}]}"#;
        std::fs::write(dir.join("xgen-client_state.json"), json).unwrap();
        // config + keypair existence → lifecycle "ready"
        std::fs::write(dir.join("xgen-client_config.toml"), "").unwrap();
        std::fs::write(dir.join("xgen-client_keypair.enc"), "x").unwrap();
    }

    #[test]
    fn argv_reconstruction_snake_keys_to_kebab_flags() {
        let mut args = Map::new();
        args.insert("space".into(), json!("S"));
        args.insert("human_pacing_ms".into(), json!(2000));
        let argv = reconstruct_argv("create-space", &args);
        assert_eq!(argv[0], "xgen-client");
        assert_eq!(argv[1], "create-space");
        assert!(argv.contains(&"--space".to_string()));
        assert!(argv.contains(&"S".to_string()));
        assert!(argv.contains(&"--human-pacing-ms".to_string()));
        assert!(argv.contains(&"2000".to_string()));
    }

    #[test]
    fn argv_reconstruction_two_token_ai_verb() {
        let mut args = Map::new();
        args.insert("space".into(), json!("S"));
        args.insert("ai".into(), json!("A"));
        let argv = reconstruct_argv("ai delegate", &args);
        assert_eq!(&argv[1..3], &["ai".to_string(), "delegate".to_string()]);
    }

    #[tokio::test]
    async fn state_verb_returns_locked_core() {
        let dir = tempfile::tempdir().unwrap();
        write_state_fixture(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one(r#"{"cmd":"state"}"#, dir.path(), &mut b, &lock()).await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["status"], "ok");
        assert_eq!(v["cmd"], "state");
        assert_eq!(v["data"]["lifecycle"], "ready");
        assert_eq!(v["data"]["identity_id"], "xgen://pubkey/ed25519:ID");
        assert_eq!(v["data"]["display_name"], "Alice");
        assert_eq!(v["data"]["is_ai"], false);
        assert_eq!(v["data"]["event_subscriptions"], 0);
        assert!(v["data"].get("bindings").is_some());
        // Dropped-in-v1 fields are absent (AC-D3c no-new-instrumentation).
        assert!(v["data"].get("home_node_connected").is_none());
    }

    #[tokio::test]
    async fn whoami_happy_path_round_trips_through_envelope() {
        let dir = tempfile::tempdir().unwrap();
        write_state_fixture(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one(r#"{"cmd":"whoami","id":"c1"}"#, dir.path(), &mut b, &lock()).await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["status"], "ok");
        assert_eq!(v["cmd"], "whoami");
        assert_eq!(v["id"], "c1");
        assert_eq!(v["data"]["identity_id"], "xgen://pubkey/ed25519:ID");
        assert_eq!(v["data"]["spaces_joined"], 1);
    }

    #[tokio::test]
    async fn malformed_command_omits_cmd() {
        let dir = tempfile::tempdir().unwrap();
        write_state_fixture(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one("not json", dir.path(), &mut b, &lock()).await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["status"], "error");
        assert!(v.get("cmd").is_none());
        assert_eq!(v["error"]["code"], "MALFORMED_COMMAND");
        assert_eq!(v["error"]["category"], "argument");
        assert_eq!(v["error"]["instance_state"], "ready");
    }

    #[tokio::test]
    async fn unknown_verb_is_unknown_command() {
        let dir = tempfile::tempdir().unwrap();
        write_state_fixture(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one(r#"{"cmd":"frobnicate"}"#, dir.path(), &mut b, &lock()).await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["error"]["code"], "UNKNOWN_COMMAND");
        assert_eq!(v["error"]["category"], "argument");
        assert_eq!(v["cmd"], "frobnicate");
    }

    #[tokio::test]
    async fn missing_required_arg_is_bad_argument() {
        let dir = tempfile::tempdir().unwrap();
        write_state_fixture(dir.path());
        let mut b = Bindings::new();
        // rooms requires --space.
        let reply = dispatch_one(r#"{"cmd":"rooms","args":{}}"#, dir.path(), &mut b, &lock()).await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["error"]["code"], "BAD_ARGUMENT");
    }

    #[tokio::test]
    async fn bad_timeout_ms_is_bad_argument() {
        let dir = tempfile::tempdir().unwrap();
        write_state_fixture(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one(
            r#"{"cmd":"whoami","args":{"timeout_ms":0}}"#,
            dir.path(),
            &mut b,
            &lock(),
        )
        .await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["error"]["code"], "BAD_ARGUMENT");
        assert_eq!(v["error"]["category"], "argument");
    }

    #[tokio::test]
    async fn ops_error_maps_to_generic_4000_protocol_message_only() {
        // No state fixture → whoami's load_client_state errors (anyhow).
        let dir = tempfile::tempdir().unwrap();
        let mut b = Bindings::new();
        let reply = dispatch_one(r#"{"cmd":"whoami"}"#, dir.path(), &mut b, &lock()).await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["status"], "error");
        assert_eq!(v["error"]["code"], "GENERIC_4000");
        assert_eq!(v["error"]["category"], "protocol");
        assert!(v["error"].get("stage").is_none(), "client errors carry no stage");
        // instance_state reflects the un-set-up instance (no config/keypair).
        assert_eq!(v["error"]["instance_state"], "setup");
    }

    #[tokio::test]
    async fn bind_then_substitute_across_two_commands() {
        let dir = tempfile::tempdir().unwrap();
        write_state_fixture(dir.path());
        let mut b = Bindings::new();
        // rooms returns space_id; bind it, then reference it in a second rooms.
        let r1 = dispatch_one(
            r#"{"cmd":"rooms","args":{"space":"xgen://hash/sha256:S"},"bind":"sp"}"#,
            dir.path(),
            &mut b,
            &lock(),
        )
        .await;
        let v1: Value = serde_json::from_str(&r1.to_line()).unwrap();
        assert_eq!(v1["status"], "ok");
        // binding recorded under primary "space_id"? rooms isn't a primary verb,
        // so $sp resolves to the whole result object; $sp.space_id reaches it.
        let r2 = dispatch_one(
            r#"{"cmd":"rooms","args":{"space":"$sp.space_id"}}"#,
            dir.path(),
            &mut b,
            &lock(),
        )
        .await;
        let v2: Value = serde_json::from_str(&r2.to_line()).unwrap();
        assert_eq!(v2["status"], "ok", "substituted space resolved: {v2}");
        assert_eq!(v2["data"]["space_id"], "xgen://hash/sha256:S");
    }

    #[tokio::test]
    async fn unknown_binding_is_binding_not_found() {
        let dir = tempfile::tempdir().unwrap();
        write_state_fixture(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one(
            r#"{"cmd":"rooms","args":{"space":"$ghost"}}"#,
            dir.path(),
            &mut b,
            &lock(),
        )
        .await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["error"]["code"], "BINDING_NOT_FOUND");
    }

    #[test]
    fn verb_tiers_classify_local_vs_network() {
        assert_eq!(verb_tier("whoami"), TimeoutTier::Read);
        assert_eq!(verb_tier("rooms"), TimeoutTier::Read);
        assert_eq!(verb_tier("send"), TimeoutTier::Write);
        assert_eq!(verb_tier("history"), TimeoutTier::Write);
        assert!(mutates_state_file("create-space"));
        assert!(!mutates_state_file("send"));
        assert!(!mutates_state_file("whoami"));
    }
}
