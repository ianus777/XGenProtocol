// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! During-chaos liveness probe (MP-R3-D4b).
//!
//! A small task that, at intervals **during** the chaos window, sends a benign
//! `state` aicontrol round-trip to each node and records whether it responded —
//! the capstone's "node stays live under sustained stacked chaos" witness. It
//! opens its **own** aicontrol connection per probe (it does not borrow the
//! director's node connections), so it composes with the director + the parallel
//! chaos task. Bounded by a fixed sample budget so the concurrent phase's
//! `join!` always returns.

use std::time::Duration;

use crate::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use crate::wire::Command;

/// One node's liveness record across the probe window.
#[derive(Debug, Clone)]
pub struct NodeLiveness {
    pub label: String,
    /// Total probe samples taken.
    pub samples: usize,
    /// Samples that got an OK reply.
    pub responses: usize,
    /// Sample index (0-based) of the FIRST unresponsive probe, if any.
    pub first_unresponsive_at: Option<usize>,
}

impl NodeLiveness {
    fn new(label: &str) -> NodeLiveness {
        NodeLiveness {
            label: label.to_string(),
            samples: 0,
            responses: 0,
            first_unresponsive_at: None,
        }
    }

    /// Record one probe outcome (`ok` = the node replied OK to `state`).
    pub fn record(&mut self, ok: bool) {
        let idx = self.samples;
        self.samples += 1;
        if ok {
            self.responses += 1;
        } else if self.first_unresponsive_at.is_none() {
            self.first_unresponsive_at = Some(idx);
        }
    }

    /// `true` if the node responded to **every** probe (and was probed at least
    /// once) — i.e. it stayed live throughout the chaos window.
    pub fn responsive(&self) -> bool {
        self.samples > 0 && self.first_unresponsive_at.is_none()
    }
}

/// The liveness report across all nodes over the probe window.
#[derive(Debug, Clone, Default)]
pub struct LivenessReport {
    pub nodes: Vec<NodeLiveness>,
}

impl LivenessReport {
    /// `true` if every node stayed live (responded to every probe).
    pub fn all_responsive(&self) -> bool {
        !self.nodes.is_empty() && self.nodes.iter().all(NodeLiveness::responsive)
    }

    /// The labels of nodes that went unresponsive at least once (a finding).
    pub fn unresponsive(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .filter(|n| !n.responsive())
            .map(|n| n.label.as_str())
            .collect()
    }
}

/// Probe one node's `.aicontrol` pipe once with a `state` round-trip; `true` if
/// it replied OK (opens a fresh connection per probe — no held connection).
async fn probe_once(pipe: &str) -> bool {
    match AicontrolClient::connect(pipe, DEFAULT_CONNECT_TIMEOUT).await {
        Ok(mut ctl) => ctl
            .send(&Command::new("state"))
            .await
            .map(|r| r.is_ok())
            .unwrap_or(false),
        Err(_) => false,
    }
}

/// Run the during-chaos liveness probe (MP-R3-D4b) — `samples` rounds, sleeping
/// `interval` between rounds, probing each `(label, aicontrol_pipe)` per round.
/// **Heavy** (opens real connections); bounded by `samples` so the concurrent
/// phase's `join!` returns.
pub async fn run_liveness_probe(
    pipes: &[(String, String)],
    samples: usize,
    interval: Duration,
) -> LivenessReport {
    let mut nodes: Vec<NodeLiveness> = pipes.iter().map(|(l, _)| NodeLiveness::new(l)).collect();
    for s in 0..samples {
        for (i, (_, pipe)) in pipes.iter().enumerate() {
            let ok = probe_once(pipe).await;
            nodes[i].record(ok);
        }
        if s + 1 < samples {
            tokio::time::sleep(interval).await;
        }
    }
    LivenessReport { nodes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liveness_report_flags_unresponsive_node() {
        // MP-R3-D4b: a node that fails a probe is flagged unresponsive (with the
        // first-failure sample index); a node that responds to every probe is live.
        let mut live = NodeLiveness::new("a");
        for _ in 0..5 {
            live.record(true);
        }
        assert!(live.responsive());
        assert_eq!(live.first_unresponsive_at, None);

        let mut dead = NodeLiveness::new("b");
        dead.record(true);
        dead.record(true);
        dead.record(false); // went unresponsive on sample index 2
        dead.record(true);
        assert!(!dead.responsive());
        assert_eq!(dead.first_unresponsive_at, Some(2));
        assert_eq!(dead.responses, 3);

        let report = LivenessReport {
            nodes: vec![live, dead],
        };
        assert!(!report.all_responsive(), "one node went unresponsive");
        assert_eq!(report.unresponsive(), vec!["b"]);

        // An all-live report passes.
        let mut x = NodeLiveness::new("x");
        x.record(true);
        let all_ok = LivenessReport { nodes: vec![x] };
        assert!(all_ok.all_responsive());

        // An empty report is NOT "all responsive" (nothing was probed).
        assert!(!LivenessReport::default().all_responsive());
    }
}
