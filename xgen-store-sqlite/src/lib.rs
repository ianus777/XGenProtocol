// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! `xgen-store-sqlite` — a durable SQLite-backed `EventStore` (Storage-Engine /
//! Plugin-Framework milestone, the first `system·node` slot instance, SE-D1 /
//! SE-D7).
//!
//! This is a **registry-constructible, reopen-correct** storage engine through
//! the compile-time module spine. It is *not yet* threaded into per-Space
//! `NodeRuntime` construction — that live substitution is a separate, design-
//! gated step (storage granularity / file layout is an open question; "one DB
//! per Space" was the §4.12.1 drift the J-228 EventStore close struck).
//!
//! ## Conformance (SE-D7)
//!
//! - **Descriptor const** — copies the slot's [`STORAGE_ENGINE_KIND_ID`] verbatim
//!   into `kind_id`; its own generated `impl_id`; `assurance = Durable`.
//! - **`StorageEngine::open`** — reads its own `[storage.sqlite]` schema
//!   ([`SqliteSettings`]) and is **loud on failure** (`EngineError`, SE-D5).
//! - **Durable append-seq** — a persisted, 0-based `seq` column so
//!   `range(since_seq)` is correct across restart (the SE-D1 contract item),
//!   matching the vanilla backend's append-order semantics.
//! - **`Send + Sync`** — `rusqlite::Connection` is `Send` but `!Sync`; a `Mutex`
//!   makes the engine `Send + Sync` so it type-erases to
//!   `Box<dyn EventStore + Send + Sync>` (SE-D6).

use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;

use xgen_common::module::{AssuranceClass, Descriptor, ModuleImplId};
use xgen_common::xgid::EventXgid;
use xgen_core::dag::store::{
    EngineError, EngineSettings, EventStore, StorageEngine, StoreError, STORAGE_ENGINE_KIND_ID,
};
use xgen_core::wire::types::Event;

/// The engine's own `[storage.sqlite]` settings schema (SE-D5). The plugin owns
/// and validates this; the host passes the sub-table opaquely.
#[derive(Debug, Deserialize)]
struct SqliteSettings {
    /// Filesystem path to the SQLite database file.
    path: String,
}

/// A durable SQLite-backed [`EventStore`] (`Durable` assurance — crash-survivable
/// via SQLite's commit). The append-sequence is persisted in a dedicated 0-based
/// `seq` column so `range(since_seq)` survives restart (SE-D1).
pub struct SqliteEngine {
    conn: Mutex<Connection>,
}

impl SqliteEngine {
    fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                 seq      INTEGER PRIMARY KEY,
                 event_id TEXT NOT NULL UNIQUE,
                 payload  TEXT NOT NULL
             );",
        )
    }
}

impl StorageEngine for SqliteEngine {
    fn open(settings: &EngineSettings) -> Result<Self, EngineError> {
        // SE-D5: validate the plugin's own schema, loud on failure.
        let cfg: SqliteSettings = settings.deserialize()?;
        let conn = Connection::open(&cfg.path)
            .map_err(|e| EngineError::Open(format!("opening {}: {e}", cfg.path)))?;
        Self::init_schema(&conn)
            .map_err(|e| EngineError::Open(format!("initialising schema: {e}")))?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn descriptor() -> Descriptor {
        Descriptor {
            kind_id: STORAGE_ENGINE_KIND_ID, // copied from the slot (SE-D2)
            impl_id: ModuleImplId::from_u128(0x2512_91e7_e995_4ef1_9efc_60a7_4114_e041),
            name: "sqlite",
            assurance: AssuranceClass::Durable,
        }
    }
}

impl EventStore for SqliteEngine {
    fn append(&mut self, event: Event) -> Result<(), StoreError> {
        let id = event.event_id.clone().ok_or(StoreError::MissingEventId)?;
        let payload = serde_json::to_string(&event)
            .map_err(|e| StoreError::Backend(format!("serialize event: {e}")))?;
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        // Persisted 0-based append-seq = current row count (append-only store,
        // no deletes, so COUNT(*) is the next sequence number).
        let next_seq: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .map_err(|e| StoreError::Backend(format!("count: {e}")))?;
        match conn.execute(
            "INSERT INTO events (seq, event_id, payload) VALUES (?1, ?2, ?3)",
            rusqlite::params![next_seq, id.as_str(), payload],
        ) {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(StoreError::DuplicateEventId(id.as_str().to_string()))
            }
            Err(e) => Err(StoreError::Backend(format!("insert: {e}"))),
        }
    }

    fn get(&self, id: &EventXgid) -> Result<Option<Event>, StoreError> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let payload: Option<String> = conn
            .query_row(
                "SELECT payload FROM events WHERE event_id = ?1",
                rusqlite::params![id.as_str()],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| StoreError::Backend(format!("get: {e}")))?;
        match payload {
            Some(p) => serde_json::from_str(&p)
                .map(Some)
                .map_err(|e| StoreError::Backend(format!("deserialize: {e}"))),
            None => Ok(None),
        }
    }

    fn range(&self, since_seq: u64) -> Result<Vec<Event>, StoreError> {
        let conn = self.conn.lock().expect("sqlite mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT payload FROM events WHERE seq >= ?1 ORDER BY seq")
            .map_err(|e| StoreError::Backend(format!("prepare: {e}")))?;
        let rows = stmt
            .query_map(rusqlite::params![since_seq as i64], |r| r.get::<_, String>(0))
            .map_err(|e| StoreError::Backend(format!("query: {e}")))?;
        let mut out = Vec::new();
        for row in rows {
            let p = row.map_err(|e| StoreError::Backend(format!("row: {e}")))?;
            out.push(
                serde_json::from_str(&p)
                    .map_err(|e| StoreError::Backend(format!("deserialize: {e}")))?,
            );
        }
        Ok(out)
    }

    fn contains(&self, id: &EventXgid) -> bool {
        // Infallible by trait signature; a healthy connection does not fail this
        // simple read (audit 4.1 limitation). On the unexpected error → false.
        let Ok(conn) = self.conn.lock() else {
            return false;
        };
        conn.query_row(
            "SELECT 1 FROM events WHERE event_id = ?1",
            rusqlite::params![id.as_str()],
            |_| Ok(()),
        )
        .optional()
        .map(|o| o.is_some())
        .unwrap_or(false)
    }

    fn len(&self) -> usize {
        let Ok(conn) = self.conn.lock() else {
            return 0;
        };
        conn.query_row("SELECT COUNT(*) FROM events", [], |r| r.get::<_, i64>(0))
            .map(|n| n as usize)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;
    use xgen_common::xgid::{IdentityXgid, RoomXgid, SpaceXgid, Xgid};
    use xgen_core::wire::types::EventType;

    fn settings_at(path: &Path) -> EngineSettings {
        let mut t = toml::map::Map::new();
        t.insert(
            "path".into(),
            toml::Value::String(path.to_string_lossy().to_string()),
        );
        EngineSettings::new(toml::Value::Table(t))
    }

    fn make_event(id: &str) -> Event {
        let mut ev = Event::new(
            EventType::MessageText,
            IdentityXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:s".to_string())),
            RoomXgid::from_xgid(Xgid::new("xgen://hash/sha256:r".to_string())),
            SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:sp".to_string())),
            vec![],
            "2026-06-03T00:00:00Z".to_string(),
            json!({ "text": id }),
        );
        ev.event_id = Some(EventXgid::from_xgid(Xgid::new(id.to_string())));
        ev
    }

    fn xid(id: &str) -> EventXgid {
        EventXgid::from_xgid(Xgid::new(id.to_string()))
    }

    #[test]
    fn descriptor_is_durable_and_copies_slot_kind() {
        let d = SqliteEngine::descriptor();
        assert_eq!(d.name, "sqlite");
        assert_eq!(d.assurance, AssuranceClass::Durable);
        assert_eq!(d.kind_id, STORAGE_ENGINE_KIND_ID);
    }

    #[test]
    fn round_trip_through_dyn_eventstore() {
        let dir = tempfile::tempdir().unwrap();
        let mut engine: Box<dyn EventStore> =
            Box::new(SqliteEngine::open(&settings_at(&dir.path().join("rt.sqlite"))).unwrap());
        engine.append(make_event("xgen://hash/sha256:a")).unwrap();
        assert_eq!(engine.len(), 1);
        assert!(engine.contains(&xid("xgen://hash/sha256:a")));
        let got = engine.get(&xid("xgen://hash/sha256:a")).unwrap().unwrap();
        assert_eq!(got.content["text"].as_str().unwrap(), "xgen://hash/sha256:a");
        assert!(engine.get(&xid("xgen://hash/sha256:nope")).unwrap().is_none());
    }

    #[test]
    fn durable_seq_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("durable.sqlite");
        {
            let mut engine = SqliteEngine::open(&settings_at(&db)).unwrap();
            engine.append(make_event("xgen://hash/sha256:a")).unwrap();
            engine.append(make_event("xgen://hash/sha256:b")).unwrap();
            engine.append(make_event("xgen://hash/sha256:c")).unwrap();
            assert_eq!(engine.len(), 3);
        } // drop closes the connection — only the file persists.

        let engine = SqliteEngine::open(&settings_at(&db)).unwrap();
        assert_eq!(engine.len(), 3, "len survives reopen");
        let all = engine.range(0).unwrap();
        let ids: Vec<&str> = all
            .iter()
            .map(|e| e.event_id.as_ref().unwrap().as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                "xgen://hash/sha256:a",
                "xgen://hash/sha256:b",
                "xgen://hash/sha256:c"
            ],
            "append/seq order survives reopen"
        );
        // Suffix semantics by persisted seq.
        assert_eq!(engine.range(2).unwrap().len(), 1);
        assert!(engine.range(3).unwrap().is_empty());
    }

    #[test]
    fn dedup_on_duplicate_event_id() {
        let dir = tempfile::tempdir().unwrap();
        let mut engine = SqliteEngine::open(&settings_at(&dir.path().join("dup.sqlite"))).unwrap();
        engine.append(make_event("xgen://hash/sha256:a")).unwrap();
        assert!(matches!(
            engine.append(make_event("xgen://hash/sha256:a")),
            Err(StoreError::DuplicateEventId(_))
        ));
        assert_eq!(engine.len(), 1);
    }

    #[test]
    fn bad_settings_is_loud_open_failure() {
        // Empty settings → required `path` missing → InvalidSettings, never a
        // silent default (SE-D5 loud-on-failure).
        assert!(matches!(
            SqliteEngine::open(&EngineSettings::empty()),
            Err(EngineError::InvalidSettings(_))
        ));
    }
}
