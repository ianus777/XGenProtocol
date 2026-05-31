// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Node admin write path — the single-source command-implementation layer for
//! `xgen-node` administrator verbs (M6). Symmetric to `xgen-client-lib::ops::*`
//! (D-067): the `--batch` pipe dispatcher and the future Node `--aicontrol`
//! surface (M7) both call into `admin_ops::*`; there are no parallel
//! implementations.
//!
//! Each verb lands in a later phase (M6 §5.1, Phases 3–10) with the shape:
//!
//! ```ignore
//! pub async fn <verb>(
//!     ctx: &mut AdminContext<'_>,
//!     args: <Verb>Args,
//! ) -> Result<<Verb>Result, AdminError>
//! ```
//!
//! where `<Verb>Result` is a pure-data struct (no I/O) and `<Verb>Args` is the
//! clap-parsed input. Dispatchers format the result for their own channel;
//! `admin_ops::*` itself emits no stdout, no logs, no pipe writes.
//!
//! **Phase 2 ships only the scaffolding** — `AdminContext`, `AdminError`, and the
//! supporting `Stage` / `ActorVia` types. No verbs yet.
//!
//! Terminology (D-082): the runtime principal is the **administrator** in prose
//! and **admin** in code/CLI/error-codes/config. Never "operator" (reserved for
//! the AI-operator role, D-059/D-064).

use std::fmt;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::Connection;
use serde::Serialize;
use tokio::sync::Mutex;
use xgen_common::wire::EventType;
use ed25519_dalek::VerifyingKey;
use xgen_common::xgid::{AuthModuleXgid, EventXgid, IdentityXgid, NodeXgid, SpaceXgid, Xgid};
use xgen_core::auth::module_registry::{AuthModuleRecord, AuthModuleRegistry};
use xgen_core::auth::tiers::AuthTier;
use xgen_core::bootstrap::registration_store::BootstrapRegistrationStore;
use xgen_core::crypto::encoding;
use xgen_core::federation::pending_queue::PendingFederationQueue;
use xgen_core::federation::federation_policy::{
    FederationPolicy, FederationPolicyStore, PolicyMode,
};
use xgen_core::federation::registry::{FederationRegistry, FederationRelationship, FederationState};
use xgen_core::identity::registry::{IdentityRecord, RegistryError};
use xgen_core::node::runtime::{DispatchOutcome, EventOrigin, NodeRuntime};
use xgen_core::space::state::{build_membership_event, sign_event};

use crate::audit::{self, AuditEntry, AuditQueryFilter};
use crate::bootstrap_client::{
    deregister_from_bootstrap, register_with_bootstrap, BootstrapClientError,
};
use crate::plugins::PluginInfo;

/// How an admin verb was invoked — the audit `actor_via` dimension (§2.6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorVia {
    /// `xgen-node --batch` pipe dispatch (M6 v1).
    Batch,
    /// Node `--aicontrol` JSONL surface (M7).
    AiControl,
    /// Direct CLI invocation (not via the resident pipe).
    CliDirect,
}

impl ActorVia {
    /// The stable string written to the audit `actor_via` column.
    pub fn as_str(&self) -> &'static str {
        match self {
            ActorVia::Batch => "batch",
            ActorVia::AiControl => "aicontrol",
            ActorVia::CliDirect => "cli-direct",
        }
    }
}

impl fmt::Display for ActorVia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The stage at which an admin verb failed (§2.6.5). Failure semantics are
/// best-effort with honest reporting: partial state is left in place on mid-verb
/// failure, and the error reports the **first** stage at which the verb failed,
/// not every stage it attempted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Input validation failed (malformed args, missing required field).
    Validate,
    /// Privilege/authorisation check failed. M6 v1: always passes (§2.6.2);
    /// reserved for M7+ per-verb gating.
    Authorize,
    /// Registry/store lookup or write failed.
    Register,
    /// Durable persistence to disk failed.
    Persist,
    /// Downstream notification (fan-out, federation push) failed.
    Notify,
    /// Federation peer interaction failed.
    Federate,
}

impl Stage {
    /// The stable string for traces / the structured (`--aicontrol`, M7) error shape.
    pub fn as_str(&self) -> &'static str {
        match self {
            Stage::Validate => "validate",
            Stage::Authorize => "authorize",
            Stage::Register => "register",
            Stage::Persist => "persist",
            Stage::Notify => "notify",
            Stage::Federate => "federate",
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The verb-agnostic catch-all code (§2.7 harmonised bands — `GENERIC_4000` is the
/// single cross-cutting code). Per-category codes (`FED_3xxx`, `AUTH_2xxx`,
/// `IDENT_6xxx`, `BOOT_7xxx`, `SPACE_8xxx`, `AUDIT_5xxx` / `LOG_51xx`,
/// `PLUGIN_9xxx`) are defined per verb in their phase.
pub const GENERIC_ERROR_CODE: &str = "GENERIC_4000";

/// Structured error returned by every `admin_ops::*` verb. Carries the
/// per-category error code (§2.7), the failure stage (§2.6.5), and a human
/// message. The `--batch` dispatcher renders it as `ERROR <CODE>: <message>`
/// (§2.7); the future `--aicontrol` surface (M7) serialises code/stage/message
/// as structured JSON without renaming the codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminError {
    /// Structured error code, e.g. `"FED_3041"` or `GENERIC_4000` (§2.7 bands).
    pub code: String,
    /// First stage at which the verb failed (§2.6.5).
    pub stage: Stage,
    /// Human-readable message.
    pub message: String,
}

impl AdminError {
    /// Construct an error with an explicit per-category code and stage.
    pub fn new(code: impl Into<String>, stage: Stage, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            stage,
            message: message.into(),
        }
    }

    /// Construct a verb-agnostic `GENERIC_4000` error (bad args, internal error).
    pub fn generic(stage: Stage, message: impl Into<String>) -> Self {
        Self::new(GENERIC_ERROR_CODE, stage, message)
    }

    /// The `--batch` plain-text reply line for this error (§2.7), without the
    /// trailing newline: `ERROR <CODE>: <message>`.
    pub fn batch_reply(&self) -> String {
        format!("ERROR {}: {}", self.code, self.message)
    }

    /// The `<CODE>: <message>` body without the leading `ERROR`. The Node pipe
    /// dispatcher feeds this through the existing M2 `ERROR: <body>` wrapper, so
    /// the wire reply is `ERROR: <CODE>: <message>` — the structured code (§2.7's
    /// load-bearing part) within the M2 wrapper. The exact `ERROR <CODE>:`
    /// spelling is refined in M7's structured `--aicontrol` JSON surface.
    pub fn code_message(&self) -> String {
        format!("{}: {}", self.code, self.message)
    }
}

impl fmt::Display for AdminError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display matches the --batch reply shape (§2.7).
        write!(f, "ERROR {}: {}", self.code, self.message)
    }
}

impl std::error::Error for AdminError {}

/// Per-invocation context threaded into every `admin_ops::*` verb, mirroring
/// `xgen-client-lib::ops::OpContext` (D-067). Paths follow the D-035 convention
/// (derived from the data directory). Phase 2 ships the disk-oriented fields;
/// later phases extend this with handles to live runtime state (registries,
/// `NodeRuntime`) as individual verbs require them.
pub struct AdminContext<'a> {
    /// Node data directory — registries, `xgen-node_audit.db`, state file (D-035).
    pub data_dir: &'a Path,
    /// Effective config file path.
    pub config_path: &'a Path,
    /// The administrator principal initiating the verb — the audit `actor`
    /// (§2.6.4). v1: OS-user-equals-administrator (§2.6.1); the pipe is
    /// OS-access-gated and unauthenticated, so this is the Node's own identity
    /// URI in v1. M7 may carry a distinct admin principal here.
    pub actor: String,
    /// How the verb was invoked — the audit `actor_via` (§2.6.4).
    pub actor_via: ActorVia,
    /// Handle to the live `NodeRuntime` of the resident this verb runs inside.
    ///
    /// **P5 decision (M6 Phase 5, J-155).** A5's mutating verbs (`identity
    /// revoke` / `set-trust-expiry` / `manage-replica`) must reach *live* Node
    /// state — A5-D1 commits revoke to "immediate, security-critical" (a
    /// disk-only write would leave the resident's in-memory registry stale, a
    /// security window, not cosmetic lag), and `ReplicaRegistry` is in-memory
    /// only (no disk backing). So this category, like A6-D1's `log set-level`
    /// reload handle, reaches into the resident. This widens `AdminContext`
    /// from the audit/log verbs' file-only shape (`data_dir`) to runtime-aware.
    /// `None` for the file-only verbs and for unit tests that don't need it;
    /// M7's `--aicontrol` dispatcher provides it the same way the pipe does.
    pub runtime: Option<Arc<Mutex<NodeRuntime>>>,
    /// Handle to the live `FederationRegistry` of the resident (P5 precedent,
    /// extended at A1/Phase 7). A1's mutating verbs (`federation defederate`)
    /// must reach the *live* registry the federation session paths consult, so
    /// a defederation takes effect at once and is persisted to
    /// `xgen-node_federation.json`. `None` for verbs/tests that don't need it.
    pub federation_registry: Option<Arc<Mutex<FederationRegistry>>>,
    /// Live client outbound senders of the resident (Option B live fan-out,
    /// J-160). When set, a Node-authored Space-DAG event (`force-eject` /
    /// `unban`) is fanned out to the Space's connected member clients
    /// immediately after persist — on top of the Option-A sync path. `None` →
    /// sync-only (the Option-A baseline; file-only verbs and unit tests).
    pub client_senders: Option<crate::fanout::ClientSenders>,
    /// Live federation peer senders of the resident (Option B live fan-out,
    /// J-160). When set, the Node-authored event is pushed to the Space's
    /// federated peers immediately after persist (`LocallySubmitted` → eligible
    /// per F-5), the same path a client-submitted event takes. `None` →
    /// sync-only.
    pub federation_peer_senders: Option<crate::fanout::FederationPeerSenders>,
    /// federation-admin-control 2a — the live pending-request approval queue
    /// (FAC-D1a). Required by `federation accept` / `reject`; it must be the
    /// *same* `Arc` the inbound gate (`handle_federation_incoming`) enqueues
    /// into, so an `accept`/`reject` sees live requests and the gate sees the
    /// removal. `None` for verbs/tests that don't need it.
    pub federation_queue: Option<Arc<Mutex<PendingFederationQueue>>>,
    /// federation-admin-control 2b — the live per-peer policy store (FAC-D3/D4).
    /// Required by `federation set-policy` / `show-policy`; it must be the *same*
    /// `Arc` the enforcement sites (`apply_federation_push` outbound,
    /// `process_inbound` inbound) consult, so a `set-policy` takes effect at
    /// once. `None` for verbs/tests that don't need it. (`federation initiate`
    /// prefers this live store when present, falling back to a disk load.)
    pub federation_policy: Option<Arc<Mutex<FederationPolicyStore>>>,
    /// auth-module-registry (A2) — the live registry of trusted Auth Modules
    /// (AMR-D1). Required by the `auth-module list`/`register`/`revoke`/
    /// `set-tiers` verbs; it is the same `Arc` the pipe server holds across
    /// connections, so a `register`/`revoke` is reflected by a later `list` in
    /// the same resident. `None` for verbs/tests that don't need it. (No runtime
    /// consumer reads it this arc — AMR-D1 standalone; the registration / 3006
    /// consultation is a deferred future arc.)
    pub auth_module_registry: Option<Arc<Mutex<AuthModuleRegistry>>>,
    /// bootstrap-client (A3) — the live local bootstrap store (BC-D1: the
    /// registrations this Node holds + the self-info it advertises). Required by
    /// the `bootstrap show`/`register`/`deregister`/`set-info`/`set-tiers` verbs;
    /// it is the same `Arc` the pipe server holds across connections, so a
    /// `register` is reflected by a later `show` in the same resident. `None` for
    /// verbs/tests that don't need it. (No automatic startup consumer this arc —
    /// the keepalive scheduler + seed auto-register are the C4 / later concern.)
    pub bootstrap_store: Option<Arc<Mutex<BootstrapRegistrationStore>>>,
}

impl<'a> AdminContext<'a> {
    /// Build a `--batch`-originated admin context with no live-runtime handle
    /// (file-only verbs: the A6 `audit *` / `log *` surface, and unit tests).
    pub fn batch(data_dir: &'a Path, config_path: &'a Path, actor: impl Into<String>) -> Self {
        Self {
            data_dir,
            config_path,
            actor: actor.into(),
            actor_via: ActorVia::Batch,
            runtime: None,
            federation_registry: None,
            client_senders: None,
            federation_peer_senders: None,
            federation_queue: None,
            federation_policy: None,
            auth_module_registry: None,
            bootstrap_store: None,
        }
    }

    /// Build a `--batch`-originated admin context carrying the live `NodeRuntime`
    /// handle — required by the A5 (and later live-mutating) verbs. See the
    /// `runtime` field's P5 note. (Equivalent to `batch(..).with_runtime(rt)`.)
    pub fn batch_with_runtime(
        data_dir: &'a Path,
        config_path: &'a Path,
        actor: impl Into<String>,
        runtime: Arc<Mutex<NodeRuntime>>,
    ) -> Self {
        Self::batch(data_dir, config_path, actor).with_runtime(runtime)
    }

    /// Builder: attach the live `NodeRuntime` handle (A5 / A4 categories).
    pub fn with_runtime(mut self, runtime: Arc<Mutex<NodeRuntime>>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Builder: attach the live `FederationRegistry` handle (A1 category).
    pub fn with_federation_registry(
        mut self,
        federation_registry: Arc<Mutex<FederationRegistry>>,
    ) -> Self {
        self.federation_registry = Some(federation_registry);
        self
    }

    /// Builder: attach the live client outbound senders (Option B live fan-out).
    pub fn with_client_senders(mut self, client_senders: crate::fanout::ClientSenders) -> Self {
        self.client_senders = Some(client_senders);
        self
    }

    /// Builder: attach the live federation peer senders (Option B live fan-out).
    pub fn with_federation_senders(
        mut self,
        federation_peer_senders: crate::fanout::FederationPeerSenders,
    ) -> Self {
        self.federation_peer_senders = Some(federation_peer_senders);
        self
    }

    /// Builder: attach the live pending-federation queue (A1 2a accept/reject).
    pub fn with_federation_queue(
        mut self,
        federation_queue: Arc<Mutex<PendingFederationQueue>>,
    ) -> Self {
        self.federation_queue = Some(federation_queue);
        self
    }

    /// Builder: attach the live federation policy store (A1 2b set/show-policy).
    pub fn with_federation_policy(
        mut self,
        federation_policy: Arc<Mutex<FederationPolicyStore>>,
    ) -> Self {
        self.federation_policy = Some(federation_policy);
        self
    }

    /// Builder: attach the live Auth Module registry (A2 auth-module verbs).
    pub fn with_auth_module_registry(
        mut self,
        auth_module_registry: Arc<Mutex<AuthModuleRegistry>>,
    ) -> Self {
        self.auth_module_registry = Some(auth_module_registry);
        self
    }

    /// Builder: attach the live bootstrap store (A3 bootstrap verbs).
    pub fn with_bootstrap_store(
        mut self,
        bootstrap_store: Arc<Mutex<BootstrapRegistrationStore>>,
    ) -> Self {
        self.bootstrap_store = Some(bootstrap_store);
        self
    }

    /// Canonical on-disk identity registry path (D-035 convention). Despite the
    /// `.db` suffix the file is JSON (`IdentityRegistry::{save,load}`); the name
    /// matches `app.rs`'s load/save site.
    pub fn identities_path(&self) -> PathBuf {
        self.data_dir.join("xgen-node_identities.db")
    }

    /// Canonical on-disk federation registry path (D-035; matches `app.rs`).
    pub fn federation_registry_path(&self) -> PathBuf {
        self.data_dir.join("xgen-node_federation.json")
    }

    /// Canonical on-disk pending-federation-queue path (D-035; matches `app.rs`).
    pub fn federation_queue_path(&self) -> PathBuf {
        self.data_dir.join("xgen-node_federation_queue.json")
    }

    /// Canonical on-disk federation-policy path (D-035; matches `app.rs`).
    pub fn federation_policy_path(&self) -> PathBuf {
        self.data_dir.join("xgen-node_federation_policy.json")
    }

    /// Canonical on-disk Auth Module registry path (D-035; matches `app.rs`).
    pub fn auth_module_registry_path(&self) -> PathBuf {
        self.data_dir.join("xgen-node_auth_modules.json")
    }

    /// Canonical on-disk bootstrap store path (D-035; matches `app.rs`).
    /// ONE combined file — registrations map + self-info (BC-D1(b)).
    pub fn bootstrap_store_path(&self) -> PathBuf {
        self.data_dir.join("xgen-node_bootstrap.json")
    }

    /// Borrow the live-runtime handle or fail with a clear `GENERIC_4000` — the
    /// A5 verbs are only reachable through the in-resident pipe dispatcher, so
    /// `None` here is a wiring bug, not a user error.
    fn require_runtime(&self, stage: Stage) -> Result<&Arc<Mutex<NodeRuntime>>, AdminError> {
        self.runtime.as_ref().ok_or_else(|| {
            AdminError::generic(stage, "no live Node runtime available for this verb")
        })
    }

    /// Borrow the live `FederationRegistry` handle or fail `GENERIC_4000` (A1).
    fn require_federation_registry(
        &self,
        stage: Stage,
    ) -> Result<&Arc<Mutex<FederationRegistry>>, AdminError> {
        self.federation_registry.as_ref().ok_or_else(|| {
            AdminError::generic(stage, "no live federation registry available for this verb")
        })
    }

    /// Borrow the live pending-federation queue or fail `GENERIC_4000` (A1 2a
    /// accept/reject). `None` is a wiring bug, not a user error.
    fn require_federation_queue(
        &self,
        stage: Stage,
    ) -> Result<&Arc<Mutex<PendingFederationQueue>>, AdminError> {
        self.federation_queue.as_ref().ok_or_else(|| {
            AdminError::generic(stage, "no live pending-federation queue available for this verb")
        })
    }

    /// Borrow the live federation policy store or fail `GENERIC_4000` (A1 2b
    /// set/show-policy). `None` is a wiring bug, not a user error.
    fn require_federation_policy(
        &self,
        stage: Stage,
    ) -> Result<&Arc<Mutex<FederationPolicyStore>>, AdminError> {
        self.federation_policy.as_ref().ok_or_else(|| {
            AdminError::generic(stage, "no live federation policy store available for this verb")
        })
    }

    /// Borrow the live Auth Module registry or fail `GENERIC_4000` (A2). `None`
    /// is a wiring bug, not a user error.
    fn require_auth_module_registry(
        &self,
        stage: Stage,
    ) -> Result<&Arc<Mutex<AuthModuleRegistry>>, AdminError> {
        self.auth_module_registry.as_ref().ok_or_else(|| {
            AdminError::generic(stage, "no live Auth Module registry available for this verb")
        })
    }

    /// Borrow the live bootstrap store or fail `GENERIC_4000` (A3). `None` is a
    /// wiring bug, not a user error.
    fn require_bootstrap_store(
        &self,
        stage: Stage,
    ) -> Result<&Arc<Mutex<BootstrapRegistrationStore>>, AdminError> {
        self.bootstrap_store.as_ref().ok_or_else(|| {
            AdminError::generic(stage, "no live bootstrap store available for this verb")
        })
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// A6 — Logging & audit administration (M6 Phase 4; design §6.A6, Appendix K.2.1)
// ════════════════════════════════════════════════════════════════════════════════

const AUDIT_QUERY_DEFAULT_LIMIT: usize = 100;
const AUDIT_QUERY_MAX_LIMIT: usize = 1000;

/// Open the admin audit DB, mapping failure to a structured error.
fn open_audit(ctx: &AdminContext<'_>) -> Result<Connection, AdminError> {
    audit::open_audit_db(ctx.data_dir).map_err(|e| {
        AdminError::new("AUDIT_5010", Stage::Register, format!("opening audit db: {e}"))
    })
}

/// Validate an optional RFC 3339 timestamp filter (`AUDIT_5010` on malformed).
fn validate_ts(label: &str, v: &Option<String>) -> Result<(), AdminError> {
    if let Some(s) = v {
        if chrono::DateTime::parse_from_rfc3339(s).is_err() {
            return Err(AdminError::new(
                "AUDIT_5010",
                Stage::Validate,
                format!("malformed {label} timestamp (expected RFC 3339): {s}"),
            ));
        }
    }
    Ok(())
}

/// Validate the shared `audit query` / `audit export` filter args and build the
/// `AuditQueryFilter`. `limit` defaults to 100, hard-capped at 1000.
#[allow(clippy::too_many_arguments)]
fn build_filter(
    actor: Option<String>,
    verb: Option<String>,
    since: Option<String>,
    until: Option<String>,
    outcome: Option<String>,
    limit: usize,
) -> Result<AuditQueryFilter, AdminError> {
    validate_ts("since", &since)?;
    validate_ts("until", &until)?;
    if let Some(o) = &outcome {
        if o != "ok" && o != "error" {
            return Err(AdminError::new(
                "AUDIT_5010",
                Stage::Validate,
                format!("unknown outcome '{o}' (expected ok|error)"),
            ));
        }
    }
    Ok(AuditQueryFilter {
        actor,
        verb,
        since,
        until,
        outcome,
        limit,
    })
}

/// Audit-the-auditor (A6-D4): write an audit entry for a WRITE/DESTRUCTIVE verb
/// (or the data-extracting `audit export`). READ verbs do NOT call this.
#[allow(clippy::too_many_arguments)]
fn record_action(
    conn: &Connection,
    ctx: &AdminContext<'_>,
    verb: &str,
    target: Option<String>,
    args_hash: String,
    outcome: &str,
    error_code: Option<String>,
    error_message: Option<String>,
) -> Result<(), AdminError> {
    let entry = AuditEntry {
        timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        verb: verb.to_string(),
        actor: ctx.actor.clone(),
        actor_via: ctx.actor_via.as_str().to_string(),
        target,
        args_hash,
        outcome: outcome.to_string(),
        error_code,
        error_message,
        correlation_id: None,
        meta_atts: "{}".to_string(),
    };
    audit::insert_entry(conn, &entry).map_err(|e| {
        AdminError::new("AUDIT_5001", Stage::Persist, format!("audit-the-auditor write failed: {e}"))
    })
}

// ── audit query — READ (not audited) ────────────────────────────────────────────

/// Args for `audit query` (§6.A6).
#[derive(Debug, Clone, Default, clap::Args)]
pub struct AuditQueryArgs {
    #[arg(long)]
    pub actor: Option<String>,
    #[arg(long)]
    pub verb: Option<String>,
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long)]
    pub until: Option<String>,
    #[arg(long)]
    pub outcome: Option<String>,
    #[arg(long)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditQueryResult {
    pub entries: Vec<AuditEntry>,
    pub total_matched: usize,
    pub returned: usize,
}

/// `audit query` — filtered, newest-first read of the SQLite admin trail.
pub async fn audit_query(
    ctx: &mut AdminContext<'_>,
    args: AuditQueryArgs,
) -> Result<AuditQueryResult, AdminError> {
    let limit = args
        .limit
        .unwrap_or(AUDIT_QUERY_DEFAULT_LIMIT)
        .min(AUDIT_QUERY_MAX_LIMIT);
    let filter = build_filter(args.actor, args.verb, args.since, args.until, args.outcome, limit)?;
    let conn = open_audit(ctx)?;
    let (entries, total_matched) = audit::query(&conn, &filter).map_err(|e| {
        AdminError::new("AUDIT_5010", Stage::Register, format!("audit query failed: {e}"))
    })?;
    let returned = entries.len();
    Ok(AuditQueryResult {
        entries,
        total_matched,
        returned,
    })
}

// ── audit export — READ but data-extracting (audited) ────────────────────────────

/// Args for `audit export` (§6.A6) — the `audit query` filter set + an output
/// file. `format` defaults to `jsonl` (`csv` reserved).
#[derive(Debug, Clone, clap::Args)]
pub struct AuditExportArgs {
    #[arg(long)]
    pub actor: Option<String>,
    #[arg(long)]
    pub verb: Option<String>,
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long)]
    pub until: Option<String>,
    #[arg(long)]
    pub outcome: Option<String>,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditExportResult {
    pub exported_count: usize,
    pub output_path: String,
    pub format: String,
}

/// `audit export` — materialise a filtered JSONL slice for SIEM. Exports ALL
/// matching rows (the `limit` cap is a `query`-only convenience). Audited (A6-D4).
pub async fn audit_export(
    ctx: &mut AdminContext<'_>,
    args: AuditExportArgs,
) -> Result<AuditExportResult, AdminError> {
    let format = args.format.clone().unwrap_or_else(|| "jsonl".to_string());
    if format != "jsonl" {
        return Err(AdminError::generic(
            Stage::Validate,
            format!("unsupported export format '{format}' (only 'jsonl' in M6; 'csv' reserved)"),
        ));
    }
    // Export every matching row, not the query default of 100. i64::MAX (not
    // usize::MAX) because SQLite's LIMIT is a signed 64-bit value — usize::MAX
    // overflows it ("datatype mismatch"). i64::MAX is effectively unbounded.
    let filter = build_filter(
        args.actor.clone(),
        args.verb.clone(),
        args.since.clone(),
        args.until.clone(),
        args.outcome.clone(),
        i64::MAX as usize,
    )?;
    let conn = open_audit(ctx)?;
    let (entries, _total) = audit::query(&conn, &filter).map_err(|e| {
        AdminError::new("AUDIT_5010", Stage::Register, format!("audit export read failed: {e}"))
    })?;
    audit::write_jsonl(&entries, &args.output).map_err(|e| {
        AdminError::new("AUDIT_5020", Stage::Persist, format!("audit export write failed: {e}"))
    })?;
    let output_path = args.output.display().to_string();
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"actor\":{:?},\"verb\":{:?},\"since\":{:?},\"until\":{:?},\"outcome\":{:?},\"output\":{:?},\"format\":{:?}}}",
        args.actor, args.verb, args.since, args.until, args.outcome, output_path, format
    ));
    record_action(
        &conn,
        ctx,
        "audit export",
        Some(output_path.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;
    Ok(AuditExportResult {
        exported_count: entries.len(),
        output_path,
        format,
    })
}

// ── audit archive — DESTRUCTIVE (audited) ────────────────────────────────────────

/// Args for `audit archive` (§6.A6, A6-D2) — export rows older than `before`,
/// then prune them. `output` defaults to a dated file under `<data_dir>/audit/`.
#[derive(Debug, Clone, clap::Args)]
pub struct AuditArchiveArgs {
    #[arg(long)]
    pub before: String,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditArchiveResult {
    pub archived_count: usize,
    pub archive_path: String,
    pub oldest_ts: Option<String>,
    pub newest_ts: Option<String>,
}

/// `audit archive` — export rows with `timestamp < before` to a dated JSONL file,
/// then prune them from the live table. Fail-safe toward retention (§2.6.4):
/// if the archive write fails the rows are NOT pruned (`AUDIT_5001`); if the
/// prune fails after a successful write the archive is kept and the rows remain
/// (`AUDIT_5002`). Audited (A6-D4).
pub async fn audit_archive(
    ctx: &mut AdminContext<'_>,
    args: AuditArchiveArgs,
) -> Result<AuditArchiveResult, AdminError> {
    if chrono::DateTime::parse_from_rfc3339(&args.before).is_err() {
        return Err(AdminError::new(
            "AUDIT_5010",
            Stage::Validate,
            format!("malformed `before` timestamp (expected RFC 3339): {}", args.before),
        ));
    }
    let conn = open_audit(ctx)?;
    let entries = audit::entries_before(&conn, &args.before).map_err(|e| {
        AdminError::new("AUDIT_5010", Stage::Register, format!("audit archive read failed: {e}"))
    })?;

    // Default archive path: <data_dir>/audit/xgen-node_audit_archive_<ts>.jsonl.
    // Filename-safe timestamp (no ':' — invalid on Windows).
    let archive_path: PathBuf = args.output.clone().unwrap_or_else(|| {
        let stamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        ctx.data_dir
            .join("audit")
            .join(format!("xgen-node_audit_archive_{stamp}.jsonl"))
    });

    // Write the archive first. On failure, do NOT prune (fail-safe retention).
    audit::write_jsonl(&entries, &archive_path).map_err(|e| {
        AdminError::new("AUDIT_5001", Stage::Persist, format!("audit archive write failed: {e}"))
    })?;
    // Prune. On failure, the archive is kept and the rows remain (retention).
    audit::delete_before(&conn, &args.before).map_err(|e| {
        AdminError::new(
            "AUDIT_5002",
            Stage::Persist,
            format!("audit archive prune failed after a successful write (rows retained): {e}"),
        )
    })?;

    let archive_path_str = archive_path.display().to_string();
    let result = AuditArchiveResult {
        archived_count: entries.len(),
        archive_path: archive_path_str.clone(),
        oldest_ts: entries.first().map(|e| e.timestamp.clone()),
        newest_ts: entries.last().map(|e| e.timestamp.clone()),
    };
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"before\":{:?},\"output\":{:?}}}",
        args.before, archive_path_str
    ));
    record_action(
        &conn,
        ctx,
        "audit archive",
        Some(archive_path_str),
        args_hash,
        "ok",
        None,
        None,
    )?;
    Ok(result)
}

// ── log show-level — READ (not audited) ─────────────────────────────────────────

/// Args for `log show-level` (§6.A6).
#[derive(Debug, Clone, Default, clap::Args)]
pub struct LogShowLevelArgs {
    /// Filter to one module path; default = all effective levels.
    #[arg(long)]
    pub module: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogLevelEntry {
    pub module: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogShowLevelResult {
    pub levels: Vec<LogLevelEntry>,
}

/// `log show-level` — report effective runtime tracing levels (`*` = global
/// default, then per-module overrides). READ → not audited.
pub async fn log_show_level(
    _ctx: &mut AdminContext<'_>,
    args: LogShowLevelArgs,
) -> Result<LogShowLevelResult, AdminError> {
    let levels = crate::app::log_levels(args.module.as_deref())
        .into_iter()
        .map(|(module, level)| LogLevelEntry { module, level })
        .collect();
    Ok(LogShowLevelResult { levels })
}

// ── log set-level — WRITE (audited) ──────────────────────────────────────────────

/// Args for `log set-level` (§6.A6, A6-D1).
#[derive(Debug, Clone, clap::Args)]
pub struct LogSetLevelArgs {
    /// Target module path (e.g. `xgen_node::federation`); default `*` = global.
    #[arg(long)]
    pub module: Option<String>,
    /// New level: error|warn|info|debug|trace|off.
    #[arg(long)]
    pub level: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogSetLevelResult {
    pub module: String,
    pub previous_level: String,
    pub new_level: String,
    pub applied: bool,
}

/// `log set-level` — apply a tracing level at runtime via the reload handle
/// (A6-D1: runtime-only, NOT persisted to config; survives until restart).
/// WRITE → audited (A6-D4): the admin action is recorded even though the level
/// change itself is not persisted.
pub async fn log_set_level(
    ctx: &mut AdminContext<'_>,
    args: LogSetLevelArgs,
) -> Result<LogSetLevelResult, AdminError> {
    use crate::app::LogSetError;
    match crate::app::apply_log_set_level(args.module.as_deref(), &args.level) {
        Ok((previous_level, applied)) => {
            let module = args.module.clone().unwrap_or_else(|| "*".to_string());
            // A6-D4: WRITE → audit the action.
            let conn = open_audit(ctx)?;
            let args_hash = AuditEntry::compute_args_hash(&format!(
                "{{\"module\":{:?},\"level\":{:?}}}",
                args.module, args.level
            ));
            record_action(
                &conn,
                ctx,
                "log set-level",
                Some(module.clone()),
                args_hash,
                "ok",
                None,
                None,
            )?;
            Ok(LogSetLevelResult {
                module,
                previous_level,
                new_level: args.level,
                applied,
            })
        }
        Err(LogSetError::InvalidLevel) => Err(AdminError::new(
            "LOG_5101",
            Stage::Validate,
            format!("invalid level '{}' (expected error|warn|info|debug|trace|off)", args.level),
        )),
        Err(LogSetError::UnsettableModule) => Err(AdminError::new(
            "LOG_5102",
            Stage::Register,
            format!(
                "unsettable module/directive for '{}'",
                args.module.clone().unwrap_or_else(|| "*".to_string())
            ),
        )),
        Err(LogSetError::NoHandle) => Err(AdminError::new(
            "LOG_5102",
            Stage::Register,
            "logging is not under runtime control on this Node (no reload handle)".to_string(),
        )),
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// A5 — Identity registry administration (M6 Phase 5; design §6.A5, Appendix K.2.2)
// ════════════════════════════════════════════════════════════════════════════════
// All A5 verbs are Node-local (D-082); none emits a protocol event (propagation =
// none; cascade deferred per A5-D1). The mutating verbs reach the *live*
// NodeRuntime via AdminContext::runtime (P5 decision — see that field's note):
// revoke must be immediate (A5-D1) and ReplicaRegistry is in-memory only.

/// Project a wire-format Identity URI to the typed key at the registry boundary.
fn ident_xgid(s: &str) -> IdentityXgid {
    IdentityXgid::from_xgid(Xgid::new(s.to_string()))
}

// ── identity show — READ (not audited; A5-D3) ────────────────────────────────────

/// Args for `identity show` (§6.A5).
#[derive(Debug, Clone, clap::Args)]
pub struct IdentityShowArgs {
    /// Identity URI (`xgen://pubkey/ed25519:...`).
    pub identity_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityShowResult {
    pub record: IdentityRecord,
}

/// `identity show` — display one stored Identity record. Reads the live
/// registry. Not audited (pure read).
pub async fn identity_show(
    ctx: &mut AdminContext<'_>,
    args: IdentityShowArgs,
) -> Result<IdentityShowResult, AdminError> {
    let runtime = Arc::clone(ctx.require_runtime(Stage::Register)?);
    let id = ident_xgid(&args.identity_id);
    let rt = runtime.lock().await;
    match rt.identity_registry.get(&id) {
        Some(rec) => Ok(IdentityShowResult { record: rec.clone() }),
        None => Err(AdminError::new(
            "IDENT_6001",
            Stage::Register,
            format!("identity not found: {}", args.identity_id),
        )),
    }
}

// ── identity revoke — DESTRUCTIVE (audited; A5-D1 block-only) ─────────────────────

/// Args for `identity revoke` (§6.A5).
#[derive(Debug, Clone, clap::Args)]
pub struct IdentityRevokeArgs {
    /// Identity URI to revoke.
    pub identity_id: String,
    /// Optional operator-supplied reason (recorded on the record + audited).
    #[arg(long)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityRevokeResult {
    pub identity_id: String,
    pub revoked_at: String,
    /// Spaces the (now-inert) Identity is still a member of (A5-D1 honest report;
    /// memberships are left in place — no cascade in M6).
    pub stale_membership_spaces: Vec<String>,
}

/// `identity revoke` — mark an Identity revoked (block-only, A5-D1). Mutates the
/// live registry (so the auth gate denies the next session-open immediately),
/// persists it to disk, and reports the Spaces left inert. DESTRUCTIVE → audited.
pub async fn identity_revoke(
    ctx: &mut AdminContext<'_>,
    args: IdentityRevokeArgs,
) -> Result<IdentityRevokeResult, AdminError> {
    let id = ident_xgid(&args.identity_id);
    let revoked_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let runtime = Arc::clone(ctx.require_runtime(Stage::Register)?);
    let identities_path = ctx.identities_path();

    let stale_membership_spaces: Vec<String> = {
        let mut rt = runtime.lock().await;
        match rt
            .identity_registry
            .revoke(&id, revoked_at.clone(), args.reason.clone())
        {
            Ok(()) => {}
            Err(RegistryError::NotFound) => {
                return Err(AdminError::new(
                    "IDENT_6001",
                    Stage::Register,
                    format!("identity not found: {}", args.identity_id),
                ));
            }
            Err(RegistryError::AlreadyRevoked) => {
                return Err(AdminError::new(
                    "IDENT_6002",
                    Stage::Register,
                    format!("identity already revoked: {}", args.identity_id),
                ));
            }
            Err(e) => {
                return Err(AdminError::generic(
                    Stage::Register,
                    format!("revoke failed: {e}"),
                ));
            }
        }
        // Persist so the revocation survives restart (memory + disk agree).
        if let Err(e) = rt.identity_registry.save(&identities_path) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("identity registry save failed: {e}"),
            ));
        }
        rt.spaces
            .iter()
            .filter(|(_, s)| s.is_member(args.identity_id.as_str()))
            .map(|(sid, _)| sid.as_str().to_string())
            .collect()
    };

    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"identity_id\":{:?},\"reason\":{:?}}}",
        args.identity_id, args.reason
    ));
    record_action(
        &conn,
        ctx,
        "identity revoke",
        Some(args.identity_id.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;
    Ok(IdentityRevokeResult {
        identity_id: args.identity_id,
        revoked_at,
        stale_membership_spaces,
    })
}

// ── identity set-trust-expiry — WRITE (audited) ──────────────────────────────────

/// Args for `identity set-trust-expiry` (§6.A5).
#[derive(Debug, Clone, clap::Args)]
pub struct IdentitySetTrustExpiryArgs {
    /// Identity URI.
    pub identity_id: String,
    /// New Trust Assertion expiry (RFC 3339).
    #[arg(long)]
    pub expiry: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentitySetTrustExpiryResult {
    pub identity_id: String,
    pub previous_expiry: Option<String>,
    pub new_expiry: String,
}

/// `identity set-trust-expiry` — set/replace the `expiry` inside an Identity's
/// Trust Assertion. WRITE → audited.
pub async fn identity_set_trust_expiry(
    ctx: &mut AdminContext<'_>,
    args: IdentitySetTrustExpiryArgs,
) -> Result<IdentitySetTrustExpiryResult, AdminError> {
    if DateTime::parse_from_rfc3339(&args.expiry).is_err() {
        return Err(AdminError::new(
            "IDENT_6010",
            Stage::Validate,
            format!("malformed expiry (RFC 3339 required): {}", args.expiry),
        ));
    }
    let id = ident_xgid(&args.identity_id);
    let runtime = Arc::clone(ctx.require_runtime(Stage::Register)?);
    let identities_path = ctx.identities_path();

    let previous_expiry = {
        let mut rt = runtime.lock().await;
        match rt.identity_registry.set_trust_expiry(&id, args.expiry.clone()) {
            Ok(prev) => {
                if let Err(e) = rt.identity_registry.save(&identities_path) {
                    return Err(AdminError::generic(
                        Stage::Persist,
                        format!("identity registry save failed: {e}"),
                    ));
                }
                prev
            }
            Err(RegistryError::NotFound) => {
                return Err(AdminError::new(
                    "IDENT_6001",
                    Stage::Register,
                    format!("identity not found: {}", args.identity_id),
                ));
            }
            Err(e) => {
                return Err(AdminError::generic(
                    Stage::Register,
                    format!("set-trust-expiry failed: {e}"),
                ));
            }
        }
    };

    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"identity_id\":{:?},\"expiry\":{:?}}}",
        args.identity_id, args.expiry
    ));
    record_action(
        &conn,
        ctx,
        "identity set-trust-expiry",
        Some(args.identity_id.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;
    Ok(IdentitySetTrustExpiryResult {
        identity_id: args.identity_id,
        previous_expiry,
        new_expiry: args.expiry,
    })
}

// ── identity manage-replica — WRITE (add/remove audited; list not) — A5-D2 ───────

/// Replica-management action (`identity manage-replica --action ...`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ReplicaAction {
    Add,
    Remove,
    List,
}

/// Args for `identity manage-replica` (§6.A5, A5-D2 thin-scope).
#[derive(Debug, Clone, clap::Args)]
pub struct IdentityManageReplicaArgs {
    /// Identity URI.
    pub identity_id: String,
    /// `add` | `remove` | `list`.
    #[arg(long, value_enum)]
    pub action: ReplicaAction,
    /// Replica Node URI (required for `add` / `remove`).
    #[arg(long)]
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityManageReplicaResult {
    pub identity_id: String,
    /// Post-action replica-Node list.
    pub replicas: Vec<String>,
}

/// `identity manage-replica` — declare/list which Nodes hold replicas of an
/// Identity record (registry-only, A5-D2: no active replication push). Operates
/// on the live in-memory `ReplicaRegistry` (not persisted — rebuilt on restart).
/// `add`/`remove` are WRITE → audited; `list` is a read → not audited.
pub async fn identity_manage_replica(
    ctx: &mut AdminContext<'_>,
    args: IdentityManageReplicaArgs,
) -> Result<IdentityManageReplicaResult, AdminError> {
    let id = ident_xgid(&args.identity_id);
    let runtime = Arc::clone(ctx.require_runtime(Stage::Register)?);

    let (replicas, audited): (Vec<String>, bool) = {
        let mut rt = runtime.lock().await;
        if !rt.identity_registry.contains(&id) {
            return Err(AdminError::new(
                "IDENT_6001",
                Stage::Register,
                format!("identity not found: {}", args.identity_id),
            ));
        }
        let iid = args.identity_id.as_str();
        match args.action {
            ReplicaAction::List => (rt.replica_registry.get_replicas(iid).to_vec(), false),
            ReplicaAction::Add => {
                let node = require_node_id(&args.node_id)?;
                if rt.replica_registry.has_replica(iid, &node) {
                    return Err(AdminError::new(
                        "IDENT_6021",
                        Stage::Register,
                        format!("replica already present: {node}"),
                    ));
                }
                rt.replica_registry.add_replica(iid, &node);
                (rt.replica_registry.get_replicas(iid).to_vec(), true)
            }
            ReplicaAction::Remove => {
                let node = require_node_id(&args.node_id)?;
                if !rt.replica_registry.has_replica(iid, &node) {
                    return Err(AdminError::new(
                        "IDENT_6021",
                        Stage::Register,
                        format!("replica not present: {node}"),
                    ));
                }
                rt.replica_registry.remove_replica(iid, &node);
                (rt.replica_registry.get_replicas(iid).to_vec(), true)
            }
        }
    };

    if audited {
        let conn = open_audit(ctx)?;
        let args_hash = AuditEntry::compute_args_hash(&format!(
            "{{\"identity_id\":{:?},\"action\":{:?},\"node_id\":{:?}}}",
            args.identity_id, args.action, args.node_id
        ));
        record_action(
            &conn,
            ctx,
            "identity manage-replica",
            Some(args.identity_id.clone()),
            args_hash,
            "ok",
            None,
            None,
        )?;
    }
    Ok(IdentityManageReplicaResult {
        identity_id: args.identity_id,
        replicas,
    })
}

/// `--node-id` is mandatory and non-empty for `add` / `remove` (IDENT_6020).
fn require_node_id(node_id: &Option<String>) -> Result<String, AdminError> {
    match node_id {
        Some(n) if !n.trim().is_empty() => Ok(n.clone()),
        _ => Err(AdminError::new(
            "IDENT_6020",
            Stage::Validate,
            "--node-id is required (and non-empty) for add/remove".to_string(),
        )),
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// A1 — Federation management (M6 Phase 7; design §6.A1, Appendix K.2.4)
// ════════════════════════════════════════════════════════════════════════════════
// HONEST SUBSET (Joe-locked, J-156). Block 4 specified 7 A1 verbs, but the
// recon found only `list` + `defederate` have real backing in the post-
// federation-milestone `FederationRegistry`: there is NO admin-approval
// pending-request queue (federation auto-establishes on handshake) and NO
// per-peer policy store / enforcement. So `accept` / `reject` / `set-policy` /
// `show-policy` (and the heavy admin-gated `initiate` handshake) presuppose
// subsystems that don't exist — they defer to a post-M6 federation-admin-control
// arc under D-071, not a verb phase (same "no half-feature on an immature
// surface" call as A3 / A7-D1 / A4-D2). A1 verbs reach the *live*
// `FederationRegistry` via AdminContext (P5 precedent). No Space-DAG event, no
// `EventAccepted`.

/// Project a `&str` peer node URI to the typed registry key.
fn node_xgid(s: &str) -> NodeXgid {
    NodeXgid::from_xgid(Xgid::new(s.to_string()))
}

const FED_LIST_DEFAULT_LIMIT: usize = 50;
const FED_LIST_MAX_LIMIT: usize = 500;

// ── federation list — READ (not audited; A1-D2 paginated) ────────────────────────

/// Args for `federation list` (§6.A1).
#[derive(Debug, Clone, Default, clap::Args)]
pub struct FederationListArgs {
    /// `active | pending | rejected | revoked | all` (default `all`). Filters on
    /// the FAC-D2 `FederationState` field. Note: *pending requests* awaiting
    /// approval live in the separate pending-federation queue, not as `Pending`
    /// relationships — `--state pending` here matches registry relationships in
    /// the `Pending` state, which are rare in 2a (the gate enqueues to the queue).
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub limit: Option<usize>,
    #[arg(long)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FederationListResult {
    pub relationships: Vec<FederationRelationship>,
    pub total_matched: usize,
    pub returned: usize,
    pub next_cursor: Option<String>,
}

/// `federation list` — paginated read of the federation relationships.
pub async fn federation_list(
    ctx: &mut AdminContext<'_>,
    args: FederationListArgs,
) -> Result<FederationListResult, AdminError> {
    // Filter on the FAC-D2 state field; `all`/None → no state filter.
    let state_filter: Option<FederationState> = match args.state.as_deref() {
        None | Some("all") => None,
        Some("active") => Some(FederationState::Active),
        Some("pending") => Some(FederationState::Pending),
        Some("rejected") => Some(FederationState::Rejected),
        Some("revoked") => Some(FederationState::Revoked),
        Some(other) => {
            return Err(AdminError::new(
                "FED_3001",
                Stage::Validate,
                format!(
                    "invalid state filter '{other}' (expected active|pending|rejected|revoked|all)"
                ),
            ));
        }
    };
    let limit = args
        .limit
        .unwrap_or(FED_LIST_DEFAULT_LIMIT)
        .min(FED_LIST_MAX_LIMIT);

    let registry = Arc::clone(ctx.require_federation_registry(Stage::Register)?);
    let mut matched: Vec<FederationRelationship> = {
        let reg = registry.lock().await;
        let mut v: Vec<FederationRelationship> = reg
            .all()
            .into_iter()
            .filter(|r| state_filter.is_none_or(|s| r.state == s))
            .cloned()
            .collect();
        v.sort_by(|a, b| a.peer_node_id.as_str().cmp(b.peer_node_id.as_str()));
        v
    };
    let total_matched = matched.len();

    // Cursor = the last peer_node_id returned on the prior page; the next page
    // starts strictly after it (relationships are sorted by peer_node_id).
    if let Some(cursor) = args.cursor.as_deref() {
        matched.retain(|r| r.peer_node_id.as_str() > cursor);
    }
    let has_more = matched.len() > limit;
    let page: Vec<FederationRelationship> = matched.into_iter().take(limit).collect();
    let next_cursor = if has_more {
        page.last().map(|r| r.peer_node_id.as_str().to_string())
    } else {
        None
    };
    Ok(FederationListResult {
        returned: page.len(),
        relationships: page,
        total_matched,
        next_cursor,
    })
}

// ── federation defederate — DESTRUCTIVE (audited) ────────────────────────────────

/// Args for `federation defederate` (§6.A1).
#[derive(Debug, Clone, clap::Args)]
pub struct FederationDefederateArgs {
    /// Peer Node URI to defederate from.
    pub peer_node_id: String,
    #[arg(long)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FederationDefederateResult {
    pub peer_node_id: String,
    pub defederated_at: String,
    /// Spaces that were shared over the terminated relationship (reported, not
    /// deep-GC'd — see scope note).
    pub cleaned_spaces: Vec<String>,
}

/// `federation defederate` — terminate the node-to-node federation relationship
/// in the *live* registry and persist it (so federation paths stop treating the
/// peer as federated at once). DESTRUCTIVE → audited.
///
/// Scope (honest, D-065): this removes the relationship record and reports its
/// `shared_spaces`; it does **not** perform deep replica-data garbage collection
/// (D-022/§3.15) — that is the federation-cleanup subsystem — nor does it send a
/// network `federation.goodbye` (the peer observes the relationship gone on next
/// interaction). Both are part of the deferred federation-admin-control arc.
pub async fn federation_defederate(
    ctx: &mut AdminContext<'_>,
    args: FederationDefederateArgs,
) -> Result<FederationDefederateResult, AdminError> {
    let defederated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let registry = Arc::clone(ctx.require_federation_registry(Stage::Register)?);
    let path = ctx.federation_registry_path();
    let key = node_xgid(&args.peer_node_id);

    let cleaned_spaces: Vec<String> = {
        let mut reg = registry.lock().await;
        if reg.get(&key).is_none() {
            return Err(AdminError::new(
                "FED_3004",
                Stage::Register,
                format!("not federated with peer: {}", args.peer_node_id),
            ));
        }
        let removed = reg.remove(&key).expect("checked present above");
        let spaces: Vec<String> = removed
            .shared_spaces
            .iter()
            .map(|s| s.as_str().to_string())
            .collect();
        if let Err(e) = reg.save(&path) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("federation registry save failed: {e}"),
            ));
        }
        spaces
    };

    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"peer_node_id\":{:?},\"reason\":{:?}}}",
        args.peer_node_id, args.reason
    ));
    record_action(
        &conn,
        ctx,
        "federation defederate",
        Some(args.peer_node_id.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;
    Ok(FederationDefederateResult {
        peer_node_id: args.peer_node_id,
        defederated_at,
        cleaned_spaces,
    })
}

// ── federation accept — WRITE (audited) — FAC-D1a sub-arc 2a ──────────────────────

/// Args for `federation accept` (§6.A1, sub-arc 2a).
#[derive(Debug, Clone, clap::Args)]
pub struct FederationAcceptArgs {
    /// Peer Node URI to approve (must have a pending request in the queue).
    pub peer_node_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FederationAcceptResult {
    pub peer_node_id: String,
    pub accepted_at: String,
    pub shared_spaces: Vec<String>,
}

/// `federation accept <peer>` — approve a queued inbound federation request
/// (FAC-D1a). Removes the request from the pending queue, upserts the
/// relationship as `Active` (so the peer's next inbound reconnect passes the
/// gate), and schedules an immediate outbound reconnect via the existing
/// scheduler path so the Node also dials the now-approved peer. WRITE → A6
/// trail. The session itself is established by the scheduler / the peer's
/// reconnect, not synchronously here.
pub async fn federation_accept(
    ctx: &mut AdminContext<'_>,
    args: FederationAcceptArgs,
) -> Result<FederationAcceptResult, AdminError> {
    let accepted_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let queue = Arc::clone(ctx.require_federation_queue(Stage::Register)?);
    let registry = Arc::clone(ctx.require_federation_registry(Stage::Register)?);
    let qpath = ctx.federation_queue_path();
    let rpath = ctx.federation_registry_path();
    let key = node_xgid(&args.peer_node_id);

    // Pull the request out of the queue (its handshake facts complete the
    // relationship). Absent → nothing to accept.
    let req = {
        let mut q = queue.lock().await;
        let req = q.remove(&key).ok_or_else(|| {
            AdminError::new(
                "FED_3005",
                Stage::Register,
                format!("no pending federation request for peer: {}", args.peer_node_id),
            )
        })?;
        if let Err(e) = q.save(&qpath) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("pending federation queue save failed: {e}"),
            ));
        }
        req
    };

    let shared_spaces: Vec<String> =
        req.shared_spaces.iter().map(|s| s.as_str().to_string()).collect();

    {
        let mut reg = registry.lock().await;
        reg.upsert(FederationRelationship {
            peer_node_id: key.clone(),
            shared_spaces: req.shared_spaces.clone(),
            negotiated_version: req.negotiated_version.clone(),
            negotiated_serialisation: req.negotiated_serialisation.clone(),
            // No live session yet — minted when the approved peer connects.
            session_id: "xgen://pending/accept".to_string(),
            last_connected: accepted_at.clone(),
            peer_url: req.peer_url.clone(),
            state: FederationState::Active,
        });
        // Schedule an immediate outbound reconnect: mark lost + make it due now
        // so the F-1c scheduler dials the approved peer on its next tick.
        let now = Utc::now();
        reg.mark_lost(&key, now);
        reg.update_next_reconnect(&key, now);
        if let Err(e) = reg.save(&rpath) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("federation registry save failed: {e}"),
            ));
        }
    }

    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"peer_node_id\":{:?}}}",
        args.peer_node_id
    ));
    record_action(
        &conn,
        ctx,
        "federation accept",
        Some(args.peer_node_id.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;
    Ok(FederationAcceptResult {
        peer_node_id: args.peer_node_id,
        accepted_at,
        shared_spaces,
    })
}

// ── federation reject — DESTRUCTIVE (audited) — FAC-D1a sub-arc 2a ─────────────────

/// Args for `federation reject` (§6.A1, sub-arc 2a).
#[derive(Debug, Clone, clap::Args)]
pub struct FederationRejectArgs {
    /// Peer Node URI to reject (must have a pending request in the queue).
    pub peer_node_id: String,
    #[arg(long)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FederationRejectResult {
    pub peer_node_id: String,
    pub rejected_at: String,
}

/// `federation reject <peer>` — deny a queued inbound federation request
/// (FAC-D1a). Removes the request from the queue and writes a permanent
/// `Rejected` tombstone to the registry (built from the queued handshake
/// facts) so the inbound gate refuses future requests from this peer *without*
/// re-queuing them (checkpoint #3). The operator can still deliberately
/// re-establish via `federation initiate` (ungated), which clears the
/// tombstone on success. DESTRUCTIVE → A6 trail.
pub async fn federation_reject(
    ctx: &mut AdminContext<'_>,
    args: FederationRejectArgs,
) -> Result<FederationRejectResult, AdminError> {
    let rejected_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let queue = Arc::clone(ctx.require_federation_queue(Stage::Register)?);
    let registry = Arc::clone(ctx.require_federation_registry(Stage::Register)?);
    let qpath = ctx.federation_queue_path();
    let rpath = ctx.federation_registry_path();
    let key = node_xgid(&args.peer_node_id);

    let req = {
        let mut q = queue.lock().await;
        let req = q.remove(&key).ok_or_else(|| {
            AdminError::new(
                "FED_3005",
                Stage::Register,
                format!("no pending federation request for peer: {}", args.peer_node_id),
            )
        })?;
        if let Err(e) = q.save(&qpath) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("pending federation queue save failed: {e}"),
            ));
        }
        req
    };

    {
        let mut reg = registry.lock().await;
        reg.upsert(FederationRelationship {
            peer_node_id: key.clone(),
            shared_spaces: req.shared_spaces.clone(),
            negotiated_version: req.negotiated_version.clone(),
            negotiated_serialisation: req.negotiated_serialisation.clone(),
            session_id: "xgen://rejected/tombstone".to_string(),
            last_connected: rejected_at.clone(),
            peer_url: req.peer_url.clone(),
            state: FederationState::Rejected,
        });
        if let Err(e) = reg.save(&rpath) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("federation registry save failed: {e}"),
            ));
        }
    }

    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"peer_node_id\":{:?},\"reason\":{:?}}}",
        args.peer_node_id, args.reason
    ));
    record_action(
        &conn,
        ctx,
        "federation reject",
        Some(args.peer_node_id.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;
    Ok(FederationRejectResult {
        peer_node_id: args.peer_node_id,
        rejected_at,
    })
}

// ── federation initiate — WRITE (audited) — FAC-D1a sub-arc 2a ────────────────────

/// Args for `federation initiate` (§6.A1, sub-arc 2a).
#[derive(Debug, Clone, clap::Args)]
pub struct FederationInitiateArgs {
    /// Peer Node URI to initiate federation with. v1 scope: the peer must
    /// already be *known* to the registry (e.g. a `Rejected` tombstone or a
    /// lost relationship) — its stored endpoint + shared Spaces drive the
    /// outbound handshake. Fresh-URL bootstrap to an unknown peer is deferred
    /// to the bootstrap-client arc.
    pub peer_node_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FederationInitiateResult {
    pub peer_node_id: String,
    pub peer_url: String,
    pub initiated_at: String,
}

/// `federation initiate <peer>` — operator-outbound federation establish
/// (FAC-D1a). **Ungated even when `require_approval = true`** — the operator
/// initiating *is* the approval (inbound-only gate). On success the outbound
/// session upserts the relationship `Active`, which clears any prior
/// `Rejected`/`Pending` state (the re-allow escape hatch for a rejected peer).
/// v1 targets a *known* peer: the relationship's stored `peer_url` +
/// `shared_spaces` drive `reconnect::attempt_reconnect`, spawned detached (the
/// handshake + long-lived session run in the background; this verb reports the
/// attempt was dispatched). WRITE → A6 trail.
pub async fn federation_initiate(
    ctx: &mut AdminContext<'_>,
    args: FederationInitiateArgs,
) -> Result<FederationInitiateResult, AdminError> {
    let initiated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let registry = Arc::clone(ctx.require_federation_registry(Stage::Register)?);
    let runtime = Arc::clone(ctx.require_runtime(Stage::Register)?);
    let key = node_xgid(&args.peer_node_id);

    // v1: only known peers. Look up the stored endpoint + shared Spaces.
    let (peer_url, shared_spaces) = {
        let reg = registry.lock().await;
        let rel = reg.get(&key).ok_or_else(|| {
            AdminError::new(
                "FED_3006",
                Stage::Register,
                format!(
                    "no known federation relationship for peer {} — initiate targets \
                     known peers in v1 (fresh-URL bootstrap is deferred)",
                    args.peer_node_id
                ),
            )
        })?;
        let url = rel.peer_url.clone().ok_or_else(|| {
            AdminError::new(
                "FED_3007",
                Stage::Register,
                format!("peer {} has no stored endpoint URL to dial", args.peer_node_id),
            )
        })?;
        (url, rel.shared_spaces.clone())
    };

    // Session handles for the outbound attempt. The Node keypair + id come from
    // the live runtime; the senders must be present (wiring, not user error).
    let client_senders = ctx
        .client_senders
        .as_ref()
        .ok_or_else(|| AdminError::generic(Stage::Register, "no client senders available"))?
        .clone();
    let federation_peer_senders = ctx
        .federation_peer_senders
        .as_ref()
        .ok_or_else(|| {
            AdminError::generic(Stage::Register, "no federation peer senders available")
        })?
        .clone();
    let (node_keypair, home_node_id) = {
        let rt = runtime.lock().await;
        (Arc::new(rt.node_keypair.clone()), rt.node_id.clone())
    };
    // local_mode + this Node's own endpoint (dial-back hint) come from config.
    let cfg = crate::app::try_load_config(ctx.config_path);
    let local_mode = cfg.as_ref().map(|c| c.node.local_mode).unwrap_or(false);
    let self_url = cfg
        .as_ref()
        .map(|c| c.node.listen.clone())
        .unwrap_or_default();
    let spaces_dir = crate::app::resolve_spaces_dir(ctx.config_path, ctx.data_dir);
    let identities_path = ctx.identities_path();
    let registry_path = ctx.federation_registry_path();
    let attempt_cursor = Arc::new(Mutex::new(HashMap::new()));
    // 2b FAC-D3 — the operator-initiated session routes inbound events through
    // process_inbound's policy gate. Prefer the live policy store (the same Arc
    // the resident's enforcement sites consult, threaded in Commit 3); fall
    // back to a disk load for callers/tests that don't carry it.
    let federation_policy = match &ctx.federation_policy {
        Some(p) => Arc::clone(p),
        None => {
            let p = ctx.federation_policy_path();
            let store = if p.exists() {
                FederationPolicyStore::load(&p).unwrap_or_default()
            } else {
                FederationPolicyStore::new()
            };
            Arc::new(Mutex::new(store))
        }
    };

    tokio::spawn(crate::reconnect::attempt_reconnect(
        runtime,
        client_senders,
        federation_peer_senders,
        registry,
        registry_path,
        node_keypair,
        home_node_id,
        spaces_dir,
        identities_path,
        local_mode,
        self_url,
        key,
        peer_url.clone(),
        shared_spaces,
        attempt_cursor,
        federation_policy,
    ));

    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"peer_node_id\":{:?}}}",
        args.peer_node_id
    ));
    record_action(
        &conn,
        ctx,
        "federation initiate",
        Some(args.peer_node_id.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;
    Ok(FederationInitiateResult {
        peer_node_id: args.peer_node_id,
        peer_url,
        initiated_at,
    })
}

// ── federation set-policy — WRITE (audited) — FAC-D3/D4 sub-arc 2b ────────────────

/// Args for `federation set-policy` (§6.A1, sub-arc 2b).
#[derive(Debug, Clone, clap::Args)]
pub struct FederationSetPolicyArgs {
    /// Peer Node URI the policy applies to (may pre-exist any relationship —
    /// pre-deny is intentional, FAC-D4).
    pub peer_node_id: String,
    /// `allow | deny`. `deny` blocks the peer entirely (inbound + outbound)
    /// without tearing down the relationship; `allow` permits, optionally
    /// narrowed by `--allowed-space`.
    #[arg(long)]
    pub mode: String,
    /// Restrictive allow-list of shared Space ids (repeatable). Omit for "all
    /// shared Spaces". Only meaningful with `--mode allow` (restrictive-only:
    /// the effective set is `shared_spaces ∩ allowed_spaces`).
    #[arg(long = "allowed-space")]
    pub allowed_space: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FederationSetPolicyResult {
    pub peer_node_id: String,
    pub mode: String,
    pub allowed_spaces: Option<Vec<String>>,
    pub set_at: String,
}

/// `federation set-policy <peer> --mode allow|deny [--allowed-space …]` —
/// upsert the per-peer federation policy (FAC-D3/D4). Effective immediately:
/// it mutates the live policy store the enforcement sites consult, then
/// persists to `xgen-node_federation_policy.json`. A peer with no relationship
/// yet may still have a policy set (pre-deny). WRITE → A6 trail.
pub async fn federation_set_policy(
    ctx: &mut AdminContext<'_>,
    args: FederationSetPolicyArgs,
) -> Result<FederationSetPolicyResult, AdminError> {
    let set_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let mode = match args.mode.to_ascii_lowercase().as_str() {
        "allow" => PolicyMode::Allow,
        "deny" => PolicyMode::Deny,
        other => {
            return Err(AdminError::new(
                "FED_3008",
                Stage::Validate,
                format!("invalid policy mode '{other}' (expected allow|deny)"),
            ));
        }
    };
    let allowed_spaces: Option<Vec<SpaceXgid>> = if args.allowed_space.is_empty() {
        None
    } else {
        Some(
            args.allowed_space
                .iter()
                .map(|s| SpaceXgid::from_xgid(Xgid::new(s.clone())))
                .collect(),
        )
    };

    let store = Arc::clone(ctx.require_federation_policy(Stage::Register)?);
    let path = ctx.federation_policy_path();
    {
        let mut s = store.lock().await;
        s.set(
            node_xgid(&args.peer_node_id),
            FederationPolicy {
                mode,
                allowed_spaces: allowed_spaces.clone(),
            },
        );
        if let Err(e) = s.save(&path) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("federation policy save failed: {e}"),
            ));
        }
    }

    let mode_str = match mode {
        PolicyMode::Allow => "allow",
        PolicyMode::Deny => "deny",
    }
    .to_string();
    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"peer_node_id\":{:?},\"mode\":{:?},\"allowed_space\":{:?}}}",
        args.peer_node_id, mode_str, args.allowed_space
    ));
    record_action(
        &conn,
        ctx,
        "federation set-policy",
        Some(args.peer_node_id.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;

    Ok(FederationSetPolicyResult {
        peer_node_id: args.peer_node_id,
        mode: mode_str,
        allowed_spaces: allowed_spaces
            .map(|v| v.iter().map(|s| s.as_str().to_string()).collect()),
        set_at,
    })
}

// ── federation show-policy — READ (not audited) — FAC-D3/D4 sub-arc 2b ────────────

/// Args for `federation show-policy` (§6.A1, sub-arc 2b).
#[derive(Debug, Clone, clap::Args)]
pub struct FederationShowPolicyArgs {
    /// Peer Node URI to show the effective policy for.
    pub peer_node_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FederationShowPolicyResult {
    pub peer_node_id: String,
    pub mode: String,
    pub allowed_spaces: Option<Vec<String>>,
    /// `true` when no policy is stored for this peer — the values shown are the
    /// default (permit-all; prime invariant), not an operator-set policy.
    pub is_default: bool,
}

/// `federation show-policy <peer>` — read the per-peer policy, or the default
/// (permit-all) with an explicit `is_default` marker when none is stored.
/// READ → not audited.
pub async fn federation_show_policy(
    ctx: &mut AdminContext<'_>,
    args: FederationShowPolicyArgs,
) -> Result<FederationShowPolicyResult, AdminError> {
    let store = Arc::clone(ctx.require_federation_policy(Stage::Register)?);
    let s = store.lock().await;
    let (mode, allowed_spaces, is_default) = match s.get(&node_xgid(&args.peer_node_id)) {
        Some(p) => {
            let mode = match p.mode {
                PolicyMode::Allow => "allow",
                PolicyMode::Deny => "deny",
            }
            .to_string();
            let spaces = p
                .allowed_spaces
                .as_ref()
                .map(|v| v.iter().map(|x| x.as_str().to_string()).collect());
            (mode, spaces, false)
        }
        // Default-permit (prime invariant): no stored policy → Allow + all.
        None => ("allow".to_string(), None, true),
    };
    Ok(FederationShowPolicyResult {
        peer_node_id: args.peer_node_id,
        mode,
        allowed_spaces,
        is_default,
    })
}

// ════════════════════════════════════════════════════════════════════════════════
// A2 — Auth Module registry (auth-module-registry D-071 arc; design §6.A2, Appendix K.2.5)
// ════════════════════════════════════════════════════════════════════════════════
// CRUD over the registry of trusted Auth Modules (AMR-D1 standalone — record +
// store + verbs, NO runtime consumer this arc; the registration-pipeline / 3006
// consultation is a deferred future arc). Admin error codes are a fresh
// `AUTHMOD_61xx` block: Auth Modules attest Identity tiers, so they sit in the
// identity 6000 domain, sub-block 6100 (free of the IDENT_60xx/602x codes) and
// DISTINCT from the deferred enforcement code `AuthModuleUntrusted`/3006 (which
// is the wire-level untrusted-attestation rejection, not an admin-verb error):
//   AUTHMOD_6101 — unknown module (revoke / set-tiers reference a missing id)
//   AUTHMOD_6102 — invalid `--pubkey` (not a base64url-encoded Ed25519 key)
//   AUTHMOD_6103 — invalid tier (a `--tier` value outside 1..=4)

/// Make an `AuthModuleXgid` from a raw module-id URI string (revoke / set-tiers
/// reference an existing module by its id). Sibling to `node_xgid`.
fn auth_module_xgid(s: &str) -> AuthModuleXgid {
    AuthModuleXgid::from_xgid(Xgid::new(s.to_string()))
}

/// Parse `--pubkey` (the module's base64url-encoded Ed25519 verifying key) into
/// a typed `AuthModuleXgid` via `from_pubkey`, so a malformed id is impossible
/// (checkpoint #1 lock — the verb derives `module_id`, AMR-D2/D3). Reuses the
/// canonical `crypto::encoding::decode` (base64url; no drift — D-067).
fn module_id_from_pubkey(pubkey: &str) -> Result<AuthModuleXgid, AdminError> {
    let invalid = |msg: String| AdminError::new("AUTHMOD_6102", Stage::Validate, msg);
    let bytes = encoding::decode(pubkey)
        .map_err(|e| invalid(format!("--pubkey is not valid base64url: {e}")))?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| invalid(format!("--pubkey decodes to {} bytes, expected 32", bytes.len())))?;
    let vk = VerifyingKey::from_bytes(&arr)
        .map_err(|e| invalid(format!("--pubkey is not a valid Ed25519 key: {e}")))?;
    Ok(AuthModuleXgid::from_pubkey(&vk))
}

/// Validate + map a repeated `--tier` set (1..=4) to `Vec<AuthTier>`
/// (`AUTHMOD_6103` on an out-of-range value).
fn parse_tiers(tiers: &[u32]) -> Result<Vec<AuthTier>, AdminError> {
    tiers
        .iter()
        .map(|t| {
            AuthTier::from_u32(*t).ok_or_else(|| {
                AdminError::new(
                    "AUTHMOD_6103",
                    Stage::Validate,
                    format!("invalid tier {t} (expected 1..=4)"),
                )
            })
        })
        .collect()
}

fn tiers_to_u32(tiers: &[AuthTier]) -> Vec<u32> {
    tiers.iter().map(|t| t.as_u32()).collect()
}

// ── auth-module list — READ (not audited) ─────────────────────────────────────────

/// Args for `auth-module list` (§6.A2).
#[derive(Debug, Clone, Default, clap::Args)]
pub struct AuthModuleListArgs {}

#[derive(Debug, Clone, Serialize)]
pub struct AuthModuleSummary {
    pub module_id: String,
    pub endpoint_url: String,
    pub accepted_tiers: Vec<u32>,
    pub registered_at: String,
    pub revoked: bool,
    pub revoked_at: Option<String>,
}

impl AuthModuleSummary {
    fn from_record(r: &AuthModuleRecord) -> Self {
        Self {
            module_id: r.module_id.as_str().to_string(),
            endpoint_url: r.endpoint_url.clone(),
            accepted_tiers: tiers_to_u32(&r.accepted_tiers),
            registered_at: r.registered_at.clone(),
            revoked: r.revoked,
            revoked_at: r.revoked_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthModuleListResult {
    pub modules: Vec<AuthModuleSummary>,
}

/// `auth-module list` — enumerate the registered Auth Modules (revoked ones
/// included, marked `revoked: true` — A2-D1 block-only). READ → not audited.
pub async fn auth_module_list(
    ctx: &mut AdminContext<'_>,
) -> Result<AuthModuleListResult, AdminError> {
    let registry = Arc::clone(ctx.require_auth_module_registry(Stage::Register)?);
    let r = registry.lock().await;
    let modules = r.all().iter().map(|rec| AuthModuleSummary::from_record(rec)).collect();
    Ok(AuthModuleListResult { modules })
}

// ── auth-module register — WRITE (audited) ─────────────────────────────────────────

/// Args for `auth-module register` (§6.A2). `--pubkey` (checkpoint #1 lock) — the
/// verb derives `module_id` from the key, so a malformed id is impossible.
#[derive(Debug, Clone, clap::Args)]
pub struct AuthModuleRegisterArgs {
    /// The module's base64url-encoded Ed25519 verifying key. `module_id` is
    /// derived from it (AMR-D2/D3 — no separate id is accepted).
    #[arg(long)]
    pub pubkey: String,
    /// Where the module is reached (the `auth-module test` probe target).
    #[arg(long)]
    pub endpoint: String,
    /// An accepted Auth Tier (1..=4), repeatable.
    #[arg(long = "tier")]
    pub tier: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthModuleRegisterResult {
    pub module_id: String,
    pub endpoint_url: String,
    pub accepted_tiers: Vec<u32>,
    pub registered_at: String,
}

/// `auth-module register --pubkey <key> --endpoint <url> [--tier N]…` — add (or
/// replace) a trusted Auth Module. `module_id` is derived from `--pubkey`
/// (AMR-D2/D3). WRITE → A6 trail.
pub async fn auth_module_register(
    ctx: &mut AdminContext<'_>,
    args: AuthModuleRegisterArgs,
) -> Result<AuthModuleRegisterResult, AdminError> {
    let registered_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let module_id = module_id_from_pubkey(&args.pubkey)?;
    let accepted_tiers = parse_tiers(&args.tier)?;

    let record = AuthModuleRecord {
        module_id: module_id.clone(),
        endpoint_url: args.endpoint.clone(),
        accepted_tiers: accepted_tiers.clone(),
        registered_at: registered_at.clone(),
        revoked: false,
        revoked_at: None,
    };

    let registry = Arc::clone(ctx.require_auth_module_registry(Stage::Register)?);
    let path = ctx.auth_module_registry_path();
    {
        let mut r = registry.lock().await;
        r.register(record);
        if let Err(e) = r.save(&path) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("auth module registry save failed: {e}"),
            ));
        }
    }

    let module_id_str = module_id.as_str().to_string();
    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"pubkey\":{:?},\"endpoint\":{:?},\"tier\":{:?}}}",
        args.pubkey, args.endpoint, args.tier
    ));
    record_action(
        &conn,
        ctx,
        "auth-module register",
        Some(module_id_str.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;

    Ok(AuthModuleRegisterResult {
        module_id: module_id_str,
        endpoint_url: args.endpoint,
        accepted_tiers: tiers_to_u32(&accepted_tiers),
        registered_at,
    })
}

// ── auth-module revoke — DESTRUCTIVE (audited) — A2-D1 block-only ────────────────────

/// Args for `auth-module revoke` (§6.A2).
#[derive(Debug, Clone, clap::Args)]
pub struct AuthModuleRevokeArgs {
    /// The `module_id` URI of the Auth Module to revoke.
    pub module_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthModuleRevokeResult {
    pub module_id: String,
    pub revoked_at: String,
}

/// `auth-module revoke <module_id>` — mark a module untrusted (A2-D1 block-only:
/// the record is RETAINED + still listed, just flagged `revoked`). Unknown id →
/// `AUTHMOD_6101`. DESTRUCTIVE → A6 trail.
pub async fn auth_module_revoke(
    ctx: &mut AdminContext<'_>,
    args: AuthModuleRevokeArgs,
) -> Result<AuthModuleRevokeResult, AdminError> {
    let revoked_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let module_id = auth_module_xgid(&args.module_id);

    let registry = Arc::clone(ctx.require_auth_module_registry(Stage::Register)?);
    let path = ctx.auth_module_registry_path();
    {
        let mut r = registry.lock().await;
        if !r.revoke(&module_id, revoked_at.clone()) {
            return Err(AdminError::new(
                "AUTHMOD_6101",
                Stage::Register,
                format!("no Auth Module registered with id {}", args.module_id),
            ));
        }
        if let Err(e) = r.save(&path) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("auth module registry save failed: {e}"),
            ));
        }
    }

    let conn = open_audit(ctx)?;
    let args_hash =
        AuditEntry::compute_args_hash(&format!("{{\"module_id\":{:?}}}", args.module_id));
    record_action(
        &conn,
        ctx,
        "auth-module revoke",
        Some(args.module_id.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;

    Ok(AuthModuleRevokeResult {
        module_id: args.module_id,
        revoked_at,
    })
}

// ── auth-module set-tiers — WRITE (audited) ─────────────────────────────────────────

/// Args for `auth-module set-tiers` (§6.A2).
#[derive(Debug, Clone, clap::Args)]
pub struct AuthModuleSetTiersArgs {
    /// The `module_id` URI of the Auth Module to update.
    pub module_id: String,
    /// An accepted Auth Tier (1..=4), repeatable. Replaces the module's set.
    #[arg(long = "tier")]
    pub tier: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthModuleSetTiersResult {
    pub module_id: String,
    pub accepted_tiers: Vec<u32>,
}

/// `auth-module set-tiers <module_id> [--tier N]…` — replace a module's accepted
/// tier set. Unknown id → `AUTHMOD_6101`; bad tier → `AUTHMOD_6103`. WRITE → A6.
pub async fn auth_module_set_tiers(
    ctx: &mut AdminContext<'_>,
    args: AuthModuleSetTiersArgs,
) -> Result<AuthModuleSetTiersResult, AdminError> {
    let accepted_tiers = parse_tiers(&args.tier)?;
    let module_id = auth_module_xgid(&args.module_id);

    let registry = Arc::clone(ctx.require_auth_module_registry(Stage::Register)?);
    let path = ctx.auth_module_registry_path();
    {
        let mut r = registry.lock().await;
        if !r.set_tiers(&module_id, accepted_tiers.clone()) {
            return Err(AdminError::new(
                "AUTHMOD_6101",
                Stage::Register,
                format!("no Auth Module registered with id {}", args.module_id),
            ));
        }
        if let Err(e) = r.save(&path) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("auth module registry save failed: {e}"),
            ));
        }
    }

    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"module_id\":{:?},\"tier\":{:?}}}",
        args.module_id, args.tier
    ));
    record_action(
        &conn,
        ctx,
        "auth-module set-tiers",
        Some(args.module_id.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;

    Ok(AuthModuleSetTiersResult {
        module_id: args.module_id,
        accepted_tiers: tiers_to_u32(&accepted_tiers),
    })
}

// ── auth-module test — READ (not audited) — A2-D2 ad-hoc probe (checkpoint #2) ──────

/// Fail-fast reachability-probe timeout. A *fresh* choice for an ad-hoc probe —
/// deliberately NOT either federation timeout (`PENDING_TIMEOUT_SECS` = 30 s,
/// `FEDERATION_RELATIONSHIP_TIMEOUT_SECS` = 180 s); a probe should fail fast.
/// Configurability is deferred.
const AUTH_MODULE_PROBE_TIMEOUT_SECS: u64 = 5;

/// Parse a `host:port` TCP target out of an Auth Module `endpoint_url`
/// (connectivity-only probe — no request is sent, so only the authority is
/// needed). Returns `None` for an unparseable / unknown-scheme endpoint; the
/// caller maps that to `reachable: false` (a result, not an error — per
/// checkpoint #2 the only error path is unknown-module). Honest v1 scope:
/// IPv6-literal `[..]:port` authorities are not specially handled.
fn endpoint_host_port(endpoint: &str) -> Option<(String, u16)> {
    let (scheme, rest) = match endpoint.split_once("://") {
        Some((s, r)) => (Some(s.to_ascii_lowercase()), r),
        None => (None, endpoint),
    };
    // Authority = up to the first path/query/fragment delimiter; drop userinfo.
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    if host_port.is_empty() {
        return None;
    }
    let (host, port) = match host_port.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().ok()?),
        None => {
            let default = match scheme.as_deref() {
                Some("https") | Some("wss") => 443,
                Some("http") | Some("ws") => 80,
                _ => return None,
            };
            (host_port.to_string(), default)
        }
    };
    if host.is_empty() {
        None
    } else {
        Some((host, port))
    }
}

/// Args for `auth-module test` (§6.A2, checkpoint #2).
#[derive(Debug, Clone, clap::Args)]
pub struct AuthModuleTestArgs {
    /// The `module_id` URI of the Auth Module to probe.
    pub module_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthModuleTestResult {
    pub module_id: String,
    pub endpoint_url: String,
    /// Whether a TCP connection to `endpoint_url` succeeded within the probe
    /// timeout.
    pub reachable: bool,
    /// Round-trip connect time in ms when `reachable`; `None` otherwise.
    pub response_time_ms: Option<u64>,
    /// Why the probe judged the module unreachable; `None` when `reachable`.
    pub reason: Option<String>,
    /// The Auth Tiers this Node has the module registered to issue (the STORED
    /// `accepted_tiers`, display-only — connectivity-only means the module
    /// reports nothing, so there is no module-reported set to compare).
    pub accepted_tiers: Vec<u32>,
}

/// `auth-module test <module_id>` — ad-hoc connectivity probe (A2-D2). TCP-connects
/// to the module's `endpoint_url` with a 5 s fail-fast timeout and reports
/// reachability + connect time + the stored tiers. **Connectivity-only (honest-thin):
/// no challenge/response** — the signed-nonce handshake is unspecced and waits for
/// the Auth Module protocol arc (AMR-D1). Unknown module → `AUTHMOD_6101`;
/// unreachable → a `reachable: false` result, NOT an error. READ → not audited.
pub async fn auth_module_test(
    ctx: &mut AdminContext<'_>,
    args: AuthModuleTestArgs,
) -> Result<AuthModuleTestResult, AdminError> {
    let module_id = auth_module_xgid(&args.module_id);

    // Pull the probe inputs out under the lock, then drop it before any network
    // I/O (never hold the registry mutex across an `.await` on the socket).
    let (endpoint_url, accepted_tiers) = {
        let registry = Arc::clone(ctx.require_auth_module_registry(Stage::Register)?);
        let r = registry.lock().await;
        match r.get(&module_id) {
            Some(rec) => (rec.endpoint_url.clone(), tiers_to_u32(&rec.accepted_tiers)),
            None => {
                return Err(AdminError::new(
                    "AUTHMOD_6101",
                    Stage::Register,
                    format!("no Auth Module registered with id {}", args.module_id),
                ));
            }
        }
    };

    let (reachable, response_time_ms, reason) = match endpoint_host_port(&endpoint_url) {
        None => (
            false,
            None,
            Some(format!("could not parse host:port from endpoint_url '{endpoint_url}'")),
        ),
        Some((host, port)) => {
            let start = Instant::now();
            let connect = tokio::net::TcpStream::connect((host.as_str(), port));
            match tokio::time::timeout(
                Duration::from_secs(AUTH_MODULE_PROBE_TIMEOUT_SECS),
                connect,
            )
            .await
            {
                Ok(Ok(_stream)) => (true, Some(start.elapsed().as_millis() as u64), None),
                Ok(Err(e)) => (false, None, Some(format!("connect failed: {e}"))),
                Err(_) => (
                    false,
                    None,
                    Some(format!(
                        "timed out after {AUTH_MODULE_PROBE_TIMEOUT_SECS}s"
                    )),
                ),
            }
        }
    };

    Ok(AuthModuleTestResult {
        module_id: args.module_id,
        endpoint_url,
        reachable,
        response_time_ms,
        reason,
        accepted_tiers,
    })
}

// ════════════════════════════════════════════════════════════════════════════════
// A3 — Bootstrap-client administration (bootstrap-client D-071 arc; design §6.A3)
// ════════════════════════════════════════════════════════════════════════════════
// CLIENT-ONLY (A3-D1): this Node registers *itself* with Bootstrap Nodes + manages
// its own advertisement. The 5 verbs operate on the local bootstrap store (BC-D1:
// registrations map + self-info, the combined `xgen-node_bootstrap.json`); the
// register/deregister verbs drive the C2 framed send-path (BC-D3 — NOT HTTP).
//
// Admin error codes are a fresh `BOOT_71xx` block in the bootstrap 7000 domain
// (spec §3.14.8), DISTINCT from the spec's wire-level 7001–7005 (those are the
// Bootstrap-Node-server's protocol rejections, not these admin-verb errors):
//   BOOT_7101 — unknown bootstrap node (deregister references an unregistered id)
//   BOOT_7102 — invalid `--pubkey` (not a base64url-encoded Ed25519 key)
//   BOOT_7103 — invalid tier (a `--tier` value outside 1..=4)
//   BOOT_7110 — bootstrap node exchange failed (connect/send/recv/timeout)
//   BOOT_7111 — ack verification failed (bad signature or node_id mismatch, Pin 2)

/// Make a `NodeXgid` from a raw bootstrap-node-id URI (deregister references an
/// existing registration by its id). Sibling to `node_xgid` / `auth_module_xgid`.
fn bootstrap_node_xgid(s: &str) -> NodeXgid {
    NodeXgid::from_xgid(Xgid::new(s.to_string()))
}

/// Parse `--pubkey` (the bootstrap node's base64url Ed25519 verifying key) into a
/// typed `NodeXgid` via `from_pubkey`, so a malformed id is impossible (checkpoint
/// #1(c)). Reuses canonical `crypto::encoding::decode` (no drift, D-067). Sibling
/// to `module_id_from_pubkey`.
fn bootstrap_id_from_pubkey(pubkey: &str) -> Result<NodeXgid, AdminError> {
    let invalid = |msg: String| AdminError::new("BOOT_7102", Stage::Validate, msg);
    let bytes = encoding::decode(pubkey)
        .map_err(|e| invalid(format!("--pubkey is not valid base64url: {e}")))?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| invalid(format!("--pubkey decodes to {} bytes, expected 32", bytes.len())))?;
    let vk = VerifyingKey::from_bytes(&arr)
        .map_err(|e| invalid(format!("--pubkey is not a valid Ed25519 key: {e}")))?;
    Ok(NodeXgid::from_pubkey(&vk))
}

/// Validate a repeated `--tier` set (1..=4) into `Vec<u8>` (`BOOT_7103` on an
/// out-of-range value). Tiers are stored as raw `u8` in `BootstrapSelfInfo`
/// (local-display only — no wire frame carries them, Checkpoint #1(d)).
fn parse_bootstrap_tiers(tiers: &[u32]) -> Result<Vec<u8>, AdminError> {
    tiers
        .iter()
        .map(|t| {
            if (1..=4).contains(t) {
                Ok(*t as u8)
            } else {
                Err(AdminError::new(
                    "BOOT_7103",
                    Stage::Validate,
                    format!("invalid tier {t} (expected 1..=4)"),
                ))
            }
        })
        .collect()
}

/// Map a send-path `BootstrapClientError` to the admin `BOOT_71xx` block: an ack
/// verification failure (Pin 2) is `BOOT_7111`; any connect/send/recv/timeout is
/// `BOOT_7110`. Both are `Stage::Federate` (the network exchange).
fn map_bootstrap_client_err(e: BootstrapClientError) -> AdminError {
    let msg = e.to_string();
    match e {
        BootstrapClientError::AckVerify(_) => {
            AdminError::new("BOOT_7111", Stage::Federate, format!("bootstrap ack verify failed: {msg}"))
        }
        _ => AdminError::new(
            "BOOT_7110",
            Stage::Federate,
            format!("bootstrap node exchange failed: {msg}"),
        ),
    }
}

// ── bootstrap show — READ (not audited) ───────────────────────────────────────────

/// Args for `bootstrap show` (§6.A3).
#[derive(Debug, Clone, Default, clap::Args)]
pub struct BootstrapShowArgs {}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapRegistrationSummary {
    pub bootstrap_id: String,
    pub url: String,
    pub directory_url: String,
    pub registered_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapSelfInfoView {
    pub endpoint: String,
    pub region: String,
    pub capabilities: Vec<String>,
    pub auth_tiers_served: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapShowResult {
    pub registrations: Vec<BootstrapRegistrationSummary>,
    pub self_info: BootstrapSelfInfoView,
}

/// `bootstrap show` — display the Bootstrap Nodes this Node is registered with +
/// the self-info it advertises. READ → not audited.
pub async fn bootstrap_show(
    ctx: &mut AdminContext<'_>,
) -> Result<BootstrapShowResult, AdminError> {
    let store = Arc::clone(ctx.require_bootstrap_store(Stage::Register)?);
    let s = store.lock().await;
    let registrations = s
        .all()
        .iter()
        .map(|r| BootstrapRegistrationSummary {
            bootstrap_id: r.bootstrap_id.as_str().to_string(),
            url: r.url.clone(),
            directory_url: r.directory_url.clone(),
            registered_at: r.registered_at.clone(),
            expires_at: r.expires_at.clone(),
        })
        .collect();
    let info = s.self_info();
    Ok(BootstrapShowResult {
        registrations,
        self_info: BootstrapSelfInfoView {
            endpoint: info.endpoint.clone(),
            region: info.region.clone(),
            capabilities: info.capabilities.clone(),
            auth_tiers_served: info.auth_tiers_served.clone(),
        },
    })
}

// ── bootstrap register — WRITE (audited) ───────────────────────────────────────────

/// Args for `bootstrap register` (§6.A3). `--url` + `--pubkey` (checkpoint #1(c));
/// the Node's own endpoint/region/capabilities come from the stored self-info,
/// not re-typed (derive-don't-retype discipline).
#[derive(Debug, Clone, clap::Args)]
pub struct BootstrapRegisterArgs {
    /// The Bootstrap Node's connect URL (framed transport target, BC-D3).
    #[arg(long)]
    pub url: String,
    /// The Bootstrap Node's base64url Ed25519 verifying key — `bootstrap_id` is
    /// derived from it (used to verify the `register_ack` signature, Pin 2).
    #[arg(long)]
    pub pubkey: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapRegisterResult {
    pub bootstrap_id: String,
    pub url: String,
    pub directory_url: String,
    pub registered_at: String,
}

/// `bootstrap register --url <u> --pubkey <k>` — register this Node with a
/// Bootstrap Node. Drives the C2 framed send-path (signed `Register` → verified
/// `RegisterAck`), then records the resulting registration. WRITE → A6 trail.
pub async fn bootstrap_register(
    ctx: &mut AdminContext<'_>,
    args: BootstrapRegisterArgs,
) -> Result<BootstrapRegisterResult, AdminError> {
    // Validate `--pubkey` first (BOOT_7102) — before reaching for runtime/network.
    let bootstrap_id = bootstrap_id_from_pubkey(&args.pubkey)?;
    let store = Arc::clone(ctx.require_bootstrap_store(Stage::Register)?);
    let runtime = Arc::clone(ctx.require_runtime(Stage::Register)?);

    // Self-info to advertise (clone under a brief lock — not held across I/O).
    let self_info = {
        let s = store.lock().await;
        s.self_info().clone()
    };
    // The Node keypair + own id come from the live runtime (sibling to
    // `federation_initiate`).
    let (node_keypair, self_node_id) = {
        let rt = runtime.lock().await;
        (rt.node_keypair.clone(), rt.node_id.clone())
    };

    // Framed send-path (no store lock held across the network exchange).
    let registration =
        register_with_bootstrap(&args.url, &bootstrap_id, &self_node_id, &self_info, &node_keypair)
            .await
            .map_err(map_bootstrap_client_err)?;
    let directory_url = registration.directory_url.clone();
    let registered_at = registration.registered_at.clone();

    let path = ctx.bootstrap_store_path();
    {
        let mut s = store.lock().await;
        s.add(registration);
        if let Err(e) = s.save(&path) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("bootstrap store save failed: {e}"),
            ));
        }
    }

    let bootstrap_id_str = bootstrap_id.as_str().to_string();
    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"url\":{:?},\"pubkey\":{:?}}}",
        args.url, args.pubkey
    ));
    record_action(
        &conn,
        ctx,
        "bootstrap register",
        Some(bootstrap_id_str.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;

    Ok(BootstrapRegisterResult {
        bootstrap_id: bootstrap_id_str,
        url: args.url,
        directory_url,
        registered_at,
    })
}

// ── bootstrap deregister — DESTRUCTIVE (audited) ────────────────────────────────────

/// Args for `bootstrap deregister` (§6.A3).
#[derive(Debug, Clone, clap::Args)]
pub struct BootstrapDeregisterArgs {
    /// The `bootstrap_id` URI of the registration to remove.
    pub bootstrap_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapDeregisterResult {
    pub bootstrap_id: String,
}

/// `bootstrap deregister <bootstrap_id>` — explicitly remove this Node from a
/// Bootstrap Node's directory. Sends a signed `Deregister` (fire-and-forget — the
/// protocol defines no ack) then drops the local registration. Unknown id →
/// `BOOT_7101` (before any network). DESTRUCTIVE → A6 trail.
pub async fn bootstrap_deregister(
    ctx: &mut AdminContext<'_>,
    args: BootstrapDeregisterArgs,
) -> Result<BootstrapDeregisterResult, AdminError> {
    let bootstrap_id = bootstrap_node_xgid(&args.bootstrap_id);
    let store = Arc::clone(ctx.require_bootstrap_store(Stage::Register)?);

    // Look up the stored registration first — unknown id is a user error
    // (BOOT_7101), reported before reaching for runtime/network.
    let url = {
        let s = store.lock().await;
        match s.get(&bootstrap_id) {
            Some(r) => r.url.clone(),
            None => {
                return Err(AdminError::new(
                    "BOOT_7101",
                    Stage::Register,
                    format!("no bootstrap registration with id {}", args.bootstrap_id),
                ));
            }
        }
    };

    let runtime = Arc::clone(ctx.require_runtime(Stage::Register)?);
    let (node_keypair, self_node_id) = {
        let rt = runtime.lock().await;
        (rt.node_keypair.clone(), rt.node_id.clone())
    };

    // Send the signed Deregister; only drop the local record once it is on the
    // wire (a send failure → BOOT_7110, registration retained so the operator
    // can retry).
    deregister_from_bootstrap(&url, &self_node_id, &node_keypair)
        .await
        .map_err(map_bootstrap_client_err)?;

    let path = ctx.bootstrap_store_path();
    {
        let mut s = store.lock().await;
        s.remove(&bootstrap_id);
        if let Err(e) = s.save(&path) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("bootstrap store save failed: {e}"),
            ));
        }
    }

    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"bootstrap_id\":{:?}}}",
        args.bootstrap_id
    ));
    record_action(
        &conn,
        ctx,
        "bootstrap deregister",
        Some(args.bootstrap_id.clone()),
        args_hash,
        "ok",
        None,
        None,
    )?;

    Ok(BootstrapDeregisterResult { bootstrap_id: args.bootstrap_id })
}

// ── bootstrap set-info — WRITE (audited) ───────────────────────────────────────────

/// Args for `bootstrap set-info` (§6.A3). Edits the wire-advertised self-info
/// fields (endpoint/region/capabilities — these map to the `Register` frame).
#[derive(Debug, Clone, clap::Args)]
pub struct BootstrapSetInfoArgs {
    /// This Node's advertised endpoint URL.
    #[arg(long)]
    pub endpoint: String,
    /// Operator-declared region.
    #[arg(long)]
    pub region: String,
    /// An advertised `xgen.*` capability token, repeatable.
    #[arg(long = "capability")]
    pub capability: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapSetInfoResult {
    pub endpoint: String,
    pub region: String,
    pub capabilities: Vec<String>,
}

/// `bootstrap set-info --endpoint <u> --region <r> [--capability C]…` — update the
/// advertised self-info. Writes the local store first (A3-D2 — the local write
/// succeeds regardless of any re-advertise). Best-effort re-advertise to
/// registered Bootstrap Nodes is wired in C4. WRITE → A6 trail.
pub async fn bootstrap_set_info(
    ctx: &mut AdminContext<'_>,
    args: BootstrapSetInfoArgs,
) -> Result<BootstrapSetInfoResult, AdminError> {
    let store = Arc::clone(ctx.require_bootstrap_store(Stage::Register)?);
    let path = ctx.bootstrap_store_path();
    // Local write first (A3-D2 — succeeds regardless of any re-advertise).
    let new_self_info = {
        let mut s = store.lock().await;
        s.set_info(args.endpoint.clone(), args.region.clone(), args.capability.clone());
        if let Err(e) = s.save(&path) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("bootstrap store save failed: {e}"),
            ));
        }
        s.self_info().clone()
    };

    let conn = open_audit(ctx)?;
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"endpoint\":{:?},\"region\":{:?},\"capability\":{:?}}}",
        args.endpoint, args.region, args.capability
    ));
    record_action(
        &conn,
        ctx,
        "bootstrap set-info",
        None,
        args_hash,
        "ok",
        None,
        None,
    )?;

    // A3-D2 best-effort re-advertise — push the updated self-info to every
    // registered Bootstrap Node (re-register; only `Register` carries
    // endpoint/region/capabilities). The local write already succeeded; a
    // fan-out failure is logged, never fails the verb. Requires the live runtime
    // (Node keypair + id); skipped when absent (store-only unit tests) — the
    // local write stands. `set-tiers` has NO re-advertise (Checkpoint #1(d)).
    if let Some(runtime) = &ctx.runtime {
        let (node_keypair, self_node_id) = {
            let rt = runtime.lock().await;
            (rt.node_keypair.clone(), rt.node_id.clone())
        };
        crate::bootstrap_keepalive::readvertise_all(
            Arc::clone(&store),
            path,
            &node_keypair,
            &self_node_id,
            &new_self_info,
        )
        .await;
    }

    Ok(BootstrapSetInfoResult {
        endpoint: args.endpoint,
        region: args.region,
        capabilities: args.capability,
    })
}

// ── bootstrap set-tiers — WRITE (audited) — Checkpoint #1(d) Option A ───────────────

/// Args for `bootstrap set-tiers` (§6.A3).
#[derive(Debug, Clone, clap::Args)]
pub struct BootstrapSetTiersArgs {
    /// An advertised Auth Tier served (1..=4), repeatable.
    #[arg(long = "tier")]
    pub tier: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapSetTiersResult {
    pub auth_tiers_served: Vec<u8>,
}

/// `bootstrap set-tiers [--tier N]…` — set the advertised Auth Tiers. **Local
/// self-info only** (Checkpoint #1(d), Option A): no `Register`/`Keepalive` wire
/// frame carries tiers, so there is NO re-advertise — `show` displays the stored
/// value. Propagating tiers on the wire (Option B) is a wire-format change
/// deferred to a separate protocol-design arc. Bad tier → `BOOT_7103`. WRITE → A6.
pub async fn bootstrap_set_tiers(
    ctx: &mut AdminContext<'_>,
    args: BootstrapSetTiersArgs,
) -> Result<BootstrapSetTiersResult, AdminError> {
    let tiers = parse_bootstrap_tiers(&args.tier)?;
    let store = Arc::clone(ctx.require_bootstrap_store(Stage::Register)?);
    let path = ctx.bootstrap_store_path();
    {
        let mut s = store.lock().await;
        s.set_tiers(tiers.clone());
        if let Err(e) = s.save(&path) {
            return Err(AdminError::generic(
                Stage::Persist,
                format!("bootstrap store save failed: {e}"),
            ));
        }
    }

    let conn = open_audit(ctx)?;
    let args_hash =
        AuditEntry::compute_args_hash(&format!("{{\"tier\":{:?}}}", args.tier));
    record_action(
        &conn,
        ctx,
        "bootstrap set-tiers",
        None,
        args_hash,
        "ok",
        None,
        None,
    )?;

    Ok(BootstrapSetTiersResult { auth_tiers_served: tiers })
}

#[cfg(test)]
mod bootstrap_verb_tests {
    use super::*;
    use tempfile::tempdir;

    fn store_ctx<'a>(
        data_dir: &'a Path,
        config_path: &'a Path,
        store: &Arc<Mutex<BootstrapRegistrationStore>>,
    ) -> AdminContext<'a> {
        AdminContext::batch(data_dir, config_path, "os-user:test")
            .with_bootstrap_store(Arc::clone(store))
    }

    /// PRIME-INVARIANT REGRESSION (C3, D-065): no `[bootstrap]` config + an empty
    /// store = registered with nobody = today byte-for-byte. `show` on an empty
    /// store returns no registrations + default self-info, and touches no network.
    #[tokio::test]
    async fn empty_store_registers_with_nobody() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let store = Arc::new(Mutex::new(BootstrapRegistrationStore::new()));
        let mut ctx = store_ctx(dir.path(), &cfg, &store);

        let r = bootstrap_show(&mut ctx).await.expect("show on empty store");
        assert!(r.registrations.is_empty());
        assert!(r.self_info.endpoint.is_empty());
        assert!(r.self_info.region.is_empty());
        assert!(r.self_info.capabilities.is_empty());
        assert!(r.self_info.auth_tiers_served.is_empty());
    }

    #[tokio::test]
    async fn set_info_then_show_reflects_it_and_persists() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let store = Arc::new(Mutex::new(BootstrapRegistrationStore::new()));
        let mut ctx = store_ctx(dir.path(), &cfg, &store);

        bootstrap_set_info(
            &mut ctx,
            BootstrapSetInfoArgs {
                endpoint: "wss://self.example.com/xgen".to_string(),
                region: "EU".to_string(),
                capability: vec!["xgen.federation".to_string()],
            },
        )
        .await
        .expect("set-info");

        let r = bootstrap_show(&mut ctx).await.expect("show");
        assert_eq!(r.self_info.endpoint, "wss://self.example.com/xgen");
        assert_eq!(r.self_info.region, "EU");
        assert_eq!(r.self_info.capabilities, vec!["xgen.federation".to_string()]);

        // Persisted to the combined store file.
        let loaded = BootstrapRegistrationStore::load(&ctx.bootstrap_store_path()).unwrap();
        assert_eq!(loaded.self_info().endpoint, "wss://self.example.com/xgen");
    }

    #[tokio::test]
    async fn set_tiers_then_show_and_bad_tier_rejected() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let store = Arc::new(Mutex::new(BootstrapRegistrationStore::new()));
        let mut ctx = store_ctx(dir.path(), &cfg, &store);

        bootstrap_set_tiers(&mut ctx, BootstrapSetTiersArgs { tier: vec![2, 3] })
            .await
            .expect("set-tiers");
        let r = bootstrap_show(&mut ctx).await.expect("show");
        assert_eq!(r.self_info.auth_tiers_served, vec![2, 3]);

        let err = bootstrap_set_tiers(&mut ctx, BootstrapSetTiersArgs { tier: vec![5] })
            .await
            .expect_err("tier 5 is out of range");
        assert_eq!(err.code, "BOOT_7103");
    }

    #[tokio::test]
    async fn register_rejects_invalid_pubkey() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let store = Arc::new(Mutex::new(BootstrapRegistrationStore::new()));
        let mut ctx = store_ctx(dir.path(), &cfg, &store);

        let err = bootstrap_register(
            &mut ctx,
            BootstrapRegisterArgs {
                url: "wss://bootstrap.example.com/xgen".to_string(),
                pubkey: "not-a-valid-key".to_string(),
            },
        )
        .await
        .expect_err("invalid pubkey rejected before any network");
        assert_eq!(err.code, "BOOT_7102");
    }

    #[tokio::test]
    async fn deregister_unknown_id_is_rejected() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let store = Arc::new(Mutex::new(BootstrapRegistrationStore::new()));
        let mut ctx = store_ctx(dir.path(), &cfg, &store);

        let err = bootstrap_deregister(
            &mut ctx,
            BootstrapDeregisterArgs { bootstrap_id: "xgen://pubkey/ed25519:unknown".to_string() },
        )
        .await
        .expect_err("unknown id rejected before any network");
        assert_eq!(err.code, "BOOT_7101");
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// A4 — Space & Room admin (M6 Phase 9 read subset; design §6.A4, Appendix K.2.6)
// ════════════════════════════════════════════════════════════════════════════════
// HONEST SUBSET (J-156 backing audit). Of A4's 5 verbs only `list-hosted` is
// backed: it reads the live hosted-Space state. `audit-events` reads the §3.11.8
// protocol audit log, which is UNIMPLEMENTED (only the `event_trace` debug-
// tracing layer exists — no structured, queryable, rotating protocol-audit store)
// → deferred to a protocol-audit-log subsystem arc. `force-eject` is design-gated
// (A4-D1 `membership.node_eject` wire sub-design). `set-node-policy`/
// `show-node-policy` need an absent `NodePolicy` store → node-policy arc.

// ── space list-hosted — READ (not audited) ───────────────────────────────────────

/// Args for `space list-hosted` (§6.A4).
#[derive(Debug, Clone, Default, clap::Args)]
pub struct SpaceListHostedArgs {
    /// Optional case-insensitive substring filter on the Space name.
    #[arg(long = "name-filter")]
    pub name_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostedSpaceSummary {
    pub space_id: String,
    pub name: Option<String>,
    pub member_count: usize,
    pub room_count: usize,
    pub federated_peers: usize,
    /// v1: the Node does not persist a per-Space creation timestamp for
    /// originated Spaces, so this is `None` (honest, D-065).
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpaceListHostedResult {
    pub spaces: Vec<HostedSpaceSummary>,
}

/// `space list-hosted` — list the Spaces this Node hosts (originates / homes),
/// i.e. `home_node == this Node` (D-082 lock #4 — never federated-in replicas).
/// Reads the live runtime; READ, not audited.
pub async fn space_list_hosted(
    ctx: &mut AdminContext<'_>,
    args: SpaceListHostedArgs,
) -> Result<SpaceListHostedResult, AdminError> {
    let runtime = Arc::clone(ctx.require_runtime(Stage::Register)?);
    let rt = runtime.lock().await;
    let me = rt.node_id.as_str().to_string();
    let filter = args.name_filter.as_deref().map(str::to_lowercase);
    let mut spaces: Vec<HostedSpaceSummary> = rt
        .spaces
        .values()
        .filter(|s| s.home_node.as_str() == me)
        .filter(|s| match &filter {
            Some(f) => s
                .name
                .as_deref()
                .map(|n| n.to_lowercase().contains(f.as_str()))
                .unwrap_or(false),
            None => true,
        })
        .map(|s| HostedSpaceSummary {
            space_id: s.space_id.as_str().to_string(),
            name: s.name.clone(),
            member_count: s.members.len(),
            room_count: s.rooms.len(),
            federated_peers: s.federation_nodes.len(),
            created_at: None,
        })
        .collect();
    spaces.sort_by(|a, b| a.space_id.cmp(&b.space_id));
    Ok(SpaceListHostedResult { spaces })
}

// ── space audit-events — READ (not audited; protocol-audit-log arc, A4-D3) ────────
//
// Reads the §3.11.8 protocol audit log (the JSONL store written by the Commit 1
// writer in `crate::protocol_audit`) filtered to one Space. PAL-D1 read-time
// scope: the store is Node-global (one monthly file covering every hosted/
// federated Space), so the per-Space filter happens here at read time. READ →
// not audited (A4-D3). This is NOT the A6 SQLite admin trail (`audit query`).

/// Args for `space audit-events` (§6.A4, A4-D3).
#[derive(Debug, Clone, Default, clap::Args)]
pub struct SpaceAuditEventsArgs {
    /// The Space whose protocol-audit entries to read (must be hosted/federated here).
    pub space_id: String,
    /// Optional exact EventType filter, e.g. `membership.join`.
    #[arg(long = "event-type")]
    pub event_type: Option<String>,
    /// Optional inclusive lower bound on entry `ts` (RFC 3339 UTC).
    #[arg(long)]
    pub since: Option<String>,
    /// Optional inclusive upper bound on entry `ts` (RFC 3339 UTC).
    #[arg(long)]
    pub until: Option<String>,
    /// Max entries to return (default 100, capped at 1000).
    #[arg(long)]
    pub limit: Option<usize>,
    /// Opaque pagination cursor returned as `next_cursor` by a prior call.
    #[arg(long)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpaceAuditEventsResult {
    pub events: Vec<crate::protocol_audit::ProtocolAuditEntry>,
    pub returned: usize,
    /// Set when more matching entries remain beyond this page; pass back as `cursor`.
    pub next_cursor: Option<String>,
}

const AUDIT_EVENTS_DEFAULT_LIMIT: usize = 100;
const AUDIT_EVENTS_MAX_LIMIT: usize = 1000;

/// `space audit-events` — read-time-filtered view of the Node protocol audit log
/// for one Space. READ, not audited (A4-D3).
pub async fn space_audit_events(
    ctx: &mut AdminContext<'_>,
    args: SpaceAuditEventsArgs,
) -> Result<SpaceAuditEventsResult, AdminError> {
    // Filter validation → SPACE_8010.
    let validate_ts = |label: &str, v: &Option<String>| -> Result<(), AdminError> {
        if let Some(s) = v {
            if chrono::DateTime::parse_from_rfc3339(s).is_err() {
                return Err(AdminError::new(
                    "SPACE_8010",
                    Stage::Validate,
                    format!("invalid {label} filter (expected RFC 3339): {s}"),
                ));
            }
        }
        Ok(())
    };
    validate_ts("since", &args.since)?;
    validate_ts("until", &args.until)?;
    let offset = match &args.cursor {
        Some(c) => c.parse::<usize>().map_err(|_| {
            AdminError::new(
                "SPACE_8010",
                Stage::Validate,
                format!("invalid cursor: {c}"),
            )
        })?,
        None => 0,
    };
    let limit = args
        .limit
        .unwrap_or(AUDIT_EVENTS_DEFAULT_LIMIT)
        .clamp(1, AUDIT_EVENTS_MAX_LIMIT);

    // SPACE_8001 — the Space must be hosted-by OR federated-to this Node (its
    // events would otherwise never reach this Node's audit log). Both hosted and
    // federated-in Spaces live in `runtime.spaces`.
    {
        let runtime = Arc::clone(ctx.require_runtime(Stage::Validate)?);
        let rt = runtime.lock().await;
        let space_xgid = SpaceXgid::from_xgid(Xgid::new(args.space_id.clone()));
        if !rt.spaces.contains_key(&space_xgid) {
            return Err(AdminError::new(
                "SPACE_8001",
                Stage::Validate,
                format!("Space not hosted or federated on this Node: {}", args.space_id),
            ));
        }
    } // drop the runtime lock before file I/O

    // Scan the month files covering [since, until] (all present files when
    // unbounded), then filter precisely on space_id + event_type + per-entry ts.
    let audit_dir = ctx.data_dir.join("audit");
    let since_month = args.since.as_deref().and_then(|s| s.get(0..7));
    let until_month = args.until.as_deref().and_then(|s| s.get(0..7));
    let all = crate::protocol_audit::read_all_entries(&audit_dir, since_month, until_month);

    let matched: Vec<crate::protocol_audit::ProtocolAuditEntry> = all
        .into_iter()
        .filter(|e| {
            e.extra
                .get("space_id")
                .and_then(|v| v.as_str())
                == Some(args.space_id.as_str())
                && args
                    .event_type
                    .as_deref()
                    .map(|t| e.event_type == t)
                    .unwrap_or(true)
                && args
                    .since
                    .as_deref()
                    .map(|s| e.ts.as_str() >= s)
                    .unwrap_or(true)
                && args
                    .until
                    .as_deref()
                    .map(|u| e.ts.as_str() <= u)
                    .unwrap_or(true)
        })
        .collect();

    let total = matched.len();
    let events: Vec<crate::protocol_audit::ProtocolAuditEntry> =
        matched.into_iter().skip(offset).take(limit).collect();
    let returned = events.len();
    let next_cursor = if offset + returned < total {
        Some((offset + returned).to_string())
    } else {
        None
    };
    Ok(SpaceAuditEventsResult {
        events,
        returned,
        next_cursor,
    })
}

// ── space audit-rebuild — WRITE (audited in A6 trail; PAL-D3) ─────────────────────
//
// Replay each in-scope Space's persisted DAG events and append protocol-audit
// entries for the audited types whose event_id is not already logged. Closes
// PAL-D2 gaps (a write that failed loudly) AND backfills cold-start Spaces (events
// predating the writer). Idempotent — dedup by event_id against the existing log,
// so a second run adds 0. Operator-invoked only; v1 has NO startup/automatic
// reconcile (PAL-D3). WRITE → recorded in the A6 admin trail (the rebuild *action*).

/// Args for `space audit-rebuild` (§6.A4, A4-D3 / PAL-D3).
#[derive(Debug, Clone, Default, clap::Args)]
pub struct SpaceAuditRebuildArgs {
    /// The Space to rebuild (must be hosted/federated here). Omit to rebuild ALL
    /// hosted/federated Spaces.
    pub space_id: Option<String>,
    /// Report what would be added without writing anything.
    #[arg(long = "dry-run")]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpaceAuditRebuildResult {
    pub spaces_scanned: usize,
    pub entries_added: usize,
    pub entries_already_present: usize,
}

/// `space audit-rebuild` — regenerate missing protocol-audit entries from the DAG
/// (PAL-D3). WRITE → audited (A6 trail), unless `--dry-run` (preview, not audited).
pub async fn space_audit_rebuild(
    ctx: &mut AdminContext<'_>,
    args: SpaceAuditRebuildArgs,
) -> Result<SpaceAuditRebuildResult, AdminError> {
    // Collect the in-scope Space ids + this Node's id under the runtime lock,
    // then drop it before any file I/O.
    let (in_scope, node_id): (Vec<String>, String) = {
        let runtime = Arc::clone(ctx.require_runtime(Stage::Validate)?);
        let rt = runtime.lock().await;
        let node_id = rt.node_id.as_str().to_string();
        let scope = match &args.space_id {
            Some(sid) => {
                let sx = SpaceXgid::from_xgid(Xgid::new(sid.clone()));
                if !rt.spaces.contains_key(&sx) {
                    return Err(AdminError::new(
                        "SPACE_8001",
                        Stage::Validate,
                        format!("Space not hosted or federated on this Node: {sid}"),
                    ));
                }
                vec![sid.clone()]
            }
            None => rt.spaces.values().map(|s| s.space_id.as_str().to_string()).collect(),
        };
        (scope, node_id)
    };

    let spaces_dir = crate::app::resolve_spaces_dir(ctx.config_path, ctx.data_dir);
    let audit_dir = ctx.data_dir.join("audit");
    // Dedup set: every event_id already present in the log (all months).
    let mut existing: std::collections::HashSet<String> =
        crate::protocol_audit::read_all_entries(&audit_dir, None, None)
            .into_iter()
            .map(|e| e.event_id)
            .collect();
    let sink = crate::protocol_audit::ProtocolAuditSink::new(audit_dir, node_id.clone());

    let mut spaces_scanned = 0usize;
    let mut entries_added = 0usize;
    let mut entries_already_present = 0usize;
    for sid in &in_scope {
        spaces_scanned += 1;
        for event in crate::app::read_persisted_events(&spaces_dir, sid) {
            let entry = match crate::protocol_audit::ProtocolAuditEntry::from_event(&event, &node_id)
            {
                Some(e) if !e.event_id.is_empty() => e,
                // Non-audited type, or an event without an event_id (untrackable).
                _ => continue,
            };
            if existing.contains(&entry.event_id) {
                entries_already_present += 1;
                continue;
            }
            if !args.dry_run {
                sink.append_entry(&entry).map_err(|e| {
                    AdminError::generic(Stage::Persist, format!("audit rebuild write failed: {e}"))
                })?;
            }
            existing.insert(entry.event_id.clone()); // avoid double-count within this run
            entries_added += 1;
        }
    }

    // WRITE → A6 trail (the rebuild action). A dry-run writes nothing → preview,
    // not audited.
    if !args.dry_run {
        let conn = open_audit(ctx)?;
        let args_hash = AuditEntry::compute_args_hash(&format!(
            "{{\"space_id\":{:?},\"dry_run\":{:?}}}",
            args.space_id, args.dry_run
        ));
        record_action(
            &conn,
            ctx,
            "space audit-rebuild",
            args.space_id.clone(),
            args_hash,
            "ok",
            None,
            None,
        )?;
    }

    Ok(SpaceAuditRebuildResult {
        spaces_scanned,
        entries_added,
        entries_already_present,
    })
}

// ════════════════════════════════════════════════════════════════════════════════
// A7 — Plugin management (M6 Phase 10; design §6.A7, Appendix K.2.7)
// ════════════════════════════════════════════════════════════════════════════════
// A7-D1: M6 ships the 2 READ verbs only; WRITE verbs (load/configure/unload) are
// deferred until a 2nd plugin exists. Backing is the honest static compiled-in
// registry (`crate::plugins::installed_plugins`) — no dynamic loader, no per-
// plugin telemetry in M6, so `events_consumed` / `last_activity` are `None`
// (honest, D-065). Both verbs are pure reads of a compile-time fact (no live
// runtime needed); not audited.

// ── plugin list — READ (not audited) ─────────────────────────────────────────────

/// Args for `plugin list` (§6.A7) — none.
#[derive(Debug, Clone, Default, clap::Args)]
pub struct PluginListArgs {}

#[derive(Debug, Clone, Serialize)]
pub struct PluginListResult {
    pub plugins: Vec<PluginInfo>,
}

/// `plugin list` — enumerate the plugins compiled into this Node.
pub async fn plugin_list(
    _ctx: &mut AdminContext<'_>,
    _args: PluginListArgs,
) -> Result<PluginListResult, AdminError> {
    Ok(PluginListResult {
        plugins: crate::plugins::installed_plugins(),
    })
}

// ── plugin status — READ (not audited) ───────────────────────────────────────────

/// Args for `plugin status` (§6.A7).
#[derive(Debug, Clone, clap::Args)]
pub struct PluginStatusArgs {
    /// Plugin name (as reported by `plugin list`).
    pub plugin_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginStatusResult {
    pub name: String,
    pub version: String,
    pub status: String,
    pub kind: String,
    /// No per-plugin telemetry in M6 (A7-D1) → always `None` (honest).
    pub events_consumed: Option<u64>,
    pub last_activity: Option<String>,
}

/// `plugin status` — detail for one compiled-in plugin; `PLUGIN_9001` if unknown.
pub async fn plugin_status(
    _ctx: &mut AdminContext<'_>,
    args: PluginStatusArgs,
) -> Result<PluginStatusResult, AdminError> {
    match crate::plugins::installed_plugins()
        .into_iter()
        .find(|p| p.name == args.plugin_name)
    {
        Some(p) => Ok(PluginStatusResult {
            name: p.name,
            version: p.version,
            status: p.status,
            kind: p.kind,
            events_consumed: None,
            last_activity: None,
        }),
        None => Err(AdminError::new(
            "PLUGIN_9001",
            Stage::Register,
            format!("unknown plugin: {}", args.plugin_name),
        )),
    }
}

// ── space force-eject / unban — DESTRUCTIVE (audited) — A4-D1 ─────────────────────
//
// Node-administrator force-eject + its reversible counterpart. The Node *authors*
// a Space-DAG event (`membership.node_eject` / `node_unban`) signed by its own
// keypair, dispatched through the live runtime (LocallySubmitted), and persisted.
// Option A (J-159 lock): propagation is via the existing sync path (the event is
// in the DAG/store), not a live push — connected clients / federated peers pick
// it up on next sync (honest, D-065; sibling to A1 `defederate`'s no-network-
// goodbye). The live resident's in-memory state updates immediately (target
// removed + banned), so the auth gate enforces the eject at once.

/// Build a Node-authored Space membership event (eject / unban), sign it with the
/// home Node keypair, dispatch it (LocallySubmitted), and persist it + any
/// drain-derived events to disk. Returns the emitted event_id. Dispatch/persist
/// failures map to `SPACE_8004`.
async fn emit_node_membership_event(
    ctx: &mut AdminContext<'_>,
    space_id: &str,
    event_type: EventType,
    content: serde_json::Value,
) -> Result<String, AdminError> {
    let runtime = Arc::clone(ctx.require_runtime(Stage::Persist)?);
    let spaces_dir = crate::app::resolve_spaces_dir(ctx.config_path, ctx.data_dir);
    let space_xgid = SpaceXgid::from_xgid(Xgid::new(space_id.to_string()));

    let (event, additional, node_id): (
        xgen_common::wire::Event,
        Vec<xgen_common::wire::Event>,
        NodeXgid,
    ) = {
        let mut rt = runtime.lock().await;
        let node_kp = rt.node_keypair.clone();
        let node_id = rt.node_id.clone();
        // Current Space tips → prev_events (node_eject/unban are non-root events).
        let tips: Vec<EventXgid> = rt
            .graphs
            .get(&space_xgid)
            .map(|g| {
                g.current_tips()
                    .into_iter()
                    .map(|s| EventXgid::from_xgid(Xgid::new(s)))
                    .collect()
            })
            .unwrap_or_default();
        let mut ev = build_membership_event(&node_kp, space_id, "", event_type, content);
        ev.prev_events = tips;
        let ev = sign_event(ev, &node_kp);
        match rt.dispatch_event(ev.clone(), EventOrigin::LocallySubmitted, None) {
            DispatchOutcome::Accepted { additional_persisted, .. } => {
                (ev, additional_persisted, node_id)
            }
            DispatchOutcome::HeldPending => {
                return Err(AdminError::new(
                    "SPACE_8004",
                    Stage::Persist,
                    "node-authored membership event held pending (unexpected)".to_string(),
                ));
            }
            DispatchOutcome::Rejected(why) => {
                return Err(AdminError::new(
                    "SPACE_8004",
                    Stage::Persist,
                    format!("node-authored membership event rejected: {why}"),
                ));
            }
        }
    };
    crate::app::persist_event(&spaces_dir, space_id, &event);
    for ev in &additional {
        let sid = if ev.space_id.as_str().is_empty() {
            ev.event_id.as_ref().map(|e| e.as_str()).unwrap_or("")
        } else {
            ev.space_id.as_str()
        };
        crate::app::persist_event(&spaces_dir, sid, ev);
    }

    // Option B (J-160): live fan-out after persist — push the accepted event to
    // the Space's connected member clients and to its federated peers right now,
    // mirroring the client-submission path (`process_inbound` →
    // `apply_fanout` + `apply_federation_push`, app.rs). Best-effort after
    // persist (D-070 honesty): a fan-out/push failure does NOT roll back the
    // eject — the event is already in the DAG + on disk; sync remains the
    // backstop. Sync-only (the Option-A baseline) when the sender maps aren't
    // wired (file-only verbs / unit tests).
    //
    // The event is Node-authored, so the Node is not a client recipient. We pass
    // the Node's id projected to `IdentityXgid` as `apply_fanout`'s `author_id`;
    // it is used only to *exclude* the author, and the Node is never in
    // `ClientSenders`, so every other connected member receives the event.
    //
    // Recipient nuance (honest, D-065): `apply_fanout` collects recipients from
    // the Space's *current* members, and `dispatch_event` above already removed
    // the target (node_eject removes + bans). So the ejected target's own
    // session is NOT in the live push — it learns of the eject via sync, exactly
    // as it would for a member-initiated kick (whose recipient set is likewise
    // post-removal). The remaining members + federated peers get it live.
    if let Some(client_senders) = ctx.client_senders.as_ref() {
        let author_id = IdentityXgid::from_xgid(Xgid::new(node_id.as_str().to_string()));
        let req = crate::fanout::FanoutRequest {
            event: Some(event.clone()),
            new_joiner: None,
        };
        crate::fanout::apply_fanout(req, &author_id, &runtime, client_senders).await;
    }
    if let Some(federation_peer_senders) = ctx.federation_peer_senders.as_ref() {
        crate::federation_session::apply_federation_push(
            &event,
            EventOrigin::LocallySubmitted,
            &runtime,
            federation_peer_senders,
            &node_id,
            // Node-authored force-eject/unban override — push unconditionally
            // (operator admin authority; not subject to per-peer federation
            // policy). 2b FAC-D3 enforcement applies to peer-originated events.
            None,
        )
        .await;
    }

    Ok(event
        .event_id
        .as_ref()
        .map(|e| e.as_str().to_string())
        .unwrap_or_default())
}

/// Write the heavy A4 audit entry, recording the emitted `event_id` in
/// `correlation_id` (design §6.A4). Distinct from `record_action` (which carries
/// no correlation id).
fn record_action_correlated(
    ctx: &AdminContext<'_>,
    verb: &str,
    target: String,
    args_hash: String,
    correlation_id: String,
    timestamp: String,
) -> Result<(), AdminError> {
    let conn = open_audit(ctx)?;
    let entry = AuditEntry {
        timestamp,
        verb: verb.to_string(),
        actor: ctx.actor.clone(),
        actor_via: ctx.actor_via.as_str().to_string(),
        target: Some(target),
        args_hash,
        outcome: "ok".to_string(),
        error_code: None,
        error_message: None,
        correlation_id: Some(correlation_id),
        meta_atts: "{}".to_string(),
    };
    audit::insert_entry(&conn, &entry).map_err(|e| {
        AdminError::new("AUDIT_5001", Stage::Persist, format!("audit write failed: {e}"))
    })
}

/// Args for `space force-eject` (§6.A4).
#[derive(Debug, Clone, clap::Args)]
pub struct SpaceForceEjectArgs {
    pub space_id: String,
    pub identity_id: String,
    #[arg(long)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpaceForceEjectResult {
    pub space_id: String,
    pub identity_id: String,
    pub ejected_at: String,
    pub event_id: String,
}

/// `space force-eject` — Node-administrator removal + ban (A4-D1, 1A). Emits a
/// Node-signed `membership.node_eject`. DESTRUCTIVE → audited (correlation_id =
/// event_id).
pub async fn space_force_eject(
    ctx: &mut AdminContext<'_>,
    args: SpaceForceEjectArgs,
) -> Result<SpaceForceEjectResult, AdminError> {
    // Pre-checks: Space hosted here, target is a member.
    {
        let runtime = Arc::clone(ctx.require_runtime(Stage::Validate)?);
        let rt = runtime.lock().await;
        let space_xgid = SpaceXgid::from_xgid(Xgid::new(args.space_id.clone()));
        let space = rt.spaces.get(&space_xgid).ok_or_else(|| {
            AdminError::new(
                "SPACE_8001",
                Stage::Validate,
                format!("Space not hosted on this Node: {}", args.space_id),
            )
        })?;
        if space.home_node.as_str() != rt.node_id.as_str() {
            return Err(AdminError::new(
                "SPACE_8001",
                Stage::Validate,
                format!("Space not hosted on this Node: {}", args.space_id),
            ));
        }
        if !space.is_member(args.identity_id.as_str()) {
            let target = IdentityXgid::from_xgid(Xgid::new(args.identity_id.clone()));
            if space.banned.contains(&target) {
                return Err(AdminError::new(
                    "SPACE_8003",
                    Stage::Validate,
                    format!("identity already removed / banned: {}", args.identity_id),
                ));
            }
            return Err(AdminError::new(
                "SPACE_8002",
                Stage::Validate,
                format!("identity is not a member of the Space: {}", args.identity_id),
            ));
        }
    }

    let mut content = serde_json::json!({ "target_identity": args.identity_id });
    if let Some(r) = &args.reason {
        content["reason"] = serde_json::Value::String(r.clone());
    }
    let event_id =
        emit_node_membership_event(ctx, &args.space_id, EventType::MembershipNodeEject, content)
            .await?;
    let ejected_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"space_id\":{:?},\"identity_id\":{:?},\"reason\":{:?}}}",
        args.space_id, args.identity_id, args.reason
    ));
    record_action_correlated(
        ctx,
        "space force-eject",
        format!("{}:{}", args.space_id, args.identity_id),
        args_hash,
        event_id.clone(),
        ejected_at.clone(),
    )?;
    Ok(SpaceForceEjectResult {
        space_id: args.space_id,
        identity_id: args.identity_id,
        ejected_at,
        event_id,
    })
}

/// Args for `space unban` (§6.A4, A4-D1 1A reversibility).
#[derive(Debug, Clone, clap::Args)]
pub struct SpaceUnbanArgs {
    pub space_id: String,
    pub identity_id: String,
    #[arg(long)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpaceUnbanResult {
    pub space_id: String,
    pub identity_id: String,
    pub unbanned_at: String,
    pub event_id: String,
}

/// `space unban` — lift a Node-eject ban (A4-D1, 1A). Emits a Node-signed
/// `membership.node_unban`. DESTRUCTIVE → audited (correlation_id = event_id).
pub async fn space_unban(
    ctx: &mut AdminContext<'_>,
    args: SpaceUnbanArgs,
) -> Result<SpaceUnbanResult, AdminError> {
    {
        let runtime = Arc::clone(ctx.require_runtime(Stage::Validate)?);
        let rt = runtime.lock().await;
        let space_xgid = SpaceXgid::from_xgid(Xgid::new(args.space_id.clone()));
        let space = rt.spaces.get(&space_xgid).ok_or_else(|| {
            AdminError::new(
                "SPACE_8001",
                Stage::Validate,
                format!("Space not hosted on this Node: {}", args.space_id),
            )
        })?;
        if space.home_node.as_str() != rt.node_id.as_str() {
            return Err(AdminError::new(
                "SPACE_8001",
                Stage::Validate,
                format!("Space not hosted on this Node: {}", args.space_id),
            ));
        }
        let target = IdentityXgid::from_xgid(Xgid::new(args.identity_id.clone()));
        if !space.banned.contains(&target) {
            return Err(AdminError::new(
                "SPACE_8003",
                Stage::Validate,
                format!("identity is not banned: {}", args.identity_id),
            ));
        }
    }

    let mut content = serde_json::json!({ "target_identity": args.identity_id });
    if let Some(r) = &args.reason {
        content["reason"] = serde_json::Value::String(r.clone());
    }
    let event_id =
        emit_node_membership_event(ctx, &args.space_id, EventType::MembershipNodeUnban, content)
            .await?;
    let unbanned_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let args_hash = AuditEntry::compute_args_hash(&format!(
        "{{\"space_id\":{:?},\"identity_id\":{:?},\"reason\":{:?}}}",
        args.space_id, args.identity_id, args.reason
    ));
    record_action_correlated(
        ctx,
        "space unban",
        format!("{}:{}", args.space_id, args.identity_id),
        args_hash,
        event_id.clone(),
        unbanned_at.clone(),
    )?;
    Ok(SpaceUnbanResult {
        space_id: args.space_id,
        identity_id: args.identity_id,
        unbanned_at,
        event_id,
    })
}

// ════════════════════════════════════════════════════════════════════════════════
// Admin verb command grouping (clap) — the two-token verb surface (§2.6.6).
// Shared by the `--batch` pipe dispatcher and the future `--aicontrol` dispatcher
// (M7); both parse tokens into this and call the same `admin_ops::*` verbs.
// Grows by one Subcommand variant per category as Phases 5–10 land.
// ════════════════════════════════════════════════════════════════════════════════

/// Top-level admin command parsed from the pipe token stream. `no_binary_name`
/// because the tokens are the verb path itself (e.g. `["audit", "query", ...]`),
/// not a process argv.
#[derive(Debug, clap::Parser)]
#[command(no_binary_name = true, disable_help_flag = true)]
pub struct AdminCli {
    #[command(subcommand)]
    pub command: AdminCommand,
}

/// One variant per verb category. Phase 4 ships `audit` and `log` (A6).
#[derive(Debug, clap::Subcommand)]
pub enum AdminCommand {
    /// `audit *` — admin audit trail (§6.A6).
    #[command(subcommand)]
    Audit(AuditCommand),
    /// `log *` — runtime tracing level control (§6.A6).
    #[command(subcommand)]
    Log(LogCommand),
    /// `identity *` — Identity registry administration (§6.A5).
    /// (`identity list` stays in the M2 read-only allowlist.)
    #[command(subcommand)]
    Identity(IdentityCommand),
    /// `federation *` — federation relationship management (§6.A1).
    /// M6 honest-subset: `list` + `defederate` only (J-156).
    #[command(subcommand)]
    Federation(FederationCommand),
    /// `space *` — Space/Room admin (§6.A4). M6 honest-subset: `list-hosted`
    /// only (J-156); `audit-events`/`force-eject`/node-policy deferred.
    #[command(subcommand)]
    Space(SpaceCommand),
    /// `plugin *` — plugin management (§6.A7). M6 ships the 2 reads (A7-D1).
    #[command(subcommand)]
    Plugin(PluginCommand),
    /// `auth-module *` — Auth Module registry administration (§6.A2).
    /// auth-module-registry arc: `list`/`register`/`revoke`/`set-tiers` (C3);
    /// `test` lands at C4.
    #[command(subcommand, name = "auth-module")]
    AuthModule(AuthModuleCommand),
    /// `bootstrap *` — bootstrap-client administration (§6.A3).
    /// bootstrap-client arc: `show`/`register`/`deregister`/`set-info`/`set-tiers`.
    #[command(subcommand)]
    Bootstrap(BootstrapCommand),
}

/// `audit` sub-verbs (A6).
#[derive(Debug, clap::Subcommand)]
pub enum AuditCommand {
    /// `audit query` — filtered read of the admin trail.
    Query(AuditQueryArgs),
    /// `audit export` — materialise a filtered JSONL slice.
    Export(AuditExportArgs),
    /// `audit archive` — export rows older than a cutoff, then prune.
    Archive(AuditArchiveArgs),
}

/// `log` sub-verbs (A6). Variant names derive to `set-level` / `show-level`.
#[derive(Debug, clap::Subcommand)]
pub enum LogCommand {
    /// `log set-level` — apply a tracing level at runtime.
    SetLevel(LogSetLevelArgs),
    /// `log show-level` — report effective tracing levels.
    ShowLevel(LogShowLevelArgs),
}

/// `identity` sub-verbs (A5). Variant names derive to `show` / `revoke` /
/// `set-trust-expiry` / `manage-replica`.
#[derive(Debug, clap::Subcommand)]
pub enum IdentityCommand {
    /// `identity show` — display one stored Identity record.
    Show(IdentityShowArgs),
    /// `identity revoke` — mark an Identity revoked (block-only).
    Revoke(IdentityRevokeArgs),
    /// `identity set-trust-expiry` — set the Trust Assertion expiry.
    SetTrustExpiry(IdentitySetTrustExpiryArgs),
    /// `identity manage-replica` — declare/list replica-holding Nodes.
    ManageReplica(IdentityManageReplicaArgs),
}

/// `federation` sub-verbs (A1). M6 shipped `list` + `defederate`; the
/// federation-admin-control 2a arc added `accept`/`reject`/`initiate` (FAC-D1a);
/// the 2b arc adds `set-policy`/`show-policy` (FAC-D3/D4).
#[derive(Debug, clap::Subcommand)]
pub enum FederationCommand {
    /// `federation list` — paginated read of federation relationships.
    List(FederationListArgs),
    /// `federation defederate` — terminate a federation relationship.
    Defederate(FederationDefederateArgs),
    /// `federation accept` — approve a queued inbound federation request (2a).
    Accept(FederationAcceptArgs),
    /// `federation reject` — deny a queued inbound federation request (2a).
    Reject(FederationRejectArgs),
    /// `federation initiate` — operator-outbound establish to a known peer (2a).
    Initiate(FederationInitiateArgs),
    /// `federation set-policy` — upsert a per-peer federation policy (2b).
    SetPolicy(FederationSetPolicyArgs),
    /// `federation show-policy` — read the per-peer policy or the default (2b).
    ShowPolicy(FederationShowPolicyArgs),
}

/// `space` sub-verbs (A4). `list-hosted` + `force-eject` + `unban` shipped in M6;
/// `audit-events` shipped in the protocol-audit-log D-071 arc (Commit 2, J-166);
/// the node-policy verbs defer to the node-policy arc.
#[derive(Debug, clap::Subcommand)]
pub enum SpaceCommand {
    /// `space list-hosted` — list Spaces this Node hosts.
    ListHosted(SpaceListHostedArgs),
    /// `space audit-events` — read the §3.11.8 protocol audit log for one Space.
    AuditEvents(SpaceAuditEventsArgs),
    /// `space audit-rebuild` — regenerate missing audit entries from the DAG (PAL-D3).
    AuditRebuild(SpaceAuditRebuildArgs),
    /// `space force-eject` — Node-administrator removal + ban (A4-D1).
    ForceEject(SpaceForceEjectArgs),
    /// `space unban` — lift a Node-eject ban (A4-D1).
    Unban(SpaceUnbanArgs),
}

/// `plugin` sub-verbs (A7). M6 ships the 2 reads; WRITE verbs (load/configure/
/// unload) deferred until a 2nd plugin exists (A7-D1).
#[derive(Debug, clap::Subcommand)]
pub enum PluginCommand {
    /// `plugin list` — enumerate compiled-in plugins.
    List(PluginListArgs),
    /// `plugin status` — detail for one plugin.
    Status(PluginStatusArgs),
}

/// `auth-module` sub-verbs (A2). Variant names derive to `list` / `register` /
/// `revoke` / `set-tiers`; `test` (the ad-hoc probe) lands at Commit 4.
#[derive(Debug, clap::Subcommand)]
pub enum AuthModuleCommand {
    /// `auth-module list` — enumerate registered Auth Modules.
    List(AuthModuleListArgs),
    /// `auth-module register` — add (or replace) a trusted Auth Module.
    Register(AuthModuleRegisterArgs),
    /// `auth-module revoke` — mark an Auth Module untrusted (block-only, A2-D1).
    Revoke(AuthModuleRevokeArgs),
    /// `auth-module set-tiers` — replace a module's accepted Auth Tier set.
    SetTiers(AuthModuleSetTiersArgs),
    /// `auth-module test` — ad-hoc connectivity probe of a module's endpoint.
    Test(AuthModuleTestArgs),
}

/// `bootstrap` sub-verbs (A3). Variant names derive to `show` / `register` /
/// `deregister` / `set-info` / `set-tiers`.
#[derive(Debug, clap::Subcommand)]
pub enum BootstrapCommand {
    /// `bootstrap show` — list registrations + the advertised self-info.
    Show(BootstrapShowArgs),
    /// `bootstrap register` — register this Node with a Bootstrap Node.
    Register(BootstrapRegisterArgs),
    /// `bootstrap deregister` — remove this Node from a Bootstrap Node's directory.
    Deregister(BootstrapDeregisterArgs),
    /// `bootstrap set-info` — update the advertised endpoint/region/capabilities.
    SetInfo(BootstrapSetInfoArgs),
    /// `bootstrap set-tiers` — set the advertised Auth Tiers (local-only, A3 Option A).
    SetTiers(BootstrapSetTiersArgs),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use xgen_core::federation::pending_queue::PendingFederationRequest;

    #[test]
    fn admin_error_batch_reply_and_display_match_section_2_7() {
        let e = AdminError::new("FED_3041", Stage::Register, "no such peer");
        assert_eq!(e.batch_reply(), "ERROR FED_3041: no such peer");
        assert_eq!(format!("{e}"), "ERROR FED_3041: no such peer");
        assert_eq!(e.stage, Stage::Register);
    }

    #[test]
    fn admin_error_generic_uses_4000_band() {
        let e = AdminError::generic(Stage::Validate, "missing --space");
        assert_eq!(e.code, "GENERIC_4000");
        assert_eq!(e.stage, Stage::Validate);
        assert!(e.batch_reply().starts_with("ERROR GENERIC_4000:"));
    }

    #[test]
    fn actor_via_and_stage_strings_are_stable() {
        assert_eq!(ActorVia::Batch.as_str(), "batch");
        assert_eq!(ActorVia::AiControl.as_str(), "aicontrol");
        assert_eq!(ActorVia::CliDirect.as_str(), "cli-direct");
        assert_eq!(Stage::Federate.as_str(), "federate");
        assert_eq!(Stage::Persist.as_str(), "persist");
    }

    #[test]
    fn admin_context_batch_constructor_sets_via() {
        let dd = PathBuf::from("/tmp/data");
        let cp = PathBuf::from("/tmp/data/xgen-node_config.toml");
        let ctx = AdminContext::batch(&dd, &cp, "xgen://pubkey/ed25519:node");
        assert_eq!(ctx.actor_via, ActorVia::Batch);
        assert_eq!(ctx.actor, "xgen://pubkey/ed25519:node");
    }

    // ── A6 audit verb tests (Phase 4) ────────────────────────────────────────────

    use tempfile::{tempdir, TempDir};

    fn mk(ts: &str, verb: &str, actor: &str, outcome: &str) -> AuditEntry {
        AuditEntry {
            timestamp: ts.to_string(),
            verb: verb.to_string(),
            actor: actor.to_string(),
            actor_via: "batch".to_string(),
            target: None,
            args_hash: "h".to_string(),
            outcome: outcome.to_string(),
            error_code: None,
            error_message: None,
            correlation_id: None,
            meta_atts: "{}".to_string(),
        }
    }

    /// Seed a data dir with three audit rows; returns the dir (kept alive by caller).
    fn seed_dir() -> TempDir {
        let dir = tempdir().unwrap();
        let conn = audit::open_audit_db(dir.path()).unwrap();
        audit::insert_entry(&conn, &mk("2026-05-01T00:00:00.000Z", "federation accept", "alice", "ok")).unwrap();
        audit::insert_entry(&conn, &mk("2026-05-10T00:00:00.000Z", "identity revoke", "bob", "error")).unwrap();
        audit::insert_entry(&conn, &mk("2026-05-20T00:00:00.000Z", "federation accept", "alice", "ok")).unwrap();
        dir
    }

    #[tokio::test]
    async fn audit_query_filters_and_is_not_audited() {
        let dir = seed_dir();
        let cfg = dir.path().join("xgen-node_config.toml");
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin");
        let r = audit_query(
            &mut ctx,
            AuditQueryArgs { actor: Some("alice".into()), ..Default::default() },
        )
        .await
        .unwrap();
        assert_eq!(r.total_matched, 2);
        assert_eq!(r.returned, 2);
        assert_eq!(r.entries[0].timestamp, "2026-05-20T00:00:00.000Z"); // newest first
        // A6-D4: a READ must not write an audit entry — still 3 rows.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 3);
    }

    #[tokio::test]
    async fn audit_query_rejects_bad_timestamp_5010() {
        let dir = seed_dir();
        let cfg = dir.path().join("xgen-node_config.toml");
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin");
        let err = audit_query(
            &mut ctx,
            AuditQueryArgs { since: Some("not-a-timestamp".into()), ..Default::default() },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "AUDIT_5010");
        assert_eq!(err.stage, Stage::Validate);
    }

    #[tokio::test]
    async fn audit_export_writes_jsonl_and_is_audited() {
        let dir = seed_dir();
        let cfg = dir.path().join("xgen-node_config.toml");
        let out = dir.path().join("export.jsonl");
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin");
        let r = audit_export(
            &mut ctx,
            AuditExportArgs {
                actor: Some("alice".into()),
                verb: None,
                since: None,
                until: None,
                outcome: None,
                output: out.clone(),
                format: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(r.exported_count, 2);
        assert_eq!(r.format, "jsonl");
        assert_eq!(std::fs::read_to_string(&out).unwrap().lines().count(), 2);
        // A6-D4: export is data-extracting → audited. 3 seed + 1 export entry = 4.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 4);
        let recent = audit::recent_entries(&conn, 1).unwrap();
        assert_eq!(recent[0].verb, "audit export");
        assert_eq!(recent[0].actor, "admin");
    }

    #[tokio::test]
    async fn audit_archive_prunes_and_is_audited() {
        let dir = seed_dir();
        let cfg = dir.path().join("xgen-node_config.toml");
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin");
        let r = audit_archive(
            &mut ctx,
            AuditArchiveArgs { before: "2026-05-15T00:00:00.000Z".into(), output: None },
        )
        .await
        .unwrap();
        assert_eq!(r.archived_count, 2);
        assert_eq!(r.oldest_ts.as_deref(), Some("2026-05-01T00:00:00.000Z"));
        assert_eq!(r.newest_ts.as_deref(), Some("2026-05-10T00:00:00.000Z"));
        assert!(std::path::Path::new(&r.archive_path).exists());
        // Live table: 3 seed − 2 pruned + 1 audit-the-auditor entry = 2.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 2);
        let recent = audit::recent_entries(&conn, 1).unwrap();
        assert_eq!(recent[0].verb, "audit archive");
    }

    #[tokio::test]
    async fn log_set_level_rejects_invalid_level_5101() {
        // Level validation happens before the reload-handle check, so this is
        // deterministic without an initialised subscriber.
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin");
        let err = log_set_level(&mut ctx, LogSetLevelArgs { module: None, level: "bogus".into() })
            .await
            .unwrap_err();
        assert_eq!(err.code, "LOG_5101");
        assert_eq!(err.stage, Stage::Validate);
    }

    #[tokio::test]
    async fn log_show_level_reports_global_entry() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin");
        let r = log_show_level(&mut ctx, LogShowLevelArgs::default()).await.unwrap();
        assert!(r.levels.iter().any(|e| e.module == "*"));
    }

    // ── A5 identity verb tests (Phase 5) ─────────────────────────────────────────

    const TEST_ID: &str = "xgen://pubkey/ed25519:AAAA";

    /// A live runtime carrying one registered (active) Identity.
    fn runtime_with_identity() -> Arc<Mutex<NodeRuntime>> {
        let kp = xgen_core::identity::keypair::generate();
        let mut rt = NodeRuntime::new(kp);
        let rec = IdentityRecord {
            identity_id: ident_xgid(TEST_ID),
            display_name: Some("Test User".into()),
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-01T00:00:00.000Z".into(),
            trust_assertion: None,
            devices: vec![],
            home_node: xgen_common::xgid::NodeXgid::from_xgid(Xgid::new(
                "xgen://pubkey/ed25519:NODE".into(),
            )),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        };
        rt.identity_registry.register(rec).unwrap();
        Arc::new(Mutex::new(rt))
    }

    #[tokio::test]
    async fn identity_show_found_and_not_found() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let rt = runtime_with_identity();
        let mut ctx = AdminContext::batch_with_runtime(dir.path(), &cfg, "admin", Arc::clone(&rt));

        let r = identity_show(&mut ctx, IdentityShowArgs { identity_id: TEST_ID.into() })
            .await
            .unwrap();
        assert_eq!(r.record.identity_id.as_str(), TEST_ID);
        assert!(!r.record.revoked);

        let err = identity_show(
            &mut ctx,
            IdentityShowArgs { identity_id: "xgen://pubkey/ed25519:NOPE".into() },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "IDENT_6001");
        // A5-D3: show is a pure read → no audit db written.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 0);
    }

    #[tokio::test]
    async fn identity_revoke_marks_persists_audits_then_rejects_double() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let rt = runtime_with_identity();
        let mut ctx = AdminContext::batch_with_runtime(dir.path(), &cfg, "admin", Arc::clone(&rt));

        let r = identity_revoke(
            &mut ctx,
            IdentityRevokeArgs { identity_id: TEST_ID.into(), reason: Some("compromise".into()) },
        )
        .await
        .unwrap();
        assert_eq!(r.identity_id, TEST_ID);
        assert!(!r.revoked_at.is_empty());
        assert!(r.stale_membership_spaces.is_empty()); // no Spaces in this runtime

        // Live registry updated (immediate, A5-D1).
        assert!(rt.lock().await.identity_registry.is_revoked(&ident_xgid(TEST_ID)));
        // Persisted to disk.
        assert!(dir.path().join("xgen-node_identities.db").exists());
        // DESTRUCTIVE → audited (one "identity revoke" entry).
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 1);
        assert_eq!(audit::recent_entries(&conn, 1).unwrap()[0].verb, "identity revoke");

        // Double-revoke → IDENT_6002.
        let err = identity_revoke(
            &mut ctx,
            IdentityRevokeArgs { identity_id: TEST_ID.into(), reason: None },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "IDENT_6002");

        // Unknown → IDENT_6001.
        let err = identity_revoke(
            &mut ctx,
            IdentityRevokeArgs { identity_id: "xgen://pubkey/ed25519:NOPE".into(), reason: None },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "IDENT_6001");
    }

    #[tokio::test]
    async fn identity_set_trust_expiry_validates_sets_and_reports_previous() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let rt = runtime_with_identity();
        let mut ctx = AdminContext::batch_with_runtime(dir.path(), &cfg, "admin", Arc::clone(&rt));

        // Malformed expiry → IDENT_6010 (before any mutation).
        let err = identity_set_trust_expiry(
            &mut ctx,
            IdentitySetTrustExpiryArgs { identity_id: TEST_ID.into(), expiry: "soon".into() },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "IDENT_6010");
        assert_eq!(err.stage, Stage::Validate);

        // First valid set: previous None.
        let r = identity_set_trust_expiry(
            &mut ctx,
            IdentitySetTrustExpiryArgs {
                identity_id: TEST_ID.into(),
                expiry: "2027-01-01T00:00:00Z".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(r.previous_expiry, None);
        assert_eq!(r.new_expiry, "2027-01-01T00:00:00Z");

        // Second set reports previous + is audited (2 entries).
        let r2 = identity_set_trust_expiry(
            &mut ctx,
            IdentitySetTrustExpiryArgs {
                identity_id: TEST_ID.into(),
                expiry: "2028-01-01T00:00:00Z".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(r2.previous_expiry.as_deref(), Some("2027-01-01T00:00:00Z"));
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 2);

        // Unknown identity → IDENT_6001.
        let err = identity_set_trust_expiry(
            &mut ctx,
            IdentitySetTrustExpiryArgs {
                identity_id: "xgen://pubkey/ed25519:NOPE".into(),
                expiry: "2027-01-01T00:00:00Z".into(),
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "IDENT_6001");
    }

    #[tokio::test]
    async fn identity_manage_replica_add_list_remove_with_guards() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let rt = runtime_with_identity();
        let mut ctx = AdminContext::batch_with_runtime(dir.path(), &cfg, "admin", Arc::clone(&rt));
        let node = "xgen://pubkey/ed25519:PEER".to_string();

        // list (empty) — not audited.
        let r = identity_manage_replica(
            &mut ctx,
            IdentityManageReplicaArgs {
                identity_id: TEST_ID.into(),
                action: ReplicaAction::List,
                node_id: None,
            },
        )
        .await
        .unwrap();
        assert!(r.replicas.is_empty());

        // add → present.
        let r = identity_manage_replica(
            &mut ctx,
            IdentityManageReplicaArgs {
                identity_id: TEST_ID.into(),
                action: ReplicaAction::Add,
                node_id: Some(node.clone()),
            },
        )
        .await
        .unwrap();
        assert_eq!(r.replicas, vec![node.clone()]);

        // add same again → IDENT_6021.
        let err = identity_manage_replica(
            &mut ctx,
            IdentityManageReplicaArgs {
                identity_id: TEST_ID.into(),
                action: ReplicaAction::Add,
                node_id: Some(node.clone()),
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "IDENT_6021");

        // add without --node-id → IDENT_6020.
        let err = identity_manage_replica(
            &mut ctx,
            IdentityManageReplicaArgs {
                identity_id: TEST_ID.into(),
                action: ReplicaAction::Add,
                node_id: None,
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "IDENT_6020");

        // remove → empty.
        let r = identity_manage_replica(
            &mut ctx,
            IdentityManageReplicaArgs {
                identity_id: TEST_ID.into(),
                action: ReplicaAction::Remove,
                node_id: Some(node.clone()),
            },
        )
        .await
        .unwrap();
        assert!(r.replicas.is_empty());

        // unknown identity → IDENT_6001.
        let err = identity_manage_replica(
            &mut ctx,
            IdentityManageReplicaArgs {
                identity_id: "xgen://pubkey/ed25519:NOPE".into(),
                action: ReplicaAction::List,
                node_id: None,
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "IDENT_6001");

        // add + remove audited; the two list/guard reads + errors not → 2 entries.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 2);
    }

    // ── A1 federation verb tests (Phase 7, honest-subset) ────────────────────────

    fn fed_rel(peer: &str, spaces: &[&str]) -> FederationRelationship {
        FederationRelationship {
            peer_node_id: node_xgid(peer),
            shared_spaces: spaces
                .iter()
                .map(|s| xgen_common::xgid::SpaceXgid::from_xgid(Xgid::new(s.to_string())))
                .collect(),
            negotiated_version: "0.1".into(),
            negotiated_serialisation: "json".into(),
            session_id: "xgen://hash/sha256:session".into(),
            last_connected: "2026-05-01T00:00:00.000Z".into(),
            peer_url: None,
            state: xgen_core::federation::registry::FederationState::Active,
        }
    }

    fn fed_registry(rels: Vec<FederationRelationship>) -> Arc<Mutex<FederationRegistry>> {
        let mut reg = FederationRegistry::new();
        for r in rels {
            reg.upsert(r);
        }
        Arc::new(Mutex::new(reg))
    }

    #[tokio::test]
    async fn federation_list_paginates_and_validates_state() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let fr = fed_registry(vec![
            fed_rel("xgen://pubkey/ed25519:AAAA", &["xgen://hash/sha256:s1"]),
            fed_rel("xgen://pubkey/ed25519:BBBB", &[]),
            fed_rel("xgen://pubkey/ed25519:CCCC", &[]),
        ]);
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_federation_registry(Arc::clone(&fr));

        // Page 1 of 2 (limit 2) — sorted by peer_node_id, next_cursor set.
        let r = federation_list(
            &mut ctx,
            FederationListArgs { state: None, limit: Some(2), cursor: None },
        )
        .await
        .unwrap();
        assert_eq!(r.total_matched, 3);
        assert_eq!(r.returned, 2);
        assert_eq!(r.relationships[0].peer_node_id.as_str(), "xgen://pubkey/ed25519:AAAA");
        assert_eq!(r.next_cursor.as_deref(), Some("xgen://pubkey/ed25519:BBBB"));

        // Page 2 — cursor continues after BBBB.
        let r2 = federation_list(
            &mut ctx,
            FederationListArgs { state: Some("all".into()), limit: Some(2), cursor: r.next_cursor },
        )
        .await
        .unwrap();
        assert_eq!(r2.returned, 1);
        assert_eq!(r2.relationships[0].peer_node_id.as_str(), "xgen://pubkey/ed25519:CCCC");
        assert_eq!(r2.next_cursor, None);

        // FAC-D2: `--state pending` filters on the real state field — the
        // fixtures are all Active, so it matches nothing here.
        let r3 = federation_list(
            &mut ctx,
            FederationListArgs { state: Some("pending".into()), limit: None, cursor: None },
        )
        .await
        .unwrap();
        assert_eq!(r3.total_matched, 0);

        // Invalid state → FED_3001.
        let err = federation_list(
            &mut ctx,
            FederationListArgs { state: Some("bogus".into()), limit: None, cursor: None },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "FED_3001");

        // READ → not audited.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 0);
    }

    #[tokio::test]
    async fn federation_defederate_removes_persists_audits_then_rejects() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let fr = fed_registry(vec![fed_rel(
            "xgen://pubkey/ed25519:PEER",
            &["xgen://hash/sha256:s1", "xgen://hash/sha256:s2"],
        )]);
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_federation_registry(Arc::clone(&fr));

        let r = federation_defederate(
            &mut ctx,
            FederationDefederateArgs {
                peer_node_id: "xgen://pubkey/ed25519:PEER".into(),
                reason: Some("ops".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(r.peer_node_id, "xgen://pubkey/ed25519:PEER");
        assert!(!r.defederated_at.is_empty());
        assert_eq!(r.cleaned_spaces, vec!["xgen://hash/sha256:s1", "xgen://hash/sha256:s2"]);

        // Live registry no longer has the peer; persisted to disk.
        assert!(fr.lock().await.get(&node_xgid("xgen://pubkey/ed25519:PEER")).is_none());
        assert!(dir.path().join("xgen-node_federation.json").exists());

        // DESTRUCTIVE → audited (1 "federation defederate" entry).
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 1);
        assert_eq!(audit::recent_entries(&conn, 1).unwrap()[0].verb, "federation defederate");

        // Not federated now → FED_3004.
        let err = federation_defederate(
            &mut ctx,
            FederationDefederateArgs {
                peer_node_id: "xgen://pubkey/ed25519:PEER".into(),
                reason: None,
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "FED_3004");
    }

    // ── A1 2a accept / reject / initiate (FAC-D1a) ───────────────────────────────

    fn pending_req(peer: &str, spaces: &[&str]) -> PendingFederationRequest {
        PendingFederationRequest {
            peer_node_id: node_xgid(peer),
            peer_url: Some("ws://127.0.0.1:8081/xgen".into()),
            received_at: "2026-05-30T00:00:00.000Z".into(),
            shared_spaces: spaces
                .iter()
                .map(|s| SpaceXgid::from_xgid(Xgid::new(s.to_string())))
                .collect(),
            negotiated_version: "0.1".into(),
            negotiated_serialisation: "json".into(),
        }
    }

    fn fed_queue(reqs: Vec<PendingFederationRequest>) -> Arc<Mutex<PendingFederationQueue>> {
        let mut q = PendingFederationQueue::new();
        for r in reqs {
            q.add(r);
        }
        Arc::new(Mutex::new(q))
    }

    #[tokio::test]
    async fn federation_accept_dequeues_activates_persists_audits() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let fr = fed_registry(vec![]);
        let fq = fed_queue(vec![pending_req(
            "xgen://pubkey/ed25519:PEER",
            &["xgen://hash/sha256:s1"],
        )]);
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_federation_registry(Arc::clone(&fr))
            .with_federation_queue(Arc::clone(&fq));

        let r = federation_accept(
            &mut ctx,
            FederationAcceptArgs { peer_node_id: "xgen://pubkey/ed25519:PEER".into() },
        )
        .await
        .unwrap();
        assert_eq!(r.shared_spaces, vec!["xgen://hash/sha256:s1"]);

        // Queue drained; registry now has the peer Active.
        assert!(fq.lock().await.get(&node_xgid("xgen://pubkey/ed25519:PEER")).is_none());
        {
            let reg = fr.lock().await;
            let rel = reg.get(&node_xgid("xgen://pubkey/ed25519:PEER")).unwrap();
            assert_eq!(rel.state, FederationState::Active);
        }
        // WRITE → audited.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 1);
        assert_eq!(audit::recent_entries(&conn, 1).unwrap()[0].verb, "federation accept");

        // Accepting an absent peer → FED_3005.
        let err = federation_accept(
            &mut ctx,
            FederationAcceptArgs { peer_node_id: "xgen://pubkey/ed25519:NOPE".into() },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "FED_3005");
    }

    #[tokio::test]
    async fn federation_reject_dequeues_tombstones_persists_audits() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let fr = fed_registry(vec![]);
        let fq = fed_queue(vec![pending_req("xgen://pubkey/ed25519:PEER", &[])]);
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_federation_registry(Arc::clone(&fr))
            .with_federation_queue(Arc::clone(&fq));

        federation_reject(
            &mut ctx,
            FederationRejectArgs {
                peer_node_id: "xgen://pubkey/ed25519:PEER".into(),
                reason: Some("spam".into()),
            },
        )
        .await
        .unwrap();

        // Queue drained; a Rejected tombstone now suppresses the gate.
        assert!(fq.lock().await.get(&node_xgid("xgen://pubkey/ed25519:PEER")).is_none());
        {
            let reg = fr.lock().await;
            let rel = reg.get(&node_xgid("xgen://pubkey/ed25519:PEER")).unwrap();
            assert_eq!(rel.state, FederationState::Rejected);
        }
        // DESTRUCTIVE → audited.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::recent_entries(&conn, 1).unwrap()[0].verb, "federation reject");

        // Rejecting an absent peer → FED_3005.
        let err = federation_reject(
            &mut ctx,
            FederationRejectArgs { peer_node_id: "xgen://pubkey/ed25519:NOPE".into(), reason: None },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "FED_3005");
    }

    #[tokio::test]
    async fn federation_set_and_show_policy_round_trip_and_audit() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let fp = Arc::new(Mutex::new(FederationPolicyStore::new()));
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_federation_policy(Arc::clone(&fp));
        let peer = "xgen://pubkey/ed25519:PEER".to_string();

        // show BEFORE any set → default (permit-all), is_default = true, not audited.
        let d = federation_show_policy(
            &mut ctx,
            FederationShowPolicyArgs { peer_node_id: peer.clone() },
        )
        .await
        .unwrap();
        assert!(d.is_default);
        assert_eq!(d.mode, "allow");
        assert!(d.allowed_spaces.is_none());

        // set deny → live store reflects it immediately.
        let r = federation_set_policy(
            &mut ctx,
            FederationSetPolicyArgs { peer_node_id: peer.clone(), mode: "deny".into(), allowed_space: vec![] },
        )
        .await
        .unwrap();
        assert_eq!(r.mode, "deny");
        assert_eq!(
            fp.lock().await.get(&node_xgid(&peer)).unwrap().mode,
            PolicyMode::Deny
        );

        // show AFTER set → not default, deny.
        let s = federation_show_policy(
            &mut ctx,
            FederationShowPolicyArgs { peer_node_id: peer.clone() },
        )
        .await
        .unwrap();
        assert!(!s.is_default);
        assert_eq!(s.mode, "deny");

        // re-set allow + restrictive allowed-spaces (insert-or-replace).
        let r2 = federation_set_policy(
            &mut ctx,
            FederationSetPolicyArgs {
                peer_node_id: peer.clone(),
                mode: "allow".into(),
                allowed_space: vec![
                    "xgen://hash/sha256:s1".into(),
                    "xgen://hash/sha256:s2".into(),
                ],
            },
        )
        .await
        .unwrap();
        assert_eq!(r2.allowed_spaces.as_ref().unwrap().len(), 2);
        let s2 = federation_show_policy(
            &mut ctx,
            FederationShowPolicyArgs { peer_node_id: peer.clone() },
        )
        .await
        .unwrap();
        assert_eq!(s2.mode, "allow");
        assert_eq!(s2.allowed_spaces.as_ref().unwrap().len(), 2);

        // invalid mode → FED_3008 (rejected at validate, before any audit).
        let err = federation_set_policy(
            &mut ctx,
            FederationSetPolicyArgs { peer_node_id: peer.clone(), mode: "maybe".into(), allowed_space: vec![] },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "FED_3008");

        // set-policy is WRITE → audited (2 successful sets); show-policy is READ
        // → NOT audited; the invalid set errored before audit. So exactly 2.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 2);
        assert_eq!(
            audit::recent_entries(&conn, 1).unwrap()[0].verb,
            "federation set-policy"
        );
    }

    #[tokio::test]
    async fn federation_initiate_error_paths() {
        use crate::node::runtime::NodeRuntime;
        use crate::identity::keypair;

        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let rt = Arc::new(Mutex::new(NodeRuntime::new(keypair::generate())));

        // No relationship for the peer → FED_3006 (initiate targets known peers).
        let fr_empty = fed_registry(vec![]);
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_runtime(Arc::clone(&rt))
            .with_federation_registry(Arc::clone(&fr_empty));
        let err = federation_initiate(
            &mut ctx,
            FederationInitiateArgs { peer_node_id: "xgen://pubkey/ed25519:UNKNOWN".into() },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "FED_3006");

        // Known peer but no stored endpoint URL → FED_3007 (fed_rel has peer_url None).
        let fr = fed_registry(vec![fed_rel("xgen://pubkey/ed25519:PEER", &[])]);
        let mut ctx2 = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_runtime(Arc::clone(&rt))
            .with_federation_registry(Arc::clone(&fr));
        let err2 = federation_initiate(
            &mut ctx2,
            FederationInitiateArgs { peer_node_id: "xgen://pubkey/ed25519:PEER".into() },
        )
        .await
        .unwrap_err();
        assert_eq!(err2.code, "FED_3007");
    }

    // ── A4 space list-hosted test (Phase 9 read subset) ──────────────────────────

    #[tokio::test]
    async fn space_list_hosted_filters_by_home_node_and_name() {
        use xgen_core::space::state::{build_space_create_event, sign_event, SpaceState};

        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let kp = xgen_core::identity::keypair::generate();
        let mut rt = NodeRuntime::new(kp.clone());
        let me = rt.node_id.as_str().to_string();

        // Two hosted Spaces (home_node == this Node).
        for name in ["Alpha", "Bravo"] {
            let ev = sign_event(build_space_create_event(&kp, name, None, 1, &me), &kp);
            let s = SpaceState::from_space_create(&ev).unwrap();
            rt.spaces.insert(s.space_id.clone(), s);
        }
        // One federated-in Space (home_node = a different Node) — must be excluded.
        let other = xgen_core::identity::keypair::generate();
        let ev = sign_event(
            build_space_create_event(&other, "Charlie", None, 1, "xgen://pubkey/ed25519:OTHER"),
            &other,
        );
        let s = SpaceState::from_space_create(&ev).unwrap();
        rt.spaces.insert(s.space_id.clone(), s);

        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_runtime(Arc::new(Mutex::new(rt)));

        // No filter → only the 2 hosted Spaces.
        let r = space_list_hosted(&mut ctx, SpaceListHostedArgs { name_filter: None })
            .await
            .unwrap();
        assert_eq!(r.spaces.len(), 2);
        let names: Vec<&str> = r.spaces.iter().filter_map(|s| s.name.as_deref()).collect();
        assert!(names.contains(&"Alpha") && names.contains(&"Bravo"));
        assert!(!names.contains(&"Charlie")); // federated-in excluded
        assert!(r.spaces.iter().all(|s| s.created_at.is_none())); // honest None

        // Case-insensitive name filter.
        let r2 = space_list_hosted(
            &mut ctx,
            SpaceListHostedArgs { name_filter: Some("alph".into()) },
        )
        .await
        .unwrap();
        assert_eq!(r2.spaces.len(), 1);
        assert_eq!(r2.spaces[0].name.as_deref(), Some("Alpha"));

        // READ → not audited.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 0);
    }

    // ── space audit-events tests (protocol-audit-log arc, Commit 2) ──────────────

    /// Build a hosted-Space runtime + ctx; returns (ctx-able runtime, space_id,
    /// audit_dir under data_dir). The audit dir is populated by the caller.
    fn audit_reader_fixture(
        dir: &std::path::Path,
    ) -> (NodeRuntime, String, std::path::PathBuf) {
        use xgen_core::space::state::{build_space_create_event, sign_event, SpaceState};
        let kp = xgen_core::identity::keypair::generate();
        let mut rt = NodeRuntime::new(kp.clone());
        let me = rt.node_id.as_str().to_string();
        let ev = sign_event(build_space_create_event(&kp, "Alpha", None, 1, &me), &kp);
        let s = SpaceState::from_space_create(&ev).unwrap();
        let space_id = s.space_id.as_str().to_string();
        rt.spaces.insert(s.space_id.clone(), s);
        (rt, space_id, dir.join("audit"))
    }

    fn audit_entry(
        event_type: &str,
        space_id: &str,
        ts: &str,
        event_id: &str,
    ) -> crate::protocol_audit::ProtocolAuditEntry {
        let mut extra = serde_json::Map::new();
        extra.insert("space_id".to_string(), serde_json::json!(space_id));
        extra.insert(
            "identity_id".to_string(),
            serde_json::json!("xgen://pubkey/ed25519:X"),
        );
        crate::protocol_audit::ProtocolAuditEntry {
            ts: ts.to_string(),
            event_type: event_type.to_string(),
            event_id: event_id.to_string(),
            node_id: "xgen://pubkey/ed25519:NODE".to_string(),
            extra,
        }
    }

    fn write_audit_month(
        audit_dir: &std::path::Path,
        month: &str,
        entries: &[crate::protocol_audit::ProtocolAuditEntry],
    ) {
        std::fs::create_dir_all(audit_dir).unwrap();
        let mut body = String::new();
        for e in entries {
            body.push_str(&serde_json::to_string(e).unwrap());
            body.push('\n');
        }
        std::fs::write(audit_dir.join(format!("protocol_audit_{month}.jsonl")), body).unwrap();
    }

    #[tokio::test]
    async fn space_audit_events_filters_by_space_event_type_and_time() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let (rt, space_a, audit_dir) = audit_reader_fixture(dir.path());
        write_audit_month(
            &audit_dir,
            "2026-05",
            &[
                audit_entry("membership.join", &space_a, "2026-05-02T00:00:00.000Z", "e1"),
                audit_entry("state.room_create", &space_a, "2026-05-10T00:00:00.000Z", "e2"),
                // Different Space — must be filtered out.
                audit_entry("membership.join", "xgen://hash/sha256:OTHER", "2026-05-03T00:00:00.000Z", "e3"),
            ],
        );
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_runtime(Arc::new(Mutex::new(rt)));

        // space_id filter only → 2 of the 3 lines.
        let r = space_audit_events(
            &mut ctx,
            SpaceAuditEventsArgs { space_id: space_a.clone(), ..Default::default() },
        )
        .await
        .unwrap();
        assert_eq!(r.returned, 2);
        assert!(r.next_cursor.is_none());
        assert!(r.events.iter().all(|e| e
            .extra
            .get("space_id")
            .and_then(|v| v.as_str())
            == Some(space_a.as_str())));

        // + event_type filter.
        let r = space_audit_events(
            &mut ctx,
            SpaceAuditEventsArgs {
                space_id: space_a.clone(),
                event_type: Some("membership.join".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(r.returned, 1);
        assert_eq!(r.events[0].event_id, "e1");

        // + since/until range (only the May-10 room_create).
        let r = space_audit_events(
            &mut ctx,
            SpaceAuditEventsArgs {
                space_id: space_a.clone(),
                since: Some("2026-05-05T00:00:00.000Z".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(r.returned, 1);
        assert_eq!(r.events[0].event_id, "e2");

        // empty result (event_type that never occurs for this Space).
        let r = space_audit_events(
            &mut ctx,
            SpaceAuditEventsArgs {
                space_id: space_a.clone(),
                event_type: Some("membership.ban".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(r.returned, 0);
        assert!(r.events.is_empty());
        assert!(r.next_cursor.is_none());
    }

    #[tokio::test]
    async fn space_audit_events_paginates_across_months() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let (rt, space_a, audit_dir) = audit_reader_fixture(dir.path());
        // Cross-month: two in April, three in May (chronological across files).
        write_audit_month(
            &audit_dir,
            "2026-04",
            &[
                audit_entry("membership.join", &space_a, "2026-04-01T00:00:00.000Z", "a1"),
                audit_entry("membership.join", &space_a, "2026-04-02T00:00:00.000Z", "a2"),
            ],
        );
        write_audit_month(
            &audit_dir,
            "2026-05",
            &[
                audit_entry("membership.join", &space_a, "2026-05-01T00:00:00.000Z", "a3"),
                audit_entry("membership.join", &space_a, "2026-05-02T00:00:00.000Z", "a4"),
                audit_entry("membership.join", &space_a, "2026-05-03T00:00:00.000Z", "a5"),
            ],
        );
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_runtime(Arc::new(Mutex::new(rt)));

        // Page 1: limit 2 → a1,a2 (chronological, April first); cursor "2".
        let p1 = space_audit_events(
            &mut ctx,
            SpaceAuditEventsArgs { space_id: space_a.clone(), limit: Some(2), ..Default::default() },
        )
        .await
        .unwrap();
        assert_eq!(p1.returned, 2);
        let ids1: Vec<&str> = p1.events.iter().map(|e| e.event_id.as_str()).collect();
        assert_eq!(ids1, vec!["a1", "a2"]);
        assert_eq!(p1.next_cursor.as_deref(), Some("2"));

        // Page 2: cursor 2, limit 2 → a3,a4 (crosses into May); cursor "4".
        let p2 = space_audit_events(
            &mut ctx,
            SpaceAuditEventsArgs {
                space_id: space_a.clone(),
                limit: Some(2),
                cursor: Some("2".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let ids2: Vec<&str> = p2.events.iter().map(|e| e.event_id.as_str()).collect();
        assert_eq!(ids2, vec!["a3", "a4"]);
        assert_eq!(p2.next_cursor.as_deref(), Some("4"));

        // Page 3: cursor 4 → a5, exhausted (no next_cursor).
        let p3 = space_audit_events(
            &mut ctx,
            SpaceAuditEventsArgs {
                space_id: space_a.clone(),
                limit: Some(2),
                cursor: Some("4".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let ids3: Vec<&str> = p3.events.iter().map(|e| e.event_id.as_str()).collect();
        assert_eq!(ids3, vec!["a5"]);
        assert!(p3.next_cursor.is_none());
    }

    #[tokio::test]
    async fn space_audit_events_rejects_bad_filters_and_unknown_space() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let (rt, space_a, _audit_dir) = audit_reader_fixture(dir.path());
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_runtime(Arc::new(Mutex::new(rt)));

        // Malformed since → SPACE_8010.
        let e = space_audit_events(
            &mut ctx,
            SpaceAuditEventsArgs {
                space_id: space_a.clone(),
                since: Some("not-a-date".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
        assert_eq!(e.code, "SPACE_8010");

        // Malformed cursor → SPACE_8010.
        let e = space_audit_events(
            &mut ctx,
            SpaceAuditEventsArgs {
                space_id: space_a.clone(),
                cursor: Some("abc".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
        assert_eq!(e.code, "SPACE_8010");

        // Unknown Space (not hosted/federated here) → SPACE_8001.
        let e = space_audit_events(
            &mut ctx,
            SpaceAuditEventsArgs {
                space_id: "xgen://hash/sha256:NOPE".into(),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
        assert_eq!(e.code, "SPACE_8001");
    }

    // ── space audit-rebuild tests (protocol-audit-log arc, Commit 3 / PAL-D3) ────

    /// Minimal audited Event with a chosen event_id + space_id, ts in 2026-05.
    fn make_audited_event(
        event_type: xgen_common::wire::EventType,
        space_id: &str,
        event_id: &str,
    ) -> xgen_common::wire::Event {
        use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};
        let mut e = xgen_common::wire::Event::new(
            event_type,
            IdentityXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:SENDER".to_string())),
            RoomXgid::from_xgid(Xgid::new("xgen://hash/sha256:room".to_string())),
            SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
            vec![],
            "2026-05-15T00:00:00.000Z".to_string(),
            serde_json::json!({}),
        );
        e.event_id = Some(EventXgid::from_xgid(Xgid::new(event_id.to_string())));
        e
    }

    fn audit_line_count(audit_dir: &std::path::Path, month: &str) -> usize {
        std::fs::read_to_string(audit_dir.join(format!("protocol_audit_{month}.jsonl")))
            .map(|s| s.lines().count())
            .unwrap_or(0)
    }

    #[tokio::test]
    async fn space_audit_rebuild_recovers_gap_and_is_idempotent() {
        use xgen_common::wire::EventType;
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let (rt, space_a, audit_dir) = audit_reader_fixture(dir.path());
        let spaces_dir = dir.path().join("spaces");

        // Three audited events persisted to the Space store.
        for id in ["j1", "j2", "j3"] {
            crate::app::persist_event(
                &spaces_dir,
                &space_a,
                &make_audited_event(EventType::MembershipJoin, &space_a, id),
            );
        }
        // Audit log already has j1, j2 — j3 is the gap.
        write_audit_month(
            &audit_dir,
            "2026-05",
            &[
                audit_entry("membership.join", &space_a, "2026-05-15T00:00:00.000Z", "j1"),
                audit_entry("membership.join", &space_a, "2026-05-15T00:00:00.000Z", "j2"),
            ],
        );

        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_runtime(Arc::new(Mutex::new(rt)));

        let r = space_audit_rebuild(
            &mut ctx,
            SpaceAuditRebuildArgs { space_id: Some(space_a.clone()), dry_run: false },
        )
        .await
        .unwrap();
        assert_eq!(r.spaces_scanned, 1);
        assert_eq!(r.entries_added, 1); // j3 recovered
        assert_eq!(r.entries_already_present, 2); // j1, j2
        assert_eq!(audit_line_count(&audit_dir, "2026-05"), 3);

        // Idempotent: a second run adds nothing.
        let r2 = space_audit_rebuild(
            &mut ctx,
            SpaceAuditRebuildArgs { space_id: Some(space_a.clone()), dry_run: false },
        )
        .await
        .unwrap();
        assert_eq!(r2.entries_added, 0);
        assert_eq!(r2.entries_already_present, 3);
        assert_eq!(audit_line_count(&audit_dir, "2026-05"), 3);

        // The WRITE rebuild is recorded in the A6 trail (2 non-dry-run runs).
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 2);
    }

    #[tokio::test]
    async fn space_audit_rebuild_cold_start_backfill_and_dry_run() {
        use xgen_common::wire::EventType;
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let (rt, space_a, audit_dir) = audit_reader_fixture(dir.path());
        let spaces_dir = dir.path().join("spaces");
        for id in ["c1", "c2"] {
            crate::app::persist_event(
                &spaces_dir,
                &space_a,
                &make_audited_event(EventType::MembershipJoin, &space_a, id),
            );
        }
        // Audit log empty (cold start).
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_runtime(Arc::new(Mutex::new(rt)));

        // dry_run: reports what would be added, writes nothing, not audited.
        let dr = space_audit_rebuild(
            &mut ctx,
            SpaceAuditRebuildArgs { space_id: None, dry_run: true },
        )
        .await
        .unwrap();
        assert_eq!(dr.entries_added, 2);
        assert_eq!(audit_line_count(&audit_dir, "2026-05"), 0); // nothing written
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 0); // dry-run not audited

        // Real run backfills both.
        let r = space_audit_rebuild(
            &mut ctx,
            SpaceAuditRebuildArgs { space_id: None, dry_run: false },
        )
        .await
        .unwrap();
        assert_eq!(r.spaces_scanned, 1); // rebuild-all over the one hosted Space
        assert_eq!(r.entries_added, 2);
        assert_eq!(audit_line_count(&audit_dir, "2026-05"), 2);
    }

    #[tokio::test]
    async fn space_audit_rebuild_unknown_space_rejects() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let (rt, _space_a, _audit_dir) = audit_reader_fixture(dir.path());
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_runtime(Arc::new(Mutex::new(rt)));
        let e = space_audit_rebuild(
            &mut ctx,
            SpaceAuditRebuildArgs {
                space_id: Some("xgen://hash/sha256:NOPE".into()),
                dry_run: false,
            },
        )
        .await
        .unwrap_err();
        assert_eq!(e.code, "SPACE_8001");
    }

    // ── A7 plugin verb tests (Phase 10) ──────────────────────────────────────────

    #[tokio::test]
    async fn plugin_list_returns_compiled_in_plugins() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin");
        let r = plugin_list(&mut ctx, PluginListArgs {}).await.unwrap();
        // One compiled-in plugin today: the temperature slot (no-op impl).
        assert_eq!(r.plugins.len(), 1);
        assert_eq!(r.plugins[0].name, "noop-temperature");
        assert_eq!(r.plugins[0].kind, "temperature");
        assert_eq!(r.plugins[0].status, "loaded");
        // READ → not audited.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 0);
    }

    #[tokio::test]
    async fn plugin_status_found_and_unknown() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin");

        let r = plugin_status(
            &mut ctx,
            PluginStatusArgs { plugin_name: "noop-temperature".into() },
        )
        .await
        .unwrap();
        assert_eq!(r.kind, "temperature");
        // No telemetry tracked in M6 → honest None.
        assert_eq!(r.events_consumed, None);
        assert_eq!(r.last_activity, None);

        let err = plugin_status(
            &mut ctx,
            PluginStatusArgs { plugin_name: "nope".into() },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "PLUGIN_9001");
    }

    // ── A4 force-eject / unban tests (Phase 9, A4-D1) ────────────────────────────

    fn active_record(uri: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: ident_xgid(uri),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-01T00:00:00.000Z".into(),
            trust_assertion: None,
            devices: vec![],
            home_node: NodeXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:NODE".into())),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    /// A live runtime with a Space hosted by this Node and `bob` as a member.
    /// Returns (runtime, space_id, bob_uri).
    fn runtime_with_hosted_space_and_member() -> (Arc<Mutex<NodeRuntime>>, String, String) {
        use xgen_core::identity::registration::identity_id_from_key;
        use xgen_core::space::membership::Role;
        use xgen_core::space::state::{build_space_create_event, sign_event, SpaceMember};

        let node_kp = xgen_core::identity::keypair::generate();
        let mut rt = NodeRuntime::new(node_kp);
        let node_uri = rt.node_id.as_str().to_string();
        let alice = xgen_core::identity::keypair::generate();
        let bob = xgen_core::identity::keypair::generate();
        let alice_uri = identity_id_from_key(&alice);
        let bob_uri = identity_id_from_key(&bob);
        rt.identity_registry.register(active_record(&alice_uri)).unwrap();
        rt.identity_registry.register(active_record(&bob_uri)).unwrap();

        // Create a Space homed at this Node (gives a real DAG + tip).
        let create =
            sign_event(build_space_create_event(&alice, "Hosted", None, 1, &node_uri), &alice);
        let space_id = create.event_id.as_ref().unwrap().as_str().to_string();
        match rt.dispatch_event(create, EventOrigin::LocallySubmitted, None) {
            DispatchOutcome::Accepted { .. } => {}
            _ => panic!("space create was not accepted"),
        }

        // Poke bob in as a member (skip the invite/join dance).
        let sx = SpaceXgid::from_xgid(Xgid::new(space_id.clone()));
        let bx = ident_xgid(&bob_uri);
        rt.spaces.get_mut(&sx).unwrap().members.insert(
            bx.clone(),
            SpaceMember {
                identity_id: bx,
                role: Role::Member,
                joined_at: "2026-05-01T00:00:00.000Z".into(),
                invited_by: None,
            },
        );
        (Arc::new(Mutex::new(rt)), space_id, bob_uri)
    }

    #[tokio::test]
    async fn space_force_eject_then_unban_full_cycle() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let (rt, space_id, bob_uri) = runtime_with_hosted_space_and_member();
        let mut ctx =
            AdminContext::batch(dir.path(), &cfg, "admin").with_runtime(Arc::clone(&rt));
        let sx = SpaceXgid::from_xgid(Xgid::new(space_id.clone()));
        let bx = ident_xgid(&bob_uri);

        // Force-eject bob: removed + banned in the live state, event emitted.
        let r = space_force_eject(
            &mut ctx,
            SpaceForceEjectArgs {
                space_id: space_id.clone(),
                identity_id: bob_uri.clone(),
                reason: Some("ops".into()),
            },
        )
        .await
        .unwrap();
        assert!(!r.event_id.is_empty());
        {
            let g = rt.lock().await;
            let space = g.spaces.get(&sx).unwrap();
            assert!(!space.is_member(bob_uri.as_str()));
            assert!(space.banned.contains(&bx));
        }
        // DESTRUCTIVE → audited; correlation_id == emitted event_id.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 1);
        let recent = audit::recent_entries(&conn, 1).unwrap();
        assert_eq!(recent[0].verb, "space force-eject");
        assert_eq!(recent[0].correlation_id.as_deref(), Some(r.event_id.as_str()));

        // Unban bob: ban lifted.
        let u = space_unban(
            &mut ctx,
            SpaceUnbanArgs {
                space_id: space_id.clone(),
                identity_id: bob_uri.clone(),
                reason: None,
            },
        )
        .await
        .unwrap();
        assert!(!u.event_id.is_empty());
        {
            let g = rt.lock().await;
            assert!(!g.spaces.get(&sx).unwrap().banned.contains(&bx));
        }
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 2);
    }

    #[tokio::test]
    async fn space_force_eject_error_paths() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let (rt, space_id, _bob) = runtime_with_hosted_space_and_member();
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin").with_runtime(rt);

        // Unknown / non-hosted Space → SPACE_8001.
        let err = space_force_eject(
            &mut ctx,
            SpaceForceEjectArgs {
                space_id: "xgen://hash/sha256:nope".into(),
                identity_id: "xgen://pubkey/ed25519:BOB".into(),
                reason: None,
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "SPACE_8001");

        // Hosted Space, but target is not a member → SPACE_8002.
        let err = space_force_eject(
            &mut ctx,
            SpaceForceEjectArgs {
                space_id: space_id.clone(),
                identity_id: "xgen://pubkey/ed25519:NOTAMEMBER".into(),
                reason: None,
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "SPACE_8002");
    }

    // ── A4 Option B live fan-out (J-160) ─────────────────────────────────────────

    /// Option B: a `space force-eject` (then `unban`) pushes the Node-authored
    /// `membership.node_eject` / `node_unban` LIVE to a registered remaining-
    /// member client sender AND a registered federation peer sender — not just
    /// persists (the Option-A baseline). The ejected target itself is removed
    /// before fan-out, so it is intentionally absent from the live push (learns
    /// via sync); we assert delivery to a *remaining* member (alice).
    #[tokio::test]
    async fn space_force_eject_fans_out_live_to_clients_and_peers() {
        use crate::fanout::{ClientSenders, FederationPeerSenders, OutboundMsg};
        use tokio::sync::mpsc;

        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let (rt, space_id, bob_uri) = runtime_with_hosted_space_and_member();
        let sx = SpaceXgid::from_xgid(Xgid::new(space_id.clone()));

        // Add a federated peer to the Space and find a remaining member (alice).
        let peer = NodeXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:PEER".into()));
        let alice_x: IdentityXgid = {
            let mut g = rt.lock().await;
            let space = g.spaces.get_mut(&sx).unwrap();
            space.federation_nodes.push(peer.clone());
            space
                .members
                .keys()
                .find(|k| k.as_str() != bob_uri)
                .cloned()
                .expect("a non-target member (alice) is present")
        };

        // Register a client sender for alice and a federation sender for the peer.
        let (alice_tx, mut alice_rx) = mpsc::channel(16);
        let client_senders: ClientSenders =
            Arc::new(Mutex::new(std::collections::HashMap::new()));
        client_senders.lock().await.insert(alice_x.clone(), alice_tx);

        let (peer_tx, mut peer_rx) = mpsc::channel(16);
        let federation_senders: FederationPeerSenders =
            Arc::new(Mutex::new(std::collections::HashMap::new()));
        federation_senders.lock().await.insert(peer.clone(), peer_tx);

        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin")
            .with_runtime(Arc::clone(&rt))
            .with_client_senders(Arc::clone(&client_senders))
            .with_federation_senders(Arc::clone(&federation_senders));

        // Force-eject bob.
        let r = space_force_eject(
            &mut ctx,
            SpaceForceEjectArgs {
                space_id: space_id.clone(),
                identity_id: bob_uri.clone(),
                reason: None,
            },
        )
        .await
        .unwrap();

        // Remaining member alice got the node_eject live (matching event_id).
        match alice_rx.try_recv().expect("alice received the live node_eject") {
            OutboundMsg::Event(ev) => {
                assert_eq!(ev.event_type, EventType::MembershipNodeEject);
                assert_eq!(ev.event_id.as_ref().unwrap().as_str(), r.event_id);
            }
            other => panic!("expected Event, got {other:?}"),
        }
        // Federation peer got it too (LocallySubmitted → eligible per F-5).
        match peer_rx.try_recv().expect("peer received the live node_eject") {
            OutboundMsg::Event(ev) => {
                assert_eq!(ev.event_type, EventType::MembershipNodeEject);
            }
            other => panic!("expected Event, got {other:?}"),
        }

        // Unban bob → node_unban also fans out live to both surfaces.
        let u = space_unban(
            &mut ctx,
            SpaceUnbanArgs {
                space_id: space_id.clone(),
                identity_id: bob_uri.clone(),
                reason: None,
            },
        )
        .await
        .unwrap();
        match alice_rx.try_recv().expect("alice received the live node_unban") {
            OutboundMsg::Event(ev) => {
                assert_eq!(ev.event_type, EventType::MembershipNodeUnban);
                assert_eq!(ev.event_id.as_ref().unwrap().as_str(), u.event_id);
            }
            other => panic!("expected Event, got {other:?}"),
        }
        match peer_rx.try_recv().expect("peer received the live node_unban") {
            OutboundMsg::Event(ev) => {
                assert_eq!(ev.event_type, EventType::MembershipNodeUnban);
            }
            other => panic!("expected Event, got {other:?}"),
        }
    }

    /// Without the sender maps wired (Option-A baseline / file-only verbs), the
    /// verb still succeeds and persists — it just does not attempt a live push.
    #[tokio::test]
    async fn space_force_eject_without_senders_is_sync_only() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let (rt, space_id, bob_uri) = runtime_with_hosted_space_and_member();
        let mut ctx = AdminContext::batch(dir.path(), &cfg, "admin").with_runtime(rt);
        let r = space_force_eject(
            &mut ctx,
            SpaceForceEjectArgs {
                space_id,
                identity_id: bob_uri,
                reason: None,
            },
        )
        .await
        .unwrap();
        assert!(!r.event_id.is_empty());
    }

    // ── A2 auth-module registry verb tests (auth-module-registry arc, C3) ─────────

    /// A valid base64url Ed25519 verifying key for `--pubkey` (deterministic by
    /// seed; encoded via the canonical `crypto::encoding::encode`).
    fn valid_pubkey_b64(seed: u8) -> String {
        let sk = ed25519_dalek::SigningKey::from_bytes(&[seed; 32]);
        encoding::encode(sk.verifying_key().as_bytes())
    }

    fn am_ctx<'a>(
        dir: &'a Path,
        cfg: &'a Path,
        reg: &Arc<Mutex<AuthModuleRegistry>>,
    ) -> AdminContext<'a> {
        AdminContext::batch(dir, cfg, "admin").with_auth_module_registry(Arc::clone(reg))
    }

    #[tokio::test]
    async fn auth_module_register_then_list_round_trip_and_audit() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let reg = Arc::new(Mutex::new(AuthModuleRegistry::new()));
        let mut ctx = am_ctx(dir.path(), &cfg, &reg);

        let res = auth_module_register(
            &mut ctx,
            AuthModuleRegisterArgs {
                pubkey: valid_pubkey_b64(0x11),
                endpoint: "https://auth.example.com/verify".to_string(),
                tier: vec![2, 3],
            },
        )
        .await
        .unwrap();
        assert!(res.module_id.starts_with("xgen://pubkey/ed25519:"));
        assert_eq!(res.accepted_tiers, vec![2, 3]);

        // list reflects the live store; persisted to the canonical path.
        let listed = auth_module_list(&mut ctx).await.unwrap();
        assert_eq!(listed.modules.len(), 1);
        assert_eq!(listed.modules[0].module_id, res.module_id);
        assert!(!listed.modules[0].revoked);
        let on_disk = AuthModuleRegistry::load(&ctx.auth_module_registry_path()).unwrap();
        assert_eq!(on_disk.len(), 1);

        // register is WRITE → audited (exactly 1 entry); list is READ → not.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 1);
        assert_eq!(audit::recent_entries(&conn, 1).unwrap()[0].verb, "auth-module register");
    }

    #[tokio::test]
    async fn auth_module_revoke_marks_untrusted_but_still_listed() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let reg = Arc::new(Mutex::new(AuthModuleRegistry::new()));
        let mut ctx = am_ctx(dir.path(), &cfg, &reg);

        let reg_res = auth_module_register(
            &mut ctx,
            AuthModuleRegisterArgs {
                pubkey: valid_pubkey_b64(0x22),
                endpoint: "https://auth.example.com/verify".to_string(),
                tier: vec![2],
            },
        )
        .await
        .unwrap();

        // revoke (A2-D1 block-only) — retained + still listed, flagged revoked.
        auth_module_revoke(
            &mut ctx,
            AuthModuleRevokeArgs { module_id: reg_res.module_id.clone() },
        )
        .await
        .unwrap();
        let listed = auth_module_list(&mut ctx).await.unwrap();
        assert_eq!(listed.modules.len(), 1);
        assert!(listed.modules[0].revoked);
        assert!(listed.modules[0].revoked_at.is_some());

        // unknown id → AUTHMOD_6101.
        let err = auth_module_revoke(
            &mut ctx,
            AuthModuleRevokeArgs {
                module_id: "xgen://pubkey/ed25519:nope".to_string(),
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "AUTHMOD_6101");

        // register + revoke both WRITE → 2 audit entries.
        let conn = audit::open_audit_db(dir.path()).unwrap();
        assert_eq!(audit::entry_count(&conn).unwrap(), 2);
    }

    #[tokio::test]
    async fn auth_module_set_tiers_replaces_and_errors() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let reg = Arc::new(Mutex::new(AuthModuleRegistry::new()));
        let mut ctx = am_ctx(dir.path(), &cfg, &reg);

        let reg_res = auth_module_register(
            &mut ctx,
            AuthModuleRegisterArgs {
                pubkey: valid_pubkey_b64(0x33),
                endpoint: "https://auth.example.com/verify".to_string(),
                tier: vec![1],
            },
        )
        .await
        .unwrap();

        let res = auth_module_set_tiers(
            &mut ctx,
            AuthModuleSetTiersArgs {
                module_id: reg_res.module_id.clone(),
                tier: vec![3, 4],
            },
        )
        .await
        .unwrap();
        assert_eq!(res.accepted_tiers, vec![3, 4]);
        let listed = auth_module_list(&mut ctx).await.unwrap();
        assert_eq!(listed.modules[0].accepted_tiers, vec![3, 4]);

        // unknown id → AUTHMOD_6101.
        let unknown = auth_module_set_tiers(
            &mut ctx,
            AuthModuleSetTiersArgs {
                module_id: "xgen://pubkey/ed25519:nope".to_string(),
                tier: vec![1],
            },
        )
        .await
        .unwrap_err();
        assert_eq!(unknown.code, "AUTHMOD_6101");

        // invalid tier → AUTHMOD_6103 (rejected at validate, before the store).
        let bad_tier = auth_module_set_tiers(
            &mut ctx,
            AuthModuleSetTiersArgs {
                module_id: reg_res.module_id.clone(),
                tier: vec![9],
            },
        )
        .await
        .unwrap_err();
        assert_eq!(bad_tier.code, "AUTHMOD_6103");
    }

    #[tokio::test]
    async fn auth_module_register_rejects_malformed_pubkey() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let reg = Arc::new(Mutex::new(AuthModuleRegistry::new()));
        let mut ctx = am_ctx(dir.path(), &cfg, &reg);

        // Not a 32-byte key → AUTHMOD_6102 at Validate; nothing stored/audited.
        let err = auth_module_register(
            &mut ctx,
            AuthModuleRegisterArgs {
                pubkey: "QQ".to_string(),
                endpoint: "https://auth.example.com/verify".to_string(),
                tier: vec![2],
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "AUTHMOD_6102");
        assert!(auth_module_list(&mut ctx).await.unwrap().modules.is_empty());
    }

    /// Register a module pointing at `endpoint`, returning the derived module_id.
    async fn register_with_endpoint(ctx: &mut AdminContext<'_>, seed: u8, endpoint: &str) -> String {
        auth_module_register(
            ctx,
            AuthModuleRegisterArgs {
                pubkey: valid_pubkey_b64(seed),
                endpoint: endpoint.to_string(),
                tier: vec![2],
            },
        )
        .await
        .unwrap()
        .module_id
    }

    #[tokio::test]
    async fn auth_module_test_reachable_against_mock_listener() {
        // A bound TcpListener completes the connect handshake from the OS
        // backlog without an explicit accept(), so connectivity succeeds.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let reg = Arc::new(Mutex::new(AuthModuleRegistry::new()));
        let mut ctx = am_ctx(dir.path(), &cfg, &reg);
        let module_id = register_with_endpoint(&mut ctx, 0x44, &format!("http://{addr}/verify")).await;

        let r = auth_module_test(&mut ctx, AuthModuleTestArgs { module_id: module_id.clone() })
            .await
            .unwrap();
        assert!(r.reachable, "expected reachable, reason: {:?}", r.reason);
        assert!(r.response_time_ms.is_some());
        assert!(r.reason.is_none());
        assert_eq!(r.accepted_tiers, vec![2]); // stored tiers, display-only
        // READ → not audited (no audit db rows from a pure test).
    }

    #[tokio::test]
    async fn auth_module_test_unreachable_is_result_not_error() {
        // Bind then drop to obtain a port nothing listens on → connect refused.
        let addr = {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            l.local_addr().unwrap()
        };

        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let reg = Arc::new(Mutex::new(AuthModuleRegistry::new()));
        let mut ctx = am_ctx(dir.path(), &cfg, &reg);
        let module_id = register_with_endpoint(&mut ctx, 0x55, &format!("http://{addr}/")).await;

        // Unreachable is a RESULT (Ok), not an Err.
        let r = auth_module_test(&mut ctx, AuthModuleTestArgs { module_id })
            .await
            .unwrap();
        assert!(!r.reachable);
        assert!(r.response_time_ms.is_none());
        assert!(r.reason.is_some());
    }

    #[tokio::test]
    async fn auth_module_test_unknown_module_errors_6101() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("xgen-node_config.toml");
        let reg = Arc::new(Mutex::new(AuthModuleRegistry::new()));
        let mut ctx = am_ctx(dir.path(), &cfg, &reg);

        let err = auth_module_test(
            &mut ctx,
            AuthModuleTestArgs {
                module_id: "xgen://pubkey/ed25519:nope".to_string(),
            },
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "AUTHMOD_6101");
    }

    #[test]
    fn endpoint_host_port_parses_common_shapes() {
        assert_eq!(
            endpoint_host_port("https://auth.example.com/verify"),
            Some(("auth.example.com".to_string(), 443))
        );
        assert_eq!(
            endpoint_host_port("http://127.0.0.1:8443/x"),
            Some(("127.0.0.1".to_string(), 8443))
        );
        assert_eq!(
            endpoint_host_port("http://host.example"),
            Some(("host.example".to_string(), 80))
        );
        // Unknown scheme + no explicit port → unparseable (caller → unreachable).
        assert_eq!(endpoint_host_port("ftp://host.example/x"), None);
        assert_eq!(endpoint_host_port(""), None);
    }
}
