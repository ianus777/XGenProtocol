// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! The client-side address book — this client's local, **observational**
//! projection of node-authoritative Identity records (M-RP-ADDRESS-BOOK,
//! runbook `tasks/RUNBOOK_ADDRESS_BOOK.md`; Phase-0 locked J-579/J-580).
//!
//! # What it is
//!
//! The *who-exists* substrate every future name, avatar and member row reads
//! from. Its records are filled from live Space traffic (F1 authors + F2
//! members, Step 3) via [`crate::ops::identity_get`], keyed by `identity_id`.
//!
//! # The book stores OBSERVATIONS, not current truth (Joe, 2026-07-25)
//!
//! Every [`SeenRecord`] means *"as of `last_seen`, this was the state."* Two
//! consequences bind every consumer:
//!
//! - a cached `revoked = true` is a historical fact that can never become
//!   wrong — revocation does not un-happen;
//! - a cached `revoked = false` is **also** only true as of `last_seen` ⇒
//!   staleness and absence must BOTH render as UNKNOWN, never as "fine".
//!
//! # The wire ceiling (runbook §2)
//!
//! `identity.get` → `identity.record` carries seven fields and **NOT**
//! `update_version`, `revoked`, or `trust_assertion` (`wire/types.rs:455-473`;
//! Appendix I §IV.1 — code and spec agree). Those three are book-local fields
//! here with no-opinion defaults (`0` / `false` / `None`); a fetched record
//! must never *claim* an identity is valid. `M13 Client Identity Lookup
//! Widening` (PENDING) turns them into live wire paths later — at which point
//! filling them is field-mapping, not redesign.
//!
//! This is **not** Ch2's private contact record (encrypted, user-authored
//! annotations), and **not** presence (ephemeral, Space-scoped). Distinct
//! names — `SeenRecord`, not `IdentityRecord` — keep the projection from being
//! mistaken for the node's source of truth.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use xgen_common::xgid::{IdentityXgid, NodeXgid};

/// The address-book storage file, beside `xgen-client_state.json` in the
/// instance data dir (runbook §1, `xgen-client_*` convention).
pub const ADDRESS_BOOK_FILE: &str = "xgen-client_address_book.json";

/// Default T1 eviction window for §6 E3, in days (6 months).
///
/// 📌 **PROVISIONAL development value (J-580).** Retention N is ultimately
/// per-tier and Auth-Module-declared, rising with tier (T4 ≈ keep-as-evidence);
/// this is the finite no-module / T1 floor — never `∞`. The eviction function
/// takes N as a parameter, so a per-tier caller passes its own value; this is
/// only the default T1 floor.
///
/// ⚠️ **This is NOT `TIER*_TTL_DAYS`** (`xgen-core/src/auth/tiers.rs`). Those
/// are Trust-Assertion **renewal** TTLs and run the OPPOSITE direction (shorter
/// at higher tier); retention N runs LONGER at higher tier. Same-shaped
/// numbers, opposite meaning — never reuse the TTL constants for eviction.
pub const T1_DEFAULT_RETENTION_DAYS: i64 = 182;

/// One entry in the address book: the client's projection of one Identity,
/// plus book-local fields the wire does not carry.
///
/// Deliberately **not** [`crate::ops::FetchedIdentity`] — that type is the
/// faithful wire boundary (seven fields); this is the narrower cache. It does
/// **not** store `registered_at` (trimmed J-585: provenance about them, needed
/// neither to recognise nor to route, required by no locked rule), `devices`,
/// or `ai_capabilities` — every field below earns its place against a locked
/// rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeenRecord {
    /// The Identity XGID — the book key. From the wire.
    pub identity_id: IdentityXgid,
    /// Global display name — the whole point of the cache. From the wire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// AI declaration (spec §3.6.10 transparency). From the wire; serde-default
    /// `false` matches the human-record byte-identity rule (`types.rs:468`).
    #[serde(default)]
    pub is_ai: bool,
    /// Home Node — needed to route a re-fetch. From the wire.
    pub home_node: NodeXgid,
    /// RFC-3339 observation time, stamped on every touch. **Book-local.**
    /// Shape copied from `FederatedPeer.last_seen_at` (`state.rs:118`). The
    /// input to §6 E3 eviction.
    pub last_seen: String,
    /// Ordering signal for §5 V2 merge-on-encounter. **Book-local, default 0.**
    /// ⚠️ The wire carries no `update_version` (runbook §2); a fetched record
    /// starts at the `0` floor and only a seeded/M13 record out-ranks it.
    #[serde(default)]
    pub update_version: u64,
    /// §5 revocation-on-encounter. **Book-local, default false.** ⚠️ The wire
    /// carries no `revoked` today (runbook §2, D-127 widens it under M13); a
    /// fetched record has NO opinion — `false` here means "unknown", never
    /// "known-valid".
    #[serde(default)]
    pub revoked: bool,
    /// §6 not-renewed annotation input (issuer's signed Trust Assertion, which
    /// carries `valid_until`). **Book-local, default None.** ⚠️ The wire carries
    /// no `trust_assertion` today (runbook §2), and even when reachable it is
    /// normally `None` (J-582) — so `None` means "no assertion to judge", and
    /// the not-renewed badge must render **nothing** on it, never "expired".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust_assertion: Option<Value>,
}

impl SeenRecord {
    /// Build a book record from a freshly-fetched wire identity, stamping the
    /// observation time `last_seen`.
    ///
    /// The three wire-absent fields take their no-opinion defaults
    /// (`update_version = 0`, `revoked = false`, `trust_assertion = None`) —
    /// a fetched record must never claim validity (runbook §2 wire ceiling).
    pub fn from_fetched(
        fetched: &crate::ops::FetchedIdentity,
        last_seen: impl Into<String>,
    ) -> Self {
        SeenRecord {
            identity_id: fetched.identity_id.clone(),
            display_name: fetched.display_name.clone(),
            is_ai: fetched.is_ai,
            home_node: fetched.home_node.clone(),
            last_seen: last_seen.into(),
            update_version: 0,
            revoked: false,
            trust_assertion: None,
        }
    }

    /// Whether this record's Trust Assertion has lapsed (§6 "not renewed"),
    /// derived LOCALLY from the issuer's signed `valid_until` by a date
    /// comparison against `now` — never a re-judgement of validity (the client
    /// is not the verifier; the source of truth is the issuer's stamped date,
    /// which rides inside the cached assertion).
    ///
    /// 🔒 Returns `None` when there is nothing to judge — no `trust_assertion`
    /// (the COMMON case: the fill populates it for nobody, J-582) or no
    /// parseable `valid_until`. **A badge over `None` must render NOTHING, not
    /// "expired"** — absence is "no opinion", never "lapsed" and never "fine".
    /// `Some(true)` iff a present, parseable assertion's `valid_until` is
    /// strictly before `now`; `Some(false)` iff still valid.
    ///
    /// Reads the canonical `TrustAssertion.valid_until` field (spec 3.6.9,
    /// `xgen-common/src/trust_assertion.rs`). ⚠️ SEEDED per §2 Option C —
    /// `identity.record` carries no `trust_assertion`, so this fires only on a
    /// book-internal seed today and goes live with M13. And it never evicts —
    /// an expired assertion FLAGS, the record stays (§6); display is
    /// M-RP-MEMBERS's.
    pub fn trust_lapsed(&self, now: &str) -> Option<bool> {
        let valid_until = self.trust_assertion.as_ref()?.get("valid_until")?.as_str()?;
        let until = chrono::DateTime::parse_from_rfc3339(valid_until).ok()?;
        let now_dt = chrono::DateTime::parse_from_rfc3339(now).ok()?;
        Some(now_dt > until)
    }
}

/// The address book: seen-records keyed by `identity_id`.
///
/// A [`BTreeMap`] — deterministic serialisation order, so the file is diffable
/// and tests are stable. Serialised **transparently** as the bare map (no
/// wrapper key): the book is a pure cache, so format evolution is delete-and-
/// refill (E1 is exactly that), and a top-level metadata key would be a slot
/// nothing writes today.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AddressBook {
    /// Keyed by the typed [`IdentityXgid`]. `AddressBook` serialises to disk
    /// (`save`/`load`) and crosses the Tauri IPC boundary (`get_address_book`),
    /// yet this map is INTERNAL state, not a boundary slot: under D-137 §1
    /// clause 3 a slot that is the external form at an edge stays `String` only
    /// if no internal state holds that form — and this map IS that state. The
    /// key stays typed; `#[serde(transparent)]` keeps its on-disk / on-wire
    /// bytes byte-identical to the old bare-`String` form.
    records: BTreeMap<IdentityXgid, SeenRecord>,
}

impl AddressBook {
    /// An empty book.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of records held.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the book holds no records.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// The record for `identity_id`, if held.
    pub fn get(&self, identity_id: &str) -> Option<&SeenRecord> {
        self.records.get(identity_id)
    }

    /// Whether a record for `identity_id` is held (at any freshness).
    pub fn contains(&self, identity_id: &str) -> bool {
        self.records.contains_key(identity_id)
    }

    /// Iterate the records in deterministic (`identity_id`) order.
    ///
    /// Yields the key as `&IdentityXgid` — forced by the typed map key, not a
    /// discretionary widening: a `BTreeMap<IdentityXgid, _>` has no `&String` to
    /// hand out. (Widening the `&str`-taking accessors is the separate follow-on
    /// filed under D-137's Leg B, runbook §5/§8.)
    pub fn iter(&self) -> impl Iterator<Item = (&IdentityXgid, &SeenRecord)> {
        self.records.iter()
    }

    /// Insert or replace a record wholesale (last-writer-wins). The
    /// unconditional put, used by tests and direct seeds; the fill uses the
    /// version-aware [`merge`](Self::merge).
    pub fn insert(&mut self, record: SeenRecord) {
        self.records.insert(record.identity_id.clone(), record);
    }

    /// Merge an incoming record on encounter (§5 V2): the record with the
    /// **higher `update_version` wins**; an **equal or lower** version is
    /// DISCARDED, not merged field-by-field, leaving the incumbent untouched.
    ///
    /// ⚠️ **Under the wire ceiling (runbook §2) a fetched record always carries
    /// `update_version = 0`**, so in the live fill this only ever *inserts* new
    /// identities (held ones are never re-fetched). The discard path — and
    /// therefore the "higher wins" rule — is exercised today only by **seeded**
    /// records (Option C: carol v1 vs v2), and goes fully live with
    /// `M13 Client Identity Lookup Widening`. A wire-fetched v0 can never
    /// displace a seeded higher version.
    pub fn merge(&mut self, incoming: SeenRecord) {
        match self.records.get(&incoming.identity_id) {
            // Incumbent is at least as new — discard the incoming record whole.
            Some(existing) if existing.update_version >= incoming.update_version => {}
            // No incumbent, or incoming is strictly newer — take it whole.
            _ => {
                self.records.insert(incoming.identity_id.clone(), incoming);
            }
        }
    }

    /// Advance an already-held record's `last_seen` to `now` — the observation
    /// contract (J-584): every record means *"as of `last_seen`, this was the
    /// state"*, so observing an identity in a Space drain is a **touch** and
    /// must advance its `last_seen` even when no re-fetch is needed (a held
    /// record is never re-fetched — the wire ceiling makes that a no-op). A
    /// record observed today whose `last_seen` still read January would be
    /// lying about its own central claim; this restores `SeenRecord`'s
    /// "set on every touch" contract. A no-op for an identity not held.
    pub fn touch(&mut self, identity_id: &str, now: &str) {
        if let Some(r) = self.records.get_mut(identity_id) {
            r.last_seen = now.to_string();
        }
    }

    /// E2 (targeted erasure, §6): remove one identity by id. Returns the
    /// removed record, or `None` if it was not held.
    pub fn remove(&mut self, identity_id: &str) -> Option<SeenRecord> {
        self.records.remove(identity_id)
    }

    /// Empty the book in memory — the in-RAM half of E1. The on-disk half is
    /// [`erase_file`](Self::erase_file).
    pub fn clear(&mut self) {
        self.records.clear();
    }

    /// E1 (wholesale erasure, §6): delete the on-disk book file. A subsequent
    /// [`load`](Self::load) returns an empty book (Step 2), so the client keeps
    /// working. Idempotent — deleting an absent file is not an error, so E1 may
    /// be re-run.
    pub fn erase_file(data_dir: &Path) -> Result<()> {
        let path = data_dir.join(ADDRESS_BOOK_FILE);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e).with_context(|| format!("failed to erase {}", path.display())),
        }
    }

    /// E3 (aged eviction, §6): drop records last seen strictly MORE than
    /// `max_age_days` before `now`; a record exactly AT the boundary is KEPT.
    /// Returns the number evicted. `now` is INJECTED (never the wall clock) so
    /// eviction is deterministic and unit-testable without a node clock harness.
    ///
    /// ⚠️ Eviction is by `last_seen` alone. A record whose `last_seen` does not
    /// parse is KEPT — an unreadable observation time is UNKNOWN, never "old"
    /// (the book's observational rule). This is NOT trust-assertion expiry:
    /// "not renewed" FLAGS, it never EVICTS ([`SeenRecord::trust_lapsed`]).
    pub fn evict_older_than(&mut self, now: &str, max_age_days: i64) -> usize {
        let now_dt = match chrono::DateTime::parse_from_rfc3339(now) {
            Ok(t) => t.with_timezone(&chrono::Utc),
            Err(_) => return 0, // no parseable "now" ⇒ judge nothing, evict nothing.
        };
        let before = self.records.len();
        self.records.retain(|_, r| {
            match chrono::DateTime::parse_from_rfc3339(&r.last_seen) {
                Ok(seen) => {
                    (now_dt - seen.with_timezone(&chrono::Utc)).num_days() <= max_age_days
                }
                Err(_) => true, // unparseable last_seen ⇒ UNKNOWN ⇒ keep.
            }
        });
        before - self.records.len()
    }

    /// Load the book from [`ADDRESS_BOOK_FILE`] in `data_dir`.
    ///
    /// ⚠️ **A missing file is an EMPTY BOOK, not an error** — unlike
    /// `load_client_state`, which bails. §6 E1 erases by deleting the file, so
    /// absence is a normal, supported state and the client keeps working
    /// through it.
    ///
    /// A **corrupt** file is an error, and the book does **not** overwrite it
    /// (D-065: surface the damage honestly rather than silently discarding a
    /// user's cached data and carrying on). Nothing on this path writes, so the
    /// damaged file is left exactly as found for inspection.
    pub fn load(data_dir: &Path) -> Result<Self> {
        let path = data_dir.join(ADDRESS_BOOK_FILE);
        match std::fs::read_to_string(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e)
                .with_context(|| format!("failed to read address book: {}", path.display())),
            Ok(json) => serde_json::from_str(&json)
                .with_context(|| format!("address book is corrupt: {}", path.display())),
        }
    }

    /// Persist the book to [`ADDRESS_BOOK_FILE`] in `data_dir` (pretty JSON,
    /// mirroring `write_client_state`).
    pub fn save(&self, data_dir: &Path) -> Result<()> {
        let path = data_dir.join(ADDRESS_BOOK_FILE);
        let json =
            serde_json::to_string_pretty(self).context("failed to serialise address book")?;
        std::fs::write(&path, json)
            .with_context(|| format!("failed to write {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use xgen_common::xgid::Xgid;

    fn ix(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn nx(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }

    fn rec(id: &str, name: Option<&str>, last_seen: &str) -> SeenRecord {
        SeenRecord {
            identity_id: ix(id),
            display_name: name.map(|s| s.to_string()),
            is_ai: false,
            home_node: nx("xgen://pubkey/ed25519:NODE"),
            last_seen: last_seen.to_string(),
            update_version: 0,
            revoked: false,
            trust_assertion: None,
        }
    }

    #[test]
    fn roundtrip_save_load_preserves_every_field() {
        let dir = tempdir().unwrap();
        let mut book = AddressBook::new();
        // A record with every field non-default, so a dropped field is caught.
        book.insert(SeenRecord {
            identity_id: ix("xgen://pubkey/ed25519:ALICE"),
            display_name: Some("Alice".to_string()),
            is_ai: true,
            home_node: nx("xgen://pubkey/ed25519:NODE"),
            last_seen: "2026-07-25T12:00:00Z".to_string(),
            update_version: 7,
            revoked: true,
            trust_assertion: Some(serde_json::json!({"expiry": "2027-01-01T00:00:00Z"})),
        });
        book.save(dir.path()).unwrap();

        let loaded = AddressBook::load(dir.path()).unwrap();
        assert_eq!(loaded, book, "save→load must preserve every field");
    }

    #[test]
    fn missing_file_is_empty_book_not_an_error() {
        // E1 erases by deleting the file; absence is a normal, supported state.
        let dir = tempdir().unwrap();
        let book = AddressBook::load(dir.path()).expect("missing file must NOT be an error");
        assert!(book.is_empty(), "missing file must load as an empty book");
    }

    #[test]
    fn corrupt_file_is_an_error_and_left_untouched_on_disk() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(ADDRESS_BOOK_FILE);
        let corrupt = b"{ this is not valid json";
        std::fs::write(&path, corrupt).unwrap();

        let result = AddressBook::load(dir.path());
        assert!(result.is_err(), "a corrupt book must be an error, not a silent empty book");

        // D-065: the damaged file is surfaced, never silently overwritten.
        let on_disk = std::fs::read(&path).unwrap();
        assert_eq!(on_disk, corrupt, "load must leave a corrupt file untouched on disk");
    }

    #[test]
    fn serialisation_order_is_stable_across_runs() {
        let dir1 = tempdir().unwrap();
        let dir2 = tempdir().unwrap();
        // Insert in a deliberately non-sorted order.
        let mut a = AddressBook::new();
        a.insert(rec("xgen://pubkey/ed25519:CHARLIE", Some("Charlie"), "2026-07-25T00:00:00Z"));
        a.insert(rec("xgen://pubkey/ed25519:ALICE", Some("Alice"), "2026-07-25T00:00:00Z"));
        a.insert(rec("xgen://pubkey/ed25519:BOB", Some("Bob"), "2026-07-25T00:00:00Z"));

        let mut b = AddressBook::new();
        b.insert(rec("xgen://pubkey/ed25519:BOB", Some("Bob"), "2026-07-25T00:00:00Z"));
        b.insert(rec("xgen://pubkey/ed25519:ALICE", Some("Alice"), "2026-07-25T00:00:00Z"));
        b.insert(rec("xgen://pubkey/ed25519:CHARLIE", Some("Charlie"), "2026-07-25T00:00:00Z"));

        a.save(dir1.path()).unwrap();
        b.save(dir2.path()).unwrap();
        let bytes_a = std::fs::read(dir1.path().join(ADDRESS_BOOK_FILE)).unwrap();
        let bytes_b = std::fs::read(dir2.path().join(ADDRESS_BOOK_FILE)).unwrap();
        assert_eq!(bytes_a, bytes_b, "BTreeMap must serialise deterministically regardless of insert order");
    }

    #[test]
    fn from_fetched_leaves_the_three_wire_absent_fields_at_no_opinion_defaults() {
        // The wire ceiling made concrete: a fetched record NEVER claims a
        // version, a revocation state, or a trust assertion (runbook §2).
        let fetched = crate::ops::FetchedIdentity {
            identity_id: ix("xgen://pubkey/ed25519:ERIN"),
            display_name: Some("Erin".to_string()),
            registered_at: "2026-07-25T00:00:00Z".to_string(),
            devices: vec![],
            home_node: nx("xgen://pubkey/ed25519:NODE"),
            is_ai: true,
            ai_capabilities: None,
        };
        let seen = SeenRecord::from_fetched(&fetched, "2026-07-25T12:00:00Z");

        // The wire fields carry over…
        assert_eq!(seen.identity_id.as_str(), "xgen://pubkey/ed25519:ERIN");
        assert_eq!(seen.display_name.as_deref(), Some("Erin"));
        assert!(seen.is_ai, "is_ai carries over from the wire");
        assert_eq!(seen.home_node.as_str(), "xgen://pubkey/ed25519:NODE");
        assert_eq!(seen.last_seen, "2026-07-25T12:00:00Z", "last_seen is stamped by the caller");
        // …and the three wire-absent fields hold their no-opinion defaults.
        assert_eq!(seen.update_version, 0, "no wire update_version → floor 0");
        assert!(!seen.revoked, "no wire revoked → false means UNKNOWN, not valid");
        assert!(seen.trust_assertion.is_none(), "no wire trust_assertion → None means no opinion");
    }

    // ── merge on encounter, §5 V2 (M-RP-ADDRESS-BOOK Leg D, Step 4) ────────────
    //
    // ⚠️ EVERY test below is SEEDED per §2 Option C: `identity.record` carries
    // no `update_version`, so a second record version cannot arise over the
    // wire — these fixtures write versioned records directly. The logic is
    // built and tested now so that when `M13 Client Identity Lookup Widening`
    // widens the wire, this becomes field-mapping, not new design.

    fn carol(name: &str, version: u64) -> SeenRecord {
        SeenRecord {
            identity_id: ix("xgen://pubkey/ed25519:CAROL"),
            display_name: Some(name.to_string()),
            is_ai: false,
            home_node: nx("xgen://pubkey/ed25519:NODE"),
            last_seen: "2026-07-25T00:00:00Z".to_string(),
            update_version: version,
            revoked: false,
            trust_assertion: None,
        }
    }

    #[test]
    fn merge_higher_version_replaces_lower() {
        // SEEDED (§2 Option C): carol v1 → v2.
        let mut book = AddressBook::new();
        book.insert(carol("carol", 1));
        book.merge(carol("Carol M.", 2));
        let got = book.get("xgen://pubkey/ed25519:CAROL").unwrap();
        assert_eq!(got.display_name.as_deref(), Some("Carol M."), "higher version wins");
        assert_eq!(got.update_version, 2);
    }

    #[test]
    fn merge_lower_version_is_discarded_incumbent_untouched() {
        // SEEDED (§2 Option C): a stale v1 must not displace the incumbent v2.
        let mut book = AddressBook::new();
        book.insert(carol("Carol M.", 2));
        book.merge(carol("carol", 1));
        let got = book.get("xgen://pubkey/ed25519:CAROL").unwrap();
        assert_eq!(got.display_name.as_deref(), Some("Carol M."), "incumbent v2 untouched by stale v1");
        assert_eq!(got.update_version, 2);
    }

    #[test]
    fn merge_equal_version_is_a_noop() {
        // SEEDED (§2 Option C): equal version ⇒ incumbent wins (the >= rule).
        let mut book = AddressBook::new();
        book.insert(carol("Carol M.", 2));
        book.merge(carol("Carol X.", 2));
        assert_eq!(
            book.get("xgen://pubkey/ed25519:CAROL").unwrap().display_name.as_deref(),
            Some("Carol M."),
            "equal version is a no-op; the incumbent is kept"
        );
    }

    #[test]
    fn merge_is_whole_record_not_per_field() {
        // SEEDED (§2 Option C): a higher version replaces the incumbent ENTIRELY
        // — no field-by-field merge keeping the old revoked flag or assertion.
        let mut book = AddressBook::new();
        let mut v1 = carol("carol", 1);
        v1.revoked = true;
        v1.trust_assertion = Some(serde_json::json!({ "stale": true }));
        book.insert(v1);
        let v2 = carol("Carol M.", 2); // revoked=false, trust_assertion=None
        book.merge(v2.clone());
        assert_eq!(
            book.get("xgen://pubkey/ed25519:CAROL").unwrap(),
            &v2,
            "merge replaces the whole record, not field-by-field"
        );
    }

    #[test]
    fn merge_wire_fetched_v0_never_displaces_a_seeded_higher_version() {
        // SEEDED (§2 Option C) — the whole reason the fill routes through merge.
        // A wire-fetched record ALWAYS carries update_version 0 (the wire has
        // no version, runbook §2); it must not overwrite a seeded v2.
        let mut book = AddressBook::new();
        book.insert(carol("Carol M.", 2));
        let wire = SeenRecord::from_fetched(
            &crate::ops::FetchedIdentity {
                identity_id: ix("xgen://pubkey/ed25519:CAROL"),
                display_name: Some("carol".to_string()), // stale wire name
                registered_at: "2026-07-25T00:00:00Z".to_string(),
                devices: vec![],
                home_node: nx("xgen://pubkey/ed25519:NODE"),
                is_ai: false,
                ai_capabilities: None,
            },
            "2026-07-25T13:00:00Z",
        );
        assert_eq!(wire.update_version, 0, "guard: a fetched record is always v0");
        book.merge(wire);
        let got = book.get("xgen://pubkey/ed25519:CAROL").unwrap();
        assert_eq!(got.display_name.as_deref(), Some("Carol M."), "seeded v2 survives a v0 wire re-fetch");
        assert_eq!(got.update_version, 2);
    }

    // ── erasure E1 + E2 + E3, §6 (M-RP-ADDRESS-BOOK Leg D, Step 5) ─────────────

    #[test]
    fn e1_erase_file_empties_the_book_and_it_keeps_working() {
        let dir = tempdir().unwrap();
        let mut book = AddressBook::new();
        book.insert(rec("xgen://pubkey/ed25519:A", Some("A"), "2026-07-25T00:00:00Z"));
        book.save(dir.path()).unwrap();
        assert!(dir.path().join(ADDRESS_BOOK_FILE).exists());

        AddressBook::erase_file(dir.path()).unwrap();
        assert!(!dir.path().join(ADDRESS_BOOK_FILE).exists(), "E1 deletes the file");
        let reloaded = AddressBook::load(dir.path()).expect("book keeps working after E1");
        assert!(reloaded.is_empty(), "after E1 the book reloads empty");
        // Idempotent — re-running E1 on an absent file is not an error.
        AddressBook::erase_file(dir.path()).expect("E1 on an absent file is fine");
    }

    #[test]
    fn e2_remove_drops_one_and_leaves_the_rest() {
        let mut book = AddressBook::new();
        book.insert(rec("xgen://pubkey/ed25519:A", Some("A"), "2026-07-25T00:00:00Z"));
        book.insert(rec("xgen://pubkey/ed25519:B", Some("B"), "2026-07-25T00:00:00Z"));
        assert!(book.remove("xgen://pubkey/ed25519:A").is_some(), "E2 returns the removed record");
        assert!(!book.contains("xgen://pubkey/ed25519:A"), "the targeted record is gone");
        assert!(book.contains("xgen://pubkey/ed25519:B"), "the rest remain");
        assert!(book.remove("xgen://pubkey/ed25519:GHOST").is_none(), "removing an absent id is None");
    }

    #[test]
    fn e3_evicts_past_n_keeps_the_boundary_and_recent() {
        // grace (§6 E3) is the PAST-N record — a book-internal SEEDED aged
        // record (§2 Option C-adjacent): the book's own last_seen field is the
        // seam, and `now` is injected, so no node clock harness is needed.
        let now_str = "2026-07-25T00:00:00Z";
        let now = chrono::DateTime::parse_from_rfc3339(now_str).unwrap();
        let boundary = (now - chrono::Duration::days(182)).to_rfc3339();
        let past = (now - chrono::Duration::days(183)).to_rfc3339();
        let recent = (now - chrono::Duration::days(1)).to_rfc3339();

        let mut book = AddressBook::new();
        book.insert(rec("xgen://pubkey/ed25519:BOUNDARY", Some("Boundary"), &boundary));
        book.insert(rec("xgen://pubkey/ed25519:GRACE", Some("Grace"), &past));
        book.insert(rec("xgen://pubkey/ed25519:RECENT", Some("Recent"), &recent));

        let evicted = book.evict_older_than(now_str, T1_DEFAULT_RETENTION_DAYS);
        assert_eq!(evicted, 1, "only the past-N record is evicted");
        assert!(book.contains("xgen://pubkey/ed25519:BOUNDARY"), "a record exactly at the boundary is KEPT");
        assert!(book.contains("xgen://pubkey/ed25519:RECENT"), "a recent record is kept");
        assert!(!book.contains("xgen://pubkey/ed25519:GRACE"), "grace: the past-N record is evicted");
    }

    #[test]
    fn e3_is_per_tier_a_larger_n_retains_what_t1_would_evict() {
        let now_str = "2026-07-25T00:00:00Z";
        let now = chrono::DateTime::parse_from_rfc3339(now_str).unwrap();
        let aged = (now - chrono::Duration::days(200)).to_rfc3339();

        let mut t1 = AddressBook::new();
        t1.insert(rec("xgen://pubkey/ed25519:X", Some("X"), &aged));
        assert_eq!(
            t1.evict_older_than(now_str, T1_DEFAULT_RETENTION_DAYS),
            1,
            "T1 (182d) evicts a 200-day-old record"
        );

        let mut higher = AddressBook::new();
        higher.insert(rec("xgen://pubkey/ed25519:X", Some("X"), &aged));
        assert_eq!(
            higher.evict_older_than(now_str, 365),
            0,
            "a per-tier N of 365d retains the same 200-day record"
        );
        assert!(higher.contains("xgen://pubkey/ed25519:X"));
    }

    #[test]
    fn e3_keeps_records_with_an_unparseable_last_seen() {
        let mut book = AddressBook::new();
        book.insert(rec("xgen://pubkey/ed25519:BAD", Some("Bad"), "not-a-timestamp"));
        assert_eq!(
            book.evict_older_than("2026-07-25T00:00:00Z", T1_DEFAULT_RETENTION_DAYS),
            0,
            "an unparseable last_seen is UNKNOWN, never 'old' — kept"
        );
        assert!(book.contains("xgen://pubkey/ed25519:BAD"));
    }

    #[test]
    fn e3_never_evicts_on_trust_expiry_only_on_last_seen() {
        // "Not renewed" FLAGS, never EVICTS (§6). A record with a long-EXPIRED
        // assertion but a RECENT last_seen must survive E3.
        let mut book = AddressBook::new();
        let mut r = rec("xgen://pubkey/ed25519:EXP", Some("Exp"), "2026-07-24T00:00:00Z");
        r.trust_assertion = Some(serde_json::json!({ "valid_until": "2026-01-01T00:00:00Z" })); // SEEDED
        book.insert(r);
        assert_eq!(
            book.evict_older_than("2026-07-25T00:00:00Z", T1_DEFAULT_RETENTION_DAYS),
            0,
            "an expired trust assertion does not cause eviction — it flags, it does not evict"
        );
        assert!(book.contains("xgen://pubkey/ed25519:EXP"));
    }

    // ── not-renewed derivation, §6 (trust_lapsed) ─────────────────────────────

    #[test]
    fn trust_lapsed_is_none_when_there_is_no_assertion() {
        // 🔒 THE J-582 FINDING: the fill populates trust_assertion for nobody,
        // so None is the COMMON case, and the badge must render NOTHING on it —
        // never "expired". This is the one non-seeded trust test: it is exactly
        // the state an ordinary fetched record is in.
        let r = rec("xgen://pubkey/ed25519:PLAIN", Some("Plain"), "2026-07-25T00:00:00Z");
        assert_eq!(r.trust_assertion, None, "guard: an ordinary fetched record has no assertion");
        assert_eq!(
            r.trust_lapsed("2026-07-25T00:00:00Z"),
            None,
            "no assertion ⇒ no badge state, NOT 'expired' and NOT 'fine'"
        );
    }

    #[test]
    fn trust_lapsed_some_true_when_valid_until_is_past() {
        // SEEDED per §2 Option C — frank's not-renewed case.
        let mut r = rec("xgen://pubkey/ed25519:FRANK", Some("Frank"), "2026-07-25T00:00:00Z");
        r.trust_assertion = Some(serde_json::json!({ "valid_until": "2026-01-15T00:00:00Z" }));
        assert_eq!(
            r.trust_lapsed("2026-07-25T00:00:00Z"),
            Some(true),
            "a lapsed assertion flags true"
        );
    }

    #[test]
    fn trust_lapsed_some_false_when_valid_until_is_future() {
        // SEEDED per §2 Option C.
        let mut r = rec("xgen://pubkey/ed25519:VALID", Some("Valid"), "2026-07-25T00:00:00Z");
        r.trust_assertion = Some(serde_json::json!({ "valid_until": "2027-01-01T00:00:00Z" }));
        assert_eq!(
            r.trust_lapsed("2026-07-25T00:00:00Z"),
            Some(false),
            "a still-valid assertion flags false"
        );
    }

    #[test]
    fn trust_lapsed_is_none_when_valid_until_is_missing_or_unparseable() {
        // ⚠️ The node's `set_trust_expiry` synthesises `{"expiry": …}` (corpus
        // §7), NOT the canonical `{"valid_until": …}`. The derivation reads
        // `valid_until` (the locked field, §6), so a synthesised-shape assertion
        // yields None — no opinion, not a false "expired". (The synthesised
        // shape never reaches the client anyway; the wire carries no assertion.)
        let mut r = rec("xgen://pubkey/ed25519:ODD", Some("Odd"), "2026-07-25T00:00:00Z");
        r.trust_assertion = Some(serde_json::json!({ "expiry": "2026-01-15T00:00:00Z" }));
        assert_eq!(r.trust_lapsed("2026-07-25T00:00:00Z"), None, "no parseable valid_until ⇒ no opinion");
    }

    // ── the observation contract: touch advances last_seen (J-584) ────────────

    #[test]
    fn touch_advances_last_seen_for_a_held_record_and_is_a_noop_otherwise() {
        let mut book = AddressBook::new();
        book.insert(rec("xgen://pubkey/ed25519:HELD", Some("Held"), "2026-01-01T00:00:00Z"));

        // Observing a held identity advances its last_seen — no other field moves.
        book.touch("xgen://pubkey/ed25519:HELD", "2026-07-25T12:00:00Z");
        let r = book.get("xgen://pubkey/ed25519:HELD").unwrap();
        assert_eq!(r.last_seen, "2026-07-25T12:00:00Z", "last_seen advances to the observation time");
        assert_eq!(r.display_name.as_deref(), Some("Held"), "touch moves nothing but last_seen");

        // Touching an identity not held does nothing (and does not create one).
        book.touch("xgen://pubkey/ed25519:GHOST", "2026-07-25T12:00:00Z");
        assert!(!book.contains("xgen://pubkey/ed25519:GHOST"), "touch never creates a record");
        assert_eq!(book.len(), 1);
    }

    // ── D-137 Leg B: the typed BTreeMap key round-trips byte-identically ───────
    //
    // These three exist because the retype's on-disk / on-wire invariance is an
    // ARGUMENT until it is a test (runbook §2c): a typed map KEY travels a
    // different serde path from a typed value, so it is BROKEN DELIBERATELY here
    // rather than confirmed.

    #[test]
    fn v3a_book_round_trips_to_the_bare_xgid_keyed_json_shape() {
        // The literal IS the test: a save→load equality alone would pass even if
        // both sides had changed shape together. This pins the on-disk bytes to
        // bare "xgen://pubkey/ed25519:…" keys and plain-string identity_id /
        // home_node — proving #[serde(transparent)] keeps the typed key's bytes
        // identical to the old bare-String form.
        let dir = tempdir().unwrap();
        let mut book = AddressBook::new();
        book.insert(rec("xgen://pubkey/ed25519:ALICE", Some("Alice"), "2026-07-25T00:00:00Z"));
        book.insert(rec("xgen://pubkey/ed25519:BOB", Some("Bob"), "2026-07-25T00:00:00Z"));
        book.save(dir.path()).unwrap();

        let on_disk = std::fs::read_to_string(dir.path().join(ADDRESS_BOOK_FILE)).unwrap();
        let expected = r#"{
  "xgen://pubkey/ed25519:ALICE": {
    "identity_id": "xgen://pubkey/ed25519:ALICE",
    "display_name": "Alice",
    "is_ai": false,
    "home_node": "xgen://pubkey/ed25519:NODE",
    "last_seen": "2026-07-25T00:00:00Z",
    "update_version": 0,
    "revoked": false
  },
  "xgen://pubkey/ed25519:BOB": {
    "identity_id": "xgen://pubkey/ed25519:BOB",
    "display_name": "Bob",
    "is_ai": false,
    "home_node": "xgen://pubkey/ed25519:NODE",
    "last_seen": "2026-07-25T00:00:00Z",
    "update_version": 0,
    "revoked": false
  }
}"#;
        assert_eq!(on_disk, expected, "on-disk bytes must be the bare-XGID-keyed shape");

        // …and it loads back equal — the key deserialises through the double
        // newtype IdentityXgid → Xgid → String.
        let loaded = AddressBook::load(dir.path()).unwrap();
        assert_eq!(loaded, book, "save→load preserves the typed-key book");
    }

    #[test]
    fn v3b_a_pre_retype_on_disk_file_still_loads() {
        // The one that proves nobody's existing book file breaks. This JSON is a
        // book written BEFORE the retype — a bare object keyed by XGID URI, the
        // shape a String-keyed BTreeMap produced. It must deserialise into the
        // typed map. Fed, not asserted (N-091): the typed-key deserialise path
        // had never run against a real on-disk file.
        let dir = tempdir().unwrap();
        let legacy = r#"{
  "xgen://pubkey/ed25519:ALICE": {
    "identity_id": "xgen://pubkey/ed25519:ALICE",
    "display_name": "Alice",
    "is_ai": false,
    "home_node": "xgen://pubkey/ed25519:NODE",
    "last_seen": "2026-07-25T00:00:00Z",
    "update_version": 0,
    "revoked": false
  }
}"#;
        std::fs::write(dir.path().join(ADDRESS_BOOK_FILE), legacy).unwrap();

        let book = AddressBook::load(dir.path()).expect("a pre-retype file must load");
        let alice = book.get("xgen://pubkey/ed25519:ALICE").expect("alice present after load");
        assert_eq!(alice.identity_id.as_str(), "xgen://pubkey/ed25519:ALICE");
        assert_eq!(alice.home_node.as_str(), "xgen://pubkey/ed25519:NODE");
        assert_eq!(alice.display_name.as_deref(), Some("Alice"));
    }

    #[test]
    fn v3c_borrow_str_lookup_path_is_real() {
        // D-137 §5 / runbook R3: the four accessors keep &str, driven by the
        // flavour's Borrow<str>. Prove get/contains/remove all reach a
        // typed-keyed record by a bare &str, not by a wrapped key.
        let mut book = AddressBook::new();
        book.insert(rec("xgen://pubkey/ed25519:ZED", Some("Zed"), "2026-07-25T00:00:00Z"));

        assert!(book.contains("xgen://pubkey/ed25519:ZED"), "contains by &str");
        assert!(book.get("xgen://pubkey/ed25519:ZED").is_some(), "get by &str");
        assert!(
            book.remove("xgen://pubkey/ed25519:ZED").is_some(),
            "remove by &str returns the record"
        );
        assert!(book.is_empty(), "the record is gone after remove");
    }
}
