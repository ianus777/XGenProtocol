// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M11 (`self` thread, D-021) — Commit 2 verb witnesses.
//!
//! Reuses the deterministic stub-WS home Node from `send_confirm_integration`
//! (a real ephemeral WS that acks the submitted events). Proves the M11-D5
//! `self` convenience verb: it auto-resolves the session identity (no typed id),
//! labels the thread `"self"`, and is **create-if-absent** — a second open finds
//! the existing thread **offline** (no network round-trip) and never duplicates it.

use xgen_core::identity::keypair;

use crate::ops::{self_open, OpContext};
use crate::tests::send_confirm_integration::{
    session_for, spawn_confirm_stub, state_path, write_fast_config, Act,
};

// `self` first-open creates the thread (invitee = session identity, labelled
// "self"); second-open is create-if-absent — it finds the existing thread
// offline (the stub accepted only the first connection), returns the same ids
// with `created = false`, and writes no duplicate KnownSpace. One test covers
// V-idempotent + V-autotarget + the "self" label (the label is written only when
// invitee == creator, so it transitively proves the auto-target).
#[tokio::test]
async fn self_open_creates_then_is_create_if_absent_idempotent() {
    let key = keypair::generate();
    let tmp = tempfile::tempdir().unwrap();
    write_fast_config(tmp.path());
    // The home Node acks the 3-event DM create chain on the first connection.
    let addr = spawn_confirm_stub(vec![Act::Ack, Act::Ack, Act::Ack]).await;
    let url = format!("ws://{addr}/xgen");
    let dd = tmp.path().to_path_buf();

    // First open → creates the self thread.
    let (sp1, rm1) = {
        let mut session = session_for(url.clone(), &key, tmp.path());
        let mut ctx = OpContext {
            session: &mut session,
            data_dir: &dd,
            node_override: None,
        };
        let r = self_open(&mut ctx)
            .await
            .expect("first self open should create the thread");
        assert!(r.created, "first open creates the self thread");
        (
            r.space_id.as_str().to_string(),
            r.room_id.as_str().to_string(),
        )
    };

    // The created thread is recorded owned + labelled "self" (auto-target proof:
    // create_dm_space only labels "self" when invitee == creator).
    {
        let raw = std::fs::read_to_string(state_path(tmp.path())).unwrap();
        let st: xgen_common::state::ClientState = serde_json::from_str(&raw).unwrap();
        let selfsp = st
            .spaces
            .iter()
            .find(|s| s.name == "self")
            .expect("a 'self'-labelled KnownSpace");
        assert_eq!(selfsp.role, "owner");
        assert_eq!(selfsp.space_id, sp1);
    }

    // Second open → create-if-absent finds it OFFLINE. The stub accepted only the
    // first connection, so any attempt to connect here would fail; success proves
    // the found path returns before connecting.
    let mut session = session_for(url, &key, tmp.path());
    let mut ctx = OpContext {
        session: &mut session,
        data_dir: &dd,
        node_override: None,
    };
    let r2 = self_open(&mut ctx)
        .await
        .expect("second self open should find the existing thread offline");
    assert!(!r2.created, "an existing self thread is opened, not recreated");
    assert_eq!(r2.space_id.as_str(), sp1, "same space id on re-open");
    assert_eq!(r2.room_id.as_str(), rm1, "same room id on re-open");

    // No duplicate self thread.
    let raw = std::fs::read_to_string(state_path(tmp.path())).unwrap();
    let st: xgen_common::state::ClientState = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        st.spaces.iter().filter(|s| s.name == "self").count(),
        1,
        "no duplicate self thread after a second open"
    );
}
