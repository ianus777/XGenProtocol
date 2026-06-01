// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! AC-D2 wire envelope — the inbound [`Command`] and the outbound [`Reply`].
//!
//! Flat error shape (AC-D2): mandatory `code`/`category`/`message`/
//! `instance_state`; optional-by-source `stage`/`hint`. The closed
//! [`Category`] set alone disambiguates the `code` namespace — drivers branch
//! on `category`, never parse `code`. See `docs/xgen_aicontrol_implementation.md`
//! §4.2/§4.3.

use serde::{Deserialize, Serialize};

use super::codes::{ControlCode, ControlError};

/// The closed `category` enumerated set (AC-D2). Adding a category is a
/// deliberate envelope change, not an ad-hoc string. `protocol` is
/// verb-sourced only — control-surface errors never use it (AC-D3d invariant,
/// enforced in [`super::codes::ControlCode::category`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Protocol,
    Lifecycle,
    Argument,
    Connection,
    Timeout,
    Permission,
}

/// One inbound JSONL command (§4.1).
///
/// `args` defaults to an empty object so the no-arg verbs (`whoami` /
/// `status` / `state`) may omit it entirely — the §4.1 "required object"
/// wording is honored leniently for the verbs that genuinely take none.
///
/// **Known coupling (CP-2 lock item b):** this struct must NOT gain
/// `#[serde(deny_unknown_fields)]`. The additive forward-compat of optional
/// fields like `token` (a driver that omits it deserialises to `None` →
/// `absent==proceed`) and the inert-seam story both depend on unknown/absent
/// fields parsing cleanly. Adding `deny_unknown_fields` would break older
/// senders and re-litigate every future optional field.
#[derive(Debug, Clone, Deserialize)]
pub struct Command {
    /// The command verb / CLI-path string (AC-D1; resolved by
    /// [`super::cmd::resolve_cmd`]).
    pub cmd: String,
    /// Named arguments (`snake_case` keys per §4.1). Empty when omitted.
    #[serde(default)]
    pub args: serde_json::Map<String, serde_json::Value>,
    /// Driver-supplied correlation id, echoed verbatim into the reply.
    #[serde(default)]
    pub id: Option<String>,
    /// Names this command's result for later `$`-substitution (§5).
    #[serde(default)]
    pub bind: Option<String>,
    /// AC-D4 per-connection control token (M7C-D1, B1). A **top-level** envelope
    /// field — deliberately NOT inside `args` (an `args` entry would be
    /// reconstructed into a `--token` clap flag and break the parse). Carried as
    /// an **opaque String**, unchanged: verification interprets it, the envelope
    /// does not. `absent==proceed`; the seam is inert in v1 (no expected token
    /// configured). **B-subsumable:** end-state B's driver-bound credential rides
    /// this same field with no wire change. See [`super::token::check_token`].
    #[serde(default)]
    pub token: Option<String>,
    /// AC-D6 idempotency key (M7C-D2, B2). Same shape rule as `token`: a
    /// **top-level**, **opaque** String (never in `args`; never reaches clap;
    /// carried unchanged). `absent==do-it-over`. A completed, successful command
    /// bearing a key is recorded per-`.aicontrol`-session (the per-connection
    /// handler state); a later command with the same key returns the prior result
    /// without re-executing. Scope (per-session now → per-driver later) lives in
    /// *where the store sits*, not on the wire — so this field is B-subsumable.
    /// See [`super::idempotency::IdempotencyStore`].
    #[serde(default)]
    pub idempotency_key: Option<String>,
}

/// Parse one JSONL line into a [`Command`].
///
/// A line that is not valid JSON, or is valid JSON lacking a non-empty `cmd`,
/// fails with [`ControlCode::MalformedCommand`] (AC-D3d) — on that error the
/// caller omits the echoed `cmd`/`id` (there is nothing to echo).
pub fn parse_command(line: &str) -> Result<Command, ControlError> {
    let cmd: Command = serde_json::from_str(line).map_err(|e| {
        ControlError::new(
            ControlCode::MalformedCommand,
            format!("line is not a valid command object: {e}"),
        )
    })?;
    if cmd.cmd.trim().is_empty() {
        return Err(ControlError::new(
            ControlCode::MalformedCommand,
            "command object has an empty `cmd`",
        ));
    }
    Ok(cmd)
}

/// The structured error body (§4.3). `stage`/`hint` are omitted from the wire
/// when absent (`skip_serializing_if`).
#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    /// Band code (`SPACE_8005`, …), `GENERIC_4000`, or an uppercase-snake
    /// control code (`BAD_ARGUMENT`, …).
    pub code: String,
    pub category: Category,
    /// Human-readable; not for programmatic parsing.
    pub message: String,
    /// The instance's lifecycle state at the time of the error.
    pub instance_state: String,
    /// The verb failure stage — present only for Node verb errors (the 6
    /// shipped `Stage` values as strings). Never present on control errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    /// A suggested next command, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// One outbound JSONL reply (§4.2). `status` tags the variant on the wire
/// (`"ok"` / `"error"`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Reply {
    /// Success — carries `data` only. `cmd` always echoed; `id` iff supplied.
    Ok {
        cmd: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: serde_json::Value,
    },
    /// Failure — carries the structured [`ErrorBody`]. `cmd` is omitted on a
    /// `MALFORMED_COMMAND` (nothing to echo).
    Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        cmd: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        error: ErrorBody,
    },
}

impl Reply {
    /// Build a success reply.
    pub fn ok(cmd: impl Into<String>, id: Option<String>, data: serde_json::Value) -> Self {
        Reply::Ok {
            cmd: cmd.into(),
            id,
            data,
        }
    }

    /// Build an error reply.
    pub fn error(cmd: Option<String>, id: Option<String>, error: ErrorBody) -> Self {
        Reply::Error { cmd, id, error }
    }

    /// True for a success reply. Used by the AC-D6 result-time idempotency
    /// binding (B2): only completed, successful operations are recorded.
    pub fn is_ok(&self) -> bool {
        matches!(self, Reply::Ok { .. })
    }

    /// Serialise to one JSONL line (no trailing newline — the pipe writer
    /// appends it). Infallible: the reply only holds owned strings and a
    /// `serde_json::Value`, both of which always serialise.
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).expect("Reply always serialises (owned strings + Value)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn token_absent_deserialises_to_none() {
        // absent==proceed seam: a driver that omits `token` parses cleanly.
        let c = parse_command(r#"{"cmd":"whoami"}"#).unwrap();
        assert!(c.token.is_none());
    }

    #[test]
    fn token_round_trips_arbitrary_opaque_value_unchanged() {
        // B-subsumability witness (M7C-D1, B1): the envelope carries any opaque
        // token value byte-for-byte — so end-state B's driver-bound credential
        // (a signed capability, a JWT-shaped blob, anything) rides this same
        // field with no wire change. The envelope must never parse/normalise it.
        let opaque = "B-cred::v9|node=xgen://pubkey/ed25519:ABC.def+/=|sig:Zm9v$bar 𝔘nicode";
        let line = format!(
            r#"{{"cmd":"whoami","token":{}}}"#,
            serde_json::to_string(opaque).unwrap()
        );
        let c = parse_command(&line).unwrap();
        assert_eq!(c.token.as_deref(), Some(opaque), "opaque token round-trips unchanged");
    }

    #[test]
    fn token_does_not_collide_with_existing_envelope_fields() {
        // token is a sibling of cmd/args/id/bind; all coexist.
        let c = parse_command(
            r#"{"cmd":"join","args":{"space":"xgen://hash/sha256:S"},"id":"c1","bind":"j","token":"t"}"#,
        )
        .unwrap();
        assert_eq!(c.cmd, "join");
        assert_eq!(c.id.as_deref(), Some("c1"));
        assert_eq!(c.bind.as_deref(), Some("j"));
        assert_eq!(c.token.as_deref(), Some("t"));
        assert_eq!(c.args.get("space").and_then(|v| v.as_str()), Some("xgen://hash/sha256:S"));
    }

    #[test]
    fn idempotency_key_absent_deserialises_to_none() {
        // absent==do-it-over seam: omitting the key parses cleanly.
        let c = parse_command(r#"{"cmd":"create-dm-space","args":{"invitee":"x"}}"#).unwrap();
        assert!(c.idempotency_key.is_none());
    }

    #[test]
    fn idempotency_key_round_trips_arbitrary_opaque_value_unchanged() {
        // B-subsumability witness (M7C-D2, B2): same shape rule as `token` —
        // the envelope carries any opaque key value byte-for-byte, so a future
        // per-driver key scheme rides this field with no wire change.
        let opaque = "idem::v3|driver=xgen://pubkey/ed25519:XYZ|n=42+/=|𝔘nicode";
        let line = format!(
            r#"{{"cmd":"whoami","idempotency_key":{}}}"#,
            serde_json::to_string(opaque).unwrap()
        );
        let c = parse_command(&line).unwrap();
        assert_eq!(c.idempotency_key.as_deref(), Some(opaque), "opaque key round-trips unchanged");
    }

    #[test]
    fn ok_reply_serialises_with_status_and_omits_absent_id() {
        let r = Reply::ok("create-space", None, json!({"space_id": "xgen://hash/sha256:S"}));
        let line = r.to_line();
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(v["status"], "ok");
        assert_eq!(v["cmd"], "create-space");
        assert_eq!(v["data"]["space_id"], "xgen://hash/sha256:S");
        assert!(v.get("id").is_none(), "absent id omitted: {line}");
        assert!(v.get("error").is_none());
    }

    #[test]
    fn ok_reply_echoes_id_when_present() {
        let r = Reply::ok("whoami", Some("c-7".to_string()), json!({"identity_id": "x"}));
        let v: serde_json::Value = serde_json::from_str(&r.to_line()).unwrap();
        assert_eq!(v["id"], "c-7");
    }

    #[test]
    fn node_verb_error_serialises_with_band_code_and_stage() {
        let body = ErrorBody {
            code: "SPACE_8005".to_string(),
            category: Category::Protocol,
            message: "action_threshold out of range".to_string(),
            instance_state: "running".to_string(),
            stage: Some("validate".to_string()),
            hint: None,
        };
        let r = Reply::error(Some("space set-node-policy".to_string()), None, body);
        let v: serde_json::Value = serde_json::from_str(&r.to_line()).unwrap();
        assert_eq!(v["status"], "error");
        assert_eq!(v["cmd"], "space set-node-policy");
        assert_eq!(v["error"]["code"], "SPACE_8005");
        assert_eq!(v["error"]["category"], "protocol");
        assert_eq!(v["error"]["stage"], "validate");
        assert!(v["error"].get("hint").is_none(), "absent hint omitted");
    }

    #[test]
    fn client_anyhow_error_is_message_only_no_stage() {
        let body = ErrorBody {
            code: "GENERIC_4000".to_string(),
            category: Category::Protocol,
            message: "connection refused".to_string(),
            instance_state: "ready".to_string(),
            stage: None,
            hint: None,
        };
        let r = Reply::error(Some("send".to_string()), None, body);
        let v: serde_json::Value = serde_json::from_str(&r.to_line()).unwrap();
        assert_eq!(v["error"]["code"], "GENERIC_4000");
        assert!(v["error"].get("stage").is_none(), "client errors carry no stage");
    }

    #[test]
    fn malformed_command_reply_omits_cmd() {
        let body = ControlError::new(ControlCode::MalformedCommand, "bad json").into_body("ready");
        let r = Reply::error(None, None, body);
        let v: serde_json::Value = serde_json::from_str(&r.to_line()).unwrap();
        assert!(v.get("cmd").is_none(), "MALFORMED_COMMAND omits echoed cmd");
        assert_eq!(v["error"]["code"], "MALFORMED_COMMAND");
        assert_eq!(v["error"]["category"], "argument");
    }

    #[test]
    fn command_parses_with_args_id_bind() {
        let c = parse_command(r#"{"cmd":"send","args":{"text":"hi"},"id":"7","bind":"m"}"#).unwrap();
        assert_eq!(c.cmd, "send");
        assert_eq!(c.args["text"], "hi");
        assert_eq!(c.id.as_deref(), Some("7"));
        assert_eq!(c.bind.as_deref(), Some("m"));
    }

    #[test]
    fn no_arg_command_parses_with_empty_args() {
        let c = parse_command(r#"{"cmd":"whoami"}"#).unwrap();
        assert_eq!(c.cmd, "whoami");
        assert!(c.args.is_empty());
        assert!(c.id.is_none());
        assert!(c.bind.is_none());
    }

    #[test]
    fn non_json_line_is_malformed_command() {
        let e = parse_command("this is not json").unwrap_err();
        assert_eq!(e.code, ControlCode::MalformedCommand);
    }

    #[test]
    fn json_without_cmd_field_is_malformed_command() {
        let e = parse_command(r#"{"args":{}}"#).unwrap_err();
        assert_eq!(e.code, ControlCode::MalformedCommand);
    }

    #[test]
    fn empty_cmd_string_is_malformed_command() {
        let e = parse_command(r#"{"cmd":"   "}"#).unwrap_err();
        assert_eq!(e.code, ControlCode::MalformedCommand);
    }
}
