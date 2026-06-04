// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Storage-engine substitution end-to-end (SE-SUB-D1…D6), feature-gated on the
//! real SQLite engine. Proves the load-bearing Scope-B property: when an engine
//! is active it is the **authoritative durability** — a Space written through the
//! per-Space factory survives a "restart" (fresh `NodeRuntime`) by rehydrating
//! from `<dir>/*.db` via `engine.range(0)`, NOT from any JSON scan — and Spaces
//! are physically isolated (one `.db` each, SE-SUB-D1).
//!
//! Vanilla-mode behaviour-neutrality is covered by the (always-on) xgen-core
//! `runtime.rs` tests (`default_store_factory_is_vanilla_and_behaviour_neutral`,
//! `ensure_store_failure_never_silently_falls_back_to_vanilla`).

#![cfg(feature = "store-sqlite")]

use xgen_common::xgid::{SpaceXgid, Xgid};
use xgen_core::dag::store::StoreFactory;
use xgen_core::identity::keypair;
use xgen_core::node::runtime::NodeRuntime;
use xgen_core::space::state::{build_room_create_event, build_space_create_event, sign_event};

/// Build the per-Space sqlite factory exactly as `run_node` does (shared
/// `build_engine_store_factory`, so no second copy of the templating logic).
fn engine_factory_for(dir: &std::path::Path) -> StoreFactory {
    let table = crate::storage_engine::build_engine_table();
    let ef = table
        .get("sqlite")
        .expect("sqlite registered under the feature")
        .factory;
    crate::storage_engine::build_engine_store_factory(ef, dir.to_path_buf())
}

fn sid(s: &str) -> SpaceXgid {
    SpaceXgid::from_xgid(Xgid::new(s.to_string()))
}

#[test]
fn engine_backed_space_survives_restart_via_engine_replay() {
    let dir = tempfile::tempdir().unwrap();
    let alice = keypair::generate();

    // ── Run 1 — engine-backed runtime ingests a two-event Space. ──
    let space_id_str: String;
    {
        let mut rt = NodeRuntime::new(keypair::generate());
        rt.set_store_factory(engine_factory_for(dir.path()));
        assert!(rt.engine_owns_durability);

        let space_ev = sign_event(
            build_space_create_event(&alice, "s", None, 1, "node", None),
            &alice,
        );
        space_id_str = space_ev.event_id.as_ref().unwrap().as_str().to_string();
        rt.ingest_event(space_ev);
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id_str, "r", None),
            &alice,
        );
        rt.ingest_event(room_ev);

        let s = sid(&space_id_str);
        assert!(rt.spaces.contains_key(&s));
        assert_eq!(rt.stores.get(&s).unwrap().len(), 2, "two events in the engine store");

        // The engine wrote its own `.db`; nothing wrote a `.json` shadow here.
        let stem = crate::app::space_file_stem(&space_id_str);
        assert!(dir.path().join(format!("{stem}.db")).exists(), "engine .db written");
        assert!(!dir.path().join(format!("{stem}.json")).exists(), "no JSON shadow");
    } // rt dropped — only the .db file persists.

    // ── Run 2 — fresh runtime, same engine dir → rehydrate from the .db. ──
    let mut rt2 = NodeRuntime::new(keypair::generate());
    rt2.set_store_factory(engine_factory_for(dir.path()));
    let rehydrated = crate::app::rehydrate_spaces_from_engine_dir(&mut rt2, dir.path());
    assert_eq!(rehydrated, 1, "one Space rehydrated from <dir>/*.db");

    let s = sid(&space_id_str);
    assert!(
        rt2.spaces.contains_key(&s),
        "SpaceState rebuilt from the engine (not a JSON scan)"
    );
    assert_eq!(
        rt2.stores.get(&s).unwrap().len(),
        2,
        "both events survived restart via engine replay"
    );
}

#[test]
fn two_spaces_are_isolated_in_separate_db_files() {
    let dir = tempfile::tempdir().unwrap();
    let alice = keypair::generate();
    let mut rt = NodeRuntime::new(keypair::generate());
    rt.set_store_factory(engine_factory_for(dir.path()));

    let s1 = sign_event(build_space_create_event(&alice, "s1", None, 1, "node", None), &alice);
    let s2 = sign_event(build_space_create_event(&alice, "s2", None, 1, "node", None), &alice);
    rt.ingest_event(s1);
    rt.ingest_event(s2);

    let db_files = std::fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("db"))
        .count();
    assert_eq!(db_files, 2, "two Spaces → two physically-isolated .db files (SE-SUB-D1)");
}
