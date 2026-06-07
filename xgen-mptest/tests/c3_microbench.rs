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

use xgen_mptest::bench::{run_microbench, tiers_from_env};
use xgen_mptest::binloc;

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
