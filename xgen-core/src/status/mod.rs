// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! Self-set identity status (PROTO-STATUS).
//!
//! A dedicated, identity-scoped, **global** state object — *not* a field on
//! `IdentityRecord` (PROTO-STATUS.0 A2): status changes often and is
//! low-stakes, so folding it into the identity record would pollute the
//! identity version history (3.6.8) and force heavyweight identity federation
//! for a mood line. It lives at the logical key `state.status/<identity_xgid>`,
//! owner-writable only, public, global-scoped.
//!
//! Because status is identity-scoped and global it does **not** ride the
//! per-Space DAG resolution (`resolution::derive_resolved` builds a
//! `SpaceState`; status is not Space state). "Register under existing `state.*`
//! machinery" (PROTO-STATUS.2 runbook) means it reuses the same *conventions*:
//! the [`StateKey`] namespace (category `state.status`, key field = the owning
//! identity's XGID — see [`status_state_key`]), a per-object monotonic
//! `update_version`, an owner-write guard, clear-by-delete, and lazy expiry.
//! There is **no new sync primitive** — a status is just another versioned
//! state object.
//!
//! Semantics (PROTO-STATUS.1):
//!
//! - **Caps** — `emoji` is exactly one Unicode grapheme cluster; `text` is
//!   ≤128 bytes UTF-8, trimmed (whitespace-only → treated as absent);
//!   `expires_at` ∈ `[now+60s, now+30d]`.
//! - **Clear = delete** the object ([`StatusStore::clear`]); absence = no
//!   status. No tombstone-vs-empty ambiguity.
//! - **Lazy expiry** — readers treat `expires_at < now` as absent
//!   ([`StatusStore::get`]); there is no active sweep.
//! - **Owner-write** — only the identity named by the key may write its own
//!   status ([`StatusStore::set`] / [`StatusStore::clear`]).

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

use xgen_common::xgid::IdentityXgid;

use crate::resolution::StateKey;

/// Minimum lifetime of an `expires_at` relative to the write instant (60 s).
pub const EXPIRES_MIN_SECS: i64 = 60;
/// Maximum lifetime of an `expires_at` relative to the write instant (30 d).
pub const EXPIRES_MAX_DAYS: i64 = 30;
/// Maximum UTF-8 byte length of `text` after trimming (128 bytes).
pub const TEXT_MAX_BYTES: usize = 128;

/// State-key category for status objects (`state.status`), reusing the
/// `resolution::StateKey` namespace convention.
pub const STATUS_CATEGORY: &str = "state.status";

// ── Errors ──────────────────────────────────────────────────────────────────

/// Validation and write-guard failures for [`StatusRecord`] / [`StatusStore`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StatusError {
    /// `emoji` was present but was not exactly one Unicode grapheme cluster
    /// (empty, or two-or-more clusters).
    #[error("status emoji must be exactly one grapheme cluster")]
    EmojiNotSingleGrapheme,

    /// `text` exceeded [`TEXT_MAX_BYTES`] bytes UTF-8 after trimming.
    #[error("status text exceeds {TEXT_MAX_BYTES} bytes after trimming")]
    TextTooLong,

    /// `expires_at` fell outside `[now+60s, now+30d]`.
    #[error("status expires_at must be within [now+60s, now+30d]")]
    ExpiresOutOfRange,

    /// A write targeted an identity other than the writer's own status object
    /// (owner-write guard).
    #[error("status write rejected: writer is not the owner of this status")]
    NotOwner,
}

// ── Record type ─────────────────────────────────────────────────────────────

/// A self-set status. Both content fields are optional; a status carries an
/// emoji, a text line, or both.
///
/// Construct through [`StatusRecord::new`] (the validating write-path
/// constructor) — direct field construction bypasses the caps. Absent
/// optionals are omitted from the serialised form (`null` is never written —
/// PROTO-STATUS.2 runbook).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusRecord {
    /// Exactly one Unicode grapheme cluster when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    /// Description line; ≤128 bytes UTF-8, trimmed. Whitespace-only input is
    /// coerced to absent at construction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Write instant (RFC-3339 UTC on the wire).
    pub updated_at: DateTime<Utc>,
    /// Optional auto-clear instant; when set, within `[now+60s, now+30d]` of
    /// the write instant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl StatusRecord {
    /// Validating write-path constructor. `updated_at` is stamped to `now`.
    ///
    /// Rejects (PROTO-STATUS.1 §3):
    /// - `emoji = Some(_)` that is not exactly one grapheme cluster,
    /// - `text` whose trimmed byte length exceeds [`TEXT_MAX_BYTES`],
    /// - `expires_at` outside `[now + 60s, now + 30d]` (bounds inclusive).
    ///
    /// Coerces whitespace-only `text` to absent (`None`), and stores `text`
    /// trimmed.
    pub fn new(
        emoji: Option<String>,
        text: Option<String>,
        expires_at: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<Self, StatusError> {
        // emoji — exactly one grapheme cluster when present.
        if let Some(ref e) = emoji {
            if e.graphemes(true).count() != 1 {
                return Err(StatusError::EmojiNotSingleGrapheme);
            }
        }

        // text — trim; whitespace-only → absent; cap the trimmed byte length.
        let text = match text {
            Some(t) => {
                let trimmed = t.trim();
                if trimmed.is_empty() {
                    None
                } else if trimmed.len() > TEXT_MAX_BYTES {
                    return Err(StatusError::TextTooLong);
                } else {
                    Some(trimmed.to_string())
                }
            }
            None => None,
        };

        // expires_at — within [now+60s, now+30d] inclusive.
        if let Some(exp) = expires_at {
            let min = now + Duration::seconds(EXPIRES_MIN_SECS);
            let max = now + Duration::days(EXPIRES_MAX_DAYS);
            if exp < min || exp > max {
                return Err(StatusError::ExpiresOutOfRange);
            }
        }

        Ok(Self { emoji, text, updated_at: now, expires_at })
    }

    /// Lazy-expiry predicate: `true` iff an `expires_at` is set and strictly
    /// precedes `now`. A status with no `expires_at` never expires.
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.is_some_and(|e| e < now)
    }
}

// ── Store (resolution wiring) ───────────────────────────────────────────────

/// The logical [`StateKey`] for an identity's status object:
/// `state.status:<identity_xgid>`. Reuses the `resolution::StateKey` namespace
/// so status is addressed under the existing `state.*` machinery.
pub fn status_state_key(identity: &IdentityXgid) -> StateKey {
    StateKey::new(STATUS_CATEGORY, identity.as_str())
}

/// A stored status carrying its per-object monotonic version.
#[derive(Debug, Clone, PartialEq, Eq)]
struct VersionedStatus {
    record: StatusRecord,
    update_version: u64,
}

/// Identity-scoped store of self-set statuses, one object per identity keyed at
/// `state.status/<identity_xgid>`.
///
/// Enforces the owner-write guard, carries a per-object monotonic
/// `update_version`, clears by deletion, and reads through lazy expiry.
#[derive(Debug, Clone, Default)]
pub struct StatusStore {
    entries: HashMap<IdentityXgid, VersionedStatus>,
}

impl StatusStore {
    /// A new, empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Write `record` as `subject`'s status. `writer` must equal `subject`
    /// (owner-write guard) or the write is rejected with
    /// [`StatusError::NotOwner`]. Returns the new monotonic `update_version`
    /// (first write → 1, each subsequent write increments).
    pub fn set(
        &mut self,
        subject: &IdentityXgid,
        writer: &IdentityXgid,
        record: StatusRecord,
    ) -> Result<u64, StatusError> {
        if writer != subject {
            return Err(StatusError::NotOwner);
        }
        let next = self.entries.get(subject).map_or(1, |v| v.update_version + 1);
        self.entries
            .insert(subject.clone(), VersionedStatus { record, update_version: next });
        Ok(next)
    }

    /// Clear (`delete`) `subject`'s status. `writer` must equal `subject`
    /// (owner-write guard). Returns `true` if an object was removed, `false`
    /// if there was nothing to clear.
    pub fn clear(
        &mut self,
        subject: &IdentityXgid,
        writer: &IdentityXgid,
    ) -> Result<bool, StatusError> {
        if writer != subject {
            return Err(StatusError::NotOwner);
        }
        Ok(self.entries.remove(subject).is_some())
    }

    /// Read `subject`'s status as of `now`, applying lazy expiry: an expired
    /// object reads as absent (`None`) but is **not** removed (no active
    /// sweep). Absent objects also read as `None`.
    pub fn get(&self, subject: &IdentityXgid, now: DateTime<Utc>) -> Option<&StatusRecord> {
        self.entries
            .get(subject)
            .map(|v| &v.record)
            .filter(|r| !r.is_expired(now))
    }

    /// The stored `update_version` for `subject`, ignoring expiry (an expired
    /// object is still stored until overwritten or cleared).
    pub fn version(&self, subject: &IdentityXgid) -> Option<u64> {
        self.entries.get(subject).map(|v| v.update_version)
    }

    /// Number of stored objects, expired-or-not (lazy expiry does not shrink
    /// the store). Primarily an observability / test aid.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the store holds no objects at all.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use xgen_common::xgid::Xgid;

    fn id(s: &str) -> IdentityXgid {
        IdentityXgid::from_xgid(Xgid::new(format!("xgen://pubkey/ed25519:{s}")))
    }

    fn t0() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-07-05T12:00:00Z").unwrap().with_timezone(&Utc)
    }

    // ── emoji cap ────────────────────────────────────────────────────────────

    #[test]
    fn emoji_single_grapheme_accepted() {
        // A plain single-codepoint emoji.
        let r = StatusRecord::new(Some("😀".to_string()), None, None, t0()).unwrap();
        assert_eq!(r.emoji.as_deref(), Some("😀"));

        // A ZWJ sequence (family) is a single grapheme cluster made of many
        // codepoints — the char-count would be > 1, the grapheme count is 1.
        let family = "👨‍👩‍👧‍👦";
        assert!(family.chars().count() > 1, "precondition: family is multi-codepoint");
        StatusRecord::new(Some(family.to_string()), None, None, t0())
            .expect("ZWJ family emoji is one grapheme cluster");

        // A skin-tone modifier is also one grapheme cluster over two codepoints.
        let thumbs = "👍🏽";
        assert!(thumbs.chars().count() > 1, "precondition: thumbs+tone is multi-codepoint");
        StatusRecord::new(Some(thumbs.to_string()), None, None, t0())
            .expect("emoji + skin-tone modifier is one grapheme cluster");
    }

    #[test]
    fn emoji_multi_grapheme_rejected() {
        let err = StatusRecord::new(Some("😀😀".to_string()), None, None, t0()).unwrap_err();
        assert_eq!(err, StatusError::EmojiNotSingleGrapheme);
    }

    #[test]
    fn emoji_empty_rejected() {
        // Empty string = zero grapheme clusters, not one → rejected.
        let err = StatusRecord::new(Some(String::new()), None, None, t0()).unwrap_err();
        assert_eq!(err, StatusError::EmojiNotSingleGrapheme);
    }

    // ── text cap ─────────────────────────────────────────────────────────────

    #[test]
    fn text_128_bytes_accepted() {
        let s = "a".repeat(TEXT_MAX_BYTES);
        assert_eq!(s.len(), 128);
        let r = StatusRecord::new(None, Some(s.clone()), None, t0()).unwrap();
        assert_eq!(r.text.as_deref(), Some(s.as_str()));
    }

    #[test]
    fn text_129_bytes_rejected() {
        let s = "a".repeat(TEXT_MAX_BYTES + 1);
        assert_eq!(s.len(), 129);
        let err = StatusRecord::new(None, Some(s), None, t0()).unwrap_err();
        assert_eq!(err, StatusError::TextTooLong);
    }

    #[test]
    fn text_whitespace_only_becomes_absent() {
        // PROTO-STATUS.1 §3 / test enumeration: whitespace-only → absent (not
        // an error), the record still constructs.
        let r = StatusRecord::new(None, Some("   \t\n ".to_string()), None, t0()).unwrap();
        assert_eq!(r.text, None);
    }

    #[test]
    fn text_is_trimmed_on_store() {
        let r = StatusRecord::new(None, Some("  heads down  ".to_string()), None, t0()).unwrap();
        assert_eq!(r.text.as_deref(), Some("heads down"));
    }

    // ── expires_at bounds ────────────────────────────────────────────────────

    #[test]
    fn expires_at_min_boundary_accepted() {
        let exp = t0() + Duration::seconds(EXPIRES_MIN_SECS); // now + 60s
        StatusRecord::new(None, Some("brb".to_string()), Some(exp), t0())
            .expect("now+60s is the inclusive lower bound");
    }

    #[test]
    fn expires_at_max_boundary_accepted() {
        let exp = t0() + Duration::days(EXPIRES_MAX_DAYS); // now + 30d
        StatusRecord::new(None, Some("on leave".to_string()), Some(exp), t0())
            .expect("now+30d is the inclusive upper bound");
    }

    #[test]
    fn expires_at_below_min_rejected() {
        let exp = t0() + Duration::seconds(EXPIRES_MIN_SECS - 1); // now + 59s
        let err = StatusRecord::new(None, Some("brb".to_string()), Some(exp), t0()).unwrap_err();
        assert_eq!(err, StatusError::ExpiresOutOfRange);
    }

    #[test]
    fn expires_at_above_max_rejected() {
        let exp = t0() + Duration::days(EXPIRES_MAX_DAYS + 1); // now + 31d
        let err = StatusRecord::new(None, Some("gone".to_string()), Some(exp), t0()).unwrap_err();
        assert_eq!(err, StatusError::ExpiresOutOfRange);
    }

    // ── is_expired ───────────────────────────────────────────────────────────

    #[test]
    fn is_expired_absent_expiry_never_expires() {
        let r = StatusRecord::new(Some("🎯".to_string()), None, None, t0()).unwrap();
        assert!(!r.is_expired(t0() + Duration::days(3650)));
    }

    #[test]
    fn is_expired_true_after_expiry_instant() {
        let exp = t0() + Duration::seconds(90);
        let r = StatusRecord::new(None, Some("brb".to_string()), Some(exp), t0()).unwrap();
        assert!(!r.is_expired(t0() + Duration::seconds(89)));
        assert!(!r.is_expired(exp)); // strictly-precedes: equal is not expired
        assert!(r.is_expired(exp + Duration::seconds(1)));
    }

    // ── resolution wiring: state key ─────────────────────────────────────────

    #[test]
    fn status_state_key_registers_under_state_status() {
        let alice = id("ALICE");
        let key = status_state_key(&alice);
        assert_eq!(key.category, "state.status");
        assert_eq!(key.key_field, alice.as_str());
        // Round-trips through the StateKey Display convention.
        assert_eq!(key.to_string(), format!("state.status:{}", alice.as_str()));
    }

    // ── store: owner-write guard ─────────────────────────────────────────────

    #[test]
    fn owner_write_only_rejects_non_owner() {
        let mut store = StatusStore::new();
        let alice = id("ALICE");
        let bob = id("BOB");
        let rec = StatusRecord::new(Some("🚀".to_string()), None, None, t0()).unwrap();

        // Bob cannot write Alice's status object.
        let err = store.set(&alice, &bob, rec.clone()).unwrap_err();
        assert_eq!(err, StatusError::NotOwner);
        assert!(store.is_empty());

        // Alice writing her own succeeds.
        assert_eq!(store.set(&alice, &alice, rec).unwrap(), 1);
        assert!(store.get(&alice, t0()).is_some());

        // Bob cannot clear Alice's status either.
        assert_eq!(store.clear(&alice, &bob).unwrap_err(), StatusError::NotOwner);
        assert!(store.get(&alice, t0()).is_some());
    }

    // ── store: clear = delete ────────────────────────────────────────────────

    #[test]
    fn clear_deletes_object_and_read_is_absent() {
        let mut store = StatusStore::new();
        let alice = id("ALICE");
        let rec = StatusRecord::new(Some("🟢".to_string()), None, None, t0()).unwrap();

        store.set(&alice, &alice, rec).unwrap();
        assert!(store.get(&alice, t0()).is_some());

        // Clear removes the object; the read is now absent.
        assert!(store.clear(&alice, &alice).unwrap());
        assert!(store.get(&alice, t0()).is_none());
        assert!(store.is_empty());

        // Clearing again removes nothing.
        assert!(!store.clear(&alice, &alice).unwrap());
    }

    // ── store: lazy expiry ───────────────────────────────────────────────────

    #[test]
    fn lazy_expiry_read_is_absent_without_sweep() {
        let mut store = StatusStore::new();
        let alice = id("ALICE");
        let exp = t0() + Duration::seconds(60);
        let rec = StatusRecord::new(None, Some("brb".to_string()), Some(exp), t0()).unwrap();
        store.set(&alice, &alice, rec).unwrap();

        // Before expiry: present.
        assert!(store.get(&alice, t0() + Duration::seconds(30)).is_some());

        // After expiry: reads as absent...
        assert!(store.get(&alice, exp + Duration::seconds(1)).is_none());

        // ...but the object was NOT swept — it is still stored (len unchanged,
        // version still readable, and a read at a pre-expiry instant returns it).
        assert_eq!(store.len(), 1);
        assert_eq!(store.version(&alice), Some(1));
        assert!(store.get(&alice, t0()).is_some());
    }

    // ── store: per-object monotonic update_version ───────────────────────────

    #[test]
    fn update_version_is_monotonic_per_object() {
        let mut store = StatusStore::new();
        let alice = id("ALICE");
        let r1 = StatusRecord::new(Some("1️⃣".to_string()), None, None, t0()).unwrap();
        let r2 = StatusRecord::new(Some("2️⃣".to_string()), None, None, t0()).unwrap();

        assert_eq!(store.set(&alice, &alice, r1).unwrap(), 1);
        assert_eq!(store.set(&alice, &alice, r2).unwrap(), 2);
        assert_eq!(store.version(&alice), Some(2));

        // Distinct identities keep independent version streams.
        let bob = id("BOB");
        let rb = StatusRecord::new(Some("🅱️".to_string()), None, None, t0()).unwrap();
        assert_eq!(store.set(&bob, &bob, rb).unwrap(), 1);
    }

    // ── serde: absent optionals omitted, no nulls ────────────────────────────

    #[test]
    fn serde_omits_absent_optionals_no_null() {
        let r = StatusRecord::new(Some("🟢".to_string()), None, None, t0()).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"emoji\""));
        assert!(!json.contains("\"text\""), "absent text must be omitted, got {json}");
        assert!(!json.contains("\"expires_at\""), "absent expires_at must be omitted, got {json}");
        assert!(!json.contains("null"), "null is forbidden, got {json}");

        // Round-trips.
        let back: StatusRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }
}
