// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! PG-09 / Arc B — Unknown-event forward-compat (Fork 2: relay-unknown).
//!
//! End-to-end at the `NodeRuntime` level. A signed event whose `type` is not in
//! the known set MUST:
//!   * deserialize + pass the F-4 validation core (type-blind) → `Accepted`,
//!   * be stored as a real DAG node (FC-D3) and be referenceable as a
//!     predecessor of a later event (FC-D4),
//!   * round-trip through the store / cold-start replay byte-identically (FC-D3),
//!   * NEVER mutate Space/Room state (FC-D6 — the apply chokepoint no-ops it).
//!
//! Known-type flows are unchanged (covered by the rest of the suite).

use crate::{
    crypto::encoding,
    identity::{keypair, registry::IdentityRecord},
    node::runtime::{DispatchOutcome, EventOrigin, NodeRuntime},
    space::state::{build_room_create_event, build_space_create_event, sign_event},
    wire::types::{Event, EventType},
};
use ed25519_dalek::SigningKey;
use serde_json::json;
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};

fn pubkey_uri(key: &SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(key.verifying_key().as_bytes())
    )
}
fn idx(s: &str) -> IdentityXgid {
    IdentityXgid::from_xgid(Xgid::new(s.to_string()))
}
fn ndx(s: &str) -> NodeXgid {
    NodeXgid::from_xgid(Xgid::new(s.to_string()))
}
fn sdx(s: &str) -> SpaceXgid {
    SpaceXgid::from_xgid(Xgid::new(s.to_string()))
}
fn edx(s: &str) -> EventXgid {
    EventXgid::from_xgid(Xgid::new(s.to_string()))
}
fn event_id_str(ev: &Event) -> String {
    ev.event_id
        .as_ref()
        .expect("signed event has event_id")
        .as_str()
        .to_string()
}

fn make_record(key: &SigningKey, home_node: &str) -> IdentityRecord {
    IdentityRecord {
        identity_id: idx(&pubkey_uri(key)),
        display_name: None,
        is_ai: false,
        ai_capabilities: None,
        registered_at: "2026-06-03T00:00:00.000Z".to_string(),
        trust_assertion: None,
        devices: vec![],
        home_node: ndx(home_node),
        update_version: 0,
        revoked: false,
        revoked_at: None,
        revocation_reason: None,
    }
}

/// NodeRuntime with Alice registered + an Alice-owned Space + a Room.
/// Returns `(rt, alice_key, space_id)`.
fn setup() -> (NodeRuntime, SigningKey, String) {
    let alice = keypair::generate();
    let node_key = keypair::generate();
    let mut rt = NodeRuntime::new(node_key);
    rt.register_identity(make_record(&alice, rt.node_id.as_str()))
        .expect("alice registration");

    let space_ev = sign_event(
        build_space_create_event(&alice, "fc-space", None, 1, rt.node_id.as_str()),
        &alice,
    );
    let space_id = event_id_str(&space_ev);
    rt.ingest_event(space_ev);

    let room_ev = sign_event(
        build_room_create_event(&alice, &space_id, "general", None),
        &alice,
    );
    rt.ingest_event(room_ev);

    (rt, alice, space_id)
}

/// A signed Space-scoped event with an UNKNOWN type, chained onto `prev`.
/// Space-scoped (empty `room_id`) so the only membership requirement is
/// Space membership — Alice (the owner) satisfies it.
fn signed_unknown_event(
    sender: &SigningKey,
    space_id: &str,
    type_str: &str,
    prev: Vec<String>,
) -> Event {
    let unsigned = Event::new(
        EventType::Unknown(type_str.to_string()),
        idx(&pubkey_uri(sender)),
        RoomXgid::from_xgid(Xgid::new(String::new())),
        sdx(space_id),
        prev.into_iter().map(|s| edx(&s)).collect(),
        "2026-06-03T00:00:01.000Z".to_string(),
        json!({ "experimental": "payload", "n": 7 }),
    );
    sign_event(unsigned, sender)
}

#[test]
fn unknown_event_validates_stores_and_is_not_applied() {
    let (mut rt, alice, space_id) = setup();
    let sx = sdx(&space_id);

    // Snapshot Space state before the unknown event lands.
    let (members_before, rooms_before, name_before, dm_before) = {
        let s = &rt.spaces[&sx];
        (
            s.members.len(),
            s.rooms.len(),
            s.name.clone(),
            s.dm_constraints_active,
        )
    };

    let ev = signed_unknown_event(&alice, &space_id, "com.example.experimental", rt.dag_tips(&sx));
    let unknown_id = event_id_str(&ev);

    let outcome = rt.dispatch_event(ev, EventOrigin::LocallySubmitted, None);
    assert!(
        matches!(outcome, DispatchOutcome::Accepted { .. }),
        "unknown-type event must validate + accept, got {outcome:?}"
    );

    // FC-D3 / FC-D4 — stored as a real DAG node and now a tip (referenceable).
    let store = rt.stores.get(&sx).expect("space store");
    assert!(
        store.contains(&edx(&unknown_id)),
        "unknown event must be in the store"
    );
    assert!(
        rt.dag_tips(&sx).contains(&unknown_id),
        "unknown event must be a DAG tip"
    );

    // FC-D6 — Space state is byte-unchanged: the event was never applied.
    let s = &rt.spaces[&sx];
    assert_eq!(s.members.len(), members_before, "members must not change");
    assert_eq!(s.rooms.len(), rooms_before, "rooms must not change");
    assert_eq!(s.name, name_before, "name must not change");
    assert_eq!(
        s.dm_constraints_active, dm_before,
        "dm constraints must not change"
    );
}

#[test]
fn unknown_event_round_trips_through_store_and_replay_byte_identically() {
    let (mut rt, alice, space_id) = setup();
    let sx = sdx(&space_id);

    let ev = signed_unknown_event(&alice, &space_id, "org.acme.widget", rt.dag_tips(&sx));
    let unknown_id = event_id_str(&ev);
    let original_json = serde_json::to_string(&ev).unwrap();

    assert!(matches!(
        rt.dispatch_event(ev, EventOrigin::LocallySubmitted, None),
        DispatchOutcome::Accepted { .. }
    ));

    // FC-D3 — fetch from the store, re-serialize: byte-identical, type survives
    // as Unknown(raw-string).
    let fetched = rt
        .stores
        .get(&sx)
        .unwrap()
        .get(&edx(&unknown_id))
        .unwrap()
        .expect("event present in store");
    assert_eq!(
        fetched.event_type,
        EventType::Unknown("org.acme.widget".to_string())
    );
    assert_eq!(
        serde_json::to_string(&fetched).unwrap(),
        original_json,
        "store round-trip must be byte-identical"
    );

    // Cold-start replay into a fresh runtime: the unknown event survives the
    // store → range → ingest cycle byte-identically and is still not applied.
    let stored: Vec<Event> = rt.stores.get(&sx).unwrap().range(0).unwrap();
    let mut rt2 = NodeRuntime::new(keypair::generate());
    for e in stored {
        rt2.ingest_event(e);
    }
    let replayed = rt2
        .stores
        .get(&sx)
        .unwrap()
        .get(&edx(&unknown_id))
        .unwrap()
        .expect("event present after replay");
    assert_eq!(
        serde_json::to_string(&replayed).unwrap(),
        original_json,
        "replay must round-trip byte-identically"
    );
}

#[test]
fn unknown_event_is_referenceable_as_predecessor() {
    // FC-D4 — a later event may list an unknown event in its prev_events; the
    // DAG treats it as a normal node, no special-casing.
    let (mut rt, alice, space_id) = setup();
    let sx = sdx(&space_id);

    let u1 = signed_unknown_event(&alice, &space_id, "com.example.v1", rt.dag_tips(&sx));
    let u1_id = event_id_str(&u1);
    assert!(matches!(
        rt.dispatch_event(u1, EventOrigin::LocallySubmitted, None),
        DispatchOutcome::Accepted { .. }
    ));

    // A second event whose sole predecessor IS the first (unknown) event.
    let u2 = signed_unknown_event(&alice, &space_id, "com.example.v2", vec![u1_id.clone()]);
    let u2_id = event_id_str(&u2);
    assert!(
        matches!(
            rt.dispatch_event(u2, EventOrigin::LocallySubmitted, None),
            DispatchOutcome::Accepted { .. }
        ),
        "an event referencing an unknown predecessor must be accepted"
    );

    let store = rt.stores.get(&sx).unwrap();
    assert!(store.contains(&edx(&u1_id)) && store.contains(&edx(&u2_id)));
    // u2 references u1, so u1 is no longer a tip; u2 is.
    let tips = rt.dag_tips(&sx);
    assert!(
        tips.contains(&u2_id) && !tips.contains(&u1_id),
        "u2 must be the tip; u1 must have been superseded"
    );
}
