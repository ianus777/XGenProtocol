// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

pub mod smoke;
// Phase 7.5 Commit 4 — paused 2026-05-20 pending the Phase 7 B3 amendment
// Joe-lock. Two production gaps surfaced in the Commit 4 integration tests
// when state.federation_add arrives via federation channel on a cold receiver:
//   1. Predecessor-chain deadlock: step 9 finds federation_add's predecessors
//      in the HeldPending buffer (federation-relationship trigger), not in
//      the store → federation_add HeldPending on missing predecessors.
//   2. Step 11 sender-membership fails: federation_add's sender is the Node
//      URI per Phase 4 §3.4.1 Q3 overload, not a Space member.
// Re-enable once the B3 amendment ships (planned Commit 3.5 between Commit 3
// and Commit 4).
// pub mod cold_start_bootstrap_integration;
pub mod federation_integration;
pub mod federation_delta_integration;
pub mod federation_push_integration;
pub mod federation_relationship_integration;
pub mod heldpending_identity_integration;
pub mod identity_integration;
pub mod reconnect_integration;
