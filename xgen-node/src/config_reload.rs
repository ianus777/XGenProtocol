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

use std::path::Path;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::app::{LogSetError, NodeConfig};

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
    // `listen` is bound once at startup; `asserts_tier` is evaluated by the
    // storage-engine tier gate only at startup (SE-D4 — a live store-swap is
    // unsound, so the gate input cannot change live either).
    if old.node.listen != new.node.listen {
        plan.restart_required.push("node.listen".to_string());
    }
    if old.node.local_mode != new.node.local_mode {
        plan.restart_required.push("node.local_mode".to_string());
    }
    if old.node.asserts_tier != new.node.asserts_tier {
        plan.restart_required.push("node.asserts_tier".to_string());
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

    // [storage] — restart-required (Storage-Engine milestone, SE-D5). The engine
    // selection feeds the startup tier gate; swapping the live store mid-run is
    // unsound, so a change reports pending-restart (the new value is already on
    // disk from the operator's edit).
    if old.storage != new.storage {
        plan.restart_required.push("storage".to_string());
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

/// Execute one `__RELOAD_CONFIG__` cycle and return the report line
/// (M7S-D2/D3/D4). Called by the pipe handler (`pipe.rs`).
///
/// Steps: re-read disk → all-or-nothing gate (parse + `[node].listen`
/// SocketAddr) → diff/classify vs the running baseline → apply `[logging].level`
/// live (A6-D1) and update the baseline snapshot → return the structured report.
/// No config write-back (F-R2 — restart-required values are already on disk from
/// the operator's edit; a full re-serialise would destroy their comments).
pub async fn handle_reload(config_path: &Path, snapshot: &Arc<Mutex<NodeConfig>>) -> String {
    handle_reload_inner(config_path, snapshot, |level| {
        crate::app::apply_log_set_level(None, level).map(|_| ())
    })
    .await
}

/// Inner core with the live `[logging].level` apply injected. Splitting the
/// apply out keeps the gate + classify + snapshot-update logic testable without
/// an initialised tracing subscriber (the real `LOG_RELOAD` handle is
/// process-global and only installed in `run_node`).
///
/// **Snapshot update rule (the CP-1 baseline, M7S-D4 no-lie):** on success, a
/// **reloadable** field updates the snapshot (so a later reload sees the new
/// running value); a **restart-required** field does **not** — the snapshot
/// stays equal to what is *actually running*, not what is on disk. The honest
/// consequence is that while disk diverges from the running value the field is
/// re-reported `PENDING_RESTART` on every reload (true until the operator
/// restarts), and an edit-then-revert back to the running value reports no
/// change (no false pending-restart).
async fn handle_reload_inner<F>(
    config_path: &Path,
    snapshot: &Arc<Mutex<NodeConfig>>,
    apply_log: F,
) -> String
where
    F: FnOnce(&str) -> Result<(), LogSetError>,
{
    // Re-read disk (M7S-D2). Parse failure → reject the whole reload, apply
    // nothing (the all-or-nothing gate, part 1).
    let new = match crate::app::try_load_config(config_path) {
        Some(c) => c,
        None => return format_reject("config", "parse"),
    };
    // All-or-nothing gate, part 2 (M7S-D3): the `[node].listen` SocketAddr
    // semantic check. On failure, apply nothing.
    if !listen_addr_valid(&new.node.listen) {
        return format_reject("node.listen", "semantic");
    }

    let mut snap = snapshot.lock().await;
    let plan = reload_plan(&snap, &new);

    // Apply the one reloadable field live (`[logging].level`). On success update
    // the snapshot; restart-required + N/A fields are reported but never applied
    // and never update the snapshot (see the update rule above).
    if plan.reloadable.iter().any(|f| f == "logging.level") {
        match apply_log(&new.logging.level) {
            Ok(()) => snap.logging.level = new.logging.level.clone(),
            Err(e) => {
                let reason = match e {
                    // No reload handle in this run mode (e.g. the desktop shell
                    // installed its own subscriber): can't apply live here.
                    LogSetError::NoHandle => "unknown",
                    // Invalid level / unsettable directive.
                    _ => "semantic",
                };
                return format_reject("logging.level", reason);
            }
        }
    }

    format_reload_report(&plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        BootstrapSection, FederationSection, LoggingSection, NodeConfig, NodeSection, PathsSection,
        StorageSection, SyncSection,
    };

    /// A known-good baseline config to diff against.
    fn base() -> NodeConfig {
        NodeConfig {
            node: NodeSection {
                listen: "ws://127.0.0.1:8080/xgen".to_string(),
                local_mode: true,
                asserts_tier: None,
                trusted_auth_modules: Vec::new(),
            },
            paths: PathsSection {
                keypair_path: "kp.enc".to_string(),
                spaces_dir: Some("spaces".to_string()),
                blobs_dir: Some("blobs".to_string()),
            },
            logging: LoggingSection {
                level: "debug".to_string(),
            },
            sync: SyncSection::default(),
            federation: FederationSection::default(),
            bootstrap: BootstrapSection::default(),
            storage: StorageSection::default(),
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
    fn storage_and_asserts_tier_are_restart_required() {
        // SE-D4/SE-D5 — both feed the startup tier gate; a live store-swap is
        // unsound, so each reports pending-restart, never reloaded.
        let mut new = base();
        new.node.asserts_tier = Some(3);
        new.storage.storage_engine = Some("sqlite".to_string());
        let plan = reload_plan(&base(), &new);
        assert!(plan
            .restart_required
            .contains(&"node.asserts_tier".to_string()));
        assert!(plan.restart_required.contains(&"storage".to_string()));
        assert!(plan.reloadable.is_empty());
        assert!(plan.not_applicable.is_empty());
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

    // ── handle_reload (Commit 2) — gate + classify + snapshot update ──────────
    //
    // The live `[logging].level` apply is injected (the real `LOG_RELOAD` handle
    // is process-global and only set in `run_node`), so these exercise the full
    // re-read → gate → diff → snapshot-update path deterministically.

    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn write_config(dir: &std::path::Path, cfg: &NodeConfig) -> std::path::PathBuf {
        let path = dir.join("xgen-node_config.toml");
        std::fs::write(&path, toml::to_string(cfg).unwrap()).unwrap();
        path
    }

    #[tokio::test]
    async fn handler_rejects_unparseable_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("xgen-node_config.toml");
        std::fs::write(&path, "this is = = not valid toml [[[").unwrap();
        let snap = Arc::new(Mutex::new(base()));
        let report = handle_reload_inner(&path, &snap, |_| Ok(())).await;
        assert_eq!(report, "REJECTED: config=parse");
        // Nothing applied.
        assert_eq!(snap.lock().await.logging.level, "debug");
    }

    #[tokio::test]
    async fn handler_rejects_bad_listen_applies_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let mut new = base();
        new.node.listen = "garbage".to_string();
        new.logging.level = "info".to_string(); // would-be reloadable, must NOT apply
        let path = write_config(dir.path(), &new);
        let snap = Arc::new(Mutex::new(base()));
        let mut applied = false;
        let report = handle_reload_inner(&path, &snap, |_| {
            applied = true;
            Ok(())
        })
        .await;
        assert_eq!(report, "REJECTED: node.listen=semantic");
        assert!(!applied, "gate failure must apply nothing");
        assert_eq!(snap.lock().await.logging.level, "debug");
    }

    #[tokio::test]
    async fn handler_applies_logging_and_updates_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let mut new = base();
        new.logging.level = "info".to_string();
        let path = write_config(dir.path(), &new);
        let snap = Arc::new(Mutex::new(base()));
        let report = handle_reload_inner(&path, &snap, |lvl| {
            assert_eq!(lvl, "info");
            Ok(())
        })
        .await;
        assert_eq!(report, "RELOADED: logging.level");
        assert_eq!(snap.lock().await.logging.level, "info");
    }

    #[tokio::test]
    async fn handler_rejects_when_logging_apply_fails() {
        let dir = tempfile::tempdir().unwrap();
        let mut new = base();
        new.logging.level = "bogus".to_string();
        let path = write_config(dir.path(), &new);
        let snap = Arc::new(Mutex::new(base()));
        let report =
            handle_reload_inner(&path, &snap, |_| Err(LogSetError::InvalidLevel)).await;
        assert_eq!(report, "REJECTED: logging.level=semantic");
        // Snapshot unchanged on a failed apply.
        assert_eq!(snap.lock().await.logging.level, "debug");
    }

    #[tokio::test]
    async fn handler_reports_restart_required_without_applying_or_updating_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let mut new = base();
        new.node.local_mode = false;
        let path = write_config(dir.path(), &new);
        let snap = Arc::new(Mutex::new(base()));
        let report = handle_reload_inner(&path, &snap, |_| Ok(())).await;
        assert_eq!(report, "PENDING_RESTART: node.local_mode");
        // Snapshot stays = running (M7S-D4 update rule).
        assert!(snap.lock().await.node.local_mode);
    }

    #[tokio::test]
    async fn handler_reports_bootstrap_na() {
        let dir = tempfile::tempdir().unwrap();
        let mut new = base();
        new.bootstrap.region = "eu-west".to_string();
        let path = write_config(dir.path(), &new);
        let snap = Arc::new(Mutex::new(base()));
        let report = handle_reload_inner(&path, &snap, |_| Ok(())).await;
        assert_eq!(report, "NA: bootstrap");
    }

    #[tokio::test]
    async fn handler_rerun_keeps_reporting_restart_required_snapshot_stable() {
        // Design A (snapshot = running): while disk diverges from the running
        // value, every reload honestly re-reports PENDING_RESTART — the snapshot
        // never drifts to the on-disk value, so no false report can arise on a
        // later edit-then-revert.
        let dir = tempfile::tempdir().unwrap();
        let mut new = base();
        new.node.local_mode = false;
        let path = write_config(dir.path(), &new);
        let snap = Arc::new(Mutex::new(base()));
        let r1 = handle_reload_inner(&path, &snap, |_| Ok(())).await;
        let r2 = handle_reload_inner(&path, &snap, |_| Ok(())).await;
        assert_eq!(r1, "PENDING_RESTART: node.local_mode");
        assert_eq!(r2, "PENDING_RESTART: node.local_mode");
        assert!(snap.lock().await.node.local_mode);
    }

    #[tokio::test]
    async fn handler_no_change_reports_ok() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_config(dir.path(), &base());
        let snap = Arc::new(Mutex::new(base()));
        let report = handle_reload_inner(&path, &snap, |_| Ok(())).await;
        assert_eq!(report, "OK: no changes");
    }
}
