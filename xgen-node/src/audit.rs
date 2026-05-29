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
use rusqlite::{params, params_from_iter, Connection, Row};
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

/// The column list selected by every read path, in `map_row` order.
const SELECT_COLS: &str = "timestamp, verb, actor, actor_via, target, args_hash, \
outcome, error_code, error_message, correlation_id, meta_atts";

/// Map a row selected via `SELECT_COLS` into an `AuditEntry`.
fn map_row(row: &Row) -> rusqlite::Result<AuditEntry> {
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
}

/// Read the most recent `limit` entries, newest first.
pub fn recent_entries(conn: &Connection, limit: usize) -> Result<Vec<AuditEntry>> {
    let sql = format!("SELECT {SELECT_COLS} FROM audit_entries ORDER BY id DESC LIMIT {limit}");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
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

/// Filter for `audit query` / `audit export` (§6.A6). Timestamps are RFC 3339
/// strings; comparison is lexicographic, which is correct for the canonical UTC
/// `…Z` millis form the codebase emits (`to_rfc3339_opts(Millis, true)`). The
/// caller validates timestamp/outcome shape and clamps `limit` (default 100,
/// cap 1000) before building this.
#[derive(Debug, Clone, Default)]
pub struct AuditQueryFilter {
    pub actor: Option<String>,
    pub verb: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub outcome: Option<String>,
    pub limit: usize,
}

impl AuditQueryFilter {
    /// Build the `WHERE` clause + its positional string params from the set
    /// filters. Shared by `query` (paged) and `export` (full).
    fn where_and_params(&self) -> (String, Vec<String>) {
        let mut clauses: Vec<&str> = Vec::new();
        let mut vals: Vec<String> = Vec::new();
        if let Some(a) = &self.actor {
            clauses.push("actor = ?");
            vals.push(a.clone());
        }
        if let Some(v) = &self.verb {
            clauses.push("verb = ?");
            vals.push(v.clone());
        }
        if let Some(s) = &self.since {
            clauses.push("timestamp >= ?");
            vals.push(s.clone());
        }
        if let Some(u) = &self.until {
            clauses.push("timestamp <= ?");
            vals.push(u.clone());
        }
        if let Some(o) = &self.outcome {
            clauses.push("outcome = ?");
            vals.push(o.clone());
        }
        let where_sql = if clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", clauses.join(" AND "))
        };
        (where_sql, vals)
    }
}

/// `audit query` (READ) — filtered, newest-first, `limit`-capped read.
/// Returns `(page, total_matched)`: `page` is at most `filter.limit` rows,
/// `total_matched` is the count ignoring the limit. `limit` is embedded as an
/// integer literal (a validated `usize`, no injection surface).
pub fn query(conn: &Connection, filter: &AuditQueryFilter) -> Result<(Vec<AuditEntry>, usize)> {
    let (where_sql, vals) = filter.where_and_params();

    let count_sql = format!("SELECT COUNT(*) FROM audit_entries{where_sql}");
    let total: i64 =
        conn.query_row(&count_sql, params_from_iter(vals.iter()), |r| r.get(0))?;

    let sel_sql = format!(
        "SELECT {SELECT_COLS} FROM audit_entries{where_sql} ORDER BY id DESC LIMIT {}",
        filter.limit
    );
    let mut stmt = conn.prepare(&sel_sql)?;
    let rows = stmt.query_map(params_from_iter(vals.iter()), map_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok((out, total as usize))
}

/// Rows with `timestamp < before`, oldest first (for `audit archive` selection).
pub fn entries_before(conn: &Connection, before: &str) -> Result<Vec<AuditEntry>> {
    let sql =
        format!("SELECT {SELECT_COLS} FROM audit_entries WHERE timestamp < ?1 ORDER BY id ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([before], map_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Delete rows with `timestamp < before` (the `audit archive` prune step).
/// Returns the number of rows removed.
pub fn delete_before(conn: &Connection, before: &str) -> Result<usize> {
    let n = conn.execute("DELETE FROM audit_entries WHERE timestamp < ?1", [before])?;
    Ok(n)
}

/// Write entries as JSONL (one `AuditEntry` JSON object per line) to `path`,
/// creating parent directories as needed. Shared by `audit archive` and
/// `audit export`.
pub fn write_jsonl(entries: &[AuditEntry], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating audit output dir {}", parent.display()))?;
        }
    }
    let mut buf = String::new();
    for e in entries {
        buf.push_str(&serde_json::to_string(e).context("serialising audit entry")?);
        buf.push('\n');
    }
    std::fs::write(path, buf).with_context(|| format!("writing audit output {}", path.display()))?;
    Ok(())
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

    fn at(ts: &str, verb: &str, actor: &str, outcome: &str) -> AuditEntry {
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

    fn seeded() -> (tempfile::TempDir, Connection) {
        let dir = tempdir().unwrap();
        let conn = open_audit_db(dir.path()).unwrap();
        insert_entry(&conn, &at("2026-05-01T00:00:00.000Z", "federation accept", "alice", "ok")).unwrap();
        insert_entry(&conn, &at("2026-05-10T00:00:00.000Z", "identity revoke", "bob", "error")).unwrap();
        insert_entry(&conn, &at("2026-05-20T00:00:00.000Z", "federation accept", "alice", "ok")).unwrap();
        (dir, conn)
    }

    #[test]
    fn query_filters_by_actor_verb_outcome_and_window() {
        let (_d, conn) = seeded();
        let f = AuditQueryFilter { actor: Some("alice".into()), limit: 100, ..Default::default() };
        let (page, total) = query(&conn, &f).unwrap();
        assert_eq!(total, 2);
        assert_eq!(page.len(), 2);
        assert_eq!(page[0].timestamp, "2026-05-20T00:00:00.000Z"); // newest first

        let f = AuditQueryFilter { verb: Some("identity revoke".into()), limit: 100, ..Default::default() };
        assert_eq!(query(&conn, &f).unwrap().1, 1);

        let f = AuditQueryFilter { outcome: Some("error".into()), limit: 100, ..Default::default() };
        assert_eq!(query(&conn, &f).unwrap().1, 1);

        let f = AuditQueryFilter {
            since: Some("2026-05-05T00:00:00.000Z".into()),
            until: Some("2026-05-15T00:00:00.000Z".into()),
            limit: 100,
            ..Default::default()
        };
        let (page, total) = query(&conn, &f).unwrap();
        assert_eq!(total, 1);
        assert_eq!(page[0].actor, "bob");
    }

    #[test]
    fn query_limit_caps_page_but_total_counts_all() {
        let (_d, conn) = seeded();
        let f = AuditQueryFilter { limit: 2, ..Default::default() };
        let (page, total) = query(&conn, &f).unwrap();
        assert_eq!(total, 3); // ignores limit
        assert_eq!(page.len(), 2); // page capped
    }

    #[test]
    fn entries_before_and_delete_before_partition_by_cutoff() {
        let (_d, conn) = seeded();
        let before = "2026-05-15T00:00:00.000Z";
        let older = entries_before(&conn, before).unwrap();
        assert_eq!(older.len(), 2);
        assert_eq!(older[0].timestamp, "2026-05-01T00:00:00.000Z"); // oldest first
        let pruned = delete_before(&conn, before).unwrap();
        assert_eq!(pruned, 2);
        assert_eq!(entry_count(&conn).unwrap(), 1); // only the 2026-05-20 row remains
    }

    #[test]
    fn write_jsonl_emits_one_object_per_line() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("out.jsonl");
        let entries = vec![at("2026-05-01T00:00:00.000Z", "a", "x", "ok"), at("2026-05-02T00:00:00.000Z", "b", "y", "ok")];
        write_jsonl(&entries, &path).unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(lines.len(), 2);
        let parsed: AuditEntry = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed.verb, "a");
    }
}
