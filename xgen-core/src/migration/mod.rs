// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Space Migration Protocol (spec 3.12.1–3.12.8).
//
// Migration transfers the full Event history, SpaceState, membership, and federation
// relationships from a source Node to a destination Node. The space_id is preserved.
//
// This module provides:
//   - state_machine  — handler functions for both source and destination sides
//   - transfer       — event batching and tail tracking
//   - verification   — post-transfer integrity checks (event count + DAG tips)

pub mod state_machine;
pub mod transfer;
pub mod verification;
