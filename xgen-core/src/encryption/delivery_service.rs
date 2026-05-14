// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Node-side MLS Delivery Service (spec 3.10.1–3.10.6).
//
// The Node acts as an MLS Delivery Service (DS):
//   - Routes MLS control messages (Welcome, Commit, Proposal) to Room members
//   - Stores and distributes KeyPackages (single-use)
//   - Tracks epoch state per Room (opaque counter — no key material)
//   - Stores encrypted content blobs without inspecting or modifying them
//
// The Node has no access to MLS private key material.

use std::collections::HashMap;


// ── Delivery record ───────────────────────────────────────────────────────────

/// An MLS message queued for delivery to a specific recipient device.
#[derive(Debug, Clone)]
pub struct PendingDelivery {
    pub room_id: String,
    pub recipient_identity_id: String,
    pub recipient_device_id: String,
    pub msg_type: MlsMessageType,
    /// Opaque bytes — the Node does not inspect MLS message content.
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MlsMessageType {
    Welcome,
    Commit,
    Proposal,
}

// ── Delivery service ──────────────────────────────────────────────────────────

/// Node-side MLS Delivery Service state.
///
/// In production, delivery happens over active WebSocket connections.
/// This struct holds the queue for testing and for offline delivery.
#[derive(Debug, Default)]
pub struct MlsDeliveryService {
    /// Pending delivery queue per room_id.
    pending: HashMap<String, Vec<PendingDelivery>>,
}

impl MlsDeliveryService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Route an MLS message to a specific recipient device.
    ///
    /// In production, if the recipient is connected, this delivers immediately.
    /// Otherwise it is queued for retrieval on next connect.
    pub fn route(
        &mut self,
        room_id: &str,
        recipient_identity_id: &str,
        recipient_device_id: &str,
        msg_type: MlsMessageType,
        payload: Vec<u8>,
    ) {
        self.pending.entry(room_id.to_string()).or_default().push(PendingDelivery {
            room_id: room_id.to_string(),
            recipient_identity_id: recipient_identity_id.to_string(),
            recipient_device_id: recipient_device_id.to_string(),
            msg_type,
            payload,
        });
    }

    /// Drain all pending deliveries for a recipient in a Room.
    pub fn drain_for_recipient(
        &mut self,
        room_id: &str,
        identity_id: &str,
    ) -> Vec<PendingDelivery> {
        let queue = match self.pending.get_mut(room_id) {
            Some(q) => q,
            None => return vec![],
        };
        let (matching, remaining): (Vec<_>, Vec<_>) =
            queue.drain(..).partition(|d| d.recipient_identity_id == identity_id);
        *queue = remaining;
        matching
    }

    /// Count pending deliveries for a recipient in a Room.
    pub fn pending_count(&self, room_id: &str, identity_id: &str) -> usize {
        self.pending
            .get(room_id)
            .map(|q| q.iter().filter(|d| d.recipient_identity_id == identity_id).count())
            .unwrap_or(0)
    }
}

// ── Encrypted content handling ────────────────────────────────────────────────

/// Validate that an encrypted content blob is non-empty and pass it through
/// without decryption. The Node stores the opaque blob as-is.
///
/// Returns the blob unchanged. Returns `None` if the blob is empty (invalid).
pub fn handle_encrypted_content(blob: Vec<u8>) -> Option<Vec<u8>> {
    if blob.is_empty() {
        None
    } else {
        Some(blob)
    }
}

/// Return whether the content field of an Event is encrypted.
/// Convention: encrypted content is a base64url string starting with "enc:".
pub fn is_encrypted_content(content_str: &str) -> bool {
    content_str.starts_with("enc:")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mls_welcome_routed_to_new_member() {
        let mut ds = MlsDeliveryService::new();

        let welcome_payload = b"opaque-welcome-bytes".to_vec();
        ds.route("room1", "bob", "bob-dev1", MlsMessageType::Welcome, welcome_payload.clone());

        assert_eq!(ds.pending_count("room1", "bob"), 1);

        let deliveries = ds.drain_for_recipient("room1", "bob");
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].msg_type, MlsMessageType::Welcome);
        assert_eq!(deliveries[0].payload, welcome_payload);
        assert_eq!(deliveries[0].recipient_identity_id, "bob");
    }

    #[test]
    fn node_cannot_decrypt_content() {
        // The Node receives an encrypted blob and passes it through unchanged.
        let blob = b"enc:base64url-encrypted-content-bytes".to_vec();
        let stored = handle_encrypted_content(blob.clone()).unwrap();
        // The stored blob is identical to the received blob — no decryption.
        assert_eq!(stored, blob);
    }

    #[test]
    fn empty_encrypted_content_rejected() {
        let result = handle_encrypted_content(vec![]);
        assert!(result.is_none());
    }

    #[test]
    fn is_encrypted_content_check() {
        assert!(is_encrypted_content("enc:abc123"));
        assert!(!is_encrypted_content(r#"{"text":"hello"}"#));
    }

    #[test]
    fn route_multiple_message_types() {
        let mut ds = MlsDeliveryService::new();
        ds.route("room1", "alice", "alice-dev1", MlsMessageType::Commit, vec![1, 2, 3]);
        ds.route("room1", "alice", "alice-dev1", MlsMessageType::Proposal, vec![4, 5, 6]);
        ds.route("room1", "bob", "bob-dev1", MlsMessageType::Welcome, vec![7, 8, 9]);

        assert_eq!(ds.pending_count("room1", "alice"), 2);
        assert_eq!(ds.pending_count("room1", "bob"), 1);

        let alice_msgs = ds.drain_for_recipient("room1", "alice");
        assert_eq!(alice_msgs.len(), 2);
        // Bob's message is unaffected.
        assert_eq!(ds.pending_count("room1", "bob"), 1);
    }
}
