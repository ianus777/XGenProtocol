// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Local fan-out for the Node — broadcast accepted Events from one connected
// client to every other client that is a member of the Event's Space, and
// push history to brand-new joiners.
//
// The actual WebSocket I/O lives in `main.rs::handle_connection`. This module
// owns the data shapes and the routing decisions so they are unit-testable
// without spawning real sockets.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use crate::node::runtime::NodeRuntime;
use crate::wire::types::Event;

/// Outbound message the fan-out path pushes into a connected client's handler.
///
/// Phase 1 keeps this minimal — Events only. `transport.sync_complete` /
/// `transport.sync_response` wrappers from spec 3.3.6 are deferred; the client
/// reads the streamed Events directly until quiet.
#[derive(Debug, Clone)]
pub enum OutboundMsg {
    /// Deliver an Event to the client (Inbound from the client's perspective).
    Event(Event),
    /// Stream of historical events in response to a `transport.sync_request`
    /// or to a fresh `membership.join`.
    HistoryBatch { events: Vec<Event> },
}

/// Per-connection outbound channels keyed by authenticated `identity_id`.
/// Phase 1 simplification: one device per Identity, so one channel per Identity.
/// On disconnect the entry is removed; on reconnect a new entry is installed.
pub type ClientSenders = Arc<Mutex<HashMap<String, mpsc::Sender<OutboundMsg>>>>;

/// Result of processing an inbound message — describes what the fan-out path
/// should broadcast to other connected clients. Returning a description
/// (instead of doing the broadcast inside the runtime lock) keeps lock scope
/// short and lets the fan-out path lock `ClientSenders` separately.
pub struct FanoutRequest {
    /// The accepted Event to broadcast to all members of its Space (the author
    /// is filtered out at fan-out time).
    pub event: Option<Event>,
    /// `Some(joiner_id)` when the inbound was a fresh `membership.join` that
    /// added a new Space member. The fan-out path pushes the Space's full
    /// event history (in causal order, excluding the join event itself) to
    /// the joiner, so the new client sees prior `state.*` and `membership.*`
    /// events. Phase 1 reuses this for both join-time history and reconnect.
    pub new_joiner: Option<String>,
}

impl FanoutRequest {
    pub fn none() -> Self {
        Self { event: None, new_joiner: None }
    }
}

/// Resolve the Space ID a given Event addresses: `state.space_create` and
/// `state.dm_space_create` carry an empty `space_id` and use their own event_id;
/// every other Event carries the Space ID explicitly.
pub fn event_space_id(event: &Event) -> Option<String> {
    if event.space_id.is_empty() {
        event.event_id.clone()
    } else {
        Some(event.space_id.clone())
    }
}

/// Broadcast a `FanoutRequest` to the relevant connected clients.
///
/// Locks the runtime briefly to fetch the Space's member list and (when
/// applicable) the Space's event history, then drops the runtime lock before
/// acquiring the `ClientSenders` mutex. This keeps the critical sections short
/// and prevents the fan-out path from blocking other handlers.
pub async fn apply_fanout(
    req: FanoutRequest,
    author_id: &str,
    runtime: &Arc<Mutex<NodeRuntime>>,
    client_senders: &ClientSenders,
) {
    let event = match req.event {
        Some(ev) => ev,
        None => return,
    };
    let space_id = match event_space_id(&event) {
        Some(s) => s,
        None => return,
    };
    let event_id = event.event_id.clone();

    let (recipients, history_for_joiner): (Vec<String>, Option<Vec<Event>>) = {
        let rt = runtime.lock().await;
        let space = match rt.spaces.get(&space_id) {
            Some(s) => s,
            None => return,
        };
        let recipients = space.members.keys().cloned().collect::<Vec<_>>();
        let history = if req.new_joiner.is_some() {
            rt.stores.get(&space_id).map(|store| {
                let all: Vec<Event> = store.values().cloned().collect();
                let sorted = topological_sort_events(all);
                sorted
                    .into_iter()
                    .filter(|e| e.event_id != event_id)
                    .collect()
            })
        } else {
            None
        };
        (recipients, history)
    };

    let senders = client_senders.lock().await;

    for rid in &recipients {
        if rid == author_id {
            continue;
        }
        if let Some(tx) = senders.get(rid) {
            let _ = tx.try_send(OutboundMsg::Event(event.clone()));
        }
    }

    if let (Some(joiner_id), Some(history)) = (req.new_joiner.as_deref(), history_for_joiner) {
        if !history.is_empty() {
            if let Some(tx) = senders.get(joiner_id) {
                let _ = tx.try_send(OutboundMsg::HistoryBatch { events: history });
            }
        }
    }
}

/// Topological sort of a set of Events by `prev_events`. Events whose
/// predecessors are all already emitted come first. The DAG is acyclic by
/// construction (self-references rejected at insertion time), so this
/// terminates. Used to order history-push so the receiver sees parents
/// before children.
pub fn topological_sort_events(mut events: Vec<Event>) -> Vec<Event> {
    let mut emitted: HashSet<String> = HashSet::new();
    let mut out: Vec<Event> = Vec::with_capacity(events.len());
    let mut changed = true;
    while !events.is_empty() && changed {
        changed = false;
        let mut i = 0;
        while i < events.len() {
            let ready = events[i].prev_events.iter().all(|p| {
                emitted.contains(p) || !events.iter().any(|e| e.event_id.as_deref() == Some(p))
            });
            if ready {
                let ev = events.remove(i);
                if let Some(id) = &ev.event_id {
                    emitted.insert(id.clone());
                }
                out.push(ev);
                changed = true;
            } else {
                i += 1;
            }
        }
    }
    // Append any stragglers (cyclic or with unknown predecessors — neither
    // should occur in practice, but guarantee the function preserves all input).
    out.extend(events);
    out
}

/// Collect events to return in response to `transport.sync_request` (spec 3.3.6).
/// Returns all events from every Space the requester is a member of. If `since`
/// is non-empty, returns only events whose `event_id` follows `since` in the
/// store's insertion order. Phase 1 simplification: no per-Space filtering;
/// the client demultiplexes by `space_id` on receipt.
pub async fn collect_sync_history(
    runtime: &Arc<Mutex<NodeRuntime>>,
    requester_id: &str,
    since: &str,
) -> Vec<Event> {
    let rt = runtime.lock().await;
    let mut out: Vec<Event> = Vec::new();
    for (space_id, space) in &rt.spaces {
        if !space.is_member(requester_id) {
            continue;
        }
        if let Some(store) = rt.stores.get(space_id) {
            let all: Vec<Event> = store.values().cloned().collect();
            let sorted = topological_sort_events(all);
            if since.is_empty() {
                out.extend(sorted);
            } else {
                let mut past = false;
                for ev in sorted {
                    if past {
                        out.push(ev);
                    } else if ev.event_id.as_deref() == Some(since) {
                        past = true;
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::crypto::encoding;
    use crate::identity::{
        keypair,
        registry::{DeviceRecord, IdentityRecord},
    };
    use crate::message::exchange::build_message_text_event;
    use crate::space::state::{
        build_membership_event, build_room_create_event, build_space_create_event, sign_event,
    };
    use crate::wire::types::EventType;

    const HOME: &str = "xgen://pubkey/ed25519:NODE";

    fn pubkey_uri(key: &ed25519_dalek::SigningKey) -> String {
        format!(
            "xgen://pubkey/ed25519:{}",
            encoding::encode(key.verifying_key().as_bytes())
        )
    }

    fn make_identity_record(id: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: id.to_string(),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-16T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![DeviceRecord {
                device_id: id.to_string(),
                device_name: None,
                authorised_at: "2026-05-16T00:00:00.000Z".to_string(),
            }],
            home_node: HOME.to_string(),
            update_version: 0,
        }
    }

    /// Build a NodeRuntime with three identities (alice, bob, carol), a Space
    /// owned by alice, and a Room. Returns runtime + space_id + room_id +
    /// signing keys + each identity_id.
    fn setup_three_member_space() -> (
        NodeRuntime,
        String,
        String,
        ed25519_dalek::SigningKey,
        ed25519_dalek::SigningKey,
        ed25519_dalek::SigningKey,
    ) {
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let bob = keypair::generate();
        let carol = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        let carol_id = pubkey_uri(&carol);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        rt.register_identity(make_identity_record(&bob_id)).unwrap();
        rt.register_identity(make_identity_record(&carol_id)).unwrap();

        // Space + Room created by alice
        let space_ev = sign_event(
            build_space_create_event(&alice, "Test", None, 1, HOME),
            &alice,
        );
        let space_id = space_ev.event_id.clone().unwrap();
        rt.ingest_event(space_ev);

        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_id = room_ev.event_id.clone().unwrap();
        rt.ingest_event(room_ev);

        // Bob is invited and joins.
        let invite = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice,
        );
        rt.ingest_event(invite);
        let bob_join = sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        );
        rt.ingest_event(bob_join);

        // Carol is invited and joins.
        let invite_c = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": carol_id, "role": "member" }),
            ),
            &alice,
        );
        rt.ingest_event(invite_c);
        let carol_join = sign_event(
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({})),
            &carol,
        );
        rt.ingest_event(carol_join);

        (rt, space_id, room_id, alice, bob, carol)
    }

    fn install_sender(senders: &ClientSenders, identity_id: &str) -> mpsc::Receiver<OutboundMsg> {
        let (tx, rx) = mpsc::channel::<OutboundMsg>(256);
        let senders_clone = senders.clone();
        let id = identity_id.to_string();
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async move {
            senders_clone.lock().await.insert(id, tx);
        });
        rx
    }

    #[tokio::test]
    async fn message_fans_out_to_other_members_and_excludes_author() {
        let (rt, space_id, room_id, alice, bob, carol) = setup_three_member_space();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        let carol_id = pubkey_uri(&carol);
        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));

        let (tx_a, mut rx_a) = mpsc::channel::<OutboundMsg>(64);
        let (tx_b, mut rx_b) = mpsc::channel::<OutboundMsg>(64);
        let (tx_c, mut rx_c) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(alice_id.clone(), tx_a);
        senders.lock().await.insert(bob_id.clone(), tx_b);
        senders.lock().await.insert(carol_id.clone(), tx_c);

        // Get DAG tip for alice's outbound message.
        let tip = runtime.lock().await.dag_tips(&space_id)[0].clone();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "hello"),
            &alice,
        );

        let req = FanoutRequest { event: Some(msg.clone()), new_joiner: None };
        apply_fanout(req, &alice_id, &runtime, &senders).await;

        // Bob and Carol must receive the event; Alice (author) must not.
        let recv_b = rx_b.recv().await.expect("bob receives");
        let recv_c = rx_c.recv().await.expect("carol receives");
        match recv_b {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event"),
        }
        match recv_c {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event"),
        }
        // Alice's channel must be empty (the author is excluded).
        assert!(rx_a.try_recv().is_err());
        let _ = bob;
        let _ = carol;
    }

    #[tokio::test]
    async fn new_joiner_receives_full_history_push() {
        // Bootstrap a Space with alice + bob and several messages. Then carol
        // joins fresh; she must receive the full prior history (Space, Room,
        // both prior joins, plus the messages).
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let bob = keypair::generate();
        let carol = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        let carol_id = pubkey_uri(&carol);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        rt.register_identity(make_identity_record(&bob_id)).unwrap();
        rt.register_identity(make_identity_record(&carol_id)).unwrap();

        let space_ev =
            sign_event(build_space_create_event(&alice, "Test", None, 1, HOME), &alice);
        let space_id = space_ev.event_id.clone().unwrap();
        rt.ingest_event(space_ev);
        let room_ev = sign_event(
            build_room_create_event(&alice, &space_id, "general", None),
            &alice,
        );
        let room_id = room_ev.event_id.clone().unwrap();
        rt.ingest_event(room_ev);
        // Bob joins.
        rt.ingest_event(sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": bob_id, "role": "member" }),
            ),
            &alice,
        ));
        rt.ingest_event(sign_event(
            build_membership_event(&bob, &space_id, "", EventType::MembershipJoin, json!({})),
            &bob,
        ));
        // Alice posts a message.
        let tip = rt.dag_tips(&space_id)[0].clone();
        let alice_msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "first"),
            &alice,
        );
        rt.ingest_event(alice_msg);

        // Carol now joins. Bob's join is the relevant historical event row 7
        // analogue from the S1 pairing table — carol must see it.
        let carol_invite = sign_event(
            build_membership_event(
                &alice,
                &space_id,
                "",
                EventType::MembershipInvite,
                json!({ "target_identity": carol_id, "role": "member" }),
            ),
            &alice,
        );
        rt.ingest_event(carol_invite);
        let carol_join = sign_event(
            build_membership_event(&carol, &space_id, "", EventType::MembershipJoin, json!({})),
            &carol,
        );
        let carol_join_id = carol_join.event_id.clone().unwrap();
        rt.ingest_event(carol_join.clone());

        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
        let (tx_a, _rx_a) = mpsc::channel::<OutboundMsg>(64);
        let (tx_b, _rx_b) = mpsc::channel::<OutboundMsg>(64);
        let (tx_c, mut rx_c) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(alice_id.clone(), tx_a);
        senders.lock().await.insert(bob_id.clone(), tx_b);
        senders.lock().await.insert(carol_id.clone(), tx_c);

        let req = FanoutRequest {
            event: Some(carol_join.clone()),
            new_joiner: Some(carol_id.clone()),
        };
        apply_fanout(req, &carol_id, &runtime, &senders).await;

        // Carol receives one HistoryBatch with prior events (Space, Room,
        // Bob invite, Bob join, Alice message, Carol invite). The join event
        // itself is excluded (Carol's client already has its own outbound copy).
        let mut got_history: Option<Vec<Event>> = None;
        while let Ok(msg) = rx_c.try_recv() {
            if let OutboundMsg::HistoryBatch { events } = msg {
                got_history = Some(events);
                break;
            }
        }
        let history = got_history.expect("Carol must receive HistoryBatch");
        // Must NOT contain Carol's own join.
        assert!(
            history.iter().all(|e| e.event_id.as_deref() != Some(&carol_join_id)),
            "history must exclude the join event itself"
        );
        // Must contain Bob's prior join (row 7 analogue).
        let bob_join_present = history.iter().any(|e| {
            matches!(e.event_type, EventType::MembershipJoin) && e.sender == bob_id
        });
        assert!(bob_join_present, "carol must see Bob's prior membership.join");
        // Must contain the prior message.text from Alice.
        let prior_msg_present = history
            .iter()
            .any(|e| matches!(e.event_type, EventType::MessageText) && e.sender == alice_id);
        assert!(prior_msg_present, "carol must see Alice's prior message");
    }

    #[tokio::test]
    async fn fanout_skips_disconnected_recipients() {
        // Bob is a member but has no sender registered (not connected).
        // The fan-out must not panic and must still deliver to Carol.
        let (rt, space_id, room_id, alice, _bob, carol) = setup_three_member_space();
        let alice_id = pubkey_uri(&alice);
        let carol_id = pubkey_uri(&carol);
        let runtime = Arc::new(Mutex::new(rt));
        let senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
        let (tx_c, mut rx_c) = mpsc::channel::<OutboundMsg>(64);
        senders.lock().await.insert(carol_id.clone(), tx_c);

        let tip = runtime.lock().await.dag_tips(&space_id)[0].clone();
        let msg = sign_event(
            build_message_text_event(&alice, &space_id, &room_id, vec![tip], "hi"),
            &alice,
        );
        let req = FanoutRequest { event: Some(msg.clone()), new_joiner: None };
        apply_fanout(req, &alice_id, &runtime, &senders).await;

        match rx_c.recv().await.unwrap() {
            OutboundMsg::Event(ev) => assert_eq!(ev.event_id, msg.event_id),
            _ => panic!("expected Event"),
        }
        let _ = carol;
    }

    #[tokio::test]
    async fn collect_sync_history_returns_only_member_spaces() {
        // Alice is in Space A; Bob is in Space B (not in A). A sync_request
        // from Bob must return only Space B's events.
        let node_key = keypair::generate();
        let mut rt = NodeRuntime::new(node_key);
        let alice = keypair::generate();
        let bob = keypair::generate();
        let alice_id = pubkey_uri(&alice);
        let bob_id = pubkey_uri(&bob);
        rt.register_identity(make_identity_record(&alice_id)).unwrap();
        rt.register_identity(make_identity_record(&bob_id)).unwrap();

        let space_a = sign_event(
            build_space_create_event(&alice, "A", None, 1, HOME),
            &alice,
        );
        let space_a_id = space_a.event_id.clone().unwrap();
        rt.ingest_event(space_a);
        let space_b = sign_event(
            build_space_create_event(&bob, "B", None, 1, HOME),
            &bob,
        );
        let space_b_id = space_b.event_id.clone().unwrap();
        rt.ingest_event(space_b);

        let runtime = Arc::new(Mutex::new(rt));
        let events_for_bob = collect_sync_history(&runtime, &bob_id, "").await;
        // Bob is a member only of Space B; sync_history must contain only its
        // space_create (no Space A leak).
        assert!(
            events_for_bob
                .iter()
                .all(|e| event_space_id(e).as_deref() == Some(&space_b_id)),
            "Bob's sync history must be limited to spaces he is a member of"
        );
        assert!(
            events_for_bob
                .iter()
                .any(|e| e.event_id.as_deref() == Some(&space_b_id)),
            "Bob's sync history must include Space B's create event"
        );
        let _ = space_a_id;
    }
}
