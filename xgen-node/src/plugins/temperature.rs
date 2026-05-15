// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Temperature plugin interface (spec 3.7.13.5, D-061).
//
// The mathematical model that computes temperature values is intentionally
// outside the protocol; it lives in a plugin running on the Room's home Node.
// Phase 2 ships only the trait surface and a no-op placeholder. The actual
// plugin selection mechanism (config-driven loading, dynamic libraries, WASM,
// external process) is a future Phase 2 implementation decision.

use xgen_common::wire::TemperatureThresholds;

/// Interface a temperature plugin presents to the home Node.
///
/// All methods return `Option<...>` so a plugin that does not yet have a value
/// (insufficient data, plugin disabled) can omit it. The Node simply leaves
/// the corresponding `meta_atts` key off outgoing Events when `None` is
/// returned; clients fall back to the Ch6 default thresholds (§6.12.2) when
/// `thresholds()` returns `None`.
pub trait TemperaturePlugin: Send + Sync {
    /// Current Room-level temperature value in `[0.0, 1.0]`.
    /// Attached to outgoing Events under `xgen.room_temperature` when `Some`.
    fn compute_room_temperature(&self, space_id: &str, room_id: &str) -> Option<f64>;

    /// Current per-member temperature value in `[0.0, 1.0]`.
    /// Attached to outgoing Events under `xgen.member_temperature` when `Some`
    /// and subject to per-recipient visibility filtering (spec 3.7.13.4).
    fn compute_member_temperature(
        &self,
        space_id: &str,
        room_id: &str,
        member_id: &str,
    ) -> Option<f64>;

    /// Threshold table for this Room (spec 3.7.13.2). When `None`, clients use
    /// the Ch6 default thresholds. When `Some`, the table MUST be valid per
    /// `TemperatureThresholds::is_valid` or the Node SHOULD omit it.
    fn thresholds(&self, space_id: &str, room_id: &str) -> Option<TemperatureThresholds> {
        let _ = (space_id, room_id);
        None
    }
}

/// No-op default plugin. Returns `None` for every value, which has the effect
/// of omitting `xgen.*_temperature` keys from outgoing Events and falling back
/// to Ch6 default thresholds on the client side.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpTemperaturePlugin;

impl TemperaturePlugin for NoOpTemperaturePlugin {
    fn compute_room_temperature(&self, _space_id: &str, _room_id: &str) -> Option<f64> {
        None
    }

    fn compute_member_temperature(
        &self,
        _space_id: &str,
        _room_id: &str,
        _member_id: &str,
    ) -> Option<f64> {
        None
    }
}

/// Default plugin loader. Phase 2 returns the no-op placeholder unconditionally.
/// Future phases will read a Node config field (likely `xgen-node_config.toml`)
/// to dispatch to a real plugin implementation.
pub fn load_default_plugin() -> Box<dyn TemperaturePlugin> {
    Box::new(NoOpTemperaturePlugin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_op_plugin_returns_none_for_room_temperature() {
        let p = NoOpTemperaturePlugin;
        assert_eq!(p.compute_room_temperature("space", "room"), None);
    }

    #[test]
    fn no_op_plugin_returns_none_for_member_temperature() {
        let p = NoOpTemperaturePlugin;
        assert_eq!(p.compute_member_temperature("space", "room", "member"), None);
    }

    #[test]
    fn no_op_plugin_returns_none_for_thresholds() {
        let p = NoOpTemperaturePlugin;
        assert_eq!(p.thresholds("space", "room"), None);
    }

    #[test]
    fn loader_returns_no_op_by_default() {
        let plugin = load_default_plugin();
        assert_eq!(plugin.compute_room_temperature("space", "room"), None);
    }
}
