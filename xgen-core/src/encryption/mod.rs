// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// End-to-End Encryption layer (spec 3.10.1–3.10.9).
//
// XGen uses MLS (Messaging Layer Security, RFC 9420) for E2E encryption.
// The Node acts as an MLS Delivery Service: routes MLS messages, stores
// encrypted blobs, tracks epochs — but cannot decrypt content.
//
// Phase 2 implementation note (D-052):
// The full RFC 9420 openmls client integration is deferred to Phase 3.
// Phase 2 implements the complete delivery service protocol and a Phase 2 MLS
// interface (epoch-based ChaCha20Poly1305) that correctly captures forward
// secrecy properties. The interface is identical to what the openmls
// integration will use; only the underlying key schedule changes.
//
// This module provides:
//   - key_package      — Node-side KeyPackage store (one-time-use, expiry-aware)
//   - group            — Node-side MLS group state tracking (epoch counter)
//   - delivery_service — Node Delivery Service: routing, epoch management
//   - client_mls       — Client-side MLS interface (Phase 2: epoch-key scheme)

pub mod client_mls;
pub mod delivery_service;
pub mod group;
pub mod key_package;
