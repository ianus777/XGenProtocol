// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

pub mod crypto;
pub mod wire;
pub mod dag;
pub mod blob_store;
pub mod transport;
pub mod node;
pub mod federation;
pub mod identity;
pub mod space;
pub mod message;
pub mod resolution;
pub mod migration;
pub mod bootstrap;
pub mod encryption;
pub mod auth;

// MP-F14: the federation/vantage-infrastructure event-kind set, surfaced at the
// crate root so cross-crate consumers (the `xgen-mptest` convergence oracle) reach
// it as `xgen_core::INFRA_EVENT_KINDS`. Defined next to `EventType` in xgen-common
// (Rust coherence forces the inherent `EventType::is_federation_infra` predicate
// there); this is the authoritative re-export surface.
pub use wire::types::INFRA_EVENT_KINDS;
