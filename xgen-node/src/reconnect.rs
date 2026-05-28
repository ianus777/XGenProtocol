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
use std::sync::Arc;
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use tokio::sync::Mutex;

use xgen_common::xgid::{NodeXgid, SpaceXgid, Xgid};
use xgen_core::federation::handshake::run_initiating;
use xgen_core::transport::client::connect_url;
use xgen_core::wire::types::FederationCapabilities;

use crate::app::{run_federation_session_post_handshake, SessionRole};
use crate::fanout::{ClientSenders, FederationPeerSenders};
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
) {
    let attempt_cursor: AttemptCursor = Arc::new(Mutex::new(HashMap::new()));

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
) {
    let now = Utc::now();

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
#[allow(clippy::too_many_arguments)]
pub async fn attempt_reconnect(
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
) {
    // 1. Open WS to peer.
    let mut conn = match connect_url(&peer_url).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                peer_node_id = %peer_node_id.as_str(),
                peer_url = %peer_url,
                error = %e,
                "Reconnect: WS connect failed"
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

    // 5. Drive the post-handshake flow (catch-up drain, bilateral stream,
    //    register, F-2 loop, cleanup) via the shared driver.
    //
    // Pass 3 — project xgen-core's session.peer_node_id (String) to NodeXgid
    // for the typed downstream call. shared_spaces stays Vec<SpaceXgid>.
    let session_peer_node_id_typed =
        NodeXgid::from_xgid(Xgid::new(session.peer_node_id));
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
}
