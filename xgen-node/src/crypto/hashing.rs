// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// Website: https://www.alchemydump.com
// Licensed under the PolyForm Noncommercial License 1.0.0
// See: https://polyformproject.org/licenses/noncommercial/1.0.0/

// SHA-256 hashing and hash URI derivation (spec 3.1.6, 3.2.3)

use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hash_uri(bytes: &[u8]) -> String {
    format!("xgen://hash/sha256:{}", sha256_hex(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_sha256() {
        // SHA-256 of empty string is well-known
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn hash_uri_format() {
        let uri = hash_uri(b"");
        assert!(uri.starts_with("xgen://hash/sha256:"));
        assert_eq!(uri.len(), "xgen://hash/sha256:".len() + 64);
    }

    #[test]
    fn hash_uri_lowercase_hex() {
        let uri = hash_uri(b"test");
        let hex_part = uri.strip_prefix("xgen://hash/sha256:").unwrap();
        assert!(hex_part.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    }

    #[test]
    fn deterministic() {
        assert_eq!(hash_uri(b"hello"), hash_uri(b"hello"));
        assert_ne!(hash_uri(b"hello"), hash_uri(b"world"));
    }
}
