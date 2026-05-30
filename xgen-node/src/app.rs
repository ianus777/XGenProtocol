// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Node resident-mode logic (D-063). All long-running protocol behaviour —
//! WebSocket accept loop, connection handlers, identity registration,
//! federation handshake, fan-out, persistence — lives here so that every
//! entry point (CLI dispatcher, Tauri shell, future control-mode commands)
//! routes through one shared command layer (D-056).

use std::{
    collections::{BTreeMap, HashMap},
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use tokio::net::TcpStream;

use xgen_common::{
    build_info,
    event_trace::{
        EventDirection, ExitReason, LocalAction, SessionContext, SpaceRole,
        trace_event, trace_local, write_session_footer, write_session_header,
    },
    state::{ConnectedClient, FederatedPeer, HostedRoom, HostedSpace, NodeState},
    xgid::{IdentityXgid, NodeXgid, SpaceXgid, Xgid},
};
use crate::{
    crypto::encoding,
    federation::{
        handshake::{negotiate_serialisation, negotiate_version, sign_msg, verify_msg},
        pending_queue::{
            should_queue_for_approval, PendingFederationQueue, PendingFederationRequest,
            FEDERATION_APPROVAL_PENDING_CODE, FEDERATION_APPROVAL_PENDING_STRING,
        },
        registry::{FederationRegistry, FederationRelationship, FederationState},
    },
    identity::{
        keypair,
        registration::accept_registration,
        registry::{IdentityRecord, IdentityRegistry},
        replication::handle_incoming_replicate,
    },
    node::runtime::{topological_sort, DispatchOutcome, EventOrigin, NodeRuntime},
    transport::{
        client::connect_url,
        connection::{Connection, Inbound},
        server::Server,
    },
    wire::types::{
        Event, FederationCapabilities, FederationMessage, IdentityDeviceEntry,
        IdentityMessage, IdentityReplicateMessage, NegotiatedCapabilities,
        TransportMessage,
    },
};

// Fan-out types live in `crate::fanout` so they are unit-testable
// without spawning real sockets.
use crate::fanout::{
    apply_fanout, collect_sync_history, ClientSenders, FanoutRequest, OutboundMsg,
};
use crate::federation_session::{apply_federation_push, stream_federation_delta};

// ── Node config ────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NodeConfig {
    pub node: NodeSection,
    pub paths: PathsSection,
    pub logging: LoggingSection,
    /// F-6b / F-7a sync-pipeline tuning. Absent in pre-F-6/F-7 configs;
    /// `#[serde(default)]` keeps existing on-disk configs parsing.
    #[serde(default)]
    pub sync: SyncSection,
    /// `[federation]` admin-control tuning (FAC-D1, sub-arc 2a). Absent in
    /// pre-2a configs; `#[serde(default)]` keeps existing on-disk configs
    /// parsing with `require_approval` defaulting to false — today's
    /// auto-establish behaviour, byte-for-byte.
    #[serde(default)]
    pub federation: FederationSection,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NodeSection {
    pub listen: String,
    pub local_mode: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PathsSection {
    pub keypair_path: String,
    pub spaces_dir: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoggingSection {
    pub level: String,
}

/// `[sync]` config section (F-6b + F-7a). Both fields are reference-implementation
/// defaults the operator may override; neither is protocol-fixed. Pattern
/// matches `[logging]` — protocol prescribes the mechanism, not the values.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncSection {
    /// F-6b safety-net timeout in seconds. When a sync_request issues, the
    /// requester waits up to this long for the peer's `SyncComplete` before
    /// surfacing a "peer never said done" error. Default 5; configurable per
    /// deployment (LAN can lower it; satellite-link can raise it).
    #[serde(default = "default_completion_timeout_seconds")]
    pub completion_timeout_seconds: u64,
    /// F-7a default page size. The Node returns at most `batch_size` events
    /// per `sync_request` response, with `continue_from` cursor on the trailing
    /// `SyncComplete` when more events remain. Default 1000.
    #[serde(default = "default_sync_batch_size")]
    pub batch_size: u32,
    /// Phase 7.5 §7 — timeout (seconds) for HeldPending entries waiting on
    /// federation-relationship arrival (third trigger, P7.5-B). Predecessor
    /// and Identity triggers remain at 30 s; federation-relationship defaults
    /// to 180 s because bootstrap streams routinely take tens of seconds to
    /// deliver the topologically-last `state.federation_add` across realistic
    /// WAN latency, especially with F-7 pagination. Operators on slow links
    /// or with very large Space histories can raise this; LAN deployments
    /// can lower it.
    #[serde(default = "default_federation_relationship_timeout_seconds")]
    pub federation_relationship_timeout_seconds: u64,
}

fn default_completion_timeout_seconds() -> u64 {
    5
}

fn default_sync_batch_size() -> u32 {
    1000
}

fn default_federation_relationship_timeout_seconds() -> u64 {
    xgen_core::dag::pending::FEDERATION_RELATIONSHIP_TIMEOUT_SECS
}

impl Default for SyncSection {
    fn default() -> Self {
        Self {
            completion_timeout_seconds: default_completion_timeout_seconds(),
            batch_size: default_sync_batch_size(),
            federation_relationship_timeout_seconds:
                default_federation_relationship_timeout_seconds(),
        }
    }
}

/// `[federation]` config section (FAC-D1, sub-arc 2a). Today carries only the
/// approval opt-in flag; the policy verbs (2b) will extend it. Pattern matches
/// `[sync]` / `[logging]` — the protocol prescribes the mechanism, the operator
/// sets the value. `#[derive(Default)]` → `require_approval = false`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FederationSection {
    /// FAC-D1 — inbound federation approval gate. Default **false** = today's
    /// auto-establish on a valid handshake, byte-for-byte (the prime
    /// default-off invariant). When `true`, an inbound handshake from a
    /// not-already-`Active` peer is queued for operator `accept` / `reject`
    /// instead of auto-establishing (the pause-point lands in Commit 3).
    #[serde(default)]
    pub require_approval: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        let dir = exe_dir();
        Self {
            node: NodeSection {
                listen: "ws://127.0.0.1:8080/xgen".to_string(),
                local_mode: true,
            },
            paths: PathsSection {
                keypair_path: dir
                    .join("xgen-node_keypair.enc")
                    .to_string_lossy()
                    .to_string(),
                spaces_dir: Some(dir.join("spaces").to_string_lossy().to_string()),
            },
            logging: LoggingSection {
                level: "debug".to_string(),
            },
            sync: SyncSection::default(),
            federation: FederationSection::default(),
        }
    }
}

// ── Connection tracking ────────────────────────────────────────────────────────

/// Pass 3 (Surface #5 Q5.15) — `identity_id` retyped to `IdentityXgid`.
/// xgen-node-internal admin-state struct; feeds `build_node_state` + admin
/// display. If M6 (new) Block 4 admin verbs surface this struct via a
/// pipe-server export, the format-boundary applies at that export site.
pub(crate) struct ConnectedClientInfo {
    pub(crate) identity_id: IdentityXgid,
    pub(crate) display_name: String,
    pub(crate) connected_at: String,
    pub(crate) events_received: u64,
}

pub(crate) type Connections = Arc<tokio::sync::Mutex<Vec<ConnectedClientInfo>>>;

// ── run (no subcommand — starts the Node server) ───────────────────────────────

/// Options controlling how `run_node` initialises itself.
#[derive(Debug, Clone)]
pub struct RunNodeOpts {
    /// Force Local Node mode regardless of the config setting. `--local` flag.
    pub local_override: bool,
    /// Override the WS listener port per D-068 (precedence: flag > config >
    /// default). When `Some(port)`, replaces the port component of
    /// `config.node.listen` before binding. Host and path components remain
    /// from config. `None` → use the port from config (or `8080` if config
    /// is missing). `--port` flag.
    pub port_override: Option<u16>,
    /// Install the global tracing subscriber + write session header. Set false
    /// when the Tauri desktop shell has already done both.
    pub init_logging: bool,
    /// Suppress startup chatter on stdout (banner, "Listening on…"). Errors
    /// still surface; structured logs are unaffected. `--quiet` flag.
    pub quiet: bool,
    /// Override the effective logging level. Wins over config and XGEN_LOG.
    /// Only consulted when `init_logging` is true. `--log-level` flag.
    pub log_level_override: Option<String>,
    /// Instance label (from `--instance`). Used to derive the named-pipe name
    /// (D-043). `None` → `\\.\pipe\xgen-node`; `Some("n1")` → `xgen-node-n1`.
    pub instance_label: Option<String>,
}

impl Default for RunNodeOpts {
    fn default() -> Self {
        Self {
            local_override: false,
            port_override: None,
            init_logging: true,
            quiet: false,
            log_level_override: None,
            instance_label: None,
        }
    }
}

// ── Runtime log-level control (M6 A6 — `log set-level` / `log show-level`) ───────
//
// The Node subscriber (built in `run_node` below) wraps its `EnvFilter` in a
// reloadable layer; the handle is stashed here so `admin_ops::log_set_level` can
// swap the filter at runtime (A6-D1 — runtime-only, NOT persisted to config; the
// level survives until restart). `EnvFilter` does not report its directives back
// as strings, so the effective state is mirrored in `LOG_STATE` for
// `log show-level`. Both are unset when logging was not initialised here (e.g.
// the desktop shell installed its own subscriber, or `--service` without
// init_logging) — the verbs surface that honestly.

static LOG_RELOAD: std::sync::OnceLock<
    tracing_subscriber::reload::Handle<tracing_subscriber::EnvFilter, tracing_subscriber::Registry>,
> = std::sync::OnceLock::new();
static LOG_STATE: std::sync::OnceLock<std::sync::Mutex<LogFilterState>> = std::sync::OnceLock::new();

/// Effective tracing filter: a default level plus per-module overrides.
#[derive(Default, Clone)]
pub struct LogFilterState {
    pub default: String,
    pub modules: std::collections::BTreeMap<String, String>,
}

impl LogFilterState {
    /// Serialise to an `EnvFilter` directive string (`default,mod=lvl,…`).
    pub fn to_directive(&self) -> String {
        let mut parts = vec![self.default.clone()];
        for (m, l) in &self.modules {
            parts.push(format!("{m}={l}"));
        }
        parts.join(",")
    }
}

/// Why a runtime `log set-level` failed (mapped to `LOG_*` codes by the verb).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSetError {
    /// Level not one of error|warn|info|debug|trace|off.
    InvalidLevel,
    /// The resulting directive was rejected by `EnvFilter` (e.g. bad module path).
    UnsettableModule,
    /// The subscriber was not initialised with a reload handle here.
    NoHandle,
}

const VALID_LOG_LEVELS: [&str; 6] = ["error", "warn", "info", "debug", "trace", "off"];

/// Apply a runtime tracing level for `module` (None / `"*"` / `""` = global
/// default). Returns `(previous_level, applied)`. A6-D1: runtime-only.
pub fn apply_log_set_level(
    module: Option<&str>,
    level: &str,
) -> Result<(String, bool), LogSetError> {
    if !VALID_LOG_LEVELS.contains(&level) {
        return Err(LogSetError::InvalidLevel);
    }
    let handle = LOG_RELOAD.get().ok_or(LogSetError::NoHandle)?;
    let state = LOG_STATE.get().ok_or(LogSetError::NoHandle)?;
    let mut st = state.lock().unwrap();
    let previous = match module {
        None | Some("*") | Some("") => {
            let p = st.default.clone();
            st.default = level.to_string();
            p
        }
        Some(m) => {
            let p = st.modules.get(m).cloned().unwrap_or_else(|| st.default.clone());
            st.modules.insert(m.to_string(), level.to_string());
            p
        }
    };
    let directive = st.to_directive();
    let new_filter = tracing_subscriber::EnvFilter::try_new(&directive)
        .map_err(|_| LogSetError::UnsettableModule)?;
    handle
        .reload(new_filter)
        .map_err(|_| LogSetError::UnsettableModule)?;
    Ok((previous, true))
}

/// Effective levels for `log show-level`: `*` (global default) first, then the
/// per-module overrides, optionally filtered to one module path. Empty default
/// string indicates logging was not initialised under runtime control here.
pub fn log_levels(module_filter: Option<&str>) -> Vec<(String, String)> {
    let st = LOG_STATE
        .get()
        .map(|m| m.lock().unwrap().clone())
        .unwrap_or_default();
    let mut out = vec![("*".to_string(), st.default.clone())];
    for (m, l) in &st.modules {
        out.push((m.clone(), l.clone()));
    }
    if let Some(f) = module_filter {
        out.retain(|(m, _)| m == f);
    }
    out
}

/// Resident-mode entry point. Long-running. Owns the lifecycle, binds the
/// WebSocket server, accepts connections, runs until Ctrl+C.
pub async fn run_node(
    config_path: &Path,
    data_dir: &Path,
    opts: RunNodeOpts,
) -> Result<()> {
    // Load config (fall back to default if missing)
    let config = try_load_config(config_path).unwrap_or_default();
    let local_mode = config.node.local_mode || opts.local_override;
    // F-7a: per-Node sync page size, resolved at startup. Used by every
    // sync_request handler in this Node process. No flag tier today — config
    // is the only override surface — so we don't go through `resolve_setting`
    // here. If a future `--sync-batch-size` flag lands it slots in via the
    // canonical helper, matching `--port` / `--log-level`.
    let sync_batch_size: usize = config.sync.batch_size as usize;

    // Load keypair before subscriber init so node_id is available for the session header.
    let keypair_path = PathBuf::from(&config.paths.keypair_path);
    if !keypair_path.exists() {
        bail!(
            "no keypair found at {}\n  Run 'xgen-node init' to initialise this Node folder.",
            keypair_path.display()
        );
    }
    let signing_key = keypair::load(&keypair_path, "").with_context(|| {
        format!(
            "failed to load keypair from {}\n  If passphrase-protected, use empty passphrase for Phase 1.",
            keypair_path.display()
        )
    })?;
    let node_id_uri = pubkey_uri(&signing_key);

    // Initialise debug log — one file per run, datetime-stamped.
    // Skipped when the desktop shell has already installed the subscriber.
    if opts.init_logging {
        use std::fs;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        use tracing_subscriber::{fmt, reload, EnvFilter};

        let log_dir = data_dir.join("logs");
        fs::create_dir_all(&log_dir).expect("Failed to create logs/ directory");
        let now = chrono::Local::now();
        let log_filename = format!("xgen-node_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));
        let log_path = log_dir.join(&log_filename);
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to open log file");
        // D-068 — flag > env (XGEN_LOG) > config (`[logging].level`) >
        // "debug". Pre-J-079 this site already implemented the chain manually
        // but ad-hoc; converged on the canonical helper for consistency with
        // the four other entry-points and as a regression lock.
        let level_str = xgen_common::precedence::resolve_log_level(
            opts.log_level_override.as_deref(),
            Some(config.logging.level.as_str()),
        );
        // M6 A6-D1 — wrap the EnvFilter in a reload layer so `log set-level` can
        // swap it at runtime. The fmt layer keeps every prior option verbatim
        // (target, no-ansi, ChronoLocal timer, level, the per-run log file), so
        // logging behaviour is unchanged. The global filter gates the fmt layer
        // exactly as `with_env_filter` did.
        let (filter_layer, reload_handle) = reload::Layer::new(EnvFilter::new(&level_str));
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_ansi(false)
            .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
                "%Y-%m-%d %H:%M:%S%.3f".to_string(),
            ))
            .with_level(true)
            .with_writer(log_file);
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
        // Stash the reload handle + mirror the directive state for the A6 verbs.
        let _ = LOG_RELOAD.set(reload_handle);
        let _ = LOG_STATE.set(std::sync::Mutex::new(LogFilterState {
            default: level_str,
            modules: std::collections::BTreeMap::new(),
        }));
    }

    let started_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let session_id = format!("{:08x}", rand::random::<u32>());

    // Resolve the effective listen address per D-068 — `--port` overrides the
    // port component of `config.node.listen` (host and path remain from
    // config). Computed once here so banner output, session header, tracing,
    // state-file writes, and the actual bind all show the same value. See
    // xgen-common::precedence::resolve_setting and tasks/CLI_PRECEDENCE_AUDIT.md
    // for the rule.
    let listen_addr_from_config = parse_ws_addr(&config.node.listen)?;
    let resolved_port = xgen_common::precedence::resolve_setting(
        opts.port_override,
        None,
        Some(listen_addr_from_config.port()),
        8080u16,
    );
    let mut listen_addr = listen_addr_from_config;
    listen_addr.set_port(resolved_port);
    let effective_endpoint: String = if resolved_port == listen_addr_from_config.port() {
        config.node.listen.clone()
    } else {
        rewrite_url_port(&config.node.listen, resolved_port)
    };

    // Session header — written once, immediately after subscriber init.
    // Skipped when the desktop shell has already emitted one (with no
    // node_id, since the keypair wasn't yet loaded at that point).
    if opts.init_logging {
        write_session_header(
            "node",
            Some(&node_id_uri),
            Some(&effective_endpoint),
            None,
            "0.1",
            build_info::VERSION,
            &session_id,
            &started_at,
        );
    } else {
        // In desktop mode, the node_id wasn't known when the session header
        // was written. Log it now as a body line so it's still traceable.
        tracing::info!(node_id = %node_id_uri, endpoint = %effective_endpoint, "Node identity loaded");
    }

    // Spaces directory — Tier 2 (default: <data_dir>/spaces, overridable via config)
    let spaces_dir = config.paths.spaces_dir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("spaces"));
    let _ = std::fs::create_dir_all(&spaces_dir);

    // Load identity registry
    let identities_path = data_dir.join("xgen-node_identities.db");
    let mut runtime = NodeRuntime::new(signing_key.clone());
    if identities_path.exists() {
        match IdentityRegistry::load(&identities_path) {
            Ok(reg) => runtime.identity_registry = reg,
            Err(e) => {
                eprintln!("{}", yellow(&format!("warning: identity registry load failed: {e}")));
                tracing::warn!(reason = %e, "Identity registry load failed");
            }
        }
    }

    // Phase 7.5 §5.3 + §5.6 — load receiver-local Space provenance metadata
    // before replay so the existing introducer mapping is available to
    // operators on restart. Replay itself does not repopulate this map
    // (LocallySubmitted dispatch produces an entry with introducer = None),
    // so without a load step the introducer attribution would be lost on
    // every restart.
    // Pass 3 (Surface #5 Q5.12 site a) — persistence-format boundary projection:
    // `load_space_local_metadata` returns `HashMap<String, _>` (§4.3 persistence
    // boundary); project to typed `HashMap<SpaceXgid, _>` at the insertion site
    // into the in-memory store.
    runtime.space_local_metadata = load_space_local_metadata(data_dir)
        .into_iter()
        .map(|(k, v)| (SpaceXgid::from_xgid(Xgid::new(k)), v))
        .collect();

    // Replay Space event logs from disk — MUST complete before network listener opens (spec 4.8.5).
    let replayed = replay_spaces_from_dir(&mut runtime, &spaces_dir);
    if replayed > 0 {
        eprintln!("Replayed {} Space event store(s) from disk.", replayed);
        tracing::info!(count = replayed, "Space event stores replayed from disk");
    }

    // Startup banner (suppressed under --quiet).
    if !opts.quiet {
        build_info::print_banner("xgen-node");
        println!();
        println!("Node ID:    {}", runtime.node_id);
        println!("Endpoint:   {}", effective_endpoint);
        println!("Mode:       {}", if local_mode { "local" } else { "production" });
        println!("Identities: {} registered", runtime.identity_registry.len());
        println!();
    }
    tracing::info!(node_id = %runtime.node_id, endpoint = %effective_endpoint, "Node started");

    // `listen_addr` and `effective_endpoint` already resolved above per D-068.

    // Shared state
    let node_id = runtime.node_id.clone();

    // PAL-D1 / Shape β (J-165 + checkpoint #1) — install the process-global
    // protocol-audit sink once. `persist_event`'s hook reads it. The audit
    // directory is `<data_dir>/audit/` per §3.11.8 + D-035 (co-located with the
    // other Tier-1 Node files). A Node process has exactly one audit dir and one
    // node_id, so a global is the honest model and avoids threading a param
    // through every accept path's signature.
    {
        let audit_dir = data_dir.join("audit");
        let _ = std::fs::create_dir_all(&audit_dir);
        crate::protocol_audit::ProtocolAuditSink::init_global(
            crate::protocol_audit::ProtocolAuditSink::new(audit_dir, node_id.as_str().to_string()),
        );
    }

    let node_keypair = Arc::new(signing_key);
    let runtime = Arc::new(tokio::sync::Mutex::new(runtime));
    let connections: Connections = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let client_senders: ClientSenders = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    // Phase 4 (runbook §3.4.1 Q2 lock): active federation peer sessions
    // keyed by peer node_id. Mirror of `client_senders` for the federation
    // direction; `apply_federation_push` reads it to find live sessions to
    // push locally-accepted events into.
    let federation_peer_senders: crate::fanout::FederationPeerSenders =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    // Phase 5 (runbook §3.5.1 Lock A — A3 storage): federation registry holds
    // both `FederationRelationship` records (protocol state) and
    // `PeerOperationalRecord` records (F-1c operational state) in a single
    // JSON file. Per the audit J-081 §2.1 finding, this is the first
    // production wiring of `FederationRegistry` in xgen-node — pre-Phase-5
    // the type existed in xgen-core but was never loaded or saved by the
    // running Node. The reconnect scheduler reads this registry each tick.
    let federation_registry_path = data_dir.join("xgen-node_federation.json");
    let federation_registry = if federation_registry_path.exists() {
        match FederationRegistry::load(&federation_registry_path) {
            Ok(r) => {
                tracing::info!(
                    path = ?federation_registry_path,
                    relationships = r.len(),
                    "Loaded federation registry"
                );
                r
            }
            Err(e) => {
                tracing::warn!(
                    path = ?federation_registry_path,
                    error = %e,
                    "Federation registry file present but failed to load; starting fresh"
                );
                FederationRegistry::new()
            }
        }
    } else {
        FederationRegistry::new()
    };
    let federation_registry = Arc::new(tokio::sync::Mutex::new(federation_registry));

    // federation-admin-control 2a (FAC-D1/D1a) — the inbound-approval gate
    // and its pending-request queue. `require_approval` defaults false
    // (default-off invariant: federation auto-establishes exactly as today);
    // the queue is a sibling store to the registry (pre-relationship records),
    // persisted JSON per the D-035 convention beside the registry file.
    let require_approval = config.federation.require_approval;
    let federation_queue_path = data_dir.join("xgen-node_federation_queue.json");
    let federation_queue = if federation_queue_path.exists() {
        match PendingFederationQueue::load(&federation_queue_path) {
            Ok(q) => {
                tracing::info!(
                    path = ?federation_queue_path,
                    pending = q.len(),
                    "Loaded pending federation request queue"
                );
                q
            }
            Err(e) => {
                tracing::warn!(
                    path = ?federation_queue_path,
                    error = %e,
                    "Pending federation queue file present but failed to load; starting fresh"
                );
                PendingFederationQueue::new()
            }
        }
    } else {
        PendingFederationQueue::new()
    };
    let federation_queue = Arc::new(tokio::sync::Mutex::new(federation_queue));

    // Phase 5 (runbook §3.5.1 Lock B) — spawn the F-1c reconnect scheduler.
    // First production caller of `run_initiating` in xgen-node/src/
    // (audit J-081 §2.2 noted zero before this milestone). Ticks every 60
    // seconds (Lock B1); each tick scans federation_registry.peer_records
    // for lost peers whose next_reconnect_attempt has elapsed, advances
    // the backoff ladder per Lock B2, and spawns detached
    // run_initiating attempts per Lock B4.
    crate::reconnect::spawn_reconnect_scheduler(
        Arc::clone(&runtime),
        Arc::clone(&client_senders),
        Arc::clone(&federation_peer_senders),
        Arc::clone(&federation_registry),
        federation_registry_path.clone(),
        Arc::clone(&node_keypair),
        node_id.clone(),
        spaces_dir.clone(),
        identities_path.clone(),
        local_mode,
        effective_endpoint.clone(),
    );

    // State writer task — writes xgen-node_state.json every 5 seconds
    // (and Phase 7.5 xgen-node_space_local_metadata.json alongside it).
    {
        let rt = Arc::clone(&runtime);
        let conns = Arc::clone(&connections);
        let fed_reg = Arc::clone(&federation_registry);
        let state_path = data_dir.join("xgen-node_state.json");
        let data_dir_w = data_dir.to_path_buf();
        let node_id_w = node_id.clone();
        let endpoint = effective_endpoint.clone();
        let mode_str = if local_mode { "local" } else { "production" }.to_string();
        let started = started_at.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                let rt_guard = rt.lock().await;
                let conns_guard = conns.lock().await;
                let peers = {
                    let reg_guard = fed_reg.lock().await;
                    build_federated_peers(&reg_guard)
                };
                let state = build_node_state(
                    &rt_guard, &conns_guard, peers, &node_id_w, &endpoint, &mode_str, &started,
                );
                // Pass 3 (Surface #5 Q5.12 site b) — persistence-format boundary
                // projection: in-memory `HashMap<SpaceXgid, _>` →
                // `HashMap<String, _>` at the save-call boundary per §4.3
                // format-boundary preservation (persistence stays String).
                let metadata_snapshot: std::collections::HashMap<
                    String,
                    xgen_common::space_local::SpaceLocalMetadata,
                > = rt_guard
                    .space_local_metadata
                    .iter()
                    .map(|(k, v)| (k.as_str().to_string(), v.clone()))
                    .collect();
                drop(rt_guard);
                drop(conns_guard);
                if let Ok(json) = serde_json::to_string_pretty(&state) {
                    let _ = std::fs::write(&state_path, json);
                }
                save_space_local_metadata(&data_dir_w, &metadata_snapshot);
            }
        });
    }

    // Pending buffer timeout sweep — every 5 s, discard events that have
    // waited longer than their effective timeout for missing dependencies
    // (spec 3.9.6). Phase 7.5 §6.3 extends Phase 6 / F-10 (runbook §3.6.1
    // Lock D) precedence to three timeout cases:
    //   - 4002 predecessor_timeout            — missing_predecessors non-empty (outright)
    //   - 4007 federation_relationship_timeout — predecessors empty AND missing_federation_relationship set
    //   - 4006 identity_record_timeout        — only missing_identity set
    //
    // Precedence: predecessor (4002) > federation-relationship (4007) > Identity (4006).
    // Rationale: federation-relationship is the most upstream blocker in the
    // dependency chain (Identity replication is conditionally downstream of
    // federation establishment because Identity events themselves flow over
    // federation transport). Reporting the most upstream blocker directs the
    // operator to the right diagnostic question.
    //
    // Per-trigger effective timeout: predecessor + Identity → 30 s
    // (PENDING_TIMEOUT_SECS); federation-relationship → 180 s default,
    // configurable via [sync].federation_relationship_timeout_seconds
    // (Phase 7.5 §7). An entry waiting on federation gets the longer window
    // even if also waiting on predecessor/Identity, so bootstrap streams have
    // headroom for federation_add to land.
    {
        let rt = Arc::clone(&runtime);
        let fed_rel_timeout = std::time::Duration::from_secs(
            config.sync.federation_relationship_timeout_seconds,
        );
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                let mut rt_guard = rt.lock().await;
                let now = std::time::Instant::now();
                for (space_id, buf) in &mut rt_guard.pending {
                    for entry in buf.drain_timed_out(now, fed_rel_timeout) {
                        // Phase 7.5 §6.3 — predecessor-code-wins precedence
                        // extended: 4002 > 4007 > 4006. The verbatim block
                        // here documents the branch order so future audits
                        // can confirm the precedence ranking holds.
                        if !entry.missing_predecessors.is_empty() {
                            tracing::warn!(
                                space_id = %space_id,
                                event_id = %entry.event_id,
                                missing = ?entry.missing_predecessors,
                                missing_identity = ?entry.missing_identity,
                                missing_federation_relationship = ?entry.missing_federation_relationship,
                                error_code = 4002,
                                "4002 predecessor_timeout: pending event discarded after timeout"
                            );
                        } else if let Some((peer, space)) =
                            entry.missing_federation_relationship.as_ref()
                        {
                            tracing::warn!(
                                space_id = %space_id,
                                event_id = %entry.event_id,
                                missing_identity = ?entry.missing_identity,
                                peer_node_id = %peer,
                                fed_space_id = %space,
                                error_code = 4007,
                                "4007 federation_relationship_timeout: pending event discarded after timeout (state.federation_add never arrived)"
                            );
                        } else {
                            tracing::warn!(
                                space_id = %space_id,
                                event_id = %entry.event_id,
                                missing_identity = ?entry.missing_identity,
                                error_code = 4006,
                                "4006 identity_record_timeout: pending event discarded after timeout (Identity record never arrived)"
                            );
                        }
                    }
                }
            }
        });
    }

    // Bind WebSocket server
    let mut server = Server::bind(listen_addr)
        .await
        .with_context(|| format!("failed to bind on {listen_addr}"))?;
    // Write the PID file now that the bind succeeded. Used by `--pid`.
    write_pid_file(data_dir);
    if !opts.quiet {
        println!("Listening on {} — press Ctrl+C to stop", effective_endpoint);
        println!();
    }

    // Named-pipe server (M2) — hosts the four control commands and the Node's
    // read-only `__BATCH__` subset. D-043 pipe-name convention. The watch-
    // channel sender must remain alive for the lifetime of this function's
    // async block — J-071 lesson; if the binding scope ends, the receiver's
    // `.changed()` returns Err immediately and the server breaks.
    let pipe_name_str = crate::pipe::pipe_name(opts.instance_label.as_deref());
    let started_at_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    #[cfg(target_os = "windows")]
    let _pipe_shutdown_hold = {
        let (tx, rx) = tokio::sync::watch::channel(false);
        let pipe_name_owned = pipe_name_str.clone();
        let pipe_data_dir = data_dir.to_path_buf();
        let pipe_config_path = config_path.to_path_buf();
        let pipe_runtime = Arc::clone(&runtime);
        let pipe_federation_registry = Arc::clone(&federation_registry);
        // Option B (J-160): the pipe server threads the live sender maps to the
        // admin layer so the A4 `space force-eject` / `unban` verbs fan their
        // Node-authored event out to connected clients + federated peers live.
        let pipe_client_senders = Arc::clone(&client_senders);
        let pipe_federation_peer_senders = Arc::clone(&federation_peer_senders);
        let pipe_connections = Arc::clone(&connections);
        tokio::spawn(async move {
            crate::pipe::start_pipe_server(
                pipe_name_owned,
                pipe_data_dir,
                pipe_config_path,
                pipe_runtime,
                pipe_federation_registry,
                pipe_client_senders,
                pipe_federation_peer_senders,
                pipe_connections,
                started_at_epoch,
                rx,
            )
            .await;
        });
        tx
    };
    #[cfg(not(target_os = "windows"))]
    {
        let _ = pipe_name_str;
        tracing::warn!(
            "named pipe server is Windows-only; --ping/--health/--stop/--reload-config/--batch will fail on this platform"
        );
    }

    // Accept loop
    loop {
        tokio::select! {
            result = server.accept() => {
                match result {
                    Ok(conn) => {
                        let rt = Arc::clone(&runtime);
                        let conns = Arc::clone(&connections);
                        let senders = Arc::clone(&client_senders);
                        let fed_senders = Arc::clone(&federation_peer_senders);
                        let fed_reg = Arc::clone(&federation_registry);
                        let fed_reg_path = federation_registry_path.clone();
                        let home = node_id.clone();
                        let lm = local_mode;
                        let ids = identities_path.clone();
                        let kp = Arc::clone(&node_keypair);
                        let sdir = spaces_dir.clone();
                        let sbs = sync_batch_size;
                        let req_appr = require_approval;
                        let fed_queue = Arc::clone(&federation_queue);
                        let fed_queue_path = federation_queue_path.clone();
                        tokio::spawn(async move {
                            handle_connection(conn, rt, conns, senders, fed_senders, fed_reg, fed_reg_path, kp, home, lm, ids, sdir, sbs, req_appr, fed_queue, fed_queue_path).await;
                        });
                    }
                    Err(e) => {
                        eprintln!("{}", red(&format!("accept error: {e}")));
                        tracing::error!(reason = %e, "Connection accept error");
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down...");
                tracing::info!("Node shutting down");
                break;
            }
        }
    }

    // Final state write on shutdown
    {
        let rt = runtime.lock().await;
        let conns = connections.lock().await;
        let peers = {
            let reg = federation_registry.lock().await;
            build_federated_peers(&reg)
        };
        let state = build_node_state(
            &rt,
            &conns,
            peers,
            &node_id,
            &effective_endpoint,
            if local_mode { "local" } else { "production" },
            &started_at,
        );
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = std::fs::write(data_dir.join("xgen-node_state.json"), json);
        }
        // Phase 7.5 §5.3 + §5.6 — flush local Space provenance on shutdown.
        // Pass 3 (Surface #5 Q5.12 site b) — boundary projection.
        let metadata_snapshot: std::collections::HashMap<
            String,
            xgen_common::space_local::SpaceLocalMetadata,
        > = rt
            .space_local_metadata
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.clone()))
            .collect();
        save_space_local_metadata(data_dir, &metadata_snapshot);
    }

    // Warn about any events still buffered (pending prev_events that never arrived).
    {
        let rt = runtime.lock().await;
        for (space_id, buf) in &rt.pending {
            if !buf.is_empty() {
                tracing::warn!(space_id = %space_id, unresolved = buf.len(), "pending_buffer_at_shutdown");
            }
        }
    }

    write_session_footer(ExitReason::Shutdown);
    Ok(())
}

// ── Connection handler ─────────────────────────────────────────────────────────

// Wide parameter list — each value comes from a different startup-pipeline
// source (FederationRegistry, ClientSenders, FederationPeerSenders, runtime,
// trace handles, ...) and packing into a struct would force every caller
// to construct an intermediate value used once. Same trade-off rationale as
// `xgen_common::event_trace::write_session_header`.
// `pub(crate)` rather than module-private so the Phase 9 in-process harness
// (`tests/phase9_harness.rs`) can drive accepted connections through the
// production connection path without re-implementing it. Production callers
// remain inside this crate; the visibility relaxation does not export the
// function across the crate boundary.
///
/// Pass 3 (Surface #5 Q5.2) — `home_node_id` retyped to owned `NodeXgid`
/// (forced-owned at the connection-handler boundary; passed deep into
/// `handle_federation_incoming` spawned-task body across awaits).
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_connection(
    mut conn: Connection<TcpStream>,
    runtime: Arc<tokio::sync::Mutex<NodeRuntime>>,
    connections: Connections,
    client_senders: ClientSenders,
    federation_peer_senders: crate::fanout::FederationPeerSenders,
    federation_registry: Arc<tokio::sync::Mutex<FederationRegistry>>,
    federation_registry_path: PathBuf,
    node_keypair: Arc<SigningKey>,
    home_node_id: NodeXgid,
    local_mode: bool,
    identities_path: PathBuf,
    spaces_dir: PathBuf,
    sync_batch_size: usize,
    require_approval: bool,
    federation_queue: Arc<tokio::sync::Mutex<PendingFederationQueue>>,
    federation_queue_path: PathBuf,
) {
    // Transport challenge-response authentication
    //
    // Pass 3 (Surface #5 Q5.1) — server_authenticate returns String (wire-
    // format Identity URI); project to typed IdentityXgid at the boundary.
    let identity_id: IdentityXgid = match conn.server_authenticate().await {
        Ok(id) => IdentityXgid::from_xgid(Xgid::new(id)),
        Err(e) => {
            tracing::error!(reason = %e, "Transport authentication failed");
            return;
        }
    };

    // M6 A5-D1: a revoked Identity is denied session-open immediately. The check
    // reads the *live* in-memory registry, which `admin_ops::identity_revoke`
    // mutates under the same lock (P5 decision) — so revocation takes effect on
    // the very next connection, not on restart. Absent records are admitted
    // (Phase 1 local mode allows unregistered keypairs to authenticate).
    {
        let rt = runtime.lock().await;
        if rt.identity_registry.is_revoked(&identity_id) {
            tracing::warn!(identity_id = %identity_id, "Session-open denied: Identity revoked");
            return;
        }
    }

    // Build session context — Phase 1 local mode: all authenticated sessions are Owner-level.
    // Phase 2 will resolve role from the space registry per space_id.
    let session_ctx = SessionContext {
        identity_id: Some(identity_id.as_str().to_string()),
        role: Some(SpaceRole::Owner),
        space_id: None,
    };

    // Read first message — determines whether this is a federation or client connection
    let first = match conn.recv().await {
        Ok(m) => m,
        Err(_) => return,
    };

    match first {
        // ── Federation connection (another Node connecting) ───────────────
        Inbound::Federation(fm) if matches!(&fm, FederationMessage::Hello { .. }) => {
            tracing::info!(peer_node_id = %identity_id, "Incoming federation connection");
            handle_federation_incoming(
                &mut conn,
                fm,
                runtime,
                node_keypair,
                home_node_id,
                spaces_dir,
                identities_path,
                local_mode,
                client_senders,
                federation_peer_senders,
                federation_registry,
                federation_registry_path,
                require_approval,
                federation_queue,
                federation_queue_path,
            )
            .await;
        }

        // ── Client connection ─────────────────────────────────────────────
        first_msg => {
            tracing::info!(identity_id = %identity_id, "Client authenticated");
            let display_name = {
                let rt = runtime.lock().await;
                rt.identity_registry
                    .get(&identity_id)
                    .and_then(|r| r.display_name.clone())
                    .unwrap_or_default()
            };
            let connected_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let mut events_received: u64 = 0;

            // Register in the active-connection tracker
            {
                let mut conns = connections.lock().await;
                conns.push(ConnectedClientInfo {
                    identity_id: identity_id.clone(),
                    display_name,
                    connected_at,
                    events_received: 0,
                });
            }

            // Outbound channel — the fan-out path and history-push path send
            // OutboundMsg into this channel; the select! arm below drains it
            // and writes to the WebSocket. Capacity is generous so a slow
            // client cannot block other clients' fan-out for long; if full,
            // try_send drops the message (the client will catch up via
            // transport.sync_request after reconnect).
            let (out_tx, mut out_rx) =
                tokio::sync::mpsc::channel::<OutboundMsg>(1024);
            {
                let mut senders = client_senders.lock().await;
                senders.insert(identity_id.clone(), out_tx.clone());
            }

            // Process the first message via the same dispatch as the loop's
            // recv arm — otherwise a sync_request arriving as the first
            // post-auth message is silently dropped (process_inbound has no
            // out_tx in scope).
            let mut deferred_first: Option<Inbound> = Some(first_msg);

            // Main loop: select between inbound recv and outbound drain.
            loop {
                // Drain the deferred first message first, otherwise call recv.
                let incoming: Result<Inbound, _> = if let Some(m) = deferred_first.take() {
                    Ok(m)
                } else {
                    tokio::select! {
                        biased;
                        r = conn.recv() => r,
                        Some(out_msg) = out_rx.recv() => {
                            match out_msg {
                                OutboundMsg::Event(ev) => {
                                    trace_event(&ev, EventDirection::Out, &session_ctx);
                                    if conn.send_event(&ev).await.is_err() {
                                        break;
                                    }
                                }
                                OutboundMsg::HistoryBatch { events } => {
                                    for ev in events {
                                        trace_event(&ev, EventDirection::Out, &session_ctx);
                                        if conn.send_event(&ev).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                OutboundMsg::SyncComplete {
                                    since,
                                    new_tip,
                                    continue_from,
                                } => {
                                    // F-6: explicit end-of-batch signal. Replaces
                                    // the 500ms quiet-time heuristic; the requester
                                    // waits for this message instead of guessing
                                    // via inter-event silence.
                                    let msg = TransportMessage::SyncComplete {
                                        protocol_version: "0.1".to_string(),
                                        since,
                                        new_tip,
                                        continue_from,
                                    };
                                    if conn.send_transport(&msg).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            continue;
                        }
                    }
                };

                // Dispatch the inbound (whether deferred-first or fresh recv).
                match incoming {
                    Ok(Inbound::Transport(TransportMessage::Goodbye { .. }))
                    | Ok(Inbound::Closed) => break,
                    Ok(Inbound::Ping(_)) | Ok(Inbound::Pong(_)) => {}
                    // transport.sync_request is the only Transport variant
                    // the Node responds to from a client (spec 3.3.6 + F-6 + F-7).
                    // After delivering the batch we emit an explicit SyncComplete
                    // with an optional `continue_from` pagination cursor — replaces
                    // the 500ms quiet-time heuristic for end-of-stream detection
                    // (D-065, honest behaviour over polite behaviour).
                    Ok(Inbound::Transport(TransportMessage::SyncRequest {
                        since,
                        limit,
                        ..
                    })) => {
                        // F-7a: per-request limit (None → config default 1000).
                        // sync_batch_size is resolved at run_node startup from
                        // [sync].batch_size (config → default 1000).
                        let effective_limit =
                            limit.map(|n| n as usize).unwrap_or(sync_batch_size);
                        let (events, continue_from) = collect_sync_history(
                            &runtime,
                            &identity_id,
                            &since,
                            effective_limit,
                        )
                        .await;
                        // new_tip: whole-batch model — last delivered event_id,
                        // or echo `since` when the page is empty (caller is
                        // caught up at the position they asked from).
                        // Pass 3 (Surface #5 §4.3 wire-format boundary) —
                        // event_id is EventXgid; project to wire String for
                        // OutboundMsg::SyncComplete.new_tip.
                        let new_tip: String = events
                            .last()
                            .and_then(|e| e.event_id.as_ref().map(|x| x.as_str().to_string()))
                            .unwrap_or_else(|| since.clone());
                        let _ = out_tx
                            .send(OutboundMsg::HistoryBatch { events })
                            .await;
                        let _ = out_tx
                            .send(OutboundMsg::SyncComplete {
                                since: since.clone(),
                                new_tip,
                                continue_from,
                            })
                            .await;
                    }
                    Ok(Inbound::Transport(_)) => {}
                    Ok(msg) => {
                        if let Inbound::Event(ref ev) = msg {
                            trace_event(ev, EventDirection::In, &session_ctx);
                        }
                        events_received += 1;
                        // Origin = LocallySubmitted — client connection is
                        // the origination point for federation-push purposes
                        // (runbook §3.4.1 R15: origin attach at entry points).
                        let fanout = process_inbound(
                            &mut conn,
                            msg,
                            &identity_id,
                            &home_node_id,
                            local_mode,
                            &runtime,
                            &identities_path,
                            &spaces_dir,
                            EventOrigin::LocallySubmitted,
                        )
                        .await;
                        // Stage 5 local fan-out (unchanged).
                        let pushed_event = fanout.event.clone();
                        apply_fanout(fanout, &identity_id, &runtime, &client_senders).await;
                        // Stage 6 federation push (Phase 4 — sibling of
                        // apply_fanout, not a wrapper). Runs for every
                        // accepted event whose origin is LocallySubmitted;
                        // the F-5 guard inside apply_federation_push is
                        // redundant here (this site only fires under
                        // LocallySubmitted) but Phase 4 calls
                        // apply_federation_push uniformly from both client
                        // and federation receive paths — the guard short-
                        // circuits the federation-receive call instead.
                        if let Some(ev) = pushed_event {
                            apply_federation_push(
                                &ev,
                                EventOrigin::LocallySubmitted,
                                &runtime,
                                &federation_peer_senders,
                                &home_node_id,
                            )
                            .await;
                        }
                        let mut conns = connections.lock().await;
                        if let Some(c) =
                            conns.iter_mut().find(|c| c.identity_id == identity_id)
                        {
                            c.events_received = events_received;
                        }
                    }
                    Err(_) => break,
                }
            }

            // Remove from active connections on disconnect
            tracing::info!(identity_id = %identity_id, "Client disconnected");
            {
                let mut senders = client_senders.lock().await;
                senders.remove(&identity_id);
            }
            let mut conns = connections.lock().await;
            if let Some(pos) = conns.iter().position(|c| c.identity_id == identity_id) {
                conns.remove(pos);
            }
        }
    }
}

// ── Federation incoming handler ────────────────────────────────────────────────

/// F-1a federation handshake receiver (runbook §3.3 Locked wire shape +
/// §3.3.1 Locks 1-7). Phase 3 reshape: replaces the pre-F-1a
/// "handshake → space.join_request → state.federation_add → dump → goodbye"
/// flow with "handshake (with bilateral tips) → bilateral delta delivery via
/// stream_federation_delta → session stays open as the F-2 persistent push
/// channel (Phase 4 push lands on top of this)."
///
/// The peer's `shared_spaces` and `tips` from Hello are the source of truth
/// for what to deliver. `SpaceControlMessage::JoinRequest` is no longer part
/// of the post-handshake flow; the receiver determines delta scope from the
/// peer's wire-visible tips per the locked semantics in runbook §3.3.
///
/// Pass 3 (Surface #5 Q5.2) — `home_node_id` retyped to owned `NodeXgid`;
/// the function consumes the value across awaits + passes deep into spawned
/// task bodies (per §4.2 v1.2 row 3 async-spawned-captures forced-owned).
#[allow(clippy::too_many_arguments)]
async fn handle_federation_incoming(
    conn: &mut Connection<TcpStream>,
    hello: FederationMessage,
    runtime: Arc<tokio::sync::Mutex<NodeRuntime>>,
    node_keypair: Arc<SigningKey>,
    home_node_id: NodeXgid,
    spaces_dir: PathBuf,
    identities_path: PathBuf,
    local_mode: bool,
    client_senders: ClientSenders,
    federation_peer_senders: crate::fanout::FederationPeerSenders,
    federation_registry: Arc<tokio::sync::Mutex<FederationRegistry>>,
    federation_registry_path: PathBuf,
    require_approval: bool,
    federation_queue: Arc<tokio::sync::Mutex<PendingFederationQueue>>,
    federation_queue_path: PathBuf,
) {
    // Verify hello signature
    if let Err(e) = verify_msg(&hello) {
        tracing::error!(reason = %e, "Federation hello: invalid signature");
        return;
    }

    // §3.3 Locked wire shape (Option 3 bilateral tips): extract peer's
    // shared_spaces + tips from Hello — these drive both the Capabilities
    // reply (our tips for the same Spaces) and the delta-delivery iteration
    // domain inside stream_federation_delta.
    let (peer_node_id, peer_caps, peer_version, peer_endpoint, peer_shared_spaces, peer_tips) =
        match hello {
            FederationMessage::Hello {
                node_id,
                capabilities,
                protocol_version,
                node_endpoint,
                shared_spaces,
                tips,
                ..
            } => (
                node_id,
                capabilities,
                protocol_version,
                node_endpoint,
                shared_spaces,
                tips,
            ),
            _ => unreachable!(),
        };

    // Negotiate capabilities — "json" is the mandatory baseline.
    let our_caps = FederationCapabilities::default();
    let serial = negotiate_serialisation(&our_caps.serialisation, &peer_caps.serialisation)
        .unwrap_or_else(|| "json".to_string());
    let neg_version =
        negotiate_version("0.1", &peer_version).unwrap_or_else(|| "0.1".to_string());

    // ── federation-admin-control 2a (FAC-D1a) — inbound approval gate ───────────
    //
    // Node-side pause-point (the xgen-core `run_receiving` primitive is left a
    // pure protocol fn; production receiving runs here). Placed right after
    // negotiation and BEFORE we send capabilities — mirroring where
    // `run_receiving` sends its 2001/2002 rejects: refuse before the
    // relationship seals (FAC-D1a reject-with-retry, do not hold the socket).
    //
    // Default-off (`require_approval = false`) → `should_queue_for_approval`
    // returns false unconditionally and this whole block is a no-op: the
    // handshake proceeds to ACTIVE exactly as before (prime invariant). When
    // on, an inbound peer that is not already `Active` in the registry is
    // enqueued for operator `accept`/`reject` and sent `Reject 2003`; the peer
    // gives up this attempt and re-establishes after approval.
    {
        let current_state = {
            let reg = federation_registry.lock().await;
            let peer_typed = NodeXgid::from_xgid(Xgid::new(peer_node_id.clone()));
            reg.get(&peer_typed).map(|r| r.state)
        };
        if should_queue_for_approval(require_approval, current_state) {
            let request = PendingFederationRequest {
                peer_node_id: NodeXgid::from_xgid(Xgid::new(peer_node_id.clone())),
                peer_url: peer_endpoint.clone(),
                received_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                shared_spaces: peer_shared_spaces
                    .iter()
                    .map(|s| SpaceXgid::from_xgid(Xgid::new(s.clone())))
                    .collect(),
                negotiated_version: neg_version.clone(),
                negotiated_serialisation: serial.clone(),
            };
            {
                let mut q = federation_queue.lock().await;
                q.add(request);
                if let Err(e) = q.save(&federation_queue_path) {
                    tracing::warn!(
                        path = ?federation_queue_path,
                        error = %e,
                        "Failed to persist pending federation request queue"
                    );
                }
            }
            let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let reject = sign_msg(
                FederationMessage::Reject {
                    protocol_version: "0.1".to_string(),
                    node_id: home_node_id.as_str().to_string(),
                    error_code: FEDERATION_APPROVAL_PENDING_CODE,
                    error_string: FEDERATION_APPROVAL_PENDING_STRING.to_string(),
                    timestamp: ts,
                    signature: None,
                },
                &node_keypair,
            );
            let _ = conn.send_federation(&reject).await;
            tracing::info!(
                peer_node_id = %peer_node_id,
                "Federation request queued for operator approval (require_approval=true); sent Reject 2003"
            );
            return;
        }
    }

    // Build our local tips for each Space the peer declares as shared. A Space
    // we don't host (no entry in stores) yields no tip; a Space with multiple
    // DAG tips (concurrent forks) picks the lexicographically smallest for
    // wire-shape determinism — Phase 1/2 DAGs are single-tip in practice, but
    // the rule is total. Empty `our_tips` is valid (Locked semantics: "I
    // participate in zero shared Spaces"); absent entry under a non-empty
    // shared_spaces means "send full history" — handled by stream_federation_delta.
    //
    // Pass 3 (Surface #5 §4.3 wire-format boundary) — peer_shared_spaces are
    // wire-format Strings; project to SpaceXgid at the runtime-call boundary.
    let our_tips: BTreeMap<String, String> = {
        let rt = runtime.lock().await;
        peer_shared_spaces
            .iter()
            .filter_map(|space_id| {
                let space_id_typed = SpaceXgid::from_xgid(Xgid::new(space_id.clone()));
                let local_tips = rt.dag_tips(&space_id_typed);
                local_tips.into_iter().min().map(|tip| (space_id.clone(), tip))
            })
            .collect()
    };

    // Send federation.capabilities (signed with node keypair) — carries our tips.
    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let caps_msg = sign_msg(
        FederationMessage::Capabilities {
            protocol_version: "0.1".to_string(),
            // Pass 3 (Surface #5 §4.3 wire-format boundary) — wire emits
            // String via NodeXgid Display projection.
            node_id: home_node_id.as_str().to_string(),
            capabilities: our_caps,
            negotiated: NegotiatedCapabilities {
                serialisation: serial.clone(),
                protocol_version: neg_version.clone(),
            },
            tips: our_tips,
            timestamp: ts,
            signature: None,
        },
        &node_keypair,
    );
    if conn.send_federation(&caps_msg).await.is_err() {
        return;
    }

    // Receive federation.accept — verify signature.
    let accept_msg = match conn.recv().await {
        Ok(Inbound::Federation(fm @ FederationMessage::Accept { .. })) => fm,
        _ => return,
    };
    if verify_msg(&accept_msg).is_err() {
        return;
    }
    let session_id = match accept_msg {
        FederationMessage::Accept { session_id, .. } => session_id,
        _ => return,
    };

    tracing::info!(
        peer_node_id = %peer_node_id,
        shared_spaces_count = peer_shared_spaces.len(),
        "Federation handshake reached ACTIVE"
    );

    // Store the peer's endpoint URL so we can push identity replicas to it later.
    // Clone-and-bind first so the Phase-5 registry upsert below (post-ACTIVE
    // hook) can also use the URL — the existing block consumes `peer_endpoint`.
    let peer_url_for_registry = peer_endpoint.clone();
    if let Some(url) = peer_endpoint {
        let mut rt = runtime.lock().await;
        rt.record_peer_url(&peer_node_id, url);
    }

    // Pass 3 (Surface #5 Q5.2 / Q5.14) — wire-format String values project to
    // owned typed XGIDs for the spawned post-handshake driver call (forced-
    // owned per §4.2 v1.2 row 3 async-spawned-captures sub-rule).
    let peer_node_id_typed = NodeXgid::from_xgid(Xgid::new(peer_node_id.clone()));
    let peer_shared_spaces_typed: Vec<SpaceXgid> = peer_shared_spaces
        .iter()
        .map(|s| SpaceXgid::from_xgid(Xgid::new(s.clone())))
        .collect();

    // Receiver-side post-handshake flow: stream_federation_delta + register
    // + mark_active + F-2 loop + cleanup. Shared with the initiator-side
    // reconnect path (`crate::reconnect::attempt_reconnect`) via
    // `run_federation_session_post_handshake` — the receiver passes
    // `SessionRole::Receiver` so the initiator-only catch-up drain is
    // skipped.
    run_federation_session_post_handshake(
        conn,
        SessionRole::Receiver,
        runtime,
        client_senders,
        federation_peer_senders,
        federation_registry,
        federation_registry_path,
        node_keypair,
        home_node_id,
        spaces_dir,
        identities_path,
        local_mode,
        peer_node_id_typed,
        session_id,
        neg_version,
        serial,
        peer_shared_spaces_typed,
        peer_tips,
        peer_url_for_registry,
    )
    .await;
}

// ── Post-handshake federation session driver ──────────────────────────────────

/// Which side of the federation session we are. Drives the small flow-
/// asymmetry inside `run_federation_session_post_handshake` — initiators
/// drain inbound until the receiver's `SyncComplete` before sending their
/// own delta; receivers stream their delta first and let the F-2 loop's
/// inbound arm consume the initiator's delta naturally as it arrives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionRole {
    Initiator,
    Receiver,
}

/// Shared post-handshake federation session driver — runs everything from
/// "handshake just reached ACTIVE" through "session ended, registry
/// updated." Called from both the receiver-side `handle_federation_incoming`
/// (after `run_receiving` + caps + accept) and the initiator-side
/// `crate::reconnect::attempt_reconnect` (after `run_initiating`).
///
/// The flow per side:
/// - **Initiator**: drain inbound until the receiver's `SyncComplete`
///   arrives (the receiver streamed its delta synchronously after handshake);
///   then stream our own delta (bilateral §3.3.1 Lock 7 R5 production caller);
///   register out_tx + mark_active; F-2 loop; cleanup.
/// - **Receiver**: stream our delta; register out_tx + mark_active; F-2 loop;
///   the loop's inbound arm consumes the initiator's delta as it arrives.
///
/// Pass 3 (Surface #5 Q5.14 v1.3 + T11) — per-parameter retype matrix:
/// `home_node_id` + `peer_node_id` → owned `NodeXgid` (forced-owned per
/// §4.2 v1.2 row 3 async-spawned-captures); `session_id` + `neg_version` +
/// `serial` stay `String` (descriptive-string slots); `peer_shared_spaces`
/// → `Vec<SpaceXgid>` (in-memory typed vec); `peer_tips` stays
/// `BTreeMap<String, String>` (§4.3 wire-format boundary; wire-derived from
/// TransportMessage Hello/Capabilities); `peer_url` stays `Option<String>`
/// (URL descriptive per §5.4).
#[allow(clippy::too_many_arguments)]
pub(crate) async fn run_federation_session_post_handshake<S>(
    conn: &mut Connection<S>,
    our_role: SessionRole,
    runtime: Arc<tokio::sync::Mutex<NodeRuntime>>,
    client_senders: ClientSenders,
    federation_peer_senders: crate::fanout::FederationPeerSenders,
    federation_registry: Arc<tokio::sync::Mutex<FederationRegistry>>,
    federation_registry_path: PathBuf,
    node_keypair: Arc<SigningKey>,
    home_node_id: NodeXgid,
    spaces_dir: PathBuf,
    identities_path: PathBuf,
    local_mode: bool,
    peer_node_id: NodeXgid,
    session_id: String,
    neg_version: String,
    serial: String,
    peer_shared_spaces: Vec<SpaceXgid>,
    peer_tips: BTreeMap<String, String>,
    peer_url: Option<String>,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send,
{
    let fed_session_ctx = SessionContext {
        identity_id: Some(peer_node_id.as_str().to_string()),
        role: Some(SpaceRole::Owner),
        space_id: None,
    };

    // Initiator-side: drain the receiver's catch-up (events ending in
    // SyncComplete) BEFORE streaming our own delta. Without this drain the
    // two sides would interleave sends and the receiver's catch-up events
    // would land in our F-2 loop alongside steady-state pushes — not wrong
    // semantically, but it would break the §3.3.1 Lock 7 ordering
    // expectation that catch-up completes before steady-state begins.
    // Pass 3 (Surface #5 Q5.1 Q3-overload) — process_inbound + apply_fanout
    // signatures take &IdentityXgid for their identity_id / author_id slots;
    // federation sessions overload the Identity-URI principal slot with the
    // peer Node URI. Build an IdentityXgid projection from the typed peer_node_id
    // once and pass at every Q3-overloaded call site below.
    let peer_as_identity =
        IdentityXgid::from_xgid(Xgid::new(peer_node_id.as_str().to_string()));

    if our_role == SessionRole::Initiator {
        loop {
            match conn.recv().await {
                Ok(Inbound::Transport(TransportMessage::SyncComplete { .. })) => break,
                Ok(Inbound::Event(ev)) => {
                    trace_event(&ev, EventDirection::In, &fed_session_ctx);
                    let fanout = process_inbound(
                        conn,
                        Inbound::Event(ev),
                        &peer_as_identity,
                        &home_node_id,
                        local_mode,
                        &runtime,
                        &identities_path,
                        &spaces_dir,
                        EventOrigin::ReceivedViaFederation,
                    )
                    .await;
                    let pushed_event = fanout.event.clone();
                    apply_fanout(fanout, &peer_as_identity, &runtime, &client_senders).await;
                    if let Some(ev) = pushed_event {
                        apply_federation_push(
                            &ev,
                            EventOrigin::ReceivedViaFederation,
                            &runtime,
                            &federation_peer_senders,
                            &home_node_id,
                        )
                        .await;
                    }
                }
                Ok(Inbound::Ping(_)) | Ok(Inbound::Pong(_)) => {}
                Ok(Inbound::Closed) | Err(_) => {
                    tracing::warn!(
                        peer_node_id = %peer_node_id.as_str(),
                        "Connection dropped during initiator-side catch-up drain"
                    );
                    return;
                }
                Ok(_) => {
                    // Federation messages would be unexpected here; ignore.
                }
            }
        }
    }

    // Both sides: stream our delta to the peer. For receiver-side this is
    // the existing Phase-3 behaviour. For initiator-side this is the
    // §3.3.1 Lock 7 R5 production caller mandated by Phase 3 → Phase 5
    // sequencing.
    if let Err(e) = stream_federation_delta(
        conn,
        &runtime,
        &peer_shared_spaces,
        &peer_tips,
        &peer_node_id,
        &session_id,
        &neg_version,
        &serial,
        &node_keypair,
        &spaces_dir,
    )
    .await
    {
        tracing::warn!(
            peer_node_id = %peer_node_id.as_str(),
            error = %e,
            role = ?our_role,
            "Federation delta delivery failed; session terminating"
        );
        return;
    }

    tracing::info!(
        peer_node_id = %peer_node_id.as_str(),
        role = ?our_role,
        "Federation delta delivery complete; session stays open"
    );

    // Phase 4 (runbook §3.4.1 Q2 lock + R12 lifecycle): wire the outbound
    // mpsc and register the sender into the shared FederationPeerSenders
    // registry so apply_federation_push (called from other connection
    // handlers when local clients post events) can drain into this peer's
    // session. Channel sized 1024 to match the client-connection precedent.
    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<OutboundMsg>(1024);
    {
        let mut fed = federation_peer_senders.lock().await;
        fed.insert(peer_node_id.clone(), out_tx.clone());
    }

    // Phase 5 (runbook §3.5.1 Lock A + Lock B2) — "successful reconnect
    // means handshake completes to ACTIVE state": this is that site for
    // BOTH sides. The receiver's hook and the initiator's hook converge
    // here. mark_active clears `next_reconnect_attempt` so the scheduler
    // stops trying; the matching attempt-count entry is dropped by the
    // scheduler itself (see crate::reconnect::AttemptCursor).
    {
        let mut reg = federation_registry.lock().await;
        let now = Utc::now();
        let last_connected = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        reg.upsert(FederationRelationship {
            peer_node_id: peer_node_id.clone(),
            shared_spaces: peer_shared_spaces.clone(),
            negotiated_version: neg_version.clone(),
            negotiated_serialisation: serial.clone(),
            session_id: session_id.clone(),
            last_connected,
            peer_url,
            state: FederationState::Active,
        });
        reg.mark_active(&peer_node_id, now);
        if let Err(e) = reg.save(&federation_registry_path) {
            tracing::warn!(
                path = ?federation_registry_path,
                error = %e,
                peer_node_id = %peer_node_id.as_str(),
                "Failed to persist federation registry on session-ACTIVE"
            );
        }
    }

    // F-2 long-lived continuous session — Phase 4 operational. Inbound arm
    // dispatches federation-received events through process_inbound with
    // EventOrigin::ReceivedViaFederation (runbook §3.4.1 R15: origin attach
    // at entry points) and applies local fan-out; apply_federation_push is
    // called uniformly but short-circuits on the F-5 anti-transitivity
    // guard for ReceivedViaFederation events. Outbound arm drains
    // OutboundMsg events that other connection handlers'
    // apply_federation_push pushed into out_tx.
    loop {
        tokio::select! {
            biased;
            r = conn.recv() => {
                match r {
                    Ok(Inbound::Transport(TransportMessage::Goodbye { .. }))
                    | Ok(Inbound::Closed)
                    | Err(_) => break,
                    Ok(Inbound::Ping(_)) | Ok(Inbound::Pong(_)) => {}
                    Ok(Inbound::Event(ev)) => {
                        trace_event(&ev, EventDirection::In, &fed_session_ctx);
                        // peer_node_id is passed as the wire-authenticated
                        // sender; process_inbound's identity_id parameter
                        // accepts any pubkey URI — see runbook §3.4 Q3 lock.
                        // Pass 3 Q3-overload: peer_as_identity built above.
                        let fanout = process_inbound(
                            conn,
                            Inbound::Event(ev),
                            &peer_as_identity,
                            &home_node_id,
                            local_mode,
                            &runtime,
                            &identities_path,
                            &spaces_dir,
                            EventOrigin::ReceivedViaFederation,
                        )
                        .await;
                        let pushed_event = fanout.event.clone();
                        apply_fanout(fanout, &peer_as_identity, &runtime, &client_senders).await;
                        if let Some(ev) = pushed_event {
                            apply_federation_push(
                                &ev,
                                EventOrigin::ReceivedViaFederation,
                                &runtime,
                                &federation_peer_senders,
                                &home_node_id,
                            )
                            .await;
                        }
                    }
                    Ok(_) => {
                        // Other inbound types not expected on a federation
                        // session in Phase 4 scope. Silently ignore.
                    }
                }
            }
            Some(out_msg) = out_rx.recv() => {
                match out_msg {
                    OutboundMsg::Event(ev) => {
                        if conn.send_event(&ev).await.is_err() {
                            // Send failure during steady-state push: peer is
                            // gone or socket broken. Exit so deregistration
                            // runs and apply_federation_push future calls
                            // see this peer as absent (R14 drop-on-peer-down).
                            break;
                        }
                    }
                    OutboundMsg::HistoryBatch { events } => {
                        for ev in events {
                            if conn.send_event(&ev).await.is_err() {
                                break;
                            }
                        }
                    }
                    OutboundMsg::SyncComplete { since, new_tip, continue_from } => {
                        let msg = TransportMessage::SyncComplete {
                            protocol_version: "0.1".to_string(),
                            since,
                            new_tip,
                            continue_from,
                        };
                        if conn.send_transport(&msg).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }

    // R12 lifecycle: deregister from FederationPeerSenders on session end.
    {
        let mut fed = federation_peer_senders.lock().await;
        fed.remove(&peer_node_id);
    }

    // Phase 5 (runbook §3.5.1 Lock A + Lock B3) — flip the F-1c operational
    // record to lost and schedule the first reconnect attempt at +15min.
    // Catches all five session-end paths (Goodbye / Inbound::Closed / recv
    // error / outbound send error / keepalive-error-as-recv-error) since
    // they all converge here.
    {
        let mut reg = federation_registry.lock().await;
        reg.mark_lost(&peer_node_id, Utc::now());
        if let Err(e) = reg.save(&federation_registry_path) {
            tracing::warn!(
                path = ?federation_registry_path,
                error = %e,
                peer_node_id = %peer_node_id.as_str(),
                "Failed to persist federation registry on session-end"
            );
        }
    }

    tracing::info!(peer_node_id = %peer_node_id.as_str(), role = ?our_role, "Federation session ended");
}

// ── M6 accept / reject signal builders (§3.2 / §3.3) ────────────────────────────

/// M6 §3.2 — build the positive accept signal (`EventAccepted`) for a
/// just-validated-and-persisted event, if one is owed. Owed only for
/// locally-submitted events with a real `event_id`: the originator is the
/// connected client. Federation-received events' originator is on another Node,
/// so no ack is owed (returns `None`). Pure (no I/O) so the emission decision is
/// unit-testable; the caller performs the actual `send_transport`.
fn accept_signal(origin: EventOrigin, event_id: &str, accepted_at: String) -> Option<TransportMessage> {
    if matches!(origin, EventOrigin::LocallySubmitted) && event_id != "(none)" {
        Some(TransportMessage::EventAccepted {
            protocol_version: "0.1".to_string(),
            event_id: event_id.to_string(),
            accepted_at,
        })
    } else {
        None
    }
}

/// M6 §3.3 — build the rejection signal (`Error`) symmetric to `accept_signal`,
/// if one is owed (same locally-submitted-only gate). `error_code` is the §2.7
/// `GENERIC_4000` band: `DispatchOutcome::Rejected` carries an opaque reason
/// string at this layer, so the reason is the human detail and a structured
/// per-reason code taxonomy is a future refinement. The `event_id` is the
/// load-bearing correlation primitive (J-081 §5 / D-070).
fn reject_signal(
    origin: EventOrigin,
    event_id: &str,
    reason: &str,
    timestamp: String,
) -> Option<TransportMessage> {
    if matches!(origin, EventOrigin::LocallySubmitted) && event_id != "(none)" {
        Some(TransportMessage::Error {
            protocol_version: "0.1".to_string(),
            error_code: 4000,
            error_string: reason.to_string(),
            timestamp,
            event_id: Some(event_id.to_string()),
        })
    } else {
        None
    }
}

// ── Inbound message processor ──────────────────────────────────────────────────

/// Process an inbound message from an authenticated wire peer.
///
/// `identity_id`: the wire-authenticated sender — an Identity URI for client
/// connections OR a Node URI for federation peer sessions (runbook §3.4 Q3
/// lock). The wire shape (`xgen://pubkey/ed25519:...`) is identical in both
/// cases; the dispatcher uses this value as the "who sent this on the wire"
/// trace context. Downstream validation does not depend on which kind of
/// principal this is — `dispatch_event`'s checks (signature, sender
/// registration where required, membership, permission) are uniform.
///
/// `origin`: F-5 anti-transitivity annotation (Phase 4, runbook §3.4.1 Q1
/// lock). The caller passes `EventOrigin::LocallySubmitted` from a client
/// connection or `EventOrigin::ReceivedViaFederation` from a federation
/// peer session. The value flows through to `apply_federation_push`'s
/// anti-transitivity guard.
#[allow(clippy::too_many_arguments)]
///
/// Pass 3 (Surface #5 Q5.1) — `identity_id` retyped to `&IdentityXgid` (in-
/// memory slot from authenticated session state); `home_node_id` retyped to
/// `&NodeXgid` (in-memory slot from runtime). Note the Q3-overload: when
/// `origin == ReceivedViaFederation`, `identity_id` carries the peer Node URI
/// (Identity-URI bytes also serve as Node-URI in the principal-flavour space
/// per Phase 4 §3.4.1 Q3 lock); the dispatch_event call site projects the
/// IdentityXgid bytes into a temporary NodeXgid wrapper for the F-3 check.
async fn process_inbound<S>(
    conn: &mut Connection<S>,
    msg: Inbound,
    identity_id: &IdentityXgid,
    home_node_id: &NodeXgid,
    local_mode: bool,
    runtime: &Arc<tokio::sync::Mutex<NodeRuntime>>,
    identities_path: &Path,
    spaces_dir: &Path,
    origin: EventOrigin,
) -> FanoutRequest
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    match msg {
        Inbound::Identity(im) => {
            handle_identity_msg(conn, im, identity_id, home_node_id, local_mode, runtime, identities_path).await;
            FanoutRequest::none()
        }
        Inbound::IdentityReplicate(irm) => {
            handle_identity_replicate_msg(conn, irm, runtime, spaces_dir).await;
            FanoutRequest::none()
        }
        Inbound::Event(event) => {
            // F-4 unified dispatcher (Phase 2 of Federation Event Propagation).
            // Replaces the pre-F-4 three-path branching (Path A messages via
            // `accept_message`; Path B `MembershipJoin` direct `ingest_event`
            // with no validation; Path C state.* direct `ingest_event` with no
            // validation). After F-4 every event family routes through
            // `NodeRuntime::dispatch_event`, which runs the validation core
            // (signature + timestamp + predecessor presence + DAG structure +
            // sender / membership / permission per F-4b) before semantic
            // pre-checks (AI role / capability / operator target) and ingest.
            //
            // HeldPending now applies uniformly to all event families per
            // F-4a (30 s timeout, shared `PendingBuffer`). Drain re-dispatches
            // unblocked events through the full pipeline — see
            // `NodeRuntime::drain_pending_uniform`.
            let event_id = event
                .event_id
                .as_ref()
                .map(|e| e.as_str().to_string())
                .unwrap_or_else(|| "(none)".to_string());
            let event_type_str = event.event_type.to_string();
            // Pass 3 (Surface #5 §4.3 persistence-format boundary) — space_id
            // for persist_event stays String at the call boundary per Q5.9.
            let space_id_for_persist: String = if event.space_id.as_str().is_empty() {
                event
                    .event_id
                    .as_ref()
                    .map(|e| e.as_str().to_string())
                    .unwrap_or_default()
            } else {
                event.space_id.as_str().to_string()
            };

            let mut rt = runtime.lock().await;
            // Phase 7 Lock C1 (runbook §3.7.1) — F-3 federation-relationship
            // check inside dispatch_event consults `peer_node_id`. For
            // federation-channel events the value is sourced from the
            // Q3-overloaded `identity_id` parameter (peer Node URI per
            // §3.4.1 Q3 lock). Locally-submitted events pass None.
            //
            // Pass 3 (Surface #5 Q5.1 Q3-overload) — `identity_id: &IdentityXgid`
            // carries the peer Node URI for federation events; project to a
            // temporary `NodeXgid` borrow for the F-3 check signature.
            let peer_node_id_owned: Option<NodeXgid> = match origin {
                EventOrigin::ReceivedViaFederation => Some(NodeXgid::from_xgid(Xgid::new(
                    identity_id.as_str().to_string(),
                ))),
                EventOrigin::LocallySubmitted => None,
            };
            let outcome = rt.dispatch_event(event.clone(), origin, peer_node_id_owned.as_ref());
            // Phase 7.5 §6 federation-relationship arrival hook is fired
            // INSIDE dispatch_event on successful state.federation_add
            // ingestion (xgen-core/src/node/runtime.rs Step 7). The hook
            // lives in the dispatcher so every caller — production
            // process_inbound, test direct dispatch, future M6 admin
            // write-path — exercises it uniformly under the same runtime
            // lock as ingest. Phase 7.5 Commit 3.5 (B3 amendment) moved
            // this from app.rs into runtime.rs; no app-side action needed.
            drop(rt);

            match outcome {
                DispatchOutcome::Accepted {
                    new_joiner,
                    additional_persisted,
                } => {
                    persist_event(spaces_dir, &space_id_for_persist, &event);
                    // Phase 7.5 persistence-amendment Q2 — persist the
                    // drain-derived events surfaced via additional_persisted.
                    // Without this loop the drained events live only in
                    // EventStore + SpaceState (in-memory); on Node restart
                    // they would never replay from disk because they were
                    // never written. persist_event is per-event idempotent
                    // (duplicate-event_id guard inside) so re-fires from
                    // future drain paths are safe.
                    //
                    // Per-event persist is best-effort: persist_event already
                    // swallows write errors via let _ = std::fs::write(...).
                    // Closing those silent writes is in candidate D-NNN
                    // "ingest path invariant encoding under bidirectional
                    // sustainability discipline" scope (see ingest_event's
                    // verbatim block at xgen-core/src/node/runtime.rs:181);
                    // do NOT broaden scope here without Joe-lock at a future
                    // audit phase.
                    //
                    // Per-event space_id resolution: drained events generally
                    // have different space_id than the triggering event
                    // (cross-Space buffering case for predecessor-drain;
                    // same-Space typical for fed-relationship drain). Resolve
                    // each drained event's own persist key honestly.
                    for drained in &additional_persisted {
                        // Pass 3 (Surface #5 §4.3 persistence-format boundary)
                        // — String per-event space key for persist_event.
                        let drained_space: String = if drained.space_id.as_str().is_empty() {
                            drained
                                .event_id
                                .as_ref()
                                .map(|e| e.as_str().to_string())
                                .unwrap_or_default()
                        } else {
                            drained.space_id.as_str().to_string()
                        };
                        if !drained_space.is_empty() {
                            persist_event(spaces_dir, &drained_space, drained);
                        }
                    }
                    trace_local(
                        LocalAction::StoreEvent,
                        &event_id,
                        Some(&event_type_str),
                        Some(&space_id_for_persist),
                        None,
                    );
                    trace_local(
                        LocalAction::ApplyEvent,
                        &event_id,
                        Some(&event_type_str),
                        Some(&space_id_for_persist),
                        None,
                    );
                    // M6 §3.2 — positive accept signal (G2). The event is
                    // validated AND durably persisted (persist_event above), and
                    // local fan-out has not yet begun (apply_fanout runs after
                    // process_inbound returns). `accept_signal` owes one only for
                    // locally-submitted events (the originator is this connected
                    // client; federation-received events' originator is on another
                    // Node, so no ack is owed). Best-effort send — a failure means
                    // the originator's connection is gone, in which case sync
                    // catch-up handles their eventual view.
                    if let Some(sig) = accept_signal(
                        origin,
                        &event_id,
                        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    ) {
                        let _ = conn.send_transport(&sig).await;
                    }
                    FanoutRequest {
                        event: Some(event),
                        new_joiner,
                    }
                }
                DispatchOutcome::HeldPending => {
                    tracing::debug!(
                        space_id = %space_id_for_persist,
                        event_id = %event_id,
                        event_type = %event_type_str,
                        "event buffered — waiting for unknown prev_events"
                    );
                    FanoutRequest::none()
                }
                DispatchOutcome::Rejected(reason) => {
                    // Phase 9 G2: stable trace event for the unified rejection
                    // wrapper. Fires for every DispatchOutcome::Rejected — the
                    // co-located rejection signal that any future audit-log
                    // wiring (M6 Phase 2) keys off. Inner cause is carried in
                    // `reason`; specific rejection sites (`f3_reject`,
                    // `validation_reject`) fire their own `event` field too,
                    // so test observers can target either layer.
                    tracing::error!(
                        event = "event_rejected",
                        space_id = %space_id_for_persist,
                        event_id = %event_id,
                        event_type = %event_type_str,
                        reason = %reason,
                        "process_inbound: event rejected"
                    );
                    trace_local(
                        LocalAction::RejectEvent,
                        &event_id,
                        Some(&event_type_str),
                        Some(&space_id_for_persist),
                        None,
                    );
                    // M6 §3.3 — rejection signal, symmetric to EventAccepted
                    // (D-070, "two events of equal importance, opposite
                    // direction"). Before M6 the originator received NO signal on
                    // rejection (J-081 §5 finding — reject paths only traced); now
                    // an Error carries the rejected event's event_id so the client
                    // can correlate it to its in-flight submission. See
                    // `reject_signal` for the GENERIC_4000 / opaque-reason rationale
                    // and the locally-submitted-only gate.
                    if let Some(sig) = reject_signal(
                        origin,
                        &event_id,
                        &reason,
                        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    ) {
                        let _ = conn.send_transport(&sig).await;
                    }
                    FanoutRequest::none()
                }
            }
        }
        _ => FanoutRequest::none(),
    }
}


// ── Identity message handler ───────────────────────────────────────────────────

///
/// Pass 3 (Surface #5 Q5.3) — `authenticated_id` retyped to `&IdentityXgid`,
/// `home_node_id` retyped to `&NodeXgid`. In-memory slots from authenticated
/// session state. accept_registration retains String-flavoured signature per
/// xgen-core wire-format-builder convention; project at boundary.
async fn handle_identity_msg<S>(
    conn: &mut Connection<S>,
    msg: IdentityMessage,
    authenticated_id: &IdentityXgid,
    home_node_id: &NodeXgid,
    local_mode: bool,
    runtime: &Arc<tokio::sync::Mutex<NodeRuntime>>,
    identities_path: &Path,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    match msg {
        IdentityMessage::Register { .. } => {
            let already = {
                let rt = runtime.lock().await;
                rt.identity_registry.contains(authenticated_id)
            };
            let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            match accept_registration(
                &msg,
                authenticated_id.as_str(),
                already,
                local_mode,
                home_node_id.as_str(),
                &ts,
            ) {
                Ok(record) => {
                    let identity_id_str = authenticated_id.as_str().to_string();
                    let node_keypair_clone = {
                        let mut rt = runtime.lock().await;
                        let _ = rt.identity_registry.register(record.clone());
                        let _ = rt.identity_registry.save(identities_path);
                        rt.node_keypair.clone()
                    };
                    let ok = IdentityMessage::RegisterOk {
                        protocol_version: "0.1".to_string(),
                        identity_id: identity_id_str.clone(),
                        registered_at: ts,
                    };
                    let _ = conn.send_identity(&ok).await;
                    tracing::info!(identity_id = %identity_id_str, "Identity registered");
                    // Replicate to peer Nodes asynchronously (spec 3.13.1).
                    let rt_clone = Arc::clone(runtime);
                    tokio::spawn(async move {
                        push_identity_to_peers(record, rt_clone, node_keypair_clone).await;
                    });
                }
                Err(e) => {
                    let (code, msg_str) = e.to_registration_code();
                    let fail = IdentityMessage::RegisterFail {
                        protocol_version: "0.1".to_string(),
                        error_code: code,
                        error_string: msg_str.to_string(),
                        timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    };
                    let _ = conn.send_identity(&fail).await;
                    tracing::warn!(identity_id = %authenticated_id.as_str(), reason = %msg_str, "Identity registration rejected");
                }
            }
        }
        IdentityMessage::Get { identity_id, .. } => {
            // Pass 3 (Surface #5 §4.3 wire-format boundary) — identity_id is
            // wire-format String from IdentityMessage::Get destructure; project
            // to typed at the registry-call boundary.
            let identity_id_typed = IdentityXgid::from_xgid(Xgid::new(identity_id.clone()));
            let rt = runtime.lock().await;
            let response = match rt.identity_registry.get(&identity_id_typed) {
                // Pass 3 (Surface #5 §4.3 wire-format boundary) — record
                // fields are typed; project to String for the wire response.
                Some(record) => IdentityMessage::Record {
                    protocol_version: "0.1".to_string(),
                    identity_id: record.identity_id.as_str().to_string(),
                    display_name: record.display_name.clone(),
                    registered_at: record.registered_at.clone(),
                    devices: record
                        .devices
                        .iter()
                        .map(|d| IdentityDeviceEntry {
                            device_id: d.device_id.clone(),
                            device_name: d.device_name.clone(),
                            authorised_at: d.authorised_at.clone(),
                        })
                        .collect(),
                    home_node: record.home_node.as_str().to_string(),
                },
                None => IdentityMessage::NotFound {
                    protocol_version: "0.1".to_string(),
                    identity_id,
                },
            };
            drop(rt);
            let _ = conn.send_identity(&response).await;
        }
        _ => {}
    }
}

// ── Identity replication — inbound (spec 3.13.4) ──────────────────────────────

/// Handle an incoming `identity.replicate` message on this Node (acting as a replica).
/// Accepts or rejects the record, then sends `identity.replicate_ack` or a transport error.
async fn handle_identity_replicate_msg<S>(
    conn: &mut Connection<S>,
    msg: IdentityReplicateMessage,
    runtime: &Arc<tokio::sync::Mutex<NodeRuntime>>,
    spaces_dir: &Path,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (identity_id, identity_record_value, update_version) = match msg {
        IdentityReplicateMessage::Replicate {
            identity_id,
            identity_record,
            update_version,
            ..
        } => (identity_id, identity_record, update_version),
        // replicate_ack arriving here would be a protocol error — ignore silently.
        IdentityReplicateMessage::ReplicateAck { .. } => return,
    };

    // Deserialise identity_record Value → IdentityRecord.
    let record: IdentityRecord = match serde_json::from_value(identity_record_value) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(identity_id = %identity_id, reason = %e, "identity.replicate: invalid record payload");
            return;
        }
    };

    let result = {
        let mut rt = runtime.lock().await;
        let outcome = handle_incoming_replicate(record, &mut rt.identity_registry);
        // Phase 6 / F-10 — fire the Identity-arrival hook on successful
        // upsert. `drain_pending_by_identity` iterates per-Space
        // PendingBuffers (runbook §3.6.1 Lock A2 cross-Space fan-out) and
        // re-dispatches any events that were buffered pending this signer's
        // Identity record. Called inside the same runtime lock as the
        // upsert so a buffered event cannot miss a just-landed identity
        // due to lock-release reordering.
        if outcome.is_ok() {
            // Phase 7.5 persistence-amendment Q2 — drain_pending_by_identity
            // returns Vec<Event> of Accepted-drained events for caller-side
            // persistence. Same shape as process_inbound's
            // additional_persisted loop above. Per-event idempotent via
            // persist_event's duplicate-guard. Loop runs inside the runtime
            // lock (same critical section as the drain itself) to keep the
            // "buffered event cannot miss a just-landed identity due to
            // lock-release reordering" invariant from the existing doc-
            // comment of this hook.
            // Pass 3 (Surface #5 Q5.4 wire-format boundary + Surface #2 Q2.5)
            // — identity_id is wire-format String (IdentityReplicateMessage
            // destructure); project to typed at drain_pending_by_identity
            // call boundary (helper signature retyped &IdentityXgid).
            let identity_id_typed =
                IdentityXgid::from_xgid(Xgid::new(identity_id.clone()));
            let drained = rt.drain_pending_by_identity(
                &identity_id_typed,
                EventOrigin::ReceivedViaFederation,
            );
            for ev in &drained {
                // Re-resolve space_id per drained event (drain spans Spaces;
                // each event's own space_id is the persist key).
                let target_space: String = if ev.space_id.as_str().is_empty() {
                    ev.event_id
                        .as_ref()
                        .map(|e| e.as_str().to_string())
                        .unwrap_or_default()
                } else {
                    ev.space_id.as_str().to_string()
                };
                if !target_space.is_empty() {
                    persist_event(spaces_dir, &target_space, ev);
                }
            }
        }
        outcome
    };

    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    match result {
        Ok(()) => {
            let ack = IdentityReplicateMessage::ReplicateAck {
                protocol_version: "0.1".to_string(),
                identity_id: identity_id.clone(),
                update_version,
                timestamp: ts,
                signature: None,
            };
            let _ = conn.send_identity_replicate(&ack).await;
            tracing::info!(identity_id = %identity_id, update_version = update_version, "Identity replica accepted");
        }
        Err(e) => {
            tracing::warn!(identity_id = %identity_id, reason = %e, "identity.replicate: rejected");
            // Send transport error 3020 so the home Node can handle the stale-version case.
            let err_msg = TransportMessage::Error {
                protocol_version: "0.1".to_string(),
                error_code: e.error_code(),
                error_string: e.to_string(),
                timestamp: ts,
                // identity-replicate failure is not a DAG-event submission, so no
                // event to correlate (M6 §3.3 — transport errors not tied to an
                // event leave event_id None).
                event_id: None,
            };
            let _ = conn.send_transport(&err_msg).await;
        }
    }
}

// ── Identity replication — outbound (spec 3.13.1) ─────────────────────────────

/// Push a newly registered Identity record to all known peer Nodes (spec 3.13.1).
/// Runs asynchronously after registration — failures are logged but not fatal.
async fn push_identity_to_peers(
    record: IdentityRecord,
    runtime: Arc<tokio::sync::Mutex<NodeRuntime>>,
    node_keypair: ed25519_dalek::SigningKey,
) {
    // Snapshot peer_urls under the lock, then release before any I/O.
    // Pass 3 (Surface #5 Q5.5 inheritance) — peer_urls keys are NodeXgid;
    // Vec is owned-clone for cross-await safety.
    let peer_urls: Vec<(NodeXgid, String)> = {
        let rt = runtime.lock().await;
        rt.peer_urls.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    };

    if peer_urls.is_empty() {
        return;
    }

    let identity_record_value = match serde_json::to_value(&record) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(reason = %e, "push_identity_to_peers: serialise failed");
            return;
        }
    };
    let identity_id = record.identity_id.clone();
    let update_version = record.update_version;

    for (peer_node_id, url) in peer_urls {
        let iid = identity_id.clone();
        let val = identity_record_value.clone();
        let kp = node_keypair.clone();
        let rt = Arc::clone(&runtime);

        tokio::spawn(async move {
            match connect_url(&url).await {
                Err(e) => {
                    tracing::warn!(peer = %peer_node_id.as_str(), url = %url, reason = %e, "replication: connect failed");
                }
                Ok(mut conn) => {
                    if conn.client_authenticate(&kp).await.is_err() {
                        tracing::warn!(peer = %peer_node_id.as_str(), "replication: authenticate failed");
                        return;
                    }
                    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
                    // Pass 3 (Surface #5 §4.3 wire-format boundary) — iid
                    // is IdentityXgid; project to wire-format String for the
                    // replicate message body.
                    let replicate = IdentityReplicateMessage::Replicate {
                        protocol_version: "0.1".to_string(),
                        identity_id: iid.as_str().to_string(),
                        identity_record: val,
                        update_version,
                        timestamp: ts,
                        signature: None,
                    };
                    if conn.send_identity_replicate(&replicate).await.is_err() {
                        tracing::warn!(peer = %peer_node_id.as_str(), "replication: send failed");
                        return;
                    }
                    // Wait for ack (best-effort; timeout is handled by recv() WebSocket layer).
                    match conn.recv().await {
                        Ok(Inbound::IdentityReplicate(IdentityReplicateMessage::ReplicateAck { .. })) => {
                            tracing::info!(identity_id = %iid.as_str(), peer = %peer_node_id.as_str(), "Replication ack received");
                            let mut rt_guard = rt.lock().await;
                            rt_guard.replica_registry.add_replica(iid.as_str(), peer_node_id.as_str());
                        }
                        other => {
                            tracing::warn!(identity_id = %iid.as_str(), peer = %peer_node_id.as_str(), msg = ?other, "replication: unexpected response");
                        }
                    }
                }
            }
        });
    }
}

// ── State file builder ─────────────────────────────────────────────────────────

fn build_node_state(
    rt: &NodeRuntime,
    conns: &[ConnectedClientInfo],
    peers: Vec<FederatedPeer>,
    node_id: &NodeXgid,
    endpoint: &str,
    mode: &str,
    started_at: &str,
) -> NodeState {
    let spaces = rt
        .spaces
        .values()
        .map(|space| {
            let store = rt.stores.get(&space.space_id);
            let total_events = store.map(|s| s.len() as u64).unwrap_or(0);

            let rooms = space
                .rooms
                .values()
                .map(|room| {
                    let room_events = store
                        .map(|s| s.values().filter(|e| e.room_id == room.room_id).count() as u64)
                        .unwrap_or(0);
                    HostedRoom {
                        room_id: room.room_id.clone(),
                        name: room.name.clone(),
                        event_count: room_events,
                        last_activity: String::new(),
                    }
                })
                .collect();

            HostedSpace {
                space_id: space.space_id.clone(),
                name: space.name.clone().unwrap_or_else(|| {
                    let sid = space.space_id.as_str();
                    sid[..sid.len().min(20)].to_string()
                }),
                member_count: space.members.len(),
                event_count: total_events,
                rooms,
            }
        })
        .collect();

    let clients = conns
        .iter()
        .map(|c| ConnectedClient {
            identity_id: c.identity_id.clone(),
            display_name: c.display_name.clone(),
            connected_at: c.connected_at.clone(),
            events_sent: 0,
            events_received: c.events_received,
        })
        .collect();

    // Phase 6 / F-10 (runbook §3.6.1 Lock C2): sum across all Spaces'
    // PendingBuffers of events currently held pending Identity-record
    // arrival. Exposes operators to the F-10 §13.7 "Identity replication
    // is the bottleneck" diagnostic via state.json.
    let pending_identity_replication: usize = rt
        .pending
        .values()
        .map(|buf| buf.pending_identity_count())
        .sum();

    // Phase 7.5 §8.2: sibling counter for the third HeldPending trigger.
    // Operators can detect "this Node is waiting on a federation_add to
    // bootstrap" via this counter alongside the f3_reject trace events.
    let pending_federation_relationship: usize = rt
        .pending
        .values()
        .map(|buf| buf.pending_federation_relationship_count())
        .sum();

    NodeState {
        node_id: node_id.clone(),
        version: build_info::VERSION.to_string(),
        build: build_info::GIT_HASH.to_string(),
        started_at: started_at.to_string(),
        mode: mode.to_string(),
        endpoint: endpoint.to_string(),
        updated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        clients,
        peers,
        spaces,
        pending_identity_replication,
        pending_federation_relationship,
    }
}

/// Phase 9 / G1 observability — render the federation registry's per-peer
/// records into the `FederatedPeer` shape exported in `xgen-node_state.json`.
///
/// Source data: `FederationRegistry::peer_records` (operational state per
/// Phase 5 runbook §3.5.1 Lock A) joined to `FederationRegistry::get(peer)`
/// (the protocol relationship, when one exists). The two are independent —
/// a peer can have an operational record from a previous session without a
/// current relationship entry, and vice versa — so we union the keys.
///
/// `state` is derived from `lost_connection`: "DISCONNECTED" when the
/// operational record flags the peer as lost; "ACTIVE" otherwise. This
/// reflects the registry's view; in-flight `FederationPeerSenders` presence
/// is a finer-grained signal not consulted here.
fn build_federated_peers(reg: &FederationRegistry) -> Vec<FederatedPeer> {
    let mut peers: Vec<FederatedPeer> = Vec::new();
    // Pass 3 (Surface #5 inheritance) — seen set retyped to HashSet<NodeXgid>
    // since rec.peer_node_id is already NodeXgid post-Pass-1.
    let mut seen: std::collections::HashSet<NodeXgid> = std::collections::HashSet::new();

    for rec in reg.peer_records() {
        seen.insert(rec.peer_node_id.clone());
        let rel = reg.get(&rec.peer_node_id);
        peers.push(FederatedPeer {
            node_id: rec.peer_node_id.clone(),
            endpoint: rel.and_then(|r| r.peer_url.clone()).unwrap_or_default(),
            state: if rec.lost_connection { "DISCONNECTED" } else { "ACTIVE" }.to_string(),
            session_id: rel.map(|r| r.session_id.clone()).unwrap_or_default(),
            version: rel.map(|r| r.negotiated_version.clone()).unwrap_or_default(),
            protocol: rel.map(|r| r.negotiated_serialisation.clone()).unwrap_or_default(),
            shared_spaces: rel.map(|r| r.shared_spaces.clone()).unwrap_or_default(),
            connected_at: rel.map(|r| r.last_connected.clone()).unwrap_or_default(),
            last_seen_at: rec.last_seen.clone(),
            lost_connection: rec.lost_connection,
            last_successful_session: rec.last_successful_session.clone(),
            next_reconnect_attempt: rec.next_reconnect_attempt.clone(),
        });
    }

    // Surface relationships that have no operational record yet (e.g., a
    // relationship entry present from a prior session before Phase 5 wired
    // the operational record). Defaults to ACTIVE so operators are not misled
    // into thinking the peer is down.
    for rel in reg.all() {
        if seen.contains(&rel.peer_node_id) {
            continue;
        }
        peers.push(FederatedPeer {
            node_id: rel.peer_node_id.clone(),
            endpoint: rel.peer_url.clone().unwrap_or_default(),
            state: "ACTIVE".to_string(),
            session_id: rel.session_id.clone(),
            version: rel.negotiated_version.clone(),
            protocol: rel.negotiated_serialisation.clone(),
            shared_spaces: rel.shared_spaces.clone(),
            connected_at: rel.last_connected.clone(),
            last_seen_at: rel.last_connected.clone(),
            lost_connection: false,
            last_successful_session: None,
            next_reconnect_attempt: None,
        });
    }

    peers.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    peers
}

// ── init ───────────────────────────────────────────────────────────────────────

pub fn cmd_init(data_dir: &Path, passphrase_arg: Option<&str>) -> Result<()> {
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("failed to create data directory: {}", data_dir.display()))?;

    let keypair_file = data_dir.join("xgen-node_keypair.enc");
    let config_file = data_dir.join("xgen-node_config.toml");

    if keypair_file.exists() {
        println!("Keypair already exists: {}", keypair_file.display());
        println!("Skipping keypair generation. Delete the file to regenerate.");
    } else {
        println!("Generating keypair...");
        let passphrase = match passphrase_arg {
            Some(p) => p.to_string(),
            None => prompt_passphrase()?,
        };
        let signing_key = keypair::generate();
        keypair::save(&signing_key, &keypair_file, &passphrase)
            .context("failed to save keypair")?;
        println!("Keypair saved:  {}", keypair_file.display());
        println!("Node ID:        {}", pubkey_uri(&signing_key));
    }

    if config_file.exists() {
        println!("Config already exists: {} — not overwritten.", config_file.display());
    } else {
        let mut cfg = NodeConfig::default();
        // Point keypair_path to this data_dir, not exe_dir
        cfg.paths.keypair_path = keypair_file.to_string_lossy().to_string();
        let toml_str = toml::to_string_pretty(&cfg).context("failed to serialise config")?;
        std::fs::write(&config_file, toml_str).context("failed to write config")?;
        println!("Config saved:   {}", config_file.display());
    }

    println!();
    println!("Run 'xgen-node --config {}' to start.", config_file.display());
    Ok(())
}

// ── status ─────────────────────────────────────────────────────────────────────

pub fn cmd_status(data_dir: &Path) -> Result<()> {
    let state = load_state(data_dir)?;
    let age = age_seconds(&state.updated_at);

    let total_events: u64 = state.spaces.iter().map(|s| s.event_count).sum();

    println!("xgen-node status");
    println!("================");
    println!("Node ID:      {}", state.node_id);
    println!("Version:      {}", state.version);
    println!("Uptime:       {}", uptime_str(&state.started_at));
    println!("Mode:         {}", state.mode);
    println!("Endpoint:     {}", state.endpoint);
    println!(
        "Connections:  {} client{}, {} federated peer{}",
        state.clients.len(),
        plural(state.clients.len()),
        state.peers.len(),
        plural(state.peers.len()),
    );
    println!("Spaces:       {} hosted", state.spaces.len());
    println!("Events:       {} total across all spaces", total_events);
    if age > 30 {
        println!(
            "State file:   {}",
            yellow(&format!("WARNING — updated {}s ago (Node may not be running)", age))
        );
    } else {
        println!("State file:   updated {}s ago", age);
    }
    Ok(())
}

// ── connections ────────────────────────────────────────────────────────────────

pub fn cmd_connections(data_dir: &Path) -> Result<()> {
    let state = load_state(data_dir)?;

    println!(
        "Connections ({} client{}, {} peer{})",
        state.clients.len(),
        plural(state.clients.len()),
        state.peers.len(),
        plural(state.peers.len()),
    );

    if state.clients.is_empty() && state.peers.is_empty() {
        println!("\n  No active connections.");
        return Ok(());
    }

    if !state.clients.is_empty() {
        println!();
        println!("CLIENTS");
        println!(
            "  {:<44}  {:<16}  {:<14}  {:<12}  Received",
            "Identity", "Display name", "Connected", "Events sent"
        );
        for c in &state.clients {
            println!(
                "  {:<44}  {:<16}  {:<14}  {:<12}  {}",
                short_id(c.identity_id.as_str()),
                c.display_name,
                format_ago(age_seconds(&c.connected_at)),
                c.events_sent,
                c.events_received,
            );
        }
    }

    if !state.peers.is_empty() {
        println!();
        println!("FEDERATED PEERS");
        println!(
            "  {:<44}  {:<30}  {:<10}  Since",
            "Node ID", "Endpoint", "State"
        );
        for p in &state.peers {
            println!(
                "  {:<44}  {:<30}  {:<10}  {}",
                short_id(p.node_id.as_str()),
                p.endpoint,
                p.state,
                format_ago(age_seconds(&p.connected_at)),
            );
        }
    }
    Ok(())
}

// ── spaces ─────────────────────────────────────────────────────────────────────

pub fn cmd_spaces(data_dir: &Path) -> Result<()> {
    let state = load_state(data_dir)?;

    println!("Spaces ({})", state.spaces.len());

    if state.spaces.is_empty() {
        println!("\n  No hosted Spaces.");
        return Ok(());
    }

    for space in &state.spaces {
        println!();
        println!("  Space: {}", space.name);
        println!("  ID:    {}", space.space_id);
        println!(
            "  Rooms: {}   Members: {}   Events: {}",
            space.rooms.len(),
            space.member_count,
            space.event_count
        );
        for room in &space.rooms {
            let activity = if room.last_activity.is_empty() {
                "no activity yet".to_string()
            } else {
                format!("{} ago", fmt_duration(age_seconds(&room.last_activity)))
            };
            println!();
            println!("    Room: {}", room.name);
            println!("    ID:   {}", room.room_id);
            println!("    Events: {}   Last activity: {}", room.event_count, activity);
        }
    }
    Ok(())
}

// ── peers ──────────────────────────────────────────────────────────────────────

pub fn cmd_peers(data_dir: &Path) -> Result<()> {
    let state = load_state(data_dir)?;

    println!("Federated Peers ({})", state.peers.len());

    if state.peers.is_empty() {
        println!("\n  No known federated peers.");
        return Ok(());
    }

    for peer in &state.peers {
        println!();
        println!("  Node ID:     {}", peer.node_id);
        println!("  Endpoint:    {}", peer.endpoint);
        println!("  State:       {}", peer.state);
        println!("  Session ID:  {}", peer.session_id);
        println!("  Version:     {} / {}", peer.version, peer.protocol);
        if !peer.shared_spaces.is_empty() {
            println!("  Spaces:      {}", peer.shared_spaces.join(", "));
        }
        println!("  Connected:   {}", format_ago(age_seconds(&peer.connected_at)));
        println!("  Last seen:   {}", format_ago(age_seconds(&peer.last_seen_at)));
    }
    Ok(())
}

// ── identity list ──────────────────────────────────────────────────────────────

pub fn cmd_identity_list(data_dir: &Path) -> Result<()> {
    let identities_path = data_dir.join("xgen-node_identities.db");

    let registry = IdentityRegistry::load(&identities_path).with_context(|| {
        format!(
            "failed to load identity registry at {}\n  Is the Node initialised? Run 'xgen-node init'.",
            identities_path.display()
        )
    })?;

    let mut all = registry.all();
    all.sort_by(|a, b| a.registered_at.cmp(&b.registered_at));

    println!("Registered Identities ({})", all.len());

    if all.is_empty() {
        println!("\n  No identities registered.");
        return Ok(());
    }

    println!();
    for record in all {
        let name = record.display_name.as_deref().unwrap_or("<no name>");
        let age = fmt_registration_age(&record.registered_at);
        println!(
            "  {}   {:<20}  registered {}   {} device{}",
            record.identity_id,
            name,
            age,
            record.devices.len(),
            plural(record.devices.len()),
        );
    }
    Ok(())
}

// ── version ────────────────────────────────────────────────────────────────────

pub fn cmd_version(config_path: &Path, data_dir: &Path) -> Result<()> {
    println!("xgen-node {}", build_info::full_version());
    println!("Commit:   {}", build_info::GIT_HASH);

    let cfg = try_load_config(config_path);
    let keypair_path = cfg
        .map(|c| c.paths.keypair_path)
        .unwrap_or_else(|| data_dir.join("xgen-node_keypair.enc").to_string_lossy().to_string());
    let keypair_path = PathBuf::from(&keypair_path);

    if keypair_path.exists() {
        match keypair::load(&keypair_path, "") {
            Ok(signing_key) => println!("Node ID:  {}", pubkey_uri(&signing_key)),
            Err(_) => println!(
                "Node ID:  (keypair is passphrase-protected — use 'xgen-node status' when running)"
            ),
        }
    } else {
        println!("Node ID:  (no keypair — run 'xgen-node init')");
    }
    Ok(())
}

// ── whoami ─────────────────────────────────────────────────────────────────────

/// Prints `node_id` (xgen://pubkey/...) by loading the keypair. The
/// operator_display_name lives on the NodeAnnouncement record, not the local
/// config today; that field will surface here once announcement metadata is
/// stored locally (post-M1).
pub fn cmd_whoami(config_path: &Path, data_dir: &Path) -> Result<()> {
    let cfg = try_load_config(config_path);
    let keypair_path = cfg
        .map(|c| c.paths.keypair_path)
        .unwrap_or_else(|| {
            data_dir
                .join("xgen-node_keypair.enc")
                .to_string_lossy()
                .to_string()
        });
    let keypair_path = PathBuf::from(&keypair_path);
    if !keypair_path.exists() {
        bail!(
            "no keypair found at {}\n  Run 'xgen-node init' to initialise this Node folder.",
            keypair_path.display()
        );
    }
    let signing_key = keypair::load(&keypair_path, "")
        .with_context(|| format!("failed to load keypair from {}", keypair_path.display()))?;
    println!("Node ID:                 {}", pubkey_uri(&signing_key));
    println!("operator_display_name:   (not in local config — see NodeAnnouncement metadata)");
    Ok(())
}

// ── check-config / print-config ───────────────────────────────────────────────

/// `--check-config`: parse the config (defaults if missing), print OK / first
/// validation error, exit 0 on success and non-zero on failure.
pub fn cmd_check_config(config_path: &Path) -> Result<()> {
    if !config_path.exists() {
        println!(
            "config OK: {} (file absent — defaults will apply)",
            config_path.display()
        );
        return Ok(());
    }
    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("cannot read {}", config_path.display()))?;
    let _: NodeConfig = toml::from_str(&content)
        .with_context(|| format!("invalid TOML in {}", config_path.display()))?;
    println!("config OK: {}", config_path.display());
    Ok(())
}

/// `--print-config`: serialise the effective config (file merged with
/// defaults) to TOML on stdout. Read-only, no pipe contact.
pub fn cmd_print_config(config_path: &Path) -> Result<()> {
    let cfg = try_load_config(config_path).unwrap_or_default();
    let s = toml::to_string_pretty(&cfg).context("failed to serialise config to TOML")?;
    print!("{}", s);
    Ok(())
}

// ── pid ────────────────────────────────────────────────────────────────────────

const PID_FILE_NAME: &str = "xgen-node.pid";

/// Write the current process PID into `<data_dir>/xgen-node.pid`. Called by
/// `run_node` immediately after the WS bind succeeds. Silent on I/O failure
/// (the PID file is a convenience for the `--pid` flag, not load-bearing).
fn write_pid_file(data_dir: &Path) {
    let pid = std::process::id();
    let path = data_dir.join(PID_FILE_NAME);
    let _ = std::fs::write(&path, pid.to_string());
}

/// `--pid`: read `<data_dir>/xgen-node.pid` and print its contents.
/// No liveness check — a stale file remains until overwritten by the next
/// resident or removed manually.
pub fn cmd_pid(data_dir: &Path) -> Result<()> {
    let path = data_dir.join(PID_FILE_NAME);
    let pid_str = std::fs::read_to_string(&path)
        .with_context(|| format!("no resident PID file at {}", path.display()))?;
    println!("{}", pid_str.trim());
    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Directory of the running executable (Tier 1 files co-located with binary).
///
/// On Windows calls GetModuleFileNameW directly — this is immune to CWD, PATH
/// lookup, symlinks, and any shell wrapper tricks. On other platforms uses the
/// standard library's current_exe(). Panics if the path cannot be determined;
/// the binary cannot operate safely without knowing where it lives.
pub fn exe_dir() -> PathBuf {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let mut buf: Vec<u16> = vec![0u16; 260]; // start at MAX_PATH
        loop {
            // NULL module handle → path of the running exe
            let n = unsafe {
                windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW(
                    std::ptr::null_mut(), // NULL → path of the running exe
                    buf.as_mut_ptr(),
                    buf.len() as u32,
                )
            };
            if n == 0 {
                panic!("GetModuleFileNameW failed — cannot determine executable path");
            }
            if n < buf.len() as u32 {
                let path = PathBuf::from(OsString::from_wide(&buf[..n as usize]));
                return path
                    .parent()
                    .expect("executable path has no parent directory")
                    .to_path_buf();
            }
            // Buffer was full — double it and retry (handles paths beyond MAX_PATH)
            let new_len = buf.len() * 2;
            buf.resize(new_len, 0);
        }
    }

    #[cfg(not(windows))]
    {
        std::env::current_exe()
            .expect("cannot determine executable path")
            .parent()
            .expect("executable path has no parent directory")
            .to_path_buf()
    }
}

fn try_load_config(path: &Path) -> Option<NodeConfig> {
    let text = std::fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()
}

/// Resolve the spaces directory the same way `run_node` does (config override,
/// else `<data_dir>/spaces`). Used by the M6 A4 `space force-eject` / `unban`
/// admin verbs to persist the Node-authored `membership.node_eject` /
/// `node_unban` events to the same on-disk location the resident replays from.
pub(crate) fn resolve_spaces_dir(config_path: &Path, data_dir: &Path) -> PathBuf {
    try_load_config(config_path)
        .and_then(|c| c.paths.spaces_dir)
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("spaces"))
}

/// Read `[logging].level` from `xgen-node_config.toml`, returning `None` if
/// the file is missing or fails to parse. Used by both Node entry-points
/// (`run_node` here for `--service`, `desktop::init_logging` for the Tauri
/// shell) so both feed `xgen_common::precedence::resolve_log_level` (D-068).
pub fn read_config_log_level(config_path: &Path) -> Option<String> {
    try_load_config(config_path).map(|c| c.logging.level)
}

/// Parse a ws://host:port/path URL to a SocketAddr for binding.
fn parse_ws_addr(url: &str) -> Result<SocketAddr> {
    let stripped = url
        .strip_prefix("ws://")
        .or_else(|| url.strip_prefix("wss://"))
        .with_context(|| format!("expected ws:// or wss:// URL, got: {url}"))?;
    let host_port = stripped.split('/').next().unwrap_or(stripped);
    host_port
        .parse::<SocketAddr>()
        .with_context(|| format!("invalid address in WebSocket URL: {host_port}"))
}

/// Rewrite the port component of a `ws://host:port/path` URL. Used to
/// reconstruct the effective endpoint string after `--port` override per
/// D-068 — so banner output, session headers, state-file writes, and
/// tracing all show the actual bound port rather than the config-as-written
/// value. Returns the original `url` unchanged if it does not start with
/// `ws://` or `wss://`.
pub(crate) fn rewrite_url_port(url: &str, new_port: u16) -> String {
    let (scheme, rest) = if let Some(r) = url.strip_prefix("ws://") {
        ("ws", r)
    } else if let Some(r) = url.strip_prefix("wss://") {
        ("wss", r)
    } else {
        return url.to_string();
    };
    let (host_port, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    let host = host_port.rsplit_once(':').map(|(h, _)| h).unwrap_or(host_port);
    format!("{scheme}://{host}:{new_port}{path}")
}

/// Load the Node state file from the data directory (Tier 1 — co-located with config).
fn load_state(data_dir: &Path) -> Result<NodeState> {
    let path = data_dir.join("xgen-node_state.json");
    if !path.exists() {
        bail!(
            "state file not found: {}\n  Is the Node running? Start it with 'xgen-node'.",
            path.display()
        );
    }
    let json = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read state file: {}", path.display()))?;
    serde_json::from_str(&json).context("state file is corrupt or has an unexpected format")
}

fn pubkey_uri(signing_key: &SigningKey) -> String {
    let encoded = encoding::encode(signing_key.verifying_key().as_bytes());
    format!("xgen://pubkey/ed25519:{}", encoded)
}

/// Truncate a full xgen:// URI for table display.
fn short_id(uri: &str) -> String {
    let rest = uri
        .strip_prefix("xgen://hash/")
        .or_else(|| uri.strip_prefix("xgen://pubkey/"))
        .unwrap_or(uri);
    if let Some((scheme, key)) = rest.split_once(':') {
        let trunc: String = key.chars().take(8).collect();
        format!("{scheme}:{trunc}...")
    } else {
        let trunc: String = uri.chars().take(20).collect();
        format!("{trunc}...")
    }
}

/// Seconds since the given RFC 3339 timestamp, or i64::MAX on parse error.
fn age_seconds(timestamp: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_seconds())
        .unwrap_or(i64::MAX)
}

/// "2h 14m 38s" — used for uptime display.
fn uptime_str(started_at: &str) -> String {
    let secs = age_seconds(started_at);
    if secs <= 0 {
        return "unknown".to_string();
    }
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

/// "14m 22s" — used in table columns.
fn fmt_duration(secs: i64) -> String {
    if secs <= 0 {
        return "0s".to_string();
    }
    if secs < 60 {
        return format!("{}s", secs);
    }
    let m = secs / 60;
    let s = secs % 60;
    if m < 60 {
        return format!("{}m {:02}s", m, s);
    }
    let h = m / 60;
    let m = m % 60;
    format!("{}h {}m", h, m)
}

fn format_ago(secs: i64) -> String {
    format!("{} ago", fmt_duration(secs))
}

fn fmt_registration_age(timestamp: &str) -> String {
    let secs = age_seconds(timestamp);
    if secs < 0 {
        return "just now".to_string();
    }
    if secs < 120 {
        return format!("{}s ago", secs);
    }
    let m = secs / 60;
    if m < 120 {
        return format!("{}m ago", m);
    }
    let h = m / 60;
    if h < 48 {
        return format!("{}h ago", h);
    }
    format!("{}d ago", h / 24)
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn prompt_passphrase() -> Result<String> {
    let pass =
        rpassword::prompt_password("Passphrase: ").context("failed to read passphrase")?;
    let confirm =
        rpassword::prompt_password("Confirm:    ").context("failed to read passphrase")?;
    if pass != confirm {
        bail!("Passphrases do not match.");
    }
    Ok(pass)
}

// ── Space event persistence (Fix 16) ──────────────────────────────────────────

/// Filename-safe representation of a space_id for use as a JSON store filename.
fn space_file_name(space_id: &str) -> String {
    let clean = space_id
        .strip_prefix("xgen://hash/sha256:")
        .map(|h| format!("sha256_{}", h))
        .unwrap_or_else(|| space_id.replace(['/', ':', '.'], "_"));
    format!("{}.json", clean)
}

/// Append one Event to the per-Space JSON store.
/// Idempotent — won't write duplicates (matched by event_id).
/// Read the persisted DAG events for one Space from its on-disk store
/// (`<spaces_dir>/<space_file>`), the same JSON-array file `persist_event` writes.
/// Returns an empty vec if the file is missing or unparseable. Used by the
/// protocol-audit rebuild (`space audit-rebuild`) to replay a Space's events.
pub(crate) fn read_persisted_events(spaces_dir: &Path, space_id: &str) -> Vec<Event> {
    if space_id.is_empty() {
        return Vec::new();
    }
    let path = spaces_dir.join(space_file_name(space_id));
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub(crate) fn persist_event(spaces_dir: &Path, space_id: &str, event: &Event) {
    if space_id.is_empty() {
        return;
    }
    let _ = std::fs::create_dir_all(spaces_dir);
    let path = spaces_dir.join(space_file_name(space_id));
    let mut events: Vec<Event> = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    // Avoid duplicate entries.
    // Pass 3 (Surface #5 §4.3 persistence-format boundary) — event_id is
    // EventXgid; compare via String projection at the persistence layer.
    if let Some(id) = &event.event_id {
        let id_str = id.as_str();
        if events
            .iter()
            .any(|e| e.event_id.as_ref().map(|x| x.as_str()) == Some(id_str))
        {
            return;
        }
    }
    events.push(event.clone());
    if let Ok(json) = serde_json::to_string(&events) {
        let _ = std::fs::write(&path, json);
    }

    // PAL-D1 (protocol-audit-log arc, J-165 + checkpoint #1) — the single
    // protocol-audit writer hook. This is the persist chokepoint every accept
    // path funnels through. It fires AFTER the per-`event_id` dedup early-return
    // above, so only the first write of an event_id audits (idempotent by
    // construction); `replay_spaces_from_dir` uses `ingest_event`, not this
    // function, so the hook never re-fires on restart. Best-effort for protocol
    // liveness but LOUD on failure (PAL-D2). NOT the A6 admin trail (`audit.rs`).
    // The sink is resolved from the process-global installed in `run_node`
    // (Shape β); absent (unit tests that never call `run_node`) → no audit, the
    // event's own persistence above is unaffected.
    if let Some(sink) = crate::protocol_audit::ProtocolAuditSink::global() {
        sink.record(event);
    }
}

// ── Phase 7.5 — SpaceLocalMetadata persistence (§5.3 + §5.6) ──────────────────
//
// Receiver-local provenance for each Space ("which peer introduced this Space
// to us"). Persisted alongside the other Tier-1 system files at
// `<data_dir>/xgen-node_space_local_metadata.json` so the introducer attribution
// survives Node restarts and is available to operators (currently via raw JSON;
// M6 (new) admin work surfaces it through a CLI verb). Saved by the same 5s
// state-writer task that maintains xgen-node_state.json; loaded once at
// startup ahead of Space event replay so dispatched events see the existing
// metadata before any new write attempts.

const SPACE_LOCAL_METADATA_FILE: &str = "xgen-node_space_local_metadata.json";

fn load_space_local_metadata(
    data_dir: &Path,
) -> std::collections::HashMap<String, xgen_common::space_local::SpaceLocalMetadata> {
    let path = data_dir.join(SPACE_LOCAL_METADATA_FILE);
    if !path.exists() {
        return std::collections::HashMap::new();
    }
    match std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
    {
        Some(map) => map,
        None => {
            tracing::warn!(
                path = ?path,
                "space_local_metadata file present but failed to parse; starting fresh"
            );
            std::collections::HashMap::new()
        }
    }
}

fn save_space_local_metadata(
    data_dir: &Path,
    metadata: &std::collections::HashMap<String, xgen_common::space_local::SpaceLocalMetadata>,
) {
    let path = data_dir.join(SPACE_LOCAL_METADATA_FILE);
    if let Ok(json) = serde_json::to_string_pretty(metadata) {
        let _ = std::fs::write(&path, json);
    }
}

/// Scan `spaces_dir` for *.json Space event stores and replay all events through
/// `NodeRuntime::ingest_event`. Returns the number of Space files replayed.
///
/// This MUST be called before the network listener opens (spec 4.8.5).
///
/// `pub(crate)` for the Phase 9 in-process harness — Scenario 3
/// (drop-and-recover) needs to replay the surviving Node's on-disk state when
/// the harness respawns the binary's runtime. Production callers remain in
/// `run_node`.
pub(crate) fn replay_spaces_from_dir(runtime: &mut NodeRuntime, spaces_dir: &Path) -> usize {
    let entries = match std::fs::read_dir(spaces_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let events: Vec<Event> = match std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            Some(e) => e,
            None => continue,
        };
        if events.is_empty() {
            continue;
        }
        // Phase 7.5 persistence-amendment Q1 (a).ii defensive layer (runbook §4.6).
        // On-disk events arrive in store-iteration order (the order they were
        // serialised to JSON, which tracks `HashMap.values()` insertion order
        // — not guaranteed to respect the DAG). Sort topologically here so
        // `graph.add_event` sees predecessors in causal order and does not
        // surface as `graph_add_event_failed` tracing::error spam for events
        // that are legitimately stored on disk but arrived in a DAG-violating
        // sequence. Under Option Y (Q1(a).iii.α — see ingest_event's verbatim
        // code-comment block at the graph.add_event call site), errors are
        // log-level visible; the sort minimises spurious noise without
        // changing semantics. Single source of truth re-export from xgen-core
        // per D-067 + D-076 no-drift-surface family.
        for event in topological_sort(events) {
            runtime.ingest_event(event);
        }
        count += 1;
    }
    count
}

/// ANSI red — applied only when stderr is a terminal.
pub fn red(s: &str) -> String {
    use std::io::IsTerminal;
    if std::io::stderr().is_terminal() {
        format!("\x1b[31m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

/// ANSI yellow — applied only when stderr is a terminal.
fn yellow(s: &str) -> String {
    use std::io::IsTerminal;
    if std::io::stderr().is_terminal() {
        format!("\x1b[33m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── FAC-D1 [federation] config section (Commit 2) ───────────────────────────

    #[test]
    fn federation_require_approval_defaults_false() {
        // The prime default-off invariant: a fresh config has approval off.
        assert!(!FederationSection::default().require_approval);
        assert!(!NodeConfig::default().federation.require_approval);
    }

    #[test]
    fn config_without_federation_section_loads_require_approval_false() {
        // A pre-2a config (no [federation] table) must still parse, with
        // require_approval defaulting to false (today's auto-establish).
        let toml_src = r#"
            [node]
            listen = "ws://127.0.0.1:8080/xgen"
            local_mode = true

            [paths]
            keypair_path = "xgen-node_keypair.enc"

            [logging]
            level = "info"
        "#;
        let cfg: NodeConfig = toml::from_str(toml_src).expect("pre-2a config must parse");
        assert!(!cfg.federation.require_approval);
    }

    #[test]
    fn config_with_federation_require_approval_true_parses() {
        let toml_src = r#"
            [node]
            listen = "ws://127.0.0.1:8080/xgen"
            local_mode = true

            [paths]
            keypair_path = "xgen-node_keypair.enc"

            [logging]
            level = "info"

            [federation]
            require_approval = true
        "#;
        let cfg: NodeConfig = toml::from_str(toml_src).expect("config with [federation] must parse");
        assert!(cfg.federation.require_approval);
    }

    // rewrite_url_port — shipped from the CLI Precedence Audit (D-068, J-079)
    // alongside the `--port` flag threading. Exercises the port-substitution
    // helper used to reconstruct the effective endpoint string after override.

    // ── M6 Phase 2 — accept/reject signal emission decision (§3.2 / §3.3) ────────

    #[test]
    fn accept_signal_owed_for_local_submission() {
        let sig = accept_signal(
            EventOrigin::LocallySubmitted,
            "xgen://hash/sha256:abc",
            "2026-05-29T12:00:00.000Z".to_string(),
        )
        .expect("accept signal owed for local submission");
        assert_eq!(sig.event_id(), Some("xgen://hash/sha256:abc"));
        match sig {
            TransportMessage::EventAccepted { event_id, accepted_at, .. } => {
                assert_eq!(event_id, "xgen://hash/sha256:abc");
                assert_eq!(accepted_at, "2026-05-29T12:00:00.000Z");
            }
            _ => panic!("expected EventAccepted"),
        }
    }

    #[test]
    fn accept_signal_not_owed_for_federation_received() {
        // The federation peer is not the originator — no ack owed.
        assert!(accept_signal(
            EventOrigin::ReceivedViaFederation,
            "xgen://hash/sha256:abc",
            "2026-05-29T12:00:00.000Z".to_string(),
        )
        .is_none());
    }

    #[test]
    fn accept_signal_not_owed_without_event_id() {
        assert!(accept_signal(
            EventOrigin::LocallySubmitted,
            "(none)",
            "2026-05-29T12:00:00.000Z".to_string(),
        )
        .is_none());
    }

    #[test]
    fn reject_signal_carries_event_id_and_generic_4000() {
        let sig = reject_signal(
            EventOrigin::LocallySubmitted,
            "xgen://hash/sha256:def",
            "federation_relationship_missing: peer X",
            "2026-05-29T12:00:00.000Z".to_string(),
        )
        .expect("reject signal owed for local submission");
        // Correlation primitive populated (the J-081 §5 gap closure).
        assert_eq!(sig.event_id(), Some("xgen://hash/sha256:def"));
        match sig {
            TransportMessage::Error { error_code, error_string, event_id, .. } => {
                assert_eq!(error_code, 4000); // §2.7 GENERIC_4000 band
                assert_eq!(error_string, "federation_relationship_missing: peer X");
                assert_eq!(event_id.as_deref(), Some("xgen://hash/sha256:def"));
            }
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn reject_signal_not_owed_for_federation_received() {
        assert!(reject_signal(
            EventOrigin::ReceivedViaFederation,
            "xgen://hash/sha256:def",
            "some reason",
            "2026-05-29T12:00:00.000Z".to_string(),
        )
        .is_none());
    }

    #[test]
    fn log_filter_state_to_directive_serialises_default_and_modules() {
        let mut st = LogFilterState {
            default: "info".to_string(),
            ..Default::default()
        };
        st.modules.insert("xgen_node::federation".to_string(), "debug".to_string());
        st.modules.insert("xgen_core".to_string(), "warn".to_string());
        // BTreeMap → deterministic (lexicographic) module order.
        assert_eq!(st.to_directive(), "info,xgen_core=warn,xgen_node::federation=debug");
    }

    #[test]
    fn rewrite_url_port_replaces_port_in_ws_url_with_path() {
        let r = rewrite_url_port("ws://127.0.0.1:8080/xgen", 9192);
        assert_eq!(r, "ws://127.0.0.1:9192/xgen");
    }

    #[test]
    fn rewrite_url_port_replaces_port_in_wss_url() {
        let r = rewrite_url_port("wss://example.com:443/xgen", 8443);
        assert_eq!(r, "wss://example.com:8443/xgen");
    }

    #[test]
    fn rewrite_url_port_preserves_path_with_multiple_segments() {
        let r = rewrite_url_port("ws://127.0.0.1:8080/xgen/v1", 9192);
        assert_eq!(r, "ws://127.0.0.1:9192/xgen/v1");
    }

    #[test]
    fn rewrite_url_port_handles_url_with_no_path() {
        let r = rewrite_url_port("ws://127.0.0.1:8080", 9192);
        assert_eq!(r, "ws://127.0.0.1:9192");
    }

    #[test]
    fn rewrite_url_port_returns_unchanged_for_unrecognised_scheme() {
        let r = rewrite_url_port("http://example.com:80/", 8080);
        assert_eq!(r, "http://example.com:80/");
    }

    // ── Phase 7.5 persistence-amendment Commit 2 tests ─────────────────────
    //
    // Test list (2 of 2 — locked at Joe-lock Option Y after Q1(a).iii.β
    // → (a).iii.α revert. Original 4-test list at runbook §4.8 reduced to
    // 2 because tests 1, 2, 4 targeted the Result-shape regression that
    // (a).iii.α no longer exposes. See milestone-close J-108 retrospective
    // for the two-step Checkpoint-#2 correction sequence.):
    //
    //   - `replay_spaces_from_dir_topologically_sorts_before_ingest`
    //   - `topological_sort_publicly_reachable_from_xgen_node`

    /// Q1 (a).ii defensive layer regression lock. Write events to a per-test
    /// `spaces` directory in DAG-violating order (room_create before its
    /// parent state.space_create). Without `topological_sort` before each
    /// `ingest_event`, `graph.add_event` would tracing::error on room_create's
    /// reference to a parent not yet in the store (post-Option-Y log-level
    /// vigilance) — and pre-existing in-place replay inside ingest_event's
    /// StateSpaceCreate arm would happen to reconstruct SpaceState because
    /// stored events get a second pass during from_space_create. The sort
    /// fixes the upstream cause: DAG sees predecessors in causal order, no
    /// error log surfaces. Test asserts the structural outcome (Space + Room
    /// present in runtime state).
    #[test]
    fn replay_spaces_from_dir_topologically_sorts_before_ingest() {
        use ed25519_dalek::SigningKey;
        use tempfile::tempdir;
        use xgen_common::wire::Event;
        use xgen_common::xgid::{RoomXgid, SpaceXgid, Xgid};
        use xgen_core::crypto::encoding;
        use xgen_core::identity::keypair;
        use xgen_core::identity::registry::IdentityRecord;
        use xgen_core::space::state::{
            build_room_create_event, build_space_create_event, sign_event,
        };

        fn pubkey_uri(key: &SigningKey) -> String {
            format!(
                "xgen://pubkey/ed25519:{}",
                encoding::encode(key.verifying_key().as_bytes())
            )
        }

        let alice = keypair::generate();
        let node_key = keypair::generate();
        let mut runtime = NodeRuntime::new(node_key);
        let runtime_node_id_str = runtime.node_id.as_str().to_string();
        runtime
            .register_identity(IdentityRecord {
                identity_id: IdentityXgid::from_xgid(Xgid::new(pubkey_uri(&alice))),
                display_name: None,
                is_ai: false,
                ai_capabilities: None,
                registered_at: "2026-05-23T00:00:00.000Z".to_string(),
                trust_assertion: None,
                devices: vec![],
                home_node: runtime.node_id.clone(),
                update_version: 0,
                revoked: false,
                revoked_at: None,
                revocation_reason: None,
            })
            .expect("test setup: register alice");

        // Construct space_create (DAG root) + room_create (non-root, refs
        // space_create as sole predecessor per D-076 v1.1 amended root set).
        let space_ev = sign_event(
            build_space_create_event(&alice, "replay-test-space", None, 1, &runtime_node_id_str),
            &alice,
        );
        let space_id_str: String = space_ev
            .event_id
            .as_ref()
            .expect("space_ev has event_id")
            .as_str()
            .to_string();
        let space_id_typed = SpaceXgid::from_xgid(Xgid::new(space_id_str.clone()));
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id_str, "general", None),
            &alice,
        );

        // Write events to disk in DAG-VIOLATING order: room_create first,
        // then space_create. Sort-on-replay folds the order back to
        // canonical before each ingest_event call.
        let dir = tempdir().expect("create tempdir");
        let spaces_dir = dir.path();
        let events_in_reverse: Vec<Event> = vec![room_ev.clone(), space_ev.clone()];
        // Mirror production's space_file_name helper (Windows-safe filename
        // derived from the space_id URI — `:` and `/` in xgen:// URIs are
        // invalid Windows filename characters).
        let space_file = spaces_dir.join(super::space_file_name(&space_id_str));
        std::fs::write(
            &space_file,
            serde_json::to_string_pretty(&events_in_reverse).expect("serialise events"),
        )
        .expect("write events to tempdir");

        let count = replay_spaces_from_dir(&mut runtime, spaces_dir);

        assert_eq!(count, 1, "replay_spaces_from_dir should return 1 file replayed");
        assert!(
            runtime.spaces.contains_key(&space_id_typed),
            "runtime must hold the Space after topologically-sorted replay"
        );
        let room_id_str: String = room_ev
            .event_id
            .as_ref()
            .expect("room_ev has event_id")
            .as_str()
            .to_string();
        let room_id_typed = RoomXgid::from_xgid(Xgid::new(room_id_str.clone()));
        assert!(
            runtime.spaces[&space_id_typed].rooms.contains_key(&room_id_typed),
            "runtime must hold the Room (room_create's apply_event ran after space_create created the state); got rooms: {:?}",
            runtime.spaces[&space_id_typed].rooms.keys().collect::<Vec<_>>()
        );
    }

    /// D-067 + D-076 no-drift-surface family regression lock. Verify the
    /// xgen-core `topological_sort` re-export is publicly reachable from
    /// xgen-node — `pub(crate)` would break the runbook §4.6 single-source-
    /// of-truth lock and would silently require a sibling implementation
    /// in xgen-node, reintroducing the drift surface D-076 was promoted to
    /// eliminate. Trivial runtime call provides anti-deadcode signal so
    /// future contributors find this lock by function signature.
    #[test]
    fn topological_sort_publicly_reachable_from_xgen_node() {
        use crate::node::runtime::topological_sort;

        let empty_in: Vec<xgen_common::wire::Event> = vec![];
        let out = topological_sort(empty_in);
        assert!(
            out.is_empty(),
            "topological_sort of empty input must produce empty output"
        );
    }

    // ── Pass 3 Commit 2a per-surface tests T7 + T8 + T11 (runbook §4.7) ──

    // T7 (Surface #5 Q5.12) — persistence-format boundary round-trip:
    // write JSON HashMap with String keys → read via load_space_local_metadata
    // → project String keys to typed SpaceXgid at insertion-into-runtime
    // boundary. Per design doc §4.3 v1.2 consolidated persistence boundary
    // preservation.
    #[test]
    fn app_handlers_persistence_format_round_trip_string_at_boundary() {
        use std::collections::HashMap;
        use tempfile::tempdir;
        use xgen_common::space_local::SpaceLocalMetadata;
        use xgen_common::xgid::{NodeXgid, SpaceXgid, Xgid};

        let dir = tempdir().expect("tempdir");

        // Build typed in-memory state — six per-space HashMap keyed by SpaceXgid.
        let space_id_typed = SpaceXgid::from_xgid(Xgid::new(
            "xgen://hash/sha256:t7-space".to_string(),
        ));
        let introducer = NodeXgid::from_xgid(Xgid::new(
            "xgen://pubkey/ed25519:introducer".to_string(),
        ));
        let metadata = SpaceLocalMetadata::new_via_federation(
            space_id_typed.clone(),
            introducer,
            "2026-05-28T12:00:00.000Z".to_string(),
        );

        // Write side (§4.3 Q5.12 site (b)): project HashMap<SpaceXgid, _> →
        // HashMap<String, _> at the save-call boundary.
        let snapshot: HashMap<String, SpaceLocalMetadata> = {
            let mut m: HashMap<SpaceXgid, SpaceLocalMetadata> = HashMap::new();
            m.insert(space_id_typed.clone(), metadata.clone());
            m.iter()
                .map(|(k, v)| (k.as_str().to_string(), v.clone()))
                .collect()
        };
        save_space_local_metadata(dir.path(), &snapshot);

        // Read side (§4.3 Q5.12 site (a)): load_space_local_metadata returns
        // HashMap<String, _>; project to typed HashMap<SpaceXgid, _> at the
        // in-memory insert boundary.
        let loaded_string_keyed: HashMap<String, SpaceLocalMetadata> =
            load_space_local_metadata(dir.path());
        let loaded_typed: HashMap<SpaceXgid, SpaceLocalMetadata> = loaded_string_keyed
            .into_iter()
            .map(|(k, v)| (SpaceXgid::from_xgid(Xgid::new(k)), v))
            .collect();

        // Round-trip: typed key retrieves the same metadata bytes.
        let retrieved = loaded_typed
            .get(&space_id_typed)
            .expect("typed key retrieves loaded entry");
        assert_eq!(retrieved.space_id, metadata.space_id);
        assert_eq!(retrieved.introducer_node_id, metadata.introducer_node_id);
    }

    // T8 (Surface #5 Q5.2) — verify handle_federation_incoming-shape forced-
    // owned `NodeXgid` parameter compiles + behaves across `tokio::spawn`
    // boundary per design §4.2 v1.2 row 3 async-spawned-captures sub-rule.
    //
    // Compile-time test of the contract: an owned NodeXgid moves into an
    // async closure that satisfies the `'static + Send` bounds tokio::spawn
    // imposes. If the signature drifts back to &str, this won't compile.
    #[tokio::test(flavor = "current_thread")]
    async fn handle_federation_incoming_spawned_task_owns_node_xgid_capture() {
        use xgen_common::xgid::{NodeXgid, Xgid};

        let home: NodeXgid = NodeXgid::from_xgid(Xgid::new(
            "xgen://pubkey/ed25519:t8-home".to_string(),
        ));

        // Move into a spawned task body (the load-bearing pattern at
        // handle_federation_incoming app.rs:1006). The `'static + Send`
        // bound on tokio::spawn forces owned values to cross the boundary.
        let handle = tokio::spawn(async move {
            // Inside the spawn body, home is owned and stays alive for the
            // task's lifetime independent of the caller's stack.
            let inner: &NodeXgid = &home;
            assert_eq!(inner.as_str(), "xgen://pubkey/ed25519:t8-home");
            inner.as_str().to_string()
        });

        let result = handle.await.expect("spawned task joins");
        assert_eq!(result, "xgen://pubkey/ed25519:t8-home");
    }

    // T11 (Surface #5 Q5.14 v1.3 + J-135 addition) — verify the bilateral
    // federation session driver's three identifier-shaped slots retype
    // correctly across the spawn boundary per design Q5.14 v1.3 per-parameter
    // matrix: home_node_id + peer_node_id owned NodeXgid; peer_shared_spaces
    // Vec<SpaceXgid>. Descriptive-string slots (session_id, neg_version,
    // serial) and wire-format-boundary slot (peer_tips: BTreeMap<String,
    // String>) verified NOT-retyped per §4.3 + §5.4 rules.
    //
    // Compile-time + runtime test: an owned NodeXgid + Vec<SpaceXgid> move
    // into an async closure that satisfies tokio::spawn's `'static + Send`
    // bounds; descriptive String slots stay separate.
    #[tokio::test(flavor = "current_thread")]
    async fn run_federation_session_post_handshake_spawned_task_owns_typed_captures() {
        use std::collections::BTreeMap;
        use xgen_common::xgid::{NodeXgid, SpaceXgid, Xgid};

        // Per Q5.14 v1.3 matrix at app.rs:1217-1230 — typed in-memory + String
        // descriptive + BTreeMap wire-format.
        let home_node_id: NodeXgid = NodeXgid::from_xgid(Xgid::new(
            "xgen://pubkey/ed25519:t11-home".to_string(),
        ));
        let peer_node_id: NodeXgid = NodeXgid::from_xgid(Xgid::new(
            "xgen://pubkey/ed25519:t11-peer".to_string(),
        ));
        let peer_shared_spaces: Vec<SpaceXgid> = vec![SpaceXgid::from_xgid(Xgid::new(
            "xgen://hash/sha256:t11-space".to_string(),
        ))];
        // Descriptive strings per §5.4 — stay String.
        let session_id: String = "session-abc".to_string();
        let neg_version: String = "0.1".to_string();
        let serial: String = "json".to_string();
        // Wire-format boundary per §4.3 + Q3.2 — stays BTreeMap<String, String>.
        let peer_tips: BTreeMap<String, String> = BTreeMap::new();

        let handle = tokio::spawn(async move {
            // All four typed slots move owned into the spawn body.
            let h: &NodeXgid = &home_node_id;
            let p: &NodeXgid = &peer_node_id;
            let s: &[SpaceXgid] = &peer_shared_spaces;
            // Descriptive strings + wire-format BTreeMap also move owned.
            let sid: &str = &session_id;
            let nv: &str = &neg_version;
            let ser: &str = &serial;
            let tips: &BTreeMap<String, String> = &peer_tips;
            (
                h.as_str().to_string(),
                p.as_str().to_string(),
                s.len(),
                sid.to_string(),
                nv.to_string(),
                ser.to_string(),
                tips.len(),
            )
        });

        let (h, p, n_spaces, sid, nv, ser, n_tips) =
            handle.await.expect("spawned task joins");
        assert_eq!(h, "xgen://pubkey/ed25519:t11-home");
        assert_eq!(p, "xgen://pubkey/ed25519:t11-peer");
        assert_eq!(n_spaces, 1);
        assert_eq!(sid, "session-abc");
        assert_eq!(nv, "0.1");
        assert_eq!(ser, "json");
        assert_eq!(n_tips, 0);
    }
}
