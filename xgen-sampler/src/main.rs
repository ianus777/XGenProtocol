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

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running xgen-sampler host");
}
