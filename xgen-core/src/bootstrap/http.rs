// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Bootstrap Node HTTP server — directory endpoint (spec 3.14.2).
//
// The Bootstrap Node serves its directory as a signed JSON document over HTTPS.
// This is the only place in XGen where HTTP is used (D-051).
//
// Phase 2: this module is a placeholder. The actual axum HTTP server binding is
// implemented in xgen-node (the thin shell crate) using the pure logic from
// bootstrap/directory.rs. The port separation (WebSocket port vs HTTP port) is
// an xgen-node concern, not a protocol logic concern.

/// Default HTTP port for the Bootstrap Node directory endpoint (D-051).
pub const BOOTSTRAP_HTTP_PORT: u16 = 8443;
