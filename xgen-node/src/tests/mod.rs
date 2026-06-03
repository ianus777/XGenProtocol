// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

pub mod smoke;
pub mod events_pipe_integration;
pub mod cold_start_bootstrap_integration;
pub mod federation_integration;
pub mod federation_delta_integration;
pub mod federation_push_integration;
pub mod federation_approval_gate;
pub mod federation_policy_enforcement;
pub mod federation_relationship_integration;
pub mod heldpending_identity_integration;
pub mod identity_integration;
pub mod reconnect_integration;
pub mod bootstrap_client_integration;
// Storage-engine substitution (SE-SUB-D1…D6) — feature-gated on the sqlite engine.
#[cfg(feature = "store-sqlite")]
pub mod storage_engine_substitution;
pub mod bootstrap_keepalive_integration;

// Phase 9 deployment-level federation scenarios (task file
// `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 3 / §3.0 revised
// five-commit shape). The harness module provides in-process N-Node spawning
// + respawn-with-saved-state for Scenario 3; per-scenario test modules use it.
pub mod phase9_harness;
pub mod phase9_two_node_smoke;
pub mod phase9_three_node_anti_transitivity;
pub mod phase9_drop_and_recover;
pub mod phase9_unknown_signer_first_contact;
pub mod phase9_federation_relationship_rejection;
pub mod phase9_compound_c2_anti_transitivity_at_load;
pub mod phase9_compound_c7_pagination_boundary;
pub mod phase9_compound_c10_identity_lock_contention;
pub mod phase9_m8_convergence_smoke;
