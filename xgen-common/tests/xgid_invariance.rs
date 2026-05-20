// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! XGID wire-format invariance test suite (XGID Adoption v1, runbook §"Test
//! suite" — five required tests). Tests that the invariance promises in
//! `docs/xgen_appendix_j_en.md` §J.5 hold end-to-end through serde.
//!
//! The five required test names below are pinned by the runbook and must
//! exist verbatim. Additional tests live in the in-module test block in
//! `xgen-common/src/xgid/flavours.rs`.
//!
//! The `event_xgid_roundtrip_through_event_canonical_form` and
//! `node_xgid_roundtrip_through_handshake_message` tests use faithful
//! test stand-ins for the Event canonical form and the federation handshake
//! message — see comments in each test. The runbook's "real Phase 7.5
//! federation message structure or a faithful test stand-in" hedge applies;
//! the canonical-form helper lives in `xgen-core` and is not visible from
//! this dev-target.

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use xgen_common::{EventXgid, NodeXgid, Xgid};

fn test_signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

#[test]
fn xgid_serializes_as_plain_string() {
    // The base Xgid and a representative flavour (NodeXgid) must each
    // serialise to a plain JSON string — no object wrapping, no flavour
    // tag, no extra fields (Appendix J §J.5 invariance 2).
    let base = Xgid::new("xgen://hash/sha256:abc".to_string());
    let base_json = serde_json::to_string(&base).expect("serialise base Xgid");
    assert_eq!(base_json, "\"xgen://hash/sha256:abc\"");

    let node = NodeXgid::from_pubkey(&test_signing_key(0x01).verifying_key());
    let node_json = serde_json::to_string(&node).expect("serialise NodeXgid");
    assert!(node_json.starts_with('"'));
    assert!(node_json.ends_with('"'));
    assert!(node_json.contains("xgen://pubkey/ed25519:"));
    // No flavour tag, no object wrapping.
    assert!(!node_json.contains('{'));
    assert!(!node_json.contains("NodeXgid"));
    assert!(!node_json.contains("flavour"));
}

#[test]
fn xgid_deserializes_from_plain_string() {
    // The base Xgid and every flavour wrapper must deserialise from a plain
    // JSON string with the inner bytes preserved byte-equal (Appendix J §J.5
    // invariance 5 — no leading/trailing whitespace tolerance, no quote-mark
    // normalisation).
    let base: Xgid =
        serde_json::from_str("\"xgen://hash/sha256:abc\"").expect("deserialise base Xgid");
    assert_eq!(base.as_str(), "xgen://hash/sha256:abc");

    let node: NodeXgid = serde_json::from_str(
        "\"xgen://pubkey/ed25519:VGhpcyBpcyBhIGZha2UgcHVia2V5IGZvciB0ZXN0aW5n\"",
    )
    .expect("deserialise NodeXgid");
    assert_eq!(
        node.as_str(),
        "xgen://pubkey/ed25519:VGhpcyBpcyBhIGZha2UgcHVia2V5IGZvciB0ZXN0aW5n"
    );

    let event: EventXgid =
        serde_json::from_str("\"xgen://hash/sha256:def\"").expect("deserialise EventXgid");
    assert_eq!(event.as_str(), "xgen://hash/sha256:def");
}

#[test]
fn flavour_wrapper_is_serde_transparent() {
    // Serialising a NodeXgid and a String containing the same URI must
    // produce byte-equal JSON (Appendix J §J.5 invariance 2 — the flavour
    // wrapper carries zero wire overhead vs the raw String it replaces).
    // Repeat for an EventXgid (hash-anchored flavour).
    let pk = test_signing_key(0x02).verifying_key();
    let node = NodeXgid::from_pubkey(&pk);
    let raw_string = node.as_str().to_string();
    let node_json = serde_json::to_string(&node).expect("serialise NodeXgid");
    let string_json = serde_json::to_string(&raw_string).expect("serialise raw String");
    assert_eq!(node_json, string_json);

    let event = EventXgid::from_canonical_bytes(b"sample canonical event bytes");
    let raw_event = event.as_str().to_string();
    let event_json = serde_json::to_string(&event).expect("serialise EventXgid");
    let event_string_json =
        serde_json::to_string(&raw_event).expect("serialise raw String (event)");
    assert_eq!(event_json, event_string_json);
}

#[test]
fn event_xgid_roundtrip_through_event_canonical_form() {
    // End-to-end invariance: construct an EventXgid from a representative
    // canonical-form byte sequence, embed it in a struct that mimics an
    // Event payload carrying an EventXgid field, serialise, deserialise,
    // recompute the EventXgid from the same canonical bytes, and assert
    // every value is byte-equal (Appendix J §J.5 invariance 3).
    //
    // Uses a fixed "canonical bytes" stand-in rather than the real
    // canonical_event_bytes helper — that helper lives in xgen-core and is
    // not visible from xgen-common's dev-target. The runbook's "faithful
    // test stand-in" hedge applies; the invariance under test is round-
    // trip-through-serde, which holds regardless of how the canonical bytes
    // were computed. Retrofit Pass 1 will add a higher-level
    // `EventXgid::from_event(&Event)` constructor and a parallel test using
    // the real helper.

    // Fixed test canonical bytes — a deterministic stand-in for whatever
    // xgen-core::wire::canonical::canonical_event_bytes would produce for a
    // representative message.text Event.
    let canonical_bytes = br#"{"protocol_version":"0.1","type":"message.text","sender":"xgen://pubkey/ed25519:test","room_id":"xgen://hash/sha256:room","space_id":"xgen://hash/sha256:space","prev_events":[],"timestamp":"2026-05-20T12:00:00.000Z","content":{"text":"Hello"}}"#;

    let original = EventXgid::from_canonical_bytes(canonical_bytes);

    // A minimal envelope that mimics how production code carries an EventXgid
    // field — equivalent to Event { event_id: Option<EventXgid>, ... }.
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct EventEnvelope {
        protocol_version: String,
        event_id: EventXgid,
    }

    let envelope = EventEnvelope {
        protocol_version: "0.1".to_string(),
        event_id: original.clone(),
    };

    let json = serde_json::to_string(&envelope).expect("serialise envelope");
    let recovered: EventEnvelope = serde_json::from_str(&json).expect("deserialise envelope");
    assert_eq!(recovered, envelope);

    // Recompute from the same canonical bytes — must equal the deserialised
    // value byte-for-byte.
    let recomputed = EventXgid::from_canonical_bytes(canonical_bytes);
    assert_eq!(recovered.event_id, recomputed);
    assert_eq!(recovered.event_id, original);

    // And the inner URI on the wire is exactly the hash-anchored form.
    assert!(recovered.event_id.as_str().starts_with("xgen://hash/sha256:"));
}

#[test]
fn node_xgid_roundtrip_through_handshake_message() {
    // End-to-end invariance: construct a NodeXgid from a representative
    // Ed25519 verifying key, embed it in a struct that mimics a Phase 7.5
    // federation handshake message carrying a NodeXgid field, serialise,
    // deserialise, decode back to the original VerifyingKey bytes, assert
    // equality at every step (Appendix J §J.5 invariance 3 + invariance 4
    // — the wire-format string for principal flavours is itself the
    // canonical encoding of the pubkey).
    //
    // Uses a faithful test stand-in for the federation handshake message
    // shape — the real type lives in xgen-core::federation::handshake which
    // is not visible from xgen-common's dev-target. The runbook's stand-in
    // hedge applies; the invariance under test is serde round-trip through
    // the NodeXgid type, which holds regardless of which wire envelope
    // carries the value.

    let pk = test_signing_key(0x03).verifying_key();
    let original = NodeXgid::from_pubkey(&pk);

    // Faithful stand-in for the federation handshake message — equivalent to
    // FederationHandshake { introducer_node_id: NodeXgid, ... } in shape.
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct HandshakeStandIn {
        protocol_version: String,
        introducer_node_id: NodeXgid,
    }

    let message = HandshakeStandIn {
        protocol_version: "0.1".to_string(),
        introducer_node_id: original.clone(),
    };

    let json = serde_json::to_string(&message).expect("serialise handshake");
    let recovered: HandshakeStandIn = serde_json::from_str(&json).expect("deserialise handshake");
    assert_eq!(recovered, message);
    assert_eq!(recovered.introducer_node_id, original);

    // Decode the recovered NodeXgid back to its pubkey bytes — must equal
    // the original verifying key bytes (invariance 4: principal-flavour
    // XGID inner bytes ARE the encoded pubkey).
    let recovered_pk = recovered
        .introducer_node_id
        .pubkey()
        .expect("decode NodeXgid back to VerifyingKey");
    assert_eq!(recovered_pk.as_bytes(), pk.as_bytes());

    // And the wire string is exactly the principal-flavour form.
    assert!(
        recovered
            .introducer_node_id
            .as_str()
            .starts_with("xgen://pubkey/ed25519:")
    );
}
