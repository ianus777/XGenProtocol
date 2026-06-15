// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! Content-addressed blob store (M12.1, M12-D4).
//!
//! Holds opaque byte blobs keyed by their own SHA-256 content hash
//! (`blob_ref = hash_uri(bytes)`, V2 — the same scheme as `event_id`). The store
//! never interprets the bytes: M12.1 only ever hands it **ciphertext** (each blob
//! is per-blob-encrypted client-side before upload, M12-D5), so the store is
//! **content-blind by construction** (W2). It is entirely separate from the event
//! store (`EventStore` is event-only, audit M12-A-03); events + their referenced
//! blobs back up as one unit because `blobs_dir` hangs off `data_dir` next to
//! `spaces_dir` (M12-A-03).
//!
//! On-disk layout: one file per blob, named `<sha256-hex>.blob` under `dir`.
//!
//! Reject codes (M12-D9, V4) live in [`BlobError`], a **parallel** type to
//! `ExchangeError` (which gates the signed-envelope path; the small descriptor
//! event still rides that path). Domain **10 = 10000–10999** (attachments/blobs)
//! per the error-code register — clear of every allocated band (≤ ~6xxx live);
//! re-grep the register before adding codes (RC-F-01 / M10.1 collision
//! discipline). `10001` is the only code M12.1 emits; `10002 blob_too_large`
//! (F6 → M12.2) and `10003 blob_unavailable` (F3 → M12.3) are reserved for their
//! sub-arcs and added with their producers.

use std::fs;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::crypto::hashing::hash_uri;

/// The canonical hash-URI prefix (`xgen://hash/sha256:<64-hex>`); the blob
/// filename is the `<hex>` tail. Kept in sync with `crypto::hashing::hash_uri`.
const HASH_URI_PREFIX: &str = "xgen://hash/sha256:";

/// Errors from the blob store / transfer-ingest gate (M12-D9). A **parallel**
/// error type to `ExchangeError` and `StoreError` — neither is the right home
/// (audit M12-A-09). Domain 10 (10000–10999).
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum BlobError {
    /// Content-address integrity failure: the bytes do not hash to their
    /// claimed `blob_ref` (on upload-ingest) or to their on-disk filename (on
    /// fetch). Wire code **10001**. The W3 witness.
    #[error("blob hash mismatch: content does not match its content-address")]
    HashMismatch,

    /// The `blob_ref` is not a well-formed `xgen://hash/sha256:<64-hex>` URI.
    /// Internal/garbage input — no wire code.
    #[error("malformed blob reference")]
    MalformedRef,

    /// A filesystem I/O failure reaching the blob store. Internal — no wire code.
    /// Carried as `String` (like `StoreError::Backend`) so `BlobError` stays
    /// `Clone + PartialEq + Eq`.
    #[error("blob store I/O error: {0}")]
    Io(String),
}

impl BlobError {
    /// Map to the protocol wire `(code, name)` tuple. Only `HashMismatch` is a
    /// protocol reject the peer correlates; `MalformedRef`/`Io` are internal.
    ///
    /// Reserved (added with their producers): `10002 blob_too_large` (F6,
    /// M12.2), `10003 blob_unavailable` (F3, M12.3).
    pub fn to_wire_code(&self) -> Option<(u32, &'static str)> {
        match self {
            Self::HashMismatch => Some((10001, "blob_hash_mismatch")),
            Self::MalformedRef | Self::Io(_) => None,
        }
    }
}

/// A content-addressed, content-blind blob store (M12-D4).
pub struct BlobStore {
    dir: PathBuf,
}

impl BlobStore {
    /// Open a blob store rooted at `dir`. The directory is created lazily on the
    /// first `put`; `new` does no I/O.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// The on-disk path for a `blob_ref`. Errors on a malformed reference (not
    /// the `xgen://hash/sha256:<64-hex>` form).
    fn path_for(&self, blob_ref: &str) -> Result<PathBuf, BlobError> {
        let hex = blob_ref
            .strip_prefix(HASH_URI_PREFIX)
            .ok_or(BlobError::MalformedRef)?;
        if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(BlobError::MalformedRef);
        }
        Ok(self.dir.join(format!("{hex}.blob")))
    }

    /// Store `ciphertext` content-addressed; returns its `blob_ref`
    /// (`hash_uri(ciphertext)`, V2). Idempotent: storing identical bytes twice
    /// is a no-op returning the same ref (the defining property of a
    /// content-addressed store).
    pub fn put(&self, ciphertext: &[u8]) -> Result<String, BlobError> {
        let blob_ref = hash_uri(ciphertext);
        let path = self.path_for(&blob_ref)?;
        if !path.exists() {
            fs::create_dir_all(&self.dir).map_err(|e| BlobError::Io(e.to_string()))?;
            fs::write(&path, ciphertext).map_err(|e| BlobError::Io(e.to_string()))?;
        }
        Ok(blob_ref)
    }

    /// Fetch a blob by `blob_ref`. `Ok(None)` if absent. **Verifies** the
    /// on-disk bytes still hash to `blob_ref` (content-address integrity, W3); a
    /// mismatch (on-disk corruption / substitution) is
    /// [`BlobError::HashMismatch`] (wire 10001) — never a silent return of bad
    /// bytes.
    pub fn get(&self, blob_ref: &str) -> Result<Option<Vec<u8>>, BlobError> {
        let path = self.path_for(blob_ref)?;
        match fs::read(&path) {
            Ok(bytes) => {
                if hash_uri(&bytes) != blob_ref {
                    return Err(BlobError::HashMismatch);
                }
                Ok(Some(bytes))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(BlobError::Io(e.to_string())),
        }
    }

    /// True if a blob with this ref is present (presence only, no integrity
    /// read). A malformed ref is treated as absent.
    pub fn contains(&self, blob_ref: &str) -> bool {
        self.path_for(blob_ref)
            .map(|p| p.exists())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn put_then_get_roundtrip() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path());
        let data = b"ciphertext-bytes-abc";
        let blob_ref = store.put(data).unwrap();
        assert_eq!(store.get(&blob_ref).unwrap(), Some(data.to_vec()));
    }

    #[test]
    fn put_returns_content_address() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path());
        let data = b"hello";
        assert_eq!(store.put(data).unwrap(), hash_uri(data));
    }

    #[test]
    fn content_address_keying_is_idempotent() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path());
        let data = b"same-bytes";
        let r1 = store.put(data).unwrap();
        let r2 = store.put(data).unwrap();
        assert_eq!(r1, r2);
        // Exactly one file on disk — identical content addresses to one blob.
        let count = std::fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn get_absent_is_none() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path());
        let absent = hash_uri(b"never-stored");
        assert_eq!(store.get(&absent).unwrap(), None);
        assert!(!store.contains(&absent));
    }

    #[test]
    fn contains_reflects_presence() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path());
        let blob_ref = store.put(b"present").unwrap();
        assert!(store.contains(&blob_ref));
    }

    #[test]
    fn get_detects_on_disk_corruption() {
        // W3 spine: a substituted/corrupted blob fails the content-address check.
        // RED-on-revert: removing the `hash_uri(&bytes) != blob_ref` check in
        // `get` makes this return the tampered bytes instead of erroring.
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path());
        let blob_ref = store.put(b"original-content").unwrap();
        let path = store.path_for(&blob_ref).unwrap();
        std::fs::write(&path, b"tampered-different-bytes").unwrap();
        assert_eq!(store.get(&blob_ref), Err(BlobError::HashMismatch));
    }

    #[test]
    fn malformed_ref_errors() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path());
        assert_eq!(store.get("not-a-hash-uri"), Err(BlobError::MalformedRef));
        assert_eq!(
            store.get("xgen://hash/sha256:tooshort"),
            Err(BlobError::MalformedRef)
        );
        assert!(!store.contains("not-a-hash-uri"));
    }

    #[test]
    fn hash_mismatch_wire_code_is_10001() {
        assert_eq!(
            BlobError::HashMismatch.to_wire_code(),
            Some((10001, "blob_hash_mismatch"))
        );
        assert_eq!(BlobError::MalformedRef.to_wire_code(), None);
        assert_eq!(BlobError::Io("x".into()).to_wire_code(), None);
    }
}
