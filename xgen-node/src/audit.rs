// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Admin audit trail (M6 §2.6.4). A SQLite store at
//! `<data_dir>/xgen-node_audit.db` (D-035 convention, consistent with the
//! Identity and Federation registries' co-location). One table, `audit_entries`,
//! indexed on `timestamp`, `actor`, `verb`.
//!
//! This is the **SQLite admin trail** — distinct from the §3.11.8 protocol audit
//! log that `space audit-events` (Phase 9, A4) reads. Do not conflate the two.
//!
//! `args_hash` stores the sha256 of the canonical-JSON args rather than the args
//! themselves, so potentially-sensitive arguments (target identity IDs, etc.)
//! stay out of the trail while it remains verifiable: re-hash a candidate args
//! block and check for a match. Strict full-args non-repudiation is an opt-in
//! out of M6 scope.
//!
//! **Phase 2 ships the storage layer + entry-insertion API.** The table is
//! created empty on first Node start; no verbs write entries yet (those land per
//! category, Phases 3–10). The `audit query` / `audit export` verbs (Phase 4, A6)
//! extend the read surface below.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use xgen_core::crypto::hashing::sha256_hex;

/// Path to the audit DB for a given data directory (D-035).
pub fn audit_db_path(data_dir: &Path) -> PathBuf {
    data_dir.join("xgen-node_audit.db")
}

/// One row of the admin audit trail (§2.6.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// RFC 3339 UTC timestamp.
    pub timestamp: String,
    /// Verb name, e.g. `"federation accept"`, `"identity revoke"`.
    pub verb: String,
    /// `identity_id` URI of the initiating administrator.
    pub actor: String,
    /// `"batch"` | `"aicontrol"` (M7+) | `"cli-direct"`.
    pub actor_via: String,
    /// Verb-specific target (peer_node_id, identity_id, …).
    pub target: Option<String>,
    /// sha256 of canonical-JSON args (see module docs).
    pub args_hash: String,
    /// `"ok"` | `"error"`.
    pub outcome: String,
    /// e.g. `"FED_3041"`; `None` on success.
    pub error_code: Option<String>,
    /// Human message; `None` on success.
    pub error_message: Option<String>,
    /// For chaining related entries.
    pub correlation_id: Option<String>,
    /// JSON map for forward-compat; defaults to `"{}"`.
    pub meta_atts: String,
}

impl AuditEntry {
    /// sha256 hex of the canonical-JSON args (§2.6.4). Reuses the protocol's
    /// `sha256_hex` so the audit hash is the same primitive used elsewhere.
    pub fn compute_args_hash(canonical_args_json: &str) -> String {
        sha256_hex(canonical_args_json.as_bytes())
    }
}

const SCHEMA: &str = "\
CREATE TABLE IF NOT EXISTS audit_entries (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp      TEXT NOT NULL,
    verb           TEXT NOT NULL,
    actor          TEXT NOT NULL,
    actor_via      TEXT NOT NULL,
    target         TEXT,
    args_hash      TEXT NOT NULL,
    outcome        TEXT NOT NULL,
    error_code     TEXT,
    error_message  TEXT,
    correlation_id TEXT,
    meta_atts      TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_entries(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_entries(actor);
CREATE INDEX IF NOT EXISTS idx_audit_verb ON audit_entries(verb);
";

/// Open (creating if absent) the audit DB and ensure the schema exists. Called on
/// Node start so the table exists — empty — before any verb writes (§5.2 item 5).
pub fn open_audit_db(data_dir: &Path) -> Result<Connection> {
    let path = audit_db_path(data_dir);
    let conn = Connection::open(&path)
        .with_context(|| format!("opening audit db at {}", path.display()))?;
    conn.execute_batch(SCHEMA)
        .context("initialising audit_entries schema")?;
    Ok(conn)
}

/// Insert one audit entry. Verbs call this after completing (ok or error).
pub fn insert_entry(conn: &Connection, entry: &AuditEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_entries
            (timestamp, verb, actor, actor_via, target, args_hash, outcome,
             error_code, error_message, correlation_id, meta_atts)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            entry.timestamp,
            entry.verb,
            entry.actor,
            entry.actor_via,
            entry.target,
            entry.args_hash,
            entry.outcome,
            entry.error_code,
            entry.error_message,
            entry.correlation_id,
            entry.meta_atts,
        ],
    )
    .context("inserting audit entry")?;
    Ok(())
}

/// Read the most recent `limit` entries, newest first. Minimal read surface for
/// the Phase 2 skeleton; `audit query` / `audit export` (Phase 4, A6) extend it
/// with filters and JSONL materialisation.
pub fn recent_entries(conn: &Connection, limit: usize) -> Result<Vec<AuditEntry>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, verb, actor, actor_via, target, args_hash, outcome,
                error_code, error_message, correlation_id, meta_atts
         FROM audit_entries ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit as i64], |row| {
        Ok(AuditEntry {
            timestamp: row.get(0)?,
            verb: row.get(1)?,
            actor: row.get(2)?,
            actor_via: row.get(3)?,
            target: row.get(4)?,
            args_hash: row.get(5)?,
            outcome: row.get(6)?,
            error_code: row.get(7)?,
            error_message: row.get(8)?,
            correlation_id: row.get(9)?,
            meta_atts: row.get(10)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Count of entries — convenience for "empty on first start" checks.
pub fn entry_count(conn: &Connection) -> Result<usize> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM audit_entries", [], |r| r.get(0))?;
    Ok(n as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample(verb: &str) -> AuditEntry {
        AuditEntry {
            timestamp: "2026-05-29T12:00:00.000Z".to_string(),
            verb: verb.to_string(),
            actor: "xgen://pubkey/ed25519:node".to_string(),
            actor_via: "batch".to_string(),
            target: Some("xgen://pubkey/ed25519:peer".to_string()),
            args_hash: AuditEntry::compute_args_hash(r#"{"peer":"x"}"#),
            outcome: "ok".to_string(),
            error_code: None,
            error_message: None,
            correlation_id: None,
            meta_atts: "{}".to_string(),
        }
    }

    #[test]
    fn open_creates_empty_table_on_first_start() {
        let dir = tempdir().unwrap();
        let conn = open_audit_db(dir.path()).unwrap();
        assert_eq!(entry_count(&conn).unwrap(), 0);
        // DB file materialised at the D-035 path.
        assert!(audit_db_path(dir.path()).exists());
    }

    #[test]
    fn insert_and_read_back_round_trip() {
        let dir = tempdir().unwrap();
        let conn = open_audit_db(dir.path()).unwrap();
        let entry = sample("federation accept");
        insert_entry(&conn, &entry).unwrap();
        assert_eq!(entry_count(&conn).unwrap(), 1);
        let got = recent_entries(&conn, 10).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], entry);
    }

    #[test]
    fn recent_returns_newest_first_and_respects_limit() {
        let dir = tempdir().unwrap();
        let conn = open_audit_db(dir.path()).unwrap();
        insert_entry(&conn, &sample("a")).unwrap();
        insert_entry(&conn, &sample("b")).unwrap();
        insert_entry(&conn, &sample("c")).unwrap();
        let got = recent_entries(&conn, 2).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].verb, "c"); // newest first
        assert_eq!(got[1].verb, "b");
    }

    #[test]
    fn schema_survives_reopen() {
        let dir = tempdir().unwrap();
        {
            let conn = open_audit_db(dir.path()).unwrap();
            insert_entry(&conn, &sample("federation accept")).unwrap();
        }
        // Reopen — schema is idempotent (CREATE IF NOT EXISTS), data persists.
        let conn = open_audit_db(dir.path()).unwrap();
        assert_eq!(entry_count(&conn).unwrap(), 1);
    }

    #[test]
    fn args_hash_is_deterministic_sha256() {
        let h1 = AuditEntry::compute_args_hash(r#"{"a":1}"#);
        let h2 = AuditEntry::compute_args_hash(r#"{"a":1}"#);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // sha256 hex
        assert_ne!(h1, AuditEntry::compute_args_hash(r#"{"a":2}"#));
    }
}
