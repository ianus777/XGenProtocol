// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Clock injection seam (M8.6 — Federation stress).
//!
//! A minimal time-source abstraction (Fork A, design §3) so the federation
//! reconnect / F-10 paths can be driven by a mock clock under test without real
//! waiting. Two non-interconvertible time domains are exposed:
//!
//! - **W — wall-clock** (`now_utc`): `chrono::DateTime<Utc>`. Ladder math +
//!   `mark_lost` / `mark_active` stamping; persisted as RFC3339, so it must be
//!   absolute calendar time.
//! - **M — monotonic** (`now_instant`): `std::time::Instant`. The F-10
//!   HeldPending window; must be jump-immune (an NTP step or DST change must not
//!   falsely expire or hold a pending event).
//!
//! The **T domain** (tokio sleeps / timeouts) is deliberately NOT in this trait
//! — tests drive it via `tokio::time::pause` / `advance`. Keeping T out lets the
//! trait (and `xgen-common`) stay tokio-free.
//!
//! `RealClock` is the always-on production implementation. `MockClock` is a
//! test-support type (feature `mock-clock`, also active under `cfg(test)` within
//! this crate) built on a single advancing cursor so one knob moves both derived
//! reads in lockstep — see the `advance_all` harness helper in the xgen-node
//! tests that pairs `MockClock::advance` with `tokio::time::advance` so W / M / T
//! move together from one call.
//!
//! Seam reuse (runbook §8): kept federation-agnostic so M9's harness can reuse
//! it. The mock backing is an `AtomicU64` of nanoseconds — a faithful, lock-free
//! realization of the design's single-cursor `Mutex<Duration>` sketch (the
//! methods are `&self`, so an atomic counter is the natural fit with no
//! poisoning/contention; the single-cursor base+offset semantic is preserved).
//! Backing-type only, below the lock line (Joe-lock checkpoint #1, 2026-06-06).

use std::time::Instant;

use chrono::{DateTime, Utc};

/// Injected time source. Two sync, non-blocking reads across two
/// non-interconvertible domains (W = wall, M = monotonic).
pub trait Clock: Send + Sync {
    /// Wall-clock now (domain W).
    fn now_utc(&self) -> DateTime<Utc>;
    /// Monotonic now (domain M).
    fn now_instant(&self) -> Instant;
}

/// Production clock — reads the real OS clocks. Behaviour-identical to the
/// pre-seam inline `Utc::now()` / `Instant::now()` reads.
#[derive(Debug, Default, Clone, Copy)]
pub struct RealClock;

impl Clock for RealClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
    fn now_instant(&self) -> Instant {
        Instant::now()
    }
}

#[cfg(any(test, feature = "mock-clock"))]
mod mock {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    use chrono::{DateTime, Duration as ChronoDuration, Utc};

    use super::Clock;

    /// Test-only mock clock (M8.6). A single advancing cursor (nanoseconds)
    /// drives both derived reads, so one `advance(d)` moves W and M forward by
    /// the same delta. Anchored at a real base captured at construction
    /// (`std::time::Instant` has no arbitrary constructor → the model is
    /// base + cursor, not absolute set).
    pub struct MockClock {
        base_utc: DateTime<Utc>,
        base_instant: Instant,
        cursor_nanos: AtomicU64,
    }

    impl MockClock {
        /// New mock anchored at the current real time, cursor at 0.
        pub fn new() -> Self {
            Self {
                base_utc: Utc::now(),
                base_instant: Instant::now(),
                cursor_nanos: AtomicU64::new(0),
            }
        }

        /// Advance the single cursor by `d` (moves both `now_utc` and
        /// `now_instant` forward by the same delta). Relaxed-safe — one
        /// independent counter, no cross-variable dependency — but `SeqCst`
        /// is kept for clarity (harmless; checkpoint #1).
        pub fn advance(&self, d: Duration) {
            self.cursor_nanos
                .fetch_add(d.as_nanos() as u64, Ordering::SeqCst);
        }

        fn cursor(&self) -> u64 {
            self.cursor_nanos.load(Ordering::SeqCst)
        }
    }

    impl Default for MockClock {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Clock for MockClock {
        fn now_utc(&self) -> DateTime<Utc> {
            self.base_utc + ChronoDuration::nanoseconds(self.cursor() as i64)
        }
        fn now_instant(&self) -> Instant {
            self.base_instant + Duration::from_nanos(self.cursor())
        }
    }
}

#[cfg(any(test, feature = "mock-clock"))]
pub use mock::MockClock;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn clock_mock_advances_utc_and_instant_in_lockstep() {
        let clock = MockClock::new();
        let utc0 = clock.now_utc();
        let inst0 = clock.now_instant();

        clock.advance(Duration::from_secs(30));

        let utc1 = clock.now_utc();
        let inst1 = clock.now_instant();

        // Both derived reads moved by the same 30s delta from one advance().
        assert_eq!((utc1 - utc0).num_seconds(), 30);
        assert_eq!(inst1.duration_since(inst0), Duration::from_secs(30));

        // A second advance accumulates on the same cursor — W and M stay locked.
        clock.advance(Duration::from_secs(90));
        assert_eq!((clock.now_utc() - utc0).num_seconds(), 120);
        assert_eq!(
            clock.now_instant().duration_since(inst0),
            Duration::from_secs(120)
        );
    }

    #[test]
    fn clock_real_reads_are_monotonic_and_wall() {
        let clock = RealClock;

        // Wall read is a real timestamp near now (sanity: within a wide window).
        let utc = clock.now_utc();
        let delta = (Utc::now() - utc).num_seconds().abs();
        assert!(delta < 5, "RealClock wall read should be ~now, delta={delta}s");

        // Monotonic read does not go backwards.
        let i0 = clock.now_instant();
        let i1 = clock.now_instant();
        assert!(i1 >= i0, "RealClock monotonic read must be non-decreasing");
    }
}
