// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M7 `--aicontrol` — node command-pipe surface (Commit C4).
//!
//! Sister to `pipe.rs`'s `--batch` server (D-066: `pipe.rs`/`--batch` is
//! untouched). Symmetric to the client C2 surface: a persistent JSONL control
//! protocol over a `…\.aicontrol` named pipe that wraps the **same**
//! `xgen-node-lib::admin_ops::*` functions `dispatch_admin` calls — the only
//! divergence is **envelope-out** (the AC-D2 `Reply`) instead of
//! plain-text-discard. No admin business logic is forked: arguments are
//! reconstructed into the token stream and fed through the existing
//! `admin_ops::AdminCli` clap parser (`no_binary_name`), then the parsed
//! `AdminCommand` is dispatched to `admin_ops::*` with the structured result
//! captured and serialised.
//!
//! Node-side specifics vs the client:
//! - **AC-D2 node mapping.** A verb error is a structured `AdminError`
//!   (`code` band + `Stage`); the envelope carries the band code, `category:
//!   protocol`, and `stage` — richer than the client's message-only `anyhow`.
//! - **`ActorVia::AiControl` attribution.** The arm builds the `AdminContext`
//!   with `actor_via = AiControl`, so audited writes are tagged distinctly
//!   from `batch`/`cli-direct`.
//! - **No separate state-file lock (the C2-rider analog is already handled).**
//!   Every mutating verb goes through an `Arc<tokio::sync::Mutex<…>>` store /
//!   the live `NodeRuntime`, and the **same** Arcs are threaded to both the
//!   `--batch` and `--aicontrol` servers — so concurrent connections (and the
//!   two servers) serialize on the stores' own mutexes. No client-style plain
//!   file write to guard.
//! - **`state.event_subscriptions`** is the live process-wide node-observer
//!   count (M7-events C3, EV-D6 — the `.events`-pipe registry; `0` until a
//!   subscriber connects).
//!
//! §7.1 as-built (D-078): the M2 read subset (`status`/`connections`/`peers`/
//! `spaces`/`whoami`/`version`/`identity list`) are `app::cmd_*` **print-only**
//! functions with no structured Result, so they are **not** wrapped here — the
//! `--aicontrol` node surface is the `admin_ops::*` verbs (writes + their
//! structured reads) plus `state`, which together cover the same ground.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Map, Value};
use tokio::sync::Mutex;

use xgen_common::aicontrol::{
    parse_command, resolve_cmd, resolve_timeout_ms, substitute, Bindings, Category, CmdResolution,
    ControlCode, ControlError, ControlVerb, ErrorBody, Reply, TimeoutTier,
};
use xgen_core::federation::pending_queue::PendingFederationQueue;
use xgen_core::federation::registry::FederationRegistry;
use xgen_core::federation::federation_policy::FederationPolicyStore;
use xgen_core::auth::module_registry::AuthModuleRegistry;
use xgen_core::bootstrap::registration_store::BootstrapRegistrationStore;
use xgen_core::space::node_policy::NodePolicyStore;
use xgen_core::node::runtime::NodeRuntime;

use crate::admin_ops::{self, ActorVia, AdminContext, AdminError};
use crate::fanout::{ClientSenders, FederationPeerSenders};

/// Live handles the node `--aicontrol` dispatch needs — the same Arcs the
/// `--batch` server is given, bundled so the per-connection handler threads
/// one value. Cloning is cheap (Arc clones).
#[derive(Clone)]
pub(crate) struct NodeAdminDeps {
    pub data_dir: PathBuf,
    pub config_path: PathBuf,
    pub runtime: Arc<Mutex<NodeRuntime>>,
    pub federation_registry: Arc<Mutex<FederationRegistry>>,
    pub client_senders: ClientSenders,
    pub federation_peer_senders: FederationPeerSenders,
    pub federation_queue: Arc<Mutex<PendingFederationQueue>>,
    pub federation_policy: Arc<Mutex<FederationPolicyStore>>,
    pub auth_module_registry: Arc<Mutex<AuthModuleRegistry>>,
    pub bootstrap_store: Arc<Mutex<BootstrapRegistrationStore>>,
    pub node_policy_store: Arc<Mutex<NodePolicyStore>>,
    pub connections: crate::app::Connections,
    pub started_at_epoch: u64,
    /// M9.2 (M9.2-D3 / F3) — the stashed `MockClock` installed at startup under
    /// `harness-control`, threaded to `AdminContext` so the FENCED `clock` verbs
    /// drive the *same* instance `runtime.clock()` returns. Exists only under
    /// the feature.
    #[cfg(feature = "harness-control")]
    pub mock_clock: Arc<xgen_common::clock::MockClock>,
}

/// Append `.aicontrol` to the `--batch` node pipe name to form the sister pipe.
pub(crate) fn aicontrol_pipe_name(batch_pipe_name: &str) -> String {
    format!("{batch_pipe_name}.aicontrol")
}

/// The administrator principal recorded as the audit `actor` (§2.6.1) — mirror
/// of `pipe::current_admin_actor` (kept local so `pipe.rs` stays untouched,
/// D-066). v1: OS-user-equals-administrator.
fn current_admin_actor() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .map(|u| format!("os-user:{u}"))
        .unwrap_or_else(|_| "os-user:unknown".to_string())
}

// ── Dispatch error → envelope (AC-D2 node mapping) ─────────────────────────────

enum DispatchError {
    /// Control-surface failure (parse / binding / timeout / unknown verb / …).
    Control(ControlError),
    /// A node verb error — structured `AdminError` (band code + `Stage`).
    NodeVerb(AdminError),
}

impl DispatchError {
    fn into_body(self, instance_state: &str) -> ErrorBody {
        match self {
            DispatchError::Control(ce) => ce.into_body(instance_state),
            DispatchError::NodeVerb(ae) => ErrorBody {
                code: ae.code,
                category: Category::Protocol,
                message: ae.message,
                instance_state: instance_state.to_string(),
                stage: Some(ae.stage.as_str().to_string()),
                hint: None,
                // MP-F5: node admin verbs are not the single-event reject path;
                // the wire-reject fields stay absent.
                reject_code: None,
                event_id: None,
            },
        }
    }
}

// ── Verb classification ─────────────────────────────────────────────────────────

/// Local reads (5 s) — disk/registry reads, no network round-trip.
const READ_VERBS: &[&str] = &[
    "federation list",
    "federation show-policy",
    "space list-hosted",
    "space audit-events",
    "space show-node-policy",
    "identity show",
    "auth-module list",
    "bootstrap show",
    "log show-level",
    "audit query",
    "plugin list",
    "plugin status",
];

/// Cross-Node handshakes (180 s).
const FEDERATION_VERBS: &[&str] = &["federation accept", "federation initiate"];

/// AC-D3a timeout tier for a node verb (keyed on the resolved `cmd` path).
fn verb_tier(cmd: &str) -> TimeoutTier {
    if FEDERATION_VERBS.contains(&cmd) {
        TimeoutTier::Federation
    } else if READ_VERBS.contains(&cmd) {
        TimeoutTier::Read
    } else {
        TimeoutTier::Write
    }
}

/// The `bind` primary field per verb (§5); else the whole result object.
fn primary_field(cmd: &str) -> Option<&'static str> {
    match cmd {
        "space force-eject" | "space unban" => Some("event_id"),
        "federation accept" | "federation reject" | "federation defederate" => Some("peer_node_id"),
        "auth-module register" => Some("module_id"),
        "bootstrap register" => Some("bootstrap_id"),
        _ => None,
    }
}

// ── JSON args → typed admin Args (serde marshaling, Option A) ───────────────────

/// Deserialize the JSONL `args` object into a typed admin `*Args` struct
/// (Option A, locked C4 marshaling): the structs derive `serde::Deserialize`,
/// so positional and flag args marshal **uniformly** — the client's
/// reconstruct-argv does NOT port because node verbs mix positional (required
/// IDs) + flag (options). Missing required field / wrong type → `BAD_ARGUMENT`.
/// Missing `Option` → `None` (serde-native); missing `Vec` → `[]`
/// (`#[serde(default)]` on the Args). Wire key = the Rust field name (AC-D1).
fn de<T: serde::de::DeserializeOwned>(args: Map<String, Value>) -> Result<T, DispatchError> {
    serde_json::from_value(Value::Object(args)).map_err(|e| {
        DispatchError::Control(ControlError::new(
            ControlCode::BadArgument,
            format!("invalid arguments: {e}"),
        ))
    })
}

// ── `state` control verb (AC-D3c node core) ────────────────────────────────────

/// The node resident lifecycle as observable to the pipe handler. The pipe
/// server only runs inside a live `run_node`, so the instance is `running`.
pub(crate) const NODE_LIFECYCLE: &str = "running";

/// Build the node `state` reply `data` (AC-D3c node core), composed from the
/// threaded live handles — no new instrumentation. `operator_display_name` is
/// **dropped** in v1 (it is not in local config — see `cmd_whoami`; sourcing it
/// would need the NodeAnnouncement record, out of the cheap local read).
/// `event_subscriptions` is the live process-wide node-observer count
/// (EV-D6 — the `.events`-pipe registry; `0` until a subscriber connects).
pub(crate) async fn build_node_state_data(deps: &NodeAdminDeps, bindings: &Bindings) -> Value {
    let mut data = Map::new();
    data.insert("lifecycle".into(), json!(NODE_LIFECYCLE));

    {
        let rt = deps.runtime.lock().await;
        data.insert("node_id".into(), json!(rt.node_id.as_str()));
        data.insert("hosted_spaces".into(), json!(rt.spaces.len()));
        data.insert(
            "registered_identities".into(),
            json!(rt.identity_registry.len()),
        );
    }
    {
        let fr = deps.federation_registry.lock().await;
        data.insert("federated_peers".into(), json!(fr.len()));
    }
    {
        let conns = deps.connections.lock().await;
        data.insert("active_connections".into(), json!(conns.len()));
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(deps.started_at_epoch);
    data.insert(
        "uptime_seconds".into(),
        json!(now.saturating_sub(deps.started_at_epoch)),
    );

    // Config-derived endpoint + auth tiers (best-effort local read).
    if let Ok(content) = std::fs::read_to_string(&deps.config_path) {
        if let Ok(cfg) = toml::from_str::<crate::app::NodeConfig>(&content) {
            data.insert("endpoint".into(), json!(cfg.node.listen));
            data.insert(
                "auth_tiers_served".into(),
                json!(cfg.bootstrap.auth_tiers_served),
            );
        }
    }

    data.insert("bindings".into(), Value::Object(bindings.snapshot()));
    // EV-D6 — live process-wide node-observer count (the `.events`-pipe
    // registry, single source of truth). Empty registry ⇒ 0 ⇒ today.
    data.insert(
        "event_subscriptions".into(),
        json!(crate::fanout::node_observers().lock().await.len()),
    );

    Value::Object(data)
}

// ── Per-command dispatch ────────────────────────────────────────────────────────

/// Handle one JSONL command line → envelope [`Reply`] (transport-free core,
/// unit-testable without a pipe).
pub(crate) async fn dispatch_one(line: &str, deps: &NodeAdminDeps, bindings: &mut Bindings) -> Reply {
    let command = match parse_command(line) {
        Ok(c) => c,
        Err(ce) => return Reply::error(None, None, ce.into_body(NODE_LIFECYCLE)),
    };
    let cmd = command.cmd.trim().to_string();
    let id = command.id.clone();

    match dispatch_resolved(&command, &cmd, deps, bindings).await {
        Ok(data) => {
            if let Some(name) = &command.bind {
                record_bind(bindings, name, &cmd, &data);
            }
            Reply::ok(cmd, id, data)
        }
        Err(de) => Reply::error(Some(cmd), id, de.into_body(NODE_LIFECYCLE)),
    }
}

async fn dispatch_resolved(
    command: &xgen_common::aicontrol::Command,
    cmd: &str,
    deps: &NodeAdminDeps,
    bindings: &Bindings,
) -> Result<Value, DispatchError> {
    if let CmdResolution::Control(ControlVerb::State) = resolve_cmd(cmd) {
        return Ok(build_node_state_data(deps, bindings).await);
    }

    let mut args = command.args.clone();
    substitute(&mut args, bindings).map_err(DispatchError::Control)?;
    let timeout_override = args.remove("timeout_ms");
    let tier = verb_tier(cmd);
    let timeout_ms =
        resolve_timeout_ms(tier, timeout_override.as_ref()).map_err(DispatchError::Control)?;

    run_verb(cmd, args, deps, timeout_ms).await
}

/// Build the `AdminContext` from the live deps with `actor_via = AiControl`,
/// mirroring `pipe::dispatch_admin`'s threading.
fn build_ctx<'a>(deps: &'a NodeAdminDeps, actor: String) -> AdminContext<'a> {
    let mut ctx = AdminContext::batch(&deps.data_dir, &deps.config_path, actor);
    ctx.actor_via = ActorVia::AiControl;
    let ctx = ctx
        .with_runtime(Arc::clone(&deps.runtime))
        .with_federation_registry(Arc::clone(&deps.federation_registry))
        .with_client_senders(Arc::clone(&deps.client_senders))
        .with_federation_senders(Arc::clone(&deps.federation_peer_senders))
        .with_federation_queue(Arc::clone(&deps.federation_queue))
        .with_federation_policy(Arc::clone(&deps.federation_policy))
        .with_auth_module_registry(Arc::clone(&deps.auth_module_registry))
        .with_bootstrap_store(Arc::clone(&deps.bootstrap_store))
        .with_node_policy_store(Arc::clone(&deps.node_policy_store));
    // M9.2 (M9.2-D3 / F3) — stash the MockClock handle so the FENCED clock verbs
    // drive the instance installed at startup (not a downcast of the trait object).
    #[cfg(feature = "harness-control")]
    let ctx = ctx.with_mock_clock(Arc::clone(&deps.mock_clock));
    ctx
}

fn to_val<T: serde::Serialize>(r: T) -> Value {
    serde_json::to_value(r).unwrap_or(Value::Null)
}

/// Dispatch one node admin verb (keyed on the resolved `cmd` path) to the
/// matching `admin_ops::*`, deserializing its typed args (`de`) + capturing the
/// structured result as JSON, under the per-command timeout (AC-D3a). The arm
/// set is the shipped `admin_ops::*` surface — the SAME fns `dispatch_admin`
/// calls (no forked admin logic); only result-capture + envelope-out differ.
/// An unmatched `cmd` → `UNKNOWN_COMMAND`.
async fn run_verb(
    cmd: &str,
    args: Map<String, Value>,
    deps: &NodeAdminDeps,
    timeout_ms: u64,
) -> Result<Value, DispatchError> {
    let actor = current_admin_actor();
    let cmd = cmd.to_string();
    let fut: Pin<Box<dyn Future<Output = Result<Value, DispatchError>> + Send + '_>> =
        Box::pin(async move {
            let mut ctx = build_ctx(deps, actor);
            macro_rules! cap {
                ($e:expr) => {
                    $e.await.map(to_val).map_err(DispatchError::NodeVerb)
                };
            }
            match cmd.as_str() {
                "audit query" => cap!(admin_ops::audit_query(&mut ctx, de(args)?)),
                "audit export" => cap!(admin_ops::audit_export(&mut ctx, de(args)?)),
                "audit archive" => cap!(admin_ops::audit_archive(&mut ctx, de(args)?)),
                "log show-level" => cap!(admin_ops::log_show_level(&mut ctx, de(args)?)),
                "log set-level" => cap!(admin_ops::log_set_level(&mut ctx, de(args)?)),
                "identity show" => cap!(admin_ops::identity_show(&mut ctx, de(args)?)),
                "identity revoke" => cap!(admin_ops::identity_revoke(&mut ctx, de(args)?)),
                "identity set-trust-expiry" => cap!(admin_ops::identity_set_trust_expiry(&mut ctx, de(args)?)),
                "identity manage-replica" => cap!(admin_ops::identity_manage_replica(&mut ctx, de(args)?)),
                "federation list" => cap!(admin_ops::federation_list(&mut ctx, de(args)?)),
                "federation defederate" => cap!(admin_ops::federation_defederate(&mut ctx, de(args)?)),
                "federation accept" => cap!(admin_ops::federation_accept(&mut ctx, de(args)?)),
                "federation reject" => cap!(admin_ops::federation_reject(&mut ctx, de(args)?)),
                "federation initiate" => cap!(admin_ops::federation_initiate(&mut ctx, de(args)?)),
                "federation set-policy" => cap!(admin_ops::federation_set_policy(&mut ctx, de(args)?)),
                "federation show-policy" => cap!(admin_ops::federation_show_policy(&mut ctx, de(args)?)),
                "space list-hosted" => cap!(admin_ops::space_list_hosted(&mut ctx, de(args)?)),
                "space audit-events" => cap!(admin_ops::space_audit_events(&mut ctx, de(args)?)),
                "space audit-rebuild" => cap!(admin_ops::space_audit_rebuild(&mut ctx, de(args)?)),
                "space force-eject" => cap!(admin_ops::space_force_eject(&mut ctx, de(args)?)),
                "space unban" => cap!(admin_ops::space_unban(&mut ctx, de(args)?)),
                "space set-node-policy" => cap!(admin_ops::node_set_policy(&mut ctx, de(args)?)),
                "space show-node-policy" => cap!(admin_ops::node_show_policy(&mut ctx, de(args)?)),
                "plugin list" => cap!(admin_ops::plugin_list(&mut ctx, de(args)?)),
                "plugin status" => cap!(admin_ops::plugin_status(&mut ctx, de(args)?)),
                "auth-module list" => cap!(admin_ops::auth_module_list(&mut ctx)),
                "auth-module register" => cap!(admin_ops::auth_module_register(&mut ctx, de(args)?)),
                "auth-module revoke" => cap!(admin_ops::auth_module_revoke(&mut ctx, de(args)?)),
                "auth-module set-tiers" => cap!(admin_ops::auth_module_set_tiers(&mut ctx, de(args)?)),
                "auth-module test" => cap!(admin_ops::auth_module_test(&mut ctx, de(args)?)),
                "bootstrap show" => cap!(admin_ops::bootstrap_show(&mut ctx)),
                "bootstrap register" => cap!(admin_ops::bootstrap_register(&mut ctx, de(args)?)),
                "bootstrap deregister" => cap!(admin_ops::bootstrap_deregister(&mut ctx, de(args)?)),
                "bootstrap set-info" => cap!(admin_ops::bootstrap_set_info(&mut ctx, de(args)?)),
                "bootstrap set-tiers" => cap!(admin_ops::bootstrap_set_tiers(&mut ctx, de(args)?)),
                // M9.2 FENCED harness seams (M9.2-D1) — exist only under
                // `--features harness-control`; absent ⇒ they fall to the
                // `_ => UNKNOWN_COMMAND` arm in a default build (the aicontrol
                // half of the fence, complementary to the clap-surface fence).
                #[cfg(feature = "harness-control")]
                "federation add-peer" => cap!(admin_ops::federation_add_peer(&mut ctx, de(args)?)),
                #[cfg(feature = "harness-control")]
                "clock advance" => cap!(admin_ops::clock_advance(&mut ctx, de(args)?)),
                #[cfg(feature = "harness-control")]
                "clock set" => cap!(admin_ops::clock_set(&mut ctx, de(args)?)),
                _ => Err(DispatchError::Control(ControlError::new(
                    ControlCode::UnknownCommand,
                    "command is not available over --aicontrol",
                ))),
            }
        });

    match tokio::time::timeout(tokio::time::Duration::from_millis(timeout_ms), fut).await {
        Ok(inner) => inner,
        Err(_) => Err(DispatchError::Control(ControlError::new(
            ControlCode::Timeout,
            format!("command exceeded its {timeout_ms} ms timeout"),
        ))),
    }
}

fn record_bind(bindings: &mut Bindings, name: &str, cmd: &str, data: &Value) {
    let fields = data.as_object().cloned().unwrap_or_default();
    let primary = primary_field(cmd)
        .and_then(|k| fields.get(k).cloned())
        .unwrap_or_else(|| data.clone());
    bindings.set(name, primary, fields);
}

// ── Named-pipe server (Windows only, D-043) ─────────────────────────────────────

/// Start the node `.aicontrol` command-pipe server. Independent of the
/// `--batch` server (D-066): its own accept loop on the sister pipe name,
/// spawning one handler task per connection (multiple connections concurrent).
/// Threads the **same** live Arcs (via `deps`), so mutations serialize on the
/// stores' own mutexes across both servers. Stops when `shutdown_rx` delivers
/// `true`.
#[cfg(target_os = "windows")]
pub(crate) async fn start_aicontrol_server(
    pipe_name_str: String,
    deps: Arc<NodeAdminDeps>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    use tokio::net::windows::named_pipe::ServerOptions;

    tracing::info!(pipe = %pipe_name_str, "node aicontrol pipe server starting");
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
                    tracing::error!(error = %e, "node aicontrol pipe create failed — server stopping");
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

        let handler_deps = Arc::clone(&deps);
        tokio::spawn(async move {
            handle_aicontrol_connection(server, handler_deps).await;
        });
    }

    tracing::info!(pipe = %pipe_name_str, "node aicontrol pipe server stopped");
}

/// One persistent JSONL session: read a line, dispatch, write the envelope
/// reply, repeat until the connection closes. Own binding namespace + serial
/// in-flight guard (the sequential loop keeps the connection strictly serial).
#[cfg(target_os = "windows")]
async fn handle_aicontrol_connection(
    server: tokio::net::windows::named_pipe::NamedPipeServer,
    deps: Arc<NodeAdminDeps>,
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
            Ok(0) => break,
            Ok(_) => {
                let line = buf.trim_end_matches('\n').trim_end_matches('\r');
                if line.trim().is_empty() {
                    continue;
                }
                let reply = if in_flight.swap(true, Ordering::SeqCst) {
                    Reply::error(
                        None,
                        None,
                        ControlError::new(
                            ControlCode::ConcurrentCommandNotAllowed,
                            "a command is already in flight on this connection",
                        )
                        .into_body(NODE_LIFECYCLE),
                    )
                } else {
                    let r = dispatch_one(line, &deps, &mut bindings).await;
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
                tracing::warn!(error = %e, "node aicontrol pipe read error");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn test_deps(dir: &std::path::Path) -> NodeAdminDeps {
        use std::collections::HashMap;
        let runtime = Arc::new(Mutex::new(NodeRuntime::new(SigningKey::from_bytes(&[3u8; 32]))));
        NodeAdminDeps {
            data_dir: dir.to_path_buf(),
            config_path: dir.join("xgen-node_config.toml"),
            runtime,
            federation_registry: Arc::new(Mutex::new(FederationRegistry::new())),
            client_senders: Arc::new(Mutex::new(HashMap::new())),
            federation_peer_senders: Arc::new(Mutex::new(HashMap::new())),
            federation_queue: Arc::new(Mutex::new(PendingFederationQueue::new())),
            federation_policy: Arc::new(Mutex::new(FederationPolicyStore::new())),
            auth_module_registry: Arc::new(Mutex::new(AuthModuleRegistry::new())),
            bootstrap_store: Arc::new(Mutex::new(BootstrapRegistrationStore::new())),
            node_policy_store: Arc::new(Mutex::new(NodePolicyStore::new())),
            connections: Arc::new(Mutex::new(Vec::new())),
            started_at_epoch: 0,
            #[cfg(feature = "harness-control")]
            mock_clock: Arc::new(xgen_common::clock::MockClock::new()),
        }
    }

    #[test]
    fn verb_tiers_classify_read_write_federation() {
        assert_eq!(verb_tier("federation list"), TimeoutTier::Read);
        assert_eq!(verb_tier("space show-node-policy"), TimeoutTier::Read);
        assert_eq!(verb_tier("federation accept"), TimeoutTier::Federation);
        assert_eq!(verb_tier("federation initiate"), TimeoutTier::Federation);
        assert_eq!(verb_tier("space force-eject"), TimeoutTier::Write);
        assert_eq!(verb_tier("auth-module register"), TimeoutTier::Write);
    }

    #[test]
    fn node_verb_error_maps_to_band_code_and_stage() {
        use crate::admin_ops::Stage;
        let body = DispatchError::NodeVerb(AdminError::new(
            "SPACE_8005",
            Stage::Validate,
            "action_threshold out of range",
        ))
        .into_body("running");
        assert_eq!(body.code, "SPACE_8005");
        assert_eq!(body.category, Category::Protocol);
        assert_eq!(body.stage.as_deref(), Some("validate"));
        assert_eq!(body.instance_state, "running");
    }

    // Serial-grouped on `node_observers`: this asserts the global observer
    // count is 0, so it must not overlap the test below that transiently
    // pushes an observer into the same process-global registry.
    #[tokio::test]
    #[serial_test::serial(node_observers)]
    async fn state_verb_returns_node_core() {
        let dir = tempfile::tempdir().unwrap();
        let deps = test_deps(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one(r#"{"cmd":"state"}"#, &deps, &mut b).await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["status"], "ok");
        assert_eq!(v["cmd"], "state");
        assert_eq!(v["data"]["lifecycle"], "running");
        assert_eq!(v["data"]["federated_peers"], 0);
        assert_eq!(v["data"]["hosted_spaces"], 0);
        assert_eq!(v["data"]["active_connections"], 0);
        assert_eq!(v["data"]["registered_identities"], 0);
        assert_eq!(v["data"]["event_subscriptions"], 0);
        assert!(v["data"].get("node_id").is_some());
        assert!(v["data"].get("uptime_seconds").is_some());
        // operator_display_name dropped in v1 (not in local config).
        assert!(v["data"].get("operator_display_name").is_none());
    }

    // EV-D6 — `state.event_subscriptions` reflects the live process-global
    // node-observer count. Pushes one observer, asserts the count is 1, then
    // prunes it (serial-grouped so it never races the `== 0` test above).
    #[tokio::test]
    #[serial_test::serial(node_observers)]
    async fn state_verb_event_subscriptions_reflects_observer_count() {
        use xgen_common::conn::ConnId;
        use xgen_common::aicontrol::Filter;

        let conn = ConnId::mint();
        let (tx, _rx) = tokio::sync::mpsc::channel::<crate::fanout::OutboundMsg>(4);
        crate::fanout::node_observers()
            .lock()
            .await
            .push((conn, Filter::default(), tx));

        let dir = tempfile::tempdir().unwrap();
        let deps = test_deps(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one(r#"{"cmd":"state"}"#, &deps, &mut b).await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["data"]["event_subscriptions"], 1);

        // Prune so the registry returns to empty for sibling serial tests.
        crate::fanout::node_observers()
            .lock()
            .await
            .retain(|(c, _, _)| *c != conn);
    }

    #[tokio::test]
    async fn unknown_verb_is_unknown_command() {
        let dir = tempfile::tempdir().unwrap();
        let deps = test_deps(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one(r#"{"cmd":"frobnicate"}"#, &deps, &mut b).await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["error"]["code"], "UNKNOWN_COMMAND");
        assert_eq!(v["cmd"], "frobnicate");
    }

    #[tokio::test]
    async fn malformed_command_omits_cmd() {
        let dir = tempfile::tempdir().unwrap();
        let deps = test_deps(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one("{not json", &deps, &mut b).await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["error"]["code"], "MALFORMED_COMMAND");
        assert!(v.get("cmd").is_none());
    }

    #[tokio::test]
    async fn node_verb_round_trips_band_code_and_stage_on_error() {
        // show-node-policy on a Space this empty Node does not host → SPACE_8001.
        let dir = tempfile::tempdir().unwrap();
        let deps = test_deps(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one(
            r#"{"cmd":"space show-node-policy","args":{"space_id":"xgen://hash/sha256:nope"}}"#,
            &deps,
            &mut b,
        )
        .await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["status"], "error");
        assert_eq!(v["cmd"], "space show-node-policy");
        assert_eq!(v["error"]["code"], "SPACE_8001");
        assert_eq!(v["error"]["category"], "protocol");
        assert!(v["error"].get("stage").is_some(), "node verb error carries a stage");
    }

    #[tokio::test]
    async fn missing_required_arg_is_bad_argument() {
        // show-node-policy requires space_id (a positional → required serde field).
        let dir = tempfile::tempdir().unwrap();
        let deps = test_deps(dir.path());
        let mut b = Bindings::new();
        let reply = dispatch_one(
            r#"{"cmd":"space show-node-policy","args":{}}"#,
            &deps,
            &mut b,
        )
        .await;
        let v: Value = serde_json::from_str(&reply.to_line()).unwrap();
        assert_eq!(v["error"]["code"], "BAD_ARGUMENT");
        assert_eq!(v["error"]["category"], "argument");
    }
}
