// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Phase 9 in-process Node harness — Federation Event Propagation deployment
//! integration tests (task file `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3
//! Commit 3, J-093 design intent).
//!
//! [`spawn_in_process_node`] starts a Node as a `tokio::spawn`-ed accept loop
//! that mirrors [`crate::app::run_node`]'s post-config body. Skipped: keypair
//! file load, startup banner, tracing-subscriber init, replay-from-disk,
//! reconnect scheduler, state-writer task, pending-buffer timeout sweep,
//! named-pipe server, Tauri, Ctrl+C handler. Kept: runtime construction,
//! shared-state wiring, `Server::bind` on `127.0.0.1:0`, accept loop driving
//! [`crate::app::handle_connection`].
//!
//! The harness is the test-side counterpart to `run_node` — a contributor
//! reading both can confirm the production path is exercised end-to-end
//! rather than mocked.
//!
//! [`federate`] uses the production [`crate::reconnect::attempt_reconnect`]
//! helper to drive A→B from one harness Node to another. Identity replication
//! is the test's responsibility (call [`InProcessNode::register_identity`] on
//! every Node that must verify a sender's signature) — cross-Node Identity
//! replication is its own subsystem and is not exercised by Phase 9 baseline
//! scenarios.

#![cfg(test)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use tempfile::TempDir;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

use crate::{
    crypto::encoding,
    federation::registry::FederationRegistry,
    identity::{keypair, registry::IdentityRecord},
    node::runtime::{DispatchOutcome, EventOrigin, NodeRuntime},
    transport::server::Server,
    wire::types::Event,
};
use crate::app::handle_connection;
use crate::fanout::{ClientSenders, FederationPeerSenders};
use crate::federation_session::apply_federation_push;
use crate::reconnect::attempt_reconnect;

/// Handle to an in-process Node. The accept loop runs in a `tokio::spawn`ed
/// task; `shutdown_tx` triggers a clean shutdown via the same `watch`-channel
/// pattern that gates `run_node`'s pipe server (J-071 lesson — the sender
/// must outlive the receiver's `.changed()` await).
///
/// All Arc-shared fields mirror the names from `run_node`'s post-config body
/// so tests can lock+inspect runtime state, registry state, and peer-sender
/// state directly — same surfaces the production state-writer task reads.
pub struct InProcessNode {
    pub node_id: String,
    pub endpoint: String,
    pub keypair: Arc<SigningKey>,
    pub runtime: Arc<Mutex<NodeRuntime>>,
    pub client_senders: ClientSenders,
    pub federation_peer_senders: FederationPeerSenders,
    pub federation_registry: Arc<Mutex<FederationRegistry>>,
    pub federation_registry_path: PathBuf,
    pub spaces_dir: PathBuf,
    pub identities_path: PathBuf,
    /// Per-Node temp directory. Held on the handle so it survives until
    /// shutdown; dropping it removes the on-disk Spaces directory and
    /// identities DB.
    pub data_dir: TempDir,
    shutdown_tx: watch::Sender<bool>,
    accept_handle: Option<JoinHandle<()>>,
}

impl InProcessNode {
    /// Trigger a clean shutdown and wait for the accept loop to exit.
    pub async fn shutdown(mut self) {
        let _ = self.shutdown_tx.send(true);
        if let Some(handle) = self.accept_handle.take() {
            // The accept loop's `tokio::select!` reads the shutdown receiver
            // and breaks; the join then completes. Bound the wait so a
            // hung listener doesn't hang the whole test.
            let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
        }
    }

    /// Register an Identity record on this Node. Idempotent — the test
    /// helper [`make_identity_record`] always builds the same record for a
    /// given key, and `IdentityRegistry::insert` updates rather than
    /// errors on second call.
    pub async fn register_identity(&self, key: &SigningKey) {
        let record = make_identity_record(key, &self.node_id);
        let mut rt = self.runtime.lock().await;
        rt.register_identity(record).expect("register identity");
    }

    /// Inject an event as if a local client submitted it via the production
    /// client path. Mirrors `process_inbound`'s Stage-5/Stage-6 surface: on
    /// `DispatchOutcome::Accepted`, calls `apply_federation_push` with
    /// `EventOrigin::LocallySubmitted` so federation peers receive E
    /// through the same `FederationPeerSenders` path the real client path
    /// uses. `apply_fanout` is skipped because the harness doesn't connect
    /// local clients — Scenario 1/2's assertions read from the federation
    /// path, not the local-fan-out path.
    pub async fn submit_locally(&self, ev: Event) -> DispatchOutcome {
        let outcome = {
            let mut rt = self.runtime.lock().await;
            rt.dispatch_event(ev.clone(), EventOrigin::LocallySubmitted, None)
        };
        if matches!(outcome, DispatchOutcome::Accepted { .. }) {
            apply_federation_push(
                &ev,
                EventOrigin::LocallySubmitted,
                &self.runtime,
                &self.federation_peer_senders,
            )
            .await;
        }
        outcome
    }

    /// Ingest an event directly into the runtime — bypasses `dispatch_event`
    /// and is the pre-federation setup primitive. Same shape the existing
    /// `federation_relationship_integration::build_node_with_alice` helper
    /// uses for pre-seeding Space + Room + membership state before driving
    /// federation traffic.
    pub async fn ingest(&self, ev: Event) {
        let mut rt = self.runtime.lock().await;
        rt.ingest_event(ev);
    }

    /// Snapshot the Space's current tips. Used to construct subsequent
    /// events with valid `prev_events`.
    pub async fn dag_tips(&self, space_id: &str) -> Vec<String> {
        let rt = self.runtime.lock().await;
        rt.dag_tips(space_id)
    }

    /// True if the event is in the Space's store. The federation-arrival
    /// honesty assertion target (Scenarios 1, 2, 3).
    pub async fn has_event(&self, space_id: &str, event_id: &str) -> bool {
        let rt = self.runtime.lock().await;
        rt.stores
            .get(space_id)
            .map(|s| s.contains(event_id))
            .unwrap_or(false)
    }

    /// True if the Space exists locally.
    pub async fn has_space(&self, space_id: &str) -> bool {
        let rt = self.runtime.lock().await;
        rt.spaces.contains_key(space_id)
    }

    /// True if this Node has an active federation session registered to
    /// `peer_node_id` (i.e., its `FederationPeerSenders` map contains the
    /// peer). The R12 register-on-handshake hook fires inside
    /// `run_federation_session_post_handshake`; absence here means the
    /// handshake hasn't reached ACTIVE yet.
    pub async fn has_federation_peer(&self, peer_node_id: &str) -> bool {
        let senders = self.federation_peer_senders.lock().await;
        senders.contains_key(peer_node_id)
    }

    /// Poll `has_event` until true or timeout. Default poll interval 25 ms.
    pub async fn wait_for_event(
        &self,
        space_id: &str,
        event_id: &str,
        timeout: Duration,
    ) -> bool {
        wait_until(timeout, || async {
            self.has_event(space_id, event_id).await
        })
        .await
    }

    /// Poll `has_space` until true or timeout.
    pub async fn wait_for_space(&self, space_id: &str, timeout: Duration) -> bool {
        wait_until(timeout, || async { self.has_space(space_id).await }).await
    }

    /// Poll `has_federation_peer` until true or timeout.
    pub async fn wait_for_federation_peer(
        &self,
        peer_node_id: &str,
        timeout: Duration,
    ) -> bool {
        wait_until(timeout, || async {
            self.has_federation_peer(peer_node_id).await
        })
        .await
    }
}

/// Spawn an in-process Node and return its handle once the WebSocket
/// listener is bound. The accept loop runs in the background and lives
/// until [`InProcessNode::shutdown`] is called.
///
/// Mirrors the post-config body of [`crate::app::run_node`] for everything
/// Phase 9 scenarios actually exercise (runtime construction, shared-state
/// wiring, accept loop). Production-only concerns deliberately skipped:
/// keypair file persistence (in-memory keypair only), startup banner,
/// tracing subscriber init (tests install their own via `#[traced_test]`),
/// disk replay (no pre-existing Spaces), reconnect scheduler (the
/// `federate` helper drives reconnect-shape paths directly), state-writer
/// task (tests poll runtime state synchronously), pending-buffer timeout
/// sweep (scenarios complete inside default-timeout windows), named-pipe
/// server, Tauri shell, Ctrl+C handler.
pub async fn spawn_in_process_node() -> InProcessNode {
    let data_dir = tempfile::tempdir().expect("create per-Node tempdir");
    let spaces_dir = data_dir.path().join("spaces");
    std::fs::create_dir_all(&spaces_dir).expect("create spaces dir");
    let identities_path = data_dir.path().join("xgen-node_identities.db");
    let federation_registry_path = data_dir.path().join("xgen-node_federation.json");

    let signing_key = keypair::generate();
    let node_id = pubkey_uri(&signing_key);
    let runtime = NodeRuntime::new(signing_key.clone());
    let runtime = Arc::new(Mutex::new(runtime));

    let client_senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
    let federation_peer_senders: FederationPeerSenders =
        Arc::new(Mutex::new(HashMap::new()));
    let federation_registry: Arc<Mutex<FederationRegistry>> =
        Arc::new(Mutex::new(FederationRegistry::new()));

    let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
        .await
        .expect("bind on 127.0.0.1:0");
    let local_addr = server.local_addr();
    let endpoint = format!("ws://{}/", local_addr);

    let node_keypair = Arc::new(signing_key);
    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

    let accept_runtime = Arc::clone(&runtime);
    let accept_client_senders = client_senders.clone();
    let accept_federation_peer_senders = federation_peer_senders.clone();
    let accept_federation_registry = Arc::clone(&federation_registry);
    let accept_federation_registry_path = federation_registry_path.clone();
    let accept_node_keypair = Arc::clone(&node_keypair);
    let accept_home_node_id = node_id.clone();
    let accept_identities_path = identities_path.clone();
    let accept_spaces_dir = spaces_dir.clone();
    let local_mode = true; // tests run with local-mode semantics (signature checks active, no remote-bootstrap quirks)
    let sync_batch_size: usize = 1000; // production default per `[sync].batch_size`

    let accept_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                changed = shutdown_rx.changed() => {
                    if changed.is_err() || *shutdown_rx.borrow() {
                        break;
                    }
                }
                result = server.accept() => {
                    match result {
                        Ok(conn) => {
                            let rt = Arc::clone(&accept_runtime);
                            // No `connections` tracker in the harness — production
                            // uses it for the state-writer task surface and admin
                            // ops; tests don't read it.
                            let conns = Arc::new(Mutex::new(Vec::new()));
                            let senders = accept_client_senders.clone();
                            let fed_senders = accept_federation_peer_senders.clone();
                            let fed_reg = Arc::clone(&accept_federation_registry);
                            let fed_reg_path = accept_federation_registry_path.clone();
                            let kp = Arc::clone(&accept_node_keypair);
                            let home = accept_home_node_id.clone();
                            let ids = accept_identities_path.clone();
                            let sdir = accept_spaces_dir.clone();
                            tokio::spawn(async move {
                                handle_connection(
                                    conn, rt, conns, senders, fed_senders, fed_reg,
                                    fed_reg_path, kp, home, local_mode, ids, sdir,
                                    sync_batch_size,
                                ).await;
                            });
                        }
                        Err(_) => {
                            // Bind closed (listener dropped) or accept failed
                            // — either way, no more inbound connections will
                            // arrive. Exit the loop.
                            break;
                        }
                    }
                }
            }
        }
    });

    InProcessNode {
        node_id,
        endpoint,
        keypair: node_keypair,
        runtime,
        client_senders,
        federation_peer_senders,
        federation_registry,
        federation_registry_path,
        spaces_dir,
        identities_path,
        data_dir,
        shutdown_tx,
        accept_handle: Some(accept_handle),
    }
}

/// Drive a federation handshake + post-handshake session from `initiator`
/// to `receiver`. After this returns, both Nodes have:
/// - the other's entry in `FederationPeerSenders` (R12 register hook)
/// - the bilateral delta exchange complete for each Space in `shared_spaces`
///   (initiator's `state.federation_add` events ingested on the receiver
///   via the F-1a tip-exchange path; receiver's reciprocal `state.federation_add`
///   sent back through the same mechanism)
/// - the F-2 long-lived session active and ready for [`InProcessNode::submit_locally`]
///   on either side to push events to the other
///
/// Calls the production [`attempt_reconnect`] under the hood; the only
/// difference from production is that the harness pre-populates the
/// initiator's registry with a "lost" record for the receiver so
/// `attempt_reconnect`'s sanity check (`session.peer_node_id ==
/// peer_node_id`) is satisfied.
pub async fn federate(
    initiator: &InProcessNode,
    receiver: &InProcessNode,
    shared_spaces: Vec<String>,
) {
    use crate::federation::registry::FederationRelationship;

    {
        let mut reg = initiator.federation_registry.lock().await;
        reg.upsert(FederationRelationship {
            peer_node_id: receiver.node_id.clone(),
            shared_spaces: shared_spaces.clone(),
            negotiated_version: "0.1".to_string(),
            negotiated_serialisation: "json".to_string(),
            session_id: "xgen://hash/sha256:phase9-harness-session".to_string(),
            last_connected: now_rfc(),
            peer_url: Some(receiver.endpoint.clone()),
        });
        reg.mark_lost(&receiver.node_id, Utc::now() - chrono::Duration::minutes(20));
    }

    let attempt_cursor = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn(attempt_reconnect(
        Arc::clone(&initiator.runtime),
        initiator.client_senders.clone(),
        Arc::clone(&initiator.federation_peer_senders),
        Arc::clone(&initiator.federation_registry),
        initiator.federation_registry_path.clone(),
        Arc::clone(&initiator.keypair),
        initiator.node_id.clone(),
        initiator.spaces_dir.clone(),
        initiator.identities_path.clone(),
        true, // local_mode
        initiator.endpoint.clone(),
        receiver.node_id.clone(),
        receiver.endpoint.clone(),
        shared_spaces,
        attempt_cursor,
    ));

    // Wait for the R12 register hook to fire on both sides.
    assert!(
        initiator
            .wait_for_federation_peer(&receiver.node_id, Duration::from_secs(10))
            .await,
        "initiator did not register receiver in FederationPeerSenders within 10s"
    );
    assert!(
        receiver
            .wait_for_federation_peer(&initiator.node_id, Duration::from_secs(10))
            .await,
        "receiver did not register initiator in FederationPeerSenders within 10s"
    );
}

// ── Small helpers ────────────────────────────────────────────────────────────

/// RFC 3339 timestamp with millisecond precision. Same shape every
/// federation/identity test uses.
pub fn now_rfc() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// `xgen://pubkey/ed25519:<base64url>` URI for the verifying key. Identity
/// IDs and Node IDs share this URI shape (the latter is derived from the
/// Node's keypair the same way).
pub fn pubkey_uri(key: &SigningKey) -> String {
    format!(
        "xgen://pubkey/ed25519:{}",
        encoding::encode(key.verifying_key().as_bytes())
    )
}

/// Build the minimal IdentityRecord shape every cross-Node test uses. The
/// `home_node` field is set to the Node hosting this Identity per the spec.
pub fn make_identity_record(key: &SigningKey, home_node: &str) -> IdentityRecord {
    IdentityRecord {
        identity_id: pubkey_uri(key),
        display_name: None,
        is_ai: false,
        ai_capabilities: None,
        registered_at: "2026-05-21T00:00:00.000Z".to_string(),
        trust_assertion: None,
        devices: vec![],
        home_node: home_node.to_string(),
        update_version: 0,
    }
}

/// Poll until `condition` returns true or `timeout` elapses. Returns the
/// final value. 25 ms poll interval matches the existing
/// `reconnect_integration::run_reconnect_scenario_initiator_to_receiver`
/// pattern.
async fn wait_until<F, Fut>(timeout: Duration, mut condition: F) -> bool
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    loop {
        if condition().await {
            return true;
        }
        if start.elapsed() >= timeout {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}
