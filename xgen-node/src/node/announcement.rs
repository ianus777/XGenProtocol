// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// Node announcement — generation, signing, verification, persistence (spec 3.5.3–3.5.6).
//
// A node_announcement is the Node's signed public declaration: who it is, where to
// reach it, what it can do, and until when the record is valid.  Any peer can verify
// it without a third party (self-certifying via the embedded node_id public key).

use std::path::Path;

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::crypto::{encoding, signing};
use crate::wire::canonical::canonical_object_json;

/// Phase 1 announcement TTL: 90 days (spec 3.5.6).
const TTL_DAYS: i64 = 90;

/// Pubkey URI prefix (spec 3.5.2).
const PUBKEY_URI_PREFIX: &str = "xgen://pubkey/ed25519:";

/// Fixed field order for canonical form, excluding `signature` (spec 3.5.3).
/// `operator_display_name` is optional — it is skipped when absent.
const CANONICAL_FIELD_ORDER: &[&str] = &[
    "protocol_version",
    "type",
    "node_id",
    "endpoint",
    "capabilities",
    "auth_tiers_served",
    "operator_display_name",
    "announcement_version",
    "valid_until",
    "timestamp",
];

// ── Capability set ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub serialisation: Vec<String>,
    pub compression: Vec<String>,
    pub extensions: Vec<String>,
}

impl Default for NodeCapabilities {
    fn default() -> Self {
        Self {
            serialisation: vec!["json".to_string()],
            compression: vec![],
            extensions: vec![],
        }
    }
}

// ── Announcement struct ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAnnouncement {
    pub protocol_version: String,

    #[serde(rename = "type")]
    pub msg_type: String, // always "node_announcement"

    /// xgen://pubkey/ed25519:<base64url-pubkey> (spec 3.5.2)
    pub node_id: String,

    /// Full WebSocket endpoint URI (spec 3.5.3)
    pub endpoint: String,

    pub capabilities: NodeCapabilities,

    pub auth_tiers_served: Vec<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_display_name: Option<String>,

    /// Monotonically increasing — higher version supersedes lower (spec 3.5.6)
    pub announcement_version: u64,

    /// RFC 3339 datetime when this announcement expires (spec 3.5.6)
    pub valid_until: String,

    /// RFC 3339 datetime when this announcement was created
    pub timestamp: String,

    /// Ed25519 signature over the canonical form (spec 3.5.4)
    pub signature: String,
}

// ── Error type ─────────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AnnouncementError {
    #[error("announcement has expired")]
    Expired,
    #[error("invalid node_id format")]
    InvalidNodeId,
    #[error("signature verification failed")]
    SignatureInvalid,
    #[error("I/O error: {0}")]
    Io(String),
    #[error("JSON error: {0}")]
    Json(String),
}

// ── Generation ─────────────────────────────────────────────────────────────────

impl NodeAnnouncement {
    /// Generate a new signed announcement for a Node.
    /// `announcement_version` should be 1 on first run, incrementing on refresh.
    pub fn generate(
        signing_key: &SigningKey,
        endpoint: &str,
        operator_display_name: Option<&str>,
        announcement_version: u64,
    ) -> Self {
        let pubkey_b64 = encoding::encode(signing_key.verifying_key().as_bytes());
        let node_id = format!("{}{}", PUBKEY_URI_PREFIX, pubkey_b64);
        let now = Utc::now();
        let valid_until = now + Duration::days(TTL_DAYS);

        let mut ann = NodeAnnouncement {
            protocol_version: "0.1".to_string(),
            msg_type: "node_announcement".to_string(),
            node_id,
            endpoint: endpoint.to_string(),
            capabilities: NodeCapabilities::default(),
            auth_tiers_served: vec![1],
            operator_display_name: operator_display_name.map(str::to_string),
            announcement_version,
            valid_until: valid_until.to_rfc3339_opts(SecondsFormat::Millis, true),
            timestamp: now.to_rfc3339_opts(SecondsFormat::Millis, true),
            signature: String::new(), // filled below
        };

        let canonical = ann.canonical_json();
        ann.signature = signing::sign(signing_key, canonical.as_bytes());
        ann
    }

    // ── Verification ───────────────────────────────────────────────────────────

    /// Verify the announcement's signature and expiry (spec 3.5.4).
    pub fn verify(&self) -> Result<(), AnnouncementError> {
        // Check expiry.
        if self.is_expired() {
            return Err(AnnouncementError::Expired);
        }

        // Extract verifying key from node_id.
        let vk = self.verifying_key()?;

        // Canonical form excludes `signature`.
        let canonical = self.canonical_json();

        // Verify signature.
        signing::verify(&vk, canonical.as_bytes(), &self.signature)
            .map_err(|_| AnnouncementError::SignatureInvalid)
    }

    /// True if `valid_until` is in the past.
    pub fn is_expired(&self) -> bool {
        match DateTime::parse_from_rfc3339(&self.valid_until) {
            Ok(t) => Utc::now() > t,
            Err(_) => true, // malformed datetime counts as expired
        }
    }

    /// True if `self` supersedes `other` — i.e., `self` has a strictly higher
    /// `announcement_version` and the same `node_id` (spec 3.5.6).
    pub fn supersedes(&self, other: &Self) -> bool {
        self.node_id == other.node_id
            && self.announcement_version > other.announcement_version
    }

    // ── Persistence ────────────────────────────────────────────────────────────

    /// Save the announcement as a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), AnnouncementError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AnnouncementError::Io(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AnnouncementError::Json(e.to_string()))?;
        std::fs::write(path, json).map_err(|e| AnnouncementError::Io(e.to_string()))
    }

    /// Load an announcement from a JSON file and verify it.
    pub fn load(path: &Path) -> Result<Self, AnnouncementError> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| AnnouncementError::Io(e.to_string()))?;
        let ann: Self = serde_json::from_str(&json)
            .map_err(|e| AnnouncementError::Json(e.to_string()))?;
        ann.verify()?;
        Ok(ann)
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    fn canonical_json(&self) -> String {
        // Serialise to Value, then apply the fixed field order.
        // `signature` is not in CANONICAL_FIELD_ORDER so it is excluded automatically.
        let v = serde_json::to_value(self).expect("NodeAnnouncement is always serialisable");
        canonical_object_json(&v, CANONICAL_FIELD_ORDER)
    }

    fn verifying_key(&self) -> Result<ed25519_dalek::VerifyingKey, AnnouncementError> {
        let b64 = self
            .node_id
            .strip_prefix(PUBKEY_URI_PREFIX)
            .ok_or(AnnouncementError::InvalidNodeId)?;
        let bytes = encoding::decode(b64).map_err(|_| AnnouncementError::InvalidNodeId)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| AnnouncementError::InvalidNodeId)?;
        ed25519_dalek::VerifyingKey::from_bytes(&arr)
            .map_err(|_| AnnouncementError::InvalidNodeId)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::keypair;
    use tempfile::NamedTempFile;

    fn make_announcement(key: &SigningKey, version: u64) -> NodeAnnouncement {
        NodeAnnouncement::generate(key, "ws://127.0.0.1:9000/", None, version)
    }

    #[test]
    fn generate_produces_valid_signature() {
        let key = keypair::generate();
        let ann = make_announcement(&key, 1);
        assert!(ann.verify().is_ok());
    }

    #[test]
    fn node_id_matches_signing_key() {
        let key = keypair::generate();
        let ann = make_announcement(&key, 1);
        let expected_id = format!(
            "{}{}",
            PUBKEY_URI_PREFIX,
            encoding::encode(key.verifying_key().as_bytes())
        );
        assert_eq!(ann.node_id, expected_id);
    }

    #[test]
    fn tampered_endpoint_invalidates_signature() {
        let key = keypair::generate();
        let mut ann = make_announcement(&key, 1);
        ann.endpoint = "ws://evil.example.com/".to_string();
        assert_eq!(ann.verify(), Err(AnnouncementError::SignatureInvalid));
    }

    #[test]
    fn tampered_node_id_invalidates_signature() {
        let key = keypair::generate();
        let other_key = keypair::generate();
        let mut ann = make_announcement(&key, 1);
        // Replace node_id with a different key's ID — signature mismatch.
        ann.node_id = format!(
            "{}{}",
            PUBKEY_URI_PREFIX,
            encoding::encode(other_key.verifying_key().as_bytes())
        );
        assert_eq!(ann.verify(), Err(AnnouncementError::SignatureInvalid));
    }

    #[test]
    fn higher_version_supersedes_lower() {
        let key = keypair::generate();
        let v1 = make_announcement(&key, 1);
        let v2 = make_announcement(&key, 2);
        assert!(v2.supersedes(&v1));
        assert!(!v1.supersedes(&v2));
    }

    #[test]
    fn same_version_does_not_supersede() {
        let key = keypair::generate();
        let a = make_announcement(&key, 3);
        let b = make_announcement(&key, 3);
        assert!(!a.supersedes(&b));
        assert!(!b.supersedes(&a));
    }

    #[test]
    fn different_node_does_not_supersede() {
        let key1 = keypair::generate();
        let key2 = keypair::generate();
        let a = make_announcement(&key1, 5);
        let b = make_announcement(&key2, 1);
        // b has lower version but different node — not a supersession relationship
        assert!(!a.supersedes(&b));
        assert!(!b.supersedes(&a));
    }

    #[test]
    fn expired_announcement_rejected() {
        let key = keypair::generate();
        let mut ann = make_announcement(&key, 1);
        // Set valid_until to the past and re-sign.
        ann.valid_until = "2020-01-01T00:00:00.000Z".to_string();
        // Re-sign so the signature is valid (expiry is checked independently).
        let canonical = ann.canonical_json();
        ann.signature = signing::sign(&key, canonical.as_bytes());
        assert_eq!(ann.verify(), Err(AnnouncementError::Expired));
    }

    #[test]
    fn with_display_name() {
        let key = keypair::generate();
        let ann = NodeAnnouncement::generate(
            &key,
            "ws://127.0.0.1:9000/",
            Some("Test Node"),
            1,
        );
        assert_eq!(ann.operator_display_name.as_deref(), Some("Test Node"));
        assert!(ann.verify().is_ok());
    }

    #[test]
    fn save_load_round_trip() {
        let key = keypair::generate();
        let ann = make_announcement(&key, 1);
        let tmp = NamedTempFile::new().unwrap();
        ann.save(tmp.path()).unwrap();
        let loaded = NodeAnnouncement::load(tmp.path()).unwrap();
        assert_eq!(loaded.node_id, ann.node_id);
        assert_eq!(loaded.announcement_version, ann.announcement_version);
        assert_eq!(loaded.endpoint, ann.endpoint);
    }

    #[test]
    fn announcement_type_field_is_correct() {
        let key = keypair::generate();
        let ann = make_announcement(&key, 1);
        assert_eq!(ann.msg_type, "node_announcement");
        // Verify it serialises to "type" in JSON.
        let json: serde_json::Value = serde_json::to_value(&ann).unwrap();
        assert_eq!(json["type"], "node_announcement");
    }

    #[test]
    fn phase1_capabilities_are_json_only() {
        let key = keypair::generate();
        let ann = make_announcement(&key, 1);
        assert_eq!(ann.capabilities.serialisation, vec!["json"]);
        assert!(ann.capabilities.compression.is_empty());
        assert!(ann.capabilities.extensions.is_empty());
    }
}
