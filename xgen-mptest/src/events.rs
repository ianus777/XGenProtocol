// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! The `.events` observation pipe client + background collector (M9-D4 / C2).
//!
//! A node's `.events` pipe is **live-only** (it forwards only events the node
//! fans out *after* the subscriber connects — no history replay). So the oracle
//! attaches an [`EventCollector`] to each node **at scenario start**, before any
//! actor drives; by quiescence the collector holds the full ordered transcript
//! of everything that node applied during the run.
//!
//! Wire protocol (mirror of `xgen-node::events_pipe`): connect the pipe, send a
//! first message `{"cmd":"subscribe","args":{<filter>}}` (empty `args` ⇒ all
//! events), read one `Reply` ack line, then read raw `Event` JSON lines (the
//! full event, **not** wrapped in a Reply envelope) until close.
//!
//! Windows-only (named pipe). A non-Windows build compiles but errors at
//! connect, so the rest of the harness type-checks anywhere.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use crate::Result;

/// Connect-retry budget, mirroring [`crate::aicontrol::DEFAULT_CONNECT_TIMEOUT`].
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// A subscription filter (AC-D3b shape). An empty filter matches all events.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub spaces: Vec<String>,
    pub event_types: Vec<String>,
}

impl Filter {
    /// All events.
    pub fn all() -> Filter {
        Filter::default()
    }

    /// Render to the `subscribe` command line.
    fn to_subscribe_line(&self) -> String {
        let mut args = serde_json::Map::new();
        if !self.spaces.is_empty() {
            args.insert("spaces".into(), serde_json::json!(self.spaces));
        }
        if !self.event_types.is_empty() {
            args.insert("event_types".into(), serde_json::json!(self.event_types));
        }
        serde_json::json!({ "cmd": "subscribe", "args": args }).to_string()
    }
}

/// A running collector: a background task tails one node's `.events` pipe and
/// appends every received `Event` (as a raw JSON [`serde_json::Value`]) to a
/// shared buffer. Snapshot at quiescence with [`snapshot`](EventCollector::snapshot).
/// Dropping the collector aborts the task.
pub struct EventCollector {
    buffer: Arc<Mutex<Vec<serde_json::Value>>>,
    task: tokio::task::JoinHandle<()>,
    /// The node/actor label this collector is attached to (for capture naming).
    pub label: String,
}

impl EventCollector {
    /// Connect + subscribe + start tailing `events_pipe`. Returns once the
    /// subscribe ack is received, so the caller knows the collector is live
    /// before driving actors.
    pub async fn start(label: &str, events_pipe: &str, filter: Filter) -> Result<EventCollector> {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let task = imp::spawn_collector(events_pipe.to_string(), filter, buffer.clone()).await?;
        Ok(EventCollector {
            buffer,
            task,
            label: label.to_string(),
        })
    }

    /// A copy of everything received so far, in arrival order.
    pub async fn snapshot(&self) -> Vec<serde_json::Value> {
        self.buffer.lock().await.clone()
    }

    /// Number of events received so far.
    pub async fn len(&self) -> usize {
        self.buffer.lock().await.len()
    }

    /// `true` if nothing has been received yet.
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

impl Drop for EventCollector {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[cfg(windows)]
mod imp {
    use super::*;
    use anyhow::{anyhow, Context};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::windows::named_pipe::ClientOptions;
    use tokio::time::Instant;

    const ERROR_FILE_NOT_FOUND: i32 = 2;
    const ERROR_PIPE_BUSY: i32 = 231;

    pub(super) async fn spawn_collector(
        pipe: String,
        filter: Filter,
        buffer: Arc<Mutex<Vec<serde_json::Value>>>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        // Connect (retry until the .events server is up).
        let deadline = Instant::now() + DEFAULT_CONNECT_TIMEOUT;
        let client = loop {
            match ClientOptions::new().open(&pipe) {
                Ok(c) => break c,
                Err(e)
                    if matches!(
                        e.raw_os_error(),
                        Some(ERROR_FILE_NOT_FOUND) | Some(ERROR_PIPE_BUSY)
                    ) =>
                {
                    if Instant::now() >= deadline {
                        return Err(anyhow!("timed out connecting to events pipe {pipe}: {e}"));
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                Err(e) => return Err(anyhow!("opening events pipe {pipe}: {e}")),
            }
        };

        let (reader_half, mut writer_half) = tokio::io::split(client);
        let mut reader = BufReader::new(reader_half);

        // Subscribe.
        let mut sub = filter.to_subscribe_line();
        sub.push('\n');
        writer_half
            .write_all(sub.as_bytes())
            .await
            .with_context(|| format!("subscribing on events pipe {pipe}"))?;
        writer_half.flush().await.ok();

        // Read the subscribe ack (one Reply line).
        let mut ack = String::new();
        let n = reader
            .read_line(&mut ack)
            .await
            .with_context(|| format!("reading subscribe ack on {pipe}"))?;
        if n == 0 {
            return Err(anyhow!("events pipe {pipe} closed before subscribe ack"));
        }
        let ack_reply = crate::wire::Reply::from_line(ack.trim())
            .with_context(|| format!("parsing subscribe ack on {pipe}: {}", ack.trim()))?;
        if !ack_reply.is_ok() {
            return Err(anyhow!("events subscribe refused on {pipe}: {ack_reply:?}"));
        }

        // Tail: each subsequent line is a raw Event JSON value.
        let task = tokio::spawn(async move {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // pipe closed (node shut down)
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                            buffer.lock().await.push(v);
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(task)
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;

    pub(super) async fn spawn_collector(
        pipe: String,
        _filter: Filter,
        _buffer: Arc<Mutex<Vec<serde_json::Value>>>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        Err(anyhow::anyhow!(
            "events pipe client is Windows-only (pipe {pipe})"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_filter_subscribe_line_has_empty_args() {
        let line = Filter::all().to_subscribe_line();
        assert_eq!(line, r#"{"args":{},"cmd":"subscribe"}"#);
    }

    #[test]
    fn space_filter_subscribe_line_includes_spaces() {
        let f = Filter {
            spaces: vec!["xgen://hash/sha256:S".to_string()],
            event_types: vec![],
        };
        let line = f.to_subscribe_line();
        assert!(line.contains(r#""spaces":["xgen://hash/sha256:S"]"#));
        assert!(line.contains(r#""cmd":"subscribe""#));
    }
}
