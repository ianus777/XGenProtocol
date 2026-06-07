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
//! dial is the knob set the scenario runner (C5) and the bigger Multiparty-tests
//! rounds (R1→R2→R3) turn; Round-0 reads its small values from the scenario
//! manifest.
//!
//! ## Clock-mode grounding (a finding, recorded per D-065)
//! M9-D5 calls for MockClock (deterministic, R1) and real-clock (R2/R3). But the
//! `mock-clock` is a **non-default build feature** of the binaries and is driven
//! **in-process** — a production `cargo build -p xgen-node` does **not** enable
//! it, and there is **no `.aicontrol` verb to advance a running node's clock**
//! from outside. So driving MockClock across the real-process boundary needs two
//! things that do not exist yet: (a) a `--features mock-clock` build, and (b) a
//! clock-advance control surface. [`ClockMode::Mock`] is therefore declared but
//! **not yet operable** for spawned binaries; Round-0 runs [`ClockMode::Real`]
//! (the smokes prove the machinery — determinism comes from the scenario shape,
//! not the clock). Operable MockClock is a Multiparty-tests prerequisite (an
//! ergonomic aicontrol addition → Joe-lock), surfaced here, not silently
//! assumed.

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
    /// Real wall/monotonic clock — the only operable mode for spawned
    /// production binaries today (see module note).
    #[default]
    Real,
    /// Deterministic MockClock — **not yet operable** across the process
    /// boundary (needs a `mock-clock` build + a clock-advance control surface).
    Mock,
}

impl ClockMode {
    /// Whether this mode can actually drive a spawned production binary today.
    pub fn is_operable(self) -> bool {
        matches!(self, ClockMode::Real)
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
        }
    }
}

impl RoundDial {
    /// Total logical participants = clients × residents/process.
    pub fn logical_participants(&self) -> usize {
        self.clients.saturating_mul(self.residents_per_process)
    }

    /// Total real OS processes the run will spawn (nodes + clients).
    pub fn total_processes(&self) -> usize {
        self.nodes.saturating_add(self.clients)
    }

    /// Reject a dial the harness cannot honor today.
    pub fn validate(&self) -> Result<()> {
        if self.nodes == 0 {
            return Err(anyhow!("round dial: nodes must be ≥1"));
        }
        if self.residents_per_process == 0 {
            return Err(anyhow!("round dial: residents_per_process must be ≥1"));
        }
        if !self.clock.is_operable() {
            return Err(anyhow!(
                "round dial: ClockMode::Mock is not operable across the process boundary yet \
                 (needs a mock-clock build + a clock-advance control surface) — use ClockMode::Real"
            ));
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
    fn mock_clock_is_rejected_as_inoperable() {
        let d = RoundDial {
            clock: ClockMode::Mock,
            ..Default::default()
        };
        assert!(!d.clock.is_operable());
        let r = d.validate();
        assert!(r.is_err());
        assert!(format!("{:#}", r.unwrap_err()).contains("not operable"));
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
