// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

// base64url encoding/decoding — RFC 4648 §5, no padding (spec 3.1.9)

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

#[derive(Debug, thiserror::Error)]
pub enum EncodingError {
    #[error("standard base64 characters (+, /, =) are not permitted; use base64url")]
    StandardBase64Chars,
    #[error("invalid base64url: {0}")]
    Decode(#[from] base64::DecodeError),
}

pub fn encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn decode(s: &str) -> Result<Vec<u8>, EncodingError> {
    if s.contains('+') || s.contains('/') || s.contains('=') {
        return Err(EncodingError::StandardBase64Chars);
    }
    Ok(URL_SAFE_NO_PAD.decode(s)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let original = b"hello xgen protocol";
        assert_eq!(decode(&encode(original)).unwrap(), original);
    }

    #[test]
    fn no_padding() {
        assert!(!encode(b"any bytes").contains('='));
    }

    #[test]
    fn url_safe_alphabet() {
        let encoded = encode(&[0xfb, 0xff, 0xfe]);
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
    }

    #[test]
    fn reject_plus() {
        assert!(matches!(decode("abc+def"), Err(EncodingError::StandardBase64Chars)));
    }

    #[test]
    fn reject_slash() {
        assert!(matches!(decode("abc/def"), Err(EncodingError::StandardBase64Chars)));
    }

    #[test]
    fn reject_padding_char() {
        assert!(matches!(decode("aGVsbG8="), Err(EncodingError::StandardBase64Chars)));
    }
}
