// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! App-agnostic "About" environment block (M-RP6.1e-C2, B2 shape / D-107).
//!
//! Everything an About dialog shows below the app name — build date, commit,
//! toolchain versions, platform, and the runtime paths — is invisible to the
//! frontend. This module produces it as one canonical [`AboutInfo`], read over
//! the Tauri IPC boundary by a thin `get_about_info` command in the shell.
//!
//! **Layering:** `xgen-common` is the protocol-layer crate and must NOT depend
//! on `tauri` or any UI toolkit. The Tauri version (`tauri::VERSION`) and the
//! resolved Svelte version are facts the *shell* knows, so they are **passed
//! in**, never derived here. The app identity (`name`/`version`/`link`) and the
//! runtime paths (`data_dir`/`config_path`) are likewise passed in — the client
//! and node resolve those differently (per `--instance`).
//!
//! Build metadata (`built`/`commit`/`rustc`) is the one thing this module reads
//! directly, from [`crate::build_info`] — the single build-metadata surface
//! (D-067). It is NOT re-derived per app.

use serde::{Deserialize, Serialize};

use crate::build_info;

/// The app-agnostic environment block. Identical field set in every XGen app.
///
/// `#[derive(Deserialize)]` is present for symmetry with the `ops::*Result`
/// structs (round-trips through JSON) even though the shell only serialises it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AboutInfo {
    // ── Identity of the app — PASSED IN ─────────────────────────────────────
    /// Display name of the app, e.g. "XGen Client".
    pub name: String,
    /// The **calling app's own** `CARGO_PKG_VERSION` — NOT `build_info::VERSION`
    /// (that is xgen-common's version; both read 0.10.3 today, so wiring the
    /// wrong one is an invisible bug until the crate versions diverge).
    pub version: String,
    /// Project / vendor website.
    pub link: String,

    // ── Build metadata — from build_info, NOT re-derived ────────────────────
    /// Last commit-triggered rebuild timestamp (`build_info::BUILD_TIMESTAMP`).
    /// This is not "when this binary was linked": build.rs reruns only on a
    /// `.git/HEAD` change, so it can be stale under uncommitted changes. The
    /// `commit` field is the exact build identifier; render the two together.
    pub built: String,
    /// Short git SHA at last compile (`build_info::GIT_HASH`).
    pub commit: String,
    /// Rust toolchain that compiled the crate (`build_info::RUSTC_VERSION`).
    pub rustc: String,

    // ── Toolchain the app itself knows — PASSED IN ──────────────────────────
    /// Tauri version (`tauri::VERSION`, read in the shell — common has no
    /// `tauri` dep).
    pub tauri: String,
    /// Resolved Svelte version (from the app's committed `package-lock.json`,
    /// emitted by the shell's `build.rs` — never the `^5` range).
    pub svelte: String,

    // ── Environment ─────────────────────────────────────────────────────────
    /// `<OS> <ARCH>` from `std::env::consts` — honest, no OS-build-number theatre.
    pub platform: String,
    /// Directory the running executable lives in (`current_exe()`'s parent),
    /// or "unknown" if it cannot be resolved.
    pub app_dir: String,
    /// Tier-1 runtime data directory for this instance — PASSED IN.
    pub data_dir: String,
    /// Full path to this instance's config file — PASSED IN.
    pub config_path: String,
}

/// The passed-in facts `collect` cannot derive inside `xgen-common`. A named
/// struct rather than seven positional `String` arguments: every field is a
/// `String`, so positional call sites would transpose silently. `built`,
/// `commit`, `rustc`, `platform`, and `app_dir` are NOT here — those `collect`
/// fills from `build_info` / `std::env` itself.
#[derive(Debug, Clone)]
pub struct AboutParams {
    pub name: String,
    pub version: String,
    pub link: String,
    pub tauri: String,
    pub svelte: String,
    pub data_dir: String,
    pub config_path: String,
}

/// Assemble the canonical [`AboutInfo`]. Build metadata comes from
/// [`crate::build_info`]; `platform` and `app_dir` from `std::env`; everything
/// else is passed in via [`AboutParams`].
pub fn collect(params: AboutParams) -> AboutInfo {
    AboutInfo {
        name: params.name,
        version: params.version,
        link: params.link,
        built: build_info::BUILD_TIMESTAMP.to_string(),
        commit: build_info::GIT_HASH.to_string(),
        rustc: build_info::RUSTC_VERSION.to_string(),
        tauri: params.tauri,
        svelte: params.svelte,
        platform: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        app_dir: current_exe_dir(),
        data_dir: params.data_dir,
        config_path: params.config_path,
    }
}

/// The directory of the running executable, or "unknown" if it cannot be
/// resolved. Never panics — an About box must not take the app down.
fn current_exe_dir() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.display().to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

/// Client-app About payload (B2, Joe-locked). Today a zero-extension wrapper —
/// the client has no client-only About fields on the field list — kept
/// deliberately as the **typed seam**: the node's About differs by ADDITION
/// (listen port, peer count, node XGID, federation role), and the wrapper lets
/// the command's return type stay stable when the first client-only field
/// lands. See runbook §2.2 Rule-6 note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientAboutInfo {
    pub common: AboutInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> AboutParams {
        AboutParams {
            name: "XGen Client".to_string(),
            version: "0.10.3".to_string(),
            link: "https://www.alchemydump.com".to_string(),
            tauri: "2.11.1".to_string(),
            svelte: "5.55.5".to_string(),
            data_dir: "/data".to_string(),
            config_path: "/data/xgen-client_config.toml".to_string(),
        }
    }

    #[test]
    fn collect_carries_passed_in_facts_verbatim() {
        let info = collect(params());
        assert_eq!(info.name, "XGen Client");
        assert_eq!(info.version, "0.10.3");
        assert_eq!(info.link, "https://www.alchemydump.com");
        assert_eq!(info.tauri, "2.11.1");
        assert_eq!(info.svelte, "5.55.5");
        assert_eq!(info.data_dir, "/data");
        assert_eq!(info.config_path, "/data/xgen-client_config.toml");
    }

    #[test]
    fn collect_reads_build_metadata_from_build_info() {
        let info = collect(params());
        // Sourced from build_info, not the params — exact-match the consts so a
        // future re-wiring to a different surface is caught.
        assert_eq!(info.built, build_info::BUILD_TIMESTAMP);
        assert_eq!(info.commit, build_info::GIT_HASH);
        assert_eq!(info.rustc, build_info::RUSTC_VERSION);
    }

    #[test]
    fn platform_is_os_space_arch() {
        let info = collect(params());
        assert_eq!(
            info.platform,
            format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
        );
    }

    #[test]
    fn about_info_round_trips_through_json() {
        let info = collect(params());
        let json = serde_json::to_string(&info).unwrap();
        let back: AboutInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, back);
    }

    #[test]
    fn client_wrapper_serialises_with_a_common_key() {
        let wrapped = ClientAboutInfo { common: collect(params()) };
        let json = serde_json::to_string(&wrapped).unwrap();
        assert!(json.contains(r#""common""#), "got {json}");
        let back: ClientAboutInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(wrapped.common, back.common);
    }
}
