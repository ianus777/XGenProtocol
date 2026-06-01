// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! AC-D4 control-token gate (M7C-D1, B1).
//!
//! Pure, transport-free check shared by both `.aicontrol` command pipes
//! (client today; node when its surface gates later — single source of truth,
//! D-067). The token authenticates the AI driver to the resident's command
//! pipe (plane 1). It is an **opaque** value: this layer only compares it, it
//! never parses it into a structured type — keeping end-state B's driver-bound
//! credential free to ride the same field with no wire change.

use super::codes::{ControlCode, ControlError};

/// Decide whether a command bearing `presented` may proceed, given the
/// resident's `expected` token.
///
/// - `expected = None` → the seam is **inert** (no token configured). Always
///   proceeds. This is the v1 production default (the reserved trio is inert
///   pending a privilege model — M7C-D1).
/// - `expected = Some(_)` with `presented = None` → **`absent==proceed`**. v1
///   cannot *require* a token (requiring presence belongs to end-state B's
///   privilege model); an omitted token proceeds.
/// - `expected = Some(e)`, `presented = Some(p)`, `p == e` → proceeds (valid).
/// - `expected = Some(_)`, `presented = Some(_)`, mismatch → `PERMISSION_DENIED`
///   (`Category::Permission`; AC-D4's reserved code, activated for this surface).
///
/// **Cadence is per-command** (B1 decision, J-220): the caller invokes this for
/// every command against the resident's `expected`, so a connection cannot
/// authenticate once and then send a non-matching token on a later command.
/// There is no cached per-connection "authed" flag.
///
/// (v1 uses `==`; a constant-time comparison is a hardening concern for the
/// privilege-model arc when `expected` becomes a real secret/credential.)
pub fn check_token(presented: Option<&str>, expected: Option<&str>) -> Result<(), ControlError> {
    match (expected, presented) {
        (None, _) => Ok(()),
        (Some(_), None) => Ok(()),
        (Some(e), Some(p)) if p == e => Ok(()),
        (Some(_), Some(_)) => Err(ControlError::new(
            ControlCode::PermissionDenied,
            "control token rejected",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aicontrol::Category;

    #[test]
    fn inert_when_no_expected_token() {
        // v1 production default: nothing configured → everything proceeds.
        assert!(check_token(None, None).is_ok());
        assert!(check_token(Some("anything"), None).is_ok());
    }

    #[test]
    fn absent_presented_proceeds_even_when_expected() {
        // absent==proceed: v1 cannot require a token.
        assert!(check_token(None, Some("expected")).is_ok());
    }

    #[test]
    fn matching_token_proceeds() {
        assert!(check_token(Some("s3cr3t"), Some("s3cr3t")).is_ok());
    }

    #[test]
    fn mismatched_token_is_permission_denied() {
        let err = check_token(Some("wrong"), Some("expected")).unwrap_err();
        let body = err.into_body("ready".to_string());
        assert_eq!(body.code, "PERMISSION_DENIED");
        assert!(matches!(body.category, Category::Permission));
    }
}
