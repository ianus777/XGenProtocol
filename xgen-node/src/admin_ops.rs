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

use chrono::{SecondsFormat, Utc};
use rusqlite::Connection;
use serde::Serialize;

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
}

impl<'a> AdminContext<'a> {
    /// Build a `--batch`-originated admin context.
    pub fn batch(data_dir: &'a Path, config_path: &'a Path, actor: impl Into<String>) -> Self {
        Self {
            data_dir,
            config_path,
            actor: actor.into(),
            actor_via: ActorVia::Batch,
        }
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
}
