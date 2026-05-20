// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! XGID flavour wrappers (XGID Adoption v1, D-072, Appendix J §J.2).
//!
//! Six flavour wrappers express the protocol-object kind in the type system
//! without adding any flavour tag to the wire. Each is `#[serde(transparent)]`
//! through the inner [`Xgid`], so a `NodeXgid` serialises byte-equal to a
//! `String` containing the same URI (Appendix J §J.5 invariance 2).
//!
//! Cross-flavour conversion is **not** provided (Appendix J §J.6). Code that
//! needs to construct, say, an [`EventXgid`] from a [`NodeXgid`]'s underlying
//! string must do so explicitly through [`Xgid`] extraction — intentional
//! friction so that flavour-violating constructions show up in code review.
//!
//! ## v1 scope discipline — high-level convenience constructors deferred
//!
//! The XGID Adoption v1 runbook prescribes higher-level convenience
//! constructors for hash-anchored flavours "where it is clean to do so":
//! `EventXgid::from_event(event: &Event)`, `SpaceXgid::from_space_create(event)`,
//! `RoomXgid::from_room_create(event)`, `TrustAssertionXgid::from_assertion(a)`.
//!
//! These are **not shipped at v1**. The canonical-form computation
//! (`canonical_event_bytes`) currently lives in `xgen-core/src/wire/canonical.rs`
//! and is not visible to `xgen-common`. Providing the convenience constructors
//! at v1 would require either:
//!
//! 1. Moving `canonical.rs` from `xgen-core` to `xgen-common` — would touch
//!    production code outside `xgen-common`, violating Commit 1's DoD.
//! 2. Duplicating canonical-form logic in `xgen-common` — exactly the drift
//!    surface XGID Adoption v1 exists to close.
//!
//! Neither is "clean to do so" at v1. The runbook's "where it is clean to do
//! so" hedge applies; these constructors are deferred. Retrofit Pass 1
//! (`xgen-common` code retype) is the right place to land them — when the
//! canonical-form code moves to `xgen-common` as part of the Pass 1
//! coordinated commit, the convenience constructors land alongside.
//!
//! At v1, hash-anchored flavours ship [`EventXgid::from_canonical_bytes`] and
//! siblings — callers compute the canonical bytes externally and pass them
//! in. Principal flavours ship [`NodeXgid::from_pubkey`] and siblings
//! because their construction-source data (`ed25519_dalek::VerifyingKey`) is
//! directly available with the freshly added `ed25519-dalek` dependency.

use std::ops::Deref;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{VerifyingKey, PUBLIC_KEY_LENGTH};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{Xgid, XgidDecodeError, XgidLike};

/// URI prefix shared by both principal flavours (`NodeXgid`, `IdentityXgid`).
/// Matches the format produced by `xgen-core::crypto::encoding::encode` over
/// an Ed25519 `VerifyingKey::as_bytes()` — Retrofit Pass 1 will unify the
/// emit sites onto the XGID constructors.
const PRINCIPAL_FLAVOUR_PREFIX: &str = "xgen://pubkey/ed25519:";

/// URI prefix for hash-anchored flavours (`EventXgid`, `SpaceXgid`,
/// `RoomXgid`, `TrustAssertionXgid`). Matches
/// `xgen-core::crypto::hashing::hash_uri`.
const HASH_FLAVOUR_PREFIX: &str = "xgen://hash/sha256:";

/// Format the hash-anchored URI for a SHA-256 digest of `bytes`.
fn hash_anchored_uri(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(HASH_FLAVOUR_PREFIX.len() + 64);
    s.push_str(HASH_FLAVOUR_PREFIX);
    for b in digest.iter() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Format the principal-flavour URI for a `VerifyingKey`.
fn principal_uri(pk: &VerifyingKey) -> String {
    let encoded = URL_SAFE_NO_PAD.encode(pk.as_bytes());
    let mut s = String::with_capacity(PRINCIPAL_FLAVOUR_PREFIX.len() + encoded.len());
    s.push_str(PRINCIPAL_FLAVOUR_PREFIX);
    s.push_str(&encoded);
    s
}

/// Decode a principal-flavour XGID inner string back to a `VerifyingKey`.
/// Used by both [`NodeXgid::pubkey`] and [`IdentityXgid::pubkey`] — same
/// wire shape, same decode path.
fn principal_decode(inner: &str) -> Result<VerifyingKey, XgidDecodeError> {
    let segment = inner
        .strip_prefix(PRINCIPAL_FLAVOUR_PREFIX)
        .ok_or(XgidDecodeError::InvalidPrefix {
            expected: PRINCIPAL_FLAVOUR_PREFIX,
        })?;

    // Reject standard-base64 characters that would otherwise decode silently —
    // matches xgen-core::crypto::encoding::decode rejection rules so an XGID
    // that round-trips through principal_decode would also round-trip through
    // the legacy decode path.
    if segment.contains('+') || segment.contains('/') || segment.contains('=') {
        return Err(XgidDecodeError::InvalidBase64(
            "standard base64 characters (+, /, =) are not permitted; use base64url".to_string(),
        ));
    }

    let bytes = URL_SAFE_NO_PAD
        .decode(segment)
        .map_err(|e| XgidDecodeError::InvalidBase64(e.to_string()))?;
    if bytes.len() != PUBLIC_KEY_LENGTH {
        return Err(XgidDecodeError::InvalidKeyLength {
            expected: PUBLIC_KEY_LENGTH,
            got: bytes.len(),
        });
    }
    let mut arr = [0u8; PUBLIC_KEY_LENGTH];
    arr.copy_from_slice(&bytes);
    VerifyingKey::from_bytes(&arr).map_err(|e| XgidDecodeError::InvalidPoint(e.to_string()))
}

// ── macro: declare a flavour wrapper with its Deref + XgidLike + ─────────────
//          inherent constructors-from-Xgid impls. The flavour-specific
//          construction methods (from_canonical_bytes, from_pubkey, pubkey)
//          are added below per-flavour because their signatures differ.
macro_rules! declare_flavour {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(Xgid);

        impl $name {
            /// Wrap an existing [`Xgid`] in this flavour without validation.
            /// Used at parse boundaries where the underlying URI is known by
            /// surrounding context (e.g. on-disk persistence, deserialised
            /// wire payloads) to carry this flavour.
            pub fn from_xgid(xgid: Xgid) -> Self {
                Self(xgid)
            }

            /// Borrow the inner [`Xgid`]. Equivalent to dereferencing.
            pub fn as_xgid(&self) -> &Xgid {
                &self.0
            }

            /// Consume the wrapper and return the underlying [`Xgid`].
            pub fn into_xgid(self) -> Xgid {
                self.0
            }
        }

        impl Deref for $name {
            type Target = Xgid;
            fn deref(&self) -> &Xgid {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl XgidLike for $name {
            fn as_xgid(&self) -> &Xgid {
                &self.0
            }
        }
    };
}

declare_flavour!(
    EventXgid,
    "Hash-anchored XGID identifying an [`crate::wire::Event`] by its canonical-form SHA-256 (Appendix J §J.2)."
);
declare_flavour!(
    SpaceXgid,
    "Hash-anchored XGID identifying a Space by its `state.space_create` event's canonical-form SHA-256 (Appendix J §J.2)."
);
declare_flavour!(
    RoomXgid,
    "Hash-anchored XGID identifying a Room by its `state.room_create` event's canonical-form SHA-256 (Appendix J §J.2)."
);
declare_flavour!(
    TrustAssertionXgid,
    "Hash-anchored XGID identifying a Trust Assertion by its canonical-form SHA-256 (Appendix J §J.2)."
);
declare_flavour!(
    NodeXgid,
    "Principal-flavour XGID identifying a Node by its Ed25519 verifying key (Appendix J §J.2)."
);
declare_flavour!(
    IdentityXgid,
    "Principal-flavour XGID identifying an Identity by its Ed25519 verifying key (Appendix J §J.2)."
);

// ── Hash-anchored flavour constructors ───────────────────────────────────────

impl EventXgid {
    /// Hash the supplied canonical bytes (caller-computed) and wrap the
    /// resulting `xgen://hash/sha256:<hex>` URI as a typed `EventXgid`.
    ///
    /// At v1 callers compute the canonical bytes externally — see the
    /// module-level note on deferred convenience constructors. The expected
    /// input is the byte form produced by `xgen-core::wire::canonical::
    /// canonical_event_bytes` (Retrofit Pass 1 may relocate that helper into
    /// `xgen-common` and add the higher-level `from_event` convenience here).
    pub fn from_canonical_bytes(bytes: &[u8]) -> Self {
        Self(Xgid::new(hash_anchored_uri(bytes)))
    }
}

impl SpaceXgid {
    /// Hash the supplied canonical bytes for a `state.space_create` event
    /// and wrap the resulting URI as a typed `SpaceXgid`.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Self {
        Self(Xgid::new(hash_anchored_uri(bytes)))
    }
}

impl RoomXgid {
    /// Hash the supplied canonical bytes for a `state.room_create` event
    /// and wrap the resulting URI as a typed `RoomXgid`.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Self {
        Self(Xgid::new(hash_anchored_uri(bytes)))
    }
}

impl TrustAssertionXgid {
    /// Hash the supplied canonical bytes for a Trust Assertion and wrap the
    /// resulting URI as a typed `TrustAssertionXgid`.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Self {
        Self(Xgid::new(hash_anchored_uri(bytes)))
    }
}

// ── Principal flavour constructors and decode methods ────────────────────────

impl NodeXgid {
    /// Format the supplied Ed25519 verifying key as a typed `NodeXgid`.
    /// Infallible — matches the URI shape produced today by
    /// `xgen-node::app::pubkey_uri` and `xgen-node::fanout::tests::pubkey_uri`,
    /// which Retrofit Pass 3 (`xgen-node` retype) will unify onto this
    /// constructor.
    pub fn from_pubkey(pk: &VerifyingKey) -> Self {
        Self(Xgid::new(principal_uri(pk)))
    }

    /// Decode the inner URI string back to a `VerifyingKey`.
    /// **Parse-fallible at v1** — the base [`Xgid`] accepts any string, so
    /// principal flavours cannot promise more than what the construction-
    /// source data supports. A future walkthrough may tighten this to
    /// infallible if parse-on-construction is adopted; not in v1 scope
    /// (Appendix J §J.8).
    pub fn pubkey(&self) -> Result<VerifyingKey, XgidDecodeError> {
        principal_decode(self.0.as_str())
    }
}

impl IdentityXgid {
    /// Format the supplied Ed25519 verifying key as a typed `IdentityXgid`.
    /// Infallible — same wire shape as `NodeXgid::from_pubkey` (Appendix J
    /// §J.2 — both principal flavours share the URI form).
    pub fn from_pubkey(pk: &VerifyingKey) -> Self {
        Self(Xgid::new(principal_uri(pk)))
    }

    /// Decode the inner URI string back to a `VerifyingKey`. Parse-fallible
    /// at v1 — see [`NodeXgid::pubkey`] for rationale.
    pub fn pubkey(&self) -> Result<VerifyingKey, XgidDecodeError> {
        principal_decode(self.0.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    /// Deterministic test signing key — avoids a dev-only `rand` dependency
    /// for what are pure round-trip tests. Any 32-byte seed produces a valid
    /// `SigningKey`; the `seed` parameter lets each test pin its own.
    fn test_signing_key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    #[test]
    fn hash_anchored_uri_matches_legacy_format() {
        // The XGID hash-anchored URI must be byte-equal to what
        // xgen-core::crypto::hashing::hash_uri produces. Lock the legacy
        // format here so Retrofit Pass 1 can prove the call-site migration
        // is wire-compatible.
        let uri = hash_anchored_uri(b"hello");
        assert_eq!(
            uri,
            "xgen://hash/sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn principal_uri_matches_legacy_format() {
        // Round-trip: encode a known pubkey, decode it back, confirm equality.
        // The wire shape "xgen://pubkey/ed25519:<base64url-no-pad>" must
        // match xgen-node::app::pubkey_uri byte-for-byte.
        let sk = test_signing_key(0x42);
        let pk = sk.verifying_key();
        let uri = principal_uri(&pk);
        assert!(uri.starts_with("xgen://pubkey/ed25519:"));
        let decoded = principal_decode(&uri).expect("round-trip");
        assert_eq!(decoded.as_bytes(), pk.as_bytes());
    }

    #[test]
    fn principal_decode_rejects_wrong_prefix() {
        let err = principal_decode("xgen://hash/sha256:abc").unwrap_err();
        assert!(matches!(err, XgidDecodeError::InvalidPrefix { .. }));
    }

    #[test]
    fn principal_decode_rejects_standard_base64() {
        // Standard base64 characters (+ / =) must be rejected to match
        // xgen-core::crypto::encoding::decode rejection rules.
        let err = principal_decode("xgen://pubkey/ed25519:abc+def").unwrap_err();
        assert!(matches!(err, XgidDecodeError::InvalidBase64(_)));
    }

    #[test]
    fn principal_decode_rejects_wrong_length() {
        // "QQ" decodes to 1 byte (0x41); too short to be a 32-byte Ed25519 key.
        let err = principal_decode("xgen://pubkey/ed25519:QQ").unwrap_err();
        assert!(matches!(err, XgidDecodeError::InvalidKeyLength { .. }));
    }

    #[test]
    fn node_xgid_from_pubkey_roundtrip() {
        let sk = test_signing_key(0x42);
        let pk = sk.verifying_key();
        let xgid = NodeXgid::from_pubkey(&pk);
        let recovered = xgid.pubkey().expect("decode");
        assert_eq!(recovered.as_bytes(), pk.as_bytes());
    }

    #[test]
    fn identity_xgid_from_pubkey_roundtrip() {
        let sk = test_signing_key(0x42);
        let pk = sk.verifying_key();
        let xgid = IdentityXgid::from_pubkey(&pk);
        let recovered = xgid.pubkey().expect("decode");
        assert_eq!(recovered.as_bytes(), pk.as_bytes());
    }

    #[test]
    fn flavour_wrappers_deref_to_xgid() {
        let xgid = Xgid::new("xgen://hash/sha256:abc".to_string());
        let event_xgid = EventXgid::from_xgid(xgid.clone());
        // Deref chain: &EventXgid -> &Xgid -> &str
        assert_eq!(event_xgid.as_str(), "xgen://hash/sha256:abc");
        assert_eq!(&*event_xgid, &xgid);
    }

    #[test]
    fn into_xgid_consumes_wrapper() {
        let xgid = Xgid::new("xgen://hash/sha256:abc".to_string());
        let event_xgid = EventXgid::from_xgid(xgid.clone());
        let recovered = event_xgid.into_xgid();
        assert_eq!(recovered, xgid);
    }

    #[test]
    fn xgid_like_trait_unifies_access() {
        // XgidLike is the right tool for code that operates over "any XGID"
        // without caring about flavour — trace logging is the canonical use.
        fn log_xgid<X: XgidLike>(x: &X) -> String {
            x.as_str().to_string()
        }
        let base = Xgid::new("xgen://hash/sha256:abc".to_string());
        let event = EventXgid::from_xgid(base.clone());
        let sk = test_signing_key(0x42);
        let node = NodeXgid::from_pubkey(&sk.verifying_key());

        assert_eq!(log_xgid(&base), "xgen://hash/sha256:abc");
        assert_eq!(log_xgid(&event), "xgen://hash/sha256:abc");
        assert_eq!(log_xgid(&node).len(), "xgen://pubkey/ed25519:".len() + 43);
    }
}
