// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Arc F (PG-11) — Space Migration node driver (C2).
//!
//! Wires the built-and-tested `xgen-core/src/migration/` core (`transition`,
//! the source/destination handlers, `transfer`, `verification`, and the
//! `apply_space_migrate` cutover applier) onto the node's transport + runtime.
//!
//! Two halves, each driving its per-session `MigrationState` through the pure
//! `transition()` sequence guard:
//!
//!   * [`run_source_migration`] — the SOURCE side. Spawned (detached) by the
//!     `migration initiate` operator verb. Opens an outbound WS to the
//!     destination, runs propose → transfer (+ tail) → transfer_complete, and
//!     on `migration.verified` performs the **cutover** (AF-D1): builds the
//!     `state.space_migrate` event, commits it to the source DAG (which flips
//!     the source's own `home_node` to the destination), ships it to the
//!     destination, pushes it to the Space's federated peers (AF-D8a — the
//!     peers' applier flips their `home_node`; `migration.federation_notify` is
//!     courtesy), and emits the per-member redirect signal (client UX OUT). The
//!     source **retains** its store (AF-D5 — teardown is a separate, operator-
//!     gated action, never automatic).
//!
//!   * [`handle_migration_incoming`] — the DESTINATION side. Reached from the
//!     `handle_connection` first-message branch when a peer's first post-auth
//!     message is a `migration.propose`. Evaluates admission (AF-D6 — accept
//!     unless already hosting; 6003/6004/6005 stay dormant), creates a fresh
//!     per-Space store (CP-4), accepts the event batches + tail, verifies, and
//!     applies the cutover event by re-deriving the Space (which flips its
//!     `home_node` to this node — it now hosts the Space as home).
//!
//! The migrate cutover event validates under the **old** `home_node`
//! (`validate_event`'s Node-authority gate, AF-D2) before any applier installs
//! the new anchor — see `xgen-core/src/message/exchange.rs`.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Mutex;

use xgen_common::xgid::{NodeXgid, SpaceXgid, Xgid};
use xgen_core::{
    migration::{
        state_machine::{
            handle_migration_propose, handle_verified, transition, DestinationResponse,
            MigrationMsgKind, MigrationState,
        },
        transfer::{batch_events, compute_batch_hash, identify_tail},
        verification::verify_transfer,
    },
    node::runtime::{DispatchOutcome, EventOrigin, NodeRuntime},
    transport::{
        client::connect_url,
        connection::{Connection, Inbound},
    },
    wire::types::{Event, EventType, MigrationMessage},
};

use crate::fanout::{ClientSenders, FederationPeerSenders};

const PROTOCOL_VERSION: &str = "0.1";

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

// ── Source side ─────────────────────────────────────────────────────────────

/// Drive a complete source-side migration of `space_id` to the destination Node.
///
/// Runs as a detached task (spawned by the `migration initiate` operator verb).
/// All failures are logged and abandon the migration; the source Space is left
/// untouched until the cutover commits (so an aborted migration is a no-op).
#[allow(clippy::too_many_arguments)]
pub async fn run_source_migration(
    runtime: Arc<Mutex<NodeRuntime>>,
    node_keypair: Arc<SigningKey>,
    home_node_id: NodeXgid,
    spaces_dir: PathBuf,
    federation_peer_senders: FederationPeerSenders,
    federation_policy: Arc<Mutex<crate::federation::federation_policy::FederationPolicyStore>>,
    space_id: String,
    destination_node_id: String,
    destination_node_url: String,
) {
    let sx = SpaceXgid::from_xgid(Xgid::new(space_id.clone()));
    let mut state = MigrationState::Idle;

    // 1. Connect + transport-auth as this Node.
    let mut conn = match connect_url(&destination_node_url).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(space_id = %space_id, dst = %destination_node_url, error = %e, "migration: connect to destination failed");
            return;
        }
    };
    if let Err(e) = conn.client_authenticate(&node_keypair).await {
        tracing::warn!(space_id = %space_id, error = %e, "migration: destination authenticate failed");
        return;
    }

    // The operator's initiate is the `migration.request` trigger (Idle → Negotiating).
    state = match transition(&state, MigrationMsgKind::Request) {
        Ok(s) => s,
        Err(_) => return,
    };

    // 2. Snapshot the Space's event log + owner + tier for the proposal.
    let (owner_id, auth_tier, snapshot): (String, u32, Vec<Event>) = {
        let rt = runtime.lock().await;
        let st = match rt.spaces.get(&sx) {
            Some(s) => s,
            None => {
                tracing::warn!(space_id = %space_id, "migration: Space not hosted here — aborting");
                return;
            }
        };
        let owner = st.owner_id.as_str().to_string();
        let tier = st.auth_tier;
        let events = rt
            .stores
            .get(&sx)
            .map(|s| s.range(0).unwrap_or_default())
            .unwrap_or_default();
        (owner, tier, events)
    };
    let snapshot_ids: HashSet<String> = snapshot
        .iter()
        .filter_map(|e| e.event_id.as_ref().map(|x| x.as_str().to_string()))
        .collect();
    let estimated_size_bytes = snapshot
        .iter()
        .map(|e| serde_json::to_vec(e).map(|v| v.len()).unwrap_or(0) as u64)
        .sum::<u64>();

    // 3. Propose → expect Accept.
    if conn
        .send_migration(&MigrationMessage::Propose {
            protocol_version: PROTOCOL_VERSION.to_string(),
            space_id: space_id.clone(),
            source_node_id: home_node_id.as_str().to_string(),
            space_auth_tier: auth_tier,
            event_count: snapshot.len() as u64,
            estimated_size_bytes,
            owner_id,
            timestamp: now(),
            signature: None,
        })
        .await
        .is_err()
    {
        tracing::warn!(space_id = %space_id, "migration: send propose failed");
        return;
    }
    match conn.recv().await {
        Ok(Inbound::Migration(MigrationMessage::Accept { .. })) => {
            state = transition(&state, MigrationMsgKind::Accept).unwrap_or(state);
        }
        Ok(Inbound::Migration(MigrationMessage::Reject { reason, .. })) => {
            tracing::info!(space_id = %space_id, %reason, "migration: destination rejected proposal");
            return;
        }
        other => {
            tracing::warn!(space_id = %space_id, ?other, "migration: unexpected reply to propose");
            return;
        }
    }

    // 4. Transfer the snapshot in batches; each batch is acked.
    let batches = batch_events(snapshot.clone());
    let batch_count = batches.len();
    for (i, batch) in batches.into_iter().enumerate() {
        if !send_batch_and_await_ack(&mut conn, &space_id, i as u64, batch).await {
            tracing::warn!(space_id = %space_id, batch = i, "migration: batch transfer failed");
            return;
        }
    }

    // 5. Tail — any events produced during the transfer window (3.12.5).
    let (all_events, total_events, tips) = {
        let rt = runtime.lock().await;
        let events = rt
            .stores
            .get(&sx)
            .map(|s| s.range(0).unwrap_or_default())
            .unwrap_or_default();
        let tips = rt.dag_tips(&sx);
        (events.clone(), events.len(), tips)
    };
    let tail: Vec<Event> = identify_tail(&all_events, &snapshot_ids)
        .into_iter()
        .cloned()
        .collect();
    if !tail.is_empty()
        && !send_batch_and_await_ack(&mut conn, &space_id, batch_count as u64, tail).await
    {
        tracing::warn!(space_id = %space_id, "migration: tail transfer failed");
        return;
    }

    // 6. Transfer complete → expect Verified.
    if conn
        .send_migration(&MigrationMessage::TransferComplete {
            protocol_version: PROTOCOL_VERSION.to_string(),
            space_id: space_id.clone(),
            total_events: total_events as u64,
            dag_tips: tips.clone(),
            timestamp: now(),
            signature: None,
        })
        .await
        .is_err()
    {
        return;
    }
    state = transition(&state, MigrationMsgKind::TransferComplete).unwrap_or(state);
    match conn.recv().await {
        Ok(Inbound::Migration(MigrationMessage::Verified { .. })) => {
            state = transition(&state, MigrationMsgKind::Verified).unwrap_or(state);
        }
        Ok(Inbound::Migration(MigrationMessage::VerificationFailed { reason, .. })) => {
            tracing::warn!(space_id = %space_id, %reason, "migration: destination verification failed");
            return;
        }
        other => {
            tracing::warn!(space_id = %space_id, ?other, "migration: unexpected reply to transfer_complete");
            return;
        }
    }
    debug_assert_eq!(state, MigrationState::Complete);

    // 7. Cutover (AF-D1) — build the migrate event, commit it locally (flips the
    //    source's own home_node), ship it to the destination, push to peers.
    let cutover = {
        let rt = runtime.lock().await;
        let st = match rt.spaces.get(&sx) {
            Some(s) => s,
            None => return,
        };
        handle_verified(
            st,
            &node_keypair,
            &destination_node_id,
            &destination_node_url,
            tips.clone(),
            &now(),
        )
    };
    let migrate = cutover.space_migrate_event;

    // Commit to the source DAG: validates under the old home_node (AF-D2),
    // then flips this node's home_node to the destination.
    {
        let mut rt = runtime.lock().await;
        match rt.dispatch_event(migrate.clone(), EventOrigin::LocallySubmitted, None) {
            DispatchOutcome::Accepted { .. } => {
                if let Err(e) = crate::app::persist_event(&spaces_dir, &space_id, &migrate) {
                    tracing::error!(space_id = %space_id, error = %e, "migration: persist cutover event failed");
                }
            }
            outcome => {
                tracing::error!(space_id = %space_id, ?outcome, "migration: cutover dispatch rejected — aborting");
                return;
            }
        }
    }

    // Ship the cutover event to the destination so it flips its home_node too.
    if !send_batch_and_await_ack(
        &mut conn,
        &space_id,
        (batch_count + 1) as u64,
        vec![migrate.clone()],
    )
    .await
    {
        tracing::warn!(space_id = %space_id, "migration: cutover event delivery to destination failed");
    }

    // Federation-notify (AF-D8a) — reuse the existing push path. The migrate is
    // a LocallySubmitted DAG event, so it fans out to the Space's federated
    // peers, whose applier flips their home_node. `migration.federation_notify`
    // is courtesy only and is subsumed by this applier reuse.
    crate::federation_session::apply_federation_push(
        &migrate,
        EventOrigin::LocallySubmitted,
        &runtime,
        &federation_peer_senders,
        &home_node_id,
        Some(&federation_policy),
    )
    .await;

    // Per-member redirect (3.12.7) — emit-only signal; client redirect UX is OUT
    // of this arc's scope.
    tracing::info!(
        space_id = %space_id,
        members = cutover.member_ids.len(),
        dst = %destination_node_id,
        "migration: cutover complete — redirect signal emitted (client UX deferred)"
    );

    // Retention (AF-D5) — the source keeps its store. Teardown is a separate,
    // operator-gated action and is intentionally NOT performed here.

    let _ = conn.goodbye("migration complete").await;
}

/// Send one `migration.event_batch` and block on the matching `migration.batch_ack`.
async fn send_batch_and_await_ack<S>(
    conn: &mut Connection<S>,
    space_id: &str,
    batch_index: u64,
    events: Vec<Event>,
) -> bool
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let batch_hash = compute_batch_hash(&events);
    if conn
        .send_migration(&MigrationMessage::EventBatch {
            protocol_version: PROTOCOL_VERSION.to_string(),
            space_id: space_id.to_string(),
            batch_index,
            events,
            batch_hash,
            timestamp: now(),
            signature: None,
        })
        .await
        .is_err()
    {
        return false;
    }
    matches!(
        conn.recv().await,
        Ok(Inbound::Migration(MigrationMessage::BatchAck { .. }))
    )
}

// ── Destination side ────────────────────────────────────────────────────────

/// Run the destination-side migration session. `first` is the opening
/// `migration.propose` already read off the connection by `handle_connection`.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_migration_incoming<S>(
    conn: &mut Connection<S>,
    first: MigrationMessage,
    runtime: Arc<Mutex<NodeRuntime>>,
    spaces_dir: PathBuf,
    _client_senders: ClientSenders,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let mut state = MigrationState::Idle;

    let space_id = match &first {
        MigrationMessage::Propose { space_id, .. } => space_id.clone(),
        other => {
            tracing::warn!(?other, "migration: first message was not a propose");
            return;
        }
    };
    let sx = SpaceXgid::from_xgid(Xgid::new(space_id.clone()));
    state = transition(&state, MigrationMsgKind::Propose).unwrap_or(state);

    // Admission (AF-D6) — accept unless already hosting. 6003/6004/6005 dormant.
    let existing: Vec<String> = {
        let rt = runtime.lock().await;
        rt.spaces.keys().map(|k| k.as_str().to_string()).collect()
    };
    match handle_migration_propose(&space_id, &existing) {
        DestinationResponse::Reject { reason } => {
            tracing::info!(space_id = %space_id, %reason, "migration: rejecting proposal");
            let _ = conn
                .send_migration(&MigrationMessage::Reject {
                    protocol_version: PROTOCOL_VERSION.to_string(),
                    space_id,
                    reason: reason.to_string(),
                    timestamp: now(),
                    signature: None,
                })
                .await;
            return;
        }
        DestinationResponse::Accept => {
            // CP-4 — fresh per-Space store via the runtime's factory.
            {
                let mut rt = runtime.lock().await;
                if let Err(e) = rt.ensure_store(&sx) {
                    tracing::error!(space_id = %space_id, error = %e, "migration: ensure_store failed");
                    return;
                }
            }
            state = transition(&state, MigrationMsgKind::Accept).unwrap_or(state);
            if conn
                .send_migration(&MigrationMessage::Accept {
                    protocol_version: PROTOCOL_VERSION.to_string(),
                    space_id: space_id.clone(),
                    timestamp: now(),
                    signature: None,
                })
                .await
                .is_err()
            {
                return;
            }
        }
    }

    // Transfer + verify + cutover loop.
    loop {
        let msg = match conn.recv().await {
            Ok(Inbound::Migration(m)) => m,
            Ok(Inbound::Closed) | Err(_) => break,
            Ok(_) => continue,
        };
        match msg {
            MigrationMessage::EventBatch {
                batch_index,
                events,
                ..
            } => {
                let has_cutover = events
                    .iter()
                    .any(|e| e.event_type == EventType::StateSpaceMigrate);
                {
                    let mut rt = runtime.lock().await;
                    for ev in &events {
                        if let Some(store) = rt.stores.get_mut(&sx) {
                            let _ = store.append(ev.clone());
                        }
                        let _ = crate::app::persist_event(&spaces_dir, &space_id, ev);
                    }
                    // Re-derive on the cutover batch so home_node flips to here.
                    if has_cutover {
                        rt.rehydrate_space_from_store(&sx);
                    }
                }
                let _ = conn
                    .send_migration(&MigrationMessage::BatchAck {
                        protocol_version: PROTOCOL_VERSION.to_string(),
                        space_id: space_id.clone(),
                        batch_index,
                        timestamp: now(),
                        signature: None,
                    })
                    .await;
            }
            MigrationMessage::TransferComplete {
                total_events,
                dag_tips,
                ..
            } => {
                // Build the destination SpaceState from the transferred log.
                let (actual_count, actual_tips) = {
                    let mut rt = runtime.lock().await;
                    rt.rehydrate_space_from_store(&sx);
                    let count = rt.stores.get(&sx).map(|s| s.len()).unwrap_or(0);
                    (count, rt.dag_tips(&sx))
                };
                let ok = verify_transfer(
                    actual_count,
                    &actual_tips,
                    total_events as usize,
                    &dag_tips,
                )
                .is_ok();
                if ok {
                    state = transition(&state, MigrationMsgKind::TransferComplete).unwrap_or(state);
                    state = transition(&state, MigrationMsgKind::Verified).unwrap_or(state);
                    let _ = conn
                        .send_migration(&MigrationMessage::Verified {
                            protocol_version: PROTOCOL_VERSION.to_string(),
                            space_id: space_id.clone(),
                            timestamp: now(),
                            signature: None,
                        })
                        .await;
                } else {
                    tracing::warn!(space_id = %space_id, "migration: verification failed (count/tip mismatch)");
                    let _ = conn
                        .send_migration(&MigrationMessage::VerificationFailed {
                            protocol_version: PROTOCOL_VERSION.to_string(),
                            space_id: space_id.clone(),
                            reason: "count_or_tip_mismatch".to_string(),
                            timestamp: now(),
                            signature: None,
                        })
                        .await;
                    break;
                }
            }
            MigrationMessage::Failed { reason, .. } => {
                tracing::info!(space_id = %space_id, %reason, "migration: source reported failure");
                break;
            }
            _ => {}
        }
    }
}

/// Validate that a Space is hosted here as its home Node (the operator-verb
/// precondition for initiating a migration). Returns the owner id on success.
pub async fn require_hosted_home_space(
    runtime: &Arc<Mutex<NodeRuntime>>,
    home_node_id: &NodeXgid,
    space_id: &str,
) -> Result<String, &'static str> {
    let sx = SpaceXgid::from_xgid(Xgid::new(space_id.to_string()));
    let rt = runtime.lock().await;
    match rt.spaces.get(&sx) {
        Some(st) if st.home_node.as_str() == home_node_id.as_str() => {
            Ok(st.owner_id.as_str().to_string())
        }
        Some(_) => Err("space is not homed on this node"),
        None => Err("space not found on this node"),
    }
}
