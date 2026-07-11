// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1

//! xgen-client build script.
//!
//! Besides `tauri_build::build()`, emits `XGEN_SVELTE_VERSION` — the *resolved*
//! Svelte version for the About dialog (M-RP6.1e-C2, option S-A). It is read
//! from the **committed** `ui/client/package-lock.json`, NOT `package.json`
//! (which declares the `^5` range, not a version) and NOT `node_modules`
//! (absent on a clean checkout). Falls back to "unknown" on any failure —
//! never a range, never a guess.
//!
//! Per-app frontend fact: the Svelte version lives here (not in `xgen-common`,
//! which has no UI). The node repeats this pattern in its own build.rs later.

use std::path::Path;

fn main() {
    let svelte_version = read_svelte_version().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=XGEN_SVELTE_VERSION={}", svelte_version);
    println!("cargo:rerun-if-changed=../ui/client/package-lock.json");

    tauri_build::build()
}

/// Parse `ui/client/package-lock.json` (lockfileVersion 3) for the resolved
/// `node_modules/svelte` version. Returns `None` on any read/parse/shape
/// failure so `main` can substitute "unknown".
fn read_svelte_version() -> Option<String> {
    let path = Path::new("../ui/client/package-lock.json");
    let text = std::fs::read_to_string(path).ok()?;
    let lock: serde_json::Value = serde_json::from_str(&text).ok()?;
    lock.get("packages")?
        .get("node_modules/svelte")?
        .get("version")?
        .as_str()
        .map(|s| s.to_string())
}
