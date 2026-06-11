// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! C3 micro-benchmark (M9, `#[ignore]` — spawns many real node binaries).
//!
//! Validates the spawn micro-benchmark + box-ceiling report against the real
//! `xgen-node.exe`. Tiers default light (`[3, 6]`) so the routine ignored run is
//! cheap; the real box-ceiling run for the 32 GB / 20-core box sets the full
//! tiers:
//!
//! ```text
//! cargo build -p xgen-node
//! XGEN_MPTEST_BENCH_TIERS=10,50,100 \
//!   cargo test -p xgen-mptest --test c3_microbench -- --ignored --nocapture
//! ```

use xgen_mptest::bench::{bench_client_mean_rss, run_microbench, tiers_from_env};
use xgen_mptest::binloc;
use xgen_mptest::sweep::CeilingFloors;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "heavy: spawns many real xgen-node processes; run with --ignored"]
async fn c3_box_ceiling_microbench() {
    let bins = binloc::locate().expect("locate built binaries");
    let tiers = tiers_from_env(&[3, 6]);

    let report = run_microbench(&bins, &tiers, 8500)
        .await
        .expect("run micro-benchmark");

    // Every tier produced samples.
    for t in &report.tiers {
        assert!(
            t.aggregate.count > 0,
            "tier {} produced no resource samples",
            t.tier
        );
        assert!(
            t.aggregate.mean_rss_mb() > 0.0,
            "tier {} mean RSS should be > 0",
            t.tier
        );
    }
    let ceiling = report.estimated_ceiling();
    assert!(ceiling > 0, "estimated ceiling should be > 0");

    // Write the box-ceiling report to a temp artifact dir + echo it.
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = report.write_to(tmp.path()).expect("write report");
    assert!(path.exists());
    eprintln!("\n{}", report.to_markdown());
    eprintln!("box-ceiling report written to {}", path.display());
}

/// MP-R3-D5c — the capstone re-bench: sample the CLIENT mean RSS too, so the
/// ceiling denominator is the combined (node + client) footprint, not node-only.
/// Feeds `CeilingFloors::from_bench_combined`. Run alongside the node bench at the
/// box-gated RUN.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "heavy: spawns a node + clients to sample client RSS; run with --ignored"]
async fn mp_r3_client_rebench_combined_floors() {
    let bins = binloc::locate().expect("locate built binaries");

    // The node ceiling (as before).
    let report = run_microbench(&bins, &tiers_from_env(&[3, 6]), 8560)
        .await
        .expect("node micro-benchmark");

    // The client mean RSS (R3-D5c) — a small representative client mix.
    let client_mean = bench_client_mean_rss(&bins, 6, 8580)
        .await
        .expect("client mean RSS bench");
    assert!(client_mean > 0.0, "client mean RSS should be > 0");

    // The combined floors track the heavier of (node, client) means.
    let node_only = CeilingFloors::from_bench(&report);
    let combined = CeilingFloors::from_bench_combined(&report, Some(client_mean));
    eprintln!(
        "MP-R3 re-bench: node ceiling ~{}; client mean RSS {:.1} MB; node-only wall {:.0} MB; combined wall {:.0} MB",
        report.estimated_ceiling(),
        client_mean / (1024.0 * 1024.0),
        node_only.rss_wall_bytes as f64 / (1024.0 * 1024.0),
        combined.rss_wall_bytes as f64 / (1024.0 * 1024.0),
    );
    // The combined wall is never softer than node-only (it tracks the larger mean).
    assert!(combined.rss_wall_bytes >= node_only.rss_wall_bytes);
}
