// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Per-process RSS + thread-count sampling (M9-D7 / C3).
//!
//! The micro-benchmark needs each spawned binary's resident-set size and thread
//! count to derive the box ceiling (audit §6.1: ~15–35 MB RSS each; threads
//! pinned 1–2). Rather than add a `sysinfo`-class dependency for a test-only
//! sampler, [`sample_process`] shells out to PowerShell `Get-Process` (Win11,
//! always present) and reads `WorkingSet64` + `Threads.Count`. The pure
//! aggregation ([`Aggregate`]) is dependency-free and unit-tested.

use crate::Result;

/// One process's resource sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceSample {
    pub pid: u32,
    pub rss_bytes: u64,
    pub thread_count: u32,
}

impl ResourceSample {
    pub fn rss_mb(&self) -> f64 {
        self.rss_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Aggregate stats over a set of samples — the box-ceiling inputs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Aggregate {
    pub count: usize,
    pub total_rss_bytes: u64,
    pub max_rss_bytes: u64,
    pub total_threads: u64,
    pub max_threads: u32,
}

impl Aggregate {
    /// Aggregate a slice of samples.
    pub fn of(samples: &[ResourceSample]) -> Aggregate {
        let mut a = Aggregate::default();
        for s in samples {
            a.count += 1;
            a.total_rss_bytes += s.rss_bytes;
            a.max_rss_bytes = a.max_rss_bytes.max(s.rss_bytes);
            a.total_threads += s.thread_count as u64;
            a.max_threads = a.max_threads.max(s.thread_count);
        }
        a
    }

    pub fn mean_rss_bytes(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_rss_bytes as f64 / self.count as f64
        }
    }

    pub fn mean_rss_mb(&self) -> f64 {
        self.mean_rss_bytes() / (1024.0 * 1024.0)
    }

    pub fn mean_threads(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_threads as f64 / self.count as f64
        }
    }
}

/// Parse the sampler's `"<rss> <threads>"` line into (rss_bytes, thread_count).
/// Pure, so it is unit-tested without spawning PowerShell.
fn parse_sample_line(line: &str) -> Option<(u64, u32)> {
    let mut it = line.split_whitespace();
    let rss = it.next()?.parse::<u64>().ok()?;
    let threads = it.next()?.parse::<u32>().ok()?;
    Some((rss, threads))
}

/// Sample one live process by pid. Windows-only (PowerShell `Get-Process`).
#[cfg(windows)]
pub fn sample_process(pid: u32) -> Result<ResourceSample> {
    use anyhow::{anyhow, Context};
    let script = format!(
        "$p = Get-Process -Id {pid} -ErrorAction Stop; \
         Write-Output ('{{0}} {{1}}' -f $p.WorkingSet64, $p.Threads.Count)"
    );
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .with_context(|| format!("sampling pid {pid} via Get-Process"))?;
    if !out.status.success() {
        return Err(anyhow!(
            "Get-Process for pid {pid} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let (rss_bytes, thread_count) = parse_sample_line(stdout.trim())
        .ok_or_else(|| anyhow!("unparseable Get-Process output for pid {pid}: {stdout:?}"))?;
    Ok(ResourceSample {
        pid,
        rss_bytes,
        thread_count,
    })
}

/// Non-Windows stub.
#[cfg(not(windows))]
pub fn sample_process(pid: u32) -> Result<ResourceSample> {
    Err(anyhow::anyhow!(
        "resource sampling is Windows-only (pid {pid})"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_line_reads_rss_and_threads() {
        assert_eq!(parse_sample_line("31457280 12"), Some((31457280, 12)));
        assert_eq!(parse_sample_line("  100  3  "), Some((100, 3)));
        assert_eq!(parse_sample_line("garbage"), None);
        assert_eq!(parse_sample_line(""), None);
    }

    #[test]
    fn aggregate_computes_totals_and_means() {
        let s = vec![
            ResourceSample { pid: 1, rss_bytes: 20 * 1024 * 1024, thread_count: 2 },
            ResourceSample { pid: 2, rss_bytes: 30 * 1024 * 1024, thread_count: 4 },
        ];
        let a = Aggregate::of(&s);
        assert_eq!(a.count, 2);
        assert_eq!(a.total_rss_bytes, 50 * 1024 * 1024);
        assert_eq!(a.max_rss_bytes, 30 * 1024 * 1024);
        assert_eq!(a.max_threads, 4);
        assert_eq!(a.total_threads, 6);
        assert!((a.mean_rss_mb() - 25.0).abs() < 1e-9);
        assert!((a.mean_threads() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn empty_aggregate_is_zero() {
        let a = Aggregate::of(&[]);
        assert_eq!(a.count, 0);
        assert_eq!(a.mean_rss_mb(), 0.0);
        assert_eq!(a.mean_threads(), 0.0);
    }
}
