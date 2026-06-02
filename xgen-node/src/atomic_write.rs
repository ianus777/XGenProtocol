// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Atomic file write — the EventStore durability floor (ES-D3, F-1).
//!
//! The vanilla file backend persists each Space as a whole JSON array, rewritten
//! on every accepted event. A plain `fs::write` truncates the live file before
//! writing the new bytes, so a crash mid-write leaves a truncated / partial JSON
//! array and the entire Space's history is unreadable on the next start (audit
//! F-1, a Critical). [`atomic_write`] closes that window: it writes a sibling
//! temp file, fsyncs it, then atomically renames it over the destination, so the
//! live file is *always* either the previous complete version or the new
//! complete version — never a partial one.
//!
//! This is a minimal no-data-loss floor on the vanilla backend, **not** a
//! storage engine. A future engine module (SQLite/redb) unifies index +
//! durability behind the `EventStore` trait and supersedes this helper.

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Atomically write `bytes` to `path`.
///
/// Sequence: write `<path>.tmp` → `File::sync_all` (flush the data to the
/// device) → `fs::rename(tmp, path)`. The rename is atomic on both POSIX
/// (`rename(2)`) and Windows (`std::fs::rename` issues `MoveFileExW` with
/// `MOVEFILE_REPLACE_EXISTING`), so a reader or a crash never observes a
/// partial destination file.
///
/// On Unix the containing directory is fsynced after the rename so the rename
/// itself is durable across power loss; this is a `#[cfg(unix)]` split, not a
/// silent skip — Windows exposes no directory-handle fsync and the
/// `MoveFileExW` metadata write is already ordered (§5.4 / ES-D3).
///
/// The temp file is a sibling in the same directory so the rename stays on one
/// filesystem (a cross-filesystem rename is not atomic and would fail `EXDEV`).
pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let dir = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "path has no parent directory")
    })?;

    let tmp = tmp_path(path);

    // Write + flush + fsync the temp file before any rename touches the live
    // destination. If any of these fail, the live file is untouched.
    {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }

    // Atomic replace of the live file.
    fs::rename(&tmp, path)?;

    // Durably record the rename in the directory entry (POSIX only — Windows
    // has no directory-handle fsync equivalent, and MoveFileExW already orders
    // the metadata write).
    #[cfg(unix)]
    {
        if let Ok(d) = File::open(dir) {
            // Best-effort: a failed dir-fsync does not mean the data was lost
            // (the file rename already succeeded), so do not fail the write.
            let _ = d.sync_all();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = dir; // `dir` is only read on unix; keep the binding used.
    }

    Ok(())
}

/// Sibling temp path: `<path>` + `.tmp`.
fn tmp_path(path: &Path) -> PathBuf {
    let mut s: OsString = path.as_os_str().to_owned();
    s.push(".tmp");
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_writes_and_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("f.json");

        atomic_write(&dest, b"first").unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"first");

        atomic_write(&dest, b"second-longer").unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"second-longer");
    }

    #[test]
    fn leaves_no_tmp_file_behind() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("f.json");
        atomic_write(&dest, b"data").unwrap();
        assert!(!tmp_path(&dest).exists(), "temp file must be renamed away");
    }

    #[test]
    fn missing_parent_errors_and_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("no_such_dir").join("f.json");
        assert!(atomic_write(&dest, b"data").is_err());
        assert!(!dest.exists());
    }

    #[test]
    fn injected_failure_leaves_existing_file_intact() {
        // Inject a mid-write failure by occupying the temp path with a
        // directory, so `File::create(<path>.tmp)` fails *before* the rename —
        // the live destination must keep its previous content (F-1 property).
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("f.json");
        fs::write(&dest, b"ORIGINAL").unwrap();
        fs::create_dir(tmp_path(&dest)).unwrap();

        let result = atomic_write(&dest, b"NEW");
        assert!(result.is_err(), "write must fail when the temp path is unusable");
        assert_eq!(
            fs::read(&dest).unwrap(),
            b"ORIGINAL",
            "the live file must be untouched on a failed write"
        );
    }
}
