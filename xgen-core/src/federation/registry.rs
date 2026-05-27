// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Federation relationship registry (spec 3.4.5).
//
// Persistent record of active federation relationships, plus the F-1c
// per-peer operational record (runbook §3.5.1 Lock A). The operational
// record sits alongside the relationship record in the same JSON file —
// `peer_records: HashMap<peer_node_id, PeerOperationalRecord>` — so the
// registry has a single save site, single load site, and type-clean
// separation between protocol state (`relationships`) and operational
// state (`peer_records`).
//
// Consulted on startup to re-establish connections without requiring a
// new handshake sequence; read by the reconnect scheduler each tick.

use std::{collections::HashMap, path::Path};

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use xgen_common::xgid::{NodeXgid, SpaceXgid, Xgid};

use super::handshake::FederationSession;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Initial delay between observing a lost session and the first reconnect
/// attempt (runbook §3.5.1 Lock B3). The peer's incoming handshake may
/// resolve a transient blip within this window; if not, the scheduler's
/// first attempt fires after the delay.
pub const INITIAL_RECONNECT_DELAY_MINUTES: i64 = 15;

// ── Types ─────────────────────────────────────────────────────────────────────

/// A single recorded federation relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationRelationship {
    pub peer_node_id: NodeXgid,
    pub shared_spaces: Vec<SpaceXgid>,
    pub negotiated_version: String,
    pub negotiated_serialisation: String,
    /// Session-binding nonce. NOT an XGID per D-072 "what XGID is not"
    /// (session IDs are ephemeral per-connection identifiers, not
    /// protocol-object handles); stays `String` across Pass 1.
    pub session_id: String,
    /// RFC 3339 timestamp of the last successful connection.
    pub last_connected: String,
    /// WebSocket endpoint URL of the peer Node (advisory; from hello node_endpoint).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub peer_url: Option<String>,
}

impl FederationRelationship {
    pub fn from_session(session: &FederationSession, last_connected: String) -> Self {
        // Pass 2 retypes `FederationSession` to carry typed XGIDs; the projection
        // wraps here collapse then. Pass 1 stays at data-structure scope per
        // runbook §Scope — `FederationSession` is algorithm-bearing (Pass 2).
        Self {
            peer_node_id: NodeXgid::from_xgid(Xgid::new(session.peer_node_id.clone())),
            shared_spaces: session
                .shared_spaces
                .iter()
                .map(|s| SpaceXgid::from_xgid(Xgid::new(s.clone())))
                .collect(),
            negotiated_version: session.negotiated_version.clone(),
            negotiated_serialisation: session.negotiated_serialisation.clone(),
            session_id: session.session_id.clone(),
            last_connected,
            peer_url: session.peer_url.clone(),
        }
    }
}

/// F-1c per-peer operational record (runbook §3.5.1 Lock A, design §4.6).
///
/// Operational state lives alongside the protocol relationship in the same
/// `FederationRegistry`. Backoff cursor (which step of the 15/30/60/120
/// ladder we're on) is transient scheduler state — see the scheduler module
/// for that — and is intentionally NOT stored here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerOperationalRecord {
    pub peer_node_id: NodeXgid,
    pub lost_connection: bool,
    /// RFC 3339 UTC — last operational signal that this peer was alive
    /// (handshake-ACTIVE on either direction). Not updated by failed
    /// reconnect attempts; those advance `next_reconnect_attempt` only.
    pub last_seen: String,
    /// RFC 3339 UTC — last time a handshake completed to ACTIVE state.
    /// Distinguished from `last_seen` so observability can show "peer was
    /// reachable but the last full session was N hours ago."
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub last_successful_session: Option<String>,
    /// RFC 3339 UTC — scheduler reads this each tick; if `lost_connection`
    /// and this is `<= now`, the scheduler fires a reconnect attempt.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub next_reconnect_attempt: Option<String>,
    /// Operator-set freeform notes (admin UI surface; not consulted by code).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub operator_notes: Option<String>,
    /// Operator-set per-peer priority for future per-peer backoff override.
    /// Phase 5 does NOT consult this field — the global backoff schedule
    /// applies uniformly. Reserved so future versions can plug in per-peer
    /// override semantics (e.g. `priority=high` mapping to a tighter ladder)
    /// without protocol or schema change. See runbook §3.5.1 forward-compat
    /// note.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub priority: Option<i32>,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// Persistent federation relationship registry, keyed by peer node_id.
///
/// JSON file shape: `{ "relationships": {...}, "peer_records": {...} }`.
/// Both maps are keyed by `peer_node_id`. `peer_records` is annotated
/// `#[serde(default)]` so loading a registry file written before F-1c
/// shipped reads as an empty operational-record map.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FederationRegistry {
    #[serde(default)]
    relationships: HashMap<NodeXgid, FederationRelationship>,
    #[serde(default)]
    peer_records: HashMap<NodeXgid, PeerOperationalRecord>,
}

impl FederationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or update a relationship. The last_connected timestamp is supplied
    /// by the caller so the registry stays deterministic in tests.
    pub fn upsert(&mut self, rel: FederationRelationship) {
        self.relationships.insert(rel.peer_node_id.clone(), rel);
    }

    /// Remove a relationship (called when `federation.goodbye` is received).
    pub fn remove(&mut self, peer_node_id: &NodeXgid) -> Option<FederationRelationship> {
        // Pass 2 (J-125, design §2.4 Q4.5 sibling-of-Q4.1) — signature retypes
        // to &NodeXgid; internal HashMap key already typed at Pass 1 (Q4.7).
        self.relationships.remove(peer_node_id)
    }

    pub fn get(&self, peer_node_id: &NodeXgid) -> Option<&FederationRelationship> {
        // Pass 2 (J-125, design §2.4 Q4.5 sibling-of-Q4.1) — signature retypes
        // to &NodeXgid.
        self.relationships.get(peer_node_id)
    }

    pub fn all(&self) -> Vec<&FederationRelationship> {
        self.relationships.values().collect()
    }

    pub fn len(&self) -> usize {
        self.relationships.len()
    }

    pub fn is_empty(&self) -> bool {
        self.relationships.is_empty()
    }

    // ── F-1c operational record API ───────────────────────────────────────────

    /// Mark a peer as active (handshake completed to ACTIVE on either
    /// direction). Idempotent: upserts the operational record with
    /// `lost_connection: false`, refreshes `last_seen` + `last_successful_session`
    /// to `now`, clears `next_reconnect_attempt`. Preserves operator-set
    /// fields (`operator_notes`, `priority`).
    pub fn mark_active(&mut self, peer_node_id: &NodeXgid, now: DateTime<Utc>) {
        // Pass 2 (J-125, design §2.4 Q4.5) — signature retypes to &NodeXgid;
        // bridge wrap dropped per Q4.6.
        let now_str = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let existing = self.peer_records.remove(peer_node_id);
        let (operator_notes, priority) = existing
            .as_ref()
            .map(|r| (r.operator_notes.clone(), r.priority))
            .unwrap_or((None, None));
        self.peer_records.insert(
            peer_node_id.clone(),
            PeerOperationalRecord {
                peer_node_id: peer_node_id.clone(),
                lost_connection: false,
                last_seen: now_str.clone(),
                last_successful_session: Some(now_str),
                next_reconnect_attempt: None,
                operator_notes,
                priority,
            },
        );
    }

    /// Mark a peer as lost (session ended for any reason — goodbye,
    /// keepalive failure, transport error). Sets `lost_connection: true`,
    /// updates `last_seen` to `now`, schedules `next_reconnect_attempt` at
    /// `now + INITIAL_RECONNECT_DELAY_MINUTES` (Lock B3).
    ///
    /// If the record was already lost (e.g. mark_lost called twice without
    /// an intervening mark_active), preserves the existing
    /// `next_reconnect_attempt` so an in-flight backoff schedule is not
    /// disturbed.
    pub fn mark_lost(&mut self, peer_node_id: &NodeXgid, now: DateTime<Utc>) {
        // Pass 2 (J-125, design §2.4 Q4.5) — signature retypes to &NodeXgid;
        // bridge wrap dropped per Q4.6.
        let now_str = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let first_attempt = (now + Duration::minutes(INITIAL_RECONNECT_DELAY_MINUTES))
            .to_rfc3339_opts(SecondsFormat::Millis, true);
        match self.peer_records.get_mut(peer_node_id) {
            Some(existing) => {
                let was_lost = existing.lost_connection;
                existing.lost_connection = true;
                existing.last_seen = now_str;
                if !was_lost {
                    existing.next_reconnect_attempt = Some(first_attempt);
                }
            }
            None => {
                self.peer_records.insert(
                    peer_node_id.clone(),
                    PeerOperationalRecord {
                        peer_node_id: peer_node_id.clone(),
                        lost_connection: true,
                        last_seen: now_str,
                        last_successful_session: None,
                        next_reconnect_attempt: Some(first_attempt),
                        operator_notes: None,
                        priority: None,
                    },
                );
            }
        }
    }

    /// Update `next_reconnect_attempt` on an existing lost peer record.
    /// Called by the scheduler after firing an attempt to record when the
    /// next attempt is due per the backoff ladder. No-op if the record
    /// doesn't exist or is no longer flagged lost (a concurrent
    /// `mark_active` won).
    pub fn update_next_reconnect(&mut self, peer_node_id: &NodeXgid, next_at: DateTime<Utc>) {
        // Pass 2 (J-125, design §2.4 Q4.5) — signature retypes to &NodeXgid;
        // bridge wrap dropped per Q4.6.
        if let Some(rec) = self.peer_records.get_mut(peer_node_id) {
            if rec.lost_connection {
                rec.next_reconnect_attempt =
                    Some(next_at.to_rfc3339_opts(SecondsFormat::Millis, true));
            }
        }
    }

    pub fn peer_record(&self, peer_node_id: &NodeXgid) -> Option<&PeerOperationalRecord> {
        // Pass 2 (J-125, design §2.4 Q4.5) — signature retypes to &NodeXgid;
        // bridge wrap dropped per Q4.6.
        self.peer_records.get(peer_node_id)
    }

    /// Return all peers currently flagged lost whose `next_reconnect_attempt`
    /// is `<= now`. Scheduler iterates this list each tick and fires
    /// `run_initiating` for each (using the `peer_url` stored on the
    /// matching `FederationRelationship`).
    ///
    /// Records with `next_reconnect_attempt: None` or with a timestamp that
    /// fails to parse are skipped — the scheduler does not crash on
    /// malformed records.
    pub fn due_for_reconnect(&self, now: DateTime<Utc>) -> Vec<&PeerOperationalRecord> {
        self.peer_records
            .values()
            .filter(|r| r.lost_connection)
            .filter(|r| match r.next_reconnect_attempt.as_deref() {
                Some(ts) => DateTime::parse_from_rfc3339(ts)
                    .map(|t| t.with_timezone(&Utc) <= now)
                    .unwrap_or(false),
                None => false,
            })
            .collect()
    }

    /// All operational records (for admin/observability surfaces).
    pub fn peer_records(&self) -> Vec<&PeerOperationalRecord> {
        self.peer_records.values().collect()
    }

    // ── Persistence ───────────────────────────────────────────────────────────

    pub fn save(&self, path: &Path) -> Result<(), RegistryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        let json = std::fs::read_to_string(path)?;
        let reg: Self = serde_json::from_str(&json)?;
        Ok(reg)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn sample_rel(peer_id: &str) -> FederationRelationship {
        FederationRelationship {
            peer_node_id: NodeXgid::from_xgid(Xgid::new(peer_id.to_string())),
            shared_spaces: vec![SpaceXgid::from_xgid(Xgid::new(
                "xgen://hash/sha256:space1".to_string(),
            ))],
            negotiated_version: "0.1".to_string(),
            negotiated_serialisation: "json".to_string(),
            session_id: "xgen://hash/sha256:session1".to_string(),
            last_connected: "2026-04-27T12:00:00.000Z".to_string(),
            peer_url: None,
        }
    }

    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    /// Project a `&str` peer node id to a typed `NodeXgid` key for direct
    /// HashMap field access in tests. Pass 2 widens the field-access surfaces
    /// where these tests reach in.
    fn node_key(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }

    #[test]
    fn upsert_and_get() {
        let mut reg = FederationRegistry::new();
        let rel = sample_rel("xgen://pubkey/ed25519:AAAA");
        reg.upsert(rel);
        assert!(reg.get(&node_key("xgen://pubkey/ed25519:AAAA")).is_some());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn upsert_updates_existing() {
        let mut reg = FederationRegistry::new();
        reg.upsert(sample_rel("xgen://pubkey/ed25519:AAAA"));
        let updated = FederationRelationship {
            last_connected: "2026-05-01T00:00:00.000Z".to_string(),
            session_id: "xgen://hash/sha256:session2".to_string(),
            ..sample_rel("xgen://pubkey/ed25519:AAAA")
        };
        reg.upsert(updated);
        let stored = reg.get(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert_eq!(stored.session_id, "xgen://hash/sha256:session2");
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn remove_returns_and_deletes() {
        let mut reg = FederationRegistry::new();
        reg.upsert(sample_rel("xgen://pubkey/ed25519:AAAA"));
        let removed = reg.remove(&node_key("xgen://pubkey/ed25519:AAAA"));
        assert!(removed.is_some());
        assert!(reg.is_empty());
    }

    #[test]
    fn all_returns_all_entries() {
        let mut reg = FederationRegistry::new();
        reg.upsert(sample_rel("xgen://pubkey/ed25519:AAAA"));
        reg.upsert(sample_rel("xgen://pubkey/ed25519:BBBB"));
        assert_eq!(reg.all().len(), 2);
    }

    #[test]
    fn save_load_round_trip() {
        let mut reg = FederationRegistry::new();
        reg.upsert(sample_rel("xgen://pubkey/ed25519:AAAA"));
        reg.upsert(sample_rel("xgen://pubkey/ed25519:BBBB"));

        let tmp = NamedTempFile::new().unwrap();
        reg.save(tmp.path()).unwrap();

        let loaded = FederationRegistry::load(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.get(&node_key("xgen://pubkey/ed25519:AAAA")).is_some());
        assert!(loaded.get(&node_key("xgen://pubkey/ed25519:BBBB")).is_some());
    }

    #[test]
    fn empty_registry_saves_and_loads() {
        let reg = FederationRegistry::new();
        let tmp = NamedTempFile::new().unwrap();
        reg.save(tmp.path()).unwrap();
        let loaded = FederationRegistry::load(tmp.path()).unwrap();
        assert!(loaded.is_empty());
    }

    // Per-surface unit test for Surface #4 Q4.5 retypes — the typed-API surface
    // (get / remove / mark_active / mark_lost / update_next_reconnect /
    // peer_record) compiles and behaves correctly with &NodeXgid arguments
    // (post-Pass-2 boundary). Sibling-shape to runbook §4.7 framing + the
    // IdentityRegistry per-surface test.
    #[test]
    fn get_and_peer_record_accept_typed_node_xgid() {
        let mut reg = FederationRegistry::new();
        let id = node_key("xgen://pubkey/ed25519:FFFF");
        reg.upsert(sample_rel("xgen://pubkey/ed25519:FFFF"));
        reg.mark_active(&id, at("2026-05-27T12:00:00.000Z"));
        assert!(reg.get(&id).is_some());
        assert!(reg.peer_record(&id).is_some());
        let removed = reg.remove(&id);
        assert!(removed.is_some());
        assert!(reg.get(&id).is_none());
    }

    // ── F-1c PeerOperationalRecord tests ──────────────────────────────────────

    #[test]
    fn mark_active_creates_record_with_lost_false() {
        let mut reg = FederationRegistry::new();
        let now = at("2026-05-19T10:00:00.000Z");
        reg.mark_active(&node_key("xgen://pubkey/ed25519:AAAA"), now);
        let rec = reg.peer_record(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert!(!rec.lost_connection);
        assert_eq!(rec.last_seen, "2026-05-19T10:00:00.000Z");
        assert_eq!(rec.last_successful_session.as_deref(), Some("2026-05-19T10:00:00.000Z"));
        assert!(rec.next_reconnect_attempt.is_none());
    }

    #[test]
    fn mark_lost_schedules_initial_attempt_at_plus_15min() {
        let mut reg = FederationRegistry::new();
        let now = at("2026-05-19T10:00:00.000Z");
        reg.mark_lost(&node_key("xgen://pubkey/ed25519:AAAA"), now);
        let rec = reg.peer_record(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert!(rec.lost_connection);
        assert_eq!(rec.last_seen, "2026-05-19T10:00:00.000Z");
        assert_eq!(rec.next_reconnect_attempt.as_deref(), Some("2026-05-19T10:15:00.000Z"));
        assert!(rec.last_successful_session.is_none());
    }

    #[test]
    fn mark_lost_then_active_clears_next_reconnect() {
        let mut reg = FederationRegistry::new();
        let t1 = at("2026-05-19T10:00:00.000Z");
        let t2 = at("2026-05-19T10:30:00.000Z");
        reg.mark_lost(&node_key("xgen://pubkey/ed25519:AAAA"), t1);
        reg.mark_active(&node_key("xgen://pubkey/ed25519:AAAA"), t2);
        let rec = reg.peer_record(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert!(!rec.lost_connection);
        assert!(rec.next_reconnect_attempt.is_none());
        assert_eq!(rec.last_successful_session.as_deref(), Some("2026-05-19T10:30:00.000Z"));
    }

    #[test]
    fn mark_lost_idempotent_preserves_next_reconnect() {
        // Two consecutive mark_lost calls (e.g. defensive call from two
        // close paths) must not bump the scheduled retry forward — the
        // scheduler's backoff progression would be disrupted otherwise.
        let mut reg = FederationRegistry::new();
        let t1 = at("2026-05-19T10:00:00.000Z");
        let t2 = at("2026-05-19T10:05:00.000Z");
        reg.mark_lost(&node_key("xgen://pubkey/ed25519:AAAA"), t1);
        reg.mark_lost(&node_key("xgen://pubkey/ed25519:AAAA"), t2);
        let rec = reg.peer_record(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert_eq!(rec.next_reconnect_attempt.as_deref(), Some("2026-05-19T10:15:00.000Z"));
        assert_eq!(rec.last_seen, "2026-05-19T10:05:00.000Z");
    }

    #[test]
    fn mark_active_preserves_operator_notes_and_priority() {
        let mut reg = FederationRegistry::new();
        let t1 = at("2026-05-19T10:00:00.000Z");
        reg.mark_active(&node_key("xgen://pubkey/ed25519:AAAA"), t1);
        // Operator sets notes + priority directly on the record.
        reg.peer_records.get_mut(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap().operator_notes =
            Some("primary east-coast peer".to_string());
        reg.peer_records.get_mut(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap().priority = Some(10);
        // mark_lost then mark_active must not lose operator-set fields.
        reg.mark_lost(&node_key("xgen://pubkey/ed25519:AAAA"), at("2026-05-19T11:00:00.000Z"));
        reg.mark_active(&node_key("xgen://pubkey/ed25519:AAAA"), at("2026-05-19T11:30:00.000Z"));
        let rec = reg.peer_record(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert_eq!(rec.operator_notes.as_deref(), Some("primary east-coast peer"));
        assert_eq!(rec.priority, Some(10));
    }

    #[test]
    fn update_next_reconnect_advances_only_if_still_lost() {
        let mut reg = FederationRegistry::new();
        let t1 = at("2026-05-19T10:00:00.000Z");
        reg.mark_lost(&node_key("xgen://pubkey/ed25519:AAAA"), t1);
        // Scheduler fires the 15-min attempt and reschedules to +30min from now.
        let scheduler_now = at("2026-05-19T10:15:00.000Z");
        let next = at("2026-05-19T10:45:00.000Z");
        reg.update_next_reconnect(&node_key("xgen://pubkey/ed25519:AAAA"), next);
        let rec = reg.peer_record(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert_eq!(rec.next_reconnect_attempt.as_deref(), Some("2026-05-19T10:45:00.000Z"));
        // Now a successful handshake races in.
        reg.mark_active(&node_key("xgen://pubkey/ed25519:AAAA"), scheduler_now);
        // A late update from a stale scheduler tick must not re-arm reconnect.
        reg.update_next_reconnect(&node_key("xgen://pubkey/ed25519:AAAA"), at("2026-05-19T11:15:00.000Z"));
        let rec = reg.peer_record(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap();
        assert!(!rec.lost_connection);
        assert!(rec.next_reconnect_attempt.is_none());
    }

    #[test]
    fn due_for_reconnect_returns_only_lost_and_due() {
        let mut reg = FederationRegistry::new();
        reg.mark_lost(&node_key("xgen://pubkey/ed25519:AAAA"), at("2026-05-19T10:00:00.000Z"));
        reg.mark_lost(&node_key("xgen://pubkey/ed25519:BBBB"), at("2026-05-19T10:10:00.000Z"));
        reg.mark_active(&node_key("xgen://pubkey/ed25519:CCCC"), at("2026-05-19T10:00:00.000Z"));
        // At 10:16, A is due (scheduled 10:15), B is not (scheduled 10:25), C is active.
        let due = reg.due_for_reconnect(at("2026-05-19T10:16:00.000Z"));
        let due_ids: Vec<&str> = due.iter().map(|r| r.peer_node_id.as_str()).collect();
        assert_eq!(due_ids, vec!["xgen://pubkey/ed25519:AAAA"]);
        // At 10:26, both A and B are due.
        let due = reg.due_for_reconnect(at("2026-05-19T10:26:00.000Z"));
        assert_eq!(due.len(), 2);
    }

    #[test]
    fn due_for_reconnect_skips_records_with_no_next_attempt() {
        // Mark lost then null out next_reconnect_attempt manually — should
        // be skipped (defensive: avoid scheduler firing forever).
        let mut reg = FederationRegistry::new();
        reg.mark_lost(&node_key("xgen://pubkey/ed25519:AAAA"), at("2026-05-19T10:00:00.000Z"));
        reg.peer_records.get_mut(&node_key("xgen://pubkey/ed25519:AAAA")).unwrap().next_reconnect_attempt = None;
        let due = reg.due_for_reconnect(at("2026-05-19T11:00:00.000Z"));
        assert!(due.is_empty());
    }

    #[test]
    fn peer_records_survive_save_load_round_trip() {
        let mut reg = FederationRegistry::new();
        reg.upsert(sample_rel("xgen://pubkey/ed25519:AAAA"));
        reg.mark_active(&node_key("xgen://pubkey/ed25519:AAAA"), at("2026-05-19T10:00:00.000Z"));
        reg.mark_lost(&node_key("xgen://pubkey/ed25519:BBBB"), at("2026-05-19T10:30:00.000Z"));
        reg.peer_records.get_mut(&node_key("xgen://pubkey/ed25519:BBBB")).unwrap().operator_notes =
            Some("flaky peer".to_string());

        let tmp = NamedTempFile::new().unwrap();
        reg.save(tmp.path()).unwrap();
        let loaded = FederationRegistry::load(tmp.path()).unwrap();

        let a = loaded.peer_record("xgen://pubkey/ed25519:AAAA").unwrap();
        assert!(!a.lost_connection);
        assert_eq!(a.last_successful_session.as_deref(), Some("2026-05-19T10:00:00.000Z"));

        let b = loaded.peer_record("xgen://pubkey/ed25519:BBBB").unwrap();
        assert!(b.lost_connection);
        assert_eq!(b.operator_notes.as_deref(), Some("flaky peer"));
        assert_eq!(b.next_reconnect_attempt.as_deref(), Some("2026-05-19T10:45:00.000Z"));
    }

    #[test]
    fn load_old_format_without_peer_records_field_works() {
        // Forward-compat: a registry JSON file written by pre-F-1c code
        // (relationships only, no peer_records key) must load cleanly with
        // an empty peer_records map. #[serde(default)] on the field
        // provides this; the test pins the behaviour.
        let json = r#"{
            "relationships": {
                "xgen://pubkey/ed25519:AAAA": {
                    "peer_node_id": "xgen://pubkey/ed25519:AAAA",
                    "shared_spaces": ["xgen://hash/sha256:space1"],
                    "negotiated_version": "0.1",
                    "negotiated_serialisation": "json",
                    "session_id": "xgen://hash/sha256:session1",
                    "last_connected": "2026-04-27T12:00:00.000Z"
                }
            }
        }"#;
        let tmp = NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), json).unwrap();
        let loaded = FederationRegistry::load(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded.peer_records().is_empty());
    }
}
