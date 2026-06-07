// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! Capture-by-default (M9-D4 §3.4 / C2).
//!
//! Every scenario run writes an artifact directory so a one-shot (R3) run is
//! reproducible after the fact (audit §6.2 rec 3). C2 writes the
//! oracle-relevant artifacts:
//! - `manifest.resolved.toml` — the manifest as the run actually used it;
//! - `<actor>.replies.jsonl` — each command id + its reply envelope;
//! - `<node>.events.jsonl` — the node's collected `.events` transcript;
//! - `verdict.txt` — the oracle verdict.
//!
//! **Deferred to C3:** per-process **RSS + thread-count** samples. That sampler
//! is the C3 micro-benchmark's headline deliverable (it derives the box-ceiling
//! report); C2 captures the convergence/rejection evidence and leaves resource
//! sampling to land with the round dial. Recorded here so it is a known gap, not
//! a silent one.

use std::path::{Path, PathBuf};

use crate::batch::ActorRun;
use crate::oracle::OracleVerdict;
use crate::process::label_nonce;
use crate::Result;
use anyhow::Context;

/// One run's artifact directory.
pub struct Capture {
    dir: PathBuf,
}

impl Capture {
    /// Create `<root>/<scenario>-<nonce>/`. The default root is
    /// `<exe_dir>/mptest-artifacts` unless `root` is given.
    pub fn new(root: impl AsRef<Path>, scenario: &str) -> Result<Capture> {
        let dir = root
            .as_ref()
            .join(format!("{scenario}-{}", label_nonce()));
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("creating artifact dir {}", dir.display()))?;
        Ok(Capture { dir })
    }

    /// The artifact directory path.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Write the resolved manifest text.
    pub fn write_resolved_manifest(&self, toml_text: &str) -> Result<()> {
        self.write_file("manifest.resolved.toml", toml_text)
    }

    /// Write one actor's command/reply log as JSONL (`{"id":..,"reply":<envelope>}`).
    pub fn write_actor_log(&self, run: &ActorRun) -> Result<()> {
        let mut out = String::new();
        for (id, reply) in &run.replies {
            let line = serde_json::json!({
                "id": id,
                "ok": reply.is_ok(),
                "cmd": reply.cmd(),
                "reply": reply_to_value(reply),
            });
            out.push_str(&line.to_string());
            out.push('\n');
        }
        self.write_file(&format!("{}.replies.jsonl", run.actor), &out)
    }

    /// Write a node's collected `.events` transcript as JSONL (raw events).
    pub fn write_transcript(&self, node_label: &str, events: &[serde_json::Value]) -> Result<()> {
        let mut out = String::new();
        for e in events {
            out.push_str(&e.to_string());
            out.push('\n');
        }
        self.write_file(&format!("{node_label}.events.jsonl"), &out)
    }

    /// Write the oracle verdict.
    pub fn write_verdict(&self, verdict: &OracleVerdict) -> Result<()> {
        let body = format!(
            "{}\n{}\n",
            if verdict.pass { "PASS" } else { "FAIL" },
            verdict.detail
        );
        self.write_file("verdict.txt", &body)
    }

    fn write_file(&self, name: &str, body: &str) -> Result<()> {
        let p = self.dir.join(name);
        std::fs::write(&p, body).with_context(|| format!("writing {}", p.display()))
    }
}

/// Re-serialize a reply to a JSON value for capture (the harness `Reply` is
/// deserialize-only, so round-trip via its observable fields).
fn reply_to_value(reply: &crate::wire::Reply) -> serde_json::Value {
    match reply {
        crate::wire::Reply::Ok { cmd, id, data } => serde_json::json!({
            "status": "ok", "cmd": cmd, "id": id, "data": data,
        }),
        crate::wire::Reply::Error { cmd, id, error } => serde_json::json!({
            "status": "error", "cmd": cmd, "id": id,
            "error": {
                "code": error.code, "category": error.category,
                "message": error.message, "instance_state": error.instance_state,
                "stage": error.stage, "hint": error.hint,
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::OracleVerdict;
    use crate::wire::Reply;

    #[test]
    fn capture_writes_artifacts() {
        let tmp = tempfile::tempdir().unwrap();
        let cap = Capture::new(tmp.path(), "MP-C-02").unwrap();
        assert!(cap.dir().exists());

        cap.write_resolved_manifest("scenario = \"MP-C-02\"\n").unwrap();
        cap.write_verdict(&OracleVerdict::pass("converged")).unwrap();
        cap.write_transcript(
            "a",
            &[serde_json::json!({"type":"membership.join","event_id":"E1"})],
        )
        .unwrap();

        let run = ActorRun {
            actor: "alice".to_string(),
            replies: vec![(
                Some("a1".to_string()),
                Reply::from_line(
                    r#"{"status":"ok","cmd":"register","id":"a1","data":{"identity_id":"ALICE"}}"#,
                )
                .unwrap(),
            )],
        };
        cap.write_actor_log(&run).unwrap();

        assert!(cap.dir().join("manifest.resolved.toml").exists());
        assert!(cap.dir().join("verdict.txt").exists());
        assert!(cap.dir().join("a.events.jsonl").exists());
        assert!(cap.dir().join("alice.replies.jsonl").exists());

        let verdict = std::fs::read_to_string(cap.dir().join("verdict.txt")).unwrap();
        assert!(verdict.starts_with("PASS"));
    }
}
