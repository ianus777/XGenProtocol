// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Spawn micro-benchmark → box-ceiling report (M9-D7 / C3).
//!
//! Spawns node processes in tiers (the runbook's 10/50/100), confirms each is up
//! (aicontrol `state`), samples RSS + thread count ([`crate::resource`]), and
//! derives how many node processes fit on the box from the measured mean RSS —
//! grounding the audit §6.1 ~800–1,200-process estimate for the 32 GB / 20-core
//! box **before** the R2/R3 numbers are fixed. The report is written to the
//! artifact dir.
//!
//! Heavy (spawns many real binaries) → driven from an `#[ignore]` test or an
//! explicit run. Tiers default light; the real box-ceiling run sets
//! `XGEN_MPTEST_BENCH_TIERS=10,50,100`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::aicontrol::{AicontrolClient, DEFAULT_CONNECT_TIMEOUT};
use crate::binloc::Binaries;
use crate::process::{instance_label, ManagedProcess};
use crate::resource::{sample_process, Aggregate, ResourceSample};
use crate::Result;
use anyhow::Context;

/// The machine the ceiling is estimated for (audit §6.1: 32 GB / 20-core).
#[derive(Debug, Clone, Copy)]
pub struct BoxSpec {
    pub total_ram_bytes: u64,
    /// RAM held back for the OS + the harness itself.
    pub reserved_bytes: u64,
    pub cores: u32,
}

impl Default for BoxSpec {
    fn default() -> Self {
        BoxSpec {
            total_ram_bytes: 32 * 1024 * 1024 * 1024,
            reserved_bytes: 4 * 1024 * 1024 * 1024,
            cores: 20,
        }
    }
}

impl BoxSpec {
    /// RAM available for node processes.
    pub fn budget_bytes(&self) -> u64 {
        self.total_ram_bytes.saturating_sub(self.reserved_bytes)
    }

    /// Estimated process ceiling from a measured mean RSS (memory is the wall —
    /// audit §6.1). `0` if `mean_rss_bytes` is non-positive.
    pub fn estimate_ceiling(&self, mean_rss_bytes: f64) -> u64 {
        if mean_rss_bytes <= 0.0 {
            return 0;
        }
        (self.budget_bytes() as f64 / mean_rss_bytes) as u64
    }
}

/// One tier's measurement.
#[derive(Debug, Clone)]
pub struct TierResult {
    pub tier: usize,
    pub aggregate: Aggregate,
}

/// The box-ceiling report.
#[derive(Debug, Clone)]
pub struct BoxCeilingReport {
    pub box_spec: BoxSpec,
    pub tiers: Vec<TierResult>,
}

impl BoxCeilingReport {
    /// The mean RSS from the largest measured tier (the steadiest estimate).
    pub fn reference_mean_rss_bytes(&self) -> f64 {
        self.tiers
            .iter()
            .max_by_key(|t| t.tier)
            .map(|t| t.aggregate.mean_rss_bytes())
            .unwrap_or(0.0)
    }

    /// Estimated node-process ceiling on the box.
    pub fn estimated_ceiling(&self) -> u64 {
        self.box_spec
            .estimate_ceiling(self.reference_mean_rss_bytes())
    }

    /// Render a human-readable report.
    pub fn to_markdown(&self) -> String {
        let mut s = String::new();
        s.push_str("# M9 box-ceiling micro-benchmark\n\n");
        s.push_str(&format!(
            "Box: {} GB RAM ({} GB reserved), {} cores. Budget for nodes: {:.1} GB.\n\n",
            self.box_spec.total_ram_bytes / (1024 * 1024 * 1024),
            self.box_spec.reserved_bytes / (1024 * 1024 * 1024),
            self.box_spec.cores,
            self.box_spec.budget_bytes() as f64 / (1024.0 * 1024.0 * 1024.0),
        ));
        s.push_str("| tier (nodes) | sampled | mean RSS (MB) | max RSS (MB) | mean threads | max threads |\n");
        s.push_str("|---:|---:|---:|---:|---:|---:|\n");
        for t in &self.tiers {
            s.push_str(&format!(
                "| {} | {} | {:.1} | {:.1} | {:.1} | {} |\n",
                t.tier,
                t.aggregate.count,
                t.aggregate.mean_rss_mb(),
                t.aggregate.max_rss_bytes as f64 / (1024.0 * 1024.0),
                t.aggregate.mean_threads(),
                t.aggregate.max_threads,
            ));
        }
        s.push_str(&format!(
            "\nReference mean RSS: {:.1} MB (largest tier).\n",
            self.reference_mean_rss_bytes() / (1024.0 * 1024.0)
        ));
        s.push_str(&format!(
            "**Estimated node-process ceiling on this box: ~{}** (memory-bound).\n",
            self.estimated_ceiling()
        ));
        s
    }

    /// Write the report to `<root>/box-ceiling-<nonce>.md` and return its path.
    pub fn write_to(&self, root: impl AsRef<Path>) -> Result<PathBuf> {
        let root = root.as_ref();
        std::fs::create_dir_all(root)
            .with_context(|| format!("creating bench artifact dir {}", root.display()))?;
        let path = root.join(format!("box-ceiling-{}.md", crate::process::label_nonce()));
        std::fs::write(&path, self.to_markdown())
            .with_context(|| format!("writing box-ceiling report {}", path.display()))?;
        Ok(path)
    }
}

/// Parse `XGEN_MPTEST_BENCH_TIERS` (comma-separated), defaulting to a light set
/// so the routine `#[ignore]` test stays cheap. The real run sets `10,50,100`.
pub fn tiers_from_env(default: &[usize]) -> Vec<usize> {
    match std::env::var("XGEN_MPTEST_BENCH_TIERS") {
        Ok(s) => s
            .split(',')
            .filter_map(|t| t.trim().parse::<usize>().ok())
            .filter(|&n| n > 0)
            .collect(),
        Err(_) => default.to_vec(),
    }
}

/// Run the micro-benchmark over `tiers`, spawning nodes on `base_port + i`.
/// Each tier: spawn → confirm-up (aicontrol `state`) → sample every pid →
/// aggregate → tear down before the next tier.
pub async fn run_microbench(
    bins: &Binaries,
    tiers: &[usize],
    base_port: u16,
) -> Result<BoxCeilingReport> {
    let mut results = Vec::new();
    for &tier in tiers {
        let mut nodes: Vec<ManagedProcess> = Vec::with_capacity(tier);
        for i in 0..tier {
            let label = instance_label("bench", &format!("n{i}"));
            let port = base_port + i as u16;
            let node = ManagedProcess::init_and_spawn_node(bins, &label, port, true)
                .with_context(|| format!("bench tier {tier}: spawn node {i}"))?;
            nodes.push(node);
        }

        // Confirm each is up (aicontrol `state` round-trip), then sample.
        let mut samples: Vec<ResourceSample> = Vec::with_capacity(tier);
        for node in &nodes {
            let mut ctl =
                AicontrolClient::connect(&node.aicontrol_pipe, DEFAULT_CONNECT_TIMEOUT).await?;
            let _ = ctl.send_verb("state").await?; // proves the node is fully up
            drop(ctl);
        }
        // Let working sets settle a beat after the readiness probes.
        tokio::time::sleep(Duration::from_millis(300)).await;
        for node in &nodes {
            match sample_process(node.pid()) {
                Ok(s) => samples.push(s),
                Err(e) => eprintln!("bench: sample pid {} failed: {e:#}", node.pid()),
            }
        }
        results.push(TierResult {
            tier,
            aggregate: Aggregate::of(&samples),
        });
        // nodes drop here → killed before the next tier (no overlap).
    }
    Ok(BoxCeilingReport {
        box_spec: BoxSpec::default(),
        tiers: results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceiling_estimate_is_budget_over_mean_rss() {
        let b = BoxSpec {
            total_ram_bytes: 32 * 1024 * 1024 * 1024,
            reserved_bytes: 4 * 1024 * 1024 * 1024,
            cores: 20,
        };
        // 28 GB budget / 28 MB mean = 1024 processes.
        let mean = 28.0 * 1024.0 * 1024.0;
        assert_eq!(b.estimate_ceiling(mean), 1024);
        assert_eq!(b.estimate_ceiling(0.0), 0);
    }

    #[test]
    fn report_uses_largest_tier_for_reference() {
        let report = BoxCeilingReport {
            box_spec: BoxSpec::default(),
            tiers: vec![
                TierResult {
                    tier: 10,
                    aggregate: Aggregate {
                        count: 10,
                        total_rss_bytes: 10 * 20 * 1024 * 1024,
                        max_rss_bytes: 25 * 1024 * 1024,
                        total_threads: 20,
                        max_threads: 3,
                    },
                },
                TierResult {
                    tier: 50,
                    aggregate: Aggregate {
                        count: 50,
                        total_rss_bytes: 50 * 30 * 1024 * 1024,
                        max_rss_bytes: 40 * 1024 * 1024,
                        total_threads: 100,
                        max_threads: 3,
                    },
                },
            ],
        };
        // Largest tier (50) mean = 30 MB.
        assert!((report.reference_mean_rss_bytes() / (1024.0 * 1024.0) - 30.0).abs() < 1e-6);
        let md = report.to_markdown();
        assert!(md.contains("box-ceiling micro-benchmark"));
        assert!(md.contains("Estimated node-process ceiling"));
    }

    #[test]
    fn tiers_from_env_default_when_unset() {
        // Note: relies on the env var being unset in the test process.
        std::env::remove_var("XGEN_MPTEST_BENCH_TIERS");
        assert_eq!(tiers_from_env(&[3, 6]), vec![3, 6]);
    }
}
