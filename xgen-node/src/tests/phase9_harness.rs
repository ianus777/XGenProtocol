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
use crate::app::{handle_connection, persist_event, replay_spaces_from_dir};
use crate::fanout::{ClientSenders, FederationPeerSenders};
use crate::federation_session::apply_federation_push;
use crate::identity::registry::IdentityRegistry;
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
    /// In-flight per-connection task handles. Pushed by the accept loop
    /// every time a new `handle_connection` task spawns; drained and
    /// aborted by `shutdown` / `shutdown_keep_data` so the WS connection
    /// to a peer closes promptly when the harness Node goes down.
    ///
    /// Phase 7.5 persistence-amendment milestone Commit 3 sub-task 3.1
    /// refinement-fold (Joe-lock Option α at Clair's verification-time
    /// surface). Closes a backward-coherence audit gap that pre-existed in
    /// `phase9_drop_and_recover.rs` docstring lines 17-19 ("abort in-flight
    /// connection tasks → A's WS to B closes") which the runbook §3.2 +
    /// checkpoint #5 locks both missed at authoring time. Symmetric
    /// application to both shutdown methods per D-067 no-drift-surface
    /// posture — asymmetric abort would be the "silent feature divergence"
    /// pattern D-077's backward-coherence question catches.
    connection_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

/// Saved per-Node state preserved across a `shutdown_keep_data` / respawn
/// cycle. Returned by [`InProcessNode::shutdown_keep_data`] and consumed
/// by [`spawn_in_process_node_with_state`].
///
/// The load-bearing field is `data_dir: TempDir` — preserving the temp
/// directory across the drop+respawn means on-disk state (Space event
/// files, identities DB, federation registry JSON) survives the cycle so
/// the respawn can replay it. A naïve fresh `tempfile::tempdir()` in the
/// respawn path would lose the data the drop-and-recover scenario is
/// designed to verify.
///
/// The `keypair` and `node_id` carry across so the respawned Node keeps
/// the same wire identity — a fresh keypair would surface as a different
/// peer to A and federate() would establish a new relationship rather
/// than recover the existing one.
///
/// The three path fields (`identities_path`, `federation_registry_path`,
/// `spaces_dir`) are clones of the working tree's `TempDir`-rooted paths;
/// they let `spawn_in_process_node_with_state` call the production replay
/// helpers (`IdentityRegistry::load`, `FederationRegistry::load`,
/// `replay_spaces_from_dir`) without re-deriving the join() chain.
pub struct SavedNodeState {
    pub node_id: String,
    pub keypair: Arc<SigningKey>,
    pub data_dir: TempDir,
    pub identities_path: PathBuf,
    pub federation_registry_path: PathBuf,
    pub spaces_dir: PathBuf,
}

impl InProcessNode {
    /// Trigger a clean shutdown and wait for the accept loop to exit.
    /// Aborts all in-flight per-connection handle_connection tasks first
    /// so WS peers see the close promptly (sibling-shape to
    /// [`InProcessNode::shutdown_keep_data`] per D-067 no-drift-surface
    /// symmetric-application discipline — Commit 3 refinement-fold).
    pub async fn shutdown(mut self) {
        // Abort in-flight per-connection tasks BEFORE the accept-loop
        // shutdown signal — same ordering shutdown_keep_data uses so
        // both methods give peer Nodes the same drop-detection latency.
        let handles: Vec<JoinHandle<()>> =
            std::mem::take(&mut *self.connection_handles.lock().await);
        for h in &handles {
            h.abort();
        }
        let _ = self.shutdown_tx.send(true);
        if let Some(handle) = self.accept_handle.take() {
            // The accept loop's `tokio::select!` reads the shutdown receiver
            // and breaks; the join then completes. Bound the wait so a
            // hung listener doesn't hang the whole test.
            let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
        }
    }

    /// Sibling to [`InProcessNode::shutdown`] that preserves on-disk state
    /// for a respawn via [`spawn_in_process_node_with_state`]. Triggers the
    /// same clean shutdown (`shutdown_tx.send(true)` + accept-loop exit +
    /// in-flight task abort via Drop on the JoinHandle) but moves the
    /// `data_dir: TempDir` out of `self` into the returned `SavedNodeState`
    /// rather than dropping it (which would delete the on-disk Spaces
    /// directory, identities DB, and federation registry JSON).
    ///
    /// Phase 7.5 persistence-amendment milestone Commit 3 (sentinel-tree
    /// refinement). Sub-task 3.1 per Joe-lock checkpoint #5 — six locked
    /// fields surface to the caller; see [`SavedNodeState`] doc-comment for
    /// the field-by-field reasoning.
    ///
    /// The "drop" semantic in Phase 9 Scenario 3 is best-effort — A's WS
    /// to B closes when the accept loop exits and the in-flight connection
    /// task's tokio context is aborted. A's session task observes the close
    /// via WS keepalive (within ~30s budget) and runs its R12 deregister
    /// cleanup. This is NOT "process crashed mid-write" failure-mode
    /// fidelity — partially-flushed `xgen-node_federation.json`,
    /// partially-committed identity DB transactions, and OS-level fd
    /// recovery are `federation-stress` follow-on territory per
    /// phase9_drop_and_recover.rs §"Two known coverage gaps" #1.
    pub async fn shutdown_keep_data(mut self) -> SavedNodeState {
        // Abort in-flight per-connection tasks so A's WS to this Node
        // closes promptly — phase9_drop_and_recover.rs §"Honesty
        // assertion #1" depends on A observing the drop and running R12
        // deregister within the test's 30s budget. Commit 3 refinement-fold
        // (Joe-lock Option α) closes the pre-existing docstring contract.
        let handles: Vec<JoinHandle<()>> =
            std::mem::take(&mut *self.connection_handles.lock().await);
        for h in &handles {
            h.abort();
        }
        let _ = self.shutdown_tx.send(true);
        if let Some(handle) = self.accept_handle.take() {
            let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
        }
        SavedNodeState {
            node_id: self.node_id,
            keypair: self.keypair,
            data_dir: self.data_dir,
            identities_path: self.identities_path,
            federation_registry_path: self.federation_registry_path,
            spaces_dir: self.spaces_dir,
        }
    }

    /// Register an Identity record on this Node. Idempotent — the test
    /// helper [`make_identity_record`] always builds the same record for a
    /// given key, and `IdentityRegistry::insert` updates rather than
    /// errors on second call.
    ///
    /// Phase 7.5 persistence-amendment milestone Commit 3 refinement-fold
    /// (Joe-lock Option α-1 sweep #1): also persists the registry to
    /// `self.identities_path` so `spawn_in_process_node_with_state` can
    /// replay it. Sibling-shape to production `accept_registration` at
    /// `xgen-node/src/app.rs:1628` which calls `save()` after `register()`.
    /// Closes the pre-existing docstring contract at phase9_drop_and_recover.rs
    /// lines 20-23 ("replays identity registry from disk") that the harness's
    /// in-memory-only original was silently violating.
    pub async fn register_identity(&self, key: &SigningKey) {
        let record = make_identity_record(key, &self.node_id);
        let mut rt = self.runtime.lock().await;
        rt.register_identity(record).expect("register identity");
        let _ = rt.identity_registry.save(&self.identities_path);
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
        // Phase 7.5 persistence-amendment milestone Commit 3 refinement-fold
        // (Joe-lock Option α-1 sweep #2): on Accepted, persist the
        // submitted event AND iterate Commit 2a's additional_persisted
        // for drain-side-effect events. Sibling-shape to production
        // process_inbound at xgen-node/src/app.rs:~1505 (post-Commit-2a).
        // Without this, respawn-replay would miss locally-submitted events
        // and the docstring's "Space event stores from disk" promise at
        // phase9_drop_and_recover.rs lines 20-23 is silently violated.
        if let DispatchOutcome::Accepted {
            new_joiner: _,
            ref additional_persisted,
        } = outcome
        {
            let event_space = if ev.space_id.is_empty() {
                ev.event_id.clone().unwrap_or_default()
            } else {
                ev.space_id.clone()
            };
            if !event_space.is_empty() {
                persist_event(&self.spaces_dir, &event_space, &ev);
            }
            for drained in additional_persisted {
                let drained_space = if drained.space_id.is_empty() {
                    drained.event_id.clone().unwrap_or_default()
                } else {
                    drained.space_id.clone()
                };
                if !drained_space.is_empty() {
                    persist_event(&self.spaces_dir, &drained_space, drained);
                }
            }
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
        // Phase 7.5 persistence-amendment milestone Commit 3 refinement-fold
        // (Joe-lock Option α-1 sweep #2): persist the ingested event so
        // `spawn_in_process_node_with_state` can replay it via
        // `replay_spaces_from_dir`. Bypasses dispatch_event so no
        // additional_persisted to iterate (drains are dispatch_event
        // surfaces, not ingest_event surfaces). Sibling-shape to production's
        // direct-ingest sites within process_inbound's surrounding flow.
        let event_space = if ev.space_id.is_empty() {
            ev.event_id.clone().unwrap_or_default()
        } else {
            ev.space_id.clone()
        };
        {
            let mut rt = self.runtime.lock().await;
            rt.ingest_event(ev.clone());
        }
        if !event_space.is_empty() {
            persist_event(&self.spaces_dir, &event_space, &ev);
        }
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
    let connection_handles: Arc<Mutex<Vec<JoinHandle<()>>>> =
        Arc::new(Mutex::new(Vec::new()));

    let accept_runtime = Arc::clone(&runtime);
    let accept_client_senders = client_senders.clone();
    let accept_federation_peer_senders = federation_peer_senders.clone();
    let accept_federation_registry = Arc::clone(&federation_registry);
    let accept_federation_registry_path = federation_registry_path.clone();
    let accept_node_keypair = Arc::clone(&node_keypair);
    let accept_home_node_id = node_id.clone();
    let accept_identities_path = identities_path.clone();
    let accept_spaces_dir = spaces_dir.clone();
    let accept_connection_handles = Arc::clone(&connection_handles);
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
                            let h = tokio::spawn(async move {
                                handle_connection(
                                    conn, rt, conns, senders, fed_senders, fed_reg,
                                    fed_reg_path, kp, home, local_mode, ids, sdir,
                                    sync_batch_size,
                                ).await;
                            });
                            // Track for shutdown_keep_data / shutdown abort
                            // (Commit 3 refinement-fold — closes pre-existing
                            // docstring contract at phase9_drop_and_recover.rs).
                            accept_connection_handles.lock().await.push(h);
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
        connection_handles,
    }
}

/// Phase 7.5 persistence-amendment milestone Commit 3 sub-task 3.2 —
/// sibling to [`spawn_in_process_node`] with a state-replay path. Used by
/// the Phase 9 Scenario 3 drop-and-recover sentinel test
/// (`phase9_drop_and_recover.rs`) to bring a Node back up after a
/// [`InProcessNode::shutdown_keep_data`] cycle without losing on-disk
/// state.
///
/// Replay-path tweaks from `spawn_in_process_node`:
///   - Reuse `saved.keypair` + `saved.node_id` (don't generate fresh)
///   - Reuse `saved.data_dir` + the three paths (don't allocate fresh `TempDir`)
///   - Load `IdentityRegistry` from `saved.identities_path` if the file exists
///   - Load `FederationRegistry` from `saved.federation_registry_path` if the file exists
///   - Replay Space event stores via `replay_spaces_from_dir(&mut runtime,
///     &saved.spaces_dir)` — the Q1(a).ii sort-on-replay defensive layer
///     already shipped at Commit 2 handles any DAG-ordering surprises
///     introduced by on-disk store-iteration order
///   - Bind fresh `127.0.0.1:0` per Pre-Commit-3b-1 Joe-lock Q1 option (b)
///     (the previous bind port is deliberately not reused — restart-in-place
///     TIME_WAIT semantics are `federation-stress` follow-on territory per
///     phase9_drop_and_recover.rs §"Two known coverage gaps" #2)
///
/// All other spawn machinery (accept loop, shutdown_tx watch channel,
/// accept_handle JoinHandle, clones into the `tokio::spawn` closure,
/// `local_mode` + `sync_batch_size` defaults) is verbatim from
/// `spawn_in_process_node`. The duplication is deliberate per Joe-lock
/// checkpoint #5 — factoring out a shared helper would itself be a
/// structural divergence beyond the locked state-replay additions.
pub async fn spawn_in_process_node_with_state(saved: SavedNodeState) -> InProcessNode {
    let data_dir = saved.data_dir;
    let spaces_dir = saved.spaces_dir;
    let identities_path = saved.identities_path;
    let federation_registry_path = saved.federation_registry_path;
    let node_id = saved.node_id;
    let node_keypair = saved.keypair;

    // Reconstruct the runtime around the same keypair so the respawned
    // Node keeps its wire identity. Load IdentityRegistry if the file
    // exists (production app.rs:344 mirror — silent fallback to empty
    // registry on load error so a fresh-tempdir respawn still works).
    let mut runtime = NodeRuntime::new((*node_keypair).clone());
    if identities_path.exists() {
        if let Ok(reg) = IdentityRegistry::load(&identities_path) {
            runtime.identity_registry = reg;
        }
    }
    // Replay Space event stores via Commit 2's Q1(a).ii-bearing helper.
    let _replayed = replay_spaces_from_dir(&mut runtime, &spaces_dir);
    let runtime = Arc::new(Mutex::new(runtime));

    let client_senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
    let federation_peer_senders: FederationPeerSenders =
        Arc::new(Mutex::new(HashMap::new()));
    let federation_registry: Arc<Mutex<FederationRegistry>> = {
        let loaded = if federation_registry_path.exists() {
            FederationRegistry::load(&federation_registry_path).ok()
        } else {
            None
        };
        Arc::new(Mutex::new(loaded.unwrap_or_else(FederationRegistry::new)))
    };

    let mut server = Server::bind("127.0.0.1:0".parse().unwrap())
        .await
        .expect("bind on 127.0.0.1:0 (respawn fresh port per Q1(b))");
    let local_addr = server.local_addr();
    let endpoint = format!("ws://{}/", local_addr);

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    let connection_handles: Arc<Mutex<Vec<JoinHandle<()>>>> =
        Arc::new(Mutex::new(Vec::new()));

    let accept_runtime = Arc::clone(&runtime);
    let accept_client_senders = client_senders.clone();
    let accept_federation_peer_senders = federation_peer_senders.clone();
    let accept_federation_registry = Arc::clone(&federation_registry);
    let accept_federation_registry_path = federation_registry_path.clone();
    let accept_node_keypair = Arc::clone(&node_keypair);
    let accept_home_node_id = node_id.clone();
    let accept_identities_path = identities_path.clone();
    let accept_spaces_dir = spaces_dir.clone();
    let accept_connection_handles = Arc::clone(&connection_handles);
    let local_mode = true;
    let sync_batch_size: usize = 1000;

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
                            let conns = Arc::new(Mutex::new(Vec::new()));
                            let senders = accept_client_senders.clone();
                            let fed_senders = accept_federation_peer_senders.clone();
                            let fed_reg = Arc::clone(&accept_federation_registry);
                            let fed_reg_path = accept_federation_registry_path.clone();
                            let kp = Arc::clone(&accept_node_keypair);
                            let home = accept_home_node_id.clone();
                            let ids = accept_identities_path.clone();
                            let sdir = accept_spaces_dir.clone();
                            let h = tokio::spawn(async move {
                                handle_connection(
                                    conn, rt, conns, senders, fed_senders, fed_reg,
                                    fed_reg_path, kp, home, local_mode, ids, sdir,
                                    sync_batch_size,
                                ).await;
                            });
                            accept_connection_handles.lock().await.push(h);
                        }
                        Err(_) => {
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
        connection_handles,
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
