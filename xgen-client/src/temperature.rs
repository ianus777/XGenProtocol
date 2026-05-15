// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Client-side temperature surface (spec Ch3 §3.7.13, Ch6 §6.12).
//
// The Tauri shell receives `xgen.room_temperature` / `xgen.member_temperature`
// values on incoming Events (via meta_atts) and emits a `temperature_update`
// event to the Svelte layer. This module owns the payload type and the bucket
// derivation logic — both bucket derivation and threshold table fallback live
// here so the Tauri shell stays a thin pass-through.

use serde::{Deserialize, Serialize};
use xgen_common::wire::TemperatureThresholds;

/// Ch6 default thresholds applied when the Node has not supplied a table
/// (spec 3.7.13.2 — clients fall back to Ch6 defaults). The numeric defaults
/// (0.25 / 0.5 / 0.75) come from Ch6 §6.12.2 and are intentionally
/// distinct from the example in spec 3.7.13.2 (0.30 / 0.55 / 0.80).
pub const DEFAULT_THRESHOLD_WARM: f64 = 0.25;
pub const DEFAULT_THRESHOLD_HOT: f64 = 0.50;
pub const DEFAULT_THRESHOLD_FIERY: f64 = 0.75;

/// Sentinel subject id for Room-level temperature updates (§6.12.3).
/// Used in the `subject_id` field of `TemperatureUpdate` when the value is
/// `xgen.room_temperature` rather than a per-member value.
pub const SUBJECT_ROOM: &str = "__room__";

/// Bucket name derived from a numeric temperature value (Ch6 §6.12.3).
/// Lowercase string values match the `data-temp-state` DOM contract.
pub const STATE_COOL: &str = "cool";
pub const STATE_WARM: &str = "warm";
pub const STATE_HOT: &str = "hot";
pub const STATE_FIERY: &str = "fiery";

/// Payload emitted to the Svelte layer via Tauri event `temperature_update`.
/// The Svelte layer projects this into `data-temp-state` and the
/// `--xgen-room-temperature` / `--xgen-member-temperature` custom properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperatureUpdate {
    pub space_id: String,
    pub room_id: String,
    /// Member identity_id, or `SUBJECT_ROOM` for Room-level temperature.
    pub subject_id: String,
    /// Numeric temperature value (clamped to `[0.0, 1.0]` upstream).
    pub temperature: f64,
    /// Derived bucket — one of `cool`, `warm`, `hot`, `fiery`.
    /// Computed once on receipt (not per frame) per Ch6 §6.12.3.
    pub state: String,
}

impl TemperatureUpdate {
    /// Construct an update for a per-member temperature value, deriving the
    /// bucket from the supplied thresholds (or the Ch6 defaults when `None`).
    pub fn for_member(
        space_id: String,
        room_id: String,
        member_id: String,
        temperature: f64,
        thresholds: Option<&TemperatureThresholds>,
    ) -> Self {
        let state = derive_state(temperature, thresholds);
        Self {
            space_id,
            room_id,
            subject_id: member_id,
            temperature,
            state: state.to_string(),
        }
    }

    /// Construct an update for the Room-level temperature.
    pub fn for_room(
        space_id: String,
        room_id: String,
        temperature: f64,
        thresholds: Option<&TemperatureThresholds>,
    ) -> Self {
        let state = derive_state(temperature, thresholds);
        Self {
            space_id,
            room_id,
            subject_id: SUBJECT_ROOM.to_string(),
            temperature,
            state: state.to_string(),
        }
    }
}

/// Derive the bucket name from a temperature value and a threshold table.
/// Falls back to Ch6 defaults when `thresholds` is `None` or the supplied
/// table fails validation (spec 3.7.13.2 — invalid tables → clients use
/// defaults).
pub fn derive_state(temperature: f64, thresholds: Option<&TemperatureThresholds>) -> &'static str {
    let (warm, hot, fiery) = match thresholds {
        Some(t) if t.is_valid() => (t.warm, t.hot, t.fiery),
        _ => (DEFAULT_THRESHOLD_WARM, DEFAULT_THRESHOLD_HOT, DEFAULT_THRESHOLD_FIERY),
    };
    // Implicit `cool` covers [0.0, warm); buckets are half-open from below.
    if temperature >= fiery {
        STATE_FIERY
    } else if temperature >= hot {
        STATE_HOT
    } else if temperature >= warm {
        STATE_WARM
    } else {
        STATE_COOL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_thresholds_match_ch6() {
        assert_eq!(DEFAULT_THRESHOLD_WARM, 0.25);
        assert_eq!(DEFAULT_THRESHOLD_HOT, 0.50);
        assert_eq!(DEFAULT_THRESHOLD_FIERY, 0.75);
    }

    #[test]
    fn derive_state_with_defaults() {
        assert_eq!(derive_state(0.0, None), STATE_COOL);
        assert_eq!(derive_state(0.24, None), STATE_COOL);
        assert_eq!(derive_state(0.25, None), STATE_WARM);
        assert_eq!(derive_state(0.49, None), STATE_WARM);
        assert_eq!(derive_state(0.50, None), STATE_HOT);
        assert_eq!(derive_state(0.74, None), STATE_HOT);
        assert_eq!(derive_state(0.75, None), STATE_FIERY);
        assert_eq!(derive_state(1.0, None), STATE_FIERY);
    }

    #[test]
    fn derive_state_with_custom_thresholds() {
        let t = TemperatureThresholds {
            warm: 0.30,
            hot: 0.55,
            fiery: 0.80,
        };
        assert_eq!(derive_state(0.29, Some(&t)), STATE_COOL);
        assert_eq!(derive_state(0.30, Some(&t)), STATE_WARM);
        assert_eq!(derive_state(0.55, Some(&t)), STATE_HOT);
        assert_eq!(derive_state(0.80, Some(&t)), STATE_FIERY);
    }

    #[test]
    fn derive_state_with_invalid_thresholds_falls_back_to_defaults() {
        let t = TemperatureThresholds { warm: 0.5, hot: 0.3, fiery: 0.8 };
        // Falls back to default thresholds: at 0.6 the default bucket is "hot".
        assert_eq!(derive_state(0.6, Some(&t)), STATE_HOT);
    }

    #[test]
    fn temperature_update_for_room_uses_subject_room_sentinel() {
        let u = TemperatureUpdate::for_room(
            "xgen://hash/sha256:space".to_string(),
            "xgen://hash/sha256:room".to_string(),
            0.4,
            None,
        );
        assert_eq!(u.subject_id, SUBJECT_ROOM);
        assert_eq!(u.state, STATE_WARM);
    }

    #[test]
    fn temperature_update_for_member_carries_member_id() {
        let u = TemperatureUpdate::for_member(
            "xgen://hash/sha256:space".to_string(),
            "xgen://hash/sha256:room".to_string(),
            "xgen://pubkey/ed25519:M".to_string(),
            0.8,
            None,
        );
        assert_eq!(u.subject_id, "xgen://pubkey/ed25519:M");
        assert_eq!(u.state, STATE_FIERY);
    }

    #[test]
    fn temperature_update_round_trips_through_json() {
        // Verifies the payload is suitable for Tauri emit().
        let u = TemperatureUpdate {
            space_id: "xgen://hash/sha256:space".to_string(),
            room_id: "xgen://hash/sha256:room".to_string(),
            subject_id: "xgen://pubkey/ed25519:M".to_string(),
            temperature: 0.42,
            state: STATE_WARM.to_string(),
        };
        let json = serde_json::to_string(&u).unwrap();
        let parsed: TemperatureUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, u);
    }
}
