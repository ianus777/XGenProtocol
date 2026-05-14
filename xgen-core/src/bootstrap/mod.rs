// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Bootstrap Node Protocol (spec 3.14.1–3.14.8) and Node Reputation Format (spec 3.15.1–3.15.4).
//
// Bootstrap Nodes are ordinary XGen Nodes with one additional declared capability:
// `xgen.bootstrap`. They maintain a signed directory of known Nodes and serve it
// over HTTPS. Any Node operator may run a Bootstrap Node.
//
// This module provides:
//   - capability   — BootstrapInfo struct, helpers for declaring/checking capability
//   - directory    — in-memory directory, signing, ordering by reputation score
//   - reputation   — ReputationComponents, score computation, merge, defederation signal
//   - http         — HTTP server placeholder (axum binding handled by xgen-node)
//   - client       — HTTP client placeholder (reqwest binding handled by callers)

pub mod capability;
pub mod directory;
pub mod http;
pub mod client;
pub mod reputation;
