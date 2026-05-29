// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Re-export xgen-core public API (all protocol modules except transport, which is extended below).
pub use xgen_core::crypto;
pub use xgen_core::wire;
pub use xgen_core::dag;
pub use xgen_core::node;
pub use xgen_core::federation;
pub use xgen_core::identity;
pub use xgen_core::space;
pub use xgen_core::message;

// Node-specific modules.
// transport: extends xgen-core transport with the WebSocket server (Node-specific).
pub mod admin_ops; // M6 — Node admin write path, single source (D-067).
pub mod app;
pub mod audit; // M6 — admin audit trail (SQLite, §2.6.4).
pub mod desktop;
pub mod fanout;
pub mod federation_session;
pub mod lifecycle;
pub mod pipe;
pub mod plugins;
pub mod reconnect;
pub mod transport;

#[cfg(test)]
pub mod tests;
