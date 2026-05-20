// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! XGID — typed protocol identifier vocabulary (XGID Adoption v1, D-072).
//!
//! This module ships the v1 type vocabulary committed to in `DECISIONS.md`
//! D-072 and expounded in `docs/xgen_appendix_j_en.md`. The base [`Xgid`]
//! newtype carries the wire-invariant URI bytes; six flavour wrappers
//! ([`EventXgid`], [`SpaceXgid`], [`RoomXgid`], [`TrustAssertionXgid`],
//! [`NodeXgid`], [`IdentityXgid`]) encode flavour in the type system without
//! adding any flavour tag to the wire.
//!
//! All XGID types are `#[serde(transparent)]` over their inner string. Two
//! XGIDs compare equal iff their inner bytes are byte-equal — there is no
//! normalisation step (Appendix J §J.5 invariance 5).
//!
//! The [`XgidLike`] trait gives flavour-agnostic access to the inner [`Xgid`]
//! at the rare use sites that genuinely operate over "any XGID" (trace
//! logging is the canonical example). Reach for [`XgidLike`] only when no
//! specific flavour would be honest at the use site — D-073 (field-name-vs-
//! type discipline) instructs us to prefer specific flavours wherever the
//! role at the use site is known.
//!
//! ## Scope at v1 (what this module DOES NOT ship)
//!
//! - **URI-grammar validation on construction.** [`Xgid::new`] accepts any
//!   string. Validation happens through the flavour-specific decode methods
//!   (`pubkey()` for principal flavours). Construction-time validation needs
//!   its own design walkthrough (e.g. fail-fast vs construct-then-fail-at-use)
//!   and is deferred past v1.
//! - **Cross-flavour conversion.** Not provided (Appendix J §J.6). Code that
//!   needs to construct, say, an [`EventXgid`] from a [`NodeXgid`]'s
//!   underlying string must do so explicitly through [`Xgid`] extraction.
//!   Intentional friction.
//! - **Normalisation hooks, case-folding, whitespace tolerance.** Strict
//!   byte-equality only.
//! - **Flavour-tagging on the wire.** Wire stays plain string. Flavour lives
//!   in the type system and in surrounding field names (D-073), not on the
//!   wire.

mod base;
mod error;
mod flavours;

pub use base::Xgid;
pub use error::XgidDecodeError;
pub use flavours::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, TrustAssertionXgid};

/// Flavour-agnostic accessor for the underlying [`Xgid`].
///
/// Implemented by [`Xgid`] itself and by all six flavour wrappers. Use only
/// when no specific flavour would be honest at the use site (trace logging
/// is the canonical example). D-073 (field-name-vs-type discipline) directs
/// us to use specific flavour types — `XgidLike` exists for the residual
/// "really, any XGID" use sites.
pub trait XgidLike {
    /// Returns a borrow of the underlying [`Xgid`].
    fn as_xgid(&self) -> &Xgid;

    /// Returns the underlying URI string. Convenience delegating to
    /// [`Xgid::as_str`] via [`as_xgid`](XgidLike::as_xgid).
    fn as_str(&self) -> &str {
        self.as_xgid().as_str()
    }
}
