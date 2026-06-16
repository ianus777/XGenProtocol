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
}
