// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// Website: https://www.alchemydump.com
// Licensed under the PolyForm Noncommercial License 1.0.0
// See: https://polyformproject.org/licenses/noncommercial/1.0.0/

// Canonical JSON form for Event signing and ID derivation (spec 3.2.4)
//
// Rules:
//   - Top-level Event envelope fields appear in fixed canonical order
//   - "event_id" and "signature" are excluded (circular — used to compute/produce them)
//   - Nested objects (content, meta_atts) have keys sorted lexicographically, recursively
//   - No whitespace outside string values
//   - Array order preserved

use serde_json::Value;

// Fixed canonical order for Event envelope fields (spec 3.2.4, rule 2)
const EVENT_FIELD_ORDER: &[&str] = &[
    "protocol_version",
    "type",
    "sender",
    "room_id",
    "space_id",
    "prev_events",
    "timestamp",
    "content",
    "meta_atts",
];

/// Produce the canonical UTF-8 bytes of an Event for signing or ID derivation.
/// Excludes "event_id" and "signature".
pub fn canonical_event_bytes(event: &Value) -> Vec<u8> {
    canonical_event_json(event).into_bytes()
}

/// Produce the canonical JSON string of an Event.
pub fn canonical_event_json(event: &Value) -> String {
    let obj = match event.as_object() {
        Some(o) => o,
        None => return canonical_value(event),
    };

    let mut parts = Vec::new();
    for &field in EVENT_FIELD_ORDER {
        if let Some(value) = obj.get(field) {
            let key_json = serde_json::to_string(field).unwrap();
            parts.push(format!("{}:{}", key_json, canonical_value(value)));
        }
    }
    format!("{{{}}}", parts.join(","))
}

// Serialize any JSON value with all object keys sorted lexicographically (recursive).
fn canonical_value(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut pairs: Vec<(&String, &Value)> = map.iter().collect();
            pairs.sort_by_key(|(k, _)| k.as_str());
            let parts: Vec<String> = pairs
                .iter()
                .map(|(k, v)| {
                    format!("{}:{}", serde_json::to_string(k).unwrap(), canonical_value(v))
                })
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(canonical_value).collect();
            format!("[{}]", parts.join(","))
        }
        _ => serde_json::to_string(value).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn field_order_is_canonical() {
        let event = json!({
            "timestamp": "2026-04-27T10:00:00.000Z",
            "type": "message.text",
            "protocol_version": "0.1",
            "sender": "xgen://pubkey/ed25519:abc",
            "room_id": "xgen://hash/sha256:def",
            "space_id": "xgen://hash/sha256:ghi",
            "prev_events": ["xgen://hash/sha256:jkl"],
            "content": {"text": "Hello"},
        });
        let canonical = canonical_event_json(&event);
        // protocol_version must come before type
        let pv_pos = canonical.find("protocol_version").unwrap();
        let type_pos = canonical.find("\"type\"").unwrap();
        assert!(pv_pos < type_pos);
        // timestamp must come before content
        let ts_pos = canonical.find("timestamp").unwrap();
        let content_pos = canonical.find("content").unwrap();
        assert!(ts_pos < content_pos);
    }

    #[test]
    fn event_id_and_signature_excluded() {
        let event = json!({
            "protocol_version": "0.1",
            "type": "message.text",
            "event_id": "xgen://hash/sha256:aaa",
            "signature": "ed25519:bbb:ccc",
            "sender": "xgen://pubkey/ed25519:ddd",
            "room_id": "xgen://hash/sha256:eee",
            "space_id": "xgen://hash/sha256:fff",
            "prev_events": [],
            "timestamp": "2026-04-27T10:00:00.000Z",
            "content": {},
        });
        let canonical = canonical_event_json(&event);
        assert!(!canonical.contains("event_id"));
        assert!(!canonical.contains("signature"));
    }

    #[test]
    fn content_keys_sorted() {
        let event = json!({
            "protocol_version": "0.1",
            "type": "message.text",
            "sender": "s",
            "room_id": "r",
            "space_id": "sp",
            "prev_events": [],
            "timestamp": "t",
            "content": {"z_key": 1, "a_key": 2, "m_key": 3},
        });
        let canonical = canonical_event_json(&event);
        let a_pos = canonical.find("a_key").unwrap();
        let m_pos = canonical.find("m_key").unwrap();
        let z_pos = canonical.find("z_key").unwrap();
        assert!(a_pos < m_pos && m_pos < z_pos);
    }

    #[test]
    fn no_whitespace() {
        let event = json!({
            "protocol_version": "0.1",
            "type": "message.text",
            "sender": "s",
            "room_id": "r",
            "space_id": "sp",
            "prev_events": [],
            "timestamp": "t",
            "content": {"text": "hello world"},
        });
        let canonical = canonical_event_json(&event);
        // No spaces or newlines outside string values
        // We check that the JSON structure itself has no spaces between tokens
        // by verifying no space appears adjacent to : or ,
        assert!(!canonical.contains(": "));
        assert!(!canonical.contains(", "));
        assert!(!canonical.contains('\n'));
    }

    #[test]
    fn deterministic() {
        let event = json!({
            "protocol_version": "0.1",
            "type": "message.text",
            "sender": "s",
            "room_id": "r",
            "space_id": "sp",
            "prev_events": [],
            "timestamp": "t",
            "content": {"text": "hello"},
        });
        assert_eq!(canonical_event_json(&event), canonical_event_json(&event));
    }

    #[test]
    fn array_order_preserved() {
        let event = json!({
            "protocol_version": "0.1",
            "type": "message.text",
            "sender": "s",
            "room_id": "r",
            "space_id": "sp",
            "prev_events": ["xgen://hash/sha256:zzz", "xgen://hash/sha256:aaa"],
            "timestamp": "t",
            "content": {},
        });
        let canonical = canonical_event_json(&event);
        let zzz_pos = canonical.find("zzz").unwrap();
        let aaa_pos = canonical.find("aaa").unwrap();
        // zzz came first in the array — must remain first
        assert!(zzz_pos < aaa_pos);
    }
}
