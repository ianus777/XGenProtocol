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
use std::sync::{Arc, Mutex as StdMutex, OnceLock, Weak};
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use tempfile::TempDir;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;
use tracing_subscriber::layer::SubscriberExt;

use crate::{
    crypto::encoding,
    federation::registry::FederationRegistry,
    identity::{keypair, registry::IdentityRecord},
    node::runtime::{DispatchOutcome, EventOrigin, NodeRuntime},
    space::state::SpaceState,
    transport::server::Server,
    wire::types::Event,
};
use crate::app::{handle_connection, persist_event, replay_spaces_from_dir};
use crate::fanout::{ClientSenders, FederationPeerSenders};
use crate::federation_session::apply_federation_push;
use crate::identity::registry::IdentityRegistry;
use crate::reconnect::attempt_reconnect;
use xgen_common::xgid::{EventXgid, IdentityXgid, NodeXgid, RoomXgid, SpaceXgid, Xgid};

// Pass 3 Commit 2a test-fixture helpers: cheap typed-XGID constructors for
// projecting String slots at the test→production boundary. Mechanical sweep
// only — no semantic changes. `idx`/`ndx`/`sdx`/`edx`/`rdx` correspond
// to IdentityXgid/NodeXgid/SpaceXgid/EventXgid/RoomXgid.
pub(crate) fn idx(s: &str) -> IdentityXgid {
    IdentityXgid::from_xgid(Xgid::new(s.to_string()))
}
pub(crate) fn ndx(s: &str) -> NodeXgid {
    NodeXgid::from_xgid(Xgid::new(s.to_string()))
}
pub(crate) fn sdx(s: &str) -> SpaceXgid {
    SpaceXgid::from_xgid(Xgid::new(s.to_string()))
}
pub(crate) fn edx(s: &str) -> EventXgid {
    EventXgid::from_xgid(Xgid::new(s.to_string()))
}
#[allow(dead_code)]
pub(crate) fn rdx(s: &str) -> RoomXgid {
    RoomXgid::from_xgid(Xgid::new(s.to_string()))
}
/// Project an Event's event_id to a String for test storage / logging.
pub(crate) fn event_id_str(ev: &Event) -> String {
    ev.event_id
        .as_ref()
        .expect("event must have event_id")
        .as_str()
        .to_string()
}

// ───────────────────────────────────────────────────────────────────────────
// Phase 9 Commit 3b-3-pre — push-attempt counter + tracing Layer
// ───────────────────────────────────────────────────────────────────────────
//
// `push_attempts` records every successful `federation_push_sent` trace event
// emitted by `apply_federation_push` in production code. The counter is
// per-`InProcessNode`, fed by a global tracing-subscriber Layer that reads
// the new `local_node_id` field on the trace event (Commit 3b-3-pre G2
// retrofit) and routes the increment to the matched Node's counter via a
// process-global registry of `Weak<...>` references.
//
// **Why a Layer and not direct instrumentation.** `apply_federation_push` is
// called from four production sites (`xgen-node/src/app.rs:928, 1207, 1342`
// + `phase9_harness::InProcessNode::submit_locally:266`). Only the last goes
// through a harness wrapper; the production sites are reached via
// `handle_connection`'s spawned receive tasks. To count B's outbound push
// attempts (the F-5 regression lock Phase 9 Compound C2 needs), the
// instrumentation must observe ALL four call sites without modifying
// production code. The tracing Layer observes the existing G2 trace events,
// satisfying Q2 (test instrumentation stays in the test layer) per the
// Pre-Commit-3b-3 Joe-lock.
//
// **Composition with `#[traced_test]`.** `tracing-test 0.2.6`'s macro uses
// `tracing::dispatcher::set_global_default` via a shared `INITIALIZED: Once`
// in the tracing-test crate (see `tracing-test-macro-0.2.6/src/lib.rs:67-86`).
// `set_global_default` can be called at most once per process — whoever
// wins (first test to fire) installs the global. Tests using `#[traced_test]`
// run first in workspace mode and own the global.
//
// Our harness counter therefore CANNOT share the global default mechanism.
// Instead, tests that read the counter install a per-test/per-thread
// subscriber via [`install_harness_subscriber`], which returns a
// `HarnessSubscriberGuard` that holds the per-thread default for the test
// body's lifetime. The per-thread default takes precedence over the global
// default on that thread. Tests using `#[traced_test]` continue to use
// tracing-test's global default unchanged; tests that read the counter
// (Compound C2, counter unit tests) install our subscriber explicitly and
// must NOT use `#[traced_test]` (the macros are mutually exclusive on
// `current_thread` runtime — they target the same thread's default slot).
//
// **Sibling log-buffer Layer.** For tests that need substring assertions on
// the captured trace stream without `#[traced_test]`, `LogBufferLayer`
// records each event as a rendered line into a global buffer. The C2 test
// uses `harness_logs_contain("federation_push_skipped_origin")` as the
// positive F-5 trace assertion sibling-shape to Scenario 2.

type CountMap = HashMap<(String, String), u64>;
type Counter = Arc<StdMutex<CountMap>>;

static REGISTRY: OnceLock<StdMutex<HashMap<String, Weak<StdMutex<CountMap>>>>> = OnceLock::new();
static LOG_BUFFER: OnceLock<StdMutex<Vec<String>>> = OnceLock::new();

fn registry() -> &'static StdMutex<HashMap<String, Weak<StdMutex<CountMap>>>> {
    REGISTRY.get_or_init(|| StdMutex::new(HashMap::new()))
}

fn log_buffer() -> &'static StdMutex<Vec<String>> {
    LOG_BUFFER.get_or_init(|| StdMutex::new(Vec::new()))
}

/// Clear the global trace-line buffer maintained by [`LogBufferLayer`].
/// Tests that rely on [`harness_logs_contain`] should call this at the
/// start of each test (or rely on `#[serial_test::serial]` to serialise
/// + call once at the start) so accumulated lines from prior tests do
///   not leak into the assertion. Sibling-shape to how `#[traced_test]`
///   scopes its capture per-test (each per-test subscriber starts fresh).
pub fn harness_clear_logs() {
    log_buffer().lock().unwrap().clear();
}

/// Substring search over the global trace-line buffer maintained by
/// [`LogBufferLayer`]. Sibling-shape to `tracing_test::logs_contain` for
/// tests that do NOT use `#[traced_test]` (so they can compose the
/// counter Layer alongside log capture in one global subscriber).
///
/// Returns `true` if any captured trace line contains `needle` as a
/// substring. Lines are rendered as `LEVEL field1=value1 field2=value2 …`
/// per [`LogBufferLayer::on_event`]; substring matches behave the same as
/// `tracing_test::logs_contain` for the field-value content. Test authors
/// should call [`harness_clear_logs`] at test start.
pub fn harness_logs_contain(needle: &str) -> bool {
    log_buffer().lock().unwrap().iter().any(|line| line.contains(needle))
}

/// Tracing Layer that observes `federation_push_sent` trace events and
/// increments the matched [`InProcessNode`]'s `push_attempts` counter. The
/// match is by the new `local_node_id` field added to the G2 trace event
/// at Commit 3b-3-pre.
struct CounterLayer;

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for CounterLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = FieldExtractor::default();
        event.record(&mut visitor);
        if visitor.event_name.as_deref() != Some("federation_push_sent") {
            return;
        }
        let (local, peer, eid) = match (
            visitor.local_node_id,
            visitor.peer_node_id,
            visitor.event_id,
        ) {
            (Some(l), Some(p), Some(e)) => (l, p, e),
            _ => return,
        };
        let reg = registry().lock().unwrap();
        if let Some(weak) = reg.get(&local) {
            if let Some(counter) = weak.upgrade() {
                let mut c = counter.lock().unwrap();
                *c.entry((eid, peer)).or_insert(0) += 1;
            }
        }
    }
}

/// Tracing Layer that buffers each observed event as a rendered text line.
/// Sibling to [`CounterLayer`]; sits in the same composed subscriber so
/// tests can do log-substring assertions alongside counter inspection
/// without `#[traced_test]`.
struct LogBufferLayer;

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for LogBufferLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = LineRenderer::default();
        event.record(&mut visitor);
        let line = format!("{} {}", event.metadata().level(), visitor.0);
        log_buffer().lock().unwrap().push(line);
    }
}

#[derive(Default)]
struct FieldExtractor {
    event_name: Option<String>,
    local_node_id: Option<String>,
    peer_node_id: Option<String>,
    event_id: Option<String>,
}

impl tracing::field::Visit for FieldExtractor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "event" => self.event_name = Some(value.to_string()),
            "local_node_id" => self.local_node_id = Some(value.to_string()),
            "peer_node_id" => self.peer_node_id = Some(value.to_string()),
            "event_id" => self.event_id = Some(value.to_string()),
            _ => {}
        }
    }
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        // `%foo` formatting in the trace! macro arrives here with the Display
        // representation already rendered into the Debug output. Strip
        // surrounding quotes so the captured value matches what the field's
        // Display impl produced (e.g., a node-id URI is not quoted in
        // human-readable form).
        let raw = format!("{:?}", value);
        let cleaned = raw.trim_matches('"').to_string();
        match field.name() {
            "event" => self.event_name = Some(cleaned),
            "local_node_id" => self.local_node_id = Some(cleaned),
            "peer_node_id" => self.peer_node_id = Some(cleaned),
            "event_id" => self.event_id = Some(cleaned),
            _ => {}
        }
    }
}

#[derive(Default)]
struct LineRenderer(String);

impl tracing::field::Visit for LineRenderer {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0
            .push_str(&format!("{}={} ", field.name(), value));
    }
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .push_str(&format!("{}={:?} ", field.name(), value));
    }
}

/// Guard returned by [`install_harness_subscriber`]. Holds the per-thread
/// tracing default for the duration of its lifetime. Drop it (typically at
/// end of test scope) to remove the per-thread default — at which point
/// future trace events on that thread fall back to the process-global
/// default (typically tracing-test's, if installed by another test).
pub struct HarnessSubscriberGuard {
    _guard: tracing::subscriber::DefaultGuard,
}

/// Install the harness's tracing subscriber on the current thread for the
/// returned guard's lifetime. Composes [`CounterLayer`] +
/// [`LogBufferLayer`] over a `tracing_subscriber::registry()`. Tests that
/// read [`InProcessNode::push_attempt_count`] or call
/// [`harness_logs_contain`] MUST hold a guard for the duration of the test
/// body, OR alternatively MUST NOT use `#[traced_test]` and instead call
/// this function as the first line of the test.
///
/// `set_default` is per-thread, so this guard does NOT conflict with the
/// global default installed by `tracing-test`'s `#[traced_test]` macro
/// (which uses `set_global_default` via a shared `INITIALIZED: Once` per
/// process). The per-thread default takes precedence on this thread; the
/// global default still applies to other threads.
///
/// Returns a [`HarnessSubscriberGuard`]. Bind the result to a name like
/// `_sub` (NOT `_` — the bare underscore drops immediately, defeating the
/// purpose). The naming convention sibling-shape to how `tracing::dispatcher`
/// callers bind their guards.
pub fn install_harness_subscriber() -> HarnessSubscriberGuard {
    let subscriber = tracing_subscriber::registry()
        .with(CounterLayer)
        .with(LogBufferLayer);
    let _guard = tracing::subscriber::set_default(subscriber);
    HarnessSubscriberGuard { _guard }
}

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
    /// federation-admin-control 2a — the pending-request approval queue
    /// (FAC-D1a). Empty + gate-off by default (`spawn_in_process_node`);
    /// `spawn_in_process_node_with_approval` enables the gate so the Commit 3
    /// pause-point test can assert inbound requests land here.
    pub federation_queue: Arc<Mutex<crate::federation::pending_queue::PendingFederationQueue>>,
    /// federation-admin-control 2b — the per-peer policy store (FAC-D3/D4).
    /// Empty by default = permit-all (prime invariant). Commit-2 enforcement
    /// tests `set` a Deny/allowed_spaces policy on this handle and assert the
    /// outbound push / inbound apply is gated end-to-end.
    pub federation_policy:
        Arc<Mutex<crate::federation::federation_policy::FederationPolicyStore>>,
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
    /// Phase 9 Commit 3b-3-pre — push-attempt counter per Joe-lock Q4.
    /// Records every `federation_push_sent` G2 trace event whose
    /// `local_node_id` field matches this Node's `node_id`, keyed by
    /// `(event_id, peer_node_id)`. Fed by the harness's global
    /// [`CounterLayer`] tracing-subscriber Layer via the process-global
    /// registry (lookup by `local_node_id`). Read-only externally via
    /// [`InProcessNode::push_attempt_count`] and [`InProcessNode::push_attempts`].
    ///
    /// Lifetime: stored as `Arc<StdMutex<_>>` so the registry can hold a
    /// `Weak` reference; when the InProcessNode is dropped (on
    /// `shutdown` / `shutdown_keep_data`), the strong ref is released and
    /// the registry's weak ref becomes invalid on the next access (the
    /// CounterLayer skips invalid weaks silently). No explicit
    /// de-registration is required.
    push_attempts: Counter,
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
            let event_space: String = if ev.space_id.as_str().is_empty() {
                ev.event_id
                    .as_ref()
                    .map(|e| e.as_str().to_string())
                    .unwrap_or_default()
            } else {
                ev.space_id.as_str().to_string()
            };
            if !event_space.is_empty() {
                let _ = persist_event(&self.spaces_dir, &event_space, &ev);
            }
            for drained in additional_persisted {
                let drained_space: String = if drained.space_id.as_str().is_empty() {
                    drained
                        .event_id
                        .as_ref()
                        .map(|e| e.as_str().to_string())
                        .unwrap_or_default()
                } else {
                    drained.space_id.as_str().to_string()
                };
                if !drained_space.is_empty() {
                    let _ = persist_event(&self.spaces_dir, &drained_space, drained);
                }
            }
            let local_node_typed = ndx(&self.node_id);
            apply_federation_push(
                &ev,
                EventOrigin::LocallySubmitted,
                &self.runtime,
                &self.federation_peer_senders,
                &local_node_typed,
                Some(&self.federation_policy),
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
        let event_space: String = if ev.space_id.as_str().is_empty() {
            ev.event_id
                .as_ref()
                .map(|e| e.as_str().to_string())
                .unwrap_or_default()
        } else {
            ev.space_id.as_str().to_string()
        };
        {
            let mut rt = self.runtime.lock().await;
            rt.ingest_event(ev.clone());
        }
        if !event_space.is_empty() {
            let _ = persist_event(&self.spaces_dir, &event_space, &ev);
        }
    }

    /// Snapshot the Space's current tips. Used to construct subsequent
    /// events with valid `prev_events`.
    pub async fn dag_tips(&self, space_id: &str) -> Vec<String> {
        let rt = self.runtime.lock().await;
        rt.dag_tips(&sdx(space_id))
    }

    /// True if the event is in the Space's store. The federation-arrival
    /// honesty assertion target (Scenarios 1, 2, 3).
    pub async fn has_event(&self, space_id: &str, event_id: &str) -> bool {
        let rt = self.runtime.lock().await;
        rt.stores
            .get(space_id)
            .map(|s| s.contains(&EventXgid::from_xgid(Xgid::new(event_id.to_string()))))
            .unwrap_or(false)
    }

    /// Clone of an event as the Node stored it (byte-for-byte the accepted
    /// event). Used by the Arc H content-blindness proof to assert the Node
    /// stored the `enc:` blob unchanged.
    pub async fn stored_event(&self, space_id: &str, event_id: &str) -> Option<Event> {
        let rt = self.runtime.lock().await;
        rt.stores.get(space_id).and_then(|s| {
            s.get(&EventXgid::from_xgid(Xgid::new(event_id.to_string())))
                .ok()
                .flatten()
        })
    }

    /// True if the Space exists locally.
    pub async fn has_space(&self, space_id: &str) -> bool {
        let rt = self.runtime.lock().await;
        rt.spaces.contains_key(space_id)
    }

    /// Clone of the Space's resolved `SpaceState`, or `None` if absent.
    /// The M8 convergence-oracle accessor: SpaceState derives `PartialEq`
    /// (C1) so two Nodes' snapshots compare directly. Used by the C3
    /// two-node convergence smoke to assert order-independent agreement.
    pub async fn space_state(&self, space_id: &str) -> Option<SpaceState> {
        let rt = self.runtime.lock().await;
        rt.spaces.get(space_id).cloned()
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

    /// Phase 9 Commit 3b-3-pre — push-attempt counter accessor (Joe-lock Q4).
    /// Returns the number of times this Node has emitted a
    /// `federation_push_sent` trace event for `(event_id, peer_node_id)`.
    /// Returns `0` for never-attempted (the typical case when the F-5 guard
    /// short-circuited the push or when the (event_id, peer) pair was never
    /// considered).
    ///
    /// The counter is fed by the harness's [`CounterLayer`] tracing
    /// subscriber, which fires only when the global subscriber is the
    /// active default — see the module-level "Composition with
    /// `#[traced_test]`" comment for the precise contract. Tests that read
    /// this counter MUST NOT use `#[traced_test]` (which would override the
    /// global default and starve the counter).
    pub fn push_attempt_count(&self, event_id: &str, peer_node_id: &str) -> u64 {
        self.push_attempts
            .lock()
            .unwrap()
            .get(&(event_id.to_string(), peer_node_id.to_string()))
            .copied()
            .unwrap_or(0)
    }

    /// Phase 9 Commit 3b-3-pre — push-attempt counter full snapshot
    /// (Joe-lock Q4). Returns a clone of the internal `(event_id,
    /// peer_node_id) -> count` map at the moment of the call. Useful for
    /// iteration in tests that need to assert across many entries (e.g.
    /// "every Alice event has exactly one push attempt to B and one to C").
    /// Returns a clone rather than a reference because the underlying map
    /// is behind a `Mutex` whose guard cannot outlive the function call.
    pub fn push_attempts(&self) -> HashMap<(String, String), u64> {
        self.push_attempts.lock().unwrap().clone()
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
    spawn_in_process_node_inner(false).await
}

/// Sibling to [`spawn_in_process_node`] with the federation-admin-control 2a
/// FAC-D1 inbound-approval gate enabled (`require_approval = true`). An
/// inbound federation handshake from a not-already-`Active` peer is held in
/// the pending queue + answered with `Reject 2003` instead of
/// auto-establishing. Used by the Commit 3 pause-point test.
pub async fn spawn_in_process_node_with_approval() -> InProcessNode {
    spawn_in_process_node_inner(true).await
}

async fn spawn_in_process_node_inner(require_approval: bool) -> InProcessNode {
    let data_dir = tempfile::tempdir().expect("create per-Node tempdir");
    let spaces_dir = data_dir.path().join("spaces");
    std::fs::create_dir_all(&spaces_dir).expect("create spaces dir");
    let identities_path = data_dir.path().join("xgen-node_identities.db");
    let federation_registry_path = data_dir.path().join("xgen-node_federation.json");
    let federation_queue_path = data_dir.path().join("xgen-node_federation_queue.json");

    let signing_key = keypair::generate();
    let node_id = pubkey_uri(&signing_key);
    let runtime = NodeRuntime::new(signing_key.clone());
    let runtime = Arc::new(Mutex::new(runtime));

    // Phase 9 Commit 3b-3-pre — per-Node push-attempt counter, registered
    // in the process-global REGISTRY keyed by node_id so CounterLayer can
    // route increments to this Node via the `local_node_id` field on
    // observed `federation_push_sent` G2 trace events. Held as Arc on the
    // InProcessNode handle; the registry holds only a Weak reference.
    let push_attempts: Counter = Arc::new(StdMutex::new(HashMap::new()));
    registry()
        .lock()
        .unwrap()
        .insert(node_id.clone(), Arc::downgrade(&push_attempts));

    let client_senders: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
    let federation_peer_senders: FederationPeerSenders =
        Arc::new(Mutex::new(HashMap::new()));
    let federation_registry: Arc<Mutex<FederationRegistry>> =
        Arc::new(Mutex::new(FederationRegistry::new()));
    let federation_queue: Arc<Mutex<crate::federation::pending_queue::PendingFederationQueue>> =
        Arc::new(Mutex::new(
            crate::federation::pending_queue::PendingFederationQueue::new(),
        ));
    // 2b FAC-D3 — empty policy store = permit-all (prime invariant); the
    // existing harness suite is therefore the default-permit regression.
    let federation_policy: Arc<Mutex<crate::federation::federation_policy::FederationPolicyStore>> =
        Arc::new(Mutex::new(
            crate::federation::federation_policy::FederationPolicyStore::new(),
        ));

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
    let accept_federation_queue = Arc::clone(&federation_queue);
    let accept_federation_queue_path = federation_queue_path.clone();
    let accept_federation_policy = Arc::clone(&federation_policy);
    let accept_require_approval = require_approval;
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
                            let req_appr = accept_require_approval;
                            let fed_queue = Arc::clone(&accept_federation_queue);
                            let fed_policy = Arc::clone(&accept_federation_policy);
                            let fed_queue_path = accept_federation_queue_path.clone();
                            let h = tokio::spawn(async move {
                                handle_connection(
                                    conn, rt, conns, senders, fed_senders, fed_reg,
                                    fed_reg_path, kp, ndx(&home), local_mode, ids, sdir,
                                    sync_batch_size, req_appr, fed_queue, fed_queue_path,
                                    fed_policy,
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
        federation_queue,
        federation_policy,
        spaces_dir,
        identities_path,
        data_dir,
        shutdown_tx,
        accept_handle: Some(accept_handle),
        connection_handles,
        push_attempts,
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

    // Phase 9 Commit 3b-3-pre — fresh push-attempt counter on respawn.
    // The respawned Node re-uses its prior `node_id` so CounterLayer routes
    // future increments to the new Arc; the prior Weak in REGISTRY becomes
    // invalid on the next access (the strong ref was dropped at
    // shutdown_keep_data time when the prior InProcessNode dropped).
    // The REGISTRY entry is overwritten with the new Weak below.
    let push_attempts: Counter = Arc::new(StdMutex::new(HashMap::new()));
    registry()
        .lock()
        .unwrap()
        .insert(node_id.clone(), Arc::downgrade(&push_attempts));

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
    // Approval gate is off for the drop-and-recover path (require_approval =
    // false); a fresh empty queue suffices since the gate never consults it.
    let federation_queue_path = data_dir.path().join("xgen-node_federation_queue.json");
    let federation_queue: Arc<Mutex<crate::federation::pending_queue::PendingFederationQueue>> =
        Arc::new(Mutex::new(
            crate::federation::pending_queue::PendingFederationQueue::new(),
        ));
    // 2b FAC-D3 — empty policy store = permit-all (prime invariant); the
    // existing harness suite is therefore the default-permit regression.
    let federation_policy: Arc<Mutex<crate::federation::federation_policy::FederationPolicyStore>> =
        Arc::new(Mutex::new(
            crate::federation::federation_policy::FederationPolicyStore::new(),
        ));

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
    let accept_federation_queue = Arc::clone(&federation_queue);
    let accept_federation_queue_path = federation_queue_path.clone();
    let accept_federation_policy = Arc::clone(&federation_policy);
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
                            let fed_queue = Arc::clone(&accept_federation_queue);
                            let fed_policy = Arc::clone(&accept_federation_policy);
                            let fed_queue_path = accept_federation_queue_path.clone();
                            let h = tokio::spawn(async move {
                                handle_connection(
                                    conn, rt, conns, senders, fed_senders, fed_reg,
                                    fed_reg_path, kp, ndx(&home), local_mode, ids, sdir,
                                    sync_batch_size, false, fed_queue, fed_queue_path,
                                    fed_policy,
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
        federation_queue,
        federation_policy,
        spaces_dir,
        identities_path,
        data_dir,
        shutdown_tx,
        accept_handle: Some(accept_handle),
        connection_handles,
        push_attempts,
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
    use crate::federation::registry::{FederationRelationship, FederationState};

    {
        let mut reg = initiator.federation_registry.lock().await;
        reg.upsert(FederationRelationship {
            peer_node_id: ndx(&receiver.node_id),
            shared_spaces: shared_spaces.iter().map(|s| sdx(s)).collect(),
            negotiated_version: "0.1".to_string(),
            negotiated_serialisation: "json".to_string(),
            session_id: "xgen://hash/sha256:phase9-harness-session".to_string(),
            last_connected: now_rfc(),
            peer_url: Some(receiver.endpoint.clone()),
            state: FederationState::Active,
        });
        reg.mark_lost(&ndx(&receiver.node_id), Utc::now() - chrono::Duration::minutes(20));
    }

    let attempt_cursor = Arc::new(Mutex::new(HashMap::new()));
    let shared_spaces_typed: Vec<SpaceXgid> = shared_spaces.iter().map(|s| sdx(s)).collect();

    tokio::spawn(attempt_reconnect(
        Arc::clone(&initiator.runtime),
        initiator.client_senders.clone(),
        Arc::clone(&initiator.federation_peer_senders),
        Arc::clone(&initiator.federation_registry),
        initiator.federation_registry_path.clone(),
        Arc::clone(&initiator.keypair),
        ndx(&initiator.node_id),
        initiator.spaces_dir.clone(),
        initiator.identities_path.clone(),
        true, // local_mode
        initiator.endpoint.clone(),
        ndx(&receiver.node_id),
        receiver.endpoint.clone(),
        shared_spaces_typed,
        attempt_cursor,
        Arc::clone(&initiator.federation_policy),
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

/// Drive an inbound federation handshake from `initiator` to `receiver`
/// without asserting establishment — used when the receiver is expected NOT
/// to establish (e.g. it has the FAC-D1 approval gate on and will answer
/// `Reject 2003`). Mirrors [`federate`]'s production initiator setup
/// (`attempt_reconnect` over a real WS connection to the receiver's
/// endpoint, which exercises the receiver's real `handle_federation_incoming`
/// server path), then returns immediately. The caller observes the outcome on
/// the receiver side (e.g. polling `receiver.federation_queue`).
pub async fn attempt_federation_no_wait(
    initiator: &InProcessNode,
    receiver: &InProcessNode,
    shared_spaces: Vec<String>,
) {
    use crate::federation::registry::{FederationRelationship, FederationState};

    {
        let mut reg = initiator.federation_registry.lock().await;
        reg.upsert(FederationRelationship {
            peer_node_id: ndx(&receiver.node_id),
            shared_spaces: shared_spaces.iter().map(|s| sdx(s)).collect(),
            negotiated_version: "0.1".to_string(),
            negotiated_serialisation: "json".to_string(),
            session_id: "xgen://hash/sha256:approval-gate-attempt".to_string(),
            last_connected: now_rfc(),
            peer_url: Some(receiver.endpoint.clone()),
            state: FederationState::Active,
        });
        reg.mark_lost(&ndx(&receiver.node_id), Utc::now() - chrono::Duration::minutes(20));
    }

    let attempt_cursor = Arc::new(Mutex::new(HashMap::new()));
    let shared_spaces_typed: Vec<SpaceXgid> = shared_spaces.iter().map(|s| sdx(s)).collect();

    tokio::spawn(attempt_reconnect(
        Arc::clone(&initiator.runtime),
        initiator.client_senders.clone(),
        Arc::clone(&initiator.federation_peer_senders),
        Arc::clone(&initiator.federation_registry),
        initiator.federation_registry_path.clone(),
        Arc::clone(&initiator.keypair),
        ndx(&initiator.node_id),
        initiator.spaces_dir.clone(),
        initiator.identities_path.clone(),
        true,
        initiator.endpoint.clone(),
        ndx(&receiver.node_id),
        receiver.endpoint.clone(),
        shared_spaces_typed,
        attempt_cursor,
        Arc::clone(&initiator.federation_policy),
    ));
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
        identity_id: idx(&pubkey_uri(key)),
        display_name: None,
        is_ai: false,
        ai_capabilities: None,
        registered_at: "2026-05-21T00:00:00.000Z".to_string(),
        trust_assertion: None,
        devices: vec![],
        home_node: ndx(home_node),
        update_version: 0,
        revoked: false,
        revoked_at: None,
        revocation_reason: None,
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

// ───────────────────────────────────────────────────────────────────────────
// Phase 9 Commit 3b-3-pre — unit tests for the push-attempt counter
// ───────────────────────────────────────────────────────────────────────────
//
// These tests deliberately do NOT use `#[traced_test]` — the counter is fed
// by the global tracing-subscriber Layer (CounterLayer + LogBufferLayer)
// installed by `ensure_subscriber_installed`, which is overridden on the
// test thread when `#[traced_test]` is active. The C2 test follows the same
// no-`#[traced_test]` pattern; these unit tests pin that contract first.
//
// `#[serial_test::serial]` is used to serialise these with each other and
// with the other Phase 9 tests so the global LOG_BUFFER and REGISTRY don't
// see cross-test interleaving.

#[cfg(test)]
mod counter_unit_tests {
    use super::*;
    use crate::node::runtime::EventOrigin;
    use crate::space::state::{build_room_create_event, build_space_create_event, sign_event};
    use crate::wire::types::Event;
    use serde_json::json;

    /// Counter starts at 0 for every (event_id, peer_node_id) lookup on
    /// a freshly-spawned Node and the snapshot map is empty. Sanity
    /// baseline that the registry registration is idempotent and the
    /// counter Arc is initialised empty.
    ///
    /// This test does not need [`install_harness_subscriber`] because it
    /// reads counter state directly via accessors without exercising the
    /// trace-event ingest path.
    #[serial_test::serial]
    #[tokio::test(flavor = "current_thread")]
    async fn counter_starts_empty_on_fresh_node() {
        harness_clear_logs();
        let node = spawn_in_process_node().await;

        assert_eq!(
            node.push_attempt_count("xgen://hash/sha256:nonexistent_event", "xgen://pubkey/ed25519:nonexistent_peer"),
            0,
            "counter must return 0 for any (event_id, peer_node_id) on a fresh Node"
        );
        let attempts = node.push_attempts();
        assert!(
            attempts.is_empty(),
            "push_attempts snapshot must be empty on a fresh Node"
        );

        node.shutdown().await;
    }

    /// Counter increments by exactly 1 per `federation_push_sent` G2 trace
    /// event whose `local_node_id` matches this Node's id. The Layer reads
    /// the (local_node_id, peer_node_id, event_id) fields from the trace
    /// and routes via the REGISTRY weak-ref lookup. Setup: two Nodes,
    /// federated for one Space, A submits one event locally → A's counter
    /// gains one entry for (event_id, B.node_id).
    #[serial_test::serial]
    #[tokio::test(flavor = "current_thread")]
    async fn counter_increments_on_federation_push_sent_for_matching_local_node() {
        let _sub = install_harness_subscriber();
        harness_clear_logs();
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        // Alice on A; her Identity registered on both Nodes so F-10 doesn't
        // hold Bob's event indefinitely (sibling-shape to Scenario 1).
        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_a.register_identity(&alice_key).await;
        node_b.register_identity(&alice_key).await;

        let space_ev = sign_event(
            build_space_create_event(
                &alice_key,
                "phase9-counter-unit-test-space",
                None,
                1,
                &node_a.node_id,
                None,
            false),
            &alice_key,
        );
        let space_id: String = event_id_str(&space_ev);
        node_a.ingest(space_ev).await;

        let room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", None),
            &alice_key,
        );
        let room_id: String = event_id_str(&room_ev);
        node_a.ingest(room_ev).await;

        let invite_ev = sign_event(
            Event::new(
                crate::wire::types::EventType::MembershipInvite,
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
        node_a.ingest(invite_ev).await;

        let join_ev = sign_event(
            Event::new(
                crate::wire::types::EventType::MembershipJoin,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&invite_id)],
                now_rfc(),
                json!({}),
            ),
            &alice_key,
        );
        node_a.ingest(join_ev).await;

        federate(&node_a, &node_b, vec![space_id.clone()]).await;

        // Wait for the bootstrap delta to land on B (sibling-shape to Sc2).
        let tips = node_a.dag_tips(&space_id).await;
        assert_eq!(tips.len(), 1);
        for tip in &tips {
            assert!(
                node_b
                    .wait_for_event(&space_id, tip, Duration::from_secs(10))
                    .await,
                "B did not ingest bootstrap-delta tip {tip} within 10s"
            );
        }

        // Snapshot A's counter before Alice's post — bootstrap activity
        // (state.federation_add) already fired through apply_federation_push
        // and may have populated some entries on both Nodes. We assert
        // the delta from Alice's single post, not the absolute count.
        let a_before = node_a.push_attempts();

        // Alice posts a single MessageText via submit_locally (the harness
        // wrapper that calls dispatch_event + apply_federation_push). The
        // federation_push_sent G2 trace fires from A's apply_federation_push
        // with local_node_id=A, peer_node_id=B, event_id=alice_msg_id.
        let alice_msg = sign_event(
            Event::new(
                crate::wire::types::EventType::MessageText,
                idx(&alice_id),
                rdx(&room_id),
                sdx(&space_id),
                tips.iter().map(|t| edx(t)).collect(),
                now_rfc(),
                json!({ "text": "counter unit test" }),
            ),
            &alice_key,
        );
        let alice_msg_id: String = event_id_str(&alice_msg);
        let _outcome = node_a.submit_locally(alice_msg).await;

        // Brief poll for the trace event to flow through the Layer and
        // increment the counter; the Layer is synchronous on event emit
        // but tokio scheduling adds a few ms.
        let mut increment_observed = false;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_millis(500) {
            if node_a.push_attempt_count(&alice_msg_id, &node_b.node_id) == 1 {
                increment_observed = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            increment_observed,
            "A's push_attempt_count((alice_msg_id, B.node_id)) must increment to 1 \
             within 500ms of submit_locally — got {} (before snapshot had {} entries)",
            node_a.push_attempt_count(&alice_msg_id, &node_b.node_id),
            a_before.len()
        );

        // B's counter must NOT increment for this event (F-5 short-circuits
        // B's apply_federation_push when origin=ReceivedViaFederation; only
        // federation_push_skipped_origin fires on B, not federation_push_sent).
        assert_eq!(
            node_b.push_attempt_count(&alice_msg_id, &node_a.node_id),
            0,
            "B's counter for (alice_msg_id, A.node_id) MUST stay 0 — F-5 \
             short-circuits B's apply_federation_push under origin=ReceivedViaFederation"
        );

        node_a.shutdown().await;
        node_b.shutdown().await;
    }

    /// Log buffer captures the `federation_push_skipped_origin` trace event
    /// (xgen-core: no — emitted from xgen-node::federation_session) so the
    /// C2 test can use harness_logs_contain as the positive F-5 trace
    /// assertion sibling of Scenario 2's `logs_contain` (Sc2 uses
    /// `#[traced_test]`; C2 uses this harness facility instead).
    #[serial_test::serial]
    #[tokio::test(flavor = "current_thread")]
    async fn log_buffer_captures_federation_push_skipped_origin_trace() {
        let _sub = install_harness_subscriber();
        harness_clear_logs();
        let node_a = spawn_in_process_node().await;
        let node_b = spawn_in_process_node().await;

        let alice_key = keypair::generate();
        let alice_id = pubkey_uri(&alice_key);
        node_a.register_identity(&alice_key).await;
        node_b.register_identity(&alice_key).await;

        let space_ev = sign_event(
            build_space_create_event(
                &alice_key,
                "phase9-log-buffer-unit-test-space",
                None,
                1,
                &node_a.node_id,
                None,
            false),
            &alice_key,
        );
        let space_id: String = event_id_str(&space_ev);
        node_a.ingest(space_ev).await;

        let room_ev = sign_event(
            build_room_create_event(&alice_key, &space_id, "general", None),
            &alice_key,
        );
        let room_id: String = event_id_str(&room_ev);
        node_a.ingest(room_ev).await;

        let invite_ev = sign_event(
            Event::new(
                crate::wire::types::EventType::MembershipInvite,
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
        node_a.ingest(invite_ev).await;

        let join_ev = sign_event(
            Event::new(
                crate::wire::types::EventType::MembershipJoin,
                idx(&alice_id),
                rdx(""),
                sdx(&space_id),
                vec![edx(&invite_id)],
                now_rfc(),
                json!({}),
            ),
            &alice_key,
        );
        node_a.ingest(join_ev).await;

        federate(&node_a, &node_b, vec![space_id.clone()]).await;
        let tips = node_a.dag_tips(&space_id).await;
        for tip in &tips {
            let _ = node_b
                .wait_for_event(&space_id, tip, Duration::from_secs(10))
                .await;
        }

        // Now dispatch an event on B with origin=ReceivedViaFederation —
        // the F-5 guard fires; G2 federation_push_skipped_origin trace
        // emits. We invoke apply_federation_push directly because the
        // harness's submit_locally uses LocallySubmitted and would not
        // exercise the F-5 path.
        let alice_msg = sign_event(
            Event::new(
                crate::wire::types::EventType::MessageText,
                idx(&alice_id),
                rdx(&room_id),
                sdx(&space_id),
                tips.iter().map(|t| edx(t)).collect(),
                now_rfc(),
                json!({ "text": "log buffer unit test" }),
            ),
            &alice_key,
        );
        // Directly call apply_federation_push on B with ReceivedViaFederation
        // origin — fires the F-5 short-circuit + G2 trace emission.
        let local_node_b = ndx(&node_b.node_id);
        crate::federation_session::apply_federation_push(
            &alice_msg,
            EventOrigin::ReceivedViaFederation,
            &node_b.runtime,
            &node_b.federation_peer_senders,
            &local_node_b,
            None,
        )
        .await;

        // Brief poll for the trace line to flow through LogBufferLayer.
        let mut captured = false;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_millis(500) {
            if harness_logs_contain("federation_push_skipped_origin") {
                captured = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            captured,
            "log buffer must capture federation_push_skipped_origin trace within 500ms"
        );

        node_a.shutdown().await;
        node_b.shutdown().await;
    }
}
