// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

//! Per-blob symmetric encryption (M12.1, M12-D5 / V3).
//!
//! Each attachment blob is encrypted client-side under a **fresh per-blob key**
//! before upload; the node stores + content-addresses the **ciphertext** only and
//! stays content-blind (M12-D4). The per-blob key rides the `Descriptor`
//! (`message::exchange::Descriptor`); the node content-addresses by
//! `hash_uri(ciphertext)`.
//!
//! **Crypto maturity (R-1, the M12-D5 flag-1 boundary).** This is the
//! per-blob-key *byte* encryption — real M12.1 work (the ciphertext-at-rest W2
//! witness). The per-blob key itself travels in the `Descriptor` as **plaintext**
//! content today, exactly as `message.text`'s body is plaintext today
//! (client-side `enc:` live-encryption is D3-fenced, Arc-H C1 Finding 1). So
//! attachments are **not stronger, not weaker, than text** (M12-D5): the node can
//! read the key today exactly as it reads text; both go node-blind together when
//! the shared text-path `enc:` wrap lands at the D3/M8.7 cutover. Building per-blob
//! encryption now is *not* premature — it is the content-address-by-ciphertext-
//! hash + clean-cutover structure (no blob-store rework at D3).
//!
//! Primitive: ChaCha20Poly1305 — the Arc-H Phase-2 primitive (D-052), the same as
//! `client_mls`. Real RFC 9420 HPKE is the D3 swap, not a reshape.
//!
//! Ciphertext layout: `nonce(12) ‖ ChaCha20Poly1305(plaintext)` (tag appended by
//! the AEAD). The nonce is random per blob; the key is fresh per blob, so a fixed
//! 12-byte nonce per (fresh) key is sound, but we still randomise it to match the
//! `client_mls` style.

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use thiserror::Error;

/// Per-blob symmetric key length (ChaCha20Poly1305 key = 32 bytes).
pub const BLOB_KEY_LEN: usize = 32;
const BLOB_NONCE_LEN: usize = 12;

/// Failure decrypting a blob (client-side, after fetch). Distinct from
/// `blob_store::BlobError` (the store/transfer-ingest reject type): this is the
/// AEAD layer.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum BlobCryptoError {
    #[error("blob decryption failed: wrong key or tampered ciphertext")]
    DecryptionFailed,
    #[error("blob ciphertext is malformed (shorter than the nonce prefix)")]
    Malformed,
}

/// Encrypt `plaintext` under a **fresh** per-blob key (M12-D5). Returns
/// `(key, ciphertext)` where `ciphertext = nonce(12) ‖ AEAD(plaintext)`. The
/// caller content-addresses by `hash_uri(ciphertext)` (the `blob_ref`) and carries
/// `key` (base64) in the `Descriptor`.
pub fn encrypt_blob(plaintext: &[u8]) -> ([u8; BLOB_KEY_LEN], Vec<u8>) {
    let mut key = [0u8; BLOB_KEY_LEN];
    OsRng.fill_bytes(&mut key);
    let mut nonce = [0u8; BLOB_NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);

    let cipher = ChaCha20Poly1305::new((&key).into());
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .expect("ChaCha20 encrypt is infallible");

    let mut out = Vec::with_capacity(BLOB_NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    (key, out)
}

/// Decrypt a blob `ciphertext` (`nonce(12) ‖ AEAD(plaintext)`) under `key`.
/// `DecryptionFailed` on a wrong key or tampered bytes; `Malformed` if shorter
/// than the nonce prefix.
pub fn decrypt_blob(key: &[u8; BLOB_KEY_LEN], ciphertext: &[u8]) -> Result<Vec<u8>, BlobCryptoError> {
    if ciphertext.len() < BLOB_NONCE_LEN {
        return Err(BlobCryptoError::Malformed);
    }
    let (nonce, ct) = ciphertext.split_at(BLOB_NONCE_LEN);
    let cipher = ChaCha20Poly1305::new(key.into());
    cipher
        .decrypt(Nonce::from_slice(nonce), ct)
        .map_err(|_| BlobCryptoError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_then_decrypt_roundtrip() {
        let plaintext = b"the quick brown fox jumps over the lazy dog";
        let (key, ct) = encrypt_blob(plaintext);
        assert_eq!(decrypt_blob(&key, &ct).unwrap(), plaintext.to_vec());
    }

    #[test]
    fn ciphertext_is_not_plaintext() {
        // Content-blind sanity: the encrypted bytes do not contain the plaintext.
        let plaintext = b"SECRET-MARKER-12345";
        let (_key, ct) = encrypt_blob(plaintext);
        assert!(!ct.windows(plaintext.len()).any(|w| w == plaintext));
        assert!(ct.len() > BLOB_NONCE_LEN); // nonce prefix + AEAD output
    }

    #[test]
    fn wrong_key_fails() {
        let (_key, ct) = encrypt_blob(b"payload");
        let wrong = [7u8; BLOB_KEY_LEN];
        assert_eq!(decrypt_blob(&wrong, &ct), Err(BlobCryptoError::DecryptionFailed));
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let (key, mut ct) = encrypt_blob(b"payload");
        let last = ct.len() - 1;
        ct[last] ^= 0xff; // flip a ciphertext/tag byte
        assert_eq!(decrypt_blob(&key, &ct), Err(BlobCryptoError::DecryptionFailed));
    }

    #[test]
    fn malformed_too_short_fails() {
        let key = [0u8; BLOB_KEY_LEN];
        assert_eq!(decrypt_blob(&key, &[0u8; 5]), Err(BlobCryptoError::Malformed));
    }

    #[test]
    fn fresh_key_and_ciphertext_each_call() {
        let plaintext = b"same-input";
        let (k1, c1) = encrypt_blob(plaintext);
        let (k2, c2) = encrypt_blob(plaintext);
        assert_ne!(k1, k2); // fresh per-blob key
        assert_ne!(c1, c2); // fresh nonce + key => distinct ciphertext
    }
}
