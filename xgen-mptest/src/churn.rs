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

/// An **event flood** (MP-A-07): a member-context client submits `count` messages
/// at `pace` inter-send delay (lower `pace` = higher rate; the intensity knob).
/// Sibling to [`run_storm`] — both are liveness-under-load drivers (flood the
/// node, then the caller probes honest-traffic liveness), NOT convergence sweeps
/// (the audit §2 + C5 finding: intensity is not a `SweepAxis`). Returns the count
/// submitted. **Heavy.**
///
/// Boundary: each submit briefly drains its ack ([`WireActor::submit`]), so this
/// is a *paced* submit stream bounded by ack latency, not a fire-hose; a true
/// fire-hose (no-drain submit) is an R3 enrichment. The flood's `prev_events`
/// all reference the create root (the messages are concurrent siblings — valid
/// for a volume/liveness test; not a convergence assertion).
pub async fn event_flood(url: &str, count: usize, pace: Duration) -> Result<usize> {
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
        if wa.submit(&ev).await.is_ok() {
            sent += 1;
        }
        if !pace.is_zero() {
            tokio::time::sleep(pace).await;
        }
    }
    Ok(sent)
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
}
