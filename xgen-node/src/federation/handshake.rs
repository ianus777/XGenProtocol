// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Federation handshake state machine (spec 3.4.1–3.4.7).
//
// States (initiating Node A / receiving Node B):
//
//   A: IDLE → [send hello] → CAPS_RECEIVED → [send accept] → ACTIVE
//   B: IDLE → [recv hello] → HELLO_RECEIVED → [send capabilities] → CAPS_SENT
//           → [recv accept] → ACTIVE
//
// Both sides sign every outgoing message with their node keypair.
// Both sides verify the signature on every incoming message.

use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use ed25519_dalek::{SigningKey, VerifyingKey};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    crypto::{encoding, hashing, signing},
    transport::connection::{Connection, Inbound, TransportError},
    wire::{
        canonical::canonical_object_json,
        types::{FederationCapabilities, FederationMessage, NegotiatedCapabilities},
    },
};

// ── Timeouts (spec 3.4.3) ────────────────────────────────────────────────────

/// Maximum seconds to wait for the peer's response message.
const WAIT_TIMEOUT_SECS: u64 = 15;

// ── Canonical field orders (signature excluded) ───────────────────────────────

const HELLO_FIELDS: &[&str] = &[
    "protocol_version", "type", "node_id", "capabilities", "shared_spaces", "timestamp",
];
const CAPS_FIELDS: &[&str] = &[
    "protocol_version", "type", "node_id", "capabilities", "negotiated", "timestamp",
];
const ACCEPT_FIELDS: &[&str] = &[
    "protocol_version", "type", "node_id", "session_id", "timestamp",
];
const REJECT_FIELDS: &[&str] = &[
    "protocol_version", "type", "node_id", "error_code", "error_string", "timestamp",
];
const GOODBYE_FIELDS: &[&str] = &[
    "protocol_version", "type", "node_id", "reason", "session_id", "timestamp",
];

// ── Public types ─────────────────────────────────────────────────────────────

/// Established federation session parameters, returned when handshake reaches ACTIVE.
#[derive(Debug, Clone)]
pub struct FederationSession {
    pub peer_node_id: String,
    pub session_id: String,
    pub negotiated_serialisation: String,
    pub negotiated_version: String,
    pub shared_spaces: Vec<String>,
}

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("no common serialisation capabilities (error 2001)")]
    NoCommonCapabilities,
    #[error("incompatible protocol versions (error 2002)")]
    VersionIncompatible,
    #[error("unexpected message in {0} state: {1}")]
    UnexpectedMessage(&'static str, String),
    #[error("signature verification failed (error 2006)")]
    SignatureInvalid,
    #[error("handshake timed out after {WAIT_TIMEOUT_SECS}s")]
    Timeout,
    #[error("peer rejected handshake (code {code}): {message}")]
    Rejected { code: u32, message: String },
}

impl HandshakeError {
    pub fn to_federation_code(&self) -> (u32, &'static str) {
        match self {
            Self::NoCommonCapabilities => (2001, "no_common_capabilities"),
            Self::VersionIncompatible => (2002, "version_incompatible"),
            Self::UnexpectedMessage(_, _) => (2005, "unexpected_message"),
            Self::SignatureInvalid => (2006, "signature_invalid"),
            _ => (2005, "unexpected_message"),
        }
    }
}

// ── Initiating Node (Node A) ──────────────────────────────────────────────────

/// Run the federation handshake as the **initiating** Node.
///
/// Sequence: send hello → receive capabilities → send accept → ACTIVE.
pub async fn run_initiating<S>(
    conn: &mut Connection<S>,
    signing_key: &SigningKey,
    caps: FederationCapabilities,
    shared_spaces: Vec<String>,
) -> Result<FederationSession, HandshakeError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let our_node_id = node_id_from_key(&signing_key.verifying_key());
    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    // Send federation.hello (signed).
    let hello = sign_msg(
        FederationMessage::Hello {
            protocol_version: "0.1".to_string(),
            node_id: our_node_id.clone(),
            capabilities: caps,
            shared_spaces: shared_spaces.clone(),
            timestamp: ts,
            signature: None,
        },
        signing_key,
    );
    conn.send_federation(&hello).await?;

    // Wait for federation.capabilities.
    let inbound = recv_with_timeout(conn).await?;
    let caps_msg = match inbound {
        Inbound::Federation(fm @ FederationMessage::Capabilities { .. }) => fm,
        Inbound::Federation(FederationMessage::Reject { error_code, error_string, .. }) => {
            return Err(HandshakeError::Rejected { code: error_code, message: error_string });
        }
        other => {
            return Err(HandshakeError::UnexpectedMessage(
                "CAPS_RECEIVED",
                format!("{other:?}"),
            ));
        }
    };

    // Verify peer's signature.
    verify_msg(&caps_msg)?;

    let (peer_node_id, negotiated) = match &caps_msg {
        FederationMessage::Capabilities { node_id, negotiated, .. } => {
            (node_id.clone(), negotiated.clone())
        }
        _ => unreachable!(),
    };

    // Send federation.accept with derived session_id (signed).
    let accept_ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let session_id = make_session_id(&our_node_id, &peer_node_id, &accept_ts);

    let accept = sign_msg(
        FederationMessage::Accept {
            protocol_version: "0.1".to_string(),
            node_id: our_node_id,
            session_id: session_id.clone(),
            timestamp: accept_ts,
            signature: None,
        },
        signing_key,
    );
    conn.send_federation(&accept).await?;

    Ok(FederationSession {
        peer_node_id,
        session_id,
        negotiated_serialisation: negotiated.serialisation,
        negotiated_version: negotiated.protocol_version,
        shared_spaces,
    })
}

// ── Receiving Node (Node B) ───────────────────────────────────────────────────

/// Run the federation handshake as the **receiving** Node.
///
/// Sequence: receive hello → send capabilities → receive accept → ACTIVE.
pub async fn run_receiving<S>(
    conn: &mut Connection<S>,
    signing_key: &SigningKey,
    caps: FederationCapabilities,
) -> Result<FederationSession, HandshakeError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let our_node_id = node_id_from_key(&signing_key.verifying_key());

    // Wait for federation.hello.
    let inbound = recv_with_timeout(conn).await?;
    let hello_msg = match inbound {
        Inbound::Federation(fm @ FederationMessage::Hello { .. }) => fm,
        other => {
            return Err(HandshakeError::UnexpectedMessage("IDLE", format!("{other:?}")));
        }
    };

    // Verify peer's signature.
    verify_msg(&hello_msg)?;

    let (peer_node_id, peer_caps, shared_spaces, peer_version) = match hello_msg {
        FederationMessage::Hello {
            node_id, capabilities, shared_spaces, protocol_version, ..
        } => (node_id, capabilities, shared_spaces, protocol_version),
        _ => unreachable!(),
    };

    // Negotiate serialisation — json is the mandatory baseline so this always succeeds
    // with Phase 1 nodes; the error path handles future optional-only format mismatches.
    let serial = match negotiate_serialisation(&caps.serialisation, &peer_caps.serialisation) {
        Some(s) => s,
        None => {
            let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let reject = sign_msg(
                FederationMessage::Reject {
                    protocol_version: "0.1".to_string(),
                    node_id: our_node_id,
                    error_code: 2001,
                    error_string: "no_common_capabilities".to_string(),
                    timestamp: ts,
                    signature: None,
                },
                signing_key,
            );
            let _ = conn.send_federation(&reject).await;
            return Err(HandshakeError::NoCommonCapabilities);
        }
    };

    let neg_version = match negotiate_version("0.1", &peer_version) {
        Some(v) => v,
        None => {
            let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let reject = sign_msg(
                FederationMessage::Reject {
                    protocol_version: "0.1".to_string(),
                    node_id: our_node_id,
                    error_code: 2002,
                    error_string: "version_incompatible".to_string(),
                    timestamp: ts,
                    signature: None,
                },
                signing_key,
            );
            let _ = conn.send_federation(&reject).await;
            return Err(HandshakeError::VersionIncompatible);
        }
    };

    // Send federation.capabilities (signed).
    let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let negotiated = NegotiatedCapabilities {
        serialisation: serial.clone(),
        protocol_version: neg_version.clone(),
    };
    let caps_msg = sign_msg(
        FederationMessage::Capabilities {
            protocol_version: "0.1".to_string(),
            node_id: our_node_id.clone(),
            capabilities: caps,
            negotiated,
            timestamp: ts,
            signature: None,
        },
        signing_key,
    );
    conn.send_federation(&caps_msg).await?;

    // Wait for federation.accept.
    let inbound = recv_with_timeout(conn).await?;
    let accept_msg = match inbound {
        Inbound::Federation(fm @ FederationMessage::Accept { .. }) => fm,
        Inbound::Federation(FederationMessage::Reject { error_code, error_string, .. }) => {
            return Err(HandshakeError::Rejected { code: error_code, message: error_string });
        }
        other => {
            return Err(HandshakeError::UnexpectedMessage(
                "CAPS_SENT",
                format!("{other:?}"),
            ));
        }
    };

    // Verify initiating Node's accept signature.
    verify_msg(&accept_msg)?;

    let session_id = match accept_msg {
        FederationMessage::Accept { session_id, .. } => session_id,
        _ => unreachable!(),
    };

    Ok(FederationSession {
        peer_node_id,
        session_id,
        negotiated_serialisation: serial,
        negotiated_version: neg_version,
        shared_spaces,
    })
}

// ── Capability negotiation (spec 3.4.4) ──────────────────────────────────────

/// Select the highest-preference format that both sides support.
/// `our_prefs` is ordered most-preferred first; we pick the first entry from
/// our list that also appears in `their_prefs`.
pub fn negotiate_serialisation(our_prefs: &[String], their_prefs: &[String]) -> Option<String> {
    for format in our_prefs {
        if their_prefs.iter().any(|f| f == format) {
            return Some(format.clone());
        }
    }
    None
}

/// Select the lower minor version between two "major.minor" strings.
/// Returns None if major versions differ (caught at transport level first).
pub fn negotiate_version(ours: &str, theirs: &str) -> Option<String> {
    let (our_maj, our_min) = parse_version(ours)?;
    let (their_maj, their_min) = parse_version(theirs)?;
    if our_maj != their_maj {
        return None;
    }
    Some(format!("{}.{}", our_maj, our_min.min(their_min)))
}

fn parse_version(v: &str) -> Option<(u32, u32)> {
    let mut parts = v.splitn(2, '.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    Some((major, minor))
}

// ── Signing / verification ────────────────────────────────────────────────────

/// Sign a federation message and return it with the `signature` field populated.
pub fn sign_msg(msg: FederationMessage, key: &SigningKey) -> FederationMessage {
    let canonical = canonical_json_of(&msg);
    let sig = signing::sign(key, canonical.as_bytes());
    with_signature(msg, sig)
}

/// Verify the signature on an incoming federation message.
/// Returns `Err(HandshakeError::SignatureInvalid)` if signature is absent or wrong.
pub fn verify_msg(msg: &FederationMessage) -> Result<(), HandshakeError> {
    let (node_id, sig_str) = extract_id_and_sig(msg)?;
    let vk = parse_verifying_key(node_id)?;
    let canonical = canonical_json_of(msg);
    signing::verify(&vk, canonical.as_bytes(), sig_str)
        .map_err(|_| HandshakeError::SignatureInvalid)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Derive a node_id URI from a verifying key.
pub fn node_id_from_key(vk: &VerifyingKey) -> String {
    format!("xgen://pubkey/ed25519:{}", encoding::encode(vk.as_bytes()))
}

/// session_id = hash_uri(sorted(node_a, node_b) || timestamp).
/// Nodes are sorted so the same pair always produces the same session_id.
fn make_session_id(node_a: &str, node_b: &str, timestamp: &str) -> String {
    let (first, second) = if node_a <= node_b {
        (node_a, node_b)
    } else {
        (node_b, node_a)
    };
    let data = format!("{}{}{}", first, second, timestamp);
    hashing::hash_uri(data.as_bytes())
}

async fn recv_with_timeout<S>(
    conn: &mut Connection<S>,
) -> Result<Inbound, HandshakeError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    tokio::time::timeout(Duration::from_secs(WAIT_TIMEOUT_SECS), conn.recv())
        .await
        .map_err(|_| HandshakeError::Timeout)?
        .map_err(HandshakeError::Transport)
}

fn canonical_json_of(msg: &FederationMessage) -> String {
    let v = serde_json::to_value(msg).expect("FederationMessage is always serialisable");
    let order = field_order_for(msg);
    canonical_object_json(&v, order)
}

fn field_order_for(msg: &FederationMessage) -> &'static [&'static str] {
    match msg {
        FederationMessage::Hello { .. } => HELLO_FIELDS,
        FederationMessage::Capabilities { .. } => CAPS_FIELDS,
        FederationMessage::Accept { .. } => ACCEPT_FIELDS,
        FederationMessage::Reject { .. } => REJECT_FIELDS,
        FederationMessage::Goodbye { .. } => GOODBYE_FIELDS,
    }
}

fn with_signature(msg: FederationMessage, sig: String) -> FederationMessage {
    match msg {
        FederationMessage::Hello {
            protocol_version, node_id, capabilities, shared_spaces, timestamp, ..
        } => FederationMessage::Hello {
            protocol_version, node_id, capabilities, shared_spaces, timestamp,
            signature: Some(sig),
        },
        FederationMessage::Capabilities {
            protocol_version, node_id, capabilities, negotiated, timestamp, ..
        } => FederationMessage::Capabilities {
            protocol_version, node_id, capabilities, negotiated, timestamp,
            signature: Some(sig),
        },
        FederationMessage::Accept {
            protocol_version, node_id, session_id, timestamp, ..
        } => FederationMessage::Accept {
            protocol_version, node_id, session_id, timestamp,
            signature: Some(sig),
        },
        FederationMessage::Reject {
            protocol_version, node_id, error_code, error_string, timestamp, ..
        } => FederationMessage::Reject {
            protocol_version, node_id, error_code, error_string, timestamp,
            signature: Some(sig),
        },
        FederationMessage::Goodbye {
            protocol_version, node_id, reason, session_id, timestamp, ..
        } => FederationMessage::Goodbye {
            protocol_version, node_id, reason, session_id, timestamp,
            signature: Some(sig),
        },
    }
}

fn extract_id_and_sig(msg: &FederationMessage) -> Result<(&str, &str), HandshakeError> {
    let (node_id, sig_opt) = match msg {
        FederationMessage::Hello { node_id, signature, .. } => {
            (node_id.as_str(), signature.as_deref())
        }
        FederationMessage::Capabilities { node_id, signature, .. } => {
            (node_id.as_str(), signature.as_deref())
        }
        FederationMessage::Accept { node_id, signature, .. } => {
            (node_id.as_str(), signature.as_deref())
        }
        FederationMessage::Reject { node_id, signature, .. } => {
            (node_id.as_str(), signature.as_deref())
        }
        FederationMessage::Goodbye { node_id, signature, .. } => {
            (node_id.as_str(), signature.as_deref())
        }
    };
    let sig = sig_opt.ok_or(HandshakeError::SignatureInvalid)?;
    Ok((node_id, sig))
}

fn parse_verifying_key(node_id: &str) -> Result<VerifyingKey, HandshakeError> {
    let b64 = node_id
        .strip_prefix("xgen://pubkey/ed25519:")
        .ok_or(HandshakeError::SignatureInvalid)?;
    let bytes = encoding::decode(b64).map_err(|_| HandshakeError::SignatureInvalid)?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| HandshakeError::SignatureInvalid)?;
    VerifyingKey::from_bytes(&arr).map_err(|_| HandshakeError::SignatureInvalid)
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::keypair;

    fn json_only_caps() -> FederationCapabilities {
        FederationCapabilities::default()
    }

    #[test]
    fn negotiate_serialisation_picks_highest_preference() {
        let ours = vec!["json".to_string(), "msgpack".to_string()];
        let theirs = vec!["json".to_string(), "cbor".to_string()];
        // Our first preference ("json") appears in theirs — select it.
        assert_eq!(negotiate_serialisation(&ours, &theirs).unwrap(), "json");
    }

    #[test]
    fn negotiate_serialisation_picks_first_common() {
        let ours = vec!["cbor".to_string(), "json".to_string()];
        let theirs = vec!["json".to_string(), "cbor".to_string()];
        // Our first preference ("cbor") appears in theirs — select it (not json).
        assert_eq!(negotiate_serialisation(&ours, &theirs).unwrap(), "cbor");
    }

    #[test]
    fn negotiate_serialisation_no_overlap_returns_none() {
        let ours = vec!["cbor".to_string()];
        let theirs = vec!["msgpack".to_string()];
        assert!(negotiate_serialisation(&ours, &theirs).is_none());
    }

    #[test]
    fn negotiate_version_lower_minor_wins() {
        assert_eq!(negotiate_version("0.3", "0.1").unwrap(), "0.1");
        assert_eq!(negotiate_version("0.1", "0.3").unwrap(), "0.1");
        assert_eq!(negotiate_version("0.2", "0.2").unwrap(), "0.2");
    }

    #[test]
    fn negotiate_version_major_mismatch_returns_none() {
        assert!(negotiate_version("0.1", "1.0").is_none());
    }

    #[test]
    fn sign_verify_hello_round_trip() {
        let key = keypair::generate();
        let node_id = node_id_from_key(&key.verifying_key());
        let msg = FederationMessage::Hello {
            protocol_version: "0.1".to_string(),
            node_id,
            capabilities: json_only_caps(),
            shared_spaces: vec![],
            timestamp: "2026-04-27T12:00:00.000Z".to_string(),
            signature: None,
        };
        let signed = sign_msg(msg, &key);
        assert!(verify_msg(&signed).is_ok());
    }

    #[test]
    fn sign_verify_capabilities_round_trip() {
        let key = keypair::generate();
        let node_id = node_id_from_key(&key.verifying_key());
        let msg = FederationMessage::Capabilities {
            protocol_version: "0.1".to_string(),
            node_id,
            capabilities: json_only_caps(),
            negotiated: NegotiatedCapabilities {
                serialisation: "json".to_string(),
                protocol_version: "0.1".to_string(),
            },
            timestamp: "2026-04-27T12:00:00.000Z".to_string(),
            signature: None,
        };
        let signed = sign_msg(msg, &key);
        assert!(verify_msg(&signed).is_ok());
    }

    #[test]
    fn sign_verify_accept_round_trip() {
        let key = keypair::generate();
        let node_id = node_id_from_key(&key.verifying_key());
        let msg = FederationMessage::Accept {
            protocol_version: "0.1".to_string(),
            node_id,
            session_id: "xgen://hash/sha256:abc123".to_string(),
            timestamp: "2026-04-27T12:00:00.000Z".to_string(),
            signature: None,
        };
        let signed = sign_msg(msg, &key);
        assert!(verify_msg(&signed).is_ok());
    }

    #[test]
    fn tampered_node_id_fails_verification() {
        let key = keypair::generate();
        let other_key = keypair::generate();
        let node_id = node_id_from_key(&key.verifying_key());
        let msg = FederationMessage::Hello {
            protocol_version: "0.1".to_string(),
            node_id,
            capabilities: json_only_caps(),
            shared_spaces: vec![],
            timestamp: "2026-04-27T12:00:00.000Z".to_string(),
            signature: None,
        };
        let signed = sign_msg(msg, &key);
        // Swap the node_id to a different key — verification must fail.
        let tampered = match signed {
            FederationMessage::Hello { protocol_version, capabilities, shared_spaces, timestamp, signature, .. } =>
                FederationMessage::Hello {
                    protocol_version,
                    node_id: node_id_from_key(&other_key.verifying_key()),
                    capabilities,
                    shared_spaces,
                    timestamp,
                    signature,
                },
            _ => unreachable!(),
        };
        assert!(verify_msg(&tampered).is_err());
    }

    #[test]
    fn session_id_is_deterministic_and_sorted() {
        let a = "xgen://pubkey/ed25519:AAAA";
        let b = "xgen://pubkey/ed25519:BBBB";
        let ts = "2026-04-27T12:00:00.000Z";
        let id_ab = make_session_id(a, b, ts);
        let id_ba = make_session_id(b, a, ts);
        // Sorting makes both calls identical.
        assert_eq!(id_ab, id_ba);
        // Different timestamp → different session_id.
        let id_other = make_session_id(a, b, "2026-04-27T13:00:00.000Z");
        assert_ne!(id_ab, id_other);
    }

    #[test]
    fn message_type_field_serialises_correctly() {
        let key = keypair::generate();
        let node_id = node_id_from_key(&key.verifying_key());
        let hello = FederationMessage::Hello {
            protocol_version: "0.1".to_string(),
            node_id,
            capabilities: json_only_caps(),
            shared_spaces: vec![],
            timestamp: "2026-04-27T12:00:00.000Z".to_string(),
            signature: None,
        };
        let v: serde_json::Value = serde_json::to_value(&hello).unwrap();
        assert_eq!(v["type"], "federation.hello");
        // Unsigned message must not include signature field.
        assert!(v.get("signature").is_none());
    }

    #[test]
    fn federation_capabilities_default_is_json_only() {
        let caps = FederationCapabilities::default();
        assert_eq!(caps.serialisation, vec!["json"]);
        assert!(caps.compression.is_empty());
        assert!(caps.extensions.is_empty());
    }
}
