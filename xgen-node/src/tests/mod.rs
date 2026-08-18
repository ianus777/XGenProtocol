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
// M8.6 (Federation stress) — shared reconnect test support (responsive mock
// receiver + lost-peer registry + silent black-hole + clock lockstep helper).
pub mod reconnect_test_support;
// M8.6 C4 — scheduler-direct churn: attempt-task gauge (spawn-leak detector) +
// backoff-ladder advancement / reset-on-ACTIVE / peer_records consistency.
pub mod m8_6_c4_reconnect_churn;
// M8.6 C1 — F-10 HeldPending drain consistency across an F-1a re-stream
// (single-Node re-delivery model; runbook below-the-lock amendment).
pub mod m8_6_c1_held_pending_reconnect;
// M8.6 C8 — bidirectional push under provoked back-pressure (cap-2 seam);
// regression lock against a future blocking-send change.
pub mod m8_6_c8_bidirectional_push;
pub mod bootstrap_client_integration;
// Storage-engine substitution (SE-SUB-D1…D6) — feature-gated on the sqlite engine.
#[cfg(feature = "store-sqlite")]
pub mod storage_engine_substitution;
pub mod bootstrap_keepalive_integration;
// MP-F9 C2 — late-federation identity catch-up: a peer that federates after a
// Space has history receives the signers' IdentityRecords on establish, so the
// backfilled events validate instead of holding on the F-10 unknown-signer gate.
pub mod late_federation_identity_catchup;

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

// M8 — Wave 1 / C2 — S2 concurrent state-event convergence (Layers 1/4/5c:
// byte-identical resolved SpaceState across Nodes + client projection, every
// arrival permutation; G-ALIGN; key-rotation substituted by thread.status, M8-D4).
pub mod m8_s2_convergence;

// M8 — Wave 2 / C3 — S3 federation: migration multiparty convergence (Arc F);
// jurisdiction + 3-node delivery + F-5 anti-transitivity referenced in findings.
pub mod m8_s3_federation;

// M8 — Wave 2 / C4 — S4 durability/replay gate (G-DURABILITY): restart → replay
// → byte-identical SpaceState, no orphans. (S5 rebind = BLOCKED, findings-only.)
pub mod m8_s4_durability;

// M8 — Wave 3 / C5 — S6 E2E content-blindness (Arc H): N-member encrypted fan-out
// stored opaque (M3); KeyPackage pool consume+replenish; epoch-advance on mls.commit.
pub mod m8_s6_e2e;

// M8 — Wave 3 / C6 — S7 privilege enforcement (Arc D): tier-gated join refused on
// all Nodes (3030); per-Room Deny override blocks + converges. Multiparty only (M8-A7).
pub mod m8_s7_privilege;

// Arc F (PG-11) — Space Migration two-node end-to-end (C2 DoD).
pub mod phase_arcf_migration_e2e;

// Arc H (PG-05) — content-blindness proof (AH-D5).
pub mod arc_h_content_blindness;

// M12.1 — blob attachment witnesses (W1–W5) against the real node via a real WS
// client: round-trip / content-blindness / hash-integrity / multi-chunk / no-federation.
pub mod m12_blob_roundtrip;
pub mod m12_3_federation_fetch;
pub mod m12_4_redact;

// M-SPACE-ADMISSION Leg A-bis leg 1 (before-assertion) — a registered third
// party is admitted to a DM it is not party to, as Role::Member with no invite;
// plus the permanent open-Space companion. Records today's behaviour before the
// admission gate (Leg D) makes the DM half unobservable (D-148 clause 4).
pub mod space_admission_third_party_join;
