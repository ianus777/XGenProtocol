// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

pub mod smoke;
pub mod cold_start_bootstrap_integration;
pub mod federation_integration;
pub mod federation_delta_integration;
pub mod federation_push_integration;
pub mod federation_relationship_integration;
pub mod heldpending_identity_integration;
pub mod identity_integration;
pub mod reconnect_integration;

// Phase 9 deployment-level federation scenarios (task file
// `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 3). The harness module
// provides in-process two-Node spawning; per-scenario test modules use it.
pub mod phase9_harness;
pub mod phase9_two_node_smoke;
