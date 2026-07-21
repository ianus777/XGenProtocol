// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Minimal Tauri host for the XGen component Sampler (D-098, M-RP3.0).
//!
//! No protocol, no CLI, no lifecycle — unlike `xgen-client`'s `desktop::run()`,
//! this is the bare Tauri builder. It opens a decorated WebView2 window at the
//! sampler's Vite `devUrl` (configured in tauri.conf.json). The frontend hosts
//! the `core` component library; live skin tuning is Vite HMR on the canonical
//! `ui/assets/skin.css` (no fs plugin needed in dev).
//!
//! M-RP4.4 (D-101) — the host gains a tiny fs+toml capability: on start it
//! wipes any subset config it finds and regenerates `[substitutions] rules`
//! from a seed (clean-slate-on-start), then exposes it via `get_substitutions`.
//! The sampler frontend hydrates through the SAME `generate → file → load →
//! command → setRules` chain the real client uses (contract-shape fidelity, not
//! code reuse — the host can't depend on `xgen-client`, D-098), so a
//! config-backed component drops into the rewritten client/node UIs with zero
//! reprogramming. See DECISIONS.md D-101.

use std::path::{Path, PathBuf};

/// First-run substitution starter pack — **hand-synced** with `xgen-client`'s
/// `app::DEFAULT_SUBSTITUTIONS_SEED` (this is the third copy of the seed; a
/// shared-const crate is explicitly out of scope this arc — N-058).
///
/// ⚠️ **The hand-sync is unguarded by any test on this side.** Both sampler tests
/// below assert against this const symbolically, so they compare a value to itself
/// and would stay green if this copy silently drifted from the client's. The seam
/// is real and known (N-058); the guard is a human read of both consts, not cargo.
///
/// **No find may be a proper prefix of another** (D-100's first amendment): the
/// edit-side engine rescans the whole field every keystroke, so a shorter rule
/// morphs first and a longer rule it prefixes can never be typed. The seed shipped
/// until 2026-07-21 paired `-->` with `--`, and `-->` was unreachable — the whole
/// reason this value changed. The pairs are also the grammar's only worked example
/// (D-100 ①), so this list may never be emptied.
const DEFAULT_SUBSTITUTIONS_SEED: &str = "-> → | <- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒";

/// Minimal `xgen-client_config.toml`-shaped subset: only the `[substitutions]`
/// section (decision 3 — the needed slice, not the whole client/node config).
/// Contract-shape parity with the client's `SubstitutionsSection`, NOT code
/// reuse (D-098: the host pulls no protocol crates).
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SamplerConfig {
    #[serde(default)]
    substitutions: SubstitutionsSection,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SubstitutionsSection {
    #[serde(default)]
    rules: String,
}

/// Resolved path to the sampler's subset config, held as managed state so
/// `get_substitutions` reads the live file without re-resolving exe_dir.
struct ConfigPath(PathBuf);

/// `<exe_dir>/xgen-sampler_config.toml` — the sampler's own subset config
/// (Joe-locked data-dir = exe_dir). Named per the D-025 binary-prefix
/// convention (NOT `xgen-client_config.toml`) so it never collides with a real
/// client config generated in the same folder — `xgen-client` with no
/// `--instance` also defaults its data-dir to exe_dir.
fn subset_config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("xgen-sampler_config.toml")
}

/// The generator half of the real chain: write the subset config from seed.
fn write_subset_config(config_path: &Path) {
    let cfg = SamplerConfig {
        substitutions: SubstitutionsSection {
            rules: DEFAULT_SUBSTITUTIONS_SEED.to_string(),
        },
    };
    if let Ok(toml_str) = toml::to_string_pretty(&cfg) {
        let _ = std::fs::write(config_path, toml_str);
    }
}

/// D-101 clean-slate-on-start (phase-scoped) — wipe any subset config found,
/// then regenerate from seed before the frontend reads it via
/// `get_substitutions`. Config is ephemeral this phase (regenerated each
/// launch); retired when the real client/node UIs gain persistent settings.
/// See DECISIONS.md D-101 for the full why + exit condition.
fn clean_slate_config(config_path: &Path) {
    if config_path.exists() {
        let _ = std::fs::remove_file(config_path);
    }
    write_subset_config(config_path);
}

/// Minimal loader — read the file → parse the one `[substitutions] rules`
/// string → return it (empty on any failure). Contract-shape parity with the
/// client's `load_substitutions_section` (D-098), not code reuse.
fn load_substitutions(config_path: &Path) -> String {
    let Ok(text) = std::fs::read_to_string(config_path) else {
        return String::new();
    };
    toml::from_str::<SamplerConfig>(&text)
        .map(|c| c.substitutions.rules)
        .unwrap_or_default()
}

/// Returns the raw `[substitutions] rules` string (mirrors the client
/// `get_substitutions` command's shape). The sampler frontend parses it in the
/// `$common` processor store (split on ` | `, first space per pair).
#[tauri::command]
fn get_substitutions(config: tauri::State<ConfigPath>) -> String {
    load_substitutions(&config.0)
}

fn main() {
    let config_path = subset_config_path();
    // D-101 — regenerate the subset config from seed on every launch (see the
    // fn doc-comment + DECISIONS.md D-101 for the phase scope + exit condition).
    clean_slate_config(&config_path);

    tauri::Builder::default()
        .manage(ConfigPath(config_path))
        .invoke_handler(tauri::generate_handler![get_substitutions])
        .run(tauri::generate_context!())
        .expect("error while running xgen-sampler host");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D-101 clean-slate-on-start — a stale subset config is wiped +
    /// regenerated to the seed on start. (Distinct temp filenames avoid any
    /// parallel-test collision without a `tempfile` dev-dep.)
    #[test]
    fn clean_slate_regenerates_subset_config_from_seed() {
        let path = std::env::temp_dir().join("xgen-sampler-test-cleanslate.toml");
        std::fs::write(&path, "[substitutions]\nrules = \"stale old\"\n").unwrap();
        assert_eq!(load_substitutions(&path), "stale old");

        clean_slate_config(&path);

        assert_eq!(
            load_substitutions(&path),
            DEFAULT_SUBSTITUTIONS_SEED,
            "clean-slate wipes the stale config and regenerates the seed"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// Absent file → empty string (no panic; mirrors the client loader).
    #[test]
    fn load_absent_config_is_empty() {
        let path = std::env::temp_dir().join("xgen-sampler-test-absent.toml");
        let _ = std::fs::remove_file(&path);
        assert_eq!(load_substitutions(&path), "");
    }

    /// The generator → loader round-trips the seed verbatim (incl. emoji +
    /// internal-space replacements).
    #[test]
    fn write_then_load_round_trips_seed() {
        let path = std::env::temp_dir().join("xgen-sampler-test-roundtrip.toml");
        let _ = std::fs::remove_file(&path);
        write_subset_config(&path);
        assert_eq!(load_substitutions(&path), DEFAULT_SUBSTITUTIONS_SEED);
        let _ = std::fs::remove_file(&path);
    }
}
