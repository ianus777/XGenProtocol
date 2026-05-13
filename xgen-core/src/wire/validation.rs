// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Event validation pipeline — structural steps 1-7 (spec 3.2.6).
//
// Steps 8-13 (cryptographic and state checks) are deferred to later layers.
//
// Pipeline:
//   1. Size check       — payload ≤ 256 KB (Local Node mode ceiling)
//   2. JSON parse       — must be a valid JSON object
//   3. Required fields  — all envelope fields present
//   4. Field types      — correct JSON types for each field
//   5. Protocol version — must be "0.1"
//   6. Event type       — must be a known EventType string
//   7. Timestamp        — must be RFC 3339 / ISO 8601 with timezone

use chrono::DateTime;
use serde_json::Value;
use thiserror::Error;

use super::framing::MAX_PAYLOAD_BYTES;
use super::types::{Event, EventType};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("step 1: payload {0} bytes exceeds limit of {}", MAX_PAYLOAD_BYTES)]
    TooLarge(usize),

    #[error("step 2: invalid JSON — {0}")]
    InvalidJson(String),

    #[error("step 3: missing required field '{0}'")]
    MissingField(&'static str),

    #[error("step 4: wrong type for field '{0}': expected {1}")]
    WrongFieldType(&'static str, &'static str),

    #[error("step 5: unsupported protocol_version '{0}' (expected '0.1')")]
    UnsupportedVersion(String),

    #[error("step 6: unknown event type '{0}'")]
    UnknownEventType(String),

    #[error("step 7: invalid timestamp '{0}' — must be RFC 3339")]
    InvalidTimestamp(String),
}

/// Run the 7 structural validation steps on raw event bytes.
/// Returns a deserialised `Event` on success; all fields are present and typed correctly.
pub fn validate_event_bytes(bytes: &[u8]) -> Result<Event, ValidationError> {
    // Step 1: size
    if bytes.len() > MAX_PAYLOAD_BYTES {
        return Err(ValidationError::TooLarge(bytes.len()));
    }

    // Step 2: JSON parse
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|e| ValidationError::InvalidJson(e.to_string()))?;

    validate_event_value(&value)
}

/// Run the 7 structural validation steps on an already-parsed JSON Value.
pub fn validate_event_value(value: &Value) -> Result<Event, ValidationError> {
    let obj = match value.as_object() {
        Some(o) => o,
        None => return Err(ValidationError::InvalidJson("not a JSON object".to_string())),
    };

    // Step 3: required fields present
    const REQUIRED: &[&str] = &[
        "protocol_version",
        "type",
        "event_id",
        "sender",
        "room_id",
        "space_id",
        "prev_events",
        "timestamp",
        "content",
        "signature",
    ];
    for &field in REQUIRED {
        if !obj.contains_key(field) {
            return Err(ValidationError::MissingField(field));
        }
    }

    // Step 4: field types
    check_string(obj, "protocol_version")?;
    check_string(obj, "type")?;
    check_string(obj, "event_id")?;
    check_string(obj, "sender")?;
    check_string(obj, "room_id")?;
    check_string(obj, "space_id")?;
    check_array(obj, "prev_events")?;
    check_string(obj, "timestamp")?;
    check_object(obj, "content")?;
    check_string(obj, "signature")?;

    // Step 5: protocol version
    let version = obj["protocol_version"].as_str().unwrap();
    if version != "0.1" {
        return Err(ValidationError::UnsupportedVersion(version.to_string()));
    }

    // Step 6: known event type
    let type_str = obj["type"].as_str().unwrap();
    if EventType::from_str(type_str).is_none() {
        return Err(ValidationError::UnknownEventType(type_str.to_string()));
    }

    // Step 7: timestamp parses as RFC 3339
    let ts = obj["timestamp"].as_str().unwrap();
    DateTime::parse_from_rfc3339(ts)
        .map_err(|_| ValidationError::InvalidTimestamp(ts.to_string()))?;

    // All structural checks passed — deserialise into Event
    // This cannot fail given the checks above established the schema.
    let ev: Event = serde_json::from_value(value.clone())
        .map_err(|e| ValidationError::InvalidJson(e.to_string()))?;
    Ok(ev)
}

fn check_string(
    obj: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<(), ValidationError> {
    if !obj[field].is_string() {
        return Err(ValidationError::WrongFieldType(field, "string"));
    }
    Ok(())
}

fn check_array(
    obj: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<(), ValidationError> {
    if !obj[field].is_array() {
        return Err(ValidationError::WrongFieldType(field, "array"));
    }
    Ok(())
}

fn check_object(
    obj: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<(), ValidationError> {
    if !obj[field].is_object() {
        return Err(ValidationError::WrongFieldType(field, "object"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_event_json() -> Value {
        json!({
            "protocol_version": "0.1",
            "type": "message.text",
            "event_id": "xgen://hash/sha256:abc",
            "sender": "xgen://pubkey/ed25519:xyz",
            "room_id": "xgen://hash/sha256:room",
            "space_id": "xgen://hash/sha256:space",
            "prev_events": [],
            "timestamp": "2026-04-27T12:00:00Z",
            "content": {"text": "hello"},
            "signature": "ed25519:key:sig",
        })
    }

    #[test]
    fn valid_event_passes() {
        let ev = valid_event_json();
        let bytes = serde_json::to_vec(&ev).unwrap();
        let result = validate_event_bytes(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn step1_size_rejected() {
        let oversized = vec![0u8; MAX_PAYLOAD_BYTES + 1];
        assert!(matches!(
            validate_event_bytes(&oversized),
            Err(ValidationError::TooLarge(_))
        ));
    }

    #[test]
    fn step2_invalid_json_rejected() {
        assert!(matches!(
            validate_event_bytes(b"not json"),
            Err(ValidationError::InvalidJson(_))
        ));
        assert!(matches!(
            validate_event_bytes(b"[1,2,3]"), // array, not object
            Err(ValidationError::InvalidJson(_))
        ));
    }

    #[test]
    fn step3_missing_field_rejected() {
        let mut ev = valid_event_json();
        ev.as_object_mut().unwrap().remove("sender");
        let bytes = serde_json::to_vec(&ev).unwrap();
        assert!(matches!(
            validate_event_bytes(&bytes),
            Err(ValidationError::MissingField("sender"))
        ));
    }

    #[test]
    fn step4_wrong_type_rejected() {
        let mut ev = valid_event_json();
        ev["sender"] = json!(42); // should be string
        let bytes = serde_json::to_vec(&ev).unwrap();
        assert!(matches!(
            validate_event_bytes(&bytes),
            Err(ValidationError::WrongFieldType("sender", "string"))
        ));
    }

    #[test]
    fn step4_prev_events_not_array_rejected() {
        let mut ev = valid_event_json();
        ev["prev_events"] = json!("not_an_array");
        let bytes = serde_json::to_vec(&ev).unwrap();
        assert!(matches!(
            validate_event_bytes(&bytes),
            Err(ValidationError::WrongFieldType("prev_events", "array"))
        ));
    }

    #[test]
    fn step4_content_not_object_rejected() {
        let mut ev = valid_event_json();
        ev["content"] = json!("just a string");
        let bytes = serde_json::to_vec(&ev).unwrap();
        assert!(matches!(
            validate_event_bytes(&bytes),
            Err(ValidationError::WrongFieldType("content", "object"))
        ));
    }

    #[test]
    fn step5_wrong_version_rejected() {
        let mut ev = valid_event_json();
        ev["protocol_version"] = json!("1.0");
        let bytes = serde_json::to_vec(&ev).unwrap();
        assert!(matches!(
            validate_event_bytes(&bytes),
            Err(ValidationError::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn step6_unknown_type_rejected() {
        let mut ev = valid_event_json();
        ev["type"] = json!("bogus.type");
        let bytes = serde_json::to_vec(&ev).unwrap();
        assert!(matches!(
            validate_event_bytes(&bytes),
            Err(ValidationError::UnknownEventType(_))
        ));
    }

    #[test]
    fn step7_invalid_timestamp_rejected() {
        let mut ev = valid_event_json();
        ev["timestamp"] = json!("not-a-date");
        let bytes = serde_json::to_vec(&ev).unwrap();
        assert!(matches!(
            validate_event_bytes(&bytes),
            Err(ValidationError::InvalidTimestamp(_))
        ));
    }

    #[test]
    fn step7_timestamp_without_timezone_rejected() {
        let mut ev = valid_event_json();
        ev["timestamp"] = json!("2026-04-27T12:00:00"); // missing timezone
        let bytes = serde_json::to_vec(&ev).unwrap();
        assert!(matches!(
            validate_event_bytes(&bytes),
            Err(ValidationError::InvalidTimestamp(_))
        ));
    }

    #[test]
    fn step7_timestamp_with_offset_accepted() {
        let mut ev = valid_event_json();
        ev["timestamp"] = json!("2026-04-27T14:00:00+02:00");
        let bytes = serde_json::to_vec(&ev).unwrap();
        assert!(validate_event_bytes(&bytes).is_ok());
    }

    #[test]
    fn validated_event_has_correct_fields() {
        let ev = valid_event_json();
        let bytes = serde_json::to_vec(&ev).unwrap();
        let result = validate_event_bytes(&bytes).unwrap();
        assert_eq!(result.event_type, EventType::MessageText);
        assert_eq!(result.event_id.as_deref(), Some("xgen://hash/sha256:abc"));
        assert_eq!(result.signature.as_deref(), Some("ed25519:key:sig"));
    }
}
