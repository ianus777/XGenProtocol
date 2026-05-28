// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Phase 9 Compound C7 — `continue_from` pagination at boundary.
//!
//! Contract per runbook §5.2 + findings §3.7: F-1a tip-exchange + F-7 paginate
//! events through `collect_sync_history(runtime, requester_id, since, limit)`.
//! `limit` defaults to 1000 per `[sync].batch_size` (F-7a). The cursor cycle:
//! - First call with `since=""` returns up to `limit` events + `Some(cursor)`
//!   when more remain, or `None` when exhausted.
//! - Subsequent call with `since=cursor` resumes past that cursor.
//!
//! 4 tests at boundary values around the 1000-event page limit. Each test
//! ingests N message.text events as siblings (each referencing room_create
//! directly so the topological sort is O(N log N), not O(N²) per linear
//! chain) and drives `collect_sync_history` to completion, asserting:
//! 1. All events are returned across the cursor chain.
//! 2. No event appears in more than one page (no duplicates).
//! 3. Cursor is None on the final page.
//!
//! File location: xgen-node-side because `collect_sync_history` lives at
//! `xgen-node/src/fanout.rs` (Joe-locked at checkpoint #2). NOT declared in
//! `xgen-core/src/node/tests/mod.rs`.

use std::collections::HashSet;
use std::sync::Arc;

use ed25519_dalek::SigningKey;
use serde_json::json;
use tokio::sync::Mutex;

use xgen_common::wire::{Event, EventType};
use xgen_core::crypto::encoding;
use xgen_core::identity::{keypair, registry::IdentityRecord};
use xgen_core::node::runtime::NodeRuntime;
use xgen_core::space::state::{build_room_create_event, build_space_create_event, sign_event};

use crate::fanout::collect_sync_history;

use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};

const BATCH_SIZE: usize = 1000;

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
fn rdx(s: &str) -> RoomXgid {
    RoomXgid::from_xgid(Xgid::new(s.to_string()))
}
fn event_id_str(ev: &Event) -> String {
    ev.event_id
        .as_ref()
        .expect("event must have event_id")
        .as_str()
        .to_string()
}

fn pubkey_uri(key: &SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(key.verifying_key().as_bytes())
    )
}

fn make_record(key: &SigningKey, home_node: &str) -> IdentityRecord {
    IdentityRecord {
        identity_id: idx(&pubkey_uri(key)),
        display_name: None,
        is_ai: false,
        ai_capabilities: None,
        registered_at: "2026-05-25T00:00:00.000Z".to_string(),
        trust_assertion: None,
        devices: vec![],
        home_node: ndx(home_node),
        update_version: 0,
    }
}

/// Build N message.text events as siblings under room_create (each event's
/// prev_events = [room_id]). Keeps topological_sort_events at O(N log N)
/// rather than O(N²) per linear-chain DAG (N=2000 case stays under 1s).
///
/// Returns (rt, alice_id, total_candidate_count) where total_candidate_count
/// = N + 2 (1 space_create + 1 room_create + N messages).
fn setup_space_with_n_sibling_messages(n: usize) -> (NodeRuntime, String, usize) {
    let alice = keypair::generate();
    let alice_id = pubkey_uri(&alice);
    let node_key = keypair::generate();
    let mut rt = NodeRuntime::new(node_key);
    let rt_node_id_str = rt.node_id.as_str().to_string();
    rt.register_identity(make_record(&alice, &rt_node_id_str)).expect("alice");

    let space_ev = sign_event(
        build_space_create_event(&alice, "c7-space", None, 1, &rt_node_id_str),
        &alice,
    );
    let space_id: String = event_id_str(&space_ev);
    rt.ingest_event(space_ev);

    let room_ev = sign_event(
        build_room_create_event(&alice, &space_id, "general", None),
        &alice,
    );
    let room_id: String = event_id_str(&room_ev);
    rt.ingest_event(room_ev);

    // N sibling messages, each prev_events = [room_id].
    let prev = [room_id.clone()];
    for i in 0..n {
        let ev = sign_event(
            Event::new(
                EventType::MessageText,
                idx(&pubkey_uri(&alice)),
                rdx(&room_id),
                sdx(&space_id),
                prev.iter().map(|p| edx(p)).collect(),
                format!("2026-05-25T00:00:{:02}.000Z", i % 60),
                json!({ "body": format!("c7-{i}") }),
            ),
            &alice,
        );
        rt.ingest_event(ev);
    }

    let total_candidate = n + 2;
    (rt, alice_id, total_candidate)
}

/// Drive `collect_sync_history` to completion. Returns the chain of pages
/// in order: `Vec<(events_in_page, cursor_returned_for_this_page)>`.
async fn drain_pages(rt: Arc<Mutex<NodeRuntime>>, alice_id: &str) -> Vec<(Vec<Event>, Option<String>)> {
    let mut pages: Vec<(Vec<Event>, Option<String>)> = Vec::new();
    let mut cursor: String = String::new();
    let alice_id_typed = idx(alice_id);
    loop {
        let (page, next_cursor) = collect_sync_history(&rt, &alice_id_typed, &cursor, BATCH_SIZE).await;
        let next_cursor_clone = next_cursor.clone();
        let page_is_empty = page.is_empty();
        pages.push((page, next_cursor));
        match next_cursor_clone {
            Some(c) => cursor = c,
            None => break,
        }
        if page_is_empty {
            // Safety: cursor returned but page empty would loop forever.
            break;
        }
    }
    pages
}

/// Common assertion: pages chain returns all N+2 candidate events with no
/// duplicates and final cursor is None.
fn assert_pagination_complete(pages: &[(Vec<Event>, Option<String>)], expected_total: usize) {
    let mut all_seen: HashSet<String> = HashSet::new();
    let mut total = 0usize;
    for (page_idx, (page, cursor)) in pages.iter().enumerate() {
        for ev in page {
            let id: String = event_id_str(ev);
            assert!(
                all_seen.insert(id.clone()),
                "duplicate event_id {id} in page {page_idx}"
            );
            total += 1;
        }
        // Every page except possibly the last has a cursor; final page MUST
        // have None.
        let is_last = page_idx == pages.len() - 1;
        if !is_last {
            assert!(cursor.is_some(), "non-final page {page_idx} must carry a cursor");
        }
    }
    assert!(
        pages.last().map(|(_, c)| c.is_none()).unwrap_or(false),
        "final page cursor must be None"
    );
    assert_eq!(total, expected_total, "total returned events must match candidate count");
}

#[tokio::test(flavor = "current_thread")]
async fn c7_pagination_n_999_below_boundary() {
    // N=999 messages → 1001 candidate events. Page 1 caps at 1000 with cursor;
    // page 2 returns the trailing 1 with None.
    let (rt, alice_id, total) = setup_space_with_n_sibling_messages(999);
    let runtime = Arc::new(Mutex::new(rt));
    let pages = drain_pages(runtime, &alice_id).await;
    assert_pagination_complete(&pages, total);
    // Boundary shape: 2 pages (1000 + 1).
    assert_eq!(pages.len(), 2, "N=999 case: expected 2 pages (1000 + 1)");
    assert_eq!(pages[0].0.len(), BATCH_SIZE);
    assert_eq!(pages[1].0.len(), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn c7_pagination_n_1000_at_boundary_exact() {
    // N=1000 messages → 1002 candidate events. Page 1 caps at 1000 with cursor;
    // page 2 returns the trailing 2 with None.
    let (rt, alice_id, total) = setup_space_with_n_sibling_messages(1000);
    let runtime = Arc::new(Mutex::new(rt));
    let pages = drain_pages(runtime, &alice_id).await;
    assert_pagination_complete(&pages, total);
    assert_eq!(pages.len(), 2, "N=1000 case: expected 2 pages (1000 + 2)");
    assert_eq!(pages[0].0.len(), BATCH_SIZE);
    assert_eq!(pages[1].0.len(), 2);
}

#[tokio::test(flavor = "current_thread")]
async fn c7_pagination_n_1001_just_above_boundary() {
    // N=1001 messages → 1003 candidate events. Page 1 caps at 1000 with cursor;
    // page 2 returns the trailing 3 with None.
    let (rt, alice_id, total) = setup_space_with_n_sibling_messages(1001);
    let runtime = Arc::new(Mutex::new(rt));
    let pages = drain_pages(runtime, &alice_id).await;
    assert_pagination_complete(&pages, total);
    assert_eq!(pages.len(), 2, "N=1001 case: expected 2 pages (1000 + 3)");
    assert_eq!(pages[0].0.len(), BATCH_SIZE);
    assert_eq!(pages[1].0.len(), 3);
}

#[tokio::test(flavor = "current_thread")]
async fn c7_pagination_n_2000_double_boundary() {
    // N=2000 messages → 2002 candidate events. Page 1: 1000 + cursor; page 2:
    // 1000 + cursor; page 3: 2 + None.
    let (rt, alice_id, total) = setup_space_with_n_sibling_messages(2000);
    let runtime = Arc::new(Mutex::new(rt));
    let pages = drain_pages(runtime, &alice_id).await;
    assert_pagination_complete(&pages, total);
    assert_eq!(pages.len(), 3, "N=2000 case: expected 3 pages (1000 + 1000 + 2)");
    assert_eq!(pages[0].0.len(), BATCH_SIZE);
    assert_eq!(pages[1].0.len(), BATCH_SIZE);
    assert_eq!(pages[2].0.len(), 2);
}
