// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Process-unique connection identifier (EV-D1, M7-events arc).
//!
//! A [`ConnId`] identifies a single transport connection for the lifetime of
//! the process. It is minted from one global `AtomicU64`, carries **no
//! connection-kind tag** (the fan-out registry stays kind-agnostic per EV-D2),
//! and is `Copy` + sortable + log-friendly. A `u64` counter does not
//! realistically wrap, so uniqueness holds for the process lifetime.
//!
//! The gating need is the Node's multi-connection-per-identity fan-out
//! (`ClientSenders` retyped to `Vec<(ConnId, Sender)>`): the same Identity may
//! hold a primary client WS and a second `.events` WS, and the registry must
//! tell them apart to remove the right one on disconnect. The id identifies a
//! connection, nothing more — type-carries-contract (D-073).

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(0);

/// A process-unique connection identifier. See the module docs.
///
/// The inner `u64` is public so tests can construct literal ids; production
/// code mints sequential ids via [`ConnId::mint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConnId(pub u64);

impl ConnId {
    /// Mint the next process-unique id from the global atomic counter.
    /// Called at each connection registration entry point.
    pub fn mint() -> Self {
        ConnId(NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// The raw counter value.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ConnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mint_is_monotonic_and_unique() {
        let a = ConnId::mint();
        let b = ConnId::mint();
        let c = ConnId::mint();
        assert_ne!(a, b);
        assert_ne!(b, c);
        // Monotonic by construction (fetch_add) — later mints compare greater.
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn literal_construction_and_display() {
        let id = ConnId(42);
        assert_eq!(id.as_u64(), 42);
        assert_eq!(id.to_string(), "42");
        assert_eq!(ConnId(7), ConnId(7));
    }
}
