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
use std::path::Path;

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
}
