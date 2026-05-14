// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Node-side KeyPackage store (spec 3.10.3).
//
// The Node stores MLS KeyPackages uploaded by clients and distributes them
// when a new member is added to an MLS group. KeyPackages are single-use:
// once distributed for a Welcome message, they are deleted.
//
// Storage key: (identity_id, device_id) → pool of KeyPackages (first-in, first-out).
// The Node maintains at least 3 unused KeyPackages per client device.

use std::collections::HashMap;

// ── KeyPackage record ─────────────────────────────────────────────────────────

/// A KeyPackage as stored by the Node (spec 3.10.3).
/// The `mls_key_package` field is an opaque base64url-encoded blob.
/// The Node does not inspect the MLS internals — it stores and distributes as-is.
#[derive(Debug, Clone)]
pub struct StoredKeyPackage {
    pub identity_id: String,
    pub device_id: String,
    /// Opaque base64url-encoded TLS-serialised MLS KeyPackage (RFC 9420 §4).
    pub mls_key_package: String,
    pub uploaded_at: String,
    pub valid_until: String,
}

// ── KeyPackage store ──────────────────────────────────────────────────────────

/// Node-side KeyPackage pool.
/// Keyed by `(identity_id, device_id)` → FIFO pool of packages.
#[derive(Debug, Default)]
pub struct KeyPackageStore {
    pools: HashMap<(String, String), Vec<StoredKeyPackage>>,
}

impl KeyPackageStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a KeyPackage uploaded by a client.
    pub fn store(&mut self, pkg: StoredKeyPackage) {
        let key = (pkg.identity_id.clone(), pkg.device_id.clone());
        self.pools.entry(key).or_default().push(pkg);
    }

    /// Retrieve and consume the next available KeyPackage for `(identity_id, device_id)`.
    /// Returns `None` if no packages are available.
    /// The package is removed from the pool (single-use).
    pub fn consume(&mut self, identity_id: &str, device_id: &str) -> Option<StoredKeyPackage> {
        let key = (identity_id.to_string(), device_id.to_string());
        let pool = self.pools.get_mut(&key)?;
        if pool.is_empty() {
            return None;
        }
        Some(pool.remove(0))
    }

    /// Count available (unused) KeyPackages for `(identity_id, device_id)`.
    pub fn available_count(&self, identity_id: &str, device_id: &str) -> usize {
        let key = (identity_id.to_string(), device_id.to_string());
        self.pools.get(&key).map(Vec::len).unwrap_or(0)
    }

    /// Total KeyPackages stored across all devices.
    pub fn total_count(&self) -> usize {
        self.pools.values().map(Vec::len).sum()
    }

    /// Discard all expired KeyPackages (valid_until < now_timestamp).
    /// Returns the number of packages discarded.
    pub fn discard_expired(&mut self, now_timestamp: &str) -> usize {
        let mut discarded = 0;
        for pool in self.pools.values_mut() {
            let before = pool.len();
            pool.retain(|pkg| pkg.valid_until.as_str() >= now_timestamp);
            discarded += before - pool.len();
        }
        discarded
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(identity_id: &str, device_id: &str, kp: &str) -> StoredKeyPackage {
        StoredKeyPackage {
            identity_id: identity_id.to_string(),
            device_id: device_id.to_string(),
            mls_key_package: kp.to_string(),
            uploaded_at: "2026-05-14T10:00:00.000Z".to_string(),
            valid_until: "2026-08-12T10:00:00.000Z".to_string(),
        }
    }

    #[test]
    fn key_package_stored_and_retrieved() {
        let mut store = KeyPackageStore::new();
        store.store(sample("alice", "alice-dev1", "KP_ALICE_1"));
        store.store(sample("alice", "alice-dev1", "KP_ALICE_2"));

        assert_eq!(store.available_count("alice", "alice-dev1"), 2);

        let pkg = store.consume("alice", "alice-dev1").unwrap();
        assert_eq!(pkg.mls_key_package, "KP_ALICE_1"); // FIFO
        assert_eq!(store.available_count("alice", "alice-dev1"), 1);
    }

    #[test]
    fn key_package_deleted_after_use() {
        let mut store = KeyPackageStore::new();
        store.store(sample("bob", "bob-dev1", "KP_BOB_1"));

        let pkg = store.consume("bob", "bob-dev1").unwrap();
        assert_eq!(pkg.mls_key_package, "KP_BOB_1");

        // Package is gone after consumption.
        assert!(store.consume("bob", "bob-dev1").is_none());
        assert_eq!(store.available_count("bob", "bob-dev1"), 0);
    }

    #[test]
    fn expired_key_packages_discarded() {
        let mut store = KeyPackageStore::new();
        store.store(sample("alice", "dev1", "KP_CURRENT"));

        // Package with past valid_until.
        store.store(StoredKeyPackage {
            identity_id: "alice".to_string(),
            device_id: "dev1".to_string(),
            mls_key_package: "KP_EXPIRED".to_string(),
            uploaded_at: "2025-01-01T00:00:00.000Z".to_string(),
            valid_until: "2025-04-01T00:00:00.000Z".to_string(), // in the past
        });

        let discarded = store.discard_expired("2026-05-14T00:00:00.000Z");
        assert_eq!(discarded, 1);
        assert_eq!(store.available_count("alice", "dev1"), 1);
        let pkg = store.consume("alice", "dev1").unwrap();
        assert_eq!(pkg.mls_key_package, "KP_CURRENT");
    }
}
