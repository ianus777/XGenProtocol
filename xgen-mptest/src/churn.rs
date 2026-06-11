// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Connection-churn driver (MP-R2-D1(c) / C4). **Test-only, never ships.**
//!
//! The net-new orchestrator capability MP-A-18 (connect/disconnect storm) +
//! MP-A-19 (slow-loris / held connections) need — the audit §2 falsification
//! found connection-churn is neither a dial-spawn axis nor a batch property, and
//! no such primitive existed (the orchestrator opens exactly one `.aicontrol`
//! per actor + one WS per injector). This opens **raw WS connections** to a
//! node's transport and holds/drops them, reusing `xgen-core`'s
//! [`connect_url`](xgen_core::transport::client::connect_url) — the same entry
//! [`crate::wireactor::WireActor`] uses (driving/observing, not patching).
//!
//! Connections are opened **post-WS-handshake, un-authenticated** — the storm
//! tests connection setup/teardown + held-idle load, including the node's own
//! handling of unauthenticated connections (it may time out + close them; that
//! is part of what is tested). The property the smokes assert is **the node
//! stays live** (a legitimate `.aicontrol` command still lands after the churn).

use std::time::Duration;

use tokio_tungstenite::MaybeTlsStream;
use xgen_core::transport::client::connect_url;
use xgen_core::transport::connection::Connection;

use crate::Result;

/// A raw connection held open by the churn driver (post-WS-handshake).
pub type RawConn = Connection<MaybeTlsStream<tokio::net::TcpStream>>;

/// One churn action — the storm **plan** is a pure value, unit-testable without
/// sockets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChurnAction {
    /// Open this many connections.
    OpenN(usize),
    /// Drop the cycle's open connections.
    DropAll,
}

/// A connect/disconnect storm: `cycles` rounds of (open `conns_per_cycle` → drop).
#[derive(Debug, Clone, Copy)]
pub struct StormPlan {
    pub cycles: usize,
    pub conns_per_cycle: usize,
}

impl StormPlan {
    /// The ordered action sequence (pure — no sockets). Each cycle is an
    /// `OpenN` followed by a `DropAll`.
    pub fn actions(&self) -> Vec<ChurnAction> {
        let mut out = Vec::with_capacity(self.cycles * 2);
        for _ in 0..self.cycles {
            out.push(ChurnAction::OpenN(self.conns_per_cycle));
            out.push(ChurnAction::DropAll);
        }
        out
    }
}

/// Open up to `n` raw connections to `url`, **best-effort** (a node under churn
/// may refuse some — that is fine; failed opens are simply not counted). Returns
/// the connections that opened. Holding the returned values keeps the sockets
/// open; dropping them closes the connections.
pub async fn open_best_effort(url: &str, n: usize) -> Vec<RawConn> {
    let mut held = Vec::with_capacity(n);
    for _ in 0..n {
        if let Ok(c) = connect_url(url).await {
            held.push(c);
        }
    }
    held
}

/// Execute a connect/disconnect storm against a live node (**heavy**). Each
/// cycle opens `conns_per_cycle` (best-effort), holds them briefly so they
/// coexist, then drops them. Returns the total count of successful opens. The
/// node-stays-live property is asserted by the caller (the MP-A-18 smoke) after.
pub async fn run_storm(url: &str, plan: StormPlan) -> Result<usize> {
    let mut opened = 0usize;
    for _ in 0..plan.cycles {
        let held = open_best_effort(url, plan.conns_per_cycle).await;
        opened += held.len();
        // Brief coexistence window, then the cycle's batch drops (DropAll).
        tokio::time::sleep(Duration::from_millis(20)).await;
        drop(held);
    }
    Ok(opened)
}

/// Open `n` connections and **hold** them for `hold` without driving traffic
/// (the slow-loris / held-idle-connection shape — MP-A-19). Returns the held
/// connections so the caller can keep holding them while it probes node
/// liveness, then drop them. **Heavy.**
pub async fn slow_loris(url: &str, n: usize, hold: Duration) -> Result<Vec<RawConn>> {
    let conns = open_best_effort(url, n).await;
    tokio::time::sleep(hold).await;
    Ok(conns)
}

/// How an event flood paces its submits (MP-R3 §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloodMode {
    /// Paced (R2): drain each ack between sends + an inter-send delay — bounded
    /// by ack latency. A `Duration::ZERO` pace is back-to-back-with-drain.
    Paced(Duration),
    /// Firehose (R3 enrichment): **no** ack-drain, no inter-send delay — submit
    /// as fast as bytes write, bounded only by write throughput. The MP-A-07
    /// intensity *curve*'s top end.
    Firehose,
}

impl FloodMode {
    /// Whether this mode drains each submit's ack (the paced bound). A firehose
    /// does not — acks pile up unread.
    pub fn drains_ack(self) -> bool {
        matches!(self, FloodMode::Paced(_))
    }

    /// The inter-send delay (zero for a firehose).
    pub fn pace(self) -> Duration {
        match self {
            FloodMode::Paced(d) => d,
            FloodMode::Firehose => Duration::ZERO,
        }
    }
}

/// An **event flood** (MP-A-07): a member-context client submits `count` messages
/// under `mode`. Sibling to [`run_storm`] — both are liveness-under-load drivers
/// (flood the node, then the caller probes honest-traffic liveness), NOT
/// convergence sweeps (the audit §2 + C5 finding: intensity is not a `SweepAxis`).
/// Returns the count submitted. **Heavy.**
///
/// The flood's `prev_events` all reference the create root (the messages are
/// concurrent siblings — valid for a volume/liveness test; not a convergence
/// assertion).
pub async fn event_flood_mode(url: &str, count: usize, mode: FloodMode) -> Result<usize> {
    use crate::injector::build_member_message;
    use crate::wireactor::WireActor;

    let mut wa = WireActor::connect(url).await?;
    wa.register("flooder").await?;
    let space = wa.create_space("FLOOD").await?;
    let room = wa.create_room(&space, "general").await?;
    let key = wa.key().clone();
    let mut sent = 0usize;
    for i in 0..count {
        let ev = build_member_message(&key, &space, &room, vec![&space], &format!("flood-{i}"));
        let ok = if mode.drains_ack() {
            wa.submit(&ev).await.is_ok()
        } else {
            // Firehose: send without draining the ack — bounded by write throughput.
            wa.submit_no_drain(&ev).await.is_ok()
        };
        if ok {
            sent += 1;
        }
        let pace = mode.pace();
        if !pace.is_zero() {
            tokio::time::sleep(pace).await;
        }
    }
    Ok(sent)
}

/// The paced event flood (R2 compatibility): [`event_flood_mode`] with
/// `FloodMode::Paced(pace)`.
pub async fn event_flood(url: &str, count: usize, pace: Duration) -> Result<usize> {
    event_flood_mode(url, count, FloodMode::Paced(pace)).await
}

/// The **firehose** event flood (MP-R3 §6.3 enrichment): [`event_flood_mode`]
/// with [`FloodMode::Firehose`] — no ack-drain, the top of the intensity curve.
pub async fn event_flood_firehose(url: &str, count: usize) -> Result<usize> {
    event_flood_mode(url, count, FloodMode::Firehose).await
}

/// The MP-A-07 intensity **curve** (MP-R3 §6.3): a descending sequence of paced
/// inter-send delays — rung `i` paces at `start_ms - step_ms*i`, clamped at 0
/// (the firehose floor). Pure (the rung schedule), so it is unit-tested without
/// sockets; the caller runs [`event_flood_mode`] per rung and records the
/// break-point-per-rate (the curve, not the R2 single liveness point).
pub fn flood_rate_curve(start_ms: u64, step_ms: u64, rungs: usize) -> Vec<Duration> {
    (0..rungs)
        .map(|i| Duration::from_millis(start_ms.saturating_sub(step_ms.saturating_mul(i as u64))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storm_plan_cycles_open_then_drop() {
        // Pure — no sockets. A 3×10 storm is 3 cycles of (OpenN(10), DropAll).
        let plan = StormPlan {
            cycles: 3,
            conns_per_cycle: 10,
        };
        let actions = plan.actions();
        assert_eq!(actions.len(), 6);
        assert_eq!(actions[0], ChurnAction::OpenN(10));
        assert_eq!(actions[1], ChurnAction::DropAll);
        assert_eq!(actions[4], ChurnAction::OpenN(10));
        assert_eq!(actions[5], ChurnAction::DropAll);
    }

    #[test]
    fn zero_cycle_storm_is_empty() {
        let plan = StormPlan {
            cycles: 0,
            conns_per_cycle: 10,
        };
        assert!(plan.actions().is_empty());
    }

    #[test]
    fn firehose_submits_without_draining_acks() {
        // MP-R3 §6.3: the firehose mode omits the ack-drain (and the inter-send
        // delay) — bounded by write throughput, the top of the intensity curve;
        // the paced mode drains + paces.
        assert!(!FloodMode::Firehose.drains_ack());
        assert_eq!(FloodMode::Firehose.pace(), Duration::ZERO);
        assert!(FloodMode::Paced(Duration::from_millis(5)).drains_ack());
        assert_eq!(
            FloodMode::Paced(Duration::from_millis(5)).pace(),
            Duration::from_millis(5)
        );
    }

    #[test]
    fn flood_rate_curve_sweeps_pace() {
        // The intensity curve descends from `start_ms` by `step_ms` per rung,
        // clamped at 0 (the firehose floor).
        let curve = flood_rate_curve(100, 25, 5);
        assert_eq!(
            curve,
            vec![
                Duration::from_millis(100),
                Duration::from_millis(75),
                Duration::from_millis(50),
                Duration::from_millis(25),
                Duration::from_millis(0), // clamped — firehose floor
            ]
        );
        // Further rungs stay clamped at zero (saturating).
        let clamped = flood_rate_curve(20, 25, 3);
        assert_eq!(
            clamped,
            vec![
                Duration::from_millis(20),
                Duration::from_millis(0),
                Duration::from_millis(0),
            ]
        );
    }
}
