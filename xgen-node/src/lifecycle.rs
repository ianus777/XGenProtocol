// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

use std::collections::HashSet;
use std::fmt;

use chrono::{SecondsFormat, Utc};

/// Node lifecycle states — Appendix E §E.1.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NodeLifecycleState {
    Initialising,
    Ready,
    DegradedFederation,
    DegradedStorage,
    DegradedAuth,
    Maintenance,
    Closing,
}

impl NodeLifecycleState {
    /// Canonical uppercase-underscore form used in log lines and JSON payloads.
    pub fn as_canonical(&self) -> &'static str {
        match self {
            NodeLifecycleState::Initialising => "INITIALISING",
            NodeLifecycleState::Ready => "READY",
            NodeLifecycleState::DegradedFederation => "DEGRADED_FEDERATION",
            NodeLifecycleState::DegradedStorage => "DEGRADED_STORAGE",
            NodeLifecycleState::DegradedAuth => "DEGRADED_AUTH",
            NodeLifecycleState::Maintenance => "MAINTENANCE",
            NodeLifecycleState::Closing => "CLOSING",
        }
    }

    /// Returns true if this state is one of the degraded conditions.
    pub fn is_degraded(&self) -> bool {
        matches!(
            self,
            Self::DegradedFederation | Self::DegradedStorage | Self::DegradedAuth
        )
    }

    /// Severity for display priority: higher = shown when multiple degraded conditions active.
    /// DegradedStorage=3, DegradedAuth=2, DegradedFederation=1, else 0.
    pub fn severity(&self) -> u8 {
        match self {
            NodeLifecycleState::DegradedStorage => 3,
            NodeLifecycleState::DegradedAuth => 2,
            NodeLifecycleState::DegradedFederation => 1,
            _ => 0,
        }
    }
}

/// Display returns the Appendix E title-case display label (shown in the UI).
impl fmt::Display for NodeLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            NodeLifecycleState::Initialising => "Initialising",
            NodeLifecycleState::Ready => "Ready",
            NodeLifecycleState::DegradedFederation => "Federation degraded",
            NodeLifecycleState::DegradedStorage => "Storage degraded",
            NodeLifecycleState::DegradedAuth => "Auth module degraded",
            NodeLifecycleState::Maintenance => "Maintenance",
            NodeLifecycleState::Closing => "Closing",
        };
        write!(f, "{}", label)
    }
}

/// Payload emitted on the `"xgen-node-state-changed"` Tauri event (D-042).
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeStateEvent {
    pub state: NodeLifecycleState,
    pub label: String,
    /// Sorted canonical names of all currently active degraded conditions.
    pub active_degraded: Vec<String>,
    /// UTC RFC 3339 timestamp with millisecond precision.
    pub timestamp: String,
}

/// Determine the display state from the primary state and the set of active degraded conditions.
///
/// Rules (in priority order):
/// 1. Closing overrides everything.
/// 2. Maintenance overrides everything except Closing.
/// 3. Initialising overrides degraded states (not yet operational).
/// 4. Among degraded conditions, highest severity wins (Storage > Auth > Federation).
/// 5. Otherwise Ready.
pub fn active_display_state(
    primary: &NodeLifecycleState,
    degraded: &HashSet<NodeLifecycleState>,
) -> NodeLifecycleState {
    match primary {
        NodeLifecycleState::Closing => NodeLifecycleState::Closing,
        NodeLifecycleState::Maintenance => NodeLifecycleState::Maintenance,
        NodeLifecycleState::Initialising => NodeLifecycleState::Initialising,
        _ => {
            // Pick highest-severity degraded condition, if any.
            if let Some(worst) = degraded.iter().max_by_key(|s| s.severity()) {
                worst.clone()
            } else {
                NodeLifecycleState::Ready
            }
        }
    }
}

/// Build a `NodeStateEvent` for the given primary state and active degraded conditions.
pub fn make_node_state_event(
    primary: NodeLifecycleState,
    degraded: &HashSet<NodeLifecycleState>,
) -> NodeStateEvent {
    let display = active_display_state(&primary, degraded);
    let label = display.to_string();
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    let mut active_degraded: Vec<String> = degraded
        .iter()
        .map(|s| s.as_canonical().to_string())
        .collect();
    active_degraded.sort();

    NodeStateEvent {
        state: display,
        label,
        active_degraded,
        timestamp,
    }
}
