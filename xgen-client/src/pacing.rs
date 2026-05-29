// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Client-side outbound pacing queue (spec Ch3 §3.7.12, Ch6 §6.14).
//
// One queue per (space_id, sender_identity_id) pair. FIFO. In-memory only.
// Caller drives time via the `attempt_send` API — no embedded timer; the
// release schedule is exposed so the harness (CLI scheduler, Tauri timer task)
// can wake at `release_at_ms` and call `drain_ready`. This keeps the core
// logic deterministic and unit-testable.

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use xgen_common::wire::{DEFAULT_AI_PACING_MS, DEFAULT_HUMAN_PACING_MS};
use xgen_common::xgid::{IdentityXgid, SpaceXgid, Xgid};

/// Pacing rules for one Space, sourced from `SpaceState.human_pacing_ms` /
/// `ai_pacing_ms`. Both fields are integers in milliseconds; zero disables
/// pacing for that member class (Ch6 §6.14.6 "Cap of zero").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpacePacing {
    pub human_pacing_ms: u64,
    pub ai_pacing_ms: u64,
}

impl Default for SpacePacing {
    fn default() -> Self {
        Self {
            human_pacing_ms: DEFAULT_HUMAN_PACING_MS,
            ai_pacing_ms: DEFAULT_AI_PACING_MS,
        }
    }
}

impl SpacePacing {
    /// Returns the cap (ms) for the given member class (Ch6 §6.14.1).
    pub fn cap_for(self, is_ai: bool) -> u64 {
        if is_ai {
            self.ai_pacing_ms
        } else {
            self.human_pacing_ms
        }
    }
}

/// Outcome of an `attempt_send` call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SendDecision {
    /// The message MAY be sent immediately. `last_send_at_ms` has been updated.
    SendNow,
    /// The message is enqueued. The caller MUST wait until `release_at_ms`
    /// (epoch milliseconds) before calling `drain_ready`.
    Queued { release_at_ms: u64 },
}

/// Per-(space, sender) queue state.
#[derive(Debug, Clone, Default)]
struct MemberQueueState {
    /// Last actual send time (epoch ms). `None` if the member has never sent
    /// — distinct from `Some(0)` (a legitimate, if implausible, real send at
    /// the Unix epoch), which preserves cap-of-zero correctness in tests.
    last_send_at_ms: Option<u64>,
    /// Release timestamps for queued messages, in FIFO order. Each entry is
    /// the `release_at_ms` value returned to the caller from `attempt_send`.
    queued: VecDeque<u64>,
    /// Cap (ms) selected at the time the most recent message entered the
    /// queue. Per Ch6 §6.14.1 the cap is selected at queue-entry time, so a
    /// pacing update mid-queue does not retroactively change wait times.
    last_cap_ms: u64,
    /// `is_ai` flag for the sender (selected once and stable for the
    /// lifetime of the Identity per spec 3.6.10.2).
    is_ai: bool,
}

/// Read-only snapshot of one queue's state — JSON-serialisable so the UI
/// layer can consume it via Tauri `invoke()` (Ch6 §6.14.3 / §6.14.4 DOM
/// contract source values).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacingState {
    pub space_id: SpaceXgid,
    pub sender_identity_id: IdentityXgid,
    /// Cap currently selected for this sender (ms).
    pub cap_ms: u64,
    /// Number of messages in queue (waiting to release).
    pub queue_count: usize,
    /// Milliseconds until the next queued message releases. `0` if queue empty
    /// or its head is already past its release time.
    pub time_to_next_send_ms: u64,
    /// Total estimated drain time (ms) — wall-clock until the last queued
    /// message would release.
    pub drain_ms: u64,
    /// Whether the active member is an AI (selects which cap applies).
    pub is_ai: bool,
}

/// Outbound pacing queue manager — thread-safe-by-construction (single owner)
/// holder for per-(space, sender) queue state and per-space pacing rules.
///
/// Time is supplied by the caller (`now_ms`). The manager never reads the
/// wall clock itself, which keeps the logic deterministic in tests and lets
/// the host system (Tauri timer task, CLI loop) own the clock concern.
#[derive(Debug, Default)]
pub struct PacingManager {
    /// Per-space pacing rules. Missing entries are treated as
    /// `SpacePacing::default()` (Ch6 §6.14.6 "No pacing rules in Space state").
    space_rules: HashMap<SpaceXgid, SpacePacing>,
    /// Per-(space_id, sender_identity_id) queue state.
    queues: HashMap<(SpaceXgid, IdentityXgid), MemberQueueState>,
}

impl PacingManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Install or replace the pacing rules for `space_id`. Typically called
    /// from the SpaceState replay loop whenever a `state.space_create` or
    /// `state.space_pacing` Event lands.
    pub fn set_space_rules(&mut self, space_id: &str, rules: SpacePacing) {
        self.space_rules.insert(SpaceXgid::from_xgid(Xgid::new(space_id.to_string())), rules);
    }

    /// Return the active pacing rules for a Space; falls back to defaults if
    /// no rules have been installed (Ch6 §6.14.6).
    pub fn rules_for(&self, space_id: &str) -> SpacePacing {
        self.space_rules
            .get(space_id)
            .copied()
            .unwrap_or_default()
    }

    /// Attempt to send a message right now. `now_ms` is the current time in
    /// epoch milliseconds. `is_ai` is the sender's declared classification
    /// (spec 3.6.10.2 — immutable for the lifetime of the Identity).
    ///
    /// On `SendNow`, the manager records `now_ms` as the new `last_send_at`.
    /// On `Queued`, the manager appends a release timestamp to the FIFO and
    /// returns it; the caller MUST eventually call `drain_ready` at or after
    /// that timestamp to release the queued message.
    pub fn attempt_send(
        &mut self,
        space_id: &str,
        sender_identity_id: &str,
        is_ai: bool,
        now_ms: u64,
    ) -> SendDecision {
        let rules = self.rules_for(space_id);
        let cap_ms = rules.cap_for(is_ai);
        let key = (
            SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
            IdentityXgid::from_xgid(Xgid::new(sender_identity_id.to_string())),
        );
        let state = self.queues.entry(key).or_default();
        state.is_ai = is_ai;

        // Cap-of-zero pass-through (Ch6 §6.14.6).
        if cap_ms == 0 {
            state.last_send_at_ms = Some(now_ms);
            state.last_cap_ms = 0;
            return SendDecision::SendNow;
        }

        // If there are already queued messages, the new message must release
        // after the last queued release (FIFO, sequential drain per §6.14.2).
        if let Some(&last_release) = state.queued.back() {
            let next_release = last_release.saturating_add(cap_ms);
            state.queued.push_back(next_release);
            state.last_cap_ms = cap_ms;
            return SendDecision::Queued {
                release_at_ms: next_release,
            };
        }

        // Empty queue: compute elapsed since last actual send.
        // Negative skew is clamped to zero (§6.14.6 "Clock skew").
        // First-ever send (`None`) always releases immediately.
        let last_send_at = match state.last_send_at_ms {
            Some(t) => t,
            None => {
                state.last_send_at_ms = Some(now_ms);
                state.last_cap_ms = cap_ms;
                return SendDecision::SendNow;
            }
        };
        let elapsed = now_ms.saturating_sub(last_send_at);
        if elapsed >= cap_ms {
            state.last_send_at_ms = Some(now_ms);
            state.last_cap_ms = cap_ms;
            return SendDecision::SendNow;
        }

        // Throttled: release_at = last_send_at + cap_ms (§6.14.2).
        let release_at = last_send_at.saturating_add(cap_ms);
        state.queued.push_back(release_at);
        state.last_cap_ms = cap_ms;
        SendDecision::Queued {
            release_at_ms: release_at,
        }
    }

    /// Release all messages whose `release_at_ms` is `<= now_ms`. Returns the
    /// count of messages released (the caller is expected to dispatch that
    /// many messages from its own outbound FIFO).
    ///
    /// `last_send_at` is advanced to the most recent release timestamp so the
    /// next `attempt_send` after the burst respects the cap from the last
    /// actual release, not from the wall clock.
    pub fn drain_ready(&mut self, space_id: &str, sender_identity_id: &str, now_ms: u64) -> usize {
        let key = (
            SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
            IdentityXgid::from_xgid(Xgid::new(sender_identity_id.to_string())),
        );
        let state = match self.queues.get_mut(&key) {
            Some(s) => s,
            None => return 0,
        };
        let mut released = 0usize;
        while let Some(&head) = state.queued.front() {
            if head > now_ms {
                break;
            }
            state.queued.pop_front();
            state.last_send_at_ms = Some(head);
            released += 1;
        }
        released
    }

    /// Read-only snapshot of one (space, sender) queue. Returns `None` if no
    /// activity has been recorded for that pair.
    pub fn snapshot(
        &self,
        space_id: &str,
        sender_identity_id: &str,
        now_ms: u64,
    ) -> Option<PacingState> {
        let key = (
            SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
            IdentityXgid::from_xgid(Xgid::new(sender_identity_id.to_string())),
        );
        let state = self.queues.get(&key)?;
        let cap_ms = self.rules_for(space_id).cap_for(state.is_ai);
        let queue_count = state.queued.len();
        let time_to_next_send_ms = state
            .queued
            .front()
            .map(|head| head.saturating_sub(now_ms))
            .unwrap_or(0);
        let drain_ms = state
            .queued
            .back()
            .map(|tail| tail.saturating_sub(now_ms))
            .unwrap_or(0);
        Some(PacingState {
            space_id: SpaceXgid::from_xgid(Xgid::new(space_id.to_string())),
            sender_identity_id: IdentityXgid::from_xgid(Xgid::new(sender_identity_id.to_string())),
            cap_ms,
            queue_count,
            time_to_next_send_ms,
            drain_ms,
            is_ai: state.is_ai,
        })
    }

    /// All queue snapshots for a Space (across every sender that has activity
    /// recorded). Convenience for the Tauri command surface (Ch6 §6.14.4).
    pub fn snapshots_for_space(&self, space_id: &str, now_ms: u64) -> Vec<PacingState> {
        self.queues
            .iter()
            .filter_map(|((sp, sender), _)| {
                if sp.as_str() == space_id {
                    self.snapshot(sp.as_str(), sender.as_str(), now_ms)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPACE_A: &str = "xgen://hash/sha256:space_a";
    const ALICE: &str = "xgen://pubkey/ed25519:ALICE";
    const BOT: &str = "xgen://pubkey/ed25519:BOT";

    #[test]
    fn first_send_releases_immediately() {
        let mut mgr = PacingManager::new();
        let d = mgr.attempt_send(SPACE_A, ALICE, false, 10_000);
        assert_eq!(d, SendDecision::SendNow);
    }

    #[test]
    fn second_send_within_cap_is_queued() {
        let mut mgr = PacingManager::new();
        // Default human cap = 500 ms.
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 10_000);
        let d = mgr.attempt_send(SPACE_A, ALICE, false, 10_100);
        match d {
            SendDecision::Queued { release_at_ms } => {
                assert_eq!(release_at_ms, 10_500, "release at last_send + cap");
            }
            other => panic!("expected Queued, got {other:?}"),
        }
    }

    #[test]
    fn second_send_after_cap_releases_immediately() {
        let mut mgr = PacingManager::new();
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 10_000);
        let d = mgr.attempt_send(SPACE_A, ALICE, false, 10_500);
        assert_eq!(d, SendDecision::SendNow);
    }

    #[test]
    fn burst_drains_sequentially() {
        // Five messages back-to-back; first releases immediately, the
        // remaining four schedule at 500 ms intervals (default cap).
        let mut mgr = PacingManager::new();
        let d0 = mgr.attempt_send(SPACE_A, ALICE, false, 0);
        assert_eq!(d0, SendDecision::SendNow);
        let releases: Vec<u64> = (1..=4)
            .map(|i| {
                match mgr.attempt_send(SPACE_A, ALICE, false, i * 10) {
                    SendDecision::Queued { release_at_ms } => release_at_ms,
                    other => panic!("expected Queued at step {i}, got {other:?}"),
                }
            })
            .collect();
        // Default cap = 500 ms, so sequential releases at 500, 1000, 1500, 2000.
        assert_eq!(releases, vec![500, 1000, 1500, 2000]);
    }

    #[test]
    fn clock_skew_treated_as_zero_elapsed() {
        // last_send_at = 10_000; now_ms goes backward to 9_500.
        // saturating_sub yields 0 → throttled (0 < 500), queued at last_send + cap.
        let mut mgr = PacingManager::new();
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 10_000);
        let d = mgr.attempt_send(SPACE_A, ALICE, false, 9_500);
        match d {
            SendDecision::Queued { release_at_ms } => {
                assert_eq!(release_at_ms, 10_500);
            }
            other => panic!("expected Queued under clock skew, got {other:?}"),
        }
    }

    #[test]
    fn zero_cap_passes_through_immediately() {
        let mut mgr = PacingManager::new();
        mgr.set_space_rules(
            SPACE_A,
            SpacePacing {
                human_pacing_ms: 0,
                ai_pacing_ms: 0,
            },
        );
        let d1 = mgr.attempt_send(SPACE_A, ALICE, false, 1000);
        let d2 = mgr.attempt_send(SPACE_A, ALICE, false, 1001);
        let d3 = mgr.attempt_send(SPACE_A, ALICE, false, 1002);
        assert_eq!(d1, SendDecision::SendNow);
        assert_eq!(d2, SendDecision::SendNow);
        assert_eq!(d3, SendDecision::SendNow);
    }

    #[test]
    fn ai_uses_ai_cap() {
        let mut mgr = PacingManager::new();
        // Defaults: human 500, AI 2000. AI's second send within 1500 ms is queued.
        let _ = mgr.attempt_send(SPACE_A, BOT, true, 10_000);
        let d = mgr.attempt_send(SPACE_A, BOT, true, 11_500);
        match d {
            SendDecision::Queued { release_at_ms } => {
                assert_eq!(release_at_ms, 12_000, "AI cap = 2000ms");
            }
            other => panic!("expected Queued for AI sender, got {other:?}"),
        }
    }

    #[test]
    fn ai_unknown_falls_back_to_human_cap() {
        // §6.14.6: if is_ai unknown at send time, the client falls back to
        // human_pacing_ms (the lighter throttle).
        let mut mgr = PacingManager::new();
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 10_000);
        let d = mgr.attempt_send(SPACE_A, ALICE, false, 10_100);
        match d {
            SendDecision::Queued { release_at_ms } => {
                assert_eq!(release_at_ms, 10_500);
            }
            other => panic!("expected Queued, got {other:?}"),
        }
    }

    #[test]
    fn missing_space_rules_uses_defaults() {
        // §6.14.6: legacy Space without explicit pacing → Ch6 defaults.
        let mgr = PacingManager::new();
        let rules = mgr.rules_for("xgen://hash/sha256:unknown_space");
        assert_eq!(rules.human_pacing_ms, DEFAULT_HUMAN_PACING_MS);
        assert_eq!(rules.ai_pacing_ms, DEFAULT_AI_PACING_MS);
    }

    #[test]
    fn drain_ready_releases_only_due_messages() {
        let mut mgr = PacingManager::new();
        // Build a queue of three (default human cap = 500 ms).
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 0); // SendNow at t=0
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 10); // queued at 500
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 20); // queued at 1000
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 30); // queued at 1500

        // At t=700, only the first queued message (release_at=500) is due.
        let drained = mgr.drain_ready(SPACE_A, ALICE, 700);
        assert_eq!(drained, 1);

        // At t=1600, both remaining are due.
        let drained = mgr.drain_ready(SPACE_A, ALICE, 1600);
        assert_eq!(drained, 2);

        // Queue is now empty.
        let drained = mgr.drain_ready(SPACE_A, ALICE, 9999);
        assert_eq!(drained, 0);
    }

    #[test]
    fn snapshot_reflects_queue_state() {
        let mut mgr = PacingManager::new();
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 0);
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 10);
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 20);
        let snap = mgr.snapshot(SPACE_A, ALICE, 100).unwrap();
        assert_eq!(snap.cap_ms, DEFAULT_HUMAN_PACING_MS);
        assert_eq!(snap.queue_count, 2);
        assert_eq!(snap.time_to_next_send_ms, 400); // head releases at 500
        assert_eq!(snap.drain_ms, 900); // tail releases at 1000
        assert!(!snap.is_ai);
    }

    #[test]
    fn snapshot_reports_zero_when_no_queue() {
        let mut mgr = PacingManager::new();
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 0);
        let snap = mgr.snapshot(SPACE_A, ALICE, 5000).unwrap();
        assert_eq!(snap.queue_count, 0);
        assert_eq!(snap.time_to_next_send_ms, 0);
        assert_eq!(snap.drain_ms, 0);
    }

    #[test]
    fn snapshots_for_space_returns_per_sender_entries() {
        let mut mgr = PacingManager::new();
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 0);
        let _ = mgr.attempt_send(SPACE_A, BOT, true, 0);
        let snaps = mgr.snapshots_for_space(SPACE_A, 100);
        assert_eq!(snaps.len(), 2);
    }

    #[test]
    fn pacing_state_serialises_to_json() {
        // Confirms the type is suitable for Tauri invoke() return values.
        let snap = PacingState {
            space_id: SpaceXgid::from_xgid(Xgid::new(SPACE_A.to_string())),
            sender_identity_id: IdentityXgid::from_xgid(Xgid::new(BOT.to_string())),
            cap_ms: 2000,
            queue_count: 3,
            time_to_next_send_ms: 850,
            drain_ms: 7150,
            is_ai: true,
        };
        let json = serde_json::to_string(&snap).unwrap();
        let parsed: PacingState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, snap);
    }

    #[test]
    fn queues_are_isolated_per_space() {
        // Spec 3.7.12.6: pacing applies per Space, per member. Sending in
        // Space A must not affect the queue in Space B.
        let mut mgr = PacingManager::new();
        let space_b = "xgen://hash/sha256:space_b";
        let d_a = mgr.attempt_send(SPACE_A, ALICE, false, 0);
        let d_b = mgr.attempt_send(space_b, ALICE, false, 0);
        assert_eq!(d_a, SendDecision::SendNow);
        assert_eq!(d_b, SendDecision::SendNow);
    }

    #[test]
    fn queues_are_isolated_per_sender() {
        // Within a Space, each sender has independent pacing.
        let mut mgr = PacingManager::new();
        let d_a = mgr.attempt_send(SPACE_A, ALICE, false, 0);
        let d_b = mgr.attempt_send(SPACE_A, BOT, false, 0);
        assert_eq!(d_a, SendDecision::SendNow);
        assert_eq!(d_b, SendDecision::SendNow);
    }

    /// T14 — `PacingManager.queues` is keyed by `(SpaceXgid, IdentityXgid)`
    /// (design doc §4.6.d / Q7.1). Insert via the `&str` API; retrieve the
    /// same entry via the `&str` API (the typed composite key is constructed
    /// at the boundary); the returned `PacingState` carries typed identifier
    /// slots.
    #[test]
    fn pacing_per_space_sender_map_insert_retrieve_with_typed_key() {
        let mut mgr = PacingManager::new();
        let _ = mgr.attempt_send(SPACE_A, ALICE, false, 0);
        let snap = mgr
            .snapshot(SPACE_A, ALICE, 100)
            .expect("entry present after insert via typed composite key");
        assert_eq!(snap.space_id.as_str(), SPACE_A);
        assert_eq!(snap.sender_identity_id.as_str(), ALICE);
    }
}
