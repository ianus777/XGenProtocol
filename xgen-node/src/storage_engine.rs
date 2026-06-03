// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Storage-engine registry + tier gate (Storage-Engine milestone, SE-D3 / SE-D4).
//!
//! ## Registry (SE-D3)
//!
//! [`EngineTable`] is the **feature-assembled** registry: each engine is added by
//! an **explicit** [`register`] call at the single [`build_engine_table`]
//! assembly site, gated by its own `#[cfg(feature = "…")]`. Link-time
//! auto-collection (`inventory` / `linkme`) is deliberately rejected — it is less
//! auditable and runs against the "built from readable source" trust model
//! (audit §8). Selection is by name; a named-but-unregistered engine is a **loud
//! reject**, never a silent fall back to vanilla (which would lie about
//! durability). The vanilla [`crate::dag::store::InMemoryEventStore`] backend is
//! **not** in the table — it is the implicit default when nothing is selected.
//!
//! ## Tier gate (SE-D4)
//!
//! At startup the Node asserts a storage-durability tier and the gate refuses to
//! start if the selected engine under-delivers it. The asserted tier is
//! `[node].asserts_tier` if set, else the **floor** = max over
//! (`bootstrap.auth_tiers_served` ∪ all module `accepted_tiers`). A present
//! `asserts_tier` may only raise the tier above the floor; below it is a loud
//! reject (a Node cannot under-declare the durability it actually serves). The
//! gate then checks the selected engine's [`AssuranceClass`] against the asserted
//! tier. A plain Node with no tier signal floors at T1, which the vanilla
//! `BestEffort` backend satisfies — today's behaviour, byte-for-byte.

use std::collections::HashMap;

use std::path::PathBuf;

use thiserror::Error;
use xgen_common::module::{AssuranceClass, Descriptor};
use xgen_common::xgid::SpaceXgid;

use crate::dag::store::{
    EngineError, EngineSettings, EventStore, StorageEngine, StoreFactory, StoreInitError,
    STORAGE_ENGINE_KIND_ID, VANILLA_DESCRIPTOR,
};

/// A boxing factory: constructs a selected engine from its settings and
/// type-erases it to the store seam the owners hold
/// (`Box<dyn EventStore + Send + Sync>`, SE-D6). A plain `fn` pointer — each
/// [`register`] site is stateless (no captured environment), so this is the lean
/// `EngineTable` value type (confirm-at-pickup §5 #3, resolved here).
pub type EngineFactory =
    fn(&EngineSettings) -> Result<Box<dyn EventStore + Send + Sync>, EngineError>;

/// One registered engine: its self-description plus a factory closing over its
/// `open`.
#[derive(Clone, Copy)]
pub struct EngineEntry {
    pub descriptor: Descriptor,
    pub factory: EngineFactory,
}

/// The feature-assembled engine registry (SE-D3), keyed by `Descriptor::name`.
#[derive(Default)]
pub struct EngineTable {
    engines: HashMap<&'static str, EngineEntry>,
}

impl EngineTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a registered engine by name.
    pub fn get(&self, name: &str) -> Option<&EngineEntry> {
        self.engines.get(name)
    }

    pub fn len(&self) -> usize {
        self.engines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.engines.is_empty()
    }
}

/// Register an engine into the table by its descriptor name (SE-D3). Explicit,
/// per-feature; the `Send + Sync` bounds let the factory hand back the
/// `Box<dyn EventStore + Send + Sync>` the owners hold (SE-D6).
pub fn register<E: StorageEngine + Send + Sync + 'static>(table: &mut EngineTable) {
    let descriptor = E::descriptor();
    // Ch4 §4.12.5 — the slot match is on the parsed 128-bit value
    // (`ModuleKindId`'s derived `PartialEq` compares the inner `Uuid`, never the
    // textual form). An engine declaring a foreign `kind_id` (e.g. a wrong/stale
    // copied slot GUID) is not registered — the handshake verifies the *declared
    // slot*, loudly. Value-not-string comparison is what makes a future
    // canonical-format revision unable to orphan an existing engine's id.
    if descriptor.kind_id != STORAGE_ENGINE_KIND_ID {
        tracing::error!(
            engine = descriptor.name,
            declared_kind = %descriptor.kind_id,
            expected_kind = %STORAGE_ENGINE_KIND_ID,
            "storage engine declares a foreign module-kind; not registering"
        );
        return;
    }
    let factory: EngineFactory = |settings| Ok(Box::new(E::open(settings)?));
    table
        .engines
        .insert(descriptor.name, EngineEntry { descriptor, factory });
}

/// Build the process engine table — the single SE-D3 assembly site. Each engine
/// registers under its own feature; with none compiled in the table is empty and
/// only the vanilla default backend is selectable.
pub fn build_engine_table() -> EngineTable {
    #[allow(unused_mut)]
    let mut table = EngineTable::new();
    #[cfg(feature = "store-sqlite")]
    register::<xgen_store_sqlite::SqliteEngine>(&mut table);
    table
}

/// SE-SUB-D2/D5 — build the per-Space [`StoreFactory`] for an active engine. The
/// host owns the dir→per-Space-path templating: for each Space it encodes
/// `<dir>/<stem>.db` (via the single `space_file_stem` encoder, D-067) and hands
/// the engine a fully-resolved `EngineSettings { path }` (SE-D5 intact — the
/// engine never sees `dir`). Production (`run_node`) and the substitution tests
/// share this one builder so the factory logic has no second copy.
pub fn build_engine_store_factory(engine: EngineFactory, dir: PathBuf) -> StoreFactory {
    Box::new(move |space_id: &SpaceXgid| {
        let stem = crate::app::space_file_stem(space_id.as_str());
        let path = dir.join(format!("{stem}.db"));
        let mut table = toml::map::Map::new();
        table.insert(
            "path".into(),
            toml::Value::String(path.to_string_lossy().to_string()),
        );
        let settings = EngineSettings::new(toml::Value::Table(table));
        engine(&settings).map_err(|e| StoreInitError::EngineOpen(e.to_string()))
    })
}

/// Start-time failures of the tier→engine gate (SE-D4). Each is a loud
/// refuse-to-start — the Node never silently downgrades durability.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StorageGateError {
    #[error(
        "[node].asserts_tier = {asserts} is below the derived floor {floor} \
         (a Node cannot assert less durability than it serves)"
    )]
    UnderDeclaredTier { asserts: u8, floor: u8 },
    #[error(
        "storage engine '{0}' is not registered (its feature may not be compiled \
         in); refusing to start rather than silently fall back to vanilla"
    )]
    UnknownEngine(String),
    #[error(
        "selected storage engine '{engine}' (assurance {assurance:?}) cannot serve \
         the asserted tier {asserts}; install a durable engine module or lower \
         [node].asserts_tier"
    )]
    EngineUnderDelivers {
        engine: String,
        assurance: AssuranceClass,
        asserts: u8,
    },
}

/// The outcome of a passing gate: which backend is active and at what tier.
/// Feeds the SE-D8 capability advert (C5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageSelection {
    pub descriptor: Descriptor,
    pub asserts_tier: u8,
}

/// Derive the tier floor over the union of served tiers and module accepted
/// tiers (SE-D4) — the maximum tier the Node already serves. No signal → T1.
pub fn derive_tier_floor(served: &[u8], module_tiers: &[u8]) -> u8 {
    served
        .iter()
        .chain(module_tiers)
        .copied()
        .max()
        .unwrap_or(1)
        .max(1)
}

/// Resolve the asserted tier from config + floor (SE-D4 clamp). `None` → the
/// floor; a present value may only raise the tier (`< floor` is a loud reject).
pub fn resolve_asserts_tier(
    config_asserts: Option<u8>,
    floor: u8,
) -> Result<u8, StorageGateError> {
    match config_asserts {
        None => Ok(floor),
        Some(a) if a < floor => Err(StorageGateError::UnderDeclaredTier { asserts: a, floor }),
        Some(a) => Ok(a),
    }
}

/// Select the active engine descriptor (SE-D3). `None` → vanilla; a named engine
/// must be registered (reject-unknown-loud, never a silent vanilla fallback).
pub fn select_descriptor(
    table: &EngineTable,
    storage_engine: Option<&str>,
) -> Result<Descriptor, StorageGateError> {
    match storage_engine {
        None => Ok(VANILLA_DESCRIPTOR),
        Some(name) => table
            .get(name)
            .map(|e| e.descriptor)
            .ok_or_else(|| StorageGateError::UnknownEngine(name.to_string())),
    }
}

/// Run the full SE-D4 tier→engine gate at startup. Returns the [`StorageSelection`]
/// on success; any failure is a loud refuse-to-start.
pub fn evaluate_storage_gate(
    table: &EngineTable,
    storage_engine: Option<&str>,
    config_asserts: Option<u8>,
    served: &[u8],
    module_tiers: &[u8],
) -> Result<StorageSelection, StorageGateError> {
    let floor = derive_tier_floor(served, module_tiers);
    let asserts_tier = resolve_asserts_tier(config_asserts, floor)?;
    let descriptor = select_descriptor(table, storage_engine)?;
    if !descriptor.assurance.satisfies_tier(asserts_tier) {
        return Err(StorageGateError::EngineUnderDelivers {
            engine: descriptor.name.to_string(),
            assurance: descriptor.assurance,
            asserts: asserts_tier,
        });
    }
    Ok(StorageSelection {
        descriptor,
        asserts_tier,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::store::{InMemoryEventStore, StoreError};
    use xgen_common::module::{ModuleImplId, ModuleKindId};
    use xgen_common::xgid::EventXgid;
    use xgen_core::wire::types::Event;

    // A test engine that advertises `Durable` so it can serve any tier.
    struct DurableTestEngine(InMemoryEventStore);

    impl EventStore for DurableTestEngine {
        fn append(&mut self, event: Event) -> Result<(), StoreError> {
            self.0.append(event)
        }
        fn get(&self, id: &EventXgid) -> Result<Option<Event>, StoreError> {
            EventStore::get(&self.0, id)
        }
        fn range(&self, since_seq: u64) -> Result<Vec<Event>, StoreError> {
            self.0.range(since_seq)
        }
        fn contains(&self, id: &EventXgid) -> bool {
            EventStore::contains(&self.0, id)
        }
        fn len(&self) -> usize {
            EventStore::len(&self.0)
        }
    }

    impl StorageEngine for DurableTestEngine {
        fn open(_settings: &EngineSettings) -> Result<Self, EngineError> {
            Ok(Self(InMemoryEventStore::new()))
        }
        fn descriptor() -> Descriptor {
            Descriptor {
                // Copies the slot kind (§4.12.5) so the register handshake admits it.
                kind_id: STORAGE_ENGINE_KIND_ID,
                impl_id: ModuleImplId::from_u128(0x01),
                name: "durable-test",
                assurance: AssuranceClass::Durable,
            }
        }
    }

    // An engine that copied the WRONG slot GUID — the handshake must reject it.
    struct WrongKindEngine(InMemoryEventStore);

    impl EventStore for WrongKindEngine {
        fn append(&mut self, event: Event) -> Result<(), StoreError> {
            self.0.append(event)
        }
        fn get(&self, id: &EventXgid) -> Result<Option<Event>, StoreError> {
            EventStore::get(&self.0, id)
        }
        fn range(&self, since_seq: u64) -> Result<Vec<Event>, StoreError> {
            self.0.range(since_seq)
        }
        fn contains(&self, id: &EventXgid) -> bool {
            EventStore::contains(&self.0, id)
        }
        fn len(&self) -> usize {
            EventStore::len(&self.0)
        }
    }

    impl StorageEngine for WrongKindEngine {
        fn open(_settings: &EngineSettings) -> Result<Self, EngineError> {
            Ok(Self(InMemoryEventStore::new()))
        }
        fn descriptor() -> Descriptor {
            Descriptor {
                kind_id: ModuleKindId::from_u128(0xdead_beef), // foreign slot
                impl_id: ModuleImplId::from_u128(0x02),
                name: "wrong-kind",
                assurance: AssuranceClass::Durable,
            }
        }
    }

    fn table_with_durable() -> EngineTable {
        let mut t = EngineTable::new();
        register::<DurableTestEngine>(&mut t);
        t
    }

    #[test]
    fn floor_is_max_over_served_and_module_tiers_min_one() {
        assert_eq!(derive_tier_floor(&[], &[]), 1); // no signal → T1
        assert_eq!(derive_tier_floor(&[1], &[]), 1);
        assert_eq!(derive_tier_floor(&[2, 1], &[3]), 3); // union max
        assert_eq!(derive_tier_floor(&[], &[2, 4]), 4); // module-only
    }

    #[test]
    fn asserts_tier_clamps_up_but_rejects_below_floor() {
        // None → floor.
        assert_eq!(resolve_asserts_tier(None, 2).unwrap(), 2);
        // Above floor → accepted (clamp up).
        assert_eq!(resolve_asserts_tier(Some(4), 2).unwrap(), 4);
        // Equal to floor → accepted.
        assert_eq!(resolve_asserts_tier(Some(2), 2).unwrap(), 2);
        // Below floor → loud reject (under-declaration).
        assert_eq!(
            resolve_asserts_tier(Some(1), 3),
            Err(StorageGateError::UnderDeclaredTier { asserts: 1, floor: 3 })
        );
    }

    #[test]
    fn default_when_none_selects_vanilla() {
        let table = EngineTable::new();
        let d = select_descriptor(&table, None).unwrap();
        assert_eq!(d.name, "vanilla");
        assert_eq!(d.assurance, AssuranceClass::BestEffort);
    }

    #[test]
    fn unknown_engine_name_is_loud_reject() {
        let table = EngineTable::new(); // empty — sqlite not registered
        assert_eq!(
            select_descriptor(&table, Some("sqlite")),
            Err(StorageGateError::UnknownEngine("sqlite".to_string()))
        );
    }

    #[test]
    fn vanilla_passes_t1_fails_t2() {
        let table = EngineTable::new();
        // Plain node (no tiers) → floor 1, vanilla passes.
        let sel = evaluate_storage_gate(&table, None, None, &[], &[]).unwrap();
        assert_eq!(sel.descriptor.name, "vanilla");
        assert_eq!(sel.asserts_tier, 1);
        // A node serving T2 with only vanilla → refuse to start.
        assert_eq!(
            evaluate_storage_gate(&table, None, None, &[2], &[]),
            Err(StorageGateError::EngineUnderDelivers {
                engine: "vanilla".to_string(),
                assurance: AssuranceClass::BestEffort,
                asserts: 2,
            })
        );
    }

    #[test]
    fn register_rejects_engine_with_foreign_kind() {
        // §4.12.5 value-based slot handshake: an engine declaring the wrong
        // kind GUID is not registered (loud), so it can't be selected.
        let mut t = EngineTable::new();
        register::<WrongKindEngine>(&mut t);
        assert!(t.get("wrong-kind").is_none());
        assert!(t.is_empty());
        // The correctly-kinded engine still registers.
        register::<DurableTestEngine>(&mut t);
        assert!(t.get("durable-test").is_some());
    }

    #[test]
    fn registered_durable_engine_serves_high_tier() {
        let table = table_with_durable();
        let sel =
            evaluate_storage_gate(&table, Some("durable-test"), Some(4), &[], &[]).unwrap();
        assert_eq!(sel.descriptor.name, "durable-test");
        assert_eq!(sel.asserts_tier, 4);
    }

    // The real SQLite engine through the spine — only built with the feature.
    #[cfg(feature = "store-sqlite")]
    #[test]
    fn sqlite_feature_registers_and_gate_accepts_at_t2() {
        let table = build_engine_table();
        assert!(table.get("sqlite").is_some(), "sqlite registered under feature");
        let sel = evaluate_storage_gate(&table, Some("sqlite"), Some(2), &[], &[]).unwrap();
        assert_eq!(sel.descriptor.name, "sqlite");
        assert_eq!(sel.descriptor.assurance, AssuranceClass::Durable);
        assert_eq!(sel.asserts_tier, 2);
    }
}
