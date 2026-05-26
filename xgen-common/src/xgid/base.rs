// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! [`Xgid`] — base XGID newtype (XGID Adoption v1, D-072, Appendix J §J.2).

use std::borrow::Borrow;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::XgidLike;

/// XGen protocol identifier — wire-invariant URI bytes.
///
/// Construction at v1 is unvalidated: [`Xgid::new`] accepts any string. The
/// URI grammar (Appendix J §J.3) is enforced at construction sites that have
/// the right context — hash-anchored flavours hash canonical bytes through
/// `from_canonical_bytes`; principal flavours format pubkey bytes through
/// `from_pubkey`. A future tightening may parse-on-construction; that
/// tightening is **not** in v1 scope (Appendix J §J.8).
///
/// Two [`Xgid`] values are equal iff their inner bytes are byte-equal
/// (Appendix J §J.5 invariance 5). There is no normalisation, no case-
/// folding, no whitespace tolerance.
///
/// Serialised wire form is a plain JSON string (Appendix J §J.5 invariance
/// 2) via `#[serde(transparent)]`.
#[derive(
    Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Xgid(String);

impl Xgid {
    /// Construct an [`Xgid`] from any string. No URI-grammar validation at v1.
    pub fn new(s: String) -> Self {
        Self(s)
    }

    /// Read-only access to the inner URI string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Xgid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl XgidLike for Xgid {
    fn as_xgid(&self) -> &Xgid {
        self
    }
}

/// `Borrow<str>` enables `HashMap<Xgid, V>::get(&str)` and sibling Borrow-driven
/// lookups (HashSet::contains, etc.) without allocating a wrapper Xgid for each
/// query. Added at Pass 1 Commit 4 to keep cross-crate consumer code paths idiomatic
/// once the protocol-surface data structures keyed by XGID flavours land. Does NOT
/// promote `Xgid` to act as a `&str` in arbitrary expressions — `Deref<Target = str>`
/// remains deferred per Appendix J §J.8 + sub-question 3 of the Pass 1 runbook.
impl Borrow<str> for Xgid {
    fn borrow(&self) -> &str {
        &self.0
    }
}
