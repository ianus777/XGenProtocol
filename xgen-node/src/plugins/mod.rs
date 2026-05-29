// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Node plugin interfaces. Phase 2 ships only the temperature plugin trait
// surface (spec 3.7.13.5); the loader / dispatcher mechanism is deferred.

pub mod temperature;

use serde::Serialize;

/// Static descriptor of a plugin compiled into this Node binary.
///
/// M6 has **no dynamic plugin loader** (A7-D1; see the module note above): the
/// set of plugins is fixed at compile time, so this is the honest "registry" —
/// a list of what is built in, not a lifecycle-managed store. There is no
/// per-plugin telemetry (events consumed / last activity) in M6, so the A7
/// `plugin status` verb reports those as `None`.
#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    /// Concrete plugin name (the loaded implementation).
    pub name: String,
    /// Version the plugin ships at — the Node binary's version, since plugins
    /// are compiled in (no independent versioning in M6).
    pub version: String,
    /// Lifecycle status. `loaded` = compiled in and active.
    pub status: String,
    /// Plugin category / slot.
    pub kind: String,
}

/// The plugins compiled into this Node — the honest static registry (A7-D1).
///
/// Exactly one today: the temperature plugin slot, whose loaded implementation
/// is the no-op placeholder (`temperature::NoOpTemperaturePlugin`, returned by
/// `temperature::load_default_plugin`). A second entry appears only when a
/// second plugin is compiled in (which is also the trigger for A7's deferred
/// WRITE verbs).
pub fn installed_plugins() -> Vec<PluginInfo> {
    vec![PluginInfo {
        name: "noop-temperature".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        status: "loaded".to_string(),
        kind: "temperature".to_string(),
    }]
}
