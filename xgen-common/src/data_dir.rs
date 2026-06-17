// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M12.2b (F9, M12-D7) — shared data-root resolution for both binaries.
//!
//! The node + client each resolve their data root the same way (no drift,
//! D-067): a `--data-dir` flag or `XGEN_DATA_DIR` env override, else a
//! **platform** data dir cleanly **outside** the install folder. The default
//! must be chosen **before** config load (the config lives *under* the root), so
//! the override is a flag/env — never a config field. `--instance` rebases under
//! whatever root resolves.
//!
//! Hand-rolled per-OS lookup (no `dirs`/`directories` crate, M12.2-D5):
//! Windows → `%LOCALAPPDATA%` (fallback `%USERPROFILE%\AppData\Local`);
//! non-Windows → `$XDG_DATA_HOME` else `$HOME/.local/share` (macOS via the Unix
//! path). The app subdir is [`APP_SUBDIR`]; both binaries share it — their files
//! are `xgen-node_*` / `xgen-client_*` prefixed and `--instance` segregates.

use std::path::{Path, PathBuf};

/// The application subdirectory appended to the platform base.
pub const APP_SUBDIR: &str = "XGenProtocol";

/// Errors resolving / validating the data root.
#[derive(Debug, thiserror::Error)]
pub enum DataDirError {
    /// No `--data-dir`, no `XGEN_DATA_DIR`, and no platform base dir could be
    /// resolved (no `LOCALAPPDATA`/`USERPROFILE` on Windows, no
    /// `XDG_DATA_HOME`/`HOME` elsewhere). We **fail fast** rather than silently
    /// fall back to the install folder (M12.2-D5/VB).
    #[error(
        "could not resolve a data directory: no --data-dir, no XGEN_DATA_DIR, \
         and no platform base dir found. Pass --data-dir <path>."
    )]
    NoPlatformBase,

    /// The resolved root could not be created (M12.2-D5/VD).
    #[error("data directory {path:?} is not creatable: {reason}")]
    NotCreatable { path: PathBuf, reason: String },

    /// The resolved root is not writable (write-probe failed) (M12.2-D5/VD).
    #[error("data directory {path:?} is not writable: {reason}")]
    NotWritable { path: PathBuf, reason: String },

    /// The resolved root is under the system temp dir — data there is wiped
    /// (M12.2-D5/VD). Pass an explicit `--data-dir` outside temp.
    #[error("data directory {path:?} is under the system temp dir (data there is not durable)")]
    UnderTemp { path: PathBuf },
}

/// The OS-specific base directory (before [`APP_SUBDIR`] is appended), read from
/// the environment. `None` if no base env var is set.
pub fn platform_base() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(p) = non_empty_env("LOCALAPPDATA") {
            return Some(PathBuf::from(p));
        }
        // Fallback: %USERPROFILE%\AppData\Local
        non_empty_env("USERPROFILE").map(|u| PathBuf::from(u).join("AppData").join("Local"))
    }
    #[cfg(not(windows))]
    {
        if let Some(p) = non_empty_env("XDG_DATA_HOME") {
            return Some(PathBuf::from(p));
        }
        // Fallback: $HOME/.local/share (covers Linux + macOS — no special-case).
        non_empty_env("HOME").map(|h| PathBuf::from(h).join(".local").join("share"))
    }
}

/// The platform **default** data dir = `<platform base>/XGenProtocol`, or `None`
/// if no platform base resolves.
pub fn platform_default_data_dir() -> Option<PathBuf> {
    platform_base().map(|b| b.join(APP_SUBDIR))
}

/// Resolve the data **root** by precedence: `--data-dir` flag > `XGEN_DATA_DIR`
/// env > platform default (M12.2-D5/VC). Fails fast if none resolve. Pure over
/// its inputs (the env var is read by the caller) so precedence is unit-testable;
/// only the platform-default branch reads the OS environment.
pub fn resolve_data_root(
    flag: Option<&Path>,
    env_var: Option<&str>,
) -> Result<PathBuf, DataDirError> {
    if let Some(f) = flag {
        return Ok(f.to_path_buf());
    }
    if let Some(e) = env_var {
        if !e.is_empty() {
            return Ok(PathBuf::from(e));
        }
    }
    platform_default_data_dir().ok_or(DataDirError::NoPlatformBase)
}

/// Rebase an `--instance` label under a resolved data root:
/// `<root>/instances/<label>` (M12.2-D5/VF). Shared by both binaries (no drift).
pub fn instance_path(root: &Path, label: &str) -> PathBuf {
    root.join("instances").join(label)
}

/// M12.2b (F9, D5/VD) — startup validation of the resolved data dir: it must be
/// **creatable**, **writable** (write-probe), and **not** under the system temp
/// dir (data there is wiped). Fail fast before the runtime is built.
pub fn validate_data_dir(path: &Path) -> Result<(), DataDirError> {
    std::fs::create_dir_all(path).map_err(|e| DataDirError::NotCreatable {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    // not-tmp — reject a root under the system temp dir. Canonicalise both (the
    // dir now exists) so the comparison is robust (symlinks / `..` / Windows
    // extended-length prefix).
    if let (Ok(canon), Ok(tmp)) = (path.canonicalize(), std::env::temp_dir().canonicalize()) {
        if canon.starts_with(&tmp) {
            return Err(DataDirError::UnderTemp { path: path.to_path_buf() });
        }
    }
    // writable — a write-probe (create/write/remove a marker).
    let probe = path.join(".xgen-write-test");
    std::fs::write(&probe, b"ok").map_err(|e| DataDirError::NotWritable {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    let _ = std::fs::remove_file(&probe);
    Ok(())
}

/// M12.2b (F9, D6/VE) — the leave-as-legacy notice. When the resolved root is the
/// **fresh platform default** (no `--data-dir`/`XGEN_DATA_DIR` override) and an
/// old `exe_dir` layout still holds data (a keypair or an `instances/` dir),
/// return a one-line notice naming the `--data-dir=<exe_dir>` escape — so an
/// upgrading operator is not surprised by a "fresh" node ignoring old data.
/// `None` when there is no migration concern (override used / same dir / no old
/// data). No auto-migration (D6).
pub fn legacy_data_notice(
    used_override: bool,
    resolved_root: &Path,
    exe_dir: &Path,
    keypair_name: &str,
) -> Option<String> {
    if used_override || resolved_root == exe_dir {
        return None;
    }
    let has_old = exe_dir.join(keypair_name).exists() || exe_dir.join("instances").exists();
    if !has_old {
        return None;
    }
    Some(format!(
        "notice: existing data found at {} — the default data directory moved to {} (M12.2b/F9). \
         Pass --data-dir {} to keep using the existing data.",
        exe_dir.display(),
        resolved_root.display(),
        exe_dir.display()
    ))
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_default_is_under_a_base_and_named() {
        // W1 — the default is an absolute path ending in the app subdir, outside
        // the install folder (it derives from a platform base env, never exe_dir).
        let d = platform_default_data_dir().expect("a platform base on the test machine");
        assert!(d.is_absolute(), "platform default is absolute: {d:?}");
        assert!(d.ends_with(APP_SUBDIR), "platform default ends with the app subdir: {d:?}");
    }

    #[test]
    fn resolve_precedence_flag_over_env_over_default() {
        // W2 — flag wins.
        let flag = PathBuf::from("/explicit/flag/root");
        assert_eq!(
            resolve_data_root(Some(&flag), Some("/env/root")).unwrap(),
            flag
        );
        // env wins when no flag.
        assert_eq!(
            resolve_data_root(None, Some("/env/root")).unwrap(),
            PathBuf::from("/env/root")
        );
        // empty env is ignored → falls through to the platform default.
        let both_none = resolve_data_root(None, None).unwrap();
        let empty_env = resolve_data_root(None, Some("")).unwrap();
        assert!(both_none.ends_with(APP_SUBDIR));
        assert_eq!(empty_env, both_none, "empty XGEN_DATA_DIR is treated as unset");
    }

    #[test]
    fn instance_rebases_under_root() {
        // W4 — `--instance` rebases under whatever root resolves.
        let root = PathBuf::from("/data/root");
        assert_eq!(
            instance_path(&root, "n1"),
            PathBuf::from("/data/root").join("instances").join("n1")
        );
    }

    #[test]
    fn validate_accepts_good_dir_rejects_temp() {
        // W3 (spine) — a normal writable dir validates; a dir under the system
        // temp dir is rejected. RED-on-revert: drop the not-tmp branch in
        // validate_data_dir → the temp path returns Ok → this assert fails.
        let good = tempfile::tempdir().expect("tempdir");
        // tempfile lives under temp_dir() → must be REJECTED as UnderTemp.
        assert!(
            matches!(validate_data_dir(good.path()), Err(DataDirError::UnderTemp { .. })),
            "a dir under the system temp dir must be rejected"
        );
        // A dir NOT under temp (a child of the manifest dir) validates.
        let outside = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("m12_2b_validate_probe");
        let r = validate_data_dir(&outside);
        let _ = std::fs::remove_dir_all(&outside);
        assert!(r.is_ok(), "a writable non-temp dir validates: {r:?}");
    }

    #[test]
    fn legacy_notice_fires_only_on_fresh_default_with_old_data() {
        // W5 — the D6 notice fires only when no override + an old exe_dir layout
        // holds data + the resolved root differs.
        let exe = tempfile::tempdir().expect("exe tempdir");
        let platform = PathBuf::from("/platform/XGenProtocol");
        let kp = "xgen-node_keypair.enc";

        // No old data → None.
        assert!(legacy_data_notice(false, &platform, exe.path(), kp).is_none());

        // Old data present + no override + different root → Some.
        std::fs::write(exe.path().join(kp), b"k").unwrap();
        let n = legacy_data_notice(false, &platform, exe.path(), kp);
        assert!(n.is_some(), "notice fires with old data + fresh default");
        assert!(n.unwrap().contains("--data-dir"), "notice names the escape");

        // Override used → None (operator chose explicitly).
        assert!(legacy_data_notice(true, &platform, exe.path(), kp).is_none());
        // Resolved == exe_dir → None (already using it).
        assert!(legacy_data_notice(false, exe.path(), exe.path(), kp).is_none());
    }
}
