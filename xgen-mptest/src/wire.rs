// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Harness-side `--aicontrol` wire types — the *opposite* serde direction from
//! the binaries.
//!
//! The binaries' `xgen-common::aicontrol::envelope` derives are direction-
//! specific: [`Command`] is `Deserialize`-only (the server reads it) and
//! `Reply` is `Serialize`-only (the server writes it). The harness needs the
//! mirror: it **writes** a Command and **reads** a Reply. Rather than add the
//! opposite derives to the shipping `xgen-common` crate (a shipped-code change
//! for a test-only need — the harness must not perturb the binaries beyond
//! drive/observe), the harness defines its own minimal mirror structs here.
//!
//! **Drift lock (D-067 spirit).** The wire JSON shapes are pinned by tests in
//! this module against literal envelopes that match `xgen-common` byte-for-byte
//! (`{"cmd","args","id","bind"}` outbound; `{"status":"ok",...}` /
//! `{"status":"error",...}` inbound). If the binaries' envelope changes, the
//! Round-0 smokes fail loudly and these tests are the canary.

use serde::{Deserialize, Serialize};

/// One outbound JSONL command (mirror of `xgen-common::aicontrol::Command`,
/// serialize side). `args` defaults to `{}` so no-arg verbs (`state`/`whoami`)
/// can omit it; absent optional fields are skipped (the binaries' `Command` is
/// `#[serde(default)]` for all of them, and explicitly must **not** carry
/// `deny_unknown_fields`).
#[derive(Debug, Clone, Serialize)]
pub struct Command {
    pub cmd: String,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty", default)]
    pub args: serde_json::Map<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<String>,
    /// AC-D4 opaque per-connection token (top-level, never in `args`). Inert in
    /// v1 (absent ⇒ proceed); the harness leaves it `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// AC-D6 opaque idempotency key (top-level). Left `None` by the harness.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

impl Command {
    /// A command with `cmd` only.
    pub fn new(cmd: impl Into<String>) -> Self {
        Command {
            cmd: cmd.into(),
            args: serde_json::Map::new(),
            id: None,
            bind: None,
            token: None,
            idempotency_key: None,
        }
    }

    /// Serialize to a single JSONL line (no trailing newline).
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).expect("Command serialize is infallible")
    }
}

/// The structured error body (mirror of `xgen-common::aicontrol::ErrorBody`,
/// deserialize side). `category` is one of the closed set
/// protocol/lifecycle/argument/connection/timeout/permission.
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorBody {
    pub code: String,
    pub category: String,
    pub message: String,
    #[serde(default)]
    pub instance_state: String,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub hint: Option<String>,
    /// MP-F5: the node's wire reject code (3030/3045/…) on a verb reject.
    #[serde(default)]
    pub reject_code: Option<u32>,
    /// MP-F5: the rejected event's id (correlation key) on a verb reject.
    #[serde(default)]
    pub event_id: Option<String>,
}

/// One inbound JSONL reply (mirror of `xgen-common::aicontrol::Reply`,
/// deserialize side). `status` tags the variant on the wire.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Reply {
    Ok {
        cmd: String,
        #[serde(default)]
        id: Option<String>,
        data: serde_json::Value,
    },
    Error {
        #[serde(default)]
        cmd: Option<String>,
        #[serde(default)]
        id: Option<String>,
        error: ErrorBody,
    },
}

impl Reply {
    /// Parse one JSONL reply line.
    pub fn from_line(line: &str) -> crate::Result<Reply> {
        Ok(serde_json::from_str(line)?)
    }

    /// `true` for a success reply.
    pub fn is_ok(&self) -> bool {
        matches!(self, Reply::Ok { .. })
    }

    /// The echoed command id, if any.
    pub fn id(&self) -> Option<&str> {
        match self {
            Reply::Ok { id, .. } | Reply::Error { id, .. } => id.as_deref(),
        }
    }

    /// The echoed command verb, if any (`cmd` is an envelope-level field — a
    /// sibling of `data`/`error`, not inside `data`). Absent on a
    /// `MALFORMED_COMMAND` error (nothing to echo).
    pub fn cmd(&self) -> Option<&str> {
        match self {
            Reply::Ok { cmd, .. } => Some(cmd),
            Reply::Error { cmd, .. } => cmd.as_deref(),
        }
    }

    /// The success `data` payload, if this is an `Ok` reply.
    pub fn data(&self) -> Option<&serde_json::Value> {
        match self {
            Reply::Ok { data, .. } => Some(data),
            Reply::Error { .. } => None,
        }
    }

    /// The error body, if this is an `Error` reply.
    pub fn error(&self) -> Option<&ErrorBody> {
        match self {
            Reply::Error { error, .. } => Some(error),
            Reply::Ok { .. } => None,
        }
    }

    /// Pull a string field out of an `Ok` reply's `data` object (e.g.
    /// `"identity_id"`, `"space_id"`, `"event_id"`). `None` if not `Ok`, the
    /// data is not an object, the field is absent, or it is not a string.
    pub fn data_str(&self, field: &str) -> Option<&str> {
        self.data()?.get(field)?.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn command_serializes_to_canonical_jsonl() {
        let mut c = Command::new("create-space");
        c.args.insert("name".into(), json!("S"));
        c.id = Some("a2".into());
        c.bind = Some("s".into());
        let line = c.to_line();
        // Order is struct-declaration order under serde_json: cmd, args, id, bind.
        assert_eq!(
            line,
            r#"{"cmd":"create-space","args":{"name":"S"},"id":"a2","bind":"s"}"#
        );
    }

    #[test]
    fn command_no_args_omits_empty_args_object() {
        let c = Command::new("state");
        // `state` takes no args; the empty map is skipped so the binary's
        // `#[serde(default)]` args fills in `{}`.
        assert_eq!(c.to_line(), r#"{"cmd":"state"}"#);
    }

    #[test]
    fn reply_ok_parses_and_exposes_data_fields() {
        // Byte-shape matches xgen-common Reply::Ok on the wire.
        let line = r#"{"status":"ok","cmd":"register","id":"b1","data":{"identity_id":"xgen://pubkey/ed25519:BOB"}}"#;
        let r = Reply::from_line(line).unwrap();
        assert!(r.is_ok());
        assert_eq!(r.id(), Some("b1"));
        assert_eq!(r.data_str("identity_id"), Some("xgen://pubkey/ed25519:BOB"));
    }

    #[test]
    fn reply_error_parses_body_and_category() {
        // Byte-shape matches xgen-common Reply::Error (flat ErrorBody, AC-D2).
        let line = r#"{"status":"error","cmd":"join","id":"b2","error":{"code":"BAD_ARGUMENT","category":"argument","message":"bad","instance_state":"ready"}}"#;
        let r = Reply::from_line(line).unwrap();
        assert!(!r.is_ok());
        let e = r.error().unwrap();
        assert_eq!(e.code, "BAD_ARGUMENT");
        assert_eq!(e.category, "argument");
        assert_eq!(r.data_str("anything"), None);
    }

    /// MP-F5 drift-lock: a verb-reject Error reply carries the additive
    /// `reject_code` + `event_id` fields (the node's wire code + the rejected
    /// event id, surfaced through the client per MP-F5-D2). `code` stays
    /// `GENERIC_4000` (AC-D3d-preserving). Pins the byte-shape against the
    /// shipping `xgen-common::ErrorBody`; the absent case stays parseable
    /// (forward-compat — old replies omit both).
    #[test]
    fn reply_error_parses_mp_f5_reject_fields() {
        let line = r#"{"status":"error","cmd":"join","id":"b2","error":{"code":"GENERIC_4000","category":"protocol","message":"join rejected by node (code 3030): tier_mismatch (3030): joiner tier 1 < required 2","instance_state":"ready","reject_code":3030,"event_id":"xgen://hash/sha256:abc"}}"#;
        let r = Reply::from_line(line).unwrap();
        let e = r.error().unwrap();
        assert_eq!(e.code, "GENERIC_4000", "client-surface code stays GENERIC_4000 (AC-D3d)");
        assert_eq!(e.reject_code, Some(3030), "wire reject code surfaced");
        assert_eq!(e.event_id.as_deref(), Some("xgen://hash/sha256:abc"));

        // Forward-compat: an Error without the MP-F5 fields still parses (None).
        let bare = r#"{"status":"error","cmd":"join","id":"b2","error":{"code":"BAD_ARGUMENT","category":"argument","message":"bad","instance_state":"ready"}}"#;
        let e2 = Reply::from_line(bare).unwrap();
        let e2 = e2.error().unwrap();
        assert_eq!(e2.reject_code, None);
        assert_eq!(e2.event_id, None);
    }

    #[test]
    fn malformed_reply_line_errors() {
        assert!(Reply::from_line("not json").is_err());
    }
}
