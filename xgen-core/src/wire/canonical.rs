// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! Canonical-form helpers re-exported from `xgen_common::canonical`.
//!
//! XGID Retrofit Pass 1 Commit 1 moved the canonical-form helpers
//! (`canonical_event_bytes`, `canonical_event_json`, `canonical_object_json`,
//! `canonical_value`) from this file into `xgen-common/src/canonical.rs` so
//! the v1 flavour-wrapper convenience constructors (`EventXgid::from_event`,
//! `SpaceXgid::from_space_create`, `RoomXgid::from_room_create`,
//! `TrustAssertionXgid::from_assertion`) implemented at Commit 2 can call
//! them from inside `xgen-common`.
//!
//! This module is now a thin re-export shim so existing xgen-core call sites
//! (`crate::wire::canonical::canonical_event_bytes`, etc.) continue to compile
//! unchanged. The shim is scheduled for removal in a future cleanup pass
//! (Pass 2 or Pass 5) when downstream call sites migrate to importing directly
//! from `xgen_common::canonical`.

pub use xgen_common::canonical::{
    canonical_event_bytes, canonical_event_json, canonical_object_json, canonical_value,
};
