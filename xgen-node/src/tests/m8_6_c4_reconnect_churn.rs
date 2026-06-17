// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! M8.6 C4 — Phase-5 reconnect scheduler under churn (design §5 C4, runbook §4).
//!
//! Two scheduler-direct tests built on the clock seam + the attempt-task gauge.
//!
//! `c4_attempt_task_gauge_returns_to_zero_after_timeouts` is THE spawn-leak
//! detector. A non-responsive (silent black-hole) peer + a MockClock + the tokio
//! paused clock: each tick spawns a detached attempt (gauge → 1); once the
//! connect-timeout elapses (advanced, not waited) the attempt resolves and the
//! gauge returns to 0. A hung attempt task (the M6 spawn-per-peer-per-tick leak)
//! would keep the gauge > 0 — red. The connect-timeout is load-bearing here
//! (without it the silent peer hangs the task forever), which is exactly what
//! the checkpoint-#3 sensitivity witness exercises.
//!
//! `c4_churn_x5_ladder_resets_and_peer_records_consistent` covers the ladder
//! math + invariants. Five failed reconnect ticks against a fast-failing peer
//! drive the backoff cursor 1→2→3→4→5(cap) with `next_reconnect_attempt` deltas
//! following 15/30/60/120/120 min; `peer_records` stays consistent with
//! `relationships` throughout; then a single successful reconnect against a
//! responsive mock receiver clears the cursor (the reset-on-ACTIVE that restarts
//! the ladder from the 15-min floor). Red: cursor mis-advances,
//! peer_records/relationships drift, or the cursor is not cleared on ACTIVE.

#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use tempfile::tempdir;
    use tokio::sync::Mutex;

    use xgen_common::clock::{Clock, MockClock};
    use xgen_common::xgid::NodeXgid;

    use crate::fanout::{ClientSenders, FederationPeerSenders};
    use crate::federation::federation_policy::FederationPolicyStore;
    use crate::identity::keypair;
    use crate::reconnect::{scheduler_tick, BACKOFF_LADDER_MINUTES, CONNECT_TIMEOUT_SECS};
    use crate::tests::reconnect_test_support::{
        advance_all, blank_runtime, ndx, pubkey_uri, registry_with_lost_peer, run_mock_receiver,
        silent_blackhole_listener,
    };
    use crate::transport::server::Server;

    /// Common scheduler-direct fixture: a MockClock-driven initiator runtime, an
    /// owned attempt_cursor + attempt_gauge, and the paths the tick needs.
    struct C4Fixture {
        clock: Arc<MockClock>,
        runtime: Arc<Mutex<crate::node::runtime::NodeRuntime>>,
        keypair: Arc<ed25519_dalek::SigningKey>,
        node_id: NodeXgid,
        client_senders: ClientSenders,
        fed_senders: FederationPeerSenders,
        registry: Arc<Mutex<crate::federation::registry::FederationRegistry>>,
        registry_path: std::path::PathBuf,
        spaces_dir: std::path::PathBuf,
        identities_path: std::path::PathBuf,
        cursor: Arc<Mutex<HashMap<NodeXgid, u32>>>,
        policy: Arc<Mutex<FederationPolicyStore>>,
        gauge: Arc<AtomicUsize>,
        _dir: tempfile::TempDir,
    }

    fn make_fixture(peer_id: &str, peer_url: &str) -> C4Fixture {
        let clock = Arc::new(MockClock::new());
        let (mut rt, init_key) = blank_runtime();
        rt.set_clock(clock.clone());
        let node_id = rt.node_id.clone();
        let runtime = Arc::new(Mutex::new(rt));
        let dir = tempdir().unwrap();
        let spaces_dir = dir.path().join("spaces");
        std::fs::create_dir_all(&spaces_dir).unwrap();
        C4Fixture {
            clock,
            runtime,
            keypair: Arc::new(init_key),
            node_id,
            client_senders: Arc::new(Mutex::new(HashMap::new())),
            fed_senders: Arc::new(Mutex::new(HashMap::new())),
            registry: Arc::new(Mutex::new(registry_with_lost_peer(peer_id, peer_url))),
            registry_path: dir.path().join("federation.json"),
            spaces_dir,
            identities_path: dir.path().join("identities.db"),
            cursor: Arc::new(Mutex::new(HashMap::new())),
            policy: Arc::new(Mutex::new(FederationPolicyStore::new())),
            gauge: Arc::new(AtomicUsize::new(0)),
            _dir: dir,
        }
    }

    impl C4Fixture {
        async fn tick(&self) {
            scheduler_tick(
                Arc::clone(&self.runtime),
                self.client_senders.clone(),
                Arc::clone(&self.fed_senders),
                Arc::clone(&self.registry),
                self.registry_path.clone(),
                self.keypair.clone(),
                self.node_id.clone(),
                self.spaces_dir.clone(),
                self.identities_path.clone(),
                true, // local_mode
                "ws://127.0.0.1:1/".to_string(),
                Arc::clone(&self.cursor),
                Arc::clone(&self.policy),
                Arc::clone(&self.gauge),
                // M12.3-D1 — blobs_dir + throwaway pending-fetch registry (no blob
                // fetch exercised by this M8.6 C4 churn test).
                self.spaces_dir.join("blobs"),
                Arc::new(Mutex::new(std::collections::HashMap::new())),
            )
            .await;
        }
    }

    // ── Test 1 — the spawn-leak gauge ─────────────────────────────────────────
    #[tokio::test(start_paused = true)]
    async fn c4_attempt_task_gauge_returns_to_zero_after_timeouts() {
        let (_blackhole, peer_url) = silent_blackhole_listener().await;
        let peer_id = pubkey_uri(&keypair::generate());
        let fx = make_fixture(&peer_id, &peer_url);
        let peer_typed = ndx(&peer_id);

        // Five churn cycles. Each: re-mark the peer lost (so it's due under the
        // advanced mock wall), tick (spawns a black-hole attempt → gauge 1),
        // advance past the connect-timeout, assert the gauge returns to 0.
        for cycle in 0..5 {
            if cycle > 0 {
                // Advance past the largest ladder step so the registry can be
                // re-marked due cleanly.
                advance_all(&fx.clock, Duration::from_secs(3 * 60 * 60)).await;
            }
            {
                let now = fx.clock.now_utc();
                let mut reg = fx.registry.lock().await;
                reg.mark_lost(&peer_typed, now - chrono::Duration::minutes(20));
            }

            fx.tick().await;

            // The AttemptGuard increments synchronously at the spawn site, so
            // the gauge is 1 the moment the tick returns (one due peer).
            assert_eq!(
                fx.gauge.load(Ordering::SeqCst),
                1,
                "cycle {cycle}: gauge must be 1 right after the tick spawns the black-hole attempt"
            );

            // Let the spawned attempt poll: connect to the black-hole (real
            // localhost TCP completes; the WS upgrade hangs) and — crucially —
            // register its connect-timeout timer. Advancing BEFORE the timer is
            // registered would place it past the advanced point and it would
            // never fire.
            for _ in 0..50 {
                tokio::task::yield_now().await;
            }

            // The silent peer never completes the WS upgrade, so ONLY the
            // connect-timeout can resolve the attempt. Advance past it.
            advance_all(&fx.clock, Duration::from_secs(CONNECT_TIMEOUT_SECS + 5)).await;

            // Let the woken attempt task run its early return + AttemptGuard drop.
            let mut zero = false;
            for _ in 0..400 {
                if fx.gauge.load(Ordering::SeqCst) == 0 {
                    zero = true;
                    break;
                }
                tokio::task::yield_now().await;
            }
            assert!(
                zero,
                "cycle {cycle}: gauge must return to 0 after the connect-timeout resolves the \
                 attempt — a value > 0 is a leaked attempt task (the M6 spawn-per-peer-per-tick bug)"
            );
        }
    }

    // ── Test 2 — ladder advancement + consistency + reset-on-ACTIVE ───────────
    #[tokio::test]
    async fn c4_churn_x5_ladder_resets_and_peer_records_consistent() {
        // ── Part A: ladder advancement against a fast-failing peer ────────────
        // peer_url is a closed localhost port → connect refused fast. The
        // attempts resolve immediately; we assert the synchronous cursor +
        // registry state the tick sets BEFORE spawning.
        let peer_id = pubkey_uri(&keypair::generate());
        let fx = make_fixture(&peer_id, "ws://127.0.0.1:1/");
        let peer_typed = ndx(&peer_id);

        for count in 1u32..=5 {
            // Make the peer due under the current mock wall (re-mark lost ~20 min
            // ago so next_reconnect is in the past), then snapshot `now` exactly
            // as the tick will read it.
            {
                let now = fx.clock.now_utc();
                let mut reg = fx.registry.lock().await;
                reg.mark_lost(&peer_typed, now - chrono::Duration::minutes(20));
            }
            let now_at_tick = fx.clock.now_utc();
            fx.tick().await;

            // (c) cursor invariant — advances 1→2→3→4→5 across the five ticks.
            let cursor_val = { fx.cursor.lock().await.get(&peer_typed).copied() };
            assert_eq!(
                cursor_val,
                Some(count),
                "tick #{count}: backoff cursor must read {count}"
            );

            // (a) ladder math — next_reconnect_attempt delta follows the ladder
            // (15/30/60/120/120) for cursor 1..=5 (the cap holds at idx 3).
            let idx = (count as usize).min(BACKOFF_LADDER_MINUTES.len()) - 1;
            let expected_min = BACKOFF_LADDER_MINUTES[idx];
            let (next_at, has_relationship) = {
                let reg = fx.registry.lock().await;
                let rec = reg
                    .peer_record(&peer_typed)
                    .expect("peer_record must exist after a scheduled tick");
                let next = rec
                    .next_reconnect_attempt
                    .clone()
                    .expect("tick must set next_reconnect_attempt");
                // (b) consistency — the relationship and the operational record
                // must both reference the peer; neither may drift away.
                (next, reg.get(&peer_typed).is_some())
            };
            assert!(
                has_relationship,
                "tick #{count}: peer must stay in BOTH peer_records and relationships (no drift)"
            );
            let parsed = chrono::DateTime::parse_from_rfc3339(&next_at)
                .expect("next_reconnect_attempt must be RFC3339");
            // Compare in seconds with a ±2 s tolerance: `next_at` is stored at
            // millisecond precision (floored), so a full-precision `15 min`
            // subtraction would truncate to 14 min under `num_minutes()`.
            let delta_secs = (parsed.with_timezone(&chrono::Utc) - now_at_tick).num_seconds();
            let expected_secs = expected_min * 60;
            assert!(
                (expected_secs - 2..=expected_secs + 2).contains(&delta_secs),
                "tick #{count}: next_reconnect_attempt delta must be ~{expected_min} min \
                 (ladder idx {idx}); got {delta_secs}s"
            );

            // Advance the mock wall past this step so the peer is due next tick.
            fx.clock
                .advance(Duration::from_secs((expected_min as u64 + 1) * 60));
        }

        // ── Part B: reset-on-ACTIVE clears the cursor ─────────────────────────
        // A responsive mock receiver completes the handshake; attempt_reconnect
        // reaches ACTIVE and clears the cursor (so a future disconnect restarts
        // the ladder from the 15-min floor). Real time (the handshake is real
        // I/O). Pre-seed the cursor non-zero to prove the clear happened.
        let (recv_rt, recv_key) = blank_runtime();
        let recv_id = pubkey_uri(&recv_key);
        let recv_runtime = Arc::new(Mutex::new(recv_rt));
        let recv_spaces = tempdir().unwrap();

        let server = Server::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let recv_url = format!("ws://{}/", server.local_addr());
        let recv_runtime_task = Arc::clone(&recv_runtime);
        let recv_key_task = recv_key.clone();
        let recv_spaces_path = recv_spaces.path().to_path_buf();
        let receiver = tokio::spawn(async move {
            run_mock_receiver(server, recv_runtime_task, recv_key_task, recv_spaces_path).await;
        });

        let fx2 = make_fixture(&recv_id, &recv_url);
        let recv_typed = ndx(&recv_id);
        // Pre-seed the cursor as if the ladder had already climbed to step 3.
        {
            fx2.cursor.lock().await.insert(recv_typed.clone(), 3);
        }

        fx2.tick().await;

        // Poll (real time) for the cursor to be cleared by the ACTIVE transition.
        let mut cleared = false;
        for _ in 0..200 {
            if fx2.cursor.lock().await.get(&recv_typed).is_none() {
                cleared = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        assert!(
            cleared,
            "after a successful reconnect (handshake-ACTIVE), the backoff cursor must be cleared \
             so the ladder restarts from the 15-min floor (reset-on-ACTIVE)"
        );

        receiver.abort();
    }
}
