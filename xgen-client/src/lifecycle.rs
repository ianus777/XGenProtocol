// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

use std::fmt;

use chrono::{SecondsFormat, Utc};

/// Client lifecycle states — Appendix E §E.2.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientLifecycleState {
    Setup,
    Initialising,
    Connecting,
    Authenticating,
    Ready,
    DegradedAuth,
    DegradedFederation,
    DegradedNode,
    Reconnecting,
    Disconnected,
    Closing,
}

impl ClientLifecycleState {
    /// Canonical uppercase-underscore form used in log lines and JSON payloads.
    pub fn as_canonical(&self) -> &'static str {
        match self {
            ClientLifecycleState::Setup => "SETUP",
            ClientLifecycleState::Initialising => "INITIALISING",
            ClientLifecycleState::Connecting => "CONNECTING",
            ClientLifecycleState::Authenticating => "AUTHENTICATING",
            ClientLifecycleState::Ready => "READY",
            ClientLifecycleState::DegradedAuth => "DEGRADED_AUTH",
            ClientLifecycleState::DegradedFederation => "DEGRADED_FEDERATION",
            ClientLifecycleState::DegradedNode => "DEGRADED_NODE",
            ClientLifecycleState::Reconnecting => "RECONNECTING",
            ClientLifecycleState::Disconnected => "DISCONNECTED",
            ClientLifecycleState::Closing => "CLOSING",
        }
    }
}

/// Display returns the Appendix E title-case display label (shown in the UI).
impl fmt::Display for ClientLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ClientLifecycleState::Setup => "Setting up",
            ClientLifecycleState::Initialising => "Initialising",
            ClientLifecycleState::Connecting => "Connecting",
            ClientLifecycleState::Authenticating => "Authenticating",
            ClientLifecycleState::Ready => "Ready",
            ClientLifecycleState::DegradedAuth => "Auth degraded",
            ClientLifecycleState::DegradedFederation => "Federation degraded",
            ClientLifecycleState::DegradedNode => "Node degraded",
            ClientLifecycleState::Reconnecting => "Reconnecting",
            ClientLifecycleState::Disconnected => "Disconnected",
            ClientLifecycleState::Closing => "Closing",
        };
        write!(f, "{}", label)
    }
}

/// Payload emitted on the `"xgen-client-state-changed"` Tauri event (D-042).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClientStateEvent {
    pub state: ClientLifecycleState,
    pub label: String,
    pub timestamp: String,
}

/// Build a `ClientStateEvent` for the given state, timestamped now.
pub fn make_state_event(state: ClientLifecycleState) -> ClientStateEvent {
    let label = state.to_string();
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    ClientStateEvent { state, label, timestamp }
}
