// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Per-actor batch runner (M9-D8 / Checkpoint #1).
//!
//! Reads an actor's `<name>.jsonl`, and for each command line:
//! 1. resolves any `{{key}}` placeholders via the shared [`Registry`] (blocking
//!    until the producing actor publishes them — the data-dependency ordering);
//! 2. [`substitute`]s the resolved values into the **raw** line (the author's
//!    exact JSON, so `args` order and `$`-bindings are preserved untouched);
//! 3. sends the line over the actor's `.aicontrol` client and captures the reply;
//! 4. if this command's `id` is an export source, publishes the named reply
//!    field back to the registry for downstream actors.
//!
//! The runner sends the line **as written** (it never re-serializes through
//! [`crate::wire::Command`]) — it only *reads* the `id` to match exports. This
//! keeps the harness faithful to the saved batch file (no silent reshaping).

use std::collections::HashMap;
use std::time::Duration;

use crate::aicontrol::AicontrolClient;
use crate::manifest::Manifest;
use crate::resolve::{placeholders_in, substitute, Registry};
use crate::wire::Reply;
use crate::Result;
use anyhow::Context;

/// One parsed batch line: the raw JSON plus its extracted `id` (if any).
#[derive(Debug, Clone)]
pub struct BatchLine {
    /// The raw JSON line, exactly as authored.
    pub raw: String,
    /// The `"id"` field value, if present (matches exports + correlates replies).
    pub id: Option<String>,
}

/// Parse a batch file's text into command lines, skipping blank lines and
/// `#`-prefixed comments. Each retained line must be a JSON object; its `id`
/// field (if any) is extracted for export matching.
pub fn parse_batch_lines(text: &str) -> Result<Vec<BatchLine>> {
    let mut out = Vec::new();
    for (lineno, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(trimmed)
            .with_context(|| format!("batch line {} is not valid JSON: {trimmed}", lineno + 1))?;
        let id = value
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        out.push(BatchLine {
            raw: trimmed.to_string(),
            id,
        });
    }
    Ok(out)
}

/// The outcome of driving one actor's batch.
#[derive(Debug, Clone)]
pub struct ActorRun {
    pub actor: String,
    /// Replies in send order, paired with the originating line id (if any).
    pub replies: Vec<(Option<String>, Reply)>,
}

impl ActorRun {
    /// `true` iff every reply was a success.
    pub fn all_ok(&self) -> bool {
        self.replies.iter().all(|(_, r)| r.is_ok())
    }

    /// The reply for a given command id, if present.
    pub fn reply_for(&self, id: &str) -> Option<&Reply> {
        self.replies
            .iter()
            .find(|(rid, _)| rid.as_deref() == Some(id))
            .map(|(_, r)| r)
    }
}

/// Drive one actor's batch end-to-end over an already-connected client.
///
/// `manifest` supplies this actor's exports; `registry` is shared across all
/// actors; `resolve_timeout` bounds how long a `{{key}}` may block waiting for
/// its producer.
pub async fn run_actor(
    actor: &str,
    lines: &[BatchLine],
    client: &mut AicontrolClient,
    manifest: &Manifest,
    registry: &Registry,
    resolve_timeout: Duration,
) -> Result<ActorRun> {
    let mut replies = Vec::with_capacity(lines.len());
    // Explicit happens-after edges (command_id → extra wait keys) — ordering the
    // args data-dependency cannot express (manifest `[[waits]]`).
    let extra_waits = manifest.waits_of(actor);

    for line in lines {
        // 1a. Explicit ordering waits for this command (not in its args).
        if let Some(id) = line.id.as_deref() {
            if let Some(keys) = extra_waits.get(id) {
                for key in keys {
                    registry.wait_for(key, resolve_timeout).await.with_context(|| {
                        format!("actor `{actor}` waiting on ordering key `{{{{{key}}}}}` for `{id}`")
                    })?;
                }
            }
        }

        // 1b. Resolve cross-actor placeholders (blocks until each is published).
        let keys = placeholders_in(&line.raw);
        let mut resolved: HashMap<String, String> = HashMap::new();
        for key in keys {
            let value = registry
                .wait_for(&key, resolve_timeout)
                .await
                .with_context(|| {
                    format!("actor `{actor}` resolving `{{{{{key}}}}}` for line `{}`", line.raw)
                })?;
            resolved.insert(key, value);
        }

        // 2. Substitute into the raw line (author's exact JSON preserved).
        let resolved_line = substitute(&line.raw, &resolved);

        // 3. Send + capture.
        let reply = client
            .send_line(&resolved_line)
            .await
            .with_context(|| format!("actor `{actor}` sending line `{resolved_line}`"))?;

        // 4. Publish exports keyed on this command's id (only on success).
        if let (Some(id), true) = (line.id.as_deref(), reply.is_ok()) {
            for export in manifest.exports_of(actor).filter(|e| e.command == id) {
                match reply.data_str(&export.field) {
                    Some(v) => registry.publish(export.key.clone(), v.to_string()).await,
                    None => {
                        anyhow::bail!(
                            "actor `{actor}` export `{}` expected reply field `{}` on command \
                             `{id}` but it was absent",
                            export.key,
                            export.field
                        );
                    }
                }
            }
        }

        replies.push((line.id.clone(), reply));
    }

    Ok(ActorRun {
        actor: actor.to_string(),
        replies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lines_and_skips_comments_and_blanks() {
        let text = "\
# a comment
{\"cmd\":\"register\",\"args\":{\"name\":\"alice\"},\"id\":\"a1\",\"bind\":\"me\"}

{\"cmd\":\"create-space\",\"args\":{\"name\":\"S\"},\"id\":\"a2\",\"bind\":\"s\"}
";
        let lines = parse_batch_lines(text).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].id.as_deref(), Some("a1"));
        assert_eq!(lines[1].id.as_deref(), Some("a2"));
        assert!(lines[0].raw.contains("register"));
    }

    #[test]
    fn rejects_non_json_line() {
        let r = parse_batch_lines("not json at all");
        assert!(r.is_err());
        assert!(format!("{:#}", r.unwrap_err()).contains("not valid JSON"));
    }

    #[test]
    fn line_without_id_parses_with_none() {
        let lines = parse_batch_lines(r#"{"cmd":"send","args":{"space":"$s","text":"hi"}}"#).unwrap();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].id.is_none());
    }
}
