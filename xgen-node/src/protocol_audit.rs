// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Node-level **protocol audit log** (spec §3.11.8) — the permanent, append-only
//! accountability record of protocol-level facts (membership lifecycle, space/room
//! creation, federation). One Node-global file per calendar month at
//! `<data_dir>/audit/protocol_audit_YYYY-MM.jsonl` (JSON Lines). Always on; cannot
//! be disabled by config. Entries are *summary facts* — the full Event is always
//! recoverable from the DAG via `event_id`, so a missed write is a replayable gap
//! (closed by `space audit-rebuild`, PAL-D3), not data loss.
//!
//! **This is NOT the A6 admin trail (`audit.rs`).** That is a SQLite store of
//! admin-*verb* actions (who ran which CLI verb). This is a JSONL projection of
//! protocol *Events*. Different store, different content, different reader. Do not
//! conflate the two — that conflation is the slip that mis-marked the A4 row twice
//! (J-157).
//!
//! **Writer placement (PAL-D1, J-165 + checkpoint #1).** The single write site is
//! inside `app::persist_event`, the persist chokepoint every accept path funnels
//! through. The sink is a **process-global** (`OnceLock`, Shape β) installed once in
//! `run_node`: a Node process has exactly one audit dir and one `node_id`, so a
//! global is the honest domain model and avoids threading a param through ~11 hot
//! async signatures. No caller can pass the wrong sink — strengthening the
//! no-drift guarantee (D-067). `persist_event`'s per-`event_id` dedup guard makes
//! the hook idempotent; `replay_spaces_from_dir` uses `ingest_event` (never
//! `persist_event`), so the hook never re-fires on restart — no dup-on-replay.
//!
//! **Loud failure (PAL-D2 / D-070).** The audit append is best-effort *for protocol
//! liveness* (a full audit disk must not halt every Space) but **never silent**: on
//! write error it logs at `error` level and increments a process-global counter.
//! The event still persists; the gap is recoverable via the rebuild verb.
//!
//! **Type coverage (D-078, checkpoint #1, J-165).** Of the 11 EventTypes §3.11.8
//! names, the live protocol emits 8 (`membership.{join,leave,invite,kick,ban}`,
//! `state.{space_create,room_create,federation_add}`); `system.key_rotation` is a
//! declared-but-dormant variant kept here as a forward-ready arm; and
//! `state.federation_remove` + `identity.register` have no EventType at all (the
//! latter is the 8-step registration pipeline, not a DAG Event — structurally
//! uncapturable at this hook). Audit field names follow the §3.11.8 schema (the
//! auditor-facing contract) populated from real sources (`event.sender`, the
//! content `target_identity`/`node_id`/`auth_tier` keys, top-level `space_id`/
//! `room_id`); fields with no source (`approving_node_id`, `reason`) are omitted.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use xgen_common::wire::{Event, EventType};

/// Process-global count of protocol-audit write failures (PAL-D2). Surfaced for
/// health/observability; an increment means a recoverable audit gap exists.
static PROTOCOL_AUDIT_WRITE_FAILURES: AtomicU64 = AtomicU64::new(0);

/// Process-global protocol-audit sink (Shape β). Installed once in `run_node`;
/// read by `persist_event`'s hook. Absent (unit tests that never call `run_node`)
/// → no audit, persistence unaffected.
static GLOBAL_SINK: OnceLock<ProtocolAuditSink> = OnceLock::new();

/// Current process-global protocol-audit write-failure count (PAL-D2).
pub fn write_failure_count() -> u64 {
    PROTOCOL_AUDIT_WRITE_FAILURES.load(Ordering::Relaxed)
}

/// One protocol-audit log entry — one JSON object per line (JSON Lines). The four
/// universal fields are always present (§3.11.8); `extra` carries the EventType-
/// specific summary fields, serialised flat alongside the universal ones.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolAuditEntry {
    /// RFC 3339 UTC timestamp, millisecond precision — the Event's own `timestamp`
    /// (the protocol fact's time), so the entry lands in its own month's file.
    pub ts: String,
    /// The XGen EventType wire string.
    pub event_type: String,
    /// The Event's `event_id` hash URI — links the entry back to the DAG.
    pub event_id: String,
    /// The Node that produced this audit entry.
    pub node_id: String,
    /// EventType-specific summary fields (§3.11.8 schema names).
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl ProtocolAuditEntry {
    /// Build the audit entry for an Event, or `None` if this EventType is not one
    /// of the audited set (D-078 / checkpoint #1). Pure — no I/O.
    pub fn from_event(event: &Event, node_id: &str) -> Option<ProtocolAuditEntry> {
        let mut extra = Map::new();
        let sender = event.sender.as_str();
        let space_id = event.space_id.as_str();
        let content = &event.content;
        let content_str = |k: &str| content.get(k).and_then(Value::as_str);
        let mut put = |k: &str, v: &str| {
            extra.insert(k.to_string(), Value::String(v.to_string()));
        };

        match event.event_type {
            EventType::MembershipJoin => {
                // approving_node_id is §3.11.8-named but absent in content — omitted.
                put("identity_id", sender);
                put("space_id", space_id);
            }
            EventType::MembershipLeave => {
                put("identity_id", sender);
                put("space_id", space_id);
            }
            EventType::MembershipInvite => {
                put("inviter_id", sender);
                if let Some(t) = content_str("target_identity") {
                    put("invitee_id", t);
                }
                put("space_id", space_id);
            }
            EventType::MembershipKick => {
                // reason is §3.11.8 "if present" — never built into content; omitted.
                put("kicker_id", sender);
                if let Some(t) = content_str("target_identity") {
                    put("kicked_id", t);
                }
                put("space_id", space_id);
            }
            EventType::MembershipBan => {
                put("banner_id", sender);
                if let Some(t) = content_str("target_identity") {
                    put("banned_id", t);
                }
                put("space_id", space_id);
            }
            EventType::StateSpaceCreate => {
                put("creator_id", sender);
                put("space_id", space_id);
                if let Some(v) = content.get("auth_tier") {
                    extra.insert("auth_tier".to_string(), v.clone());
                }
            }
            EventType::StateRoomCreate => {
                put("creator_id", sender);
                put("room_id", event.room_id.as_str());
                put("space_id", space_id);
            }
            EventType::StateFederationAdd => {
                // The signing party is the initiator; content `node_id` is the peer.
                put("initiating_node_id", sender);
                if let Some(n) = content_str("node_id") {
                    put("receiving_node_id", n);
                }
                put("space_id", space_id);
            }
            EventType::SystemKeyRotation => {
                // Forward-ready (checkpoint #1, 2A): the variant exists but no
                // builder emits it yet, so old_key_hash/new_key_hash content is
                // undefined. Record the universal fields + identity_id; the key
                // hashes light up when key rotation is implemented.
                put("identity_id", sender);
            }
            // Everything else (message.*, node_eject/unban, dm.*, migration.*,
            // and the spec-named-but-nonexistent federation_remove/identity.register)
            // is not audited.
            _ => return None,
        }

        Some(ProtocolAuditEntry {
            ts: event.timestamp.clone(),
            event_type: event.event_type.as_str().to_string(),
            event_id: event
                .event_id
                .as_ref()
                .map(|e| e.as_str().to_string())
                .unwrap_or_default(),
            node_id: node_id.to_string(),
            extra,
        })
    }
}

/// The monthly JSONL file name for an entry timestamp — `protocol_audit_YYYY-MM.jsonl`.
/// Month derived from the entry's `ts` (not `now()`), so an event's line lands in
/// its own month's file even when written late (e.g. by a rebuild).
pub fn monthly_file_name(ts: &str) -> String {
    let ym = ts.get(0..7).unwrap_or("unknown");
    format!("protocol_audit_{ym}.jsonl")
}

/// Extract the `YYYY-MM` from a `protocol_audit_YYYY-MM.jsonl` file name.
fn month_of_file(name: &str) -> Option<String> {
    let stem = name.strip_prefix("protocol_audit_")?.strip_suffix(".jsonl")?;
    if stem.len() == 7 && stem.as_bytes()[4] == b'-' {
        Some(stem.to_string())
    } else {
        None
    }
}

fn within_month_range(m: &str, since: Option<&str>, until: Option<&str>) -> bool {
    if let Some(s) = since {
        if m < s {
            return false;
        }
    }
    if let Some(u) = until {
        if m > u {
            return false;
        }
    }
    true
}

/// Read protocol-audit entries from the monthly JSONL files under `audit_dir`,
/// restricted to month files within `[since_month, until_month]` (each `YYYY-MM`,
/// inclusive; `None` = open on that side). When both bounds are absent, **all**
/// present month files are scanned (the correct compliance behaviour — a reader
/// that silently returned only the current month would hide history). Files are
/// read in chronological order and entries returned in append order
/// (chronological within and across months). Malformed lines are skipped (a single
/// corrupt line must not fail a compliance read). Missing `audit_dir` → empty.
///
/// This is the coarse month-level pre-filter; precise per-`space_id` / event_type
/// / per-entry-`ts` filtering is the caller's (PAL-D1 read-time scope) — used by
/// the Commit 2 reader (`space audit-events`).
pub fn read_all_entries(
    audit_dir: &Path,
    since_month: Option<&str>,
    until_month: Option<&str>,
) -> Vec<ProtocolAuditEntry> {
    let mut files: Vec<(String, PathBuf)> = match std::fs::read_dir(audit_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                month_of_file(&name).map(|m| (m, e.path()))
            })
            .filter(|(m, _)| within_month_range(m, since_month, until_month))
            .collect(),
        Err(_) => return Vec::new(),
    };
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = Vec::new();
    for (_, path) in files {
        if let Ok(body) = std::fs::read_to_string(&path) {
            for line in body.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(entry) = serde_json::from_str::<ProtocolAuditEntry>(line) {
                    out.push(entry);
                }
            }
        }
    }
    out
}

/// The process-global protocol-audit sink (Shape β) — `audit_dir` + `node_id`,
/// the two per-process constants the writer needs.
#[derive(Debug, Clone)]
pub struct ProtocolAuditSink {
    audit_dir: PathBuf,
    node_id: String,
}

impl ProtocolAuditSink {
    /// Construct a sink. `audit_dir` is the `<data_dir>/audit/` directory.
    pub fn new(audit_dir: PathBuf, node_id: String) -> Self {
        Self { audit_dir, node_id }
    }

    /// Install the process-global sink. Idempotent — a second call is a no-op
    /// (`OnceLock::set` returns `Err` once set, ignored). Called once in `run_node`.
    pub fn init_global(sink: ProtocolAuditSink) {
        let _ = GLOBAL_SINK.set(sink);
    }

    /// The installed process-global sink, if any.
    pub fn global() -> Option<&'static ProtocolAuditSink> {
        GLOBAL_SINK.get()
    }

    /// Record one Event to the protocol audit log. No-op for non-audited
    /// EventTypes. Best-effort but LOUD on failure (PAL-D2): on write error,
    /// increments the failure counter + logs at `error` level. The Event has
    /// already persisted to the DAG; the gap is recoverable via `space audit-rebuild`.
    pub fn record(&self, event: &Event) {
        let entry = match ProtocolAuditEntry::from_event(event, &self.node_id) {
            Some(e) => e,
            None => return,
        };
        if let Err(e) = self.append(&entry) {
            PROTOCOL_AUDIT_WRITE_FAILURES.fetch_add(1, Ordering::Relaxed);
            tracing::error!(
                event_id = %entry.event_id,
                event_type = %entry.event_type,
                audit_dir = ?self.audit_dir,
                error = %e,
                "protocol_audit_write_failed: audit entry not persisted (event is in the DAG; recover via `space audit-rebuild`)"
            );
        }
    }

    /// Append one entry as a JSON line to its month's file. Creates `audit_dir`
    /// and the month file on demand; never truncates or deletes (§3.11.8 — rotation
    /// is new-file-on-month-boundary only).
    fn append(&self, entry: &ProtocolAuditEntry) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.audit_dir)?;
        let path = self.audit_dir.join(monthly_file_name(&entry.ts));
        let line = serde_json::to_string(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        f.write_all(line.as_bytes())?;
        f.write_all(b"\n")?;
        Ok(())
    }

    /// The audit directory this sink writes to (read access for the Commit 2
    /// reader + Commit 3 rebuild, which resolve it the same way `run_node` does).
    pub fn audit_dir(&self) -> &Path {
        &self.audit_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use xgen_common::wire::Event;
    use xgen_common::xgid::{EventXgid, IdentityXgid, RoomXgid, SpaceXgid, Xgid};

    const NODE_ID: &str = "xgen://pubkey/ed25519:NODEAAAA";

    /// Minimal hand-built Event. from_event/persist do not validate, so this is
    /// sufficient to exercise the audit projection + store.
    fn ev(event_type: EventType, content: Value, event_id: &str) -> Event {
        let mut e = Event::new(
            event_type,
            IdentityXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:SENDERAA".to_string())),
            RoomXgid::from_xgid(Xgid::new("xgen://hash/sha256:room1".to_string())),
            SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:space1".to_string())),
            vec![],
            "2026-05-30T14:35:31.014Z".to_string(),
            content,
        );
        e.event_id = Some(EventXgid::from_xgid(Xgid::new(event_id.to_string())));
        e
    }

    // Test 1 — entry serialises to one valid JSON line carrying universal + extra
    // fields, across a sample of the audited types, and round-trips back.
    #[test]
    fn protocol_audit_entry_serde_jsonl_roundtrip() {
        let samples = [
            ev(EventType::MembershipJoin, json!({}), "xgen://hash/sha256:e1"),
            ev(
                EventType::MembershipInvite,
                json!({ "target_identity": "xgen://pubkey/ed25519:BOB", "role": "member" }),
                "xgen://hash/sha256:e2",
            ),
            ev(
                EventType::StateSpaceCreate,
                json!({ "name": "s", "auth_tier": 1, "nonce": "n", "home_node": NODE_ID }),
                "xgen://hash/sha256:e3",
            ),
            ev(
                EventType::StateFederationAdd,
                json!({ "node_id": "xgen://pubkey/ed25519:PEER" }),
                "xgen://hash/sha256:e4",
            ),
        ];
        for e in &samples {
            let entry = ProtocolAuditEntry::from_event(e, NODE_ID).expect("audited type");
            let line = serde_json::to_string(&entry).unwrap();
            assert!(!line.contains('\n'), "one line per entry");
            // Universal fields flattened alongside the extra fields.
            let v: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(v["event_type"], e.event_type.as_str());
            assert_eq!(v["node_id"], NODE_ID);
            assert_eq!(v["ts"], "2026-05-30T14:35:31.014Z");
            assert!(v.get("event_id").is_some());
            // Round-trip.
            let back: ProtocolAuditEntry = serde_json::from_str(&line).unwrap();
            assert_eq!(back, entry);
        }
        // §3.11.8 schema-name spot checks (1A field naming).
        let invite = ProtocolAuditEntry::from_event(&samples[1], NODE_ID).unwrap();
        assert_eq!(
            invite.extra.get("invitee_id").and_then(Value::as_str),
            Some("xgen://pubkey/ed25519:BOB")
        );
        assert!(invite.extra.contains_key("inviter_id"));
        let fed = ProtocolAuditEntry::from_event(&samples[3], NODE_ID).unwrap();
        assert_eq!(
            fed.extra.get("receiving_node_id").and_then(Value::as_str),
            Some("xgen://pubkey/ed25519:PEER")
        );
        assert!(fed.extra.contains_key("initiating_node_id"));
    }

    // Test 2 — month file name derives from the entry ts, not now().
    #[test]
    fn monthly_file_path_derived_from_entry_ts() {
        let e = ev(EventType::MembershipJoin, json!({}), "xgen://hash/sha256:e1");
        let entry = ProtocolAuditEntry::from_event(&e, NODE_ID).unwrap();
        assert_eq!(monthly_file_name(&entry.ts), "protocol_audit_2026-05.jsonl");
        assert_eq!(
            monthly_file_name("2027-01-02T00:00:00.000Z"),
            "protocol_audit_2027-01.jsonl"
        );
    }

    // Test 3 — recording a listed EventType appends exactly one line to the
    // correct month file with the expected fields.
    #[test]
    fn record_writes_audit_entry_for_listed_eventtype() {
        let dir = tempfile::tempdir().unwrap();
        let sink = ProtocolAuditSink::new(dir.path().join("audit"), NODE_ID.to_string());
        sink.record(&ev(EventType::MembershipJoin, json!({}), "xgen://hash/sha256:e1"));
        let path = dir.path().join("audit").join("protocol_audit_2026-05.jsonl");
        let body = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body.lines().count(), 1);
        let v: Value = serde_json::from_str(body.lines().next().unwrap()).unwrap();
        assert_eq!(v["event_type"], "membership.join");
        assert_eq!(v["identity_id"], "xgen://pubkey/ed25519:SENDERAA");
        assert_eq!(v["space_id"], "xgen://hash/sha256:space1");
    }

    // Test 4 — an unlisted EventType writes no audit line at all.
    #[test]
    fn record_skips_audit_for_unlisted_eventtype() {
        let dir = tempfile::tempdir().unwrap();
        let sink = ProtocolAuditSink::new(dir.path().join("audit"), NODE_ID.to_string());
        // message.text and node_eject are explicitly NOT audited.
        sink.record(&ev(
            EventType::MessageText,
            json!({ "text": "hi" }),
            "xgen://hash/sha256:m1",
        ));
        sink.record(&ev(
            EventType::MembershipNodeEject,
            json!({ "target_identity": "xgen://pubkey/ed25519:X" }),
            "xgen://hash/sha256:m2",
        ));
        assert!(ProtocolAuditEntry::from_event(
            &ev(EventType::MessageText, json!({}), "xgen://hash/sha256:m1"),
            NODE_ID
        )
        .is_none());
        // No file created.
        assert!(!dir
            .path()
            .join("audit")
            .join("protocol_audit_2026-05.jsonl")
            .exists());
    }

    // Test 5 — persist (via the production hook) writes one audit line; replay
    // from disk does NOT duplicate it (replay uses ingest_event, never
    // persist_event). The single global-using test; it owns the OnceLock.
    #[test]
    fn persist_audits_then_replay_does_not_duplicate() {
        use xgen_core::identity::keypair;
        use xgen_core::node::runtime::NodeRuntime;

        let dir = tempfile::tempdir().unwrap();
        let spaces_dir = dir.path().join("spaces");
        let audit_dir = dir.path().join("audit");
        ProtocolAuditSink::init_global(ProtocolAuditSink::new(
            audit_dir.clone(),
            NODE_ID.to_string(),
        ));

        let event = ev(EventType::MembershipJoin, json!({}), "xgen://hash/sha256:join1");
        crate::app::persist_event(&spaces_dir, "xgen://hash/sha256:space1", &event);

        let audit_file = audit_dir.join("protocol_audit_2026-05.jsonl");
        let lines_after_persist = std::fs::read_to_string(&audit_file)
            .map(|s| s.lines().count())
            .unwrap_or(0);
        assert_eq!(lines_after_persist, 1, "persist wrote one audit line");

        // Replay the on-disk Space store: must not append a second line.
        let mut runtime = NodeRuntime::new(keypair::generate());
        let _ = crate::app::replay_spaces_from_dir(&mut runtime, &spaces_dir);
        let lines_after_replay = std::fs::read_to_string(&audit_file)
            .map(|s| s.lines().count())
            .unwrap_or(0);
        assert_eq!(
            lines_after_replay, 1,
            "replay used ingest_event, not persist_event — no dup-on-restart"
        );
    }

    // Test 6 — a write failure is loud (counter increments), never swallowed.
    #[test]
    fn audit_write_failure_is_loud_not_swallowed() {
        let dir = tempfile::tempdir().unwrap();
        // Make audit_dir uncreatable: point it under a regular file, so
        // create_dir_all fails.
        let blocker = dir.path().join("blocker");
        std::fs::write(&blocker, b"not a dir").unwrap();
        let sink = ProtocolAuditSink::new(blocker.join("audit"), NODE_ID.to_string());

        let before = write_failure_count();
        sink.record(&ev(EventType::MembershipJoin, json!({}), "xgen://hash/sha256:e1"));
        let after = write_failure_count();
        assert!(after > before, "write failure incremented the loud counter");
    }
}
