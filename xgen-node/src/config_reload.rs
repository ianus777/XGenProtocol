// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M7-standalone — live config reload (`--reload-config`).
//!
//! This module is the mechanism behind the `__RELOAD_CONFIG__` pipe control
//! token (`pipe.rs`), which returned a `NOT_IMPLEMENTED` stub through M2. The
//! shape (design `tasks/M7_STANDALONE_DESIGN.md` v1.3, M7S-D1…D6):
//!
//!   re-read disk → all-or-nothing gate → diff/classify → apply `[logging].level`
//!   live → return a structured report line.
//!
//! **Node-only v1** (M7S-D5); **legacy pipe surface only** (M7S-D2 — not an
//! AI-driver verb). `--batch` / `--aicontrol` are untouched (D-066).
//!
//! ## What is reloadable vs not (M7S-D6, classified on a read-pattern trace)
//!
//! - `[logging].level` → **Reloadable** live via the `LOG_RELOAD` reload handle
//!   (A6-D1). The one field applied in v1.
//! - `[node].listen` / `[node].local_mode` / `[paths].*` / `[sync].*` /
//!   `[federation].require_approval` → **Restart-required**: read once at startup
//!   and threaded by value into the running loops; a live flip would desync state
//!   (most pointedly `local_mode`, which gates trust admission — M7S-D1). The new
//!   value is **already on disk** (the operator edits `config.toml` *before*
//!   running `--reload-config`), so reload writes nothing back (F-R2 — a full
//!   re-serialise would destroy operator comments) and reports it pending-restart.
//! - `[bootstrap]` → **N/A / seed-only**: the JSON store (`xgen-node_bootstrap.json`)
//!   is truth; config only seeds it on first run, so a config edit has no live
//!   effect *and* a restart won't apply it either — reporting it pending-restart
//!   would be a lie. Reported `NA`.
//!
//! ## No-lie report contract (M7S-D1/D4, load-bearing)
//!
//! A parsed-but-restart-required field surfaces as `PENDING_RESTART` (never
//! silently dropped); a seed-only field surfaces as `NA` (never pending-restart);
//! a `REJECTED` line **always carries a reason** (`parse` / `semantic` /
//! `unknown`). The report is one human-readable control line (matches
//! `__HEALTH__`; not JSON), listing only changed deltas.

use crate::app::NodeConfig;

/// Per-field dispositions for a single reload (the changed deltas only).
///
/// Produced by [`reload_plan`] after the all-or-nothing gate has passed. A gate
/// failure is reported separately via [`format_reject`] — it never produces a
/// plan (apply nothing, report `REJECTED`).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ReloadPlan {
    /// Changed reloadable fields — applied live and reported `RELOADED`.
    pub reloadable: Vec<String>,
    /// Changed restart-required fields — reported `PENDING_RESTART`, not applied.
    pub restart_required: Vec<String>,
    /// Changed seed-only fields — reported `NA`, neither applied nor persisted.
    pub not_applicable: Vec<String>,
}

impl ReloadPlan {
    /// No field changed in any category.
    pub fn is_empty(&self) -> bool {
        self.reloadable.is_empty()
            && self.restart_required.is_empty()
            && self.not_applicable.is_empty()
    }
}

/// Diff the re-read config (`new`) against the running baseline (`old`) and bin
/// each changed field by its M7S-D6 disposition. Pure — no I/O, no globals.
pub fn reload_plan(old: &NodeConfig, new: &NodeConfig) -> ReloadPlan {
    let mut plan = ReloadPlan::default();

    // [logging].level — the one reloadable field (A6-D1 LOG_RELOAD path).
    if old.logging.level != new.logging.level {
        plan.reloadable.push("logging.level".to_string());
    }

    // [node].* — restart-required. `local_mode` gates trust admission (M7S-D1);
    // `listen` is bound once at startup.
    if old.node.listen != new.node.listen {
        plan.restart_required.push("node.listen".to_string());
    }
    if old.node.local_mode != new.node.local_mode {
        plan.restart_required.push("node.local_mode".to_string());
    }

    // [paths].* — restart-required (keypair loaded + spaces replayed at startup).
    if old.paths.keypair_path != new.paths.keypair_path {
        plan.restart_required.push("paths.keypair_path".to_string());
    }
    if old.paths.spaces_dir != new.paths.spaces_dir {
        plan.restart_required.push("paths.spaces_dir".to_string());
    }

    // [sync].* — restart-required (frozen-at-startup tuning, M7S-D6 (b)).
    // Enumerate sub-fields so the report names the precise change.
    if old.sync.completion_timeout_seconds != new.sync.completion_timeout_seconds {
        plan.restart_required
            .push("sync.completion_timeout_seconds".to_string());
    }
    if old.sync.batch_size != new.sync.batch_size {
        plan.restart_required.push("sync.batch_size".to_string());
    }
    if old.sync.federation_relationship_timeout_seconds
        != new.sync.federation_relationship_timeout_seconds
    {
        plan.restart_required
            .push("sync.federation_relationship_timeout_seconds".to_string());
    }

    // [federation].require_approval — restart-required (read once at startup,
    // M7S-D6 (b); traced independently from local_mode).
    if old.federation.require_approval != new.federation.require_approval {
        plan.restart_required
            .push("federation.require_approval".to_string());
    }

    // [bootstrap] — N/A / seed-only (store is truth, M7S-D6 (c)). Compared as a
    // whole section; a change in any sub-field reports the single `bootstrap`
    // token — never pending-restart (a restart won't apply it either).
    if old.bootstrap != new.bootstrap {
        plan.not_applicable.push("bootstrap".to_string());
    }

    plan
}

/// The `[node].listen` `SocketAddr` semantic check (M7S-D3) used as part of the
/// all-or-nothing gate. Returns true if `listen` parses to a bindable
/// `ws://host:port/path` address.
pub fn listen_addr_valid(listen: &str) -> bool {
    crate::app::listen_addr_is_valid(listen)
}

/// Format the success-path report (M7S-D4): one control line listing only the
/// changed deltas. Empty plan → `OK: no changes`.
pub fn format_reload_report(plan: &ReloadPlan) -> String {
    if plan.is_empty() {
        return "OK: no changes".to_string();
    }
    let mut segs: Vec<String> = Vec::new();
    if !plan.reloadable.is_empty() {
        segs.push(format!("RELOADED: {}", plan.reloadable.join(", ")));
    }
    if !plan.restart_required.is_empty() {
        segs.push(format!("PENDING_RESTART: {}", plan.restart_required.join(", ")));
    }
    if !plan.not_applicable.is_empty() {
        segs.push(format!("NA: {}", plan.not_applicable.join(", ")));
    }
    segs.join("; ")
}

/// Format a gate-rejection line (M7S-D4): `REJECTED` always carries a reason
/// (`parse` / `semantic` / `unknown`). A reasonless rejection is the same
/// silent-lie M7S-D1 forbids.
pub fn format_reject(field: &str, reason: &str) -> String {
    format!("REJECTED: {field}={reason}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        BootstrapSection, FederationSection, LoggingSection, NodeConfig, NodeSection, PathsSection,
        SyncSection,
    };

    /// A known-good baseline config to diff against.
    fn base() -> NodeConfig {
        NodeConfig {
            node: NodeSection {
                listen: "ws://127.0.0.1:8080/xgen".to_string(),
                local_mode: true,
            },
            paths: PathsSection {
                keypair_path: "kp.enc".to_string(),
                spaces_dir: Some("spaces".to_string()),
            },
            logging: LoggingSection {
                level: "debug".to_string(),
            },
            sync: SyncSection::default(),
            federation: FederationSection::default(),
            bootstrap: BootstrapSection::default(),
        }
    }

    #[test]
    fn no_change_yields_empty_plan_and_ok_line() {
        let plan = reload_plan(&base(), &base());
        assert!(plan.is_empty());
        assert_eq!(format_reload_report(&plan), "OK: no changes");
    }

    #[test]
    fn logging_level_is_reloadable() {
        let mut new = base();
        new.logging.level = "info".to_string();
        let plan = reload_plan(&base(), &new);
        assert_eq!(plan.reloadable, vec!["logging.level".to_string()]);
        assert!(plan.restart_required.is_empty());
        assert!(plan.not_applicable.is_empty());
        assert_eq!(format_reload_report(&plan), "RELOADED: logging.level");
    }

    #[test]
    fn local_mode_is_restart_required_not_silently_ignored() {
        // The load-bearing M7S-D1 no-lie case: an edited `local_mode` MUST
        // surface as pending-restart, never silently dropped.
        let mut new = base();
        new.node.local_mode = false;
        let plan = reload_plan(&base(), &new);
        assert_eq!(plan.restart_required, vec!["node.local_mode".to_string()]);
        assert!(plan.reloadable.is_empty());
        assert_eq!(format_reload_report(&plan), "PENDING_RESTART: node.local_mode");
    }

    #[test]
    fn listen_is_restart_required() {
        let mut new = base();
        new.node.listen = "ws://127.0.0.1:9090/xgen".to_string();
        let plan = reload_plan(&base(), &new);
        assert_eq!(plan.restart_required, vec!["node.listen".to_string()]);
    }

    #[test]
    fn paths_are_restart_required() {
        let mut new = base();
        new.paths.keypair_path = "other.enc".to_string();
        new.paths.spaces_dir = Some("other_spaces".to_string());
        let plan = reload_plan(&base(), &new);
        assert_eq!(
            plan.restart_required,
            vec![
                "paths.keypair_path".to_string(),
                "paths.spaces_dir".to_string(),
            ]
        );
    }

    #[test]
    fn sync_subfields_are_restart_required_and_named() {
        let mut new = base();
        new.sync.batch_size = 500;
        new.sync.completion_timeout_seconds = 10;
        let plan = reload_plan(&base(), &new);
        assert!(plan
            .restart_required
            .contains(&"sync.batch_size".to_string()));
        assert!(plan
            .restart_required
            .contains(&"sync.completion_timeout_seconds".to_string()));
        assert!(!plan
            .restart_required
            .contains(&"sync.federation_relationship_timeout_seconds".to_string()));
    }

    #[test]
    fn federation_require_approval_is_restart_required() {
        let mut new = base();
        new.federation.require_approval = true;
        let plan = reload_plan(&base(), &new);
        assert_eq!(
            plan.restart_required,
            vec!["federation.require_approval".to_string()]
        );
    }

    #[test]
    fn bootstrap_is_na_never_pending_restart() {
        // The load-bearing M7S-D6 (c) no-lie case: a `[bootstrap]` edit is
        // seed-only — it must report NA, never PENDING_RESTART.
        let mut new = base();
        new.bootstrap.region = "eu-west".to_string();
        let plan = reload_plan(&base(), &new);
        assert_eq!(plan.not_applicable, vec!["bootstrap".to_string()]);
        assert!(plan.restart_required.is_empty());
        assert_eq!(format_reload_report(&plan), "NA: bootstrap");
    }

    #[test]
    fn mixed_change_lists_all_segments_in_order() {
        let mut new = base();
        new.logging.level = "trace".to_string();
        new.node.local_mode = false;
        new.bootstrap.region = "eu-west".to_string();
        let plan = reload_plan(&base(), &new);
        assert_eq!(
            format_reload_report(&plan),
            "RELOADED: logging.level; PENDING_RESTART: node.local_mode; NA: bootstrap"
        );
    }

    #[test]
    fn listen_addr_valid_accepts_ws_and_rejects_garbage() {
        assert!(listen_addr_valid("ws://127.0.0.1:8080/xgen"));
        assert!(listen_addr_valid("wss://127.0.0.1:8443/xgen"));
        assert!(!listen_addr_valid("not-a-url"));
        assert!(!listen_addr_valid("ws://host-without-port/xgen"));
    }

    #[test]
    fn reject_always_carries_reason() {
        assert_eq!(format_reject("config", "parse"), "REJECTED: config=parse");
        assert_eq!(
            format_reject("node.listen", "semantic"),
            "REJECTED: node.listen=semantic"
        );
    }
}
