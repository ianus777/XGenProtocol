// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! The round dial (M9-D7 / C3).
//!
//! Scale is **parameterized, never hardwired** (audit §6.1): a run is
//! `N nodes × M clients × R residents/process × ramp profile × clock-mode`. The
//! dial is the knob set the scenario runner ([`crate::runner`]) and the bigger
//! Multiparty-tests rounds (R1→R2→R3) turn. In R1 the scenario **manifest** is
//! authoritative for the explicit topology (`run_scenario` spawns one node per
//! `manifest.nodes`); the dial carries the clock mode + the R2/R3 multiplexing
//! knobs the sweep ([`crate::sweep`], C2) turns.
//!
//! ## Clock-mode (un-staled, MP-R1-D1 / G-2)
//! M9-D5 calls for MockClock (deterministic, R1) and real-clock (R2/R3). When
//! this module was first written neither a `mock-clock` binary build nor a
//! clock-advance control surface existed, so [`ClockMode::Mock`] was declared
//! **inoperable** and rejected by [`RoundDial::validate`]. M9.2 shipped both: a
//! `--features harness-control` node build installs a `MockClock` at startup and
//! the fenced `clock advance` / `clock set` `.aicontrol` verbs drive it (M9.2-D3
//! / F3). So [`ClockMode::Mock`] is now a **valid** mode. Operability is a
//! property of the spawned node *build*, not the dial: a `Mock` run requires a
//! `--features harness-control` node, which [`crate::runner::run_scenario`]
//! asserts at run start (a `clock advance 0s` probe, failing loud like the F2/F3
//! smokes) rather than the dial pre-rejecting it.

use crate::Result;
use anyhow::anyhow;

/// How participants are introduced over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RampProfile {
    /// All actors started at once (Round-0 + small R1).
    #[default]
    AllAtOnce,
    /// One actor every `step_ms` (gentler load onset for the bigger rounds).
    Staggered { step_ms: u64 },
}

/// The clock the spawned binaries run on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockMode {
    /// Real wall/monotonic clock — the default production build (R2/R3).
    #[default]
    Real,
    /// Deterministic `MockClock` — installed at startup on a
    /// `--features harness-control` node build and driven over `.aicontrol`
    /// via `clock advance` / `clock set` (M9.2-D3 / F3). The R1 mode.
    Mock,
}

impl ClockMode {
    /// Whether a `Mock` run additionally requires a `--features harness-control`
    /// node build (the `MockClock` install + the clock verbs are fenced). `Real`
    /// runs on any build.
    pub fn requires_harness_control(self) -> bool {
        matches!(self, ClockMode::Mock)
    }
}

/// One run's scale parameters (M9-D7). Never a hardwired constant — the runner
/// and the rounds set these explicitly.
#[derive(Debug, Clone)]
pub struct RoundDial {
    /// Real node processes (topology width — HW-bound).
    pub nodes: usize,
    /// Real client processes.
    pub clients: usize,
    /// Logical participants multiplexed per client/AI-resident process (load
    /// depth — cheap; an AI resident drives many logical participants).
    pub residents_per_process: usize,
    /// Participant introduction profile.
    pub ramp: RampProfile,
    /// Clock mode.
    pub clock: ClockMode,
    /// tokio worker threads pinned per spawned binary (M9-D7 — scheduler-thrash
    /// guard at scale). `None` ⇒ the harness default (`TOKIO_WORKER_THREADS`).
    pub worker_threads: Option<u32>,
    /// MP-R3-D4c — elastic settle ceiling. The post-drive poll-until-stable
    /// quiesce window's MAXIMUM (seconds). `None` ⇒ the default 15s (R1/R2
    /// behaviour); a capstone heal + catch-up sets it higher. The stable-for-2
    /// termination is unchanged, so a quiesced run still returns early — only the
    /// ceiling extends.
    pub settle_max_secs: Option<u64>,
    /// MP-R3-D4b — run the during-chaos liveness probe (a periodic `state`
    /// round-trip to each node) concurrently with the drive. `false` (default) ⇒
    /// no probe (R1/R2 behaviour); chaos-overlay rows set it `true`.
    pub liveness_probe: bool,
}

impl Default for RoundDial {
    fn default() -> Self {
        // Round-0 default: two nodes, two clients, one logical participant each,
        // all at once, real clock.
        RoundDial {
            nodes: 2,
            clients: 2,
            residents_per_process: 1,
            ramp: RampProfile::AllAtOnce,
            clock: ClockMode::Real,
            worker_threads: Some(2),
            settle_max_secs: None,
            liveness_probe: false,
        }
    }
}

/// The default settle ceiling (seconds) when the dial leaves `settle_max_secs`
/// unset — the R1/R2 value (MP-R3-D4c).
pub const DEFAULT_SETTLE_MAX_SECS: u64 = 15;

impl RoundDial {
    /// Total logical participants = clients × residents/process.
    pub fn logical_participants(&self) -> usize {
        self.clients.saturating_mul(self.residents_per_process)
    }

    /// MP-R3-D4c — the elastic settle ceiling: the dial's `settle_max_secs` if
    /// set, else [`DEFAULT_SETTLE_MAX_SECS`] (15). Pure, so the resolution is
    /// unit-tested without spawning.
    pub fn settle_max(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.settle_max_secs.unwrap_or(DEFAULT_SETTLE_MAX_SECS))
    }

    /// Total real OS processes the run will spawn (nodes + clients).
    pub fn total_processes(&self) -> usize {
        self.nodes.saturating_add(self.clients)
    }

    /// Reject a structurally-invalid dial. (`ClockMode::Mock` is now valid — its
    /// operability is a property of the spawned node build, asserted at run start
    /// by [`crate::runner::run_scenario`], not pre-rejected here; MP-R1-D1/G-2.)
    pub fn validate(&self) -> Result<()> {
        if self.nodes == 0 {
            return Err(anyhow!("round dial: nodes must be ≥1"));
        }
        if self.residents_per_process == 0 {
            return Err(anyhow!("round dial: residents_per_process must be ≥1"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_dial_is_round_0_shape() {
        let d = RoundDial::default();
        assert_eq!(d.nodes, 2);
        assert_eq!(d.clients, 2);
        assert_eq!(d.total_processes(), 4);
        assert_eq!(d.logical_participants(), 2);
        assert!(d.validate().is_ok());
    }

    #[test]
    fn logical_participants_multiplies() {
        let d = RoundDial {
            clients: 4,
            residents_per_process: 250,
            ..Default::default()
        };
        assert_eq!(d.logical_participants(), 1000);
    }

    #[test]
    fn mock_clock_is_accepted_and_flags_harness_control() {
        // Un-staled (MP-R1-D1 / G-2): Mock is a valid mode; the build
        // requirement is surfaced via `requires_harness_control`, not a
        // validate() rejection.
        let d = RoundDial {
            clock: ClockMode::Mock,
            ..Default::default()
        };
        assert!(d.clock.requires_harness_control());
        assert!(d.validate().is_ok());
    }

    #[test]
    fn real_clock_needs_no_harness_control() {
        assert!(!ClockMode::Real.requires_harness_control());
        assert!(RoundDial::default().validate().is_ok());
    }

    #[test]
    fn settle_max_secs_resolves_default_and_override() {
        // MP-R3-D4c: unset → the 15s default (R1/R2); set → the override ceiling.
        let d = RoundDial::default();
        assert!(d.settle_max_secs.is_none());
        assert_eq!(d.settle_max(), std::time::Duration::from_secs(15));
        let capstone = RoundDial {
            settle_max_secs: Some(90),
            ..Default::default()
        };
        assert_eq!(capstone.settle_max(), std::time::Duration::from_secs(90));
    }

    #[test]
    fn zero_nodes_rejected() {
        let d = RoundDial {
            nodes: 0,
            ..Default::default()
        };
        assert!(d.validate().is_err());
    }
}
