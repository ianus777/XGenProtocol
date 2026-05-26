// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: GPL-2.0-or-later
// Licensed under the GNU General Public License v2.0 or later
// See LICENSE-CORE in the project root for full terms.

// Transport frame encode / decode (spec 3.1.2).
//
// Frame layout:
//   [1B  format_len ] — byte count of the format string (u8)
//   [N B format     ] — UTF-8 format identifier, e.g. "json"
//   [4B  payload_len] — payload byte count, u32 big-endian
//   [M B payload    ] — raw payload bytes
//
// Phase 1 always uses format "json".  The encoder emits only "json" frames;
// the decoder accepts any format string and returns it alongside the payload
// so the caller can reject unknown formats if needed.

use thiserror::Error;

/// Maximum payload size in a single frame (Local Node mode ceiling, spec 3.1.2).
pub const MAX_PAYLOAD_BYTES: usize = 256 * 1024; // 256 KB

/// Format string used in Phase 1.
pub const FORMAT_JSON: &str = "json";

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("buffer too short: need at least {need} bytes, have {have}")]
    TooShort { need: usize, have: usize },
    #[error("format string is not valid UTF-8")]
    InvalidFormatUtf8,
    #[error("payload length {0} exceeds limit {}", MAX_PAYLOAD_BYTES)]
    PayloadTooLarge(u32),
    #[error("buffer contains {have} bytes after header, but payload length says {need}")]
    IncompletePayload { need: usize, have: usize },
}

/// Decoded frame: format string + raw payload bytes.
pub struct Frame {
    pub format: String,
    pub payload: Vec<u8>,
}

/// Wrap `payload` in a transport frame using the "json" format identifier.
pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let fmt = FORMAT_JSON.as_bytes();
    let fmt_len = fmt.len() as u8; // always 4 for "json"
    let pay_len = payload.len() as u32;

    let mut buf = Vec::with_capacity(1 + fmt.len() + 4 + payload.len());
    buf.push(fmt_len);
    buf.extend_from_slice(fmt);
    buf.extend_from_slice(&pay_len.to_be_bytes());
    buf.extend_from_slice(payload);
    buf
}

/// Parse the leading frame from `buf`.  Returns the decoded `Frame`.
/// Does not consume `buf`; the caller knows the frame boundary from `Frame::total_len()`.
pub fn decode_frame(buf: &[u8]) -> Result<Frame, FrameError> {
    // Byte 0: format string length
    if buf.is_empty() {
        return Err(FrameError::TooShort { need: 1, have: 0 });
    }
    let fmt_len = buf[0] as usize;

    // Bytes 1..1+fmt_len: format string
    let header_end = 1 + fmt_len + 4; // 1B len + N fmt + 4B payload_len
    if buf.len() < header_end {
        return Err(FrameError::TooShort { need: header_end, have: buf.len() });
    }
    let format = std::str::from_utf8(&buf[1..1 + fmt_len])
        .map_err(|_| FrameError::InvalidFormatUtf8)?
        .to_string();

    // Bytes 1+fmt_len .. header_end: payload length (u32 BE)
    let pay_len_bytes: [u8; 4] = buf[1 + fmt_len..header_end].try_into().unwrap();
    let pay_len = u32::from_be_bytes(pay_len_bytes);

    if pay_len as usize > MAX_PAYLOAD_BYTES {
        return Err(FrameError::PayloadTooLarge(pay_len));
    }

    let total = header_end + pay_len as usize;
    if buf.len() < total {
        return Err(FrameError::IncompletePayload {
            need: pay_len as usize,
            have: buf.len() - header_end,
        });
    }

    let payload = buf[header_end..total].to_vec();
    Ok(Frame { format, payload })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_round_trip() {
        let payload = b"hello xgen";
        let frame = encode_frame(payload);
        let decoded = decode_frame(&frame).unwrap();
        assert_eq!(decoded.format, FORMAT_JSON);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn frame_structure_is_correct() {
        let payload = b"{}";
        let frame = encode_frame(payload);
        // Byte 0: format length (4 for "json")
        assert_eq!(frame[0], 4);
        // Bytes 1-4: "json"
        assert_eq!(&frame[1..5], b"json");
        // Bytes 5-8: payload length big-endian (2 for "{}")
        assert_eq!(&frame[5..9], &[0, 0, 0, 2]);
        // Bytes 9-10: payload
        assert_eq!(&frame[9..], b"{}");
    }

    #[test]
    fn empty_payload_round_trip() {
        let frame = encode_frame(b"");
        let decoded = decode_frame(&frame).unwrap();
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn too_short_buffer_errors() {
        assert!(decode_frame(&[]).is_err());
        // Only the format-length byte, no format bytes
        assert!(decode_frame(&[4]).is_err());
    }

    #[test]
    fn incomplete_payload_errors() {
        let mut frame = encode_frame(b"hello xgen");
        frame.truncate(frame.len() - 3); // chop off last 3 bytes
        assert!(matches!(
            decode_frame(&frame),
            Err(FrameError::IncompletePayload { .. })
        ));
    }

    #[test]
    fn oversized_payload_rejected() {
        // Craft a frame header claiming a 300 KB payload (exceeds 256 KB limit)
        let fmt = b"json";
        let pay_len: u32 = (MAX_PAYLOAD_BYTES + 1024) as u32;
        let mut buf = vec![4u8];
        buf.extend_from_slice(fmt);
        buf.extend_from_slice(&pay_len.to_be_bytes());
        // Don't add any payload bytes — size check fires before reading payload
        assert!(matches!(
            decode_frame(&buf),
            Err(FrameError::PayloadTooLarge(_))
        ));
    }

    #[test]
    fn event_json_round_trip_through_frame() {
        use crate::wire::types::{Event, EventType};
        use serde_json::json;
        use xgen_common::xgid::{IdentityXgid, RoomXgid, SpaceXgid, Xgid};

        let ev = Event::new(
            EventType::MessageText,
            IdentityXgid::from_xgid(Xgid::new("xgen://pubkey/ed25519:abc".to_string())),
            RoomXgid::from_xgid(Xgid::new("xgen://hash/sha256:room".to_string())),
            SpaceXgid::from_xgid(Xgid::new("xgen://hash/sha256:space".to_string())),
            vec![],
            "2026-04-27T12:00:00Z".to_string(),
            json!({"text": "hello"}),
        );

        let payload = serde_json::to_vec(&ev).unwrap();
        let frame = encode_frame(&payload);
        let decoded = decode_frame(&frame).unwrap();
        assert_eq!(decoded.format, FORMAT_JSON);

        let ev2: Event = serde_json::from_slice(&decoded.payload).unwrap();
        assert_eq!(ev2.event_type, EventType::MessageText);
        assert_eq!(ev2.protocol_version, "0.1");
    }
}
