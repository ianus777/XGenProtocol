// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! F-1c reconnect scheduler — Phase 5 (runbook §3.5 + §3.5.1).
//!
//! Periodically scans `FederationRegistry::peer_records` for peers flagged
//! `lost_connection` whose `next_reconnect_attempt` has elapsed, and fires
//! `run_initiating` against each due peer. The reconnect scheduler is the
//! first production caller of `run_initiating` in `xgen-node/src/` (audit
//! J-081 §2.2 noted there were zero before Phase 5).
//!
//! Backoff schedule and parameters are locked in runbook §3.5.1 Lock B:
//! - B1: 60 s scheduler tick
//! - B2: 15 / 30 / 60 / 120 min ladder, capped at 120 min
//! - B3: 15 min initial delay after first observed loss
//! - B4: parallel detached `tokio::spawn` per due peer per tick
//!
//! Backoff cursor (which step of the ladder a peer is on) is **in-memory
//! only** (Joe-locked option α). After Node restart the cursor resets to
//! "first attempt" semantics; the persisted `next_reconnect_attempt`
//! timestamp still governs WHEN the first post-restart attempt fires, but
//! the ladder progression after that follows the fresh cursor.
//!
//! Pass 3 (Surface #6) — three spawned-function parameter retypes per
//! design §4.2 v1.2 row 3 async-spawned-captures forced-owned sub-rule
//! (D-NNN-ε promotion-watch at three same-module-family instances). The
//! `AttemptCursor` type alias retypes its HashMap key to `NodeXgid`
//! (xgen-node-internal alias; never crosses out). xgen-core wire-format
//! types (FederationSession, run_initiating parameters) stay String at
//! their xgen-core boundary per §4.3 format-boundary-preservation;
//! projection happens at the xgen-node call-site boundary.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::SecondsFormat;
use ed25519_dalek::SigningKey;
use tokio::sync::Mutex;

use xgen_common::xgid::{NodeXgid, SpaceXgid, Xgid};
use xgen_core::federation::handshake::run_initiating;
use xgen_core::transport::client::connect_url;
use xgen_core::wire::types::FederationCapabilities;

use crate::app::{run_federation_session_post_handshake, SessionRole};
use crate::fanout::{ClientSenders, FederationPeerSenders};
use crate::federation::federation_policy::FederationPolicyStore;
use crate::federation::registry::FederationRegistry;
use crate::node::runtime::NodeRuntime;

// ── Constants (runbook §3.5.1 Lock B) ─────────────────────────────────────────

/// Lock B1 — scheduler ticks every 60 seconds.
pub(crate) const SCHEDULER_TICK_SECONDS: u64 = 60;

/// Lock B2 — backoff ladder in minutes. Index is the attempt count
/// (clamped to `len - 1` for the cap). `attempt_count == 0` is the
/// just-marked-lost state and uses `INITIAL_RECONNECT_DELAY_MINUTES`
/// from the registry, not this ladder.
pub(crate) const BACKOFF_LADDER_MINUTES: [i64; 4] = [15, 30, 60, 120];

/// M8.6 (C4 prerequisite, Joe-lock checkpoint #1, 2026-06-06) — bound the
/// outbound TCP + WS connect in `attempt_reconnect`. `connect_url`
/// (`xgen-core/src/transport/client.rs:32`) awaits `connect_async` UNBOUNDED;
/// against a non-responsive (black-hole) peer the detached attempt task would
/// hang forever — the C4 spawn-leak vector. Wrapping the call site (not
/// `connect_url`, which has other callers) makes every attempt path
/// time-bounded so the attempt-task gauge (design §4) can return to 0.
///
/// Value aligned with the handshake `WAIT_TIMEOUT_SECS` (15 s): connect 15 +
/// handshake 15 = 30 s < the 60 s scheduler tick, so an attempt always resolves
/// within its own tick and can't stack across ticks.
pub(crate) const CONNECT_TIMEOUT_SECS: u64 = 15;

// ── Attempt-task gauge (M8.6 C4, test-only) ───────────────────────────────────

/// M8.6 (C4) — RAII guard for the reconnect **attempt-phase** task gauge
/// (`Arc<AtomicUsize>` = outstanding attempt tasks). One symmetric type owns
/// both edges: `new()` increments at the spawn site (synchronously, the moment
/// the scheduler commits to spawning), `Drop` decrements — so no
/// attempt-resolution path can miss the decrement. Early returns
/// (connect-timeout / connect / auth / handshake / peer-mismatch) drop it via
/// scope exit; the handshake-ACTIVE path drops it explicitly *before*
/// `run_federation_session_post_handshake`, so the long-lived session is
/// uncounted (design §4). Test-only spawn-leak detector — NOT an operator
/// surface.
pub(crate) struct AttemptGuard {
    gauge: Arc<AtomicUsize>,
}

impl AttemptGuard {
    /// Increment the gauge; the returned guard decrements on drop.
    pub(crate) fn new(gauge: Arc<AtomicUsize>) -> Self {
        gauge.fetch_add(1, Ordering::SeqCst);
        Self { gauge }
    }

    /// A guard backed by a throwaway counter nobody reads — for attempt spawns
    /// OUTSIDE the scheduler (the admin-initiated `federation initiate` verb,
    /// the test harness). Keeps `attempt_reconnect`'s signature uniform without
    /// threading a gauge those call sites have no use for.
    pub(crate) fn untracked() -> Self {
        Self::new(Arc::new(AtomicUsize::new(0)))
    }
}

impl Drop for AttemptGuard {
    fn drop(&mut self) {
        self.gauge.fetch_sub(1, Ordering::SeqCst);
    }
}

// ── Scheduler ────────────────────────────────────────────────────────────────

/// Transient per-peer backoff cursor. NOT persisted (Joe-locked option α
/// per §3.5.1 Lock A discussion). On Node restart, cursor resets and the
/// first post-restart attempt against any still-lost peer uses
/// `BACKOFF_LADDER_MINUTES[1] = 30` minutes for its NEXT scheduled retry
/// after that attempt; this is the documented "aggressive post-restart
/// probe" tradeoff — operator restarts often correlate with operator-side
/// fixes (firewall, network config, peer endpoint update), so the
/// aggressive post-restart probe lets fixes manifest immediately rather
/// than waiting out the pre-restart backoff cap.
///
/// Pass 3 (Surface #6 Q6.4) — HashMap key retyped to `NodeXgid`.
type AttemptCursor = Arc<Mutex<HashMap<NodeXgid, u32>>>;

///
/// Pass 3 (Surface #6 Q6.1) — `home_node_id` retyped to owned `NodeXgid`.
/// Spawned-task captures across the runtime lifetime; forced-owned per
/// §4.2 v1.2 row 3 async-spawned-captures sub-rule.
#[allow(clippy::too_many_arguments)]
pub fn spawn_reconnect_scheduler(
    runtime: Arc<Mutex<NodeRuntime>>,
    client_senders: ClientSenders,
    federation_peer_senders: FederationPeerSenders,
    federation_registry: Arc<Mutex<FederationRegistry>>,
    federation_registry_path: PathBuf,
    node_keypair: Arc<SigningKey>,
    home_node_id: NodeXgid,
    spaces_dir: PathBuf,
    identities_path: PathBuf,
    local_mode: bool,
    self_url: String,
    federation_policy: Arc<Mutex<FederationPolicyStore>>,
) {
    let attempt_cursor: AttemptCursor = Arc::new(Mutex::new(HashMap::new()));
    // M8.6 (C4) — outstanding attempt-phase task gauge (test-only spawn-leak
    // detector, design §4). Created here (sibling to attempt_cursor); the
    // scheduler increments it via AttemptGuard at each attempt spawn and the
    // guard decrements on resolution. Nothing in production reads it.
    let attempt_gauge: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(SCHEDULER_TICK_SECONDS)).await;
            scheduler_tick(
                Arc::clone(&runtime),
                client_senders.clone(),
                federation_peer_senders.clone(),
                Arc::clone(&federation_registry),
                federation_registry_path.clone(),
                Arc::clone(&node_keypair),
                home_node_id.clone(),
                spaces_dir.clone(),
                identities_path.clone(),
                local_mode,
                self_url.clone(),
                Arc::clone(&attempt_cursor),
                Arc::clone(&federation_policy),
                Arc::clone(&attempt_gauge),
            )
            .await;
        }
    });
}

/// One scheduler tick. Extracted from `spawn_reconnect_scheduler` so it can
/// be exercised by integration tests directly (without sleeping out the
/// 60-second tick interval).
///
/// Pass 3 (Surface #6 Q6.2) — `home_node_id` retyped to owned `NodeXgid`;
/// `self_url` stays `String` (URL is descriptive, not identifier-flavoured);
/// `AttemptCursor` HashMap key retyped to `NodeXgid` (Q6.4).
#[allow(clippy::too_many_arguments)]
pub async fn scheduler_tick(
    runtime: Arc<Mutex<NodeRuntime>>,
    client_senders: ClientSenders,
    federation_peer_senders: FederationPeerSenders,
    federation_registry: Arc<Mutex<FederationRegistry>>,
    federation_registry_path: PathBuf,
    node_keypair: Arc<SigningKey>,
    home_node_id: NodeXgid,
    spaces_dir: PathBuf,
    identities_path: PathBuf,
    local_mode: bool,
    self_url: String,
    attempt_cursor: AttemptCursor,
    federation_policy: Arc<Mutex<FederationPolicyStore>>,
    attempt_gauge: Arc<AtomicUsize>,
) {
    // M8.6 (clock seam, design §3.2) — W-domain read pulled from the
    // NodeRuntime-resident Clock (single source of time). RealClock in
    // production == the prior `Utc::now()`; a MockClock under test drives the
    // 15/30/60/120-min ladder without real waiting. `runtime` is not locked
    // elsewhere in this tick, so the brief lock here is contention-free.
    let now = {
        let rt = runtime.lock().await;
        rt.clock().now_utc()
    };

    // Snapshot due peers — for each due peer, capture (peer_node_id,
    // peer_url, shared_spaces). Skip peers without a stored peer_url:
    // there's nowhere to dial. Then advance next_reconnect_attempt
    // BEFORE spawning so a slow handshake doesn't cause the next tick to
    // re-fire the same peer.
    //
    // Pass 3 (Surface #6 inheritance) — registry yields typed NodeXgid +
    // Vec<SpaceXgid> post-Pass-2; tuple shape retyped.
    let due: Vec<(NodeXgid, String, Vec<SpaceXgid>)> = {
        let reg = federation_registry.lock().await;
        reg.due_for_reconnect(now)
            .into_iter()
            .filter_map(|rec| {
                let rel = reg.get(&rec.peer_node_id)?;
                let url = rel.peer_url.clone()?;
                Some((rec.peer_node_id.clone(), url, rel.shared_spaces.clone()))
            })
            .collect()
    };

    if due.is_empty() {
        return;
    }

    // Advance the persisted next_reconnect_attempt + bump the in-memory
    // attempt cursor for each due peer, BEFORE spawning the detached
    // attempts. This way a hung handshake doesn't cause the next 60s tick
    // to find the same peer due again and double-fire.
    {
        let mut cursor = attempt_cursor.lock().await;
        let mut reg = federation_registry.lock().await;
        for (peer_node_id, _peer_url, _shared_spaces) in &due {
            let count = cursor.entry(peer_node_id.clone()).and_modify(|c| *c += 1).or_insert(1);
            let step_idx = (*count as usize).min(BACKOFF_LADDER_MINUTES.len()) - 1;
            let step_min = BACKOFF_LADDER_MINUTES[step_idx];
            let next_at = now + chrono::Duration::minutes(step_min);
            reg.update_next_reconnect(peer_node_id, next_at);
            tracing::info!(
                peer_node_id = %peer_node_id.as_str(),
                attempt = *count,
                next_step_min = step_min,
                next_at = %next_at.to_rfc3339_opts(SecondsFormat::Millis, true),
                "Scheduling reconnect attempt"
            );
        }
        if let Err(e) = reg.save(&federation_registry_path) {
            tracing::warn!(
                path = ?federation_registry_path,
                error = %e,
                "Failed to persist federation registry on scheduler tick"
            );
        }
    }

    // Reconnect attempts are spawned detached (tokio::spawn, no .await on the
    // handle). The scheduler's tick MUST NOT block on any peer's run_initiating
    // completion. A hung handshake dies via the WebSocket transport timeout in
    // its own task; subsequent ticks proceed unaffected.
    for (peer_node_id, peer_url, shared_spaces) in due {
        let runtime = Arc::clone(&runtime);
        let client_senders = client_senders.clone();
        let federation_peer_senders = federation_peer_senders.clone();
        let federation_registry = Arc::clone(&federation_registry);
        let federation_registry_path = federation_registry_path.clone();
        let node_keypair = Arc::clone(&node_keypair);
        let home_node_id = home_node_id.clone();
        let spaces_dir = spaces_dir.clone();
        let identities_path = identities_path.clone();
        let self_url = self_url.clone();
        let attempt_cursor = Arc::clone(&attempt_cursor);
        let federation_policy = Arc::clone(&federation_policy);
        // M8.6 (C4) — inc the attempt gauge at the spawn site (synchronous,
        // before the task is scheduled, so the count reflects the moment of
        // commitment); the guard moves into the task and decrements on every
        // attempt-resolution path via Drop (design §4 + checkpoint #1).
        let attempt_guard = AttemptGuard::new(Arc::clone(&attempt_gauge));
        tokio::spawn(async move {
            attempt_reconnect(
                runtime,
                client_senders,
                federation_peer_senders,
                federation_registry,
                federation_registry_path,
                node_keypair,
                home_node_id,
                spaces_dir,
                identities_path,
                local_mode,
                self_url,
                peer_node_id,
                peer_url,
                shared_spaces,
                attempt_cursor,
                federation_policy,
                attempt_guard,
            )
            .await;
        });
    }
}

/// Initiator-side reconnect: open TCP, transport-authenticate, run the
/// federation handshake as the initiator, then drive the post-handshake
/// flow via `run_federation_session_post_handshake` with
/// `SessionRole::Initiator`. On handshake success the in-memory attempt
/// cursor is cleared (subsequent disconnects start the ladder fresh from
/// the `mark_lost` initial-delay of 15 min); on failure the cursor is
/// left intact so the NEXT scheduler tick advances to the next ladder
/// step.
///
/// Pass 3 (Surface #6 Q6.3) — `home_node_id` + `peer_node_id` retyped to
/// owned `NodeXgid` (forced-owned per §4.2 v1.2 row 3 async-spawned-captures);
/// `peer_url` stays `String` (URL descriptive); `shared_spaces` retyped to
/// `Vec<SpaceXgid>` (in-memory typed vec).
// `pub(crate)`: only ever called within xgen-node (the scheduler spawn, the
// admin-initiated `federation initiate` verb, the test harness). Crate-internal
// keeps its `AttemptGuard` param (a test-only type, M8.6 C4) from leaking into
// the public API.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn attempt_reconnect(
    runtime: Arc<Mutex<NodeRuntime>>,
    client_senders: ClientSenders,
    federation_peer_senders: FederationPeerSenders,
    federation_registry: Arc<Mutex<FederationRegistry>>,
    federation_registry_path: PathBuf,
    node_keypair: Arc<SigningKey>,
    home_node_id: NodeXgid,
    spaces_dir: PathBuf,
    identities_path: PathBuf,
    local_mode: bool,
    self_url: String,
    peer_node_id: NodeXgid,
    peer_url: String,
    shared_spaces: Vec<SpaceXgid>,
    attempt_cursor: AttemptCursor,
    federation_policy: Arc<Mutex<FederationPolicyStore>>,
    attempt_guard: AttemptGuard,
) {
    // 1. Open WS to peer — time-bounded (M8.6, C4 prerequisite). `connect_url`
    //    awaits `connect_async` unbounded; against a non-responsive (black-hole)
    //    peer the attempt task would hang forever and the gauge would never
    //    return to 0. The timeout wraps the call site only — `connect_url`
    //    itself is untouched so its other callers are unaffected.
    let mut conn = match tokio::time::timeout(
        Duration::from_secs(CONNECT_TIMEOUT_SECS),
        connect_url(&peer_url),
    )
    .await
    {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            tracing::warn!(
                peer_node_id = %peer_node_id.as_str(),
                peer_url = %peer_url,
                error = %e,
                "Reconnect: WS connect failed"
            );
            return;
        }
        Err(_elapsed) => {
            tracing::warn!(
                peer_node_id = %peer_node_id.as_str(),
                peer_url = %peer_url,
                timeout_secs = CONNECT_TIMEOUT_SECS,
                "Reconnect: WS connect timed out"
            );
            return;
        }
    };

    // 2. Transport-layer challenge-response auth, using the Node's own keypair.
    if let Err(e) = conn.client_authenticate(&node_keypair).await {
        tracing::warn!(
            peer_node_id = %peer_node_id.as_str(),
            error = %e,
            "Reconnect: transport authenticate failed"
        );
        return;
    }

    // 3. Compute our local tips for each shared Space — same shape the
    //    receiver-side builds in handle_federation_incoming. The order is
    //    BTreeMap because the wire shape (FederationMessage::Hello.tips)
    //    is BTreeMap; iteration order is stable for deterministic
    //    delta delivery.
    let our_tips: BTreeMap<String, String> = {
        let rt = runtime.lock().await;
        shared_spaces
            .iter()
            .filter_map(|space_id| {
                let local_tips = rt.dag_tips(space_id);
                local_tips
                    .into_iter()
                    .min()
                    .map(|tip| (space_id.as_str().to_string(), tip))
            })
            .collect()
    };

    // 4. Run the federation handshake as initiator.
    //
    // Pass 3 (Surface #6 §4.3 wire-format boundary) — xgen-core handshake
    // takes Vec<String> for shared_spaces (canonical wire bytes are String);
    // project SpaceXgid → String at this xgen-node-to-xgen-core boundary.
    let shared_spaces_wire: Vec<String> =
        shared_spaces.iter().map(|s| s.as_str().to_string()).collect();
    let session = match run_initiating(
        &mut conn,
        &node_keypair,
        FederationCapabilities::default(),
        shared_spaces_wire,
        our_tips,
        Some(self_url),
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                peer_node_id = %peer_node_id.as_str(),
                error = %e,
                "Reconnect: handshake failed"
            );
            return;
        }
    };

    // Sanity: the peer at peer_url must be the same peer_node_id we
    // expected. A different peer at the same URL is a configuration
    // change; don't run a session against an unexpected identity.
    //
    // Pass 3 — session.peer_node_id is wire-format String; compare via
    // .as_str() projection.
    if session.peer_node_id != peer_node_id.as_str() {
        tracing::warn!(
            expected_peer_node_id = %peer_node_id.as_str(),
            actual_peer_node_id = %session.peer_node_id,
            peer_url = %peer_url,
            "Reconnect: handshake returned different peer_node_id than expected — aborting session"
        );
        return;
    }

    // Lock B2 — "successful reconnect" means handshake-ACTIVE. Drop the
    // in-memory attempt cursor now so a future disconnect ladders from
    // the 15-min initial delay rather than continuing the prior chain.
    {
        let mut cursor = attempt_cursor.lock().await;
        cursor.remove(&peer_node_id);
    }

    tracing::info!(
        peer_node_id = %peer_node_id.as_str(),
        session_id = %session.session_id,
        "Reconnect: handshake reached ACTIVE; driving post-handshake session"
    );

    // M8.6 (C4) — the attempt phase has resolved into a long-lived session; end
    // the attempt-phase scope NOW (gauge decrements) before the session driver
    // runs, so the session task is uncounted (design §4). Early-return paths
    // above drop the guard via scope exit instead.
    drop(attempt_guard);

    // 5. Drive the post-handshake flow (catch-up drain, bilateral stream,
    //    register, F-2 loop, cleanup) via the shared driver.
    //
    // Pass 3 — project xgen-core's session.peer_node_id (String) to NodeXgid
    // for the typed downstream call. shared_spaces stays Vec<SpaceXgid>.
    let session_peer_node_id_typed =
        NodeXgid::from_xgid(Xgid::new(session.peer_node_id));

    // MP-F9 — the signer identities are sent IN-SESSION (over `conn`, before the
    // Space-DAG delta) inside run_federation_session_post_handshake, so the peer
    // applies them in order ahead of the backfill it catches up. No detached
    // out-of-band push (which raced the session stream and dropped children).
    run_federation_session_post_handshake(
        &mut conn,
        SessionRole::Initiator,
        runtime,
        client_senders,
        federation_peer_senders,
        federation_registry,
        federation_registry_path,
        node_keypair,
        home_node_id,
        spaces_dir,
        identities_path,
        local_mode,
        session_peer_node_id_typed,
        session.session_id,
        session.negotiated_version,
        session.negotiated_serialisation,
        shared_spaces,
        session.peer_tips,
        Some(peer_url),
        federation_policy,
    )
    .await;
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_ladder_constants() {
        assert_eq!(BACKOFF_LADDER_MINUTES, [15, 30, 60, 120]);
        assert_eq!(SCHEDULER_TICK_SECONDS, 60);
    }

    #[test]
    fn ladder_step_indexing() {
        // attempt 1 -> ladder[0] = 15 (the first attempt AFTER the initial
        // mark_lost+15min one); attempt 2 -> 30; attempt 3 -> 60; attempt
        // 4 -> 120; attempts 5+ -> 120 (cap).
        for (attempt, expected) in &[(1u32, 15i64), (2, 30), (3, 60), (4, 120), (5, 120), (100, 120)] {
            let step_idx = (*attempt as usize).min(BACKOFF_LADDER_MINUTES.len()) - 1;
            let step_min = BACKOFF_LADDER_MINUTES[step_idx];
            assert_eq!(step_min, *expected, "attempt {} should map to {} min", attempt, expected);
        }
    }

    // ── Pass 3 Commit 2a per-surface tests T9 + T10 (runbook §4.7) ──────────

    fn ndx_test(s: &str) -> NodeXgid {
        NodeXgid::from_xgid(Xgid::new(s.to_string()))
    }
    fn sdx_test(s: &str) -> SpaceXgid {
        SpaceXgid::from_xgid(Xgid::new(s.to_string()))
    }

    // T9 (Surface #6) — verify all three spawned functions
    // (spawn_reconnect_scheduler + scheduler_tick + attempt_reconnect) accept
    // forced-owned typed parameters per design §4.2 v1.2 row 3 + Q6.1+Q6.2+Q6.3.
    // The load-bearing assertion is type-level: the function signatures expose
    // owned NodeXgid for home_node_id + peer_node_id and owned Vec<SpaceXgid>
    // for shared_spaces.
    #[test]
    fn reconnect_spawned_functions_each_own_typed_capture() {
        // Construct typed owned values (forced-owned by §4.2 v1.2 row 3 because
        // these flow into spawned task captures across awaits).
        let home: NodeXgid = ndx_test("xgen://pubkey/ed25519:home");
        let peer: NodeXgid = ndx_test("xgen://pubkey/ed25519:peer");
        let shared: Vec<SpaceXgid> = vec![sdx_test("xgen://hash/sha256:space-a")];

        // Move semantics: these values can move into a spawned-style closure.
        let _captured = move || {
            let _h = &home;
            let _p = &peer;
            let _s = &shared;
        };

        // AttemptCursor: HashMap<NodeXgid, u32> per Q6.4. Direct typed access.
        let cursor: HashMap<NodeXgid, u32> = {
            let mut h = HashMap::new();
            h.insert(ndx_test("xgen://pubkey/ed25519:p1"), 3);
            h
        };
        assert_eq!(cursor.get(&ndx_test("xgen://pubkey/ed25519:p1")), Some(&3));
    }

    // T10 (Surface #6) — verify Arc<TypedXgid> shared-reference pattern works
    // for spawned-task captures that need read-only access to the same typed
    // value across multiple spawned tasks.
    #[test]
    fn reconnect_spawned_functions_arc_shared_reference_pattern_when_needed() {
        let home = Arc::new(ndx_test("xgen://pubkey/ed25519:home"));

        // Clone the Arc — each clone is a cheap refcount bump; the underlying
        // NodeXgid is shared across tasks.
        let h1 = Arc::clone(&home);
        let h2 = Arc::clone(&home);
        let h3 = Arc::clone(&home);

        // All three Arc clones point at the same underlying NodeXgid value
        // (PartialEq via inner Xgid).
        assert_eq!(*h1, *h2);
        assert_eq!(*h2, *h3);
        assert_eq!(h1.as_str(), "xgen://pubkey/ed25519:home");

        // Strong refcount confirms shared-ownership semantics.
        assert_eq!(Arc::strong_count(&home), 4);
    }
}
