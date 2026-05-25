// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! Phase 9 Commit 3b-4 NodeRuntime-level tests.
//!
//! Three test files (Sc4 + C5 + C9) at this directory per runbook §4.4 +
//! checkpoint #2 Joe-lock (C7 + C10 live xgen-node-side because their
//! production targets — `compute_federation_delta_for_space` and
//! `handle_identity_replicate_msg` respectively — are in `xgen-node`).
//!
//! See `tasks/FEDERATION_PROPAGATION_PHASE_9_COMMIT_3B_4_IMPL.md` for the
//! contracts these tests pin.

mod phase9_compound_c5_validation_under_load;
mod phase9_compound_c9_drain_time_hazard;
mod phase9_validation_asymmetry;
