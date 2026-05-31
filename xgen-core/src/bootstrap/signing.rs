// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! Bootstrap message signing / verification (bootstrap-client D-071 arc, C2).
//!
//! `BootstrapMessage` frames carry an Ed25519 signature over their canonical
//! object form (spec §3.14.3 — "the Bootstrap Node verifies the signature").
//! This module is the exact sibling of `federation/handshake.rs`'s
//! `sign_msg` / `verify_msg` for `BootstrapMessage`: pure crypto logic, no I/O.
//! The socket orchestration (connect, send, receive) lives in
//! `xgen-node::bootstrap_client` per BC-D3 (xgen-core stays transport-pure).
//!
//! **Verification trust model (Pin 2, J-192 lock):** an ack is verified
//! against the **operator-supplied / stored `bootstrap_id`** — NOT the ack's
//! own self-declared `node_id` (which would be circular: a man-in-the-middle
//! at the URL could self-declare any id and sign with the matching key). The
//! verifier asserts the message's `node_id` field equals the expected id AND
//! that the signature verifies against the key derived from the expected id.

use ed25519_dalek::SigningKey;
use thiserror::Error;
use xgen_common::xgid::{NodeXgid, Xgid};

use crate::crypto::signing;
use crate::wire::canonical::canonical_object_json;
use crate::wire::types::BootstrapMessage;

// ── Canonical field orders (signature excluded) ───────────────────────────────
// Sibling to `federation/handshake.rs`'s HELLO_FIELDS etc. The `type` field is
// the serde tag ("bootstrap.register" …) and is part of the signed form.

const REGISTER_FIELDS: &[&str] = &[
    "protocol_version", "type", "node_id", "endpoint", "region", "capabilities", "timestamp",
];
const REGISTER_ACK_FIELDS: &[&str] = &[
    "protocol_version", "type", "node_id", "directory_url", "timestamp",
];
const KEEPALIVE_FIELDS: &[&str] = &["protocol_version", "type", "node_id", "timestamp"];
const KEEPALIVE_ACK_FIELDS: &[&str] = &["protocol_version", "type", "node_id", "timestamp"];
const DEREGISTER_FIELDS: &[&str] = &["protocol_version", "type", "node_id", "timestamp"];

#[derive(Debug, Error)]
pub enum BootstrapSignError {
    #[error("signature field is absent")]
    SignatureMissing,
    #[error("signature verification failed (bootstrap_signature_invalid, 7003)")]
    SignatureInvalid,
    #[error("ack node_id {got} does not match the expected bootstrap node {expected}")]
    NodeIdMismatch { expected: String, got: String },
    #[error("cannot derive a verifying key from node_id: {0}")]
    KeyDecode(String),
}

/// Sign a bootstrap message and return it with the `signature` field populated.
/// Signs with the supplied keypair (the registrant's, for `Register`/`Keepalive`/
/// `Deregister`; the bootstrap node's, for the `*_ack` frames a server emits).
pub fn sign_bootstrap(msg: BootstrapMessage, key: &SigningKey) -> BootstrapMessage {
    let canonical = canonical_json_of(&msg);
    let sig = signing::sign(key, canonical.as_bytes());
    with_signature(msg, sig)
}

/// Verify a signed bootstrap message against the **expected** node id (Pin 2).
///
/// `expected_node_id` is the operator-supplied / stored `bootstrap_id` for an
/// inbound ack. Asserts (a) the message's `node_id` field equals
/// `expected_node_id`, and (b) the signature verifies against the key derived
/// from `expected_node_id`. Returns `Err` otherwise.
pub fn verify_bootstrap_signed(
    msg: &BootstrapMessage,
    expected_node_id: &str,
) -> Result<(), BootstrapSignError> {
    let (node_id, sig_opt) = extract_id_and_sig(msg);
    let sig_str = sig_opt.ok_or(BootstrapSignError::SignatureMissing)?;
    if node_id != expected_node_id {
        return Err(BootstrapSignError::NodeIdMismatch {
            expected: expected_node_id.to_string(),
            got: node_id.to_string(),
        });
    }
    // Derive the verifying key from the EXPECTED id, not the message's own
    // (Pin 2: verifying against the self-declared id would be circular).
    let vk = NodeXgid::from_xgid(Xgid::new(expected_node_id.to_string()))
        .pubkey()
        .map_err(|e| BootstrapSignError::KeyDecode(e.to_string()))?;
    let canonical = canonical_json_of(msg);
    signing::verify(&vk, canonical.as_bytes(), sig_str)
        .map_err(|_| BootstrapSignError::SignatureInvalid)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn canonical_json_of(msg: &BootstrapMessage) -> String {
    let v = serde_json::to_value(msg).expect("BootstrapMessage is always serialisable");
    canonical_object_json(&v, field_order_for(msg))
}

fn field_order_for(msg: &BootstrapMessage) -> &'static [&'static str] {
    match msg {
        BootstrapMessage::Register { .. } => REGISTER_FIELDS,
        BootstrapMessage::RegisterAck { .. } => REGISTER_ACK_FIELDS,
        BootstrapMessage::Keepalive { .. } => KEEPALIVE_FIELDS,
        BootstrapMessage::KeepaliveAck { .. } => KEEPALIVE_ACK_FIELDS,
        BootstrapMessage::Deregister { .. } => DEREGISTER_FIELDS,
    }
}

fn extract_id_and_sig(msg: &BootstrapMessage) -> (&str, Option<&str>) {
    match msg {
        BootstrapMessage::Register { node_id, signature, .. }
        | BootstrapMessage::RegisterAck { node_id, signature, .. }
        | BootstrapMessage::Keepalive { node_id, signature, .. }
        | BootstrapMessage::KeepaliveAck { node_id, signature, .. }
        | BootstrapMessage::Deregister { node_id, signature, .. } => {
            (node_id.as_str(), signature.as_deref())
        }
    }
}

fn with_signature(msg: BootstrapMessage, sig: String) -> BootstrapMessage {
    match msg {
        BootstrapMessage::Register {
            protocol_version, node_id, endpoint, region, capabilities, timestamp, ..
        } => BootstrapMessage::Register {
            protocol_version, node_id, endpoint, region, capabilities, timestamp,
            signature: Some(sig),
        },
        BootstrapMessage::RegisterAck {
            protocol_version, node_id, directory_url, timestamp, ..
        } => BootstrapMessage::RegisterAck {
            protocol_version, node_id, directory_url, timestamp,
            signature: Some(sig),
        },
        BootstrapMessage::Keepalive { protocol_version, node_id, timestamp, .. } => {
            BootstrapMessage::Keepalive { protocol_version, node_id, timestamp, signature: Some(sig) }
        }
        BootstrapMessage::KeepaliveAck { protocol_version, node_id, timestamp, .. } => {
            BootstrapMessage::KeepaliveAck { protocol_version, node_id, timestamp, signature: Some(sig) }
        }
        BootstrapMessage::Deregister { protocol_version, node_id, timestamp, .. } => {
            BootstrapMessage::Deregister { protocol_version, node_id, timestamp, signature: Some(sig) }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn node_id_uri(key: &SigningKey) -> String {
        format!(
            "xgen://pubkey/ed25519:{}",
            crate::crypto::encoding::encode(key.verifying_key().as_bytes())
        )
    }

    fn sample_ack(bootstrap_node_id: &str) -> BootstrapMessage {
        BootstrapMessage::RegisterAck {
            protocol_version: "0.1".to_string(),
            node_id: bootstrap_node_id.to_string(),
            directory_url: "https://bootstrap.example.com/xgen-directory".to_string(),
            timestamp: "2026-05-31T12:00:00.000Z".to_string(),
            signature: None,
        }
    }

    #[test]
    fn sign_then_verify_round_trip() {
        let bk = SigningKey::from_bytes(&[0x42; 32]);
        let bid = node_id_uri(&bk);
        let signed = sign_bootstrap(sample_ack(&bid), &bk);
        // Verified against the expected bootstrap id (== the signer here).
        assert!(verify_bootstrap_signed(&signed, &bid).is_ok());
    }

    #[test]
    fn tampered_message_fails_verification() {
        let bk = SigningKey::from_bytes(&[0x42; 32]);
        let bid = node_id_uri(&bk);
        let signed = sign_bootstrap(sample_ack(&bid), &bk);
        // Flip a signed field after signing.
        let tampered = match signed {
            BootstrapMessage::RegisterAck { protocol_version, node_id, timestamp, signature, .. } => {
                BootstrapMessage::RegisterAck {
                    protocol_version,
                    node_id,
                    directory_url: "https://evil.example.com/dir".to_string(),
                    timestamp,
                    signature,
                }
            }
            _ => unreachable!(),
        };
        assert!(matches!(
            verify_bootstrap_signed(&tampered, &bid),
            Err(BootstrapSignError::SignatureInvalid)
        ));
    }

    #[test]
    fn wrong_expected_id_is_rejected_not_circular() {
        // Pin 2: an ack signed by key A but verified against expected id B is
        // rejected on the node_id mismatch BEFORE any signature check — a MITM
        // self-declaring its own id cannot pass.
        let attacker = SigningKey::from_bytes(&[0x11; 32]);
        let attacker_id = node_id_uri(&attacker);
        let expected_bk = SigningKey::from_bytes(&[0x99; 32]);
        let expected_id = node_id_uri(&expected_bk);

        // Attacker signs an ack self-declaring its own id (internally consistent).
        let signed = sign_bootstrap(sample_ack(&attacker_id), &attacker);
        let err = verify_bootstrap_signed(&signed, &expected_id).unwrap_err();
        assert!(matches!(err, BootstrapSignError::NodeIdMismatch { .. }));
    }

    #[test]
    fn forged_signature_under_expected_id_is_rejected() {
        // Attacker claims the expected id but cannot produce a valid signature
        // for it (doesn't hold the key). node_id matches → falls through to the
        // signature check, which fails.
        let attacker = SigningKey::from_bytes(&[0x11; 32]);
        let expected_bk = SigningKey::from_bytes(&[0x99; 32]);
        let expected_id = node_id_uri(&expected_bk);

        // Self-declare the expected id but sign with the attacker's key.
        let signed = sign_bootstrap(sample_ack(&expected_id), &attacker);
        assert!(matches!(
            verify_bootstrap_signed(&signed, &expected_id),
            Err(BootstrapSignError::SignatureInvalid)
        ));
    }

    #[test]
    fn missing_signature_is_rejected() {
        let bk = SigningKey::from_bytes(&[0x42; 32]);
        let bid = node_id_uri(&bk);
        let unsigned = sample_ack(&bid); // signature: None
        assert!(matches!(
            verify_bootstrap_signed(&unsigned, &bid),
            Err(BootstrapSignError::SignatureMissing)
        ));
    }

    #[test]
    fn register_frame_round_trips_too() {
        // The same machinery covers the outbound Register frame (signed by the
        // registrant, verified against the registrant's own id).
        let rk = SigningKey::from_bytes(&[0x07; 32]);
        let rid = node_id_uri(&rk);
        let reg = BootstrapMessage::Register {
            protocol_version: "0.1".to_string(),
            node_id: rid.clone(),
            endpoint: "wss://self.example.com/xgen".to_string(),
            region: "EU".to_string(),
            capabilities: vec!["xgen.federation".to_string()],
            timestamp: "2026-05-31T12:00:00.000Z".to_string(),
            signature: None,
        };
        let signed = sign_bootstrap(reg, &rk);
        assert!(verify_bootstrap_signed(&signed, &rid).is_ok());
    }
}
