// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Module / plugin framework spine — descriptor and identity vocabulary
//! (Storage-Engine / Plugin-Framework milestone, SE-D2).
//!
//! These are the host-neutral types every module/plugin slot shares. They live
//! in `xgen-common` so a later *client* slot (`kind = plugin`, `host = client`)
//! inherits the exact same vocabulary as the first *node* slot (`kind = module`,
//! `host = node`) for free. There is one unified handshake mechanism; the code
//! term **`kind`** carries the system/ui distinction:
//!
//! - a **module** is a *system* plugin (`host = node`),
//! - a **plugin** is a *ui* plugin (`host = client`).
//!
//! ## Identity ([`ModuleKindId`], [`ModuleImplId`]) — UUIDv4, never `Xgid`
//!
//! Slot and implementation identity are **UUIDv4**, not [`crate::Xgid`]. An
//! `Xgid` is protocol-assigned and *federates*; a module GUID is **local,
//! developer-assigned, and never crosses the wire**. Conflating them would put
//! a dev-local identifier on the federation surface. The **kind GUID** is
//! generated once per slot and *copied* by every implementer (it names the
//! slot — "the node storage-engine slot"); the **impl GUID** is generated once
//! per implementation by its author.
//!
//! ## Trust posture (SE-D2, §8 security record)
//!
//! The descriptor is a **const in the plugin's own code** — there is no
//! manifest file, because a compile-time framework has no host↔plugin gap to
//! bridge. Metadata is *authoritative*, location is *never trusted*, and an
//! unknown descriptor is **rejected loudly** at the registration site. The
//! GUID/descriptor handshake verifies *declaration honesty*, never *behaviour*.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identity of a module/plugin **slot** — e.g. "the node storage-engine slot".
///
/// Generated once per slot and copied verbatim by every implementer, so the
/// host can recognise "this crate claims to fill *that* slot". UUIDv4, never an
/// [`crate::Xgid`] (local, dev-assigned, never federates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModuleKindId(pub Uuid);

impl ModuleKindId {
    /// Const constructor for declaring a slot GUID inline:
    /// `const KIND: ModuleKindId = ModuleKindId::from_u128(0x…);`.
    pub const fn from_u128(value: u128) -> Self {
        Self(Uuid::from_u128(value))
    }

    /// The underlying UUID.
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl fmt::Display for ModuleKindId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identity of a single module/plugin **implementation** filling a slot.
///
/// Generated once per implementation by its author. UUIDv4, never an
/// [`crate::Xgid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModuleImplId(pub Uuid);

impl ModuleImplId {
    /// Const constructor for declaring an implementation GUID inline.
    pub const fn from_u128(value: u128) -> Self {
        Self(Uuid::from_u128(value))
    }

    /// Generate a fresh implementation GUID. Intended for the *one-off*
    /// developer step of minting a new impl identity (the result is then pasted
    /// as a `from_u128` const), not for per-run use.
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }

    /// The underlying UUID.
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl fmt::Display for ModuleImplId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The durability class a storage engine advertises (SE-D4).
///
/// An ordered ladder, kept deliberately small in v1 — extend when a second
/// engine lands. `BestEffort < Durable`, so [`Ord`] reads as "more assurance":
///
/// - [`AssuranceClass::BestEffort`] — don't-corrupt / don't-silently-lose, but
///   *not* crash-proof. The vanilla in-memory + atomic-file floor (D-084). Serves
///   tier 1 only.
/// - [`AssuranceClass::Durable`] — fsync-on-commit, crash-survivable. Required
///   at tiers 2–4 (§L.8 conformance).
///
/// The tier→engine gate (SE-D4) compares this to the node's asserted tier and
/// **refuses to start** if the selected engine under-delivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssuranceClass {
    /// Don't-corrupt / don't-silently-lose, not crash-proof. Vanilla floor.
    BestEffort,
    /// fsync-on-commit, crash-survivable.
    Durable,
}

impl AssuranceClass {
    /// The highest node tier this class is permitted to serve.
    ///
    /// v1 ladder: `BestEffort` tops out at tier 1; `Durable` serves the full
    /// 1–4 range. Extend (insert intermediate classes / raise ceilings) when a
    /// second engine lands.
    pub const fn max_tier(self) -> u8 {
        match self {
            AssuranceClass::BestEffort => 1,
            AssuranceClass::Durable => 4,
        }
    }

    /// Whether this class satisfies a node asserting `tier`. The gate (SE-D4)
    /// rejects start-up when this returns `false`.
    pub const fn satisfies_tier(self, tier: u8) -> bool {
        tier <= self.max_tier()
    }

    /// Operator-facing label (matches the serde snake_case form). Used by the
    /// SE-D8 capability advert.
    pub const fn label(self) -> &'static str {
        match self {
            AssuranceClass::BestEffort => "best_effort",
            AssuranceClass::Durable => "durable",
        }
    }
}

/// A module/plugin's self-description — a **const in the implementation's own
/// code** (SE-D2). No manifest, no file: the metadata is authoritative, the
/// location is never trusted.
///
/// `name` is a `&'static str` because the descriptor is compiled in. The type
/// is [`Copy`] and const-constructible so an implementation can write:
///
/// ```ignore
/// const DESC: Descriptor = Descriptor {
///     kind_id: SLOT_KIND,                         // copied from the slot
///     impl_id: ModuleImplId::from_u128(0x…),      // this impl's own GUID
///     name: "xgen-store-sqlite",
///     assurance: AssuranceClass::Durable,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Descriptor {
    /// The slot this implementation claims to fill (copied from the slot's GUID).
    pub kind_id: ModuleKindId,
    /// This implementation's own GUID.
    pub impl_id: ModuleImplId,
    /// Human-readable implementation name (compiled in).
    pub name: &'static str,
    /// The durability class this implementation advertises.
    pub assurance: AssuranceClass,
}

#[cfg(test)]
mod tests {
    use super::*;

    const KIND_A: ModuleKindId = ModuleKindId::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0001);
    const KIND_B: ModuleKindId = ModuleKindId::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0002);

    #[test]
    fn kind_id_const_and_roundtrip() {
        // Const construction works (declared above).
        assert_eq!(KIND_A, ModuleKindId::from_u128(1));
        // serde round-trip is transparent (plain UUID).
        let json = serde_json::to_string(&KIND_A).unwrap();
        let back: ModuleKindId = serde_json::from_str(&json).unwrap();
        assert_eq!(KIND_A, back);
    }

    #[test]
    fn kind_ids_differ_across_kinds() {
        assert_ne!(KIND_A, KIND_B);
        // Display delegates to the inner UUID.
        assert_eq!(KIND_A.to_string(), KIND_A.as_uuid().to_string());
    }

    #[test]
    fn impl_id_new_v4_is_unique() {
        let a = ModuleImplId::new_v4();
        let b = ModuleImplId::new_v4();
        assert_ne!(a, b);
        let json = serde_json::to_string(&a).unwrap();
        let back: ModuleImplId = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn assurance_class_orders_best_effort_below_durable() {
        assert!(AssuranceClass::BestEffort < AssuranceClass::Durable);
    }

    #[test]
    fn assurance_satisfies_tier_table() {
        // BestEffort (vanilla floor) serves tier 1 only.
        assert!(AssuranceClass::BestEffort.satisfies_tier(1));
        assert!(!AssuranceClass::BestEffort.satisfies_tier(2));
        assert!(!AssuranceClass::BestEffort.satisfies_tier(4));
        // Durable serves the full 1–4 range.
        assert!(AssuranceClass::Durable.satisfies_tier(1));
        assert!(AssuranceClass::Durable.satisfies_tier(2));
        assert!(AssuranceClass::Durable.satisfies_tier(4));
    }

    #[test]
    fn descriptor_is_const_constructible() {
        const DESC: Descriptor = Descriptor {
            kind_id: KIND_A,
            impl_id: ModuleImplId::from_u128(0x42),
            name: "test-engine",
            assurance: AssuranceClass::Durable,
        };
        assert_eq!(DESC.name, "test-engine");
        assert_eq!(DESC.kind_id, KIND_A);
        assert_eq!(DESC.assurance, AssuranceClass::Durable);
        // Serializes (Deserialize intentionally omitted — `name: &'static str`).
        assert!(serde_json::to_string(&DESC).unwrap().contains("test-engine"));
    }
}
