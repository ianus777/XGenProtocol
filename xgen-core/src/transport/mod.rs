// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Transport module — WebSocket connections, authentication, keepalive (spec 3.3).
// server.rs is not part of xgen-core; it lives in xgen-node (Node-specific).

pub mod auth;
pub mod client;
pub mod connection;
