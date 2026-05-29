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
use std::sync::Arc;

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::Connection;
use serde::Serialize;
use tokio::sync::Mutex;
use xgen_common::xgid::{IdentityXgid, NodeXgid, Xgid};
use xgen_core::federation::registry::{FederationRegistry, FederationRelationship};
use xgen_core::identity::registry::{IdentityRecord, RegistryError};
use xgen_core::node::runtime::NodeRuntime;

use crate::audit::{self, AuditEntry, AuditQueryFilter};

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
    /// `active | pending | revoked | all` (default `all`). Honest-subset note:
    /// the registry has no state field — a recorded relationship is active — so
    /// `pending`/`revoked` are accepted but always match zero (no such state is
    /// tracked in M6).
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
    // active/all → all recorded relationships; pending/revoked → none (no such
    // state exists in M6, A1 honest-subset); anything else → FED_3001.
    let want_active = match args.state.as_deref() {
        None | Some("all") | Some("active") => true,
        Some("pending") | Some("revoked") => false,
        Some(other) => {
            return Err(AdminError::new(
                "FED_3001",
                Stage::Validate,
                format!("invalid state filter '{other}' (expected active|pending|revoked|all)"),
            ));
        }
    };
    let limit = args
        .limit
        .unwrap_or(FED_LIST_DEFAULT_LIMIT)
        .min(FED_LIST_MAX_LIMIT);

    let registry = Arc::clone(ctx.require_federation_registry(Stage::Register)?);
    let mut matched: Vec<FederationRelationship> = if want_active {
        let reg = registry.lock().await;
        let mut v: Vec<FederationRelationship> = reg.all().into_iter().cloned().collect();
        v.sort_by(|a, b| a.peer_node_id.as_str().cmp(b.peer_node_id.as_str()));
        v
    } else {
        Vec::new()
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

/// `federation` sub-verbs (A1). M6 honest-subset ships `list` + `defederate`;
/// `accept`/`reject`/`initiate`/`set-policy`/`show-policy` defer to the
/// federation-admin-control subsystem arc (D-071, J-156).
#[derive(Debug, clap::Subcommand)]
pub enum FederationCommand {
    /// `federation list` — paginated read of federation relationships.
    List(FederationListArgs),
    /// `federation defederate` — terminate a federation relationship.
    Defederate(FederationDefederateArgs),
}

/// `space` sub-verbs (A4). M6 honest-subset ships `list-hosted` only;
/// `audit-events` (unbuilt §3.11.8 protocol log), `force-eject` (A4-D1
/// design-gated), and the node-policy verbs defer to their arcs (J-156).
#[derive(Debug, clap::Subcommand)]
pub enum SpaceCommand {
    /// `space list-hosted` — list Spaces this Node hosts.
    ListHosted(SpaceListHostedArgs),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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

        // Honest-subset: pending/revoked match nothing (no such state tracked).
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
}
