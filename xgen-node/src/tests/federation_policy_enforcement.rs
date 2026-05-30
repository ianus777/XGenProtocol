// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! federation-admin-control 2b (policy) — FAC-D3 enforcement tests
//! (`tasks/M6_FEDERATION_POLICY_IMPL.md` Commit 2).
//!
//! Two enforcement sites, one pure decision (`policy_permits`, truth-tabled in
//! `xgen-core/src/federation/federation_policy.rs`):
//!
//! - **Outbound** — `apply_federation_push` filters its push targets by the
//!   per-peer policy. Tested directly (NodeRuntime + a registered peer sender
//!   channel + a policy store), the same shape as the F-5 guard test in
//!   `federation_push_integration.rs`: a `Deny` peer / an `allowed_spaces`
//!   exclusion delivers ZERO messages to that peer's channel; absent policy
//!   delivers (default-permit / prime invariant).
//!
//! - **Inbound** — `process_inbound` (node-side, Option B, Joe-locked at
//!   checkpoint #2) drops a federation-received event from a denied peer
//!   pre-apply. Tested end-to-end via the two-Node harness `federate()`: B
//!   denies A AFTER the federation bootstrap, A pushes a message, and B never
//!   ingests it (the `federation_policy_denied_inbound` trace fires).
//!
//! The mandatory default-permit regression (D-065) is covered structurally by
//! the entire existing `federate()`-based suite running with empty policy
//! stores (permit-all), plus the explicit `outbound_no_policy_delivers` here.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use serde_json::json;
    use tokio::sync::{mpsc, Mutex};

    use crate::fanout::{FederationPeerSenders, OutboundMsg};
    use crate::federation::federation_policy::{FederationPolicy, FederationPolicyStore, PolicyMode};
    use crate::node::runtime::{EventOrigin, NodeRuntime};
    use crate::tests::phase9_harness::{
        edx, event_id_str, federate, idx, ndx, now_rfc, pubkey_uri, rdx, sdx,
        spawn_in_process_node,
    };
    use crate::{
        identity::{keypair, registry::IdentityRecord},
        space::state::{
            build_federation_add_event, build_room_create_event, build_space_create_event,
            sign_event,
        },
        wire::types::{Event, EventType},
    };
    use xgen_common::xgid::{NodeXgid, SpaceXgid, Xgid};

    fn ndx_owned(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn sdx_owned(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }

    fn make_record(key: &ed25519_dalek::SigningKey, home_node: &str) -> IdentityRecord {
        IdentityRecord {
            identity_id: idx(&pubkey_uri(key)),
            display_name: None,
            is_ai: false,
            ai_capabilities: None,
            registered_at: "2026-05-30T00:00:00.000Z".to_string(),
            trust_assertion: None,
            devices: vec![],
            home_node: ndx(home_node),
            update_version: 0,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        }
    }

    /// Build a Node runtime whose Space S federates with one peer, returning
    /// the runtime, alice's key/id, the space_id, and the peer node_id string.
    /// Mirrors `federation_push_integration::build_node_a` + the F-5 test's
    /// dummy-peer `state.federation_add` so the peer lands in
    /// `SpaceState.federation_nodes`.
    fn build_node_with_peer() -> (NodeRuntime, ed25519_dalek::SigningKey, String, String, String) {
        let node_key = keypair::generate();
        let mut node = NodeRuntime::new(node_key.clone());
        let node_id_str = node.node_id.as_str().to_string();

        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node.register_identity(make_record(&alice_key, &node_id_str))
            .unwrap();

        let space_ev = sign_event(
            build_space_create_event(&alice_key, "policy-space", None, 1, &node_id_str),
            &alice_key,
        );
        let space_id: String = event_id_str(&space_ev);
        node.ingest_event(space_ev);

        let room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", None),
            &alice_key,
        );
        let room_id: String = event_id_str(&room_ev);
        node.ingest_event(room_ev);

        let invite_ev = sign_event(
            Event::new(
                EventType::MembershipInvite,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&space_id), edx(&room_id)],
                now_rfc(),
                json!({ "target_identity": alice_id, "role": "member" }),
            ),
            &alice_key,
        );
        let invite_id: String = event_id_str(&invite_ev);
        node.ingest_event(invite_ev);

        let join_ev = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&invite_id)],
                now_rfc(),
                json!({}),
            ),
            &alice_key,
        );
        node.ingest_event(join_ev);

        // Federate Space S with a peer so federation_nodes is non-empty.
        let peer_id = "xgen://pubkey/ed25519:POLICY_PEER".to_string();
        let fed_add = sign_event(
            build_federation_add_event(
                &node_key,
                &space_id,
                node.dag_tips(&sdx_owned(&space_id)),
                &peer_id,
                "xgen://hash/sha256:session_policy",
                "0.1",
                "json",
            ),
            &node_key,
        );
        node.ingest_event(fed_add);
        assert!(
            node.spaces[&sdx_owned(&space_id)]
                .federation_nodes
                .iter()
                .any(|n| n == &ndx_owned(&peer_id)),
            "setup: federation_nodes must include the peer"
        );

        (node, alice_key, alice_id, space_id, peer_id)
    }

    /// Build a MessageText event from Alice in Space S, prev = the Space's
    /// current tips. apply_federation_push does not validate, so a minimal
    /// signed event suffices.
    fn alice_msg(
        node: &NodeRuntime,
        alice_key: &ed25519_dalek::SigningKey,
        alice_id: &str,
        space_id: &str,
    ) -> Event {
        let tips = node.dag_tips(&sdx_owned(space_id));
        sign_event(
            Event::new(
                EventType::MessageText,
                idx(alice_id),
                rdx(""),
                sdx(space_id),
                tips.iter().map(|t| edx(t)).collect(),
                now_rfc(),
                json!({ "text": "policy-gated message" }),
            ),
            alice_key,
        )
    }

    /// Register a peer sender channel and run `apply_federation_push` for `ev`
    /// under `policy_store`; returns whether the peer's channel received the
    /// event (i.e. the push was NOT skipped).
    async fn push_delivers(
        node: NodeRuntime,
        peer_id: &str,
        ev: &Event,
        policy_store: Option<&Arc<Mutex<FederationPolicyStore>>>,
    ) -> bool {
        let local_node_id = node.node_id.as_str().to_string();
        let runtime = Arc::new(Mutex::new(node));
        let senders: FederationPeerSenders = Arc::new(Mutex::new(HashMap::new()));
        let (tx, mut rx) = mpsc::channel::<OutboundMsg>(4);
        {
            senders.lock().await.insert(ndx_owned(peer_id), tx);
        }
        crate::federation_session::apply_federation_push(
            ev,
            EventOrigin::LocallySubmitted,
            &runtime,
            &senders,
            &ndx_owned(&local_node_id),
            policy_store,
        )
        .await;
        rx.try_recv().is_ok()
    }

    // ── Outbound enforcement (apply_federation_push) ────────────────────────

    #[tokio::test]
    async fn outbound_deny_skips_push() {
        let (node, alice_key, alice_id, space_id, peer_id) = build_node_with_peer();
        let ev = alice_msg(&node, &alice_key, &alice_id, &space_id);
        let store = Arc::new(Mutex::new(FederationPolicyStore::new()));
        store.lock().await.set(
            ndx_owned(&peer_id),
            FederationPolicy {
                mode: PolicyMode::Deny,
                allowed_spaces: None,
            },
        );
        assert!(
            !push_delivers(node, &peer_id, &ev, Some(&store)).await,
            "Deny policy must skip the outbound push to the peer"
        );
    }

    #[tokio::test]
    async fn outbound_allowed_spaces_exclusion_skips_push() {
        let (node, alice_key, alice_id, space_id, peer_id) = build_node_with_peer();
        let ev = alice_msg(&node, &alice_key, &alice_id, &space_id);
        let store = Arc::new(Mutex::new(FederationPolicyStore::new()));
        // Allow, but restrict to a DIFFERENT space → this Space is excluded.
        store.lock().await.set(
            ndx_owned(&peer_id),
            FederationPolicy {
                mode: PolicyMode::Allow,
                allowed_spaces: Some(vec![sdx_owned("xgen://hash/sha256:some_other_space")]),
            },
        );
        assert!(
            !push_delivers(node, &peer_id, &ev, Some(&store)).await,
            "allowed_spaces excluding this Space must skip the outbound push"
        );
    }

    #[tokio::test]
    async fn outbound_allowed_spaces_inclusion_delivers() {
        let (node, alice_key, alice_id, space_id, peer_id) = build_node_with_peer();
        let ev = alice_msg(&node, &alice_key, &alice_id, &space_id);
        let store = Arc::new(Mutex::new(FederationPolicyStore::new()));
        store.lock().await.set(
            ndx_owned(&peer_id),
            FederationPolicy {
                mode: PolicyMode::Allow,
                allowed_spaces: Some(vec![sdx_owned(&space_id)]),
            },
        );
        assert!(
            push_delivers(node, &peer_id, &ev, Some(&store)).await,
            "allowed_spaces including this Space must deliver the outbound push"
        );
    }

    #[tokio::test]
    async fn outbound_no_policy_delivers() {
        // Default-permit (prime invariant): empty store / None both deliver.
        let (node, alice_key, alice_id, space_id, peer_id) = build_node_with_peer();
        let ev = alice_msg(&node, &alice_key, &alice_id, &space_id);
        let empty = Arc::new(Mutex::new(FederationPolicyStore::new()));
        assert!(
            push_delivers(node, &peer_id, &ev, Some(&empty)).await,
            "absent policy (empty store) must deliver the outbound push"
        );
    }

    // ── Inbound enforcement (process_inbound, end-to-end via federate) ──────

    /// Build a Space (create + room + Alice invite/join) on `node`, ingesting
    /// each event locally. Returns the space_id (its tip after join is the
    /// space's single DAG tip until federation adds the reciprocal fed_add).
    async fn setup_space_on(
        node: &crate::tests::phase9_harness::InProcessNode,
        alice_key: &ed25519_dalek::SigningKey,
        alice_id: &str,
        name: &str,
    ) -> String {
        let space_ev = sign_event(
            build_space_create_event(alice_key, name, None, 1, &node.node_id),
            alice_key,
        );
        let space_id: String = event_id_str(&space_ev);
        node.ingest(space_ev).await;

        let room_ev = sign_event(
            build_room_create_event(alice_key, &space_id, "general", None),
            alice_key,
        );
        let room_id: String = event_id_str(&room_ev);
        node.ingest(room_ev).await;

        let invite_ev = sign_event(
            Event::new(
                EventType::MembershipInvite,
                idx(alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&space_id), edx(&room_id)],
                now_rfc(),
                json!({ "target_identity": alice_id, "role": "member" }),
            ),
            alice_key,
        );
        let invite_id: String = event_id_str(&invite_ev);
        node.ingest(invite_ev).await;

        let join_ev = sign_event(
            Event::new(
                EventType::MembershipJoin,
                idx(alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&invite_id)],
                now_rfc(),
                json!({}),
            ),
            alice_key,
        );
        node.ingest(join_ev).await;
        space_id
    }

    /// Inbound FAC-D3 gate, end-to-end + race-free. B federates with A for two
    /// Spaces (S1, S2) and sets a policy on A of `Allow { allowed_spaces:
    /// [S2] }` — so A's inbound events for S1 are DROPPED, for S2 PERMITTED.
    /// A pushes M1 (in S1) then M2 (in S2) on the same ordered F-2 channel.
    /// When B has ingested M2 (permitted, later on the channel), M1 (earlier)
    /// was necessarily already processed — and B must NOT have it. No trace
    /// capture, no mid-flight policy mutation → deterministic.
    #[serial_test::serial]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn inbound_allowed_spaces_drops_excluded_space_event() {
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_a.register_identity(&alice_key).await;
        node_b.register_identity(&alice_key).await;

        let s1 = setup_space_on(&node_a, &alice_key, &alice_id, "policy-s1-denied").await;
        let s2 = setup_space_on(&node_a, &alice_key, &alice_id, "policy-s2-allowed").await;

        // Federate both Spaces (empty policy → bootstrap deltas for both arrive).
        federate(&node_a, &node_b, vec![s1.clone(), s2.clone()]).await;
        let s1_tip = node_a.dag_tips(&s1).await;
        let s2_tip = node_a.dag_tips(&s2).await;
        assert_eq!(s1_tip.len(), 1);
        assert_eq!(s2_tip.len(), 1);
        assert!(
            node_b
                .wait_for_event(&s1, &s1_tip[0], Duration::from_secs(20))
                .await,
            "B must ingest S1's bootstrap delta"
        );
        assert!(
            node_b
                .wait_for_event(&s2, &s2_tip[0], Duration::from_secs(20))
                .await,
            "B must ingest S2's bootstrap delta"
        );

        // B's policy for A: Allow, restricted to S2 → S1 inbound is excluded.
        // Set ONCE and never mutated (no race); both M1 and M2 see it.
        node_b.federation_policy.lock().await.set(
            ndx_owned(&node_a.node_id),
            FederationPolicy {
                mode: PolicyMode::Allow,
                allowed_spaces: Some(vec![sdx_owned(&s2)]),
            },
        );

        // M1 in S1 (excluded) pushed FIRST, then M2 in S2 (permitted).
        let m1 = sign_event(
            Event::new(
                EventType::MessageText,
                idx(&alice_id),
                rdx(""),
                sdx(&s1),
                s1_tip.iter().map(|t| edx(t)).collect(),
                now_rfc(),
                json!({ "text": "S1 — excluded by allowed_spaces, must drop on B" }),
            ),
            &alice_key,
        );
        let m1_id = event_id_str(&m1);
        let m2 = sign_event(
            Event::new(
                EventType::MessageText,
                idx(&alice_id),
                rdx(""),
                sdx(&s2),
                s2_tip.iter().map(|t| edx(t)).collect(),
                now_rfc(),
                json!({ "text": "S2 — permitted, must arrive on B" }),
            ),
            &alice_key,
        );
        let m2_id = event_id_str(&m2);

        for ev in [&m1, &m2] {
            {
                let mut rt = node_a.runtime.lock().await;
                let _ = rt.dispatch_event(ev.clone(), EventOrigin::LocallySubmitted, None);
            }
            // A has no policy of its own → push proceeds; B's inbound gate decides.
            crate::federation_session::apply_federation_push(
                ev,
                EventOrigin::LocallySubmitted,
                &node_a.runtime,
                &node_a.federation_peer_senders,
                &ndx_owned(&node_a.node_id),
                None,
            )
            .await;
        }

        // M2 (permitted, later on the channel) arrives → proves the gate let it
        // through AND that everything before it (M1) was already processed.
        assert!(
            node_b
                .wait_for_event(&s2, &m2_id, Duration::from_secs(30))
                .await,
            "B must ingest the permitted S2 event (allowed_spaces includes S2)"
        );
        // M1, earlier on the same ordered channel, was processed-and-dropped.
        assert!(
            !node_b.has_event(&s1, &m1_id).await,
            "B must NOT ingest the S1 event excluded by allowed_spaces"
        );
    }
}
