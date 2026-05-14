// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Bootstrap Node HTTP client — directory fetch (spec 3.14.4).
//
// Nodes use this to fetch a remote Bootstrap Node's directory over HTTPS.
//
// Phase 2: this module is a placeholder. The actual reqwest HTTP client calls
// are implemented in xgen-node (the thin shell crate). The protocol logic for
// processing a fetched directory document lives in bootstrap/directory.rs.

/// Maximum age of a directory document before it must be re-fetched (WD-24).
/// Value: 3600 seconds (1 hour).
pub const DIRECTORY_MAX_AGE_SECS: u64 = 3600;
