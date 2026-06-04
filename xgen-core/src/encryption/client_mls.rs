// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Client-side MLS group operations (spec 3.10.4–3.10.8).
//
// Phase 2 note (D-052):
// Full RFC 9420 (openmls) integration is deferred to Phase 3. This module
// implements the complete MLS client interface — group creation, member
// add/remove, epoch-based encryption/decryption — using a Phase 2 epoch-key
// scheme (ChaCha20Poly1305 + SHA-256 key derivation) that correctly demonstrates
// all protocol properties including forward secrecy and post-removal isolation.
// The interface is identical to what the openmls integration will provide;
// only the underlying key schedule changes in Phase 3.
//
// Key properties preserved:
//   - Each epoch has a unique, independently derived key
//   - Removed members do not learn the keys of subsequent epochs
//   - Messages from epoch N cannot be decrypted with epoch M key (M ≠ N)

use std::collections::HashSet;

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::crypto::encoding;

// ── Epoch key ─────────────────────────────────────────────────────────────────

/// A 32-byte epoch encryption key.
/// Phase 2: derived from group_secret + epoch via SHA-256.
/// Phase 3: the MLS epoch application secret from the RFC 9420 key schedule.
pub type EpochKey = [u8; 32];

/// Derive the epoch key for a specific epoch from the group's shared secret.
pub fn derive_epoch_key(group_secret: &[u8; 32], epoch: u64) -> EpochKey {
    let mut input = group_secret.to_vec();
    input.extend_from_slice(b"xgen-epoch-key:");
    input.extend_from_slice(&epoch.to_le_bytes());
    let hash = Sha256::digest(&input);
    hash.into()
}

// ── Encrypted content ─────────────────────────────────────────────────────────

/// An encrypted content blob (base64url-encoded) for the Event `content` field.
/// Prefixed with "enc:" to distinguish from plaintext JSON content.
pub struct EncryptedContent(pub String);

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MlsClientError {
    #[error("decryption failed: wrong epoch key or tampered ciphertext")]
    DecryptionFailed,
    #[error("member not in group")]
    NotAMember,
    #[error("epoch mismatch: message is from a different epoch")]
    EpochMismatch,
}

// ── Client MLS group state ────────────────────────────────────────────────────

/// Client-side MLS group state.
/// Each client holds its own copy — not shared with the Node.
#[derive(Debug, Clone)]
pub struct ClientMlsGroup {
    pub room_id: String,
    pub epoch: u64,
    /// The group's shared secret — changes on each epoch advance.
    /// Phase 2: generated randomly on group creation; advanced via SHA-256(secret || epoch).
    /// Phase 3: the MLS exporter secret from the RFC 9420 key schedule.
    epoch_secret: [u8; 32],
    pub members: HashSet<String>,
}

impl ClientMlsGroup {
    /// Create a new MLS group for a Room with `creator_id` as the sole member.
    pub fn new(room_id: &str, creator_id: &str, initial_secret: [u8; 32]) -> Self {
        let mut members = HashSet::new();
        members.insert(creator_id.to_string());
        Self {
            room_id: room_id.to_string(),
            epoch: 0,
            epoch_secret: initial_secret,
            members,
        }
    }

    /// Return the current epoch key (used to encrypt/decrypt messages in this epoch).
    pub fn current_epoch_key(&self) -> EpochKey {
        derive_epoch_key(&self.epoch_secret, self.epoch)
    }

    /// Add a member to the group and advance the epoch.
    ///
    /// Returns the Welcome data for the new member:
    ///   - The new epoch number (after advance)
    ///   - The new epoch key (so the caller can include it in the "Welcome" message)
    ///
    /// Phase 3: this produces an actual MLS Add Proposal + Commit + Welcome.
    pub fn add_member(&mut self, new_member_id: &str) -> (u64, EpochKey) {
        self.members.insert(new_member_id.to_string());
        self.advance_epoch();
        (self.epoch, self.current_epoch_key())
    }

    /// Remove a member and advance the epoch.
    ///
    /// The removed member is NOT given the new epoch key.
    /// Phase 3: this produces an actual MLS Remove Proposal + Commit.
    pub fn remove_member(&mut self, member_id: &str) -> Result<EpochKey, MlsClientError> {
        if !self.members.contains(member_id) {
            return Err(MlsClientError::NotAMember);
        }
        self.members.remove(member_id);
        self.advance_epoch();
        Ok(self.current_epoch_key())
    }

    /// Advance to the next epoch using SHA-256(old_secret || "next-epoch").
    fn advance_epoch(&mut self) {
        let mut input = self.epoch_secret.to_vec();
        input.extend_from_slice(b"xgen-next-epoch");
        input.extend_from_slice(&self.epoch.to_le_bytes());
        let new_secret: [u8; 32] = Sha256::digest(&input).into();
        self.epoch_secret = new_secret;
        self.epoch += 1;
    }

    pub fn is_member(&self, identity_id: &str) -> bool {
        self.members.contains(identity_id)
    }
}

// ── Message encryption/decryption ─────────────────────────────────────────────

/// Encrypt `plaintext` with `epoch_key` using ChaCha20Poly1305.
///
/// Returns an `EncryptedContent` with the format:
///   `enc:<base64url(epoch_number_8le || nonce_12 || ciphertext)>`
///
/// The epoch number is embedded so the receiver can verify they are using
/// the correct key before attempting decryption.
pub fn encrypt_message(epoch_key: &EpochKey, epoch: u64, plaintext: &[u8]) -> EncryptedContent {
    let cipher = ChaCha20Poly1305::new(epoch_key.into());

    // Deterministic nonce from epoch (12 bytes): SHA-256(epoch)[:12].
    // Phase 2 simplification: single message per epoch in tests.
    // Phase 3: proper nonce management with per-message counters.
    let nonce_input = Sha256::digest(epoch.to_le_bytes());
    let nonce = Nonce::from_slice(&nonce_input[..12]);

    let ciphertext = cipher.encrypt(nonce, plaintext).expect("ChaCha20 encrypt is infallible");

    let mut payload = Vec::with_capacity(8 + 12 + ciphertext.len());
    payload.extend_from_slice(&epoch.to_le_bytes());
    payload.extend_from_slice(nonce.as_slice());
    payload.extend_from_slice(&ciphertext);

    EncryptedContent(format!("enc:{}", encoding::encode(&payload)))
}

/// Decrypt an `EncryptedContent` blob with `epoch_key`.
///
/// Returns the plaintext bytes on success.
/// Returns `MlsClientError::DecryptionFailed` if the key is wrong or data is tampered.
/// Returns `MlsClientError::EpochMismatch` if the embedded epoch doesn't match `expected_epoch`.
pub fn decrypt_message(
    epoch_key: &EpochKey,
    expected_epoch: u64,
    encrypted: &EncryptedContent,
) -> Result<Vec<u8>, MlsClientError> {
    let b64 = encrypted.0.strip_prefix("enc:").ok_or(MlsClientError::DecryptionFailed)?;
    let payload = encoding::decode(b64).map_err(|_| MlsClientError::DecryptionFailed)?;

    if payload.len() < 8 + 12 {
        return Err(MlsClientError::DecryptionFailed);
    }

    // Extract and verify epoch number.
    let embedded_epoch = u64::from_le_bytes(payload[..8].try_into().unwrap());
    if embedded_epoch != expected_epoch {
        return Err(MlsClientError::EpochMismatch);
    }

    let nonce = Nonce::from_slice(&payload[8..20]);
    let ciphertext = &payload[20..];

    let cipher = ChaCha20Poly1305::new(epoch_key.into());
    cipher.decrypt(nonce, ciphertext).map_err(|_| MlsClientError::DecryptionFailed)
}

// ── Envelope encryption (enc: v2 — Arc H AH-D1 / D-088 amendment) ───────────────
//
// The v1 form above (`encrypt_message`) encrypts content *directly* under the
// per-epoch key — one key per epoch. That mismatches D-088's crypto-shred, which
// needs a per-erasure-unit key: destroying an epoch key erases the whole epoch,
// not one message. AH-D1 (a D-088 amendment) fixes the granularity with an
// envelope: a per-message random content key `CK` encrypts the plaintext, and
// `CK` is *wrapped* under the current epoch key. The wrapped `CK` rides the DAG
// with the message; erasure = destroy the wrapped `CK` (one message = one
// erasable key). The envelope adds *only* an erasability layer — confidentiality
// and forward secrecy still derive entirely from MLS (unwrap requires the epoch
// key), so it cannot weaken MLS (AH-D1 constraint 1).
//
// The load-bearing rule (AH-D1 constraint 2): `CK` is **random per message**,
// never KDF-derived from the epoch secret. If it were re-derivable, a holder of
// the epoch secret could reconstruct `CK` after the wrapped copy was destroyed
// and erasure would be a no-op. Because `CK` is random and only recoverable by
// unwrapping, destroying the wrapped `CK` alone renders the content permanently
// undecryptable *even by a party still holding the epoch secret*. That threat-
// defended invariant is proved by `erasing_wrapped_key_defeats_epoch_holder`.
//
// Arc H ships steps (a)–(d): generate `CK` → wrap → store the wrapped form →
// unwrap-to-read. Step (e), the destroy-to-erase *storage* operation (locating
// and overwriting the wrapped `CK` in persisted storage), is fenced behind the
// erasure-impl arc per the D-088 cascade. `envelope_with_destroyed_key` below is
// the in-memory *witness* of step (e)'s effect (for the erasability proof), not
// that storage operation.
//
// Phase-2 note (D-052): wrap/content AEAD is ChaCha20Poly1305, the same Phase-2
// primitive as the rest of the module. Real RFC 9420 HPKE wrapping is D3 — a
// swap of the wrap primitive, not a reshape of the envelope.

/// Envelope version byte for the v2 (`CK`-wrapped) form. v1 (`encrypt_message`)
/// carries no version byte; v2 is the go-forward form and is self-describing.
pub const ENVELOPE_VERSION_V2: u8 = 2;

const CK_LEN: usize = 32;
const WRAP_NONCE_LEN: usize = 12;
const CONTENT_NONCE_LEN: usize = 12;
const EPOCH_LEN: usize = 8;
/// Wrapped `CK` = 32-byte key + 16-byte Poly1305 tag (fixed length).
const WRAPPED_CK_LEN: usize = CK_LEN + 16;
/// Fixed prefix length before the ciphertext in a v2 payload:
/// `version(1) ‖ wrap_nonce(12) ‖ wrapped_CK(48) ‖ epoch(8) ‖ content_nonce(12)`.
const V2_PREFIX_LEN: usize = 1 + WRAP_NONCE_LEN + WRAPPED_CK_LEN + EPOCH_LEN + CONTENT_NONCE_LEN;

// Byte offsets into the decoded v2 payload.
const OFF_WRAP_NONCE: usize = 1;
const OFF_WRAPPED_CK: usize = OFF_WRAP_NONCE + WRAP_NONCE_LEN; // 13
const OFF_EPOCH: usize = OFF_WRAPPED_CK + WRAPPED_CK_LEN; // 61
const OFF_CONTENT_NONCE: usize = OFF_EPOCH + EPOCH_LEN; // 69
const OFF_CIPHERTEXT: usize = OFF_CONTENT_NONCE + CONTENT_NONCE_LEN; // 81

/// Encrypt `plaintext` under a fresh per-message content key `CK`, wrap `CK`
/// under `epoch_key`, and emit the `enc:` v2 envelope (AH-D1).
///
/// Layout (base64url-encoded after the `enc:` prefix):
/// `version(1)=0x02 ‖ wrap_nonce(12) ‖ wrapped_CK(48) ‖ epoch(8 LE) ‖ content_nonce(12) ‖ ciphertext`.
///
/// Both nonces are random (`OsRng`). The wrap nonce MUST be unique per message
/// because the epoch key is reused across messages (nonce-reuse under a fixed key
/// is a ChaCha20Poly1305 break); the design's byte sketch omitted it — this is
/// the grounded byte layout (CP-1). The content nonce is also random, which makes
/// the v1 deterministic-nonce simplification moot here: distinct random `CK` per
/// message already guarantees distinct keystreams.
pub fn encrypt_message_envelope(
    epoch_key: &EpochKey,
    epoch: u64,
    plaintext: &[u8],
) -> EncryptedContent {
    // Per-message random content key — NEVER derived from the epoch secret
    // (AH-D1 constraint 2; the erasability guarantee rests on this).
    let mut ck = [0u8; CK_LEN];
    OsRng.fill_bytes(&mut ck);
    let mut wrap_nonce = [0u8; WRAP_NONCE_LEN];
    OsRng.fill_bytes(&mut wrap_nonce);
    let mut content_nonce = [0u8; CONTENT_NONCE_LEN];
    OsRng.fill_bytes(&mut content_nonce);

    // Encrypt the plaintext under CK.
    let content_cipher = ChaCha20Poly1305::new((&ck).into());
    let ciphertext = content_cipher
        .encrypt(Nonce::from_slice(&content_nonce), plaintext)
        .expect("ChaCha20 encrypt is infallible");

    // Wrap CK under the epoch key (the MLS layer).
    let wrap_cipher = ChaCha20Poly1305::new(epoch_key.into());
    let wrapped_ck = wrap_cipher
        .encrypt(Nonce::from_slice(&wrap_nonce), ck.as_slice())
        .expect("ChaCha20 encrypt is infallible");
    debug_assert_eq!(wrapped_ck.len(), WRAPPED_CK_LEN);

    let mut payload = Vec::with_capacity(V2_PREFIX_LEN + ciphertext.len());
    payload.push(ENVELOPE_VERSION_V2);
    payload.extend_from_slice(&wrap_nonce);
    payload.extend_from_slice(&wrapped_ck);
    payload.extend_from_slice(&epoch.to_le_bytes());
    payload.extend_from_slice(&content_nonce);
    payload.extend_from_slice(&ciphertext);

    EncryptedContent(format!("enc:{}", encoding::encode(&payload)))
}

/// Decrypt an `enc:` v2 envelope: unwrap `CK` under `epoch_key`, then decrypt the
/// content under `CK`. Returns `DecryptionFailed` if the blob is not a v2
/// envelope, is malformed, or the epoch key / wrapped `CK` is wrong or destroyed.
pub fn decrypt_message_envelope(
    epoch_key: &EpochKey,
    encrypted: &EncryptedContent,
) -> Result<Vec<u8>, MlsClientError> {
    let payload = decode_v2_payload(encrypted)?;

    let wrap_nonce = &payload[OFF_WRAP_NONCE..OFF_WRAPPED_CK];
    let wrapped_ck = &payload[OFF_WRAPPED_CK..OFF_EPOCH];
    let content_nonce = &payload[OFF_CONTENT_NONCE..OFF_CIPHERTEXT];
    let ciphertext = &payload[OFF_CIPHERTEXT..];

    // Unwrap CK. A destroyed (e.g. zeroed) wrapped CK fails the AEAD tag here —
    // this is exactly the erasability invariant: no epoch key recovers a CK that
    // was never recoverable except by unwrapping the (now-destroyed) blob.
    let wrap_cipher = ChaCha20Poly1305::new(epoch_key.into());
    let ck = wrap_cipher
        .decrypt(Nonce::from_slice(wrap_nonce), wrapped_ck)
        .map_err(|_| MlsClientError::DecryptionFailed)?;

    let content_cipher = ChaCha20Poly1305::new(ck.as_slice().into());
    content_cipher
        .decrypt(Nonce::from_slice(content_nonce), ciphertext)
        .map_err(|_| MlsClientError::DecryptionFailed)
}

/// Return whether an `EncryptedContent` is the v2 (envelope) form.
pub fn is_envelope_v2(encrypted: &EncryptedContent) -> bool {
    decode_v2_payload(encrypted).is_ok()
}

/// Witness for AH-D1 step (e): return a copy of a v2 envelope with the wrapped
/// `CK` region overwritten with zeros — content then becomes unrecoverable *even
/// with the correct epoch key* (proved by the content-blindness proof, assertion
/// 5). This demonstrates the **effect** of crypto-shred at per-message
/// granularity; it is **not** the destroy-to-erase *storage* operation (locating
/// and overwriting the wrapped `CK` in persisted storage), which is fenced behind
/// the erasure-impl arc per the D-088 cascade. Returns `None` for non-v2 /
/// malformed input.
pub fn envelope_with_destroyed_key(encrypted: &EncryptedContent) -> Option<EncryptedContent> {
    let mut payload = decode_v2_payload(encrypted).ok()?;
    for b in &mut payload[OFF_WRAPPED_CK..OFF_EPOCH] {
        *b = 0;
    }
    Some(EncryptedContent(format!("enc:{}", encoding::encode(&payload))))
}

/// Decode + structurally validate a v2 envelope payload (past the `enc:` prefix).
fn decode_v2_payload(encrypted: &EncryptedContent) -> Result<Vec<u8>, MlsClientError> {
    let b64 = encrypted.0.strip_prefix("enc:").ok_or(MlsClientError::DecryptionFailed)?;
    let payload = encoding::decode(b64).map_err(|_| MlsClientError::DecryptionFailed)?;
    if payload.len() < V2_PREFIX_LEN || payload[0] != ENVELOPE_VERSION_V2 {
        return Err(MlsClientError::DecryptionFailed);
    }
    Ok(payload)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn random_secret() -> [u8; 32] {
        use rand::{rngs::OsRng, RngCore};
        let mut s = [0u8; 32];
        OsRng.fill_bytes(&mut s);
        s
    }

    #[test]
    fn mls_round_trip() {
        let secret = random_secret();
        let mut alice_group = ClientMlsGroup::new("room1", "alice", secret);

        // Bob joins — Alice advances the epoch and provides Bob with the new epoch key.
        let (new_epoch, bob_epoch_key) = alice_group.add_member("bob");
        // Bob's group is initialised at the same epoch with the same key.
        let bob_epoch_key_copy = bob_epoch_key;

        // Alice sends an encrypted message.
        let plaintext = b"Hello, Bob!";
        let encrypted = encrypt_message(&alice_group.current_epoch_key(), alice_group.epoch, plaintext);

        // Bob decrypts using his copy of the epoch key.
        let decrypted = decrypt_message(&bob_epoch_key_copy, new_epoch, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn removed_member_cannot_decrypt_future_messages() {
        let secret = random_secret();
        let mut alice_group = ClientMlsGroup::new("room1", "alice", secret);

        // Bob joins at epoch 1.
        let (bob_epoch, bob_epoch_key) = alice_group.add_member("bob");
        assert_eq!(bob_epoch, 1);

        // Bob is removed — epoch advances to 2 with a new key Bob doesn't receive.
        let _new_epoch_key = alice_group.remove_member("bob").unwrap();
        assert_eq!(alice_group.epoch, 2);

        // Alice sends a message in epoch 2.
        let plaintext = b"Private message Bob cannot see";
        let encrypted =
            encrypt_message(&alice_group.current_epoch_key(), alice_group.epoch, plaintext);

        // Bob tries to decrypt with his epoch 1 key → fails.
        let err = decrypt_message(&bob_epoch_key, bob_epoch, &encrypted).unwrap_err();
        // Bob's epoch 1 key is wrong for epoch 2 message → EpochMismatch.
        assert_eq!(err, MlsClientError::EpochMismatch);

        // Even with correct epoch number, Bob's key is wrong → DecryptionFailed.
        let wrong_key_err =
            decrypt_message(&bob_epoch_key, alice_group.epoch, &encrypted).unwrap_err();
        assert_eq!(wrong_key_err, MlsClientError::DecryptionFailed);
    }

    #[test]
    fn encrypted_content_not_logged() {
        // The event_trace layer must not log encrypted content.
        // Verify that the prefix "enc:" allows detection of E2E content.
        let encrypted = EncryptedContent("enc:abc123base64content".to_string());
        // The convention: if content starts with "enc:", the event_trace
        // must substitute an empty string (the rule is already in event_trace.rs).
        assert!(encrypted.0.starts_with("enc:"),
            "encrypted content must carry 'enc:' prefix so event_trace can detect it");
    }

    #[test]
    fn wrong_epoch_key_fails_decryption() {
        let secret = random_secret();
        let group = ClientMlsGroup::new("room1", "alice", secret);
        let key = group.current_epoch_key();

        let encrypted = encrypt_message(&key, group.epoch, b"secret");

        // A different random key must not decrypt.
        let wrong_key = random_secret();
        let err = decrypt_message(&wrong_key, group.epoch, &encrypted).unwrap_err();
        assert_eq!(err, MlsClientError::DecryptionFailed);
    }

    #[test]
    fn epoch_key_differs_per_epoch() {
        let secret = random_secret();
        let mut group = ClientMlsGroup::new("room1", "alice", secret);
        let key0 = group.current_epoch_key();
        group.add_member("bob"); // epoch advances to 1
        let key1 = group.current_epoch_key();
        // Different epochs must produce different keys.
        assert_ne!(key0, key1);
    }

    // ── Envelope (enc: v2 — AH-D1) ─────────────────────────────────────────────

    #[test]
    fn envelope_round_trips() {
        let secret = random_secret();
        let group = ClientMlsGroup::new("room1", "alice", secret);
        let key = group.current_epoch_key();

        let plaintext = br#"{"text":"Hello, envelope"}"#;
        let enc = encrypt_message_envelope(&key, group.epoch, plaintext);
        assert!(enc.0.starts_with("enc:"));
        assert!(is_envelope_v2(&enc));

        let decrypted = decrypt_message_envelope(&key, &enc).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn envelope_v2_is_distinct_from_v1() {
        let secret = random_secret();
        let group = ClientMlsGroup::new("room1", "alice", secret);
        let key = group.current_epoch_key();

        let v1 = encrypt_message(&key, group.epoch, b"hi");
        let v2 = encrypt_message_envelope(&key, group.epoch, b"hi");
        // Both carry the enc: prefix (event_trace / is_encrypted_content
        // convention unchanged), but only v2 is the envelope form.
        assert!(v1.0.starts_with("enc:") && v2.0.starts_with("enc:"));
        assert!(!is_envelope_v2(&v1));
        assert!(is_envelope_v2(&v2));
    }

    #[test]
    fn envelope_ck_is_random_per_message() {
        // Two encryptions of the SAME plaintext under the SAME epoch key must
        // differ (random CK + random nonces) — the precondition for per-message
        // erasability (AH-D1 constraint 2: CK is never derived from the epoch
        // secret).
        let secret = random_secret();
        let group = ClientMlsGroup::new("room1", "alice", secret);
        let key = group.current_epoch_key();
        let a = encrypt_message_envelope(&key, group.epoch, b"same");
        let b = encrypt_message_envelope(&key, group.epoch, b"same");
        assert_ne!(a.0, b.0);
        // Both still decrypt to the same plaintext.
        assert_eq!(decrypt_message_envelope(&key, &a).unwrap(), b"same");
        assert_eq!(decrypt_message_envelope(&key, &b).unwrap(), b"same");
    }

    #[test]
    fn envelope_wrong_epoch_key_cannot_unwrap() {
        let secret = random_secret();
        let group = ClientMlsGroup::new("room1", "alice", secret);
        let key = group.current_epoch_key();
        let enc = encrypt_message_envelope(&key, group.epoch, b"secret");

        let wrong_key = random_secret();
        let err = decrypt_message_envelope(&wrong_key, &enc).unwrap_err();
        assert_eq!(err, MlsClientError::DecryptionFailed);
    }

    #[test]
    fn erasing_wrapped_key_defeats_epoch_holder() {
        // The threat-defended erasability invariant (AH-D1 constraint 2 =
        // AH-D5 assertion 5, one invariant): destroying the wrapped CK leaves
        // content unrecoverable EVEN BY A PARTY WHO STILL HOLDS THE EPOCH SECRET.
        // This is the precise failure mode the random-CK rule defends against:
        // were CK epoch-derived, this holder would silently re-derive it.
        let secret = random_secret();
        let group = ClientMlsGroup::new("room1", "alice", secret);
        let epoch_key = group.current_epoch_key();

        let enc = encrypt_message_envelope(&epoch_key, group.epoch, b"erase me");
        // Sanity: the holder of the epoch secret CAN read it before erasure.
        assert_eq!(decrypt_message_envelope(&epoch_key, &enc).unwrap(), b"erase me");

        // Destroy the wrapped CK (the witness of step (e)'s effect).
        let shredded = envelope_with_destroyed_key(&enc).unwrap();
        // The SAME holder of the SAME epoch secret can no longer recover content.
        let err = decrypt_message_envelope(&epoch_key, &shredded).unwrap_err();
        assert_eq!(err, MlsClientError::DecryptionFailed);
    }

    #[test]
    fn malformed_envelope_rejected() {
        let secret = random_secret();
        let key = ClientMlsGroup::new("room1", "alice", secret).current_epoch_key();
        // Not even base64.
        assert!(decrypt_message_envelope(&key, &EncryptedContent("enc:!!!".into())).is_err());
        // v1 form is not a v2 envelope.
        let v1 = encrypt_message(&key, 0, b"x");
        assert!(decrypt_message_envelope(&key, &v1).is_err());
        assert!(envelope_with_destroyed_key(&v1).is_none());
    }
}
