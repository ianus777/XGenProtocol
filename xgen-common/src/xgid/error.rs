// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Error types for XGID flavour decode methods (XGID Adoption v1).

use thiserror::Error;

/// Failure modes for parse-fallible XGID decode methods —
/// [`crate::xgid::NodeXgid::pubkey`] and [`crate::xgid::IdentityXgid::pubkey`].
///
/// The base [`crate::xgid::Xgid`] type accepts any string at v1; principal
/// flavours therefore cannot promise more than what the construction-source
/// data supports. These variants capture the distinct ways a principal-
/// flavour XGID's inner string may fail to round-trip back to an Ed25519
/// public key.
#[derive(Debug, Error)]
pub enum XgidDecodeError {
    /// The inner string does not start with the expected `xgen://pubkey/ed25519:`
    /// principal-flavour prefix.
    #[error("XGID does not have the expected principal-flavour prefix {expected:?}")]
    InvalidPrefix {
        /// The prefix the decode method was looking for.
        expected: &'static str,
    },

    /// The pubkey segment after the prefix is not a valid base64url-no-pad
    /// encoding (matches `xgen-core::crypto::encoding::decode` rejection
    /// rules — no `+`, `/`, or `=`).
    #[error("XGID pubkey segment is not valid base64url-no-pad: {0}")]
    InvalidBase64(String),

    /// The decoded pubkey bytes are not the expected length for an Ed25519
    /// verifying key (32 bytes).
    #[error("XGID decoded pubkey has wrong length: expected {expected} bytes, got {got}")]
    InvalidKeyLength {
        /// Expected byte length (32 for Ed25519).
        expected: usize,
        /// Actual byte length recovered from the base64url segment.
        got: usize,
    },

    /// The decoded bytes are the right length but `ed25519_dalek` rejected
    /// them as not a valid point on the curve.
    #[error("XGID decoded pubkey is not a valid Ed25519 point: {0}")]
    InvalidPoint(String),
}
