// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! AC-D3d control-surface error catalogue + the substrate [`ControlError`].
//!
//! The v1 catalogue is **closed** at nine codes (§8). Each maps onto exactly
//! one [`Category`]. **Invariant:** no control code uses `Category::Protocol`
//! — `protocol` is reserved for verb-sourced errors (band codes / client
//! `anyhow`). There is no `INTERNAL` code in v1.

use super::envelope::{Category, ErrorBody};

/// A control-surface error code (AC-D3d, §8). Distinct from the numeric
/// protocol error domain — these are uppercase-snake strings on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCode {
    /// The instance has not finished SETUP / cannot accept this command yet.
    InstanceNotReady,
    /// The `cmd` verb is not recognised on this binary.
    UnknownCommand,
    /// Required argument missing, type mismatch, or value out of range.
    BadArgument,
    /// A `$<name>` substitution referenced an unknown binding (§5).
    BindingNotFound,
    /// A second command was issued while a first was still in-flight (§2.3).
    ConcurrentCommandNotAllowed,
    /// The persistent home-Node connection / a required resource is gone.
    ConnectionLost,
    /// The command did not complete within its per-command window (§10).
    Timeout,
    /// The session lacks authority. Reserved/unused in v1 (AC-D4 trio).
    PermissionDenied,
    /// The line is not valid JSON or has no `cmd` (reply omits echoed `cmd`).
    MalformedCommand,
}

impl ControlCode {
    /// Every control code — for exhaustive catalogue / invariant tests.
    pub const ALL: [ControlCode; 9] = [
        ControlCode::InstanceNotReady,
        ControlCode::UnknownCommand,
        ControlCode::BadArgument,
        ControlCode::BindingNotFound,
        ControlCode::ConcurrentCommandNotAllowed,
        ControlCode::ConnectionLost,
        ControlCode::Timeout,
        ControlCode::PermissionDenied,
        ControlCode::MalformedCommand,
    ];

    /// The wire string (uppercase-snake).
    pub fn code(self) -> &'static str {
        match self {
            ControlCode::InstanceNotReady => "INSTANCE_NOT_READY",
            ControlCode::UnknownCommand => "UNKNOWN_COMMAND",
            ControlCode::BadArgument => "BAD_ARGUMENT",
            ControlCode::BindingNotFound => "BINDING_NOT_FOUND",
            ControlCode::ConcurrentCommandNotAllowed => "CONCURRENT_COMMAND_NOT_ALLOWED",
            ControlCode::ConnectionLost => "CONNECTION_LOST",
            ControlCode::Timeout => "TIMEOUT",
            ControlCode::PermissionDenied => "PERMISSION_DENIED",
            ControlCode::MalformedCommand => "MALFORMED_COMMAND",
        }
    }

    /// The fixed category for this code. Never `Category::Protocol` (the
    /// AC-D3d invariant — verified exhaustively in tests).
    pub fn category(self) -> Category {
        match self {
            ControlCode::InstanceNotReady => Category::Lifecycle,
            ControlCode::UnknownCommand
            | ControlCode::BadArgument
            | ControlCode::BindingNotFound
            | ControlCode::ConcurrentCommandNotAllowed
            | ControlCode::MalformedCommand => Category::Argument,
            ControlCode::ConnectionLost => Category::Connection,
            ControlCode::Timeout => Category::Timeout,
            ControlCode::PermissionDenied => Category::Permission,
        }
    }
}

/// A control-surface failure raised by the pure substrate.
///
/// Carries everything the wire [`ErrorBody`] needs **except** `instance_state`
/// (runtime-specific) — the binary's dispatch arm stamps that via
/// [`ControlError::into_body`]. Control errors never carry a `stage` (verb-only).
#[derive(Debug, Clone)]
pub struct ControlError {
    pub code: ControlCode,
    pub message: String,
    pub hint: Option<String>,
}

impl ControlError {
    /// A control error with no hint.
    pub fn new(code: ControlCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: None,
        }
    }

    /// Attach a suggested-next-command hint.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Compose the wire [`ErrorBody`], stamping the instance lifecycle state.
    pub fn into_body(self, instance_state: impl Into<String>) -> ErrorBody {
        ErrorBody {
            code: self.code.code().to_string(),
            category: self.code.category(),
            message: self.message,
            instance_state: instance_state.into(),
            stage: None,
            hint: self.hint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_has_nine_codes() {
        assert_eq!(ControlCode::ALL.len(), 9);
    }

    #[test]
    fn no_control_code_uses_category_protocol() {
        // AC-D3d invariant: protocol is verb-sourced only.
        for code in ControlCode::ALL {
            assert_ne!(
                code.category(),
                Category::Protocol,
                "{} must not be category protocol",
                code.code()
            );
        }
    }

    #[test]
    fn codes_are_uppercase_snake() {
        for code in ControlCode::ALL {
            let s = code.code();
            assert!(
                s.chars().all(|c| c.is_ascii_uppercase() || c == '_'),
                "{s} is not uppercase-snake"
            );
        }
    }

    #[test]
    fn into_body_stamps_state_and_omits_stage() {
        let body = ControlError::new(ControlCode::BadArgument, "x must be > 0")
            .with_hint("set x to a positive integer")
            .into_body("ready");
        assert_eq!(body.code, "BAD_ARGUMENT");
        assert_eq!(body.category, Category::Argument);
        assert_eq!(body.instance_state, "ready");
        assert_eq!(body.hint.as_deref(), Some("set x to a positive integer"));
        assert!(body.stage.is_none(), "control errors never carry a stage");
    }
}
