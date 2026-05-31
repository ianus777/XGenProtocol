// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Bootstrap keepalive scheduler + best-effort re-advertise (bootstrap-client
//! arc, C4 — Pin 3 lock).
//!
//! A **separate** scheduler from the federation `reconnect.rs` (Pin 3): same
//! structure (`spawn_*` → an extracted `*_tick` for tests → per-due detached
//! spawn) but its own store, exchange, and timing. It refreshes this Node's
//! Bootstrap-Node directory entries before their 7-day TTL (spec §3.14.3,
//! WD-25) lapses, via the C2 `keepalive_bootstrap` send-path.
//!
//! It also hosts `readvertise_all` — the A3-D2 best-effort re-advertise the
//! `bootstrap set-info` verb invokes after its local write (a fan-out failure
//! never fails the verb or rolls back the local write).
//!
//! Spawned unconditionally at `run_node` (mirroring the reconnect scheduler);
//! an empty store makes every tick a no-op (no network) — the prime invariant
//! (registered with nobody = today byte-for-byte) is held by the early return
//! in `bootstrap_keepalive_tick`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use tokio::sync::Mutex;

use xgen_common::xgid::NodeXgid;
use xgen_core::bootstrap::registration_store::{BootstrapRegistrationStore, BootstrapSelfInfo};

use crate::bootstrap_client::{keepalive_bootstrap, register_with_bootstrap};

/// Scheduler tick interval (mirrors the reconnect scheduler's 60 s).
pub(crate) const BOOTSTRAP_KEEPALIVE_TICK_SECONDS: u64 = 60;

/// Refresh a registration when its TTL is within this lead window of expiring.
/// The directory TTL is 7 days (WD-25); refreshing a day ahead keeps the entry
/// alive with comfortable margin while a successful keepalive pushes `expires_at`
/// ~6 days out, so a given registration is due at most once per ~6 days.
pub(crate) const KEEPALIVE_REFRESH_LEAD_SECONDS: i64 = 24 * 60 * 60;

/// A registration is due for keepalive when it has no recorded expiry, or its
/// expiry is within (or past) the lead window. A malformed `expires_at` is
/// treated as due (refresh it to a well-formed value).
pub(crate) fn due_for_keepalive(expires_at: &Option<String>, now: DateTime<Utc>) -> bool {
    match expires_at {
        None => true,
        Some(s) => match DateTime::parse_from_rfc3339(s) {
            Ok(exp) => exp.with_timezone(&Utc) - now <= chrono::Duration::seconds(KEEPALIVE_REFRESH_LEAD_SECONDS),
            Err(_) => true,
        },
    }
}

/// Spawn the long-running keepalive scheduler. Ticks every
/// `BOOTSTRAP_KEEPALIVE_TICK_SECONDS`; no-ops while the store is empty.
pub fn spawn_bootstrap_keepalive_scheduler(
    store: Arc<Mutex<BootstrapRegistrationStore>>,
    store_path: PathBuf,
    node_keypair: Arc<SigningKey>,
    self_node_id: NodeXgid,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(BOOTSTRAP_KEEPALIVE_TICK_SECONDS)).await;
            bootstrap_keepalive_tick(
                Arc::clone(&store),
                store_path.clone(),
                Arc::clone(&node_keypair),
                self_node_id.clone(),
            )
            .await;
        }
    });
}

/// One scheduler tick — snapshot the due registrations, then spawn a detached
/// keepalive attempt for each (the tick never blocks on any one exchange).
/// Extracted from `spawn_bootstrap_keepalive_scheduler` so tests can fire it
/// directly without sleeping out the interval (sibling to `scheduler_tick`).
pub async fn bootstrap_keepalive_tick(
    store: Arc<Mutex<BootstrapRegistrationStore>>,
    store_path: PathBuf,
    node_keypair: Arc<SigningKey>,
    self_node_id: NodeXgid,
) {
    let now = Utc::now();
    let due: Vec<(NodeXgid, String)> = {
        let s = store.lock().await;
        s.all()
            .iter()
            .filter(|r| due_for_keepalive(&r.expires_at, now))
            .map(|r| (r.bootstrap_id.clone(), r.url.clone()))
            .collect()
    };

    if due.is_empty() {
        return; // prime invariant: empty / nothing-due = no network this tick
    }

    for (bootstrap_id, url) in due {
        let store = Arc::clone(&store);
        let store_path = store_path.clone();
        let node_keypair = Arc::clone(&node_keypair);
        let self_node_id = self_node_id.clone();
        tokio::spawn(async move {
            attempt_keepalive(store, store_path, node_keypair, self_node_id, bootstrap_id, url).await;
        });
    }
}

/// Send one keepalive; on the verified ack, store the refreshed TTL. A failure
/// is logged and left for the next tick to retry (best-effort).
async fn attempt_keepalive(
    store: Arc<Mutex<BootstrapRegistrationStore>>,
    store_path: PathBuf,
    node_keypair: Arc<SigningKey>,
    self_node_id: NodeXgid,
    bootstrap_id: NodeXgid,
    url: String,
) {
    match keepalive_bootstrap(&url, &bootstrap_id, &self_node_id, &node_keypair).await {
        Ok(new_expires_at) => {
            let mut s = store.lock().await;
            if let Some(reg) = s.get_mut(&bootstrap_id) {
                reg.expires_at = Some(new_expires_at);
            }
            if let Err(e) = s.save(&store_path) {
                tracing::warn!(
                    bootstrap_id = %bootstrap_id.as_str(),
                    error = %e,
                    "Bootstrap keepalive: store save failed"
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                bootstrap_id = %bootstrap_id.as_str(),
                url = %url,
                error = %e,
                "Bootstrap keepalive failed; will retry next tick"
            );
        }
    }
}

/// Best-effort re-advertise (A3-D2) — re-register the updated self-info with
/// every Bootstrap Node this Node holds. Invoked by `bootstrap set-info` after
/// its local write; a per-node failure is logged and skipped (the local write
/// already succeeded). A successful re-register resets that registration's
/// `directory_url` + TTL.
///
/// Re-advertise uses `register` (not `keepalive`) because only the `Register`
/// frame carries endpoint/region/capabilities (`keepalive` carries only
/// node_id). Tiers are NOT re-advertised — no wire frame carries them
/// (Checkpoint #1(d), Option A), so `set-tiers` has no re-advertise at all.
pub async fn readvertise_all(
    store: Arc<Mutex<BootstrapRegistrationStore>>,
    store_path: PathBuf,
    node_keypair: &SigningKey,
    self_node_id: &NodeXgid,
    self_info: &BootstrapSelfInfo,
) {
    let targets: Vec<(NodeXgid, String)> = {
        let s = store.lock().await;
        s.all().iter().map(|r| (r.bootstrap_id.clone(), r.url.clone())).collect()
    };

    for (bootstrap_id, url) in targets {
        match register_with_bootstrap(&url, &bootstrap_id, self_node_id, self_info, node_keypair).await {
            Ok(updated) => {
                let mut s = store.lock().await;
                s.add(updated);
                if let Err(e) = s.save(&store_path) {
                    tracing::warn!(
                        bootstrap_id = %bootstrap_id.as_str(),
                        error = %e,
                        "Bootstrap re-advertise: store save failed"
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    bootstrap_id = %bootstrap_id.as_str(),
                    url = %url,
                    error = %e,
                    "Bootstrap re-advertise failed (best-effort, A3-D2); local self-info already saved"
                );
            }
        }
    }
}

// ── Tests (pure due-logic) ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_when_no_expiry_recorded() {
        assert!(due_for_keepalive(&None, Utc::now()));
    }

    #[test]
    fn due_when_within_lead_window() {
        let now = Utc::now();
        // Expires in 12 h — inside the 24 h lead window → due.
        let soon = (now + chrono::Duration::hours(12)).to_rfc3339();
        assert!(due_for_keepalive(&Some(soon), now));
    }

    #[test]
    fn not_due_when_well_ahead() {
        let now = Utc::now();
        // Expires in 6 days — outside the lead window → not due (just refreshed).
        let later = (now + chrono::Duration::days(6)).to_rfc3339();
        assert!(!due_for_keepalive(&Some(later), now));
    }

    #[test]
    fn due_when_already_expired() {
        let now = Utc::now();
        let past = (now - chrono::Duration::hours(1)).to_rfc3339();
        assert!(due_for_keepalive(&Some(past), now));
    }

    #[test]
    fn malformed_expiry_is_due() {
        assert!(due_for_keepalive(&Some("not-a-timestamp".to_string()), Utc::now()));
    }
}
